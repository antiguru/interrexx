//! Private API directories for test and benchmark runs. On Linux the service
//! can also find and stop a detached daemon that inherited its directory.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(target_os = "linux")]
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

/// A private service directory, stopped and removed on drop.
#[derive(Debug)]
pub struct PrivateService {
    dir: PathBuf,
    side: Side,
    crate_rxapi: Option<PathBuf>,
}

impl PrivateService {
    /// Keeps the inherited runtime on hosts where the benchmark cannot stop
    /// an on-demand daemon. A known interrexx stop binary also works outside Linux.
    pub fn for_benchmark(
        side: Side,
        crate_rxapi: Option<PathBuf>,
    ) -> std::io::Result<Option<Self>> {
        if cfg!(target_os = "linux") || crate_rxapi.is_some() {
            Self::new(side, crate_rxapi).map(Some)
        } else {
            Ok(None)
        }
    }

    /// A new, empty directory in the system temporary directory, short enough
    /// to hold the service's socket. The optional binary is interrexx's rxapi.
    pub fn new(side: Side, crate_rxapi: Option<PathBuf>) -> std::io::Result<Self> {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::path::absolute(std::env::temp_dir())?;
        loop {
            let dir = root.join(format!(
                "rx-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed),
            ));
            if dir.as_os_str().len() + 1 + SOCKET_NAME_ROOM > SOCKET_PATH_MAX {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("{} leaves no room for an API socket name", dir.display()),
                ));
            }
            let mut builder = std::fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&dir) {
                Ok(()) => {
                    return Ok(Self {
                        dir,
                        side,
                        crate_rxapi,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
    }

    /// The variable and value that name the service to its side.
    pub fn variable(&self) -> (&'static str, &Path) {
        (self.side.variable(), &self.dir)
    }

    /// Uses this service and prevents a child inheriting the caller's queue.
    pub fn configure(&self, command: &mut Command) {
        let (variable, dir) = self.variable();
        command
            .env(variable, dir)
            .env_remove("RXQUEUE")
            .env_remove("RXQUEUESESSION");
    }

    /// Stops interrexx's service through its own binary where available.
    fn stop(&self) {
        if let Some(rxapi) = &self.crate_rxapi {
            let mut command = Command::new(rxapi);
            self.configure(&mut command);
            let _ = command
                .arg("--stop")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
        #[cfg(target_os = "linux")]
        self.stop_detached();
    }

    /// Stops the oracle's daemon and any interrexx daemon without a stop binary.
    #[cfg(target_os = "linux")]
    #[expect(
        clippy::disallowed_methods,
        reason = "daemon cleanup waits on a real process"
    )]
    fn stop_detached(&self) {
        let started = Instant::now();
        loop {
            let left = daemons_naming(&self.dir);
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
            if started.elapsed() >= Duration::from_secs(15) {
                eprintln!("an rxapi naming {} outlived SIGKILL", self.dir.display());
                return;
            }
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

/// Each live `rxapi` process whose environment names the private directory.
#[cfg(target_os = "linux")]
fn daemons_naming(dir: &Path) -> Vec<rustix::process::Pid> {
    let names = [Side::Crate, Side::Oracle].map(|side| {
        [
            side.variable().as_bytes(),
            b"=",
            dir.as_os_str().as_encoded_bytes(),
        ]
        .concat()
    });
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
                .any(|entry| names.iter().any(|name| entry == name.as_slice()))
                .then(|| rustix::process::Pid::from_raw(pid))
                .flatten()
        })
        .collect()
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Stdio};

    #[expect(
        clippy::disallowed_methods,
        reason = "the fixture waits for the kernel's process state"
    )]
    fn wait_until<T>(mut ready: impl FnMut() -> Option<T>) -> Option<T> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(result) = ready() {
                return Some(result);
            }
            if Instant::now() >= deadline {
                return None;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn stopped_by_cleanup(status: Option<ExitStatus>) -> bool {
        use rustix::process::Signal;
        status
            .and_then(|status| status.signal())
            .is_some_and(|signal| [Signal::TERM.as_raw(), Signal::KILL.as_raw()].contains(&signal))
    }

    #[test]
    fn dropping_a_service_stops_only_its_daemon_and_removes_its_directory() {
        const CHILD: &str = "INTERREXX_PRIVATE_SERVICE_TEST_DAEMON";
        if std::env::var_os(CHILD).is_some() {
            // The parent keeps this pipe open. If it exits unexpectedly, EOF
            // lets the fixture child exit too.
            std::io::stdin().read_line(&mut String::new()).unwrap();
            return;
        }

        let first = PrivateService::new(Side::Oracle, None).unwrap();
        let second = PrivateService::new(Side::Oracle, None).unwrap();
        let (_, first_dir) = first.variable();
        let (_, second_dir) = second.variable();
        let first_dir = first_dir.to_path_buf();
        let second_dir = second_dir.to_path_buf();

        let rxapi = second_dir.join("rxapi");
        // Linux reports the copied executable's name in /proc. Reusing the
        // test binary avoids differences in multicall coreutils dispatch.
        std::fs::copy(std::env::current_exe().unwrap(), &rxapi).unwrap();
        let daemon = |service: &PrivateService| {
            let mut command = Command::new(&rxapi);
            service.configure(&mut command);
            command
                .env(CHILD, "1")
                .args([
                    "--exact",
                    "private_service::tests::dropping_a_service_stops_only_its_daemon_and_removes_its_directory",
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .spawn()
                .unwrap()
        };
        let mut first_daemon = daemon(&first);
        let mut second_daemon = daemon(&second);
        let first_pid = rustix::process::Pid::from_child(&first_daemon);
        let second_pid = rustix::process::Pid::from_child(&second_daemon);
        let daemons_ready = wait_until(|| {
            (daemons_naming(&first_dir).contains(&first_pid)
                && daemons_naming(&second_dir).contains(&second_pid))
            .then_some(())
        })
        .is_some();

        drop(first);
        let first_status = wait_until(|| first_daemon.try_wait().unwrap());
        let second_status = second_daemon.try_wait().unwrap();
        drop(second);
        let second_cleanup = wait_until(|| second_daemon.try_wait().unwrap());
        // Keep the test bounded even if daemon cleanup regresses.
        for mut child in [first_daemon, second_daemon] {
            let _ = child.kill();
            let _ = child.wait();
        }

        assert!(daemons_ready, "both daemons must be visible before cleanup");
        assert!(
            stopped_by_cleanup(first_status),
            "the first daemon did not terminate in cleanup: {first_status:?}"
        );
        assert!(
            second_status.is_none(),
            "the second daemon was stopped early"
        );
        assert!(
            stopped_by_cleanup(second_cleanup),
            "the second daemon did not terminate in cleanup: {second_cleanup:?}"
        );
        assert!(!first_dir.exists());
        assert!(!second_dir.exists());
    }
}
