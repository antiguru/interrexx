//! `Relation`'s and `Bag`'s own primitive methods, over the store their parent
//! module keeps.

use super::{
    Arity, Cleared, Failure, Interp, Keys, NativeMethod, ObjRef, keys_of, native_hash_supplier,
    not_this_task, owns, pairs, same_index, slot_at, store_of,
};

/// Every live entry under `index`, oldest first, paired with the store it
/// belongs to.
///
/// One pass over the candidates the store held when the search began, each
/// compared at most once: a comparison that changes the receiver never starts
/// the search again, so the search ends whatever the comparisons do. Which
/// entries it answers after such a change is unspecified
/// (`r[class.relation.deviation.33]`).
fn slots_for(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<(super::Store, Vec<usize>), Failure> {
    let keys = keys_of(interp, receiver);
    let code = super::code_of(super::hash_of(interp, keys, index)?);
    let store = store_of(interp, receiver)?;
    // Keep the saved array identities live across replacement callbacks.
    for array in [store.indexes, store.items, store.codes, store.slots] {
        interp.roots.activity_mut().push_temp(array);
    }
    // The candidates and their indexes, read before any comparison sends:
    // the chain of the index's code, which lists each entry of that code
    // once, oldest first.
    let mut candidates = Vec::new();
    {
        let (slots, codes) = super::codes_of(interp, &store)?;
        let indexes = super::array_slots(interp, store.indexes)?;
        if let Ok((_, head)) = super::chain_in(slots, codes, code) {
            for position in codes.chain(head) {
                if let Some(held) = indexes.get(position).copied().flatten() {
                    candidates.push((position, held));
                }
            }
        }
    }
    for &(_, held) in &candidates {
        interp.roots.activity_mut().push_temp(held);
    }
    let mut found = Vec::with_capacity(candidates.len());
    for &(position, held) in &candidates {
        // An earlier comparison may have removed this candidate: one that no
        // longer holds its entry is skipped. A position is filled once, so a
        // live one still holds the entry collected.
        if slot_at(interp, store.indexes, position)? != Some(held) {
            continue;
        }
        if same_index(interp, &store, keys, position, index)?
            && slot_at(interp, store.indexes, position)? == Some(held)
        {
            found.push(position);
        }
    }
    Ok((store, found))
}

/// The oldest live entry under `index`, with the store it belongs to: the
/// search a `Bag` makes, whose item is its index, so one match is the answer.
///
/// The chain is walked from its oldest entry and stops at the first match,
/// each candidate compared at most once. A comparison that changes the
/// receiver's store, or removes the entry it compared, ends the search: its
/// own match stands while its entry is still where it was, and otherwise the
/// search answers none (`r[class.relation.deviation.33]`). A comparison that
/// writes only other stores does not end it.
fn first_slot_for(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<(super::Store, Option<usize>), Failure> {
    let keys = keys_of(interp, receiver);
    let code = super::code_of(super::hash_of(interp, keys, index)?);
    let store = store_of(interp, receiver)?;
    // Keep the saved array identities live across replacement callbacks.
    for array in [store.indexes, store.items, store.codes, store.slots] {
        interp.roots.activity_mut().push_temp(array);
    }
    let mut at = {
        let (slots, codes) = super::codes_of(interp, &store)?;
        super::chain_in(slots, codes, code)
            .ok()
            .map(|(_, head)| head)
    };
    while let Some(position) = at {
        let Some(held) = slot_at(interp, store.indexes, position)? else {
            at = super::codes_of(interp, &store)?.1.next(position);
            continue;
        };
        let generation = interp.store_generation;
        let writes = interp.store_writes;
        let matches = same_index(interp, &store, keys, position, index)?;
        if generation != interp.store_generation || writes != interp.store_writes {
            // The comparison ran code that wrote a store, which may be another
            // collection's. The receiver's own store and the entry at hand
            // decide: unchanged, the walk goes on as if nothing was written.
            let current = store_of(interp, receiver)?;
            let held_still = slot_at(interp, current.indexes, position)? == Some(held);
            if !current.same_layout(&store) || !held_still {
                let stands = matches && current.same_arrays(&store) && held_still;
                return Ok((store, stands.then_some(position)));
            }
        }
        if matches {
            return Ok((store, Some(position)));
        }
        at = super::codes_of(interp, &store)?.1.next(position);
    }
    Ok((store, None))
}

/// `Relation~allAt(index)`: every item under that index, oldest first.
fn native_relation_all_at(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ALLAT"));
    }
    // **93.903 and not 88.901.** `Relation`'s own methods take their index
    // positionally where the other hash collections' take it as a named
    // argument: measured on a `.Relation~new`, `allAt()` and `removeAll()`
    // report 93.903 (argument 1) where `at()` and
    // `remove()` report 88.901, naming `index`.
    let index = super::collection::item_argument(args)?;
    let (store, slots) = slots_for(interp, receiver, index)?;
    let mut items = Vec::new();
    for slot in slots {
        items.push(slot_at(interp, store.items, slot)?.unwrap_or(ObjRef::NIL));
    }
    Ok(Some(super::collection::array_of(interp, items)))
}

/// `Relation~allIndex(item)`: every index whose item matches.
fn native_relation_all_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ALLINDEX"));
    }
    let wanted = super::collection::item_argument(args)?;
    let mut indexes = Vec::new();
    for (index, item) in pairs(interp, receiver)? {
        if super::collection::same_item(interp, wanted, item)? {
            indexes.push(index);
        }
    }
    Ok(Some(super::collection::array_of(interp, indexes)))
}

