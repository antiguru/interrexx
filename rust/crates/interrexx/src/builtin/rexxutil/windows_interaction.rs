//! Windows RexxUtil routines that interact with the console or desktop.

use interrexx_core::ObjRef;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use windows_sys::Win32::{
    Foundation::{ERROR_INVALID_PARAMETER, ERROR_NOT_ENOUGH_MEMORY, ERROR_OPERATION_ABORTED},
    UI::WindowsAndMessaging::{
        MB_ABORTRETRYIGNORE, MB_ICONASTERISK, MB_ICONEXCLAMATION, MB_ICONHAND, MB_ICONQUESTION,
        MB_OK, MB_OKCANCEL, MB_RETRYCANCEL, MB_YESNO, MB_YESNOCANCEL,
    },
};

use super::{answer, c_string, check_arity, int_argument, optional_text, required_c_string};
use crate::error::Raised;
use crate::scheduler::ParkReason;
use crate::{Failure, Interp, host};

enum Reply {
    Status(u32),
    Key(host::ConsoleKey),
}

pub(crate) struct Pending {
    reply: Arc<Mutex<Option<Reply>>>,
    stopped: Arc<AtomicBool>,
    on_failure: fn(u32) -> Reply,
}

impl Drop for Pending {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
    }
}

pub(crate) fn waiting(interp: &Interp) -> bool {
    // Message~halt queues HALT for the target activity. Stop waiting at the
    // next poll so the activity reaches its clause boundary and the worker
    // sees the cancellation before the interpreter processes that HALT.
    if interp.running_activation().is_some_and(|activation| {
        !interp.trap_for(b"HALT").is_some_and(|trap| trap.delayed)
            && interp.activity.pending_traps.iter().any(|pending| {
                pending.request
                    && pending.activation == activation.id
                    && pending.fragment_depth == interp.activity.fragment_depth
            })
    }) {
        return false;
    }
    interp
        .activity
        .windows_util_wait
        .as_ref()
        .is_some_and(|pending| {
            pending
                .reply
                .lock()
                .expect("Windows utility reply lock")
                .is_none()
        })
}

pub(crate) fn next_poll(interp: &Interp) -> ParkReason {
    ParkReason::Sleep {
        deadline: interp.now() + Duration::from_millis(12),
    }
}

fn finish(interp: &mut Interp, reply: Reply) -> ObjRef {
    match reply {
        Reply::Status(status) => answer(interp, i64::from(status)),
        Reply::Key((byte, echo)) => {
            if !echo.is_empty() {
                interp.write_out(&echo);
            }
            interp.text_built(byte.into_iter().collect())
        }
    }
}

fn resume(interp: &mut Interp, _: ObjRef) -> Result<Option<ObjRef>, Failure> {
    interp.activity.asleep = None;
    let halted = std::mem::take(&mut interp.activity.woken_by_halt);
    let Some(pending) = interp.activity.windows_util_wait.take() else {
        return Ok(None);
    };
    let result = if halted {
        None
    } else {
        pending
            .reply
            .lock()
            .expect("Windows utility reply lock")
            .take()
    };
    let reply = result.unwrap_or_else(|| (pending.on_failure)(ERROR_OPERATION_ABORTED));
    Ok(Some(finish(interp, reply)))
}

fn wait(
    interp: &mut Interp,
    on_failure: fn(u32) -> Reply,
    job: impl FnOnce(&AtomicBool) -> Reply + Send + 'static,
) -> Result<ObjRef, Failure> {
    let Some(worker) = interp.pool.reserve()? else {
        return Ok(finish(interp, on_failure(ERROR_NOT_ENOUGH_MEMORY)));
    };
    let reply = Arc::new(Mutex::new(None));
    let stopped = Arc::new(AtomicBool::new(false));
    interp.activity.windows_util_wait = Some(Pending {
        reply: Arc::clone(&reply),
        stopped: Arc::clone(&stopped),
        on_failure,
    });
    worker.run(Box::new(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| job(&stopped)))
            .unwrap_or_else(|_| on_failure(ERROR_OPERATION_ABORTED));
        *reply.lock().expect("Windows utility reply lock") = Some(result);
    }));
    let parked = interp.park_rust_routine(next_poll(interp), resume, ObjRef::NIL);
    if interp.activity.native_park.is_none() {
        interp.activity.windows_util_wait = None;
    }
    parked
}

