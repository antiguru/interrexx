//! The golden tests for `ir::compile`: what a body renders as, recorded in
//! `src/ir/golden_cases/`, and in Rust what a rendering does not show: the
//! `op_of` boundary `run_bounded`'s inclusive absorption guard needs, the
//! chunk cache's "compiled once" guarantee, and the cases that build their
//! source or check the interpreter.

use std::rc::Rc;

use interrexx_parse::{ExprKind, InstructionKind, parse_program};

use super::golden::render;
use super::{Chunk, ChunkTooLarge, NodePath};
use crate::Interp;
use crate::plan::{BodyKey, BodyKind, Plan, ProgramId};
use crate::trace::{ChunkTrace, TraceMode};

/// Parses `source`, builds its plan and compiles it **under the setting every
/// activation starts at** -- the three steps `Interp::chunk_for` otherwise
/// runs one at a time, collapsed for tests that only want the resulting
/// `Chunk`. `#[cfg(test)]` only: this is not a crate entry point.
fn compile_for_test(source: &[u8]) -> Result<Chunk, ChunkTooLarge> {
    compile_for_test_under(source, ChunkTrace::of(TraceMode::NORMAL))
}

/// [`compile_for_test`] under a named trace setting, which is an input to
/// what `compile` emits.
fn compile_for_test_under(source: &[u8], trace: ChunkTrace) -> Result<Chunk, ChunkTooLarge> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    super::compile(&program.main, &plan, trace)
}

/// The setting `TRACE I` puts in force, as far as compilation can see it.
fn intermediates() -> ChunkTrace {
    ChunkTrace::of(crate::trace::mode_from_setting(b"i").expect("I is a valid TRACE setting"))
}

#[test]
fn use_arg_compiles_defaults_without_splitting_the_default_free_binder() {
    let chunk = compile_for_test(b"use strict arg >a, b = (a + 1), , d = (f())\n")
        .expect("compiled defaults");
    let ops = render(&chunk);
    assert!(ops.contains("UseArg CheckCount"), "{ops}");
    assert!(ops.contains("UseArg Bind { slot: 1, skip: 4 }"), "{ops}");
    assert!(
        ops.contains("UseArg BindDefault { slot: 1, src: 0 }"),
        "{ops}"
    );
    assert!(ops.contains("CallArgs slot=3 path=root"), "{ops}");
    assert!(!ops.contains("EvalExpr") && !ops.contains("Exec"), "{ops}");

    let chunk = compile_for_test(b"use strict arg >a, b\n").expect("compiled USE ARG");
    assert_eq!(render(&chunk), "0: Clause index=0 end=2\n1: Exec index=0\n");
}

/// The setting `TRACE R` puts in force, as far as compilation can see it.
fn traced() -> ChunkTrace {
    ChunkTrace::of(crate::trace::mode_from_setting(b"r").expect("R is a valid TRACE setting"))
}

/// A call the address cannot reach leaves the whole slot general, which is the
/// answer it had before there was an address at all.
#[test]
fn a_call_nested_past_the_paths_width_leaves_the_slot_general() {
    // `zz = zb + (zb + (... length('abc') ...))`, right-nested so that the
    // call sits `depth` steps down and every one of those steps is the `R`
    // one.
    fn nested(depth: usize) -> Vec<u8> {
        let mut source = b"zz = ".to_vec();
        for _ in 0..depth {
            source.extend_from_slice(b"zb + (");
        }
        source.extend_from_slice(b"length('abc')");
        source.extend(std::iter::repeat_n(b')', depth));
        source
    }

    let mut path = NodePath::ROOT;
    let mut width = 0usize;
    while let Some(deeper) = path.child(true) {
        path = deeper;
        width += 1;
    }

    let at_the_width = compile_for_test(&nested(width)).expect("the chunk fits");
    let rendered = render(&at_the_width);
    let address = format!("root{}", ".R".repeat(width));
    assert!(
        rendered.contains(&format!("CallArgs slot=0 path={address} site=0 argc=1 ")),
        "the call at the deepest address a NodePath carries did not take one\n{rendered}"
    );
    assert!(
        !rendered.contains("EvalExpr"),
        "a slot whose every node has an op still fell back to eval.rs\n{rendered}"
    );

    let past_the_width = compile_for_test(&nested(width + 1)).expect("the chunk fits");
    assert_eq!(
        render(&past_the_width),
        "0: Clause index=0 end=3\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: Store index=0 at=0 src=0\n"
    );
}

