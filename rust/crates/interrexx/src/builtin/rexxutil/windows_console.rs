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
