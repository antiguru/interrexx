---
element: "Ticker"
kind: class
deviations: [109]
rexxref: "Builtin Classes > Utility Classes > Ticker Class; Builtin Classes > Utility Classes > Ticker Class > attachment; Builtin Classes > Utility Classes > Ticker Class > cancel; Builtin Classes > Utility Classes > Ticker Class > canceled/cancelled; Builtin Classes > Utility Classes > Ticker Class > init; Builtin Classes > Utility Classes > Ticker Class > interval"
---

# Ticker

A `Ticker` is a repeating timer. It is made for an interval, a target that
wants to hear of it and, if the program likes, an object to attach; at the end
of every interval it sends its target `triggered`, until it is cancelled, and
when it is cancelled it sends the target `cancel`. The target is an
implementation of [`AlarmNotification`](alarmnotification.md), which states
what the target is sent and when. A ticker is made as an [`Alarm`](alarm.md)
is, with an interval in place of a time; it has no `scheduledTime` and no
`triggered` of its own, and it asks for the target and the attachment as an
alarm does. This page states how a ticker is made, what its interval may be
and what its own methods answer. How the library runs a ticker beside the
program, so at which times the target is reached and by which activity, is
specified in the environment chapter, and so is whether a `triggered` that is
still running delays the next one. The examples that wait use a guard that the
target opens, and print nothing that varies with the clock.

## The class

r[class.ticker.creation]
`Ticker` has the instance methods `attachment`, `cancel`, `canceled`,
`cancelled`, `init` and `interval`, besides the `new` that every class has. It
may have private methods besides these; they are not part of this
specification. Its only superclass is `Object`, it inherits no mixin class and
is not itself a mixin class ([the hierarchy rule](r:class.index.hierarchy)),
and so it is no implementation of `AlarmNotification` and cannot be the target
of an alarm. `new` is the one [that `Class` supplies](r:class.index.new): it
sends `init` the arguments of `new`, which [the next
rule](r:class.ticker.init) states, and answers an instance of the class that
received it, so a subclass of `Ticker` makes instances of itself.

```rexx
say .ticker~superclasses~items .ticker~superclass~id .ticker~queryMixinClass
have = .set~new
s = .ticker~methods(.nil)
do while s~available
  have~put(s~index)
  s~next
end
names = 'attachment cancel canceled cancelled'
names = (names 'init interval')~makeArray(' ')
count = 0
do name over names
  if have~hasItem(name~upper) then count = count + 1
end
say count 'of 6' have~hasItem('TRIGGERED') have~hasItem('SCHEDULEDTIME')
n = .note~new
a = .ticker~new(60, n)
b = .sub~new(60, n)
say (a~class == .ticker) (b~class == .sub) a~isInstanceOf(.alarmnotification)
signal on syntax name bad
c = .alarm~new(60, a)
bad: say condition('o')~code
a~cancel
b~cancel
::class note inherit AlarmNotification
::method triggered
  nop
::class sub subclass ticker
```

```output
1 Object 0
6 of 6 0 0
1 1 0
88.914
```

## Making a ticker

r[class.ticker.init]
`new` sends `init(interval, target, attachment)`, and `init` sets the ticker up.
It needs the first two arguments, so `new` with none or only the first raises
`SYNTAX 93.901` and an omitted first or second argument raises `SYNTAX 93.903`;
a fourth argument raises `SYNTAX 93.902`. The checks of the arguments that are
present are made in this order: first the target, which must be an
implementation of [`AlarmNotification`](r:class.alarmnotification.creation) or
`SYNTAX 88.914` is raised, and then the interval, which raises `SYNTAX 93.951`
for every value that [the rule below](r:class.ticker.init.interval) does not
accept, except the very large ones for which it names other errors. So a target
that is no implementation is reported even when the interval is wrong too, and a
refused ticker is not set. `attachment` can be any object, and is `.nil` when it
is omitted. The ticker is set when `init` ends normally.

```rexx
n = .note~new
call try ".ticker~new"
call try ".ticker~new(60)"
call try ".ticker~new(, n)"
call try ".ticker~new(60, , 'x')"
call try ".ticker~new(60, n, 'x', 'y')"
call try ".ticker~new('abc', .nil)"
call try ".ticker~new('abc', n)"
call try ".ticker~new(60, n)"
call try ".ticker~new(60, n, .nil)"
call try ".ticker~new(60, n, .array~new)"
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=> ok'
  x~cancel
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class note inherit AlarmNotification
::method triggered
  nop
```

