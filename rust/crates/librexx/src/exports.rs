//! The `librexx` C entry points: shims over `rexx_exec`. The `rexx` executable compiles this file
//! too, so an extension it loads reaches its interpreter.
// D-U1 as extended 2026-11-02: one export module per C library.
#![allow(unsafe_code)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_int, c_short, c_void};

use rexx_api::layout::{
    CONSTRXSTRING, RXSTRING, RXSYSEXIT, RexxContextExit, RexxInstance_, RexxReturnCode,
    RexxStemObject, RexxThreadContext_, SHVBLOCK, ValueDescriptor,
};

/// `RXARI_NOT_FOUND`.
const TARGET_NOT_FOUND: RexxReturnCode = 1;

/// `RXENDLST`, which ends a list of system exits.
const END_OF_EXITS: c_int = 0;

/// The entries of an `RXSYSEXIT` list up to its `RXENDLST`, each a function code and a name.
///
/// # Safety
/// A non-null `list` is such a list, each name before the end NUL-terminated or null.
unsafe fn registered_exits(list: *const RXSYSEXIT) -> Vec<(c_int, Vec<u8>)> {
    let mut out = Vec::new();
    if list.is_null() {
        return out;
    }
    let mut at = list;
    // SAFETY: the caller's guarantee, entry by entry.
    unsafe {
        while (*at).sysexit_code != END_OF_EXITS {
            out.push((
                (*at).sysexit_code,
                text_of((*at).sysexit_name).unwrap_or_default(),
            ));
            at = at.add(1);
        }
    }
    out
}

/// The entries of a `RexxContextExit` list up to its `RXENDLST`, each a function code and its
/// handler; a null handler installs nothing.
///
/// # Safety
/// A non-null `list` is such a list, each handler before the end a `RexxContextExitHandler`
/// the caller keeps callable while the instance lives.
unsafe fn context_exits(list: *const RexxContextExit) -> Vec<(c_int, rexx_api::load::ContextExit)> {
    let mut out = Vec::new();
    if list.is_null() {
        return out;
    }
    let mut at = list;
    // SAFETY: the caller's guarantee, entry by entry.
    unsafe {
        while (*at).sysexit_code != END_OF_EXITS {
            if let Some(handler) = rexx_api::load::ContextExit::new((*at).handler) {
                out.push(((*at).sysexit_code, handler));
            }
            at = at.add(1);
        }
    }
    out
}

/// `RexxStart`: the program `name`, or the in-store source `instore[0]`, run with `argc`
/// arguments in an interpreter of its own, called as `call_type` says, with `environment` the
/// initial `ADDRESS` and the system exits `exits` names. The result goes into `result`'s buffer
/// where it fits, else a block `RexxFreeMemory` releases.
///
/// # Safety
/// Every pointer is null or what `api/rexx.h` declares: `argv` holds `argc` strings each
/// readable for its length (a null `strptr` an omitted argument), `name` and `environment` are
/// NUL-terminated, `instore` holds two strings, `exits` is a list ending `RXENDLST` whose names
/// are NUL-terminated, and `rc` and `result` are writable.
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
    // SAFETY: the caller's guarantee.
    let exits = unsafe { registered_exits(exits.cast()) };
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
        exits,
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

/// `RexxOption` (`api/oorexxapi.h`): a name and a value of the type the name
/// implies, read here as the word it is.
#[repr(C)]
struct RexxOption {
    name: *const c_char,
    option: ValueDescriptor,
}

/// `RexxContextEnvironment`, `RexxRedirectingEnvironment` and
/// `RexxRegisteredEnvironment`: a pointer and a name.
#[repr(C)]
struct NamedEntry {
    first: *mut c_void,
    name: *const c_char,
}

/// The bytes of a NUL-terminated string, or `None` for a null pointer.
///
/// # Safety
/// A non-null `text` is NUL-terminated.
unsafe fn text_of(text: *const c_char) -> Option<Vec<u8>> {
    // SAFETY: the caller's guarantee.
    (!text.is_null()).then(|| {
        unsafe { std::ffi::CStr::from_ptr(text) }
            .to_bytes()
            .to_vec()
    })
}

