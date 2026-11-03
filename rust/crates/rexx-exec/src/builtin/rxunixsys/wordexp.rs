//! `SysWordexp`: POSIX word expansion with no flags, done here without a
//! shell (Deviation 90).
//!
//! The expansions are glibc `wordexp`'s as the oracle answers them, measured
//! input by input: field splitting on `IFS`, quote removal, tilde, parameter
//! (`$NAME`, `${NAME}`, `${#NAME}`, the `-`, `=`, `?` and `+` forms with and
//! without `:`, and `#`, `##`, `%`, `%%`), arithmetic (`$((...))` and
//! `$[...]`) and pathname expansion, against the program's own environment
//! and directory. Command substitution (`$(...)` and backquotes) is refused,
//! as is everything `wordexp` itself refuses: an unquoted newline, `|`, `&`,
//! `;`, `<`, `>`, `(`, `)`, `{` or `}`, an unterminated quote, a malformed
//! `${...}` and an arithmetic error. The oracle crashes on every such refusal
//! (Deviation 88); a refusal here is the routine failure.
//!
//! What the measurements showed where the shell would differ:
//!
//! * A quoted empty string (`''`, `""`, `"$UNSET"`) that ends while the
//!   current word is empty is a word at once, so `''x` is two words.
//! * Field splitting drops an expansion's leading and trailing `IFS` white
//!   space without ending the word around it (`x${V}y` with `V=' a '` is
//!   `xay`); white space inside it, and every other `IFS` byte, splits.
//! * The value of a set, empty variable that is substituted in a quoted
//!   context into an empty word ends the expansion: the words finished so
//!   far are the result (`a "$EMPTY" b` is `a`).
//! * The word of a `${NAME-word}` form is expanded with single quotes
//!   expanding `$` too, backslashes kept, and a quoted `*` or `?` kept as
//!   `\*` or `\?`; its result is split as an unquoted expansion's is.
//! * `$0`, `$1`... `$#`, `$*` and `$@` are the process's own arguments.
//! * A tilde after `=` (or after `:` following an `=`) expands in the first
//!   word only.
//! * A word holding an unquoted `*`, `?` or `[` is a pattern as a whole,
//!   quoted parts included.

use std::os::unix::ffi::OsStrExt as _;

use crate::Interp;

/// Why an expansion stopped early.
enum Stop {
    /// `wordexp` refuses the text: the routine fails.
    Refused,
    /// A set, empty value in a quoted context ended the expansion; the words
    /// already finished stand.
    Ended,
}

type Step<T> = Result<T, Stop>;

/// The words `text` expands to, in the program's own environment and
/// directory, or `None` where `wordexp` refuses it.
pub(super) fn expand(interp: &mut Interp, text: &[u8]) -> Option<Vec<Vec<u8>>> {
    let mut expander = Expander {
        ifs: interp
            .env_get(b"IFS")
            .map_or_else(|| b" \t\n".to_vec(), <[u8]>::to_vec),
        interp,
        words: Vec::new(),
        word: None,
        glob: false,
    };
    match expander.top(text) {
        Ok(()) => {
            expander.end_word();
            Some(expander.words)
        }
        Err(Stop::Ended) => Some(expander.words),
        Err(Stop::Refused) => None,
    }
}

/// What one `$` expansion produced.
enum Piece {
    /// A value. `ends` marks a set variable's value (plain or with a pattern
    /// removed), the substitution whose emptiness ends a quoted expansion.
    Value { bytes: Vec<u8>, ends: bool },
    /// `$*` (`star`) or `$@`: the process's arguments.
    Arguments { list: Vec<Vec<u8>>, star: bool },
}

struct Expander<'a> {
    interp: &'a mut Interp,
    ifs: Vec<u8>,
    /// The words finished so far.
    words: Vec<Vec<u8>>,
    /// The word being built: `None` until something is in it.
    word: Option<Vec<u8>>,
    /// Whether the word holds an unquoted `*`, `?` or `[`.
    glob: bool,
}

