//! **Every refusal has a committed disposition.**
//!
//! No refusal names a phase that owes its work. So this file reads every `Loud` constructor in `src/`
//! and requires each one to have a row in `corpus/refusal-dispositions.tsv`,
//! saying why no phase owes it: GUARD (no program reaches it; the record names
//! the probes that tried), DEVIATION (matching the oracle is wrong; the record
//! cites an `oracle-crashes.txt` entry or an active Deviation), or LIMIT (a
//! design limit; the record cites the deviation or the design document that
//! licenses it). A `Loud` struct literal outside a constructor has no name to
//! key a row on, so it is refused.

#[path = "support/deviations.rs"]
mod deviations;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// The three dispositions.
const KINDS: &[&str] = &["GUARD", "DEVIATION", "LIMIT"];

/// One committed row.
#[derive(Debug)]
struct Row {
    constructor: String,
    kind: String,
    phase: String,
    record: String,
}

fn rust_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the crate sits two levels below `rust/`")
        .to_path_buf()
}

fn table_path() -> PathBuf {
    rust_root().join("corpus/refusal-dispositions.tsv")
}

fn parse_table(text: &str) -> Vec<Row> {
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(
                fields.len(),
                4,
                "four tab-separated fields (constructor, kind, phase, record) in {line:?}"
            );
            Row {
                constructor: fields[0].to_string(),
                kind: fields[1].to_string(),
                phase: fields[2].to_string(),
                record: fields[3].to_string(),
            }
        })
        .collect()
}

/// Every non-test `.rs` file under `src/`, with `#[cfg(test)] mod` blocks
/// dropped and line comments stripped, as `(line number, code)` pairs.
fn sources() -> Vec<(String, Vec<(usize, String)>)> {
    let mut found = Vec::new();
    let mut pending = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).expect("a directory this crate compiles") {
            let path = entry.expect("a readable directory entry").path();
            let shown = path.display().to_string();
            if path.is_dir() {
                if !shown.ends_with("/tests") {
                    pending.push(path);
                }
            } else if shown.ends_with(".rs")
                && !shown.ends_with("/tests.rs")
                && !shown.ends_with("_tests.rs")
            {
                let text = fs::read_to_string(&path).expect("a source file this crate compiles");
                let relative = path
                    .strip_prefix(rust_root())
                    .expect("a path under rust/")
                    .display()
                    .to_string();
                found.push((relative, code_lines(&text)));
            }
        }
    }
    found.sort();
    found
}

fn code_lines(text: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut at = 0;
    while at < lines.len() {
        if lines[at].starts_with("#[cfg(test)]") {
            let mut opener = at + 1;
            while opener < lines.len() && lines[opener].starts_with("#[") {
                opener += 1;
            }
            if opener < lines.len()
                && lines[opener].starts_with("mod ")
                && lines[opener].trim_end().ends_with('{')
            {
                let mut close = opener + 1;
                while close < lines.len() && lines[close] != "}" {
                    close += 1;
                }
                at = close + 1;
                continue;
            }
        }
        let code = match lines[at].find("//") {
            Some(cut) => &lines[at][..cut],
            None => lines[at],
        };
        out.push((at + 1, code.to_string()));
        at += 1;
    }
    out
}

/// A `Loud` constructor: its name, where it is and its body.
struct Constructor {
    name: String,
    site: String,
    body: String,
}

/// Whether the line at `index` of `lines` sits inside an `impl Loud` block,
/// for each line.
fn in_impl_loud(lines: &[(usize, String)]) -> Vec<bool> {
    let mut inside = false;
    lines
        .iter()
        .map(|(_, code)| {
            if code.starts_with("impl Loud") {
                inside = true;
            } else if code == "}" {
                inside = false;
            }
            inside
        })
        .collect()
}

