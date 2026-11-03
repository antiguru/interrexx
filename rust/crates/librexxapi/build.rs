//! The SONAME extensions built against the oracle's headers name, `librexx.so.4` as a `NEEDED`
//! entry, and a `DT_RPATH` of the library's own directory, where `librexx.so.4` is installed
//! beside it.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,librexxapi.so.4");
    println!("cargo:rustc-cdylib-link-arg=-Wl,--disable-new-dtags");
    println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,$ORIGIN");
    // An empty library carrying `librexx.so.4` as its SONAME, which the link records as a
    // `NEEDED` entry. The forwarding exports look their targets up at run time, so no symbol
    // binds to it and `--as-needed` would drop the entry.
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    let stub = out.join("librexx.so");
    let compiler = std::env::var_os("CC").unwrap_or_else(|| "cc".into());
    let status = Command::new(&compiler)
        .args(["-shared", "-nostdlib", "-Wl,-soname,librexx.so.4", "-o"])
        .arg(&stub)
        .args(["-x", "c", "/dev/null"])
        .status()
        .expect("a C compiler driver to link the NEEDED stub");
    assert!(status.success(), "linking {} failed", stub.display());
    println!("cargo:rustc-cdylib-link-arg=-Wl,--no-as-needed");
    println!("cargo:rustc-cdylib-link-arg={}", stub.display());
    println!("cargo:rustc-cdylib-link-arg=-Wl,--as-needed");
    println!("cargo:rerun-if-changed=build.rs");
}
