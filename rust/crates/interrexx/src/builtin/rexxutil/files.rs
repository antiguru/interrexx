//! The `REXXUTIL` routines over files and search paths.

use chrono::{Datelike as _, NaiveDate, TimeDelta, Timelike as _};
use interrexx_core::ObjRef;
use rustix::fs::{AtFlags, FileType, Mode, OFlags, Timespec, Timestamps};
use rustix::io::Errno;

use super::{
    answer, c_string, check_arity, optional_text, os_path, path_argument, required_c_string,
    scan_triple,
};
use crate::builtin::platform::qualify_name;
use crate::error::Raised;
use crate::{Failure, Interp};

/// `EEXIST`, which the copy and the move answer for a source and target that
/// are one file, and the move for a target that is a file already.
const EEXIST: i32 = libc::EEXIST;
const EISDIR: i32 = libc::EISDIR;

/// The type of whatever `path` names, following a symbolic link.
fn file_type(path: &[u8]) -> Option<FileType> {
    let stat = rustix::fs::stat(path).ok()?;
    Some(FileType::from_raw_mode(stat.st_mode))
}

/// Whether `path` names a regular file.
fn is_regular(path: &[u8]) -> bool {
    file_type(path) == Some(FileType::RegularFile)
}

/// Whether both names exist and resolve to one path. Every
/// file system is taken to be case-sensitive.
fn same_paths(first: &[u8], second: &[u8]) -> bool {
    let Ok(first) = std::fs::canonicalize(os_path(first)) else {
        return false;
    };
    std::fs::canonicalize(os_path(second)).is_ok_and(|second| first == second)
}

/// A whole second, as `utime` sets it.
fn whole_second(seconds: i64) -> Timespec {
    Timespec {
        tv_sec: seconds,
        tv_nsec: 0,
    }
}

/// Copy following links: `from`'s data written over `to`, a
/// link at either end followed. Answers `0` or the errno of the step that
/// failed, and a target opened before a failed read keeps what was written.
fn copy_following_links(from: &[u8], to: &[u8], preserve_mode: bool) -> i32 {
    if same_paths(from, to) {
        return EEXIST;
    }
    let source_stat = match rustix::fs::stat(from) {
        Ok(stat) => stat,
        Err(errno) => return errno.raw_os_error(),
    };
    // A directory is refused, and a target that is the source by device and
    // inode (a hard link, say) is refused, before the target is opened and
    // emptied.
    if FileType::from_raw_mode(source_stat.st_mode) == FileType::Directory {
        return EISDIR;
    }
    let target_stat = rustix::fs::stat(to).ok();
    if target_stat
        .as_ref()
        .is_some_and(|stat| stat.st_dev == source_stat.st_dev && stat.st_ino == source_stat.st_ino)
    {
        return EEXIST;
    }
    let source = match rustix::fs::open(from, OFlags::RDONLY, Mode::empty()) {
        Ok(fd) => fd,
        Err(errno) => return errno.raw_os_error(),
    };
    let created = target_stat.is_none();
    let flags = OFlags::WRONLY | OFlags::CREATE | OFlags::TRUNC;
    let target = match rustix::fs::open(to, flags, Mode::from_raw_mode(0o666)) {
        Ok(fd) => fd,
        Err(errno) => return errno.raw_os_error(),
    };
    let mut buffer = [0u8; 4096];
    loop {
        let count = match rustix::io::read(&source, &mut buffer) {
            Ok(0) => break,
            Ok(count) => count,
            Err(errno) => return errno.raw_os_error(),
        };
        if let Err(errno) = rustix::io::write(&target, &buffer[..count]) {
            return errno.raw_os_error();
        }
    }
    drop(source);
    drop(target);
    let times = Timestamps {
        last_access: whole_second(source_stat.st_atime),
        last_modification: whole_second(source_stat.st_mtime),
    };
    let _ = rustix::fs::utimensat(rustix::fs::CWD, to, &times, AtFlags::empty());
    if created || preserve_mode {
        let _ = rustix::fs::chmod(to, Mode::from_raw_mode(source_stat.st_mode));
    }
    0
}

