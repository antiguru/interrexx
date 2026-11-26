//! RexxUtil's Windows profile-file interface.

use interrexx_core::ObjRef;

use super::{c_string, check_arity, optional_text, required, required_c_string, search::Found};
use crate::{Failure, Interp, host};

const PROFILE_ERROR: &[u8] = b"ERROR:";

pub(crate) fn ini(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 4)?;
    let file = optional_text(interp, args, 1)?;
    let section = required_c_string(interp, args, 2)?;
    let Some(file) = wide(
        file.as_deref()
            .map(c_string)
            .filter(|name| !name.is_empty())
            .unwrap_or(b"WIN.INI"),
    ) else {
        return Ok(interp.text_built(PROFILE_ERROR.to_vec()));
    };

    if section.eq_ignore_ascii_case(b"ALL:") {
        let target = required(args, 3)?;
        check_arity(args, 3)?;
        return list(interp, &file, None, target, 3);
    }
    let Some(section) = wide(&section) else {
        return Ok(interp.text_built(PROFILE_ERROR.to_vec()));
    };
    let key = required_c_string(interp, args, 3)?;
    if key.eq_ignore_ascii_case(b"ALL:") {
        return list(interp, &file, Some(&section), required(args, 4)?, 4);
    }

    let result = match key.as_slice() {
        b"" => None,
        key if key.eq_ignore_ascii_case(b"DELETE:") => remove_section(&file, &section),
        key => {
            let Some(key) = wide(key) else {
                return Ok(interp.text_built(PROFILE_ERROR.to_vec()));
            };
            match optional_text(interp, args, 4)?.as_deref().map(c_string) {
                None => get_value(&file, &section, &key),
                Some(value) if value.eq_ignore_ascii_case(b"DELETE:") => {
                    if get_value(&file, &section, &key).is_none() {
                        None
                    } else {
                        write_value(&file, &section, Some(&key), None)
                    }
                }
                Some(value) => wide(value)
                    .and_then(|value| write_value(&file, &section, Some(&key), Some(&value))),
            }
        }
    };
    Ok(interp.text_built(result.unwrap_or_else(|| PROFILE_ERROR.to_vec())))
}

fn list(
    interp: &mut Interp,
    file: &[u16],
    section: Option<&[u16]>,
    target: ObjRef,
    position: usize,
) -> Result<ObjRef, Failure> {
    let found = Found::resolve(interp, target, position)?;
    let Some(names) = host::read_profile(file, section, None, None) else {
        found.complete(interp, 0);
        return Ok(interp.text_built(PROFILE_ERROR.to_vec()));
    };
    let mut count = 0;
    for name in names
        .split(|&unit| unit == 0)
        .filter(|name| !name.is_empty())
    {
        count += 1;
        found.add(interp, count, String::from_utf16_lossy(name).into_bytes())?;
    }
    found.complete(interp, count);
    let status = if count == 0 { PROFILE_ERROR } else { b"" };
    Ok(interp.text_built(status.to_vec()))
}

fn remove_section(file: &[u16], section: &[u16]) -> Option<Vec<u8>> {
    if host::read_profile(file, Some(section), None, None)?.is_empty() {
        return None;
    }
    write_value(file, section, None, None)
}

fn get_value(file: &[u16], section: &[u16], key: &[u16]) -> Option<Vec<u8>> {
    let missing = [1, 0];
    let value = host::read_profile(file, Some(section), Some(key), Some(&missing))?;
    if value != [1] {
        return Some(String::from_utf16_lossy(&value).into_bytes());
    }
    // Disambiguate a missing key from an actual value equal to the fallback.
    let value = host::read_profile(file, Some(section), Some(key), Some(&[2, 0]))?;
    (value == [1]).then_some(vec![1])
}

fn write_value(
    file: &[u16],
    section: &[u16],
    key: Option<&[u16]>,
    value: Option<&[u16]>,
) -> Option<Vec<u8>> {
    host::write_profile(file, section, key, value).then(Vec::new)
}

fn wide(bytes: &[u8]) -> Option<Vec<u16>> {
    let text = std::str::from_utf8(bytes).ok()?;
    Some(text.encode_utf16().chain([0]).collect())
}
