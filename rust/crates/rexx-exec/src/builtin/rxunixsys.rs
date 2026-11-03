//! The `rxunixsys` extension library, run in Rust: the Unix system routines
//! `docs/superpowers/specs/extensions-behaviour.md` ("rxunixsys") describes.
//!
//! Each routine makes the OS call its name says through `rustix`, or through
//! `nix` where `rustix` has no safe process-wide form, and answers the call's
//! result: 0 or a value on success, -1 on failure, never a condition. A
//! failing call's `errno` is kept on the activity for `SysGeterrno`
//! (`Activity::rxunixsys_errno`); a success leaves it as it was, as C's
//! `errno` is left. The user, group and service lookups set it to 0 first,
//! as the oracle does. A relative path is the interpreter's own directory's,
//! not the process's. `SysCrypt` is not provided (Rulings).

mod messages;
mod tzname;
mod wordexp;

use rexx_core::{BehaviourId, Body, Decoded, ObjRef};
use rexx_num::{DIGITS64, Number};

use super::rxmath::{check_arity, required};
use crate::error::Raised;
use crate::{Failure, Interp};

/// A routine body.
type Result<T> = std::result::Result<T, Failure>;

/// `Error 40.1: External routine "NAME" failed.`, NAME the routine as called.
pub(super) fn failed(interp: &Interp) -> Failure {
    Raised::routine_failed(&interp.native_frame().name.to_ascii_uppercase()).into()
}

/// A string argument as C reads it: the string value the native boundary
/// requests, 88.909 where there is none, cut at its first NUL.
pub(super) fn c_string(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Vec<u8>> {
    let value = required(args, position)?;
    let Some(text) = interp.blamed_string_conversion(value)? else {
        return Err(Raised::native_argument_needs_a_string_value(position).into());
    };
    let mut bytes = interp.to_text(text).into_owned();
    if let Some(end) = bytes.iter().position(|byte| *byte == 0) {
        bytes.truncate(end);
    }
    Ok(bytes)
}

/// A 32-bit integer argument: 88.907 for anything that is not a whole number
/// from -2147483648 to 2147483647.
pub(super) fn int(interp: &mut Interp, args: &[Option<ObjRef>], position: usize) -> Result<i32> {
    let value = required(args, position)?;
    let number = match value.decode() {
        Decoded::SmallInt(number) => Some(number),
        _ => match interp.blamed_string_conversion(value)? {
            Some(text) => Number::parse_bytes(&interp.to_text(text))
                .and_then(|number| number.int64_value(DIGITS64)),
            None => None,
        },
    };
    if let Some(number) = number.and_then(|number| i32::try_from(number).ok()) {
        return Ok(number);
    }
    let found = interp.native_found(value);
    Err(
        Raised::native_argument_outside_range(position, i32::MIN.into(), i32::MAX.into(), &found)
            .into(),
    )
}

/// The first character of an option, upper-cased, `None` for the null string.
fn option(text: &[u8]) -> Option<u8> {
    text.first().map(u8::to_ascii_uppercase)
}

/// A whole number as the routine's answer.
pub(super) fn number(interp: &mut Interp, value: impl Into<i128>) -> ObjRef {
    let value = value.into();
    match i64::try_from(value).ok().and_then(ObjRef::small_int) {
        Some(small) => small,
        None => interp.text_built(value.to_string().into_bytes()),
    }
}

/// A string answer.
fn string(interp: &mut Interp, bytes: &[u8]) -> ObjRef {
    interp.text_built(bytes.to_vec())
}

/// An `Array` of strings, as the native API's `NewArray` makes one.
fn array(interp: &mut Interp, items: Vec<Vec<u8>>) -> ObjRef {
    let frame = interp.roots.activity_mut().push_frame();
    let mut slots = Vec::with_capacity(items.len());
    for item in items {
        let value = interp.text_built(item);
        interp.roots.activity_mut().push_temp(value);
        slots.push(Some(value));
    }
    let object = interp.alloc_charged(
        BehaviourId::ARRAY,
        Body::Array {
            dimensions: None,
            slots: slots.into(),
        },
    );
    interp.roots.activity_mut().pop_frame(frame);
    object
}

/// `result`'s value, keeping a failure's `errno` for `SysGeterrno`.
fn kept<T, E: Into<i32>>(interp: &mut Interp, result: std::result::Result<T, E>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            interp.activity.rxunixsys_errno = error.into();
            None
        }
    }
}

