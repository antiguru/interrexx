//! The mapped collections' store, and the two classes that are the store with
//! nothing on top.

mod probe;
use probe::{probe, probe_in};

mod store;
pub(crate) use store::PoolNames;
use store::*;

mod removal;
use removal::{
    Removal, native_hash_empty, native_hash_remove_item, take, take_at, take_in, take_merged,
    take_positions,
};

use std::rc::Rc;

use interrexx_core::NameMap;

use super::collection;
use super::required_string_named_argument;
use super::{
    Arity, ArrayArgument, BehaviourId, Body, Cleared, Decoded, Failure, Interp, Loud, NativeMethod,
    ObjRef, Raised, array_argument, array_argument_object, array_slots, new_instance,
    required_string_argument,
};

// `Relation`'s and `Bag`'s own primitive methods, chained into
// `ObjectModel::build` after this module's own.
pub(super) mod relation;

// `Stem`'s primitive methods, chained into `ObjectModel::build` after this
// module's own.
pub(super) mod stem;

/// The class the store's entries are bound under.
fn hash_scope(interp: &mut Interp) -> ObjRef {
    interp.object_model().table
}

/// The two classes this task owns.
const OWNED: &[&str] = &[
    "Table",
    "IdentityTable",
    "Set",
    "Relation",
    "Bag",
    "Directory",
    "StringTable",
    "Properties",
];

/// The classes whose `put` takes its index from its value --
/// `IndexOnlyHashCollection`, whose only two subclasses are `Set` and `Bag`.
const INDEX_ONLY: &[&str] = &["Set", "Bag"];

/// The classes whose `put` adds a second entry under an existing index instead
/// of replacing it: the multi-value contents, whose `put` adds another entry
/// after every other.
const MULTI_VALUE: &[&str] = &["Relation", "Bag"];

/// The answers of this file's class tests for one class, as bits.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct HashKind(u8);

impl HashKind {
    /// [`OWNED`].
    const OWNED: u8 = 1;
    /// [`INDEX_ONLY`].
    const INDEX_ONLY: u8 = 1 << 1;
    /// [`MULTI_VALUE`].
    const MULTI_VALUE: u8 = 1 << 2;
    /// [`STRING_KEYED`].
    const STRING_KEYED: u8 = 1 << 3;
    /// `IdentityTable`.
    const IDENTITY: u8 = 1 << 4;
    /// [`STORE_HOLDERS`].
    const STORE_HOLDER: u8 = 1 << 5;

    fn has(self, bit: u8) -> bool {
        self.0 & bit != 0
    }
}

/// Each class's [`HashKind`], valid while the class registry's ancestry stamp
/// is `stamp`. Every answer is a function of the superclass lists and the
/// registered names alone, and the stamp moves with either, so a class that
/// `inherit`s, `uninherit`s or is defined at run time is classified afresh.
#[derive(Default)]
pub(crate) struct HashKinds {
    stamp: u64,
    kinds: NameMap<ObjRef, HashKind>,
    /// The class asked about last and its kind, ahead of the map: a loop over
    /// one collection asks about one class several times a message.
    last: Option<(ObjRef, HashKind)>,
}

/// Whether `class` is or inherits from the class registered as one of `names`.
fn class_is_a_named(interp: &mut Interp, class: ObjRef, names: &[&str]) -> bool {
    for name in names {
        let Some(base) = interp.classes().lookup(name) else {
            continue;
        };
        if interp.classes().is_a(class, base) {
            return true;
        }
    }
    false
}

/// `class`'s answers to every class test here, worked out from the class
/// graph.
#[cold]
fn classify(interp: &mut Interp, class: ObjRef) -> HashKind {
    let mut kind = 0;
    for (bit, names) in [
        (HashKind::OWNED, OWNED),
        (HashKind::INDEX_ONLY, INDEX_ONLY),
        (HashKind::MULTI_VALUE, MULTI_VALUE),
        (HashKind::STRING_KEYED, STRING_KEYED),
        (HashKind::IDENTITY, &["IdentityTable"][..]),
    ] {
        if class_is_a_named(interp, class, names) {
            kind |= bit;
        }
    }
    if holds_store_class(interp, class) {
        kind |= HashKind::STORE_HOLDER;
    }
    HashKind(kind)
}

/// `class`'s [`HashKind`], from [`HashKinds`] while the ancestry stamp holds.
fn kind_of_class(interp: &mut Interp, class: ObjRef) -> HashKind {
    let stamp = interp.classes().ancestry_stamp();
    let kinds = &mut interp.hash_kinds;
    if kinds.stamp != stamp {
        kinds.kinds.clear();
        kinds.last = None;
        kinds.stamp = stamp;
    } else if let Some((last, kind)) = kinds.last
        && last == class
    {
        return kind;
    } else if let Some(&kind) = kinds.kinds.get(&class) {
        kinds.last = Some((class, kind));
        return kind;
    }
    let kind = classify(interp, class);
    interp.hash_kinds.kinds.insert(class, kind);
    interp.hash_kinds.last = Some((class, kind));
    kind
}

/// The [`HashKind`] of `receiver`'s class; none of the bits for a value with
/// no class.
fn kind_of(interp: &mut Interp, receiver: ObjRef) -> HashKind {
    // An instance's class is in its body, which is where `class_of_value`
    // would find it after classifying the receiver.
    if let Some(Body::Instance { class, .. }) = interp.heap.get(receiver).map(|object| &object.body)
    {
        let class = *class;
        return kind_of_class(interp, class);
    }
    match interp.class_of_value(receiver) {
        Some(class) => kind_of_class(interp, class),
        None => HashKind::default(),
    }
}

/// Whether `receiver` holds more than one item per index.
fn multi_value(interp: &mut Interp, receiver: ObjRef) -> bool {
    kind_of(interp, receiver).has(HashKind::MULTI_VALUE)
}

/// Whether `receiver`'s `put` is the index-only one.
fn index_only(interp: &mut Interp, receiver: ObjRef) -> bool {
    kind_of(interp, receiver).has(HashKind::INDEX_ONLY)
}

/// Whether `receiver`'s class is one of [`OWNED`].
pub(crate) fn owns(interp: &mut Interp, receiver: ObjRef) -> bool {
    // **A `Body::Native` receiver is never one of these**, however its class
    // reads: it has no store (see [`store_of`]).
    if matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) {
        return false;
    }
    kind_of(interp, receiver).has(HashKind::OWNED)
}

/// Whether `receiver`'s class is or inherits from one of `names`.
fn class_is_one_of(interp: &mut Interp, receiver: ObjRef, names: &[&str]) -> bool {
    let Some(class) = interp.class_of_value(receiver) else {
        return false;
    };
    class_is_a_named(interp, class, names)
}

/// The refusal for a receiver [`owns`] declines: a collection of one of
/// `classes` with no store is the class's own unbuilt method, and any other
/// receiver is of the wrong type, which raises `SYNTAX 93.900`.
fn not_this_task(interp: &mut Interp, receiver: ObjRef, method: &[u8]) -> Failure {
    refusal_outside(
        interp,
        receiver,
        method,
        OWNED,
        "a value that is not a hash collection",
    )
}

