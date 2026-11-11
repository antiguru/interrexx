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

The locks of objects and methods and the `GUARD` instruction on the one hand,
and `HALT` and the end of a program that still has activities on the other,
are specified on pages of this chapter that are not linked here yet: the locks
page and the halt page.

The methods of the classes `Message`, `Alarm`, `Ticker`, `Monitor`,
`EventSemaphore` and `MutexSemaphore`, and the instructions `REPLY` and
`GUARD`, are specified on their own pages; this chapter links them and
specifies only what they have in common.

Every example in this chapter that uses more than one activity forces the
order of what it prints. It waits for the result of a message, parks an
activity on a `GUARD ... WHEN` that the program opens when it is ready, or
asks for results in a fixed order; it never relies on how long an activity
takes, and the output of such an example is the same on every run.
