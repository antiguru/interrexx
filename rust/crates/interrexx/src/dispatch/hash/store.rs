//! The hash store's layout in its receiver's pool, and the primitives that
//! read and write it.
//!
//! A store is a compact table. Its entries sit in dense arrays in the order
//! they were added, and a sparse slot index, open-addressed, maps a hash to
//! the entries filed under it. A new entry goes at the end of the entry
//! arrays; a removed one leaves a hole there and a deleted marker in its slot.
//! When the entry arrays fill, the store is rebuilt into fresh arrays that
//! keep the live entries in their order and re-slot them from each entry's
//! cached hash, so a rebuild sends nothing. Iteration walks the entry arrays,
//! which is what makes every enumeration answer insertion order.
//!
//! **Within one set of arrays, a position holds at most one entry in its
//! life**: entries are only appended, a hole is never filled again, and every
//! operation that would start over at position zero (a rebuild, `EMPTY`)
//! installs fresh arrays. A caller that kept a position across a send can
//! therefore trust it while the arrays are the same and the position is live.

use interrexx_core::SMALL_INT_MAX;

use super::*;

/// The five pool entries a hash store is, bound in the receiver's own pool
/// under the `MapCollection` class as scope: the entries' indexes, their
/// items and their cached hashes ([`code_of`], `None` for a hole), all in
/// insertion order; the slot index ([`EMPTY`], [`DELETED`], or an entry's
/// position plus one); and how many entry positions are used, holes included.
pub(super) const HASH_INDEXES: &[u8] = b"HASHINDEXES";
pub(super) const HASH_ITEMS: &[u8] = b"HASHITEMS";
pub(super) const HASH_NEXT: &[u8] = b"HASHNEXT";
pub(super) const HASH_BUCKETS: &[u8] = b"HASHBUCKETS";
pub(super) const HASH_FREE: &[u8] = b"HASHFREE";

/// The five pool entries one store occupies.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Half {
    pub(super) indexes: &'static [u8],
    pub(super) items: &'static [u8],
    pub(super) codes: &'static [u8],
    pub(super) slots: &'static [u8],
    pub(super) used: &'static [u8],
}

/// The store every mapped collection has.
pub(super) const CONTENTS: Half = Half {
    indexes: HASH_INDEXES,
    items: HASH_ITEMS,
    codes: HASH_NEXT,
    slots: HASH_BUCKETS,
    used: HASH_FREE,
};

pub(super) const METHOD_INDEXES: &[u8] = b"METHODINDEXES";
pub(super) const METHOD_ITEMS: &[u8] = b"METHODITEMS";
pub(super) const METHOD_NEXT: &[u8] = b"METHODNEXT";
pub(super) const METHOD_BUCKETS: &[u8] = b"METHODBUCKETS";
pub(super) const METHOD_FREE: &[u8] = b"METHODFREE";

/// The method table, which only a `Directory` ever installs.
pub(super) const METHODS: Half = Half {
    indexes: METHOD_INDEXES,
    items: METHOD_ITEMS,
    codes: METHOD_NEXT,
    slots: METHOD_BUCKETS,
    used: METHOD_FREE,
};

/// The one method `setMethod` keeps out of the method table, because `UNKNOWN`
/// is not an entry -- it is what answers when there is none. Measured: with it
/// set, `items` is still 1 for a directory holding one ordinary entry and
/// `allIndexes` does not name it.
pub(super) const UNKNOWN_METHOD: &[u8] = b"METHODUNKNOWN";

/// A slot no entry has been filed in since the slots were built: a probe
/// that reaches one has seen every entry of its hash.
const EMPTY: i64 = 0;
/// A slot whose entry was removed: a probe passes over it, and an append
/// does not reuse it, so the entries of one hash meet a probe oldest first.
const DELETED: i64 = -1;

/// The fewest slots a store has.
const MINIMUM_SLOTS: usize = 8;

/// The most slots a store is sized at.
const MAXIMUM_SLOTS: usize = 1 << 31;

/// The entry capacity of a store of `slots` slots: two thirds, so that the
/// slots, which hold one mark per used entry position, never fill and every
/// probe ends at an empty one.
fn capacity_of(slots: usize) -> usize {
    slots * 2 / 3
}

