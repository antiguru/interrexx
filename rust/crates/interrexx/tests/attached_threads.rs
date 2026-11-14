//! **Attached threads** (`r[native.threads.attach]`). The extension
//! `tests/attached_threads/attach.cpp`, compiled with `g++` against this worktree's `api/` into a
//! directory of this run's own, starts threads that attach to the interpreter through the
//! instance a call's thread context links, send messages, and detach. The programs beside it start
//! them and run Rexx code, waiting with `SysSleep` until every thread has detached, then print what
//! the messages did. Each expectation is literal. Under `REXX_CORPUS_GATE` the oracle runs the same
//! program from the same directory and must print the same.

#![cfg(target_os = "linux")]

mod support;

#[path = "../../interrexx-capi-rxapi/tests/support/cdylibs.rs"]
mod cdylibs;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The `memcap` limit each run is under.
const MEMORY: &str = "2G";

/// The seconds `timeout` gives each run before it kills it, so a run that deadlocks fails
/// instead of hanging. It runs inside `memcap`, so the signal reaches `rexx` itself.
const SECONDS: &str = "120";

const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    std::env::var(GATE_ENV).is_ok_and(|value| !value.is_empty() && value != "0")
}

fn worktree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/attached_threads")
        .join(name)
}

/// A fresh directory of this run's own, named for `name`, holding the extension `attach`.
fn build(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a directory of this run's own");
    let api = worktree().join("third_party/ooRexx/api");
    let built = Command::new("g++")
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(fixture("attach.cpp"))
        .args([
            "-shared",
            "-fPIC",
            "-O1",
            "-static-libstdc++",
            "-pthread",
            "-o",
        ])
        .arg(dir.join("libattach.so"))
        .status()
        .expect("g++ runs, which this test needs to build its fixture");
    assert!(built.success(), "the fixture did not build");
    dir
}

/// What a run answered: stdout, stderr and the exit status.
type Answer = (String, String, i32);

/// Runs the fixture program `program` under `rexx`, memory-capped by `memcap` and limited in time
/// by `timeout`, from a directory holding the extension, answering its stdout, stderr and exit
/// status. Under `REXX_CORPUS_GATE` the oracle runs the same program from the same directory and
/// must answer the same.
fn run(name: &str, program: &str) -> Answer {
    run_timed(name, program).0
}

/// [`run`], answering besides how long our run took, from its start to its exit.
fn run_timed(name: &str, program: &str) -> (Answer, std::time::Duration) {
    run_against(name, program, gate_mode())
}

/// [`run`] where the oracle is not asked even under `REXX_CORPUS_GATE`: it aborts the program,
/// or what the program reports is a property of our memory use that the oracle does not share.
fn run_ours(name: &str, program: &str) -> Answer {
    run_against(name, program, false).0
}

/// [`run_timed`], asking the oracle where `oracle`.
#[expect(
    clippy::disallowed_methods,
    reason = "the run's wall time is what an idle thread must not stretch"
)]
fn run_against(name: &str, program: &str, oracle: bool) -> (Answer, std::time::Duration) {
    let dir = build(name);
    let program = fixture(program);
    let began = std::time::Instant::now();
    let output = Command::new("memcap")
        .arg(MEMORY)
        .args(["timeout", "-s", "KILL", SECONDS])
        .arg(env!("CARGO_BIN_EXE_rexx"))
        .arg(&program)
        .current_dir(&dir)
        .env("LD_LIBRARY_PATH", &dir)
        .env("REXX_API_DIR", dir.join("api"))
        .stdin(Stdio::null())
        .output()
        .expect("memcap runs, which every run of this test is under");
    let took = began.elapsed();
    let _ = Command::new(Path::new(env!("CARGO_BIN_EXE_rexx")).with_file_name("rxapi"))
        .arg("--stop")
        .env("REXX_API_DIR", dir.join("api"))
        .output();
    let ours = (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
    );
    if oracle {
        let oracle = support::oracle::locate();
        let library_path = format!("{}:{}", oracle.lib_dir().display(), dir.display());
        let theirs = oracle.run_in(&program, &dir, &[("LD_LIBRARY_PATH", &library_path)]);
        assert_eq!(
            (
                String::from_utf8_lossy(&theirs.stdout).into_owned(),
                String::from_utf8_lossy(&theirs.stderr).into_owned(),
                theirs.expect_exit_code(),
            ),
            ours,
            "the oracle"
        );
    }
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
    (ours, took)
}

