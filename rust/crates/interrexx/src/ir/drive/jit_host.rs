//! The driver's side of [`crate::jit`]: which flat loops get compiled, the
//! [`Pass`] a loop's body compiles to, and the [`Host`] that code calls back
//! into.
//!
//! **Compiled code runs the driver's own steps, in the driver's order.** The
//! pass boundary is [`Interp::flat_loop_step_top`] counted exactly as
//! `Op::LoopNext` counts it. A clause opens either here, through the same
//! `enter_stepped_clause` `open_clause!` calls, or in compiled code, which
//! does it only while [`Driven::steady`] holds: then every write
//! `enter_stepped_clause` would make is one to the countdown and the clause
//! state, which compiled code makes in [`Native`] and [`Host::enter`] copies
//! across before any Rust code can read them, and every other write it would
//! make is to a value it already has. A clause closes through
//! `finish_plain_clause` or `leave_stepped_clause` as `clause_region!` closes
//! one, except that one compiled code opened and that called nothing closes
//! with no call at all: nothing ran that could have queued anything at its
//! boundary or pushed a temporary. A simple variable's read and write go to
//! its slot directly through the window `RootSet::top_window` gives, which is
//! what `Op::Load`'s and `Op::Store`'s own fast paths reach, and every other
//! op but a constant and the small-integer fast path of `+`, `-` and `*` is
//! run by `region_ops!` itself, one op at a time. Every state compiled code
//! does not handle hands the driver the op it would have run next.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use interrexx_parse::Operator;

use super::*;
use crate::jit::layout::{self, Native};
use crate::jit::{self, Arith, BRANCH, CONTINUE, Compare, FAST, Host, JUMP, Pass, STOP, Step};
use crate::run::FlatStep;
use crate::run::QuickKind;

/// Passes a loop runs driven before it is compiled. Compiling a body and the
/// saving on each pass of it both grow with its length, so the passes that
/// earn a compile back are about the same for any body; this many bounds what
/// a loop that ends just after it is compiled pays for it at about the time it
/// already spent driven.
const THRESHOLD: u32 = 7168;
/// A body is compiled only when it has at least this many steps compiled code
/// does itself for each op the host runs.
const HOSTED_SHARE: usize = 8;
/// Passes a compiled loop runs before its hosted ops are weighed.
const RETIRE_AFTER: u64 = 16;
/// A loop whose body has an op the compiled code does not take.
const REFUSED: u32 = u32::MAX;

/// Per `DO`/`LOOP` instruction of one chunk: its pass count while it is
/// driven, then where its compiled code is.
pub(crate) struct Loops {
    /// Below [`THRESHOLD`], passes so far; [`THRESHOLD`] plus `k`, the `k`th
    /// entry of `compiled`; [`REFUSED`], never compiled.
    state: Box<[Cell<u32>]>,
    /// Each compiled loop with its budget: how many ops per pass the host
    /// may run before the loop is retired.
    compiled: RefCell<Vec<(Rc<jit::Compiled>, u64)>>,
}

impl Loops {
    /// Keeps every loop of this chunk from being compiled.
    pub(crate) fn refuse_all(&self) {
        for state in &self.state {
            state.set(REFUSED);
        }
    }

    /// Counts a pass of the loop at instruction `do_index`, answering whether
    /// it is hot: compiled, or due to be. Asked in the `Op::LoopNext` arm
    /// itself, so a loop that is still being counted, or was refused, costs
    /// no call into [`Interp::jit_passes`].
    #[inline(always)]
    pub(super) fn hot(&self, do_index: u32) -> bool {
        let Some(state) = self.state.get(do_index as usize) else {
            return false;
        };
        match state.get() {
            REFUSED => false,
            passes if passes < THRESHOLD - 1 => {
                state.set(passes + 1);
                false
            }
            _ => true,
        }
    }

    pub(crate) fn new(instructions: usize) -> Loops {
        Loops {
            state: (0..instructions).map(|_| Cell::new(0)).collect(),
            compiled: RefCell::new(Vec::new()),
        }
    }
}

/// What the `Op::LoopNext` arm does after compiled passes: carry on at an op,
/// end the range with what `ops_loop` returns, or settle a `Flow` from an op.
pub(super) enum JitAction {
    At(u32),
    Return(Result<Exit, Failure>),
    Settle(Flow, u32),
}

/// How a run of compiled passes ended.
enum JitExit {
    /// Drive on from this op.
    At(u32),
    /// The slice ended before the clause, or the pass boundary, at this op.
    Slice(u32),
    /// The loop is over and this is its answer.
    Done(Flow),
    /// A clause answered this `Flow`, settled from this op.
    Flow(Flow, u32),
    Fail(Failure),
    /// A `SELECT`'s close let this `Flow` escape the range being driven.
    Escaped(Flow),
    /// A call op parked the open clause for its callee: what
    /// `clause_region!` hands `Interp::park_call`.
    Parked {
        park: Park,
        entry: SteppedClause,
        clause_pc: u32,
        end: u32,
        index: usize,
    },
}

/// The clause a compiled pass has open.
struct Open {
    pc: u32,
    index: usize,
    end: u32,
    entry: SteppedClause,
}

struct Driven<'a, 'c> {
    interp: &'a mut Interp,
    code: &'a Code<'c>,
    chunk: &'a Chunk,
    registers: RegFrame<'a>,
    source: Option<&'a ProgramSource>,
    top: bool,
    /// The `Op::LoopNext` this loop's passes end at.
    loop_pc: u32,
    do_index: usize,
    body_start: u32,
    open: Option<Open>,
    /// The driver's frame base and the instruction range it drives, which a
    /// `SELECT`'s close settles against.
    range: (usize, usize, usize),
    /// Where the last [`JUMP`] answer goes on.
    jump_to: u32,
    /// Whether [`Host::leave`] works out the loop's quick pass.
    ask_loop: bool,
    /// The settings the quick pass in the block was worked out under, while
    /// it stands.
    loop_key: Option<(u64, u64, crate::trace::TraceMode)>,
    /// Ops per pass the host may run, and the ops it has run: a loop whose
    /// values keep the compiled code's own paths from answering costs more
    /// compiled than driven, so past its budget it is retired.
    budget: u64,
    hosted: u64,
    /// Whether the loop is being retired: it leaves for the driver at the
    /// next clause opening or pass boundary and is not compiled again.
    retire: bool,
    /// Whether the clause's last op closed it, so its `Step::Close` has
    /// nothing left to do.
    closed: bool,
    exit: Option<JitExit>,
}

