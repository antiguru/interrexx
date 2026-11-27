//! Unix directory enumeration and output for `SysFileTree`.

use rustix::fs::{FileType, Stat};

use super::super::{
    files::{CivilTime, local_time, program_zone},
    os_path,
};
use super::TreeOptions;
use crate::Interp;
use crate::builtin::platform::{expand_tilde, qualify_name};

pub(super) fn find(
    interp: &Interp,
    spec: &[u8],
    options: TreeOptions,
    _target_mask: &[u8],
    _new_mask: &[u8],
) -> Vec<Vec<u8>> {
    let (spec, unresolved) = adjust_directory(interp, spec);
    let zone = program_zone(interp);
    let mut lines = Vec::new();
    if let Some((directory, pattern)) = full_path(interp, &spec, unresolved) {
        search(&directory, &pattern, options, &zone, &mut lines);
    }
    lines
}

/// Canonicalisation of a name starting with `~`: the
/// tilde resolved, and the result normalised where it is absolute. `None`
/// where the tilde does not resolve: an unknown user, or `HOME` unset.
fn canonicalize_tilde(interp: &Interp, name: &[u8]) -> Option<Vec<u8>> {
    let expanded = expand_tilde(interp, name)?;
    Some(if expanded.first() == Some(&b'/') {
        crate::paths::normalize_bytes(&expanded, b"/")
    } else {
        expanded
    })
}

/// `*` becomes `./*`, a spec ending in `/`,
/// `/.` or `/..` gains a wildcard, and a leading `~` is resolved. The flag is
/// set where a leading `~` did not resolve and the spec is kept as it is.
fn adjust_directory(interp: &Interp, spec: &[u8]) -> (Vec<u8>, bool) {
    let mut spec = spec.to_vec();
    if spec == b"*" {
        spec = b"./*".to_vec();
    } else if spec.ends_with(b"/") {
        spec.push(b'*');
    } else if spec.ends_with(b"/.") || spec.ends_with(b"/..") {
        spec.extend_from_slice(b"/*");
    }
    if spec.first() == Some(&b'~') {
        match canonicalize_tilde(interp, &spec) {
            Some(resolved) => spec = resolved,
            None => return (spec, true),
        }
    }
    (spec, false)
}

/// The directory to search, ending in `/`, and
/// the pattern for the names in it.
///
/// A spec with no `/` is searched for in the current directory, unqualified,
/// and `..` searches the directory named by the current one's name with `..`
/// appended. Measured, `SysFileTree('..', s.)` run from `/` lists `/../bin`.
/// The directory part of any other spec is qualified, and where that fails
/// the search is in `/`. A spec whose `~` did not resolve (`unresolved`) is
/// searched for in `/` when one component follows its first slash, and
/// `None`, no search, when more do.
fn full_path(interp: &Interp, spec: &[u8], unresolved: bool) -> Option<(Vec<u8>, Vec<u8>)> {
    let (mut directory, pattern) = match spec.iter().rposition(|byte| *byte == b'/') {
        None => {
            let mut directory = interp.cwd_bytes().to_vec();
            let pattern = match spec {
                b"." => b"*".to_vec(),
                b".." => {
                    directory.extend_from_slice(b"..");
                    b"*".to_vec()
                }
                _ => spec.to_vec(),
            };
            (directory, pattern)
        }
        Some(slash) if unresolved && spec[..slash].contains(&b'/') => return None,
        Some(slash) => {
            let pattern = if spec.ends_with(b"/") {
                b"*".to_vec()
            } else {
                spec[slash + 1..].to_vec()
            };
            (qualify_name(interp, &spec[..=slash]), pattern)
        }
    };
    if directory.last() != Some(&b'/') {
        directory.push(b'/');
    }
    Some((directory, pattern))
}

/// The entries of `directory` whose names
/// match `pattern`, then, under `S`, every subdirectory's the same way. A
/// directory that does not open contributes nothing.
fn search(
    directory: &[u8],
    pattern: &[u8],
    options: TreeOptions,
    zone: &tz::TimeZone,
    out: &mut Vec<Vec<u8>>,
) {
    for (name, stat) in entries(directory, |name| matches(pattern, name, options.caseless)) {
        let mut path = directory.to_vec();
        path.extend_from_slice(&name);
        if let Some(line) = entry_line(options, zone, path, &stat) {
            out.push(line);
        }
    }
    if options.recurse {
        for (name, stat) in entries(directory, |_| true) {
            if FileType::from_raw_mode(stat.st_mode) != FileType::Directory {
                continue;
            }
            let mut path = directory.to_vec();
            path.extend_from_slice(&name);
            path.push(b'/');
            search(&path, pattern, options, zone, out);
        }
    }
}

