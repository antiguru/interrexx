//! The `rexx` executable exports the C API it compiles in, so an extension it loads binds to its
//! interpreter rather than to a second copy in `librexx.so.4`.

fn main() {
    println!("cargo:rustc-link-arg-bin=rexx=-Wl,--export-dynamic-symbol=Rexx*");
}
