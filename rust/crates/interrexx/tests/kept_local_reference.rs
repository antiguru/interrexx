//! A local reference an extension keeps past the call that made it, through
//! the extension in `kept_local_reference/`, compiled with `c++` against this
//! worktree's `include/` into a directory of this run's own.

#![cfg(target_os = "linux")]
#![expect(
    clippy::disallowed_methods,
    reason = "this harness builds and runs a real extension"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use interrexx::{Invocation, Outcome, run_program, run_program_collect_every_alloc};

fn worktree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// Builds `kept_local_reference/kept.cpp` into `dir` as `libkept.so`.
fn build_library(dir: &Path) {
    let api = worktree().join("include");
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/kept_local_reference/kept.cpp");
    let built = Command::new("c++")
        .args(["-shared", "-fPIC", "-O1", "-Wall", "-Werror"])
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(&source)
        .arg("-o")
        .arg(dir.join("libkept.so"))
        .status()
        .expect("`c++` runs, which this test needs to build its extension");
    assert!(built.success(), "libkept.so did not build");
}

/// A fresh directory of this run's own, named for `name`.
fn fresh_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("a directory of this run's own");
    dir
}

/// Runs `body`, which may use `k`, an instance whose methods are `kept`'s.
fn run_in(dir: &Path, body: &str, stress: bool) -> (String, String, i32) {
    run_with(dir, body, stress, None)
}

/// [`run_in`], under the switch mode `mode` where there is one.
fn run_with(
    dir: &Path,
    body: &str,
    stress: bool,
    mode: Option<interrexx::SwitchMode>,
) -> (String, String, i32) {
    run_with_argument(dir, body, stress, mode, None)
}

/// [`run_with`], with `argument` as the program's argument where there is one.
fn run_with_argument(
    dir: &Path,
    body: &str,
    stress: bool,
    mode: Option<interrexx::SwitchMode>,
    argument: Option<&str>,
) -> (String, String, i32) {
    let program = format!(
        "k = .K~new\n{body}::class K\n\
         ::method make external 'LIBRARY kept KeepMake'\n\
         ::method anchored external 'LIBRARY kept KeepAnchored'\n\
         ::method length external 'LIBRARY kept KeepLength'\n\
         ::method values external 'LIBRARY kept KeepValues'\n\
         ::method loose external 'LIBRARY kept KeepLoose'\n\
         ::method commands external 'LIBRARY kept KeepCommands'\n\
         ::method address external 'LIBRARY kept KeepAddress'\n\
         ::method wordnow external 'LIBRARY kept KeepWordNow'\n\
         ::method wordlater external 'LIBRARY kept KeepWordLater'\n\
         ::method lendmine external 'LIBRARY kept KeepLendMine'\n\
         ::method role external 'LIBRARY kept KeepRole'\n\
         ::method publish external 'LIBRARY kept KeepPublish'\n\
         ::method signal external 'LIBRARY kept KeepSignal'\n\
         ::method wait external 'LIBRARY kept KeepWait'\n\
         ::method use external 'LIBRARY kept KeepUse'\n"
    );
    let invocation = Invocation::none()
        .with_directory(dir.to_path_buf())
        .with_environment(vec![(
            b"LD_LIBRARY_PATH".to_vec(),
            dir.as_os_str().as_encoded_bytes().to_vec(),
        )])
        .with_deadline(Duration::from_secs(60));
    let invocation = match mode {
        Some(mode) => invocation.with_switch_mode(mode),
        None => invocation,
    };
    let invocation = match argument {
        Some(argument) => invocation.with_arguments(vec![Some(argument.as_bytes().to_vec())]),
        None => invocation,
    };
    let path = dir.join("kept.rex").to_string_lossy().into_owned();
    let text = program.into_bytes();
    let outcome: Outcome = match stress {
        true => run_program_collect_every_alloc(&path, text, invocation),
        false => run_program(&path, text, invocation),
    };
    (
        String::from_utf8_lossy(&outcome.stdout).into_owned(),
        String::from_utf8_lossy(&outcome.stderr).into_owned(),
        outcome.exit_code,
    )
}