/// The slot count of a store sized to hold `entries` entries.
fn slots_for(entries: usize) -> usize {
    let mut slots = MINIMUM_SLOTS;
    while capacity_of(slots) < entries && slots < MAXIMUM_SLOTS {
        slots *= 2;
    }
    slots
}

/// The capacity a store is built at when nothing asks for more, which
/// [`slots_for`] gives the fewest slots.
pub(super) const MINIMUM_ENTRIES: usize = 0;

/// The capacity a rebuild gives a store holding `live` entries: room for as
/// many appends again before the next rebuild. Sized by the live entries
/// alone, so a store that churns through few of them stays at the minimum
/// however many it has removed.
fn regrown(live: usize) -> usize {
    live.saturating_mul(2)
}

/// The form a hash takes in an entry's cache and in the probe sequence:
/// mixed, so that the low bits that pick the first slot depend on every bit
/// of the hash, and cut to what a SmallInt holds. Equal hashes give equal
/// codes, which is all a comparison of codes relies on.
pub(super) fn code_of(hash: u64) -> u64 {
    let mixed = hash.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (mixed ^ (mixed >> 29)) & SMALL_INT_MAX as u64
}

/// The slots a code probes, in order: a perturbed sequence over a power of
/// two slots, which takes every bit of the code into account and, once the
/// perturbation has shifted out, visits every slot.
#[derive(Clone, Copy)]
pub(super) struct Sequence {
    mask: usize,
    at: usize,
    perturb: u64,
}

impl Sequence {
    pub(super) fn new(code: u64, slots: usize) -> Sequence {
        let mask = slots - 1;
        Sequence {
            mask,
            at: code as usize & mask,
            perturb: code,
        }
    }

    /// The slot this step of the sequence reads.
    pub(super) fn at(&self) -> usize {
        self.at
    }

    pub(super) fn advance(&mut self) {
        self.perturb >>= 5;
        self.at = self
            .at
            .wrapping_mul(5)
            .wrapping_add(1)
            .wrapping_add(self.perturb as usize)
            & self.mask;
    }
}

/// What one slot holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Mark {
    Empty,
    Deleted,
    Entry(usize),
}

/// The mark a slot array holds at `slot`. A slot array that is not one
/// reads as empty everywhere.
pub(super) fn mark_in(slots: &[Option<ObjRef>], slot: usize) -> Mark {
    match slots.get(slot).copied().flatten().map(ObjRef::decode) {
        Some(Decoded::SmallInt(DELETED)) => Mark::Deleted,
        Some(Decoded::SmallInt(value)) if value > EMPTY => Mark::Entry(value as usize - 1),
        _ => Mark::Empty,
    }
}

/// The code cached for the entry at a position, `None` for a hole.
pub(super) fn code_in(codes: &[Option<ObjRef>], position: usize) -> Option<u64> {
    match codes.get(position).copied().flatten().map(ObjRef::decode) {
        Some(Decoded::SmallInt(code)) if code >= 0 => Some(code as u64),
        _ => None,
    }
}

/// A SmallInt handle for a value known to be in range: a code, a mark or a
/// count no larger than an array.
fn small(value: i64) -> ObjRef {
    ObjRef::small_int(value).expect("a hash store's numbers fit a SmallInt")
}

/// A hash store's four arrays and its count, read out together.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct Store {
    pub(super) indexes: ObjRef,
    pub(super) items: ObjRef,
    pub(super) codes: ObjRef,
    pub(super) slots: ObjRef,
    /// Entry positions used, holes included: where the next entry goes.
    pub(super) used: usize,
    /// Entry positions the arrays have.
    pub(super) capacity: usize,
}

impl Store {
    /// Whether `other` is these same arrays, whatever their count.
    pub(super) fn same_arrays(&self, other: &Store) -> bool {
        self.indexes == other.indexes
            && self.items == other.items
            && self.codes == other.codes
            && self.slots == other.slots
    }
}

pub(super) fn pool_variable(
    interp: &Interp,
    owner: ObjRef,
    scope: ObjRef,
    name: &[u8],
) -> Option<ObjRef> {
    match interp.heap.get(owner).map(|object| &object.body) {
        Some(Body::Instance { pools, .. }) => pools.get(scope, name),
        _ => None,
    }
}

