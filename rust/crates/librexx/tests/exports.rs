//! `librexx.so`'s SONAME, `NEEDED` entry, search path and exported names.

#[path = "../../librexxapi/tests/support/names.rs"]
mod names;

use names::{
    LIBREXX, LIBREXXAPI, NOT_EXPORTED, deps_dir, dynamic_entries, exported_rexx_functions,
    oracle_lib, set,
};

fn library() -> std::path::PathBuf {
    deps_dir().join("librexx.so")
}

#[test]
fn the_library_names_itself_and_needs_librexxapi() {
    assert_eq!(dynamic_entries(&library(), "SONAME"), ["librexx.so.4"]);
    assert!(
        dynamic_entries(&library(), "NEEDED").contains(&"librexxapi.so.4".to_owned()),
        "{:?}",
        dynamic_entries(&library(), "NEEDED")
    );
    assert_eq!(dynamic_entries(&library(), "RPATH"), ["$ORIGIN"]);
}

/// `librexxapi`'s names are carried too, so a process that loads both keeps one API state.
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
