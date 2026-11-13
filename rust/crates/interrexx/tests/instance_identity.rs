//! **One interpreter instance for every activity** (`r[native.threads.instance]`). The extension
//! `tests/instance_identity/inst.cpp`, compiled with `g++` against this worktree's `api/` into a
//! directory of this run's own, answers the instance its thread context links and the application
//! data that instance carries. `inst.rex` asks it from the main activity, from the activity
//! `REPLY` makes and from one `start` makes, and prints `same` where all three answers are the
//! main activity's. Under `REXX_CORPUS_GATE` the oracle runs the same program and prints the same.
//! Built as a program, the fixture creates an interpreter with `APPLICATION_DATA` set and runs
//! `inst.rex` through it, so the data reaches every activity of an embedding.
//!
//! **The members that act on the whole interpreter** (`r[native.threads.halt]`,
//! `r[native.threads.trace]`). The extension `tests/instance_identity/halt_trace.cpp` keeps the
//! instance and calls `Halt` and `SetTrace` through it from a thread of its own; the programs
//! beside it are run under `rexx` and, under `REXX_CORPUS_GATE`, under the oracle, which must print
//! the same. Built as a program, it calls the members after the interpreter has ended.

#![cfg(target_os = "linux")]

mod support;

#[path = "../../interrexx-capi-rxapi/tests/support/cdylibs.rs"]
mod cdylibs;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The `memcap` limit each run is under.
const MEMORY: &str = "2G";

const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    std::env::var(GATE_ENV).is_ok_and(|value| !value.is_empty() && value != "0")
}

fn worktree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/instance_identity")
        .join(name)
}

/// A fresh directory of this run's own, named for `name`.
fn fresh_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a directory of this run's own");
    dir
}

/// `g++` compiling the fixture `source` against this worktree's `api/`, with `arguments` after the
/// source.
fn compile(source: &str, arguments: &[&str]) {
    let api = worktree().join("third_party/ooRexx/api");
    let built = Command::new("g++")
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(fixture(source))
        .args(arguments)
        .status()
        .expect("g++ runs, which this test needs to build its fixture");
    assert!(built.success(), "the fixture did not build: {arguments:?}");
}

/// Runs `program` with `argument` under `binary`, or `binary` alone where `program` is `None`,
/// memory-capped by `memcap`, from `dir`, with `dir` on the library path, answering stdout, stderr
/// and the exit status.
fn run(
    binary: &Path,
    dir: &Path,
    program: Option<&Path>,
    argument: Option<&str>,
) -> (String, String, i32) {
    let output = Command::new("memcap")
        .arg(MEMORY)
        .arg(binary)
        .args(program)
        .args(argument)
        .current_dir(dir)
        .env("LD_LIBRARY_PATH", dir)
        .env("REXX_API_DIR", dir.join("api"))
        .stdin(Stdio::null())
        .output()
        .expect("memcap runs, which every run of this test is under");
    let _ = Command::new(Path::new(env!("CARGO_BIN_EXE_rexx")).with_file_name("rxapi"))
        .arg("--stop")
        .env("REXX_API_DIR", dir.join("api"))
        .output();
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
    )
}

#[test]
fn one_instance_for_every_activity() {
    let dir = fresh_dir("instance-identity");
    compile(
        "inst.cpp",
        &[
            "-shared",
            "-fPIC",
            "-O1",
            "-static-libstdc++",
            "-o",
            &dir.join("libinst.so").to_string_lossy(),
        ],
    );
    let program = fixture("inst.rex");
    assert_eq!(
        run(
            Path::new(env!("CARGO_BIN_EXE_rexx")),
            &dir,
            Some(&program),
            None
        ),
        ("same\n".to_owned(), String::new(), 0)
    );
    if gate_mode() {
        let oracle = support::oracle::locate();
        let library_path = format!("{}:{}", oracle.lib_dir().display(), dir.display());
        let theirs = oracle.run_in(&program, &dir, &[("LD_LIBRARY_PATH", &library_path)]);
        assert_eq!(
            (
                String::from_utf8_lossy(&theirs.stdout).into_owned(),
                String::from_utf8_lossy(&theirs.stderr).into_owned(),
                theirs.expect_exit_code(),
            ),
            ("same\n".to_owned(), String::new(), 0),
            "the oracle"
        );
    }
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}

