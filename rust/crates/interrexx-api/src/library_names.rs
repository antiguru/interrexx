//! The names the C API's two shared libraries go by on each host: the name a
//! program or library that links them records, which the loader looks for,
//! and how the one that forwards finds the one that owns the state. Build
//! scripts include this file by path, with the target's `CARGO_CFG_TARGET_OS`;
//! `librexxapi`'s build script hands the names to its source as constants, so
//! that library links no code of this crate. Code that runs on the host reads
//! [`host_api_libraries`].

/// The C API's libraries on one host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApiLibraries {
    /// The interpreter's library, which owns all of the API's state.
    pub librexx: &'static str,
    /// The classic API's library, which forwards each name to [`ApiLibraries::librexx`].
    pub librexxapi: &'static str,
    /// The name [`ApiLibraries::librexx`] records for itself, which a program linked against it
    /// records as its dependency: the SONAME on ELF hosts, the install name on Apple's.
    pub librexx_id: &'static str,
    /// The name [`ApiLibraries::librexxapi`] records for itself.
    pub librexxapi_id: &'static str,
    /// The path the forwarding library records for [`ApiLibraries::librexx`] and opens it by:
    /// beside itself, where the two are installed together.
    pub librexx_beside: &'static str,
}

/// The libraries on the host `target_os` names, as Rust spells it (`linux`, `macos`).
/// Hosts other than Apple's take the ELF names, which every other Unix shares.
#[must_use]
pub fn api_libraries(target_os: &str) -> ApiLibraries {
    match target_os {
        "macos" | "ios" => ApiLibraries {
            librexx: "librexx.4.dylib",
            librexxapi: "librexxapi.4.dylib",
            librexx_id: "@rpath/librexx.4.dylib",
            librexxapi_id: "@rpath/librexxapi.4.dylib",
            librexx_beside: "@loader_path/librexx.4.dylib",
        },
        _ => ApiLibraries {
            librexx: "librexx.so.4",
            librexxapi: "librexxapi.so.4",
            librexx_id: "librexx.so.4",
            librexxapi_id: "librexxapi.so.4",
            librexx_beside: "librexx.so.4",
        },
    }
}

/// The libraries on the host this was compiled for.
#[must_use]
pub fn host_api_libraries() -> ApiLibraries {
    api_libraries(std::env::consts::OS)
}
