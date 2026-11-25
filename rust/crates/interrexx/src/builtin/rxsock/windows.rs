//! Winsock-specific operations missing from the portable socket API.

use super::{Os, RawFd};
use rustix::fd::{AsRawFd as _, BorrowedFd};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use windows_sys::Win32::Networking::WinSock as wsa;

fn raw_error() -> i32 {
    // SAFETY: reads the calling thread's Winsock error, with no arguments.
    unsafe { wsa::WSAGetLastError() }
}

pub(crate) fn ensure_started() -> Os<()> {
    static START: OnceLock<Os<()>> = OnceLock::new();
    *START.get_or_init(|| rustix::net::wsa_startup().map(|_| ()).map_err(os))
}

fn os(error: rustix::io::Errno) -> i32 {
    error.raw_os_error()
}

fn socket_modes() -> &'static Mutex<HashMap<RawFd, bool>> {
    // Winsock can set FIONBIO but cannot read back the mode. Store whether
    // each socket released to Rexx is nonblocking. The pool's own sockets
    // are never in this table and cannot be borrowed or closed from Rexx.
    static MODES: OnceLock<Mutex<HashMap<RawFd, bool>>> = OnceLock::new();
    MODES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn publish_socket(fd: RawFd, nonblocking: bool) -> RawFd {
    socket_modes().lock().unwrap().insert(fd, nonblocking);
    fd
}

pub(crate) fn blocks(socket: BorrowedFd<'_>) -> bool {
    !socket_modes()
        .lock()
        .unwrap()
        .get(&socket.as_raw_fd())
        .copied()
        .unwrap_or(false)
}

pub(crate) fn set_nonblocking(fd: RawFd, enabled: bool) -> Os<()> {
    super::with_socket(fd, |socket| {
        let mut modes = socket_modes().lock().unwrap();
        let current = modes.get_mut(&fd).ok_or(wsa::WSAENOTSOCK)?;
        rustix::io::ioctl_fionbio(socket, enabled).map_err(os)?;
        *current = enabled;
        Ok(())
    })
}

pub(crate) fn close(fd: RawFd) -> Os<()> {
    ensure_started()?;
    let mut modes = socket_modes().lock().unwrap();
    if !modes.contains_key(&fd) {
        return Err(wsa::WSAENOTSOCK);
    }
    // SAFETY: closesocket accepts an arbitrary socket number and validates it.
    if unsafe { wsa::closesocket(fd as wsa::SOCKET) } != 0 {
        return Err(raw_error());
    }
    modes.remove(&fd);
    Ok(())
}

pub(super) fn check_open(fd: RawFd) -> Os<()> {
    ensure_started()?;
    if !socket_modes().lock().unwrap().contains_key(&fd) {
        return Err(wsa::WSAENOTSOCK);
    }
    get_int(fd, wsa::SO_TYPE).map(|_| ())
}

fn get_int(fd: RawFd, option: i32) -> Os<i32> {
    let mut value = 0;
    let mut len = 4;
    // SAFETY: value is a writable 4-byte integer and len is writable.
    let result = unsafe {
        wsa::getsockopt(
            fd as wsa::SOCKET,
            wsa::SOL_SOCKET,
            option,
            (&raw mut value).cast(),
            &mut len,
        )
    };
    if result == 0 {
        Ok(value)
    } else {
        Err(raw_error())
    }
}

fn option_name(option: i32) -> Os<i32> {
    match option {
        1 => Ok(wsa::SO_DEBUG),
        2 => Ok(wsa::SO_REUSEADDR),
        3 => Ok(wsa::SO_TYPE),
        4 => Ok(wsa::SO_ERROR),
        5 => Ok(wsa::SO_DONTROUTE),
        6 => Ok(wsa::SO_BROADCAST),
        7 => Ok(wsa::SO_SNDBUF),
        8 => Ok(wsa::SO_RCVBUF),
        9 => Ok(wsa::SO_KEEPALIVE),
        10 => Ok(wsa::SO_OOBINLINE),
        13 => Ok(wsa::SO_LINGER),
        18 => Ok(wsa::SO_RCVLOWAT),
        19 => Ok(wsa::SO_SNDLOWAT),
        20 => Ok(wsa::SO_RCVTIMEO),
        21 => Ok(wsa::SO_SNDTIMEO),
        64 => Ok(wsa::SO_USELOOPBACK),
        _ => Err(wsa::WSAENOPROTOOPT),
    }
}

