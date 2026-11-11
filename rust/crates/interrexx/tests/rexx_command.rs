//! The `rexx` command line against the oracle's: each case of
//! `tests/rexx_command/cases` runs both binaries with the same words from a
//! directory of the same programs, and the three descriptors must agree. The
//! recorded block is the oracle's, normalised; the `-v` banner is
//! interrexx's own (Deviation 96), so the oracle's is mapped to it first.
//!
//! A case's input is its words, one per line: `\xHH` is a byte, and a line
//! `<empty>` is the empty word. `rexx_path=DIR` sets `REXX_PATH` to the
//! fixture's `DIR`, `path=DIR` puts it in front of `PATH`, and `cwd=DIR` runs
//! from it. `DIR` is a name [`directory`] spells. `env=NAME.VALUE`, which may
//! list several words, sets each environment variable `NAME` to `VALUE` for
//! both binaries and leaves the rest of the environment as it is, `RXTRACE`
//! apart, which every run starts without; a case file word holds only
//! letters, digits, `-`, `_` and `.`, so a `_` in `VALUE` stands for a blank,
//! and `env=NAME.` sets the empty value.

#[path = "support/banner.rs"]
mod banner;
#[path = "support/oracle_runtime.rs"]
mod oracle_runtime;

use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const CASE_DIR: &str = "tests/rexx_command";

/// The oracle's address-space cap, in KiB.
const ORACLE_MEMORY_LIMIT_KIB: u64 = 1_048_576;

const ORACLE_ROOT: &str = env!("REXX_ORACLE_ROOT");

