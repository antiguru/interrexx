//! What a differential test that runs one `rexx` process per case shares:
//! the scratch directory, the private API service directory, the command for
//! either side (this crate's `rexx` or the oracle's), and the service's
//! stop. Each case gets an API service of its own, so a registration made in
//! one case, or a failed run, is not seen by the next.

#![allow(dead_code)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The scratch directory one run of a test binary works under: short,
/// because the API service's socket lives in it, and named for the process,
/// so that two runs sharing a target directory do not empty each other's
/// cases.
pub fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

/// Removes the scratch directory [`scratch`] made for `name`, once every case
/// has run. A test calls it only after it has passed, so the directory of a
/// failing run stays for diagnosis.
pub fn remove_scratch(name: &str) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(dir);
}

/// A fresh service directory `api<case>` under `root`, private to the owner:
/// the service refuses a directory that others can use, and says so.
pub fn service_dir(root: &Path, case: usize) -> PathBuf {
    let api = root.join(format!("api{case}"));
    let _ = std::fs::remove_dir_all(&api);
    std::fs::create_dir_all(&api).expect("a service directory");
    std::fs::set_permissions(&api, PermissionsExt::from_mode(0o700))
        .expect("a private service directory");
    api
}

/// The oracle's installation root.
pub fn oracle_root() -> &'static Path {
    Path::new(env!("REXX_ORACLE_ROOT"))
}

/// `rexx` under a time limit, so that a call that waits for ever fails the
/// case rather than the run.
pub fn bounded(rexx: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bounded-run"));
    command.args(["-k", "5", "60"]).arg(rexx);
    command
}

/// The command that runs a program on the oracle (`oracle`) or on this
/// crate's `rexx`, with the API service in `api`. `rexx` is the base command,
/// plain or [`bounded`].
pub fn with_service(mut command: Command, oracle: bool, api: &Path) -> Command {
    if oracle {
        command
            .env("LD_LIBRARY_PATH", oracle_root().join("lib"))
            .env("XDG_RUNTIME_DIR", api);
    } else {
        command
            .env("REXX_API_DIR", api)
            .env_remove("LD_LIBRARY_PATH");
    }
    command
}

/// The oracle's `rexx`.
pub fn oracle_rexx() -> PathBuf {
    oracle_root().join("bin/rexx")
}

/// This crate's `rexx`.
pub fn own_rexx() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_rexx"))
}

/// Stops the service this crate's `rexx` started in `api` and removes the
/// directory. The oracle's daemons are the recorder's to stop.
pub fn stop_own_service(api: &Path) {
    let _ = Command::new(own_rexx().with_file_name("rxapi"))
        .arg("--stop")
        .env("REXX_API_DIR", api)
        .output();
    let _ = std::fs::remove_dir_all(api);
}
