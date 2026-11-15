//! A class row's coverage: the construction measurements, read together
//! with the reference's own passages saying a class cannot be constructed.

use std::collections::BTreeMap;

use super::{
    Book, CONSTRUCTION, CONSTRUCTION_DIRECTIVES, CONSTRUCTION_PROGRAMS, Coverage, Route, Status,
    UNCONSTRUCTIBLE, UNCONSTRUCTIBLE_BOOK,
};

/// Re-reads each [`UNCONSTRUCTIBLE`] passage at the lines it cites and
/// compares its [`digest`] with the committed one, so a citation that has
/// gone stale or been reworded reddens rather than being carried.
pub(super) fn unconstructible_index(
    books: &[Book],
) -> BTreeMap<&'static str, (usize, usize, Status)> {
    let book = books
        .iter()
        .find(|b| b.name == UNCONSTRUCTIBLE_BOOK)
        .unwrap_or_else(|| panic!("{UNCONSTRUCTIBLE_BOOK} is not among the books"));
    let lines: Vec<&str> = book.text.lines().collect();
    let mut out = BTreeMap::new();
    for &(class, first, last, expected, status) in UNCONSTRUCTIBLE {
        let span = lines
            .get(first - 1..last)
            .unwrap_or_else(|| panic!("{UNCONSTRUCTIBLE_BOOK} has no lines {first}-{last}"))
            .join(" ");
        let plain = collapse(&strip_tags(&span));
        let actual = digest(&plain);
        assert!(
            actual == expected,
            "{UNCONSTRUCTIBLE_BOOK}:{first}-{last} no longer matches its committed digest \
             {expected:#018x}; it now hashes to {actual:#018x} and reads {plain:?}. Re-read \
             the passage: this citation grounds {class}'s status, and a status grounded in a \
             passage that moved or changed is grounded in nothing. Commit the new digest \
             only if the status still follows"
        );
        out.insert(class, (first, last, status));
    }
    out
}

/// Everything outside a `<...>`, with the tags replaced by nothing.
fn strip_tags(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find('<') {
        out.push_str(&rest[..at]);
        rest = match rest[at..].find('>') {
            Some(end) => &rest[at + end + 1..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// FNV-1a over the bytes of a normalised passage. It is fixed by its
/// definition rather than by a toolchain release, so a committed value stays
/// comparable. It detects drift and does not resist a deliberate collision.
fn digest(plain: &str) -> u64 {
    plain.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// A class's coverage and the reason `class-set.txt` states it in.
pub(super) fn coverage_of(
    name: &str,
    passage: Option<(usize, usize, Status)>,
) -> (Coverage, String) {
    let upper = name.to_ascii_uppercase();
    let construction = CONSTRUCTION
        .iter()
        .find(|(n, _)| *n == upper)
        .map(|(_, outcome)| *outcome);
    let program = CONSTRUCTION_PROGRAMS
        .iter()
        .find(|(n, _)| *n == upper)
        .map(|(_, program)| *program);
    let directive = CONSTRUCTION_DIRECTIVES
        .iter()
        .find(|(n, _)| *n == upper)
        .map(|(_, directive)| (*directive).to_string());
    assert!(
        program.is_none() || construction != Some("new"),
        "{name} has a committed construction program and a bare ~new that constructs on \
         the oracle, so the program is a route to nothing the bare ~new does not already reach"
    );
    assert!(
        directive.is_none() || program.is_some(),
        "{name} has a committed construction directive and no construction program, so \
         nothing derives a probe that would carry the directive"
    );
    let route = |expression: &str| Route {
        expression: expression.to_string(),
        directive: directive.clone(),
    };
    if let Some((first, last, status)) = passage {
        let at = if first == last {
            format!("{first}")
        } else {
            format!("{first}-{last}")
        };
        let cited = format!("{UNCONSTRUCTIBLE_BOOK}:{at}");
        if status == Status::Unreachable {
            assert!(
                program.is_none(),
                "{name} is unreachable and has a committed construction program"
            );
            return (
                Coverage::Unreachable,
                format!("the reference says instances come only from native code -- {cited}"),
            );
        }
        return match program {
            Some(program) => (
                Coverage::Covered(route(program)),
                format!(
                    "a committed construction program answers an instance; the reference \
                     says the user cannot create one and names a Rexx-level route instead \
                     -- {cited}"
                ),
            ),
            None => (
                Coverage::NotCovered,
                format!(
                    "no construction program is committed; the reference says the user \
                     cannot create one and names a Rexx-level route instead -- {cited}"
                ),
            ),
        };
    }
    if let Some(program) = program {
        let code = construction.unwrap_or_default();
        return (
            Coverage::Covered(route(program)),
            format!(
                "a committed construction program answers an instance; a bare ~new raises \
                 {code} on the oracle"
            ),
        );
    }
    match construction {
        Some("new") => (
            Coverage::Covered(route(&format!(".{name}~new"))),
            "a bare ~new constructs an instance on the oracle".into(),
        ),
        Some(code) => (
            Coverage::NotCovered,
            format!(
                "no construction program is committed; a bare ~new raises {code} on the oracle"
            ),
        ),
        None => (
            Coverage::NotCovered,
            "no construction program is committed; the name is not an .environment class entry"
                .into(),
        ),
    }
}
