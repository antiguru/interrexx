//! Drops the message text of an error report, and what a build may word or
//! level differently, from a run's error stream.
//!
//! The text of a message is not specified (`r[conv.error.text]`): its code and
//! subcode are. Whether a built-in method is a level of the traceback is not
//! specified either (`r[class.stackframe.builtin]`). A comparison of this
//! crate's output with the oracle's applies [`cut_error_texts`] to both sides'
//! stderr, so that a difference in wording, or in which built-in levels a
//! build shows, is not a difference in behaviour. Standard output is compared
//! whole: a program that prints a message prints its code instead. The cut
//! recognises a line by its structure, which the specification fixes, and
//! never by a wording. A file of its own, so that a test binary without the
//! rest of `support` can include it with `#[path]`.

/// `bytes`, a run's stderr, with the message text of every error report
/// dropped, so that only an error's code and subcode are observable
/// (`r[conv.error.text]`). An
/// error is reported by two lines,
/// `Error <code> running <file> line <n>:  <text>` and
/// `Error <code>.<subcode>:  <text>` (`Error <code>:  <text>` where it fails
/// before a program runs); each keeps its text up to the colon that ends the
/// numbers and loses what follows. A line typed at an interactive trace stop
/// is reported by two lines that start with `+++` and hold the code and the
/// code with its subcode as the first word that ends in a colon
/// (`r[env.debugging.input.error]`): each keeps `+++` and that word. The
/// notice of interactive tracing is the other line that starts with `+++`
/// and holds no word that ends in a colon (`r[env.debugging.notice]`): it
/// keeps `+++`. The announcement of a routine or method in trace output, a
/// line of blanks where a line number would stand and then `>I> ` or `<I< `
/// (`r[instr.trace.tags]`), keeps the flag. A line that shows a built-in
/// method or routine as a level of a traceback is dropped whole, because a
/// build may not have that level (`r[class.stackframe.builtin]`): six blanks
/// where a line number would stand, `*-* ` and a text that names the method
/// or routine in double quotes and ends with a period, as every text that
/// names one does (`r[conv.error.text]`). The level of a method or routine
/// written in Rexx has a number (`r[class.stackframe.traceline]`) and stays,
/// and so does an unnumbered line of any other text. Every other line is a
/// separate comparison and stays exact.
pub fn cut_error_texts(bytes: &[u8]) -> Vec<u8> {
    let had_trailing_newline = bytes.last() == Some(&b'\n');
    let mut lines: Vec<&[u8]> = bytes.split(|&b| b == b'\n').collect();
    if had_trailing_newline {
        lines.pop();
    }
    let out: Vec<Vec<u8>> = lines
        .into_iter()
        .filter(|line| !is_built_in_frame_line(line))
        .map(|line| error_report_without_text(&announcement_without_text(line)))
        .collect();
    let mut joined = out.join(&b'\n');
    if had_trailing_newline {
        joined.push(b'\n');
    }
    joined
}

/// The blanks that stand for the six columns of a line number, and the blank
/// after them, in a trace line (`r[instr.trace.line]`).
const NO_LINE_NUMBER: &[u8] = b"       ";

/// Whether `line` has the structure of the traceback line of a built-in
/// method or routine: [`NO_LINE_NUMBER`], `*-* `, and a text that holds a
/// non-empty name in double quotes and ends with a period. A numbered line,
/// a trace line, and an unnumbered line of other text are not.
fn is_built_in_frame_line(line: &[u8]) -> bool {
    let Some(text) = line
        .strip_prefix(NO_LINE_NUMBER)
        .and_then(|rest| rest.strip_prefix(b"*-* "))
    else {
        return false;
    };
    let names = text
        .split(|&b| b == b'"')
        .skip(1)
        .step_by(2)
        .any(|quoted| !quoted.is_empty());
    let quotes = text.iter().filter(|&&b| b == b'"').count();
    names && quotes.is_multiple_of(2) && text.ends_with(b".")
}

