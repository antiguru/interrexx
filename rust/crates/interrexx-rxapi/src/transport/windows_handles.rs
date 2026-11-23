//! Account identity, access control and cancellation for local Windows pipes.
#![allow(unsafe_code)]

use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::ptr::{null, null_mut};
use windows_sys::Win32::{
    Foundation::{HANDLE, LocalFree},
    Security::{
        Authorization::ConvertSidToStringSidW, GetTokenInformation, PSID, TOKEN_QUERY, TOKEN_USER,
        TokenUser,
    },
    System::{
        IO::CancelIoEx,
        Pipes::{DisconnectNamedPipe, GetNamedPipeServerProcessId, PeekNamedPipe},
        Threading::{
            GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
        },
    },
};

pub(super) fn account() -> io::Result<String> {
    // SAFETY: the current-process pseudo-handle is valid for this call.
    sid(unsafe { GetCurrentProcess() })
}

pub(super) fn server_account(pipe: &impl AsRawHandle) -> io::Result<String> {
    let mut pid = 0;
    // SAFETY: the borrowed pipe and output pointer are live.
    if unsafe { GetNamedPipeServerProcessId(pipe.as_raw_handle(), &mut pid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: Windows validates the ID and returns a new handle on success.
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: transfer the successful OpenProcess result exactly once.
    let process = unsafe { OwnedHandle::from_raw_handle(process) };
    sid(process.as_raw_handle())
}

fn sid(process: HANDLE) -> io::Result<String> {
    let mut token = null_mut();
    // SAFETY: process is live and the result slot is writable.
    if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the returned token is uniquely owned.
    let token = unsafe { OwnedHandle::from_raw_handle(token) };
    let mut size = 0;
    // SAFETY: a null zero-length buffer asks for the size.
    unsafe { GetTokenInformation(token.as_raw_handle(), TokenUser, null_mut(), 0, &mut size) };
    if size == 0 {
        return Err(io::Error::last_os_error());
    }
    let mut data = vec![0usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
    // SAFETY: buffer is aligned and large enough for the reported size.
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            data.as_mut_ptr().cast(),
            size,
            &mut size,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the successful query initialized a TOKEN_USER and its SID stays in data.
    sid_string(unsafe { (*data.as_ptr().cast::<TOKEN_USER>()).User.Sid })
}

fn sid_string(sid: PSID) -> io::Result<String> {
    let mut text = null_mut();
    // SAFETY: the caller supplied a live SID and text is writable.
    if unsafe { ConvertSidToStringSidW(sid, &mut text) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the OS returned an allocated NUL-terminated string.
    let result = unsafe { widestring::U16CStr::from_ptr_str(text) }.to_string_lossy();
    // SAFETY: ConvertSidToStringSidW requires LocalFree.
    unsafe { LocalFree(text.cast()) };
    Ok(result)
}

pub(super) fn readable(pipe: &impl AsRawHandle) -> io::Result<u32> {
    let mut bytes = 0;
    // SAFETY: the only non-null output points at bytes.
    if unsafe {
        PeekNamedPipe(
            pipe.as_raw_handle(),
            null_mut(),
            0,
            null_mut(),
            &mut bytes,
            null_mut(),
        )
    } != 0
    {
        Ok(bytes)
    } else {
        Err(io::Error::last_os_error())
    }
}

pub(super) fn cancel(pipe: &impl AsRawHandle) {
    // SAFETY: the live handle is local and no OVERLAPPED pointer is supplied.
    unsafe { CancelIoEx(pipe.as_raw_handle(), null()) };
}

pub(super) fn disconnect(pipe: &impl AsRawHandle) {
    // SAFETY: caller holds the server end of the live pipe.
    unsafe { DisconnectNamedPipe(pipe.as_raw_handle()) };
}
