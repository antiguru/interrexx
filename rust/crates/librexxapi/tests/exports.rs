//! `librexxapi.so`'s SONAME and exported names.

#[path = "support/names.rs"]
mod names;

use names::{
    LIBREXX, LIBREXXAPI, NOT_EXPORTED, deps_dir, dynamic_entries, exported_rexx_functions,
    oracle_lib, set,
};

#[test]
fn the_library_names_itself_as_extensions_need_it() {
    let library = deps_dir().join("librexxapi.so");
    assert_eq!(dynamic_entries(&library, "SONAME"), ["librexxapi.so.4"]);
}

#[test]
fn the_library_exports_its_api_names_and_no_others() {
    let exported = exported_rexx_functions(&deps_dir().join("librexxapi.so"));
    assert_eq!(exported, set(LIBREXXAPI));
    for name in LIBREXX.iter().chain(NOT_EXPORTED) {
        assert!(!exported.contains(*name), "{name} is exported");
    }
}

/// The list is the oracle's `librexxapi.so.4` exports without its internal resolvers, read where
/// the oracle is built.
#[test]
fn the_list_is_the_oracles_public_names() {
    let Some(lib) = oracle_lib() else {
        return;
    };
    let mut theirs = exported_rexx_functions(&lib.join("librexxapi.so.4"));
    for name in NOT_EXPORTED {
        assert!(theirs.remove(*name), "the oracle no longer exports {name}");
    }
    assert_eq!(theirs, set(LIBREXXAPI));
}
