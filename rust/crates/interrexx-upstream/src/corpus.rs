//! The field encoding of `corpus/errors/parse-errors.tsv`, and the recording
//! of a message's inserts from the line the oracle printed.
//!
//! A field is bytes with `\\`, `\t`, `\n`, `\r` and `\xNN` as the only
//! escapes, so a printable character other than the backslash stands for
//! itself. The `inserts` field holds the values the oracle's text showed, one
//! item `k=value` for each position `k` it shows, each item ending in the
//! two characters `\|`. A message with no shown insert is `-`. The end mark
//! cannot occur inside a value, because a backslash in a value is written
//! doubled.

use crate::template::{INSERTS, Template};

/// `bytes` as a field.
pub fn escape(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &byte in bytes {
        match byte {
            b'\\' => out.push_str("\\\\"),
            b'\t' => out.push_str("\\t"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            0x20..=0x7e => out.push(char::from(byte)),
            _ => out.push_str(&format!("\\x{byte:02x}")),
        }
    }
    out
}

/// The bytes a field stands for.
pub fn unescape(field: &str) -> Result<Vec<u8>, String> {
    let bytes = field.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] != b'\\' {
            out.push(bytes[at]);
            at += 1;
            continue;
        }
        match bytes.get(at + 1) {
            Some(b'\\') => out.push(b'\\'),
            Some(b't') => out.push(b'\t'),
            Some(b'n') => out.push(b'\n'),
            Some(b'r') => out.push(b'\r'),
            Some(b'x') => {
                let hex = field
                    .get(at + 2..at + 4)
                    .ok_or_else(|| format!("truncated \\x escape in {field:?}"))?;
                out.push(
                    u8::from_str_radix(hex, 16)
                        .map_err(|e| format!("bad \\x escape {hex:?}: {e}"))?,
                );
                at += 2;
            }
            other => return Err(format!("unknown escape {other:?} in {field:?}")),
        }
        at += 2;
    }
    Ok(out)
}

/// The `inserts` field for these `(position, value)` items.
pub fn encode_inserts(items: &[(usize, Vec<u8>)]) -> String {
    if items.is_empty() {
        return "-".to_string();
    }
    items
        .iter()
        .map(|(k, value)| format!("{k}={}\\|", escape(value)))
        .collect()
}

/// The `(position, value)` items of an `inserts` field.
pub fn decode_inserts(field: &str) -> Result<Vec<(usize, Vec<u8>)>, String> {
    if field == "-" {
        return Ok(Vec::new());
    }
    let bytes = field.as_bytes();
    let mut items = Vec::new();
    let mut start = 0;
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'\\' {
            if bytes.get(at + 1) == Some(&b'|') {
                items.push(&field[start..at]);
                at += 2;
                start = at;
            } else {
                // An escape: skip the byte it quotes.
                at += 2;
            }
        } else {
            at += 1;
        }
    }
    if start != bytes.len() {
        return Err(format!("an item of {field:?} has no end mark"));
    }
    items
        .into_iter()
        .map(|item| {
            let (k, value) = item
                .split_once('=')
                .ok_or_else(|| format!("{item:?} is not k=value"))?;
            let k = k
                .parse()
                .map_err(|e| format!("{k:?} is not a position: {e}"))?;
            Ok((k, unescape(value)?))
        })
        .collect()
}

/// The values `template`, the text of a message, shows in `line`, for the
/// positions in `rendered`, and whether another way to split `line` gives
/// different values. The shortest first insert wins: a text that places two
/// inserts side by side (`&1 &2`) cannot say where one ends.
pub fn record_inserts(
    template: &str,
    rendered: &[usize],
    line: &[u8],
) -> Result<(Vec<(usize, Vec<u8>)>, bool), String> {
    let splits = Template::parse(template).splits(line);
    let values = |split: &[Option<(usize, usize)>; INSERTS + 1]| -> Vec<(usize, Vec<u8>)> {
        rendered
            .iter()
            .filter_map(|&k| split[k].map(|(from, to)| (k, line[from..to].to_vec())))
            .collect()
    };
    let first = values(
        splits
            .first()
            .ok_or("the line does not match the template")?,
    );
    let ambiguous = splits[1..].iter().any(|split| values(split) != first);
    Ok((first, ambiguous))
}

#[cfg(test)]
mod tests;
