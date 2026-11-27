//! A model test of [`Tails`]: random sequences of operations, applied to a
//! stem's tails and to a list of `(key, value)` pairs in order, must agree
//! after every operation.
//!
//! The model is the contract of [`Tails`] with nothing of its storage: a
//! tail is a key and a value or a tombstone, the order is the list's, an
//! overwrite or a tombstone keeps a tail's place, and a tombstone given a
//! value, like a new tail, goes to the end. An ordinal is a position in the
//! slab, which the model does not have; what it promises is checked instead:
//! the ordinal a write answers names the tail, and while the epoch stays the
//! same every tail the operation did not write keeps its ordinal.
//!
//! Each sequence draws its keys from a pool of one of several sizes, so that
//! some sequences stay within the inline slots and others cross the
//! compaction threshold again and again, and it runs in phases that mostly
//! add or mostly drop, so the slab both fills and empties.

use super::*;
use interrexx_cast::{CastFrom, CastTruncate};

/// splitmix64: a small generator whose sequences are fixed by their seed, so
/// a failing seed replays.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        // The remainder is below `n`, so truncating it to a `usize` keeps it whole.
        usize::cast_truncate(self.next() % u64::cast_from(n))
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }
}

/// The tails in order: each key once, with its value or `None` for a
/// tombstone.
#[derive(Clone, Debug, Default)]
struct Model(Vec<(Vec<u8>, Option<ObjRef>)>);

impl Model {
    fn position(&self, key: &[u8]) -> Option<usize> {
        self.0.iter().position(|(held, _)| held == key)
    }

    fn set(&mut self, key: &[u8], value: Option<ObjRef>) {
        match self.position(key) {
            Some(at) if self.0[at].1.is_some() || value.is_none() => self.0[at].1 = value,
            Some(at) => {
                self.0.remove(at);
                self.0.push((key.to_vec(), value));
            }
            None => self.0.push((key.to_vec(), value)),
        }
    }

    fn insert_if_absent(&mut self, key: &[u8], value: Option<ObjRef>) {
        if self.position(key).is_none() {
            self.0.push((key.to_vec(), value));
        }
    }

    fn remove(&mut self, key: &[u8]) {
        if let Some(at) = self.position(key) {
            self.0.remove(at);
        }
    }

    /// [`Tails::set_at`] at the ordinal of `key`: a tail holding a value is
    /// overwritten in place, a tombstone refused.
    fn set_at(&mut self, key: &[u8], value: Option<ObjRef>) -> bool {
        match self.position(key) {
            Some(at) if self.0[at].1.is_some() => {
                self.0[at].1 = value;
                true
            }
            _ => false,
        }
    }
}

/// The pool's `i`th key. Index 0 is the empty tail, and every seventh key is
/// longer than a hasher's word, so short, empty and long keys mix.
fn key_of(i: usize) -> Vec<u8> {
    match i {
        0 => Vec::new(),
        _ if i.is_multiple_of(7) => format!("A.LONG.TAIL.NUMBER.{i}").into_bytes(),
        _ => format!("K{i}").into_bytes(),
    }
}

/// Checks everything `tails` answers against `model`, and the bound on the
/// slab's holes that compaction keeps.
fn check(tails: &Tails, model: &Model, pool: usize, context: &dyn Fn() -> String) {
    let in_order: Vec<(Vec<u8>, Option<ObjRef>)> = tails
        .in_order()
        .into_iter()
        .map(|(key, value)| (key.to_vec(), value))
        .collect();
    assert_eq!(in_order, model.0, "in_order, {}", context());
    assert_eq!(tails.len(), model.0.len(), "len, {}", context());
    assert_eq!(tails.is_empty(), model.0.is_empty(), "{}", context());
    assert_eq!(tails.map_len(), model.0.len(), "map_len, {}", context());
    let values: Vec<Option<ObjRef>> = tails.values().collect();
    let expected: Vec<Option<ObjRef>> = model.0.iter().map(|(_, value)| *value).collect();
    assert_eq!(values, expected, "values, {}", context());
    let mut ordinals = Vec::with_capacity(model.0.len());
    for (key, value) in &model.0 {
        assert!(tails.contains_key(key), "contains_key, {}", context());
        assert_eq!(tails.get(key), Some(*value), "get, {}", context());
        let Some((ordinal, found)) = tails.lookup(key) else {
            panic!("lookup of a held tail found none, {}", context());
        };
        assert_eq!(found, *value, "lookup, {}", context());
        assert_eq!(tails.value_at(ordinal), *value, "value_at, {}", context());
        ordinals.push(ordinal);
    }
    let mut sorted = ordinals.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, ordinals, "ordinals follow the order, {}", context());
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        ordinals.len(),
        "ordinals are distinct, {}",
        context()
    );
    for i in 0..pool {
        let key = key_of(i);
        if model.position(&key).is_none() {
            assert!(!tails.contains_key(&key), "contains_key, {}", context());
            assert_eq!(tails.get(&key), None, "get, {}", context());
        }
    }
    // Compaction looks only when a hole is made, and not in a slab of at
    // most `INLINE` slots; the holes of such a slab stay while it grows.
    let slab = tails.slab_len();
    let holes = slab - tails.map_len();
    assert!(
        holes <= INLINE || 2 * holes <= slab,
        "{holes} holes in a slab of {slab}, {}",
        context()
    );
}

/// The operations a sequence draws from.
#[derive(Clone, Copy, Debug)]
enum Op {
    Set(usize, Option<ObjRef>),
    InsertIfAbsent(usize, Option<ObjRef>),
    Remove(usize),
    SetAt(usize, Option<ObjRef>),
    SetAtBeyond(usize, Option<ObjRef>),
    Clear,
    Snapshot,
}