/// The embedding driver `name` from `tests/attached_threads/`, built into a fresh directory of
/// this run's own against this workspace's `librexx.so.4` and `librexxapi.so.4`, which it holds
/// beside the driver.
fn build_driver(name: &str) -> (PathBuf, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a directory of this run's own");
    for (source, library) in [
        (cdylibs::librexx(), "librexx"),
        (cdylibs::librexxapi(), "librexxapi"),
    ] {
        fs::copy(&source, dir.join(format!("{library}.so.4"))).expect("a copy of the library");
        std::os::unix::fs::symlink(format!("{library}.so.4"), dir.join(format!("{library}.so")))
            .expect("the library's link");
    }
    let api = worktree().join("third_party/ooRexx/api");
    let binary = dir.join(name);
    let built = Command::new("g++")
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(fixture(&format!("{name}.cpp")))
        .arg("-o")
        .arg(&binary)
        .arg(format!("-L{}", dir.display()))
        .arg(format!("-Wl,-rpath,{}", dir.display()))
        .args(["-pthread", "-lrexx", "-lrexxapi"])
        .status()
        .expect("g++ runs, which this test needs to build its driver");
    assert!(built.success(), "the driver did not build");
    (dir, binary)
}

/// Runs the embedding driver `name` with `arguments`, memory-capped and limited in time as
/// [`run`] is, answering its stdout, stderr and exit status.
fn run_driver(name: &str, arguments: &[&str]) -> Answer {
    let (dir, binary) = build_driver(name);
    let output = Command::new("memcap")
        .arg(MEMORY)
        .args(["timeout", "-s", "KILL", SECONDS])
        .arg(&binary)
        .args(arguments)
        .current_dir(&dir)
        .env("REXX_API_DIR", dir.join("api"))
        .stdin(Stdio::null())
        .output()
        .expect("memcap runs, which every run of this test is under");
    let _ = Command::new(Path::new(env!("CARGO_BIN_EXE_rexx")).with_file_name("rxapi"))
        .arg("--stop")
        .env("REXX_API_DIR", dir.join("api"))
        .output();
    let answer = (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
    );
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
    answer
}

/// A thread that attaches sends `TICK` while the main program runs Rexx code, which sleeps until
/// the thread has detached: the message runs, and the program ends normally.
// spec: native.threads.attach
#[test]
fn an_attached_thread_sends_while_main_runs_rexx() {
    assert_eq!(
        run("attach-send", "send.rex"),
        ("ticked 1\n".to_owned(), String::new(), 0)
    );
}

/// The message an attached thread sends runs as an activity of its own: `.context~thread` in it
/// is not the main program's.
// spec: native.threads.attach
#[test]
fn an_attached_callback_has_its_own_thread_number() {
    assert_eq!(
        run("attach-thread", "thread.rex"),
        ("differs\n".to_owned(), String::new(), 0)
    );
}

/// Two threads attached at once each send `TICK` five times while the main program sleeps: every
/// message runs.
// spec: native.threads.attach
#[test]
fn two_attached_threads_call_back_concurrently() {
    assert_eq!(
        run("attach-two", "two.rex"),
        ("ticked 10\n".to_owned(), String::new(), 0)
    );
}

/// A message an attached thread sends raises SYNTAX 40: the thread finds the condition with
/// `CheckCondition`, clears it, and its next message runs.
// spec: native.threads.attach
#[test]
fn a_raising_callback_leaves_the_attachment_usable() {
    assert_eq!(
        run("attach-raise", "raise.rex"),
        (
            "checked 1 cleared 0\nticked 1\n".to_owned(),
            String::new(),
            0
        )
    );
}

/// A string an attached thread makes with `NewStringFromAsciiz` is a local reference of its
/// attachment: a later `SendMessage1` passes it, after a `TICK` between, and the method says it.
// spec: native.threads.attach
#[test]
fn a_local_reference_survives_between_callbacks() {
    assert_eq!(
        run("attach-local", "local.rex"),
        ("shown kept\nticked 1\n".to_owned(), String::new(), 0)
    );
}

/// The address `StringData` answers on an attached thread stays readable after a later callback
/// of the attachment: the thread reads the bytes after a `TICK`.
// spec: native.threads.attach
#[test]
fn string_data_on_an_attached_thread_outlives_its_callback() {
    assert_eq!(
        run("attach-string", "string.rex"),
        ("string abc\nticked 1\n".to_owned(), String::new(), 0)
    );
}

/// A buffer string an attached thread makes with `NewBufferString` is written through the
/// address a later `BufferStringData` answers, finished at three of its four bytes, and passed to
/// a method, which says it. `BufferStringLength` then answers the finished length.
// spec: native.threads.attach
#[test]
fn a_buffer_string_on_an_attached_thread_is_written_and_finished() {
    assert_eq!(
        run("attach-buffer", "buffer.rex"),
        ("shown abc\nbuffer 3\n".to_owned(), String::new(), 0)
    );
}

