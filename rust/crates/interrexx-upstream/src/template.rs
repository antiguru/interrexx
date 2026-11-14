//! A catalogue text split at its `&k` markers, and the matching of a line
//! against it: the line's inserts are the pieces of it that fill the markers.
//!
//! The harness that maps the oracle's message lines to interrexx's texts and
//! the corpus tool that records a message's inserts both read a printed text
//! this way.

/// The highest insert position a template names, `&1` to `&9`.
pub const INSERTS: usize = 9;

/// The most splits of one line the matcher keeps. Two are enough to know a
/// line is ambiguous; the cap bounds the search on a line with many inserts.
const MAX_SPLITS: usize = 8;

/// One piece of a template: text, or the place insert `k` goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    Text(Vec<u8>),
    Insert(usize),
}

/// A catalogue text split at its `&k` markers.
#[derive(Clone, Debug)]
pub struct Template {
    pieces: Vec<Piece>,
}

/// Where each insert sits in a matched line: entry `k` is a byte range of
/// the line, or `None` where the template does not show insert `k`.
pub type Split = [Option<(usize, usize)>; INSERTS + 1];

impl Template {
    pub fn parse(text: &str) -> Template {
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
    pub fn has_anchor(&self) -> bool {
        self.pieces.iter().any(|piece| match piece {
            Piece::Text(text) => text.iter().any(|byte| !byte.is_ascii_whitespace()),
            Piece::Insert(_) => false,
        })
    }

    /// Every way (up to [`MAX_SPLITS`]) the whole of `line` is this template
    /// with its inserts filled in, shortest insert first.
    pub fn splits(&self, line: &[u8]) -> Vec<Split> {
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
    pub fn render(&self, line: &[u8], split: &Split) -> Option<Vec<u8>> {
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
    pub fn literal(&self) -> Option<Vec<u8>> {
        self.render(&[], &[None; INSERTS + 1])
    }
}
