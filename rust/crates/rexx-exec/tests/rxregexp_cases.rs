//! The `rxregexp` library and `rxregexp.cls` run in Rust
//! (`docs/superpowers/specs/extensions-behaviour.md`, "rxregexp").
//!
//! * `probe`: each input line is a mode (`max` or `min`), a pattern and a
//!   subject, tab-separated, with `%hh` standing for a byte. The output line
//!   is `parse` rc@position, then `match` and `pos`, each @position, on one
//!   `RegularExpression`. Every expected line is the oracle's.
//! * `deviation`: the same, for patterns Deviation 87 rules this crate answers
//!   by the grammar. The expected lines are this crate's; the case comment
//!   gives the oracle's.
//! * `ours`: the input is a whole program, for Deviation 88, whose oracle
//!   answer is a crash.
//! * `program`: the input is a whole program; the case records the oracle's
//!   rc, stdout and stderr.

use rexx_exec::Invocation;

mod watchdog;

const CASES: &str = "tests/rxregexp_cases";

const PATH: &str = "probe.rex";

/// The routine a `probe` case's program calls once per line.
const PROBE: &str = "probe: procedure
  use arg mode, pat, subj
  r = .RegularExpression~new
  rc1 = r~parse(dec(pat), (mode = 'min')~?('MINIMAL', 'MAXIMAL'))
  a = rc1'@'r~position
  b = r~match(dec(subj))'@'r~position
  c = r~pos(dec(subj))'@'r~position
  say a b c
  return
dec: procedure
  parse arg t
  out = ''
  do while t \\== ''
    parse var t c 2 rest
    if c == '%' then do
      out = out || x2c(left(rest, 2)); t = substr(rest, 3)
    end
    else do
      out = out || c; t = rest
    end
  end
  return out
::requires 'rxregexp.cls'
";

/// `text` as a Rexx string literal.
fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', "''"))
}

/// The program a `probe` or `deviation` case runs.
fn probe_program(input: &str) -> String {
    let mut program = String::new();
    for line in input.lines() {
        let mut fields = line.splitn(3, '\t');
        let mode = fields.next().expect("a mode");
        let pattern = fields.next().expect("a pattern");
        let subject = fields.next().unwrap_or("");
        program.push_str(&format!(
            "call probe {}, {}, {}\n",
            quoted(mode),
            quoted(pattern),
            quoted(subject)
        ));
    }
    program.push_str("exit\n");
    program.push_str(PROBE);
    program
}

fn run(program: String) -> rexx_exec::Outcome {
    watchdog::run_bounded(PATH, program.into_bytes(), Invocation::none())
}

#[test]
fn rxregexp_answers_as_described() {
    let mut cases = 0;
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            cases += 1;
            match case.directive.as_str() {
                "probe" | "deviation" => {
                    let outcome = run(probe_program(&case.input));
                    assert_eq!(
                        outcome.exit_code,
                        0,
                        "{}",
                        String::from_utf8_lossy(&outcome.stderr)
                    );
                    String::from_utf8_lossy(&outcome.stdout).into_owned()
                }
                "ours" | "program" => {
                    let outcome = run(format!("{}\n", case.input.trim_end()));
                    format!(
                        "rc {}\n[stdout]\n{}[stderr]\n{}",
                        outcome.exit_code,
                        String::from_utf8_lossy(&outcome.stdout),
                        String::from_utf8_lossy(&outcome.stderr),
                    )
                }
                other => panic!("unknown directive {other}"),
            }
        });
    });
    assert!(cases > 0, "no case ran");
}