/// The programs every case runs among.
fn lay_fixture(dir: &Path) {
    let file = |name: &[u8], text: &str| {
        let path = dir.join(PathBuf::from(OsString::from_vec(name.to_vec())));
        std::fs::write(path, text).expect("a fixture file");
    };
    for sub in ["sub", "rp", "pp", "dironly.rex", "rp/sub2"] {
        std::fs::create_dir_all(dir.join(sub)).expect("a fixture directory");
    }
    file(b"t.rex", "say 'hi from t'\n");
    // The search rules: endings, case, places and the caller's directory.
    file(b"ord.rex", "say 'ord rex'\n");
    file(b"ord.REX", "say 'ord REX'\n");
    file(b"ordc.cls", "say 'cls was tried'\n");
    file(b"ordx.rex", "say 'ordx rex in the current directory'\n");
    file(b"rp/ordx.REX", "say 'ordx REX in REXX_PATH'\n");
    file(b"rp/ordy.rex", "say 'ordy in REXX_PATH'\n");
    file(b"pp/ordy.rex", "say 'ordy in PATH'\n");
    file(b"rp/sub2/inrp.rex", "say 'inrp'\n");
    file(b"ordn.rex", "say 'ordn rex'\n");
    file(b"ordn", "say 'ordn none'\n");
    file(b"ordm", "say 'ordm none in the current directory'\n");
    file(b"rp/ordm.rex", "say 'ordm rex in REXX_PATH'\n");
    file(b"lowonly.rex", "say 'lowonly'\n");
    file(b"sub/callsib.rex", "say sibonly()\n");
    file(b"sub/sibonly.rex", "return 'sibling in sub'\n");
    file(b"sub/t.rex", "say 'hi from sub/t'\n");
    file(b"rp/viarp.rex", "say 'hi from rp'\nparse source s; say s\n");
    file(b"pp/viapp.rex", "say 'hi from pp'\nparse source s; say s\n");
    file(b"UP.REX", "say 'upper UP.REX'\n");
    file(b"A3UPONLY.REX", "say 'upper A3UPONLY.REX'\n");
    file(b"noext", "say 'noext file'\nparse source s; say s\n");
    // The RXTRACE variable: the main program, a program called by name, and a
    // method each answer the setting they started with.
    file(
        b"rxtrace.rex",
        ".traceoutput~destination(.quiet~new)\nsay 'main' trace()\nsay 'external' rxtrace2()\nsay 'method' .t~new~m\nexit\n::class quiet\n::method lineout\n  return 0\n::class t\n::method m\n  return trace()\n",
    );
    file(b"rxtrace2.rex", "return trace()\n");
    // A program called by name under the variable is announced with `>I>` and `<I<`.
    file(b"rxtraceannounce.rex", "call rxtraceannounce2\n");
    file(b"rxtraceannounce2.rex", "say 'sub'\n");
    // A `::ROUTINE` is called by name, as a function, and as a routine object: each
    // starts with the setting the variable gives it.
    file(
        b"rxtraceroutine.rex",
        ".traceoutput~destination(.quiet~new)\nsay 'main' trace()\nsay 'function' rt()\ncall rt2\nsay 'object' .routine~new('o', 'return trace()')~call\nexit\n::class quiet\n::method lineout\n  return 0\n::routine rt\n  return trace()\n::routine rt2\n  say 'call' trace()\n",
    );
    file(b"src.rex", "parse source s; say s\n");
    file(
        b"showargs.rex",
        "say 'arg()='arg() 'arg(1)=['arg(1)']'\n\
         s = .syscargs\n\
         say 'items='s~items 'size='s~size 'last='s~last\n\
         do i = 1 to s~size\n\
         \x20 if s~hasIndex(i) then say i'=['s[i]']'\n\
         \x20 else say i' empty'\n\
         end\n\
         say '['s~toString(, ', ')']'\n",
    );
    file(
        b"opts.rex",
        "say 'digits='digits() 'form='form()\nsay 'trace='trace()\n",
    );
    file(
        b"ownopts.rex",
        "say 'digits='digits()\n::options digits 5\n",
    );
    file(
        b"req.rex",
        "say 'main digits='digits()\ncall libr\n::requires 'lib.rex'\n",
    );
    file(
        b"lib.rex",
        "::routine libr public\nsay 'lib digits='digits()\n",
    );
    file(b"calllib.rex", "call libr\n");
    file(
        b"plib.rex",
        "say 'prolog ran'\n::routine pr public\nsay 'pr'\n",
    );
    file(b"err42.rex", "say 1/0\n");
    file(b"err35.rex", "say 'x' +\n");
    for (name, body) in [
        ("abc", "return 'abc'"),
        ("neg3", "return -3"),
        ("e300", "exit 300"),
        ("f15", "return 1.5"),
        ("big", "return 2147483648"),
        ("nbig", "return -2147483649"),
        ("null", "return ''"),
        ("none", "nop"),
        ("bare", "exit"),
        ("r255", "return 255"),
        ("r256", "return 256"),
        ("sp", "return ' 4 '"),
        ("e1", "return 1e1"),
        ("obj", "return .array~new"),
        ("33000", "exit 33000"),
        ("70000", "exit 70000"),
    ] {
        file(format!("ret_{name}.rex").as_bytes(), &format!("{body}\n"));
    }
    let failing = "parse source s; say s\nsay 1/0\n";
    file(b"my prog.rex", failing);
    file(b"p\xff.rex", failing);
    file(b"q\xff", failing);
    file(
        b"local.rex",
        "say .local~items\nsay .local~allIndexes~sort~makeString('L', ', ')\n",
    );
    for sub in [&b"d\xc3\xa9"[..], b"d\xff", "d\u{20ac}".as_bytes()] {
        let dir_name = PathBuf::from(OsString::from_vec(sub.to_vec()));
        std::fs::create_dir_all(dir.join(&dir_name)).expect("a fixture directory");
        let inside = |name: &str| [sub, b"/", name.as_bytes()].concat();
        file(
            &inside("req.rex"),
            "say 'main'\ncall rqr\n::requires \"rq.rex\"\n",
        );
        file(
            &inside("rq.rex"),
            "::routine rqr public\nparse source s; say s\n",
        );
        file(&inside("ext.rex"), "say 'main'\ncall other\n");
        file(&inside("other.rex"), "parse source s; say s\n");
    }
    for sub in [&b"rp\xc3\xa9"[..], b"rp\xff"] {
        let dir_name = PathBuf::from(OsString::from_vec(sub.to_vec()));
        std::fs::create_dir_all(dir.join(&dir_name)).expect("a fixture directory");
        file(
            &[sub, b"/farcall.rex"].concat(),
            "say 'far'\nparse source s; say s\nsay 1/0\n",
        );
    }
    file(b"farmain.rex", "call farcall\n");
    file(b"sub/ps.rex", "parse source s; say s\nsay 1/0\n");
    std::os::unix::fs::symlink("sub/ps.rex", dir.join("link.rex")).expect("a link");
    std::os::unix::fs::symlink("sub", dir.join("lsub")).expect("a directory link");
    file(b"bytes.rex", "say c2x(arg(1))\nsay c2x(.syscargs[1])\n");
    file(b"locked.rex", "say 'read'\n");
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(
        dir.join("locked.rex"),
        std::fs::Permissions::from_mode(0o000),
    )
    .expect("the unreadable program");
}