/// Each index once, in first-seen order.
fn native_relation_unique_indexes(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"UNIQUEINDEXES"));
    }
    let keys = keys_of(interp, receiver);
    // Two indexes are one when they hash alike and `==`, as a lookup sees them.
    let mut unique: Vec<(ObjRef, u64)> = Vec::new();
    for (index, _) in pairs(interp, receiver)? {
        let hash = match keys {
            Keys::Equality => super::hash_of(interp, keys, index)?,
            Keys::Identity | Keys::StringValue => 0,
        };
        let mut seen = false;
        for &(held, held_hash) in &unique {
            let same = match keys {
                Keys::Identity => held == index,
                Keys::Equality => {
                    held_hash == hash && super::collection::same_item(interp, index, held)?
                }
                Keys::StringValue => {
                    let wanted = interp.to_text(index).into_owned();
                    let found = interp.to_text(held).into_owned();
                    wanted == found
                }
            };
            if same {
                seen = true;
                break;
            }
        }
        if !seen {
            unique.push((index, hash));
        }
    }
    let unique = unique.into_iter().map(|(index, _)| index).collect();
    Ok(Some(super::collection::array_of(interp, unique)))
}

/// `Relation~items([index])`: the whole count, or the count under one index.
/// Measured, a relation holding `k -> v1`, `k -> v2` and `j -> w` answers
/// `items` 3, `items('k')` 2 and `items('z')` 0.
fn native_relation_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ITEMS"));
    }
    let count = match args.first().copied().flatten() {
        Some(index) => slots_for(interp, receiver, index)?.1.len(),
        None => super::count_in_store(interp, receiver)?,
    };
    Ok(Some(interp.counted(count)))
}

/// `Relation~removeAll(index)`: takes every entry under that index out and
/// answers them as an array, oldest first -- an empty `Array` for an index the
/// relation does not hold.
fn native_relation_remove_all(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"REMOVEALL"));
    }
    // **93.903 and not 88.901.** `Relation`'s own methods take their index
    // positionally where the other hash collections' take it as a named
    // argument: measured on a `.Relation~new`, `allAt()` and `removeAll()`
    // report 93.903 (argument 1) where `at()` and
    // `remove()` report 88.901, naming `index`.
    let index = super::collection::item_argument(args)?;
    // One lookup, one `hashCode`. Taking an entry moves no other, so the
    // saved positions stay valid while nothing is sent between the takes.
    let (store, slots) = slots_for(interp, receiver, index)?;
    let removed = super::take_positions(interp, receiver, &store, slots)?;
    Ok(Some(super::collection::array_of(interp, removed)))
}

/// `Relation~supplier([index])`: a supplier over the whole relation, or over
/// one index's items.
fn native_relation_supplier(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"SUPPLIER"));
    }
    let Some(index) = args.first().copied().flatten() else {
        return native_hash_supplier(interp, cleared, receiver, &[]);
    };
    let (store, slots) = slots_for(interp, receiver, index)?;
    let mut items = Vec::with_capacity(slots.len());
    for slot in slots {
        items.push(slot_at(interp, store.items, slot)?.unwrap_or(ObjRef::NIL));
    }
    let indexes = vec![index; items.len()];
    let items = super::collection::array_of(interp, items);
    let indexes = super::collection::array_of(interp, indexes);
    super::collection::new_supplier(interp, items, indexes).map(Some)
}

