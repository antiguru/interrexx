//! `librexxapi.so.4`'s C entry points, each forwarding to the definition `librexx.so.4` exports
//! This library holds no API state: `librexx.so.4`, which it
//! records as `NEEDED`, owns it, so a program linked against either library or both, in either
//! order, registers and runs against one state.
// The export modules of the C libraries.
#![allow(unsafe_code)]
#![allow(non_snake_case)]

use std::ffi::{CStr, c_char, c_void};
use std::sync::OnceLock;

use rexx_api::layout::{
    CONSTRXSTRING, REXXDATETIME, REXXPFN, RXSTRING, RexxQueueTime, RexxReturnCode,
};

/// The definition of `name` in `librexx.so.4`, which the loader mapped as this library's
/// `NEEDED` entry. Lookup goes through `librexx.so.4`'s handle, never the global scope, where
/// this library's own definition may come first. Aborts where the library or the name is missing:
/// there is no state to answer from.
fn target(name: &CStr) -> *mut c_void {
    static LIBREXX: OnceLock<usize> = OnceLock::new();
    let handle = *LIBREXX.get_or_init(|| {
        let soname = c"librexx.so.4";
        // SAFETY: `soname` is NUL-terminated; `RTLD_NOLOAD` maps nothing new.
        let mut handle =
            unsafe { libc::dlopen(soname.as_ptr(), libc::RTLD_NOW | libc::RTLD_NOLOAD) };
        if handle.is_null() {
            // SAFETY: as above; loading the library runs only its own initialisers.
            handle = unsafe { libc::dlopen(soname.as_ptr(), libc::RTLD_NOW) };
        }
        if handle.is_null() {
            failed("librexx.so.4 is not loaded and cannot be");
        }
        handle as usize
    });
    // SAFETY: `handle` came from `dlopen` and is never closed; `name` is NUL-terminated.
    let symbol = unsafe { libc::dlsym(handle as *mut c_void, name.as_ptr()) };
    if symbol.is_null() {
        failed(&format!(
            "librexx.so.4 defines no {}",
            name.to_string_lossy()
        ));
    }
    symbol
}

fn failed(what: &str) -> ! {
    eprintln!("librexxapi.so.4: {what}");
    std::process::abort()
}

/// Forwards to `librexx.so.4`'s `RexxRegisterSubcomDll`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterSubcomDll(
    name: *const c_char,
    library: *const c_char,
    procedure: *const c_char,
    user: *const c_char,
    drop_authority: usize,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxRegisterSubcomDll` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(
                *const c_char,
                *const c_char,
                *const c_char,
                *const c_char,
                usize,
            ) -> RexxReturnCode,
        >(target(c"RexxRegisterSubcomDll"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, library, procedure, user, drop_authority) }
}

/// Forwards to `librexx.so.4`'s `RexxRegisterSubcomExe`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterSubcomExe(
    name: *const c_char,
    entry: REXXPFN,
    user: *const c_char,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxRegisterSubcomExe` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, REXXPFN, *const c_char) -> RexxReturnCode,
        >(target(c"RexxRegisterSubcomExe"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, entry, user) }
}

/// Forwards to `librexx.so.4`'s `RexxQuerySubcom`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQuerySubcom(
    name: *const c_char,
    library: *const c_char,
    flags: *mut u16,
    user: *mut c_char,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxQuerySubcom` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(
                *const c_char,
                *const c_char,
                *mut u16,
                *mut c_char,
            ) -> RexxReturnCode,
        >(target(c"RexxQuerySubcom"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, library, flags, user) }
}

/// Forwards to `librexx.so.4`'s `RexxDeregisterSubcom`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDeregisterSubcom(
    name: *const c_char,
    library: *const c_char,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxDeregisterSubcom` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, *const c_char) -> RexxReturnCode,
        >(target(c"RexxDeregisterSubcom"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, library) }
}

/// Forwards to `librexx.so.4`'s `RexxLoadSubcom`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxLoadSubcom(
    name: *const c_char,
    library: *const c_char,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxLoadSubcom` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, *const c_char) -> RexxReturnCode,
        >(target(c"RexxLoadSubcom"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, library) }
}

/// Forwards to `librexx.so.4`'s `RexxRegisterExitDll`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterExitDll(
    name: *const c_char,
    library: *const c_char,
    procedure: *const c_char,
    user: *const c_char,
    drop_authority: usize,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxRegisterExitDll` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(
                *const c_char,
                *const c_char,
                *const c_char,
                *const c_char,
                usize,
            ) -> RexxReturnCode,
        >(target(c"RexxRegisterExitDll"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, library, procedure, user, drop_authority) }
}

