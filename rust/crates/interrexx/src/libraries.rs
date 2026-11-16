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
    /// The libraries whose unloaders its termination ran, in the order they
    /// loaded, where this interpreter was the last live one: its drop closes
    /// them.
    last: Vec<Unload>,
    /// The round's numbers of the libraries this interpreter loaded first
    /// in the round, with their names: their loaders were handed its
    /// context.
    recorded: Vec<(u64, Vec<u8>)>,
    /// Whether a library it holds was found closed, which takes the command
    /// handlers extensions registered out of use once.
    closed_seen: bool,
    /// The thread this interpreter entered [`round_gate`] on, until it
    /// leaves.
    #[cfg(test)]
    gate: Option<ThreadId>,
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
    /// The threads waiting in [`Libraries::new`] or [`await_loader`].
    waits: Vec<Wait>,
    /// The number the next entry of `loaded` is known by.
    next: u64,
}

/// A thread waiting for hooks that other threads run.
struct Wait {
    /// The waiting thread's [`lineage`]: the thread last, after the threads
    /// that created the instance it runs, which may be waiting for it in
    /// ways this module does not see.
    blocked: Vec<ThreadId>,
    /// The threads running the hooks it waits for.
    holders: Vec<ThreadId>,
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
        waits: Vec::new(),
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

/// Whether this thread is to wait while `holder` runs a loader or unloaders.
fn waits_for(holder: ThreadId) -> bool {
    // A hook that creates an instance or loads a library itself waits for
    // that instance to start or that load to end, so the thread doing either
    // for it must not wait for the hook in turn.
    holder != std::thread::current().id()
        && !ANCESTORS.with_borrow(|ancestors| ancestors.contains(&holder))
}

/// Whether this thread, were it to wait for the hooks that `holders` run,
/// would wait for itself: whether one of them waits, directly or through
/// other waits, for this thread or for a thread of its [`lineage`].
fn closes_cycle(round: &Round, holders: &[ThreadId]) -> bool {
    let mine = lineage();
    let mut reached = holders.to_vec();
    let mut at = 0;
    while let Some(&thread) = reached.get(at) {
        at += 1;
        if mine.contains(&thread) {
            return true;
        }
        // A thread waits for what its own wait names, and so does every
        // thread of whose lineage it is part, as it may be waiting for that
        // thread to start or to answer.
        for wait in &round.waits {
            if wait.blocked.contains(&thread) {
                for holder in &wait.holders {
                    if !reached.contains(holder) {
                        reached.push(*holder);
                    }
                }
            }
        }
    }
    false
}

/// Waits on [`SETTLED`] for `holders` as this thread's wait, which
/// [`closes_cycle`] sees meanwhile.
fn wait_for(
    mut round: MutexGuard<'static, Round>,
    holders: Vec<ThreadId>,
) -> MutexGuard<'static, Round> {
    let blocked = lineage();
    let thread = std::thread::current().id();
    round.waits.push(Wait { blocked, holders });
    round = wait(round);
    if let Some(at) = (round.waits.iter()).position(|wait| wait.blocked.last() == Some(&thread)) {
        round.waits.remove(at);
    }
    round
}

/// The thread whose loader a load of `name` from the file `identity` is to
/// wait for: the one running the loader of that load, where this thread
/// waits for it and the wait would not close a cycle.
fn loader_to_wait_for(round: &Round, name: &[u8], identity: usize) -> Option<ThreadId> {
    let holder = round
        .loaded
        .iter()
        .find(|loaded| loaded.name == name && loaded.identity == identity)?
        .loading?;
    // A wait that would close a cycle is not made: the load finds the library
    // loaded, as a load from the thread running its loader does.
    (waits_for(holder) && !closes_cycle(round, &[holder])).then_some(holder)
}

/// Whether a load of `name` from the file `identity` is to wait, with
/// [`await_loader`], for a loader that another thread runs.
pub(crate) fn loader_to_await(name: &[u8], identity: usize) -> bool {
    loader_to_wait_for(&round(), name, identity).is_some()
}

