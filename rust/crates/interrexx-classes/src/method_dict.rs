//! `MethodDict`: a flat name-to-method map plus scope ordering.

use interrexx_core::{MethodId, NameMap, ObjRef};
use std::collections::BTreeSet;
use std::collections::hash_map::Entry;
use std::rc::Rc;

/// One entry under one name. Several of these can share a name when more
/// than one class in the ancestor chain defines it -- that is exactly the
/// case a scope-override (`SUPER`) send has to distinguish.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MethodSlot {
    /// A real, invokable entry: the scope it was defined at and the method
    /// it names.
    Defined { scope: ObjRef, method: MethodId },
    /// The `.nil` tombstone. It carries neither scope nor method, which is not
    /// an omission but the shape of the hide operation, a `put` of `.nil` under
    /// the name: the two dictionary walks that read scopes (rescoping and
    /// scope-override lookup) each test for `.nil` and skip it rather than
    /// asking it for one.
    Hidden,
    /// What `define` with `.nil` as the method leaves: hides the name from
    /// the search as [`MethodSlot::Hidden`] does, but `Class~method` finds
    /// no entry under it. `delete` removes it.
    Withdrawn,
}

impl MethodSlot {
    /// The scope this entry was defined at, or `None` for a tombstone.
    fn scope(self) -> Option<ObjRef> {
        match self {
            MethodSlot::Defined { scope, .. } => Some(scope),
            MethodSlot::Hidden | MethodSlot::Withdrawn => None,
        }
    }
}

/// The entries under one name, highest priority first. Most names hold one,
/// which is kept inline.
#[derive(Clone, Debug)]
enum Slots {
    One(MethodSlot),
    Many(Vec<MethodSlot>),
}

impl Slots {
    fn as_slice(&self) -> &[MethodSlot] {
        match self {
            Slots::One(slot) => std::slice::from_ref(slot),
            Slots::Many(slots) => slots,
        }
    }

    fn as_mut_slice(&mut self) -> &mut [MethodSlot] {
        match self {
            Slots::One(slot) => std::slice::from_mut(slot),
            Slots::Many(slots) => slots,
        }
    }

    /// Puts `slot` ahead of every entry already here.
    fn push_front(&mut self, slot: MethodSlot) {
        match self {
            Slots::One(only) => *self = Slots::Many(vec![slot, *only]),
            Slots::Many(slots) => slots.insert(0, slot),
        }
    }
}

/// A name as a dictionary keys it: upper case, and shared by every dictionary
/// the name is merged into.
type Name = Rc<str>;

/// A flat name-to-method(s) map, plus the scope ordering a scope-override
/// send needs to resolve correctly.
#[derive(Clone, Default)]
pub struct MethodDict {
    entries: NameMap<Name, Slots>,
    /// Every scope folded in, in the order it was. What a scope sees past
    /// itself is the scopes folded in before it, the part of this list ahead
    /// of it, so its position is all a scope-override send needs.
    scope_list: Vec<ObjRef>,
    /// At least the largest method any entry names, and `None` while none
    /// has been entered since the dictionary was built or cleared: an upper
    /// bound, kept by every write that enters a method and never lowered by
    /// one that takes a method out, so a search for a method newer than it
    /// can pass the dictionary by without reading its entries.
    newest: Option<MethodId>,
}

impl MethodDict {
    pub fn new() -> Self {
        Self::default()
    }

    /// Discard everything, the step every cascade takes before rebuilding a
    /// behaviour from scratch.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Room for `names` names without growing.
    pub fn reserve(&mut self, names: usize) {
        self.entries.reserve(names);
    }

    /// The scopes folded in before `scope`, or `None` when `scope` never was.
    fn scopes_before(&self, scope: ObjRef) -> Option<&[ObjRef]> {
        let at = self.scope_list.iter().position(|folded| *folded == scope)?;
        Some(&self.scope_list[..at])
    }

