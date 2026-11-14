//! `interrexx-upstream write` rewrites the checked-in interface tables from
//! the pinned ooRexx tree (`$REXX_UPSTREAM`). `interrexx-upstream check`
//! derives them in memory and exits non-zero, printing the difference, when a
//! checked-in table is not what the tree gives.
//!
//! The writer tools compare interrexx's own message texts with the
//! catalogue's, and read the catalogue only to do that. They run locally and
//! print codes and scores, never a catalogue text:
//!
//! * `similarity [--threshold D] [--run N] [--all] [--allow FILE]` lists the
//!   codes whose text is too close to the catalogue's. `FILE` excuses codes
//!   (`major.sub<TAB>reason` per line).
//! * `scan [--comments-only] [PATH]` lists the tracked files holding a
//!   catalogue text, and on stderr the codes it cannot look for.
//! * `brief MAJOR OUTDIR` writes the writer's briefs for one major outside
//!   the repository. Run it under a memory cap (`memcap 8G cargo run ...`):
//!   it runs the probe programs on `$REXX_BIN` (default
//!   `rust/target/debug/rexx`).

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use interrexx_upstream::{
    brief, builtins, classes, difference, errors, messages, scan, similarity,
};

/// One checked-in table: where it lives (relative to the `crates` directory)
/// and how to derive its text from the source tree.
struct Table {
    path: &'static str,
    derive: fn(upstream: &Path, sha: &str) -> String,
}

const TABLES: &[Table] = &[
    Table {
        path: "interrexx-classes/data/classes.tsv",
        derive: |upstream, sha| {
            classes::Table::parse(&read(upstream, "interpreter/memory/Setup.cpp")).to_tsv(sha)
        },
    },
    Table {
        path: "interrexx-inventory/data/errors.tsv",
        derive: |upstream, sha| {
            errors::Table::parse(&read(upstream, "interpreter/messages/rexxmsg.xml")).to_tsv(sha)
        },
    },
    Table {
        path: "interrexx-inventory/data/builtins.txt",
        derive: |upstream, sha| {
            let names = builtins::parse(&read(
                upstream,
                "interpreter/expression/BuiltinFunctions.cpp",
            ));
            builtins::to_text(&names, sha)
        },
    },
];

/// The text of `file` under the source tree.
fn read(upstream: &Path, file: &str) -> String {
    let source = upstream.join(file);
    std::fs::read_to_string(&source)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", source.display()))
}

