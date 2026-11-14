//! Finds copies of the catalogue's message texts in a repository.
//!
//! A text is found by its longest literal piece (the stretch between two
//! inserts, at least [`PIECE`] characters and [`PIECE_WORDS`] words, so a
//! stock phrase of two words does not count) or by its whole template (its
//! literals in order, each insert standing for a short stretch of anything).
//! The template catches a text made mostly of inserts whose literals are
//! each short; it needs two literals with an insert between them. A text with neither kind of literal (for example `&1`) cannot
//! be found, and [`unfindable`] lists those codes.
//!
//! Text is compared lower-case with every run of white space as one blank,
//! line breaks and comment markers (`//`, `///`, `//!`, `#`, `*`, `>`) at the
//! start of a line included, so a text wrapped across lines or comment lines
//! is still found. Nothing here prints a catalogue text, only the code.

use std::path::Path;
use std::process::Command;

use crate::errors::Message;

/// The least length of the literal piece that finds a text on its own.
pub const PIECE: usize = 12;
/// The fewest words of that piece.
pub const PIECE_WORDS: usize = 3;
/// The most characters one insert may stand for in a template match.
const GAP: usize = 60;
/// The least total length of a template's literals.
const TEMPLATE_LITERALS: usize = 10;

/// What finds one catalogue text.
#[derive(Clone, Debug)]
pub struct Pattern {
    pub major: u16,
    pub sub: u16,
    pub symbol: String,
    /// The longest literal, when it is at least [`PIECE`] long.
    piece: Option<String>,
    /// The literals in order, when there are at least two (so an insert
    /// lies between) and they are long enough together.
    template: Option<Vec<String>>,
}

impl Pattern {
    pub fn new(message: &Message) -> Pattern {
        let literals = literals(&message.text);
        let piece = literals
            .iter()
            .filter(|l| l.len() >= PIECE && l.split(' ').count() >= PIECE_WORDS)
            .max_by_key(|l| l.len())
            .cloned();
        // One literal is the piece's business: a template needs an insert
        // between two literals.
        let template =
            (literals.len() >= 2 && total_len(&literals) >= TEMPLATE_LITERALS).then_some(literals);
        Pattern {
            major: message.major,
            sub: message.sub,
            symbol: message.symbol.clone(),
            piece,
            template,
        }
    }

    /// Whether a match is possible at all.
    pub fn findable(&self) -> bool {
        self.piece.is_some() || self.template.is_some()
    }

    /// The byte offset in `haystack` (normalised) and the way the text
    /// matches, or `None`.
    pub fn find(&self, haystack: &str) -> Option<(usize, Kind)> {
        if let Some(piece) = &self.piece
            && let Some(at) = haystack.find(piece.as_str())
        {
            return Some((at, Kind::Piece));
        }
        let template = self.template.as_ref()?;
        let first = template.first()?;
        let mut from = 0;
        while let Some(found) = haystack[from..].find(first.as_str()) {
            let start = from + found;
            if template_matches(haystack, start, template) {
                return Some((start, Kind::Template));
            }
            from = start + first.len().max(1);
            while !haystack.is_char_boundary(from) {
                from += 1;
            }
        }
        None
    }
}

/// Whether `template[0]` at `start` is followed by the rest of the
/// literals, each within [`GAP`] characters of the one before.
fn template_matches(haystack: &str, start: usize, template: &[String]) -> bool {
    let mut at = start + template[0].len();
    for literal in &template[1..] {
        let mut end = (at + GAP + literal.len()).min(haystack.len());
        while !haystack.is_char_boundary(end) {
            end += 1;
        }
        let Some(found) = haystack[at..end].find(literal.as_str()) else {
            return false;
        };
        at += found + literal.len();
    }
    true
}

/// How a text was found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Piece,
    Template,
}

/// The normalised literals of `text`: what lies between inserts, trimmed,
/// without the empty ones.
fn literals(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' && chars.peek().is_some_and(char::is_ascii_digit) {
            chars.next();
            out.push(std::mem::take(&mut current));
        } else {
            current.push(c);
        }
    }
    out.push(current);
    out.into_iter()
        .map(|l| normalise_line(&l))
        .filter(|l| !l.is_empty())
        .collect()
}

