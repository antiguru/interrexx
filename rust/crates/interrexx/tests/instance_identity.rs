//! **One interpreter instance for every activity** (`r[native.threads.instance]`). The extension
//! `tests/instance_identity/inst.cpp`, compiled with `g++` against this worktree's `include/` into a
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

/// The address-space limit in KiB where `memcap` is not on `PATH`.
const FALLBACK_LIMIT_KIB: u64 = 4_194_304;

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

/// A fresh directory of this run's own, named for `name`. The API service's directory is not in
/// it ([`support::api_socket::api_dir`]), so the socket path does not depend on the checkout.
fn fresh_dir(name: &str) -> PathBuf {
    let dir =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{:x}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a directory of this run's own");
    dir
}

/// Whether `name` is an executable file on this process's `PATH`.
fn on_path(name: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(name).is_file()))
}

/// `g++` compiling the fixture `source` against this worktree's `include/`, with `arguments` after
/// the source.
fn compile(source: &str, arguments: &[&str]) {
    let api = worktree().join("include");
    let built = Command::new("g++")
        .args(["-Wall", "-Werror"])
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(fixture(source))
        .args(arguments)
        .status()
        .expect("g++ runs, which this test needs to build its fixture");
    assert!(built.success(), "the fixture did not build: {arguments:?}");
}

