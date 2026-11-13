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

/// The threads of process `pid`.
#[cfg(target_os = "linux")]
fn threads(pid: u32) -> usize {
    std::fs::read_dir(format!("/proc/{pid}/task"))
        .expect("the process's threads")
        .count()
}

/// `count` activities each blocked in `SockRecv` on a datagram socket of its
/// own, started once the program has printed `base` and reported by `ready`
/// once all are in their receive; run under a descriptor limit that has room
/// for them.
#[cfg(target_os = "linux")]
fn start_receivers(name: &str, count: usize) -> (Child, PathBuf) {
    let source = "parse arg n\nsay 'base'\ncall SysSleep 0.3\n\
         do i = 1 to n\n  s = SockSocket('AF_INET', 'SOCK_DGRAM', 0)\n\
         a.family = 'AF_INET'; a.port = 0; a.addr = '127.0.0.1'\n\
         if SockBind(s, 'a.') \\= 0 then do; say 'bind' i errno; exit 1; end\n\
         .receiver~new~start('receive', s)\nend\n\
         do until .receiver~waiting = n; call SysSleep 0.05; end\n\
         call SysSleep 0.3\nsay 'ready'\ncall SysSleep 60\n\
         ::class receiver\n::method waiting class attribute\n\
         ::method arrived class\n  expose waiting\n  if \\var('waiting') then waiting = 0\n  waiting += 1\n\
         ::method receive\n  use arg s\n  .receiver~arrived\n  r = SockRecv(s, 'd', 10)\n";
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("rxsock-waits-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the probe directory");
    let file = dir.join(format!("{name}.rex"));
    std::fs::write(&file, format!("{source}::requires 'rxsock' LIBRARY\n")).expect("the probe");
    let child = Command::new("sh")
        .arg("-c")
        .arg("ulimit -n 8192 && exec \"$0\" \"$@\"")
        .arg(env!("CARGO_BIN_EXE_rexx"))
        .arg(&file)
        .arg(count.to_string())
        .current_dir(&dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("rexx starts");
    (child, dir)
}

/// A thousand activities blocked in `SockRecv` at once are served by the
/// one reactor thread the `rxsock` library started when it was loaded, not
/// a thread each: the process has no more threads with them waiting than
/// it had before the first. Linux only: elsewhere there is no reactor yet,
/// and each wait takes a pool thread.
#[cfg(target_os = "linux")]
#[test]
fn a_thousand_blocked_receives_share_one_thread() {
    let (mut child, dir) = start_receivers("thousand", 1000);
    let mut out = BufReader::new(child.stdout.take().expect("piped stdout"));
    assert_eq!(line(&mut out), "base");
    let base = threads(child.id());
    assert_eq!(line(&mut out), "ready");
    let waiting = threads(child.id());
    finish(child, &dir);
    assert!(
        waiting <= base + 1,
        "{base} threads before the receives and {waiting} with them waiting"
    );
}

/// Two activities waiting on one socket -- a `SockSelect` for it to be
/// readable and a `SockRecv` peeking at it -- are both served when data
/// arrives: the reactor keeps one registration per descriptor with both
/// waiters on it.
#[test]
fn two_waits_on_one_socket_both_end() {
    let (mut child, dir) = start(
        "shared",
        &format!(
            "{LISTEN}c = SockAccept(s)\n\
             .both~new~start('selects', c)\n.both~new~start('peeks', c)\n\
             call SysSleep 0.3\nsay 'waiting'\n\
             ::class both\n\
             ::method selects\n  use arg c\n  r.0 = 1; r.1 = c\n\
               say 'select' SockSelect('r.', '', '') r.0\n\
             ::method peeks\n  use arg c\n\
               say 'peek' SockRecv(c, 'd', 100, 'MSG_PEEK') d\n"
        ),
    );
    let mut out = BufReader::new(child.stdout.take().expect("piped stdout"));
    let port: u16 = line(&mut out).parse().expect("a port");
    let mut peer = TcpStream::connect(("127.0.0.1", port)).expect("a connection");
    assert_eq!(line(&mut out), "waiting");
    peer.write_all(b"hello").expect("the send");
    let mut ended = [line(&mut out), line(&mut out)];
    ended.sort();
    drop(peer);
    finish(child, &dir);
    assert_eq!(ended, ["peek 5 hello", "select 1 1"]);
}

/// A socket closed while an activity waits on it, and its number taken by a
/// new socket another activity then receives on, does not wedge the
/// reactor: the receive answers when its datagram arrives, and the first
/// wait still ends by its timeout. The receive waits in a `SockSelect`
/// with a 3 s timeout first, so that a wedged reactor fails the test rather
/// than hanging it.
#[test]
fn a_reused_descriptor_number_is_waited_on_afresh() {
    let (mut child, dir) = start(
        "reuse",
        "a.family = 'AF_INET'; a.port = 0; a.addr = '127.0.0.1'\n\
         old = SockSocket('AF_INET', 'SOCK_DGRAM', 0)\ncall SockBind old, 'a.'\n\
         .waits~new~start('selects', old)\ncall SysSleep 0.2\ncall SockClose old\n\
         new = SockSocket('AF_INET', 'SOCK_DGRAM', 0)\ncall SockBind new, 'a.'\n\
         call SockGetSockName new, 'n.'\nsay (new = old) n.port\n\
         .waits~new~start('receives', new)\n\
         ::class waits\n\
         ::method selects\n  use arg s\n  r.0 = 1; r.1 = s\n\
           say 'select' SockSelect('r.', '', '', 2)\n\
         ::method receives\n  use arg s\n  q.0 = 1; q.1 = s\n\
           if SockSelect('q.', '', '', 3) = 0 then say 'recv timed out'\n\
           else say 'recv' SockRecv(s, 'd', 100) d\n",
    );
    let mut out = BufReader::new(child.stdout.take().expect("piped stdout"));
    let reused = line(&mut out);
    let words: Vec<&str> = reused.split_whitespace().collect();
    assert_eq!(
        words[0], "1",
        "the new socket took the old one's number: {reused}"
    );
    let port: u16 = words[1].parse().expect("a port");
    std::thread::sleep(Duration::from_millis(100));
    let peer = std::net::UdpSocket::bind("127.0.0.1:0").expect("a socket");
    let sent = Instant::now();
    peer.send_to(b"datagram", ("127.0.0.1", port))
        .expect("the send");
    let mut lines = Vec::new();
    while lines.len() < 2 {
        let next = line(&mut out);
        if next.starts_with("recv") {
            assert!(
                sent.elapsed() < Duration::from_millis(500),
                "the receive answered {:?} after its datagram",
                sent.elapsed()
            );
        }
        lines.push(next);
    }
    finish(child, &dir);
    assert!(lines.contains(&"recv 8 datagram".to_string()), "{lines:?}");
    assert!(
        lines.iter().any(|line| line.starts_with("select")),
        "{lines:?}"
    );
}

/// A halt reaches every activity waiting on a socket: with a hundred
/// receives and a `SockSelect` with no timeout all waiting at once, SIGINT
/// ends the program within half a second.
#[test]
fn a_halt_ends_many_socket_waits_promptly() {
    let (mut child, dir) = start(
        "halt",
        "do 100\n  s = SockSocket('AF_INET', 'SOCK_DGRAM', 0)\n\
         a.family = 'AF_INET'; a.port = 0; a.addr = '127.0.0.1'\n\
         call SockBind s, 'a.'\n  .halted~new~start('receive', s)\nend\n\
         r.0 = 1; r.1 = s\ncall SysSleep 0.3\nsay 'ready'\n\
         n = SockSelect('r.', '', '')\nsay 'after' n\n\
         ::class halted\n::method receive\n  use arg s\n  r = SockRecv(s, 'd', 10)\n",
    );
    let mut out = BufReader::new(child.stdout.take().expect("piped stdout"));
    assert_eq!(line(&mut out), "ready");
    std::thread::sleep(Duration::from_millis(200));
    let sent = Instant::now();
    let signalled = Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .expect("kill runs");
    assert!(signalled.success());
    let mut ended = None;
    while sent.elapsed() < Duration::from_secs(5) {
        if let Some(status) = child.try_wait().expect("the child's state") {
            ended = Some((status, sent.elapsed()));
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let mut rest = String::new();
    let _ = out.read_to_string(&mut rest);
    finish(child, &dir);
    let (status, took) = ended.expect("the program ended after the halt");
    assert!(!status.success(), "a halted program fails: {status:?}");
    assert!(!rest.contains("after"), "the select answered: {rest}");
    assert!(took < Duration::from_millis(500), "the halt took {took:?}");
}
