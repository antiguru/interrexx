//! The API service one run of a group gets on one side: a directory of the run's own, which this
//! crate's `rexx` is given as `REXX_API_DIR` and the oracle's as `XDG_RUNTIME_DIR`, so a side
//! shares its daemon with neither the other side, another run, nor the user. Dropping it stops the
//! daemon the run started, and removes the directory.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// The longest socket path `sun_path` holds, its NUL left out.
const SOCKET_PATH_MAX: usize = 107;

/// The longest socket name either side makes in the directory: the oracle's
/// `.ooRexx-VERSION-BITS-USER.service`, 31 bytes for `5.3.0`, `64` and a
/// six-letter user, with room for a longer user name.
const SOCKET_NAME_ROOM: usize = 40;

/// Which side a service is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Crate,
    Oracle,
}

impl Side {
    /// The variable that names the service's directory to the side.
    pub fn variable(self) -> &'static str {
        match self {
            Side::Crate => "REXX_API_DIR",
            Side::Oracle => "XDG_RUNTIME_DIR",
        }
    }
}

/// This crate's `rxapi`, `rxqueue` and `rxsubcom`, which a workspace build puts beside `rexx`.
pub fn service_binaries() -> [PathBuf; 3] {
    let rexx = Path::new(env!("CARGO_BIN_EXE_rexx"));
    ["rxapi", "rxqueue", "rxsubcom"].map(|name| {
        let binary = rexx.with_file_name(name);
        assert!(
            binary.is_file(),
            "{} is missing; run `cargo build --workspace --bins` first",
            binary.display()
        );
        binary
    })
}

/// A private service directory, stopped and removed on drop.
pub struct PrivateService {
    dir: PathBuf,
    side: Side,
}

impl PrivateService {
    /// A new, empty directory for `side`, short enough that a socket in it can be bound.
    pub fn new(side: Side) -> PrivateService {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "a-{}-{}{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            match side {
                Side::Crate => 'c',
                Side::Oracle => 'o',
            }
        ));
        assert!(
            dir.as_os_str().len() + 1 + SOCKET_NAME_ROOM <= SOCKET_PATH_MAX,
            "{} leaves no room for a socket name",
            dir.display()
        );
        if dir.exists() {
            std::fs::remove_dir_all(&dir)
                .unwrap_or_else(|e| panic!("cannot remove {}: {e}", dir.display()));
        }
        std::fs::DirBuilder::new()
            .recursive(true)
            .create(&dir)
            .unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
        std::fs::set_permissions(&dir, std::os::unix::fs::PermissionsExt::from_mode(0o700))
            .unwrap_or_else(|e| panic!("cannot restrict {}: {e}", dir.display()));
        PrivateService { dir, side }
    }

    /// The variable and value that name the service to its side.
    pub fn variable(&self) -> (&'static str, &Path) {
        (self.side.variable(), &self.dir)
    }

    /// Stops the side's daemon: this crate's through `rxapi --stop`, and then any `rxapi` whose
    /// environment names this directory, which is how the oracle's is found.
    #[expect(
        clippy::disallowed_methods,
        reason = "this harness waits on a real process"
    )]
    fn stop(&self) {
        if self.side == Side::Crate {
            let [rxapi, _, _] = service_binaries();
            // A service that never started has nothing to stop.
            let _ = std::process::Command::new(rxapi)
                .arg("--stop")
                .env("REXX_API_DIR", &self.dir)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
        let started = Instant::now();
        loop {
            let left = daemons_naming(self.side.variable(), &self.dir);
            if left.is_empty() {
                return;
            }
            let signal = if started.elapsed() < Duration::from_secs(5) {
                rustix::process::Signal::TERM
            } else {
                rustix::process::Signal::KILL
            };
            for pid in left {
                // It may have exited since it was listed.
                let _ = rustix::process::kill_process(pid, signal);
            }
            assert!(
                started.elapsed() < Duration::from_secs(15),
                "an rxapi naming {} outlived SIGKILL",
                self.dir.display()
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

impl Drop for PrivateService {
    fn drop(&mut self) {
        self.stop();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Each live `rxapi` process whose environment holds `variable=dir`.
pub fn daemons_naming(variable: &str, dir: &Path) -> Vec<rustix::process::Pid> {
    let wanted = [
        variable.as_bytes(),
        b"=",
        dir.as_os_str().as_encoded_bytes(),
    ]
    .concat();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let pid: i32 = entry.file_name().to_str()?.parse().ok()?;
            let comm = std::fs::read(entry.path().join("comm")).ok()?;
            if comm.strip_suffix(b"\n").unwrap_or(&comm) != b"rxapi" {
                return None;
            }
            let environ = std::fs::read(entry.path().join("environ")).ok()?;
            environ
                .split(|byte| *byte == 0)
                .any(|entry| entry == wanted.as_slice())
                .then(|| rustix::process::Pid::from_raw(pid))
                .flatten()
        })
        .collect()
}
