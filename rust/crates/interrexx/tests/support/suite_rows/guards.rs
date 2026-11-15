//! The assertions an Unspecified row names, and the guard that keeps each from
//! running while one of its rows' tests runs.

use super::*;
use crate::group_runner::{RUNNING_TEST, read_bytes};

/// The prefix of an Unspecified row's reason that names its assertions.
const ASSERTIONS: &str = "assertions ";

/// The lines of its group file that an Unspecified row's reason names in its
/// leading `assertions LIST; `, where LIST is space-separated line numbers
/// and `FIRST-LAST` ranges, each line named once.
pub fn assertion_lines(reason: &str) -> Result<BTreeSet<usize>, String> {
    let list = reason
        .strip_prefix(ASSERTIONS)
        .and_then(|rest| rest.split_once("; "))
        .map(|(list, _)| list)
        .ok_or("the reason does not start `assertions LINES; `")?;
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
        let lines = assertion_lines(&row.reason)
            .unwrap_or_else(|problem| panic!("{}: {problem}", row.at()));
        for line in lines {
            guards.entry(line).or_default().insert(row.test.clone());
        }
    }
    guards
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
/// otherwise. The guard goes in front
/// of the clause on the same line, so no line number changes, and it is an IF
/// with its own ELSE, so an ELSE on a later line binds as before.
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
        assert!(
            names.iter().all(|name| name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')),
            "test names of letters, digits and underscores: {names:?}"
        );
        let indent = line
            .iter()
            .position(|byte| !byte.is_ascii_whitespace())
            .unwrap_or(line.len());
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
