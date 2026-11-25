//! Host socket operations and the descriptor access needed by `HANDLE:` streams.
//!
//! Descriptor validation and borrowing live alongside the socket operations
//! they support. `HANDLE:n` streams use `duplicate` to retain their own
//! descriptor on Unix. Options without a wrapper in `rustix` or `nix` use the raw
//! `getsockopt` and `setsockopt` calls here.

#![allow(unsafe_code)]

pub(crate) use rustix::event::PollFlags;
use rustix::event::{PollFd, Timespec};
#[cfg(unix)]
use rustix::fd::AsRawFd as _;
use rustix::fd::{BorrowedFd, RawFd};
use std::net::SocketAddrV4;
use std::time::Duration;

/// The host identifier a Rexx socket operation receives.
pub(crate) type SocketId = RawFd;

/// A socket and the events a wait is interested in.
pub(crate) type PollRequest = (SocketId, PollFlags);

/// An OS call's outcome: its value, or the `errno` it failed with.
pub(crate) type Os<T> = std::result::Result<T, i32>;

/// The calling thread's `errno`, as the last failed call left it.
#[cfg(unix)]
fn errno() -> i32 {
    std::io::Error::last_os_error()
        .raw_os_error()
        .unwrap_or(libc::EINVAL)
}

/// `EBADF` unless `fd` names an open descriptor of this process, as
/// `fcntl(F_GETFD)` finds it; a negative number names none, and neither does
/// one of the interpreter's own (`interrexx_rxapi::reserved`).
#[cfg(unix)]
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

/// A duplicate of descriptor `fd`, close-on-exec, which shares its file offset
/// and status flags and closes alone: what a `HANDLE:n` stream reads and
/// writes. `EBADF` where `fd` names no open descriptor or one of the
/// interpreter's reserved ones (`interrexx_rxapi::reserved`); a descriptor the
/// interpreter holds that is not reserved, such as a command's capture pipe,
/// is duplicated as any other. Safe: the borrow is [`with_socket`]'s, and the duplicate
/// is a new descriptor the returned `OwnedFd` owns.
#[cfg(unix)]
pub(crate) fn duplicate(fd: RawFd) -> Os<std::os::fd::OwnedFd> {
    with_socket(fd, |borrowed| {
        borrowed
            .try_clone_to_owned()
            .map_err(|error| error.raw_os_error().unwrap_or(libc::EBADF))
    })
}

