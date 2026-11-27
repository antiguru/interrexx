use super::*;
use interrexx_cast::CastFrom;

/// Up to `len` bytes: from `position` (1-based) of a persistent stream, or
/// the next ones of a transient stream, which has no position and waits
/// until `len` arrive or its input ends (rexxref funct.xml:1329-1335). A
/// byte [`transient_has_data`] took comes first.
pub(super) fn read_at(
    open: &mut interrexx_core::OpenFile,
    position: u64,
    len: usize,
) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    if !open.transient {
        return read_from(open.flushed(), position, len);
    }
    let mut out = Vec::new();
    let mut len = len;
    if len > 0
        && let Some(byte) = open.pending.take()
    {
        out.push(byte);
        len -= 1;
    }
    let mut chunk = [0; CHUNK];
    let file = open.flushed();
    while len > 0 {
        let wanted = len.min(chunk.len());
        let read = file.read(&mut chunk[..wanted])?;
        if read == 0 {
            break;
        }
        out.extend_from_slice(&chunk[..read]);
        len -= read;
    }
    Ok(out)
}

/// Whether data waits on a transient stream, found without reading it: the
/// byte count `FIONREAD` reports for a pipe, FIFO or terminal (rexxref
/// iostrms.xml:189-193) where it is above 0. A device that refuses
/// `FIONREAD`, or whose count the host does not trust
/// (`host::bytes_waiting`), is asked by a `poll`: one that finds nothing
/// readable is 0, and otherwise
/// one byte is read and kept for the next read, which returns it first; a
/// read at the end of input finds none, and is 0. A state that cannot be
/// learned answers 1.
#[cfg(unix)]
pub(super) fn transient_has_data(open: &mut interrexx_core::OpenFile) -> bool {
    use rustix::event::{PollFd, PollFlags, Timespec, poll};
    use std::io::Read;
    use std::os::fd::AsFd as _;
    if open.pending.is_some() {
        return true;
    }
    // A trusted count answers without reading: a probe read would use up a
    // terminal's end of file, which the next read must still see.
    if let Some(waiting) = crate::host::bytes_waiting(open.file().as_fd()) {
        return waiting > 0;
    }
    let zero = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let mut polled = [PollFd::new(open.file(), PollFlags::IN)];
    match poll(&mut polled, Some(&zero)) {
        Ok(_) if !polled[0].revents().contains(PollFlags::IN) => return false,
        Ok(_) => {}
        Err(_) => return true,
    }
    let mut byte = [0u8];
    match open.flushed().read(&mut byte) {
        Ok(0) => false,
        Ok(_) => {
            open.pending = Some(byte[0]);
            true
        }
        Err(_) => true,
    }
}

/// One line from `position`, and the position after it. The line drops its
/// terminating `\n` and the `\r` immediately before that -- a bare `\r` is
/// data (`SysFile::gets`) -- and a final unterminated line is still a line.
/// `None` at the end of the file. A transient stream is read a byte at a
/// time, so nothing after the line end is taken from it.
pub(super) fn read_line_from(
    open: &mut interrexx_core::OpenFile,
    position: u64,
) -> std::io::Result<Option<(Vec<u8>, i64)>> {
    let mut line = Vec::new();
    let mut at = position;
    let want = if open.transient { 1 } else { CHUNK };
    loop {
        let chunk = read_at(open, at, want)?;
        if chunk.is_empty() {
            return Ok(if line.is_empty() {
                None
            } else {
                Some((line, as_position(at)))
            });
        }
        match chunk.iter().position(|byte| *byte == b'\n') {
            Some(end) => {
                line.extend_from_slice(&chunk[..end]);
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                return Ok(Some((line, as_position(at + u64::cast_from(end) + 1))));
            }
            None => {
                line.extend_from_slice(&chunk);
                at += u64::cast_from(chunk.len());
            }
        }
    }
}

#[cfg(windows)]
pub(super) fn transient_has_data(open: &interrexx_core::OpenFile) -> bool {
    open.pending.is_some() || crate::host::bytes_waiting(open.file()).is_none_or(|bytes| bytes > 0)
}
