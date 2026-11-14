//! The texts of interrexx's own error messages, as checked in under
//! `interrexx-inventory/messages/<major>.tsv`.
//!
//! One file per major, one row per message: `major<TAB>sub<TAB>text`. The
//! text holds `&k` where insert `k` goes, and only for a `k` in the row's
//! rendered set in `errors.tsv`. Lines that are empty or start with `#`
//! are comments. A text holds no tab and no line break.

use std::collections::BTreeSet;
use std::path::Path;

use crate::errors::{Table, rendered_positions};

/// One row of a messages file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text {
    pub major: u16,
    pub sub: u16,
    /// The text, with `&k` marking insert `k`.
    pub text: String,
}

/// Parses `contents`, the file `<major>.tsv` (its name is `file`, used in
/// the problems and to check each row's major). Every problem found is
/// returned, not only the first.
pub fn parse_file(file: &str, contents: &str) -> Result<Vec<Text>, Vec<String>> {
    let expected: Option<u16> = file.strip_suffix(".tsv").and_then(|n| n.parse().ok());
    let mut rows = Vec::new();
    let mut problems = Vec::new();
    if expected.is_none() {
        problems.push(format!("{file}: the name is not `<major>.tsv`"));
    }
    for (index, line) in contents.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = format!("{file}:{}", index + 1);
        let fields: Vec<&str> = line.split('\t').collect();
        let [major, sub, text] = fields.as_slice() else {
            problems.push(format!("{at}: want `major<TAB>sub<TAB>text`"));
            continue;
        };
        let (Ok(major), Ok(sub)) = (major.parse::<u16>(), sub.parse::<u16>()) else {
            problems.push(format!("{at}: major and sub are not numbers"));
            continue;
        };
        if expected.is_some_and(|m| m != major) {
            problems.push(format!("{at}: major {major} does not belong in {file}"));
        }
        if text.trim().is_empty() {
            problems.push(format!("{at}: {major}.{sub} has an empty text"));
            continue;
        }
        rows.push(Text {
            major,
            sub,
            text: (*text).to_string(),
        });
    }
    if problems.is_empty() {
        Ok(rows)
    } else {
        Err(problems)
    }
}

/// The problems of `texts` against `table`: a code the table does not have,
/// a code given twice, and an insert outside the row's rendered set (which
/// includes any `&k` of a message that shows no insert). Empty when the
/// texts are sound.
pub fn validate(table: &Table, texts: &[Text]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut seen = BTreeSet::new();
    for t in texts {
        let code = format!("{}.{}", t.major, t.sub);
        let Some(row) = table
            .rows
            .iter()
            .find(|r| r.major == t.major && r.sub == t.sub)
        else {
            problems.push(format!("{code} is not in errors.tsv"));
            continue;
        };
        if !seen.insert((t.major, t.sub)) {
            problems.push(format!("{code} has more than one text"));
        }
        for k in rendered_positions(&t.text) {
            if !row.rendered.contains(&k) {
                problems.push(format!(
                    "{code} uses &{k}, but its rendered inserts are {:?}",
                    row.rendered
                ));
            }
        }
    }
    problems
}

/// Reads every `*.tsv` file of `dir` (none if the directory holds none),
/// in file-name order. A problem in any file is returned with the rest.
pub fn load_dir(dir: &Path) -> Result<Vec<Text>, Vec<String>> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map_err(|e| vec![format!("cannot read {}: {e}", dir.display())])?
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".tsv"))
        .collect();
    names.sort();
    let mut texts = Vec::new();
    let mut problems = Vec::new();
    for name in names {
        let path = dir.join(&name);
        match std::fs::read_to_string(&path) {
            Ok(contents) => match parse_file(&name, &contents) {
                Ok(rows) => texts.extend(rows),
                Err(p) => problems.extend(p),
            },
            Err(e) => problems.push(format!("cannot read {}: {e}", path.display())),
        }
    }
    if problems.is_empty() {
        Ok(texts)
    } else {
        Err(problems)
    }
}

#[cfg(test)]
mod tests;