/// `Relation~hasItem(item [, index])`: whether the relation holds that item,
/// optionally under that index. Measured, `hasItem('v1','k')` is 1 and
/// `hasItem('v1','j')` is 0 for a relation holding `k -> v1`.
fn native_relation_has_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"HASITEM"));
    }
    let wanted = super::collection::item_argument(args)?;
    let bag = super::index_only(interp, receiver);
    let supplied_index = args.get(1).copied().flatten();
    // Bag's item is its index. Its optional argument is compared before
    // hashing, and the lookup then sends `==` only to the members of the
    // item's hash up to the first equal one. An index unequal to the item
    // answers 0 (r[class.bag.deviation.45]).
    if bag
        && let Some(index) = supplied_index
        && !super::collection::same_item(interp, wanted, index)?
    {
        return Ok(Some(crate::eval::logical(false)));
    }
    if bag {
        let found = first_slot_for(interp, receiver, wanted)?.1.is_some();
        return Ok(Some(crate::eval::logical(found)));
    }
    let under = supplied_index;
    let (store, candidates) = match under {
        Some(index) => slots_for(interp, receiver, index)?,
        None => {
            let store = store_of(interp, receiver)?;
            let all = super::entries(interp, &store)?;
            (store, all)
        }
    };
    // The active lookup keeps its hash store through growth. These arrays are
    // actually read after callbacks, even if a rebuild detached them from the
    // receiver. EMPTY clears them in place.
    for array in [store.indexes, store.items, store.codes, store.slots] {
        interp.roots.activity_mut().push_temp(array);
    }
    // One pass over the candidates the search began with, each compared at
    // most once, as [`slots_for`] does.
    for position in candidates {
        // A callback may empty this location or replace its value.
        // Read the selected contents' current item only if it is live.
        if slot_at(interp, store.indexes, position)?.is_none() {
            continue;
        }
        if let Some(item) = slot_at(interp, store.items, position)?
            && super::collection::same_item(interp, wanted, item)?
        {
            return Ok(Some(crate::eval::logical(true)));
        }
    }
    Ok(Some(crate::eval::logical(false)))
}

/// `Relation~removeItem(item [, index])`: takes out the first entry holding
/// that item, optionally under that index, and answers it.
fn native_relation_remove_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"REMOVEITEM"));
    }
    let wanted = super::collection::item_argument(args)?;
    let bag = super::index_only(interp, receiver);
    let supplied_index = args.get(1).copied().flatten();
    // Bag's item is its index. Its optional argument is compared before
    // hashing, as hasItem does, and a mismatch removes nothing.
    if bag
        && let Some(index) = supplied_index
        && !super::collection::same_item(interp, wanted, index)?
    {
        return Ok(Some(ObjRef::NIL));
    }
    if bag {
        // A Bag's item is its index, so the first equal index is the match.
        let (store, found) = first_slot_for(interp, receiver, wanted)?;
        let removed = match found {
            Some(position) => super::take_at(interp, receiver, &store, position)?,
            None => None,
        };
        return Ok(Some(removed.unwrap_or(ObjRef::NIL)));
    }
    let under = supplied_index;
    let (store, candidates) = match under {
        Some(index) => slots_for(interp, receiver, index)?,
        None => {
            let store = store_of(interp, receiver)?;
            let all = super::entries(interp, &store)?;
            (store, all)
        }
    };
    // Keep the saved array identities live across replacement callbacks.
    for array in [store.indexes, store.items, store.codes, store.slots] {
        interp.roots.activity_mut().push_temp(array);
    }
    // One pass over the candidates the search began with, each compared at
    // most once, as [`slots_for`] does. A match that a callback has removed or
    // moved to new arrays is skipped.
    for position in candidates {
        if slot_at(interp, store.indexes, position)?.is_none() {
            continue;
        }
        let Some(item) = slot_at(interp, store.items, position)? else {
            continue;
        };
        if super::collection::same_item(interp, wanted, item)?
            && let Some(removed) = super::take_at(interp, receiver, &store, position)?
        {
            return Ok(Some(removed));
        }
    }
    Ok(Some(ObjRef::NIL))
}

/// `Relation`'s and `Bag`'s primitive methods, chained into
/// `ObjectModel::build` beside [`super::NATIVE_METHODS`].
pub(in crate::dispatch) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    // **`Relation` and `Bag` have identical native entry-point sets**, which is
    // why they are one task: `Bag`'s `AllAt`, `AllIndex`, `Items`, `RemoveAll`,
    // `Supplier` and `UniqueIndexes` are `Relation`'s bodies, and only
    // `HasItem` and `RemoveItem` are `Bag`'s own. A Bag's index is its item, so
    // HASITEM specializes the optional argument check and index-only comparison
    // in its shared body.
    ("Relation", "ALLAT", Arity::Fixed(1), native_relation_all_at),
    (
        "Relation",
        "ALLINDEX",
        Arity::Fixed(1),
        native_relation_all_index,
    ),
    (
        "Relation",
        "UNIQUEINDEXES",
        Arity::Fixed(0),
        native_relation_unique_indexes,
    ),
    ("Relation", "ITEMS", Arity::Fixed(1), native_relation_items),
    (
        "Relation",
        "REMOVEALL",
        Arity::Fixed(1),
        native_relation_remove_all,
    ),
    (
        "Relation",
        "SUPPLIER",
        Arity::Fixed(1),
        native_relation_supplier,
    ),
    (
        "Relation",
        "HASITEM",
        Arity::Fixed(2),
        native_relation_has_item,
    ),
    (
        "Relation",
        "REMOVEITEM",
        Arity::Fixed(2),
        native_relation_remove_item,
    ),
    ("Bag", "HASITEM", Arity::Fixed(2), native_relation_has_item),
    (
        "Bag",
        "REMOVEITEM",
        Arity::Fixed(2),
        native_relation_remove_item,
    ),
];
