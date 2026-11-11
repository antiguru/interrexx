---
element: "SysCreateMutexSem"
kind: function
syntax: "SysCreateMutexSem(name); SysOpenMutexSem(name); SysRequestMutexSem(handle, [timeout]); SysReleaseMutexSem(handle); SysCloseMutexSem(handle)"
deviations: [20, 43, 158]
rexxref: "Deprecated Rexx features > RexxUtil Semaphore functions"
---

# The mutex semaphore functions

The functions SysCreateMutexSem, SysOpenMutexSem, SysRequestMutexSem,
SysReleaseMutexSem and SysCloseMutexSem make and use a mutex semaphore, a lock
that one activity at a time can hold. They are deprecated; [the MutexSemaphore
class](../classes/mutexsemaphore.md) takes their place, and differs from them
in two ways that the rules below show: a lock of the class has an owner, the
activity that took it, and the owner can take it again, where a lock of these
functions is not owned and is taken once. [The event semaphore
functions](syseventsem.md) own the rules for the handle, for the timeout and
for closing, which these functions share.

## Making one

r[util.sysmutexsem.create]
SysCreateMutexSem with the null string as its name makes a mutex semaphore of
its own, which nobody holds, and answers its handle: a whole number above 0,
different from the handle of every other semaphore that is still open. An
object whose string value is the null string does the same. The name is the
only argument; a second argument raises `SYNTAX 88.922`, and so does an
omitted place that is followed by an argument. A name that is a string other
than the null string, or an object whose string value is such a string, asks
for a semaphore that other programs can open, which
[Deviation 43](r:util.sysmutexsem.deviation.43) says is refused. A name that
is an object with no string value, as `.nil` is, raises `SYNTAX 88.909`; the
second example shows it. Which handle a call answers is not specified. When
an argument past the last one comes with an omitted or wrong argument,
[Deviation 101](r:util.index.deviation.101) says which error is raised.

```rexx
numeric digits 20
h1 = SysCreateMutexSem('')
h2 = SysCreateMutexSem('', )
h3 = SysCreateMutexSem(.mutablebuffer~new)
say datatype(h1, 'W') (h1 > 0) datatype(h3, 'W') (h3 > 0)
say (h1 \= h2) (h1 \= h3) (h2 \= h3)
say SysRequestMutexSem(h1, 20) SysRequestMutexSem(h2, 20) SysRequestMutexSem(h3, 20)
call try "SysCreateMutexSem('', 1)"
call try "SysCreateMutexSem('', 'x')"
call try "SysCreateMutexSem(, 1)"
call try "SysCreateMutexSem(1, 2, 3)"
do h over .array~of(h1, h2, h3)
  call SysCloseMutexSem h
end
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
1 1 1 1
1 1 1
0 0 0
88.922
88.922
88.922
88.922
```

```rexx
call try "SysCreateMutexSem(.nil)"
call try "SysCreateMutexSem(.object~new)"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output defect=#246
88.909
88.909
```

## Requesting and releasing

r[util.sysmutexsem.request]
SysRequestMutexSem takes the lock and answers `0`, or answers `121` when it
could not take it before the `timeout` passed. A lock that nobody holds is
taken at once. A lock that is held makes the caller wait, whoever holds it,
the caller included: the lock is not taken twice by the activity that holds
it, so a second request by that activity waits for its whole `timeout`.
`timeout` is read as [the event wait rule](r:util.syseventsem.wait) reads it:
a number of milliseconds from `-2147483648` to `2147483647`, where `0` or no
`timeout` sets no limit, a value above `0` is a limit that the call does not
answer `121` before, and a value below `0` answers `0` at once without a wait
and without taking the lock. Any other value raises `SYNTAX 88.907`.

```rexx
numeric digits 20
m = SysCreateMutexSem('')
say SysRequestMutexSem(m)
call time 'R'
say SysRequestMutexSem(m, 120) (time('E') >= 0.12)
say SysRequestMutexSem(m, -1) SysRequestMutexSem(m, -2147483648)
say SysReleaseMutexSem(m)
say SysRequestMutexSem(m, -1) SysRequestMutexSem(m, 20)
say SysReleaseMutexSem(m)
say SysRequestMutexSem(m, 0) SysReleaseMutexSem(m)
say SysRequestMutexSem(m, 2147483647) SysReleaseMutexSem(m)
call try "SysRequestMutexSem(m, 'x')"
call try "SysRequestMutexSem(m, 1.5)"
call try "SysRequestMutexSem(m, 2147483648)"
call try "SysRequestMutexSem(m, -2147483649)"
call try "SysRequestMutexSem(m, 1, 2)"
call try "SysRequestMutexSem()"
call SysCloseMutexSem m
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
0
121 1
0 0
0
0 0
0
0 0
0 0
88.907
88.907
88.907
88.907
88.922
88.901
```