#[test]
fn application_data_reaches_a_started_activity() {
    let dir = fresh_dir("instance-application-data");
    // This workspace's `librexx.so.4`, under the name the driver links and the name it loads.
    for library in [cdylibs::librexx(), cdylibs::librexxapi()] {
        let name = library.file_name().expect("a library name").to_owned();
        let mut versioned = name.clone();
        versioned.push(".4");
        fs::copy(&library, dir.join(&versioned)).expect("a copy of the library");
        std::os::unix::fs::symlink(&versioned, dir.join(&name)).expect("the link name");
    }
    let driver = dir.join("instdriver");
    compile(
        "inst.cpp",
        &[
            "-DDRIVER",
            "-pthread",
            "-o",
            &driver.to_string_lossy(),
            &format!("-L{}", dir.display()),
            &format!("-Wl,-rpath,{}", dir.display()),
            "-lrexx",
            "-lrexxapi",
        ],
    );
    assert_eq!(
        run(&driver, &dir, Some(&fixture("inst.rex")), None),
        ("same\ndata set\n".to_owned(), String::new(), 0)
    );
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}

/// What a run answered: stdout, stderr and the exit status.
type Answer = (String, String, i32);

/// Builds `halt_trace.cpp` as the extension `halttrace` into a fresh directory named for `name`
/// and runs the fixture program `program` from it under `rexx`, answering `view` of its stdout,
/// stderr and exit status. Under `REXX_CORPUS_GATE` the oracle runs the same program from the
/// same directory, and `view` of its answer must be the same.
fn run_halt_trace(name: &str, program: &str, view: fn(Answer) -> Answer) -> Answer {
    let dir = fresh_dir(name);
    compile(
        "halt_trace.cpp",
        &[
            "-shared",
            "-fPIC",
            "-O1",
            "-static-libstdc++",
            "-pthread",
            "-o",
            &dir.join("libhalttrace.so").to_string_lossy(),
        ],
    );
    let program = fixture(program);
    let ours = view(run(
        Path::new(env!("CARGO_BIN_EXE_rexx")),
        &dir,
        Some(&program),
        None,
    ));
    if gate_mode() {
        let oracle = support::oracle::locate();
        let library_path = format!("{}:{}", oracle.lib_dir().display(), dir.display());
        let theirs = oracle.run_in(&program, &dir, &[("LD_LIBRARY_PATH", &library_path)]);
        assert_eq!(
            view((
                String::from_utf8_lossy(&theirs.stdout).into_owned(),
                String::from_utf8_lossy(&theirs.stderr).into_owned(),
                theirs.expect_exit_code(),
            )),
            ours,
            "the oracle"
        );
    }
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
    ours
}

/// An answer as it is.
fn whole(answer: Answer) -> Answer {
    answer
}

/// The trace lines of `stderr` with the program's path, which the first of them names, replaced
/// by `PROGRAM`.
fn traced(stderr: &str, program: &str) -> String {
    stderr.replace(&fixture(program).to_string_lossy().into_owned(), "PROGRAM")
}

/// `Halt` through the instance from a thread that runs no Rexx code reaches every activity of a
/// program `rexx` runs: the main activity, which a native call has left to wait for that thread,
/// and the activity `start` made.
// spec: native.threads.halt
#[test]
fn halt_through_the_instance_reaches_a_started_activity() {
    assert_eq!(
        run_halt_trace("halt-instance", "halt.rex", whole),
        ("main HALT / worker halted\n".to_owned(), String::new(), 0)
    );
}

