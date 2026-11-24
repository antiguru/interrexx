//! A stem's tails: each tail's value, kept in a slab at the tail's ordinal,
//! and a map from the tail's bytes to that ordinal.
//!
//! **The ordinal is load-bearing.** A stem answers its tails in insertion
//! order, which is ordinal order, and a map cannot say which tail came first.
//! A stem with a default keeps a dropped tail as a tombstone so it reads
//! as unset. A drop from a stem without a default removes the key and makes
//! its slot a hole. A new tail or an assignment after a drop goes to the
//! end. Overwriting an existing value keeps its ordinal.
//!
//! Holes are reclaimed by compaction: when they make up more than half of a
//! slab larger than [`INLINE`], the slab is rebuilt in ordinal order without
//! them, and every ordinal is renumbered. This bounds the slab even when
//! a stem without a default drops a different tail on each pass. Tombstones
//! needed to shadow a default stay until reassignment or [`Tails::clear`].
//! Renumbering, and
//! [`Tails::clear`], change [`Tails::epoch`]: an ordinal is good only with
//! the epoch it was read under.
//!
//! The slab is what lets a caller that has found a tail once write it again
//! by its ordinal, without hashing the tail's bytes a second time.
//!
//! The map hashes with a keyed hash whose keys come from the interpreter's
//! [`HashSeed`], so a program cannot choose tails that all collide in it
//! without knowing the seed. The order comes from the slab, so the seed never
//! shows in it. That hash is not the one a string's hash code answers: a
//! tail's hash is never observable, so a faster keyed hash serves.

use std::hash::{BuildHasher, Hasher};

use hashbrown::HashMap;
use hashbrown::hash_map::EntryRef;

use crate::{HashSeed, ObjRef};

/// The tails of one stem. Allocated on the first tail, so a stem that never
/// holds one costs a null pointer.
#[derive(Clone, Default, Debug)]
pub struct Tails(Option<Box<Store>>);

#[derive(Clone, Debug)]
struct Store {
    ordinals: HashMap<Vec<u8>, usize, TailHasher>,
    /// Indexed by ordinal: one slot per entry of `ordinals`, and the holes.
    values: Slab,
    /// How many slots of `values` are holes.
    holes: usize,
    /// Changed whenever ordinals are renumbered or dropped wholesale.
    epoch: u64,
}

/// The map's hasher: a keyed hash of a tail's bytes, keyed from the
/// interpreter's seed. A key is a byte string, compared whole when its hash
/// matches, so the length a slice writes ahead of its bytes is left out of
/// the hash.
#[derive(Clone, Debug)]
struct TailHasher(ahash::RandomState);

impl TailHasher {
    /// The hasher keyed from `seed`.
    fn of(seed: HashSeed) -> TailHasher {
        let [k0, k1, k2, k3] = seed.four_keys();
        TailHasher(ahash::RandomState::with_seeds(k0, k1, k2, k3))
    }
}

impl BuildHasher for TailHasher {
    type Hasher = BytesHasher;

    #[inline]
    fn build_hasher(&self) -> BytesHasher {
        BytesHasher(self.0.build_hasher())
    }
}

/// [`TailHasher`]'s hasher: the keyed hasher, passing over the length
/// prefix, which is the only `usize` a byte string writes.
struct BytesHasher(ahash::AHasher);

impl Hasher for BytesHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }

    #[inline]
    fn write_usize(&mut self, _length: usize) {}

    #[inline]
    fn finish(&self) -> u64 {
        self.0.finish()
    }
}

/// How many values a stem keeps beside its map before it allocates for
/// more: a stem made fresh for a few tails, as `avar. = x` makes one per
/// loop, pays for the store and the map and no third allocation.
const INLINE: usize = 4;

/// One slot of the slab. An `ObjRef` has no niche, so `Option<ObjRef>` is
/// already a tag beside the handle, and the third state costs no space.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
enum Slot {
    /// A tail holding a value.
    Value(ObjRef),
    /// A dropped tail: keeps its place, reads as unset.
    Tombstone,
    /// No tail: left by removal or by a tombstone that was assigned again.
    #[default]
    Hole,
}

