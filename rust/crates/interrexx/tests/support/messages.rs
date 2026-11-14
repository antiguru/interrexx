//! Maps the oracle's message lines to this crate's own message texts.
//!
//! The oracle prints its own wording of every catalogue message, and this
//! crate prints its own (`r[conv.error.text]`: the wording is unspecified,
//! the code, the inserts' values and their order, and the report frame are
//! specified). A harness that compares the two therefore maps each oracle
//! line first. The code on the line selects the oracle's template, the
//! template yields the inserts, and this crate's template for the same code
//! renders them. A line the translator cannot match is left alone, so it
//! shows as a mismatch.
//!
//! Where a text reaches the oracle's output:
//!
//! * the error report: `Error N running F line L:  text` and `Error N.M:  text`
//!   (the frame stays, the text after the colon is mapped);
//! * the same report without a position (`Error N:  text`);
//! * a debug pause's error lines, the prefix of 101.4 followed by ` N:` or
//!   ` N.M:` and the text;
//! * whole lines that are a catalogue text: the debug notice (101.5), the
//!   traceback lines of 101.18 to 101.26 and the utility messages of 999.
//!
//! The short names (`Error`, `running`, `line`, the month names) are not
//! mapped: they are the same in both catalogues.
//!
//! A text a program prints itself (`say errortext(n)`) is not translated:
//! such programs print codes instead.
//!
//! An insert that holds text matching the template's next literal makes the
//! split ambiguous. The translator takes the shortest insert that leaves the
//! rest matching and reports the line when another split would have
//! rendered differently.
//!
//! The oracle's templates are read at run time from `$REXX_UPSTREAM`, so
//! only a run that compares with the oracle needs them.

// This module is pulled in by `mod support;` and by `#[path]` in several
// integration test binaries, each of which links only the part it uses.
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

/// The highest insert position a template names, `&1` to `&9`.
const INSERTS: usize = 9;

/// The most splits of one line the matcher keeps. Two are enough to know a
/// line is ambiguous; the cap bounds the search on a line with many inserts.
const MAX_SPLITS: usize = 8;

/// One piece of a template: text, or the place insert `k` goes.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Piece {
    Text(Vec<u8>),
    Insert(usize),
}

/// A catalogue text split at its `&k` markers.
#[derive(Clone, Debug)]
struct Template {
    pieces: Vec<Piece>,
}

/// Where each insert sits in a matched line: entry `k` is a byte range of
/// the line, or `None` where the template does not show insert `k`.
type Split = [Option<(usize, usize)>; INSERTS + 1];

impl Template {
    fn parse(text: &str) -> Template {
        let bytes = text.as_bytes();
        let mut pieces = Vec::new();
        let mut run = Vec::new();
        let mut at = 0;
        while at < bytes.len() {
            let digit = bytes.get(at + 1).filter(|d| (b'1'..=b'9').contains(d));
            if bytes[at] == b'&'
                && let Some(&digit) = digit
            {
                if !run.is_empty() {
                    pieces.push(Piece::Text(std::mem::take(&mut run)));
                }
                pieces.push(Piece::Insert(usize::from(digit - b'0')));
                at += 2;
            } else {
                run.push(bytes[at]);
                at += 1;
            }
        }
        if !run.is_empty() {
            pieces.push(Piece::Text(run));
        }
        Template { pieces }
    }

    /// Whether the template has text with a character that is not a blank,
    /// so that matching it anchors on something.
    fn has_anchor(&self) -> bool {
        self.pieces.iter().any(|piece| match piece {
            Piece::Text(text) => text.iter().any(|byte| !byte.is_ascii_whitespace()),
            Piece::Insert(_) => false,
        })
    }

    /// Every way (up to [`MAX_SPLITS`]) the whole of `line` is this template
    /// with its inserts filled in, shortest insert first.
    fn splits(&self, line: &[u8]) -> Vec<Split> {
        let mut found = Vec::new();
        let mut split: Split = [None; INSERTS + 1];
        self.walk(0, 0, line, &mut split, &mut found);
        found
    }

    fn walk(
        &self,
        piece: usize,
        at: usize,
        line: &[u8],
        split: &mut Split,
        found: &mut Vec<Split>,
    ) {
        if found.len() >= MAX_SPLITS {
            return;
        }
        let Some(current) = self.pieces.get(piece) else {
            if at == line.len() {
                found.push(*split);
            }
            return;
        };
        match current {
            Piece::Text(text) => {
                if line[at..].starts_with(text) {
                    self.walk(piece + 1, at + text.len(), line, split, found);
                }
            }
            Piece::Insert(k) => {
                if let Some((start, end)) = split[*k] {
                    // An insert shown twice holds one value.
                    let value = &line[start..end];
                    if line[at..].starts_with(value) {
                        self.walk(piece + 1, at + value.len(), line, split, found);
                    }
                    return;
                }
                for end in at..=line.len() {
                    // The next text must follow; no next piece means the
                    // insert is the rest of the line.
                    let fits = match self.pieces.get(piece + 1) {
                        Some(Piece::Text(next)) => line[end..].starts_with(next),
                        Some(Piece::Insert(_)) => true,
                        None => end == line.len(),
                    };
                    if !fits {
                        continue;
                    }
                    split[*k] = Some((at, end));
                    self.walk(piece + 1, end, line, split, found);
                    split[*k] = None;
                }
            }
        }
    }

