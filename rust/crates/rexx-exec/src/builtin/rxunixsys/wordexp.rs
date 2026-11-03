//! `SysWordexp`: POSIX word expansion with no flags, as glibc's `wordexp`
//! performs it, by the shell glibc itself runs for a command substitution.
//!
//! [`script`] first scans the text as `wordexp` does and refuses what it
//! refuses: an unquoted newline, `|`, `&`, `;`, `<`, `>`, `(`, `)`, `{` or
//! `}` (`WRDE_BADCHAR`), and an unterminated quote, backquote, `$(`, `$((`
//! or `${`, or a backslash ending the text (`WRDE_SYNTAX`). What it accepts
//! cannot hold a second command outside a command substitution, so it is
//! handed to `/bin/sh` as the arguments of `set --`, and the shell performs
//! the expansions -- tilde, parameter, command, arithmetic, field splitting,
//! pathname and quote removal -- and prints the fields. A `#` starting a word
//! is not a comment to `wordexp`, so it is escaped first. An expansion the
//! shell fails (`${X?}`, `$((1/0))`) is the third refusal.
//!
//! The oracle crashes on every refusal (measured: SIGSEGV for `a;b`, `{a,b}`,
//! `"a`, `$((1/0))`); this raises the routine failure instead.

use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::process::CommandExt as _;
use std::process::{Command, Stdio};

use crate::Interp;

/// The words `text` expands to, in the program's own environment and
/// directory, or `None` where `wordexp` refuses it.
pub(super) fn expand(interp: &mut Interp, text: &[u8]) -> Option<Vec<Vec<u8>>> {
    let words = script(text)?;
    let mut program = b"set -- ".to_vec();
    program.extend_from_slice(&words);
    program.extend_from_slice(b"\nprintf '%s\\0' \"$#\" \"$@\"\n");
    let mut command = Command::new("/bin/sh");
    command
        .arg0("sh")
        .arg("-c")
        .arg(std::ffi::OsStr::from_bytes(&program))
        .env_clear()
        .stderr(Stdio::null());
    for (name, value) in &interp.env {
        command.env(
            std::ffi::OsStr::from_bytes(name),
            std::ffi::OsStr::from_bytes(value),
        );
    }
    command.current_dir(crate::paths::os_path(interp.cwd_bytes()));
    interp.sync_input_offset();
    let output = command.output();
    interp.adopt_input_offset();
    let output = output.ok()?;
    if !output.status.success() {
        return None;
    }
    let mut fields = output.stdout.split(|byte| *byte == 0);
    let count: usize = std::str::from_utf8(fields.next()?).ok()?.parse().ok()?;
    let words: Vec<Vec<u8>> = fields.take(count).map(<[u8]>::to_vec).collect();
    (words.len() == count).then_some(words)
}

/// `text` as the shell is to read it after `set --`, or `None` where
/// `wordexp` refuses it.
pub(super) fn script(text: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() + 8);
    let mut at = 0;
    let mut word_start = true;
    while let Some(&byte) = text.get(at) {
        let end = match byte {
            b'\\' => {
                text.get(at + 1)?;
                at + 2
            }
            b'\'' => at + 1 + text[at + 1..].iter().position(|byte| *byte == b'\'')? + 1,
            b'"' => double_quoted(text, at + 1)?,
            b'$' => dollar(text, at)?,
            b'`' => backquoted(text, at + 1)?,
            b'\n' | b'|' | b'&' | b';' | b'<' | b'>' | b'(' | b')' | b'{' | b'}' => return None,
            b'#' if word_start => {
                out.extend_from_slice(b"\\#");
                at += 1;
                word_start = false;
                continue;
            }
            b' ' | b'\t' => {
                out.push(byte);
                at += 1;
                word_start = true;
                continue;
            }
            _ => at + 1,
        };
        out.extend_from_slice(&text[at..end]);
        at = end;
        word_start = false;
    }
    Some(out)
}

/// The index after the `"` closing a double-quoted run whose text starts at
/// `at`.
fn double_quoted(text: &[u8], mut at: usize) -> Option<usize> {
    loop {
        at = match *text.get(at)? {
            b'"' => return Some(at + 1),
            b'\\' => {
                text.get(at + 1)?;
                at + 2
            }
            b'$' => dollar(text, at)?,
            b'`' => backquoted(text, at + 1)?,
            _ => at + 1,
        };
    }
}

/// The index after the backquote closing a command substitution whose text
/// starts at `at`.
fn backquoted(text: &[u8], mut at: usize) -> Option<usize> {
    loop {
        at = match *text.get(at)? {
            b'`' => return Some(at + 1),
            b'\\' => at + 2,
            _ => at + 1,
        };
    }
}

/// The index after the expansion the `$` at `at` starts: `$(...)` and
/// `$((...))` through their matching parenthesis, `${...}` through its
/// matching brace, and anything else the `$` alone.
fn dollar(text: &[u8], at: usize) -> Option<usize> {
    let (open, close) = match text.get(at + 1) {
        Some(b'(') => (b'(', b')'),
        Some(b'{') => (b'{', b'}'),
        _ => return Some(at + 1),
    };
    let mut depth = 1usize;
    let mut index = at + 2;
    loop {
        let byte = *text.get(index)?;
        index = match byte {
            b'\\' => index + 2,
            b'\'' => index + 1 + text[index + 1..].iter().position(|byte| *byte == b'\'')? + 1,
            b'"' => double_quoted(text, index + 1)?,
            b'`' => backquoted(text, index + 1)?,
            b'$' => dollar(text, index)?,
            _ if byte == open => {
                depth += 1;
                index + 1
            }
            _ if byte == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + 1);
                }
                index + 1
            }
            _ => index + 1,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::script;

    /// What `wordexp` refuses, each measured as a crash of the oracle.
    #[test]
    fn the_refusals_are_wordexps() {
        for text in [
            "a;b",
            "a|b",
            "{a,b}",
            "\"unterminated",
            "$(",
            "x > y",
            "(a)",
            "a&b",
            "a\nb",
            "a\\",
            "'a",
            "`a",
            "${a",
        ] {
            assert_eq!(script(text.as_bytes()), None, "{text:?}");
        }
    }

    /// Quoted and substituted metacharacters stand, and a word's leading `#`
    /// is escaped where one inside a word is not.
    #[test]
    fn what_wordexp_accepts_reaches_the_shell() {
        for (text, shell) in [
            ("\"a;b\"", "\"a;b\""),
            ("$(echo a;echo b)", "$(echo a;echo b)"),
            ("$((1+(2)))", "$((1+(2)))"),
            ("${A:-x}", "${A:-x}"),
            ("a#b #c", "a#b \\#c"),
            ("'(a)'", "'(a)'"),
            ("a\\;b", "a\\;b"),
        ] {
            assert_eq!(
                script(text.as_bytes()).as_deref(),
                Some(shell.as_bytes()),
                "{text:?}"
            );
        }
    }
}
