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
/// then a directory holding only the oracle's `rxregexp`, the C extension the framework itself
/// loads, so no other library of the oracle's is found.
fn library_path(with_testbinaries: bool) -> OsString {
    static FRAMEWORK: OnceLock<PathBuf> = OnceLock::new();
    let framework = FRAMEWORK.get_or_init(|| {
        let dir = scratch().join("framework-libraries");
        std::fs::create_dir_all(&dir).expect("the framework's library directory");
        let link = dir.join("librxregexp.so");
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(
            PathBuf::from(env!("REXX_ORACLE_ROOT")).join("lib/librxregexp.so"),
            &link,
        )
        .expect("the rxregexp link");
        dir
    });
    let mut directories = Vec::new();
    if with_testbinaries {
        directories.push(testbinaries().to_path_buf());
    }
    directories.push(framework.clone());
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
    copy(
        &repository().join("third_party/ooRexx/extensions/rxregexp/rxregexp.cls"),
        &run,
    );
    let api = scratch().join(format!("api-{group}"));
    let group_file = format!("ooRexx/{dir}/{group}.testGroup");
    let output = rexx(
        &["testOORexx.rex", "-f", &group_file, "-U", "-V", "2"],
        &run,
        &api,
        library_path,
    )
    .output()
    .expect("rexx runs");
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

/// Queue item 4: two interpreters in one process, `rexx` and one an extension starts with
/// `RexxStart`, each wait on a command when the process group takes a SIGINT, as a terminal's
/// Ctrl-C sends it. The kernel hands the process's signal to one of its threads, often not the
/// one waiting on the inner command, and that command's child ends at once; the inner command's
/// clause is halted all the same, never run on past. The miss is a race the thread the signal
/// went to loses when it is slow to run its handler, so the runs go in batches beside threads
/// that keep every CPU busy.
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "this test times and loads real runs"
)]
fn a_group_signal_halts_both_interpreters_waiting_on_commands() {
    const BATCHES: usize = 4;
    const AT_ONCE: usize = 30;
    let tester =
        std::fs::read_to_string(repository().join("ootest/ooRexx/API/oo/INVOCATIONTester.cls"))
            .expect("INVOCATIONTester.cls");
    let work = scratch().join("group-signal");
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).expect("a work directory");
    for (file, text) in [
        (
            "main.rex",
            "signal on halt\nparse arg id\n\
             a = .InvocationTester~new~start('callRexxStart', 'inner.rex', .array~of(id))\n\
             say 'started'\n'sleep 10'\nsay 'outer missed' rc a~result\nexit\n\
             halt: say 'outer halted' a~result\n::requires 'INVOCATIONTester.cls'\n",
        ),
        (
            "inner.rex",
            "signal on halt\nparse arg id\ncall lineout 'inner.'id, 'x'\ncall lineout 'inner.'id\n\
             'sleep 10'\nreturn 'inner missed' rc\nhalt: return 'inner halted'\n",
        ),
        ("INVOCATIONTester.cls", &tester),
    ] {
        std::fs::write(work.join(file), text).expect("a file is written");
    }
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
                let child = Command::new(REXX)
                    .args(["main.rex", &id])
                    .current_dir(&work)
                    .env("REXX_API_DIR", &api)
                    .env("LD_LIBRARY_PATH", library_path(true))
                    .stdin(Stdio::null())
                    .stdout(std::fs::File::create(&out).expect("the output file"))
                    .stderr(Stdio::null())
                    .process_group(0)
                    .spawn()
                    .expect("rexx starts");
                (id, out, child)
            })
            .collect();
        let signalled: Vec<_> = runs
            .into_iter()
            .map(|(id, out, mut child)| {
                let began = std::time::Instant::now();
                while !(work.join(format!("inner.{id}")).exists()
                    && std::fs::read_to_string(&out).is_ok_and(|text| text.contains("started")))
                    && began.elapsed() < deadline
                {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                // Both commands have started.
                std::thread::sleep(std::time::Duration::from_millis(300));
                let group = format!("-{}", child.id());
                let sent = Command::new("kill")
                    .args(["-INT", "--", &group])
                    .status()
                    .is_ok_and(|status| status.success());
                if !sent {
                    let _ = child.kill();
                }
                (id, out, child)
            })
            .collect();
        for (id, out, mut child) in signalled {
            let began = std::time::Instant::now();
            while child.try_wait().ok().flatten().is_none() && began.elapsed() < deadline {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            let _ = child.kill();
            let _ = child.wait();
            let stdout = std::fs::read_to_string(&out).unwrap_or_default();
            if stdout != "started\nouter halted inner halted\n" {
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
/// `ProcessInvocation` 54 and 284. `TestLoadLibrary` alone fails, one assertion short: it loads
/// `rxmath`, which is not on the library path the harness gives.
#[test]
fn the_invocation_groups_run_programs_in_instances() {
    for (group, ran, assertions) in [("INVOCATION", 55, 294), ("ProcessInvocation", 54, 283)] {
        let outcome = run_group("API/oo", group, library_path(true));
        let failed: Vec<&str> = outcome.failed.iter().map(String::as_str).collect();
        assert!(
            failed == ["TESTLOADLIBRARY"] && outcome.ran == ran && outcome.assertions == assertions,
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
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/classic_api/{name}.cpp"));
    let out = testbinaries().join(name);
    let api = repository().join("third_party/ooRexx/api");
    let output = Command::new("c++")
        .arg("-o")
        .arg(&out)
        .arg(&source)
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(format!("-L{}", testbinaries().display()))
        .arg(format!("-Wl,-rpath,{}", testbinaries().display()))
        .args(["-pthread", "-lrexx", "-lrexxapi"])
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
    let binary = driver(name);
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
        .stdin(Stdio::null())
        .output()
        .expect("the driver runs");
    stop_service(&api);
    output
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
