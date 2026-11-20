//! Programs whose loops run past [`super::THRESHOLD`], run with the `jit`
//! feature, against what the interpreter prints for them
//! (`src/ir/drive/jit_cases`). Each case also says whether compiled code ran
//! at all, so a case whose loop stopped compiling fails rather than passing
//! on the driver alone.

use super::{compiled_runs, reset_compiled_runs};

#[test]
fn compiled_loops_print_what_the_interpreter_prints() {
    datadriven::walk("src/ir/drive/jit_cases", |file| {
        file.run(|case| {
            assert_eq!(case.directive, "run", "the one directive");
            reset_compiled_runs();
            let outcome = crate::run_program(
                "t.rex",
                case.input.as_bytes().to_vec(),
                crate::Invocation::none(),
            );
            let compiled = if compiled_runs() > 0 { "yes" } else { "no" };
            format!(
                "{}{}compiled: {compiled}\n",
                String::from_utf8_lossy(&outcome.stdout),
                String::from_utf8_lossy(&outcome.stderr)
            )
        });
    });
}
