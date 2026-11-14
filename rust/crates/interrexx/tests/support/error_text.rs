//! Drops the message text of an error report, and the frame lines of built-in
//! methods, from a run's output.
//!
//! The text of a message is not specified (`r[conv.error.text]`): its code and
//! subcode are. Whether a built-in method is a level of the traceback is not
//! specified either (`r[class.stackframe.builtin]`). A comparison of this
//! crate's output with the oracle's applies [`cut_error_texts`] to both sides,
//! so that a difference in wording, or in which built-in levels a build shows,
//! is not a difference in behaviour. The text of a level with no source and
//! of a trace line that announces a routine or method is ours on this crate's
//! side and the oracle's on the other; [`cut_error_texts`] writes the
//! oracle's as ours, so that the names they carry are still compared. A file
//! of its own, so that a test binary without the rest of `support` can
//! include it with `#[path]`.

/// `bytes` with the message text of every error report dropped, so that only
/// an error's code and subcode are observable (`r[conv.error.text]`). An
/// error is reported by two lines,
/// `Error <code> running <file> line <n>:  <text>` and
/// `Error <code>.<subcode>:  <text>` (`Error <code>:  <text>` where it fails
/// before a program runs), and a line typed at an interactive
/// trace stop is reported by two lines that start with `+++` and hold the
/// code and the code with its subcode as the first word that ends in a colon.
/// Each keeps its text up to the colon that ends the numbers and loses what
/// follows. A line that shows a built-in method or routine as a level of a
/// traceback is dropped whole, because a build may not have that level
/// (`r[class.stackframe.builtin]`): the line that starts with blanks only, then
/// `*-*`, and then the text of a built-in method's or routine's level
/// ([`is_built_in_frame_line`]). The level of a method or routine written in
/// Rexx, the library's included, is a numbered line and stays; where it has
/// no source, or where a trace line announces it, its text in the oracle's
/// wording is written in this crate's ([`frame_text_as_ours`]), and so is the
/// notice of interactive tracing, the prefix of its error lines and a
/// message of a utility program ([`line_text_as_ours`]). Every other
/// line is a separate comparison and stays exact. The
/// comparisons between this crate and the oracle apply it to both sides, so
/// that a difference in wording is not a difference in behaviour.
pub fn cut_error_texts(bytes: &[u8]) -> Vec<u8> {
    let had_trailing_newline = bytes.last() == Some(&b'\n');
    let mut lines: Vec<&[u8]> = bytes.split(|&b| b == b'\n').collect();
    if had_trailing_newline {
        lines.pop();
    }
    let out: Vec<Vec<u8>> = lines
        .into_iter()
        .filter(|line| !is_built_in_frame_line(line))
        .map(|line| {
            frame_text_as_ours(line)
                .or_else(|| line_text_as_ours(line))
                .unwrap_or_else(|| line.to_vec())
        })
        .map(|line| error_report_without_text(&line))
        .collect();
    let mut joined = out.join(&b'\n');
    if had_trailing_newline {
        joined.push(b'\n');
    }
    joined
}

/// Whether `line` is the traceback line of a built-in method or routine:
/// blanks where a line number would stand, `*-* ` and then the text of
/// message 101.20 or 101.21 with its inserts, as this crate writes it, or the
/// oracle's `Compiled method "<name>" with scope "<scope>".` or
/// `Compiled routine "<name>".`. A numbered line, a trace line and any other
/// text are not.
fn is_built_in_frame_line(line: &[u8]) -> bool {
    let blanks = line.iter().take_while(|&&b| b == b' ').count();
    let Some(rest) = line[blanks..].strip_prefix(b"*-* ") else {
        return false;
    };
    if blanks == 0 {
        return false;
    }
    let ours = [(101, 20), (101, 21)].into_iter().any(|(major, sub)| {
        interrexx_inventory::errors::lookup(major, sub)
            .is_some_and(|message| matches_template(rest, message.text))
    });
    ours || matches_template(rest, "Compiled method \"&1\" with scope \"&2\".")
        || matches_template(rest, "Compiled routine \"&1\".")
}

/// Whether `text` is `template` with each `&k` replaced by some bytes.
fn matches_template(text: &[u8], template: &str) -> bool {
    inserts_of(text, template).is_some()
}