/// [`with_socket`] for several descriptors at once: `call` gets them borrowed
/// in the order of `fds`, or the first invalid one's `errno` is answered.
fn with_sockets<T>(fds: &[RawFd], call: impl for<'a> FnOnce(&[BorrowedFd<'a>]) -> Os<T>) -> Os<T> {
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

/// Polls the requested sockets in order. A cancellation descriptor, when
/// supplied, contributes the final readiness entry.
pub(crate) fn poll(
    requests: impl Iterator<Item = PollRequest> + Clone,
    timeout: Option<Duration>,
    cancel: Option<BorrowedFd<'_>>,
) -> Os<(usize, Vec<PollFlags>)> {
    let numbers: Vec<_> = requests.clone().map(|(fd, _)| fd).collect();
    #[cfg(windows)]
    if numbers.is_empty() && cancel.is_none() {
        return Ok((0, Vec::new()));
    }
    let timeout = timeout.map(|left| Timespec {
        tv_sec: i64::try_from(left.as_secs()).unwrap_or(i64::MAX),
        tv_nsec: left
            .subsec_nanos()
            .try_into()
            .expect("nanoseconds fit in Timespec"),
    });
    with_sockets(&numbers, |sockets| {
        let mut descriptors: Vec<_> = sockets
            .iter()
            .zip(requests)
            .map(|(fd, (_, events))| PollFd::new(fd, events))
            .collect();
        if let Some(cancel) = cancel {
            descriptors.push(PollFd::from_borrowed_fd(cancel, PollFlags::IN));
        }
        let count = rustix::event::poll(&mut descriptors, timeout.as_ref())
            .map_err(rustix::io::Errno::raw_os_error)?;
        Ok((count, descriptors.iter().map(PollFd::revents).collect()))
    })
}

/// `getsockopt(socket, level, name)` for an option whose value is a C `int`.
#[cfg(unix)]
fn get_int_option(socket: BorrowedFd<'_>, level: i32, name: i32) -> Os<i32> {
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
#[cfg(unix)]
fn set_int_option(socket: BorrowedFd<'_>, level: i32, name: i32, value: i32) -> Os<()> {
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

/// The options neither `rustix` nor `nix` names, read and set by number as
/// C `int`s: `SO_DEBUG`, `SO_RCVLOWAT`, `SO_SNDLOWAT` and the number 64
/// `SO_USELOOPBACK` passes.
#[cfg(unix)]
const RAW_OPTIONS: [i32; 4] = [1, 18, 19, 64];

/// The host's number for the raw option `option` identifies. On Apple's
/// systems the option numbers are not Linux's (`SO_RCVLOWAT` is 0x1004, not
/// 18); 64 is passed as itself on every host.
#[cfg(unix)]
fn raw_option(option: i32) -> i32 {
    match option {
        1 => libc::SO_DEBUG,
        18 => libc::SO_RCVLOWAT,
        19 => libc::SO_SNDLOWAT,
        other => other,
    }
}

#[cfg(unix)]
fn timeval_millis(seconds: i64, micros: i64) -> i64 {
    seconds * 1000 + micros / 1000
}

/// Reads a RexxSock `SOL_SOCKET` option, formatting the OS value for Rexx.
#[cfg(unix)]
pub(crate) fn get_option(fd: RawFd, option: Option<i32>) -> Os<String> {
    use nix::sys::socket::{getsockopt, sockopt};

    let bool_of = |result: nix::Result<bool>| result.map(|on| i64::from(on).to_string());
    match option {
        None => Err(libc::ENOPROTOOPT),
        Some(option) if RAW_OPTIONS.contains(&option) => with_socket(fd, |socket| {
            get_int_option(socket, libc::SOL_SOCKET, raw_option(option)).map(|on| on.to_string())
        }),
        Some(option) => with_socket(fd, |socket| {
            match option {
                2 => bool_of(getsockopt(&socket, sockopt::ReuseAddr)),
                3 => {
                    return rustix::net::sockopt::socket_type(socket)
                        .map(|kind| {
                            match kind {
                                rustix::net::SocketType::STREAM => "STREAM",
                                rustix::net::SocketType::DGRAM => "DGRAM",
                                rustix::net::SocketType::RAW => "RAW",
                                _ => "UNKNOWN",
                            }
                            .to_owned()
                        })
                        .map_err(rustix::io::Errno::raw_os_error);
                }
                4 => getsockopt(&socket, sockopt::SocketError).map(|errno| errno.to_string()),
                5 => bool_of(getsockopt(&socket, sockopt::DontRoute)),
                6 => bool_of(getsockopt(&socket, sockopt::Broadcast)),
                7 => getsockopt(&socket, sockopt::SndBuf).map(|size| size.to_string()),
                8 => getsockopt(&socket, sockopt::RcvBuf).map(|size| size.to_string()),
                9 => bool_of(getsockopt(&socket, sockopt::KeepAlive)),
                10 => bool_of(getsockopt(&socket, sockopt::OobInline)),
                13 => getsockopt(&socket, sockopt::Linger)
                    .map(|linger| format!("{} {}", linger.l_onoff, linger.l_linger)),
                20 | 21 => {
                    let time = match option {
                        20 => getsockopt(&socket, sockopt::ReceiveTimeout),
                        _ => getsockopt(&socket, sockopt::SendTimeout),
                    };
                    time.map(|time| {
                        use nix::sys::time::TimeValLike as _;
                        let micros = time.num_microseconds();
                        timeval_millis(micros / 1_000_000, micros % 1_000_000).to_string()
                    })
                }
                _ => return Err(libc::ENOPROTOOPT),
            }
            .map_err(|error| error as i32)
        }),
    }
}

/// Sets a RexxSock `SOL_SOCKET` option from a parsed integer or linger pair.
#[cfg(unix)]
pub(crate) fn set_option(fd: RawFd, option: i32, numeric: i32, linger: (u16, u16)) -> Os<()> {
    use nix::sys::socket::{setsockopt, sockopt};

    let on = numeric != 0;
    with_socket(fd, |socket| {
        #[expect(
            clippy::cast_sign_loss,
            reason = "C passes the int, which the kernel reads unsigned"
        )]
        let size = numeric as u32 as usize;
        let set = match option {
            option if RAW_OPTIONS.contains(&option) => {
                return set_int_option(socket, libc::SOL_SOCKET, raw_option(option), numeric);
            }
            2 => setsockopt(&socket, sockopt::ReuseAddr, &on),
            5 => setsockopt(&socket, sockopt::DontRoute, &on),
            6 => setsockopt(&socket, sockopt::Broadcast, &on),
            7 => setsockopt(&socket, sockopt::SndBuf, &size),
            8 => setsockopt(&socket, sockopt::RcvBuf, &size),
            9 => setsockopt(&socket, sockopt::KeepAlive, &on),
            10 => setsockopt(&socket, sockopt::OobInline, &on),
            13 => {
                let (enabled, seconds) = linger;
                let linger = nix::libc::linger {
                    l_onoff: i32::from(enabled),
                    l_linger: i32::from(seconds),
                };
                setsockopt(&socket, sockopt::Linger, &linger)
            }
            20 | 21 => {
                let seconds = numeric / 1000;
                let micros = (numeric % 1000) * 1000;
                if micros < 0 {
                    return Err(libc::EDOM);
                }
                let micros = if seconds < 0 {
                    0
                } else {
                    i64::from(seconds) * 1_000_000 + i64::from(micros)
                };
                let time =
                    <nix::sys::time::TimeVal as nix::sys::time::TimeValLike>::microseconds(micros);
                match option {
                    20 => setsockopt(&socket, sockopt::ReceiveTimeout, &time),
                    _ => setsockopt(&socket, sockopt::SendTimeout, &time),
                }
            }
            _ => return Err(libc::ENOPROTOOPT),
        };
        set.map_err(|error| error as i32)
    })
}

/// Closes a socket that belongs to the program.
#[cfg(unix)]
pub(crate) fn close(fd: RawFd) -> Os<()> {
    // The interpreter's own descriptors are not the program's to close.
    if interrexx_rxapi::reserved::is_reserved(fd) {
        Err(libc::EBADF)
    } else {
        nix::unistd::close(fd).map_err(|error| error as i32)
    }
}

pub(crate) fn shutdown(fd: RawFd, how: i32) -> Os<()> {
    with_socket(fd, |socket| {
        let how = match how {
            0 => rustix::net::Shutdown::Read,
            1 => rustix::net::Shutdown::Write,
            2 => rustix::net::Shutdown::Both,
            _ => return Err(rustix::io::Errno::INVAL.raw_os_error()),
        };
        rustix::net::shutdown(socket, how).map_err(rustix::io::Errno::raw_os_error)
    })
}

pub(crate) fn listen(fd: RawFd, backlog: i32) -> Os<()> {
    with_socket(fd, |socket| {
        rustix::net::listen(socket, backlog).map_err(rustix::io::Errno::raw_os_error)
    })
}

pub(crate) fn bind(fd: RawFd, address: &SocketAddrV4) -> Os<()> {
    with_socket(fd, |socket| {
        rustix::net::bind(socket, address).map_err(rustix::io::Errno::raw_os_error)
    })
}

#[cfg(unix)]
pub(crate) fn set_nonblocking(fd: RawFd, enabled: bool) -> Os<()> {
    with_socket(fd, |socket| {
        rustix::io::ioctl_fionbio(socket, enabled).map_err(rustix::io::Errno::raw_os_error)
    })
}

pub(crate) fn available_bytes(fd: RawFd) -> Os<u64> {
    with_socket(fd, |socket| {
        rustix::io::ioctl_fionread(socket).map_err(rustix::io::Errno::raw_os_error)
    })
}

#[cfg(windows)]
#[path = "rxsock/windows.rs"]
mod windows;
#[cfg(windows)]
pub(crate) use windows::{
    blocks, close, ensure_started, get_option, hostname, publish_socket, set_nonblocking,
    set_option,
};

#[cfg(windows)]
use windows::check_open;

#[cfg(unix)]
pub(crate) use crate::host::{
    CONNECT_WAIT_EVENTS, POLL_HANGS_UP_UNCONNECTED, is_listening, no_sigpipe_flags, prepare_socket,
};
#[cfg(windows)]
pub(crate) use windows::{
    CONNECT_WAIT_EVENTS, POLL_HANGS_UP_UNCONNECTED, is_listening, no_sigpipe_flags, prepare_socket,
};
