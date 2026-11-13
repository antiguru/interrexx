//! A boolean a native extension reads is the handle of `.true` or `.false`
//! (#599, ruling D15): `True()`/`False()` equal an argument's handle and the
//! answer of any message that answers a boolean. The extension returns one
//! line per case, which the program says.

#![cfg(target_os = "linux")]

mod support;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use interrexx::{Invocation, run_program};

const GATE_ENV: &str = "REXX_CORPUS_GATE";

const EXPECTED: &str = "argument_true T\nargument_false F\nhasentry_missing F\n\
hasentry_present T\ncompare T\nisa T\nrexx_method T\n";

fn gate_mode() -> bool {
    env::var(GATE_ENV).is_ok_and(|value| !value.is_empty() && value != "0")
}

fn probes() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/boolean_handles")
}

fn build_library(source: &Path, library: &Path) {
    let api = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../third_party/ooRexx/api");
    let built = Command::new("c++")
        .args(["-shared", "-fPIC", "-O1", "-static-libstdc++"])
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(source)
        .arg("-o")
        .arg(library)
        .status()
        .expect("c++ runs");
    assert!(built.success(), "the extension did not build");
}

#[test]
fn a_boolean_handle_is_true_or_false_as_on_the_oracle() {
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("boolean-handles-{}", std::process::id()));
    let (forge, run) = (base.join("forge"), base.join("run"));
    fs::create_dir_all(&forge).expect("the extension directory");
    fs::create_dir_all(&run).expect("the run directory");
    build_library(
        &probes().join("booleans.cpp"),
        &forge.join("libbooleans.so"),
    );

    let program = probes().join("booleans.rex");
    let mut environment: Vec<(Vec<u8>, Vec<u8>)> = env::vars()
        .filter(|(name, _)| name != "LD_LIBRARY_PATH")
        .map(|(name, value)| (name.into_bytes(), value.into_bytes()))
        .collect();
    let oracle = gate_mode().then(support::oracle::locate);
    let library_path = match &oracle {
        Some(oracle) => format!("{}:{}", oracle.lib_dir().display(), forge.display()),
        None => forge.display().to_string(),
    };
    environment.push((
        b"LD_LIBRARY_PATH".to_vec(),
        library_path.clone().into_bytes(),
    ));
    let invocation = Invocation::none()
        .with_directory(run.clone())
        .with_environment(environment)
        .with_deadline(support::oracle::RUN_DEADLINE);
    if let Some(oracle) = oracle {
        let theirs = oracle.run_in(&program, &run, &[("LD_LIBRARY_PATH", &library_path)]);
        let status = theirs.expect_exit_code();
        assert_eq!(
            (String::from_utf8_lossy(&theirs.stdout).as_ref(), status),
            (EXPECTED, 0),
            "oracle stderr: {}",
            String::from_utf8_lossy(&theirs.stderr)
        );
    }
    let text = fs::read(&program).expect("the program");
    let ours = run_program(&program.to_string_lossy(), text, invocation);
    let stdout = String::from_utf8_lossy(&ours.stdout).into_owned();
    assert_eq!(
        (stdout.as_str(), ours.exit_code),
        (EXPECTED, 0),
        "stderr: {}",
        String::from_utf8_lossy(&ours.stderr)
    );

    fs::remove_dir_all(&base).expect("cannot remove the run directory");
}
