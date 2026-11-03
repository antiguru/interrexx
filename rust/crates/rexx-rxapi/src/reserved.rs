//! Descriptors the interpreter owns for its own use, kept out of a program's reach.
//!
//! A program's `SockClose(n)` or `SysClose(n)` names a descriptor by number, and the numbers
//! the process hands out first are the lowest free ones. So that a program closing small
//! numbers cannot close the interpreter's own descriptors (the timer's wake socket the signal
//! handler writes to, its deadline timer, the rxapi connection, a running command's pidfd),
//! each is moved at creation to the lowest free number at or above [`floor`] and recorded
//! here; `rxsock` and `rxunixsys` answer `EBADF` for a recorded number, as for a closed one.
//!
//! The floor is 1000, lowered to 24 below the `RLIMIT_NOFILE` soft limit where that is
//! smaller, so the interpreter's descriptors sit above the numbers a program normally reaches
//! and below the limit. Where no number at or above it is free, the descriptor keeps its number
//! and is still recorded.

use std::ops::Deref;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd, RawFd};
use std::sync::{Mutex, PoisonError};

/// The highest floor; see the module documentation.
pub const FLOOR: RawFd = 1000;

/// The numbers recorded, in no order.
static RESERVED: Mutex<Vec<RawFd>> = Mutex::new(Vec::new());

fn reserved() -> std::sync::MutexGuard<'static, Vec<RawFd>> {
    RESERVED.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The number the interpreter's descriptors are moved to or above.
pub fn floor() -> RawFd {
    let soft = rustix::process::getrlimit(rustix::process::Resource::Nofile)
        .current
        .and_then(|soft| RawFd::try_from(soft).ok())
        .unwrap_or(RawFd::MAX);
    FLOOR.min(soft.saturating_sub(24)).max(3)
}

/// Whether `fd` is one of the interpreter's own descriptors.
pub fn is_reserved(fd: RawFd) -> bool {
    reserved().contains(&fd)
}

/// An interpreter-owned descriptor, recorded while it is open.
pub struct Reserved<T: AsRawFd> {
    inner: Option<T>,
    fd: RawFd,
}

/// Moves `object`'s descriptor to [`floor`] or above, close-on-exec, and records it.
pub fn reserve<T>(object: T) -> Reserved<T>
where
    T: AsRawFd + From<OwnedFd> + Into<OwnedFd>,
{
    let owned: OwnedFd = object.into();
    let owned = match rustix::io::fcntl_dupfd_cloexec(&owned, floor()) {
        Ok(lifted) => lifted,
        Err(_) => owned,
    };
    let fd = owned.as_raw_fd();
    reserved().push(fd);
    Reserved {
        inner: Some(T::from(owned)),
        fd,
    }
}

impl<T: AsRawFd> Deref for Reserved<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.as_ref().expect("held until drop")
    }
}

impl<T: AsRawFd + AsFd> AsFd for Reserved<T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        (**self).as_fd()
    }
}

impl<T: AsRawFd> AsRawFd for Reserved<T> {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl<T: AsRawFd> Drop for Reserved<T> {
    /// Closes the descriptor, then forgets its number: the other order would let a program
    /// close it in between.
    fn drop(&mut self) {
        drop(self.inner.take());
        let mut numbers = reserved();
        if let Some(at) = numbers.iter().position(|held| *held == self.fd) {
            numbers.swap_remove(at);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reserved_descriptor_sits_at_the_floor_and_is_forgotten_when_closed() {
        let (one, _other) = std::os::unix::net::UnixStream::pair().expect("a pair");
        let low = one.as_raw_fd();
        let held = reserve(one);
        let fd = held.as_raw_fd();
        assert!(fd >= floor(), "{fd} below {}", floor());
        assert!(is_reserved(fd));
        assert!(!is_reserved(low));
        drop(held);
        assert!(!is_reserved(fd));
    }
}
