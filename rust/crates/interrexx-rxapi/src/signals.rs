//! Stopping a daemon on `SIGTERM`, `SIGINT` or `SIGHUP`.
// The `unsafe` calls are the C library's signal-mask and wait functions, which no safe crate of
// the workspace offers.
#![allow(unsafe_code)]

use std::io;

/// The signals that stop a daemon.
const STOPPING: [libc::c_int; 3] = [libc::SIGTERM, libc::SIGINT, libc::SIGHUP];

/// Makes the first of the stopping signals call `stop`, on a thread of its own, whatever the
/// signals' disposition and mask were when this process started: a client that starts the daemon
/// may hold them blocked or ignored, and both survive an `exec`.
///
/// The signals are blocked in the calling thread, and so in every thread it starts afterwards,
/// which is why only one thread waits for them. Call it while the process has one thread.
pub(crate) fn stop_on_signal(stop: impl FnOnce() + Send + 'static) -> io::Result<()> {
    // SAFETY: the set is initialised by `sigemptyset` before it is added to, through a pointer
    // to a live local; `signal` with `SIG_DFL` installs no code of ours; the mask pointers are
    // to live values or null.
    let set = unsafe {
        let mut set: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&raw mut set);
        for signal in STOPPING {
            libc::sigaddset(&raw mut set, signal);
            libc::signal(signal, libc::SIG_DFL);
        }
        let failed = libc::pthread_sigmask(libc::SIG_BLOCK, &raw const set, std::ptr::null_mut());
        if failed != 0 {
            return Err(io::Error::from_raw_os_error(failed));
        }
        set
    };
    std::thread::Builder::new()
        .name("rxapi-signal".to_owned())
        .spawn(move || {
            let mut received: libc::c_int = 0;
            // SAFETY: `set` is an initialised set and `received` a live local.
            if unsafe { libc::sigwait(&raw const set, &raw mut received) } == 0 {
                stop();
            }
        })?;
    Ok(())
}
