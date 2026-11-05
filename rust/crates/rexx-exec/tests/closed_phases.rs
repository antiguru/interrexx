//! **No refusal names a phase that has closed.**
//!
//! A loud refusal ends with the phase that owes the work, and a program can
//! read it. Naming a closed phase tells a reader to wait for something that
//! has already happened, so a phase's close-out has to move every refusal it
//! leaves behind to whoever actually owes it -- and the ones it does not move
//! are the ones nobody looked at.
//!
//! The scan is over the crate's own sources rather than over a table,
//! because an owner reaches a refusal three ways: as a row in a table, as a
//! literal argument to a `Loud` constructor, and as a match arm. Only the
//! first is enumerable. A literal is read whole, so an owner inside a longer
//! refusal text (`"... is not implemented (Phase 8)"`) is found too. The
//! owner tables in `tests/` are scanned the same way.

use std::fs;
use std::path::{Path, PathBuf};

#[path = "support/literals.rs"]
mod source_literals;

use source_literals::{literals, names, shown, source_files};

/// The phases whose work is finished and which this assertion covers.
const CLOSED: &[&str] = &[
    "Phase 5", "Phase 6", "Phase 7", "Phase 8", "Phase 9", "Phase 10",
];

/// This crate's test files holding owner tables: the instruction and
/// expression tags, the assertion harnesses' exempt rows and the phase
/// vocabularies their attributions are checked against.
const OWNER_TABLES: &[&str] = &[
    "owners.rs",
    "assertions.rs",
    "bif_assertions.rs",
    "keyword_assertions.rs",
];

/// Every string literal naming a closed phase, alone (`"Phase 8"`) or
/// inside a longer text (`"... not implemented (Phase 8)"`), which is what a
/// refusal is built from.
fn mentions(crate_dir: &str) -> Vec<String> {
    mentions_in(source_files(crate_dir))
}

/// [`mentions`] over `paths`.
fn mentions_in(paths: Vec<PathBuf>) -> Vec<String> {
    let mut found = Vec::new();
    for path in paths {
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for (line, literal) in literals(&text) {
            for phase in CLOSED {
                if names(&literal, phase) {
                    found.push(format!("{}:{line}: {phase}", shown(&path)));
                }
            }
        }
    }
    found
}

/// The whole claim, over every crate a refusal can come from.
#[test]
fn no_refusal_names_a_closed_phase() {
    for crate_dir in [
        "rexx-exec",
        "rexx-parse",
        "rexx-core",
        "rexx-inventory",
        "rexx-api",
    ] {
        let named = mentions(crate_dir);
        assert!(
            named.is_empty(),
            "these sites name a phase that has closed, so the refusal they build \
             tells a reader to wait for work that is already done:\n{named:#?}"
        );
    }
}

/// The same claim over the owner tables in this crate's `tests/`, which name
/// the phase a row or a tag waits for.
#[test]
fn no_owner_table_names_a_closed_phase() {
    let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let paths: Vec<PathBuf> = OWNER_TABLES.iter().map(|name| tests.join(name)).collect();
    for path in &paths {
        assert!(path.is_file(), "{} is not a file", path.display());
    }
    let named = mentions_in(paths);
    assert!(
        named.is_empty(),
        "these owner-table entries name a phase that has closed:\n{named:#?}"
    );
}

/// The negative control for the scan: a phase that is still open is found by
/// the same walk, so an empty result above means the sites are gone and not
/// that the scan is looking in the wrong place. No source of this crate names
/// an open phase, so the witness is a fixture walked as a crate's `src/`
/// (`tests/closed_phases/src/`), beside a check that the walk of this crate
/// reaches its sources and reads their literals.
#[test]
fn the_scan_finds_an_open_phase_it_is_not_asked_about() {
    let open = source_files("rexx-exec/tests/closed_phases")
        .iter()
        .flat_map(|path| literals(&fs::read_to_string(path).expect("a source")))
        .filter(|(_, literal)| names(literal, "Phase 11"))
        .count();
    assert!(
        open >= 2,
        "the scan found {open} refusals naming a phase that is still open in the \
         fixture, so it does not read what the other test clears"
    );
    let sources = source_files("rexx-exec");
    assert!(
        sources.iter().any(|path| shown(path) == "src/lib.rs"),
        "the walk of rexx-exec did not reach src/lib.rs"
    );
    let read = sources
        .iter()
        .flat_map(|path| literals(&fs::read_to_string(path).expect("a source")))
        .count();
    assert!(read > 0, "the walk of rexx-exec read no literal");
}

/// The literal scan reads a phase inside a longer refusal text, across a
/// continued line, and not in a comment, a character or a lifetime.
#[test]
fn the_literal_scan_finds_a_phase_inside_a_longer_text() {
    let source = r##"fn f<'a>(x: &'a str) -> char {
// "not (Phase 8)"
/* "(Phase 8)" */
let q = '"'; let e = '\''; let s = "a \"b\" (Phase 8)";
let t = "x is \
      not implemented (Phase 8)";
let r = r#"raw "q" Phase 8"#; let u = "Phase 80";
}
"##;
    let named: Vec<usize> = literals(source)
        .into_iter()
        .filter(|(_, literal)| names(literal, "Phase 8"))
        .map(|(line, _)| line)
        .collect();
    assert_eq!(named, [4, 5, 7]);
    let later_phase = literals(r#"let phase = "not implemented (Phase 9.1)";"#);
    assert!(!names(&later_phase[0].1, "Phase 9"));
    assert!(names("owned by Phase 9.", "Phase 9"));
    let found = mentions("rexx-api")
        .into_iter()
        .filter(|site| site.starts_with("src/ffi.rs:"))
        .count();
    assert_eq!(found, 0, "rexx-api's ffi.rs names no closed phase");
    let read = source_files("rexx-api")
        .iter()
        .filter(|path| shown(path) == "src/ffi.rs")
        .flat_map(|path| literals(&fs::read_to_string(path).expect("a source")))
        .filter(|(_, literal)| literal.starts_with("RexxThreadInterface."))
        .count();
    assert!(
        read >= 1,
        "the scan read no literal naming a thread-table member, which ffi.rs \
         holds, so it is not reading the file it clears"
    );
}
