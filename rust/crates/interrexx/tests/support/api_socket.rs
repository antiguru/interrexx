//! The API service directory of one run, kept short. The service's socket lives in it, and a Unix
//! socket path is bounded (`sun_path`), so a directory named for the checkout and the target
//! directory would overrun it under a long checkout path. The directory is under the system's
//! temporary directory and named for the process and a process-wide count, so that runs do not
//! share a service.

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};

/// The bytes a socket path may have, short of its terminator.
const SUN_PATH_BYTES: usize = 107;

/// The socket's file name under the service directory.
const SOCKET: &str = "rxapi.2.sock";

static MADE: AtomicUsize = AtomicUsize::new(0);

static ASSIGNED: Mutex<Option<HashMap<PathBuf, PathBuf>>> = Mutex::new(None);

/// The service directory of the run in `dir`: the same one for every call with the same `dir`, and
/// a different one for each `dir`. Asserts that the socket path in it fits `sun_path`.
pub fn api_dir(dir: &Path) -> PathBuf {
    let mut assigned = ASSIGNED.lock().unwrap_or_else(PoisonError::into_inner);
    let api = assigned
        .get_or_insert_with(HashMap::new)
        .entry(dir.to_path_buf())
        .or_insert_with(|| {
            let made = MADE.fetch_add(1, Ordering::Relaxed);
            std::env::temp_dir().join(format!("rxa-{:x}-{made:x}", std::process::id()))
        })
        .clone();
    let socket = api.join(SOCKET);
    let bytes = socket.as_os_str().len();
    assert!(
        bytes <= SUN_PATH_BYTES,
        "the API socket path {} is {bytes} bytes, over the {SUN_PATH_BYTES} a Unix socket path may have: \
         set TMPDIR to a shorter directory",
        socket.display()
    );
    api
}

/// Removes the service directory of the run in `dir`, once its service is stopped.
pub fn end_api_dir(dir: &Path) {
    let api = ASSIGNED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_mut()
        .and_then(|assigned| assigned.remove(dir));
    if let Some(api) = api {
        let _ = std::fs::remove_dir_all(api);
    }
}
