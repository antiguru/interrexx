//! The `rxunixsys` library run in Rust, against the oracle's
//! `librxunixsys.so` (`docs/superpowers/specs/extensions-behaviour.md`,
//! "rxunixsys"). Every program runs as its own `rexx` process, because these
//! routines change process state (the umask, the root, the session), from a
//! fresh directory, `{dir}` in a case's text.
//!
//! * `eval`: each input line is an expression, shown as `"value"`, as
//!   `Array[n]: a|b` for an `Array`, or as `ERR code message` for the SYNTAX
//!   condition it raises; the program requires `rxunixsys` LIBRARY.
//! * `program`: the input is the whole program; the case records rc, stdout
//!   and stderr.
//! * `ruled`: as `program`, where this crate's answer is not the oracle's by
//!   a ruling the case's comment names: the oracle crashes (Deviation 88),
//!   reports an `errno` of its own I/O (Rulings, errno), or the routine is
//!   refused (Deviation 90).
//!
//! Each case gets an API service of its own, stopped after it, so that a
//! registration `RxFuncAdd` made in one case (or a failed run) is not seen
//! by the next.
//!
//! Recording: `RXUNIXSYS_RECORD=1 REWRITE=1` runs `eval` and `program`
//! cases on the oracle (`REXX_ORACLE_ROOT`) instead, each with an
//! `XDG_RUNTIME_DIR` of its own under the scratch directory, and `ruled`
//! cases on this crate. The oracle's `rxapi` daemons it starts are left
//! running and are the recorder's to stop.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const CASES: &str = "tests/rxunixsys_results";

/// The scratch directory every run of this test binary works under: short,
/// because the API service's socket lives in it.
fn scratch() -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("rxunixsys");
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

/// The program an `eval` case runs.
fn eval_program(lines: &[&str]) -> String {
    let mut program = String::new();
    for (index, line) in lines.iter().enumerate() {
        program.push_str(&format!(
            "signal on syntax name s{index}\nv = {line}\ncall show v\nsignal n{index}\n\
             s{index}: say 'ERR' condition('o')~code condition('o')~message\nn{index}: nop\n"
        ));
    }
    program.push_str(
        "exit\nshow: procedure\n  use arg v\n  if v~isA(.array) then \
         say 'Array['v~items']:' v~toString('L', '|')\n  else say '\"'v'\"'\n  return\n\
         ::requires 'rxunixsys' LIBRARY\n",
    );
    program
}

/// Runs `program` from a fresh directory, on the oracle where `oracle`.
fn run(name: usize, program: &str, oracle: bool) -> (i32, String, String) {
    let root = scratch();
    let dir = root.join(format!("case{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("sub")).expect("a case directory");
    std::fs::write(dir.join("f"), b"hello\n").expect("a case file");
    let program = program.replace("{dir}", &dir.display().to_string());
    let path = dir.join("probe.rex");
    std::fs::write(&path, &program).expect("the program");
    let api = root.join(format!("api{name}"));
    let _ = std::fs::remove_dir_all(&api);
    std::fs::create_dir_all(&api).expect("a service directory");
    let mut command;
    if oracle {
        let oracle_root = Path::new(env!("REXX_ORACLE_ROOT"));
        command = Command::new(oracle_root.join("bin/rexx"));
        command
            .env("LD_LIBRARY_PATH", oracle_root.join("lib"))
            .env("XDG_RUNTIME_DIR", &api);
    } else {
        command = Command::new(env!("CARGO_BIN_EXE_rexx"));
        command
            .env("REXX_API_DIR", &api)
            .env_remove("LD_LIBRARY_PATH");
    }
    let output = command
        .arg("probe.rex")
        .current_dir(&dir)
        .env("TZ", "UTC")
        .env_remove("TZDIR")
        .stdin(Stdio::null())
        .output()
        .expect("rexx runs");
    if !oracle {
        let _ = Command::new(Path::new(env!("CARGO_BIN_EXE_rexx")).with_file_name("rxapi"))
            .arg("--stop")
            .env("REXX_API_DIR", &api)
            .output();
        let _ = std::fs::remove_dir_all(&api);
    }
    let _ = std::fs::remove_dir_all(&dir);
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).replace(&dir.display().to_string(), "{dir}"),
        String::from_utf8_lossy(&output.stderr).replace(&dir.display().to_string(), "{dir}"),
    )
}

#[test]
fn rxunixsys_answers_as_the_oracle_does() {
    let record = std::env::var_os("RXUNIXSYS_RECORD").is_some();
    let mut cases = 0;
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            cases += 1;
            let lines: Vec<&str> = case.input.lines().collect();
            match case.directive.as_str() {
                "eval" => {
                    let (rc, stdout, stderr) = run(cases, &eval_program(&lines), record);
                    assert_eq!(rc, 0, "{stderr}");
                    stdout
                }
                "program" | "ruled" => {
                    let oracle = record && case.directive == "program";
                    let (rc, stdout, stderr) =
                        run(cases, &format!("{}\n", case.input.trim_end()), oracle);
                    format!("rc {rc}\n[stdout]\n{stdout}[stderr]\n{stderr}")
                }
                other => panic!("unknown directive {other}"),
            }
        });
    });
    assert!(cases > 0, "no case ran");
}