pub(super) fn counted(value: ObjRef) -> Option<usize> {
    match value.decode() {
        Decoded::SmallInt(value) if value >= 0 => Some(value as usize),
        _ => None,
    }
}

/// The five names of each half, as the pool keeps them, made once.
pub(crate) struct PoolNames {
    contents: [Rc<[u8]>; 5],
    methods: [Rc<[u8]>; 5],
}

impl Default for PoolNames {
    fn default() -> PoolNames {
        let names = |half: Half| {
            [half.indexes, half.items, half.codes, half.slots, half.used].map(Rc::from)
        };
        PoolNames {
            contents: names(CONTENTS),
            methods: names(METHODS),
        }
    }
}

/// Builds an empty store sized for `capacity` entries, and binds it to
/// `receiver`.
pub(super) fn install_store(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    capacity: usize,
) -> Store {
    build(interp, receiver, half, &[], capacity)
}

/// Builds a store holding `entries` -- (index, item, code) in their order --
/// sized for at least `capacity`, and binds it to `receiver` in place of
/// whatever it had.
fn build(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    entries: &[(ObjRef, Option<ObjRef>, u64)],
    capacity: usize,
) -> Store {
    let slot_count = slots_for(capacity.max(entries.len()));
    let capacity = capacity_of(slot_count);
    let scope = hash_scope(interp);
    let mut index_slots = vec![None; capacity];
    let mut item_slots = vec![None; capacity];
    let mut code_slots = vec![None; capacity];
    let mut marks = vec![Some(small(EMPTY)); slot_count];
    for (position, &(index, item, code)) in entries.iter().enumerate() {
        index_slots[position] = Some(index);
        item_slots[position] = item;
        code_slots[position] = Some(small(code as i64));
        let mut sequence = Sequence::new(code, slot_count);
        while mark_in(&marks, sequence.at()) != Mark::Empty {
            sequence.advance();
        }
        marks[sequence.at()] = Some(small(position as i64 + 1));
    }
    // Each array is rooted as soon as it exists: the next allocation may
    // collect. The entries copied in stay reachable through the arrays still
    // bound until the new ones replace them.
    let indexes = interp.alloc_charged(BehaviourId::ARRAY, Body::array(index_slots));
    interp.roots.activity_mut().push_temp(indexes);
    let items = interp.alloc_charged(BehaviourId::ARRAY, Body::array(item_slots));
    interp.roots.activity_mut().push_temp(items);
    let codes = interp.alloc_charged(BehaviourId::ARRAY, Body::array(code_slots));
    interp.roots.activity_mut().push_temp(codes);
    let slots = interp.alloc_charged(BehaviourId::ARRAY, Body::array(marks));
    interp.roots.activity_mut().push_temp(slots);
    let used = entries.len();
    let used_value = small(used as i64);
    let names = if half == CONTENTS {
        &interp.hash_names.contents
    } else {
        &interp.hash_names.methods
    };
    let [n_indexes, n_items, n_codes, n_slots, n_used] = names.clone();
    // A capacity [`initialize_store`] left pending goes, so that the five
    // entries are bound together in their order, as [`shared_run`] reads them.
    if pool_variable(interp, receiver, scope, half.indexes).is_none()
        && pool_variable(interp, receiver, scope, half.slots).is_some()
    {
        interp.clear_pool_variable(receiver, scope, half.slots);
    }
    interp.set_pool_variables(
        receiver,
        scope,
        &[
            (&n_indexes, indexes),
            (&n_items, items),
            (&n_codes, codes),
            (&n_slots, slots),
            (&n_used, used_value),
        ],
    );
    bump_store_generation(interp);
    Store {
        indexes,
        items,
        codes,
        slots,
        used,
        capacity,
    }
}

/// The receiver's store, built at the minimum size on demand.
pub(super) fn read_store(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
) -> Result<Option<Store>, Failure> {
    let parts = read_parts(interp, receiver);
    let found = if half == CONTENTS {
        parts.contents
    } else {
        parts.methods
    };
    found.store(interp)
}

