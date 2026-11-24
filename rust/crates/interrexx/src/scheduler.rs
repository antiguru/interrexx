//! The scheduler seam and the activity table: the running activity sits inline
//! on `Interp`, every other one boxed here, and a switch swaps the two where a
//! driver has exited or, for a pinned wait, inside the Rust frames that pin the
//! outgoing activity. A pinned waiter that becomes ready while a loop nested
//! above its own runs waits for that loop to return, so a buried sleeper can
//! wake after a later deadline: a recorded divergence, counted as a late wake.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::time::Instant;

use interrexx_core::{ActivityRoots, ObjRef};
use rustc_hash::FxHashMap;

use crate::activity::Activity;
use crate::dispatch::{Caller, Then};
use crate::error::{Failure, Raised};
use crate::ir::{DriveStart, Driven};
use crate::run::{Ended, Flow, Started};
use crate::timer::SLICE;
use crate::{Interp, Loud, PendingTrap, SwitchMode};

/// What an instruction run by [`crate::ir::Op::Exec`] answers the driver.
pub(crate) enum ExecOutcome {
    /// It ran, and this is its `Flow`.
    Done(Flow),
    /// The instruction parks the activity for this reason, and runs again
    /// once it wakes.
    Park(ParkReason),
    /// A `REPLY` with this value: the rest of its body goes to a new
    /// activity at the next clause boundary.
    Split(Option<ObjRef>),
    /// A `FORWARD` whose send the driver begins. Boxed so that the outcome
    /// every [`crate::ir::Op::Exec`] instruction answers keeps its size.
    Forward(Box<crate::run::ForwardSend>),
}

// Every `Op::Exec` instruction (`CALL`, `NUMERIC`, `GUARD` and the rest)
// answers this, so a variant that widened it would cost all of them.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Result<ExecOutcome, Failure>>() == 40);

/// An activity's handle: its offset in the interpreter's activity table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ActivityId(u32);

impl ActivityId {
    /// The handle's offset in the activity table.
    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

#[cfg(test)]
impl ActivityId {
    pub(crate) fn test(index: u32) -> ActivityId {
        ActivityId(index)
    }
}

/// A timer the natives behind `.Alarm` and `.Ticker` wait on, numbered per
/// interpreter from 1; its `EVENTSEMHANDLE` pointer holds the number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TimerId(u32);

impl TimerId {
    pub(crate) fn from_address(address: usize) -> Option<TimerId> {
        u32::try_from(address).ok().map(TimerId)
    }

    pub(crate) fn address(self) -> usize {
        self.0 as usize
    }
}

/// A timer's state: an event semaphore, posted until reset, and the activities
/// waiting on it.
#[derive(Default)]
struct Timer {
    posted: bool,
    /// Whether a post woke a waiter whose object was cancelled.
    cancelled: bool,
    /// Whether the timer ends with a wait on it, as an alarm's does.
    ends_with_wait: bool,
    /// The wait about to park: its receiver, scope and the end of its whole
    /// days.
    pending: Option<(ObjRef, ObjRef, Instant)>,
    waiters: Vec<TimerWaiter>,
}

/// One activity waiting on a timer.
struct TimerWaiter {
    activity: ActivityId,
    /// Its sleeper's park order.
    order: u64,
    /// The waiting call's receiver and method scope, whose `CANCELED` a post
    /// reads.
    receiver: ObjRef,
    scope: ObjRef,
    days_end: Instant,
}

/// A message object's identity among the activities waiting on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MessageId(u32);

/// Why an activity parks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParkReason {
    /// A guard lock, until a release makes this activity its owner.
    Guard(crate::guards::GuardKey),
    /// A `GUARD WHEN`, until a store or `DROP` of a variable it watches.
    GuardWhen,
    /// `~result`, until the message's send completes.
    MessageResult(MessageId),
    /// `~wait`, until the message's send completes.
    MessageWait(MessageId),
    /// `SysSleep`, until `deadline`.
    Sleep { deadline: Instant },
    /// A timer's wait, until `deadline` or a post to `cancel`.
    Timer { deadline: Instant, cancel: TimerId },
    /// A semaphore's wait, until a post or release hands it over or a timed
    /// wait's end.
    Semaphore(crate::semaphores::SemaphoreWait),
    /// A native call prepared on the baton, until its completion is drained.
    Native,
    /// A blocking operation run off the baton, until it has ended.
    Block,
    /// An `rxsock` call's wait for its sockets, run off the baton by the
    /// socket reactor or a pool thread ([`crate::builtin::rxsock`]), until
    /// its end has been posted.
    #[cfg(unix)]
    Socket,
}

impl ParkReason {
    /// What a wait for this reason waits for, as a refusal names it.
    fn waits_for(self) -> &'static str {
        match self {
            ParkReason::Guard(_) => "a guard",
            ParkReason::GuardWhen => "a guard's WHEN",
            ParkReason::MessageResult(_) | ParkReason::MessageWait(_) => "a message's completion",
            ParkReason::Sleep { .. } => "a sleep's end",
            ParkReason::Timer { .. } => "a timer's end",
            ParkReason::Semaphore(_) => "a semaphore",
            ParkReason::Native => "a native call's return",
            ParkReason::Block => "a command's end",
            #[cfg(unix)]
            ParkReason::Socket => "a socket",
        }
    }
}

/// What other threads post to an interpreter's inbox.
pub(crate) enum Posted {
    Completed(Completed),
    Recall(Recall),
    /// `activity`'s blocking operation, posted under `token`, has ended.
    Unblocked {
        activity: ActivityId,
        token: u64,
        ended: crate::command::Waited,
    },
    /// What `activity`'s redirected command, posted under `token`, wrote to
    /// its output target, which the parked clause writes.
    Captured {
        activity: ActivityId,
        token: u64,
        bytes: Vec<u8>,
    },
    /// `activity`'s socket wait, started under `token`, has ended: its
    /// sockets are ready, its deadline has passed, or it was cancelled.
    #[cfg(unix)]
    SocketWaited {
        activity: ActivityId,
        token: u64,
    },
    /// What a command's child off the baton wrote, to standard error where
    /// `error`, written to the interpreter's own as it arrives.
    Output {
        error: bool,
        bytes: Vec<u8>,
    },
    /// A pool thread's job panicked; the baton's holder panics with it.
    Panicked(Box<dyn std::any::Any + Send>),
    /// A signal arrived: every activity is halted ([`Interp::halt_all`]).
    Halt,
    /// `RexxSetTrace` (`true`) or `RexxResetTrace` named this run's thread: the running Rexx
    /// frame's external trace ([`Interp::set_external_trace`]).
    Trace(bool),
    /// `SetTrace` of an instance: every activity's external trace, on or off
    /// ([`Interp::set_external_trace_all`]).
    TraceAll(bool),
    /// A chunk of standard input, empty at its end.
    Input(Vec<u8>),
    /// `activity`'s native call in the native frame `frame`, which ran with
    /// the baton vacated ([`Vacancy`]), has returned: the thread serving the
    /// vacancy stops and gives the baton back.
    Returned {
        activity: ActivityId,
        frame: u64,
    },
}

/// A native call running with the baton vacated, from just before it
/// prepares until it has finished: its activity, and its native frame as
/// [`Interp::native_token`] names it. The calls nest, innermost last, as the
/// vacancies on the baton do.
pub(crate) struct Vacancy {
    pub(crate) activity: ActivityId,
    pub(crate) frame: u64,
    /// Whether a thread serves it ([`Interp::serve_vacancy`]), which counts
    /// it in flight.
    served: bool,
    /// Whether its call has returned ([`Posted::Returned`]).
    returned: bool,
    /// Whether the callback whose call it is was marked waiting before it,
    /// where its activity is the innermost lend's callback: a callback that
    /// arrives during the call is served rather than held.
    waiting: Option<bool>,
}

impl crate::timer::Inbox<Posted> {
    /// Asks every activity of the interpreter to halt at its next clause
    /// (`r[native.threads.halt]`): `Halt` of an instance, from any thread, and
    /// `RexxSetHalt` naming a run. A native call that finds the request pending when
    /// it returns has the calling clause's boundary serve it.
    pub(crate) fn post_halt(&self) {
        if self.post_unless_closed(Posted::Halt) {
            self.requests()
                .set(crate::timer::HALT | crate::timer::BOUNDARY);
        }
    }

    /// Asks every activity of the interpreter to turn its external trace on or off
    /// (`r[native.threads.trace]`), served as [`Self::post_halt`]'s request is.
    pub(crate) fn post_trace_all(&self, on: bool) {
        if self.post_unless_closed(Posted::TraceAll(on)) {
            self.requests().set(crate::timer::BOUNDARY);
        }
    }
}

// Another thread posts it, so it holds no `ObjRef`.
const _: () = {
    const fn require_send<T: Send>() {}
    require_send::<Posted>();
};

/// A native call's completion, posted to the inbox by the thread that ran
/// the call, for the activity that made it.
pub(crate) struct Completed {
    pub(crate) activity: ActivityId,
    /// The call's native frame, as [`Interp::native_token`] names it: a call
    /// abandoned while it ran completes no later call of its activity.
    pub(crate) frame: u64,
    pub(crate) completion: interrexx_api::invoke::Completion,
}

/// A thread's request for the baton, for a callback or a native call's end.
pub(crate) struct Recall {
    pub(crate) by: Recaller,
    pub(crate) thread: std::thread::ThreadId,
    /// The address of a local near the base of that thread's stack, where
    /// Rexx code may run under the lend.
    pub(crate) base: Option<usize>,
    /// How deep Rexx code may run from `base`, where it is not a pool
    /// thread's: a thread attached to an embedding instance.
    pub(crate) room: Option<usize>,
}

/// Which activity runs while a recalled baton is lent.
pub(crate) enum Recaller {
    /// A pool thread running `activity`'s native call in the native frame
    /// `frame`, as [`Interp::native_token`] names it.
    Call { activity: ActivityId, frame: u64 },
    /// A thread running no native call of this interpreter's, calling back
    /// through the activity's thread context at this address.
    Context(usize),
}

/// A started activity's first step: the send `~start` asked for.
pub(crate) struct StartedSend {
    pub(crate) receiver: ObjRef,
    pub(crate) name: Vec<u8>,
    pub(crate) scope: Option<ObjRef>,
    pub(crate) args: Vec<Option<ObjRef>>,
    pub(crate) caller: Caller,
}

impl StartedSend {
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        out.push(self.receiver);
        out.extend(self.scope);
        out.extend(self.args.iter().flatten().copied());
        out.extend(self.caller.receiver());
    }
}

/// An activity that is not running.
pub(crate) struct Idle {
    pub(crate) activity: Activity,
    pub(crate) roots: ActivityRoots,
}

impl Idle {
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        self.activity.object_roots(out);
        out.extend(self.roots.iter());
    }
}

/// A new activity's first step.
pub(crate) enum First {
    /// The send `~start` asked for.
    Send(Box<StartedSend>),
    /// The rest of a replied method body, already on the activity's record.
    Reply,
}

/// Every activity but the running one, and what the scheduler knows of them.
pub(crate) struct Activities {
    running: ActivityId,
    /// Indexed by [`ActivityId`]; `None` for the running activity and for a
    /// free handle.
    idle: Vec<Option<Box<Idle>>>,
    free: Vec<u32>,
    /// Woken and spawned activities, in arrival order.
    pub(crate) ready: VecDeque<ActivityId>,
    /// Sleeping activities, earliest deadline first, then in the order they
    /// parked.
    sleepers: BinaryHeap<Reverse<(Instant, u64, ActivityId)>>,
    /// The order the next sleeper parks in.
    next_sleeper: u64,
    /// The activities whose loops of [`Interp::run_others`], or whose runs
    /// under a recall [`Interp::serve_recall`] serves, are on the Rust stack,
    /// outermost first, each with how its frames lie below the loops above.
    owners: Vec<(ActivityId, Below)>,
    /// Ready owners a loop could not run, each with the index in `owners` of
    /// the loop that set it aside; it goes back to the ready queue when that
    /// loop returns.
    set_aside: Vec<(ActivityId, usize)>,
    timers: FxHashMap<TimerId, Timer>,
    next_timer: u32,
    message_ids: FxHashMap<ObjRef, MessageId>,
    next_message: u32,
    /// The activity each `Message` object was last sent from (`MessageClass`'s `startActivity`,
    /// set by `dispatch`), which `Message~halt` asks. Pruned of swept messages at each
    /// collection.
    senders: FxHashMap<ObjRef, ActivityId>,
    /// The parked activities waiting on each message, in the order they
    /// parked.
    waiters: FxHashMap<MessageId, Vec<ActivityId>>,
    /// The thread contexts of ended activities that made a native call or
    /// ran a package hook, which an extension may have kept and which last
    /// until interpreter termination.
    retired: Vec<interrexx_api::ffi::ThreadContext>,
    /// The thread table every activity's context links.
    thread_table: interrexx_api::ffi::ThreadTable,
    /// The numbers of ended activities a new one takes first, oldest first: a
    /// pool of idle threads, whose threads keep their numbers, including none.
    pooled: VecDeque<Option<u32>>,
    /// The last number assigned.
    last_number: u32,
    /// The next [`crate::NativeFrame`] number, wrapping: one counter for
    /// every activity, so a frame token names its call on one of them.
    next_native: u32,
    /// Failures kept by [`Interp::keep_late_failure`].
    late_failures: Vec<Failure>,
    /// Native calls and blocking operations that have left their driver and
    /// whose completions have not been drained.
    in_flight: usize,
    /// The native frames of calls abandoned while they ran, by frame token
    /// ([`Interp::native_token`]), until their completions are drained.
    abandoned: Vec<(u64, Box<crate::NativeFrame>)>,
    /// Those of the calls in flight whose run is below the running loop,
    /// under a callback, so that no loop above them can wait for their
    /// completions: each by its activity and native frame, once however
    /// many lends nest above it.
    runs_below: Vec<(ActivityId, u64)>,
    /// An embedding instance's root call, by its activity and native frame,
    /// once it has left its driver: it waits until `Terminate`.
    root_call: Option<(ActivityId, u64)>,
    /// The lends that run an attached thread's callback.
    callbacks_lent: usize,
    /// The last token a blocking operation was posted under.
    next_block: u64,
    /// The tokens of blocking operations a halt abandoned, no longer in
    /// flight, until their ends are drained.
    abandoned_blocks: Vec<u64>,
    /// The guard locks, and what each parked activity waits on.
    pub(crate) guards: crate::guards::GuardTable,
    pub(crate) semaphores: crate::semaphores::Semaphores,
    /// The switches from one activity to another.
    switches: u64,
    /// Whether main's program has ended, so main waits for the others as an
    /// activity that has finished.
    main_finished: bool,
    /// Whether the program's end found activities that nothing can wake any
    /// more and refused their wait (Deviation 20).
    ended_stuck: bool,
    /// The level of the loop [`Interp::run_started_activities`] runs the
    /// other activities in at the program's end, while it does.
    end_level: Option<usize>,
    /// The attached activities filed ([`Interp::file_attached`]).
    attached: usize,
    /// The native calls running with the baton vacated, innermost last.
    pub(crate) vacancies: Vec<Vacancy>,
    /// The lends of the baton to a callback ([`Interp::serve_recall`]),
    /// innermost last. Each lender runs again only once its lend has ended
    /// (Deviation 178).
    lends: Vec<Lend>,
    /// The callbacks held while the innermost lend's callback runs, in the
    /// order they arrived, each with the guard lock it waits to be granted,
    /// and the attached activity queued for it, where it gave the baton back
    /// for one ([`Interp::retries_entry_guard`]).
    pub(crate) held: VecDeque<(Recall, Option<(crate::guards::GuardKey, ActivityId)>)>,
    /// The thread contexts whose callback gave the baton back to wait for a
    /// guard lock, each with that lock, until the callback's next recall.
    retrying: Vec<(usize, crate::guards::GuardKey)>,
    /// When the innermost callback wait that only an activity a lend stopped
    /// may end, or the innermost inverted wait that goes on, is reported on
    /// stderr, unless it has ended by then ([`Interp::report_stall`]).
    stall: Option<Stall>,
}

/// A lend of the baton to a callback: the activity it stopped, the one it
/// runs, and whether that one waits, so that a callback arriving meanwhile is
/// served on top of it rather than held.
#[derive(Clone, Copy)]
struct Lend {
    lender: ActivityId,
    lendee: ActivityId,
    waiting: bool,
}

/// A callback's wait that only an activity a lend stopped may end, or a
/// pinned wait that is reported only once it is inverted, and when it is
/// reported.
#[derive(Clone, Copy)]
struct Stall {
    due: Instant,
    reason: ParkReason,
    /// The activity the callback's lend stopped, or `None` for a pinned wait
    /// that is reported only where it is inverted (Deviation 14), naming the
    /// activity below it that it keeps from running.
    lender: Option<ActivityId>,
}

/// How long a callback waits, for something only an activity a lend stopped
/// can give, or an inverted wait that is not refused goes on, before stderr
/// says so.
const STALL_REPORT: std::time::Duration = std::time::Duration::from_secs(5);

/// Milliseconds in a day: a timer's wait is whole days of this, then a
/// remainder.
pub(crate) const TIMER_DAY: std::time::Duration = std::time::Duration::from_secs(86_400);

/// Main's handle, which `Activities::new` gives the activity that runs the
/// program.
const MAIN: ActivityId = ActivityId(0);

/// The Rust stack kept free below the deepest point
/// [`Interp::stack_exhausted`] admits, for the work between two of its
/// checks and the raise of 11.1.
const STACK_MARGIN: usize = 32 * 1024 * 1024;

/// An ended activity is pooled while the pool holds no more than this.
const MAX_POOLED: usize = 5;

impl Activities {
    /// Drops the sender and waiter-identity records of messages `live` no
    /// longer answers for.
    pub(crate) fn prune_messages(&mut self, live: impl Fn(ObjRef) -> bool) {
        self.senders.retain(|message, _| live(*message));
        self.message_ids.retain(|message, _| live(*message));
    }

