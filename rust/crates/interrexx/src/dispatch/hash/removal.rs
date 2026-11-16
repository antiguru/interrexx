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
    let Some(position) = found.found else {
        return Ok(None);
    };
    remove_entry(interp, &store, position)
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
    remove_entry(interp, &current, position)
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
        let Some(position) = found.found else {
            return Ok(None);
        };
        if removal == Removal::Answered {
            item_at(interp, &store, position)?;
        }
        return remove_entry(interp, &store, position);
    }
    // `get` runs a method only where the contents miss, so a contents entry
    // that is discarded needs no read.
    let old = if removal == Removal::Discarded && probe(interp, receiver, index)?.1.found.is_some()
    {
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
        'search: loop {
            let store = store_of(interp, receiver)?;
            // Keep the saved array identities live across replacement callbacks.
            for array in [store.indexes, store.items, store.codes, store.slots] {
                interp.roots.activity_mut().push_temp(array);
            }
            for position in entries(interp, &store)? {
                let Some(item) = item_at(interp, &store, position)? else {
                    continue;
                };
                if same_held_item(interp, keys, wanted, item)? {
                    if let Some(removed) = take_at(interp, receiver, &store, position)? {
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
