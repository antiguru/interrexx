//! **Attached threads** (`r[native.threads.attach]`). The extension
//! `tests/attached_threads/attach.cpp`, compiled with `g++` against this worktree's `api/` into a
//! directory of this run's own, starts threads that attach to the interpreter through the
//! instance a call's thread context links, send messages, and detach. The programs beside it start
//! them and run Rexx code, waiting with `SysSleep` until every thread has detached, then print what
//! the messages did. Each expectation is literal; under `REXX_CORPUS_GATE` the oracle runs the same
//! program from the same directory and must print the same.

#![cfg(target_os = "linux")]

mod support;

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

/// Runs the fixture program `program` under `rexx`, memory-capped by `memcap`, from a directory
/// holding the extension, answering its stdout, stderr and exit status. Under `REXX_CORPUS_GATE`
/// the oracle runs the same program from the same directory and must answer the same.
fn run(name: &str, program: &str) -> Answer {
    let dir = build(name);
    let program = fixture(program);
    let output = Command::new("memcap")
        .arg(MEMORY)
        .arg(env!("CARGO_BIN_EXE_rexx"))
        .arg(&program)
        .current_dir(&dir)
        .env("LD_LIBRARY_PATH", &dir)
        .env("REXX_API_DIR", dir.join("api"))
        .stdin(Stdio::null())
        .output()
        .expect("memcap runs, which every run of this test is under");
    let _ = Command::new(Path::new(env!("CARGO_BIN_EXE_rexx")).with_file_name("rxapi"))
        .arg("--stop")
        .env("REXX_API_DIR", dir.join("api"))
        .output();
    let ours = (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
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
            ours,
            "the oracle"
        );
    }
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
    ours
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