    /// Every message a sender or waiter-identity record is keyed by.
    #[cfg(test)]
    pub(crate) fn message_rows(&self) -> impl Iterator<Item = ObjRef> + '_ {
        self.senders.keys().chain(self.message_ids.keys()).copied()
    }

    /// The table of an interpreter whose activities' thread contexts link
    /// `thread_table`.
    pub(crate) fn new(thread_table: interrexx_api::ffi::ThreadTable) -> Activities {
        Activities {
            running: MAIN,
            idle: vec![None],
            free: Vec::new(),
            ready: VecDeque::new(),
            sleepers: BinaryHeap::new(),
            next_sleeper: 0,
            owners: Vec::new(),
            set_aside: Vec::new(),
            timers: FxHashMap::default(),
            next_timer: 0,
            message_ids: FxHashMap::default(),
            next_message: 0,
            senders: FxHashMap::default(),
            waiters: FxHashMap::default(),
            retired: Vec::new(),
            thread_table,
            pooled: VecDeque::new(),
            last_number: 0,
            next_native: 0,
            late_failures: Vec::new(),
            in_flight: 0,
            abandoned: Vec::new(),
            runs_below: Vec::new(),
            root_call: None,
            callbacks_lent: 0,
            next_block: 0,
            abandoned_blocks: Vec::new(),
            guards: crate::guards::GuardTable::default(),
            semaphores: crate::semaphores::Semaphores::default(),
            switches: 0,
            main_finished: false,
            ended_stuck: false,
            end_level: None,
            attached: 0,
            vacancies: Vec::new(),
            lends: Vec::new(),
            held: VecDeque::new(),
            retrying: Vec::new(),
            stall: None,
        }
    }

    /// The switches from one activity to another so far.
    pub(crate) fn switches(&self) -> u64 {
        self.switches
    }

    /// The handles the table has made, free ones included.
    pub(crate) fn handles(&self) -> usize {
        self.idle.len()
    }

    /// The running activity's offset in the table.
    pub(crate) fn running_index(&self) -> usize {
        self.running.index()
    }

    /// Puts `activity` at the back of the ready queue unless it is ready
    /// already: a second entry would end the next park it makes.
    fn make_ready(&mut self, activity: ActivityId) {
        if !self.ready.contains(&activity)
            && !self.set_aside.iter().any(|(set, _)| *set == activity)
        {
            self.ready.push_back(activity);
        }
    }

    /// [`crate::activity::Activity::running_programs`] of every activity
    /// that is not running: the only holder of a method that unset its own
    /// entry and is parked in `GUARD WHEN` on a started activity.
    pub(crate) fn running_programs(&self, out: &mut Vec<crate::plan::ProgramId>) {
        for idle in self.idle.iter().flatten() {
            idle.activity.running_programs(out);
        }
    }

    /// [`crate::activity::Activity::for_each_activation_mut`] for every
    /// activity that is not running.
    pub(crate) fn for_each_activation_mut(
        &mut self,
        visit: &mut impl FnMut(&mut crate::activation::Activation),
    ) {
        for idle in self.idle.iter_mut().flatten() {
            idle.activity.for_each_activation_mut(visit);
        }
    }

    /// [`Activity::running_bodies`] for every activity that is not running.
    pub(crate) fn running_bodies(&self, out: &mut Vec<(crate::plan::ProgramId, Option<usize>)>) {
        for idle in self.idle.iter().flatten() {
            idle.activity.running_bodies(out);
        }
    }

    /// [`Activity::running_methods`] for every activity that is not running.
    pub(crate) fn running_methods(&self, out: &mut Vec<interrexx_classes::MethodId>) {
        for idle in self.idle.iter().flatten() {
            idle.activity.running_methods(out);
        }
    }

    /// Appends every `ObjRef` an activity that is not running holds.
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        for failure in &self.late_failures {
            if let Failure::Exited(value) = failure {
                out.extend(*value);
            }
        }
        for idle in self.idle.iter().flatten() {
            idle.object_roots(out);
        }
        for (_, frame) in &self.abandoned {
            frame.object_roots(out);
        }
        for timer in self.timers.values() {
            out.extend(
                timer
                    .pending
                    .iter()
                    .flat_map(|(receiver, scope, _)| [*receiver, *scope]),
            );
            out.extend(
                timer
                    .waiters
                    .iter()
                    .flat_map(|waiter| [waiter.receiver, waiter.scope]),
            );
        }
        self.semaphores.object_roots(out);
    }

    /// The identity `message`'s waiters park under.
    pub(crate) fn message_id(&mut self, message: ObjRef) -> MessageId {
        let next = &mut self.next_message;
        *self.message_ids.entry(message).or_insert_with(|| {
            *next += 1;
            MessageId(*next)
        })
    }

    /// Whether a native call or a blocking operation is in flight.
    /// Holds the native frame of a call abandoned while it ran, named by the
    /// frame token `token`, until its completion is drained.
    pub(crate) fn keep_abandoned(&mut self, token: u64, frame: Box<crate::NativeFrame>) {
        self.abandoned.push((token, frame));
    }

    /// The native frame [`Activities::keep_abandoned`] holds for `token`.
    pub(crate) fn take_abandoned(&mut self, token: u64) -> Option<Box<crate::NativeFrame>> {
        let row = self.abandoned.iter().position(|(held, _)| *held == token)?;
        Some(self.abandoned.swap_remove(row).1)
    }

    pub(crate) fn in_flight(&self) -> bool {
        self.in_flight > 0
    }

    /// Whether no message is held or to be sent again, and no call is in flight but those a
    /// lend's callback runs above (`r[native.threads.attach.deviation.178]`).
    fn nothing_held_or_in_flight(&self) -> bool {
        self.in_flight == self.runs_below.len() && self.held.is_empty() && self.retrying.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn retired(&self) -> &[interrexx_api::ffi::ThreadContext] {
        &self.retired
    }
}

/// How an owner's frames lie below the loops above it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Below {
    /// Its state is all in its record: a loop above may run it.
    Free,
    /// It waits from Rust frames that stay live below the loops above.
    Buried,
    /// Buried in the middle of a run, which a recall served at one of its
    /// clause boundaries interrupted: it runs on once that recall's lend
    /// returns, parked on nothing.
    Interrupted,
}

/// Where a drain of the inbox finds the running activity.
#[derive(Clone, Copy)]
pub(crate) enum DrainAt {
    /// At a clause boundary in the middle of its run, which goes on once the
    /// drain returns.
    MidRun,
    /// Stopped: a loop's own activity, parked, or ready where it yields
    /// mid-run to a round of the others (a pinned yield, or the yield after a
    /// notifier's failure), its frames buried below that round; or one a loop
    /// ran that has parked, ended or come to its slice's end. A recall served
    /// here marks the activity it runs above as buried, which a loop above
    /// sets aside while it is ready.
    Stopped,
}

/// How a loop of [`Interp::run_others`] ended.
enum Waited {
    /// The activity it set aside is ready.
    Woken,
    /// Nothing it can run is ready; `inverted` where an activity is ready
    /// that only a loop below it can run, or a recall below it interrupted
    /// one.
    Blocked { inverted: bool },
}

/// How a run of the running activity stopped.
enum Stopped {
    /// Parked, or at the back of the ready queue with its slice over.
    Parked,
    Ended,
}

/// The deterministic switch mode in force, and the clauses counted under it.
pub(crate) struct Switch {
    mode: SwitchMode,
    clauses: u64,
}

/// What the driver asks of whatever runs the interpreter's activities.
pub(crate) trait Scheduler {
    /// Files `idle`, a record [`Interp::new_activity`] made, as ready.
    fn spawn(&mut self, idle: Box<Idle>) -> ActivityId;
    /// Runs the running activity, a started one, until it parks, which
    /// answers `true`, or ends.
    fn run_until_park(&mut self) -> Result<bool, Failure>;
    /// Registers the running activity as waiting for `reason`.
    fn park(&mut self, reason: ParkReason);
    /// Moves a parked activity to the back of the ready queue.
    fn unpark(&mut self, activity: ActivityId);
    /// Puts the running activity, whose slice is over, at the back of the
    /// ready queue.
    fn yield_at_slice(&mut self);
    /// Stops every activity's driver for a collection: with the one thread
    /// holding the baton running a driver, that is holding the baton.
    fn stop_the_world(&self);
    /// Runs the native call the running activity, parked for it, prepared,
    /// off the baton: on a pool thread, which this
    /// thread lends the baton to while it enters the call, or, where the pool
    /// is configured inline, here on the baton. Its completion is posted.
    /// A spawn failure wakes the original continuation without starting it.
    fn exit_for_native(&mut self);
    /// Runs `block`, a blocking operation of the running activity's that
    /// touches no island value, on a pool thread, which posts what it ended
    /// with under the token answered. The worker was reserved before the
    /// command child was started.
    fn exit_for_block(
        &mut self,
        block: crate::command::Block,
        worker: crate::command::ReservedWorker,
    ) -> u64;
    /// Posts `completion`, of `activity`'s native call in the native frame
    /// `frame`, to `inbox` for the baton's holder to drain; it touches
    /// nothing else.
    fn post_completion(
        inbox: &crate::timer::Inbox<crate::timer::Posted>,
        activity: ActivityId,
        frame: u64,
        completion: interrexx_api::invoke::Completion,
    ) where
        Self: Sized;
}

impl Scheduler for Interp {
    fn spawn(&mut self, idle: Box<Idle>) -> ActivityId {
        self.guards_go_live(None);
        // The spawner is numbered.
        self.activity_number();
        let table = &mut self.activities;
        let id = match table.free.pop() {
            Some(index) => ActivityId(index),
            None => {
                table.idle.push(None);
                ActivityId((table.idle.len() - 1) as u32)
            }
        };
        table.idle[id.0 as usize] = Some(idle);
        table.ready.push_back(id);
        if self.sim.is_some() {
            self.sim_spawned(id.index());
        }
        id
    }

    fn run_until_park(&mut self) -> Result<bool, Failure> {
        let ended = match self.run_started() {
            Ok(Stopped::Parked) => {
                self.exit_if_awaited();
                return Ok(true);
            }
            Ok(Stopped::Ended) => self.run_ending_uninits(),
            Err(failure) => Err(failure),
        };
        self.release_ended_mutexes();
        ended.map(|()| false)
    }

    fn park(&mut self, reason: ParkReason) {
        let running = self.activities.running;
        let table = &mut self.activities;
        self.activity.woken_by_halt = false;
        self.activity.semaphore_wait = None;
        self.activity.parked_block = None;
        self.activity.parked_socket = None;
        match reason {
            ParkReason::Guard(_) => {}
            ParkReason::GuardWhen => self.activity.when_parked = true,
            ParkReason::MessageResult(id) | ParkReason::MessageWait(id) => {
                table.waiters.entry(id).or_default().push(running);
                table
                    .guards
                    .set_waiting(running, crate::guards::Waiting::Message(id));
            }
            ParkReason::Sleep { deadline } => {
                table.next_sleeper += 1;
                table
                    .sleepers
                    .push(Reverse((deadline, table.next_sleeper, running)));
                self.activity.asleep = Some(table.next_sleeper);
            }
            ParkReason::Timer { deadline, cancel } => {
                table.next_sleeper += 1;
                let order = table.next_sleeper;
                table.sleepers.push(Reverse((deadline, order, running)));
                if let Some(timer) = table.timers.get_mut(&cancel)
                    && let Some((receiver, scope, days_end)) = timer.pending.take()
                {
                    timer.waiters.push(TimerWaiter {
                        activity: running,
                        order,
                        receiver,
                        scope,
                        days_end,
                    });
                }
            }
            ParkReason::Native => {}
            ParkReason::Block => {
                self.activity.parked_block = self.activity.blocked.innermost_token()
            }
            #[cfg(unix)]
            ParkReason::Socket => match crate::builtin::rxsock::helper_state(&self.activity) {
                // Its end was drained before the park: woken at once.
                Some((_, true)) | None => self.unpark(running),
                Some((token, false)) => self.activity.parked_socket = Some(token),
            },
            ParkReason::Semaphore(wait) => {
                let order = wait.deadline.map(|deadline| {
                    table.next_sleeper += 1;
                    let order = table.next_sleeper;
                    table.sleepers.push(Reverse((deadline, order, running)));
                    order
                });
                self.activity.semaphore_wait = Some(order);
                if !matches!(wait.kind, crate::semaphores::WaitKind::Poll(_)) {
                    table.semaphores.enqueue(
                        wait.key,
                        crate::semaphores::Waiter {
                            activity: running,
                            order,
                        },
                    );
                }
            }
        }
    }

    fn unpark(&mut self, activity: ActivityId) {
        self.activities.guards.woken(activity);
        self.activities.make_ready(activity);
    }

    fn yield_at_slice(&mut self) {
        let running = self.activities.running;
        self.activities.ready.push_back(running);
    }

    fn stop_the_world(&self) {
        assert!(
            self.baton.held_here(),
            "a collection on a thread not holding the interpreter's baton"
        );
    }

    fn exit_for_native(&mut self) {
        if self.stress_collect {
            self.collect_now();
        }
        if self.activity.native_call.is_none() {
            return;
        }
        let simulated = self.sim_native_pool();
        let worker = match self.pool.reserve_native() {
            Ok(worker) => worker,
            Err(failure) => {
                self.activity
                    .native_call
                    .as_mut()
                    .expect("prepared call")
                    .spawn_failed(failure);
                self.unpark(self.activities.running);
                return;
            }
        };
        let park = self.activity.native_park.take();
        let Some(record) = self.activity.native_call.as_mut() else {
            self.activity.native_park = park;
            return;
        };
        let (mut call, frame, pending) = record.leave(park);
        #[cfg(test)]
        NATIVE_EXITS.with(|exits| exits.set(exits.get() + 1));
        self.activities.in_flight += 1;
        let thread = self.thread_context();
        let baton = crate::sync::Arc::clone(&self.baton);
        let inbox = self.timer.inbox();
        let activity = self.activities.running;
        if let Some(worker) = worker {
            let to = worker.thread();
            let pooled = crate::dispatch::library::PooledCall::new(
                call,
                thread,
                frame,
                pending,
                activity,
                &baton,
                crate::sync::Arc::clone(&inbox),
            );
            let lent = crate::sync::Arc::clone(&baton);
            let simulation = simulated.then(|| crate::sync::Arc::new(crate::timer::Inbox::new()));
            let posts = simulation.clone().unwrap_or(inbox);
            let bound = self.sim_block_bound();
            worker.run(Box::new(move || {
                pool::posting_panics(&posts, &lent, || pooled.run(&posts, bound));
            }));
            baton.lend(to, crate::island::Island::of(self));
            if let Some(inbox) = simulation {
                self.await_simulated_native(&inbox);
                self.pool.wait_for_return(to);
            }
            return;
        }
        pin_enter!(self, crate::pinning::PinKind::NativeApiCallback);
        self.activities.runs_below.push((activity, frame));
        let watch = self.sim_watch_native();
        let pending = call.run(frame, pending, self, &baton, &thread, |completion| {
            Self::post_completion(&inbox, activity, frame, completion);
        });
        if let Some(watch) = watch {
            self.sim_end_watch(watch);
        }
        self.activities.runs_below.pop();
        pin_leave!(self);
        if let Some(record) = self.activity.native_call.as_mut() {
            self.activity.native_park = record.back(call, pending);
        }
    }

    fn exit_for_block(
        &mut self,
        block: crate::command::Block,
        worker: crate::command::ReservedWorker,
    ) -> u64 {
        self.activities.in_flight += 1;
        self.activities.next_block += 1;
        let token = self.activities.next_block;
        let inbox = self.timer.inbox();
        let baton = crate::sync::Arc::clone(&self.baton);
        let activity = self.activities.running;
        worker.run(Box::new(move || {
            pool::posting_panics(&inbox, &baton, || {
                let ended = crate::signal::unblocked(|| {
                    block.stream(&|piece, bytes| {
                        inbox.post(match piece {
                            crate::command::Piece::Out => Posted::Output {
                                error: false,
                                bytes,
                            },
                            crate::command::Piece::Err => Posted::Output { error: true, bytes },
                            crate::command::Piece::Captured => Posted::Captured {
                                activity,
                                token,
                                bytes,
                            },
                        });
                    })
                });
                inbox.post(Posted::Unblocked {
                    activity,
                    token,
                    ended,
                });
            });
        }));
        token
    }

    fn post_completion(
        inbox: &crate::timer::Inbox<crate::timer::Posted>,
        activity: ActivityId,
        frame: u64,
        completion: interrexx_api::invoke::Completion,
    ) {
        inbox.post(Posted::Completed(Completed {
            activity,
            frame,
            completion,
        }));
    }
}

/// The clauses an activation runs before it may give up the kernel.
const DELIVERY_YIELD_CLAUSES: u32 = 50;

/// The slices that fall due while a trace delivery holds a guard, with another
/// activity ready, before the delivery is refused as an inverted wait, or,
/// where waits are not refused as unending, reported on stderr. The bound lies
/// well past the slices a destination that is doing work takes.
const GUARDED_DELIVERY_SLICES: u32 = 250;

impl Interp {
    /// Registers a wait for `polls` until `deadline` with the process's
    /// socket reactor ([`crate::builtin::rxsock_wait`]) for the running
    /// activity, and answers the token its end is posted under
    /// ([`Posted::SocketWaited`]) and the registration; `None` where there
    /// is no reactor to take it. Counted in flight like a pool thread's
    /// wait, so [`Interp::wind_down`] waits for its end, which a
    /// cancellation posts too.
    #[cfg(unix)]
    pub(crate) fn register_socket_wait(
        &mut self,
        polls: &[(i32, rustix::event::PollFlags)],
        deadline: Option<std::time::Instant>,
    ) -> Option<(u64, crate::builtin::rxsock_wait::Wait)> {
        let token = self.activities.next_block + 1;
        let inbox = self.timer.inbox();
        let activity = self.activities.running;
        let registered = crate::builtin::rxsock_wait::register(
            polls,
            deadline,
            Box::new(move || inbox.post(Posted::SocketWaited { activity, token })),
        )
        .ok()?;
        self.activities.next_block = token;
        self.activities.in_flight += 1;
        Some((token, registered))
    }

    /// Runs `wait` on `worker`, a pool thread, for the running activity,
    /// and answers the token its end is posted under
    /// ([`Posted::SocketWaited`]).
    #[cfg(unix)]
    pub(crate) fn exit_for_socket_wait(
        &mut self,
        worker: Worker,
        wait: Box<dyn FnOnce() + Send>,
    ) -> u64 {
        self.activities.in_flight += 1;
        self.activities.next_block += 1;
        let token = self.activities.next_block;
        let inbox = self.timer.inbox();
        let baton = crate::sync::Arc::clone(&self.baton);
        let activity = self.activities.running;
        worker.run(Box::new(move || {
            pool::posting_panics(&inbox, &baton, || {
                wait();
                inbox.post(Posted::SocketWaited { activity, token });
            });
        }));
        token
    }

    /// A record for a new activity, with an empty arena of the running one's
    /// block size and the oldest pooled number.
    pub(crate) fn new_activity(&mut self) -> Box<Idle> {
        let mut activity = Activity::new();
        activity.number = self.activities.pooled.pop_front().flatten();
        #[cfg(feature = "sharing")]
        {
            activity.sharing_tag = self.heap.sharing_tag();
        }
        let mut roots = ActivityRoots::new();
        roots.set_frame_block(self.roots.activity().frames().size());
        Box::new(Idle { activity, roots })
    }

    /// The running activity's thread context, made at its first native call
    /// or package hook. The main activity's first one is the interpreter's
    /// root context, which a thread running no native call of the
    /// interpreter attaches to (`r[native.threads.instance]`).
    pub(crate) fn thread_context(&mut self) -> interrexx_api::ffi::ThreadContext {
        let main = self.activities.running == MAIN;
        self.activity
            .thread
            .get_or_insert_with(|| {
                let thread =
                    interrexx_api::ffi::ThreadContext::linking(&self.activities.thread_table);
                if main {
                    thread.make_root();
                }
                thread
            })
            .clone()
    }

    /// The idle activity holding the activation whose `RexxContext` is
    /// `context`.
    pub(crate) fn idle_context_owner(&self, context: ObjRef) -> Option<ActivityId> {
        let holds = |idle: &Idle| {
            let activity = &idle.activity;
            activity
                .running
                .iter()
                .chain(&activity.suspended)
                .any(|activation| activation.context_object == Some(context))
        };
        let index = self
            .activities
            .idle
            .iter()
            .position(|idle| idle.as_deref().is_some_and(holds))?;
        Some(ActivityId(u32::try_from(index).expect("handles fit u32")))
    }

