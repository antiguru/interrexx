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