impl Expander<'_> {
    fn push(&mut self, bytes: &[u8]) {
        self.word
            .get_or_insert_with(Vec::new)
            .extend_from_slice(bytes);
    }

    fn word_is_empty(&self) -> bool {
        self.word.as_ref().is_none_or(Vec::is_empty)
    }

    /// Ends the word being built, if there is one, expanding it as a
    /// pattern where it holds an unquoted pattern byte.
    fn end_word(&mut self) {
        if let Some(word) = self.word.take() {
            self.emit(word);
        }
    }

    fn emit(&mut self, mut word: Vec<u8>) {
        if std::mem::take(&mut self.glob) {
            let found = glob(self.interp.cwd_bytes(), &word);
            if !found.is_empty() {
                self.words.extend(found);
                return;
            }
            // Measured: a pattern matching nothing stands for itself, less a
            // trailing `/` (`sub/*/` in an empty `sub` is `sub/*`).
            if word.len() > 1 && word.last() == Some(&b'/') {
                word.pop();
            }
        }
        self.words.push(word);
    }

    /// Ends the word being built, as an empty word if nothing is in it.
    fn force_word(&mut self) {
        let word = self.word.take().unwrap_or_default();
        self.emit(word);
    }

    /// A quoted run has closed: an empty one in an empty word is a word.
    fn close_quote(&mut self) {
        if self.word_is_empty() {
            self.word = None;
            self.force_word();
        }
    }

    fn top(&mut self, text: &[u8]) -> Step<()> {
        let mut at = 0;
        while let Some(&byte) = text.get(at) {
            match byte {
                b' ' | b'\t' => {
                    self.end_word();
                    at += 1;
                }
                b'\n' | b'|' | b'&' | b';' | b'<' | b'>' | b'(' | b')' | b'{' | b'}' | b'`' => {
                    return Err(Stop::Refused);
                }
                b'\\' => {
                    let escaped = *text.get(at + 1).ok_or(Stop::Refused)?;
                    if escaped != b'\n' {
                        self.push(&[escaped]);
                    }
                    at += 2;
                }
                b'\'' => {
                    let close = find(text, at + 1, b'\'').ok_or(Stop::Refused)?;
                    let quoted = &text[at + 1..close];
                    if quoted.is_empty() {
                        self.close_quote();
                    } else {
                        self.push(quoted);
                    }
                    at = close + 1;
                }
                b'"' => at = self.double_quoted(text, at + 1)?,
                b'$' => {
                    let (end, piece) = self.dollar(text, at)?;
                    if let Some(piece) = piece {
                        self.join_unquoted(piece);
                    } else {
                        self.push(b"$");
                    }
                    at = end;
                }
                b'~' if self.tilde_here() => {
                    let (end, value) = self.tilde(text, at, text.len());
                    if !value.is_empty() {
                        self.push(&value);
                    }
                    at = end;
                }
                b'*' | b'?' | b'[' => {
                    self.glob = true;
                    self.push(&[byte]);
                    at += 1;
                }
                _ => {
                    self.push(&[byte]);
                    at += 1;
                }
            }
        }
        Ok(())
    }

    /// The double-quoted run whose text starts at `at`; the index after its
    /// closing quote.
    fn double_quoted(&mut self, text: &[u8], mut at: usize) -> Step<usize> {
        loop {
            match *text.get(at).ok_or(Stop::Refused)? {
                b'"' => {
                    self.close_quote();
                    return Ok(at + 1);
                }
                b'\\' => {
                    let escaped = *text.get(at + 1).ok_or(Stop::Refused)?;
                    match escaped {
                        b'\n' => {}
                        b'$' | b'`' | b'"' | b'\\' => self.push(&[escaped]),
                        _ => self.push(&[b'\\', escaped]),
                    }
                    at += 2;
                }
                b'`' => return Err(Stop::Refused),
                b'$' => {
                    let (end, piece) = self.dollar(text, at)?;
                    match piece {
                        None => self.push(b"$"),
                        Some(Piece::Value { bytes, ends }) => {
                            if ends && bytes.is_empty() && self.word_is_empty() {
                                return Err(Stop::Ended);
                            }
                            self.push(&bytes);
                        }
                        Some(Piece::Arguments { list, star: true }) => {
                            self.push(&list.join(&b' '));
                        }
                        Some(Piece::Arguments { list, star: false }) => {
                            for (index, argument) in list.iter().enumerate() {
                                if index > 0 {
                                    self.force_word();
                                }
                                self.push(argument);
                            }
                        }
                    }
                    at = end;
                }
                byte => {
                    self.push(&[byte]);
                    at += 1;
                }
            }
        }
    }

    /// An unquoted expansion's value joined to the word: split into fields.
    fn join_unquoted(&mut self, piece: Piece) {
        match piece {
            Piece::Value { bytes, .. } => self.split(&bytes),
            Piece::Arguments { list, .. } => {
                for (index, argument) in list.iter().enumerate() {
                    if index > 0 {
                        self.end_word();
                    }
                    self.split(argument);
                }
            }
        }
    }

    /// Field splitting of one unquoted value on `IFS`.
    fn split(&mut self, value: &[u8]) {
        if self.ifs.is_empty() {
            if !value.is_empty() {
                self.push(value);
            }
            return;
        }
        let ifs = self.ifs.clone();
        let white = |byte: u8| matches!(byte, b' ' | b'\t' | b'\n') && ifs.contains(&byte);
        let delimiter = |byte: u8| ifs.contains(&byte) && !white(byte);
        let skip_white = |mut at: usize| {
            while value.get(at).is_some_and(|byte| white(*byte)) {
                at += 1;
            }
            at
        };
        let mut at = skip_white(0);
        while let Some(&byte) = value.get(at) {
            if delimiter(byte) {
                self.force_word();
                at = skip_white(at + 1);
            } else if white(byte) {
                let next = skip_white(at);
                match value.get(next) {
                    None => break,
                    Some(after) if delimiter(*after) => {}
                    Some(_) => self.end_word(),
                }
                at = next;
            } else {
                self.push(&[byte]);
                at += 1;
            }
        }
    }

    /// Whether a `~` at this point starts a tilde prefix: at the start of a
    /// word, or in the first word after `=`, or after `:` following an `=`.
    fn tilde_here(&self) -> bool {
        match &self.word {
            None => true,
            Some(word) => after_assignment(word, self.words.is_empty()),
        }
    }

    /// The tilde prefix at `at` (a `~`), read no further than `limit`: the
    /// index after it and what it expands to.
    fn tilde(&self, text: &[u8], at: usize, limit: usize) -> (usize, Vec<u8>) {
        let mut end = at + 1;
        let mut name = Vec::new();
        let mut escaped = false;
        while end < limit {
            match text[end] {
                b'/' | b':' | b' ' | b'\t' => break,
                b'\\' if end + 1 < limit => {
                    escaped = true;
                    name.push(text[end + 1]);
                    end += 2;
                }
                byte => {
                    name.push(byte);
                    end += 1;
                }
            }
        }
        let verbatim = |name: &[u8]| {
            let mut kept = b"~".to_vec();
            kept.extend_from_slice(name);
            kept
        };
        if escaped {
            return (end, verbatim(&name));
        }
        if name.is_empty() {
            if let Some(home) = self.interp.env_get(b"HOME") {
                return (end, home.to_vec());
            }
            let home = nix::unistd::User::from_uid(nix::unistd::getuid())
                .ok()
                .flatten()
                .map_or_else(
                    || b"~".to_vec(),
                    |user| user.dir.as_os_str().as_bytes().to_vec(),
                );
            return (end, home);
        }
        let home = std::str::from_utf8(&name)
            .ok()
            .and_then(|user| nix::unistd::User::from_name(user).ok().flatten())
            .map(|user| user.dir.as_os_str().as_bytes().to_vec());
        (end, home.unwrap_or_else(|| verbatim(&name)))
    }

    /// The `$` expansion at `at`: the index after it and its value, `None`
    /// where the `$` stands for itself.
    fn dollar(&mut self, text: &[u8], at: usize) -> Step<(usize, Option<Piece>)> {
        let value = |bytes: Vec<u8>, ends: bool| Some(Piece::Value { bytes, ends });
        let Some(&next) = text.get(at + 1) else {
            return Ok((at + 1, None));
        };
        match next {
            b'(' if text.get(at + 2) == Some(&b'(') => {
                let close = arithmetic_end(text, at + 3)?;
                let result = self.arithmetic(&text[at + 3..close])?;
                Ok((close + 2, value(result, false)))
            }
            b'(' => Err(Stop::Refused),
            b'[' => {
                let close = bracket_end(text, at + 2)?;
                let result = self.arithmetic(&text[at + 2..close])?;
                Ok((close + 1, value(result, false)))
            }
            b'{' => self.braced(text, at + 2),
            b'*' | b'@' => Ok((
                at + 2,
                Some(Piece::Arguments {
                    list: arguments().into_iter().skip(1).collect(),
                    star: next == b'*',
                }),
            )),
            b'#' | b'$' | b'0'..=b'9' => {
                let found = special(&[next]);
                let ends = found.is_some();
                Ok((at + 2, value(found.unwrap_or_default(), ends)))
            }
            byte if byte == b'_' || byte.is_ascii_alphabetic() => {
                let end = name_end(text, at + 1);
                let found = self.interp.env_get(&text[at + 1..end]).map(<[u8]>::to_vec);
                let ends = found.is_some();
                Ok((end, value(found.unwrap_or_default(), ends)))
            }
            _ => Ok((at + 1, None)),
        }
    }

    /// The `${...}` form whose text starts at `at`, just after the brace.
    fn braced(&mut self, text: &[u8], at: usize) -> Step<(usize, Option<Piece>)> {
        let value = |bytes: Vec<u8>, ends: bool| Some(Piece::Value { bytes, ends });
        if text.get(at) == Some(&b'#') {
            let start = at + 1;
            let end = match text.get(start) {
                Some(b'$' | b'*' | b'@') => start + 1,
                Some(byte) if byte.is_ascii_digit() => digits_end(text, start),
                Some(byte) if *byte == b'_' || byte.is_ascii_alphabetic() => name_end(text, start),
                _ => return Err(Stop::Refused),
            };
            if text.get(end) != Some(&b'}') {
                return Err(Stop::Refused);
            }
            let name = &text[start..end];
            let length = if matches!(name, b"*" | b"@") {
                arguments().len().saturating_sub(1)
            } else {
                self.lookup(name).map_or(0, |found| found.len())
            };
            return Ok((end + 1, value(length.to_string().into_bytes(), false)));
        }
        let end = match text.get(at) {
            Some(b'$' | b'*' | b'@' | b'#') => at + 1,
            Some(byte) if byte.is_ascii_digit() => digits_end(text, at),
            Some(byte) if *byte == b'_' || byte.is_ascii_alphabetic() => name_end(text, at),
            _ => return Err(Stop::Refused),
        };
        let name = text[at..end].to_vec();
        let mut operator_at = end;
        let colon = text.get(operator_at) == Some(&b':');
        if colon {
            operator_at += 1;
        }
        let operator = *text.get(operator_at).ok_or(Stop::Refused)?;
        if operator == b'}' && !colon {
            if matches!(name.as_slice(), b"*" | b"@") {
                let list = arguments().into_iter().skip(1).collect();
                return Ok((
                    operator_at + 1,
                    Some(Piece::Arguments {
                        list,
                        star: name == b"*",
                    }),
                ));
            }
            let found = self.lookup(&name);
            let ends = found.is_some();
            return Ok((operator_at + 1, value(found.unwrap_or_default(), ends)));
        }
        let (operator, word_at) = match operator {
            b'-' | b'=' | b'?' | b'+' => (vec![operator], operator_at + 1),
            b'#' | b'%' if !colon => {
                if text.get(operator_at + 1) == Some(&operator) {
                    (vec![operator, operator], operator_at + 2)
                } else {
                    (vec![operator], operator_at + 1)
                }
            }
            _ => return Err(Stop::Refused),
        };
        let (word_end, after) = word_end(text, word_at)?;
        let word = &text[word_at..word_end];
        if has_command(word) {
            return Err(Stop::Refused);
        }
        let found = self.lookup(&name);
        let missing = match &found {
            None => true,
            Some(found) => colon && found.is_empty(),
        };
        let result = match operator.as_slice() {
            b"-" => {
                if missing {
                    value(self.nested(word)?, false)
                } else {
                    value(found.unwrap_or_default(), false)
                }
            }
            b"=" => {
                if missing {
                    let assigned = self.nested(word)?;
                    if !name[0].is_ascii_digit() && !matches!(name[0], b'$' | b'#') {
                        self.interp.env_set(&name, Some(assigned.clone()));
                    }
                    value(assigned, false)
                } else {
                    value(found.unwrap_or_default(), false)
                }
            }
            b"?" => {
                if missing {
                    let mut message = self.nested(word)?;
                    if message.is_empty() {
                        message = b"parameter null or not set".to_vec();
                    }
                    let trace = &mut self.interp.trace;
                    trace.extend_from_slice(&name);
                    trace.extend_from_slice(b": ");
                    trace.extend_from_slice(&message);
                    trace.push(b'\n');
                    value(Vec::new(), false)
                } else {
                    value(found.unwrap_or_default(), false)
                }
            }
            b"+" => {
                if missing {
                    value(Vec::new(), false)
                } else {
                    value(self.nested(word)?, false)
                }
            }
            removal => {
                let pattern = self.nested(word)?;
                match found {
                    None => value(Vec::new(), false),
                    Some(found) => value(remove(&found, &pattern, removal), true),
                }
            }
        };
        Ok((after, result))
    }

    /// A parameter's value: a variable, `$$`, `$#` or a positional
    /// parameter; `None` where it is not set.
    fn lookup(&self, name: &[u8]) -> Option<Vec<u8>> {
        if name[0] == b'_' || name[0].is_ascii_alphabetic() {
            self.interp.env_get(name).map(<[u8]>::to_vec)
        } else if matches!(name, b"*" | b"@") {
            Some(
                arguments()
                    .into_iter()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join(&b' '),
            )
        } else {
            special(name)
        }
    }

    /// The word of a `${NAME-word}` form expanded: quotes removed with
    /// `$` expanded inside both kinds, backslashes kept, a quoted `*` or `?`
    /// kept escaped, and no field splitting.
    fn nested(&mut self, word: &[u8]) -> Step<Vec<u8>> {
        let mut out: Vec<u8> = Vec::new();
        let mut quote = None;
        let mut at = 0;
        while let Some(&byte) = word.get(at) {
            match byte {
                b'\\' => {
                    out.push(b'\\');
                    if let Some(&escaped) = word.get(at + 1) {
                        out.push(escaped);
                    }
                    at += 2;
                    continue;
                }
                b'\'' | b'"' if quote.is_none() => quote = Some(byte),
                b'\'' | b'"' if quote == Some(byte) => quote = None,
                b'`' => return Err(Stop::Refused),
                b'$' => {
                    let (end, piece) = self.dollar(word, at)?;
                    match piece {
                        None => out.push(b'$'),
                        Some(Piece::Value { bytes, ends }) => {
                            if ends && bytes.is_empty() && out.is_empty() {
                                return Err(Stop::Ended);
                            }
                            out.extend_from_slice(&bytes);
                        }
                        Some(Piece::Arguments { list, .. }) => {
                            out.extend_from_slice(&list.join(&b' '))
                        }
                    }
                    at = end;
                    continue;
                }
                b'~' if quote.is_none() && (out.is_empty() || after_assignment(&out, true)) => {
                    let (end, value) = self.tilde(word, at, word.len());
                    out.extend_from_slice(&value);
                    at = end;
                    continue;
                }
                b'*' | b'?' if quote.is_some() => out.extend_from_slice(&[b'\\', byte]),
                _ => out.push(byte),
            }
            at += 1;
        }
        Ok(out)
    }

    /// An arithmetic expansion's text, after `$` expansion, evaluated.
    fn arithmetic(&mut self, text: &[u8]) -> Step<Vec<u8>> {
        let mut expression: Vec<u8> = Vec::new();
        let mut at = 0;
        while let Some(&byte) = text.get(at) {
            // Measured: these refuse the expansion; `| & < > , =` do not.
            if matches!(byte, b'`' | b'\n' | b';' | b'{' | b'}') {
                return Err(Stop::Refused);
            }
            if byte != b'$' {
                expression.push(byte);
                at += 1;
                continue;
            }
            let (end, piece) = self.dollar(text, at)?;
            match piece {
                None => expression.push(b'$'),
                Some(Piece::Value { bytes, ends }) => {
                    if ends && bytes.is_empty() && expression.is_empty() {
                        return Err(Stop::Ended);
                    }
                    expression.extend_from_slice(&bytes);
                }
                Some(Piece::Arguments { list, .. }) => {
                    expression.extend_from_slice(&list.join(&b' '))
                }
            }
            at = end;
        }
        if expression.is_empty() {
            return Ok(b"0".to_vec());
        }
        let mut at = 0;
        let result = sum(&expression, &mut at).ok_or(Stop::Refused)?;
        Ok(result.to_string().into_bytes())
    }
}

