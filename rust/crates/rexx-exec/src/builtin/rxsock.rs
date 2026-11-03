//! The `rxsock` extension library, run in Rust: the IPv4 socket routines
//! `docs/superpowers/specs/extensions-behaviour.md` ("rxsock") describes.
//!
//! A socket is its descriptor number, as on the oracle. Each call borrows the
//! number for the call ([`rxsock_sys::with_socket`]) and makes its OS calls
//! through `rustix` or `nix` on the borrow; the options neither names go
//! through [`rxsock_sys`]'s raw `getsockopt` and `setsockopt`. Most routines set
//! `errno` and `h_errno` in the caller's variables ([`report`]); the `errno`
//! is also kept on the activity for `SockSock_Errno`.
//!
//! `SockAccept`, `SockConnect`, `SockRecv`, `SockRecvFrom` and `SockSelect`
//! wait without holding up the other activities: a call that would block
//! parks its activity on a short sleep whose re-test polls the sockets
//! ([`wait_again`]), and the OS call is made once they are ready.

mod hosts;

use std::net::{Ipv4Addr, SocketAddrV4};
use std::os::fd::{BorrowedFd, IntoRawFd as _, OwnedFd};
use std::time::{Duration, Instant};

use rexx_api::callbacks::Surface as _;
use rexx_core::{Body, Decoded, ObjRef};
use rexx_num::{DIGITS64, Number};
use rustix::event::{PollFd, PollFlags};
use rustix::net::{RecvFlags, SendFlags};

use super::rxmath::{check_arity, double_argument, required};
use super::rxsock_sys::{self, Os, with_socket, with_sockets};
use super::rxunixsys::{c_string, failed, int, number};
use crate::scheduler::ParkReason;
use crate::{Failure, Interp};

type Result<T> = std::result::Result<T, Failure>;

const EINTR: i32 = 4;
const EAGAIN: i32 = 11;
const EBADF: i32 = 9;
const EINVAL: i32 = 22;
const EDOM: i32 = 33;
const ENOPROTOOPT: i32 = 92;
const EAFNOSUPPORT: i32 = 97;
const EINPROGRESS: i32 = 115;

/// How often a waiting call polls its sockets.
const WAIT_POLL: Duration = Duration::from_millis(5);

/// The `errno` values the variable names symbolically; any other is set in
/// decimal.
const ERRNO_NAMES: &[(i32, &str)] = &[
    (11, "EWOULDBLOCK"),
    (115, "EINPROGRESS"),
    (114, "EALREADY"),
    (88, "ENOTSOCK"),
    (89, "EDESTADDRREQ"),
    (90, "EMSGSIZE"),
    (91, "EPROTOTYPE"),
    (92, "ENOPROTOOPT"),
    (93, "EPROTONOSUPPORT"),
    (94, "ESOCKTNOSUPPORT"),
    (95, "EOPNOTSUPP"),
    (96, "EPFNOSUPPORT"),
    (97, "EAFNOSUPPORT"),
    (98, "EADDRINUSE"),
    (99, "EADDRNOTAVAIL"),
    (100, "ENETDOWN"),
    (101, "ENETUNREACH"),
    (102, "ENETRESET"),
    (103, "ECONNABORTED"),
    (104, "ECONNRESET"),
    (105, "ENOBUFS"),
    (106, "EISCONN"),
    (107, "ENOTCONN"),
    (108, "ESHUTDOWN"),
    (109, "ETOOMANYREFS"),
    (110, "ETIMEDOUT"),
    (111, "ECONNREFUSED"),
    (40, "ELOOP"),
    (36, "ENAMETOOLONG"),
    (112, "EHOSTDOWN"),
    (113, "EHOSTUNREACH"),
    (39, "ENOTEMPTY"),
];

/// The text the `errno` variable gets for `errno`.
fn errno_text(errno: i32) -> Vec<u8> {
    match ERRNO_NAMES.iter().find(|(value, _)| *value == errno) {
        Some((_, name)) => name.as_bytes().to_vec(),
        None => errno.to_string().into_bytes(),
    }
}

fn os(error: rustix::io::Errno) -> i32 {
    error.raw_os_error()
}

fn nix_os(error: nix::errno::Errno) -> i32 {
    error as i32
}

/// Sets the caller's `errno` to `errno`'s text and `h_errno` to 0, and keeps
/// a failure's `errno` for `SockSock_Errno`; a success leaves that as it
/// was, as C's `errno` is left.
fn report(interp: &mut Interp, errno: i32) {
    if errno != 0 {
        interp.activity.rxsock_errno = errno;
    }
    let text = interp.text_built(errno_text(errno));
    interp.set_context_variable(b"ERRNO", text);
    let zero = number(interp, 0);
    interp.set_context_variable(b"H_ERRNO", zero);
}

/// A call's answer: its value with `errno` 0, or -1 with its `errno`.
fn answer(interp: &mut Interp, result: Os<i64>) -> ObjRef {
    let value = match result {
        Ok(value) => {
            report(interp, 0);
            value
        }
        Err(errno) => {
            report(interp, errno);
            -1
        }
    };
    number(interp, value)
}

/// [`answer`] for a call answering nothing on success: 0.
fn code(interp: &mut Interp, result: Os<()>) -> ObjRef {
    answer(interp, result.map(|()| 0))
}

/// Whether the socket blocks: its descriptor is not in non-blocking mode.
fn blocks(fd: BorrowedFd<'_>) -> bool {
    rustix::fs::fcntl_getfl(fd).is_ok_and(|flags| !flags.contains(rustix::fs::OFlags::NONBLOCK))
}

/// The string value of a data argument, every byte of it.
fn data(interp: &mut Interp, args: &[Option<ObjRef>], position: usize) -> Result<Vec<u8>> {
    let value = required(args, position)?;
    let Some(text) = interp.blamed_string_conversion(value)? else {
        return Err(crate::error::Raised::native_argument_needs_a_string_value(position).into());
    };
    Ok(interp.to_text(text).into_owned())
}

