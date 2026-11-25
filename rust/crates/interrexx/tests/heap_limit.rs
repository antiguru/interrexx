//! The heap limit (`r[env.invocation.heap-limit]`): what `REXX_HEAP_LIMIT`
//! and `Invocation::with_heap_limit` accept, and what a run under one raises.
//! The runs are each a `rexx` process of its own, so that a request the limit
//! fails to refuse is that process's memory, not this harness's.

use std::process::{Command, Stdio};

/// What `rexx -e program` writes and its status, with `REXX_HEAP_LIMIT` set to
/// `limit` or, for `None`, unset.
fn run(program: &str, limit: Option<&str>) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bounded-run"));
    command
        .args(["-k", "5", "120"])
        .arg(env!("CARGO_BIN_EXE_rexx"))
        .arg("-e")
        .arg(program)
        .env_remove("REXX_HEAP_LIMIT")
        .env_remove("REXX_SWITCH_MODE")
        .stdin(Stdio::null());
    if let Some(limit) = limit {
        command.env("REXX_HEAP_LIMIT", limit);
    }
    let output = command.output().expect("rexx runs");
    format!(
        "{}rc={}\n{}",
        String::from_utf8_lossy(&output.stdout),
        output
            .status
            .code()
            .map_or("signal".into(), |code| code.to_string()),
        String::from_utf8_lossy(&output.stderr)
    )
}

// spec: env.invocation.heap-limit, env.invocation.heap-limit.raise
/// The cases under `tests/heap_limit/`: `run limit=SIZE` runs its input as a
/// program with `REXX_HEAP_LIMIT` set to `SIZE`, and `parse` reads its input
/// as a `REXX_HEAP_LIMIT` value. `REWRITE=1` re-records them.
#[test]
fn heap_limit_cases() {
    let mut cases = 0;
    datadriven::walk("tests/heap_limit", |file| {
        file.run(|case| {
            cases += 1;
            match case.directive.as_str() {
                "run" => {
                    let limit: String = case.take_arg("limit").expect("a limit");
                    run(case.input.trim_end(), Some(&limit))
                }
                "parse" => match interrexx::parse_heap_limit(case.input.trim_end().as_bytes()) {
                    Ok(Some(bytes)) => format!("{bytes}\n"),
                    Ok(None) => "unbounded\n".into(),
                    Err(message) => format!("error: {message}\n"),
                },
                other => panic!("unknown directive {other}"),
            }
        });
    });
    assert!(cases > 0, "no case ran");
}

/// A request that a limit refuses outright.
const REFUSED: &str = "a = .Array~new(10000000); say a~size";

/// An empty `REXX_HEAP_LIMIT` is no limit.
#[test]
fn an_empty_value_leaves_the_heap_unbounded() {
    assert_eq!(run(REFUSED, Some("")), "10000000\nrc=0\n");
}

/// `rexx` refuses a value it cannot read before running anything.
#[test]
fn the_command_refuses_an_unreadable_value() {
    let out = run("say 'ran'", Some("12Q"));
    assert!(out.starts_with("rc=2\nrexx: REXX_HEAP_LIMIT"), "{out}");
}

/// An embedded run reads `REXX_HEAP_LIMIT` from the environment it is given,
/// warns of a value it cannot read and runs unbounded, and an invocation's
/// own limit takes the place of the variable's.
#[test]
fn an_embedded_run_reads_the_variable_and_the_invocation_overrides_it() {
    let embedded = |environment: &str, limit: Option<usize>| {
        let mut invocation = interrexx::Invocation::none().with_environment(vec![(
            b"REXX_HEAP_LIMIT".to_vec(),
            environment.as_bytes().to_vec(),
        )]);
        if let Some(limit) = limit {
            invocation = invocation.with_heap_limit(limit);
        }
        let outcome = interrexx::run_program(
            "heap.rex",
            format!("signal on syntax\n{REFUSED}\nexit\nsyntax: say condition('o')~code\n")
                .into_bytes(),
            invocation,
        );
        (
            String::from_utf8_lossy(&outcome.stdout).into_owned(),
            String::from_utf8_lossy(&outcome.stderr).into_owned(),
        )
    };
    assert_eq!(embedded("64M", None), ("5.0\n".into(), String::new()));
    assert_eq!(
        embedded("1G", Some(64 << 20)),
        ("5.0\n".into(), String::new())
    );
    assert_eq!(embedded("", Some(64 << 20)).0, "5.0\n");
    let (stdout, stderr) = embedded("12Q", None);
    assert_eq!(stdout, "10000000\n");
    assert!(
        stderr.starts_with("interrexx: REXX_HEAP_LIMIT") && stderr.ends_with("unbounded\n"),
        "{stderr}"
    );
}

/// Growth in small steps is raised before the heap passes the limit by more
/// than a sixteenth of it and a step: the collection falls due at the limit,
/// not at the doubling of the live heap that paces a run without one.
#[test]
fn small_steps_are_raised_close_to_the_limit() {
    const LIMIT: usize = 64 << 20;
    const STEP: usize = 1 << 20;
    let outcome = interrexx::run_program(
        "steps.rex",
        format!(
            "signal on syntax\n\
             a = .Array~new\n\
             do i = 1 to 1000\n\
             \x20 a[i] = copies('s', {STEP})\n\
             end\n\
             exit\n\
             syntax: say condition('o')~code a~items\n"
        )
        .into_bytes(),
        interrexx::Invocation::none().with_heap_limit(LIMIT),
    );
    let stdout = String::from_utf8_lossy(&outcome.stdout);
    let items: usize = stdout
        .strip_prefix("5.0 ")
        .and_then(|rest| rest.trim_end().parse().ok())
        .unwrap_or_else(|| panic!("{stdout}"));
    assert!(
        items * STEP <= LIMIT + LIMIT / 16 + 2 * STEP,
        "raised at {items} steps of {STEP} bytes"
    );
}
