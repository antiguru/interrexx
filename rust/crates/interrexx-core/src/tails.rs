//! A stem's tails: each tail's value, kept in a slab at the tail's insertion
//! ordinal, and a map from the tail's bytes to that ordinal.
//!
//! **The ordinal is load-bearing.** A stem answers its tails in post-order
//! over a balanced tree its insertions built, so the order depends on which
//! tail came first, and a map cannot say. A dropped tail stays as a
//! tombstone (`None`): it keeps its place in that order, and it reads as
//! unset rather than as the stem's default. Nothing but [`Tails::clear`]
//! removes a tail, so the ordinals are exactly `0..len`, and the ordinal is
//! also the tail's slot in the slab.
//!
//! The slab is what lets a caller that has found a tail once write it again
//! by its ordinal, without hashing the tail's bytes a second time.

use crate::{NameMap, ObjRef};

/// The tails of one stem. Allocated on the first tail, so a stem that never
/// holds one costs a null pointer.
#[derive(Clone, Default, Debug)]
pub struct Tails(Option<Box<Store>>);

#[derive(Clone, Default, Debug)]
struct Store {
    ordinals: NameMap<Vec<u8>, usize>,
    /// Indexed by ordinal, one slot per entry of `ordinals`.
    values: Slab,
}

/// How many values a stem keeps beside its map before it allocates for
/// more: a stem made fresh for a few tails, as `avar. = x` makes one per
/// loop, pays for the store and the map and no third allocation.
const INLINE: usize = 4;

/// The values, the first [`INLINE`] of them in the store itself.
#[derive(Clone, Default, Debug)]
struct Slab {
    len: usize,
    inline: [Option<ObjRef>; INLINE],
    spill: Vec<Option<ObjRef>>,
}

impl Slab {
    fn len(&self) -> usize {
        self.len
    }

    fn get(&self, at: usize) -> Option<&Option<ObjRef>> {
        if at >= self.len {
            None
        } else if at < INLINE {
            Some(&self.inline[at])
        } else {
            self.spill.get(at - INLINE)
        }
    }

    fn get_mut(&mut self, at: usize) -> Option<&mut Option<ObjRef>> {
        if at >= self.len {
            None
        } else if at < INLINE {
            Some(&mut self.inline[at])
        } else {
            self.spill.get_mut(at - INLINE)
        }
    }

    fn push(&mut self, value: Option<ObjRef>) {
        if self.len < INLINE {
            self.inline[self.len] = value;
        } else {
            self.spill.push(value);
        }
        self.len += 1;
    }

    fn iter(&self) -> impl Iterator<Item = Option<ObjRef>> + '_ {
        self.inline[..self.len.min(INLINE)]
            .iter()
            .chain(&self.spill)
            .copied()
    }
}

impl Tails {
    /// No tails.
    pub fn new() -> Tails {
        Tails(None)
    }

    /// How many tails there are, tombstones included.
    pub fn len(&self) -> usize {
        self.0.as_ref().map_or(0, |store| store.values.len())
    }

    /// Whether there are none, tombstones included.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The tail `key`'s ordinal and value: `None` where the stem has no such
    /// tail, `Some((ordinal, None))` where it is a tombstone.
    #[inline]
    pub fn lookup(&self, key: &[u8]) -> Option<(usize, Option<ObjRef>)> {
        let store = self.0.as_deref()?;
        let ordinal = *store.ordinals.get(key)?;
        Some((ordinal, *store.values.get(ordinal)?))
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

    /// Sets the tail `key` to `value`, a tombstone where `value` is `None`,
    /// keeping its ordinal where it has one and taking the next otherwise.
    /// Answers the ordinal.
    #[inline]
    pub fn set(&mut self, key: &[u8], value: Option<ObjRef>) -> usize {
        let store = self.0.get_or_insert_with(Box::default);
        // **Looked up before it is inserted, because the key is already there
        // on all but the first write to a tail**: inserting needs an owned
        // key whether or not the map keeps it.
        match store.ordinals.get(key) {
            Some(&ordinal) => {
                if let Some(slot) = store.values.get_mut(ordinal) {
                    *slot = value;
                }
                ordinal
            }
            None => {
                let ordinal = store.values.len();
                store.ordinals.insert(key.to_vec(), ordinal);
                store.values.push(value);
                ordinal
            }
        }
    }

    /// Adds the tail `key` holding `value` where the stem has no such tail,
    /// and leaves an existing one, tombstone or not, as it is.
    pub fn insert_if_absent(&mut self, key: &[u8], value: Option<ObjRef>) {
        if !self.contains_key(key) {
            self.set(key, value);
        }
    }

    /// The value at `ordinal`: `None` for a tombstone or an ordinal the stem
    /// does not have.
    #[inline]
    pub fn value_at(&self, ordinal: usize) -> Option<ObjRef> {
        self.0
            .as_deref()
            .and_then(|store| store.values.get(ordinal).copied().flatten())
    }

    /// Overwrites the value of the tail at `ordinal`, answering whether there
    /// is one.
    #[inline]
    pub fn set_at(&mut self, ordinal: usize, value: Option<ObjRef>) -> bool {
        match self
            .0
            .as_deref_mut()
            .and_then(|store| store.values.get_mut(ordinal))
        {
            Some(slot) => {
                *slot = value;
                true
            }
            None => false,
        }
    }

    /// Removes every tail.
    pub fn clear(&mut self) {
        self.0 = None;
    }

    /// Every tail's value, tombstones as `None`, in ordinal order.
    pub fn values(&self) -> impl Iterator<Item = Option<ObjRef>> + '_ {
        self.0
            .as_deref()
            .into_iter()
            .flat_map(|store| store.values.iter())
    }

