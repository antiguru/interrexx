//! The `LIBRARY REXX` entry points behind `.CommandHandler`,
//! `.CommandContext`, `.CommandInput`, `.CommandOutput` and
//! `.NativeCommandHandler`. The table they read and write is
//! [`crate::command::Environments`].

use interrexx_api::callbacks::Surface;
use interrexx_core::ObjRef;

use super::string::subclass_text;
use super::{Cleared, class_protocol::new_instance, hash, required_string_argument};
use crate::activation::ActivationId;
use crate::builtin::datatype::{SymbolKind, classify};
use crate::command::NATIVE_AT;
use crate::error::Raised;
use crate::{Failure, Interp, Loud};

/// Answer type of every entry point here.
type Answer = Result<Option<ObjRef>, Failure>;

/// 88.901 for argument `at` (zero-based) left out, as a native signature check
/// raises it for these entry points: numbered, and reported at the sending
/// clause's line (`.CommandHandler~handler`).
fn missing(at: usize) -> Failure {
    Raised::missing_named_argument(&(at + 1).to_string()).into()
}

/// Argument `at` (zero-based), which the native entry declares as a required
/// string (`CSTRING` or `RexxStringObject`).
fn string_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    at: usize,
) -> Result<Vec<u8>, Failure> {
    match args.get(at).copied().flatten() {
        Some(value) => interp.string_value_text(value),
        None => Err(missing(at)),
    }
}

/// Argument `at` (zero-based), a required object (`RexxObjectPtr`).
fn object_argument(args: &[Option<ObjRef>], at: usize) -> Result<ObjRef, Failure> {
    args.get(at).copied().flatten().ok_or_else(|| missing(at))
}

/// `1` or `0`, as a `logical_t` answers.
fn flag(interp: &mut Interp, yes: bool) -> Answer {
    Ok(Some(interp.text(if yes { b"1" } else { b"0" })))
}

/// The object or `.nil`.
fn or_nil(found: Option<ObjRef>) -> Answer {
    Ok(Some(found.unwrap_or(ObjRef::NIL)))
}

/// What the receiver's `CSELF` holds: the identity of the command its context
/// belongs to.
fn cself_of(interp: &mut Interp, receiver: ObjRef) -> Option<ObjRef> {
    let owner = interp.pool_owner(receiver).ok()?;
    interp.pools_of(owner)?.find(b"CSELF")
}

/// The live-state check on the receiver's `CSELF`: the running command's
/// index.
fn live(interp: &mut Interp, receiver: ObjRef) -> Result<usize, Failure> {
    let cself = cself_of(interp, receiver);
    interp.live_command(cself)
}

/// An object of its own identity, nothing else: the `CSELF` of one
/// `.CommandContext`.
pub(crate) fn new_identity(interp: &mut Interp) -> ObjRef {
    let class = interp.object_model().object;
    match new_instance(interp, class) {
        Ok(object) => object,
        Err(_) => unreachable!("Object is not abstract"),
    }
}

impl Interp {
    /// A `.NativeCommandHandler` with `NAME` set to `name`, made as
    /// `CommandHandler::getRexxObject` makes one: its private `NEW` run
    /// directly, so no `INIT`.
    pub(crate) fn native_handler_object(
        &mut self,
        name: &[u8],
        at: usize,
    ) -> Result<ObjRef, Failure> {
        let class = self
            .rexx_package_class(b"NATIVECOMMANDHANDLER")
            .ok_or_else(|| Loud::missing_rexx_class("NativeCommandHandler"))?;
        let object = new_instance(self, class)?;
        let name = self.text(name);
        self.roots.activity_mut().push_temp(name);
        self.set_pool_variable(object, class, b"NAME", name);
        // Which native handler it stands for, in a variable of its own so
        // that a copy, which copies the variables, stands for the same one.
        let index = self.text(at.to_string().as_bytes());
        self.roots.activity_mut().push_temp(index);
        self.set_pool_variable(object, class, NATIVE_AT, index);
        Ok(object)
    }