/// The fixture directory an argument names: one of the non-ASCII names
/// `datadriven` cannot carry in an argument, or the name itself.
fn directory(name: &str) -> PathBuf {
    let bytes: &[u8] = match name {
        "d_euro" => "d\u{20ac}".as_bytes(),
        "rp_e_acute" => b"rp\xc3\xa9",
        "rp_ff" => b"rp\xff",
        other => other.as_bytes(),
    };
    PathBuf::from(OsString::from_vec(bytes.to_vec()))
}

/// A case's words.
fn words(input: &str) -> Vec<OsString> {
    input
        .lines()
        .map(|line| {
            if line == "<empty>" {
                return OsString::new();
            }
            let mut bytes = Vec::new();
            let mut rest = line.as_bytes();
            while let Some((&byte, tail)) = rest.split_first() {
                if let Some(hex) = rest.strip_prefix(b"\\x").filter(|hex| hex.len() >= 2) {
                    let digits = std::str::from_utf8(&hex[..2]).expect("hex digits");
                    bytes.push(u8::from_str_radix(digits, 16).expect("a hex byte"));
                    rest = &hex[2..];
                } else {
                    bytes.push(byte);
                    rest = tail;
                }
            }
            OsString::from_vec(bytes)
        })
        .collect()
}

/// `bytes` with every byte outside printable ASCII as `\xHH`.
fn escaped(bytes: &[u8]) -> String {
    let mut out = String::new();
    for &byte in bytes {
        if byte == b'\n' || (0x20..0x7f).contains(&byte) && byte != b'\\' {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("\\x{byte:02X}"));
        }
    }
    out
}

/// The three descriptors as the case file records them, with the fixture's
/// directory as `DIR` and the banner as [`banner::recorded`] shows it.
fn render(dir: &Path, stdout: &[u8], stderr: &[u8], status: Option<i32>, oracle: bool) -> String {
    let dir = escaped(dir.as_os_str().as_encoded_bytes());
    let mut rendered = match status {
        Some(code) => format!("rc> {code}\n"),
        None => "rc> none\n".to_string(),
    };
    for (tag, bytes) in [("out", stdout), ("err", stderr)] {
        for line in escaped(bytes).replace(&dir, "DIR").lines() {
            let Some(line) = banner::recorded(line, oracle) else {
                continue;
            };
            rendered.push_str(&format!("{tag}> {line}\n"));
        }
    }
    rendered
}

/// Runs `program` with `args` from `dir` under `timeout`.
fn run(
    program: &Path,
    args: &[OsString],
    dir: &Path,
    case: &datadriven::TestCase,
    oracle: bool,
) -> String {
    let mut command = Command::new("timeout");
    command.args(["-k", "5", "20"]);
    if oracle {
        command
            .arg("sh")
            .arg("-c")
            .arg(format!(
                "ulimit -v {ORACLE_MEMORY_LIMIT_KIB} && exec \"$0\" \"$@\""
            ))
            .env("LD_LIBRARY_PATH", Path::new(ORACLE_ROOT).join("lib"))
            .env("XDG_RUNTIME_DIR", oracle_runtime::dir());
    }
    command
        .arg(program)
        .args(args)
        .current_dir(dir)
        .env_remove("REXX_PATH")
        .env_remove("RXTRACE")
        .stdin(Stdio::null());
    for (key, values) in &case.args {
        if key == "env" {
            for word in values {
                let (name, value) = word.split_once('.').expect("env=NAME.VALUE");
                command.env(name, value.replace('_', " "));
            }
            continue;
        }
        let entry = dir.join(directory(values.first().expect("a directory")));
        match key.as_str() {
            "cwd" => command.current_dir(entry),
            "rexx_path" => command.env("REXX_PATH", entry),
            "path" => {
                let mut path = entry.into_os_string();
                path.push(":");
                path.push(std::env::var_os("PATH").unwrap_or_default());
                command.env("PATH", path)
            }
            other => panic!("unknown argument {other}"),
        };
    }
    let output = command.output().expect("the binary starts");
    render(
        dir,
        &output.stdout,
        &output.stderr,
        output.status.code(),
        oracle,
    )
}