#[test]
fn qualified_calls_use_call_ops_in_computed_clauses() {
    let padding = ", ".repeat(31);
    let source = format!(
        "numeric digits ns:digits('12')\nsay 1 + ns:number(act(), , 4)\n\
         say ns:f({padding}act())\n"
    );
    let chunk = compile_for_test(source.as_bytes()).expect("compiled qualified calls");
    let rendered = render(&chunk);
    assert!(
        rendered.contains("CallArgs slot=0 path=root "),
        "{rendered}"
    );
    assert!(
        rendered.contains("CallArgs slot=0 path=root.R "),
        "{rendered}"
    );
    assert!(
        rendered.contains("CallExpr index=2 slot=0 path=root "),
        "{rendered}"
    );
    assert!(!rendered.contains("EvalExpr"), "{rendered}");

    let traced =
        compile_for_test_under(source.as_bytes(), intermediates()).expect("compiled TRACE I");
    let rendered = render(&traced);
    assert!(
        rendered.contains("CallArgs slot=0 path=root "),
        "{rendered}"
    );
    assert!(
        rendered.contains("CallExpr index=2 slot=0 path=root "),
        "{rendered}"
    );
    assert!(
        rendered.contains("TraceFunction index=2 slot=0 path=root "),
        "{rendered}"
    );
}

/// **The symbol a compiled read names is the one its own expression names**,
/// which the rendered stream cannot say -- `render`'s own comment has why it
/// prints no symbol index.
#[test]
fn a_compiled_read_names_the_symbol_its_expression_does() {
    let program = parse_program(b"zw = za.zi\n".to_vec()).expect("test program parses");
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    let chunk = super::compile(&program.main, &plan, intermediates()).expect("compiles");

    let InstructionKind::Assignment { value, .. } = &program.main.instructions[0].kind else {
        panic!("the program's one instruction is an assignment");
    };
    let ExprKind::Compound(id) = &value.kind else {
        panic!("its value is a compound");
    };
    assert_eq!(
        program.symbols.name(*id),
        "ZA.ZI",
        "the expression names the compound, not the target"
    );

    let named: Vec<_> = chunk
        .ops
        .iter()
        .filter_map(|op| match op {
            super::Op::Load { symbol, .. } | super::Op::TraceRead { symbol, .. } => Some(*symbol),
            _ => None,
        })
        .collect();
    assert_eq!(
        named,
        vec![*id, *id],
        "the load and its echo both name the symbol the expression does"
    );
}

/// The compiled form of the plan's own example loop: the header is a clause
/// region of its own and the construct is the op that closes it.
#[test]
fn a_counted_loop_compiles_its_header_to_a_clause_region_and_its_body_to_a_marker() {
    let chunk = compile_for_test(b"do i = 1 to 3\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=6\n\
         1: LoadConstant dst=0\n\
         2: LoopHeaderValue role=Initial src=0\n\
         3: LoadConstant dst=1\n\
         4: LoopHeaderValue role=To src=1\n\
         5: LoopRun index=0\n\
         6: Clause index=1 end=7\n\
         7: LoopNext index=0\n"
    );
    // One register per header expression, and they are **not** released at the
    // region's end: the loop runs from op 8 with the body's clauses stepped
    // between, so a register handed out again there would be overwritten while
    // the running loop still reads it.
    assert_eq!(chunk.registers, 2);
    assert_eq!(chunk.op_of, vec![0, 6, 7, 8]);
}

/// The compiled `IF` with an `ELSE`, which is the shape the whole promotion
/// is about: both paths are jumps in one stream.
#[test]
fn an_if_with_an_else_compiles_to_a_clause_region_and_two_jumps() {
    let chunk =
        compile_for_test(b"if 1 = 1 then say 'a'\nelse say 'b'\nsay 'c'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=5\n\
         1: LoadConstant dst=0\n\
         2: LoadConstant dst=1\n\
         3: Binary op== lhs=0 rhs=1 dst=0\n\
         4: ConditionJump index=0 reg=0 keyword=IF target=11\n\
         5: Clause index=1 end=6\n\
         6: Clause index=2 end=9\n\
         7: Const dst=0 konst=0\n\
         8: Say index=2 src=0\n\
         9: EndBranch\n\
         10: Jump target=15\n\
         11: Clause index=3 end=12\n\
         12: Clause index=4 end=15\n\
         13: Const dst=0 konst=1\n\
         14: Say index=4 src=0\n\
         15: Clause index=5 end=18\n\
         16: Const dst=0 konst=2\n\
         17: Say index=5 src=0\n"
    );
    // Two registers throughout: the comparison's right operand takes one of
    // its own beside the register the condition lands in, and gives it back
    // inside `push_native`'s own binary arm as soon as the operation has run.
    // The condition's own register is released at the `IF`'s clause end, so
    // each promoted `SAY` below gets register 0 back.
    assert_eq!(chunk.registers, 2);
    assert_eq!(chunk.op_of, vec![0, 5, 6, 9, 12, 15, 18]);
    // Three distinct literals, one entry each, in the order they were first
    // seen -- which is the order the `konst` fields above read.
    assert_eq!(
        chunk.consts,
        vec![
            Box::from(&b"a"[..]),
            Box::from(&b"b"[..]),
            Box::from(&b"c"[..])
        ]
    );
}

