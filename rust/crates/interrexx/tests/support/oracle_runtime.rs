//! The `XDG_RUNTIME_DIR` every oracle run of a test process is given, so the oracle's `rxapi`,
//! which every oracle run starts or reaches and which never exits on its own, is a daemon of this
//! test run's and never the user's. Under `cargo test` the directory is the process's own. Under
//! `cargo nextest`, where every test is a process of its own, it is shared by the whole run
//! (keyed by `NEXTEST_RUN_ID`), so a run starts one daemon and not one per test. A watcher
//! started with the first use stops the daemon and removes the directory once the process, or
//! nextest, has ended.

#![allow(dead_code)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

/// The name of the directory under the target's temporary directory, and the process whose end
/// stops its daemon: this process, or the nextest that runs it.
///
/// A run's id is a UUID and a socket path is short, so the shared name is a short hash of it.
fn name_and_owner() -> (String, u32) {
    match std::env::var("NEXTEST_RUN_ID") {
        Ok(id) if !id.is_empty() => {
            let mut hasher = DefaultHasher::new();
            id.hash(&mut hasher);
            (
                format!("oracle-runtime-n{:010x}", hasher.finish() & 0xff_ffff_ffff),
                std::os::unix::process::parent_id(),
            )
        }
        _ => (
            format!("oracle-runtime-{}", std::process::id()),
            std::process::id(),
        ),
    }
}

/// The directory, created with its watcher on first use: by the process that creates it.
pub fn dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let (name, owner) = name_and_owner();
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        std::fs::create_dir_all(dir.parent().expect("a parent")).expect("the temporary directory");
        match std::fs::DirBuilder::new().mode(0o700).create(&dir) {
            Ok(()) => {}
            // Another test process of this nextest run created it, and its watcher is running.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return dir,
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
            .spawn();
        dir
    })
}
