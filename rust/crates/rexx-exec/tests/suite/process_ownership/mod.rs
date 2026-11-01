use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

/// Whether a process runs `sleep 41.5`, which `Grandchild`'s test starts.
pub(super) fn grandchild_sleeps(run: &Path) -> bool {
    let run = run.canonicalize().expect("canonical fixture run directory");
    fs::read_dir("/proc")
        .expect("/proc")
        .filter_map(Result::ok)
        .any(|entry| {
            fs::read(entry.path().join("cmdline"))
                .is_ok_and(|cmdline| cmdline == b"sleep\x0041.5\x00")
                && fs::read_link(entry.path().join("cwd")).is_ok_and(|cwd| cwd == run)
        })
}

/// Whether `Grandchild`'s sleeper disappears within `bound`, which is shorter
/// than its sleep. SIGKILL returns before the target has exited, and the
/// sleeper's output is not a pipe the run waits on.
pub(super) fn grandchild_ends_within(run: &Path, bound: Duration) -> bool {
    let start = Instant::now();
    while grandchild_sleeps(run) {
        if start.elapsed() >= bound {
            return false;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    true
}

#[cfg(test)]
mod tests;
