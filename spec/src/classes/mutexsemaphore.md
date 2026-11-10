---
element: "MutexSemaphore"
kind: class
deviations: [109]
rexxref: "Builtin Classes > Utility Classes > MutexSemaphore Class; Builtin Classes > Utility Classes > MutexSemaphore Class > new (Class Method); Builtin Classes > Utility Classes > MutexSemaphore Class > acquire; Builtin Classes > Utility Classes > MutexSemaphore Class > release; Builtin Classes > Utility Classes > MutexSemaphore Class > uninit"
---

# MutexSemaphore

A `MutexSemaphore` lets one activity at a time own something. An activity that
is about to enter a part of the program that only one activity may be in at a
time calls `acquire`; when it leaves it calls `release`. While one activity
owns the semaphore, another that calls `acquire` waits for the owner to let
go, or gives up after a timeout. An activity that owns the semaphore can
acquire it again without waiting, and then has to release it as many times.
This page states what each method answers and checks, how the count of
acquisitions works and what a timeout may be. How activities are scheduled,
and so which of several that wait gets the semaphore next, is specified in the
environment chapter. The examples that use two activities start a message from
the program and use event semaphores to force the order of what happens, so
that nothing in their output depends on speed.

## The class

r[class.mutexsemaphore.creation]
`MutexSemaphore` has the instance methods `acquire`, `release` and `uninit`,
and the class method `new`, besides the methods that every class has. It may
have private methods besides these; they are not part of this specification.
Its only superclass is `Object` and it inherits no mixin class ([the hierarchy
rule](r:class.index.hierarchy)).

```rexx
say .mutexsemaphore~id .mutexsemaphore~superclasses~items,
  .mutexsemaphore~superclass~id .mutexsemaphore~queryMixinClass
have = .set~new
s = .mutexsemaphore~methods(.nil)
do while s~available
  have~put(s~index)
  s~next
end
names = 'acquire release uninit'~makeArray(' ')
count = 0
do name over names
  if have~hasItem(name~upper) then count = count + 1
end
say count 'of 3'
```

```output
MutexSemaphore 1 Object 0
3 of 3
```

r[class.mutexsemaphore.new]
`new` makes a semaphore that no activity owns, and answers an instance of the
class that received it, so a subclass makes instances of itself. It makes the
semaphore itself and then sends `init` the arguments of `new`, as [the new
rule](r:class.index.new) says, so the semaphore works whatever the `init` of a
subclass does, even when it does not send `init` to its superclass, and `new`
with an argument raises `SYNTAX 93.902` for the class itself, where `init` is
that of `Object`, and not for a subclass whose `init` takes it.

```rexx
m = .mutexsemaphore~new
say m~release m~acquire(0) m~release
s = .count~new(6)
say s~tag s~acquire(0) s~release
t = .lazy~new
say t~class~id t~acquire(0) t~release t~release
signal on syntax name bad
x = .mutexsemaphore~new(1)
bad: say condition('o')~code
::class count subclass mutexsemaphore
::method init
  expose tag
  use arg tag
  self~init:super
::method tag
  expose tag
  return tag
::class lazy subclass mutexsemaphore
::method init
  nop
```

```output
0 1 1
6 1 1
LAZY 1 1 0
93.902
```

## Owning

r[class.mutexsemaphore.acquire]
`acquire(timeout)` gives the activity that calls it ownership, and answers 1,
when no activity owns the semaphore and when the activity already owns it. It
answers 0 when another activity owns it and keeps it for the whole timeout. The
semaphore counts how many times its owner has acquired it: a call from the owner
does not wait, adds one to the count and answers 1, and the semaphore becomes
free only when [`release`](r:class.mutexsemaphore.release) has taken the count
back to zero. The timeout bounds the wait of a caller that does not get the
semaphore at once. Omitted or below zero, there is no bound, so the answer can
only be 1. Zero means no wait: the answer says whether the semaphore was free or
the caller's already. Above zero it is a bound in seconds. When the owner lets
go, one of the activities that wait gets the semaphore, which of them is not
specified. The timeout is read as
[the next rule](r:class.mutexsemaphore.timeout) says, before the semaphore is
looked at, so an invalid one raises its error whoever owns the semaphore.
Ownership does not outlive the activity: when an activity ends, every count it
held is dropped and the semaphore is free for others. In the first example one
activity does all the calls, and in the second a message that the program starts
owns the semaphore while the program calls.

```rexx
m = .mutexsemaphore~new
say m~acquire m~acquire(0) m~acquire(0.05) m~acquire('0.05') m~acquire(-1)
say m~release m~release m~release m~release m~release m~release
say m~acquire(5) m~acquire(.timespan~fromSeconds(1)),
  m~release m~release m~release
```

```output
1 1 1 1 1
1 1 1 1 1 0
1 1 1 1 0
```