/// The names in `directory` other than `.` and `..` that `wanted` accepts, each
/// with what `stat` answers for it, following a symbolic link. Only those names
/// are stat'ed. Where `stat` fails (a dangling link, a chain of links past the
/// kernel's limit) it is `lstat`'s answer; the reference run reports whatever
/// its buffer last held (oracle defect, `corpus/oracle-crashes.txt` entry 40).
/// A name both fail on, removed since it was read, is left out.
fn entries(directory: &[u8], wanted: impl Fn(&[u8]) -> bool) -> Vec<(Vec<u8>, Stat)> {
    use std::os::unix::ffi::OsStrExt as _;
    let Ok(listing) = std::fs::read_dir(os_path(directory)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in listing.flatten() {
        let name = entry.file_name().as_bytes().to_vec();
        if !wanted(&name) {
            continue;
        }
        let mut path = directory.to_vec();
        path.extend_from_slice(&name);
        let stat =
            rustix::fs::stat(path.as_slice()).or_else(|_| rustix::fs::lstat(path.as_slice()));
        if let Ok(stat) = stat {
            out.push((name, stat));
        }
    }
    out
}

/// The reading for a time the calendar cannot hold.
const EPOCH: CivilTime = CivilTime {
    year: 1970,
    month0: 0,
    day: 1,
    hour: 0,
    minute: 0,
    second: 0,
};

/// The line for one
/// entry, or `None` where `F` or `D` leaves its kind out. The qualified name
/// alone under `O`, otherwise the date, the size, the mode string and the
/// name, two blanks after each of the first three.
fn entry_line(
    options: TreeOptions,
    zone: &tz::TimeZone,
    path: Vec<u8>,
    stat: &Stat,
) -> Option<Vec<u8>> {
    let kind = FileType::from_raw_mode(stat.st_mode);
    let wanted = if kind == FileType::Directory {
        options.directories
    } else {
        options.files
    };
    if !wanted {
        return None;
    }
    if options.names_only {
        return Some(path);
    }
    let when = local_time(zone, stat.st_mtime).unwrap_or(EPOCH);
    let month = when.month0 + 1;
    let two_digit_year = (when.year - 1900 + 10_000) % 100;
    let mut out = if options.long_time {
        format!(
            "{:4}-{month:02}-{:02} {:02}:{:02}:{:02}  ",
            when.year, when.day, when.hour, when.minute, when.second
        )
    } else if options.editable_time {
        format!(
            "{two_digit_year:02}/{month:02}/{:02}/{:02}/{:02}  ",
            when.day, when.hour, when.minute
        )
    } else {
        let hour = if when.hour < 13 && when.hour != 0 {
            when.hour
        } else {
            (when.hour - 12).abs()
        };
        let meridiem = if when.hour < 12 { 'a' } else { 'p' };
        format!(
            "{month:2}/{:02}/{two_digit_year:02}  {hour:2}:{:02}{meridiem}  ",
            when.day, when.minute
        )
    };
    if options.long_size {
        out.push_str(&format!("{:20}  ", stat.st_size));
    } else {
        out.push_str(&format!("{:10}  ", stat.st_size.min(9_999_999_999)));
    }
    let mut out = out.into_bytes();
    out.extend_from_slice(&mode_string(kind, crate::host::mode_bits(stat.st_mode)));
    out.extend_from_slice(b"  ");
    out.extend_from_slice(&path);
    Some(out)
}

/// The ten-character mode string `ls -l` shows: the kind, then each class's
/// permissions with set-user-ID, set-group-ID and sticky folded into the
/// execute letters.
fn mode_string(kind: FileType, mode: u32) -> [u8; 10] {
    let letter = match kind {
        FileType::Symlink => b'l',
        FileType::BlockDevice => b'b',
        FileType::CharacterDevice => b'c',
        FileType::Directory => b'd',
        FileType::Fifo => b'p',
        FileType::Socket => b's',
        FileType::RegularFile | FileType::Unknown => b'-',
    };
    let bit = |mask: u32, set: u8| if mode & mask == 0 { b'-' } else { set };
    let execute = |special: u32, execute: u32, both: u8, special_only: u8| match (
        mode & special != 0,
        mode & execute != 0,
    ) {
        (true, true) => both,
        (true, false) => special_only,
        (false, true) => b'x',
        (false, false) => b'-',
    };
    [
        letter,
        bit(0o400, b'r'),
        bit(0o200, b'w'),
        execute(0o4000, 0o100, b's', b'S'),
        bit(0o040, b'r'),
        bit(0o020, b'w'),
        execute(0o2000, 0o010, b's', b'S'),
        bit(0o004, b'r'),
        bit(0o002, b'w'),
        execute(0o1000, 0o001, b't', b'T'),
    ]
}

/// `fnmatch(3)` with `FNM_PATHNAME | FNM_NOESCAPE` in the C locale, over one
/// name, folding ASCII case under `caseless` as `FNM_CASEFOLD` does: `*`,
/// `?`, and a bracket expression with ranges, `[:class:]`, `[.c.]` and
/// `[=c=]` forms and a leading `!` or `^`. A backslash is an ordinary byte, and a leading `.` needs no
/// literal match.
fn matches(pattern: &[u8], name: &[u8], caseless: bool) -> bool {
    let fold = |byte: u8| {
        if caseless {
            byte.to_ascii_lowercase()
        } else {
            byte
        }
    };
    // The classic backtracking walk: `star` remembers where the last `*` was
    // so a failed tail can resume one byte later.
    let (mut pattern_at, mut name_at) = (0, 0);
    let (mut star, mut resume) = (None, 0);
    while name_at < name.len() {
        let step = match pattern.get(pattern_at) {
            Some(b'*') => {
                star = Some(pattern_at);
                resume = name_at;
                pattern_at += 1;
                continue;
            }
            Some(b'?') => Some(pattern_at + 1),
            Some(b'[') => bracket(pattern, pattern_at, name[name_at], caseless),
            Some(byte) if fold(*byte) == fold(name[name_at]) => Some(pattern_at + 1),
            _ => None,
        };
        match (step, star) {
            (Some(next), _) => {
                pattern_at = next;
                name_at += 1;
            }
            (None, Some(at)) => {
                pattern_at = at + 1;
                resume += 1;
                name_at = resume;
            }
            (None, None) => return false,
        }
    }
    pattern[pattern_at..].iter().all(|byte| *byte == b'*')
}

/// The longest character class name glibc's `fnmatch` scans for.
const CLASS_NAME_LIMIT: usize = 256;

/// One bracket expression opening at `open` against one byte: the index just
/// past its `]` when it matches, `None` when it does not. A port of the bracket
/// case of glibc's `posix/fnmatch_loop.c` (2.43) in the C locale with
/// `FNM_NOESCAPE`. Each item is tried in turn and the first that matches ends
/// the scan, which then skips to the `]` and fails the pattern on a malformed
/// `[=` or `[.` it passes over. A bracket with no `]` is a literal `[`. A `^`
/// negates as `!` does, which glibc does unless `POSIXLY_CORRECT` is set.
fn bracket(pattern: &[u8], open: usize, byte: u8, caseless: bool) -> Option<usize> {
    // The pattern is a C string: past its end every read is a NUL.
    let at = |index: usize| pattern.get(index).copied().unwrap_or(0);
    let fold = |value: u8| {
        if caseless {
            value.to_ascii_lowercase()
        } else {
            value
        }
    };
    let literal = (byte == b'[').then_some(open + 1);
    let folded = fold(byte);
    let mut p = open + 1;
    let negated = matches!(at(p), b'!' | b'^');
    if negated {
        p += 1;
    }
    let mut c = at(p);
    p += 1;
    let matched = loop {
        // `Some(item)` runs glibc's `normal_bracket` on `item`.
        let mut normal = None;
        // The low end of a range, when the item can start one.
        let mut low = None;
        if c == b'[' && at(p) == b':' {
            let start = p;
            let mut class = Vec::new();
            let is_name = loop {
                if class.len() == CLASS_NAME_LIMIT {
                    return None;
                }
                p += 1;
                c = at(p);
                if c == b':' && at(p + 1) == b']' {
                    p += 2;
                    break true;
                }
                if !in_class_name(c) {
                    break false;
                }
                class.push(c);
            };
            if is_name {
                if !is_class(&class) {
                    return None;
                }
                if in_class(&class, byte) {
                    break true;
                }
                c = at(p);
                p += 1;
            } else {
                p = start;
                normal = Some(b'[');
            }
        } else if c == b'[' && at(p) == b'=' {
            let start = p;
            let named = at(p + 1);
            if named != 0 && at(p + 2) == b'=' && at(p + 3) == b']' {
                p += 4;
                if byte == named {
                    break true;
                }
                c = at(p);
                p += 1;
            } else {
                p = start;
                normal = Some(b'[');
            }
        } else if c == 0 {
            return literal;
        } else if c == b'[' && at(p) == b'.' {
            let (symbol, after) = collating(pattern, p)?;
            p = after;
            let is_range = at(p) == b'-' && at(p + 1) != 0;
            if !is_range && byte == symbol {
                break true;
            }
            low = Some(symbol);
            c = at(p);
            p += 1;
        } else {
            normal = Some(fold(c));
        }
        if let Some(item) = normal {
            let is_range = at(p) == b'-' && at(p + 1) != 0 && at(p + 1) != b']';
            if !is_range && item == folded {
                break true;
            }
            low = Some(item);
            c = at(p);
            p += 1;
        }
        if let Some(low) = low
            && c == b'-'
            && at(p) != b']'
        {
            let mut high = at(p);
            p += 1;
            if high == b'[' && at(p) == b'.' {
                let (symbol, after) = collating(pattern, p)?;
                high = symbol;
                p = after;
            } else {
                if high == 0 {
                    return None;
                }
                high = fold(high);
            }
            if low <= folded && folded <= high {
                break true;
            }
            c = at(p);
            p += 1;
        }
        if c == b']' {
            break false;
        }
    };
    if !matched {
        return negated.then_some(p);
    }
    // Skip the rest of a bracket that matched.
    loop {
        c = at(p);
        p += 1;
        match c {
            b']' => break,
            0 => return literal,
            b'[' if at(p) == b':' => {
                let start = p;
                let mut scanned = 0;
                loop {
                    p += 1;
                    c = at(p);
                    scanned += 1;
                    if scanned == CLASS_NAME_LIMIT {
                        return None;
                    }
                    if at(p) == b':' && at(p + 1) == b']' {
                        break;
                    }
                    if !in_class_name(c) {
                        p = start - 2;
                        break;
                    }
                }
                p += 2;
            }
            b'[' if at(p) == b'=' => {
                if at(p + 1) == 0 || at(p + 2) != b'=' || at(p + 3) != b']' {
                    return None;
                }
                p += 4;
            }
            b'[' if at(p) == b'.' => {
                p = collating_end(pattern, p)?;
            }
            _ => {}
        }
    }
    (!negated).then_some(p)
}

/// Whether `c` may continue a `[:` class name: glibc tests
/// `c < 'a' || c >= 'z'`, which leaves `z` out.
fn in_class_name(c: u8) -> bool {
    (b'a'..=b'y').contains(&c)
}

/// The index just past the `.]` closing a `[.` whose `.` is at `dot`.
fn collating_end(pattern: &[u8], dot: usize) -> Option<usize> {
    let length = pattern
        .get(dot + 1..)?
        .windows(2)
        .position(|pair| pair == b".]")?;
    let symbol = &pattern[dot + 1..dot + 1 + length];
    // A NUL ends the C string before any `.]` past it.
    (!symbol.contains(&0)).then_some(dot + 1 + length + 2)
}

/// The byte a `[.c.]` whose `.` is at `dot` names and the index past it.
/// `None`, which fails the pattern, where it does not close or names other
/// than one byte, the only names the C locale has.
fn collating(pattern: &[u8], dot: usize) -> Option<(u8, usize)> {
    let after = collating_end(pattern, dot)?;
    (after - dot == 4).then(|| (pattern[dot + 1], after))
}

/// Whether `name` is a character class the C locale defines.
fn is_class(name: &[u8]) -> bool {
    matches!(
        name,
        b"alnum"
            | b"alpha"
            | b"blank"
            | b"cntrl"
            | b"digit"
            | b"graph"
            | b"lower"
            | b"print"
            | b"punct"
            | b"space"
            | b"upper"
            | b"xdigit"
    )
}

/// Whether `byte` is in the C locale's character class `name`.
fn in_class(name: &[u8], byte: u8) -> bool {
    match name {
        b"alnum" => byte.is_ascii_alphanumeric(),
        b"alpha" => byte.is_ascii_alphabetic(),
        b"blank" => byte == b' ' || byte == b'\t',
        b"cntrl" => byte.is_ascii_control(),
        b"digit" => byte.is_ascii_digit(),
        b"graph" => byte.is_ascii_graphic(),
        b"lower" => byte.is_ascii_lowercase(),
        b"print" => byte.is_ascii_graphic() || byte == b' ',
        b"punct" => byte.is_ascii_punctuation(),
        b"space" => matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'),
        b"upper" => byte.is_ascii_uppercase(),
        b"xdigit" => byte.is_ascii_hexdigit(),
        _ => false,
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
