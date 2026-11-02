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
            .args(["-k", "5", "30"])
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

/// Review Focus 3: a halt ends a wait on an empty external queue, and the queue still works.
#[test]
fn a_halt_ends_a_queue_wait_and_leaves_the_queue_usable() {
    let service = Service::new();
    let output = service.run(
        "q = .RexxQueue~create('HALTQ')\n\
         call rxqueue 'set', q\n\
         '(sleep 1; kill -INT $PPID) &'\n\
         signal on halt\n\
         line = linein('QUEUE:')\n\
         say 'not halted' line\n\
         exit\n\
         halt: say 'halted' condition('c')\n\
         queue 'after'\n\
         say queued() linein('QUEUE:')\n\
         say .RexxQueue~delete(q)\n",
    );
    assert_eq!(
        stdout(&output),
        "halted HALT\n1 after\n0\n",
        "{}",
        stderr(&output)
    );
    assert_eq!(output.status.code(), Some(0));
}

/// A `lineIn` waiting on the session queue keeps waiting when a background command starts the
/// service during the wait and the queue moves there, and gets the line the command adds later.
#[test]
fn a_session_wait_survives_the_queue_moving_to_the_service() {
    for wait in ["linein('QUEUE:')", ".RexxQueue~new('SESSION')~lineIn"] {
        let service = Service::new();
        let output = service.run(&format!(
            "'(sleep 1; (sleep 2; echo late) | rxqueue) &'\n\
             t = time('e'); l = {wait}\n\
             say '['l']' (time('e') - t > 2.5)\n"
        ));
        assert_eq!(stdout(&output), "[late] 1\n", "{wait}: {}", stderr(&output));
        assert_eq!(output.status.code(), Some(0), "{wait}");
    }
}

/// Review Focus 4: a function one program registers is registered for the next.
#[test]
fn a_registration_outlives_its_program() {
    let service = Service::new();
    let first = service.run("say rxfuncadd('NOFN', 'nosuchlib') rxfuncquery('NOFN')\n");
    assert_eq!(stdout(&first), "1 0\n", "{}", stderr(&first));
    let second = service.run("say rxfuncquery('NOFN') rxfuncquery('OTHERFN')\n");
    assert_eq!(stdout(&second), "0 1\n", "{}", stderr(&second));
}

/// A macro one program adds is found by the next, before or after external files as ordered.
#[test]
fn macros_reach_later_programs() {
    let service = Service::new();
    std::fs::write(
        service.root.join("work/mm.rex"),
        "parse source s; say 'mac' s; return arg(1)*2\n",
    )
    .expect("the macro source is written");
    let first = service.run(
        "say SysAddRexxMacro('PMAC', 'mm.rex') SysQueryRexxMacro('PMAC')\n\
         say pmac(21)\n\
         say SysReorderRexxMacro('PMAC', 'A') SysQueryRexxMacro('pmac')\n\
         say SysSaveRexxMacroSpace('saved.mac')\n\
         say SysDropRexxMacro('PMAC') '['SysQueryRexxMacro('PMAC')']' SysDropRexxMacro('PMAC')\n\
         say SysLoadRexxMacroSpace('saved.mac') SysQueryRexxMacro('PMAC')\n\
         say SysAddRexxMacro('NOPE', 'nofile.rex')\n",
    );
    assert_eq!(
        stdout(&first),
        "0 B\nmac LINUX FUNCTION PMAC\n42\n0 A\n0\n0 [] 2\n0 A\n7\n",
        "{}",
        stderr(&first)
    );
    let second = service.run(
        "say SysQueryRexxMacro('PMAC') pmac(5)\n\
         say SysClearRexxMacroSpace() '['SysQueryRexxMacro('PMAC')']'\n",
    );
    assert_eq!(
        stdout(&second),
        "mac LINUX FUNCTION PMAC\nA 10\n0 []\n",
        "{}",
        stderr(&second)
    );
}

/// `RXQUEUE` creates, selects, opens and deletes named queues. A selected name reads back in
/// upper case.
#[test]
fn rxqueue_selects_and_manages_named_queues() {
    let service = Service::new();
    let output = service.run(
        "say rxqueue('get')\n\
         n = rxqueue('create')\n\
         say rxqueue('exists', n) rxqueue('set', n)\n\
         queue 'x'; say queued() (rxqueue('get') == translate(n))\n\
         say (rxqueue('set', 'SESSION') == translate(n)) rxqueue('get') queued()\n\
         say rxqueue('delete', n) rxqueue('exists', n)\n\
         say rxqueue('create', 'pq1') rxqueue('open', 'pq1') rxqueue('delete', 'pq1')\n\
         say rxqueue('Open', 'pq1') rxqueue('d', 'pq1')\n\
         say rxqueue('delete', 'session') rxqueue('exists', 'session')\n",
    );
    assert_eq!(
        stdout(&output),
        "SESSION\n1 SESSION\n1 1\n1 SESSION 0\n0 0\npq1 0 0\n0 0\n5 1\n",
        "{}",
        stderr(&output)
    );
}

/// A `REGISTERED` routine is registered before its entry point fails to resolve, so a later
/// program finds the name registered.
#[test]
fn a_registered_routine_stays_registered_after_its_program_fails() {
    let service = Service::new();
    let first = service.run("say 'main ran'\n::routine foo6 external \"REGISTERED nosuchlib\"\n");
    assert_eq!(stdout(&first), "");
    assert!(
        stderr(&first).contains("Error 90.999:  Unable to find external routine \"FOO6\"."),
        "{}",
        stderr(&first)
    );
    assert_eq!(first.status.code(), Some(166));
    let second = service.run("say rxfuncquery('FOO6')\n");
    assert_eq!(stdout(&second), "0\n", "{}", stderr(&second));
}

/// A waiting `lineIn` gives up its guard, as the oracle's `SetGuardOff` does, so another
/// activity's guarded `lineOut` on the same queue object delivers its line: run directly and
/// from inside an `INTERPRET`, where the wait does not return to the scheduler.
#[test]
fn a_waiting_line_in_lets_a_line_out_through_its_guard() {
    let service = Service::new();
    for wait in ["say q~lineIn", "interpret 'say q~lineIn'"] {
        let output = service.run(&format!(
            "t = .t~new\nt~go\n::class t\n::method go\n  .RexxQueue~delete('WAITQ')\n  \
             q = .RexxQueue~new('WAITQ')\n  self~delayedWrite(q, 'written')\n  \
             say q~queued\n  {wait}\n  say q~delete\n\
             ::method delayedWrite unguarded\n  use strict arg q, line\n  reply\n  \
             call SysSleep 0.1\n  q~lineOut(line)\n"
        ));
        assert_eq!(
            stdout(&output),
            "0\nwritten\n0\n",
            "{wait}: {}",
            stderr(&output)
        );
        assert_eq!(output.status.code(), Some(0), "{wait}");
    }
}