/// Whether a word built so far ends where a tilde prefix may start besides
/// a word's start: in the first word (`first`), after `=`, or after `:`
/// once the word holds an `=`.
fn after_assignment(word: &[u8], first: bool) -> bool {
    first
        && match word.last() {
            Some(b'=') => true,
            Some(b':') => word.contains(&b'='),
            _ => false,
        }
}

/// The process's arguments, `$0` first.
fn arguments() -> Vec<Vec<u8>> {
    std::env::args_os()
        .map(|argument| argument.as_bytes().to_vec())
        .collect()
}

/// `$$`, `$#` or a positional parameter; `None` where it is not set.
fn special(name: &[u8]) -> Option<Vec<u8>> {
    match name {
        b"$" => Some(std::process::id().to_string().into_bytes()),
        b"#" => Some(arguments().len().saturating_sub(1).to_string().into_bytes()),
        digits => {
            let index: usize = std::str::from_utf8(digits).ok()?.parse().ok()?;
            arguments().into_iter().nth(index)
        }
    }
}

fn find(text: &[u8], from: usize, byte: u8) -> Option<usize> {
    text.get(from..)?
        .iter()
        .position(|held| *held == byte)
        .map(|at| from + at)
}

fn name_end(text: &[u8], mut at: usize) -> usize {
    while text
        .get(at)
        .is_some_and(|byte| *byte == b'_' || byte.is_ascii_alphanumeric())
    {
        at += 1;
    }
    at
}

