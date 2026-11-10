---
element: "EventSemaphore"
kind: class
deviations: [109]
rexxref: "Builtin Classes > Utility Classes > EventSemaphore Class; Builtin Classes > Utility Classes > EventSemaphore Class > new (Class Method); Builtin Classes > Utility Classes > EventSemaphore Class > isPosted; Builtin Classes > Utility Classes > EventSemaphore Class > post; Builtin Classes > Utility Classes > EventSemaphore Class > reset; Builtin Classes > Utility Classes > EventSemaphore Class > uninit; Builtin Classes > Utility Classes > EventSemaphore Class > wait"
---

# EventSemaphore

An `EventSemaphore` is a flag that activities can wait for. It is either
posted or cleared. An activity that has to wait for something another activity
does calls `wait`; the activity that does it calls `post`, and every activity
that waits goes on. A semaphore that is posted lets `wait` return at once,
until it is cleared again with `reset`. This page states what each method
answers and checks, and what a timeout may be. How activities are scheduled,
and so which of several waiters runs first after a `post`, is specified in the
environment chapter. The examples that use two activities start a message and
wait on the semaphore from the program; the order of their output is forced by
the semaphore itself.

## The class

r[class.eventsemaphore.creation]
`EventSemaphore` has the instance methods `isPosted`, `post`, `reset`,
`uninit` and `wait`, and the class method `new`, besides the methods that
every class has. It may have private methods besides these; they are not part
of this specification. Its only superclass is `Object` and it inherits no
mixin class ([the hierarchy rule](r:class.index.hierarchy)).

```rexx
say .eventsemaphore~id .eventsemaphore~superclasses~items,
  .eventsemaphore~superclass~id .eventsemaphore~queryMixinClass
have = .set~new
s = .eventsemaphore~methods(.nil)
do while s~available
  have~put(s~index)
  s~next
end
names = 'isPosted post reset uninit wait'~makeArray(' ')
count = 0
do name over names
  if have~hasItem(name~upper) then count = count + 1
end
say count 'of 5'
```

```output
EventSemaphore 1 Object 0
5 of 5
```

r[class.eventsemaphore.new]
`new` makes a semaphore that is cleared, and answers an instance of the class
that received it, so a subclass makes instances of itself. It makes the
semaphore itself and then sends `init` the arguments of `new`, as [the new
rule](r:class.index.new) says, so the semaphore works whatever the `init` of a
subclass does, even when it does not send `init` to its superclass, and `new`
with an argument raises `SYNTAX 93.902` for the class itself, where `init` is
that of `Object`, and not for a subclass whose `init` takes it.

```rexx
say .eventsemaphore~new~isPosted
s = .count~new(5)
say s~tag s~isPosted s~wait(0)
s~post
say s~isPosted s~wait(0)
t = .lazy~new
say t~class~id t~isPosted t~wait(0)
t~post
say t~isPosted t~wait(0)
signal on syntax name bad
x = .eventsemaphore~new(1)
bad: say condition('o')~code
::class count subclass eventsemaphore
::method init
  expose tag
  use arg tag
  self~init:super
::method tag
  expose tag
  return tag
::class lazy subclass eventsemaphore
::method init
  nop
```

```output
0
5 0 0
1 1
LAZY 0 0
1 1
93.902
```

## States

r[class.eventsemaphore.isposted]
`isPosted` answers 1 when the semaphore is posted and 0 when it is cleared. It
takes no argument (`SYNTAX 93.902`) and sends no message.

```rexx
e = .eventsemaphore~new
say e~isPosted
e~post
say e~isPosted
e~reset
say e~isPosted
signal on syntax name bad
say e~isPosted(1)
bad: say condition('o')~code
```

```output
0
1
0
93.902
```

r[class.eventsemaphore.post]
`post` makes the semaphore posted and releases every activity that waits for
it; a call of `wait` that comes after it, while the semaphore is posted,
returns without waiting. Posting a semaphore that is posted changes nothing.
`post` answers nothing, so using its answer as a value raises `SYNTAX 91.999`,
and takes no argument (`SYNTAX 93.902`).

```rexx
e = .eventsemaphore~new
e~post
say e~isPosted e~wait(0)
e~post
say e~isPosted e~wait(0)
signal on syntax name bad
x = e~post
bad: say condition('o')~code
signal on syntax name bad2
e~post(1)
bad2: say condition('o')~code
```

```output
1 1
1 1
91.999
93.902
```

r[class.eventsemaphore.reset]
`reset` makes the semaphore cleared; a call of `wait` after it waits for the
next `post`. Resetting a semaphore that is cleared changes nothing. `reset`
answers nothing, so using its answer as a value raises `SYNTAX 91.999`, and
takes no argument (`SYNTAX 93.902`).

