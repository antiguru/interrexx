//! Readings of a process that the tests take from the host: from `/proc` on Linux, and from
//! `getrusage`, `ps` and `lsof` on Apple's hosts. Shared by `#[path]`.

#![allow(dead_code, reason = "each test uses some of these")]

use std::path::Path;
#[cfg(not(target_os = "linux"))]
use std::process::Command;

/// This process's peak resident set, in KiB.
#[cfg(target_os = "linux")]
pub fn own_peak_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .expect("/proc/self/status")
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|rest| rest.trim().trim_end_matches("kB").trim().parse().ok())
        .expect("a VmHWM line")
}

/// This process's peak resident set, in KiB. Apple's `ru_maxrss` counts bytes.
#[cfg(not(target_os = "linux"))]
pub fn own_peak_kib() -> u64 {
    let usage = nix::sys::resource::getrusage(nix::sys::resource::UsageWho::RUSAGE_SELF)
        .expect("getrusage");
    let peak = u64::try_from(usage.max_rss()).expect("a non-negative peak");
    if cfg!(target_vendor = "apple") {
        peak / 1024
    } else {
        peak
    }
}

/// Process `pid`'s user and system time in hundredths of a second (Linux's clock ticks), or
/// `None` once it is gone.
#[cfg(target_os = "linux")]
pub fn cpu_centiseconds(pid: u32) -> Option<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let fields: Vec<&str> = stat.rsplit_once(')')?.1.split_whitespace().collect();
    let ticks = fields.get(11)?.parse::<u64>().ok()? + fields.get(12)?.parse::<u64>().ok()?;
    let hertz = rustix::param::clock_ticks_per_second().max(1);
    Some(ticks * 100 / hertz)
}

/// Process `pid`'s user and system time in hundredths of a second, as `ps` prints it
/// (`[hh:]mm:ss.cc`), or `None` once it is gone.
#[cfg(not(target_os = "linux"))]
pub fn cpu_centiseconds(pid: u32) -> Option<u64> {
    let output = Command::new("ps")
        .args(["-o", "time=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    let text = String::from_utf8(output.stdout).ok()?;
    let text = text.trim();
    let (clock, hundredths) = text.split_once('.')?;
    let seconds = clock.split(':').try_fold(0u64, |total, part| {
        Some(total * 60 + part.parse::<u64>().ok()?)
    })?;
    Some(seconds * 100 + hundredths.parse::<u64>().ok()?)
}

/// The number of threads of process `pid`.
#[cfg(target_os = "linux")]
pub fn thread_count(pid: u32) -> usize {
    std::fs::read_dir(format!("/proc/{pid}/task"))
        .expect("the process's threads")
        .count()
}

/// The number of threads of process `pid`: `ps -M` prints a heading and a line per thread.
#[cfg(not(target_os = "linux"))]
pub fn thread_count(pid: u32) -> usize {
    let output = Command::new("ps")
        .args(["-M", "-p", &pid.to_string()])
        .output()
        .expect("ps runs");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .count()
        .saturating_sub(1)
}

/// Whether a live process runs exactly `args` in the directory `cwd`.
#[cfg(target_os = "linux")]
pub fn runs_in(args: &[&str], cwd: &Path) -> bool {
    let wanted: Vec<u8> = args
        .iter()
        .flat_map(|arg| [arg.as_bytes(), b"\0"].concat())
        .collect();
    std::fs::read_dir("/proc")
        .expect("/proc")
        .flatten()
        .any(|entry| {
            std::fs::read(entry.path().join("cmdline")).is_ok_and(|cmdline| cmdline == wanted)
                && std::fs::read_link(entry.path().join("cwd")).is_ok_and(|dir| dir == cwd)
        })
}

/// Whether a live process runs exactly `args` in the directory `cwd`: the processes `ps` lists
/// with those arguments, and each one's working directory as `lsof` reports it.
#[cfg(not(target_os = "linux"))]
pub fn runs_in(args: &[&str], cwd: &Path) -> bool {
    let wanted = args.join(" ");
    let output = Command::new("ps")
        .args(["-axo", "pid=,args="])
        .output()
        .expect("ps runs");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let (pid, command) = line.trim_start().split_once(' ')?;
            (command.trim() == wanted).then(|| pid.to_owned())
        })
        .any(|pid| {
            Command::new("lsof")
                .args(["-a", "-d", "cwd", "-Fn", "-p", &pid])
                .output()
                .is_ok_and(|out| {
                    String::from_utf8_lossy(&out.stdout)
                        .lines()
                        .filter_map(|line| line.strip_prefix('n'))
                        .any(|dir| Path::new(dir) == cwd)
                })
        })
}
