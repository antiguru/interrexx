//! The loader, run against the checkout-local compiled extension.

use interrexx_api::layout::{RexxMethodEntry, RexxPackageEntry, RexxRoutineEntry};
use interrexx_api::load::{self, CURRENT_INTERPRETER_VERSION, Failure};
use std::ffi::c_int;
use std::path::{Path, PathBuf};

/// The repository root, which is three directories above this crate.
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root is three directories above this crate")
}

/// The checkout-local `rxregexp` extension, kept separate from the oracle
/// build: it is loaded, never rebuilt by this test.
fn rxregexp() -> PathBuf {
    repository_root().join("build/lib/librxregexp.so")
}

fn open_rxregexp() -> load::Library {
    load::open_path(&rxregexp(), "rxregexp")
        .expect("the extension asks for 4.0.0, which is below this interpreter")
        .expect("checkout-local extension missing or invalid: run python3 rust/scripts/test_preflight.py")
}

/// The style every row of `rxregexp_methods` carries, `METHOD_TYPED_STYLE`.
const METHOD_TYPED_STYLE: c_int = 1;

#[test]
fn the_oracles_extension_loads_and_names_itself() {
    let library = open_rxregexp();
    assert_eq!(library.name(), Some(b"rxregexp".as_slice()));
    assert_eq!(library.version(), Some(b"4.0".as_slice()));
}

#[test]
fn the_method_table_is_what_the_extension_declares() {
    let library = open_rxregexp();
    let table: Vec<(c_int, &[u8])> = library
        .methods()
        .iter()
        .map(|row| (row.style, row.name.as_slice()))
        .collect();
    assert_eq!(
        table,
        vec![
            (METHOD_TYPED_STYLE, b"RegExp_Init".as_slice()),
            (METHOD_TYPED_STYLE, b"RegExp_Uninit".as_slice()),
            (METHOD_TYPED_STYLE, b"RegExp_Parse".as_slice()),
            (METHOD_TYPED_STYLE, b"RegExp_Pos".as_slice()),
            (METHOD_TYPED_STYLE, b"RegExp_Match".as_slice()),
        ],
        "the walk answered a different table than `extensions/rxregexp/\
         rxregexp.cpp:207-215` declares"
    );
    for row in library.methods() {
        assert!(
            row.has_entry_point(),
            "{:?} carries no entry point",
            String::from_utf8_lossy(&row.name)
        );
    }
}

#[test]
fn the_extension_declares_no_routines() {
    // The extension passes NULL for the routine table, and a null table
    // is read as no rows rather than as a failure.
    assert_eq!(open_rxregexp().routines(), &[]);
}

#[test]
fn a_method_is_found_without_regard_to_case() {
    let library = open_rxregexp();
    let found = library
        .method(b"regexp_parse")
        .expect("the table carries RegExp_Parse");
    assert_eq!(found.name, b"RegExp_Parse".to_vec());
    assert_eq!(found.style, METHOD_TYPED_STYLE);
    assert!(found.has_entry_point());
    assert_eq!(library.method(b"RegExp_Zork"), None);
}

#[test]
fn a_library_that_is_nowhere_is_not_an_error() {
    let answer = load::open("zorkolib", &[], std::path::Path::new("."))
        .expect("a library that is not there is not an error");
    assert!(answer.is_none());
}

/// A name resolves through a directory the caller supplies, which is the
/// channel a `LD_LIBRARY_PATH` the process did not start with reaches the
/// loader by. The empty search is the control: the same name answers nothing.
#[test]
fn a_search_directory_resolves_a_name_the_undecorated_attempt_misses() {
    let directory = repository_root().join("build/lib");
    assert!(
        load::open("rxregexp", &[], std::path::Path::new("."))
            .expect("a library that is not there is not an error")
            .is_none(),
        "librxregexp.so is already on the loader's own path, so the search \
         directory below is not what this test would be reading"
    );
    let found = load::open("rxregexp", &[directory], std::path::Path::new("."))
        .expect("the extension asks for 4.0.0, which is below this interpreter")
        .expect("checkout-local extension missing or invalid: run python3 rust/scripts/test_preflight.py");
    assert_eq!(found.name(), Some(b"rxregexp".as_slice()));
}

#[test]
fn a_library_without_the_exporter_is_not_an_error() {
    // Without the first assertion the second cannot tell "loaded, no
    // `RexxGetPackage`" from "did not load": both answer `Ok(None)`.
    let libc = Path::new("libc.so.6");
    assert!(load::loads(libc), "libc.so.6 did not load");
    let answer = load::open_path(libc, "c").expect("a library with no exporter is not an error");
    assert!(answer.is_none());
}

