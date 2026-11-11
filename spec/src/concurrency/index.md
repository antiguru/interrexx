---
element: "Concurrency"
kind: chapter
rexxref: "Concurrency"
---

# Concurrency

This chapter specifies how a program does more than one thing at a time: the
activities that run its code, how they are made, what they share and what they
keep apart, and what one of them can and cannot know of the others.

The pages of this chapter so far are:

* [Activities](activities.md): what an activity is, which code runs on which
  activity when a program uses `start`, `REPLY`, a message object, an alarm or
  a ticker, how the result or the error of a message that another activity ran
  reaches the program, and which things happen in an order that a program can
  count on and which do not.

* [Locks](locks.md): what a lock belongs to, which methods take one and when,
  what an activity that holds one may do, how often it is held, and what
  happens when activities wait for each other in a ring.

* [Halt and program end](halt.md): where a `HALT` asked for by another
  activity or by the host lands in an activity and what it does not end, what
  keeps a program alive after its main program has ended and what its status
  is then, and how the message `UNINIT` that the interpreter sends works with
  more than one activity.

The methods of the classes [`Message`](../classes/message.md),
[`Alarm`](../classes/alarm.md), [`Ticker`](../classes/ticker.md),
[`Monitor`](../classes/monitor.md),
[`EventSemaphore`](../classes/eventsemaphore.md) and
[`MutexSemaphore`](../classes/mutexsemaphore.md), and the instructions
[`REPLY`](../instructions/reply.md) and [`GUARD`](../instructions/guard.md),
are specified on their own pages; this chapter links them and specifies only
what they have in common.

Every example in this chapter that uses more than one activity forces the
order of what it prints. It waits for the result of a message, parks an
activity on a `GUARD ... WHEN` that the program opens when it is ready, or
asks for results in a fixed order; it never relies on how long an activity
takes, and the output of such an example is the same on every run.