impl Interp {
    /// Runs the passes of the loop whose `Op::LoopNext` is at `pc`, compiled,
    /// or `None` while it is driven: not yet hot, or refused.
    #[expect(
        clippy::too_many_arguments,
        reason = "the values the Op::LoopNext arm holds"
    )]
    // Cold as well as out of line: the `Op::LoopNext` arm is in the driver's
    // own loop, and a call it makes once per loop entry is not one the loop
    // should be laid out around.
    #[cold]
    #[inline(never)]
    pub(super) fn jit_passes(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        source: Option<&ProgramSource>,
        pc: u32,
        do_index: u32,
        top: bool,
        range: (usize, usize, usize),
    ) -> Option<JitAction> {
        let state = chunk.jit.state.get(do_index as usize)?;
        let compiled = match state.get() {
            REFUSED => return None,
            passes if passes < THRESHOLD - 1 => return None,
            passes if passes == THRESHOLD - 1 => {
                let generation = self.routine_generation;
                let Some((compiled, budget)) = pass_of(code, chunk, pc, do_index, generation)
                    .and_then(|(pass, budget)| Some((jit::compile(&pass)?, budget)))
                else {
                    state.set(REFUSED);
                    return None;
                };
                let compiled = Rc::new(compiled);
                let mut all = chunk.jit.compiled.borrow_mut();
                state.set(THRESHOLD + all.len() as u32);
                all.push((Rc::clone(&compiled), budget));
                (compiled, budget)
            }
            at => {
                let all = chunk.jit.compiled.borrow();
                let (compiled, budget) = all.get((at - THRESHOLD) as usize)?;
                (Rc::clone(compiled), *budget)
            }
        };
        let (compiled, budget) = compiled;
        let body_start = chunk.op_at(do_index as usize + 1)?;
        let mut driven = Driven {
            interp: self,
            code,
            chunk,
            registers,
            source,
            top,
            loop_pc: pc,
            do_index: do_index as usize,
            body_start,
            open: None,
            range,
            jump_to: 0,
            ask_loop: true,
            loop_key: None,
            budget,
            hosted: 0,
            retire: false,
            closed: false,
            exit: None,
        };
        #[cfg(test)]
        COMPILED_RUNS.with(|runs| runs.set(runs.get() + 1));
        let answer = compiled.run(registers, &mut driven);
        debug_assert_eq!(answer, STOP, "compiled passes end only on a STOP");
        debug_assert!(
            driven.open.is_none(),
            "compiled passes ended with a clause open"
        );
        let exit = driven
            .exit
            .unwrap_or_else(|| JitExit::Fail(Loud::op_not_driven("a compiled pass").into()));
        // Everything but where the driver goes on is settled here, out of
        // the driver's loop: the `Op::LoopNext` arm's own answers, and
        // `clause_region!`'s for a parked call.
        Some(match exit {
            JitExit::At(target) => JitAction::At(target),
            JitExit::Slice(at) => JitAction::Return(Ok(Exit::Slice(at))),
            JitExit::Done(flow) => {
                self.activity.frames.pop();
                JitAction::Settle(flow, pc)
            }
            JitExit::Flow(flow, next) => JitAction::Settle(flow, next),
            JitExit::Fail(failure) => JitAction::Return(Err(failure)),
            JitExit::Escaped(flow) => JitAction::Return(Ok(Exit::Flow(flow))),
            JitExit::Parked {
                park,
                entry,
                clause_pc,
                end,
                index,
            } => {
                self.park_call(park, entry, clause_pc, end, index, false, false);
                JitAction::Return(Ok(Exit::Parked))
            }
        })
    }
}