fn digits_end(text: &[u8], mut at: usize) -> usize {
    while text.get(at).is_some_and(u8::is_ascii_digit) {
        at += 1;
    }
    at
}

/// Whether a `${NAME-word}` word holds a command substitution.
fn has_command(word: &[u8]) -> bool {
    let mut at = 0;
    while let Some(&byte) = word.get(at) {
        match byte {
            b'\\' => at += 1,
            b'`' => return true,
            b'$' if word.get(at + 1) == Some(&b'(') && word.get(at + 2) != Some(&b'(') => {
                return true;
            }
            _ => {}
        }
        at += 1;
    }
    false
}

/// The end of a `${NAME-word}` word starting at `at`: the index of the brace
/// closing it and the index after the form. A backslash outside quotes
/// escapes the next byte, single and double quotes hide braces (a backslash
/// inside them is an ordinary byte), and braces nest. Reaching the end of
/// the text is accepted only where the text's last byte is `}`; the word is
/// then the rest of the text.
fn word_end(text: &[u8], at: usize) -> Step<(usize, usize)> {
    let mut depth = 1usize;
    let mut quote = None;
    let mut index = at;
    while let Some(&byte) = text.get(index) {
        match (quote, byte) {
            (None, b'\\') => index += 1,
            (None, b'\'' | b'"') => quote = Some(byte),
            (Some(open), _) if open == byte => quote = None,
            (None, b'{') => depth += 1,
            (None, b'}') => {
                depth -= 1;
                if depth == 0 {
                    return Ok((index, index + 1));
                }
            }
            _ => {}
        }
        index += 1;
    }
    if text.last() == Some(&b'}') && text.len() > at {
        Ok((text.len(), text.len()))
    } else {
        Err(Stop::Refused)
    }
}