/// [`not_this_task`] for the classes `classes`, with `kind` naming what a
/// wrong-type receiver is not.
fn refusal_outside(
    interp: &mut Interp,
    receiver: ObjRef,
    method: &[u8],
    classes: &[&str],
    kind: &str,
) -> Failure {
    if !class_is_one_of(interp, receiver, classes) {
        return crate::Raised::receiver_kind(kind).into();
    }
    match interp.receiver_class_id(receiver) {
        Some(id) => Loud::hash_collection_without_store(method, &id).into(),
        None => Loud::unknown_receiver(kind).into(),
    }
}

/// Records a write to a package's `.ROUTINES` table, which is what that
/// package's routine names are resolved through from then on, and which a call
/// site that kept an answer looked up again on every call has to see.
fn note_routines_write(interp: &mut Interp, receiver: ObjRef) {
    if let Some(package) = interp.routines_table_package(receiver)
        && let Some(data) = interp.package_data_mut(package)
    {
        data.routines_written = true;
        interp.routines_written = true;
        interp.routines_changed();
    }
}

/// The classes whose instances get a store, named rather than derived for
/// [`hash_scope`]'s reason, and resolved once into the object model's
/// `store_holders`. `Properties` is a `Directory` subclass.
pub(super) const STORE_HOLDERS: &[&str] = &[
    "Table",
    "IdentityTable",
    "Set",
    "Bag",
    "Relation",
    "Directory",
    "StringTable",
    "Stem",
];

/// Whether `receiver` is an instance of a class this file gives a store to.
fn holds_hash_store(interp: &mut Interp, receiver: ObjRef) -> bool {
    kind_of(interp, receiver).has(HashKind::STORE_HOLDER)
}

/// Whether `class` is one of [`STORE_HOLDERS`] or a subclass of one.
fn holds_store_class(interp: &mut Interp, class: ObjRef) -> bool {
    let model = interp.object_model();
    model.store_holders.contains(&class)
        || model
            .store_holders
            .iter()
            .any(|&holder| model.classes.is_a(class, holder))
}

// ---- the key protocols ----

/// Which key protocol a receiver's class uses.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Keys {
    /// `IdentityHashContents`: reference identity, hashed by the object's own
    /// hash value rather than by the identity hash, which would lose its
    /// lookups in a saved image.
    Identity,
    /// `EqualityHashContents`: `equalValue`, hashed by `hash()`.
    Equality,
    /// `StringHashContents`: the index's STRING VALUE, compared as bytes and
    /// hashed by the string hash.
    StringValue,
}

/// The classes whose keys are string values.
const STRING_KEYED: &[&str] = &["Directory", "StringTable", "Properties"];

/// Whether `receiver`'s class is one of [`STRING_KEYED`].
pub(crate) fn string_keyed(interp: &mut Interp, receiver: ObjRef) -> bool {
    kind_of(interp, receiver).has(HashKind::STRING_KEYED)
}

fn keys_of(interp: &mut Interp, receiver: ObjRef) -> Keys {
    let kind = kind_of(interp, receiver);
    if kind.has(HashKind::STRING_KEYED) {
        Keys::StringValue
    } else if kind.has(HashKind::IDENTITY) {
        Keys::Identity
    } else {
        Keys::Equality
    }
}

/// An object's own hash value, which is what the identity contents hashes with
/// and what the equality contents falls back on for a base class.
fn get_hash_value(interp: &mut Interp, value: ObjRef) -> u64 {
    super::hash_value(interp, value)
}

/// A `HASHCODE` answer turned back into a number.
fn object_hash_code(text: &[u8]) -> u64 {
    if text.is_empty() {
        return 1;
    }
    if text.len() >= 8 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&text[..8]);
        return u64::from_le_bytes(bytes);
    }
    let low = u16::from(text[0]);
    let high = if text.len() > 1 {
        u16::from(text[1])
    } else {
        0
    };
    i64::from((low | (high << 8)).cast_signed()).cast_unsigned()
}

/// The hash a store looks an index up by.
fn hash_of(interp: &mut Interp, keys: Keys, index: ObjRef) -> Result<u64, Failure> {
    if keys == Keys::StringValue {
        let text = interp.to_text(index).into_owned();
        return Ok(super::string_hash(interp.hash_seed, &text));
    }
    // `r[class.table.indexes.sends]` sends `HASHCODE`; for an index of a base
    // class that runs `Object`'s primitive, which answers this hash value.
    // **"Has a string value" and "is of a base class" are different
    // questions**: an instance of a `String` subclass and an enhanced string
    // have a string value and are not of a base class, so their own `HASHCODE`
    // is sent.
    if keys == Keys::Identity || interp.is_base_class(index) {
        return Ok(get_hash_value(interp, index));
    }
    let caller = interp.caller();
    let answer = interp.send_message(index, b"HASHCODE", None, &[], caller)?;
    let Some(answer) = answer else {
        return Err(Raised::no_result(b"HASHCODE").into());
    };
    let text = interp.to_text(answer).into_owned();
    Ok(object_hash_code(&text))
}

/// Whether the entry at `slot` is the one `index` names.
fn same_index(
    interp: &mut Interp,
    store: &Store,
    keys: Keys,
    slot: usize,
    index: ObjRef,
) -> Result<bool, Failure> {
    let Some(held) = slot_at(interp, store.indexes, slot)? else {
        return Ok(false);
    };
    match keys {
        Keys::Identity => Ok(held == index),
        Keys::Equality => super::collection::same_item(interp, index, held),
        Keys::StringValue => {
            let wanted = interp.to_text(index).into_owned();
            let found = interp.to_text(held).into_owned();
            Ok(wanted == found)
        }
    }
}

/// Whether `wanted` is the held `item` under the receiver's item protocol:
/// identity for [`Keys::Identity`], and `==` for the equality and string
/// contents.
fn same_held_item(
    interp: &mut Interp,
    keys: Keys,
    wanted: ObjRef,
    item: ObjRef,
) -> Result<bool, Failure> {
    match keys {
        Keys::Identity => Ok(wanted == item),
        Keys::Equality | Keys::StringValue => super::collection::same_item(interp, wanted, item),
    }
}

// ---- `Directory`'s method table ----

/// The method table's store, or `None` when `setMethod` has never been sent
/// to this receiver.
fn method_store(interp: &mut Interp, receiver: ObjRef) -> Result<Option<Store>, Failure> {
    read_store(interp, receiver, METHODS)
}

/// The unknown method, or `None` when there is none.
fn unknown_method(interp: &mut Interp, receiver: ObjRef) -> Option<ObjRef> {
    let scope = hash_scope(interp);
    pool_variable(interp, receiver, scope, UNKNOWN_METHOD).filter(|held| *held != ObjRef::NIL)
}

fn set_unknown_method(interp: &mut Interp, receiver: ObjRef, held: Option<ObjRef>) {
    let scope = hash_scope(interp);
    let value = held.unwrap_or(ObjRef::NIL);
    interp.set_pool_variable(receiver, scope, UNKNOWN_METHOD, value);
    bump_store_generation(interp);
}