/// `SetTrace` through the instance from a thread that runs no Rexx code traces the activity
/// `start` made, which is waiting when it is called. Only the worker's clauses after the wait are
/// compared: the main activity's lines interleave with them in no fixed order.
// spec: native.threads.trace
#[test]
fn set_trace_through_the_instance_traces_a_started_activity() {
    fn worker((stdout, stderr, code): Answer) -> Answer {
        let lines: String = stderr
            .lines()
            .filter(|line| line.contains("*-* say 'worker ran'") || line.contains("*-* done~post"))
            .map(|line| format!("{line}\n"))
            .collect();
        (stdout, lines, code)
    }
    assert_eq!(
        run_halt_trace("trace-started", "trace_started.rex", worker),
        (
            "worker ran\n".to_owned(),
            "    24 *-* say 'worker ran'\n    25 *-* done~post\n".to_owned(),
            0
        )
    );
}

/// `SetTrace(1)` traces the main activity from the clause after the call, with the interactive
/// pause of `TRACE ?R`, and `SetTrace(0)` ends it: the second call of `sub` prints no line.
// spec: native.threads.trace
#[test]
fn set_trace_on_and_off_through_the_instance_traces_the_main_activity() {
    let (stdout, stderr, code) = run_halt_trace("trace-main", "trace_main.rex", whole);
    assert_eq!((stdout.as_str(), code), ("end\n", 0), "{stderr}");
    assert_eq!(
        traced(&stderr, "trace_main.rex"),
        "       +++ \"LINUX COMMAND PROGRAM\"\n\
         \x20    5 *-* call sub\n\
         \x20   10 *-*   sub:\n\
         +++ Interactive trace. \"Trace Off\" to end debug, ENTER to continue. +++\n\
         \x20   11 *-*   x = 1 + 2\n\
         \x20      >>>     \"3\"\n\
         \x20   12 *-*   return\n\
         +++ Interactive trace. \"Trace Off\" to end debug, ENTER to continue. +++\n\
         \x20    6 *-* call httrace 0\n\
         \x20      >>>   \"0\"\n"
    );
}

/// A body run before the extension is loaded, and so compiled before any `SetTrace`, is traced
/// when it is entered again after one: the library is loaded part-way through the run, the first
/// `SetTrace` turns tracing off, and the second, inside `sub`, traces the rest of `sub`.
// spec: native.threads.trace
#[test]
fn set_trace_traces_a_body_compiled_before_the_extension_was_loaded() {
    let (stdout, stderr, code) = run_halt_trace("trace-load", "trace_load.rex", whole);
    assert_eq!((stdout.as_str(), code), ("end\n", 0), "{stderr}");
    assert_eq!(
        traced(&stderr, "trace_load.rex"),
        "       +++ \"LINUX COMMAND PROGRAM\"\n\
         \x20   13 *-*   x = 1 + 2\n\
         \x20      >>>     \"3\"\n\
         +++ Interactive trace. \"Trace Off\" to end debug, ENTER to continue. +++\n\
         \x20   14 *-*   return\n"
    );
}

/// `Halt` and `SetTrace` through the instance of an interpreter that has ended do nothing and
/// return: the driver creates an interpreter, terminates it, calls the three and prints
/// `returned`. Not compared with the oracle: a member called through an ended instance is
/// outside what native code may do (`r[native.threads.instance]`).
// spec: native.threads.trace
#[test]
fn halt_and_set_trace_after_the_interpreter_ended_return() {
    let dir = fresh_dir("halt-trace-ended");
    for library in [cdylibs::librexx(), cdylibs::librexxapi()] {
        let name = library.file_name().expect("a library name").to_owned();
        let mut versioned = name.clone();
        versioned.push(".4");
        fs::copy(&library, dir.join(&versioned)).expect("a copy of the library");
        std::os::unix::fs::symlink(&versioned, dir.join(&name)).expect("the link name");
    }
    let driver = dir.join("htdriver");
    compile(
        "halt_trace.cpp",
        &[
            "-DDRIVER",
            "-pthread",
            "-o",
            &driver.to_string_lossy(),
            &format!("-L{}", dir.display()),
            &format!("-Wl,-rpath,{}", dir.display()),
            "-lrexx",
            "-lrexxapi",
        ],
    );
    assert_eq!(
        run(&driver, &dir, None, None),
        ("returned\n".to_owned(), String::new(), 0)
    );
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}
