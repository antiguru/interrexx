---
element: "SysCreateEventSem"
kind: function
syntax: "SysCreateEventSem([name], [reserved]); SysOpenEventSem(name); SysPostEventSem(handle); SysResetEventSem(handle); SysWaitEventSem(handle, [timeout]); SysCloseEventSem(handle); SysPulseEventSem(handle)"
deviations: [20, 43, 158]
rexxref: "Deprecated Rexx features > RexxUtil Semaphore functions"
---

# The event semaphore functions

The functions SysCreateEventSem, SysOpenEventSem, SysPostEventSem,
SysResetEventSem, SysWaitEventSem and SysCloseEventSem make and use an event
semaphore, a counter of posts that an activity can wait on. They are
deprecated; [the EventSemaphore class](../classes/eventsemaphore.md) takes
their place, and differs from them: one post of an object of the class ends
the wait of every activity that waits and stays until it is reset, and one
post of these functions ends one wait. [The mutex semaphore
functions](sysmutexsem.md) work the same way for a lock.

A semaphore is named by its handle, a string of digits that the function that
made it answers. Every function that takes a handle takes it as its first
argument, and answers `0` when it did what it was asked; the other numbers
that a function answers are listed with it.

## Making one

r[util.syseventsem.create]
SysCreateEventSem makes an event semaphore of its own, with no post, and
answers its handle: a whole number above 0, different from the handle of every
other semaphore that is still open. It does so when its first argument is
omitted, as in `SysCreateEventSem()` and `SysCreateEventSem(,1)`. Its second
argument is not used: it may be any string, and an object that has no string
value raises `SYNTAX 88.909`, as [the argument rule](r:util.index.args) says.
A third argument raises `SYNTAX 88.922`. When the first argument is the null
string, or an object whose string value it is, no semaphore is made and the
call answers the null string. A name that is not the null string asks for a
semaphore that other programs can open, which [Deviation
43](r:util.syseventsem.deviation.43) says is refused. Which handle a call
answers is not specified.

```rexx
numeric digits 20
h1 = SysCreateEventSem()
h2 = SysCreateEventSem(, 1)
h3 = SysCreateEventSem(,)
h4 = SysCreateEventSem(, 'x')
say datatype(h1, 'W') (h1 > 0) datatype(h4, 'W') (h4 > 0)
say (h1 \= h2) (h1 \= h3) (h1 \= h4) (h2 \= h3) (h2 \= h4) (h3 \= h4)
say '['SysCreateEventSem('')']' '['SysCreateEventSem('', 1)']' '['SysCreateEventSem('', 'x')']'
say '['SysCreateEventSem(.mutablebuffer~new)']'
say SysWaitEventSem(h1, 20)
signal on syntax name e
say SysCreateEventSem('', 1, 2)
e: say condition('o')~code
do h over .array~of(h1, h2, h3, h4)
  call SysCloseEventSem h
end
```

```output
1 1 1 1
1 1 1 1 1 1
[] [] []
[]
121
88.922
```

```rexx
signal on syntax name e
say SysCreateEventSem(, .nil)
e: say condition('o')~code
```

```output defect=#238
88.909
```

## Posting, resetting and waiting

r[util.syseventsem.post]
SysPostEventSem adds one post to the semaphore and answers `0`. The posts are
counted: a semaphore that has been posted three times lets three waits end,
and a fourth wait finds none. A post that nobody waits for is kept until a
wait uses it or SysResetEventSem discards it.

```rexx
h = SysCreateEventSem()
do 3
  say SysPostEventSem(h)
end
say SysWaitEventSem(h, 20) SysWaitEventSem(h, 20) SysWaitEventSem(h, 20) SysWaitEventSem(h, 20)
say SysPostEventSem(h)
say SysWaitEventSem(h, 20) SysWaitEventSem(h, 20)
call SysCloseEventSem h
```

```output
0
0
0
0 0 0 121
0
0 121
```

r[util.syseventsem.reset]
SysResetEventSem discards every post that the semaphore holds and answers `0`,
whether it held posts or not. A wait that follows it finds none.

```rexx
h = SysCreateEventSem()
say SysResetEventSem(h)
call SysPostEventSem h
call SysPostEventSem h
say SysResetEventSem(h) SysWaitEventSem(h, 20)
call SysPostEventSem h
say SysWaitEventSem(h, 20) SysResetEventSem(h) SysResetEventSem(h) SysWaitEventSem(h, 20)
call SysCloseEventSem h
```

```output
0
0 121
0 0 0 121
```

r[util.syseventsem.wait]
SysWaitEventSem uses up one post and answers `0`, or answers `121` when the
`timeout` passed without a post to use. `timeout` is a number of milliseconds,
a whole number from `-2147483648` to `2147483647`, read whatever `NUMERIC
DIGITS` is in force; a value that is not a number or is outside that range
raises `SYNTAX 88.907`, as does a fraction. A
`timeout` that is `0` or omitted sets no limit: the wait ends only when a post
is there. A `timeout` above `0` is a limit, and the call does not answer `121`
before at least that long has gone by; how much longer it takes is not
specified. A `timeout` below `0` makes the call answer `0` at once, without a
wait and without using a post, whether the semaphore has one or not.

