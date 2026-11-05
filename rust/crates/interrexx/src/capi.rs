//! The classic C API's queue, registration and macrospace operations, over this process's one
//! API connection, and the entry-point (`...Exe`) registrations, which stay in the process.
//!
//! A `None` name is a NULL pointer at the C API. Statuses are the `rexxapidefs.h` values.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use interrexx_api::load::ClassicEntry;
use interrexx_rxapi::message::code::{DUPLICATE, MACRO_FILE_ERROR, NO_SERVICE, OK};
use interrexx_rxapi::message::{Item, Reply, Request};

pub use interrexx_rxapi::message::{Registry, Timestamp};

/// `RXSUBCOM_NOTREG`, also `RXFUNC_NOTREG`.
pub const NOT_REGISTERED: u64 = interrexx_rxapi::message::code::NOT_REGISTERED;
/// `RXAPI_MEMFAIL`.
pub const MEMORY_FAILURE: u64 = 1002;
/// `RXSUBCOM_BADENTRY`: a null entry point.
pub const BAD_ENTRY: u64 = 1001;

use crate::builtin::rexxutil::macros;
use crate::rxapi_client;

/// `RXSUBCOM_NONDROP`: only the registering process may drop.
pub const OWNER_ONLY: usize = 1;

/// How many macrospace and registration changes this module has made, which an interpreter of
/// this process compares when a native call ends, so that a call site does not keep a
/// resolution made before one ([`crate::Interp::sync_api_changes`]).
static CHANGES: AtomicU64 = AtomicU64::new(0);

/// [`CHANGES`] now.
pub(crate) fn changes() -> u64 {
    CHANGES.load(Ordering::Acquire)
}

/// Answers `status`, counting a change that may move what a name resolves to.
fn changed(status: u64) -> u64 {
    CHANGES.fetch_add(1, Ordering::AcqRel);
    status
}

/// An entry-point registration.
struct Registered {
    registry: Registry,
    name: Vec<u8>,
    entry: ClassicEntry,
    user_area: [u8; 16],
}

/// This process's entry-point registrations, in registration order.
static ENTRY_POINTS: Mutex<Vec<Registered>> = Mutex::new(Vec::new());

fn entry_points() -> MutexGuard<'static, Vec<Registered>> {
    ENTRY_POINTS
        .lock()
        .expect("the registrations' critical sections cannot panic")
}

fn find(entries: &[Registered], registry: Registry, name: &[u8]) -> Option<usize> {
    entries
        .iter()
        .position(|entry| entry.registry == registry && entry.name.eq_ignore_ascii_case(name))
}

/// `request`'s reply, connecting and starting the service on first use.
fn call(request: Request) -> Result<Reply, u64> {
    rxapi_client::client()?.call(request)
}

fn status_of(reply: &Result<Reply, u64>) -> u64 {
    match reply {
        Ok(
            Reply::Status(status)
            | Reply::QueueCreated { status, .. }
            | Reply::QueueOpened { status, .. }
            | Reply::QueueCount { status, .. }
            | Reply::Pulled { status, .. }
            | Reply::Registration { status, .. }
            | Reply::MacroPosition { status, .. }
            | Reply::MacroImage { status, .. }
            | Reply::Macros { status, .. },
        ) => *status,
        Ok(Reply::Cancelled) => NO_SERVICE,
        Err(status) => *status,
    }
}

fn session() -> u64 {
    rxapi_client::session_handle()
}

/// Where `name` is the session queue, has an interpreter whose native call this thread runs
/// move the session queue it still holds in process to the service first, as it does before a
/// command, so that this operation and the program see one queue.
fn attach_session(name: Option<&[u8]>) {
    if name.is_none_or(|name| name.eq_ignore_ascii_case(b"SESSION")) {
        interrexx_api::ffi::classic::share_session_queue();
    }
}

fn owned(name: Option<&[u8]>) -> Option<Vec<u8>> {
    name.map(<[u8]>::to_vec)
}

/// `RexxCreateSessionQueue`: connects and attaches this process's session queue.
pub fn create_session_queue() -> u64 {
    match rxapi_client::client() {
        Ok(_) => OK,
        Err(status) => status,
    }
}

/// `RexxDeleteSessionQueue`: leaves the queue and the connection alone.
pub fn delete_session_queue() -> u64 {
    OK
}

/// `RexxCreateQueue`: the status, the name to hand back and the duplicate flag, each `None` where
/// the caller's output stays untouched.
pub fn create_queue(requested: Option<&[u8]>, size: usize) -> (u64, Option<Vec<u8>>, Option<bool>) {
    let reply = call(Request::CreateQueue {
        session: session(),
        requested: owned(requested),
        size: size as u64,
    });
    match reply {
        Ok(Reply::QueueCreated {
            status,
            name,
            duplicate,
        }) => (status, name, duplicate),
        reply => (status_of(&reply), None, None),
    }
}

