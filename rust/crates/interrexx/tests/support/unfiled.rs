//! `corpus/unfiled.txt`, read alike by the corpus gate and by the ruled
//! runner.

use std::fs;
use std::path::Path;

/// The companion file naming the ones it does not run, and why.
pub const UNFILED_FILE: &str = "unfiled.txt";

/// The reason an [`UNFILED_FILE`] entry gives when the ruled runner pins the
/// program's answer. That runner hands the program its `.stdin` and nothing
/// else of a sidecar.
pub const RULED_RUNNER: &str =
    "input_oracle::the_unfiled_ruled_programs_answer_as_this_crate_rules";

/// Every `path<TAB>reason` in `corpus/unfiled.txt`.
pub fn read_unfiled(corpus_dir: &Path) -> Vec<(String, String)> {
    let path = corpus_dir.join(UNFILED_FILE);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (entry, reason) = line.split_once('\t').unwrap_or_else(|| {
                panic!("{}: {line:?} is not a `path<TAB>reason`", path.display())
            });
            assert!(
                !reason.trim().is_empty(),
                "{}: {entry} is named with no reason",
                path.display()
            );
            (entry.to_string(), reason.to_string())
        })
        .collect()
}
