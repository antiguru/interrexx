//! ooTest's classic-API groups on `rexx`, through the `testbinaries/` libraries compiled by
//! `scripts/build-testbinaries.sh` against this workspace's `librexx.so.4` and
//! `librexxapi.so.4`. Each run has a private `REXX_API_DIR` and stops its daemon after.

use std::collections::BTreeSet;
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

/// A group's outcome: how many tests ran and which failed or errored.
struct Outcome {
    ran: usize,
    failed: BTreeSet<String>,
    stdout: String,
}

/// Runs `group` of `dir` through `testOORexx.rex`, from a fresh copy of the framework and the
/// group's directory.
fn run_group(dir: &str, group: &str) -> Outcome {
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
    let oracle_lib = PathBuf::from(env!("REXX_ORACLE_ROOT")).join("lib");
    let library_path =
        std::env::join_paths([testbinaries().to_path_buf(), oracle_lib]).expect("a library path");
    let bin = Path::new(REXX)
        .parent()
        .expect("rexx's directory")
        .to_path_buf();
    let path = std::env::join_paths(std::iter::once(bin.clone()).chain(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    )))
    .expect("a PATH");
    let output = Command::new("timeout")
        .args(["-k", "5", "600"])
        .arg(REXX)
        .arg("testOORexx.rex")
        .arg("-f")
        .arg(format!("ooRexx/{dir}/{group}.testGroup"))
        .args(["-U", "-V", "2"])
        .current_dir(&run)
        .env("REXX_API_DIR", &api)
        .env("LD_LIBRARY_PATH", library_path)
        .env("PATH", path)
        .env_remove("RXQUEUESESSION")
        .env_remove("RXQUEUE")
        .stdin(Stdio::null())
        .output()
        .expect("rexx runs");
    let _ = Command::new(bin.join("rxapi"))
        .arg("--stop")
        .env("REXX_API_DIR", &api)
        .output();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let ran = stdout
        .lines()
        .find_map(|line| line.strip_prefix("Tests ran:"))
        .and_then(|count| count.trim().parse().ok())
        .unwrap_or_else(|| {
            panic!(
                "no test count:\n{stdout}\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
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
        ran,
        failed,
        stdout,
    }
}

fn classic() -> &'static Outcome {
    static OUTCOME: OnceLock<Outcome> = OnceLock::new();
    OUTCOME.get_or_init(|| run_group("API/classic", "CLASSIC"))
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
