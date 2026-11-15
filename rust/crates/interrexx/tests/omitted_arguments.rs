//! An omitted argument at a built-in method: each documented instance
//! method of a class with a construction program in
//! `corpus/docs/class-set.txt` is sent argument lists with a hole in them,
//! `(,'a')`, `('a',)` and `(,,'a')`, and the error each raises, or `ok`, is
//! compared with the oracle's recorded answer. A method that checks its
//! arguments with its body rather than at the call raises whatever the body
//! happens to raise, which is what this table catches.
//!
//! One case file per class under `tests/omitted_arguments/`, named after the
//! class, whose `sends` case lists `method arguments` lines. The receiver
//! is the class's construction program, run afresh for every send. Some
//! construction programs answer a shared object (`.Class~package`,
//! `.Object~method('objectName')`, `.routines~r`, `.RexxInfo`), so a send
//! that succeeds and changes its receiver's state would reach the later
//! sends; the methods that would are left out ([`SKIPPED_METHODS`]).
//!
//! Recording: `OMITTED_ARGUMENTS_RECORD=1 REWRITE=1` runs the cases on the
//! oracle (`REXX_ORACLE_ROOT`), each with an `XDG_RUNTIME_DIR` of its own.

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use support::rexx_service;

const CASES: &str = "tests/omitted_arguments";

/// The argument lists every method is sent.
const PATTERNS: &[&str] = &["(,'a')", "('a',)", "(,,'a')"];

/// Classes with a construction program that are left out, each with why: their
/// methods touch files, run commands, start activities or wait, so a send with
/// a stray `'a'` could act on the machine or not return.
const SKIPPED_CLASSES: &[(&str, &str)] = &[
    ("Alarm", "its methods start and cancel timers"),
    (
        "CommandContext",
        "its methods act on a command that has ended",
    ),
    ("CommandHandler", "its methods run commands"),
    ("EventSemaphore", "its methods wait"),
    ("File", "its methods create, rename and delete files"),
    ("InputOutputStream", "its methods read and write files"),
    ("InputStream", "its methods read files"),
    ("Message", "its methods start activities and wait for them"),
    (
        "Monitor",
        "its methods forward to a destination that can be anything",
    ),
    ("MutexSemaphore", "its methods wait"),
    ("NativeCommandHandler", "its methods run commands"),
    ("OutputStream", "its methods write files"),
    ("RexxQueue", "its methods read and write the session queue"),
    ("Stream", "its methods read and write files"),
    ("StreamSupplier", "its methods read files"),
    ("Ticker", "its methods start and cancel timers"),
    (
        "WeakReference",
        "its referent can be collected between sends",
    ),
];

/// The rows left out of a probed class, each with why.
const SKIPPED_ROWS: &[(&str, &str, &str)] = &[
    (
        "DateTime",
        "('a',)",
        "Deviation 126: a comparison with a string is SYNTAX 88.914",
    ),
    (
        "TimeSpan",
        "('a',)",
        "Deviation 126: a comparison with a string is SYNTAX 88.914",
    ),
];

/// Methods left out of every probed class, each with why.
const SKIPPED_METHODS: &[(&str, &str)] = &[
    (
        "setSecurityManager",
        "the receivers that have it are shared objects (a package, a routine, a method), so \
         a manager it sets would intercept every later send",
    ),
    (
        "start",
        "a send it accepts starts an activity that runs on past the send and raises in its \
         own time, as the classes left out for starting activities would",
    ),
    (
        "startWith",
        "as `start`: a send it accepts starts an activity that runs on past the send",
    ),
];

/// The methods [`SKIPPED_ROWS`] applies to: `compareTo` and the comparison
/// operators.
const COMPARISONS: &[&str] = &[
    "compareTo",
    "=",
    "==",
    "<",
    "<=",
    "<<",
    "<<=",
    "<>",
    ">",
    ">=",
    "><",
    ">>",
    ">>=",
    "\\=",
    "\\==",
    "\\<",
    "\\<<",
    "\\>",
    "\\>>",
];

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/docs")
}

/// The rows of a tab-separated table, comments and blank lines left out.
fn rows(name: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(corpus().join(name))
        .expect("a docs table")
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| line.split('\t').map(str::to_string).collect())
        .collect()
}

/// Each probed class's construction program and directives: a class with a
/// construction program and a documented instance method, not skipped.
fn classes() -> BTreeMap<String, (String, String)> {
    let skipped: BTreeSet<&str> = SKIPPED_CLASSES.iter().map(|(class, _)| *class).collect();
    rows("class-set.txt")
        .into_iter()
        .filter(|row| row[4] == "covered" && !skipped.contains(row[0].as_str()))
        .filter(|row| !methods(&row[0]).is_empty())
        .map(|row| (row[0].clone(), (row[6].clone(), row[7].clone())))
        .collect()
}

