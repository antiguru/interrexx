use super::*;

/// Takes the entry out, answering the item it held.
pub(super) fn take(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    take_in(interp, receiver, CONTENTS, index)
}

/// [`take`] from either half of a `Directory`.
pub(super) fn take_in(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    interp.bump_route_generation();
    let Some(store) = read_store(interp, receiver, half)? else {
        return Ok(None);
    };
    let (store, found) = probe_in(interp, receiver, half, store, index)?;
    let Some(position) = found.found() else {
        return Ok(None);
    };
    take_one(interp, receiver, half, store, position)
}

/// Removes an already compared entry if it is still live in its original
/// backing arrays, answering the value actually removed.
///
/// `removeItem` identifies an entry rather than an index because two entries
/// under one index can differ only by their item.
pub(super) fn take_at(
    interp: &mut Interp,
    receiver: ObjRef,
    store: &Store,
    position: usize,
) -> Result<Option<ObjRef>, Failure> {
    let current = store_of(interp, receiver)?;
    if !current.same_arrays(store) {
        return Ok(None);
    }
    // A callback can remove the selected entry. In the same arrays a live
    // position still holds the entry that was compared, since a position is
    // filled once.
    if slot_at(interp, current.indexes, position)?.is_none() {
        return Ok(None);
    }
    interp.bump_route_generation();
    take_one(interp, receiver, CONTENTS, current, position)
}

fn take_one(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    mut store: Store,
    position: usize,
) -> Result<Option<ObjRef>, Failure> {
    let item = remove_entry(interp, receiver, half, &mut store, position)?;
    compact_after_removal(interp, receiver, half, &store, item)?;
    Ok(item)
}

/// Removes the positions a search selected without compacting between them.
pub(super) fn take_positions(
    interp: &mut Interp,
    receiver: ObjRef,
    store: &Store,
    positions: Vec<usize>,
) -> Result<Vec<ObjRef>, Failure> {
    let mut current = store_of(interp, receiver)?;
    if !current.same_arrays(store) {
        return Ok(Vec::new());
    }
    let mut removed = Vec::new();
    for position in positions {
        if slot_at(interp, current.indexes, position)?.is_none() {
            continue;
        }
        interp.bump_route_generation();
        if let Some(item) = remove_entry(interp, receiver, CONTENTS, &mut current, position)? {
            interp.roots.activity_mut().push_temp(item);
            removed.push(item);
        }
    }
    compact_after_removal(interp, receiver, CONTENTS, &current, None)?;
    Ok(removed)
}

/// Whether a removal refuses an owed entry before taking it: `Answered` for
/// one that hands the item to the program unread, `Discarded` for one that
/// drops it or whose caller has already read it through [`item_at`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Removal {
    Answered,
    Discarded,
}

/// Answers what `get` would -- which may run a method, or the unknown method
/// -- and then drops the name from both halves.
///
/// An [`Removal::Answered`] removal refuses an owed entry; a discarded one
/// takes it without reading it.
pub(super) fn take_merged(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    removal: Removal,
) -> Result<Option<ObjRef>, Failure> {
    if method_store(interp, receiver)?.is_none() && unknown_method(interp, receiver).is_none() {
        interp.bump_route_generation();
        let (store, found) = probe(interp, receiver, index)?;
        let Some(position) = found.found() else {
            return Ok(None);
        };
        if removal == Removal::Answered {
            item_at(interp, &store, position)?;
        }
        return take_one(interp, receiver, CONTENTS, store, position);
    }
    // `get` runs a method only where the contents miss, so a contents entry
    // that is discarded needs no read.
    let old =
        if removal == Removal::Discarded && probe(interp, receiver, index)?.1.found().is_some() {
            None
        } else {
            merged_get(interp, receiver, index)?
        };
    take(interp, receiver, index)?;
    take_in(interp, receiver, METHODS, index)?;
    Ok(old)
}

/// Drops every entry and answers the receiver.
pub(super) fn native_hash_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    note_routines_write(interp, receiver);
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"EMPTY"));
    }
    clear(interp, receiver, CONTENTS)?;
    Ok(Some(receiver))
}

pub(super) fn native_hash_remove_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    note_routines_write(interp, receiver);
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"REMOVEITEM"));
    }
    let wanted = super::collection::item_argument(args)?;
    let keys = keys_of(interp, receiver);
    if method_store(interp, receiver)?.is_none() && unknown_method(interp, receiver).is_none() {
        let store = store_of(interp, receiver)?;
        // Keep the saved array identities live across replacement callbacks.
        for array in [store.indexes, store.items, store.codes, store.slots] {
            interp.roots.activity_mut().push_temp(array);
        }
        // One pass over the entries the search began with, each compared at
        // most once: a comparison that changes the receiver never starts the
        // search again, so the search ends whatever the comparisons do. A
        // match that a callback has removed or moved to new arrays is skipped
        // (`r[class.table.deviation.33]`).
        for position in entries(interp, &store)? {
            if slot_at(interp, store.indexes, position)?.is_none() {
                continue;
            }
            let Some(item) = item_at(interp, &store, position)? else {
                continue;
            };
            if same_held_item(interp, keys, wanted, item)?
                && let Some(removed) = take_at(interp, receiver, &store, position)?
            {
                return Ok(Some(removed));
            }
        }
        return Ok(Some(ObjRef::NIL));
    }
    let Some((index, item, from_method)) = search_item(interp, receiver, wanted)? else {
        return Ok(Some(ObjRef::NIL));
    };
    if from_method {
        // Removed as `remove` removes it: the method runs again and its
        // second answer is the answer.
        let answer = take_merged(interp, receiver, index, Removal::Answered)?;
        return Ok(Some(answer.unwrap_or(ObjRef::NIL)));
    }
    take_merged(interp, receiver, index, Removal::Discarded)?;
    Ok(Some(item))
}
