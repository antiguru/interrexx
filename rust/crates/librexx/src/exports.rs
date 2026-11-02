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
    // SAFETY: as above, for the in-store pair, a null `strptr` an absent string.
    let pair = (!instore.is_null()).then(|| unsafe { [*instore, *instore.add(1)] });
    let held = |string: RXSTRING| {
        (!string.strptr.is_null()).then(|| text(string.strptr, string.strlength))
    };
    let instore_pair = pair.map(|[source, image]| rexx_exec::embedding::Instore {
        source: held(source),
        image: held(image),
    });
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
        instore: instore_pair,
        arguments,
        environment: c_text(environment).filter(|environment| !environment.is_empty()),
        start,
        exits,
    });
    if let (false, Some(short)) = (rc.is_null(), started.short) {
        // SAFETY: the caller's guarantee.
        unsafe { rc.write(short) };
    }
    if let Some(image) = started.image {
        // SAFETY: `instore` holds two strings, the second the caller's to receive the image.
        unsafe { give_block(&mut *instore.add(1), &image) };
    }
    // SAFETY: the caller's guarantee.
    let mut result = unsafe { result.as_mut() };
    // A run that answered nothing empties the caller's result (`RexxStartDispatcher::run`); a
    // `//T` translation, which leaves the short alone, ran nothing and leaves it too.
    let ran = started.short.is_some();
    if let (Some(result), 0, None, true) =
        (result.as_deref_mut(), started.code, &started.result, ran)
    {
        result.strptr = std::ptr::null_mut();
        result.strlength = 0;
    }
    if let (Some(result), Some(value)) = (result, started.result) {
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

/// Hands `bytes` back through `string` in a block `RexxFreeMemory` releases, NUL-terminated;
/// a null block where none could be allocated.
///
/// # Safety
/// `string` is writable.
unsafe fn give_block(string: &mut RXSTRING, bytes: &[u8]) {
    // SAFETY: `malloc` of a non-zero size; `RexxFreeMemory` is `free`.
    let block = unsafe { libc::malloc(bytes.len() + 1) }.cast::<u8>();
    if block.is_null() {
        string.strptr = std::ptr::null_mut();
        string.strlength = 0;
        return;
    }
    // SAFETY: the block holds `bytes.len() + 1` bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), block, bytes.len());
        block.add(bytes.len()).write(0);
    }
    string.strptr = block.cast();
    string.strlength = bytes.len();
}

/// `RexxWaitForTermination`, a no-op the header keeps for binary compatibility.
#[unsafe(no_mangle)]
pub extern "C" fn RexxWaitForTermination() {}

/// `RexxDidRexxTerminate`, kept for binary compatibility.
#[unsafe(no_mangle)]
pub extern "C" fn RexxDidRexxTerminate() -> RexxReturnCode {
    1
}

/// `RexxTranslateProgram`: `RexxCompileProgram` without encoding.
///
/// # Safety
/// As [`RexxCompileProgram`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxTranslateProgram(
    input: *const c_char,
    output: *const c_char,
    exits: *mut c_void,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    unsafe { RexxCompileProgram(input, output, exits, false) }
}

/// `RexxTranslateInstoreProgram`: `source`, named `name`, translated, its image into `image`
/// in a block `RexxFreeMemory` releases; 0, or the negated error number, leaving `image`.
///
/// # Safety
/// `name` is null or NUL-terminated, `source` is readable for its length, and `image` is
/// writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxTranslateInstoreProgram(
    name: *const c_char,
    source: *const CONSTRXSTRING,
    image: *mut RXSTRING,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    let name = unsafe { text_of(name) }.unwrap_or_default();
    // SAFETY: the caller's guarantee.
    let source = unsafe { source.as_ref() }
        .filter(|source| !source.strptr.is_null())
        .map_or_else(Vec::new, |source| unsafe {
            std::slice::from_raw_parts(source.strptr.cast::<u8>(), source.strlength).to_vec()
        });
    match rexx_exec::embedding::translate_instore(&name, source) {
        Ok(bytes) => {
            // SAFETY: the caller's guarantee.
            if let Some(image) = unsafe { image.as_mut() } {
                // SAFETY: `image` is writable.
                unsafe { give_block(image, &bytes) };
            }
            0
        }
        Err(code) => code,
    }
}

