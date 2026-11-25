//! Windows directory enumeration and output for `SysFileTree`.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt as _;
use std::path::{Path, PathBuf};

use windows_sys::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_ARCHIVE, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_NORMAL,
    FILE_ATTRIBUTE_NOT_CONTENT_INDEXED, FILE_ATTRIBUTE_OFFLINE, FILE_ATTRIBUTE_READONLY,
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_SYSTEM, FILE_ATTRIBUTE_TEMPORARY,
    WIN32_FIND_DATAW,
};

use super::TreeOptions;
use crate::builtin::platform::qualify_name;
use crate::paths::os_path;
use crate::{Interp, host};

const ATTRIBUTES: [(u32, u8); 5] = [
    (FILE_ATTRIBUTE_ARCHIVE, b'A'),
    (FILE_ATTRIBUTE_DIRECTORY, b'D'),
    (FILE_ATTRIBUTE_HIDDEN, b'H'),
    (FILE_ATTRIBUTE_READONLY, b'R'),
    (FILE_ATTRIBUTE_SYSTEM, b'S'),
];

pub(super) fn find(
    interp: &Interp,
    spec: &[u8],
    options: TreeOptions,
    target_mask: &[u8],
    new_mask: &[u8],
) -> Vec<Vec<u8>> {
    let mut spec = spec.to_vec();
    if spec.ends_with(b"/") || spec.ends_with(b"\\") || spec.ends_with(b":") {
        spec.push(b'*');
    } else if matches!(
        spec.rsplit(|&byte| matches!(byte, b'/' | b'\\')).next(),
        Some(b"." | b"..")
    ) {
        spec.extend_from_slice(b"\\*");
    }
    let qualified = qualify_name(interp, &spec);
    let path = os_path(&qualified);
    let (Some(directory), Some(pattern)) = (path.parent(), path.file_name()) else {
        return Vec::new();
    };
    if !directory.is_absolute() {
        return Vec::new();
    }

    let mut pending = vec![directory.to_path_buf()];
    let mut lines = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in host::find_files(&directory.join(pattern)) {
            let is_directory = entry.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0;
            if (is_directory && !options.directories) || (!is_directory && !options.files) {
                continue;
            }
            if !matches_mask(entry.dwFileAttributes, target_mask) {
                continue;
            }
            let Some(path) = entry_path(&directory, &entry) else {
                continue;
            };
            if options.names_only {
                lines.push(path.to_string_lossy().into_owned().into_bytes());
                continue;
            }
            let attributes = apply_mask(&path, entry.dwFileAttributes, new_mask);
            lines.push(entry_line(options, &path, &entry, attributes));
        }
        if options.recurse {
            let children = host::find_files(&directory.join("*"));
            pending.extend(children.iter().rev().filter_map(|child| {
                let flags = child.dwFileAttributes;
                // A junction or symlink can lead back to this directory.
                (flags & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT)
                    == FILE_ATTRIBUTE_DIRECTORY)
                    .then(|| entry_path(&directory, child))
                    .flatten()
            }));
        }
    }
    lines
}

fn entry_path(directory: &Path, entry: &WIN32_FIND_DATAW) -> Option<PathBuf> {
    let end = entry.cFileName.iter().position(|&unit| unit == 0)?;
    let name = &entry.cFileName[..end];
    if name == [u16::from(b'.')] || name == [u16::from(b'.'), u16::from(b'.')] {
        return None;
    }
    Some(directory.join(OsString::from_wide(name)))
}

fn matches_mask(attributes: u32, mask: &[u8]) -> bool {
    ATTRIBUTES
        .iter()
        .zip(mask)
        .all(|(&(flag, _), &wanted)| match wanted {
            b'+' => attributes & flag != 0,
            b'-' => attributes & flag == 0,
            _ => true,
        })
}

fn apply_mask(path: &Path, original: u32, mask: &[u8]) -> u32 {
    let mut updated = original;
    for (&(flag, _), &change) in ATTRIBUTES.iter().zip(mask) {
        if flag == FILE_ATTRIBUTE_DIRECTORY {
            continue;
        }
        match change {
            b'+' => updated |= flag,
            b'-' => updated &= !flag,
            _ => {}
        }
    }
    if updated == original {
        return original;
    }
    // SetFileAttributesW accepts this subset. Other properties belong to the
    // filesystem or have their own APIs and are preserved by this operation.
    let writable = updated
        & (FILE_ATTRIBUTE_ARCHIVE
            | FILE_ATTRIBUTE_HIDDEN
            | FILE_ATTRIBUTE_NOT_CONTENT_INDEXED
            | FILE_ATTRIBUTE_OFFLINE
            | FILE_ATTRIBUTE_READONLY
            | FILE_ATTRIBUTE_SYSTEM
            | FILE_ATTRIBUTE_TEMPORARY);
    let writable = if writable == 0 {
        FILE_ATTRIBUTE_NORMAL
    } else {
        writable
    };
    if host::set_file_attributes(path, writable) {
        updated
    } else {
        original
    }
}

fn entry_line(
    options: TreeOptions,
    path: &Path,
    entry: &WIN32_FIND_DATAW,
    attributes: u32,
) -> Vec<u8> {
    let raw_time = entry.ftLastWriteTime;
    let time = host::local_file_time(
        (u64::from(raw_time.dwHighDateTime) << 32) | u64::from(raw_time.dwLowDateTime),
    )
    .unwrap_or_default();
    let (year, month, day) = (time.wYear, time.wMonth, time.wDay);
    let (hour, minute, second) = (time.wHour, time.wMinute, time.wSecond);
    let short_year = year % 100;
    let date = if options.long_time {
        format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
    } else if options.editable_time {
        format!("{short_year:02}/{month:02}/{day:02}/{hour:02}/{minute:02}")
    } else {
        let meridiem = if hour < 12 { 'a' } else { 'p' };
        let hour = (hour + 11) % 12 + 1;
        format!("{month:2}/{day:02}/{short_year:02}  {hour:2}:{minute:02}{meridiem}")
    };
    let size = (u64::from(entry.nFileSizeHigh) << 32) | u64::from(entry.nFileSizeLow);
    let size = if options.long_size {
        format!("{size:20}")
    } else {
        format!("{:10}", size.min(9_999_999_999))
    };
    let flags: String = ATTRIBUTES
        .iter()
        .map(|&(flag, letter)| {
            if attributes & flag != 0 {
                char::from(letter)
            } else {
                '-'
            }
        })
        .collect();
    format!("{date}  {size}  {flags}  {}", path.display()).into_bytes()
}