/// The compiled `SELECT` with an `OTHERWISE`: one clause region for the header
/// and one per listed `WHEN`, chained by the `JumpUnless` each `WHEN` ends
/// with, and a frame opened over whichever branch wins.
#[test]
fn a_select_with_an_otherwise_compiles_to_a_scan_chain_and_two_frames() {
    let chunk = compile_for_test(
        b"select\n  when 1 = 0 then say 'a'\n  when 2 = 2 then say 'b'\n  otherwise say 'o'\n\
          end\nsay 'after'\n",
    )
    .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=1\n\
         1: Clause index=1 end=6\n\
         2: LoadConstant dst=0\n\
         3: LoadConstant dst=1\n\
         4: Binary op== lhs=0 rhs=1 dst=0\n\
         5: ConditionJump index=1 reg=0 keyword=WHEN target=12\n\
         6: EnterWhen select=0 when=1\n\
         7: Clause index=2 end=8\n\
         8: Clause index=3 end=11\n\
         9: Const dst=0 konst=0\n\
         10: Say index=3 src=0\n\
         11: EndWhen\n\
         12: Clause index=4 end=17\n\
         13: LoadConstant dst=0\n\
         14: LoadConstant dst=1\n\
         15: Binary op== lhs=0 rhs=1 dst=0\n\
         16: ConditionJump index=4 reg=0 keyword=WHEN target=23\n\
         17: EnterWhen select=0 when=4\n\
         18: Clause index=5 end=19\n\
         19: Clause index=6 end=22\n\
         20: Const dst=0 konst=1\n\
         21: Say index=6 src=0\n\
         22: EndWhen\n\
         23: EnterOtherwise select=0\n\
         24: Clause index=7 end=25\n\
         25: Clause index=8 end=28\n\
         26: Const dst=0 konst=2\n\
         27: Say index=8 src=0\n\
         28: EndWhen\n\
         29: Clause index=9 end=31\n\
         30: Exec index=9\n\
         31: Clause index=10 end=34\n\
         32: Const dst=0 konst=3\n\
         33: Say index=10 src=0\n"
    );
    assert_eq!(
        chunk.registers, 2,
        "the second WHEN reuses the registers the first one released, and a \
         comparison's right operand takes one beside the answer's own"
    );
    assert_eq!(
        chunk.op_of,
        vec![0, 1, 7, 8, 11, 18, 19, 22, 25, 28, 31, 34]
    );
}

/// An expression whose qualified call is deeper than a call op can address.
fn unaddressable_call() -> String {
    let depth = u32::BITS as usize;
    format!("{}zn:zf(){}", "0 + (".repeat(depth), ")".repeat(depth))
}

/// **A `WHEN`'s condition outside the native set stays one
/// [`super::Op::WhenTest`] and takes no `ConditionJump` op**, which is the
/// adjacent refusal to the two `SELECT` streams above and the sibling of
/// `a_condition_outside_the_native_set_stays_one_eval_expr`.
#[test]
fn a_when_condition_outside_the_native_set_stays_one_when_test() {
    let source = format!("select\n  when {} then nop\nend\n", unaddressable_call());
    let chunk = compile_for_test(source.as_bytes()).expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=1\n\
         1: Clause index=1 end=4\n\
         2: WhenTest index=1 case=- dst=0\n\
         3: JumpUnless reg=0 target=8\n\
         4: EnterWhen select=0 when=1\n\
         5: Clause index=2 end=6\n\
         6: Clause index=3 end=7\n\
         7: EndWhen\n\
         8: Clause index=4 end=10\n\
         9: Exec index=4\n"
    );
}

