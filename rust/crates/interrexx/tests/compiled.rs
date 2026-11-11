//! Compiled programs, restored by every route the oracle restores one.
//!
//! Each side's `rexxc` compiles `tests/compiled/programs` into that side's
//! own directory, and each side's `rexx` runs the cases of
//! `tests/compiled/cases` there. The three descriptors must agree, and the
//! recorded block is the oracle's. A case's input is its words, one per
//! line. Images this crate refuses are pinned against this crate alone: the
//! oracle crashes on a truncated image.

#[path = "support/oracle_runtime.rs"]
mod oracle_runtime;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const CASE_DIR: &str = "tests/compiled/cases";

const PROGRAMS: &str = "tests/compiled/programs";

/// An image the oracle's `rexxc` wrote.
const ORACLE_IMAGE: &str = "tests/compiled/oracle.rxo";

/// The oracle's address-space cap, in KiB.
const ORACLE_MEMORY_LIMIT_KIB: u64 = 1_048_576;

const ORACLE_ROOT: &str = env!("REXX_ORACLE_ROOT");

/// What every image starts with, on either side.
const IMAGE_PREFIX: &[u8] = b"#!/usr/bin/env rexx\n/**/@REXX";

/// One side: its two binaries, and whether they are the oracle's.
struct Side {
    rexx: PathBuf,
    rexxc: PathBuf,
    oracle: bool,
    dir: PathBuf,
}

impl Side {
    fn oracle() -> Side {
        let bin = Path::new(ORACLE_ROOT).join("bin");
        assert!(
            bin.join("rexx").is_file(),
            "the oracle is missing at {}",
            bin.display()
        );
        Side {
            rexx: bin.join("rexx"),
            rexxc: bin.join("rexxc"),
            oracle: true,
            dir: fixture("oracle"),
        }
    }

    /// This crate's side, in a directory of its own named `name`.
    fn ours(name: &str) -> Side {
        Side {
            rexx: PathBuf::from(env!("CARGO_BIN_EXE_rexx")),
            rexxc: PathBuf::from(env!("CARGO_BIN_EXE_rexxc")),
            oracle: false,
            dir: fixture(name),
        }
    }

    /// A command running `program` from this side's directory under
    /// `timeout`, the oracle's under its address-space cap.
    fn command(&self, program: &Path) -> Command {
        let mut command = Command::new("timeout");
        command.args(["-k", "5", "20"]);
        if self.oracle {
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
            .current_dir(&self.dir)
            .env_remove("REXX_PATH")
            .stdin(Stdio::null());
        command
    }

    /// Runs one case on this side, answering its descriptors as the case
    /// file records them.
    fn run(&self, case: &datadriven::TestCase) -> String {
        let words: Vec<&str> = case.input.lines().collect();
        let program = match case.directive.as_str() {
            "compile" | "rexxc" => &self.rexxc,
            "rexx" => &self.rexx,
            other => panic!("unknown directive {other:?}"),
        };
        let mut command = self.command(program);
        command.args(&words);
        for (key, values) in &case.args {
            assert_eq!(key, "stdin", "unknown argument {key}");
            let file = std::fs::File::open(self.dir.join(&values[0])).expect("the input file");
            command.stdin(file);
        }
        let output = command.output().expect("the binary starts");
        let rendered = render(
            &self.dir,
            &output.stdout,
            &output.stderr,
            output.status.code(),
        );
        if case.directive == "compile" {
            assert_eq!(output.status.code(), Some(0), "{words:?}: {rendered}");
            let image = std::fs::read(self.dir.join(words[1])).expect("the image");
            assert!(
                image.starts_with(IMAGE_PREFIX),
                "{words:?} on {}",
                self.rexxc.display()
            );
        }
        rendered
    }
}

/// A fresh directory below the target's temporary directory.
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("compiled-{name}-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("an old scratch directory");
    }
    std::fs::create_dir_all(&dir).expect("the scratch directory");
    dir
}

/// A fresh directory holding the fixture's programs.
fn fixture(side: &str) -> PathBuf {
    let dir = scratch(side);
    copy_tree(Path::new(PROGRAMS), &dir);
    dir
}

fn copy_tree(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).expect("the fixture directory") {
        let entry = entry.expect("a fixture entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("its type").is_dir() {
            std::fs::create_dir_all(&target).expect("a fixture directory");
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("a fixture file");
        }
    }
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

