//! Drops the message text of an error report from a run's output.
//!
//! The text of a message is not specified (`r[conv.error.text]`): its code and
//! subcode are. A comparison of this crate's output with the oracle's applies
//! [`cut_error_texts`] to both sides, so that a difference in wording is not a
//! difference in behaviour. A file of its own, so that a test binary without
//! the rest of `support` can include it with `#[path]`.

/// `bytes` with the message text of every error report dropped, so that only
/// an error's code and subcode are observable (`r[conv.error.text]`). An
/// error is reported by two lines,
/// `Error <code> running <file> line <n>:  <text>` and
/// `Error <code>.<subcode>:  <text>` (`Error <code>:  <text>` where it fails
/// before a program runs), and a line typed at an interactive
/// trace stop is reported by two lines that start with `+++` and hold the
/// code and the code with its subcode as the first word that ends in a colon.
/// Each keeps its text up to the colon that ends the numbers and loses what
/// follows. Every other line is a separate comparison and stays exact. The
/// comparisons between this crate and the oracle apply it to both sides, so
/// that a difference in wording is not a difference in behaviour.
pub fn cut_error_texts(bytes: &[u8]) -> Vec<u8> {
    let had_trailing_newline = bytes.last() == Some(&b'\n');
    let mut lines: Vec<&[u8]> = bytes.split(|&b| b == b'\n').collect();
    if had_trailing_newline {
        lines.pop();
    }
    let out: Vec<Vec<u8>> = lines.into_iter().map(error_report_without_text).collect();
    let mut joined = out.join(&b'\n');
    if had_trailing_newline {
        joined.push(b'\n');
    }
    joined
}

/// `line` up to and including the colon that ends an error report's numbers,
/// when it is one of the two report lines; otherwise `line` unchanged.
fn error_report_without_text(line: &[u8]) -> Vec<u8> {
    if line.starts_with(b"+++") {
        return debug_error_without_text(line);
    }
    let Some(rest) = line.strip_prefix(b"Error ") else {
        return line.to_vec();
    };
    let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
    if digits == 0 {
        return line.to_vec();
    }
    let after = &rest[digits..];
    let numbers_end = if after.starts_with(b":") {
        // `Error <code>:`, the report of a failure before any program runs.
        Some(0)
    } else if after.starts_with(b".") {
        // `Error <code>.<subcode>:`
        let sub = after[1..].iter().take_while(|b| b.is_ascii_digit()).count();
        (sub > 0 && after.get(1 + sub) == Some(&b':')).then_some(1 + sub)
    } else if after.starts_with(b" running ") {
        // `Error <code> running <file> line <n>:`; the file name may hold a
        // colon, so the colon wanted is the one after ` line <n>`.
        after
            .windows(b" line ".len())
            .enumerate()
            .filter(|(_, w)| *w == b" line ")
            .map(|(at, _)| at + b" line ".len())
            .filter_map(|from| {
                let n = after[from..]
                    .iter()
                    .take_while(|b| b.is_ascii_digit())
                    .count();
                (n > 0 && after.get(from + n) == Some(&b':')).then_some(from + n)
            })
            .next_back()
    } else {
        None
    };
    let numbers_end = numbers_end.or_else(|| {
        // The lineless library form, `Error <code> running REXX:`.
        after
            .starts_with(b" running REXX:")
            .then_some(b" running REXX".len())
    });
    match numbers_end {
        Some(end) => line[..6 + digits + end + 1].to_vec(),
        None => line.to_vec(),
    }
}

/// `line` up to and including its first word that ends in a colon, when it
/// starts with `+++` and that word is a code (`42:`) or a code and subcode
/// (`42.3:`), as `r[env.debugging.input.error]` shows an error reported at a
/// stop; otherwise `line` unchanged. The notice is not an error line and has
/// no such word.
fn debug_error_without_text(line: &[u8]) -> Vec<u8> {
    let mut at = 0;
    for word in line.split(|&b| b == b' ') {
        let end = at + word.len();
        if let Some(number) = word.strip_suffix(b":")
            && is_code(number)
        {
            return line[..end].to_vec();
        }
        if word.ends_with(b":") {
            // The first colon-ended word is not a code: not an error line.
            break;
        }
        at = end + 1;
    }
    line.to_vec()
}