/// Lower-case, one blank for each run of white space, and no backslash
/// before a quotation mark (as source and golden files escape them).
fn normalise_line(line: &str) -> String {
    let unescaped = line.replace("\\\"", "\"").replace("\\'", "'");
    unescaped
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn total_len(literals: &[String]) -> usize {
    literals.iter().map(String::len).sum()
}

/// The codes whose text no pattern can find.
pub fn unfindable(patterns: &[Pattern]) -> Vec<(u16, u16)> {
    patterns
        .iter()
        .filter(|p| !p.findable())
        .map(|p| (p.major, p.sub))
        .collect()
}

const COMMENT_MARKERS: [&str; 6] = ["///", "//!", "//", "#", "*", ">"];

/// A file's text normalised for matching, with the line each part came from.
pub struct Haystack {
    text: String,
    /// `(offset in text, line number)` for each line that added text.
    lines: Vec<(usize, usize)>,
}

impl Haystack {
    /// `content` normalised. With `comments_only`, only the part of each
    /// line from its first `//` counts.
    pub fn new(content: &str, comments_only: bool) -> Haystack {
        let mut text = String::new();
        let mut lines = Vec::new();
        for (index, raw) in content.lines().enumerate() {
            let line = if comments_only {
                match raw.find("//") {
                    Some(at) => &raw[at..],
                    None => continue,
                }
            } else {
                raw
            };
            let mut line = line.trim_start();
            for marker in COMMENT_MARKERS {
                if let Some(rest) = line.strip_prefix(marker) {
                    line = rest;
                    break;
                }
            }
            let line = normalise_line(line);
            if line.is_empty() {
                continue;
            }
            if !text.is_empty() {
                text.push(' ');
            }
            lines.push((text.len(), index + 1));
            text.push_str(&line);
        }
        Haystack { text, lines }
    }

    /// The line of the content that the normalised offset `at` lies on.
    fn line_of(&self, at: usize) -> usize {
        match self.lines.binary_search_by_key(&at, |(offset, _)| *offset) {
            Ok(i) => self.lines[i].1,
            Err(0) => 1,
            Err(i) => self.lines[i - 1].1,
        }
    }

    /// Every pattern that finds a match here, with the line of the match.
    pub fn matches<'a>(&self, patterns: &'a [Pattern]) -> Vec<(&'a Pattern, usize, Kind)> {
        patterns
            .iter()
            .filter_map(|p| {
                let (at, kind) = p.find(&self.text)?;
                Some((p, self.line_of(at), kind))
            })
            .collect()
    }
}

/// One file that holds a catalogue text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub path: String,
    pub line: usize,
    pub major: u16,
    pub sub: u16,
    pub symbol: String,
    pub kind: Kind,
}

/// The matches in `files` (path and content), in file order. `comments_only`
/// limits it to Rust files' `//` comments.
pub fn scan<I>(patterns: &[Pattern], files: I, comments_only: bool) -> Vec<Found>
where
    I: IntoIterator<Item = (String, String)>,
{
    let mut found = Vec::new();
    for (path, content) in files {
        if comments_only && !path.ends_with(".rs") {
            continue;
        }
        for (p, line, kind) in Haystack::new(&content, comments_only).matches(patterns) {
            found.push(Found {
                path: path.clone(),
                line,
                major: p.major,
                sub: p.sub,
                symbol: p.symbol.clone(),
                kind,
            });
        }
    }
    found
}

/// What `git ls-files` lists under `root`, each file's path (relative to
/// `root`) and text. A file that is not valid UTF-8 is read lossily and
/// counted in the second answer, so a binary file is not skipped unseen. A
/// listed path that is not a file (a submodule) is skipped.
pub fn tracked_files(root: &Path, under: &str) -> Result<(Vec<(String, String)>, usize), String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z", "--", under])
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let mut files = Vec::new();
    let mut lossy = 0;
    for name in output.stdout.split(|b| *b == 0).filter(|n| !n.is_empty()) {
        let name = String::from_utf8_lossy(name).into_owned();
        let path = root.join(&name);
        if !path.is_file() {
            continue;
        }
        let bytes = std::fs::read(&path).map_err(|e| format!("cannot read {name}: {e}"))?;
        let text = match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(e) => {
                lossy += 1;
                String::from_utf8_lossy(e.as_bytes()).into_owned()
            }
        };
        files.push((name, text));
    }
    Ok((files, lossy))
}

#[cfg(test)]
mod tests;
