//! The `rexx` executable exports the C API, so an extension it loads binds to its interpreter
//! rather than to `librexx.so.4`'s.

#[path = "../../librexxapi/tests/support/names.rs"]
mod names;

use names::{LIBREXX, LIBREXXAPI, exported_rexx_functions, set};

#[test]
fn the_executable_exports_every_c_entry_point() {
    let exported = exported_rexx_functions(std::path::Path::new(env!("CARGO_BIN_EXE_rexx")));
    let mut expected = set(LIBREXX);
    expected.extend(set(LIBREXXAPI));
    assert_eq!(exported, expected);
}
