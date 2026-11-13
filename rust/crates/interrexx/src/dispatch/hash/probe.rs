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
    let plain: Option<&[u8]> = match index.decode() {
        Decoded::Text(text) => {
            inline = text;
            Some(&inline)
        }
        Decoded::Heap { .. } => match interp.heap.get(index).map(|object| &object.body) {
            Some(Body::Text { bytes, .. }) => Some(bytes.as_slice()),
            _ => None,
        },
        _ => None,
    };
    if let Some(wanted) = plain
        && let Some(probe) = probe_plain_strings(interp, &store, wanted)
    {
        return Ok((store, probe));
    }
    let short;
    let owned;
    let wanted: &[u8] = match index.decode() {
        Decoded::Text(text) => {
            short = text;
            &short
        }
        _ => {
            owned = interp.to_text(index).into_owned();
            &owned
        }
    };
    let bucket = bucket_of(&store, wanted);
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

/// The bucket a string-keyed store files `wanted` under.
fn bucket_of(store: &Store, wanted: &[u8]) -> usize {
    (super::super::string_hash(wanted) % store.buckets as u64) as usize
}

/// [`probe_string_value`]'s chain walk with both arrays borrowed once, for a
/// plain string `wanted` and a chain whose every index is one too: inline or
/// a `Body::Text`, whose bytes are what `to_text` would answer. `None`, and
/// the slot-by-slot walk instead, for anything else -- an index of another
/// kind on the chain, or an array that is not one, which that walk refuses.
fn probe_plain_strings(interp: &Interp, store: &Store, wanted: &[u8]) -> Option<Probe> {
    let bucket = bucket_of(store, wanted);
    let indexes = super::array_slots(interp, store.indexes).ok()?;
    let index_at = |slot: usize| indexes.get(slot).copied().flatten();
    if index_at(bucket).is_none() {
        return Some(Probe {
            bucket,
            found: None,
            last: None,
        });
    }
    let links = super::array_slots(interp, store.next).ok()?;
    let mut slot = bucket;
    let mut last = bucket;
    loop {
        let matches = match index_at(slot) {
            Some(held) => plain_text_is(interp, held, wanted)?,
            None => false,
        };
        if matches {
            return Some(Probe {
                bucket,
                found: Some(slot),
                last: Some(last),
            });
        }
        last = slot;
        slot = match links.get(slot).copied().flatten().map(ObjRef::decode) {
            Some(Decoded::SmallInt(link)) if link >= 0 => link as usize,
            _ => store.no_more(),
        };
        if slot >= store.total {
            return Some(Probe {
                bucket,
                found: None,
                last: Some(last),
            });
        }
    }
}

/// Whether `held`'s string value is `wanted`, for a plain string only: the
/// bytes in the handle, or a `Body::Text`'s own, which no redirect applies
/// to. `None` for any other value.
fn plain_text_is(interp: &Interp, held: ObjRef, wanted: &[u8]) -> Option<bool> {
    match held.decode() {
        Decoded::Text(text) => Some(*text == *wanted),
        Decoded::Heap { .. } => match &interp.heap.get(held)?.body {
            Body::Text { bytes, .. } => Some(bytes.as_slice() == wanted),
            _ => None,
        },
        _ => None,
    }
}