/// `RexxOpenQueue`: the status and the duplicate flag.
pub fn open_queue(name: Option<&[u8]>) -> (u64, Option<bool>) {
    match call(Request::OpenQueue { name: owned(name) }) {
        Ok(Reply::QueueOpened { status, duplicate }) => (status, duplicate),
        reply => (status_of(&reply), None),
    }
}

/// `RexxQueueExists`.
pub fn queue_exists(name: Option<&[u8]>) -> u64 {
    status_of(&call(Request::QueueExists { name: owned(name) }))
}

/// `RexxDeleteQueue`.
pub fn delete_queue(name: Option<&[u8]>) -> u64 {
    status_of(&call(Request::DeleteQueue { name: owned(name) }))
}

/// `RexxClearQueue`.
pub fn clear_queue(name: Option<&[u8]>) -> u64 {
    attach_session(name);
    status_of(&call(Request::ClearQueue {
        session: session(),
        name: owned(name),
    }))
}

/// `RexxQueryQueue`: the status and the count.
pub fn query_queue(name: Option<&[u8]>) -> (u64, Option<u64>) {
    attach_session(name);
    match call(Request::QueryQueue {
        session: session(),
        name: owned(name),
    }) {
        Ok(Reply::QueueCount { status, count }) => (status, count),
        reply => (status_of(&reply), None),
    }
}

/// `RexxAddQueue`.
pub fn add_queue(name: Option<&[u8]>, data: Vec<u8>, flag: u64) -> u64 {
    attach_session(name);
    status_of(&call(Request::AddQueue {
        session: session(),
        name: owned(name),
        data,
        flag,
        time: Default::default(),
    }))
}

/// `RexxPullFromQueue`: the status and the item.
pub fn pull_queue(name: Option<&[u8]>, wait: u64) -> (u64, Option<Item>) {
    attach_session(name);
    match call(Request::PullQueue {
        session: session(),
        name: owned(name),
        wait,
    }) {
        Ok(Reply::Pulled { status, item }) => (status, item),
        reply => (status_of(&reply), None),
    }
}

/// `RexxRegister*Dll`. The function registry is owner-only whatever `drop_authority` says.
pub fn register_library(
    registry: Registry,
    name: &[u8],
    library: &[u8],
    procedure: &[u8],
    user_area: Option<[u8; 16]>,
    drop_authority: usize,
) -> u64 {
    changed(status_of(&call(Request::Register {
        registry,
        name: name.to_vec(),
        library: library.to_vec(),
        procedure: procedure.to_vec(),
        user_area,
        owner_only: drop_authority == OWNER_ONLY,
    })))
}

/// `RexxRegister*Exe`: `RXSUBCOM_DUP` where this process already has `name`, the first entry
/// staying.
pub fn register_entry(
    registry: Registry,
    name: &[u8],
    entry: ClassicEntry,
    user_area: Option<[u8; 16]>,
) -> u64 {
    let mut entries = entry_points();
    if find(&entries, registry, name).is_some() {
        return DUPLICATE;
    }
    entries.push(Registered {
        registry,
        name: name.to_vec(),
        entry,
        user_area: user_area.unwrap_or_default(),
    });
    changed(OK)
}

/// The entry point this process registered as `name`.
pub fn registered_entry(registry: Registry, name: &[u8]) -> Option<ClassicEntry> {
    let entries = entry_points();
    find(&entries, registry, name).map(|at| entries[at].entry)
}

/// `RexxQuery*`: the status and the user area. A `None` library looks at this process's entry
/// points first; an empty one goes to the service by name alone.
pub fn query_registration(
    registry: Registry,
    name: &[u8],
    library: Option<&[u8]>,
) -> (u64, Option<[u8; 16]>) {
    if library.is_none() {
        let entries = entry_points();
        if let Some(at) = find(&entries, registry, name) {
            return (OK, Some(entries[at].user_area));
        }
    }
    match call(Request::QueryRegistration {
        registry,
        name: name.to_vec(),
        library: library.unwrap_or_default().to_vec(),
    }) {
        Ok(Reply::Registration { status, registered }) => {
            (status, registered.map(|registered| registered.user_area))
        }
        reply => (status_of(&reply), None),
    }
}

/// `RexxDeregister*`. A `None` library removes this process's entry point first.
pub fn deregister(registry: Registry, name: &[u8], library: Option<&[u8]>) -> u64 {
    if library.is_none() {
        let mut entries = entry_points();
        if let Some(at) = find(&entries, registry, name) {
            entries.remove(at);
            return changed(OK);
        }
    }
    changed(status_of(&call(Request::Deregister {
        registry,
        name: name.to_vec(),
        library: library.unwrap_or_default().to_vec(),
    })))
}

