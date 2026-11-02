//! Classic C handlers: a registered function's `RexxFunctionHandler` and a registered
//! environment's `RexxSubcomHandler`, resolved through the API and kept once found.

use rexx_api::load::ClassicEntry;
use rexx_core::ObjRef;

use crate::capi::{self, Registry};
use crate::command::{CommandOutcome, ReturnStatus, Supplied, whole_value};
use crate::error::Raised;
use crate::{Failure, Interp};

/// `RXSUBCOM_ERROR`.
const SUBCOM_ERROR: u16 = 1;
/// `RXSUBCOM_FAILURE`.
const SUBCOM_FAILURE: u16 = 2;

impl Interp {
    /// The classic handler `name` names, kept from an earlier call or resolved now. A lookup
    /// starts no service.
    pub(crate) fn classic_function(&mut self, name: &[u8]) -> Option<ClassicEntry> {
        let upper = name.to_ascii_uppercase();
        if let Some(entry) = self.classic_functions.get(&upper) {
            return Some(*entry);
        }
        let entry = capi::resolve(Registry::Function, &upper, &self.library_search, false)
            .ok()
            .flatten()?;
        self.classic_functions.insert(upper, entry);
        Some(entry)
    }

    /// Resolves `name` as `RxFuncAdd` does once it has registered it, starting a service if
    /// needed, and keeps the handler. Whether it resolved.
    pub(crate) fn resolve_classic_function(&mut self, upper: &[u8]) -> bool {
        match capi::resolve(Registry::Function, upper, &self.library_search, true) {
            Ok(Some(entry)) => {
                self.classic_functions.insert(upper.to_vec(), entry);
                true
            }
            _ => false,
        }
    }

    /// Calls the classic function `name` with `arguments` as strings
    /// (`NativeActivation::callRegisteredRoutine`, `execution/NativeActivation.cpp:1466`). A
    /// non-zero return is 40.1, its traceback line the routine's `Compiled routine`.
    pub(crate) fn call_classic_function(
        &mut self,
        name: &[u8],
        arguments: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let Some(entry) = self.classic_function(name) else {
            return Err(Raised::routine_not_found(name).into());
        };
        let mut texts = Vec::with_capacity(arguments.len());
        for argument in arguments {
            texts.push(match argument {
                Some(value) => Some(self.string_value_text(*value)?),
                None => None,
            });
        }
        let queue = self.current_queue_name()?;
        let borrowed: Vec<Option<&[u8]>> = texts.iter().map(Option::as_deref).collect();
        let called = self.run_classic_handler(name, arguments, || {
            entry.call_function(name, &borrowed, &queue)
        })?;
        if called.rc != 0 {
            self.blame_native_routine(name, None, arguments, crate::RoutineObject::OfCode, None);
            return Err(Raised::routine_failed(name).into());
        }
        Ok(called.value.map(|bytes| self.text(&bytes)))
    }

    /// The name `RXQUEUE('Get')` answers.
    pub(crate) fn current_queue_name(&mut self) -> Result<Vec<u8>, Failure> {
        let Some(queue) = self.local_route(b"STDQUE")? else {
            return Ok(b"SESSION".to_vec());
        };
        let caller = self.caller();
        match self.send_message(queue, b"GET", None, &[], caller)? {
            Some(name) => self.string_value_text(name),
            None => Ok(b"SESSION".to_vec()),
        }
    }

    /// The classic subcommand handler registered for `environment`, kept once resolved: a
    /// handler dropped afterwards stays usable here. A lookup starts no service.
    pub(crate) fn classic_subcom(&mut self, environment: &[u8]) -> Option<ClassicEntry> {
        let upper = environment.to_ascii_uppercase();
        if let Some(entry) = self.classic_subcoms.get(&upper) {
            return Some(*entry);
        }
        let entry = capi::resolve(Registry::Subcom, environment, &self.library_search, false)
            .ok()
            .flatten()?;
        self.classic_subcoms.insert(upper, entry);
        Some(entry)
    }

    /// `CommandHandlerDispatcher` (`concurrency/CommandHandler.cpp`): a non-zero return code is
    /// `RC`, else the result string, else 0; the flags raise `FAILURE` or `ERROR`. A redirected
    /// command is 98.921.
    pub(crate) fn run_classic_command(
        &mut self,
        entry: ClassicEntry,
        environment: &[u8],
        command: &[u8],
        redirected: bool,
    ) -> Result<CommandOutcome, Failure> {
        if redirected {
            return Err(Raised::redirection_not_supported(environment).into());
        }
        let (called, flags) = entry.call_subcom(command);
        let text = match (called.rc, called.value) {
            (0, Some(value)) => value,
            (0, None) => b"0".to_vec(),
            (rc, _) => rc.to_string().into_bytes(),
        };
        let object = self.text(&text);
        let status = if flags & SUBCOM_FAILURE != 0 {
            ReturnStatus::Failure
        } else if flags & SUBCOM_ERROR != 0 {
            ReturnStatus::Error
        } else {
            ReturnStatus::Normal
        };
        Ok(CommandOutcome {
            rc: whole_value(&text).unwrap_or(0),
            status,
            supplied: Some(Supplied { object, text }),
            condition: None,
        })
    }
}