/// Whether the name is the one `setMethod` keeps outside the table.
fn is_unknown_name(interp: &mut Interp, name: ObjRef) -> bool {
    interp.to_text(name).as_ref() == b"UNKNOWN"
}

/// Runs a stored method with the directory as the receiver.
fn run_stored_method(
    interp: &mut Interp,
    receiver: ObjRef,
    name: &[u8],
    stored: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    // An entry holding an object rather than a method answers that object:
    // `.environment`'s `LOCAL`, which is installed as a method answering the
    // activity's local environment.
    let Some(method) = interp
        .executable_sources
        .get(&stored)
        .and_then(|record| record.installed)
    else {
        return Ok(stored);
    };
    Ok(pinned!(
        interp,
        crate::pinning::PinKind::native(name),
        super::invoke_executable(interp, method, stored, receiver, name, args)
    )?
    .unwrap_or(ObjRef::NIL))
}

/// The result of the method stored under `index`, run with NO arguments and
/// under its own name.
fn method_table_value(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    let Some(store) = method_store(interp, receiver)? else {
        return Ok(None);
    };
    let (store, found) = probe_in(interp, receiver, METHODS, store, index)?;
    let Some(slot) = found.found() else {
        return Ok(None);
    };
    let Some(stored) = slot_at(interp, store.items, slot)? else {
        return Ok(None);
    };
    let name = interp.to_text(index).to_vec();
    run_stored_method(interp, receiver, &name, stored, &[]).map(Some)
}

/// Run under the name `UNKNOWN` and with the index as its ONE argument --
/// unlike a method table entry, which is run with none.
fn unknown_value(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    let Some(stored) = unknown_method(interp, receiver) else {
        return Ok(None);
    };
    run_stored_method(interp, receiver, b"UNKNOWN", stored, &[Some(index)]).map(Some)
}

/// The contents, then the method table, then the unknown method.
fn merged_get(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    let (store, found) = probe(interp, receiver, index)?;
    if let Some(slot) = found.found() {
        return Ok(Some(item_at(interp, &store, slot)?.unwrap_or(ObjRef::NIL)));
    }
    if let Some(value) = method_table_value(interp, receiver, index)? {
        return Ok(Some(value));
    }
    unknown_value(interp, receiver, index)
}

/// `Directory~at` for the native API's `DirectoryAt` and `StringTableAt`,
/// which call the collection rather than send `AT`.
pub(crate) fn store_get(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    merged_get(interp, receiver, index)
}

/// `Directory~remove` for the native API's `DirectoryRemove` and
/// `StringTableRemove`, likewise.
pub(crate) fn store_remove(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    note_routines_write(interp, receiver);
    take_merged(interp, receiver, index, Removal::Answered)
}

/// Either half, and **not** the unknown method. Measured, with an `UNKNOWN`
/// method set: `hasIndex('nosuch')` is 0 while `d['nosuch']` answers what the
/// method returns.
fn merged_has_index(interp: &mut Interp, receiver: ObjRef, index: ObjRef) -> Result<bool, Failure> {
    let (_, found) = probe(interp, receiver, index)?;
    if found.found().is_some() {
        return Ok(true);
    }
    let Some(store) = method_store(interp, receiver)? else {
        return Ok(false);
    };
    let (_, found) = probe_in(interp, receiver, METHODS, store, index)?;
    Ok(found.found().is_some())
}

