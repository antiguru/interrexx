//! [`ScopePools`], the storage `EXPOSE` binds a method's names to.

use interrexx_core::{BehaviourHandle, Body, ObjRef, ScopePools};

fn scopes() -> (ObjRef, ObjRef) {
    (ObjRef::heap(1_000_000, 0), ObjRef::heap(1_000_000 + 1, 0))
}

#[test]
fn a_name_the_pool_never_held_reads_as_unset() {
    let (sup, _) = scopes();
    assert_eq!(ScopePools::new().get(sup, b"V"), None);
}

/// The pool's whole reason for being keyed on a scope: one object, one name,
/// two values at once. Measured on the oracle -- a class method on `sup` and
/// one on `sub subclass sup`, each `expose v`, each writing its own value, and
/// each reading it back through the receiving class object `.sub` -- prints
/// `S B`. A pool keyed on the name alone prints `B B`.
#[test]
fn two_scopes_hold_one_name_at_two_values() {
    let (sup, sub) = scopes();
    let s = ObjRef::heap(1, 0);
    let b = ObjRef::heap(2, 0);
    let mut pools = ScopePools::new();
    pools.set(sup, b"V", s);
    pools.set(sub, b"V", b);
    assert_eq!(pools.get(sup, b"V"), Some(s));
    assert_eq!(pools.get(sub, b"V"), Some(b));
}

#[test]
fn a_second_write_to_one_name_replaces_rather_than_shadows() {
    let (sup, _) = scopes();
    let first = ObjRef::heap(1, 0);
    let second = ObjRef::heap(2, 0);
    let mut pools = ScopePools::new();
    pools.set(sup, b"V", first);
    pools.set(sup, b"V", second);
    assert_eq!(pools.get(sup, b"V"), Some(second));
    // A shadowing push would leave the first value reachable and would make
    // the pool grow without bound under a loop that assigns one variable.
    let mut out = Vec::new();
    let class = ObjRef::heap(1_000_000 + 2, 0);
    Body::Instance {
        class,
        behaviour: BehaviourHandle::new(0),
        name: None,
        pools,
        own: None,
        native: None,
    }
    .trace(&mut out);
    assert_eq!(out, vec![class, sup, second]);
}

/// `DROP` on an exposed name, and the neighbouring success that pins it to the
/// name rather than to the pool: the other scope's binding of the same name is
/// untouched.
#[test]
fn clearing_one_scopes_binding_leaves_the_others() {
    let (sup, sub) = scopes();
    let s = ObjRef::heap(1, 0);
    let b = ObjRef::heap(2, 0);
    let mut pools = ScopePools::new();
    pools.set(sup, b"V", s);
    pools.set(sub, b"V", b);
    pools.clear(sup, b"V");
    assert_eq!(pools.get(sup, b"V"), None);
    assert_eq!(pools.get(sub, b"V"), Some(b));
}

#[test]
fn clearing_a_name_the_pool_never_held_is_a_no_op() {
    let (sup, sub) = scopes();
    let s = ObjRef::heap(1, 0);
    let mut pools = ScopePools::new();
    pools.set(sup, b"V", s);
    pools.clear(sup, b"NEVERBOUND");
    pools.clear(sub, b"V");
    assert_eq!(pools.get(sup, b"V"), Some(s));
}

/// A scope that binds a second name after another scope has bound one keeps
/// its entries together, in binding order, which is what a store's five
/// names read back as a run rely on.
#[test]
fn a_scopes_entries_stay_together_when_scopes_interleave() {
    let (sup, sub) = scopes();
    let (v, w, x) = (ObjRef::heap(1, 0), ObjRef::heap(2, 0), ObjRef::heap(3, 0));
    let y = ObjRef::heap(4, 0);
    let mut pools = ScopePools::new();
    pools.set(sup, b"V", v);
    pools.set(sub, b"V", x);
    pools.set(sup, b"W", w);
    pools.set(sup, b"Y", y);
    let names = |scope| {
        pools
            .entries(scope)
            .iter()
            .map(|entry| (entry.name.to_vec(), entry.value))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        names(sup),
        vec![(b"V".to_vec(), v), (b"W".to_vec(), w), (b"Y".to_vec(), y)]
    );
    assert_eq!(names(sub), vec![(b"V".to_vec(), x)]);
    assert_eq!(pools.get(sup, b"W"), Some(w));
    // The pool created last that binds a name answers for it.
    assert_eq!(pools.find(b"V"), Some(x));
    assert_eq!(pools.find(b"W"), Some(w));
}

/// A scope is traced once, ahead of the values bound in it, whatever order
/// its names were bound in.
#[test]
fn tracing_names_each_scope_once() {
    let (sup, sub) = scopes();
    let (v, w, x) = (ObjRef::heap(1, 0), ObjRef::heap(2, 0), ObjRef::heap(3, 0));
    let mut pools = ScopePools::new();
    pools.set(sup, b"V", v);
    pools.set(sub, b"V", x);
    pools.set(sup, b"W", w);
    let class = ObjRef::heap(1_000_000 + 2, 0);
    let mut out = Vec::new();
    Body::Instance {
        class,
        behaviour: BehaviourHandle::new(0),
        name: None,
        pools,
        own: None,
        native: None,
    }
    .trace(&mut out);
    assert_eq!(out, vec![class, sup, v, w, sub, x]);
}