/// **A kept handle names its object in a later call for as long as the
/// object lives,** as an answer and as the receiver of a message. Measured on
/// the oracle with the same program: this stdout, empty stderr, rc 0. Under a
/// collection at every allocation, so the answers do not depend on when the
/// collector runs.
#[test]
fn a_kept_handle_names_its_live_object_in_a_later_call() {
    let dir = fresh_dir("kept-live");
    build_library(&dir);
    let ours = run_in(
        &dir,
        "say k~make\nsay k~anchored\nsay k~length\nsay 'end'\n",
        true,
    );
    assert_eq!(
        ours,
        (
            "made\nanchored in an object variable\n30\nend\n".to_owned(),
            String::new(),
            0
        )
    );
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}

/// **A kept handle that carries its value names that value in a later
/// call:** here a 5-byte string and a small integer, as ooSQLite keeps its
/// NULL text and its last error code. Measured on the oracle with the same
/// program: this stdout, empty stderr, rc 0. On a 32-bit target a value whose
/// bits the address cannot carry, such as this string, has a numbered handle,
/// which its call gives back (native-api.md, Handles and lifetimes).
#[cfg(target_pointer_width = "64")]
#[test]
fn a_kept_handle_carrying_its_value_names_it_in_a_later_call() {
    let dir = fresh_dir("kept-values");
    build_library(&dir);
    let ours = run_in(&dir, "say k~make\nsay k~values\nsay 'end'\n", true);
    assert_eq!(ours, ("made\nshort 42\nend\n".to_owned(), String::new(), 0));
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}

/// **A kept handle whose object was collected is refused loudly** (Deviation
/// 95): nothing but the extension held the string, and a collection at every
/// allocation frees it before the later call. The API leaves this use
/// undefined, so the harness runs no oracle.
#[test]
fn a_kept_handle_whose_object_was_collected_is_refused() {
    let dir = fresh_dir("kept-collected");
    build_library(&dir);
    let ours = run_in(&dir, "say k~make\nsay k~loose\nsay 'end'\n", true);
    assert_eq!(
        ours,
        (
            "made\n".to_owned(),
            "interrexx: a handle naming no live object is not implemented\n".to_owned(),
            120
        )
    );
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}

/// **A command handler's kept handle names its string in a later command**,
/// as a native method's does: the command string it was handed for `stash`
/// is still held by the program's variable when `use` sends it LENGTH.
/// Measured on the oracle with the same program: this stdout, empty stderr,
/// rc 0. Under a collection at every allocation.
#[test]
fn a_command_handlers_kept_handle_names_its_string_in_a_later_command() {
    let dir = fresh_dir("kept-command");
    build_library(&dir);
    let ours = run_in(
        &dir,
        "say k~commands\n\
         cmd = 'stash a command string long enough for the heap'\n\
         address KEPT cmd\n\
         say rc\n\
         address KEPT 'use'\n\
         say rc length(cmd)\n",
        true,
    );
    assert_eq!(
        ours,
        ("added\nstashed\n47 47\n".to_owned(), String::new(), 0)
    );
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}

/// Held by each test here that runs simulated interpreters: a simulated
/// interpreter's handle key depends on how many others run at once, which
/// `cargo test`'s threads would otherwise vary.
static SIMULATED: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// **Under the simulation mode a run's handles are its seed's**: the same
/// seed run twice in one process hands an extension the same handle, so a
/// replay that hashes, sorts or prints handles follows its recording.
#[test]
fn the_same_simulation_seed_hands_out_the_same_handles() {
    let _alone = SIMULATED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = fresh_dir("kept-sim");
    build_library(&dir);
    let sim = || {
        Some(interrexx::SwitchMode::Sim(
            interrexx::SimConfig::parse("sim:7").expect("a simulation mode"),
        ))
    };
    let first = run_with(&dir, "say k~address\n", false, sim());
    let second = run_with(&dir, "say k~address\n", false, sim());
    assert_eq!((first.1.as_str(), first.2), ("", 0));
    assert_eq!(first, second);
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}

