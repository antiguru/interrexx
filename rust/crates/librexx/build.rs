//! The SONAME extensions name. `librexx.so.4` needs no other library of the API: it owns the
//! API's state and exports `librexxapi`'s names itself.

fn main() {
    println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,librexx.so.4");
    println!("cargo:rerun-if-changed=build.rs");
}
