//! Command environments written in Rexx (`.CommandHandler`,
//! `interpreter/RexxClasses/CommandClasses.orx`) on `rexx`: ooTest's own
//! group for them, held to the oracle's counts, and a program for what the
//! group does not reach, held to the oracle's output. Each run has a private
//! `REXX_API_DIR` and stops its daemon after.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const REXX: &str = env!("CARGO_BIN_EXE_rexx");

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

/// This test's directory `name` under cargo's target, emptied first.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("command-handler")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

/// `rexx` running `arguments` in `cwd` with a service of its own, which is
/// stopped after; `library_path`, where given, is the run's library path.
fn run_rexx(arguments: &[&str], cwd: &Path, library_path: Option<&Path>) -> Output {
    let api = cwd.join("api");
    std::fs::create_dir_all(&api).expect("the service directory");
    let bin = Path::new(REXX).parent().expect("rexx has a directory");
    let path = std::env::join_paths(std::iter::once(bin.to_path_buf()).chain(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
    ))
    .expect("a PATH");
    let mut command = Command::new("timeout");
    command
        .args(["-k", "5", "300"])
        .arg(REXX)
        .args(arguments)
        .current_dir(cwd)
        .env("REXX_API_DIR", &api)
        .env("PATH", path)
        .env_remove("LD_LIBRARY_PATH")
        .env_remove("REXX_SWITCH_MODE")
        .stdin(Stdio::null());
    if let Some(library_path) = library_path {
        command.env("LD_LIBRARY_PATH", library_path);
    }
    let output = command.output().expect("rexx runs");
    let _ = Command::new(bin.join("rxapi"))
        .arg("--stop")
        .env("REXX_API_DIR", &api)
        .output();
    output
}

fn copy(from: &Path, to: &Path) {
    let status = Command::new("cp")
        .arg("-r")
        .arg(from)
        .arg(to)
        .status()
        .expect("cp runs");
    assert!(status.success(), "cp {} {}", from.display(), to.display());
}

/// The number a summary line `label` reports.
fn count(stdout: &str, label: &str) -> usize {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix(label))
        .and_then(|count| count.trim().parse().ok())
        .unwrap_or_else(|| panic!("no {label} line:\n{stdout}"))
}

/// `base/class/CommandHandler.testGroup` through `testOORexx.rex`: the
/// oracle runs 68 tests making 150 assertions, none failing.
#[test]
fn the_command_handler_group_passes_with_the_oracles_counts() {
    let run = scratch("group");
    let ootest = repository().join("ootest");
    let dir = run.join("ooRexx/base/class");
    std::fs::create_dir_all(&dir).expect("the group's directory");
    copy(&ootest.join("framework"), &run.join("framework"));
    copy(
        &ootest.join("ooRexx/base/class/CommandHandler.testGroup"),
        &dir,
    );
    for file in ["testOORexx.rex", "worker.rex", "ooTest.frm"] {
        copy(&ootest.join(file), &run);
    }
    // `ooTest.frm` requires `rxregexp.cls`, which this crate ships and runs:
    // nothing is copied or put on a library path for it.
    let output = run_rexx(
        &[
            "testOORexx.rex",
            "-f",
            "ooRexx/base/class/CommandHandler.testGroup",
            "-U",
            "-V",
            "2",
        ],
        &run,
        None,
    );
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        stdout.contains("Tests ran:"),
        "no summary; exit status {:?}:\n{stdout}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let counts = [
        count(&stdout, "Tests ran:"),
        count(&stdout, "Assertions:"),
        count(&stdout, "Failures:"),
        count(&stdout, "Errors:"),
    ];
    assert_eq!(
        counts,
        [68, 150, 0, 0],
        "tests, assertions, failures, errors:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0), "the exit status");
}