/// Every (index, stored entry) the method table holds, rooted, none of them
/// run.
fn method_entries(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<(ObjRef, ObjRef)>, Failure> {
    let Some(store) = method_store(interp, receiver)? else {
        return Ok(Vec::new());
    };
    let mut held = Vec::new();
    for slot in entries(interp, &store)? {
        let Some(index) = slot_at(interp, store.indexes, slot)? else {
            continue;
        };
        let Some(stored) = slot_at(interp, store.items, slot)? else {
            continue;
        };
        interp.roots.activity_mut().push_temp(index);
        held.push((index, stored));
    }
    Ok(held)
}

/// The answer of the method table's entry `stored` under `index`, rooted.
fn method_entry_answer(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    stored: ObjRef,
) -> Result<ObjRef, Failure> {
    let name = interp.to_text(index).to_vec();
    let item = run_stored_method(interp, receiver, &name, stored, &[])?;
    interp.roots.activity_mut().push_temp(item);
    Ok(item)
}

/// Every (index, result) the method table answers, in its own insertion
/// order.
fn method_pairs(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<(ObjRef, ObjRef)>, Failure> {
    let held = method_entries(interp, receiver)?;
    let mut pairs = Vec::with_capacity(held.len());
    for (index, stored) in held {
        let item = method_entry_answer(interp, receiver, index, stored)?;
        pairs.push((index, item));
    }
    Ok(pairs)
}

/// How many entries the method table holds, WITHOUT running any of them;
/// `items` adds the two counts. The unknown method is not one of them:
/// measured, a directory holding one ordinary entry and an `UNKNOWN` method
/// answers `items` as 1.
fn method_count(interp: &mut Interp, receiver: ObjRef) -> Result<usize, Failure> {
    let Some(store) = method_store(interp, receiver)? else {
        return Ok(0);
    };
    Ok(count_in(&store))
}

/// How many entries the contents hold: [`entries_of`]'s length.
fn count_in_store(interp: &mut Interp, receiver: ObjRef) -> Result<usize, Failure> {
    match store_if_built(interp, receiver)? {
        Some(store) => Ok(count_in(&store)),
        None => Ok(0),
    }
}

/// The contents' entries and the method table's together, which is what
/// `items` answers, from one read of the pool: [`entries_of`] then
/// [`method_count`], in that order.
fn entry_count(interp: &mut Interp, receiver: ObjRef) -> Result<usize, Failure> {
    let parts = read_parts(interp, receiver);
    // A receiver `store_of` would build an empty store for holds nothing, and
    // is left without one until something is put in it, with the size that
    // store would have had kept.
    let count = match parts.contents.store() {
        Some(store) => count_in(&store),
        None => {
            refuse_storeless(interp, receiver)?;
            if parts.contents.slots.is_none() {
                keep_capacity(interp, receiver, MINIMUM_ENTRIES);
            }
            0
        }
    };
    match parts.methods.store() {
        Some(methods) => Ok(count + count_in(&methods)),
        None => Ok(count),
    }
}

// ---- the shared surface ----

/// The index argument the store's index-taking methods require.
fn index_argument(args: &[Option<ObjRef>], position: usize) -> Result<ObjRef, Failure> {
    args.get(position - 1)
        .copied()
        .flatten()
        .ok_or_else(|| Raised::missing_named_argument("index").into())
}

/// `validateIndex`: a string-keyed collection takes its index as
/// `stringArgument(index, "index")`, and every other collection takes it as
/// given.
fn validated_index(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<ObjRef, Failure> {
    if keys_of(interp, receiver) == Keys::StringValue {
        return super::required_string_named_argument(interp, index, "index");
    }
    Ok(index)
}

/// `~at` and `[]`: the item under that index, or `.nil`.
#[inline(always)]
pub(super) fn store_at(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let index = index_argument(args, 1)?;
    let index = validated_index(interp, receiver, index)?;
    Ok(Some(
        merged_get(interp, receiver, index)?.unwrap_or(ObjRef::NIL),
    ))
}

/// `~put` and `[]=`: **the item comes first and the index second**, the
/// opposite of the reading order of `t['k'] = 'v'`.
pub(super) fn store_put(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    note_routines_write(interp, receiver);
    let Some(item) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("item").into());
    };
    // **A `Set`'s index is its value.**
    // `IndexOnlyHashCollection::validateValueIndex`: the value is required,
    // the index is optional, and an index that is given must equal the value
    // or the send is 93.949. Measured: `s~put('a')` twice leaves `items` at 1,
    // `s~put('b','b')` is accepted, and `s~put('c','d')` raises.
    let index = if index_only(interp, receiver) {
        match args.get(1).copied().flatten() {
            // The index-equality test is `isIndexEqual(value, index)` -- the
            // VALUE receives the `==`.
            Some(given) if !super::collection::same_item(interp, item, given)? => {
                return Err(Raised::index_does_not_match().into());
            }
            _ => item,
        }
    } else {
        let index = index_argument(args, 2)?;
        validated_index(interp, receiver, index)?
    };
    if multi_value(interp, receiver) {
        insert_duplicate(interp, receiver, index, Some(item))?;
    } else {
        insert(interp, receiver, index, Some(item))?;
    }
    Ok(None)
}

/// Every (index, item) pair the receiver holds, in insertion order.
fn pairs(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<(ObjRef, ObjRef)>, Failure> {
    let mut pairs = stored_pairs(interp, receiver)?;
    // `allIndexes` and its siblings append the method table AFTER the
    // contents, each half in its own insertion order: a directory given
    // `AAA`(method), `zzz`, `MMM`(method), `bbb` answers `zzz bbb AAA MMM`.
    pairs.extend(method_pairs(interp, receiver)?);
    Ok(pairs)
}

/// [`pairs`] without the method table: the contents only, in insertion order.
fn stored_pairs(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<(ObjRef, ObjRef)>, Failure> {
    let store = store_of(interp, receiver)?;
    let mut pairs = Vec::new();
    for slot in entries_of(interp, receiver)? {
        let Some(index) = slot_at(interp, store.indexes, slot)? else {
            continue;
        };
        let item = item_at(interp, &store, slot)?.unwrap_or(ObjRef::NIL);
        // Rooted for `ordered_pairs`'s reason: a caller sends `==` per pair
        // and the callback may empty the collection.
        interp.roots.activity_mut().push_temp(index);
        interp.roots.activity_mut().push_temp(item);
        pairs.push((index, item));
    }
    Ok(pairs)
}

/// The first (index, item) whose item `wanted` is equal to, the stored
/// items visited before the answers of the methods, and a method run only
/// when no stored item was equal and no earlier method's answer was. The
/// bool tells whether the match is a method's answer.
fn search_item(
    interp: &mut Interp,
    receiver: ObjRef,
    wanted: ObjRef,
) -> Result<Option<(ObjRef, ObjRef, bool)>, Failure> {
    let keys = keys_of(interp, receiver);
    for (index, item) in stored_pairs(interp, receiver)? {
        if same_held_item(interp, keys, wanted, item)? {
            return Ok(Some((index, item, false)));
        }
    }
    for (index, stored) in method_entries(interp, receiver)? {
        let item = method_entry_answer(interp, receiver, index, stored)?;
        if same_held_item(interp, keys, wanted, item)? {
            return Ok(Some((index, item, true)));
        }
    }
    Ok(None)
}

/// Every index the receiver holds, in [`pairs`]'s order, **running no
/// method**: the method table's names are appended after the contents, and are
/// read as names only.
fn indexes(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<ObjRef>, Failure> {
    let mut indexes = Vec::new();
    let store = store_of(interp, receiver)?;
    let methods = method_store(interp, receiver)?;
    for half in std::iter::once(store).chain(methods) {
        for slot in entries(interp, &half)? {
            if let Some(index) = slot_at(interp, half.indexes, slot)? {
                interp.roots.activity_mut().push_temp(index);
                indexes.push(index);
            }
        }
    }
    Ok(indexes)
}

fn native_hash_all_indexes(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ALLINDEXES"));
    }
    let indexes = indexes(interp, receiver)?;
    Ok(Some(super::collection::array_of(interp, indexes)))
}

fn native_hash_all_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ALLITEMS"));
    }
    let items = pairs(interp, receiver)?
        .into_iter()
        .map(|(_, item)| item)
        .collect();
    Ok(Some(super::collection::array_of(interp, items)))
}

/// `makeArray` answers the INDEXES, where `Array`'s answers the items.
/// Measured, a `.Directory` holding `k1` and `k2` answers `makeArray` as
/// `k1 k2` against `allItems` as `v1 v2`.
fn native_hash_make_array(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"MAKEARRAY"));
    }
    native_hash_all_indexes(interp, cleared, receiver, args)
}

fn native_hash_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ITEMS"));
    }
    let count = entry_count(interp, receiver)?;
    Ok(Some(interp.counted(count)))
}

fn native_hash_is_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ISEMPTY"));
    }
    let empty = count_in_store(interp, receiver)? == 0 && method_count(interp, receiver)? == 0;
    Ok(Some(crate::eval::logical(empty)))
}

fn native_hash_has_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"HASINDEX"));
    }
    let index = index_argument(args, 1)?;
    let index = validated_index(interp, receiver, index)?;
    let held = merged_has_index(interp, receiver, index)?;
    Ok(Some(crate::eval::logical(held)))
}

fn native_hash_has_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"HASITEM"));
    }
    let wanted = super::collection::item_argument(args)?;
    let found = search_item(interp, receiver, wanted)?;
    Ok(Some(crate::eval::logical(found.is_some())))
}

fn native_hash_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"INDEX"));
    }
    let wanted = super::collection::item_argument(args)?;
    if index_only(interp, receiver) {
        // `IndexOnlyHashCollection` looks the item up as an index -- one
        // `hashCode` to it, `==` against the stored indexes of that hash --
        // and answers the stored object, `.nil` for none.
        let (store, found) = probe(interp, receiver, wanted)?;
        let Some(slot) = found.found() else {
            return Ok(Some(ObjRef::NIL));
        };
        return Ok(Some(
            slot_at(interp, store.items, slot)?.unwrap_or(ObjRef::NIL),
        ));
    }
    let found = search_item(interp, receiver, wanted)?;
    Ok(Some(found.map_or(ObjRef::NIL, |(index, _, _)| index)))
}

fn native_hash_remove(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    note_routines_write(interp, receiver);
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"REMOVE"));
    }
    let index = index_argument(args, 1)?;
    let index = validated_index(interp, receiver, index)?;
    Ok(Some(
        take_merged(interp, receiver, index, Removal::Answered)?.unwrap_or(ObjRef::NIL),
    ))
}

