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
use crate::jit::layout::Native;
use crate::jit::{self, Arith, CONTINUE, Host, Pass, STOP, Step};
use crate::run::FlatStep;

/// Passes a loop runs driven before it is compiled.
const THRESHOLD: u32 = 64;
/// A loop whose body has an op the compiled code does not take.
const REFUSED: u32 = u32::MAX;

/// Per `DO`/`LOOP` instruction of one chunk: its pass count while it is
/// driven, then where its compiled code is.
pub(crate) struct Loops {
    /// Below [`THRESHOLD`], passes so far; [`THRESHOLD`] plus `k`, the `k`th
    /// entry of `compiled`; [`REFUSED`], never compiled.
    state: Box<[Cell<u32>]>,
    compiled: RefCell<Vec<Rc<jit::Compiled>>>,
}

impl Loops {
    pub(crate) fn new(instructions: usize) -> Loops {
        Loops {
            state: (0..instructions).map(|_| Cell::new(0)).collect(),
            compiled: RefCell::new(Vec::new()),
        }
    }
}

/// How a run of compiled passes ended, which the `Op::LoopNext` arm turns
/// into what the driver does next.
pub(super) enum JitExit {
    /// Drive on from this op.
    At(u32),
    /// The slice ended before the clause, or the pass boundary, at this op.
    Slice(u32),
    /// The loop is over and this is its answer.
    Done(Flow),
    /// A clause answered this `Flow`, settled from this op.
    Flow(Flow, u32),
    Fail(Failure),
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
    pub(super) fn jit_passes(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        source: Option<&ProgramSource>,
        pc: u32,
        do_index: u32,
        top: bool,
    ) -> Option<JitExit> {
        let state = chunk.jit.state.get(do_index as usize)?;
        let compiled = match state.get() {
            REFUSED => return None,
            passes if passes < THRESHOLD - 1 => {
                state.set(passes + 1);
                return None;
            }
            passes if passes == THRESHOLD - 1 => {
                let Some(compiled) =
                    pass_of(chunk, pc, do_index).and_then(|(pass, _)| jit::compile(&pass))
                else {
                    state.set(REFUSED);
                    return None;
                };
                let compiled = Rc::new(compiled);
                let mut all = chunk.jit.compiled.borrow_mut();
                state.set(THRESHOLD + all.len() as u32);
                all.push(Rc::clone(&compiled));
                compiled
            }
            at => Rc::clone(chunk.jit.compiled.borrow().get((at - THRESHOLD) as usize)?),
        };
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
            closed: false,
            exit: None,
        };
        let answer = compiled.run(registers, &mut driven);
        debug_assert_eq!(answer, STOP, "compiled passes end only on a STOP");
        debug_assert!(
            driven.open.is_none(),
            "compiled passes ended with a clause open"
        );
        Some(
            driven
                .exit
                .unwrap_or_else(|| JitExit::Fail(Loud::op_not_driven("a compiled pass").into())),
        )
    }
}

/// The body of the loop whose `Op::DO` is instruction `do_index` and whose
/// `Op::LoopNext` is at `loop_pc`, as a [`Pass`], or `None` where it holds
/// anything but plain clause regions of value ops and stores.
fn pass_of(chunk: &Chunk, loop_pc: u32, do_index: u32) -> Option<(Pass, u32)> {
    let body_start = chunk.op_at(do_index as usize + 1)?;
    if body_start >= loop_pc {
        return None;
    }
    let mut steps = Vec::new();
    let mut pc = body_start;
    while pc < loop_pc {
        let (Some(&Op::Clause { index, end }) | Some(&Op::CallingClause { index, end })) =
            chunk.ops_in(pc, pc + 1)?.first()
        else {
            return None;
        };
        if end <= pc || end > loop_pc {
            return None;
        }
        let opening = chunk.opening_at(index as usize);
        steps.push(Step::Open {
            pc,
            index,
            tabled: opening.tabled,
            line: opening.position.line,
            indent: opening.position.indent,
            register_top: opening.register_top,
        });
        for (at, op) in (pc + 1..).zip(chunk.ops_in(pc + 1, end)?) {
            steps.push(match *op {
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
                    _ => Step::Op { pc: at },
                },
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
                // Ops that neither jump nor trace, which `general` runs; one
                // that parks the clause for a callee hands it to the driver.
                Op::Load { .. }
                | Op::Binary { .. }
                | Op::Prefix { .. }
                | Op::Store { .. }
                | Op::Exec { .. }
                | Op::PushArg { .. }
                | Op::CallArgs { .. }
                | Op::CallExpr { .. }
                | Op::CallNamed { .. }
                | Op::Call { .. }
                | Op::Send { .. }
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
            });
        }
        steps.push(Step::Close { next: end });
        pc = end;
    }
    Some((
        Pass {
            steps,
            registers: chunk.registers,
        },
        body_start,
    ))
}