    /// Runs `read` with the idle activity `id` in the running one's place, as
    /// [`Interp::switch_to`] puts it there, and both back where they were
    /// after. `read` may allocate; it may not park, switch or run Rexx.
    pub(crate) fn with_idle_activity<R>(
        &mut self,
        id: ActivityId,
        read: impl FnOnce(&mut Interp) -> R,
    ) -> R {
        self.swap_idle(id);
        let frame = self.roots.activity_mut().push_frame();
        let before = self.idle_read_state();
        let answer = read(self);
        debug_assert!(
            self.idle_read_state() == before,
            "a read of an idle activity parked, switched or ran Rexx"
        );
        self.roots.activity_mut().pop_frame(frame);
        self.swap_idle(id);
        answer
    }

    /// What [`Interp::with_idle_activity`]'s `read` may not change: the
    /// running handle, the ready queue's length, the pin depth, the
    /// activation stack and the open driver constructs.
    fn idle_read_state(&self) -> (ActivityId, usize, u32, usize, Option<u64>, usize) {
        let activity = &self.activity;
        (
            self.activities.running,
            self.activities.ready.len(),
            activity.pin_depth,
            activity.suspended.len(),
            activity.running.as_ref().map(|running| running.id.0),
            activity.frames.len(),
        )
    }

    fn swap_idle(&mut self, id: ActivityId) {
        let Some(idle) = self.activities.idle[id.0 as usize].as_deref_mut() else {
            unreachable!("an idle handle names an idle activity");
        };
        std::mem::swap(&mut self.activity, &mut idle.activity);
        std::mem::swap(self.roots.activity_mut(), &mut idle.roots);
    }

    /// A number for a new native frame.
    pub(crate) fn next_native_id(&mut self) -> u32 {
        let id = self.activities.next_native;
        self.activities.next_native = id.wrapping_add(1);
        id
    }

    /// The idle activity whose native call the frame token `frame`
    /// ([`Interp::native_token`]) names, and that call's row.
    pub(crate) fn idle_native_owner(&self, frame: u64) -> Option<(ActivityId, usize)> {
        let row = usize::try_from(frame >> 32).ok()?;
        let id = frame & u64::from(u32::MAX);
        let index = self.activities.idle.iter().position(|idle| {
            idle.as_deref().is_some_and(|idle| {
                idle.activity
                    .native_handles
                    .get(row)
                    .is_some_and(|native| u64::from(native.id) == id)
            })
        })?;
        Some((
            ActivityId(u32::try_from(index).expect("handles fit u32")),
            row,
        ))
    }

    /// Whether the running activity has the native frame the frame token
    /// `frame` names.
    pub(crate) fn runs_native_frame(&self, frame: u64) -> bool {
        let id = frame & u64::from(u32::MAX);
        usize::try_from(frame >> 32)
            .ok()
            .and_then(|row| self.activity.native_handles.get(row))
            .is_some_and(|native| u64::from(native.id) == id)
    }

    /// `read` of the activity whose native call the frame token `frame`
    /// names: the running one as it is, or an idle one as
    /// [`Interp::with_idle_activity`] puts it in place.
    pub(crate) fn with_native_owner<R>(
        &mut self,
        frame: u64,
        read: impl FnOnce(&mut Interp) -> Option<R>,
    ) -> Option<R> {
        if self.runs_native_frame(frame) {
            return read(self);
        }
        let (owner, _) = self.idle_native_owner(frame)?;
        self.with_idle_activity(owner, read)
    }

    /// Whether an activity waits in the ready queue.
    pub(crate) fn any_ready(&self) -> bool {
        !self.activities.ready.is_empty()
    }

    /// How many activity slots exist besides main's: none before the first
    /// spawn.
    pub(crate) fn activities_spawned(&self) -> usize {
        self.activities.idle.len() - 1
    }

    /// Moves the oldest pooled number to the pool's tail, as a spawned
    /// activity that ends at once does.
    pub(crate) fn rotate_pooled(&mut self) {
        let pooled = &mut self.activities.pooled;
        let number = pooled.pop_front().flatten();
        if pooled.len() <= MAX_POOLED {
            pooled.push_back(number);
        }
    }

    /// The running activity's number, which `.context~thread` answers: main is
    /// 1 and an activity is numbered when first asked, as threads are numbered.
    pub(crate) fn activity_number(&mut self) -> u32 {
        if let Some(number) = self.activity.number {
            return number;
        }
        self.activities.last_number += 1;
        let number = self.activities.last_number;
        self.activity.number = Some(number);
        number
    }

    /// The running activity's handle.
    pub(crate) fn running_activity(&self) -> ActivityId {
        self.activities.running
    }

    /// Whether the waits chained from `owner`'s lead back to the running
    /// activity: each waiting activity waits on a guard lock's owner or on
    /// the activity running a message's send.
    pub(crate) fn deadlocks(&self, owner: ActivityId) -> bool {
        let me = self.activities.running;
        let mut at = owner;
        for _ in 0..self.activities.idle.len() {
            match self.waited_on(at) {
                Some(next) if next == me => return true,
                Some(next) => at = next,
                None => return false,
            }
        }
        false
    }

    /// Whether some activity waits for the result or the completion of a
    /// message whose send it runs itself and has not replied from: a wait
    /// nothing can end, whatever was loaded (Deviation 20).
    pub(crate) fn waits_on_its_own_message(&self) -> bool {
        // The send completes only when its method returns, and its method is
        // what waits, so no thread that attaches later can end the wait. A
        // send whose method has replied has its result already, and its
        // `result` and `wait` answer without a wait.
        (self.activities.guards.waiters()).any(|waiter| self.waits_on_a_message_it_runs(waiter))
    }

    /// Whether `at` waits for the result or the completion of a message
    /// whose send it runs itself and has not replied from.
    fn waits_on_a_message_it_runs(&self, at: ActivityId) -> bool {
        self.waited_on(at) == Some(at)
    }

    /// The activity `at` waits on, as [`Interp::waits_on`] names it.
    fn waited_on(&self, at: ActivityId) -> Option<ActivityId> {
        self.waits_on(self.activities.guards.waiting(at)?)
    }

    /// The activity a wait for `waiting` waits on: the owner of the guard
    /// lock, or the activity running the send of the message whose
    /// completion it waits for.
    fn waits_on(&self, waiting: crate::guards::Waiting) -> Option<ActivityId> {
        match waiting {
            crate::guards::Waiting::Guard(key) => self.activities.guards.owner(key),
            crate::guards::Waiting::Message(id) => self
                .activities
                .message_ids
                .iter()
                .find(|(_, waited)| **waited == id)
                .and_then(|(message, _)| self.message_runner(*message)),
        }
    }

    /// The activity a lend to a callback stopped that a wait for `reason`
    /// waits on, through a chain of waits. It runs again only once that
    /// callback has ended (Deviation 178).
    fn lender_waited_on(&self, reason: ParkReason) -> Option<ActivityId> {
        let lends = &self.activities.lends;
        if lends.is_empty() {
            return None;
        }
        let mut at = match reason {
            ParkReason::Guard(key) => self.waits_on(crate::guards::Waiting::Guard(key)),
            ParkReason::MessageResult(id) | ParkReason::MessageWait(id) => {
                self.waits_on(crate::guards::Waiting::Message(id))
            }
            _ => None,
        };
        for _ in 0..self.activities.idle.len() {
            let next = at?;
            if lends.iter().any(|lend| lend.lender == next) {
                return Some(next);
            }
            at = self.waited_on(next);
        }
        None
    }

    /// Whether a wait for `reason` is refused at once: it waits on an activity
    /// a lend to a callback stopped, unless it waits for a guard lock and
    /// that activity is an attached activity's callback, a wait that goes on
    /// (Deviation 178).
    fn waits_on_a_lender(&self, reason: ParkReason) -> bool {
        self.lender_waited_on(reason).is_some_and(|lender| {
            !matches!(reason, ParkReason::Guard(_))
                || !self.record(lender).is_some_and(|record| record.attached)
        })
    }

    /// `activity`'s record, running or idle.
    fn record(&self, activity: ActivityId) -> Option<&Activity> {
        if activity == self.activities.running {
            return Some(&self.activity);
        }
        (self.activities.idle.get(activity.0 as usize))
            .and_then(Option::as_ref)
            .map(|idle| &idle.activity)
    }

    /// Whether the guard lock `key` that a callback's top-level send waits
    /// for at its entry, before it has run anything, is held by an activity
    /// a lend to a callback stopped, and the callback's member sends again
    /// when asked. If so, the callback's thread is asked to give the baton
    /// back and send again, its attached activity keeps the place in `key`'s
    /// queue that the wait took when it began, at `place` for a wait that
    /// had queued, and its next recall is held until a release grants it the
    /// lock in that turn ([`Interp::queue_held_retry`]), as ooRexx has the
    /// send wait for the lock (`r[native.threads.attach.deviation.178]`).
    /// The caller then fails the send with [`Loud::retried_send`], which the
    /// send answers no one. `entered` says whether the running activation is
    /// the method whose entry waits, rather than one the wait comes in the
    /// middle of.
    pub(crate) fn retries_entry_guard(
        &mut self,
        key: crate::guards::GuardKey,
        entered: bool,
        place: Option<usize>,
    ) -> bool {
        // Nothing of the callback's message has run only where no activation
        // lies below the wait, and none is running but the one it enters: a
        // send made by a running activation comes after that activation's
        // earlier clauses, which a second send would run again.
        if !self.activity.attached
            || self.activity.native_handles.len() != 1
            || !self.activity.suspended.is_empty()
            || self.activity.running.is_some() != entered
            || self.lender_waited_on(ParkReason::Guard(key)).is_none()
        {
            return false;
        }
        let Some(thread) = &self.activity.thread else {
            return false;
        };
        if !thread.ask_retry() {
            return false;
        }
        let address = thread.address();
        self.activities.retrying.push((address, key));
        let me = self.running_activity();
        self.activities.guards.queue_retry(key, me, place);
        // The send runs again from its entry, which announces the method once
        // per message: this run leaves without `<I<`, and the next enters
        // without `>I>`.
        if entered && self.activation().trace_entry == crate::activation::TraceEntry::Done {
            self.activation_mut().trace_entry = crate::activation::TraceEntry::Spent;
            self.activity.retraced = Some(key);
        }
        true
    }

    /// Whether the running activation enters the method whose `>I>` its
    /// callback wrote before it gave the baton back to send again
    /// ([`Interp::retries_entry_guard`]). The record ends at the send's
    /// next announced entry, whichever method that is.
    pub(crate) fn entry_traced_before_retry(&mut self) -> bool {
        let Some(key) = self.activity.retraced.take() else {
            return false;
        };
        self.activity.suspended.is_empty()
            && (self.running_activation())
                .and_then(|running| running.method_identity.as_ref())
                .is_some_and(|identity| {
                    identity.receiver == key.object && identity.scope == key.scope
                })
    }

    /// Whether the running activity's send failed only to be sent again
    /// ([`Interp::retries_entry_guard`]).
    pub(crate) fn send_retried(&self) -> bool {
        let Some(thread) = &self.activity.thread else {
            return false;
        };
        let address = thread.address();
        (self.activities.retrying.iter()).any(|(context, _)| *context == address)
    }

    /// The activity running `message`'s send, started or sent, while it runs.
    pub(crate) fn message_runner(&self, message: ObjRef) -> Option<ActivityId> {
        let runs = |activity: &Activity| {
            matches!(activity.root_then, Some(Then::Started(started)) if started == message)
                || activity
                    .native_tails
                    .iter()
                    .any(|tail| tail.records(message))
                || activity
                    .native_park
                    .as_ref()
                    .is_some_and(|park| park.records(message))
        };
        if runs(&self.activity) {
            return Some(self.activities.running);
        }
        let index = self
            .activities
            .idle
            .iter()
            .position(|idle| idle.as_deref().is_some_and(|idle| runs(&idle.activity)))?;
        Some(ActivityId(u32::try_from(index).expect("handles fit u32")))
    }

    /// Posts `watcher`'s guard semaphore: its next `GUARD WHEN` wait ends at
    /// once, and one it is parked in ends now.
    pub(crate) fn post_guard(&mut self, watcher: ActivityId) {
        if watcher == self.activities.running {
            self.activity.guard_posted = true;
            return;
        }
        let Some(Some(idle)) = self.activities.idle.get_mut(watcher.0 as usize) else {
            return;
        };
        idle.activity.guard_posted = true;
        if std::mem::take(&mut idle.activity.when_parked) {
            self.unpark(watcher);
        }
    }

    /// Wakes every activity waiting on `message`, whose send has completed.
    pub(crate) fn message_completed(&mut self, message: ObjRef) {
        let Some(id) = self.activities.message_ids.remove(&message) else {
            return;
        };
        let mark = self.activities.ready.len();
        for waiter in self.activities.waiters.remove(&id).unwrap_or_default() {
            self.unpark(waiter);
        }
        if self.sim.is_some() {
            self.sim_order_event(mark);
        }
    }

    /// Puts the running activity's state into `next`'s record and runs
    /// `next`; the outgoing activity stays filed under its handle, or is
    /// retired where it has `ended`.
    fn switch_to(&mut self, next: ActivityId, ended: bool) {
        let table = &mut self.activities;
        let Some(mut idle) = table.idle[next.0 as usize].take() else {
            unreachable!("a ready handle names an idle activity");
        };
        std::mem::swap(&mut self.activity, &mut idle.activity);
        std::mem::swap(self.roots.activity_mut(), &mut idle.roots);
        // The key a compound read holds is the outgoing activity's, and the
        // incoming one may be in the middle of a clause of its own: one
        // waiting off the baton for a native call, which pushed nothing.
        self.shared_key_for = None;
        #[cfg(feature = "sharing")]
        self.heap.share_as(self.activity.sharing_tag);
        let outgoing = std::mem::replace(&mut table.running, next);
        table.switches += 1;
        if ended {
            table.retired.extend(idle.activity.thread.take());
            table.free.push(outgoing.0);
            if table.pooled.len() <= MAX_POOLED {
                table.pooled.push_back(idle.activity.number);
            }
        } else {
            table.idle[outgoing.0 as usize] = Some(idle);
        }
        // A slice belongs to the activity it was set for.
        self.timer.requests().clear(SLICE);
        self.slice_deferred = false;
        if self.activities.ready.is_empty() && self.activities.in_flight == 0 {
            self.timer.disarm();
        }
        if self.stress_collect {
            self.collect_now();
        }
        if self.sim.is_some() {
            self.sim_check_switch();
        }
    }

    /// The scheduler's invariants, which the
    /// simulation mode checks at every switch: this thread holds the baton;
    /// each activity is running, ready, parked or finished, and one only; a
    /// ready activity holds no park reason and a parked one has a wake
    /// source; the guard queues agree with the waits recorded. One pass over
    /// the table's queues marks each handle, then each handle is checked.
    pub(crate) fn check_invariants(&self) -> Result<(), Loud> {
        let fail = |what| Err(Loud::scheduler_inconsistency(what));
        #[cfg(test)]
        let held = self.baton.held_here() && !BATON_ELSEWHERE.with(std::cell::Cell::get);
        #[cfg(not(test))]
        let held = self.baton.held_here();
        if !held {
            return fail("a switch on a thread not holding the baton");
        }
        let table = &self.activities;
        let running = table.running;
        let filed =
            |activity: ActivityId| matches!(table.idle.get(activity.index()), Some(Some(_)));
        if filed(running) {
            return fail("a running activity filed as idle");
        }
        if let Some(what) = table
            .guards
            .inconsistency(|activity| activity == running || filed(activity))
        {
            return fail(what);
        }
        let handles = table.idle.len();
        // A sleeper, a message's waiter or a guard's waiter; a `GUARD WHEN`
        // is the other park reason, kept on the activity's own record.
        let mut parked = vec![false; handles];
        let sleepers = table
            .sleepers
            .iter()
            .map(|Reverse((_, _, sleeper))| *sleeper);
        let waiters = table.waiters.values().flatten().copied();
        for activity in sleepers.chain(waiters).chain(table.guards.waiters()) {
            if let Some(mark) = parked.get_mut(activity.index()) {
                *mark = true;
            }
        }
        let when_parked = |activity: ActivityId| {
            matches!(
                table.idle.get(activity.index()),
                Some(Some(idle)) if idle.activity.when_parked
            )
        };
        let mut ready = vec![false; handles];
        let readied = (table.ready.iter().copied())
            .chain(table.set_aside.iter().map(|(activity, _)| *activity));
        for activity in readied {
            if activity == running {
                return fail("an activity both running and ready");
            }
            if ready.get(activity.index()) == Some(&true) {
                return fail("an activity ready twice");
            }
            if !filed(activity) {
                return fail("a ready handle naming no idle activity");
            }
            if parked[activity.index()] || when_parked(activity) {
                return fail("a ready activity holding a park reason");
            }
            ready[activity.index()] = true;
        }
        let mut free = vec![false; handles];
        for &handle in &table.free {
            if let Some(mark) = free.get_mut(handle as usize) {
                *mark = true;
            }
        }
        for (index, slot) in table.idle.iter().enumerate() {
            let activity = ActivityId(u32::try_from(index).expect("handles fit u32"));
            match slot {
                _ if activity == running => {
                    if free[index] {
                        return fail("a running activity's handle free");
                    }
                }
                None if !free[index] => return fail("a handle neither free nor filed"),
                None => {}
                Some(_) if free[index] => return fail("a free handle naming an activity"),
                Some(idle) => {
                    let record = &idle.activity;
                    let finished =
                        record.root_end.is_some() || activity == MAIN && table.main_finished;
                    // A deadline, a post (a message's completion, a
                    // semaphore's post or release, a native call's or a
                    // command's end) or a guard (its release, a store a
                    // `GUARD WHEN` watches); a halt ends each of these.
                    // An attached activity idle between its callbacks is woken
                    // by the next callback its thread makes, and one whose native
                    // call runs with the baton vacated by that call's return.
                    let wakes = parked[index]
                        || record.when_parked
                        || table.semaphores.queues(activity)
                        || record.native_call.is_some()
                        || !record.blocked.is_empty()
                        || record.attached && record.running.is_none()
                        || table
                            .vacancies
                            .iter()
                            .any(|vacancy| vacancy.activity == activity);
                    if !ready[index] && !finished && !wakes {
                        return fail("a parked activity with no wake source");
                    }
                }
            }
        }
        let attached = (table.idle.iter().flatten())
            .filter(|idle| idle.activity.attached)
            .count()
            + usize::from(self.activity.attached);
        if attached != self.attached_count() {
            return fail("an attached activity not counted");
        }
        Ok(())
    }

    /// Runs other activities until the running one, parked, is woken. The
    /// failure, where nothing left to run can wake it or another activity
    /// failed with something that is not a condition, is the one its wait
    /// answers.
    pub(crate) fn wait_until_woken(&mut self) -> Option<Failure> {
        #[cfg(test)]
        if self.activity.native_call.is_some() && std::mem::take(&mut self.fail_native_wait) {
            self.cancel_wait();
            return Some(Raised::insufficient_stack().into());
        }
        let failure = match self.run_others(false) {
            Ok(Waited::Woken) => return None,
            Ok(Waited::Blocked { .. }) => Loud::unsatisfiable_wait().into(),
            Err(failure) => failure,
        };
        self.cancel_wait();
        Some(failure)
    }

