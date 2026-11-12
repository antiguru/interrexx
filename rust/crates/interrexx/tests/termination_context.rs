//! **Termination runs on a thread context of its own.** The oracle creates an instance to
//! terminate on, runs the last-chance `UNINIT`s, and then unloads the packages, so the
//! last-chance `UNINIT`s and the package unloaders are handed a thread context that is
//! not the one the loader and the program's own native calls got, and a call through a
//! context kept from before termination answers nothing.
//!
//! The extension is the surface forge (a C++ extension under `tests/termination_context/`, its
//! routines `Stash`, `UseStash` and `Same`), compiled with `g++` against this worktree's `api/`
//! into a directory of this run's own, and the programs are run by this crate's `rexx` binary,
//! because the forge's loader, unloader and destructor write to the process's own standard output.
//! The expected lines are the oracle's, measured three times each with identical output, rc 0 and
//! empty stderr. The Rexx lines and the forge's lines are compared each in their own order, since
//! `SAY` output and an extension's `printf` interleave differently by decision.

#![cfg(target_os = "linux")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The `memcap` limit each run is under.
const MEMORY: &str = "2G";

fn worktree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// Builds the forge as `libforgea.so` in `dir`.
fn build_forge(dir: &Path) {
    let api = worktree().join("third_party/ooRexx/api");
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/termination_context/forge.cpp");
    let built = Command::new("g++")
        .args(["-shared", "-fPIC", "-O1", "-static-libstdc++"])
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg("-DTAG=\"forgea\"")
        .arg(&source)
        .arg("-o")
        .arg(dir.join("libforgea.so"))
        .status()
        .expect("g++ runs, which this test needs to build its extension");
    assert!(built.success(), "libforgea.so did not build");
}

/// A fresh directory of this run's own, named for `name`.
fn fresh_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a directory of this run's own");
    dir
}

/// Whether `name` is an executable file on this process's `PATH`.
fn on_path(name: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(name).is_file()))
}

/// Runs `source` with the forge on the library path, answering its Rexx
/// lines, the forge's lines, its stderr and its exit status.
fn run(dir: &Path, name: &str, source: &str) -> (Vec<String>, Vec<String>, String, Option<i32>) {
    let program = dir.join(format!("{name}.rex"));
    fs::write(&program, source).expect("the program is writable");
    let mut command = if on_path("memcap") {
        let mut command = Command::new("memcap");
        command.arg(MEMORY).arg(env!("CARGO_BIN_EXE_rexx"));
        command
    } else {
        Command::new(env!("CARGO_BIN_EXE_rexx"))
    };
    let output = command
        .arg(&program)
        .current_dir(dir)
        .env("LD_LIBRARY_PATH", dir)
        .stdin(Stdio::null())
        .output()
        .expect("the run spawns");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let (forge, rexx): (Vec<String>, Vec<String>) = stdout
        .lines()
        .map(str::to_owned)
        .partition(|line| line.starts_with("forgea "));
    (
        rexx,
        forge,
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code(),
    )
}

/// The forge's lines on every probe but the third: the loader's context is
/// not the one termination hands the unloader.
const FORGE_NOT_SAME: &[&str] = &[
    "forgea loader",
    "forgea loader answer null=0",
    "forgea unloader",
    "forgea unloader answer null=0 same=0",
    "forgea destructor",
];

#[test]
fn termination_hands_its_uninits_and_unloaders_a_context_of_its_own() {
    let dir = fresh_dir("termination-context");
    build_forge(&dir);
    let class = "::requires 'forgea' LIBRARY\n::class thing\n::method uninit\n";
    let cases: [(&str, String, &[&str], &[&str]); 4] = [
        // `uninit4.rex`: a context kept by the program, then a termination
        // `UNINIT` comparing its own with it and calling through it.
        (
            "uninit4",
            format!(
                "call stash\n.local~keep = .thing~new\nsay 'main'\n{class}\
                 say 'uninit local same' same()\nsay 'use' usestash()\n"
            ),
            &["main", "uninit local same 0"],
            FORGE_NOT_SAME,
        ),
        // `uninit5.rex`: the same against the loader's context.
        (
            "uninit5",
            format!(
                ".local~keep = .thing~new\nsay 'main'\n{class}\
                 say 'uninit local same' same()\nsay 'use' usestash()\n"
            ),
            &["main", "uninit local same 0"],
            FORGE_NOT_SAME,
        ),
        // A context kept inside a termination `UNINIT` is the one the
        // unloader is handed, and a call through it answers.
        (
            "uninit6",
            format!(
                "call stash\n.local~keep = .thing~new\nsay 'main'\n{class}\
                 say 'uninit stash' stash()\nsay 'uninit same' same()\n\
                 say 'use' usestash()\nsay 'after'\n"
            ),
            &["main", "uninit stash 1", "uninit same 1", "use 42", "after"],
            &[
                "forgea loader",
                "forgea loader answer null=0",
                "forgea unloader",
                "forgea unloader answer null=0 same=1",
                "forgea destructor",
            ],
        ),
        // The call through the context kept before termination answers no
        // object and raises nothing: the routine returning none is 44.1.
        (
            "uninit7",
            format!(
                "call stash\n.local~keep = .thing~new\nsay 'main'\n{class}\
                 signal on syntax\nsay 'use' usestash()\nsay 'after'\nreturn\n\
                 syntax:\nsay 'trapped' condition('o')~code\n"
            ),
            &["main", "trapped 44.1"],
            FORGE_NOT_SAME,
        ),
    ];
    for (name, source, rexx, forge) in cases {
        assert_eq!(
            run(&dir, name, &source),
            (
                rexx.iter().map(|line| (*line).to_owned()).collect(),
                forge.iter().map(|line| (*line).to_owned()).collect(),
                String::new(),
                Some(0)
            ),
            "{name}"
        );
    }
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}
