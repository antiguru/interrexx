//! Searching file contents and returning results in a stem or array.

use interrexx_core::ObjRef;

use super::{
    answer, c_string, check_arity, marshalled, optional_text, required, required_c_string,
};
use crate::builtin::platform::qualify_name;
use crate::error::Raised;
use crate::paths::os_path;
use crate::{Failure, Interp};

/// Where `SysFileSearch` and `SysFileTree` put what they find.
#[derive(Clone, Copy)]
pub(super) enum Found {
    /// An `Array`, emptied first and appended to.
    Array(ObjRef),
    /// A stem, written from tail 1 with the count in tail 0 at the end. Tails
    /// past the count are left as they were.
    Stem(ObjRef),
}

impl Found {
    /// The argument at `position` as a stem handler resolves it: an
    /// instance of `Array` itself, a stem, or a string naming a stem variable.
    /// 40.925 for anything else.
    pub(super) fn resolve(
        interp: &mut Interp,
        value: ObjRef,
        position: usize,
    ) -> Result<Found, Failure> {
        if interp.array_body(value).is_some() {
            let caller = interp.caller();
            interp.send_message(value, b"EMPTY", None, &[], caller)?;
            return Ok(Found::Array(value));
        }
        if interp.is_stem_object(value) {
            return Ok(Found::Stem(value));
        }
        // `ResolveStemVariable` clears whatever the string conversion raised.
        if let Ok(Some(text)) = interp.string_conversion(value) {
            let name = interp.to_text(text).into_owned();
            if let Some(stem) = interp.context_stem_named(&name) {
                return Ok(Found::Stem(stem));
            }
        }
        Err(Raised::not_an_array_or_stem(position).into())
    }

    /// Adds `line` as the `count`th value.
    pub(super) fn add(
        self,
        interp: &mut Interp,
        count: usize,
        line: Vec<u8>,
    ) -> Result<(), Failure> {
        let value = interp.text_built(line);
        match self {
            Found::Array(array) => {
                interp.array_append(array, value)?;
            }
            Found::Stem(stem) => {
                interp.stem_object_write(stem, count.to_string().as_bytes(), value)
            }
        }
        Ok(())
    }

    /// Stores a stem's count into tail 0.
    pub(super) fn complete(self, interp: &mut Interp, count: usize) {
        if let Found::Stem(stem) = self {
            let count = answer(interp, i64::try_from(count).unwrap_or(i64::MAX));
            interp.stem_object_write(stem, b"0", count);
        }
    }
}

/// The lines `LineReader` reads: each ends at a LF, which is dropped with one
/// CR before it, and a last line with no LF ends at the end of the file.
fn lines(contents: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut rest = contents;
    std::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        match rest.iter().position(|byte| *byte == b'\n') {
            Some(end) => {
                let line = &rest[..end];
                rest = &rest[end + 1..];
                Some(line.strip_suffix(b"\r").unwrap_or(line))
            }
            None => Some(std::mem::take(&mut rest)),
        }
    })
}

/// `mystrstr`: whether `needle` occurs in `line`, ASCII case folded when
/// `caseless`. An empty needle occurs nowhere.
fn contains(line: &[u8], needle: &[u8], caseless: bool) -> bool {
    if needle.is_empty() || needle.len() > line.len() {
        return false;
    }
    line.windows(needle.len()).any(|window| {
        if caseless {
            window.eq_ignore_ascii_case(needle)
        } else {
            window == needle
        }
    })
}

/// The longest message `SysFileSearch` formats for a bad option: its
/// `snprintf` buffer less the NUL.
pub(super) const OPTION_MESSAGE_LIMIT: usize = 255;

/// `SysFileSearch(needle, file, stem [, options])`: the lines of `file`
/// holding `needle`, into a stem or an array, each after its line number and
/// a blank under option `N`, matched without case unless option `C`. Answers
/// `0`, or `3` where the file is a directory or does not open, with nothing
/// found.
pub(crate) fn file_search(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 4)?;
    let needle = required(args, 1)?;
    let Some(needle) = interp.string_conversion(needle)? else {
        return Err(marshalled(Raised::argument_needs_a_string_value(1)));
    };
    let needle = interp.to_text(needle).into_owned();
    let file = required_c_string(interp, args, 2)?;
    let target = required(args, 3)?;
    let options = optional_text(interp, args, 4)?;

    let (mut numbered, mut caseless) = (false, true);
    if let Some(options) = &options {
        let options = c_string(options);
        for letter in options {
            match letter.to_ascii_uppercase() {
                b'N' => numbered = true,
                b'I' => caseless = true,
                b'C' => caseless = false,
                _ => {
                    let mut text = b"SysFileSearch options argument must be a combination of \
                                     C, I, or N; found \""
                        .to_vec();
                    text.extend_from_slice(options);
                    text.push(b'"');
                    text.truncate(OPTION_MESSAGE_LIMIT);
                    return Err(Raised::routine_user_defined(&text).into());
                }
            }
        }
    }

    let found = Found::resolve(interp, target, 3)?;
    let path = qualify_name(interp, &file);
    let path = os_path(&path);
    let contents = if path.is_dir() {
        None
    } else {
        std::fs::read(path).ok()
    };
    let Some(contents) = contents else {
        found.complete(interp, 0);
        return Ok(interp.text(b"3"));
    };
    let mut count = 0;
    for (index, line) in lines(&contents).enumerate() {
        if !contains(line, &needle, caseless) {
            continue;
        }
        let mut value = Vec::with_capacity(line.len() + 8);
        if numbered {
            value.extend_from_slice(format!("{} ", index + 1).as_bytes());
        }
        value.extend_from_slice(line);
        count += 1;
        found.add(interp, count, value)?;
    }
    found.complete(interp, count);
    Ok(interp.text(b"0"))
}

