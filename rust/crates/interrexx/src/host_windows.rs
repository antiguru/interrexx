//! Windows host operations.
#![allow(unsafe_code)]

pub(crate) fn pipe() -> std::io::Result<(std::io::PipeReader, std::io::PipeWriter)> {
    std::io::pipe()
}

pub(crate) fn spawning<T>(spawn: impl FnOnce() -> T) -> T {
    spawn()
}

pub(crate) fn standard_file(number: i32) -> std::io::Result<std::fs::File> {
    use std::os::windows::io::AsHandle as _;
    let owned = match number {
        0 => std::io::stdin().as_handle().try_clone_to_owned(),
        1 => std::io::stdout().as_handle().try_clone_to_owned(),
        2 => std::io::stderr().as_handle().try_clone_to_owned(),
        _ => return Err(std::io::Error::from_raw_os_error(6)), // ERROR_INVALID_HANDLE
    };
    owned.map(std::fs::File::from)
}

pub(crate) fn file_kind(file: &std::fs::File) -> u32 {
    use std::os::windows::io::AsRawHandle as _;
    // SAFETY: the file owns this handle for the duration of the query.
    unsafe { windows_sys::Win32::Storage::FileSystem::GetFileType(file.as_raw_handle()) }
}

pub(crate) fn transient(file: &std::fs::File) -> bool {
    use windows_sys::Win32::Storage::FileSystem::{FILE_TYPE_CHAR, FILE_TYPE_PIPE};
    matches!(file_kind(file), FILE_TYPE_CHAR | FILE_TYPE_PIPE)
}

