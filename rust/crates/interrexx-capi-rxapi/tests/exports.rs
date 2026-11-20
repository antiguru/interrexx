//! `librexxapi`'s recorded name, dependency and exported names.

#[path = "support/cdylibs.rs"]
mod cdylibs;
#[path = "support/names.rs"]
mod names;

use interrexx_api::library_names::host_api_libraries;
use names::{
    LIBREXX, LIBREXXAPI, NOT_EXPORTED, exported_rexx_functions, needed, oracle_lib, own_name,
    search_path, set, symbol_table,
};

#[test]
fn the_library_names_itself_as_extensions_need_it() {
    let library = cdylibs::librexxapi();
    assert_eq!(own_name(&library), [host_api_libraries().librexxapi_id]);
}

/// Every export forwards to `librexx`, which the library needs and finds beside itself: by
/// SONAME and a `$ORIGIN` search path on ELF hosts, by an `@loader_path` load command on
/// Apple's.
#[test]
fn the_library_needs_librexx_beside_it() {
    let library = cdylibs::librexxapi();
    let names = host_api_libraries();
    let needed = needed(&library);
    assert!(
        needed.iter().any(|entry| entry == names.librexx_beside),
        "{needed:?}"
    );
    let expected_search: &[&str] = if cfg!(target_vendor = "apple") {
        &[]
    } else {
        &["$ORIGIN"]
    };
    assert_eq!(search_path(&library), expected_search);
}

/// The library holds no state of its own: no code of the interpreter or of its API client is
/// linked into it, so nothing here can register or queue apart from `librexx.so.4`.
#[test]
fn the_library_carries_no_interpreter_code() {
    let table = symbol_table(&cdylibs::librexxapi());
    for crate_name in ["interrexx", "interrexx_rxapi", "interrexx_core"] {
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
    let exported = exported_rexx_functions(&cdylibs::librexxapi());
    assert_eq!(exported, set(LIBREXXAPI));
    for name in LIBREXX.iter().chain(NOT_EXPORTED) {
        assert!(!exported.contains(*name), "{name} is exported");
    }
}

/// The list is the oracle's `librexxapi.so.4` exports without its internal resolvers, read where
/// the oracle is built. It fails loudly without the oracle build, and the `standalone` nextest
/// profile filters it out instead.
#[test]
fn the_list_is_the_oracles_public_names() {
    let lib = oracle_lib().expect("the oracle's lib directory is missing (REXX_ORACLE_ROOT)");
    let mut theirs = exported_rexx_functions(&lib.join("librexxapi.so.4"));
    for name in NOT_EXPORTED {
        assert!(theirs.remove(*name), "the oracle no longer exports {name}");
    }
    assert_eq!(theirs, set(LIBREXXAPI));
}