const USAGE: &str =
    "usage: interrexx-upstream (write | check | similarity | scan | brief MAJOR OUTDIR)";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let write = match args.first().map(String::as_str) {
        Some("write") => true,
        Some("check") => false,
        Some("similarity") => return run_similarity(&args[1..]),
        Some("scan") => return run_scan(&args[1..]),
        Some("brief") => return run_brief(&args[1..]),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let upstream = upstream();
    let sha = revision(&upstream);
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut failed = false;
    for table in TABLES {
        let want = (table.derive)(&upstream, &sha);
        let path = crates.join(table.path);
        if write {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)
                    .unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
            }
            std::fs::write(&path, &want)
                .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
            println!("wrote {}", table.path);
            continue;
        }
        let have = std::fs::read_to_string(&path).unwrap_or_default();
        if let Some(report) = difference(&have, &want) {
            failed = true;
            eprintln!("{} differs from the pinned tree:\n{report}", table.path);
        }
    }
    if failed {
        eprintln!("run `cargo run -p interrexx-upstream -- write` and commit the result");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// The pinned source tree, named by `$REXX_UPSTREAM`.
fn upstream() -> PathBuf {
    PathBuf::from(
        std::env::var_os("REXX_UPSTREAM").expect("REXX_UPSTREAM names the pinned ooRexx tree"),
    )
}

/// The repository root.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// The catalogue's messages, texts included.
fn catalogue() -> Vec<errors::Message> {
    errors::parse_messages(&read(&upstream(), "interpreter/messages/rexxmsg.xml"))
}

/// The value of `--name` among `args`, and the arguments without it.
fn option(args: &mut Vec<String>, name: &str) -> Option<String> {
    let at = args.iter().position(|a| a == name)?;
    assert!(at + 1 < args.len(), "{name} needs a value");
    let value = args.remove(at + 1);
    args.remove(at);
    Some(value)
}

/// Whether the flag `name` is among `args`, removing it.
fn flag(args: &mut Vec<String>, name: &str) -> bool {
    let found = args.iter().position(|a| a == name);
    if let Some(at) = found {
        args.remove(at);
    }
    found.is_some()
}

fn run_similarity(args: &[String]) -> ExitCode {
    let mut args = args.to_vec();
    let mut threshold = similarity::Threshold::default();
    if let Some(d) = option(&mut args, "--threshold") {
        threshold.dice = d.parse().expect("--threshold is a number");
    }
    if let Some(n) = option(&mut args, "--run") {
        threshold.run = n.parse().expect("--run is a number");
    }
    let allow = option(&mut args, "--allow").map(|path| {
        similarity::parse_allow(
            &std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}")),
        )
    });
    let all = flag(&mut args, "--all");
    if !args.is_empty() {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    let dir = repo_root().join("rust/crates/interrexx-inventory/messages");
    let ours = match messages::load_dir(&dir) {
        Ok(texts) => texts,
        Err(problems) => {
            eprintln!("{}", problems.join("\n"));
            return ExitCode::FAILURE;
        }
    };
    let catalogue = catalogue();
    let wanted = if all {
        similarity::Threshold { dice: 0.0, run: 0 }
    } else {
        threshold
    };
    let mut refused = 0;
    for hit in similarity::hits(&ours, &catalogue, wanted) {
        let excuse = allow
            .iter()
            .flatten()
            .find(|(code, _)| *code == (hit.major, hit.sub))
            .map(|(_, reason)| reason);
        let over = threshold.exceeded_by(hit.score);
        let verdict = match (over, excuse) {
            (false, _) => String::new(),
            (true, Some(reason)) => format!("  allowed: {reason}"),
            (true, None) => {
                refused += 1;
                "  TOO CLOSE".to_string()
            }
        };
        println!(
            "{}.{} {}  dice {:.2}  run {}{verdict}",
            hit.major, hit.sub, hit.symbol, hit.score.dice, hit.score.run
        );
    }
    eprintln!("{} texts compared, {refused} too close", ours.len());
    if refused == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_scan(args: &[String]) -> ExitCode {
    let mut args = args.to_vec();
    let comments_only = flag(&mut args, "--comments-only");
    let under = args.first().cloned().unwrap_or_else(|| ".".to_string());
    let patterns: Vec<scan::Pattern> = catalogue().iter().map(scan::Pattern::new).collect();
    let (files, lossy) = match scan::tracked_files(&repo_root(), &under) {
        Ok(found) => found,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let found = scan::scan(&patterns, files, comments_only);
    for hit in &found {
        let kind = match hit.kind {
            scan::Kind::Piece => "piece",
            scan::Kind::Template => "template",
        };
        println!(
            "{}:{}: {}.{} {} ({kind})",
            hit.path, hit.line, hit.major, hit.sub, hit.symbol
        );
    }
    let blind: Vec<String> = scan::unfindable(&patterns)
        .iter()
        .map(|(major, sub)| format!("{major}.{sub}"))
        .collect();
    eprintln!(
        "{} hits; {lossy} files not valid UTF-8 were read lossily; \
         {} codes have no literal to look for: {}",
        found.len(),
        blind.len(),
        blind.join(" ")
    );
    if found.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_brief(args: &[String]) -> ExitCode {
    let [major, out] = args else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let Ok(major) = major.parse::<u16>() else {
        eprintln!("MAJOR is a number");
        return ExitCode::from(2);
    };
    let root = repo_root();
    let crates = root.join("rust/crates");
    let mut code: Vec<PathBuf> = std::fs::read_dir(&crates)
        .expect("lists the crates")
        .filter_map(Result::ok)
        .filter(|e| e.file_name() != "interrexx-upstream")
        .map(|e| e.path().join("src"))
        .filter(|p| p.is_dir())
        .collect();
    code.sort();
    let sources = brief::Sources {
        errors: crates.join("interrexx-inventory/data/errors.tsv"),
        probes: crates.join("interrexx-upstream/data/probes"),
        code,
        spec: root.join("spec/src"),
        rexx: std::env::var_os("REXX_BIN")
            .map_or_else(|| root.join("rust/target/debug/rexx"), PathBuf::from),
        root,
    };
    // The control: a brief that holds a catalogue text is not written.
    let patterns: Vec<scan::Pattern> = catalogue().iter().map(scan::Pattern::new).collect();
    match brief::write(&sources, major, Path::new(out), Some(&patterns)) {
        Ok(count) => {
            println!("wrote {count} briefs to {out}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// The revision the source tree is checked out at.
fn revision(upstream: &Path) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(upstream)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "{} is not a git checkout: {}",
        upstream.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("a revision is text")
        .trim()
        .to_string()
}