/// A scope keeps its place among the scopes when it drops every name and then
/// binds again, so the pool created last that binds a name is still the one
/// that was created last.
#[test]
fn an_emptied_scope_keeps_its_place_when_it_binds_again() {
    let (sup, sub) = scopes();
    let (v, x, again) = (ObjRef::heap(1, 0), ObjRef::heap(2, 0), ObjRef::heap(3, 0));
    let mut pools = ScopePools::new();
    pools.set(sup, b"V", v);
    pools.set(sub, b"V", x);
    pools.clear(sup, b"V");
    assert_eq!(pools.find(b"V"), Some(x));
    pools.set(sup, b"V", again);
    assert_eq!(pools.find(b"V"), Some(x));
    assert_eq!(pools.get(sup, b"V"), Some(again));
}

/// The same, through a scope that was emptied and then bound a different name
/// while a later scope was also bound in the meantime.
#[test]
fn an_emptied_scope_keeps_its_place_ahead_of_scopes_bound_after_it() {
    let (sup, sub) = scopes();
    let third = ObjRef::heap(1_000_000 + 5, 0);
    let (v, x, t, again) = (
        ObjRef::heap(1, 0),
        ObjRef::heap(2, 0),
        ObjRef::heap(3, 0),
        ObjRef::heap(4, 0),
    );
    let mut pools = ScopePools::new();
    pools.set(sup, b"V", v);
    pools.set(sub, b"V", x);
    pools.clear(sup, b"V");
    pools.set(third, b"V", t);
    pools.set(sup, b"V", again);
    // Order of first binding is sup, sub, third.
    assert_eq!(pools.find(b"V"), Some(t));
    pools.clear(third, b"V");
    assert_eq!(pools.find(b"V"), Some(x));
}

/// The order records that keep an emptied scope's place are not variables: no
/// name reads them, and the scopes still trace as before.
#[test]
fn an_emptied_scope_leaves_no_variable_behind() {
    let (sup, sub) = scopes();
    let (v, x, again) = (ObjRef::heap(1, 0), ObjRef::heap(2, 0), ObjRef::heap(3, 0));
    let mut pools = ScopePools::new();
    pools.set(sup, b"V", v);
    pools.set(sub, b"V", x);
    pools.clear(sup, b"V");
    assert_eq!(pools.get(sup, b"V"), None);
    assert!(pools.entries(sup).is_empty());
    assert_eq!(pools.find(b""), None);
    pools.set(sup, b"W", again);
    assert_eq!(pools.entries(sup).len(), 1);
    assert_eq!(pools.entries(sub).len(), 1);
    assert_eq!(pools.find(b"W"), Some(again));
}

/// A method run by `~run` exposes in the nil scope, which is a scope like any
/// other: the pool created last that binds a name answers for it.
#[test]
fn a_name_bound_in_the_nil_scope_is_found() {
    let y = ObjRef::heap(1, 0);
    let mut pools = ScopePools::new();
    pools.set(ObjRef::NIL, b"CSELF", y);
    assert_eq!(pools.find(b"CSELF"), Some(y));
}

/// An emptied scope keeps its place when the scope bound last is the nil
/// scope.
#[test]
fn an_emptied_scope_keeps_its_place_after_a_nil_scope_binding() {
    let (sup, sub) = scopes();
    let (v, x, y, again) = (
        ObjRef::heap(1, 0),
        ObjRef::heap(2, 0),
        ObjRef::heap(3, 0),
        ObjRef::heap(4, 0),
    );
    let mut pools = ScopePools::new();
    pools.set(sup, b"CSELF", v);
    pools.set(sub, b"CSELF", x);
    pools.set(ObjRef::NIL, b"Y", y);
    pools.clear(sup, b"CSELF");
    pools.set(sup, b"CSELF", again);
    assert_eq!(pools.find(b"CSELF"), Some(x));
    assert_eq!(pools.get(sup, b"CSELF"), Some(again));
}

/// The nil scope's entries are only the names bound in it, whatever order
/// records an emptied scope leaves.
#[test]
fn the_nil_scopes_entries_are_its_own_names() {
    let (sup, sub) = scopes();
    let (v, x, y, again) = (
        ObjRef::heap(1, 0),
        ObjRef::heap(2, 0),
        ObjRef::heap(3, 0),
        ObjRef::heap(4, 0),
    );
    let mut pools = ScopePools::new();
    pools.set(sup, b"CSELF", v);
    pools.set(sub, b"CSELF", x);
    pools.set(ObjRef::NIL, b"Y", y);
    pools.clear(sup, b"CSELF");
    pools.set(sup, b"CSELF", again);
    let names = pools
        .entries(ObjRef::NIL)
        .iter()
        .map(|entry| (entry.name.to_vec(), entry.value))
        .collect::<Vec<_>>();
    assert_eq!(names, vec![(b"Y".to_vec(), y)]);
}
