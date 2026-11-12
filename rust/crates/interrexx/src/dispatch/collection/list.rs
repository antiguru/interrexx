//! `List`'s primitive methods: the three pool entries a list is, and the
//! methods over them.

use super::{
    Arity, BehaviourId, Body, Cleared, Failure, Interp, NativeMethod, ObjRef, Raised, array_of,
    array_of_slots, item_argument, new_instance, new_supplier, optional_length_argument,
    pool_variable, same_item,
};

// ---- List ----

/// A `List` index argument, whose position in the error is **two** whatever
/// its position in the argument list is.
fn list_index_argument(args: &[Option<ObjRef>], at: usize) -> Result<ObjRef, Failure> {
    args.get(at)
        .copied()
        .flatten()
        .ok_or_else(|| Raised::missing_method_argument(2).into())
}

/// A `List`'s one pool entry, under the `List` class as scope: a chain of
/// entries addressed by handle number, doubly linked in list order.
///
/// The array starts with [`HEADER`] slots -- the first and last handle, the
/// count of entries and the top of the free stack -- and then holds
/// [`ENTRY`] slots per handle ever issued: the item ([`ITEM`], empty for an
/// entry holding nothing), the next entry ([`NEXT`]) and the previous one
/// ([`PREV`]). A link is the handle plus one as an inline integer, `0` for
/// none. An entry in use always has a `PREV` slot; a freed one has none, and
/// its `NEXT` is the freed handle below it on the free stack. So lookup by
/// handle, insertion next to an entry and removal are each constant time,
/// and only walks cost the length of the list.
const LIST_CHAIN: &[u8] = b"CHAIN";

const HEAD: usize = 0;
const TAIL: usize = 1;
const COUNT: usize = 2;
const FREE: usize = 3;
const HEADER: usize = 4;
const ENTRY: usize = 3;
const ITEM: usize = 0;
const NEXT: usize = 1;
const PREV: usize = 2;

/// The `List` class, which is the scope the chain is bound under.
fn list_scope(interp: &mut Interp) -> ObjRef {
    interp.object_model().list
}

/// A link or a count as the inline integer the chain stores.
fn stored(value: usize) -> Option<ObjRef> {
    i64::try_from(value).ok().and_then(ObjRef::small_int)
}

/// A link: the handle plus one, or `0` for none.
fn link(handle: Option<usize>) -> Option<ObjRef> {
    stored(handle.map_or(0, |handle| handle + 1))
}

/// The whole number a stored count or link is.
fn number(slot: Option<ObjRef>) -> usize {
    match slot.map(ObjRef::decode) {
        Some(crate::dispatch::Decoded::SmallInt(small)) => usize::try_from(small).unwrap_or(0),
        _ => 0,
    }
}

/// The handle a stored link names.
fn linked(slot: Option<ObjRef>) -> Option<usize> {
    number(slot).checked_sub(1)
}

/// The slot of field `field` of entry `handle`.
fn at(handle: usize, field: usize) -> usize {
    HEADER + handle * ENTRY + field
}

/// A read-only view of a chain's slots.
#[derive(Clone, Copy)]
struct Chain<'a>(&'a [Option<ObjRef>]);

impl Chain<'_> {
    fn head(self) -> Option<usize> {
        linked(self.0.get(HEAD).copied().flatten())
    }

    fn tail(self) -> Option<usize> {
        linked(self.0.get(TAIL).copied().flatten())
    }

    fn count(self) -> usize {
        number(self.0.get(COUNT).copied().flatten())
    }

    /// How many handles have ever been issued since the list was last empty.
    fn issued(self) -> usize {
        self.0.len().saturating_sub(HEADER) / ENTRY
    }

    fn in_use(self, handle: usize) -> bool {
        handle < self.issued() && self.0[at(handle, PREV)].is_some()
    }

    fn item(self, handle: usize) -> Option<ObjRef> {
        self.0[at(handle, ITEM)]
    }

    fn next(self, handle: usize) -> Option<usize> {
        linked(self.0[at(handle, NEXT)])
    }

    fn previous(self, handle: usize) -> Option<usize> {
        linked(self.0[at(handle, PREV)])
    }
}