/// The same `IF` compiled under `TRACE R`: the clause echo is an **op** of the
/// clause's own region, and the region is one op longer for it.
#[test]
fn a_traced_if_carries_its_clause_echo_as_an_op_of_the_region() {
    let chunk = compile_for_test_under(b"if 1 = 1 then say 'a'\nelse say 'b'\nsay 'c'\n", traced())
        .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=6\n\
         1: TraceClause index=0\n\
         2: LoadConstant dst=0\n\
         3: LoadConstant dst=1\n\
         4: Binary op== lhs=0 rhs=1 dst=0\n\
         5: ConditionJump index=0 reg=0 keyword=IF target=14\n\
         6: Clause index=1 end=8\n\
         7: TraceClause index=1\n\
         8: Clause index=2 end=12\n\
         9: TraceClause index=2\n\
         10: Const dst=0 konst=0\n\
         11: Say index=2 src=0\n\
         12: EndBranch\n\
         13: Jump target=20\n\
         14: Clause index=3 end=16\n\
         15: TraceClause index=3\n\
         16: Clause index=4 end=20\n\
         17: TraceClause index=4\n\
         18: Const dst=0 konst=1\n\
         19: Say index=4 src=0\n\
         20: Clause index=5 end=24\n\
         21: TraceClause index=5\n\
         22: Const dst=0 konst=2\n\
         23: Say index=5 src=0\n"
    );
    // The echo op addresses no register, so the extra op changes nothing the
    // driver has to reserve.
    assert_eq!(chunk.registers, 2);
    assert_eq!(chunk.op_of, vec![0, 6, 8, 12, 16, 20, 24]);
}

/// `run_bounded`'s absorption guard is inclusive, so a construct's resume
/// point can be `end`, which is one past its last instruction. A map that
/// stops at `len - 1` panics there rather than at compile.
#[test]
fn the_instruction_map_has_an_entry_one_past_the_last_instruction() {
    let source = b"if 1 = 1 then say 'a'\nsay 'b'\n";
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    let chunk =
        super::compile(&program.main, &plan, ChunkTrace::of(TraceMode::NORMAL)).expect("compiles");
    assert_eq!(
        chunk.op_of.len(),
        program.main.instructions.len() + 1,
        "one entry per instruction plus the end entry"
    );
}

/// A program calling one routine in a loop compiles that routine's body
/// once: `chunk_for` must answer every later lookup from the cache rather
/// than recompiling it. This counts `compile`'s own call count rather than
/// inspecting what a lookup returns, because an implementation that
/// recompiles on every call and happens to return an equal-looking `Chunk`
/// would pass a check that only looks at the result -- `Rc::ptr_eq` below
/// pins the cache's *identity* guarantee, which the call count alone does
/// not, but neither one alone rules out both degenerate implementations at
/// once.
#[test]
fn chunk_for_compiles_a_body_once_across_repeated_lookups() {
    let program = parse_program(b"say 1\n".to_vec()).expect("test program parses");
    let key = BodyKey {
        program: ProgramId::nth(0),
        directive: None,
    };
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );

    let mut interp = Interp::new();
    let before = super::compile::compile_calls();

    let untraced = ChunkTrace::of(TraceMode::NORMAL);
    let first = interp
        .chunk_for(key, untraced, &program.main, &plan)
        .expect("compiles");
    let second = interp
        .chunk_for(key, untraced, &program.main, &plan)
        .expect("cached");
    let third = interp
        .chunk_for(key, untraced, &program.main, &plan)
        .expect("cached");

    assert_eq!(
        super::compile::compile_calls() - before,
        1,
        "compile ran exactly once across three lookups under the same key"
    );
    assert!(
        Rc::ptr_eq(&first, &second),
        "second lookup is the same chunk"
    );
    assert!(Rc::ptr_eq(&first, &third), "third lookup is the same chunk");
}

