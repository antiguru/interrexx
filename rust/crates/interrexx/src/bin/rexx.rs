//! `rexx`, the command-line interpreter.

use std::io::{IsTerminal, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::process::ExitCode;

use interrexx::launcher::{self, Command, Program};

#[path = "../../../interrexx-capi/src/rexxapi_exports.rs"]
mod rexxapi_exports;

#[path = "../../../interrexx-capi/src/exports.rs"]
mod rexx_exports;

fn main() -> ExitCode {
    interrexx::install_signal_handlers();
    let args: Vec<Vec<u8>> = std::env::args_os()
        .skip(1)
        .map(OsStringExt::into_vec)
        .collect();
    let run = match launcher::parse(&args) {
        Command::Usage => {
            let _ = std::io::stderr().write_all(launcher::USAGE);
            return ExitCode::from(launcher::USAGE_STATUS);
        }
        Command::Version => {
            let mut banner = launcher::version_banner();
            banner.push(b'\n');
            let _ = std::io::stdout().write_all(&banner);
            let _ = std::io::stdout().flush();
            return ExitCode::SUCCESS;
        }
        Command::Run(run) => run,
    };

    let mut invocation = interrexx::join_command_line(&run.words)
        .with_input(interrexx::ProgramInput::Stdin)
        .with_standard_transient(std::array::from_fn(|fd| {
            use std::os::unix::fs::FileTypeExt;
            std::fs::metadata(format!("/proc/self/fd/{fd}"))
                .is_ok_and(|meta| meta.file_type().is_char_device() || meta.file_type().is_fifo())
        }))
        .with_sinks(interrexx::Sinks {
            stdout: Box::new(|bytes| {
                let mut stdout = std::io::stdout();
                stdout.write_all(bytes).and_then(|()| stdout.flush())
            }),
            stderr: Box::new(|bytes| std::io::stderr().write_all(bytes)),
            stdout_terminal: std::io::stdout().is_terminal(),
            own_descriptors: true,
        });

    // `REXX_SWITCH_MODE` is the deterministic switch mode, for tests:
    // `every`, `at:K` for the program's K-th clause, or the simulation mode,
    // `sim`, `sim:SEED[,...]` or `sim:replay=FILE[,trace=FILE]`.
    invocation = match std::env::var("REXX_SWITCH_MODE").as_deref() {
        Ok("every") => invocation.with_switch_mode(interrexx::SwitchMode::EveryOpportunity),
        Ok(mode) if mode == "sim" || mode.starts_with("sim:") => {
            match interrexx::SimConfig::parse(mode) {
                Ok(config) => {
                    if mode == "sim" {
                        eprintln!("rexx-sim: seed={}", config.seed);
                    }
                    invocation.with_switch_mode(interrexx::SwitchMode::Sim(config))
                }
                Err(error) => {
                    eprintln!("rexx: REXX_SWITCH_MODE: {error}");
                    return ExitCode::from(2);
                }
            }
        }
        Ok(mode) => match mode.strip_prefix("at:").map(str::parse) {
            Some(Ok(clause)) => {
                invocation.with_switch_mode(interrexx::SwitchMode::AtClause(clause))
            }
            _ => {
                eprintln!(
                    "rexx: REXX_SWITCH_MODE is `every`, `at:K` or `sim[:SEED,...]`, not `{mode}`"
                );
                return ExitCode::from(2);
            }
        },
        Err(_) => invocation,
    };

    let (outcome, instore) = match run.program {
        Program::Instore(text) => (
            interrexx::run_program_named(b"INSTORE", text, invocation.without_syscargs()),
            true,
        ),
        Program::File(name) => {
            invocation = invocation.with_syscargs(run.syscargs, run.syscargs_base);
            let options = run.default_options.is_some();
            if let Some((directive, count)) = run.default_options {
                invocation = invocation.with_default_options(directive, count);
            }
            match launcher::load(&name) {
                Ok((found, text)) => (
                    interrexx::run_program_named(found.as_os_str().as_bytes(), text, invocation),
                    false,
                ),
                // The oracle sends `-o`'s options before it looks for the
                // program, so their failure is the one reported. An empty
                // program runs nothing but that send.
                Err(failure) if options => {
                    let outcome = interrexx::run_program_named(b"", Vec::new(), invocation);
                    if outcome.exit_code == 0 {
                        return report(failure);
                    }
                    (outcome, false)
                }
                Err(failure) => return report(failure),
            }
        }
    };

    // What the sinks did not already take.
    let _ = std::io::stdout().write_all(&outcome.stdout);
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().write_all(&outcome.stderr);
    if let Some(sim) = outcome
        .sim
        .filter(|_| std::env::var_os(interrexx::SIM_CHILD_ENV).is_none())
    {
        let trace = sim
            .trace_hash
            .map(|hash| format!(" trace={hash:016x}"))
            .unwrap_or_default();
        let profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        eprintln!(
            "rexx-sim: seed={} policy={} steps={} contended={} switches={}{trace} profile={profile} \
             stack={}",
            sim.seed,
            sim.policy,
            sim.steps,
            sim.contended,
            sim.switches,
            interrexx::INTERPRETER_STACK_BYTES
        );
    }

    let status = if instore {
        launcher::instore_status(outcome.exit_code)
    } else {
        outcome.exit_code
    };
    // A process exit status is one byte.
    ExitCode::from(status as u8)
}

/// Writes a report the launcher makes itself and exits with its status.
fn report((text, status): (Vec<u8>, i32)) -> ExitCode {
    let _ = std::io::stderr().write_all(&text);
    ExitCode::from(status as u8)
}