/// The list's chain, built on demand.
pub(super) fn list_state(interp: &mut Interp, receiver: ObjRef) -> Result<ObjRef, Failure> {
    let scope = list_scope(interp);
    if let Some(chain) = pool_variable(interp, receiver, scope, LIST_CHAIN) {
        return Ok(chain);
    }
    // Built on demand for [`store_of`]'s reason: the contents belong to the
    // allocated object, so a subclass `INIT` that does not forward still has
    // them: a `List` subclass whose `INIT` only sets an exposed variable
    // answers `items 1` after `append('a')`.
    if !is_list(interp, receiver) {
        return Err(crate::Raised::receiver_kind("a value that is not a list").into());
    }
    let chain = interp.alloc_charged(BehaviourId::ARRAY, Body::array(vec![link(None); HEADER]));
    interp.roots.activity_mut().push_temp(chain);
    interp.set_pool_variable(receiver, scope, LIST_CHAIN, chain);
    Ok(chain)
}

/// `read` over a chain [`list_state`] answered.
fn read_chain<R>(
    interp: &Interp,
    chain: ObjRef,
    read: impl FnOnce(Chain<'_>) -> R,
) -> Result<R, Failure> {
    match interp.array_slots(chain) {
        Some(slots) => Ok(read(Chain(slots))),
        None => Err(crate::Raised::receiver_kind("a value that is not a list").into()),
    }
}

/// `write` over a chain [`list_state`] answered, charging what the slots
/// grew by.
fn write_chain<R>(
    interp: &mut Interp,
    chain: ObjRef,
    write: impl FnOnce(&mut interrexx_core::ArraySlots) -> R,
) -> Result<R, Failure> {
    // As every splice of a collection's slots does.
    interp.bump_route_generation();
    let (answer, grown) = match interp.heap.get_mut(chain).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            let before = slots.capacity();
            let answer = write(slots);
            (answer, slots.capacity().saturating_sub(before))
        }
        _ => return Err(crate::Raised::receiver_kind("a value that is not a list").into()),
    };
    interp.charge_growth(grown * interrexx_core::SLOT_BYTES);
    Ok(answer)
}

/// Whether `receiver` is a `List` or something deriving from one --
/// [`super::is_queue`]'s question for the other family.
pub(super) fn is_list(interp: &mut Interp, receiver: ObjRef) -> bool {
    let Some(class) = interp.class_of_value(receiver) else {
        return false;
    };
    let list = list_scope(interp);
    interp.classes().is_a(class, list)
}

/// The entry `handle` names, or `None` for an index the list does not hold.
fn list_position(
    interp: &mut Interp,
    chain: ObjRef,
    handle: ObjRef,
) -> Result<Option<usize>, Failure> {
    let Some(wanted) = super::unsigned_index(interp, handle)? else {
        let written = interp.to_text(handle);
        return Err(Raised::incorrect_list_index(&written).into());
    };
    // `isIndexValid` is `isInUse`, so `validateIndex` answers NoLink
    // for an entry holding nothing and every lookup reads it as
    // absent. Measured over `l~put(, 1)`: `hasIndex(1)` is `0`,
    // `at(1)` and `next(1)` are `.nil`, and `section(1,1)` raises --
    // while the chain still walks through the entry, which is
    // [`list_pairs`]'s business and not this function's.
    read_chain(interp, chain, |chain| {
        (chain.in_use(wanted) && chain.item(wanted).is_some()).then_some(wanted)
    })
}

/// Links a new entry holding `item` in after `after`, or at the front for
/// `None`, and answers its handle number. The handle is the last one removal
/// freed, or else a fresh one: fresh handles count from zero and never
/// repeat while the list holds them.
fn link_after(
    slots: &mut interrexx_core::ArraySlots,
    after: Option<usize>,
    item: Option<ObjRef>,
) -> usize {
    let chain = Chain(slots);
    let next = match after {
        Some(after) => chain.next(after),
        None => chain.head(),
    };
    let count = chain.count();
    let handle = match linked(slots[FREE]) {
        Some(freed) => {
            let below = slots[at(freed, NEXT)];
            slots.set(FREE, below);
            freed
        }
        None => {
            let fresh = Chain(slots).issued();
            slots.resize(at(fresh + 1, 0));
            fresh
        }
    };
    slots.set(at(handle, ITEM), item);
    slots.set(at(handle, NEXT), link(next));
    slots.set(at(handle, PREV), link(after));
    match after {
        Some(after) => slots.set(at(after, NEXT), link(Some(handle))),
        None => slots.set(HEAD, link(Some(handle))),
    }
    match next {
        Some(next) => slots.set(at(next, PREV), link(Some(handle))),
        None => slots.set(TAIL, link(Some(handle))),
    }
    slots.set(COUNT, stored(count + 1));
    handle
}

