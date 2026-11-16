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
fn a_seed_that_is_not_decimal_digits_below_2_64_is_refused() {
    for value in ["x1", "+5", "-5", " 5", "18446744073709551616"] {
        let output = run(CODE, &[("REXX_HASH_SEED", value)]);
        assert_eq!(output.status.code(), Some(2), "{value}");
        assert!(output.stdout.is_empty(), "{value}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("REXX_HASH_SEED"), "{value}: {stderr}");
    }
    assert_eq!(
        stdout(CODE, &[("REXX_HASH_SEED", "18446744073709551615")]),
        stdout(CODE, &[("REXX_HASH_SEED", "18446744073709551615")])
    );
}

/// An empty value is the same as none: the seed is drawn for each run.
#[test]
fn an_empty_seed_is_unset() {
    assert_ne!(
        stdout(CODE, &[("REXX_HASH_SEED", "")]),
        stdout(CODE, &[("REXX_HASH_SEED", "")])
    );
}

/// An embedded run cannot be refused, so a value the `rexx` command refuses
/// runs with a drawn seed and a warning on the run's standard error.
#[test]
fn an_embedded_run_warns_of_a_refused_seed_and_draws_one() {
    let run = |value: &str| {
        interrexx::run_program(
            "/t.rex",
            CODE.as_bytes().to_vec(),
            interrexx::Invocation::none().with_environment(vec![(
                b"REXX_HASH_SEED".to_vec(),
                value.as_bytes().to_vec(),
            )]),
        )
    };
    let (first, second) = (run("+5"), run("+5"));
    for outcome in [&first, &second] {
        assert_eq!(outcome.exit_code, 0);
        let stderr = String::from_utf8_lossy(&outcome.stderr);
        assert!(
            stderr.contains("REXX_HASH_SEED") && stderr.contains("`+5`"),
            "{stderr}"
        );
    }
    assert_ne!(first.stdout, second.stdout);
    assert!(run("5").stderr.is_empty());
}

/// A seed `REXX_HASH_SEED` names in the simulation mode is recorded in the
/// trace's header and in the run's `rexx-sim:` line, so a replay of the
/// trace hashes under it with the variable unset.
#[test]
fn a_named_seed_is_recorded_for_a_replay() {
    let dir = std::env::temp_dir().join(format!("interrexx-hash-seed-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("scratch directory");
    let trace = dir.join("trace");
    let mode = format!("sim:7,trace={}", trace.display());
    let recorded = run(
        CODE,
        &[("REXX_SWITCH_MODE", &mode), ("REXX_HASH_SEED", "1")],
    );
    assert_eq!(recorded.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&recorded.stderr);
    assert!(stderr.contains(" hash=1 "), "{stderr}");
    let header = std::fs::read_to_string(&trace).expect("the trace was written");
    assert!(
        header
            .lines()
            .next()
            .unwrap_or_default()
            .contains(",hash=1 "),
        "{header}"
    );
    let replay = format!("sim:replay={}", trace.display());
    let replayed = stdout(CODE, &[("REXX_SWITCH_MODE", &replay)]);
    assert_eq!(replayed, String::from_utf8_lossy(&recorded.stdout));
    assert_ne!(replayed, stdout(CODE, &[("REXX_SWITCH_MODE", "sim:7")]));
    let _ = std::fs::remove_dir_all(&dir);
}