/// A fresh fixture directory for one side.
fn fixture(side: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("rexx-command-{side}-{}", std::process::id()));
    if dir.exists() {
        remove(&dir);
    }
    std::fs::create_dir_all(&dir).expect("the fixture directory");
    lay_fixture(&dir);
    dir
}

/// Removes a fixture, the unreadable program's mode first.
fn remove(dir: &Path) {
    use std::os::unix::fs::PermissionsExt as _;
    let _ = std::fs::set_permissions(
        dir.join("locked.rex"),
        std::fs::Permissions::from_mode(0o644),
    );
    std::fs::remove_dir_all(dir).expect("the fixture is removed");
}

// The cases cover the rules of the invocation page, each by the group of
// cases whose heading comment in `rexx_command/cases` names its rule.
// spec: env.invocation.forms, env.invocation.words, env.invocation.version
// spec: env.invocation.expression, env.invocation.optstring
// spec: env.invocation.program, env.invocation.notfound
// spec: env.invocation.arguments, env.invocation.syscargs
// spec: env.invocation.status
// spec: env.invocation.deviation.96
// spec: env.search.program, env.search.names, env.search.extension
// spec: env.search.case, env.search.places, env.search.caller
// spec: env.debugging.rxtrace
#[test]
fn the_command_line_answers_as_the_oracles_does() {
    let oracle = Path::new(ORACLE_ROOT).join("bin/rexx");
    assert!(
        oracle.is_file(),
        "the oracle is missing at {}",
        oracle.display()
    );
    let ours = Path::new(env!("CARGO_BIN_EXE_rexx"));
    let (theirs_dir, ours_dir) = (fixture("oracle"), fixture("crate"));
    let mut cases = 0usize;
    let mut diverged = Vec::new();
    datadriven::walk(CASE_DIR, |file| {
        file.run(|case| {
            assert_eq!(
                case.directive, "run",
                "unknown directive {:?}",
                case.directive
            );
            cases += 1;
            let args = words(&case.input);
            let theirs = run(&oracle, &args, &theirs_dir, case, true);
            let crate_side = run(ours, &args, &ours_dir, case, false);
            if crate_side != theirs {
                diverged.push(format!(
                    "{}:\n--- oracle\n{theirs}--- crate\n{crate_side}",
                    case.input.replace('\n', " ")
                ));
            }
            theirs
        });
    });
    remove(&theirs_dir);
    remove(&ours_dir);
    assert!(cases > 0, "{CASE_DIR} produced no cases");
    assert!(diverged.is_empty(), "{}", diverged.join("\n"));
}

// Deviation 141: the oracle drops one leading blank of the argument string
// of a `-e` program, so the harness above cannot hold the case; this one
// runs interrexx alone and compares `-e` with a program file.
// spec: env.invocation.deviation.141
#[test]
fn a_dash_e_program_gets_its_argument_as_a_file_does() {
    let ours = Path::new(env!("CARGO_BIN_EXE_rexx"));
    let dir = fixture("dash-e");
    let text = "say '[' || arg(1) || ']' arg()";
    std::fs::write(dir.join("echo.rex"), format!("{text}\n")).expect("the program");
    let stdout = |args: &[&str]| {
        let output = Command::new("timeout")
            .args(["-k", "5", "20"])
            .arg(ours)
            .args(args)
            .current_dir(&dir)
            .env_remove("REXX_PATH")
            .stdin(Stdio::null())
            .output()
            .expect("the binary starts");
        String::from_utf8(output.stdout).expect("text")
    };
    for words in [
        &[" ", "x"][..],
        &["", "x"],
        &["  ", "x"],
        &["a", "", "b"],
        &[],
    ] {
        let mut file = vec!["echo.rex"];
        file.extend_from_slice(words);
        let mut string = vec!["-e", text];
        string.extend_from_slice(words);
        assert_eq!(stdout(&string), stdout(&file), "words {words:?}");
    }
    assert_eq!(stdout(&["-e", text, " ", "x"]), "[  x] 1\n");
    remove(&dir);
}
