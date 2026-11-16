//! The hash store's layout in its receiver's pool, and the primitives that
//! read and write it.
//!
//! A store is a compact table. Its entries sit in dense arrays in the order
//! they were added, and a sparse slot index, open-addressed, maps a hash to
//! the entries filed under it. A new entry goes at the end of the entry
//! arrays; a removed one leaves a hole there. When the entry arrays fill, the
//! store is rebuilt into fresh arrays that keep the live entries in their
//! order and re-slot them from each entry's cached hash, so a rebuild sends
//! nothing. Iteration walks the entry arrays, which is what makes every
//! enumeration answer insertion order.
//!
//! **The live entries of one code are one chain**, filed at one slot: the
//! slot names the oldest of them, each links to the next, oldest first, and
//! the oldest also names the newest, so an append joins its chain without
//! walking it. Duplicates of one index share a code, so however many a
//! `Relation` or `Bag` holds, a lookup reaches the oldest at the chain's head
//! and an append or a removal of the oldest passes over no other. Distinct
//! indexes whose hashes fold alike share a chain too, and a lookup compares
//! them by `==` in the chain's order. A slot whose chain has lost its last
//! entry is marked deleted.
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
/// items and their [`FIELDS`] (cached hash and chain links), all in
/// insertion order; the slot index ([`EMPTY`], [`DELETED`], or the position
/// of a chain's oldest entry plus one); and how many entry positions are
/// used, holes included.
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
/// A slot whose chain lost its last entry: a probe passes over it, and a new
/// chain may take it once the probe has found no chain of its code.
const DELETED: i64 = -1;

/// The fields each entry has in the codes array of a linked store, at
/// `position * FIELDS`: its [`CODE`], the position of the [`NEXT`] entry of
/// its chain, and, for the oldest entry of a chain, the position of the
/// chain's [`TAIL`]. The links reuse the codes array rather than a sixth pool
/// entry, so that the store stays five names that `copy`, a rebuild and
/// `EMPTY` already treat as one: a copy duplicates the links with the codes,
/// and fresh arrays start with none.
///
/// **A store is linked only once it needs to be.** While no two of its
/// entries share a code every chain is one entry long, so its codes array
/// holds the code alone, one field per entry, and a table of distinct keys
/// pays nothing for links. The first append that joins an existing chain
/// gives the store a linked codes array ([`relink`]), and nothing else; a
/// rebuild keeps the layout it finds, and `EMPTY` installs an unlinked one. Which layout an array has is read off
/// its length: [`FIELDS`] per entry position, or one.
const FIELDS: usize = 3;
/// The entry's [`code_of`], `None` for a hole.
const CODE: usize = 0;
/// The next entry of the chain, `None` for its newest.
const NEXT: usize = 1;
/// At a chain's oldest entry, the chain's newest; `None` elsewhere.
const TAIL: usize = 2;

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

/// A store's codes array, read in its layout.
#[derive(Clone, Copy)]
pub(super) struct Codes<'a> {
    fields: &'a [Option<ObjRef>],
    /// Fields per entry: [`FIELDS`] when linked, else one.
    stride: usize,
}

impl<'a> Codes<'a> {
    /// The codes array `fields` of a store with `slot_count` slots.
    #[inline]
    pub(super) fn new(fields: &'a [Option<ObjRef>], slot_count: usize) -> Codes<'a> {
        let stride = if fields.len() > capacity_of(slot_count) {
            FIELDS
        } else {
            1
        };
        Codes { fields, stride }
    }

    #[inline]
    fn linked(&self) -> bool {
        self.stride == FIELDS
    }

    #[inline]
    fn field(&self, position: usize, field: usize) -> Option<i64> {
        if field >= self.stride {
            return None;
        }
        match self
            .fields
            .get(position * self.stride + field)
            .copied()
            .flatten()
            .map(ObjRef::decode)
        {
            Some(Decoded::SmallInt(value)) if value >= 0 => Some(value),
            _ => None,
        }
    }

    /// The code cached for the entry at a position, `None` for a hole.
    #[inline]
    pub(super) fn code(&self, position: usize) -> Option<u64> {
        self.field(position, CODE).map(|code| code as u64)
    }

    /// The entry after the one at `position` in its chain, `None` for the
    /// newest.
    #[inline]
    pub(super) fn next(&self, position: usize) -> Option<usize> {
        self.field(position, NEXT).map(|next| next as usize)
    }

    /// The newest entry of the chain whose oldest is at `head`.
    #[inline]
    fn tail(&self, head: usize) -> usize {
        self.field(head, TAIL).map_or(head, |tail| tail as usize)
    }

    /// The positions of the chain whose oldest entry is at `head`, oldest
    /// first.
    pub(super) fn chain(self, head: usize) -> impl Iterator<Item = usize> + 'a {
        std::iter::successors(Some(head), move |&position| self.next(position))
    }
}

