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
    let description = host::default_printer().map_or_else(Vec::new, |name| {
        host::printers()
            .ok()
            .and_then(|entries| {
                entries
                    .into_iter()
                    .find(|entry| host::printer_name_eq(&entry[0], &name))
            })
            .map_or_else(
                || format!("{name},,").into_bytes(),
                |entry| entry.join(",").into_bytes(),
            )
    });
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
    let status = host::legacy_windows_text(&description)
        .filter(|text| !text.is_empty())
        .map_or(ERROR_INVALID_PRINTER_NAME, |text| {
            // The first comma delimits the printer name. The port field
            // can itself contain several comma-separated ports.
            let name = text.split_once(',').map_or(text.as_str(), |(name, _)| name);
            if name.is_empty() {
                ERROR_INVALID_PRINTER_NAME
            } else {
                host::set_default_printer(name)
            }
        });
    Ok(answer(interp, i64::from(status)))
}