/// **One body, two settings, two chunks.** The trace setting is an input to
/// compilation, so a cache keyed on `BodyKey` alone hands the body
/// entered under the second setting a stream compiled for the first one.
#[test]
fn one_body_under_two_trace_settings_is_two_cached_chunks() {
    let program = parse_program(b"if 1 = 1 then say 1\n".to_vec()).expect("test program parses");
    let key = BodyKey {
        program: ProgramId::nth(0),
        directive: None,
    };
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    let untraced = ChunkTrace::of(TraceMode::NORMAL);

    let mut interp = Interp::new();
    let before = super::compile::compile_calls();

    let silent = interp
        .chunk_for(key, untraced, &program.main, &plan)
        .expect("compiles");
    let echoing = interp
        .chunk_for(key, traced(), &program.main, &plan)
        .expect("compiles");

    assert_eq!(
        super::compile::compile_calls() - before,
        2,
        "the second setting was answered from the first setting's cached chunk"
    );
    assert!(
        !Rc::ptr_eq(&silent, &echoing),
        "both settings got the same chunk, so one of them is running the other's stream"
    );
    assert!(
        render(&silent) != render(&echoing),
        "the two settings compiled to the same stream, so the setting decided nothing"
    );
    assert!(
        Rc::ptr_eq(
            &silent,
            &interp
                .chunk_for(key, untraced, &program.main, &plan)
                .expect("cached")
        ),
        "the first setting's chunk was evicted rather than kept beside the second's"
    );
}

/// **The first `SetTrace` retires the cache** (`r[native.threads.trace]`): a body compiled
/// settled before it is compiled again, unsettled, the next time it is looked up, and the settled
/// chunk is kept beside the cache until its program is freed, when its rows go with the
/// program's. The plan is the cached one, as a running activation's is, so the lookup ahead of
/// the cache is forgotten too.
#[test]
fn making_the_trace_switchable_recompiles_each_body_unsettled() {
    use super::trace_flow::Setting;
    let program = parse_program(b"x = 1\nsay x\n".to_vec()).expect("test program parses");
    let key = BodyKey {
        program: ProgramId::nth(0),
        directive: None,
    };
    let plan = Rc::new(Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    ));
    let untraced = ChunkTrace::of(TraceMode::NORMAL);

    let mut interp = Interp::new();
    interp.plans.insert(key, Rc::clone(&plan));
    let settled = interp
        .chunk_for(key, untraced, &program.main, &plan)
        .expect("compiles");
    assert_eq!(settled.setting_at(0), Some(Setting::Known(untraced)));

    interp.make_trace_switchable();
    let unsettled = interp
        .chunk_for(key, untraced, &program.main, &plan)
        .expect("compiles");
    assert!(
        !Rc::ptr_eq(&settled, &unsettled),
        "the lookup after the switch answered the chunk compiled before it"
    );
    assert_eq!(unsettled.setting_at(0), Some(Setting::Unknown));
    assert!(
        Rc::ptr_eq(
            &unsettled,
            &interp
                .chunk_for(key, untraced, &program.main, &plan)
                .expect("cached")
        ),
        "the unsettled chunk was not cached"
    );
    assert!(
        interp
            .retired_chunks
            .iter()
            .any(|(retired, chunk)| *retired == key && Rc::ptr_eq(chunk, &settled)),
        "the settled chunk was dropped rather than retired"
    );

    interp.prune_program_rows(&[key.program]);
    assert!(
        interp.retired_chunks.is_empty(),
        "a freed program's retired chunk outlived it"
    );
}

/// **A `RETURN` whose expression is outside the native set keeps its own op**,
/// with one [`super::Op::EvalExpr`] in front of it -- which is the `SAY`
/// fallback and **not** the `IF` one.
#[test]
fn a_return_expression_outside_the_native_set_keeps_its_return_op() {
    let source = format!("return {}\n", unaddressable_call());
    let chunk = compile_for_test(source.as_bytes()).expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: Return index=0 src=0 keyword=RETURN\n"
    );
}

/// The cases in `src/ir/golden_cases/`: each `compile` case compiles its input
/// under the setting `trace=` names (`r` for `TRACE R`, `i` for `TRACE I`,
/// none for the setting every activation starts at) and answers the op stream
/// [`render`] writes, the registers the chunk reserves and its constant table.
/// `REWRITE=1` re-records them after an intended change to the compiler.
#[test]
fn the_recorded_chunks_render_as_committed() {
    datadriven::walk("src/ir/golden_cases", |file| {
        file.run(|case| {
            assert_eq!(case.directive, "compile", "the one directive");
            let trace = match case.args.get("trace").map(|values| values.as_slice()) {
                None => ChunkTrace::of(TraceMode::NORMAL),
                Some([setting]) if setting == "r" => traced(),
                Some([setting]) if setting == "i" => intermediates(),
                Some(other) => panic!("trace={other:?} is not a setting"),
            };
            let chunk =
                compile_for_test_under(case.input.as_bytes(), trace).expect("the chunk fits");
            let consts: String = chunk
                .consts
                .iter()
                .map(|konst| format!(" '{}'", konst.escape_ascii()))
                .collect();
            format!(
                "{}registers: {}\nconsts:{consts}\n",
                render(&chunk),
                chunk.registers
            )
        });
    });
}

