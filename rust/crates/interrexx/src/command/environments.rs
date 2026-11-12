//! The command environments of this interpreter as the `.CommandHandler`
//! classes see them: handlers written in Rexx (`.CommandHandler~register`),
//! stacked over the native ones they cover, every environment as an object,
//! and the `.CommandContext` a handler written in Rexx is handed.

use std::rc::Rc;

use interrexx_api::load::{ClassicEntry, CommandHandler};
use interrexx_api::redirect::Redirector;
use interrexx_core::ObjRef;
use rustc_hash::FxHashMap;

use super::{
    Child, CommandOutcome, Handler, HandlerCondition, NOT_REGISTERED, ReturnStatus, Supplied,
    handler_for, run_internally, start, whole_value,
};
use crate::activation::ActivationId;
use crate::error::{Failure, Raised};
use crate::redirect::{IoContext, split_lines};
use crate::scheduler::ActivityId;
use crate::{Interp, Loud};

/// The names the oracle gives the built-in handler, each an environment of
/// its own.
const SYSTEM_NAMES: &[&[u8]] = &[
    b"SH", b"", b"COMMAND", b"SYSTEM", b"KSH", b"CSH", b"BSH", b"BASH", b"TCSH", b"ZSH", b"PATH",
];

/// A native command handler: what the oracle's `CommandHandler` holds for
/// every `HandlerType` but `REXX_OBJECT`.
#[derive(Clone)]
pub(crate) enum NativeKind {
    /// The built-in handler under one of [`SYSTEM_NAMES`]: `ioCommandHandler`,
    /// a `REDIRECTING` one.
    System(Handler),
    /// One an extension registered (`AddCommandEnvironment`), `DIRECT` or
    /// `REDIRECTING`.
    Registered(Rc<CommandHandler>),
    /// A classic subcommand handler (`REGISTERED_NAME`).
    Classic(ClassicEntry),
    /// A [`NativeKind::Registered`] whose library has closed, which no
    /// command reaches any more.
    Closed,
}

impl NativeKind {
    pub(crate) fn kind_name(&self) -> &'static str {
        match self {
            NativeKind::System(_) => "REDIRECTING",
            NativeKind::Registered(handler) if handler.redirects() => "REDIRECTING",
            NativeKind::Registered(_) => "DIRECT",
            NativeKind::Classic(_) => "SUBCOM",
            NativeKind::Closed => "UNRESOLVED",
        }
    }
}

/// The `.NativeCommandHandler` variable holding the index of the native
/// handler an object stands for.
pub(crate) const NATIVE_AT: &[u8] = b"NATIVE_AT";

/// One native handler, with the `.NativeCommandHandler` that stands for it,
/// made once and kept, so one native handler is always one object.
struct NativeEnvironment {
    kind: NativeKind,
    /// The upper-cased name it was registered under, which a forward hands it
    /// as its address.
    name: Vec<u8>,
    object: Option<ObjRef>,
}

/// One handler of an environment's stack.
#[derive(Copy, Clone)]
enum Entry {
    /// A handler object written in Rexx.
    Rexx(ObjRef),
    /// An index into [`Environments::natives`].
    Native(usize),
    /// The marker a classic subcommand handler's `deregister` leaves alone on
    /// its stack, which keeps the API service from bringing it back in this
    /// instance.
    Unresolved,
}

/// The handler in use on `stack`.
fn in_use(stack: &[Entry]) -> Option<Entry> {
    stack
        .last()
        .copied()
        .filter(|entry| !matches!(entry, Entry::Unresolved))
}

/// The `ADDRESS ... WITH` redirection of a command a handler written in Rexx
/// runs (`CommandIOContext`): the input lines, read one at a time, and the
/// lines written, which reach their targets when the command returns.
struct CommandIo {
    context: Rc<IoContext>,
    input: Option<Vec<Vec<u8>>>,
    next: usize,
    out: Vec<Vec<u8>>,
    err: Vec<Vec<u8>>,
}

