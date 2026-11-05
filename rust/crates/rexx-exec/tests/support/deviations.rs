//! `rust/corpus/deviations.tsv`, the machine-readable half of the
//! specification's Deviations chapter.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// One Deviation's row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deviation {
    pub active: bool,
    /// The model tests (`numeric_model`, `stream_model`) whose oracle
    /// departures it licenses.
    pub models: Vec<String>,
    /// Its rows in `licensed_divergences.rs`'s table, by name.
    pub witnesses: Vec<String>,
}

/// The table's path, relative to the repository.
pub const TABLE: &str = "rust/corpus/deviations.tsv";

/// The chapter's path, relative to the repository.
pub const CHAPTER: &str = "spec/src/deviations.md";

/// The repository root.
pub fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// The table at `repo`, by number.
///
/// # Panics
/// If the file cannot be read or a row is malformed.
pub fn load(repo: &Path) -> BTreeMap<u32, Deviation> {
    let path = repo.join(TABLE);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    parse(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The rows of `text`, by number; a `#` line is a comment.
pub fn parse(text: &str) -> Result<BTreeMap<u32, Deviation>, String> {
    let list = |field: &str| -> Vec<String> {
        if field == "-" {
            Vec::new()
        } else {
            field.split(',').map(str::to_string).collect()
        }
    };
    let mut rows = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let [number, status, models, witnesses] = fields[..] else {
            return Err(format!("line {}: expected 4 fields", index + 1));
        };
        let number: u32 = number
            .parse()
            .map_err(|_| format!("line {}: {number:?} is not a number", index + 1))?;
        let active = match status {
            "active" => true,
            "withdrawn" => false,
            _ => return Err(format!("line {}: status {status:?}", index + 1)),
        };
        let row = Deviation {
            active,
            models: list(models),
            witnesses: list(witnesses),
        };
        if rows.insert(number, row).is_some() {
            return Err(format!("line {}: Deviation {number} twice", index + 1));
        }
    }
    Ok(rows)
}

/// The chapter's `## N. Title` headings in order: `true` for an active
/// entry, `false` for one titled `Withdrawn`.
pub fn chapter_headings(text: &str) -> Vec<(u32, bool)> {
    let mut headings = Vec::new();
    for line in text.lines() {
        let Some((number, title)) = line.strip_prefix("## ").and_then(|h| h.split_once(". "))
        else {
            continue;
        };
        if let Ok(number) = number.parse::<u32>() {
            headings.push((number, title.trim() != "Withdrawn"));
        }
    }
    headings
}
