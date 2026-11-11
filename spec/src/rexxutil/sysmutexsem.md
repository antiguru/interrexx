---
element: "SysCreateMutexSem"
kind: function
syntax: "SysCreateMutexSem(name); SysOpenMutexSem(name); SysRequestMutexSem(handle, [timeout]); SysReleaseMutexSem(handle); SysCloseMutexSem(handle)"
deviations: [20, 43]
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
omitted place that is followed by an argument. A name that is not the null
string asks for a semaphore that other programs can open, which [Deviation
43](r:util.sysmutexsem.deviation.43) says is refused. Which handle a call
answers is not specified.

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
nobody holds it. It frees the lock once, however often the lock was requested,
since a request that waits does not take it a second time. A request that
waits for the lock takes it when the lock is free, as [the next
rule](r:util.sysmutexsem.blocking) shows. In the example another activity
holds the lock, and the program frees it, then takes it.

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
value that is no whole number in range `SYNTAX 88.907`. What a call does with
the handle of an event semaphore, with a whole number that no call answered,
or with the handle of a closed semaphore, is not specified.

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
say SysRequestMutexSem(' ' || h || ' ', 20) SysReleaseMutexSem('+' || h)
say SysRequestMutexSem(h || '.0', 20) SysReleaseMutexSem(.mutablebuffer~new(h))
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
0 0
0 0
0
```

r[util.sysmutexsem.close]
SysCloseMutexSem ends the use of a semaphore and answers `0`. It answers `6`
when the semaphore is closed already, and so a second call with the same
handle answers `6`. Calls with a closed handle other than SysCloseMutexSem are
not specified.

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
refused ([Deviation 43](r:util.sysmutexsem.deviation.43)).

```rexx
call try "SysOpenMutexSem()"
call try "SysOpenMutexSem(,)"
call try "SysOpenMutexSem('', 1)"
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
A call of SysCreateMutexSem with a name that is not the null string, and a
call of SysOpenMutexSem with a name that is not the null string, end the
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