    /// A wait for `reason` with Rust frames between the scheduler and the
    /// running driver: the other activities run
    /// on this stack until the running one is woken. Where nothing can run,
    /// the refusal is an inverted wait if an activity is ready whose frames
    /// lie below a loop on the stack, and a wait nothing can end where no
    /// activity is ready at all. A wait [`Interp::waits_on_a_lender`] finds
    /// on an activity a callback stopped is refused as an inverted wait at
    /// once, and so is a guard wait whose lock a release grants such an
    /// activity later ([`Interp::stop_callbacks_behind`]), which leaves
    /// [`Activity::lock_to_lender`] set for the caller.
    pub(crate) fn pinned_wait(&mut self, reason: ParkReason) -> Option<Failure> {
        pinned_park!(self, reason);
        self.activity.lock_to_lender = false;
        self.activity.kept_place = None;
        self.park(reason);
        if self.sim_fails_wait() {
            pinned_unpark!(self);
            self.cancel_wait();
            return Some(Raised::insufficient_stack().into());
        }
        // Whatever else could still run, nothing it does lets that activity
        // run before this wait ends.
        let waited = if self.waits_on_a_lender(reason) {
            Ok(Waited::Blocked { inverted: true })
        } else {
            let waiting = self.set_callback_waiting(true);
            let replaced = self.watch_stall(reason);
            let waited = self.run_others(true);
            // A watch this wait left in place is the loop's own, and where its
            // line was written during the wait it stays consumed.
            if let Some(previous) = replaced {
                self.activities.stall = previous;
            }
            if let Some(waiting) = waiting {
                self.set_callback_waiting(waiting);
            }
            match waited {
                Ok(Waited::Woken) if self.activity.lock_to_lender => {
                    Ok(Waited::Blocked { inverted: true })
                }
                waited => waited,
            }
        };
        pinned_unpark!(self);
        let failure = match waited {
            Ok(Waited::Woken) => return None,
            Ok(Waited::Blocked { inverted: false }) => Loud::unsatisfiable_wait().into(),
            Ok(Waited::Blocked { inverted: true }) => {
                inverted_wait!(self, reason);
                Loud::inverted_wait(reason.waits_for()).into()
            }
            Err(failure) => failure,
        };
        self.cancel_wait();
        Some(failure)
    }

    /// Withdraws the running activity from every message's waiters, from the
    /// guard queues, from the sleepers, from the timers and from the
    /// semaphores.
    fn cancel_wait(&mut self) {
        let running = self.activities.running;
        #[cfg(test)]
        self.note_cancelled_wait();
        self.activity.when_parked = false;
        #[cfg(unix)]
        {
            self.activity.socket_wait = None;
        }
        self.activities.guards.withdraw(running);
        self.activity.kept_place = self.activities.guards.give_up_place(running);
        self.activity.guard_waits.retain(|wait| !wait.queued);
        self.activity.guarded_send = None;
        for waiters in self.activities.waiters.values_mut() {
            waiters.retain(|waiter| *waiter != running);
        }
        self.activities.semaphores.withdraw(running);
        self.activities.timers.retain(|_, timer| {
            let before = timer.waiters.len();
            timer.waiters.retain(|waiter| waiter.activity != running);
            !(timer.ends_with_wait && timer.waiters.len() < before)
        });
        self.activities
            .sleepers
            .retain(|Reverse((_, _, sleeper))| *sleeper != running);
    }

    /// Counts in [`CANCELLED_WAITS`] what [`Interp::cancel_wait`] finds to
    /// withdraw the running activity from.
    #[cfg(test)]
    fn note_cancelled_wait(&self) {
        let running = self.activities.running;
        let found = CancelledWaits {
            when_parks: u64::from(self.activity.when_parked),
            guard_waits: u64::from(self.activities.guards.queues(running)),
            sleeps: u64::from(
                (self.activities.sleepers.iter())
                    .any(|Reverse((_, _, sleeper))| *sleeper == running),
            ),
        };
        CANCELLED_WAITS.with(|cancelled| {
            let counts = cancelled.get();
            cancelled.set(CancelledWaits {
                when_parks: counts.when_parks + found.when_parks,
                guard_waits: counts.guard_waits + found.guard_waits,
                sleeps: counts.sleeps + found.sleeps,
            });
        });
    }

    /// Measures this interpreter's Rust stack from `base`, an address near
    /// the base of a thread stack of `bytes`.
    ///
    /// # Panics
    ///
    /// If `bytes` does not exceed the margin kept free below the deepest
    /// point.
    pub(crate) fn measure_stack(&mut self, base: usize, bytes: usize) {
        assert!(
            bytes > STACK_MARGIN,
            "a thread stack of {bytes} bytes leaves nothing beyond the {STACK_MARGIN}-byte margin"
        );
        self.stack_base = base;
        self.stack_room = bytes - STACK_MARGIN;
    }

    /// Whether `here`, an address on this thread's stack, lies deeper than
    /// the stack may reach: past it a call, a nested scheduler loop or an
    /// expression level raises 11.1 rather than risk the thread's end.
    #[inline(always)]
    pub(crate) fn stack_exhausted<T>(&self, here: *const T) -> bool {
        self.stack_base.abs_diff(here as usize) > self.stack_room
    }

    /// A new timer, not posted, which ends with a wait on it where
    /// `ends_with_wait`.
    pub(crate) fn create_timer(&mut self, ends_with_wait: bool) -> TimerId {
        let table = &mut self.activities;
        table.next_timer += 1;
        let id = TimerId(table.next_timer);
        let timer = Timer {
            ends_with_wait,
            ..Timer::default()
        };
        table.timers.insert(id, timer);
        id
    }

    /// Ends `id`, whose waits are over.
    pub(crate) fn remove_timer(&mut self, id: TimerId) {
        self.activities.timers.remove(&id);
    }

    /// Whether `id` is posted, or `None` for a timer that has ended.
    pub(crate) fn timer_posted(&self, id: TimerId) -> Option<bool> {
        self.activities.timers.get(&id).map(|timer| timer.posted)
    }

    /// Clears `id`'s post.
    pub(crate) fn reset_timer(&mut self, id: TimerId) {
        if let Some(timer) = self.activities.timers.get_mut(&id) {
            timer.posted = false;
        }
    }

    /// Records the wait on `id` about to park: the call's receiver and method
    /// scope, and when its whole days end.
    pub(crate) fn begin_timer_wait(
        &mut self,
        id: TimerId,
        receiver: ObjRef,
        scope: ObjRef,
        days_end: Instant,
    ) {
        if let Some(timer) = self.activities.timers.get_mut(&id) {
            timer.pending = Some((receiver, scope, days_end));
        }
    }

    /// Ends the running activity's wait on `id`: whether the timer is
    /// posted, and whether a post woke a cancelled waiter.
    pub(crate) fn end_timer_wait(&mut self, id: TimerId) -> (bool, bool) {
        let running = self.activities.running;
        let Some(timer) = self.activities.timers.get_mut(&id) else {
            return (false, false);
        };
        timer.waiters.retain(|waiter| waiter.activity != running);
        (timer.posted, timer.cancelled)
    }

    /// The receiver and scope of each wait on `id`, in park order.
    pub(crate) fn timer_waiters(&self, id: TimerId) -> Vec<(ObjRef, ObjRef)> {
        self.activities
            .timers
            .get(&id)
            .map_or_else(Vec::new, |timer| {
                timer
                    .waiters
                    .iter()
                    .map(|waiter| (waiter.receiver, waiter.scope))
                    .collect()
            })
    }

    /// Posts `id`, `cancelled` saying for each of [`Interp::timer_waiters`]
    /// whether its object's `CANCELED` is `.true`, and wakes every waiter. A
    /// waiter in its whole days whose object is not cancelled resets the post
    /// and has its current day end instead, as the day loop does.
    ///
    /// # Panics
    ///
    /// If `cancelled` does not hold one flag per waiter.
    pub(crate) fn post_timer(&mut self, id: TimerId, cancelled: &[bool]) {
        let now = self.now();
        let mark = self.activities.ready.len();
        self.post_timer_at(id, cancelled, now);
        if self.sim.is_some() {
            self.sim_order_event(mark);
        }
    }

    /// [`Interp::post_timer`] at `now`, before the order of the waiters it
    /// readies is settled.
    fn post_timer_at(&mut self, id: TimerId, cancelled: &[bool], now: Instant) {
        let table = &mut self.activities;
        let Some(timer) = table.timers.get_mut(&id) else {
            return;
        };
        assert_eq!(
            timer.waiters.len(),
            cancelled.len(),
            "a timer posted with a cancel flag per waiter"
        );
        let mut moved: Vec<(u64, Instant, Instant)> = Vec::new();
        let mut woken: Vec<u64> = Vec::new();
        let mut waiting = Vec::new();
        for (waiter, &cancelled) in std::mem::take(&mut timer.waiters)
            .into_iter()
            .zip(cancelled)
        {
            let end = waiter.days_end;
            if !cancelled && now < end {
                let begun = (end - now).as_nanos().div_ceil(TIMER_DAY.as_nanos());
                let left = u32::try_from(begun - 1).unwrap_or(u32::MAX);
                let ended = now + TIMER_DAY * left;
                moved.push((waiter.order, end, ended));
                waiting.push(TimerWaiter {
                    days_end: ended,
                    ..waiter
                });
            } else {
                timer.cancelled |= cancelled;
                woken.push(waiter.order);
                table.ready.extend(
                    table
                        .sleepers
                        .iter()
                        .find(|Reverse((_, parked, _))| *parked == waiter.order)
                        .map(|Reverse((_, _, activity))| *activity),
                );
            }
        }
        timer.posted = moved.is_empty();
        timer.waiters = waiting;
        let mut sleepers = std::mem::take(&mut table.sleepers).into_vec();
        sleepers.retain(|Reverse((_, parked, _))| !woken.contains(parked));
        for Reverse((deadline, parked, _)) in &mut sleepers {
            if let Some((_, end, ended)) = moved.iter().find(|(order, ..)| order == parked) {
                *deadline = *ended + (*deadline - *end);
            }
        }
        table.sleepers = BinaryHeap::from(sleepers);
    }

    /// Readies `waiter`, parked on a semaphore, where it is still parked,
    /// answering whether it was: an untimed one, or a timed one still asleep.
    pub(crate) fn wake_semaphore_waiter(&mut self, waiter: crate::semaphores::Waiter) -> bool {
        if let Some(order) = waiter.order {
            let table = &mut self.activities;
            let mut sleepers = std::mem::take(&mut table.sleepers).into_vec();
            let before = sleepers.len();
            sleepers.retain(|Reverse((_, parked, _))| *parked != order);
            let asleep = sleepers.len() < before;
            table.sleepers = BinaryHeap::from(sleepers);
            if !asleep {
                return false;
            }
        }
        self.unpark(waiter.activity);
        true
    }

    /// The driver exit of the running activity, where its driver has just
    /// parked it for a native call.
    fn exit_if_awaited(&mut self) {
        if self
            .activity
            .native_call
            .as_ref()
            .is_some_and(|call| call.awaits_exit())
        {
            self.exit_for_native();
        }
    }

    /// Handles this worker's recalls at its launch point, independent of
    /// when the host runs it. Its answer alone is delivered later.
    fn await_simulated_native(&mut self, inbox: &crate::timer::Inbox<Posted>) {
        loop {
            for post in inbox.idle() {
                match post {
                    Posted::Recall(recall) => self.serve_recall(recall, DrainAt::Stopped),
                    Posted::Completed(completion) => {
                        self.sim_stage_native(completion);
                        return;
                    }
                    Posted::Panicked(payload) => std::panic::resume_unwind(payload),
                    _ => unreachable!("a simulated worker only recalls, completes or panics"),
                }
            }
        }
    }

    /// Serves what the calls still in flight post once the run has ended --
    /// a pool thread's recall to end its call, a completion, a command's
    /// end -- until none is in flight, then drops the interpreter.
    pub(crate) fn wind_down(mut self) {
        // A socket wait's thread ends only when told to or when its sockets
        // are ready; nothing will take up its answer now.
        #[cfg(unix)]
        crate::builtin::rxsock::cancel_helper(&self.activity);
        #[cfg(unix)]
        for idle in self.activities.idle.iter().flatten() {
            crate::builtin::rxsock::cancel_helper(&idle.activity);
        }
        self.sim_deliver_native(Some(u64::MAX));
        while self.activities.in_flight() {
            let posted = self.timer.idle();
            self.file_completions(posted, DrainAt::Stopped);
        }
    }

    /// Files the completions posted since the last drain.
    pub(crate) fn drain_completions(&mut self, at: DrainAt) {
        let posted = self.timer.drain();
        self.file_completions(posted, at);
    }

    /// Hands each completion of `posted` to the parked activity that made its
    /// call and readies that activity, and lends the baton to each thread
    /// that recalls it. What follows a recall goes back to the inbox before
    /// the lend, where a loop the lendee nests can drain it.
    pub(crate) fn file_completions(&mut self, posted: VecDeque<Posted>, at: DrainAt) {
        let hold = self.holds_callbacks();
        self.file_posts(posted, at, hold);
    }

    /// Whether a callback that arrives now is held until the innermost
    /// lend's callback returns or waits: that callback runs, so one served on
    /// top of it would stop it between two of its clauses, holding what it
    /// holds (`r[native.threads.attach.deviation.178]`).
    fn holds_callbacks(&self) -> bool {
        self.activities
            .lends
            .last()
            .is_some_and(|lend| !lend.waiting)
    }

    /// Marks the innermost lend's callback, where it is the running activity,
    /// as waiting or not, answering what it was.
    fn set_callback_waiting(&mut self, waiting: bool) -> Option<bool> {
        let running = self.activities.running;
        let lend = (self.activities.lends.last_mut()).filter(|lend| lend.lendee == running)?;
        Some(std::mem::replace(&mut lend.waiting, waiting))
    }

    /// Starts watching a pinned wait for `reason` of the running activity,
    /// where it is an attached activity's callback and only an activity its
    /// lend stopped may end it, or where waits are not refused as unending
    /// ([`Interp::refuses_unending_waits`]) and it may turn out to be an
    /// inverted wait. Answers `Some` with the watch it replaced, and `None`
    /// where it left the watch in place. Waits that end by themselves, a
    /// sleep's or a timed one's, are not watched, and a wait that is not a
    /// callback's leaves a callback's watch in place.
    fn watch_stall(&mut self, reason: ParkReason) -> Option<Option<Stall>> {
        let previous = self.activities.stall;
        let endless = match reason {
            ParkReason::Guard(_)
            | ParkReason::GuardWhen
            | ParkReason::MessageResult(_)
            | ParkReason::MessageWait(_) => true,
            ParkReason::Semaphore(wait) => wait.deadline.is_none(),
            _ => false,
        };
        let running = self.activities.running;
        let lend = (self.activities.lends.iter().rev()).find(|lend| lend.lendee == running);
        // An instance's root call waits until `Terminate` whatever the lend
        // does, so the callback does not stop anything there that was making
        // progress, and a line naming it would point at nothing frozen.
        let root = self.root_call_in_flight().map(|(activity, _)| activity);
        if let (true, true, Some(lend)) = (endless, self.activity.attached, lend)
            && root != Some(lend.lender)
        {
            let lender = lend.lender;
            self.activities.stall = Some(Stall {
                due: self.now() + STALL_REPORT,
                reason,
                lender: Some(lender),
            });
            Some(previous)
        } else if endless
            && !self.refuses_unending_waits()
            && previous.is_none_or(|stall| stall.lender.is_none())
        {
            // An inverted wait goes on where native code from outside may
            // end it (Deviation 14). Whether it is inverted shows only once
            // the loop has nothing to run, so the watch counts only then.
            // A callback's watch stays: the wait made in its loop does not
            // end it, and the callback's line must still come.
            self.activities.stall = Some(Stall {
                due: self.now() + STALL_REPORT,
                reason,
                lender: None,
            });
            Some(previous)
        } else {
            None
        }
    }

    /// The watched wait of the loop at `level`, where it counts: a
    /// callback's wait, or a pinned wait that is inverted there, with nothing
    /// but a stopped activity left that could end it.
    fn counted_stall(&self, level: usize) -> Option<Stall> {
        (self.activities.stall).filter(|stall| {
            (stall.lender.is_some() || self.stopped_below(level).is_some())
                && self.only_stopped_activities_remain()
        })
    }

    /// The activity the loop at `level` keeps from running where a wait in
    /// it is inverted, as [`Interp::blocked`] counts it: one a recall
    /// interrupted below the loop, the innermost, else one set aside. An
    /// embedding instance's root activity is not one.
    fn stopped_below(&self, level: usize) -> Option<ActivityId> {
        let table = &self.activities;
        // The root's call waits until `Terminate` whatever the loop does, as
        // in [`Interp::watch_stall`], so a line naming it would point at
        // nothing frozen.
        let root = self.root_call_in_flight().map(|(activity, _)| activity);
        (table.owners[..level].iter().rev())
            .find(|&&(_, below)| below == Below::Interrupted)
            .map(|&(owner, _)| owner)
            .or_else(|| table.set_aside.first().map(|&(activity, _)| activity))
            .filter(|&activity| Some(activity) != root)
    }

    /// Writes one line on stderr for the watched wait of the loop at `level`
    /// once it is due and counts ([`Interp::counted_stall`]), naming what it
    /// waits for and the activity it stopped, and stops watching it. The
    /// wait goes on.
    fn report_stall(&mut self, level: usize) {
        let Some(stall) = self.counted_stall(level) else {
            return;
        };
        if self.now() < stall.due {
            return;
        }
        let Some(lender) = stall.lender.or_else(|| self.stopped_below(level)) else {
            return;
        };
        self.activities.stall = None;
        let stopped = if lender == MAIN {
            "the main program".to_owned()
        } else {
            format!("activity {}", lender.0)
        };
        // The time the wait has gone on, which is past the bound where
        // something else that could end it, such as a call in flight, did
        // not end it in time.
        let began = stall.due.checked_sub(STALL_REPORT).unwrap_or(stall.due);
        let waited = self.now().saturating_duration_since(began).as_secs();
        // A callback stops the activity its lend interrupted. A pinned wait
        // that is not a callback's keeps the activity set aside below it from
        // running.
        let line = if stall.lender.is_some()
            || (self.activities.owners[..level].iter())
                .any(|&(owner, below)| owner == lender && below == Below::Interrupted)
        {
            format!(
                "interrexx: a callback has waited {waited} s for {} while {stopped}, which it \
                 stopped, makes no progress\n",
                stall.reason.waits_for(),
            )
        } else {
            format!(
                "interrexx: a pinned wait has waited {waited} s for {} while {stopped}, which \
                 it keeps from running, makes no progress\n",
                stall.reason.waits_for(),
            )
        };
        self.write_err(line.as_bytes());
    }

