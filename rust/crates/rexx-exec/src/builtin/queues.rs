//! `RXQUEUE`: the current queue's name, and named queues through `.RexxQueue`
//! (`expression/BuiltinFunctions.cpp`'s `RXQUEUE`).

use rexx_core::ObjRef;

use super::datatype::{SymbolKind, classify};
use super::{Args, arg, required_string};
use crate::Interp;
use crate::error::{Failure, Raised};

/// The options `RXQUEUE` names in its 40.904.
const OPTIONS: &str = "CDEGOS";

/// `RXQUEUE(option [, name])`.
pub(crate) fn rxqueue(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let option = required_string(interp, args, 1);
    let queue_name = match arg(args, 2) {
        Some(value) => Some(interp.string_value_text(value)?),
        None => None,
    };
    let letter = option.first().map(u8::to_ascii_uppercase);
    let required = |queue_name: Option<Vec<u8>>| match queue_name {
        Some(queue_name) => {
            if matches!(classify(&queue_name), SymbolKind::Bad) {
                return Err(Failure::from(Raised::argument_not_a_symbol(
                    name,
                    2,
                    &queue_name,
                )));
            }
            Ok(queue_name)
        }
        None => Err(Raised::not_enough_arguments(name, 2).into()),
    };
    let (target, message, argument) = match letter {
        Some(b'G') => {
            if queue_name.is_some() {
                return Err(Raised::too_many_arguments(name, 1).into());
            }
            (stdque(interp)?, b"GET".as_slice(), None)
        }
        Some(b'C') => {
            let argument = match queue_name {
                Some(_) => Some(required(queue_name)?),
                None => None,
            };
            (queue_class(interp), b"CREATE".as_slice(), argument)
        }
        Some(b'S') => {
            // The queue exit's pass at the new name, before it is checked (`BUILTIN(RXQUEUE)`).
            let queue_name = match queue_name {
                Some(given) if interp.exits.is_some() => {
                    Some(interp.queue_name_exit(&given)?.unwrap_or(given))
                }
                other => other,
            };
            let argument = required(queue_name)?;
            (stdque(interp)?, b"SET".as_slice(), Some(argument))
        }
        Some(b'O') => (
            queue_class(interp),
            b"OPEN".as_slice(),
            Some(required(queue_name)?),
        ),
        Some(b'E') => (
            queue_class(interp),
            b"EXISTS".as_slice(),
            Some(required(queue_name)?),
        ),
        Some(b'D') => (
            queue_class(interp),
            b"DELETE".as_slice(),
            Some(required(queue_name)?),
        ),
        _ => return Err(Raised::argument_not_in_list(name, 1, OPTIONS, &option).into()),
    };
    let frame = interp.roots.activity_mut().push_frame();
    let argument = argument.map(|text| {
        let value = interp.text(&text);
        interp.roots.activity_mut().push_temp(value);
        value
    });
    let caller = interp.caller();
    let arguments: &[Option<ObjRef>] = match argument {
        Some(_) => &[argument],
        None => &[],
    };
    let sent = pinned!(
        interp,
        crate::pinning::PinKind::PullWrapper,
        interp.send_message(target, message, None, arguments, caller)
    );
    interp.roots.activity_mut().pop_frame(frame);
    Ok(sent?.unwrap_or_else(|| interp.text(b"")))
}

/// `.local`'s `STDQUE`.
fn stdque(interp: &mut Interp) -> Result<ObjRef, Failure> {
    Ok(interp.local_route(b"STDQUE")?.unwrap_or(ObjRef::NIL))
}

/// The `REXX` package's `RexxQueue`.
fn queue_class(interp: &mut Interp) -> ObjRef {
    interp
        .rexx_package_class(b"REXXQUEUE")
        .unwrap_or(ObjRef::NIL)
}
