//! `compile`: the one pass that turns a body into a [`Chunk`] (the plan's
//! Decisions section: "compilation is whole-body and lazy: one body at a
//! time, on first entry, cached").

use interrexx_cast::CastFrom;
use rustc_hash::FxHashMap;

use interrexx_parse::{
    Args, Call, CodeBody, ExprRef, ExprView, Instruction, InstructionKind, LoopKind, ParseSource,
    SymbolId, Use,
};

use super::{
    Calls, Chunk, ChunkTooLarge, ClauseOpening, ClausePosition, ConditionKeyword, ConditionOps,
    Hints, LoopTestOps, NodePath, Op, ParseOpening, PlanSlot, SendForm, UseArgOp,
};
use crate::eval::{SymbolRead, is_arithmetic, is_concatenation, is_native_binary};
use crate::plan::Plan;
use crate::run::{
    FORWARD_ARRAY_SLOT, ForwardPart, HeaderRole, LOOP_CONDITION_SLOT, QueueKeyword, ReturnKeyword,
    if_targets, loop_header_plan, otherwise_range, when_targets,
};
use crate::trace::ChunkTrace;

mod invariants;
use invariants::{
    assert_call_echoes_follow_their_op, assert_exec_regions_hold_nothing_else,
    assert_keyword_echoes_precede_their_value, assert_literal_echoes_follow_their_load,
    assert_operator_echoes_follow_their_op, assert_prefix_echoes_follow_their_op,
    assert_read_echoes_follow_their_load, assert_region_ops_name_their_clause,
    assert_trace_ops_open_a_clause_region,
};

/// The compile-time register stack (the plan's Decisions section: "register
/// allocation is a compile-time stack, and the chunk records its high-water
/// mark").
struct Registers {
    /// The next index `alloc` hands out.
    next: u16,
    /// The largest `next` ever reached, which is what [`Chunk::registers`]
    /// records -- the size of the region the driver has to reserve for a run
    /// of this chunk to address every register it names.
    high_water: u16,
}

/// A saved register top, handed back to [`Registers::release`].
#[derive(Clone, Copy)]
struct Mark(u16);

impl Registers {
    fn new() -> Registers {
        Registers {
            next: 0,
            high_water: 0,
        }
    }

    fn mark(&self) -> Mark {
        Mark(self.next)
    }

    /// The next register, or the refusal a body needing more than `u16::MAX`
    /// live at once earns (the plan's Decisions section: "register indices
    /// are `u16`").
    fn alloc(&mut self) -> Result<u16, ChunkTooLarge> {
        let index = self.next;
        self.next = self
            .next
            .checked_add(1)
            .ok_or(ChunkTooLarge { what: "registers" })?;
        self.high_water = self.high_water.max(self.next);
        Ok(index)
    }

    /// Puts the top back to `mark`, so every register allocated since is
    /// handed out again.
    fn release(&mut self, mark: Mark) {
        self.next = mark.0;
    }

    fn high_water(&self) -> u16 {
        self.high_water
    }
}

/// The constant table [`Op::Const`] indexes, built as the pass goes and
/// **interned**: a literal written twice gets one entry.
struct Constants<'a> {
    /// One entry per distinct literal, in the order they were first seen,
    /// which is what [`Chunk::consts`] becomes.
    values: Vec<Box<[u8]>>,
    /// Which entry a literal's bytes already have.
    index: FxHashMap<&'a [u8], u32>,
}

impl<'a> Constants<'a> {
    fn new() -> Constants<'a> {
        Constants {
            values: Vec::new(),
            index: FxHashMap::default(),
        }
    }

    /// The entry `bytes` has, adding one if this is the first occurrence.
    fn intern(&mut self, bytes: &'a [u8]) -> Result<u32, ChunkTooLarge> {
        if let Some(&at) = self.index.get(bytes) {
            return Ok(at);
        }
        let at =
            u32::try_from(self.values.len()).map_err(|_| ChunkTooLarge { what: "constants" })?;
        self.values.push(Box::from(bytes));
        self.index.insert(bytes, at);
        Ok(at)
    }
}

/// A jump whose target is an instruction index the forward pass has not
/// reached yet, and the op that has to be rewritten once it has.
struct Patch {
    /// The op to rewrite.
    op: u32,
    /// The instruction it continues at.
    target: usize,
    /// Which of the two ops an instruction can be entered at this jump wants.
    kind: PatchKind,
}

/// An op that has to be emitted in front of an instruction's own first op,
/// so that arriving at that instruction from elsewhere runs it and falling
/// into the instruction from the op before it does not.
enum Before {
    /// The end of an `IF`'s true branch: its clause boundary
    /// ([`Op::EndBranch`]), and the jump past the `ELSE`
    /// when there is one to skip. The `IF` is at `owner`.
    ThenEnd { resume: Option<usize>, owner: usize },
    /// A `SELECT`'s [`Op::EnterOtherwise`], in front of the `OTHERWISE` marker
    /// whose branch it opens a frame over. The `SELECT` is at this index.
    EnterOtherwise(usize),
    /// A `SELECT` branch runs out in front of this instruction
    /// ([`Op::EndWhen`]): a listed `WHEN`'s own `false_target`, or the end of
    /// the `OTHERWISE` body.
    SelectBranchEnd,
    /// A labelled `DO` block runs out in front of this instruction, its
    /// `END` ([`Op::EndWhen`]). It goes after the boundaries of any `IF`
    /// ending there: they belong inside the block, and an `IF` whose
    /// condition is false lands past them on this op.
    BlockEnd,
}

/// What a listed `WHEN` needs from the `SELECT` that collected it, recorded
/// when that `SELECT` compiles and read when the `WHEN` itself does.
struct WhenInfo {
    /// The `SELECT` this `WHEN` belongs to.
    select: u32,
    /// The register that `SELECT`'s `CASE` value is in, and `None` for a plain
    /// `SELECT`.
    case: Option<u16>,
    /// Where the scan goes when this `WHEN` does not hold: the next listed
    /// `WHEN`, the `OTHERWISE`, or the `END` whose 7.3 is what "no `WHEN`
    /// matched and there is no `OTHERWISE`" means.
    false_target: usize,
    /// Which of that instruction's entries the scan wants. It is
    /// [`PatchKind::EnterOtherwise`] for an `OTHERWISE`, whose
    /// [`Op::EnterOtherwise`] opens the frame, and [`PatchKind::Enter`] for the
    /// other two. Neither runs the ops the branch before it owes there -- its
    /// close, and the boundaries of the `IF`s it ends with -- because the scan
    /// never entered that branch.
    false_kind: PatchKind,
}

/// Which op a jump to an instruction means, for an instruction that has ops
/// in front of its own ([`Before`]), each owed by a different construct.
#[derive(Clone, Copy)]
enum PatchKind {
    /// The instruction's own first op: run the instruction.
    Enter,
    /// Where control resumes when it arrives at this instruction from
    /// anywhere else, which is `Chunk::op_of`'s own answer: the branch-end
    /// jump when there is one in front, so that a true branch finishing --
    /// by falling off its end or by a nested construct's `Flow::Goto` landing
    /// exactly on the boundary -- skips the `ELSE` rather than running it.
    Resume,
    /// The false path of the `IF` at this index, which [`emit_before`]
    /// records as it emits that `IF`'s [`Before::ThenEnd`]: past that
    /// boundary and the ones of the `IF`s nested in its true branch, and
    /// short of every op an enclosing construct owes there.
    IfFalse(usize),
    /// The [`Op::EnterOtherwise`] of the `SELECT` at this index, which
    /// [`emit_before`] records as it emits it: past the ops the last listed
    /// `WHEN`'s branch owes in front of the `OTHERWISE`.
    EnterOtherwise(usize),
}

/// Compiles `body` into a [`Chunk`], once, whole.
pub(crate) fn compile(
    body: &CodeBody,
    plan: &Plan,
    trace: ChunkTrace,
) -> Result<Chunk, ChunkTooLarge> {
    compile_settled(body, plan, trace, true)
}

/// [`compile`] for a body whose trace setting can change from outside it between any two of
/// its clauses -- an embedding's trace exit turns tracing on at a clause boundary -- so no
/// clause's setting is settled and every echo keeps its run-time gate.
pub(crate) fn compile_unsettled(
    body: &CodeBody,
    plan: &Plan,
    trace: ChunkTrace,
) -> Result<Chunk, ChunkTooLarge> {
    compile_settled(body, plan, trace, false)
}

