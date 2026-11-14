//! The baton: the right to touch an interpreter's state, held by one thread
//! at a time. The thread that creates an
//! interpreter holds it. A holder may lend it, with a value `L`, to a named
//! thread and waits, touching nothing, until that thread gives it back; lends
//! nest, so the holders form a stack whose top alone runs.
//!
//! A holder may also vacate it, with a value `L`, for as long as it runs code
//! that touches nothing of the interpreter's: a lend to no thread yet, which
//! the vacating thread takes back directly where it is still free, and which
//! a thread that wants the baton meanwhile may claim and hand to a thread of
//! its choosing. The vacating thread then waits until that thread gives it
//! back, as a lender does.

use crate::sync::{Condvar, Mutex, lock, thread, wait};

pub(crate) struct Baton<L: Copy> {
    state: Mutex<State<L>>,
    /// Notified at each change of holder.
    changed: Condvar,
    /// How many times a callback took it.
    #[cfg(all(test, not(loom)))]
    takes: std::sync::atomic::AtomicU64,
}

struct State<L> {
    /// The thread holding it, or `None` while it is free.
    holder: Option<thread::ThreadId>,
    /// The lends in force, innermost last.
    lends: Vec<Lend<L>>,
    /// Nothing will lend it again ([`Baton::close`]).
    closed: bool,
}

struct Lend<L> {
    lender: thread::ThreadId,
    /// The thread it is lent to, `None` for a vacancy nobody holds.
    lendee: Option<thread::ThreadId>,
    lent: L,
    /// Whether it is a vacancy ([`Baton::vacate`]).
    vacancy: bool,
    /// Whether a thread holds the vacancy by its claim
    /// ([`Baton::await_lend_or_vacancy`]), until it hands it on or frees it.
    claimed: bool,
}

impl<L> Lend<L> {
    /// Whether it is a vacancy nobody holds.
    fn free(&self) -> bool {
        self.vacancy && self.lendee.is_none()
    }
}

/// What [`Baton::await_lend_or_vacancy`] found.
pub(crate) enum Awaited<L> {
    /// The baton is lent to this thread, with this.
    Lent(L),
    /// This thread claimed a free vacancy, vacated with this, and holds the
    /// baton until it hands it on ([`Baton::hand_vacancy`]) or frees it
    /// ([`Baton::free_vacancy`]).
    Vacancy(L),
    /// The baton is closed ([`Baton::close`]).
    Closed,
}

