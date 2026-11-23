//! Where cargo left this workspace's shared objects, `librexx` of `interrexx-capi` and
//! `librexxapi` of `interrexx-capi-rxapi` (`.so`, or `.dylib` on Apple's hosts).
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

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

/// The file cargo builds for the library `name`: `lib<name>.so`, or `lib<name>.dylib` on Apple's
/// hosts.
pub fn cargo_file(name: &str) -> String {
    format!(
        "{}{name}{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    )
}

/// `librexx`, built by the package named `interrexx-capi`.
pub fn librexx() -> PathBuf {
    locate("interrexx-capi", &cargo_file("rexx"))
}

/// `librexxapi`, built by the package named `interrexx-capi-rxapi`.
pub fn librexxapi() -> PathBuf {
    locate("interrexx-capi-rxapi", &cargo_file("rexxapi"))
}

/// A directory under `scratch` holding both libraries, which `scripts/build-*.sh --libs` read
/// from one place. Hard links where the filesystem allows, copies otherwise.
pub fn libs_dir(scratch: &Path) -> PathBuf {
    let dir = scratch.join("libs");
    fs::create_dir_all(&dir).expect("a directory for the libraries");
    for source in [librexx(), librexxapi()] {
        let name = source.file_name().expect("a library name");
        stage_library(&source, &dir.join(name)).expect("the library in place");
    }
    dir
}

pub(crate) fn stage_library(source: &Path, destination: &Path) -> io::Result<()> {
    static NEXT_STAGE: AtomicU64 = AtomicU64::new(0);
    let parent = destination
        .parent()
        .expect("the library has a staging directory");
    let stage = loop {
        let sequence = NEXT_STAGE.fetch_add(1, Ordering::Relaxed);
        let directory = parent.join(format!(".stage-{}-{sequence}", std::process::id()));
        match fs::create_dir(&directory) {
            Ok(()) => break directory,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    // Only this call can create a link at the copy destination. A shared
    // pathname there could make a copy truncate the source through a hard link.
    let staged = stage.join("library");
    let result = if fs::hard_link(source, &staged).is_ok() {
        fs::rename(&staged, destination)
    } else {
        fs::copy(source, &staged).and_then(|_| fs::rename(&staged, destination))
    };
    // Renaming one hard link over another to the same inode can leave both names.
    let cleanup = fs::remove_dir_all(&stage);
    result.and(cleanup)
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