impl Slot {
    fn of(value: Option<ObjRef>) -> Slot {
        value.map_or(Slot::Tombstone, Slot::Value)
    }

    /// The tail's value, `None` for a tombstone and a hole alike.
    fn value(self) -> Option<ObjRef> {
        match self {
            Slot::Value(value) => Some(value),
            Slot::Tombstone | Slot::Hole => None,
        }
    }
}

/// The slots, the first [`INLINE`] of them in the store itself.
#[derive(Clone, Default, Debug)]
struct Slab {
    len: usize,
    inline: [Slot; INLINE],
    spill: Vec<Slot>,
}

impl Slab {
    fn len(&self) -> usize {
        self.len
    }

    fn get(&self, at: usize) -> Option<Slot> {
        if at >= self.len {
            None
        } else if at < INLINE {
            Some(self.inline[at])
        } else {
            self.spill.get(at - INLINE).copied()
        }
    }

    fn get_mut(&mut self, at: usize) -> Option<&mut Slot> {
        if at >= self.len {
            None
        } else if at < INLINE {
            Some(&mut self.inline[at])
        } else {
            self.spill.get_mut(at - INLINE)
        }
    }

    fn push(&mut self, slot: Slot) {
        if self.len < INLINE {
            self.inline[self.len] = slot;
        } else {
            self.spill.push(slot);
        }
        self.len += 1;
    }

    fn iter(&self) -> impl Iterator<Item = Slot> + '_ {
        self.inline[..self.len.min(INLINE)]
            .iter()
            .chain(&self.spill)
            .copied()
    }
}

impl Store {
    /// No tails, its map hashing with `hasher`.
    fn new(hasher: TailHasher) -> Store {
        Store {
            ordinals: HashMap::with_hasher(hasher),
            values: Slab::default(),
            holes: 0,
            epoch: 0,
        }
    }

    /// Rebuilds the slab without its holes where they make up more than half
    /// of it, renumbering the map to match and changing the epoch. Answers
    /// whether it did.
    fn compact_if_sparse(&mut self) -> bool {
        let len = self.values.len();
        if len <= INLINE || self.holes * 2 <= len {
            return false;
        }
        // `renumbered[old]` is the new ordinal of the slot at `old`: how many
        // live slots precede it.
        let mut renumbered = Vec::with_capacity(len);
        let mut values = Slab::default();
        for slot in self.values.iter() {
            renumbered.push(values.len());
            if slot != Slot::Hole {
                values.push(slot);
            }
        }
        for ordinal in self.ordinals.values_mut() {
            *ordinal = renumbered[*ordinal];
        }
        self.values = values;
        self.holes = 0;
        self.epoch = self.epoch.wrapping_add(1);
        true
    }
}

impl Tails {
    /// No tails.
    pub fn new() -> Tails {
        Tails(None)
    }

    /// How many tails there are, tombstones included and holes not.
    pub fn len(&self) -> usize {
        self.0.as_ref().map_or(0, |store| store.ordinals.len())
    }

    /// Whether there are none, tombstones included.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The epoch the ordinals are numbered under: an ordinal read under one
    /// epoch names nothing under another.
    #[inline]
    pub fn epoch(&self) -> u64 {
        self.0.as_ref().map_or(0, |store| store.epoch)
    }

    /// The tail `key`'s ordinal and value: `None` where the stem has no such
    /// tail, `Some((ordinal, None))` where it is a tombstone.
    #[inline]
    pub fn lookup(&self, key: &[u8]) -> Option<(usize, Option<ObjRef>)> {
        let store = self.0.as_deref()?;
        let ordinal = *store.ordinals.get(key)?;
        Some((ordinal, store.values.get(ordinal)?.value()))
    }

    /// The tail `key`'s value: `None` where the stem has no such tail,
    /// `Some(None)` where it is a tombstone.
    #[inline]
    pub fn get(&self, key: &[u8]) -> Option<Option<ObjRef>> {
        self.lookup(key).map(|(_, value)| value)
    }

    /// Whether the stem has the tail `key`, a tombstone included.
    pub fn contains_key(&self, key: &[u8]) -> bool {
        self.0
            .as_deref()
            .is_some_and(|store| store.ordinals.contains_key(key))
    }

