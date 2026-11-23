//! `rexxc`, the command-line compiler.

use std::io::Write;
use std::process::ExitCode;

use interrexx::compiled;
use interrexx::launcher;

fn main() -> ExitCode {
    interrexx::install_signal_handlers();
    let args: Vec<Vec<u8>> = std::env::args_os()
        .skip(1)
        .map(std::ffi::OsString::into_encoded_bytes)
        .collect();
    let compile = match launcher::parse_compile(&args) {
        Ok(compile) => compile,
        Err(usage) => {
            let _ = std::io::stdout().write_all(&usage);
            let _ = std::io::stdout().flush();
            return ExitCode::from(launcher::COMPILE_USAGE_STATUS);
        }
    };
    if !compile.silent {
        let mut banner = launcher::version_banner();
        banner.push(b'\n');
        let _ = std::io::stdout().write_all(&banner);
    }
    if compile.output.as_ref() == Some(&compile.input) {
        let _ = std::io::stdout().write_all(&launcher::same_file_message());
        let _ = std::io::stdout().flush();
        return ExitCode::from(launcher::COMPILE_SAME_FILE_STATUS);
    }
    let _ = std::io::stdout().flush();
    let (found, text) = match launcher::load(&compile.input) {
        Ok(loaded) => loaded,
        Err(failure) => return report(failure),
    };
    let image = compile.output.as_ref().map(|_| {
        compiled::write_image(&text, found.as_os_str().as_encoded_bytes(), compile.encode)
    });
    let outcome = launcher::translate(&found, text);
    let _ = std::io::stdout().write_all(&outcome.stdout);
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().write_all(&outcome.stderr);
    if outcome.exit_code != 0 {
        // A process exit status is one byte.
        return ExitCode::from(outcome.exit_code as u8);
    }
    if let (Some(output), Some(image)) = (&compile.output, image)
        && launcher::write_compiled_file(output, &image).is_err()
    {
        return report(launcher::output_unwritable(output));
    }
    ExitCode::SUCCESS
}

/// Writes a report the launcher makes itself and exits with its status.
fn report((text, status): (Vec<u8>, i32)) -> ExitCode {
    let _ = std::io::stderr().write_all(&text);
    ExitCode::from(status as u8)
}
