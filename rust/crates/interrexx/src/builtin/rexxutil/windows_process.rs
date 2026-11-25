//! Windows process, thread and system routines from RexxUtil.

use interrexx_core::ObjRef;
use std::path::{Path, PathBuf};
use windows_sys::Win32::{
    Foundation::{ERROR_FILE_NOT_FOUND, ERROR_INVALID_PARAMETER},
    System::Threading::{
        HIGH_PRIORITY_CLASS, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, REALTIME_PRIORITY_CLASS,
    },
    UI::WindowsAndMessaging::{
        SW_HIDE, SW_MINIMIZE, SW_SHOWMAXIMIZED, SW_SHOWMINIMIZED, SW_SHOWMINNOACTIVE,
        SW_SHOWNOACTIVATE, SW_SHOWNORMAL,
    },
};

use super::{answer, c_string, check_arity, int_argument, optional_text, required_c_string};
use crate::builtin::platform::qualify_name;
use crate::error::Raised;
use crate::{Failure, Interp, host};

const PROCESS_PRIORITIES: &[(u32, &[u8])] = &[
    (IDLE_PRIORITY_CLASS, b"IDLE"),
    (NORMAL_PRIORITY_CLASS, b"NORMAL"),
    (HIGH_PRIORITY_CLASS, b"HIGH"),
    (REALTIME_PRIORITY_CLASS, b"REALTIME"),
];

const THREAD_PRIORITIES: &[(i32, &[u8])] = &[
    (-15, b"IDLE"),
    (-2, b"LOWEST"),
    (-1, b"BELOW_NORMAL"),
    (0, b"NORMAL"),
    (1, b"ABOVE_NORMAL"),
    (2, b"HIGHEST"),
    (15, b"TIME_CRITICAL"),
];

pub(crate) fn query_process(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let selector = optional_text(interp, args, 1)?;
    let selector = selector.as_deref().map_or(b"PID".as_slice(), c_string);
    let text = match selector.to_ascii_uppercase().as_slice() {
        b"PID" => return Ok(answer(interp, i64::from(std::process::id()))),
        b"TID" => return Ok(answer(interp, i64::from(host::thread_id()))),
        b"PPRIO" => priority_name(PROCESS_PRIORITIES, host::process_priority()).to_vec(),
        b"TPRIO" => priority_name(THREAD_PRIORITIES, host::thread_priority()).to_vec(),
        b"PTIME" | b"TTIME" => host::execution_times(selector.eq_ignore_ascii_case(b"TTIME"))
            .map_or_else(Vec::new, |[created, kernel, user]| {
                let date = host::local_file_time(created).unwrap_or_default();
                let duration = |ticks| format!("{}.{:07}", ticks / 10_000_000, ticks % 10_000_000);
                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02} {} {}",
                    date.wYear,
                    date.wMonth,
                    date.wDay,
                    date.wHour,
                    date.wMinute,
                    date.wSecond,
                    duration(kernel),
                    duration(user)
                )
                .into_bytes()
            }),
        _ => {
            return Err(Raised::invalid_routine_option(
                "SysQueryProcess",
                "selector",
                "PID, TID, PPRIO, TPRIO, PTIME or TTIME",
                selector,
            )
            .into());
        }
    };
    Ok(interp.text_built(text))
}