    /// True once `scope` has been folded into this dictionary. This is the
    /// cascade's own de-duplication guard: the instance- and class-behaviour
    /// construction both skip a superclass already merged, which is what makes a
    /// diamond ancestor safe to reach by more than one path.
    pub fn has_scope(&self, scope: ObjRef) -> bool {
        self.scope_list.contains(&scope)
    }

    /// Fold a new scope into the ordering, after everything folded in so far.
    /// A repeat is a no-op.
    pub fn add_scope(&mut self, scope: ObjRef) {
        if !self.has_scope(scope) {
            self.scope_list.push(scope);
        }
    }

    /// The entries under `key` where it holds some, and otherwise `None`, having
    /// entered `slot` as its only entry.
    fn add_slot(&mut self, key: &Name, slot: MethodSlot) -> Option<&mut Slots> {
        match self.entries.entry(Rc::clone(key)) {
            Entry::Occupied(slots) => Some(slots.into_mut()),
            Entry::Vacant(vacant) => {
                vacant.insert(Slots::One(slot));
                None
            }
        }
    }

    /// [`Self::add_method`] for a key already upper case.
    fn add_method_key(&mut self, key: &Name, scope: ObjRef, method: MethodId) {
        self.note_method(method);
        let slot = MethodSlot::Defined { scope, method };
        let Some(list) = self.add_slot(key, slot) else {
            return;
        };
        if let Some(existing) = list
            .as_mut_slice()
            .iter_mut()
            .find(|e| e.scope() == Some(scope))
        {
            *existing = slot;
            return;
        }
        list.push_front(slot);
    }

    /// [`Self::add_hidden`] for a key already upper case.
    fn add_hidden_key(&mut self, key: &Name) {
        if let Some(list) = self.add_slot(key, MethodSlot::Hidden) {
            list.push_front(MethodSlot::Hidden);
        }
    }

    /// `name` in upper case, as a key.
    fn key(name: &str) -> Name {
        Rc::from(name.to_ascii_uppercase())
    }

    /// Add one method under `name`. A pre-existing entry from the same `scope`
    /// is replaced in place; otherwise the new entry becomes the
    /// highest-priority entry for `name`.
    pub fn add_method(&mut self, name: &str, scope: ObjRef, method: MethodId) {
        self.add_method_key(&Self::key(name), scope, method);
    }

    /// Replace everything under `name` with one entry, which is a bare `put`:
    /// the replaced name may have been inherited from another class, so any
    /// existing method by that name is completely removed.
    pub fn replace_method(&mut self, name: &str, scope: ObjRef, method: MethodId) {
        self.note_method(method);
        self.entries.insert(
            Self::key(name),
            Slots::One(MethodSlot::Defined { scope, method }),
        );
    }

    /// Repoint the entry under `name` that `scope` defines as `old` to
    /// `new`, leaving its place in the list, and answer whether there was one.
    pub fn repoint(&mut self, name: &str, scope: ObjRef, old: MethodId, new: MethodId) -> bool {
        let Some(list) = self.entries.get_mut(name.to_ascii_uppercase().as_str()) else {
            return false;
        };
        let mut found = false;
        for entry in list.as_mut_slice() {
            if *entry == (MethodSlot::Defined { scope, method: old }) {
                *entry = MethodSlot::Defined { scope, method: new };
                found = true;
            }
        }
        if found {
            self.note_method(new);
        }
        found
    }

    /// Raise [`Self::newest`] to cover `method`.
    fn note_method(&mut self, method: MethodId) {
        if self.newest.is_none_or(|newest| newest.0 < method.0) {
            self.newest = Some(method);
        }
    }

    /// No entry names a method larger than this, and none names any method
    /// where it is `None`; see the field.
    pub fn newest(&self) -> Option<MethodId> {
        self.newest
    }