fn compile_settled(
    body: &CodeBody,
    plan: &Plan,
    trace: ChunkTrace,
    settled: bool,
) -> Result<Chunk, ChunkTooLarge> {
    #[cfg(test)]
    count_compile_call();

    // **Whether a clause carries a value echo at all**, decided here rather
    // than gated per execution: an op that is not in the stream cannot be
    // gated back on, so this decision is only safe where the setting the
    // clause runs under is settled. `trace_flow` settles it per clause, from
    // the setting the chunk is keyed under; a clause it cannot settle answers
    // [`super::trace_flow::Setting::Disagrees`] or
    // [`super::trace_flow::Setting::Unknown`], which keep the echoes and keep
    // their run-time gate.
    let settings = if settled {
        super::trace_flow::analyse(body, plan, trace)
    } else {
        vec![super::trace_flow::Setting::Unknown; body.instructions.len()].into_boxed_slice()
    };
    // An unsettled body widens rather than narrows: its echoes stay where the analysis would
    // have dropped them.
    #[cfg(debug_assertions)]
    if settled {
        assert_analysis_only_narrows(plan, trace, &settings);
    }

    let len = body.instructions.len();
    let mut ops: Vec<Op> = Vec::with_capacity(len);
    let mut op_of = Vec::with_capacity(len + 1);
    let mut registers = Registers::new();
    let mut consts = Constants::new();
    let mut hints = Hints::new();
    let mut calls = Calls::new();
    let mut patches: Vec<Patch> = Vec::new();
    // Indexed by instruction: the op that goes in front of that instruction's
    // own, emitted at its `op_of` entry. A `Vec` keyed by the instruction the
    // op sits in front of, rather than a stack, because each entry belongs to
    // exactly one construct -- an `ELSE` to one `THEN`, an `OTHERWISE` to one
    // `SELECT` -- and the `debug_assert`s below are what say so rather than
    // assuming it. A list per instruction, not one entry: nested `IF`s whose
    // branches end at the same instruction each owe a boundary of their own,
    // which runs as one synthetic instruction per branch. `len + 1` entries,
    // because a branch can end at the body's own end and still owe one.
    let mut before: Vec<Vec<Before>> = Vec::new();
    before.resize_with(len + 1, Vec::new);
    // Indexed by instruction: what a listed `WHEN` compiling at that index
    // needs from the `SELECT` that collected it.
    let mut when_info: Vec<Option<WhenInfo>> = Vec::new();
    when_info.resize_with(len, || None);
    // Indexed by instruction: a register top to put back on reaching it. A
    // `SELECT CASE`'s own value is what needs one -- it is allocated in the
    // enclosing scope so that every member clause's release leaves it alone,
    // and this is where that allocation ends.
    let mut release_at: Vec<Option<Mark>> = vec![None; len];
    // Indexed by instruction, and recorded by `emit_before`: for an `IF`, the
    // op its false path lands on (`PatchKind::IfFalse`), and for a `SELECT`
    // with an `OTHERWISE`, that `OTHERWISE`'s `Op::EnterOtherwise`
    // (`PatchKind::EnterOtherwise`).
    let mut landing: Vec<Option<u32>> = vec![None; len];

    // Indexed by instruction: for an `END` that closes a repeating
    // `DO`/`LOOP`, that loop's own instruction index. Filled in at the header,
    // which is where the kind and the `END`'s position are both known, and read
    // when the walk reaches the `END` itself.
    let mut loop_of_end: Vec<Option<u32>> = vec![None; len];

    // Indexed by instruction: the instruction's own first op, which is
    // `op_of`'s entry except where a `Before` op sits in front of it.
    // Compile-time only -- the driver never wants it, because `PatchKind`'s
    // own doc comment has why the ops that do are emitted here.
    let mut first_op_of = Vec::with_capacity(len);

    // Indexed by instruction: the register top as its clause opens
    // ([`Chunk::register_tops`]).
    let mut register_tops: Vec<u16> = Vec::with_capacity(len);

    // The loop conditions [`Chunk::conditions`] holds, and which loop each
    // belongs to.
    let mut conditions: Vec<Op> = Vec::new();
    let mut condition_of: Vec<Option<ConditionOps>> = vec![None; len];
    let mut loop_test_of: Vec<Option<LoopTestOps>> = vec![None; len];

    for (index, instruction) in body.instructions.iter().enumerate() {
        op_of.push(op_index(&ops)?);
        if let Some(mark) = release_at[index] {
            registers.release(mark);
        }
        register_tops.push(registers.mark().0);
        emit_before(&mut ops, &mut patches, &mut landing, &mut before[index])?;
        first_op_of.push(op_index(&ops)?);
        // This clause's own answers, which is where a per-body `bool` used to
        // stand. The `>K>` op asks a different bit than the value echoes:
        // `TraceMode::results`' own doc names `>>>`/`>K>` as the pair it owns,
        // and `>K>` is printed under `R` and `I` and under neither `A` nor `N`
        // (`do i = 1 to 2`).
        let pool = super::trace_flow::for_emission(plan, index, settings[index]);
        let echoes_values = pool.echoes_values();
        let echoes_keyword = pool.echoes_keyword();
        match &instruction.kind {
            // `DO` and `LOOP` are the same construct under two spellings
            // (`step`'s own arm matches them together), so they compile the
            // same way: the header's own clause region, then the op that runs
            // the construct from what that region evaluated.
            InstructionKind::Do(body_node) | InstructionKind::Loop(body_node) => {
                // Allocated in the *enclosing* scope and released past the
                // whole loop, because these registers are what roots the
                // header's values while the loop runs: a `DO OVER`'s target
                // lives in `LoopState` for the construct's lifetime, and a
                // register released at a body clause's own boundary would be
                // handed out again and overwritten while it is still in use.
                let outer = registers.mark();
                // Named apart from `plan`, which this arm reads too: a slot
                // that compiles natively resolves its reads against it,
                // exactly as an assignment's value does.
                let header_plan = loop_header_plan(body_node);
                let roles = header_plan.roles();
                let mut header = Vec::with_capacity(roles.len());
                for &role in roles {
                    header.push((role, registers.alloc()?));
                }
                let header_top = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                // One group per header expression, and `LoopRun` inside the
                // region behind them: the whole loop runs inside the `DO` clause.
                // `close_region` below is
                // what fixes the region's end, from what was actually pushed.
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                for (slot, &(role, dst)) in header.iter().enumerate() {
                    let slot = instruction_index(slot)?;
                    // The counter's reset value, filed before any header
                    // expression is evaluated; its op echoes it.
                    if role == HeaderRole::Counter {
                        ops.push(Op::Const {
                            dst,
                            konst: consts.intern(b"0")?,
                        });
                        ops.push(Op::LoopHeaderValue { role, src: dst });
                        continue;
                    }
                    // **Each slot decides for itself**, so a header holding one
                    // expression outside the native set keeps native ops for
                    // its others. A slot `loop_header_slot` has no expression
                    // for takes the same answer as one `native_shape` declines,
                    // and they are one arm because they are one answer: the
                    // slot stays a whole `Op::EvalExpr`, which is what it was
                    // before any of it compiled. `run.rs`'s own
                    // `eval_chunk_expr` is where a slot with no expression then
                    // ends up, and it is loud there.
                    match crate::run::loop_header_slot(body, body_node, slot) {
                        Some(expr) if native_shape(expr, Some(NodePath::ROOT)) => push_native(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expr,
                            instruction_index(index)?,
                            slot,
                            Some(NodePath::ROOT),
                            dst,
                        )?,
                        _ => ops.push(Op::EvalExpr {
                            index: instruction_index(index)?,
                            slot,
                            dst,
                        }),
                    }
                    if echoes_keyword && role.keyword().is_some() {
                        ops.push(Op::TraceKeyword { role, src: dst });
                    }
                    ops.push(Op::LoopHeaderValue { role, src: dst });
                }
                // **The header's own registers, exactly: neither more nor
                // fewer.** An equality rather than a bound, because the two
                // directions are different defects and only one of them is
                // harmful.
                debug_assert_eq!(
                    registers.mark().0,
                    header_top.0,
                    "the register top after the header is not the header's own: above it is a \
                     register nothing reuses until the END, below it is a header value handed \
                     to the body while the loop still reads it"
                );
                ops.push(Op::LoopRun {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
                // **The `WHILE`/`UNTIL` condition, compiled beside the stream
                // rather than in it**, for the header to run in place of
                // walking the expression each pass. Its registers are taken
                // after the header's and held as theirs are, to past the
                // `END`: the body's clauses allocate above them, so a pass
                // boundary finds them untouched. No echo op is emitted; the
                // header walks the expression instead whenever `TRACE` shows
                // intermediates.
                if let Some(conditional) = &body_node.conditional
                    && !value_only(body.expr(conditional.condition))
                    && native_shape(body.expr(conditional.condition), Some(NodePath::ROOT))
                    && plan.trace_events().get(index) == Some(&crate::trace::TraceEvent::Keeps)
                {
                    // **A condition that calls or sends is a region of its own
                    // in the stream**, between the header's region and the
                    // body, because a call op parks the region it is in and
                    // the side table has none. The pass boundary jumps to its
                    // `LoopTest`, which opens the header clause; `LoopTested`
                    // decides the pass. Its register is held to past the `END`
                    // as the header's are. No echo op is emitted; the boundary
                    // walks the expression instead whenever `TRACE` shows
                    // intermediates. A loop whose clause can change the
                    // `TRACE` setting (a `TRACE()` call in the condition) keeps
                    // walking it: the region decides once, on entry, and the
                    // rest of a test after the change would go unechoed.
                    let dst = registers.alloc()?;
                    let test = op_index(&ops)?;
                    ops.push(Op::LoopTest {
                        index: instruction_index(index)?,
                        end: 0,
                    });
                    push_native(
                        &mut ops,
                        false,
                        &mut consts,
                        &mut registers,
                        &mut hints,
                        &mut calls,
                        plan,
                        body.expr(conditional.condition),
                        instruction_index(index)?,
                        LOOP_CONDITION_SLOT,
                        Some(NodePath::ROOT),
                        dst,
                    )?;
                    ops.push(Op::LoopTested {
                        index: instruction_index(index)?,
                        src: dst,
                    });
                    let body = op_index(&ops)?;
                    if let Op::LoopTest { end, .. } = &mut ops[usize::cast_from(test)] {
                        *end = body;
                    }
                    loop_test_of[index] = Some(LoopTestOps { test, body });
                } else if let Some(conditional) = &body_node.conditional
                    && value_only(body.expr(conditional.condition))
                    && native_shape(body.expr(conditional.condition), Some(NodePath::ROOT))
                {
                    let dst = registers.alloc()?;
                    let start = op_index(&conditions)?;
                    push_native(
                        &mut conditions,
                        false,
                        &mut consts,
                        &mut registers,
                        &mut hints,
                        &mut calls,
                        plan,
                        body.expr(conditional.condition),
                        instruction_index(index)?,
                        0,
                        Some(NodePath::ROOT),
                        dst,
                    )?;
                    debug_assert!(
                        conditions[usize::cast_from(start)..]
                            .iter()
                            .all(|op| matches!(
                                op,
                                Op::Const { .. }
                                    | Op::LoadConstant { .. }
                                    | Op::Load { .. }
                                    | Op::Arith { .. }
                                    | Op::Binary { .. }
                                    | Op::Prefix { .. }
                            )),
                        "a compiled loop condition holds an op `Interp::run_condition` does not run"
                    );
                    condition_of[index] = Some(ConditionOps {
                        start,
                        end: op_index(&conditions)?,
                        dst,
                    });
                }
                // A repeating loop's `END` carries the op that ends
                // a pass. `Simple` has no pass to end, and its `END` stays the
                // delegating `Op::Exec` region, a no-op clause.
                if let Some(end) = body_node.end
                    && end < len
                    && !matches!(body_node.kind, LoopKind::Simple)
                {
                    loop_of_end[end] = Some(instruction_index(index)?);
                }
                // A labelled block's `END` closes the frame the driver opened
                // over its body.
                if let Some(end) = body_node.end
                    && end < len
                    && matches!(body_node.kind, LoopKind::Simple)
                    && body_node.label.is_some()
                {
                    before[end].push(Before::BlockEnd);
                }
                // Past the `END`, so nothing between here and there can reuse a
                // register the running loop still reads. A loop whose `END` is
                // the body's last instruction has nothing to hang the release
                // on and needs none: there is nothing after it to hand the
                // registers to.
                if let Some(end) = body_node.end
                    && end + 1 < len
                {
                    release_to(&mut release_at[end + 1], outer);
                }
            }
            // The `IF` clause is its condition and nothing else -- the branch
            // it chooses is not inside it, which is the boundary
            // `run.rs`'s own `If` arm measured (`SIGL` reports the `IF`'s line,
            // not the branch's). So the region is the condition's ops and the
            // jump that reads them, and the register the condition lands in is
            // released at its end.
            InstructionKind::If {
                condition,
                false_target,
                ..
            } => {
                let targets = if_targets(&body.instructions, *false_target);
                let mark = registers.mark();
                let dst = registers.alloc()?;
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let mut patch = |op| {
                    patches.push(Patch {
                        op,
                        target: targets.false_target,
                        kind: PatchKind::IfFalse(index),
                    })
                };
                if !push_condition(
                    &mut ops,
                    echoes_values,
                    &mut consts,
                    &mut registers,
                    &mut hints,
                    &mut calls,
                    plan,
                    body.expr(*condition),
                    instruction_index(index)?,
                    dst,
                    ConditionKeyword::If,
                    &mut patch,
                )? {
                    ops.push(Op::EvalExpr {
                        index: instruction_index(index)?,
                        slot: 0,
                        dst,
                    });
                    patch(op_index(&ops)?);
                    ops.push(Op::JumpUnless {
                        reg: dst,
                        target: 0,
                    });
                }
                close_region(&mut ops, at)?;
                registers.release(mark);
                // Only when the false path lands on an `ELSE`: without one,
                // the true branch's fallthrough is already the resume, and a
                // jump to where control was going anyway is an op the driver
                // would execute for nothing.
                // The boundary is owed either way; the jump only when the
                // false path lands on an `ELSE`, since without one the true
                // branch's fallthrough is already the resume and a jump to
                // where control was going anyway is an op the driver would run
                // for nothing.
                let resume = (targets.resume != targets.false_target).then_some(targets.resume);
                before[targets.false_target].push(Before::ThenEnd {
                    resume,
                    owner: index,
                });
            }
            // The `SELECT` clause is its `CASE` expression and nothing else,
            // which is the boundary `run.rs`'s own `Select` arm measured
            // (`SIGL` reports the `SELECT`'s line, not the first `WHEN`'s), and
            // a plain `SELECT` has an empty region rather than none, because
            // the clause and its boundary are owed either way.
            InstructionKind::Select {
                case,
                whens,
                otherwise,
                end,
                ..
            } => {
                let select_end = otherwise_range(len, *end);
                // Allocated in the *enclosing* scope, before any member
                // clause's mark is taken, because the last `WHEN`'s test comes
                // after every earlier `WHEN`'s branch has already run: a
                // register released at a clause boundary inside the construct
                // would be handed out again while this one is still live.
                let outer = registers.mark();
                let case_reg = match case {
                    Some(_) => Some(registers.alloc()?),
                    None => None,
                };
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                if let (Some(dst), Some(case)) =
                    (case_reg, case.as_ref().map(|case| body.expr(*case)))
                {
                    if native_shape(case, Some(NodePath::ROOT)) {
                        push_native(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            case,
                            instruction_index(index)?,
                            0,
                            Some(NodePath::ROOT),
                            dst,
                        )?;
                        ops.push(Op::SelectCase {
                            index: instruction_index(index)?,
                            src: dst,
                        });
                    } else {
                        ops.push(Op::EvalExpr {
                            index: instruction_index(index)?,
                            slot: 0,
                            dst,
                        });
                    }
                }
                close_region(&mut ops, at)?;
                // Past the whole construct, so nothing between here and the
                // `END` can reuse the register. A `select_end` of `len` has no
                // instruction to hang the release on and needs none: there is
                // nothing after it to hand the register to.
                if select_end < len {
                    release_to(&mut release_at[select_end], outer);
                }
                // Where the scan goes once no listed `WHEN` is left, which is
                // also where it starts when there is no `WHEN` at all.
                let no_match = match otherwise {
                    Some(otherwise_index) => {
                        debug_assert!(
                            !before[*otherwise_index]
                                .iter()
                                .any(|before| matches!(before, Before::EnterOtherwise(_))),
                            "two SELECTs want a frame opened in front of instruction \
                             {otherwise_index}, so an OTHERWISE belongs to more than one SELECT"
                        );
                        before[*otherwise_index].push(Before::EnterOtherwise(index));
                        // The `OTHERWISE` body's own end, which is the
                        // `SELECT`'s: `otherwise_frame` gives that frame this
                        // same range.
                        before[select_end].push(Before::SelectBranchEnd);
                        (*otherwise_index, PatchKind::EnterOtherwise(index))
                    }
                    // Landing on the `END` is what makes 7.3 the `END`'s own
                    // clause rather than this `SELECT`'s.
                    None => (select_end, PatchKind::Enter),
                };
                for (position, &when_index) in whens.iter().enumerate() {
                    let (false_target, false_kind) = match whens.get(position + 1) {
                        Some(&next) => (next, PatchKind::Enter),
                        None => no_match,
                    };
                    when_info[when_index] = Some(WhenInfo {
                        select: instruction_index(index)?,
                        case: case_reg,
                        false_target,
                        false_kind,
                    });
                }
                // The first listed `WHEN` is the instruction after this one in
                // every program that parses, so the scan is reached by falling
                // through and a jump to it would be an op the driver runs for
                // nothing. Emitted only when it is not -- a `SELECT` with no
                // listed `WHEN` at all is the case that reaches this.
                if whens.first() != Some(&(index + 1)) {
                    let (target, kind) = match whens.first() {
                        Some(&first) => (first, PatchKind::Enter),
                        None => no_match,
                    };
                    let op = op_index(&ops)?;
                    ops.push(Op::Jump { target: 0 });
                    patches.push(Patch { op, target, kind });
                }
            }
            // A **listed** `WHEN`/`WHEN CASE`: one whose `SELECT` collected it,
            // which is what `when_info` holds an entry for. The parser refuses
            // a `WHEN` anywhere else with 9.1, so every one is listed; the
            // arm below is for one that is not.
            InstructionKind::When { .. } | InstructionKind::WhenCase { .. }
                if when_info[index].is_some() =>
            {
                let info = when_info[index]
                    .take()
                    .expect("the guard above just observed one");
                // **Where this `WHEN`'s branch runs out, taken from the
                // instruction rather than from the `SELECT`'s scan chain.**
                // `Interp::when_frame` reads `when_targets` for the frame's
                // own `op_end`, and the two need not agree, so the branch's end
                // is the instruction's own.
                before[when_targets(&instruction.kind, len).body_end].push(Before::SelectBranchEnd);
                let mark = registers.mark();
                let dst = registers.alloc()?;
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                // **Only a plain `WHEN`'s condition promotes.** A `WhenCase`
                // holds a list of values the enclosing `SELECT CASE`'s value
                // is sent `==` with through `Interp::test_case_when`, which is
                // not a condition at all: it traces two `>>>` lines per value.
                // So it stays on `Op::WhenTest`, which is the op that does
                // that whole job, and so does a `When` whose condition
                // `native_shape` declines.
                let mut patch = |op| {
                    patches.push(Patch {
                        op,
                        target: info.false_target,
                        kind: info.false_kind,
                    })
                };
                let native = match &instruction.kind {
                    InstructionKind::When { condition, .. } => push_condition(
                        &mut ops,
                        echoes_values,
                        &mut consts,
                        &mut registers,
                        &mut hints,
                        &mut calls,
                        plan,
                        body.expr(*condition),
                        instruction_index(index)?,
                        dst,
                        ConditionKeyword::When,
                        &mut patch,
                    )?,
                    _ => false,
                };
                if !native {
                    ops.push(Op::WhenTest {
                        index: instruction_index(index)?,
                        case: info.case,
                        dst,
                    });
                    patch(op_index(&ops)?);
                    ops.push(Op::JumpUnless {
                        reg: dst,
                        target: 0,
                    });
                }
                // Closed before `EnterWhen`, which opens the branch's frame and
                // is the branch's business rather than the clause's.
                close_region(&mut ops, at)?;
                registers.release(mark);
                ops.push(Op::EnterWhen {
                    select: info.select,
                    when: instruction_index(index)?,
                });
            }
            // An assignment's clause is its value expression and the write,
            // both inside the region: the write is what the clause *is*, and
            // the boundary that follows it is where a `CALL ON` handler queued
            // by the value expression runs -- measured, the handler sees the
            // assignment already done.
            InstructionKind::Assignment { target, value } => {
                let mark = registers.mark();
                let dst = registers.alloc()?;
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                push_value(
                    &mut ops,
                    echoes_values,
                    &mut consts,
                    &mut registers,
                    &mut hints,
                    &mut calls,
                    plan,
                    body.expr(*value),
                    instruction_index(index)?,
                    0,
                    dst,
                )?;
                let shared = share_tail_key(
                    &mut ops[usize::cast_from(at)..],
                    body.expr(*target),
                    body.expr(*value),
                );
                ops.push(Op::Store {
                    index: instruction_index(index)?,
                    at: write_slot(plan, body.expr(*target)),
                    src: dst,
                    shared,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // A `SAY` is the same shape with the print in place of the write,
            // and one register fewer when it has no expression: the bare form
            // prints a blank line and traces the null string, which is a
            // decision `Op::Say` carries rather than an empty register.
            InstructionKind::Say { expression } => {
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match expression {
                    Some(expression) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            body.expr(*expression),
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    None => None,
                };
                ops.push(Op::Say {
                    index: instruction_index(index)?,
                    src,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // `SIGNAL`, all three forms. The `SAY` shape, with the expression
            // present only for `VALUE` -- and it goes through `push_value`
            // like any other, so a `SIGNAL VALUE` whose expression is a
            // literal or a bare symbol reaches the same native op an
            // assignment's value would.
            InstructionKind::Signal(signal) => {
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match &**signal {
                    interrexx_parse::Signal::Value(expression) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            body.expr(*expression),
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    interrexx_parse::Signal::Label(_) | interrexx_parse::Signal::Trap(_) => None,
                };
                ops.push(Op::Signal {
                    index: instruction_index(index)?,
                    src,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // `PARSE`, `ARG` and `PULL`, which are one instruction with three
            // spellings and reach one `exec_parse`. The `SAY` shape, with the
            // expression present only for `PARSE VALUE expr WITH` -- every
            // other source reads a variable, an argument list or a line, none
            // of which is an expression a register could hold. The template then
            // compiles to its walk, with runs for the operands the stream
            // evaluates where `compiled_template` takes it, and otherwise to
            // the walk's opening alone.
            InstructionKind::Parse(parse)
            | InstructionKind::Arg(parse)
            | InstructionKind::Pull(parse) => {
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match &parse.source {
                    ParseSource::Value(Some(expression)) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            body.expr(*expression),
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    _ => None,
                };
                // Whether a line is traced is the opening's to ask when it
                // runs: a clause that may trace, because something in the
                // body can change the setting, is still walked while it does
                // not. Its operands get no ops, since those carry no echoes.
                let opening = parse_opening(parse, plan);
                let runs = !echoes_values && !echoes_keyword;
                let known_traced = matches!(pool, super::trace_flow::Setting::Known(setting)
                    if setting.results() || setting.intermediates());
                if !known_traced
                    && let Some(scratch) = src
                    && let Some(writes) = plan.literal_parse(index)
                {
                    debug_assert!(
                        parse
                            .template
                            .iter()
                            .flatten()
                            .flat_map(|trigger| trigger.targets.iter().flatten())
                            .map(|target| match body.expr(target).kind() {
                                ExprView::Variable(id) => plan.slot_for_symbol(id),
                                _ => None,
                            })
                            .eq(writes.iter().map(|&(slot, _)| Some(usize::cast_from(slot)))),
                        "prepared PARSE writes disagree with the target slots in this body's plan"
                    );
                    ops.push(Op::ParseBegin {
                        index: instruction_index(index)?,
                        src: scratch,
                        base: Op::ARG_OMITTED,
                        to: 0,
                        opening: ParseOpening::Literal,
                        strings: 0,
                    });
                    ops.extend(writes.iter().map(|&(slot, value)| Op::ParseStore {
                        slot,
                        value,
                        scratch,
                    }));
                } else {
                    push_template(
                        body,
                        &mut ops,
                        &mut consts,
                        &mut registers,
                        &mut hints,
                        &mut calls,
                        plan,
                        parse,
                        instruction_index(index)?,
                        src,
                        opening,
                        runs,
                    )?;
                }
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // `RETURN` and `EXIT` are the `SAY` shape with a `Flow` in place
            // of the print, and one arm rather than two because the keyword is
            // all that differs -- read off the kind here exactly as `step`'s
            // own `PUSH`/`QUEUE` arm reads its end of the queue.
            InstructionKind::Return { expression } | InstructionKind::Exit { expression } => {
                let keyword = if matches!(instruction.kind, InstructionKind::Return { .. }) {
                    ReturnKeyword::Return
                } else {
                    ReturnKeyword::Exit
                };
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match expression {
                    Some(expression) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            body.expr(*expression),
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    None => None,
                };
                ops.push(Op::Return {
                    index: instruction_index(index)?,
                    src,
                    keyword,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // `PUSH` and `QUEUE` are the `SAY` shape with the queue in place
            // of the print, one arm for the reason the pair above is one.
            InstructionKind::Push { expression } | InstructionKind::Queue { expression } => {
                let keyword = if matches!(instruction.kind, InstructionKind::Push { .. }) {
                    QueueKeyword::Push
                } else {
                    QueueKeyword::Queue
                };
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match expression {
                    Some(expression) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            body.expr(*expression),
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    None => None,
                };
                ops.push(Op::Queue {
                    index: instruction_index(index)?,
                    src,
                    keyword,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // A named or qualified CALL: argument k is expression slot k.
            // Where an argument cannot be addressed, the call walks its
            // arguments and traces them itself.
            InstructionKind::Call(call)
                if matches!(&**call, Call::Named { .. } | Call::Qualified { .. }) =>
            {
                let (args, qualified) = match &**call {
                    Call::Named { args, .. } => (args, false),
                    Call::Qualified { args, .. } => (args, true),
                    _ => unreachable!("the guard above admits only named or qualified calls"),
                };
                let reserve_site = |calls: &mut Calls| {
                    if qualified {
                        Ok(super::UNCACHED_CALL)
                    } else {
                        calls.reserve()
                    }
                };
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                // **The arguments become ops of their own where they can**,
                // exactly as `ExprView::Call`'s arm does it. Argument `k` is
                // the instruction's expression slot `k`.
                let args = body.args(args);
                let native = u16::try_from(args.len()).is_ok()
                    && args
                        .iter()
                        .all(|arg| arg.is_none_or(|expr| native_shape(expr, Some(NodePath::ROOT))));
                if native {
                    let argc = push_arguments(
                        &mut ops,
                        echoes_values,
                        &mut consts,
                        &mut registers,
                        &mut hints,
                        &mut calls,
                        plan,
                        args,
                        instruction_index(index)?,
                        |k| {
                            (
                                u32::try_from(k).expect("an argument of one clause"),
                                Some(NodePath::ROOT),
                            )
                        },
                        true,
                    )?;
                    ops.push(Op::CallNamed {
                        index: instruction_index(index)?,
                        site: reserve_site(&mut calls)?,
                        argc,
                    });
                } else {
                    ops.push(Op::Call {
                        index: instruction_index(index)?,
                        site: reserve_site(&mut calls)?,
                    });
                }
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // A message send as a whole clause whose term compiles: the
            // receiver, the assigned value of the message-assignment form, the
            // arguments, and the send. The term is slot 0 and the assigned
            // value slot 1.
            InstructionKind::Message { term, value }
                if native_message_instruction(
                    body.expr(*term),
                    value.as_ref().map(|value| body.expr(*value)),
                ) =>
            {
                let ExprView::Message {
                    target,
                    name,
                    super_class,
                    args,
                    ..
                } = body.expr(*term).kind()
                else {
                    unreachable!("the guard above admits only a message term")
                };
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let recv = registers.alloc()?;
                push_native(
                    &mut ops,
                    echoes_values,
                    &mut consts,
                    &mut registers,
                    &mut hints,
                    &mut calls,
                    plan,
                    target,
                    instruction_index(index)?,
                    0,
                    NodePath::ROOT.nth(0),
                    recv,
                )?;
                let scope = push_scope(
                    &mut ops,
                    echoes_values,
                    &mut consts,
                    &mut registers,
                    &mut hints,
                    &mut calls,
                    plan,
                    super_class,
                    instruction_index(index)?,
                    recv,
                )?;
                let assigned = match value.as_ref().map(|value| body.expr(*value)) {
                    Some(value) => {
                        let src = registers.alloc()?;
                        push_native(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            value,
                            instruction_index(index)?,
                            1,
                            Some(NodePath::ROOT),
                            src,
                        )?;
                        ops.push(Op::PushArg { src });
                        if echoes_values {
                            ops.push(Op::TraceArgument { src });
                        }
                        1
                    }
                    None => 0,
                };
                let argc = push_arguments(
                    &mut ops,
                    echoes_values,
                    &mut consts,
                    &mut registers,
                    &mut hints,
                    &mut calls,
                    plan,
                    args,
                    instruction_index(index)?,
                    |k| (0, NodePath::ROOT.nth(k + 1)),
                    true,
                )?
                .checked_add(assigned)
                .ok_or(ChunkTooLarge {
                    what: "call arguments past u16",
                })?;
                // The message-assignment form sends the term's name with `=`
                // appended.
                let (name, form) = match value {
                    Some(_) => ([&name[..], b"="].concat(), SendForm::Assign),
                    None => (name.to_vec(), SendForm::Clause),
                };
                let site = calls.reserve_send(name.into_boxed_slice())?;
                ops.push(if scope == Op::NO_SCOPE {
                    Op::Send {
                        site,
                        recv,
                        argc,
                        dst: recv,
                        form,
                    }
                } else {
                    Op::SendWith {
                        site,
                        recv,
                        argc,
                        dst: recv,
                        scope,
                        form,
                    }
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // A message send as a whole clause whose term does not compile.
            // No register and no expression slot -- `Interp::exec_message`
            // evaluates the term itself, so what this promotion decides is the
            // clause region around it and nothing about the send.
            InstructionKind::Message { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Message {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
            // `EXPOSE`. No register and no expression slot: the names come
            // out of the instruction, and a `VariableRef::Indirect` reads its
            // selector through `Interp::read_list_selector`, so what this
            // promotion decides is the clause region around it and nothing
            // about the binding.
            InstructionKind::Expose { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Expose {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
            // `LEAVE`/`ITERATE`. No register and no expression slot: the name
            // is in the instruction and the origin is captured at run time
            // from the clause the region has open.
            InstructionKind::Leave { .. } | InstructionKind::Iterate { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Escape {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
            // The marker instructions: `step` answers `Ok(Flow::Next)` for
            // each and computes nothing, so the clause region is the whole of
            // what they do and it holds no op but its own echo
            // ([`Op::Clause`]'s own doc comment).
            InstructionKind::Nop
            | InstructionKind::Then
            | InstructionKind::Else { .. }
            | InstructionKind::Otherwise
            | InstructionKind::Label { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                close_region(&mut ops, at)?;
            }
            // The `END` of a repeating loop, which the flattened
            // form reaches by falling out of the body rather than by a range
            // check. An `END` closing anything else keeps the delegating
            // `Op::Exec` region below.
            InstructionKind::End { .. } if loop_of_end[index].is_some() => {
                ops.push(Op::LoopNext {
                    index: loop_of_end[index].expect("the guard just observed it"),
                });
            }
            // An instruction whose expression is its first act: the
            // expression's ops, then `Op::ExecValue`, which does the rest.
            // A `REPLY` has `Op::ReplyCheck` ahead of them, the check it makes
            // before evaluating.
            kind if let Some(expression) = crate::Interp::exec_value_expression(body, kind) => {
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                if matches!(kind, InstructionKind::Reply { .. }) {
                    ops.push(Op::ReplyCheck {
                        index: instruction_index(index)?,
                    });
                }
                let dst = registers.alloc()?;
                push_value(
                    &mut ops,
                    echoes_values,
                    &mut consts,
                    &mut registers,
                    &mut hints,
                    &mut calls,
                    plan,
                    expression,
                    instruction_index(index)?,
                    0,
                    dst,
                )?;
                // An instruction whose rest can transfer control or run Rexx
                // code of its own runs as `Op::Exec` does, from the value.
                ops.push(if crate::Interp::exec_value_transfers(&instruction.kind) {
                    Op::ExecWith {
                        index: instruction_index(index)?,
                        src: dst,
                    }
                } else {
                    Op::ExecValue {
                        index: instruction_index(index)?,
                        src: dst,
                    }
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // Bare FORWARD has no option expressions to compile.
            InstructionKind::Forward(forward)
                if forward.array.as_ref().is_none_or(|array| {
                    array.len() <= usize::cast_from(u32::from(u16::MAX) - FORWARD_ARRAY_SLOT)
                }) && (forward.array.is_some()
                    || crate::run::forward_option_parts(body, forward)
                        .iter()
                        .any(|(_, expr)| expr.is_some())) =>
            {
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let clause = instruction_index(index)?;
                ops.push(Op::Clause {
                    index: clause,
                    end: 0,
                });
                push_echo(&mut ops, echoes(trace, instruction), clause);
                ops.push(Op::ForwardPart {
                    index: clause,
                    part: ForwardPart::Method,
                    src: Op::ARG_OMITTED,
                });
                let scalars = crate::run::forward_option_parts(body, forward);
                let base = registers.alloc()?;
                for _ in 1..scalars.iter().filter(|(_, expr)| expr.is_some()).count() {
                    registers.alloc()?;
                }
                let mut src = base;
                for (slot, (part, expr)) in scalars.into_iter().enumerate() {
                    let Some(expr) = expr else { continue };
                    push_value(
                        &mut ops,
                        echoes_values,
                        &mut consts,
                        &mut registers,
                        &mut hints,
                        &mut calls,
                        plan,
                        expr,
                        clause,
                        u32::try_from(slot).expect("one of the option parts"),
                        src,
                    )?;
                    ops.push(Op::ForwardPart {
                        index: clause,
                        part,
                        src,
                    });
                    src += 1;
                }
                if let Some(items) = &forward.array {
                    for (slot, item) in items.iter().enumerate() {
                        let src = if let Some(expr) = item {
                            let src = registers.alloc()?;
                            push_value(
                                &mut ops,
                                echoes_values,
                                &mut consts,
                                &mut registers,
                                &mut hints,
                                &mut calls,
                                plan,
                                body.expr(*expr),
                                clause,
                                FORWARD_ARRAY_SLOT
                                    + u32::try_from(slot).expect("within the bound the arm checks"),
                                src,
                            )?;
                            src
                        } else {
                            Op::ARG_OMITTED
                        };
                        ops.push(Op::PushArg { src });
                        if echoes_values {
                            ops.push(Op::TraceArgument { src });
                        }
                    }
                }
                ops.push(Op::ExecWith {
                    index: clause,
                    src: base,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // A `RAISE` with an expression: each expression's ops into its
            // part's registers, each followed by the `Op::RaisePart` that does
            // what the instruction does with that value before it evaluates
            // the next (its trace line, its conversion, its checks), then
            // `Op::ExecWith`, which raises from what the parts left. A `RAISE`
            // with none stays the delegating `Op::Exec` region below.
            InstructionKind::Raise(raise)
                if let Some(layout) = crate::run::RaiseLayout::of(raise) =>
            {
                let mark = registers.mark();
                let at_op = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                let clause = instruction_index(index)?;
                ops.push(Op::Clause {
                    index: clause,
                    end: 0,
                });
                push_echo(&mut ops, echo, clause);
                let base = registers.alloc()?;
                for _ in 1..layout.len {
                    registers.alloc()?;
                }
                use crate::run::RaisePart;
                // In the order `r[instr.raise.order]` evaluates them: each
                // expression into its register and the part that follows it,
                // or a part with no expression of its own.
                let at = |offset: Option<u16>| offset.map(|offset| base + offset);
                let mut steps: Vec<(Option<(ExprRef<'_>, u32)>, RaisePart, u16)> = Vec::new();
                fn value<'a>(
                    expr: Option<ExprRef<'a>>,
                    slot: u32,
                    dst: Option<u16>,
                    part: RaisePart,
                ) -> Option<(Option<(ExprRef<'a>, u32)>, RaisePart, u16)> {
                    expr.zip(dst)
                        .map(|(expr, dst)| (Some((expr, slot)), part, dst))
                }
                steps.extend(value(
                    raise.rc.as_ref().map(|e| body.expr(*e)),
                    0,
                    at(layout.code),
                    RaisePart::Code,
                ));
                steps.extend(value(
                    raise.description.as_ref().map(|e| body.expr(*e)),
                    1,
                    at(layout.description),
                    RaisePart::Description,
                ));
                steps.extend(value(
                    raise.additional.as_ref().map(|e| body.expr(*e)),
                    2,
                    at(layout.additional),
                    RaisePart::Additional,
                ));
                if let (Some(items), Some(array)) = (&raise.array, at(layout.array)) {
                    for (k, item) in items.iter().enumerate() {
                        let slot = u32::try_from(k).ok().and_then(|k| k.checked_add(4)).ok_or(
                            ChunkTooLarge {
                                what: "RAISE ARRAY items past u32",
                            },
                        )?;
                        let dst = u16::try_from(k + 1)
                            .ok()
                            .and_then(|k| array.checked_add(k))
                            .ok_or(ChunkTooLarge { what: "registers" })?;
                        match item {
                            Some(expr) => {
                                steps.push((Some((body.expr(*expr), slot)), RaisePart::Item, dst))
                            }
                            None => steps.push((None, RaisePart::Item, Op::ARG_OMITTED)),
                        }
                    }
                    steps.push((None, RaisePart::Array, array));
                }
                if let Some(result) = &raise.result {
                    steps.extend(value(
                        result.value.as_ref().map(|e| body.expr(*e)),
                        3,
                        at(layout.result),
                        RaisePart::Result,
                    ));
                }
                for (expr, part, src) in steps {
                    if let Some((expr, slot)) = expr {
                        push_value(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expr,
                            clause,
                            slot,
                            src,
                        )?;
                    }
                    // An item's part is its trace lines and nothing else.
                    if part == RaisePart::Item && !echoes_values {
                        continue;
                    }
                    ops.push(Op::RaisePart {
                        index: clause,
                        part,
                        src,
                    });
                }
                ops.push(Op::ExecWith {
                    index: clause,
                    src: base,
                });
                close_region(&mut ops, at_op)?;
                registers.release(mark);
            }
            // Only clauses with defaults need a conditional region. The
            // default-free binder still runs as a single Op::Exec.
            InstructionKind::Use(use_)
                if let Use::Arg {
                    strict, targets, ..
                } = &**use_
                    && u16::try_from(targets.len()).is_ok()
                    && targets.iter().flatten().any(|t| t.default.is_some()) =>
            {
                let at = op_index(&ops)?;
                let clause = instruction_index(index)?;
                ops.push(Op::Clause {
                    index: clause,
                    end: 0,
                });
                push_echo(&mut ops, echoes(trace, instruction), clause);
                if *strict {
                    ops.push(Op::UseArg(UseArgOp::CheckCount));
                }
                for (slot, target) in targets.iter().enumerate() {
                    let Some(target) = target else { continue };
                    let slot = u16::try_from(slot).expect("the targets fit u16");
                    let bind = op_index(&ops)?;
                    ops.push(Op::UseArg(UseArgOp::Bind { slot, skip: 0 }));
                    let Some(default) = &target.default else {
                        continue;
                    };
                    let mark = registers.mark();
                    let src = registers.alloc()?;
                    push_value(
                        &mut ops,
                        echoes_values,
                        &mut consts,
                        &mut registers,
                        &mut hints,
                        &mut calls,
                        plan,
                        body.expr(*default),
                        clause,
                        u32::from(slot),
                        src,
                    )?;
                    ops.push(Op::UseArg(UseArgOp::BindDefault { slot, src }));
                    let skip = op_index(&ops)? - bind - 1;
                    ops[usize::cast_from(bind)] = Op::UseArg(UseArgOp::Bind { slot, skip });
                    registers.release(mark);
                }
                close_region(&mut ops, at)?;
            }
            // A `GUARD ... WHEN` whose condition only reads and compares: the
            // condition's ops go to the side table `Chunk::conditions`, where
            // `exec_guard` runs them on every test (the first and each one
            // after a wake), as a loop's `WHILE` runs there; the clause itself
            // stays the delegating `Op::Exec` region below. The registers are
            // the clause's own while it runs, which nothing else holds.
            InstructionKind::Guard(guard)
                if let Some(condition) = guard.condition.as_ref().map(|c| body.expr(*c))
                    && value_only(condition)
                    && native_shape(condition, Some(NodePath::ROOT)) =>
            {
                let mark = registers.mark();
                let dst = registers.alloc()?;
                let start = op_index(&conditions)?;
                push_native(
                    &mut conditions,
                    false,
                    &mut consts,
                    &mut registers,
                    &mut hints,
                    &mut calls,
                    plan,
                    condition,
                    instruction_index(index)?,
                    0,
                    Some(NodePath::ROOT),
                    dst,
                )?;
                condition_of[index] = Some(ConditionOps {
                    start,
                    end: op_index(&conditions)?,
                    dst,
                });
                registers.release(mark);
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Exec {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
            // The kinds whose whole execution is one call into
            // `Interp::exec_instruction`. What the promotion decides is the
            // clause region around that call and nothing about the call: the
            // region owes the echo, the boundary, the temps frame, the
            // deadline count and the failing clause's site, and
            // [`Op::Exec`] owes the work.
            InstructionKind::Command { .. }
            | InstructionKind::End { .. }
            | InstructionKind::Catch(_)
            | InstructionKind::Drop { .. }
            | InstructionKind::Call(..)
            | InstructionKind::Procedure { .. }
            | InstructionKind::Interpret { .. }
            | InstructionKind::Guard { .. }
            | InstructionKind::Reply { .. }
            | InstructionKind::Forward { .. }
            | InstructionKind::Raise { .. }
            | InstructionKind::Use { .. }
            | InstructionKind::Numeric { .. }
            | InstructionKind::Address { .. }
            | InstructionKind::Trace { .. }
            | InstructionKind::Options { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Exec {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
            // A `WHEN` or `WHEN CASE` no `SELECT` collected, which the parser
            // does not produce (9.1). It is compiled as a clause whose
            // `Op::Exec` reaches `Interp::exec_instruction`, which refuses it
            // loudly rather than guessing at its branch.
            InstructionKind::When { .. } | InstructionKind::WhenCase { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Exec {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
        }
    }
    // A branch that ends at the body's own end still owes its boundary, and
    // there is no instruction iteration left to emit it in.
    emit_before(&mut ops, &mut patches, &mut landing, &mut before[len])?;
    // One entry past the last instruction, pushed after the loop above:
    // `run_bounded`'s absorption guard is inclusive, so a construct's
    // resume point can be `end`, one past its last instruction, and a table
    // that stopped at `len - 1` would panic there instead of failing loudly
    // at compile.
    let end = op_index(&ops)?;
    op_of.push(end);
    first_op_of.push(end);

    for patch in patches {
        // In range for every target: both tables have an entry per
        // instruction plus the end one, and `if_targets` clamps `None` to
        // `len`.
        let target = match patch.kind {
            PatchKind::Enter => first_op_of[patch.target],
            PatchKind::Resume => op_of[patch.target],
            // Every `IF` pushed its `ThenEnd` at the instruction its false
            // path targets, and every instruction's entries and the end
            // one's were emitted above.
            PatchKind::IfFalse(owner) => landing[owner]
                .expect("an IF's ThenEnd is emitted at its false target before patching"),
            // Likewise every `SELECT` with an `OTHERWISE` pushed its
            // `EnterOtherwise` in front of it.
            PatchKind::EnterOtherwise(select) => {
                landing[select].expect("a SELECT's EnterOtherwise is emitted before patching")
            }
        };
        match &mut ops[usize::cast_from(patch.op)] {
            Op::Jump { target: slot }
            | Op::JumpUnless { target: slot, .. }
            | Op::ConditionJump { target: slot, .. } => *slot = target,
            _ => unreachable!("a patch names the op it was recorded beside"),
        }
    }

    assert_trace_ops_open_a_clause_region(&ops);
    assert_literal_echoes_follow_their_load(&ops);
    assert_read_echoes_follow_their_load(&ops);
    assert_operator_echoes_follow_their_op(&ops);
    assert_prefix_echoes_follow_their_op(&ops);
    assert_call_echoes_follow_their_op(&ops);
    assert_keyword_echoes_precede_their_value(&ops);
    assert_region_ops_name_their_clause(&ops);
    assert_exec_regions_hold_nothing_else(&ops);

    let consts = consts.values;
    // Sized from the ops rather than from the program's symbol table, which
    // `compile` does not have. Both the main stream and side-table conditions
    // cache constant symbols, so use the widest id either of them names.
    // `Chunk::interned_symbols` has why the gap in front of a high id is the
    // right trade.
    let widest_constant_symbol = ops
        .iter()
        .chain(&conditions)
        .filter_map(|op| match op {
            Op::LoadConstant { symbol, .. } => Some(symbol.index()),
            _ => None,
        })
        .max();
    let registers = registers.high_water();
    Ok(Chunk {
        trace,
        settled,
        #[cfg(debug_assertions)]
        settings,
        ops: super::valid::ValidOps::new(ops, registers)?,
        #[cfg(feature = "jit")]
        jit: Box::new(super::drive::jit_host::Loops::new(op_of.len())),
        op_of,
        registers,
        openings: clause_openings(body, plan, &register_tops),
        interned: vec![std::cell::Cell::new(interrexx_core::ObjRef::NIL); consts.len()],
        interned_symbols: vec![
            std::cell::Cell::new(interrexx_core::ObjRef::NIL);
            widest_constant_symbol.map_or(0, |widest| widest + 1)
        ],
        consts,
        hints,
        calls,
        conditions: super::valid::ValidOps::new(conditions, registers)?,
        condition_of: condition_of.into_boxed_slice(),
        loop_test_of: loop_test_of.into_boxed_slice(),
    })
}

/// Whether `expr` is built from reads, literals and operators alone, which is
/// what [`Chunk::conditions`] may hold: no call, which would have to park the
/// driver, and no comma list, whose elements are judged one by one.
fn value_only(expr: ExprRef<'_>) -> bool {
    match expr.kind() {
        ExprView::Literal(_)
        | ExprView::Constant(_)
        | ExprView::Variable(_)
        | ExprView::Stem(_)
        | ExprView::Compound(_)
        | ExprView::DotVariable(_) => true,
        ExprView::Binary { left, right, .. } => value_only(left) && value_only(right),
        ExprView::Prefix { operand, .. } => value_only(operand),
        _ => false,
    }
}

/// The ops that leave expression `slot` of instruction `index` in register
/// `dst`.
#[expect(
    clippy::too_many_arguments,
    reason = "the emission sinks, the plan a read resolves its slot against, and the address an \
              EvalExpr or a call op has to name"
)]
fn push_value<'a>(
    ops: &mut Vec<Op>,
    echoes_values: bool,
    consts: &mut Constants<'a>,
    registers: &mut Registers,
    hints: &mut Hints,
    calls: &mut Calls,
    plan: &Plan,
    expr: ExprRef<'a>,
    index: u32,
    slot: u32,
    dst: u16,
) -> Result<(), ChunkTooLarge> {
    if native_shape(expr, Some(NodePath::ROOT)) {
        push_native(
            ops,
            echoes_values,
            consts,
            registers,
            hints,
            calls,
            plan,
            expr,
            index,
            slot,
            Some(NodePath::ROOT),
            dst,
        )
    } else {
        ops.push(Op::EvalExpr { index, slot, dst });
        Ok(())
    }
}

/// A native condition branches out of its one clause region when any of its
/// values is false. Only a comma list addresses its values in slots starting
/// at one; slot zero is the unsplit condition.
#[expect(
    clippy::too_many_arguments,
    reason = "the expression emitter's state plus the branch patch"
)]
fn push_condition<'a>(
    ops: &mut Vec<Op>,
    echoes_values: bool,
    consts: &mut Constants<'a>,
    registers: &mut Registers,
    hints: &mut Hints,
    calls: &mut Calls,
    plan: &Plan,
    condition: ExprRef<'a>,
    index: u32,
    dst: u16,
    keyword: ConditionKeyword,
    patch: &mut impl FnMut(u32),
) -> Result<bool, ChunkTooLarge> {
    let comma_list = matches!(condition.kind(), ExprView::Logical(_));
    let items: Vec<ExprRef<'a>> = match condition.kind() {
        ExprView::Logical(items) => items.iter().flatten().collect(),
        _ => vec![condition],
    };
    if u16::try_from(items.len()).is_err()
        || items
            .iter()
            .any(|&item| !native_shape(item, Some(NodePath::ROOT)))
    {
        return Ok(false);
    }
    for (item_at, &item) in items.iter().enumerate() {
        let slot = if comma_list {
            u32::try_from(item_at).expect("an item of one clause") + 1
        } else {
            0
        };
        push_native(
            ops,
            echoes_values,
            consts,
            registers,
            hints,
            calls,
            plan,
            item,
            index,
            slot,
            Some(NodePath::ROOT),
            dst,
        )?;
        let keyword = if !comma_list {
            keyword
        } else if item_at + 1 == items.len() {
            ConditionKeyword::ListLast
        } else {
            ConditionKeyword::ListItem
        };
        patch(op_index(ops)?);
        ops.push(Op::ConditionJump {
            index,
            reg: dst,
            keyword,
            target: 0,
        });
    }
    Ok(true)
}

/// How the opening of `parse` takes it, from what the template and the plan
/// already say, and for `PARSE ARG` how many parse strings it reads. A
/// `PARSE VAR` name with a dot in it is a stem or a compound, which the plan
/// splits; `PARSE VAR name w1 ... rest` keeps the engine's own walk for that
/// shape.
fn parse_opening(parse: &interrexx_parse::Parse, plan: &Plan) -> (ParseOpening, u8) {
    use interrexx_parse::TriggerKind;
    if parse.upper || parse.lower {
        return (ParseOpening::Engine, 0);
    }
    match &parse.source {
        ParseSource::Value(_) => (ParseOpening::Value, 0),
        ParseSource::Var(id) => {
            let words = matches!(
                parse.template.as_slice(),
                [Some(trigger)] if trigger.kind == TriggerKind::End
            );
            if words || plan.compound(*id).is_some() {
                (ParseOpening::Engine, 0)
            } else {
                (ParseOpening::Var, 0)
            }
        }
        ParseSource::Arg => {
            let strings = parse
                .template
                .iter()
                .filter(|entry| entry.is_none())
                .count()
                + 1;
            match u8::try_from(strings) {
                Ok(strings) => (ParseOpening::Arg, strings),
                Err(_) => (ParseOpening::Engine, 0),
            }
        }
        ParseSource::LineIn | ParseSource::Pull | ParseSource::Source | ParseSource::Version => {
            (ParseOpening::Engine, 0)
        }
    }
}

/// Whether the template of `parse` is walked by [`Op::ParseBegin`] and its
/// [`Op::ParseRun`]s: a source the compiled walk reads in place, no fold, and
/// every trigger operand an expression the stream can evaluate. `PARSE VAR
/// name w1 ... rest` keeps the template engine's own walk for that shape.
fn compiled_template(body: &CodeBody, parse: &interrexx_parse::Parse) -> bool {
    use interrexx_parse::TriggerKind;
    let source = match &parse.source {
        ParseSource::Value(_) | ParseSource::Arg => true,
        ParseSource::Var(_) => !matches!(
            parse.template.as_slice(),
            [Some(trigger)] if trigger.kind == TriggerKind::End
        ),
        ParseSource::LineIn | ParseSource::Pull | ParseSource::Source | ParseSource::Version => {
            false
        }
    };
    source
        && !(parse.upper || parse.lower)
        && u16::try_from(parse.template.len()).is_ok()
        && parse.template.iter().flatten().all(|trigger| {
            trigger
                .value
                .as_ref()
                .is_none_or(|operand| native_shape(body.expr(*operand), Some(NodePath::ROOT)))
        })
}

/// The ops of a `PARSE`: the walk's opening, alone for an opening that hands
/// the clause to the engine
/// and otherwise, for a template [`compiled_template`] accepts, the opening,
/// which walks the entries up to the first with an operand that is not a
/// literal, and per such entry its operand's ops and the run from it to the
/// next. An operand's ops follow the run before it, because it is evaluated
/// after the earlier pieces' targets are assigned
/// (`r[parse.patterns.variable]`). A literal or a simple variable is read
/// where its run reaches it, as the template engine reads it.
#[expect(
    clippy::too_many_arguments,
    reason = "the emission sinks, the plan an operand's reads resolve against, the clause the \
              ops name, how its opening takes it, and whether its operands may get ops"
)]
fn push_template<'a>(
    body: &'a CodeBody,
    ops: &mut Vec<Op>,
    consts: &mut Constants<'a>,
    registers: &mut Registers,
    hints: &mut Hints,
    calls: &mut Calls,
    plan: &Plan,
    parse: &'a interrexx_parse::Parse,
    index: u32,
    src: Option<u16>,
    (opening, strings): (ParseOpening, u8),
    untraced: bool,
) -> Result<(), ChunkTooLarge> {
    let runs = untraced && opening != ParseOpening::Engine && compiled_template(body, parse);
    /// The operand of `entry` that the stream evaluates: one that is not a
    /// literal or a simple variable, which the walk reads where it reaches
    /// it, as the template engine reads them.
    fn evaluated<'a>(
        body: &'a CodeBody,
        entry: &'a Option<interrexx_parse::ParseTrigger>,
    ) -> Option<ExprRef<'a>> {
        entry
            .as_ref()
            .and_then(|trigger| trigger.value.as_ref())
            .map(|operand| body.expr(*operand))
            .filter(|operand| {
                !matches!(
                    operand.kind(),
                    ExprView::Literal(_) | ExprView::Constant(_) | ExprView::Variable(_)
                )
            })
    }
    let width = |at: usize| u16::try_from(at).map_err(|_| ChunkTooLarge { what: "template" });
    // The entries whose operand the stream evaluates, each the start of a
    // run, and the template's end. A template the walk runs whole has none.
    let starts: Vec<usize> = if runs {
        (parse.template.iter().enumerate())
            .filter(|(_, entry)| evaluated(body, entry).is_some())
            .map(|(at, _)| at)
            .collect()
    } else {
        Vec::new()
    };
    let end = parse.template.len();
    let base = if starts.is_empty() {
        Op::ARG_OMITTED
    } else {
        let base = registers.alloc()?;
        for _ in 1..crate::parse_template::WALK_REGISTERS {
            registers.alloc()?;
        }
        base
    };
    ops.push(Op::ParseBegin {
        index,
        src: src.unwrap_or(Op::ARG_OMITTED),
        base,
        to: width(starts.first().copied().unwrap_or(end))?,
        opening,
        strings,
    });
    for (run, &from) in starts.iter().enumerate() {
        let Some(operand) = evaluated(body, &parse.template[from]) else {
            continue;
        };
        let mark = registers.mark();
        let dst = registers.alloc()?;
        push_native(
            ops,
            false,
            consts,
            registers,
            hints,
            calls,
            plan,
            operand,
            index,
            // Slot zero names the PARSE VALUE source. A trigger's slot
            // follows its template index, including any comma fences.
            u32::from(width(from + 1)?),
            Some(NodePath::ROOT),
            dst,
        )?;
        registers.release(mark);
        ops.push(Op::ParseRun {
            index,
            base,
            from: width(from)?,
            to: width(starts.get(run + 1).copied().unwrap_or(end))?,
            operand: dst,
        });
    }
    Ok(())
}

/// Whether `expr` compiles to native ops entire, with `eval.rs` not entered
/// for any part of it.
fn native_shape(expr: ExprRef<'_>, path: Option<NodePath>) -> bool {
    match expr.kind() {
        ExprView::Literal(_)
        | ExprView::Constant(_)
        | ExprView::Variable(_)
        | ExprView::Stem(_)
        | ExprView::Compound(_)
        | ExprView::DotVariable(_) => true,
        // A call needs an address, and a node past the width has none. The
        // whole slot then falls to `Op::EvalExpr`, which is the answer it had
        // before there was an address at all.
        ExprView::Call { .. } | ExprView::QualifiedCall { .. } => path.is_some(),
        // A send needs an address for the same reason; the receiver is child
        // 0 and the arguments follow it. A `:scope` override has no address
        // of its own, so a call in it leaves the slot general.
        ExprView::Message {
            target,
            super_class,
            args,
            ..
        } => {
            path.is_some()
                && native_shape(target, descend_nth(path, 0))
                && super_class.is_none_or(|scope| native_shape(scope, None))
                && native_arguments(args, path, 1)
        }
        ExprView::List(items) => native_arguments(items, path, 0),
        // A reference names its variable through its address.
        ExprView::VariableReference { .. } | ExprView::ClassResolver { .. } => path.is_some(),
        ExprView::Binary { op, left, right } => {
            is_native_binary(op)
                && native_shape(left, descend(path, false))
                && native_shape(right, descend(path, true))
        }
        // No operator condition beside the operand's, unlike the arm above:
        // `Interp::apply_prefix` matches `PrefixOp` exhaustively, so a variant
        // added to that enum is a compile error there rather than an operator
        // silently promoted here.
        ExprView::Prefix { operand, .. } => native_shape(operand, descend(path, false)),
        _ => false,
    }
}

/// Whether a message instruction's term and assigned `value` compile to
/// native ops. A cascade answers its receiver, which only the assignment
/// form, whose answer is never the term's, can ignore.
fn native_message_instruction(term: ExprRef<'_>, value: Option<ExprRef<'_>>) -> bool {
    let ExprView::Message {
        target,
        super_class,
        args,
        cascade,
        ..
    } = term.kind()
    else {
        return false;
    };
    (value.is_some() || !cascade)
        && native_shape(target, NodePath::ROOT.nth(0))
        && super_class.is_none_or(|scope| native_shape(scope, None))
        && native_arguments(args, Some(NodePath::ROOT), 1)
        && value.is_none_or(|value| native_shape(value, Some(NodePath::ROOT)))
}

/// The address of one child of the node addressed by `path`: the right child
/// when `right`, and the left or only one otherwise.
fn descend(path: Option<NodePath>, right: bool) -> Option<NodePath> {
    path?.child(right)
}

/// The address of child `index` of the node with a list of children that
/// `path` addresses.
fn descend_nth(path: Option<NodePath>, index: usize) -> Option<NodePath> {
    path?.nth(index)
}

/// Whether every one of `args` is omitted or compiles to native ops, each at
/// child `first + k` of the node `path` addresses, and their count fits an op.
fn native_arguments(args: Args<'_>, path: Option<NodePath>, first: usize) -> bool {
    u16::try_from(args.len()).is_ok()
        && args
            .iter()
            .enumerate()
            .all(|(k, arg)| arg.is_none_or(|expr| native_shape(expr, descend_nth(path, first + k))))
}

/// The ops that push each of `args` onto the driver's argument stack, the
/// `k`th addressed by `address(k)`, and the `>A>` line each owes: an omitted
/// position's too where `trace_omitted`. Answers how many were pushed.
#[expect(
    clippy::too_many_arguments,
    reason = "the emission sinks, the plan a read resolves its slot against, and the address               each argument's own call ops name"
)]
fn push_arguments<'a>(
    ops: &mut Vec<Op>,
    echoes_values: bool,
    consts: &mut Constants<'a>,
    registers: &mut Registers,
    hints: &mut Hints,
    calls: &mut Calls,
    plan: &Plan,
    args: Args<'a>,
    index: u32,
    address: impl Fn(usize) -> (u32, Option<NodePath>),
    trace_omitted: bool,
) -> Result<u16, ChunkTooLarge> {
    let argc = u16::try_from(args.len()).map_err(|_| ChunkTooLarge {
        what: "call arguments past u16",
    })?;
    for (k, arg) in args.iter().enumerate() {
        let src = match arg {
            None => Op::ARG_OMITTED,
            Some(expr) => {
                let (slot, path) = address(k);
                let src = registers.alloc()?;
                push_native(
                    ops,
                    echoes_values,
                    consts,
                    registers,
                    hints,
                    calls,
                    plan,
                    expr,
                    index,
                    slot,
                    path,
                    src,
                )?;
                src
            }
        };
        ops.push(Op::PushArg { src });
        // Behind the argument's own ops, so its `>L>`/`>V>` lines print
        // first -- the order `invoke_call`'s own loop produces.
        if echoes_values && (trace_omitted || src != Op::ARG_OMITTED) {
            ops.push(Op::TraceArgument { src });
        }
    }
    Ok(argc)
}

/// The ops of a send's `:scope` override, if it has one, into a register of
/// their own, and its check against the receiver in `recv`; answers that
/// register, or [`Op::NO_SCOPE`]. The override has no address, so it holds no
/// call ([`native_shape`]).
#[expect(
    clippy::too_many_arguments,
    reason = "the emission sinks, the plan a read resolves its slot against, and the receiver"
)]
fn push_scope<'a>(
    ops: &mut Vec<Op>,
    echoes_values: bool,
    consts: &mut Constants<'a>,
    registers: &mut Registers,
    hints: &mut Hints,
    calls: &mut Calls,
    plan: &Plan,
    scope: Option<ExprRef<'a>>,
    index: u32,
    recv: u16,
) -> Result<u16, ChunkTooLarge> {
    let Some(scope) = scope else {
        return Ok(Op::NO_SCOPE);
    };
    let reg = registers.alloc()?;
    push_native(
        ops,
        echoes_values,
        consts,
        registers,
        hints,
        calls,
        plan,
        scope,
        index,
        0,
        None,
        reg,
    )?;
    ops.push(Op::SendScope { recv, scope: reg });
    Ok(reg)
}

/// The ops that leave `expr` -- which [`native_shape`] has already accepted --
/// in register `dst`.
#[expect(
    clippy::too_many_arguments,
    reason = "the emission sinks, the plan a read resolves its slot against, and the address a \
              call op has to name"
)]
fn push_native<'a>(
    ops: &mut Vec<Op>,
    echoes_values: bool,
    consts: &mut Constants<'a>,
    registers: &mut Registers,
    hints: &mut Hints,
    calls: &mut Calls,
    plan: &Plan,
    expr: ExprRef<'a>,
    index: u32,
    slot: u32,
    path: Option<NodePath>,
    dst: u16,
) -> Result<(), ChunkTooLarge> {
    match expr.kind() {
        ExprView::Literal(bytes) => {
            ops.push(Op::Const {
                dst,
                konst: consts.intern(bytes)?,
            });
            // Behind the load rather than in front of it, because `eval.rs`
            // emits this line post-order, with the value in hand.
            if echoes_values {
                ops.push(Op::TraceLiteral { src: dst });
            }
        }
        // A constant symbol's value is its own upcased spelling, which lives
        // in the symbol table rather than in the node -- so the symbol travels
        // in the op and the spelling is read at run time, where the table is
        // in hand. The echo behind it is the literal's, because the line is the
        // same `>L>`.
        ExprView::Constant(id) => {
            ops.push(Op::LoadConstant { symbol: id, dst });
            if echoes_values {
                ops.push(Op::TraceLiteral { src: dst });
            }
        }
        // The three bare-symbol reads. **`ExprView::DotVariable` and
        // `ExprView::VariableReference` are not among them and are not
        // reads**: they trace `>E>` and `>O>` rather than `>V>`.
        ExprView::Variable(id) => push_read(ops, echoes_values, plan, SymbolRead::Simple, id, dst),
        ExprView::Stem(id) => push_read(ops, echoes_values, plan, SymbolRead::Stem, id, dst),
        ExprView::Compound(id) => {
            push_read(ops, echoes_values, plan, SymbolRead::Compound, id, dst)
        }
        ExprView::DotVariable(id) => {
            ops.push(Op::Load {
                symbol: id,
                read: SymbolRead::Environment,
                at: calls.reserve_environment(),
                dst,
            });
            if echoes_values {
                ops.push(Op::TraceRead {
                    symbol: id,
                    read: SymbolRead::Environment,
                    src: dst,
                });
            }
        }
        ExprView::Binary { op, left, right } => {
            // **The left operand lands in `dst` itself and only the right one
            // takes a register of its own**, which is what keeps a chain's
            // register cost at two however long it runs: `za + zb + zc + zd`
            // is left-nested, so each operator writes its result back into the
            // register the next one reads as its left. Both sources are read
            // before the destination is written, which is the whole of what
            // makes the aliasing safe.
            push_native(
                ops,
                echoes_values,
                consts,
                registers,
                hints,
                calls,
                plan,
                left,
                index,
                slot,
                descend(path, false),
                dst,
            )?;
            let mark = registers.mark();
            let rhs = registers.alloc()?;
            push_native(
                ops,
                echoes_values,
                consts,
                registers,
                hints,
                calls,
                plan,
                right,
                index,
                slot,
                descend(path, true),
                rhs,
            )?;
            // **Only arithmetic takes a hint slot**, because only arithmetic
            // has a second path for one to choose. `Chunk::hints` is dense over
            // the ops that can specialise, so a slot reserved for an operator
            // that never reads one would shift every later arithmetic site's
            // index by one and hand it somebody else's decision.
            ops.push(if is_arithmetic(op) {
                Op::Arith {
                    op,
                    hint: hints.reserve()?,
                    lhs: dst,
                    rhs,
                    dst,
                }
            } else {
                Op::Binary {
                    op,
                    lhs: dst,
                    rhs,
                    dst,
                    text: crate::eval::compares_as_text(op, left, right),
                    // The left operand compiled into `dst` ends in its own
                    // concatenation op, which is what `onto` claims.
                    onto: is_concatenation(op)
                        && matches!(
                            left.kind(),
                            ExprView::Binary { op: inner, .. } if is_concatenation(inner)
                        ),
                }
            });
            // Behind the operation rather than in front of it, because
            // `eval.rs` emits this line post-order, with the value in hand --
            // so an inner operator's line precedes the outer one's.
            if echoes_values {
                ops.push(Op::TraceOperator { op, src: dst });
            }
            // Held until the operation has run: the right operand's register
            // is live right up to it, and a release any earlier would hand it
            // out to the next operand of an enclosing operator.
            registers.release(mark);
        }
        ExprView::Prefix { op, operand } => {
            // **The operand lands in `dst` itself and no register is taken**,
            // which is the one-operand case of the discipline the binary arm
            // above states: the operand is read before the destination is
            // written, so a chain of prefixes costs the registers its innermost
            // term does and no more.
            push_native(
                ops,
                echoes_values,
                consts,
                registers,
                hints,
                calls,
                plan,
                operand,
                index,
                slot,
                descend(path, false),
                dst,
            )?;
            ops.push(Op::Prefix { op, src: dst, dst });
            // Behind the operation rather than in front of it, because
            // `eval.rs` emits this line post-order, with the value in hand --
            // so an inner operator's line precedes the outer one's.
            if echoes_values {
                ops.push(Op::TracePrefix { op, src: dst });
            }
        }
        // The call's address leads back to the target and to any arguments
        // that need the expression evaluator.
        ExprView::Call { args, .. } | ExprView::QualifiedCall { args, .. } => {
            let slot16 = u16::try_from(slot).map_err(|_| ChunkTooLarge {
                what: "expression slots past u16",
            })?;
            let path = path.expect("native_shape accepts a call only where an address reaches it");
            let qualified = matches!(expr.kind(), ExprView::QualifiedCall { .. });
            let reserve_site = |calls: &mut Calls| {
                if qualified {
                    Ok(super::UNCACHED_CALL)
                } else {
                    calls.reserve()
                }
            };
            // **The arguments become ops of their own where they can.** Then
            // the call op takes values off the driver's argument stack
            // instead of walking back to the node for expressions and
            // evaluating each through `eval`. What that saves is the descent,
            // the name lookup and one `eval` entry per argument; measured on
            // a ladder of `zq = length(s)` clauses, a builtin call cost 617
            // instructions against the -O3 interpreter's 293.
            if native_arguments(args, Some(path), 0) {
                let mark = registers.mark();
                let argc = push_arguments(
                    ops,
                    echoes_values,
                    consts,
                    registers,
                    hints,
                    calls,
                    plan,
                    args,
                    index,
                    |k| (slot, path.nth(k)),
                    true,
                )?;
                ops.push(Op::CallArgs {
                    slot: slot16,
                    path,
                    site: reserve_site(calls)?,
                    argc,
                    dst,
                });
                registers.release(mark);
                if echoes_values {
                    ops.push(Op::TraceFunction {
                        index,
                        slot: slot16,
                        path,
                        src: dst,
                    });
                }
                return Ok(());
            }
            let slot = slot16;
            ops.push(Op::CallExpr {
                index,
                slot,
                path,
                site: reserve_site(calls)?,
                dst,
            });
            // Behind the call rather than in front of it, because `eval.rs`
            // emits this line post-order, with the value in hand.
            if echoes_values {
                ops.push(Op::TraceFunction {
                    index,
                    slot,
                    path,
                    src: dst,
                });
            }
        }
        // The receiver lands in `dst` itself, which the send reads before it
        // writes its answer there.
        ExprView::Message {
            target,
            name,
            super_class,
            args,
            cascade,
        } => {
            let path = path.expect("native_shape accepts a send only where an address reaches it");
            push_native(
                ops,
                echoes_values,
                consts,
                registers,
                hints,
                calls,
                plan,
                target,
                index,
                slot,
                path.nth(0),
                dst,
            )?;
            let mark = registers.mark();
            let scope = push_scope(
                ops,
                echoes_values,
                consts,
                registers,
                hints,
                calls,
                plan,
                super_class,
                index,
                dst,
            )?;
            let argc = push_arguments(
                ops,
                echoes_values,
                consts,
                registers,
                hints,
                calls,
                plan,
                args,
                index,
                |k| (slot, path.nth(k + 1)),
                true,
            )?;
            let site = calls.reserve_send(Box::from(&name[..]))?;
            ops.push(if scope == Op::NO_SCOPE && !cascade {
                Op::Send {
                    site,
                    recv: dst,
                    argc,
                    dst,
                    form: SendForm::Value,
                }
            } else {
                Op::SendWith {
                    site,
                    recv: dst,
                    argc,
                    dst,
                    scope,
                    // `~~` answers the receiver.
                    form: if cascade {
                        SendForm::Cascade
                    } else {
                        SendForm::Value
                    },
                }
            });
            registers.release(mark);
        }
        // `>name`/`<name`, which traces as an operator (`>O>`) through the
        // same node-addressed echo a call's `>F>` uses.
        ExprView::VariableReference { .. } => {
            let slot = u16::try_from(slot).map_err(|_| ChunkTooLarge {
                what: "expression slots past u16",
            })?;
            let path =
                path.expect("native_shape accepts a reference only where an address reaches it");
            ops.push(Op::Reference { slot, path, dst });
            if echoes_values {
                ops.push(Op::TraceFunction {
                    index,
                    slot,
                    path,
                    src: dst,
                });
            }
        }
        // `namespace:Name`, whose `>N>` line is the node-addressed echo too.
        ExprView::ClassResolver { .. } => {
            let slot = u16::try_from(slot).map_err(|_| ChunkTooLarge {
                what: "expression slots past u16",
            })?;
            let path = path
                .expect("native_shape accepts a class resolver only where an address reaches it");
            ops.push(Op::ClassAt { slot, path, dst });
            if echoes_values {
                ops.push(Op::TraceFunction {
                    index,
                    slot,
                    path,
                    src: dst,
                });
            }
        }
        // An omitted item owes no `>A>` line, where an omitted argument owes
        // an empty one (`eval_list`).
        ExprView::List(items) => {
            let mark = registers.mark();
            let argc = push_arguments(
                ops,
                echoes_values,
                consts,
                registers,
                hints,
                calls,
                plan,
                items,
                index,
                |k| (slot, descend_nth(path, k)),
                false,
            )?;
            ops.push(Op::List { argc, dst });
            registers.release(mark);
        }
        _ => unreachable!("this descends only into an expression native_shape accepted"),
    }
    Ok(())
}

/// Whether the write of `target = value` can use the tail key a read of the
/// same compound in `value` resolves, which is so where `value` calls
/// nothing: no function, no message, no environment symbol. Its operators
/// can still run code, an operator being a message to an object, and so can
/// a tail's string request and a NOVALUE hook, but each of those pushes an
/// activation or enters a native call, and either drops the key the read
/// holds, as a switch to another activity does. Where the answer is yes,
/// the reads of the compound among `ops`, the ops compiled for `value`, are
/// marked [`SymbolRead::CompoundShared`].
pub(crate) fn share_tail_key(ops: &mut [Op], target: ExprRef<'_>, value: ExprRef<'_>) -> bool {
    let ExprView::Compound(id) = target.kind() else {
        return false;
    };
    // The write takes the key of the last read before it, so every read of
    // the compound has to run before the write does, unconditionally.
    if !calls_nothing(value)
        || ops.iter().any(|op| {
            matches!(
                op,
                Op::Jump { .. } | Op::JumpUnless { .. } | Op::ConditionJump { .. }
            )
        })
    {
        return false;
    }
    // The echo behind a read repeats its kind, so it is marked with it.
    let mut shared = false;
    for op in ops {
        if let Op::Load { symbol, read, .. } | Op::TraceRead { symbol, read, .. } = op
            && *symbol == id
            && *read == SymbolRead::Compound
        {
            *read = SymbolRead::CompoundShared;
            shared = true;
        }
    }
    shared
}

/// Whether `expr` is literals, symbols and operators only. Walked with a
/// worklist rather than by recursion, so that an operator chain as deep as
/// the parser accepts costs heap rather than stack.
fn calls_nothing(expr: ExprRef<'_>) -> bool {
    let mut pending = vec![expr];
    while let Some(expr) = pending.pop() {
        match expr.kind() {
            ExprView::Literal(_)
            | ExprView::Constant(_)
            | ExprView::Variable(_)
            | ExprView::Stem(_)
            | ExprView::Compound(_) => {}
            ExprView::Prefix { operand, .. } => pending.push(operand),
            ExprView::Binary { left, right, .. } => {
                pending.push(right);
                pending.push(left);
            }
            _ => return false,
        }
    }
    true
}

/// The slot one assignment *target* resolves to, or [`PlanSlot::UNRESOLVED`]
/// for a target that does not write a slot by name.
fn write_slot(plan: &Plan, target: ExprRef<'_>) -> PlanSlot {
    match target.kind() {
        ExprView::Variable(id) => plan
            .slot_for_symbol(id)
            .map_or(PlanSlot::UNRESOLVED, PlanSlot::of),
        _ => PlanSlot::UNRESOLVED,
    }
}

/// The two ops one bare-symbol read is: the load, and the `>V>`/`>C>` line
/// that reading it owes.
fn push_read(
    ops: &mut Vec<Op>,
    echoes_values: bool,
    plan: &Plan,
    read: SymbolRead,
    symbol: SymbolId,
    dst: u16,
) {
    let at = match read {
        SymbolRead::Simple | SymbolRead::Stem => plan
            .slot_for_symbol(symbol)
            .map_or(PlanSlot::UNRESOLVED, PlanSlot::of),
        SymbolRead::Compound | SymbolRead::CompoundShared => PlanSlot::UNRESOLVED,
        SymbolRead::Environment => unreachable!("a `.NAME` read is compiled with its memo site"),
    };
    ops.push(Op::Load {
        symbol,
        read,
        at,
        dst,
    });
    // Behind the load rather than in front of it, because `eval.rs` emits
    // these lines post-order, with the value in hand.
    if echoes_values {
        ops.push(Op::TraceRead {
            symbol,
            read,
            src: dst,
        });
    }
}

/// Whether a promoted clause of `instruction` echoes under `trace`.
fn echoes(trace: ChunkTrace, instruction: &Instruction) -> bool {
    trace.echoes(
        matches!(instruction.kind, InstructionKind::Label { .. }),
        matches!(instruction.kind, InstructionKind::Command { .. }),
    )
}

/// Pushes the clause echo op, if `echo`, as the **first** op of the region
/// that follows -- the position clause unit echoes at,
/// before anything the clause computes.
fn push_echo(ops: &mut Vec<Op>, echo: bool, index: u32) {
    if echo {
        ops.push(Op::TraceClause { index });
    }
}

/// Emits the ops that go in front of one instruction, innermost first.
fn emit_before(
    ops: &mut Vec<Op>,
    patches: &mut Vec<Patch>,
    landing: &mut [Option<u32>],
    before: &mut [Before],
) -> Result<(), ChunkTooLarge> {
    before.reverse();
    // **A `SELECT` branch's close goes in front of everything else here**,
    // which is where the driver used to ask the question: it tested for an
    // ended frame before fetching the op at all, so a `WHEN` whose body is an
    // `IF` left the branch -- and jumped to the `SELECT`'s own end -- without
    // ever reaching that `IF`'s [`Op::EndBranch`]. Emitting the close second
    // runs that boundary first, which is a clause boundary that did not run
    // before. The sort is stable, so nested branches keep the innermost-first
    // order the reverse above gives them.
    before.sort_by_key(|entry| match entry {
        Before::SelectBranchEnd => 0,
        Before::ThenEnd { .. } | Before::EnterOtherwise(_) => 1,
        Before::BlockEnd => 2,
    });
    debug_assert!(
        before.iter().rev().skip(1).all(|entry| !matches!(
            entry,
            Before::ThenEnd {
                resume: Some(_),
                ..
            }
        )),
        "an op in front of an instruction carries a jump with more ops behind it, which \
         nothing would reach"
    );
    // **Where an `IF` whose true branch ends here lands when its condition is
    // false**: past its own boundary and those of the `IF`s nested in that
    // branch, which come before it here, and short of everything else. A
    // branch close in front of it is never skipped, because the `IF` is
    // inside that branch and its false path ends the branch as surely as its
    // true path does -- so then the false path lands where the true path
    // falls, on the close, which leaves the `SELECT` before any boundary
    // behind it runs. Past the close is the next `WHEN` or the `OTHERWISE`,
    // with the branch's frame still open.
    let start = op_index(ops)?;
    let closes = matches!(before.first(), Some(Before::SelectBranchEnd));
    for entry in before.iter() {
        match entry {
            Before::ThenEnd { resume, owner } => {
                ops.push(Op::EndBranch);
                if let Some(resume) = resume {
                    let op = op_index(ops)?;
                    ops.push(Op::Jump { target: 0 });
                    patches.push(Patch {
                        op,
                        target: *resume,
                        kind: PatchKind::Resume,
                    });
                }
                landing[*owner] = Some(if closes { start } else { op_index(ops)? });
            }
            Before::SelectBranchEnd | Before::BlockEnd => ops.push(Op::EndWhen),
            Before::EnterOtherwise(select) => {
                landing[*select] = Some(op_index(ops)?);
                ops.push(Op::EnterOtherwise {
                    select: instruction_index(*select)?,
                });
            }
        }
    }
    Ok(())
}

/// Records that `mark` is dead by the time the instruction this slot belongs to
/// is reached, keeping **the lowest** mark any construct wants released there.
fn release_to(slot: &mut Option<Mark>, mark: Mark) {
    let lowest = match *slot {
        Some(existing) if existing.0 <= mark.0 => existing,
        _ => mark,
    };
    *slot = Some(lowest);
}

/// Rewrites the [`Op::Clause`] at `at` so that its `end` is the position one
/// past everything pushed since -- the region's own end -- and so that it is
/// an [`Op::CallingClause`] where the region holds an op a callee resumes.
fn close_region(ops: &mut [Op], at: u32) -> Result<(), ChunkTooLarge> {
    let end = op_index(ops)?;
    let (head, region) = ops[usize::cast_from(at)..].split_at_mut(1);
    let calling = region.iter().any(|op| {
        matches!(
            op,
            Op::CallExpr { .. } | Op::CallArgs { .. } | Op::Send { .. } | Op::SendWith { .. }
        )
    });
    match &mut head[0] {
        Op::Clause { index, .. } if calling => {
            head[0] = Op::CallingClause { index: *index, end };
        }
        Op::Clause { end: slot, .. } => *slot = end,
        _ => unreachable!("close_region is given the index of the Clause op it closes"),
    }
    Ok(())
}

/// Every clause's [`ClauseOpening`]: its register top from `register_tops`,
/// and its position where the plan can supply one for every clause.
fn clause_openings(body: &CodeBody, plan: &Plan, register_tops: &[u16]) -> Box<[ClauseOpening]> {
    let positions = clause_positions(body, plan);
    register_tops
        .iter()
        .enumerate()
        .map(|(index, &register_top)| match &positions {
            Some(positions) => ClauseOpening {
                position: positions[index],
                tabled: true,
                register_top,
            },
            None => ClauseOpening {
                position: ClausePosition { line: 0, indent: 0 },
                tabled: false,
                register_top,
            },
        })
        .collect()
}

/// Every clause's [`ClausePosition`], or `None` where the plan cannot supply
/// one for each.
fn clause_positions(body: &CodeBody, plan: &Plan) -> Option<Vec<ClausePosition>> {
    let len = body.instructions.len();
    if plan.lines.len() != len || plan.indents.len() != len {
        return None;
    }
    let mut positions = Vec::with_capacity(len);
    for (line, indent) in plan.lines.iter().zip(plan.indents.iter()) {
        let (Ok(line), Ok(indent)) = (u32::try_from(*line), u32::try_from(*indent)) else {
            return None;
        };
        positions.push(ClausePosition { line, indent });
    }
    Some(positions)
}

/// The index the next op will be pushed at, refused rather than wrapped.
fn op_index(ops: &[Op]) -> Result<u32, ChunkTooLarge> {
    u32::try_from(ops.len()).map_err(|_| ChunkTooLarge { what: "op stream" })
}

/// An instruction index as an op payload, refused rather than wrapped.
fn instruction_index(index: usize) -> Result<u32, ChunkTooLarge> {
    u32::try_from(index).map_err(|_| ChunkTooLarge { what: "op stream" })
}

/// **What wiring the analysis into emission may and may not do**, asserted
/// while it is not yet wired in, so that the commit that wires it in cannot
/// be the one that discovers either.
///
/// * A body nothing in which can change the setting, entered outside
///   interactive debug, has one answer at every clause, and it is the setting
///   this chunk is keyed under -- so no such body loses the compile-time
///   decision it already has. Under debug a line typed at a pause can change
///   the setting, and `trace_flow::analyse` answers `Unknown`.
/// * No clause gains an echo the body-wide bool does not already carry, in
///   either of the two questions those bools answer. A body entered in debug
///   is answered `Unknown` throughout and asks nothing of either.
#[cfg(debug_assertions)]
fn assert_analysis_only_narrows(
    plan: &Plan,
    trace: ChunkTrace,
    settings: &[super::trace_flow::Setting],
) {
    // The two body-wide decisions this analysis replaced, restated here
    // because this is now the only thing that reads them.
    let never_retraces = plan
        .trace_events()
        .iter()
        .all(|event| *event == crate::trace::TraceEvent::Keeps);
    let echoes_values = trace.intermediates() || !never_retraces;
    let echoes_keyword = trace.results() || !never_retraces;
    debug_assert!(
        !never_retraces
            || trace.debugging()
            || settings
                .iter()
                .all(|setting| *setting == super::trace_flow::Setting::Known(trace)),
        "nothing in this body can change the TRACE setting and it is not entered in debug, so \
         every clause runs under the one the chunk is keyed to: {settings:?}"
    );
    debug_assert!(
        trace.debugging()
            || settings
                .iter()
                .all(|setting| (!setting.echoes_values() || echoes_values)
                    && (!setting.echoes_keyword() || echoes_keyword)),
        "a clause's own answer asks for an echo this chunk does not emit today: {settings:?}"
    );
}

// Test-only instrumentation: how many times `compile` has actually run. The
// chunk-cache test (`golden_tests.rs`) needs this to tell "the chunk cache
// compiled the body once" apart from "the second lookup happened not to
// fail" -- an implementation that recompiles on every call and returns an
// equal-looking `Chunk` would still pass a check that only inspects the
// result, and a `thread_local` counter is what lets the test inspect the
// call count instead.
#[cfg(test)]
thread_local! {
    static COMPILE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn count_compile_call() {
    COMPILE_CALLS.with(|calls| calls.set(calls.get() + 1));
}

#[cfg(test)]
pub(crate) fn compile_calls() -> usize {
    COMPILE_CALLS.with(|calls| calls.get())
}

#[cfg(test)]
mod tests;
