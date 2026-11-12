//! The `unsafe` the Rust `rxsock` needs and
//! nothing else: borrowing a socket's descriptor number for one call
//! ([`with_socket`], [`with_sockets`]) and reading or setting a socket option
//! by its raw level and number ([`get_int_option`], [`set_int_option`]), for
//! the options neither `rustix` nor `nix` names (`SO_DEBUG`, `SO_RCVLOWAT`,
//! `SO_SNDLOWAT` and the number 64 `SO_USELOOPBACK` passes on Linux). Every
//! other OS call `rxsock` makes goes through `rustix` or `nix` on the
//! borrowed descriptor.

#![allow(unsafe_code)]

use std::os::fd::{AsRawFd as _, BorrowedFd, RawFd};

/// An OS call's outcome: its value, or the `errno` it failed with.
pub(crate) type Os<T> = std::result::Result<T, i32>;

/// The calling thread's `errno`, as the last failed call left it.
fn errno() -> i32 {
    std::io::Error::last_os_error()
        .raw_os_error()
        .unwrap_or(libc::EINVAL)
}

/// `EBADF` unless `fd` names an open descriptor of this process, as
/// `fcntl(F_GETFD)` finds it; a negative number names none, and neither does
/// one of the interpreter's own (`interrexx_rxapi::reserved`).
fn check_open(fd: RawFd) -> Os<()> {
    if fd < 0 || interrexx_rxapi::reserved::is_reserved(fd) {
        return Err(libc::EBADF);
    }
    // SAFETY: `fcntl(F_GETFD)` takes no pointer and changes nothing; it reads
    // the descriptor flags of any number and fails with `EBADF` for one that
    // names no open descriptor.
    match unsafe { libc::fcntl(fd, libc::F_GETFD) } {
        -1 => Err(errno()),
        _ => Ok(()),
    }
}

/// Calls `call` with the descriptor numbered `fd` borrowed, as C code uses the
/// number: `EBADF` (without calling) where `fd` names no open descriptor.
pub(crate) fn with_socket<T>(
    fd: RawFd,
    call: impl for<'a> FnOnce(BorrowedFd<'a>) -> Os<T>,
) -> Os<T> {
    check_open(fd)?;
    // SAFETY: `borrow_raw` asks that `fd` be open, and not -1, for as long as
    // the borrow lives. `check_open` has just found it open (so it is not -1)
    // and not one of the interpreter's own descriptors, which no Rust object of
    // this process but `interrexx_rxapi::reserved`'s holders owns; the borrow
    // lives only for `call`, which its higher-ranked lifetime keeps from storing
    // it. The number can still be closed while it is borrowed: by another
    // interpreter thread's `SockClose` or `SysClose` of the same number, or by
    // code outside the interpreter. The descriptor is then the program's, owned
    // by no Rust object, so no object's descriptor is closed under it; and
    // `call` only passes the number to system calls through `rustix` and `nix`,
    // which read and write no memory through it. A closed or reused number makes
    // the call fail with `EBADF` or act on the descriptor now under it, as C
    // code does; that is a wrong answer to a racing program, not undefined
    // behaviour.
    let socket = unsafe { BorrowedFd::borrow_raw(fd) };
    call(socket)
}

/// [`with_socket`] for several descriptors at once: `call` gets them borrowed
/// in the order of `fds`, or the first invalid one's `errno` is answered.
pub(crate) fn with_sockets<T>(
    fds: &[RawFd],
    call: impl for<'a> FnOnce(&[BorrowedFd<'a>]) -> Os<T>,
) -> Os<T> {
    for fd in fds {
        check_open(*fd)?;
    }
    let sockets: Vec<BorrowedFd<'_>> = fds
        .iter()
        // SAFETY: as in `with_socket`: each number was just found open, and
        // the borrows live only for `call`, which cannot store them.
        .map(|fd| unsafe { BorrowedFd::borrow_raw(*fd) })
        .collect();
    call(&sockets)
}

/// `getsockopt(socket, level, name)` for an option whose value is a C `int`.
pub(crate) fn get_int_option(socket: BorrowedFd<'_>, level: i32, name: i32) -> Os<i32> {
    let mut value: libc::c_int = 0;
    let mut length = libc::socklen_t::try_from(std::mem::size_of::<libc::c_int>())
        .expect("an int's size fits a socklen_t");
    // SAFETY: `value` and `length` are live locals of the sizes `length`
    // states; the kernel writes at most `length` bytes into `value` and the
    // count it wrote into `length`. The descriptor is borrowed, so open.
    let result = unsafe {
        libc::getsockopt(
            socket.as_raw_fd(),
            level,
            name,
            (&raw mut value).cast(),
            &raw mut length,
        )
    };
    match result {
        -1 => Err(errno()),
        _ => Ok(value),
    }
}

/// `setsockopt(socket, level, name)` to the C `int` `value`.
pub(crate) fn set_int_option(socket: BorrowedFd<'_>, level: i32, name: i32, value: i32) -> Os<()> {
    let value: libc::c_int = value;
    let length = libc::socklen_t::try_from(std::mem::size_of::<libc::c_int>())
        .expect("an int's size fits a socklen_t");
    // SAFETY: `value` is a live local of `length` bytes, which the kernel only
    // reads. The descriptor is borrowed, so open.
    let result = unsafe {
        libc::setsockopt(
            socket.as_raw_fd(),
            level,
            name,
            (&raw const value).cast(),
            length,
        )
    };
    match result {
        -1 => Err(errno()),
        _ => Ok(()),
    }
}
