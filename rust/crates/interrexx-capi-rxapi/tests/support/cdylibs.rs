//! Where cargo left this workspace's shared objects, `librexx.so` of `interrexx-capi` and
//! `librexxapi.so` of `interrexx-capi-rxapi`.
//!
//! They are cargo's own artifacts, not a build script's output, so no `OUT_DIR` or `DEP_*`
//! variable names them, and cargo has no compile-time variable for a library's path as it has
//! for a binary's (`CARGO_BIN_EXE_*`). Before cargo 1.100 every one sat in the test binary's own
//! directory, `target/<profile>/deps`. Since then the test binary is in
//! `build/<package>/<hash>/out` and each library is in the same kind of directory of its own
//! package, under another hash. A test finds a library from where it runs, so the lookup works
//! beside a build, in a nextest archive extracted anywhere, and under either layout: beside the
//! test binary first, and otherwise among its package's `out` directories, where the most
//! recently written copy is the one the run just built.

#![allow(dead_code, reason = "each test uses some of these")]

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// `librexx.so`, built by the package named `interrexx-capi`.
pub fn librexx() -> PathBuf {
    locate("interrexx-capi", "librexx.so")
}

/// `librexxapi.so`, built by the package named `interrexx-capi-rxapi`.
pub fn librexxapi() -> PathBuf {
    locate("interrexx-capi-rxapi", "librexxapi.so")
}

/// A directory under `scratch` holding both libraries, which `scripts/build-*.sh --libs` read
/// from one place. Hard links where the filesystem allows, copies otherwise.
pub fn libs_dir(scratch: &Path) -> PathBuf {
    let dir = scratch.join("libs");
    std::fs::create_dir_all(&dir).expect("a directory for the libraries");
    for source in [librexx(), librexxapi()] {
        let name = source.file_name().expect("a library name");
        // Tests run as processes of their own: link under a private name, then rename over.
        let mut staged = name.to_owned();
        staged.push(format!(".{}", std::process::id()));
        let staged = dir.join(staged);
        let _ = std::fs::remove_file(&staged);
        if std::fs::hard_link(&source, &staged).is_err() {
            std::fs::copy(&source, &staged).expect("a copy of the library");
        }
        std::fs::rename(&staged, dir.join(name)).expect("the library in place");
    }
    dir
}

fn locate(package: &str, library: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("the test binary's path");
    let beside = exe.parent().expect("the test binary has a directory");
    let old = beside.join(library);
    if old.is_file() {
        return old;
    }
    // `build/<package>/<hash>/out/<test binary>`: the packages' directories are `build`'s children.
    let build = beside
        .ancestors()
        .nth(3)
        .filter(|build| build.file_name().is_some_and(|name| name == "build"))
        .unwrap_or_else(|| panic!("{library} is not beside {exe:?}, and the layout is unknown"));
    let newest = std::fs::read_dir(build.join(package))
        .unwrap_or_else(|error| panic!("no build directory of {package}: {error}"))
        .filter_map(|entry| {
            let candidate = entry.ok()?.path().join("out").join(library);
            let modified = candidate.metadata().ok()?.modified().ok()?;
            Some((modified, candidate))
        })
        .max_by_key(|(modified, _): &(SystemTime, PathBuf)| *modified);
    newest
        .unwrap_or_else(|| {
            panic!(
                "{library} is not under {}; build the workspace",
                build.display()
            )
        })
        .1
}
