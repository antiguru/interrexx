//! Unix test binaries that embed the interpreter export the C API. The build date `PARSE VERSION`
//! carries is stamped here, from `SOURCE_DATE_EPOCH` when set; it is the date the build script
//! last ran.

#![expect(
    clippy::disallowed_methods,
    reason = "the build date reads the build host's clock"
)]

#[path = "build_date.rs"]
mod build_date;

mod cli_build;
use std::path::Path;

fn main() {
    // In-process tests that embed the interpreter export the C API.
    if std::env::var("CARGO_CFG_TARGET_FAMILY").as_deref() == Ok("unix") {
        // Apple's linker spells the export with the C symbol prefix, and an exported symbol is also
        // kept from dead stripping.
        let export = if std::env::var("CARGO_CFG_TARGET_VENDOR").as_deref() == Ok("apple") {
            "-Wl,-exported_symbol,_Rexx*"
        } else {
            "-Wl,--export-dynamic-symbol=Rexx*"
        };
        println!("cargo:rustc-link-arg-tests={export}");
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build_date.rs");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    let seconds = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(text) => text
            .trim()
            .parse::<i64>()
            .expect("SOURCE_DATE_EPOCH is a count of seconds"),
        Err(std::env::VarError::NotUnicode(_)) => panic!("SOURCE_DATE_EPOCH is not UTF-8"),
        Err(std::env::VarError::NotPresent) => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("the clock is after the epoch");
            i64::try_from(now.as_secs()).expect("seconds fit an i64")
        }
    };
    println!(
        "cargo:rustc-env=INTERREXX_BUILD_DATE={}",
        build_date::format(seconds)
    );

    // The launcher and C API interpreter share this build-script output.
    let out = std::env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR");
    let out = Path::new(&out);
    let unit = out
        .parent()
        .and_then(Path::file_name)
        .expect("Cargo names the build unit");
    let crate_dir = std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo names the crate");
    let id = cli_build::fingerprint(Path::new(&crate_dir), unit, seconds);
    std::fs::write(out.join("cli-build"), id.to_le_bytes()).expect("write the CLI build ID");
    println!("cargo:rustc-env=INTERREXX_CLI_BUILD={id}");
}
