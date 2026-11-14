//! The probe programs of `interrexx-upstream/data/probes`: one per code of
//! the error table, each raising its code on `rexx` and printing the code
//! and the ADDITIONAL values and nothing else, or a row in the list of codes
//! no program reaches. The writer of a message text works from what a probe
//! prints, so a probe that raises another code, or prints a message, is a
//! wrong brief.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;

use interrexx_inventory::errors::MESSAGES;

const REXX: &str = env!("CARGO_BIN_EXE_rexx");

fn probes() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../interrexx-upstream/data/probes")
}

/// The code `major.sub` of every row of the error table.
fn table() -> BTreeSet<String> {
    MESSAGES
        .iter()
        .map(|m| format!("{}.{}", m.major, m.sub))
        .collect()
}

/// The probe programs, as (code, path).
fn programs() -> Vec<(String, PathBuf)> {
    let mut found: Vec<(String, PathBuf)> = std::fs::read_dir(probes())
        .expect("the probe directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "rex"))
        .map(|path| {
            let code = path
                .file_stem()
                .expect("a file name")
                .to_string_lossy()
                .into_owned();
            (code, path)
        })
        .collect();
    found.sort();
    found
}

/// The codes the list names.
fn listed() -> Vec<String> {
    interrexx_upstream::brief::unreachable(&probes())
        .expect("the list reads")
        .into_iter()
        .map(|(code, _, _)| code)
        .collect()
}

#[test]
fn every_code_has_a_probe_or_a_reason() {
    let table = table();
    let probed: Vec<String> = programs().into_iter().map(|(code, _)| code).collect();
    let listed = listed();
    let stray: Vec<&String> = probed
        .iter()
        .chain(&listed)
        .filter(|code| !table.contains(*code))
        .collect();
    assert!(stray.is_empty(), "codes the table does not have: {stray:?}");
    let both: Vec<&String> = listed.iter().filter(|c| probed.contains(c)).collect();
    assert!(both.is_empty(), "codes with a probe and a reason: {both:?}");
    let mut twice = listed.clone();
    twice.sort();
    twice.dedup();
    assert_eq!(twice.len(), listed.len(), "a code is listed twice");
    let missing: Vec<&String> = table
        .iter()
        .filter(|code| !probed.contains(code) && !listed.contains(code))
        .collect();
    assert!(
        missing.is_empty(),
        "codes with neither a probe nor a reason: {missing:?}"
    );
}

/// What a probe may mention: the code and the ADDITIONAL values only, so no
/// message text can reach a brief through it.
#[test]
fn a_probe_does_not_ask_for_a_message() {
    let banned = [
        "errortext",
        "~message",
        "~description",
        "condition('d')",
        "condition(\"d\")",
        "sysgetmessage",
    ];
    // The probe of 40.903 calls ERRORTEXT with a bad argument, which is the
    // error it raises, and prints no text.
    let exempt = ["40.903"];
    let mut asking = Vec::new();
    for (code, path) in programs() {
        let text = std::fs::read_to_string(&path)
            .expect("a probe reads")
            .to_ascii_lowercase();
        if !exempt.contains(&code.as_str()) && banned.iter().any(|word| text.contains(word)) {
            asking.push(code);
        }
    }
    assert!(
        asking.is_empty(),
        "probes that ask for a message: {asking:?}"
    );
}

/// `rexx` running the probe in a scratch directory of its own with no input,
/// for at most 60 seconds, and what it wrote to standard output.
fn run(code: &str, program: &Path, api: &Path, bin: &Path) -> (std::process::ExitStatus, String) {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("message-probes")
        .join(code);
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("a scratch directory");
    let path = std::env::join_paths(std::iter::once(bin.to_path_buf()).chain(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
    ))
    .expect("a PATH");
    let output = Command::new(env!("CARGO_BIN_EXE_bounded-run"))
        .args(["-k", "5", "60"])
        .arg(REXX)
        .arg(program.canonicalize().expect("the probe exists"))
        .current_dir(&scratch)
        .env("REXX_API_DIR", api)
        .env("PATH", path)
        .env_remove("LD_LIBRARY_PATH")
        .env_remove("REXX_SWITCH_MODE")
        .stdin(Stdio::null())
        .output()
        .expect("rexx runs");
    let _ = std::fs::remove_dir_all(&scratch);
    (
        output.status,
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

/// Every probe ends with its own code: the first line it prints is the code,
/// and each later line is an ADDITIONAL value, its position and the value.
#[test]
fn every_probe_raises_its_own_code_and_prints_only_additional_values() {
    let bin = Path::new(REXX).parent().expect("rexx has a directory");
    let api = Path::new(env!("CARGO_TARGET_TMPDIR")).join("message-probes-api");
    let _ = std::fs::remove_dir_all(&api);
    std::fs::create_dir_all(&api).expect("the service directory");
    // The service refuses a directory that others can use, and says so.
    std::fs::set_permissions(&api, std::os::unix::fs::PermissionsExt::from_mode(0o700))
        .expect("a private service directory");
    let programs = programs();
    let wrong = Mutex::new(Vec::new());
    let next = Mutex::new(0usize);
    std::thread::scope(|scope| {
        for _ in 0..std::thread::available_parallelism().map_or(4, usize::from) {
            scope.spawn(|| {
                loop {
                    let at = {
                        let mut next = next.lock().expect("the counter");
                        *next += 1;
                        *next - 1
                    };
                    let Some((code, program)) = programs.get(at) else {
                        return;
                    };
                    let (status, stdout) = run(code, program, &api, bin);
                    let mut lines = stdout.lines();
                    let first = lines.next();
                    let additional = lines.all(|line| {
                        line.split_once(' ')
                            .is_some_and(|(index, _)| index.parse::<usize>().is_ok())
                    });
                    if !status.success() || first != Some(code.as_str()) || !additional {
                        wrong.lock().expect("the list").push(format!(
                            "{code}: {status}, printed {:?}",
                            stdout.chars().take(120).collect::<String>()
                        ));
                    }
                }
            });
        }
    });
    let _ = Command::new(bin.join("rxapi"))
        .arg("--stop")
        .env("REXX_API_DIR", &api)
        .output();
    let mut wrong = wrong.into_inner().expect("the list");
    wrong.sort();
    assert!(
        wrong.is_empty(),
        "probes that do not end with their own code:\n{}",
        wrong.join("\n")
    );
}