/// An `errno` as [`kept`] records it.
struct Errno(i32);

impl From<Errno> for i32 {
    fn from(errno: Errno) -> i32 {
        errno.0
    }
}

fn os(error: rustix::io::Errno) -> Errno {
    Errno(error.raw_os_error())
}

fn nix_errno(error: nix::errno::Errno) -> Errno {
    Errno(error as i32)
}

/// A call's return code: 0, or -1 with its `errno` kept.
fn code(interp: &mut Interp, result: std::result::Result<(), Errno>) -> ObjRef {
    let code = if kept(interp, result).is_some() {
        0
    } else {
        -1
    };
    number(interp, code)
}

/// A call answering a value, or -1 with its `errno` kept.
fn value(interp: &mut Interp, result: std::result::Result<i64, Errno>) -> ObjRef {
    let answer = kept(interp, result).unwrap_or(-1);
    number(interp, answer)
}

/// `path` as the interpreter opens it: a relative name is its own
/// directory's. The null string stays null, which no call finds.
fn at_cwd(interp: &Interp, path: &[u8]) -> Vec<u8> {
    if path.is_empty() || path.first() == Some(&b'/') {
        return path.to_vec();
    }
    let mut out = interp.cwd_bytes().to_vec();
    out.push(b'/');
    out.extend_from_slice(path);
    out
}

/// A path argument resolved by [`at_cwd`].
fn path(interp: &mut Interp, args: &[Option<ObjRef>], position: usize) -> Result<Vec<u8>> {
    let text = c_string(interp, args, position)?;
    Ok(at_cwd(interp, &text))
}

fn pid(raw: i32) -> nix::unistd::Pid {
    nix::unistd::Pid::from_raw(raw)
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C passes the int argument as uid_t or gid_t, so -1 is all ones"
)]
fn unsigned(raw: i32) -> u32 {
    raw as u32
}

// --- Process and identity ---------------------------------------------------

pub(crate) fn getpid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    Ok(number(
        interp,
        rustix::process::getpid().as_raw_nonzero().get(),
    ))
}

pub(crate) fn getppid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    let parent = rustix::process::getppid().map_or(0, |parent| parent.as_raw_nonzero().get());
    Ok(number(interp, parent))
}

/// The calling thread's `pthread_self()`, unsigned.
pub(crate) fn gettid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    let thread = crate::signal::current_thread() as u64;
    Ok(number(interp, thread))
}

pub(crate) fn getuid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    Ok(number(interp, rustix::process::getuid().as_raw()))
}

pub(crate) fn geteuid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    Ok(number(interp, rustix::process::geteuid().as_raw()))
}

pub(crate) fn getgid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    Ok(number(interp, rustix::process::getgid().as_raw()))
}

pub(crate) fn getegid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    Ok(number(interp, rustix::process::getegid().as_raw()))
}

/// `SysSetuid(uid)`: drops the supplementary groups, then `setuid`.
/// Measured, oracle: setting the caller's own uid answers 0 with `errno` 1,
/// the `setgroups` refusal.
pub(crate) fn setuid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let uid = int(interp, args, 1)?;
    let dropped = nix::unistd::setgroups(&[]).map_err(nix_errno);
    kept(interp, dropped);
    let set = nix::unistd::setuid(nix::unistd::Uid::from_raw(unsigned(uid))).map_err(nix_errno);
    Ok(code(interp, set))
}

pub(crate) fn seteuid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let uid = int(interp, args, 1)?;
    let set = nix::unistd::seteuid(nix::unistd::Uid::from_raw(unsigned(uid))).map_err(nix_errno);
    Ok(code(interp, set))
}

pub(crate) fn setgid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let gid = int(interp, args, 1)?;
    let set = nix::unistd::setgid(nix::unistd::Gid::from_raw(unsigned(gid))).map_err(nix_errno);
    Ok(code(interp, set))
}