/// Copy keeping links, as the move calls it, never
/// replacing a target: a link is copied as a link, and an existing target
/// where either end is a link answers `EEXIST`.
fn copy_keeping_links(from: &[u8], to: &[u8]) -> i32 {
    if same_paths(from, to) {
        return EEXIST;
    }
    let source = match rustix::fs::lstat(from) {
        Ok(stat) => FileType::from_raw_mode(stat.st_mode),
        Err(errno) => return errno.raw_os_error(),
    };
    let target = rustix::fs::lstat(to)
        .ok()
        .map(|stat| FileType::from_raw_mode(stat.st_mode));
    if let Some(target) = target
        && (source == FileType::Symlink || target == FileType::Symlink)
    {
        return EEXIST;
    }
    if source != FileType::Symlink {
        return copy_following_links(from, to, true);
    }
    let link = match rustix::fs::readlink(from, Vec::new()) {
        Ok(link) => link,
        Err(errno) => return errno.raw_os_error(),
    };
    match rustix::fs::symlink(link.as_c_str(), to) {
        Ok(()) => 0,
        Err(errno) => errno.raw_os_error(),
    }
}

/// `path` with six digits appended, naming no regular
/// file, or `None` where every one does.
fn spare_name(path: &[u8]) -> Option<Vec<u8>> {
    let start = random() % 1_000_000;
    let mut number = start;
    loop {
        let mut name = path.to_vec();
        name.extend_from_slice(format!("{number:06}").as_bytes());
        if !is_regular(&name) {
            return Some(name);
        }
        number = (number + 1) % 1_000_000;
        if number == start {
            return None;
        }
    }
}

/// A number from the process's random keys, for a name nobody predicts.
fn random() -> u64 {
    use std::hash::BuildHasher as _;
    std::hash::RandomState::new().hash_one(0u8)
}

/// `SysFileCopy(from, to)`: keeps the
/// source's times to the second and gives a target it creates the source's
/// mode. Answers `0` or an errno.
pub(crate) fn file_copy(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let from = path_argument(interp, args, 1)?;
    let to = path_argument(interp, args, 2)?;
    let code = copy_following_links(&from, &to, false);
    Ok(answer(interp, code.into()))
}

/// `SysFileMove(from, to)`. A target that is a
/// regular file already, or the source itself, is `EEXIST`. Across file
/// systems it copies, keeping links, and then unlinks the source, and a
/// failed unlink there answers `-1` rather than an errno.
pub(crate) fn file_move(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let from = path_argument(interp, args, 1)?;
    let to = path_argument(interp, args, 2)?;
    let code = move_file(&from, &to);
    Ok(answer(interp, code.into()))
}

/// [`file_move`]'s body.
fn move_file(from: &[u8], to: &[u8]) -> i32 {
    if same_paths(from, to) || is_regular(to) {
        return EEXIST;
    }
    finish_move(from, to, rustix::fs::rename(from, to))
}

