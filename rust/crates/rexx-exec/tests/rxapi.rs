//! Programs reaching the API service: each test runs `rexx` with a private `REXX_API_DIR` and
//! stops its daemon after. `rxapi` and `rxqueue` are the `rexx-rxapi` binaries beside `rexx`,
//! which a workspace build makes.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

const REXX: &str = env!("CARGO_BIN_EXE_rexx");

/// The directory holding `rexx`, `rxapi` and `rxqueue`.
fn bin_dir() -> PathBuf {
    let dir = Path::new(REXX).parent().expect("rexx has a directory");
    for name in ["rxapi", "rxqueue"] {
        assert!(
            dir.join(name).is_file(),
            "{name} is not beside rexx; build the workspace's binaries first"
        );
    }
    dir.to_path_buf()
}

/// A private service directory and a program directory, whose daemon is stopped on drop.
struct Service {
    root: PathBuf,
}

impl Service {
    fn new() -> Service {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "rexx-rxapi-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(root.join("work")).expect("a scratch directory");
        Service { root }
    }

    fn api(&self) -> PathBuf {
        self.root.join("api")
    }

    fn command(&self, program: &str) -> Command {
        let file = self.root.join("work/program.rex");
        std::fs::write(&file, program).expect("the program is written");
        let path = std::env::join_paths(std::iter::once(bin_dir()).chain(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        )))
        .expect("a PATH");
        let mut command = Command::new("timeout");
        command
            .arg("60")
            .arg(REXX)
            .arg(&file)
            .current_dir(self.root.join("work"))
            .env("REXX_API_DIR", self.api())
            .env("PATH", path)
            .env_remove("RXQUEUESESSION")
            .env_remove("RXQUEUE")
            .stdin(Stdio::null());
        command
    }

    fn run(&self, program: &str) -> Output {
        self.command(program).output().expect("rexx runs")
    }

    /// Whether a daemon is listening in this service's directory.
    fn has_socket(&self) -> bool {
        std::fs::read_dir(self.api()).is_ok_and(|entries| {
            entries
                .flatten()
                .any(|entry| entry.file_name().to_string_lossy().ends_with(".sock"))
        })
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        let _ = Command::new(bin_dir().join("rxapi"))
            .arg("--stop")
            .env("REXX_API_DIR", self.api())
            .output();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Review Focus 1: a command's `rxqueue` reaches the program's session queue, with no daemon
/// running beforehand.
#[test]
fn a_piped_rxqueue_delivers_to_the_program() {
    let service = Service::new();
    assert!(!service.has_socket());
    let output = service.run(
        "'printf \"one\\ntwo\\n\" | rxqueue'\n\
         say queued()\n\
         pull a; pull b\n\
         say a b\n",
    );
    assert_eq!(stdout(&output), "2\nONE TWO\n", "{}", stderr(&output));
    assert_eq!(output.status.code(), Some(0));
}

/// Lines queued before the first command reach the command's `rxqueue` in order.
#[test]
fn queued_lines_move_to_the_service_before_a_command() {
    let service = Service::new();
    let output = service.run(
        "queue 'first'\n\
         'printf \"second\\n\" | rxqueue'\n\
         push 'zero'\n\
         do queued(); pull line; say line; end\n",
    );
    assert_eq!(
        stdout(&output),
        "ZERO\nFIRST\nSECOND\n",
        "{}",
        stderr(&output)
    );
}

/// Review Focus 2: `QUEUE` and `PULL` alone start no daemon.
#[test]
fn a_program_without_commands_starts_no_daemon() {
    let service = Service::new();
    let output = service.run(
        "queue 'a'; push 'b'\n\
         say queued()\n\
         pull x; say x\n\
         say .stdque~queued .stdque~pull\n",
    );
    assert_eq!(stdout(&output), "2\nB\n1 a\n", "{}", stderr(&output));
    assert!(!service.api().exists(), "nothing connected to a service");
}

/// Review Focus 5: a daemon killed under a running program gives a clean error, not a hang or
/// a panic.
#[test]
fn a_killed_daemon_gives_a_clean_error() {
    let service = Service::new();
    let output = service.run(
        "say .RexxQueue~open('KILLED')\n\
         'kill -9 $(cat \"$REXX_API_DIR\"/rxapi.*.lock)'\n\
         say queued()\n\
         signal on syntax\n\
         queue 'x'\n\
         say 'not raised'\n\
         exit\n\
         syntax: say condition('o')~code condition('o')~message\n",
    );
    assert_eq!(
        stdout(&output),
        "0\n0\n48.1 Failure in system service: SYSTEM QUEUE (RXAPI_NORXAPI).\n",
        "{}",
        stderr(&output)
    );
    assert_eq!(output.status.code(), Some(0));
}