    /// The template with `split`'s inserts, or `None` where it names an
    /// insert the split has no value for.
    fn render(&self, line: &[u8], split: &Split) -> Option<Vec<u8>> {
        let mut out = Vec::new();
        for piece in &self.pieces {
            match piece {
                Piece::Text(text) => out.extend_from_slice(text),
                Piece::Insert(k) => {
                    let (start, end) = split[*k]?;
                    out.extend_from_slice(&line[start..end]);
                }
            }
        }
        Some(out)
    }

    /// The text of a template with no inserts.
    fn literal(&self) -> Option<Vec<u8>> {
        self.render(&[], &[None; INSERTS + 1])
    }
}

/// The result of [`Translator::translate`].
#[derive(Debug)]
pub struct Translated {
    pub bytes: Vec<u8>,
    /// The lines whose inserts split in more than one way with different
    /// results, each as `code: line`. The first split was used.
    pub ambiguous: Vec<String>,
}

/// One catalogue entry: its code and text, with `&k` marking insert `k`.
pub type Entry<'a> = (u16, u16, &'a str);

pub struct Translator {
    upstream: HashMap<(u16, u16), Template>,
    ours: HashMap<(u16, u16), Template>,
    /// The codes whose text is a line of its own: tried against every line.
    whole_line: Vec<(u16, u16)>,
}

/// Whether the text of `major.sub` is printed as a line of its own and is
/// not a short name: the debug notice, the traceback lines and the utility
/// messages.
fn is_whole_line(major: u16, sub: u16) -> bool {
    matches!((major, sub), (101, 5 | 18..=26) | (999, _))
}

impl Translator {
    /// A translator from the oracle's `upstream` templates to `ours`.
    pub fn new(upstream: &[Entry<'_>], ours: &[Entry<'_>]) -> Translator {
        let table = |entries: &[Entry<'_>]| -> HashMap<(u16, u16), Template> {
            entries
                .iter()
                .map(|(major, sub, text)| ((*major, *sub), Template::parse(text)))
                .collect()
        };
        let upstream = table(upstream);
        let ours = table(ours);
        let mut whole_line: Vec<(u16, u16)> = upstream
            .iter()
            .filter(|((major, sub), template)| {
                is_whole_line(*major, *sub)
                    && template.has_anchor()
                    && ours.contains_key(&(*major, *sub))
            })
            .map(|(code, _)| *code)
            .collect();
        whole_line.sort_unstable();
        Translator {
            upstream,
            ours,
            whole_line,
        }
    }

    /// `bytes` with every message line mapped, line ends kept.
    pub fn translate(&self, bytes: &[u8]) -> Translated {
        let mut out = Vec::with_capacity(bytes.len());
        let mut ambiguous = Vec::new();
        for raw in bytes.split_inclusive(|byte| *byte == b'\n') {
            let (line, end) = match raw.strip_suffix(b"\n") {
                Some(line) => (line, &b"\n"[..]),
                None => (raw, &b""[..]),
            };
            match self.line(line, &mut ambiguous) {
                Some(mapped) => out.extend_from_slice(&mapped),
                None => out.extend_from_slice(line),
            }
            out.extend_from_slice(end);
        }
        Translated {
            bytes: out,
            ambiguous,
        }
    }

    /// The mapped `line`, or `None` where it is not a message line.
    fn line(&self, line: &[u8], ambiguous: &mut Vec<String>) -> Option<Vec<u8>> {
        self.report_line(line, ambiguous)
            .or_else(|| self.debug_error_line(line, ambiguous))
            .or_else(|| self.whole_line(line, ambiguous))
    }

    /// Maps `text`, the part of a line that is the oracle's text of
    /// `code`, to this crate's text with the inserts the oracle's shows.
    fn text(&self, code: (u16, u16), text: &[u8], ambiguous: &mut Vec<String>) -> Option<Vec<u8>> {
        let theirs = self.upstream.get(&code)?;
        let ours = self.ours.get(&code)?;
        let splits = theirs.splits(text);
        let first = splits.first()?;
        let mapped = ours.render(text, first)?;
        let differs = splits[1..]
            .iter()
            .any(|split| ours.render(text, split).as_ref() != Some(&mapped));
        if differs {
            ambiguous.push(format!(
                "{}.{}: {}",
                code.0,
                code.1,
                String::from_utf8_lossy(text)
            ));
        }
        Some(mapped)
    }