/// An optional string argument, as [`c_string`] reads one.
fn optional_string(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Option<Vec<u8>>> {
    match args.get(position - 1).copied().flatten() {
        Some(_) => c_string(interp, args, position).map(Some),
        None => Ok(None),
    }
}

/// `text` with leading and trailing blanks removed.
fn trimmed(text: &[u8]) -> &[u8] {
    let start = text
        .iter()
        .position(|byte| *byte != b' ')
        .unwrap_or(text.len());
    let end = text
        .iter()
        .rposition(|byte| *byte != b' ')
        .map_or(start, |at| at + 1);
    &text[start..end]
}

/// C's `atoi`: leading white space, a sign and the decimal digits after it,
/// 0 where there are none; wrapped to 32 bits as C's `int` overflow is.
fn atoi(text: &[u8]) -> i32 {
    let mut rest = text;
    while let Some((first, tail)) = rest.split_first()
        && first.is_ascii_whitespace()
    {
        rest = tail;
    }
    let negative = rest.first() == Some(&b'-');
    if matches!(rest.first(), Some(b'+' | b'-')) {
        rest = &rest[1..];
    }
    let mut value: i64 = 0;
    for digit in rest.iter().take_while(|byte| byte.is_ascii_digit()) {
        value = value.wrapping_mul(10).wrapping_add(i64::from(digit - b'0'));
    }
    if negative {
        value = value.wrapping_neg();
    }
    #[expect(clippy::cast_possible_truncation, reason = "C's int wraps")]
    let wrapped = value as i32;
    wrapped
}

/// A whole number, as a stem's count or entries are read: `None` for any
/// other text.
fn whole(text: &[u8]) -> Option<i64> {
    Number::parse_bytes(text).and_then(|number| number.int64_value(DIGITS64))
}

// --- Stems ------------------------------------------------------------------

/// A stem argument: the stem, and the text every element name it reads or
/// writes starts with.
#[derive(Clone)]
struct Stem {
    object: ObjRef,
    prefix: Vec<u8>,
}

/// The stem `value` names: a stem object itself, or a string naming one of
/// the caller's stems, `name.tail` prefixing each element with `TAIL`.
/// `None` where the string names no stem.
fn stem_of(interp: &mut Interp, value: ObjRef, position: usize) -> Result<Option<Stem>> {
    if interp.is_stem_object(value) {
        return Ok(Some(Stem {
            object: value,
            prefix: Vec::new(),
        }));
    }
    let Some(text) = interp.blamed_string_conversion(value)? else {
        return Err(crate::error::Raised::native_argument_needs_a_string_value(position).into());
    };
    let mut name = interp.to_text(text).to_ascii_uppercase();
    if let Some(end) = name.iter().position(|byte| *byte == 0) {
        name.truncate(end);
    }
    let (stem_name, prefix) = match name.iter().position(|byte| *byte == b'.') {
        Some(dot) => (name[..=dot].to_vec(), name[dot + 1..].to_vec()),
        None => (name.clone(), Vec::new()),
    };
    if stem_name.is_empty() {
        return Ok(None);
    }
    Ok(interp
        .context_stem_named(&stem_name)
        .map(|object| Stem { object, prefix }))
}

/// A required stem argument.
fn stem_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Option<Stem>> {
    let value = required(args, position)?;
    stem_of(interp, value, position)
}

impl Stem {
    fn key(&self, element: &[u8]) -> Vec<u8> {
        let mut key = self.prefix.clone();
        key.extend_from_slice(element);
        key
    }

    /// An element's own value as text, `None` where it is unset.
    fn get(&self, interp: &mut Interp, element: &[u8]) -> Option<Vec<u8>> {
        let value = interp.stem_object_tail(self.object, &self.key(element))?;
        Some(interp.to_text(value).into_owned())
    }

    fn set(&self, interp: &mut Interp, element: &[u8], value: ObjRef) {
        interp.roots.activity_mut().push_temp(value);
        interp.stem_object_write(self.object, &self.key(element), value);
    }

    fn set_text(&self, interp: &mut Interp, element: &[u8], text: &[u8]) {
        let value = interp.text(text);
        self.set(interp, element, value);
    }

    fn set_number(&self, interp: &mut Interp, element: &[u8], value: i64) {
        let value = number(interp, value);
        self.set(interp, element, value);
    }
}

// --- Addresses --------------------------------------------------------------

/// An address stem's three elements, as the OS call takes them.
#[derive(Clone, Copy)]
struct Address {
    family: u16,
    port: u16,
    addr: Ipv4Addr,
}

impl Address {
    const ZERO: Address = Address {
        family: 0,
        port: 0,
        addr: Ipv4Addr::UNSPECIFIED,
    };

    fn v4(self) -> SocketAddrV4 {
        SocketAddrV4::new(self.addr, self.port)
    }
}

/// `inet_aton`: one to four dotted parts, each decimal, octal (`0`) or hex
/// (`0x`), the last filling the bytes the others leave; trailing white space
/// ends the address. `None` for anything else.
fn inet_aton(text: &[u8]) -> Option<Ipv4Addr> {
    let end = text
        .iter()
        .position(u8::is_ascii_whitespace)
        .unwrap_or(text.len());
    let text = &text[..end];
    let mut parts = Vec::new();
    for part in text.split(|byte| *byte == b'.') {
        if !part.first().is_some_and(u8::is_ascii_digit) {
            return None;
        }
        let (digits, radix) = match part {
            [b'0', b'x' | b'X', rest @ ..] => (rest, 16),
            [b'0', rest @ ..] if !rest.is_empty() => (rest, 8),
            _ => (part, 10),
        };
        let mut value: u64 = 0;
        for byte in digits {
            let digit = char::from(*byte).to_digit(radix)?;
            value = value
                .checked_mul(u64::from(radix))?
                .checked_add(u64::from(digit))?;
            if value > u64::from(u32::MAX) {
                return None;
            }
        }
        parts.push(value);
    }
    let (last, leading) = parts.split_last()?;
    if leading.len() > 3 || leading.iter().any(|part| *part > 0xff) {
        return None;
    }
    let room = 32 - 8 * leading.len();
    if room < 32 && *last >> room != 0 {
        return None;
    }
    let mut value = *last;
    for (index, part) in leading.iter().enumerate() {
        value |= part << (24 - 8 * index);
    }
    u32::try_from(value).ok().map(Ipv4Addr::from)
}

/// An `ADDR` value: `INADDR_ANY` caselessly, else `inet_addr`'s reading,
/// 255.255.255.255 where that fails.
fn parse_addr(text: &[u8]) -> Ipv4Addr {
    if text.eq_ignore_ascii_case(b"INADDR_ANY") {
        return Ipv4Addr::UNSPECIFIED;
    }
    inet_aton(text).unwrap_or(Ipv4Addr::BROADCAST)
}

/// The address a stem holds, all zero where any element is unset.
fn read_address(interp: &mut Interp, stem: &Stem) -> Address {
    let (Some(family), Some(port), Some(addr)) = (
        stem.get(interp, b"FAMILY"),
        stem.get(interp, b"PORT"),
        stem.get(interp, b"ADDR"),
    ) else {
        return Address::ZERO;
    };
    let family = trimmed(&family);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "C's u_short"
    )]
    let family = if family.eq_ignore_ascii_case(b"AF_INET") {
        2
    } else {
        atoi(family) as u16
    };
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "C's htons"
    )]
    let port = atoi(trimmed(&port)) as u16;
    Address {
        family,
        port,
        addr: parse_addr(trimmed(&addr)),
    }
}

/// Writes `address` to `stem`: the family as a number, the address dotted.
fn write_address(interp: &mut Interp, stem: &Stem, family: i64, address: SocketAddrV4) {
    stem.set_number(interp, b"FAMILY", family);
    stem.set_number(interp, b"PORT", i64::from(address.port()));
    stem.set_text(interp, b"ADDR", address.ip().to_string().as_bytes());
}

/// Writes a socket address an OS call answered: an IPv4 one as itself, any
/// other with its family and zeros.
fn write_socket_address(
    interp: &mut Interp,
    stem: &Stem,
    address: Option<rustix::net::SocketAddrAny>,
) {
    let unspecified = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
    match address {
        Some(any) => {
            let family = i64::from(any.address_family().as_raw());
            let v4 = SocketAddrV4::try_from(any).unwrap_or(unspecified);
            write_address(interp, stem, family, v4);
        }
        None => write_address(interp, stem, 0, unspecified),
    }
}

