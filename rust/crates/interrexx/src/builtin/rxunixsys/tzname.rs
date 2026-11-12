//! `SysGettzname1` and `SysGettzname2`: glibc's `tzname[0]` and `tzname[1]`
//! after `tzset`, derived from the program's own `TZ` as glibc derives them.
//!
//! * `TZ` unset reads `/etc/localtime`; an empty `TZ` is the name
//!   `Universal`; a leading `:` is dropped. A name not starting with `/` is
//!   looked up under `TZDIR`, or `/usr/share/zoneinfo` where that is unset or
//!   empty.
//! * From a TZif file: walking the transitions from the last back, the first
//!   standard type's designation and the first daylight type's; with no
//!   transitions the first type's, for both. For example, `Asia/Tokyo` is `JST
//!   JDT` and `Europe/Dublin` `IST GMT`.
//! * Otherwise the text is a POSIX rule: `XYZ3` is `XYZ XYZ`, `XYZ3ABC` is
//!   `XYZ ABC`, `XYZ` (no offset) is `XYZ` and the null string, and a name of
//!   fewer than three letters that is not `<...>` is two null strings.
//!
//! Where `localtime` has been called since `TZ` last changed (by `TIME` or
//! `DATE`, or a `SysStat` time), glibc's `tzname` holds the names of the rule
//! in force then instead, and `tzset` with the same `TZ` does not reset it:
//! `Asia/Tokyo` answers `JST JST` after `time()`. This answers as a program
//! that has not.

/// `(tzname[0], tzname[1])` for `tz`, the program's `TZ` (`None` where unset),
/// with `tzdir` its `TZDIR`.
pub(super) fn names(tz: Option<&[u8]>, tzdir: Option<&[u8]>) -> (Vec<u8>, Vec<u8>) {
    let name: &[u8] = match tz {
        None => b"/etc/localtime",
        Some(b"") => b"Universal",
        Some(text) => text.strip_prefix(b":").unwrap_or(text),
    };
    if let Some(names) = file_names(name, tzdir) {
        return names;
    }
    rule_names(name)
}

/// The names a TZif file at `name` gives, or `None` where no file there
/// reads as one.
fn file_names(name: &[u8], tzdir: Option<&[u8]>) -> Option<(Vec<u8>, Vec<u8>)> {
    if name.is_empty() {
        return None;
    }
    let mut path = Vec::new();
    if name.first() != Some(&b'/') {
        let directory = match tzdir {
            Some(dir) if !dir.is_empty() => dir,
            _ => b"/usr/share/zoneinfo",
        };
        path.extend_from_slice(directory);
        path.push(b'/');
    }
    path.extend_from_slice(name);
    let bytes = std::fs::read(crate::paths::os_path(&path)).ok()?;
    let zone = tz::TimeZone::from_tz_data(&bytes).ok()?;
    let zone = zone.as_ref();
    let types = zone.local_time_types();
    let designation = |index: usize| {
        types
            .get(index)
            .map(|kind| kind.time_zone_designation().as_bytes().to_vec())
    };
    let mut found: [Option<Vec<u8>>; 2] = [None, None];
    for transition in zone.transitions().iter().rev() {
        let index = transition.local_time_type_index();
        let dst = usize::from(types.get(index).is_some_and(tz::LocalTimeType::is_dst));
        if found[dst].is_none() {
            found[dst] = designation(index);
            if found[1 - dst].is_some() {
                break;
            }
        }
    }
    let [standard, daylight] = found;
    let standard = standard.or_else(|| designation(0)).unwrap_or_default();
    let daylight = daylight.unwrap_or_else(|| standard.clone());
    Some((standard, daylight))
}

/// The names glibc's `__tzset_parse_tz` leaves for `text`.
fn rule_names(text: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut rest = text;
    let Some(standard) = zone_name(&mut rest) else {
        return (Vec::new(), Vec::new());
    };
    if !offset(&mut rest, true) {
        return (standard, Vec::new());
    }
    if rest.is_empty() {
        return (standard.clone(), standard);
    }
    let daylight = zone_name(&mut rest).unwrap_or_default();
    (standard, daylight)
}

/// `parse_tzname`: three or more ASCII letters, or `<` three or more
/// letters, digits, `+` and `-`, then `>`.
fn zone_name(rest: &mut &[u8]) -> Option<Vec<u8>> {
    let letters = rest
        .iter()
        .take_while(|byte| byte.is_ascii_alphabetic())
        .count();
    if letters >= 3 {
        let name = rest[..letters].to_vec();
        *rest = &rest[letters..];
        return Some(name);
    }
    let quoted = rest.strip_prefix(b"<")?;
    let length = quoted
        .iter()
        .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-'))
        .count();
    if quoted.get(length) != Some(&b'>') || length < 3 {
        return None;
    }
    let name = quoted[..length].to_vec();
    *rest = &quoted[length + 1..];
    Some(name)
}

/// `parse_offset` for the standard rule: an optional sign and `hh[:mm[:ss]]`.
/// `false` where no offset is there.
fn offset(rest: &mut &[u8], standard: bool) -> bool {
    let mut text = *rest;
    if standard
        && !matches!(text.first(), Some(byte) if *byte == b'+' || *byte == b'-' || byte.is_ascii_digit())
    {
        return false;
    }
    if matches!(text.first(), Some(b'+' | b'-')) {
        text = &text[1..];
    }
    let mut consumed = 0;
    for field in 0..3 {
        let start = consumed + usize::from(field > 0);
        if field > 0 && text.get(consumed) != Some(&b':') {
            break;
        }
        let digits = text[start.min(text.len())..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if digits == 0 {
            break;
        }
        consumed = start + digits;
    }
    if consumed == 0 {
        return false;
    }
    *rest = &text[consumed..];
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(text: &str) -> (String, String) {
        let (standard, daylight) = rule_names(text.as_bytes());
        (
            String::from_utf8(standard).unwrap(),
            String::from_utf8(daylight).unwrap(),
        )
    }

    /// For example, each as `TZ`, none of them a file under
    /// `/usr/share/zoneinfo`.
    #[test]
    fn a_posix_rule_names_as_glibc_parses_it() {
        let pair = |a: &str, b: &str| (a.to_owned(), b.to_owned());
        assert_eq!(rule("XYZ3"), pair("XYZ", "XYZ"));
        assert_eq!(rule("XYZ3ABC"), pair("XYZ", "ABC"));
        assert_eq!(rule("XYZ3ABC,M3.2.0,M11.1.0"), pair("XYZ", "ABC"));
        assert_eq!(rule("<+03>-3"), pair("+03", "+03"));
        assert_eq!(rule("AB3"), pair("", ""));
        assert_eq!(rule("XYZ"), pair("XYZ", ""));
        assert_eq!(rule("XYZ+3:30"), pair("XYZ", "XYZ"));
        assert_eq!(rule("XYZ-a"), pair("XYZ", ""));
        assert_eq!(rule("Universal"), pair("Universal", ""));
        assert_eq!(rule("Nowhere/X"), pair("Nowhere", ""));
    }
}