/// The body of the loop whose `Op::DO` is instruction `do_index` and whose
/// `Op::LoopNext` is at `loop_pc`, as a [`Pass`], or `None` where it holds
/// anything but plain clause regions of value ops and stores.
fn pass_of(
    code: &Code<'_>,
    chunk: &Chunk,
    loop_pc: u32,
    do_index: u32,
    generation: u32,
) -> Option<(Pass, u64)> {
    let body_start = chunk.op_at(do_index as usize + 1)?;
    if body_start >= loop_pc {
        return None;
    }
    // A condition that calls or sends is a test region in the stream, before
    // the body (`Chunk::loop_test_at`), which the pass boundary hands to the
    // driver: compiled passes would leave for it on every pass, and enter
    // again at the `LoopNext`.
    if chunk.loop_test_at(do_index as usize).is_some() {
        return None;
    }
    let mut steps = Vec::new();
    let mut pc = body_start;
    // An `IF`'s branches stay inside the body, and its `EndBranch` and
    // `Jump` sit between clause regions.
    let inside = |target: u32| (body_start..=loop_pc).contains(&target);
    while pc < loop_pc {
        let (index, end) = match *chunk.ops_in(pc, pc + 1)?.first()? {
            Op::Clause { index, end } | Op::CallingClause { index, end } => (index, end),
            Op::EndBranch => {
                steps.push(Step::EndBranch { pc });
                pc += 1;
                continue;
            }
            // A `SELECT`'s own ops, which the host runs as the driver does.
            Op::EnterWhen { .. } | Op::EnterOtherwise { .. } | Op::EndWhen => {
                steps.push(Step::Between { pc });
                pc += 1;
                continue;
            }
            Op::Jump { target } if inside(target) => {
                steps.push(Step::Jump { target });
                pc += 1;
                continue;
            }
            _ => return None,
        };
        if end <= pc || end > loop_pc {
            return None;
        }
        let opening = chunk.opening_at(index as usize);
        steps.push(Step::Open {
            pc,
            tabled: opening.tabled,
            register_top: opening.register_top,
        });
        // The `Step::PushArg`s whose arguments no op has taken yet, as indices
        // into `steps`. Every op that takes pushed arguments takes the
        // `argc` pushed last, so a call's own are the last `argc` here.
        let mut pending: Vec<usize> = Vec::new();
        for (at, op) in (pc + 1..).zip(chunk.ops_in(pc + 1, end)?) {
            match *op {
                // A call its site has resolved to a quiet builtin, with few
                // enough arguments to carry: its pushes are dropped, and the
                // call reads their registers itself (`Host::quiet_builtin`),
                // which nothing writes before the call, since the compiler
                // releases them only after it.
                Op::CallArgs { site, argc, .. }
                    if let Some(Resolved::Builtin(target)) =
                        chunk.resolved_call(site, generation)
                        && crate::builtin::quiet(target)
                        && usize::from(argc) <= jit::CARRIED_ARGS
                        && usize::from(argc) <= pending.len() =>
                {
                    let mut args = jit::NO_ARGS;
                    let from = pending.len() - usize::from(argc);
                    for (k, index) in pending.drain(from..).enumerate() {
                        let Step::PushArg { src } = steps[index] else {
                            return None;
                        };
                        steps[index] = Step::Dropped;
                        args = jit::carry_arg(args, k, src);
                    }
                    steps.push(Step::Builtin { pc: at, args });
                    continue;
                }
                // A `do; ... end` block, unlabelled: its start runs on the
                // host, whose region ends at the clause's own end, where
                // compiled code carries on into the block's clauses.
                Op::LoopRun { index }
                    if matches!(
                        code.body.instructions.get(index as usize).map(|clause| &clause.kind),
                        Some(InstructionKind::Do(body) | InstructionKind::Loop(body))
                            if matches!(body.kind, interrexx_parse::LoopKind::Simple)
                                && body.label.is_none()
                    ) =>
                {
                    pending.clear();
                    steps.push(Step::BlockStart { pc: at });
                    continue;
                }
                // The `END` of such a block answers `Flow::Next` and does
                // nothing else; its clause, which counts it, stays.
                Op::Exec { index }
                    if matches!(
                        code.body.instructions.get(index as usize).map(|clause| &clause.kind),
                        Some(InstructionKind::End {
                            closes: Some(closes),
                            ..
                        }) if matches!(closes.style, interrexx_parse::EndStyle::Do)
                    ) =>
                {
                    continue;
                }
                Op::PushArg { src } => {
                    pending.push(steps.len());
                    steps.push(Step::PushArg { src });
                    continue;
                }
                Op::CallArgs { argc, .. }
                | Op::CallNamed { argc, .. }
                | Op::Send { argc, .. }
                | Op::SendWith { argc, .. }
                | Op::List { argc, .. } => {
                    let left = pending.len().saturating_sub(usize::from(argc));
                    pending.truncate(left);
                }
                _ => {}
            }
            let step = match *op {
                Op::Const { dst, konst } => match chunk.interned_konst(konst) {
                    ObjRef::NIL => Step::Op { pc: at },
                    value => Step::Const {
                        dst,
                        bits: value.bits(),
                    },
                },
                Op::LoadConstant { symbol, dst } => match chunk.interned_symbol(symbol) {
                    ObjRef::NIL => Step::Op { pc: at },
                    value => Step::Const {
                        dst,
                        bits: value.bits(),
                    },
                },
                // A site whose small-integer path has already declined
                // (`Chunk::tries_small_int`) is one the host will run.
                Op::Arith { hint, .. } if !chunk.tries_small_int(hint) => Step::Op { pc: at },
                Op::Arith {
                    op, lhs, rhs, dst, ..
                } => match op {
                    Operator::Plus => Step::Arith {
                        pc: at,
                        op: Arith::Add,
                        lhs,
                        rhs,
                        dst,
                    },
                    Operator::Subtract => Step::Arith {
                        pc: at,
                        op: Arith::Subtract,
                        lhs,
                        rhs,
                        dst,
                    },
                    Operator::Multiply => Step::Arith {
                        pc: at,
                        op: Arith::Multiply,
                        lhs,
                        rhs,
                        dst,
                    },
                    Operator::IntDiv => Step::Arith {
                        pc: at,
                        op: Arith::IntDiv,
                        lhs,
                        rhs,
                        dst,
                    },
                    Operator::Remainder => Step::Arith {
                        pc: at,
                        op: Arith::Remainder,
                        lhs,
                        rhs,
                        dst,
                    },
                    _ => Step::Op { pc: at },
                },
                // A comparison the compiler did not route to the string rule.
                Op::Binary {
                    op,
                    lhs,
                    rhs,
                    dst,
                    text: false,
                    ..
                } if let Some(op) = quick_compare(op) => Step::Compare {
                    pc: at,
                    op,
                    lhs,
                    rhs,
                    dst,
                },
                // The region's last op, so the clause's close follows it.
                Op::ConditionJump { reg, target, .. } if at + 1 == end && inside(target) => {
                    Step::Branch {
                        pc: at,
                        reg,
                        target,
                    }
                }
                Op::Load {
                    read: SymbolRead::Simple,
                    at: slot,
                    dst,
                    ..
                } if let Some(slot) = slot.resolved()
                    && let Ok(slot) = u32::try_from(slot) =>
                {
                    Step::Load { pc: at, slot, dst }
                }
                Op::Store { at: slot, src, .. }
                    if let Some(slot) = slot.resolved()
                        && let Ok(slot) = u32::try_from(slot) =>
                {
                    Step::Store { pc: at, slot, src }
                }
                // A compound's read and its write: one call with no bracket
                // where that holds (`Host::quiet_compound`).
                Op::Load {
                    read: SymbolRead::Compound | SymbolRead::CompoundShared,
                    ..
                } => Step::Compound { pc: at },
                Op::Store { index, .. }
                    if matches!(
                        code.body.instructions.get(index as usize).map(|clause| &clause.kind),
                        Some(InstructionKind::Assignment { target, .. })
                            if matches!(target.kind, ExprKind::Compound(_))
                    ) =>
                {
                    Step::Compound { pc: at }
                }
                // Ops that neither jump nor trace, which `general` runs; one
                // that parks the clause for a callee hands it to the driver.
                Op::PushArg { src } => Step::PushArg { src },
                Op::Load { .. }
                | Op::Binary { .. }
                | Op::Prefix { .. }
                | Op::Store { .. }
                | Op::Exec { .. }
                | Op::CallArgs { .. }
                | Op::CallExpr { .. }
                | Op::CallNamed { .. }
                | Op::Call { .. }
                | Op::Send { .. }
                | Op::SendWith { .. }
                | Op::SendScope { .. }
                | Op::Reference { .. }
                | Op::ClassAt { .. }
                | Op::SelectCase { .. }
                | Op::ExecValue { .. }
                | Op::List { .. }
                | Op::Say { .. }
                | Op::Queue { .. }
                | Op::ParseBegin { .. }
                | Op::ParseRun { .. }
                | Op::Message { .. }
                | Op::EvalExpr { .. }
                | Op::Escape { .. }
                | Op::Return { .. }
                | Op::Signal { .. } => Step::Op { pc: at },
                _ => return None,
            };
            // Any other op the host runs might take pushed arguments the
            // pairing does not see: the pushes before it stay pushes.
            if matches!(step, Step::Op { .. })
                && !matches!(
                    op,
                    Op::CallArgs { .. }
                        | Op::CallNamed { .. }
                        | Op::Send { .. }
                        | Op::SendWith { .. }
                        | Op::List { .. }
                )
            {
                pending.clear();
            }
            steps.push(step);
        }
        steps.push(Step::Close { next: end });
        pc = end;
    }
    steps.retain(|step| !matches!(step, Step::Dropped));
    // **Compiled only where it pays.** An op the host runs costs a call, the
    // block written back and read again, and the driver's own arm, which is
    // several times what the driver spends dispatching it; a body made mostly
    // of those runs slower compiled than driven.
    let hosted = steps
        .iter()
        .filter(|step| matches!(step, Step::Op { .. } | Step::Between { .. }))
        .count();
    let native = steps
        .iter()
        .filter(|step| {
            !matches!(
                step,
                Step::Op { .. }
                    | Step::Between { .. }
                    | Step::Builtin { .. }
                    | Step::Compound { .. }
                    | Step::Open { .. }
                    | Step::Close { .. }
            )
        })
        .count();
    if hosted * HOSTED_SHARE > native {
        return None;
    }
    let budget = (native / HOSTED_SHARE).max(1) as u64;
    Some((
        Pass {
            steps,
            registers: chunk.registers,
            end: loop_pc,
        },
        budget,
    ))
}

