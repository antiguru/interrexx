//! Programs whose loops run past [`super::THRESHOLD`], run with the `jit`
//! feature, against what the interpreter prints for them
//! (`src/ir/drive/jit_cases`). Each case also says whether compiled code ran
//! at all, so a case whose loop stopped compiling fails rather than passing
//! on the driver alone.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{
    compiled_runs, handed_back, host_boundaries, quiet_calls, quiet_compounds, reset_compiled_runs,
};

#[test]
fn compiled_loops_print_what_the_interpreter_prints() {
    datadriven::walk("src/ir/drive/jit_cases", |file| {
        file.run(|case| {
            assert_eq!(case.directive, "run", "the one directive");
            // Counted on the interpreter's own thread, which is where the
            // counter lives, and handed back.
            let runs = Arc::new(AtomicUsize::new(0));
            let seen = Arc::clone(&runs);
            let quiet = Arc::new(AtomicUsize::new(0));
            let quiet_seen = Arc::clone(&quiet);
            let compounds = Arc::new(AtomicUsize::new(0));
            let compounds_seen = Arc::clone(&compounds);
            let text = case.input.as_bytes().to_vec();
            let outcome = crate::on_thread_of(crate::INTERPRETER_STACK_BYTES, move || {
                reset_compiled_runs();
                let ran = crate::execute_kept(
                    b"t.rex",
                    text,
                    crate::Entry::Run,
                    crate::Invocation::none(),
                    Some(crate::INTERPRETER_STACK_BYTES),
                );
                seen.store(compiled_runs(), Ordering::Relaxed);
                quiet_seen.store(quiet_calls(), Ordering::Relaxed);
                compounds_seen.store(quiet_compounds(), Ordering::Relaxed);
                ran
            });
            let compiled = if runs.load(Ordering::Relaxed) > 0 {
                "yes"
            } else {
                "no"
            };
            // Whether a builtin call took the quiet path, printed only where
            // one did, so the cases with no builtin call read as before.
            let quiet = if quiet.load(Ordering::Relaxed) > 0 {
                "quiet builtin calls: yes\n"
            } else {
                ""
            };
            let compounds = if compounds.load(Ordering::Relaxed) > 0 {
                "quiet compound accesses: yes\n"
            } else {
                ""
            };
            format!(
                "{}{}compiled: {compiled}\n{quiet}{compounds}",
                String::from_utf8_lossy(&outcome.stdout),
                String::from_utf8_lossy(&outcome.stderr)
            )
        });
    });
}

/// Runs `source` as a program on this thread, as `run::tests::run_source`
/// does, and answers the interpreter after it.
fn ran(source: &[u8]) -> crate::Interp {
    use crate::plan::BodyKey;
    use std::rc::Rc;
    let mut interp = crate::Interp::new();
    let program =
        Rc::new(interrexx_parse::parse_program(source.to_vec()).expect("test program parses"));
    let program_id = interp.programs.insert(Rc::clone(&program));
    let plan = interp.plan_for(
        BodyKey {
            program: program_id,
            directive: None,
        },
        &program.main,
        &program.symbols,
        &program.source,
    );
    let frame = interp.roots.push_slots(plan.len());
    let id = interp.next_activation_id();
    interp.push_activation(crate::Activation::new(
        id,
        Rc::clone(&program),
        program_id,
        plan,
        frame,
    ));
    interp.run_activation().expect("test program runs");
    interp
}

/// The most temporaries live at once in a run of `body` in a compiled loop
/// of `n` passes, which also ran `counted` quietly.
fn temps_high_water(body: &str, n: usize, counted: fn() -> usize) -> usize {
    reset_compiled_runs();
    let interp = ran(format!("x = 0; t. = 0\ndo i = 1 to {n}\n  {body}\nend\nsay x").as_bytes());
    assert!(
        compiled_runs() > 0 && counted() > 0,
        "{n} passes compiled, quietly"
    );
    interp.roots.activity().temps_capacity()
}

/// A quiet builtin that roots temporaries (`MAX` does) leaves none behind it
/// in a clause compiled code opened: the most live at once is the same for a
/// long compiled loop as for a short one.
#[test]
fn quiet_builtin_calls_do_not_accumulate_temps() {
    let body = "x = x + max(i // 7, 3)";
    let long = temps_high_water(body, 40000, quiet_calls);
    assert!(
        long < 1000,
        "{long} temporaries live at once after 40000 passes"
    );
    assert_eq!(
        temps_high_water(body, 10000, quiet_calls),
        long,
        "the mark grows with the passes"
    );
}

/// A quiet compound write roots its value, and releases it with the write.
#[test]
fn quiet_compound_writes_do_not_accumulate_temps() {
    let body = "k = i // 7; t.k = i";
    let long = temps_high_water(body, 40000, quiet_compounds);
    assert!(
        long < 1000,
        "{long} temporaries live at once after 40000 passes"
    );
    assert_eq!(
        temps_high_water(body, 10000, quiet_compounds),
        long,
        "the mark grows with the passes"
    );
}

