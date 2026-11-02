//! The `librexx` C entry points: shims over `rexx_exec`. The `rexx` executable compiles this file
//! too, so an extension it loads reaches its interpreter.
// D-U1 as extended 2026-11-02: one export module per C library.
#![allow(unsafe_code)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_int, c_short, c_void};

use rexx_api::layout::{
    CONSTRXSTRING, RXSTRING, RXSYSEXIT, RexxInstance_, RexxReturnCode, RexxStemObject,
    RexxThreadContext_, SHVBLOCK,
};

/// `RXARI_NOT_FOUND`.
const TARGET_NOT_FOUND: RexxReturnCode = 1;

/// `RXENDLST`, which ends a list of system exits.
const END_OF_EXITS: c_int = 0;

/// `RexxStart`: the program `name`, or the in-store source `instore[0]`, run with `argc`
/// arguments in an interpreter of its own, called as `call_type` says, with `environment` the
/// initial `ADDRESS`. The result goes into `result`'s buffer where it fits, else a block
/// `RexxFreeMemory` releases. System exits are refused.
///
/// # Safety
/// Every pointer is null or what `api/rexx.h` declares: `argv` holds `argc` strings each
/// readable for its length (a null `strptr` an omitted argument), `name` and `environment` are
/// NUL-terminated, `instore` holds two strings, `exits` is a list ending `RXENDLST`, and `rc`
/// and `result` are writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxStart(
    argc: usize,
    argv: *const CONSTRXSTRING,
    name: *const c_char,
    instore: *mut RXSTRING,
    environment: *const c_char,
    call_type: c_int,
    exits: *mut c_void,
    rc: *mut c_short,
    result: *mut RXSTRING,
) -> c_int {
    // SAFETY: a non-null list begins with an `RXSYSEXIT`, whose second member is its code.
    if !exits.is_null() && unsafe { exits.cast::<RXSYSEXIT>().read() }.sysexit_code != END_OF_EXITS
    {
        eprintln!("RexxStart's system exits are not implemented (Phase 10 Embedding)");
        return -1;
    }
    // SAFETY: the caller's guarantee for each string.
    let text = |strptr: *const c_char, length: usize| unsafe {
        std::slice::from_raw_parts(strptr.cast::<u8>(), length).to_vec()
    };
    let arguments = (0..argc)
        .map(|index| {
            // SAFETY: `argv` holds `argc` strings.
            let argument = unsafe { *argv.add(index) };
            (!argument.strptr.is_null()).then(|| text(argument.strptr, argument.strlength))
        })
        .collect();
    // SAFETY: as above, for the in-store pair.
    let source = (!instore.is_null())
        .then(|| unsafe { *instore })
        .filter(|source| !source.strptr.is_null())
        .map(|source| text(source.strptr, source.strlength));
    // SAFETY: NUL-terminated where non-null.
    let c_text = |pointer: *const c_char| unsafe {
        (!pointer.is_null()).then(|| std::ffi::CStr::from_ptr(pointer).to_bytes().to_vec())
    };
    let start = match call_type {
        1 => rexx_exec::StartType::Subroutine,
        2 => rexx_exec::StartType::Function,
        _ => rexx_exec::StartType::Command,
    };
    let started = rexx_exec::embedding::rexx_start(rexx_exec::embedding::Start {
        name: c_text(name).unwrap_or_default(),
        source,
        arguments,
        environment: c_text(environment).filter(|environment| !environment.is_empty()),
        start,
    });
    if !rc.is_null() {
        // SAFETY: the caller's guarantee.
        unsafe { rc.write(started.short) };
    }
    // SAFETY: the caller's guarantee.
    if let (Some(result), Some(value)) = (unsafe { result.as_mut() }, started.result) {
        if result.strptr.is_null() || result.strlength <= value.len() {
            // SAFETY: `malloc` of a non-zero size; `RexxFreeMemory` is `free`.
            result.strptr = unsafe { libc::malloc(value.len() + 1) }.cast();
        }
        if !result.strptr.is_null() {
            // SAFETY: the buffer holds at least `value.len() + 1` bytes: ours, or the caller's
            // of `strlength`, which is more than `value.len()`. The result is NUL-terminated.
            unsafe {
                std::ptr::copy_nonoverlapping(value.as_ptr(), result.strptr.cast(), value.len());
                result.strptr.add(value.len()).write(0);
            }
            result.strlength = value.len();
        }
    }
    started.code
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
