//! A real daemon, each test in its own `REXX_API_DIR`.

use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::time::Duration;

use rexx_rxapi::daemon::{self, Exit, Options};
use rexx_rxapi::message::code::*;
use rexx_rxapi::message::wire::PROTOCOL_VERSION;
use rexx_rxapi::message::{FIFO, Registry, Reply, Request, Timestamp, WAIT};
use rexx_rxapi::transport::unix::{Unix, lock_path, prepare_dir, socket_path};
use rexx_rxapi::transport::{Transport, TransportError};

const DAEMON: &str = env!("CARGO_BIN_EXE_rxapi");
const CHILD_DIR: &str = "RXAPI_TEST_CHILD_DIR";

/// A private service directory, whose daemon is stopped on drop.
struct ServiceDir {
    root: PathBuf,
    dir: PathBuf,
}

impl ServiceDir {
    fn new() -> ServiceDir {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "rxapi-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).expect("the temp directory is writable");
        let dir = root.join("api");
        ServiceDir { root, dir }
    }

    fn connect(&self) -> Unix {
        Unix::connect_or_start_with(&self.dir, Path::new(DAEMON)).expect("a daemon")
    }

    fn pid(&self) -> u32 {
        std::fs::read_to_string(lock_path(&self.dir, PROTOCOL_VERSION))
            .expect("a lock file")
            .trim()
            .parse()
            .expect("the lock file holds the pid")
    }

    fn rxapi(&self, args: &[&str]) -> Output {
        Command::new(DAEMON)
            .args(args)
            .env("REXX_API_DIR", &self.dir)
            .output()
            .expect("rxapi runs")
    }
}

impl Drop for ServiceDir {
    fn drop(&mut self) {
        let _ = Unix::stop(&self.dir);
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn open(client: &impl Transport, queue: &str) -> Reply {
    client
        .call(Request::OpenQueue {
            name: Some(queue.as_bytes().to_vec()),
        })
        .expect("a reply")
}

fn delete(client: &impl Transport, queue: &str) -> Reply {
    client
        .call(Request::DeleteQueue {
            name: Some(queue.as_bytes().to_vec()),
        })
        .expect("a reply")
}

fn add_request(queue: &str, data: &[u8]) -> Request {
    Request::AddQueue {
        session: 1,
        name: Some(queue.as_bytes().to_vec()),
        data: data.to_vec(),
        flag: FIFO,
        time: Timestamp::default(),
    }
}

fn pull_request(queue: &str) -> Request {
    Request::PullQueue {
        session: 1,
        name: Some(queue.as_bytes().to_vec()),
        wait: WAIT,
    }
}

fn pulled_data(reply: Reply) -> Vec<u8> {
    match reply {
        Reply::Pulled {
            status: OK,
            item: Some(item),
        } => item.data,
        other => panic!("no item: {other:?}"),
    }
}

fn pause() {
    #[expect(clippy::disallowed_methods, reason = "polling a real daemon")]
    std::thread::sleep(Duration::from_millis(5));
}

/// Polls `condition` until it holds, for the seconds a loaded host may need.
fn eventually(what: &str, mut condition: impl FnMut() -> bool) {
    for _ in 0..4000 {
        if condition() {
            return;
        }
        pause();
    }
    panic!("never happened: {what}");
}

/// A delete refused with `QUEUE_ACCESS` is the only sign of a parked pull, so a probe that arrives
/// first deletes the queue. This reopens it, and `blocking_pull` retries a pull that met the gap.
fn wait_until_parked(client: &impl Transport, queue: &str) {
    eventually("the pull is parked", || {
        let parked = delete(client, queue) == Reply::Status(QUEUE_ACCESS);
        if !parked {
            open(client, queue);
        }
        parked
    });
}

fn blocking_pull(client: &impl Transport, queue: &str) -> Result<Reply, TransportError> {
    loop {
        client.call(Request::OpenQueue {
            name: Some(queue.as_bytes().to_vec()),
        })?;
        let reply = client.call(pull_request(queue))?;
        let gap = Reply::Pulled {
            status: QUEUE_NOT_REGISTERED,
            item: None,
        };
        if reply != gap {
            return Ok(reply);
        }
    }
}

fn is_running(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}/comm")).exists()
        && std::fs::read_to_string(format!("/proc/{pid}/stat"))
            .is_ok_and(|stat| !stat.contains(") Z "))
}

fn kill(pid: u32) {
    let status = Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .status()
        .expect("kill runs");
    assert!(status.success());
    eventually("the daemon is dead", || !is_running(pid));
}

fn stdout_lines(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

/// Runs the `#[ignore]`d test `name` of this binary as a separate client process.
fn spawn_child(name: &str, service: &ServiceDir) -> Child {
    Command::new(std::env::current_exe().expect("the test binary"))
        .args(["--exact", name, "--ignored", "--nocapture"])
        .env(CHILD_DIR, &service.dir)
        .stdout(Stdio::null())
        .spawn()
        .expect("the child starts")
}

/// Review Focus 1.
#[test]
fn racing_starters_end_with_one_daemon() {
    let service = Arc::new(ServiceDir::new());
    let barrier = Arc::new(Barrier::new(8));
    let starters: Vec<_> = (0..8)
        .map(|_| {
            let service = Arc::clone(&service);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                service.connect()
            })
        })
        .collect();
    let clients: Vec<Unix> = starters
        .into_iter()
        .map(|starter| starter.join().expect("a starter"))
        .collect();

