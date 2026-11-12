//! `bounded-run`, a test helper: the time limit the tests put around a program they start.
//! It is not part of the interpreter and nothing outside the tests runs it.
//!
//! The tests used to start programs through the system's `timeout`, whose
//! answers differ between coreutils implementations: a Rust `timeout` refuses
//! an argument that is not UTF-8 and reports 124 where GNU's reports 137 for a
//! run its `--foreground -s KILL` ended. This helper takes the same command
//! line as the part of `timeout` the tests use, so that what a test asserts
//! about a kill at the bound does not depend on the machine:
//!
//! ```text
//! bounded-run [--foreground] [-s SIGNAL] [-k DURATION] DURATION COMMAND [ARG]...
//! ```
//!
//! The command runs with this process's standard streams and environment.
//! Arguments are taken as bytes, so any of them may be non-UTF-8. When the
//! command ends by itself its status is the answer, 128 plus the signal number
//! if a signal ended it. When `DURATION` passes first, SIGNAL (TERM unless
//! named) is sent to the command, and with `-k` SIGKILL follows if it is
//! still running after that `DURATION`. The answer is then 137 if SIGKILL was
//! sent and ended the command, else 124. Without `--foreground` the command
//! leads its own process group and the signals go to the whole group; with it
//! the command shares this process's group and only the command is signalled.
//! A failure to start the command answers 127 (not found) or 126, and a
//! malformed command line 125.

#![expect(
    clippy::disallowed_methods,
    reason = "a time limit around a child process reads the real clock and sleeps"
)]

use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::{Child, Command, ExitCode};
use std::thread::sleep;
use std::time::{Duration, Instant};

use rustix::process::{Pid, Signal, kill_process, kill_process_group};

const USAGE_STATUS: u8 = 125;
const TIMED_OUT_STATUS: u8 = 124;
const KILLED_STATUS: u8 = 137;

fn usage(why: &str) -> ExitCode {
    eprintln!("bounded-run: {why}");
    ExitCode::from(USAGE_STATUS)
}

fn parse_duration(text: &OsStr) -> Option<Duration> {
    let text = text.to_str()?;
    let (number, scale) = match text.chars().last()? {
        's' => (&text[..text.len() - 1], 1.0),
        'm' => (&text[..text.len() - 1], 60.0),
        'h' => (&text[..text.len() - 1], 3600.0),
        'd' => (&text[..text.len() - 1], 86400.0),
        _ => (text, 1.0),
    };
    let seconds: f64 = number.parse().ok()?;
    if !seconds.is_finite() || seconds < 0.0 {
        return None;
    }
    // A bound that overflows `Duration` is no bound.
    Duration::try_from_secs_f64(seconds * scale)
        .ok()
        .or(Some(Duration::MAX / 4))
}

fn parse_signal(text: &OsStr) -> Option<Signal> {
    let text = text.to_str()?;
    if let Ok(number) = text.parse::<i32>() {
        return Signal::from_named_raw(number);
    }
    let name = text.strip_prefix("SIG").unwrap_or(text);
    Some(match name {
        "HUP" => Signal::HUP,
        "INT" => Signal::INT,
        "QUIT" => Signal::QUIT,
        "KILL" => Signal::KILL,
        "USR1" => Signal::USR1,
        "USR2" => Signal::USR2,
        "ALRM" => Signal::ALARM,
        "TERM" => Signal::TERM,
        "CONT" => Signal::CONT,
        _ => return None,
    })
}

fn send(child: &Child, group: bool, signal: Signal) {
    let Some(pid) = i32::try_from(child.id()).ok().and_then(Pid::from_raw) else {
        return;
    };
    // The command may have ended between the check and the signal.
    let _ = if group {
        kill_process_group(pid, signal)
    } else {
        kill_process(pid, signal)
    };
    // A stopped command does not act on a signal until it is continued.
    if signal != Signal::KILL && signal != Signal::CONT {
        let _ = if group {
            kill_process_group(pid, Signal::CONT)
        } else {
            kill_process(pid, Signal::CONT)
        };
    }
}

/// Waits for `child` until `deadline`; the status, or `None` at the deadline.
fn wait_until(child: &mut Child, deadline: Instant) -> Option<std::process::ExitStatus> {
    let mut nap = Duration::from_micros(200);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => {}
            Err(_) => return None,
        }
        let now = Instant::now();
        if now >= deadline {
            return None;
        }
        sleep(nap.min(deadline - now));
        nap = (nap * 2).min(Duration::from_millis(5));
    }
}

fn status_code(status: std::process::ExitStatus) -> u8 {
    match (status.code(), status.signal()) {
        (Some(code), _) => u8::try_from(code & 0xff).unwrap_or(1),
        (None, Some(signal)) => u8::try_from(128 + signal).unwrap_or(255),
        (None, None) => 1,
    }
}

fn main() -> ExitCode {
    let mut args: Vec<OsString> = std::env::args_os().skip(1).collect();
    args.reverse();
    let mut foreground = false;
    let mut signal = Signal::TERM;
    let mut kill_after = None;
    let duration = loop {
        let Some(arg) = args.pop() else {
            return usage("missing operand");
        };
        match arg.as_bytes() {
            b"--foreground" => foreground = true,
            b"-s" => {
                let Some(value) = args.pop() else {
                    return usage("option requires an argument -- 's'");
                };
                let Some(parsed) = parse_signal(&value) else {
                    return usage("invalid signal");
                };
                signal = parsed;
            }
            b"-k" => {
                let Some(value) = args.pop() else {
                    return usage("option requires an argument -- 'k'");
                };
                let Some(parsed) = parse_duration(&value) else {
                    return usage("invalid time interval");
                };
                kill_after = Some(parsed);
            }
            _ => match parse_duration(&arg) {
                Some(duration) => break duration,
                None => return usage("invalid time interval"),
            },
        }
    };
    let Some(program) = args.pop() else {
        return usage("missing command");
    };
    args.reverse();
    let mut command = Command::new(&program);
    command.args(&args);
    if !foreground {
        command.process_group(0);
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            eprintln!("bounded-run: failed to run command: {error}");
            let missing = error.kind() == std::io::ErrorKind::NotFound;
            return ExitCode::from(if missing { 127 } else { 126 });
        }
    };
    let group = !foreground;
    let start = Instant::now();
    if let Some(status) = wait_until(&mut child, start + duration) {
        return ExitCode::from(status_code(status));
    }
    send(&child, group, signal);
    let mut killed = signal == Signal::KILL;
    let status = match kill_after {
        Some(grace) => match wait_until(&mut child, Instant::now() + grace) {
            Some(status) => Some(status),
            None => {
                send(&child, group, Signal::KILL);
                killed = true;
                child.wait().ok()
            }
        },
        None => child.wait().ok(),
    };
    let ended_by_kill = status.is_some_and(|status| status.signal() == Some(9));
    ExitCode::from(if killed && ended_by_kill {
        KILLED_STATUS
    } else {
        TIMED_OUT_STATUS
    })
}
