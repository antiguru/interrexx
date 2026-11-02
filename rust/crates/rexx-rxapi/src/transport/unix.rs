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

use rustix::fs::{Access, access};
use rustix::net::sockopt::socket_peercred;
use rustix::net::{SendFlags, send};

use super::{CancelToken, Transport, TransportError};
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
/// directory of this user's under the temp directory. A candidate whose socket path the platform
/// cannot bind is passed over, and a passed-over `REXX_API_DIR` is reported once on standard
/// error.
pub fn service_dir() -> PathBuf {
    let (dir, skipped) = choose_dir(
        std::env::var_os("REXX_API_DIR"),
        std::env::var_os("XDG_RUNTIME_DIR"),
        &std::env::temp_dir(),
        rustix::process::getuid().as_raw(),
    );
    if let Some(skipped) = skipped {
        static WARNED: std::sync::Once = std::sync::Once::new();
        WARNED.call_once(|| {
            use std::io::Write;
            let _ = writeln!(
                io::stderr(),
                "rxapi: REXX_API_DIR {} is not used: its socket path is too long",
                skipped.display()
            );
        });
    }
    dir
}

/// Whether a socket in `dir` has a path the platform can bind.
fn socket_fits(dir: &Path) -> bool {
    let socket = socket_path(dir, PROTOCOL_VERSION);
    let socket = std::path::absolute(&socket).unwrap_or(socket);
    rustix::net::SocketAddrUnix::new(socket.as_path()).is_ok()
}

/// The directory, and `REXX_API_DIR`'s value where it was set and passed over.
fn choose_dir(
    api: Option<OsString>,
    xdg: Option<OsString>,
    temp: &Path,
    uid: u32,
) -> (PathBuf, Option<PathBuf>) {
    let set = |value: Option<OsString>| value.filter(|value| !value.is_empty()).map(PathBuf::from);
    let api = set(api);
    let fallback = temp.join(format!("interrexx-{uid}"));
    let chosen = api
        .iter()
        .cloned()
        .chain(set(xdg).map(|xdg| xdg.join("interrexx")))
        .find(|dir| socket_fits(dir))
        .unwrap_or(fallback);
    let skipped = api.filter(|api| *api != chosen);
    (chosen, skipped)
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

fn own_uid() -> u32 {
    rustix::process::getuid().as_raw()
}

/// The user whose process is at the other end of `stream`.
pub fn peer_uid(stream: &UnixStream) -> io::Result<u32> {
    Ok(socket_peercred(stream)?.uid.as_raw())
}

/// The first `rxapi` that can run, beside this executable, else on `PATH`. The answer is absolute
/// because the daemon starts in `/`.
fn find_daemon() -> Result<PathBuf, TransportError> {
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join("rxapi")));
    let on_path = std::env::var_os("PATH").unwrap_or_default();
    let here = std::env::current_dir().map_err(|error| unavailable(&error))?;
    beside
        .into_iter()
        .chain(std::env::split_paths(&on_path).map(|dir| here.join(dir).join("rxapi")))
        .find(|candidate| candidate.is_file() && access(candidate, Access::EXEC_OK).is_ok())
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

fn private(dir: &Path) -> Result<PathBuf, TransportError> {
    prepare_dir(dir).map_err(|error| unavailable(&error))
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
    /// Another user's process answered.
    Foreign {
        uid: u32,
    },
}

