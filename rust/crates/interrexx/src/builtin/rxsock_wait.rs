//! Where an `rxsock` call's wait for its sockets runs while its activity is
//! parked: the platform-neutral face of it.
//!
//! A wait is a list of descriptors with what each is waited for, an
//! optional deadline, and a way to post its end to the interpreter that
//! owns it ([`OnEnd`]). [`register`] hands it to the process's one reactor
//! thread, which runs it with every other interpreter's waits; the answer is
//! a [`Wait`] whose [`Wait::cancel`] (or drop) ends it early. Each
//! registration's end is posted **exactly once**: when one of its
//! descriptors answers it, when its deadline passes, or when it is
//! cancelled.
//!
//! Behind it:
//!
//! - Linux: an `epoll` reactor ([`epoll`]), started on the first
//!   registration.
//! - Elsewhere: no reactor yet ([`reactor_available`] answers `false`), and
//!   the caller runs the wait on a pool thread of its own, polling the
//!   sockets with a [`Canceller`]'s descriptor that ends it early. `kqueue`
//!   and Windows reactors go in beside `epoll`.

use std::time::Instant;

use rustix::event::PollFlags;

/// How a wait's end reaches the interpreter that registered it. Called once,
/// on the reactor's thread, so it must not block.
pub(crate) type OnEnd = Box<dyn FnOnce() + Send>;

/// Whether this platform has a reactor that [`register`] can use.
pub(crate) fn reactor_available() -> bool {
    imp::available()
}

/// Starts the reactor where this platform has one, when the `rxsock`
/// library is loaded rather than at the first wait: its descriptors are
/// then open before a program's first look at its own, and a count of open
/// descriptors taken around a socket's use does not see them appear.
pub(crate) fn prepare() {
    imp::available();
}

/// Hands a wait for `polls` until `deadline` to the reactor, which calls
/// `on_end` once it is over. Answers `on_end` back where there is no reactor
/// to take it, or it cannot be started, so the caller can wait another way.
pub(crate) fn register(
    polls: &[(i32, PollFlags)],
    deadline: Option<Instant>,
    on_end: OnEnd,
) -> Result<Wait, OnEnd> {
    imp::register(polls, deadline, on_end).map(Wait)
}

/// A registered wait. Dropping it cancels it.
pub(crate) struct Wait(imp::Wait);

impl Wait {
    /// Ends the wait now, if it has not ended: its end is posted once the
    /// reactor has taken the cancellation.
    pub(crate) fn cancel(&self) {
        self.0.cancel();
    }
}

impl Drop for Wait {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

/// A descriptor a waiting thread polls beside its sockets, and the means to
/// make it readable: an `eventfd` where there is one, and a pipe elsewhere.
pub(crate) struct Canceller {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    event: std::os::fd::OwnedFd,
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    pipe: (std::os::fd::OwnedFd, std::os::fd::OwnedFd),
}

impl Canceller {
    /// A new canceller, or `None` where no descriptor can be had.
    pub(crate) fn new() -> Option<Canceller> {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            let event = rustix::event::eventfd(
                0,
                rustix::event::EventfdFlags::CLOEXEC | rustix::event::EventfdFlags::NONBLOCK,
            )
            .ok()?;
            Some(Canceller { event })
        }
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        {
            let (read, write) = rustix::pipe::pipe_with(rustix::pipe::PipeFlags::CLOEXEC).ok()?;
            rustix::fs::fcntl_setfl(&write, rustix::fs::OFlags::NONBLOCK).ok()?;
            Some(Canceller {
                pipe: (read, write),
            })
        }
    }

