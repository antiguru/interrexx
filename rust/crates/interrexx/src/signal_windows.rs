//! Windows console interrupts and cancellation of a thread's blocking I/O.
#![allow(unsafe_code)]

use interrexx_cast::CastFrom;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, Ordering},
};
use windows_sys::Win32::System::{
    Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler},
    IO::CancelSynchronousIo,
    Threading::{GetCurrentThreadId, OpenThread, THREAD_TERMINATE},
};

static WAKE: OnceLock<&'static crate::sync::Wake> = OnceLock::new();
static PENDING: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn console_event(event: u32) -> i32 {
    if !matches!(event, CTRL_C_EVENT | CTRL_BREAK_EVENT) {
        return 0;
    }
    PENDING.store(true, Ordering::Release);
    if let Some(wake) = WAKE.get() {
        wake.notify();
    }
    1
}

pub(crate) fn install(wake: &'static crate::sync::Wake) {
    if WAKE.set(wake).is_ok() {
        // SAFETY: the process-lifetime callback uses a process-lifetime wake.
        unsafe { SetConsoleCtrlHandler(Some(console_event), 1) };
    }
}

pub(crate) fn take_pending() -> bool {
    PENDING.swap(false, Ordering::AcqRel)
}

pub(crate) fn current_thread() -> usize {
    // SAFETY: no caller preconditions.
    unsafe { usize::cast_from(GetCurrentThreadId()) }
}

pub(crate) fn flush_c_streams() {
    // SAFETY: a null argument flushes all C output streams.
    unsafe { libc::fflush(std::ptr::null_mut()) };
}

// Windows console callbacks run on a separate thread, without POSIX signal masks.
pub(crate) fn block() {}
pub(crate) fn receive() {}

pub(crate) fn unblocked<R>(wait: impl FnOnce() -> R) -> R {
    wait()
}

pub(crate) struct Interrupter(Option<OwnedHandle>);

impl Interrupter {
    pub(crate) fn here() -> Self {
        // SAFETY: this thread is live; a non-null handle belongs to the caller.
        let handle = unsafe { OpenThread(THREAD_TERMINATE, 0, GetCurrentThreadId()) };
        Self((!handle.is_null()).then(|| {
            // SAFETY: OpenThread returned a unique owned handle.
            unsafe { OwnedHandle::from_raw_handle(handle) }
        }))
    }

    pub(crate) fn interrupt(&self) {
        if let Some(handle) = &self.0 {
            // SAFETY: this owned handle grants cancellation access to the thread.
            unsafe { CancelSynchronousIo(handle.as_raw_handle()) };
        }
    }
}
