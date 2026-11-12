//! System exits (the `rexxpg` "System Exit Interface"): the handlers an embedding installs
//! through `RexxStart`'s exit list or `RexxCreateInterpreter`'s `REGISTERED_EXITS` and
//! `DIRECT_EXITS`, and the points of a run that call them, each before what it would
//! otherwise do.
//!
//! A handler that answers `RXEXIT_NOT_HANDLED` leaves the run to do what it does without one;
//! one that answers `RXEXIT_RAISE_ERROR` raises 48.1 naming the exit, and an I/O exit that
//! does so is removed first, so that reporting that error does not call it again.

use std::ffi::c_int;

use interrexx_api::load::{ContextExit, ExitHandler, ExitReply, ExitRequest, exit_code};
use interrexx_core::ObjRef;

use crate::activation::CallType;
use crate::capi::{self, Registry};
use crate::command::{CommandOutcome, ReturnStatus, Supplied, whole_value};
use crate::error::Raised;
use crate::{Failure, Interp};

/// The exits one interpreter has installed, by function code.
pub(crate) struct Exits {
    handlers: [Option<ExitHandler>; exit_code::COUNT],
    /// The interpreter is an embedding instance, whose own root program runs no
    /// initialization or termination exit: only the programs it is asked to call do.
    instance: bool,
    /// An instance's root program, which stands in for a native root: no clause exit runs
    /// between its clauses and no halt it takes is cleared.
    root: Option<crate::ProgramId>,
}

impl Exits {
    fn get(&self, code: c_int) -> Option<ExitHandler> {
        usize::try_from(code)
            .ok()
            .and_then(|code| self.handlers.get(code))
            .copied()
            .flatten()
    }
}

/// `rxfferr`, `rxfcfail`, `rxfhhalt`, `rxftrace` and `rxfmlifo`: a block's first flag.
const FIRST: u32 = 1;
/// `rxffnfnd` and `rxfcerr`: a block's second flag.
const SECOND: u32 = 2;

impl Interp {
    /// Installs `registered`, each a function code and the name of a registered exit handler,
    /// resolved now as `RexxResolveExit` resolves it (an unresolved name installs nothing),
    /// then `context`, each a code and a context handler; a later entry for a code replaces an
    /// earlier one. `instance` says this interpreter is an embedding instance.
    pub(crate) fn install_exits(
        &mut self,
        registered: &[(c_int, Vec<u8>)],
        context: &[(c_int, ContextExit)],
        instance: bool,
    ) {
        let mut exits = Exits {
            handlers: [None; exit_code::COUNT],
            instance,
            root: None,
        };
        let mut set = |code: c_int, handler: Option<ExitHandler>| {
            if let Some(slot) = usize::try_from(code)
                .ok()
                .and_then(|code| exits.handlers.get_mut(code))
            {
                *slot = handler;
            }
        };
        for (code, name) in registered {
            let handler =
                capi::resolve(Registry::Exit, name, &self.library_search, &self.cwd, true)
                    .ok()
                    .flatten()
                    .map(ExitHandler::Registered);
            set(*code, handler);
        }
        for (code, handler) in context {
            set(*code, Some(ExitHandler::Context(*handler)));
        }
        if exits.handlers.iter().all(Option::is_none) {
            return;
        }
        self.clause_exits =
            exits.get(exit_code::HLT).is_some() || exits.get(exit_code::TRC).is_some();
        self.exits = Some(Box::new(exits));
    }

    /// Whether an exit is installed for `code`.
    fn exit_installed(&self, code: c_int) -> bool {
        self.exits
            .as_ref()
            .is_some_and(|exits| exits.get(code).is_some())
    }

    /// Whether the main program of this run calls the initialization and termination exits:
    /// any main program but an embedding instance's own root.
    pub(crate) fn main_calls_program_exits(&self) -> bool {
        self.exits.as_ref().is_some_and(|exits| !exits.instance) && !self.library_bootstrap
    }

