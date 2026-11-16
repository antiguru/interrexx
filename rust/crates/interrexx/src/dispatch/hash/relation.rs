//! `Relation`'s and `Bag`'s own primitive methods, over the store their parent
//! module keeps.

use super::{
    Arity, Cleared, Failure, Interp, Keys, NativeMethod, ObjRef, keys_of, native_hash_supplier,
    not_this_task, owns, pairs, same_index, slot_at, store_of,
};

/// Every live entry under `index`, oldest first, paired with the store it
/// belongs to.
fn slots_for(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<(super::Store, Vec<usize>), Failure> {
    let keys = keys_of(interp, receiver);
    let code = super::code_of(super::hash_of(interp, keys, index)?);
    'search: loop {
        let store = store_of(interp, receiver)?;
        // Keep the saved array identities live across replacement callbacks.
        for array in [store.indexes, store.items, store.codes, store.slots] {
            interp.roots.activity_mut().push_temp(array);
        }
        // The candidates and their indexes, read before any comparison sends.
        let mut candidates = Vec::new();
        {
            let slots = super::array_slots(interp, store.slots)?;
            let codes = super::array_slots(interp, store.codes)?;
            let indexes = super::array_slots(interp, store.indexes)?;
            let mut sequence = super::Sequence::new(code, slots.len());
            loop {
                match super::mark_in(slots, sequence.at()) {
                    super::Mark::Empty => break,
                    super::Mark::Deleted => {}
                    super::Mark::Entry(position) => {
                        if super::code_in(codes, position) == Some(code)
                            && let Some(held) = indexes.get(position).copied().flatten()
                        {
                            candidates.push((position, held));
                        }
                    }
                }
                sequence.advance();
            }
        }
        // The sequence can pass one slot more than once before it reaches an
        // empty one, so a candidate can be met twice: each entry is counted
        // once. Entries of one code first meet the sequence in the order they
        // were added, since an append files at the first empty slot and never
        // reuses a deleted one; sorted all the same, so the order rests on
        // positions.
        candidates.sort_unstable_by_key(|&(position, _)| position);
        candidates.dedup_by_key(|&mut (position, _)| position);
        for &(_, held) in &candidates {
            interp.roots.activity_mut().push_temp(held);
        }
        let mut found = Vec::new();
        for &(position, _) in &candidates {
            let writes = interp.store_writes;
            let generation = interp.store_generation;
            let matches = same_index(interp, &store, keys, position, index)?;
            // Equality can replace arrays or remove an earlier candidate.
            // Validate every saved candidate before retaining positions.
            if writes != interp.store_writes || generation != interp.store_generation {
                let current = store_of(interp, receiver)?;
                if current != store {
                    continue 'search;
                }
                for &(saved, held) in &candidates {
                    if slot_at(interp, current.indexes, saved)? != Some(held) {
                        continue 'search;
                    }
                }
            }
            if matches {
                found.push(position);
            }
        }
        return Ok((store, found));
    }
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
    let mut removed = Vec::new();
    for position in slots {
        if let Some(item) = super::take_at(interp, receiver, &store, position)? {
            interp.roots.activity_mut().push_temp(item);
            removed.push(item);
        }
    }
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
    let mut items = Vec::new();
    let mut indexes = Vec::new();
    for slot in slots {
        items.push(slot_at(interp, store.items, slot)?.unwrap_or(ObjRef::NIL));
        indexes.push(slot_at(interp, store.indexes, slot)?.unwrap_or(ObjRef::NIL));
    }
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
    // hashing, and a successful lookup sends only the index comparison. An
    // index unequal to the item answers 0 (r[class.bag.deviation.45]).
    if bag
        && let Some(index) = supplied_index
        && !super::collection::same_item(interp, wanted, index)?
    {
        return Ok(Some(crate::eval::logical(false)));
    }
    let under = if bag { Some(wanted) } else { supplied_index };
    'search: loop {
        let (store, candidates) = match under {
            Some(index) => slots_for(interp, receiver, index)?,
            None => {
                let store = store_of(interp, receiver)?;
                let all = super::entries(interp, &store)?;
                (store, all)
            }
        };
        if bag && !candidates.is_empty() {
            return Ok(Some(crate::eval::logical(true)));
        }
        // The active lookup keeps its hash store through growth. These arrays
        // are actually read after callbacks, even if a rebuild detached them
        // from the receiver. EMPTY clears them in place.
        for array in [store.indexes, store.items, store.codes, store.slots] {
            interp.roots.activity_mut().push_temp(array);
        }
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
            // A comparison that gave the receiver new arrays or new entries
            // leaves the saved candidates short of them: search again, as
            // `removeItem` does.
            if store_of(interp, receiver)? != store {
                continue 'search;
            }
        }
        return Ok(Some(crate::eval::logical(false)));
    }
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
    let under = if bag { Some(wanted) } else { supplied_index };
    'search: loop {
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
        for position in candidates {
            let Some(item) = slot_at(interp, store.items, position)? else {
                continue;
            };
            // A Bag's item is its index, which already matched.
            if bag || super::collection::same_item(interp, wanted, item)? {
                if let Some(removed) = super::take_at(interp, receiver, &store, position)? {
                    return Ok(Some(removed));
                }
                continue 'search;
            }
            if store_of(interp, receiver)? != store {
                continue 'search;
            }
        }
        return Ok(Some(ObjRef::NIL));
    }
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
