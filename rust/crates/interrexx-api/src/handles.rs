//! The local-reference table: the `ObjRef` behind an extension's handle.

use rustc_hash::FxHashMap;

use interrexx_cast::CastFrom;
use interrexx_core::ObjRef;

use crate::layout::RexxObjectPtr;

/// The objects one activation has handed a native call, held so the collector
/// can see them (D5).
///
/// A handle is derived from the whole of the `ObjRef`, generation included, in
/// the [`HandleSpace`] the table was made in, so two handles of one space are
/// equal exactly when they name the same object while either resolves, and a
/// handle minted before a slot was recycled does not name the slot's next
/// occupant. On a 64-bit target a heap object's handle also carries the
/// space's key ([`instance_key`]), so the same slot and generation in two
/// interpreters make different handles.
pub struct Table {
    entries: FxHashMap<usize, ObjRef>,
    /// `None` until the first [`Table::release`].
    released: Option<Box<Released>>,
    /// The interpreter's constants, registered on every native call
    /// ([`Table::register_constants`]): held apart from `entries`, so that
    /// registering them hashes nothing. Empty until then, and again after
    /// [`Table::clear`].
    constants: Option<[ObjRef; CONSTANTS]>,
    space: HandleSpace,
}

/// How many objects [`Table::register_constants`] holds.
pub const CONSTANTS: usize = 4;

/// The handles [`Table::release`] dropped, which root nothing and resolve
/// while their object lives.
#[derive(Default)]
struct Released {
    handles: FxHashMap<usize, ObjRef>,
    /// The size of `handles` at which [`Table::release`] next drops the
    /// handles whose object is gone.
    limit: usize,
}

/// The least [`Released::limit`].
const RELEASED_FLOOR: usize = 4096;

impl Table {
    /// An empty table whose handles are `space`'s: every table one
    /// interpreter resolves a handle through is made in that interpreter's
    /// space.
    pub fn new(space: &HandleSpace) -> Table {
        Table {
            entries: FxHashMap::default(),
            released: None,
            constants: None,
            space: space.clone(),
        }
    }

    /// The handle for `object`, held until [`Table::release`] or
    /// [`Table::clear`]. Registering an object this table already holds
    /// answers the same handle and changes nothing.
    #[cfg(target_pointer_width = "64")]
    pub fn register(&mut self, object: ObjRef) -> RexxObjectPtr {
        let address = self.space.hold(object);
        self.entries.insert(address, object);
        if let Some(released) = &mut self.released
            && !released.handles.is_empty()
        {
            released.handles.remove(&address);
        }
        std::ptr::without_provenance_mut(address)
    }

    /// The handle for `object`, held until [`Table::release`] or
    /// [`Table::clear`]. Registering an object this table already holds,
    /// registered or released, answers the same handle and holds its number
    /// no more than once.
    #[cfg(not(target_pointer_width = "64"))]
    pub fn register(&mut self, object: ObjRef) -> RexxObjectPtr {
        if let Some(address) = self.space.peek(object) {
            if self.entries.contains_key(&address) {
                return std::ptr::without_provenance_mut(address);
            }
            if let Some(released) = &mut self.released
                && released.handles.remove(&address).is_some()
            {
                self.entries.insert(address, object);
                return std::ptr::without_provenance_mut(address);
            }
        }
        let address = self.space.hold(object);
        self.entries.insert(address, object);
        std::ptr::without_provenance_mut(address)
    }

    /// The handles for `objects`, the interpreter's constants, held until
    /// [`Table::clear`], as [`Table::register`] holds an object.
    ///
    /// A constant is never collected, so releasing its handle changes nothing
    /// [`Table::resolve`] answers, and none is kept for it.
    pub fn register_constants(
        &mut self,
        objects: [ObjRef; CONSTANTS],
    ) -> [RexxObjectPtr; CONSTANTS] {
        if let Some(held) = self.constants.replace(objects) {
            held.iter().for_each(|object| self.space.unhold(*object));
        }
        objects.map(|object| std::ptr::without_provenance_mut(self.space.hold(object)))
    }

    /// The constant `address` names, if [`Table::register_constants`] holds
    /// one.
    fn constant(&self, address: usize) -> Option<ObjRef> {
        self.constants
            .as_ref()?
            .iter()
            .copied()
            .find(|object| self.space.peek(*object) == Some(address))
    }

    /// The object `handle` names, held or released, or `None` if this table
    /// has neither.
    pub fn resolve(&self, handle: RexxObjectPtr) -> Option<ObjRef> {
        self.resolve_live(handle, |_| true)
    }

