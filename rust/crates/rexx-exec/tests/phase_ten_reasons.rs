//! **A refusal naming Phase 10 carries a reason from D3's closed set**
//! (spec `2026-10-10-phase-9-core-conformance.md` D3).
//!
//! The reason is a `rexx_core::Deferral` inside `rexx_core::Owner::Phase10`.
//! No string literal in the production sources of the crates
//! `closed_phases.rs` scans for refusals (`rexx-exec`, `rexx-api`) may name
//! Phase 10: one that does is a phase name that reached a refusal without a
//! reason. Test modules hold vocabularies and are out of scope.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "support/literals.rs"]
mod source_literals;

use source_literals::{literals, names, shown, source_files};

/// The crates whose refusals the scan covers.
const SCANNED: &[&str] = &["rexx-exec", "rexx-api"];

/// `text` with the body of every inline `#[cfg(test)] mod NAME {` blanked,
/// line breaks kept so line numbers hold. The block ends at the `}` on its
/// own line at the `mod` line's indent, as rustfmt writes it.
fn without_test_modules(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::with_capacity(lines.len());
    let mut at = 0;
    while at < lines.len() {
        if lines[at].trim() == "#[cfg(test)]" {
            let mut opener = at + 1;
            while opener < lines.len() && lines[opener].trim_start().starts_with("#[") {
                opener += 1;
            }
            if let Some(line) = lines.get(opener)
                && module_name(line).is_some()
                && line.trim_end().ends_with('{')
            {
                let indent = &line[..line.len() - line.trim_start().len()];
                let closing = format!("{indent}}}");
                let mut close = opener;
                while close < lines.len() && lines[close] != closing {
                    close += 1;
                }
                assert!(
                    close < lines.len(),
                    "the test module at line {} never closes",
                    at + 1
                );
                out.extend(std::iter::repeat_n("", close + 1 - at));
                at = close + 1;
                continue;
            }
        }
        out.push(lines[at]);
        at += 1;
    }
    out.join("\n")
}

/// The module `line` declares, inline or as a file.
fn module_name(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let line = line
        .strip_prefix("pub(crate) ")
        .or_else(|| line.strip_prefix("pub(super) "))
        .or_else(|| line.strip_prefix("pub "))
        .unwrap_or(line);
    let rest = line.strip_prefix("mod ")?;
    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    (end > 0).then(|| &rest[..end])
}

/// The files and directories the `#[cfg(test)] mod NAME;` declarations in
/// `path` load.
fn test_module_paths(path: &Path, text: &str) -> Vec<PathBuf> {
    let directory = path.parent().expect("a file in a directory");
    let stem = path.file_stem().and_then(|s| s.to_str()).expect("a name");
    let base = if matches!(stem, "lib" | "main" | "mod") {
        directory.to_path_buf()
    } else {
        directory.join(stem)
    };
    let lines: Vec<&str> = text.lines().collect();
    let mut found = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        if line.trim() != "#[cfg(test)]" {
            continue;
        }
        let mut opener = at + 1;
        while opener < lines.len() && lines[opener].trim_start().starts_with("#[") {
            opener += 1;
        }
        let Some(declared) = lines.get(opener) else {
            continue;
        };
        if let Some(name) = module_name(declared)
            && declared.trim_end().ends_with(';')
        {
            found.push(base.join(format!("{name}.rs")));
            found.push(base.join(name));
        }
    }
    found
}

/// The production sources of `crate_dir`: every file under `src/` but a
/// `tests.rs` and the files a `#[cfg(test)] mod NAME;` loads, as each path
/// and its text with inline test modules blanked.
fn production_sources(crate_dir: &str) -> Vec<(PathBuf, String)> {
    let files = source_files(crate_dir);
    let mut excluded: Vec<PathBuf> = Vec::new();
    for path in &files {
        let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        excluded.extend(test_module_paths(path, &text));
    }
    files
        .into_iter()
        .filter(|path| {
            path.file_name().is_none_or(|name| name != "tests.rs")
                && !excluded.iter().any(|test| path.starts_with(test))
        })
        .map(|path| {
            let text = fs::read_to_string(&path).expect("a source");
            let kept = without_test_modules(&text);
            (path, kept)
        })
        .collect()
}

/// Every production literal of `crate_dir` naming Phase 10, as `file:line`.
fn phase_ten_literals(crate_dir: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (path, text) in production_sources(crate_dir) {
        for (line, literal) in literals(&text) {
            if names(&literal, "Phase 10") {
                found.push(format!("{crate_dir}/{}:{line}: {literal:?}", shown(&path)));
            }
        }
    }
    found
}