    /// Whether a routine or a called program calls the initialization and termination exits.
    pub(crate) fn calls_program_exits(&self) -> bool {
        self.exits.is_some() && !self.library_bootstrap
    }

    /// Calls the exit `request` names, where one is installed: the reply of a handler
    /// that handled it, `None` where none is installed or it passed.
    fn exit_call(
        &mut self,
        request: ExitRequest<'_, Option<ObjRef>>,
    ) -> Result<Option<ExitReply<Option<ObjRef>>>, Failure> {
        let (code, _) = request.codes();
        let Some(handler) = self.exits.as_ref().and_then(|exits| exits.get(code)) else {
            return Ok(None);
        };
        match self.run_exit(handler, request)? {
            interrexx_api::load::ExitOutcome::Handled(reply) => Ok(Some(reply)),
            interrexx_api::load::ExitOutcome::NotHandled
            | interrexx_api::load::ExitOutcome::Unwound => Ok(None),
            interrexx_api::load::ExitOutcome::Failed => {
                if code == exit_code::SIO
                    && let Some(exits) = &mut self.exits
                {
                    exits.handlers[exit_code::SIO as usize] = None;
                }
                Err(Raised::syntax(48, 1, vec![exit_code::name(code).as_bytes().to_vec()]).into())
            }
        }
    }

    /// The object-valued then the string-valued function exit, for a call that gets past the
    /// running package's own routines: the value of a call one of them handled, `None` where
    /// neither did.
    pub(crate) fn function_exits(
        &mut self,
        name: &[u8],
        call_type: CallType,
        arguments: &[Option<ObjRef>],
    ) -> Result<Option<Option<ObjRef>>, Failure> {
        let subroutine = call_type != CallType::Function;
        if self.exit_installed(exit_code::OFNC) {
            let request = ExitRequest::ObjectFunction {
                scripting: false,
                name,
                subroutine,
                arguments: arguments.to_vec(),
            };
            if let Some(reply) = self.exit_call(request)? {
                return function_answer(name, subroutine, reply.flags, reply.object).map(Some);
            }
        }
        if self.exit_installed(exit_code::FNC) {
            let mut texts = Vec::with_capacity(arguments.len());
            for argument in arguments {
                texts.push(match argument {
                    Some(value) => Some(self.string_value_text(*value)?),
                    None => None,
                });
            }
            let queue = self.current_queue_name()?;
            let borrowed: Vec<Option<&[u8]>> = texts.iter().map(Option::as_deref).collect();
            let request = ExitRequest::Function {
                name,
                subroutine,
                queue: &queue,
                arguments: &borrowed,
            };
            if let Some(reply) = self.exit_call(request)? {
                let value = reply.text.map(|text| self.text(&text));
                return function_answer(name, subroutine, reply.flags, value).map(Some);
            }
        }
        Ok(None)
    }

    /// The scripting function exit, the last step before 43.1.
    pub(crate) fn scripting_exit(
        &mut self,
        name: &[u8],
        call_type: CallType,
        arguments: &[Option<ObjRef>],
    ) -> Result<Option<Option<ObjRef>>, Failure> {
        let subroutine = call_type != CallType::Function;
        let request = ExitRequest::ObjectFunction {
            scripting: true,
            name,
            subroutine,
            arguments: arguments.to_vec(),
        };
        match self.exit_call(request)? {
            Some(reply) => function_answer(name, subroutine, reply.flags, reply.object).map(Some),
            None => Ok(None),
        }
    }

