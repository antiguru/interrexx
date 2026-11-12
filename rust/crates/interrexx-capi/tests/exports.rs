//! `librexx.so`'s SONAME, `NEEDED` entry, search path and exported names.

#[path = "../../interrexx-capi-rxapi/tests/support/cdylibs.rs"]
mod cdylibs;
#[path = "../../interrexx-capi-rxapi/tests/support/names.rs"]
mod names;

use names::{
    LIBREXX, LIBREXXAPI, NOT_EXPORTED, dynamic_entries, exported_rexx_functions, oracle_lib, set,
};

fn library() -> std::path::PathBuf {
    cdylibs::librexx()
}

/// `librexx.so.4` owns the API's state and needs no library of
/// the API: `librexxapi.so.4` needs it, never the reverse.
#[test]
fn the_library_names_itself_and_needs_no_other_api_library() {
    assert_eq!(dynamic_entries(&library(), "SONAME"), ["librexx.so.4"]);
    let needed = dynamic_entries(&library(), "NEEDED");
    assert!(
        !needed.iter().any(|entry| entry.starts_with("librexx")),
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

#[test]
fn the_list_is_the_oracles_names() {
    let Some(lib) = oracle_lib() else {
        return;
    };
    assert_eq!(
        exported_rexx_functions(&lib.join("librexx.so.4")),
        set(LIBREXX)
    );
}