/// The three descriptors as the case file records them, with the side's
/// directory as `DIR`.
fn render(dir: &Path, stdout: &[u8], stderr: &[u8], status: Option<i32>) -> String {
    let dir = escaped(dir.as_os_str().as_encoded_bytes());
    let mut rendered = match status {
        Some(code) => format!("rc> {code}\n"),
        None => "rc> none\n".to_string(),
    };
    for (tag, bytes) in [("out", stdout), ("err", stderr)] {
        for line in escaped(bytes).replace(&dir, "DIR").lines() {
            rendered.push_str(&format!("{tag}> {line}\n"));
        }
    }
    rendered
}

// The cases of `compiled/cases/restore` run images by each route the
// invocation page's section "Programs without source" names.
// spec: env.invocation.rexxc.run, env.invocation.rexxc.source
// spec: env.invocation.rexxc.trace, env.invocation.rexxc.options
// spec: env.invocation.rexxc.routes
#[test]
fn each_route_runs_a_restored_program_as_the_oracles_does() {
    let (theirs, ours) = (Side::oracle(), Side::ours("crate"));
    let mut cases = 0usize;
    let mut diverged = Vec::new();
    datadriven::walk(CASE_DIR, |file| {
        file.run(|case| {
            cases += 1;
            let oracle = theirs.run(case);
            let crate_side = ours.run(case);
            if crate_side != oracle {
                diverged.push(format!(
                    "{} {}:\n--- oracle\n{oracle}--- crate\n{crate_side}",
                    case.directive,
                    case.input.replace('\n', " ")
                ));
            }
            oracle
        });
    });
    std::fs::remove_dir_all(&theirs.dir).expect("the oracle directory is removed");
    std::fs::remove_dir_all(&ours.dir).expect("the crate directory is removed");
    assert!(cases > 0, "{CASE_DIR} produced no cases");
    assert!(diverged.is_empty(), "{}", diverged.join("\n"));
}

/// The report this crate's `rexx` makes for `image` in `dir`.
fn ours_on(side: &Side, image: &str, bytes: &[u8]) -> String {
    std::fs::write(side.dir.join(image), bytes).expect("the image");
    let output = side
        .command(&side.rexx)
        .arg(image)
        .output()
        .expect("rexx starts");
    render(
        &side.dir,
        &output.stdout,
        &output.stderr,
        output.status.code(),
    )
}

/// The report for an image refused with `sub`: 3.903 or 3.905.
fn refused(image: &str, sub: u16) -> String {
    let message = match sub {
        903 => format!(
            "Error 3.903:  Program \"DIR/{image}\" cannot be run by this version of the REXX \
             interpreter."
        ),
        _ => format!("Error 3.905:  Encoded compiled program \"DIR/{image}\" cannot be decoded."),
    };
    format!("rc> 253\nerr> Error 3:  Failure during initialization.\nerr> {message}\n")
}

// spec: env.invocation.rexxc.foreign, env.invocation.rexxc.damaged
// spec: env.invocation.deviation.41
/// Review Focus 2: an image this crate does not write, or a damaged one,
/// never panics. Undecodable base64 is 3.905 and every other damage 3.903,
/// rc 253, each measured on the oracle where the oracle survives the image.
#[test]
fn a_foreign_or_damaged_image_is_refused() {
    let side = Side::ours("damaged");
    let oracle_image = std::fs::read(ORACLE_IMAGE).expect("the oracle's image");
    assert!(oracle_image.starts_with(IMAGE_PREFIX));
    assert_eq!(
        ours_on(&side, "oracle.rxo", &oracle_image),
        refused("oracle.rxo", 903)
    );

    let compile = |output: &str, flags: &[&str]| -> Vec<u8> {
        let written = side
            .command(&side.rexxc)
            .args(["fail.rex", output, "-s"])
            .args(flags)
            .output()
            .expect("rexxc starts");
        assert_eq!(written.status.code(), Some(0), "{output}");
        std::fs::read(side.dir.join(output)).expect("the image")
    };
    let binary = compile("b.rxo", &[]);
    let encoded = compile("e.rxo", &["-e"]);
    // The `#!` line, then the 48-byte header, then the source.
    let header = 20 + 48;
    let body = 20 + "/**/@REXX@\n".len();

    let mut damaged: Vec<(&str, Vec<u8>, u16)> = vec![
        ("header.rxo", binary[..header - 8].to_vec(), 903),
        ("source.rxo", binary[..binary.len() - 3].to_vec(), 903),
        // Line ends between quartets: a whole line cut away.
        ("line.rxo", encoded[..body + 73].to_vec(), 903),
        // Inside a line, on a quartet boundary.
        ("boundary.rxo", encoded[..body + 8].to_vec(), 903),
        // Inside a quartet.
        ("quartet.rxo", encoded[..body + 6].to_vec(), 903),
    ];
    let mut broken = encoded.clone();
    broken[body + 5] = b'!';
    damaged.push(("broken.rxo", broken, 905));
    // Deviation 41: the encoded tag over base64 of bytes without the binary
    // tag. The oracle runs whatever it decoded in place.
    let mut untagged = b"#!/usr/bin/env rexx\n/**/@REXX@\n".to_vec();
    untagged.extend_from_slice(b"c2F5ICdkZWNvZGVkJwo=\n");
    damaged.push(("untagged.rxo", untagged, 903));
    for (image, bytes, sub) in damaged {
        assert_eq!(
            ours_on(&side, image, &bytes),
            refused(image, sub),
            "{image}"
        );
    }
    std::fs::remove_dir_all(&side.dir).expect("the directory is removed");
}

