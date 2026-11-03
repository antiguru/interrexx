//! ooTest's classic-API groups on `rexx`, through the `testbinaries/` libraries compiled by
//! `scripts/build-testbinaries.sh` against this workspace's `librexx.so.4` and
//! `librexxapi.so.4`. Each run has a private `REXX_API_DIR` and stops its daemon after. `rxapi`
//! and `rxsubcom` are the `rexx-rxapi` binaries beside `rexx`, which a workspace build makes.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

const REXX: &str = env!("CARGO_BIN_EXE_rexx");

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

/// This test's directory under cargo's target, which each run overwrites.
fn scratch() -> &'static Path {
    static SCRATCH: OnceLock<PathBuf> = OnceLock::new();
    SCRATCH.get_or_init(|| {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("classic-api");
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        dir
    })
}

/// The directory holding `rexx`, `rxapi` and `rxsubcom`.
fn bin_dir() -> PathBuf {
    let dir = Path::new(REXX).parent().expect("rexx has a directory");
    for name in ["rxapi", "rxsubcom"] {
        assert!(
            dir.join(name).is_file(),
            "{name} is not beside rexx; build the workspace's binaries first"
        );
    }
    dir.to_path_buf()
}

/// The compiled test libraries, built once from the shared objects cargo left beside this test.
fn testbinaries() -> &'static Path {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT.get_or_init(|| {
        let deps = std::env::current_exe()
            .expect("the test's path")
            .parent()
            .expect("the deps directory")
            .to_path_buf();
        let out = scratch().join("testbinaries");
        let output = Command::new(repository().join("scripts/build-testbinaries.sh"))
            .arg("--libs")
            .arg(&deps)
            .arg("--out")
            .arg(&out)
            .output()
            .expect("the build script runs");
        assert!(
            output.status.success(),
            "build-testbinaries.sh failed:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        out
    })
}

/// The library search path a run gets: the compiled test libraries where `with_testbinaries`,
/// and otherwise none. The framework's own `rxregexp.cls` and `rxregexp` are this crate's.
fn library_path(with_testbinaries: bool) -> OsString {
    let mut directories = Vec::new();
    if with_testbinaries {
        directories.push(testbinaries().to_path_buf());
    }
    std::env::join_paths(directories).expect("a library path")
}

/// `rexx` running `arguments` from `cwd` against the service in `api`, with `library_path`.
fn rexx(arguments: &[&str], cwd: &Path, api: &Path, library_path: OsString) -> Command {
    // The test drivers (`rexxinstance`) after `rexx`'s own directory.
    let path = std::env::join_paths([bin_dir(), testbinaries().to_path_buf()].into_iter().chain(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
    ))
    .expect("a PATH");
    let mut command = Command::new("timeout");
    command
        .args(["-k", "5", "600"])
        .arg(REXX)
        .args(arguments)
        .current_dir(cwd)
        .env("REXX_API_DIR", api)
        .env("LD_LIBRARY_PATH", library_path)
        .env("PATH", path)
        .env_remove("RXQUEUESESSION")
        .env_remove("RXQUEUE")
        .stdin(Stdio::null());
    command
}

fn stop_service(api: &Path) {
    let _ = Command::new(bin_dir().join("rxapi"))
        .arg("--stop")
        .env("REXX_API_DIR", api)
        .output();
}

/// A group's outcome: how many tests ran, how many assertions they made, and which failed or
/// errored.
struct Outcome {
    ran: usize,
    assertions: usize,
    failed: BTreeSet<String>,
    stdout: String,
}

fn count(stdout: &str, label: &str) -> usize {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix(label))
        .and_then(|count| count.trim().parse().ok())
        .unwrap_or_else(|| panic!("no {label} line:\n{stdout}"))
}

/// Runs `group` of `dir` through `testOORexx.rex`, from a fresh copy of the framework and the
/// group's directory, with `library_path`.
fn run_group(dir: &str, group: &str, library_path: OsString) -> Outcome {
    run_group_in(dir, group, library_path, &[])
}

/// [`run_group`] with `env` added to the run's environment.
fn run_group_in(dir: &str, group: &str, library_path: OsString, env: &[(&str, &Path)]) -> Outcome {
    let ootest = repository().join("ootest");
    let run = scratch().join(format!("run-{group}"));
    let _ = std::fs::remove_dir_all(&run);
    std::fs::create_dir_all(run.join("ooRexx").join(dir)).expect("the run directory");
    let copy = |from: &Path, to: &Path| {
        let status = Command::new("cp")
            .arg("-r")
            .arg(from)
            .arg(to)
            .status()
            .expect("cp runs");
        assert!(status.success(), "cp {} {}", from.display(), to.display());
    };
    copy(&ootest.join("framework"), &run.join("framework"));
    copy(
        &ootest.join("ooRexx").join(dir),
        &run.join("ooRexx").join(dir).join(".."),
    );
    for file in ["testOORexx.rex", "worker.rex", "ooTest.frm"] {
        copy(&ootest.join(file), &run);
    }
    let api = scratch().join(format!("api-{group}"));
    let group_file = format!("ooRexx/{dir}/{group}.testGroup");
    let mut command = rexx(
        &["testOORexx.rex", "-f", &group_file, "-U", "-V", "2"],
        &run,
        &api,
        library_path,
    );
    command.envs(env.iter().copied());
    let output = command.output().expect("rexx runs");
    stop_service(&api);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        stdout.contains("Tests ran:"),
        "no summary:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut failed = BTreeSet::new();
    let mut in_detail = false;
    for line in stdout.lines() {
        if line.starts_with("[failure]") || line.starts_with("[error]") {
            in_detail = true;
        } else if in_detail && let Some(test) = line.trim().strip_prefix("Test:") {
            failed.insert(test.trim().to_owned());
            in_detail = false;
        }
    }
    Outcome {
        ran: count(&stdout, "Tests ran:"),
        assertions: count(&stdout, "Assertions:"),
        failed,
        stdout,
    }
}

fn classic() -> &'static Outcome {
    static OUTCOME: OnceLock<Outcome> = OnceLock::new();
    OUTCOME.get_or_init(|| run_group("API/classic", "CLASSIC", library_path(true)))
}

