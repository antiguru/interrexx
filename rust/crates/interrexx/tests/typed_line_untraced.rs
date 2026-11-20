//! A method that a line typed at an interactive trace stop calls runs
//! untraced, and a TRACE instruction in it changes nothing
//! (`r[env.debugging.deviation.182]`).
//!
//! The analysis of a chunk claims the setting a `TRACE` instruction in the
//! method leaves; a debug build checks that claim at each clause against the
//! setting the trace sink obeys, which is off here, so a check that excused
//! only the activation of the line itself would abort the run.

use std::process::Command;

const PROGRAM: &str = "\
.traceoutput~destination(.quiet~new)
.debuginput~destination(.in~new('x = .o~new~m'))
trace ?a
nop
nop
nop
say 'reads' .in~count trace()
exit
::class o
::method m
  trace r
  nop
  return 1
::class quiet
::method lineout
  return 0
::class in
::attribute count class
::method init
  expose line
  use arg line
  .in~count = 0
::method linein
  expose line
  .in~count += 1
  if .in~count = 1 then return line
  return ''
";

#[test]
fn a_method_called_from_a_typed_line_runs_untraced_in_a_debug_build() {
    let dir = std::env::temp_dir().join(format!("rexx-typed-untraced-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a private directory");
    let path = dir.join("typed.rex");
    std::fs::write(&path, PROGRAM).expect("the program is writable");
    let out = Command::new(env!("CARGO_BIN_EXE_rexx"))
        .arg("typed.rex")
        .current_dir(&dir)
        .env("REXX_API_DIR", dir.join("api"))
        .output()
        .expect("rexx runs");
    // One line is read for the typed line and one for each of the three
    // stops that follow it; the method adds none.
    assert_eq!(
        (
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
            out.status.code()
        ),
        ("reads 4 ?A\n".to_owned(), String::new(), Some(0))
    );
}

/// The activity a typed line starts is not running while the line runs, so
/// the untraced state does not follow it: the started method is traced as it
/// would be in the program.
const STARTED: &str = "\
.traceoutput~destination(.show~new)
.debuginput~destination(.in~new('r = .o~new~start(\"m\"); r~result'))
trace ?a
nop
nop
exit
::class o
::method m
  trace r
  x = 'in m'
  return 1
::class show
::method lineout
  say 'T>' arg(1)
  return 0
::class in
::attribute count class
::method init
  expose line
  use arg line
  .in~count = 0
::method linein
  expose line
  .in~count += 1
  if .in~count = 1 then return line
  return ''
";

#[test]
fn a_method_started_from_a_typed_line_is_traced() {
    let dir = std::env::temp_dir().join(format!("rexx-typed-started-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a private directory");
    std::fs::write(dir.join("typed.rex"), STARTED).expect("the program is writable");
    let out = Command::new(env!("CARGO_BIN_EXE_rexx"))
        .arg("typed.rex")
        .current_dir(&dir)
        .env("REXX_API_DIR", dir.join("api"))
        .output()
        .expect("rexx runs");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("*-* x = 'in m'"),
        "the started method shows no trace:\n{stdout}"
    );
}
