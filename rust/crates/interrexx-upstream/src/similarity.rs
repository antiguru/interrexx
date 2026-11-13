//! Compares the texts of interrexx's messages with the texts of the
//! catalogue they replace, per code. A hit is a text that is too close to
//! its catalogue counterpart. The comparison is local: nothing here
//! prints a catalogue text, only the code and the scores.

use crate::errors::Message;
use crate::messages::Text;

/// How close two texts are.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Score {
    /// Twice the longest common subsequence of the two texts' words over
    /// the sum of their word counts: 1.0 for the same words in the same
    /// order, 0.0 for no word in common.
    pub dice: f64,
    /// The longest run of consecutive words both texts hold.
    pub run: usize,
}

/// The words of `text`: lower-case runs of letters and digits. An insert
/// marker `&k` is not a word, and it ends a run.
pub fn words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' && chars.peek().is_some_and(char::is_ascii_digit) {
            chars.next();
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            // Keeps the insert from joining the words beside it into a run.
            words.push(String::new());
        } else if c.is_alphanumeric() {
            current.extend(c.to_lowercase());
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// The score of `ours` against `theirs`.
pub fn compare(ours: &str, theirs: &str) -> Score {
    let a = words(ours);
    let b = words(theirs);
    // An empty entry stands for an insert: it separates runs, but it is not
    // a word that two texts can share.
    let same = |x: &String, y: &String| !x.is_empty() && x == y;
    let mut subsequence = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    let mut contiguous = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    let mut run = 0;
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            if same(&a[i - 1], &b[j - 1]) {
                subsequence[i][j] = subsequence[i - 1][j - 1] + 1;
                contiguous[i][j] = contiguous[i - 1][j - 1] + 1;
                run = run.max(contiguous[i][j]);
            } else {
                subsequence[i][j] = subsequence[i - 1][j].max(subsequence[i][j - 1]);
            }
        }
    }
    let count = |w: &[String]| w.iter().filter(|x| !x.is_empty()).count();
    let total = count(&a) + count(&b);
    let dice = if total == 0 {
        0.0
    } else {
        // Word counts are far below 2^52, so the conversions are exact.
        (2 * subsequence[a.len()][b.len()]) as f64 / total as f64
    };
    Score { dice, run }
}

/// The limits above which a text counts as too close.
#[derive(Clone, Copy, Debug)]
pub struct Threshold {
    pub dice: f64,
    pub run: usize,
}

impl Default for Threshold {
    fn default() -> Threshold {
        Threshold { dice: 0.6, run: 6 }
    }
}

impl Threshold {
    pub fn exceeded_by(&self, score: Score) -> bool {
        score.dice >= self.dice || score.run >= self.run
    }
}

/// One code whose text is too close to the catalogue's.
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub major: u16,
    pub sub: u16,
    pub symbol: String,
    pub score: Score,
}

/// Every text of `ours` that has a catalogue counterpart (same code) and
/// reaches `threshold` against it, in `ours` order. Codes the catalogue
/// does not have are skipped: validation reports them.
pub fn hits(ours: &[Text], catalogue: &[Message], threshold: Threshold) -> Vec<Hit> {
    ours.iter()
        .filter_map(|t| {
            let theirs = catalogue
                .iter()
                .find(|m| m.major == t.major && m.sub == t.sub)?;
            let score = compare(&t.text, &theirs.text);
            threshold.exceeded_by(score).then(|| Hit {
                major: t.major,
                sub: t.sub,
                symbol: theirs.symbol.clone(),
                score,
            })
        })
        .collect()
}

/// The codes `allow` excuses, each with its reason: one `major.sub` and a
/// reason per line, tab separated, `#` comments.
pub fn parse_allow(contents: &str) -> Vec<((u16, u16), String)> {
    contents
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|line| {
            let (code, reason) = line
                .split_once('\t')
                .unwrap_or_else(|| panic!("allow line needs a reason: {line:?}"));
            let (major, sub) = code
                .split_once('.')
                .unwrap_or_else(|| panic!("allow code is `major.sub`: {code:?}"));
            let number = |n: &str| -> u16 {
                n.parse()
                    .unwrap_or_else(|_| panic!("allow code is `major.sub`: {code:?}"))
            };
            ((number(major), number(sub)), reason.to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests;