fn native_hash_supplier(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"SUPPLIER"));
    }
    let held = pairs(interp, receiver)?;
    let items: Vec<ObjRef> = held.iter().map(|(_, item)| *item).collect();
    let indexes: Vec<ObjRef> = held.into_iter().map(|(index, _)| index).collect();
    let items = super::collection::array_of(interp, items);
    let indexes = super::collection::array_of(interp, indexes);
    super::collection::new_supplier(interp, items, indexes).map(Some)
}

/// The constructor of `Table` and its siblings: an instance with no store yet
/// and `INIT` with every argument. The store is sized by `INIT`
/// ([`initialize_store`]), or at the default capacity on first use by a
/// subclass whose `INIT` does not forward.
pub(super) fn native_hash_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<super::NativeStarted, Failure> {
    let class = super::class_receiver(interp, receiver)?;
    let object = new_instance(interp, class)?;
    super::begin_init(interp, object, args)
}

/// the store sized for `capacity`, unless the receiver already has one. The
/// capacity is a sizing hint only: entries come back in insertion order
/// whatever it is.
pub(super) fn initialize_store(
    interp: &mut Interp,
    receiver: ObjRef,
    capacity: usize,
) -> Result<(), Failure> {
    let scope = hash_scope(interp);
    let class = match interp.heap.get(receiver).map(|object| &object.body) {
        Some(Body::Instance { class, pools, .. })
            if pools.get(scope, HASH_INDEXES).is_none()
                && pools.get(scope, HASH_BUCKETS).is_none() =>
        {
            *class
        }
        _ => return Ok(()),
    };
    if kind_of_class(interp, class).has(HashKind::STORE_HOLDER) {
        // Only the capacity is kept: [`store_of`] builds the store at that
        // size when something first needs one, and a read that finds nothing
        // answers the same without it.
        keep_capacity(interp, receiver, capacity);
    }
    Ok(())
}

// ---- the string-keyed classes' own accessors ----

/// The name an entry method looks up, which is **upper-cased** where the
/// index family's is not.
fn entry_name(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    let Some(name) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("index").into());
    };
    let name = super::required_string_named_argument(interp, name, "index")?;
    let upper = interp.to_text(name).to_ascii_uppercase();
    // Rooted: the store it is looked up or put in may be built or grown
    // first, which allocates.
    let index = interp.text_built(upper);
    interp.roots.activity_mut().push_temp(index);
    Ok(index)
}

/// Every index the receiver's store holds, in the store's own order, or an
/// empty list for a receiver that has no store.
pub(crate) fn store_indexes(interp: &mut Interp, receiver: ObjRef) -> Vec<ObjRef> {
    let scope = hash_scope(interp);
    if pool_variable(interp, receiver, scope, HASH_INDEXES).is_none() {
        return Vec::new();
    }
    let Ok(store) = store_of(interp, receiver) else {
        return Vec::new();
    };
    let Ok(order) = entries_of(interp, receiver) else {
        return Vec::new();
    };
    order
        .into_iter()
        .filter_map(|slot| slot_at(interp, store.indexes, slot).ok().flatten())
        .collect()
}

/// [`insert`] for a caller that built `receiver` and validated `index`
/// itself, without a `PUT` send.
pub(crate) fn store_insert(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    item: ObjRef,
) -> Result<(), Failure> {
    insert(interp, receiver, index, Some(item))
}

/// The item the receiver's store holds under `index`.
pub(crate) fn store_item(interp: &mut Interp, receiver: ObjRef, index: ObjRef) -> Option<ObjRef> {
    let (store, found) = probe(interp, receiver, index).ok()?;
    let slot = found.found()?;
    slot_at(interp, store.items, slot).ok().flatten()
}

/// A fresh `Directory` whose store is sized for `capacity` entries.
pub(crate) fn new_directory(interp: &mut Interp, capacity: usize) -> Result<ObjRef, Failure> {
    let class = interp.object_model().directory;
    let directory = new_instance(interp, class)?;
    install_store(interp, directory, CONTENTS, capacity);
    Ok(directory)
}

/// A fresh `StringTable` with the store `.StringTable~new` gives one.
pub(crate) fn new_string_table(interp: &mut Interp) -> Result<ObjRef, Failure> {
    let class = interp.object_model().string_table;
    let table = new_instance(interp, class)?;
    install_store(interp, table, CONTENTS, MINIMUM_ENTRIES);
    Ok(table)
}

/// `Directory~put` of `item` under the string `name`.
pub(crate) fn directory_put(
    interp: &mut Interp,
    directory: ObjRef,
    name: &[u8],
    item: ObjRef,
) -> Result<(), Failure> {
    let index = interp.text(name);
    interp.roots.activity_mut().push_temp(index);
    insert(interp, directory, index, Some(item))
}

/// Removes `name`'s entry from `directory` without reading the item it held.
pub(crate) fn directory_remove(
    interp: &mut Interp,
    directory: ObjRef,
    name: &[u8],
) -> Result<(), Failure> {
    let index = interp.text(name);
    interp.roots.activity_mut().push_temp(index);
    take_merged(interp, directory, index, Removal::Discarded).map(|_| ())
}

/// `put` of `item` under the string `name`, run for the native API's
/// `DirectoryPut` and `StringTablePut` on any collection this file owns: a
/// multi-value one adds another entry, as its contents' `put` does, and a
/// `Directory` runs its own `put`.
pub(crate) fn native_put(
    interp: &mut Interp,
    receiver: ObjRef,
    name: &[u8],
    item: ObjRef,
) -> Result<(), Failure> {
    note_routines_write(interp, receiver);
    let index = interp.text(name);
    interp.roots.activity_mut().push_temp(index);
    if multi_value(interp, receiver) {
        insert_duplicate(interp, receiver, index, Some(item))
    } else {
        insert(interp, receiver, index, Some(item))
    }
}

/// Puts `value` into the method-table half under `name`, as an entry that
/// answers `value` when run.
pub(crate) fn directory_put_method_value(
    interp: &mut Interp,
    directory: ObjRef,
    name: &[u8],
    value: ObjRef,
) -> Result<(), Failure> {
    let index = interp.text(name);
    interp.roots.activity_mut().push_temp(index);
    insert_in(interp, directory, METHODS, index, Some(value))
}

/// What `Directory~get` finds for a name.
pub(crate) enum DirectoryEntry {
    Found(ObjRef),
    /// An entry the reference run holds that this crate does not build.
    Owed,
    Absent,
}

/// A reading of one directory's pool entries: both halves and the unknown
/// method, valid while no store's pool entries have been written since.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct StoreView {
    generation: u64,
    contents: Store,
    methods: Option<Store>,
    unknown: Option<ObjRef>,
}

fn read_view(interp: &mut Interp, directory: ObjRef) -> Result<StoreView, Failure> {
    let parts = read_parts(interp, directory);
    let contents = match parts.contents.store() {
        Some(store) => store,
        None => store_of(interp, directory)?,
    };
    let methods = parts.methods.store();
    Ok(StoreView {
        generation: interp.store_generation,
        contents,
        methods,
        unknown: parts.unknown,
    })
}

