//! The embedding entry points over safe values: the classic `RexxStart`
//! (`api/rexx.h`) and the instance API's `RexxCreateInterpreter`
//! (`api/oorexxapi.h`). The `librexx` exports read the caller's C structures
//! and write the answers back.

use std::io::{IsTerminal, Write};

use crate::invocation::{ProgramInput, Sinks, StartType};
use crate::{Invocation, launcher, run_program_named};

/// What `RexxStart` runs: `name` and, for an in-store program, its source.
pub struct Start {
    pub name: Vec<u8>,
    pub source: Option<Vec<u8>>,
    pub arguments: Vec<Option<Vec<u8>>>,
    pub environment: Option<Vec<u8>>,
    pub start: StartType,
}

/// What `RexxStart` answers: its return code, negative an error number, the
/// result string and its value as a `short`.
#[derive(Debug, PartialEq, Eq)]
pub struct Started {
    pub code: i32,
    pub result: Option<Vec<u8>>,
    pub short: i16,
}

/// `Error_Program_unreadable` (`messages/RexxErrorCodes.h`), negated as
/// `RexxStart` answers it.
const UNREADABLE: i32 = -3;

/// Runs `start` in an interpreter of its own, its output on this process's
/// standard output and error, and answers as `RexxStart` does
/// (`RexxStartDispatcher::run`): 0, the result and, where it is a whole
/// number a `short` holds, its value; or the negated error number of the
/// condition that ended it.
pub fn rexx_start(start: Start) -> Started {
    let (path, text) = match start.source {
        Some(source) => (start.name, source),
        None => match launcher::load(&start.name) {
            Ok((found, text)) => (found.into_os_string().into_encoded_bytes(), text),
            Err(_) => {
                return Started {
                    code: UNREADABLE,
                    result: None,
                    short: 0,
                };
            }
        },
    };
    let mut invocation = Invocation::none()
        .with_input(ProgramInput::Stdin)
        .with_sinks(Sinks {
            stdout: Box::new(|bytes| {
                let mut stdout = std::io::stdout();
                stdout.write_all(bytes).and_then(|()| stdout.flush())
            }),
            stderr: Box::new(|bytes| std::io::stderr().write_all(bytes)),
            stdout_terminal: std::io::stdout().is_terminal(),
            own_descriptors: true,
        })
        .with_arguments(start.arguments)
        .with_start(start.start);
    if let Some(environment) = start.environment {
        invocation = invocation.with_address(environment);
    }
    let outcome = run_program_named(&path, text, invocation);
    let _ = std::io::stdout().write_all(&outcome.stdout);
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().write_all(&outcome.stderr);
    if let Some(number) = outcome.error {
        return Started {
            code: -number,
            result: None,
            short: 0,
        };
    }
    let short = outcome
        .result
        .as_deref()
        .and_then(|text| std::str::from_utf8(text).ok())
        .and_then(|text| text.trim().parse::<i16>().ok())
        .unwrap_or(0);
    Started {
        code: 0,
        result: outcome.result,
        short,
    }
}

/// What `RexxCreateInterpreter`'s options set up, read from the caller's
/// `RexxOption` list.
#[derive(Default)]
pub struct InstanceOptions {
    /// `INITIAL_ADDRESS_ENVIRONMENT`.
    pub initial_address: Option<Vec<u8>>,
    /// `APPLICATION_DATA`, the instance's `applicationData`.
    pub application_data: usize,
    /// `DIRECT_ENVIRONMENTS` and `REDIRECTING_ENVIRONMENTS`.
    pub command_handlers: Vec<(Vec<u8>, rexx_api::load::SentCommandHandler)>,
    /// `REGISTERED_ENVIRONMENTS`: an environment name and the classic
    /// subcommand handler registered under the second name that serves it.
    pub registered_environments: Vec<(Vec<u8>, Vec<u8>)>,
    /// `LOAD_REQUIRED_LIBRARY`.
    pub load_libraries: Vec<Vec<u8>>,
    /// `EXTERNAL_CALL_PATH`: searched after the current directory.
    pub external_call_path: Option<Vec<u8>>,
    /// `EXTERNAL_CALL_EXTENSIONS`, split at its commas: tried before the
    /// default extensions.
    pub external_call_extensions: Vec<Vec<u8>>,
}

/// Where an instance has got to, under [`Shared::state`].
#[derive(Default)]
struct State {
    /// The root thread context, once the root activity waits in it.
    context: Option<usize>,
    /// `Terminate` was called.
    terminating: bool,
    /// The interpreter thread has ended.
    ended: bool,
}

/// One instance, as the threads that use it share it.
#[derive(Default)]
struct Shared {
    state: std::sync::Mutex<State>,
    changed: std::sync::Condvar,
    inbox: std::sync::OnceLock<crate::sync::Arc<crate::timer::Inbox<crate::timer::Posted>>>,
    thread: std::sync::Mutex<Option<std::thread::JoinHandle<crate::Outcome>>>,
}

