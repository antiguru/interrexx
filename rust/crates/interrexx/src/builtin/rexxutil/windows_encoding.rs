//! RexxUtil's Windows code page conversions. Rexx strings carry the UTF-16
//! result as bytes, independently of the code page used by the console.

use interrexx_core::ObjRef;
use windows_sys::Win32::{
    Foundation::{ERROR_INVALID_FLAGS, ERROR_INVALID_PARAMETER},
    Globalization::{
        CP_ACP, CP_OEMCP, CP_SYMBOL, CP_THREAD_ACP, CP_UTF7, CP_UTF8, MB_COMPOSITE,
        MB_ERR_INVALID_CHARS, MB_PRECOMPOSED, MB_USEGLYPHCHARS, WC_COMPOSITECHECK, WC_DEFAULTCHAR,
        WC_DISCARDNS, WC_SEPCHARS,
    },
};

use super::{answer, c_string, check_arity, optional_text, required_text, stems::stem_argument};
use crate::{Failure, Interp, host};

pub(crate) fn to_unicode(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 4)?;
    let text = required_text(interp, args, 1)?;
    let code_page = optional_text(interp, args, 2)?;
    let options = optional_text(interp, args, 3)?;
    let stem = stem_argument(interp, args, 4)?;
    let converted = decode(&text, code_page.as_deref(), options.as_deref());
    match converted {
        Ok(wide) => {
            let bytes = wide.into_iter().flat_map(u16::to_le_bytes).collect();
            let text = interp.text_built(bytes);
            interp.stem_object_write(stem, b"!TEXT", text);
            Ok(answer(interp, 0))
        }
        Err(status) => Ok(answer(interp, i64::from(status))),
    }
}

pub(crate) fn from_unicode(
    interp: &mut Interp,
    _: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 5)?;
    let text = required_text(interp, args, 1)?;
    let code_page = optional_text(interp, args, 2)?;
    let options = optional_text(interp, args, 3)?;
    let default = optional_text(interp, args, 4)?;
    let stem = stem_argument(interp, args, 5)?;
    let converted = encode(
        &text,
        code_page.as_deref(),
        options.as_deref(),
        default.as_deref(),
    );
    match converted {
        Ok((bytes, used_default)) => {
            let text = interp.text_built(bytes);
            interp.stem_object_write(stem, b"!TEXT", text);
            let used = answer(interp, i64::from(used_default));
            interp.stem_object_write(stem, b"!USEDDEFAULTCHAR", used);
            Ok(answer(interp, 0))
        }
        Err(status) => Ok(answer(interp, i64::from(status))),
    }
}

fn code_page(option: Option<&[u8]>) -> Result<u32, u32> {
    match option
        .map(c_string)
        .map(<[u8]>::to_ascii_uppercase)
        .as_deref()
    {
        None | Some(b"OEMCP") => Ok(CP_OEMCP),
        Some(b"ACP") => Ok(CP_ACP),
        Some(b"SYMBOL") => Ok(CP_SYMBOL),
        Some(b"THREAD_ACP") => Ok(CP_THREAD_ACP),
        Some(b"UTF7") => Ok(CP_UTF7),
        Some(b"UTF8") => Ok(CP_UTF8),
        Some(number) => std::str::from_utf8(number)
            .ok()
            .and_then(|digits| digits.parse().ok())
            .ok_or(ERROR_INVALID_PARAMETER),
    }
}

fn flags(option: Option<&[u8]>, names: &[(&[u8], u32)]) -> Result<u32, u32> {
    let option = option.map(c_string).unwrap_or_default();
    option
        .split(|byte| byte.is_ascii_whitespace() || matches!(byte, b'|' | b',' | b'+'))
        .filter(|word| !word.is_empty())
        .try_fold(0, |flags, word| {
            if word == b"0" {
                return Ok(flags);
            }
            names
                .iter()
                .find(|(name, _)| word.eq_ignore_ascii_case(name))
                .map(|(_, flag)| flags | flag)
                .ok_or(ERROR_INVALID_FLAGS)
        })
}

fn decode(text: &[u8], page: Option<&[u8]>, options: Option<&[u8]>) -> Result<Vec<u16>, u32> {
    let page = code_page(page)?;
    let flags = flags(
        options,
        &[
            (b"PRECOMPOSED", MB_PRECOMPOSED),
            (b"COMPOSITE", MB_COMPOSITE),
            (b"ERR_INVALID_CHARS", MB_ERR_INVALID_CHARS),
            (b"USEGLYPHCHARS", MB_USEGLYPHCHARS),
        ],
    )?;
    host::decode_code_page(page, flags, text)
}

fn encode(
    text: &[u8],
    page: Option<&[u8]>,
    options: Option<&[u8]>,
    default: Option<&[u8]>,
) -> Result<(Vec<u8>, bool), u32> {
    let page = code_page(page)?;
    let flags = flags(
        options,
        &[
            (b"COMPOSITECHECK", WC_COMPOSITECHECK),
            (b"SEPCHARS", WC_SEPCHARS),
            (b"DISCARDNS", WC_DISCARDNS),
            (b"DEFAULTCHAR", WC_DEFAULTCHAR),
        ],
    )?;
    let (wide, trailing) = text.as_chunks::<2>();
    if !trailing.is_empty() {
        return Err(ERROR_INVALID_PARAMETER);
    }
    let wide: Vec<_> = wide.iter().copied().map(u16::from_le_bytes).collect();
    let default = match default.map(c_string).unwrap_or_default() {
        [] => None,
        [one] => Some([*one, 0]),
        [one, two] => Some([*one, *two]),
        _ => return Err(ERROR_INVALID_PARAMETER),
    };
    host::encode_code_page(page, flags, &wide, default)
}