    /// Whether nothing the loop could run or wait for remains: no activity
    /// ready, none asleep but one a lend stopped, no callback held or to be
    /// sent again, and no call in flight but those a lend's callback runs
    /// above. Only an
    /// activity a lend stopped, or a callback that is yet to arrive, can then
    /// end a wait.
    fn only_stopped_activities_remain(&self) -> bool {
        let table = &self.activities;
        // A stopped activity's sleep ends with nothing it can do: the
        // activity goes on only once the callback that stopped it has
        // finished.
        let stopped = |sleeper: ActivityId| {
            (table.owners.iter()).any(|&(owner, below)| owner == sleeper && below != Below::Free)
        };
        table.ready.is_empty()
            && (table.sleepers.iter()).all(|Reverse((_, _, sleeper))| stopped(*sleeper))
            && table.nothing_held_or_in_flight()
    }

    /// Serves the held callbacks, the earliest first, each once nothing
    /// holds it any more: one waiting for a guard lock once a release has
    /// granted it the lock, in the turn its wait took in the lock's queue.
    /// `drain` is where the drain serving them finds the running
    /// activity.
    fn serve_held(&mut self, drain: DrainAt) {
        let mut at = 0;
        while at < self.activities.held.len() && !self.holds_callbacks() {
            let granted = match self.activities.held[at].1 {
                Some((key, held)) => self.activities.guards.owner(key) == Some(held),
                None => true,
            };
            if !granted {
                at += 1;
                continue;
            }
            let Some((recall, waited)) = self.activities.held.remove(at) else {
                break;
            };
            self.serve_recall(recall, drain);
            // A send that did not reserve the lock it was granted, as one
            // refused once the interpreter has ended, passes it on.
            if let Some((key, held)) = waited
                && self.activities.guards.granted_unreserved(key, held)
                && let Some(next) = self.activities.guards.pass_unreserved(key)
            {
                self.grant_guard(key, next);
            }
            at = 0;
        }
    }

    /// Holds `recall`, queued for the guard lock of `retrying` where its
    /// callback's send gave the baton back to be sent again.
    fn file_held(&mut self, recall: Recall, retrying: Option<(usize, crate::guards::GuardKey)>) {
        match retrying {
            Some((address, key)) => self.queue_held_retry(recall, address, key),
            None => self.activities.held.push_back((recall, None)),
        }
    }

    /// Files `recall`, the next recall of a callback whose send gave the
    /// baton back to be sent again once it is granted `key`
    /// ([`Interp::retries_entry_guard`]): its attached activity has kept its
    /// place in `key`'s queue, or been granted the lock from it, and the
    /// recall is held until the lock is granted.
    fn queue_held_retry(&mut self, recall: Recall, address: usize, key: crate::guards::GuardKey) {
        let waited = self.context_owner(address).map(|held| {
            self.activities.guards.queue_retry(key, held, None);
            (key, held)
        });
        self.activities.held.push_back((recall, waited));
    }

    /// Ends the guard waits for `key` of the callbacks whose lends stopped
    /// `granted`, which a release has just made its owner: `granted` runs
    /// only once those callbacks have ended, so their waits would never end.
    /// A wait ended so keeps its place in the lock's queue until it ends, and
    /// is sent again from that place or refused by its caller, as one that
    /// found the lock held so at its start is (Deviation 178). A lender that
    /// is an attached activity's callback leaves the waits as they are, as
    /// at their start.
    pub(crate) fn stop_callbacks_behind(
        &mut self,
        key: crate::guards::GuardKey,
        granted: ActivityId,
    ) {
        let lends = &self.activities.lends;
        let Some(first) = lends.iter().position(|lend| lend.lender == granted) else {
            return;
        };
        if self.record(granted).is_some_and(|record| record.attached) {
            return;
        }
        let stopped: Vec<ActivityId> = (lends[first..].iter())
            .map(|lend| lend.lendee)
            .filter(|lendee| {
                self.activities.guards.waiting(*lendee) == Some(crate::guards::Waiting::Guard(key))
            })
            .collect();
        for lendee in stopped {
            self.activities.guards.keep_place(lendee);
            if let Some(record) = self.record_of(lendee) {
                record.lock_to_lender = true;
            }
            self.unpark(lendee);
        }
    }

    /// [`Interp::file_completions`], holding a callback's recall where
    /// `hold`, and serving the held ones first where not.
    fn file_posts(&mut self, mut posted: VecDeque<Posted>, at: DrainAt, hold: bool) {
        if self.sim.is_some() {
            self.sim_screen_posts(&posted);
        }
        if !hold && !self.activities.held.is_empty() {
            self.timer.requeue(std::mem::take(&mut posted));
            self.serve_held(at);
            posted = self.timer.drain();
        }
        while let Some(next) = posted.pop_front() {
            match next {
                Posted::Completed(completion) => self.file_native_completion(completion),
                Posted::Recall(recall) => {
                    let context = match recall.by {
                        Recaller::Context(address) => Some(address),
                        Recaller::Call { .. } => None,
                    };
                    let retrying = (self.activities.retrying.iter())
                        .position(|(held, _)| Some(*held) == context)
                        .map(|at| self.activities.retrying.swap_remove(at));
                    if hold && context.is_some() {
                        self.file_held(recall, retrying);
                        continue;
                    }
                    self.timer.requeue(std::mem::take(&mut posted));
                    if hold {
                        self.serve_recall(recall, at);
                    } else {
                        self.file_held(recall, retrying);
                        self.serve_held(at);
                    }
                    posted = self.timer.drain();
                    if self.sim.is_some() {
                        self.sim_screen_posts(&posted);
                    }
                }
                Posted::Output {
                    error: false,
                    bytes,
                } => self.write_out(&bytes),
                Posted::Output { error: true, bytes } => self.write_err(&bytes),
                Posted::Captured {
                    activity,
                    token,
                    bytes,
                } => {
                    // Recorded for the wait wherever it is; the activity is
                    // woken only where it is parked on that wait, since any
                    // other park it is in keeps its own wake.
                    let woken = self.record_of(activity).is_some_and(|record| {
                        record.blocked.capture(token, bytes) && record.parked_block == Some(token)
                    });
                    if woken {
                        self.unpark(activity);
                    }
                }
                #[cfg(unix)]
                Posted::SocketWaited { activity, token } => {
                    self.activities.in_flight -= 1;
                    let woken = self.record_of(activity).is_some_and(|record| {
                        crate::builtin::rxsock::helper_ended(record, token);
                        let parked = record.parked_socket == Some(token);
                        if parked {
                            record.parked_socket = None;
                        }
                        parked
                    });
                    if woken {
                        self.unpark(activity);
                    }
                }
                Posted::Panicked(payload) => std::panic::resume_unwind(payload),
                Posted::Halt => {
                    self.timer.requests().clear(crate::timer::HALT);
                    self.halt_all();
                }
                Posted::Trace(on) => self.set_external_trace(on),
                Posted::TraceAll(on) => self.set_external_trace_all(on),
                Posted::Input(chunk) => self.input.receive(chunk),
                Posted::Returned { activity, frame } => self.vacancy_returned(activity, frame),
                Posted::Unblocked {
                    activity,
                    token,
                    ended,
                } => {
                    self.serve_signal_now();
                    let abandoned = &mut self.activities.abandoned_blocks;
                    if let Some(at) = abandoned.iter().position(|held| *held == token) {
                        abandoned.swap_remove(at);
                        continue;
                    }
                    self.activities.in_flight -= 1;
                    let woken = self.record_of(activity).is_some_and(|record| {
                        record.blocked.end(token, ended) && record.parked_block == Some(token)
                    });
                    if woken {
                        self.unpark(activity);
                    }
                }
            }
        }
    }

    /// Records a call's completion and readies its activity, or drops the
    /// call's frame if its activity no longer waits for it.
    pub(crate) fn file_native_completion(&mut self, completed: Completed) {
        let Completed {
            activity,
            frame,
            completion,
        } = completed;
        self.activities.in_flight -= 1;
        if let Some(call) = self
            .record_of(activity)
            .and_then(|record| record.native_call.as_mut())
            .filter(|call| call.frame() == frame)
        {
            call.complete(completion);
            self.unpark(activity);
        } else {
            self.end_abandoned_call(frame);
        }
    }

    /// `activity`'s record, running or idle, or `None` for a handle no
    /// activity has.
    fn record_of(&mut self, activity: ActivityId) -> Option<&mut Activity> {
        if activity == self.activities.running {
            return Some(&mut self.activity);
        }
        self.activities
            .idle
            .get_mut(activity.0 as usize)
            .and_then(Option::as_mut)
            .map(|idle| &mut idle.activity)
    }

    /// Lends the baton to the thread `recall` names, with the activity it
    /// calls back for as the running one, its frames pinned, and the stack
    /// measured from that thread's: the callback or the call's end runs above
    /// the activity that ran last, whose frames stay below. A thread running
    /// no native call of this interpreter's gets no stack to run Rexx code
    /// on. `at` says whether the activity that ran last is in the middle of
    /// its run.
    fn serve_recall(&mut self, recall: Recall, at: DrainAt) {
        let Recall {
            by,
            thread,
            base,
            room,
        } = recall;
        // Once the interpreter has terminated, a callback is refused
        // (`r[native.threads.end]`): its thread is lent the baton only to
        // find that out and give it straight back.
        if matches!(by, Recaller::Context(address) if address != 0)
            && self.timer.inbox().is_ending()
        {
            let baton = crate::sync::Arc::clone(&self.baton);
            baton.lend(thread, crate::island::Island::of(self));
            return;
        }
        let outgoing = self.activities.running;
        // An embedding instance's root call counts as run below for every
        // lend, whoever calls back: only `Terminate` ends it, and its end
        // can satisfy no wait a callback's loop refuses, a wait on the
        // waiting activity's own message or an inverted one. A call a lend
        // further down counts already is not counted again, so nested lends
        // count each call once.
        let below: Vec<_> = [
            self.recalled_call_in_flight(&by),
            self.root_call_in_flight(),
        ]
        .into_iter()
        .flatten()
        .collect();
        let callback = matches!(by, Recaller::Context(_));
        let activity = match by {
            Recaller::Call { activity, frame } => {
                self.activities
                    .idle
                    .get(activity.0 as usize)
                    .and_then(Option::as_ref)
                    .is_some_and(|idle| {
                        idle.activity
                            .native_call
                            .as_ref()
                            .is_some_and(|call| call.frame() == frame)
                            || self.activities.vacancies.iter().any(|vacancy| {
                                vacancy.activity == activity && vacancy.frame == frame
                            })
                    })
                    .then_some(activity)
            }
            Recaller::Context(address) => self
                .context_owner(address)
                .or_else(|| self.file_attached(address)),
        }
        .filter(|activity| *activity != outgoing);
        if let Some(activity) = activity {
            self.swap_running(activity, outgoing);
            let below = match at {
                DrainAt::MidRun => Below::Interrupted,
                DrainAt::Stopped => Below::Buried,
            };
            self.activities.owners.push((outgoing, below));
            self.activities.lends.push(Lend {
                lender: outgoing,
                lendee: activity,
                waiting: false,
            });
        }
        // An attached activity runs nothing between its callbacks, so what a
        // callback rooted as a temporary is dropped once it gives the baton
        // back. The references it made are in the base frame's table.
        let temps = (self.activity.attached).then(|| self.roots.activity_mut().push_frame());
        pin_enter!(self, crate::pinning::PinKind::NativeApiCallback);
        let counted = self.activities.runs_below.len();
        for call in below {
            if !self.activities.runs_below.contains(&call) {
                self.activities.runs_below.push(call);
            }
        }
        self.activities.callbacks_lent += usize::from(callback);
        let stack = (self.stack_base, self.stack_room);
        self.stack_base = base.unwrap_or(self.stack_base);
        self.stack_room = match (base, room) {
            (Some(_), Some(room)) => room,
            (Some(_), None) => self.pool.stack().saturating_sub(STACK_MARGIN),
            (None, _) => 0,
        };
        let baton = crate::sync::Arc::clone(&self.baton);
        baton.lend(thread, crate::island::Island::of(self));
        (self.stack_base, self.stack_room) = stack;
        self.activities.runs_below.truncate(counted);
        self.activities.callbacks_lent -= usize::from(callback);
        pin_leave!(self);
        if let Some(frame) = temps {
            self.roots.activity_mut().pop_frame(frame);
        }
        if let Some(activity) = activity {
            self.activities.lends.pop();
            self.activities.owners.pop();
            self.swap_running(outgoing, activity);
        }
    }

    /// The call `by` recalls the baton for, by its activity and native
    /// frame, where it counts in flight ([`Activities::in_flight`]), so that
    /// a lend for its callback counts it as run below
    /// ([`Activities::runs_below`]): a pool thread's call, which counts from
    /// its exit, a vacated call while a thread serves its vacancy, or the
    /// call in flight of the activity whose thread context a thread calls
    /// back through, as the thread that created an embedding instance does
    /// for the root's call. An attached thread's callback recalls for no
    /// call.
    fn recalled_call_in_flight(&self, by: &Recaller) -> Option<(ActivityId, u64)> {
        // A lend that counted a call below which is not in flight would hide
        // an unrelated call in flight from the tests that compare the two,
        // and a wait that call may end would be refused.
        match *by {
            Recaller::Call { activity, frame } => {
                let record = if activity == self.activities.running {
                    Some(&self.activity)
                } else {
                    (self.activities.idle.get(activity.0 as usize))
                        .and_then(Option::as_ref)
                        .map(|idle| &idle.activity)
                };
                let pooled = record.is_some_and(|record| {
                    (record.native_call.as_ref()).is_some_and(|call| call.frame() == frame)
                });
                (pooled
                    || self.activities.vacancies.iter().any(|vacancy| {
                        vacancy.activity == activity && vacancy.frame == frame && vacancy.served
                    }))
                .then_some((activity, frame))
            }
            // The callback runs as the activity whose call is out, so that
            // call's completion can resume nothing before the callback has
            // returned.
            Recaller::Context(address) => {
                let (activity, record) = match self.context_owner(address) {
                    Some(owner) => (
                        owner,
                        (self.activities.idle.get(owner.0 as usize))
                            .and_then(Option::as_ref)
                            .map(|idle| &idle.activity),
                    ),
                    None => (
                        self.activities.running,
                        (self.activity.thread.as_ref())
                            .is_some_and(|thread| thread.address() == address)
                            .then_some(&self.activity),
                    ),
                };
                let call = record?.native_call.as_ref()?;
                call.left().then_some((activity, call.frame()))
            }
        }
    }

    /// The embedding instance's root call, by its activity and native frame,
    /// while it is in flight ([`Activities::in_flight`]).
    fn root_call_in_flight(&self) -> Option<(ActivityId, u64)> {
        let (activity, frame) = self.activities.root_call?;
        let record = if activity == self.activities.running {
            Some(&self.activity)
        } else {
            (self.activities.idle.get(activity.0 as usize))
                .and_then(Option::as_ref)
                .map(|idle| &idle.activity)
        };
        let call = record?.native_call.as_ref()?;
        (call.frame() == frame && call.left()).then_some((activity, frame))
    }

    /// Notes that the running activity's native call, in its native frame
    /// `frame`, is the embedding instance's root call, which waits until
    /// `Terminate`.
    pub(crate) fn note_root_call(&mut self, frame: u64) {
        self.activities.root_call = Some((self.activities.running, frame));
    }

    /// Ends the embedding instance's root call where the running activity's
    /// native call in its native frame `frame` is that call, once its
    /// completion is drained or abandoned.
    pub(crate) fn end_root_call(&mut self, frame: u64) {
        if self.activities.root_call == Some((self.activities.running, frame)) {
            self.activities.root_call = None;
        }
    }

    /// Marks the running activity as a callback that runs, as a lend to it
    /// does, where `running`, and ends that where not.
    #[cfg(test)]
    pub(crate) fn pretend_callback_runs(&mut self, running: bool) {
        if running {
            let activity = self.activities.running;
            self.activities.lends.push(Lend {
                lender: activity,
                lendee: activity,
                waiting: false,
            });
        } else {
            self.activities.lends.pop();
        }
    }

    /// The address of a context `AttachThread` would have built for a foreign
    /// thread, pending until a callback through it files it.
    #[cfg(test)]
    pub(crate) fn pending_attached_context(&self) -> Option<usize> {
        self.activities.thread_table.build_attached_context()
    }

    /// The attached activities filed and not yet ended.
    pub(crate) fn attached_count(&self) -> usize {
        self.activities.attached
    }

    /// The first callback through the context at `address` that `AttachThread`
    /// built for a thread with no activity of this interpreter
    /// (`r[native.threads.attach]`): files the attached activity whose context
    /// it is, idle until a callback through it runs it, with a base native
    /// frame for what those callbacks make, and answers its handle. `None`
    /// where `address` is no such context. As a spawn does, it makes guards
    /// live and numbers the running activity.
    fn file_attached(&mut self, address: usize) -> Option<ActivityId> {
        let thread = self.activities.thread_table.take_pending(address)?;
        self.guards_go_live(None);
        self.activity_number();
        let mut idle = self.new_activity();
        idle.activity.thread = Some(thread);
        idle.activity.attached = true;
        let table = &mut self.activities;
        let id = match table.free.pop() {
            Some(index) => ActivityId(index),
            None => {
                table.idle.push(None);
                ActivityId(u32::try_from(table.idle.len() - 1).expect("handles fit u32"))
            }
        };
        table.idle[id.index()] = Some(idle);
        table.attached += 1;
        self.with_idle_activity(id, Interp::push_attached_frame);
        Some(id)
    }

    /// The last `DetachThread` of the attached activity whose thread context
    /// is at `context`, idle: its base frame's references are released, and
    /// its handle and number are freed, and the answer is true. The context,
    /// which the thread holds the address of, is kept, as `AttachThread`
    /// built it, and this interpreter holds no clone of it afterwards.
    /// Anything else, an activity running or not attached, is left as it is.
    pub(crate) fn detach_activity(&mut self, context: usize) -> bool {
        let Some(id) = self.context_owner(context) else {
            return false;
        };
        let attached = self.activities.idle[id.index()]
            .as_ref()
            .is_some_and(|idle| idle.activity.attached);
        if !attached {
            return false;
        }
        self.with_idle_activity(id, Interp::pop_attached_frame);
        let table = &mut self.activities;
        let Some(idle) = table.idle[id.index()].take() else {
            unreachable!("an attached handle names an idle activity");
        };
        table.ready.retain(|ready| *ready != id);
        table.free.push(id.0);
        if table.pooled.len() <= MAX_POOLED {
            table.pooled.push_back(idle.activity.number);
        }
        table.attached -= 1;
        true
    }

    /// Starts a pool thread that serves the innermost vacancy, which the
    /// calling thread has claimed from a native call running with the baton
    /// vacated: it is handed the vacancy, runs the other activities and
    /// serves the recalls posted meanwhile, until the call returns
    /// ([`Interp::serve_vacancy`]). Answers the thread, or `None` where the
    /// pool has none to give, so that the vacating thread serves what was
    /// posted once it has the baton back.
    pub(crate) fn start_vacancy_server(&self) -> Option<std::thread::ThreadId> {
        let worker = self.pool.reserve().ok().flatten()?;
        let server = worker.thread();
        let baton = crate::sync::Arc::clone(&self.baton);
        let inbox = self.timer.inbox();
        worker.run(Box::new(move || {
            pool::posting_panics(&inbox, &baton, || {
                let probe = 0u8;
                let mut lent = crate::island::Lent::wait(&baton);
                lent.interp().serve_vacancy(&raw const probe as usize);
            });
        }));
        Some(server)
    }