/// The comparisons [`Step::Compare`] answers, by `compare_op`'s mapping: the
/// non-strict ones, which `small_int_compare` settles for two small integers.
fn quick_compare(op: Operator) -> Option<Compare> {
    Some(match op {
        Operator::Equal => Compare::Equal,
        Operator::BackslashEqual
        | Operator::LessThanGreaterThan
        | Operator::GreaterThanLessThan => Compare::NotEqual,
        Operator::GreaterThan => Compare::Greater,
        Operator::LessThan => Compare::Less,
        Operator::GreaterThanEqual | Operator::BackslashLessThan => Compare::GreaterEqual,
        Operator::LessThanEqual | Operator::BackslashGreaterThan => Compare::LessEqual,
        _ => return None,
    })
}

impl Driven<'_, '_> {
    /// Leaves for the driver at `pc`, a clause opening or the pass boundary,
    /// and keeps the loop from being compiled again.
    fn retired(&mut self, pc: u32) -> u32 {
        if let Some(state) = self.chunk.jit.state.get(self.do_index) {
            state.set(REFUSED);
        }
        self.stop(JitExit::At(pc))
    }

    fn stop(&mut self, exit: JitExit) -> u32 {
        self.exit = Some(exit);
        STOP
    }

    /// Closes the open clause around `ran`: `clause_region!`'s path for a
    /// region that did not end plainly.
    fn leave(&mut self, ran: Result<RegionEnd, Failure>) -> u32 {
        self.leave_from(ran, false)
    }

    /// [`Driven::leave`], carrying on in compiled code where `in_place` and
    /// the clause closed with the `Flow::Next` its region answered.
    fn leave_from(&mut self, ran: Result<RegionEnd, Failure>, in_place: bool) -> u32 {
        let Some(open) = self.open.take() else {
            return self.stop(JitExit::Fail(
                Loud::op_not_driven("a close with no clause").into(),
            ));
        };
        let Some(clause) = self.code.body.instructions.get(open.index) else {
            return self.stop(JitExit::Fail(Loud::chunk_map_too_short().into()));
        };
        // `open` hands a debugging clause to the driver, so this is the
        // driver's own test with its `debugging` false.
        let ran = if self.interp.activity.debug_stop_owed.is_some() {
            self.interp.debug_pause_in_region(
                ran,
                self.code,
                open.index,
                clause,
                self.source,
                open.pc,
            )
        } else {
            ran
        };
        let exit = match self.interp.leave_stepped_clause(
            open.entry,
            self.code,
            open.index,
            clause,
            self.source,
            ran,
        ) {
            Ok(ClauseOutcome::Ran(Ok(RegionEnd::At(next)))) if in_place && next == open.end => {
                #[cfg(test)]
                HANDED_BACK.with(|handed| handed.set(handed.get() + 1));
                self.closed = true;
                return CONTINUE;
            }
            Ok(ClauseOutcome::Ran(Ok(RegionEnd::At(next)))) => JitExit::At(next),
            Ok(ClauseOutcome::Ran(Ok(RegionEnd::Flowed(Flow::Next)))) if in_place => {
                self.closed = true;
                return CONTINUE;
            }
            Ok(ClauseOutcome::Ran(Ok(RegionEnd::Flowed(flow)))) => JitExit::Flow(flow, open.end),
            Ok(ClauseOutcome::Ran(Err(failure))) | Err(failure) => JitExit::Fail(failure),
            Ok(ClauseOutcome::Ended(exit)) => JitExit::Flow(Flow::Exit(exit.value()), open.pc),
        };
        self.stop(exit)
    }
}

