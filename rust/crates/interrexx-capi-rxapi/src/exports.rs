//! `librexxapi.so.4`'s C entry points, each forwarding to the definition `librexx.so.4` exports
//! This library holds no API state: `librexx.so.4`, which it
//! records as `NEEDED`, owns it, so a program linked against either library or both, in either
//! order, registers and runs against one state.
// The export modules of the C libraries.
#![allow(unsafe_code)]
#![allow(non_snake_case)]

use std::ffi::{CStr, c_char, c_void};
use std::sync::OnceLock;

use interrexx_api::layout::{
    CONSTRXSTRING, REXXPFN, RXSTRING, RexxLegacyQueueTime, RexxQueueTime, RexxReturnCode,
};

/// The definition of `name` in `librexx` (`librexx.so.4` on ELF hosts), which the loader mapped
/// as a dependency of this library. Lookup goes through `librexx`'s handle, never the global scope, where
/// this library's own definition may come first. Aborts where the library or the name is missing:
/// there is no state to answer from.
#[cfg(unix)]
fn target(name: &CStr) -> *mut c_void {
    static LIBREXX: OnceLock<usize> = OnceLock::new();
    let handle = *LIBREXX.get_or_init(|| {
        let beside = env!("REXX_LIBREXX_BESIDE");
        let Ok(soname) = std::ffi::CString::new(beside) else {
            failed("the name of librexx holds a NUL");
        };
        // SAFETY: `soname` is NUL-terminated; `RTLD_NOLOAD` maps nothing new.
        let mut handle =
            unsafe { libc::dlopen(soname.as_ptr(), libc::RTLD_NOW | libc::RTLD_NOLOAD) };
        if handle.is_null() {
            // SAFETY: as above; loading the library runs only its own initialisers.
            handle = unsafe { libc::dlopen(soname.as_ptr(), libc::RTLD_NOW) };
        }
        if handle.is_null() {
            failed(&format!("{beside} is not loaded and cannot be"));
        }
        handle.expose_provenance()
    });
    // SAFETY: `handle` came from `dlopen` and is never closed; `name` is NUL-terminated.
    let symbol =
        unsafe { libc::dlsym(std::ptr::with_exposed_provenance_mut(handle), name.as_ptr()) };
    if symbol.is_null() {
        failed(&format!(
            "{} defines no {}",
            env!("REXX_LIBREXX"),
            name.to_string_lossy()
        ));
    }
    symbol
}

/// Resolves an export in the interpreter DLL beside the forwarding DLL.
#[cfg(windows)]
fn target(name: &CStr) -> *mut c_void {
    use windows_sys::Win32::System::LibraryLoader::GetProcAddress;

    static LIBREXX: OnceLock<usize> = OnceLock::new();
    let handle = *LIBREXX.get_or_init(load_sibling);
    // SAFETY: the handle owns a process-lifetime reference to the interpreter
    // DLL, and the symbol name is NUL-terminated.
    let symbol = unsafe {
        GetProcAddress(
            std::ptr::with_exposed_provenance_mut(handle),
            name.as_ptr().cast(),
        )
    };
    let Some(symbol) = symbol else {
        failed(&format!(
            "{} defines no {}",
            env!("REXX_LIBREXX"),
            name.to_string_lossy()
        ));
    };
    #[expect(
        clippy::as_conversions,
        reason = "a function pointer has no method that answers its address"
    )]
    let symbol = symbol as *mut c_void;
    symbol
}

#[cfg(windows)]
fn load_sibling() -> usize {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::path::PathBuf;

    use interrexx_cast::CastFrom;
    use windows_sys::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        GetModuleFileNameW, GetModuleHandleExW, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
        LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LoadLibraryExW,
    };

    let mut own_module = std::ptr::null_mut();
    #[expect(
        clippy::as_conversions,
        reason = "a function item has no method that answers its address"
    )]
    let own_address = load_sibling as *const ();
    // SAFETY: FROM_ADDRESS interprets the second argument as an address inside
    // our loaded module. The output points to writable space for one handle.
    let found = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            own_address.cast::<u16>(),
            &mut own_module,
        )
    };
    if found == 0 {
        failed(&format!(
            "locating the forwarding module: {}",
            std::io::Error::last_os_error()
        ));
    }

    // Windows extended-length paths can hold up to 32,767 UTF-16 code units.
    let mut filename = vec![0u16; 32_768];
    let capacity = u32::try_from(filename.len()).expect("the buffer holds 32,768 code units");
    // SAFETY: the forwarding module is executing, and the buffer is writable
    // for the requested number of UTF-16 code units.
    let length = usize::cast_from(unsafe {
        GetModuleFileNameW(own_module, filename.as_mut_ptr(), capacity)
    });
    if length == 0 || length >= filename.len() {
        failed(&format!(
            "reading the forwarding module path: {}",
            std::io::Error::last_os_error()
        ));
    }
    let mut path = PathBuf::from(OsString::from_wide(&filename[..length]));
    path.set_file_name(env!("REXX_LIBREXX_BESIDE"));
    let path_wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // SAFETY: this is a NUL-terminated absolute path to the interpreter's
    // companion DLL. Keep the loader reference for as long as exports can run.
    let handle = unsafe {
        LoadLibraryExW(
            path_wide.as_ptr(),
            std::ptr::null_mut(),
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
        )
    };
    if handle.is_null() {
        failed(&format!(
            "loading {}: {}",
            path.display(),
            std::io::Error::last_os_error()
        ));
    }
    handle.expose_provenance()
}