pub(crate) fn setegid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let gid = int(interp, args, 1)?;
    let set = nix::unistd::setegid(nix::unistd::Gid::from_raw(unsigned(gid))).map_err(nix_errno);
    Ok(code(interp, set))
}

pub(crate) fn setsid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    let made = nix::unistd::setsid()
        .map(|session| i64::from(session.as_raw()))
        .map_err(nix_errno);
    Ok(value(interp, made))
}

pub(crate) fn getsid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let of = int(interp, args, 1)?;
    let session = nix::unistd::getsid(Some(pid(of)))
        .map(|session| i64::from(session.as_raw()))
        .map_err(nix_errno);
    Ok(value(interp, session))
}

/// `SysSetpgrp()`: `setpgid(0, 0)`, which is what `setpgrp` is.
pub(crate) fn setpgrp(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    let set = nix::unistd::setpgid(pid(0), pid(0)).map_err(nix_errno);
    Ok(code(interp, set))
}

pub(crate) fn getpgrp(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    Ok(number(interp, nix::unistd::getpgrp().as_raw()))
}

pub(crate) fn setpgid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let of = int(interp, args, 1)?;
    let group = int(interp, args, 2)?;
    let set = nix::unistd::setpgid(pid(of), pid(group)).map_err(nix_errno);
    Ok(code(interp, set))
}

pub(crate) fn getpgid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let of = int(interp, args, 1)?;
    let group = nix::unistd::getpgid(Some(pid(of)))
        .map(|group| i64::from(group.as_raw()))
        .map_err(nix_errno);
    Ok(value(interp, group))
}

/// `SysKill(pid, sig)`: `kill` with the raw signal number, real-time
/// signals included (`crate::signal::send`). Signal 0 tests for the process.
/// The output written so far is handed over first, as before a command: the
/// signal may end this process, whose `SAY`s the oracle has already written.
/// A halting signal this process sent itself is served at once, as one a
/// command's child sent is at the command's end: measured, oracle, `SysKill`
/// of its own pid with SIGINT raises HALT before a following `SysSleep`
/// sleeps.
pub(crate) fn kill(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let of = int(interp, args, 1)?;
    let signal = int(interp, args, 2)?;
    interp.hand_over_output();
    let sent = crate::signal::unblocked(|| crate::signal::send(of, signal)).map_err(Errno);
    interp.serve_signal_now();
    Ok(code(interp, sent))
}

/// `SysSignal(sig, action)`: `D` sets the default action and `I` ignores the
/// signal, by the action's first character caselessly
/// (`crate::signal::set_disposition`); 0 whatever `signal` answered, its
/// refusal's `errno` kept. It replaces the interpreter's own handler for a
/// halting signal: measured, oracle and this crate alike, `SysKill` of the
/// process with SIGINT raises HALT before, is ignored after
/// `SysSignal(2, 'I')`, and ends the process (status 130) after
/// `SysSignal(2, 'D')`.
pub(crate) fn signal(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let signal = int(interp, args, 1)?;
    let action = c_string(interp, args, 2)?;
    let ignore = match option(&action) {
        Some(b'D') => false,
        Some(b'I') => true,
        _ => return Err(failed(interp)),
    };
    let set = crate::signal::set_disposition(signal, ignore).map_err(Errno);
    kept(interp, set);
    Ok(number(interp, 0))
}

/// `SysUmask(mask)`: sets the mask and answers the previous one.
pub(crate) fn umask(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let mask = int(interp, args, 1)?;
    let previous = rustix::process::umask(rustix::fs::Mode::from_raw_mode(unsigned(mask) & 0o777));
    Ok(number(interp, previous.as_raw_mode()))
}

// --- Files -------------------------------------------------------------------

pub(crate) fn symlink(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    // The target is stored as written, never resolved.
    let target = c_string(interp, args, 1)?;
    let link = path(interp, args, 2)?;
    let made = rustix::fs::symlink(target.as_slice(), link.as_slice()).map_err(os);
    Ok(code(interp, made))
}

pub(crate) fn link(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let old = path(interp, args, 1)?;
    let new = path(interp, args, 2)?;
    let made = rustix::fs::link(old.as_slice(), new.as_slice()).map_err(os);
    Ok(code(interp, made))
}