/// A block whose start the host runs, as it does where the countdown has
/// one clause left in it, hands back to compiled code at the start's own
/// clause end: the loop is entered compiled once, however many slices its
/// passes cross.
#[test]
fn a_block_started_on_the_host_carries_on_compiled() {
    reset_compiled_runs();
    ran(b"a = 0\ndo i = 1 to 40000\n  do\n    a = a + i\n  end\nend\nsay a");
    assert!(handed_back() > 0, "no block start ran on the host");
    assert_eq!(
        compiled_runs(),
        1,
        "the loop left compiled code at a block start"
    );
}

/// A loop whose body does work of its own around an inner loop compiles
/// whole once it is hot. Its inner loop, three times as hot, compiles first
/// and is entered on every outer pass until then; from the outer loop's
/// compile on, compiled code is not left, so the entries do not grow with the
/// passes after it.
#[test]
fn an_outer_loop_compiles_with_its_inner_loop() {
    let entries = |n: u32| {
        reset_compiled_runs();
        ran(format!(
            "a = 0; b = 0; c = 0; t = 0\n\
             do i = 1 to {n}\n\
             \x20 a = a + i; b = b + a // 7; c = c + b - a // 3\n\
             \x20 d = a * 2; e = d - c; f = e + 1; g = f // 5; h = g + a\n\
             \x20 do j = 1 to 3\n\
             \x20   t = t + j\n\
             \x20 end\n\
             \x20 a = a - h // 11\n\
             end\n\
             say a b c t"
        )
        .as_bytes());
        compiled_runs()
    };
    let short = entries(super::THRESHOLD + 2000);
    assert_eq!(
        entries(super::THRESHOLD + 20000),
        short,
        "compiled code was left on outer passes after the outer loop compiled"
    );
}

/// A loop whose `WHILE` calls a builtin runs its test region in its compiled
/// passes: once hot it is entered once, not once a pass.
#[test]
fn a_loop_with_a_test_region_is_entered_once() {
    reset_compiled_runs();
    ran(format!(
        "n = 0; m = 0\n\
         do i = 1 while length(i) < 6 & i <= {}\n\
         \x20 n = n + i; a = n // 7; b = a * 3; c = b - i; d = c + a\n\
         \x20 m = m + d; e = m // 11; g = e - b; h = g + c; k = h // 5\n\
         end\n\
         say n m k i",
        super::THRESHOLD + 5000
    )
    .as_bytes());
    assert_eq!(
        compiled_runs(),
        1,
        "the loop left compiled code at its test"
    );
}

/// An inner loop whose `WHILE` calls a builtin ends in its test region, which
/// settles in compiled code: from the outer loop's compile on, the entries do
/// not grow with the passes.
#[test]
fn an_inner_loop_ending_in_its_test_settles_in_compiled_code() {
    let entries = |n: u32| {
        reset_compiled_runs();
        ran(format!(
            "t = 0; s = 0\n\
             do i = 1 to {n}\n\
             \x20 a = i * 3; b = a // 7; c = b + i; d = c - a; e = d * 2; g = e // 9\n\
             \x20 do j = 1 while min(j, 99) <= 2\n\
             \x20   t = t + j\n\
             \x20 end\n\
             \x20 h = t // 13; m = h + g; n = m - b; p = n * 2; q = p // 3; r = q + s\n\
             end\n\
             say t r"
        )
        .as_bytes());
        compiled_runs()
    };
    let short = entries(super::THRESHOLD + 2000);
    assert_eq!(
        entries(super::THRESHOLD + 20000),
        short,
        "compiled code was left where an inner loop's test ended it"
    );
}

/// A loop whose `WHILE` or `UNTIL` runs from `Chunk::conditions` has its pass
/// boundary taken by generated code, condition and all. The host takes the
/// ones where the record is worked out, the last, where the condition no
/// longer holds, and those the clause countdown runs out at, about one in a
/// thousand clauses: far fewer than one a pass.
#[test]
fn a_value_only_condition_is_tested_in_generated_code() {
    for header in [
        "do i = 1 while i <= LIMIT",
        "do i = 1 until i >= LIMIT",
        "do LIMIT while n >= 0 & p < LIMIT",
        "do while p < LIMIT | n < 0",
    ] {
        let boundaries = |limit: u32| {
            reset_compiled_runs();
            let header = header.replace("LIMIT", &limit.to_string());
            ran(format!(
                "n = 0; m = 0; p = 0\n\
                 {header}\n\
                 \x20 n = n + 1; p = p + 1; a = n // 7; b = a * 3; c = b - 1; d = c + a\n\
                 \x20 m = m + d // 5; e = m // 11; g = e - b; h = g + c; k = h // 5\n\
                 end\n\
                 say n m k p"
            )
            .as_bytes());
            host_boundaries()
        };
        let more = 18000;
        let short = boundaries(super::THRESHOLD + 2000);
        let long = boundaries(super::THRESHOLD + 2000 + more);
        assert!(
            long - short < more as usize / 100,
            "{header}: the host took {} of {more} more pass boundaries",
            long - short
        );
    }
}