/// `RexxCompileProgram`: the program file `input` translated and, where `output` is not null,
/// its image written to that file, base64-encoded where `encode` is set; 0, or the negated
/// error number. The system exits are not called: a translation runs no exit's event.
///
/// # Safety
/// `input` and `output` are null or NUL-terminated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxCompileProgram(
    input: *const c_char,
    output: *const c_char,
    _exits: *mut c_void,
    encode: bool,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    let (input, output) = unsafe { (text_of(input), text_of(output)) };
    rexx_exec::embedding::translate_file(&input.unwrap_or_default(), output.as_deref(), encode)
}

/// `RexxCreateInterpreterImage`.
///
/// # Safety
/// As [`RexxStart`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxCreateInterpreterImage(_target: *const c_char) {}

/// `RexxGetVersionInformation`: the version banner in a block `RexxFreeMemory` releases, or
/// null where none could be allocated.
#[unsafe(no_mangle)]
pub extern "C" fn RexxGetVersionInformation() -> *mut c_char {
    let text = rexx_exec::embedding::version_information();
    // SAFETY: `malloc` of a non-zero size; `RexxFreeMemory` is `free`.
    let block = unsafe { libc::malloc(text.len() + 1) }.cast::<u8>();
    if !block.is_null() {
        // SAFETY: the block holds `text.len() + 1` bytes.
        unsafe {
            std::ptr::copy_nonoverlapping(text.as_ptr(), block, text.len());
            block.add(text.len()).write(0);
        }
    }
    block.cast()
}

/// `RexxGetErrorMessage`: the text of the message `code` (major times 1000 plus minor), null
/// where there is none. The text lives as long as the library.
#[unsafe(no_mangle)]
pub extern "C" fn RexxGetErrorMessage(code: c_int) -> *const c_char {
    rexx_exec::embedding::error_message(code).map_or(std::ptr::null(), std::ffi::CStr::as_ptr)
}

/// `RexxGetErrorMessageByNumber`: the text of the message `number` maps to, null where there
/// is none.
#[unsafe(no_mangle)]
pub extern "C" fn RexxGetErrorMessageByNumber(number: c_int) -> *const c_char {
    rexx_exec::embedding::error_message_by_number(number)
        .map_or(std::ptr::null(), std::ffi::CStr::as_ptr)
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

/// `RXARI_OK` where `done`, else `RXARI_NOT_FOUND`.
fn found(done: bool) -> RexxReturnCode {
    if done { 0 } else { TARGET_NOT_FOUND }
}

/// `RexxSetHalt`: the `RexxStart` run on `thread` halted at its next clause. The process id is
/// ignored, as the oracle ignores it.
#[unsafe(no_mangle)]
pub extern "C" fn RexxSetHalt(_process: i32, thread: usize) -> RexxReturnCode {
    found(rexx_exec::embedding::classic_halt(thread))
}

/// `RexxSetTrace`: the external trace of the `RexxStart` run on `thread` turned on.
#[unsafe(no_mangle)]
pub extern "C" fn RexxSetTrace(_process: i32, thread: usize) -> RexxReturnCode {
    found(rexx_exec::embedding::classic_trace(thread, true))
}

/// `RexxResetTrace`: the external trace of the `RexxStart` run on `thread` turned off.
#[unsafe(no_mangle)]
pub extern "C" fn RexxResetTrace(_process: i32, thread: usize) -> RexxReturnCode {
    found(rexx_exec::embedding::classic_trace(thread, false))
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