    /// Add a tombstone under `name` as its highest-priority entry, leaving
    /// whatever was already there behind it -- the no-method branch of adding a
    /// method, which is an unconditional push of `.nil`.
    pub fn add_hidden(&mut self, name: &str) {
        self.add_hidden_key(&Self::key(name));
    }

    /// Replace everything under `name` with a tombstone, and the
    /// omitted-second-argument arm of `~define`, which reaches the same
    /// replacing `put` with the same `.nil`.
    pub fn hide_method(&mut self, name: &str) {
        self.entries
            .insert(Self::key(name), Slots::One(MethodSlot::Hidden));
    }

    /// Replace everything under `name` with a [`MethodSlot::Withdrawn`]
    /// mark.
    pub fn withdraw_method(&mut self, name: &str) {
        self.entries
            .insert(Self::key(name), Slots::One(MethodSlot::Withdrawn));
    }

    /// Take out the entries under `name` that `scope` defined, leaving the
    /// others in their order, and the name itself when that leaves none.
    pub fn remove_scoped(&mut self, name: &str, scope: ObjRef) {
        let key = name.to_ascii_uppercase();
        let Some(list) = self.entries.get_mut(key.as_str()) else {
            return;
        };
        let kept: Vec<MethodSlot> = list
            .as_slice()
            .iter()
            .copied()
            .filter(|slot| slot.scope() != Some(scope))
            .collect();
        match kept.as_slice() {
            [] => {
                self.entries.remove(key.as_str());
            }
            [only] => *list = Slots::One(*only),
            _ => *list = Slots::Many(kept),
        }
    }

    /// Whether the two hold the same entries under the same names, in the same
    /// order under each name, and the same scopes in the same order: what makes
    /// them answer every lookup alike.
    pub fn same_as(&self, other: &MethodDict) -> bool {
        self.scope_list == other.scope_list
            && self.entries.len() == other.entries.len()
            && self.entries.iter().all(|(name, slots)| {
                other
                    .entries
                    .get(name)
                    .is_some_and(|theirs| theirs.as_slice() == slots.as_slice())
            })
    }

    /// Take `name` out of the dictionary entirely, whose `bool` says whether
    /// there was anything to remove. `~delete` reads that answer: it propagates
    /// to the subclasses only when the dictionary actually changed.
    pub fn remove_method(&mut self, name: &str) -> bool {
        self.entries
            .remove(name.to_ascii_uppercase().as_str())
            .is_some()
    }

    /// The highest-priority entry under `name`, tombstone included, which
    /// `~method` reads and method lookup filters.
    pub fn slot(&self, name: &str) -> Option<MethodSlot> {
        // Every send arrives here, and the uppercase copy was an allocation
        // per send. `to_ascii_uppercase` changes a name exactly when it
        // holds an ASCII lower-case byte, so the two arms answer alike and
        // the common one -- a name the scanner already upcased -- borrows.
        let list = if name.bytes().any(|b| b.is_ascii_lowercase()) {
            self.entries.get(name.to_ascii_uppercase().as_str())
        } else {
            self.entries.get(name)
        };
        list.and_then(|l| l.as_slice().first()).copied()
    }

    /// Overlay `source`'s methods on top of this dictionary's own. Iterating
    /// `source` in reverse priority order and folding each entry in through
    /// [`Self::add_method`] is what preserves `source`'s own relative priority
    /// after the merge: so the methods get added to this directory in the same
    /// relative order that they appear in the source directory.
    pub fn merge_methods(&mut self, source: &MethodDict) {
        for (name, list) in &source.entries {
            for entry in list.as_slice().iter().rev() {
                match *entry {
                    MethodSlot::Defined { scope, method } => {
                        self.add_method_key(name, scope, method);
                    }
                    MethodSlot::Hidden | MethodSlot::Withdrawn => self.add_hidden_key(name),
                }
            }
        }
    }