/// How an inet socket takes `address` for `bind`, `connect` or `sendto`.
enum Target {
    Inet(SocketAddrV4),
    /// `AF_UNSPEC`, which `connect` reads as a disconnect, `bind` accepts with
    /// the any address, and `sendto` reads as `AF_INET`.
    Unspec(SocketAddrV4),
    /// Any other family, which an inet socket refuses with `EAFNOSUPPORT`.
    Other,
}

fn target(address: Address) -> Target {
    match address.family {
        2 => Target::Inet(address.v4()),
        0 => Target::Unspec(address.v4()),
        _ => Target::Other,
    }
}

// --- Waiting ----------------------------------------------------------------

/// A call waiting for its sockets, on [`crate::activity::Activity::socket_wait`].
pub(crate) struct SocketWait {
    /// The descriptors and the readiness each waits for.
    polls: Vec<(i32, PollFlags)>,
    /// When the wait ends whatever the sockets do.
    deadline: Option<Instant>,
    op: Pending,
}

/// What a waiting call does once its wait ends.
enum Pending {
    Accept {
        fd: i32,
        stem: Option<Vec<u8>>,
        /// The stem argument named no stem: the descriptor is lost.
        lost: bool,
        /// The connection the wait's re-test took, once it has.
        accepted: Option<Os<Accepted>>,
    },
    Connect {
        fd: i32,
        to: SocketAddrV4,
    },
    Recv {
        fd: i32,
        var: Vec<u8>,
        len: usize,
        flags: RecvFlags,
        from: Option<Vec<u8>>,
    },
    Select {
        sets: [Option<Set>; 3],
    },
}

/// The readiness of `polls` now, each entry's received events; `None` where
/// any descriptor is invalid.
fn poll_now(polls: &[(i32, PollFlags)]) -> Option<Vec<PollFlags>> {
    let numbers: Vec<i32> = polls.iter().map(|(fd, _)| *fd).collect();
    with_sockets(&numbers, |sockets| {
        let mut fds: Vec<PollFd<'_>> = sockets
            .iter()
            .zip(polls)
            .map(|(fd, (_, events))| PollFd::new(fd, *events))
            .collect();
        let zero = rustix::time::Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        rustix::event::poll(&mut fds, Some(&zero)).map_err(os)?;
        Ok(fds.iter().map(PollFd::revents).collect())
    })
    .ok()
}

/// Whether `revents` answers a wait for `events`, as `select` reads it:
/// hang-up and error count as readable and error as writable.
fn answers(events: PollFlags, revents: PollFlags) -> bool {
    let readable = PollFlags::IN | PollFlags::HUP | PollFlags::ERR;
    let writable = PollFlags::OUT | PollFlags::ERR;
    (events.contains(PollFlags::IN) && revents.intersects(readable))
        || (events.contains(PollFlags::OUT) && revents.intersects(writable))
        || (events.contains(PollFlags::PRI) && revents.contains(PollFlags::PRI))
        || revents.contains(PollFlags::NVAL)
}

/// Whether a wait for `polls` is over: any of them ready, or any invalid.
fn any_ready(polls: &[(i32, PollFlags)]) -> bool {
    match poll_now(polls) {
        Some(revents) => polls
            .iter()
            .zip(revents)
            .any(|((_, events), revents)| answers(*events, revents)),
        None => true,
    }
}

/// The running activity's socket wait, re-tested at its sleep's end: the
/// next poll's time while it goes on, `None` once its sockets are ready or
/// its deadline has passed.
pub(crate) fn wait_again(interp: &mut Interp) -> Option<Instant> {
    let now = interp.now();
    let wait = interp.activity.socket_wait.as_mut()?;
    if wait.deadline.is_some_and(|deadline| now >= deadline) {
        return None;
    }
    if any_ready(&wait.polls) {
        // A ready listening socket can lose its connection to another
        // accept, in this process or another, before this one runs: the
        // accept is tried here without blocking, and the wait goes on where
        // it finds nothing.
        let Pending::Accept { fd, accepted, .. } = &mut wait.op else {
            return None;
        };
        match try_accept(*fd) {
            Err(EAGAIN) => {}
            result => {
                *accepted = Some(result);
                return None;
            }
        }
    }
    let next = now + WAIT_POLL;
    Some(wait.deadline.map_or(next, |deadline| deadline.min(next)))
}

/// Parks the running call until `wait`'s sockets are ready or its deadline
/// passes, then answers what [`resume`] does; `stems` are the stems the
/// finishing call writes, kept rooted meanwhile.
fn park(interp: &mut Interp, wait: SocketWait, stems: &[Option<ObjRef>]) -> Result<ObjRef> {
    let now = interp.now();
    let next = now + WAIT_POLL;
    let deadline = wait.deadline.map_or(next, |deadline| deadline.min(next));
    let receiver = interp.new_array(stems);
    interp.activity.socket_wait = Some(Box::new(wait));
    let parked = interp.park_rust_routine(ParkReason::Sleep { deadline }, resume, receiver);
    if interp.activity.native_park.is_none() {
        interp.activity.socket_wait = None;
    }
    parked
}

/// The `index`th stem a parked call kept.
fn kept_stem(interp: &Interp, receiver: ObjRef, index: usize) -> Option<ObjRef> {
    match &interp.heap.get(receiver)?.body {
        Body::Array { slots, .. } => slots.get(index).copied().flatten(),
        _ => None,
    }
}

/// A waiting call's continuation: the OS call it waited to make, or, where a
/// halt ended the wait, -1 with `EINTR`.
fn resume(interp: &mut Interp, receiver: ObjRef) -> Result<Option<ObjRef>> {
    interp.activity.asleep = None;
    let halted = std::mem::take(&mut interp.activity.woken_by_halt);
    let Some(wait) = interp.activity.socket_wait.take() else {
        return Ok(Some(number(interp, -1)));
    };
    let stem = |interp: &Interp, index: usize, prefix: Option<Vec<u8>>| {
        prefix.and_then(|prefix| {
            kept_stem(interp, receiver, index).map(|object| Stem { object, prefix })
        })
    };
    let answered = match wait.op {
        _ if halted => {
            if let Pending::Recv { var, .. } = &wait.op {
                let empty = interp.text(b"");
                interp.set_context_variable(var, empty);
            }
            answer(interp, Err(EINTR))
        }
        Pending::Accept {
            fd,
            stem: prefix,
            lost,
            accepted,
        } => {
            // At the deadline, the one try answers `EAGAIN` where nothing
            // came: measured, oracle, `SO_RCVTIMEO` 500 then `SockAccept`
            // gives -1 with `errno` 11 after half a second.
            let accepted = accepted.unwrap_or_else(|| try_accept(fd));
            let target = stem(interp, 0, prefix);
            finish_accept(interp, accepted, target.as_ref(), lost)
        }
        Pending::Connect { fd, to } => {
            // Not writable is the deadline: measured, oracle, `SO_SNDTIMEO`
            // 500 then a `SockConnect` that cannot complete gives -1 with
            // `errno` 115 after half a second, and the connection goes on.
            let result = if any_ready(&[(fd, PollFlags::OUT)]) {
                with_socket(fd, |socket| finish_connect(socket, to))
            } else {
                Err(EINPROGRESS)
            };
            code(interp, result)
        }
        Pending::Recv {
            fd,
            var,
            len,
            flags,
            from,
        } => {
            let target = stem(interp, 0, from);
            finish_recv(
                interp,
                fd,
                &var,
                len,
                flags | RecvFlags::DONTWAIT,
                target.as_ref(),
            )
        }
        Pending::Select { sets } => {
            let mut sets = sets;
            for (index, set) in sets.iter_mut().enumerate() {
                if let Some(set) = set {
                    set.stem.object = kept_stem(interp, receiver, index).unwrap_or(ObjRef::NIL);
                }
            }
            finish_select(interp, &sets)
        }
    };
    Ok(Some(answered))
}

