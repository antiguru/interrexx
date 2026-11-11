//! The macrospace routines (`runtime/RexxUtilCommon.cpp`), over the API service's macrospace.
//! A macro's image is this crate's compiled form of its source. Reads ask only a service
//! already running: with none, the macrospace is empty.

use interrexx_core::ObjRef;
use interrexx_rxapi::macro_file;
use interrexx_rxapi::message::code::{MACRO_FILE_ERROR, MACRO_NOT_FOUND, NO_SERVICE, OK};
use interrexx_rxapi::message::{Reply, Request};

use super::{answer, check_arity, path_argument, required_c_string};
use crate::error::Raised;
use crate::rxapi_client::{self, ApiClient};
use crate::{Failure, Interp};

/// `RXMACRO_SEARCH_BEFORE`.
pub(crate) const BEFORE: u64 = 1;
/// `RXMACRO_SEARCH_AFTER`.
pub(crate) const AFTER: u64 = 2;
/// `RXMACRO_SOURCE_NOT_FOUND`.
pub(crate) const SOURCE_NOT_FOUND: u64 = 7;

fn status_of(reply: Result<Reply, u64>) -> u64 {
    match reply {
        Ok(
            Reply::Status(status)
            | Reply::MacroPosition { status, .. }
            | Reply::MacroImage { status, .. }
            | Reply::Macros { status, .. },
        ) => status,
        Ok(_) => OK,
        Err(status) => status,
    }
}

/// `request`'s reply from a service already running, or `absent` with none.
fn ask_running(request: Request, absent: u64) -> u64 {
    match rxapi_client::running_client() {
        Some(client) => status_of(client.call(request)),
        None => absent,
    }
}

fn code(interp: &mut Interp, status: u64) -> ObjRef {
    answer(interp, i64::try_from(status).unwrap_or(i64::MAX))
}

/// The order an `A` or `B` option names, or 40.920.
fn order(routine: &str, option: &[u8]) -> Result<u64, Failure> {
    match option.first() {
        Some(b'B' | b'b') => Ok(BEFORE),
        Some(b'A' | b'a') => Ok(AFTER),
        _ => Err(Raised::invalid_routine_option(routine, "order", "'A' or 'B'", option).into()),
    }
}

/// The image of the program in `path`, or `None` where it cannot be read or does not translate.
/// A compiled program is its own image.
pub(crate) fn translate(path: &[u8], name: &[u8]) -> Option<Vec<u8>> {
    let text = std::fs::read(crate::paths::os_path(path)).ok()?;
    if text.is_empty() {
        return None;
    }
    let source = crate::compiled::program_source(text.clone(), path).ok()?;
    interrexx_parse::program_from(source).ok()?;
    Some(match crate::compiled::restore(&text) {
        crate::compiled::Restore::Source => crate::compiled::write_image(&text, name, false),
        _ => text,
    })
}

/// `SysAddRexxMacro(name, file [, order])`.
pub(crate) fn add(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 3)?;
    let name = required_c_string(interp, args, 1)?;
    let file = path_argument(interp, args, 2)?;
    let position = match super::optional_text(interp, args, 3)? {
        Some(option) => order("SysAddRexxMacro", super::c_string(&option))?,
        None => BEFORE,
    };
    let Some(image) = translate(&file, &name) else {
        return Ok(code(interp, SOURCE_NOT_FOUND));
    };
    let status = match rxapi_client::client() {
        Ok(client) => status_of(client.call(Request::AddMacro {
            name,
            image,
            position,
        })),
        Err(status) => status,
    };
    interp.routines_changed();
    Ok(code(interp, status))
}

/// `SysReorderRexxMacro(name, order)`.
pub(crate) fn reorder(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let name = required_c_string(interp, args, 1)?;
    let option = required_c_string(interp, args, 2)?;
    let position = order("SysReorderRexxMacro", &option)?;
    let status = ask_running(Request::ReorderMacro { name, position }, MACRO_NOT_FOUND);
    interp.routines_changed();
    Ok(code(interp, status))
}

