//! The seed of the keyed string hash (`r[class.string.hashcode]`): drawn per
//! run, fixed by `REXX_HASH_SEED`, and in the simulation mode by the
//! simulation seed. Each run is its own process, since the seed is drawn once
//! per interpreter.

use std::process::{Command, Output, Stdio};

/// Runs `program` in `rexx` with only the hash-seed settings `env` names.
fn run(program: &str, env: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rexx"));
    command
        .arg("-e")
        .arg(program)
        .env_remove("REXX_HASH_SEED")
        .env_remove("REXX_SWITCH_MODE")
        .stdin(Stdio::null());
    for (name, value) in env {
        command.env(name, value);
    }
    command.output().expect("rexx runs")
}

/// What `program` wrote, after checking that it ran to a clean end.
fn stdout(program: &str, env: &[(&str, &str)]) -> String {
    let output = run(program, env);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

const CODE: &str = "say c2x('abc'~hashCode)";

#[test]
fn the_string_hash_code_differs_between_seeds() {
    let one = stdout(CODE, &[("REXX_HASH_SEED", "1")]);
    let two = stdout(CODE, &[("REXX_HASH_SEED", "2")]);
    assert_ne!(one, two);
}

#[test]
fn the_string_hash_code_is_stable_within_a_run() {
    assert_eq!(
        stdout("say 'abc'~hashCode == ('ab' || 'c')~hashCode", &[]),
        "1\n"
    );
    assert_eq!(
        stdout(CODE, &[("REXX_HASH_SEED", "1")]),
        stdout(CODE, &[("REXX_HASH_SEED", "1")])
    );
}

#[test]
fn an_unset_seed_is_drawn_for_each_run() {
    assert_ne!(stdout(CODE, &[]), stdout(CODE, &[]));
}

#[test]
fn a_simulation_seed_fixes_the_hash_seed() {
    let first = stdout(CODE, &[("REXX_SWITCH_MODE", "sim:7")]);
    assert_eq!(first, stdout(CODE, &[("REXX_SWITCH_MODE", "sim:7")]));
    assert_ne!(first, stdout(CODE, &[("REXX_SWITCH_MODE", "sim:8")]));
}

#[test]
fn a_named_seed_wins_over_the_simulation_seed() {
    assert_eq!(
        stdout(
            CODE,
            &[("REXX_SWITCH_MODE", "sim:7"), ("REXX_HASH_SEED", "1")]
        ),
        stdout(CODE, &[("REXX_HASH_SEED", "1")])
    );
}

#[test]
fn a_seed_that_is_not_a_number_is_refused() {
    let output = run(CODE, &[("REXX_HASH_SEED", "x1")]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}