// --- Loading and status -----------------------------------------------------

pub(crate) fn version(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    Ok(interp.text(crate::version::VERSION_NUMBER))
}

/// `SockLoadFuncs([x])`: the library is loaded already; `""`.
pub(crate) fn load_funcs(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    optional_string(interp, args, 1)?;
    Ok(interp.text(b""))
}

pub(crate) fn drop_funcs(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    Ok(interp.text(b""))
}

pub(crate) fn init(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    report(interp, 0);
    Ok(number(interp, 0))
}

/// `SockSock_Errno()`: the `errno` of this activity's last `rxsock` call.
pub(crate) fn sock_errno(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    let errno = interp.activity.rxsock_errno;
    Ok(number(interp, errno))
}

pub(crate) fn psock_errno(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let text = optional_string(interp, args, 1)?.unwrap_or_default();
    let mut line = b"RxSOCK Error: ".to_vec();
    line.extend_from_slice(&text);
    line.push(b'\n');
    interp.write_err(&line);
    Ok(interp.text(b""))
}

// --- Sockets ----------------------------------------------------------------

/// `SockSocket(domain, type, protocol)`.
pub(crate) fn socket_new(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 3)?;
    let domain = c_string(interp, args, 1)?;
    let kind = c_string(interp, args, 2)?;
    let protocol = c_string(interp, args, 3)?;
    if !trimmed(&domain).eq_ignore_ascii_case(b"AF_INET") {
        return Err(failed(interp));
    }
    let kind = match trimmed(&kind).to_ascii_uppercase().as_slice() {
        b"SOCK_STREAM" => rustix::net::SocketType::STREAM,
        b"SOCK_DGRAM" => rustix::net::SocketType::DGRAM,
        b"SOCK_RAW" => rustix::net::SocketType::RAW,
        _ => return Err(failed(interp)),
    };
    let protocol = match trimmed(&protocol).to_ascii_uppercase().as_slice() {
        b"IPPROTO_UDP" => Some(rustix::net::ipproto::UDP),
        b"IPPROTO_TCP" => Some(rustix::net::ipproto::TCP),
        b"0" => None,
        _ => return Err(failed(interp)),
    };
    let made = rustix::net::socket(rustix::net::AddressFamily::INET, kind, protocol)
        .map(|socket| i64::from(socket.into_raw_fd()))
        .map_err(os);
    Ok(answer(interp, made))
}

/// `SockClose(s)` and `SockSoClose(s)`.
pub(crate) fn close(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let fd = int(interp, args, 1)?;
    // The interpreter's own descriptors are not the program's to close.
    let closed = if rexx_rxapi::reserved::is_reserved(fd) {
        Err(libc::EBADF)
    } else {
        nix::unistd::close(fd).map_err(nix_os)
    };
    Ok(code(interp, closed))
}

pub(crate) fn shutdown(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let fd = int(interp, args, 1)?;
    let how = int(interp, args, 2)?;
    let result = with_socket(fd, |socket| {
        let how = match how {
            0 => rustix::net::Shutdown::Read,
            1 => rustix::net::Shutdown::Write,
            2 => rustix::net::Shutdown::Both,
            _ => return Err(EINVAL),
        };
        rustix::net::shutdown(socket, how).map_err(os)
    });
    Ok(code(interp, result))
}

pub(crate) fn listen(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let fd = int(interp, args, 1)?;
    let backlog = int(interp, args, 2)?;
    let result = with_socket(fd, |socket| {
        rustix::net::listen(socket, backlog).map_err(os)
    });
    Ok(code(interp, result))
}

pub(crate) fn bind(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let fd = int(interp, args, 1)?;
    let Some(stem) = stem_argument(interp, args, 2)? else {
        return Ok(number(interp, 0));
    };
    let address = read_address(interp, &stem);
    let result = with_socket(fd, |socket| match target(address) {
        Target::Inet(v4) => rustix::net::bind(socket, &v4).map_err(os),
        Target::Unspec(v4) if v4.ip().is_unspecified() => {
            rustix::net::bind(socket, &v4).map_err(os)
        }
        Target::Unspec(_) | Target::Other => Err(EAFNOSUPPORT),
    });
    Ok(code(interp, result))
}

/// `SockConnect(s, addrstem)`. A blocking socket connects without blocking
/// and the call waits for the outcome.
pub(crate) fn connect(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let fd = int(interp, args, 1)?;
    let Some(stem) = stem_argument(interp, args, 2)? else {
        return Ok(number(interp, 0));
    };
    let address = read_address(interp, &stem);
    // Ok(Some(to)) for a connection under way, which the call waits for.
    let begun = with_socket(fd, |socket| {
        let v4 = match target(address) {
            Target::Inet(v4) => v4,
            Target::Unspec(_) => {
                return rustix::net::connect_unspec(socket)
                    .map(|()| None)
                    .map_err(os);
            }
            Target::Other => return Err(EAFNOSUPPORT),
        };
        if !blocks(socket) {
            return rustix::net::connect(socket, &v4).map(|()| None).map_err(os);
        }
        let flags = rustix::fs::fcntl_getfl(socket).map_err(os)?;
        rustix::fs::fcntl_setfl(socket, flags | rustix::fs::OFlags::NONBLOCK).map_err(os)?;
        let connected = rustix::net::connect(socket, &v4).map_err(os);
        let _ = rustix::fs::fcntl_setfl(socket, flags);
        match connected {
            Err(EINPROGRESS) => Ok(Some(v4)),
            result => result.map(|()| None),
        }
    });
    match begun {
        Ok(Some(to)) => {
            let wait = SocketWait {
                polls: vec![(fd, PollFlags::OUT)],
                deadline: socket_deadline(interp, fd, rustix::net::sockopt::Timeout::Send),
                op: Pending::Connect { fd, to },
            };
            park(interp, wait, &[])
        }
        result => Ok(code(interp, result.map(|_| ()))),
    }
}

/// A connection's outcome once its socket is writable: a second `connect`,
/// without blocking, which answers `EISCONN` for one made and the error for
/// one refused, and which disconnects the failed socket as a blocking
/// connect does (its local address goes back to 0.0.0.0).
fn finish_connect(socket: BorrowedFd<'_>, to: SocketAddrV4) -> Os<()> {
    let flags = rustix::fs::fcntl_getfl(socket).map_err(os)?;
    rustix::fs::fcntl_setfl(socket, flags | rustix::fs::OFlags::NONBLOCK).map_err(os)?;
    let connected = rustix::net::connect(socket, &to);
    let _ = rustix::fs::fcntl_setfl(socket, flags);
    match connected {
        Ok(()) | Err(rustix::io::Errno::ISCONN) => Ok(()),
        Err(error) => Err(os(error)),
    }
}

