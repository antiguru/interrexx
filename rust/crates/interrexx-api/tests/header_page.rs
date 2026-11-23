// SPDX-License-Identifier: MIT OR Apache-2.0

//! The lists of the header page (`spec/src/native/header.md`) against the Rust tables the
//! headers are generated from.
//!
//! The page lists constants, exit flag bits and convenience methods by name and value. Each
//! list is read here and compared with the table `src/header.rs` writes the header from, so the
//! page and the header cannot disagree; the ABI dump (`abi_dump.rs`) then checks the header
//! against a C and a C++ compiler.

use interrexx_api::constants::{
    API_VERSIONS, ARGUMENT_TYPES, CLASSIC_CONSTANTS, CLASSIC_LIMITS, ENTRY_CONSTANTS,
    OPTIONAL_VALUES, VALUE_ALIASES,
};
use interrexx_api::layout::{EXIT_FLAG_BITS, THREAD_ALIASES, c_type};
use interrexx_api::values::rows;
use std::path::PathBuf;

fn page() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../spec/src/native/header.md");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The items of the list in the paragraph of rule `id`, each joined onto one line.
fn items(page: &str, id: &str) -> Vec<String> {
    let marker = format!("r[{id}]");
    let mut lines = page
        .lines()
        .skip_while(|line| *line != marker)
        .skip(1)
        .take_while(|line| !line.is_empty());
    let mut out: Vec<String> = Vec::new();
    for line in lines.by_ref() {
        if let Some(item) = line.strip_prefix("* ") {
            out.push(item.to_string());
        } else if let Some(last) = out.last_mut() {
            last.push(' ');
            last.push_str(line.trim_start());
        }
    }
    assert!(!out.is_empty(), "rule {id} has no list");
    out
}

/// spec: native.header.value-codes
#[test]
fn the_page_lists_every_value_code() {
    let type_of = |name: &str| -> String {
        let (_, tokens) = ARGUMENT_TYPES
            .iter()
            .find(|(row, _)| *row == name)
            .unwrap_or_else(|| panic!("no argument type for {name}"));
        c_type(tokens)
    };
    let line = |name: &str, code: u16, optional: bool| {
        let form = if optional {
            ", with an optional form"
        } else {
            ""
        };
        format!("`{name}`: code {code}, type `{}`{form}", type_of(name))
    };
    let mut expected: Vec<String> = rows()
        .map(|(code, name)| line(name, code, OPTIONAL_VALUES.contains(&name)))
        .collect();
    for (alias, name) in VALUE_ALIASES {
        let (code, _) = rows()
            .find(|(_, row)| row == name)
            .expect("an alias names a code");
        expected.push(line(alias, code, false));
    }
    assert_eq!(items(&page(), "native.header.value-codes"), expected);
}

/// spec: native.header.versions
#[test]
fn the_page_lists_every_version() {
    let expected: Vec<String> = API_VERSIONS
        .iter()
        .map(|constant| format!("`{}`: `{}`", constant.name, constant.text()))
        .collect();
    assert_eq!(items(&page(), "native.header.versions"), expected);
}

/// spec: native.header.entry-constants
#[test]
fn the_page_lists_every_entry_constant() {
    let listed: Vec<String> = items(&page(), "native.header.entry-constants")
        .into_iter()
        .map(|item| item.split(',').next().unwrap_or_default().to_string())
        .collect();
    let expected: Vec<String> = ENTRY_CONSTANTS
        .iter()
        .map(|constant| format!("`{}`: `{}`", constant.name, constant.text()))
        .collect();
    assert_eq!(listed, expected);
}

// spec: native.header.classic-start native.header.classic-handlers native.header.classic-functions
// spec: native.header.classic-service native.header.classic-queues native.header.classic-macros
#[test]
fn the_page_lists_the_classic_start_registration_and_service_constants() {
    let page = page();
    let mut listed = Vec::new();
    for rule in [
        "start",
        "handlers",
        "functions",
        "service",
        "queues",
        "macros",
    ] {
        for item in items(&page, &format!("native.header.classic-{rule}")) {
            let (definition, _) = item.split_once(',').expect("the constant has a meaning");
            let (names, value) = definition
                .split_once(": ")
                .expect("the constant has a value");
            listed.extend(names.split(" / ").map(|name| format!("{name}: {value}")));
        }
    }
    let prefixes = [
        "RXSUBCOM_",
        "RXEXIT_",
        "RXFUNC_",
        "RXAPI_",
        "RXQUEUE_",
        "RXMACRO_",
    ];
    let single = [
        "RXCOMMAND",
        "RXSUBROUTINE",
        "RXFUNCTION",
        "RXMETHOD",
        "RXSCRIPT",
        "RXAUTOBUFLEN",
        "MAX_QUEUE_NAME_LENGTH",
    ];
    let mut expected: Vec<String> = CLASSIC_CONSTANTS
        .iter()
        .chain(CLASSIC_LIMITS)
        .filter(|constant| {
            single.contains(&constant.name.as_ref())
                || prefixes
                    .iter()
                    .any(|prefix| constant.name.starts_with(prefix))
        })
        .map(|constant| format!("`{}`: `{}`", constant.name, constant.text()))
        .collect();
    listed.sort_unstable();
    expected.sort_unstable();
    assert_eq!(listed, expected);
}

/// spec: native.header.exit-flags
#[test]
fn the_page_gives_every_exit_flag_its_bit() {
    let listed = items(&page(), "native.header.exit-flags");
    for (word, flags) in EXIT_FLAG_BITS {
        let item = listed
            .iter()
            .find(|item| {
                let names = item.split(':').next().unwrap_or_default();
                names.contains(&format!("`{word}`"))
            })
            .unwrap_or_else(|| panic!("the page lists no item for {word}"));
        for (flag, bit) in *flags {
            assert!(
                item.contains(&format!("`{flag}` (bit {bit})")),
                "{word}.{flag} is bit {bit}, which the page's item does not say: {item}"
            );
        }
    }
}

/// spec: native.header.convenience
#[test]
fn the_page_lists_every_convenience_method() {
    let mut expected: Vec<(&str, Vec<&str>)> = Vec::new();
    for alias in THREAD_ALIASES {
        match expected.iter_mut().find(|(name, _)| *name == alias.name) {
            Some((_, targets)) => targets.push(alias.target),
            None => expected.push((alias.name, vec![alias.target])),
        }
    }
    let expected: Vec<String> = expected
        .into_iter()
        .map(|(name, targets)| {
            let targets: Vec<String> = targets.iter().map(|t| format!("`{t}`")).collect();
            format!("`{name}`: {}", targets.join(", "))
        })
        .collect();
    assert_eq!(items(&page(), "native.header.convenience"), expected);
}
