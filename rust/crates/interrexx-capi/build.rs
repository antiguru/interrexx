//! The name extensions record for this library: the SONAME on ELF hosts, the install name on
//! Apple's. `librexx` needs no other library of the API: it owns the API's state and exports
//! `librexxapi`'s names itself.

#[allow(dead_code, reason = "the build script reads only some of the names")]
#[path = "../interrexx-api/src/library_names.rs"]
mod library_names;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("cargo sets CARGO_CFG_TARGET_OS");
    let names = library_names::api_libraries(&target_os);
    if std::env::var("CARGO_CFG_TARGET_VENDOR").as_deref() == Ok("apple") {
        println!(
            "cargo:rustc-cdylib-link-arg=-Wl,-install_name,@rpath/{}",
            names.librexx
        );
    } else {
        println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,{}", names.librexx);
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../interrexx-api/src/library_names.rs");
}
