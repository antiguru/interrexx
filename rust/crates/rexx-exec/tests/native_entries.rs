//! Every `LIBRARY REXX` entry point the registry names runs rather than
//! refusing a send.

#[test]
fn every_registry_entry_point_is_implemented() {
    let rows = rexx_exec::native_entry_points();
    assert!(!rows.is_empty(), "the registry names no entry points");
    let refused: Vec<&str> = rows
        .iter()
        .filter(|row| !row.implemented || row.owner.is_some())
        .map(|row| row.entry)
        .collect();
    assert!(
        refused.is_empty(),
        "these `LIBRARY REXX` entry points refuse a send: {refused:?}"
    );
}
