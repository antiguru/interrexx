//! `interrexx-upstream write` rewrites the checked-in interface tables from
//! the pinned ooRexx tree (`$REXX_UPSTREAM`). `interrexx-upstream check`
//! derives them in memory and exits non-zero, printing the difference, when a
//! checked-in table is not what the tree gives.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use interrexx_upstream::{classes, difference};

/// One checked-in table: where it lives (relative to the `crates` directory)
/// and how to derive its text from the source tree.
struct Table {
    path: &'static str,
    derive: fn(upstream: &Path, sha: &str) -> String,
}

const TABLES: &[Table] = &[Table {
    path: "interrexx-classes/data/classes.tsv",
    derive: |upstream, sha| {
        let source = upstream.join("interpreter/memory/Setup.cpp");
        let text = std::fs::read_to_string(&source)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", source.display()));
        classes::Table::parse(&text).to_tsv(sha)
    },
}];

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
