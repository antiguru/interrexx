//! The error table of the class library: for each class, the errors its
//! methods raise, as the code and subcode that `condition('O')~code` answers.
//! The message text of a library error is not specified, so the tables
//! never print it. Each case is a Rexx program, or an expression that the
//! harness wraps in a program that traps SYNTAX and prints the code, and its
//! expected block is the program's standard output.
//!
//! A case runs on this interpreter always, and on the oracle too when
//! `REXX_CORPUS_GATE` is set, where its output must be the same. A case under
//! `ours` is for an answer the spec rules differently from the oracle: only
//! this interpreter runs it.
//!
//! Directives: `error` (the input is one expression, evaluated as the
//! right-hand side of an assignment; `no error` is printed when it raises
//! nothing), `program` (the input is the whole program), and `ours`
//! (`program`, without the oracle). Rewrite the expected blocks with
//! `REWRITE=1`.

#![expect(
    clippy::disallowed_methods,
    reason = "this harness times or bounds real runs"
)]

mod support;
mod watchdog;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use interrexx::Invocation;
use support::oracle::{Termination, locate};

const CASES: &str = "tests/library_errors";

/// Env var that adds the oracle's run of each case.
const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    std::env::var(GATE_ENV).is_ok_and(|value| !value.is_empty() && value != "0")
}

/// The program that evaluates `expression` and prints the code of the SYNTAX
/// condition it raises.
fn wrapped(expression: &str) -> String {
    format!(
        "signal on syntax name h\nv = {}\nsay 'no error'\nexit\nh: say condition('o')~code\n",
        expression.trim()
    )
}

fn fresh_run_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("library-errors-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    dir
}

/// This interpreter's stdout for `program`, which must have ended normally.
fn ours(name: &str, program: &str) -> String {
    let outcome = watchdog::run_bounded(
        name,
        program.as_bytes().to_vec(),
        Invocation::none().with_input(interrexx::ProgramInput::Bytes(Vec::new())),
    );
    assert!(
        !watchdog::did_not_finish(&outcome),
        "{name} did not finish: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    String::from_utf8_lossy(&outcome.stdout).into_owned()
}

/// The oracle's stdout for `program`, from a directory of this run's own.
fn oracle(name: &str, program: &str) -> String {
    let dir = fresh_run_dir();
    let file = dir.join("case.rex");
    fs::write(&file, program).unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    let outcome = locate().run(&file);
    let _ = fs::remove_dir_all(&dir);
    assert!(
        matches!(outcome.termination, Termination::Exited(_)),
        "the oracle did not finish {name}: {:?}",
        outcome.termination
    );
    String::from_utf8_lossy(&outcome.stdout).into_owned()
}

#[test]
fn library_methods_raise_the_errors_their_tables_list() {
    let mut cases = 0;
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            cases += 1;
            let program = match case.directive.as_str() {
                "error" => wrapped(&case.input),
                "program" | "ours" => case.input.clone(),
                other => panic!("unknown directive {other}"),
            };
            let name = format!("{CASES}: {}", case.input.trim());
            let answer = ours(&name, &program);
            if gate_mode() && case.directive != "ours" {
                let theirs = oracle(&name, &program);
                assert_eq!(
                    answer, theirs,
                    "the oracle answers {name} differently; put the case under `ours` \
                     if the spec rules the difference"
                );
            }
            answer
        });
    });
    assert!(cases > 0, "{CASES} ran no case");
}