/// Puts `item` after entry `after`, or at the front for `None`, and answers
/// the new handle.
fn list_insert_after(
    interp: &mut Interp,
    chain: ObjRef,
    after: Option<usize>,
    item: Option<ObjRef>,
) -> Result<ObjRef, Failure> {
    let handle = write_chain(interp, chain, |slots| link_after(slots, after, item))?;
    Ok(interp.counted(handle))
}

/// Puts `item` at the end with a fresh handle, and answers the handle.
pub(super) fn list_append(
    interp: &mut Interp,
    receiver: ObjRef,
    item: Option<ObjRef>,
) -> Result<ObjRef, Failure> {
    let chain = list_state(interp, receiver)?;
    let tail = read_chain(interp, chain, |chain| chain.tail())?;
    list_insert_after(interp, chain, tail, item)
}

/// `List~init([size])`: the optional capacity, and the chain.
fn native_list_init(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    super::optional_length_argument(interp, args, 0)?;
    // **`INIT` sent a second time keeps the contents.** `List~init` takes the
    // optional size and nothing else; the entries belong to the allocated
    // object. Measured, `l~init` on a two-item list still answers `items 2`,
    // where rebuilding the entries here answered `0`.
    list_state(interp, receiver)?;
    Ok(None)
}

/// `List~append(item)`: adds at the end and answers the new handle.
fn native_list_append(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let item = item_argument(args)?;
    Ok(Some(list_append(interp, receiver, Some(item))?))
}

/// `List~insert(item [, index])`: after `index`, or at the end when the index
/// is omitted, or at the front when it is `.nil`.
fn native_list_insert(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    // The value is optional -- `insert` requires no argument for it. Measured:
    // `.List~new~insert` answers `0` at rc 0.
    let item = args.first().copied().flatten();
    let after = match args.get(1).copied() {
        Some(Some(index)) if index == ObjRef::NIL => None,
        Some(Some(index)) => match list_position(interp, chain, index)? {
            Some(handle) => Some(handle),
            None => {
                let written = interp.to_text(index);
                return Err(Raised::incorrect_list_index(&written).into());
            }
        },
        Some(None) | None => return Ok(Some(list_append(interp, receiver, item)?)),
    };
    Ok(Some(list_insert_after(interp, chain, after, item)?))
}

/// `List~at(index)` and `List~[index]`: the item at that handle, or `.nil`.
fn native_list_at(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    // **Argument TWO, not one.** `List~at` validates the index as argument 2,
    // so a bare `~at` reports `argument 2 is required` even though the method
    // takes one argument.
    let handle = list_index_argument(args, 0)?;
    let Some(entry) = list_position(interp, chain, handle)? else {
        return Ok(Some(ObjRef::NIL));
    };
    let item = read_chain(interp, chain, |chain| chain.item(entry))?;
    Ok(Some(item.unwrap_or(ObjRef::NIL)))
}

/// `List~put(item, index)` and `List~[index] = item`: replaces at that
/// handle, and 93.918 for a handle the list does not hold -- the same error
/// `Queue~put` raises, which is why the constructor is shared.
fn native_list_put(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    // The INDEX is what is required, and it is argument two:
    // `requiredIndex(argIndex, ARG_TWO)`. The value is not checked at all --
    // measured, `l~put(, handle)` succeeds and leaves the entry holding
    // nothing.
    let handle = list_index_argument(args, 1)?;
    let item = args.first().copied().flatten();
    let Some(entry) = list_position(interp, chain, handle)? else {
        let written = interp.to_text(handle);
        return Err(Raised::incorrect_list_index(&written).into());
    };
    write_chain(interp, chain, |slots| slots.set(at(entry, ITEM), item))?;
    Ok(None)
}

/// `List~remove(index)` and `List~delete(index)`, one body: takes the item out,
/// frees the handle and answers the item, or `.nil` for a handle the list does
/// not hold.
fn native_list_remove(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    let handle = item_argument(args)?;
    let Some(entry) = list_position(interp, chain, handle)? else {
        return Ok(Some(ObjRef::NIL));
    };
    list_take(interp, chain, entry)
}