    /// [`Table::resolve`], a released handle answered only where `live`
    /// says its object is.
    pub fn resolve_live(
        &self,
        handle: RexxObjectPtr,
        live: impl Fn(ObjRef) -> bool,
    ) -> Option<ObjRef> {
        let address = handle.addr();
        self.entries.get(&address).copied().or_else(|| {
            if let Some(object) = self.constant(address) {
                return Some(object);
            }
            self.released
                .as_ref()?
                .handles
                .get(&address)
                .copied()
                .filter(|object| live(*object))
        })
    }

    /// Whether this table holds `object`.
    pub fn holds(&self, object: ObjRef) -> bool {
        self.space.peek(object).is_some_and(|address| {
            self.entries.contains_key(&address) || self.constant(address).is_some()
        })
    }

    /// Drops `handle`'s registration, so it no longer roots its object. A
    /// handle this table does not hold is ignored. Each time the released
    /// handles pass their limit, those whose object `live` says is gone are
    /// forgotten.
    pub fn release(&mut self, handle: RexxObjectPtr, live: impl Fn(ObjRef) -> bool) {
        let address = handle.addr();
        let Some(object) = self.entries.remove(&address) else {
            return;
        };
        let released = self.released.get_or_insert_with(Box::default);
        released.handles.insert(address, object);
        if released.handles.len() >= released.limit.max(RELEASED_FLOOR) {
            let space = &self.space;
            released.handles.retain(|_, object| {
                let kept = live(*object);
                if !kept {
                    space.unhold(*object);
                }
                kept
            });
            released.limit = released.handles.len() * 2;
        }
    }

    /// Drops every registration and every released handle, which is what
    /// the end of an activation does.
    pub fn clear(&mut self) {
        self.unhold_all();
        // Clearing an empty table that has capacity still rewrites its control bytes.
        if !self.entries.is_empty() {
            self.entries.clear();
        }
        self.constants = None;
        if let Some(released) = &mut self.released {
            released.handles.clear();
            released.limit = 0;
        }
    }

    /// Gives back to the space every number this table holds: its
    /// registrations, its released handles and its constants.
    fn unhold_all(&self) {
        if !HandleSpace::COUNTS {
            return;
        }
        let released = self
            .released
            .iter()
            .flat_map(|released| released.handles.values());
        for object in self
            .entries
            .values()
            .chain(released)
            .chain(self.constants.iter().flatten())
        {
            self.space.unhold(*object);
        }
    }

    /// Every object this table holds, for the collector.
    pub fn roots(&self) -> impl Iterator<Item = ObjRef> + '_ {
        self.entries
            .values()
            .copied()
            .chain(self.constants.into_iter().flatten())
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        self.unhold_all();
    }
}

/// A key for a new interpreter's handles. Its two low bits are zero, so
/// mixing it in keeps a handle's tag. It is splitmix64 of a per-process
/// random base plus a counter, with the tag bits cleared: two calls in a
/// process answer the same key with a chance of about 2^-62, and the keys
/// differ from one process to the next.
#[cfg(target_pointer_width = "64")]
#[must_use]
pub fn instance_key() -> u64 {
    use std::hash::{BuildHasher, Hasher};
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU64, Ordering};
    static BASE: OnceLock<u64> = OnceLock::new();
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let base = *BASE.get_or_init(|| {
        std::collections::hash_map::RandomState::new()
            .build_hasher()
            .finish()
    });
    let counter = NEXT.fetch_add(1, Ordering::Relaxed);
    let mut z = base.wrapping_add(counter);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^= z >> 31;
    z & !0b11
}

/// The handle a thread table's data members carry for one of the
/// interpreter's constants before any call has registered them: the handle
/// every [`Table`] answers for it.
///
/// # Panics
/// On a target narrower than an `ObjRef`, for an object whose handle would
/// need a [`HandleSpace`]'s number, which none of the constants does.
#[must_use]
pub fn handle_of(object: ObjRef) -> RexxObjectPtr {
    let address = carried(object).expect("a constant's handle is carried in its address");
    std::ptr::without_provenance_mut(address)
}

/// `object`'s handle, as an address, where the address carries the `ObjRef`
/// itself, which is always on a 64-bit target: the `ObjRef`'s own bits, plus
/// one so that the `ObjRef` whose bits are zero -- heap slot zero at
/// generation zero, the first object a fresh heap allocates -- does not encode
/// to `NULLOBJECT`, the null pointer.
///
/// # Panics
/// If every bit of `object` is set, which no constructible `ObjRef` has: the
/// tag reserves the low two bits, and the only tag that sets both leaves the
/// three above the length clear.
#[cfg(target_pointer_width = "64")]
fn carried(object: ObjRef) -> Option<usize> {
    use interrexx_cast::CastFrom;

    let biased = object.bits().wrapping_add(1);
    assert!(biased != 0, "an ObjRef with every bit set has no handle");
    Some(usize::cast_from(biased))
}