    /// On the thread the innermost vacancy was handed to, `base` the address
    /// of a local near its stack's base: runs the other activities, with the
    /// one whose call vacated the baton buried below, since its frames are on
    /// the vacating thread's stack, and serves what is posted, until that
    /// call has returned. The call counts in flight meanwhile, so a loop
    /// idles for its return rather than refusing a wait. A failure the other
    /// activities answer that is not a condition is kept for the program's
    /// end, and only posts are served after it.
    pub(crate) fn serve_vacancy(&mut self, base: usize) {
        let Some(at) = self.activities.vacancies.len().checked_sub(1) else {
            return;
        };
        let me = self.activities.running;
        debug_assert!(
            self.activities.vacancies[at].activity == me,
            "a vacancy served with another activity running than its call's"
        );
        self.activities.vacancies[at].served = true;
        self.activities.in_flight += 1;
        let stack = (self.stack_base, self.stack_room);
        self.stack_base = base;
        self.stack_room = self.pool.stack().saturating_sub(STACK_MARGIN);
        let mut running_others = true;
        while !self.activities.vacancies[at].returned {
            if running_others {
                match self.run_others(true) {
                    Ok(Waited::Woken) => continue,
                    Ok(Waited::Blocked { .. }) => {}
                    Err(failure) => {
                        self.keep_late_failure(failure);
                        running_others = false;
                    }
                }
            }
            if !self.activities.vacancies[at].returned {
                let posted = self.timer.idle();
                self.file_completions(posted, DrainAt::Stopped);
            }
        }
        // Readied by the return, where no loop took it.
        self.activities.ready.retain(|ready| *ready != me);
        (self.stack_base, self.stack_room) = stack;
    }

    /// The return of `activity`'s native call in the native frame `frame`,
    /// which ran with the baton vacated: the thread serving the vacancy
    /// stops. A return posted for a vacancy nobody served, or one already
    /// reclaimed, changes nothing.
    fn vacancy_returned(&mut self, activity: ActivityId, frame: u64) {
        let Some(vacancy) = (self.activities.vacancies.iter_mut())
            .find(|vacancy| vacancy.activity == activity && vacancy.frame == frame)
        else {
            return;
        };
        vacancy.returned = true;
        if vacancy.served {
            self.activities.in_flight -= 1;
            self.activities.make_ready(activity);
        }
    }

    /// Records the native call of the running activity in the native frame
    /// `frame` as one about to run with the baton vacated.
    pub(crate) fn push_vacancy(&mut self, frame: u64) {
        #[cfg(test)]
        VACATES.with(|vacates| vacates.set(vacates.get() + 1));
        let activity = self.activities.running;
        let waiting = self.set_callback_waiting(true);
        self.activities.vacancies.push(Vacancy {
            activity,
            frame,
            served: false,
            returned: false,
            waiting,
        });
    }

    /// Drops the record [`Interp::push_vacancy`] made, once the call has
    /// finished on the baton.
    pub(crate) fn pop_vacancy(&mut self, frame: u64) {
        let popped = self.activities.vacancies.pop();
        debug_assert!(
            popped
                .as_ref()
                .is_some_and(|vacancy| vacancy.frame == frame),
            "a vacancy dropped out of order"
        );
        if let Some(waiting) = popped.and_then(|vacancy| vacancy.waiting) {
            self.set_callback_waiting(waiting);
        }
    }

    /// The idle activity whose thread context is at `address`.
    fn context_owner(&self, address: usize) -> Option<ActivityId> {
        let index = self.activities.idle.iter().position(|idle| {
            idle.as_ref()
                .and_then(|idle| idle.activity.thread.as_ref())
                .is_some_and(|thread| thread.address() == address)
        })?;
        Some(ActivityId(u32::try_from(index).expect("handles fit u32")))
    }

    /// Makes the idle `next` the running activity in place of `running`,
    /// whose record goes under its own handle, as [`Interp::switch_to`] does
    /// without a switch's other work.
    fn swap_running(&mut self, next: ActivityId, running: ActivityId) {
        let table = &mut self.activities;
        let Some(mut idle) = table.idle[next.0 as usize].take() else {
            unreachable!("a recalled handle names an idle activity");
        };
        std::mem::swap(&mut self.activity, &mut idle.activity);
        std::mem::swap(self.roots.activity_mut(), &mut idle.roots);
        // The key a compound read holds is the outgoing activity's, and the
        // incoming one may be in the middle of a clause of its own: one
        // waiting off the baton for a native call, which pushed nothing.
        self.shared_key_for = None;
        #[cfg(feature = "sharing")]
        self.heap.share_as(self.activity.sharing_tag);
        table.idle[running.0 as usize] = Some(idle);
        table.running = next;
    }

    /// Whether an activity other than the running one is alive.
    pub(crate) fn others_live(&self) -> bool {
        self.activities.idle.len() - 1 > self.activities.free.len()
    }

    /// Moves every sleeper whose deadline is due to the ready queue, in
    /// deadline order.
    fn wake_due_sleepers(&mut self) {
        let now = self.now();
        let table = &mut self.activities;
        while let Some(Reverse((deadline, _, sleeper))) = table.sleepers.peek().copied() {
            if deadline > now {
                break;
            }
            table.sleepers.pop();
            table.make_ready(sleeper);
        }
    }

    /// [`Interp::run_started_activities`] until it has nothing left to run or
    /// the run's deadline has passed, with each failure it answered on the
    /// way, in order.
    pub(crate) fn run_started_to_end(&mut self) -> Vec<(Failure, Vec<crate::FailureSite>)> {
        let mut failures = self.take_late_failures();
        while let Err(failure) = self.run_started_activities() {
            let last = matches!(failure, Failure::Deadline)
                || self.sim_is_stuck()
                || std::mem::take(&mut self.activities.ended_stuck);
            failures.push((failure, Vec::new()));
            if last {
                break;
            }
        }
        failures
    }

    /// The `UNINIT`s a collection readied, run on an activity whose dispatch
    /// has ended or whose activation returns. The first refusal they meet is
    /// answered; the later ones are kept for the program's end, in order.
    pub(crate) fn run_ending_uninits(&mut self) -> Result<(), Failure> {
        let mut refused = self.run_ready_uninits().into_iter();
        let Some(first) = refused.next() else {
            return Ok(());
        };
        for loud in refused {
            self.keep_late_failure(loud.into());
        }
        Err(first.into())
    }

    /// The failures kept for the program's end, in order.
    pub(crate) fn take_late_failures(&mut self) -> Vec<(Failure, Vec<crate::FailureSite>)> {
        std::mem::take(&mut self.activities.late_failures)
            .into_iter()
            .map(|failure| (failure, Vec::new()))
            .collect()
    }

    /// A failure another activity answered main's root loop with after
    /// main's own end, kept for the program's end to report.
    pub(crate) fn keep_late_failure(&mut self, failure: Failure) {
        self.activities.late_failures.push(failure);
    }

    /// Runs every started activity to its end, for the end of the program.
    /// Where the activities that remain wait for something nothing left can
    /// bring about, the wait is refused as it is while the main program runs
    /// (`r[conc.halt.end.alive]`, Deviation 20), and the program ends without
    /// waiting for them (`r[conc.halt.deviation.21]`). Once waits are no
    /// longer refused as unending, the end waits for them instead.
    pub(crate) fn run_started_activities(&mut self) -> Result<(), Failure> {
        self.activities.main_finished = true;
        self.cancel_wait();
        self.activities.end_level = Some(self.activities.owners.len());
        let ended = self.run_to_end();
        self.activities.end_level = None;
        ended
    }

    /// [`Interp::run_started_activities`]' loop. Where the activities that
    /// remain wait for something nothing left can bring about and waits are
    /// not refused as unending ([`Interp::refuses_unending_waits`]), it
    /// waits for a post and runs them again, unless one of them waits on its
    /// own message ([`Interp::waits_on_its_own_message`]), which is refused.
    fn run_to_end(&mut self) -> Result<(), Failure> {
        loop {
            self.run_others(false)?;
            let table = &self.activities;
            let me = table.running.0 as usize;
            // An attached activity idle between its callbacks does not hold
            // the end (`r[native.threads.end]`). One running a callback holds
            // the baton, so the end comes after it.
            // One parked inside a callback holds it too: the callback's lend
            // lasts until the callback returns, and this loop goes on only
            // once the lend has ended, so an attached activity it finds here
            // is between callbacks.
            let others = (table.idle.iter().enumerate()).any(|(index, idle)| {
                index != me && idle.as_ref().is_some_and(|idle| !idle.activity.attached)
            });
            if !others {
                return Ok(());
            }
            // A held callback can run, so it keeps the end from refusing.
            if table.ready.is_empty()
                && table.sleepers.is_empty()
                && table.held.is_empty()
                && table.in_flight == table.runs_below.len()
            {
                if self.sim.is_some() {
                    return Err(self.sim_endless_wait());
                }
                if self.refuses_unending_waits() || self.waits_on_its_own_message() {
                    self.activities.ended_stuck = true;
                    return Err(Loud::unsatisfiable_wait().into());
                }
                // A thread an external native library started may attach
                // and end the wait, so the end waits for it as ooRexx does
                // (Deviation 20).
                self.idle_for_posts()?;
            }
        }
    }

    /// Runs ready activities with the running one set aside, until it is
    /// ready again, it has ended as main ends in another loop, or nothing can
    /// run. It is running again on return. The running activity is `buried`
    /// where it waits from Rust frames below this loop; a buried activity is
    /// not run by a loop above its own: it is set aside and goes back to the
    /// ready queue for its own loop when this one returns. An activity whose
    /// state is all in its record runs in any loop.
    fn run_others(&mut self, buried: bool) -> Result<Waited, Failure> {
        self.run_round(buried).map(|(waited, _)| waited)
    }

    /// [`Interp::run_others`], answering besides whether the loop set aside
    /// a buried activity that was ready.
    fn run_round(&mut self, buried: bool) -> Result<(Waited, bool), Failure> {
        let me = self.activities.running;
        let level = self.activities.owners.len();
        let probe = 0u8;
        if self.stack_exhausted(&raw const probe) {
            self.activities.ready.retain(|ready| *ready != me);
            return Err(Raised::insufficient_stack().into());
        }
        let below = if buried { Below::Buried } else { Below::Free };
        self.activities.owners.push((me, below));
        let waited = self.run_others_from(me, level);
        let table = &mut self.activities;
        let mine = table
            .set_aside
            .iter()
            .position(|(_, by)| *by >= level)
            .unwrap_or(table.set_aside.len());
        let set_aside = mine < table.set_aside.len();
        for (activity, _) in table.set_aside.drain(mine..).rev() {
            table.ready.push_front(activity);
        }
        table.owners.pop();
        waited.map(|waited| (waited, set_aside))
    }

    /// [`Interp::run_others`]' loop, the one at `level` of the stack.
    fn run_others_from(&mut self, me: ActivityId, level: usize) -> Result<Waited, Failure> {
        let Some(next) = self.next_runnable(me, level)? else {
            return Ok(self.blocked(level));
        };
        if next == me {
            return Ok(Waited::Woken);
        }
        self.switch_to(next, false);
        loop {
            let parked = self.run_until_park();
            let ended = !matches!(parked, Ok(true));
            if let Err(failure) = parked {
                self.activities.ready.retain(|ready| *ready != me);
                self.switch_to(me, ended);
                return Err(failure);
            }
            if self.root_ended(me) {
                self.switch_to(me, ended);
                return Ok(Waited::Woken);
            }
            let woken = match self.next_runnable(me, level) {
                Ok(woken) => woken,
                Err(failure) => {
                    self.activities.ready.retain(|ready| *ready != me);
                    self.switch_to(me, ended);
                    return Err(failure);
                }
            };
            match woken {
                // The activity whose slice ended, with nothing else this loop
                // can run, runs on.
                Some(ready) if ready == self.activities.running => {}
                Some(ready) if ready != me => self.switch_to(ready, ended),
                woken => {
                    self.switch_to(me, ended);
                    return Ok(match woken {
                        Some(_) => Waited::Woken,
                        None => self.blocked(level),
                    });
                }
            }
        }
    }

    /// The loop at `level` has nothing to run: inverted where some activity
    /// is ready whose frames lie below a loop on the stack, which only that
    /// loop can run, or where a recall below this loop interrupted an
    /// activity, which runs on only once that recall's lend returns, and the
    /// running activity does not wait on a message it runs itself.
    ///
    /// Both are a best guess at what the blocked wait is for: neither asks
    /// whether the activity below could end it. Any interrupted owner below
    /// makes the wait inverted, even one nothing could end, as any activity
    /// set aside does; telling the two apart would need to know which
    /// activity each wait's end depends on, which the loop does not track.
    /// The refusal and exit code are the same either way, only the text and
    /// the inverted-wait count differ.
    fn blocked(&self, level: usize) -> Waited {
        let table = &self.activities;
        let interrupted = table.owners[..level]
            .iter()
            .any(|&(_, below)| below == Below::Interrupted);
        // An activity set aside or interrupted cannot end a wait on a message
        // the running activity runs itself either, so that wait is not
        // inverted.
        Waited::Blocked {
            inverted: (!table.set_aside.is_empty() || interrupted)
                && !self.waits_on_a_message_it_runs(table.running),
        }
    }

    /// Whether `activity`, not running, is main whose root driver ended in a
    /// loop other than its own.
    fn root_ended(&self, activity: ActivityId) -> bool {
        self.activities
            .idle
            .get(activity.0 as usize)
            .and_then(Option::as_ref)
            .is_some_and(|idle| idle.activity.root_end.is_some())
    }

    /// Whether waits are refused where nothing left to run can end them
    /// (Deviation 20): while no external native library has loaded and no
    /// host outside the interpreter runs the program, and never again in the
    /// run once either holds, also after the library is dropped. Libraries
    /// built into the interpreter do not count.
    pub(crate) fn refuses_unending_waits(&self) -> bool {
        // Only native code from outside can start a thread that attaches
        // later and ends the wait, and dropping a library does not end the
        // threads it started.
        !self.external_native_code
    }

    /// Notes that an external native library has loaded, through
    /// `::REQUIRES ... LIBRARY`, an `EXTERNAL` directive, `RxFuncAdd`, a
    /// classic registration or a library a host registers: waits are no
    /// longer refused as unending.
    pub(crate) fn note_external_library(&mut self) {
        self.external_native_code = true;
    }

    /// Notes that a host outside the interpreter runs the program through
    /// the API, `RexxStart` or `RexxCreateInterpreter`: waits are not refused
    /// as unending.
    pub(crate) fn note_outside_host(&mut self) {
        self.external_native_code = true;
    }

    /// Whether the loop at `level`, with nothing to run and nothing in
    /// flight, waits for a post rather than answering that it is blocked:
    /// where waits are not refused as unending
    /// ([`Interp::refuses_unending_waits`]), an inverted wait above an
    /// activity set aside or interrupted included, except for a wait on the
    /// waiting activity's own message
    /// ([`Interp::waits_on_its_own_message`]), the loop of the program's end
    /// and the simulation mode.
    fn waits_for_attaching_threads(&self, level: usize) -> bool {
        // A thread an external library started may attach later and end the
        // wait (Deviation 20). An inverted wait, one an activity set aside
        // makes or one an activity a recall interrupted below this loop
        // makes, as [`Interp::blocked`] counts them, follows the same switch
        // (Deviation 14): such a thread may end what it waits on too, and
        // the loop below runs once it has. Where it goes on, stderr says so
        // once it has waited a while ([`Interp::report_stall`]). The
        // program's end loop answers that it is blocked, and waits for posts
        // itself only while an activity waits ([`Interp::run_to_end`]). The
        // simulation mode refuses every attach, so nothing can post there. No
        // thread can end an activity's wait on its own message, so the loop
        // refuses it whatever was loaded.
        !self.refuses_unending_waits()
            && self.sim.is_none()
            && !self.waits_on_its_own_message()
            && self.activities.end_level != Some(level)
    }

    /// The next ready activity the loop at `level` can run, or `me`; a buried
    /// activity is set aside. With none ready, this thread idles until the
    /// earliest sleeper is due; the run's deadline passing first fails it.
    fn next_runnable(
        &mut self,
        me: ActivityId,
        level: usize,
    ) -> Result<Option<ActivityId>, Failure> {
        if self.timer.requests().pending(crate::timer::INBOX)
            || (!self.activities.held.is_empty() && !self.holds_callbacks())
        {
            self.drain_completions(DrainAt::Stopped);
        }
        loop {
            if !self.activities.sleepers.is_empty() {
                self.wake_due_sleepers();
            }
            let next = if self.sim.is_some() {
                self.sim_pick()
            } else {
                self.activities.ready.pop_front()
            };
            let Some(ready) = next else {
                let sleeper = (self.activities.sleepers.peek()).map(|Reverse((due, _, _))| *due);
                // While an attached thread's callback runs, a later callback
                // may end a wait, so a loop with nothing else to run or wait
                // for idles for posts instead of refusing the wait
                // (`r[native.threads.attach.deviation.178]`).
                // No callback can end this loop's own wait on a message its
                // activity runs, so that wait is refused even inside a
                // callback, a host's call included (Deviation 20). A callback
                // whose send is to be sent again counts as held before its
                // recall arrives, as its place in a guard lock's queue may
                // be the turn the other waits wait for.
                if sleeper.is_none()
                    && self.activities.nothing_held_or_in_flight()
                    && (self.waits_on_a_message_it_runs(me)
                        || (self.activities.callbacks_lent == 0
                            && !self.waits_for_attaching_threads(level)))
                {
                    return Ok(None);
                }
                // A watched wait is reported only once nothing else could
                // end it, so a sleeper, which may, keeps it unreported. A
                // sleeper a lend stopped may not, so it does not hold the
                // report back to its wake.
                let stall = self.counted_stall(level).map(|stall| stall.due);
                match (sleeper, stall) {
                    (Some(sleeper), Some(stall)) => self.idle_until(sleeper.min(stall))?,
                    (Some(due), None) | (None, Some(due)) => self.idle_until(due)?,
                    (None, None) => self.idle_for_posts()?,
                }
                self.report_stall(level);
                continue;
            };
            let buried = self
                .activities
                .owners
                .iter()
                .any(|&(owner, below)| owner == ready && below != Below::Free);
            if ready == me || !buried {
                return Ok(Some(ready));
            }
            #[cfg(feature = "pinning")]
            if let Some(Some(idle)) = self.activities.idle.get(ready.0 as usize) {
                late_wake!(self, idle.activity);
            }
            self.activities.set_aside.push((ready, level));
        }
    }

