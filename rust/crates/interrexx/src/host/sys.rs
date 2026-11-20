//! The `unsafe` that [`super`] needs and nothing else: the calls a host
//! offers through no safe crate. On Apple's systems those are `setgroups`,
//! which `nix` leaves out there, and `select` and `fcntl` on a descriptor
//! number that may name no open descriptor, which no safe wrapper takes.
//! Other hosts need none of it.

#![allow(unsafe_code)]

/// The calling thread's `errno`, as the last failed call left it.
#[cfg(target_vendor = "apple")]
fn errno() -> nix::errno::Errno {
    nix::errno::Errno::last()
}

/// Drops every supplementary group of the process: `setgroups(0, NULL)`.
#[cfg(target_vendor = "apple")]
pub(super) fn setgroups_none() -> nix::Result<()> {
    // SAFETY: with a count of 0, `setgroups` reads no element of the list,
    // so the null pointer is never read. The call changes only the process's
    // credentials, which no Rust object holds.
    match unsafe { libc::setgroups(0, std::ptr::null()) } {
        0 => Ok(()),
        _ => Err(errno()),
    }
}

/// Whether `fd` names an open descriptor of this process, as
/// `fcntl(F_GETFD)` finds it.
#[cfg(target_vendor = "apple")]
pub(super) fn is_open(fd: i32) -> bool {
    // SAFETY: `fcntl(F_GETFD)` takes no pointer and changes nothing; it reads
    // the descriptor flags of any number and fails with `EBADF` for one that
    // names no open descriptor.
    unsafe { libc::fcntl(fd, libc::F_GETFD) != -1 }
}

/// Whether `select`, asked at once whether the descriptor numbered `fd` is
/// readable, answers `EBADF`: `fd` is closed and inside the process's
/// descriptor table. `select` answers `EINVAL` for an `fd` at or above
/// `FD_SETSIZE`, and that is `false`.
#[cfg(target_vendor = "apple")]
pub(super) fn select_rejects(fd: i32) -> bool {
    /// Bits in one word of an `fd_set`, as Apple's headers lay it out.
    const WORD: usize = 32;
    let Ok(index) = usize::try_from(fd) else {
        return false;
    };
    // A set of at least an `fd_set`'s size, with `fd`'s bit as the headers'
    // `FD_SET` places it.
    let words = (index / WORD + 1).max(std::mem::size_of::<libc::fd_set>() / 4);
    let mut set = vec![0u32; words];
    set[index / WORD] |= 1 << (index % WORD);
    let mut timeout = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    // SAFETY: `select` reads and writes the first `fd + 1` bits of the read
    // set, which `set` holds (it has `index / 32 + 1` words or more, and is
    // at least an `fd_set` long, aligned as its `i32` words are); the other
    // sets are null, which `select` takes as empty; `timeout` is a live
    // local, zero so the call does not wait. The descriptor's own state is
    // only read.
    let answer = unsafe {
        libc::select(
            fd.saturating_add(1),
            set.as_mut_ptr().cast::<libc::fd_set>(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &raw mut timeout,
        )
    };
    answer == -1 && errno() == nix::errno::Errno::EBADF
}