/// `object`'s handle, as an address, where the address carries the `ObjRef`
/// itself: on a target narrower than an `ObjRef`, one whose bits fit in one
/// bit less than an address, shifted up with the low bit set, so the address
/// is odd and never null. `None` for any other, which a [`HandleSpace`]
/// numbers.
#[cfg(not(target_pointer_width = "64"))]
fn carried(object: ObjRef) -> Option<usize> {
    let bits = object.bits();
    (bits < 1 << (usize::BITS - 1)).then(|| {
        (usize::try_from(bits).expect("the bits are below half the address range") << 1) | 1
    })
}

/// The handles of one interpreter. On a 64-bit target a handle is the
/// `ObjRef`'s bits and the space holds nothing.
///
/// On a narrower target an `ObjRef` the address cannot carry ([`carried`]) is
/// given a number, and its handle is that number shifted up, so it is even and
/// never null. A number is held by each [`Table`] that registered the
/// `ObjRef`, released handles included, and by each holder the interpreter
/// keeps beside them ([`HandleSpace::hold`]); it is given back when the last
/// holder lets it go. Numbers are handed out in turn and wrap at the largest
/// an address carries, skipping those held, so a number given back is not
/// given to another `ObjRef` until every other number has been handed out
/// since: a handle an extension kept past its activation does not name the
/// next object it might, short of that. The space is the interpreter's, as
/// its tables are: nothing resolves one interpreter's handle in another's.
#[derive(Clone, Default)]
pub struct HandleSpace {
    #[cfg(not(target_pointer_width = "64"))]
    numbering: std::sync::Arc<std::sync::Mutex<Numbering>>,
    /// The key mixed into every heap object's handle, shared by every clone,
    /// so that re-keying a fresh interpreter ([`HandleSpace::set_key`])
    /// reaches the tables already made in its space.
    #[cfg(target_pointer_width = "64")]
    key: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl HandleSpace {
    /// Whether this target's space holds anything, so that a table that is
    /// dropped or cleared need not walk its handles where it holds nothing.
    const COUNTS: bool = cfg!(not(target_pointer_width = "64"));

    /// A space with no handle numbered, and on a 64-bit target a fresh key
    /// ([`instance_key`]).
    pub fn new() -> HandleSpace {
        let space = HandleSpace::default();
        #[cfg(target_pointer_width = "64")]
        space.set_key(instance_key());
        space
    }

    /// Makes heap objects' handles carry `key` from now on, in every clone of
    /// this space: for an interpreter that has handed native code none yet.
    #[cfg(target_pointer_width = "64")]
    pub fn set_key(&self, key: u64) {
        debug_assert_eq!(key & 0b11, 0, "a key keeps a handle's tag bits");
        self.key.store(key, std::sync::atomic::Ordering::Relaxed);
    }

    /// The `ObjRef` a handle this space minted names, if its bits say: a heap
    /// object's reference, whether or not that object still lives, or the
    /// value a handle carries (an integer, a short string, `.nil`). A heap
    /// handle another space minted decodes to an unrelated slot and
    /// generation. `None` for an address no `ObjRef` encodes to here.
    #[cfg(target_pointer_width = "64")]
    pub fn decode(&self, handle: RexxObjectPtr) -> Option<ObjRef> {
        let bits = u64::cast_from(handle.addr()).wrapping_sub(1);
        ObjRef::heap_bits(bits ^ self.key.load(std::sync::atomic::Ordering::Relaxed))
            .or_else(|| ObjRef::carried(bits))
    }

    /// The `ObjRef` a handle this space minted names, if its bits say: on a
    /// 32-bit target only an odd handle carries one, a heap object's
    /// reference or a carried value, with no key. A numbered handle resolves
    /// through the tables and holders that hold its number, not its bits.
    #[cfg(not(target_pointer_width = "64"))]
    pub fn decode(&self, handle: RexxObjectPtr) -> Option<ObjRef> {
        let address = handle.addr();
        if address & 1 == 0 {
            return None;
        }
        let bits = u64::cast_from(address >> 1);
        ObjRef::heap_bits(bits).or_else(|| ObjRef::carried(bits))
    }

    /// `object`'s handle: a heap object's bits mixed with the key, a carried
    /// value's as they are (a value is the same value in every interpreter).
    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn keyed(&self, object: ObjRef) -> usize {
        match object.decode() {
            interrexx_core::Decoded::Heap { .. } => {
                // The key's two low bits are clear, so the tag stays the heap's
                // and the address stays odd, never null.
                let key = self.key.load(std::sync::atomic::Ordering::Relaxed);
                usize::cast_from((object.bits() ^ key).wrapping_add(1))
            }
            _ => carried(object).expect("a 64-bit address carries every ObjRef"),
        }
    }

    /// `object`'s handle, as an address, held once more until a matching
    /// [`HandleSpace::unhold`].
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub fn hold(&self, object: ObjRef) -> usize {
        self.keyed(object)
    }

    /// `object`'s handle, as an address, without holding it: `None` when the
    /// address cannot carry it and nothing holds a number for it.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub fn peek(&self, object: ObjRef) -> Option<usize> {
        Some(self.keyed(object))
    }

