//! The embedding entry points over safe values: the classic `RexxStart`
//! (`api/rexx.h`) and the instance API's `RexxCreateInterpreter`
//! (`api/oorexxapi.h`). The `librexx` exports read the caller's C structures
//! and write the answers back.

use std::io::{IsTerminal, Write};

use crate::invocation::{ProgramInput, Sinks, StartType};
use crate::{Invocation, launcher, run_program_named};

/// What `RexxStart` runs: the program `name`, or the in-store pair `instore`.
pub struct Start {
    pub name: Vec<u8>,
    /// `RexxStart`'s `instore` array, where the caller passed one.
    pub instore: Option<Instore>,
    pub arguments: Vec<Option<Vec<u8>>>,
    pub environment: Option<Vec<u8>>,
    pub start: StartType,
    /// The system exits, each a function code and the name of a registered exit handler.
    pub exits: Vec<(std::ffi::c_int, Vec<u8>)>,
}

/// `RexxStart`'s `instore` pair (`LanguageParser::processInstore`): the source, its image from
/// an earlier call, both, or neither -- which runs the macro `name` names.
#[derive(Default)]
pub struct Instore {
    pub source: Option<Vec<u8>>,
    pub image: Option<Vec<u8>>,
}

/// What `RexxStart` answers: its return code, negative an error number, the result string, its
/// value as a `short` (`None` leaves the caller's untouched), and the image of an in-store
/// source it translated, for `instore[1]`.
#[derive(Debug, PartialEq, Eq)]
pub struct Started {
    pub code: i32,
    pub result: Option<Vec<u8>>,
    pub short: Option<i16>,
    pub image: Option<Vec<u8>>,
}

/// `Error_Program_unreadable` (`messages/RexxErrorCodes.h`), negated as
/// `RexxStart` answers it.
const UNREADABLE: i32 = -3;

/// `RexxGetVersionInformation`'s text (`api/InterpreterAPI.cpp:250`), the banner `rexx -v`
/// writes.
pub fn version_information() -> Vec<u8> {
    launcher::version_banner()
}

/// Each message of `rexx_inventory::errors::MESSAGES` as a C string, in its order; built once
/// and kept, since `RexxGetErrorMessage` answers a pointer the caller never frees.
fn message_texts() -> &'static [std::ffi::CString] {
    static TEXTS: std::sync::OnceLock<Vec<std::ffi::CString>> = std::sync::OnceLock::new();
    TEXTS.get_or_init(|| {
        rexx_inventory::errors::MESSAGES
            .iter()
            .map(|message| std::ffi::CString::new(message.text).unwrap_or_default())
            .collect()
    })
}

/// `RexxGetErrorMessage` (`runtime/Interpreter.cpp:794`): the text of the message whose code
/// (major times 1000 plus minor) is `code`.
pub fn error_message(code: i32) -> Option<&'static std::ffi::CStr> {
    let at = rexx_inventory::errors::MESSAGES
        .iter()
        .position(|message| i32::from(message.major) * 1000 + i32::from(message.sub) == code)?;
    message_texts().get(at).map(std::ffi::CString::as_c_str)
}

/// `RexxGetErrorMessageByNumber` (`runtime/Interpreter.cpp:819`): the text of the message the
/// message number `number` maps to.
pub fn error_message_by_number(number: i32) -> Option<&'static std::ffi::CStr> {
    let message = crate::builtin::rexxutil::messages::mapped_message(number)?;
    error_message(i32::from(message.major) * 1000 + i32::from(message.sub))
}

/// A program `RexxStart` or a translation reads: its name, its text, and whether the text is
/// an in-store source whose image the caller is given back.
struct Loaded {
    path: Vec<u8>,
    text: Vec<u8>,
    image_wanted: bool,
}

/// Reads the program `name` and `instore` give (`RexxStartDispatcher::run`): a file, an image
/// from an earlier call (its source, where given too, is not read again), an in-store source,
/// or, with neither, the macro `name`. A refusal is the report written and the negated error.
fn load(name: &[u8], instore: Option<Instore>) -> Result<Loaded, (Vec<u8>, i32)> {
    let Some(instore) = instore else {
        return match launcher::load(name) {
            Ok((found, text)) => Ok(Loaded {
                path: found.into_os_string().into_encoded_bytes(),
                text,
                image_wanted: false,
            }),
            Err((report, _)) => Err((report, UNREADABLE)),
        };
    };
    let unreadable = || (launcher::program_unreadable(name).0, UNREADABLE);
    match instore {
        Instore {
            image: Some(image), ..
        } => Ok(Loaded {
            path: name.to_vec(),
            text: image,
            image_wanted: false,
        }),
        Instore {
            source: Some(source),
            image: None,
        } => Ok(Loaded {
            path: name.to_vec(),
            text: source,
            image_wanted: true,
        }),
        Instore {
            source: None,
            image: None,
        } => match crate::builtin::rexxutil::macros::macro_image(name) {
            Some(text) => Ok(Loaded {
                path: name.to_vec(),
                text,
                image_wanted: false,
            }),
            None => Err(unreadable()),
        },
    }
}

