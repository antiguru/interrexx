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
        Storage::FileSystem::FileTimeToLocalFileTime,
        System::Time::FileTimeToSystemTime,
    };

    let ticks = ticks.to_le_bytes();
    let utc = FILETIME {
        dwLowDateTime: u32::from_le_bytes(ticks[..4].try_into().unwrap()),
        dwHighDateTime: u32::from_le_bytes(ticks[4..].try_into().unwrap()),
    };
    let mut local = FILETIME::default();
    let mut civil = SYSTEMTIME::default();
    // SAFETY: all pointers refer to live structures of the required type.
    if unsafe { FileTimeToLocalFileTime(&utc, &mut local) } == 0
        || unsafe { FileTimeToSystemTime(&local, &mut civil) } == 0
    {
        return None;
    }
    Some(civil)
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

/// These Win32 queries return the required capacity, including the
/// terminator, if the supplied buffer cannot hold the path.
fn system_path(query: unsafe extern "system" fn(*mut u16, u32) -> u32) -> Option<String> {
    let mut buffer = vec![0; 260];
    loop {
        let capacity = u32::try_from(buffer.len()).ok()?;
        // SAFETY: the buffer is writable for exactly capacity UTF-16 units.
        let count = unsafe { query(buffer.as_mut_ptr(), capacity) };
        if count == 0 {
            return None;
        }
        if count < capacity {
            return String::from_utf16(&buffer[..count as usize]).ok();
        }
        buffer.resize((count as usize).checked_add(1)?, 0);
    }
}