    /// Lets go of one hold of `object`'s handle.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub fn unhold(&self, _object: ObjRef) {}

    /// `object`'s handle, as an address, held once more until a matching
    /// [`HandleSpace::unhold`].
    ///
    /// # Panics
    /// If every number an address carries is held.
    #[cfg(not(target_pointer_width = "64"))]
    pub fn hold(&self, object: ObjRef) -> usize {
        carried(object).unwrap_or_else(|| self.numbering().hold(object.bits()) << 1)
    }

    /// `object`'s handle, as an address, without holding it: `None` when the
    /// address cannot carry it and nothing holds a number for it.
    #[cfg(not(target_pointer_width = "64"))]
    pub fn peek(&self, object: ObjRef) -> Option<usize> {
        carried(object).or_else(|| {
            self.numbering()
                .peek(object.bits())
                .map(|number| number << 1)
        })
    }

    /// Lets go of one hold of `object`'s handle; the last one gives its
    /// number back.
    #[cfg(not(target_pointer_width = "64"))]
    pub fn unhold(&self, object: ObjRef) {
        if carried(object).is_none() {
            self.numbering().unhold(object.bits());
        }
    }

    /// How many `ObjRef`s hold a number, which is what the space's memory
    /// grows with.
    #[cfg(not(target_pointer_width = "64"))]
    pub fn numbered(&self) -> usize {
        self.numbering().held.len()
    }

    #[cfg(not(target_pointer_width = "64"))]
    fn numbering(&self) -> std::sync::MutexGuard<'_, Numbering> {
        self.numbering
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// The numbers of a [`HandleSpace`] on a narrow target.
#[cfg(not(target_pointer_width = "64"))]
struct Numbering {
    /// Each numbered `ObjRef`'s bits, its number and how many hold it.
    held: FxHashMap<u64, (usize, usize)>,
    /// The numbers in `held`.
    taken: rustc_hash::FxHashSet<usize>,
    /// The number handed out next, unless it is taken.
    next: usize,
    /// The largest number, after which the turn wraps to one.
    last: usize,
}

#[cfg(not(target_pointer_width = "64"))]
impl Default for Numbering {
    fn default() -> Numbering {
        Numbering::up_to(usize::MAX >> 1)
    }
}

#[cfg(not(target_pointer_width = "64"))]
impl Numbering {
    fn up_to(last: usize) -> Numbering {
        Numbering {
            held: FxHashMap::default(),
            taken: rustc_hash::FxHashSet::default(),
            next: 1,
            last,
        }
    }

    fn hold(&mut self, bits: u64) -> usize {
        if let Some((number, holders)) = self.held.get_mut(&bits) {
            *holders += 1;
            return *number;
        }
        assert!(
            self.taken.len() < self.last,
            "every handle an address can carry is held"
        );
        let number = loop {
            let number = self.next;
            self.next = if number == self.last { 1 } else { number + 1 };
            if !self.taken.contains(&number) {
                break number;
            }
        };
        self.taken.insert(number);
        self.held.insert(bits, (number, 1));
        number
    }

    fn peek(&self, bits: u64) -> Option<usize> {
        self.held.get(&bits).map(|(number, _)| *number)
    }

