//! `librexxapi.so`'s SONAME and exported names.

#[path = "support/names.rs"]
mod names;

use names::{
    LIBREXX, LIBREXXAPI, NOT_EXPORTED, deps_dir, dynamic_entries, exported_rexx_functions,
    oracle_lib, set, symbol_table,
};

#[test]
fn the_library_names_itself_as_extensions_need_it() {
    let library = deps_dir().join("librexxapi.so");
    assert_eq!(dynamic_entries(&library, "SONAME"), ["librexxapi.so.4"]);
}

/// Every export forwards to `librexx.so.4`, which the library
/// needs and finds beside itself.
#[test]
fn the_library_needs_librexx_beside_it() {
    let library = deps_dir().join("librexxapi.so");
    assert!(
        dynamic_entries(&library, "NEEDED").contains(&"librexx.so.4".to_owned()),
        "{:?}",
        dynamic_entries(&library, "NEEDED")
    );
    assert_eq!(dynamic_entries(&library, "RPATH"), ["$ORIGIN"]);
}

/// The library holds no state of its own: no code of the interpreter or of its API client is
/// linked into it, so nothing here can register or queue apart from `librexx.so.4`.
#[test]
fn the_library_carries_no_interpreter_code() {
    let table = symbol_table(&deps_dir().join("librexxapi.so"));
    for crate_name in ["rexx_exec", "rexx_rxapi", "rexx_core"] {
        let found: Vec<&str> = table
            .lines()
            .filter(|line| line.contains(crate_name))
            .take(3)
            .collect();
        assert!(found.is_empty(), "{crate_name}: {found:#?}");
    }
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