/// A name a string-keyed store is searched for, with its string hash.
#[derive(Clone, Copy)]
pub(crate) struct Key<'a> {
    name: &'a [u8],
    hash: u64,
}

impl<'a> Key<'a> {
    pub(crate) fn new(seed: interrexx_core::HashSeed, name: &'a [u8]) -> Key<'a> {
        Key {
            name,
            hash: super::string_hash(seed, name),
        }
    }
}

/// `Directory~get` over a string-keyed store, by the name's bytes: the
/// contents, then the method table, then the unknown method.
///
/// `view` is a reading of `directory`'s pool from an earlier call, used when
/// still valid. Returns the answer, and the reading this call made when it
/// could not use `view`.
pub(crate) fn directory_get(
    interp: &mut Interp,
    directory: ObjRef,
    key: Key<'_>,
    view: Option<&StoreView>,
) -> Result<(DirectoryEntry, Option<StoreView>), Failure> {
    if let Some(view) = view
        && view.generation == interp.store_generation
    {
        debug_assert_eq!(
            Some(*view),
            unshared!(interp, read_view(interp, directory).ok()),
            "a StoreView was reused after its directory's pool changed without a \
             bump_store_generation"
        );
        return Ok((view_get(interp, directory, key, view)?, None));
    }
    let view = read_view(interp, directory)?;
    Ok((view_get(interp, directory, key, &view)?, Some(view)))
}

/// [`directory_get`] for a package local, kept apart from it so that the
/// `.local` and `.environment` lookup is its one caller.
pub(crate) fn package_local_get(
    interp: &mut Interp,
    local: ObjRef,
    key: Key<'_>,
    view: Option<&StoreView>,
) -> Result<(DirectoryEntry, Option<StoreView>), Failure> {
    if let Some(view) = view
        && view.generation == interp.store_generation
    {
        debug_assert_eq!(
            Some(*view),
            unshared!(interp, read_view(interp, local).ok()),
            "a StoreView was reused after its directory's pool changed without a \
             bump_store_generation"
        );
        return Ok((view_get(interp, local, key, view)?, None));
    }
    let view = read_view(interp, local)?;
    Ok((view_get(interp, local, key, &view)?, Some(view)))
}

fn view_get(
    interp: &mut Interp,
    directory: ObjRef,
    key: Key<'_>,
    view: &StoreView,
) -> Result<DirectoryEntry, Failure> {
    let name = key.name;
    let store = view.contents;
    if let Some(slot) = text_slot(interp, &store, key)? {
        let item = slot_at(interp, store.items, slot)?.unwrap_or(ObjRef::NIL);
        return Ok(if interp.is_owed_entry(item) {
            DirectoryEntry::Owed
        } else {
            DirectoryEntry::Found(item)
        });
    }
    if let Some(methods) = view.methods
        && let Some(slot) = text_slot(interp, &methods, key)?
        && let Some(stored) = slot_at(interp, methods.items, slot)?
    {
        return run_stored_method(interp, directory, name, stored, &[]).map(DirectoryEntry::Found);
    }
    let Some(stored) = view.unknown else {
        return Ok(DirectoryEntry::Absent);
    };
    let index = interp.text(name);
    interp.roots.activity_mut().push_temp(index);
    run_stored_method(interp, directory, b"UNKNOWN", stored, &[Some(index)])
        .map(DirectoryEntry::Found)
}

/// The position holding `name` in a string-keyed store, found without
/// building an index object.
fn text_slot(interp: &mut Interp, store: &Store, key: Key<'_>) -> Result<Option<usize>, Failure> {
    probe::find_text(interp, store, code_of(key.hash), key.name)
}

/// Whether `table` still holds an entry this crate does not build.
pub(crate) fn holds_owed_entry(interp: &mut Interp, table: ObjRef) -> bool {
    let Ok(Some(store)) = read_store(interp, table, CONTENTS) else {
        return false;
    };
    let Ok(order) = entries(interp, &store) else {
        return false;
    };
    order.into_iter().any(|slot| {
        matches!(slot_at(interp, store.items, slot), Ok(Some(item)) if interp.is_owed_entry(item))
    })
}

/// The `UNKNOWN` entry read, over the store rather than the environment's map.
pub(super) fn store_entry_read(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(
        merged_get(interp, receiver, index)?.unwrap_or(ObjRef::NIL),
    ))
}

/// [`store_entry_read`]'s other half.
pub(super) fn store_entry_write(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    item: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    note_routines_write(interp, receiver);
    insert(interp, receiver, index, Some(item))?;
    Ok(None)
}

/// The index argument a hash-collection method was given, as the bytes it is
/// stored and looked up under.
fn hash_index(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Vec<u8>, Failure> {
    let Some(Some(argument)) = args.get(position - 1).copied() else {
        return Err(Raised::missing_named_argument("index").into());
    };
    let argument = required_string_named_argument(interp, argument, "index")?;
    Ok(interp.to_text(argument).to_vec())
}

/// `~at(index)` and `~[index]` on a `Directory` or a `StringTable`: the entry
/// stored under `index`, or `.nil`. Both classes are given the same method as
/// the other hash collections.
pub(super) fn native_hash_at(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    hash_at(interp, receiver, args, b"AT")
}

/// `~[](index)`, [`native_hash_at`] under its own name.
pub(super) fn native_hash_bracket_at(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    hash_at(interp, receiver, args, b"[]")
}

/// [`native_hash_at`] sent as `name`.
#[inline(always)]
fn hash_at(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    name: &[u8],
) -> Result<Option<ObjRef>, Failure> {
    // **One body, two stores.** `IdentityTable`'s `At`, `Put` and `[]` rows are
    // donated to `StringTable` and that whole set on to `Directory`, so
    // `Table`, `IdentityTable`, `StringTable` and `Directory` share one method
    // identity. A table this crate built on `NativeObject`'s map reads that
    // map; everything else reads the store.
    if !matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) && owns(interp, receiver)
    {
        // An index not of a base class may hash and compare through Rexx
        // methods of its own.
        if args
            .first()
            .copied()
            .flatten()
            .is_some_and(|index| interp.may_run_rexx(index))
        {
            return leveled_store_at(interp, receiver, args, name);
        }
        return store_at(interp, receiver, args);
    }
    let index = hash_index(interp, args, 1)?;
    Ok(Some(interp.hash_entry_read(receiver, &index)))
}

/// [`store_at`] as the primitive `name`'s own level, for an index whose
/// `HASHCODE` and `==` may be Rexx code.
#[cold]
#[inline(never)]
fn leveled_store_at(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    name: &[u8],
) -> Result<Option<ObjRef>, Failure> {
    interp.primitive_level(receiver, name, args, |interp| {
        store_at(interp, receiver, args)
    })
}

/// `~put(item, index)` on a `Directory` or a `StringTable`: stores `item`
/// under `index`, replacing whatever was there.
pub(super) fn native_hash_put(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    hash_put(interp, receiver, args, b"PUT")
}

/// `~[]=(item, index)`, [`native_hash_put`] under its own name.
pub(super) fn native_hash_bracket_put(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    hash_put(interp, receiver, args, b"[]=")
}