/// Every `fn NAME(...) -> Loud`, and `-> Self` inside `impl Loud`, in
/// `files`, with the body up to the closing brace at the `fn` line's indent.
fn constructors(files: &[(String, Vec<(usize, String)>)]) -> Vec<Constructor> {
    let mut found = Vec::new();
    for (path, lines) in files {
        let impl_loud = in_impl_loud(lines);
        for (index, (number, code)) in lines.iter().enumerate() {
            let Some(at) = code.find("fn ") else {
                continue;
            };
            if at > 0 && code.as_bytes()[at - 1].is_ascii_alphanumeric() {
                continue;
            }
            // The signature may wrap; join lines until the body opens.
            let mut signature = code[at + 3..].to_string();
            let mut end = index;
            while !signature.contains('{') && !signature.contains(';') && end + 1 < lines.len() {
                end += 1;
                signature.push(' ');
                signature.push_str(lines[end].1.trim());
            }
            let Some(open) = signature.find('(') else {
                continue;
            };
            let name = signature[..open].trim().to_string();
            if name.is_empty() || !name.bytes().all(|b| b == b'_' || b.is_ascii_alphanumeric()) {
                continue;
            }
            let Some(close) = signature.rfind(')') else {
                continue;
            };
            let returns = signature[close + 1..].trim_start();
            let Some(returns) = returns.strip_prefix("->") else {
                continue;
            };
            let returns = returns.trim_start();
            let names = |kind: &str| {
                returns.starts_with(kind)
                    && !returns
                        .as_bytes()
                        .get(kind.len())
                        .is_some_and(|b| *b == b'_' || b.is_ascii_alphanumeric())
            };
            if !(names("Loud") || (impl_loud[index] && names("Self"))) {
                continue;
            }
            let indent = code.len() - code.trim_start().len();
            let closing = format!("{}}}", " ".repeat(indent));
            let mut stop = end + 1;
            while stop < lines.len() && lines[stop].1.trim_end() != closing {
                stop += 1;
            }
            let body = lines[index..stop.min(lines.len())]
                .iter()
                .map(|(_, code)| code.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            found.push(Constructor {
                name,
                site: format!("{path}:{number}"),
                body,
            });
        }
    }
    found
}

/// The site of every `Loud { ... }` struct literal outside a constructor,
/// and `Self { ... }` inside `impl Loud`.
fn literals(files: &[(String, Vec<(usize, String)>)], constructors: &[Constructor]) -> Vec<String> {
    let mut found = Vec::new();
    for (path, lines) in files {
        let impl_loud = in_impl_loud(lines);
        for (index, (number, code)) in lines.iter().enumerate() {
            if code.contains("struct Loud")
                || code.contains("impl Loud")
                || code.contains("-> Loud")
                || code.contains("-> Self")
            {
                continue;
            }
            let opener = ["Loud {", "Self {"]
                .iter()
                .filter(|opener| **opener == "Loud {" || impl_loud[index])
                .filter_map(|opener| code.find(opener))
                .find(|&at| at == 0 || !code.as_bytes()[at - 1].is_ascii_alphanumeric());
            if opener.is_none() {
                continue;
            }
            let inside = constructors.iter().any(|ctor| {
                let Some((file, line)) = ctor.site.rsplit_once(':') else {
                    return false;
                };
                let line: usize = line.parse().expect("a line number");
                let length = ctor.body.lines().count();
                file == path && (line..line + length).contains(number)
            });
            if !inside {
                found.push(format!("{path}:{number}"));
            }
        }
    }
    found
}

/// Every constructor's site, by name.
fn sites(constructors: &[Constructor]) -> BTreeMap<String, String> {
    constructors
        .iter()
        .map(|ctor| (ctor.name.clone(), ctor.site.clone()))
        .collect()
}

/// Whether `record` cites an `oracle-crashes.txt` entry that exists, and the
/// citations that do not.
fn crash_citations(record: &str, crashes: &str) -> (bool, Vec<String>) {
    citations(record, "oracle-crashes.txt entry ", |number| {
        crashes.contains(&format!("\n## {number}. "))
    })
}

/// Whether `record` cites a Deviation, and the citations that are not one
/// of the `active` numbers.
fn deviation_citations(record: &str, active: &BTreeSet<u32>) -> (bool, Vec<String>) {
    citations(record, "Deviation ", |number| {
        number.parse().is_ok_and(|n: u32| active.contains(&n))
    })
}

fn citations(record: &str, marker: &str, exists: impl Fn(&str) -> bool) -> (bool, Vec<String>) {
    let mut any = false;
    let mut missing = Vec::new();
    for (at, _) in record.match_indices(marker) {
        let number: String = record[at + marker.len()..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();
        if number.is_empty() || !number.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        any = true;
        if !exists(&number) {
            missing.push(format!("{marker}{number}"));
        }
    }
    (any, missing)
}

/// The repository paths `record` names (`docs/...`, `rust/...`), each with
/// any `:line` suffix dropped, that do not exist.
fn missing_paths(record: &str, repository: &Path) -> (bool, Vec<String>) {
    let mut any = false;
    let mut missing = Vec::new();
    for word in record.split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '`') {
        if !(word.starts_with("docs/") || word.starts_with("rust/")) {
            continue;
        }
        let path = word
            .split(':')
            .next()
            .unwrap_or(word)
            .trim_end_matches(['.', ')']);
        any = true;
        if !repository.join(path).exists() {
            missing.push(path.to_string());
        }
    }
    (any, missing)
}

/// Everything wrong with `rows` against the constructors `sites` derives
/// from the source and the `literals` outside a constructor.
fn problems(
    sites: &BTreeMap<String, String>,
    literals: &[String],
    rows: &[Row],
    documents: &Documents,
) -> Vec<String> {
    let mut found = Vec::new();
    for site in literals {
        found.push(format!(
            "{site}: a `Loud` struct literal; make it a constructor so it can carry a \
             disposition row"
        ));
    }
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for row in rows {
        *seen.entry(row.constructor.as_str()).or_default() += 1;
        if !KINDS.contains(&row.kind.as_str()) {
            found.push(format!(
                "{}: kind {:?} is not one of {KINDS:?}",
                row.constructor, row.kind
            ));
            continue;
        }
        if !sites.contains_key(&row.constructor) {
            found.push(format!(
                "{}: a row with no constructor in src/",
                row.constructor
            ));
            continue;
        }
        if row.record.trim().is_empty() || row.record == "-" {
            found.push(format!("{}: a row with no record", row.constructor));
        }
        if row.phase != "-" {
            found.push(format!(
                "{}: phase {:?} on a {} row; no refusal names a phase",
                row.constructor, row.phase, row.kind
            ));
        }
        let (crash, bad_crash) = crash_citations(&row.record, &documents.crashes);
        let (deviation, bad_deviation) = deviation_citations(&row.record, &documents.deviations);
        let (path, bad_path) = missing_paths(&row.record, &documents.repository);
        for bad in bad_crash.iter().chain(&bad_deviation).chain(&bad_path) {
            found.push(format!(
                "{}: cites {bad}, which does not exist",
                row.constructor
            ));
        }
        match row.kind.as_str() {
            "DEVIATION" if !(crash || deviation) => found.push(format!(
                "{}: a DEVIATION cites an oracle-crashes.txt entry or a Deviation",
                row.constructor
            )),
            "LIMIT" if !(deviation || path) => found.push(format!(
                "{}: a LIMIT cites the Deviation or the design document that licenses it",
                row.constructor
            )),
            _ => {}
        }
    }
    for (name, count) in seen {
        if count > 1 {
            found.push(format!("{name}: {count} rows"));
        }
    }
    for (name, site) in sites {
        if !rows.iter().any(|row| row.constructor == *name) {
            found.push(format!("{name} ({site}): no disposition row"));
        }
    }
    found
}

/// The files a record's citations are checked against.
struct Documents {
    crashes: String,
    /// The active Deviations' numbers.
    deviations: BTreeSet<u32>,
    repository: PathBuf,
}

fn documents() -> Documents {
    let repository = rust_root()
        .parent()
        .expect("rust/ sits in the repository")
        .to_path_buf();
    let read = |path: &str| {
        fs::read_to_string(repository.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
    };
    Documents {
        crashes: read("rust/corpus/oracle-crashes.txt"),
        deviations: active_deviations(&read(deviations::TABLE)),
        repository,
    }
}

/// The active numbers of a Deviations table's `text`.
fn active_deviations(text: &str) -> BTreeSet<u32> {
    deviations::parse(text)
        .unwrap_or_else(|e| panic!("{}: {e}", deviations::TABLE))
        .into_iter()
        .filter(|(_, d)| d.active)
        .map(|(n, _)| n)
        .collect()
}

/// The whole claim.
#[test]
fn every_refusal_has_a_disposition() {
    let files = sources();
    let constructors = constructors(&files);
    let literals = literals(&files, &constructors);
    let text = fs::read_to_string(table_path()).unwrap_or_default();
    let found = problems(
        &sites(&constructors),
        &literals,
        &parse_table(&text),
        &documents(),
    );
    assert!(
        found.is_empty(),
        "corpus/refusal-dispositions.tsv and src/ disagree:\n{}",
        found.join("\n")
    );
}

/// The scanner finds the constructors it has to, so an empty problem list is
/// a measurement and not a scan of nothing.
#[test]
fn the_scanner_finds_each_constructor_form() {
    // A constructor outside lib.rs, and a `-> Self` one inside `impl Loud`,
    // from a fixture.
    let fixture = "\
impl Loud {
    fn deferred_send(name: &[u8]) -> Loud {
        let shown = String::from_utf8_lossy(name).into_owned();
        Loud {
            message: owned_message(&format!(\"send {shown}\")),
        }
    }
    fn wrapped(
        what: &str,
    ) -> Self {
        Self {
            message: owned_message(what),
        }
    }
}
";
    let fixture_sites = sites(&constructors(&[(
        "src/dispatch/send.rs".to_string(),
        code_lines(fixture),
    )]));
    assert_eq!(
        fixture_sites.keys().map(String::as_str).collect::<Vec<_>>(),
        ["deferred_send", "wrapped"]
    );
    assert_eq!(fixture_sites["deferred_send"], "src/dispatch/send.rs:2");
    let sites = sites(&constructors(&sources()));
    for (name, form) in [
        ("named_semaphore", "a Deviation refusal"),
        ("fork_child", "a Deviation refusal"),
        ("internal_routine", "a constructor taking a name"),
        ("unresolved_call", "a constructor with a local binding"),
        ("binary_operator", "a message with no owned_message"),
        ("instruction", "a message from a match"),
        ("unfilled_slot", "a constructor outside its caller's file"),
        ("sim_endless_wait", "a constructor in sim.rs"),
    ] {
        assert!(sites.contains_key(name), "{name}: {form} is not found");
    }
}

/// Each way a table can be wrong is found: a constructor with no row, a row
/// with no constructor, a row naming a phase, a kind that is not a
/// disposition, a duplicate row, a struct literal outside a constructor, a
/// citation that does not resolve, and a `-> Self` constructor.
#[test]
fn the_check_finds_each_kind_of_disagreement() {
    let source = "\
impl Loud {
    fn plain() -> Loud {
        Loud {
            message: \"x\".to_string(),
        }
    }
    fn none(what: &str) -> Loud {
        Loud {
            message: owned_message(what),
        }
    }
    fn mutant_self(what: &str) -> Self {
        Self {
            message: owned_message(what),
        }
    }
    fn mixed(what: &str, early: bool) -> Loud {
        if early {
            return Loud {
                message: owned_message(what),
            };
        }
        Loud {
            message: what.to_string(),
        }
    }
}
fn elsewhere() -> Result<(), Failure> {
    Err(Loud {
        message: owned_message(\"v\"),
    }
    .into())
}
fn adjacent(r: u8) -> Failure {
    match r {
        0 => return Loud { message: String::new() }.into(),
        _ => Loud {
            message: owned_message(\"t\"),
        }
        .into(),
    }
}
fn hostemu() -> Loud {
    Loud {
        message: owned_message(\"s\"),
    }
}
";
    let files = vec![("src/lib.rs".to_string(), code_lines(source))];
    let constructors = constructors(&files);
    let sites = sites(&constructors);
    let literals = literals(&files, &constructors);
    assert_eq!(
        sites.keys().map(String::as_str).collect::<Vec<_>>(),
        ["hostemu", "mixed", "mutant_self", "none", "plain"]
    );
    let documents = Documents {
        crashes: "\n## 6. An array\n".to_string(),
        deviations: active_deviations("20\tactive\t-\t-\n21\twithdrawn\t-\t-\n"),
        repository: rust_root().parent().expect("the repository").to_path_buf(),
    };
    let table = "\
plain\tGUARD\t-\tprobe p1
none\tDEVIATION\t-\toracle-crashes.txt entry 6
mutant_self\tLIMIT\t-\tDeviation 20
mixed\tGUARD\t-\tprobe p5
hostemu\tGUARD\t-\tprobe p6
";
    assert!(
        problems(&sites, &[], &parse_table(table), &documents).is_empty(),
        "{:?}",
        problems(&sites, &[], &parse_table(table), &documents)
    );
    let wrong = "\
plain\tGUARD\t-\tprobe p1
plain\tGUARD\t-\tprobe p1
none\tDEVIATION\t-\toracle-crashes.txt entry 7
gone\tGUARD\t-\tprobe p2
hostemu\tREHOME\tPhase 11\t\"the row's text\"
mixed\tGUARD\tPhase 11\tprobe p3
mutant_self\tLIMIT\t-\tDeviation 21
";
    let found = problems(&sites, &literals, &parse_table(wrong), &documents);
    let expect = [
        "src/lib.rs:29: a `Loud` struct literal",
        "src/lib.rs:36: a `Loud` struct literal",
        "src/lib.rs:37: a `Loud` struct literal",
        "plain: 2 rows",
        "none: cites oracle-crashes.txt entry 7",
        "gone: a row with no constructor",
        "hostemu: kind \"REHOME\" is not one of",
        "mixed: phase \"Phase 11\" on a GUARD row",
        "mutant_self: cites Deviation 21, which does not exist",
    ];
    for needle in expect {
        assert!(
            found.iter().any(|problem| problem.starts_with(needle)),
            "{needle:?} not among {found:#?}"
        );
    }
    assert_eq!(found.len(), expect.len(), "{found:#?}");
}