/// `SysDropRexxMacro(name)`.
pub(crate) fn drop_macro(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let name = required_c_string(interp, args, 1)?;
    let status = ask_running(Request::DropMacro { name }, MACRO_NOT_FOUND);
    interp.routines_changed();
    Ok(code(interp, status))
}

/// `SysQueryRexxMacro(name)`: `B`, `A`, or the null string where there is no such macro.
pub(crate) fn query(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let name = required_c_string(interp, args, 1)?;
    let answer: &[u8] = match macro_position(&name) {
        Some(BEFORE) => b"B",
        Some(_) => b"A",
        None => b"",
    };
    Ok(interp.text(answer))
}

/// The search position of the macro `name`, where a running service holds one.
pub(crate) fn macro_position(name: &[u8]) -> Option<u64> {
    let client = rxapi_client::running_client()?;
    match client.call(Request::QueryMacro {
        name: name.to_vec(),
    }) {
        Ok(Reply::MacroPosition {
            status: OK,
            position,
        }) => Some(position),
        _ => None,
    }
}

/// The image of the macro `name`, where a running service holds one.
pub(crate) fn macro_image(name: &[u8]) -> Option<Vec<u8>> {
    let client = rxapi_client::running_client()?;
    match client.call(Request::ResolveMacro {
        name: name.to_vec(),
    }) {
        Ok(Reply::MacroImage {
            status: OK,
            image: Some(image),
        }) => Some(image),
        _ => None,
    }
}

/// `SysClearRexxMacroSpace()`.
pub(crate) fn clear(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 0)?;
    let status = ask_running(Request::ClearMacroSpace, OK);
    interp.routines_changed();
    Ok(code(interp, status))
}

/// `SysSaveRexxMacroSpace(file)`: every macro, in this crate's layout. A failure leaves no
/// file.
pub(crate) fn save(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let file = path_argument(interp, args, 1)?;
    let reply = match rxapi_client::running_client() {
        Some(client) => client.call(Request::SaveMacros { names: None }),
        None => Err(MACRO_FILE_ERROR),
    };
    let status = match reply {
        Ok(Reply::Macros { status: OK, macros }) => write_file(&file, &macro_file::write(&macros)),
        reply => status_of(reply),
    };
    Ok(code(interp, status))
}

/// Writes `bytes` to `file`. A file that cannot be opened for writing is left as it was,
/// with its bytes and its permission; a partial file is removed on a failed write.
pub(crate) fn write_file(file: &[u8], bytes: &[u8]) -> u64 {
    use std::io::Write as _;
    let path = crate::paths::os_path(file);
    let Ok(mut out) = std::fs::File::create(path) else {
        return MACRO_FILE_ERROR;
    };
    match out.write_all(bytes) {
        Ok(()) => OK,
        Err(_) => {
            drop(out);
            if std::fs::metadata(path).is_ok_and(|metadata| metadata.is_file()) {
                let _ = std::fs::remove_file(path);
            }
            MACRO_FILE_ERROR
        }
    }
}

/// `SysLoadRexxMacroSpace(file)`: every macro the file holds, replacing those of the same name.
/// Macros read before a failure are loaded.
pub(crate) fn load(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let file = path_argument(interp, args, 1)?;
    let Ok(bytes) = std::fs::read(crate::paths::os_path(&file)) else {
        return Ok(code(interp, MACRO_FILE_ERROR));
    };
    let (status, macros) = macro_file::read(&bytes);
    let loaded = match macros.is_empty() {
        true => OK,
        false => load_into(rxapi_client::client(), macros),
    };
    interp.routines_changed();
    Ok(code(interp, if status == OK { loaded } else { status }))
}

fn load_into(client: Result<&ApiClient, u64>, macros: Vec<interrexx_rxapi::message::Macro>) -> u64 {
    match client {
        Ok(client) => status_of(client.call(Request::LoadMacros {
            names: None,
            macros,
        })),
        Err(_) => NO_SERVICE,
    }
}
