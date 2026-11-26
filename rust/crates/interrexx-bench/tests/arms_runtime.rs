//! The real rexx-arms command with a script in place of perf and each interpreter.

#![cfg(target_os = "linux")]

use interrexx_oracle::private_service::{PrivateService, Side};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

fn script(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn separate_builds_keep_their_services_only_for_a_sitting() {
    let scratch = PrivateService::new(Side::Crate, None).unwrap();
    let (_, dir) = scratch.variable();
    let report = dir.join("runtimes");
    let axis = dir.join("axis.rex");
    fs::write(&axis, "n = 4\nsay n\n").unwrap();
    script(
        &dir.join("perf"),
        r#"#!/bin/sh
shift 4
"$@" || exit "$?"
printf '100,,cycles:u\n200,,instructions:u\n' >&2
"#,
    );
    for name in ["base", "head"] {
        script(
            &dir.join(name),
            r#"#!/bin/sh
printf '%s\t%s\t%s\t%s\t%s\n' "${0##*/}" "$REXX_API_DIR" "$XDG_RUNTIME_DIR" \
  "${RXQUEUE-unset}" "${RXQUEUESESSION-unset}" >>"$REPORT"
test -d "$REXX_API_DIR" || exit 2
exit "$STATUS"
"#,
        );
    }

    let run_sitting = |status: &str| {
        fs::write(&report, "").unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_rexx-arms"))
            .env("PATH", dir)
            .env("REXX_API_DIR", dir)
            .env("XDG_RUNTIME_DIR", dir)
            .env("RXQUEUE", "inherited")
            .env("RXQUEUESESSION", "inherited")
            .env("REPORT", &report)
            .env("STATUS", status)
            .args(["--build", &format!("base={}", dir.join("base").display())])
            .args(["--build", &format!("head={}", dir.join("head").display())])
            .arg("--axis")
            .arg(&axis)
            .arg("--workdir")
            .arg(dir)
            .output()
            .unwrap();
        let observations = fs::read_to_string(&report).unwrap();
        let mut runtimes = BTreeMap::new();
        for line in observations.lines() {
            let fields: Vec<_> = line.split('\t').collect();
            let [build, rexx_api, oracle_api, "unset", "unset"] = fields.as_slice() else {
                panic!("unexpected child environment: {line}");
            };
            assert_eq!(rexx_api, oracle_api);
            let runtime = Path::new(*rexx_api);
            assert_ne!(runtime, dir, "the child used the inherited service");
            assert!(!runtime.exists(), "{} was not removed", runtime.display());
            if let Some(previous) = runtimes.insert(*build, runtime) {
                assert_eq!(previous, runtime, "build {build} changed runtime");
            }
        }
        (
            out,
            runtimes
                .values()
                .map(|path| path.to_path_buf())
                .collect::<Vec<_>>(),
        )
    };

    let (done, runtimes) = run_sitting("0");
    assert!(
        done.status.success(),
        "{}",
        String::from_utf8_lossy(&done.stderr)
    );
    let [base, head] = runtimes.as_slice() else {
        panic!("expected both builds, got {runtimes:?}");
    };
    assert_ne!(base, head, "the builds shared a service");

    let (failed, failure_runtimes) = run_sitting("7");
    assert!(!failed.status.success());
    assert_eq!(failure_runtimes.len(), 1);
    assert!(!runtimes.contains(&failure_runtimes[0]));
}
