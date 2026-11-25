//! The text conversions both grammars need, built from the runtime services the
//! parsers reach into rather than from either parser.

use interrexx_num::Number;
// The precision `::OPTIONS DIGITS` and `::OPTIONS FUZZ` convert under, which is
// `ARGUMENT_DIGITS` and belongs to the numeric layer along with everything else
// about numbers here.
pub(crate) use interrexx_num::ARGUMENT_DIGITS;

/// The number `text` denotes, or `None` if it denotes none.
fn number(text: &[u8]) -> Option<Number> {
    Number::parse(std::str::from_utf8(text).ok()?)
}

/// Whether `text` is a Rexx number at all, without asking what its value is.
pub(crate) fn is_number(text: &[u8]) -> bool {
    number(text).is_some()
}

/// The value of `text` as a whole number under `digits` precision, or `None` if
/// it has none.
pub(crate) fn whole_number(text: &[u8], digits: usize) -> Option<i64> {
    number(text)?.whole_value(digits)
}

/// Whether `text` is a usable `TRACE` option string.
pub(crate) fn check_trace_setting(text: &[u8]) -> Result<(), ()> {
    for &byte in text {
        if byte == b'?' {
            continue;
        }
        return match byte.to_ascii_uppercase() {
            b'A' | b'C' | b'L' | b'E' | b'F' | b'N' | b'O' | b'R' | b'I' => Ok(()),
            _ => Err(()),
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests;

/// The error code, `major * 1000 + sub`, that the argument of `RAISE SYNTAX`
/// names, or `None` where the argument is not a valid error number.
pub fn syntax_code(text: &[u8]) -> Option<u64> {
    /// One half of the argument as a Rexx number, then a
    /// whole-number check. `None` for anything that is not a whole number.
    fn whole(text: &str) -> Option<i64> {
        Number::parse(text)?.whole_value(
            usize::try_from(interrexx_num::DEFAULT_DIGITS).expect("DEFAULT_DIGITS is 9"),
        )
    }

    let text = String::from_utf8_lossy(text);
    let (major, sub) = match text.split_once('.') {
        // A decimal point with an empty tail is rejected rather than read as
        // zero: `'40.'` is 33.904 where `40` is the `(40, 0)` entry.
        Some((_, "")) => return None,
        Some((major, sub)) => (whole(major), whole(sub)),
        None => (whole(text.as_ref()), Some(0)),
    };
    let (Some(major), Some(sub)) = (major, sub) else {
        return None;
    };
    if !(1..=99).contains(&major) || !(0..=999).contains(&sub) {
        return None;
    }
    // Both bounds are checked above, so neither narrowing can lose anything.
    Some(major.unsigned_abs() * 1000 + sub.unsigned_abs())
}

/// A CATCH SYNTAX matcher is a decimal major with an optional nonzero minor.
pub(crate) fn valid_catch_syntax_number(text: &[u8]) -> bool {
    fn component(text: &[u8], max_digits: usize, max_value: u16) -> bool {
        if text.is_empty() || text.len() > max_digits || !text.iter().all(u8::is_ascii_digit) {
            return false;
        }
        let value = text
            .iter()
            .fold(0, |value, digit| value * 10 + u16::from(digit - b'0'));
        (1..=max_value).contains(&value)
    }

    let mut parts = text.splitn(2, |&byte| byte == b'.');
    let major = parts.next().expect("split always yields a first part");
    component(major, 3, 99) && parts.next().is_none_or(|minor| component(minor, 4, 999))
}