/// The oracle's `CommandState`: what a `.CommandContext` reaches while its
/// command runs, found by the object its `CSELF` holds.
struct CommandState {
    cself: ObjRef,
    activity: ActivityId,
    activation: ActivationId,
    io: Option<CommandIo>,
}

/// What an environment name resolves to for a command.
pub(crate) enum Resolved {
    /// A handler object written in Rexx.
    Rexx(ObjRef),
    Native(NativeKind),
    /// Nothing, a name `.CommandHandler~deregister` took the last handler
    /// from included.
    Unknown,
}

/// The instance's command environments beyond the native table.
#[derive(Default)]
pub(crate) struct Environments {
    /// Each name whose handlers `.CommandHandler` has touched, the one in use
    /// last. An empty stack is a name `deregister` took out of the oracle's
    /// table, which only a classic subcommand handler answers again.
    stacks: FxHashMap<Box<[u8]>, Vec<Entry>>,
    natives: Vec<NativeEnvironment>,
    /// The commands handlers written in Rexx are running, innermost last.
    states: Vec<CommandState>,
}

impl Environments {
    /// Every object the table holds: the handlers, the native handlers'
    /// objects, and each running command's `CSELF` and redirection targets.
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        for stack in self.stacks.values() {
            out.extend(stack.iter().filter_map(|entry| match entry {
                Entry::Rexx(object) => Some(*object),
                Entry::Native(_) | Entry::Unresolved => None,
            }));
        }
        out.extend(self.natives.iter().filter_map(|native| native.object));
        for state in &self.states {
            out.push(state.cself);
            if let Some(io) = &state.io {
                io.context.object_roots(out);
            }
        }
    }

    /// Takes every handler an extension registered out of use, as the first
    /// library close empties the native table.
    pub(crate) fn close_registered(&mut self) {
        for native in &mut self.natives {
            if matches!(native.kind, NativeKind::Registered(_)) {
                native.kind = NativeKind::Closed;
            }
        }
    }

    /// `AddCommandEnvironment`'s `put`: the new handler replaces whatever
    /// stack the name had.
    pub(crate) fn replace(&mut self, upper: &[u8]) {
        self.stacks.remove(upper);
    }

    /// The running command whose `.CommandContext` holds `cself`.
    fn state_of(&self, cself: ObjRef) -> Option<usize> {
        self.states.iter().position(|state| state.cself == cself)
    }

    /// The native handler `object` stands for.
    fn native_of(&self, object: ObjRef) -> Option<usize> {
        self.natives
            .iter()
            .position(|native| native.object == Some(object))
    }
}

/// `liveState`'s two refusals.
fn ended() -> Failure {
    Raised::execution_user_defined("this CommandContext belongs to a command that has ended").into()
}

impl Interp {
    /// The native handler `resolveCommandHandler` finds for `environment`
    /// with nothing stacked under its name: an extension's, the built-in
    /// one's, or a classic subcommand handler.
    fn native_environment(&mut self, environment: &[u8]) -> Option<NativeKind> {
        let upper = environment.to_ascii_uppercase();
        if let Some(handler) = self.command_handlers.get(upper.as_slice()) {
            return Some(NativeKind::Registered(Rc::clone(handler)));
        }
        if let Some(handler) = handler_for(&upper) {
            return Some(NativeKind::System(handler));
        }
        self.classic_subcom(environment).map(NativeKind::Classic)
    }

    /// What a command to `environment` runs.
    pub(crate) fn resolve_environment(&mut self, environment: &[u8]) -> Resolved {
        let upper = environment.to_ascii_uppercase();
        if let Some(stack) = self.environments.stacks.get(upper.as_slice()) {
            return match stack.last() {
                None => match self.classic_subcom(environment) {
                    Some(entry) => Resolved::Native(NativeKind::Classic(entry)),
                    None => Resolved::Unknown,
                },
                Some(Entry::Unresolved) => Resolved::Unknown,
                Some(Entry::Rexx(object)) => Resolved::Rexx(*object),
                Some(Entry::Native(at)) => {
                    Resolved::Native(self.environments.natives[*at].kind.clone())
                }
            };
        }
        match self.native_environment(environment) {
            Some(kind) => Resolved::Native(kind),
            None => Resolved::Unknown,
        }
    }