fn priority_name<T: PartialEq>(values: &'static [(T, &[u8])], priority: T) -> &'static [u8] {
    values
        .iter()
        .find(|(value, _)| *value == priority)
        .map_or(b"UNKNOWN", |(_, name)| *name)
}

pub(crate) fn set_priority(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let class = required_c_string(interp, args, 1)?;
    let delta = required_c_string(interp, args, 2)?;
    let symbolic = |text: &[u8], name: &[u8]| text.eq_ignore_ascii_case(name);
    let number = |text: &[u8]| std::str::from_utf8(text).ok()?.trim().parse::<i32>().ok();
    let class = PROCESS_PRIORITIES
        .iter()
        .enumerate()
        .find(|(index, (_, name))| symbolic(&class, name) || number(&class) == Some(*index as i32))
        .map(|(_, (priority, _))| *priority);
    let delta = THREAD_PRIORITIES
        .iter()
        .find(|(_, name)| symbolic(&delta, name))
        .map(|(value, _)| *value)
        .or_else(|| number(&delta))
        .filter(|delta| (-15..=15).contains(delta));
    let status = match (class, delta) {
        (Some(class), Some(delta)) => host::set_priority(class, delta),
        (None, _) => 307, // The RexxUtil invalid-priority-class status.
        (_, None) => ERROR_INVALID_PARAMETER,
    };
    Ok(answer(interp, i64::from(status)))
}

pub(crate) fn win_exec(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let command = required_c_string(interp, args, 1)?;
    let show = optional_text(interp, args, 2)?;
    let show = show.as_deref().map_or(b"SHOWNORMAL".as_slice(), c_string);
    let mode = match show.to_ascii_uppercase().as_slice() {
        b"SHOWNORMAL" => SW_SHOWNORMAL,
        b"SHOWNOACTIVATE" => SW_SHOWNOACTIVATE,
        b"SHOWMINNOACTIVE" => SW_SHOWMINNOACTIVE,
        b"SHOWMINIMIZED" => SW_SHOWMINIMIZED,
        b"SHOWMAXIMIZED" => SW_SHOWMAXIMIZED,
        b"HIDE" => SW_HIDE,
        b"MINIMIZE" => SW_MINIMIZE,
        _ => {
            return Err(Raised::invalid_routine_option(
                "RxWinExec", "window style",
                "SHOWNORMAL, SHOWNOACTIVATE, SHOWMINNOACTIVE, SHOWMINIMIZED, SHOWMAXIMIZED, HIDE or MINIMIZE",
                show,
            ).into());
        }
    };
    let outcome = std::str::from_utf8(&command)
        .map_err(|_| ERROR_INVALID_PARAMETER)
        .and_then(|command| {
            let command = command.trim_start();
            let image = executable_name(command).ok_or(ERROR_FILE_NOT_FOUND)?;
            let image = find_executable(interp, image).ok_or(ERROR_FILE_NOT_FOUND)?;
            host::launch_windows_program(
                &image,
                command,
                crate::paths::os_path(interp.cwd_bytes()),
                mode as u16,
                &interp.env,
            )
        })
        .map(i64::from)
        .unwrap_or_else(|code| {
            if code > 32 {
                -i64::from(code)
            } else {
                i64::from(code)
            }
        });
    Ok(answer(interp, outcome))
}

fn executable_name(command: &str) -> Option<&str> {
    let name = if let Some(quoted) = command.strip_prefix('"') {
        quoted.split_once('"')?.0
    } else {
        command.split_ascii_whitespace().next()?
    };
    (!name.is_empty()).then_some(name)
}

fn find_executable(interp: &Interp, name: &str) -> Option<PathBuf> {
    let name = Path::new(name);
    let name = if name.extension().is_none() {
        name.with_extension("exe")
    } else {
        name.to_path_buf()
    };
    let qualify = |path: &Path| qualify_name(interp, path.as_os_str().as_encoded_bytes());
    if name.components().count() != 1 || name.has_root() {
        let qualified = qualify(&name);
        let qualified = crate::paths::os_path(&qualified);
        return qualified.is_absolute().then(|| qualified.to_path_buf());
    }
    let paths = [
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf)),
        Some(crate::paths::os_path(interp.cwd_bytes()).to_path_buf()),
        host::system_directory().map(PathBuf::from),
        host::windows_directory().map(PathBuf::from),
    ];
    paths
        .into_iter()
        .flatten()
        .chain(
            interp
                .env_get(b"PATH")
                .and_then(crate::paths::os_str)
                .into_iter()
                .flat_map(std::env::split_paths),
        )
        .find_map(|dir| {
            let candidate = qualify(&dir.join(&name));
            let candidate = crate::paths::os_path(&candidate);
            candidate.is_file().then(|| candidate.to_path_buf())
        })
}

pub(crate) fn shutdown_system(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 5)?;
    let computer = optional_text(interp, args, 1)?;
    let message = optional_text(interp, args, 2)?;
    let timeout = match args.get(2).copied().flatten() {
        Some(timeout) => int_argument(interp, timeout, 3)?,
        None => 30,
    };
    let logical = |text: Option<Vec<u8>>| match text.as_deref() {
        None | Some(b"0") => Ok(false),
        Some(b"1") => Ok(true),
        Some(text) => Err(Raised::not_logical(text)),
    };
    let force = logical(optional_text(interp, args, 4)?)?;
    let reboot = logical(optional_text(interp, args, 5)?)?;
    let wide = |text: Option<&[u8]>| {
        let bytes = text.map(c_string).unwrap_or_default();
        std::str::from_utf8(bytes)
            .map(|text| text.encode_utf16().chain([0]).collect::<Vec<_>>())
            .ok()
    };
    let status = match (
        wide(computer.as_deref()),
        wide(message.as_deref()),
        u32::try_from(timeout),
    ) {
        (Some(computer), Some(message), Ok(timeout)) => {
            // The empty computer names the local machine.
            let computer = (computer.len() > 1).then_some(computer.as_slice());
            host::request_shutdown(computer, &message, timeout, force, reboot)
        }
        _ => ERROR_INVALID_PARAMETER,
    };
    Ok(answer(interp, i64::from(status)))
}