    /// Every tail's bytes, ordinal and value, in no particular order.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], usize, Option<ObjRef>)> + '_ {
        self.0.as_deref().into_iter().flat_map(|store| {
            store.ordinals.iter().map(|(key, &ordinal)| {
                (
                    key.as_slice(),
                    ordinal,
                    store.values.get(ordinal).copied().flatten(),
                )
            })
        })
    }

    /// Every tail's bytes and value, in ordinal order.
    pub fn in_order(&self) -> Vec<(&[u8], Option<ObjRef>)> {
        let Some(store) = self.0.as_deref() else {
            return Vec::new();
        };
        let mut keys: Vec<&[u8]> = vec![&[]; store.values.len()];
        for (key, &ordinal) in &store.ordinals {
            keys[ordinal] = key;
        }
        keys.into_iter().zip(store.values.iter()).collect()
    }

    /// How many slots the slab holds, which is [`Tails::len`]: one per tail,
    /// tombstones included, and none besides.
    pub fn slab_len(&self) -> usize {
        self.0.as_ref().map_or(0, |store| store.values.len())
    }

    /// How many entries the map holds.
    pub fn map_len(&self) -> usize {
        self.0.as_ref().map_or(0, |store| store.ordinals.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tail_keeps_its_ordinal_through_a_tombstone() {
        let mut tails = Tails::new();
        let one = ObjRef::NIL;
        assert_eq!(tails.set(b"A", Some(one)), 0);
        assert_eq!(tails.set(b"B", Some(one)), 1);
        assert_eq!(tails.set(b"A", None), 0);
        assert_eq!(tails.get(b"A"), Some(None));
        assert_eq!(tails.set(b"A", Some(one)), 0);
        assert_eq!(tails.set(b"C", None), 2);
        assert_eq!(tails.lookup(b"C"), Some((2, None)));
        assert_eq!(tails.get(b"D"), None);
        let order: Vec<&[u8]> = tails.in_order().into_iter().map(|(k, _)| k).collect();
        assert_eq!(order, [&b"A"[..], b"B", b"C"]);
    }

    #[test]
    fn the_slab_holds_one_slot_per_tail_under_churn() {
        let mut tails = Tails::new();
        for i in 0..1_000_000u32 {
            let key = i.to_string().into_bytes();
            tails.set(&key, Some(ObjRef::NIL));
            tails.set(&key, None);
        }
        assert_eq!(tails.len(), 1_000_000);
        assert_eq!(tails.slab_len(), tails.map_len());
        tails.clear();
        assert_eq!((tails.slab_len(), tails.map_len()), (0, 0));
    }

    #[test]
    fn writing_by_ordinal_needs_the_ordinal_to_exist() {
        let mut tails = Tails::new();
        assert!(!tails.set_at(0, Some(ObjRef::NIL)));
        tails.set(b"X", None);
        assert!(tails.set_at(0, Some(ObjRef::NIL)));
        assert_eq!(tails.value_at(0), Some(ObjRef::NIL));
        assert!(!tails.set_at(1, None));
    }
}
