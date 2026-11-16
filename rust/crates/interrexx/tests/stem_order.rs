//! The order a stem answers its tails in: assignment order
//! (`r[class.stem.deviation.1]`). Each case in
//! `tests/stem_order_cases/` is a program and its standard output. The
//! oracle answers in an order its internal storage fixes, so no differential
//! covers these, and the expected blocks follow from
//! `r[class.stem.deviation.1]`.

use interrexx::{Invocation, run_program};

const CASES: &str = "tests/stem_order_cases";

/// Every case runs as a program; the answer is what it wrote, a line each.
#[test]
fn stem_order() {
    let mut cases = 0;
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            cases += 1;
            assert_eq!(
                case.directive, "program",
                "unknown directive {:?}",
                case.directive
            );
            let outcome = run_program(
                "/t.rex",
                case.input.clone().into_bytes(),
                Invocation::none(),
            );
            let stderr = String::from_utf8_lossy(&outcome.stderr);
            assert_eq!(outcome.exit_code, 0, "{}: {stderr}", case.input);
            String::from_utf8_lossy(&outcome.stdout).into_owned()
        });
    });
    assert!(cases > 0, "{CASES} ran no case");
}