impl Host for Driven<'_, '_> {
    fn pass(&mut self) -> u32 {
        if self.retire {
            return self.retired(self.loop_pc);
        }
        self.ask_loop = true;
        let counted = match self.interp.count_clause_against_deadline(self.top) {
            Ok(counted) => counted,
            Err(Failure::Slice) => return self.stop(JitExit::Slice(self.loop_pc)),
            Err(failure) => return self.stop(JitExit::Fail(failure)),
        };
        match self.interp.flat_loop_step_top(
            self.code,
            self.source,
            self.chunk,
            self.registers,
            Flow::Next,
            |_| Ok(counted),
        ) {
            Ok(FlatStep::Body(at)) if at == self.body_start => CONTINUE,
            Ok(FlatStep::Body(at)) => self.stop(JitExit::At(at)),
            Ok(FlatStep::Done(flow)) => self.stop(JitExit::Done(flow)),
            Err(failure) => self.stop(JitExit::Fail(failure)),
        }
    }

    fn open(&mut self, pc: u32) -> u32 {
        if self.retire {
            return self.retired(pc);
        }
        let Some(&(Op::Clause { index, end } | Op::CallingClause { index, end })) =
            self.chunk.ops_in(pc, pc + 1).and_then(<[Op]>::first)
        else {
            return self.stop(JitExit::Fail(
                Loud::op_not_driven("a compiled clause").into(),
            ));
        };
        // A clause whose echo the chunk no longer answers for, or one under
        // interactive debug, is the driver's: asked before the clause is
        // counted, so the driver counts it once.
        let sink = self.interp.chunk_trace();
        if self.chunk.trace().clause_echoes() != sink.clause_echoes() || sink.debugging() {
            return self.stop(JitExit::At(pc));
        }
        let counted = match self.interp.count_clause_against_deadline(self.top) {
            Ok(counted) => counted,
            Err(Failure::Slice) => return self.stop(JitExit::Slice(pc)),
            Err(failure) => return self.stop(JitExit::Fail(failure)),
        };
        let index = index as usize;
        let Some(clause) = self.code.body.instructions.get(index) else {
            return self.stop(JitExit::Fail(Loud::chunk_map_too_short().into()));
        };
        let opening = self.chunk.opening_at(index);
        self.registers.clear_from(opening.register_top);
        let entry = self.interp.enter_stepped_clause(
            Echo::Compiled,
            self.code,
            index,
            clause,
            self.source,
            (opening.tabled, opening.position),
            counted,
        );
        self.interp.activity.region_procedure_permitted = if self.top {
            std::mem::take(&mut self.interp.activity.procedure_permitted)
        } else {
            false
        };
        self.open = Some(Open {
            pc,
            index,
            end,
            entry,
        });
        CONTINUE
    }

    fn close(&mut self, next: u32) -> u32 {
        if std::mem::take(&mut self.closed) {
            return CONTINUE;
        }
        if self.interp.activity.pending_traps.is_empty() && !self.interp.clause_exits {
            let Some(open) = self.open.take() else {
                return self.stop(JitExit::Fail(
                    Loud::op_not_driven("a close with no clause").into(),
                ));
            };
            self.interp.finish_plain_clause(open.entry);
            return CONTINUE;
        }
        self.leave(Ok(RegionEnd::At(next)))
    }

    fn op(&mut self, pc: u32) -> u32 {
        self.hosted += 1;
        // Between clauses: the driver's own arms for these ops.
        match self.chunk.ops_in(pc, pc + 1).and_then(<[Op]>::first) {
            Some(Op::EndBranch) => {
                return match self.interp.end_promoted_branch(Flow::Next) {
                    Ok(Flow::Next) => CONTINUE,
                    Ok(flow) => self.stop(JitExit::Flow(flow, pc + 1)),
                    Err(failure) => self.stop(JitExit::Fail(failure)),
                };
            }
            Some(&Op::EnterWhen { select, when }) => {
                return match self.interp.when_frame(
                    self.code,
                    self.chunk,
                    select as usize,
                    when as usize,
                ) {
                    Ok(frame) => {
                        self.interp.activity.frames.push(Frame::select(frame));
                        CONTINUE
                    }
                    Err(failure) => self.stop(JitExit::Fail(failure)),
                };
            }
            Some(&Op::EnterOtherwise { select }) => {
                return match self
                    .interp
                    .otherwise_frame(self.code, self.chunk, select as usize)
                {
                    Ok(frame) => {
                        self.interp.activity.frames.push(Frame::select(frame));
                        CONTINUE
                    }
                    Err(failure) => self.stop(JitExit::Fail(failure)),
                };
            }
            Some(Op::EndWhen) => {
                let (base, start, end) = self.range;
                return match self.interp.leave_ended_select_branch(
                    self.code,
                    self.chunk,
                    self.registers,
                    base,
                    pc,
                    start,
                    end,
                    self.source,
                ) {
                    Ok(BranchEnd::At(target)) => {
                        self.jump_to = target;
                        self.exit = Some(JitExit::At(target));
                        JUMP
                    }
                    Ok(BranchEnd::Escaped(flow)) => self.stop(JitExit::Escaped(flow)),
                    // The driver's `(Flow::Next, pc + 1)`, settled.
                    Ok(BranchEnd::None) => CONTINUE,
                    Err(failure) => self.stop(JitExit::Fail(failure)),
                };
            }
            // A call the site already resolved to a builtin, which runs to
            // completion: the driver's own `begin_call_args`, without the
            // region around it.
            Some(&Op::CallArgs {
                slot,
                path,
                site,
                argc,
                dst,
            }) if self.top
                && matches!(
                    self.chunk
                        .resolved_call(site, self.interp.routine_generation),
                    Some(Resolved::Builtin(_))
                ) =>
            {
                let Some(open) = &self.open else {
                    return self.stop(JitExit::Fail(
                        Loud::op_not_driven("an op with no clause").into(),
                    ));
                };
                let Some(clause) = self.code.body.instructions.get(open.index) else {
                    return self.stop(JitExit::Fail(Loud::chunk_map_too_short().into()));
                };
                return match self
                    .interp
                    .begin_call_args(self.code, self.chunk, clause, slot, path, site, argc)
                {
                    Ok(Started::Ran(value)) => {
                        self.registers.set(dst, value);
                        CONTINUE
                    }
                    Ok(Started::Entered) => self.stop(JitExit::Fail(
                        Loud::op_not_driven("a builtin call that entered a body").into(),
                    )),
                    Err(failure) => self.leave(Err(failure)),
                };
            }
            _ => {}
        }
        // `Op::Load`'s and `Op::Store`'s own fast paths for a simple
        // variable whose slot the plan resolved, taken here for the same
        // reason they are taken there; everything else is `general`.
        match self.chunk.ops_in(pc, pc + 1).and_then(<[Op]>::first) {
            Some(&Op::Load {
                symbol,
                read: SymbolRead::Simple,
                at,
                dst,
            }) if let Some(slot) = at.resolved() => {
                let (value, novalue) = self.interp.read_slot(self.code, symbol, slot);
                if let Err(failure) = self.interp.novalue_check(novalue, value) {
                    return self.leave(Err(failure));
                }
                self.registers.set(dst, value);
                CONTINUE
            }
            Some(&Op::Store { at, src, .. })
                if let Some(slot) = at.resolved()
                    && !self.interp.trace_mode().results =>
            {
                let value = self.registers.get(src);
                let frame = self.interp.activation().frame;
                self.interp.set_variable(frame, slot, value);
                CONTINUE
            }
            _ => self.general(pc),
        }
    }