    /// [`Self::merge_methods`] for the names in `keys` alone, each already
    /// upper case: what the merge makes of every other name is left as it
    /// stands, which is what a cascade that knows only `keys` changed needs.
    pub fn merge_methods_named(&mut self, source: &MethodDict, keys: &[String]) {
        for key in keys {
            let Some((name, list)) = source.entries.get_key_value(key.as_str()) else {
                continue;
            };
            for entry in list.as_slice().iter().rev() {
                match *entry {
                    MethodSlot::Defined { scope, method } => {
                        self.add_method_key(name, scope, method);
                    }
                    MethodSlot::Hidden | MethodSlot::Withdrawn => self.add_hidden_key(name),
                }
            }
        }
    }

    /// What `from` holds under `key` (upper case), moved here in place of
    /// what was there, or no entry under it where `from` holds none.
    pub fn adopt_entry(&mut self, from: &mut MethodDict, key: &str) {
        let Some((name, list)) = from.entries.remove_entry(key) else {
            self.entries.remove(key);
            return;
        };
        for slot in list.as_slice() {
            if let MethodSlot::Defined { method, .. } = *slot {
                self.note_method(method);
            }
        }
        match self.entries.get_mut(key) {
            Some(held) => *held = list,
            None => {
                self.entries.insert(name, list);
            }
        }
    }

    /// Whether `key` (upper case) holds an entry, tombstone included.
    pub fn has_entry(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Whether the two hold the same entries under every name and the same
    /// scopes, and list them in the same order, which `Class~methods`
    /// answers in: what a cascade over the same inputs in the same order
    /// makes.
    #[cfg(debug_assertions)]
    pub fn same_contents(&self, other: &MethodDict) -> bool {
        self.entries
            .iter()
            .map(|(name, list)| (name, list.as_slice()))
            .eq(other
                .entries
                .iter()
                .map(|(name, list)| (name, list.as_slice())))
            && self.scope_list == other.scope_list
    }

    /// Overlay `source`'s methods **and its already-folded scope order** on top
    /// of this dictionary, which does a method merge (identical to
    /// [`Self::merge_methods`]) followed by a scope merge, which folds every
    /// scope already in `source`'s own `scope_list`, in `source`'s order, into
    /// this one via [`Self::add_scope`]. This is distinct from
    /// [`Self::merge_methods`] alone: that merges methods only, and is what an
    /// ordinary `INHERIT` mixin cascade uses (`class_graph.rs`'s `cascade_build`)
    /// because a mixin's own ancestors are walked independently by that same
    /// cascade. This method is for the one case where the source's scope history
    /// has to come along wholesale instead of being re-derived: the
    /// class-behaviour metaclass merge, where a class's class-behaviour absorbs
    /// its metaclass's already-built instance behaviour verbatim.
    pub fn merge(&mut self, source: &MethodDict) {
        self.merge_methods(source);
        for &scope in &source.scope_list {
            self.add_scope(scope);
        }
    }

    /// Copy `source`'s entries **defined at `filter`** into this dictionary
    /// under `scope`, leaving `source` untouched -- a method-table replacement
    /// filtered by scope.
    pub fn replace_methods_from(&mut self, source: &MethodDict, filter: ObjRef, scope: ObjRef) {
        for (name, list) in &source.entries {
            for entry in list.as_slice().iter().rev() {
                match *entry {
                    MethodSlot::Defined {
                        scope: defined_at,
                        method,
                    } if defined_at == filter => self.add_method_key(name, scope, method),
                    MethodSlot::Defined { .. } => {}
                    MethodSlot::Hidden | MethodSlot::Withdrawn => self.add_hidden_key(name),
                }
            }
        }
    }

    /// Rewrite every entry's scope to `scope`, what
    /// `Class~inheritInstanceMethods` uses to make a donor's methods present
    /// under the recipient's own identity rather than the donor's.
    pub fn set_method_scope(&mut self, scope: ObjRef) {
        for list in self.entries.values_mut() {
            for entry in list.as_mut_slice() {
                if let MethodSlot::Defined { method, .. } = *entry {
                    *entry = MethodSlot::Defined { scope, method };
                }
            }
        }
    }

    /// Whether `name`, already upper case, holds an entry, tombstones included.
    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    /// How many entries `name`, already upper case, holds, every scope's and
    /// tombstones included.
    pub fn entry_count(&self, name: &str) -> usize {
        self.entries
            .get(name)
            .map_or(0, |list| list.as_slice().len())
    }

    /// How many names hold an entry, tombstones included.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// The method of every defined entry, at every scope.
    pub fn methods(&self) -> impl Iterator<Item = MethodId> + '_ {
        self.entries
            .values()
            .flat_map(Slots::as_slice)
            .filter_map(|slot| match slot {
                MethodSlot::Defined { method, .. } => Some(*method),
                MethodSlot::Hidden | MethodSlot::Withdrawn => None,
            })
    }