pub(crate) fn get_option(fd: RawFd, option: Option<i32>) -> Os<String> {
    let option = option_name(option.ok_or(wsa::WSAENOPROTOOPT)?)?;
    check_open(fd)?;
    if option == wsa::SO_LINGER {
        let mut linger = wsa::LINGER::default();
        let mut len = 4;
        // SAFETY: linger is a writable pair of 16-bit fields and len is writable.
        let result = unsafe {
            wsa::getsockopt(
                fd as wsa::SOCKET,
                wsa::SOL_SOCKET,
                option,
                (&raw mut linger).cast(),
                &mut len,
            )
        };
        return if result == 0 {
            Ok(format!("{} {}", linger.l_onoff, linger.l_linger))
        } else {
            Err(raw_error())
        };
    }
    let answer = get_int(fd, option)?;
    if option == wsa::SO_TYPE {
        let name = match answer {
            wsa::SOCK_STREAM => "STREAM",
            wsa::SOCK_DGRAM => "DGRAM",
            wsa::SOCK_RAW => "RAW",
            _ => "UNKNOWN",
        };
        return Ok(name.to_owned());
    }
    Ok(answer.to_string())
}

pub(crate) fn set_option(fd: RawFd, option: i32, numeric: i32, linger: (u16, u16)) -> Os<()> {
    let option = option_name(option)?;
    check_open(fd)?;
    let linger = wsa::LINGER {
        l_onoff: linger.0,
        l_linger: linger.1,
    };
    let value = if option == wsa::SO_LINGER {
        (&raw const linger).cast()
    } else {
        (&raw const numeric).cast()
    };
    // SAFETY: both possible values occupy 4 bytes and live across the call.
    let result = unsafe { wsa::setsockopt(fd as wsa::SOCKET, wsa::SOL_SOCKET, option, value, 4) };
    if result == 0 {
        Ok(())
    } else {
        Err(raw_error())
    }
}

pub(crate) fn hostname() -> Os<Vec<u8>> {
    ensure_started()?;
    let mut name = [0; 256];
    // SAFETY: the buffer is writable for 256 bytes.
    let outcome = unsafe { wsa::gethostname(name.as_mut_ptr(), 256) };
    if outcome != 0 {
        return Err(raw_error());
    }
    Ok(name.into_iter().take_while(|byte| *byte != 0).collect())
}

pub(crate) const CONNECT_WAIT_EVENTS: super::PollFlags =
    super::PollFlags::OUT.union(super::PollFlags::IN);
pub(crate) const POLL_HANGS_UP_UNCONNECTED: bool = false;

pub(crate) fn is_listening(socket: BorrowedFd<'_>) -> bool {
    get_int(socket.as_raw_fd(), wsa::SO_ACCEPTCONN).is_ok_and(|on| on != 0)
}

pub(crate) fn no_sigpipe_flags(_: BorrowedFd<'_>) -> rustix::net::SendFlags {
    rustix::net::SendFlags::empty()
}

pub(crate) fn prepare_socket(socket: BorrowedFd<'_>) -> rustix::io::Result<()> {
    use windows_sys::Win32::Foundation::{HANDLE_FLAG_INHERIT, SetHandleInformation};
    // SAFETY: the socket is a live borrowed kernel handle.
    let ok = unsafe { SetHandleInformation(socket.as_raw_fd() as _, HANDLE_FLAG_INHERIT, 0) };
    if ok == 0 {
        Err(rustix::io::Errno::INVAL)
    } else {
        Ok(())
    }
}
