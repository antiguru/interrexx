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
//!
//! **The lifetime of a library across instances** (`r[native.loading.shared]`,
//! `r[native.loading.unload]`, `r[native.loading.again]`,
//! `r[native.loading.reentrant]`). The extension
//! `tests/instance_identity/lifetime.cpp` prints `loader` from its loader and `unloader` from its
//! unloader. Built as a program, it creates instances that each load it and prints a line after
//! each creation and each `Terminate`. Under `REXX_CORPUS_GATE` the same program built against the
//! oracle's library prints the same, except in the scenarios where one instance is created while
//! another runs a hook, which the oracle does not wait for. The fixtures
//! `tests/instance_identity/crossload.cpp` and `tests/instance_identity/cycle.cpp` load libraries
//! from two instances at once, while a loader runs on another thread.

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

/// The seconds after which `timeout` kills a run, so that one that hangs fails.
const DEADLINE: &str = "120";

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
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(fixture(source))
        .args(arguments)
        .status()
        .expect("g++ runs, which this test needs to build its fixture");
    assert!(built.success(), "the fixture did not build: {arguments:?}");
}

/// A command that runs its arguments memory-capped by `memcap`, or by an address-space limit where
/// `memcap` is not on `PATH`, and killed by `timeout` after [`DEADLINE`].
fn capped() -> Command {
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
    command.args(["timeout", "-s", "KILL", DEADLINE]);
    command
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
    let output = capped()
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

/// Builds `lifetime.cpp` as the extension `lifetime` and as the driver, linked against this
/// workspace's `librexx.so.4`, into a fresh directory named for `name`, and runs the driver with
/// `scenario`. Under `REXX_CORPUS_GATE` the driver is built again against the oracle's library,
/// and must print the same.
fn run_lifetime(name: &str, scenario: &str) -> Answer {
    let mut runs = lifetime_runs(name, scenario, gate_mode()).into_iter();
    let ours = runs.next().expect("this workspace's run");
    if let Some(theirs) = runs.next() {
        assert_eq!(theirs, ours, "the oracle");
    }
    ours
}

/// [`run_lifetime`]'s runs: this workspace's, and where `oracle` is true the oracle's after it.
fn lifetime_runs(name: &str, scenario: &str, oracle: bool) -> Vec<Answer> {
    // The second library is the one the scenarios `other` and `unloaderother` load from a hook.
    driver_runs(
        name,
        "lifetime.cpp",
        &[(None, "lifetime"), (Some("-DOTHER"), "lifetime2")],
        scenario,
        oracle,
    )
}

/// Builds the fixture `source` once for each of `extensions`, with its macro where one is given,
/// as the extension it names, and as the driver, linked against this workspace's `librexx.so.4`,
/// into a fresh directory named for `name`. Runs the driver with `scenario`, and where `oracle` is
/// true runs it again built against the oracle's library. Answers the runs, this workspace's
/// first.
fn driver_runs(
    name: &str,
    source: &str,
    extensions: &[(Option<&str>, &str)],
    scenario: &str,
    oracle: bool,
) -> Vec<Answer> {
    let (dir, driver) = build_driver(name, source, extensions);
    let mut runs = vec![run(&driver, &dir, None, Some(scenario))];
    if oracle {
        // A slow hook's marks, which the oracle's run makes again.
        for mark in ["hook-started", "cyclefirst-started", "cyclesecond-started"] {
            let _ = fs::remove_file(dir.join(mark));
        }
        let oracle = support::oracle::locate();
        let theirs_driver = dir.join("ltdriver-oracle");
        link_driver(source, &theirs_driver, oracle.lib_dir());
        let output = capped()
            .arg(&theirs_driver)
            .arg(scenario)
            .current_dir(&dir)
            .env(
                "LD_LIBRARY_PATH",
                format!("{}:{}", oracle.lib_dir().display(), dir.display()),
            )
            .env("XDG_RUNTIME_DIR", support::oracle_runtime::dir())
            .stdin(Stdio::null())
            .output()
            .expect("the capped run spawns");
        // The oracle's run starts its `rxapi` inside the run's `memcap` group, so `memcap` may
        // report that it left the group behind. That line is the harness's, not the program's.
        let stderr: String = String::from_utf8_lossy(&output.stderr)
            .lines()
            .filter(|line| !(line.starts_with("memcap: left ") && line.ends_with(" exit")))
            .map(|line| format!("{line}\n"))
            .collect();
        runs.push((
            String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr,
            output.status.code().unwrap_or(-1),
        ));
    }
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
    runs
}

/// Builds what [`driver_runs`] runs: the fixture `source` as each of `extensions` and as the
/// driver linked against this workspace's `librexx.so.4`, into a fresh directory named for `name`.
/// Answers the directory and the driver.
fn build_driver(
    name: &str,
    source: &str,
    extensions: &[(Option<&str>, &str)],
) -> (PathBuf, PathBuf) {
    let dir = fresh_dir(name);
    for (define, extension) in extensions {
        let output = dir.join(format!("lib{extension}.so"));
        let mut arguments: Vec<&str> = define.iter().copied().collect();
        arguments.extend(["-shared", "-fPIC", "-O1", "-o"]);
        let output = output.to_string_lossy();
        arguments.push(&output);
        compile(source, &arguments);
    }
    for library in [cdylibs::librexx(), cdylibs::librexxapi()] {
        let name = library.file_name().expect("a library name").to_owned();
        let mut versioned = name.clone();
        versioned.push(".4");
        fs::copy(&library, dir.join(&versioned)).expect("a copy of the library");
        std::os::unix::fs::symlink(&versioned, dir.join(&name)).expect("the link name");
    }
    let driver = dir.join("ltdriver");
    link_driver(source, &driver, &dir);
    (dir, driver)
}

/// Builds the fixture `source` as the driver `driver`, linked against the `librexx` and
/// `librexxapi` in `libraries`.
fn link_driver(source: &str, driver: &Path, libraries: &Path) {
    compile(
        source,
        &[
            "-DDRIVER",
            "-pthread",
            "-o",
            &driver.to_string_lossy(),
            &format!("-L{}", libraries.display()),
            &format!("-Wl,-rpath,{}", libraries.display()),
            "-lrexx",
            "-lrexxapi",
        ],
    );
}

/// Checks that `stdout` is `first`, then `middle` in any order, then `last`, answering the lines
/// of `middle` in the order they came.
///
/// The scenarios that check this are not compared with the oracle. In `loading` and `unloading` it
/// creates the second instance while the first one's loader or unloader is still running, and in
/// `unloading` that instance then runs neither the loader nor the unloader. In the scenarios of
/// `crossload.cpp` and `cycle.cpp` it does not wait for a loader that another thread runs.
fn lines_in_order<'a>(
    stdout: &'a str,
    first: &[&str],
    middle: &[&str],
    last: &[&str],
) -> Vec<&'a str> {
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        first.len() + middle.len() + last.len(),
        "{stdout}"
    );
    let (head, rest) = lines.split_at(first.len());
    let (between, tail) = rest.split_at(middle.len());
    assert_eq!(head, first, "{stdout}");
    assert_eq!(tail, last, "{stdout}");
    let mut sorted = between.to_vec();
    sorted.sort_unstable();
    let mut expected = middle.to_vec();
    expected.sort_unstable();
    assert_eq!(sorted, expected, "{stdout}");
    between.to_vec()
}