    /// Whether no name holds an entry.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// True if `name` resolves, which answers false for a name whose
    /// highest-priority entry is a tombstone.
    pub fn has_method(&self, name: &str) -> bool {
        matches!(self.slot(name), Some(MethodSlot::Defined { .. }))
    }

    /// Every name this dictionary answers to, regardless of which scope
    /// supplies it. A name whose highest-priority entry is a tombstone is
    /// not one of them, matching [`Self::has_method`]. This is what a **method-set** assertion reads --
    /// `superClasses`/`ancestors` cannot see a donation that added no
    /// scope, but this can.
    pub fn method_names(&self) -> BTreeSet<String> {
        self.entries
            .iter()
            .filter(|(_, l)| matches!(l.as_slice().first(), Some(MethodSlot::Defined { .. })))
            .map(|(name, _)| name.to_string())
            .collect()
    }

    /// Every defined entry under every name, shadowed ones and ones behind a
    /// tombstone included -- what a whole-dictionary method walk visits.
    pub fn defined_entries(&self) -> impl Iterator<Item = (&str, ObjRef, MethodId)> + '_ {
        self.entries.iter().flat_map(|(name, list)| {
            list.as_slice().iter().filter_map(move |slot| match *slot {
                MethodSlot::Defined { scope, method } => Some((&**name, scope, method)),
                MethodSlot::Hidden | MethodSlot::Withdrawn => None,
            })
        })
    }

    /// The highest-priority method for an ordinary (unscoped) send --
    /// index 0 of `name`'s entry list.
    pub fn lookup(&self, name: &str) -> Option<(ObjRef, MethodId)> {
        match self.slot(name) {
            Some(MethodSlot::Defined { scope, method }) => Some((scope, method)),
            Some(MethodSlot::Hidden | MethodSlot::Withdrawn) | None => None,
        }
    }

    /// Resolve `name` as a scope-override send starting at `start_scope`, as a
    /// super-method lookup does: the first (highest priority) entry whose scope
    /// is `start_scope` itself, or one folded in before it.
    /// Returns `None` if `start_scope` was never folded into this dictionary,
    /// as a scope-table miss does.
    pub fn lookup_from_scope(&self, name: &str, start_scope: ObjRef) -> Option<(ObjRef, MethodId)> {
        let visible = self.scopes_before(start_scope)?;
        let list = self.entries.get(name.to_ascii_uppercase().as_str())?;
        list.as_slice()
            .iter()
            .filter_map(|e| match *e {
                MethodSlot::Defined { scope, method } => Some((scope, method)),
                MethodSlot::Hidden | MethodSlot::Withdrawn => None,
            })
            .find(|(scope, _)| *scope == start_scope || visible.contains(scope))
    }

    /// The immediate superscope for `scope`: the scope folded in just before
    /// it, or `None` for the topmost scope .
    pub fn resolve_super_scope(&self, scope: ObjRef) -> Option<ObjRef> {
        self.scopes_before(scope)?.last().copied()
    }
}
