//! Fingerprint the sources that go into the interpreter shared by the CLI and the C API.

use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

pub fn fingerprint(crate_dir: &Path, unit: &std::ffi::OsStr, epoch: i64) -> u64 {
    let crates = crate_dir.parent().expect("the crate is inside crates");
    let workspace = crates.parent().expect("crates is inside the workspace");
    let mut hash = DefaultHasher::new();
    (unit, epoch).hash(&mut hash);
    for name in [
        "interrexx",
        "interrexx-api",
        "interrexx-capi",
        "interrexx-cast",
        "interrexx-classes",
        "interrexx-core",
        "interrexx-inventory",
        "interrexx-lib",
        "interrexx-num",
        "interrexx-parse",
        "interrexx-rxapi",
        "interrexx-upstream",
    ] {
        for part in [
            "Cargo.toml",
            "build.rs",
            "build_date.rs",
            "cli_build.rs",
            "src",
            "data",
            "messages",
            "packages",
            "rexx",
        ] {
            track(workspace, &crates.join(name).join(part), &mut hash);
        }
    }
    for part in ["Cargo.toml", "Cargo.lock", ".cargo/config.toml"] {
        track(workspace, &workspace.join(part), &mut hash);
    }
    hash.finish()
}

fn track(root: &Path, path: &Path, hash: &mut DefaultHasher) {
    // A packaged crate need not have all the workspace siblings.
    if path.exists() {
        println!("cargo:rerun-if-changed={}", path.display());
        hash_path(root, path, hash);
    }
}

fn hash_path(root: &Path, path: &Path, hash: &mut DefaultHasher) {
    path.strip_prefix(root)
        .expect("a workspace input")
        .hash(hash);
    if path.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path)
            .expect("read the source directory")
            .map(|entry| entry.expect("read a source entry").path())
            .collect();
        entries.sort();
        for entry in entries {
            hash_path(root, &entry, hash);
        }
    } else {
        fs::read(path).expect("read a build input").hash(hash);
    }
}
