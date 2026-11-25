//! **The corpus-wide net over what `compile` emitted, as an invariant rather
//! than a transcript.**

use interrexx_cast::CastFrom;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use interrexx_parse::{
    Args, Call, CodeBody, DirectiveKind, ExprRef, ExprView, InstructionKind, Operator,
    parse_program,
};

use super::golden::render;
use super::{Chunk, ConditionKeyword, Op};
use crate::plan::{BodyKind, Plan};
use crate::trace::{ChunkTrace, TraceMode};

/// The op a promoted clause's value expression must end in.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Root {
    Const,
    LoadConstant,
    Load,
    Arith,
    Binary,
    Prefix,
    CallExpr,
    Send,
    List,
    Reference,
    ClassAt,
    EvalExpr,
}

impl Root {
    /// The `Root` an emitted op is, or `None` for an op that is not one an
    /// expression's own value can end in.
    fn of(op: &Op) -> Option<Root> {
        match op {
            Op::Const { .. } => Some(Root::Const),
            Op::LoadConstant { .. } => Some(Root::LoadConstant),
            Op::Load { .. } => Some(Root::Load),
            Op::Arith { .. } => Some(Root::Arith),
            Op::Binary { .. } => Some(Root::Binary),
            Op::Prefix { .. } => Some(Root::Prefix),
            Op::EvalExpr { .. } => Some(Root::EvalExpr),
            Op::CallExpr { .. } | Op::CallArgs { .. } => Some(Root::CallExpr),
            Op::Send { .. } | Op::SendWith { .. } => Some(Root::Send),
            Op::List { .. } => Some(Root::List),
            Op::Reference { .. } => Some(Root::Reference),
            Op::ClassAt { .. } => Some(Root::ClassAt),
            Op::CallNamed { .. }
            | Op::PushArg { .. }
            | Op::TraceArgument { .. }
            | Op::TraceKeyword { .. }
            | Op::LoopHeaderValue { .. }
            | Op::LoopRun { .. }
            | Op::LoopNext { .. }
            | Op::LoopTest { .. }
            | Op::LoopTested { .. }
            | Op::SelectCase { .. }
            | Op::ExecValue { .. }
            | Op::ExecWith { .. }
            | Op::ReplyCheck { .. }
            | Op::RaisePart { .. }
            | Op::ForwardPart { .. }
            | Op::SendScope { .. }
            | Op::Signal { .. }
            | Op::UseArg(..)
            | Op::ParseBegin { .. }
            | Op::ParseStore { .. }
            | Op::ParseRun { .. }
            | Op::Clause { .. }
            | Op::CallingClause { .. }
            | Op::TraceClause { .. }
            | Op::WhenTest { .. }
            | Op::TraceLiteral { .. }
            | Op::TraceRead { .. }
            | Op::TraceOperator { .. }
            | Op::TracePrefix { .. }
            | Op::Store { .. }
            | Op::Say { .. }
            | Op::Return { .. }
            | Op::Queue { .. }
            | Op::Call { .. }
            | Op::Message { .. }
            | Op::Expose { .. }
            | Op::Exec { .. }
            | Op::Escape { .. }
            | Op::TraceFunction { .. }
            | Op::EndBranch
            | Op::EndWhen
            | Op::EnterWhen { .. }
            | Op::EnterOtherwise { .. }
            | Op::Jump { .. }
            | Op::JumpUnless { .. }
            | Op::ConditionJump { .. } => None,
        }
    }
}

/// Which construct of the minimum promotion set an instruction is, or `None`
/// for one outside it.
fn promoted_as(kind: &InstructionKind, index: usize, listed: &[usize]) -> Option<&'static str> {
    match kind {
        InstructionKind::Do(_) => Some("DO"),
        InstructionKind::Loop(_) => Some("LOOP"),
        InstructionKind::If { .. } => Some("IF"),
        InstructionKind::Select { .. } => Some("SELECT"),
        InstructionKind::When { .. } if listed.contains(&index) => Some("WHEN"),
        InstructionKind::WhenCase { .. } if listed.contains(&index) => Some("WHEN CASE"),
        InstructionKind::Assignment { .. } => Some("assignment"),
        InstructionKind::Say { .. } => Some("SAY"),
        InstructionKind::Return { .. } => Some("RETURN"),
        InstructionKind::Exit { .. } => Some("EXIT"),
        InstructionKind::Push { .. } => Some("PUSH"),
        InstructionKind::Queue { .. } => Some("QUEUE"),
        InstructionKind::Call(call) if matches!(&**call, Call::Named { .. }) => Some("CALL name"),
        InstructionKind::Message { .. } => Some("message send"),
        InstructionKind::Leave { .. } => Some("LEAVE"),
        InstructionKind::Iterate { .. } => Some("ITERATE"),
        InstructionKind::Nop => Some("NOP"),
        InstructionKind::Then => Some("THEN"),
        InstructionKind::Label { .. } => Some("label"),
        InstructionKind::Signal(_) => Some("SIGNAL"),
        InstructionKind::Parse(_) => Some("PARSE"),
        InstructionKind::Arg(_) => Some("ARG"),
        InstructionKind::Pull(_) => Some("PULL"),
        InstructionKind::Numeric {
            expression: Some(_),
            ..
        } => Some("NUMERIC"),
        InstructionKind::Trace(interrexx_parse::Trace::Value(_)) => Some("TRACE VALUE"),
        InstructionKind::Address(address)
            if address.command.is_none()
                && address.environment.is_none()
                && address.dynamic.is_some() =>
        {
            Some("ADDRESS VALUE")
        }
        InstructionKind::Options { .. } => Some("OPTIONS"),
        InstructionKind::Interpret { .. } => Some("INTERPRET"),
        InstructionKind::Command {
            expression: Some(_),
        } => Some("command"),
        InstructionKind::Guard(guard) if guard.condition.is_some() => Some("GUARD WHEN"),
        InstructionKind::Reply { .. } => Some("REPLY"),
        InstructionKind::Raise(_) => Some("RAISE"),
        _ => None,
    }
}

