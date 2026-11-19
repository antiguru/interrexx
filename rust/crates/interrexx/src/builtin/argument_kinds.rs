//! What each builtin reads its arguments as, for the error an argument raises
//! when its `STRING` method answers an object with no string value
//! (`r[conv.string.builtin-argument]`, `r[conv.string.argument-kind]`).

use std::cell::Cell;

use interrexx_core::ObjRef;

use super::{datetime, queues, state, stream};
use crate::Interp;
use crate::error::{Failure, Notation, Raised};

/// One argument of a builtin call being converted: the row's name, its
/// 1-based position, and the refusal the call has deferred so far.
#[derive(Copy, Clone)]
pub(crate) struct BuiltinArgument<'a> {
    pub(crate) name: &'static [u8],
    pub(crate) position: usize,
    pub(crate) deferred: &'a Deferred,
}

/// The text an argument holds in place of a string value it does not have,
/// when its refusal is deferred: no check of an argument that is not read as a
/// string admits it, so the builtin's body fails it where that check falls.
const UNCHECKED: &[u8] = b"\x01\x01";

/// The refusal of the first argument of a builtin call whose `STRING` answer
/// has no string value and that follows an argument with a check of its own.
#[derive(Default)]
pub(crate) struct Deferred(Cell<Option<Box<Raised>>>);

impl Deferred {
    /// The call's answer once its body ran with [`UNCHECKED`] in place of the
    /// deferred arguments. An error the body raised is the call's, with
    /// [`UNCHECKED`] quoted as the string of `.nil`; an answer means that no
    /// check failed the text, which only an argument read as a string allows,
    /// and the deferred refusal is the call's error then.
    pub(crate) fn settle(
        &self,
        interp: &mut Interp,
        answer: Result<ObjRef, Failure>,
    ) -> Result<ObjRef, Failure> {
        let Some(refusal) = self.0.take() else {
            return answer;
        };
        match answer {
            Ok(_) => Err(Failure::Raised(refusal)),
            Err(Failure::Raised(mut raised)) => {
                let nil = interp.to_text(ObjRef::NIL).into_owned();
                for item in &mut raised.additional {
                    if item.as_slice() == UNCHECKED {
                        item.clone_from(&nil);
                    }
                }
                replace_pending_additional(interp, &nil);
                Err(Failure::Raised(raised))
            }
            Err(other) => Err(other),
        }
    }
}

/// Puts `nil` in place of [`UNCHECKED`] in the object a raise made by Rexx code
/// gave as its `ADDITIONAL`, which the condition object carries instead of the
/// error's substitutions: the body of `STREAM` sends `COMMAND`, whose
/// `RAISE SYNTAX 93.914 ARRAY (...)` quotes the command it was given.
fn replace_pending_additional(interp: &mut Interp, nil: &[u8]) {
    let Some(object) = interp.activity.pending_additional else {
        return;
    };
    let Some(slots) = interp.array_slots_of(object) else {
        if interp.to_text(object).as_ref() == UNCHECKED {
            interp.activity.pending_additional = Some(interp.text(nil));
        }
        return;
    };
    let unchecked = |interp: &mut Interp, slot: Option<ObjRef>| {
        slot.is_some_and(|item| interp.to_text(item).as_ref() == UNCHECKED)
    };
    if !slots.iter().any(|&slot| unchecked(interp, slot)) {
        return;
    }
    let frame = interp.roots.activity_mut().push_frame();
    let mut replaced = Vec::with_capacity(slots.len());
    for slot in slots {
        let item = if unchecked(interp, slot) {
            let text = interp.text(nil);
            interp.roots.activity_mut().push_temp(text);
            Some(text)
        } else {
            slot
        };
        replaced.push(item);
    }
    let array = interp.alloc_charged(
        interrexx_core::BehaviourId::ARRAY,
        interrexx_core::Body::array(replaced),
    );
    interp.roots.activity_mut().pop_frame(frame);
    interp.activity.pending_additional = Some(array);
}