/// The index of the first `)` of the `))` closing an arithmetic expansion
/// whose text starts at `at`.
fn arithmetic_end(text: &[u8], at: usize) -> Step<usize> {
    let mut depth = 0usize;
    let mut index = at;
    loop {
        match *text.get(index).ok_or(Stop::Refused)? {
            b'(' => depth += 1,
            b')' if depth == 0 => {
                return if text.get(index + 1) == Some(&b')') {
                    Ok(index)
                } else {
                    Err(Stop::Refused)
                };
            }
            b')' => depth -= 1,
            _ => {}
        }
        index += 1;
    }
}

/// The index of the `]` closing a `$[` expansion whose text starts at `at`.
fn bracket_end(text: &[u8], at: usize) -> Step<usize> {
    let mut depth = 0usize;
    let mut index = at;
    loop {
        match *text.get(index).ok_or(Stop::Refused)? {
            b'[' => depth += 1,
            b']' if depth == 0 => return Ok(index),
            b']' => depth -= 1,
            _ => {}
        }
        index += 1;
    }
}

/// `+` and `-` over terms, left to right; text after the last term is
/// ignored.
fn sum(text: &[u8], at: &mut usize) -> Option<i64> {
    let mut total = product(text, at)?;
    loop {
        let mark = skip_space(text, *at);
        match text.get(mark) {
            Some(b'+') => {
                *at = mark + 1;
                total = total.wrapping_add(product(text, at)?);
            }
            Some(b'-') => {
                *at = mark + 1;
                total = total.wrapping_sub(product(text, at)?);
            }
            _ => return Some(total),
        }
    }
}