/// Whether `word` is `<digits>` or `<digits>.<digits>`.
fn is_code(word: &[u8]) -> bool {
    let mut parts = word.split(|&b| b == b'.');
    let digits = |part: Option<&[u8]>| {
        part.is_some_and(|p| !p.is_empty() && p.iter().all(u8::is_ascii_digit))
    };
    digits(parts.next())
        && parts.next().is_none_or(|sub| digits(Some(sub)))
        && parts.next().is_none()
}

#[cfg(test)]
mod tests {
    /// Two reports of one error that differ only in message text compare
    /// equal; a different code, subcode or program does not, and a line
    /// that is not a report is kept whole.
    #[test]
    fn cut_error_texts_drops_message_text_and_nothing_else() {
        use super::cut_error_texts;
        let one = b"     2 *-* say x~put(1)\nError 93 running /t/x.rex line 2:  Invented major text.\nError 93.902:  Invented sub text, 2 shown.\n";
        let other = b"     2 *-* say x~put(1)\nError 93 running /t/x.rex line 2:  Wording differs.\nError 93.902:  Another text: with a colon.\n";
        assert_eq!(cut_error_texts(one), cut_error_texts(other));
        let subcode = b"     2 *-* say x~put(1)\nError 93 running /t/x.rex line 2:  Invented major text.\nError 93.903:  Invented sub text, 2 shown.\n";
        assert_ne!(cut_error_texts(one), cut_error_texts(subcode));
        let code = b"     2 *-* say x~put(1)\nError 94 running /t/x.rex line 2:  Invented major text.\nError 93.902:  Invented sub text, 2 shown.\n";
        assert_ne!(cut_error_texts(one), cut_error_texts(code));
        let line = b"     2 *-* say x~put(1)\nError 93 running /t/x.rex line 3:  Invented major text.\nError 93.902:  Invented sub text, 2 shown.\n";
        assert_ne!(cut_error_texts(one), cut_error_texts(line));
        let traceback = b"     3 *-* say x~put(1)\nError 93 running /t/x.rex line 2:  Invented major text.\nError 93.902:  Invented sub text, 2 shown.\n";
        assert_ne!(cut_error_texts(one), cut_error_texts(traceback));
        let plain =
            b"say with Error 93: text\nErrors: 5\nError: 93 is no code\n Error 93:  indented\n";
        assert_eq!(cut_error_texts(plain), plain.to_vec());
        // The report of a failure before any program runs has a code only.
        assert_eq!(
            cut_error_texts(b"Error 3:  Some wording.\nError 3.901:  Other.\n"),
            b"Error 3:\nError 3.901:\n".to_vec()
        );
        let lineless = b"Error 98 running REXX:  Something\n";
        assert_eq!(
            cut_error_texts(lineless),
            b"Error 98 running REXX:\n".to_vec()
        );
        // A line typed at an interactive stop is reported with `+++` lines
        // whose first colon-ended word is the code, then the code and subcode.
        let debug = b"+++ Interactive trace.  Error 42:   Some wording.\n+++ Interactive trace.  Error 42.3:  Other: wording.\n";
        assert_eq!(
            cut_error_texts(debug),
            b"+++ Interactive trace.  Error 42:\n+++ Interactive trace.  Error 42.3:\n".to_vec()
        );
        let reworded = b"+++ Stopped.  Error 42:   Different wording.\n+++ Stopped.  Error 42.3:  Words: more.\n";
        assert_eq!(
            cut_error_texts(reworded),
            b"+++ Stopped.  Error 42:\n+++ Stopped.  Error 42.3:\n".to_vec()
        );
        let other_subcode = b"+++ Interactive trace.  Error 42:   Some wording.\n+++ Interactive trace.  Error 42.4:  Other: wording.\n";
        assert_ne!(cut_error_texts(debug), cut_error_texts(other_subcode));
        // The notice has no code, and a `+++` line whose first colon-ended
        // word is not a code is kept whole.
        let notice = b"+++ Interactive trace.  Type: say or exit.\n+++ note: 5 and 6:  x\n";
        assert_eq!(cut_error_texts(notice), notice.to_vec());
    }
}