pub(crate) fn unlink(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let name = path(interp, args, 1)?;
    let removed = rustix::fs::unlink(name.as_slice()).map_err(os);
    Ok(code(interp, removed))
}

/// An owner or group argument: -1 leaves it unchanged.
fn owner(raw: i32) -> Option<u32> {
    (raw != -1).then(|| unsigned(raw))
}

fn chown_with(interp: &mut Interp, args: &[Option<ObjRef>], follow: bool) -> Result<ObjRef> {
    check_arity(args, 3)?;
    let name = path(interp, args, 1)?;
    let uid = owner(int(interp, args, 2)?).map(rustix::fs::Uid::from_raw);
    let gid = owner(int(interp, args, 3)?).map(rustix::fs::Gid::from_raw);
    let flags = if follow {
        rustix::fs::AtFlags::empty()
    } else {
        rustix::fs::AtFlags::SYMLINK_NOFOLLOW
    };
    let changed =
        rustix::fs::chownat(rustix::fs::CWD, name.as_slice(), uid, gid, flags).map_err(os);
    Ok(code(interp, changed))
}

pub(crate) fn chown(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    chown_with(interp, args, true)
}

pub(crate) fn lchown(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    chown_with(interp, args, false)
}

/// `SysChroot(path)`: the directory becomes `/` first, so a relative path is
/// read from there -- measured, oracle: `SysChroot('sub')` from a directory
/// holding `sub` fails with `ENOENT`, and `directory()` is `/` after it.
pub(crate) fn chroot(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let text = c_string(interp, args, 1)?;
    interp.set_cwd(std::path::PathBuf::from("/"));
    let name = at_cwd(interp, &text);
    let changed = rustix::process::chroot(name.as_slice()).map_err(os);
    Ok(code(interp, changed))
}

pub(crate) fn close(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let descriptor = int(interp, args, 1)?;
    let closed = nix::unistd::close(descriptor).map_err(nix_errno);
    Ok(code(interp, closed))
}

fn access_with(interp: &mut Interp, args: &[Option<ObjRef>], effective: bool) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let name = path(interp, args, 1)?;
    let mode = rustix::fs::Access::from_bits_retain(unsigned(int(interp, args, 2)?));
    let flags = if effective {
        rustix::fs::AtFlags::EACCESS
    } else {
        rustix::fs::AtFlags::empty()
    };
    let answered = rustix::fs::accessat(rustix::fs::CWD, name.as_slice(), mode, flags).map_err(os);
    Ok(code(interp, answered))
}

pub(crate) fn access(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    access_with(interp, args, false)
}

pub(crate) fn euidaccess(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    access_with(interp, args, true)
}

/// `SysMkdirUnix(dir, mode)`, the mode under the umask.
pub(crate) fn mkdir(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let name = path(interp, args, 1)?;
    let mode = rustix::fs::Mode::from_raw_mode(unsigned(int(interp, args, 2)?));
    let made = rustix::fs::mkdir(name.as_slice(), mode).map_err(os);
    Ok(code(interp, made))
}

pub(crate) fn rmdir(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let name = path(interp, args, 1)?;
    let removed = rustix::fs::rmdir(name.as_slice()).map_err(os);
    Ok(code(interp, removed))
}

/// The mode `SysChmod`'s nine characters spell, or `None` where one is not
/// from its position's set.
fn permissions(text: &[u8]) -> Option<u32> {
    let [ur, uw, ux, gr, gw, gx, or, ow, ox] = <[u8; 9]>::try_from(text).ok()?;
    let mut mode = 0;
    for (byte, wanted, bit) in [
        (ur, b'r', 0o400),
        (uw, b'w', 0o200),
        (gr, b'r', 0o040),
        (gw, b'w', 0o020),
        (or, b'r', 0o004),
        (ow, b'w', 0o002),
    ] {
        match byte {
            b'-' => {}
            _ if byte == wanted => mode |= bit,
            _ => return None,
        }
    }
    for (byte, execute, special, set, unset) in [
        (ux, 0o100, 0o4000, b's', b'S'),
        (gx, 0o010, 0o2000, b's', b'S'),
        (ox, 0o001, 0o1000, b't', b'T'),
    ] {
        mode |= match byte {
            b'-' => 0,
            b'x' => execute,
            _ if byte == set => execute | special,
            _ if byte == unset => special,
            _ => return None,
        };
    }
    Some(mode)
}