    fn quiet_builtin(&mut self, pc: u32, args: u64) -> bool {
        // The checks `begin_call_args` makes for a resolved builtin, and the
        // two the quiet path adds: the builtin is quiet, and no argument is
        // an object whose string value runs Rexx code.
        let Some(&Op::CallArgs {
            site, argc, dst, ..
        }) = self.chunk.ops_in(pc, pc + 1).and_then(<[Op]>::first)
        else {
            return false;
        };
        if !self.top {
            return false;
        }
        let interp = &mut *self.interp;
        let Some(Resolved::Builtin(target)) =
            self.chunk.resolved_call(site, interp.routine_generation)
        else {
            return false;
        };
        // A setting that shows values is the full path's, which has the
        // trace ops. Compiled code does not reach here under one: a clause
        // opened after `TRACE` changes hands the loop to the driver, and the
        // chunk compiled for the clause it changed in has no trace ops. The
        // check keeps that from being what this rests on.
        let mode = interp.trace_mode();
        if !crate::builtin::quiet(target) || mode.results || mode.intermediates {
            return false;
        }
        let argc = usize::from(argc);
        let mut values = [None; jit::CARRIED_ARGS];
        for (k, value) in values.iter_mut().enumerate().take(argc) {
            let src = jit::carried_arg(args, k);
            if src != jit::OMITTED {
                let arg = self.registers.get(src);
                if !interp.plain_string(arg) {
                    return false;
                }
                *value = Some(arg);
            }
        }
        let probe = 0u8;
        // A probe that trips is the full path's to raise.
        if interp.enter_eval_node(&raw const probe).is_err() {
            return false;
        }
        // A failure is raised again by the full path, from a clause the
        // interpreter holds: a quiet builtin answers the same failure for the
        // same arguments.
        // The temporaries the builtin roots are its own: released here, as
        // the clause around a call would release them, since a clause
        // compiled code opened has no temporaries frame of its own to close.
        // The answer is rooted in its register from here.
        let frame = interp.roots.activity_mut().push_frame();
        let ran = crate::builtin::run(interp, b"", target, &values[..argc]);
        interp.roots.activity_mut().pop_frame(frame);
        interp.activity.depth -= 1;
        match ran {
            Ok(value) => {
                self.registers.set(dst, value);
                #[cfg(test)]
                QUIET_CALLS.with(|calls| calls.set(calls.get() + 1));
                true
            }
            Err(_) => false,
        }
    }

    fn quiet_compound(&mut self, pc: u32) -> bool {
        let interp = &mut *self.interp;
        // A setting that shows values is the full path's, as for a builtin.
        // No test reaches this: a loop whose setting starts to show values
        // leaves compiled code at its next clause, as for a builtin.
        let mode = interp.trace_mode();
        if mode.results || mode.intermediates {
            return false;
        }
        match self.chunk.ops_in(pc, pc + 1).and_then(<[Op]>::first) {
            Some(&Op::Load {
                symbol, read, dst, ..
            }) => {
                let shared = match read {
                    SymbolRead::Compound => false,
                    SymbolRead::CompoundShared => true,
                    _ => return false,
                };
                match interp.read_compound_quiet(self.code, symbol, shared) {
                    Some(value) => {
                        self.registers.set(dst, value);
                        #[cfg(test)]
                        QUIET_COMPOUNDS.with(|accesses| accesses.set(accesses.get() + 1));
                        true
                    }
                    None => false,
                }
            }
            Some(&Op::Store {
                index, src, shared, ..
            }) => {
                let Some(InstructionKind::Assignment { target, .. }) = self
                    .code
                    .body
                    .instructions
                    .get(index as usize)
                    .map(|clause| &clause.kind)
                else {
                    return false;
                };
                let ExprKind::Compound(id) = target.kind else {
                    return false;
                };
                // The key the write takes: the one the read before it holds,
                // or a tail whose pieces are plain.
                if !(shared && interp.shared_key_for == Some(id)) {
                    let mut key = interp.take_key_buffer();
                    let plain = interp.quiet_tail_key_into(self.code, id, &mut key);
                    interp.give_key_buffer(key);
                    if !plain {
                        return false;
                    }
                }
                let value = self.registers.get(src);
                // The value rooted for the write is released with it, as for
                // a builtin's temporaries.
                let frame = interp.roots.activity_mut().push_frame();
                let wrote = interp.assign_evaluated(self.code, target, value, None, shared);
                interp.roots.activity_mut().pop_frame(frame);
                // A write that failed is made again by the full path, which
                // raises its failure; a second write of the same value to the
                // same tail is the first.
                #[cfg(test)]
                if wrote.is_ok() {
                    QUIET_COMPOUNDS.with(|accesses| accesses.set(accesses.get() + 1));
                }
                wrote.is_ok()
            }
            _ => false,
        }
    }

    fn stage_args(&mut self, pc: u32, args: u64) {
        let Some(&Op::CallArgs { argc, .. }) =
            self.chunk.ops_in(pc, pc + 1).and_then(<[Op]>::first)
        else {
            return;
        };
        for k in 0..usize::from(argc) {
            self.push_arg(jit::carried_arg(args, k));
        }
    }