/// Unlinks entry `handle`, which is in use, pushes it on the free stack and
/// answers what it held.
fn unlink(slots: &mut interrexx_core::ArraySlots, handle: usize) -> Option<ObjRef> {
    let chain = Chain(slots);
    let (item, next, previous) = (
        chain.item(handle),
        chain.next(handle),
        chain.previous(handle),
    );
    let count = chain.count();
    match previous {
        Some(previous) => slots.set(at(previous, NEXT), link(next)),
        None => slots.set(HEAD, link(next)),
    }
    match next {
        Some(next) => slots.set(at(next, PREV), link(previous)),
        None => slots.set(TAIL, link(previous)),
    }
    let below = slots[FREE];
    slots.set(at(handle, ITEM), None);
    slots.set(at(handle, NEXT), below);
    slots.set(at(handle, PREV), None);
    slots.set(FREE, link(Some(handle)));
    slots.set(COUNT, stored(count.saturating_sub(1)));
    item
}

/// Removes entry `handle`, frees it and answers the item.
fn list_take(interp: &mut Interp, chain: ObjRef, handle: usize) -> Result<Option<ObjRef>, Failure> {
    let item = write_chain(interp, chain, |slots| {
        Chain(slots).in_use(handle).then(|| unlink(slots, handle))
    })?;
    Ok(Some(item.flatten().unwrap_or(ObjRef::NIL)))
}

/// The first entry from the front whose item is `wanted`, or `None`. Walks
/// the live chain rather than a copy of it, so a search that matches early
/// stops early; the `==` it sends may change the list, after which the walk
/// goes on from the entry's present successor, or from its successor before
/// the send when the entry has gone.
fn list_find(interp: &mut Interp, chain: ObjRef, wanted: ObjRef) -> Result<Option<usize>, Failure> {
    // The entry last compared and its successor as it was before the send.
    let mut last: Option<(usize, Option<usize>)> = None;
    loop {
        let step = read_chain(interp, chain, |chain| {
            let handle = match last {
                None => chain.head(),
                Some((handle, _)) if chain.in_use(handle) => chain.next(handle),
                Some((_, next)) => next.filter(|next| chain.in_use(*next)),
            };
            handle.map(|handle| (handle, chain.item(handle), chain.next(handle)))
        })?;
        let Some((handle, item, next)) = step else {
            return Ok(None);
        };
        if let Some(item) = item
            && same_item(interp, wanted, item)?
        {
            return Ok(Some(handle));
        }
        last = Some((handle, next));
    }
}

/// `List~removeItem(item)`: removes the first entry holding it.
fn native_list_remove_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    let wanted = item_argument(args)?;
    match list_find(interp, chain, wanted)? {
        Some(handle) => list_take(interp, chain, handle),
        None => Ok(Some(ObjRef::NIL)),
    }
}

/// Up to `limit` entries in chain order from entry `from`: each handle number
/// with the item it holds, or `None` for an entry holding nothing.
fn chain_run(chain: Chain<'_>, from: Option<usize>, limit: usize) -> Vec<(usize, Option<ObjRef>)> {
    let mut run = Vec::with_capacity(limit.min(chain.count()));
    let mut cursor = from;
    while let Some(handle) = cursor
        && run.len() < limit
    {
        run.push((handle, chain.item(handle)));
        cursor = chain.next(handle);
    }
    run
}

/// The list's entries in chain order: every handle, each with the item it
/// holds or `None` for an entry holding nothing.
fn list_pairs(
    interp: &mut Interp,
    receiver: ObjRef,
) -> Result<Vec<(ObjRef, Option<ObjRef>)>, Failure> {
    let chain = list_state(interp, receiver)?;
    let run = read_chain(interp, chain, |chain| {
        chain_run(chain, chain.head(), chain.count())
    })?;
    let pairs: Vec<(ObjRef, Option<ObjRef>)> = run
        .into_iter()
        .map(|(handle, item)| (interp.counted(handle), item))
        .collect();
    // Rooted for [`ordered_pairs`]'s reason: a caller sends `==` per pair and
    // the callback may empty the list.
    for (handle, item) in &pairs {
        interp.roots.activity_mut().push_temp(*handle);
        if let Some(item) = *item {
            interp.roots.activity_mut().push_temp(item);
        }
    }
    Ok(pairs)
}