/// **Two simulated interpreters running at once with the same seed share no
/// objects** (`decisions.md`, interpreter instances share no objects). Both
/// run the same program under `sim:7` on threads of their own, so up to the
/// string each lends their heaps are laid out alike, slot for slot. A
/// publishes its kept handle and waits; B's own kept handle names its string,
/// and A's names nothing in B. The replay half of the rule is
/// [`the_same_simulation_seed_hands_out_the_same_handles`].
// A 32-bit target's handles carry no instance key (native-api.md, Handles
// and lifetimes), so instance isolation of kept handles is a 64-bit property.
#[cfg(target_pointer_width = "64")]
#[test]
fn simulated_interpreters_running_at_once_share_no_objects() {
    let _alone = SIMULATED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = fresh_dir("kept-sim-at-once");
    build_library(&dir);
    let sim = || {
        Some(interrexx::SwitchMode::Sim(
            interrexx::SimConfig::parse("sim:7").expect("a simulation mode"),
        ))
    };
    // One program for both, so that their heaps match up to the lend: the
    // role, A for the first to ask, comes after it.
    let body = "say k~lendmine\n\
                if k~role = 'A' then do\n\
                  say k~publish\n\
                  say k~signal(1)\n\
                  say k~wait(2)\n\
                end\n\
                else do\n\
                  say k~wait(1)\n\
                  say 'B, its own:' k~use('mine')\n\
                  say \"B, A's:\" k~use('published')\n\
                  say k~signal(2)\n\
                end\n";
    let (first, second) = std::thread::scope(|scope| {
        let first = scope.spawn(|| run_with(&dir, body, false, sim()));
        let second = scope.spawn(|| run_with(&dir, body, false, sim()));
        (first.join().expect("a run"), second.join().expect("a run"))
    });
    let (a, b) = if first.0.contains("published") {
        (first, second)
    } else {
        (second, first)
    };
    assert_eq!(
        a,
        (
            "lent\npublished\nsignalled\n1\n".to_owned(),
            String::new(),
            0
        )
    );
    assert_eq!(
        b,
        (
            "lent\n1\nB, its own: 33\nB, A's: refused\nsignalled\n".to_owned(),
            String::new(),
            0
        )
    );
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}

/// **A replay hands out its recording's handles, whatever else ran beside
/// the recording.** Two runs under `sim:7` record traces while both are live,
/// so they hold live ordinals 0 and 1 and their keys differ; each then
/// replays alone, holding ordinal 0, and hands native code the same handle
/// its recording did. The recording that held ordinal 1 carries its key in
/// its trace header (`ordinal=1`); the other carries none, and its replay
/// takes ordinal 0's key.
// A 32-bit target's handles carry no instance key (native-api.md, Handles
// and lifetimes), so instance isolation of kept handles is a 64-bit property.
#[cfg(target_pointer_width = "64")]
#[test]
fn a_replay_hands_out_its_recordings_handles_whatever_ran_beside_it() {
    let _alone = SIMULATED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = fresh_dir("kept-sim-replay");
    build_library(&dir);
    let mode = |text: String| {
        Some(interrexx::SwitchMode::Sim(
            interrexx::SimConfig::parse(&text).expect("a simulation mode"),
        ))
    };
    // A signals and waits for B, so the two are live at once; in a replay
    // the flags are already set and neither waits.
    let body = "say k~address\n\
                parse arg role\n\
                if role = 'A' then do\n\
                  x = k~signal(3)\n\
                  x = k~wait(4)\n\
                end\n\
                else do\n\
                  x = k~wait(3)\n\
                  x = k~signal(4)\n\
                end\n";
    let trace = |role: &str| dir.join(format!("{role}.trace"));
    let recorded = std::thread::scope(|scope| {
        let runs = ["A", "B"].map(|role| {
            let (dir, trace) = (&dir, trace(role));
            scope.spawn(move || {
                run_with_argument(
                    dir,
                    body,
                    false,
                    mode(format!("sim:7,trace={}", trace.display())),
                    Some(role),
                )
            })
        });
        runs.map(|run| run.join().expect("a recording ran"))
    });
    for run in &recorded {
        assert_eq!((run.1.as_str(), run.2), ("", 0), "a recording: {run:?}");
    }
    assert_ne!(
        recorded[0].0, recorded[1].0,
        "the two recordings were not live at once, so their keys did not differ"
    );
    for (role, recording) in ["A", "B"].into_iter().zip(&recorded) {
        let replayed = run_with_argument(
            &dir,
            body,
            false,
            mode(format!("sim:replay={}", trace(role).display())),
            Some(role),
        );
        assert_eq!(replayed, *recording, "{role}'s replay");
    }
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}

