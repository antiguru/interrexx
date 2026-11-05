//! The `librexxapi` C entry points: shims over `interrexx::capi`, exported by `librexx`, which
//! owns the API state. `librexxapi.so.4` forwards each name here.
//! The `rexx` executable compiles this file too, so an extension it loads reaches its own state.
// The export modules of the C libraries.
#![allow(unsafe_code)]
#![allow(non_snake_case)]

use std::ffi::{CStr, c_char, c_void};

use interrexx::capi::{self, Registry, Timestamp};
use interrexx_api::layout::{
    CONSTRXSTRING, REXXDATETIME, REXXPFN, RXSTRING, RexxQueueTime, RexxReturnCode,
};
use interrexx_api::load::ClassicEntry;

fn code(status: u64) -> RexxReturnCode {
    RexxReturnCode::try_from(status).unwrap_or(RexxReturnCode::MAX)
}

/// The bytes of the C string at `pointer`, `None` for null.
///
/// # Safety
/// `pointer` is null or a NUL-terminated string that outlives `'a`.
unsafe fn text<'a>(pointer: *const c_char) -> Option<&'a [u8]> {
    // SAFETY: the caller's guarantee.
    (!pointer.is_null()).then(|| unsafe { CStr::from_ptr(pointer) }.to_bytes())
}

/// The bytes of a name the API requires, the empty string for null.
///
/// # Safety
/// As [`text`].
unsafe fn name<'a>(pointer: *const c_char) -> &'a [u8] {
    // SAFETY: the caller's guarantee.
    unsafe { text(pointer) }.unwrap_or_default()
}

/// The 16 bytes of a user area, `None` for null.
///
/// # Safety
/// `pointer` is null or addresses 16 readable bytes.
unsafe fn user_area(pointer: *const c_char) -> Option<[u8; 16]> {
    // SAFETY: the caller's guarantee.
    (!pointer.is_null()).then(|| unsafe { pointer.cast::<[u8; 16]>().read_unaligned() })
}

/// The names a macrospace call lists, `None` for a null list.
///
/// # Safety
/// `names` is null or addresses `count` pointers, each as [`text`] asks.
unsafe fn names(count: usize, names: *const *const c_char) -> Option<Vec<Vec<u8>>> {
    if names.is_null() {
        return None;
    }
    // SAFETY: the caller's guarantee.
    let list = unsafe { std::slice::from_raw_parts(names, count) };
    // SAFETY: as above, for each element.
    Some(
        list.iter()
            .map(|&each| unsafe { name(each) }.to_vec())
            .collect(),
    )
}

/// Hands `bytes` back through `data`: into the caller's buffer where it is strictly shorter than
/// the buffer, else into a new block `RexxFreeMemory` releases, with a NUL after the bytes.
/// `false` where no block could be allocated.
///
/// # Safety
/// `data` addresses an initialised `RXSTRING` whose non-null `strptr` has `strlength` writable
/// bytes.
unsafe fn hand_back(data: *mut RXSTRING, bytes: &[u8]) -> bool {
    // SAFETY: the caller's guarantee.
    let data = unsafe { &mut *data };
    if !data.strptr.is_null() && bytes.len() < data.strlength {
        // SAFETY: the buffer holds `strlength` bytes, more than `bytes.len()`.
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), data.strptr.cast(), bytes.len()) };
        data.strlength = bytes.len();
        return true;
    }
    // SAFETY: `malloc` has no precondition.
    let block = unsafe { libc::malloc(bytes.len() + 1) }.cast::<u8>();
    if block.is_null() {
        return false;
    }
    // SAFETY: the block holds `bytes.len() + 1` bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), block, bytes.len());
        block.add(bytes.len()).write(0);
    }
    data.strptr = block.cast();
    data.strlength = bytes.len();
    true
}

fn queue_time(time: Timestamp) -> RexxQueueTime {
    RexxQueueTime {
        hours: time.hours,
        minutes: time.minutes,
        seconds: time.seconds,
        hundredths: 0,
        day: time.day,
        month: time.month,
        year: time.year,
        weekday: time.weekday,
        microseconds: 0,
        yearday: u32::from(time.yearday),
    }
}