fn failed(what: &str) -> ! {
    eprintln!("{}: {what}", env!("REXX_LIBREXXAPI"));
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
    names: *mut *const c_char,
    file: *const c_char,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxSaveMacroSpace` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(usize, *mut *const c_char, *const c_char) -> RexxReturnCode,
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
    names: *mut *const c_char,
    file: *const c_char,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxLoadMacroSpace` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(usize, *mut *const c_char, *const c_char) -> RexxReturnCode,
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
    data: *mut CONSTRXSTRING,
    flag: usize,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxAddQueue` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(*const c_char, *mut CONSTRXSTRING, usize) -> RexxReturnCode,
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
    time: *mut RexxLegacyQueueTime,
    wait: usize,
) -> RexxReturnCode {
    // SAFETY: `librexx.so.4` defines `RexxPullQueue` with this signature.
    let forward = unsafe {
        std::mem::transmute::<
            *mut c_void,
            unsafe extern "C" fn(
                *const c_char,
                *mut RXSTRING,
                *mut RexxLegacyQueueTime,
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

// Each export has the type `interrexx_api::exports` declares it with, which is what the
// headers in `include/` declare.
const _: () = {
    use interrexx_api::exports::signatures as declared;
    let _: declared::RexxRegisterSubcomDll = RexxRegisterSubcomDll;
    let _: declared::RexxRegisterSubcomExe = RexxRegisterSubcomExe;
    let _: declared::RexxQuerySubcom = RexxQuerySubcom;
    let _: declared::RexxDeregisterSubcom = RexxDeregisterSubcom;
    let _: declared::RexxLoadSubcom = RexxLoadSubcom;
    let _: declared::RexxRegisterExitDll = RexxRegisterExitDll;
    let _: declared::RexxRegisterExitExe = RexxRegisterExitExe;
    let _: declared::RexxDeregisterExit = RexxDeregisterExit;
    let _: declared::RexxQueryExit = RexxQueryExit;
    let _: declared::RexxRegisterFunctionDll = RexxRegisterFunctionDll;
    let _: declared::RexxRegisterFunctionExe = RexxRegisterFunctionExe;
    let _: declared::RexxDeregisterFunction = RexxDeregisterFunction;
    let _: declared::RexxQueryFunction = RexxQueryFunction;
    let _: declared::RexxAddMacro = RexxAddMacro;
    let _: declared::RexxDropMacro = RexxDropMacro;
    let _: declared::RexxSaveMacroSpace = RexxSaveMacroSpace;
    let _: declared::RexxLoadMacroSpace = RexxLoadMacroSpace;
    let _: declared::RexxQueryMacro = RexxQueryMacro;
    let _: declared::RexxReorderMacro = RexxReorderMacro;
    let _: declared::RexxClearMacroSpace = RexxClearMacroSpace;
    let _: declared::RexxCreateQueue = RexxCreateQueue;
    let _: declared::RexxOpenQueue = RexxOpenQueue;
    let _: declared::RexxQueueExists = RexxQueueExists;
    let _: declared::RexxDeleteQueue = RexxDeleteQueue;
    let _: declared::RexxQueryQueue = RexxQueryQueue;
    let _: declared::RexxAddQueue = RexxAddQueue;
    let _: declared::RexxPullFromQueue = RexxPullFromQueue;
    let _: declared::RexxPullQueue = RexxPullQueue;
    let _: declared::RexxClearQueue = RexxClearQueue;
    let _: declared::RexxCreateSessionQueue = RexxCreateSessionQueue;
    let _: declared::RexxDeleteSessionQueue = RexxDeleteSessionQueue;
    let _: declared::RexxShutDownAPI = RexxShutDownAPI;
    let _: declared::RexxAllocateMemory = RexxAllocateMemory;
    let _: declared::RexxFreeMemory = RexxFreeMemory;
};
