//! The compiled template walk, run: each test names the part of the walk it
//! is the only witness of.

use crate::{Invocation, SimConfig, SwitchMode, run_program};

/// The path these programs are reported under; nothing reads it back.
const TEST_PATH: &str = "/nonexistent/parse-walk-test.rex";

/// What `text` prints, after checking it ended cleanly.
fn stdout_of(text: &str) -> String {
    let outcome = run_program(TEST_PATH, text.as_bytes().to_vec(), Invocation::none());
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    String::from_utf8(outcome.stdout).expect("the test programs print text")
}

/// **A source that is not its own string value is the template engine's**,
/// asked for its string value once. The walk hands the clause back before it
/// has sent anything, and the driver goes on past the steps it skipped.
#[test]
fn an_object_source_is_parsed_by_the_engine_and_asked_once() {
    let printed = stdout_of(
        ".local~sent = 0\n\
         parse value .thing~new with p q\n\
         say p'|'q'|'.sent\n\
         ::class thing\n\
         ::method string\n\
         .local~sent = .local~sent + 1\n\
         return 'p q'\n",
    );
    assert_eq!(printed, "p|q|1\n");
}

/// A number held as an object renders before it parses, which the walk
/// leaves to the engine as well.
#[test]
fn a_number_object_source_is_parsed_by_the_engine() {
    let printed = stdout_of("x = 2**70\nparse var x p +3 q\nsay p'|'q\n");
    assert_eq!(printed, "1.1|8059162E+21\n");
}

/// `PARSE ARG` reads the next argument at each comma, and an absent one is
/// the null string.
#[test]
fn each_comma_of_parse_arg_steps_to_the_next_argument() {
    let printed = stdout_of(
        "call show 'a b', 'c d', , 'e'\n\
         exit\n\
         show:\n\
         parse arg p q, r, s, t, u\n\
         say p'|'q'|'r'|'s'|'t'|'u'|'\n\
         return\n",
    );
    assert_eq!(printed, "a|b|c d||e||\n");
}

/// **An operand is evaluated after the targets before it are assigned**,
/// which is where the compiled stream puts its ops: between two steps.
#[test]
fn an_operand_reads_a_target_the_template_assigned_before_it() {
    let printed =
        stdout_of("parse value 'k:rest k more' with key ':' . (key) tail\nsay key'|'tail'|'\n");
    assert_eq!(printed, "k| more|\n");
}

/// The same for an operand the stream computes: its ops run between the run
/// that assigned `key` and the run that searches for it.
#[test]
fn a_computed_operand_reads_a_target_the_template_assigned_before_it() {
    let printed = stdout_of(
        "parse value 'k:rest kk more' with key ':' . (key || key) tail\nsay key'|'tail'|'\n",
    );
    assert_eq!(printed, "k| more|\n");
}

/// An unset `PARSE VAR` source raises `NOVALUE` from the walk's own read.
#[test]
fn an_unset_var_source_raises_novalue() {
    let printed = stdout_of(
        "signal on novalue\nparse var unset a '-' b\nsay 'parsed'\nexit\nnovalue:\nsay condition('D')\n",
    );
    assert_eq!(printed, "UNSET\n");
}

/// A compound source is read with its tail resolved, by the engine.
#[test]
fn a_compound_var_source_reads_its_resolved_tail() {
    let printed = stdout_of("i = 7\ns.7 = 'p q'\nparse var s.i f ' ' g\nsay f'|'g'|'\n");
    assert_eq!(printed, "p|q|\n");
}

/// **The parse string stays rooted while a target overwrites its variable**:
/// with a collection at every allocation, the pieces built after `s` is
/// assigned would otherwise find their source collected.
#[test]
fn a_var_source_its_own_target_overwrites_stays_rooted() {
    let config = SimConfig::parse("sim:7,gc=1").expect("a sim spec");
    let outcome = run_program(
        TEST_PATH,
        b"do i = 1 to 3\n\
          s = copies('x', 30) 'yyyyyyyyyyyy zzzzzzzzzzzzzz' i\n\
          parse var s s ' ' t ' ' u ' ' v\n\
          end\n\
          say length(s) t u v\n"
            .to_vec(),
        Invocation::none().with_switch_mode(SwitchMode::Sim(config)),
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert!(
        outcome
            .stdout
            .starts_with(b"30 yyyyyyyyyyyy zzzzzzzzzzzzzz 3\n"),
        "{}",
        String::from_utf8_lossy(&outcome.stdout)
    );
}