```output
.ticker~new => 93.901
.ticker~new(60) => 93.901
.ticker~new(, n) => 93.903
.ticker~new(60, , 'x') => 93.903
.ticker~new(60, n, 'x', 'y') => 93.902
.ticker~new('abc', .nil) => 88.914
.ticker~new('abc', n) => 93.951
.ticker~new(60, n) => ok
.ticker~new(60, n, .nil) => ok
.ticker~new(60, n, .array~new) => ok
```

r[class.ticker.init.interval]
An interval that is a string is a number of seconds. It is read as a number as
[the DATATYPE `N` rule](r:bif.datatype.number) reads it, so blanks at either end
are allowed and a decimal point or an exponent is too, and it has to be zero or
more: a string that is no number, the empty string and a negative number raise
`SYNTAX 93.951`. An interval that is a `TimeSpan` is that length, and must not
be negative. An interval longer than the longest `TimeSpan`, 315537897599.999999
seconds, raises `SYNTAX 40.19`, whether it is positive or negative. A number
that counts 10**12 seconds or more, positive or negative, once it is rounded as
said below, raises `SYNTAX 88.903`. Nothing else is an interval, so
`SYNTAX 93.951` is raised for a `DateTime`, for a string with a time of day such
as `12:00:00`, for `.nil`, for an array and for an instance of `Object`. The
interval is kept to the microsecond: a longer fraction is cut off and not
rounded, and the number is cut before its sign is checked, so `-0.0000001` is an
interval of zero; a number with more than 18 significant digits is first rounded
to 18, so `1.999999999999999999` is 2 seconds. The number is read in full
whatever the NUMERIC DIGITS setting. `interval` answers the interval, as
[the next rule](r:class.ticker.interval) says. The example makes tickers whose
intervals are too long to end before they are cancelled.

```rexx
n = .note~new
call try 'string " 90 "', ' 90 '
call try 'string 1.5e2', '1.5e2'
call try 'string +.5e3', '+.5e3'
call try 'empty string', ''
call try 'string 1x', '1x'
call try 'string -1', '-1'
call try 'string -0.0000001', '-0.0000001'
call try 'string 1.999999999999999999', '1.999999999999999999'
call try 'string 315537897600', '315537897600'
call try 'string -315537897600', '-315537897600'
call try 'string 1e12', '1e12'
call try 'string -1e12', '-1e12'
call try 'string 12:00:00', '12:00:00'
call try 'span of 600 s', .timespan~fromSeconds(600)
call try 'span of -600 s', .timespan~fromSeconds(-600)
call try 'DateTime', .datetime~new~addSeconds(600)
call try '.nil', .nil
call try 'array', .array~new
call try 'object', .object~new
numeric digits 3
call try 'digits 3', '123456.789'
exit
try:
  signal on syntax name bad
  t = .ticker~new(arg(2), n)
  say arg(1) '=> ok' t~interval
  t~cancel
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class note inherit AlarmNotification
::method triggered
  nop
```

```output
string " 90 " => ok 00:01:30.000000
string 1.5e2 => ok 00:02:30.000000
string +.5e3 => ok 00:08:20.000000
empty string => 93.951
string 1x => 93.951
string -1 => 93.951
string -0.0000001 => ok 00:00:00.000000
string 1.999999999999999999 => ok 00:00:02.000000
string 315537897600 => 40.19
string -315537897600 => 40.19
string 1e12 => 88.903
string -1e12 => 88.903
string 12:00:00 => 93.951
span of 600 s => ok 00:10:00.000000
span of -600 s => 93.951
DateTime => 93.951
.nil => 93.951
array => 93.951
object => 93.951
digits 3 => ok 1.10:17:36.789000
```

r[class.ticker.init.sends]
`init` sends messages that a program can override, in this order: first `isA`
to the target, with `AlarmNotification` as its one argument, once; the answer
decides whether the target is an implementation. Then, for an interval that is
a `TimeSpan`, the span is sent `sign`, `days` and `totalMicroseconds`, once
each and in that order, and for an interval that is a string nothing is sent
([Deviation 109](r:class.ticker.deviation.109) covers a subclass of `String`).
When the span is negative `init` stops after `sign`; the further messages that
build the text of the error are not specified. A ticker that is refused for
its target sends the interval nothing. `init` sends no message to the
attachment, and none to the ticker itself, so an override of `attachment` or
`interval` does not run. The example logs the sends of an override of each
message in a subclass of `TimeSpan`.

