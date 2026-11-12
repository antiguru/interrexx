//! `Relation`'s and `Bag`'s own primitive methods, over the store their parent
//! module keeps.

use super::{
    Arity, Cleared, Failure, Interp, Keys, NativeMethod, ObjRef, keys_of, native_hash_supplier,
    not_this_task, owns, pairs, same_index, slot_at, store_of, walk,
};

/// Every matching slot in chain order, paired with the store it belongs to.
fn slots_for(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<(super::Store, Vec<usize>), Failure> {
    let (store, _, found) = slots_in_bucket(interp, receiver, index)?;
    Ok((store, found))
}

/// [`slots_for`], also answering the bucket the index hashes to.
fn slots_in_bucket(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<(super::Store, usize, Vec<usize>), Failure> {
    let keys = keys_of(interp, receiver);
    let hash = super::hash_of(interp, keys, index)?;
    'search: loop {
        let store = store_of(interp, receiver)?;
        // Keep the saved array identities live across replacement callbacks.
        for array in [store.indexes, store.items, store.next] {
            interp.roots.activity_mut().push_temp(array);
        }
        let bucket = (hash % store.buckets as u64) as usize;
        let entries = super::entries_in(interp, &store, Some(bucket))?;
        let mut indexes = Vec::with_capacity(entries.len());
        for &(slot, _) in &entries {
            let held = slot_at(interp, store.indexes, slot)?;
            if let Some(held) = held {
                interp.roots.activity_mut().push_temp(held);
            }
            indexes.push(held);
        }
        let mut found = Vec::new();
        for &(slot, _) in &entries {
            let writes = interp.store_writes;
            let generation = interp.store_generation;
            let matches = same_index(interp, &store, keys, slot, index)?;
            // Equality can replace arrays or change an earlier collision
            // entry. Validate the whole saved chain before retaining slots.
            if writes != interp.store_writes || generation != interp.store_generation {
                let current = store_of(interp, receiver)?;
                if current != store || super::entries_in(interp, &current, Some(bucket))? != entries
                {
                    continue 'search;
                }
                for (&(saved, _), &held) in entries.iter().zip(&indexes) {
                    if slot_at(interp, current.indexes, saved)? != held {
                        continue 'search;
                    }
                }
            }
            if matches {
                found.push(slot);
            }
        }
        return Ok((store, bucket, found));
    }
}

/// `Relation~allAt(index)`: every item under that index, newest first.
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
    // report `Missing argument in method; argument 1 is required.` where
    // `at()` and `remove()` report `Missing argument; argument index is
    // required.`
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
        None => walk(interp, receiver)?.len(),
    };
    Ok(Some(interp.counted(count)))
}

/// `Relation~removeAll(index)`: takes every entry under that index out and
/// answers them as an array, newest first -- an empty `Array` for an index the
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
    // report `Missing argument in method; argument 1 is required.` where
    // `at()` and `remove()` report `Missing argument; argument index is
    // required.`
    let index = super::collection::item_argument(args)?;
    // One lookup, one `hashCode`. Entries are taken last first: taking the
    // bucket's own slot copies its successor into it, which would move a
    // later matching entry out from under its saved slot.
    let (store, bucket, slots) = slots_in_bucket(interp, receiver, index)?;
    let mut removed = Vec::new();
    for slot in slots.into_iter().rev() {
        if let Some(item) = super::take_at(interp, receiver, &store, slot, bucket)? {
            interp.roots.activity_mut().push_temp(item);
            removed.push(item);
        }
    }
    removed.reverse();
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
    // hashing, and a successful lookup sends only the index comparison.
    if bag
        && let Some(index) = supplied_index
        && !super::collection::same_item(interp, wanted, index)?
    {
        return Ok(Some(ObjRef::NIL));
    }
    let under = if bag { Some(wanted) } else { supplied_index };
    let keys = keys_of(interp, receiver);
    let hash = under
        .map(|index| super::hash_of(interp, keys, index))
        .transpose()?;
    let store = store_of(interp, receiver)?;
    // The active lookup keeps its hash store through growth. These arrays are
    // actually read after callbacks, even if expansion detached them from the
    // receiver. EMPTY clears them in place.
    for array in [store.indexes, store.items, store.next] {
        interp.roots.activity_mut().push_temp(array);
    }
    let first = hash.map_or(0, |hash| (hash % store.buckets as u64) as usize);
    let end = if under.is_some() {
        first + 1
    } else {
        store.buckets
    };
    for bucket in first..end {
        let mut slot = bucket;
        while slot < store.total && slot_at(interp, store.indexes, slot)?.is_some() {
            let matches = match under {
                Some(index) => same_index(interp, &store, keys, slot, index)?,
                None => true,
            };
            if bag && matches {
                return Ok(Some(crate::eval::logical(true)));
            }
            // A callback may empty this location or replace its value.
            // Read the selected contents' current item only if it is live.
            if slot_at(interp, store.indexes, slot)?.is_none() {
                break;
            }
            if matches
                && let Some(item) = slot_at(interp, store.items, slot)?
                && super::collection::same_item(interp, wanted, item)?
            {
                return Ok(Some(crate::eval::logical(true)));
            }
            // A false comparison can replace the collision tail too.
            // Follow its current link, never a saved entry list or a free slot.
            if slot_at(interp, store.indexes, slot)?.is_none() {
                break;
            }
            slot = super::link_at(interp, &store, slot)?;
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
    let under = if bag { Some(wanted) } else { supplied_index };
    let keys = keys_of(interp, receiver);
    let hash = under
        .map(|index| super::hash_of(interp, keys, index))
        .transpose()?;
    'search: loop {
        let store = store_of(interp, receiver)?;
        // Keep the saved array identities live across replacement callbacks.
        for array in [store.indexes, store.items, store.next] {
            interp.roots.activity_mut().push_temp(array);
        }
        let selected = hash.map(|hash| (hash % store.buckets as u64) as usize);
        let mut bucket = selected.unwrap_or(0);
        for (slot, previous) in super::entries_in(interp, &store, selected)? {
            if previous.is_none() {
                bucket = slot;
            }
            if let Some(index) = under {
                let matches = same_index(interp, &store, keys, slot, index)?;
                if store_of(interp, receiver)? != store {
                    continue 'search;
                }
                if !matches {
                    continue;
                }
            }
            let Some(item) = slot_at(interp, store.items, slot)? else {
                continue;
            };
            // A Bag's item is its index, which already matched.
            if bag || super::collection::same_item(interp, wanted, item)? {
                if let Some(removed) = super::take_at(interp, receiver, &store, slot, bucket)? {
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
    // **`Relation` and `Bag` have identical native entry-point sets**, which
    // is why they are one task: the oracle writes `Bag`'s `AllAt`, `AllIndex`,
    // `Items`, `RemoveAll`, `Supplier` and `UniqueIndexes` with `Relation`'s
    // bodies, and only `HasItem` and `RemoveItem` are `Bag`'s own. A Bag's
    // index is its item, so HASITEM specializes the optional argument check
    // and index-only comparison in its shared body.
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
