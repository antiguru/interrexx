//! The `rxsock` library and its packages run in Rust, against the oracle's
//! `librxsock.so` (`docs/design/extensions.md`). Every program runs as its own `rexx` process from a fresh
//! directory, `{dir}` in a case's text, and talks to nothing but loopback
//! sockets it opens itself. Descriptor numbers are never printed: this
//! crate's process holds one more descriptor than the oracle's at start.
//!
//! * `eval`: each input line is an expression, shown as `"value"`, as
//!   `Array[n]: a|b` for an `Array`, or as `ERR code message` for the SYNTAX
//!   condition it raises; the program requires `rxsock` LIBRARY.
//! * `eval-packages`: as `eval`, the program requiring `socket.cls` and
//!   `streamsocket.cls` instead.
//! * `program`: the input is the whole program; the case records rc, stdout
//!   and stderr.
//! * `ruled`: as `program`, where this crate's answer is not the oracle's by
//!   a ruling the case's comment names: the oracle crashes (Deviation 88), or
//!   reports an `errno` of its own I/O (Rulings, errno).
//!
//! The packages a program requires (`socket.cls` and the rest) are this
//! crate's on both sides: the clean tree's oracle finds them through links in
//! its `bin` directory.
//!
//! Each case gets an API service of its own, stopped after it.
//!
//! Recording: `RXSOCK_RECORD=1 REWRITE=1` runs `eval` and `program` cases on
//! the oracle (`REXX_ORACLE_ROOT`) instead, each with an `XDG_RUNTIME_DIR` of
//! its own under the scratch directory, and `ruled` cases on this crate. The
//! oracle's `rxapi` daemons it starts are left running and are the
//! recorder's to stop.

mod support;

use std::path::PathBuf;
use std::process::Stdio;

use support::rexx_service;

const CASES: &str = "tests/rxsock_results";

/// The scratch directory every run of this test binary works under: short,
/// because the API service's socket lives in it.
fn scratch() -> PathBuf {
    rexx_service::scratch("rxsock")
}

/// The program an `eval` case runs, requiring `requires`.
fn eval_program(lines: &[&str], requires: &str) -> String {
    let mut program = String::new();
    for (index, line) in lines.iter().enumerate() {
        program.push_str(&format!(
            "signal on syntax name s{index}\nv = {line}\ncall show v\nsignal n{index}\n\
             s{index}: say 'ERR' condition('o')~code condition('o')~message\nn{index}: nop\n"
        ));
    }
    program.push_str(
        "exit\nshow: procedure\n  use arg v\n  if v~isA(.array) then \
         say 'Array['v~items']:' v~toString('L', '|')\n  else say '\"'v'\"'\n  return\n",
    );
    program.push_str(requires);
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
    let api = rexx_service::service_dir(&root, name);
    let rexx = if oracle {
        rexx_service::oracle_rexx()
    } else {
        rexx_service::own_rexx().to_path_buf()
    };
    let mut command = rexx_service::with_service(rexx_service::bounded(&rexx), oracle, &api);
    if oracle {
        // The packages' links, which `::REQUIRES` finds on the `PATH`.
        let path = std::env::var_os("PATH").unwrap_or_default();
        let mut directories = vec![rexx_service::oracle_root().join("bin")];
        directories.extend(std::env::split_paths(&path));
        command.env("PATH", std::env::join_paths(directories).expect("a PATH"));
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
        rexx_service::stop_own_service(&api);
    }
    let _ = std::fs::remove_dir_all(&dir);
    // The oracle's message lines are mapped to this crate's texts first.
    let channel = |bytes: &[u8]| {
        let bytes = if oracle {
            support::messages::oracle_output(bytes)
        } else {
            bytes.to_vec()
        };
        String::from_utf8_lossy(&bytes).replace(&dir.display().to_string(), "{dir}")
    };
    (
        output.status.code().unwrap_or(-1),
        channel(&output.stdout),
        channel(&output.stderr),
    )
}

#[test]
fn rxsock_answers_as_the_oracle_does() {
    let record = std::env::var_os("RXSOCK_RECORD").is_some();
    let mut cases = 0;
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            cases += 1;
            let lines: Vec<&str> = case.input.lines().collect();
            match case.directive.as_str() {
                "eval" | "eval-packages" => {
                    let requires = match case.directive.as_str() {
                        "eval" => "::requires 'rxsock' LIBRARY\n",
                        _ => "::requires 'socket.cls'\n::requires 'streamsocket.cls'\n",
                    };
                    let program = eval_program(&lines, requires);
                    let (rc, stdout, stderr) = run(cases, &program, record);
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
    rexx_service::remove_scratch("rxsock");
}
