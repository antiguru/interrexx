//! What `rexx` takes each standard stream to be: a terminal or a pipe is
//! transient, a regular file persistent (`QUERY STREAMTYPE`), read from the
//! descriptor `rexx` was started with.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Writes each standard stream's type to standard error, which is a pipe in
/// every run below.
const PROGRAM: &str = "do s over .array~of('STDIN', 'STDOUT', 'STDERR')\n\
                       call lineout 'STDERR', s stream(s, 'C', 'QUERY STREAMTYPE')\n\
                       end\n";

fn probe_dir(label: &str, program: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "standard-stream-kind-{label}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("the probe directory");
    std::fs::write(dir.join("p.rex"), program).expect("the probe");
    dir
}

/// The type lines `rexx` writes with `stdin` and `stdout` as given.
fn types(dir: &Path, stdin: Stdio, stdout: Stdio) -> Vec<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_rexx"))
        .arg(dir.join("p.rex"))
        .current_dir(dir)
        .stdin(stdin)
        .stdout(stdout)
        .stderr(Stdio::piped())
        .output()
        .expect("rexx runs");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn regular_files_on_standard_input_and_output_are_persistent() {
    let dir = probe_dir("files", PROGRAM);
    let input = std::fs::File::open(dir.join("p.rex")).expect("the program as input");
    let output = std::fs::File::create(dir.join("out")).expect("an output file");
    let answered = types(&dir, input.into(), output.into());
    std::fs::remove_dir_all(&dir).expect("the probe directory is removed");
    assert_eq!(
        answered,
        ["STDIN PERSISTENT", "STDOUT PERSISTENT", "STDERR TRANSIENT"]
    );
}

#[test]
fn pipes_on_standard_input_and_output_are_transient() {
    let dir = probe_dir("pipes", PROGRAM);
    let answered = types(&dir, Stdio::piped(), Stdio::piped());
    std::fs::remove_dir_all(&dir).expect("the probe directory is removed");
    assert_eq!(
        answered,
        ["STDIN TRANSIENT", "STDOUT TRANSIENT", "STDERR TRANSIENT"]
    );
}

/// spec: bif.stream.handle
#[test]
fn open_standard_handles_report_the_sizes_of_redirected_regular_files() {
    let program = "sizes = ''\n\
                   do n = 0 to 2\n\
                   handle = .Stream~new('HANDLE:' || n)\n\
                   if n = 0 then opened = handle~open('read')\n\
                   else opened = handle~open('write')\n\
                   sizes = space(sizes handle~query('size'))\n\
                   end\n\
                   call lineout 'sizes', sizes\n\
                   call lineout 'sizes'\n";
    let dir = probe_dir("handle-sizes", program);
    std::fs::write(dir.join("in"), b"1234567").expect("a seven-byte input file");
    std::fs::write(dir.join("out"), b"12345").expect("a five-byte output file");
    std::fs::write(dir.join("err"), b"123").expect("a three-byte error file");
    let input = std::fs::File::open(dir.join("in")).expect("redirected input");
    let append = |file| {
        std::fs::OpenOptions::new()
            .append(true)
            .open(dir.join(file))
            .expect("a writable redirection")
    };
    let status = Command::new(env!("CARGO_BIN_EXE_rexx"))
        .arg(dir.join("p.rex"))
        .current_dir(&dir)
        .stdin(input)
        .stdout(append("out"))
        .stderr(append("err"))
        .status()
        .expect("rexx runs");
    let sizes = std::fs::read_to_string(dir.join("sizes"));
    let error = std::fs::read_to_string(dir.join("err")).expect("redirected error");
    std::fs::remove_dir_all(&dir).expect("the probe directory is removed");
    assert!(status.success(), "{status}: {error}");
    assert_eq!(
        sizes.expect("the probe wrote its answer").trim_end(),
        "7 5 3"
    );
}