    /// Sets the tail `key` to `value`, a tombstone where `value` is `None`.
    /// A tail that holds a value, or is written a tombstone, keeps its
    /// ordinal. A tombstone given a value, and a new tail, take the next
    /// ordinal at the end of the order. Answers the tail's ordinal, which a
    /// compaction this write caused has already renumbered. `seed` is the
    /// interpreter's, which the map takes when this is the stem's first tail.
    #[inline]
    pub fn set(&mut self, seed: HashSeed, key: &[u8], value: Option<ObjRef>) -> usize {
        let store = self
            .0
            .get_or_insert_with(|| Box::new(Store::new(TailHasher::of(seed))));
        // **One lookup serves an existing tail and a new one alike**: the
        // entry borrows the key, so a write to an existing tail allocates
        // nothing, and a new tail is hashed once and copied only when filed.
        match store.ordinals.entry_ref(key) {
            EntryRef::Occupied(mut entry) => {
                let ordinal = entry.get_mut();
                let Some(slot) = store.values.get_mut(*ordinal) else {
                    unreachable!("a mapped ordinal is a slot of the slab");
                };
                if *slot != Slot::Tombstone || value.is_none() {
                    *slot = Slot::of(value);
                    return *ordinal;
                }
                // A tombstone given a value goes to the end, leaving a hole.
                *slot = Slot::Hole;
                *ordinal = store.values.len();
                store.values.push(Slot::of(value));
                store.holes += 1;
                let moved = *ordinal;
                if store.compact_if_sparse() {
                    store.ordinals[key]
                } else {
                    moved
                }
            }
            EntryRef::Vacant(entry) => {
                let ordinal = store.values.len();
                entry.insert(ordinal);
                store.values.push(Slot::of(value));
                ordinal
            }
        }
    }

    /// Adds the tail `key` holding `value` where the stem has no such tail,
    /// and leaves an existing one, tombstone or not, as it is.
    pub fn insert_if_absent(&mut self, seed: HashSeed, key: &[u8], value: Option<ObjRef>) {
        let store = self
            .0
            .get_or_insert_with(|| Box::new(Store::new(TailHasher::of(seed))));
        if let EntryRef::Vacant(entry) = store.ordinals.entry_ref(key) {
            entry.insert(store.values.len());
            store.values.push(Slot::of(value));
        }
    }

    /// Removes a tail entirely, when its absence already has the right
    /// value. Its ordinal stays vacant until compaction invalidates it.
    pub fn remove(&mut self, key: &[u8]) {
        let Some(store) = self.0.as_deref_mut() else {
            return;
        };
        if let Some(ordinal) = store.ordinals.remove(key) {
            let slot = store.values.get_mut(ordinal).expect("a mapped ordinal");
            *slot = Slot::Hole;
            store.holes += 1;
            if store.compact_if_sparse() {
                store.ordinals.shrink_to_fit();
            }
        }
    }

    /// The value at `ordinal`: `None` for a tombstone, a hole, or an ordinal
    /// the stem does not have.
    #[inline]
    pub fn value_at(&self, ordinal: usize) -> Option<ObjRef> {
        self.0
            .as_deref()
            .and_then(|store| store.values.get(ordinal))
            .and_then(Slot::value)
    }

    /// Overwrites, or drops where `value` is `None`, the tail at `ordinal`,
    /// answering whether there is a tail holding a value there. A tombstone
    /// is refused: a value assigned to it goes to the end of the order,
    /// which only [`Tails::set`], knowing the key, can do.
    #[inline]
    pub fn set_at(&mut self, ordinal: usize, value: Option<ObjRef>) -> bool {
        match self
            .0
            .as_deref_mut()
            .and_then(|store| store.values.get_mut(ordinal))
        {
            Some(slot @ Slot::Value(_)) => {
                *slot = Slot::of(value);
                true
            }
            _ => false,
        }
    }

    /// Removes every tail, and changes the epoch.
    pub fn clear(&mut self) {
        // The store stays, so that its epoch can move on: a fresh store
        // would start the count again, and an ordinal held across the clear
        // would name the first tail assigned after it.
        if let Some(store) = self.0.as_deref_mut() {
            *store = Store {
                epoch: store.epoch.wrapping_add(1),
                ..Store::new(store.ordinals.hasher().clone())
            };
        }
    }

