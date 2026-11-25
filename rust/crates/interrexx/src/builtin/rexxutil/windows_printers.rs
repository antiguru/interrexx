//! RexxUtil routines over the Windows print spooler.

use interrexx_core::ObjRef;
use windows_sys::Win32::Foundation::ERROR_INVALID_PRINTER_NAME;

use super::{answer, check_arity, required, required_c_string, search::Found};
use crate::{Failure, Interp, host};

pub(crate) fn get_default(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 0)?;
    let description = host::default_printer()
        .and_then(|name| {
            host::printers()
                .ok()?
                .into_iter()
                .find(|entry| entry[0] == name)
        })
        .map_or_else(Vec::new, |entry| entry.join(",").into_bytes());
    Ok(interp.text_built(description))
}

pub(crate) fn list(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let target = required(args, 1)?;
    let found = Found::resolve(interp, target, 1)?;
    let Ok(entries) = host::printers() else {
        found.complete(interp, 0);
        return Ok(answer(interp, 1));
    };
    let count = entries.len();
    for (index, entry) in entries.into_iter().enumerate() {
        found.add(interp, index + 1, entry.join(",").into_bytes())?;
    }
    found.complete(interp, count);
    Ok(answer(interp, 0))
}

pub(crate) fn set_default(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let description = required_c_string(interp, args, 1)?;
    let status = std::str::from_utf8(&description)
        .ok()
        .filter(|description| !description.is_empty())
        .map_or(ERROR_INVALID_PRINTER_NAME, |description| {
            // A description may come back from this package with driver and
            // port fields. Try the complete name first, since it may contain commas.
            let status = host::set_default_printer(description);
            if status == ERROR_INVALID_PRINTER_NAME
                && let Some((name, _)) = description
                    .rsplit_once(',')
                    .and_then(|(left, _)| left.rsplit_once(','))
                && !name.is_empty()
            {
                return host::set_default_printer(name);
            }
            status
        });
    Ok(answer(interp, i64::from(status)))
}
