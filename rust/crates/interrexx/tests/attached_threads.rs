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
    let asked = if gate_mode() {
        Oracle::Whole
    } else {
        Oracle::Not
    };
    run_against(name, program, asked, SECONDS, &[])
}

/// [`run`] where the oracle is not asked even under `REXX_CORPUS_GATE`: it aborts the program,
/// or what the program reports is a property of our memory use that the oracle does not share.
fn run_ours(name: &str, program: &str) -> Answer {
    run_against(name, program, Oracle::Not, SECONDS, &[]).0
}

/// [`run_ours`], killed after `seconds` rather than [`SECONDS`]: the program waits for ever.
fn run_ours_for(name: &str, program: &str, seconds: &str) -> Answer {
    run_against(name, program, Oracle::Not, seconds, &[]).0
}

/// [`run`] where the oracle, under `REXX_CORPUS_GATE`, must answer the same stderr and exit status
/// only: its stdout differs as a Deviation says.
fn run_agreeing_on_status(name: &str, program: &str) -> Answer {
    let asked = if gate_mode() {
        Oracle::Status
    } else {
        Oracle::Not
    };
    run_against(name, program, asked, SECONDS, &[]).0
}

/// [`run`] where the oracle, under `REXX_CORPUS_GATE`, must answer the same stdout only: its
/// stderr and exit status differ as a Deviation says.
fn run_agreeing_on_stdout(name: &str, program: &str) -> Answer {
    let asked = if gate_mode() {
        Oracle::Stdout
    } else {
        Oracle::Not
    };
    run_against(name, program, asked, SECONDS, &[]).0
}

/// [`run`] of a program that our run ends with the refusal of a wait nothing can end, where the
/// oracle, under `REXX_CORPUS_GATE`, waits for ever: it must print the same up to that refusal and
/// still be running when it is killed.
fn run_up_to_refusal(name: &str, program: &str) -> Answer {
    let asked = if gate_mode() {
        Oracle::UpToRefusal
    } else {
        Oracle::Not
    };
    run_against(name, program, asked, SECONDS, &[]).0
}

/// What of a run the oracle is asked to answer the same.
#[derive(Clone, Copy, PartialEq)]
enum Oracle {
    Not,
    Whole,
    Status,
    Stdout,
    /// Everything our run printed before the refusal of a wait nothing can end, the oracle having
    /// waited for ever instead.
    UpToRefusal,
}