fn finish_move(from: &[u8], to: &[u8], renamed: rustix::io::Result<()>) -> i32 {
    match renamed {
        Ok(()) => return 0,
        Err(errno) if errno != Errno::XDEV => return errno.raw_os_error(),
        Err(_) => {}
    }
    // The source must be one this process can unlink before anything is
    // copied, which a rename in place shows.
    let Some(spare) = spare_name(from) else {
        return EEXIST;
    };
    if let Err(errno) = rustix::fs::rename(from, spare.as_slice()) {
        return errno.raw_os_error();
    }
    if let Err(errno) = rustix::fs::rename(spare.as_slice(), from) {
        return errno.raw_os_error();
    }
    let copied = copy_keeping_links(from, to);
    if copied != 0 {
        return copied;
    }
    match rustix::fs::unlink(from) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// `SysIsFileDirectory(name)`: whether it names a directory, a link to one
/// included.
pub(crate) fn is_file_directory(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let path = path_argument(interp, args, 1)?;
    let found = file_type(&path) == Some(FileType::Directory);
    Ok(answer(interp, i64::from(found)))
}

/// `SysIsFileLink(name)`: whether the name itself is a symbolic link. The
/// name is qualified first, which keeps a trailing `/`, so `lstat` reads the
/// directory a link names rather than the link (`r[util.sysisfilelink.answer]`).
pub(crate) fn is_file_link(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let path = path_argument(interp, args, 1)?;
    let found = rustix::fs::lstat(path.as_slice())
        .is_ok_and(|stat| FileType::from_raw_mode(stat.st_mode) == FileType::Symlink);
    Ok(answer(interp, i64::from(found)))
}

/// A local time's fields as `struct tm` holds them: the full year, the month
/// from zero, and the rest as read.
#[derive(Clone, Copy)]
pub(crate) struct CivilTime {
    pub(crate) year: i64,
    pub(crate) month0: i64,
    pub(crate) day: i64,
    pub(crate) hour: i64,
    pub(crate) minute: i64,
    pub(crate) second: i64,
}

pub(crate) use super::timezone::program_zone;

/// The local time type `zone` has in force at `seconds` since the epoch.
fn zone_type(zone: &tz::TimeZone, seconds: i64) -> Option<tz::LocalTimeType> {
    zone.find_local_time_type(seconds).ok().copied()
}

/// `localtime`: the reading of `seconds` since the epoch in `zone`.
pub(crate) fn local_time(zone: &tz::TimeZone, seconds: i64) -> Option<CivilTime> {
    let offset = i64::from(zone_type(zone, seconds)?.ut_offset());
    let at = chrono::DateTime::from_timestamp(seconds.checked_add(offset)?, 0)?;
    Some(CivilTime {
        year: i64::from(at.year()),
        month0: i64::from(at.month0()),
        day: i64::from(at.day()),
        hour: i64::from(at.hour()),
        minute: i64::from(at.minute()),
        second: i64::from(at.second()),
    })
}

/// `mktime` with `tm_isdst` of `-1` in `zone`: fields past their range carry
/// into the next, and a reading the zone repeats is its earlier instant. A
/// reading the zone skips is taken at whichever of the two offsets around it
/// lands it in a local time type flagged daylight saving, which is glibc's
/// choice: the offset before the skip in most zones, the one after it where
/// the zone flags winter time as daylight saving (`Europe/Dublin`). `-1` where
/// the result is outside what the calendar holds.
fn make_time(zone: &tz::TimeZone, fields: CivilTime) -> i64 {
    let months = fields.year * 12 + fields.month0;
    let first = i32::try_from(months.div_euclid(12)).ok().and_then(|year| {
        let month = u32::try_from(months.rem_euclid(12) + 1).ok()?;
        NaiveDate::from_ymd_opt(year, month, 1)?.and_hms_opt(0, 0, 0)
    });
    let seconds =
        (fields.day - 1) * 86_400 + fields.hour * 3_600 + fields.minute * 60 + fields.second;
    let Some(local) =
        first.and_then(|first| first.checked_add_signed(TimeDelta::try_seconds(seconds)?))
    else {
        return -1;
    };
    let local = local.and_utc().timestamp();
    // The offsets a day either side: a reading a transition repeats or skips
    // lies between them.
    let offsets = [local - 86_400, local + 86_400]
        .map(|around| zone_type(zone, around).map(|found| i64::from(found.ut_offset())));
    let [Some(before), Some(after)] = offsets else {
        return -1;
    };
    let candidates = [local - before, local - after];
    let reads_back = |at: i64| {
        zone_type(zone, at).is_some_and(|found| i64::from(found.ut_offset()) == local - at)
    };
    if let Some(at) = candidates.into_iter().filter(|at| reads_back(*at)).min() {
        return at;
    }
    let daylight = |at: i64| zone_type(zone, at).is_some_and(|found| found.is_dst());
    match candidates.map(daylight) {
        [false, true] => candidates[1],
        _ => candidates[0],
    }
}

/// `SysGetFileDateTime(name [, selector])`: the local time of the last write,
/// or with a selector starting `A` the last access, as
/// `YYYY-MM-DD HH:MM:SS` with the year space-padded to four. `-1` where the
/// name does not resolve.
pub(crate) fn get_file_date_time(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let path = path_argument(interp, args, 1)?;
    let access = match optional_text(interp, args, 2)? {
        None => false,
        Some(text) => {
            let selector = c_string(&text);
            match selector.first().map(u8::to_ascii_uppercase) {
                Some(b'A') => true,
                Some(b'W') => false,
                _ => {
                    return Err(Raised::invalid_routine_option(
                        "SysGetFileDateTime",
                        "time selector",
                        "'A' or 'W'",
                        selector,
                    )
                    .into());
                }
            }
        }
    };
    let Ok(stat) = rustix::fs::stat(path.as_slice()) else {
        return Ok(answer(interp, -1));
    };
    let seconds = if access { stat.st_atime } else { stat.st_mtime };
    let Some(at) = local_time(&program_zone(interp), seconds) else {
        return Ok(answer(interp, -1));
    };
    let text = format!(
        "{:4}-{:02}-{:02} {:02}:{:02}:{:02}",
        at.year,
        at.month0 + 1,
        at.day,
        at.hour,
        at.minute,
        at.second
    );
    Ok(interp.text_built(text.into_bytes()))
}

/// `SysSetFileDateTime(name [, date] [, time])`: the last-write time set to
/// the local `date` (`YYYY-MM-DD`) and `time` (`HH:MM:SS`), either one
/// omitted keeping that half of the current value, both omitted meaning now.
/// The access time stays, to the second. `0`, or `-1` where the name does not
/// resolve, a given half does not scan, or the update fails.
pub(crate) fn set_file_date_time(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 3)?;
    let path = path_argument(interp, args, 1)?;
    let date = optional_text(interp, args, 2)?;
    let time = optional_text(interp, args, 3)?;
    let Ok(stat) = rustix::fs::stat(path.as_slice()) else {
        return Ok(answer(interp, -1));
    };
    let modified = if date.is_none() && time.is_none() {
        interp
            .wall_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| {
                i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
            })
    } else {
        let zone = program_zone(interp);
        let Some(mut fields) = local_time(&zone, stat.st_mtime) else {
            return Ok(answer(interp, -1));
        };
        if let Some(date) = date {
            let Some([year, month, day]) = scan_triple(c_string(&date), b'-', [4, 2, 2]) else {
                return Ok(answer(interp, -1));
            };
            (fields.year, fields.month0, fields.day) = (year, month - 1, day);
        }
        if let Some(time) = time {
            let Some([hour, minute, second]) = scan_triple(c_string(&time), b':', [2, 2, 2]) else {
                return Ok(answer(interp, -1));
            };
            (fields.hour, fields.minute, fields.second) = (hour, minute, second);
        }
        make_time(&zone, fields)
    };
    let times = Timestamps {
        last_access: whole_second(stat.st_atime),
        last_modification: whole_second(modified),
    };
    let set = rustix::fs::utimensat(rustix::fs::CWD, path.as_slice(), &times, AtFlags::empty());
    Ok(answer(interp, if set.is_ok() { 0 } else { -1 }))
}

