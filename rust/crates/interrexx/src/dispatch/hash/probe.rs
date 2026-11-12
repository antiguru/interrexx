use super::*;

/// Where `index` belongs, and where it is: the bucket, the slot holding it if
/// any, and the last slot on that bucket's chain.
pub(super) struct Probe {
    pub(super) bucket: usize,
    pub(super) found: Option<usize>,
    pub(super) last: Option<usize>,
}

pub(super) fn probe(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<(Store, Probe), Failure> {
    let store = store_of(interp, receiver)?;
    probe_in(interp, receiver, CONTENTS, store, index)
}

/// [`probe`] against a store the caller has already read, so that a
/// `Directory`'s method table can be searched with the same chain walk.
#[inline]
pub(super) fn probe_in(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    store: Store,
    index: ObjRef,
) -> Result<(Store, Probe), Failure> {
    let keys = keys_of(interp, receiver);
    // String-value hash and index comparison never send, so that protocol
    // has a traversal of its own that carries no callback restart state.
    match keys {
        Keys::StringValue => probe_string_value(interp, store, index),
        _ => probe_by_protocol(interp, receiver, half, store, index, keys),
    }
}

/// [`probe_in`] for the string-value protocol, which never sends: the index's
/// string value is read once, for the hash and for every comparison, rather
/// than copied out again for each.
fn probe_string_value(
    interp: &mut Interp,
    store: Store,
    index: ObjRef,
) -> Result<(Store, Probe), Failure> {
    let inline;
    let owned;
    let wanted: &[u8] = match index.decode() {
        Decoded::Text(text) => {
            inline = text;
            &inline
        }
        _ => {
            owned = interp.to_text(index).into_owned();
            &owned
        }
    };
    let hash = super::super::string_hash(wanted);
    let bucket = (hash % store.buckets as u64) as usize;
    if slot_at(interp, store.indexes, bucket)?.is_none() {
        return Ok((
            store,
            Probe {
                bucket,
                found: None,
                last: None,
            },
        ));
    }
    let mut slot = bucket;
    let mut last = bucket;
    loop {
        let matches = match slot_at(interp, store.indexes, slot)? {
            Some(held) => *interp.to_text(held) == *wanted,
            None => false,
        };
        if matches {
            return Ok((
                store,
                Probe {
                    bucket,
                    found: Some(slot),
                    last: Some(last),
                },
            ));
        }
        last = slot;
        slot = link_at(interp, &store, slot)?;
        if slot >= store.total {
            return Ok((
                store,
                Probe {
                    bucket,
                    found: None,
                    last: Some(last),
                },
            ));
        }
    }
}

fn probe_by_protocol(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    mut store: Store,
    index: ObjRef,
    keys: Keys,
) -> Result<(Store, Probe), Failure> {
    let generation = interp.store_generation;
    let hash = hash_of(interp, keys, index)?;
    if generation != interp.store_generation {
        store = store_in(interp, receiver, half)?;
    }
    'search: loop {
        let bucket = (hash % store.buckets as u64) as usize;
        if slot_at(interp, store.indexes, bucket)?.is_none() {
            return Ok((
                store,
                Probe {
                    bucket,
                    found: None,
                    last: None,
                },
            ));
        }
        let mut slot = bucket;
        let mut last = bucket;
        loop {
            let generation = interp.store_generation;
            let matches = same_index(interp, &store, keys, slot, index)?;
            if generation != interp.store_generation {
                let current = store_in(interp, receiver, half)?;
                if current != store {
                    store = current;
                    continue 'search;
                }
            }
            if matches {
                return Ok((
                    store,
                    Probe {
                        bucket,
                        found: Some(slot),
                        last: Some(last),
                    },
                ));
            }
            last = slot;
            slot = link_at(interp, &store, slot)?;
            if slot >= store.total {
                return Ok((
                    store,
                    Probe {
                        bucket,
                        found: None,
                        last: Some(last),
                    },
                ));
            }
        }
    }
}