```rexx
e = .eventsemaphore~new
e~reset
say e~isPosted e~wait(0)
e~post
e~reset
say e~isPosted e~wait(0.05)
e~reset
say e~isPosted
signal on syntax name bad
x = e~reset
bad: say condition('o')~code
signal on syntax name bad2
e~reset(1)
bad2: say condition('o')~code
```

```output
0 0
0 0
0
91.999
93.902
```

r[class.eventsemaphore.copy]
`copy` answers a new semaphore in the state the original is in: posted when it
is posted, cleared when it is cleared. From then on the two are separate: a
`post` or `reset` of one does not change the other, and a `wait` on one is not
ended by a `post` of the other.

```rexx
e = .eventsemaphore~new
e~post
k = e~copy
say k~isPosted k~wait(0)
e~reset
say k~isPosted e~isPosted
k~reset
e~post
say k~isPosted e~isPosted k~wait(0.05)
c = .eventsemaphore~new~copy
say c~isPosted c~wait(0)
```

```output defect=#182
1 1
1 0
0 1 0
0 0
```

## Waiting

r[class.eventsemaphore.wait]
`wait(timeout)` answers 1 when the semaphore is posted, at the call or before
the timeout ends, and 0 when the timeout ended with the semaphore still cleared.
The timeout says how long the caller is prepared to wait. Omitted or below zero,
there is no limit, so the answer can only be 1. Zero is no wait at all, and the
answer is the state at the call, as `isPosted` gives it. Above zero it is a
limit in seconds. One `post` ends the wait of every caller that is waiting,
however many there are; which of them runs on first is not specified. A call of
`wait` that is released by `post` and one that finds the semaphore posted answer
alike. The timeout is read before the semaphore is looked at, as
[the next rule](r:class.eventsemaphore.wait.timeout) says, so an invalid one
raises its error even for a posted semaphore. In the example a message that the
program starts posts after a short pause, and a second starts two waiters, one
without a timeout and one with a long one that never ends before the post; the
program waits for the post too, and prints what each answered, which the post
forces to be 1 for all, and what `wait(0)` and a short timeout answer on a
cleared semaphore.

```rexx
e = .eventsemaphore~new
say e~wait(0) e~wait(0.05)
w1 = .message~new(e, 'wait')~~start
w2 = .message~new(e, 'wait', 'I', 30)~~start
w3 = .message~new(.poster, 'later', 'I', e)~~start
say 'blocking wait:' e~wait
say w1~result w2~result w3~result
say e~isPosted
say e~wait e~wait(0) e~wait(-1) e~wait(5)
e~reset
say e~isPosted e~wait(0.05)
p = .message~new(.poster, 'later', 'I', e)~~start
say p~result e~wait(30)
::class poster
::method later class
  use arg sem
  call syssleep 0.05
  sem~post
  return 'posted'
```

```output
0 0
blocking wait: 1
1 1 posted
1
1 1 1 1
0 0
posted 1
```

r[class.eventsemaphore.wait.timeout]
The timeout is a number of seconds or a `TimeSpan`, and it is read before
anything else is done, so an invalid one raises `SYNTAX 88.902` whatever the
state of the semaphore. A string is a valid timeout when it is a number as
[the DATATYPE `N` rule](r:bif.datatype.number) reads it, so blanks at either
end, a sign, a decimal point and an exponent are allowed; the empty string, a
string of blanks and a string that is no number raise `SYNTAX 88.902`. An object
that is neither a string nor a `TimeSpan` is replaced by its string value, as
[the conversion rule](r:conv.string.object) says, and that string is read in the
same way; `.nil`, a `DateTime` and an object whose string value is no number,
such as an instance of `Object`, raise `SYNTAX 88.902`. A `TimeSpan` is read
through its `totalSeconds`, which is sent to it once, and a span that is
negative means the same as a negative number. More than one argument raises
`SYNTAX 93.902`. The first example gives each form to a cleared semaphore with a
short timeout, and to a posted one. In the second, a `TimeSpan` that is negative
waits as long as a negative number does, until the post, which comes from
another activity.