fn assert_pass(outcome: &Outcome, tests: &[&str]) {
    let failing: Vec<&&str> = tests
        .iter()
        .filter(|test| outcome.failed.contains(**test))
        .collect();
    assert!(
        failing.is_empty() && outcome.ran >= tests.len(),
        "{failing:?} failed of {} run:\n{}",
        outcome.ran,
        outcome.stdout
    );
}

/// Queues, memory and the macrospace through `orxclassic`'s methods, which call `librexxapi`.
#[test]
fn the_classic_queue_memory_and_macrospace_tests_pass() {
    assert_pass(
        classic(),
        &[
            "TEST_01_CREATE_QUEUE",
            "TEST_02_OPEN_QUEUE",
            "TEST_03_QUEUE_EXISTS",
            "TEST_04_DELETE_QUEUE",
            "TEST_05_QUERY_QUEUE",
            "TEST_06_ADD_QUEUE_1",
            "TEST_07_QUERY_QUEUE_2",
            "TEST_08_PULLFROM_QUEUE_1",
            "TEST_09_CLEAR_QUEUE",
            "TEST_10_PULLFROM_QUEUE_2",
            "TEST_11_DELETE_QUEUE",
            "TEST_20_ALLOCATEFREE_MEMORY",
            "TEST_70_MACROSPACE",
            "TEST_71_MACROSPACE",
            "TEST_72_MACROSPACE",
            "TEST_73_MACROSPACE",
            "TEST_74_MACROSPACE",
            "TEST_75_MACROSPACE",
            "TEST_76_MACROSPACE",
        ],
    );
}

/// `RxFuncAdd` of a C library, a classic function calling the registration API from inside the
/// interpreter, an entry point registered from C and called from Rexx, and registered `Dll` and
/// `Exe` subcommand environments.
#[test]
fn the_classic_function_and_subcommand_tests_pass() {
    assert_pass(
        classic(),
        &[
            "TEST_50_EXTERNALFUNCTION",
            "TEST_51_EXTERNALFUNCTION",
            "TEST_52_EXTERNALFUNCTION",
            "TEST_53_EXTERNALFUNCTION",
            "TEST_54_EXTERNALFUNCTION",
            "TEST_56_EXTERNALFUNCTION",
            "TEST_60_SUBCOM",
            "TEST_61_SUBCOM",
            "TEST_62_SUBCOM",
            "TEST_63_SUBCOM",
            "TEST_77_FUNCTION",
        ],
    );
}

/// The whole CLASSIC group, its variable-pool tests included: `RexxVariablePool` from a routine
/// reaches its caller's variables (set, fetch and drop, direct and symbolic, the private names
/// and `RXSHV_NEXTV`), and from a method answers `RXSHV_NOAVL`.
#[test]
fn the_whole_classic_group_passes() {
    let outcome = classic();
    assert!(
        outcome.failed.is_empty() && outcome.ran == 72,
        "{:?} failed of {} run:\n{}",
        outcome.failed,
        outcome.ran,
        outcome.stdout
    );
}

/// `RexxCallBack` from a classic function runs a label of its caller as an internal function:
/// results, arguments, shared variables, `SIGL`, nesting, and a condition or `EXIT` in the
/// label ending the external function.
#[test]
fn the_rexx_call_back_group_passes() {
    let outcome = run_group("API/classic", "RexxCallBack", library_path(true));
    assert!(
        outcome.failed.is_empty() && outcome.ran == 19,
        "{:?} failed of {} run:\n{}",
        outcome.failed,
        outcome.ran,
        outcome.stdout
    );
}

/// `rxsubcom LOAD` loads `orxclassic1` and finds its handler, so the group's `haveLibrary` tests
/// run their bodies: more assertions than the same group makes without the library to load.
#[test]
fn the_rxsubcom_group_runs_its_library_tests() {
    let with = run_group("utilities/rxsubcom", "rxsubcom", library_path(true));
    assert!(
        with.failed.is_empty(),
        "{:?}:\n{}",
        with.failed,
        with.stdout
    );
    let without = run_group("utilities/rxsubcom", "rxsubcom", library_path(false));
    assert!(
        without.failed.is_empty(),
        "{:?}:\n{}",
        without.failed,
        without.stdout
    );
    assert!(
        with.assertions > without.assertions,
        "{} assertions with the library, {} without",
        with.assertions,
        without.assertions
    );
}

/// Review Focus 3: a classic function answering non-zero raises 40.1, blaming the routine as the
/// oracle does.
#[test]
fn a_failing_classic_function_raises_40_1() {
    let work = scratch().join("failing");
    std::fs::create_dir_all(&work).expect("a work directory");
    std::fs::write(
        work.join("program.rex"),
        "say rxfuncadd('TestCallBack', 'orxclassic1', 'TestCallBack')\n\
         x = TestCallBack()\n",
    )
    .expect("the program is written");
    let api = scratch().join("api-failing");
    let output = rexx(&["program.rex"], &work, &api, library_path(true))
        .output()
        .expect("rexx runs");
    stop_service(&api);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "0\n");
    assert_eq!(output.status.code(), Some(216), "{stderr}");
    assert_eq!(
        stderr.lines().next(),
        Some("       *-* Compiled routine \"TESTCALLBACK\"."),
        "{stderr}"
    );
    assert!(
        stderr.ends_with("Error 40.1:  External routine \"TESTCALLBACK\" failed.\n"),
        "{stderr}"
    );
}

/// Runs `program` as `program.rex` in a fresh directory `name` holding `files`, with the
/// compiled test libraries, against a private service it stops after. Its stdout.
fn run_program(name: &str, program: &str, files: &[(&str, &str)]) -> String {
    let work = scratch().join(name);
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).expect("a work directory");
    std::fs::write(work.join("program.rex"), program).expect("the program is written");
    for (file, text) in files {
        std::fs::write(work.join(file), text).expect("a file is written");
    }
    let api = scratch().join(format!("api-{name}"));
    let output = rexx(&["program.rex"], &work, &api, library_path(true))
        .output()
        .expect("rexx runs");
    stop_service(&api);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The two programs of the group-signal tests and the tester class `main.rex` requires, written