/// `RexxPullFromQueue` and `RexxPullQueue`: the status, with the item's time where an item was
/// handed back.
///
/// # Safety
/// As [`hand_back`], and `name` as [`text`].
unsafe fn pull(
    name: *const c_char,
    data: *mut RXSTRING,
    wait: usize,
) -> (RexxReturnCode, Option<Timestamp>) {
    // SAFETY: the caller's guarantee.
    let (status, item) = capi::pull_queue(unsafe { text(name) }, wait as u64);
    let Some(item) = item else {
        return (code(status), None);
    };
    // SAFETY: the caller's guarantee.
    if !data.is_null() && !unsafe { hand_back(data, &item.data) } {
        return (code(capi::MEMORY_FAILURE), None);
    }
    (code(status), Some(item.time))
}

/// An entry-point registration.
///
/// # Safety
/// `name` and `user` as [`text`] and [`user_area`] ask; a non-null `entry` is a handler of
/// `registry`'s signature, mapped for the process's life.
unsafe fn register_entry(
    registry: Registry,
    name: *const c_char,
    entry: REXXPFN,
    user: *const c_char,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee that `entry` is such a handler.
    let Some(entry) = (unsafe { ClassicEntry::new(entry) }) else {
        return code(capi::BAD_ENTRY);
    };
    // SAFETY: the caller's guarantee.
    let status =
        unsafe { capi::register_entry(registry, self::name(name), entry, user_area(user)) };
    code(status)
}

/// A query: `*flags` is 0, then 1 when found, and `user` gets the stored area when found.
///
/// # Safety
/// As [`RexxQuerySubcom`].
unsafe fn query(
    registry: Registry,
    name: *const c_char,
    library: *const c_char,
    flags: *mut u16,
    user: *mut c_char,
) -> RexxReturnCode {
    if !flags.is_null() {
        // SAFETY: the caller's guarantee.
        unsafe { flags.write(0) };
    }
    // SAFETY: the caller's guarantee.
    let (status, area) =
        unsafe { capi::query_registration(registry, self::name(name), text(library)) };
    if status == 0 {
        if !flags.is_null() {
            // SAFETY: as above.
            unsafe { flags.write(1) };
        }
        if let Some(area) = area
            && !user.is_null()
        {
            // SAFETY: the caller's guarantee of 16 writable bytes.
            unsafe { user.cast::<[u8; 16]>().write_unaligned(area) };
        }
    }
    code(status)
}

/// A library registration.
///
/// # Safety
/// Each pointer as [`text`] or [`user_area`] asks.
unsafe fn register_library(
    registry: Registry,
    name: *const c_char,
    library: *const c_char,
    procedure: *const c_char,
    user: *const c_char,
    drop_authority: usize,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee, for each pointer.
    let status = unsafe {
        capi::register_library(
            registry,
            self::name(name),
            self::name(library),
            self::name(procedure),
            user_area(user),
            drop_authority,
        )
    };
    code(status)
}

/// `RexxRegisterSubcomDll`.
///
/// # Safety
/// Every pointer is null or what `api/rexx.h` declares.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterSubcomDll(
    name: *const c_char,
    library: *const c_char,
    procedure: *const c_char,
    user: *const c_char,
    drop_authority: usize,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    unsafe {
        register_library(
            Registry::Subcom,
            name,
            library,
            procedure,
            user,
            drop_authority,
        )
    }
}

/// `RexxRegisterSubcomExe`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`]; a non-null `entry` is a `RexxSubcomHandler`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterSubcomExe(
    name: *const c_char,
    entry: REXXPFN,
    user: *const c_char,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    unsafe { register_entry(Registry::Subcom, name, entry, user) }
}

/// `RexxQuerySubcom`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`]; a non-null `user` has 16 writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQuerySubcom(
    name: *const c_char,
    library: *const c_char,
    flags: *mut u16,
    user: *mut c_char,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    unsafe { query(Registry::Subcom, name, library, flags, user) }
}

/// `RexxDeregisterSubcom`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDeregisterSubcom(
    name: *const c_char,
    library: *const c_char,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    code(unsafe { capi::deregister(Registry::Subcom, self::name(name), text(library)) })
}

/// `RexxLoadSubcom`: loads `name`'s handler; the library is ignored.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxLoadSubcom(
    name: *const c_char,
    _library: *const c_char,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    code(capi::load_subcom(unsafe { self::name(name) }))
}

/// `RexxRegisterExitDll`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterExitDll(
    name: *const c_char,
    library: *const c_char,
    procedure: *const c_char,
    user: *const c_char,
    drop_authority: usize,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    unsafe {
        register_library(
            Registry::Exit,
            name,
            library,
            procedure,
            user,
            drop_authority,
        )
    }
}