    /// The command exit: the outcome of a command it handled, its return string the `RC`,
    /// `rxfcfail` a `FAILURE` and `rxfcerr` an `ERROR`.
    pub(crate) fn command_exit(
        &mut self,
        address: &[u8],
        command: &[u8],
    ) -> Result<Option<CommandOutcome>, Failure> {
        let Some(reply) = self.exit_call(ExitRequest::Command { address, command })? else {
            return Ok(None);
        };
        let text = reply.text.unwrap_or_default();
        let status = if reply.flags & FIRST != 0 {
            ReturnStatus::Failure
        } else if reply.flags & SECOND != 0 {
            ReturnStatus::Error
        } else {
            ReturnStatus::Normal
        };
        let object = self.text(&text);
        Ok(Some(CommandOutcome {
            rc: whole_value(&text).unwrap_or(0),
            status,
            supplied: Some(Supplied { object, text }),
            condition: None,
        }))
    }

    /// The queue exit's pull: the line a handler returned, `Some(None)` where it returned
    /// none, which reads as an empty queue.
    pub(crate) fn pull_exit(&mut self) -> Result<Option<Option<Vec<u8>>>, Failure> {
        Ok(self.exit_call(ExitRequest::Pull)?.map(|reply| reply.text))
    }

    /// The queue exit's push or queue: whether a handler took the line.
    pub(crate) fn push_exit(&mut self, line: &[u8], lifo: bool) -> Result<bool, Failure> {
        Ok(self.exit_call(ExitRequest::Push { line, lifo })?.is_some())
    }

    /// The queue exit's size.
    pub(crate) fn queue_size_exit(&mut self) -> Result<Option<usize>, Failure> {
        Ok(self
            .exit_call(ExitRequest::QueueSize)?
            .map(|reply| reply.size))
    }

    /// The queue exit's new queue name: the name a handler put in place of `name`.
    pub(crate) fn queue_name_exit(&mut self, name: &[u8]) -> Result<Option<Vec<u8>>, Failure> {
        Ok(self
            .exit_call(ExitRequest::QueueName { name })?
            .map(|reply| reply.text.unwrap_or_default()))
    }

    /// The I/O exit's `SAY`: whether a handler took the line.
    pub(crate) fn say_exit(&mut self, line: &[u8]) -> Result<bool, Failure> {
        Ok(self.exit_call(ExitRequest::Say { line })?.is_some())
    }

    /// The I/O exit's trace line: whether a handler took it.
    pub(crate) fn trace_exit(&mut self, line: &[u8]) -> Result<bool, Failure> {
        Ok(self.exit_call(ExitRequest::Trace { line })?.is_some())
    }

    /// The I/O exit's terminal read: the line a handler returned.
    pub(crate) fn terminal_read_exit(&mut self) -> Result<Option<Vec<u8>>, Failure> {
        Ok(self
            .exit_call(ExitRequest::TerminalRead)?
            .map(|reply| reply.text.unwrap_or_default()))
    }

    /// The I/O exit's interactive debug read.
    pub(crate) fn debug_read_exit(&mut self) -> Result<Option<Vec<u8>>, Failure> {
        Ok(self
            .exit_call(ExitRequest::DebugRead)?
            .map(|reply| reply.text.unwrap_or_default()))
    }

    /// The halt and trace exits at a clause boundary: a halt the halt exit asks for is raised
    /// there, and a trace the trace exit asks for is turned on as `RXTRACE` turns it on. The
    /// library's own code calls neither. Before them, what a native call posted to this
    /// interpreter's own inbox is served (`boundary_posts`).
    #[cold]
    #[inline(never)]
    pub(crate) fn run_clause_exits(&mut self) -> Result<(), Failure> {
        // Served at a boundary of the activation that made the call, or of one it returned to:
        // not at one of an `INTERPRET` a debug pause runs above it, whose setting a trace
        // request does not reach.
        if let Some(depth) = self.boundary_posts
            && self.activation_depth() <= depth
        {
            self.boundary_posts = None;
            self.clause_exits =
                self.exit_installed(exit_code::HLT) || self.exit_installed(exit_code::TRC);
            self.drain_completions();
        }
        if !self.exit_installed(exit_code::HLT) && !self.exit_installed(exit_code::TRC) {
            return Ok(());
        }
        let Some(program) = self.running_activation().map(|running| running.program_id) else {
            return Ok(());
        };
        if self.library_programs.contains(&program) || self.is_instance_root(program) {
            return Ok(());
        }
        if let Some(reply) = self.exit_call(ExitRequest::HaltTest)?
            && reply.flags & FIRST != 0
        {
            self.halt_running();
        }
        if let Some(reply) = self.exit_call(ExitRequest::TraceTest)?
            && reply.flags & FIRST != 0
            && self.running_activation().is_some()
        {
            self.set_trace_mode(crate::trace::TraceMode {
                debug: true,
                ..crate::trace::TraceMode::RESULTS
            });
        }
        Ok(())
    }

