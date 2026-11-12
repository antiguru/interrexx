//! `ClassGraph::define_all` against the `ClassGraph::define` calls it stands
//! for: every behaviour the two leave answers alike, and the retired
//! behaviours are charged alike.

use interrexx_classes::{ClassGraph, ClassKind, MethodId};
use interrexx_core::ObjRef;

fn id(n: u32) -> ObjRef {
    ObjRef::heap(n, 0)
}

const OBJECT: u32 = 1;
const MIXIN: u32 = 2;
const BASE: u32 = 3;
const SUB: u32 = 4;
const SUBSUB: u32 = 5;

/// A root, a mixin defining `SHARED` and `MIXED`, a base class inheriting it
/// with `SHARED` and `OWN` of its own, and two levels of subclasses below the
/// base, one of which defines `SHARED` itself.
fn graph(rexx_defined: bool) -> ClassGraph {
    let mut g = ClassGraph::new();
    g.define_class(id(OBJECT), None, ClassKind::Regular, id(OBJECT));
    g.define(id(OBJECT), "STRING", MethodId(1));
    g.define_class(id(MIXIN), Some(id(OBJECT)), ClassKind::Mixin, id(OBJECT));
    g.define(id(MIXIN), "SHARED", MethodId(2));
    g.define(id(MIXIN), "MIXED", MethodId(3));
    g.define_class(id(BASE), Some(id(OBJECT)), ClassKind::Regular, id(OBJECT));
    g.inherit(id(BASE), id(MIXIN)).expect("a legal inherit");
    g.define(id(BASE), "OWN", MethodId(4));
    g.define_class(id(SUB), Some(id(BASE)), ClassKind::Regular, id(OBJECT));
    g.define(id(SUB), "SHARED", MethodId(5));
    g.define_class(id(SUBSUB), Some(id(SUB)), ClassKind::Regular, id(OBJECT));
    if rexx_defined {
        for class in [OBJECT, MIXIN, BASE, SUB, SUBSUB] {
            g.set_rexx_defined(id(class));
        }
    }
    g.take_retired_entries();
    g
}

/// Everything a program can ask of one class's two behaviours, in an order
/// that does not depend on how a dictionary is laid out.
fn answers(g: &ClassGraph, class: u32) -> Vec<String> {
    let mut seen = Vec::new();
    for handle in [
        g.instance_behaviour_handle(id(class)),
        g.class_behaviour_handle(id(class)),
    ] {
        let mut entries = g.defined_entries_at(handle);
        entries.sort_by_key(|(name, scope, method)| (name.clone(), scope.bits(), method.0));
        seen.push(format!("{entries:?}"));
        for (name, ..) in &entries {
            seen.push(format!("{name} {:?}", g.lookup_at(handle, name)));
            for scope in [OBJECT, MIXIN, BASE, SUB, SUBSUB] {
                seen.push(format!(
                    "{name} from {scope} {:?} {:?}",
                    g.lookup_from_scope_at(handle, name, id(scope)),
                    g.resolve_super_scope_at(handle, id(scope)),
                ));
            }
        }
    }
    seen
}

fn compare(class: u32, methods: &[(&str, u32)], rexx_defined: bool) {
    let methods: Vec<(String, MethodId)> = methods
        .iter()
        .map(|(name, method)| ((*name).to_owned(), MethodId(*method)))
        .collect();
    let mut one_by_one = graph(rexx_defined);
    let before = one_by_one.instance_behaviour_handle(id(class));
    for (name, method) in &methods {
        one_by_one.define(id(class), name, *method);
    }
    let mut together = graph(rexx_defined);
    together.define_all(id(class), &methods);
    for each in [OBJECT, MIXIN, BASE, SUB, SUBSUB] {
        assert_eq!(
            answers(&one_by_one, each),
            answers(&together, each),
            "class {each} after defining {methods:?} on {class}"
        );
    }
    assert_eq!(
        one_by_one.take_retired_entries(),
        together.take_retired_entries(),
        "retired entries after defining {methods:?} on {class}"
    );
    assert_eq!(
        one_by_one.instance_behaviour_handle(id(class)) != before,
        together.instance_behaviour_handle(id(class)) != before,
        "an instance made before keeps the old behaviour"
    );
    assert_eq!(
        one_by_one.has_retired_behaviours(),
        together.has_retired_behaviours()
    );
}

#[test]
fn defining_together_answers_as_defining_one_by_one() {
    for rexx_defined in [false, true] {
        // New names only, some of them inherited from the mixin or the root.
        compare(BASE, &[("A", 10), ("B", 11), ("C", 12)], rexx_defined);
        compare(
            BASE,
            &[("A", 10), ("SHARED", 11), ("MIXED", 12)],
            rexx_defined,
        );
        compare(
            SUB,
            &[("STRING", 10), ("Lower", 11), ("Z", 12)],
            rexx_defined,
        );
        compare(
            OBJECT,
            &[("A", 10), ("SHARED", 11), ("B", 12)],
            rexx_defined,
        );
        compare(SUBSUB, &[("OWN", 10), ("X", 11)], rexx_defined);
        // What makes the calls go one by one: a name given twice, a name the
        // class defines already, `UNINIT`.
        compare(BASE, &[("A", 10), ("a", 11)], rexx_defined);
        compare(BASE, &[("OWN", 10), ("B", 11)], rexx_defined);
        compare(BASE, &[("UNINIT", 10), ("B", 11)], rexx_defined);
        compare(BASE, &[("ONLY", 10)], rexx_defined);
        compare(BASE, &[], rexx_defined);
    }
}
