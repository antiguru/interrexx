//! A `MutableBuffer` grown by a mutator that then fails is still counted as
//! holding what it grew to. The failure needs a reservation that succeeds for
//! the buffer and fails for the mutator's result, so the run is capped by
//! `ulimit -v`, which an in-process run cannot be.
//!
//! Linux only: macOS refuses `setrlimit(RLIMIT_AS)` below what a process has
//! already reserved (4 GB refused, 1 TB accepted), and without a cap its
//! allocator grants both reservations, so the failure this test needs cannot
//! be made there.

#![cfg(target_os = "linux")]
#![expect(clippy::disallowed_methods, reason = "this harness bounds a real run")]

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

/// The position `INSERT` is asked to put its one byte at, so the size the
/// buffer grows to is this and one more, and the size of the result it then
/// fails to make.
#[cfg(target_pointer_width = "64")]
const POSITION: u64 = 2_000_000_000;

/// The address-space cap, in KiB: the buffer's 2 GB reservation fits under
/// it and the result's second 2 GB does not. Measured on a debug build, the
/// second fails alone at every cap from 3 000 000 to 4 500 000, both fail at
/// 2 000 000, and both succeed at 5 000 000.
#[cfg(target_pointer_width = "64")]
const CAP_KIB: u32 = 3_500_000;

/// [`POSITION`] on a 32-bit target, whose 4 GB of address space hold a 2 GB
/// reservation only where ASLR left one contiguous hole that large, which it
/// does on some runs and not others (#1101). 600 MB fits in any layout.
#[cfg(not(target_pointer_width = "64"))]
const POSITION: u64 = 600_000_000;

/// [`CAP_KIB`] for the 32-bit [`POSITION`]. Measured on i686, six runs at
/// each cap, the second reservation fails alone at every cap from 900 000 to
/// 1 400 000 in the `test` profile (1 000 000 to 1 400 000 in `dev`), both
/// fail at 800 000 in `dev`, and both succeed at 1 500 000.
#[cfg(not(target_pointer_width = "64"))]
const CAP_KIB: u32 = 1_200_000;

/// **`INSERT` raises 5.0 after the buffer grew, and the collections that
/// follow, with the buffer live and then dropped, find the bytes it holds
/// counted.** In a debug build a collection that disagrees panics; at the
/// commit before the growth was charged where it happens, this run panicked
/// with the survivors holding 2000010805 body bytes against a running figure
/// of 11060.
#[test]
fn a_buffer_grown_before_a_failed_insert_is_counted() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("buffer-growth-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("a probe directory of this run's own");
    fs::write(
        dir.join("main.rex"),
        format!(
            "b = .mutableBuffer~new\n\
             signal on syntax\n\
             b~insert('z', {POSITION})\n\
             say 'inserted'\n\
             exit\n\
             syntax:\n\
             say condition('o')~code b~getBufferSize\n\
             do i = 1 to 200000; x = .object~new; end\n\
             drop b\n\
             do i = 1 to 200000; x = .object~new; end\n\
             say 'done'\n"
        ),
    )
    .expect("the program is writable");
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("ulimit -v {CAP_KIB} && exec \"$0\" main.rex"))
        .arg(env!("CARGO_BIN_EXE_rexx"))
        .current_dir(&dir)
        .stdin(Stdio::null())
        .output()
        .expect("the capped run spawns");
    assert_eq!(
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned()
        ),
        (
            Some(0),
            format!("5.0 {}\ndone\n", POSITION + 1),
            String::new()
        )
    );
}
