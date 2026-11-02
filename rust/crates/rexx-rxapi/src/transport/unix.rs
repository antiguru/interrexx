//! The Unix socket client, and the directory the service lives in.

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Read};
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use rustix::net::{SendFlags, send};

use super::{Transport, TransportError};
use crate::message::wire::{self, PROTOCOL_VERSION};
use crate::message::{Reply, Request};

/// How long a client keeps trying to reach a service it may have started.
const START_TIMEOUT: Duration = Duration::from_secs(5);
/// How often a client still without a service tries starting one.
const RESPAWN_INTERVAL: Duration = Duration::from_millis(250);
/// How long a connected peer may take to answer the handshake.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(2);

/// The socket of `version` in `dir`.
pub fn socket_path(dir: &Path, version: u32) -> PathBuf {
    dir.join(format!("rxapi.{version}.sock"))
}

/// The lock file of `version` in `dir`.
pub fn lock_path(dir: &Path, version: u32) -> PathBuf {
    dir.join(format!("rxapi.{version}.lock"))
}

/// The service directory for this user: `REXX_API_DIR`, else `$XDG_RUNTIME_DIR/interrexx`, else a
/// directory of this user's under the temp directory.
pub fn service_dir() -> PathBuf {
    choose_dir(
        std::env::var_os("REXX_API_DIR"),
        std::env::var_os("XDG_RUNTIME_DIR"),
        &std::env::temp_dir(),
        rustix::process::getuid().as_raw(),
    )
}

fn choose_dir(api: Option<OsString>, xdg: Option<OsString>, temp: &Path, uid: u32) -> PathBuf {
    let set = |value: Option<OsString>| value.filter(|value| !value.is_empty());
    if let Some(api) = set(api) {
        return PathBuf::from(api);
    }
    if let Some(xdg) = set(xdg) {
        return Path::new(&xdg).join("interrexx");
    }
    temp.join(format!("interrexx-{uid}"))
}

/// Creates `dir` if needed and insists it is a `0700` directory of this user, answering its
/// absolute path.
pub fn prepare_dir(dir: &Path) -> io::Result<PathBuf> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)?;
    let metadata = fs::symlink_metadata(dir)?;
    let ours = metadata.uid() == rustix::process::getuid().as_raw();
    if !metadata.is_dir() || !ours || metadata.mode() & 0o077 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("{} is not private to this user", dir.display()),
        ));
    }
    fs::canonicalize(dir)
}

/// `rxapi` beside this executable, else on `PATH`.
fn find_daemon() -> Result<PathBuf, TransportError> {
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join("rxapi")));
    let on_path = std::env::var_os("PATH").unwrap_or_default();
    beside
        .into_iter()
        .chain(std::env::split_paths(&on_path).map(|dir| dir.join("rxapi")))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| TransportError::Unavailable("no rxapi executable found".to_owned()))
}

fn start_daemon(daemon: &Path, dir: &Path) -> Result<(), TransportError> {
    let child = Command::new(daemon)
        .env("REXX_API_DIR", dir)
        .current_dir("/")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| unavailable(&error))?;
    // Reaps a starter that lost the race, and otherwise waits out the daemon.
    std::thread::spawn(move || {
        let mut child = child;
        let _ = child.wait();
    });
    Ok(())
}

fn unavailable(error: &io::Error) -> TransportError {
    TransportError::Unavailable(error.to_string())
}

#[expect(clippy::disallowed_methods, reason = "a real start's real deadline")]
fn now() -> Instant {
    Instant::now()
}

#[expect(clippy::disallowed_methods, reason = "a real start's real backoff")]
fn pause() {
    std::thread::sleep(Duration::from_millis(5));
}