/// The bytes that stand for each `&k` of `template` in `text`, by `k`, when
/// `text` is `template` with its inserts filled in: it starts with the
/// template's first literal part, ends with its last, and holds the parts
/// between in order, each taken where it first occurs.
fn inserts_of<'t>(text: &'t [u8], template: &str) -> Option<Vec<(u8, &'t [u8])>> {
    let mut parts: Vec<&[u8]> = Vec::new();
    let mut positions = Vec::new();
    let mut rest = template.as_bytes();
    while let Some(at) = rest
        .windows(2)
        .position(|pair| pair[0] == b'&' && pair[1].is_ascii_digit())
    {
        parts.push(&rest[..at]);
        positions.push(rest[at + 1] - b'0');
        rest = &rest[at + 2..];
    }
    parts.push(rest);
    let (first, last) = (parts[0], parts[parts.len() - 1]);
    if parts.len() == 1 {
        return (text == first).then(Vec::new);
    }
    let middle = text.strip_prefix(first)?.strip_suffix(last)?;
    let mut values = Vec::new();
    let mut from = 0;
    for (index, part) in parts[1..parts.len() - 1].iter().enumerate() {
        let at = middle[from..]
            .windows(part.len())
            .position(|w| w == *part)?;
        values.push((positions[index], &middle[from..from + at]));
        from += at + part.len();
    }
    values.push((positions[positions.len() - 1], &middle[from..]));
    Some(values)
}

/// The oracle's wording of the texts of a level with no source (101.22 to
/// 101.26) and of the announcement of a routine or method in trace output
/// (101.18, 101.19), with the message each stands for.
const ORACLE_FRAME_TEXTS: &[((u16, u16), &str)] = &[
    ((101, 18), "Routine \"&1\" in package \"&2\"."),
    (
        (101, 19),
        "Method \"&1\" with scope \"&2\" in package \"&3\".",
    ),
    ((101, 22), "Package \"&1\" (no source available)."),
    ((101, 23), "Interpreter code (no source available)."),
    (
        (101, 24),
        "Method &1 with scope \"&2\" in package \"&3\" (no source available).",
    ),
    (
        (101, 25),
        "Routine &1 in package \"&2\" (no source available).",
    ),
    ((101, 26), "Program in \"&1\" (no source available)."),
];

/// The oracle's wording of texts that make up a whole line: the notice of
/// interactive tracing (101.5) and the messages of the utility programs
/// (999), with the message each stands for.
const ORACLE_LINE_TEXTS: &[((u16, u16), &str)] = &[
    (
        (101, 5),
        "+++ Interactive trace. \"Trace Off\" to end debug, ENTER to continue. +++",
    ),
    ((999, 0), "The RXSUBCOM parameters are incorrect."),
    ((999, 1), "The RXSUBCOM REGISTER parameters are incorrect."),
    ((999, 2), "The RXSUBCOM DROP parameters are incorrect."),
    ((999, 3), "The RXSUBCOM LOAD parameters are incorrect."),
    ((999, 4), "The REXX rxapi queuing system is not available."),
    ((999, 5), "The size of the data is incorrect."),
    ((999, 6), "Storage for data queues is exhausted."),
    ((999, 7), "The name &1 is not a valid queue name."),
    ((999, 8), "The queue access mode is not correct."),
    ((999, 10), "The queue &1 does not exist."),
    ((999, 11), "The RXSUBCOM QUERY parameters are incorrect."),
    ((999, 12), "The rexxc command parameters are incorrect."),
    (
        (999, 13),
        "Output file name must be different from input file name.",
    ),
    ((999, 14), "SYNTAX: rexxc inputfile [outputfile] [-s] [-e]"),
    (
        (999, 15),
        "Without outputfile rexxc only performs a syntax check.",
    ),
    ((999, 16), "The syntax of the command is incorrect."),
    (
        (999, 17),
        "System error occurred while processing the command.",
    ),
];

/// The oracle's wording of the prefix of the error lines of interactive
/// tracing (101.4).
const ORACLE_DEBUG_ERROR_PREFIX: &[u8] = b"+++ Interactive trace.  Error ";

