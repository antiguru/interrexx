//! `PARSE VERSION`'s string, checked against the running oracle rather than
//! against a copy of itself.

#![expect(
    clippy::disallowed_methods,
    reason = "this harness times or bounds real runs"
)]

mod support;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use support::oracle::{descriptor_diffs, did_not_finish};

/// Env var that flips this file from a skip into the check. `corpus.rs`'s own
/// switch, deliberately reused rather than given a fourth spelling.
const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    match std::env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// A directory of this run's own, because an unresolved Rexx name searches the
/// current directory for an external routine -- `support::oracle::Oracle::run`'s
/// own doc comment has the measured consequence of running a synthesised
/// program from a directory of stale `.rex` files.
fn fresh_run_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("parse-version-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    dir
}

/// `stdout` with the build date, the last three words of the line, replaced by
/// `<date>`, and the language level before it, a word `n.nn`, by `<level>`.
/// The two sides answer different levels by decision (Deviation 179;
/// `docs/design/decisions.md`, the language level is the one interrexx
/// implements), so the level is not compared; every other word is.
fn mask_date_and_level(stdout: &mut Vec<u8>) {
    let text = String::from_utf8_lossy(stdout).into_owned();
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
    let masked: Vec<String> = text
        .lines()
        .map(|line| {
            let mut words: Vec<&str> = line.split(' ').collect();
            let Some(keep) = words.len().checked_sub(3) else {
                return line.to_string();
            };
            words.truncate(keep);
            if let Some(level) = words.last_mut()
                && level
                    .split_once('.')
                    .is_some_and(|(major, minor)| digits(major) && digits(minor))
            {
                *level = "<level>";
            }
            format!("{} <date>", words.join(" "))
        })
        .collect();
    *stdout = format!("{}\n", masked.join("\n")).into_bytes();
}

/// The level and the date are the only words masked: two levels mask alike,
/// and a change to any other word still differs.
#[test]
fn only_the_language_level_and_the_build_date_are_masked() {
    const OURS: &str = "REXX-ooRexx_5.3.0(MT)_64-bit 6.06 14 Nov 2026";
    let masked = |line: &str| {
        let mut bytes = line.as_bytes().to_vec();
        mask_date_and_level(&mut bytes);
        String::from_utf8(bytes).expect("the mask keeps UTF-8")
    };
    assert_eq!(
        masked(OURS),
        "REXX-ooRexx_5.3.0(MT)_64-bit <level> <date>\n"
    );
    assert_eq!(
        masked(&OURS.replace("6.06 14 Nov", "6.07 30 Jul")),
        masked(OURS)
    );
    for (from, to) in [
        ("REXX-ooRexx", "REXX-ooRexy"),
        ("5.3.0", "5.3.1"),
        ("(MT)", "(ST)"),
        ("64-bit", "32-bit"),
        ("6.06", "6.06 extra"),
        ("6.06", "6.x"),
    ] {
        assert_ne!(
            masked(&OURS.replacen(from, to, 1)),
            masked(OURS),
            "{from} -> {to}"
        );
    }
}

/// `PARSE VERSION` answers what the oracle answers, or this crate's recorded
/// constant has gone stale against the built interpreter.
#[test]
fn parse_version_still_answers_what_the_oracle_answers() {
    if !gate_mode() {
        eprintln!(
            "*** SKIPPED -- PARSE VERSION's string is not compared against the \
             oracle unless {GATE_ENV} is set. Nothing else in the tree guards \
             it, so a green run here is not evidence the constant is current. ***"
        );
        return;
    }

    let dir = fresh_run_dir();
    let file = dir.join("parse_version.rex");
    fs::write(&file, b"parse version v\nsay v\n")
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    let abs = fs::canonicalize(&file)
        .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", file.display()));
    let path = abs
        .to_str()
        .unwrap_or_else(|| panic!("probe path {} is not valid UTF-8", abs.display()));

    let oracle = support::oracle::locate();
    let text = fs::read(&abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
    let rust = interrexx::run_program(
        path,
        text,
        interrexx::Invocation::none().with_deadline(support::oracle::RUN_DEADLINE),
    );
    let cpp = oracle.run(&abs);

    // Named even though this file runs exactly one program: `descriptor_diffs`
    // would otherwise reach `expect_exit_code`'s path-less panic, and an
    // unconditional `assert!` naming the program is the shape this crate's
    // other oracle-invoking harnesses use for the same event.
    assert!(
        !did_not_finish(&cpp),
        "PARSE VERSION's oracle run did not finish: {:?} -- a structural \
         failure, not a byte comparison",
        cpp.termination
    );

    // The build date is each binary's own and the level is licensed; the rest
    // of the string must agree.
    let (mut rust, mut cpp) = (rust, cpp);
    mask_date_and_level(&mut rust.stdout);
    mask_date_and_level(&mut cpp.stdout);
    let diffs = descriptor_diffs(&rust, &cpp);
    assert!(
        diffs.is_empty(),
        "PARSE VERSION disagrees with the oracle on [{}]. This crate's \
         version.rs VERSION constant is a recorded measurement of the \
         oracle's own build identity, its name, and a rebuilt oracle \
         changes it; the language level is interrexx's own (Deviation 179). Re-measure rather than editing this \
         test:\n  rust:   {:?}\n  oracle: {:?}\n  exit: rust {} oracle {}",
        diffs.join(", "),
        String::from_utf8_lossy(&rust.stdout),
        String::from_utf8_lossy(&cpp.stdout),
        rust.exit_code,
        cpp.expect_exit_code()
    );
    assert_eq!(
        oracle.invocations(),
        1,
        "the comparison above must have actually started the oracle; a check \
         that compares this crate's answer against nothing is the shape this \
         file exists to replace"
    );

    // Only on success: a failing run's file is the evidence.
    let _ = fs::remove_dir_all(&dir);
}