impl Shared {
    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn wait<'a>(
        &self,
        state: std::sync::MutexGuard<'a, State>,
    ) -> std::sync::MutexGuard<'a, State> {
        self.changed
            .wait(state)
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// The live instances, by root thread context address.
static INSTANCES: std::sync::Mutex<Vec<(usize, std::sync::Arc<Shared>)>> =
    std::sync::Mutex::new(Vec::new());

fn instances() -> std::sync::MutexGuard<'static, Vec<(usize, std::sync::Arc<Shared>)>> {
    INSTANCES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn instance(context: usize) -> Option<std::sync::Arc<Shared>> {
    instances()
        .iter()
        .find(|(at, _)| *at == context)
        .map(|(_, shared)| std::sync::Arc::clone(shared))
}

/// Instances not yet waiting in their root call, by the interpreter thread's
/// id, which [`wait_in_root`] finds its own in.
static STARTING: std::sync::Mutex<Vec<(usize, std::sync::Arc<Shared>)>> =
    std::sync::Mutex::new(Vec::new());

/// `RexxCreateInterpreter`: an interpreter of its own on a thread of its own,
/// whose root activity waits in a native call that the calling thread, now
/// attached to it, acts as; answers the root thread context's address, or
/// `None` where the interpreter did not start.
pub fn create_instance(options: InstanceOptions) -> Option<usize> {
    rexx_api::ffi::set_embedder(&EMBEDDER);
    // The oracle's instance takes the halting signals (`SystemInterpreter`),
    // which reach every live interpreter; the caller's mask is its own.
    crate::timer::install_signals();
    let shared = std::sync::Arc::new(Shared::default());
    let starting = std::sync::Arc::clone(&shared);
    let ending = std::sync::Arc::clone(&shared);
    let spawned = std::thread::Builder::new()
        .name("rexx-interp".to_string())
        .stack_size(crate::INTERPRETER_STACK_BYTES)
        .spawn(move || {
            crate::signal::receive();
            let outcome = crate::run_instance(
                options,
                Box::new(move |context, inbox| {
                    let _ = starting.inbox.set(inbox);
                    STARTING
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push((context, starting));
                }),
            );
            ending.state().ended = true;
            ending.changed.notify_all();
            outcome
        })
        .ok()?;
    *shared
        .thread
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(spawned);
    let mut state = shared.state();
    while state.context.is_none() && !state.ended {
        state = shared.wait(state);
    }
    let context = state.context;
    drop(state);
    match context {
        Some(context) => {
            rexx_api::ffi::attach_current_thread(context);
            Some(context)
        }
        None => {
            end(&shared);
            None
        }
    }
}

/// The root call's body, on the thread running it: the instance is live
/// until `Terminate`. A thread attached and idle is not waited for, as the
/// oracle's `terminate` removes inactive activities; one inside a callback
/// holds the baton the call's end needs, so the instance ends after it.
pub(crate) fn wait_in_root(context: usize) {
    let shared = {
        let mut starting = STARTING
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(at) = starting.iter().position(|(at, _)| *at == context) else {
            return;
        };
        starting.remove(at).1
    };
    instances().push((context, std::sync::Arc::clone(&shared)));
    let mut state = shared.state();
    state.context = Some(context);
    shared.changed.notify_all();
    while !state.terminating {
        state = shared.wait(state);
    }
}

/// Joins the instance's interpreter thread and writes what it left unwritten.
fn end(shared: &Shared) {
    let thread = shared
        .thread
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    if let Some(thread) = thread
        && let Ok(outcome) = thread.join()
    {
        let _ = std::io::stdout().write_all(&outcome.stdout);
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().write_all(&outcome.stderr);
    }
}

/// The [`rexx_api::ffi::Embedder`] the instance members reach.
struct Embedder;

static EMBEDDER: Embedder = Embedder;

impl rexx_api::ffi::Embedder for Embedder {
    fn terminate(&self, context: usize) -> bool {
        let Some(shared) = instance(context) else {
            return false;
        };
        {
            let mut state = shared.state();
            if state.terminating {
                return false;
            }
            state.terminating = true;
            shared.changed.notify_all();
        }
        end(&shared);
        instances().retain(|(at, _)| *at != context);
        true
    }

    fn halt(&self, context: usize) -> bool {
        let Some(shared) = instance(context) else {
            return false;
        };
        if let Some(inbox) = shared.inbox.get() {
            inbox.post(crate::timer::Posted::Halt);
            inbox.requests().set(crate::timer::HALT);
        }
        true
    }

    fn set_trace(&self, _context: usize, _on: bool) -> bool {
        false
    }
}
