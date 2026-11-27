//! The ELF executable owns its C API. On macOS the launcher uses the shared library
//! for both the CLI and the C API.

#![cfg(not(windows))]

#[path = "../../interrexx-capi-rxapi/tests/support/names.rs"]
mod names;

use names::exported_rexx_functions;
#[cfg(not(target_os = "macos"))]
use names::{LIBREXX, LIBREXXAPI, set};

#[cfg(not(target_os = "macos"))]
#[test]
fn the_executable_exports_every_c_entry_point() {
    let exported = exported_rexx_functions(std::path::Path::new(env!("CARGO_BIN_EXE_rexx")));
    let mut expected = set(LIBREXX);
    expected.extend(set(LIBREXXAPI));
    assert_eq!(exported, expected);
}

#[cfg(target_os = "macos")]
#[test]
fn the_launcher_has_no_c_api_exports_of_its_own() {
    assert!(exported_rexx_functions(std::path::Path::new(env!("CARGO_BIN_EXE_rexx"))).is_empty());
}
