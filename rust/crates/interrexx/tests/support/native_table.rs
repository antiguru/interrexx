//! The interpreter's native class table, read from the checked-in
//! `interrexx-classes/data/classes.tsv`, for the native-or-Rexx column and the
//! arity that `collection_scopes.rs` and `introspection_scopes.rs` both join
//! onto the oracle's own scope answer.

#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use interrexx_upstream::classes::{Arity, Kind, Table};

/// The arity of one native method, as the scope tables write it: a number or `any`.
pub type NativeArity = String;

/// Every class block's instance and class tables, with
/// `InheritInstanceMethods` resolved, keyed by the block name and then the
/// upper-cased method name.
pub fn native_arities() -> (
    HashMap<String, HashMap<String, NativeArity>>,
    HashMap<String, HashMap<String, NativeArity>>,
) {
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../interrexx-classes/data/classes.tsv"
    ));
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read the native table {}: {e}", path.display()));
    arities_of(&Table::from_tsv(&text))
}

fn arities_of(
    table: &Table,
) -> (
    HashMap<String, HashMap<String, NativeArity>>,
    HashMap<String, HashMap<String, NativeArity>>,
) {
    let mut instance: HashMap<String, HashMap<String, NativeArity>> = HashMap::new();
    let mut class: HashMap<String, HashMap<String, NativeArity>> = HashMap::new();
    for block in &table.blocks {
        instance.entry(block.name.clone()).or_default();
        class.entry(block.name.clone()).or_default();
        for op in &block.ops {
            match op.kind {
                Kind::InheritInstanceMethods => {
                    let inherited = instance
                        .get(&op.name)
                        .unwrap_or_else(|| {
                            panic!(
                                "InheritInstanceMethods({}) names a block not yet defined",
                                op.name
                            )
                        })
                        .clone();
                    let own = instance.entry(block.name.clone()).or_default();
                    for (name, arity) in inherited {
                        own.entry(name).or_insert(arity);
                    }
                }
                Kind::RemoveInstanceMethod | Kind::HideInstanceMethod => {}
                _ => {
                    let arity = match op.arity.expect("an adding operation has an arity") {
                        Arity::Max(n) => n.to_string(),
                        Arity::Any => "any".to_string(),
                    };
                    let tables = if op.kind.is_class_side() {
                        &mut class
                    } else {
                        &mut instance
                    };
                    tables
                        .entry(block.name.clone())
                        .or_default()
                        .insert(op.name.to_uppercase(), arity);
                }
            }
        }
    }
    (instance, class)
}