    /// Marks `program` as this instance's root program.
    pub(crate) fn set_instance_root(&mut self, program: crate::ProgramId) {
        if let Some(exits) = self.exits.as_mut() {
            exits.root = Some(program);
        }
    }

    /// Whether `program` is this instance's root program.
    fn is_instance_root(&self, program: crate::ProgramId) -> bool {
        self.exits
            .as_ref()
            .is_some_and(|exits| exits.root == Some(program))
    }

    /// The running Rexx frame's external trace, `TRACE ?R` on and `TRACE O` off. An
    /// idle activity's frame is not reached.
    pub(crate) fn set_external_trace(&mut self, on: bool) {
        if self.running_activation().is_none() {
            return;
        }
        let mode = if on {
            crate::trace::TraceMode {
                debug: true,
                ..crate::trace::TraceMode::RESULTS
            }
        } else {
            crate::trace::TraceMode::OFF
        };
        self.set_trace_mode(mode);
    }

    /// The halt exit's clear, once a trap has taken a `HALT`, unless the trap is the instance
    /// root's own.
    pub(crate) fn halt_clear_exit(&mut self) -> Result<(), Failure> {
        let root = self
            .running_activation()
            .is_some_and(|running| self.is_instance_root(running.program_id));
        if root {
            return Ok(());
        }
        self.exit_call(ExitRequest::HaltClear).map(drop)
    }

    /// The initialization exit, before a program or routine's first clause.
    pub(crate) fn initialization_exit(&mut self) -> Result<(), Failure> {
        self.exit_call(ExitRequest::Initialization).map(drop)
    }

    /// The termination exit, once a program or routine has returned or exited.
    pub(crate) fn termination_exit(&mut self) -> Result<(), Failure> {
        self.exit_call(ExitRequest::Termination).map(drop)
    }

    /// The `NOVALUE` exit: the value a handler gave the variable `name`.
    pub(crate) fn novalue_exit(&mut self, name: ObjRef) -> Result<Option<Option<ObjRef>>, Failure> {
        Ok(self
            .exit_call(ExitRequest::Novalue { name: Some(name) })?
            .map(|reply| reply.object))
    }

    /// The `VALUE` exit, for a selector nothing else knows: the old value a handler returned.
    pub(crate) fn value_exit(
        &mut self,
        selector: ObjRef,
        name: ObjRef,
        value: Option<ObjRef>,
    ) -> Result<Option<Option<ObjRef>>, Failure> {
        let request = ExitRequest::Value {
            selector: Some(selector),
            name: Some(name),
            value,
        };
        Ok(self.exit_call(request)?.map(|reply| reply.object))
    }
}

/// What a function exit that handled a call answers: `rxfferr` is 40.1, `rxffnfnd` 43.1, and a
/// function given no value 44.1.
fn function_answer(
    name: &[u8],
    subroutine: bool,
    flags: u32,
    value: Option<ObjRef>,
) -> Result<Option<ObjRef>, Failure> {
    if flags & FIRST != 0 {
        return Err(Raised::routine_failed(name).into());
    }
    if flags & SECOND != 0 {
        return Err(Raised::routine_not_found(name).into());
    }
    if value.is_none() && !subroutine {
        return Err(Raised::no_data_returned(name).into());
    }
    Ok(value)
}