pub(crate) fn chmod(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let name = path(interp, args, 1)?;
    let text = c_string(interp, args, 2)?;
    let Some(mode) = permissions(&text) else {
        return Err(failed(interp));
    };
    let changed =
        rustix::fs::chmod(name.as_slice(), rustix::fs::Mode::from_raw_mode(mode)).map_err(os);
    Ok(code(interp, changed))
}

/// The ten characters `ls -l` shows for `mode`.
fn permission_text(mode: u32) -> Vec<u8> {
    let kind = match mode & 0o170_000 {
        0o040_000 => b'd',
        0o120_000 => b'l',
        0o140_000 => b's',
        0o020_000 => b'c',
        0o060_000 => b'b',
        0o010_000 => b'p',
        _ => b'-',
    };
    let bit = |mask: u32, on: u8| if mode & mask != 0 { on } else { b'-' };
    let special =
        |execute: u32, flag: u32, set: u8, unset: u8| match (mode & execute != 0, mode & flag != 0)
        {
            (true, true) => set,
            (false, true) => unset,
            (true, false) => b'x',
            (false, false) => b'-',
        };
    vec![
        kind,
        bit(0o400, b'r'),
        bit(0o200, b'w'),
        special(0o100, 0o4000, b's', b'S'),
        bit(0o040, b'r'),
        bit(0o020, b'w'),
        special(0o010, 0o2000, b's', b'S'),
        bit(0o004, b'r'),
        bit(0o002, b'w'),
        special(0o001, 0o1000, b't', b'T'),
    ]
}

/// `seconds` since the epoch in the program's local time, as
/// `YYYY-MM-DD HH:MM:SS`.
fn local_stamp(interp: &Interp, seconds: i64) -> Vec<u8> {
    let zone = super::rexxutil::files::program_zone(interp);
    let Some(at) = super::rexxutil::files::local_time(&zone, seconds) else {
        return Vec::new();
    };
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        at.year,
        at.month0 + 1,
        at.day,
        at.hour,
        at.minute,
        at.second
    )
    .into_bytes()
}

/// `SysStat(path, option)`.
pub(crate) fn stat(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let name = c_string(interp, args, 1)?;
    let which = c_string(interp, args, 2)?;
    if name.is_empty() || which.is_empty() {
        return Err(failed(interp));
    }
    let name = at_cwd(interp, &name);
    let found = rustix::fs::stat(name.as_slice()).map_err(os);
    let Some(stat) = kept(interp, found) else {
        return Ok(string(interp, b""));
    };
    let answer = match option(&which) {
        Some(b'D') => number(interp, stat.st_dev),
        Some(b'I') => number(interp, stat.st_ino),
        Some(b'N') => number(interp, stat.st_nlink),
        Some(b'U') => number(interp, stat.st_uid),
        Some(b'G') => number(interp, stat.st_gid),
        Some(b'R') => number(interp, stat.st_rdev),
        Some(b'S') => number(interp, stat.st_size),
        Some(b'A') => {
            let text = local_stamp(interp, stat.st_atime);
            string(interp, &text)
        }
        Some(b'M') => {
            let text = local_stamp(interp, stat.st_mtime);
            string(interp, &text)
        }
        Some(b'C') => {
            let text = local_stamp(interp, stat.st_ctime);
            string(interp, &text)
        }
        Some(b'P') => {
            let text = permission_text(stat.st_mode);
            string(interp, &text)
        }
        _ => return Err(failed(interp)),
    };
    Ok(answer)
}