/// A package entry with no tables, no strings and no hooks, for the fields a
/// test does set to be the only thing under assertion.
fn synthetic_entry(required_version: c_int) -> RexxPackageEntry {
    RexxPackageEntry {
        size: 0,
        api_version: 0,
        required_version,
        package_name: std::ptr::null(),
        package_version: std::ptr::null(),
        loader: None,
        unloader: None,
        routines: std::ptr::null_mut::<RexxRoutineEntry>(),
        methods: std::ptr::null_mut::<RexxMethodEntry>(),
    }
}

#[test]
fn a_package_asking_for_a_newer_interpreter_is_refused() {
    let entry = synthetic_entry(CURRENT_INTERPRETER_VERSION + 1);
    assert_eq!(
        load::check_version(&entry, "zorkolib"),
        Err(Failure::LibraryVersion("zorkolib".to_owned()))
    );
    assert_eq!(
        Failure::LibraryVersion("zorkolib".to_owned()).code(),
        (98, 982)
    );
}

#[test]
fn a_package_asking_for_this_interpreter_or_less_is_accepted() {
    // Zero is "any version", and `rxregexp`'s own
    // `REXX_INTERPRETER_4_0_0` is the below-current case.
    for required in [0, 0x0004_0000, CURRENT_INTERPRETER_VERSION] {
        assert_eq!(
            load::check_version(&synthetic_entry(required), "zorkolib"),
            Ok(()),
            "requiredVersion {required:#010x} was refused"
        );
    }
}

/// A closed library stays mapped until the process exits, after its last handle is dropped too, so
/// code of its own still running in it does not fault (`r[native.threads.end]`,
/// `r[native.loading.static]`, #683).
#[test]
fn a_closed_library_stays_mapped() {
    let mapped = || {
        std::fs::read_to_string("/proc/self/maps")
            .expect("the process's maps")
            .contains("librxregexp.so")
    };
    let library = open_rxregexp();
    assert!(mapped());
    assert!(library.close());
    assert!(!library.is_open());
    assert!(mapped());
    drop(library);
    assert!(mapped());
}

/// A shared object named `lib<name>.so` in a directory of its own, built from the C `source`, and
/// whether the process has it mapped.
fn plain_library(name: &str, source: &str) -> (PathBuf, impl Fn() -> bool) {
    let dir = std::env::temp_dir().join(format!("interrexx-plain-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the directory is creatable");
    let file_source = dir.join("plain.c");
    std::fs::write(&file_source, source).expect("the source is writable");
    let file = format!("lib{name}.so");
    let built = std::process::Command::new("cc")
        .args(["-shared", "-fPIC", "-o"])
        .arg(dir.join(&file))
        .arg(&file_source)
        .status()
        .expect("cc runs, which this test needs to build its fixture");
    assert!(built.success(), "the fixture did not build");
    let mapped = move || {
        std::fs::read_to_string("/proc/self/maps")
            .expect("the process's maps")
            .contains(&file)
    };
    (dir, mapped)
}

/// A C file that defines one function and no package exporter.
const NO_EXPORTER: &str = "int plain(void) { return 1; }\n";

/// A library whose exporter answers no package entry stays mapped, as one whose exporter answers
/// one does (`r[native.loading.static]`).
#[test]
fn a_library_whose_exporter_answers_no_entry_stays_mapped() {
    let (dir, mapped) = plain_library(
        "nullpackage",
        "void *RexxGetPackage(void) { return (void *)0; }\n",
    );
    assert!(!mapped());
    let answer = load::open_path(&dir.join("libnullpackage.so"), "nullpackage")
        .expect("a null package entry is not an error");
    assert!(answer.is_none());
    assert!(mapped());
    std::fs::remove_dir_all(&dir).expect("the directory is removable");
}

/// A library that is no package stays mapped once the load has found it is none, as one that is
/// does: its file may be mapped for a classic registration's use too (`r[native.loading.static]`).
#[test]
fn a_library_without_the_exporter_stays_mapped() {
    let (dir, mapped) = plain_library("plainpackage", NO_EXPORTER);
    assert!(!mapped());
    let answer = load::open_path(&dir.join("libplainpackage.so"), "plainpackage")
        .expect("a library with no exporter is not an error");
    assert!(answer.is_none());
    assert!(mapped());
    std::fs::remove_dir_all(&dir).expect("the directory is removable");
}

/// A classic registration naming a procedure its library lacks leaves the library mapped, as a
/// package load does (`r[native.loading.static]`).
#[test]
fn a_classic_miss_leaves_its_library_mapped() {
    let (dir, mapped) = plain_library("plainclassic", NO_EXPORTER);
    assert!(!mapped());
    let found = load::classic_procedure(
        b"plainclassic",
        b"NOSUCHPROCEDURE",
        std::slice::from_ref(&dir),
        &dir,
    );
    assert!(found.is_none());
    assert!(mapped());
    std::fs::remove_dir_all(&dir).expect("the directory is removable");
}