    fn refresh_holds(&mut self, native: &Native) -> bool {
        let mut fresh = Native::new(native.registers);
        // `leave` with the counters it keeps put back as they were.
        let (retire, ask_loop, loop_key) = (self.retire, self.ask_loop, self.loop_key);
        fresh.passes = native.passes;
        fresh.loop_kind = native.loop_kind;
        fresh.loop_slot = native.loop_slot;
        self.ask_loop = false;
        Host::leave(self, &mut fresh);
        (self.retire, self.ask_loop, self.loop_key) = (retire, ask_loop, loop_key);
        fresh.slots == native.slots
            && fresh.slot_count == native.slot_count
            && fresh.steady == native.steady
            && fresh.small_int_bound == native.small_int_bound
            && fresh.quick_values == native.quick_values
            && fresh.jump_to == native.jump_to
    }

    fn enter(&mut self, native: &mut Native) {
        self.interp.clause_countdown = u32::try_from(native.countdown).unwrap_or(u32::MAX);
        if native.loop_dirty != 0 {
            native.loop_dirty = 0;
            self.interp.quick_passes_reached(native.loop_value);
        }
        match std::mem::replace(&mut native.clause_at, layout::CLAUSE_NONE) {
            layout::CLAUSE_NONE => {}
            // A pass boundary's header clause is the loop's `END`, as the
            // driver's `flat_loop_step` makes it (`Interp::quick_pass` takes
            // only a header blamed on the `END`).
            layout::CLAUSE_HEADER => {
                self.interp.enter_compiled_clause(
                    native.loop_end as usize,
                    native.loop_line as usize,
                    native.loop_indent as usize,
                );
            }
            // A clause compiled code opened, which `pass_of` took only from
            // an `Op::Clause` or `Op::CallingClause` and the chunk's table.
            pc => {
                let op = u32::try_from(pc)
                    .ok()
                    .and_then(|pc| self.chunk.ops_in(pc, pc + 1))
                    .and_then(<[Op]>::first);
                if let Some(&(Op::Clause { index, .. } | Op::CallingClause { index, .. })) = op {
                    let position = self.chunk.opening_at(index as usize).position;
                    let indent = position.indent as usize + self.interp.activity.activation_indent;
                    self.interp.enter_compiled_clause(
                        index as usize,
                        position.line as usize,
                        indent,
                    );
                } else {
                    debug_assert!(
                        false,
                        "compiled code opened a clause at {pc}, which no clause op is"
                    );
                }
            }
        }
    }

    fn leave(&mut self, native: &mut Native) {
        native.jump_to = u64::from(self.jump_to);
        native.countdown = u64::from(self.interp.clause_countdown);
        native.steady = u64::from(self.steady());
        let mode = self.interp.trace_mode();
        let interp = &*self.interp;
        let activation = interp.activation();
        let settings = &activation.settings;
        let digits = settings.digits();
        let fuzz = settings.fuzz();
        native.small_int_bound = crate::value::small_int_bound(digits);
        native.quick_values = u64::from(fuzz == 0 && !mode.results);
        // `Op::Load`'s and `Op::Store`'s fast paths reach the slot only for a
        // frame no `EXPOSE` redirects, and a store only while `TRACE` shows no
        // results; the window is offered only where both hold.
        let frame = activation.frame;
        let exposed = activation.exposed.is_some();
        let window = if !exposed && !mode.results {
            self.interp.roots.top_window(frame)
        } else {
            None
        };
        // Kept as raw parts and not as the slice: the interpreter writes these
        // slots through `RootSet` while compiled code is suspended in a call,
        // and its `Vec` may move them then, so no reference may outlive this
        // call. Generated code reads the parts only between calls, and this
        // runs after every one.
        (native.slots, native.slot_count) = match window {
            Some(window) => (window.as_mut_ptr(), window.len() as u64),
            None => (std::ptr::null_mut(), 0),
        };
        if !self.retire
            && native.passes >= RETIRE_AFTER
            && self.hosted > native.passes * self.budget
        {
            self.retire = true;
        }
        if self.retire || native.steady == 0 {
            // Every opening and boundary goes through the host: a retired
            // loop leaves there, and an unsteady one is the host's to step.
            if self.retire {
                native.steady = 0;
            }
            native.loop_ready = 0;
            self.loop_key = None;
            return;
        }
        // **What the quick pass rests on that a call can change**: `DIGITS`
        // and `FUZZ`, which only this activation's own instructions set, and
        // the trace setting, which a `TRACE()` call also sets. The loop's own
        // state changes only at a pass boundary the host takes, which asks
        // again; the control variable is read from its slot on every pass.
        let key = (digits, fuzz, mode);
        if !std::mem::take(&mut self.ask_loop) && self.loop_key == Some(key) {
            if native.loop_kind == layout::LOOP_CONTROLLED && native.loop_slot >= native.slot_count
            {
                native.loop_ready = 0;
            }
            return;
        }
        native.loop_ready = 0;
        self.loop_key = None;
        if let Some(quick) = self.interp.quick_pass(self.do_index) {
            match quick.kind {
                QuickKind::Controlled { slot, step, to } if (slot as u64) < native.slot_count => {
                    native.loop_ready = 1;
                    native.loop_kind = layout::LOOP_CONTROLLED;
                    native.loop_slot = slot as u64;
                    native.loop_step = step;
                    native.loop_to = to;
                }
                QuickKind::Controlled { .. } => {}
                QuickKind::Count { remaining } => {
                    native.loop_ready = 1;
                    native.loop_kind = layout::LOOP_COUNT;
                    native.loop_value = i64::try_from(remaining).unwrap_or(i64::MAX);
                }
                QuickKind::Forever => {
                    native.loop_ready = 1;
                    native.loop_kind = layout::LOOP_FOREVER;
                }
            }
            native.loop_line = quick.line as u64;
            native.loop_indent = quick.open_indent as u64;
            native.loop_end = quick.end_index as u64;
            if native.loop_ready != 0 {
                self.loop_key = Some(key);
            }
        }
    }