/// [`native_hash_put`] sent as `name`.
fn hash_put(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    name: &[u8],
) -> Result<Option<ObjRef>, Failure> {
    // [`native_hash_at`]'s split, for the same reason.
    if !matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) && owns(interp, receiver)
    {
        // An index not of a base class may hash and compare through Rexx
        // methods of its own, and so may a `Set`'s or `Bag`'s item.
        let item = args.first().copied().flatten();
        let plain = match args.get(1).copied().flatten() {
            None => item.is_none_or(|item| interp.is_base_class(item)),
            Some(index) => {
                interp.is_base_class(index)
                    && (!index_only(interp, receiver)
                        || item.is_none_or(|item| interp.is_base_class(item)))
            }
        };
        if plain {
            return store_put(interp, receiver, args);
        }
        return interp.primitive_level(receiver, name, args, |interp| {
            store_put(interp, receiver, args)
        });
    }
    let Some(Some(item)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("item").into());
    };
    let index = hash_index(interp, args, 2)?;
    interp.hash_entry_write(receiver, &index, item)?;
    Ok(None)
}

/// `~unknown(message, arguments)` on a `Directory` or a `StringTable`: **the
/// entry-method mechanism**, `StringHashCollection::unknown`.
pub(super) fn native_hash_unknown(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    interp.primitive_level(receiver, b"UNKNOWN", args, |interp| {
        hash_unknown(interp, receiver, args)
    })
}

/// [`hash_unknown`] for a message `name` that does not end in `=`, sent with
/// whatever arguments, for a receiver whose `UNKNOWN` is that primitive: the
/// read, which uses neither the forwarded arguments nor the message name as
/// an object, so the caller builds neither.
pub(super) fn unknown_entry_read(
    interp: &mut Interp,
    receiver: ObjRef,
    name: &[u8],
) -> Result<Option<ObjRef>, Failure> {
    let store = !matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) && owns(interp, receiver);
    let mut upper = interp.take_result_buffer();
    upper.extend_from_slice(name);
    upper.make_ascii_uppercase();
    entry_read_upper(interp, receiver, upper, store)
}

/// The entry an upper-cased `name` reads: from the store when `store`, and
/// otherwise from `NativeObject`'s map.
fn entry_read_upper(
    interp: &mut Interp,
    receiver: ObjRef,
    name: Vec<u8>,
    store: bool,
) -> Result<Option<ObjRef>, Failure> {
    if store {
        // Rooted for the write's reason in [`hash_unknown`]: a read may build
        // the store.
        let index = interp.text_built(name);
        interp.roots.activity_mut().push_temp(index);
        return store_entry_read(interp, receiver, index);
    }
    Ok(Some(interp.hash_entry_read(receiver, &name)))
}

/// [`native_hash_unknown`]'s work, which runs without a send too.
pub(super) fn hash_unknown(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(message)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let message = required_string_argument(interp, message, 1)?;
    // Built in the result buffer, which the short index it becomes hands
    // back, rather than in a fresh allocation per send.
    let mut name = interp.take_result_buffer();
    name.extend_from_slice(&interp.to_text(message));
    let Some(Some(arguments)) = args.get(1).copied() else {
        return Err(Raised::missing_method_argument(2).into());
    };
    let forwarded = array_argument_object(interp, arguments, ArrayArgument::Positional)?;
    // [`native_hash_at`]'s split again: a store or `NativeObject`'s map.
    // Measured, a `.Directory` given
    // `setEntry('alpha', 42)` answers `d~alpha` as `42`, and `d~beta = 7`
    // stores under `BETA`.
    let store = !matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) && owns(interp, receiver);
    name.make_ascii_uppercase();
    if name.pop_if(|last| *last == b'=').is_none() {
        return entry_read_upper(interp, receiver, name, store);
    }
    let index = name;
    let Some(Some(item)) = interp
        .array_slots(forwarded)
        .and_then(|slots| slots.first().copied())
    else {
        return Err(Loud::entry_method_without_a_value(&index).into());
    };
    if store {
        // **Rooted**: a long name is an object of its own, and putting it in
        // may build or grow the store, which allocates. The caller's frame
        // pops it.
        let index = interp.text_built(index);
        interp.roots.activity_mut().push_temp(index);
        return store_entry_write(interp, receiver, index, item);
    }
    interp.hash_entry_write(receiver, &index, item)?;
    Ok(None)
}

fn native_hash_entry(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ENTRY"));
    }
    let name = entry_name(interp, args)?;
    Ok(Some(
        merged_get(interp, receiver, name)?.unwrap_or(ObjRef::NIL),
    ))
}

fn native_hash_has_entry(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"HASENTRY"));
    }
    let name = entry_name(interp, args)?;
    let held = merged_has_index(interp, receiver, name)?;
    Ok(Some(crate::eval::logical(held)))
}

/// `~setEntry(name [, value])`: stores under the upper-cased name, and
/// **removes the entry when the value is omitted** -- measured,
/// `d~setEntry('beta')` after `d~setEntry('beta', 2)` leaves `items` at 0.
fn native_hash_set_entry(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    note_routines_write(interp, receiver);
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"SETENTRY"));
    }
    let name = entry_name(interp, args)?;
    match args.get(1).copied().flatten() {
        Some(value) => insert(interp, receiver, name, Some(value))?,
        None => {
            take_merged(interp, receiver, name, Removal::Discarded)?;
        }
    }
    Ok(None)
}

/// attaches a method whose RESULT is the entry's item, recomputed on every
/// read.
fn native_directory_set_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    note_routines_write(interp, receiver);
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"SETMETHOD"));
    }
    let name = entry_name(interp, args)?;
    interp.roots.activity_mut().push_temp(name);
    let unknown = is_unknown_name(interp, name);
    match args.get(1).copied().flatten() {
        // The `Method` object is built with the entry's name, and the table
        // holds it: it is compiled under the entry's name, which is what
        // `.context~executable` answers when the entry runs.
        Some(source) => {
            let object = if interp.receiver_kind(source) == Ok(super::Primitive::Method) {
                source
            } else {
                let compiled = interp.to_text(name).to_vec();
                super::compile_method_source(
                    interp,
                    &compiled,
                    source,
                    "method",
                    super::SourceTaker::Method,
                )?
            };
            let (stored, _) = super::scoped_method(interp, object, ObjRef::NIL)?;
            interp.roots.activity_mut().push_temp(stored);
            if unknown {
                set_unknown_method(interp, receiver, Some(stored));
            } else {
                insert_in(interp, receiver, METHODS, name, Some(stored))?;
            }
        }
        None if unknown => set_unknown_method(interp, receiver, None),
        None => {
            take_in(interp, receiver, METHODS, name)?;
        }
    }
    take(interp, receiver, name)?;
    Ok(None)
}

/// Drops a method, and unlike `setMethod` leaves the contents alone.
fn native_directory_unset_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    note_routines_write(interp, receiver);
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"UNSETMETHOD"));
    }
    let name = entry_name(interp, args)?;
    if is_unknown_name(interp, name) {
        set_unknown_method(interp, receiver, None);
    } else {
        take_in(interp, receiver, METHODS, name)?;
    }
    Ok(None)
}