impl<L: Copy> Baton<L> {
    /// A free baton.
    pub(crate) fn new() -> Baton<L> {
        Baton {
            state: Mutex::new(State {
                holder: None,
                lends: Vec::new(),
                closed: false,
            }),
            changed: Condvar::new(),
            #[cfg(all(test, not(loom)))]
            takes: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Counts a callback's take.
    #[cfg(all(test, not(loom)))]
    pub(crate) fn count_take(&self) {
        self.takes
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// How many takes [`Baton::count_take`] counted.
    #[cfg(all(test, not(loom)))]
    pub(crate) fn takes(&self) -> u64 {
        self.takes.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Blocks until the baton is free, then holds it on this thread.
    ///
    /// # Panics
    ///
    /// If this thread holds it already.
    pub(crate) fn acquire(&self) {
        assert!(
            self.take_unless_held(),
            "a thread acquired a baton it holds"
        );
    }

    /// Frees the baton for the next thread waiting in [`Baton::acquire`].
    ///
    /// # Panics
    ///
    /// If this thread does not hold it, or holds it by a lend.
    pub(crate) fn release(&self) {
        let mut state = lock(&self.state);
        let me = thread::current().id();
        assert_eq!(
            state.holder,
            Some(me),
            "a thread released a baton it does not hold"
        );
        assert!(
            state
                .lends
                .last()
                .is_none_or(|lend| lend.lendee != Some(me)),
            "a thread released a baton lent to it"
        );
        state.holder = None;
        drop(state);
        self.changed.notify_all();
    }

    /// Whether this thread holds the baton.
    pub(crate) fn held_here(&self) -> bool {
        lock(&self.state).holder == Some(thread::current().id())
    }

    /// Takes the baton unless this thread holds it, as a callback into the
    /// interpreter does, answering whether it took it.
    pub(crate) fn take_unless_held(&self) -> bool {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        if state.holder == Some(me) {
            return false;
        }
        // A free vacancy is the vacating thread's to take back, or a waiting
        // thread's to claim: a plain take waits until it is reclaimed.
        while state.holder.is_some() || state.lends.last().is_some_and(|lend| lend.vacancy) {
            state = wait(&self.changed, state);
        }
        state.holder = Some(me);
        true
    }

    /// Lends the baton this thread holds to `to` with `lent`, and blocks until
    /// `to` gives it back.
    ///
    /// # Panics
    ///
    /// If this thread does not hold it.
    pub(crate) fn lend(&self, to: thread::ThreadId, lent: L) {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        assert_eq!(
            state.holder,
            Some(me),
            "a thread lent a baton it does not hold"
        );
        let depth = state.lends.len();
        state.lends.push(Lend {
            lender: me,
            lendee: Some(to),
            lent,
            vacancy: false,
            claimed: false,
        });
        state.holder = Some(to);
        self.changed.notify_all();
        while state.lends.len() > depth || state.holder != Some(me) {
            state = wait(&self.changed, state);
        }
    }

    /// Blocks until the baton is lent to this thread, and answers what was
    /// lent with it.
    pub(crate) fn await_lend(&self) -> L {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        loop {
            if state.holder == Some(me)
                && let Some(lend) = state.lends.last()
                && lend.lendee == Some(me)
            {
                return lend.lent;
            }
            state = wait(&self.changed, state);
        }
    }

    /// Wakes every thread in [`Baton::await_lend_or_vacancy`] that waits
    /// unless it is closed, and every later one, with [`Awaited::Closed`]: the
    /// state the baton guards is gone, and no holder is left to lend it.
    pub(crate) fn close(&self) {
        lock(&self.state).closed = true;
        self.changed.notify_all();
    }

    /// What was lent with the baton, where this thread holds it by the
    /// innermost lend.
    pub(crate) fn lent(&self) -> Option<L> {
        let me = thread::current().id();
        let state = lock(&self.state);
        match state.lends.last() {
            Some(lend) if state.holder == Some(me) && lend.lendee == Some(me) => Some(lend.lent),
            _ => None,
        }
    }

    /// Gives the baton lent to this thread back to its lender.
    ///
    /// # Panics
    ///
    /// If the innermost lend is not to this thread.
    pub(crate) fn give_back(&self) {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        assert_eq!(
            state.holder,
            Some(me),
            "a thread gave back a baton it does not hold"
        );
        let lend = state
            .lends
            .pop()
            .expect("a thread gave back a baton nobody lent");
        assert_eq!(
            lend.lendee,
            Some(me),
            "a thread gave back another thread's lend"
        );
        state.holder = Some(lend.lender);
        drop(state);
        self.changed.notify_all();
    }

    /// Vacates the baton this thread holds, with `lent`, and answers the
    /// vacancy's depth, which
    /// [`Baton::take_vacancy`], [`Baton::reclaim_free`] and
    /// [`Baton::await_return`] name it by.
    ///
    /// # Panics
    ///
    /// If this thread does not hold it.
    pub(crate) fn vacate(&self, lent: L) -> usize {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        assert_eq!(
            state.holder,
            Some(me),
            "a thread vacated a baton it does not hold"
        );
        let depth = state.lends.len();
        state.lends.push(Lend {
            lender: me,
            lendee: None,
            lent,
            vacancy: true,
            claimed: false,
        });
        state.holder = None;
        drop(state);
        self.changed.notify_all();
        depth
    }

    /// Takes the vacancy at `depth` back for this thread, which vacated it,
    /// where it is free and innermost, until [`Baton::give_back_or_free`]:
    /// whether it took it. While a thread holds the vacancy by its claim, it
    /// first blocks until that thread hands it on or frees it. It answers
    /// `false` where a thread holds the baton otherwise, by the vacancy
    /// handed to it or by a lend above it.
    pub(crate) fn take_vacancy(&self, depth: usize) -> bool {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        loop {
            if Self::take_vacancy_locked(&mut state, depth, me) {
                return true;
            }
            let claimed = state.lends.len() == depth + 1
                && state.lends.last().is_some_and(|lend| lend.claimed);
            if !claimed {
                return false;
            }
            // A claimer either hands the vacancy on or frees it, whatever this
            // thread does, so the wait never waits for this thread itself.
            state = wait(&self.changed, state);
        }
    }

    fn take_vacancy_locked(state: &mut State<L>, depth: usize, me: thread::ThreadId) -> bool {
        if state.holder.is_some() || state.lends.len() != depth + 1 {
            return false;
        }
        let Some(lend) = state.lends.last_mut() else {
            return false;
        };
        if !lend.free() || lend.lender != me {
            return false;
        }
        lend.lendee = Some(me);
        state.holder = Some(me);
        true
    }

    /// Gives up what this thread holds by its innermost lend: a vacancy it
    /// vacated and took back is freed again, any other lend goes back to its
    /// lender.
    ///
    /// # Panics
    ///
    /// As [`Baton::give_back`].
    pub(crate) fn give_back_or_free(&self) {
        let me = thread::current().id();
        let own = lock(&self.state)
            .lends
            .last()
            .is_some_and(|lend| lend.vacancy && lend.lender == me && lend.lendee == Some(me));
        if own {
            self.free_vacancy();
        } else {
            self.give_back();
        }
    }

    /// Frees the vacancy this thread holds by a claim or a take, for the
    /// next thread to take or claim.
    ///
    /// # Panics
    ///
    /// If this thread holds no vacancy.
    pub(crate) fn free_vacancy(&self) {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        assert_eq!(
            state.holder,
            Some(me),
            "a thread freed a baton it does not hold"
        );
        let lend = state
            .lends
            .last_mut()
            .filter(|lend| lend.vacancy && lend.lendee == Some(me))
            .expect("a thread freed a vacancy it does not hold");
        lend.lendee = None;
        lend.claimed = false;
        state.holder = None;
        drop(state);
        self.changed.notify_all();
    }

    /// Hands the vacancy this thread claimed to `to`, which holds the baton
    /// by it as by a lend, and gives it back to the vacating thread.
    ///
    /// # Panics
    ///
    /// If this thread holds no vacancy.
    pub(crate) fn hand_vacancy(&self, to: thread::ThreadId) {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        assert_eq!(
            state.holder,
            Some(me),
            "a thread handed a baton it does not hold"
        );
        let lend = state
            .lends
            .last_mut()
            .filter(|lend| lend.vacancy && lend.lendee == Some(me))
            .expect("a thread handed a vacancy it does not hold");
        lend.lendee = Some(to);
        lend.claimed = false;
        state.holder = Some(to);
        drop(state);
        self.changed.notify_all();
    }

    /// Blocks until the baton is lent to this thread, or, where `claim`, a
    /// free vacancy is innermost, which this thread then claims, or, where
    /// `unless_closed`, the baton is closed ([`Baton::close`]) with neither,
    /// so that what this thread waits for will never come.
    pub(crate) fn await_lend_or_vacancy(&self, claim: bool, unless_closed: bool) -> Awaited<L> {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        loop {
            if state.holder == Some(me)
                && let Some(lend) = state.lends.last()
                && lend.lendee == Some(me)
            {
                return Awaited::Lent(lend.lent);
            }
            if claim
                && state.holder.is_none()
                && let Some(lend) = state.lends.last_mut()
                && lend.free()
            {
                lend.lendee = Some(me);
                lend.claimed = true;
                let lent = lend.lent;
                state.holder = Some(me);
                return Awaited::Vacancy(lent);
            }
            if unless_closed && state.closed {
                return Awaited::Closed;
            }
            state = wait(&self.changed, state);
        }
    }

    /// Takes back the vacancy at `depth`, which this thread vacated, where
    /// it is free: whether it did. This thread then holds the baton as
    /// before [`Baton::vacate`].
    pub(crate) fn reclaim_free(&self, depth: usize) -> bool {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        Self::reclaim_locked(&mut state, depth, me)
    }

    /// Blocks until the vacancy at `depth`, which this thread vacated, is
    /// given back by the thread it was handed to, or is free and taken back.
    pub(crate) fn await_return(&self, depth: usize) {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        loop {
            if state.holder == Some(me) && state.lends.len() == depth {
                return;
            }
            if Self::reclaim_locked(&mut state, depth, me) {
                return;
            }
            state = wait(&self.changed, state);
        }
    }

    fn reclaim_locked(state: &mut State<L>, depth: usize, me: thread::ThreadId) -> bool {
        let free = state.holder.is_none()
            && state.lends.len() == depth + 1
            && state
                .lends
                .last()
                .is_some_and(|lend| lend.free() && lend.lender == me);
        if free {
            state.lends.pop();
            state.holder = Some(me);
        }
        free
    }

    /// What was lent with the baton, where this thread holds it by a lend
    /// above the vacancy at `depth`.
    pub(crate) fn lent_above(&self, depth: usize) -> Option<L> {
        let me = thread::current().id();
        let state = lock(&self.state);
        match state.lends.last() {
            Some(lend)
                if state.lends.len() > depth + 1
                    && state.holder == Some(me)
                    && lend.lendee == Some(me) =>
            {
                Some(lend.lent)
            }
            _ => None,
        }
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::{Awaited, Baton};

    /// A thread claims the vacancy and frees it again, as one that finds no
    /// pool thread to hand it to does: the vacating thread, taking it back
    /// meanwhile, waits out the claim and then takes it.
    #[test]
    fn a_vacating_thread_takes_back_a_vacancy_a_claim_freed() {
        let baton = Baton::<u8>::new();
        baton.acquire();
        let depth = baton.vacate(1);
        let (claimed, seen) = std::sync::mpsc::channel();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                assert!(matches!(
                    baton.await_lend_or_vacancy(true, false),
                    Awaited::Vacancy(1)
                ));
                claimed.send(()).expect("the test waits for the claim");
                std::thread::sleep(std::time::Duration::from_millis(50));
                baton.free_vacancy();
            });
            seen.recv().expect("the claimer claims");
            assert!(baton.take_vacancy(depth), "the vacancy was not taken back");
            assert!(baton.held_here());
            baton.give_back_or_free();
        });
        assert!(baton.reclaim_free(depth));
        baton.release();
    }

    /// A thread claims the vacancy and hands it to a third: the vacating
    /// thread does not take it back, and recalls it from that thread instead.
    #[test]
    fn a_vacating_thread_does_not_take_back_a_vacancy_handed_on() {
        let baton = Baton::<u8>::new();
        baton.acquire();
        let depth = baton.vacate(1);
        let (handed, seen) = std::sync::mpsc::channel();
        std::thread::scope(|scope| {
            let server = scope.spawn(|| {
                assert_eq!(baton.await_lend(), 1);
                handed.send(()).expect("the test waits for the hand");
            });
            let to = server.thread().id();
            let claimer = &baton;
            scope.spawn(move || {
                assert!(matches!(
                    claimer.await_lend_or_vacancy(true, false),
                    Awaited::Vacancy(1)
                ));
                claimer.hand_vacancy(to);
            });
            seen.recv().expect("the server is handed the vacancy");
            assert!(
                !baton.take_vacancy(depth),
                "a served vacancy was taken back"
            );
        });
    }
}
