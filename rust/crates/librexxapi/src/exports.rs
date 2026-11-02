//! The `librexxapi` C entry points: shims over `rexx_exec::capi`. The `rexx` executable compiles
//! this file too, so an extension it loads reaches its interpreter's state.
// D-U1 as extended 2026-11-02: one export module per C library.
#![allow(unsafe_code)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_void};

use rexx_api::layout::{
    CONSTRXSTRING, REXXDATETIME, REXXPFN, RXSTRING, RexxQueueTime, RexxReturnCode,
};

/// `RXAPI_NORXAPI`.
const NOT_AVAILABLE: RexxReturnCode = 1004;

/// `RexxRegisterSubcomDll`.
///
/// # Safety
/// Every pointer is null or what `api/rexx.h` declares.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterSubcomDll(
    _name: *const c_char,
    _library: *const c_char,
    _procedure: *const c_char,
    _user_area: *const c_char,
    _drop_authority: usize,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxRegisterSubcomExe`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterSubcomExe(
    _name: *const c_char,
    _entry: REXXPFN,
    _user_area: *const c_char,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxQuerySubcom`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQuerySubcom(
    _name: *const c_char,
    _library: *const c_char,
    _flags: *mut u16,
    _user_word: *mut c_char,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxDeregisterSubcom`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDeregisterSubcom(
    _name: *const c_char,
    _library: *const c_char,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxLoadSubcom`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxLoadSubcom(
    _name: *const c_char,
    _library: *const c_char,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxRegisterExitDll`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterExitDll(
    _name: *const c_char,
    _library: *const c_char,
    _procedure: *const c_char,
    _user_area: *const c_char,
    _drop_authority: usize,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxRegisterExitExe`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterExitExe(
    _name: *const c_char,
    _entry: REXXPFN,
    _user_area: *const c_char,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxDeregisterExit`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDeregisterExit(
    _name: *const c_char,
    _library: *const c_char,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxQueryExit`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueryExit(
    _name: *const c_char,
    _library: *const c_char,
    _flags: *mut u16,
    _user_word: *mut c_char,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxRegisterFunctionDll`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterFunctionDll(
    _name: *const c_char,
    _library: *const c_char,
    _procedure: *const c_char,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxRegisterFunctionExe`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterFunctionExe(
    _name: *const c_char,
    _entry: REXXPFN,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxDeregisterFunction`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDeregisterFunction(_name: *const c_char) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxQueryFunction`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueryFunction(_name: *const c_char) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxAddMacro`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxAddMacro(
    _name: *const c_char,
    _file: *const c_char,
    _position: usize,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxDropMacro`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDropMacro(_name: *const c_char) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxSaveMacroSpace`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxSaveMacroSpace(
    _count: usize,
    _names: *const *const c_char,
    _file: *const c_char,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxLoadMacroSpace`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxLoadMacroSpace(
    _count: usize,
    _names: *const *const c_char,
    _file: *const c_char,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxQueryMacro`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueryMacro(
    _name: *const c_char,
    _position: *mut u16,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxReorderMacro`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxReorderMacro(
    _name: *const c_char,
    _position: usize,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxClearMacroSpace`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxClearMacroSpace() -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxCreateQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxCreateQueue(
    _name: *mut c_char,
    _size: usize,
    _requested: *const c_char,
    _duplicate: *mut usize,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxOpenQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxOpenQueue(
    _name: *const c_char,
    _duplicate: *mut usize,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxQueueExists`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueueExists(_name: *const c_char) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxDeleteQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDeleteQueue(_name: *const c_char) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxQueryQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueryQueue(
    _name: *const c_char,
    _count: *mut usize,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxAddQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxAddQueue(
    _name: *const c_char,
    _data: *const CONSTRXSTRING,
    _flag: usize,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxPullFromQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxPullFromQueue(
    _name: *const c_char,
    _data: *mut RXSTRING,
    _time: *mut RexxQueueTime,
    _wait: usize,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxPullQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxPullQueue(
    _name: *const c_char,
    _data: *mut RXSTRING,
    _time: *mut REXXDATETIME,
    _wait: usize,
) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxClearQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxClearQueue(_name: *const c_char) -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxCreateSessionQueue`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxCreateSessionQueue() -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxDeleteSessionQueue`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxDeleteSessionQueue() -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxShutDownAPI`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxShutDownAPI() -> RexxReturnCode {
    NOT_AVAILABLE
}

/// `RexxAllocateMemory`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxAllocateMemory(_size: usize) -> *mut c_void {
    std::ptr::null_mut()
}

/// `RexxFreeMemory`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxFreeMemory(_block: *mut c_void) -> RexxReturnCode {
    0
}
