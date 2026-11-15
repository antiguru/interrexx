//! A builtin argument whose user `STRING` method answers an object with no
//! string value (Deviations 30 and 169). Each `call` and `refuses` case runs
//! `say <call>` with `o` an instance whose `STRING` answers the `string=`
//! object; `program` runs its input as the whole program. A `call` or
//! `program` case records the oracle's rc, stdout and stderr. A `refuses`
//! case records this crate's answer, and its comment the oracle's: SYNTAX 88.909
//! where the call's argument is one the builtin reads as a string, and SYNTAX
//! 98.973 at a position the builtin reads as a number, a character or an
//! option (`r[conv.string.deviation.169]`). It records as well what a trap of
//! the error sees: the code, and for 88.909 the argument's position
//! (`r[conv.string.builtin-argument]`).

use interrexx::Invocation;

#[path = "support/error_text.rs"]
mod error_text;
mod watchdog;

const CASES: &str = "tests/string_answer_arguments";

/// The path every case runs as, which the recorded stderr spells `PROGRAM`.
const PATH: &str = "probe.rex";

/// The program a `call` case runs.
fn call_program(call: &str, answer: &str) -> String {
    format!(
        "o = .q~new\nsay {call}\nsay \"after\"\nexit\n::class q\n::method string\n  return {answer}\n"
    )
}

/// The program a `refuses` case runs with the error trapped: it prints the
/// code, and for 88.909 the position of the argument, the item that
/// `r[conv.string.builtin-argument]` fixes as `ADDITIONAL[1]`.
fn trapped_program(call: &str, answer: &str) -> String {
    format!(
        "o = .q~new\nsignal on syntax\nsay {call}\nsay \"after\"\nexit\nsyntax: c = condition('o')\nif c~code == 88.909 then say c~code c~additional[1]\nelse say c~code\nexit\n::class q\n::method string\n  return {answer}\n"
    )
}

/// `rc`, stdout and stderr as the case file records them.
fn transcript(text: String) -> String {
    let outcome = watchdog::run_bounded(PATH, text.into_bytes(), Invocation::none());
    // The message text of an error report is not specified, so a recorded
    // answer holds the code and subcode of each report on stderr and not its
    // text.
    format!(
        "rc {}\n[stdout]\n{}[stderr]\n{}",
        outcome.exit_code,
        String::from_utf8_lossy(&outcome.stdout),
        String::from_utf8_lossy(&error_text::cut_error_texts(&outcome.stderr))
            .replace(PATH, "PROGRAM"),
    )
}

#[test]
fn string_answers_at_builtin_argument_positions() {
    let mut cases = 0;
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            cases += 1;
            let input = case.input.trim_end();
            match case.directive.as_str() {
                "call" | "refuses" => {
                    let answer = match case.args.get("string").map(|v| v[0].as_str()) {
                        Some("object") => ".object~new",
                        Some("directory") => ".directory~new",
                        other => panic!("unknown string= {other:?}"),
                    };
                    let mut out = transcript(call_program(input, answer));
                    if case.directive == "refuses" {
                        let trapped = watchdog::run_bounded(
                            PATH,
                            trapped_program(input, answer).into_bytes(),
                            Invocation::none(),
                        );
                        out.push_str("[trapped]\n");
                        out.push_str(&String::from_utf8_lossy(&trapped.stdout));
                    }
                    out
                }
                "program" => transcript(format!("{input}\n")),
                other => panic!("unknown directive {other}"),
            }
        });
    });
    assert!(cases > 0, "no case ran");
}
