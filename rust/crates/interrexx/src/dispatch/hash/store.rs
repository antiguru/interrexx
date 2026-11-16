//! The hash store's layout in its receiver's pool, and the primitives that
//! read and write it.

use super::*;

/// The five pool entries a hash store is, bound in the receiver's own pool
/// under the `MapCollection` class as scope.
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
    pub(super) next: &'static [u8],
    pub(super) buckets: &'static [u8],
    pub(super) free: &'static [u8],
}

/// The store every mapped collection has.
pub(super) const CONTENTS: Half = Half {
    indexes: HASH_INDEXES,
    items: HASH_ITEMS,
    next: HASH_NEXT,
    buckets: HASH_BUCKETS,
    free: HASH_FREE,
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
    next: METHOD_NEXT,
    buckets: METHOD_BUCKETS,
    free: METHOD_FREE,
};

/// The one method `setMethod` keeps out of the method table, because `UNKNOWN`
/// is not an entry -- it is what answers when there is none. Measured: with it
/// set, `items` is still 1 for a directory holding one ordinary entry and
/// `allIndexes` does not name it.
pub(super) const UNKNOWN_METHOD: &[u8] = b"METHODUNKNOWN";

pub(super) const MINIMUM_BUCKET_SIZE: usize = 17;

pub(super) fn calculate_bucket_size(capacity: usize) -> usize {
    if capacity >= 1 << 30 {
        return 1 << 30;
    }
    if capacity < MINIMUM_BUCKET_SIZE {
        return MINIMUM_BUCKET_SIZE;
    }
    // `(capacity & 1) == 0`: an odd bucket count is preferred.
    if capacity.is_multiple_of(2) {
        capacity + 1
    } else {
        capacity
    }
}

/// A hash store's three arrays and its two scalars, read out together.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct Store {
    pub(super) indexes: ObjRef,
    pub(super) items: ObjRef,
    pub(super) next: ObjRef,
    pub(super) buckets: usize,
    pub(super) free: usize,
    pub(super) total: usize,
}

impl Store {
    /// The chain terminator, which is one past the last slot.
    pub(super) fn no_more(&self) -> usize {
        self.total
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
            [half.indexes, half.items, half.next, half.buckets, half.free].map(Rc::from)
        };
        PoolNames {
            contents: names(CONTENTS),
            methods: names(METHODS),
        }
    }
}