    let created = clients
        .iter()
        .filter(|client| {
            open(*client, "one")
                == Reply::QueueOpened {
                    status: OK,
                    duplicate: Some(false),
                }
        })
        .count();
    assert_eq!(created, 1, "every client reached the same state");
    assert!(is_running(service.pid()));
    assert_eq!(service.rxapi(&[]).status.code(), Some(13));
}

/// Review Focus 3.
#[test]
fn a_killed_daemon_gives_clean_errors_and_the_next_connect_restarts() {
    let service = ServiceDir::new();
    let client = Arc::new(service.connect());
    open(&*client, "q");
    let blocked = {
        let client = Arc::clone(&client);
        std::thread::spawn(move || blocking_pull(&*client, "q"))
    };
    wait_until_parked(&*client, "q");
    let pid = service.pid();

    kill(pid);

    let error = blocked
        .join()
        .expect("the blocked thread ends")
        .expect_err("the pull cannot succeed");
    assert_eq!(error.status(), NO_SERVICE);
    let error = client
        .call(Request::QueueExists { name: None })
        .expect_err("a connected client stays disconnected");
    assert_eq!(error.status(), NO_SERVICE);

    let fresh = service.connect();
    assert_ne!(service.pid(), pid);
    assert_eq!(
        open(&fresh, "q"),
        Reply::QueueOpened {
            status: OK,
            duplicate: Some(false),
        },
        "a new daemon holds nothing"
    );
    assert_eq!(
        client
            .call(Request::QueueExists { name: None })
            .expect_err("the old connection stays closed")
            .status(),
        NO_SERVICE
    );
}

/// Review Focus 4.
#[test]
fn a_stale_socket_does_not_block_the_next_start() {
    let service = ServiceDir::new();
    prepare_dir(&service.dir).expect("the directory");
    let socket = socket_path(&service.dir, PROTOCOL_VERSION);
    let lock = lock_path(&service.dir, PROTOCOL_VERSION);
    drop(UnixListener::bind(&socket).expect("a socket nobody serves"));
    std::fs::write(&lock, "999999\n").expect("a stale lock file");
    assert!(socket.exists());

    let client = service.connect();

    assert_eq!(
        open(&client, "q"),
        Reply::QueueOpened {
            status: OK,
            duplicate: Some(false),
        }
    );
    assert_ne!(service.pid(), 999999);
}