/// **A replay that cannot have its recording's handle key is refused, and
/// runs once the key is free.** A trace is recorded under `sim:7` alone, so
/// with ordinal 0's key. While another `sim:7` run holds that key, a replay
/// of the trace is refused before it starts (`rexx` status 2) rather than
/// given another key; once that run has ended, the replay runs and hands out
/// the handle its recording did.
#[cfg(target_pointer_width = "64")]
#[test]
fn a_replay_whose_key_is_held_is_refused_until_it_is_free() {
    let _alone = SIMULATED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = fresh_dir("kept-sim-replay-held");
    build_library(&dir);
    let mode = |text: String| {
        Some(interrexx::SwitchMode::Sim(
            interrexx::SimConfig::parse(&text).expect("a simulation mode"),
        ))
    };
    let trace = dir.join("alone.trace");
    let recorded = run_with(
        &dir,
        "say k~address\n",
        false,
        mode(format!("sim:7,trace={}", trace.display())),
    );
    assert_eq!((recorded.1.as_str(), recorded.2), ("", 0), "{recorded:?}");
    let started = dir.join("holder-started");
    std::thread::scope(|scope| {
        // Holds ordinal 0's key under seed 7 until flag 6 is set.
        let holder = scope.spawn(|| {
            run_with(
                &dir,
                &format!(
                    "call lineout '{}', 'started'\ncall lineout '{}'\nsay k~wait(6)\n",
                    started.display(),
                    started.display()
                ),
                false,
                mode("sim:7".to_owned()),
            )
        });
        for _ in 0..2000 {
            if started.exists() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(started.exists(), "the holder never started");
        let refused = run_with(
            &dir,
            "say k~address\n",
            false,
            mode(format!("sim:replay={}", trace.display())),
        );
        assert_eq!((refused.0.as_str(), refused.2), ("", 2), "{refused:?}");
        assert!(
            refused
                .1
                .starts_with("rexx: REXX_SWITCH_MODE: the handle key of seed 7 and ordinal 0"),
            "{refused:?}"
        );
        // Set flag 6 from a run outside the simulation mode, which holds no key.
        assert_eq!(run_in(&dir, "say k~signal(6)\n", false).2, 0);
        let holder = holder.join().expect("the holder ran");
        assert_eq!((holder.1.as_str(), holder.2), ("", 0), "{holder:?}");
    });
    let replayed = run_with(
        &dir,
        "say k~address\n",
        false,
        mode(format!("sim:replay={}", trace.display())),
    );
    assert_eq!(replayed, recorded);
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}

/// **A kept handle to a value whose bits the address cannot carry: usable in
/// its call, and on a 32-bit target refused after it.** The 5-byte string
/// `short` is the same live value throughout. Within the call that made its
/// handle the handle names it on every target. In a later call it names it
/// on a 64-bit target, where every value is carried in the address, and is
/// refused on a 32-bit one, where the handle is a number its call gave back
/// (native-api.md, Handles and lifetimes).
#[test]
fn a_kept_numbered_value_handle_is_usable_in_its_call_and_refused_after_it_on_32_bit() {
    let dir = fresh_dir("kept-numbered-value");
    build_library(&dir);
    let ours = run_in(
        &dir,
        "say k~wordnow\nsay k~wordlater\nsay length('short')\n",
        true,
    );
    let later = if cfg!(target_pointer_width = "64") {
        "5"
    } else {
        "refused"
    };
    assert_eq!(ours, (format!("5\n{later}\n5\n"), String::new(), 0));
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}
