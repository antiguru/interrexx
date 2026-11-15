//! The assertions an Unspecified row names, and the guard that keeps each from
//! running while one of its rows' tests runs.

use super::*;
use crate::group_runner::{RUNNING_TEST, read_bytes};

/// The prefix of an Unspecified row's reason that names its assertions.
const ASSERTIONS: &str = "assertions ";

/// The prefix, after the assertions, of the part of an Unspecified row's reason
/// that names conditions and what replaces them.
const CONDITIONS: &str = "conditions ";

/// Separates a condition list from its replacement.
const REPLACED_BY: &str = " as ";

/// What an Unspecified row's reason names in its group file: the lines of its
/// assertions and the lines of its conditions with the one expression that
/// replaces each condition's test while the row's test runs.
#[derive(Debug, PartialEq, Eq)]
pub struct Named {
    pub assertions: BTreeSet<usize>,
    pub conditions: BTreeSet<usize>,
    /// The replacement of every line in `conditions`, empty where there are none.
    pub replacement: String,
}

/// The lines of its group file that an Unspecified row's reason names in its
/// leading `assertions LIST; ` and then `conditions LIST as EXPRESSION; `, each
/// part optional but one present. LIST is space-separated line numbers and
/// `FIRST-LAST` ranges, each line named once in the reason, and EXPRESSION is a
/// Rexx expression without `; `.
pub fn named_lines(reason: &str) -> Result<Named, String> {
    let mut rest = reason;
    let mut named = Named {
        assertions: BTreeSet::new(),
        conditions: BTreeSet::new(),
        replacement: String::new(),
    };
    if let Some(after) = rest.strip_prefix(ASSERTIONS) {
        let (list, after) = after
            .split_once("; ")
            .ok_or("the assertions are not ended by `; `")?;
        named.assertions = line_list(list)?;
        rest = after;
    }
    if let Some(after) = rest.strip_prefix(CONDITIONS) {
        let (list, after) = after
            .split_once(REPLACED_BY)
            .ok_or("the conditions are not followed by ` as EXPRESSION; `")?;
        let (expression, _) = after
            .split_once("; ")
            .ok_or("the replacement expression is not ended by `; `")?;
        if expression.trim().is_empty() || expression.contains(['\n', '\r']) {
            return Err("the replacement expression is empty or spans lines".to_string());
        }
        named.conditions = line_list(list)?;
        named.replacement = expression.to_string();
    }
    if named.assertions.is_empty() && named.conditions.is_empty() {
        return Err(
            "the reason does not start `assertions LINES; ` or `conditions LINES as EXPRESSION; `"
                .to_string(),
        );
    }
    if let Some(line) = named.assertions.intersection(&named.conditions).next() {
        return Err(format!("line {line} is both an assertion and a condition"));
    }
    Ok(named)
}

/// The lines a space-separated LIST of numbers and `FIRST-LAST` ranges names,
/// each once.
fn line_list(list: &str) -> Result<BTreeSet<usize>, String> {
    let number = |text: &str| {
        text.parse::<usize>()
            .ok()
            .filter(|n| *n > 0 && text.bytes().all(|byte| byte.is_ascii_digit()))
    };
    let mut lines = BTreeSet::new();
    for word in list.split(' ') {
        let (first, last) = word.split_once('-').unwrap_or((word, word));
        let (Some(first), Some(last)) = (number(first), number(last)) else {
            return Err(format!("{word:?} is not a line number or a range of them"));
        };
        if first > last {
            return Err(format!("the range {word} is empty"));
        }
        for line in first..=last {
            if !lines.insert(line) {
                return Err(format!("line {line} is named twice"));
            }
        }
    }
    Ok(lines)
}

/// Each line `group`'s Unspecified rows name, with the tests whose rows name
/// it. The rows are well formed ([`parse_rows`] reports one whose lines do
/// not parse).
pub fn guarded_assertions(rows: &[Row], group: &str) -> BTreeMap<usize, BTreeSet<String>> {
    let mut guards: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
    for row in rows
        .iter()
        .filter(|row| row.group == group && row.owner == Owner::Unspecified)
    {
        let named =
            named_lines(&row.reason).unwrap_or_else(|problem| panic!("{}: {problem}", row.at()));
        for line in named.assertions {
            guards.entry(line).or_default().insert(row.test.clone());
        }
    }
    guards
}

