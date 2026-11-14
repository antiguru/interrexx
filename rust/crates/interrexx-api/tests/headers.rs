// SPDX-License-Identifier: MIT OR Apache-2.0

//! The headers in `include/` hold what the Rust definitions generate.
//!
//! Run with `REXX_HEADERS_REFRESH=1` to write the generated sections into the
//! files instead of comparing; the hand-written text around them is kept.

use interrexx_api::header::{ErrorRow, files, sections, splice};
use interrexx_upstream::errors::Table;
use std::path::PathBuf;

/// The repository's `include/` directory.
fn include_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../include")
}

/// The error table, from which `oorexxerrors.h` is written.
fn error_table() -> Table {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../interrexx-inventory/data/errors.tsv");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    Table::from_tsv(&text)
}

/// **Every header's generated sections are what the Rust definitions give**:
/// each file under `include/` that has generated sections equals itself with
/// those sections rendered afresh.
#[test]
fn headers_are_current() {
    let refresh = std::env::var_os("REXX_HEADERS_REFRESH").is_some_and(|v| v == "1");
    let table = error_table();
    let errors: Vec<ErrorRow<'_>> = table
        .rows
        .iter()
        .map(|row| ErrorRow {
            major: row.major,
            sub: row.sub,
            symbol: &row.symbol,
        })
        .collect();
    let all = sections(&errors);
    let mut stale = Vec::new();
    for file in files() {
        let path = include_dir().join(&file);
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let mine: Vec<(&str, &str)> = all
            .iter()
            .filter(|section| section.file == file)
            .map(|section| (section.name, section.text.as_str()))
            .collect();
        let fresh = splice(&text, &mine).unwrap_or_else(|e| panic!("{file}: {e}"));
        if fresh != text {
            if refresh {
                std::fs::write(&path, fresh).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            } else {
                stale.push(file);
            }
        }
    }
    assert!(
        stale.is_empty(),
        "these headers differ from what the Rust definitions generate; rerun with \
         REXX_HEADERS_REFRESH=1 and read the diff: {stale:?}"
    );
}
