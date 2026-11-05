//! The `rexx` executable exports the C API it compiles in, so an extension it loads binds to its
//! interpreter rather than to a second copy in `librexx.so.4`. The build date `PARSE VERSION`
//! carries is stamped here, from `SOURCE_DATE_EPOCH` when set; it is the date the build script
//! last ran.

#![expect(
    clippy::disallowed_methods,
    reason = "the build date reads the build host's clock"
)]

#[path = "build_date.rs"]
mod build_date;

fn main() {
    println!("cargo:rustc-link-arg-bin=rexx=-Wl,--export-dynamic-symbol=Rexx*");
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
}