    /// The descriptor to poll for readability.
    pub(crate) fn polled(&self) -> std::os::fd::BorrowedFd<'_> {
        use std::os::fd::AsFd as _;
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            self.event.as_fd()
        }
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        {
            self.pipe.0.as_fd()
        }
    }

    /// Makes [`Canceller::polled`] readable. A full counter or pipe is
    /// already readable, so a refused write changes nothing.
    pub(crate) fn signal(&self) {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        let _ = rustix::io::write(&self.event, &1u64.to_ne_bytes());
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        let _ = rustix::io::write(&self.pipe.1, &[1u8]);
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
use epoll as imp;
#[cfg(not(any(target_os = "linux", target_os = "android")))]
use none as imp;

/// No reactor: every registration is answered back.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
mod none {
    use super::{OnEnd, PollFlags};
    use std::time::Instant;

    pub(super) enum Wait {}

    impl Wait {
        pub(super) fn cancel(&self) {
            match *self {}
        }
    }

    pub(super) fn available() -> bool {
        false
    }

    pub(super) fn register(
        _polls: &[(i32, PollFlags)],
        _deadline: Option<Instant>,
        on_end: OnEnd,
    ) -> Result<Wait, OnEnd> {
        Err(on_end)
    }
}

/// The `epoll` reactor: one thread per process, started on the first
/// registration and never ended, serving every interpreter's waits.
///
/// **One `epoll` registration per descriptor**, with the union of what its
/// waiters wait for, level-triggered: an `epoll` instance takes a
/// descriptor once (`EEXIST` on a second add), and two activities can wait
/// on one socket. The interest is modified as waiters come and go and the
/// descriptor deleted when none is left. A descriptor closed while
/// registered leaves the interest list silently and its number can be
/// reused, so a modify's `ENOENT` adds it again and an add's `EEXIST`
/// modifies it.
///
/// An event ends each waiter whose interest it answers, by
/// [`super::rxsock::answers`]'s rule. `epoll` reports hang-up and error
/// whatever was asked, and those answer no wait for priority data alone, so
/// a descriptor whose event answers none of its waiters would end every
/// `epoll_wait` at once. Such a descriptor is **muted** -- deleted from the
/// interest list -- and added again [`MUTED_RECHECK`] later, since its
/// socket may yet be connected.
///
/// Registrations and cancellations reach the thread through a command
/// queue and an `eventfd` that wakes it; its `epoll_wait` timeout is the
/// earliest deadline or muted descriptor's recheck.
mod epoll {
    use super::{OnEnd, PollFlags};
    use crate::builtin::rxsock::answers;
    use crate::builtin::rxsock_sys::with_socket;
    use interrexx_rxapi::reserved::{Reserved, reserve};
    use rustc_hash::FxHashMap;
    use rustix::event::epoll::{self, EventData, EventFlags};
    use std::collections::BTreeSet;
    use std::os::fd::OwnedFd;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{Arc, Mutex, OnceLock};
    use std::time::{Duration, Instant};

    /// How long a muted descriptor stays out of the interest list.
    const MUTED_RECHECK: Duration = Duration::from_millis(50);

    /// The data the wake descriptor is registered under: no descriptor
    /// number reaches it.
    const WAKE: u64 = u64::MAX;

    /// The most events one `epoll_wait` takes.
    const EVENTS: usize = 256;

    /// The process's reactor, or `None` where it could not be started.
    static REACTOR: OnceLock<Option<Reactor>> = OnceLock::new();

    /// What the registering side holds of the reactor.
    struct Reactor {
        commands: Mutex<Vec<Command>>,
        wake: Arc<Reserved<OwnedFd>>,
        next_id: AtomicU64,
        /// Set when the thread has stopped on an error it cannot recover
        /// from, after it has ended every wait it held.
        stopped: AtomicBool,
    }

    /// The command queue, whatever a panicking holder left in it: a
    /// command is whole once pushed.
    fn lock(commands: &Mutex<Vec<Command>>) -> std::sync::MutexGuard<'_, Vec<Command>> {
        commands
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    enum Command {
        Register {
            id: u64,
            polls: Vec<(i32, PollFlags)>,
            deadline: Option<Instant>,
            on_end: OnEnd,
        },
        Cancel(u64),
    }

    pub(super) struct Wait {
        id: u64,
        reactor: &'static Reactor,
    }

    impl Wait {
        pub(super) fn cancel(&self) {
            self.reactor.send(Command::Cancel(self.id));
        }
    }

    impl Reactor {
        fn send(&self, command: Command) {
            lock(&self.commands).push(command);
            let _ = rustix::io::write(&**self.wake, &1u64.to_ne_bytes());
        }
    }

    pub(super) fn available() -> bool {
        reactor().is_some()
    }

    pub(super) fn register(
        polls: &[(i32, PollFlags)],
        deadline: Option<Instant>,
        on_end: OnEnd,
    ) -> Result<Wait, OnEnd> {
        let Some(reactor) = reactor() else {
            return Err(on_end);
        };
        if reactor.stopped.load(Ordering::Acquire) {
            return Err(on_end);
        }
        let id = reactor.next_id.fetch_add(1, Ordering::Relaxed);
        reactor.send(Command::Register {
            id,
            polls: polls.to_vec(),
            deadline,
            on_end,
        });
        Ok(Wait { id, reactor })
    }

    /// The reactor, started on first use.
    fn reactor() -> Option<&'static Reactor> {
        REACTOR.get_or_init(start).as_ref()
    }

    fn start() -> Option<Reactor> {
        // Both moved above the numbers a program's own descriptors take,
        // as the interpreter's other descriptors are, so that a socket's
        // number does not depend on whether the reactor runs.
        let epoll = reserve(epoll::create(epoll::CreateFlags::CLOEXEC).ok()?);
        let wake = Arc::new(reserve(
            rustix::event::eventfd(
                0,
                rustix::event::EventfdFlags::CLOEXEC | rustix::event::EventfdFlags::NONBLOCK,
            )
            .ok()?,
        ));
        epoll::add(&*epoll, &**wake, EventData::new_u64(WAKE), EventFlags::IN).ok()?;
        let drained = Arc::clone(&wake);
        std::thread::Builder::new()
            .name("rxsock reactor".into())
            // Its frames are shallow; the default would reserve megabytes
            // of address space for nothing.
            .stack_size(64 * 1024)
            .spawn(move || run(&epoll, &drained))
            .ok()?;
        Some(Reactor {
            commands: Mutex::new(Vec::new()),
            wake,
            next_id: AtomicU64::new(0),
            stopped: AtomicBool::new(false),
        })
    }

    /// One registered wait, as the thread holds it.
    struct Waiter {
        /// Each descriptor once, with everything waited for on it.
        fds: Vec<(i32, PollFlags)>,
        deadline: Option<Instant>,
        on_end: OnEnd,
    }

    /// One descriptor, as the thread holds it.
    #[derive(Default)]
    struct Watched {
        waiters: Vec<u64>,
        /// The interest it is registered with, or `None` while it is not in
        /// the interest list.
        registered: Option<EventFlags>,
        /// When a muted descriptor is added again.
        muted_until: Option<Instant>,
        /// Whether its registration is to be made again even where the
        /// interest has not changed.
        recheck: bool,
    }

    #[derive(Default)]
    struct State {
        waiters: FxHashMap<u64, Waiter>,
        watched: FxHashMap<i32, Watched>,
        deadlines: BTreeSet<(Instant, u64)>,
        mutes: BTreeSet<(Instant, i32)>,
    }

    /// The thread: takes commands, waits, ends what is over.
    fn run(epoll: &OwnedFd, wake: &OwnedFd) {
        let mut state = State::default();
        let mut events: Vec<epoll::Event> = Vec::with_capacity(EVENTS);
        loop {
            take_commands(epoll, &mut state);
            let now = crate::sync::now();
            state.expire(epoll, now);
            let next = [
                state.deadlines.first().map(|(at, _)| *at),
                state.mutes.first().map(|(at, _)| *at),
            ]
            .into_iter()
            .flatten()
            .min();
            let timeout = next.map(|at| {
                let left = at.saturating_duration_since(now);
                rustix::time::Timespec {
                    tv_sec: i64::try_from(left.as_secs()).unwrap_or(i64::MAX),
                    tv_nsec: i64::from(left.subsec_nanos()),
                }
            });
            events.clear();
            match epoll::wait(
                epoll,
                rustix::buffer::spare_capacity(&mut events),
                timeout.as_ref(),
            ) {
                Ok(_) => {}
                Err(rustix::io::Errno::INTR) => continue,
                Err(_) => {
                    stop(&mut state);
                    return;
                }
            }
            for event in &events {
                let data = event.data.u64();
                if data == WAKE {
                    let mut count = [0u8; 8];
                    let _ = rustix::io::read(wake, &mut count);
                    continue;
                }
                let Ok(fd) = i32::try_from(data) else {
                    continue;
                };
                state.event(epoll, fd, event.flags);
            }
        }
    }

    /// Ends every wait after an error the thread cannot go on from, and
    /// refuses registrations from then on.
    fn stop(state: &mut State) {
        if let Some(Some(reactor)) = REACTOR.get() {
            reactor.stopped.store(true, Ordering::Release);
            for command in std::mem::take(&mut *lock(&reactor.commands)) {
                if let Command::Register { on_end, .. } = command {
                    on_end();
                }
            }
        }
        for (_, waiter) in state.waiters.drain() {
            (waiter.on_end)();
        }
    }

    fn take_commands(epoll: &OwnedFd, state: &mut State) {
        let Some(Some(reactor)) = REACTOR.get() else {
            return;
        };
        let commands = std::mem::take(&mut *lock(&reactor.commands));
        for command in commands {
            match command {
                Command::Register {
                    id,
                    polls,
                    deadline,
                    on_end,
                } => state.add(epoll, id, &polls, deadline, on_end),
                Command::Cancel(id) => state.end(epoll, id),
            }
        }
    }

    /// What `epoll` is asked to report for a wait for `flags`. Hang-up and
    /// error it reports unasked.
    fn interest(flags: PollFlags) -> EventFlags {
        let mut interest = EventFlags::empty();
        if flags.contains(PollFlags::IN) {
            interest |= EventFlags::IN;
        }
        if flags.contains(PollFlags::OUT) {
            interest |= EventFlags::OUT;
        }
        if flags.contains(PollFlags::PRI) {
            interest |= EventFlags::PRI;
        }
        interest
    }

    /// What `epoll` reported, as `poll` would have.
    fn reported(flags: EventFlags) -> PollFlags {
        let mut revents = PollFlags::empty();
        for (event, poll) in [
            (EventFlags::IN, PollFlags::IN),
            (EventFlags::OUT, PollFlags::OUT),
            (EventFlags::PRI, PollFlags::PRI),
            (EventFlags::ERR, PollFlags::ERR),
            (EventFlags::HUP, PollFlags::HUP),
        ] {
            if flags.contains(event) {
                revents |= poll;
            }
        }
        revents
    }

    impl State {
        fn add(
            &mut self,
            epoll: &OwnedFd,
            id: u64,
            polls: &[(i32, PollFlags)],
            deadline: Option<Instant>,
            on_end: OnEnd,
        ) {
            let mut fds: Vec<(i32, PollFlags)> = Vec::with_capacity(polls.len());
            for &(fd, flags) in polls {
                match fds.iter_mut().find(|(seen, _)| *seen == fd) {
                    Some((_, union)) => *union |= flags,
                    None => fds.push((fd, flags)),
                }
            }
            if let Some(deadline) = deadline {
                self.deadlines.insert((deadline, id));
            }
            self.waiters.insert(
                id,
                Waiter {
                    fds: fds.clone(),
                    deadline,
                    on_end,
                },
            );
            for (fd, _) in fds {
                let watched = self.watched.entry(fd).or_default();
                watched.waiters.push(id);
                // The number may name a new socket since it was registered:
                // a closed one leaves the interest list silently. Telling
                // `epoll` again costs a call per wait and finds out.
                watched.recheck = watched.registered.is_some();
                // A descriptor that cannot be registered (closed, or not
                // one `epoll` takes) ends its waiters, as a `poll` of it
                // would have reported it at once.
                if !self.update(epoll, fd) {
                    self.end_waiters_of(epoll, fd);
                }
                if !self.waiters.contains_key(&id) {
                    break;
                }
            }
        }

        /// Ends wait `id`, if it has not ended, and posts its end.
        fn end(&mut self, epoll: &OwnedFd, id: u64) {
            let Some(waiter) = self.waiters.remove(&id) else {
                return;
            };
            if let Some(deadline) = waiter.deadline {
                self.deadlines.remove(&(deadline, id));
            }
            for (fd, _) in &waiter.fds {
                if let Some(watched) = self.watched.get_mut(fd) {
                    watched.waiters.retain(|&other| other != id);
                }
                self.update(epoll, *fd);
            }
            (waiter.on_end)();
        }

        fn end_waiters_of(&mut self, epoll: &OwnedFd, fd: i32) {
            let ids = self
                .watched
                .get(&fd)
                .map(|watched| watched.waiters.clone())
                .unwrap_or_default();
            for id in ids {
                self.end(epoll, id);
            }
        }

        /// Brings `fd`'s registration in line with its waiters, and answers
        /// whether it could be.
        fn update(&mut self, epoll: &OwnedFd, fd: i32) -> bool {
            let Some(watched) = self.watched.get_mut(&fd) else {
                return true;
            };
            if watched.waiters.is_empty() {
                if watched.registered.is_some() {
                    // A closed descriptor has already left the list.
                    let _ = with_socket(fd, |socket| {
                        epoll::delete(epoll, socket).map_err(|error| error.raw_os_error())
                    });
                }
                if let Some(until) = watched.muted_until {
                    self.mutes.remove(&(until, fd));
                }
                self.watched.remove(&fd);
                return true;
            }
            if watched.muted_until.is_some() {
                return true;
            }
            let wanted = watched
                .waiters
                .iter()
                .filter_map(|id| self.waiters.get(id))
                .flat_map(|waiter| waiter.fds.iter())
                .filter(|(other, _)| *other == fd)
                .fold(EventFlags::empty(), |union, (_, flags)| {
                    union | interest(*flags)
                });
            if watched.registered == Some(wanted) && !std::mem::take(&mut watched.recheck) {
                return true;
            }
            let data = EventData::new_u64(u64::try_from(fd).unwrap_or(WAKE));
            let registered = watched.registered;
            let done = with_socket(fd, |socket| {
                let first = match registered {
                    Some(_) => epoll::modify(epoll, socket, data, wanted),
                    None => epoll::add(epoll, socket, data, wanted),
                };
                match first {
                    Err(rustix::io::Errno::NOENT) => epoll::add(epoll, socket, data, wanted),
                    Err(rustix::io::Errno::EXIST) => epoll::modify(epoll, socket, data, wanted),
                    other => other,
                }
                .map_err(|error| error.raw_os_error())
            });
            let watched = self.watched.get_mut(&fd).expect("looked up above");
            match done {
                Ok(()) => {
                    watched.registered = Some(wanted);
                    watched.recheck = false;
                    true
                }
                Err(_) => {
                    watched.registered = None;
                    false
                }
            }
        }

        /// `epoll` reported `flags` for `fd`: ends each waiter they answer,
        /// and mutes `fd` where they answer none.
        fn event(&mut self, epoll: &OwnedFd, fd: i32, flags: EventFlags) {
            let revents = reported(flags);
            let Some(watched) = self.watched.get(&fd) else {
                return;
            };
            let answered: Vec<u64> = watched
                .waiters
                .iter()
                .copied()
                .filter(|id| {
                    self.waiters.get(id).is_some_and(|waiter| {
                        waiter
                            .fds
                            .iter()
                            .any(|(other, wanted)| *other == fd && answers(*wanted, revents))
                    })
                })
                .collect();
            if answered.is_empty() {
                if !revents.is_empty() {
                    self.mute(epoll, fd);
                }
                return;
            }
            for id in answered {
                self.end(epoll, id);
            }
        }

        fn mute(&mut self, epoll: &OwnedFd, fd: i32) {
            let Some(watched) = self.watched.get_mut(&fd) else {
                return;
            };
            if watched.registered.take().is_some() {
                let _ = with_socket(fd, |socket| {
                    epoll::delete(epoll, socket).map_err(|error| error.raw_os_error())
                });
            }
            let until = crate::sync::now() + MUTED_RECHECK;
            watched.muted_until = Some(until);
            self.mutes.insert((until, fd));
        }

        /// Ends the waits whose deadline has passed and adds back the muted
        /// descriptors whose recheck is due.
        fn expire(&mut self, epoll: &OwnedFd, now: Instant) {
            while let Some(&(at, id)) = self.deadlines.first() {
                if at > now {
                    break;
                }
                self.deadlines.remove(&(at, id));
                self.end(epoll, id);
            }
            while let Some(&(at, fd)) = self.mutes.first() {
                if at > now {
                    break;
                }
                self.mutes.remove(&(at, fd));
                if let Some(watched) = self.watched.get_mut(&fd) {
                    watched.muted_until = None;
                }
                if !self.update(epoll, fd) {
                    self.end_waiters_of(epoll, fd);
                }
            }
        }
    }
}
