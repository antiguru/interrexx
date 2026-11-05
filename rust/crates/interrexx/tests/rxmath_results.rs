//! The `rxmath` library run in Rust, against the oracle's `librxmath.so`
//! (`docs/design/extensions.md`). Every
//! expected answer in `tests/rxmath_results/` was recorded by running the same
//! program on the oracle, except the `MathLoadFuncs` banner, which is
//! interrexx's own (Deviation 96) with its version masked as `<version>`.
//!
//! * `eval digits=D [form=F]`: each input line is an expression, said under
//!   `NUMERIC DIGITS D` (and `NUMERIC FORM F`), with `::requires 'rxmath'
//!   LIBRARY`.
//! * `raises`: each input line is an expression whose SYNTAX condition is
//!   trapped and shown as its code and secondary message.
//! * `program`: the input is the whole program; the case records rc, stdout
//!   and stderr.

use std::collections::HashMap;

use interrexx::Invocation;

mod watchdog;

const CASES: &str = "tests/rxmath_results";

/// The path every case runs as.
const PATH: &str = "probe.rex";

/// The program an `eval` case runs.
fn eval_program(args: &HashMap<String, Vec<String>>, lines: &[&str]) -> String {
    let digits = args.get("digits").map_or("9", |values| values[0].as_str());
    let mut program = format!("numeric digits {digits}\n");
    if let Some(form) = args.get("form") {
        program.push_str(&format!("numeric form {}\n", form[0]));
    }
    for line in lines {
        program.push_str(&format!("say {line}\n"));
    }
    program.push_str("::requires 'rxmath' LIBRARY\n");
    program
}

/// The program a `raises` case runs.
fn raises_program(lines: &[&str]) -> String {
    let mut program = String::new();
    for (index, line) in lines.iter().enumerate() {
        program.push_str(&format!(
            "signal on syntax name s{index}\nsay {line}\nsay \"no error\"\nsignal n{index}\n\
             s{index}: say condition(\"o\")~code condition(\"o\")~message\nn{index}: nop\n"
        ));
    }
    program.push_str("::requires 'rxmath' LIBRARY\n");
    program
}

fn run(program: String) -> interrexx::Outcome {
    watchdog::run_bounded(PATH, program.into_bytes(), Invocation::none())
}

#[test]
fn rxmath_answers_as_the_oracle_does() {
    let mut cases = 0;
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            cases += 1;
            let lines: Vec<&str> = case.input.lines().collect();
            let args: HashMap<String, Vec<String>> = case
                .args
                .iter()
                .map(|(key, values)| (key.clone(), values.clone()))
                .collect();
            match case.directive.as_str() {
                "eval" | "raises" => {
                    let program = if case.directive == "eval" {
                        eval_program(&args, &lines)
                    } else {
                        raises_program(&lines)
                    };
                    let outcome = run(program);
                    assert_eq!(
                        outcome.exit_code,
                        0,
                        "{}",
                        String::from_utf8_lossy(&outcome.stderr)
                    );
                    String::from_utf8_lossy(&outcome.stdout).into_owned()
                }
                "program" => {
                    let outcome = run(format!("{}\n", case.input.trim_end()));
                    format!(
                        "rc {}\n[stdout]\n{}[stderr]\n{}",
                        outcome.exit_code,
                        String::from_utf8_lossy(&outcome.stdout)
                            .replace(env!("CARGO_PKG_VERSION"), "<version>"),
                        String::from_utf8_lossy(&outcome.stderr),
                    )
                }
                other => panic!("unknown directive {other}"),
            }
        });
    });
    assert!(cases > 0, "no case ran");
}
