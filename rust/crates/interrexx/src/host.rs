//! What differs from one host system to the next. Each concern is one
//! function with an implementation per host, chosen by `cfg`, so a port adds
//! an implementation beside the others and no caller changes. The raw calls
//! some hosts need, where no safe crate offers them, are in [`sys`].

use std::os::fd::{BorrowedFd, OwnedFd};
use std::process::Child;

use rustix::net::SendFlags;

mod sys;

/// The flags of every send on a socket: they keep it from raising `SIGPIPE`
/// where the send can say so. Elsewhere [`prepare_socket`] sets that on the
/// socket instead, and a socket the interpreter did not make relies on
/// `SIGPIPE` being ignored, as every interpreter start makes it.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) const SEND_FLAGS: SendFlags = SendFlags::NOSIGNAL;

/// The flags of every send on a socket: they keep it from raising `SIGPIPE`
/// where the send can say so. Elsewhere [`prepare_socket`] sets that on the
/// socket instead, and a socket the interpreter did not make relies on
/// `SIGPIPE` being ignored, as every interpreter start makes it.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub(crate) const SEND_FLAGS: SendFlags = SendFlags::empty();

/// Readies a socket the interpreter has just made or accepted: where the
/// send cannot say so, its writes answer `EPIPE` instead of raising
/// `SIGPIPE`.
#[cfg(any(target_os = "linux", target_os = "android"))]
#[expect(
    clippy::unnecessary_wraps,
    reason = "the socket option can fail on other hosts"
)]
pub(crate) fn prepare_socket(_socket: BorrowedFd<'_>) -> rustix::io::Result<()> {
    Ok(())
}

/// Readies a socket the interpreter has just made or accepted: where the
/// send cannot say so, its writes answer `EPIPE` instead of raising
/// `SIGPIPE`.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub(crate) fn prepare_socket(socket: BorrowedFd<'_>) -> rustix::io::Result<()> {
    rustix::net::sockopt::set_socket_nosigpipe(socket, true)
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

/// Whether `poll` reports a stream socket with no peer as hung up. Linux's
/// does; Apple's does not, and other hosts are not relied on to.
pub(crate) const POLL_HANGS_UP_UNCONNECTED: bool =
    cfg!(any(target_os = "linux", target_os = "android"));

/// Whether a socket `accept` answers takes the listening socket's
/// `O_NONBLOCK`. Linux's starts without it, whatever the listening socket's
/// mode; Apple's systems and the BSDs copy it.
pub(crate) const ACCEPT_INHERITS_NONBLOCK: bool =
    !cfg!(any(target_os = "linux", target_os = "android"));

/// The process's descriptor table as `select` sees it, taken once for a
/// call: `select` looks at an open descriptor always, and at a closed one
/// where it lies inside the table, which makes `select` answer `EBADF`. One
/// past the table, and a negative number, is never ready and no error.
pub(crate) struct SelectTable {
    /// The table's size, `FDSize` in `/proc/self/status`.
    #[cfg(any(target_os = "linux", target_os = "android"))]
    size: i32,
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl SelectTable {
    /// The table as it is now.
    pub(crate) fn now() -> SelectTable {
        let size = std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|status| {
                status
                    .lines()
                    .find_map(|line| line.strip_prefix("FDSize:"))
                    .and_then(|size| size.trim().parse().ok())
            })
            .unwrap_or(i32::MAX);
        SelectTable { size }
    }

    /// Whether `select` looks at the descriptor numbered `fd`.
    pub(crate) fn reaches(&self, fd: i32) -> bool {
        (0..self.size).contains(&fd)
    }
}

/// Apple's kernel does not say how large its table has grown, so a closed
/// descriptor is put to `select` itself; one at or above `FD_SETSIZE`, which
/// that `select` refuses with `EINVAL`, is taken to be past the table.
#[cfg(target_vendor = "apple")]
impl SelectTable {
    /// The table as it is now.
    pub(crate) fn now() -> SelectTable {
        SelectTable {}
    }

    /// Whether `select` looks at the descriptor numbered `fd`.
    #[expect(
        clippy::unused_self,
        reason = "the table is read when it is asked, not when it is taken"
    )]
    pub(crate) fn reaches(&self, fd: i32) -> bool {
        fd >= 0 && (sys::is_open(fd) || sys::select_rejects(fd))
    }
}

/// Hosts with no means to learn their table's size take every number that
/// is not negative to be inside it.
#[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
impl SelectTable {
    /// The table as it is now.
    pub(crate) fn now() -> SelectTable {
        SelectTable {}
    }

    /// Whether `select` looks at the descriptor numbered `fd`.
    #[expect(
        clippy::unused_self,
        reason = "the table is read when it is asked, not when it is taken"
    )]
    pub(crate) fn reaches(&self, fd: i32) -> bool {
        fd >= 0
    }
}

/// How many bytes a read of `file` finds at once, where the host counts them
/// in a way to trust; `None` where it does not, and the caller then asks a
/// `poll`.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) fn bytes_waiting(file: BorrowedFd<'_>) -> Option<u64> {
    rustix::io::ioctl_fionread(file).ok()
}

/// How many bytes a read of `file` finds at once, where the host counts them
/// in a way to trust; `None` where it does not, and the caller then asks a
/// `poll`. Apple's `FIONREAD` on a FIFO open for both reading and writing
/// counts 0 with bytes waiting, so a 0 there is not trusted.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub(crate) fn bytes_waiting(file: BorrowedFd<'_>) -> Option<u64> {
    let count = rustix::io::ioctl_fionread(file).ok()?;
    let fifo = || {
        rustix::fs::fstat(file).is_ok_and(|stat| {
            rustix::fs::FileType::from_raw_mode(stat.st_mode) == rustix::fs::FileType::Fifo
        })
    };
    (count > 0 || !fifo()).then_some(count)
}