/// One half's pool entries, as [`read_parts`] found them.
#[derive(Clone, Copy, Default)]
pub(super) struct Found {
    pub(super) indexes: Option<ObjRef>,
    pub(super) items: Option<ObjRef>,
    pub(super) codes: Option<ObjRef>,
    /// The slot array, or before the store is built the capacity
    /// [`keep_capacity`] left for it.
    pub(super) slots: Option<ObjRef>,
    pub(super) used: Option<usize>,
}

impl Found {
    /// The store these entries make, or `None` when any is missing.
    pub(super) fn store(self, interp: &Interp) -> Result<Option<Store>, Failure> {
        let (Some(indexes), Some(items), Some(codes), Some(slots), Some(used)) =
            (self.indexes, self.items, self.codes, self.slots, self.used)
        else {
            return Ok(None);
        };
        let capacity = array_slots(interp, indexes)?.len();
        Ok(Some(Store {
            indexes,
            items,
            codes,
            slots,
            used,
            capacity,
        }))
    }

    /// The capacity a store not built yet is to be built at, if one was
    /// kept.
    pub(super) fn kept_capacity(self) -> Option<usize> {
        self.slots.and_then(counted)
    }
}

/// Both halves' entries and the unknown method, from one pass over the pool.
pub(super) struct Parts {
    pub(super) contents: Found,
    pub(super) methods: Found,
    pub(super) unknown: Option<ObjRef>,
}

pub(super) fn read_parts(interp: &mut Interp, receiver: ObjRef) -> Parts {
    let scope = hash_scope(interp);
    let mut parts = Parts {
        contents: Found::default(),
        methods: Found::default(),
        unknown: None,
    };
    let Some(Body::Instance { pools, .. }) = interp.heap.get(receiver).map(|object| &object.body)
    else {
        return parts;
    };
    let names = &interp.hash_names;
    let mut rest = pools.entries(scope);
    while !rest.is_empty() {
        // [`install_store`] binds a half's five names together, in this order
        // and as copies of [`PoolNames`]' own, so a run of them is recognised
        // by identity before any name is compared byte by byte.
        if let Some(found) = shared_run(rest, &names.contents) {
            parts.contents = found;
            rest = &rest[5..];
            continue;
        }
        if let Some(found) = shared_run(rest, &names.methods) {
            parts.methods = found;
            rest = &rest[5..];
            continue;
        }
        let (name, value) = &rest[0];
        rest = &rest[1..];
        let value = *value;
        match name.as_ref() {
            HASH_INDEXES => parts.contents.indexes = Some(value),
            HASH_ITEMS => parts.contents.items = Some(value),
            HASH_NEXT => parts.contents.codes = Some(value),
            HASH_BUCKETS => parts.contents.slots = Some(value),
            HASH_FREE => parts.contents.used = counted(value),
            METHOD_INDEXES => parts.methods.indexes = Some(value),
            METHOD_ITEMS => parts.methods.items = Some(value),
            METHOD_NEXT => parts.methods.codes = Some(value),
            METHOD_BUCKETS => parts.methods.slots = Some(value),
            METHOD_FREE => parts.methods.used = counted(value),
            UNKNOWN_METHOD => parts.unknown = Some(value).filter(|held| *held != ObjRef::NIL),
            _ => {}
        }
    }
    parts
}

/// The contents' entries when they are the first the pool binds, which is
/// where [`install_store`] puts them in a store built before anything else
/// is bound: [`read_parts`]'s first step, without the rest of its pass.
#[inline]
pub(super) fn leading_contents(interp: &mut Interp, receiver: ObjRef) -> Option<Found> {
    let scope = hash_scope(interp);
    let Some(Body::Instance { pools, .. }) = interp.heap.get(receiver).map(|object| &object.body)
    else {
        return None;
    };
    shared_run(pools.entries(scope), &interp.hash_names.contents)
}