/// `values`, each `(k, value)` placed at position `k`, as the list
/// [`interrexx_inventory::errors::substitute`] takes.
fn by_position(inserts: Vec<(u8, &[u8])>) -> Vec<&[u8]> {
    let mut values: Vec<&[u8]> = Vec::new();
    for (k, value) in inserts {
        let slot = usize::from(k) - 1;
        if values.len() <= slot {
            values.resize(slot + 1, b"");
        }
        values[slot] = value;
    }
    values
}

/// `line` in this crate's wording, when the whole line is one of
/// [`ORACLE_LINE_TEXTS`] or starts with [`ORACLE_DEBUG_ERROR_PREFIX`];
/// `None` for any other line.
fn line_text_as_ours(line: &[u8]) -> Option<Vec<u8>> {
    if let Some(rest) = line.strip_prefix(ORACLE_DEBUG_ERROR_PREFIX) {
        let mut out = interrexx_inventory::errors::lookup(101, 4)?
            .text
            .as_bytes()
            .to_vec();
        out.push(b' ');
        out.extend_from_slice(rest);
        return Some(out);
    }
    ORACLE_LINE_TEXTS
        .iter()
        .find_map(|&((major, sub), template)| {
            let values = by_position(inserts_of(line, template)?);
            let ours = interrexx_inventory::errors::lookup(major, sub)?.text;
            Some(interrexx_inventory::errors::substitute(ours, &values))
        })
}