    /// The running activity, a started one, run until it parks or ends: its
    /// first step, or the rest of the work its wake left. Its outcome is
    /// recorded on its message; a failure that is not a condition is
    /// answered.
    fn run_started(&mut self) -> Result<Stopped, Failure> {
        if self.activities.running == MAIN {
            self.root_step(None);
            return Ok(Stopped::Parked);
        }
        let sent = match self.activity.first.take() {
            Some(First::Send(send)) => {
                let frame = self.roots.activity_mut().push_frame();
                let mut roots = Vec::new();
                send.object_roots(&mut roots);
                for object in roots {
                    self.roots.activity_mut().push_temp(object);
                }
                let StartedSend {
                    receiver,
                    name,
                    scope,
                    args,
                    caller,
                } = *send;
                let started = self.begin_send(receiver, &name, scope, &args, caller);
                self.roots.activity_mut().pop_frame(frame);
                match started {
                    Ok(Started::Ran(value)) => Ok(value),
                    Err(failure) => Err(failure),
                    Ok(Started::Entered) => match self.activity.native_park.as_ref() {
                        Some(park) => {
                            let reason = park.reason();
                            self.park(reason);
                            return Ok(Stopped::Parked);
                        }
                        None => match self.prepare_level() {
                            Ok(level) => {
                                let driven = self.drive_from(DriveStart::Level(level), true);
                                match self.started_driven(driven) {
                                    Some(sent) => sent,
                                    None => return Ok(Stopped::Parked),
                                }
                            }
                            Err(failure) => self.finish_send(Err(failure)),
                        },
                    },
                }
            }
            first => 'resumed: {
                // A resumed activation announces itself again where it had
                // before the `REPLY`.
                if matches!(first, Some(First::Reply)) {
                    if !self.reserve_for_continuation() {
                        self.activity.first = first;
                        return Ok(Stopped::Parked);
                    }
                    let empty = self.activity.replied_level.is_none();
                    self.open_replied_level();
                    self.trace_invocation_entry();
                    if empty {
                        break 'resumed self.finish_empty_rest();
                    }
                } else if !self.activity.guard_waits.is_empty() {
                    self.take_granted_guard();
                }
                let driven = match self.activity.sliced.take() {
                    Some((floor, at)) => self.drive_from(DriveStart::Sliced { floor, at }, true),
                    None => {
                        if self.parked_again() {
                            return Ok(Stopped::Parked);
                        }
                        let sent = self.resume_parked(None);
                        if let Some(reason) = self.parked_by_resume(&sent) {
                            self.park(reason);
                            return Ok(Stopped::Parked);
                        }
                        let Some(floor) = self.activity.drive_floor.take() else {
                            break 'resumed sent;
                        };
                        self.drive_from(DriveStart::Woken { floor, sent }, true)
                    }
                };
                match self.started_driven(driven) {
                    Some(sent) => sent,
                    None => return Ok(Stopped::Parked),
                }
            }
        };
        let sent = match sent {
            Err(Failure::Exited(value)) => Ok(value),
            other => other,
        };
        let sent = if self.trace_refusal.is_some() {
            self.ended_with_trace_refusal(sent)
        } else {
            sent
        };
        let untrapped = match &sent {
            Err(Failure::Raised(raised)) => Some(raised.clone()),
            _ => None,
        };
        let Some(then) = self.activity.root_then.take() else {
            unreachable!("a started activity records its outcome");
        };
        let started = match then {
            Then::Started(message) => Some(message),
            _ => None,
        };
        let recorded = self.apply_then(then, sent);
        let (failed, notified) = match (untrapped, recorded) {
            (Some(raised), _) => (raised, false),
            // A notification of the send's completion failed, which the
            // message is told of as the send's own failure. Its targets
            // have been sent `messageComplete` and are not sent it again
            // (Deviation 116).
            (None, Err(Failure::Raised(raised))) => {
                self.yield_after_notifier_failure()?;
                if let Some(message) = started {
                    self.activity.failed_sends.push(message);
                    self.message_outcomes.insert(message, Some(raised.clone()));
                }
                (raised, true)
            }
            (None, Err(failure)) => return Err(failure),
            (None, Ok(_)) => return Ok(Stopped::Ended),
        };
        self.end_failed_started(started, failed, notified)?;
        Ok(Stopped::Ended)
    }

    /// Settles and reports `failed`, the failure a started activity ended with,
    /// as the activity handler and then the thread's own do: the message
    /// `started` is told of it, and where a notifier fails, that failure
    /// replaces it and is reported. A target is sent `messageComplete` once for
    /// the message (Deviation 116): where `notified` says the failure is that
    /// of a notifier, the targets are not sent it again, and a replacement is
    /// not told to them.
    fn end_failed_started(
        &mut self,
        started: Option<ObjRef>,
        failed: Box<Raised>,
        notified: bool,
    ) -> Result<(), Failure> {
        self.settle_failed_sends(&failed)?;
        let Some(message) = started.filter(|_| !notified) else {
            self.report_started_failure(&failed);
            return Ok(());
        };
        let replaced = match self.notify_failed_send(message, false) {
            Ok(()) => {
                self.report_started_failure(&failed);
                return Ok(());
            }
            Err(Failure::Raised(replaced)) => replaced,
            Err(failure) => return Err(failure),
        };
        self.yield_after_notifier_failure()?;
        self.activity.failed_sends.push(message);
        self.message_outcomes
            .insert(message, Some(replaced.clone()));
        self.settle_failed_sends(&replaced)?;
        self.report_started_failure(&replaced);
        Ok(())
    }

    /// What a root driver's run of the running started activity answers:
    /// its send's outcome, or `None` where it parked or its slice ended.
    fn started_driven(
        &mut self,
        driven: Result<Driven, Failure>,
    ) -> Option<Result<Option<ObjRef>, Failure>> {
        match driven {
            Ok(Driven::Parked(floor)) => {
                self.activity.drive_floor = Some(floor);
                None
            }
            Ok(Driven::Sliced { floor, at }) => {
                self.activity.sliced = Some((floor, at));
                if !self.parked_for_guard() {
                    self.yield_at_slice();
                }
                None
            }
            Ok(Driven::Ended(ended)) => Some(self.finish_send(Ok(ended))),
            Err(failure) => Some(self.finish_send(Err(failure))),
        }
    }

    /// The refusal of a park that reached a caller other than its activity's
    /// root driver with no pinned frame counted, with the park dropped.
    #[cold]
    pub(crate) fn refuse_unrooted_park(&mut self) -> Failure {
        self.activity.native_park = None;
        Loud::scheduler_inconsistency("a wait outside every root driver and pinned frame").into()
    }

    /// The re-test a woken wait makes before it resumes: `Some` reason to
    /// park again for.
    pub(crate) fn retest_wait(&mut self, reason: ParkReason) -> Option<ParkReason> {
        match reason {
            ParkReason::Semaphore(wait) => self.retest_semaphore(wait).map(ParkReason::Semaphore),
            ParkReason::Sleep { .. }
                if !self.activity.woken_by_halt
                    && self.queue_wait_pending()
                    && !self.queue_wait_is_unending() =>
            {
                let deadline = self.now() + crate::dispatch::queue_methods::WAIT_POLL;
                Some(ParkReason::Sleep { deadline })
            }
            #[cfg(unix)]
            ParkReason::Sleep { .. } if !self.activity.woken_by_halt => {
                crate::builtin::rxsock::wait_again(self)
                    .map(|deadline| ParkReason::Sleep { deadline })
            }
            #[cfg(unix)]
            ParkReason::Socket if !self.activity.woken_by_halt => {
                crate::builtin::rxsock::block_again(self)
            }
            _ => None,
        }
    }

    /// Parks the running activity, woken from a park its root driver
    /// recorded, again where its re-test says so, answering whether it did.
    fn parked_again(&mut self) -> bool {
        let Some(reason) = self.activity.native_park.as_ref().map(|park| park.reason()) else {
            return false;
        };
        let Some(again) = self.retest_wait(reason) else {
            return false;
        };
        if let Some(park) = self.activity.native_park.as_mut() {
            park.set_reason(again);
        }
        self.park(again);
        true
    }

    /// The reason of the park the running activity's continuation recorded
    /// while it resumed, where it parked again rather than answering `sent`.
    fn parked_by_resume(&self, sent: &Result<Option<ObjRef>, Failure>) -> Option<ParkReason> {
        if sent.is_err() {
            return None;
        }
        self.activity.native_park.as_ref().map(|park| park.reason())
    }

    /// The running activity's park, resumed now that it has woken.
    fn resume_parked(&mut self, failure: Option<Failure>) -> Result<Option<ObjRef>, Failure> {
        let failure = failure.or_else(|| {
            self.activity
                .native_call
                .as_mut()
                .and_then(|call| call.take_spawn_failure())
        });
        let park = self.activity.native_park.take().or_else(|| {
            self.activity
                .native_call
                .as_mut()
                .and_then(|call| call.take_park())
        });
        match park {
            Some(park) => self.resume_native_park(*park, failure),
            None => {
                Err(Loud::scheduler_inconsistency("a woken activity with no wait recorded").into())
            }
        }
    }

    /// The traceback an untrapped condition writes on the started activity it
    /// ended, at once.
    fn report_started_failure(&mut self, raised: &crate::Raised) {
        let mut sites = std::mem::take(&mut self.activity.failure_sites);
        sites.extend(self.activity.failure_site.take());
        let path = self.program_path.clone();
        let site = crate::ClauseSite {
            path: &path,
            sites: &sites,
        };
        // With no level that has a line, the report names no program or line
        // (as a condition with no `POSITION` is displayed).
        let report = if sites.iter().any(crate::error::FailureSite::positions) {
            raised.report_lines(&site)
        } else {
            let mut positionless = raised.clone();
            positionless.delivery.positionless = true;
            positionless.report_lines(&site)
        };
        self.write_trace_report_lines(&report);
    }

    /// The running activity's main-program root, a park or slice in which
    /// runs the other activities until it is ready again. Its continuation
    /// is kept in its record, so a nested loop may run it too, and its end
    /// there is recorded for this loop to answer.
    pub(crate) fn run_activity_root(&mut self) -> Result<Ended, Failure> {
        let level = self.prepare_level()?;
        let driven = self.drive_from(DriveStart::Level(level), true);
        self.root_driven(driven);
        self.exit_if_awaited();
        loop {
            if let Some(end) = self.activity.root_end.take() {
                return end;
            }
            let failure = self.wait_until_woken();
            match failure {
                // Main ended in a nested round, and this failure came after:
                // the program's end reports it, as it does once main has
                // ended here.
                Some(failure) if self.activity.root_end.is_some() => {
                    self.keep_late_failure(failure);
                }
                failure if self.activity.root_end.is_none() => {
                    self.root_step(failure);
                    self.exit_if_awaited();
                }
                _ => {}
            }
        }
    }

    /// Main's root driver run from its recorded continuation, with `failure`
    /// as the answer of the wait it ends.
    fn root_step(&mut self, failure: Option<Failure>) {
        let driven = match (self.activity.sliced.take(), failure) {
            (Some(_), Some(failure)) => Err(failure),
            (Some((floor, at)), None) => {
                if !self.activity.guard_waits.is_empty() {
                    self.take_granted_guard();
                }
                self.drive_from(DriveStart::Sliced { floor, at }, true)
            }
            (None, None) if self.parked_again() => return,
            (None, failure) => match self.activity.drive_floor.take() {
                Some(floor) => {
                    let sent = self.resume_parked(failure);
                    if let Some(reason) = self.parked_by_resume(&sent) {
                        self.activity.drive_floor = Some(floor);
                        self.park(reason);
                        return;
                    }
                    self.drive_from(DriveStart::Woken { floor, sent }, true)
                }
                None => Err(Loud::scheduler_inconsistency("a main with no continuation").into()),
            },
        };
        self.root_driven(driven);
    }

    /// Records where main's root driver stopped: its park, its slice, which
    /// puts it at the back of the ready queue, or its end.
    fn root_driven(&mut self, driven: Result<Driven, Failure>) {
        match driven {
            Ok(Driven::Parked(floor)) => self.activity.drive_floor = Some(floor),
            Ok(Driven::Sliced { floor, at }) => {
                self.activity.sliced = Some((floor, at));
                if !self.parked_for_guard() {
                    self.yield_at_slice();
                }
            }
            Ok(Driven::Ended(ended)) => self.activity.root_end = Some(Ok(ended)),
            Err(failure) => self.activity.root_end = Some(Err(failure)),
        }
    }

    /// The clause boundaries the switch mode has counted.
    pub(crate) fn switch_clauses(&self) -> u64 {
        self.switch.as_ref().map_or(0, |switch| switch.clauses)
    }

    /// Whether the running activation has replied and its split has yet to
    /// run.
    pub(crate) fn split_owed(&self) -> bool {
        self.running_activation()
            .and_then(|activation| activation.replied.as_ref())
            .is_some_and(|replied| replied.continuation.is_none())
    }

    /// Switches activities as `mode` says, and never on the timer's word.
    pub(crate) fn set_switch_mode(&mut self, mode: SwitchMode) {
        if let SwitchMode::Sim(config) = &mode {
            self.start_sim(config.clone());
        }
        self.switch = Some(Switch { mode, clauses: 0 });
        self.timer.disarm();
        self.clause_countdown = 1;
    }

    /// The requests a visit of the clause countdown serves, before the clause
    /// begins: the sleepers now due, the switch mode's count, the timer's
    /// arming, and `SLICE`. A slice ends where the clause `yields` and
    /// nothing pins the activity; anywhere else it is deferred to the next
    /// clause that does, and a second slice that finds
    /// the activity still pinned takes a pinned yield.
    pub(crate) fn serve_requests(&mut self, yields: bool) -> Result<(), Failure> {
        if !self.activity.guard_waits.is_empty() {
            self.serve_guard_wait(yields)?;
        }
        if self.activity.splits_owed > 0 {
            if yields && self.split_owed() {
                return Err(Failure::Slice);
            }
            // A boundary closing a construct's branch, or a clause of a trap
            // handler the `REPLY` clause's end ran: the split waits for the
            // replier's next clause.
            self.clause_countdown = 1;
        }
        if self.stress_collect {
            self.collect_now();
        }
        if self.timer.requests().pending(crate::timer::INBOX) {
            self.drain_completions(DrainAt::MidRun);
        }
        if !self.activities.sleepers.is_empty() {
            self.wake_due_sleepers();
        }
        if let Some(switch) = &mut self.switch {
            self.clause_countdown = 1;
            switch.clauses += 1;
            let due = match &switch.mode {
                SwitchMode::EveryOpportunity => true,
                SwitchMode::AtClause(clause) => switch.clauses == *clause,
                SwitchMode::Sim(_) => {
                    let clauses = switch.clauses;
                    self.sim_clause(clauses)?
                }
            };
            if due {
                self.timer.requests().set(SLICE);
            }
        } else if self.activities.ready.is_empty() && self.activities.in_flight == 0 {
            self.timer.disarm();
        } else {
            self.timer.arm();
        }
        let fresh = self.timer.requests().pending(SLICE);
        if !fresh && !self.slice_deferred {
            return Ok(());
        }
        if fresh {
            self.timer.requests().clear(SLICE);
        }
        if self.activities.ready.is_empty() {
            self.slice_deferred = false;
            return Ok(());
        }
        if yields && self.activity.pin_depth == 0 {
            self.slice_deferred = false;
            return Err(Failure::Slice);
        }
        if self.slice_deferred && self.activity.routing_trace {
            self.count_delivery_clause();
        }
        // Inside a delivery the slice is taken at the first clause that may
        // yield, as the count is checked at each instruction.
        if (fresh || self.activity.routing_trace)
            && self.slice_deferred
            && self.activity.pin_depth > 0
            && self.delivery_may_yield()
        {
            self.slice_deferred = false;
            return self.pinned_yield();
        }
        // A delivery that holds a guard never yields (a second delivery
        // waiting for that guard would be an inverted wait), so one that
        // waits for another activity to act spins. After a bound of slices
        // it is refused as the inverted wait it is (Deviation 14), where
        // waits are refused as unending. Where they are not, a thread that
        // native code from outside started may still end the spin, so it
        // goes on, and stderr says so once.
        let refuses = self.refuses_unending_waits();
        if fresh
            && self.slice_deferred
            && self.activity.routing_trace
            && self.activity.pin_depth > 0
            && self.delivery_holds_a_guard()
            && let Some(delivery) = self.deliveries.last_mut()
        {
            delivery.guarded_slices = delivery.guarded_slices.saturating_add(1);
            if delivery.guarded_slices > GUARDED_DELIVERY_SLICES && refuses {
                self.slice_deferred = false;
                inverted_wait!(self, ParkReason::GuardWhen);
                return Err(Loud::inverted_wait("a guard").into());
            }
            if delivery.guarded_slices == GUARDED_DELIVERY_SLICES + 1 {
                self.write_err(
                    b"interrexx: a trace line's delivery holds a guard and has kept the \
                      activities ready to run from running for several seconds\n",
                );
            }
        }
        if fresh && !self.slice_deferred {
            if self.activity.routing_trace
                && let Some(delivery) = self.deliveries.last_mut()
            {
                delivery.counts.clear();
            }
            self.slice_deferred = true;
            deferred_slice!(self);
        }
        self.clause_countdown = 1;
        Ok(())
    }

    /// A REPLY rest with no clause, which ends where it starts.
    #[cold]
    #[inline(never)]
    fn finish_empty_rest(&mut self) -> Result<Option<ObjRef>, Failure> {
        self.finish_send(Ok(crate::run::Ended::Exited(None)))
    }

    /// The outcome of an activity that ends with
    /// [`Interp::trace_refusal`]: the refusal, which replaces `sent`.
    #[cold]
    #[inline(never)]
    fn ended_with_trace_refusal(
        &mut self,
        sent: Result<Option<ObjRef>, Failure>,
    ) -> Result<Option<ObjRef>, Failure> {
        drop(sent);
        Err(self
            .trace_refusal
            .take()
            .expect("the caller saw the refusal"))
    }

    /// Whether a pinned yield may be taken here, which outside a trace line's
    /// delivery it always may. Inside one it may not while the line's
    /// `TraceObject` is made, nor while an activation the destination runs
    /// holds its receiver's guard: another activity's delivery would wait for
    /// that guard under a pin of its own. Otherwise it may once the running
    /// activation has run more than [`DELIVERY_YIELD_CLAUSES`] clauses of its
    /// own since the slice fell due, its callees' not counted and not
    /// restarting its count. A yield happens only in an activation past its
    /// yield-instruction count, which each activation keeps, or at a native
    /// call, so a short destination's clauses never let another activity's
    /// lines in between.
    fn delivery_may_yield(&self) -> bool {
        let activity = &self.activity;
        if !activity.routing_trace {
            return true;
        }
        let Some(delivery) = self.deliveries.last() else {
            return false;
        };
        if delivery.floor.is_none() {
            return false;
        }
        let holds_a_guard = self.delivery_holds_a_guard();
        let running = activity.running.as_deref().map(|activation| activation.id);
        !holds_a_guard
            && delivery.counts.last().is_some_and(|&(id, clauses)| {
                Some(id) == running && clauses > DELIVERY_YIELD_CLAUSES
            })
    }

    /// Whether an activation of the running delivery's destination holds a
    /// guard.
    fn delivery_holds_a_guard(&self) -> bool {
        let activity = &self.activity;
        let Some(floor) = self.deliveries.last().and_then(|delivery| delivery.floor) else {
            return false;
        };
        activity
            .suspended
            .iter()
            .map(Box::as_ref)
            .chain(activity.running.as_deref())
            .skip(floor)
            .any(|activation| activation.flags.reserved())
    }

    /// Counts a clause of the running activation toward
    /// [`Delivery::counts`](crate::activity::Delivery::counts). The counts above its own belong to
    /// callees that have returned, and are dropped.
    fn count_delivery_clause(&mut self) {
        let Some(running) = self.running_activation().map(|activation| activation.id) else {
            return;
        };
        let Some(delivery) = self.deliveries.last_mut() else {
            return;
        };
        let counts = &mut delivery.counts;
        match counts.iter().rposition(|&(id, _)| id == running) {
            Some(at) => {
                counts.truncate(at + 1);
                counts[at].1 += 1;
            }
            None => counts.push((running, 1)),
        }
    }

    /// One round of the other ready activities, where a started activity ending
    /// holds no frame of its driver's: the activity a notifier failed in gives
    /// up the kernel lock while the failure unwinds, and an activity the
    /// completion woke runs then. Its end runs on below the round, so a loop
    /// above sets it aside. Each such yield nests, so one made with less than
    /// half the stack's room left is not made, keeping room for the code of the
    /// activities ending below it.
    fn yield_after_notifier_failure(&mut self) -> Result<(), Failure> {
        let probe = 0u8;
        if !self.any_ready()
            || self.stack_base.abs_diff(&raw const probe as usize) > self.stack_room / 2
        {
            return Ok(());
        }
        self.yield_at_slice();
        self.run_round(true).map(|_| ())
    }

    /// A pinned activity's yield: one round of the other ready activities on
    /// its own stack, as a pinned wait runs them, each until it parks, ends or
    /// its own slice ends; those whose frames lie below are set aside, and a
    /// round that set one aside is counted as an inverted yield. A pinned
    /// busy-wait on a buried activity therefore spins where a completed one
    /// would not, a recorded divergence.
    fn pinned_yield(&mut self) -> Result<(), Failure> {
        pinned_yield!(self);
        self.yield_at_slice();
        let (_, set_aside) = self.run_round(true)?;
        if set_aside {
            inverted_yield!(self);
        }
        Ok(())
    }

    /// The halt a clause boundary delivers, at the end of the clause the
    /// request found running, in that clause's activation: a `CALL ON` trap
    /// queues it, a delayed one included, a `SIGNAL ON` trap takes it, and
    /// untrapped it is 4.1.
    pub(crate) fn raise_requested_halt(
        &mut self,
        description: Option<Vec<u8>>,
        description_is_nil: bool,
    ) -> Result<(), Failure> {
        let raised = Raised {
            description: description.clone(),
            ..Raised::condition(std::borrow::Cow::Borrowed("HALT"))
        };
        // A trap taking the halt calls the halt exit's clear.
        if self.exits.is_some() && self.trap_for(b"HALT").is_some() {
            self.halt_clear_exit()?;
        }
        let description_object = description_is_nil.then_some(ObjRef::NIL);
        match self.trap_for(b"HALT") {
            Some(trap) if trap.call => {
                self.activity.pending_description = description_object;
                let object = self.build_condition_object(&raised, Some(true))?;
                // In a forwarding activation, such as the `.INPUT` monitor's
                // `UNKNOWN` a halted read ran in, the trap is the caller's,
                // whose clause is still running and takes it at its end:
                // measured, `parse pull` halted runs the handler before the
                // next clause.
                let running = self.activation().id;
                let owner = self.trap_frame().map_or(running, |frame| frame.id);
                self.activity.pending_traps.push_back(PendingTrap {
                    condition: b"HALT".as_slice().into(),
                    rc: None,
                    description,
                    object: Some(object),
                    activation: owner,
                    queued_during_delivery: owner == running,
                    request: false,
                    fragment_depth: self.activity.fragment_depth,
                    trace_failure: None,
                });
                Ok(())
            }
            Some(_) => {
                self.activity.pending_description = description_object;
                Err(raised.into())
            }
            None => Err(Raised::halt().into()),
        }
    }

    /// `Message~halt`: false for a message never started or sent and for an
    /// activity already asked; true where the request is queued for the
    /// running activation of the activity that started or sent it, and where
    /// that activity has no Rexx frame to make it to, which drops it.
    pub(crate) fn halt_message(&mut self, message: ObjRef, description: Option<Vec<u8>>) -> bool {
        if !self.started_messages.contains(&message) {
            let Some(&sender) = self.activities.senders.get(&message) else {
                return false;
            };
            let target = if sender == self.activities.running {
                Some(&mut self.activity)
            } else {
                match self.activities.idle.get_mut(sender.index()) {
                    Some(Some(idle)) => Some(&mut idle.activity),
                    _ => None,
                }
            };
            return match target {
                Some(target) => request_halt(target, description, false) != Some(false),
                None => true,
            };
        }
        let runs = |activity: &Activity| {
            matches!(
                activity.root_then,
                Some(Then::Started(started)) if started == message
            )
        };
        let target = if runs(&self.activity) {
            Some(&mut self.activity)
        } else {
            self.activities
                .idle
                .iter_mut()
                .flatten()
                .map(|idle| &mut idle.activity)
                .find(|activity| runs(activity))
        };
        match target {
            Some(target) => request_halt(target, description, false) != Some(false),
            None => true,
        }
    }

    /// Records the running activity as `message`'s sender, which
    /// [`Interp::halt_message`] asks.
    pub(crate) fn record_message_sender(&mut self, message: ObjRef) {
        self.activities
            .senders
            .insert(message, self.activities.running);
    }

    /// A signal's halt: every activity with a Rexx frame is asked to raise
    /// `HALT` with no description, and one parked in `SysSleep`, a `GUARD
    /// WHEN`, a semaphore wait or a command's wait is woken to take it, in
    /// handle order.
    pub(crate) fn halt_all(&mut self) {
        let mark = self.activities.ready.len();
        self.halt_each();
        if self.sim.is_some() {
            self.sim_order_event(mark);
        }
    }

    /// A halt for the running activity (`HaltThread`): its running Rexx frame
    /// is asked to halt.
    pub(crate) fn halt_running(&mut self) {
        let running = self.activities.running;
        if request_halt(&mut self.activity, None, false) == Some(true) {
            self.wake_for_halt(running);
        }
    }

    /// A halt with a description for the running activation, as a handler
    /// written in Rexx asks it with `caller~raise("HALT", rc)`: seen at the
    /// end of the clause, as a Ctrl-C is.
    pub(crate) fn request_running_halt(&mut self, description: Vec<u8>, description_is_nil: bool) {
        request_halt(&mut self.activity, Some(description), description_is_nil);
    }

    /// [`Interp::halt_all`] before the order of the activities it readies is
    /// settled.
    fn halt_each(&mut self) {
        let running = self.activities.running;
        for index in 0..self.activities.idle.len() {
            let activity = ActivityId(u32::try_from(index).expect("handles fit u32"));
            let record = if activity == running {
                &mut self.activity
            } else {
                match self.activities.idle.get_mut(index) {
                    Some(Some(idle)) => &mut idle.activity,
                    _ => continue,
                }
            };
            if request_halt(record, None, false) == Some(true) {
                self.wake_for_halt(activity);
            }
        }
    }

    /// [`Interp::set_external_trace`] for every activity of the interpreter, as a halt
    /// reaches them (`r[native.threads.trace]`): the running one and each idle one, whose
    /// frames are reached by putting it in the running one's place, as a read of it is.
    /// From the first such request on, every body compiles without settling a clause's
    /// trace setting ([`Interp::make_trace_switchable`]).
    pub(crate) fn set_external_trace_all(&mut self, on: bool) {
        self.make_trace_switchable();
        self.set_instance_activity_trace(on);
        for index in 0..self.activities.idle.len() {
            if self.activities.idle[index].is_some() {
                let activity = ActivityId(u32::try_from(index).expect("handles fit u32"));
                self.with_idle_activity(activity, |interp| interp.set_instance_activity_trace(on));
            }
        }
    }

    /// Ends `activity`'s park, if it is in one a halt ends.
    fn wake_for_halt(&mut self, activity: ActivityId) {
        let table = &mut self.activities;
        let record = if activity == table.running {
            &mut self.activity
        } else {
            match table.idle.get_mut(activity.0 as usize) {
                Some(Some(idle)) => &mut idle.activity,
                _ => return,
            }
        };
        if let Some(order) = record.asleep.take() {
            let before = table.sleepers.len();
            table
                .sleepers
                .retain(|Reverse((_, sleeper, _))| *sleeper != order);
            if table.sleepers.len() < before {
                record.woken_by_halt = true;
                table.make_ready(activity);
            }
        } else if std::mem::take(&mut record.when_parked) {
            record.woken_by_halt = true;
            self.unpark(activity);
        } else if let Some(token) = record.parked_block
            && record.blocked.abandon(token)
        {
            table.in_flight -= 1;
            table.abandoned_blocks.push(token);
            self.unpark(activity);
        } else if record.parked_socket.is_some() {
            // Its thread is told to stop, and the activity wakes when that
            // thread posts its end.
            record.woken_by_halt = true;
            #[cfg(unix)]
            crate::builtin::rxsock::cancel_helper(record);
        } else if let Some(order) = record.semaphore_wait.take() {
            let before = table.sleepers.len();
            if let Some(order) = order {
                table
                    .sleepers
                    .retain(|Reverse((_, sleeper, _))| *sleeper != order);
            }
            if table.sleepers.len() < before || table.semaphores.withdraw(activity) {
                record.woken_by_halt = true;
                self.unpark(activity);
            }
        }
    }

    /// Serves a signal's halt posted while this thread ran outside the clause
    /// loop, in a blocking operation that has returned: the halt is queued
    /// for the end of the running clause. Everything else posted waits for
    /// the next drain.
    #[inline(always)]
    pub(crate) fn serve_posted_halt(&mut self) {
        if self.timer.requests().pending(crate::timer::HALT) {
            self.serve_halt_now();
        }
    }

    /// Serves a signal whose handler has run and whose halt the timer thread
    /// has not posted yet: a terminal's Ctrl-C ends a command's child and
    /// signals this process at once, and the halt belongs to the command's
    /// clause.
    pub(crate) fn serve_signal_now(&mut self) {
        crate::timer::serve_signal();
        self.serve_posted_halt();
    }

    #[cold]
    #[inline(never)]
    fn serve_halt_now(&mut self) {
        self.timer.requests().clear(crate::timer::HALT);
        let mut posted = self.timer.drain();
        let before = posted.len();
        posted.retain(|post| !matches!(post, Posted::Halt));
        let halted = posted.len() < before;
        self.timer.requeue(posted);
        if halted {
            self.halt_all();
        }
    }

    /// Takes the running activation's halt request and raises it, for a
    /// `GUARD WHEN` a halt woke: `true` where one was taken.
    pub(crate) fn take_requested_halt(&mut self) -> Result<bool, Failure> {
        let here = self.activation().id;
        let Some(at) = self
            .activity
            .pending_traps
            .iter()
            .position(|pending| pending.request && pending.activation == here)
        else {
            return Ok(false);
        };
        let pending = self
            .activity
            .pending_traps
            .remove(at)
            .expect("position answered an index inside the queue");
        self.raise_requested_halt(pending.description, pending.object == Some(ObjRef::NIL))?;
        Ok(true)
    }
}

