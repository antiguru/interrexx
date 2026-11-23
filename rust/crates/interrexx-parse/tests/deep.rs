//! A deep `1 + 1 + ... + 1` chain must parse and drop on a normal,
//! default-stack test thread, not just on the wide thread `rexx` uses.

use interrexx_parse::{Expr, ExprKind, MAX_EXPR_DEPTH, Operator, parse_program, with_parser_stack};

/// Builds `say (((...('a')...)))` with `depth` levels of parenthesis
/// nesting.
fn deep_paren_program(depth: usize) -> Vec<u8> {
    let mut src = String::from("say ");
    src.push_str(&"(".repeat(depth));
    src.push_str("'a'");
    src.push_str(&")".repeat(depth));
    src.push('\n');
    src.into_bytes()
}

/// Builds `say f(f(...f('a')...))` with `depth` levels of call nesting.
fn deep_call_program(depth: usize) -> Vec<u8> {
    let mut src = String::from("say ");
    src.push_str(&"f(".repeat(depth));
    src.push_str("'a'");
    src.push_str(&")".repeat(depth));
    src.push('\n');
    src.into_bytes()
}

/// Parenthesis nesting past `MAX_EXPR_DEPTH` raises `11.1`
/// (`r[expr.index.nesting]`) instead of aborting with a native stack
/// overflow. The parser recurses to `MAX_EXPR_DEPTH` before it raises, so
/// this also checks that the sized thread's stack holds that many levels.
#[test]
fn a_paren_nesting_past_the_native_cliff_raises_11_1_instead_of_aborting() {
    let bytes = deep_paren_program(100_000);
    let err = with_parser_stack(|| parse_program(bytes))
        .expect("spawns")
        .expect_err("100,000 parens is past MAX_EXPR_DEPTH");
    assert_eq!((err.code(), err.sub()), (11, 1));
}

/// Documents, rather than fixes, a gap: `MAX_EXPR_DEPTH` is sized for the
/// 512 MiB thread, and a default-stack thread overflows natively far below
/// it. A caller on a small thread is not protected by the counter, only a
/// sized thread is. Shallow nesting still parses there.
#[test]
fn a_shallow_paren_nesting_still_parses_on_a_default_stack_thread() {
    let program = parse_program(deep_paren_program(300)).expect("parses");
    drop(program);
}

/// Inputs accepted by the grammar can need more stack than the caller's
/// thread. This exercises the same entry point a standalone embedder uses.
#[test]
fn a_deeply_nested_program_parses_on_the_parser_stack() {
    let source = deep_call_program(5_000);
    let line = source.strip_suffix(b"\n").expect("a physical line");
    let program = with_parser_stack(|| interrexx_parse::parse_lines(&[line]))
        .expect("spawns")
        .expect("5,000 nested calls are within the grammar limit");
    assert_eq!(program.main.instructions.len(), 1);
}

/// Builds `total = 1 + 1 + ... + 1` (`terms` copies of `1`) then `say total`,
/// as one flat, left-associative chain.
fn deep_sum_program(terms: usize) -> Vec<u8> {
    let mut src = String::from("total = 1");
    for _ in 0..terms {
        src.push_str(" + 1");
    }
    src.push_str("\nsay total\n");
    src.into_bytes()
}

/// A long flat chain parses and drops on a default-stack test thread. Its
/// length is past where the parser and the drop used to recurse once per
/// term (`examples/depth_probe.rs` finds that depth), so a regression in any
/// of those recursions aborts here.
#[test]
fn a_chain_past_the_pre_fix_cliff_parses_and_drops() {
    let program = parse_program(deep_sum_program(5_000)).expect("parses");
    drop(program);
}

/// A flat chain as long as the oracle still answers parses and drops: the
/// parser must not be shallower than the implementation it is compared with.
#[test]
fn the_depth_the_oracle_still_answers_also_parses_and_drops() {
    let program = parse_program(deep_sum_program(100_000)).expect("parses");
    drop(program);
}

