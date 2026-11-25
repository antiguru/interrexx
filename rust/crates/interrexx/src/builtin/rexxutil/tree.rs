//! Argument handling shared by the host implementations of `SysFileTree`.

use interrexx_core::ObjRef;

use super::search::{Found, OPTION_MESSAGE_LIMIT};
use super::{answer, c_string, check_arity, optional_text, required, required_c_string};
use crate::error::Raised;
use crate::{Failure, Interp};

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use unix as platform;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows as platform;

/// What one call looks for and how it reports what it finds, from the option
/// letters.
#[derive(Clone, Copy)]
struct TreeOptions {
    files: bool,
    directories: bool,
    recurse: bool,
    /// `O`: the name alone, with no date, size or attributes.
    names_only: bool,
    /// `T`: the date as `YY/MM/DD/HH/MM`.
    editable_time: bool,
    /// `L`: the date as `YYYY-MM-DD HH:MM:SS`, over `T`.
    long_time: bool,
    /// `H`: the size twenty wide and never capped.
    long_size: bool,
    /// `I`: the final name matched without case.
    #[cfg(unix)]
    caseless: bool,
}

impl TreeOptions {
    /// The letters in any case, each of `F`, `D` and
    /// `B` replacing what an earlier one chose. `None` for any other letter.
    fn parse(text: &[u8]) -> Option<TreeOptions> {
        let mut options = TreeOptions {
            files: true,
            directories: true,
            recurse: false,
            names_only: false,
            editable_time: false,
            long_time: false,
            long_size: false,
            #[cfg(unix)]
            caseless: false,
        };
        for letter in text {
            match letter.to_ascii_uppercase() {
                b'F' => (options.files, options.directories) = (true, false),
                b'D' => (options.files, options.directories) = (false, true),
                b'B' => (options.files, options.directories) = (true, true),
                b'S' => options.recurse = true,
                b'O' => options.names_only = true,
                b'T' => options.editable_time = true,
                b'L' => options.long_time = true,
                b'H' => options.long_size = true,
                #[cfg(unix)]
                b'I' => options.caseless = true,
                #[cfg(windows)]
                b'I' => {}
                _ => return None,
            }
        }
        Some(options)
    }
}

/// A 40.900 whose message `snprintf` formats into 256 bytes.
fn user_defined(mut text: Vec<u8>) -> Failure {
    text.truncate(OPTION_MESSAGE_LIMIT);
    Raised::routine_user_defined(&text).into()
}

/// An attribute mask, cut at a NUL, is empty or up
/// to five of `+`, `-` and `*`. Unix applies neither mask.
fn check_mask(mask: Option<&[u8]>, position: usize) -> Result<(), Failure> {
    let mask = mask.map_or(&[][..], c_string);
    if mask.len() > 5 || mask.iter().any(|byte| !matches!(byte, b'+' | b'-' | b'*')) {
        let mut text = format!(
            "SysFileTree argument {position} must be 5 characters or less in length \
             containing only '+', '-', or '*'; found \""
        )
        .into_bytes();
        text.extend_from_slice(mask);
        text.push(b'"');
        return Err(user_defined(text));
    }
    Ok(())
}

/// `SysFileTree(filespec, stem [, options [, tattrib [, nattrib]]])`: the
/// names matching `filespec`, into a stem or an array. The directory part of
/// the spec is literal and only its final name may hold wildcards.
///
/// Errors after the stem is resolved still set its tail `0` to `0`, as
/// `StemHandler`'s destructor does. The order entries are found in is the
/// directory's own, which is not specified.
pub(crate) fn file_tree(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 5)?;
    let spec = required_c_string(interp, args, 1)?;
    let target = required(args, 2)?;
    let options = optional_text(interp, args, 3)?;
    let target_mask = optional_text(interp, args, 4)?;
    let new_mask = optional_text(interp, args, 5)?;
    let found = Found::resolve(interp, target, 2)?;

    let options = match validate(
        &spec,
        options.as_deref(),
        target_mask.as_deref(),
        new_mask.as_deref(),
    ) {
        Ok(options) => options,
        Err(failure) => {
            found.complete(interp, 0);
            return Err(failure);
        }
    };
    let lines = platform::find(
        interp,
        &spec,
        options,
        target_mask.as_deref().map(c_string).unwrap_or_default(),
        new_mask.as_deref().map(c_string).unwrap_or_default(),
    );
    let count = lines.len();
    for (index, line) in lines.into_iter().enumerate() {
        found.add(interp, index + 1, line)?;
    }
    found.complete(interp, count);
    Ok(answer(interp, 0))
}

/// The arguments' checks in the order `TreeFinder`'s constructor makes them:
/// the spec, the options, then the two masks.
fn validate(
    spec: &[u8],
    options: Option<&[u8]>,
    target_mask: Option<&[u8]>,
    new_mask: Option<&[u8]>,
) -> Result<TreeOptions, Failure> {
    if spec.is_empty() {
        return Err(Raised::routine_null_argument("SysFileTree", 1).into());
    }
    let options = match options.map(c_string) {
        None => TreeOptions::parse(b"").expect("no letters is valid"),
        Some([]) => return Err(Raised::routine_null_argument("SysFileTree", 3).into()),
        Some(letters) => TreeOptions::parse(letters).ok_or_else(|| {
            let mut text = b"SysFileTree options argument must be a combination of \
                             F, D, B, S, T, L, I, O, or H; found \""
                .to_vec();
            text.extend_from_slice(letters);
            text.push(b'"');
            user_defined(text)
        })?,
    };
    check_mask(target_mask, 4)?;
    check_mask(new_mask, 5)?;
    Ok(options)
}