/// A list's items in order, read the way `List~makeArray` reads them when
/// called directly rather than sent.
pub(in crate::dispatch) fn list_items(
    interp: &mut Interp,
    receiver: ObjRef,
) -> Result<Vec<ObjRef>, Failure> {
    let pairs = list_pairs(interp, receiver)?;
    Ok(pairs.into_iter().filter_map(|(_, item)| item).collect())
}

fn native_list_all_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let items = list_pairs(interp, receiver)?
        .into_iter()
        .map(|(_, item)| item)
        .collect();
    Ok(Some(array_of_slots(interp, items)))
}

fn native_list_all_indexes(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let handles = list_pairs(interp, receiver)?
        .into_iter()
        .map(|(handle, _)| handle)
        .collect();
    Ok(Some(array_of(interp, handles)))
}

fn native_list_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    let count = read_chain(interp, chain, |chain| chain.count())?;
    Ok(Some(interp.counted(count)))
}

fn native_list_is_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    let empty = read_chain(interp, chain, |chain| chain.count() == 0)?;
    Ok(Some(crate::eval::logical(empty)))
}

/// `List~empty`: drops every entry, after which the handles start again from
/// zero.
fn native_list_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    write_chain(interp, chain, |slots| {
        slots.clear();
        for _ in 0..HEADER {
            slots.push(link(None));
        }
    })?;
    // Answers the receiver, as `Array~empty` does -- measured, `say l~empty`
    // prints `a List` at rc 0.
    Ok(Some(receiver))
}

/// `List~first`, `~last`, `~firstItem`, `~lastItem`: the handle or the item at
/// either end, or `.nil` for an empty list.
fn list_end(
    interp: &mut Interp,
    receiver: ObjRef,
    take_last: bool,
    want_item: bool,
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    let end = read_chain(interp, chain, |chain| {
        let end = if take_last {
            chain.tail()
        } else {
            chain.head()
        };
        end.map(|handle| (handle, chain.item(handle)))
    })?;
    let Some((handle, item)) = end else {
        return Ok(Some(ObjRef::NIL));
    };
    Ok(Some(match want_item {
        // `firstItem` over an entry holding nothing is `.nil`, not the first
        // entry that holds something -- measured, and likewise `lastItem`.
        true => item.unwrap_or(ObjRef::NIL),
        false => interp.counted(handle),
    }))
}

fn native_list_first(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    list_end(interp, receiver, false, false)
}

fn native_list_last(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    list_end(interp, receiver, true, false)
}

fn native_list_first_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    list_end(interp, receiver, false, true)
}

fn native_list_last_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    list_end(interp, receiver, true, true)
}

/// `List~next(index)` and `List~previous(index)`: the neighbouring handle, or
/// `.nil` past either end and for a handle the list does not hold.
fn list_step(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    forward: bool,
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    let handle = item_argument(args)?;
    let Some(entry) = list_position(interp, chain, handle)? else {
        return Ok(Some(ObjRef::NIL));
    };
    let neighbour = read_chain(interp, chain, |chain| match forward {
        true => chain.next(entry),
        false => chain.previous(entry),
    })?;
    Ok(Some(match neighbour {
        Some(neighbour) => interp.counted(neighbour),
        None => ObjRef::NIL,
    }))
}

fn native_list_next(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    list_step(interp, receiver, args, true)
}

fn native_list_previous(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    list_step(interp, receiver, args, false)
}

/// `List~hasIndex(index)`.
fn native_list_has_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    let handle = item_argument(args)?;
    let held = list_position(interp, chain, handle)?.is_some();
    Ok(Some(crate::eval::logical(held)))
}

/// `List~hasItem(item)`.
fn native_list_has_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    let wanted = item_argument(args)?;
    let found = list_find(interp, chain, wanted)?.is_some();
    Ok(Some(crate::eval::logical(found)))
}

/// `List~index(item)`: the handle of the first entry holding it, or `.nil`.
fn native_list_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    let wanted = item_argument(args)?;
    Ok(Some(match list_find(interp, chain, wanted)? {
        Some(handle) => interp.counted(handle),
        None => ObjRef::NIL,
    }))
}