/// Two instances that are live together and each load the library run its loader once, when the
/// first loads it.
// spec: native.loading.shared
#[test]
fn live_instances_share_one_load_of_a_library() {
    let (stdout, stderr, code) = run_lifetime("lifetime-shared", "two");
    assert_eq!((stderr.as_str(), code), ("", 0), "{stdout}");
    assert!(
        stdout.starts_with("loader\ncreated A\ncreated B\n"),
        "{stdout}"
    );
}

/// Terminating the first of two live instances runs no unloader; terminating the second, the
/// last live one, runs it once, whichever instance loaded the library.
// spec: native.loading.unload
#[test]
fn the_unloader_runs_once_when_the_last_instance_ends() {
    assert_eq!(
        run_lifetime("lifetime-unload", "two"),
        (
            "loader\ncreated A\ncreated B\nterminated A\nunloader\nterminated B\n".to_owned(),
            String::new(),
            0
        )
    );
}

/// An instance created after the last live one has ended loads the library again: its loader and
/// its unloader each run once more.
// spec: native.loading.again
#[test]
fn an_instance_after_the_last_one_loads_the_library_again() {
    assert_eq!(
        run_lifetime("lifetime-again", "three"),
        (
            "loader\ncreated A\ncreated B\nterminated A\nunloader\nterminated B\n\
             loader\ncreated C\nunloader\nterminated C\n"
                .to_owned(),
            String::new(),
            0
        )
    );
}

