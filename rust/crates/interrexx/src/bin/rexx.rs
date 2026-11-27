//! `rexx`, the command-line interpreter.

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut arguments = Vec::new();
    for argument in std::env::args_os().skip(1) {
        arguments.extend(argument.into_encoded_bytes());
        arguments.push(0);
    }
    match interrexx_api::load::run_cli(&arguments) {
        Ok(status) => ExitCode::from(status),
        Err(error) => {
            eprintln!("rexx: {error}");
            ExitCode::FAILURE
        }
    }
}