/// into `work`: `main.rex ID` starts `inner.rex ID` through `RexxStart` on an activity of its own
/// and runs `main_command` (each `ID` substituted); `inner.rex` runs `touch inner.ID; sleep 30`.
/// Each traps `HALT`, and `main.rex` says how its command and the inner program ended.
fn write_group_signal_programs(work: &Path, before_command: &str, main_command: &str) {
    let tester =
        std::fs::read_to_string(repository().join("ootest/ooRexx/API/oo/INVOCATIONTester.cls"))
            .expect("INVOCATIONTester.cls");
    let _ = std::fs::remove_dir_all(work);
    std::fs::create_dir_all(work).expect("a work directory");
    let main = format!(
        "signal on halt\nparse arg id\n\
         a = .InvocationTester~new~start('callRexxStart', 'inner.rex', .array~of(id))\n\
         {before_command}'{main_command}'\nsay 'outer missed' rc a~result\nexit\n\
         halt: say 'outer halted' a~result\n::requires 'INVOCATIONTester.cls'\n"
    );
    for (file, text) in [
        ("main.rex", main.as_str()),
        (
            "inner.rex",
            "signal on halt\nparse arg id\n'touch inner.'id'; sleep 30'\n\
             return 'inner missed' rc\nhalt: return 'inner halted'\n",
        ),
        ("INVOCATIONTester.cls", &tester),
    ] {
        std::fs::write(work.join(file), text).expect("a file is written");
    }
}

/// `rexx main.rex id` in `work`, a process group of its own, its standard output in `out`.
fn start_group_signal_run(work: &Path, api: &Path, id: &str, out: &Path) -> std::process::Child {
    Command::new(REXX)
        .args(["main.rex", id])
        .current_dir(work)
        .env("REXX_API_DIR", api)
        .env("LD_LIBRARY_PATH", library_path(true))
        .stdin(Stdio::null())
        .stdout(std::fs::File::create(out).expect("the output file"))
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .expect("rexx starts")
}

/// Waits until every one of `files` exists, or `deadline` has passed since `began`.
#[expect(
    clippy::disallowed_methods,
    reason = "this waits on real processes' marker files"
)]
fn wait_for_files(files: &[PathBuf], began: std::time::Instant, deadline: std::time::Duration) {
    while !files.iter().all(|file| file.exists()) && began.elapsed() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// SIGINT to `child`'s process group, as a terminal's Ctrl-C sends it; a kill where it fails.
fn interrupt_group(child: &mut std::process::Child) {
    let group = format!("-{}", child.id());
    let sent = Command::new("kill")
        .args(["-INT", "--", &group])
        .status()
        .is_ok_and(|status| status.success());
    if !sent {
        let _ = child.kill();
    }
}

/// Waits for `child` to end, killing it after `deadline`.
#[expect(clippy::disallowed_methods, reason = "this waits on a real process")]
fn finish(child: &mut std::process::Child, deadline: std::time::Duration) {
    let began = std::time::Instant::now();
    while child.try_wait().ok().flatten().is_none() && began.elapsed() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// Queue item 4: two interpreters in one process, `rexx` and one an extension starts with
/// `RexxStart`, each wait on a command when the process group takes a SIGINT, as a terminal's
/// Ctrl-C sends it. The kernel hands the process's signal to one of its threads, often not the
/// one waiting on the inner command, and that command's child ends at once; the inner command's
/// clause is halted all the same, never run on past. The miss is a race the thread the signal
/// went to loses when it is slow to run its handler, so the runs go in batches beside threads
/// that keep every CPU busy.
///
/// Each command writes a marker before it sleeps, and a run's signal is sent once both of its
/// markers exist. The earlier form sent the signals one after another with a fixed 300 ms pause
/// each, so under load a batch's last runs were signalled after their 10-second `sleep` had
/// ended; traced, the halt was posted to both interpreters 10.1 s after the outer command's spawn,
/// which then reads as `outer missed 0`.
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "this test times and loads real runs"
)]
fn a_group_signal_halts_both_interpreters_waiting_on_commands() {
    const BATCHES: usize = 4;
    const AT_ONCE: usize = 30;
    let work = scratch().join("group-signal");
    write_group_signal_programs(&work, "", "touch outer.'id'; sleep 30");
    let api = scratch().join("api-group-signal");
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cpus = std::thread::available_parallelism().map_or(4, std::num::NonZero::get);
    let busy: Vec<_> = (0..cpus + cpus / 2)
        .map(|_| {
            let stop = std::sync::Arc::clone(&stop);
            std::thread::spawn(move || {
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    std::hint::spin_loop();
                }
            })
        })
        .collect();
    let deadline = std::time::Duration::from_secs(60);
    let mut wrong = Vec::new();
    for batch in 0..BATCHES {
        let runs: Vec<_> = (0..AT_ONCE)
            .map(|at| {
                let id = format!("{batch}-{at}");
                let out = work.join(format!("out.{id}"));
                let child = start_group_signal_run(&work, &api, &id, &out);
                (id, out, child, std::time::Instant::now())
            })
            .collect();
        let signalled: Vec<_> = runs
            .into_iter()
            .map(|(id, out, mut child, began)| {
                let markers = [
                    work.join(format!("inner.{id}")),
                    work.join(format!("outer.{id}")),
                ];
                wait_for_files(&markers, began, deadline);
                interrupt_group(&mut child);
                (id, out, child)
            })
            .collect();
        for (id, out, mut child) in signalled {
            finish(&mut child, deadline);
            let stdout = std::fs::read_to_string(&out).unwrap_or_default();
            if stdout != "outer halted inner halted\n" {
                wrong.push(format!("{id}: {stdout:?}"));
            }
        }
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    for thread in busy {
        thread.join().expect("a busy thread");
    }
    stop_service(&api);
    assert!(
        wrong.is_empty(),
        "{} of {}:\n{wrong:#?}",
        wrong.len(),
        BATCHES * AT_ONCE
    );
}

