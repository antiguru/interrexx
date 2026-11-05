//! `smtp.cls` and `mime.cls` against a scripted SMTP server on loopback:
//! the client's byte stream and its `cmdresponse` are the session
//! `docs/design/extensions.md` records from the oracle, byte for byte.
//!
//! `SMTP_WIRE_ORACLE=1` runs the same session on the oracle
//! (`REXX_ORACLE_ROOT`) as well, which finds this crate's packages through
//! the clean tree's links in its `bin` directory, with an `XDG_RUNTIME_DIR`
//! of its own whose `rxapi` daemon is the caller's to stop.

use std::io::{BufRead as _, BufReader, Write as _};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Stdio};

/// The session's client bytes (extensions-behaviour.md, "smtp.cls").
const CLIENT_BYTES: &str = "EHLO 127.0.0.l\r\n\
AUTH LOGIN\r\n\
dXNlckB4LmNvbQ==\r\n\
c2VjcmV0\r\n\
MAIL FROM:<a@x.com>\r\n\
RCPT TO: <b@y.com>\r\n\
RCPT TO: <c@z.com>\r\n\
DATA\r\n\
From: <a@x.com>\r\n\
To: <b@y.com>,\r\n    <c@z.com>\r\n\
Subject: Hello there\r\n\
MIME-Version: 1.0\r\n\
Content-Type: text/plain\r\n\
\r\n\
Body line\r\n\
..\r\n\
\r\n\
\r\n\
.\r\n\
QUIT\r\n";

/// What the program prints: `send`'s answer, `smtperrno`, then
/// `cmdresponse` line by line, then `logoff`'s answer.
const PRINTED: &str = "connect 0\n\
send 0\n\
smtperrno 354\n\
220 scripted\n\
EHLO 127.0.0.l\n\
250-scripted\n\
250-AUTH LOGIN PLAIN\n\
250 ok\n\
AUTH LOGIN\n\
334 VXNlcm5hbWU6\n\
dXNlckB4LmNvbQ==\n\
334 UGFzc3dvcmQ6\n\
c2VjcmV0\n\
235 ok\n\
MAIL FROM:<a@x.com>\n\
250 ok\n\
RCPT TO: <b@y.com>\n\
250 ok\n\
RCPT TO: <c@z.com>\n\
250 ok\n\
DATA\n\
354 go\n\
250 queued\n\
logoff 0\n";

const PROGRAM: &str = "parse arg port
smtp = .smtp~new
say 'connect' smtp~connect('127.0.0.1:'port, 'user@x.com', 'secret')
msg = .smtpmsg~new
msg~from = '<a@x.com>'
msg~addRecipient('<b@y.com>')
msg~addRecipient('<c@z.com>')
msg~subject = 'Hello there'
part = .mimepart~new
part~addContent('Body line')
part~addContent('.')
msg~content = part
say 'send' smtp~send(msg)
say 'smtperrno' smtp~smtperrno
do line over smtp~cmdresponse
  say line
end
say 'logoff' smtp~logoff
::requires 'smtp.cls'
";

/// The server's reply to one client line, `None` for none.
fn reply(line: &[u8], in_data: &mut bool) -> Option<&'static [u8]> {
    if *in_data {
        if line == b".\r\n" {
            *in_data = false;
            return Some(b"250 queued\r\n");
        }
        return None;
    }
    let reply: &[u8] = match line {
        l if l.starts_with(b"EHLO") => b"250-scripted\r\n250-AUTH LOGIN PLAIN\r\n250 ok\r\n",
        l if l.starts_with(b"AUTH LOGIN") => b"334 VXNlcm5hbWU6\r\n",
        b"dXNlckB4LmNvbQ==\r\n" => b"334 UGFzc3dvcmQ6\r\n",
        b"c2VjcmV0\r\n" => b"235 ok\r\n",
        l if l.starts_with(b"MAIL") || l.starts_with(b"RCPT") => b"250 ok\r\n",
        b"DATA\r\n" => {
            *in_data = true;
            b"354 go\r\n"
        }
        _ => return None,
    };
    Some(reply)
}

/// Runs the session with `rexx`, answering as a scripted server, and answers
/// the bytes the client sent and what the program printed.
fn session(mut rexx: Command) -> (String, String) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
    let port = listener.local_addr().expect("its address").port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("the client connects");
        stream.write_all(b"220 scripted\r\n").expect("the greeting");
        let mut reader = BufReader::new(stream.try_clone().expect("a reader"));
        let mut received = Vec::new();
        let mut in_data = false;
        loop {
            let mut line = Vec::new();
            if reader.read_until(b'\n', &mut line).expect("a read") == 0 {
                break;
            }
            received.extend_from_slice(&line);
            if let Some(answer) = reply(&line, &mut in_data) {
                stream.write_all(answer).expect("a reply");
            }
        }
        String::from_utf8(received).expect("text")
    });
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("smtp-wire");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a run directory");
    std::fs::write(dir.join("smtp.rex"), PROGRAM).expect("the program");
    let output = rexx
        .arg("smtp.rex")
        .arg(port.to_string())
        .current_dir(&dir)
        .stdin(Stdio::null())
        .output()
        .expect("rexx runs");
    let received = server.join().expect("the server ends");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    (
        received,
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

fn crate_rexx() -> Command {
    let api = Path::new(env!("CARGO_TARGET_TMPDIR")).join("smtp-wire-api");
    let _ = std::fs::remove_dir_all(&api);
    std::fs::create_dir_all(&api).expect("a service directory");
    let mut command = Command::new("timeout");
    command
        .args(["-k", "5", "60"])
        .arg(env!("CARGO_BIN_EXE_rexx"))
        .env("REXX_API_DIR", &api)
        .env_remove("LD_LIBRARY_PATH");
    command
}

fn stop_crate_service() {
    let api = Path::new(env!("CARGO_TARGET_TMPDIR")).join("smtp-wire-api");
    let _ = Command::new(Path::new(env!("CARGO_BIN_EXE_rexx")).with_file_name("rxapi"))
        .arg("--stop")
        .env("REXX_API_DIR", &api)
        .output();
}

#[test]
fn smtp_and_mime_send_the_oracles_bytes_to_a_scripted_server() {
    let (received, printed) = session(crate_rexx());
    stop_crate_service();
    assert_eq!(received, CLIENT_BYTES);
    assert_eq!(printed, PRINTED);
    if std::env::var_os("SMTP_WIRE_ORACLE").is_some() {
        let oracle = Path::new(env!("REXX_ORACLE_ROOT"));
        let runtime = Path::new(env!("CARGO_TARGET_TMPDIR")).join("smtp-wire-oracle");
        std::fs::create_dir_all(&runtime).expect("a runtime directory");
        let path = std::env::var_os("PATH").unwrap_or_default();
        let mut directories = vec![oracle.join("bin")];
        directories.extend(std::env::split_paths(&path));
        let mut command = Command::new("timeout");
        command
            .args(["-k", "5", "60"])
            .arg(oracle.join("bin/rexx"))
            .env("LD_LIBRARY_PATH", oracle.join("lib"))
            .env("PATH", std::env::join_paths(directories).expect("a PATH"))
            .env("XDG_RUNTIME_DIR", &runtime);
        let (received, printed) = session(command);
        assert_eq!(received, CLIENT_BYTES, "oracle");
        assert_eq!(printed, PRINTED, "oracle");
    }
}
