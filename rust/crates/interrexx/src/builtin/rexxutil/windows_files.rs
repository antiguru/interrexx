//! File routines backed by the Windows filesystem APIs.

use super::{answer, c_string, check_arity, optional_text, path_argument, scan_triple};
use crate::error::Raised;
use crate::{Failure, Interp};
use interrexx_core::ObjRef;
use std::io;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt as _};
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

/// Answers Windows creation, access or write time in local civil time.
pub(crate) fn get_file_date_time(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let path = path_argument(interp, args, 1)?;
    let selector = optional_text(interp, args, 2)?;
    let selector = selector.as_deref().map_or(b"W".as_slice(), c_string);
    let time = match selector.first().map(u8::to_ascii_uppercase) {
        Some(b'A') => std::fs::Metadata::last_access_time,
        Some(b'C') => std::fs::Metadata::creation_time,
        Some(b'W') => std::fs::Metadata::last_write_time,
        _ => {
            return Err(Raised::invalid_routine_option(
                "SysGetFileDateTime",
                "time selector",
                "'A', 'C', or 'W'",
                selector,
            )
            .into());
        }
    };
    let Ok(metadata) = std::fs::symlink_metadata(crate::paths::os_path(&path)) else {
        return Ok(answer(interp, -1));
    };
    let Some(civil) = crate::host::local_file_time(time(&metadata)) else {
        return Ok(answer(interp, -1));
    };
    Ok(interp.text_built(
        format!(
            "{:4}-{:02}-{:02} {:02}:{:02}:{:02}",
            civil.wYear, civil.wMonth, civil.wDay, civil.wHour, civil.wMinute, civil.wSecond
        )
        .into_bytes(),
    ))
}

/// Updates the last-write time of an existing Windows file or directory.
pub(crate) fn set_file_date_time(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES,
        FILE_WRITE_ATTRIBUTES,
    };

    check_arity(args, 3)?;
    let path = path_argument(interp, args, 1)?;
    let date = optional_text(interp, args, 2)?;
    let time = optional_text(interp, args, 3)?;
    let Ok(file) = std::fs::OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(crate::paths::os_path(&path))
    else {
        return Ok(answer(interp, -1));
    };

    let updated = if date.is_none() && time.is_none() {
        file.set_modified(interp.wall_now()).is_ok()
    } else {
        edit_file_date_time(&file, date.as_deref(), time.as_deref()).is_some()
    };
    Ok(answer(interp, if updated { 0 } else { -1 }))
}

fn edit_file_date_time(
    file: &std::fs::File,
    date: Option<&[u8]>,
    time: Option<&[u8]>,
) -> Option<()> {
    let mut civil = crate::host::local_file_time(file.metadata().ok()?.last_write_time())?;
    if let Some(date) = date {
        let [year, month, day] = scan_triple(c_string(date), b'-', [4, 2, 2])?;
        if year < 1800 {
            return None;
        }
        civil.wYear = u16::try_from(year).ok()?;
        civil.wMonth = u16::try_from(month).ok()?;
        civil.wDay = u16::try_from(day).ok()?;
    }
    if let Some(time) = time {
        let [hour, minute, second] = scan_triple(c_string(time), b':', [2, 2, 2])?;
        civil.wHour = u16::try_from(hour).ok()?;
        civil.wMinute = u16::try_from(minute).ok()?;
        civil.wSecond = u16::try_from(second).ok()?;
        civil.wMilliseconds = 0;
    }
    crate::host::set_local_file_time(file, &civil)
}

pub(crate) fn boot_drive(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 0)?;
    let drive = crate::host::boot_drive();
    Ok(interp.text(drive.as_ref().map_or(b"", |drive| drive.as_slice())))
}

pub(crate) fn system_directory(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 0)?;
    Ok(interp.text_built(
        crate::host::system_directory()
            .unwrap_or_default()
            .into_bytes(),
    ))
}

pub(crate) fn drive_info(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    volume_query(interp, args, |drive, root| {
        let (label, _) = crate::host::volume_names(root)?;
        let (free, total) = crate::host::volume_space(root)?;
        Some(format!("{drive} {free} {total} {label}"))
    })
}

pub(crate) fn file_system_type(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    volume_query(interp, args, |_, root| {
        crate::host::volume_names(root).map(|(_, filesystem)| filesystem)
    })
}

pub(crate) fn volume_label(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    volume_query(interp, args, |_, root| {
        crate::host::volume_names(root).map(|(label, _)| label)
    })
}

fn volume_query(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    query: impl FnOnce(&str, &Path) -> Option<String>,
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let argument = optional_text(interp, args, 1)?;
    let drive = argument.as_deref().map_or(interp.cwd_bytes(), c_string);
    let result = volume_identifier(drive).and_then(|drive| {
        let root = format!("{drive}\\");
        query(&drive, Path::new(&root))
    });
    Ok(interp.text_built(result.unwrap_or_default().into_bytes()))
}

fn volume_identifier(drive: &[u8]) -> Option<String> {
    use std::path::{Component, Prefix};

    let path = std::str::from_utf8(drive).ok()?;
    if let [letter] = drive
        && letter.is_ascii_alphabetic()
    {
        return Some(format!("{}:", char::from(letter.to_ascii_uppercase())));
    }
    let Component::Prefix(prefix) = Path::new(path).components().next()? else {
        return None;
    };
    match prefix.kind() {
        Prefix::Disk(letter) => Some(format!("{}:", char::from(letter))),
        Prefix::VerbatimDisk(_) | Prefix::UNC(_, _) | Prefix::VerbatimUNC(_, _) => {
            prefix.as_os_str().to_str().map(str::to_owned)
        }
        _ => None,
    }
}