/// [`run_timed`], asking the oracle as `oracle` says, our run killed after `seconds` and given
/// the environment variables `environment` besides.
#[expect(
    clippy::disallowed_methods,
    reason = "the run's wall time is what an idle thread must not stretch"
)]
fn run_against(
    name: &str,
    program: &str,
    oracle: Oracle,
    seconds: &str,
    environment: &[(&str, &str)],
) -> (Answer, std::time::Duration) {
    let dir = build(name);
    let program = fixture(program);
    let began = std::time::Instant::now();
    let output = Command::new("memcap")
        .arg(MEMORY)
        .args(["timeout", "-s", "KILL", seconds])
        .arg(env!("CARGO_BIN_EXE_rexx"))
        .arg(&program)
        .current_dir(&dir)
        .env("LD_LIBRARY_PATH", &dir)
        .env("REXX_API_DIR", dir.join("api"))
        .envs(environment.iter().copied())
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
    if oracle != Oracle::Not {
        let located = support::oracle::locate();
        let library_path = format!("{}:{}", located.lib_dir().display(), dir.display());
        let theirs = located.run_in(&program, &dir, &[("LD_LIBRARY_PATH", &library_path)]);
        if oracle == Oracle::UpToRefusal {
            assert!(
                support::oracle::did_not_finish(&theirs),
                "the oracle ended its wait"
            );
            let before = ours
                .1
                .strip_suffix(UNENDING_WAIT)
                .unwrap_or_else(|| panic!("our run did not end with the refusal: {}", ours.1));
            assert_eq!(
                (
                    String::from_utf8_lossy(&theirs.stdout).as_ref(),
                    String::from_utf8_lossy(&theirs.stderr).as_ref()
                ),
                (ours.0.as_str(), before),
                "the oracle, up to the refusal"
            );
            fs::remove_dir_all(&dir).expect("the run's directory is removable");
            return (ours, took);
        }
        let theirs = (
            String::from_utf8_lossy(&theirs.stdout).into_owned(),
            String::from_utf8_lossy(&theirs.stderr).into_owned(),
            theirs.expect_exit_code(),
        );
        match oracle {
            Oracle::Whole => assert_eq!(theirs, ours, "the oracle"),
            Oracle::Status => assert_eq!(
                (&theirs.1, theirs.2),
                (&ours.1, ours.2),
                "the oracle's status"
            ),
            Oracle::Stdout => assert_eq!(theirs.0, ours.0, "the oracle's stdout"),
            Oracle::Not | Oracle::UpToRefusal => {}
        }
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
    run_driver_for(name, arguments, SECONDS)
}

/// [`run_driver`], killed after `seconds` rather than [`SECONDS`]: the driver waits for ever.
fn run_driver_for(name: &str, arguments: &[&str], seconds: &str) -> Answer {
    let (dir, binary) = build_driver(name);
    let output = Command::new("memcap")
        .arg(MEMORY)
        .args(["timeout", "-s", "KILL", seconds])
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

/// Two threads attached at once each send `TICK` to one object five times while the main program
/// sleeps: every message runs. A `TICK` that arrives while the other thread's runs is held until
/// that one returns, so it never waits for the guard that one holds.
// spec: native.threads.attach
#[test]
fn two_attached_threads_call_back_concurrently() {
    assert_eq!(
        run("attach-two", "two.rex"),
        ("ticked 10\n".to_owned(), String::new(), 0)
    );
}

/// [`two_attached_threads_call_back_concurrently`]'s program, run ten times in a row: every run
/// ticks ten times, however the two threads' messages meet.
// spec: native.threads.attach
#[test]
fn two_attached_threads_call_back_concurrently_every_time() {
    call_back_concurrently(10);
}

/// [`two_attached_threads_call_back_concurrently_every_time`] with two hundred runs. The
/// simulation mode refuses a callback from a thread running none of the interpreter's native
/// calls, so it cannot vary how the two threads' messages meet, and this many real runs stand in
/// for its seeds. CI runs it in a job of its own.
// spec: native.threads.attach
#[test]
#[ignore = "two hundred runs; CI runs it with --run-ignored ignored-only"]
fn two_attached_threads_call_back_concurrently_two_hundred_times() {
    call_back_concurrently(200);
}

/// Runs [`two_attached_threads_call_back_concurrently`]'s program `runs` times in a row, each
/// expected to tick ten times.
fn call_back_concurrently(runs: usize) {
    for run in 0..runs {
        assert_eq!(
            run_ours("attach-two-stress", "two.rex"),
            ("ticked 10\n".to_owned(), String::new(), 0),
            "run {run}"
        );
    }
}

/// A second attached thread's `TICK` arrives while the first thread's message runs Rexx code in a
/// guarded method of the same object: it is held until that message has returned, then runs.
// spec: native.threads.attach
#[test]
fn a_callback_arriving_inside_another_is_held_until_it_returns() {
    assert_eq!(
        run("attach-busy", "busy.rex"),
        ("ticked 1\n".to_owned(), String::new(), 0)
    );
}

/// A second attached thread's `LATER` arrives while the first thread's message runs Rexx code in a
/// guarded method of an object, and `LATER` sends `TICK` to that object from inside its own method:
/// it is held until the first message has returned, so its `TICK` finds the guard free. Served
/// on top of the first message instead, its `TICK` would wait, in the middle of its message, for a
/// guard only the stopped message can release.
// spec: native.threads.attach
#[test]
fn a_callback_arriving_inside_another_is_held_before_it_runs_anything() {
    assert_eq!(
        run("attach-busylater", "busylater.rex"),
        ("ticked 1\n".to_owned(), String::new(), 0)
    );
}

/// A second attached thread's `TICK` arrives while the first thread's message sleeps in a guarded
/// method of the same object: the `TICK` runs on top of the sleeping message, finds the guard
/// held, gives the interpreter back, and runs once the first message has returned.
// spec: native.threads.attach
#[test]
fn a_callback_waiting_for_a_guard_a_sleeping_callback_holds_runs_after_it() {
    assert_eq!(
        run("attach-sleepy", "sleepy.rex"),
        ("ticked 1\n".to_owned(), String::new(), 0)
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

/// A native call starts a thread that attaches, sends `TICK` and detaches, and joins it before it
/// returns: the message runs while the call blocks, with main's the only activity, and the program
/// says the count after the call.
// spec: native.threads.attach
#[test]
fn a_callback_while_main_blocks_in_a_native_call() {
    assert_eq!(
        run("attach-block", "block.rex"),
        ("ticked 1\n".to_owned(), String::new(), 0)
    );
}

/// An attached thread's message calls the routine that starts a second thread and joins it, a
/// call made while a callback runs and so pinned: the second thread's `TICK` runs while that call
/// blocks.
// spec: native.threads.attach
#[test]
fn a_callback_while_a_pinned_native_call_blocks() {
    assert_eq!(
        run("attach-pinned", "pinned.rex"),
        ("ticked 1\n".to_owned(), String::new(), 0)
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
/// holds the program's end until it finishes, and the program ends with rc 0. The main program
/// says it ends only once the message has finished, as it makes no progress while the message runs
/// (Deviation 176). The oracle says so first, so it is asked for the exit status only.
// spec: native.threads.end, native.threads.attach.deviation.176
#[test]
fn a_running_callback_holds_program_end() {
    assert_eq!(
        run_agreeing_on_status("attach-slow", "slow.rex"),
        ("slow done\nmain ends\n".to_owned(), String::new(), 0)
    );
}

/// After the main program has ended, while the end waits for a started activity that naps, a
/// message an attached thread sent waits, parked, on an event semaphore that a second thread posts
/// after the nap: the parked message holds the program's end as a running one does, and says it
/// saw the post. The oracle agrees under the gate.
// spec: native.threads.end
#[test]
fn a_parked_callback_holds_program_end() {
    assert_eq!(
        run("attach-parkedend", "parkedend.rex"),
        ("main ends\nsem seen\n".to_owned(), String::new(), 0)
    );
}

/// [`a_running_callback_holds_program_end`]'s program a hundred times in a row. Program end ends
/// the attached thread's message and returns while the thread is still leaving the extension, so
/// the extension has to stay mapped until the process exits, or a run in about thirty crashes in
/// it (#683).
// spec: native.threads.end
#[test]
#[ignore = "a hundred runs; CI runs it with --run-ignored ignored-only"]
fn a_running_callback_holds_program_end_a_hundred_times() {
    for run in 0..100 {
        assert_eq!(
            run_ours("attach-slow", "slow.rex"),
            ("slow done\nmain ends\n".to_owned(), String::new(), 0),
            "run {run}"
        );
    }
}

/// A message an attached thread sent sleeps a second while the main program loops on
/// `SysSleep 0.05`: the main program makes no progress meanwhile, its sleeps included, and counts
/// fewer than five (Deviation 176). The oracle runs it meanwhile and says `main ran`, so it is asked
/// for the exit status only.
// spec: native.threads.attach.deviation.176
#[test]
fn a_callback_runs_while_main_runs_rexx() {
    assert_eq!(
        run_agreeing_on_status("attach-nap", "nap.rex"),
        ("main stopped\n".to_owned(), String::new(), 0)
    );
}

/// A message an attached thread sends while the main program is in a call into the extension,
/// which blocks for a second, runs during the call and sleeps two seconds. The call returns
/// first, but the main program goes on only once the message has finished (Deviation 176). The
/// oracle goes on at the return, so it is asked for the exit status only.
// spec: native.threads.attach.deviation.176
#[test]
fn a_callback_during_a_native_call_holds_its_return() {
    assert_eq!(
        run_agreeing_on_status("attach-pause", "pause.rex"),
        ("slow done\nmain returned\n".to_owned(), String::new(), 0)
    );
}

/// A message an attached thread sends while the main program waits on an event semaphore runs
/// during the wait and posts the semaphore, then sleeps a second. The main program wakes only once
/// the message has finished (Deviation 176). The oracle wakes it at the post, so it is asked for
/// the exit status only.
// spec: native.threads.attach.deviation.176
#[test]
fn a_callback_during_a_wait_holds_its_wake() {
    assert_eq!(
        run_agreeing_on_status("attach-semwait", "semwait.rex"),
        ("callback ends\nsem seen\n".to_owned(), String::new(), 0)
    );
}

/// A second attached thread's message arrives while the first's sleeps, and sleeps as long: it runs
/// on top of the sleeping one, and the first, whose sleep ends first, finishes only after the
/// second (Deviation 176). The oracle finishes them in the order
/// their sleeps end (`a b`), so it is asked for the exit status only.
// spec: native.threads.attach.deviation.176
#[test]
fn callbacks_are_served_one_on_top_of_another() {
    assert_eq!(
        run_agreeing_on_status("attach-lifo", "lifo.rex"),
        ("b a\n".to_owned(), String::new(), 0)
    );
}

/// An attached thread's `TICK` needs the guard of an object whose guarded method the main program
/// runs, the method that started the thread. The `TICK` gives the interpreter back before it runs
/// anything, and runs once the method has returned.
// spec: native.threads.attach
#[test]
fn a_callback_waiting_for_a_guard_the_main_program_holds_runs_after_it() {
    assert_eq!(
        run("attach-guard", "guard.rex"),
        ("ticked 1\n".to_owned(), String::new(), 0)
    );
}

/// [`a_callback_waiting_for_a_guard_the_main_program_holds_runs_after_it`] with the package traced:
/// the `TICK` that gives the interpreter back and runs again announces its method once, with one
/// `>I>` and one `<I<`, as the oracle's single run does.
// spec: native.threads.attach
#[test]
fn a_callback_that_runs_again_for_a_guard_is_traced_once() {
    let (stdout, stderr, status) = run("attach-traced", "traced.rex");
    assert_eq!((stdout.as_str(), status), ("ticked 1\n", 0));
    for line in [">I> Method \"TICK\"", "<I< Method \"TICK\""] {
        assert_eq!(stderr.matches(line).count(), 1, "stderr: {stderr}");
    }
}

/// An attached thread's unguarded message sends `TICK` to an object whose guarded method the main
/// program runs. The `TICK`'s wait comes after its message has begun, so the message cannot give
/// the interpreter back, and only the main program, which makes no progress while the message
/// runs, can end it: the wait is refused at once as an inverted wait (Deviation 176), and the
/// program ends with rc 120. The oracle says `ticked 1`, so it is not asked.
// spec: native.threads.attach.deviation.176
#[test]
fn a_callback_waiting_within_its_message_for_a_guard_the_main_program_holds_is_refused() {
    assert_eq!(
        run_ours("attach-later", "later.rex"),
        ("ticked 0\n".to_owned(), inverted_wait("a guard"), 120)
    );
}

/// An attached thread's unguarded message says a line, then reads an attribute through its guarded
/// getter while the main program, stopped by the message, runs a guarded method of the same
/// object. The read comes after the message has begun, so the message cannot give the
/// interpreter back and send again, which would say the line twice: the wait is refused as an
/// inverted wait (Deviation 176), the line is said once, and the program ends with rc 120. The
/// oracle's read waits for the method to return, so it is asked for the stdout only.
// spec: native.threads.attach.deviation.176
#[test]
fn a_callback_reading_a_guarded_attribute_within_its_message_runs_it_once() {
    assert_eq!(
        run_agreeing_on_stdout("attach-getter", "getter.rex"),
        (
            "getlater runs\ndone\n".to_owned(),
            inverted_wait("a guard"),
            120
        )
    );
}

/// An attached thread calls a routine through `CallRoutine` that says a line, then reads an
/// attribute through its guarded getter while the main program, stopped by the call, runs a
/// guarded method of the same object. Only a send gives the interpreter back and sends again, so
/// the read is refused as an inverted wait (Deviation 176), the line is said once, and the
/// thread's next send, a `TICK` to a second object, runs once. The oracle's read waits for the
/// method to return, so it is asked for the stdout only.
// spec: native.threads.attach.deviation.176
#[test]
fn a_routine_an_attached_thread_calls_runs_once_and_so_does_its_next_send() {
    assert_eq!(
        run_agreeing_on_stdout("attach-call", "call.rex"),
        (
            "routine runs\nticked 1\n".to_owned(),
            inverted_wait("a guard"),
            120
        )
    );
}

/// An attached thread puts an item through `ArrayPut` into an array whose class defines a
/// guarded `PUT` that says it runs, while the main program, stopped by the put, runs a guarded
/// method of the same array. `ArrayPut` stores into the array without sending `PUT`, so the
/// method neither runs nor waits for the guard, and the thread's next send, a `TICK` to a second
/// object, runs once.
// spec: native.threads.attach
#[test]
fn an_array_put_from_an_attached_thread_stores_without_a_send() {
    assert_eq!(
        run("attach-put", "put.rex"),
        ("ticked 1 items 1\n".to_owned(), String::new(), 0)
    );
}

/// [`an_array_put_from_an_attached_thread_stores_without_a_send`] with the main program asleep in
/// an unguarded method of the array: the array's `PUT`, which nothing keeps from running, does not
/// run either.
// spec: native.threads.attach
#[test]
fn an_array_put_from_an_attached_thread_runs_no_put_method() {
    assert_eq!(
        run("attach-putfree", "putfree.rex"),
        ("ticked 1 items 1\n".to_owned(), String::new(), 0)
    );
}

/// The members of the thread interface that store into or fetch from an array, a stem, a supplier,
/// a directory or a string table, ask an object `IsInstanceOf` or `HasMethod`, or ask a package,
/// a routine or a method for what it holds, act on the object itself: a method its class defines
/// under the name of the matching message, which says that it runs, is never sent to. The
/// program's own `a~items` afterwards is a message, so that method says it runs.
// spec: native.threads.attach
#[test]
fn thread_interface_members_send_no_message_a_subclass_redefines() {
    let expected = "\
ArrayAt v
ArrayAt null
ArrayAppend 2
ArrayAppendString 3
ArraySize 3
ArrayItems 3
ArrayDimension 1
GetStemElement v
GetStemElement null
GetStemArrayElement w
GetAllStemElements present
GetStemArrayElement null
GetStemValue S.
SupplierAvailable 1
SupplierItem i1
SupplierIndex x1
SupplierAvailable 0
DirectoryAt v
DirectoryRemove v
StringTableAt v
StringTableRemove v
IsInstanceOf 1
HasMethod 0
FindPackageClass null
GetPackageRoutines present
GetPackagePublicRoutines present
GetPackageClasses present
GetPackagePublicClasses present
GetPackageMethods present
GetRoutinePackage present
GetMethodPackage present
ITEMS runs
array 3 directory 0 table 0
";
    assert_eq!(
        run("attach-members", "members.rex"),
        (expected.to_owned(), String::new(), 0)
    );
}

/// [`thread_interface_members_send_no_message_a_subclass_redefines`] on instances of the built-in
/// classes themselves, each given methods of its own under the names of the matching messages with
/// `setMethod` and the scope `OBJECT`: none of those methods runs.
// spec: native.threads.attach
#[test]
fn thread_interface_members_run_no_method_an_object_sets_on_itself() {
    let expected = "\
Array Stem Supplier Object Package Routine Method
ITEMS runs
direct: fake
ArrayAt v
ArrayAt null
ArrayAppend 2
ArrayAppendString 3
ArraySize 3
ArrayItems 3
ArrayDimension 1
GetStemElement v
GetStemElement null
GetStemArrayElement w
GetAllStemElements present
GetStemArrayElement null
GetStemValue S.
SupplierAvailable 1
SupplierItem i1
SupplierIndex x1
SupplierAvailable 0
DirectoryAt v
DirectoryRemove v
StringTableAt v
StringTableRemove v
IsInstanceOf 1
HasMethod 0
FindPackageClass null
GetPackageRoutines present
GetPackagePublicRoutines present
GetPackageClasses present
GetPackagePublicClasses present
GetPackageMethods present
GetRoutinePackage present
GetMethodPackage present
";
    assert_eq!(
        run("attach-objm", "objm.rex"),
        (expected.to_owned(), String::new(), 0)
    );
}

/// [`thread_interface_members_send_no_message_a_subclass_redefines`] for `IsInstanceOf` and
/// `HasMethod` on an instance of a class of the program that inherits `Comparable`, a mixin class
/// of the interpreter's own library, and then a mixin class of the program that redefines the
/// methods of those names. `Comparable` comes ahead of that mixin in the instance's class order,
/// and the redefinitions do not run either.
// spec: native.threads.attach
#[test]
fn thread_interface_members_send_no_message_a_mixin_behind_a_library_mixin_redefines() {
    assert_eq!(
        run("attach-mixin", "mixin.rex"),
        ("IsInstanceOf 1\nHasMethod 0\n".to_owned(), String::new(), 0)
    );
}

/// [`thread_interface_members_send_no_message_a_subclass_redefines`] for `IsInstanceOf` and
/// `HasMethod` on an instance of a subclass of a class that `Class~new` made, which takes the
/// package of the interpreter's own library and was given methods of those names with `define`:
/// those methods do not run either.
// spec: native.threads.attach
#[test]
fn thread_interface_members_send_no_message_a_class_new_made_redefines() {
    assert_eq!(
        run("attach-cnew", "cnew.rex"),
        (
            "The M class\nISINSTANCEOF runs\n1\nIsInstanceOf 1\nHasMethod 0\n".to_owned(),
            String::new(),
            0
        )
    );
}

/// The supplier members of the thread interface on the supplier a stream answers, whose class is
/// a subclass of `Supplier` the interpreter's own library defines: that subclass's methods answer,
/// so the members read the stream's lines. Not asked of the oracle, which ends this program with a
/// segmentation fault.
// spec: native.threads.attach
#[test]
fn thread_interface_supplier_members_run_a_library_subclass_s_own_methods() {
    let expected = "\
StreamSupplier 1 line1 1
SupplierAvailable 1
SupplierItem line1
SupplierIndex 1
SupplierAvailable 1
after: 1 line2
";
    assert_eq!(
        run_ours("attach-ssup", "ssup.rex"),
        (expected.to_owned(), String::new(), 0)
    );
}

/// An attached thread's message waits on an event semaphore that a started activity, asleep
/// when the message arrives, posts once its sleep of seven seconds ends. The sleep may end the
/// wait, so stderr says nothing of it however long it takes.
// spec: native.threads.attach
#[test]
fn a_callback_waiting_for_a_sleeping_activitys_post_is_not_reported() {
    assert_eq!(
        run("attach-latepost", "latepost.rex"),
        ("sem seen\ndone\n".to_owned(), String::new(), 0)
    );
}

/// [`a_callback_waiting_for_a_sleeping_activitys_post_is_not_reported`] with the started activity
/// in a call into the extension, which blocks for seven seconds, instead of asleep. The call may
/// end the wait, so the wait is neither refused nor reported, and the post ends it.
// spec: native.threads.attach
#[test]
fn a_callback_waiting_for_a_post_after_a_native_call_is_not_refused() {
    assert_eq!(
        run("attach-latecall", "latecall.rex"),
        ("sem seen\ndone\n".to_owned(), String::new(), 0)
    );
}

/// An attached thread's message waits with `GUARD OFF WHEN` for a flag the main program sets once
/// it has run Rexx code for a second. The main program makes no progress while the message runs,
/// so the wait never ends (Deviation 176). After five seconds stderr says so, and the wait goes on
/// until the run is killed. The oracle says `flag seen`, so it is not asked.
// spec: native.threads.attach.deviation.176
#[test]
fn a_callback_waiting_for_a_flag_the_main_program_sets_waits_for_ever() {
    let (stdout, stderr, status) = run_ours_for("attach-flag", "flag.rex", "10");
    assert_eq!((stdout.as_str(), status), ("", 137));
    assert_stalled_on_flag(&stderr);
}

/// [`a_callback_waiting_for_a_flag_the_main_program_sets_waits_for_ever`] with the message arriving
/// while the main program sleeps for an hour before it sets the flag. The main program makes no
/// progress while the message runs, so its sleep cannot end the wait either, and stderr says so
/// after five seconds, as it does where the main program runs Rexx code.
// spec: native.threads.attach.deviation.176
#[test]
fn a_callback_waiting_for_a_flag_a_sleeping_main_program_sets_is_reported() {
    let (stdout, stderr, status) = run_ours_for("attach-flagsleep", "flagsleep.rex", "10");
    assert_eq!((stdout.as_str(), status), ("", 137));
    assert_stalled_on_flag(&stderr);
}

/// Asserts that `stderr` begins with the line that reports a callback's wait for a `GUARD WHEN`
/// while the main program it stopped makes no progress, written once the wait was five seconds
/// old and less than seven.
fn assert_stalled_on_flag(stderr: &str) {
    let waited = stderr
        .strip_prefix("interrexx: a callback has waited ")
        .and_then(|rest| rest.split_once(" s for a guard's WHEN while the main program, "))
        .filter(|(_, rest)| rest.starts_with("which it stopped, makes no progress\n"))
        .and_then(|(seconds, _)| seconds.parse::<u64>().ok());
    assert!(
        waited.is_some_and(|seconds| (5..7).contains(&seconds)),
        "stderr: {stderr}"
    );
}

/// An attached thread's message waits for the result of the message whose send the main program
/// runs. The wait is refused at once as an inverted wait, as a wait for a guard the main program
/// holds is (Deviation 176). The oracle says `got served`, so it is not asked.
// spec: native.threads.attach.deviation.176
#[test]
fn a_callback_waiting_for_a_result_the_main_program_gives_is_refused() {
    assert_eq!(
        run_ours("attach-reply", "reply.rex"),
        (
            "done\n".to_owned(),
            inverted_wait("a message's completion"),
            120
        )
    );
}

/// An attached thread's message waits on an event semaphore that a second attached thread's
/// message, sent a second later, posts. The first waits on nothing the main program would
/// provide, so it is not refused, and the second message, served on top of it, ends its wait
/// before stderr would say that it waits.
// spec: native.threads.attach
#[test]
fn attached_callbacks_wait_for_each_other() {
    assert_eq!(
        run("attach-coord", "coord.rex"),
        ("sem seen\ndone\n".to_owned(), String::new(), 0)
    );
}

/// The line on stderr that refuses a pinned wait for `what` as an inverted wait.
fn inverted_wait(what: &str) -> String {
    format!(
        "interrexx: a pinned wait for {what} that only an activity pinned below it can end is \
         not implemented\n"
    )
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
// spec: native.threads.attach, native.threads.attach.deviation.175
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
// spec: native.threads.end, native.threads.end.deviation.174
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

/// An embedding host's program waits on an event semaphore that only a thread of the host posts,
/// after it attaches: the host holds the instance, so the wait is not refused, and the routine
/// answers once the thread has posted.
// spec: conc.activities.deviation.20
#[test]
fn a_wait_an_embedding_hosts_thread_ends_is_not_refused() {
    assert_eq!(
        run_driver("hostwait", &["instance"]),
        ("routine posted\n".to_owned(), String::new(), 0)
    );
}

/// An embedding host's routine starts an activity that waits on an event semaphore nothing posts,
/// and the host calls `Terminate`: the end of the run is not refused, and it waits for the
/// activity for ever, as the oracle's does. The run is killed.
// spec: conc.activities.deviation.20
#[test]
fn an_embedding_hosts_terminate_waits_for_a_wait_nothing_ends() {
    // The host may still end the wait, so the end does not refuse it.
    let (stdout, stderr, status) = run_driver_for("hostwait", &["terminate"], "3");
    assert_eq!(
        (stdout.as_str(), status),
        ("routine started\n", 137),
        "{stderr}"
    );
    assert!(!stderr.contains(UNENDING_WAIT), "{stderr}");
}

/// A program `RexxStart` runs waits on an event semaphore nothing in it posts: the wait is not
/// refused, and the halt the host sends later ends it.
// spec: conc.activities.deviation.20
#[test]
fn a_wait_in_a_program_rexx_start_runs_is_not_refused() {
    // The host may still end the wait, so it is not refused.
    let program = fixture("hostwait.rex");
    let program = program.to_str().expect("the fixture's path is UTF-8");
    assert_eq!(
        run_driver("hostwait", &["start", program]),
        ("rc 0 result halted\n".to_owned(), String::new(), 0)
    );
}

/// The line on stderr of a wait refused because nothing left to run can end it (Deviation 20).
const UNENDING_WAIT: &str =
    "interrexx: a wait that nothing left to run can end is not implemented\n";

/// A program that loaded only libraries built into the interpreter, RexxUtil and rxmath, waits on
/// an event semaphore that nothing posts: the wait is refused, rc 120, as no thread of an
/// external library can attach and post it. The oracle waits for ever, so it is not asked.
// spec: conc.activities.deviation.20, util.syseventsem.deviation.20
#[test]
fn a_deadlock_with_only_builtin_libraries_is_refused() {
    assert_eq!(
        run_ours("attach-builtins", "builtins.rex"),
        ("2\nwaiting\n".to_owned(), UNENDING_WAIT.to_owned(), 120)
    );
}

/// After the extension's library loaded, a started method waits for its own message's result
/// before it replies, while the main program has lines left to print: the main program prints them
/// all, and the wait is then refused, rc 120. Under the gate the oracle prints the same and waits
/// for ever.
// spec: conc.activities.deviation.20
#[test]
fn a_wait_on_the_activitys_own_message_is_refused_after_the_others_ran() {
    assert_eq!(
        run_up_to_refusal("attach-early", "early.rex"),
        (
            "in run\nmain 1\nmain 2\nmain 3\nmain done\n".to_owned(),
            UNENDING_WAIT.to_owned(),
            120
        )
    );
}

/// After the extension's library loaded, a started method waits for its own message's result
/// before it replies, and the main program then raises error 41: the error is reported, and the
/// wait is then refused, rc 120. Under the gate the oracle prints the same and waits for ever.
// spec: conc.activities.deviation.20
#[test]
fn a_wait_on_the_activitys_own_message_leaves_the_others_errors_reported() {
    let (stdout, stderr, status) = run_up_to_refusal("attach-early2", "early2.rex");
    assert_eq!(
        (stdout.as_str(), status),
        ("in run\nmain 1\n", 120),
        "{stderr}"
    );
    let error = stderr
        .strip_suffix(UNENDING_WAIT)
        .unwrap_or_else(|| panic!("the run did not end with the refusal: {stderr}"));
    assert!(
        error.starts_with("    10 *-* x = 1 + 'a'\nError 41 running ")
            && error
                .ends_with("Error 41.1:  Nonnumeric value (\"a\") used in arithmetic operation.\n"),
        "{stderr}"
    );
}

/// A program `RexxStart` runs starts a method that waits for its own message's result before it
/// replies, and the main program goes on to its end: the wait is refused once nothing else can
/// run, before the halt the host sends later.
// spec: conc.activities.deviation.20
#[test]
fn a_wait_on_the_activitys_own_message_is_refused_under_a_host() {
    // A host turns off the refusal of other waits, but no native code can end this one.
    let program = fixture("hostself.rex");
    let program = program.to_str().expect("the fixture's path is UTF-8");
    let (stdout, stderr, status) = run_driver("hostwait", &["start", program]);
    assert_eq!(
        (stdout.as_str(), stderr.as_str(), status),
        ("in run\nmain done\nrc 0 result main\n", UNENDING_WAIT, 0)
    );
}

/// An embedding host calls a routine on an instance it created, and the routine sends a message
/// whose method waits for that message's own result before it replies: the wait is refused, the
/// call answers no result with a condition pending, and the refusal is reported once the host
/// calls `Terminate`.
// spec: conc.activities.deviation.20
#[test]
fn a_wait_on_the_activitys_own_message_in_a_hosts_call_is_refused() {
    // The call runs as a callback of the host's thread, and no later callback can end this wait.
    assert_eq!(
        run_driver("hostwait", &["self"]),
        (
            "in run\nroutine (none) condition 1\nterminated\n".to_owned(),
            UNENDING_WAIT.to_owned(),
            0
        )
    );
}

/// As in the hosts-call case, but the thread that created the instance clears the condition
/// before it calls `Terminate`: a refusal is not a Rexx condition, so the refusal is still
/// reported at `Terminate`.
// spec: conc.activities.deviation.20
#[test]
fn a_refusal_the_creating_thread_cleared_is_still_reported_at_terminate() {
    assert_eq!(
        run_driver("hostwait", &["selfclear"]),
        (
            "in run\nroutine (none) condition 1\ncleared condition 0\nterminated\n".to_owned(),
            UNENDING_WAIT.to_owned(),
            0
        )
    );
}

/// A callback an attached thread makes on an instance waits for an event semaphore another
/// attached thread posts only after more than the stall bound, while the thread that created the
/// instance is outside any call: that thread's root call waits for `Terminate` whatever the
/// callback does, so stderr does not report it as stopped.
// spec: native.threads.attach
#[test]
fn a_callback_wait_is_not_reported_as_stalling_the_instances_root_call() {
    let (stdout, stderr, status) = run_driver_for("hostwait", &["stall", "7000"], "60");
    assert_eq!(
        (stdout.as_str(), stderr.as_str(), status),
        ("waiter posted\nterminated\n", "", 0)
    );
}

/// A thread of an embedding host attaches to an instance the host created and calls a routine that
/// sends a message whose method waits for that message's own result before it replies, while the
/// thread that created the instance is outside any call: the wait is refused, the call answers no
/// result with a condition pending, and the refusal is reported once the host calls `Terminate`.
// spec: conc.activities.deviation.20
#[test]
fn a_wait_on_the_activitys_own_message_in_an_attached_hosts_call_is_refused() {
    // The instance's root call stays in flight until `Terminate`, and its end resumes nothing
    // before the callback returns, so it does not hold the refusal back.
    let (stdout, stderr, status) = run_driver("hostwait", &["attached"]);
    // The method's `SAY` reaches stdout once the interpreter's thread idles, which races the
    // attached thread's own line after its call returns, so only the host's lines keep an order.
    let host = stdout.replacen("in run\n", "", 1);
    assert_eq!(
        (
            stdout.contains("in run\n"),
            host.as_str(),
            stderr.as_str(),
            status
        ),
        (
            true,
            "thread routine (none) condition 1\njoined\nterminated\n",
            UNENDING_WAIT,
            0
        )
    );
}

/// As in the attached case, but while the thread that created the instance is inside a routine
/// that waits on an event semaphore: the wait is refused, and the call answers no result with a
/// condition pending. The thread then clears the condition and posts the semaphore, so the host's
/// routine answers, and the refusal is still reported once the host calls `Terminate`.
// spec: conc.activities.deviation.20
#[test]
fn a_wait_on_the_activitys_own_message_in_a_call_nested_in_a_hosts_call_is_refused() {
    // Two lends are out, the host's and the attached thread's, and the root call is counted below
    // them once.
    let (stdout, stderr, status) = run_driver("hostwait", &["nested"]);
    // The method's `SAY` races the attached thread's line, as in the attached case.
    let host = stdout.replacen("in run\n", "", 1);
    assert_eq!(
        (
            stdout.contains("in run\n"),
            host.as_str(),
            stderr.as_str(),
            status
        ),
        (
            true,
            "thread routine (none) condition 1\nroutine posted\nterminated\n",
            UNENDING_WAIT,
            0
        )
    );
}

/// After the extension's library loaded, an attached thread sends `SEND` to a message whose method
/// waits for that message's own result before it replies, while the main program waits for the
/// method to post that it runs: the main program runs to its end, and the wait is refused, rc 120.
/// Under the gate the oracle prints the same and waits for ever.
// spec: conc.activities.deviation.20
#[test]
fn a_wait_on_the_activitys_own_message_in_an_attached_callback_is_refused() {
    // The wait runs in the attached thread's callback, and no later callback can end it.
    assert_eq!(
        run_up_to_refusal("attach-selfsend", "selfsend.rex"),
        (
            "in run\nmain done\n".to_owned(),
            UNENDING_WAIT.to_owned(),
            120
        )
    );
}

/// After the extension's library loaded, a method that has replied waits with `result` and `wait`
/// on the message whose send ran it: the reply settled the result, so both answer at once. The
/// oracle agrees under the gate.
// spec: conc.activities.deviation.20
#[test]
fn a_replied_methods_wait_on_its_own_message_answers() {
    assert_eq!(
        run("attach-replied", "replied.rex"),
        ("result r\nwaited\nsent r\n".to_owned(), String::new(), 0)
    );
}

/// The main program waits on an event semaphore before the thread that posts it has attached:
/// after the extension's library loaded, the wait is not refused, and it ends once the thread's
/// message posts.
// spec: conc.activities.deviation.20, native.threads.attach
#[test]
fn a_wait_ended_by_a_thread_that_attaches_later_succeeds() {
    assert_eq!(
        run("attach-postlater", "postlater.rex"),
        ("sem seen\nposted\n".to_owned(), String::new(), 0)
    );
}

/// `GUARD WHEN` and `Message~result` wait, each before the thread that ends the wait has
/// attached, as a semaphore's wait does: the variable the thread's message sets ends the first,
/// and the message the second thread's message sends ends the second.
// spec: conc.activities.deviation.20, conc.locks.deviation.20
#[test]
fn guard_when_and_message_result_follow_the_switch() {
    assert_eq!(
        run("attach-whenlater", "whenlater.rex"),
        ("flag seen\nanswered\n".to_owned(), String::new(), 0)
    );
}

/// After `RxFuncAdd` loaded a classic library and `RxFuncDrop` dropped its function, a wait that
/// nothing ends is not refused: the run is still going after two seconds, with nothing on stderr,
/// and is killed. A classic library could have started a thread, so dropping it does not bring
/// the refusal back. The oracle waits for ever too, and is not asked.
// spec: conc.activities.deviation.20
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "the run must still be going after a while"
)]
fn the_switch_survives_rxfuncdrop() {
    let dir = build("attach-classic");
    let api = worktree().join("third_party/ooRexx/api");
    let built = Command::new("g++")
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(fixture("classic.cpp"))
        .args(["-shared", "-fPIC", "-O1", "-o"])
        .arg(dir.join("libattachclassic.so"))
        .status()
        .expect("g++ runs, which this test needs to build its fixture");
    assert!(built.success(), "the classic fixture did not build");
    let mut child = Command::new("memcap")
        .arg(MEMORY)
        .args(["timeout", "-s", "KILL", SECONDS])
        .arg(env!("CARGO_BIN_EXE_rexx"))
        .arg(fixture("classic.rex"))
        .current_dir(&dir)
        .env("LD_LIBRARY_PATH", &dir)
        .env("REXX_API_DIR", dir.join("api"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("memcap runs, which every run of this test is under");
    std::thread::sleep(std::time::Duration::from_secs(2));
    let still_running = child.try_wait().expect("the run's status").is_none();
    // `memcap` runs `timeout` as its child, and `timeout` runs `rexx` as its own: killing `rexx`
    // ends all three.
    let children = |pid: u32| {
        fs::read_to_string(format!("/proc/{pid}/task/{pid}/children")).unwrap_or_default()
    };
    for timeout in children(child.id()).split_whitespace() {
        let timeout: u32 = timeout.parse().expect("a process id");
        for rexx in children(timeout).split_whitespace() {
            let _ = Command::new("kill").args(["-KILL", rexx]).status();
        }
    }
    let output = child.wait_with_output().expect("the run's output");
    let _ = Command::new(Path::new(env!("CARGO_BIN_EXE_rexx")).with_file_name("rxapi"))
        .arg("--stop")
        .env("REXX_API_DIR", dir.join("api"))
        .output();
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        still_running,
        "the wait ended: {:?} stdout {stdout:?} stderr {stderr:?}",
        output.status
    );
    assert_eq!(stdout, "0\n1\n0\nwaiting\n");
    assert!(!stderr.contains(UNENDING_WAIT), "stderr {stderr:?}");
}

/// An attached thread's message loops for ever while the main program is in a call into the
/// extension, which calls `Halt` through the instance: the message takes the HALT condition, which
/// it does not trap, so the thread's `CheckCondition` finds SYNTAX, and the main program traps its
/// own HALT once the call returns.
// spec: native.threads.halt
#[test]
fn halt_reaches_a_running_attached_callback() {
    assert_eq!(
        run("halt-running", "haltrun.rex"),
        (
            "main halted\ncondition SYNTAX\n".to_owned(),
            String::new(),
            0
        )
    );
}

/// An attached thread's message calls `HaltThread` on its attachment's thread context, from the
/// thread running the message: the message takes the HALT condition at its next clause, and the
/// thread finds SYNTAX, while the main program, which was not asked, runs on to `main done`.
// spec: native.threads.halt
#[test]
fn halt_thread_halts_only_its_attached_activity() {
    assert_eq!(
        run("halt-thread", "haltself.rex"),
        ("condition SYNTAX\nmain done\n".to_owned(), String::new(), 0)
    );
}

/// A thread attaches and stays idle while the main program calls `Halt` through the instance, then
/// sends a method of three clauses: the halt asked nothing of the idle activity, as the oracle
/// shows, so the method runs to its end and leaves no condition.
// spec: native.threads.halt
#[test]
fn an_idle_attached_activity_drops_a_halt_per_the_probe() {
    assert_eq!(
        run("halt-idle", "idlehalt.rex"),
        (
            "main halted\none\ntwo\nthree\nno condition\n".to_owned(),
            String::new(),
            0
        )
    );
}

/// [`an_idle_attached_activity_drops_a_halt_per_the_probe`] with `SetTrace(1)` instead of `Halt`:
/// the method runs untraced, as the oracle shows, so no trace line names `ticks.cls`. The main
/// program's own trace lines name the program's path, so the oracle is asked for the stdout only.
// spec: native.threads.trace
#[test]
fn an_idle_attached_activity_is_not_traced_per_the_probe() {
    let (stdout, stderr, code) = run_agreeing_on_stdout("trace-idle", "idletrace.rex");
    assert_eq!(
        (stdout.as_str(), code),
        ("one\ntwo\nthree\nno condition\n", 0),
        "{stderr}"
    );
    assert!(stderr.contains("call AtOpen"), "main is traced: {stderr}");
    assert!(
        !stderr.contains("say 'one'"),
        "the method is traced: {stderr}"
    );
}

/// A thread that attaches in the simulation mode is refused, whose timing is real, and the run
/// fails at the next clause boundary. Not asked of the oracle, which has no simulation mode.
// spec: native.threads.attach
#[test]
fn attach_from_a_foreign_thread_is_refused_in_simulation() {
    let ((stdout, stderr, code), _) = run_against(
        "sim-attach",
        "simattach.rex",
        Oracle::Not,
        SECONDS,
        &[("REXX_SWITCH_MODE", "sim")],
    );
    assert!(
        stderr.contains("attaching a thread in the simulation mode is not implemented"),
        "stdout {stdout:?} stderr {stderr:?}"
    );
    assert_ne!(code, 0, "stdout {stdout:?} stderr {stderr:?}");
    assert!(!stdout.contains("ticked"), "stdout {stdout:?}");
}