/// One half's entries, when `entries` starts with that half's five shared
/// names in [`install_store`]'s order. A pool binds a name once, so what this
/// finds is what the byte-by-byte reading would.
#[inline]
pub(super) fn shared_run(entries: &[(Rc<[u8]>, ObjRef)], names: &[Rc<[u8]>; 5]) -> Option<Found> {
    let [
        (indexes_name, indexes),
        (items_name, items),
        (codes_name, codes),
        (slots_name, slots),
        (used_name, used),
        ..,
    ] = entries
    else {
        return None;
    };
    let [indexes_at, items_at, codes_at, slots_at, used_at] = names;
    (Rc::ptr_eq(indexes_name, indexes_at)
        && Rc::ptr_eq(items_name, items_at)
        && Rc::ptr_eq(codes_name, codes_at)
        && Rc::ptr_eq(slots_name, slots_at)
        && Rc::ptr_eq(used_name, used_at))
    .then(|| Found {
        indexes: Some(*indexes),
        items: Some(*items),
        codes: Some(*codes),
        slots: Some(*slots),
        used: counted(*used),
    })
}

/// The store `half` names, installing an empty one if it is not there yet.
pub(super) fn store_in(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
) -> Result<Store, Failure> {
    if half == CONTENTS {
        return store_of(interp, receiver);
    }
    match read_store(interp, receiver, half)? {
        Some(store) => Ok(store),
        None => Ok(install_store(interp, receiver, half, MINIMUM_ENTRIES)),
    }
}

pub(super) fn store_of(interp: &mut Interp, receiver: ObjRef) -> Result<Store, Failure> {
    if let Some(found) = leading_contents(interp, receiver)
        && let Some(store) = found.store(interp)?
    {
        return Ok(store);
    }
    let parts = read_parts(interp, receiver);
    if let Some(store) = parts.contents.store(interp)? {
        return Ok(store);
    }
    refuse_storeless(interp, receiver)?;
    // The capacity [`initialize_store`] left pending, if it left one.
    let capacity = parts.contents.kept_capacity().unwrap_or(MINIMUM_ENTRIES);
    Ok(install_store(interp, receiver, CONTENTS, capacity))
}

/// The contents' store, or `None` for a receiver [`store_of`] would build an
/// empty one for: what a read that finds nothing in an empty store answers
/// the same without building it. Refuses as [`store_of`] does.
pub(super) fn store_if_built(
    interp: &mut Interp,
    receiver: ObjRef,
) -> Result<Option<Store>, Failure> {
    if let Some(found) = leading_contents(interp, receiver)
        && let Some(store) = found.store(interp)?
    {
        return Ok(Some(store));
    }
    let parts = read_parts(interp, receiver);
    if let Some(store) = parts.contents.store(interp)? {
        return Ok(Some(store));
    }
    refuse_storeless(interp, receiver)?;
    if parts.contents.slots.is_none() {
        keep_capacity(interp, receiver, MINIMUM_ENTRIES);
    }
    Ok(None)
}

/// Leaves `capacity` as the size [`store_of`] builds `receiver`'s store at,
/// for a receiver that has neither a store nor a size kept for one. Taking
/// the place of building the store, it is what makes a later `INIT` find the
/// size already decided, as it would find a store built.
pub(super) fn keep_capacity(interp: &mut Interp, receiver: ObjRef, capacity: usize) {
    let scope = hash_scope(interp);
    let capacity = interp.counted(capacity);
    let [_, _, _, name, _] = &interp.hash_names.contents;
    let name = Rc::clone(name);
    interp.set_pool_variables(receiver, scope, &[(&name, capacity)]);
}

/// [`store_of`]'s refusal of a receiver that has no store and is not given
/// one.
pub(super) fn refuse_storeless(interp: &mut Interp, receiver: ObjRef) -> Result<(), Failure> {
    // **A `Body::Native` receiver is not this store's**, and the check has to
    // be here because the two are one method. `IdentityTable`'s `At`, `Put` and
    // `[]` rows are donated to `StringTable` and then `StringTable`'s whole set
    // to `Directory`, so all four classes share one method identity and one
    // body -- which is why registering these names for `Table` alone reached a
    // `StringTable` during the bootstrap. `Directory`, `StringTable` and
    // `Properties` keep the entry map they already have until the task that
    // owns them.
    if matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) || !holds_hash_store(interp, receiver)
    {
        return Err(crate::Raised::receiver_kind("a value that is not a hash collection").into());
    }
    Ok(())
}

