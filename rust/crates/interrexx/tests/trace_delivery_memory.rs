//! Trace destinations the oracle cannot run, under a bound: one whose
//! `LINEOUT` fails deep in a recursion, under the address-space cap the corpus
//! gives the oracle (`corpus/oracle-crashes.txt` entries 51 and 52), and one
//! whose own clauses are traced (entry 14). The oracle crashes on both, so
//! there is no differential to run.

#![expect(clippy::disallowed_methods, reason = "this harness bounds a real run")]

mod support;

use interrexx_inventory::errors::expand;
use std::fs;
use std::path::Path;
use support::memory_cap::run_under_memory_cap;
use support::oracle::ORACLE_MEMORY_LIMIT_KIB;

/// The peak resident set, in KiB, that counts as a run that grew where the
/// address-space cap is not enforced. Measured on a debug build: 39 MB with
/// each failed delivery dropped, against more than 431 MB without.
const PEAK_RSS_KIB: u64 = 262_144;

/// The recursion each failed delivery runs before it raises.
const DEPTH: usize = 800;

/// **Each report line's failed delivery leaves nothing behind.** The 42.3
/// raised in the traced clause is reported through the same destination,
/// which fails again for every line and leaves the line on stdout.
/// Measured, before a failed delivery's levels were dropped: rc 134 under
/// the cap, peak RSS 431 MB; after, rc 214 at 44 MB.
#[test]
fn a_destination_failing_deep_for_every_report_line_runs_under_the_oracles_memory_cap() {
    let source = format!(
        ".traceOutput~destination(.deep~new)\n\
         trace r\n\
         x = 1\n\
         trace off\n\
         .traceOutput~destination\n\
         say 'after'\n\
         ::class deep\n\
         ::method lineout unguarded\n\
         \x20 call spin {DEPTH}\n\
         \x20 return 0\n\
         ::routine spin\n\
         \x20 use arg k\n\
         \x20 if k = 0 then return 1/0\n\
         \x20 call spin k - 1\n"
    );
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("trace-delivery-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("a probe directory of this run's own");
    let file = dir.join("main.rex");
    fs::write(&file, source).expect("the program is writable");
    let output = run_under_memory_cap(
        ORACLE_MEMORY_LIMIT_KIB,
        PEAK_RSS_KIB,
        env!("CARGO_BIN_EXE_rexx"),
        &[file.as_os_str()],
        &dir,
    );
    let path = file.to_string_lossy().into_owned();
    // A line number in the class library is unspecified
    // (`r[cond.information.library]`), so it is compared as a placeholder.
    let stdout = String::from_utf8_lossy(&support::normalize_library_lines(
        String::from_utf8_lossy(&output.stdout)
            .replace(&path, "PROGRAM")
            .as_bytes(),
    ))
    .into_owned();
    // The traced clause's own line is refused first, and a refused line goes
    // to stdout (`r[instr.trace.destination]`).
    let mut expected = String::from("     3 *-* x = 1\n    13 *-*     return 1/0\n");
    for _ in 0..DEPTH {
        expected.push_str("    14 *-* call spin k - 1\n");
    }
    expected.push_str(&expand(&format!(
        "     9 *-* call spin {DEPTH}\n\
         \x20    N *-* <no source>\n\
         \x20    3 *-* x = 1\n\
         Error 42 running PROGRAM line 13:  ⟨42⟩\n\
         Error 42.3:  ⟨42.3⟩\n"
    )));
    assert_eq!(
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).into_owned()
        ),
        (Some(214), String::new())
    );
    assert!(stdout == expected, "stdout:\n{stdout}");
    fs::remove_dir_all(&dir).expect("the probe directory is removed");
}

#[test]
fn a_destination_exhausting_the_stack_cannot_recurse_for_every_report_line() {
    let source = "\
.traceoutput~destination(.deep~new)
trace r
x = 1
trace off
say 'after'
::class deep
::method lineout unguarded
  call spin
  return 0
spin: call spin
";
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("trace-stack-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("a probe directory of this run's own");
    let file = dir.join("main.rex");
    fs::write(&file, source).expect("the program is writable");
    let output = run_under_memory_cap(
        ORACLE_MEMORY_LIMIT_KIB,
        PEAK_RSS_KIB,
        env!("CARGO_BIN_EXE_bounded-run"),
        &[
            "-s".as_ref(),
            "KILL".as_ref(),
            "30".as_ref(),
            env!("CARGO_BIN_EXE_rexx").as_ref(),
            file.as_os_str(),
        ],
        &dir,
    );
    fs::remove_dir_all(&dir).expect("the probe directory is removed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(245),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.starts_with("     3 *-* x = 1\n"));
    assert!(stdout.lines().any(|line| line.starts_with("Error 11.1:")));
    assert!(output.stderr.is_empty());
}

/// **A destination writes no trace lines of its own.** Under package-wide
/// `::OPTIONS TRACE I` the sink's `LINEOUT` is traced too; its lines are
/// dropped rather than delivered, which would deliver again without end. The
/// main program's lines up to the destination's reset reach the sink.
#[test]
fn a_destination_whose_own_clauses_are_traced_delivers_none_of_them() {
    let source = "s = .sink~new\n\
         zz = .traceoutput~destination(s)\n\
         aa = 1 + 1\n\
         zz = .traceoutput~destination(.stderr)\n\
         zz = .stderr~lineout('total=' s~count)\n\
         ::options trace i\n\
         ::class sink\n\
         ::method init\n\
         \x20 expose count\n\
         \x20 count = 0\n\
         ::method count\n\
         \x20 expose count\n\
         \x20 return count\n\
         ::method lineout\n\
         \x20 expose count\n\
         \x20 count = count + 1\n\
         \x20 return 0\n";
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("trace-own-lines-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("a probe directory of this run's own");
    let file = dir.join("main.rex");
    fs::write(&file, source).expect("the program is writable");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bounded-run"))
        .args(["-s", "KILL", "20"])
        .arg(env!("CARGO_BIN_EXE_rexx"))
        .arg(&file)
        .current_dir(&dir)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("bounded-run starts");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        (
            output.status.code(),
            stderr
                .lines()
                .filter(|line| line.starts_with("total="))
                .collect::<Vec<_>>()
        ),
        (Some(0), vec!["total= 13"]),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    fs::remove_dir_all(&dir).expect("the probe directory is removed");
}