/// The op the expression [`check_body`] hands this must end in.
fn root_of(expr: ExprRef<'_>) -> Root {
    match expr.kind() {
        // A call at the root is the whole slot's expression, so the route down
        // to it has no steps and every address reaches it. A call *below* the
        // root is an operand and never the last op, so it is [`native`]'s
        // business rather than this function's.
        ExprView::Call { .. } | ExprView::QualifiedCall { .. } => Root::CallExpr,
        ExprView::Literal(_) => Root::Const,
        ExprView::Constant(_) => Root::LoadConstant,
        ExprView::Variable(_)
        | ExprView::Stem(_)
        | ExprView::Compound(_)
        | ExprView::DotVariable(_) => Root::Load,
        ExprView::Message { .. } if native(expr, Some(0)) => Root::Send,
        ExprView::List(_) if native(expr, Some(0)) => Root::List,
        ExprView::VariableReference { .. } => Root::Reference,
        ExprView::ClassResolver { .. } => Root::ClassAt,
        ExprView::Binary { op, left, right }
            if native(left, step(Some(0), 1)) && native(right, step(Some(0), 1)) =>
        {
            if arithmetic(op) {
                Root::Arith
            } else if other_family(op) {
                Root::Binary
            } else {
                Root::EvalExpr
            }
        }
        // No operator condition beside the operand's, unlike the arm above.
        // The expectation this file states is that a prefix promotes whatever
        // its operator, and one that did not would leave an `Op::EvalExpr`
        // where this arm calls for `Root::Prefix` -- which reddens rather than
        // passes, so the arm is the claim and not an assumption behind it.
        ExprView::Prefix { operand, .. } if native(operand, step(Some(0), 1)) => Root::Prefix,
        _ => Root::EvalExpr,
    }
}

/// How many steps a route can take below its slot's root: one per bit a
/// `u32` holds below the sentinel bit that marks where the route starts.
const WIDEST_ROUTE: usize = 31;

/// The steps a route has taken once it takes `width` more, or `None` past
/// [`WIDEST_ROUTE`] or once it had none: a node there has no address.
fn step(taken: Option<usize>, width: usize) -> Option<usize> {
    taken
        .map(|taken| taken + width)
        .filter(|taken| *taken <= WIDEST_ROUTE)
}

/// Whether every part of `expr` has a native op, which is what licenses the
/// whole tree compiling without `eval.rs` being entered. `taken` is how many
/// route steps stand between `expr` and its slot's own root: one below an
/// operator, `k + 1` for the `k`th child of a list of them, and `None` once
/// no address reaches it.
fn native(expr: ExprRef<'_>, taken: Option<usize>) -> bool {
    match expr.kind() {
        ExprView::Literal(_)
        | ExprView::Constant(_)
        | ExprView::Variable(_)
        | ExprView::Stem(_)
        | ExprView::Compound(_)
        | ExprView::DotVariable(_) => true,
        ExprView::Call { .. }
        | ExprView::QualifiedCall { .. }
        | ExprView::VariableReference { .. }
        | ExprView::ClassResolver { .. } => taken.is_some(),
        // A `:scope` override has no route of its own.
        ExprView::Message {
            target,
            super_class,
            args,
            ..
        } => {
            taken.is_some()
                && native(target, step(taken, 1))
                && super_class.is_none_or(|scope| native(scope, None))
                && arguments(args, taken, 1)
        }
        ExprView::List(items) => arguments(items, taken, 0),
        ExprView::Binary { op, left, right } => {
            (arithmetic(op) || other_family(op))
                && native(left, step(taken, 1))
                && native(right, step(taken, 1))
        }
        ExprView::Prefix { operand, .. } => native(operand, step(taken, 1)),
        _ => false,
    }
}

/// Whether every written one of `args` is [`native`], the `k`th being child
/// `first + k` of a node `taken` steps down.
fn arguments(args: Args<'_>, taken: Option<usize>, first: usize) -> bool {
    args.iter()
        .enumerate()
        .all(|(k, arg)| arg.is_none_or(|expr| native(expr, step(taken, first + k + 1))))
}