fn set_used(interp: &mut Interp, receiver: ObjRef, half: Half, used: usize) {
    let scope = hash_scope(interp);
    interp.set_pool_variable(receiver, scope, half.used, small(used as i64));
    bump_store_generation(interp);
}

/// Marks every [`StoreView`] taken before this call as stale.
pub(super) fn bump_store_generation(interp: &mut Interp) {
    interp.store_generation = interp.store_generation.wrapping_add(1);
}

pub(super) fn slot_at(
    interp: &Interp,
    array: ObjRef,
    slot: usize,
) -> Result<Option<ObjRef>, Failure> {
    Ok(array_slots(interp, array)?.get(slot).copied().flatten())
}

/// The item at `slot`, for a read that answers it or compares it rather than
/// moving it.
///
/// Refuses an entry `.environment` or `.local` holds that this crate does not
/// build.
pub(super) fn item_at(
    interp: &mut Interp,
    store: &Store,
    slot: usize,
) -> Result<Option<ObjRef>, Failure> {
    let item = slot_at(interp, store.items, slot)?;
    if let Some(held) = item
        && interp.is_owed_entry(held)
    {
        let index = slot_at(interp, store.indexes, slot)?;
        let name = index
            .map(|index| interp.to_text(index).into_owned())
            .unwrap_or_default();
        return Err(Loud::environment_entry(&name).into());
    }
    Ok(item)
}

pub(super) fn write_slot(interp: &mut Interp, array: ObjRef, slot: usize, value: Option<ObjRef>) {
    interp.store_writes = interp.store_writes.wrapping_add(1);
    if let Some(Body::Array { slots, .. }) =
        interp.heap.get_mut(array).map(|object| &mut object.body)
    {
        slots.set_within(slot, value);
    }
}

// ---- the store's own operations ----