/// The documented instance methods of `class`, as they are sent.
fn methods(class: &str) -> Vec<String> {
    rows("class-methods.txt")
        .into_iter()
        .filter(|row| row[0] == class && row[2] == "instance")
        .map(|row| row[1].replace(' ', ""))
        .collect()
}

fn skipped_row(class: &str, method: &str, pattern: &str) -> bool {
    SKIPPED_METHODS.iter().any(|(skip, _)| *skip == method)
        || COMPARISONS.contains(&method)
            && SKIPPED_ROWS
                .iter()
                .any(|(skip, arguments, _)| *skip == class && *arguments == pattern)
}

/// The program a `sends` case runs.
fn program(construction: &str, directives: &str, lines: &[&str]) -> String {
    let mut program = String::new();
    for line in lines {
        let (method, arguments) = line.split_once(' ').expect("method arguments");
        let method = method.replace('\'', "''");
        program.push_str(&format!("call t '{method}', \"{arguments}\"\n"));
    }
    let construction = construction.replace('\'', "''");
    program.push_str(&format!(
        "exit\nt: procedure\n  interpret 'o = {construction}'\n  signal on syntax name bad\n  \
         interpret 'o~\"'arg(1)'\"'arg(2)\n  say arg(1) arg(2) '=> ok'\n  return\n\
         bad: say arg(1) arg(2) '=>' condition('o')~code\n  return\n"
    ));
    if directives != "-" {
        program.push_str(directives);
        program.push('\n');
    }
    program
}

/// Runs `program` from a fresh directory, on the oracle where `oracle`, and
/// answers its stdout.
fn run(name: usize, program: &str, oracle: bool) -> String {
    let root = rexx_service::scratch("omitted_arguments");
    let dir = root.join(format!("case{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a case directory");
    std::fs::write(dir.join("probe.rex"), program).expect("the program");
    let api = rexx_service::service_dir(&root, name);
    let mut command = rexx_service::with_service(
        Command::new(if oracle {
            rexx_service::oracle_rexx()
        } else {
            rexx_service::own_rexx().to_path_buf()
        }),
        oracle,
        &api,
    );
    let output = command
        .arg("probe.rex")
        .current_dir(&dir)
        .stdin(Stdio::null())
        .output()
        .expect("rexx runs");
    if !oracle {
        rexx_service::stop_own_service(&api);
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn an_omitted_argument_raises_what_the_oracle_raises() {
    let record = std::env::var_os("OMITTED_ARGUMENTS_RECORD").is_some();
    let classes = classes();
    let mut cases = 0;
    datadriven::walk(CASES, |file| {
        let class = Path::new(&file.filename)
            .file_name()
            .and_then(|name| name.to_str())
            .expect("a case file named after its class")
            .to_string();
        let (construction, directives) = classes
            .get(&class)
            .unwrap_or_else(|| panic!("{class} is not a probed class"))
            .clone();
        file.run(|case| {
            cases += 1;
            assert_eq!(case.directive, "sends");
            let lines: Vec<&str> = case.input.lines().collect();
            run(cases, &program(&construction, &directives, &lines), record)
        });
    });
    assert!(cases > 0, "no case ran");
    rexx_service::remove_scratch("omitted_arguments");
}

/// The case files are exactly the probed classes, and each sends every
/// documented instance method every pattern, but for [`SKIPPED_ROWS`].
#[test]
fn the_cases_cover_every_method_of_every_probed_class() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(CASES);
    let mut files = BTreeSet::new();
    for entry in std::fs::read_dir(&dir).expect("the case directory") {
        let path = entry.expect("an entry").path();
        let class = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("a name")
            .to_string();
        let text = std::fs::read_to_string(&path).expect("a case file");
        // Every case's input, in order: each `sends` line opens one, and its
        // `----` closes it.
        let mut input = Vec::new();
        let mut lines = text.lines();
        while let Some(line) = lines.next() {
            if line.trim() == "sends" {
                input.extend(
                    lines
                        .by_ref()
                        .take_while(|line| *line != "----")
                        .map(str::to_string),
                );
            }
        }
        let expected: Vec<String> = methods(&class)
            .iter()
            .flat_map(|method| {
                PATTERNS
                    .iter()
                    .map(move |pattern| (method.clone(), *pattern))
            })
            .filter(|(method, pattern)| !skipped_row(&class, method, pattern))
            .map(|(method, pattern)| format!("{method} {pattern}"))
            .collect();
        assert_eq!(
            input, expected,
            "{class}: the sends are not every method and pattern"
        );
        files.insert(class);
    }
    let probed: BTreeSet<String> = classes().into_keys().collect();
    assert_eq!(files, probed, "the case files are not the probed classes");
}