/// Where the chain of `code` is: its slot and the position of its oldest
/// entry. When the store holds no entry of that code, `Err` with the slot a
/// new chain takes: the first deleted one the probe passed, or the empty one
/// it ended at.
#[inline]
pub(super) fn chain_in(
    slots: &[Option<ObjRef>],
    codes: Codes<'_>,
    code: u64,
) -> Result<(usize, usize), usize> {
    let mut sequence = Sequence::new(code, slots.len());
    let mut free = None;
    loop {
        match mark_in(slots, sequence.at()) {
            Mark::Empty => return Err(free.unwrap_or(sequence.at())),
            Mark::Deleted => {
                free.get_or_insert(sequence.at());
            }
            Mark::Entry(head) => {
                if codes.code(head) == Some(code) {
                    return Ok((sequence.at(), head));
                }
            }
        }
        sequence.advance();
    }
}

/// A store's codes array and slot index, read together.
#[inline]
pub(super) fn codes_of<'a>(
    interp: &'a Interp,
    store: &Store,
) -> Result<(&'a [Option<ObjRef>], Codes<'a>), Failure> {
    let slots = array_slots(interp, store.slots)?;
    let codes = Codes::new(array_slots(interp, store.codes)?, slots.len());
    Ok((slots, codes))
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
    /// Whether `other` is these same arrays, whatever their count: the same
    /// positions holding the same entries. The codes array is left out,
    /// since [`relink`] replaces it alone and moves no entry.
    pub(super) fn same_arrays(&self, other: &Store) -> bool {
        self.indexes == other.indexes && self.items == other.items && self.slots == other.slots
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
    build(interp, receiver, half, &[], capacity, false)
}

/// Builds a store holding `entries` -- (index, item, code) in their order --
/// sized for at least `capacity`, and binds it to `receiver` in place of
/// whatever it had. An unlinked one is built only for entries of distinct
/// codes, which is what an unlinked store holds.
fn build(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    entries: &[(ObjRef, Option<ObjRef>, u64)],
    capacity: usize,
    linked: bool,
) -> Store {
    let slot_count = slots_for(capacity.max(entries.len()));
    let capacity = capacity_of(slot_count);
    let stride = if linked { FIELDS } else { 1 };
    let scope = hash_scope(interp);
    let mut index_slots = vec![None; capacity];
    let mut item_slots = vec![None; capacity];
    let mut code_slots = vec![None; capacity * stride];
    let mut marks = vec![Some(small(EMPTY)); slot_count];
    for (position, &(index, item, code)) in entries.iter().enumerate() {
        index_slots[position] = Some(index);
        item_slots[position] = item;
        code_slots[position * stride + CODE] = Some(small(code as i64));
        let codes = Codes::new(&code_slots, slot_count);
        match chain_in(&marks, codes, code) {
            Ok((_, head)) => {
                debug_assert!(linked, "an unlinked store holds one entry per code");
                let tail = codes.tail(head);
                code_slots[tail * stride + NEXT] = Some(small(position as i64));
                code_slots[head * stride + TAIL] = Some(small(position as i64));
            }
            Err(slot) => marks[slot] = Some(small(position as i64 + 1)),
        }
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

/// Compacts the live entries, in their order, into fresh arrays sized for
/// `capacity`, and files them again from their cached codes. Sends nothing.
/// The store stays linked or unlinked as it was.
pub(super) fn rebuild(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    capacity: usize,
) -> Result<Store, Failure> {
    let store = store_in(interp, receiver, half)?;
    let indexes = array_slots(interp, store.indexes)?;
    let items = array_slots(interp, store.items)?;
    let (_, codes) = codes_of(interp, &store)?;
    let linked = codes.linked();
    let mut live = Vec::new();
    for (position, index) in indexes.iter().enumerate().take(store.used) {
        if let Some(index) = *index
            && let Some(code) = codes.code(position)
        {
            live.push((index, items.get(position).copied().flatten(), code));
        }
    }
    Ok(build(interp, receiver, half, &live, capacity, linked))
}

/// Adds an entry after every other, without looking for an index the store
/// already holds -- the append half of `put`, and the whole of a multi-value
/// `put`. `code` is the index's [`code_of`]. A full store is rebuilt first.
///
/// `known` is where a probe of the receiver's store, as it is now, last
/// found the chain of `code` ([`Probe::chain`]).
pub(super) fn append(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    index: ObjRef,
    item: Option<ObjRef>,
    code: u64,
    mut known: Option<Result<(usize, usize), usize>>,
) -> Result<(), Failure> {
    let mut store = store_in(interp, receiver, half)?;
    if store.used >= store.capacity {
        let live = count_in(interp, &store)?;
        store = rebuild(interp, receiver, half, regrown(live))?;
        known = None;
    }
    let (mut chain, mut stride) = chain_for_append(interp, &store, code, known)?;
    if chain.is_ok() && stride != FIELDS {
        // The first entry to join a chain: the store becomes linked. Its
        // slots and positions stay, so the chain found stays where it is.
        store = relink(interp, receiver, half, &store)?;
        (chain, stride) = chain_for_append(interp, &store, code, None)?;
    }
    let position = store.used;
    write_slot(interp, store.indexes, position, Some(index));
    write_slot(interp, store.items, position, item);
    write_field(interp, &store, stride, position, CODE, Some(code as usize));
    match chain {
        Ok((head, tail)) => {
            write_field(interp, &store, stride, tail, NEXT, Some(position));
            write_field(interp, &store, stride, head, TAIL, Some(position));
        }
        Err(slot) => write_slot(interp, store.slots, slot, Some(small(position as i64 + 1))),
    }
    set_used(interp, receiver, half, position + 1);
    Ok(())
}

/// Gives an unlinked store a linked codes array holding the same codes, and
/// keeps its other arrays: every chain of an unlinked store is one entry
/// long, so no slot or position moves and there are no links to copy.
fn relink(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    store: &Store,
) -> Result<Store, Failure> {
    let mut fields = vec![None; store.capacity * FIELDS];
    {
        let (_, codes) = codes_of(interp, store)?;
        for position in 0..store.used {
            if let Some(code) = codes.code(position) {
                fields[position * FIELDS + CODE] = Some(small(code as i64));
            }
        }
    }
    let codes = interp.alloc_charged(BehaviourId::ARRAY, Body::array(fields));
    interp.roots.activity_mut().push_temp(codes);
    let scope = hash_scope(interp);
    interp.set_pool_variable(receiver, scope, half.codes, codes);
    bump_store_generation(interp);
    Ok(Store { codes, ..*store })
}

/// What [`append`] joins for `code`: the oldest and newest entry of its
/// chain, or the slot a new chain takes; and the store's stride.
///
/// `known` is read again at its slot rather than trusted: the probe's
/// comparisons may have removed entries since it walked the slots, which
/// moves a chain's oldest entry or marks its slot deleted, though, without
/// an append, it neither fills a free slot nor starts a chain. Only a slot
/// that no longer reads as `known` said sends the walk round again.
#[inline]
fn chain_for_append(
    interp: &Interp,
    store: &Store,
    code: u64,
    known: Option<Result<(usize, usize), usize>>,
) -> Result<(Result<(usize, usize), usize>, usize), Failure> {
    let (slots, codes) = codes_of(interp, store)?;
    let reread = known.and_then(|known| {
        let slot = match known {
            Ok((slot, _)) | Err(slot) => slot,
        };
        match (known, mark_in(slots, slot)) {
            (Ok(_), Mark::Entry(head)) if codes.code(head) == Some(code) => Some(Ok(head)),
            (Ok(_), Mark::Deleted) | (Err(_), Mark::Empty | Mark::Deleted) => Some(Err(slot)),
            _ => None,
        }
    });
    let chain = match reread {
        Some(chain) => chain,
        None => chain_in(slots, codes, code).map(|(_, head)| head),
    };
    Ok((chain.map(|head| (head, codes.tail(head))), codes.stride))
}

/// Writes one of the fields of the entry at `position`, in a codes array of
/// `stride` fields per entry.
fn write_field(
    interp: &mut Interp,
    store: &Store,
    stride: usize,
    position: usize,
    field: usize,
    value: Option<usize>,
) {
    debug_assert!(field < stride, "an unlinked store has no links to write");
    let value = value.map(|value| small(value as i64));
    write_slot(interp, store.codes, position * stride + field, value);
}

/// How taking the entry at a position out relinks its chain.
enum Unlink {
    /// It was the oldest: its slot names the next one, which takes over the
    /// chain's tail, or is marked deleted when it was the only one.
    Head {
        slot: usize,
        next: Option<usize>,
        tail: usize,
    },
    /// It followed `previous`, which now links past it, and the chain's
    /// oldest, at `head`, names `previous` as its tail if it was the tail.
    After {
        head: usize,
        previous: usize,
        next: Option<usize>,
        was_tail: bool,
    },
}

/// Takes the entry at `position` out: a hole in the entry arrays, and its
/// chain relinked past it. Answers the item it held.
pub(super) fn remove_entry(
    interp: &mut Interp,
    store: &Store,
    position: usize,
) -> Result<Option<ObjRef>, Failure> {
    let item = slot_at(interp, store.items, position)?;
    let (unlink, stride) = {
        let (slots, codes) = codes_of(interp, store)?;
        let unlink = codes
            .code(position)
            .and_then(|code| chain_in(slots, codes, code).ok())
            .and_then(|(slot, head)| {
                let next = codes.next(position);
                if head == position {
                    return Some(Unlink::Head {
                        slot,
                        next,
                        tail: codes.tail(head),
                    });
                }
                let previous = codes
                    .chain(head)
                    .find(|&at| codes.next(at) == Some(position))?;
                Some(Unlink::After {
                    head,
                    previous,
                    next,
                    was_tail: codes.tail(head) == position,
                })
            });
        (unlink, codes.stride)
    };
    match unlink {
        Some(Unlink::Head {
            slot,
            next: Some(next),
            tail,
        }) => {
            write_slot(interp, store.slots, slot, Some(small(next as i64 + 1)));
            write_field(interp, store, stride, next, TAIL, Some(tail));
        }
        Some(Unlink::Head {
            slot, next: None, ..
        }) => {
            write_slot(interp, store.slots, slot, Some(small(DELETED)));
        }
        Some(Unlink::After {
            head,
            previous,
            next,
            was_tail,
        }) => {
            write_field(interp, store, stride, previous, NEXT, next);
            if was_tail {
                write_field(interp, store, stride, head, TAIL, Some(previous));
            }
        }
        None => {}
    }
    write_slot(interp, store.indexes, position, None);
    write_slot(interp, store.items, position, None);
    for field in 0..stride {
        write_field(interp, store, stride, position, field, None);
    }
    Ok(item)
}

/// Empties `half` in place, and then gives it fresh arrays.
pub(super) fn clear(interp: &mut Interp, receiver: ObjRef, half: Half) -> Result<(), Failure> {
    let store = store_in(interp, receiver, half)?;
    let stride = codes_of(interp, &store)?.1.stride;
    // Active lookups retain these contents through callbacks and growth. EMPTY
    // clears that contents in place, and then installs fresh arrays, so that
    // no position of these arrays is ever filled a second time.
    for position in 0..store.used.min(store.capacity) {
        write_slot(interp, store.indexes, position, None);
        write_slot(interp, store.items, position, None);
        for field in 0..stride {
            write_field(interp, &store, stride, position, field, None);
        }
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
    // is still absent and its chain where the probe found it; the append
    // reads the store afresh and sends nothing.
    append(
        interp,
        receiver,
        half,
        index,
        item,
        found.code,
        Some(found.chain),
    )
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
    append(interp, receiver, CONTENTS, index, item, code, None)
}

#[cfg(test)]
mod tests;
