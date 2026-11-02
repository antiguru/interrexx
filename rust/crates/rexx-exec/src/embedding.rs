//! The classic embedding entry point, `RexxStart` (`api/rexx.h`), over safe
//! values: the `librexx` export reads the caller's C strings and writes the
//! answer back.

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
