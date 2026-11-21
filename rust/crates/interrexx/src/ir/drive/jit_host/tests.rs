//! Programs whose loops run past [`super::THRESHOLD`], run with the `jit`
//! feature, against what the interpreter prints for them
//! (`src/ir/drive/jit_cases`). Each case also says whether compiled code ran
//! at all, so a case whose loop stopped compiling fails rather than passing
//! on the driver alone.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{compiled_runs, quiet_calls, reset_compiled_runs};

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
            format!(
                "{}{}compiled: {compiled}\n{quiet}",
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

/// A quiet builtin that roots temporaries (`MAX` does) leaves none behind it
/// in a clause compiled code opened: the most live at once is the same for a
/// long compiled loop as for a short one.
#[test]
fn quiet_builtin_calls_do_not_accumulate_temps() {
    let high_water = |n: usize| {
        reset_compiled_runs();
        let interp =
            ran(format!("x = 0\ndo i = 1 to {n}\n  x = x + max(i // 7, 3)\nend\nsay x").as_bytes());
        assert!(
            compiled_runs() > 0 && quiet_calls() > 0,
            "{n} passes compiled, quietly"
        );
        interp.roots.activity().temps_capacity()
    };
    let long = high_water(40000);
    assert!(
        long < 1000,
        "{long} temporaries live at once after 40000 passes"
    );
    assert_eq!(
        high_water(10000),
        long,
        "the high-water mark grows with the pass count"
    );
}
