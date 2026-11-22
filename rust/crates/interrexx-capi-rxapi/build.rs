//! The name extensions built against the oracle's headers record for this library, and the
//! dependency on `librexx` it records itself, found beside it: on ELF hosts a SONAME, a `NEEDED`
//! entry and a `DT_RPATH` of the library's own directory; on Apple's an install name and a load
//! command for `@loader_path`. On Windows the forwarding code loads the sibling DLL.

use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(dead_code, reason = "the build script reads only some of the names")]
#[path = "../interrexx-api/src/library_names.rs"]
mod library_names;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("cargo sets CARGO_CFG_TARGET_OS");
    let names = library_names::api_libraries(&target_os);
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    // The names the forwarding reads, as constants: the library links no code of another crate.
    println!("cargo:rustc-env=REXX_LIBREXX={}", names.librexx);
    println!("cargo:rustc-env=REXX_LIBREXXAPI={}", names.librexxapi);
    println!(
        "cargo:rustc-env=REXX_LIBREXX_BESIDE={}",
        names.librexx_beside
    );
    if std::env::var("CARGO_CFG_TARGET_VENDOR").as_deref() == Ok("apple") {
        apple(&names, &out);
    } else if target_os != "windows" {
        elf(&names, &out);
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../interrexx-api/src/library_names.rs");
}

fn elf(names: &library_names::ApiLibraries, out: &Path) {
    println!(
        "cargo:rustc-cdylib-link-arg=-Wl,-soname,{}",
        names.librexxapi_id
    );
    println!("cargo:rustc-cdylib-link-arg=-Wl,--disable-new-dtags");
    println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,$ORIGIN");
    // An empty library carrying `librexx`'s SONAME, which the link records as a `NEEDED` entry.
    // The forwarding exports look their targets up at run time, so no symbol binds to it and
    // `--as-needed` would drop the entry.
    let stub = out.join("librexx.so");
    let soname = format!("-Wl,-soname,{}", names.librexx_id);
    stub_library(&stub, &["-shared", "-nostdlib", &soname]);
    println!("cargo:rustc-cdylib-link-arg=-Wl,--no-as-needed");
    println!("cargo:rustc-cdylib-link-arg={}", stub.display());
    println!("cargo:rustc-cdylib-link-arg=-Wl,--as-needed");
}

fn apple(names: &library_names::ApiLibraries, out: &Path) {
    println!(
        "cargo:rustc-cdylib-link-arg=-Wl,-install_name,{}",
        names.librexxapi_id
    );
    // An empty library whose install name is `librexx` beside this one, which the link records
    // as a load command. `-needed_library` keeps it although no symbol binds to it. Apple's
    // linker requires every dylib to link `libSystem`, so it keeps the C library.
    let stub = out.join("librexx.dylib");
    stub_library(
        &stub,
        &["-dynamiclib", "-install_name", names.librexx_beside],
    );
    println!(
        "cargo:rustc-cdylib-link-arg=-Wl,-needed_library,{}",
        stub.display()
    );
}

/// Links an empty shared library at `stub` with the driver's `flags`.
fn stub_library(stub: &Path, flags: &[&str]) {
    let compiler = std::env::var_os("CC").unwrap_or_else(|| "cc".into());
    let status = Command::new(&compiler)
        .args(flags)
        .arg("-o")
        .arg(stub)
        .args(["-x", "c", "/dev/null"])
        .status()
        .expect("a C compiler driver to link the stub library");
    assert!(status.success(), "linking {} failed", stub.display());
}
