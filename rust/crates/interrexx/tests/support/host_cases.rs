//! Case files whose answers the host decides (`docs/design/decisions.md`): a file named
//! `<name>.<host>`, for a host Rust names, holds that host's answers and runs only there.
//! Shared by `#[path]`.

/// Whether `file` holds the answers of a host other than this one, for `walk_exclusive`.
pub fn for_another_host(file: &datadriven::TestFile) -> bool {
    let host = std::path::Path::new(&file.filename)
        .extension()
        .and_then(|extension| extension.to_str());
    host.is_some_and(|host| {
        ["linux", "macos", "windows"].contains(&host) && host != std::env::consts::OS
    })
}