/// A thread that attached to the first instance and detached, after which no library of the
/// process is unmapped, leaves the loader and the unloader running as often: a later instance
/// still runs both again.
// spec: native.loading.again, native.loading.static
#[test]
fn an_instance_after_an_attached_thread_loads_the_library_again() {
    assert_eq!(
        run_lifetime("lifetime-attached", "attached"),
        (
            "loader\ncreated A\ncreated B\nterminated A\nunloader\nterminated B\n\
             loader\ncreated C\nunloader\nterminated C\n"
                .to_owned(),
            String::new(),
            0
        )
    );
}

/// An instance created while another instance's loader for a library is still running completes
/// its load of the library only once that loader has returned: the loader runs once, and both
/// creations come after its end.
// spec: native.loading.shared
#[test]
fn a_load_during_another_instances_loader_waits_for_it() {
    let (stdout, stderr, code) = lifetime_runs("lifetime-loading", "loading", false).remove(0);
    assert_eq!((stderr.as_str(), code), ("", 0), "{stdout}");
    lines_in_order(
        &stdout,
        &["loader start", "loader end"],
        &["created A", "created B"],
        &["terminated A", "unloader", "terminated B"],
    );
}

/// An instance created while the last instance's unloader is still running is created only once
/// that unloader has returned, so its loader runs after the unloader's end, and its own last end
/// runs the unloader again.
// spec: native.loading.again
#[test]
fn an_instance_created_during_the_last_unloader_waits_for_it() {
    let (stdout, stderr, code) = lifetime_runs("lifetime-unloading", "unloading", false).remove(0);
    assert_eq!((stderr.as_str(), code), ("", 0), "{stdout}");
    let middle = lines_in_order(
        &stdout,
        &["loader", "created A", "unloader start", "unloader end"],
        &["terminated A", "loader", "created B"],
        &["unloader start", "unloader end", "terminated B"],
    );
    let at = |line: &str| middle.iter().position(|held| *held == line);
    assert!(at("loader") < at("created B"), "{stdout}");
}

/// A loader that creates an instance loading the same library does not wait for itself: the
/// inner instance shares the load whose loader is running, and runs no loader of its own.
// spec: native.loading.shared
#[test]
fn an_instance_a_loader_creates_shares_the_load_in_progress() {
    assert_eq!(
        run_lifetime("lifetime-nested", "nested"),
        (
            "loader\ncreated inner\nterminated inner\ncreated A\nunloader\nterminated A\n"
                .to_owned(),
            String::new(),
            0
        )
    );
}

/// A loader that loads, through its thread context, the library whose loader it is finds it
/// loaded: the load answers true at once and runs no second loader.
// spec: native.loading.reentrant
#[test]
fn a_loader_loading_its_own_library_finds_it_loaded() {
    assert_eq!(
        run_lifetime("lifetime-reload", "reload"),
        (
            "loader\nreloaded 1\ncreated A\nunloader\nterminated A\n".to_owned(),
            String::new(),
            0
        )
    );
}

/// A loader that loads another library gets the load without waiting, and the other library's
/// loader runs inside it. Both unloaders run, in the order the libraries were asked for, when the
/// instance ends.
// spec: native.loading.reentrant
#[test]
fn a_loader_loading_another_library_runs_that_loader_inside_its_own() {
    assert_eq!(
        run_lifetime("lifetime-other", "other"),
        (
            "loader\nother loader\nother 1\ncreated A\nunloader\nother unloader\nterminated A\n"
                .to_owned(),
            String::new(),
            0
        )
    );
}

/// An unloader that creates an instance, which loads the library, does not wait for itself: the
/// instance is created and terminated inside the unloader. The terminating instance is still live
/// while its unloaders run, so the inner instance shares its load and runs no loader, and its
/// termination is not the last one and runs no unloader.
// spec: native.loading.reentrant
#[test]
fn an_instance_an_unloader_creates_shares_the_load_being_unloaded() {
    assert_eq!(
        run_lifetime("lifetime-unloadernew", "unloadernew"),
        (
            "loader\ncreated A\nunloader\ncreated inner\nterminated inner\nterminated A\n"
                .to_owned(),
            String::new(),
            0
        )
    );
}