```rexx
.environment~log = .list~new
n = .note~new
ts = .span~fromSeconds(600)
.environment~log~empty
a = .ticker~new(ts, n)
say 'span:' .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
c = .ticker~new(600, n, .tracked~new)
say 'seconds:' .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
signal on syntax name bad
d = .ticker~new(.span~fromSeconds(-600), n)
bad: say 'negative:' condition('o')~code,
  .environment~log~makeArray~makeString('l', ' ')~subword(1, 3)
.environment~log~empty
signal on syntax name bad2
e = .ticker~new(ts, .object~new)
bad2: say 'refused:' condition('o')~code .environment~log~items
a~cancel
c~cancel
::class note inherit AlarmNotification
::method triggered
  nop
::method isA
  .environment~log~append('isA' arg(1)~id)
  forward class (super)
::class span subclass timespan
::method totalMicroseconds
  .environment~log~append('totalMicroseconds')
  forward class (super)
::method sign
  .environment~log~append('sign')
  forward class (super)
::method days
  .environment~log~append('days')
  forward class (super)
::class tracked
::method string
  .environment~log~append('string')
  return 'tracked'
```

```output
span: isA AlarmNotification sign days totalMicroseconds
seconds: isA AlarmNotification
negative: 93.951 isA AlarmNotification sign
refused: 88.914 0
```

## What the ticker answers

r[class.ticker.interval]
`interval` answers a `TimeSpan`, the length of the interval that was set, and
it takes no argument (`SYNTAX 93.902`). For an interval that was a `TimeSpan`
it is a span equal to it ([the compare rule](r:class.timespan.compare)); for a
string it is that number of seconds, cut off to the microsecond. It does not
change when the ticker is cancelled, and it sends no message.

```rexx
n = .note~new
say .ticker~new(1.5e2, n)~~cancel~interval
say .ticker~new('86400', n)~~cancel~interval
say .ticker~new('100000.5', n)~~cancel~interval
say .ticker~new('1.9999999', n)~~cancel~interval
say .ticker~new('60.0000009', n)~~cancel~interval
ts = .timespan~fromSeconds(7200)
t = .ticker~new(ts, n)
say t~interval~class~id (t~interval == ts) t~interval
t~cancel
say t~interval
signal on syntax name bad
say t~interval(1)
bad: say condition('o')~code
::class note inherit AlarmNotification
::method triggered
  nop
```

```output
00:02:30.000000
1.00:00:00.000000
1.03:46:40.500000
00:00:01.999999
00:01:00.000000
TimeSpan 1 02:00:00.000000
02:00:00.000000
93.902
```

r[class.ticker.attachment]
`attachment` answers the object that `new` was given as the third argument,
the very object, and `.nil` when there was none. It takes no argument
(`SYNTAX 93.902`) and sends no message. It answers the same after the ticker
has been cancelled.

```rexx
n = .note~new
o = .object~new
a = .ticker~new(60, n, o)
b = .ticker~new(60, n)
c = .ticker~new(60, n, 'text')
say (a~attachment == o) (b~attachment == .nil) c~attachment
a~cancel
say (a~attachment == o)
signal on syntax name bad
say a~attachment(1)
bad: say condition('o')~code
b~cancel
c~cancel
::class note inherit AlarmNotification
::method triggered
  nop
```

```output
1 1 text
1
93.902
```

r[class.ticker.triggered]
The ticker sends its target `triggered` with itself as the argument at the end
of each interval, until it is cancelled; the target's method is the one of
[the notification rule](r:class.alarmnotification.triggered). How many times it
is sent before a program cancels the ticker is not something the program can
count on, as it depends on the times that the environment chapter specifies, so
the example cancels from the method itself after the third, and shows that the
ticker stops: the target is sent `cancel` once and nothing after it, even though
the program waits for several more intervals. Unlike an alarm, a ticker has no
method `triggered` of its own.

```rexx
n = .note~new(3)
t = .ticker~new(0.02, n, 'tick')
n~awaitCancel
call syssleep 0.2
say n~log~strip
say t~canceled t~hasMethod('triggered')
::class note inherit AlarmNotification
::method init
  expose log count cdone limit
  use arg limit
  log = ''
  count = 0
  cdone = .false
::method triggered unguarded
  expose log count limit
  use arg src
  count = count + 1
  log = log 'triggered' src~attachment
  if count = limit then src~cancel
::method cancel unguarded
  expose log cdone
  use arg src
  log = log 'cancel' src~attachment
  cdone = .true
::method awaitCancel unguarded
  expose cdone
  guard on when cdone
::method log unguarded
  expose log
  return log
```

```output
triggered tick triggered tick triggered tick cancel tick
1 0
```

## Cancelling