/// The operators `Interp::eval_arithmetic` computes, as this file's own
/// statement of the set.
fn arithmetic(op: Operator) -> bool {
    matches!(
        op,
        Operator::Plus
            | Operator::Subtract
            | Operator::Multiply
            | Operator::Divide
            | Operator::IntDiv
            | Operator::Remainder
            | Operator::Power
    )
}

/// The operators `Interp::apply_binary` computes -- concatenation, comparison
/// and logical -- as this file's own statement of that set, spelled out for
/// the reason [`arithmetic`] is.
fn other_family(op: Operator) -> bool {
    matches!(
        op,
        Operator::Concatenate
            | Operator::Abuttal
            | Operator::Blank
            | Operator::Equal
            | Operator::BackslashEqual
            | Operator::GreaterThan
            | Operator::BackslashGreaterThan
            | Operator::LessThan
            | Operator::BackslashLessThan
            | Operator::GreaterThanEqual
            | Operator::LessThanEqual
            | Operator::StrictEqual
            | Operator::StrictBackslashEqual
            | Operator::StrictGreaterThan
            | Operator::StrictBackslashGreaterThan
            | Operator::StrictLessThan
            | Operator::StrictBackslashLessThan
            | Operator::StrictGreaterThanEqual
            | Operator::StrictLessThanEqual
            | Operator::LessThanGreaterThan
            | Operator::GreaterThanLessThan
            | Operator::And
            | Operator::Or
            | Operator::Xor
    )
}

/// Whether `expr` holds a function call, a message send or a comma list
/// anywhere, which the side table of loop conditions cannot run.
fn outside_side_table(expr: ExprRef<'_>) -> bool {
    match expr.kind() {
        ExprView::Call { .. }
        | ExprView::QualifiedCall { .. }
        | ExprView::Message { .. }
        | ExprView::Logical(_) => true,
        ExprView::Binary { left, right, .. } => {
            outside_side_table(left) || outside_side_table(right)
        }
        ExprView::Prefix { operand, .. } => outside_side_table(operand),
        ExprView::List(items) => items.iter().flatten().any(outside_side_table),
        _ => false,
    }
}

fn at_u32(index: usize) -> u32 {
    u32::try_from(index).expect("a corpus body is not four billion instructions")
}

/// The compiled region of each promoted clause, keyed by instruction index:
/// the ops strictly inside `Op::Clause`'s own `(here, end)`.
fn regions(chunk: &Chunk) -> BTreeMap<u32, &[Op]> {
    let mut out = BTreeMap::new();
    for (at, op) in chunk.ops.iter().enumerate() {
        if let Op::Clause { index, end } | Op::CallingClause { index, end } = op {
            let previous = out.insert(*index, &chunk.ops[at + 1..usize::cast_from(*end)]);
            assert!(
                previous.is_none(),
                "instruction {index} opened two Clause regions"
            );
        }
    }
    out
}

/// The instruction indices every `SELECT` in `body` collected as its own
/// `WHEN`s.
fn listed_whens(body: &CodeBody) -> Vec<usize> {
    let mut out = Vec::new();
    for instruction in &body.instructions {
        if let InstructionKind::Select { whens, .. } = &instruction.kind {
            out.extend(whens.iter().copied());
        }
    }
    out
}

/// Every construct and every expression root this sweep saw fire, so that a
/// row the corpus never reaches is a loud gap rather than a silently
/// satisfied assertion.
#[derive(Default)]
struct Seen {
    constructs: BTreeMap<&'static str, usize>,
    roots: BTreeMap<Root, usize>,
    /// How many `Op::ConditionJump` ops the sweep saw, keyed by the keyword each
    /// is tagged with. Keyed rather than counted in one number, because a
    /// corpus holding a native `IF` condition and no native `WHEN` one would
    /// leave the `WHEN` row below vacuous while the total still looked
    /// healthy.
    native_conditions: BTreeMap<&'static str, usize>,
    /// How many `DO`/`LOOP` header slots compiled to something other than one
    /// `Op::EvalExpr`.
    native_header_values: usize,
    /// How many `WHILE`/`UNTIL` conditions compiled to a test region in the
    /// stream, and how many to [`Chunk::conditions`].
    loop_tests: usize,
    side_conditions: usize,
    /// How many `GUARD ... WHEN` conditions compiled to [`Chunk::conditions`].
    guard_side_conditions: usize,
    /// How many `REPLY`s with an expression compiled to `Op::ExecWith`, and
    /// how many bare ones stayed `Op::Exec`.
    reply_values: usize,
    bare_replies: usize,
    /// How many `RAISE`s with an expression compiled to their parts and
    /// `Op::ExecWith`, how many with none stayed `Op::Exec`, and how many of
    /// each part the first kind compiled.
    raise_values: usize,
    bare_raises: usize,
    raise_parts: BTreeMap<&'static str, usize>,
}