    /// The handler in use under `environment`, its native one entered on a
    /// stack of the name's own first, so that it stays one object; `None`
    /// for a name nothing answers.
    fn environment_in_use(&mut self, environment: &[u8]) -> Option<Entry> {
        let upper: Box<[u8]> = environment.to_ascii_uppercase().into();
        let native = match self.environments.stacks.get(&upper) {
            None => Some(self.native_environment(environment)?),
            // Out of the table: only the API service can answer it again.
            Some(stack) if stack.is_empty() => {
                Some(NativeKind::Classic(self.classic_subcom(environment)?))
            }
            Some(_) => None,
        };
        if let Some(kind) = native {
            let at = self.add_native(kind, &upper);
            self.environments
                .stacks
                .insert(upper.clone(), vec![Entry::Native(at)]);
        }
        in_use(self.environments.stacks.get(&upper)?)
    }

    fn add_native(&mut self, kind: NativeKind, upper: &[u8]) -> usize {
        self.environments.natives.push(NativeEnvironment {
            kind,
            name: upper.to_vec(),
            object: None,
        });
        self.environments.natives.len() - 1
    }

    /// the handler object, or the native handler's `.NativeCommandHandler`,
    /// made on first use with `NAME` set and kept.
    fn entry_object(&mut self, entry: Entry) -> Result<ObjRef, Failure> {
        let at = match entry {
            Entry::Rexx(object) => return Ok(object),
            Entry::Native(at) => at,
            Entry::Unresolved => return Ok(ObjRef::NIL),
        };
        if let Some(object) = self.environments.natives[at].object {
            return Ok(object);
        }
        let name = self.environments.natives[at].name.clone();
        let object = self.native_handler_object(&name, at)?;
        self.environments.natives[at].object = Some(object);
        Ok(object)
    }

    /// `.CommandHandler~handler(name)`: the object of the handler in use, or
    /// `None`.
    pub(crate) fn environment_handler(
        &mut self,
        environment: &[u8],
    ) -> Result<Option<ObjRef>, Failure> {
        match self.environment_in_use(environment) {
            Some(entry) => self.entry_object(entry).map(Some),
            None => Ok(None),
        }
    }

    /// the objects of the handlers under `environment`, the one in use first.
    pub(crate) fn environment_handlers(
        &mut self,
        environment: &[u8],
    ) -> Result<Vec<ObjRef>, Failure> {
        let entries: Vec<Entry> = match self.environment_in_use(environment) {
            Some(_) => self.environments.stacks[environment.to_ascii_uppercase().as_slice()]
                .iter()
                .rev()
                .copied()
                .collect(),
            None => Vec::new(),
        };
        let mut objects = Vec::with_capacity(entries.len());
        for entry in entries {
            let object = self.entry_object(entry)?;
            self.roots.activity_mut().push_temp(object);
            objects.push(object);
        }
        Ok(objects)
    }

    /// Each environment name this instance knows with the object of its
    /// handler in use.
    pub(crate) fn environment_list(&mut self) -> Result<Vec<(Vec<u8>, ObjRef)>, Failure> {
        let mut names: Vec<Vec<u8>> = SYSTEM_NAMES.iter().map(|name| name.to_vec()).collect();
        names.extend(self.command_handlers.keys().map(|name| name.to_vec()));
        names.extend(self.classic_subcoms.keys().cloned());
        names.extend(self.environments.stacks.keys().map(|name| name.to_vec()));
        names.sort();
        names.dedup();
        let mut listed = Vec::new();
        for name in names {
            if let Some(object) = self.environment_handler(&name)? {
                self.roots.activity_mut().push_temp(object);
                listed.push((name, object));
            }
        }
        Ok(listed)
    }

