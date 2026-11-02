//! The service: per-client threads around one owner of `Core`.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{FileTypeExt, OpenOptionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use rustix::fs::{FlockOperation, flock};

use crate::core::{ClientId, Core, Outcome};
use crate::message::wire::{self, Command};
use crate::message::{Reply, Request, Timestamp};
use crate::transport::unix::{lock_path, socket_path};

/// How long a silent connector may hold a thread before its handshake.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(2);

/// What a daemon run needs.
pub struct Options<'a> {
    pub dir: &'a Path,
    pub version: u32,
    /// Take the lock, report, and stop instead of listening.
    pub dry_run: bool,
    /// How long a daemon nobody has connected to waits before exiting.
    pub idle_grace: Duration,
    /// Receives each `rxapi: ...` line.
    pub report: &'a dyn Fn(&str),
}

/// How a daemon run ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    Stopped,
    /// Another daemon holds the lock.
    LockHeld,
    /// Something other than a socket is at the service path.
    PathOccupied,
}

fn timestamp<Tz: TimeZone>(time: &DateTime<Tz>) -> Timestamp {
    let narrow = |value: u32| value as u16;
    Timestamp {
        year: u16::try_from(time.year() - 1900).unwrap_or(0),
        month: narrow(time.month0()),
        day: narrow(time.day()),
        weekday: narrow(time.weekday().num_days_from_sunday()),
        yearday: narrow(time.ordinal0()),
        hours: narrow(time.hour()),
        minutes: narrow(time.minute()),
        seconds: narrow(time.second()),
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "an item's timestamp is real local time"
)]
fn now() -> Timestamp {
    timestamp(&Local::now())
}

/// A client thread's message to the owner.
enum Event {
    Connected {
        connection: u64,
        stream: UnixStream,
        outgoing: Sender<Outgoing>,
    },
    Frame {
        connection: u64,
        call: u64,
        command: Command,
    },
    Closed {
        connection: u64,
    },
}

/// What a connection's writer sends.
enum Outgoing {
    Welcome,
    Reply(u64, Reply),
}

fn take_lock(path: &Path) -> io::Result<Option<File>> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path)?;
    match flock(&file, FlockOperation::NonBlockingLockExclusive) {
        Ok(()) => {}
        Err(rustix::io::Errno::WOULDBLOCK) => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    file.set_len(0)?;
    writeln!(file, "{}", std::process::id())?;
    Ok(Some(file))
}