/// Waits until a load of `name` from the file `identity` no longer has a
/// loader to wait for, which [`Libraries::hold`] answers `Err` for. Touches
/// nothing but the round, so a thread may wait here with its interpreter's
/// baton vacated.
pub(crate) fn await_loader(name: &[u8], identity: usize) {
    let mut round = round();
    while let Some(holder) = loader_to_wait_for(&round, name, identity) {
        round = wait_for(round, vec![holder]);
    }
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
/// once they have returned, ends the count of the instance that ran them and
/// lets the instances waiting to be created go on.
#[must_use]
pub(crate) struct Unloading {
    thread: ThreadId,
    /// The libraries [`Unloading::next`] has answered.
    walked: Vec<u64>,
}

impl Unloading {
    /// The libraries whose unloaders run next, with their names, in the order
    /// they run: those of the round no earlier call answered, which once the
    /// first call's have run are those an unloader loaded, leaving out each
    /// library whose loader another thread is still running. Answers nothing
    /// once every other library of the round has been answered
    /// (`r[native.loading.unload]`).
    pub(crate) fn next(&mut self) -> Vec<(Vec<u8>, Unload)> {
        let round = round();
        let mut batch = Vec::new();
        let thread = std::thread::current().id();
        for name in round.order.names() {
            for loaded in &round.loaded {
                // A library still loading on another thread is left in the
                // round to the instance that thread loads it for, which an
                // unloader created and which outlives this termination: its
                // own termination unloads it once the loader has returned.
                let loading_elsewhere = loaded.loading.is_some_and(|holder| holder != thread);
                if loaded.name == name && !loading_elsewhere && !self.walked.contains(&loaded.id) {
                    self.walked.push(loaded.id);
                    batch.push((loaded.name.clone(), loaded.unload.clone()));
                }
            }
        }
        batch
    }

    /// The libraries [`Unloading::next`] has answered, in the order they
    /// loaded.
    pub(crate) fn libraries(&self) -> Vec<Unload> {
        // The others are held by an instance that outlives this termination.
        round()
            .loaded
            .iter()
            .filter(|loaded| self.walked.contains(&loaded.id))
            .map(|loaded| loaded.unload.clone())
            .collect()
    }
}

impl Drop for Unloading {
    fn drop(&mut self) {
        let mut round = round();
        if let Some(at) = round.unloading.iter().position(|held| *held == self.thread) {
            round.unloading.remove(at);
        }
        round.live -= 1;
        // An instance an unloader created and left live goes on without the
        // libraries whose unloaders have run.
        let walked = std::mem::take(&mut self.walked);
        round.loaded.retain(|loaded| !walked.contains(&loaded.id));
        if round.live == 0 {
            round.loaded.clear();
            round.order = PackageTable::new();
        }
        drop(round);
        SETTLED.notify_all();
    }
}

impl Libraries {
    /// Counts a live instance, once no round's unloaders run on a thread this
    /// one waits for (`r[native.loading.again]`).
    pub(crate) fn new() -> Libraries {
        #[cfg(test)]
        let gate = Some(round_gate::enter());
        let mut round = round();
        loop {
            let holders: Vec<ThreadId> = (round.unloading.iter().copied())
                .filter(|holder| waits_for(*holder))
                .collect();
            // An instance whose wait would close a cycle joins the round
            // whose unloaders run, as one an unloader creates does.
            if holders.is_empty() || closes_cycle(&round, &holders) {
                break;
            }
            // While unloaders run, the live instances are the one whose
            // termination runs them, on the baton the unloading thread holds
            // or has vacated for the unloader, and those its unloaders
            // created, on threads of that thread's lineage, which do not wait
            // here. So a thread waiting here holds no baton the unloaders
            // need, and vacates none.
            round = wait_for(round, holders);
        }
        round.live += 1;
        Libraries {
            held: FxHashMap::default(),
            loaded: Vec::new(),
            live: true,
            last: Vec::new(),
            recorded: Vec::new(),
            closed_seen: false,
            #[cfg(test)]
            gate,
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
    /// name (`r[native.loading.shared]`). Where one has, `library` takes the
    /// mapping of the round's library instead of its own, so that it closes
    /// with the round (`r[native.loading.again]`).
    ///
    /// Answers `library` back as the error, holding nothing, where another
    /// thread runs the loader of that load and this thread is to wait for it
    /// first, with [`await_loader`], and then ask again.
    pub(crate) fn hold(
        &mut self,
        name: &[u8],
        mut library: Rc<Library>,
    ) -> Result<(Rc<Library>, Option<Loading>), Rc<Library>> {
        if let Some(held) = self.held.get(name) {
            return Ok((Rc::clone(held), None));
        }
        let identity = library.identity();
        let loading = {
            let mut round = round();
            if loader_to_wait_for(&round, name, identity).is_some() {
                return Err(library);
            }
            let known = round
                .loaded
                .iter()
                .find(|loaded| loaded.name == name && loaded.identity == identity)
                .map(|known| known.unload.clone());
            round.order.put(name);
            match known {
                Some(unload) => {
                    // Freshly opened, so this is its only holder.
                    if let Some(own) = Rc::get_mut(&mut library) {
                        own.share_mapping(&unload);
                    }
                    None
                }
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
                    self.recorded.push((id, name.to_vec()));
                    Some(Loading { id })
                }
            }
        };
        self.loaded.push(name.to_vec());
        self.held.insert(name.to_vec(), Rc::clone(&library));
        Ok((library, loading))
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
    /// last, answers the [`Unloading`] that walks the libraries of the process
    /// and is held while their unloaders run. This interpreter counts as live
    /// until that is dropped, and the next instance starts afresh
    /// (`r[native.loading.again]`) then. Otherwise answers nothing, and the
    /// libraries stay loaded for the instances still live. Answers nothing
    /// once ended.
    pub(crate) fn end(&mut self) -> Option<Unloading> {
        if !std::mem::replace(&mut self.live, false) {
            return None;
        }
        let mut round = round();
        if round.live > 1 {
            round.live -= 1;
            #[cfg(test)]
            self.leave_gate();
            // The library may call through the context its loader was
            // handed until it is closed, after this interpreter has gone.
            // Only the instance whose load a round recorded keeps one, so an
            // instance keeps at most one context per library it loaded first,
            // beside the root context and the instance block that every
            // instance keeps for the process. A load from the main activity
            // was handed that root, so it keeps nothing more.
            for (id, name) in &self.recorded {
                if round.loaded.iter().any(|loaded| loaded.id == *id)
                    && let Some(library) = self.held.get(name)
                {
                    library.keep_thread_context_for_process();
                }
            }
            return None;
        }
        // The libraries stay findable while the unloaders run, so that an
        // instance an unloader creates shares them (`r[native.loading.reentrant]`).
        let thread = std::thread::current().id();
        round.unloading.push(thread);
        Some(Unloading {
            thread,
            walked: Vec::new(),
        })
    }

    /// Whether a library this interpreter holds has been closed since the
    /// first time this answered true, which it does once: a library of the
    /// round closes with the round, whichever instance's termination closes
    /// it, and its first close takes the command handlers extensions
    /// registered out of use, as [`Interp::run_package_unloaders`] does for
    /// the instance that closes it.
    pub(crate) fn first_closed(&mut self) -> bool {
        if self.closed_seen || self.held.values().all(|library| library.is_open()) {
            return false;
        }
        self.closed_seen = true;
        true
    }

    /// Records that this interpreter's own termination closed a library and
    /// took the command handlers out of use ([`Libraries::first_closed`]).
    pub(crate) fn note_closed(&mut self) {
        self.closed_seen = true;
    }

    /// Takes `unloading` once its unloaders have run, and drops it, after
    /// which this interpreter's drop closes every library it walked.
    pub(crate) fn unloaded(&mut self, unloading: Unloading) {
        self.last = unloading.libraries();
        drop(unloading);
        #[cfg(test)]
        self.leave_gate();
    }

    /// Leaves [`round_gate`] where this interpreter is still in it.
    #[cfg(test)]
    fn leave_gate(&mut self) {
        if let Some(thread) = self.gate.take() {
            round_gate::leave(thread);
        }
    }
}

/// Ends the count of an interpreter that nothing ended, and closes every
/// library the termination's walk reached, or this drop's own walk where
/// nothing terminated it, in the order they loaded, which is the order
/// process exit runs their destructors in, after an unloader raised too. A
/// library still loading on another thread, and one a walk that ended early
/// did not reach, is not closed here. An interpreter that was not the last
/// live one closes nothing: the libraries stay loaded for the others.
impl Drop for Libraries {
    fn drop(&mut self) {
        // No unloader runs here: outside the unit tests, an interpreter that
        // nothing terminated is dropped only while a panic unwinds out of its
        // run, when its state is not one extension code can call back into.
        if let Some(mut unloading) = self.end() {
            // The walk runs without its unloaders, so that what it answers,
            // which leaves out a library still loading on another thread, is
            // closed below.
            while !unloading.next().is_empty() {}
            self.unloaded(unloading);
        }
        #[cfg(test)]
        self.leave_gate();
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

/// The round is the process's, so a test whose assertions read it, such as
/// one that asks whether an unloader ran, sees every other interpreter of a
/// `cargo test` process that is live meanwhile. Such a test holds
/// [`round_gate::exclusive`] for its length: it waits until no other
/// interpreter is live, and no interpreter is created meanwhile but its own
/// and those of the threads it started. nextest runs each test in a process
/// of its own and these one at a time besides.
#[cfg(test)]
pub(crate) mod round_gate {
    use std::sync::{Condvar, Mutex, MutexGuard};
    use std::thread::ThreadId;
    use std::time::Duration;

    struct Gate {
        /// The thread of the test holding the gate.
        exclusive: Option<ThreadId>,
        /// How many tests wait to hold it.
        waiting: usize,
        /// The thread each live interpreter entered on, one entry each.
        live: Vec<ThreadId>,
    }

    static GATE: Mutex<Gate> = Mutex::new(Gate {
        exclusive: None,
        waiting: 0,
        live: Vec::new(),
    });

    static CHANGED: Condvar = Condvar::new();

    /// How long a test waits for the other interpreters to end before it
    /// fails, rather than hang where one of them never ends.
    const PATIENCE: Duration = Duration::from_secs(120);

    std::thread_local! {
        /// The threads that started this one, through
        /// [`crate::on_interpreter_thread`] or a [`super::lineage`], which a
        /// test's gate treats as the test's own.
        static STARTED_BY: std::cell::RefCell<Vec<ThreadId>> =
            const { std::cell::RefCell::new(Vec::new()) };
    }

    fn gate() -> MutexGuard<'static, Gate> {
        GATE.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// This thread and the threads that started it.
    pub(crate) fn family() -> Vec<ThreadId> {
        let mut family = super::lineage();
        STARTED_BY.with_borrow(|started| family.extend(started.iter().copied()));
        family
    }

    /// Makes `family`, the starting thread's, this thread's own starters.
    pub(crate) fn started_by(family: Vec<ThreadId>) {
        STARTED_BY.set(family);
    }

    /// Enters an interpreter created on this thread: at once where a thread of
    /// its family holds the gate or runs a live interpreter, and otherwise
    /// once no test holds the gate or waits for it.
    pub(super) fn enter() -> ThreadId {
        let family = family();
        let mut gate = gate();
        loop {
            let ours = gate.exclusive.is_some_and(|held| family.contains(&held))
                || gate.live.iter().any(|live| family.contains(live));
            if ours || (gate.exclusive.is_none() && gate.waiting == 0) {
                break;
            }
            gate = CHANGED
                .wait(gate)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        let thread = std::thread::current().id();
        gate.live.push(thread);
        thread
    }

    /// Leaves for an interpreter that [`enter`] answered `thread` for.
    pub(super) fn leave(thread: ThreadId) {
        let mut gate = gate();
        if let Some(at) = gate.live.iter().position(|live| *live == thread) {
            gate.live.swap_remove(at);
        }
        drop(gate);
        CHANGED.notify_all();
    }

    /// Holds the gate until dropped.
    pub(crate) struct Exclusive;

    /// Waits until no other interpreter is live and no other test holds the
    /// gate, and holds it.
    ///
    /// # Panics
    ///
    /// Where that has not happened within [`PATIENCE`].
    #[expect(
        clippy::disallowed_methods,
        reason = "a bound on how long a test waits for others, in real time"
    )]
    pub(crate) fn exclusive() -> Exclusive {
        let mut gate = gate();
        gate.waiting += 1;
        let began = std::time::Instant::now();
        while gate.exclusive.is_some() || !gate.live.is_empty() {
            gate = CHANGED
                .wait_timeout(gate, Duration::from_millis(100))
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .0;
            if began.elapsed() >= PATIENCE {
                gate.waiting -= 1;
                drop(gate);
                CHANGED.notify_all();
                panic!("another test's interpreter stayed live");
            }
        }
        gate.waiting -= 1;
        gate.exclusive = Some(std::thread::current().id());
        Exclusive
    }

    impl Drop for Exclusive {
        fn drop(&mut self) {
            gate().exclusive = None;
            CHANGED.notify_all();
        }
    }
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
        let _round = super::round_gate::exclusive();
        let mut libraries = Libraries::new();
        let first = Rc::new(interrexx_api::load::hooks_only(None, None));
        let second = Rc::new(interrexx_api::load::hooks_only(None, None));
        let (held, _) = libraries
            .hold(b"lib", Rc::clone(&first))
            .unwrap_or_else(|_| panic!("no other thread loads it"));
        assert!(Rc::ptr_eq(&held, &first));
        let (again, loads) = libraries
            .hold(b"lib", second)
            .unwrap_or_else(|_| panic!("no other thread loads it"));
        assert!(Rc::ptr_eq(&again, &first));
        assert!(loads.is_none());
        assert!(
            libraries
                .get(b"lib")
                .is_some_and(|got| Rc::ptr_eq(got, &first))
        );
    }

    /// The walk of the last instance's termination leaves out a library whose
    /// loader another thread still runs, and the libraries it answers for the
    /// drop to close leave it out too, also once that loader has returned:
    /// the library belongs to an instance that outlives the termination.
    ///
    /// The round is the process's, so this holds [`super::round_gate`].
    #[test]
    fn the_walk_leaves_a_library_loading_on_another_thread() {
        let _round = super::round_gate::exclusive();
        let mut libraries = Libraries::new();
        let walked = Rc::new(interrexx_api::load::hooks_only(None, None));
        let (_, loading) = libraries
            .hold(b"walked", Rc::clone(&walked))
            .unwrap_or_else(|_| panic!("no other thread loads it"));
        drop(loading);
        let mut unloading = libraries.end().expect("the only live instance is the last");
        let other = std::thread::spawn(|| std::thread::current().id())
            .join()
            .expect("the thread returns its id");
        let elsewhere = interrexx_api::load::hooks_only(None, None);
        {
            let mut round = super::round();
            round.order.put(b"elsewhere");
            let id = round.next;
            round.next += 1;
            round.loaded.push(super::Loaded {
                id,
                name: b"elsewhere".to_vec(),
                identity: elsewhere.identity(),
                unload: elsewhere.unload(),
                loading: Some(other),
            });
        }
        let names: Vec<Vec<u8>> = unloading.next().into_iter().map(|(name, _)| name).collect();
        assert_eq!(names, [b"walked".to_vec()]);
        assert!(unloading.next().is_empty());
        for loaded in &mut super::round().loaded {
            loaded.loading = None;
        }
        let closed = unloading.libraries();
        assert_eq!(closed.len(), 1);
        assert!(walked.is_unloaded_by(&closed[0]));
    }

    /// The internal packages' keys, in the order the table iterates them; a
    /// miss on `REXXUTIL` leaves them as they were.
    ///
    /// The table is the process's, so this needs a round no other live
    /// interpreter is in, which [`super::round_gate`] gives it.
    #[test]
    fn the_table_starts_with_the_internal_packages_keys() {
        let _round = super::round_gate::exclusive();
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
