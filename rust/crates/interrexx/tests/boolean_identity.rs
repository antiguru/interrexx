//! A boolean that a built-in answers is the handle of `.true` or `.false`
//! (#599, ruling D15), as the oracle's is. Identity at the Rexx level is
//! unspecified (`spec/src/deviations.md`, Deviation 4); the table is this
//! crate's own consistency check, recorded from the oracle.
//!
//! * `eval`: each input line is an expression; the output is one line per
//!   input: `true` where the value's `identityHash` is that of `.true`,
//!   `false` likewise for `.false`, `other` for anything else (a number, say),
//!   or `ERR code` for the SYNTAX condition it raises.
//! * `eval_regexp`: as `eval`, with the crate's own `rxregexp.cls` required
//!   (the oracle runs it too, over its `rxregexp` library).
//! * `program`: the input is the whole program; the case records rc, stdout
//!   and stderr.
//!
//! Every program runs as its own `rexx` process with an API service of its
//! own. Recording: `BOOLEAN_IDENTITY_RECORD=1 REWRITE=1` runs the cases on the
//! oracle (`REXX_ORACLE_ROOT`), each with an `XDG_RUNTIME_DIR` of its own; the
//! `rxapi` daemons it starts are left running and are the recorder's to stop.

mod support;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use support::rexx_service;

const CASES: &str = "tests/boolean_identity";

fn scratch() -> PathBuf {
    rexx_service::scratch("boolean_identity")
}

/// The program an `eval` case runs.
fn eval_program(lines: &[&str], requires: Option<&str>) -> String {
    let mut program = String::new();
    for (index, line) in lines.iter().enumerate() {
        program.push_str(&format!(
            "signal on syntax name s{index}\nv = {line}\ncall show v\nsignal n{index}\n\
             s{index}: say 'ERR' condition('o')~code\nn{index}: nop\n"
        ));
    }
    program.push_str(
        "exit\nshow: procedure\n  use arg v\n  h = v~identityHash\n  \
         if h == .true~identityHash then say 'true'\n  \
         else if h == .false~identityHash then say 'false'\n  else say 'other'\n  return\n",
    );
    if let Some(package) = requires {
        program.push_str(&format!("::requires '{package}'\n"));
    }
    program
}

/// Runs `program` from a fresh directory, on the oracle where `oracle`.
fn run(name: usize, program: &str, oracle: bool) -> (i32, String, String) {
    let root = scratch();
    let dir = root.join(format!("case{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a case directory");
    let path = dir.join("probe.rex");
    std::fs::write(&path, program).expect("the program");
    if program.contains("rxregexp.cls") {
        let package = Path::new(env!("CARGO_MANIFEST_DIR")).join("packages/rxregexp.cls");
        std::fs::copy(package, dir.join("rxregexp.cls")).expect("the package");
    }
    let api = rexx_service::service_dir(&root, name);
    let mut command = rexx_service::with_service(
        Command::new(if oracle {
            rexx_service::oracle_rexx()
        } else {
            rexx_service::own_rexx().to_path_buf()
        }),
        oracle,
        &api,
    );
    let output = command
        .arg("probe.rex")
        .current_dir(&dir)
        .stdin(Stdio::null())
        .output()
        .expect("rexx runs");
    if !oracle {
        rexx_service::stop_own_service(&api);
    }
    let _ = std::fs::remove_dir_all(&dir);
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn built_in_booleans_are_the_handles_the_oracle_answers() {
    let record = std::env::var_os("BOOLEAN_IDENTITY_RECORD").is_some();
    let mut cases = 0;
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            cases += 1;
            let lines: Vec<&str> = case.input.lines().collect();
            match case.directive.as_str() {
                "eval" | "eval_regexp" => {
                    let requires = (case.directive == "eval_regexp").then_some("rxregexp.cls");
                    let (rc, stdout, stderr) = run(cases, &eval_program(&lines, requires), record);
                    assert_eq!(rc, 0, "{stderr}");
                    stdout
                }
                "program" => {
                    let (rc, stdout, stderr) =
                        run(cases, &format!("{}\n", case.input.trim_end()), record);
                    format!("rc {rc}\n[stdout]\n{stdout}[stderr]\n{stderr}")
                }
                other => panic!("unknown directive {other}"),
            }
        });
    });
    assert!(cases > 0, "no case ran");
}