/// Every live entry position of the receiver's contents, in insertion order.
pub(super) fn entries_of(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<usize>, Failure> {
    let store = store_of(interp, receiver)?;
    entries(interp, &store)
}

/// Every live entry position of a store the caller has already read, in
/// insertion order.
pub(super) fn entries(interp: &Interp, store: &Store) -> Result<Vec<usize>, Failure> {
    let indexes = array_slots(interp, store.indexes)?;
    Ok(indexes
        .iter()
        .take(store.used)
        .enumerate()
        .filter_map(|(position, index)| index.map(|_| position))
        .collect())
}

/// How many entries [`entries`] would answer, without collecting them.
pub(super) fn count_in(interp: &Interp, store: &Store) -> Result<usize, Failure> {
    let indexes = array_slots(interp, store.indexes)?;
    Ok(indexes
        .iter()
        .take(store.used)
        .filter(|index| index.is_some())
        .count())
}

/// The slot that files the entry at `position`, found along its code's
/// sequence.
fn slot_of(interp: &Interp, store: &Store, position: usize) -> Result<Option<usize>, Failure> {
    let Some(code) = code_in(array_slots(interp, store.codes)?, position) else {
        return Ok(None);
    };
    let slots = array_slots(interp, store.slots)?;
    let mut sequence = Sequence::new(code, slots.len());
    loop {
        match mark_in(slots, sequence.at()) {
            Mark::Empty => return Ok(None),
            Mark::Entry(held) if held == position => return Ok(Some(sequence.at())),
            _ => sequence.advance(),
        }
    }
}

/// Compacts the live entries, in their order, into fresh arrays sized for
/// `capacity`, and files them again from their cached codes. Sends nothing.
pub(super) fn rebuild(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    capacity: usize,
) -> Result<Store, Failure> {
    let store = store_in(interp, receiver, half)?;
    let indexes = array_slots(interp, store.indexes)?;
    let items = array_slots(interp, store.items)?;
    let codes = array_slots(interp, store.codes)?;
    let mut live = Vec::new();
    for (position, index) in indexes.iter().enumerate().take(store.used) {
        if let Some(index) = *index
            && let Some(code) = code_in(codes, position)
        {
            live.push((index, items.get(position).copied().flatten(), code));
        }
    }
    Ok(build(interp, receiver, half, &live, capacity))
}

/// Adds an entry after every other, without looking for an index the store
/// already holds -- the append half of `put`, and the whole of a multi-value
/// `put`. `code` is the index's [`code_of`]. A full store is rebuilt first.
pub(super) fn append(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    index: ObjRef,
    item: Option<ObjRef>,
    code: u64,
) -> Result<(), Failure> {
    let mut store = store_in(interp, receiver, half)?;
    if store.used >= store.capacity {
        let live = count_in(interp, &store)?;
        store = rebuild(interp, receiver, half, regrown(live))?;
    }
    let position = store.used;
    let slot = {
        let slots = array_slots(interp, store.slots)?;
        let mut sequence = Sequence::new(code, slots.len());
        while mark_in(slots, sequence.at()) != Mark::Empty {
            sequence.advance();
        }
        sequence.at()
    };
    write_slot(interp, store.indexes, position, Some(index));
    write_slot(interp, store.items, position, item);
    write_slot(interp, store.codes, position, Some(small(code as i64)));
    write_slot(interp, store.slots, slot, Some(small(position as i64 + 1)));
    set_used(interp, receiver, half, position + 1);
    Ok(())
}

/// Takes the entry at `position` out: a hole in the entry arrays, and a
/// deleted mark in its slot. Answers the item it held.
pub(super) fn remove_entry(
    interp: &mut Interp,
    store: &Store,
    position: usize,
) -> Result<Option<ObjRef>, Failure> {
    let item = slot_at(interp, store.items, position)?;
    if let Some(slot) = slot_of(interp, store, position)? {
        write_slot(interp, store.slots, slot, Some(small(DELETED)));
    }
    write_slot(interp, store.indexes, position, None);
    write_slot(interp, store.items, position, None);
    write_slot(interp, store.codes, position, None);
    Ok(item)
}

/// Empties `half` in place, and then gives it fresh arrays.
pub(super) fn clear(interp: &mut Interp, receiver: ObjRef, half: Half) -> Result<(), Failure> {
    let store = store_in(interp, receiver, half)?;
    // Active lookups retain these contents through callbacks and growth. EMPTY
    // clears that contents in place, and then installs fresh arrays, so that
    // no position of these arrays is ever filled a second time.
    for position in 0..store.used.min(store.capacity) {
        write_slot(interp, store.indexes, position, None);
        write_slot(interp, store.items, position, None);
        write_slot(interp, store.codes, position, None);
    }
    let slot_count = array_slots(interp, store.slots)?.len();
    for slot in 0..slot_count {
        write_slot(interp, store.slots, slot, Some(small(EMPTY)));
    }
    install_store(interp, receiver, half, MINIMUM_ENTRIES);
    Ok(())
}

/// Replaces the value under an index the table already holds, keeping its
/// place, and otherwise appends a new entry.
pub(super) fn insert(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    // A put replaces any method of the same name. Measured: `setMethod('M',
    // 'return 2')` then `d['M'] = 5` answers 5, and `unsetMethod('M')`
    // afterwards still answers 5.
    if method_store(interp, receiver)?.is_some() {
        take_in(interp, receiver, METHODS, index)?;
    }
    insert_in(interp, receiver, CONTENTS, index, item)
}

/// [`insert`] into either half of a `Directory`.
pub(super) fn insert_in(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    index: ObjRef,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    // `.local['OUTPUT'] = x` moves a route's far end.
    interp.bump_route_generation();
    let store = store_in(interp, receiver, half)?;
    let (store, found) = probe_in(interp, receiver, half, store, index)?;
    if let Some(position) = found.found {
        write_slot(interp, store.items, position, item);
        return Ok(());
    }
    // The probe sent nothing after its last look at the store, so the index
    // is still absent; the append reads the store afresh and sends nothing.
    append(interp, receiver, half, index, item, found.code)
}

/// A multi-value `put`: another entry under `index`, after every other,
/// whatever the store already holds under it.
pub(super) fn insert_duplicate(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    let keys = keys_of(interp, receiver);
    let code = code_of(hash_of(interp, keys, index)?);
    append(interp, receiver, CONTENTS, index, item, code)
}

#[cfg(test)]
mod tests;