```rexx
numeric digits 20
h = SysCreateEventSem()
call time 'R'
say SysWaitEventSem(h, 120) (time('E') >= 0.12)
say SysWaitEventSem(h, -1) SysWaitEventSem(h, -2147483648) SysWaitEventSem(h, '-5')
call SysPostEventSem h
say SysWaitEventSem(h, -1) SysWaitEventSem(h, 20) SysWaitEventSem(h, 20)
call SysPostEventSem h
call SysPostEventSem h
say SysWaitEventSem(h, 0) SysWaitEventSem(h, 2147483647) SysWaitEventSem(h, 20)
call try "SysWaitEventSem(h, 'x')"
call try "SysWaitEventSem(h, '')"
call try "SysWaitEventSem(h, 1.5)"
call try "SysWaitEventSem(h, 2147483648)"
call try "SysWaitEventSem(h, -2147483649)"
call try "SysWaitEventSem(h, .nil)"
numeric digits 3
call try "SysWaitEventSem(h, 1000.4)"
numeric digits 20
call try "SysWaitEventSem(h, 1, 2)"
call SysCloseEventSem h
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
121 1
0 0 0
0 0 121
0 0 121
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.922
```

r[util.syseventsem.blocking]
A wait that has to wait suspends the activity that made it, and the other
activities go on; a post that another activity makes ends it, and the call
answers `0`. Each post ends one wait: when more activities wait than the
semaphore has posts, the others go on waiting, and which of them gets a post
is not specified. In the first example the message that the program starts
waits with no limit, and the program posts. In the second the program posts
twice and then starts three messages that wait for 600 milliseconds: two waits
answer `0` and the third, which no post is left for, answers `121`. (The locks
page of the concurrency chapter has the rules for an activity that waits.)

```rexx
h = SysCreateEventSem()
m = .w~new~start('run', h)
say SysPostEventSem(h)
say m~result
call SysCloseEventSem h
::class w
::method run unguarded
  use arg h
  return 'waited' SysWaitEventSem(h)
```

```output
0
waited 0
```

```rexx
h = SysCreateEventSem()
call SysPostEventSem h
call SysPostEventSem h
r = .array~new
do 3
  r~append(.w~new~start('run', h))
end
res = r[1]~result r[2]~result r[3]~result
say 'zeros' countstr(' 0', ' 'res) 'timeouts' countstr('121', res)
call SysCloseEventSem h
::class w
::method run unguarded
  use arg h
  return SysWaitEventSem(h, 600)
```

```output
zeros 2 timeouts 1
```

## The handle

r[util.syseventsem.handle]
The functions that take a handle read it as a whole number from `0` to
`18446744073709551615`, in the form the creating call answered it. Blanks
around it, a plus sign and a fraction of only zeros do no harm. A handle that
is missing raises `SYNTAX 88.901`, and one argument too many raises `SYNTAX
88.922`. A value that is not a whole number or is out of that range raises
`SYNTAX 88.907`: a word, the null string, a negative number, a number with a
fraction and `.nil`. When an argument past the last one comes with an omitted
or wrong argument, [Deviation 101](r:util.index.deviation.101) says which
error is raised. A whole number that is not the handle of an open semaphore is
answered as [Deviation 158](r:util.syseventsem.deviation.158) says.

```rexx
h = SysCreateEventSem()
call try "SysPostEventSem()"
call try "SysPostEventSem(h, 1)"
call try "SysPostEventSem('x')"
call try "SysPostEventSem('')"
call try "SysPostEventSem(-1)"
call try "SysPostEventSem(1.5)"
call try "SysPostEventSem(.nil)"
call try "SysPostEventSem(h || ' 1')"
call try "SysPostEventSem(18446744073709551616)"
call try "SysWaitEventSem()"
call try "SysWaitEventSem('x', 1)"
call try "SysWaitEventSem(.nil, 1)"
call try "SysResetEventSem()"
call try "SysResetEventSem(h, 1)"
call try "SysResetEventSem(-1)"
call try "SysCloseEventSem()"
call try "SysCloseEventSem(h, 1)"
call try "SysCloseEventSem('x')"
say SysPostEventSem(' ' || h || ' ') SysPostEventSem('+' || h) SysPostEventSem(h || '.0')
say SysPostEventSem(.mutablebuffer~new(h)) SysPostEventSem(h || '.00')
say SysWaitEventSem(' ' || h || ' ', 20) SysWaitEventSem('+' || h, 20)
say SysResetEventSem(' ' || h || ' ') SysResetEventSem('+' || h)
say SysCloseEventSem(' ' || h || ' ')
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
88.922
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.901
88.907
88.907
88.901
88.922
88.907
88.901
88.922
88.907
0 0 0
0 0
0 0
0 0
0
```

## Closing