r[util.sysmutexsem.release]
SysReleaseMutexSem makes the lock free and answers `0`. It does so whichever
activity calls it, the one that holds the lock or another, and also when
nobody holds it. One release makes the lock free, however many requests were
made while it was held. A request that waits for the lock takes it when the
lock is free, as [the next rule](r:util.sysmutexsem.blocking) shows. In the
example another activity holds the lock, and the program frees it, then takes
it.

```rexx
numeric digits 20
m = SysCreateMutexSem('')
say SysReleaseMutexSem(m)
a = SysCreateEventSem()
g = SysCreateEventSem()
t = .w~new~start('run', m, a, g)
call SysWaitEventSem a
say 'held:' SysRequestMutexSem(m, 50)
say 'release by another activity:' SysReleaseMutexSem(m)
say 'taken:' SysRequestMutexSem(m, 50)
say 'held again:' SysRequestMutexSem(m, 50)
call SysPostEventSem g
say t~result
say SysReleaseMutexSem(m) SysReleaseMutexSem(m)
say SysRequestMutexSem(m, 50)
call SysCloseMutexSem m
::class w
::method run unguarded
  use arg m, a, g
  r = SysRequestMutexSem(m)
  call SysPostEventSem a
  call SysWaitEventSem g
  return 'other ends' r
```

```output
0
held: 121
release by another activity: 0
taken: 0
held again: 121
other ends 0
0 0
0
```

r[util.sysmutexsem.blocking]
A request that has to wait suspends the activity that made it, and the other
activities go on; the release that makes the lock free ends the wait of the
request, which takes the lock and answers `0`. In the example the message the
program starts asks for a lock that the program holds, with no limit, and
waits; the program frees the lock, and the message gets it. (The locks page of
the concurrency chapter has the rules for an activity that waits.)

```rexx
m = SysCreateMutexSem('')
call SysRequestMutexSem m
t = .w~new~start('run', m)
call SysReleaseMutexSem m
say t~result
say 'held by the message:' SysRequestMutexSem(m, 30)
call SysCloseMutexSem m
::class w
::method run unguarded
  use arg m
  return 'got it' SysRequestMutexSem(m)
```

```output
got it 0
held by the message: 121
```

## The handle and closing

r[util.sysmutexsem.handle]
The functions that take a handle read it as [the handle rule of the event
semaphore functions](r:util.syseventsem.handle) says, and raise the same
errors for the same values: a missing handle `SYNTAX 88.901`, a second
argument of SysReleaseMutexSem or SysCloseMutexSem `SYNTAX 88.922`, and a
value that is no whole number in range `SYNTAX 88.907`. When an argument past
the last one comes with an omitted or wrong argument, [Deviation
101](r:util.index.deviation.101) says which error is raised. A whole number
that is not the handle of an open semaphore is answered as [Deviation
158](r:util.sysmutexsem.deviation.158) says.

```rexx
numeric digits 20
h = SysCreateMutexSem('')
call try "SysRequestMutexSem()"
call try "SysReleaseMutexSem()"
call try "SysReleaseMutexSem(h, 1)"
call try "SysReleaseMutexSem('x')"
call try "SysReleaseMutexSem('')"
call try "SysReleaseMutexSem(-1)"
call try "SysReleaseMutexSem(1.5)"
call try "SysReleaseMutexSem(.nil)"
call try "SysRequestMutexSem('x', 1)"
call try "SysCloseMutexSem()"
call try "SysCloseMutexSem(h, 1)"
call try "SysCloseMutexSem('x')"
call try "SysReleaseMutexSem('1.0000000000000000001')"
say SysRequestMutexSem(' ' || h || ' ', 20) SysReleaseMutexSem('+' || h)
say SysRequestMutexSem(h || '.0', 20) SysReleaseMutexSem(.mutablebuffer~new(h))
say SysRequestMutexSem(h || '.00000000000000000001', 20) SysReleaseMutexSem(h)
say SysCloseMutexSem(' ' || h || ' ')
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
88.901
88.901
88.922
88.907
88.907
88.907
88.907
88.907
88.907
88.901
88.922
88.907
88.907
0 0
0 0
0 0
0
```

```rexx
m = SysCreateMutexSem('')
call try "SysRequestMutexSem(.sv~new(m), 10)"
call try "SysRequestMutexSem(m, .sv~new('10'))"
call try "SysReleaseMutexSem(.sv~new(m))"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::class sv
::method init
  expose v
  use arg v
::method string
  expose v
  return v
```

```output defect=#230
0
121
0
```