/// `line` without the text after the flag of a trace announcement (blanks,
/// then `>I> ` or `<I< `); any other line unchanged.
fn announcement_without_text(line: &[u8]) -> Vec<u8> {
    let blanks = line.iter().take_while(|&&b| b == b' ').count();
    let rest = &line[blanks..];
    if blanks > 0 && (rest.starts_with(b">I> ") || rest.starts_with(b"<I< ")) {
        return line[..blanks + 3].to_vec();
    }
    line.to_vec()
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

/// `+++` and the first word of `line` that ends in a colon, when `line`
/// starts with `+++` and that word is a code (`42:`) or a code and subcode
/// (`42.3:`), as `r[env.debugging.input.error]` shows an error reported at a
/// stop; `+++` alone when `line` is the notice, which has no word that ends
/// in a colon (`r[env.debugging.notice]`); otherwise `line` unchanged.
fn debug_error_without_text(line: &[u8]) -> Vec<u8> {
    let mut words = line.split(|&b| b == b' ').skip(1);
    match words.find(|word| word.ends_with(b":")) {
        None => b"+++".to_vec(),
        Some(word) if is_code(&word[..word.len() - 1]) => {
            let mut out = b"+++ ".to_vec();
            out.extend_from_slice(word);
            out
        }
        Some(_) => line.to_vec(),
    }
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
        // whose first colon-ended word is the code, then the code and subcode;
        // the words before it are not compared.
        let debug = b"+++ Some words.  Error 42:   Some wording.\n+++ Some words.  Error 42.3:  Other: wording.\n";
        assert_eq!(cut_error_texts(debug), b"+++ 42:\n+++ 42.3:\n".to_vec());
        let reworded = b"+++ Stopped.  Error 42:   Different wording.\n+++ Stopped.  Error 42.3:  Words: more.\n";
        assert_eq!(cut_error_texts(reworded), cut_error_texts(debug));
        let other_subcode = b"+++ Some words.  Error 42:   Some wording.\n+++ Some words.  Error 42.4:  Other: wording.\n";
        assert_ne!(cut_error_texts(debug), cut_error_texts(other_subcode));
        // The notice has no colon-ended word and is cut to `+++`; a `+++`
        // line whose first colon-ended word is not a code is kept whole.
        let notice = b"+++ Tracing is on; end it somehow.\n+++ note: 5 and 6:  x\n";
        assert_eq!(
            cut_error_texts(notice),
            b"+++\n+++ note: 5 and 6:  x\n".to_vec()
        );
        // An indented `+++` line is a command trace and stays.
        let command = b"       +++ \"cmd\"\n";
        assert_eq!(cut_error_texts(command), command.to_vec());
    }

    /// Two outputs that differ only by the traceback line of a built-in
    /// method or routine compare equal, whether the build shows the level or
    /// not and however it words it; a difference in the line of Rexx code
    /// does not.
    #[test]
    fn cut_error_texts_drops_built_in_frame_lines_and_nothing_else() {
        use super::cut_error_texts;
        let without = b"     3 *-* x~put(1)\n     1 *-* call p\nError 93 running /t/x.rex line 3:  Text.\nError 93.902:  Text.\n";
        let with = b"       *-* Some words \"SEND\" and \"Message\".\n     3 *-* x~put(1)\n       *-* Other words \"R\".\n     1 *-* call p\nError 93 running /t/x.rex line 3:  Text.\nError 93.902:  Text.\n";
        assert_eq!(cut_error_texts(with), cut_error_texts(without));
        // A different line of Rexx code, or a different level count of Rexx
        // code, still differs.
        let other_line = b"     4 *-* x~put(1)\n     1 *-* call p\nError 93 running /t/x.rex line 3:  Text.\nError 93.902:  Text.\n";
        assert_ne!(cut_error_texts(with), cut_error_texts(other_line));
        let fewer = b"     3 *-* x~put(1)\nError 93 running /t/x.rex line 3:  Text.\nError 93.902:  Text.\n";
        assert_ne!(cut_error_texts(with), cut_error_texts(fewer));
        // A numbered line is Rexx code, and so is a line without the blanks.
        let numbered = b"     3 *-* x\n*-* y\n";
        assert_eq!(cut_error_texts(numbered), numbered.to_vec());
        // An unnumbered line that names nothing in quotes, or does not end
        // with a period, or is indented otherwise, is not a built-in level:
        // an extra or a missing one is a difference.
        let clause = b"     3 *-* say x\n";
        for extra in [
            &b"       *-* say y\n"[..],
            b"       *-* say \"y\"\n",
            b"       *-* say \"\".\n",
            b"        *-* Words \"R\".\n",
            b"      *-* Words \"R\".\n",
        ] {
            let more = [&clause[..], extra].concat();
            assert_ne!(
                cut_error_texts(&more),
                cut_error_texts(clause),
                "{}",
                String::from_utf8_lossy(extra)
            );
        }
        // The last line may have no newline.
        assert_eq!(
            cut_error_texts(b"       *-* Some words \"R\".\n     1 *-* x"),
            b"     1 *-* x".to_vec()
        );
    }

    /// The text of a trace announcement is not compared, its flag and
    /// indentation are; other trace lines are.
    #[test]
    fn cut_error_texts_drops_the_text_of_an_announcement() {
        use super::cut_error_texts;
        let one = b"       >I> Some words \"R\".\n       <I< More words.\n";
        let other = b"       >I> Other words.\n       <I< Else.\n";
        assert_eq!(cut_error_texts(one), b"       >I>\n       <I<\n".to_vec());
        assert_eq!(cut_error_texts(one), cut_error_texts(other));
        let flag = b"       <I< Other words.\n       <I< Else.\n";
        assert_ne!(cut_error_texts(one), cut_error_texts(flag));
        let value = b"       >V>   X => \"1\"\n";
        assert_eq!(cut_error_texts(value), value.to_vec());
    }
}
