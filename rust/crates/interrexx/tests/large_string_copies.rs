//! How many copies of a large string a builtin, a method or `SAY` holds at
//! once, read from the peak resident set the run reports of itself
//! (`VmHWM` in `/proc/self/status`), against a run that only builds the
//! string. Each case holds its operand and its answer and nothing more; the
//! corpus runs this crate in-process, where no such peak can be read.

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
         f = '/proc/self/status'\n\
         do forever\n\
         \x20 l = linein(f)\n\
         \x20 if stream(f) \\= 'READY' then leave\n\
         \x20 if l~startsWith('VmHWM') then peak = l~word(2)\n\
         end\n\
         say peak\n"
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

/// #65: each case holds no more copies of `x`, beyond `x` itself, than its
/// answer needs: one for an answer as long as `x` (and for `SAY`, the
/// output buffer, and for `compare`, its other operand), none for a short one,
/// and the answer's own length where that is a multiple of `x`'s.
#[test]
fn a_large_operand_is_read_where_it_is() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("large-string-copies");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let base = peak_kib(&dir, 'a', "nop");
    let left = format!("y = left(x, {LENGTH})");
    let cases: [(char, &str, f64); 25] = [
        ('a', "say x", 1.0),
        ('a', "y = strip(x)", 1.0),
        ('a', "y = strip(x, 'L')", 1.0),
        ('a', "y = left(x, 10)", 0.0),
        ('a', &left, 1.0),
        ('a', "y = right(x, 10)", 0.0),
        ('a', "y = center(x, 10)", 0.0),
        ('a', "y = overlay('b', x)", 1.0),
        ('a', "y = insert('b', x)", 1.0),
        ('a', "y = subword(x, 1)", 1.0),
        ('a', "y = space(x)", 1.0),
        ('a', "y = wordlength(x, 1) wordindex(x, 1)", 0.0),
        ('a', "y = x~left(10)", 0.0),
        ('a', "y = x~right(10)", 0.0),
        ('a', "y = x~center(10)", 0.0),
        ('a', "y = x~copies(1)", 0.0),
        ('a', "y = x~strip", 0.0),
        ('a', "y = x~abbrev('a')", 0.0),
        ('a', "y = x~caselessAbbrev('a')", 0.0),
        ('a', "y = x~compare(x)", 1.0),
        ('a', "y = x~caselessCompare(x)", 1.0),
        ('a', "y = x~c2x", 2.0),
        ('a', "y = x~x2c", 0.5),
        ('a', "y = x~x2b", 4.0),
        ('1', "y = x~b2x", 0.25),
    ];
    let copy_kib = LENGTH as f64 / 1024.0;
    let mut wrong = Vec::new();
    for (fill, case, copies) in cases {
        let extra = peak_kib(&dir, fill, case).saturating_sub(base) as f64 / copy_kib;
        // Half a copy of slack, for the allocator's own rounding.
        if extra > copies + 0.5 {
            wrong.push(format!(
                "{fill} {case}: {extra:.2} copies past x, where {copies} is enough"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    std::fs::remove_dir_all(&dir).expect("the directory is removed");
}
