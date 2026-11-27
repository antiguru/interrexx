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
    // Every required argument is checked before any name is converted, so a
    // missing argument raises the same error whatever the file name is.
    let all_sections = section.eq_ignore_ascii_case(b"ALL:");
    let (key, target) = if all_sections {
        (Vec::new(), Some(required(args, 3)?))
    } else {
        let key = required_c_string(interp, args, 3)?;
        let target = if key.eq_ignore_ascii_case(b"ALL:") {
            Some(required(args, 4)?)
        } else {
            None
        };
        (key, target)
    };
    let Some(file) = wide(
        file.as_deref()
            .map(c_string)
            .filter(|name| !name.is_empty())
            .unwrap_or(b"WIN.INI"),
    ) else {
        return Ok(interp.text_built(PROFILE_ERROR.to_vec()));
    };

    if all_sections {
        let target = target.expect("ALL: as the section takes a target");
        return list(interp, &file, None, target, 3);
    }
    let Some(section) = wide(&section) else {
        return Ok(interp.text_built(PROFILE_ERROR.to_vec()));
    };
    if let Some(target) = target {
        return list(interp, &file, Some(&section), target, 4);
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
        if let Err(failure) = found.add(
            interp,
            count + 1,
            String::from_utf16_lossy(name).into_bytes(),
        ) {
            // Keep the count in step with the names written before the failure.
            found.complete(interp, count);
            return Err(failure);
        }
        count += 1;
    }
    found.complete(interp, count);
    let status = if count == 0 { PROFILE_ERROR } else { b"" };
    Ok(interp.text_built(status.to_vec()))
}

fn remove_section(file: &[u16], section: &[u16]) -> Option<Vec<u8>> {
    // A section whose last key was deleted keeps its header, so existence is
    // decided by the list of section names, not by the section's keys.
    let names = host::read_profile(file, None, None, None)?;
    let name = section.strip_suffix(&[0]).unwrap_or(section);
    if !names
        .split(|&unit| unit == 0)
        .any(|candidate| host::profile_name_eq(candidate, name))
    {
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