    fn unhold(&mut self, bits: u64) {
        let Some((number, holders)) = self.held.get_mut(&bits) else {
            return;
        };
        *holders -= 1;
        if *holders == 0 {
            let number = *number;
            self.held.remove(&bits);
            self.taken.remove(&number);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The constants resolve like any registration, survive a release of
    /// their handle, are counted as roots, and are forgotten by a clear.
    #[test]
    fn constants_resolve_until_the_table_is_cleared() {
        let objects = [0, 1, 2, 3].map(|slot| ObjRef::heap(slot, 7));
        let other = ObjRef::heap(9, 7);
        let mut table = Table::new(&HandleSpace::new());
        let handles = table.register_constants(objects);
        let plain = table.register(other);
        for (handle, object) in handles.iter().zip(objects) {
            assert_eq!(table.resolve(*handle), Some(object));
            assert!(table.holds(object));
        }
        assert_eq!(handles[0], table.register(objects[0]));
        table.release(handles[0], |_| true);
        assert_eq!(table.resolve(handles[0]), Some(objects[0]));
        let mut roots: Vec<ObjRef> = table.roots().collect();
        roots.sort_by_key(|object| object.bits());
        assert_eq!(roots, [objects.as_slice(), &[other]].concat());
        table.clear();
        assert_eq!(table.resolve(handles[1]), None);
        assert_eq!(table.resolve(plain), None);
        assert!(!table.holds(objects[1]));
    }

    /// On a narrow target the references carried in the address and the
    /// numbered ones never share a handle, none is null, and the numbered
    /// ones answer the same handle in every table of the space.
    #[cfg(not(target_pointer_width = "64"))]
    #[test]
    fn narrow_handles_are_distinct_and_shared_between_tables() {
        let carried = [ObjRef::heap(0, 0), ObjRef::heap(1, 0), ObjRef::NIL];
        let numbered = [ObjRef::heap(0, 1), ObjRef::heap(1, 1), ObjRef::heap(0, 2)];
        let space = HandleSpace::new();
        let mut first = Table::new(&space);
        let mut second = Table::new(&space);
        let handles: Vec<RexxObjectPtr> = carried
            .iter()
            .chain(&numbered)
            .map(|object| first.register(*object))
            .collect();
        for (at, handle) in handles.iter().enumerate() {
            assert!(!handle.is_null());
            assert!(handles[..at].iter().all(|other| other != handle));
        }
        for (object, handle) in carried.iter().chain(&numbered).zip(&handles) {
            assert_eq!(second.register(*object), *handle);
            assert_eq!(second.resolve(*handle), Some(*object));
        }
        for object in carried {
            assert_eq!(handle_of(object), first.register(object));
        }
        assert!(
            handles[..carried.len()]
                .iter()
                .all(|handle| handle.addr() & 1 == 1)
        );
        assert!(
            handles[carried.len()..]
                .iter()
                .all(|handle| handle.addr() & 1 == 0)
        );
    }

    /// A number stays held while any table holds its `ObjRef`, registered or
    /// released, and is given back when the last one clears or is dropped.
    #[cfg(not(target_pointer_width = "64"))]
    #[test]
    fn a_number_is_given_back_when_no_table_holds_it() {
        let space = HandleSpace::new();
        let object = ObjRef::heap(3, 9);
        let mut first = Table::new(&space);
        let mut second = Table::new(&space);
        let handle = first.register(object);
        assert_eq!(first.register(object), handle);
        assert_eq!(second.register(object), handle);
        assert_eq!(space.numbered(), 1);
        first.release(handle, |_| true);
        assert_eq!(first.register(object), handle);
        first.release(handle, |_| true);
        first.clear();
        assert_eq!(space.numbered(), 1, "the second table still holds it");
        assert_eq!(second.resolve(handle), Some(object));
        drop(second);
        assert_eq!(space.numbered(), 0);
        assert!(!first.holds(object));
    }

    /// A number given back is not handed to another `ObjRef` until the turn
    /// has wrapped, and the turn skips numbers still held.
    #[cfg(not(target_pointer_width = "64"))]
    #[test]
    fn a_number_given_back_waits_for_the_turn_to_wrap() {
        let mut numbering = Numbering::up_to(3);
        assert_eq!(numbering.hold(10), 1);
        assert_eq!(numbering.hold(11), 2);
        numbering.unhold(10);
        assert_eq!(
            numbering.hold(12),
            3,
            "1 was given back but its turn has passed"
        );
        assert_eq!(
            numbering.hold(13),
            1,
            "the turn wraps to the number given back"
        );
        assert_eq!(numbering.peek(10), None);
        numbering.unhold(12);
        assert_eq!(
            numbering.hold(14),
            3,
            "the wrapped turn skips 2, still held"
        );
        assert_eq!(numbering.hold(11), 2, "a held ObjRef keeps its number");
    }
}