/// Queues a halt request for `target`'s running activation: `None` where it
/// has none, `Some(false)` where one is already queued, else `Some(true)`.
fn request_halt(
    target: &mut Activity,
    description: Option<Vec<u8>>,
    description_is_nil: bool,
) -> Option<bool> {
    let activation = target.running.as_ref()?.id;
    if target.pending_traps.iter().any(|pending| pending.request) {
        return Some(false);
    }
    target.pending_traps.push_back(PendingTrap {
        condition: b"HALT".as_slice().into(),
        rc: None,
        description,
        object: description_is_nil.then_some(ObjRef::NIL),
        activation,
        queued_during_delivery: true,
        request: true,
        fragment_depth: target.fragment_depth,
        trace_failure: None,
    });
    Some(true)
}

#[cfg(test)]
thread_local! {
    /// How many driver exits for a native call this thread has made.
    static NATIVE_EXITS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// How many abandoned calls' frames the last run on this thread still
    /// held at its end.
    static ABANDONED_HELD: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// What the cancelled waits on this thread withdrew their activities
    /// from.
    static CANCELLED_WAITS: std::cell::Cell<CancelledWaits> =
        const { std::cell::Cell::new(CancelledWaits { when_parks: 0, guard_waits: 0, sleeps: 0 }) };
}

/// How many cancelled waits found their activity in each wait
/// [`Interp::cancel_wait`] withdraws it from.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CancelledWaits {
    /// A `GUARD WHEN` park.
    pub(crate) when_parks: u64,
    /// A guard lock's queue.
    pub(crate) guard_waits: u64,
    /// The sleepers.
    pub(crate) sleeps: u64,
}

/// [`CANCELLED_WAITS`]'s counts.
#[cfg(test)]
pub(crate) fn cancelled_waits() -> CancelledWaits {
    CANCELLED_WAITS.with(std::cell::Cell::get)
}

/// [`ABANDONED_HELD`]'s count.
#[cfg(test)]
pub(crate) fn abandoned_held() -> usize {
    ABANDONED_HELD.with(std::cell::Cell::get)
}

/// Sets [`ABANDONED_HELD`] from `activities`.
#[cfg(test)]
pub(crate) fn note_abandoned_held(activities: &Activities) {
    ABANDONED_HELD.with(|held| held.set(activities.abandoned.len()));
}

#[cfg(test)]
thread_local! {
    /// How many native calls this thread's interpreters ran with the baton
    /// vacated.
    static VACATES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// [`VACATES`]'s count.
#[cfg(test)]
pub(crate) fn vacates() -> u64 {
    VACATES.with(std::cell::Cell::get)
}

/// [`NATIVE_EXITS`]'s count.
#[cfg(test)]
pub(crate) fn native_exits() -> u64 {
    NATIVE_EXITS.with(std::cell::Cell::get)
}

/// An outcome [`crate::Interp::exec_instruction`] answers in place of running
/// its instruction.
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum Scripted {
    /// `Park` for a sleep already due, without running the instruction.
    Park,
}

/// A state the next switch in the simulation mode breaks before it checks
/// the invariants, where the run has what it needs.
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum Corruption {
    /// The first ready activity queued again.
    ReadyTwice,
    /// The running activity queued.
    RunningReady,
    /// The first ready activity recorded as a message's waiter.
    ReadyParked,
    /// A sleeper taken off the sleepers, readied by nothing.
    NoWakeSource,
    /// A guard waiter's wait record dropped.
    GuardQueue,
    /// The baton read as held by another thread.
    Baton,
    /// A handle past the table's end queued as ready.
    BogusReady,
    /// A handle neither free nor naming an activity.
    UnfiledHandle,
    /// The first ready activity's handle also freed.
    FreeFiled,
    /// The running activity's handle freed.
    RunningFree,
    /// The first ready activity's record moved into the running slot.
    RunningFiled,
    /// A lock's owner set to a handle naming no activity.
    GuardOwnerDead,
    /// A lock's first waiter queued again.
    GuardWaiterTwice,
    /// A lock's first waiter taken off its queue, its wait record kept.
    GuardWaitUnqueued,
    /// The baton released for the check, and taken again after it.
    BatonReleased,
}

#[cfg(test)]
thread_local! {
    pub(crate) static CORRUPTION: std::cell::Cell<Option<Corruption>> =
        const { std::cell::Cell::new(None) };
    static BATON_ELSEWHERE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static BATON_RELEASED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
impl Interp {
    /// Applies the [`Corruption`] this thread asked for, once the run has a
    /// state it applies to.
    pub(crate) fn sim_corrupt(&mut self) {
        let Some(corruption) = CORRUPTION.with(std::cell::Cell::take) else {
            return;
        };
        let table = &mut self.activities;
        let running = table.running;
        let first = table.ready.front().copied();
        let applied = match (corruption, first) {
            (Corruption::ReadyTwice, Some(first)) => {
                table.ready.push_back(first);
                true
            }
            (Corruption::RunningReady, _) => {
                table.ready.push_back(running);
                true
            }
            (Corruption::ReadyParked, Some(first)) => {
                table
                    .waiters
                    .entry(MessageId(u32::MAX))
                    .or_default()
                    .push(first);
                true
            }
            (Corruption::NoWakeSource, _) => {
                let sleeper = table
                    .sleepers
                    .iter()
                    .map(|Reverse((_, _, sleeper))| *sleeper)
                    .find(|sleeper| *sleeper != running);
                if let Some(sleeper) = sleeper {
                    table
                        .sleepers
                        .retain(|Reverse((_, _, other))| *other != sleeper);
                }
                sleeper.is_some()
            }
            (Corruption::GuardQueue, _) => match table.guards.first_queued() {
                Some(waiter) => {
                    table.guards.woken(waiter);
                    true
                }
                None => false,
            },
            (Corruption::Baton, _) => {
                BATON_ELSEWHERE.with(|elsewhere| elsewhere.set(true));
                true
            }
            (Corruption::BogusReady, _) => {
                table.ready.push_back(ActivityId(9999));
                true
            }
            (Corruption::UnfiledHandle, _) => {
                table.idle.push(None);
                true
            }
            (Corruption::FreeFiled, Some(first)) => {
                table.free.push(first.0);
                true
            }
            (Corruption::RunningFree, _) => {
                table.free.push(running.0);
                true
            }
            (Corruption::RunningFiled, Some(first)) => {
                let moved = table.idle[first.index()].take();
                table.idle[running.index()] = moved;
                true
            }
            (Corruption::GuardOwnerDead, _) => table.guards.corrupt_owner(ActivityId(9999)),
            (Corruption::GuardWaiterTwice, _) => table.guards.corrupt_queue_twice(),
            (Corruption::GuardWaitUnqueued, _) => table.guards.corrupt_unqueue(),
            (Corruption::BatonReleased, _) => {
                self.baton.release();
                BATON_RELEASED.with(|released| released.set(true));
                true
            }
            (
                Corruption::ReadyTwice
                | Corruption::ReadyParked
                | Corruption::FreeFiled
                | Corruption::RunningFiled,
                None,
            ) => false,
        };
        if !applied {
            CORRUPTION.with(|asked| asked.set(Some(corruption)));
        }
    }

    /// Takes the baton back where [`Corruption::BatonReleased`] released it.
    pub(crate) fn sim_uncorrupt(&self) {
        if BATON_RELEASED.with(std::cell::Cell::take) {
            self.baton.acquire();
        }
    }
}

#[cfg(test)]
thread_local! {
    static SCRIPT: std::cell::RefCell<std::collections::VecDeque<Scripted>> =
        const { std::cell::RefCell::new(std::collections::VecDeque::new()) };
}

/// Queues `outcomes` for the next `Op::Exec` instructions this thread runs.
#[cfg(test)]
pub(crate) fn script(outcomes: &[Scripted]) {
    SCRIPT.with(|script| script.borrow_mut().extend(outcomes.iter().copied()));
}

/// The next queued outcome, outside the library bootstrap.
#[cfg(test)]
pub(crate) fn take_scripted() -> Option<Scripted> {
    if !crate::ir::drive::counting() {
        return None;
    }
    SCRIPT.with(|script| script.borrow_mut().pop_front())
}

mod pool;
pub(crate) use pool::{POOL_BOUND, POOL_STACK_BYTES, Pool, Worker, posting_panics};
#[cfg(test)]
pub(crate) use pool::{last_pool_held, threads_beyond_bound, threads_spawned};

#[cfg(test)]
#[expect(clippy::disallowed_methods, reason = "these tests time real runs")]
mod tests;