    /// Registering a Rexx-object handler: `handler` covers the handler in use
    /// under `environment`, whose object this answers.
    pub(crate) fn register_environment(
        &mut self,
        environment: &[u8],
        handler: ObjRef,
    ) -> Result<Option<ObjRef>, Failure> {
        let covered = match self.environment_in_use(environment) {
            Some(entry) => Some(self.entry_object(entry)?),
            None => None,
        };
        let upper: Box<[u8]> = environment.to_ascii_uppercase().into();
        let stack = self.environments.stacks.entry(upper).or_default();
        // The new handler replaces an `UNRESOLVED` marker.
        stack.retain(|entry| !matches!(entry, Entry::Unresolved));
        stack.push(Entry::Rexx(handler));
        Ok(covered)
    }

    /// An extension's handler covers the one in use, as `register` covers it.
    pub(crate) fn stack_native_environment(&mut self, environment: &[u8], handler: CommandHandler) {
        let upper = environment.to_ascii_uppercase();
        let at = self.add_native(NativeKind::Registered(Rc::new(handler)), &upper);
        let _ = self.environment_in_use(environment);
        let stack = self.environments.stacks.entry(upper.into()).or_default();
        stack.retain(|entry| !matches!(entry, Entry::Unresolved));
        stack.push(Entry::Native(at));
    }

    /// Takes the handler in use off `environment`, uncovering the one it
    /// covered, and answers its object. A native one taken off the last
    /// leaves the name unknown for good.
    pub(crate) fn deregister_environment(
        &mut self,
        environment: &[u8],
    ) -> Result<Option<ObjRef>, Failure> {
        let Some(entry) = self.environment_in_use(environment) else {
            return Ok(None);
        };
        let removed = self.entry_object(entry)?;
        let classic = match entry {
            Entry::Native(at) => {
                matches!(self.environments.natives[at].kind, NativeKind::Classic(_))
            }
            Entry::Rexx(_) | Entry::Unresolved => false,
        };
        let upper = environment.to_ascii_uppercase();
        if let Some(stack) = self.environments.stacks.get_mut(upper.as_slice()) {
            stack.pop();
            if stack.is_empty() && classic {
                stack.push(Entry::Unresolved);
            }
        }
        Ok(Some(removed))
    }

    /// The native handler `object` stands for, its kind and its name, or
    /// 93.900 for any other object.
    pub(crate) fn native_handler_of(
        &mut self,
        object: ObjRef,
    ) -> Result<(NativeKind, Vec<u8>), Failure> {
        let found = match self.environments.native_of(object) {
            Some(at) => Some(at),
            None => self.copied_native_of(object),
        };
        match found {
            Some(at) => {
                let native = &self.environments.natives[at];
                Ok((native.kind.clone(), native.name.clone()))
            }
            None => Err(Raised::method_user_defined(
                "not a native command handler made by the interpreter",
            )
            .into()),
        }
    }

    /// The native handler a copy of a `.NativeCommandHandler` stands for,
    /// which the copy's variables name.
    fn copied_native_of(&mut self, object: ObjRef) -> Option<usize> {
        let class = self.rexx_package_class(b"NATIVECOMMANDHANDLER")?;
        let index = self.pools_of(object)?.get(class, NATIVE_AT)?;
        let text = self.default_string_value_text(index);
        let at: usize = std::str::from_utf8(&text).ok()?.parse().ok()?;
        (at < self.environments.natives.len()).then_some(at)
    }

    /// The running command whose context holds `cself`, or 98.900 once it has
    /// ended or from another activity.
    pub(crate) fn live_command(&self, cself: Option<ObjRef>) -> Result<usize, Failure> {
        let at = cself
            .and_then(|cself| self.environments.state_of(cself))
            .ok_or_else(ended)?;
        if self.environments.states[at].activity != self.running_activity() {
            return Err(Raised::execution_user_defined(
                "this CommandContext can only be used on the thread of its command",
            )
            .into());
        }
        Ok(at)
    }

    /// Whether the command whose context holds `cself` is running
    /// (`cmdcontext_isValid`).
    pub(crate) fn command_running(&self, cself: Option<ObjRef>) -> bool {
        cself.is_some_and(|cself| self.environments.state_of(cself).is_some())
    }

    /// The activation that issued the running command `at`.
    pub(crate) fn command_issuer(&self, at: usize) -> ActivationId {
        self.environments.states[at].activation
    }

