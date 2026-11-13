//! `rxqueue [queuename] [/FIFO | /LIFO | /CLEAR]`: copies standard input to a queue, a line per
//! item.

use std::io::{BufRead, Write};
use std::process::ExitCode;

use interrexx_rxapi::client::{Session, connect};
use interrexx_rxapi::message::code::{
    NO_SERVICE, OK, QUEUE_BAD_NAME, QUEUE_NOT_REGISTERED, QUEUE_PRIORITY,
};
use interrexx_rxapi::message::{FIFO, LIFO, Reply, Request};
use interrexx_rxapi::transport::Transport;
use interrexx_rxapi::transport::unix::Unix;
use interrexx_rxapi::utility::{
    QUEUE_ACCESS, QUEUE_EXIST, QUEUE_NAME, QUEUE_NO_MEMORY, QUEUE_NOT_INITIALISED, QUEUE_SIZE,
    QUEUE_SYNTAX, message,
};

/// The longest line kept; the excess is discarded.
const MAX_LINE: usize = 65472;
/// The byte that ends input.
const END_OF_FILE: u8 = 0x1a;
/// `RXQUEUE_SIZE`.
const SIZE: u64 = 2;
/// `RXAPI_MEMFAIL`.
const MEMORY_FAILURE: u64 = 1002;

enum Action {
    Add(u64),
    Clear,
}

fn say(line: &str) {
    let mut out = std::io::stdout();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}

/// The queue name and action, or `None` for a syntax error.
fn parse(arguments: &[Vec<u8>]) -> Option<(Option<Vec<u8>>, Action)> {
    let mut name = None;
    let mut action = None;
    for argument in arguments {
        match argument.first() {
            Some(b'/' | b'-') => {
                if action.is_some() {
                    return None;
                }
                let option = argument[1..].to_ascii_uppercase();
                action = Some(match option.as_slice() {
                    b"FIFO" => Action::Add(FIFO),
                    b"LIFO" => Action::Add(LIFO),
                    b"CLEAR" => Action::Clear,
                    _ => return None,
                });
            }
            _ => {
                if name.is_some() {
                    return None;
                }
                name = Some(argument.clone());
            }
        }
    }
    Some((name, action.unwrap_or(Action::Add(FIFO))))
}

/// The message for `status` on the queue `name`.
fn failure_text(status: u64, name: &[u8]) -> String {
    let sub = match status {
        QUEUE_BAD_NAME => QUEUE_NAME,
        QUEUE_NOT_REGISTERED => QUEUE_EXIST,
        QUEUE_PRIORITY => QUEUE_ACCESS,
        SIZE => QUEUE_SIZE,
        NO_SERVICE => QUEUE_NOT_INITIALISED,
        MEMORY_FAILURE => QUEUE_NO_MEMORY,
        _ => QUEUE_SYNTAX,
    };
    message(sub, &[name])
}

/// Reports `status` for `name` and answers the exit status.
fn fail(status: u64, name: &[u8]) -> ExitCode {
    say(&failure_text(status, name));
    ExitCode::from((status % 256) as u8)
}

fn status(reply: Result<Reply, u64>) -> u64 {
    match reply {
        Ok(Reply::Status(status) | Reply::QueueCount { status, .. }) => status,
        Ok(_) => OK,
        Err(status) => status,
    }
}

/// Splits input into lines as the command does, handing each to `line`, which answers
/// whether to go on.
pub fn split(mut input: impl BufRead, mut line: impl FnMut(Vec<u8>) -> bool) {
    let mut current = Vec::new();
    let mut after_cr = false;
    loop {
        let chunk = match input.fill_buf() {
            Ok([]) => break,
            Ok(chunk) => chunk.to_vec(),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        };
        input.consume(chunk.len());
        for byte in chunk {
            if after_cr {
                after_cr = false;
                // The byte after a lone `\r` starts the next line untested, and a NUL there is
                // lost.
                if byte != b'\n' && byte != 0 {
                    current.push(byte);
                }
                continue;
            }
            match byte {
                b'\n' | b'\r' => {
                    after_cr = byte == b'\r';
                    current.truncate(MAX_LINE);
                    if !line(std::mem::take(&mut current)) {
                        return;
                    }
                }
                END_OF_FILE => {
                    if current.is_empty() {
                        line(Vec::new());
                    }
                    return;
                }
                _ => current.push(byte),
            }
        }
    }
    if !current.is_empty() {
        current.truncate(MAX_LINE);
        line(current);
    }
}

