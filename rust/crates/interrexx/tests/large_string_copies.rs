//! How many copies of a large string a builtin, a method or `SAY` holds at
//! once, read from the peak resident set the run reports of itself
//! (`VmHWM` in `/proc/self/status`), against a run that only builds the
//! string. Each case holds its operand and its answer and nothing more; the
//! corpus runs this crate in-process, where no such peak can be read.

use interrexx_cast::CastLossy;
use std::path::Path;
use std::process::{Command, Stdio};

/// The length of the large string, in bytes.
const LENGTH: u64 = 64_000_000;

/// The program that builds `x` of `fill`, runs `case`, and says its own peak
/// resident set in KiB as its last line.
fn program(fill: char, case: &str) -> String {
    format!(
        "x = copies('{fill}', {LENGTH})\n\
         {case}\n\
         say SysQueryProcess('PMEM')~word(2)\n"
    )
}

/// The peak resident set in KiB of a run of `case` over `x` of `fill`, with
/// its standard output a pipe.
fn peak_kib(dir: &Path, fill: char, case: &str) -> u64 {
    let file = dir.join("case.rex");
    std::fs::write(&file, program(fill, case)).expect("the program is writable");
    let output = Command::new(env!("CARGO_BIN_EXE_bounded-run"))
        .args(["-k", "5", "120"])
        .arg(env!("CARGO_BIN_EXE_rexx"))
        .arg(&file)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .expect("the run spawns");
    assert_eq!(
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ),
        (Some(0), "".into()),
        "{case}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let last = stdout.lines().last().expect("the run says its peak");
    last.parse().unwrap_or_else(|_| panic!("{case}: {last:?}"))
}

/// The cases under `tests/large_string_copies/`: each `copies` case runs its
/// input over `x` of `fill`, and answers how many copies of `x` the run held
/// at its peak beyond `x` itself, to the nearest quarter. A case holds one
/// for an answer as long as `x` (and for `SAY`, the output buffer, and for
/// `compare`, its other operand), none for a short one, and the answer's own
/// length where that is a multiple of `x`'s. `REWRITE=1` re-records them.
#[test]
fn a_large_operand_is_read_where_it_is() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("large-string-copies");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let base = peak_kib(&dir, 'a', "nop");
    let copy_kib = f64::cast_lossy(LENGTH) / 1024.0;
    let mut cases = 0;
    datadriven::walk("tests/large_string_copies", |file| {
        file.run(|case| {
            cases += 1;
            assert_eq!(case.directive, "copies");
            let fill: char = case.take_arg("fill").expect("a fill");
            let program = case.input.trim_end();
            let extra =
                f64::cast_lossy(peak_kib(&dir, fill, program).saturating_sub(base)) / copy_kib;
            format!("{:.2}\n", (extra * 4.0).round() / 4.0)
        });
    });
    assert!(cases > 0, "no case ran");
    std::fs::remove_dir_all(&dir).expect("the directory is removed");
}