/// Each entry of a list ending at a null name or a null first member.
///
/// # Safety
/// A non-null `list` is such a list.
unsafe fn entries(list: *const NamedEntry) -> Vec<(*mut c_void, Vec<u8>)> {
    let mut out = Vec::new();
    if list.is_null() {
        return out;
    }
    let mut at = list;
    // SAFETY: the caller's guarantee: each entry up to the terminator is
    // readable and its name NUL-terminated.
    unsafe {
        while !(*at).name.is_null() && !(*at).first.is_null() {
            out.push(((*at).first, text_of((*at).name).unwrap_or_default()));
            at = at.add(1);
        }
    }
    out
}

/// The options `options` lists, up to the entry with a null name.
///
/// # Safety
/// A non-null `options` is a list of `RexxOption`s ending with a null name,
/// each value of the type its name implies.
unsafe fn instance_options(options: *const RexxOption) -> rexx_exec::embedding::InstanceOptions {
    let mut read = rexx_exec::embedding::InstanceOptions::default();
    if options.is_null() {
        return read;
    }
    let mut at = options;
    // SAFETY: the caller's guarantee, entry by entry.
    unsafe {
        while !(*at).name.is_null() {
            let name = std::ffi::CStr::from_ptr((*at).name).to_bytes();
            let word = (*at).option.value.value_POINTER;
            match name {
                b"InitialAddress" => read.initial_address = text_of(word.cast()),
                b"ApplicationData" => read.application_data = word as usize,
                b"LoadRequiredLibrary" => read.load_libraries.extend(text_of(word.cast())),
                b"ExternalCallPath" => read.external_call_path = text_of(word.cast()),
                b"ExternalCallPathExt" => {
                    let list = text_of(word.cast()).unwrap_or_default();
                    read.external_call_extensions
                        .extend(list.split(|byte| *byte == b',').map(<[u8]>::to_vec));
                }
                b"DirectEnvironments" | b"RedirectingEnvironments" => {
                    let kind = if name == b"DirectEnvironments" { 1 } else { 2 };
                    for (handler, name) in entries(word.cast()) {
                        read.command_handlers.extend(
                            rexx_api::load::CommandHandler::sendable(handler, kind)
                                .map(|handler| (name, handler)),
                        );
                    }
                }
                b"RegisteredExits" => read.registered_exits.extend(registered_exits(word.cast())),
                b"DirectExits" => read.context_exits.extend(context_exits(word.cast())),
                b"RegisteredEnvironments" => {
                    for (registered, name) in entries(word.cast()) {
                        let registered = text_of(registered.cast()).unwrap_or_default();
                        read.registered_environments.push((name, registered));
                    }
                }
                _ => {}
            }
            at = at.add(1);
        }
    }
    read
}

/// `RexxCreateInterpreter` (`api/oorexxapi.h`): an instance of its own, the
/// calling thread attached to its root thread context. Answers 1 with the
/// instance and the context written, else 0.
///
/// # Safety
/// `instance` and `thread` are writable, and a non-null `options` is a list
/// of `RexxOption`s ending with a null name.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxCreateInterpreter(
    instance: *mut *mut RexxInstance_,
    thread: *mut *mut RexxThreadContext_,
    options: *mut c_void,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    let options = unsafe { instance_options(options.cast()) };
    let Some(context) = rexx_exec::embedding::create_instance(options) else {
        return 0;
    };
    let context = context as *mut RexxThreadContext_;
    // SAFETY: `context` is the live root thread context, whose `instance`
    // link its interpreter wrote; the writes are the caller's guarantee.
    unsafe {
        if !thread.is_null() {
            thread.write(context);
        }
        if !instance.is_null() {
            instance.write((*context).instance);
        }
    }
    1
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