/// What the group does not reach, measured against the oracle: a native
/// handler's object (`kind`, `name`, one object per handler), a command a
/// handler written in Rexx forwards to the native one it covers with the
/// caller's `ADDRESS ... WITH` output, error and input, the forwarded
/// command's `ERROR`, and the natives' refusals.
#[test]
fn a_forwarded_command_keeps_the_callers_redirection() {
    const PROGRAM: &str = r#"native = .CommandHandler~handler("sh")
say native~kind native~name (native == .CommandHandler~environments["SH"])
.CommandHandler~register("sh", .Forward~new(native))
address sh "echo out; echo err 1>&2" with output stem o. error stem e.
say rc o.0 o.1 e.0 e.1
in.1 = "x y"; in.0 = 1
address sh "cat" with input stem in. output stem o.
say rc o.0 o.1
call on error name got
address sh "exit 5"
say "after" rc
say (.CommandHandler~deregister("sh") \== native) (.CommandHandler~handler("sh") == native)
.CommandHandler~register("z", .Bad~new)
address z "a"
say "z" rc
call refused .message~new(.CommandHandler, "DEREGISTER", "I", "nosuchname")
call refused .message~new(.CommandContext~new(.object~new, "z"), "GETVAR", "I", "a")
exit
got:
  say "ERROR" rc condition("D")
  return
refused: procedure
  use arg msg
  signal on syntax name bad
  msg~send
  say "no error"
  return
bad:
  say condition("O")~code condition("O")~message
  return
::class Forward inherit CommandHandler
::method init
  expose below
  use arg below
::method command
  expose below
  use arg command, caller
  return below~command(command, caller)
::class Bad inherit CommandHandler
::method command
  use arg c, ctx
  signal on syntax name x
  return ctx~isRedirected("zz")
x: return condition("O")~code condition("O")~message
"#;
    // Measured, oracle, exit status 0 and nothing on stderr.
    const EXPECTED: &str = "REDIRECTING SH 1\n\
                            0 1 out 1 err\n\
                            0 1 x y\n\
                            ERROR 5 exit 5\n\
                            after 5\n\
                            1 1\n\
                            z 93.900 isRedirected: the argument must be Input, Output, Error or Same.\n\
                            93.900 deregister: there is no command environment named nosuchname.\n\
                            98.900 this CommandContext belongs to a command that has ended.\n";
    let run = scratch("forward");
    std::fs::write(run.join("forward.rex"), PROGRAM).expect("the program");
    let output = run_rexx(&["forward.rex"], &run, None);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        EXPECTED,
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0), "the exit status");
}

/// A `.CommandHandler` native sent without a required argument raises 88.901, numbered and at
/// the sending clause's line, as the oracle's native signature check does; ours raised 93.903.
#[test]
fn a_missing_native_argument_raises_88_901() {
    const PROGRAM: &str = r#"call refused .message~new(.CommandHandler, "HANDLER")
call refused .message~new(.CommandHandler, "HANDLER", "A", .array~new(1))
.CommandHandler~handler
exit
refused: procedure
  use arg msg
  signal on syntax name bad
  msg~send
  say "no error"
  return
bad:
  say condition("O")~code condition("O")~message
  return
"#;
    // Measured, oracle, exit status 168.
    const STDOUT: &str = "88.901 Missing argument; argument 1 is required.\n\
                          88.901 Missing argument; argument 1 is required.\n";
    const STDERR: &str = "       *-* Compiled method \"HANDLER\" with scope \"CommandHandler\".\n\
                          \x20    3 *-* .CommandHandler~handler\n\
                          Error 88 running {path} line 3:  Invalid argument.\n\
                          Error 88.901:  Missing argument; argument 1 is required.\n";
    let run = scratch("missing");
    std::fs::write(run.join("missing.rex"), PROGRAM).expect("the program");
    let output = run_rexx(&["missing.rex"], &run, None);
    let path = run.join("missing.rex");
    assert_eq!(output.status.code(), Some(168), "{output:?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), STDOUT);
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        STDERR.replace("{path}", &path.display().to_string())
    );
}