/// `SockAccept(s [, addrstem])`.
pub(crate) fn accept(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let fd = int(interp, args, 1)?;
    let (stem, lost) = match args.get(1).copied().flatten() {
        Some(value) => match stem_of(interp, value, 2)? {
            Some(stem) => (Some(stem), false),
            None => (None, true),
        },
        None => (None, false),
    };
    let blocking = with_socket(fd, |socket| Ok(blocks(socket))).unwrap_or(false);
    // A blocking socket's accept is tried without blocking once it is
    // ready, and waits again where another accept took the connection.
    let accepted = match blocking && !any_ready(&[(fd, PollFlags::IN)]) {
        true => None,
        false => match try_accept(fd) {
            Err(EAGAIN) if blocking => None,
            result => Some(result),
        },
    };
    let Some(accepted) = accepted else {
        let wait = SocketWait {
            polls: vec![(fd, PollFlags::IN)],
            deadline: socket_deadline(interp, fd, rustix::net::sockopt::Timeout::Recv),
            op: Pending::Accept {
                fd,
                stem: stem.as_ref().map(|stem| stem.prefix.clone()),
                lost,
                accepted: None,
            },
        };
        let kept = [stem.as_ref().map(|stem| stem.object)];
        return park(interp, wait, &kept);
    };
    Ok(finish_accept(interp, accepted, stem.as_ref(), lost))
}

/// An accepted connection and its peer's address.
type Accepted = (OwnedFd, Option<rustix::net::SocketAddrAny>);

/// `accept` on the socket numbered `fd` without blocking: a blocking socket
/// is made non-blocking for the call.
fn try_accept(fd: i32) -> Os<Accepted> {
    with_socket(fd, |socket| {
        let flags = rustix::fs::fcntl_getfl(socket).map_err(os)?;
        let toggled = !flags.contains(rustix::fs::OFlags::NONBLOCK);
        if toggled {
            rustix::fs::fcntl_setfl(socket, flags | rustix::fs::OFlags::NONBLOCK).map_err(os)?;
        }
        let accepted = rustix::net::acceptfrom(socket).map_err(os);
        if toggled {
            let _ = rustix::fs::fcntl_setfl(socket, flags);
        }
        accepted
    })
}

/// The deadline a socket's `SO_RCVTIMEO` (`Timeout::Recv`) or `SO_SNDTIMEO`
/// (`Timeout::Send`) sets for a wait starting now; `None` where it is 0.
fn socket_deadline(
    interp: &Interp,
    fd: i32,
    which: rustix::net::sockopt::Timeout,
) -> Option<Instant> {
    with_socket(fd, |socket| {
        rustix::net::sockopt::socket_timeout(socket, which).map_err(os)
    })
    .ok()
    .flatten()
    .map(|timeout| interp.now() + timeout)
}

/// An accept's answer: the new socket's number, or -1. With a stem it gets
/// the peer's address, even on failure. Where the stem argument named no
/// stem (`lost`), the connection's descriptor is lost and the call answers
/// 0 without setting `errno`.
fn finish_accept(
    interp: &mut Interp,
    accepted: Os<Accepted>,
    stem: Option<&Stem>,
    lost: bool,
) -> ObjRef {
    if lost {
        if let Ok((accepted, _)) = accepted {
            let _ = accepted.into_raw_fd();
        }
        return number(interp, 0);
    }
    let (result, peer) = match accepted {
        Ok((accepted, peer)) => (Ok(i64::from(accepted.into_raw_fd())), peer),
        Err(errno) => (Err(errno), None),
    };
    if let Some(stem) = stem {
        write_socket_address(interp, stem, peer);
    }
    answer(interp, result)
}

/// `SockGetSockName(s, stem)` and `SockGetPeerName(s, stem)`.
fn name_of(interp: &mut Interp, args: &[Option<ObjRef>], peer: bool) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let fd = int(interp, args, 1)?;
    let Some(stem) = stem_argument(interp, args, 2)? else {
        return Ok(number(interp, 0));
    };
    let named = with_socket(fd, |socket| match peer {
        true => rustix::net::getpeername(socket).map_err(os),
        false => rustix::net::getsockname(socket).map(Some).map_err(os),
    });
    let (result, address) = match named {
        Ok(address) => (Ok(()), address),
        Err(errno) => (Err(errno), None),
    };
    write_socket_address(interp, &stem, address);
    Ok(code(interp, result))
}

pub(crate) fn get_sock_name(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    name_of(interp, args, false)
}

pub(crate) fn get_peer_name(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    name_of(interp, args, true)
}

// --- Hosts ------------------------------------------------------------------

/// Fills a host stem from `host`.
fn write_host(interp: &mut Interp, stem: &Stem, host: &hosts::Host) {
    stem.set_text(interp, b"NAME", &host.name);
    let aliases = i64::try_from(host.aliases.len()).unwrap_or(i64::MAX);
    stem.set_number(interp, b"ALIAS.0", aliases);
    for (index, alias) in host.aliases.iter().enumerate() {
        stem.set_text(interp, format!("ALIAS.{}", index + 1).as_bytes(), alias);
    }
    stem.set_text(interp, b"ADDRTYPE", b"AF_INET");
    if let Some(first) = host.addresses.first() {
        stem.set_text(interp, b"ADDR", first.to_string().as_bytes());
    }
    let addresses = i64::try_from(host.addresses.len()).unwrap_or(i64::MAX);
    stem.set_number(interp, b"ADDR.0", addresses);
    for (index, address) in host.addresses.iter().enumerate() {
        stem.set_text(
            interp,
            format!("ADDR.{}", index + 1).as_bytes(),
            address.to_string().as_bytes(),
        );
    }
}

/// A lookup's answer: 1 with the stem filled, or 0.
fn host_answer(interp: &mut Interp, stem: &Stem, found: Os<Option<hosts::Host>>) -> ObjRef {
    let (found, errno) = match found {
        Ok(found) => (found, 0),
        Err(errno) => (None, errno),
    };
    // A lookup sets C's `errno` to 0 first: measured, oracle,
    // `SockSock_Errno()` is 0 after a successful `SockGetHostByName`
    // that followed a failing call.
    interp.activity.rxsock_errno = 0;
    report(interp, errno);
    match found {
        Some(host) => {
            write_host(interp, stem, &host);
            number(interp, 1)
        }
        None => number(interp, 0),
    }
}

pub(crate) fn get_host_by_name(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let name = c_string(interp, args, 1)?;
    let Some(stem) = stem_argument(interp, args, 2)? else {
        return Ok(number(interp, 0));
    };
    let found = hosts::by_name(&name);
    Ok(host_answer(interp, &stem, Ok(found)))
}

pub(crate) fn get_host_by_addr(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 3)?;
    let addr = c_string(interp, args, 1)?;
    let Some(value) = args.get(1).copied().flatten() else {
        return Err(crate::error::Raised::missing_native_argument(2).into());
    };
    let domain = match args.get(2).copied().flatten() {
        Some(_) => int(interp, args, 3)?,
        None => 2,
    };
    let Some(stem) = stem_of(interp, value, 2)? else {
        return Ok(number(interp, 0));
    };
    let found = match domain {
        2 => Ok(hosts::by_addr(parse_addr(trimmed(&addr)))),
        _ => Err(EAFNOSUPPORT),
    };
    Ok(host_answer(interp, &stem, found))
}