/// `*` and `/` over factors, left to right.
fn product(text: &[u8], at: &mut usize) -> Option<i64> {
    let mut total = factor(text, at)?;
    loop {
        let mark = skip_space(text, *at);
        match text.get(mark) {
            Some(b'*') => {
                *at = mark + 1;
                total = total.wrapping_mul(factor(text, at)?);
            }
            Some(b'/') => {
                *at = mark + 1;
                total = total.checked_div(factor(text, at)?)?;
            }
            _ => return Some(total),
        }
    }
}

/// A parenthesised sum (text after its last term ignored, as at the top)
/// or a number as C's `strtol` with base 0 reads it.
fn factor(text: &[u8], at: &mut usize) -> Option<i64> {
    let start = skip_space(text, *at);
    if text.get(start) == Some(&b'(') {
        let mut depth = 0usize;
        let mut index = start + 1;
        let close = loop {
            match *text.get(index)? {
                b'(' => depth += 1,
                b')' if depth == 0 => break index,
                b')' => depth -= 1,
                _ => {}
            }
            index += 1;
        };
        let inner = &text[start + 1..close];
        let mut inner_at = 0;
        let value = sum(inner, &mut inner_at)?;
        *at = close + 1;
        return Some(value);
    }
    strtol(text, at)
}

fn skip_space(text: &[u8], mut at: usize) -> usize {
    while text
        .get(at)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r'))
    {
        at += 1;
    }
    at
}

/// C's `strtol(text, &end, 0)` from `at`: `None` where no digit is read, the
/// value saturated where it overflows.
fn strtol(text: &[u8], at: &mut usize) -> Option<i64> {
    let mut index = skip_space(text, *at);
    let negative = match text.get(index) {
        Some(b'-') => {
            index += 1;
            true
        }
        Some(b'+') => {
            index += 1;
            false
        }
        _ => false,
    };
    let hex = text.get(index) == Some(&b'0')
        && matches!(text.get(index + 1), Some(b'x' | b'X'))
        && text.get(index + 2).is_some_and(u8::is_ascii_hexdigit);
    let (radix, mut index) = if hex {
        (16, index + 2)
    } else if text.get(index) == Some(&b'0') {
        (8, index)
    } else {
        (10, index)
    };
    let first = index;
    let mut magnitude: u64 = 0;
    let mut overflow = false;
    while let Some(digit) = text
        .get(index)
        .and_then(|byte| char::from(*byte).to_digit(radix))
    {
        match magnitude
            .checked_mul(u64::from(radix))
            .and_then(|value| value.checked_add(u64::from(digit)))
        {
            Some(value) => magnitude = value,
            None => overflow = true,
        }
        index += 1;
    }
    if index == first {
        return None;
    }
    *at = index;
    let value = if negative {
        if overflow || magnitude > i64::MIN.unsigned_abs() {
            i64::MIN
        } else {
            0i64.wrapping_sub_unsigned(magnitude)
        }
    } else if overflow || magnitude > i64::MAX as u64 {
        i64::MAX
    } else {
        magnitude as i64
    };
    Some(value)
}