/// `SysGetdirlist(dir)`: every entry the directory stream returns, `.` and
/// `..` included, in its order; an empty `Array` where it cannot be opened.
pub(crate) fn getdirlist(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let name = path(interp, args, 1)?;
    let flags = rustix::fs::OFlags::RDONLY
        | rustix::fs::OFlags::DIRECTORY
        | rustix::fs::OFlags::NONBLOCK
        | rustix::fs::OFlags::CLOEXEC;
    let opened = rustix::fs::open(name.as_slice(), flags, rustix::fs::Mode::empty())
        .and_then(rustix::fs::Dir::read_from)
        .map_err(os);
    let mut names = Vec::new();
    if let Some(mut directory) = kept(interp, opened) {
        while let Some(entry) = directory.read() {
            let Some(entry) = kept(interp, entry.map_err(os)) else {
                break;
            };
            names.push(entry.file_name().to_bytes().to_vec());
        }
    }
    Ok(array(interp, names))
}

// --- Users, groups and services --------------------------------------------

/// A user's entry by `option`'s first character, or `None` for a character
/// no field has.
fn user_field(user: &nix::unistd::User, option: u8) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt as _;
    Some(match option {
        b'N' => user.name.clone().into_bytes(),
        b'U' => user.uid.as_raw().to_string().into_bytes(),
        b'G' => user.gid.as_raw().to_string().into_bytes(),
        b'R' => user.gecos.as_bytes().to_vec(),
        b'D' => user.dir.as_os_str().as_bytes().to_vec(),
        b'S' => user.shell.as_os_str().as_bytes().to_vec(),
        b'P' => Vec::new(),
        _ => return None,
    })
}

fn user_answer(
    interp: &mut Interp,
    found: nix::Result<Option<nix::unistd::User>>,
    which: &[u8],
) -> Result<ObjRef> {
    interp.activity.rxunixsys_errno = 0;
    let Some(Some(user)) = kept(interp, found.map_err(nix_errno)) else {
        return Ok(string(interp, b""));
    };
    match option(which).and_then(|option| user_field(&user, option)) {
        Some(text) => Ok(string(interp, &text)),
        None => Err(failed(interp)),
    }
}

/// `SysGetpwnam(user, option)`.
pub(crate) fn getpwnam(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let name = c_string(interp, args, 1)?;
    let which = c_string(interp, args, 2)?;
    if name.is_empty() || which.is_empty() {
        return Err(failed(interp));
    }
    let found = match std::str::from_utf8(&name) {
        Ok(name) => nix::unistd::User::from_name(name),
        Err(_) => Ok(None),
    };
    user_answer(interp, found, &which)
}

/// `SysGetpwuid(uid, option)`.
pub(crate) fn getpwuid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let uid = int(interp, args, 1)?;
    let which = c_string(interp, args, 2)?;
    if which.is_empty() {
        return Err(failed(interp));
    }
    let found = nix::unistd::User::from_uid(nix::unistd::Uid::from_raw(unsigned(uid)));
    user_answer(interp, found, &which)
}

/// A group's entry by `option`: `N`, `G`, `P` or `M`.
fn group_answer(
    interp: &mut Interp,
    found: nix::Result<Option<nix::unistd::Group>>,
    which: &[u8],
) -> Result<ObjRef> {
    let option = option(which);
    if !matches!(option, Some(b'N' | b'G' | b'P' | b'M')) {
        return Err(failed(interp));
    }
    interp.activity.rxunixsys_errno = 0;
    // Deviation 88: the oracle crashes on a group that does not exist.
    let Some(Some(group)) = kept(interp, found.map_err(nix_errno)) else {
        return Err(failed(interp));
    };
    Ok(match option {
        Some(b'N') => string(interp, group.name.as_bytes()),
        Some(b'G') => number(interp, group.gid.as_raw()),
        Some(b'M') => {
            let members = group.mem.into_iter().map(String::into_bytes).collect();
            array(interp, members)
        }
        _ => string(interp, b""),
    })
}

/// `SysGetgrnam(group, option)`.
pub(crate) fn getgrnam(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let name = c_string(interp, args, 1)?;
    let which = c_string(interp, args, 2)?;
    if name.is_empty() || which.is_empty() {
        return Err(failed(interp));
    }
    let found = match std::str::from_utf8(&name) {
        Ok(name) => nix::unistd::Group::from_name(name),
        Err(_) => Ok(None),
    };
    group_answer(interp, found, &which)
}

