//! An interpreter with nothing to run does not wake: while every activity
//! is parked on work in flight off the baton, the timer's slices are off,
//! and the process is switched in at most a handful of times (#591).
//!
//! The assertion is a wall-clock window, so it runs in the `wall-clock`
//! group.

#![expect(clippy::disallowed_methods, reason = "this harness times a real run")]

use std::io::{BufRead as _, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Context switches, voluntary and not, of every thread of process `pid`.
fn switches(pid: u32) -> u64 {
    std::fs::read_dir(format!("/proc/{pid}/task"))
        .expect("the process's threads")
        .flatten()
        .map(|task| {
            std::fs::read_to_string(task.path().join("status"))
                .unwrap_or_default()
                .lines()
                .filter(|line| line.contains("ctxt_switches:"))
                .filter_map(|line| line.split_whitespace().nth(1)?.parse::<u64>().ok())
                .sum::<u64>()
        })
        .sum()
}

/// A hundred activities, each parked on a command run off the baton after
/// enough clauses that the clause countdown is visited while commands are
/// in flight, which armed the slices; the main activity then sleeps. Over
/// half a second the process is switched in fewer than 10 times, where a
/// slice every 24 ms was 21.
#[test]
fn parked_activities_with_work_in_flight_do_not_wake_the_timer() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("idle-wakes-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the probe directory");
    let file = dir.join("parked.rex");
    std::fs::write(
        &file,
        "do 100\n  .parked~new~start('park')\nend\ncall SysSleep 1\nsay 'ready'\n\
         call SysSleep 1\n::class parked\n::method park\n  do 20; nop; end\n\
         address system 'sleep 2'\n",
    )
    .expect("the probe");
    let mut child = Command::new(env!("CARGO_BIN_EXE_rexx"))
        .arg(&file)
        .current_dir(&dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("rexx starts");
    let mut out = BufReader::new(child.stdout.take().expect("piped stdout"));
    let mut ready = String::new();
    out.read_line(&mut ready).expect("a line");
    assert_eq!(ready.trim_end(), "ready");
    std::thread::sleep(Duration::from_millis(200));
    let before = switches(child.id());
    std::thread::sleep(Duration::from_millis(500));
    let woken = switches(child.id()) - before;
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(woken < 10, "the idle process was switched in {woken} times");
}