/// `SysTempFileName(template [, filler])`. Each filler
/// character in the template becomes a digit, the first the lowest, from a
/// random start counting up until the qualified name names nothing. The null
/// string where the template has no filler or more than nine, or every name
/// is taken.
pub(crate) fn temp_file_name(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let template = required_c_string(interp, args, 1)?;
    let filler = match optional_text(interp, args, 2)? {
        None => b'?',
        Some(text) => match c_string(&text) {
            [filler] => *filler,
            other => {
                return Err(Raised::routine_argument_not_a_character(
                    "SysTempFileName",
                    "filler",
                    other,
                )
                .into());
            }
        },
    };
    let name = unique_name(interp, &template, filler);
    Ok(interp.text_built(name))
}

/// [`temp_file_name`]'s search.
fn unique_name(interp: &Interp, template: &[u8], filler: u8) -> Vec<u8> {
    let places = template.iter().filter(|byte| **byte == filler).count();
    if places == 0 || places > 9 {
        return Vec::new();
    }
    let limit = 10u64.pow(u32::try_from(places).expect("nine at most"));
    let start = random() % limit;
    let mut number = start;
    let mut candidate = template.to_vec();
    loop {
        let mut rest = number;
        for (at, byte) in template.iter().enumerate() {
            if *byte == filler {
                candidate[at] = b'0' + u8::try_from(rest % 10).expect("a digit");
                rest /= 10;
            }
        }
        let qualified = qualify_name(interp, &candidate);
        if rustix::fs::stat(qualified.as_slice()).is_err() {
            return qualified;
        }
        number = (number + 1) % limit;
        if number == start {
            return Vec::new();
        }
    }
}

#[cfg(test)]
mod tests;