/// `SockGetHostName()`: the node name, at most 255 bytes.
pub(crate) fn get_host_name(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    let info = rustix::system::uname();
    let name = info.nodename().to_bytes();
    let name = name[..name.len().min(255)].to_vec();
    report(interp, 0);
    Ok(interp.text_built(name))
}

/// `SockGetHostId()`: the first interface address, in interface-list order,
/// whose text does not begin `127`, the last interface never examined (D9);
/// 40.1 where none qualifies (Rulings), `0.0.0.0` where the list cannot be
/// read.
pub(crate) fn get_host_id(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    let Ok(interfaces) = nix::ifaddrs::getifaddrs() else {
        return Ok(interp.text(b"0.0.0.0"));
    };
    let addresses: Vec<Ipv4Addr> = interfaces
        .filter_map(|interface| {
            interface
                .address
                .as_ref()
                .and_then(|address| address.as_sockaddr_in().map(|v4| v4.ip()))
        })
        .collect();
    let examined = addresses.len().saturating_sub(1);
    match addresses[..examined]
        .iter()
        .map(Ipv4Addr::to_string)
        .find(|text| !text.starts_with("127"))
    {
        Some(text) => Ok(interp.text_built(text.into_bytes())),
        None => Err(failed(interp)),
    }
}

// --- Data -------------------------------------------------------------------

/// The flags a flag word list names, caselessly, unknown words ignored.
fn flag_words(text: &[u8], known: &[(&str, u32)]) -> u32 {
    text.split(u8::is_ascii_whitespace)
        .filter(|word| !word.is_empty())
        .filter_map(|word| {
            known
                .iter()
                .find(|(name, _)| word.eq_ignore_ascii_case(name.as_bytes()))
                .map(|(_, bit)| *bit)
        })
        .fold(0, |flags, bit| flags | bit)
}

fn send_flags(text: &[u8], oob: bool) -> SendFlags {
    let oob_bit = if oob { SendFlags::OOB.bits() } else { 0 };
    let bits = flag_words(
        text,
        &[
            ("MSG_OOB", oob_bit),
            ("MSG_DONTROUTE", SendFlags::DONTROUTE.bits()),
        ],
    );
    SendFlags::from_bits_retain(bits) | SendFlags::NOSIGNAL
}

fn recv_flags(text: &[u8]) -> RecvFlags {
    let bits = flag_words(
        text,
        &[
            ("MSG_OOB", RecvFlags::OOB.bits()),
            ("MSG_PEEK", RecvFlags::PEEK.bits()),
        ],
    );
    RecvFlags::from_bits_retain(bits)
}

pub(crate) fn send(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 3)?;
    let fd = int(interp, args, 1)?;
    let bytes = data(interp, args, 2)?;
    let flags = optional_string(interp, args, 3)?.unwrap_or_default();
    let flags = send_flags(&flags, true);
    // A blocking send takes a halting signal on this thread, which ends it,
    // and the halt is served at this clause: measured, oracle, SIGINT during
    // a blocked `SockSend` raises HALT on the `SockSend` line.
    let sent = crate::signal::unblocked(|| {
        with_socket(fd, |socket| {
            rustix::net::send(socket, &bytes, flags)
                .map(|count| i64::try_from(count).unwrap_or(i64::MAX))
                .map_err(os)
        })
    });
    let answered = answer(interp, sent);
    interp.serve_signal_now();
    Ok(answered)
}

/// `SockSendTo(s, data, addrstem)` or `SockSendTo(s, data, flags, addrstem)`.
pub(crate) fn send_to(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 4)?;
    let fd = int(interp, args, 1)?;
    let bytes = data(interp, args, 2)?;
    required(args, 3)?;
    let (flags, stem) = match args.get(3).copied().flatten() {
        Some(value) => {
            let flags = c_string(interp, args, 3)?;
            (flags, stem_of(interp, value, 4)?)
        }
        None => (Vec::new(), stem_argument(interp, args, 3)?),
    };
    let Some(stem) = stem else {
        return Ok(number(interp, 0));
    };
    let flags = send_flags(&flags, false);
    let address = read_address(interp, &stem);
    // As `SockSend`: a halting signal ends a blocked send at this clause.
    let sent = crate::signal::unblocked(|| {
        with_socket(fd, |socket| {
            let count = match target(address) {
                Target::Inet(v4) | Target::Unspec(v4) => {
                    rustix::net::sendto(socket, &bytes, flags, &v4)
                }
                Target::Other
                    if rustix::net::sockopt::socket_type(socket)
                        == Ok(rustix::net::SocketType::STREAM) =>
                {
                    rustix::net::send(socket, &bytes, flags)
                }
                Target::Other => return Err(EAFNOSUPPORT),
            };
            count
                .map(|count| i64::try_from(count).unwrap_or(i64::MAX))
                .map_err(os)
        })
    });
    let answered = answer(interp, sent);
    interp.serve_signal_now();
    Ok(answered)
}

/// The most a receive reads in one call, whatever length it asks for.
const RECV_MOST: usize = 16 << 20;

/// `SockRecv(s, var, len [, flags])`.
pub(crate) fn recv(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 4)?;
    receive(interp, args, false)
}

/// `SockRecvFrom(s, var, len, addrstem)` or
/// `SockRecvFrom(s, var, len, flags, addrstem)`.
pub(crate) fn recv_from(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 5)?;
    receive(interp, args, true)
}

fn receive(interp: &mut Interp, args: &[Option<ObjRef>], from: bool) -> Result<ObjRef> {
    let fd = int(interp, args, 1)?;
    let var = c_string(interp, args, 2)?;
    let len = int(interp, args, 3)?;
    let (flags, stem) = if from {
        required(args, 4)?;
        match args.get(4).copied().flatten() {
            Some(value) => {
                let flags = c_string(interp, args, 4)?;
                (flags, stem_of(interp, value, 5)?)
            }
            None => (Vec::new(), stem_argument(interp, args, 4)?),
        }
    } else {
        (optional_string(interp, args, 4)?.unwrap_or_default(), None)
    };
    let Ok(len) = usize::try_from(len) else {
        return Err(failed(interp));
    };
    if from && stem.is_none() {
        return Ok(number(interp, 0));
    }
    let flags = recv_flags(&flags);
    let waits = !flags.contains(RecvFlags::OOB)
        && with_socket(fd, |socket| Ok(blocks(socket))).unwrap_or(false)
        && !any_ready(&[(fd, PollFlags::IN)]);
    if waits {
        let deadline = socket_deadline(interp, fd, rustix::net::sockopt::Timeout::Recv);
        let wait = SocketWait {
            polls: vec![(fd, PollFlags::IN)],
            deadline,
            op: Pending::Recv {
                fd,
                var,
                len,
                flags,
                from: stem.as_ref().map(|stem| stem.prefix.clone()),
            },
        };
        let kept = [stem.as_ref().map(|stem| stem.object)];
        return park(interp, wait, &kept);
    }
    Ok(finish_recv(interp, fd, &var, len, flags, stem.as_ref()))
}