/// Writes a run's output to this process's standard output and error.
fn write_outcome(outcome: &crate::Outcome) {
    let _ = std::io::stdout().write_all(&outcome.stdout);
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().write_all(&outcome.stderr);
}

/// Runs `start` in an interpreter of its own, its output on this process's
/// standard output and error, and answers as `RexxStart` does
/// (`RexxStartDispatcher::run`): 0, the result and, where it is a whole
/// number a `short` holds, its value; or the negated error number of the
/// condition that ended it. A single command argument with a leading blank loses it, and a
/// single command argument `//T` translates the program without running it
/// (`api/InterpreterAPI.cpp:127`), leaving the `short` alone.
pub fn rexx_start(start: Start) -> Started {
    // The oracle's interpreter takes the halting signals (`SystemInterpreter`), so a SIGINT
    // during the run halts the program rather than ending the process, and the handlers stay
    // installed once it returns -- measured, `RexxStart` of `call SysSleep 5` answers -4 with
    // Error 4.1 reported, or the trapped halt's result.
    crate::timer::install_signals();
    let Start {
        name,
        instore,
        mut arguments,
        environment,
        start,
        exits,
    } = start;
    let command = start == StartType::Command;
    let translate_only =
        command && matches!(&arguments[..], [Some(only)] if only.eq_ignore_ascii_case(b"//T"));
    let loaded = match load(&name, instore) {
        Ok(loaded) => loaded,
        Err((report, code)) => {
            let _ = std::io::stderr().write_all(&report);
            return Started {
                code,
                result: None,
                short: (!translate_only).then(|| i16::try_from(code).unwrap_or(0)),
                image: None,
            };
        }
    };
    // An in-store source's image is saved once it has translated, before it runs
    // (`LanguageParser::processInstore`), so a run that fails still leaves it.
    let image = (loaded.image_wanted
        && crate::translate_program(&loaded.path, loaded.text.clone())
            .error
            .is_none())
    .then(|| crate::compiled::write_image(&loaded.text, &loaded.path, false));
    if translate_only {
        let outcome = crate::translate_program(&loaded.path, loaded.text);
        write_outcome(&outcome);
        let code = outcome.error.map_or(0, |number| -number);
        return Started {
            code,
            result: None,
            short: None,
            image: image.filter(|_| code == 0),
        };
    }
    if command
        && let [Some(only)] = &mut arguments[..]
        && only.len() > 1
        && only[0] == b' '
    {
        only.remove(0);
    }
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
        .with_arguments(arguments)
        .with_start(start)
        .with_exits(exits)
        .with_classic_thread(crate::signal::current_thread());
    if let Some(environment) = environment {
        invocation = invocation.with_address(environment);
    }
    let outcome = run_program_named(&loaded.path, loaded.text, invocation);
    write_outcome(&outcome);
    if let Some(number) = outcome.error {
        // Measured, the oracle's `rc` is the negated error number too.
        return Started {
            code: -number,
            result: None,
            short: Some(i16::try_from(-number).unwrap_or(0)),
            image,
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
        short: Some(short),
        image,
    }
}

/// `RexxTranslateProgram` and `RexxCompileProgram` (`TranslateDispatcher::run`): the program
/// file `input` translated and, where `output` names a file, its image written there, encoded
/// as `encode` says. An error is reported and answers its negated number.
pub fn translate_file(input: &[u8], output: Option<&[u8]>, encode: bool) -> i32 {
    let loaded = match load(input, None) {
        Ok(loaded) => loaded,
        Err((report, code)) => {
            let _ = std::io::stderr().write_all(&report);
            return code;
        }
    };
    let image = crate::compiled::write_image(&loaded.text, &loaded.path, encode);
    let outcome = crate::translate_program(&loaded.path, loaded.text);
    write_outcome(&outcome);
    if let Some(number) = outcome.error {
        return -number;
    }
    let Some(output) = output else {
        return 0;
    };
    let path = <std::ffi::OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(output);
    if std::fs::write(path, &image).is_err() {
        let _ = std::fs::remove_file(path);
        let (report, _) = launcher::output_unwritable(output);
        let _ = std::io::stderr().write_all(&report);
        return UNREADABLE;
    }
    0
}

/// `RexxTranslateInstoreProgram` (`TranslateInstoreDispatcher::run`): `source`, named `name`,
/// translated, and its image; an error is reported nowhere and answers its negated number.
pub fn translate_instore(name: &[u8], source: Vec<u8>) -> Result<Vec<u8>, i32> {
    let image = crate::compiled::write_image(&source, name, false);
    let outcome = crate::translate_program(name, source);
    match outcome.error {
        Some(number) => Err(-number),
        None => Ok(image),
    }
}

/// The `RexxStart` runs in flight, each reachable through the threads it runs on.
fn classic_runs() -> std::sync::MutexGuard<'static, Vec<([usize; 2], ClassicInbox)>> {
    static RUNS: std::sync::Mutex<Vec<([usize; 2], ClassicInbox)>> =
        std::sync::Mutex::new(Vec::new());
    RUNS.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

type ClassicInbox = std::sync::Arc<crate::timer::Inbox<crate::timer::Posted>>;

/// A `RexxStart` run registered as reachable by `RexxSetHalt` and `RexxSetTrace`, until this
/// is dropped.
pub(crate) struct ClassicRun([usize; 2]);

impl ClassicRun {
    /// Registers the run whose inbox is `inbox` under `threads`: the thread `RexxStart` was
    /// called on, and the one the interpreter runs on, which is where its handlers run.
    pub(crate) fn register(threads: [usize; 2], inbox: ClassicInbox) -> ClassicRun {
        classic_runs().push((threads, inbox));
        ClassicRun(threads)
    }
}

impl Drop for ClassicRun {
    fn drop(&mut self) {
        classic_runs().retain(|(threads, _)| *threads != self.0);
    }
}

/// The inbox of the `RexxStart` run on `thread`, if one is in flight.
fn classic_inbox(thread: usize) -> Option<ClassicInbox> {
    classic_runs()
        .iter()
        .find(|(threads, _)| threads.contains(&thread))
        .map(|(_, inbox)| std::sync::Arc::clone(inbox))
}

/// `RexxSetHalt` (`ActivityManager::haltActivity`): the run on `thread` halted at its next
/// clause; `false` where no run is in flight there.
pub fn classic_halt(thread: usize) -> bool {
    let Some(inbox) = classic_inbox(thread) else {
        return false;
    };
    inbox.post(crate::timer::Posted::Halt);
    inbox
        .requests()
        .set(crate::timer::HALT | crate::timer::BOUNDARY);
    true
}

/// `RexxSetTrace` and `RexxResetTrace` (`ActivityManager::setActivityTrace`): the external
/// trace of the run on `thread`, on or off; `false` where no run is in flight there.
pub fn classic_trace(thread: usize, on: bool) -> bool {
    let Some(inbox) = classic_inbox(thread) else {
        return false;
    };
    inbox.post(crate::timer::Posted::Trace(on));
    inbox.requests().set(crate::timer::BOUNDARY);
    true
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
    /// `REGISTERED_EXITS`: each a function code and the name of a registered exit handler.
    pub registered_exits: Vec<(std::ffi::c_int, Vec<u8>)>,
    /// `DIRECT_EXITS`: each a function code and its context handler.
    pub context_exits: Vec<(std::ffi::c_int, rexx_api::load::ContextExit)>,
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
    /// What the interpreter thread left unwritten, once it has ended.
    outcome: std::sync::Mutex<Option<crate::Outcome>>,
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
            let (outcome, kept) = crate::run_instance(
                options,
                Box::new(move |context, inbox| {
                    let _ = starting.inbox.set(inbox);
                    STARTING
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push((context, starting));
                }),
            );
            *ending
                .outcome
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(outcome);
            ending.state().ended = true;
            ending.changed.notify_all();
            if let Some(interp) = kept {
                interp.wind_down();
            }
        })
        .ok()?;
    drop(spawned);
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

/// Waits for the instance's interpreter thread to end its run, and writes
/// what it left unwritten. A call still in flight then is served on that
/// thread, which ends after it.
fn end(shared: &Shared) {
    let mut state = shared.state();
    while !state.ended {
        state = shared.wait(state);
    }
    drop(state);
    let outcome = shared
        .outcome
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    if let Some(outcome) = outcome {
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