/// Writes all of `bytes` without raising `SIGPIPE` in the embedding process.
fn send_all(stream: &UnixStream, mut bytes: &[u8]) -> io::Result<()> {
    while !bytes.is_empty() {
        match send(stream, bytes, SendFlags::NOSIGNAL) {
            Ok(sent) => bytes = &bytes[sent..],
            Err(rustix::io::Errno::INTR) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

/// Calls in flight, by call number.
#[derive(Default)]
struct Calls {
    next: u64,
    waiting: HashMap<u64, mpsc::Sender<Reply>>,
    closed: bool,
}

type SharedCalls = Arc<Mutex<Calls>>;

fn lock_calls(calls: &Mutex<Calls>) -> MutexGuard<'_, Calls> {
    calls.lock().expect("the critical sections cannot panic")
}

/// Delivers replies until the connection ends, then fails every call in flight.
fn read_replies(mut stream: UnixStream, calls: SharedCalls) {
    while let Ok(Some((call, body))) = wire::read_frame(&mut stream) {
        let Ok(reply) = wire::decode_reply(&body) else {
            break;
        };
        if let Some(waiter) = lock_calls(&calls).waiting.remove(&call) {
            let _ = waiter.send(reply);
        }
    }
    let mut calls = lock_calls(&calls);
    calls.closed = true;
    calls.waiting.clear();
}

/// Why a connect attempt failed.
enum ConnectFailure {
    /// Nothing answered, or what did went away.
    Absent(io::Error),
    Refused {
        theirs: u32,
    },
}

/// One connection to a daemon.
///
/// Concurrent calls share it, tagged by call number, so its one identity owns every registration
/// the process makes. When the connection fails it stays failed.
pub struct Unix {
    stream: Mutex<UnixStream>,
    calls: SharedCalls,
    reader: Option<JoinHandle<()>>,
}

impl Unix {
    /// Connects to the service in `dir`, starting it if nothing answers.
    pub fn connect_or_start(dir: &Path) -> Result<Unix, TransportError> {
        match Unix::connect_version(dir, PROTOCOL_VERSION) {
            Err(TransportError::Unavailable(_)) => {
                Unix::connect_or_start_with(dir, &find_daemon()?)
            }
            result => result,
        }
    }

    /// As `connect_or_start`, with the executable to start.
    pub fn connect_or_start_with(dir: &Path, daemon: &Path) -> Result<Unix, TransportError> {
        let dir = prepare_dir(dir).map_err(|error| unavailable(&error))?;
        let deadline = now() + START_TIMEOUT;
        let mut next_start = now();
        loop {
            let error = match Unix::attempt(&dir, PROTOCOL_VERSION) {
                Ok(connected) => return Ok(connected),
                Err(ConnectFailure::Refused { theirs }) => {
                    return Err(TransportError::Refused {
                        ours: PROTOCOL_VERSION,
                        theirs,
                    });
                }
                Err(ConnectFailure::Absent(error)) => error,
            };
            let time = now();
            if time >= deadline {
                return Err(unavailable(&error));
            }
            if time >= next_start {
                start_daemon(daemon, &dir)?;
                next_start = time + RESPAWN_INTERVAL;
            }
            pause();
        }
    }

    /// Connects to a running service speaking `version`, starting nothing.
    pub fn connect_version(dir: &Path, version: u32) -> Result<Unix, TransportError> {
        Unix::attempt(dir, version).map_err(|failure| match failure {
            ConnectFailure::Absent(error) => unavailable(&error),
            ConnectFailure::Refused { theirs } => TransportError::Refused {
                ours: version,
                theirs,
            },
        })
    }

    fn attempt(dir: &Path, version: u32) -> Result<Unix, ConnectFailure> {
        let stream =
            UnixStream::connect(socket_path(dir, version)).map_err(ConnectFailure::Absent)?;
        handshake(&stream, version)?;
        let reader_stream = stream.try_clone().map_err(ConnectFailure::Absent)?;
        let calls = SharedCalls::default();
        let reader = {
            let calls = Arc::clone(&calls);
            std::thread::spawn(move || read_replies(reader_stream, calls))
        };
        Ok(Unix {
            stream: Mutex::new(stream),
            calls,
            reader: Some(reader),
        })
    }

    /// Stops the service in `dir`, answering whether one was running. Returns once it closed
    /// this connection, by which time it has removed its socket.
    pub fn stop(dir: &Path) -> Result<bool, TransportError> {
        Unix::stop_version(dir, PROTOCOL_VERSION)
    }

    /// As `stop`, for the service speaking `version`.
    pub fn stop_version(dir: &Path, version: u32) -> Result<bool, TransportError> {
        let mut client = match Unix::attempt(dir, version) {
            Ok(client) => client,
            Err(ConnectFailure::Absent(_)) => return Ok(false),
            Err(ConnectFailure::Refused { theirs }) => {
                return Err(TransportError::Refused {
                    ours: version,
                    theirs,
                });
            }
        };
        client
            .send(0, &wire::encode_stop())
            .map_err(|error| unavailable(&error))?;
        if let Some(reader) = client.reader.take() {
            let _ = reader.join();
        }
        Ok(true)
    }

    fn send(&self, call: u64, body: &[u8]) -> io::Result<()> {
        let stream = self.stream.lock().expect("sends cannot panic");
        send_all(&stream, &wire::frame(call, body))
    }
}

fn handshake(stream: &UnixStream, version: u32) -> Result<(), ConnectFailure> {
    let absent = ConnectFailure::Absent;
    send_all(stream, &wire::hello(version)).map_err(absent)?;
    stream
        .set_read_timeout(Some(HANDSHAKE_TIMEOUT))
        .map_err(absent)?;
    let mut welcome = [0; wire::WELCOME_LENGTH];
    (&*stream).read_exact(&mut welcome).map_err(absent)?;
    stream.set_read_timeout(None).map_err(absent)?;
    match wire::read_welcome(&welcome) {
        (true, _) => Ok(()),
        (false, theirs) => Err(ConnectFailure::Refused { theirs }),
    }
}

impl Transport for Unix {
    fn call(&self, request: Request) -> Result<Reply, TransportError> {
        let lost = || TransportError::Unavailable("the connection is closed".to_owned());
        let (sender, receiver) = mpsc::channel();
        let call = {
            let mut calls = lock_calls(&self.calls);
            if calls.closed {
                return Err(lost());
            }
            calls.next += 1;
            let call = calls.next;
            calls.waiting.insert(call, sender);
            call
        };
        if self.send(call, &wire::encode_request(&request)).is_err() {
            lock_calls(&self.calls).waiting.remove(&call);
            return Err(lost());
        }
        receiver.recv().map_err(|_| lost())
    }
}

impl Drop for Unix {
    fn drop(&mut self) {
        if let Ok(stream) = self.stream.get_mut() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(api: Option<&str>, xdg: Option<&str>) -> PathBuf {
        choose_dir(api.map(Into::into), xdg.map(Into::into), Path::new("/t"), 7)
    }

    #[test]
    fn the_directory_comes_from_the_override_then_xdg_then_temp() {
        assert_eq!(dir(Some("/a"), Some("/x")), Path::new("/a"));
        assert_eq!(dir(None, Some("/x")), Path::new("/x/interrexx"));
        assert_eq!(dir(None, None), Path::new("/t/interrexx-7"));
        assert_eq!(dir(Some(""), Some("")), Path::new("/t/interrexx-7"));
    }

    #[test]
    fn socket_names_carry_the_version() {
        assert_ne!(
            socket_path(Path::new("/d"), 1),
            socket_path(Path::new("/d"), 2)
        );
        assert_ne!(lock_path(Path::new("/d"), 1), lock_path(Path::new("/d"), 2));
    }

    #[test]
    fn a_directory_others_can_reach_is_refused() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!("rxapi-unit-{}", std::process::id()));
        let private = root.join("private");
        fs::create_dir_all(&root).expect("the temp directory");

        let prepared = prepare_dir(&private).expect("a new directory is private");
        assert_eq!(
            fs::metadata(&prepared).expect("metadata").mode() & 0o777,
            0o700
        );

        fs::set_permissions(&private, fs::Permissions::from_mode(0o755)).expect("chmod");
        let error = prepare_dir(&private).expect_err("a shared directory");
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);

        fs::remove_dir_all(&root).expect("cleanup");
    }
}