/// `SysGetgrgid(gid, option)`.
pub(crate) fn getgrgid(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let gid = int(interp, args, 1)?;
    let which = c_string(interp, args, 2)?;
    if which.is_empty() {
        return Err(failed(interp));
    }
    let found = nix::unistd::Group::from_gid(nix::unistd::Gid::from_raw(unsigned(gid)));
    group_answer(interp, found, &which)
}

/// One service of the services database: its official name, port and
/// aliases.
struct Service {
    name: Vec<u8>,
    port: u16,
    aliases: Vec<Vec<u8>>,
}

/// The first line of `/etc/services`, the files source glibc's
/// `getservbyname` and `getservbyport` read, for protocol `protocol` that
/// `wanted` accepts.
fn service(protocol: &[u8], wanted: impl Fn(&Service) -> bool) -> Option<Service> {
    let table = std::fs::read("/etc/services").ok()?;
    for line in table.split(|byte| *byte == b'\n') {
        let line = line.split(|byte| *byte == b'#').next().unwrap_or_default();
        let mut fields = line
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|field| !field.is_empty());
        let (Some(name), Some(entry)) = (fields.next(), fields.next()) else {
            continue;
        };
        let Some(slash) = entry.iter().position(|byte| *byte == b'/') else {
            continue;
        };
        if &entry[slash + 1..] != protocol {
            continue;
        }
        let Some(port) = std::str::from_utf8(&entry[..slash])
            .ok()
            .and_then(|port| port.parse().ok())
        else {
            continue;
        };
        let found = Service {
            name: name.to_vec(),
            port,
            aliases: fields.map(<[u8]>::to_vec).collect(),
        };
        if wanted(&found) {
            return Some(found);
        }
    }
    None
}

fn service_answer(interp: &mut Interp, found: Option<Service>, which: &[u8]) -> Result<ObjRef> {
    interp.activity.rxunixsys_errno = 0;
    let Some(found) = found else {
        return Ok(string(interp, b""));
    };
    Ok(match option(which) {
        Some(b'N') => string(interp, &found.name),
        Some(b'P') => number(interp, found.port),
        Some(b'A') => array(interp, found.aliases),
        _ => return Err(failed(interp)),
    })
}

/// `SysGetservbyname(name, proto, option)`: by official name or alias.
pub(crate) fn getservbyname(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 3)?;
    let name = c_string(interp, args, 1)?;
    let protocol = c_string(interp, args, 2)?;
    let which = c_string(interp, args, 3)?;
    if name.is_empty() || protocol.is_empty() {
        return Err(failed(interp));
    }
    let found = service(&protocol, |found| {
        found.name == name || found.aliases.contains(&name)
    });
    service_answer(interp, found, &which)
}

/// `SysGetservbyport(port, proto, option)`: a port from 1 to 65534.
pub(crate) fn getservbyport(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 3)?;
    let port = int(interp, args, 1)?;
    let protocol = c_string(interp, args, 2)?;
    let which = c_string(interp, args, 3)?;
    if port <= 0 || port >= 65535 || protocol.is_empty() {
        return Err(failed(interp));
    }
    let found = service(&protocol, |found| i32::from(found.port) == port);
    service_answer(interp, found, &which)
}

// --- System ------------------------------------------------------------------

/// `SysUname([option])`: the system name where omitted.
pub(crate) fn uname(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let which = match args.first().copied().flatten() {
        Some(_) => option(&c_string(interp, args, 1)?),
        None => Some(b'S'),
    };
    let info = rustix::system::uname();
    let field = match which {
        Some(b'S') => info.sysname(),
        Some(b'N') => info.nodename(),
        Some(b'R') => info.release(),
        Some(b'V') => info.version(),
        Some(b'M') => info.machine(),
        _ => return Err(failed(interp)),
    };
    Ok(string(interp, field.to_bytes()))
}

pub(crate) fn gethostname(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    let info = rustix::system::uname();
    Ok(string(interp, info.nodename().to_bytes()))
}

pub(crate) fn getsizeofptr(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    Ok(number(interp, usize::BITS))
}