/// Builds a store of `buckets` primary slots and as many overflow slots, and
/// binds it to `receiver`.
pub(super) fn install_store(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    buckets: usize,
) -> Store {
    let total = buckets * 2;
    let scope = hash_scope(interp);
    let indexes = interp.alloc_charged(BehaviourId::ARRAY, Body::array(vec![None; total]));
    interp.roots.activity_mut().push_temp(indexes);
    let items = interp.alloc_charged(BehaviourId::ARRAY, Body::array(vec![None; total]));
    interp.roots.activity_mut().push_temp(items);
    // Every bucket slot ends its own chain, and the overflow slots are chained
    // upward from the bucket count with the last one ending the chain.
    let mut links: Vec<Option<ObjRef>> = Vec::with_capacity(total);
    for slot in 0..total {
        let link = if slot < buckets || slot + 1 == total {
            total
        } else {
            slot + 1
        };
        links.push(Some(interp.counted(link)));
    }
    let next = interp.alloc_charged(BehaviourId::ARRAY, Body::array(links));
    interp.roots.activity_mut().push_temp(next);
    let buckets_value = interp.counted(buckets);
    let free_value = interp.counted(buckets);
    let names = if half == CONTENTS {
        &interp.hash_names.contents
    } else {
        &interp.hash_names.methods
    };
    let [n_indexes, n_items, n_next, n_buckets, n_free] = names.clone();
    // A bucket count [`initialize_store`] left pending goes, so that the five
    // entries are bound together in their order, as [`shared_run`] reads them.
    if pool_variable(interp, receiver, scope, half.indexes).is_none()
        && pool_variable(interp, receiver, scope, half.buckets).is_some()
    {
        interp.clear_pool_variable(receiver, scope, half.buckets);
    }
    interp.set_pool_variables(
        receiver,
        scope,
        &[
            (&n_indexes, indexes),
            (&n_items, items),
            (&n_next, next),
            (&n_buckets, buckets_value),
            (&n_free, free_value),
        ],
    );
    bump_store_generation(interp);
    Store {
        indexes,
        items,
        next,
        buckets,
        free: buckets,
        total,
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
    pub(super) next: Option<ObjRef>,
    pub(super) buckets: Option<usize>,
    pub(super) free: Option<usize>,
}

impl Found {
    /// The store these entries make, or `None` when any is missing.
    pub(super) fn store(self, interp: &Interp) -> Result<Option<Store>, Failure> {
        let (Some(indexes), Some(items), Some(next), Some(buckets), Some(free)) =
            (self.indexes, self.items, self.next, self.buckets, self.free)
        else {
            return Ok(None);
        };
        let total = array_slots(interp, indexes)?.len();
        Ok(Some(Store {
            indexes,
            items,
            next,
            buckets,
            free,
            total,
        }))
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
            HASH_NEXT => parts.contents.next = Some(value),
            HASH_BUCKETS => parts.contents.buckets = counted(value),
            HASH_FREE => parts.contents.free = counted(value),
            METHOD_INDEXES => parts.methods.indexes = Some(value),
            METHOD_ITEMS => parts.methods.items = Some(value),
            METHOD_NEXT => parts.methods.next = Some(value),
            METHOD_BUCKETS => parts.methods.buckets = counted(value),
            METHOD_FREE => parts.methods.free = counted(value),
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
        (next_name, next),
        (buckets_name, buckets),
        (free_name, free),
        ..,
    ] = entries
    else {
        return None;
    };
    let [indexes_at, items_at, next_at, buckets_at, free_at] = names;
    (Rc::ptr_eq(indexes_name, indexes_at)
        && Rc::ptr_eq(items_name, items_at)
        && Rc::ptr_eq(next_name, next_at)
        && Rc::ptr_eq(buckets_name, buckets_at)
        && Rc::ptr_eq(free_name, free_at))
    .then(|| Found {
        indexes: Some(*indexes),
        items: Some(*items),
        next: Some(*next),
        buckets: counted(*buckets),
        free: counted(*free),
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
        None => Ok(install_store(interp, receiver, half, MINIMUM_BUCKET_SIZE)),
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
    // The bucket count [`initialize_store`] left pending, if it left one.
    let buckets = parts.contents.buckets.unwrap_or(MINIMUM_BUCKET_SIZE);
    Ok(install_store(interp, receiver, CONTENTS, buckets))
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
    if parts.contents.buckets.is_none() {
        keep_bucket_count(interp, receiver, MINIMUM_BUCKET_SIZE);
    }
    Ok(None)
}

/// Leaves `buckets` as the size [`store_of`] builds `receiver`'s store at,
/// for a receiver that has neither a store nor a size kept for one. Taking
/// the place of building the store, it is what makes a later `INIT` find the
/// size already decided, as it would find a store built.
pub(super) fn keep_bucket_count(interp: &mut Interp, receiver: ObjRef, buckets: usize) {
    let scope = hash_scope(interp);
    let buckets = interp.counted(buckets);
    let [_, _, _, name, _] = &interp.hash_names.contents;
    let name = Rc::clone(name);
    interp.set_pool_variables(receiver, scope, &[(&name, buckets)]);
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

pub(super) fn set_free(interp: &mut Interp, receiver: ObjRef, half: Half, free: usize) {
    let scope = hash_scope(interp);
    let value = interp.counted(free);
    interp.set_pool_variable(receiver, scope, half.free, value);
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

pub(super) fn link_at(interp: &Interp, store: &Store, slot: usize) -> Result<usize, Failure> {
    match slot_at(interp, store.next, slot)?.map(|value| value.decode()) {
        Some(Decoded::SmallInt(link)) if link >= 0 => Ok(link as usize),
        _ => Ok(store.no_more()),
    }
}

pub(super) fn write_slot(interp: &mut Interp, array: ObjRef, slot: usize, value: Option<ObjRef>) {
    interp.store_writes = interp.store_writes.wrapping_add(1);
    if let Some(Body::Array { slots, .. }) =
        interp.heap.get_mut(array).map(|object| &mut object.body)
    {
        slots.set_within(slot, value);
    }
}

pub(super) fn write_link(interp: &mut Interp, store: &Store, slot: usize, link: usize) {
    let value = interp.counted(link);
    write_slot(interp, store.next, slot, Some(value));
}

// ---- the store's own operations ----

/// Every occupied slot, in bucket order following each chain.
pub(super) fn walk(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<usize>, Failure> {
    let store = store_of(interp, receiver)?;
    walk_in(interp, &store)
}

/// [`walk`] over a store the caller has already read.
pub(super) fn walk_in(interp: &Interp, store: &Store) -> Result<Vec<usize>, Failure> {
    let mut order = Vec::new();
    walk_each(interp, store, |slot| order.push(slot))?;
    Ok(order)
}

/// How many entries [`walk_in`] would answer, without collecting them.
pub(super) fn count_in(interp: &Interp, store: &Store) -> Result<usize, Failure> {
    let mut count = 0;
    walk_each(interp, store, |_| count += 1)?;
    Ok(count)
}

/// Calls `visit` with every occupied slot in [`walk_in`]'s order. The two
/// arrays are borrowed once rather than looked up per slot, each at the
/// point the slot-by-slot walk first reads it, so a store whose arrays are
/// not arrays refuses where it did.
pub(super) fn walk_each(
    interp: &Interp,
    store: &Store,
    mut visit: impl FnMut(usize),
) -> Result<(), Failure> {
    if store.buckets == 0 || store.total == 0 {
        return Ok(());
    }
    let indexes = array_slots(interp, store.indexes)?;
    let mut next: Option<&[Option<ObjRef>]> = None;
    for bucket in 0..store.buckets {
        let mut slot = bucket;
        while slot < store.total && indexes.get(slot).copied().flatten().is_some() {
            visit(slot);
            let links = match next {
                Some(links) => links,
                None => *next.insert(array_slots(interp, store.next)?),
            };
            slot = match links.get(slot).copied().flatten().map(ObjRef::decode) {
                Some(Decoded::SmallInt(link)) if link >= 0 => link as usize,
                _ => store.no_more(),
            };
        }
    }
    Ok(())
}

/// Occupied entries and their predecessors, restricted to a bucket when supplied.
pub(super) fn entries_in(
    interp: &Interp,
    store: &Store,
    bucket: Option<usize>,
) -> Result<Vec<(usize, Option<usize>)>, Failure> {
    let mut entries = Vec::new();
    let buckets = match bucket {
        Some(bucket) => bucket..bucket + 1,
        None => 0..store.buckets,
    };
    for bucket in buckets {
        let mut previous = None;
        let mut slot = bucket;
        while slot < store.total && slot_at(interp, store.indexes, slot)?.is_some() {
            entries.push((slot, previous));
            previous = Some(slot);
            slot = link_at(interp, store, slot)?;
        }
    }
    Ok(entries)
}

/// Grows the table and re-adds every entry in old bucket order.
pub(super) fn expand(interp: &mut Interp, receiver: ObjRef, half: Half) -> Result<(), Failure> {
    let store = store_in(interp, receiver, half)?;
    let order = walk_in(interp, &store)?;
    let mut carried = Vec::with_capacity(order.len());
    for slot in order {
        let index = slot_at(interp, store.indexes, slot)?;
        let item = slot_at(interp, store.items, slot)?;
        if let Some(index) = index {
            interp.roots.activity_mut().push_temp(index);
            if let Some(item) = item {
                interp.roots.activity_mut().push_temp(item);
            }
            carried.push((index, item));
        }
    }
    let buckets = calculate_bucket_size(store.total * 2);
    install_store(interp, receiver, half, buckets);
    for (index, item) in carried {
        // **Never `insert`**, which replaces an index the table already holds:
        // a `Relation` carries duplicate indexes and would lose one per pair
        // on every growth. The merge adds rather than puts, and adding in the
        // old walk order is what keeps each index's chain in its order.
        append_entry(interp, receiver, half, index, item)?;
    }
    Ok(())
}

/// Appends an entry at the end of its bucket's chain without looking for an
/// index the table already holds -- the append half of `put`.
pub(super) fn append_entry(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    index: ObjRef,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    let store = store_in(interp, receiver, half)?;
    if store.free >= store.total {
        expand(interp, receiver, half)?;
        return append_entry(interp, receiver, half, index, item);
    }
    let store = store_in(interp, receiver, half)?;
    let keys = keys_of(interp, receiver);
    let hash = hash_of(interp, keys, index)?;
    let bucket = (hash % store.buckets as u64) as usize;
    if slot_at(interp, store.indexes, bucket)?.is_none() {
        write_slot(interp, store.indexes, bucket, Some(index));
        write_slot(interp, store.items, bucket, item);
        write_link(interp, &store, bucket, store.no_more());
        return Ok(());
    }
    let mut last = bucket;
    loop {
        let next = link_at(interp, &store, last)?;
        if next >= store.total {
            break;
        }
        last = next;
    }
    let slot = store.free;
    let next_free = link_at(interp, &store, slot)?;
    set_free(interp, receiver, half, next_free);
    write_slot(interp, store.indexes, slot, Some(index));
    write_slot(interp, store.items, slot, item);
    write_link(interp, &store, last, slot);
    write_link(interp, &store, slot, store.no_more());
    Ok(())
}

/// A new entry for an index the table already holds goes to the FRONT of that
/// index's chain.
pub(super) fn insert_front(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    let store = store_of(interp, receiver)?;
    if store.free >= store.total {
        expand(interp, receiver, CONTENTS)?;
        return insert_front(interp, receiver, index, item);
    }
    let store = store_of(interp, receiver)?;
    let keys = keys_of(interp, receiver);
    let hash = hash_of(interp, keys, index)?;
    let bucket = (hash % store.buckets as u64) as usize;
    if slot_at(interp, store.indexes, bucket)?.is_none() {
        write_slot(interp, store.indexes, bucket, Some(index));
        write_slot(interp, store.items, bucket, item);
        write_link(interp, &store, bucket, store.no_more());
        return Ok(());
    }
    let moved = store.free;
    let next_free = link_at(interp, &store, moved)?;
    set_free(interp, receiver, CONTENTS, next_free);
    let head_index = slot_at(interp, store.indexes, bucket)?;
    let head_item = slot_at(interp, store.items, bucket)?;
    let head_next = link_at(interp, &store, bucket)?;
    write_slot(interp, store.indexes, moved, head_index);
    write_slot(interp, store.items, moved, head_item);
    write_link(interp, &store, moved, head_next);
    write_slot(interp, store.indexes, bucket, Some(index));
    write_slot(interp, store.items, bucket, item);
    write_link(interp, &store, bucket, moved);
    Ok(())
}

/// Replaces the value under an index the table already holds, and otherwise
/// appends to the end of the bucket's chain, expanding first if the free chain
/// has run out.
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
    // **The fullness test comes first, and it is the free chain rather than the
    // item count**: a table can run its overflow dry while primary buckets
    // stand empty, and the table expands before every `put` rather than only
    // when one needs a slot. In the simulation this was written from, testing
    // the item count instead runs the free chain out on the forty-integer case.
    let store = store_in(interp, receiver, half)?;
    if store.free >= store.total {
        expand(interp, receiver, half)?;
    }
    let store = store_in(interp, receiver, half)?;
    let (store, found) = probe_in(interp, receiver, half, store, index)?;
    if let Some(slot) = found.found {
        write_slot(interp, store.items, slot, item);
        return Ok(());
    }
    let Some(last) = found.last else {
        write_slot(interp, store.indexes, found.bucket, Some(index));
        write_slot(interp, store.items, found.bucket, item);
        write_link(interp, &store, found.bucket, store.no_more());
        return Ok(());
    };
    let slot = store.free;
    let next_free = link_at(interp, &store, slot)?;
    set_free(interp, receiver, half, next_free);
    write_slot(interp, store.indexes, slot, Some(index));
    write_slot(interp, store.items, slot, item);
    write_link(interp, &store, last, slot);
    write_link(interp, &store, slot, store.no_more());
    Ok(())
}