/// A group SIGINT that arrives while the inner interpreter waits on its command and the outer
/// one runs clauses before issuing its own: the halt reaches the outer interpreter at its next
/// clause, before any command is spawned, and the inner command's clause is halted. The outer
/// program loops until halted, so the order is fixed: no marker of an outer command may exist.
#[test]
#[expect(clippy::disallowed_methods, reason = "this test times real runs")]
fn a_group_signal_before_the_outer_command_halts_both_interpreters() {
    let work = scratch().join("group-signal-before");
    write_group_signal_programs(
        &work,
        "do while \\ SysFileExists('inner.'id); call SysSleep 0.01; end\n\
         call lineout 'ready.'id, 'x'; call lineout 'ready.'id\n\
         do forever; nop; end\n",
        "touch outer.'id'; sleep 30",
    );
    let api = scratch().join("api-group-signal-before");
    let deadline = std::time::Duration::from_secs(60);
    for id in ["a", "b", "c"] {
        let out = work.join(format!("out.{id}"));
        let mut child = start_group_signal_run(&work, &api, id, &out);
        wait_for_files(
            &[work.join(format!("ready.{id}"))],
            std::time::Instant::now(),
            deadline,
        );
        interrupt_group(&mut child);
        finish(&mut child, deadline);
        assert_eq!(
            std::fs::read_to_string(&out).unwrap_or_default(),
            "outer halted inner halted\n",
            "{id}"
        );
        assert!(!work.join(format!("outer.{id}")).exists(), "{id}");
    }
    stop_service(&api);
}

/// A macro an extension adds or drops through the C API reaches a call site that resolved the
/// name before: the site does not keep its earlier resolution. Matches the oracle.
#[test]
fn a_macrospace_change_through_the_c_api_reaches_a_resolved_call_site() {
    let stdout = run_program(
        "macro-generation",
        "m = .M~new\n\
         do i = 1 to 3\n\
         \x20 if i = 2 then say 'add' m~add('FOO', 'macro.rex', 1)\n\
         \x20 if i = 3 then say 'drop' m~drop('FOO')\n\
         \x20 say foo()\n\
         end\n\
         ::class M\n\
         ::method add external \"LIBRARY orxclassic TestAddMacro\"\n\
         ::method drop external \"LIBRARY orxclassic TestDropMacro\"\n",
        &[
            ("foo.rex", "return 'file'\n"),
            ("macro.rex", "return 'macro'\n"),
        ],
    );
    assert_eq!(stdout, "file\nadd 0\nmacro\ndrop 0\nfile\n");
}

/// The C API and the program see one session queue: an extension's query and pull find a line
/// the program queued while the interpreter held the queue in process, and the program pulls a
/// line the extension added. Matches the oracle.
#[test]
fn the_c_api_and_the_program_share_the_session_queue() {
    let stdout = run_program(
        "session-queue",
        "q = .Q~new\n\
         queue 'from rexx'\n\
         say 'count' q~count('SESSION') q~flag\n\
         say 'pull' q~pull('SESSION') q~flag\n\
         say 'add' q~add('SESSION', 'from c', 0) queued()\n\
         parse pull line; say 'line' line\n\
         ::class Q\n\
         ::attribute flag get\n\
         ::method count external \"LIBRARY orxclassic TestQueryQueue\"\n\
         ::method pull external \"LIBRARY orxclassic TestPullFromQueue\"\n\
         ::method add external \"LIBRARY orxclassic TestAddQueue\"\n",
        &[],
    );
    assert_eq!(
        stdout,
        "count 0 1\npull 0 from rexx\nadd 0 1\nline from c\n"
    );
}

/// `RexxStart` in process (`RexxStart`, an extension's native method calling it) and from the
/// `rexxinstance` driver (`ProcessRexxStart`): arguments, an omitted one, the call type, the
/// initial environment, a classic subcommand handler using the variable pool, a syntax
/// error's negated number, and every system exit, registered in the process or from a library.
/// The oracle's counts: `RexxStart` 20 tests and 60 assertions, `ProcessRexxStart` 20 and 59,
/// none failing.
#[test]
fn rexx_start_runs_programs_in_process_and_from_a_driver() {
    for (group, assertions) in [("RexxStart", 60), ("ProcessRexxStart", 59)] {
        let outcome = run_group("API/oo", group, library_path(true));
        assert!(
            outcome.failed.is_empty() && outcome.ran == 20 && outcome.assertions == assertions,
            "{group}: {:?} failed, {} ran, {} assertions:\n{}",
            outcome.failed,
            outcome.ran,
            outcome.assertions,
            outcome.stdout
        );
    }
}

/// Review Focus 2: `RexxStart` called twice in one process, then from two activities at once,
/// each in an interpreter of its own. Matches the oracle.
#[test]
fn rexx_start_runs_twice_and_from_two_activities_at_once() {
    let tester =
        std::fs::read_to_string(repository().join("ootest/ooRexx/API/oo/INVOCATIONTester.cls"))
            .expect("INVOCATIONTester.cls");
    let stdout = run_program(
        "rexx-start-twice",
        "say .InvocationTester~new~callRexxStart('calc.rex', .array~of(5))\n\
         say .InvocationTester~new~callRexxStart('calc.rex', .array~of(6))\n\
         a = .InvocationTester~new~start('callRexxStart', 'calc.rex', .array~of(3))\n\
         b = .InvocationTester~new~start('callRexxStart', 'calc.rex', .array~of(4))\n\
         say a~result b~result\n\
         ::requires 'INVOCATIONTester.cls'\n",
        &[
            ("calc.rex", "do 100000; end\nreturn arg(1) * 2 arg()\n"),
            ("INVOCATIONTester.cls", &tester),
        ],
    );
    assert_eq!(stdout, "10 1\n12 1\n6 1 8 1\n");
}

