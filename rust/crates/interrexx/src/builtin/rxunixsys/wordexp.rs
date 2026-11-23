//! The quoted word-list subset of POSIX word expansion (Deviation 90).
//! The parser works on bytes. Parameter values are appended as data and
//! never become input to the parser.

pub(super) fn words<'env>(
    mut input: &[u8],
    env: impl Fn(&[u8]) -> Option<&'env [u8]>,
) -> Option<Vec<Vec<u8>>> {
    if input.is_empty() || input.contains(&0) {
        return None;
    }

    let mut words = Vec::new();
    let mut word = Vec::new();
    // An empty quoted word is still a word.
    let mut started = false;
    while let Some((&byte, rest)) = input.split_first() {
        input = rest;
        match byte {
            b' ' | b'\t' => {
                if started {
                    words.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            b'\'' => {
                let end = input.iter().position(|&byte| byte == b'\'')?;
                word.extend_from_slice(&input[..end]);
                input = &input[end + 1..];
                started = true;
            }
            b'"' => {
                input = double_quoted(input, &mut word, &env)?;
                started = true;
            }
            b'\\' => {
                let (&escaped, rest) = input.split_first()?;
                if escaped == b'\n' {
                    return None;
                }
                input = rest;
                word.push(escaped);
                started = true;
            }
            b'#' if !started => return None,
            b'\n' | b'|' | b'&' | b';' | b'<' | b'>' | b'(' | b')' | b'{' | b'}' | b'*' | b'?'
            | b'[' | b'~' | b'$' | b'`' => return None,
            literal => {
                word.push(literal);
                started = true;
            }
        }
    }
    if started {
        words.push(word);
    }
    Some(words)
}

fn double_quoted<'input, 'env>(
    mut input: &'input [u8],
    word: &mut Vec<u8>,
    env: &impl Fn(&[u8]) -> Option<&'env [u8]>,
) -> Option<&'input [u8]> {
    while let Some((&byte, rest)) = input.split_first() {
        input = rest;
        match byte {
            b'"' => return Some(input),
            b'\\' => {
                let (&escaped, rest) = input.split_first()?;
                match escaped {
                    b'$' | b'"' | b'\\' | b'`' => {
                        word.push(escaped);
                        input = rest;
                    }
                    b'\n' => return None,
                    _ => word.push(b'\\'),
                }
            }
            b'$' => {
                let (name, rest) = parameter(input)?;
                let value = env(name).unwrap_or_default();
                if value.contains(&0) {
                    return None;
                }
                word.extend_from_slice(value);
                input = rest;
            }
            b'`' => return None,
            literal => word.push(literal),
        }
    }
    None
}

/// A plain variable name, with optional braces. All other forms are
/// unsupported, including nested syntax inside braces.
fn parameter(input: &[u8]) -> Option<(&[u8], &[u8])> {
    let (braced, input) = match input.strip_prefix(b"{") {
        Some(input) => (true, input),
        None => (false, input),
    };
    let (&first, rest) = input.split_first()?;
    if !first.is_ascii_alphabetic() && first != b'_' {
        return None;
    }
    let end = 1 + rest
        .iter()
        .take_while(|&&byte| byte.is_ascii_alphanumeric() || byte == b'_')
        .count();
    let (name, rest) = input.split_at(end);
    let rest = if braced {
        rest.strip_prefix(b"}")?
    } else {
        rest
    };
    Some((name, rest))
}