fn tzname(interp: &mut Interp, args: &[Option<ObjRef>], daylight: bool) -> Result<ObjRef> {
    check_arity(args, 0)?;
    let (standard, summer) = tzname::names(interp.env_get(b"TZ"), interp.env_get(b"TZDIR"));
    Ok(string(interp, if daylight { &summer } else { &standard }))
}

pub(crate) fn gettzname1(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    tzname(interp, args, false)
}

pub(crate) fn gettzname2(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    tzname(interp, args, true)
}

/// `SysWordexp(text)`; see [`wordexp`].
pub(crate) fn wordexp(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let text = c_string(interp, args, 1)?;
    if text.is_empty() {
        return Err(failed(interp));
    }
    match wordexp::expand(interp, &text) {
        Some(words) => Ok(array(interp, words)),
        None => Err(failed(interp)),
    }
}

/// `SysSetxattr(path, name, value)`: the value with a trailing NUL.
pub(crate) fn setxattr(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 3)?;
    let name = path(interp, args, 1)?;
    let attribute = c_string(interp, args, 2)?;
    let mut stored = c_string(interp, args, 3)?;
    stored.push(0);
    let set = rustix::fs::setxattr(
        name.as_slice(),
        attribute.as_slice(),
        &stored,
        rustix::fs::XattrFlags::empty(),
    )
    .map_err(os);
    Ok(code(interp, set))
}

/// What `read` fills a buffer with, the buffer doubled for as long as the
/// call answers `ERANGE`, up to the 64 KiB an attribute holds at most.
fn grown(read: impl Fn(&mut Vec<u8>) -> rustix::io::Result<usize>) -> rustix::io::Result<Vec<u8>> {
    let mut size = 256;
    loop {
        let mut buffer = vec![0; size];
        match read(&mut buffer) {
            Ok(length) => {
                buffer.truncate(length);
                return Ok(buffer);
            }
            Err(rustix::io::Errno::RANGE) if size < 1 << 17 => size *= 2,
            Err(error) => return Err(error),
        }
    }
}

/// `SysGetxattr(path, name)`: the value up to its first NUL, `""` where it
/// cannot be read.
pub(crate) fn getxattr(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let name = path(interp, args, 1)?;
    let attribute = c_string(interp, args, 2)?;
    let read = grown(|buffer| {
        rustix::fs::getxattr(name.as_slice(), attribute.as_slice(), &mut buffer[..])
    })
    .map_err(os);
    let mut text = kept(interp, read).unwrap_or_default();
    if let Some(end) = text.iter().position(|byte| *byte == 0) {
        text.truncate(end);
    }
    Ok(string(interp, &text))
}

/// `SysListxattr(path)`: an `Array` of the names, or `""` (a string) where
/// they cannot be listed.
pub(crate) fn listxattr(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let name = path(interp, args, 1)?;
    let listed =
        grown(|buffer| rustix::fs::listxattr(name.as_slice(), &mut buffer[..])).map_err(os);
    let Some(list) = kept(interp, listed) else {
        return Ok(string(interp, b""));
    };
    let names = list
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(<[u8]>::to_vec)
        .collect();
    Ok(array(interp, names))
}

pub(crate) fn removexattr(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 2)?;
    let name = path(interp, args, 1)?;
    let attribute = c_string(interp, args, 2)?;
    let removed = rustix::fs::removexattr(name.as_slice(), attribute.as_slice()).map_err(os);
    Ok(code(interp, removed))
}

/// `SysGeterrno()`: the `errno` of this library's last failing call on this
/// activity, 0 before any.
pub(crate) fn geterrno(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 0)?;
    let errno = interp.activity.rxunixsys_errno;
    Ok(number(interp, errno))
}

/// `SysGeterrnomsg(n)`: the message for `n`, `Unknown` outside the table.
pub(crate) fn geterrnomsg(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef> {
    check_arity(args, 1)?;
    let errno = int(interp, args, 1)?;
    let message = usize::try_from(errno)
        .ok()
        .and_then(|index| messages::MESSAGES.get(index))
        .copied()
        .unwrap_or("Unknown");
    Ok(string(interp, message.as_bytes()))
}

#[cfg(test)]
mod tests;
