// SPDX-License-Identifier: MIT OR Apache-2.0

//! Every export, linked against this workspace's `librexx` and `librexxapi` from a C and a C++
//! program compiled against `include/`.
//!
//! A program that takes the address of every function the headers declare links only where each
//! declaration names a symbol a library defines: in C++ a declaration without C linkage names a
//! mangled symbol, which neither library defines. A function the libraries export and the headers
//! do not declare (`Declared::Nowhere`) is declared by the program itself, from the same table,
//! and found by its symbol alone.

#[path = "../../interrexx-capi-rxapi/tests/support/cdylibs.rs"]
mod cdylibs;
#[path = "../../interrexx-capi-rxapi/tests/support/names.rs"]
mod names;

use interrexx_api::exports::{Declared, EXPORTS};
use interrexx_api::header::prototype;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A directory of this test's own under the target's scratch directory.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("link")
        .join(name);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    dir
}

/// The repository's `include/` directory.
fn include_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../include")
}

/// Both libraries in `dir` under their run-time names (`lib<name>.so.4`, or `lib<name>.4.dylib`
/// on Apple's hosts, which each records and `librexxapi` names as its dependency), with the
/// link-time names cargo gave them beside them.
fn stage_libraries(dir: &Path) {
    let names = interrexx_api::library_names::host_api_libraries();
    for (source, run_time) in [
        (cdylibs::librexx(), names.librexx),
        (cdylibs::librexxapi(), names.librexxapi),
    ] {
        let name = source
            .file_name()
            .and_then(|name| name.to_str())
            .expect("a library name")
            .to_string();
        let versioned = dir.join(run_time);
        let _ = std::fs::remove_file(&versioned);
        std::fs::copy(&source, &versioned)
            .unwrap_or_else(|e| panic!("{} to {}: {e}", source.display(), versioned.display()));
        let link = dir.join(&name);
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(run_time, &link)
            .unwrap_or_else(|e| panic!("{}: {e}", link.display()));
    }
}

/// The program: the headers, a declaration of each undeclared export, and a table holding the
/// address of every export, which `main` counts.
fn program() -> String {
    let mut out = String::from(
        "#include <stdio.h>\n#include <stdbool.h>\n#include \"rexx.h\"\n#include \"oorexxapi.h\"\n\n",
    );
    out.push_str("#ifdef __cplusplus\nextern \"C\" {\n#endif\n");
    for export in EXPORTS.iter().filter(|e| e.declared == Declared::Nowhere) {
        writeln!(out, "{}", prototype(export)).unwrap();
    }
    out.push_str("#ifdef __cplusplus\n}\n#endif\n\n");
    out.push_str("static void *const exported[] = {\n");
    for export in EXPORTS {
        writeln!(out, "    (void *)&{},", export.name).unwrap();
    }
    out.push_str("};\n\n");
    out.push_str(
        "int main(void) {\n    size_t found = 0;\n    for (size_t at = 0; at < sizeof exported / sizeof exported[0]; at++) {\n        if (exported[at] != NULL) found++;\n    }\n    printf(\"%zu\\n\", found);\n    return 0;\n}\n",
    );
    out
}

/// Compiles and links the program as `compiler`'s language and runs it, answering what it
/// printed.
fn link_and_run(compiler: &str, source_name: &str) -> String {
    let dir = scratch(compiler.replace('+', "x").as_str());
    let libs = dir.join("libs");
    std::fs::create_dir_all(&libs).unwrap_or_else(|e| panic!("{}: {e}", libs.display()));
    stage_libraries(&libs);
    let source = dir.join(source_name);
    std::fs::write(&source, program()).unwrap_or_else(|e| panic!("{}: {e}", source.display()));
    let binary = dir.join("linked");
    let include = include_dir();
    let built = Command::new(compiler)
        .arg("-Wall")
        .arg("-I")
        .arg(&include)
        .arg("-I")
        .arg(include.join("platform/unix"))
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .arg(format!("-L{}", libs.display()))
        .arg(format!("-Wl,-rpath,{}", libs.display()))
        .arg("-lrexx")
        .arg("-lrexxapi")
        .output()
        .unwrap_or_else(|e| panic!("{compiler}: {e}"));
    assert!(
        built.status.success(),
        "the program does not link as {compiler}:\n{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new(&binary)
        .output()
        .unwrap_or_else(|e| panic!("{}: {e}", binary.display()));
    assert!(
        run.status.success(),
        "{} failed: {}",
        binary.display(),
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the program prints text")
}

/// **Every export links as C**: each declared one through its declaration in `include/`.
#[test]
fn every_export_links_as_c() {
    assert_eq!(
        link_and_run("cc", "linked.c").trim(),
        EXPORTS.len().to_string()
    );
}

/// **Every export links as C++**, where a declaration without C linkage would name a mangled
/// symbol.
#[test]
fn every_export_links_as_cxx() {
    assert_eq!(
        link_and_run("c++", "linked.cc").trim(),
        EXPORTS.len().to_string()
    );
}

/// The table of exports names every function the two libraries export, and no other.
#[test]
fn the_table_is_every_exported_name() {
    let mut expected = names::set(names::LIBREXX);
    expected.extend(names::set(names::LIBREXXAPI));
    let table = EXPORTS.iter().map(|e| e.name.to_string()).collect();
    assert_eq!(expected, table);
}
