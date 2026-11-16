use super::*;

/// Where `index` is: the position of the oldest live entry under it, or,
/// when there is none, where the chain of its code was when the probe last
/// walked the slots of the store it answers with; and the index's code,
/// which an append files a new entry under. An append that follows with
/// nothing sent in between starts from that place rather than walking the
/// slots again.
pub(super) struct Probe {
    pub(super) at: Result<usize, Place>,
    pub(super) code: u64,
}

impl Probe {
    /// The position of the oldest live entry under the index, if any.
    #[inline]
    pub(super) fn found(&self) -> Option<usize> {
        self.at.ok()
    }
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
        let code = code_of(super::super::string_hash(interp.hash_seed, wanted));
        if let Some(at) = find_plain_string(interp, &store, code, wanted) {
            return Ok((store, Probe { at, code }));
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
    let code = code_of(super::super::string_hash(interp.hash_seed, wanted));
    let at = find_text_in(interp, &store, code, wanted)?;
    Ok((store, Probe { at, code }))
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
    Ok(find_text_in(interp, store, code, wanted)?.ok())
}

/// [`find_text`], or where the chain of `code` is, as [`Probe::at`].
fn find_text_in(
    interp: &mut Interp,
    store: &Store,
    code: u64,
    wanted: &[u8],
) -> Result<Result<usize, Place>, Failure> {
    if let Some(found) = find_plain_string(interp, store, code, wanted) {
        return Ok(found);
    }
    let (slots, codes) = codes_of(interp, store)?;
    let chain = chain_in(slots, codes, code);
    let place = Place::of(chain, codes);
    let mut at = chain.ok().map(|(_, head)| head);
    while let Some(position) = at {
        if let Some(held) = slot_at(interp, store.indexes, position)?
            && *interp.to_text(held) == *wanted
        {
            return Ok(Ok(position));
        }
        at = codes_of(interp, store)?.1.next(position);
    }
    Ok(Err(place))
}

/// What a lookup knows once one of its comparisons has written a store: the
/// candidates the chain of the index's code held then, and the answer of each
/// comparison sent since. From then on each of those candidates is compared at
/// most once and an entry put after is not compared, so the lookup ends
/// whatever the comparisons do.
struct Guard {
    candidates: Vec<ObjRef>,
    compared: Vec<(ObjRef, bool)>,
}

/// What a [`Guard`] says of one candidate.
enum Seen {
    /// Compared since the guard began, with this answer.
    Compared(bool),
    /// In the chain when the guard began, and not compared since.
    Pending,
    /// Put after the guard began.
    Unseen,
}

impl Guard {
    /// The guard of a lookup whose comparison of `held` answered `matches`
    /// and wrote a store, with `store` the receiver's store after it.
    fn began(
        interp: &mut Interp,
        store: &Store,
        code: u64,
        held: ObjRef,
        matches: bool,
    ) -> Result<Guard, Failure> {
        let mut candidates = Vec::new();
        {
            let (slots, codes) = codes_of(interp, store)?;
            let indexes = array_slots(interp, store.indexes)?;
            if let Ok((_, head)) = chain_in(slots, codes, code) {
                for position in codes.chain(head) {
                    if let Some(candidate) = indexes.get(position).copied().flatten() {
                        candidates.push(candidate);
                    }
                }
            }
        }
        // A candidate a later comparison removes must not be swept and its
        // handle reused by an entry put after. The temporaries stay rooted
        // until the clause that sent the lookup ends, whose frame
        // (`Interp::enter_stepped_clause`) truncates them.
        interp.roots.activity_mut().push_temp(held);
        for &candidate in &candidates {
            interp.roots.activity_mut().push_temp(candidate);
        }
        Ok(Guard {
            candidates,
            compared: vec![(held, matches)],
        })
    }

    fn seen(&self, held: ObjRef) -> Seen {
        if let Some(&(_, matches)) = self.compared.iter().find(|(object, _)| *object == held) {
            Seen::Compared(matches)
        } else if self.candidates.contains(&held) {
            Seen::Pending
        } else {
            Seen::Unseen
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
    let code = code_of(hash_of(interp, keys, index)?);
    if generation != interp.store_generation {
        store = store_in(interp, receiver, half)?;
    }
    // None until a comparison writes a store, so a lookup whose comparisons
    // write nothing keeps nothing beside its walk.
    let mut guard: Option<Guard> = None;
    'search: loop {
        // A differing cached code is a different index for every key that
        // keeps the hash contract, and nothing is sent to it: only the chain
        // of the index's own code is compared.
        let (slots, codes) = codes_of(interp, &store)?;
        let chain = chain_in(slots, codes, code);
        let place = Place::of(chain, codes);
        let mut at = chain.ok().map(|(_, head)| head);
        while let Some(position) = at {
            let Some(held) = slot_at(interp, store.indexes, position)? else {
                at = codes_of(interp, &store)?.1.next(position);
                continue;
            };
            let known = match guard.as_ref().map(|guard| guard.seen(held)) {
                None | Some(Seen::Pending) => None,
                Some(Seen::Compared(matches)) => Some(matches),
                Some(Seen::Unseen) => {
                    at = codes_of(interp, &store)?.1.next(position);
                    continue;
                }
            };
            let matches = match known {
                Some(matches) => matches,
                None => {
                    let generation = interp.store_generation;
                    let writes = interp.store_writes;
                    let matches = same_index(interp, &store, keys, position, index)?;
                    if let Some(guard) = guard.as_mut() {
                        guard.compared.push((held, matches));
                    }
                    if generation != interp.store_generation || writes != interp.store_writes {
                        // The comparison ran code that wrote a store. New
                        // arrays, or the compared entry gone, start the walk
                        // again. In the same arrays a position keeps its entry
                        // while it is live, so the answer stands.
                        let current = store_in(interp, receiver, half)?;
                        if guard.is_none() {
                            guard = Some(Guard::began(interp, &current, code, held, matches)?);
                        }
                        if current != store
                            || slot_at(interp, current.indexes, position)? != Some(held)
                        {
                            store = current;
                            continue 'search;
                        }
                    }
                    matches
                }
            };
            if matches {
                return Ok((
                    store,
                    Probe {
                        at: Ok(position),
                        code,
                    },
                ));
            }
            // Read after the comparison: a removal it made relinked the chain.
            at = codes_of(interp, &store)?.1.next(position);
        }
        return Ok((
            store,
            Probe {
                at: Err(place),
                code,
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
) -> Option<Result<usize, Place>> {
    let (slots, codes) = codes_of(interp, store).ok()?;
    let indexes = array_slots(interp, store.indexes).ok()?;
    let chain = chain_in(slots, codes, code);
    let Ok((_, head)) = chain else {
        return Some(Err(Place::of(chain, codes)));
    };
    for position in codes.chain(head) {
        if let Some(held) = indexes.get(position).copied().flatten()
            && plain_text_is(interp, held, wanted)?
        {
            return Some(Ok(position));
        }
    }
    Some(Err(Place::of(chain, codes)))
}

/// Whether `held`'s string value is `wanted`, for a plain string only: the
/// bytes in the handle, or a `Body::Text`'s own, which no redirect applies
/// to. `None` for any other value.
#[inline]
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