/// `${NAME#pattern}` and its kin: `value` with the shortest (`#`, `%`) or
/// longest (`##`, `%%`) prefix (`#`) or suffix (`%`) matching `pattern`
/// removed.
fn remove(value: &[u8], pattern: &[u8], operator: &[u8]) -> Vec<u8> {
    let length = value.len();
    let found = match operator {
        b"#" => (0..=length)
            .find(|&cut| fnmatch(pattern, &value[..cut], false))
            .map(|cut| value[cut..].to_vec()),
        b"##" => (0..=length)
            .rev()
            .find(|&cut| fnmatch(pattern, &value[..cut], false))
            .map(|cut| value[cut..].to_vec()),
        b"%" => (0..=length)
            .rev()
            .find(|&cut| fnmatch(pattern, &value[cut..], false))
            .map(|cut| value[..cut].to_vec()),
        _ => (0..=length)
            .find(|&cut| fnmatch(pattern, &value[cut..], false))
            .map(|cut| value[..cut].to_vec()),
    };
    found.unwrap_or_else(|| value.to_vec())
}

/// `fnmatch(3)` in the C locale with backslash escapes: `*`, `?` and
/// bracket expressions. Under `period` a leading `.` of `name` must be
/// matched by a literal `.`.
fn fnmatch(pattern: &[u8], name: &[u8], period: bool) -> bool {
    if period && name.first() == Some(&b'.') && !matches!(pattern, [b'.', ..] | [b'\\', b'.', ..]) {
        return false;
    }
    match_from(pattern, name)
}

fn match_from(pattern: &[u8], name: &[u8]) -> bool {
    let Some(&first) = pattern.first() else {
        return name.is_empty();
    };
    match first {
        b'*' => (0..=name.len()).any(|skip| match_from(&pattern[1..], &name[skip..])),
        b'?' => !name.is_empty() && match_from(&pattern[1..], &name[1..]),
        b'[' => match (name.first(), bracket(pattern)) {
            (Some(&byte), Some((set, rest))) => set(byte) && match_from(rest, &name[1..]),
            (Some(&b'['), None) => match_from(&pattern[1..], &name[1..]),
            _ => false,
        },
        b'\\' if pattern.len() > 1 => {
            name.first() == Some(&pattern[1]) && match_from(&pattern[2..], &name[1..])
        }
        byte => name.first() == Some(&byte) && match_from(&pattern[1..], &name[1..]),
    }
}

/// The bracket expression `pattern` opens with: a membership test and the
/// pattern after its `]`, or `None` where it has no `]` (a literal `[`).
fn bracket(pattern: &[u8]) -> Option<(impl Fn(u8) -> bool, &[u8])> {
    let mut at = 1;
    let negated = matches!(pattern.get(at), Some(b'!' | b'^'));
    if negated {
        at += 1;
    }
    let mut members = [false; 256];
    let mut first = true;
    loop {
        let byte = *pattern.get(at)?;
        if byte == b']' && !first {
            at += 1;
            break;
        }
        first = false;
        if byte == b'['
            && pattern.get(at + 1) == Some(&b':')
            && let Some(length) = pattern[at + 2..].windows(2).position(|pair| pair == b":]")
        {
            let name = &pattern[at + 2..at + 2 + length];
            for value in 0..=255u8 {
                if in_class(name, value) {
                    members[usize::from(value)] = true;
                }
            }
            at += length + 4;
            continue;
        }
        let (low, next) = if byte == b'\\' && pattern.len() > at + 1 {
            (pattern[at + 1], at + 2)
        } else {
            (byte, at + 1)
        };
        if pattern.get(next) == Some(&b'-')
            && pattern.get(next + 1).is_some_and(|high| *high != b']')
        {
            let (high, after) = if pattern[next + 1] == b'\\' && pattern.len() > next + 2 {
                (pattern[next + 2], next + 3)
            } else {
                (pattern[next + 1], next + 2)
            };
            for value in low..=high {
                members[usize::from(value)] = true;
            }
            at = after;
        } else {
            members[usize::from(low)] = true;
            at = next;
        }
    }
    Some((
        move |byte: u8| members[usize::from(byte)] != negated,
        &pattern[at..],
    ))
}

