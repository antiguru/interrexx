//! No tracked file refers to the development records the repository no longer holds.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Paths that were removed; a reference to one is a dangling pointer.
const REMOVED: &str = r"docs/superpowers|\.superpowers/|phase-4-exclusions|rust/CLAUDE\.md|ooRexx-commit-map|docs/history";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root exists")
}

/// The tracked files `git grep` finds matching `pattern`, this file and `.gitignore` excepted.
fn tracked_files_matching(pattern: &str) -> Vec<String> {
    let out = Command::new("git")
        .current_dir(repo_root())
        .args(["grep", "-l", "-I", "-E", pattern, "--", "."])
        .args([
            ":!.gitignore",
            ":!rust/crates/interrexx/tests/repo_hygiene.rs",
        ])
        .output()
        .expect("running git grep");
    // git grep exits 1 when nothing matches and 2 or more on an error.
    assert!(
        out.status.code().is_some_and(|c| c <= 1),
        "git grep failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn no_tracked_file_refers_to_removed_records() {
    let found = tracked_files_matching(REMOVED);
    assert!(
        found.is_empty(),
        "files referring to removed paths: {found:#?}"
    );
}

#[test]
fn the_search_finds_a_reference_when_one_exists() {
    let found = tracked_files_matching("docs/design/architecture");
    assert!(
        !found.is_empty(),
        "the search found nothing, so an empty result above would prove nothing"
    );
}
