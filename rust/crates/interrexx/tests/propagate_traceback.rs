//! The report of a SYNTAX condition that RAISE PROPAGATE passes on untrapped,
//! which is written to stderr and so is out of reach of a specification
//! example.

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use interrexx::{Invocation, run_program};

/// spec: instr.raise.propagate.traceback
///
/// The report shows the condition object's `TRACEBACK` as it stands when the
/// condition reaches the top: a line added to the list `CONDITION('O')`
/// answers is shown, and one that is not a string by its string value.
#[test]
fn a_propagated_condition_reports_the_traceback_as_rexx_code_left_it() {
    let outcome = run_program(
        "/propagate.rex",
        b"call sub\n\
          exit\n\
          sub:\n\
          \x20 signal on syntax\n\
          \x20 x = 1/0\n\
          syntax:\n\
          \x20 t = condition('o')~traceback\n\
          \x20 t~insert('an added line')\n\
          \x20 t~insert(.array~of(1, 2))\n\
          \x20 raise propagate\n"
            .to_vec(),
        Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 214);
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(
        lines[..4],
        [
            "     5 *-*   x = 1/0",
            "     1 *-* call sub",
            "an added line",
            "an Array",
        ],
        "{stderr}"
    );
    assert!(lines[4].starts_with("Error 42:"), "{stderr}");
}

/// spec: instr.raise.propagate
///
/// Re-raised in the caller of the routine that trapped it, the condition's
/// report shows the object's `TRACEBACK` and names the line of the caller's
/// clause.
#[test]
fn a_propagated_condition_reraised_in_a_caller_names_the_callers_line() {
    let outcome = run_program(
        "/propagate.rex",
        b"say 'start'\n\
          call r\n\
          exit\n\
          ::routine r\n\
          \x20 signal on syntax\n\
          \x20 x = 1/0\n\
          syntax:\n\
          \x20 raise propagate\n"
            .to_vec(),
        Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 214);
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(
        lines[..2],
        ["     6 *-* x = 1/0", "     2 *-* call r"],
        "{stderr}"
    );
    assert!(
        lines[2].starts_with("Error 42 running /propagate.rex line 2:"),
        "{stderr}"
    );
}

/// spec: instr.raise.propagate.traceback
///
/// A propagated condition a `SIGNAL ON` trap takes is reported by nothing: a
/// later untrapped condition is reported from its own clause, without the
/// earlier condition's `TRACEBACK`.
#[test]
fn a_trapped_propagated_condition_leaves_no_traceback_for_a_later_report() {
    let outcome = run_program(
        "/propagate.rex",
        b"signal on syntax name h\n\
          call r\n\
          h:\n\
          say 1 + 'a'\n\
          ::routine r\n\
          \x20 signal on syntax name s\n\
          \x20 x = 1/0\n\
          s:\n\
          \x20 raise propagate\n"
            .to_vec(),
        Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 215);
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines[0], "     4 *-* say 1 + 'a'", "{stderr}");
    assert!(
        lines[1].starts_with("Error 41 running /propagate.rex line 4:"),
        "{stderr}"
    );
}

/// spec: instr.raise.propagate
///
/// A HALT that an extension routine raised, trapped and propagated by the
/// routine that called it, is reported at the line of the caller's clause:
/// the extension's level, which names its package and no line while it is
/// the failing one, is a level already left once the condition is raised
/// again. The routine loads the oracle checkout's `liborxfunction.so`.
#[test]
fn a_propagated_extension_condition_names_the_callers_line() {
    let lib_dir = Path::new(env!("REXX_ORACLE_ROOT")).join("lib");
    let library = lib_dir.join("liborxfunction.so");
    assert!(
        library.is_file(),
        "the oracle's {} is missing, which this test loads",
        library.display()
    );
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("propagate-extension-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("a probe directory of this run's own");
    let program = dir.join("main.rex");
    fs::write(
        &program,
        "call r\n\
         say 'not reached'\n\
         exit\n\
         ::routine r\n\
         \x20 signal on syntax\n\
         \x20 call tc 'HALT', 'stop'\n\
         \x20 return\n\
         syntax:\n\
         \x20 raise propagate\n\
         ::routine tc external \"LIBRARY orxfunction TestThrowCondition\"\n",
    )
    .expect("the program is writable");
    let output = Command::new(env!("CARGO_BIN_EXE_rexx"))
        .arg(&program)
        .current_dir(&dir)
        .env("LD_LIBRARY_PATH", &lib_dir)
        .stdin(Stdio::null())
        .output()
        .expect("the run spawns");
    assert_eq!(output.status.code(), Some(252));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(
        lines[1..3],
        ["     6 *-* call tc 'HALT', 'stop'", "     1 *-* call r"],
        "{stderr}"
    );
    let major = format!("Error 4 running {} line 1:", program.display());
    assert!(lines[3].starts_with(&major), "{stderr}");
}