/// The adjacent refusal: what occupies the path is not a socket, so it is not ours to remove.
#[test]
fn a_file_at_the_service_path_is_refused_and_kept() {
    let service = ServiceDir::new();
    prepare_dir(&service.dir).expect("the directory");
    let socket = socket_path(&service.dir, PROTOCOL_VERSION);
    std::fs::write(&socket, "not a socket").expect("a plain file");

    let output = service.rxapi(&[]);

    assert_eq!(output.status.code(), Some(13));
    assert_eq!(
        stdout_lines(&output).last().map(String::as_str),
        Some("rxapi: service is locked by another rxapi instance; exiting")
    );
    assert_eq!(
        std::fs::read_to_string(&socket).expect("the file"),
        "not a socket"
    );
}

fn serve_version(dir: PathBuf, version: u32) -> std::thread::JoinHandle<Exit> {
    std::thread::spawn(move || {
        daemon::serve(&Options {
            dir: &dir,
            version,
            dry_run: false,
            idle_grace: Duration::from_secs(60),
            report: &|_| {},
        })
        .expect("the other version serves")
    })
}

fn connect_version(dir: &Path, version: u32) -> Unix {
    let mut connected = None;
    eventually("the other version listens", || {
        connected = Unix::connect_version(dir, version).ok();
        connected.is_some()
    });
    connected.expect("connected")
}

/// Review Focus 5.
#[test]
fn another_protocol_version_gets_its_own_socket_and_daemon() {
    let service = ServiceDir::new();
    prepare_dir(&service.dir).expect("the directory");
    let old_version = PROTOCOL_VERSION + 1;
    let old_daemon = serve_version(service.dir.clone(), old_version);
    let old_client = connect_version(&service.dir, old_version);
    open(&old_client, "q");

    let client = service.connect();

    assert_eq!(
        open(&client, "q"),
        Reply::QueueOpened {
            status: OK,
            duplicate: Some(false),
        },
        "this version has its own daemon"
    );
    assert!(socket_path(&service.dir, old_version).exists());
    assert!(socket_path(&service.dir, PROTOCOL_VERSION).exists());
    assert_ne!(
        socket_path(&service.dir, old_version),
        socket_path(&service.dir, PROTOCOL_VERSION)
    );
    assert_eq!(
        open(&old_client, "q"),
        Reply::QueueOpened {
            status: OK,
            duplicate: Some(true),
        },
        "the old daemon keeps serving its clients"
    );

    drop(old_client);
    Unix::stop_version(&service.dir, old_version).expect("the old daemon stops");
    assert_eq!(old_daemon.join().expect("the old daemon"), Exit::Stopped);
}

/// A version mismatch reached by another route: here, a socket under the wrong name.
#[test]
fn a_daemon_of_another_version_on_our_socket_is_a_clean_refusal() {
    let service = ServiceDir::new();
    prepare_dir(&service.dir).expect("the directory");
    let other = PROTOCOL_VERSION + 1;
    let daemon = serve_version(service.dir.clone(), other);
    let held = connect_version(&service.dir, other);
    std::fs::rename(
        socket_path(&service.dir, other),
        socket_path(&service.dir, PROTOCOL_VERSION),
    )
    .expect("the rename");

    let error = Unix::connect_or_start_with(&service.dir, Path::new(DAEMON))
        .err()
        .expect("the versions differ");

    assert_eq!(
        error,
        TransportError::Refused {
            ours: PROTOCOL_VERSION,
            theirs: other,
        }
    );
    assert_eq!(error.status(), NO_SERVICE);

    std::fs::rename(
        socket_path(&service.dir, PROTOCOL_VERSION),
        socket_path(&service.dir, other),
    )
    .expect("the rename back");
    Unix::stop_version(&service.dir, other).expect("the daemon stops");
    assert_eq!(daemon.join().expect("the daemon"), Exit::Stopped);
    drop(held);
}

