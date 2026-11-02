//! `.RexxQueue`'s body (`classes/RexxQueueMethods.cpp`): each instance method reads the queue
//! name from the receiver's `NAMED_QUEUE`, and each class method takes it as its argument.

use std::time::Duration;

use rexx_core::ObjRef;
use rexx_rxapi::message::code::{
    NO_SERVICE, OK, QUEUE_ACCESS, QUEUE_BAD_NAME, QUEUE_BAD_WAIT_FLAG, QUEUE_EMPTY,
    QUEUE_NOT_REGISTERED, QUEUE_PRIORITY,
};

use super::{Cleared, NativeStarted};
use crate::error::Raised;
use crate::scheduler::ParkReason;
use crate::{Failure, Interp};

/// How often a waiting `lineIn` looks for its line.
pub(crate) const WAIT_POLL: Duration = Duration::from_millis(5);

/// `RXAPI_MEMFAIL`.
const MEMORY_FAILURE: u64 = 1002;

/// The receiver's `NAMED_QUEUE` of `scope`, or 98.901 where `INIT` never set it.
fn queue_name(interp: &mut Interp, receiver: ObjRef, scope: ObjRef) -> Result<Vec<u8>, Failure> {
    let owner = interp.pool_owner(receiver)?;
    let named = interp
        .pools_of(owner)
        .and_then(|pools| pools.get(scope, b"NAMED_QUEUE"));
    match named {
        Some(named) => interp.string_value_text(named),
        None => {
            let shown = interp.string_value_text(receiver)?;
            Err(Raised::syntax(98, 901, vec![shown]).into())
        }
    }
}

/// A `CSTRING` argument: 88.901 where it is missing.
fn name_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Vec<u8>, Failure> {
    match args.get(index).copied().flatten() {
        Some(value) => interp.string_value_text(value),
        None => Err(Raised::missing_native_argument(index + 1).into()),
    }
}

fn ran(value: ObjRef) -> Result<NativeStarted, Failure> {
    Ok(NativeStarted::Ran(Some(value)))
}

fn code(interp: &mut Interp, status: u64) -> Result<NativeStarted, Failure> {
    ran(interp.counted(usize::try_from(status).unwrap_or(usize::MAX)))
}

/// `rexx_create_queue([name])`: the created queue's name, or the null string.
pub(super) fn create(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let requested = match args.first().copied().flatten() {
        Some(value) => Some(interp.string_value_text(value)?),
        None => None,
    };
    let name = interp
        .queue_create(requested.as_deref())
        .unwrap_or_default();
    ran(interp.text(&name))
}

/// `rexx_open_queue(name)`.
pub(super) fn open(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let name = name_argument(interp, args, 0)?;
    let status = interp.queue_open(&name);
    code(interp, status)
}

/// `rexx_queue_exists(name)`: whether the queue exists.
pub(super) fn exists(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let name = name_argument(interp, args, 0)?;
    let exists = interp.queue_exists(&name) == OK;
    code(interp, u64::from(exists))
}

/// `rexx_delete_queue(name)`.
pub(super) fn delete(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let name = name_argument(interp, args, 0)?;
    let status = interp.queue_delete(&name);
    code(interp, status)
}

/// `rexx_query_queue()`: the line count, 0 on any failure.
pub(super) fn query(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    scope: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let name = queue_name(interp, receiver, scope)?;
    let count = interp.queue_count(&name);
    code(interp, count)
}

/// `rexx_clear_queue()`.
pub(super) fn clear(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    scope: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let name = queue_name(interp, receiver, scope)?;
    let status = interp.queue_clear(&name);
    code(interp, status)
}

/// `rexx_push_queue([line])`.
pub(super) fn push(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    add(interp, receiver, scope, args, true)
}

/// `rexx_queue_queue([line])`.
pub(super) fn queue(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    add(interp, receiver, scope, args, false)
}

/// `rexx_add_queue`: 0, or 48.1 naming the failure.
fn add(
    interp: &mut Interp,
    receiver: ObjRef,
    scope: ObjRef,
    args: &[Option<ObjRef>],
    lifo: bool,
) -> Result<NativeStarted, Failure> {
    let line = match args.first().copied().flatten() {
        Some(value) => interp.string_value_text(value)?,
        None => Vec::new(),
    };
    let name = queue_name(interp, receiver, scope)?;
    match interp.queue_add(&name, line, lifo) {
        OK => code(interp, OK),
        status => Err(queue_failure(status).into()),
    }
}

/// 48.1 for an add that failed with `status`.
pub(crate) fn queue_failure(status: u64) -> Raised {
    let reason = match status {
        NO_SERVICE => Some("RXAPI_NORXAPI"),
        MEMORY_FAILURE => Some("RXAPI_MEMFAIL"),
        QUEUE_BAD_NAME => Some("RXQUEUE_BADQNAME"),
        QUEUE_PRIORITY => Some("RXQUEUE_PRIORITY"),
        QUEUE_BAD_WAIT_FLAG => Some("RXQUEUE_BADWAITFLAG"),
        QUEUE_EMPTY => Some("RXQUEUE_EMPTY"),
        QUEUE_NOT_REGISTERED => Some("RXQUEUE_NOTREG"),
        QUEUE_ACCESS => Some("RXQUEUE_ACCESS"),
        _ => None,
    };
    let text = match reason {
        Some(reason) => format!("SYSTEM QUEUE ({reason})"),
        None => format!("SYSTEM QUEUE (reason code {status})"),
    };
    Raised::syntax(48, 1, vec![text.into_bytes()])
}

/// `rexx_pull_queue()`: the head line, or `.nil` on any failure, without waiting.
pub(super) fn pull(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    scope: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let name = queue_name(interp, receiver, scope)?;
    match interp.queue_pull(&name) {
        Ok(line) => ran(interp.text(&line)),
        Err(_) => ran(ObjRef::NIL),
    }
}

/// `rexx_linein_queue()`: the head line, waiting for one, or `.nil` on any failure. The wait
/// gives up the guard lock, and a halt ends it.
pub(super) fn linein(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    scope: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let name = queue_name(interp, receiver, scope)?;
    match interp.queue_pull(&name) {
        Ok(line) => return ran(interp.text(&line)),
        Err(QUEUE_EMPTY) => {}
        Err(_) => return ran(ObjRef::NIL),
    }
    if let Some(key) = interp.activity.scoped_guard.take() {
        interp.release_guard(key);
    }
    interp.begin_queue_wait(name);
    let deadline = interp.now() + WAIT_POLL;
    interp.park_native(ParkReason::Sleep { deadline }, linein_woken, receiver)
}

/// A waiting `lineIn`'s answer once its activity wakes with its line, or from a halt.
fn linein_woken(interp: &mut Interp, _receiver: ObjRef) -> Result<Option<ObjRef>, Failure> {
    interp.activity.asleep = None;
    let halted = std::mem::take(&mut interp.activity.woken_by_halt);
    Ok(Some(match interp.end_queue_wait(halted) {
        Some(line) => interp.text(&line),
        None => ObjRef::NIL,
    }))
}