/// An instance that an unloader creates and leaves live outlives the instance whose termination
/// ran the unloader, without the libraries whose unloaders that termination ran: its own
/// termination, now the last, runs no unloader of theirs again. An instance created afterwards
/// loads the library afresh.
// spec: native.loading.reentrant
#[test]
fn an_instance_an_unloader_leaves_live_does_not_unload_the_library_again() {
    assert_eq!(
        run_lifetime("lifetime-unloaderkeep", "unloaderkeep"),
        (
            "loader\ncreated A\nunloader\ncreated inner\nterminated A\nterminated inner\n\
             loader\ncreated C\nunloader\nterminated C\n"
                .to_owned(),
            String::new(),
            0
        )
    );
}

/// The instance an unloader left live loaded the library from the same file while its round's
/// unloaders ran, so the library closes with that round (`r[native.loading.again]`): attached to
/// after the termination that ran the unloader, the instance no longer runs the library's routine.
/// The oracle agrees under the gate.
// spec: native.loading.again
#[test]
fn an_instance_an_unloader_leaves_live_runs_no_routine_of_the_closed_library() {
    assert_eq!(
        run_lifetime("lifetime-unloaderprobe", "unloaderprobe"),
        (
            "loader\ncreated A\nunloader\ncreated inner\nterminated A\nprobe after -1\n\
             terminated inner\n"
                .to_owned(),
            String::new(),
            0
        )
    );
}

/// [`an_instance_an_unloader_leaves_live_runs_no_routine_of_the_closed_library`] with a command
/// to the environment the library's own handler serves in the instance the unloader left live:
/// the handler went with the library, so the environment is unknown, and the command's RC is 30
/// with its failure traced. The oracle's run of this crashes in the closed library, so it is not
/// asked.
// spec: native.loading.again
#[test]
fn an_instance_an_unloader_leaves_live_runs_no_handler_of_the_closed_library() {
    let (stdout, stderr, code) = lifetime_runs("ltcmd", "unloadercommand", false).remove(0);
    assert_eq!(
        (stdout.as_str(), stderr.as_str(), code),
        (
            "loader\ncreated A\nunloader\ncreated inner\nterminated A\ncommand after 30\n\
             terminated inner\n",
            "     1 *-* address lifetime 'x';\n       >>>   \"x\"\n       +++   \"RC(30)\"\n",
            0
        )
    );
}

/// An unloader that creates an instance, leaves it live and starts a thread that loads a second
/// library through it, with a slow loader, returns once that loader has started. The termination
/// that ran the unloader leaves the second library to the live instance: it runs no unloader of
/// it, neither during its loader nor after, and does not close it. The live instance's own
/// termination, the last, runs that library's unloader once, after its loader has returned. The
/// run is not compared with the oracle, which does not wait for loaders (see [`lines_in_order`]).
// spec: native.loading.unload, native.loading.reentrant
#[test]
fn a_library_loading_on_another_thread_is_left_to_the_instance_that_holds_it() {
    let (stdout, stderr, code) =
        lifetime_runs("lifetime-unloaderthread", "unloaderthread", false).remove(0);
    assert_eq!((stderr.as_str(), code), ("", 0), "{stdout}");
    lines_in_order(
        &stdout,
        &[
            "loader",
            "created A",
            "unloader",
            "created inner",
            "other loader start",
        ],
        &["terminated A", "other loader end"],
        &["thread other 1", "other unloader", "terminated inner"],
    );
}

/// An unloader that loads a library new to the process runs that library's loader inside it, and
/// the same termination runs the new library's unloader too. An instance created afterwards loads
/// both afresh, and its termination unloads both.
// spec: native.loading.unload, native.loading.again
#[test]
fn a_library_an_unloader_loads_is_unloaded_in_the_same_termination() {
    assert_eq!(
        run_lifetime("lifetime-unloaderother", "unloaderother"),
        (
            "loader\ncreated A\nunloader\nother loader\nunloader other 1\nother unloader\n\
             terminated A\nloader\ncreated C\nother loader\nC other 1\nunloader\n\
             other unloader\nterminated C\n"
                .to_owned(),
            String::new(),
            0
        )
    );
}