/// Nested calls past the sized thread's native cliff raise `11.1` instead of
/// aborting the process.
#[test]
fn a_call_nesting_past_the_native_cliff_raises_11_1_instead_of_aborting() {
    let err = with_parser_stack(|| parse_program(deep_call_program(100_000)))
        .expect("spawns")
        .expect_err("100,000 nested calls is past MAX_EXPR_DEPTH");
    assert_eq!((err.code(), err.sub()), (11, 1));
}

/// Parentheses and calls share **one** depth budget, and this is the test that
/// makes that a requirement rather than a preference.
#[test]
fn parens_and_calls_share_one_budget_rather_than_one_each() {
    let half = usize::try_from(MAX_EXPR_DEPTH).expect("fits") / 2 + 100;
    let mut src = String::from("say ");
    src.push_str(&"f(".repeat(half));
    src.push_str(&"(".repeat(half));
    src.push_str("'a'");
    src.push_str(&")".repeat(half));
    src.push_str(&")".repeat(half));
    src.push('\n');

    let err = with_parser_stack(|| parse_program(src.into_bytes()))
        .expect("spawns")
        .expect_err("half the budget in each construct must still exceed the shared one");
    assert_eq!((err.code(), err.sub()), (11, 1));
}

/// These traits exhaust the default stack on a chain of this length.
#[test]
fn recursive_expression_traits_run_on_the_parser_stack() {
    let leaf = || Expr::new(ExprKind::Literal(Box::from(&b"1"[..])), 0..1);
    let expr = (0..5_000).fold(leaf(), |left, _| Expr::binary(Operator::Plus, left, leaf()));

    with_parser_stack(|| {
        let clone = expr.clone();
        assert!(expr == clone);
        assert!(format!("{expr:?}").starts_with("Expr { kind: Binary"));
    })
    .expect("spawns");
}

/// Builds `say - - ... - 1` with `depth` prefix operators (blank-separated,
/// because `--` starts a comment).
fn deep_prefix_program(depth: usize) -> Vec<u8> {
    let mut src = String::from("say ");
    src.push_str(&"- ".repeat(depth));
    src.push_str("1\n");
    src.into_bytes()
}

/// A prefix chain exactly `MAX_EXPR_DEPTH` long parses on the sized thread,
/// and one more operator raises `11.1` (`r[expr.index.nesting]`).
#[test]
fn a_prefix_chain_parses_at_the_limit_and_raises_11_1_one_past_it() {
    let limit = usize::try_from(MAX_EXPR_DEPTH).expect("fits");
    let program = with_parser_stack(|| parse_program(deep_prefix_program(limit)))
        .expect("spawns")
        .expect("a chain at the limit parses");
    with_parser_stack(move || drop(program)).expect("spawns");
    let err = with_parser_stack(|| parse_program(deep_prefix_program(limit + 1)))
        .expect("spawns")
        .expect_err("one operator past the limit");
    assert_eq!((err.code(), err.sub()), (11, 1));
}

/// A chain far past where the parser used to overflow the native stack raises
/// `11.1` instead of aborting.
#[test]
fn a_prefix_chain_past_the_native_cliff_raises_11_1_instead_of_aborting() {
    let err = with_parser_stack(|| parse_program(deep_prefix_program(2_000_000)))
        .expect("spawns")
        .expect_err("2,000,000 prefix operators is past MAX_EXPR_DEPTH");
    assert_eq!((err.code(), err.sub()), (11, 1));
}

/// Prefix operators and parentheses share the one budget: neither half alone
/// reaches the limit.
#[test]
fn prefix_operators_and_parens_share_one_budget() {
    let half = usize::try_from(MAX_EXPR_DEPTH).expect("fits") / 2 + 100;
    let mut src = String::from("say ");
    src.push_str(&"- ( ".repeat(half));
    src.push('1');
    src.push_str(&")".repeat(half));
    src.push('\n');
    let err = with_parser_stack(|| parse_program(src.into_bytes()))
        .expect("spawns")
        .expect_err("half the budget in each construct must still exceed the shared one");
    assert_eq!((err.code(), err.sub()), (11, 1));
}