/// Forwards to `librexx.so.4`'s `RexxRegisterExitExe`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterExitExe(
    name: *const c_char,
    entry: REXXPFN,
    user: *const c_char,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxRegisterExitExe` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, REXXPFN, *const c_char) -> RexxReturnCode,
        >(target(c"RexxRegisterExitExe"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, entry, user) }
}

/// Forwards to `librexx.so.4`'s `RexxDeregisterExit`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDeregisterExit(
    name: *const c_char,
    library: *const c_char,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxDeregisterExit` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, *const c_char) -> RexxReturnCode,
        >(target(c"RexxDeregisterExit"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, library) }
}

/// Forwards to `librexx.so.4`'s `RexxQueryExit`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueryExit(
    name: *const c_char,
    library: *const c_char,
    flags: *mut u16,
    user: *mut c_char,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxQueryExit` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(
                *const c_char,
                *const c_char,
                *mut u16,
                *mut c_char,
            ) -> RexxReturnCode,
        >(target(c"RexxQueryExit"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, library, flags, user) }
}

/// Forwards to `librexx.so.4`'s `RexxRegisterFunctionDll`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterFunctionDll(
    name: *const c_char,
    library: *const c_char,
    procedure: *const c_char,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxRegisterFunctionDll` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, *const c_char, *const c_char) -> RexxReturnCode,
        >(target(c"RexxRegisterFunctionDll"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, library, procedure) }
}

/// Forwards to `librexx.so.4`'s `RexxRegisterFunctionExe`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterFunctionExe(
    name: *const c_char,
    entry: REXXPFN,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxRegisterFunctionExe` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, REXXPFN) -> RexxReturnCode,
        >(target(c"RexxRegisterFunctionExe"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, entry) }
}

/// Forwards to `librexx.so.4`'s `RexxDeregisterFunction`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDeregisterFunction(name: *const c_char) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxDeregisterFunction` with this signature.
    let forward = unsafe {
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn(*const c_char) -> RexxReturnCode>(
            target(c"RexxDeregisterFunction"),
        )
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name) }
}

/// Forwards to `librexx.so.4`'s `RexxQueryFunction`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueryFunction(name: *const c_char) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxQueryFunction` with this signature.
    let forward = unsafe {
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn(*const c_char) -> RexxReturnCode>(
            target(c"RexxQueryFunction"),
        )
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name) }
}

/// Forwards to `librexx.so.4`'s `RexxAddMacro`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxAddMacro(
    name: *const c_char,
    file: *const c_char,
    position: usize,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxAddMacro` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, *const c_char, usize) -> RexxReturnCode,
        >(target(c"RexxAddMacro"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, file, position) }
}

/// Forwards to `librexx.so.4`'s `RexxDropMacro`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDropMacro(name: *const c_char) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxDropMacro` with this signature.
    let forward = unsafe {
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn(*const c_char) -> RexxReturnCode>(
            target(c"RexxDropMacro"),
        )
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name) }
}

/// Forwards to `librexx.so.4`'s `RexxSaveMacroSpace`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxSaveMacroSpace(
    count: usize,
    names: *const *const c_char,
    file: *const c_char,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxSaveMacroSpace` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(usize, *const *const c_char, *const c_char) -> RexxReturnCode,
        >(target(c"RexxSaveMacroSpace"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(count, names, file) }
}

/// Forwards to `librexx.so.4`'s `RexxLoadMacroSpace`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxLoadMacroSpace(
    count: usize,
    names: *const *const c_char,
    file: *const c_char,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxLoadMacroSpace` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(usize, *const *const c_char, *const c_char) -> RexxReturnCode,
        >(target(c"RexxLoadMacroSpace"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(count, names, file) }
}

/// Forwards to `librexx.so.4`'s `RexxQueryMacro`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueryMacro(name: *const c_char, position: *mut u16) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxQueryMacro` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, *mut u16) -> RexxReturnCode,
        >(target(c"RexxQueryMacro"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, position) }
}

/// Forwards to `librexx.so.4`'s `RexxReorderMacro`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxReorderMacro(name: *const c_char, position: usize) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxReorderMacro` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, usize) -> RexxReturnCode,
        >(target(c"RexxReorderMacro"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, position) }
}

/// Forwards to `librexx.so.4`'s `RexxClearMacroSpace`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxClearMacroSpace() -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxClearMacroSpace` with this signature.
    let forward = unsafe {
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn() -> RexxReturnCode>(target(
            c"RexxClearMacroSpace",
        ))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward() }
}

