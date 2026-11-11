//! The five native methods the `rxregexp` library exports, which
//! `rxregexp.cls` binds to `RegularExpression`
//! (`docs/design/extensions.md`). The
//! compiled pattern and the match mode live in the instance's own native
//! state; `!POS` is the receiver's variable in the method's scope.

use std::rc::Rc;

use interrexx_core::{Body, ExtensionFields, ExtensionState, NativeState, ObjRef};

use interrexx_api::values::Host;

use crate::error::Raised;
use crate::regexp::{self, Regex};
use crate::{Failure, Interp};

/// What an instance's pattern is.
#[derive(Clone, Debug)]
enum Compiled {
    /// None was given: `new` with no pattern.
    Absent,
    Pattern(Rc<Regex>),
    /// The last `parse` failed, after which everything matches.
    Failed,
    /// `uninit` has run (Deviation 88).
    Freed,
}

#[derive(Clone, Debug)]
struct RegExpState {
    compiled: Compiled,
    minimal: bool,
}

impl ExtensionFields for RegExpState {
    fn clone_fields(&self) -> Box<dyn ExtensionFields> {
        Box::new(self.clone())
    }
}

/// The receiver's state, or `None` where no `init` has stored one.
fn state(interp: &Interp, receiver: ObjRef) -> Option<&RegExpState> {
    match &interp.heap.get(receiver)?.body {
        Body::Instance {
            native: Some(native),
            ..
        } => match &**native {
            NativeState::Extension(state) => state.downcast_ref(),
            _ => None,
        },
        _ => None,
    }
}

/// Stores `state` as the receiver's. A receiver that cannot hold one raises
/// 93.900, as the methods used without one do.
fn store(interp: &mut Interp, receiver: ObjRef, state: RegExpState) -> Result<(), Failure> {
    let Some(object) = interp.heap.get_mut(receiver) else {
        unreachable!("a live receiver")
    };
    let Body::Instance { native, .. } = &mut object.body else {
        return Err(Raised::method_user_defined(UNUSABLE).into());
    };
    if native
        .as_deref()
        .is_some_and(|held| !matches!(held, NativeState::Extension(_)))
    {
        return Err(Raised::method_user_defined(UNUSABLE).into());
    }
    *native = Some(Box::new(NativeState::Extension(ExtensionState::new(
        Box::new(state),
    ))));
    Ok(())
}

/// Deviation 88's message for an instance no pattern was ever compiled for.
const NO_PATTERN: &str = "RegularExpression has no pattern; give one to new or parse";
/// Deviation 88's message for an instance after `uninit`.
const UNINITIALIZED: &str = "RegularExpression has been uninitialized";
/// The message for a receiver whose body has another native state.
const UNUSABLE: &str = "RegularExpression methods cannot run on this receiver";

/// Sets the receiver's `!POS` in `scope`.
fn set_position(
    interp: &mut Interp,
    receiver: ObjRef,
    scope: ObjRef,
    position: usize,
) -> Result<(), Failure> {
    let value = interp.counted(position);
    let owner = interp.pool_owner(receiver)?;
    interp.set_pool_variable(owner, scope, b"!POS", value);
    Ok(())
}

/// A string argument's bytes, 88.909 for an object with no string value, as
/// the native boundary converts one.
fn text_argument(interp: &mut Interp, value: ObjRef, position: usize) -> Result<Vec<u8>, Failure> {
    let Some(text) = interp.blamed_string_conversion(value)? else {
        return Err(Raised::native_argument_needs_a_string_value(position).into());
    };
    Ok(interp.to_text(text).into_owned())
}

/// 88.922 where more than `arity` arguments were passed, as the native
/// boundary checks a method's signature.
fn check_arity(args: &[Option<ObjRef>], arity: usize) -> Result<(), Failure> {
    if args.len() > arity {
        return Err(Raised::too_many_external_arguments(arity).into());
    }
    Ok(())
}

/// The arguments a method takes, read from the left as the native boundary
/// reads them: a required one that is missing is 88.901 and one with no
/// string value 88.909, and only then are too many arguments 88.922, so an
/// argument beyond the last the method takes is not read. `required` leading
/// arguments must be present.
fn read_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    arity: usize,
    required: usize,
) -> Result<Vec<Option<Vec<u8>>>, Failure> {
    let mut texts = Vec::with_capacity(arity);
    for at in 0..arity {
        match args.get(at).copied().flatten() {
            Some(value) => texts.push(Some(text_argument(interp, value, at + 1)?)),
            None if at < required => return Err(Raised::missing_native_argument(at + 1).into()),
            None => texts.push(None),
        }
    }
    check_arity(args, arity)?;
    Ok(texts)
}

/// A match type: `Some(minimal)` for `MINIMAL` or `MAXIMAL`, `Some(None)`
/// for `CURRENT` where `current` allows it, `None` for anything else.
fn match_type(text: &[u8], current: bool) -> Option<Option<bool>> {
    let upper = text.to_ascii_uppercase();
    match &upper[..] {
        b"MINIMAL" => Some(Some(true)),
        b"MAXIMAL" => Some(Some(false)),
        b"CURRENT" if current => Some(None),
        _ => None,
    }
}

/// Error 93 with no subcode: a match type that is not one of the names.
fn incorrect_call() -> Failure {
    Raised::syntax(93, 0, Vec::new()).into()
}

