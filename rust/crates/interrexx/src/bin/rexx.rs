//! `rexx`, the command-line interpreter.

use std::process::ExitCode;

#[cfg(not(any(windows, target_os = "macos")))]
#[path = "../../../interrexx-capi/src/rexxapi_exports.rs"]
mod rexxapi_exports;

#[cfg(not(any(windows, target_os = "macos")))]
#[path = "../../../interrexx-capi/src/exports.rs"]
mod rexx_exports;

#[cfg(not(any(windows, target_os = "macos")))]
fn main() -> ExitCode {
    ExitCode::from(interrexx::launcher::run_cli())
}

#[cfg(any(windows, target_os = "macos"))]
fn main() -> ExitCode {
    match interrexx_api::load::run_cli() {
        Ok(status) => ExitCode::from(status),
        Err(error) => {
            eprintln!("rexx: {error}");
            ExitCode::FAILURE
        }
    }
}
