//! ooTest's classic-API groups on `rexx`, through the `testbinaries/` libraries compiled by
//! `scripts/build-testbinaries.sh` against this workspace's `librexx.so.4` and
//! `librexxapi.so.4`. Each run has a private `REXX_API_DIR` and stops its daemon after. `rxapi`
//! and `rxsubcom` are the `rexx-rxapi` binaries beside `rexx`, which a workspace build makes.

use std::collections::BTreeSet;
use std::ffi::OsString;
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
    let path = std::env::join_paths(std::iter::once(bin_dir()).chain(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    )))
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
