//! `HANDLE:n` streams (`r[bif.stream.handle]`), run on this crate alone with
//! the standard streams an `Invocation` gives, which no specification example
//! can arrange. `corpus/lang/stream_handle_names.rex` holds more shapes.

use interrexx::{Invocation, Outcome, ProgramInput, run_program};

fn run(program: &str, invocation: Invocation) -> Outcome {
    run_program("stream_handle.rex", program.as_bytes().to_vec(), invocation)
}

fn stdout(outcome: &Outcome) -> String {
    String::from_utf8_lossy(&outcome.stdout).into_owned()
}

/// spec: bif.stream.handle
///
/// `HANDLE:0` reads standard input.
#[test]
fn handle_zero_reads_standard_input() {
    let program = "say linein('HANDLE:0')\n\
                   s = .Stream~new('HANDLE:0')\n\
                   say s~open('read') s~linein s~state\n";
    let outcome = run(
        program,
        Invocation::none().with_input(ProgramInput::Bytes(b"first\nsecond\n".to_vec())),
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(stdout(&outcome), "first\nREADY: second READY\n");
}

/// spec: bif.stream.handle
///
/// `HANDLE:1` and `HANDLE:2` write to standard output and standard error.
#[test]
fn handle_one_and_two_write_standard_output_and_error() {
    let program = "call lineout 'HANDLE:1', 'to out'\n\
                   call lineout 'HANDLE:2', 'to err'\n\
                   s = .Stream~new('handle:1')\n\
                   say s~charout('chars') s~state\n";
    let outcome = run(program, Invocation::none());
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(stdout(&outcome), "to out\nchars0 READY\n");
    assert_eq!(String::from_utf8_lossy(&outcome.stderr), "to err\n");
}

/// An opened `HANDLE:n` answers its descriptor for `QUERY HANDLE`, and an
/// unopened one the null string.
#[test]
fn query_handle_after_open_answers_the_descriptor() {
    let program = "do n = 0 to 2\n\
                   s = .Stream~new('HANDLE:' || n)\n\
                   before = s~query('handle')\n\
                   opened = s~open('read')\n\
                   say '[' || before || ']' s~query('handle')\n\
                   end\n";
    let outcome = run(program, Invocation::none());
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(stdout(&outcome), "[] 0\n[] 1\n[] 2\n");
}

/// A second open of a `HANDLE:n` stream opens the descriptor again.
#[test]
fn a_second_open_opens_the_descriptor_again() {
    let program = "s = .Stream~new('HANDLE:0')\n\
                   say s~open('read') s~close s~open('read') s~state\n";
    let outcome = run(program, Invocation::none());
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(stdout(&outcome), "READY: READY: READY: READY\n");
}

/// An opened `HANDLE:n`'s stream type is the one the `Invocation` reports for
/// that standard stream. Unopened it is `UNKNOWN`, as on the oracle.
#[test]
fn the_stream_type_follows_the_invocation_once_opened() {
    let program = "do n = 0 to 2\n\
                   s = .Stream~new('HANDLE:' || n)\n\
                   before = s~query('streamtype')\n\
                   opened = s~open('read')\n\
                   say before s~query('streamtype')\n\
                   end\n";
    let outcome = run(
        program,
        Invocation::none().with_standard_transient([false, true, false]),
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        stdout(&outcome),
        "UNKNOWN PERSISTENT\nUNKNOWN TRANSIENT\nUNKNOWN PERSISTENT\n"
    );
}

/// spec: bif.stream.handle
///
/// `HANDLE:n` for a descriptor another stream holds writes through it, and
/// closing the `HANDLE:` stream leaves the descriptor open: a second one on the
/// same number opens and writes. Nothing exists by such a name.
#[test]
fn another_descriptor_is_written_and_stays_open_after_a_close() {
    let dir = std::env::temp_dir().join(format!("interrexx-handle-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let file = dir.join("data.txt");
    let program = format!(
        "file = .Stream~new('{}')\n\
         say file~open('write replace')\n\
         h = file~query('handle')\n\
         first = .Stream~new('HANDLE:' || h)\n\
         say first~lineout('first') '[' || first~query('exists') || ']' first~close\n\
         second = .Stream~new('HANDLE:' || h)\n\
         say second~open('write') second~close\n\
         say file~close\n",
        file.display()
    );
    let outcome = run(&program, Invocation::none());
    let written = std::fs::read_to_string(&file).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        stdout(&outcome),
        "READY:\n0 [] READY:\nREADY: READY:\nREADY:\n"
    );
    assert_eq!(written, "first\n");
}

/// An `APPEND` stream opened on a write-only file still writes at the end of
/// it, past what a command appended meanwhile, where the oracle drops the
/// append flag and overwrites that text.
#[test]
fn an_append_stream_on_a_write_only_file_writes_at_the_end() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("rexx-append-wo-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a private directory");
    let file = dir.join("wo.txt");
    std::fs::write(&file, "y\n").expect("the file is writable");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o222)).expect("chmod 222");
    let path = file.display();
    let program = format!(
        "w = .stream~new('{path}'); say w~open('write append')\n\
         say w~lineout('w1')\n\
         address system \"echo wcmd >> '{path}'\"\n\
         say w~lineout('w2'); say w~close\n"
    );
    let outcome = run(&program, Invocation::none());
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).expect("chmod 644");
    let held = std::fs::read(&file).expect("the file reads back");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(stdout(&outcome), "READY:\n0\n0\nREADY:\n");
    assert_eq!(String::from_utf8_lossy(&held), "y\nwcmd\nw1\nw2\n");
}

/// An `APPEND` stream's write position cannot move into the file as it was
/// opened, on a file larger than 4 GiB too.
#[test]
fn an_append_stream_refuses_positions_within_a_file_over_four_gib() {
    let dir = std::env::temp_dir().join(format!("rexx-append-big-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a private directory");
    let file = dir.join("big.dat");
    std::fs::File::create(&file)
        .and_then(|created| created.set_len(5_368_709_120))
        .expect("a sparse file");
    let path = file.display();
    let program = format!(
        "s = .stream~new('{path}'); say s~open('write append')\n\
         say s~seek('=4294967290 write char') s~state\n\
         say s~seek('=4294967296 write char') s~state\n\
         say s~seek('=5368709120 write char') s~state\n\
         say s~charout('x', 5000000000) s~state\n\
         say s~lineout('L') s~state\n\
         say s~seek('=5368709122 write char') s~state\n\
         say s~query('size'); s~close\n"
    );
    let outcome = run(&program, Invocation::none());
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        stdout(&outcome),
        "READY:\n0 NOTREADY\n0 NOTREADY\n0 NOTREADY\n1 NOTREADY\n0 READY\n5368709122 READY\n\
         5368709122\n"
    );
}
