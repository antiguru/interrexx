use std::path::{Path, PathBuf};

/// Where `rexx` is built, release first.
pub(super) fn rust_binary_candidates() -> [PathBuf; 2] {
    rust_binary_candidates_for(
        std::env::var_os("CARGO_TARGET_DIR").as_deref(),
        &std::env::current_dir().expect("invocation cwd"),
        &std::env::current_exe().expect("benchmark executable"),
        Path::new(env!("CARGO_MANIFEST_DIR")),
    )
}

fn rust_binary_candidates_for(
    target_dir: Option<&std::ffi::OsStr>,
    cwd: &Path,
    exe: &Path,
    manifest: &Path,
) -> [PathBuf; 2] {
    let (profile, under_cargo) = cargo_profile_dir(exe);
    let executable_target = profile.and_then(Path::parent);
    let target = match target_dir {
        Some(target) if Path::new(target).is_absolute() => PathBuf::from(target),
        // Cargo runs tests from the package directory, losing the origin
        // against which it resolved a relative target directory.
        Some(_) if under_cargo && executable_target.is_some() => executable_target
            .expect("recognized test profile")
            .to_path_buf(),
        Some(target) => cwd.join(target),
        None => executable_target.map_or_else(|| manifest.join("../../target"), Path::to_path_buf),
    };
    [target.join("release/rexx"), target.join("debug/rexx")]
}

/// The `debug` or `release` directory an executable was built under, and whether
/// cargo placed it there as a test or intermediate artifact. Cargo puts a final
/// binary directly in the profile directory, a test binary in `deps/` (before
/// Rust 1.100), or under `build/<package>/<hash>/out/` (from Rust 1.100 on).
fn cargo_profile_dir(exe: &Path) -> (Option<&Path>, bool) {
    let is_profile = |path: &Path| {
        path.file_name()
            .is_some_and(|name| name == "release" || name == "debug")
    };
    let parent = exe.parent();
    if let Some(dir) = parent.filter(|path| is_profile(path)) {
        return (Some(dir), false);
    }
    for profile in exe.ancestors().skip(2).take(4) {
        if !is_profile(profile) {
            continue;
        }
        let below = exe
            .strip_prefix(profile)
            .ok()
            .and_then(|rest| rest.iter().next());
        if below.is_some_and(|first| first == "deps" || first == "build") {
            return (Some(profile), true);
        }
    }
    (None, false)
}

#[cfg(test)]
mod tests;