impl Driven<'_, '_> {
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
        let counted = match self.interp.count_clause_against_deadline(self.top) {
            Ok(counted) => counted,
            Err(Failure::Slice) => return self.stop(JitExit::Slice(self.loop_pc)),
            Err(failure) => return self.stop(JitExit::Fail(failure)),
        };
        match self.interp.flat_loop_step_top(
            self.code,
            self.source,
            self.chunk,
            Some(self.registers),
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
        let Some(&Op::Clause { index, end }) =
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

    fn enter(&mut self, native: &mut Native) {
        self.interp.clause_countdown = u32::try_from(native.countdown).unwrap_or(u32::MAX);
        if native.loop_dirty != 0 {
            native.loop_dirty = 0;
            self.interp.quick_passes_reached(native.loop_value);
        }
        if native.clause_dirty != 0 {
            native.clause_dirty = 0;
            self.interp.enter_compiled_clause(
                native.clause_index as usize,
                native.clause_line as usize,
                native.clause_indent as usize,
            );
        }
    }

    fn leave(&mut self, native: &mut Native) {
        native.countdown = u64::from(self.interp.clause_countdown);
        native.small_int_bound =
            crate::value::small_int_bound(self.interp.activation().settings.digits());
        native.steady = u64::from(self.steady());
        native.indent_base =
            (self.interp.activity.activation_indent + self.interp.activity.indent_offset) as u64;
        // `Op::Load`'s and `Op::Store`'s fast paths reach the slot only for a
        // frame no `EXPOSE` redirects, and a store only while `TRACE` shows no
        // results; the window is offered only where both hold.
        let frame = self.interp.activation().frame;
        let window =
            if self.interp.activation().exposed.is_none() && !self.interp.trace_mode().results {
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
        // The clause the interpreter has running, so that a pass boundary
        // compiled code takes, which moves only the line and the indent,
        // leaves the instruction it names as it is.
        let state = &self.interp.activity.clause_state;
        native.clause_index = state.current_clause_index as u64;
        native.clause_line = state.line() as u64;
        native.clause_indent = state.current_value_indent as u64;
        native.loop_ready = 0;
        if native.steady != 0
            && let Some(quick) = self.interp.quick_pass(self.do_index)
            && (quick.slot as u64) < native.slot_count
        {
            native.loop_ready = 1;
            native.loop_slot = quick.slot as u64;
            native.loop_step = quick.step;
            native.loop_to = quick.to;
            native.loop_line = quick.line as u64;
            native.loop_indent = quick.open_indent as u64;
        }
    }

    fn adopt(&mut self, pc: u32) -> u32 {
        let Some(&Op::Clause { index, end }) =
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
        // The region's last op answering `Flow::Next` is the clause done, which
        // the driver settles at `end`: the op after this one, where compiled
        // code carries on.
        let in_place = pc + 1 == end && matches!(ran, Ok(RegionEnd::Flowed(Flow::Next)));
        self.leave_from(ran, in_place)
    }
}