/// `List~section(index, count)`: a new `List` over that run, which is a
/// `List` and not an `Array`.
fn native_list_section(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let chain = list_state(interp, receiver)?;
    let handle = item_argument(args)?;
    let Some(entry) = list_position(interp, chain, handle)? else {
        let written = interp.to_text(handle);
        return Err(Raised::incorrect_list_index(&written).into());
    };
    let limit = optional_length_argument(interp, args, 1)?.unwrap_or(usize::MAX);
    let run = read_chain(interp, chain, |chain| chain_run(chain, Some(entry), limit))?;
    // Rooted across the `INIT` below, as [`list_pairs`] roots its pairs.
    for (_, item) in &run {
        if let Some(item) = *item {
            interp.roots.activity_mut().push_temp(item);
        }
    }
    // **Always a `List`, and never the receiver's class.** `List~section`
    // always builds a new `List`, where `Array` and `Queue` both build from
    // the receiver's class. Measured, with one subclass of each:
    // `.MyList~of('a','b','c')~section(0,2)~class~id` is `List`, while the
    // `Queue` subclass answers `MYQ` and the `Array` subclass `MYARR`.
    let class = list_scope(interp);
    let section = new_instance(interp, class)?;
    interp.roots.activity_mut().push_temp(section);
    let caller = interp.caller();
    interp.send_message(section, super::INIT, None, &[], caller)?;
    let chain = list_state(interp, section)?;
    write_chain(interp, chain, |slots| {
        let mut tail = Chain(slots).tail();
        for (_, item) in run {
            tail = Some(link_after(slots, tail, item));
        }
    })?;
    Ok(Some(section))
}

/// `List~supplier`: a `Supplier` over the items and their handles.
fn native_list_supplier(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let pairs = list_pairs(interp, receiver)?;
    let items: Vec<Option<ObjRef>> = pairs.iter().map(|(_, item)| *item).collect();
    let handles: Vec<ObjRef> = pairs.into_iter().map(|(handle, _)| handle).collect();
    let items = array_of_slots(interp, items);
    let handles = array_of(interp, handles);
    new_supplier(interp, items, handles).map(Some)
}

/// `List~makeArray`: the `Array` side of the virtual -- items, not handles.
fn native_list_make_array(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    native_list_all_items(interp, cleared, receiver, args)
}

/// `List`'s primitive methods, chained into `ObjectModel::build` beside
/// [`super::super::NATIVE_METHODS`].
pub(in crate::dispatch) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    ("List", "INIT", Arity::Fixed(1), native_list_init),
    ("List", "APPEND", Arity::Fixed(1), native_list_append),
    ("List", "INSERT", Arity::Fixed(2), native_list_insert),
    ("List", "AT", Arity::Fixed(1), native_list_at),
    ("List", "[]", Arity::Fixed(1), native_list_at),
    ("List", "PUT", Arity::Fixed(2), native_list_put),
    ("List", "[]=", Arity::Fixed(2), native_list_put),
    ("List", "REMOVE", Arity::Fixed(1), native_list_remove),
    ("List", "DELETE", Arity::Fixed(1), native_list_remove),
    (
        "List",
        "REMOVEITEM",
        Arity::Fixed(1),
        native_list_remove_item,
    ),
    ("List", "ALLITEMS", Arity::Fixed(0), native_list_all_items),
    (
        "List",
        "ALLINDEXES",
        Arity::Fixed(0),
        native_list_all_indexes,
    ),
    ("List", "ITEMS", Arity::Fixed(0), native_list_items),
    ("List", "ISEMPTY", Arity::Fixed(0), native_list_is_empty),
    ("List", "EMPTY", Arity::Fixed(0), native_list_empty),
    ("List", "FIRST", Arity::Fixed(0), native_list_first),
    ("List", "LAST", Arity::Fixed(0), native_list_last),
    ("List", "FIRSTITEM", Arity::Fixed(0), native_list_first_item),
    ("List", "LASTITEM", Arity::Fixed(0), native_list_last_item),
    ("List", "NEXT", Arity::Fixed(1), native_list_next),
    ("List", "PREVIOUS", Arity::Fixed(1), native_list_previous),
    ("List", "HASINDEX", Arity::Fixed(1), native_list_has_index),
    ("List", "HASITEM", Arity::Fixed(1), native_list_has_item),
    ("List", "INDEX", Arity::Fixed(1), native_list_index),
    ("List", "SECTION", Arity::Fixed(2), native_list_section),
    ("List", "SUPPLIER", Arity::Fixed(0), native_list_supplier),
    ("List", "MAKEARRAY", Arity::Fixed(0), native_list_make_array),
];
