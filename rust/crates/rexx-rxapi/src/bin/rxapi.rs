//! `rxapi`: the service, and `rxapi --stop` to stop it. Clients start it on demand.

use std::io::Write;
use std::process::ExitCode;
use std::time::Duration;

use rexx_rxapi::daemon::{Exit, Options, serve};
use rexx_rxapi::message::wire::PROTOCOL_VERSION;
use rexx_rxapi::transport::unix::{Unix, prepare_dir, service_dir};

/// The status `rxapi.testGroup` sees when another instance holds the lock.
const LOCKED: u8 = 13;
const IDLE_GRACE: Duration = Duration::from_secs(5);

fn say(line: &str) {
    let _ = writeln!(std::io::stdout(), "{line}");
}

/// `REXX_API_IDLE_MS`, when set, replaces the grace for a daemon nobody connected to.
fn idle_grace() -> Duration {
    std::env::var("REXX_API_IDLE_MS")
        .ok()
        .and_then(|value| value.parse().ok())
        .map_or(IDLE_GRACE, Duration::from_millis)
}

fn main() -> ExitCode {
    let mut dry_run = false;
    let mut stop = false;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--dryrun" => {
                dry_run = true;
                say("rxapi: dryrun");
            }
            "--stop" => stop = true,
            _ => say("rxapi: no args allowed"),
        }
    }
    let dir = service_dir();
    if stop {
        return match Unix::stop(&dir) {
            Ok(_) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("rxapi: {error}");
                ExitCode::FAILURE
            }
        };
    }
    let dir = match prepare_dir(&dir) {
        Ok(dir) => dir,
        Err(error) => {
            eprintln!("rxapi: {error}");
            return ExitCode::FAILURE;
        }
    };
    // Fails harmlessly for a process group leader, which a shell's foreground job is.
    let _ = rustix::process::setsid();
    let options = Options {
        dir: &dir,
        version: PROTOCOL_VERSION,
        dry_run,
        idle_grace: idle_grace(),
        report: &say,
    };
    match serve(&options) {
        Ok(Exit::Stopped) => ExitCode::SUCCESS,
        Ok(Exit::LockHeld | Exit::PathOccupied) => ExitCode::from(LOCKED),
        Err(error) => {
            eprintln!("rxapi: {error}");
            ExitCode::FAILURE
        }
    }
}