impl ConnectFailure {
    fn into_error(self, version: u32) -> TransportError {
        match self {
            ConnectFailure::Absent(error) => unavailable(&error),
            ConnectFailure::Refused { theirs } => TransportError::Refused {
                ours: version,
                theirs,
            },
            ConnectFailure::Foreign { uid } => TransportError::Foreign { uid },
        }
    }
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
        let dir = private(dir)?;
        match Unix::attempt(&dir, PROTOCOL_VERSION, own_uid()) {
            Err(ConnectFailure::Absent(_)) => {
                Unix::start_and_connect(&dir, &find_daemon()?, own_uid())
            }
            result => result.map_err(|failure| failure.into_error(PROTOCOL_VERSION)),
        }
    }

    /// As `connect_or_start`, with the executable to start.
    pub fn connect_or_start_with(dir: &Path, daemon: &Path) -> Result<Unix, TransportError> {
        Unix::start_and_connect(&private(dir)?, daemon, own_uid())
    }

    fn start_and_connect(dir: &Path, daemon: &Path, uid: u32) -> Result<Unix, TransportError> {
        let deadline = now() + START_TIMEOUT;
        let mut next_start = now();
        loop {
            let error = match Unix::attempt(dir, PROTOCOL_VERSION, uid) {
                Ok(connected) => return Ok(connected),
                Err(ConnectFailure::Absent(error)) => error,
                Err(failure) => return Err(failure.into_error(PROTOCOL_VERSION)),
            };
            let time = now();
            if time >= deadline {
                return Err(unavailable(&error));
            }
            if time >= next_start {
                start_daemon(daemon, dir)?;
                next_start = time + RESPAWN_INTERVAL;
            }
            pause();
        }
    }

    /// Connects to a running service speaking `version`, starting nothing.
    pub fn connect_version(dir: &Path, version: u32) -> Result<Unix, TransportError> {
        Unix::attempt(&private(dir)?, version, own_uid())
            .map_err(|failure| failure.into_error(version))
    }

    /// Connects to the service in the private `dir`, which must run as `uid`.
    fn attempt(dir: &Path, version: u32, uid: u32) -> Result<Unix, ConnectFailure> {
        let stream =
            UnixStream::connect(socket_path(dir, version)).map_err(ConnectFailure::Absent)?;
        let peer = peer_uid(&stream).map_err(ConnectFailure::Absent)?;
        if peer != uid {
            return Err(ConnectFailure::Foreign { uid: peer });
        }
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
        Unix::stop_as(&private(dir)?, version, own_uid())
    }

    fn stop_as(dir: &Path, version: u32, uid: u32) -> Result<bool, TransportError> {
        let mut client = match Unix::attempt(dir, version, uid) {
            Ok(client) => client,
            Err(ConnectFailure::Absent(_)) => return Ok(false),
            Err(failure) => return Err(failure.into_error(version)),
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
    fn call_cancellable(
        &self,
        request: Request,
        token: &CancelToken,
    ) -> Result<Reply, TransportError> {
        let lost = || TransportError::Unavailable("the connection is closed".to_owned());
        let mut started = token.started();
        if started.cancelled {
            return Ok(Reply::Cancelled);
        }
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
        started.call = Some(call);
        if self.send(call, &wire::encode_request(&request)).is_err() {
            lock_calls(&self.calls).waiting.remove(&call);
            return Err(lost());
        }
        drop(started);
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

    fn chosen(api: Option<&str>, xdg: Option<&str>) -> (PathBuf, Option<PathBuf>) {
        choose_dir(api.map(Into::into), xdg.map(Into::into), Path::new("/t"), 7)
    }

    fn dir(api: Option<&str>, xdg: Option<&str>) -> PathBuf {
        chosen(api, xdg).0
    }

    #[test]
    fn the_directory_comes_from_the_override_then_xdg_then_temp() {
        assert_eq!(dir(Some("/a"), Some("/x")), Path::new("/a"));
        assert_eq!(dir(None, Some("/x")), Path::new("/x/interrexx"));
        assert_eq!(dir(None, None), Path::new("/t/interrexx-7"));
        assert_eq!(dir(Some(""), Some("")), Path::new("/t/interrexx-7"));
    }

    /// A candidate whose socket path cannot be bound is passed over for the next, and a
    /// passed-over `REXX_API_DIR` is named for the warning.
    #[test]
    fn a_directory_too_long_for_a_socket_is_passed_over() {
        let long = format!("/{}", "d".repeat(120));
        let long_xdg = format!("/{}", "x".repeat(120));
        assert_eq!(chosen(Some("/a"), Some("/x")), (PathBuf::from("/a"), None));
        assert_eq!(
            chosen(Some(&long), Some("/x")),
            (PathBuf::from("/x/interrexx"), Some(PathBuf::from(&long)))
        );
        assert_eq!(
            chosen(Some(&long), Some(&long_xdg)),
            (PathBuf::from("/t/interrexx-7"), Some(PathBuf::from(&long)))
        );
        assert_eq!(
            chosen(None, Some(&long_xdg)),
            (PathBuf::from("/t/interrexx-7"), None)
        );
        let fits = format!("/{}", "d".repeat(80));
        assert_eq!(chosen(Some(&fits), None), (PathBuf::from(&fits), None));
    }

    /// A listener in a private directory, which `uid` here is never the owner of.
    fn foreign_listener(name: &str) -> (PathBuf, std::os::unix::net::UnixListener, u32) {
        let root = std::env::temp_dir().join(format!("rxapi-{name}-{}", std::process::id()));
        let dir = prepare_dir(&root.join("api")).expect("a private directory");
        let socket = socket_path(&dir, PROTOCOL_VERSION);
        let listener = std::os::unix::net::UnixListener::bind(socket).expect("a listener");
        listener.set_nonblocking(true).expect("non-blocking");
        (root, listener, own_uid() + 1)
    }

    fn assert_told_nothing(listener: &std::os::unix::net::UnixListener) {
        let (mut accepted, _) = listener.accept().expect("the attempt connected");
        accepted.set_nonblocking(true).expect("non-blocking");
        let mut byte = [0];
        match accepted.read(&mut byte) {
            Ok(0) => {}
            Ok(_) => panic!("the peer was sent bytes"),
            Err(error) => assert_eq!(error.kind(), io::ErrorKind::WouldBlock),
        }
    }

    #[test]
    fn a_service_of_another_user_is_an_error_not_an_absence() {
        let (root, listener, uid) = foreign_listener("foreign");
        let dir = root.join("api");
        let foreign = TransportError::Foreign { uid: own_uid() };

        assert_eq!(
            Unix::stop_as(&dir, PROTOCOL_VERSION, uid).err(),
            Some(foreign.clone())
        );
        assert_told_nothing(&listener);
        let starting = Unix::start_and_connect(&dir, Path::new("/nonexistent"), uid);
        assert_eq!(starting.err(), Some(foreign.clone()));
        assert_told_nothing(&listener);
        assert_eq!(foreign.status(), crate::message::code::NO_SERVICE);

        fs::remove_dir_all(&root).expect("cleanup");
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
