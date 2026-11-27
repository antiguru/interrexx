//! The CLI launcher uses the shared library for both the CLI and the C API.

#![cfg(unix)]

#[path = "../../interrexx-capi-rxapi/tests/support/names.rs"]
mod names;

#[test]
fn the_launcher_has_no_c_api_exports_of_its_own() {
    let launcher = std::path::Path::new(env!("CARGO_BIN_EXE_rexx"));
    assert!(names::exported_rexx_functions(launcher).is_empty());
}