/// `RexxRegisterExitExe`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`]; a non-null `entry` is a `RexxExitHandler`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterExitExe(
    name: *const c_char,
    entry: REXXPFN,
    user: *const c_char,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    unsafe { register_entry(Registry::Exit, name, entry, user) }
}

/// `RexxDeregisterExit`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDeregisterExit(
    name: *const c_char,
    library: *const c_char,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    code(unsafe { capi::deregister(Registry::Exit, self::name(name), text(library)) })
}

/// `RexxQueryExit`.
///
/// # Safety
/// As [`RexxQuerySubcom`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueryExit(
    name: *const c_char,
    library: *const c_char,
    flags: *mut u16,
    user: *mut c_char,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    unsafe { query(Registry::Exit, name, library, flags, user) }
}

/// `RexxRegisterFunctionDll`: always owner-only.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterFunctionDll(
    name: *const c_char,
    library: *const c_char,
    procedure: *const c_char,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    unsafe {
        register_library(
            Registry::Function,
            name,
            library,
            procedure,
            std::ptr::null(),
            capi::OWNER_ONLY,
        )
    }
}

/// `RexxRegisterFunctionExe`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`]; a non-null `entry` is a `RexxFunctionHandler`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxRegisterFunctionExe(
    name: *const c_char,
    entry: REXXPFN,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    unsafe { register_entry(Registry::Function, name, entry, std::ptr::null()) }
}

/// `RexxDeregisterFunction`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDeregisterFunction(name: *const c_char) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    code(unsafe { capi::deregister(Registry::Function, self::name(name), None) })
}

/// `RexxQueryFunction`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueryFunction(name: *const c_char) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    code(capi::query_function(unsafe { self::name(name) }))
}

/// `RexxAddMacro`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxAddMacro(
    name: *const c_char,
    file: *const c_char,
    position: usize,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee, for each pointer.
    code(unsafe { capi::add_macro(self::name(name), self::name(file), position as u64) })
}

/// `RexxDropMacro`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDropMacro(name: *const c_char) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    code(capi::drop_macro(unsafe { self::name(name) }))
}

/// `RexxSaveMacroSpace`: every macro where `names` is null.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`]; a non-null `names` addresses `count` names.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxSaveMacroSpace(
    count: usize,
    names: *const *const c_char,
    file: *const c_char,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee, for each pointer.
    code(unsafe { capi::save_macros(self::names(count, names), self::name(file)) })
}

/// `RexxLoadMacroSpace`: every macro where `names` is null.
///
/// # Safety
/// As [`RexxSaveMacroSpace`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxLoadMacroSpace(
    count: usize,
    names: *const *const c_char,
    file: *const c_char,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee, for each pointer.
    code(unsafe { capi::load_macros(self::names(count, names), self::name(file)) })
}

/// `RexxQueryMacro`: `*position` is 0 where the macro is absent.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueryMacro(name: *const c_char, position: *mut u16) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    let (status, found) = capi::query_macro(unsafe { self::name(name) });
    if !position.is_null() {
        // SAFETY: the caller's guarantee. The header's `unsigned short` truncates.
        unsafe { position.write(found as u16) };
    }
    code(status)
}

/// `RexxReorderMacro`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxReorderMacro(name: *const c_char, position: usize) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    code(capi::reorder_macro(
        unsafe { self::name(name) },
        position as u64,
    ))
}

/// `RexxClearMacroSpace`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxClearMacroSpace() -> RexxReturnCode {
    code(capi::clear_macros())
}

/// `RexxCreateQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`]; `name` has `size` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxCreateQueue(
    name: *mut c_char,
    size: usize,
    requested: *const c_char,
    duplicate: *mut usize,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    let (status, created, flag) = capi::create_queue(unsafe { text(requested) }, size);
    if let Some(created) = created
        && !name.is_null()
        && size > 0
    {
        let length = created.len().min(size - 1);
        // SAFETY: `name` holds `size` bytes, and `length + 1` is at most that.
        unsafe {
            std::ptr::copy_nonoverlapping(created.as_ptr(), name.cast::<u8>(), length);
            name.add(length).write(0);
        }
    }
    if let Some(flag) = flag
        && !duplicate.is_null()
    {
        // SAFETY: the caller's guarantee.
        unsafe { duplicate.write(usize::from(flag)) };
    }
    code(status)
}