/// **A `PARSE` template compiles to its walk**: the opening, which walks
/// the entries before the first operand the stream evaluates, then per such
/// operand its ops and the run from its entry to the next, with the walk's
/// registers below the operand's. A simple variable, like a literal, is read
/// where the walk reaches it.
#[test]
fn a_parse_template_compiles_to_its_walk() {
    let chunk =
        compile_for_test(b"parse var s a (p) b '-' c +(n + 1) d\n").expect("the chunk fits");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=6\n\
         1: ParseBegin index=0 src=omitted base=0 to=2 opening=Var strings=0\n\
         2: Load read=Simple at=4 dst=5\n\
         3: LoadConstant dst=6\n\
         4: Arith op=+ hint=0 lhs=5 rhs=6 dst=5\n\
         5: ParseRun index=0 base=0 from=2 to=4 operand=5\n"
    );
}

/// A template with no operand to evaluate is walked whole by its opening.
#[test]
fn a_template_of_literals_is_one_parse_begin() {
    let chunk = compile_for_test(b"parse value v with a +5 b ',' c, d\n").expect("the chunk fits");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: Load read=Simple at=0 dst=0\n\
         2: ParseBegin index=0 src=0 base=omitted to=5 opening=Value strings=0\n"
    );
}

#[test]
fn parse_trigger_calls_address_their_own_template_entries() {
    let source = b"parse value input() with a ' ' b (separator(a)) c, d +(position(c)) e\n";
    let rendered = render(&compile_for_test(source).expect("the chunk fits"));
    for (slot, site) in [(0, 0), (2, 1), (5, 2)] {
        assert!(
            rendered.contains(&format!("CallArgs slot={slot} path=root site={site} ")),
            "{rendered}"
        );
    }
    assert_eq!(rendered.matches("ParseRun").count(), 2, "{rendered}");
}

/// The shapes the walk takes no runs for are its opening alone, with no
/// walk registers, and the opening names the ones it hands to the template
/// engine whole: a fold, a source read as a line, `PARSE VAR` into words or
/// of a compound. A clause that may trace is walked whole while it does not,
/// including an operand that can change the clause's TRACE setting.
#[test]
fn a_template_with_no_runs_is_its_opening_alone() {
    let normal = || ChunkTrace::of(TraceMode::NORMAL);
    for (source, trace, opening) in [
        (&b"parse value v with a (p + 1) b\n"[..], traced(), "Value"),
        (
            b"parse value v with a (p + 1) b\n",
            intermediates(),
            "Value",
        ),
        (b"parse upper arg a (p + 1) b\n", normal(), "Engine"),
        (b"parse pull a (p + 1) b\n", normal(), "Engine"),
        (b"parse var s a b c\n", normal(), "Engine"),
        (b"parse var s.i a ' ' b\n", normal(), "Engine"),
        (b"parse var s a (trace('I')) b\n", normal(), "Var"),
        (b"parse arg a ' ' b, c, d\n", normal(), "Arg strings=3"),
    ] {
        let rendered = render(&compile_for_test_under(source, trace).expect("the chunk fits"));
        assert!(
            rendered.contains("base=omitted to=")
                && rendered.contains(&format!("opening={opening}"))
                && !rendered.contains("ParseRun"),
            "{}\n{rendered}",
            String::from_utf8_lossy(source)
        );
    }
}

#[test]
fn forward_options_compile_with_early_checks_and_their_own_slots() {
    let source = b"forward array (last(), , 'z') class (scope()) message (name()) to (target())";
    let chunk = compile_for_test(source).expect("compiled FORWARD");
    let ops = render(&chunk);
    let mut after = 0;
    for line in [
        "part=Method",
        "CallArgs slot=0 path=root",
        "part=To",
        "CallArgs slot=1 path=root",
        "part=Message",
        "CallArgs slot=2 path=root",
        "part=Class",
        "CallArgs slot=4 path=root",
        "PushArg src=omitted",
        "ExecWith index=0 src=0",
    ] {
        let at = ops[after..]
            .find(line)
            .unwrap_or_else(|| panic!("{line}\n{ops}"));
        after += at + line.len();
    }
    assert!(
        !ops.contains("EvalExpr") && !ops.contains(" Exec "),
        "{ops}"
    );
}