/// The whole claim.
#[test]
fn no_production_literal_names_phase_10() {
    let found: Vec<String> = SCANNED
        .iter()
        .flat_map(|crate_dir| phase_ten_literals(crate_dir))
        .collect();
    assert!(
        found.is_empty(),
        "these literals name Phase 10 without a reason; build the owner as \
         `Owner::Phase10(Deferral::...)` and pass it to the constructor:\n{found:#?}"
    );
}

/// The negative control: the same walk over every file, test modules
/// included, finds the vocabularies the claim leaves out, so an empty result
/// above is not a scan of nothing.
///
/// No literal of this crate names Phase 10 any more, test modules included, so
/// the vocabularies are a fixture walked as a crate's `src/`
/// (`tests/phase_ten_reasons/src/`): Phase 10 named in an inline test module
/// and in a `#[cfg(test)] mod tests;` file, found by the walk of every file and
/// left out by the production walk.
#[test]
fn the_scan_reads_the_test_modules_it_leaves_out() {
    const FIXTURE: &str = "rexx-exec/tests/phase_ten_reasons";
    let everywhere = source_files(FIXTURE)
        .iter()
        .flat_map(|path| literals(&fs::read_to_string(path).expect("a source")))
        .filter(|(_, literal)| names(literal, "Phase 10"))
        .count();
    assert_eq!(
        everywhere, 2,
        "the walk of every file did not find the fixture's two test-module literals"
    );
    assert_eq!(phase_ten_literals(FIXTURE), Vec::<String>::new());
    let fixture_kept: Vec<String> = production_sources(FIXTURE)
        .iter()
        .map(|(path, _)| shown(path))
        .collect();
    assert_eq!(fixture_kept.len(), 1, "{fixture_kept:?}");
    let kept: BTreeSet<String> = production_sources("rexx-exec")
        .iter()
        .map(|(path, _)| shown(path))
        .collect();
    for test in [
        "src/dispatch/tests.rs",
        "src/run/tests/directives.rs",
        "src/scheduler/tests/native.rs",
        "src/ir/golden_tests.rs",
    ] {
        assert!(!kept.contains(test), "{test} is a test module");
    }
    for production in ["src/lib.rs", "src/dispatch/native.rs", "src/environment.rs"] {
        assert!(kept.contains(production), "{production} was left out");
    }
    let (_, lib) = production_sources("rexx-exec")
        .into_iter()
        .find(|(path, _)| shown(path) == "src/lib.rs")
        .expect("lib.rs is kept");
    assert!(
        literals(&lib)
            .iter()
            .any(|(_, literal)| literal
                == "a STRING method answering an object with no string value"),
        "lib.rs's production text was blanked"
    );
}

/// An inline test module is blanked to its closing brace and nothing past
/// it; a literal before or after it is still read, on its own line.
#[test]
fn an_inline_test_module_is_left_out_and_its_neighbours_are_not() {
    let source = "\
fn a() -> &'static str { \"Phase 10 before\" }
#[cfg(test)]
#[allow(dead_code)]
mod tests {
    fn b() -> &'static str { \"Phase 10 inside\" }
    mod inner {
    }
}
fn c() -> &'static str { \"Phase 10 after\" }
#[cfg(test)]
fn d() -> &'static str { \"Phase 10 test fn\" }
";
    let named: Vec<(usize, String)> = literals(&without_test_modules(source))
        .into_iter()
        .filter(|(_, literal)| names(literal, "Phase 10"))
        .collect();
    assert_eq!(
        named,
        [
            (1, "Phase 10 before".to_string()),
            (9, "Phase 10 after".to_string()),
            (11, "Phase 10 test fn".to_string()),
        ]
    );
}

/// A `#[cfg(test)] mod NAME;` resolves beside a `mod.rs` or `lib.rs`, and
/// below the declaring file's own directory otherwise.
#[test]
fn a_test_module_file_resolves_where_rustc_finds_it() {
    let text = "#[cfg(test)]\nmod tests;\n#[cfg(test)]\n#[allow(dead_code)]\npub(crate) mod fake;\nmod live;\n";
    assert_eq!(
        test_module_paths(Path::new("/c/src/lib.rs"), text),
        [
            PathBuf::from("/c/src/tests.rs"),
            PathBuf::from("/c/src/tests"),
            PathBuf::from("/c/src/fake.rs"),
            PathBuf::from("/c/src/fake"),
        ]
    );
    assert_eq!(
        test_module_paths(
            Path::new("/c/src/ir.rs"),
            "#[cfg(test)]\nmod golden_tests;\n"
        ),
        [
            PathBuf::from("/c/src/ir/golden_tests.rs"),
            PathBuf::from("/c/src/ir/golden_tests"),
        ]
    );
}