/// `line` with its text in this crate's wording, when it is a traceback line
/// (a line number or blanks, `*-* ` and any indentation) or a `>I>` or `<I<`
/// trace line whose text is one of [`ORACLE_FRAME_TEXTS`]; `None` for any
/// other line. The inserts are kept, so a different name, scope or package
/// is still a difference.
fn frame_text_as_ours(line: &[u8]) -> Option<Vec<u8>> {
    let blanks = line.iter().take_while(|&&b| b == b' ').count();
    let digits = line[blanks..]
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .count();
    let after = &line[blanks + digits..];
    let lead = if let Some(rest) = after.strip_prefix(b" *-* ") {
        let indent = rest.iter().take_while(|&&b| b == b' ').count();
        blanks + digits + b" *-* ".len() + indent
    } else if blanks > 0
        && digits == 0
        && (after.starts_with(b">I> ") || after.starts_with(b"<I< "))
    {
        blanks + b">I> ".len()
    } else if blanks > 0 && digits == 0 && after.starts_with(b"*-* ") {
        blanks + b"*-* ".len()
    } else {
        return None;
    };
    let text = &line[lead..];
    ORACLE_FRAME_TEXTS
        .iter()
        .find_map(|&((major, sub), template)| {
            let values = by_position(inserts_of(text, template)?);
            let ours = interrexx_inventory::errors::lookup(major, sub)?.text;
            let mut out = line[..lead].to_vec();
            out.extend(interrexx_inventory::errors::substitute(ours, &values));
            Some(out)
        })
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
        // The oracle's prefix of these lines is written as this crate's.
        let prefix = interrexx_inventory::errors::expand("⟨101.4⟩");
        assert_eq!(
            cut_error_texts(debug),
            format!("{prefix} 42:\n{prefix} 42.3:\n").into_bytes()
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

    /// Two outputs that differ only by the traceback line of a built-in
    /// method or routine compare equal, whether the build shows the level or
    /// not; a difference in the line of Rexx code does not.
    #[test]
    fn cut_error_texts_drops_built_in_frame_lines_and_nothing_else() {
        use super::cut_error_texts;
        let without = b"     3 *-* x~put(1)\n     1 *-* call p\nError 93 running /t/x.rex line 3:  Text.\nError 93.902:  Text.\n";
        let with = b"       *-* Compiled method \"PUT\" with scope \"Directory\".\n     3 *-* x~put(1)\n       *-* Compiled routine \"SYSSLEEP\".\n     1 *-* call p\nError 93 running /t/x.rex line 3:  Text.\nError 93.902:  Text.\n";
        assert_eq!(cut_error_texts(with), cut_error_texts(without));
        // This crate's own lines, the texts of 101.20 and 101.21, too.
        let mut ours = format!(
            "       *-* {}\n     3 *-* x~put(1)\n       *-* {}\n",
            interrexx_inventory::errors::render(
                interrexx_inventory::errors::E101_20,
                ["PUT", "Directory"]
            ),
            interrexx_inventory::errors::render(interrexx_inventory::errors::E101_21, ["SYSSLEEP"]),
        )
        .into_bytes();
        ours.extend_from_slice(
            b"     1 *-* call p\nError 93 running /t/x.rex line 3:  Text.\nError 93.902:  Text.\n",
        );
        assert_eq!(cut_error_texts(&ours), cut_error_texts(without));
        // A different line of Rexx code, or a different level count of Rexx
        // code, still differs.
        let other_line = b"     4 *-* x~put(1)\n     1 *-* call p\nError 93 running /t/x.rex line 3:  Text.\nError 93.902:  Text.\n";
        assert_ne!(cut_error_texts(with), cut_error_texts(other_line));
        let fewer = b"     3 *-* x~put(1)\nError 93 running /t/x.rex line 3:  Text.\nError 93.902:  Text.\n";
        assert_ne!(cut_error_texts(with), cut_error_texts(fewer));
        // A numbered line is Rexx code, even when it reads like a built-in
        // level, and so is a line without the blanks.
        let numbered = b"     3 *-* Compiled routine \"R\".\n*-* Compiled routine \"R\".\n";
        assert_eq!(cut_error_texts(numbered), numbered.to_vec());
        // The last line may have no newline.
        assert_eq!(
            cut_error_texts(b"       *-* Compiled routine \"R\".\n     1 *-* x"),
            b"     1 *-* x".to_vec()
        );
    }

    /// A level with no source, and a trace line announcing a routine or a
    /// method, compare equal in the oracle's wording and in this crate's; a
    /// different name, scope or package, or line, still differs.
    #[test]
    fn cut_error_texts_writes_the_oracles_frame_texts_as_ours() {
        use super::cut_error_texts;
        use interrexx_inventory::errors::{E101_18, E101_19, E101_24, E101_25, render};
        let theirs = b"  1459 *-*   Method UNKNOWN with scope \"Monitor\" in package \"REXX\" (no source available).\n       >I> Routine \"R\" in package \"/t/x.rex\".\n       <I< Method \"M\" with scope \"K\" in package \"/t/x.rex\".\n    12 *-* Routine R in package \"/t/y.rex\" (no source available).\n";
        let ours = format!(
            "  1459 *-*   {}\n       >I> {}\n       <I< {}\n    12 *-* {}\n",
            render(E101_24, ["UNKNOWN", "Monitor", "REXX"]),
            render(E101_18, ["R", "/t/x.rex"]),
            render(E101_19, ["M", "K", "/t/x.rex"]),
            render(E101_25, ["R", "/t/y.rex"]),
        );
        assert_eq!(cut_error_texts(theirs), cut_error_texts(ours.as_bytes()));
        assert_eq!(cut_error_texts(ours.as_bytes()), ours.as_bytes().to_vec());
        let text = String::from_utf8(theirs.to_vec()).unwrap();
        for (from, to) in [
            ("UNKNOWN", "OTHER"),
            ("\"Monitor\"", "\"Monitor2\""),
            ("\"/t/x.rex\".\n       <I<", "\"/t/z.rex\".\n       <I<"),
            ("  1459", "  1460"),
        ] {
            let changed = text.replacen(from, to, 1);
            assert_ne!(
                cut_error_texts(changed.as_bytes()),
                cut_error_texts(ours.as_bytes()),
                "{changed}"
            );
        }
        // The notice of interactive tracing and a utility program's message
        // are whole lines.
        let theirs = b"+++ Interactive trace. \"Trace Off\" to end debug, ENTER to continue. +++\nThe queue Q1 does not exist.\n";
        let ours = interrexx_inventory::errors::expand("⟨101.5⟩\n⟨999.10¦Q1⟩\n");
        assert_eq!(cut_error_texts(theirs), ours.as_bytes().to_vec());
        let other_queue = b"The queue Q2 does not exist.\n";
        assert_ne!(
            cut_error_texts(other_queue),
            cut_error_texts(&theirs[theirs.len() - 29..])
        );
        // A clause that reads like one of the texts but is not a traceback or
        // announcement line stays.
        let said = b"say Routine R in package \"x\" (no source available).\n";
        assert_eq!(cut_error_texts(said), said.to_vec());
    }
}
