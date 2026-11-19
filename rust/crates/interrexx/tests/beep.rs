//! BEEP's answer, which no specification example can show on both
//! interpreters: an accepted call writes a bell here when no tone can be
//! sounded, and ooRexx at level 6.07 writes none (Deviation 179).

use interrexx::{Invocation, run_program};

/// spec: bif.beep.answer
///
/// The answer is the null string for every accepted pair, and a CALL sets
/// RESULT to it. The bells the calls write are taken out of the output; their
/// own rule is `bif.beep.bell`.
#[test]
fn an_accepted_call_answers_the_null_string_and_sets_result() {
    let outcome = run_program(
        "/beep.rex",
        b"say '[' || beep(100, 0) || ']' length(beep(37, 0))\n\
          call beep 32767, 0\n\
          say symbol('RESULT') length(result)\n"
            .to_vec(),
        Invocation::none(),
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    let shown: Vec<u8> = outcome.stdout.iter().copied().filter(|&b| b != 7).collect();
    assert_eq!(String::from_utf8_lossy(&shown), "[] 0\nVAR 0\n");
}