/// Serialises the creation of the interpreter's own descriptors with the
/// start of child processes, on hosts that cannot create a descriptor
/// close-on-exec in the same call: a child started in between would inherit
/// it. Creating takes the lock shared, starting a child exclusive.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
static INHERITANCE: std::sync::RwLock<()> = std::sync::RwLock::new(());

/// Runs `create`, which makes descriptors and sets them close-on-exec, so
/// that no child [`spawning`] starts meanwhile inherits them.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
fn creating<T>(create: impl FnOnce() -> T) -> T {
    let _shared = INHERITANCE
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    create()
}

/// Runs `spawn`, which starts a child process, while no descriptor of the
/// interpreter's own is half made: one the child would inherit before it
/// is set close-on-exec.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) fn spawning<T>(spawn: impl FnOnce() -> T) -> T {
    spawn()
}

/// Runs `spawn`, which starts a child process, while no descriptor of the
/// interpreter's own is half made: one the child would inherit before it
/// is set close-on-exec.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub(crate) fn spawning<T>(spawn: impl FnOnce() -> T) -> T {
    let _exclusive = INHERITANCE
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    spawn()
}

/// A pipe whose two ends are close-on-exec, as `(read, write)`; no child
/// process inherits either. There is no `pipe2` here, so the flag is set
/// apart from the call, while [`spawning`] waits. Linux's waits use an
/// `eventfd` and a pidfd, made close-on-exec in their one call, and need no
/// pipe.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub(crate) fn pipe_cloexec() -> rustix::io::Result<(OwnedFd, OwnedFd)> {
    creating(|| {
        let (read, write) = rustix::pipe::pipe()?;
        for end in [&read, &write] {
            rustix::io::fcntl_setfd(end, rustix::io::FdFlags::CLOEXEC)?;
        }
        Ok((read, write))
    })
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

/// Drops the process's supplementary groups. `nix` has no `setgroups` for
/// Apple's systems, so the call is [`sys`]'s.
#[cfg(target_vendor = "apple")]
pub(crate) fn drop_supplementary_groups() -> nix::Result<()> {
    sys::setgroups_none()
}

/// Drops the process's supplementary groups.
#[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
pub(crate) fn drop_supplementary_groups() -> nix::Result<()> {
    nix::unistd::setgroups(&[])
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
    let queue = creating(|| {
        let queue = Kqueue::new().ok()?;
        rustix::io::fcntl_setfd(&queue, rustix::io::FdFlags::CLOEXEC).ok()?;
        Some(queue)
    })?;
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
            let (read, write) = pipe_cloexec().ok()?;
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

#[cfg(test)]
mod tests {
    use std::os::fd::{AsFd as _, AsRawFd as _};

    use super::{SelectTable, bytes_waiting};

    /// A descriptor just closed lies inside the table, which grew to hold it,
    /// so `select` looks at it (and answers `EBADF`); a negative one never.
    #[test]
    fn select_looks_at_a_descriptor_just_closed() {
        let (read, write) = rustix::pipe::pipe().expect("a pipe");
        let highest = read.as_raw_fd().max(write.as_raw_fd());
        let table = SelectTable::now();
        assert!(table.reaches(read.as_raw_fd()));
        drop((read, write));
        let table = SelectTable::now();
        assert!(table.reaches(highest), "closed descriptor {highest}");
        assert!(!table.reaches(-1));
    }

    /// A count of 0 is trusted where the host's count is, so a terminal's end
    /// of file is never read to learn there is no data: on Linux for every
    /// device, and on Apple's systems for all but a FIFO, which a pipe is
    /// there. A count above 0 is trusted everywhere.
    #[test]
    fn an_empty_count_is_trusted_but_for_an_apple_fifo() {
        let (read, write) = rustix::pipe::pipe().expect("a pipe");
        let empty_pipe = if cfg!(any(target_os = "linux", target_os = "android")) {
            Some(0)
        } else {
            None
        };
        assert_eq!(bytes_waiting(read.as_fd()), empty_pipe);
        rustix::io::write(&write, b"abc").expect("a write");
        assert_eq!(bytes_waiting(read.as_fd()), Some(3));

        let (socket, _peer) = std::os::unix::net::UnixStream::pair().expect("a socket pair");
        assert_eq!(bytes_waiting(socket.as_fd()), Some(0));
    }

    /// Apple's `FIONREAD` on a FIFO open for reading and writing counts 0
    /// with bytes waiting, so an empty count there is not trusted.
    #[cfg(target_vendor = "apple")]
    #[test]
    fn a_fifo_count_of_zero_is_not_trusted_on_apple() {
        use std::hash::BuildHasher as _;
        let unique = std::hash::RandomState::new().hash_one(std::process::id());
        let path = std::env::temp_dir().join(format!("host-fifo-{unique:016x}"));
        // Neither `rustix` nor this crate's `nix` features make a FIFO on Apple's systems.
        let made = std::process::Command::new("mkfifo")
            .arg(&path)
            .status()
            .expect("mkfifo runs");
        assert!(made.success(), "a FIFO");
        let fifo = rustix::fs::open(&path, rustix::fs::OFlags::RDWR, rustix::fs::Mode::empty())
            .expect("the FIFO, open both ways");
        let _ = std::fs::remove_file(&path);
        assert_eq!(bytes_waiting(fifo.as_fd()), None);
    }
}
