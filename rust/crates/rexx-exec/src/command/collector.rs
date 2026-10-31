/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

use super::{Piece, UNKNOWN_COMMAND, Waited, exit_code};
use std::io::Read;
use std::os::fd::AsFd;

/// A child's pieces' destination: the sink and where standard output and
/// standard error go, `None` for standard error kept whole.
type Routes<'a> = (&'a (dyn Fn(Piece, Vec<u8>) + Sync), Piece, Option<Piece>);

/// Reads `reader` to its end, into `kept`, or piece by piece into `written`
/// as `piece` where there is one.
///
/// With `ended`, the child's pidfd, reading stops once the child has ended,
/// which `ended` reports by becoming readable: what the pipe holds then is read,
/// and the rest, which only a process the command left running can write, is
/// read and dropped by a thread of its own, so that process's writes do not
/// fail. A command completes when its process does (rexxref intro.xml,
/// "Commands to External Environments": the environment "returns control to
/// the language processor after setting a return code").
fn read_all<R: Read + AsFd + Send + 'static>(
    mut reader: R,
    ended: Option<&std::os::fd::OwnedFd>,
    kept: &mut Vec<u8>,
    written: Option<(&(dyn Fn(Piece, Vec<u8>) + Sync), Piece)>,
) {
    let mut piece = [0u8; 4096];
    let mut take = |bytes: &[u8]| match written {
        Some((written, as_piece)) => written(as_piece, bytes.to_vec()),
        None => kept.extend_from_slice(bytes),
    };
    loop {
        if let Some(ended) = ended {
            use rustix::event::{PollFd, PollFlags, poll};
            let mut polled = [
                PollFd::new(&reader, PollFlags::IN),
                PollFd::new(ended, PollFlags::IN),
            ];
            match poll(&mut polled, None) {
                Ok(_) => {}
                Err(rustix::io::Errno::INTR) => continue,
                Err(_) => return,
            }
            if !polled[1].revents().is_empty() {
                let mut left = rustix::io::ioctl_fionread(&reader).unwrap_or(0) as usize;
                while left > 0 {
                    match reader.read(&mut piece[..left.min(4096)]) {
                        Ok(0) => return,
                        Ok(count) => {
                            take(&piece[..count]);
                            left -= count;
                        }
                        Err(failed) if failed.kind() == std::io::ErrorKind::Interrupted => {}
                        Err(_) => return,
                    }
                }
                std::thread::spawn(move || {
                    let _ = std::io::copy(&mut reader, &mut std::io::sink());
                });
                return;
            }
        }
        match reader.read(&mut piece) {
            Ok(0) => return,
            Ok(count) => take(&piece[..count]),
            Err(failed) if failed.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return,
        }
    }
}

/// Collects what `running` writes and its return code, writing `input` to
/// it first where there is one.
///
/// **Both pipes are drained concurrently**: a child filling one while this
/// thread reads only the other deadlocks once a pipe buffer fills.
///
/// `ends` says, for standard output and standard error, whether its pipe is
/// read only until the child ends ([`read_all`]): one going to this
/// interpreter's own output. A redirected one is read to its end, as the
/// oracle reads it.
pub(super) fn collect(
    mut running: std::process::Child,
    input: Option<Vec<u8>>,
    merged: Option<std::io::PipeReader>,
    ends: [bool; 2],
    routes: Option<Routes<'_>>,
) -> Waited {
    let stdin = running.stdin.take();
    let stdout = running.stdout.take();
    let stderr = running.stderr.take();
    let out_route = routes.map(|(written, out, _)| (written, out));
    let err_route = routes.and_then(|(written, _, err)| Some((written, err?)));
    // The readers poll it for the child's end, which reaps nothing; the reap
    // comes after them. Without one, every pipe is read to its end.
    let ended = i32::try_from(running.id())
        .ok()
        .and_then(rustix::process::Pid::from_raw)
        .and_then(|pid| {
            rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).ok()
        });
    let ended_reader = ended.as_ref();
    let (out, err) = std::thread::scope(|scope| {
        if let (Some(mut stdin), Some(bytes)) = (stdin, input.as_deref()) {
            // A write error is dropped: a child exiting before it reads its
            // input gives `EPIPE`, which the oracle's own writer ignores.
            scope.spawn(move || {
                use std::io::Write;
                let _ = stdin.write_all(bytes);
            });
        }
        let errors = scope.spawn(|| {
            let mut bytes = Vec::new();
            if let Some(stderr) = stderr {
                read_all(
                    stderr,
                    ended_reader.filter(|_| ends[1]),
                    &mut bytes,
                    err_route,
                );
            }
            bytes
        });
        let output = scope.spawn(|| {
            let mut bytes = Vec::new();
            match merged {
                Some(merged) => read_all(merged, None, &mut bytes, out_route),
                None => {
                    if let Some(stdout) = stdout {
                        read_all(
                            stdout,
                            ended_reader.filter(|_| ends[0]),
                            &mut bytes,
                            out_route,
                        );
                    }
                }
            }
            bytes
        });
        (
            output.join().unwrap_or_default(),
            errors.join().unwrap_or_default(),
        )
    });
    let rc = match running.wait() {
        Ok(status) => exit_code(status),
        Err(_) => UNKNOWN_COMMAND,
    };
    Waited { rc, out, err }
}

