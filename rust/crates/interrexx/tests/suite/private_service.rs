use std::path::{Path, PathBuf};

/// This crate's `rxapi`, `rxqueue` and `rxsubcom`, which a workspace build puts beside `rexx`.
pub fn service_binaries() -> [PathBuf; 3] {
    let rexx = Path::new(env!("CARGO_BIN_EXE_rexx"));
    ["rxapi", "rxqueue", "rxsubcom"].map(|name| {
        let binary = rexx.with_file_name(name);
        assert!(
            binary.is_file(),
            "{} is missing; run `cargo build --workspace --bins` first",
            binary.display()
        );
        binary
    })
}