/// Forwards to `librexx.so.4`'s `RexxCreateQueue`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxCreateQueue(
    name: *mut c_char,
    size: usize,
    requested: *const c_char,
    duplicate: *mut usize,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxCreateQueue` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*mut c_char, usize, *const c_char, *mut usize) -> RexxReturnCode,
        >(target(c"RexxCreateQueue"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, size, requested, duplicate) }
}

/// Forwards to `librexx.so.4`'s `RexxOpenQueue`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxOpenQueue(
    name: *const c_char,
    duplicate: *mut usize,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxOpenQueue` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, *mut usize) -> RexxReturnCode,
        >(target(c"RexxOpenQueue"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, duplicate) }
}

/// Forwards to `librexx.so.4`'s `RexxQueueExists`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueueExists(name: *const c_char) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxQueueExists` with this signature.
    let forward = unsafe {
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn(*const c_char) -> RexxReturnCode>(
            target(c"RexxQueueExists"),
        )
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name) }
}

/// Forwards to `librexx.so.4`'s `RexxDeleteQueue`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDeleteQueue(name: *const c_char) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxDeleteQueue` with this signature.
    let forward = unsafe {
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn(*const c_char) -> RexxReturnCode>(
            target(c"RexxDeleteQueue"),
        )
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name) }
}

/// Forwards to `librexx.so.4`'s `RexxQueryQueue`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueryQueue(name: *const c_char, count: *mut usize) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxQueryQueue` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, *mut usize) -> RexxReturnCode,
        >(target(c"RexxQueryQueue"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, count) }
}

/// Forwards to `librexx.so.4`'s `RexxAddQueue`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxAddQueue(
    name: *const c_char,
    data: *const CONSTRXSTRING,
    flag: usize,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxAddQueue` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, *const CONSTRXSTRING, usize) -> RexxReturnCode,
        >(target(c"RexxAddQueue"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, data, flag) }
}

/// Forwards to `librexx.so.4`'s `RexxPullFromQueue`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxPullFromQueue(
    name: *const c_char,
    data: *mut RXSTRING,
    time: *mut RexxQueueTime,
    wait: usize,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxPullFromQueue` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(
                *const c_char,
                *mut RXSTRING,
                *mut RexxQueueTime,
                usize,
            ) -> RexxReturnCode,
        >(target(c"RexxPullFromQueue"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, data, time, wait) }
}

/// Forwards to `librexx.so.4`'s `RexxPullQueue`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxPullQueue(
    name: *const c_char,
    data: *mut RXSTRING,
    time: *mut REXXDATETIME,
    wait: usize,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxPullQueue` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(
                *const c_char,
                *mut RXSTRING,
                *mut REXXDATETIME,
                usize,
            ) -> RexxReturnCode,
        >(target(c"RexxPullQueue"))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name, data, time, wait) }
}

/// Forwards to `librexx.so.4`'s `RexxClearQueue`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxClearQueue(name: *const c_char) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxClearQueue` with this signature.
    let forward = unsafe {
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn(*const c_char) -> RexxReturnCode>(
            target(c"RexxClearQueue"),
        )
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(name) }
}

/// Forwards to `librexx.so.4`'s `RexxCreateSessionQueue`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxCreateSessionQueue() -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxCreateSessionQueue` with this signature.
    let forward = unsafe {
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn() -> RexxReturnCode>(target(
            c"RexxCreateSessionQueue",
        ))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward() }
}

/// Forwards to `librexx.so.4`'s `RexxDeleteSessionQueue`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDeleteSessionQueue() -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxDeleteSessionQueue` with this signature.
    let forward = unsafe {
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn() -> RexxReturnCode>(target(
            c"RexxDeleteSessionQueue",
        ))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward() }
}

/// Forwards to `librexx.so.4`'s `RexxShutDownAPI`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxShutDownAPI() -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxShutDownAPI` with this signature.
    let forward = unsafe {
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn() -> RexxReturnCode>(target(
            c"RexxShutDownAPI",
        ))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward() }
}

/// Forwards to `librexx.so.4`'s `RexxAllocateMemory`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxAllocateMemory(size: usize) -> *mut c_void {
    // SAFETY: `librexx.so.4` defines `RexxAllocateMemory` with this signature.
    let forward = unsafe {
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn(usize) -> *mut c_void>(target(
            c"RexxAllocateMemory",
        ))
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(size) }
}

/// Forwards to `librexx.so.4`'s `RexxFreeMemory`.
///
/// # Safety
/// As that function asks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxFreeMemory(block: *mut c_void) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxFreeMemory` with this signature.
    let forward = unsafe {
        std::mem::transmute::<*mut c_void, unsafe extern "C" fn(*mut c_void) -> RexxReturnCode>(
            target(c"RexxFreeMemory"),
        )
    };
    // SAFETY: the caller's guarantee.
    unsafe { forward(block) }
}
