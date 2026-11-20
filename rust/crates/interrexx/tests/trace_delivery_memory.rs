//! A trace destination whose `LINEOUT` fails deep in a recursion, under the
//! address-space cap the corpus gives the oracle. The oracle segfaults on this
//! shape (`corpus/oracle-crashes.txt` entries 51 and 52), so there is no
//! differential to run.

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
