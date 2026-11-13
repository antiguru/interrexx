//! `interrexx-upstream write` rewrites the checked-in interface tables from
//! the pinned ooRexx tree (`$REXX_UPSTREAM`). `interrexx-upstream check`
//! derives them in memory and exits non-zero, printing the difference, when a
//! checked-in table is not what the tree gives.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use interrexx_upstream::{builtins, classes, difference, errors};

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

fn main() -> ExitCode {
    let mode = std::env::args().nth(1);
    let write = match mode.as_deref() {
        Some("write") => true,
        Some("check") => false,
        _ => {
            eprintln!("usage: interrexx-upstream (write | check)");
            return ExitCode::from(2);
        }
    };
    let upstream = PathBuf::from(
        std::env::var_os("REXX_UPSTREAM").expect("REXX_UPSTREAM names the pinned ooRexx tree"),
    );
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