/// The handler `name` resolves to in `registry`, as `RexxResolveSubcom` and its siblings find
/// it: this process's entry point, else the registration's library loaded in this process and
/// its procedure found, first as registered and then uppercased, with `search` tried before the
/// loader's own path. `Ok(None)` for a registration naming no library; `RXSUBCOM_NOTREG` where
/// nothing is registered or the library or procedure is not found. `start` lets the lookup start
/// a service; without it, no running service means nothing is registered.
pub fn resolve(
    registry: Registry,
    name: &[u8],
    search: &[std::path::PathBuf],
    start: bool,
) -> Result<Option<ClassicEntry>, u64> {
    if let Some(entry) = registered_entry(registry, name) {
        return Ok(Some(entry));
    }
    let client = match start {
        true => rxapi_client::client()?,
        false => rxapi_client::running_client().ok_or(NOT_REGISTERED)?,
    };
    let reply = client.call(Request::QueryRegistration {
        registry,
        name: name.to_vec(),
        library: Vec::new(),
    });
    let registered = match reply {
        Ok(Reply::Registration {
            status: OK,
            registered: Some(registered),
        }) => registered,
        reply => {
            return Err(match status_of(&reply) {
                OK => NOT_REGISTERED,
                status => status,
            });
        }
    };
    if registered.library.is_empty() {
        return Ok(None);
    }
    interrexx_api::load::classic_procedure(&registered.library, &registered.procedure, search)
        .map(Some)
        .ok_or(NOT_REGISTERED)
}

/// `RexxLoadSubcom`: whether `name`'s handler resolves. The library argument is ignored.
pub fn load_subcom(name: &[u8]) -> u64 {
    match resolve(Registry::Subcom, name, &[], true) {
        Ok(_) => OK,
        Err(status) => status,
    }
}

/// `RexxQueryFunction`: an entry point of this process's, else a registration.
pub fn query_function(name: &[u8]) -> u64 {
    query_registration(Registry::Function, name, None).0
}

/// `RexxAddMacro`: `file` translated in this process, relative to its current directory.
pub fn add_macro(name: &[u8], file: &[u8], position: u64) -> u64 {
    let Some(image) = macros::translate(file, name) else {
        return macros::SOURCE_NOT_FOUND;
    };
    changed(status_of(&call(Request::AddMacro {
        name: name.to_vec(),
        image,
        position,
    })))
}

/// `RexxDropMacro`.
pub fn drop_macro(name: &[u8]) -> u64 {
    changed(status_of(&call(Request::DropMacro {
        name: name.to_vec(),
    })))
}

/// `RexxClearMacroSpace`.
pub fn clear_macros() -> u64 {
    changed(status_of(&call(Request::ClearMacroSpace)))
}

/// `RexxQueryMacro`: the status and the position, 0 where absent.
pub fn query_macro(name: &[u8]) -> (u64, u64) {
    match call(Request::QueryMacro {
        name: name.to_vec(),
    }) {
        Ok(Reply::MacroPosition { status, position }) => (status, position),
        reply => (status_of(&reply), 0),
    }
}

/// `RexxReorderMacro`.
pub fn reorder_macro(name: &[u8], position: u64) -> u64 {
    changed(status_of(&call(Request::ReorderMacro {
        name: name.to_vec(),
        position,
    })))
}

/// `RexxSaveMacroSpace`: the macros `names` lists, or every one, written to `file`. A failure
/// leaves no file.
pub fn save_macros(names: Option<Vec<Vec<u8>>>, file: &[u8]) -> u64 {
    match call(Request::SaveMacros { names }) {
        Ok(Reply::Macros {
            status: OK,
            macros: saved,
        }) => macros::write_file(file, &interrexx_rxapi::macro_file::write(&saved)),
        reply => status_of(&reply),
    }
}

/// `RexxLoadMacroSpace`: the macros `names` lists, or every one, from `file`. Macros read before a
/// failure are loaded.
pub fn load_macros(names: Option<Vec<Vec<u8>>>, file: &[u8]) -> u64 {
    let Ok(bytes) = std::fs::read(crate::paths::os_path(file)) else {
        return MACRO_FILE_ERROR;
    };
    let (status, read) = interrexx_rxapi::macro_file::read(&bytes);
    if read.is_empty() {
        return status;
    }
    let loaded = status_of(&call(Request::LoadMacros {
        names,
        macros: read,
    }));
    changed(if status == OK { loaded } else { status })
}
