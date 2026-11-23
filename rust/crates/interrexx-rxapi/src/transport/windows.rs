//! Windows RXAPI over a per-account local named pipe.

use super::{CancelToken, Transport, TransportError, windows_handles as sys};
use crate::message::wire::{self, PROTOCOL_VERSION};
use crate::message::{Reply, Request};
use interprocess::os::windows::{
    named_pipe::{DuplexPipeStream, PipeListener, PipeListenerOptions, pipe_mode::Bytes},
    security_descriptor::SecurityDescriptor,
};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::os::windows::process::CommandExt;
use std::os::windows::{ffi::OsStrExt, fs::OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use widestring::U16CString;
use windows_sys::Win32::Storage::FileSystem::{FILE_FLAG_OVERLAPPED, SECURITY_IDENTIFICATION};
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

const HANDSHAKE: Duration = Duration::from_secs(2);

#[derive(Clone)]
pub(crate) struct Pipe(Arc<DuplexPipeStream<Bytes>>);

impl Pipe {
    pub(crate) fn close(&self) {
        sys::disconnect(&*self.0);
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "the service handshake has a real-time deadline"
    )]
    pub(crate) fn hello(&mut self, bytes: &mut [u8]) -> io::Result<()> {
        let deadline = Instant::now() + HANDSHAKE;
        let mut taken = 0;
        while taken < bytes.len() {
            let available = sys::readable(&*self.0)? as usize;
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            if available == 0 {
                thread::sleep(Duration::from_millis(5));
                continue;
            }
            let end = (taken + available).min(bytes.len());
            self.read_exact(&mut bytes[taken..end])?;
            taken = end;
        }
        Ok(())
    }
}

impl Read for Pipe {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        (&*self.0).read(bytes)
    }
}

impl Write for Pipe {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        (&*self.0).write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

pub fn service_dir() -> PathBuf {
    std::env::var_os("REXX_API_DIR").map_or_else(
        || {
            std::env::var_os("LOCALAPPDATA")
                .map_or_else(std::env::temp_dir, PathBuf::from)
                .join("interrexx")
        },
        PathBuf::from,
    )
}

fn pipe_name(dir: &Path, account: &str, version: u32) -> String {
    // FNV-1a on the UTF-16 spelling keeps the pipe name stable across Rust releases.
    let hash = dir
        .as_os_str()
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .fold(0xcbf29ce484222325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        });
    format!(r"\\.\pipe\interrexx-rxapi-{account}-{version}-{hash:016x}")
}

pub(crate) struct Listener(PipeListener<Bytes, Bytes>);

impl Listener {
    pub(crate) fn bind(dir: &Path, version: u32) -> io::Result<Self> {
        let sid = sys::account()?;
        let descriptor =
            U16CString::from_str(format!("O:{sid}D:P(A;;GA;;;{sid})")).map_err(io::Error::other)?;
        let descriptor = SecurityDescriptor::deserialize(&descriptor)?;
        let name = U16CString::from_str(pipe_name(dir, &sid, version)).map_err(io::Error::other)?;
        let listener = PipeListenerOptions::new()
            .path(name)
            .security_descriptor(Some(descriptor))
            .nonblocking(true)
            .create_duplex::<Bytes>()?;
        Ok(Self(listener))
    }

    pub(crate) fn accept(&self) -> io::Result<Pipe> {
        let stream = self.0.accept()?;
        stream.set_nonblocking(false)?;
        Ok(Pipe(Arc::new(stream)))
    }
}

#[derive(Default)]
struct Calls {
    waiters: HashMap<u64, mpsc::Sender<Reply>>,
    next: u64,
    closed: bool,
}

pub struct Windows {
    pipe: Mutex<Pipe>,
    calls: Arc<Mutex<Calls>>,
    closing: Arc<AtomicBool>,
    reader: Option<JoinHandle<()>>,
}

fn unavailable(error: io::Error) -> TransportError {
    TransportError::Unavailable(error.to_string())
}