/// `SysSearchPath(variable, file [, option])`: the qualified name of the first
/// regular file called `file` along the directories the environment variable
/// lists, the current directory first unless the option starts `N`. The null
/// string where it is not found.
pub(crate) fn search_path(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 3)?;
    let variable = required_c_string(interp, args, 1)?;
    let file = required_c_string(interp, args, 2)?;
    let current_first = match optional_text(interp, args, 3)? {
        None => true,
        Some(text) => {
            let option = c_string(&text);
            match option.first().map(u8::to_ascii_uppercase) {
                Some(b'C') => true,
                Some(b'N') => false,
                _ => {
                    return Err(Raised::invalid_routine_option(
                        "SysSearchPath",
                        "option",
                        "'C' or 'N'",
                        option,
                    )
                    .into());
                }
            }
        }
    };
    let value = interp.env_get(&variable).unwrap_or_default();
    let found = search_path_for_file(interp, &file, value, current_first);
    Ok(interp.text_built(found))
}

/// The search-path lookup. A name starting `/` or `~` is only qualified.
/// Otherwise, `./name` and `../name` included, each non-empty entry of `path`
/// is tried in turn, and the first that names anything ends the search: a
/// regular file is the answer, anything else is not found.
#[cfg(unix)]
fn search_path_for_file(interp: &Interp, file: &[u8], path: &[u8], current_first: bool) -> Vec<u8> {
    let has_directory = file.first() == Some(&b'~') || file.first() == Some(&b'/');
    if has_directory {
        let qualified = qualify_name(interp, file);
        return if os_path(&qualified).is_file() {
            qualified
        } else {
            Vec::new()
        };
    }
    let current_and_path;
    let path = if current_first {
        let mut combined = interp.cwd_bytes().to_vec();
        if !path.is_empty() {
            combined.push(b':');
            combined.extend_from_slice(path);
        }
        current_and_path = combined;
        &current_and_path
    } else {
        path
    };
    for entry in path.split(|byte| *byte == b':') {
        if entry.is_empty() {
            continue;
        }
        let mut candidate = entry.to_vec();
        if candidate.last() != Some(&b'/') {
            candidate.push(b'/');
        }
        candidate.extend_from_slice(file);
        let qualified = qualify_name(interp, &candidate);
        if qualified.is_empty() {
            continue;
        }
        if let Ok(metadata) = std::fs::metadata(os_path(&qualified)) {
            return if metadata.is_file() {
                qualified
            } else {
                Vec::new()
            };
        }
    }
    Vec::new()
}

#[cfg(windows)]
fn search_path_for_file(interp: &Interp, file: &[u8], path: &[u8], current_first: bool) -> Vec<u8> {
    let file_path = os_path(file);
    if file.is_empty() || file.iter().any(|byte| matches!(byte, b'*' | b'?')) {
        return Vec::new();
    }
    if file_path.has_root()
        || matches!(
            file_path.components().next(),
            Some(std::path::Component::Prefix(_))
        )
    {
        let qualified = qualify_name(interp, file);
        return if os_path(&qualified).is_file() {
            qualified
        } else {
            Vec::new()
        };
    }

    let current = current_first.then(|| os_path(interp.cwd_bytes()).to_path_buf());
    let directories = std::env::split_paths(os_path(path).as_os_str());
    for directory in current.into_iter().chain(directories) {
        if directory.as_os_str().is_empty() {
            continue;
        }
        let candidate = directory.join(file_path);
        let Some(candidate) = candidate.to_str() else {
            continue;
        };
        let qualified = qualify_name(interp, candidate.as_bytes());
        if os_path(&qualified).is_file() {
            return qualified;
        }
    }
    Vec::new()
}