    /// `isRedirected(which)`: input, output, error, and both outputs on one
    /// target.
    pub(crate) fn command_redirects(&self, at: usize) -> [bool; 4] {
        match &self.environments.states[at].io {
            None => [false; 4],
            Some(io) => [
                io.input.is_some(),
                io.context.redirects_output(),
                io.context.redirects_error(),
                io.context.shares_one_target(),
            ],
        }
    }

    /// The next line of the running command `at`'s input, or `None` at its
    /// end or with none.
    pub(crate) fn command_read_line(&mut self, at: usize) -> Option<Vec<u8>> {
        let io = self.environments.states[at].io.as_mut()?;
        let line = io.input.as_ref()?.get(io.next)?.clone();
        io.next += 1;
        Some(line)
    }

    /// `writeOutput` and `writeError`: one line for the running command
    /// `at`'s output or error, an error line going to the output where both
    /// share one target. Nothing where that stream is not redirected.
    pub(crate) fn command_write_line(&mut self, at: usize, error: bool, line: Vec<u8>) {
        let Some(io) = self.environments.states[at].io.as_mut() else {
            return;
        };
        let shared = io.context.shares_one_target();
        if error && !shared {
            if io.context.redirects_error() {
                io.err.push(line);
            }
        } else if (!error && io.context.redirects_output()) || (error && shared) {
            io.out.push(line);
        }
    }

    /// `handler~command(command, caller)`, on this activity, with a fresh
    /// `.CommandContext` as `caller`. Its answer is `RC`; a condition it asked
    /// for with `caller~raise` is raised at the command's clause, a `SYNTAX`
    /// one without the handler's frames and a `HALT` one as a halt of the
    /// issuing activation. An error inside the handler reaches the caller as
    /// it is, the redirection left unwritten.
    pub(super) fn run_rexx_command(
        &mut self,
        handler: ObjRef,
        environment: &[u8],
        command: &[u8],
        io: Option<&Rc<IoContext>>,
    ) -> Result<CommandOutcome, Failure> {
        let class = self
            .rexx_package_class(b"COMMANDCONTEXT")
            .ok_or_else(|| Loud::missing_rexx_class("CommandContext"))?;
        // Any object of its own identity: the context's `CSELF`.
        let cself = crate::dispatch::new_identity(self);
        self.roots.activity_mut().push_temp(cself);
        let address = self.text(environment);
        self.roots.activity_mut().push_temp(address);
        let issued = self.text(command);
        self.roots.activity_mut().push_temp(issued);
        let caller = self.caller();
        let context = self
            .send_message(class, b"NEW", None, &[Some(cself), Some(address)], caller)?
            .unwrap_or(ObjRef::NIL);
        self.roots.activity_mut().push_temp(context);
        if let Some(io) = io {
            io.begin(self)?;
        }
        self.environments.states.push(CommandState {
            cself,
            activity: self.running_activity(),
            activation: self.activation().id,
            io: io.map(|io| CommandIo {
                context: Rc::clone(io),
                input: io.input_lines(),
                next: 0,
                out: Vec::new(),
                err: Vec::new(),
            }),
        });
        let answered = self
            .send_message(
                handler,
                b"COMMAND",
                None,
                &[Some(issued), Some(context)],
                caller,
            )
            .and_then(|answer| {
                if let Some(answer) = answer {
                    self.roots.activity_mut().push_temp(answer);
                }
                // What the output streams' `charout` left, while still valid.
                self.send_message(context, b"FINISH", None, &[], caller)
                    .map(|_| answer)
            });
        // The context is dead from here on, whatever the handler did.
        let state = self
            .environments
            .state_of(cself)
            .map(|at| self.environments.states.remove(at));
        let answer = answered?;
        if let Some(CommandState {
            io: Some(written), ..
        }) = state
        {
            written
                .context
                .finish_lines(self, &written.out, &written.err)?;
        }
        let pending = self
            .send_message(context, b"PENDING", None, &[], caller)?
            .and_then(|pending| self.array_slots_of(pending));
        match pending {
            Some(request) => self.pending_command_condition(command, &request),
            None => {
                let object = match answer {
                    Some(value) => value,
                    None => self.counted(0),
                };
                self.supplied_outcome(object, None)
            }
        }
    }