/// Runs a daemon until it exits. The caller has prepared `options.dir`.
pub fn serve(options: &Options<'_>) -> io::Result<Exit> {
    let report = options.report;
    let lock = lock_path(options.dir, options.version);
    report(&format!("rxapi: lockfile path is {}", lock.display()));
    let Some(lock_file) = take_lock(&lock)? else {
        report("rxapi: lockfile is locked by another rxapi instance; exiting");
        return Ok(Exit::LockHeld);
    };
    report("rxapi: lockfile lock acquired");

    let socket = socket_path(options.dir, options.version);
    report(&format!("rxapi: service path is {}", socket.display()));
    let occupant = match fs::symlink_metadata(&socket) {
        Ok(metadata) => Some(metadata.file_type()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    if occupant.is_some_and(|kind| !kind.is_socket()) {
        report("rxapi: service is locked by another rxapi instance; exiting");
        return Ok(Exit::PathOccupied);
    }
    if !options.dry_run {
        if occupant.is_some() {
            fs::remove_file(&socket)?;
        }
        let listener = UnixListener::bind(&socket)?;
        report("rxapi: service successfully started; listening");

        let (events, incoming) = mpsc::channel();
        let version = options.version;
        std::thread::spawn(move || accept(listener, version, events));
        let connections = own_core(incoming, options.idle_grace);

        let _ = fs::remove_file(&socket);
        for stream in connections {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    }
    drop(lock_file);
    report("rxapi: service stopped and lockfile released; exiting");
    Ok(Exit::Stopped)
}

fn accept(listener: UnixListener, version: u32, events: Sender<Event>) {
    let mut next_connection = 0;
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let connection = next_connection;
        next_connection += 1;
        let events = events.clone();
        std::thread::spawn(move || read_client(stream, connection, version, events));
    }
}

/// Answers a connection's handshake, then feeds its frames to the owner until it ends.
fn read_client(mut stream: UnixStream, connection: u64, version: u32, events: Sender<Event>) {
    if !accepted_hello(&mut stream, version) {
        return;
    }
    let Ok(writer) = stream.try_clone() else {
        return;
    };
    let Ok(owner_stream) = stream.try_clone() else {
        return;
    };
    let (outgoing, to_write) = mpsc::channel();
    std::thread::spawn(move || write_client(writer, version, to_write));
    let connected = Event::Connected {
        connection,
        stream: owner_stream,
        outgoing,
    };
    if events.send(connected).is_err() {
        return;
    }
    while let Ok(Some((call, body))) = wire::read_frame(&mut stream) {
        let Ok(command) = wire::decode_command(&body) else {
            break;
        };
        let frame = Event::Frame {
            connection,
            call,
            command,
        };
        if events.send(frame).is_err() {
            return;
        }
    }
    let _ = events.send(Event::Closed { connection });
}

/// Reads the `hello`, refusing anything but this `version`.
fn accepted_hello(stream: &mut UnixStream, version: u32) -> bool {
    let mut hello = [0; wire::HELLO_LENGTH];
    let read = stream
        .set_read_timeout(Some(HANDSHAKE_TIMEOUT))
        .and_then(|()| stream.read_exact(&mut hello))
        .and_then(|()| stream.set_read_timeout(None));
    if read.is_err() {
        return false;
    }
    if wire::hello_version(&hello) == Some(version) {
        return true;
    }
    let _ = stream.write_all(&wire::welcome(false, version));
    false
}

/// Writes on its own thread, so a client that stops reading stalls only itself.
fn write_client(mut stream: UnixStream, version: u32, to_write: Receiver<Outgoing>) {
    for message in to_write {
        let written = match message {
            Outgoing::Welcome => stream.write_all(&wire::welcome(true, version)),
            Outgoing::Reply(call, reply) => {
                stream.write_all(&wire::frame(call, &wire::encode_reply(&reply)))
            }
        };
        if written.is_err() {
            return;
        }
    }
}

struct Connection {
    stream: UnixStream,
    outgoing: Sender<Outgoing>,
}

/// Owns the core until the daemon should exit, answering the streams still open.
fn own_core(events: Receiver<Event>, idle_grace: Duration) -> Vec<UnixStream> {
    let mut core = Core::new();
    let mut connections: HashMap<u64, Connection> = HashMap::new();
    let mut served = false;
    loop {
        let event = if served {
            events.recv().map_err(|_| RecvTimeoutError::Disconnected)
        } else {
            events.recv_timeout(idle_grace)
        };
        let Ok(event) = event else { break };
        match event {
            Event::Connected {
                connection,
                stream,
                outgoing,
            } => {
                served = true;
                let _ = outgoing.send(Outgoing::Welcome);
                connections.insert(connection, Connection { stream, outgoing });
            }
            Event::Frame {
                command: Command::Stop,
                ..
            } => break,
            Event::Frame {
                connection,
                call,
                command: Command::Request(request),
            } => {
                let client = ClientId { connection, call };
                if let Outcome::Reply(reply) = core.handle(client, stamped(request)) {
                    send_reply(&connections, client, reply);
                }
                for (client, reply) in core.take_ready() {
                    send_reply(&connections, client, reply);
                }
            }
            Event::Closed { connection } => {
                core.client_closed(ClientId::connection(connection));
                if let Some(closed) = connections.remove(&connection) {
                    let _ = closed.stream.shutdown(std::net::Shutdown::Both);
                }
            }
        }
        if served && connections.is_empty() && core.is_empty() {
            break;
        }
    }
    connections
        .into_values()
        .map(|connection| connection.stream)
        .collect()
}

/// Items carry the daemon's local time, whatever the client sent.
fn stamped(mut request: Request) -> Request {
    if let Request::AddQueue { time, .. } = &mut request {
        *time = now();
    }
    request
}

fn send_reply(connections: &HashMap<u64, Connection>, client: ClientId, reply: Reply) {
    if let Some(connection) = connections.get(&client.connection) {
        let _ = connection
            .outgoing
            .send(Outgoing::Reply(client.call, reply));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn timestamps_count_as_struct_tm_does() {
        let monday = Utc
            .with_ymd_and_hms(2026, 11, 2, 13, 14, 15)
            .single()
            .expect("a valid date");
        assert_eq!(
            timestamp(&monday),
            Timestamp {
                year: 126,
                month: 10,
                day: 2,
                weekday: 1,
                yearday: 305,
                hours: 13,
                minutes: 14,
                seconds: 15,
            }
        );
    }

    #[test]
    fn only_an_add_is_stamped() {
        let add = Request::AddQueue {
            session: 0,
            name: None,
            data: Vec::new(),
            flag: 0,
            time: Timestamp::default(),
        };
        assert!(matches!(
            stamped(add),
            Request::AddQueue { time, .. } if time != Timestamp::default()
        ));
        assert_eq!(stamped(Request::ClearMacroSpace), Request::ClearMacroSpace);
    }
}
