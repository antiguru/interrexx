//! The `XDG_RUNTIME_DIR` every oracle run of a test process is given, so the oracle's `rxapi`,
//! which every oracle run starts or reaches and which never exits on its own, is a daemon of this
//! test run's and never the user's. Under `cargo test` the directory is the process's own. Under
//! `cargo nextest`, where every test is a process of its own, it is shared by the whole run
//! (keyed by `NEXTEST_RUN_ID`), so a run starts one daemon and not one per test. A watcher
//! started with the first use stops the daemon and removes the directory once the process, or
//! nextest, has ended. A run whose test process is not a direct child of `cargo-nextest` (a
//! runner or wrapper in between) uses the process's own directory.
//!
//! The directory is under `std::env::temp_dir()`, not the target directory: the daemon's socket
//! lives in it, a unix socket path holds at most 107 bytes, and a longer checkout path (a CI
//! workspace) would make the oracle quietly reach another daemon instead of failing.

#![allow(dead_code)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

/// Whether the parent process is `cargo-nextest`.
fn parent_is_nextest() -> bool {
    let parent = std::os::unix::process::parent_id();
    std::fs::read_to_string(format!("/proc/{parent}/comm"))
        .is_ok_and(|comm| comm.trim_end() == "cargo-nextest")
}

/// The name of the directory under the system's temporary directory, and the process whose end
/// stops its daemon: this process, or the nextest that runs it.
///
/// A run's id is a UUID and a socket path is short, so the shared name is a short hash of it.
/// The name itself is short for the same reason.
fn name_and_owner() -> (String, u32, bool) {
    match std::env::var("NEXTEST_RUN_ID") {
        Ok(id) if !id.is_empty() && parent_is_nextest() => {
            let mut hasher = DefaultHasher::new();
            id.hash(&mut hasher);
            (
                format!("orc-n{:010x}", hasher.finish() & 0xff_ffff_ffff),
                std::os::unix::process::parent_id(),
                true,
            )
        }
        _ => (
            format!("orc-{}", std::process::id()),
            std::process::id(),
            false,
        ),
    }
}

/// The longest a unix socket path can be: `sun_path` holds 108 bytes, one of them the NUL.
const SOCKET_PATH_MAX: usize = 107;

/// Panics unless the oracle's `rxapi` socket in `dir` fits in a unix socket path.
///
/// The socket is `.ooRexx-<version>-64-<user>.service` (found by running the oracle and listing
/// its runtime directory). A user name is at most 32 bytes on Linux, so that is assumed when the
/// environment does not name one.
fn assert_socket_path_fits(dir: &Path) {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .map_or(32, |user| user.len().max(1));
    // `.ooRexx-5.3.0-64-` before the user, `.service` after it, and a separator.
    let longest = dir.as_os_str().len() + 1 + ".ooRexx-5.3.0-64-".len() + user + ".service".len();
    assert!(
        longest <= SOCKET_PATH_MAX,
        "the oracle's rxapi socket path in {} would be {longest} bytes, over the \
         {SOCKET_PATH_MAX} a unix socket allows, and the oracle would then reach another daemon; \
         set TMPDIR to a shorter directory",
        dir.display(),
    );
}

/// The directory, created with its watcher on first use: by the process that creates it.
pub fn dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let (name, owner, shared) = name_and_owner();
        let dir = std::env::temp_dir().join(name);
        assert_socket_path_fits(&dir);
        match std::fs::DirBuilder::new().mode(0o700).create(&dir) {
            Ok(()) => {}
            // Another test process of this nextest run created it, and its watcher is running.
            Err(error) if shared && error.kind() == std::io::ErrorKind::AlreadyExists => {
                return dir;
            }
            Err(error) => panic!("a runtime directory for the oracle: {error}"),
        }
        let _ = Command::new("sh")
            .arg("-c")
            .arg(
                "while kill -0 \"$0\" 2>/dev/null; do sleep 1; done; \
                 for p in /proc/[0-9]*; do \
                   [ \"$(cat \"$p/comm\" 2>/dev/null)\" = rxapi ] || continue; \
                   tr '\\0' '\\n' < \"$p/environ\" 2>/dev/null \
                     | grep -qxF \"XDG_RUNTIME_DIR=$1\" && kill \"${p#/proc/}\"; \
                 done; rm -rf \"$1\"",
            )
            .arg(owner.to_string())
            .arg(&dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            // Outside the test's process group: a signal sent to the test does not reach it.
            .process_group(0)
            .spawn();
        dir
    })
}