r[util.syseventsem.close]
SysCloseEventSem ends the use of a semaphore and answers `0`. It answers `6`
when the semaphore is closed already, and so a second call with the same
handle answers `6`. The other functions answer a closed handle as [Deviation
158](r:util.syseventsem.deviation.158) says.

```rexx
h = SysCreateEventSem()
say SysCloseEventSem(h)
say SysCloseEventSem(h) SysCloseEventSem(h)
```

```output
0
6 6
```

## Opening by name, and pulsing

r[util.syseventsem.open]
SysOpenEventSem takes the name of a semaphore and answers its handle, or `0`
for a name that it cannot open. The name is required: a call without one
raises `SYNTAX 88.901`, a second argument `SYNTAX 88.922`, and an object that
has no string value `SYNTAX 88.909`. The null string is no name of a
semaphore, so that the call answers `0`. A name that is not the null string is
refused ([Deviation 43](r:util.syseventsem.deviation.43)). An omitted name and
an object that has no string value are reported before an argument that
follows them: `SysOpenEventSem(, 1)` raises `SYNTAX 88.901` and
`SysOpenEventSem(.nil, 1)` `SYNTAX 88.909`.

```rexx
call try "SysOpenEventSem()"
call try "SysOpenEventSem(,)"
call try "SysOpenEventSem('', 1)"
call try "SysOpenEventSem(, 1)"
call try "SysOpenEventSem(.nil, 1)"
call try "SysOpenEventSem(.nil)"
call try "SysOpenEventSem('')"
call try "SysOpenEventSem(.mutablebuffer~new)"
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

r[util.syseventsem.pulse]
SysPulseEventSem is not provided. A call, with any arguments, raises `SYNTAX
43.1` as a call of a name that no routine has does, and
[RXFUNCQUERY](r:bif.rxfuncquery.answer) answers `1` for it, where it answers
`0` for the other functions of this page.

```rexx
h = SysCreateEventSem()
call try "SysPulseEventSem(h)"
call try "SysPulseEventSem()"
call try "SysPulseEventSem('x', 1, 2)"
say rxfuncquery('SysPulseEventSem') rxfuncquery('SysPostEventSem') rxfuncquery('SysWaitEventSem')
say rxfuncquery('SysCreateEventSem') rxfuncquery('SysOpenEventSem') rxfuncquery('SysCloseEventSem')
call SysCloseEventSem h
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
43.1
43.1
43.1
1 0 0
0 0 0
```

## Deviations

r[util.syseventsem.deviation.20]
A call of SysWaitEventSem that has no limit, and finds no post, while no other
activity or timer of the program can post the semaphore, ends the program at
once, with exit code `120` and a message on the error stream ([Deviation
20](../deviations.md#20-a-wait-nothing-can-end)). ooRexx waits for ever.

```rexx
say 'before'
h = SysCreateEventSem()
say SysWaitEventSem(h)
say 'after'
```

```output rc=120
before
```

r[util.syseventsem.deviation.43]
A call of SysCreateEventSem with a first argument that is not the null string,
and a call of SysOpenEventSem with a name that is not the null string, end the
program at once, with exit code `120` and a message on the error stream: the
semaphore is not made or opened, and nothing that follows the call runs.
ooRexx makes a semaphore of that name that every program of the system can
open, and answers its handle; SysOpenEventSem answers `0` for a name that no
program has made. The first example is a creating call, the second an opening
one.

```rexx
say 'before'
h = SysCreateEventSem('c5zqevent')
say 'after'
```

```output rc=120
before
```

```rexx
say 'before'
h = SysOpenEventSem('c5zqevent')
say 'after'
```

```output rc=120
before
```

r[util.syseventsem.deviation.158]
A whole number that is not the handle of an open semaphore makes every
function of this page and of [the mutex semaphore functions](sysmutexsem.md)
that takes a handle answer `6`, and use no semaphore: the handle of a
semaphore that is closed, a handle that arithmetic has rounded, as the sum
`h + 0` of a handle of thirteen digits is under `NUMERIC DIGITS 9`, and a
number that no call answered. ooRexx uses the number as the address of the
semaphore and crashes, with a signal that ends the program, for a closed
handle (all calls but SysCloseEventSem, which answers `6`) and for a rounded
one. The first example is a rounded handle and a closed one, the second a
number that no call answered. SysResetEventSem answers `0` in interrexx for
such a handle, where it should answer `6` like the others, as the third
example shows.

```rexx
h = SysCreateEventSem()
r = h + 0
say (r \== h) SysPostEventSem(r) SysWaitEventSem(r, 10) SysCloseEventSem(r)
call SysCloseEventSem h
say SysPostEventSem(h) SysWaitEventSem(h, 10)
```

```output
1 6 6 6
6 6
```

```rexx
say SysPostEventSem(12345) SysWaitEventSem(12345, 10) SysCloseEventSem(12345)
```

```output
6 6 6
```

```rexx
h = SysCreateEventSem()
call SysCloseEventSem h
say SysResetEventSem(h)
```

```output defect=#236
6
```
