//! ERRORTEXT's second argument (`r[bif.errortext.values]`). Each case in
//! `tests/errortext_values_cases/` is a program and its standard output. A
//! message's wording is unspecified, so the programs never write one: `fills`
//! computes what the message must be from the text ERRORTEXT answers without
//! values, by the rule's own words, and compares.

use interrexx::{Invocation, run_program};

const CASES: &str = "tests/errortext_values_cases";

/// The routines every program may call, appended to it.
///
/// `fills(code, values)` answers `ok`, or both texts, for whether
/// `ERRORTEXT(code, values)` is the message of `code` with its first
/// `values~size` places filled: the place `&i` with the item at `i`, the null
/// string where there is none, and the places after those as they stand; a
/// string stands for an array of that one item.
/// `refused(code, values)` answers the code of the condition it raises.
const ROUTINES: &str = r#"
::routine fills
  use arg code, values
  text = errortext(code)
  given = values
  if \values~isA(.array) then values = .array~of(values)
  want = ''
  places = 0
  do while pos('&', text) > 0
    parse var text before '&' text
    digit = left(text, 1)
    text = substr(text, 2)
    want = want || before
    places = places + 1
    if places > values~size then
      want = want || '&' || digit
    else if values~hasIndex(digit) then
      want = want || values[digit]
  end
  want = want || text
  got = errortext(code, given)
  if got == want then return 'ok'
  return 'got ['got'] want ['want']'
::routine refused
  use arg code, values
  signal on syntax name raised
  x = errortext(code, values)
  return 'no error'
raised:
  return condition('o')~code
"#;

/// Every case runs as a program; the answer is what it wrote, a line each.
#[test]
fn errortext_values() {
    let mut cases = 0;
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            cases += 1;
            assert_eq!(
                case.directive, "program",
                "unknown directive {:?}",
                case.directive
            );
            let source = format!("{}\n{ROUTINES}", case.input);
            let outcome = run_program("/t.rex", source.into_bytes(), Invocation::none());
            let stderr = String::from_utf8_lossy(&outcome.stderr);
            assert_eq!(outcome.exit_code, 0, "{}: {stderr}", case.input);
            String::from_utf8_lossy(&outcome.stdout).into_owned()
        });
    });
    assert!(cases > 0, "{CASES} ran no case");
}