/// The lines of one run of the scenario `callrace` that the rules fix: it completes, the loader and
/// the unloader each run once, and a call by name made after the load has returned answers. The
/// calls made while the loader runs may answer or raise, which is the program's concern, so the
/// `race` line is checked to be there and nothing more.
fn check_callrace((stdout, stderr, code): &Answer) {
    assert_eq!((stderr.as_str(), *code), ("", 0), "{stdout}");
    let lines: Vec<&str> = stdout.lines().collect();
    for once in [
        "created A",
        "loader start",
        "loader end",
        "after 1",
        "unloader",
    ] {
        assert_eq!(
            lines.iter().filter(|line| **line == once).count(),
            1,
            "{once}: {stdout}"
        );
    }
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with("race "))
            .count(),
        1,
        "{stdout}"
    );
    assert_eq!(lines.last(), Some(&"terminated A"), "{stdout}");
}

/// A routine of the library that one activity calls by name, again and again, while another
/// activity of the same instance runs the library's loader with the baton vacated, leaves the
/// interpreter sound: the run completes, the loader and the unloader each run once, a call made
/// after the load has returned answers, and `Terminate` returns. What the calls made during the
/// loader answer is unspecified, so it is not checked, and the run is not compared with the
/// oracle.
// spec: native.loading.shared
#[test]
fn a_routine_called_by_name_during_its_loader_leaves_the_interpreter_sound() {
    check_callrace(&lifetime_runs("lifetime-callrace", "callrace", false).remove(0));
}

/// [`a_routine_called_by_name_during_its_loader_leaves_the_interpreter_sound`]'s scenario a
/// hundred times in a row, each run a process of its own under the test's deadline, so that a
/// crash, a panic, a hang or a corrupted state that the race reaches only now and then fails it.
// spec: native.loading.shared
#[test]
#[ignore = "a hundred runs; CI runs it with --run-ignored ignored-only"]
fn a_routine_called_by_name_during_its_loader_leaves_the_interpreter_sound_a_hundred_times() {
    // A short name, as the API service's socket path under the run's directory must fit a
    // socket address, or the service says on stderr that it is not used.
    let (dir, driver) = build_driver(
        "lifetime-race100",
        "lifetime.cpp",
        &[(None, "lifetime"), (Some("-DOTHER"), "lifetime2")],
    );
    for _ in 0..100 {
        check_callrace(&run(&driver, &dir, None, Some("callrace")));
    }
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}

/// A thread attached to the last instance before the embedding terminates it, which calls into the
/// instance while that termination runs the library's unloader, does not crash or hang the
/// termination: its send is refused and answers no object, the unloader returns and `Terminate`
/// returns. Under `REXX_CORPUS_GATE` the oracle prints the same, except that it runs the send and
/// answers `3`, as Deviation 176 (`native.threads.end.deviation.176`) records.
// spec: native.loading.unload, native.loading.reentrant, native.threads.end.deviation.176
#[test]
fn a_thread_calling_in_during_the_last_unloader_is_refused() {
    let mut runs = lifetime_runs("lifetime-unloadercall", "unloadercall", gate_mode()).into_iter();
    let ours = runs.next().expect("this workspace's run");
    let expected = |answer: &str| {
        format!(
            "loader\ncreated A\nunloader start\nthread callback {answer}\nunloader end\n\
             terminated A\njoined\n"
        )
    };
    assert_eq!(ours, (expected("none"), String::new(), 0));
    if let Some(theirs) = runs.next() {
        assert_eq!(theirs, (expected("3"), String::new(), 0), "the oracle");
    }
}

/// Builds `crossload.cpp` as the extensions `crossslow`, `crossquick` and `crossouter` and as the
/// driver, and runs the driver with `scenario`, and where `oracle` is true the oracle's driver
/// after it.
fn crossload_runs(name: &str, scenario: &str, oracle: bool) -> Vec<Answer> {
    driver_runs(
        name,
        "crossload.cpp",
        &[
            (Some("-DSLOW"), "crossslow"),
            (Some("-DQUICK"), "crossquick"),
            (Some("-DOUTER"), "crossouter"),
        ],
        scenario,
        oracle,
    )
}

