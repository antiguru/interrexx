//! The `librexx` C entry points: shims over `rexx_exec`. The `rexx` executable compiles this file
//! too, so an extension it loads reaches its interpreter.
// D-U1 as extended 2026-11-02: one export module per C library.
#![allow(unsafe_code)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_int, c_short, c_void};

use rexx_api::layout::{
    CONSTRXSTRING, RXSTRING, RexxInstance_, RexxReturnCode, RexxStemObject, RexxThreadContext_,
    SHVBLOCK,
};

/// `RXARI_NOT_FOUND`.
const TARGET_NOT_FOUND: RexxReturnCode = 1;

/// `RexxStart`.
///
/// # Safety
/// Every pointer is null or what `api/rexx.h` declares.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxStart(
    _argc: usize,
    _argv: *const CONSTRXSTRING,
    _name: *const c_char,
    _instore: *mut RXSTRING,
    _environment: *const c_char,
    _call_type: c_int,
    _exits: *mut c_void,
    _rc: *mut c_short,
    _result: *mut RXSTRING,
) -> c_int {
    -1
}

/// `RexxWaitForTermination`, a no-op the header keeps for binary compatibility.
#[unsafe(no_mangle)]
pub extern "C" fn RexxWaitForTermination() {}

/// `RexxDidRexxTerminate`, kept for binary compatibility.
#[unsafe(no_mangle)]
pub extern "C" fn RexxDidRexxTerminate() -> RexxReturnCode {
    1
}

/// `RexxTranslateProgram`.
///
/// # Safety
/// As [`RexxStart`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxTranslateProgram(
    _input: *const c_char,
    _output: *const c_char,
    _exits: *mut c_void,
) -> RexxReturnCode {
    TARGET_NOT_FOUND
}

/// `RexxTranslateInstoreProgram`.
///
/// # Safety
/// As [`RexxStart`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxTranslateInstoreProgram(
    _name: *const c_char,
    _source: *const CONSTRXSTRING,
    _image: *mut RXSTRING,
) -> RexxReturnCode {
    TARGET_NOT_FOUND
}

/// `RexxCompileProgram`.
///
/// # Safety
/// As [`RexxStart`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxCompileProgram(
    _input: *const c_char,
    _output: *const c_char,
    _exits: *mut c_void,
    _encode: bool,
) -> RexxReturnCode {
    TARGET_NOT_FOUND
}

/// `RexxCreateInterpreterImage`.
///
/// # Safety
/// As [`RexxStart`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxCreateInterpreterImage(_target: *const c_char) {}

/// `RexxGetVersionInformation`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxGetVersionInformation() -> *mut c_char {
    std::ptr::null_mut()
}

/// `RexxGetErrorMessage`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxGetErrorMessage(_code: c_int) -> *const c_char {
    std::ptr::null()
}

/// `RexxGetErrorMessageByNumber`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxGetErrorMessageByNumber(_number: c_int) -> *const c_char {
    std::ptr::null()
}

/// `RexxVariablePool`, against the activation that called the routine, classic function or
/// exit this thread runs; `RXSHV_NOAVL` where it runs none.
///
/// # Safety
/// As [`RexxStart`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxVariablePool(blocks: *mut SHVBLOCK) -> RexxReturnCode {
    // SAFETY: the caller's guarantee, which is `variable_pool`'s.
    unsafe { rexx_api::ffi::classic::variable_pool(blocks) }
}

/// `RexxCallBack`: a label of the program whose external function this thread runs, called
/// as an internal function. A condition or `EXIT` in it unwinds the external function.
///
/// # Safety
/// As [`RexxStart`].
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn RexxCallBack(
    label: *const c_char,
    argc: usize,
    argv: *const CONSTRXSTRING,
    rc: *mut c_short,
    result: *mut RXSTRING,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee, which is `call_back`'s.
    unsafe { rexx_api::ffi::classic::call_back(label, argc, argv, rc, result) }
}

/// `RexxSetHalt`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxSetHalt(_process: i32, _thread: usize) -> RexxReturnCode {
    TARGET_NOT_FOUND
}

/// `RexxSetTrace`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxSetTrace(_process: i32, _thread: usize) -> RexxReturnCode {
    TARGET_NOT_FOUND
}

/// `RexxResetTrace`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxResetTrace(_process: i32, _thread: usize) -> RexxReturnCode {
    TARGET_NOT_FOUND
}

/// `RexxCreateInterpreter`: 0, no instance.
///
/// # Safety
/// As [`RexxStart`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxCreateInterpreter(
    _instance: *mut *mut RexxInstance_,
    _thread: *mut *mut RexxThreadContext_,
    _options: *mut c_void,
) -> RexxReturnCode {
    0
}

/// `RexxStemSort`.
///
/// # Safety
/// As [`RexxStart`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxStemSort(
    _stem: RexxStemObject,
    _tail: *const c_char,
    _order: c_int,
    _kind: c_int,
    _start: isize,
    _end: isize,
    _first_column: isize,
    _last_column: isize,
) -> RexxReturnCode {
    TARGET_NOT_FOUND
}