/// What `rexx` writes for the file `name` that holds `bytes`, run in `side`'s
/// directory with the argument `x`.
fn report(side: &Side, name: &str, bytes: &[u8]) -> String {
    std::fs::write(side.dir.join(name), bytes).expect("the file");
    let output = side
        .command(&side.rexx)
        .args([name, "x"])
        .output()
        .expect("rexx starts");
    render(
        &side.dir,
        &output.stdout,
        &output.stderr,
        output.status.code(),
    )
}

/// The image `side`'s `rexxc` writes for `hello.rex`, with the flags.
fn hello(side: &Side, flags: &[&str]) -> Vec<u8> {
    let written = side
        .command(&side.rexxc)
        .args(["hello.rex", "hello.out", "-s"])
        .args(flags)
        .output()
        .expect("rexxc starts");
    assert_eq!(written.status.code(), Some(0), "{flags:?}");
    let bytes = std::fs::read(side.dir.join("hello.out")).expect("the image");
    std::fs::remove_file(side.dir.join("hello.out")).expect("the image is removed");
    bytes
}

/// The value of a base64 digit.
fn digit(byte: u8) -> u32 {
    u32::from(match byte {
        b'A'..=b'Z' => byte - b'A',
        b'a'..=b'z' => byte - b'a' + 26,
        b'0'..=b'9' => byte - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        other => panic!("{other} is no base64 digit"),
    })
}

/// The bytes of whole base64 quartets.
fn decode(text: &[u8]) -> Vec<u8> {
    text.chunks(4)
        .flat_map(|quartet| {
            let value = quartet.iter().fold(0u32, |all, &byte| all << 6 | digit(byte));
            [(value >> 16) as u8, (value >> 8) as u8, value as u8]
        })
        .collect()
}

/// An image that both interpreters write the same way at its opening, and
/// its encoded form as lines of base64 text.
// spec: env.invocation.rexxc.image
#[test]
fn both_interpreters_write_an_image_with_the_same_opening() {
    for side in [Side::oracle(), Side::ours("opening")] {
        let binary = hello(&side, &[]);
        let opening = b"#!/usr/bin/env rexx\n/**/@REXX\0\0";
        assert!(binary.starts_with(opening), "{}", side.rexx.display());
        assert!(binary.contains(&0), "{}", side.rexx.display());
        let encoded = hello(&side, &["-e"]);
        let tag = b"#!/usr/bin/env rexx\n/**/@REXX@\n";
        assert!(encoded.starts_with(tag), "{}", side.rexx.display());
        let text = &encoded[tag.len()..];
        assert_eq!(text.last(), Some(&b'\n'));
        let lines: Vec<&[u8]> = text[..text.len() - 1].split(|&byte| byte == b'\n').collect();
        let (last, whole) = lines.split_last().expect("lines");
        assert!(whole.iter().all(|line| line.len() == 72));
        assert!(!last.is_empty() && last.len() <= 72);
        let decoded = decode(&lines[0][..16]);
        assert!(
            decoded.starts_with(b"/**/@REXX\0\0"),
            "{}: {decoded:?}",
            side.rexx.display()
        );
        std::fs::remove_dir_all(&side.dir).expect("the directory is removed");
    }
}

