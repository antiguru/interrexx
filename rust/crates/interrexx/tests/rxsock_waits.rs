//! An `rxsock` call that would block waits off the baton: a pool thread
//! `poll`s its sockets, so the call answers as soon as they are ready, a
//! timed wait ends at its deadline, and an idle wait takes no CPU. Each
//! witness runs `rexx` in a subprocess and talks to it over loopback.
//!
//! Every test here depends on a wall-clock boundary, so they run in the
//! `wall-clock` group, one at a time.

#![expect(clippy::disallowed_methods, reason = "this harness times real runs")]

use std::io::{BufReader, Read as _, Write as _};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// A program listening on a loopback port it prints, once, on its own line.
const LISTEN: &str = "s = SockSocket('AF_INET', 'SOCK_STREAM', 0)\n\
     a.family = 'AF_INET'; a.port = 0; a.addr = '127.0.0.1'\n\
     call SockBind s, 'a.'\ncall SockListen s, 1\ncall SockGetSockName s, 'n.'\n\
     say n.port\n";

/// `source` started under `rexx` in a directory of its own, with its
/// standard output piped.
fn start(name: &str, source: &str) -> (Child, PathBuf) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("rxsock-waits-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the probe directory");
    let file = dir.join(format!("{name}.rex"));
    std::fs::write(&file, format!("{source}::requires 'rxsock' LIBRARY\n")).expect("the probe");
    let child = Command::new(env!("CARGO_BIN_EXE_rexx"))
        .arg(&file)
        .current_dir(&dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("rexx starts");
    (child, dir)
}

/// The next line `child` writes, without its newline.
fn line(reader: &mut impl std::io::BufRead) -> String {
    let mut text = String::new();
    reader.read_line(&mut text).expect("a line");
    text.trim_end().to_string()
}

/// Ends `child` and removes `dir`.
fn finish(mut child: Child, dir: &PathBuf) {
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(dir);
}

/// Context switches, voluntary and not, of every thread of process `pid`.
fn switches(pid: u32) -> u64 {
    let tasks = format!("/proc/{pid}/task");
    std::fs::read_dir(&tasks)
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

/// A `SockRecv` blocked on a connected socket answers within a millisecond
/// of the peer's send, not at the next of a series of polls: the median
/// round trip of an echo, each send made a few milliseconds after the last
/// reply, is under 1 ms.
#[test]
fn a_blocked_receive_answers_as_the_data_arrives() {
    let (mut child, dir) = start(
        "echo",
        &format!(
            "{LISTEN}c = SockAccept(s)\n\
             do forever\n  if SockRecv(c, 'd', 100) <= 0 then leave\n  call SockSend c, d\nend\n"
        ),
    );
    let mut out = BufReader::new(child.stdout.take().expect("piped stdout"));
    let port: u16 = line(&mut out).parse().expect("a port");
    let mut peer = TcpStream::connect(("127.0.0.1", port)).expect("a connection");
    peer.set_nodelay(true).expect("no delay");
    let mut trips = Vec::new();
    let mut reply = [0u8; 8];
    for _ in 0..41 {
        std::thread::sleep(Duration::from_millis(3));
        let sent = Instant::now();
        peer.write_all(b"x").expect("the send");
        peer.read_exact(&mut reply[..1]).expect("the echo");
        trips.push(sent.elapsed());
    }
    drop(peer);
    finish(child, &dir);
    trips.sort();
    let median = trips[trips.len() / 2];
    assert!(
        median < Duration::from_millis(1),
        "the median round trip was {median:?}: {trips:?}"
    );
}

/// `SockSelect` with a timeout and nothing ready answers 0 once the timeout
/// has passed, and promptly after it; the stem keeps its count, as the
/// oracle's does.
#[test]
fn a_select_with_a_timeout_ends_at_the_timeout() {
    let (mut child, dir) = start(
        "select",
        &format!(
            "{LISTEN}do 5\n  r.0 = 1; r.1 = s\n  t = .datetime~new\n\
             n = SockSelect('r.', '', '', 0.2)\n\
             say n r.0 (.datetime~new - t)~microseconds\nend\n"
        ),
    );
    let mut out = BufReader::new(child.stdout.take().expect("piped stdout"));
    let _port = line(&mut out);
    for _ in 0..5 {
        let answer = line(&mut out);
        let words: Vec<&str> = answer.split_whitespace().collect();
        assert_eq!(words[..2], ["0", "1"], "{answer}");
        let took: u64 = words[2].parse().expect("microseconds");
        assert!(
            (200_000..240_000).contains(&took),
            "a 0.2 s select took {took} us"
        );
    }
    let mut rest = String::new();
    out.read_to_string(&mut rest).expect("the rest");
    finish(child, &dir);
}

/// A program blocked in `SockRecv` uses no CPU while it waits: over half a
/// second, its threads are switched in a handful of times at most, where a
/// wait that polled every 5 ms would be woken a hundred times.
#[test]
fn a_blocked_receive_does_not_wake_while_idle() {
    let (mut child, dir) = start(
        "idle",
        &format!(
            "{LISTEN}c = SockSocket('AF_INET', 'SOCK_STREAM', 0)\n\
             call SockConnect c, 'n.'\nsay 'ready'\nr = SockRecv(c, 'd', 10)\n"
        ),
    );
    let mut out = BufReader::new(child.stdout.take().expect("piped stdout"));
    let _port = line(&mut out);
    assert_eq!(line(&mut out), "ready");
    std::thread::sleep(Duration::from_millis(200));
    let before = switches(child.id());
    std::thread::sleep(Duration::from_millis(500));
    let woken = switches(child.id()) - before;
    finish(child, &dir);
    assert!(woken < 20, "the idle wait was switched in {woken} times");
}

/// A `SockSelect` whose exception set holds an unconnected socket, which
/// `poll` reports as hung up whatever it is asked, waits out its timeout
/// without spinning: a hang-up answers no wait for priority data, and the
/// wait must not wake on it again and again.
#[test]
fn a_hung_up_exception_entry_does_not_spin_a_select() {
    let (mut child, dir) = start(
        "hangup",
        "s = SockSocket('AF_INET', 'SOCK_STREAM', 0)\nsay 'ready'\n\
         e.0 = 1; e.1 = s\nn = SockSelect('', '', 'e.', 1)\nsay 'select' n e.0\n",
    );
    let mut out = BufReader::new(child.stdout.take().expect("piped stdout"));
    assert_eq!(line(&mut out), "ready");
    std::thread::sleep(Duration::from_millis(200));
    let before = switches(child.id());
    std::thread::sleep(Duration::from_millis(500));
    let woken = switches(child.id()) - before;
    assert_eq!(line(&mut out), "select 0 1");
    finish(child, &dir);
    assert!(woken < 100, "the select was switched in {woken} times");
}
