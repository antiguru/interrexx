//! The SONAME extensions built against the oracle's headers name.

fn main() {
    println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,librexxapi.so.4");
}
