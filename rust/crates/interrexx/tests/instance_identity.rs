//! **One interpreter instance for every activity** (`r[native.threads.instance]`). The extension
//! `tests/instance_identity/inst.cpp`, compiled with `g++` against this worktree's `api/` into a
//! directory of this run's own, answers the instance its thread context links and the application
//! data that instance carries. `inst.rex` asks it from the main activity, from the activity
//! `REPLY` makes and from one `start` makes, and prints `same` where all three answers are the
//! main activity's. Under `REXX_CORPUS_GATE` the oracle runs the same program and prints the same.
//! Built as a program, the fixture creates an interpreter with `APPLICATION_DATA` set and runs
//! `inst.rex` through it, so the data reaches every activity of an embedding.

#![cfg(target_os = "linux")]

mod support;

#[path = "../../interrexx-capi-rxapi/tests/support/cdylibs.rs"]
mod cdylibs;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The `memcap` limit each run is under.
const MEMORY: &str = "2G";

const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    std::env::var(GATE_ENV).is_ok_and(|value| !value.is_empty() && value != "0")
}

fn worktree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/instance_identity")
        .join(name)
}

/// A fresh directory of this run's own, named for `name`.
fn fresh_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a directory of this run's own");
    dir
}

/// `g++` compiling the fixture against this worktree's `api/`, with `arguments` after the source.
fn compile(arguments: &[&str]) {
    let api = worktree().join("third_party/ooRexx/api");
    let built = Command::new("g++")
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(fixture("inst.cpp"))
        .args(arguments)
        .status()
        .expect("g++ runs, which this test needs to build its fixture");
    assert!(built.success(), "the fixture did not build: {arguments:?}");
}

/// Runs `program` with `argument` under `binary`, memory-capped by `memcap`, from `dir`, with `dir`
/// on the library path, answering stdout, stderr and the exit status.
fn run(binary: &Path, dir: &Path, program: &Path, argument: Option<&str>) -> (String, String, i32) {
    let output = Command::new("memcap")
        .arg(MEMORY)
        .arg(binary)
        .arg(program)
        .args(argument)
        .current_dir(dir)
        .env("LD_LIBRARY_PATH", dir)
        .env("REXX_API_DIR", dir.join("api"))
        .stdin(Stdio::null())
        .output()
        .expect("memcap runs, which every run of this test is under");
    let _ = Command::new(Path::new(env!("CARGO_BIN_EXE_rexx")).with_file_name("rxapi"))
        .arg("--stop")
        .env("REXX_API_DIR", dir.join("api"))
        .output();
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
    )
}

#[test]
fn one_instance_for_every_activity() {
    let dir = fresh_dir("instance-identity");
    compile(&[
        "-shared",
        "-fPIC",
        "-O1",
        "-static-libstdc++",
        "-o",
        &dir.join("libinst.so").to_string_lossy(),
    ]);
    let program = fixture("inst.rex");
    assert_eq!(
        run(Path::new(env!("CARGO_BIN_EXE_rexx")), &dir, &program, None),
        ("same\n".to_owned(), String::new(), 0)
    );
    if gate_mode() {
        let oracle = support::oracle::locate();
        let library_path = format!("{}:{}", oracle.lib_dir().display(), dir.display());
        let theirs = oracle.run_in(&program, &dir, &[("LD_LIBRARY_PATH", &library_path)]);
        assert_eq!(
            (
                String::from_utf8_lossy(&theirs.stdout).into_owned(),
                String::from_utf8_lossy(&theirs.stderr).into_owned(),
                theirs.expect_exit_code(),
            ),
            ("same\n".to_owned(), String::new(), 0),
            "the oracle"
        );
    }
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}

#[test]
fn application_data_reaches_a_started_activity() {
    let dir = fresh_dir("instance-application-data");
    // This workspace's `librexx.so.4`, under the name the driver links and the name it loads.
    for library in [cdylibs::librexx(), cdylibs::librexxapi()] {
        let name = library.file_name().expect("a library name").to_owned();
        let mut versioned = name.clone();
        versioned.push(".4");
        fs::copy(&library, dir.join(&versioned)).expect("a copy of the library");
        std::os::unix::fs::symlink(&versioned, dir.join(&name)).expect("the link name");
    }
    let driver = dir.join("instdriver");
    compile(&[
        "-DDRIVER",
        "-pthread",
        "-o",
        &driver.to_string_lossy(),
        &format!("-L{}", dir.display()),
        &format!("-Wl,-rpath,{}", dir.display()),
        "-lrexx",
        "-lrexxapi",
    ]);
    assert_eq!(
        run(&driver, &dir, &fixture("inst.rex"), None),
        ("same\ndata set\n".to_owned(), String::new(), 0)
    );
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}
