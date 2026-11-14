//! The interpreter's state on another OS thread: a pool thread reaches it only through what the baton's
//! holder lends it, and only while it holds the baton.

#![allow(unsafe_code)]

use std::ptr::NonNull;

use crate::Interp;
use crate::dispatch::library::OffBaton;

/// The baton every interpreter's state is reached under.
pub(crate) type InterpBaton = crate::baton::Baton<Island>;

/// A value of the interpreter's that moves to another OS thread.
pub(crate) struct Islanded<T>(T);

/// What an [`Islanded`] may carry: each payload is listed here, and its
/// argument in the `Send` grant below.
pub(crate) trait IslandPayload {}

impl IslandPayload for NonNull<Interp> {}

impl IslandPayload for (Box<OffBaton>, interrexx_api::ffi::ThreadContext) {}

// SAFETY: D-U2. The island is reachable only through one root pointer and the
// baton, and a thread does no refcount operation and no interior access on an
// island value (derives a borrow, clones or drops an `Rc`, reads or writes a
// `Cell`) off the baton. A value made here is made on the baton, moves without
// being touched, and is taken out only by `Islanded::take`, which requires the
// baton; so no two threads ever touch it at once, which is the property its
// `Rc` and `Cell` interior needs. A call run where it is made keeps a
// `ThreadContext` clone across the release, made before the release and
// dropped after the reacquire; a pool thread's clone moves here and is dropped
// under a lend. `Sync` is not granted. Per payload: the root pointer
// `NonNull<Interp>` is dereferenced only by the thread it is lent to, while
// its lender waits: by `Lent::interp`, and by `HostRef`'s `Deref` and
// `DerefMut` (`interrexx-api/src/ffi.rs`) for a callback, which receives it through
// `Island::host`. A native call's `(Box<OffBaton>, ThreadContext)` is taken
// out by `PooledCall::run` under a lend, and the context, an `Rc`, is dropped
// under the recall's lend. The box holds no `Rc` or `Cell` (its parts are
// `ObjRef` words, byte buffers, stub pointers and an `Arc`): it goes back on
// the activity's record under that lend, or, for a call abandoned while it
// ran, drops after the lend is given back.
unsafe impl<T: IslandPayload> Send for Islanded<T> {}

impl<T> Islanded<T> {
    /// `value`, made by the baton's holder.
    pub(crate) fn new(value: T, baton: &InterpBaton) -> Islanded<T> {
        assert!(baton.held_here(), "an island value moved off the baton");
        Islanded(value)
    }

    /// The value, for the baton's holder.
    pub(crate) fn take(self, baton: &InterpBaton) -> T {
        assert!(baton.held_here(), "an island value taken off the baton");
        self.0
    }
}

/// The interpreter's root pointer, which the baton's holder lends with it.
pub(crate) type Island = Islanded<NonNull<Interp>>;

impl Clone for Island {
    fn clone(&self) -> Island {
        *self
    }
}

impl Copy for Island {}

impl Island {
    /// The root pointer of `interp`, which its holder lends while it waits.
    pub(crate) fn of(interp: &mut Interp) -> Island {
        Islanded(NonNull::from(interp))
    }

    /// The interpreter as the API reaches it.
    pub(crate) fn host(self) -> NonNull<dyn interrexx_api::values::Host> {
        self.0
    }
}

/// The baton lent to this thread, given back when this drops.
pub(crate) struct Lent<'b> {
    baton: &'b InterpBaton,
    island: Island,
}