/// Runs `program` with `argument` under `binary`, or `binary` alone where `program` is `None`,
/// memory-capped by `memcap`, or by an address-space limit where `memcap` is not on `PATH`, from
/// `dir`, with `dir` on the library path, answering stdout, stderr and the exit status.
fn run(
    binary: &Path,
    dir: &Path,
    program: Option<&Path>,
    argument: Option<&str>,
) -> (String, String, i32) {
    let mut command = if on_path("memcap") {
        let mut command = Command::new("memcap");
        command.arg(MEMORY);
        command
    } else {
        let mut command = Command::new("bash");
        command.arg("-c").arg(format!(
            "ulimit -v {FALLBACK_LIMIT_KIB} && exec \"$0\" \"$@\""
        ));
        command
    };
    let output = command
        .arg(binary)
        .args(program)
        .args(argument)
        .current_dir(dir)
        .env("LD_LIBRARY_PATH", dir)
        .env("REXX_API_DIR", support::api_socket::api_dir(dir))
        .stdin(Stdio::null())
        .output()
        .expect("the capped run spawns");
    let _ = Command::new(Path::new(env!("CARGO_BIN_EXE_rexx")).with_file_name("rxapi"))
        .arg("--stop")
        .env("REXX_API_DIR", support::api_socket::api_dir(dir))
        .output();
    support::api_socket::end_api_dir(dir);
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

/// Builds `halt_trace.cpp` as the extension `halttrace` into a fresh directory named for `name`,
/// answering that directory.
fn build_halt_trace(name: &str) -> PathBuf {
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
    dir
}

/// Builds `halt_trace.cpp` as the extension `halttrace` into a fresh directory named for `name`
/// and runs the fixture program `program` from it under `rexx`, answering `view` of its stdout,
/// stderr and exit status. Under `REXX_CORPUS_GATE` the oracle runs the same program from the
/// same directory, and `view` of its answer must be the same, with the message texts of both
/// sides' stderr cut ([`support::error_text::cut_error_texts`]).
fn run_halt_trace(name: &str, program: &str, view: fn(Answer) -> Answer) -> Answer {
    let dir = build_halt_trace(name);
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
        let cut = |stderr: &[u8]| {
            String::from_utf8_lossy(&support::error_text::cut_error_texts(stderr)).into_owned()
        };
        let theirs = view((
            String::from_utf8_lossy(&theirs.stdout).into_owned(),
            String::from_utf8_lossy(&theirs.stderr).into_owned(),
            theirs.expect_exit_code(),
        ));
        assert_eq!(
            (theirs.0, cut(theirs.1.as_bytes()), theirs.2),
            (ours.0.clone(), cut(ours.1.as_bytes()), ours.2),
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

/// An answer with only the `>K>` lines of its stderr.
fn keyword_lines((stdout, stderr, code): Answer) -> Answer {
    let lines: String = stderr
        .lines()
        .filter(|line| line.contains(">K>"))
        .map(|line| format!("{line}\n"))
        .collect();
    (stdout, lines, code)
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
         +++ Interactive tracing is on; TRACE OFF ends it and an empty line continues.\n\
         \x20   11 *-*   x = 1 + 2\n\
         \x20      >>>     \"3\"\n\
         \x20   12 *-*   return\n\
         +++ Interactive tracing is on; TRACE OFF ends it and an empty line continues.\n\
         \x20    6 *-* call httrace 0\n\
         \x20      >>>   \"0\"\n"
    );
}

/// A `::ROUTINE` run before the extension is loaded, and so compiled before any `SetTrace`, prints
/// every trace line when it is entered again after one: the library is loaded part-way through the
/// run, the first `SetTrace` turns tracing off, and the second, inside `r`, traces the rest of `r`,
/// the `>K>` lines of its `DO` header among them. A body compiled before the first `SetTrace`
/// would leave those out, which is what this witnesses being compiled again.
// spec: native.threads.trace, native.threads.trace.deviation.166
#[test]
fn set_trace_traces_a_body_compiled_before_the_extension_was_loaded() {
    let (stdout, stderr, code) = run_halt_trace("trace-load", "trace_load.rex", whole);
    assert_eq!((stdout.as_str(), code), ("end\n", 0), "{stderr}");
    assert_eq!(
        traced(&stderr, "trace_load.rex"),
        "       +++ \"LINUX SUBROUTINE PROGRAM\"\n\
         \x20   14 *-* do i = 1 to 2 by 1 for 2\n\
         \x20      >K>   \"TO\" => \"2\"\n\
         \x20      >K>   \"BY\" => \"1\"\n\
         \x20      >K>   \"FOR\" => \"2\"\n\
         +++ Interactive tracing is on; TRACE OFF ends it and an empty line continues.\n\
         \x20   15 *-*   nop\n\
         \x20   16 *-* end\n\
         \x20   14 *-* do i = 1 to 2 by 1 for 2\n\
         \x20      >>>     \"1\"\n\
         \x20      >>>     \"2\"\n\
         \x20   15 *-*   nop\n\
         \x20   16 *-* end\n\
         \x20   14 *-* do i = 1 to 2 by 1 for 2\n\
         \x20      >>>     \"2\"\n\
         \x20      >>>     \"3\"\n\
         \x20   17 *-* if arg(1) \n\
         \x20      >>>   \"1\"\n\
         \x20   17 *-*   then\n\
         \x20   17 *-*     call httrace 0\n\
         \x20      >>>       \"0\"\n"
    );
}

/// The `::ROUTINE` `r` is running when the first `SetTrace` is served, so it runs on with the body
/// compiled before it, which leaves out the `>K>` lines of its `DO` header for `TO`, `BY` and
/// `FOR` and prints the one for `WHILE`. Not compared with the oracle, which prints all four: this
/// pins the difference the Deviation declares.
// spec: native.threads.trace.deviation.166
#[test]
fn set_trace_leaves_header_lines_out_of_a_routine_already_running() {
    let dir = build_halt_trace("trace-running");
    let (stdout, stderr, code) = run(
        Path::new(env!("CARGO_BIN_EXE_rexx")),
        &dir,
        Some(&fixture("trace_running.rex")),
        None,
    );
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
    assert_eq!((stdout.as_str(), code), ("end\n", 0), "{stderr}");
    assert_eq!(
        traced(&stderr, "trace_running.rex"),
        "       +++ \"LINUX SUBROUTINE PROGRAM\"\n\
         \x20   12 *-* do i = 1 to 2 by 1 for 2 while i < 5\n\
         \x20      >K>     \"WHILE\" => \"1\"\n\
         +++ Interactive tracing is on; TRACE OFF ends it and an empty line continues.\n\
         \x20   13 *-*   nop\n\
         \x20   14 *-* end\n\
         \x20   12 *-* do i = 1 to 2 by 1 for 2 while i < 5\n\
         \x20      >>>     \"1\"\n\
         \x20      >>>     \"2\"\n\
         \x20      >K>     \"WHILE\" => \"1\"\n\
         \x20   13 *-*   nop\n\
         \x20   14 *-* end\n\
         \x20   12 *-* do i = 1 to 2 by 1 for 2 while i < 5\n\
         \x20      >>>     \"2\"\n\
         \x20      >>>     \"3\"\n\
         \x20   15 *-* call httrace 0\n\
         \x20      >>>   \"0\"\n"
    );
}

/// `SetThreadTrace`, the thread context's own member, switches the trace as `SetTrace` does: its
/// first call, from the main program, comes before `r` is entered, so `r`, which turns its trace
/// on with a second call, prints every line, the `>K>` lines of its `DO` header among them. Only
/// the `>K>` lines are compared: the `>>>` line of each call's own clause is printed under a
/// different setting on the two sides.
// spec: native.threads.trace.deviation.166
#[test]
fn set_thread_trace_traces_a_routine_entered_after_its_first_call() {
    assert_eq!(
        run_halt_trace("trace-thread", "trace_thread.rex", keyword_lines),
        (
            "end\n".to_owned(),
            "       >K>   \"TO\" => \"2\"\n\
             \x20      >K>   \"BY\" => \"1\"\n\
             \x20      >K>   \"FOR\" => \"2\"\n"
                .to_owned(),
            0
        )
    );
}

/// The first `SetTrace` is served in the `::ROUTINE` `r` while the main program waits for it, and
/// an internal routine the main program calls afterwards prints every line: `sub` turns its trace
/// on, and the `>K>` lines of its `DO` header are printed. An internal routine runs in its caller's
/// body, and the chunk the caller was running when the first `SetTrace` was served leaves those
/// lines out. Only the `>K>` lines are compared, as for `SetThreadTrace`.
// spec: native.threads.trace, native.threads.trace.deviation.166
#[test]
fn set_trace_traces_an_internal_routine_entered_after_the_first() {
    assert_eq!(
        run_halt_trace("trace-internal", "trace_internal.rex", keyword_lines),
        (
            "end\n".to_owned(),
            "       >K>     \"TO\" => \"2\"\n\
             \x20      >K>     \"BY\" => \"1\"\n"
                .to_owned(),
            0
        )
    );
}

/// `SetTrace(1)` traces the main program, and a `::ROUTINE` it calls afterwards starts with its
/// own setting: none of its clauses is traced, and the main program's are again after it returns.
// spec: native.threads.trace
#[test]
fn set_trace_leaves_a_routine_called_afterwards_its_own_setting() {
    let (stdout, stderr, code) = run_halt_trace("trace-callee", "trace_callee.rex", whole);
    assert_eq!((stdout.as_str(), code), ("end\n", 0), "{stderr}");
    assert_eq!(
        traced(&stderr, "trace_callee.rex"),
        "       +++ \"LINUX COMMAND PROGRAM\"\n\
         \x20    6 *-* call r\n\
         +++ Interactive tracing is on; TRACE OFF ends it and an empty line continues.\n\
         \x20    7 *-* call httrace 0\n\
         \x20      >>>   \"0\"\n"
    );
}

/// `Halt` and `SetTrace` through the instance of an interpreter that has ended do nothing and
/// return: the driver creates an interpreter, terminates it, calls the three and prints
/// `returned`, as `r[native.threads.trace]` says a request made after the interpreter has ended
/// does nothing. Not compared with the oracle.
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