impl Windows {
    pub fn connect_or_start(dir: &Path) -> Result<Self, TransportError> {
        Self::connect_or_start_with(dir, &daemon_path().map_err(unavailable)?)
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "daemon startup is bounded by real time"
    )]
    pub fn connect_or_start_with(dir: &Path, daemon: &Path) -> Result<Self, TransportError> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut start_at = Instant::now();
        loop {
            match Self::attempt(dir, PROTOCOL_VERSION) {
                Ok(client) => return Ok(client),
                Err(error) if !matches!(error, TransportError::Unavailable(_)) => {
                    return Err(error);
                }
                Err(error) if Instant::now() >= deadline => return Err(error),
                Err(_) => {}
            }
            if Instant::now() >= start_at {
                Command::new(daemon)
                    .env("REXX_API_DIR", dir)
                    .creation_flags(CREATE_NO_WINDOW)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .map_err(unavailable)?;
                start_at = Instant::now() + Duration::from_millis(250);
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn connect_version(dir: &Path, version: u32) -> Result<Self, TransportError> {
        Self::attempt(dir, version)
    }

    fn attempt(dir: &Path, version: u32) -> Result<Self, TransportError> {
        let account = sys::account().map_err(unavailable)?;
        // Identification-only QoS prevents an unrelated pipe server from impersonating this client.
        let handle = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(FILE_FLAG_OVERLAPPED)
            .security_qos_flags(SECURITY_IDENTIFICATION)
            .open(pipe_name(dir, &account, version))
            .map_err(|error| {
                if error.kind() == io::ErrorKind::PermissionDenied {
                    TransportError::NotPrivate {
                        dir: dir.display().to_string(),
                    }
                } else {
                    unavailable(error)
                }
            })?;
        let raw = DuplexPipeStream::try_from(std::os::windows::io::OwnedHandle::from(handle))
            .map_err(|error| unavailable(io::Error::other(format!("{error:?}"))))?;
        if sys::server_account(&raw).ok().as_deref() != Some(&account) {
            return Err(TransportError::NotPrivate {
                dir: dir.display().to_string(),
            });
        }
        let mut pipe = Pipe(Arc::new(raw));
        pipe.write_all(&wire::hello(version)).map_err(unavailable)?;
        let mut welcome = [0; wire::WELCOME_LENGTH];
        pipe.hello(&mut welcome).map_err(unavailable)?;
        let (accepted, theirs) = wire::read_welcome(&welcome);
        if !accepted || theirs != version {
            return Err(TransportError::Refused {
                ours: version,
                theirs,
            });
        }
        let calls = Arc::new(Mutex::new(Calls::default()));
        let closing = Arc::new(AtomicBool::new(false));
        let reader = {
            let mut pipe = pipe.clone();
            let calls = Arc::clone(&calls);
            let closing = Arc::clone(&closing);
            thread::Builder::new()
                .name("rexx-api".to_owned())
                .spawn(move || {
                    while !closing.load(Ordering::Acquire) {
                        let Ok(Some((call, body))) = wire::read_frame(&mut pipe) else {
                            break;
                        };
                        let Ok(reply) = wire::decode_reply(&body) else {
                            break;
                        };
                        if let Some(waiter) = calls.lock().expect("call lock").waiters.remove(&call)
                        {
                            let _ = waiter.send(reply);
                        }
                    }
                    let mut state = calls.lock().expect("call lock");
                    state.closed = true;
                    state.waiters.clear();
                })
                .map_err(unavailable)?
        };
        Ok(Self {
            pipe: Mutex::new(pipe),
            calls,
            closing,
            reader: Some(reader),
        })
    }

    pub fn stop(dir: &Path) -> Result<bool, TransportError> {
        let mut client = match Self::attempt(dir, PROTOCOL_VERSION) {
            Ok(client) => client,
            Err(TransportError::Unavailable(_)) => return Ok(false),
            Err(error) => return Err(error),
        };
        client.send(0, &wire::encode_stop()).map_err(unavailable)?;
        if let Some(reader) = client.reader.take() {
            let _ = reader.join();
        }
        Ok(true)
    }

    fn send(&self, call: u64, body: &[u8]) -> io::Result<()> {
        self.pipe
            .lock()
            .expect("send lock")
            .write_all(&wire::frame(call, body))
    }
}

fn daemon_path() -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os("REXX_API_DAEMON") {
        return Ok(PathBuf::from(path));
    }
    Ok(std::env::current_exe()?.with_file_name("rxapi.exe"))
}

impl Transport for Windows {
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
            let mut calls = self.calls.lock().expect("call lock");
            if calls.closed {
                return Err(lost());
            }
            calls.next += 1;
            let call = calls.next;
            calls.waiters.insert(call, sender);
            call
        };
        started.call = Some(call);
        if self.send(call, &wire::encode_request(&request)).is_err() {
            self.calls.lock().expect("call lock").waiters.remove(&call);
            return Err(lost());
        }
        drop(started);
        receiver.recv().map_err(|_| lost())
    }
}

impl Drop for Windows {
    #[expect(
        clippy::disallowed_methods,
        reason = "cancelling pending Windows I/O waits for the reader thread"
    )]
    fn drop(&mut self) {
        self.closing.store(true, Ordering::Release);
        if let Some(reader) = self.reader.take() {
            if let Ok(pipe) = self.pipe.get_mut() {
                while !reader.is_finished() {
                    sys::cancel(&*pipe.0);
                    thread::sleep(Duration::from_millis(2));
                }
            }
            let _ = reader.join();
        }
    }
}