fn main() -> ExitCode {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let arguments: Vec<Vec<u8>> = std::env::args_os()
        .skip(1)
        .map(OsStringExt::into_vec)
        .collect();
    let Some((name, action)) = parse(&arguments) else {
        say(&message(QUEUE_SYNTAX, &[]));
        return ExitCode::SUCCESS;
    };
    let name = name
        .or_else(|| std::env::var_os("RXQUEUE").map(|value| value.as_bytes().to_vec()))
        .unwrap_or_else(|| b"SESSION".to_vec());
    let session = Session::of_process();
    // Deviation 93: with no session handed down, the session queue would be this process's own,
    // gone when it disconnects, so its lines would reach nobody.
    if name.eq_ignore_ascii_case(b"SESSION") && !session.inherited() {
        say("There is no session queue: name a queue, or run rxqueue from a Rexx program.");
        return ExitCode::from(QUEUE_NOT_REGISTERED as u8);
    }
    let client: Unix = match connect() {
        Ok(client) => client,
        Err(status) => return fail(status, &name),
    };
    let _ = client.call(Request::CreateSessionQueue {
        session: session.handle(),
    });
    let call = |request| client.call(request).map_err(|error| error.status());
    let queried = status(call(Request::QueryQueue {
        session: session.handle(),
        name: Some(name.clone()),
    }));
    if queried != OK {
        return fail(queried, &name);
    }
    let flag = match action {
        Action::Clear => {
            let cleared = status(call(Request::ClearQueue {
                session: session.handle(),
                name: Some(name.clone()),
            }));
            return match cleared {
                OK => ExitCode::SUCCESS,
                failed => fail(failed, &name),
            };
        }
        Action::Add(flag) => flag,
    };
    let mut failed = OK;
    split(std::io::stdin().lock(), |line| {
        failed = status(call(Request::AddQueue {
            session: session.handle(),
            name: Some(name.clone()),
            data: line,
            flag,
            time: Default::default(),
        }));
        failed == OK
    });
    match failed {
        OK => ExitCode::SUCCESS,
        failed => fail(failed, &name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(input: &[u8]) -> Vec<Vec<u8>> {
        let mut found = Vec::new();
        split(input, |line| {
            found.push(line);
            true
        });
        found
    }

    #[test]
    fn terminators_and_the_end_byte_split_as_described() {
        let list = |items: &[&[u8]]| items.iter().map(|item| item.to_vec()).collect::<Vec<_>>();
        assert_eq!(lines(b"a\nb\r\nc\rd"), list(&[b"a", b"b", b"c", b"d"]));
        assert_eq!(lines(b"a\n\nb\n"), list(&[b"a", b"", b"b"]));
        assert_eq!(lines(b""), list(&[]));
        assert_eq!(lines(b"a\0b\n"), list(&[b"a\0b"]));
        assert_eq!(lines(b"a\nb\x1ac\nd\n"), list(&[b"a"]));
        assert_eq!(lines(b"ab\x1a"), list(&[]));
        assert_eq!(lines(b"\x1aab\n"), list(&[b""]));
        assert_eq!(lines(b"\r\r\n\n"), list(&[b"", b"\r", b""]));
        assert_eq!(lines(b"\r\0a\n"), list(&[b"", b"a"]));
    }

    #[test]
    fn each_failure_prints_its_catalogue_message() {
        use interrexx_inventory::errors::{lookup, substitute};
        let expected = |sub, inserts: &[&[u8]]| {
            let entry = lookup(999, sub).expect("catalogued");
            String::from_utf8_lossy(&substitute(entry.text, inserts)).into_owned()
        };
        let cases: [(u64, u16); 7] = [
            (QUEUE_BAD_NAME, 7),
            (QUEUE_NOT_REGISTERED, 10),
            (QUEUE_PRIORITY, 8),
            (SIZE, 5),
            (NO_SERVICE, 4),
            (MEMORY_FAILURE, 6),
            (u64::MAX, 16),
        ];
        for (status, sub) in cases {
            assert_eq!(
                failure_text(status, b"qq-probe"),
                expected(sub, &[b"qq-probe"]),
                "status {status}"
            );
        }
        // The names reach the two messages that carry one.
        assert!(failure_text(QUEUE_BAD_NAME, b"qq-probe").contains("qq-probe"));
        assert!(failure_text(QUEUE_NOT_REGISTERED, b"qq-probe").contains("qq-probe"));
    }

    #[test]
    fn a_long_line_keeps_its_first_bytes() {
        let mut input = vec![b'x'; MAX_LINE + 10];
        input.push(b'\n');
        input.extend_from_slice(&[b'y'; MAX_LINE]);
        let found = lines(&input);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0], vec![b'x'; MAX_LINE]);
        assert_eq!(found[1], vec![b'y'; MAX_LINE]);
    }

    #[test]
    fn options_are_caseless_and_single() {
        let words = |items: &[&str]| {
            items
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect::<Vec<_>>()
        };
        assert!(matches!(
            parse(&words(&["q", "-lifo"])),
            Some((Some(_), Action::Add(LIFO)))
        ));
        assert!(matches!(
            parse(&words(&["/Clear"])),
            Some((None, Action::Clear))
        ));
        assert!(matches!(
            parse(&words(&[])),
            Some((None, Action::Add(FIFO)))
        ));
        assert!(parse(&words(&["/fifo", "/fifo"])).is_none());
        assert!(parse(&words(&["/"])).is_none());
        assert!(parse(&words(&["a", "b"])).is_none());
        assert!(parse(&words(&["/bogus"])).is_none());
    }
}