fn in_class(name: &[u8], byte: u8) -> bool {
    match name {
        b"alpha" => byte.is_ascii_alphabetic(),
        b"digit" => byte.is_ascii_digit(),
        b"alnum" => byte.is_ascii_alphanumeric(),
        b"upper" => byte.is_ascii_uppercase(),
        b"lower" => byte.is_ascii_lowercase(),
        b"space" => matches!(byte, b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r'),
        b"blank" => matches!(byte, b' ' | b'\t'),
        b"punct" => byte.is_ascii_punctuation(),
        b"print" => (0x20..0x7f).contains(&byte),
        b"graph" => byte.is_ascii_graphic(),
        b"cntrl" => byte.is_ascii_control(),
        b"xdigit" => byte.is_ascii_hexdigit(),
        _ => false,
    }
}

/// Whether one path component holds an unescaped pattern byte.
fn has_pattern(component: &[u8]) -> bool {
    let mut at = 0;
    while let Some(&byte) = component.get(at) {
        match byte {
            b'\\' => at += 1,
            b'*' | b'?' | b'[' => return true,
            _ => {}
        }
        at += 1;
    }
    false
}

fn unescape(component: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(component.len());
    let mut at = 0;
    while let Some(&byte) = component.get(at) {
        if byte == b'\\' && at + 1 < component.len() {
            at += 1;
        }
        out.push(component[at]);
        at += 1;
    }
    out
}

/// Pathname expansion of `pattern` against `cwd`, as `glob(3)` answers it:
/// the matching paths sorted by byte, a leading `.` matched only by a
/// literal `.`, `.` and `..` listed; empty where nothing matches.
fn glob(cwd: &[u8], pattern: &[u8]) -> Vec<Vec<u8>> {
    let absolute = pattern.first() == Some(&b'/');
    let directory_only = pattern.last() == Some(&b'/');
    let components: Vec<&[u8]> = pattern
        .split(|byte| *byte == b'/')
        .filter(|part| !part.is_empty())
        .collect();
    if components.is_empty() {
        return Vec::new();
    }
    let mut found: Vec<Vec<u8>> = vec![if absolute { b"/".to_vec() } else { Vec::new() }];
    let resolve = |path: &[u8]| -> std::path::PathBuf {
        let base = crate::paths::os_path(cwd);
        if path.is_empty() {
            base.to_path_buf()
        } else {
            base.join(crate::paths::os_path(path))
        }
    };
    let last = components.len() - 1;
    for (index, component) in components.iter().enumerate() {
        let separator = |path: &[u8]| -> Vec<u8> {
            let mut joined = path.to_vec();
            if !joined.is_empty() && joined.last() != Some(&b'/') {
                joined.push(b'/');
            }
            joined
        };
        let mut next = Vec::new();
        if has_pattern(component) {
            for path in &found {
                let Ok(entries) = std::fs::read_dir(resolve(path)) else {
                    continue;
                };
                let mut names: Vec<Vec<u8>> = vec![b".".to_vec(), b"..".to_vec()];
                names.extend(
                    entries
                        .flatten()
                        .map(|entry| entry.file_name().as_bytes().to_vec()),
                );
                for name in names {
                    if fnmatch(component, &name, true) {
                        let mut joined = separator(path);
                        joined.extend_from_slice(&name);
                        next.push(joined);
                    }
                }
            }
        } else {
            let literal = unescape(component);
            for path in &found {
                let mut joined = separator(path);
                joined.extend_from_slice(&literal);
                next.push(joined);
            }
        }
        let must_be_directory = index < last || directory_only;
        next.retain(|path| match std::fs::metadata(resolve(path)) {
            Ok(metadata) => !must_be_directory || metadata.is_dir(),
            Err(_) => std::fs::symlink_metadata(resolve(path)).is_ok() && !must_be_directory,
        });
        found = next;
        if found.is_empty() {
            return found;
        }
    }
    if directory_only {
        for path in &mut found {
            path.push(b'/');
        }
    }
    found.sort();
    found
}

#[cfg(test)]
mod tests {
    use super::{fnmatch, remove, strtol, sum};

    fn evaluate(text: &str) -> Option<i64> {
        let mut at = 0;
        sum(text.as_bytes(), &mut at)
    }

    /// Arithmetic as the oracle answers it: `+ - * /` with C precedence,
    /// numbers as `strtol` reads them, trailing text ignored.
    #[test]
    fn arithmetic_is_wordexps() {
        for (text, answer) in [
            ("1+2", Some(3)),
            ("-7/2", Some(-3)),
            ("7%3", Some(7)),
            ("1 2", Some(1)),
            ("2*(3+4)", Some(14)),
            ("0x10", Some(16)),
            ("010", Some(8)),
            ("09", Some(0)),
            ("1--1", Some(2)),
            ("--1", None),
            ("-(2)", None),
            ("1/0", None),
            ("1+", None),
            (" ", None),
            ("99999999999999999999", Some(i64::MAX)),
            ("9223372036854775807+1", Some(i64::MIN)),
        ] {
            assert_eq!(evaluate(text), answer, "{text:?}");
        }
        let mut at = 0;
        assert_eq!(strtol(b"0x", &mut at), Some(0));
    }

    #[test]
    fn patterns_remove_as_the_oracle_does() {
        for (value, pattern, operator, answer) in [
            ("abc.def.ghi", ".*", "%", "abc.def"),
            ("abc.def.ghi", ".*", "%%", "abc"),
            ("abc.def.ghi", "*.", "#", "def.ghi"),
            ("abc.def.ghi", "*.", "##", "ghi"),
            ("abc.def.ghi", "\\*", "#", "abc.def.ghi"),
            ("abc.def.ghi", "[gh]i", "%", "abc.def.g"),
            ("abc.def.ghi", "[!a]", "#", "abc.def.ghi"),
        ] {
            assert_eq!(
                remove(value.as_bytes(), pattern.as_bytes(), operator.as_bytes()),
                answer.as_bytes(),
                "{value} {operator} {pattern}"
            );
        }
        assert!(!fnmatch(b"*", b".hidden", true));
        assert!(fnmatch(b".*", b".hidden", true));
    }
}