/// Checks one body's compiled stream against what its instructions call for,
/// and records what fired.
fn check_body(
    body: &CodeBody,
    symbols: &interrexx_parse::SymbolTable,
    source: &interrexx_parse::ProgramSource,
    where_: &str,
    seen: &mut Seen,
) {
    let plan = Plan::build(body, symbols, Some(source), BodyKind::Plain);
    let chunk = match super::compile(body, &plan, ChunkTrace::of(TraceMode::NORMAL)) {
        Ok(chunk) => chunk,
        // The one error `compile` has is a machine width, and no corpus
        // program is four billion instructions long. Loud rather than
        // skipped, because a refusal here would make every assertion below
        // vacuous for that body.
        Err(error) => panic!("{where_}: compile refused this body: {error:?}"),
    };
    let regions = regions(&chunk);
    let listed = listed_whens(body);

    for (index, instruction) in body.instructions.iter().enumerate() {
        let Some(construct) = promoted_as(&instruction.kind, index, &listed) else {
            continue;
        };
        let at = u32::try_from(index).expect("a corpus body is not four billion instructions");
        *seen.constructs.entry(construct).or_default() += 1;

        // **The `Op::Exec` check that stood here is gone with the op.** It
        // asserted that a construct in the minimum promotion set had not
        // fallen back; nothing can fall back now, so the question is
        // unaskable. What it was really protecting is the line below: the
        // instruction opened a region of its own.
        let region = regions.get(&at).unwrap_or_else(|| {
            panic!(
                "{where_}: instruction {index} ({construct}) opened no Clause region\n{}",
                render(&chunk)
            )
        });

        // Whether this clause carries a compiled condition, and for which
        // keyword. Counted because the `If` and `When` rows below say nothing
        // at all on a corpus whose every condition is outside the native set:
        // `root_of` would call for `Root::EvalExpr`, the stream would hold
        // what a declining condition compiles to, and both rows would pass
        // without the promotion ever having fired.
        for op in region.iter() {
            if let Op::ConditionJump { keyword, .. } = op {
                *seen
                    .native_conditions
                    .entry(match keyword {
                        ConditionKeyword::If => "IF",
                        ConditionKeyword::When => "WHEN",
                        ConditionKeyword::ListItem | ConditionKeyword::ListLast => "comma list",
                    })
                    .or_default() += 1;
            }
        }

        // **A `DO`/`LOOP` header is a list of expressions rather than one**, so
        // its expectation is per slot. Each `Op::LoopHeaderValue` ends one
        // slot's group, and a group starts where the one before it ended, or at
        // the region's own start for the first. The last `Root` in a group is
        // what that slot's expression must end in: neither `Op::TraceKeyword`
        // nor `Op::LoopHeaderValue` is a `Root`, so the scan back inside a
        // group finds the expression's own op and nothing else.
        if let InstructionKind::Do(loop_) | InstructionKind::Loop(loop_) = &instruction.kind {
            let mut slot = 0u32;
            let mut group = 0;
            for (op_at, op) in region.iter().enumerate() {
                let Op::LoopHeaderValue { role, .. } = op else {
                    continue;
                };
                // A `COUNTER`'s slot has no expression and loads its reset
                // value as a constant.
                let expected = if *role == crate::run::HeaderRole::Counter {
                    Root::Const
                } else {
                    crate::run::loop_header_slot(body, loop_, slot).map_or(Root::EvalExpr, root_of)
                };
                let actual = region[group..op_at].iter().filter_map(Root::of).next_back();
                assert_eq!(
                    actual,
                    Some(expected),
                    "{where_}: instruction {index} ({construct}) has a header slot {slot} \
                     calling for {expected:?} and a compiled group ending in {actual:?}\n{}",
                    render(&chunk)
                );
                *seen.roots.entry(expected).or_default() += 1;
                if expected != Root::EvalExpr {
                    seen.native_header_values += 1;
                }
                group = op_at + 1;
                slot += 1;
            }
            // **The `WHILE`/`UNTIL` condition**: a native one that calls or
            // sends is a test region in the stream ending in its own
            // `LoopTested`, and one that does neither is in the side table.
            // Neither walks.
            if let Some(conditional) = &loop_.conditional {
                let expected = root_of(body.expr(conditional.condition));
                let test = chunk.loop_test_at(index);
                let side = chunk.condition_at(index);
                if expected == Root::EvalExpr {
                    assert!(
                        test.is_none() && side.is_none(),
                        "{where_}: instruction {index} ({construct}) compiled a condition \
                         that is not native\n{}",
                        render(&chunk)
                    );
                } else if outside_side_table(body.expr(conditional.condition)) {
                    let test = test.unwrap_or_else(|| {
                        panic!(
                            "{where_}: instruction {index} ({construct}) has a condition that \
                             calls or sends and no test region\n{}",
                            render(&chunk)
                        )
                    });
                    let ops = &chunk.ops[usize::cast_from(test.test)..usize::cast_from(test.body)];
                    assert!(
                        matches!(ops.first(), Some(Op::LoopTest { index: at, .. }) if *at == at_u32(index))
                            && matches!(ops.last(), Some(Op::LoopTested { index: at, .. }) if *at == at_u32(index)),
                        "{where_}: instruction {index} ({construct}) has a test region that is \
                         not its own LoopTest .. LoopTested\n{}",
                        render(&chunk)
                    );
                    let actual = ops.iter().filter_map(Root::of).next_back();
                    assert_eq!(
                        actual,
                        Some(expected),
                        "{where_}: instruction {index} ({construct}) has a condition calling \
                         for {expected:?} and a test region ending in {actual:?}\n{}",
                        render(&chunk)
                    );
                    assert!(side.is_none());
                    seen.loop_tests += 1;
                } else {
                    assert!(
                        side.is_some() && test.is_none(),
                        "{where_}: instruction {index} ({construct}) has a value-only \
                         condition outside the side table\n{}",
                        render(&chunk)
                    );
                    seen.side_conditions += 1;
                }
            }
            continue;
        }

        // **A `GUARD ... WHEN` condition** of reads and operators is in the
        // side table, and any other is not: one that calls or sends, or that
        // is not native, is walked on every test.
        if let InstructionKind::Guard(guard) = &instruction.kind {
            let side = chunk.condition_at(index);
            match guard.condition.as_ref().map(|c| body.expr(*c)) {
                Some(condition)
                    if root_of(condition) != Root::EvalExpr && !outside_side_table(condition) =>
                {
                    assert!(
                        side.is_some(),
                        "{where_}: instruction {index} ({construct}) has a value-only WHEN \
                         outside the side table\n{}",
                        render(&chunk)
                    );
                    seen.guard_side_conditions += 1;
                }
                _ => assert!(
                    side.is_none(),
                    "{where_}: instruction {index} ({construct}) put a WHEN that calls, sends \
                     or is not native in the side table\n{}",
                    render(&chunk)
                ),
            }
        }

        // **A `REPLY` with an expression** opens its region with
        // `Op::ReplyCheck`, so that its refusal outside a method comes before
        // the expression's ops, and ends it with `Op::ExecWith`; a bare one is
        // the one `Op::Exec`.
        if let InstructionKind::Reply { expression } = &instruction.kind {
            let ops: Vec<&Op> = region
                .iter()
                .filter(|op| !matches!(op, Op::TraceClause { .. }))
                .collect();
            if expression.is_some() {
                assert!(
                    matches!(ops.first(), Some(Op::ReplyCheck { index }) if *index == at)
                        && matches!(ops.last(), Some(Op::ExecWith { index, .. }) if *index == at)
                        && !ops.iter().any(|op| matches!(op, Op::Exec { .. })),
                    "{where_}: instruction {index} ({construct}) is not ReplyCheck, its \
                     expression's ops, then ExecWith\n{}",
                    render(&chunk)
                );
                seen.reply_values += 1;
            } else {
                assert!(
                    matches!(ops[..], [Op::Exec { index }] if *index == at),
                    "{where_}: instruction {index} ({construct}) is a bare REPLY compiled to \
                     something other than its one Exec\n{}",
                    render(&chunk)
                );
                seen.bare_replies += 1;
            }
        }

        // **A `RAISE` with an expression** is, for each expression in the
        // order `r[instr.raise.order]` gives, its ops into the register its
        // part's layout names and the `Op::RaisePart` behind them, and then
        // `Op::ExecWith` from the first of those registers; nothing else, and
        // no `Op::Exec`. One with none is the one `Op::Exec`.
        if let InstructionKind::Raise(raise) = &instruction.kind {
            let ops: Vec<&Op> = region
                .iter()
                .filter(|op| !matches!(op, Op::TraceClause { .. }))
                .collect();
            match crate::run::RaiseLayout::of(raise) {
                Some(layout) => {
                    let Some(Op::ExecWith {
                        index: last,
                        src: base,
                    }) = ops.last()
                    else {
                        panic!(
                            "{where_}: instruction {index} ({construct}) does not end in \
                             ExecWith\n{}",
                            render(&chunk)
                        );
                    };
                    assert_eq!(*last, at);
                    let base = *base;
                    let slot = |offset: Option<u16>| base + offset.expect("the part is present");
                    let mut expected: Vec<(crate::run::RaisePart, u16, Option<ExprRef<'_>>)> =
                        Vec::new();
                    use crate::run::RaisePart;
                    if let Some(expr) = raise.rc.as_ref().map(|e| body.expr(*e)) {
                        expected.push((RaisePart::Code, slot(layout.code), Some(expr)));
                    }
                    if let Some(expr) = raise.description.as_ref().map(|e| body.expr(*e)) {
                        expected.push((
                            RaisePart::Description,
                            slot(layout.description),
                            Some(expr),
                        ));
                    }
                    if let Some(expr) = raise.additional.as_ref().map(|e| body.expr(*e)) {
                        expected.push((RaisePart::Additional, slot(layout.additional), Some(expr)));
                    }
                    // An item has a part of its own, its trace lines, only
                    // in a clause that echoes values, and then every item
                    // has one. Without it the items' ops are the `Array`
                    // part's group.
                    let echoed = ops.iter().any(|op| {
                        matches!(
                            op,
                            Op::RaisePart {
                                part: RaisePart::Item,
                                ..
                            }
                        )
                    });
                    if let Some(items) = &raise.array {
                        let array = slot(layout.array);
                        for (k, item) in (1..).zip(items) {
                            if echoed {
                                let src = match item {
                                    Some(_) => array + k,
                                    None => Op::ARG_OMITTED,
                                };
                                expected.push((
                                    RaisePart::Item,
                                    src,
                                    item.as_ref().map(|e| body.expr(*e)),
                                ));
                            }
                        }
                        expected.push((RaisePart::Array, array, None));
                    }
                    if let Some(expr) = raise
                        .result
                        .as_ref()
                        .and_then(|r| r.value.as_ref())
                        .map(|e| body.expr(*e))
                    {
                        expected.push((RaisePart::Result, slot(layout.result), Some(expr)));
                    }
                    let mut group = 0;
                    let mut parts = expected.iter();
                    for (op_at, op) in ops[..ops.len() - 1].iter().enumerate() {
                        assert!(
                            !matches!(op, Op::Exec { .. } | Op::ExecWith { .. }),
                            "{where_}: instruction {index} ({construct}) holds a second \
                             Exec or ExecWith\n{}",
                            render(&chunk)
                        );
                        let Op::RaisePart {
                            index: owner,
                            part,
                            src,
                        } = op
                        else {
                            continue;
                        };
                        let Some((want, want_src, expr)) = parts.next() else {
                            panic!(
                                "{where_}: instruction {index} ({construct}) has a {part:?} \
                                 past its last part\n{}",
                                render(&chunk)
                            );
                        };
                        assert!(
                            *owner == at && part == want && src == want_src,
                            "{where_}: instruction {index} ({construct}) has {part:?} src={src} \
                             where {want:?} src={want_src} was due\n{}",
                            render(&chunk)
                        );
                        match expr {
                            None if *part == RaisePart::Item => {
                                assert!(
                                    op_at == group,
                                    "{where_}: instruction {index} ({construct}) has ops for an \
                                     omitted item\n{}",
                                    render(&chunk)
                                );
                            }
                            None if echoed => {
                                assert!(
                                    op_at == group,
                                    "{where_}: instruction {index} ({construct}) has ops between \
                                     its last item and its Array part\n{}",
                                    render(&chunk)
                                );
                            }
                            Some(expr) => {
                                let root = root_of(*expr);
                                let actual = ops[group..op_at]
                                    .iter()
                                    .copied()
                                    .filter_map(Root::of)
                                    .next_back();
                                assert_eq!(
                                    actual,
                                    Some(root),
                                    "{where_}: instruction {index} ({construct}) has a {part:?} \
                                     calling for {root:?} and a group ending in {actual:?}\n{}",
                                    render(&chunk)
                                );
                                *seen.roots.entry(root).or_default() += 1;
                            }
                            // The items: one value per written item, in order.
                            None => {
                                let written = raise
                                    .array
                                    .iter()
                                    .flatten()
                                    .filter(|item| item.is_some())
                                    .count();
                                assert!(
                                    ops[group..op_at]
                                        .iter()
                                        .all(|op| !matches!(op, Op::RaisePart { .. }))
                                        && written <= op_at - group,
                                    "{where_}: instruction {index} ({construct}) has an ARRAY \
                                     whose items did not compile ahead of its part\n{}",
                                    render(&chunk)
                                );
                            }
                        }
                        *seen
                            .raise_parts
                            .entry(match part {
                                RaisePart::Code => "Code",
                                RaisePart::Description => "Description",
                                RaisePart::Additional => "Additional",
                                RaisePart::Item => "Item",
                                RaisePart::Array => "Array",
                                RaisePart::Result => "Result",
                            })
                            .or_default() += 1;
                        group = op_at + 1;
                    }
                    assert!(
                        parts.next().is_none(),
                        "{where_}: instruction {index} ({construct}) is missing a part\n{}",
                        render(&chunk)
                    );
                    seen.raise_values += 1;
                }
                None => {
                    assert!(
                        matches!(ops[..], [Op::Exec { index }] if *index == at),
                        "{where_}: instruction {index} ({construct}) is a RAISE with no \
                         expression compiled to something other than its one Exec\n{}",
                        render(&chunk)
                    );
                    seen.bare_raises += 1;
                }
            }
            continue;
        }

        // The expression a promoted clause's region must end in, for the
        // constructs whose expression `compile` offers to `push_native` as a
        // whole slot. A `WHEN CASE`'s values are compared inside
        // `Op::WhenTest`, which is not a slot this file has anything to state
        // about that the op's presence in the stream has not already said.
        let expected = match &instruction.kind {
            InstructionKind::Assignment { value, .. } => Some(root_of(body.expr(*value))),
            // A `SAY`, a `RETURN`, an `EXIT`, a `PUSH` and a `QUEUE` are one
            // row: each offers its whole expression to `push_native` and each
            // bare form holds none, so `None` here is a clause whose region
            // ends in no `Root` at all.
            InstructionKind::Say { expression }
            | InstructionKind::Return { expression }
            | InstructionKind::Exit { expression }
            | InstructionKind::Push { expression }
            | InstructionKind::Queue { expression } => {
                expression.as_ref().map(|e| root_of(body.expr(*e)))
            }
            InstructionKind::If { condition, .. } => Some(condition_root(body.expr(*condition))),
            // An instruction `Op::ExecValue` runs from its expression's value.
            InstructionKind::Numeric {
                expression: Some(expression),
                ..
            }
            | InstructionKind::Trace(interrexx_parse::Trace::Value(expression)) => {
                Some(root_of(body.expr(*expression)))
            }
            InstructionKind::Address(address)
                if address.command.is_none() && address.environment.is_none() =>
            {
                address.dynamic.as_ref().map(|e| root_of(body.expr(*e)))
            }
            InstructionKind::Options { expression } | InstructionKind::Interpret { expression } => {
                Some(root_of(body.expr(*expression)))
            }
            InstructionKind::Command { expression } | InstructionKind::Reply { expression } => {
                expression.as_ref().map(|e| root_of(body.expr(*e)))
            }
            // A `SELECT CASE`'s expression, filed by `Op::SelectCase` behind
            // it, and a plain `SELECT`'s region, which ends in no `Root`.
            InstructionKind::Select { case, .. } => case.as_ref().map(|e| root_of(body.expr(*e))),
            InstructionKind::When { condition, .. } => {
                match condition_root(body.expr(*condition)) {
                    Root::EvalExpr => None,
                    root => Some(root),
                }
            }
            // A message instruction whose term and assigned value are native
            // ends in its send, and one whose are not in no `Root` at all.
            InstructionKind::Message { term, value } => {
                let native_term = match body.expr(*term).kind() {
                    ExprView::Message {
                        target,
                        super_class,
                        args,
                        cascade,
                        ..
                    } => {
                        (value.is_some() || !cascade)
                            && native(target, Some(1))
                            && super_class.is_none_or(|scope| native(scope, None))
                            && arguments(args, Some(0), 1)
                            && value
                                .as_ref()
                                .is_none_or(|value| native(body.expr(*value), Some(0)))
                    }
                    _ => false,
                };
                native_term.then_some(Root::Send)
            }
            _ => continue,
        };
        let actual = region.iter().filter_map(Root::of).next_back();
        assert_eq!(
            actual,
            expected,
            "{where_}: instruction {index} ({construct}) has an expression calling for \
             {expected:?} and a compiled clause ending in {actual:?}\n{}",
            render(&chunk)
        );
        if let Some(root) = expected {
            *seen.roots.entry(root).or_default() += 1;
        }
    }
}

fn condition_root(condition: ExprRef<'_>) -> Root {
    if let ExprView::Logical(items) = condition.kind()
        && u16::try_from(items.len()).is_ok()
        && items.iter().flatten().all(|item| native(item, Some(0)))
        && let Some(last) = items.iter().flatten().last()
    {
        return root_of(last);
    }
    root_of(condition)
}

/// Every body of `program`: the main one and each directive's.
fn bodies(program: &interrexx_parse::Program) -> Vec<(&CodeBody, String)> {
    let mut out = vec![(&program.main, "main".to_string())];
    for directive in &program.directives {
        let (body, what) = match &directive.kind {
            DirectiveKind::Method(method) => (method.body.as_ref(), "::METHOD"),
            DirectiveKind::Attribute(attribute) => (attribute.body.as_ref(), "::ATTRIBUTE"),
            DirectiveKind::Routine(routine) => (routine.body.as_ref(), "::ROUTINE"),
            DirectiveKind::Annotate(_)
            | DirectiveKind::Class(_)
            | DirectiveKind::Constant(_)
            | DirectiveKind::Options(_)
            | DirectiveKind::Requires(_)
            | DirectiveKind::Resource(_) => (None, ""),
        };
        if let Some(body) = body {
            out.push((body, what.to_string()));
        }
    }
    out
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

/// Every corpus program `differential.txt` names, each once, sorted.
fn corpus_programs() -> Vec<String> {
    let path = corpus_dir().join("differential.txt");
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let mut names: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect();
    names.sort();
    names.dedup();
    assert!(
        !names.is_empty(),
        "{} names no program -- this sweep would run on nothing",
        path.display()
    );
    names
}

/// **Every instruction of every corpus body that the minimum promotion set
/// covers compiles to that construct's own ops**, and every promoted value
/// expression ends in the op its shape calls for.
#[test]
fn every_corpus_body_compiles_the_minimum_promotion_set_to_its_own_ops() {
    // **On the stack the interpreter compiles bodies on, and that is not a
    // precaution.** `compile`'s expression walk takes a frame per operator,
    // and `corpus/lang/deep_nested_expr.rex` is one assignment nesting three
    // thousand of them. Calling `compile` from a libtest thread is ordinary --
    // `golden_tests` does it throughout -- and what is not ordinary is doing
    // it on a body this deep, which is why this sweep is the one that
    // overflowed and why it is this sweep that asks for a stack rather than
    // the callers around it. `INTERPRETER_STACK_BYTES` is the size because
    // that is what `on_interpreter_thread` gives the same walk in production.
    // Measured with the sweep called inline instead: it aborts the
    // whole test binary at `RUST_MIN_STACK=2621440` and passes at `2883584`,
    // against a libtest thread's own 2 MiB.
    let sweep = std::thread::Builder::new()
        .stack_size(crate::INTERPRETER_STACK_BYTES)
        .spawn(sweep_every_corpus_body)
        .expect("spawning the sweep thread");
    if let Err(panic) = sweep.join() {
        std::panic::resume_unwind(panic);
    }
}

/// The translation errors that the directives declared so far decide.
const DECLARATION_ERRORS: &[(u16, u16)] = &[
    (33, 1),
    (99, 901),
    (99, 902),
    (99, 903),
    (99, 906),
    (99, 931),
    (99, 932),
    (99, 942),
    (99, 945),
];

/// The sweep itself, split out so that the test above is the thread it runs
/// on and nothing else.
fn sweep_every_corpus_body() {
    let dir = corpus_dir();
    let programs = corpus_programs();
    let mut seen = Seen::default();
    let mut bodies_checked = 0usize;

    for name in &programs {
        let path = dir.join(name);
        let text =
            fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        // Loud rather than skipped, except for a program whose subject is a
        // translation error its declarations decide, which the oracle refuses
        // as well and which has no body to run.
        let program = match parse_program(text) {
            Ok(program) => program,
            Err(e) if DECLARATION_ERRORS.contains(&(e.code(), e.sub())) => continue,
            Err(e) => panic!("{name}: a corpus program does not parse: {e:?}"),
        };
        for (body, what) in bodies(&program) {
            check_body(
                body,
                &program.symbols,
                &program.source,
                &format!("{name} {what}"),
                &mut seen,
            );
            bodies_checked += 1;
        }
    }

    // The population is not empty and the bodies are not empty, because every
    // assertion above is inside two loops and an empty one of either passes
    // without checking anything.
    assert!(
        programs.len() > 1,
        "differential.txt named {} programs",
        programs.len()
    );
    assert!(bodies_checked >= programs.len());

    // **The anti-vacuity control.** Every construct of the minimum promotion
    // set, and every expression root a promoted clause can end in, is reached
    // by this population at least once -- so a row of `promoted_as` or of
    // `root_of` that nothing exercises is red here rather than a check that
    // happens to hold over an empty set.
    for construct in [
        "IF",
        "SELECT",
        "WHEN",
        "WHEN CASE",
        "assignment",
        "SAY",
        "RETURN",
        "EXIT",
        "PUSH",
        "QUEUE",
        "CALL name",
        "message send",
        "LEAVE",
        "ITERATE",
        "NOP",
        "THEN",
        "label",
        "NUMERIC",
        "TRACE VALUE",
        "ADDRESS VALUE",
        "OPTIONS",
        "INTERPRET",
        "command",
        "GUARD WHEN",
        "REPLY",
        "RAISE",
    ] {
        assert!(
            seen.constructs.contains_key(construct),
            "no corpus body contains a promoted {construct}, so this sweep says nothing about it"
        );
    }
    assert!(
        seen.constructs.contains_key("DO") || seen.constructs.contains_key("LOOP"),
        "no corpus body contains a promoted DO or LOOP"
    );
    for keyword in ["IF", "WHEN"] {
        assert!(
            seen.native_conditions.contains_key(keyword),
            "no corpus {keyword} compiled its condition to native ops, so the {keyword} rows \
             above hold only because every condition declined"
        );
    }
    assert!(
        seen.loop_tests > 0 && seen.side_conditions > 0,
        "no corpus loop condition compiled to a test region ({}) or to the side table ({}), so \
         the condition rows above hold only over an empty set",
        seen.loop_tests,
        seen.side_conditions
    );
    assert!(
        seen.guard_side_conditions > 0,
        "no corpus GUARD WHEN compiled its condition to the side table, so the GUARD row above \
         holds only over an empty set"
    );
    assert!(
        seen.reply_values > 0 && seen.bare_replies > 0,
        "no corpus REPLY with an expression ({}) or bare REPLY ({}) was seen, so the REPLY rows \
         above hold only over an empty set",
        seen.reply_values,
        seen.bare_replies
    );
    assert!(
        seen.raise_values > 0 && seen.bare_raises > 0,
        "no corpus RAISE with an expression ({}) or with none ({}) was seen, so the RAISE rows \
         above hold only over an empty set",
        seen.raise_values,
        seen.bare_raises
    );
    for part in ["Code", "Description", "Additional", "Array", "Result"] {
        assert!(
            seen.raise_parts.contains_key(part),
            "no corpus RAISE compiled a {part} part, so the RAISE row says nothing about it: {:?}",
            seen.raise_parts
        );
    }
    assert!(
        seen.native_header_values > 0,
        "no corpus DO/LOOP header slot compiled to native ops, so the header rows above hold \
         only because every header expression declined"
    );
    for root in [
        Root::CallExpr,
        Root::Send,
        Root::List,
        Root::Reference,
        Root::Const,
        Root::LoadConstant,
        Root::Load,
        Root::Arith,
        Root::Binary,
        Root::Prefix,
    ] {
        assert!(
            seen.roots.contains_key(&root),
            "no promoted corpus clause has a {root:?} expression, so this sweep says nothing \
             about that shape"
        );
    }
}
