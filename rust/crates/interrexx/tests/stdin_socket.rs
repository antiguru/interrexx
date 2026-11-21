//! A socket as the `rexx` binary's standard input, which no specification
//! example can arrange: the binary reports the stream type of the descriptor
//! it was given.

use std::io::Write;
use std::os::unix::net::UnixStream;
use std::process::{Command, Stdio};

/// spec: bif.stream.query.type
///
/// A socket on standard input is `TRANSIENT`, and a supplier over it reads
/// its lines one by one (upstream bug #2173): two suppliers made before any
/// read both start at the first line, and the first one moves on.
#[test]
fn a_socket_on_standard_input_is_a_transient_stream() {
    let dir = std::env::temp_dir().join(format!("interrexx-stdin-socket-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let program = dir.join("socket.rex");
    std::fs::write(
        &program,
        "say stream('STDIN', 'c', 'query streamtype')\n\
         s = .stream~new('STDIN')\n\
         a = s~supplier\n\
         b = s~supplier\n\
         say a~index a~item\n\
         say b~index b~item\n\
         a~next\n\
         say a~index a~item\n",
    )
    .expect("the program is written");
    let (mut ours, theirs) = UnixStream::pair().expect("a socket pair");
    ours.write_all(b"one\ntwo\nthree\n")
        .expect("the lines are sent");
    drop(ours);
    let output = Command::new(env!("CARGO_BIN_EXE_bounded-run"))
        .args(["-k", "5", "30"])
        .arg(env!("CARGO_BIN_EXE_rexx"))
        .arg(&program)
        .current_dir(&dir)
        .stdin(Stdio::from(std::os::fd::OwnedFd::from(theirs)))
        .output()
        .expect("rexx runs");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "TRANSIENT\n1 one\n1 two\n2 three\n",
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}
