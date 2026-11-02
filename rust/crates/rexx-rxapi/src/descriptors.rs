//! Closing the descriptors a started daemon inherited from its starter (ruled 2026-11-02).
// The Phase 10 grant: one function, at daemon start-up.
#![allow(unsafe_code)]

use std::os::fd::{AsRawFd, BorrowedFd, RawFd};

use rustix::fs::{Dir, Mode, OFlags};

/// Closes every descriptor of this process above standard error except `keep`, so a daemon a
/// client started holds none of that client's pipes or sockets open. Call it while the process
/// has one thread and no descriptor besides `keep` and the standard three that anything owns.
pub(crate) fn close_inherited(keep: &[BorrowedFd<'_>]) {
    let Ok(listing) = rustix::fs::open(
        "/proc/self/fd",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    ) else {
        return;
    };
    // The listing names its own descriptor, which it closes itself.
    let mut skip: Vec<RawFd> = keep.iter().map(AsRawFd::as_raw_fd).collect();
    skip.push(listing.as_raw_fd());
    let Ok(dir) = Dir::new(listing) else {
        return;
    };
    // Listed in full, and the listing closed, before any is closed.
    let open: Vec<RawFd> = dir
        .filter_map(|entry| entry.ok()?.file_name().to_str().ok()?.parse().ok())
        .collect();
    for fd in open.into_iter().filter(|fd| *fd > 2 && !skip.contains(fd)) {
        // SAFETY: `fd` was open at the listing, and the caller guarantees that no object of
        // this process owns it, it being neither a standard stream nor in `keep` nor the
        // listing's own, and that no other thread can close it or open another descriptor
        // under its number before this close. So it is still open and closing it invalidates
        // nothing any code will use.
        unsafe { rustix::io::close(fd) };
    }
}