    fn release(&mut self) -> u32 {
        if !self.interp.activity.pending_traps.is_empty() || self.interp.clause_exits {
            return CONTINUE;
        }
        // The region's last op may have closed the clause already.
        let Some(open) = self.open.take() else {
            return CONTINUE;
        };
        self.interp.finish_plain_clause(open.entry);
        FAST
    }

    fn push_arg(&mut self, src: u16) {
        self.interp.push_call_arg(self.registers, src);
    }

    fn window_holds(&mut self, native: &Native) -> bool {
        let frame = self.interp.activation().frame;
        match self.interp.roots.top_window(frame) {
            Some(window) => {
                native.slots.is_null()
                    || (window.as_mut_ptr() == native.slots
                        && window.len() as u64 == native.slot_count)
            }
            None => native.slots.is_null(),
        }
    }

    fn adopt(&mut self, pc: u32) -> u32 {
        let Some(&(Op::Clause { index, end } | Op::CallingClause { index, end })) =
            self.chunk.ops_in(pc, pc + 1).and_then(<[Op]>::first)
        else {
            return self.stop(JitExit::Fail(
                Loud::op_not_driven("a compiled clause").into(),
            ));
        };
        let entry = self.interp.adopt_stepped_clause();
        self.open = Some(Open {
            pc,
            index: index as usize,
            end,
            entry,
        });
        CONTINUE
    }
}

impl Driven<'_, '_> {
    /// Whether compiled code may open a clause itself: every condition under
    /// which `open_clause!` and `enter_stepped_clause` would do more than
    /// write the countdown and the clause state is absent, and no write they
    /// make to anything else would change it.
    fn steady(&self) -> bool {
        let interp = &*self.interp;
        let activity = &interp.activity;
        let activation = interp.activation();
        let sink = interp.chunk_trace();
        self.source.is_some()
            && self.chunk.trace().clause_echoes() == sink.clause_echoes()
            && !sink.debugging()
            && activity.pending_traps.is_empty()
            && !interp.clause_exits
            && !activity.rerun_pending()
            && activity.clause_line_override.is_none()
            && !activity.procedure_permitted
            && !activity.region_procedure_permitted
            && activity.value_buffer.is_empty()
            && activation.clock_stale
            && !activation.trace_entry.steps()
    }
}

impl Driven<'_, '_> {
    /// The op at `pc`, run by `region_ops!` itself.
    #[inline(never)]
    fn general(&mut self, pc: u32) -> u32 {
        let Some(open) = &self.open else {
            return self.stop(JitExit::Fail(
                Loud::op_not_driven("an op with no clause").into(),
            ));
        };
        let index: usize = open.index;
        let end: u32 = open.end;
        let code = self.code;
        let chunk = self.chunk;
        let registers = self.registers;
        let source = self.source;
        let top = self.top;
        let stale = false;
        let Some(clause) = code.body.instructions.get(index) else {
            return self.stop(JitExit::Fail(Loud::chunk_map_too_short().into()));
        };
        let Some(ops) = chunk.ops_in(pc, pc + 1) else {
            return self.stop(JitExit::Fail(Loud::chunk_map_too_short().into()));
        };
        let interp = &mut *self.interp;
        let ran: Result<RegionEnd, Failure> = 'cold: {
            let next: u32 = 'region: {
                let mut header: Option<LoopHeaderValues> = None;
                let park: Park = 'park: {
                    region_ops!(
                        interp, code, chunk, registers, source, clause, index, stale, header, end,
                        ops, pc, top, 'cold, 'region, 'park
                    );
                    return CONTINUE;
                };
                let Some(open) = self.open.take() else {
                    break 'cold Err(Loud::op_not_driven("a park with no clause").into());
                };
                return self.stop(JitExit::Parked {
                    park,
                    entry: open.entry,
                    clause_pc: open.pc,
                    end: open.end,
                    index: open.index,
                });
            };
            Ok(RegionEnd::At(next))
        };
        // A condition that did not hold: the region left for the branch's
        // target, which the driver's hot exit closes plainly when nothing is
        // queued at the boundary.
        if let Ok(RegionEnd::At(next)) = ran
            && let Some(&Op::ConditionJump { target, .. }) = ops.first()
            && next == target
            && self.interp.activity.pending_traps.is_empty()
            && !self.interp.clause_exits
            && let Some(open) = self.open.take()
        {
            self.interp.finish_plain_clause(open.entry);
            return BRANCH;
        }
        // The region's last op answering `Flow::Next` is the clause done, which
        // the driver settles at `end`: the op after this one, where compiled
        // code carries on.
        // So is a region that ended at the clause's own end, as a block's
        // start does.
        let in_place = pc + 1 == end
            && (matches!(ran, Ok(RegionEnd::Flowed(Flow::Next)))
                || matches!(ran, Ok(RegionEnd::At(next)) if next == end));
        self.leave_from(ran, in_place)
    }
}

// Test-only instrumentation: how many times compiled passes have run on this
// thread.
#[cfg(test)]
thread_local! {
    static COMPILED_RUNS: Cell<usize> = const { Cell::new(0) };
    static QUIET_CALLS: Cell<usize> = const { Cell::new(0) };
    static QUIET_COMPOUNDS: Cell<usize> = const { Cell::new(0) };
    static HANDED_BACK: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
fn compiled_runs() -> usize {
    COMPILED_RUNS.with(Cell::get)
}

#[cfg(test)]
fn reset_compiled_runs() {
    COMPILED_RUNS.with(|runs| runs.set(0));
    QUIET_CALLS.with(|calls| calls.set(0));
    QUIET_COMPOUNDS.with(|accesses| accesses.set(0));
    HANDED_BACK.with(|handed| handed.set(0));
}

/// How many regions that ended at their clause's end the host has closed in
/// place, for compiled code to carry on, on this thread.
#[cfg(test)]
fn handed_back() -> usize {
    HANDED_BACK.with(Cell::get)
}

/// How many compound reads and writes `Host::quiet_compound` has made on
/// this thread.
#[cfg(test)]
fn quiet_compounds() -> usize {
    QUIET_COMPOUNDS.with(Cell::get)
}

/// How many builtin calls `Host::quiet_builtin` has run on this thread.
#[cfg(test)]
fn quiet_calls() -> usize {
    QUIET_CALLS.with(Cell::get)
}

#[cfg(test)]
mod tests;