```rexx
m = .mutexsemaphore~new
got = .eventsemaphore~new
go = .eventsemaphore~new
h = .holder~new(m, got, go)
r = h~start('run')
say 'held by other:' got~wait
say m~acquire(0) m~release m~acquire(0.05) m~release
go~post
say 'main acquires:' m~acquire
say m~release m~release
say r~result
-- an activity that ends while it owns the semaphore gives it up, however
-- often it acquired it
h2 = .holder~new(m, got, go)
r2 = h2~start('runAndKeep')
say r2~result
say m~acquire(0) m~release
::class holder
::method init
  expose m got go
  use arg m, got, go
::method run unguarded
  expose m got go
  a = m~acquire
  got~post
  go~wait
  return 'other:' a m~release
::method runAndKeep unguarded
  expose m
  return 'keep acquires:' m~acquire m~acquire m~acquire
```

```output
held by other: 1
0 0 0 0
main acquires: 1
1 0
other: 1 1
keep acquires: 1 1 1
1 1
```

r[class.mutexsemaphore.release]
`release` answers 1 when the activity that calls it owns the semaphore, and
takes one off the count of its acquisitions; when the count reaches zero the
semaphore is free, and one of the activities that wait for it gets it. It
answers 0 when the activity does not own the semaphore, because no activity
does or because another does, and changes nothing. It takes no argument
(`SYNTAX 93.902`) and sends no message. In the example the program owns the
semaphore twice, a message that it starts releases it without owning it and
answers 0, and only the program's own two releases free it.

```rexx
m = .mutexsemaphore~new
say m~release
say m~acquire m~acquire
other = .message~new(m, 'release')~~start
say other~result
say m~release m~release m~release
other = .message~new(m, 'acquire', 'I', 0)~~start
say other~result
say m~release
signal on syntax name bad
say m~release(1)
bad: say condition('o')~code
```

```output
0
1 1
0
1 1 0
1
0
93.902
```

r[class.mutexsemaphore.copy]
`copy` answers a new semaphore owned by the same activity with the same count
as the original, or by none when the original is free. From then on the two are
separate: releasing one does not release the other. When the owning activity
ends, only the semaphores it acquired itself are freed: a copy that it owns
stays owned, so no activity can acquire it. The first example has one activity;
in the second, a message that the program starts owns the semaphore when the
program copies it, and ends owning both.

```rexx
m = .mutexsemaphore~new
say m~acquire(0) m~acquire(0)
j = m~copy
say j~release j~release j~release
say m~release m~release m~release
f = .mutexsemaphore~new~copy
say f~release f~acquire(0) f~release
```

```output defect=#182
1 1
1 1 0
1 1 0
0 1 1
```

```rexx
m = .mutexsemaphore~new
got = .eventsemaphore~new
go = .eventsemaphore~new
r = .holder~new~start('run', m, got, go)
say 'held by other:' got~wait(5)
j = m~copy
say 'copy free here:' j~acquire(0)
go~post
say r~result
say 'after its end:' m~acquire(0) m~release j~acquire(0)
::class holder
::method run unguarded
  use arg m, got, go
  a = m~acquire
  got~post
  go~wait(5)
  return 'other ends owning both:' a
```

```output defect=#182
held by other: 1
copy free here: 0
other ends owning both: 1
after its end: 1 1 0
```

r[class.mutexsemaphore.timeout]
The timeout is a number of seconds or a `TimeSpan`, and it is read before
anything else is done, so an invalid one raises `SYNTAX 88.902` whoever owns the
semaphore. A string is a valid timeout when it is a number as
[the DATATYPE `N` rule](r:bif.datatype.number) reads it, so blanks at either
end, a sign, a decimal point and an exponent are allowed; the empty string, a
string of blanks and a string that is no number raise `SYNTAX 88.902`. An object
that is neither a string nor a `TimeSpan` is replaced by its string value, as
[the conversion rule](r:conv.string.object) says, and that string is read in the
same way; `.nil`, a `DateTime` and an object whose string value is no number,
such as an instance of `Object`, raise `SYNTAX 88.902`. A `TimeSpan` is read
through its `totalSeconds`, which is sent to it once, and a span that is
negative means the same as a negative number. More than one argument raises
`SYNTAX 93.902`. The example gives each form to a semaphore that nothing owns,
and counts the acquisitions it made, to free the semaphore.

