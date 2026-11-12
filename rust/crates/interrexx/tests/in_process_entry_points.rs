//! Entry points the spec examples reach through the `rexx` binary, which a
//! coverage run does not see and a unit test does not name: `D2X`/`D2C` as
//! messages, `Package~options` and the `::OPTIONS` text it and
//! `Package~defaultOptions` read, and the executable generation behind
//! `Method~new`, `Routine~new` and their `newFile` forms.
//!
//! One program drives them in process, so that each has an assertion of its
//! own that fails when the entry point is cut.

use interrexx::{Invocation, run_program};

#[test]
fn the_message_forms_the_options_message_and_the_executable_generators_answer() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("in-process-entry-points");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let loaded = dir.join("loaded.rex");
    std::fs::write(&loaded, "say 'b'\n::routine zz\n").expect("the loaded file");
    let program = format!(
        "say '255'~d2x '65'~d2c '-1'~d2x(2)
p = .context~package
say p~options('DIGITS')
say p~options('DIGITS', 20)
say p~options('DIGITS')
junk = p~options('SETOPTIONS', '::OPTIONS FUZZ 2')
say p~options('FUZZ')
junk = .Package~defaultOptions('D', '::OPTIONS FORM ENGINEERING')
m = .method~new('m', 'say 1; ::class foo')
say m~package~classes~hasIndex('FOO')
r = .routine~new('r', 'say 1; ::routine q')
say r~package~routines~hasIndex('Q')
f = .routine~newFile({path:?})
say f~package~routines~hasIndex('ZZ')
g = .method~newFile({path:?})
say g~package~routines~hasIndex('ZZ')
",
        path = loaded.to_str().expect("a UTF-8 path")
    );
    let outcome = run_program(
        dir.join("main.rex").to_str().expect("a UTF-8 path"),
        program.into_bytes(),
        Invocation::none(),
    );
    assert_eq!(
        String::from_utf8_lossy(&outcome.stderr),
        "",
        "the program must not fail"
    );
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "FF A FF\n9\n9\n20\n2\n1\n1\n1\n1\n"
    );
    assert_eq!(outcome.exit_code, 0);
}
