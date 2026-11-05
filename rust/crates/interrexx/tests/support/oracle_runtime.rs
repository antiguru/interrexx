//! The `XDG_RUNTIME_DIR` every oracle run of a test process is given: a directory of that
//! process's own, so the oracle's `rxapi`, which every oracle run starts or reaches and which
//! never exits on its own, is a daemon of this process's and never the user's. A watcher started
//! with the first use stops that daemon and removes the directory once the process has ended.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

/// The directory, created with its watcher on first use.
pub fn dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("oracle-runtime-{}", std::process::id()));
        std::fs::DirBuilder::new()
            .recursive(true)
            .create(&dir)
            .expect("a runtime directory of this process's own");
        std::fs::set_permissions(&dir, std::os::unix::fs::PermissionsExt::from_mode(0o700))
            .expect("a private runtime directory");
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
            .arg(std::process::id().to_string())
            .arg(&dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        dir
    })
}