r[class.ticker.cancel]
`cancel` takes no argument (`SYNTAX 93.902`) and answers nothing, so using its
answer as a value raises `SYNTAX 91.999`. Sent to a ticker that has not been
cancelled it stops the ticker: no `triggered` is sent from then on, `canceled`
answers 1, and the target is sent `cancel` with the ticker as its argument, as
[the notification rule](r:class.alarmnotification.cancel) says. The flag is
already 1 when the target receives `cancel`. A second `cancel` of a ticker
sends the target nothing. A ticker that is cancelled from the `triggered` of
its own target stops in the same way.

```rexx
n = .note~new
a = .ticker~new(60, n, 'waiting')
a~cancel
n~awaitCancel
say n~log~strip
say a~canceled a~cancelled
n2 = .quiet~new
z = .ticker~new(60, n2)
y = .ticker~new(60, n2)
signal on syntax name e1
x = z~cancel
e1: say condition('o')~code
signal on syntax name e2
y~cancel(1)
e2: say condition('o')~code
y~cancel
a~cancel
call syssleep 0.2
say n~log~words
::class quiet inherit AlarmNotification
::method triggered
  nop
::class note inherit AlarmNotification
::method init
  expose log cdone
  log = ''
  cdone = .false
::method triggered unguarded
  nop
::method cancel unguarded
  expose log cdone
  use arg src
  log = log 'cancel' src~attachment src~canceled
  cdone = .true
::method awaitCancel unguarded
  expose cdone
  guard on when cdone
::method log unguarded
  expose log
  return log
```

```output
cancel waiting 1
1 1
91.999
93.902
3
```

r[class.ticker.canceled]
`canceled` and `cancelled` answer 1 when the ticker has been cancelled and 0
otherwise. The two names have one meaning and are two methods: neither sends
the other, and a subclass that overrides one leaves the other as it was. Each
takes no argument (`SYNTAX 93.902`).

```rexx
n = .note~new
a = .ticker~new(60, n)
b = .ticker~new(60, n)
say a~canceled a~cancelled
a~cancel
say a~canceled a~cancelled b~canceled
s = .sub~new(60, n)
say s~cancelled s~canceled
s~cancel
say s~cancelled s~canceled
b~cancel
signal on syntax name bad
say a~canceled(1)
bad: say condition('o')~code
::class note inherit AlarmNotification
::method triggered
  nop
::class sub subclass ticker
::method canceled
  return 'override'
```

```output
0 0
1 1 0
0 override
1 override
93.902
```

r[class.ticker.cancel.sends]
`cancel` sends the target `cancel` with the ticker as the argument, once and
only when the ticker was not cancelled before. It sends no message to the
ticker itself and none to the attachment, so overrides of `canceled`,
`cancelled`, `attachment` and `interval` do not run when the ticker is
cancelled. The reading methods `attachment`, `canceled`, `cancelled` and
`interval` send no message at all. The example overrides each of them in a
subclass, cancels the ticker, waits until the target has been sent `cancel`,
calls the readers and shows that the log holds the `cancel` of the target and
each reader once, from the call of the program.

```rexx
.environment~log = .list~new
n = .note~new
a = .sub~new(60, n, 'att')
a~cancel
n~awaitCancel
say 'cancelled'
x = a~attachment
x = a~canceled
x = a~cancelled
x = a~interval
say .environment~log~makeArray~makeString('l', ' ')
::class note inherit AlarmNotification
::method init
  expose cdone
  cdone = .false
::method triggered
  nop
::method cancel unguarded
  expose cdone
  .environment~log~append('target-cancel')
  cdone = .true
::method awaitCancel unguarded
  expose cdone
  guard on when cdone
::class sub subclass ticker
::method attachment
  .environment~log~append('attachment')
  forward class (super)
::method canceled
  .environment~log~append('canceled')
  forward class (super)
::method cancelled
  .environment~log~append('cancelled')
  forward class (super)
::method interval
  .environment~log~append('interval')
  forward class (super)
```

```output
cancelled
target-cancel attachment canceled cancelled interval
```

## Deviations

r[class.ticker.deviation.109]
A string interval that is an instance of a subclass of `String` is read by its
own characters, and `request`, `makeString` and `string` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an instance
`request` with `STRING`, so an override can change what is read. In the
example the overrides would make the interval `-1`; they count their sends,
and the string is read as 60 seconds without one.

```rexx
.environment~sent = 0
n = .note~new
t = .ticker~new(.digits~new('60'), n)
say t~interval 'sends' .environment~sent
t~cancel
::class note inherit AlarmNotification
::method triggered
  nop
::class digits subclass string
::method makestring
  .environment~sent = .environment~sent + 1
  return '-1'
::method request
  .environment~sent = .environment~sent + 1
  return '-1'
```

```output defect=#140
00:01:00.000000 sends 0
```