pub(crate) fn bytes_waiting(pipe: &impl std::os::windows::io::AsRawHandle) -> Option<u32> {
    use windows_sys::Win32::System::Pipes::PeekNamedPipe;
    let mut available = 0;
    // SAFETY: the handle is live and the only non-null out parameter points to u32.
    let result = unsafe {
        PeekNamedPipe(
            pipe.as_raw_handle(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            &mut available,
            std::ptr::null_mut(),
        )
    };
    if result != 0 {
        Some(available)
    } else if std::io::Error::last_os_error().raw_os_error() == Some(109) {
        Some(0) // ERROR_BROKEN_PIPE
    } else {
        None
    }
}

pub(crate) fn terminate(process: &std::os::windows::io::OwnedHandle) {
    use std::os::windows::io::AsRawHandle as _;
    // SAFETY: the handle was cloned from the child this interpreter started.
    unsafe { windows_sys::Win32::System::Threading::TerminateProcess(process.as_raw_handle(), 1) };
}

/// Checks for child exit, waiting briefly only when its pipe has no data.
pub(crate) fn child_ended(process: &std::os::windows::io::OwnedHandle, pipe_empty: bool) -> bool {
    use std::os::windows::io::AsRawHandle as _;
    use windows_sys::Win32::{Foundation::WAIT_OBJECT_0, System::Threading::WaitForSingleObject};
    // SAFETY: the handle is a live process handle with synchronization rights.
    let timeout = if pipe_empty { 10 } else { 0 };
    let status = unsafe { WaitForSingleObject(process.as_raw_handle(), timeout) };
    status == WAIT_OBJECT_0
}

fn file_path(path: &std::path::Path) -> std::io::Result<Vec<u16>> {
    use std::os::windows::ffi::OsStrExt as _;
    let mut wide: Vec<_> = path.as_os_str().encode_wide().collect();
    if wide.contains(&0) {
        return Err(std::io::ErrorKind::InvalidInput.into());
    }
    wide.push(0);
    Ok(wide)
}

pub(crate) fn copy_file(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    use windows_sys::Win32::Storage::FileSystem::CopyFileW;
    let from = file_path(from)?;
    let to = file_path(to)?;
    // SAFETY: both pointers address live NUL-terminated paths.
    if unsafe { CopyFileW(from.as_ptr(), to.as_ptr(), 0) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

pub(crate) fn set_file_encryption(path: &std::path::Path, encrypted: bool) -> std::io::Result<()> {
    use windows_sys::Win32::Storage::FileSystem::{DecryptFileW, EncryptFileW};

    let path = file_path(path)?;
    // SAFETY: path is a NUL-terminated UTF-16 string live across the call.
    let done = unsafe {
        if encrypted {
            EncryptFileW(path.as_ptr())
        } else {
            DecryptFileW(path.as_ptr(), 0)
        }
    };
    if done == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub(crate) fn move_file(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    use windows_sys::Win32::Storage::FileSystem::MoveFileW;
    let from = file_path(from)?;
    let to = file_path(to)?;
    // SAFETY: both pointers address live NUL-terminated paths.
    if unsafe { MoveFileW(from.as_ptr(), to.as_ptr()) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Decodes the filesystem's UTC FILETIME counter in Windows local time.
pub(crate) fn local_file_time(ticks: u64) -> Option<windows_sys::Win32::Foundation::SYSTEMTIME> {
    use windows_sys::Win32::{
        Foundation::{FILETIME, SYSTEMTIME},
        System::Time::{FileTimeToSystemTime, SystemTimeToTzSpecificLocalTimeEx},
    };

    let ticks = ticks.to_le_bytes();
    let raw = FILETIME {
        dwLowDateTime: u32::from_le_bytes(ticks[..4].try_into().unwrap()),
        dwHighDateTime: u32::from_le_bytes(ticks[4..].try_into().unwrap()),
    };
    let mut utc = SYSTEMTIME::default();
    let mut civil = SYSTEMTIME::default();
    // SAFETY: all non-null pointers refer to distinct live structures.
    // The null zone uses the active Windows zone and its date-specific rules.
    let ok = unsafe {
        FileTimeToSystemTime(&raw, &mut utc) != 0
            && SystemTimeToTzSpecificLocalTimeEx(std::ptr::null(), &utc, &mut civil) != 0
    };
    ok.then_some(civil)
}

/// Writes the modification time, interpreting `civil` in Windows local time.
pub(crate) fn set_local_file_time(
    file: &std::fs::File,
    civil: &windows_sys::Win32::Foundation::SYSTEMTIME,
) -> Option<()> {
    use std::os::windows::io::AsRawHandle as _;
    use windows_sys::Win32::{
        Foundation::{FILETIME, SYSTEMTIME},
        Storage::FileSystem::SetFileTime,
        System::Time::{SystemTimeToFileTime, TzSpecificLocalTimeToSystemTimeEx},
    };

    let mut utc = SYSTEMTIME::default();
    let mut raw = FILETIME::default();
    // SAFETY: both conversions use distinct live structures. The file
    // handle stays open and null leaves creation and access unchanged.
    let ok = unsafe {
        TzSpecificLocalTimeToSystemTimeEx(std::ptr::null(), civil, &mut utc) != 0
            && SystemTimeToFileTime(&utc, &mut raw) != 0
            && SetFileTime(
                file.as_raw_handle(),
                std::ptr::null(),
                std::ptr::null(),
                &raw,
            ) != 0
    };
    ok.then_some(())
}

pub(crate) fn console_info()
-> Option<windows_sys::Win32::System::Console::CONSOLE_SCREEN_BUFFER_INFO> {
    use windows_sys::Win32::System::Console::{
        CONSOLE_SCREEN_BUFFER_INFO, GetConsoleScreenBufferInfo, GetStdHandle, STD_OUTPUT_HANDLE,
    };

    let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
    // SAFETY: info is writable and GetConsoleScreenBufferInfo validates the handle.
    let ok = unsafe { GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), &mut info) };
    (ok != 0).then_some(info)
}

pub(crate) fn resize_console_buffer(rows: i16, columns: i16) -> i64 {
    use windows_sys::Win32::{
        Foundation::GetLastError,
        System::Console::{COORD, GetStdHandle, STD_OUTPUT_HANDLE, SetConsoleScreenBufferSize},
    };

    // SAFETY: SetConsoleScreenBufferSize validates the dimensions and handle.
    unsafe {
        let size = COORD {
            X: columns,
            Y: rows,
        };
        if SetConsoleScreenBufferSize(GetStdHandle(STD_OUTPUT_HANDLE), size) != 0 {
            0
        } else {
            i64::from(GetLastError())
        }
    }
}

pub(crate) fn set_console_window([top, left, bottom, right]: [i16; 4]) -> i64 {
    use windows_sys::Win32::{
        Foundation::GetLastError,
        System::Console::{GetStdHandle, SMALL_RECT, STD_OUTPUT_HANDLE, SetConsoleWindowInfo},
    };

    // SAFETY: the rectangle lives through the call, which validates the handle.
    unsafe {
        let rect = SMALL_RECT {
            Top: top,
            Left: left,
            Bottom: bottom,
            Right: right,
        };
        if SetConsoleWindowInfo(GetStdHandle(STD_OUTPUT_HANDLE), 1, &rect) != 0 {
            0
        } else {
            i64::from(GetLastError())
        }
    }
}

/// Clears the console screen buffer and moves the cursor back to its origin.
pub(crate) fn clear_console() -> i64 {
    use windows_sys::Win32::{
        Foundation::GetLastError,
        System::Console::{
            CONSOLE_SCREEN_BUFFER_INFO, COORD, FillConsoleOutputAttribute,
            FillConsoleOutputCharacterW, GetConsoleScreenBufferInfo, GetStdHandle,
            STD_OUTPUT_HANDLE, SetConsoleCursorPosition,
        },
    };

    let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
    // SAFETY: info is writable and the console functions validate the handle.
    unsafe {
        let console = GetStdHandle(STD_OUTPUT_HANDLE);
        if GetConsoleScreenBufferInfo(console, &mut info) == 0 {
            return i64::from(GetLastError());
        }
        let area =
            u32::from(info.dwSize.X.unsigned_abs()) * u32::from(info.dwSize.Y.unsigned_abs());
        let origin = COORD { X: 0, Y: 0 };
        let mut written = 0;
        if FillConsoleOutputCharacterW(console, u16::from(b' '), area, origin, &mut written) == 0
            || FillConsoleOutputAttribute(console, info.wAttributes, area, origin, &mut written)
                == 0
            || SetConsoleCursorPosition(console, origin) == 0
        {
            return i64::from(GetLastError());
        }
    }
    0
}

/// The drive containing the Windows installation.
pub(crate) fn boot_drive() -> Option<[u8; 2]> {
    use std::path::{Component, Path, Prefix};
    use windows_sys::Win32::System::SystemInformation::GetSystemWindowsDirectoryW;

    let directory = system_path(GetSystemWindowsDirectoryW)?;
    match Path::new(&directory).components().next()? {
        Component::Prefix(prefix) => match prefix.kind() {
            Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => Some([drive, b':']),
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn system_directory() -> Option<String> {
    use windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW;

    system_path(GetSystemDirectoryW)
}

pub(crate) fn file_name_spelling(path: &std::path::Path, short: bool) -> Option<String> {
    use windows_sys::Win32::Storage::FileSystem::{GetLongPathNameW, GetShortPathNameW};

    let path = file_path(path).ok()?;
    let convert = if short {
        GetShortPathNameW
    } else {
        GetLongPathNameW
    };
    wide_path_answer(|buffer, capacity| {
        // SAFETY: path is NUL-terminated, and buffer is writable for capacity UTF-16 units.
        unsafe { convert(path.as_ptr(), buffer, capacity) }
    })
}

fn system_path(query: unsafe extern "system" fn(*mut u16, u32) -> u32) -> Option<String> {
    wide_path_answer(|buffer, capacity| {
        // SAFETY: buffer is writable for capacity UTF-16 units.
        unsafe { query(buffer, capacity) }
    })
}

/// These Win32 queries return the required capacity, including the
/// terminator, if the supplied buffer cannot hold the path.
fn wide_path_answer(query: impl Fn(*mut u16, u32) -> u32) -> Option<String> {
    let mut buffer = vec![0; 260];
    loop {
        let capacity = u32::try_from(buffer.len()).ok()?;
        let count = query(buffer.as_mut_ptr(), capacity);
        if count == 0 {
            return None;
        }
        if count < capacity {
            return String::from_utf16(&buffer[..count as usize]).ok();
        }
        buffer.resize((count as usize).checked_add(1)?, 0);
    }
}

/// Returns the volume label and filesystem name. The root includes a trailing backslash.
pub(crate) fn volume_names(root: &std::path::Path) -> Option<(String, String)> {
    use std::ptr::null_mut;
    use windows_sys::Win32::Storage::FileSystem::GetVolumeInformationW;

    let root = file_path(root).ok()?;
    let mut label = [0; 261];
    let mut filesystem = [0; 261];
    // SAFETY: root ends in NUL, and both output buffers have the advertised capacity.
    let result = unsafe {
        GetVolumeInformationW(
            root.as_ptr(),
            label.as_mut_ptr(),
            label.len() as u32,
            null_mut(),
            null_mut(),
            null_mut(),
            filesystem.as_mut_ptr(),
            filesystem.len() as u32,
        )
    };
    if result == 0 {
        return None;
    }
    Some((
        String::from_utf16_lossy(label.split(|unit| *unit == 0).next()?),
        String::from_utf16_lossy(filesystem.split(|unit| *unit == 0).next()?),
    ))
}

/// Returns the free bytes on the volume and its total size.
pub(crate) fn volume_space(root: &std::path::Path) -> Option<(u64, u64)> {
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let root = file_path(root).ok()?;
    let mut free = 0;
    let mut total = 0;
    // SAFETY: root ends in NUL and the non-null outputs are writable u64s.
    let result =
        unsafe { GetDiskFreeSpaceExW(root.as_ptr(), std::ptr::null_mut(), &mut total, &mut free) };
    (result != 0).then_some((free, total))
}

/// Read the running version regardless of the caller's application manifest.
pub(crate) fn windows_version() -> Option<String> {
    use windows_sys::Wdk::System::SystemServices::RtlGetVersion;
    use windows_sys::Win32::System::SystemInformation::OSVERSIONINFOW;

    let mut info = OSVERSIONINFOW {
        dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };
    // SAFETY: info is writable and its size identifies the structure version.
    let status = unsafe { RtlGetVersion(&mut info) };
    (status >= 0).then(|| {
        format!(
            "Windows {}.{}.{}",
            info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber
        )
    })
}

/// The bit for each drive letter currently assigned by Windows.
pub(crate) fn logical_drives() -> Option<u32> {
    use windows_sys::Win32::Storage::FileSystem::GetLogicalDrives;
    // SAFETY: GetLogicalDrives has no parameters or memory to validate.
    let drives = unsafe { GetLogicalDrives() };
    (drives != 0).then_some(drives)
}

pub(crate) fn drive_type(letter: u8) -> u32 {
    use windows_sys::Win32::Storage::FileSystem::GetDriveTypeW;
    let root = [u16::from(letter), u16::from(b':'), u16::from(b'\\'), 0];
    // SAFETY: root is NUL-terminated and lives through the query.
    unsafe { GetDriveTypeW(root.as_ptr()) }
}

/// Enumerates a Windows search path whose final component can contain wildcards.
pub(crate) fn find_files(
    search: &std::path::Path,
) -> Vec<windows_sys::Win32::Storage::FileSystem::WIN32_FIND_DATAW> {
    use windows_sys::Win32::{
        Foundation::INVALID_HANDLE_VALUE,
        Storage::FileSystem::{FindClose, FindFirstFileW, FindNextFileW, WIN32_FIND_DATAW},
    };

    let Ok(search) = file_path(search) else {
        return Vec::new();
    };
    let mut entry = WIN32_FIND_DATAW::default();
    // SAFETY: search is NUL terminated and entry is a live writable structure.
    let handle = unsafe { FindFirstFileW(search.as_ptr(), &mut entry) };
    if handle == INVALID_HANDLE_VALUE {
        return Vec::new();
    }
    let mut entries = vec![entry];
    // SAFETY: the handle came from FindFirstFileW and entry remains writable.
    while unsafe { FindNextFileW(handle, &mut entry) } != 0 {
        entries.push(entry);
    }
    // SAFETY: this is the search handle returned by FindFirstFileW.
    unsafe { FindClose(handle) };
    entries
}

pub(crate) fn set_file_attributes(path: &std::path::Path, attributes: u32) -> bool {
    use windows_sys::Win32::Storage::FileSystem::SetFileAttributesW;

    let Ok(path) = file_path(path) else {
        return false;
    };
    // SAFETY: path is NUL terminated and remains live for the call.
    unsafe { SetFileAttributesW(path.as_ptr(), attributes) != 0 }
}

pub(crate) fn decode_code_page(page: u32, flags: u32, bytes: &[u8]) -> Result<Vec<u16>, u32> {
    use std::ptr::null_mut;
    use windows_sys::Win32::{
        Foundation::{ERROR_INVALID_PARAMETER, GetLastError},
        Globalization::MultiByteToWideChar,
    };

    let length = i32::try_from(bytes.len()).map_err(|_| ERROR_INVALID_PARAMETER)?;
    // SAFETY: bytes contains length readable bytes. A null destination and
    // zero capacity ask the API to compute the required number of code units.
    let needed = unsafe { MultiByteToWideChar(page, flags, bytes.as_ptr(), length, null_mut(), 0) };
    if needed == 0 {
        // SAFETY: reads the calling thread's error for the immediately preceding call.
        return Err(unsafe { GetLastError() });
    }
    let mut wide = vec![0; needed as usize];
    // SAFETY: wide has room for needed units and does not alias bytes.
    let written = unsafe {
        MultiByteToWideChar(
            page,
            flags,
            bytes.as_ptr(),
            length,
            wide.as_mut_ptr(),
            needed,
        )
    };
    if written == 0 {
        // SAFETY: reads the calling thread's error for the immediately preceding call.
        return Err(unsafe { GetLastError() });
    }
    wide.truncate(written as usize);
    Ok(wide)
}

pub(crate) fn encode_code_page(
    page: u32,
    flags: u32,
    wide: &[u16],
    default: Option<[u8; 2]>,
) -> Result<(Vec<u8>, bool), u32> {
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::{
        Foundation::{ERROR_INVALID_PARAMETER, GetLastError},
        Globalization::{CP_UTF7, CP_UTF8, WideCharToMultiByte},
    };

    let length = i32::try_from(wide.len()).map_err(|_| ERROR_INVALID_PARAMETER)?;
    let replacement = default.as_ref().map_or(null(), |bytes| bytes.as_ptr());
    let mut used = 0;
    let used_pointer = if matches!(page, CP_UTF7 | CP_UTF8) {
        null_mut()
    } else {
        &mut used
    };
    // SAFETY: the input has length readable units, the optional replacement
    // has room for the longest replacement character, and a null output with
    // zero capacity asks the API for the required buffer size.
    let needed = unsafe {
        WideCharToMultiByte(
            page,
            flags,
            wide.as_ptr(),
            length,
            null_mut(),
            0,
            replacement,
            used_pointer,
        )
    };
    if needed == 0 {
        // SAFETY: reads the calling thread's error for the immediately preceding call.
        return Err(unsafe { GetLastError() });
    }
    let mut bytes = vec![0; needed as usize];
    // SAFETY: bytes has room for needed bytes; the remaining pointers have
    // the same valid backing memory as in the size query.
    let written = unsafe {
        WideCharToMultiByte(
            page,
            flags,
            wide.as_ptr(),
            length,
            bytes.as_mut_ptr(),
            needed,
            replacement,
            used_pointer,
        )
    };
    if written == 0 {
        // SAFETY: reads the calling thread's error for the immediately preceding call.
        return Err(unsafe { GetLastError() });
    }
    bytes.truncate(written as usize);
    Ok((bytes, used != 0))
}

pub(crate) fn read_profile(
    file: &[u16],
    section: Option<&[u16]>,
    key: Option<&[u16]>,
    fallback: Option<&[u16]>,
) -> Vec<u16> {
    use windows_sys::Win32::System::WindowsProgramming::GetPrivateProfileStringW;

    let string = |value: Option<&[u16]>| value.map_or(std::ptr::null(), <[u16]>::as_ptr);
    let mut buffer = vec![0; 256];
    loop {
        // SAFETY: the callers pass NUL-terminated strings, and the output
        // buffer is writable for the specified number of units.
        let count = unsafe {
            GetPrivateProfileStringW(
                string(section),
                string(key),
                string(fallback),
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                file.as_ptr(),
            )
        } as usize;
        let terminator = if key.is_some() { 1 } else { 2 };
        if count < buffer.len() - terminator || buffer.len() >= 32_768 {
            buffer.truncate(count);
            return buffer;
        }
        buffer.resize(buffer.len() * 2, 0);
    }
}

pub(crate) fn write_profile(
    file: &[u16],
    section: &[u16],
    key: Option<&[u16]>,
    value: Option<&[u16]>,
) -> bool {
    use windows_sys::Win32::System::WindowsProgramming::WritePrivateProfileStringW;

    let string = |value: Option<&[u16]>| value.map_or(std::ptr::null(), <[u16]>::as_ptr);
    // SAFETY: the callers pass live NUL-terminated strings and the optional
    // null pointers request removal of the key or section.
    unsafe {
        WritePrivateProfileStringW(section.as_ptr(), string(key), string(value), file.as_ptr()) != 0
    }
}

pub(crate) fn move_console_cursor(row: i16, column: i16) {
    use windows_sys::Win32::System::Console::{
        COORD, GetStdHandle, STD_OUTPUT_HANDLE, SetConsoleCursorPosition,
    };

    // SAFETY: both APIs validate the arguments and the coordinate is passed by value.
    unsafe {
        SetConsoleCursorPosition(GetStdHandle(STD_OUTPUT_HANDLE), COORD { X: column, Y: row })
    };
}

pub(crate) fn set_console_cursor_visible(visible: bool) -> i64 {
    use windows_sys::Win32::{
        Foundation::GetLastError,
        System::Console::{
            CONSOLE_CURSOR_INFO, GetConsoleCursorInfo, GetStdHandle, STD_OUTPUT_HANDLE,
            SetConsoleCursorInfo,
        },
    };

    let mut cursor = CONSOLE_CURSOR_INFO::default();
    // SAFETY: both APIs validate this handle; cursor is a live writable struct.
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if GetConsoleCursorInfo(handle, &mut cursor) == 0 {
            return i64::from(GetLastError());
        }
        cursor.bVisible = i32::from(visible);
        if SetConsoleCursorInfo(handle, &cursor) == 0 {
            return i64::from(GetLastError());
        }
    }
    0
}

pub(crate) fn read_console_screen(row: i16, column: i16, requested: u32) -> Vec<u8> {
    use windows_sys::Win32::System::Console::{
        COORD, GetStdHandle, ReadConsoleOutputCharacterW, STD_OUTPUT_HANDLE,
    };

    let Some(info) = console_info() else {
        return Vec::new();
    };
    let (width, height) = (info.dwSize.X, info.dwSize.Y);
    if !(0..height).contains(&row) || !(0..width).contains(&column) {
        return Vec::new();
    }
    let available =
        u32::from(width as u16) * u32::from((height - row) as u16) - u32::from(column as u16);
    let length = available.min(requested);
    // Read only actual characters, in chunks so a large reported screen
    // buffer does not cause one huge preallocation.
    let mut wide = Vec::new();
    let mut buffer = [0; 4096];
    while (wide.len() as u32) < length {
        let offset = u32::from(column as u16) + wide.len() as u32;
        let width = u32::from(width as u16);
        let coord = COORD {
            X: (offset % width) as i16,
            Y: row + (offset / width) as i16,
        };
        let count = (length - wide.len() as u32).min(buffer.len() as u32);
        let mut read = 0;
        // SAFETY: buffer covers count writable units, and read is a live u32.
        let ok = unsafe {
            ReadConsoleOutputCharacterW(
                GetStdHandle(STD_OUTPUT_HANDLE),
                buffer.as_mut_ptr(),
                count,
                coord,
                &mut read,
            )
        };
        if ok == 0 || read == 0 {
            break;
        }
        wide.extend_from_slice(&buffer[..read as usize]);
    }
    String::from_utf16_lossy(&wide).into_bytes()
}

pub(crate) fn printers() -> std::io::Result<Vec<[String; 3]>> {
    use std::{mem::MaybeUninit, ptr::null_mut};
    use windows_sys::Win32::{
        Foundation::ERROR_INSUFFICIENT_BUFFER,
        Graphics::Printing::{
            EnumPrintersW, PRINTER_ENUM_CONNECTIONS, PRINTER_ENUM_LOCAL, PRINTER_INFO_2W,
        },
    };

    let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;
    let stride = std::mem::size_of::<PRINTER_INFO_2W>();
    let mut buffer: Vec<MaybeUninit<PRINTER_INFO_2W>> = Vec::new();
    loop {
        let length = (buffer.len() * stride) as u32;
        let mut needed = 0;
        let mut count = 0;
        let data = if length == 0 {
            null_mut()
        } else {
            buffer.as_mut_ptr().cast()
        };
        // SAFETY: the buffer is suitably aligned and has length bytes. Both
        // output sizes refer to separate live u32s.
        let ok =
            unsafe { EnumPrintersW(flags, null_mut(), 2, data, length, &mut needed, &mut count) };
        if ok == 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(ERROR_INSUFFICIENT_BUFFER as i32) || needed <= length {
                return Err(error);
            }
            let units = (needed as usize).div_ceil(stride);
            if units > u32::MAX as usize / stride {
                return Err(std::io::ErrorKind::OutOfMemory.into());
            }
            buffer.resize_with(units, MaybeUninit::uninit);
            continue;
        }
        let buffer_limit = data.addr() + length as usize;
        let entries = buffer
            .iter()
            .take(count as usize)
            .map(|entry| {
                // SAFETY: a successful level-2 result initialized its count
                // of entries at the beginning of the buffer.
                let entry = unsafe { entry.assume_init_ref() };
                [entry.pPrinterName, entry.pDriverName, entry.pPortName].map(|text| {
                    if text.addr() < data.addr()
                        || text.addr() >= buffer_limit
                        || !text.is_aligned()
                    {
                        return String::new();
                    }
                    let units = (buffer_limit - text.addr()) / 2;
                    for offset in 0..units {
                        // SAFETY: the spooler initializes the returned NUL-terminated
                        // string, and each read stays inside the result buffer.
                        if unsafe { *text.add(offset) } == 0 {
                            // SAFETY: all offset preceding code units were read above.
                            let name = unsafe { std::slice::from_raw_parts(text, offset) };
                            return String::from_utf16_lossy(name);
                        }
                    }
                    String::new()
                })
            })
            .collect();
        return Ok(entries);
    }
}

pub(crate) fn default_printer() -> Option<String> {
    use std::ptr::null_mut;
    use windows_sys::Win32::Graphics::Printing::GetDefaultPrinterW;

    let mut needed = 0;
    // SAFETY: null queries the length and needed is writable.
    unsafe { GetDefaultPrinterW(null_mut(), &mut needed) };
    if needed == 0 {
        return None;
    }
    let mut name = vec![0; needed as usize];
    // SAFETY: name covers the requested number of writable UTF-16 units.
    if unsafe { GetDefaultPrinterW(name.as_mut_ptr(), &mut needed) } == 0 {
        return None;
    }
    let end = name.iter().position(|&unit| unit == 0)?;
    Some(String::from_utf16_lossy(&name[..end]))
}

pub(crate) fn set_default_printer(name: &str) -> u32 {
    use windows_sys::Win32::{Foundation::GetLastError, Graphics::Printing::SetDefaultPrinterW};
    let name: Vec<_> = name.encode_utf16().chain([0]).collect();
    // SAFETY: name is a live NUL-terminated string.
    unsafe {
        if SetDefaultPrinterW(name.as_ptr()) == 0 {
            return GetLastError();
        }
    }
    0
}

pub(crate) fn console_key(stopped: &std::sync::atomic::AtomicBool) -> Vec<u8> {
    use std::sync::atomic::Ordering;
    use windows_sys::Win32::{
        Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT},
        System::{
            Console::{
                GetConsoleMode, GetStdHandle, INPUT_RECORD, KEY_EVENT, ReadConsoleInputW,
                STD_INPUT_HANDLE,
            },
            Threading::WaitForSingleObject,
        },
    };

    let mut mode = 0;
    // SAFETY: GetStdHandle supplies a borrowed standard handle; it is validated
    // by GetConsoleMode and the pointer to mode is live and writable.
    let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if unsafe { GetConsoleMode(handle, &mut mode) } == 0 {
        return Vec::new();
    }
    let mut units = Vec::with_capacity(2);
    while !stopped.load(Ordering::Relaxed) {
        // SAFETY: the console handle was validated above and is not owned here.
        match unsafe { WaitForSingleObject(handle, 30) } {
            WAIT_TIMEOUT => continue,
            WAIT_OBJECT_0 => {}
            _ => break,
        }
        let mut event = INPUT_RECORD::default();
        let mut count = 0;
        // SAFETY: both outputs point into distinct live stack locals.
        if unsafe { ReadConsoleInputW(handle, &mut event, 1, &mut count) } == 0 || count == 0 {
            break;
        }
        if u32::from(event.EventType) != KEY_EVENT {
            continue;
        }
        // SAFETY: KEY_EVENT identifies the initialized union member.
        let key = unsafe { event.Event.KeyEvent };
        // SAFETY: this event was read using ReadConsoleInputW.
        let unit = unsafe { key.uChar.UnicodeChar };
        if key.bKeyDown == 0 || unit == 0 {
            continue;
        }
        units.push(unit);
        if units.len() == 2 || !(0xd800..=0xdbff).contains(&unit) {
            return String::from_utf16_lossy(&units).into_bytes();
        }
    }
    Vec::new()
}

pub(crate) fn wait_named_pipe(
    name: &[u16],
    timeout: i64,
    stopped: &std::sync::atomic::AtomicBool,
) -> u32 {
    use std::sync::atomic::Ordering;
    use std::time::Duration;
    use windows_sys::Win32::{
        Foundation::{ERROR_OPERATION_ABORTED, ERROR_SEM_TIMEOUT, GetLastError},
        System::Pipes::WaitNamedPipeW,
    };

    let end = (timeout > 0).then(|| crate::sync::now() + Duration::from_millis(timeout as u64));
    loop {
        if stopped.load(Ordering::Relaxed) {
            return ERROR_OPERATION_ABORTED;
        }
        let slice = match end {
            Some(end) => {
                let remaining = end.saturating_duration_since(crate::sync::now());
                u32::try_from(remaining.as_millis().clamp(1, 100)).unwrap_or(100)
            }
            None if timeout < 0 => 100,
            None => 0, // The named pipe server's default timeout.
        };
        // SAFETY: name is the live NUL-terminated path supplied by RexxUtil.
        let code = unsafe {
            if WaitNamedPipeW(name.as_ptr(), slice) != 0 {
                return 0;
            }
            GetLastError()
        };
        if code != ERROR_SEM_TIMEOUT
            || timeout == 0
            || end.is_some_and(|end| crate::sync::now() >= end)
        {
            return code;
        }
    }
}

thread_local! {
    // The modal loop and timer callback run on the same pool thread.
    static MESSAGE_BOX_STOPPED: std::cell::Cell<*const std::sync::atomic::AtomicBool> =
        const { std::cell::Cell::new(std::ptr::null()) };
}

pub(crate) fn message_box(
    text: &[u16],
    title: &[u16],
    flags: u32,
    stopped: &std::sync::atomic::AtomicBool,
) -> u32 {
    use std::{ptr::null_mut, sync::atomic::Ordering};
    use windows_sys::Win32::UI::WindowsAndMessaging::{IDCANCEL, KillTimer, MessageBoxW, SetTimer};

    if stopped.load(Ordering::Relaxed) {
        return IDCANCEL as u32;
    }
    // SAFETY: timer callbacks run on this same thread during the modal loop.
    let timer = unsafe { SetTimer(null_mut(), 0, 25, Some(cancel_modal_dialog)) };
    if timer == 0 {
        return 0;
    }
    MESSAGE_BOX_STOPPED.set(std::ptr::from_ref(stopped));
    // SAFETY: the strings are live and NUL terminated; null selects no owner.
    let result = unsafe { MessageBoxW(null_mut(), text.as_ptr(), title.as_ptr(), flags) };
    MESSAGE_BOX_STOPPED.set(std::ptr::null());
    // SAFETY: timer is the identifier SetTimer returned on this thread.
    unsafe { KillTimer(null_mut(), timer) };
    result as u32
}

unsafe extern "system" fn cancel_modal_dialog(
    _: windows_sys::Win32::Foundation::HWND,
    _: u32,
    _: usize,
    _: u32,
) {
    use std::sync::atomic::Ordering;
    use windows_sys::Win32::UI::WindowsAndMessaging::EnumThreadWindows;

    // SAFETY: message_box clears the thread-local pointer before the borrow
    // ends. A timer message left in the queue after KillTimer sees null.
    if unsafe { MESSAGE_BOX_STOPPED.get().as_ref() }
        .is_some_and(|stopped| stopped.load(Ordering::Relaxed))
    {
        // SAFETY: only this worker's own thread windows are enumerated.
        unsafe { EnumThreadWindows(thread_id(), Some(close_modal_dialog), 0) };
    }
}

unsafe extern "system" fn close_modal_dialog(
    window: windows_sys::Win32::Foundation::HWND,
    _: isize,
) -> i32 {
    use windows_sys::Win32::UI::WindowsAndMessaging::{EndDialog, GetClassNameW, IDCANCEL};
    let mut class = [0; 16];
    // SAFETY: Windows supplies the current thread's window. The class
    // buffer is live and writable for class.len() UTF-16 units.
    let length = unsafe { GetClassNameW(window, class.as_mut_ptr(), class.len() as i32) };
    let dialog_class = [35, 51, 50, 55, 55, 48]; // #32770
    if length == dialog_class.len() as i32 && class[..dialog_class.len()] == dialog_class {
        // SAFETY: this thread owns the dialog. EndDialog exits its modal
        // loop even when the selected button set has no Cancel button.
        unsafe { EndDialog(window, IDCANCEL as isize) };
        return 0;
    }
    1
}

pub(crate) fn switch_window(title: &[u16]) -> u32 {
    use std::ptr::null;
    use windows_sys::Win32::{
        Foundation::{ERROR_ACCESS_DENIED, ERROR_INVALID_WINDOW_HANDLE},
        UI::WindowsAndMessaging::{FindWindowW, FlashWindow, SetForegroundWindow},
    };

    // SAFETY: title is live and NUL terminated, and Windows validates window handles.
    unsafe {
        let window = FindWindowW(null(), title.as_ptr());
        if window.is_null() {
            return ERROR_INVALID_WINDOW_HANDLE;
        }
        if SetForegroundWindow(window) == 0 {
            FlashWindow(window, 1);
            return ERROR_ACCESS_DENIED;
        }
    }
    0
}

pub(crate) fn thread_id() -> u32 {
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    // SAFETY: this query has no arguments.
    unsafe { GetCurrentThreadId() }
}

pub(crate) fn process_priority() -> u32 {
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetPriorityClass};
    // SAFETY: the pseudo handle refers to the process making the query.
    unsafe { GetPriorityClass(GetCurrentProcess()) }
}

pub(crate) fn thread_priority() -> i32 {
    use windows_sys::Win32::System::Threading::{GetCurrentThread, GetThreadPriority};
    // SAFETY: the pseudo handle refers to the current thread.
    unsafe { GetThreadPriority(GetCurrentThread()) }
}

pub(crate) fn set_priority(class: u32, delta: i32) -> u32 {
    use windows_sys::Win32::{
        Foundation::GetLastError,
        System::Threading::{
            GetCurrentProcess, GetCurrentThread, GetPriorityClass, SetPriorityClass,
            SetThreadPriority,
        },
    };
    // SAFETY: pseudo handles always refer to the caller; the APIs validate
    // the numeric priorities. Restore the process class when the thread
    // change fails and return the original error.
    unsafe {
        let process = GetCurrentProcess();
        let previous = GetPriorityClass(process);
        if SetPriorityClass(process, class) == 0 {
            return GetLastError();
        }
        if SetThreadPriority(GetCurrentThread(), delta) == 0 {
            let status = GetLastError();
            if previous != 0 {
                SetPriorityClass(process, previous);
            }
            return status;
        }
    }
    0
}

pub(crate) fn execution_times(thread: bool) -> Option<[u64; 3]> {
    use windows_sys::Win32::{
        Foundation::FILETIME,
        System::Threading::{GetCurrentProcess, GetCurrentThread, GetProcessTimes, GetThreadTimes},
    };

    let mut created = FILETIME::default();
    let mut exited = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    // SAFETY: all output pointers refer to distinct FILETIMEs.
    let ok = unsafe {
        if thread {
            GetThreadTimes(
                GetCurrentThread(),
                &mut created,
                &mut exited,
                &mut kernel,
                &mut user,
            )
        } else {
            GetProcessTimes(
                GetCurrentProcess(),
                &mut created,
                &mut exited,
                &mut kernel,
                &mut user,
            )
        }
    };
    (ok != 0).then(|| {
        [created, kernel, user]
            .map(|time| (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime))
    })
}

pub(crate) fn windows_directory() -> Option<String> {
    use windows_sys::Win32::System::SystemInformation::GetWindowsDirectoryW;
    // SAFETY: wide_path_answer provides a buffer that can hold length units.
    wide_path_answer(|buffer, length| unsafe { GetWindowsDirectoryW(buffer, length) })
}

pub(crate) fn launch_windows_program(
    image: &std::path::Path,
    command: &str,
    cwd: &std::path::Path,
    show: u16,
    environment: &[(Vec<u8>, Vec<u8>)],
) -> Result<u32, u32> {
    use std::ptr::null;
    use windows_sys::Win32::{
        Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, GetLastError},
        System::Threading::{
            CREATE_UNICODE_ENVIRONMENT, CreateProcessW, PROCESS_INFORMATION, STARTF_USESHOWWINDOW,
            STARTUPINFOW,
        },
    };

    let image = file_path(image).map_err(|_| ERROR_INVALID_PARAMETER)?;
    let cwd = file_path(cwd).map_err(|_| ERROR_INVALID_PARAMETER)?;
    let mut command: Vec<_> = command.encode_utf16().chain([0]).collect();
    let mut entries: Vec<_> = environment
        .iter()
        .filter_map(|(name, value)| {
            let name = std::str::from_utf8(name).ok()?;
            let value = std::str::from_utf8(value).ok()?;
            (!name.contains('\0') && !value.contains('\0')).then_some((name, value))
        })
        .collect();
    entries.sort_by(|(left, _), (right, _)| {
        left.to_ascii_uppercase().cmp(&right.to_ascii_uppercase())
    });
    let mut block = Vec::new();
    for (name, value) in entries {
        block.extend(
            name.encode_utf16()
                .chain([u16::from(b'=')])
                .chain(value.encode_utf16())
                .chain([0]),
        );
    }
    if block.is_empty() {
        block.push(0);
    }
    block.push(0);
    let startup = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        dwFlags: STARTF_USESHOWWINDOW,
        wShowWindow: show,
        ..Default::default()
    };
    let mut child = PROCESS_INFORMATION::default();
    // SAFETY: the strings and environment block are live and terminated as
    // required; command is writable and startup and child are distinct.
    unsafe {
        if CreateProcessW(
            image.as_ptr(),
            command.as_mut_ptr(),
            null(),
            null(),
            0,
            CREATE_UNICODE_ENVIRONMENT,
            block.as_ptr().cast(),
            cwd.as_ptr(),
            &startup,
            &mut child,
        ) == 0
        {
            return Err(GetLastError());
        }
        CloseHandle(child.hThread);
        CloseHandle(child.hProcess);
    }
    Ok(child.dwProcessId)
}

pub(crate) fn request_shutdown(
    computer: Option<&[u16]>,
    message: &[u16],
    timeout: u32,
    force: bool,
    reboot: bool,
) -> u32 {
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::{
        Foundation::{ERROR_SUCCESS, GetLastError, LUID},
        Security::{
            AdjustTokenPrivileges, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW,
            SE_PRIVILEGE_ENABLED, SE_REMOTE_SHUTDOWN_NAME, SE_SHUTDOWN_NAME,
            TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
        },
        System::{
            Shutdown::InitiateSystemShutdownW,
            Threading::{GetCurrentProcess, OpenProcessToken},
        },
    };

    // Keep changes to the process privilege serialized, and restore the
    // prior token state whether the requested shutdown succeeds or fails.
    static SHUTDOWN: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = SHUTDOWN.lock().expect("Windows shutdown privilege lock");
    let mut raw = null_mut();
    let mut identifier = LUID::default();
    let privilege = if computer.is_some() {
        SE_REMOTE_SHUTDOWN_NAME
    } else {
        SE_SHUTDOWN_NAME
    };
    // SAFETY: identifier and raw each point to live writable storage.
    let token = unsafe {
        if LookupPrivilegeValueW(null(), privilege, &mut identifier) == 0
            || OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                &mut raw,
            ) == 0
        {
            return GetLastError();
        }
        OwnedHandle::from_raw_handle(raw)
    };
    let requested = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: identifier,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };
    let mut previous = TOKEN_PRIVILEGES::default();
    let mut previous_len = 0;
    // SAFETY: token is owned for these calls; buffers have room for the
    // single privilege requested, and the inputs are NUL-terminated.
    unsafe {
        let ok = AdjustTokenPrivileges(
            token.as_raw_handle(),
            0,
            &requested,
            std::mem::size_of_val(&previous) as u32,
            &mut previous,
            &mut previous_len,
        );
        let error = GetLastError();
        if ok == 0 || error != ERROR_SUCCESS {
            return error;
        }
        let result = InitiateSystemShutdownW(
            computer.map_or(null(), <[u16]>::as_ptr),
            message.as_ptr(),
            timeout,
            i32::from(force),
            i32::from(reboot),
        );
        let status = if result == 0 {
            GetLastError()
        } else {
            ERROR_SUCCESS
        };
        if previous.PrivilegeCount != 0 {
            AdjustTokenPrivileges(
                token.as_raw_handle(),
                0,
                &previous,
                0,
                null_mut(),
                null_mut(),
            );
        }
        status
    }
}