/// `RegExp_Init([pattern [, matchtype]])`: compiles the pattern, if any, and
/// sets the mode. A compile error raises 38, ahead of a bad match type's 93.
pub(crate) fn init(
    interp: &mut Interp,
    receiver: ObjRef,
    _scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let texts = read_arguments(interp, args, 2, 0)?;
    let mut minimal = Some(false);
    if let Some(text) = &texts[1] {
        minimal = match_type(text, false).map(|kind| kind.unwrap_or(false));
    }
    let mut compiled = Compiled::Absent;
    if let Some(pattern) = &texts[0] {
        match regexp::compile(pattern) {
            Ok((regex, _)) => compiled = Compiled::Pattern(Rc::new(regex)),
            Err(_) => return Err(Raised::syntax(38, 0, Vec::new()).into()),
        }
    }
    let Some(minimal) = minimal else {
        return Err(incorrect_call());
    };
    store(interp, receiver, RegExpState { compiled, minimal })?;
    // The library keeps its automaton behind `CSELF`, which a class binding
    // these methods itself can read: a `.Pointer` until `uninit` drops it.
    let cself = Host::new_pointer(interp, std::ptr::null_mut());
    Host::set_object_variable(interp, b"CSELF", Some(cself));
    Ok(None)
}

/// `RegExp_Uninit()`: frees the pattern. Once freed, a second `uninit`, as
/// the collection runs after an explicit one, does nothing.
pub(crate) fn uninit(
    interp: &mut Interp,
    receiver: ObjRef,
    _scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    check_arity(args, 0)?;
    Host::drop_object_variable(interp, b"CSELF");
    if let Some(held) = state(interp, receiver) {
        let minimal = held.minimal;
        store(
            interp,
            receiver,
            RegExpState {
                compiled: Compiled::Freed,
                minimal,
            },
        )?;
    }
    Ok(None)
}

/// `RegExp_Parse(pattern [, matchtype])`: 0 or a compile error code, with
/// `!POS` the bytes consumed or the error's position. A failed parse leaves
/// an instance that matches everything.
pub(crate) fn parse(
    interp: &mut Interp,
    receiver: ObjRef,
    scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let texts = read_arguments(interp, args, 2, 1)?;
    let held = state(interp, receiver).cloned();
    if matches!(
        held.as_ref().map(|held| &held.compiled),
        Some(Compiled::Freed)
    ) {
        return Err(Raised::method_user_defined(UNINITIALIZED).into());
    }
    let mut minimal = held.as_ref().is_some_and(|held| held.minimal);
    if let Some(text) = &texts[1] {
        match match_type(text, true) {
            Some(Some(kind)) => minimal = kind,
            Some(None) => {}
            None => return Err(incorrect_call()),
        }
    }
    let Some(pattern) = &texts[0] else {
        unreachable!("the pattern is required")
    };
    let (compiled, code, position) = match regexp::compile(pattern) {
        Ok((regex, consumed)) => (Compiled::Pattern(Rc::new(regex)), 0, consumed),
        Err(error) => (Compiled::Failed, usize::from(error.code), error.position),
    };
    store(interp, receiver, RegExpState { compiled, minimal })?;
    set_position(interp, receiver, scope, position)?;
    Ok(Some(interp.counted(code)))
}

/// The receiver's pattern for `match` and `pos`, 93.900 where there is none.
fn pattern_of(interp: &Interp, receiver: ObjRef) -> Result<(Option<Rc<Regex>>, bool), Failure> {
    let Some(held) = state(interp, receiver) else {
        return Err(Raised::method_user_defined(NO_PATTERN).into());
    };
    match &held.compiled {
        Compiled::Pattern(regex) => Ok((Some(Rc::clone(regex)), held.minimal)),
        Compiled::Failed => Ok((None, held.minimal)),
        Compiled::Absent => Err(Raised::method_user_defined(NO_PATTERN).into()),
        Compiled::Freed => Err(Raised::method_user_defined(UNINITIALIZED).into()),
    }
}

/// The subject argument of `match` and `pos`, 88.901 where it is omitted.
fn subject(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<Vec<u8>, Failure> {
    let mut texts = read_arguments(interp, args, 1, 1)?;
    Ok(texts.swap_remove(0).expect("the subject is required"))
}

/// `RegExp_Match(string)`: 1 or 0, with `!POS` set.
pub(crate) fn match_string(
    interp: &mut Interp,
    receiver: ObjRef,
    scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let text = subject(interp, args)?;
    let (regex, minimal) = pattern_of(interp, receiver)?;
    let (matched, position) = match regex {
        None => (true, 0),
        Some(regex) if minimal => regex.match_minimal(&text),
        Some(regex) => regex.match_maximal(&text),
    };
    set_position(interp, receiver, scope, position)?;
    Ok(Some(interp.counted(usize::from(matched))))
}

/// `RegExp_Pos(haystack)`: the 1-based start of the first match or 0, with
/// `!POS` set.
pub(crate) fn pos(
    interp: &mut Interp,
    receiver: ObjRef,
    scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let text = subject(interp, args)?;
    let (regex, minimal) = pattern_of(interp, receiver)?;
    let (start, position) = match regex {
        None => (1, 0),
        Some(regex) => regex.pos(&text, minimal),
    };
    set_position(interp, receiver, scope, position)?;
    Ok(Some(interp.counted(start)))
}
