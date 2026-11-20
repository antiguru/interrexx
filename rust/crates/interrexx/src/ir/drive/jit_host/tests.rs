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