fn finish_recv(
    interp: &mut Interp,
    fd: i32,
    var: &[u8],
    len: usize,
    flags: RecvFlags,
    from: Option<&Stem>,
) -> ObjRef {
    let mut buffer = vec![0; len.min(RECV_MOST)];
    let received = with_socket(fd, |socket| match from {
        Some(_) => rustix::net::recvfrom(socket, &mut buffer[..], flags)
            .map(|(_, count, address)| (count, address))
            .map_err(os),
        None => rustix::net::recv(socket, &mut buffer[..], flags)
            .map(|(_, count)| (count, None))
            .map_err(os),
    });
    let (result, address) = match received {
        Ok((count, address)) => {
            buffer.truncate(count.min(buffer.len()));
            (Ok(i64::try_from(count).unwrap_or(i64::MAX)), address)
        }
        Err(errno) => {
            buffer.clear();
            (Err(errno), None)
        }
    };
    let value = interp.text_built(buffer);
    interp.roots.activity_mut().push_temp(value);
    interp.set_context_variable(var, value);
    if let Some(stem) = from
        && result.is_ok()
    {
        write_socket_address(interp, stem, address);
    }
    answer(interp, result)
}

// --- Options and control ----------------------------------------------------

/// The option names and their Linux numbers.
const OPTIONS: &[(&str, i32)] = &[
    ("SO_DEBUG", 1),
    ("SO_REUSEADDR", 2),
    ("SO_TYPE", 3),
    ("SO_ERROR", 4),
    ("SO_DONTROUTE", 5),
    ("SO_BROADCAST", 6),
    ("SO_SNDBUF", 7),
    ("SO_RCVBUF", 8),
    ("SO_KEEPALIVE", 9),
    ("SO_OOBINLINE", 10),
    ("SO_LINGER", 13),
    ("SO_RCVLOWAT", 18),
    ("SO_SNDLOWAT", 19),
    ("SO_RCVTIMEO", 20),
    ("SO_SNDTIMEO", 21),
    ("SO_USELOOPBACK", 64),
];

/// The option `name` names caselessly, `None` for an unknown one.
fn option_number(name: &[u8]) -> Option<i32> {
    OPTIONS
        .iter()
        .find(|(known, _)| name.eq_ignore_ascii_case(known.as_bytes()))
        .map(|(_, number)| *number)
}

/// Raises 40.1 unless `level` is `SOL_SOCKET`, caselessly.
fn check_level(interp: &Interp, level: &[u8]) -> Result<()> {
    match level.eq_ignore_ascii_case(b"SOL_SOCKET") {
        true => Ok(()),
        false => Err(failed(interp)),
    }
}

/// The options neither `rustix` nor `nix` names, read and set by number as
/// C `int`s: `SO_DEBUG`, `SO_RCVLOWAT`, `SO_SNDLOWAT` and the number 64
/// `SO_USELOOPBACK` passes.
const RAW_OPTIONS: [i32; 4] = [1, 18, 19, 64];

fn timeval_millis(seconds: i64, micros: i64) -> i64 {
    seconds * 1000 + micros / 1000
}

/// `SockGetSockOpt(s, level, option, var)`.
pub(crate) fn get_sock_opt(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    use nix::sys::socket::{getsockopt, sockopt};
    check_arity(args, 4)?;
    let fd = int(interp, args, 1)?;
    let level = c_string(interp, args, 2)?;
    let option = c_string(interp, args, 3)?;
    let var = c_string(interp, args, 4)?;
    check_level(interp, &level)?;
    let option = option_number(&option);
    let bool_of = |result: nix::Result<bool>| result.map(|on| i64::from(on).to_string());
    let read: Os<String> = match option {
        None => Err(ENOPROTOOPT),
        Some(option) if RAW_OPTIONS.contains(&option) => with_socket(fd, |socket| {
            rxsock_sys::get_int_option(socket, libc::SOL_SOCKET, option).map(|on| on.to_string())
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
                        .map_err(os);
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
                _ => return Err(ENOPROTOOPT),
            }
            .map_err(nix_os)
        }),
    };
    let (text, result) = match read {
        Ok(text) => (text, Ok(())),
        Err(errno) => {
            let text = match option {
                Some(3) => "UNKNOWN",
                Some(13) => "0 0",
                _ => "0",
            };
            (text.to_owned(), Err(errno))
        }
    };
    let value = interp.text_built(text.into_bytes());
    interp.roots.activity_mut().push_temp(value);
    interp.set_context_variable(&var, value);
    Ok(code(interp, result))
}

/// `SockSetSockOpt(s, level, option, value)`.
pub(crate) fn set_sock_opt(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    use nix::sys::socket::{setsockopt, sockopt};
    check_arity(args, 4)?;
    let fd = int(interp, args, 1)?;
    let level = c_string(interp, args, 2)?;
    let option = c_string(interp, args, 3)?;
    let value = c_string(interp, args, 4)?;
    check_level(interp, &level)?;
    let option = match option_number(&option) {
        None => return Ok(answer(interp, Err(ENOPROTOOPT))),
        Some(3 | 4) => return Ok(number(interp, -1)),
        Some(option) => option,
    };
    let numeric = atoi(&value);
    let on = numeric != 0;
    let result = with_socket(fd, |socket| {
        #[expect(
            clippy::cast_sign_loss,
            reason = "C passes the int, which the kernel reads unsigned"
        )]
        let size = numeric as u32 as usize;
        let set = match option {
            option if RAW_OPTIONS.contains(&option) => {
                return rxsock_sys::set_int_option(socket, libc::SOL_SOCKET, option, numeric);
            }
            2 => setsockopt(&socket, sockopt::ReuseAddr, &on),
            5 => setsockopt(&socket, sockopt::DontRoute, &on),
            6 => setsockopt(&socket, sockopt::Broadcast, &on),
            7 => setsockopt(&socket, sockopt::SndBuf, &size),
            8 => setsockopt(&socket, sockopt::RcvBuf, &size),
            9 => setsockopt(&socket, sockopt::KeepAlive, &on),
            10 => setsockopt(&socket, sockopt::OobInline, &on),
            13 => {
                let mut words = value
                    .split(u8::is_ascii_whitespace)
                    .filter(|word| !word.is_empty());
                #[expect(clippy::cast_possible_truncation, reason = "each is a 16-bit field")]
                let mut half = || i32::from(atoi(words.next().unwrap_or_default()) as u16);
                let linger = nix::libc::linger {
                    l_onoff: half(),
                    l_linger: half(),
                };
                setsockopt(&socket, sockopt::Linger, &linger)
            }
            20 | 21 => {
                let seconds = numeric / 1000;
                let micros = (numeric % 1000) * 1000;
                if micros < 0 {
                    return Err(EDOM);
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
            _ => return Err(ENOPROTOOPT),
        };
        set.map_err(nix_os)
    });
    Ok(code(interp, result))
}

