//! RexxUtil routines for the Windows console screen buffer.

use super::{answer, c_string, check_arity, int_argument, optional_text, required};
use crate::{Failure, Interp, error::Raised};
use interrexx_core::ObjRef;
use windows_sys::Win32::Foundation::ERROR_INVALID_PARAMETER;

pub(crate) fn screen_size(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 5)?;
    let option = optional_text(interp, args, 1)?;
    let option: &[u8] = option.as_deref().map_or(b"BUFFERSIZE", c_string);
    let setting = args.iter().skip(1).any(Option::is_some);
    let result = match option.to_ascii_uppercase().as_slice() {
        b"BUFFERSIZE" => {
            check_arity(args, 3)?;
            if setting {
                let status = match dimensions(interp, args)? {
                    Some([rows, columns]) => crate::host::resize_console_buffer(rows, columns),
                    None => i64::from(ERROR_INVALID_PARAMETER),
                };
                return Ok(answer(interp, status));
            }
            let size = crate::host::console_info().unwrap_or_default().dwSize;
            format!("{} {}", size.Y, size.X)
        }
        b"WINDOWRECT" => {
            if setting {
                let status = match dimensions(interp, args)? {
                    Some(rect) => crate::host::set_console_window(rect),
                    None => i64::from(ERROR_INVALID_PARAMETER),
                };
                return Ok(answer(interp, status));
            }
            let rect = crate::host::console_info().unwrap_or_default().srWindow;
            format!("{} {} {} {}", rect.Top, rect.Left, rect.Bottom, rect.Right)
        }
        b"MAXWINDOWSIZE" => {
            check_arity(args, 1)?;
            let size = crate::host::console_info()
                .unwrap_or_default()
                .dwMaximumWindowSize;
            format!("{} {}", size.Y, size.X)
        }
        _ => {
            return Err(Raised::invalid_routine_option(
                "SysTextScreenSize",
                "option",
                "'BUFFERSIZE', 'WINDOWRECT' or 'MAXWINDOWSIZE'",
                option,
            )
            .into());
        }
    };
    Ok(interp.text_built(result.into_bytes()))
}

fn dimensions<const N: usize>(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<Option<[i16; N]>, Failure> {
    let mut dimensions = [0; N];
    let mut valid = true;
    for (index, dimension) in dimensions.iter_mut().enumerate() {
        let position = index + 2;
        let value = required(args, position)?;
        match i16::try_from(int_argument(interp, value, position)?) {
            Ok(value) => *dimension = value,
            Err(_) => valid = false,
        }
    }
    Ok(valid.then_some(dimensions))
}

pub(crate) fn cursor_position(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let previous = crate::host::console_info()
        .unwrap_or_default()
        .dwCursorPosition;
    if args.iter().any(Option::is_some)
        && let Some([row, column]) = coordinates(interp, args)?
    {
        crate::host::move_console_cursor(row, column);
    }
    Ok(interp.text_built(format!("{} {}", previous.Y, previous.X).into_bytes()))
}

pub(crate) fn cursor_state(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let state = super::required_c_string(interp, args, 1)?;
    let visible = match state.to_ascii_uppercase().as_slice() {
        b"ON" => true,
        b"OFF" => false,
        _ => {
            return Err(Raised::invalid_routine_option(
                "SysCurState",
                "state",
                "'ON' or 'OFF'",
                &state,
            )
            .into());
        }
    };
    Ok(answer(
        interp,
        crate::host::set_console_cursor_visible(visible),
    ))
}

pub(crate) fn screen_read(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 3)?;
    let point = coordinates(interp, args)?;
    let length = args
        .get(2)
        .copied()
        .flatten()
        .map(|length| int_argument(interp, length, 3))
        .transpose()?;
    let text = match (point, length) {
        (Some([row, column]), None) => crate::host::read_console_screen(row, column, u32::MAX),
        (Some([row, column]), Some(length)) if length > 0 => {
            crate::host::read_console_screen(row, column, u32::try_from(length).unwrap_or(u32::MAX))
        }
        _ => Vec::new(),
    };
    Ok(interp.text_built(text))
}

fn coordinates(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<Option<[i16; 2]>, Failure> {
    let row = required(args, 1)?;
    let row = int_argument(interp, row, 1)?;
    let column = required(args, 2)?;
    let column = int_argument(interp, column, 2)?;
    Ok(i16::try_from(row)
        .ok()
        .zip(i16::try_from(column).ok())
        .map(|(row, column)| [row, column]))
}