impl<'b> Lent<'b> {
    /// Waits until the baton is lent to this thread.
    pub(crate) fn wait(baton: &'b InterpBaton) -> Lent<'b> {
        Lent {
            island: baton.await_lend(),
            baton,
        }
    }

    /// The interpreter, for as long as this lend.
    pub(crate) fn interp(&mut self) -> &mut Interp {
        // SAFETY: the lender derived the pointer from its own `&mut Interp`
        // and waits in `Baton::lend`, touching nothing, until this thread
        // gives the baton back, which only this value's drop does. The answer
        // borrows `self` uniquely, so it ends before that drop and no second
        // borrow is live beside it.
        unsafe { self.island.0.as_mut() }
    }
}

impl Drop for Lent<'_> {
    /// Gives the baton back, except as a panic unwinds: the pool thread's
    /// job posts the panic first, so the lender drains it before it runs a
    /// clause.
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.baton.give_back();
        }
    }
}

/// Waits until the baton is lent to this thread, which has posted a recall
/// for it. Where the baton is vacated by a native call running off it
/// ([`crate::baton::Baton::vacate`]) and nobody serves that recall, this
/// thread claims the vacancy and hands it to a pool thread it starts to run
/// the interpreter until the call returns, which serves the recall; where
/// the pool gives none, the vacancy is freed again and this thread waits for
/// the vacating thread to serve it. `None` where `unless_closed` and the
/// interpreter is gone first, so nothing will lend the baton ([`Closing`]).
pub(crate) fn wait_served(baton: &InterpBaton, unless_closed: bool) -> Option<Lent<'_>> {
    let mut claim = true;
    loop {
        match baton.await_lend_or_vacancy(claim, unless_closed) {
            crate::baton::Awaited::Lent(island) => return Some(Lent { baton, island }),
            crate::baton::Awaited::Closed => return None,
            crate::baton::Awaited::Vacancy(island) => {
                // SAFETY: the vacating thread derived the pointer from its
                // own `&mut Interp` ([`Island::of`]) before it vacated, and
                // touches the interpreter again only once it has reclaimed
                // the vacancy, which waits while this thread holds it by its
                // claim; no other thread holds the baton meanwhile. The borrow
                // ends before the vacancy is handed on or freed below.
                let started = unsafe { island.0.as_ref() }.start_vacancy_server();
                match started {
                    Some(server) => baton.hand_vacancy(server),
                    None => {
                        baton.free_vacancy();
                        claim = false;
                    }
                }
            }
        }
    }
}

/// Closes the baton it holds when it drops, as the interpreter holding it
/// does: a thread still waiting for a lend, a callback whose recall nothing
/// served, wakes and is refused.
pub(crate) struct Closing(pub(crate) crate::sync::Arc<InterpBaton>);

impl Drop for Closing {
    fn drop(&mut self) {
        self.0.close();
    }
}

// The interpreter moves to another thread only as an `Islanded` payload, and a
// frame not at all (static_assertions' `assert_not_impl_any`).
trait AmbiguousIfSend<A> {
    fn some_item() {}
}
impl<T: ?Sized> AmbiguousIfSend<()> for T {}
impl<T: ?Sized + Send> AmbiguousIfSend<u8> for T {}
const _: fn() = || {
    let _ = <Interp as AmbiguousIfSend<_>>::some_item;
    let _ = <interrexx_core::RegFrame<'static> as AmbiguousIfSend<_>>::some_item;
};

#[cfg(test)]
impl IslandPayload for i32 {}

#[cfg(test)]
mod tests {
    use super::{InterpBaton, Islanded};

    /// An island value is made only by the baton's holder, in every build.
    #[test]
    #[should_panic(expected = "an island value moved off the baton")]
    fn an_island_value_is_made_only_on_the_baton() {
        let baton = InterpBaton::new();
        let _ = Islanded::new(1, &baton);
    }

    /// An island value is taken out only by the baton's holder, in every
    /// build.
    #[test]
    #[should_panic(expected = "an island value taken off the baton")]
    fn an_island_value_is_taken_only_on_the_baton() {
        let baton = InterpBaton::new();
        baton.acquire();
        let value = Islanded::new(1, &baton);
        std::thread::scope(|scope| {
            scope
                .spawn(|| value.take(&baton))
                .join()
                .unwrap_or_else(|payload| std::panic::resume_unwind(payload))
        });
    }
}