    /// The variable retriever for `name` run against `issuer`: `serve` with
    /// that activation swapped in as the running one, or 93.900 for a name
    /// that is not a variable's.
    fn in_issuer<R>(
        &mut self,
        issuer: ActivationId,
        name: &[u8],
        serve: impl FnOnce(&mut Interp, &[u8]) -> R,
    ) -> Result<R, Failure> {
        let upper = name.to_ascii_uppercase();
        if !matches!(
            classify(&upper),
            SymbolKind::Name | SymbolKind::Stem | SymbolKind::CompoundName
        ) {
            let text = format!(
                "not a Rexx variable name: {}",
                String::from_utf8_lossy(name)
            );
            return Err(Raised::method_user_defined(&text).into());
        }
        self.in_activation(issuer, |interp| serve(interp, &upper))
            .ok_or_else(|| {
                Raised::execution_user_defined(
                    "this CommandContext belongs to a command that has ended",
                )
                .into()
            })
    }
}

/// `cmdhandler_environments`: a `StringTable` of every environment name
/// with the object of its handler in use.
pub(super) fn environments(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Answer {
    let listed = interp.environment_list()?;
    let table = hash::new_string_table(interp)?;
    interp.roots.activity_mut().push_temp(table);
    for (name, object) in listed {
        hash::native_put(interp, table, &name, object)?;
    }
    Ok(Some(table))
}

/// `cmdhandler_handler(name)`: the object of the handler in use, or `.nil`.
pub(super) fn handler(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Answer {
    let name = string_argument(interp, args, 0)?;
    or_nil(interp.environment_handler(&name)?)
}

/// `cmdhandler_handlers(name)`: the handlers' objects, the one in use first.
pub(super) fn handlers(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Answer {
    let name = string_argument(interp, args, 0)?;
    let objects: Vec<Option<ObjRef>> = interp
        .environment_handlers(&name)?
        .into_iter()
        .map(Some)
        .collect();
    Ok(Some(Surface::new_array(interp, &objects)))
}

/// `cmdhandler_register(name, handler)`: the object of the handler covered,
/// or `.nil`.
pub(super) fn register(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Answer {
    let name = string_argument(interp, args, 0)?;
    let handler = object_argument(args, 1)?;
    or_nil(interp.register_environment(&name, handler)?)
}

/// `cmdhandler_deregister(name)`: the object of the handler removed, or
/// `.nil` where there was none.
pub(super) fn deregister(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Answer {
    let name = string_argument(interp, args, 0)?;
    or_nil(interp.deregister_environment(&name)?)
}

/// `cmdcontext_isRedirected(which)`: by its first letter, `Input`, `Output`,
/// `Error` or `Same`. The checks run in the order of
/// `r[class.commandcontext.isredirected]`: `which` present (88.901), a string
/// (88.909), the argument count (88.922), then the letter (93.900).
pub(super) fn is_redirected(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Answer {
    let which = object_argument(args, 0)?;
    // An instance of a subclass of `String` is read by its own characters
    // (spec: class.commandcontext.deviation.109); any other object is asked
    // for the string `request('STRING')` answers.
    let which = match subclass_text(interp, which) {
        Some(text) => text.to_vec(),
        None => {
            let string = required_string_argument(interp, which, 1)?;
            interp.string_value_text(string)?
        }
    };
    if args.len() > 1 {
        return Err(Raised::too_many_external_arguments(1).into());
    }
    let at = live(interp, receiver)?;
    let [input, output, error, same] = interp.command_redirects(at);
    let answer = match which.first().map(u8::to_ascii_uppercase) {
        Some(b'I') => input,
        Some(b'O') => output,
        Some(b'E') => error,
        Some(b'S') => same,
        _ => {
            return Err(Raised::method_user_defined(
                "isRedirected: the argument must be Input, Output, Error or Same",
            )
            .into());
        }
    };
    flag(interp, answer)
}

/// `cmdcontext_isValid`: whether the context's command is still running.
pub(super) fn is_valid(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Answer {
    let cself = cself_of(interp, receiver);
    let running = interp.command_running(cself);
    flag(interp, running)
}

/// `cmdcontext_check`: nothing, or the live-state check's 98.900.
pub(super) fn check(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Answer {
    live(interp, receiver)?;
    Ok(Some(interp.text(b"0")))
}

/// `cmdcontext_getVar(name)`: the caller's variable, or `.nil` where it has
/// no value.
pub(super) fn get_var(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Answer {
    let name = string_argument(interp, args, 0)?;
    let at = live(interp, receiver)?;
    let issuer = interp.command_issuer(at);
    let value = interp.in_issuer(issuer, &name, |interp, upper| {
        Surface::context_variable(interp, upper)
    })?;
    or_nil(value)
}

/// `cmdcontext_hasVar(name)`: whether the caller's variable has a value.
pub(super) fn has_var(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Answer {
    let name = string_argument(interp, args, 0)?;
    let at = live(interp, receiver)?;
    let issuer = interp.command_issuer(at);
    let value = interp.in_issuer(issuer, &name, |interp, upper| {
        Surface::context_variable(interp, upper)
    })?;
    flag(interp, value.is_some())
}

/// `cmdcontext_setVar(name, value)`: assigns the caller's variable, a
/// compound's tail resolved in the caller.
pub(super) fn set_var(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Answer {
    let name = string_argument(interp, args, 0)?;
    let value = object_argument(args, 1)?;
    let at = live(interp, receiver)?;
    let issuer = interp.command_issuer(at);
    interp.in_issuer(issuer, &name, |interp, upper| {
        Surface::set_context_variable(interp, upper, value);
    })?;
    Ok(None)
}

/// `cmdcontext_dropVar(name)`: drops the caller's variable.
pub(super) fn drop_var(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Answer {
    let name = string_argument(interp, args, 0)?;
    let at = live(interp, receiver)?;
    let issuer = interp.command_issuer(at);
    interp.in_issuer(issuer, &name, |interp, upper| {
        Surface::drop_context_variable(interp, upper);
    })?;
    Ok(None)
}

/// `cmdcontext_variables`: a `Directory` of the caller's variables.
pub(super) fn variables(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Answer {
    let at = live(interp, receiver)?;
    let issuer = interp.command_issuer(at);
    interp
        .in_activation(issuer, |interp| super::context::local_variables(interp, 0))
        .unwrap_or_else(|| {
            Err(Raised::execution_user_defined(
                "this CommandContext belongs to a command that has ended",
            )
            .into())
        })
        .map(Some)
}

/// `cmdcontext_forward(target, command)`: the command sent to the native
/// handler `target` stands for, as the code that issued it; its condition
/// becomes the context's `PENDING`. Answers `RC`.
pub(super) fn forward(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Answer {
    let target = object_argument(args, 0)?;
    let command = string_argument(interp, args, 1)?;
    let at = live(interp, receiver)?;
    let (kind, name) = interp.native_handler_of(target)?;
    let (rc, pending) = interp.forward_command(at, kind, &name, &command)?;
    if let Some(pending) = pending {
        let class = interp
            .rexx_package_class(b"COMMANDCONTEXT")
            .ok_or_else(|| Loud::missing_rexx_class("CommandContext"))?;
        let items: Vec<Option<ObjRef>> = pending.into_iter().map(Some).collect();
        let array = Surface::new_array(interp, &items);
        interp.roots.activity_mut().push_temp(array);
        let owner = interp.pool_owner(receiver)?;
        interp.set_pool_variable(owner, class, b"PENDING", array);
    }
    Ok(Some(rc))
}

/// `cmdcontext_lineIn`: the next line of the redirected input, `.nil` at its
/// end.
pub(super) fn line_in(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Answer {
    let at = live(interp, receiver)?;
    match interp.command_read_line(at) {
        Some(line) => Ok(Some(interp.text(&line))),
        None => Ok(Some(ObjRef::NIL)),
    }
}

/// `cmdcontext_write(stream, line)`: one line to the redirected output
/// (`O`) or error (`E`).
pub(super) fn write(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Answer {
    let stream = string_argument(interp, args, 0)?;
    let line = string_argument(interp, args, 1)?;
    let at = live(interp, receiver)?;
    let error = matches!(stream.first(), Some(b'E' | b'e'));
    interp.command_write_line(at, error, line);
    Ok(None)
}

/// `cmdnative_kind`: `SUBCOM`, `DIRECT` or `REDIRECTING`.
pub(super) fn kind(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Answer {
    let (kind, _) = interp.native_handler_of(receiver)?;
    Ok(Some(interp.text(kind.kind_name().as_bytes())))
}
