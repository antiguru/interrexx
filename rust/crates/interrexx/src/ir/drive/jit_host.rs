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
use crate::jit::{
    self, Arith, BRANCH, CONTINUE, Compare, CondStep, FAST, Host, JUMP, Pass, STOP, Step,
};
use crate::run::FlatStep;
use crate::run::{QuickKind, QuickTest};

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
    compiled: RefCell<Vec<Compiling>>,
}

/// A compiled loop: its code, its budget, its loop records' loops, and the
/// test each record's quick pass must carry for generated code to take it
/// ([`QuickTest`]), or `None` where generated code never takes it.
type Compiling = (Rc<jit::Compiled>, u64, Rc<[u32]>, Rc<[Option<QuickTest>]>);

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
/// or end the range with what `ops_loop` returns. A `Flow` an op answered is
/// settled before the arm sees it, as the driver's own `settle` after a step
/// would: an edge into the driver's step from the arm makes the driver's loop
/// compile worse for every program, compiled or not (#849).
pub(super) enum JitAction {
    At(u32),
    Return(Result<Exit, Failure>),
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
    /// Where each pass starts ([`pass_start`]).
    first: u32,
    open: Option<Open>,
    /// The driver's frame base and the instruction range it drives, which a
    /// `SELECT`'s close settles against.
    range: (usize, usize, usize),
    /// Where the last [`JUMP`] answer goes on.
    jump_to: u32,
    /// The loop each of the block's loop records is, by `DO` index: this
    /// loop's own first, then the inner loops its body runs.
    loops: Rc<[u32]>,
    /// The test each loop record's quick pass carries in generated code.
    tests: Rc<[Option<QuickTest>]>,
    /// Per loop record, what [`Host::leave`] keeps of its quick pass.
    loop_state: [LoopState; layout::MAX_LOOPS],

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
    // own loop, and a call it makes once per entry into compiled passes is
    // not one the loop should be laid out around.
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
                let Some((compiled, budget, loops, tests)) =
                    pass_of(code, chunk, pc, do_index, generation).and_then(|(pass, budget)| {
                        let tests: Rc<[Option<QuickTest>]> = pass
                            .tests
                            .iter()
                            .map(|test| match test {
                                jit::LoopTest::None => Some(QuickTest::None),
                                jit::LoopTest::Native { until: true, .. } => Some(QuickTest::Until),
                                jit::LoopTest::Native { until: false, .. } => {
                                    Some(QuickTest::While)
                                }
                                jit::LoopTest::Host => None,
                            })
                            .collect();
                        Some((jit::compile(&pass)?, budget, pass.loops.into(), tests))
                    })
                else {
                    state.set(REFUSED);
                    return None;
                };
                let compiled = Rc::new(compiled);
                let mut all = chunk.jit.compiled.borrow_mut();
                state.set(THRESHOLD + all.len() as u32);
                all.push((
                    Rc::clone(&compiled),
                    budget,
                    Rc::clone(&loops),
                    Rc::clone(&tests),
                ));
                (compiled, budget, loops, tests)
            }
            at => {
                let all = chunk.jit.compiled.borrow();
                let (compiled, budget, loops, tests) = all.get((at - THRESHOLD) as usize)?;
                (
                    Rc::clone(compiled),
                    *budget,
                    Rc::clone(loops),
                    Rc::clone(tests),
                )
            }
        };
        let (compiled, budget, loops, tests) = compiled;
        let first = pass_start(chunk, do_index)?;
        let mut driven = Driven {
            interp: self,
            code,
            chunk,
            registers,
            source,
            top,
            loop_pc: pc,
            do_index: do_index as usize,
            first,
            open: None,
            range,
            jump_to: 0,
            loops,
            tests,
            loop_state: [LoopState::ASK; layout::MAX_LOOPS],

            budget,
            hosted: 0,
            retire: false,
            closed: false,
            exit: None,
        };
        #[cfg(test)]
        COMPILED_RUNS.with(|runs| runs.set(runs.get() + 1));
        #[cfg(test)]
        COMPILED_LINES.with(|lines| {
            lines
                .borrow_mut()
                .insert(chunk.opening_at(do_index as usize).position.line)
        });
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
        let (base, start, end) = range;
        let settle = |interp: &mut Interp, flow: Flow, next: u32| match interp
            .settle(code, chunk, registers, base, flow, next, start, end, source)
        {
            Ok(Settled::At(target)) => JitAction::At(target),
            Ok(Settled::Escaped(other)) => JitAction::Return(Ok(Exit::Flow(other))),
            Err(failure) => JitAction::Return(Err(failure)),
        };
        Some(match exit {
            JitExit::At(target) => JitAction::At(target),
            JitExit::Slice(at) => JitAction::Return(Ok(Exit::Slice(at))),
            JitExit::Done(flow) => {
                self.activity.frames.pop();
                settle(self, flow, pc)
            }
            JitExit::Flow(flow, next) => settle(self, flow, next),
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
    // A condition that calls or sends is a test region in the stream, in
    // front of the body (`Chunk::loop_test_at`), where each pass boundary
    // goes on: the pass starts there.
    let first = pass_start(chunk, do_index)?;
    if first >= loop_pc {
        return None;
    }
    let mut steps = Vec::new();
    // The loop each loop record is: this one, then each inner loop in the
    // order its start appears.
    let mut loops = vec![do_index];
    let mut pc = first;
    // An `IF`'s branches stay inside the body, and its `EndBranch` and
    // `Jump` sit between clause regions.
    let inside = |target: u32| (first..=loop_pc).contains(&target);
    while pc < loop_pc {
        let (index, end) = match *chunk.ops_in(pc, pc + 1)?.first()? {
            Op::Clause { index, end } | Op::CallingClause { index, end } => (index, end),
            // A loop's header test region, which the host opens as the
            // driver's `Op::LoopTest` arm does, and whose `Op::LoopTested`
            // the host runs: a clause of its own, whose ops are its
            // condition's.
            Op::LoopTest { index, end } => (index, end),
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
            // An inner loop's pass boundary; its start below is what let the
            // loop into the pass.
            Op::LoopNext { index } if index != do_index => {
                let record = loops.iter().position(|&loop_index| loop_index == index)?;
                steps.push(Step::InnerNext {
                    pc,
                    record: u8::try_from(record).ok()?,
                    body: pass_start(chunk, index)?,
                });
                pc += 1;
                continue;
            }
            _ => return None,
        };
        if end <= pc || end > loop_pc {
            return None;
        }
        let test = matches!(chunk.ops_in(pc, pc + 1)?.first()?, Op::LoopTest { .. });
        if test {
            steps.push(Step::Test { pc });
        } else {
            let opening = chunk.opening_at(index as usize);
            steps.push(Step::Open {
                pc,
                tabled: opening.tabled,
                register_top: opening.register_top,
            });
        }
        // The `Step::PushArg`s whose arguments no op has taken yet, as indices
        // into `steps`. Every op that takes pushed arguments takes the
        // `argc` pushed last, so a call's own are the last `argc` here.
        let mut pending: Vec<usize> = Vec::new();
        // An inner loop's `DO` clause: from its first header value on, the
        // clause runs on the host in one region (`Step::Region`), since the
        // header values are filed for the `Op::LoopRun` that follows them.
        // Only a loop the driver flattens.
        let ops = chunk.ops_in(pc + 1, end)?;
        let region = match ops.iter().find_map(|op| match *op {
            Op::LoopRun { index } => Some(index),
            _ => None,
        }) {
            Some(index)
                if matches!(
                    code.body.instructions.get(index as usize).map(|clause| &clause.kind),
                    Some(InstructionKind::Do(body) | InstructionKind::Loop(body))
                        if !matches!(body.kind, interrexx_parse::LoopKind::Simple)
                ) =>
            {
                if loops.len() >= layout::MAX_LOOPS {
                    return None;
                }
                loops.push(index);
                (pc + 1..).zip(ops).find_map(|(at, op)| {
                    matches!(
                        op,
                        Op::LoopHeaderValue { .. } | Op::TraceKeyword { .. } | Op::LoopRun { .. }
                    )
                    .then_some(at)
                })
            }
            _ => None,
        };
        for (at, op) in (pc + 1..).zip(ops) {
            if region == Some(at) {
                steps.push(Step::Region { pc: at });
                break;
            }
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
                // A `do; ... end` block: its start runs on the host, whose
                // region ends at the clause's own end, where compiled code
                // carries on into the block's clauses. An unlabelled block's
                // start is a clause counted; a labelled one's also opens the
                // frame its `END`'s `Op::EndWhen` closes, which the host's
                // `region_ops!` does.
                Op::LoopRun { index }
                    if let Some(InstructionKind::Do(body) | InstructionKind::Loop(body)) = code
                        .body
                        .instructions
                        .get(index as usize)
                        .map(|clause| &clause.kind)
                        && matches!(body.kind, interrexx_parse::LoopKind::Simple) =>
                {
                    pending.clear();
                    steps.push(if body.label.is_none() {
                        Step::BlockStart { pc: at }
                    } else {
                        Step::Op { pc: at }
                    });
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
            // **A test region whose call can park is the driver's.** A call
            // to a routine or a send to a method runs its callee stackless
            // from the driver, which then finishes the pass itself: compiled
            // code would be entered and left on every pass and its body never
            // run. A call its site resolved to a builtin runs to completion.
            if test
                && match *op {
                    Op::CallArgs { site, .. } => !matches!(
                        chunk.resolved_call(site, generation),
                        Some(Resolved::Builtin(_))
                    ),
                    Op::CallExpr { .. }
                    | Op::CallNamed { .. }
                    | Op::Call { .. }
                    | Op::Send { .. }
                    | Op::SendWith { .. }
                    | Op::SendScope { .. }
                    | Op::Message { .. } => true,
                    _ => false,
                }
            {
                return None;
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
                // The test region's last op, which decides the pass.
                Op::LoopTested { .. } if test && at + 1 == end => Step::Tested { pc: at },
                // A value echo, whose own gate generated code asks: the op
                // through the host while `TRACE` shows intermediates, and
                // nothing otherwise. A body that sets `TRACE` keeps them
                // (#850).
                Op::TraceArgument { .. }
                | Op::TraceFunction { .. }
                | Op::TraceLiteral { .. }
                | Op::TraceRead { .. }
                | Op::TraceOperator { .. }
                | Op::TracePrefix { .. } => Step::Trace { pc: at },
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
    // An inner loop's own body is what compiling it alone would compile, so
    // it is not this loop's to weigh. What its start and its end cost every
    // pass of this loop, two calls to the host, is.
    let mut hosted = 0;
    let mut native = 0;
    let mut inner = 0usize;
    for step in &steps {
        match step {
            Step::Region { .. } => {
                hosted += 2;
                inner += 1;
            }
            Step::InnerNext { .. } => inner = inner.saturating_sub(1),
            // The test region's opening and its end, one call each where the
            // driver takes them in its own arms; an inner loop's are in its
            // `Step::Region`'s weight.
            Step::Test { .. } | Step::Tested { .. } if inner == 0 => hosted += 1,
            Step::Test { .. } | Step::Tested { .. } => {}
            Step::Op { .. } | Step::Between { .. } => hosted += 1,
            Step::Builtin { .. }
            | Step::Compound { .. }
            | Step::Open { .. }
            | Step::Close { .. }
            | Step::Trace { .. } => {}
            _ if inner == 0 => native += 1,
            _ => {}
        }
    }
    if hosted * HOSTED_SHARE > native {
        return None;
    }
    let budget = (native / HOSTED_SHARE).max(1) as u64;
    let tests = loops
        .iter()
        .map(|&index| loop_test(code, chunk, index))
        .collect();
    Some((
        Pass {
            steps,
            registers: chunk.registers,
            end: loop_pc,
            loops,
            tests,
        },
        budget,
    ))
}

/// What the pass boundary of the loop whose `Op::DO` is instruction
/// `do_index` tests: its `WHILE` or `UNTIL` from `Chunk::conditions`, as
/// generated code runs it where each op has a quick form, and the host's
/// otherwise.
fn loop_test(code: &Code<'_>, chunk: &Chunk, do_index: u32) -> jit::LoopTest {
    let Some(InstructionKind::Do(body) | InstructionKind::Loop(body)) = code
        .body
        .instructions
        .get(do_index as usize)
        .map(|clause| &clause.kind)
    else {
        return jit::LoopTest::Host;
    };
    let Some(conditional) = &body.conditional else {
        return jit::LoopTest::None;
    };
    if body.counter.is_some() || chunk.loop_test_at(do_index as usize).is_some() {
        return jit::LoopTest::Host;
    }
    let Some(condition) = chunk.condition_at(do_index as usize) else {
        return jit::LoopTest::Host;
    };
    let Some(ops) = chunk.condition_ops(condition) else {
        return jit::LoopTest::Host;
    };
    let mut steps = Vec::with_capacity(ops.len());
    for op in ops {
        let step = match *op {
            Op::Const { dst, konst } => match chunk.interned_konst(konst) {
                ObjRef::NIL => return jit::LoopTest::Host,
                value => CondStep::Const {
                    dst,
                    bits: value.bits(),
                },
            },
            Op::LoadConstant { symbol, dst } => match chunk.interned_symbol(symbol) {
                ObjRef::NIL => return jit::LoopTest::Host,
                value => CondStep::Const {
                    dst,
                    bits: value.bits(),
                },
            },
            Op::Load {
                read: SymbolRead::Simple,
                at: slot,
                dst,
                ..
            } if let Some(slot) = slot.resolved()
                && let Ok(slot) = u32::try_from(slot) =>
            {
                CondStep::Load { slot, dst }
            }
            Op::Arith {
                op,
                hint,
                lhs,
                rhs,
                dst,
            } if chunk.tries_small_int(hint) => CondStep::Arith {
                op: match op {
                    Operator::Plus => Arith::Add,
                    Operator::Subtract => Arith::Subtract,
                    Operator::Multiply => Arith::Multiply,
                    Operator::IntDiv => Arith::IntDiv,
                    Operator::Remainder => Arith::Remainder,
                    _ => return jit::LoopTest::Host,
                },
                lhs,
                rhs,
                dst,
            },
            Op::Binary {
                op,
                lhs,
                rhs,
                dst,
                text: false,
                ..
            } if let Some(op) = quick_compare(op) => CondStep::Compare { op, lhs, rhs, dst },
            Op::Binary {
                op: op @ (Operator::And | Operator::Or),
                lhs,
                rhs,
                dst,
                ..
            } => CondStep::Logic {
                and: op == Operator::And,
                lhs,
                rhs,
                dst,
            },
            _ => return jit::LoopTest::Host,
        };
        steps.push(step);
    }
    jit::LoopTest::Native {
        steps,
        dst: condition.dst,
        until: conditional.until,
    }
}

/// Where a pass of the loop whose `Op::DO` is instruction `do_index` starts:
/// its header test region where it has one, its body's first op otherwise.
fn pass_start(chunk: &Chunk, do_index: u32) -> Option<u32> {
    match chunk.loop_test_at(do_index as usize) {
        Some(test) => Some(test.test),
        None => chunk.op_at(do_index as usize + 1),
    }
}

/// What [`Host::leave`] keeps of one loop record's quick pass.
#[derive(Clone, Copy)]
struct LoopState {
    /// Whether the quick pass is to be worked out afresh.
    ask: bool,
    /// The settings it was worked out under, while it stands.
    key: Option<(u64, u64, crate::trace::TraceMode)>,
}

impl LoopState {
    const ASK: LoopState = LoopState {
        ask: true,
        key: None,
    };
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
        #[cfg(test)]
        RETIREMENTS.with(|retired| retired.set(retired.get() + 1));
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
        self.leave_from(ran, false, false)
    }

    /// [`Driven::leave`], carrying on in compiled code where `in_place` and
    /// the clause closed with the `Flow::Next` its region answered.
    fn leave_from(
        &mut self,
        ran: Result<RegionEnd, Failure>,
        in_place: bool,
        settles: bool,
    ) -> u32 {
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
        if ran.is_err() {
            self.interp
                .blame_failed_test(self.code, self.chunk, open.pc, self.source);
        }
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
            Ok(ClauseOutcome::Ran(Ok(RegionEnd::Flowed(flow)))) if settles => {
                return self.settle_inner(flow, open.pc);
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
        #[cfg(test)]
        HOST_BOUNDARIES.with(|taken| taken.set(taken.get() + 1));
        if self.retire {
            return self.retired(self.loop_pc);
        }
        self.loop_state[0].ask = true;
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
            Ok(FlatStep::Body(at)) if at == self.first => CONTINUE,
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

    fn region(&mut self, pc: u32) -> u32 {
        let Some(end) = self.open.as_ref().map(|open| open.end) else {
            return self.stop(JitExit::Fail(
                Loud::op_not_driven("a region with no clause").into(),
            ));
        };
        // An inner loop that starts here has its record asked already: its
        // first start is asked from the block's own start, and every run of it
        // that ended in compiled code ended in `inner_next` or in its test
        // region's `Op::LoopTested`, which ask it, or in its test region's
        // opening, whose loop has no quick pass.
        self.general_to(pc, end, false)
    }

    fn op(&mut self, pc: u32) -> u32 {
        self.hosted += 1;
        // Between clauses: the driver's own arms for these ops.
        match self.chunk.ops_in(pc, pc + 1).and_then(<[Op]>::first) {
            // An inner loop's pass boundary, as the driver's `Op::LoopNext` arm
            // takes it: not an op the host runs in the driver's stead, so not
            // weighed against the native steps.
            Some(&Op::LoopNext { index }) if index as usize != self.do_index => {
                self.hosted -= 1;
                return self.inner_next(pc);
            }
            // A header test region's boundaries, which the gate weighed, as
            // the driver's `Op::LoopTest` arm and `region_ops!` take them.
            Some(&Op::LoopTest { index, .. }) => {
                self.hosted -= 1;
                return self.test_open(pc, index as usize != self.do_index);
            }
            Some(&Op::LoopTested { index, .. }) => {
                self.hosted -= 1;
                let inner = index as usize != self.do_index;
                if inner && let Some(k) = self.loops.iter().position(|&at| at == index) {
                    self.loop_state[k].ask = true;
                }
                return self.general_to(pc, pc + 1, inner);
            }
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
        let kept = (self.retire, self.loop_state);
        fresh.passes = native.passes;
        fresh.loops = native.loops;
        for state in &mut self.loop_state {
            state.ask = false;
        }
        Host::leave(self, &mut fresh);
        (self.retire, self.loop_state) = kept;
        fresh.slots == native.slots
            && fresh.slot_count == native.slot_count
            && fresh.steady == native.steady
            && fresh.small_int_bound == native.small_int_bound
            && fresh.quick_values == native.quick_values
            && fresh.intermediates == native.intermediates
            && fresh.jump_to == native.jump_to
    }

    fn enter(&mut self, native: &mut Native) {
        self.interp.clause_countdown = u32::try_from(native.countdown).unwrap_or(u32::MAX);
        // A record generated code stepped is the innermost loop's: its
        // boundary is taken only while that loop runs innermost, and the host
        // is called before any other loop is.
        let own = &mut native.loops[0];
        if own.dirty != 0 {
            own.dirty = 0;
            self.interp.quick_passes_reached(own.value);
        }
        if self.loops.len() > 1 {
            for record in &mut native.loops[1..] {
                if record.dirty != 0 {
                    record.dirty = 0;
                    self.interp.quick_passes_reached(record.value);
                }
            }
        }
        match std::mem::replace(&mut native.clause_at, layout::CLAUSE_NONE) {
            layout::CLAUSE_NONE => {}
            // A pass boundary's header clause is the loop's `END`, as the
            // driver's `flat_loop_step` makes it (`Interp::quick_pass` takes
            // only a header blamed on the `END`).
            at if at <= layout::clause_header(0)
                && at >= layout::clause_header(layout::MAX_LOOPS - 1) =>
            {
                let record = &native.loops[(layout::clause_header(0) - at) as usize];
                self.interp.enter_compiled_clause(
                    record.end as usize,
                    record.line as usize,
                    record.indent as usize,
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
        native.intermediates = u64::from(interp.tracing_intermediates());
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
            for record in &mut native.loops {
                record.ready = 0;
            }
            for state in &mut self.loop_state {
                state.key = None;
            }
            return;
        }
        // The record worked out is the innermost running loop's, the only one
        // whose boundary generated code can reach before the host is called
        // again. A loop that left for an inner one and is innermost again
        // has the state it had: its record stands. A pass that holds no inner
        // loop has only its own record, and its loop is the innermost
        // whenever the host is called.
        let key = (digits, fuzz, mode);
        if self.loops.len() == 1 {
            self.refresh_loop(native, 0, key);
        } else {
            let top = self
                .interp
                .activity
                .flat_top
                .as_ref()
                .map(|flat| flat.do_index);
            if let Some(k) =
                top.and_then(|top| self.loops.iter().position(|&index| index as usize == top))
            {
                self.refresh_loop(native, k, key);
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
    /// [`Host::leave`]'s quick pass for loop record `k`, under the settings
    /// `key`: worked out afresh where it was asked for or `key` moved, else
    /// left as it stands.
    #[inline(always)]
    fn refresh_loop(
        &mut self,
        native: &mut Native,
        k: usize,
        key: (u64, u64, crate::trace::TraceMode),
    ) {
        let record = &mut native.loops[k];
        let state = &mut self.loop_state[k];
        // **What the quick pass rests on that a call can change**: `DIGITS`
        // and `FUZZ`, which only this activation's own instructions set, and
        // the trace setting, which a `TRACE()` call also sets. The loop's own
        // state changes only at a pass boundary the host takes, which asks
        // again; the control variable is read from its slot on every pass.
        if !std::mem::take(&mut state.ask) && state.key == Some(key) {
            if record.kind == layout::LOOP_CONTROLLED && record.slot >= native.slot_count {
                record.ready = 0;
            }
            return;
        }
        record.ready = 0;
        state.key = None;
        // A loop whose boundary generated code never takes needs no record.
        let Some(compiled) = self.tests.get(k).copied().flatten() else {
            return;
        };
        if let Some(quick) = self.interp.quick_pass(self.loops[k] as usize) {
            // Both read off the loop's own instruction.
            debug_assert_eq!(
                compiled, quick.test,
                "loop record {k}'s quick pass is not what its code was compiled for"
            );
            match quick.kind {
                QuickKind::Controlled { slot, step, to } if (slot as u64) < native.slot_count => {
                    record.ready = 1;
                    record.kind = layout::LOOP_CONTROLLED;
                    record.slot = slot as u64;
                    record.step = step;
                    record.to = to;
                }
                QuickKind::Controlled { .. } => {}
                QuickKind::Count { remaining } => {
                    record.ready = 1;
                    record.kind = layout::LOOP_COUNT;
                    record.value = i64::try_from(remaining).unwrap_or(i64::MAX);
                }
                QuickKind::Forever => {
                    record.ready = 1;
                    record.kind = layout::LOOP_FOREVER;
                }
            }
            record.line = quick.line as u64;
            record.indent = quick.open_indent as u64;
            record.end = quick.end_index as u64;
            if record.ready != 0 {
                state.key = Some(key);
            }
        }
    }
}

impl Driven<'_, '_> {
    /// An inner loop's `Op::LoopNext` at `pc`: the header clause counted, the
    /// next pass stepped, and where it goes on, as the driver's arm.
    fn inner_next(&mut self, pc: u32) -> u32 {
        #[cfg(test)]
        HOST_BOUNDARIES.with(|taken| taken.set(taken.get() + 1));
        // The inner loop's own state moves here, so its record is asked again.
        if let Some(&Op::LoopNext { index }) = self.chunk.ops_in(pc, pc + 1).and_then(<[Op]>::first)
        {
            // As the driver's arm asserts: every way out of an inner loop
            // other than its own boundary leaves compiled code, so the loop
            // innermost here is the one this op ends a pass of.
            debug_assert_eq!(
                self.interp
                    .activity
                    .flat_top
                    .as_ref()
                    .map(|flat| flat.do_index),
                Some(index as usize),
                "an inner LoopNext ended a pass of a loop other than the one it names"
            );
            if let Some(k) = self
                .loops
                .iter()
                .position(|&loop_index| loop_index == index)
            {
                self.loop_state[k].ask = true;
            }
        }
        let counted = match self.interp.count_clause_against_deadline(self.top) {
            Ok(counted) => counted,
            Err(Failure::Slice) => return self.stop(JitExit::Slice(pc)),
            Err(failure) => return self.stop(JitExit::Fail(failure)),
        };
        let target = match self.interp.flat_loop_step_top(
            self.code,
            self.source,
            self.chunk,
            self.registers,
            Flow::Next,
            |_| Ok(counted),
        ) {
            Ok(FlatStep::Body(at)) => at,
            Ok(FlatStep::Done(flow)) => {
                self.interp.activity.frames.pop();
                return self.settle_inner(flow, pc);
            }
            Err(failure) => return self.stop(JitExit::Fail(failure)),
        };
        self.go_on(target, pc)
    }

    /// Where an inner loop that answered `flow` at `pc`, its frame already
    /// closed, goes on, as the driver settles it.
    fn settle_inner(&mut self, flow: Flow, pc: u32) -> u32 {
        let (base, start, end) = self.range;
        match self.interp.settle(
            self.code,
            self.chunk,
            self.registers,
            base,
            flow,
            pc,
            start,
            end,
            self.source,
        ) {
            Ok(Settled::At(target)) => self.go_on(target, pc),
            Ok(Settled::Escaped(flow)) => self.stop(JitExit::Escaped(flow)),
            Err(failure) => self.stop(JitExit::Fail(failure)),
        }
    }

    /// Compiled code going on at `target` from the op at `pc`: the next op,
    /// a label of the pass, or the driver at any other op.
    fn go_on(&mut self, target: u32, pc: u32) -> u32 {
        if target == pc + 1 {
            return CONTINUE;
        }
        self.jump_to = target;
        self.exit = Some(JitExit::At(target));
        JUMP
    }

    /// A loop's header test region at `pc`, opened as the driver's
    /// `Op::LoopTest` arm opens it: counted, its clause entered, and its
    /// condition's ops left to run in it. A loop that ends here instead is
    /// settled, an `inner` one's here, the compiled loop's by the driver.
    fn test_open(&mut self, pc: u32, inner: bool) -> u32 {
        if self.retire {
            return self.retired(pc);
        }
        // As for `Host::open`: the driver's, before the clause is counted.
        let sink = self.interp.chunk_trace();
        if self.chunk.trace().clause_echoes() != sink.clause_echoes() || sink.debugging() {
            return self.stop(JitExit::At(pc));
        }
        match self
            .interp
            .loop_test_entry(self.code, self.chunk, self.source, pc, self.top)
        {
            Ok(TestEntry::Region(parked)) => {
                self.open = Some(Open {
                    pc,
                    index: parked.index,
                    end: parked.end,
                    entry: parked.entry,
                });
                CONTINUE
            }
            Ok(TestEntry::Done(flow)) => {
                self.interp.activity.frames.pop();
                if inner {
                    self.settle_inner(flow, pc)
                } else {
                    self.stop(JitExit::Flow(flow, pc))
                }
            }
            Ok(TestEntry::Slice) => self.stop(JitExit::Slice(pc)),
            Err(failure) => self.stop(JitExit::Fail(failure)),
        }
    }

    /// The op at `pc`, run by `region_ops!` itself.
    #[inline(always)]
    fn general(&mut self, pc: u32) -> u32 {
        self.general_to(pc, pc + 1, false)
    }

    /// The ops of `[pc, to)` of the open clause, run by `region_ops!` in one
    /// region, as the driver runs a clause's. With `settles`, a `Flow` the
    /// clause answers is settled here rather than by the driver: an inner
    /// loop's test that ended it.
    #[inline(never)]
    fn general_to(&mut self, pc: u32, to: u32, settles: bool) -> u32 {
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
        let Some(ops) = chunk.ops_in(pc, to) else {
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
        let in_place = to == end
            && (matches!(ran, Ok(RegionEnd::Flowed(Flow::Next)))
                || matches!(ran, Ok(RegionEnd::At(next)) if next == end));
        self.leave_from(ran, in_place, settles)
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
    static HOST_BOUNDARIES: Cell<usize> = const { Cell::new(0) };
    static RETIREMENTS: Cell<usize> = const { Cell::new(0) };
    static COMPILED_LINES: RefCell<std::collections::BTreeSet<u32>> =
        const { RefCell::new(std::collections::BTreeSet::new()) };
}

/// The source lines of the `DO`s whose compiled passes have run on this
/// thread.
#[cfg(test)]
fn compiled_lines() -> Vec<u32> {
    COMPILED_LINES.with(|lines| lines.borrow().iter().copied().collect())
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
    HOST_BOUNDARIES.with(|taken| taken.set(0));
    RETIREMENTS.with(|retired| retired.set(0));
    COMPILED_LINES.with(|lines| lines.borrow_mut().clear());
}

/// How many pass boundaries of compiled loops, theirs or an inner loop's,
/// the host has taken on this thread.
#[cfg(test)]
fn host_boundaries() -> usize {
    HOST_BOUNDARIES.with(Cell::get)
}

/// How many compiled loops have been retired on this thread.
#[cfg(test)]
fn retirements() -> usize {
    RETIREMENTS.with(Cell::get)
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