```rexx
e = .eventsemaphore~new
say e~wait(' 0.05 ') e~wait('+.05') e~wait('5e-2'),
  e~wait(.timespan~fromMicroseconds(50000)) e~wait(.mutablebuffer~new('0.05'))
say e~wait(.timespan~new(0)) e~wait(0) e~wait('0')
call try "e~wait('')"
call try "e~wait(' ')"
call try "e~wait('abc')"
call try "e~wait('0.05x')"
call try "e~wait(.nil)"
call try "e~wait(.object~new)"
call try "e~wait(.datetime~new)"
call try "e~wait(1, 2)"
call try "e~wait(, 1)"
e~post
call try "e~wait('')"
call try "e~wait(.nil)"
call try "e~wait(1, 2)"
call try "e~wait"
call try "e~wait(-5)"
call try "e~wait(.timespan~fromSeconds(-5))"
call try "e~wait(.timespan~fromSeconds(5))"
call try "e~wait('1e5')"
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=>' x
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
0 0 0 0 0
0 0 0
e~wait('') => 88.902
e~wait(' ') => 88.902
e~wait('abc') => 88.902
e~wait('0.05x') => 88.902
e~wait(.nil) => 88.902
e~wait(.object~new) => 88.902
e~wait(.datetime~new) => 88.902
e~wait(1, 2) => 93.902
e~wait(, 1) => 93.902
e~wait('') => 88.902
e~wait(.nil) => 88.902
e~wait(1, 2) => 93.902
e~wait => 1
e~wait(-5) => 1
e~wait(.timespan~fromSeconds(-5)) => 1
e~wait(.timespan~fromSeconds(5)) => 1
e~wait('1e5') => 1
```

```rexx
e = .eventsemaphore~new
p = .message~new(.poster, 'later', 'I', e)~~start
say e~wait(.timespan~fromSeconds(-5)) p~result
::class poster
::method later class
  use arg sem
  call syssleep 0.05
  sem~post
  return 'posted'
```

```output
1 posted
```

r[class.eventsemaphore.wait.sends]
`wait` sends the message `totalSeconds` to a timeout that is a `TimeSpan`, once
and before it looks at the semaphore, and no other message to a `TimeSpan`; an
object that is neither a string nor a `TimeSpan` is sent what
[the conversion rule](r:conv.string.object) says
([Deviation 109](r:class.string.deviation.109) says that a string that is an
instance of a subclass of `String` is read by its own characters). No method of
the class sends a message to the semaphore itself: an override of `post`,
`reset`, `isPosted` or `wait` runs when the program sends that message and not
because another of them was. The first example overrides each method and
`totalSeconds` of a subclass of `TimeSpan` and shows what runs; the second gives
a timeout whose class has its own `request` and `makeString`, and only `request`
is sent.

```rexx
.environment~log = .list~new
e = .watch~new
span = .span~fromMicroseconds(20000)
.environment~log~empty
x = e~wait(span)
say 'wait cleared:' x .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
e~post
say 'post:' .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
x = e~wait(span)
say 'wait posted:' x .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
x = e~wait('0.01')
say 'wait string:' x .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
e~reset
say 'reset:' .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
x = e~isPosted
say 'isPosted:' x .environment~log~makeArray~makeString('l', ' ')
::class watch subclass eventsemaphore
::method wait
  .environment~log~append('wait')
  forward class (super)
::method post
  .environment~log~append('post')
  forward class (super)
::method reset
  .environment~log~append('reset')
  forward class (super)
::method isPosted
  .environment~log~append('isPosted')
  forward class (super)
::class span subclass timespan
::method totalSeconds
  .environment~log~append('totalSeconds')
  forward class (super)
```

```output
wait cleared: 0 wait totalSeconds
post: post
wait posted: 1 wait totalSeconds
wait string: 1 wait
reset: reset
isPosted: 0 isPosted
```

```rexx
e = .eventsemaphore~new
e~post
.environment~log = 'sent:'
say e~wait(.timeout~new) .log
::class timeout
::method request
  .environment~log = .log 'request' arg(1)
  return '0.05'
::method makestring
  .environment~log = .log 'makeString'
  return '0.05'
```

```output defect=#164
1 sent: request STRING
```

## Cleaning up

r[class.eventsemaphore.uninit]
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
s2 = .eventsemaphore~new
x = s2~uninit
bad: say condition('o')~code
signal on syntax name bad2
s3 = .eventsemaphore~new
s3~uninit(1)
bad2: say condition('o')~code
::class tidy subclass eventsemaphore
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

r[class.eventsemaphore.deviation.109]
A timeout that is an instance of a subclass of `String` is read by its own
characters, and `request`, `makeString` and `string` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an instance
`request` with `STRING`, so an override can change what is read. In the
example the overrides would make the timeout invalid; they count their sends,
and the string is read as 0.01 seconds without one.

```rexx
.environment~sent = 0
e = .eventsemaphore~new
say e~wait(.digits~new('0.01')) 'sends' .environment~sent
::class digits subclass string
::method makestring
  .environment~sent = .environment~sent + 1
  return 'abc'
::method request
  .environment~sent = .environment~sent + 1
  return 'abc'
```

```output defect=#140
0 sends 0
```