/// A native call starts a thread that attaches and detaches, and joins it before it returns:
/// the attach answers, with the only activity in that call, and the call answers 1.
// spec: native.threads.attach
#[test]
fn a_thread_a_native_call_joins_attaches_and_detaches() {
    assert_eq!(
        run("attach-join", "join.rex"),
        ("1\n".to_owned(), String::new(), 0)
    );
}

/// A thread that has sent its message and stays attached, idle, for two seconds does not hold the
/// program's end: the run exits 0 well inside the thread's idle time. The oracle takes 0.28 s.
// spec: native.threads.end
#[test]
fn an_idle_attached_thread_does_not_hold_program_end() {
    let (answer, took) = run_timed("attach-idle", "idle.rex");
    assert_eq!(answer, ("main ends\n".to_owned(), String::new(), 0));
    assert!(
        took < std::time::Duration::from_secs(1),
        "the run took {took:?}"
    );
}

/// A message an attached thread sent before the main program ends, still running when it does,
/// holds the program's end until it finishes, and the program ends with rc 0. The callback buries
/// the main activity while it runs, so the run does not separate the hold on the program's end
/// from main waiting for the callback.
// spec: native.threads.end
#[test]
fn a_running_callback_holds_program_end() {
    assert_eq!(
        run("attach-slow", "slow.rex"),
        ("slow done\n".to_owned(), String::new(), 0)
    );
}

/// One thread attaches, sends `TICK` and detaches ten thousand times: every message runs, and
/// the contexts its attaches answer are no more than two distinct ones, as an attach takes the
/// context the last detach freed. The oracle answers a new context to each attach, so it is not
/// asked.
// spec: native.threads.attach
#[test]
fn attach_detach_cycles_stay_bounded() {
    assert_eq!(
        run_ours("attach-cycles", "cycles.rex"),
        (
            "ticks 10000\ncontexts bounded\n".to_owned(),
            String::new(),
            0
        )
    );
}

/// A `DetachThread` from a thread not attached to the context, and a second `DetachThread` from
/// the one that was, do nothing: the attachment's second message, sent between them, runs. The
/// oracle aborts the process (rc 134), so it is not asked.
// spec: native.threads.attach, native.threads.attach.deviation.168
#[test]
fn detach_from_the_wrong_thread_or_twice_does_nothing() {
    assert_eq!(
        run_ours("attach-wrong", "wrong.rex"),
        (
            "first answered second answered\nticked 2\n".to_owned(),
            String::new(),
            0
        )
    );
}

/// A message sent through a thread context after its thread detached from it answers nothing
/// and does not run, where it aborted the process before. The oracle aborts the process
/// (rc 134), so it is not asked.
// spec: native.threads.attach
#[test]
fn a_callback_after_detach_is_refused() {
    assert_eq!(
        run_ours("attach-late", "late.rex"),
        (
            "first answered late null\nticked 1\n".to_owned(),
            String::new(),
            0
        )
    );
}

/// A thread detaches from inside a message it sent through its attachment, from a native call
/// that message makes: the attached activity ends once the send returns, so the thread's next
/// attach is answered the same context, and its message runs. The oracle answers a new context
/// to each attach, so it is not asked.
// spec: native.threads.attach
#[test]
fn a_detach_inside_a_callback_ends_the_activity_after_it() {
    assert_eq!(
        run_ours("attach-inner", "inner.rex"),
        ("reused ticked 1\n".to_owned(), String::new(), 0)
    );
}

/// One attachment makes, writes, finishes and releases thirty thousand buffer strings of 100000
/// bytes: each answers its own area and its length, and the areas of the released ones, which
/// would hold three gigabytes, are dropped, so the run stays under its memory cap.
// spec: native.threads.attach
#[test]
fn buffer_strings_released_on_one_attachment_do_not_pile_up() {
    assert_eq!(
        run("attach-buffers", "buffers.rex"),
        ("buffers 30000\n".to_owned(), String::new(), 0)
    );
}

/// An embedding's thread, attached and idle when the creating thread calls `Terminate`, sends a
/// message afterwards: the send answers nothing, `DetachThread` returns, and nothing aborts or
/// hangs. The oracle, with the same driver, still runs the message (`after answered`).
// spec: native.threads.end, native.threads.end.deviation.167
#[test]
fn a_callback_after_terminate_is_refused() {
    assert_eq!(
        run_driver("embed", &[]),
        (
            "before 3\nterminated\nafter null\ndetached\n".to_owned(),
            String::new(),
            0
        )
    );
}