/// What a builtin argument whose `STRING` answer has no string value reads as.
/// Where every argument before it is read as a string, names a stream or is the
/// string the builtin works on, none of which has a check of its own, its
/// refusal is raised at once. Otherwise it reads as [`UNCHECKED`], so that a
/// call with another fault as well raises the error ooRexx raises for it, a
/// convenience, since which of several faults is raised is unspecified
/// (`r[conv.string.argument-kind]`, `r[conv.error.precedence]`). An argument
/// with no kind keeps `SYNTAX 98.973`. `readable` is the text the refusal
/// quotes.
pub(crate) fn refuse(
    interp: &mut Interp,
    argument: BuiltinArgument<'_>,
    readable: &[u8],
) -> Result<ObjRef, Failure> {
    let BuiltinArgument { name, position, .. } = argument;
    let Some(kind) = kind_of(name, position) else {
        return Err(Raised::nostring_syntax(readable).into());
    };
    let refusal = kind_refusal(name, position, kind, readable);
    if !follows_a_check(name, position) {
        return Err(refusal.into());
    }
    let first = argument.deferred.0.take();
    argument
        .deferred
        .0
        .set(Some(first.unwrap_or_else(|| Box::new(refusal))));
    let text = interp.text(UNCHECKED);
    interp.roots.activity_mut().push_temp(text);
    Ok(text)
}

