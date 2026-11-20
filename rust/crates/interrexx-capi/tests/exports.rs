//! `librexx`'s recorded name, dependencies and exported names.

#[path = "../../interrexx-capi-rxapi/tests/support/cdylibs.rs"]
mod cdylibs;
#[path = "../../interrexx-capi-rxapi/tests/support/names.rs"]
mod names;

use interrexx_api::library_names::host_api_libraries;
use names::{
    LIBREXX, LIBREXXAPI, NOT_EXPORTED, exported_rexx_functions, needed, oracle_lib, own_name, set,
};

fn library() -> std::path::PathBuf {
    cdylibs::librexx()
}

/// `librexx` owns the API's state and needs no library of the API: `librexxapi` needs it, never
/// the reverse.
#[test]
fn the_library_names_itself_and_needs_no_other_api_library() {
    assert_eq!(own_name(&library()), [host_api_libraries().librexx_id]);
    let needed = needed(&library());
    assert!(
        !needed.iter().any(|entry| entry.contains("librexx")),
        "{needed:?}"
    );
}

/// `librexxapi`'s names are exported here too, where their state is: a program linked against
/// this library alone registers and runs, and `librexxapi.so.4` forwards to these definitions.
#[test]
fn the_library_exports_both_libraries_names() {
    let exported = exported_rexx_functions(&library());
    let mut expected = set(LIBREXX);
    expected.extend(set(LIBREXXAPI));
    assert_eq!(exported, expected);
    for name in NOT_EXPORTED {
        assert!(!exported.contains(*name), "{name} is exported");
    }
}

/// Needs the oracle build: it fails loudly without one, and the `standalone` nextest profile
/// filters it out instead.
#[test]
fn the_list_is_the_oracles_names() {
    let lib = oracle_lib().expect("the oracle's lib directory is missing (REXX_ORACLE_ROOT)");
    assert_eq!(
        exported_rexx_functions(&lib.join("librexx.so.4")),
        set(LIBREXX)
    );
}