    /// Every tail's value, tombstones as `None`, in ordinal order.
    pub fn values(&self) -> impl Iterator<Item = Option<ObjRef>> + '_ {
        self.0.as_deref().into_iter().flat_map(|store| {
            store
                .values
                .iter()
                .filter(|slot| *slot != Slot::Hole)
                .map(Slot::value)
        })
    }

    /// Every tail's bytes and value, tombstones as `None`, in ordinal order.
    pub fn in_order(&self) -> Vec<(&[u8], Option<ObjRef>)> {
        let Some(store) = self.0.as_deref() else {
            return Vec::new();
        };
        let mut keys: Vec<Option<&[u8]>> = vec![None; store.values.len()];
        for (key, &ordinal) in &store.ordinals {
            keys[ordinal] = Some(key);
        }
        keys.into_iter()
            .zip(store.values.iter())
            .filter_map(|(key, slot)| Some((key?, slot.value())))
            .collect()
    }

    /// How many slots the slab holds: one per tail, tombstones included, and
    /// the holes not yet compacted away.
    pub fn slab_len(&self) -> usize {
        self.0.as_ref().map_or(0, |store| store.values.len())
    }

    /// How many entries the map holds.
    pub fn map_len(&self) -> usize {
        self.0.as_ref().map_or(0, |store| store.ordinals.len())
    }

    /// The hash the map gives `key`.
    #[cfg(test)]
    fn tail_hash(&self, key: &[u8]) -> Option<u64> {
        let store = self.0.as_deref()?;
        Some(store.ordinals.hasher().hash_one(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: HashSeed = HashSeed::new(1, 2);

    fn keys(tails: &Tails) -> Vec<&[u8]> {
        tails.in_order().into_iter().map(|(key, _)| key).collect()
    }

    #[test]
    fn a_reassigned_dropped_tail_goes_to_the_end() {
        let mut tails = Tails::new();
        let one = ObjRef::NIL;
        assert_eq!(tails.set(SEED, b"A", Some(one)), 0);
        assert_eq!(tails.set(SEED, b"B", Some(one)), 1);
        assert_eq!(tails.set(SEED, b"A", None), 0);
        assert_eq!(tails.set(SEED, b"A", Some(one)), 2);
        assert_eq!(tails.lookup(b"A"), Some((2, Some(one))));
        assert_eq!(tails.value_at(0), None);
        assert_eq!(keys(&tails), [&b"B"[..], b"A"]);
        assert_eq!(tails.len(), 2);
        assert_eq!(tails.values().collect::<Vec<_>>(), [Some(one), Some(one)]);
    }

    #[test]
    fn a_dropped_tail_keeps_its_place_until_reassigned() {
        let mut tails = Tails::new();
        let one = ObjRef::NIL;
        tails.set(SEED, b"A", Some(one));
        tails.set(SEED, b"B", Some(one));
        assert_eq!(tails.set(SEED, b"A", None), 0);
        assert_eq!(tails.get(b"A"), Some(None));
        assert_eq!(tails.get(b"D"), None);
        assert_eq!(
            tails.in_order(),
            [(&b"A"[..], None), (&b"B"[..], Some(one))]
        );
        assert_eq!(tails.set(SEED, b"C", None), 2);
        assert_eq!(tails.lookup(b"C"), Some((2, None)));
        assert_eq!(keys(&tails), [&b"A"[..], b"B", b"C"]);
    }

    #[test]
    fn an_overwrite_keeps_the_place() {
        let mut tails = Tails::new();
        let one = ObjRef::NIL;
        tails.set(SEED, b"A", Some(one));
        tails.set(SEED, b"B", Some(one));
        assert_eq!(tails.set(SEED, b"A", Some(one)), 0);
        assert_eq!(keys(&tails), [&b"A"[..], b"B"]);
        assert_eq!(tails.slab_len(), 2);
    }

    #[test]
    fn churn_on_one_tail_stays_bounded() {
        let mut tails = Tails::new();
        tails.set(SEED, b"Y", Some(ObjRef::NIL));
        for _ in 0..1_000_000u32 {
            tails.set(SEED, b"X", None);
            tails.set(SEED, b"X", Some(ObjRef::NIL));
            assert!(tails.slab_len() <= 2 * tails.map_len().max(INLINE));
        }
        assert_eq!(keys(&tails), [&b"Y"[..], b"X"]);
    }

    #[test]
    fn compaction_bumps_the_epoch() {
        let mut tails = Tails::new();
        for key in [&b"A"[..], b"B", b"C", b"D", b"E", b"F"] {
            tails.set(SEED, key, Some(ObjRef::NIL));
        }
        let before = tails.epoch();
        let mut cycles = 0;
        while tails.epoch() == before {
            tails.set(SEED, b"B", None);
            tails.set(SEED, b"B", Some(ObjRef::NIL));
            cycles += 1;
            assert!(cycles < 1_000, "no compaction after {cycles} cycles");
        }
        assert_eq!(tails.slab_len(), tails.map_len());
        assert_eq!(keys(&tails), [&b"A"[..], b"C", b"D", b"E", b"F", b"B"]);
        for (ordinal, key) in keys(&tails).into_iter().enumerate() {
            assert_eq!(tails.lookup(key), Some((ordinal, Some(ObjRef::NIL))));
        }
    }

    #[test]
    fn clearing_bumps_the_epoch() {
        let mut tails = Tails::new();
        tails.set(SEED, b"A", Some(ObjRef::NIL));
        let before = tails.epoch();
        tails.clear();
        assert_ne!(tails.epoch(), before);
        assert_eq!((tails.len(), tails.slab_len(), tails.map_len()), (0, 0, 0));
    }

    #[test]
    fn the_slab_stays_bounded_under_churn_on_many_tails() {
        let mut tails = Tails::new();
        for i in 0..1_000_000u32 {
            let key = (i % 1000).to_string().into_bytes();
            tails.set(SEED, &key, Some(ObjRef::NIL));
            tails.set(SEED, &key, None);
        }
        assert_eq!(tails.len(), 1000);
        assert!(tails.slab_len() <= 2 * tails.map_len().max(INLINE));
        tails.clear();
        assert_eq!((tails.slab_len(), tails.map_len()), (0, 0));
    }

    #[test]
    fn writing_by_ordinal_needs_a_tail_holding_a_value() {
        let mut tails = Tails::new();
        assert!(!tails.set_at(0, Some(ObjRef::NIL)));
        tails.set(SEED, b"X", None);
        assert!(!tails.set_at(0, Some(ObjRef::NIL)));
        assert_eq!(tails.get(b"X"), Some(None));
        tails.set(SEED, b"Y", Some(ObjRef::NIL));
        assert!(tails.set_at(1, Some(ObjRef::NIL)));
        assert_eq!(tails.value_at(1), Some(ObjRef::NIL));
        assert!(!tails.set_at(2, None));
    }

    /// The map hashes a tail's bytes alone with the hash keyed from the
    /// seed.
    #[test]
    fn a_tail_hashes_as_the_keyed_hash_of_its_bytes_alone() {
        let mut tails = Tails::new();
        let [k0, k1, k2, k3] = SEED.four_keys();
        let state = ahash::RandomState::with_seeds(k0, k1, k2, k3);
        for key in [&b"X"[..], b"LONGER.THAN.EIGHT", b""] {
            tails.set(SEED, key, None);
            let mut bytes_alone = state.build_hasher();
            bytes_alone.write(key);
            assert_eq!(tails.tail_hash(key), Some(bytes_alone.finish()));
        }
    }

    /// The map hashes a tail under the seed its first tail was set with.
    #[test]
    fn two_seeds_hash_a_tail_differently() {
        let mut first = Tails::new();
        let mut second = Tails::new();
        first.set(HashSeed::new(1, 2), b"X", None);
        second.set(HashSeed::new(3, 4), b"X", None);
        assert_ne!(first.tail_hash(b"X"), second.tail_hash(b"X"));
        first.clear();
        first.set(HashSeed::new(3, 4), b"X", None);
        assert_ne!(first.tail_hash(b"X"), second.tail_hash(b"X"));
    }
}
