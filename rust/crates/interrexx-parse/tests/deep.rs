//! A deep `1 + 1 + ... + 1` chain must parse and drop on a normal,
//! default-stack test thread, not just on the wide thread `rexx` uses.

use interrexx_parse::{MAX_EXPR_DEPTH, parse_program};

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

/// Runs `parse_program` on a 512 MiB thread, as `interrexx`'s public entry
/// point runs, which is the stack every depth limit here is calibrated
/// against.
fn parse_on_a_sized_thread(bytes: Vec<u8>) -> Result<(), interrexx_parse::ParseError> {
    std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(move || parse_program(bytes).map(drop))
        .expect("spawns")
        .join()
        .expect("the thread itself must not panic")
}

/// Parenthesis nesting past `MAX_EXPR_DEPTH` raises `11.1`
/// (`r[expr.index.nesting]`) instead of aborting with a native stack
/// overflow. The parser recurses to `MAX_EXPR_DEPTH` before it raises, so
/// this also checks that the sized thread's stack holds that many levels.
#[test]
fn a_paren_nesting_past_the_native_cliff_raises_11_1_instead_of_aborting() {
    let bytes = deep_paren_program(100_000);
    let handle = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(move || parse_program(bytes))
        .expect("spawns");
    let err = handle.join().expect("the thread itself must not panic");
    let err = err.expect_err("100,000 parens is past MAX_EXPR_DEPTH");
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
    let err = parse_on_a_sized_thread(deep_call_program(100_000))
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

    let err = parse_on_a_sized_thread(src.into_bytes())
        .expect_err("half the budget in each construct must still exceed the shared one");
    assert_eq!((err.code(), err.sub()), (11, 1));
}
