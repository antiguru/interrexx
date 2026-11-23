//! Argument conversion shared by native extension packages.

use interrexx_core::{Decoded, ObjRef};
use interrexx_num::{DIGITS64, Number};

use super::rxmath::required;
use crate::error::Raised;
use crate::{Failure, Interp};

type Result<T> = std::result::Result<T, Failure>;

/// Error 40.1, whose insert is the routine's name as called.
pub(super) fn failed(interp: &Interp) -> Failure {
    Raised::routine_failed(&interp.native_frame().name.to_ascii_uppercase()).into()
}

/// A string argument as C reads it: the string value the native boundary
/// requests, 88.909 where there is none, cut at its first NUL.
pub(super) fn c_string(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Vec<u8>> {
    let value = required(args, position)?;
    let Some(text) = interp.blamed_string_conversion(value)? else {
        return Err(Raised::native_argument_needs_a_string_value(position).into());
    };
    let mut bytes = interp.to_text(text).into_owned();
    if let Some(end) = bytes.iter().position(|byte| *byte == 0) {
        bytes.truncate(end);
    }
    Ok(bytes)
}

/// A 32-bit integer argument: 88.907 for anything that is not a whole number
/// from -2147483648 to 2147483647.
pub(super) fn int(interp: &mut Interp, args: &[Option<ObjRef>], position: usize) -> Result<i32> {
    let value = required(args, position)?;
    let number = match value.decode() {
        Decoded::SmallInt(number) => Some(number),
        _ => match interp.blamed_string_conversion(value)? {
            Some(text) => Number::parse_bytes(&interp.to_text(text))
                .and_then(|number| number.int64_value(DIGITS64)),
            None => None,
        },
    };
    if let Some(number) = number.and_then(|number| i32::try_from(number).ok()) {
        return Ok(number);
    }
    let found = interp.native_found(value);
    Err(
        Raised::native_argument_outside_range(position, i32::MIN.into(), i32::MAX.into(), &found)
            .into(),
    )
}

/// A whole number as the routine's answer.
pub(super) fn number(interp: &mut Interp, value: impl Into<i128>) -> ObjRef {
    let value = value.into();
    match i64::try_from(value).ok().and_then(ObjRef::small_int) {
        Some(small) => small,
        None => interp.text_built(value.to_string().into_bytes()),
    }
}