    /// The condition a handler written in Rexx asked for, `[name, rc,
    /// description, additional]`.
    fn pending_command_condition(
        &mut self,
        command: &[u8],
        request: &[Option<ObjRef>],
    ) -> Result<CommandOutcome, Failure> {
        let item = |at: usize| {
            request
                .get(at)
                .copied()
                .flatten()
                .filter(|item| *item != ObjRef::NIL)
        };
        let name = match item(0) {
            Some(name) => self.string_value_text(name)?.to_ascii_uppercase(),
            None => Vec::new(),
        };
        let rc = item(1).unwrap_or(ObjRef::NIL);
        let additional = item(3);
        if name == b"SYNTAX" {
            // Raised here, at the caller's clause, with no frame of the
            // handler's; `rc` is the error number, major * 1000 + minor.
            let code = self.string_value_text(rc)?;
            let code = std::str::from_utf8(&code)
                .ok()
                .and_then(|code| code.trim().parse::<u64>().ok())
                .unwrap_or(0);
            let texts = additional
                .and_then(|array| self.array_slots_of(array))
                .unwrap_or_default()
                .into_iter()
                .map(|slot| {
                    slot.map(|object| self.unsent_string_value_text(object))
                        .unwrap_or_default()
                })
                .collect();
            if additional.is_some() {
                self.activity.pending_additional = additional;
            }
            return Err(crate::run::raised_for_code(code, texts).into());
        }
        let text = match item(2) {
            // A `String` subclass instance is read by its own characters
            // (spec: class.commandcontext.deviation.109); any other object
            // by its string value, not named as a message insert names it.
            Some(description) if crate::dispatch::subclass_text(self, description).is_some() => {
                self.string_value_text(description)?
            }
            Some(description) => {
                let text = self.required_string_value(description)?;
                self.string_value_text(text)?
            }
            None => command.to_vec(),
        };
        if name == b"HALT" {
            // Given back as a HALT, seen at the end of the clause as a
            // Ctrl-C is.
            self.request_running_halt(text);
            return self.supplied_outcome(rc, None);
        }
        let condition = HandlerCondition {
            name,
            description: Some(text),
            additional,
            result: Some(rc),
        };
        self.supplied_outcome(rc, Some(Box::new(condition)))
    }

    /// The outcome of a command whose `RC` is `object`, raising `condition`
    /// where there is one.
    fn supplied_outcome(
        &mut self,
        object: ObjRef,
        condition: Option<Box<HandlerCondition>>,
    ) -> Result<CommandOutcome, Failure> {
        self.roots.activity_mut().push_temp(object);
        let text = self.string_value_text(object)?;
        let status = match condition.as_ref().map(|held| held.name.as_slice()) {
            Some(b"FAILURE") => ReturnStatus::Failure,
            Some(b"ERROR") => ReturnStatus::Error,
            _ => ReturnStatus::Normal,
        };
        Ok(CommandOutcome {
            rc: whole_value(&text).unwrap_or(0),
            status,
            supplied: Some(Supplied { object, text }),
            condition,
        })
    }

    /// The running command `at` sent to the native handler `kind` registered
    /// under `name`, with the redirection the handler written in Rexx was
    /// given, already begun and written back by it. Answers `RC` and the
    /// condition the native handler raised as `[name, rc, description,
    /// additional]`, or `None`.
    pub(crate) fn forward_command(
        &mut self,
        at: usize,
        kind: NativeKind,
        name: &[u8],
        command: &[u8],
    ) -> Result<(ObjRef, Option<[ObjRef; 4]>), Failure> {
        let outcome = self.forwarded_outcome(at, kind, name, command)?;
        let result = match &outcome.supplied {
            Some(supplied) => supplied.object,
            None => self.text(outcome.rc.to_string().as_bytes()),
        };
        self.roots.activity_mut().push_temp(result);
        let pending = match (&outcome.condition, outcome.status.condition()) {
            (Some(held), _) => Some((
                held.name.clone(),
                held.result.unwrap_or(result),
                held.description.clone(),
                held.additional,
            )),
            (None, Some(condition)) => Some((
                condition.as_bytes().to_vec(),
                result,
                Some(command.to_vec()),
                None,
            )),
            (None, None) => None,
        };
        let Some((condition, rc, description, additional)) = pending else {
            return Ok((result, None));
        };
        let condition = self.text(&condition);
        self.roots.activity_mut().push_temp(condition);
        let description = match description {
            Some(text) => self.text(&text),
            None => ObjRef::NIL,
        };
        self.roots.activity_mut().push_temp(description);
        Ok((
            rc,
            Some([
                condition,
                rc,
                description,
                additional.unwrap_or(ObjRef::NIL),
            ]),
        ))
    }