#[test]
#[ignore = "the client half of two tests, run through spawn_child"]
fn child_pulls_from_shared() {
    let dir = PathBuf::from(std::env::var_os(CHILD_DIR).expect("run by spawn_child"));
    let client = Unix::connect_or_start_with(&dir, Path::new(DAEMON)).expect("a daemon");
    let item = pulled_data(blocking_pull(&client, "shared").expect("a reply"));
    assert_eq!(item, b"from the parent");
}

#[test]
fn two_processes_add_and_block_on_one_queue() {
    let service = ServiceDir::new();
    let client = service.connect();
    open(&client, "shared");
    let mut child = spawn_child("child_pulls_from_shared", &service);
    wait_until_parked(&client, "shared");

    assert_eq!(
        client
            .call(add_request("shared", b"from the parent"))
            .expect("a reply"),
        Reply::Status(OK)
    );

    assert!(child.wait().expect("the child").success());
    assert_eq!(delete(&client, "shared"), Reply::Status(OK));
}

/// Review Focus 2, across a real kill.
#[test]
fn a_killed_blocked_process_leaves_its_queue_deletable_and_the_daemon_alive() {
    let service = ServiceDir::new();
    let client = service.connect();
    open(&client, "shared");
    let mut child = spawn_child("child_pulls_from_shared", &service);
    wait_until_parked(&client, "shared");
    let pid = service.pid();

    child.kill().expect("the kill");
    child.wait().expect("the child");

    eventually("the waiter is forgotten", || {
        delete(&client, "shared") != Reply::Status(QUEUE_ACCESS)
    });
    assert!(is_running(pid));
    assert_eq!(
        client
            .call(Request::QueueExists {
                name: Some(b"shared".to_vec())
            })
            .expect("the daemon answers"),
        Reply::Status(QUEUE_NOT_REGISTERED)
    );
}

#[test]
fn a_blocked_thread_does_not_delay_another_threads_calls() {
    let service = ServiceDir::new();
    let client = Arc::new(service.connect());
    open(&*client, "q");
    let blocked: Vec<_> = (0..2)
        .map(|_| {
            let client = Arc::clone(&client);
            std::thread::spawn(move || blocking_pull(&*client, "q"))
        })
        .collect();
    wait_until_parked(&*client, "q");

    for data in [&b"a"[..], b"b"] {
        assert_eq!(
            client.call(add_request("q", data)).expect("a reply"),
            Reply::Status(OK)
        );
    }

    let mut items: Vec<Vec<u8>> = blocked
        .into_iter()
        .map(|thread| pulled_data(thread.join().expect("a puller").expect("a reply")))
        .collect();
    items.sort();
    assert_eq!(items, [b"a", b"b"]);
}

#[test]
fn the_daemon_exits_when_nobody_is_connected_and_it_holds_nothing() {
    let service = ServiceDir::new();
    let socket = socket_path(&service.dir, PROTOCOL_VERSION);
    let client = service.connect();
    let pid = service.pid();
    open(&client, "q");
    drop(client);

    let client = service.connect();
    assert_eq!(service.pid(), pid, "a held queue keeps the daemon up");
    assert_eq!(delete(&client, "q"), Reply::Status(OK));
    assert!(is_running(pid), "a connected client keeps the daemon up");
    let other = service.connect();
    drop(client);
    assert_eq!(
        other
            .call(Request::QueueExists { name: None })
            .expect("still served"),
        Reply::Status(QUEUE_NOT_REGISTERED)
    );

    drop(other);
    eventually("the daemon exits", || !is_running(pid));
    assert!(!socket.exists(), "the daemon removes its socket");
}