r[util.sysmutexsem.close]
SysCloseMutexSem ends the use of a semaphore and answers `0`. It answers `6`
when the semaphore is closed already, and so a second call with the same
handle answers `6`. The other functions answer a closed handle as [Deviation
158](r:util.sysmutexsem.deviation.158) says.

```rexx
h = SysCreateMutexSem('')
say SysCloseMutexSem(h)
say SysCloseMutexSem(h) SysCloseMutexSem(h)
```

```output
0
6 6
```

## Opening by name

r[util.sysmutexsem.open]
SysOpenMutexSem takes the name of a semaphore and answers its handle, or `0`
for a name that it cannot open. The name is required: a call without one
raises `SYNTAX 88.901`, a second argument `SYNTAX 88.922`, and an object that
has no string value `SYNTAX 88.909`. The null string is no name of a
semaphore, so that the call answers `0`. A name that is not the null string is
refused ([Deviation 43](r:util.sysmutexsem.deviation.43)). An omitted name and
an object that has no string value are reported before an argument that
follows them: `SysOpenMutexSem(, 1)` raises `SYNTAX 88.901` and
`SysOpenMutexSem(.nil, 1)` `SYNTAX 88.909`.

```rexx
call try "SysOpenMutexSem()"
call try "SysOpenMutexSem(,)"
call try "SysOpenMutexSem('', 1)"
call try "SysOpenMutexSem(, 1)"
call try "SysOpenMutexSem(.nil, 1)"
call try "SysOpenMutexSem(.nil)"
call try "SysOpenMutexSem('')"
call try "SysOpenMutexSem(.mutablebuffer~new)"
exit
try:
  signal on syntax name e
  interpret 'x =' arg(1)
  say x
  return
e: say condition('o')~code
  return
```

```output
88.901
88.901
88.922
88.901
88.909
88.909
0
0
```

## Deviations

r[util.sysmutexsem.deviation.20]
A call of SysRequestMutexSem that has no limit, and finds the lock held, while
no other activity or timer of the program can free it, ends the program at
once, with exit code `120` and a message on the error stream ([Deviation
20](../deviations.md#20-a-wait-nothing-can-end)). ooRexx waits for ever. In
the example the program asks twice for a lock that it holds itself.

```rexx
say 'before'
m = SysCreateMutexSem('')
say SysRequestMutexSem(m)
say SysRequestMutexSem(m)
say 'after'
```

```output rc=120
before
0
```

r[util.sysmutexsem.deviation.43]
A call of SysCreateMutexSem with a name that
is a string other than the null string, or an object whose string value is
such a string, and a call of SysOpenMutexSem with a name of that kind, end the
program at once, with exit code `120` and a message on the error stream: the
semaphore is not made or opened, and nothing that follows the call runs.
ooRexx makes a semaphore of that name that every program of the system can
open, and answers its handle; SysOpenMutexSem answers `0` for a name that no
program has made. The first example is a creating call, the second an opening
one.

```rexx
say 'before'
h = SysCreateMutexSem('c5zqmutex')
say 'after'
```

```output rc=120
before
```

```rexx
say 'before'
h = SysOpenMutexSem('c5zqmutex')
say 'after'
```

```output rc=120
before
```

r[util.sysmutexsem.deviation.158]
A whole number that is not the handle of an open semaphore makes
SysRequestMutexSem, SysReleaseMutexSem and SysCloseMutexSem answer `6`, and
use no semaphore, as [the event semaphore
functions](r:util.syseventsem.deviation.158) do: the handle of a semaphore
that is closed, a handle that arithmetic has rounded, as the sum `h + 0` of a
handle of thirteen digits is under `NUMERIC DIGITS 9`, and a number that no
call answered. ooRexx uses the number as the address of the semaphore and
crashes, with a signal that ends the program, for a closed handle
(SysRequestMutexSem and SysReleaseMutexSem), for a rounded one, as it does for
SysPostEventSem, and for a number that no call answered; SysRequestMutexSem
with an event semaphore's handle hangs. The handle of an open event semaphore
is such a number too: these calls answer `6` for it and change neither
semaphore (issue #247 for interrexx, which accepts it today; [the event
semaphore rule](r:util.syseventsem.deviation.158) has the example). The first
example is a rounded handle and a closed one, the second a number that no call
answered.

```rexx
m = SysCreateMutexSem('')
r = m + 0
say (r \== m) SysRequestMutexSem(r, 10) SysReleaseMutexSem(r) SysCloseMutexSem(r)
call SysCloseMutexSem m
say SysRequestMutexSem(m, 10) SysReleaseMutexSem(m)
```

```output
1 6 6 6
6 6
```

```rexx
say SysRequestMutexSem(12345, 10) SysReleaseMutexSem(12345) SysCloseMutexSem(12345)
```

```output
6 6 6
```
