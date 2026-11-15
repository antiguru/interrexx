//! What each library name has resolved to, and the libraries the process
//! has loaded for the instances that are live.
//!
//! A module of its own so that the map is private to the write rule below and
//! not to whichever module happens to resolve libraries.

use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Condvar, LazyLock, Mutex, MutexGuard};
use std::thread::ThreadId;

use interrexx_api::load::{Library, Unload};

/// Every library name that has loaded, with the library it loaded.
///
/// Only a library is held, never a miss: a package whose load answered false
/// is removed, so the next ask looks again.
///
/// A name's library is written once and afterwards neither removed nor
/// replaced, so a library this hands an [`Rc`] out for stays loaded as long as
/// the interpreter does. That is a correctness constraint: an object of a
/// class the library contributed a method to can stay reachable, and the
/// finalizer sweep then calls the library's `UNINIT`, the block a `CSELF`
/// addresses and entry points resolved out of the package tables.
///
/// Dropping the interpreter that was the last live one closes its libraries,
/// after which no row or hook of theirs is called. Closing never unmaps the
/// code (`r[native.loading.static]`), so an address of a library stays
/// callable for the rest of the process. The unloaders run at termination
/// ([`Libraries::end`]), not at the drop.
///
/// Each one counts as a live instance of the process ([`Round`]) from its
/// creation until [`Libraries::end`], or its drop where nothing ended it.
pub(crate) struct Libraries {
    held: FxHashMap<Vec<u8>, Rc<Library>>,
    /// The names held, in the order they loaded.
    loaded: Vec<Vec<u8>>,
    /// Whether this interpreter still counts as live.
    live: bool,
    /// The libraries of the process, in the order they loaded, where this
    /// interpreter was the last live one: its drop closes them.
    last: Vec<Unload>,
}

/// The libraries loaded for the instances live in the process, from the first
/// instance's creation to the end of the last one
/// (`r[native.loading.shared]`, `r[native.loading.unload]`).
struct Round {
    /// The instances live.
    live: usize,
    /// The names asked for, as the package table holds them, which is the
    /// order the unloaders run in.
    order: PackageTable,
    /// Every library loaded, in the order it loaded.
    loaded: Vec<Loaded>,
    /// The threads running the unloaders of a round that has ended, one entry
    /// for each such round, until they have returned.
    unloading: Vec<ThreadId>,
    /// The number the next entry of `loaded` is known by.
    next: u64,
}

/// One library a round has loaded.
struct Loaded {
    /// What [`Loading`] finds it by.
    id: u64,
    name: Vec<u8>,
    /// The file it came from ([`Library::identity`]).
    identity: usize,
    /// What its unloader needs.
    unload: Unload,
    /// The thread running its loader, until the loader has returned.
    loading: Option<ThreadId>,
}

static ROUND: LazyLock<Mutex<Round>> = LazyLock::new(|| {
    Mutex::new(Round {
        live: 0,
        order: PackageTable::new(),
        loaded: Vec::new(),
        unloading: Vec::new(),
        next: 0,
    })
});

/// Signalled whenever a loader or a round's unloaders return, which is what a
/// thread waiting in [`Libraries::new`] or [`Libraries::hold`] waits for.
static SETTLED: Condvar = Condvar::new();