/// Each condition line `group`'s Unspecified rows name, with the tests whose
/// rows name it and the expression that replaces its test for them. The rows
/// are well formed, and rows that name one line agree on its expression.
pub fn guarded_conditions(rows: &[Row], group: &str) -> BTreeMap<usize, Condition> {
    let mut guards: BTreeMap<usize, Condition> = BTreeMap::new();
    for row in rows
        .iter()
        .filter(|row| row.group == group && row.owner == Owner::Unspecified)
    {
        let named =
            named_lines(&row.reason).unwrap_or_else(|problem| panic!("{}: {problem}", row.at()));
        for line in named.conditions {
            let guard = guards.entry(line).or_insert_with(|| Condition {
                tests: BTreeSet::new(),
                replacement: named.replacement.clone(),
            });
            assert_eq!(
                guard.replacement,
                named.replacement,
                "{}: line {line} is replaced differently by another row",
                row.at()
            );
            guard.tests.insert(row.test.clone());
        }
    }
    guards
}

/// A condition line's guard: the tests that run its replacement and the
/// replacement.
#[derive(Debug, PartialEq, Eq)]
pub struct Condition {
    pub tests: BTreeSet<String>,
    pub replacement: String,
}

/// Whether the source line `line` starts, after its indentation, with an
/// assertion: a `self~assert...` or `self~fail...` message.
pub fn starts_assertion(line: &[u8]) -> bool {
    let clause = clause_of(line).to_ascii_lowercase();
    clause.starts_with(b"self~assert") || clause.starts_with(b"self~fail")
}

/// Whether a guard may go in front of the source line `line`, followed by
/// `next`: the line starts an assertion, or it is an IF that ends in THEN and
/// `next` starts an assertion, so the IF's condition is part of what the
/// assertion checks.
pub fn guardable(line: &[u8], next: Option<&[u8]>) -> bool {
    if starts_assertion(line) {
        return true;
    }
    let clause = clause_of(line).trim_ascii_end().to_ascii_lowercase();
    clause.starts_with(b"if")
        && clause.get(2).is_some_and(u8::is_ascii_whitespace)
        && clause.ends_with(b" then")
        && next.is_some_and(starts_assertion)
}

/// The byte range of the condition of a source line that is an IF ending in
/// THEN, the text between the two without its blanks, `None` for any other
/// line.
pub fn condition_span(line: &[u8]) -> Option<std::ops::Range<usize>> {
    let indent = line.len() - clause_of(line).len();
    let clause = clause_of(line).trim_ascii_end();
    let rest = clause
        .get(..2)
        .filter(|keyword| keyword.eq_ignore_ascii_case(b"if"))
        .and(clause.get(2..))
        .filter(|rest| rest.first().is_some_and(u8::is_ascii_whitespace))?;
    let then = rest.len().checked_sub(5)?;
    if !rest[then..].eq_ignore_ascii_case(b" then") {
        return None;
    }
    let condition = rest[..then].trim_ascii();
    let start = indent + 2 + (rest.len() - rest.trim_ascii_start().len());
    (!condition.is_empty()).then(|| start..start + condition.len())
}

/// `line` after its indentation.
fn clause_of(line: &[u8]) -> &[u8] {
    let start = line
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(line.len());
    &line[start..]
}

/// The `::METHOD` or `::ROUTINE` whose body holds line `line` of `text`, as
/// its lower-case keyword and its upper-case name; `None` where the nearest
/// directive above the line is another or there is none.
pub fn enclosing(text: &str, line: usize) -> Option<(String, String)> {
    let directive = text
        .lines()
        .take(line.saturating_sub(1))
        .map(str::trim_start)
        .filter(|source| source.starts_with("::"))
        .last()?;
    let mut words = directive[2..].split_whitespace();
    let keyword = words.next()?.to_ascii_lowercase();
    let name = words.next()?.trim_matches(['"', '\'']).to_ascii_uppercase();
    matches!(keyword.as_str(), "method" | "routine").then_some((keyword, name))
}

