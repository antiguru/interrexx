//! Run the CLI inside the interpreter that owns the process's native API.

use interrexx_cast::CastTruncate;
use std::io::{IsTerminal, Write};

use super::{Command, Program};

/// Runs the interpreter with this process's host arguments and standard streams.
pub fn run_cli() -> u8 {
    crate::install_signal_handlers();
    let args: Vec<Vec<u8>> = std::env::args_os()
        .skip(1)
        .map(std::ffi::OsString::into_encoded_bytes)
        .collect();
    let run = match super::parse(&args) {
        Command::Usage => {
            let _ = std::io::stderr().write_all(super::USAGE);
            return super::USAGE_STATUS;
        }
        Command::Version => {
            let mut banner = super::version_banner();
            banner.push(b'\n');
            let _ = std::io::stdout().write_all(&banner);
            let _ = std::io::stdout().flush();
            return 0;
        }
        Command::Run(run) => run,
    };

    let mut invocation = crate::join_command_line(&run.words)
        .with_input(crate::ProgramInput::Stdin)
        .with_standard_transient(transient_standard_streams())
        // After what native code wrote through C's streams, in that order.
        .with_sinks(crate::Sinks {
            stdout: Box::new(|bytes| {
                crate::flush_c_streams();
                let mut stdout = std::io::stdout();
                stdout.write_all(bytes).and_then(|()| stdout.flush())
            }),
            stderr: Box::new(|bytes| {
                crate::flush_c_streams();
                std::io::stderr().write_all(bytes)
            }),
            stdout_terminal: std::io::stdout().is_terminal(),
            unbuffered: run.unbuffered || crate::unbuffered_requested(),
            own_descriptors: true,
        });

    // `REXX_HASH_SEED` fixes the seed of the keyed string hash. The
    // interpreter reads it, and passes over a value it cannot read, so that
    // value is refused here. An empty value leaves the seed unset.
    if let Some(value) = std::env::var_os(crate::HASH_SEED_ENV)
        && let Err(message) = crate::parse_hash_seed(value.as_encoded_bytes())
    {
        eprintln!("rexx: {message}");
        return 2;
    }

    // `REXX_HEAP_LIMIT` bounds the heap (r[env.invocation.heap-limit]). An
    // embedded run passes over a value it cannot read, so that value is
    // refused here.
    if let Some(value) = std::env::var_os(crate::HEAP_LIMIT_ENV)
        && let Err(message) = crate::parse_heap_limit(value.as_encoded_bytes())
    {
        eprintln!("rexx: {message}");
        return 2;
    }

    // `REXX_SWITCH_MODE` is the deterministic switch mode, for tests:
    // `every`, `at:K` for the program's K-th clause, or the simulation mode,
    // `sim`, `sim:SEED[,...]` or `sim:replay=FILE[,trace=FILE]`.
    invocation = match std::env::var("REXX_SWITCH_MODE").as_deref() {
        Ok("every") => invocation.with_switch_mode(crate::SwitchMode::EveryOpportunity),
        Ok(mode) if mode == "sim" || mode.starts_with("sim:") => {
            match crate::SimConfig::parse(mode) {
                Ok(config) => {
                    if mode == "sim" {
                        eprintln!("rexx-sim: seed={}", config.seed);
                    }
                    invocation.with_switch_mode(crate::SwitchMode::Sim(config))
                }
                Err(error) => {
                    eprintln!("rexx: REXX_SWITCH_MODE: {error}");
                    return 2;
                }
            }
        }
        Ok(mode) => match mode.strip_prefix("at:").map(str::parse) {
            Some(Ok(clause)) => invocation.with_switch_mode(crate::SwitchMode::AtClause(clause)),
            _ => {
                eprintln!(
                    "rexx: REXX_SWITCH_MODE is `every`, `at:K` or `sim[:SEED,...]`, not `{mode}`"
                );
                return 2;
            }
        },
        Err(_) => invocation,
    };

    let (outcome, instore) = match run.program {
        Program::Instore(text) => (
            crate::run_program_named(b"INSTORE", text, invocation.without_syscargs()),
            true,
        ),
        Program::File(name) => {
            invocation = invocation.with_syscargs(run.syscargs, run.syscargs_base);
            let options = run.default_options.is_some();
            if let Some((directive, count)) = run.default_options {
                invocation = invocation.with_default_options(directive, count);
            }
            match super::load(&name) {
                Ok((found, text)) => (
                    crate::run_program_named(
                        found.as_os_str().as_encoded_bytes(),
                        text,
                        invocation,
                    ),
                    false,
                ),
                // The oracle sends `-o`'s options before it looks for the
                // program, so their failure is the one reported. An empty
                // program runs nothing but that send.
                Err(failure) if options => {
                    let outcome = crate::run_program_named(b"", Vec::new(), invocation);
                    if outcome.exit_code == 0 {
                        return report(failure);
                    }
                    (outcome, false)
                }
                Err(failure) => return report(failure),
            }
        }
    };

    // What the sinks did not already take, after what native code left in
    // C's streams.
    crate::flush_c_streams();
    let _ = std::io::stdout().write_all(&outcome.stdout);
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().write_all(&outcome.stderr);
    if let Some(sim) = outcome
        .sim
        .filter(|_| std::env::var_os(crate::SIM_CHILD_ENV).is_none())
    {
        let trace = sim
            .trace_hash
            .map(|hash| format!(" trace={hash:016x}"))
            .unwrap_or_default();
        let hash = sim
            .hash_seed
            .map(|seed| format!(" hash={seed}"))
            .unwrap_or_default();
        let profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        eprintln!(
            "rexx-sim: seed={} policy={} steps={} contended={} switches={}{trace}{hash} profile={profile} \
             stack={}",
            sim.seed,
            sim.policy,
            sim.steps,
            sim.contended,
            sim.switches,
            crate::INTERPRETER_STACK_BYTES
        );
    }

    let status = if instore {
        super::instore_status(outcome.exit_code)
    } else {
        outcome.exit_code
    };
    // A process exit status is one byte.
    u8::cast_truncate(status.cast_unsigned())
}

/// Writes a report the launcher makes itself and exits with its status.
fn report((text, status): (Vec<u8>, i32)) -> u8 {
    let _ = std::io::stderr().write_all(&text);
    u8::cast_truncate(status.cast_unsigned())
}

/// Whether standard input, output and error are each a terminal or a pipe, from
/// each descriptor's own status, which every Unix answers. A closed one is not.
#[cfg(unix)]
fn transient_standard_streams() -> [bool; 3] {
    use std::os::fd::{AsFd, BorrowedFd};
    let transient = |fd: BorrowedFd<'_>| {
        rustix::fs::fstat(fd).is_ok_and(|status| {
            matches!(
                rustix::fs::FileType::from_raw_mode(status.st_mode),
                rustix::fs::FileType::CharacterDevice
                    | rustix::fs::FileType::Fifo
                    | rustix::fs::FileType::Socket
            )
        })
    };
    [
        transient(std::io::stdin().as_fd()),
        transient(std::io::stdout().as_fd()),
        transient(std::io::stderr().as_fd()),
    ]
}

#[cfg(windows)]
fn transient_standard_streams() -> [bool; 3] {
    super::transient_standard_streams()
}