    /// `Error N running F line L:  text`, `Error N:  text` and
    /// `Error N.M:  text`.
    fn report_line(&self, line: &[u8], ambiguous: &mut Vec<String>) -> Option<Vec<u8>> {
        let rest = line.strip_prefix(b"Error ")?;
        let (major, after) = number(rest)?;
        let replaced =
            |text: &[u8], mapped: &[u8]| [&line[..line.len() - text.len()], mapped].concat();
        if let Some(with_sub) = after.strip_prefix(b".") {
            let (sub, after) = number(with_sub)?;
            let text = after.strip_prefix(b":  ")?;
            let mapped = self.text((major, sub), text, ambiguous)?;
            return Some(replaced(text, &mapped));
        }
        let mut search = if after.starts_with(b" running ") {
            b" running ".len()
        } else if after.starts_with(b":  ") {
            0
        } else {
            return None;
        };
        // The separator can occur inside a path, so each one is tried.
        while let Some(found) = find(&after[search..], b":  ") {
            let text = &after[search + found + 3..];
            if let Some(mapped) = self.text((major, 0), text, ambiguous) {
                return Some(replaced(text, &mapped));
            }
            search += found + 1;
        }
        None
    }

    /// The error lines a debug pause prints: the debug prefix (101.4), then
    /// ` N:` or ` N.M:` and the text.
    fn debug_error_line(&self, line: &[u8], ambiguous: &mut Vec<String>) -> Option<Vec<u8>> {
        let prefix = self.upstream.get(&(101, 4))?.literal()?;
        let ours = self.ours.get(&(101, 4))?.literal()?;
        let rest = line.strip_prefix(prefix.as_slice())?.strip_prefix(b" ")?;
        let (major, after) = number(rest)?;
        let (code, after) = match after.strip_prefix(b".") {
            Some(with_sub) => {
                let (sub, after) = number(with_sub)?;
                ((major, sub), after)
            }
            None => ((major, 0), after),
        };
        let after = after.strip_prefix(b":")?;
        let blanks = after.iter().take_while(|byte| **byte == b' ').count();
        let text = &after[blanks..];
        let mapped = self.text(code, text, ambiguous)?;
        let head = &line[prefix.len()..line.len() - text.len()];
        Some([ours.as_slice(), head, &mapped].concat())
    }

    /// A line that is a whole catalogue text. Mapped only where exactly one
    /// code's text matches it.
    fn whole_line(&self, line: &[u8], ambiguous: &mut Vec<String>) -> Option<Vec<u8>> {
        let mut hits = self
            .whole_line
            .iter()
            .filter(|code| !self.upstream[*code].splits(line).is_empty());
        let code = *hits.next()?;
        if hits.next().is_some() {
            ambiguous.push(format!(
                "several texts match: {}",
                String::from_utf8_lossy(line)
            ));
            return None;
        }
        self.text(code, line, ambiguous)
    }
}

/// The decimal number at the start of `bytes` and what follows it.
fn number(bytes: &[u8]) -> Option<(u16, &[u8])> {
    let digits = bytes
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digits == 0 {
        return None;
    }
    let value = std::str::from_utf8(&bytes[..digits]).ok()?.parse().ok()?;
    Some((value, &bytes[digits..]))
}

/// The first place `needle` occurs in `haystack`.
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Where the oracle's message templates are: `$REXX_UPSTREAM` at run time,
/// else the path cargo set at build time.
fn upstream_root() -> PathBuf {
    std::env::var_os("REXX_UPSTREAM")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("REXX_UPSTREAM")))
}

/// The translator between the oracle's catalogue, read from the pinned
/// upstream tree, and this crate's own.
pub fn standard() -> &'static Translator {
    static STANDARD: OnceLock<Translator> = OnceLock::new();
    STANDARD.get_or_init(|| {
        let path = upstream_root().join("interpreter/messages/rexxmsg.xml");
        let xml = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "cannot read the oracle's message templates at {}: {e}. A comparison with the \
                 oracle maps its message lines to this crate's texts and needs them \
                 (REXX_UPSTREAM names the tree)",
                path.display()
            )
        });
        let upstream = interrexx_upstream::errors::parse_messages(&xml);
        let upstream: Vec<Entry<'_>> = upstream
            .iter()
            .map(|m| (m.major, m.sub, m.text.as_str()))
            .collect();
        let ours: Vec<Entry<'_>> = interrexx_inventory::errors::MESSAGES
            .iter()
            .map(|m| (m.major, m.sub, m.text))
            .collect();
        Translator::new(&upstream, &ours)
    })
}

/// `bytes`, one channel of an oracle run, with its message lines mapped to
/// this crate's texts. Both channels are mapped: the report, debug and
/// trace lines can be redirected to standard output. A line that more than
/// one split of its inserts could explain differently is named on standard
/// error.
pub fn oracle_output(bytes: &[u8]) -> Vec<u8> {
    if bytes.is_empty() {
        return Vec::new();
    }
    let translated = standard().translate(bytes);
    for line in &translated.ambiguous {
        eprintln!("message translation is ambiguous for {line}");
    }
    translated.bytes
}
