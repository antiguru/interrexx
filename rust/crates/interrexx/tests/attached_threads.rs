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
    run_against(name, program, asked, SECONDS)
}

/// [`run`] where the oracle is not asked even under `REXX_CORPUS_GATE`: it aborts the program,
/// or what the program reports is a property of our memory use that the oracle does not share.
fn run_ours(name: &str, program: &str) -> Answer {
    run_against(name, program, Oracle::Not, SECONDS).0
}

/// [`run_ours`], killed after `seconds` rather than [`SECONDS`]: the program waits for ever.
fn run_ours_for(name: &str, program: &str, seconds: &str) -> Answer {
    run_against(name, program, Oracle::Not, seconds).0
}

/// [`run`] where the oracle, under `REXX_CORPUS_GATE`, must answer the same stderr and exit status
/// only: its stdout differs as a Deviation says.
fn run_agreeing_on_status(name: &str, program: &str) -> Answer {
    let asked = if gate_mode() {
        Oracle::Status
    } else {
        Oracle::Not
    };
    run_against(name, program, asked, SECONDS).0
}

/// [`run`] where the oracle, under `REXX_CORPUS_GATE`, must answer the same stdout only: its
/// stderr and exit status differ as a Deviation says.
fn run_agreeing_on_stdout(name: &str, program: &str) -> Answer {
    let asked = if gate_mode() {
        Oracle::Stdout
    } else {
        Oracle::Not
    };
    run_against(name, program, asked, SECONDS).0
}

/// What of a run the oracle is asked to answer the same.
#[derive(Clone, Copy, PartialEq)]
enum Oracle {
    Not,
    Whole,
    Status,
    Stdout,
}

/// [`run_timed`], asking the oracle as `oracle` says, our run killed after `seconds`.
#[expect(
    clippy::disallowed_methods,
    reason = "the run's wall time is what an idle thread must not stretch"
)]
fn run_against(
    name: &str,
    program: &str,
    oracle: Oracle,
    seconds: &str,
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
            Oracle::Not => {}
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
    for run in 0..10 {
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
/// (Deviation 174). The oracle says so first, so it is asked for the exit status only.
// spec: native.threads.end, native.threads.attach.deviation.174
#[test]
fn a_running_callback_holds_program_end() {
    assert_eq!(
        run_agreeing_on_status("attach-slow", "slow.rex"),
        ("slow done\nmain ends\n".to_owned(), String::new(), 0)
    );
}

/// A message an attached thread sent sleeps a second while the main program loops on
/// `SysSleep 0.05`: the main program makes no progress meanwhile, its sleeps included, and counts
/// fewer than five (Deviation 174). The oracle runs it meanwhile and says `main ran`, so it is asked
/// for the exit status only.
// spec: native.threads.attach.deviation.174
#[test]
fn a_callback_runs_while_main_runs_rexx() {
    assert_eq!(
        run_agreeing_on_status("attach-nap", "nap.rex"),
        ("main stopped\n".to_owned(), String::new(), 0)
    );
}

/// A message an attached thread sends while the main program is in a call into the extension,
/// which blocks for a second, runs during the call and sleeps two seconds. The call returns
/// first, but the main program goes on only once the message has finished (Deviation 174). The
/// oracle goes on at the return, so it is asked for the exit status only.
// spec: native.threads.attach.deviation.174
#[test]
fn a_callback_during_a_native_call_holds_its_return() {
    assert_eq!(
        run_agreeing_on_status("attach-pause", "pause.rex"),
        ("slow done\nmain returned\n".to_owned(), String::new(), 0)
    );
}

/// A message an attached thread sends while the main program waits on an event semaphore runs
/// during the wait and posts the semaphore, then sleeps a second. The main program wakes only once
/// the message has finished (Deviation 174). The oracle wakes it at the post, so it is asked for
/// the exit status only.
// spec: native.threads.attach.deviation.174
#[test]
fn a_callback_during_a_wait_holds_its_wake() {
    assert_eq!(
        run_agreeing_on_status("attach-semwait", "semwait.rex"),
        ("callback ends\nsem seen\n".to_owned(), String::new(), 0)
    );
}

/// A second attached thread's message arrives while the first's sleeps, and sleeps as long: it runs
/// on top of the sleeping one, and the first, whose sleep ends first, finishes only after the
/// second (Deviation 174). The oracle finishes them in the order
/// their sleeps end (`a b`), so it is asked for the exit status only.
// spec: native.threads.attach.deviation.174
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
/// runs, can end it: the wait is refused at once as an inverted wait (Deviation 174), and the
/// program ends with rc 120. The oracle says `ticked 1`, so it is not asked.
// spec: native.threads.attach.deviation.174
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
/// inverted wait (Deviation 174), the line is said once, and the program ends with rc 120. The
/// oracle's read waits for the method to return, so it is asked for the stdout only.
// spec: native.threads.attach.deviation.174
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
/// the read is refused as an inverted wait (Deviation 174), the line is said once, and the
/// thread's next send, a `TICK` to a second object, runs once. The oracle's read waits for the
/// method to return, so it is asked for the stdout only.
// spec: native.threads.attach.deviation.174
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
/// guarded `PUT`, while the main program, stopped by the put, runs a guarded method of the same
/// array. Only a send gives the interpreter back and sends again, so the `PUT`'s wait is refused
/// as an inverted wait (Deviation 174), and the thread's next send, a `TICK` to a second object,
/// runs once. The oracle waits for the method to return, so it is asked for the stdout only.
// spec: native.threads.attach.deviation.174
#[test]
fn an_array_put_from_an_attached_thread_leaves_its_next_send_to_run_once() {
    assert_eq!(
        run_agreeing_on_stdout("attach-put", "put.rex"),
        ("ticked 1\n".to_owned(), inverted_wait("a guard"), 120)
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

/// An attached thread's message waits with `GUARD OFF WHEN` for a flag the main program sets once
/// it has run Rexx code for a second. The main program makes no progress while the message runs,
/// so the wait never ends (Deviation 174). After five seconds stderr says so, and the wait goes on
/// until the run is killed. The oracle says `flag seen`, so it is not asked.
// spec: native.threads.attach.deviation.174
#[test]
fn a_callback_waiting_for_a_flag_the_main_program_sets_waits_for_ever() {
    let (stdout, stderr, status) = run_ours_for("attach-flag", "flag.rex", "10");
    assert_eq!((stdout.as_str(), status), ("", 137));
    assert!(
        stderr.starts_with(
            "interrexx: a callback has waited 5 s for a guard's WHEN while the main program, \
             which it stopped, makes no progress\n"
        ),
        "stderr: {stderr}"
    );
}

/// An attached thread's message waits for the result of the message whose send the main program
/// runs. The wait is refused at once as an inverted wait, as a wait for a guard the main program
/// holds is (Deviation 174). The oracle says `got served`, so it is not asked.
// spec: native.threads.attach.deviation.174
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
// spec: native.threads.attach, native.threads.attach.deviation.173
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
// spec: native.threads.end, native.threads.end.deviation.172
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
