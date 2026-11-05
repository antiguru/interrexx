//! Parser and loader for the interrexx language specification under `spec/`.
//!
//! The checker test and the mdBook preprocessor both read pages through this
//! crate; it records what pages contain and leaves resolving links and running
//! examples to its consumers.

mod front_matter;
mod load;
mod page;
mod render;

use std::path::PathBuf;

pub use front_matter::{FrontMatter, Kind};
pub use load::{load, prefix_for};
pub use page::{Example, Page, Rule, RuleLink, parse_page};
pub use render::render;

/// A defect in the specification, located at a file and 1-based line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub path: PathBuf,
    pub line: usize,
    pub message: String,
}

/// Whether `id` is a well-formed rule ID: `^[a-z][a-z0-9]*(\.[a-z0-9][a-z0-9-]*)+$`.
pub fn is_rule_id(id: &str) -> bool {
    let mut parts = id.split('.');
    let Some(first) = parts.next() else {
        return false;
    };
    let mut chars = first.chars();
    let head_ok = chars.next().is_some_and(|c| c.is_ascii_lowercase());
    if !head_ok || !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
        return false;
    }
    let mut rest = 0;
    for part in parts {
        rest += 1;
        let mut chars = part.chars();
        let ok = chars
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
        if !ok || !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return false;
        }
    }
    rest > 0
}
