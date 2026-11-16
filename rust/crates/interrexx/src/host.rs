//! What differs from one host system to the next. Each concern is one
//! function with an implementation per host, chosen by `cfg`, so a port adds
//! an implementation beside the others and no caller changes.

use std::os::fd::{BorrowedFd, OwnedFd};
use std::process::Child;

use rustix::net::SendFlags;

/// The flags that keep a send on `socket` from raising `SIGPIPE`, where the
/// send can say so; elsewhere the socket is set to answer `EPIPE` instead, and
/// no flag is needed.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) fn send_without_sigpipe(_socket: BorrowedFd<'_>) -> SendFlags {
    SendFlags::NOSIGNAL
}

/// The flags that keep a send on `socket` from raising `SIGPIPE`, where the
/// send can say so; elsewhere the socket is set to answer `EPIPE` instead, and
/// no flag is needed. A socket that refuses the option is sent on as it is.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub(crate) fn send_without_sigpipe(socket: BorrowedFd<'_>) -> SendFlags {
    let _ = rustix::net::sockopt::set_socket_nosigpipe(socket, true);
    SendFlags::empty()
}

/// Whether `socket` is listening for connections; `false` where that cannot be
/// read.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) fn is_listening(socket: BorrowedFd<'_>) -> bool {
    rustix::net::sockopt::socket_acceptconn(socket).unwrap_or(false)
}

/// Whether `socket` is listening for connections; `false` where that cannot be
/// read.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub(crate) fn is_listening(socket: BorrowedFd<'_>) -> bool {
    nix::sys::socket::getsockopt(&socket, nix::sys::socket::sockopt::AcceptConn).unwrap_or(false)
}

/// `bits` as the host's `mode_t`, cut to its width as C's conversion cuts it.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) fn raw_mode(bits: u32) -> rustix::fs::RawMode {
    bits
}

/// `bits` as the host's `mode_t`, cut to its width as C's conversion cuts it.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub(crate) fn raw_mode(bits: u32) -> rustix::fs::RawMode {
    let [low, high, _, _] = bits.to_le_bytes();
    rustix::fs::RawMode::from_le_bytes([low, high])
}

/// A `mode_t`, as 32 bits whatever the host's width.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) fn mode_bits(raw: rustix::fs::RawMode) -> u32 {
    raw
}

/// A `mode_t`, as 32 bits whatever the host's width.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub(crate) fn mode_bits(raw: rustix::fs::RawMode) -> u32 {
    u32::from(raw)
}

/// `bits` as the mode argument of `access`, its bits kept as C's conversion
/// keeps them.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) fn access_mode(bits: u32) -> rustix::fs::Access {
    rustix::fs::Access::from_bits_retain(bits)
}

/// `bits` as the mode argument of `access`, its bits kept as C's conversion
/// keeps them.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub(crate) fn access_mode(bits: u32) -> rustix::fs::Access {
    rustix::fs::Access::from_bits_retain(i32::from_ne_bytes(bits.to_ne_bytes()))
}

/// Drops the process's supplementary groups.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) fn drop_supplementary_groups() -> nix::Result<()> {
    nix::unistd::setgroups(&[])
}

/// Drops the process's supplementary groups. No safe call does so here, so
/// this answers `ENOSYS` and the caller keeps its privileges as they are.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub(crate) fn drop_supplementary_groups() -> nix::Result<()> {
    Err(nix::errno::Errno::ENOSYS)
}

/// A descriptor that becomes readable once `child` has ended, and stays
/// readable; it reaps nothing. `None` where the host cannot say, and the
/// caller then waits for the child's pipes to close instead.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) fn exit_watch(child: &Child) -> Option<OwnedFd> {
    let pid = rustix::process::Pid::from_raw(i32::try_from(child.id()).ok()?)?;
    rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).ok()
}

/// A descriptor that becomes readable once `child` has ended, and stays
/// readable; it reaps nothing. A `kqueue` holding the child's exit event,
/// which stays pending since nothing collects it. A child that has already
/// ended cannot be registered, and is answered by a pipe that holds a byte.
#[cfg(any(
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly"
))]
pub(crate) fn exit_watch(child: &Child) -> Option<OwnedFd> {
    use nix::errno::Errno;
    use nix::sys::event::{EvFlags, EventFilter, FilterFlag, KEvent, Kqueue};
    let queue = Kqueue::new().ok()?;
    rustix::io::fcntl_setfd(&queue, rustix::io::FdFlags::CLOEXEC).ok()?;
    let exit = KEvent::new(
        usize::try_from(child.id()).ok()?,
        EventFilter::EVFILT_PROC,
        EvFlags::EV_ADD,
        FilterFlag::NOTE_EXIT,
        0,
        0,
    );
    match queue.kevent(&[exit], &mut [], None) {
        Ok(_) => Some(queue.into()),
        Err(Errno::ESRCH) => {
            let (read, write) = rustix::pipe::pipe().ok()?;
            rustix::io::fcntl_setfd(&read, rustix::io::FdFlags::CLOEXEC).ok()?;
            rustix::io::write(&write, &[1]).ok()?;
            Some(read)
        }
        Err(_) => None,
    }
}

/// Hosts with no way to watch a process: the caller reads the pipes to their end.
#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly"
)))]
pub(crate) fn exit_watch(_child: &Child) -> Option<OwnedFd> {
    None
}
