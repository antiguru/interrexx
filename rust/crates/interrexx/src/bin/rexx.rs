//! `rexx`, the command-line interpreter.

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut arguments = Vec::new();
    for argument in std::env::args_os().skip(1) {
        arguments.extend(argument.into_encoded_bytes());
        arguments.push(0);
    }
    let build = env!("INTERREXX_CLI_BUILD")
        .parse()
        .expect("the launcher build ID");
    match interrexx_api::load::run_cli(&arguments, build) {
        Ok(status) => ExitCode::from(status),
        Err(error) => {
            eprintln!("rexx: {error}");
            ExitCode::FAILURE
        }
    }
}
