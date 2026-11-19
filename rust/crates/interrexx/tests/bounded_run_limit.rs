//! `bounded-run -v`: the address-space limit the tests put on a program they start.
#![cfg(target_os = "linux")]

use std::process::Command;

/// The `Max address space` soft limit a command started under `args` reads from
/// `/proc/self/limits`.
fn soft_limit_under(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bounded-run"))
        .args(args)
        .args([
            "20",
            "sh",
            "-c",
            "grep '^Max address space' /proc/self/limits",
        ])
        .output()
        .expect("bounded-run starts");
    assert!(output.status.success(), "{output:?}");
    let line = String::from_utf8(output.stdout).expect("the limits are text");
    line.split_whitespace()
        .nth(3)
        .expect("a soft limit")
        .to_owned()
}

#[test]
fn the_command_runs_under_the_address_space_limit_in_kib() {
    assert_eq!(soft_limit_under(&["-v", "262144"]), "268435456");
}

#[test]
fn a_limit_above_the_hard_one_leaves_the_hard_one() {
    let output = Command::new("sh")
        .args(["-c", "ulimit -v 1048576 && exec \"$@\"", "sh"])
        .arg(env!("CARGO_BIN_EXE_bounded-run"))
        .args(["-v", "4194304", "20", "sh", "-c"])
        .arg("grep '^Max address space' /proc/self/limits")
        .output()
        .expect("sh starts");
    assert!(output.status.success(), "{output:?}");
    let line = String::from_utf8(output.stdout).expect("the limits are text");
    let fields: Vec<&str> = line.split_whitespace().collect();
    assert_eq!(fields[3..5], ["1073741824", "1073741824"], "{line}");
}

#[test]
fn a_limit_that_is_not_a_number_is_a_usage_error() {
    let status = Command::new(env!("CARGO_BIN_EXE_bounded-run"))
        .args(["-v", "4G", "20", "true"])
        .status()
        .expect("bounded-run starts");
    assert_eq!(status.code(), Some(125));
}
