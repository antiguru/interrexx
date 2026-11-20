//! A run of the interpreter binary under a memory bound the host can enforce,
//! for the tests whose regression is a run that grows far past what it needs.
//!
//! Linux caps the address space with `ulimit -v`, so a regression is killed or
//! refused at the cap. macOS refuses any such limit below what a process has
//! already reserved, so there the run is measured instead: `/usr/bin/time -l`
//! reports its peak resident set, and a peak above the bound fails the run
//! after the fact. That finds a program that grows by gigabytes, which is what
//! these tests guard against, but it does not stop one that never ends.

#![allow(dead_code)]
#![expect(clippy::disallowed_methods, reason = "this harness bounds a real run")]

use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// Runs `program` with `args` in `dir`, with no standard input, under the
/// host's memory bound, and answers its output.
///
/// On Linux the program starts under `ulimit -v address_space_kib`. On macOS
/// it starts under `/usr/bin/time -l` and the call panics if the run's peak
/// resident set passed `peak_rss_kib`. The output is the program's own: the
/// measurement goes to a file in `dir`, never to the program's stderr.
pub fn run_under_memory_cap(
    address_space_kib: u64,
    peak_rss_kib: u64,
    program: &str,
    args: &[&OsStr],
    dir: &Path,
) -> Output {
    if cfg!(target_vendor = "apple") {
        let report = dir.join("time-report.txt");
        let output = Command::new("/usr/bin/time")
            .arg("-l")
            .arg("-o")
            .arg(&report)
            .arg(program)
            .args(args)
            .current_dir(dir)
            .stdin(Stdio::null())
            .output()
            .expect("the measured run spawns");
        let text = std::fs::read_to_string(&report).expect("time wrote its report");
        // The figure is in bytes on macOS, on a line ending in the label.
        let peak_bytes = text
            .lines()
            .find_map(|line| line.trim().strip_suffix("maximum resident set size"))
            .and_then(|figure| figure.trim().parse::<u64>().ok())
            .unwrap_or_else(|| panic!("no peak resident set in time's report:\n{text}"));
        assert!(
            peak_bytes <= peak_rss_kib * 1024,
            "the run's peak resident set was {peak_bytes} bytes, over the bound of {} bytes",
            peak_rss_kib * 1024
        );
        output
    } else {
        Command::new("bash")
            .arg("-c")
            .arg(format!(
                "ulimit -v {address_space_kib} && exec \"$0\" \"$@\""
            ))
            .arg(program)
            .args(args)
            .current_dir(dir)
            .stdin(Stdio::null())
            .output()
            .expect("the capped run spawns")
    }
}