pub(crate) fn get_key(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let option = optional_text(interp, args, 1)?;
    let option = option.as_deref().map_or(b"ECHO".as_slice(), c_string);
    let echo = if option.eq_ignore_ascii_case(b"ECHO") {
        true
    } else if option.eq_ignore_ascii_case(b"NOECHO") {
        false
    } else {
        return Err(Raised::invalid_routine_option(
            "SysGetKey",
            "echo",
            "'ECHO' or 'NOECHO'",
            option,
        )
        .into());
    };
    if !interp.input_is_process_stdin() {
        return Ok(interp.text_built(Vec::new()));
    }
    interp.hand_over_output();
    wait(
        interp,
        |_| Reply::Key((None, Vec::new())),
        move |stopped| {
            let (byte, text) = host::console_key(stopped);
            Reply::Key((byte, if echo { text } else { Vec::new() }))
        },
    )
}

pub(crate) fn wait_named_pipe(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let name = required_c_string(interp, args, 1)?;
    let Some(name) = wide(&name) else {
        return Ok(answer(interp, i64::from(ERROR_INVALID_PARAMETER)));
    };
    let timeout = match args.get(1).copied().flatten() {
        Some(timeout) => int_argument(interp, timeout, 2)?,
        None => 0,
    };
    if timeout < -1 {
        return Ok(answer(interp, i64::from(ERROR_INVALID_PARAMETER)));
    }
    wait(interp, Reply::Status, move |stopped| {
        Reply::Status(host::wait_named_pipe(&name, timeout, stopped))
    })
}

pub(crate) fn message_box(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 4)?;
    let text = required_c_string(interp, args, 1)?;
    let title = optional_text(interp, args, 2)?;
    let buttons = optional_text(interp, args, 3)?;
    let icon = optional_text(interp, args, 4)?;
    let buttons = buttons.as_deref().map_or(b"OK".as_slice(), c_string);
    let buttons = match buttons.to_ascii_uppercase().as_slice() {
        b"OK" => MB_OK,
        b"OKCANCEL" => MB_OKCANCEL,
        b"RETRYCANCEL" => MB_RETRYCANCEL,
        b"ABORTRETRYIGNORE" => MB_ABORTRETRYIGNORE,
        b"YESNO" => MB_YESNO,
        b"YESNOCANCEL" => MB_YESNOCANCEL,
        _ => {
            return Err(Raised::invalid_routine_option(
                "RxMessageBox",
                "button",
                "OK, OKCANCEL, RETRYCANCEL, ABORTRETRYIGNORE, YESNO or YESNOCANCEL",
                buttons,
            )
            .into());
        }
    };
    let icon = icon.as_deref().map_or(b"NONE".as_slice(), c_string);
    let icon = match icon.to_ascii_uppercase().as_slice() {
        b"NONE" => 0,
        b"HAND" | b"STOP" | b"ERROR" => MB_ICONHAND,
        b"QUESTION" | b"QUERY" => MB_ICONQUESTION,
        b"EXCLAMATION" | b"WARNING" => MB_ICONEXCLAMATION,
        b"ASTERISK" | b"INFORMATION" => MB_ICONASTERISK,
        _ => {
            return Err(Raised::invalid_routine_option(
                "RxMessageBox", "icon", "NONE, HAND, QUESTION, EXCLAMATION, ASTERISK, INFORMATION, STOP, QUERY, WARNING or ERROR", icon,
            ).into());
        }
    };
    let title = title.as_deref().map_or(b"Error!".as_slice(), c_string);
    let (Some(text), Some(title)) = (wide(&text), wide(title)) else {
        return Ok(answer(interp, 0));
    };
    wait(
        interp,
        |_| Reply::Status(0),
        move |stopped| Reply::Status(host::message_box(&text, &title, buttons | icon, stopped)),
    )
}

pub(crate) fn switch_session(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let title = required_c_string(interp, args, 1)?;
    let status = wide(&title).map_or(ERROR_INVALID_PARAMETER, |title| host::switch_window(&title));
    Ok(answer(interp, i64::from(status)))
}

fn wide(bytes: &[u8]) -> Option<Vec<u16>> {
    Some(
        host::legacy_windows_text(bytes)?
            .encode_utf16()
            .chain([0])
            .collect(),
    )
}
