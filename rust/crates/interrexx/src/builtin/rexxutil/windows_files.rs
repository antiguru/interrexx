//! File routines backed by the Windows filesystem APIs.

use super::{answer, check_arity, path_argument};
use crate::{Failure, Interp};
use interrexx_core::ObjRef;
use std::io;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use windows_sys::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
};

fn one_path(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    action: impl FnOnce(&Path) -> io::Result<()>,
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let path = path_argument(interp, args, 1)?;
    let status = action(crate::paths::os_path(&path));
    Ok(answer(interp, result_code(status)))
}

fn two_paths(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    action: impl FnOnce(&Path, &Path) -> io::Result<()>,
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let from = path_argument(interp, args, 1)?;
    let to = path_argument(interp, args, 2)?;
    let status = action(crate::paths::os_path(&from), crate::paths::os_path(&to));
    Ok(answer(interp, result_code(status)))
}

fn result_code(result: io::Result<()>) -> i64 {
    use windows_sys::Win32::Foundation::ERROR_INVALID_NAME;
    result.err().map_or(0, |error| {
        error
            .raw_os_error()
            .map_or(i64::from(ERROR_INVALID_NAME), i64::from)
    })
}

pub(crate) fn mk_dir(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    one_path(interp, args, |path| std::fs::create_dir(path))
}

pub(crate) fn rm_dir(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    one_path(interp, args, |path| std::fs::remove_dir(path))
}

pub(crate) fn file_delete(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    one_path(interp, args, |path| std::fs::remove_file(path))
}

pub(crate) fn file_copy(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    two_paths(interp, args, crate::host::copy_file)
}

pub(crate) fn file_move(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    two_paths(interp, args, crate::host::move_file)
}

pub(crate) fn has_attribute<const ATTRIBUTE: u32>(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let path = path_argument(interp, args, 1)?;
    let has = std::fs::symlink_metadata(crate::paths::os_path(&path))
        .is_ok_and(|metadata| metadata.file_attributes() & ATTRIBUTE != 0);
    Ok(answer(interp, i64::from(has)))
}

pub(crate) fn is_directory(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    has_attribute::<FILE_ATTRIBUTE_DIRECTORY>(interp, name, args)
}

pub(crate) fn is_link(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    has_attribute::<FILE_ATTRIBUTE_REPARSE_POINT>(interp, name, args)
}
