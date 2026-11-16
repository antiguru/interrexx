use super::*;

/// Where `index` is: the position of the oldest live entry under it, if any,
/// and the index's code, which an append files a new entry under.
pub(super) struct Probe {
    pub(super) found: Option<usize>,
    pub(super) code: u64,
    /// Where the chain of `code` was when the probe last walked the slots of
    /// the store it answers with: the chain's slot and oldest entry, or the
    /// slot a new chain takes. An append that follows with nothing sent in
    /// between starts from it rather than walking the slots again.
    pub(super) chain: Result<(usize, usize), usize>,
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
    if let Some(wanted) = plain {
        let code = code_of(super::super::string_hash(wanted));
        if let Some((found, chain)) = find_plain_string(interp, &store, code, wanted) {
            return Ok((store, Probe { found, code, chain }));
        }
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
    let code = code_of(super::super::string_hash(wanted));
    let (found, chain) = find_text_in(interp, &store, code, wanted)?;
    Ok((store, Probe { found, code, chain }))
}

/// The position of the entry whose index's string value is `wanted`, in a
/// string-keyed store, looked up by `code` -- `wanted`'s string hash's
/// [`code_of`] -- without building an index object.
pub(super) fn find_text(
    interp: &mut Interp,
    store: &Store,
    code: u64,
    wanted: &[u8],
) -> Result<Option<usize>, Failure> {
    Ok(find_text_in(interp, store, code, wanted)?.0)
}

/// [`find_text`], and where the chain of `code` is, as [`Probe::chain`].
fn find_text_in(
    interp: &mut Interp,
    store: &Store,
    code: u64,
    wanted: &[u8],
) -> Result<(Option<usize>, Result<(usize, usize), usize>), Failure> {
    if let Some(found) = find_plain_string(interp, store, code, wanted) {
        return Ok(found);
    }
    let (slots, codes) = codes_of(interp, store)?;
    let chain = chain_in(slots, codes, code);
    let mut at = chain.ok().map(|(_, head)| head);
    while let Some(position) = at {
        if let Some(held) = slot_at(interp, store.indexes, position)?
            && *interp.to_text(held) == *wanted
        {
            return Ok((Some(position), chain));
        }
        at = codes_of(interp, store)?.1.next(position);
    }
    Ok((None, chain))
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
    let code = code_of(hash_of(interp, keys, index)?);
    if generation != interp.store_generation {
        store = store_in(interp, receiver, half)?;
    }
    'search: loop {
        // A differing cached code is a different index for every key that
        // keeps the hash contract, and nothing is sent to it: only the chain
        // of the index's own code is compared.
        let (slots, codes) = codes_of(interp, &store)?;
        let chain = chain_in(slots, codes, code);
        let mut at = chain.ok().map(|(_, head)| head);
        while let Some(position) = at {
            let Some(held) = slot_at(interp, store.indexes, position)? else {
                at = codes_of(interp, &store)?.1.next(position);
                continue;
            };
            let generation = interp.store_generation;
            let writes = interp.store_writes;
            let matches = same_index(interp, &store, keys, position, index)?;
            if generation != interp.store_generation || writes != interp.store_writes {
                // The comparison ran code that wrote a store. New arrays, or a
                // new entry, start the search again; in the same arrays a
                // position keeps its entry while it is live, so the answer
                // stands unless the compared entry has gone.
                let current = store_in(interp, receiver, half)?;
                if current != store || slot_at(interp, current.indexes, position)? != Some(held) {
                    store = current;
                    continue 'search;
                }
            }
            if matches {
                return Ok((
                    store,
                    Probe {
                        found: Some(position),
                        code,
                        chain,
                    },
                ));
            }
            // Read after the comparison: a removal it made relinked the chain.
            at = codes_of(interp, &store)?.1.next(position);
        }
        return Ok((
            store,
            Probe {
                found: None,
                code,
                chain,
            },
        ));
    }
}

/// [`find_text`]'s chain walk with the arrays borrowed once, for a plain
/// string `wanted` and candidates whose every index is one too: inline or a
/// `Body::Text`, whose bytes are what `to_text` would answer. `None`, and the
/// one-by-one walk instead, for anything else -- a candidate index of another
/// kind, or an array that is not one, which that walk refuses.
fn find_plain_string(
    interp: &Interp,
    store: &Store,
    code: u64,
    wanted: &[u8],
) -> Option<(Option<usize>, Result<(usize, usize), usize>)> {
    let (slots, codes) = codes_of(interp, store).ok()?;
    let indexes = array_slots(interp, store.indexes).ok()?;
    let chain = chain_in(slots, codes, code);
    let Ok((_, head)) = chain else {
        return Some((None, chain));
    };
    for position in codes.chain(head) {
        if let Some(held) = indexes.get(position).copied().flatten()
            && plain_text_is(interp, held, wanted)?
        {
            return Some((Some(position), chain));
        }
    }
    Some((None, chain))
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