/// What a builtin reads one of its arguments as.
#[derive(Copy, Clone)]
enum Kind {
    /// A string, numbered among the arguments of the method the builtin
    /// stands for: 88.909 (`r[conv.string.builtin-argument]`).
    String(usize),
    /// An option read as a string, numbered as [`Kind::String`] is: 88.909.
    /// Its letter has a check of its own.
    StringOption(usize),
    /// A whole number: 40.12.
    Whole,
    /// A pad or another single character: 40.23.
    Character,
    /// An option, one of the letters or words given: 40.904.
    Option(&'static str),
    /// The number a numeric builtin works on: 40.11.
    Target,
    /// The number `D2C` converts: 93.929.
    D2c,
    /// The number `D2X` converts: 93.928.
    D2x,
    /// A hexadecimal or a binary string: 93.933 or 93.934.
    Digits(Notation),
    /// The first argument of `XRANGE`, a class name or a single character:
    /// 40.28.
    ClassOrCharacter,
    /// The setting of `TRACE`: 24.1.
    Trace,
    /// The name of a stream: 93.938, numbered 1.
    StreamName,
    /// An argument after one with a check of its own, so that its refusal is
    /// always deferred and its error is the one the builtin's body raises for
    /// [`UNCHECKED`].
    Checked,
}

/// Each builtin's arguments that have a kind, as `(position, kind)`. A position
/// not listed is the string the builtin works on, or an argument that keeps
/// `SYNTAX 98.973` (`r[conv.string.deviation.169]`). `MAX`, `MIN` and `XRANGE`
/// take any number of arguments and are answered by [`kind_of`] itself.
static KINDS: &[(&[u8], &[(usize, Kind)])] = &[
    (b"ABBREV", &[(2, Kind::String(1)), (3, Kind::Whole)]),
    (b"ABS", &[(1, Kind::Target)]),
    (b"ARG", &[(1, Kind::Whole), (2, Kind::Checked)]),
    (b"B2X", &[(1, Kind::Digits(Notation::Binary))]),
    (b"BITAND", &[(2, Kind::String(1)), (3, Kind::Character)]),
    (b"BITOR", &[(2, Kind::String(1)), (3, Kind::Character)]),
    (b"BITXOR", &[(2, Kind::String(1)), (3, Kind::Character)]),
    (b"C2D", &[(2, Kind::Whole)]),
    (b"CENTER", &[(2, Kind::Whole), (3, Kind::Checked)]),
    (b"CENTRE", &[(2, Kind::Whole), (3, Kind::Checked)]),
    (
        b"CHANGESTR",
        &[(1, Kind::String(1)), (3, Kind::String(2)), (4, Kind::Whole)],
    ),
    (
        b"CHARIN",
        &[(1, Kind::StreamName), (2, Kind::Whole), (3, Kind::Checked)],
    ),
    (
        b"CHAROUT",
        &[
            (1, Kind::StreamName),
            (2, Kind::String(1)),
            (3, Kind::Whole),
        ],
    ),
    (b"CHARS", &[(1, Kind::StreamName)]),
    (b"COMPARE", &[(2, Kind::String(1)), (3, Kind::Character)]),
    (b"CONDITION", &[(1, Kind::Option(state::CONDITION_OPTIONS))]),
    (b"COPIES", &[(2, Kind::Whole)]),
    (b"COUNTSTR", &[(1, Kind::String(1))]),
    (b"D2C", &[(1, Kind::D2c), (2, Kind::Checked)]),
    (b"D2X", &[(1, Kind::D2x), (2, Kind::Checked)]),
    (b"DATATYPE", &[(2, Kind::StringOption(1))]),
    (
        b"DATE",
        &[
            (1, Kind::Option(datetime::DATE_OPTIONS1)),
            (2, Kind::Checked),
            (3, Kind::Checked),
            (4, Kind::Checked),
            (5, Kind::Checked),
        ],
    ),
    (b"DELSTR", &[(2, Kind::Whole), (3, Kind::Checked)]),
    (b"DELWORD", &[(2, Kind::Whole), (3, Kind::Checked)]),
    (b"ERRORTEXT", &[(1, Kind::Whole)]),
    (
        b"FORMAT",
        &[
            (1, Kind::Target),
            (2, Kind::Checked),
            (3, Kind::Checked),
            (4, Kind::Checked),
            (5, Kind::Checked),
        ],
    ),
    (b"GC", &[(1, Kind::Option(state::GC_OPTIONS))]),
    (
        b"INSERT",
        &[
            (1, Kind::String(1)),
            (3, Kind::Whole),
            (4, Kind::Checked),
            (5, Kind::Checked),
        ],
    ),
    (
        b"LASTPOS",
        &[(1, Kind::String(1)), (3, Kind::Whole), (4, Kind::Checked)],
    ),
    (b"LEFT", &[(2, Kind::Whole), (3, Kind::Checked)]),
    (
        b"LINEIN",
        &[(1, Kind::StreamName), (2, Kind::Whole), (3, Kind::Checked)],
    ),
    (
        b"LINEOUT",
        &[
            (1, Kind::StreamName),
            (2, Kind::String(1)),
            (3, Kind::Whole),
        ],
    ),
    (
        b"LINES",
        &[
            (1, Kind::StreamName),
            (2, Kind::Option(stream::LINES_OPTIONS)),
        ],
    ),
    (b"LOWER", &[(2, Kind::Whole), (3, Kind::Checked)]),
    (
        b"OVERLAY",
        &[
            (1, Kind::String(1)),
            (3, Kind::Whole),
            (4, Kind::Checked),
            (5, Kind::Checked),
        ],
    ),
    (
        b"POS",
        &[(1, Kind::String(1)), (3, Kind::Whole), (4, Kind::Checked)],
    ),
    (
        b"RANDOM",
        &[(1, Kind::Whole), (2, Kind::Checked), (3, Kind::Checked)],
    ),
    (b"RIGHT", &[(2, Kind::Whole), (3, Kind::Checked)]),
    (
        b"RXQUEUE",
        &[(1, Kind::Option(queues::OPTIONS)), (2, Kind::Checked)],
    ),
    (b"SIGN", &[(1, Kind::Target)]),
    (b"SOURCELINE", &[(1, Kind::Whole)]),
    (b"SPACE", &[(2, Kind::Whole), (3, Kind::Checked)]),
    (
        b"STREAM",
        &[
            (1, Kind::StreamName),
            (2, Kind::Option(stream::STREAM_OPTIONS)),
            (3, Kind::Checked),
        ],
    ),
    // The character is read as a string, so it holds `UNCHECKED` through the
    // body, and its refusal is raised once the option has passed its check.
    (
        b"STRIP",
        &[(2, Kind::StringOption(1)), (3, Kind::String(2))],
    ),
    (
        b"SUBSTR",
        &[(2, Kind::Whole), (3, Kind::Checked), (4, Kind::Checked)],
    ),
    (b"SUBWORD", &[(2, Kind::Whole), (3, Kind::Checked)]),
    (
        b"TIME",
        &[
            (1, Kind::Option(datetime::TIME_OPTIONS1)),
            (2, Kind::Checked),
            (3, Kind::Checked),
        ],
    ),
    (b"TRACE", &[(1, Kind::Trace)]),
    (
        b"TRANSLATE",
        &[
            (2, Kind::String(1)),
            (3, Kind::String(2)),
            (4, Kind::Character),
            (5, Kind::Checked),
            (6, Kind::Checked),
        ],
    ),
    (b"TRUNC", &[(1, Kind::Target), (2, Kind::Checked)]),
    (b"UPPER", &[(2, Kind::Whole), (3, Kind::Checked)]),
    (
        b"VERIFY",
        &[
            (2, Kind::String(1)),
            (3, Kind::StringOption(2)),
            (4, Kind::Checked),
            (5, Kind::Checked),
        ],
    ),
    (b"WORD", &[(2, Kind::Whole)]),
    (b"WORDINDEX", &[(2, Kind::Whole)]),
    (b"WORDLENGTH", &[(2, Kind::Whole)]),
    (b"WORDPOS", &[(1, Kind::String(1)), (3, Kind::Whole)]),
    (b"X2B", &[(1, Kind::Digits(Notation::Hex))]),
    (b"X2C", &[(1, Kind::Digits(Notation::Hex))]),
    (
        b"X2D",
        &[(1, Kind::Digits(Notation::Hex)), (2, Kind::Checked)],
    ),
];

/// Whether an argument before `position` of `name` has a check of its own:
/// one that is not read as a string, does not name a stream and is not the
/// string the builtin works on.
fn follows_a_check(name: &[u8], position: usize) -> bool {
    (1..position).any(|before| {
        !matches!(
            kind_of(name, before),
            None | Some(Kind::String(_) | Kind::StreamName)
        )
    })
}

/// The kind of the argument at `position` of the builtin `name`, from
/// [`KINDS`], or for `MAX`, `MIN` and `XRANGE` from its position alone.
fn kind_of(name: &[u8], position: usize) -> Option<Kind> {
    match name {
        b"MAX" | b"MIN" if position == 1 => Some(Kind::Target),
        b"XRANGE" if position == 1 => Some(Kind::ClassOrCharacter),
        b"MAX" | b"MIN" | b"XRANGE" => Some(Kind::Checked),
        _ => KINDS
            .iter()
            .find(|(row, _)| *row == name)
            .and_then(|(_, kinds)| kinds.iter().find(|(at, _)| *at == position))
            .map(|&(_, kind)| kind),
    }
}

/// The error the argument at `position` of `name`, of `kind`, raises when its
/// `STRING` answer has no string value. `found` is the text the error quotes,
/// which is not specified.
fn kind_refusal(name: &'static [u8], position: usize, kind: Kind, found: &[u8]) -> Raised {
    let first = found.first().copied().unwrap_or(b' ');
    match kind {
        Kind::String(number) | Kind::StringOption(number) => {
            Raised::argument_needs_a_string_value(number)
        }
        Kind::Whole => Raised::argument_not_whole(name, position, found),
        Kind::Character => Raised::argument_not_a_pad(name, position, found),
        Kind::Option(valid) => Raised::argument_not_in_list(name, position, valid, found),
        Kind::Target => Raised::argument_not_a_number(name, position, found),
        Kind::D2c => Raised::d2c_value_not_whole(found),
        Kind::D2x => Raised::d2x_value_not_whole(found),
        Kind::Digits(notation) => Raised::invalid_digit(notation, first),
        Kind::ClassOrCharacter => Raised::argument_not_a_pad_or_class_name(name, position, found),
        Kind::Trace => crate::trace::raised_invalid_trace_letter(first),
        Kind::StreamName => Raised::method_argument_needs_a_string_value(1),
        // Never raised: `refuse` defers this kind, and the body's check fails
        // `UNCHECKED` first.
        Kind::Checked => Raised::nostring_syntax(found),
    }
}

#[cfg(test)]
mod tests {
    use super::{KINDS, Kind, follows_a_check};

    /// A [`Kind::Checked`] argument is never refused at once, which is what
    /// keeps its placeholder refusal from being raised.
    #[test]
    fn a_checked_argument_follows_an_argument_with_a_check() {
        for (name, kinds) in KINDS {
            for &(position, kind) in *kinds {
                if !matches!(kind, Kind::Checked) {
                    continue;
                }
                assert!(
                    follows_a_check(name, position),
                    "{} argument {position}",
                    String::from_utf8_lossy(name)
                );
            }
        }
    }
}
