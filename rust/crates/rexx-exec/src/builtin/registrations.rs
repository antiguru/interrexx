//! `RXFUNCADD`, `RXFUNCQUERY` and `RXFUNCDROP` (`package/PackageManager.cpp`'s
//! `addRegisteredRoutine`, `queryRegisteredRoutine` and `dropRegisteredRoutine`).
//!
//! A function a package library exports, or an internal package's, is callable without the
//! registry. Any other library is a classic one: its function is registered with the service,
//! and resolves to its `RexxFunctionHandler`.

use rexx_core::ObjRef;
use rexx_rxapi::message::code::OK;
use rexx_rxapi::message::{Registry, Reply, Request};

use super::{Args, arg, required_string};
use crate::Interp;
use crate::LibraryLoad;
use crate::capi;
use crate::error::Failure;
use crate::internal_routines::{self, Package};
use crate::rxapi_client;

fn boolean(interp: &mut Interp, failed: bool) -> ObjRef {
    interp.text(if failed { b"1" } else { b"0" })
}

impl Interp {
    /// Whether a function `upper` names is loaded in this interpreter: an internal package's or
    /// a loaded library's.
    fn function_loaded(&self, upper: &[u8]) -> bool {
        internal_routines::lookup(upper).is_some_and(|row| row.body.is_some())
            || self.package_routine(upper).is_some()
    }
}

/// `RXFUNCADD(name, module [, procedure])`: 0 where the function is callable, 1 otherwise.
pub(crate) fn rxfuncadd(
    interp: &mut Interp,
    _name: &[u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let upper = required_string(interp, args, 1).to_ascii_uppercase();
    let module = required_string(interp, args, 2);
    let procedure = match arg(args, 3) {
        Some(value) => interp.to_text(value).into_owned(),
        None => upper.clone(),
    };
    if interp.function_loaded(&upper) || interp.classic_functions.contains_key(&upper) {
        return Ok(boolean(interp, false));
    }
    if let Some(package) = Package::named(&module) {
        let found = internal_routines::package_routine(package, &upper).is_some();
        return Ok(boolean(interp, !found));
    }
    match interp.resolve_library(&module) {
        LibraryLoad::Loaded(_) => {
            let loaded = interp.function_loaded(&upper);
            return Ok(boolean(interp, !loaded));
        }
        LibraryLoad::Raised(failure) => return Err(failure),
        LibraryLoad::Missing | LibraryLoad::Version => {}
    }
    register_function(&upper, &module, &procedure);
    let resolved = interp.resolve_classic_function(&upper);
    interp.routines_changed();
    Ok(boolean(interp, !resolved))
}

/// Registers `name` as `library`'s `procedure` with the service, whatever the outcome.
pub(crate) fn register_function(name: &[u8], library: &[u8], procedure: &[u8]) {
    if let Ok(client) = rxapi_client::client() {
        let _ = client.call(Request::Register {
            registry: Registry::Function,
            name: name.to_ascii_uppercase(),
            library: library.to_vec(),
            procedure: procedure.to_vec(),
            user_area: None,
            owner_only: true,
        });
    }
}

/// `RXFUNCQUERY(name)`: 0 where the function is loaded here or registered, 1 otherwise.
pub(crate) fn rxfuncquery(
    interp: &mut Interp,
    _name: &[u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let upper = required_string(interp, args, 1).to_ascii_uppercase();
    if interp.function_loaded(&upper)
        || interp.classic_functions.contains_key(&upper)
        || capi::registered_entry(Registry::Function, &upper).is_some()
    {
        return Ok(boolean(interp, false));
    }
    let status = registry_status(Request::QueryRegistration {
        registry: Registry::Function,
        name: upper,
        library: Vec::new(),
    });
    Ok(boolean(interp, status != OK))
}

/// `RXFUNCDROP(name)`: 0 where the registration was removed, 1 otherwise.
pub(crate) fn rxfuncdrop(
    interp: &mut Interp,
    _name: &[u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let upper = required_string(interp, args, 1).to_ascii_uppercase();
    interp.classic_functions.remove(&upper);
    interp.routines_changed();
    if capi::registered_entry(Registry::Function, &upper).is_some() {
        let status = capi::deregister(Registry::Function, &upper, None);
        return Ok(boolean(interp, status != OK));
    }
    let status = registry_status(Request::Deregister {
        registry: Registry::Function,
        name: upper,
        library: Vec::new(),
    });
    Ok(boolean(interp, status != OK))
}

/// `request`'s status from a running service. With none, nothing is registered.
fn registry_status(request: Request) -> u64 {
    let Some(client) = rxapi_client::running_client() else {
        return rexx_rxapi::message::code::NOT_REGISTERED;
    };
    match client.call(request) {
        Ok(Reply::Status(status) | Reply::Registration { status, .. }) => status,
        Ok(_) => OK,
        Err(status) => status,
    }
}