```rexx
m = .mutexsemaphore~new
held = 0
call try "m~acquire(' 0.05 ')"
call try "m~acquire('+.05')"
call try "m~acquire('5e-2')"
call try "m~acquire(.timespan~fromMicroseconds(50000))"
call try "m~acquire(.timespan~fromSeconds(-5))"
call try "m~acquire(-0.5)"
call try "m~acquire(.mutablebuffer~new('0.05'))"
call try "m~acquire('')"
call try "m~acquire(' ')"
call try "m~acquire('abc')"
call try "m~acquire('0.05x')"
call try "m~acquire(.nil)"
call try "m~acquire(.object~new)"
call try "m~acquire(.datetime~new)"
call try "m~acquire(1, 2)"
call try "m~acquire(, 1)"
do held
  x = m~release
end
say m~release
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  held = held + 1
  say arg(1) '=>' x
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
m~acquire(' 0.05 ') => 1
m~acquire('+.05') => 1
m~acquire('5e-2') => 1
m~acquire(.timespan~fromMicroseconds(50000)) => 1
m~acquire(.timespan~fromSeconds(-5)) => 1
m~acquire(-0.5) => 1
m~acquire(.mutablebuffer~new('0.05')) => 1
m~acquire('') => 88.902
m~acquire(' ') => 88.902
m~acquire('abc') => 88.902
m~acquire('0.05x') => 88.902
m~acquire(.nil) => 88.902
m~acquire(.object~new) => 88.902
m~acquire(.datetime~new) => 88.902
m~acquire(1, 2) => 93.902
m~acquire(, 1) => 93.902
0
```

r[class.mutexsemaphore.sends]
`acquire` sends the message `totalSeconds` to a timeout that is a `TimeSpan`,
once and before it looks at the semaphore, and no other message to a `TimeSpan`;
an object that is neither a string nor a `TimeSpan` is sent what
[the conversion rule](r:conv.string.object) says
([Deviation 109](r:class.string.deviation.109) says that a string that is an
instance of a subclass of `String` is read by its own characters). No method of
the class sends a message to the semaphore itself: an override of `acquire` or
`release` runs when the program sends that message and not because the other
was. The first example overrides both methods and `totalSeconds` of a subclass
of `TimeSpan` and shows what runs; the second gives a timeout whose class has
its own `request` and `makeString`, and only `request` is sent.

```rexx
.environment~log = .list~new
m = .watch~new
span = .span~fromMicroseconds(20000)
.environment~log~empty
x = m~acquire(span)
say 'acquire:' x .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
x = m~acquire('0.01')
say 'acquire string:' x .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
x = m~release
say 'release:' x .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
x = m~release
say 'release again:' x .environment~log~makeArray~makeString('l', ' ')
::class watch subclass mutexsemaphore
::method acquire
  .environment~log~append('acquire')
  forward class (super)
::method release
  .environment~log~append('release')
  forward class (super)
::class span subclass timespan
::method totalSeconds
  .environment~log~append('totalSeconds')
  forward class (super)
```

```output
acquire: 1 acquire totalSeconds
acquire string: 1 acquire
release: 1 release
release again: 1 release
```

```rexx
m = .mutexsemaphore~new
.environment~log = 'sent:'
say m~acquire(.timeout~new) m~release .log
::class timeout
::method request
  .environment~log = .log 'request' arg(1)
  return '0.05'
::method makestring
  .environment~log = .log 'makeString'
  return '0.05'
```

```output defect=#164
1 1 sent: request STRING
```

## Cleaning up

r[class.mutexsemaphore.uninit]
`uninit` frees what the semaphore holds. The library sends it when the semaphore
is reclaimed, at a time that a program does not control, which is not specified.
An `uninit` of a subclass replaces this one, so it frees the semaphore only by
sending `self~uninit:super`. `uninit` takes no argument (`SYNTAX 93.902`) and
answers nothing, so using its answer as a value raises `SYNTAX 91.999`.

```rexx
.environment~log = .list~new
s = .tidy~new
s~uninit
say .environment~log~makeArray~makeString('l', ' ')
signal on syntax name bad
s2 = .mutexsemaphore~new
x = s2~uninit
bad: say condition('o')~code
signal on syntax name bad2
s3 = .mutexsemaphore~new
s3~uninit(1)
bad2: say condition('o')~code
::class tidy subclass mutexsemaphore
::method uninit
  .environment~log~append('tidy-uninit')
  self~uninit:super
```

```output
tidy-uninit
91.999
93.902
```

## Deviations

r[class.mutexsemaphore.deviation.109]
A timeout that is an instance of a subclass of `String` is read by its own
characters, and `request`, `makeString` and `string` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an instance
`request` with `STRING`, so an override can change what is read. In the
example the overrides would make the timeout invalid; they count their sends,
and the string is read as 0.01 seconds without one.

```rexx
.environment~sent = 0
m = .mutexsemaphore~new
say m~acquire(.digits~new('0.01')) 'sends' .environment~sent
::class digits subclass string
::method makestring
  .environment~sent = .environment~sent + 1
  return 'abc'
::method request
  .environment~sent = .environment~sent + 1
  return 'abc'
```

```output defect=#140
1 sends 0
```