fn native_hash_remove_entry(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    note_routines_write(interp, receiver);
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"REMOVEENTRY"));
    }
    let name = entry_name(interp, args)?;
    Ok(Some(
        take_merged(interp, receiver, name, Removal::Answered)?.unwrap_or(ObjRef::NIL),
    ))
}

// ---- `Relation` and `Bag`'s own surface ----

/// `Bag~of(item, ...)`: unlike `Set~of`, keeps the duplicates -- measured,
/// `.Bag~of('p','p','q')~items` is 3.
pub(super) fn native_bag_of(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = super::class_receiver(interp, receiver)?;
    for (at, argument) in args.iter().enumerate() {
        if argument.is_none() {
            return Err(Raised::missing_method_argument(at + 1).into());
        }
    }
    let object = new_instance(interp, class)?;
    interp.roots.activity_mut().push_temp(object);
    install_store(interp, object, CONTENTS, MINIMUM_ENTRIES);
    let caller = interp.caller();
    interp.send_message(object, super::INIT, None, &[], caller)?;
    for argument in args.iter().flatten() {
        insert_duplicate(interp, object, *argument, Some(*argument))?;
    }
    Ok(Some(object))
}

/// A new `Set` of the receiver's own class holding the arguments, each of
/// which is its own index.
pub(super) fn native_set_of(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = super::class_receiver(interp, receiver)?;
    for (at, argument) in args.iter().enumerate() {
        if argument.is_none() {
            return Err(Raised::missing_method_argument(at + 1).into());
        }
    }
    let object = new_instance(interp, class)?;
    interp.roots.activity_mut().push_temp(object);
    install_store(interp, object, CONTENTS, MINIMUM_ENTRIES);
    let caller = interp.caller();
    interp.send_message(object, super::INIT, None, &[], caller)?;
    for argument in args.iter().flatten() {
        insert(interp, object, *argument, Some(*argument))?;
    }
    Ok(Some(object))
}

/// The mapped collections' primitive methods, chained into
/// `ObjectModel::build` ahead of [`relation::NATIVE_METHODS`].
pub(super) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    (
        "Table",
        "ALLINDEXES",
        Arity::Fixed(0),
        native_hash_all_indexes,
    ),
    ("Table", "ALLITEMS", Arity::Fixed(0), native_hash_all_items),
    (
        "Table",
        "MAKEARRAY",
        Arity::Fixed(0),
        native_hash_make_array,
    ),
    ("Table", "ITEMS", Arity::Fixed(0), native_hash_items),
    ("Table", "ISEMPTY", Arity::Fixed(0), native_hash_is_empty),
    ("Table", "EMPTY", Arity::Fixed(0), native_hash_empty),
    ("Table", "HASINDEX", Arity::Fixed(1), native_hash_has_index),
    ("Table", "HASITEM", Arity::Fixed(1), native_hash_has_item),
    ("Table", "INDEX", Arity::Fixed(1), native_hash_index),
    ("Table", "REMOVE", Arity::Fixed(1), native_hash_remove),
    (
        "Table",
        "REMOVEITEM",
        Arity::Fixed(1),
        native_hash_remove_item,
    ),
    ("Table", "SUPPLIER", Arity::Fixed(0), native_hash_supplier),
    (
        "IdentityTable",
        "ALLINDEXES",
        Arity::Fixed(0),
        native_hash_all_indexes,
    ),
    (
        "IdentityTable",
        "ALLITEMS",
        Arity::Fixed(0),
        native_hash_all_items,
    ),
    (
        "IdentityTable",
        "MAKEARRAY",
        Arity::Fixed(0),
        native_hash_make_array,
    ),
    ("IdentityTable", "ITEMS", Arity::Fixed(0), native_hash_items),
    (
        "IdentityTable",
        "ISEMPTY",
        Arity::Fixed(0),
        native_hash_is_empty,
    ),
    ("IdentityTable", "EMPTY", Arity::Fixed(0), native_hash_empty),
    (
        "IdentityTable",
        "HASINDEX",
        Arity::Fixed(1),
        native_hash_has_index,
    ),
    (
        "IdentityTable",
        "HASITEM",
        Arity::Fixed(1),
        native_hash_has_item,
    ),
    ("IdentityTable", "INDEX", Arity::Fixed(1), native_hash_index),
    (
        "IdentityTable",
        "REMOVE",
        Arity::Fixed(1),
        native_hash_remove,
    ),
    (
        "IdentityTable",
        "REMOVEITEM",
        Arity::Fixed(1),
        native_hash_remove_item,
    ),
    (
        "IdentityTable",
        "SUPPLIER",
        Arity::Fixed(0),
        native_hash_supplier,
    ),
    // **`Set`'s two overrides route to the INDEX bodies.** `Set`'s `HasItem`
    // and `RemoveItem` use the index-based bodies, for a collection whose
    // index is its value: searching by index is the faster operation and
    // answers the same thing. Rows of their own here because they are method
    // identities of their own -- `Set~hasItem` is not `Table~hasItem`, which
    // is why adding `Set` to [`OWNED`] alone left it refusing.
    ("Set", "HASITEM", Arity::Fixed(1), native_hash_has_index),
    ("Set", "REMOVEITEM", Arity::Fixed(1), native_hash_remove),
    // `StringHashCollection`'s four, which read the same store the index
    // family reads and upper-case the name on the way in.
    ("Directory", "ENTRY", Arity::Fixed(1), native_hash_entry),
    (
        "Directory",
        "HASENTRY",
        Arity::Fixed(1),
        native_hash_has_entry,
    ),
    (
        "Directory",
        "SETENTRY",
        Arity::Fixed(2),
        native_hash_set_entry,
    ),
    (
        "Directory",
        "REMOVEENTRY",
        Arity::Fixed(1),
        native_hash_remove_entry,
    ),
    // `Directory`'s own two, which override `Object`'s private pair. Neither
    // reaches `StringTable`: `setMethod` is declared only on `Directory`, and
    // a `setMethod` send to a `StringTable` falls through to its `unknown` and
    // answers `.nil`.
    (
        "Directory",
        "SETMETHOD",
        Arity::Fixed(2),
        native_directory_set_method,
    ),
    (
        "Directory",
        "UNSETMETHOD",
        Arity::Fixed(1),
        native_directory_unset_method,
    ),
];

/// A fresh `Directory` holding one entry per pair, string-keyed -- what
/// `RexxContext~variables` answers.
pub(super) fn directory_of(
    interp: &mut Interp,
    entries: Vec<(Vec<u8>, ObjRef)>,
) -> Result<ObjRef, Failure> {
    let class = interp.object_model().directory;
    let directory = new_instance(interp, class)?;
    let frame = interp.roots.activity_mut().push_frame();
    for (name, value) in entries {
        let key = interp.text_built(name);
        interp.roots.activity_mut().push_temp(key);
        insert(interp, directory, key, Some(value))?;
    }
    interp.roots.activity_mut().pop_frame(frame);
    Ok(directory)
}