/// A load of a library nobody is loading does not wait for the loader of another library, here
/// one that another thread runs and that calls into the instance the load is made through: the
/// load completes, with its own loader, while the other loader sleeps, and that loader's call then
/// completes too.
// spec: native.loading.shared
#[test]
fn a_load_does_not_wait_for_the_loader_of_another_library() {
    let mut runs = crossload_runs("crossload-other", "other", gate_mode()).into_iter();
    let ours = runs.next().expect("this workspace's run");
    assert_eq!(
        ours,
        (
            "created first\ncreated second\nslow loader start\nquick loader\nmain crossquick 1\n\
             slow loader callback 3\nthread crossslow 1\nterminated\n"
                .to_owned(),
            String::new(),
            0
        )
    );
    if let Some(theirs) = runs.next() {
        assert_eq!(theirs, ours, "the oracle");
    }
}

/// A load that waits for another thread's loader of the same library does not keep that loader
/// from calling into the instance the load is made through: the loader's call completes, and the
/// load completes once the loader has returned. The oracle does not wait for the loader, so the
/// run is not compared with it.
// spec: native.loading.shared
#[test]
fn a_load_waiting_for_a_loader_lets_it_call_into_the_waiting_instance() {
    let (stdout, stderr, code) = crossload_runs("crossload-same", "same", false).remove(0);
    assert_eq!((stderr.as_str(), code), ("", 0), "{stdout}");
    lines_in_order(
        &stdout,
        &[
            "created first",
            "created second",
            "slow loader start",
            "slow loader callback 3",
        ],
        &["main crossslow 1", "thread crossslow 1"],
        &["terminated"],
    );
}

/// A loader runs with its instance's baton vacated, as an external call does, so a load it makes
/// through another instance may wait for a loader that calls into the first instance: the main
/// thread runs `crossouter`'s loader on `second`, and that loader waits for `crossslow`'s loader,
/// which calls into `second` meanwhile. Each load completes once its loader has returned. The
/// oracle does not wait for the loader, so the run is not compared with it.
// spec: native.loading.shared
#[test]
fn a_loader_waiting_for_a_loader_lets_it_call_into_the_instance_running_the_first() {
    let (stdout, stderr, code) = crossload_runs("crossload-hook", "hook", false).remove(0);
    assert_eq!((stderr.as_str(), code), ("", 0), "{stdout}");
    let middle = lines_in_order(
        &stdout,
        &[
            "created first",
            "created second",
            "slow loader start",
            "outer loader start",
            "slow loader callback 3",
        ],
        &[
            "outer loaded crossslow 1",
            "outer loader end",
            "main crossouter 1",
            "thread crossslow 1",
        ],
        &["terminated"],
    );
    let at = |line: &str| {
        middle
            .iter()
            .position(|held| *held == line)
            .expect("lines_in_order checked every line is there")
    };
    assert!(
        at("outer loaded crossslow 1") < at("outer loader end")
            && at("outer loader end") < at("main crossouter 1"),
        "{stdout}"
    );
}

/// Two loaders, running at once on two threads, that each load the other's library do not wait
/// for each other. The load of the second to ask would close the cycle, so it finds the library
/// loaded and does not wait, and the first's load waits for the second's loader to return. The
/// oracle waits for neither loader, so the run is not compared with it.
// spec: native.loading.reentrant
#[test]
fn loaders_loading_each_others_library_do_not_wait_for_each_other() {
    let (stdout, stderr, code) = driver_runs(
        "cycle",
        "cycle.cpp",
        &[
            (Some("-DFIRST"), "cyclefirst"),
            (Some("-DSECOND"), "cyclesecond"),
        ],
        "cycle",
        false,
    )
    .remove(0);
    assert_eq!((stderr.as_str(), code), ("", 0), "{stdout}");
    let middle = lines_in_order(
        &stdout,
        &[
            "cyclefirst loader start",
            "cyclesecond loader start",
            "cyclesecond loaded cyclefirst 1",
            "cyclesecond loader end",
        ],
        &[
            "main cyclesecond 1",
            "cyclefirst loaded cyclesecond 1",
            "cyclefirst loader end",
            "thread cyclefirst 1",
        ],
        &["terminated"],
    );
    let at = |line: &str| {
        middle
            .iter()
            .position(|held| *held == line)
            .expect("lines_in_order checked every line is there")
    };
    assert!(
        at("cyclefirst loaded cyclesecond 1") < at("cyclefirst loader end")
            && at("cyclefirst loader end") < at("thread cyclefirst 1"),
        "{stdout}"
    );
}