/// `SockIoctl(s, cmd, arg)`.
pub(crate) fn ioctl(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 3)?;
    let fd = int(interp, args, 1)?;
    let command = c_string(interp, args, 2)?;
    let value = required(args, 3)?;
    if command.eq_ignore_ascii_case(b"FIONBIO") {
        let on = match value.decode() {
            Decoded::SmallInt(number) => i32::try_from(number).ok(),
            _ => {
                let text = c_string(interp, args, 3)?;
                whole(&text).and_then(|number| i32::try_from(number).ok())
            }
        };
        let Some(on) = on else {
            return Err(failed(interp));
        };
        let result = with_socket(fd, |socket| {
            rustix::io::ioctl_fionbio(socket, on != 0).map_err(os)
        });
        return Ok(code(interp, result));
    }
    if command.eq_ignore_ascii_case(b"FIONREAD") {
        let var = c_string(interp, args, 3)?;
        let read = with_socket(fd, |socket| rustix::io::ioctl_fionread(socket).map_err(os));
        let count = *read.as_ref().unwrap_or(&0);
        let count = number(interp, i128::from(count));
        interp.roots.activity_mut().push_temp(count);
        interp.set_context_variable(&var, count);
        return Ok(code(interp, read.map(|_| ())));
    }
    Ok(number(interp, -1))
}

// --- Select -----------------------------------------------------------------

/// One of `SockSelect`'s sets: its stem and the sockets it lists.
#[derive(Clone)]
struct Set {
    stem: Stem,
    sockets: Vec<i32>,
}

/// The set an argument names, `None` where it takes no part: omitted,
/// naming no stem, or with a count that is not a whole number. 40.1 for an
/// entry that is not a whole number (Deviation 88).
fn set_of(interp: &mut Interp, args: &[Option<ObjRef>], position: usize) -> Result<Option<Set>> {
    let Some(value) = args.get(position - 1).copied().flatten() else {
        return Ok(None);
    };
    // Measured, oracle: a name with a tail (`'q.x'`) takes no part, whatever
    // `Q.0` or `Q.X0` hold.
    let Some(stem) = stem_of(interp, value, position)?.filter(|stem| stem.prefix.is_empty()) else {
        return Ok(None);
    };
    let Some(count) = stem.get(interp, b"0").and_then(|count| whole(&count)) else {
        return Ok(None);
    };
    let mut sockets = Vec::new();
    for index in 1..=count.max(0) {
        let entry = stem.get(interp, index.to_string().as_bytes());
        let Some(fd) = entry
            .as_deref()
            .and_then(whole)
            .and_then(|fd| i32::try_from(fd).ok())
        else {
            return Err(failed(interp));
        };
        sockets.push(fd);
    }
    Ok(Some(Set { stem, sockets }))
}

const SET_EVENTS: [PollFlags; 3] = [PollFlags::IN, PollFlags::OUT, PollFlags::PRI];

/// The size of this process's descriptor table, `FDSize` in
/// `/proc/self/status`: `select` reads no descriptor at or above it.
fn descriptor_table_size() -> i32 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find_map(|line| line.strip_prefix("FDSize:"))
                .and_then(|size| size.trim().parse().ok())
        })
        .unwrap_or(i32::MAX)
}

/// Whether `select` looks at `fd`: measured, oracle, a negative entry or one
/// past the descriptor table is never ready and no error, while a closed one
/// below it answers -1 with `EBADF`.
fn selectable(fd: i32, table: i32) -> bool {
    (0..table).contains(&fd)
}

fn select_polls(sets: &[Option<Set>; 3]) -> Vec<(i32, PollFlags)> {
    let table = descriptor_table_size();
    sets.iter()
        .zip(SET_EVENTS)
        .filter_map(|(set, events)| set.as_ref().map(|set| (set, events)))
        .flat_map(|(set, events)| set.sockets.iter().map(move |fd| (*fd, events)))
        .filter(|(fd, _)| selectable(*fd, table))
        .collect()
}

/// `SockSelect([reads], [writes], [excepts] [, timeout])`.
pub(crate) fn select(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 4)?;
    let reads = set_of(interp, args, 1)?;
    let writes = set_of(interp, args, 2)?;
    let excepts = set_of(interp, args, 3)?;
    let timeout = match args.get(3).copied().flatten() {
        Some(_) => Some(double_argument(interp, args, 4)?),
        None => None,
    };
    let sets = [reads, writes, excepts];
    let polls = select_polls(&sets);
    let ready = any_ready(&polls);
    if ready || timeout.is_some_and(|seconds| seconds <= 0.0) {
        return Ok(finish_select(interp, &sets));
    }
    let deadline = timeout.map(|seconds| {
        interp.now() + Duration::try_from_secs_f64(seconds).unwrap_or(Duration::MAX / 4)
    });
    let kept: Vec<Option<ObjRef>> = sets
        .iter()
        .map(|set| set.as_ref().map(|set| set.stem.object))
        .collect();
    let wait = SocketWait {
        polls,
        deadline,
        op: Pending::Select { sets },
    };
    park(interp, wait, &kept)
}

fn finish_select(interp: &mut Interp, sets: &[Option<Set>; 3]) -> ObjRef {
    let table = descriptor_table_size();
    let polls = select_polls(sets);
    let Some(revents) = poll_now(&polls) else {
        return answer(interp, Err(EBADF));
    };
    let mut revents = revents.into_iter();
    let mut count = 0;
    let mut kept: Vec<Option<Vec<i32>>> = Vec::new();
    for (set, events) in sets.iter().zip(SET_EVENTS) {
        let Some(set) = set else {
            kept.push(None);
            continue;
        };
        let mut ready = Vec::new();
        let mut counted = Vec::new();
        for fd in set.sockets.iter().copied() {
            if !selectable(fd, table) {
                continue;
            }
            let revents = revents.next().unwrap_or_else(PollFlags::empty);
            if answers(events, revents) {
                ready.push(fd);
                if !counted.contains(&fd) {
                    counted.push(fd);
                }
            }
        }
        count += counted.len();
        kept.push(Some(ready));
    }
    if count > 0 {
        for (set, ready) in sets.iter().zip(kept) {
            let (Some(set), Some(ready)) = (set, ready) else {
                continue;
            };
            let total = i64::try_from(ready.len()).unwrap_or(i64::MAX);
            set.stem.set_number(interp, b"0", total);
            for (index, fd) in ready.into_iter().enumerate() {
                set.stem
                    .set_number(interp, (index + 1).to_string().as_bytes(), i64::from(fd));
            }
        }
    }
    answer(interp, Ok(i64::try_from(count).unwrap_or(i64::MAX)))
}

#[cfg(test)]
mod tests {
    use std::os::fd::AsRawFd as _;

    use super::{EAGAIN, blocks, try_accept, with_socket};

    /// A blocking listening socket whose connection another accept took is
    /// tried without blocking (PR 4 review M3): `EAGAIN` at once, the socket
    /// left blocking, and a later connection accepted.
    #[test]
    fn an_accept_that_finds_no_connection_does_not_block() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a listener");
        let fd = listener.as_raw_fd();
        assert_eq!(try_accept(fd).err(), Some(EAGAIN));
        assert_eq!(with_socket(fd, |socket| Ok(blocks(socket))), Ok(true));
        let _client = std::net::TcpStream::connect(listener.local_addr().expect("an address"))
            .expect("a connection");
        assert!(try_accept(fd).is_ok());
    }
}
