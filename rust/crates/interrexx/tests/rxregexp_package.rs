//! Where `rxregexp.cls` comes from, and what a pathological pattern costs.
//!
//! * PR 4 Review Focus 1: this crate ships `rxregexp.cls` and finds it after
//!   every search route, so a program whose directory holds its own
//!   `rxregexp.cls` loads that one, as on the oracle (measured: a program
//!   beside an `rxregexp.cls` defining `RegularExpression~which` prints
//!   `the program's own` from either directory).
//! * Review Focus 3: the patterns Deviation 87 rules on, which never answer
//!   on the oracle, answer here in time and memory bounded by the subject and
//!   the pattern.

use std::time::{Duration, Instant};

use interrexx::Invocation;

mod watchdog;

#[test]
fn a_programs_own_rxregexp_cls_wins() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("rxregexp-own-package");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    std::fs::write(
        dir.join("rxregexp.cls"),
        "::class RegularExpression public\n::method which\n  return \"the program's own\"\n",
    )
    .expect("the program's package");
    let program = dir.join("main.rex");
    let source = "say .RegularExpression~new~which\n::requires \"rxregexp.cls\"\n";
    let outcome = watchdog::run_bounded(
        program.to_str().expect("a UTF-8 path"),
        source.as_bytes().to_vec(),
        Invocation::none(),
    );
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "the program's own\n",
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
}

/// The process's peak resident set, in KiB.
fn peak_kib() -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").expect("/proc/self/status");
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|rest| rest.trim().trim_end_matches("kB").trim().parse().ok())
        .expect("a VmHWM line")
}

fn run(source: String) -> String {
    let outcome = watchdog::run_bounded("bounds.rex", source.into_bytes(), Invocation::none());
    assert_eq!(
        outcome.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    String::from_utf8_lossy(&outcome.stdout).into_owned()
}

/// The program matching each pattern against `subject`, a megabyte long.
fn program(length: usize) -> String {
    format!(
        "s = copies('a', {length})'b'\n\
         do p over '(a*)*', '(a*)+b', '((a)*)*', '(a*b*)*c', '[' || '01'x || '-' || '7F'x || ']*'\n\
           r = .RegularExpression~new(p)\n\
           say r~match(s) r~position r~pos(s) r~position\n\
         end\n\
         ::requires 'rxregexp.cls'\n"
    )
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "the bound is on the real run's wall time"
)]
fn repeated_empty_matches_and_a_range_to_7f_answer_in_bounded_time_and_memory() {
    // A small run first, so the interpreter's own start is in the baseline.
    let small = run(program(10));
    assert_eq!(
        small,
        "0 10 1 10\n1 11 1 11\n0 10 1 10\n0 11 0 0\n1 11 1 11\n"
    );
    let baseline = peak_kib();
    let length = 1 << 20;
    let started = Instant::now();
    let answered = run(program(length));
    let took = started.elapsed();
    let n = length;
    assert_eq!(
        answered,
        format!(
            "0 {n} 1 {n}\n1 {m} 1 {m}\n0 {n} 1 {n}\n0 {m} 0 0\n1 {m} 1 {m}\n",
            m = n + 1
        )
    );
    assert!(took < Duration::from_secs(30), "took {took:?}");
    // The subject itself is a megabyte, held a few times over by the
    // program; the automaton and its thread sets are a few kilobytes.
    let grown = peak_kib().saturating_sub(baseline);
    assert!(grown < 64 * 1024, "the peak grew by {grown} KiB");
}

/// A count whose copies would pass the automaton's state budget is compile
/// error 5 at its `{` (Deviation 87), answered before anything is built:
/// `new` raises 38, `parse` returns 5, and the object then matches
/// everything. The oracle crashes on each of these (measured). A count
/// within the budget answers as before.
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "the bound is on the real run's wall time"
)]
fn huge_and_nested_counts_answer_in_bounded_time_and_memory() {
    let source = "do p over 'a{2000000000}', '((a{1000}){1000}){1000}', 'a{2147483647}', 'b(a{50000}){50000}'\n\
           r = .RegularExpression~new\n\
           say r~parse(p) r~position r~match('aa') r~position make(p)\n\
         end\n\
         r = .RegularExpression~new('(a{1000}){100}')\n\
         say r~match(copies('a', 100000)) r~position r~match(copies('a', 99999)) r~position\n\
         exit\n\
         make: procedure\n\
           use arg p\n\
           signal on syntax name refused\n\
           r = .RegularExpression~new(p)\n\
           return 'made'\n\
         refused:\n\
           return condition('o')~code\n\
         ::requires 'rxregexp.cls'\n";
    run(program(10));
    let baseline = peak_kib();
    let started = Instant::now();
    let answered = run(source.to_owned());
    let took = started.elapsed();
    assert_eq!(
        answered,
        "5 1 1 0 38.0\n5 17 1 0 38.0\n5 1 1 0 38.0\n5 11 1 0 38.0\n1 100000 0 99999\n"
    );
    assert!(took < Duration::from_secs(30), "took {took:?}");
    let grown = peak_kib().saturating_sub(baseline);
    assert!(grown < 64 * 1024, "the peak grew by {grown} KiB");
}