fn round() -> MutexGuard<'static, Round> {
    ROUND
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn wait(round: MutexGuard<'static, Round>) -> MutexGuard<'static, Round> {
    SETTLED
        .wait(round)
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

thread_local! {
    /// The threads whose loaders and unloaders this thread does not wait for:
    /// those that created the instance it runs, directly or through instances
    /// in between ([`lineage`]).
    static ANCESTORS: RefCell<Vec<ThreadId>> = const { RefCell::new(Vec::new()) };
}

/// This thread and the threads it does not wait for, which a thread that runs
/// an instance this thread creates is to [`adopt`].
pub(crate) fn lineage() -> Vec<ThreadId> {
    let mut lineage = ANCESTORS.with_borrow(Clone::clone);
    lineage.push(std::thread::current().id());
    lineage
}

/// Makes this thread, which runs an instance created on another, not wait for
/// the threads of `lineage`.
pub(crate) fn adopt(lineage: Vec<ThreadId>) {
    ANCESTORS.set(lineage);
}

/// Whether this thread is to go on while `holder` runs a loader or unloaders.
fn waits_for(holder: ThreadId) -> bool {
    // A hook that creates an instance or loads a library itself waits for
    // that instance to start or that load to end, so the thread doing either
    // for it must not wait for the hook in turn.
    holder != std::thread::current().id()
        && !ANCESTORS.with_borrow(|ancestors| ancestors.contains(&holder))
}

/// A load whose loader the holder runs: dropping it, once the loader has
/// returned or unwound, lets the loads waiting for it go on.
#[must_use]
pub(crate) struct Loading {
    id: u64,
}

impl Drop for Loading {
    fn drop(&mut self) {
        let mut round = round();
        if let Some(loaded) = round.loaded.iter_mut().find(|loaded| loaded.id == self.id) {
            loaded.loading = None;
        }
        drop(round);
        SETTLED.notify_all();
    }
}

/// The unloaders of a round that has ended, which the holder runs: dropping it,
/// once they have returned, lets the instances waiting to be created go on.
#[must_use]
pub(crate) struct Unloading {
    thread: ThreadId,
}

impl Drop for Unloading {
    fn drop(&mut self) {
        let mut round = round();
        if let Some(at) = round.unloading.iter().position(|held| *held == self.thread) {
            round.unloading.remove(at);
        }
        drop(round);
        SETTLED.notify_all();
    }
}

impl Libraries {
    /// Counts a live instance, once no round's unloaders run on a thread this
    /// one waits for (`r[native.loading.again]`).
    pub(crate) fn new() -> Libraries {
        let mut round = round();
        while round.unloading.iter().any(|holder| waits_for(*holder)) {
            round = wait(round);
        }
        round.live += 1;
        Libraries {
            held: FxHashMap::default(),
            loaded: Vec::new(),
            live: true,
            last: Vec::new(),
        }
    }

    /// The library `name` loaded, or `None` for a name nothing is held for.
    pub(crate) fn get(&self, name: &[u8]) -> Option<&Rc<Library>> {
        self.held.get(name)
    }

    /// Holds `library` for `name` where nothing is held yet, and answers what
    /// is held afterwards, which for a name already held is the earlier
    /// library and not `library`, with a [`Loading`] where its loader is to
    /// run: it is where no live instance has loaded the same file under that
    /// name (`r[native.loading.shared]`).
    ///
    /// Where another thread runs the loader of that load, this answers once it
    /// has returned. A load that would run a loader waits too while another
    /// thread runs any loader, so two threads never each wait inside a loader
    /// for the other's.
    pub(crate) fn hold(
        &mut self,
        name: &[u8],
        library: Rc<Library>,
    ) -> (Rc<Library>, Option<Loading>) {
        if let Some(held) = self.held.get(name) {
            return (Rc::clone(held), None);
        }
        let identity = library.identity();
        let loading = {
            let mut round = round();
            let id = loop {
                let known = round
                    .loaded
                    .iter()
                    .find(|loaded| loaded.name == name && loaded.identity == identity);
                let waits = match known {
                    Some(known) => known.loading.is_some_and(waits_for),
                    None => round
                        .loaded
                        .iter()
                        .any(|loaded| loaded.loading.is_some_and(waits_for)),
                };
                if !waits {
                    break known.map(|known| known.id);
                }
                round = wait(round);
            };
            round.order.put(name);
            match id {
                Some(_) => None,
                None => {
                    let id = round.next;
                    round.next += 1;
                    round.loaded.push(Loaded {
                        id,
                        name: name.to_vec(),
                        identity,
                        unload: library.unload(),
                        loading: Some(std::thread::current().id()),
                    });
                    Some(Loading { id })
                }
            }
        };
        self.loaded.push(name.to_vec());
        self.held.insert(name.to_vec(), Rc::clone(&library));
        (library, loading)
    }

    /// Records an ask for `name` that loaded nothing, which the table sees as a
    /// put and then a remove.
    pub(crate) fn missed(&self, name: &[u8]) {
        let mut round = round();
        if !round.order.contains(name) {
            round.order.put(name);
            round.order.remove(name);
        }
    }

    /// Ends this interpreter's count as a live instance. Where it was the
    /// last, answers every library the process loaded, with its name, in the
    /// order their unloaders run, and an [`Unloading`] to hold while they run.
    /// The next instance starts afresh (`r[native.loading.again]`) once that
    /// is dropped, and this interpreter's drop closes the libraries.
    /// Otherwise answers nothing, and the libraries stay loaded for the
    /// instances still live. Answers nothing once ended.
    pub(crate) fn end(&mut self) -> (Vec<(Vec<u8>, Unload)>, Option<Unloading>) {
        if !std::mem::replace(&mut self.live, false) {
            return (Vec::new(), None);
        }
        let mut round = round();
        round.live -= 1;
        if round.live > 0 {
            // The library may call through the context its loader was
            // handed until it is closed, after this interpreter has gone.
            // Only the instance whose load a round recorded keeps one, so an
            // instance keeps at most one context per library it loaded first,
            // beside the root context and the instance block that every
            // instance keeps for the process. A load from the main activity
            // was handed that root, so it keeps nothing more.
            for loaded in &round.loaded {
                for library in self.held.values() {
                    if library.is_unloaded_by(&loaded.unload) {
                        library.keep_thread_context_for_process();
                    }
                }
            }
            return (Vec::new(), None);
        }
        let thread = std::thread::current().id();
        round.unloading.push(thread);
        let loaded = std::mem::take(&mut round.loaded);
        let order = std::mem::replace(&mut round.order, PackageTable::new());
        drop(round);
        let walk = order
            .names()
            .flat_map(|name| {
                loaded
                    .iter()
                    .filter(move |loaded| loaded.name == name)
                    .map(|loaded| (loaded.name.clone(), loaded.unload.clone()))
            })
            .collect();
        self.last = loaded.into_iter().map(|loaded| loaded.unload).collect();
        (walk, Some(Unloading { thread }))
    }
}

/// Ends the count of an interpreter that nothing ended, and closes every
/// library termination left open, in the order they loaded, which is the
/// order process exit runs their destructors in, after an unloader raised
/// too. An interpreter that was not the last live one closes nothing: the
/// libraries stay loaded for the others.
impl Drop for Libraries {
    fn drop(&mut self) {
        // No unloader runs here: outside the unit tests, an interpreter that
        // nothing terminated is dropped only while a panic unwinds out of its
        // run, when its state is not one extension code can call back into.
        let _ = self.end();
        for unload in &self.last {
            unload.library().close();
        }
        if self.last.is_empty() {
            return;
        }
        for name in &self.loaded {
            if let Some(library) = self.held.get(name) {
                library.close();
            }
        }
    }
}

/// The names in the package string table, laid out as its hash contents lay
/// them out, which is what fixes the order the table iterates in: bucket by
/// bucket, each bucket's chain in the order it was built.
struct PackageTable {
    buckets: usize,
    names: Vec<Option<Vec<u8>>>,
    next: Vec<Option<usize>>,
    /// The head of the free chain, which runs through the overflow half.
    free: Option<usize>,
}

impl PackageTable {
    /// The table built at startup: the default size, holding the two internal
    /// packages under the names `REXX` and `REXXUTIL`, which is spelled
    /// `rexxutil`.
    fn new() -> PackageTable {
        let mut table = PackageTable::with_buckets(bucket_size(DEFAULT_TABLE_SIZE));
        table.put(b"REXX");
        table.put(b"rexxutil");
        table
    }

    fn with_buckets(buckets: usize) -> PackageTable {
        let total = buckets * 2;
        PackageTable {
            buckets,
            names: vec![None; total],
            next: (0..total)
                .map(|slot| (slot >= buckets && slot + 1 < total).then_some(slot + 1))
                .collect(),
            free: Some(buckets),
        }
    }

    fn bucket(&self, name: &[u8]) -> usize {
        let buckets = u64::try_from(self.buckets).expect("a bucket count fits a u64");
        usize::try_from(string_hash(name) % buckets).expect("a bucket fits a usize")
    }

    fn contains(&self, name: &[u8]) -> bool {
        self.names().any(|held| held == name)
    }

    /// Adds a name as a hash collection does: expands first where the free chain is empty,
    /// then appends to the end of the chain.
    fn put(&mut self, name: &[u8]) {
        if self.free.is_none() {
            // `expandContents(capacity() * 2)`, whose capacity is the total.
            let mut larger = PackageTable::with_buckets(bucket_size(self.buckets * 4));
            for held in self.names() {
                larger.put(held);
            }
            *self = larger;
        }
        let mut at = self.bucket(name);
        if self.names[at].is_none() {
            self.names[at] = Some(name.to_vec());
            self.next[at] = None;
            return;
        }
        loop {
            if self.names[at].as_deref() == Some(name) {
                return;
            }
            match self.next[at] {
                Some(next) => at = next,
                None => break,
            }
        }
        let slot = self.free.expect("an expanded table has a free slot");
        self.free = self.next[slot];
        self.names[slot] = Some(name.to_vec());
        self.next[slot] = None;
        self.next[at] = Some(slot);
    }

    fn remove(&mut self, name: &[u8]) {
        let mut at = self.bucket(name);
        let mut previous = None;
        loop {
            match self.names[at].as_deref() {
                None => return,
                Some(held) if held == name => break,
                Some(_) => {}
            }
            previous = Some(at);
            match self.next[at] {
                Some(next) => at = next,
                None => return,
            }
        }
        match previous {
            None => match self.next[at] {
                None => self.names[at] = None,
                Some(next) => {
                    self.names[at] = self.names[next].take();
                    self.next[at] = self.next[next];
                    self.release(next);
                }
            },
            Some(previous) => {
                self.next[previous] = self.next[at];
                self.names[at] = None;
                self.release(at);
            }
        }
    }

    /// `returnToFreeChain`: the slot becomes the head of the free chain.
    fn release(&mut self, slot: usize) {
        self.next[slot] = self.free;
        self.free = Some(slot);
    }

    fn names(&self) -> impl Iterator<Item = &[u8]> {
        (0..self.buckets).flat_map(move |bucket| {
            let mut at = Some(bucket);
            std::iter::from_fn(move || {
                let slot = at?;
                let name = self.names[slot].as_deref()?;
                at = self.next[slot];
                Some(name)
            })
        })
    }
}

const DEFAULT_TABLE_SIZE: usize = 17;

const MINIMUM_BUCKET_SIZE: usize = 17;

/// The bucket size for `capacity`, below the table sizes this can reach.
fn bucket_size(capacity: usize) -> usize {
    let capacity = capacity.max(MINIMUM_BUCKET_SIZE);
    if capacity.is_multiple_of(2) {
        capacity + 1
    } else {
        capacity
    }
}

/// The string hash, over `char`, which is signed on this platform.
fn string_hash(name: &[u8]) -> u64 {
    name.iter().fold(0, |hash: u64, byte| {
        hash.wrapping_mul(31)
            .wrapping_add_signed(i64::from(i8::from_ne_bytes([*byte])))
    })
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::Libraries;

    /// A name's library is held once: a second hold answers the first, which
    /// is what keeps a routine row resolved in it valid for the
    /// interpreter's life.
    #[test]
    fn a_held_name_keeps_its_first_library() {
        let mut libraries = Libraries::new();
        let first = Rc::new(interrexx_api::load::hooks_only(None, None));
        let second = Rc::new(interrexx_api::load::hooks_only(None, None));
        let (held, _) = libraries.hold(b"lib", Rc::clone(&first));
        assert!(Rc::ptr_eq(&held, &first));
        let (again, loads) = libraries.hold(b"lib", second);
        assert!(Rc::ptr_eq(&again, &first));
        assert!(loads.is_none());
        assert!(
            libraries
                .get(b"lib")
                .is_some_and(|got| Rc::ptr_eq(got, &first))
        );
    }

    /// The internal packages' keys, in the order the table iterates them; a
    /// miss on `REXXUTIL` leaves them as they were.
    ///
    /// The table is the process's, so this needs a process in which no
    /// interpreter has asked for a library yet, which nextest gives each test.
    #[test]
    fn the_table_starts_with_the_internal_packages_keys() {
        let libraries = Libraries::new();
        let names = || {
            super::round()
                .order
                .names()
                .map(<[u8]>::to_vec)
                .collect::<Vec<_>>()
        };
        assert_eq!(names(), [b"rexxutil".to_vec(), b"REXX".to_vec()]);
        libraries.missed(b"REXXUTIL");
        assert_eq!(names(), [b"rexxutil".to_vec(), b"REXX".to_vec()]);
        assert!(!super::round().order.contains(b"REXXUTIL"));
    }
}