/// Rewrites each line of the copied `group_file` that `guards` names so that
/// its clause, an assertion or an IF of one ([`guardable`]), does not run
/// while one of the line's tests runs, by [`RUNNING_TEST`], and runs as before
/// otherwise. The guard goes in front of the clause on the same line, so no
/// line number changes, and it is an IF with its own ELSE, so an ELSE on a
/// later line binds as before.
pub fn guard_assertions(group_file: &Path, guards: &BTreeMap<usize, BTreeSet<String>>) {
    if guards.is_empty() {
        return;
    }
    let text = read_bytes(group_file);
    let mut rewritten = Vec::with_capacity(text.len());
    let mut done = 0;
    let lines: Vec<&[u8]> = text.split_inclusive(|byte| *byte == b'\n').collect();
    for (at, line) in lines.iter().copied().enumerate() {
        let Some(tests) = guards.get(&(at + 1)) else {
            rewritten.extend_from_slice(line);
            continue;
        };
        assert!(
            guardable(line, lines.get(at + 1).copied()),
            "{} line {} starts no assertion",
            group_file.display(),
            at + 1
        );
        let names: Vec<&str> = tests.iter().map(String::as_str).collect();
        // The names are words of a quoted string.
        assert!(
            names.iter().all(|name| !name
                .bytes()
                .any(|byte| byte == b'\'' || byte.is_ascii_whitespace())),
            "test names without a quote or a blank: {names:?}"
        );
        let indent = line.len() - clause_of(line).len();
        rewritten.extend_from_slice(&line[..indent]);
        rewritten.extend_from_slice(
            format!(
                "if wordpos(.{RUNNING_TEST}, '{}') > 0 then nop; else ",
                names.join(" ")
            )
            .as_bytes(),
        );
        rewritten.extend_from_slice(&line[indent..]);
        done += 1;
    }
    assert_eq!(
        done,
        guards.len(),
        "every guarded line of {} exists",
        group_file.display()
    );
    fs::write(group_file, rewritten).expect("cannot rewrite the copied group file");
}

/// Rewrites each IF line of the copied `group_file` that `guards` names so that
/// its condition is the line's replacement, not the line's own, while one of
/// the line's tests runs ([`RUNNING_TEST`]), and the line's own otherwise. The
/// rewritten condition stays on the same line, so no line number changes, and
/// the IF keeps its THEN and what follows it, so its ELSE binds as before. Both
/// conditions are evaluated, which a condition that raises for another test's
/// data must not rely on avoiding.
pub fn guard_conditions(group_file: &Path, guards: &BTreeMap<usize, Condition>) {
    if guards.is_empty() {
        return;
    }
    let text = read_bytes(group_file);
    let mut rewritten = Vec::with_capacity(text.len());
    let mut done = 0;
    for (at, line) in text.split_inclusive(|byte| *byte == b'\n').enumerate() {
        let Some(guard) = guards.get(&(at + 1)) else {
            rewritten.extend_from_slice(line);
            continue;
        };
        let span = condition_span(line).unwrap_or_else(|| {
            panic!(
                "{} line {} is no IF ending in THEN",
                group_file.display(),
                at + 1
            )
        });
        let names: Vec<&str> = guard.tests.iter().map(String::as_str).collect();
        // The names are words of a quoted string.
        assert!(
            names.iter().all(|name| !name
                .bytes()
                .any(|byte| byte == b'\'' || byte.is_ascii_whitespace())),
            "test names without a quote or a blank: {names:?}"
        );
        let running = format!("wordpos(.{RUNNING_TEST}, '{}')", names.join(" "));
        // Everything before the span is the indentation and the IF, everything
        // after it the THEN and the rest of the line.
        rewritten.extend_from_slice(&line[..span.start]);
        rewritten.extend_from_slice(
            format!(
                "({running} > 0 & ({})) | ({running} = 0 & (",
                guard.replacement
            )
            .as_bytes(),
        );
        rewritten.extend_from_slice(&line[span.clone()]);
        rewritten.extend_from_slice(b"))");
        rewritten.extend_from_slice(&line[span.end..]);
        done += 1;
    }
    assert_eq!(
        done,
        guards.len(),
        "every guarded condition of {} exists",
        group_file.display()
    );
    fs::write(group_file, rewritten).expect("cannot rewrite the copied group file");
}