/// An image runs by its bytes alone: without its first line, with another
/// first line that starts `#!`, under any name; and the search adds no `.rxo`.
// spec: env.invocation.rexxc.run
#[test]
fn an_image_runs_by_its_opening_and_not_by_its_name() {
    for side in [Side::oracle(), Side::ours("byname")] {
        let ran = "rc> 4\nout> hello x\n";
        for flags in [&[][..], &["-e"][..]] {
            let image = hello(&side, flags);
            let rest = &image[image.iter().position(|&b| b == b'\n').expect("a line") + 1..];
            let other = [&b"#!x\n"[..], rest].concat();
            let variants = [
                ("a.rxo", image.clone()),
                ("plain", image.clone()),
                ("a.rex", image.clone()),
                ("no-line.rxo", rest.to_vec()),
                ("other-line.rxo", other),
            ];
            for (name, bytes) in variants {
                assert_eq!(report(&side, name, &bytes), ran, "{name} {flags:?}");
            }
        }
        let missing = report(&side, "solo2.rxo", &hello(&side, &[]));
        assert_eq!(missing, ran);
        let output = side
            .command(&side.rexx)
            .arg("solo2")
            .output()
            .expect("rexx starts");
        assert_eq!(output.status.code(), Some(253));
        std::fs::remove_dir_all(&side.dir).expect("the directory is removed");
    }
}

/// A file that begins like an image, with a changed letter of `REXX` or a
/// changed byte of the tag, is source and stops at its `@`.
// spec: env.invocation.rexxc.run
#[test]
fn an_opening_with_a_changed_byte_is_source() {
    let stop = "rc> 243\n\
                err>      2 *-* /**/@\n\
                err> Error 13 running DIR/t.rxo line 2:  Invalid character in program.\n\
                err> Error 13.1:  Incorrect character in program \"@\" ('40'X).\n";
    for side in [Side::oracle(), Side::ours("changed")] {
        for (flags, offsets) in [(&[][..], 25..=29), (&["-e"][..], 25..=30)] {
            let image = hello(&side, flags);
            for offset in offsets {
                let mut bytes = image.clone();
                bytes[offset] ^= 1;
                assert_eq!(report(&side, "t.rxo", &bytes), stop, "{offset} {flags:?}");
            }
        }
        std::fs::remove_dir_all(&side.dir).expect("the directory is removed");
    }
}

/// An image of either interpreter is refused by the other with 3.903, and an
/// image whose identification has a changed byte is refused by its own;
/// a line of an encoded image that is not base64 is refused with 3.905.
// spec: env.invocation.rexxc.foreign, env.invocation.rexxc.damaged
#[test]
fn both_interpreters_refuse_a_foreign_image_and_an_undecodable_line() {
    let (theirs, ours) = (Side::oracle(), Side::ours("foreign"));
    let (their_image, our_image) = (hello(&theirs, &[]), hello(&ours, &[]));
    assert_eq!(
        report(&theirs, "x.rxo", &our_image),
        refused("x.rxo", 903),
        "the oracle on ours"
    );
    assert_eq!(
        report(&ours, "x.rxo", &their_image),
        refused("x.rxo", 903),
        "ours on the oracle's"
    );
    // The header's identification: each byte of it that is changed gives the
    // refusal on both sides (measured at these offsets).
    for (side, image) in [(&theirs, &their_image), (&ours, &our_image)] {
        for offset in [36, 40, 44, 47] {
            let mut bytes = image.clone();
            bytes[offset] ^= 1;
            assert_eq!(
                report(side, "x.rxo", &bytes),
                refused("x.rxo", 903),
                "{offset} on {}",
                side.rexx.display()
            );
        }
    }
    let body = b"#!/usr/bin/env rexx\n/**/@REXX@\n".len();
    for side in [&theirs, &ours] {
        let encoded = hello(side, &["-e"]);
        for (offset, byte) in [(body + 5, b'!'), (body + 5, b' '), (body + 73 + 9, b'!')] {
            let mut bytes = encoded.clone();
            bytes[offset] = byte;
            assert_eq!(
                report(side, "x.rxo", &bytes),
                refused("x.rxo", 905),
                "{offset} {byte} on {}",
                side.rexx.display()
            );
        }
    }
    std::fs::remove_dir_all(&theirs.dir).expect("the oracle directory is removed");
    std::fs::remove_dir_all(&ours.dir).expect("the crate directory is removed");
}