/// `RexxOpenQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxOpenQueue(
    name: *const c_char,
    duplicate: *mut usize,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    let (status, flag) = capi::open_queue(unsafe { text(name) });
    if let Some(flag) = flag
        && !duplicate.is_null()
    {
        // SAFETY: the caller's guarantee.
        unsafe { duplicate.write(usize::from(flag)) };
    }
    code(status)
}

/// `RexxQueueExists`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueueExists(name: *const c_char) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    code(capi::queue_exists(unsafe { text(name) }))
}

/// `RexxDeleteQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxDeleteQueue(name: *const c_char) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    code(capi::delete_queue(unsafe { text(name) }))
}

/// `RexxQueryQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxQueryQueue(name: *const c_char, count: *mut usize) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    let (status, found) = capi::query_queue(unsafe { text(name) });
    if let Some(found) = found
        && !count.is_null()
    {
        // SAFETY: the caller's guarantee.
        unsafe { count.write(found as usize) };
    }
    code(status)
}

/// `RexxAddQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`]; `data` addresses a `CONSTRXSTRING` whose non-null `strptr` has
/// `strlength` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxAddQueue(
    name: *const c_char,
    data: *const CONSTRXSTRING,
    flag: usize,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    let item = unsafe { data.as_ref() }
        .filter(|data| !data.strptr.is_null())
        // SAFETY: the caller's guarantee.
        .map(|data| unsafe { std::slice::from_raw_parts(data.strptr.cast::<u8>(), data.strlength) })
        .unwrap_or_default();
    // SAFETY: the caller's guarantee.
    code(capi::add_queue(
        unsafe { text(name) },
        item.to_vec(),
        flag as u64,
    ))
}

/// `RexxPullFromQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`]; `data` addresses an initialised `RXSTRING` whose non-null
/// `strptr` has `strlength` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxPullFromQueue(
    name: *const c_char,
    data: *mut RXSTRING,
    time: *mut RexxQueueTime,
    wait: usize,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    let (status, stamp) = unsafe { pull(name, data, wait) };
    if let Some(stamp) = stamp
        && !time.is_null()
    {
        // SAFETY: the caller's guarantee.
        unsafe { time.write(queue_time(stamp)) };
    }
    status
}

/// `RexxPullQueue`: `RexxPullFromQueue` with the older time stamp, `valid` set.
///
/// # Safety
/// As [`RexxPullFromQueue`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxPullQueue(
    name: *const c_char,
    data: *mut RXSTRING,
    time: *mut REXXDATETIME,
    wait: usize,
) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    let (status, stamp) = unsafe { pull(name, data, wait) };
    if let Some(stamp) = stamp
        && !time.is_null()
    {
        let stamp = queue_time(stamp);
        let older = REXXDATETIME {
            hours: stamp.hours,
            minutes: stamp.minutes,
            seconds: stamp.seconds,
            hundredths: 0,
            day: stamp.day,
            month: stamp.month,
            year: stamp.year,
            weekday: stamp.weekday,
            microseconds: 0,
            yearday: stamp.yearday as usize,
            valid: 1,
        };
        // SAFETY: the caller's guarantee.
        unsafe { time.write(older) };
    }
    status
}

/// `RexxClearQueue`.
///
/// # Safety
/// As [`RexxRegisterSubcomDll`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxClearQueue(name: *const c_char) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    code(capi::clear_queue(unsafe { text(name) }))
}

/// `RexxCreateSessionQueue`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxCreateSessionQueue() -> RexxReturnCode {
    code(capi::create_session_queue())
}

/// `RexxDeleteSessionQueue`.
#[unsafe(no_mangle)]
pub extern "C" fn RexxDeleteSessionQueue() -> RexxReturnCode {
    code(capi::delete_session_queue())
}

/// `RexxShutDownAPI`: does nothing and answers 1 (Resolution 1).
#[unsafe(no_mangle)]
pub extern "C" fn RexxShutDownAPI() -> RexxReturnCode {
    1
}

/// `RexxAllocateMemory`: `malloc`, so `free` releases the block too.
#[unsafe(no_mangle)]
pub extern "C" fn RexxAllocateMemory(size: usize) -> *mut c_void {
    // SAFETY: `malloc` has no precondition.
    unsafe { libc::malloc(size) }
}

/// `RexxFreeMemory`: `free`, answering 0.
///
/// # Safety
/// `block` is null or a block `RexxAllocateMemory` or `malloc` returned and not yet released.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RexxFreeMemory(block: *mut c_void) -> RexxReturnCode {
    // SAFETY: the caller's guarantee.
    unsafe { libc::free(block) };
    0
}