/// The instance API from an extension's native method (`INVOCATION`, a second instance nested in
/// the running one) and from the `rexxinstance` driver (`ProcessInvocation`): an instance
/// created, a program called through its thread context with arguments, its result or its
/// condition decoded, and the instance terminated; the call type, the initial environment,
/// direct, redirecting and registered command environments, a classic subcommand handler, the
/// instance's own search path and extensions, and every system exit, as context exits and as
/// registered ones. The oracle's counts: `INVOCATION` 55 tests and 295 assertions,
/// `ProcessInvocation` 54 and 284, which this crate matches with no failure: `TestLoadLibrary`
/// loads `rxmath`, which it runs in Rust.
#[test]
fn the_invocation_groups_run_programs_in_instances() {
    for (group, ran, assertions) in [("INVOCATION", 55, 295), ("ProcessInvocation", 54, 284)] {
        let outcome = run_group("API/oo", group, library_path(true));
        let failed: Vec<&str> = outcome.failed.iter().map(String::as_str).collect();
        assert!(
            failed.is_empty() && outcome.ran == ran && outcome.assertions == assertions,
            "{group}: {failed:?} failed, {} ran, {} assertions:\n{}",
            outcome.ran,
            outcome.assertions,
            outcome.stdout
        );
    }
}

/// `provoke_locks`: eight threads create an instance at once, each runs a program that leaves
/// reply activities running, and terminates it, ten rounds over: every instance starts, nothing
/// hangs (the driver's watchdog exits 99) and nothing crashes.
#[test]
fn concurrent_instances_start_run_and_terminate() {
    let api = scratch().join("api-provoke-locks");
    let output = Command::new("timeout")
        .args(["-k", "5", "300"])
        .arg(testbinaries().join("provoke_locks"))
        .arg(testbinaries().join("provoke_locks.rex"))
        .args(["8", "10", "240"])
        .current_dir(scratch())
        .env("REXX_API_DIR", &api)
        .stdin(Stdio::null())
        .output()
        .expect("provoke_locks runs");
    stop_service(&api);
    assert!(
        output.status.success(),
        "{:?}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A driver from `tests/classic_api/`, compiled against the test libraries' `librexx.so.4`.
fn driver(name: &str) -> PathBuf {
    driver_linked(name, name, &["-lrexx", "-lrexxapi"])
}

/// The driver `name` from `tests/classic_api/`, built as `out` with `flags` ending the command
/// line (`-l` options in that order), against the test libraries.
fn driver_linked(name: &str, out: &str, flags: &[&str]) -> PathBuf {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/classic_api/{name}.cpp"));
    let out = testbinaries().join(out);
    let api = repository().join("third_party/ooRexx/api");
    let output = Command::new("c++")
        .arg("-o")
        .arg(&out)
        .arg(&source)
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(format!("-L{}", testbinaries().display()))
        .arg(format!("-Wl,-rpath,{}", testbinaries().display()))
        .arg("-pthread")
        .args(flags)
        .output()
        .expect("c++ runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    out
}

/// Runs the driver `name` from a directory holding `files`, with `arguments`.
fn run_driver(name: &str, files: &[(&str, &str)], arguments: &[&str]) -> std::process::Output {
    run_binary(&driver(name), name, files, arguments)
}

/// Runs `binary`, a driver, as `name` from a directory holding `files`, with `arguments`.
/// glibc's allocator fills freed blocks with a pattern (`MALLOC_PERTURB_`), so a driver reading
/// memory the interpreter freed reads that pattern rather than whatever was there before.
fn run_binary(
    binary: &Path,
    name: &str,
    files: &[(&str, &str)],
    arguments: &[&str],
) -> std::process::Output {
    let dir = scratch().join(format!("driver-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the driver's directory");
    for (file, text) in files {
        std::fs::write(dir.join(file), text).expect("a program written");
    }
    let api = scratch().join(format!("api-{name}"));
    let output = Command::new("timeout")
        .args(["-k", "5", "60"])
        .arg(binary)
        .args(arguments)
        .current_dir(&dir)
        .env("REXX_API_DIR", &api)
        .env("MALLOC_PERTURB_", "165")
        .stdin(Stdio::null())
        .output()
        .expect("the driver runs");
    stop_service(&api);
    output
}

/// A program linked against `librexxapi.so.4` before `librexx.so.4`, after it, against
/// `librexx.so.4` alone, or against `librexxapi.so.4` alone (finding `RexxStart` in the
/// `librexx.so.4` loaded as its dependency) registers a function with `RexxRegisterFunctionExe`
/// and calls it from a program `RexxStart` runs: `librexx.so.4` owns the one API state and
/// `librexxapi.so.4` forwards to it (Moritz, 2026-11-03, option A). Before, `librexxapi.so.4`
/// held a state of its own, the registration went there and `RexxStart` raised Error 43.1 for
/// `-lrexxapi -lrexx`. The oracle answers as here in both two-library orders; it links neither
/// single-library form (its `librexx.so.4` exports no registration, its `librexxapi.so.4` no
/// `RexxStart`).
#[test]
fn a_registration_reaches_rexx_start_in_every_link_order() {
    for (out, libraries) in [
        ("order-rexx-first", &["-lrexx", "-lrexxapi"][..]),
        ("order-rexxapi-first", &["-lrexxapi", "-lrexx"][..]),
        ("order-rexx-alone", &["-lrexx"][..]),
        (
            "order-rexxapi-alone",
            &["-DFIND_REXXSTART", "-lrexxapi", "-ldl"][..],
        ),
    ] {
        let binary = driver_linked("order", out, libraries);
        let output = run_binary(&binary, out, &[], &[]);
        assert!(output.status.success(), "{out}: {output:?}");
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "register 0 query 0\ncode 0 result fromC\nderegister 0\n",
            "{out}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// `RexxCreateInterpreter`'s `REGISTER_LIBRARY` option: the embedder's package table serves a
/// program's `::requires 'mylib' LIBRARY`, and then a program without the directive. An unknown
/// option name ends the list, so an `INITIAL_ADDRESS_ENVIRONMENT` after it is not read, where
/// alone it is. Measured on the oracle with the same driver; ours dropped the library option and
/// raised Error 43.1, and read options past an unknown one.
#[test]
fn an_embedders_registered_library_serves_its_programs() {
    let output = run_driver("reglib", &[("address.rex", "return address()\n")], &[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "requires: fromlib\nplain: fromlib\nafter unknown: sh\nknown: ELSEWHERE\n",
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A C program's stdio output and the output of the programs it runs with `RexxStart` land in
/// the order they were written, with standard output and error sent to files, where stdio holds
/// the C program's lines back until something flushes them. Measured on the oracle with the same
/// driver; ours wrote every C line on standard output after all of Rexx's. (Sent to one file, the
/// oracle also puts a program's `SAY` before its later `.STDERR` line; ours hands a redirected
/// standard output over in blocks, Deviation 40.)
#[test]
fn c_stdio_and_rexx_output_keep_their_order_in_files() {
    let binary = driver("stdio");
    let dir = scratch().join("driver-stdio");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the driver's directory");
    let (out, err) = (dir.join("out.txt"), dir.join("err.txt"));
    let api = scratch().join("api-stdio");
    let status = Command::new("timeout")
        .args(["-k", "5", "60"])
        .arg(binary)
        .current_dir(&dir)
        .env("REXX_API_DIR", &api)
        .stdin(Stdio::null())
        .stdout(std::fs::File::create(&out).expect("the output file"))
        .stderr(std::fs::File::create(&err).expect("the error file"))
        .status()
        .expect("the driver runs");
    stop_service(&api);
    assert!(status.success(), "{status:?}");
    assert_eq!(
        std::fs::read_to_string(&out).expect("the output"),
        "c one\nrexx one\nc two\nrexx two\nrexx three\nc three\nc four\n"
    );
    assert_eq!(
        std::fs::read_to_string(&err).expect("the error output"),
        "c err one\nrexx err one\nrexx err two\n"
    );
}

/// `RexxStart`'s result into a caller's buffer: one exactly the result's length gets a new block
/// and is left untouched, one a byte longer takes the result and its NUL and nothing past them,
/// a shorter one gets a new block. The bytes after each buffer are checked. Measured on the
/// oracle with the same driver.
#[test]
fn rexx_start_writes_a_result_buffer_only_where_it_has_room_for_the_nul() {
    let output = run_driver("resultbuf", &[], &[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "size 3: new block length 3 value abc block XXXXXXXX\n\
         size 4: in buffer length 3 value abc block abc0XXXX\n\
         size 2: new block length 3 value abc block XXXXXXXX\n",
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Review Focus 4: a thread the instance never saw attaches, calls a program through the
/// context it is given, and detaches; `Terminate`, called while that thread is attached but
/// idle, returns without waiting for it, and its later `DetachThread` is harmless. The order
/// is the oracle's.
#[test]
fn a_foreign_thread_attaches_and_terminate_does_not_wait_for_it() {
    let output = run_driver(
        "attach",
        &[("double.rex", "parse arg n\nreturn n * 2\n")],
        &["double.rex"],
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "attach 1 result 42\nterminating\nterminated\ndetaching\n",
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A thread still attached when its instance terminates detaches afterwards, then calls through
/// the instance it was handed, and the creating thread terminates the instance again: the thread
/// context, the instance and the tables they link outlive `Terminate`, with the heap churned
/// between. Before, `Terminate` freed the root context's allocation and its function table, and
/// the late `DetachThread` called through freed memory (SIGSEGV). The output is the oracle's,
/// measured with the same driver.
#[test]
fn an_attached_threads_context_and_instance_outlive_terminate() {
    let output = run_driver(
        "attach_after",
        &[("double.rex", "parse arg n\nreturn n * 2\n")],
        &["double.rex"],
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "attach 1 result 42\nterminating\nterminated\ndetaching\ndetached\n\
         version through the instance ok\nterminated again\n",
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Review Focus 5: two instances in one process. `.local` is each instance's own, and so is
/// `.environment` (Deviation 85; the oracle's second instance sees the first's entry), and a
/// SIGINT to the process halts a program running in each.
#[test]
fn two_instances_keep_their_environments_and_both_take_a_halt() {
    let output = run_driver(
        "two",
        &[
            (
                "set.rex",
                ".environment~setEntry('RF5ENV', 'one')\n.local~setEntry('RF5LOCAL', 'one')\n\
                 return .environment~hasEntry('RF5ENV') .local~hasEntry('RF5LOCAL')\n",
            ),
            (
                "see.rex",
                "return .environment~hasEntry('RF5ENV') .local~hasEntry('RF5LOCAL')\n",
            ),
            ("loop.rex", "do forever\n  call SysSleep 0.01\nend\n"),
        ],
        &["set.rex", "see.rex", "loop.rex"],
    );
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines[..3],
        ["first 1 1", "second 0 0", "first again 1 1"],
        "{stdout}"
    );
    assert_eq!(lines.pop(), Some("done"), "{stdout}");
    let mut loops = lines.split_off(3);
    loops.sort_unstable();
    assert_eq!(
        loops,
        [
            "first loop condition SYNTAX",
            "second loop condition SYNTAX"
        ],
        "{stdout}"
    );
}

/// The rest of the classic family, from a driver: `RexxGetErrorMessage`,
/// `RexxGetErrorMessageByNumber` and `RexxGetVersionInformation`; `RexxStart`'s leading-blank
/// argument, its emptied result, its `//T` translation, an in-store source's image reused and
/// the macrospace an empty pair names; `RexxTranslateInstoreProgram`, `RexxTranslateProgram`
/// and `RexxCompileProgram` and the images they write run; `RexxSetHalt`, `RexxSetTrace` and
/// `RexxResetTrace` naming the thread a program runs on, from another thread and from a
/// function the program calls. Standard output and error are the oracle's, measured with the
/// same driver built against it.
#[test]
fn the_classic_family_answers_as_the_oracle_does() {
    let output = run_driver(
        "family",
        &[
            (
                "good.rex",
                "say \"good ran\" arg(1)\nreturn \"res\" arg(1)\n",
            ),
            ("bad.rex", "say \"bad\"\nif then\n"),
            ("nores.rex", "say \"noresult\"\n"),
            (
                "loop.rex",
                "signal on halt\ndo forever\n  if time(\"E\") > 5 then return \"not halted\"\nend\n\
                 halt: return \"halted at\" condition(\"D\")\n",
            ),
            (
                "self.rex",
                "signal on halt\nx = selfhalt()\nsay \"after selfhalt\" x\nreturn \"not halted\"\n\
                 halt: return \"halted\" x\n",
            ),
            (
                "trace.rex",
                "x = selftrace()\na = 1 + 1\ny = selftrace(1)\nb = trace()\nreturn x y b\n",
            ),
        ],
        &[],
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), FAMILY_STDOUT);
    let dir = scratch().join("driver-family");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        FAMILY_STDERR.replace("{dir}", &dir.display().to_string())
    );
}

/// `callrexx1`, the classic sample (`samples/api/classic/unix/callrexx`), built against our
/// libraries and run from a copy of its directory with every prompt answered: each `RexxStart`
/// answers as on the oracle -- results the program returns into a null, a large enough and a
/// too-small buffer, arguments, an in-store source and then its image alone -- through the
/// eighth call, which loads a macro. The oracle then ends with SIGSEGV running that macro from
/// the macrospace, where ours runs it and goes on to delete it.
#[test]
fn callrexx1_runs_as_on_the_oracle() {
    let sample = repository().join("third_party/ooRexx/samples/api/classic/unix/callrexx");
    let dir = scratch().join("callrexx1");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the sample's directory");
    for entry in std::fs::read_dir(&sample).expect("the sample's sources") {
        let path = entry.expect("an entry").path();
        if path.extension().is_some_and(|extension| extension == "rex") {
            std::fs::copy(&path, dir.join(path.file_name().expect("a name"))).expect("a copy");
        }
    }
    let api = scratch().join("api-callrexx1");
    let mut child = Command::new("timeout")
        .args(["-k", "5", "120"])
        .arg(testbinaries().join("callrexx1"))
        .current_dir(&dir)
        .env("REXX_API_DIR", &api)
        .env("TERM", "dumb")
        .env(
            "PATH",
            std::env::join_paths([bin_dir(), "/usr/bin".into(), "/bin".into()]).expect("a PATH"),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("callrexx1 runs");
    {
        use std::io::Write as _;
        let mut stdin = child.stdin.take().expect("its input");
        let _ = stdin.write_all(&[b'\n'; 20]);
    }
    let output = child.wait_with_output().expect("callrexx1 ends");
    stop_service(&api);
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let returned: Vec<&str> = stdout
        .lines()
        .filter(|line| line.starts_with("CALLREXX1 - "))
        .collect();
    let oracle: Vec<&str> = CALLREXX1_RETURNS.lines().collect();
    assert_eq!(returned.get(..oracle.len()), Some(&oracle[..]), "{stdout}");
    assert_eq!(returned.len(), oracle.len() + 9, "{stdout}");
    assert!(
        stdout.contains("Now inside the file macros.rex which is located in the REXX Macrospace"),
        "{stdout}"
    );
}

/// `callrexx1`'s report of each of its first eight `RexxStart` calls on the oracle.
const CALLREXX1_RETURNS: &str = "\
CALLREXX1 - Back from REXXSTART:  Return Code: 0
CALLREXX1 - RESULT-LENGTH:           8
CALLREXX1 - RESULT-Value:            99999999
CALLREXX1 - Back from REXXSTART:  Return Code: 0
CALLREXX1 - RESULT-LENGTH:           8
CALLREXX1 - RESULT-Value:            99999999
CALLREXX1 - Back from REXXSTART:  Return Code: 0
CALLREXX1 - RESULT-LENGTH:           8
CALLREXX1 - RESULT-Value:            99999999
CALLREXX1 - Back from REXXSTART:  Return Code: 0
CALLREXX1 - RESULT-LENGTH:           1
CALLREXX1 - RESULT-Value:            0
CALLREXX1 - Back from REXXSTART:  Return Code: 0
CALLREXX1 - RESULT-LENGTH:           1
CALLREXX1 - RESULT-Value:            0
CALLREXX1 - Back from REXXSTART:  Return Code: 0
CALLREXX1 - RESULT-LENGTH:           0
CALLREXX1 - RESULT-Value:            
CALLREXX1 - Back from REXXSTART:  Return Code: 0
CALLREXX1 - RESULT-LENGTH:           0
CALLREXX1 - RESULT-Value:            (null)
CALLREXX1 - Back from REXXSTART:  Return Code: 0
CALLREXX1 - RESULT-LENGTH:           0
CALLREXX1 - RESULT-Value:            (null)
";

/// A SIGINT while a classic `RexxStart` runs halts the program and leaves the process running,
/// the handler installed afterwards: an untrapped halt is reported as Error 4.1 and answers -4,
/// a trapped one returns its value. Measured on the oracle with the same driver; ours ended the
/// process (status 130) before `RexxStart` installed the handlers.
#[test]
fn a_sigint_during_rexx_start_halts_the_program_not_the_process() {
    let files = [
        (
            "sleep.rex",
            "say \"sleeping\"\ncall SysSleep 5\nsay \"woke\"\nreturn \"done\"\n",
        ),
        (
            "sleeph.rex",
            "signal on halt\nsay \"sleeping\"\ncall SysSleep 5\nsay \"woke\"\nreturn \"done\"\n\
             halt: return \"halted\"\n",
        ),
    ];
    let output = run_driver("sigint", &files, &["sleep.rex"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "before\nsleeping\nafter rc=-4 short=-4 result=(null)\nhandler afterwards: installed\n"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.ends_with("Error 4.1:  Program interrupted with HALT condition.\n"),
        "{stderr}"
    );
    let output = run_driver("sigint", &files, &["sleeph.rex"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "before\nsleeping\nafter rc=0 short=0 result=halted\nhandler afterwards: installed\n"
    );
}

/// The family driver's standard output on the oracle.
const FAMILY_STDOUT: &str = r#"version Open Object Rexx Version 5.3.0 r0
code 3001: Failure during initialization: File "&1" is unreadable.
code 35929: Missing expression in logical expression list.
code 40001: External routine "&1" failed.
code 0: (null)
code 3: (null)
code 99999: (null)
code 999013: Output file name must be different from input file name.
number 1: (null)
number 200: Failure during initialization: File "&1" is unreadable.
number 534: Failure during initialization: Program "&1" was not found.
number 0: (null)
number 99999: (null)
terminated 1
good ran x
good: rc=0 short=0 result=res x len=5
good ran lead
leading blank: rc=0 short=0 result=res lead len=8
noresult
no result: rc=0 short=0 result=(null) len=0
missing: rc=-3 short=-3 result=preset len=16
//T good: rc=0 short=-99 result=preset len=16
//t bad: rc=-35 short=-99 result=preset len=16
bad: rc=-35 short=-35 result=preset len=16
instore ran a
instore source: rc=0 short=0 result=is a len=4
image filled: yes
instore ran b
instore image: rc=0 short=0 result=is b len=4
macrospace missing: rc=-3 short=-3 result=preset len=16
instore bad: rc=-35 short=-35 result=preset len=16
translate instore rc=0 image=yes
instore ran c
translated image: rc=0 short=0 result=is c len=4
translate instore bad rc=-35 image=no
translate rc=0
good ran t
run translated: rc=0 short=0 result=res t len=5
compile encoded rc=0
good ran e
run encoded: rc=0 short=0 result=res e len=5
translate bad rc=-35 exists=0
translate missing rc=-3
translate no output rc=0
idle halt=1 trace=1 reset=1
halt from another thread=0
runner loop.rex: rc=0 short=0 result=halted at 
after the run=1
runner self.rex: rc=0 short=0 result=halted 0
runner trace.rex: rc=0 short=0 result=0 0 O
"#;

/// The family driver's standard error on the oracle, `{dir}` the directory it ran in.
const FAMILY_STDERR: &str = r#"Error 3:  Failure during initialization.
Error 3.901:  Failure during initialization: Program "nothere.rex" was not found.
     2 *-* if then
Error 35 running {dir}/bad.rex line 2:  Invalid expression.
Error 35.929:  Missing expression in logical expression list.
     2 *-* if then
Error 35 running {dir}/bad.rex line 2:  Invalid expression.
Error 35.929:  Missing expression in logical expression list.
Error 3:  Failure during initialization.
Error 3.1:  Failure during initialization: File "nores.rex" is unreadable.
     1 *-* if then
Error 35 running ibad line 1:  Invalid expression.
Error 35.929:  Missing expression in logical expression list.
     2 *-* if then
Error 35 running {dir}/bad.rex line 2:  Invalid expression.
Error 35.929:  Missing expression in logical expression list.
Error 3:  Failure during initialization.
Error 3.901:  Failure during initialization: Program "nothere.rex" was not found.
       +++ "LINUX COMMAND {dir}/trace.rex"
     2 *-* a = 1 + 1
       >>>   "2"
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
     3 *-* y = selftrace(1)
       >>>   "0"
"#;

/// The `rxmath` and `rxregexp` extension groups, on this crate's Rust libraries and its own
/// `rxregexp.cls`, with nothing on the library path: the oracle's counts, recorded from the full
/// checkout on 2026-11-03 (PR 4 plan, "Oracle targets").
#[test]
fn the_rust_extension_library_groups_match_the_oracles_counts() {
    for (dir, group, ran, assertions) in [
        ("extensions/rxmath", "RxMath", 289, 478),
        ("extensions/rxregexp", "rxregexp", 33, 11741),
    ] {
        let outcome = run_group(dir, group, library_path(false));
        assert_eq!(
            (outcome.ran, outcome.assertions, outcome.failed.len()),
            (ran, assertions, 0),
            "{group}:\n{}",
            outcome.stdout
        );
    }
}

/// The `rxsock` group on this crate's Rust library and its own `socket.cls`: the oracle's
/// counts, recorded from the full checkout on 2026-11-03 (PR 4 plan, "Oracle targets"). Its
/// client and server activities talk over loopback port 8083.
#[test]
fn the_rxsock_group_matches_the_oracles_counts() {
    let outcome = run_group("extensions/rxsock", "socketClass", library_path(false));
    assert_eq!(
        (outcome.ran, outcome.assertions, outcome.failed.len()),
        (23, 114, 0),
        "{}",
        outcome.stdout
    );
}

/// The `rxunixsys` group on this crate's Rust library: the oracle's counts and failing tests,
/// recorded from the full checkout on 2026-11-03 (PR 4 plan, "Oracle targets"). `TEST_SYSCRYPT`
/// fails on both, which provide no `SysCrypt` (Rulings). The six `TEST_SYSSTAT_*` failures are
/// the group's `socket` constant, `$XDG_RUNTIME_DIR/.ooRexx-...service`: the oracle's `rxapi`
/// socket, which this crate's service, under `REXX_API_DIR`, does not create. The run is given
/// an empty `XDG_RUNTIME_DIR` so that the path is absent whatever the host runs; measured, with
/// a socket there both engines pass those six, 30 / 176 / 0 + 1.
#[test]
fn the_rxunixsys_group_matches_the_oracles_counts_and_failures() {
    let runtime = scratch().join("rxunixsys-runtime");
    let _ = std::fs::remove_dir_all(&runtime);
    std::fs::create_dir_all(&runtime).expect("a runtime directory");
    let outcome = run_group_in(
        "extensions/platform/unix/rxunixsys",
        "SysUnix",
        library_path(false),
        &[("XDG_RUNTIME_DIR", &runtime)],
    );
    let failed: Vec<&str> = outcome.failed.iter().map(String::as_str).collect();
    assert_eq!(
        (outcome.ran, outcome.assertions, failed),
        (
            30,
            139,
            vec![
                "TEST_SYSCRYPT",
                "TEST_SYSSTAT_DEVICE",
                "TEST_SYSSTAT_IDS",
                "TEST_SYSSTAT_INODE",
                "TEST_SYSSTAT_NUMBERLINKS",
                "TEST_SYSSTAT_PERMISSIONS",
                "TEST_SYSSTAT_SIZE",
            ]
        ),
        "{}",
        outcome.stdout
    );
}
