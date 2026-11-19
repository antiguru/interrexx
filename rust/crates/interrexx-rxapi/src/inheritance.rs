//! Which descriptors a child process inherits. On a host that cannot create a
//! descriptor close-on-exec in the same call, a child started by one thread
//! while another thread is between creating a descriptor and setting the flag
//! inherits it. Every descriptor of the process's own made that way is made
//! under [`creating`], which takes a shared lock, and every child is started
//! under [`spawning`], which takes it exclusively. The lock lives in this
//! crate, the lowest that both the API client and the interpreter build on, so
//! that both take the same one.

/// Serialises descriptor creation with the start of child processes, where
/// the host needs it.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
static INHERITANCE: std::sync::RwLock<()> = std::sync::RwLock::new(());

/// Runs `create`, which makes descriptors and sets them close-on-exec, so that
/// no child [`spawning`] starts meanwhile inherits them. Linux creates them
/// close-on-exec in the one call, and needs no lock.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn creating<T>(create: impl FnOnce() -> T) -> T {
    create()
}

/// Runs `create`, which makes descriptors and sets them close-on-exec, so that
/// no child [`spawning`] starts meanwhile inherits them.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub fn creating<T>(create: impl FnOnce() -> T) -> T {
    let _shared = INHERITANCE
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    create()
}

/// Runs `spawn`, which starts a child process, while no descriptor of the
/// process's own is half made. Hold it for the start only, not while the child
/// runs.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn spawning<T>(spawn: impl FnOnce() -> T) -> T {
    spawn()
}

/// Runs `spawn`, which starts a child process, while no descriptor of the
/// process's own is half made. Hold it for the start only, not while the child
/// runs.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub fn spawning<T>(spawn: impl FnOnce() -> T) -> T {
    let _exclusive = INHERITANCE
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    spawn()
}