    /// The outcome of forwarding the running command `at`.
    fn forwarded_outcome(
        &mut self,
        at: usize,
        kind: NativeKind,
        name: &[u8],
        command: &[u8],
    ) -> Result<CommandOutcome, Failure> {
        let io = self.environments.states[at]
            .io
            .as_ref()
            .map(|io| Rc::clone(&io.context));
        match kind {
            NativeKind::System(handler) => self.forward_to_system(at, &handler, command, io),
            NativeKind::Registered(handler) => {
                if io.is_some() && !handler.redirects() {
                    return Err(Raised::redirection_not_supported(name).into());
                }
                let redirector = match &self.environments.states[at].io {
                    None => Redirector::unrequested(),
                    Some(io) => Redirector::new(
                        io.input
                            .as_ref()
                            .map(|lines| lines[io.next.min(lines.len())..].to_vec()),
                        io.context.redirects_output(),
                        io.context.redirects_error(),
                        io.context.shares_one_target(),
                    ),
                };
                let handled = self.run_command_handler(&handler, name, command, &redirector)?;
                let (out, err) = redirector.finish();
                if let Some(io) = self
                    .environments
                    .states
                    .get_mut(at)
                    .and_then(|state| state.io.as_mut())
                {
                    io.out.extend(out);
                    io.err.extend(err);
                }
                self.registered_outcome(handled)
            }
            NativeKind::Classic(entry) => {
                self.run_classic_command(entry, name, command, io.is_some())
            }
            NativeKind::Closed => Ok(CommandOutcome::of_failure(NOT_REGISTERED)),
        }
    }

    /// The built-in handler run for a forwarded command: a child waited for
    /// on the baton, whose redirected streams become lines of the running
    /// command `at`'s own.
    fn forward_to_system(
        &mut self,
        at: usize,
        handler: &Handler,
        command: &[u8],
        io: Option<Rc<IoContext>>,
    ) -> Result<CommandOutcome, Failure> {
        if io.is_none()
            && matches!(handler, Handler::Shell(_))
            && let Some(outcome) = run_internally(self, command)
        {
            return Ok(outcome);
        }
        self.share_session_queue()?;
        self.hand_over_output();
        let switch_mode = self.child_switch_mode();
        let redirects = [
            io.as_ref().is_some_and(|io| io.redirects_output()),
            io.as_ref().is_some_and(|io| io.redirects_error()),
        ];
        let waited = match start(self, handler, command, io.as_deref(), switch_mode)? {
            Child::Ran(waited) => waited,
            Child::Running {
                worker,
                running,
                input,
                merged,
            } => {
                drop(worker);
                self.collect_on_baton(running, input, merged, [!redirects[0], !redirects[1]])?
            }
        };
        self.adopt_input_offset();
        if let Some(io) = self
            .environments
            .states
            .get_mut(at)
            .and_then(|state| state.io.as_mut())
        {
            if redirects[0] {
                io.out.extend(split_lines(&waited.out));
            }
            if redirects[1] && !io.context.shares_one_target() {
                io.err.extend(split_lines(&waited.err));
            }
        }
        if !redirects[0] {
            self.write_out(&waited.out);
        }
        if !redirects[1] {
            self.write_err(&waited.err);
        }
        Ok(CommandOutcome::of(waited.rc))
    }
}