#[test]
fn a_registration_keeps_the_daemon_up_after_its_registrant_ends() {
    let service = ServiceDir::new();
    let register = Request::Register {
        registry: Registry::Function,
        name: b"F".to_vec(),
        library: b"lib".to_vec(),
        procedure: b"F".to_vec(),
        user_area: None,
        owner_only: false,
    };
    let client = service.connect();
    let pid = service.pid();
    assert_eq!(client.call(register).expect("a reply"), Reply::Status(OK));
    drop(client);

    let later = service.connect();

    assert_eq!(service.pid(), pid);
    let query = Request::QueryRegistration {
        registry: Registry::Function,
        name: b"f".to_vec(),
        library: Vec::new(),
    };
    assert!(matches!(
        later.call(query).expect("a reply"),
        Reply::Registration { status: OK, .. }
    ));
}

#[test]
fn a_daemon_nobody_connects_to_exits_after_its_grace() {
    let service = ServiceDir::new();
    let output = Command::new(DAEMON)
        .env("REXX_API_DIR", &service.dir)
        .env("REXX_API_IDLE_MS", "50")
        .output()
        .expect("rxapi runs");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout_lines(&output).last().map(String::as_str),
        Some("rxapi: service stopped and lockfile released; exiting")
    );
}

#[test]
fn large_items_and_the_daemons_timestamp_arrive() {
    let service = ServiceDir::new();
    let client = service.connect();
    open(&client, "big");
    let data = vec![b'x'; 50_000_000];

    assert_eq!(
        client.call(add_request("big", &data)).expect("a reply"),
        Reply::Status(OK)
    );

    let Reply::Pulled {
        status: OK,
        item: Some(item),
    } = client.call(pull_request("big")).expect("a reply")
    else {
        panic!("no item");
    };
    assert_eq!(item.data, data);
    assert!(
        item.time.year >= 126,
        "stamped by the daemon: {:?}",
        item.time
    );
    assert!((1..=31).contains(&item.time.day));
}

#[test]
fn rxapi_prints_what_the_test_group_relies_on_when_the_lock_is_held() {
    let service = ServiceDir::new();
    let client = service.connect();

    let output = service.rxapi(&[]);

    assert_eq!(output.status.code(), Some(13));
    let lock = lock_path(&service.dir, PROTOCOL_VERSION);
    assert_eq!(
        stdout_lines(&output),
        [
            format!(
                "rxapi: lockfile path is {}",
                std::fs::canonicalize(&lock).expect("the lock").display()
            ),
            "rxapi: lockfile is locked by another rxapi instance; exiting".to_owned(),
        ]
    );
    drop(client);
}

#[test]
fn rxapi_dryrun_and_unknown_arguments_print_their_lines() {
    let service = ServiceDir::new();

    let output = service.rxapi(&["--dryrun", "extra"]);

    assert_eq!(output.status.code(), Some(0));
    let lines = stdout_lines(&output);
    assert_eq!(lines[0], "rxapi: dryrun");
    assert_eq!(lines[1], "rxapi: no args allowed");
    assert!(lines[2].starts_with("rxapi: lockfile path is "));
    assert_eq!(lines[3], "rxapi: lockfile lock acquired");
    assert!(lines[4].starts_with("rxapi: service path is "));
    assert_eq!(
        lines[5],
        "rxapi: service stopped and lockfile released; exiting"
    );
    assert_eq!(lines.len(), 6);
    assert!(!socket_path(&service.dir, PROTOCOL_VERSION).exists());
}

#[test]
fn rxapi_stop_stops_a_daemon_holding_state_and_clients_see_a_clean_error() {
    let service = ServiceDir::new();
    let client = service.connect();
    open(&client, "q");
    let pid = service.pid();

    let output = service.rxapi(&["--stop"]);

    assert_eq!(output.status.code(), Some(0));
    eventually("the daemon exits", || !is_running(pid));
    assert_eq!(
        client
            .call(Request::QueueExists { name: None })
            .expect_err("the daemon is gone")
            .status(),
        NO_SERVICE
    );
}

#[test]
fn rxapi_stop_with_nothing_running_is_quiet_success() {
    let service = ServiceDir::new();
    let output = service.rxapi(&["--stop"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(!lock_path(&service.dir, PROTOCOL_VERSION).exists());
}