/// Runs one sequence of `ops` operations under `seed`.
fn run(seed: u64, ops: usize) {
    let mut rng = Rng(seed);
    let pool = [3, INLINE, 2 * INLINE + 1, 40, 300][rng.below(5)];
    let mut tails = Tails::new();
    let mut model = Model::default();
    // Each clone with the model it must still read as, and the step and op
    // it was taken at.
    let mut snapshots: Vec<(Tails, Model, usize, Op)> = Vec::new();
    let mut counter = 0i64;
    // Most of a phase adds or drops by its bias; a new phase every so often.
    let mut adding = true;
    // Whether a write has allocated the store: `clear` on a stem that never
    // held a tail has no store, and so no epoch, to move.
    let mut allocated = false;
    for step in 0..ops {
        if rng.chance(2) {
            adding = !adding;
        }
        counter += 1;
        let value = if rng.chance(20) {
            None
        } else {
            Some(ObjRef::small_int(counter).expect("a small counter"))
        };
        let key = rng.below(pool);
        let grow = if adding { 70 } else { 25 };
        let op = match rng.below(100) {
            r if r < grow => Op::Set(key, value),
            r if r < grow + 5 => Op::InsertIfAbsent(key, value),
            r if r < grow + 10 => Op::SetAt(key, value),
            r if r < grow + 11 => Op::SetAtBeyond(rng.below(4), value),
            r if r < grow + 12 && rng.chance(10) => Op::Clear,
            r if r < grow + 13 => Op::Snapshot,
            _ => Op::Remove(key),
        };
        let context = || format!("seed {seed}, pool {pool}, step {step}, op {op:?}");
        let epoch = tails.epoch();
        let before: Vec<(Vec<u8>, usize)> = model
            .0
            .iter()
            .map(|(held, _)| (held.clone(), tails.lookup(held).expect("a held tail").0))
            .collect();
        // Whether this op compacts, worked out from the slab before it: only
        // a new hole can, which a removal makes, and a tombstone given a
        // value makes along with a slot at the end. The slab is compacted
        // when it then has more than `INLINE` slots and more than half of
        // them are holes.
        let slab = tails.slab_len();
        let holes = slab - tails.map_len();
        let new_hole = match op {
            Op::Remove(i) => model.position(&key_of(i)).map(|_| 0),
            Op::Set(i, Some(_)) => model
                .position(&key_of(i))
                .filter(|&at| model.0[at].1.is_none())
                .map(|_| 1),
            _ => None,
        };
        let compacts = new_hole.is_some_and(|pushed| {
            let len = slab + pushed;
            len > INLINE && 2 * (holes + 1) > len
        });
        let mut written: Option<Vec<u8>> = None;
        match op {
            Op::Set(i, value) => {
                allocated = true;
                let key = key_of(i);
                let ordinal = tails.set(SEED, &key, value);
                model.set(&key, value);
                assert_eq!(
                    tails.lookup(&key),
                    Some((ordinal, value)),
                    "set answers the tail's ordinal, {}",
                    context()
                );
                written = Some(key);
            }
            Op::InsertIfAbsent(i, value) => {
                allocated = true;
                let key = key_of(i);
                tails.insert_if_absent(SEED, &key, value);
                model.insert_if_absent(&key, value);
            }
            Op::Remove(i) => {
                let key = key_of(i);
                tails.remove(&key);
                model.remove(&key);
            }
            Op::SetAt(i, value) => {
                let key = key_of(i);
                if let Some((ordinal, _)) = tails.lookup(&key) {
                    let wrote = tails.set_at(ordinal, value);
                    assert_eq!(wrote, model.set_at(&key, value), "set_at, {}", context());
                }
            }
            Op::SetAtBeyond(past, value) => {
                let ordinal = tails.slab_len() + past;
                assert!(!tails.set_at(ordinal, value), "set_at, {}", context());
            }
            Op::Clear => {
                tails.clear();
                model = Model::default();
                if allocated {
                    assert_ne!(tails.epoch(), epoch, "clear moves the epoch, {}", context());
                }
            }
            Op::Snapshot => {
                if snapshots.len() >= 4 {
                    snapshots.remove(rng.below(snapshots.len()));
                }
                snapshots.push((tails.clone(), model.clone(), step, op));
            }
        }
        check(&tails, &model, pool, &context);
        if !matches!(op, Op::Clear) {
            assert_eq!(
                tails.epoch() != epoch,
                compacts,
                "compaction at {holes} holes in a slab of {slab}, {}",
                context()
            );
        }
        if tails.epoch() == epoch {
            for (held, ordinal) in &before {
                if written.as_deref() == Some(held.as_slice()) {
                    continue;
                }
                if let Some((now, _)) = tails.lookup(held) {
                    assert_eq!(
                        now,
                        *ordinal,
                        "an ordinal moved within one epoch, {}",
                        context()
                    );
                }
            }
        } else if !matches!(op, Op::Clear) {
            assert_eq!(
                tails.slab_len(),
                tails.map_len(),
                "a compaction leaves no hole, {}",
                context()
            );
        }
        // A clone shares nothing with its original: each snapshot still
        // reads as it did when taken.
        if step % 64 == 63 || step + 1 == ops {
            for (snapshot, expected, taken, taken_op) in &snapshots {
                let context = || {
                    format!(
                        "seed {seed}, pool {pool}, snapshot taken at step {taken}, \
                         op {taken_op:?}, checked at step {step}"
                    )
                };
                check(snapshot, expected, pool, &context);
            }
        }
    }
}

const SEED: HashSeed = HashSeed::new(7, 11);

/// Random sequences against the model, over every pool size.
#[test]
fn random_sequences_agree_with_the_model() {
    for seed in 0..200 {
        run(seed, 1_500);
    }
}
