//! Symbol interning: `SymbolId`, and the `SymbolTable` that hands them out.

use rustc_hash::FxBuildHasher;
use std::hash::BuildHasher;

/// A symbol's identity: the upcased spelling, interned. Two symbols with the
/// same `SymbolId` name the same variable, method or label.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SymbolId(u32);

impl SymbolId {
    /// This id as an index into the `SymbolTable` that interned it: **dense,
    /// zero-based, and assigned in interning order**, so `n` distinct symbols
    /// occupy exactly `0..n` and `SymbolTable::len` is the length a caller
    /// needs to size a `Vec`.
    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// The id whose [`Self::index`] is `index`, where one can be.
    pub(crate) fn from_index(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(SymbolId)
    }
}

/// Interns upcased symbol spellings. Owned by `ProgramSource`'s parse, handed
/// to `Program` so the executor can resolve a `SymbolId` back to text for error
/// messages and `SIGNAL`'s label lookup.
///
/// A table may sit on a frozen **base** table (`over`): the base's symbols keep
/// their ids `0..base.len()`, and this table's own symbols follow from
/// `base.len()`. The ids are then exactly those a table that first interned the
/// base's symbols, in the base's order, would hand out, so ids stay dense and
/// `len` still sizes a `Vec`. The scanner puts every chunk over one
/// process-wide table of the keyword spellings, so compiling a short chunk (an
/// `INTERPRET` fragment, a `Method~new` source) costs in proportion to the
/// chunk rather than re-interning the ~150 keywords each time.
///
/// The index is a linear-probing table of its own rather than a `HashMap`, for
/// two reasons a map cannot serve: a table over a base starts from a copy of
/// the base's index (one small allocation and a copy, so a lookup is one hash
/// and one probe sequence whichever table the symbol is in), and a symbol's
/// spelling is stored once, in `names`, rather than once as a key and again
/// for `name`.
#[derive(Default, Debug)]
pub struct SymbolTable {
    base: Option<&'static SymbolTable>,
    /// The index: a power-of-two number of slots, or none before the first
    /// symbol. Each slot is empty or holds an id and its spelling's hash.
    slots: Vec<Slot>,
    /// This table's own spellings, upcased, by id less the base's length.
    names: Vec<Box<str>>,
}

/// One index slot: `id_plus_one` 0 when empty, else the id plus one and the
/// low half of its spelling's hash, which settles almost every mismatch
/// without reading the spelling.
#[derive(Copy, Clone, Default, Debug)]
struct Slot {
    hash: u32,
    id_plus_one: u32,
}

/// The hash an upcased spelling's bytes are indexed by.
fn hash_of(key: &[u8]) -> u32 {
    // The low half: the index never has 2^32 slots.
    FxBuildHasher.hash_one(key) as u32
}

impl SymbolTable {
    /// An empty table over the frozen `base`, which must itself have no base.
    pub fn over(base: &'static SymbolTable) -> Self {
        debug_assert!(base.base.is_none(), "a base table has no base of its own");
        SymbolTable {
            base: Some(base),
            slots: base.slots.clone(),
            names: Vec::new(),
        }
    }

    /// How many ids the base holds: this table's own ids start here, and
    /// every id below it names a keyword spelling every table shares.
    pub fn base_len(&self) -> usize {
        self.base.map_or(0, |base| base.names.len())
    }

    /// Intern `text`, upcasing it. Returns the same id for every spelling that
    /// differs only in case.
    pub fn intern(&mut self, text: &str) -> SymbolId {
        // The upcased key is built without allocating whenever it fits a
        // stack buffer, which every symbol a real program writes does: the
        // lookup path is the common case by an order of magnitude, and source
        // is mostly lower case. Only a genuine miss allocates, for the stored
        // spelling.
        // Bytes rather than `str` until a miss: the upcased copy is ASCII-
        // upcased valid UTF-8, but proving that to `str` costs a validation per
        // lookup.
        let mut buffer = [0u8; 64];
        let upcased: String;
        let key: &[u8] = if !text.bytes().any(|b| b.is_ascii_lowercase()) {
            text.as_bytes()
        } else if let Some(bytes) = buffer.get_mut(..text.len()) {
            bytes.copy_from_slice(text.as_bytes());
            bytes.make_ascii_uppercase();
            bytes
        } else {
            upcased = text.to_ascii_uppercase();
            upcased.as_bytes()
        };
        let hash = hash_of(key);
        let slot = match self.find(key, hash) {
            Ok(id) => return id,
            Err(slot) => slot,
        };
        let id = u32::try_from(self.len()).expect("symbols fit u32");
        let spelling = std::str::from_utf8(key).expect("ASCII upcasing keeps UTF-8 valid");
        self.names.push(spelling.into());
        self.slots[slot] = Slot {
            hash,
            id_plus_one: id + 1,
        };
        // At most three quarters full, so a probe sequence stays short and
        // always ends at an empty slot.
        if self.len() * 4 > self.slots.len() * 3 {
            self.grow();
        }
        SymbolId(id)
    }

    /// The id of `key`, or the empty slot where it belongs.
    fn find(&mut self, key: &[u8], hash: u32) -> Result<SymbolId, usize> {
        if self.slots.is_empty() {
            self.slots = vec![Slot::default(); 16];
        }
        let mask = self.slots.len() - 1;
        let mut at = hash as usize & mask;
        loop {
            let slot = self.slots[at];
            if slot.id_plus_one == 0 {
                return Err(at);
            }
            let id = SymbolId(slot.id_plus_one - 1);
            if slot.hash == hash && self.name(id).as_bytes() == key {
                return Ok(id);
            }
            at = (at + 1) & mask;
        }
    }

    /// Doubles the index, re-placing every slot by its stored hash.
    fn grow(&mut self) {
        let doubled = vec![Slot::default(); self.slots.len() * 2];
        let old = std::mem::replace(&mut self.slots, doubled);
        let mask = self.slots.len() - 1;
        for slot in old.into_iter().filter(|slot| slot.id_plus_one != 0) {
            let mut at = slot.hash as usize & mask;
            while self.slots[at].id_plus_one != 0 {
                at = (at + 1) & mask;
            }
            self.slots[at] = slot;
        }
    }

    /// The upcased spelling. Panics on an id from a different table, which is
    /// a parser bug rather than a source error.
    pub fn name(&self, id: SymbolId) -> &str {
        let index = id.0 as usize;
        match self.base {
            Some(base) if index < base.names.len() => &base.names[index],
            _ => &self.names[index - self.base_len()],
        }
    }

    /// How many distinct symbols are interned, the base's included.
    pub fn len(&self) -> usize {
        self.base_len() + self.names.len()
    }

    /// Whether nothing is interned yet, in this table or its base.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
