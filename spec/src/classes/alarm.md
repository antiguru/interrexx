---
element: "Alarm"
kind: class
deviations: [109, 119]
rexxref: "Builtin Classes > Utility Classes > Alarm Class; Builtin Classes > Utility Classes > Alarm Class > attachment; Builtin Classes > Utility Classes > Alarm Class > cancel; Builtin Classes > Utility Classes > Alarm Class > canceled/cancelled; Builtin Classes > Utility Classes > Alarm Class > init; Builtin Classes > Utility Classes > Alarm Class > scheduledTime; Builtin Classes > Utility Classes > Alarm Class > triggered"
---

# Alarm

An `Alarm` is a one-shot timer. It is made for a moment, a target that wants
to hear of it and, if the program likes, an object to attach; when the moment
comes it sends its target `triggered`, and when it is cancelled before that it
sends the target `cancel`. The target has to be an implementation of
[`AlarmNotification`](alarmnotification.md), which states what the target is
sent and when; this page states how an alarm is made, what its time may be and
what its own methods answer. How the library runs an alarm beside the program,
and so at which time the target is reached and by which activity, is specified
in the environment chapter. The examples do not depend on it: they set an
alarm for a time that the program does not wait for and cancel it, or they
wait on a guard that the target opens, and they print nothing that varies with
the clock.

## The class

r[class.alarm.creation]
`Alarm` has the instance methods `attachment`, `cancel`, `canceled`,
`cancelled`, `init`, `scheduledTime` and `triggered`, besides the `new` that
every class has. It may have private methods besides these; they are not part
of this specification. Its only superclass is `Object`, it inherits no mixin
class and is not itself a mixin class ([the hierarchy
rule](r:class.index.hierarchy)). In particular an `Alarm` is not an
implementation of `AlarmNotification`: it cannot be the target of another
alarm. `new` is the one [that `Class` supplies](r:class.index.new): it sends
`init` the arguments of `new`, which [the next rule](r:class.alarm.init)
states, and answers an instance of the class that received it, so a subclass
of `Alarm` makes instances of itself. `cancel` and the readers `canceled`,
`cancelled` and `triggered` are stated in [the rules on
cancelling](r:class.alarm.cancel).

```rexx
say .alarm~id .alarm~superclasses~items .alarm~superclass~id,
  .alarm~queryMixinClass
have = .set~new
s = .alarm~methods(.nil)
do while s~available
  have~put(s~index)
  s~next
end
names = 'attachment cancel canceled cancelled init'
names = (names 'scheduledTime triggered')~makeArray(' ')
count = 0
do name over names
  if have~hasItem(name~upper) then count = count + 1
end
say count 'of 7'
n = .note~new
a = .alarm~new(60, n)
b = .sub~new(60, n)
say a~class~id b~class~id a~isInstanceOf(.alarmnotification)
signal on syntax name bad
c = .alarm~new(60, a)
bad: say condition('o')~code
a~cancel
b~cancel
::class note inherit AlarmNotification
::method triggered
  nop
::class sub subclass alarm
```

```output
Alarm 1 Object 0
7 of 7
Alarm SUB 0
88.914
```

## Making an alarm

r[class.alarm.init]
`new` sends `init(atime, target, attachment)`, and `init` sets the alarm up. It
needs the first two arguments, so `new` with none or only the first raises
`SYNTAX 93.901` and an omitted first or second argument raises `SYNTAX 93.903`;
a fourth argument raises `SYNTAX 93.902`. The checks of the arguments that are
present are made in this order: first the target, which must be an
implementation of [`AlarmNotification`](r:class.alarmnotification.creation) or
`SYNTAX 88.914` is raised, and then the time, which raises `SYNTAX 93.951` for
every value that [the rules below](r:class.alarm.init.seconds) do not accept,
except the very large ones for which they name other errors. So a target that is
no implementation is reported even when the time is wrong too, and a refused
alarm is not set. `attachment` can be any object, and is `.nil` when it is
omitted. The alarm is set when `init` ends normally.

```rexx
n = .note~new
call try ".alarm~new"
call try ".alarm~new(60)"
call try ".alarm~new(, n)"
call try ".alarm~new(60, , 'x')"
call try ".alarm~new(60, n, 'x', 'y')"
call try ".alarm~new('abc', .nil)"
call try ".alarm~new('abc', n)"
call try ".alarm~new(60, n)"
call try ".alarm~new(60, n, .nil)"
call try ".alarm~new(60, n, .array~new)"
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
.alarm~new => 93.901
.alarm~new(60) => 93.901
.alarm~new(, n) => 93.903
.alarm~new(60, , 'x') => 93.903
.alarm~new(60, n, 'x', 'y') => 93.902
.alarm~new('abc', .nil) => 88.914
.alarm~new('abc', n) => 93.951
.alarm~new(60, n) => ok
.alarm~new(60, n, .nil) => ok
.alarm~new(60, n, .array~new) => ok
```

r[class.alarm.init.seconds]
A time that is a string without a colon is a number of seconds from now. It is
read as a number as [the DATATYPE `N` rule](r:bif.datatype.number) reads it, so
blanks at either end are allowed and a decimal point or an exponent is too, and
it has to be zero or more: a string that is no number, the empty string and a
negative number raise `SYNTAX 93.951`. The number is cut to the microsecond
before its sign is checked, so `-0.0000001` is a time of zero. A number with
more than 18 significant digits is first rounded to 18, and the number is read
in full whatever the NUMERIC DIGITS setting. A number of `10**12` or more, or of
`-10**12` or less, once it is rounded as said above, raises `SYNTAX 88.903`
instead, and one for which the date and time of the moment in the host's
offset, as `scheduledTime` would answer it, would fall after 31 December 9999 at
23:59:59.999999 or before 1 January 0001 at 00:00:00.000000
([the limits of `DateTime`](r:class.datetime.limits)) raises `SYNTAX 40.19`,
which is checked before the sign. The largest and smallest times accepted so
depend on the host's offset, as they do for [`addSeconds`](r:class.datetime.add)
of a `DateTime` of now. A `TimeSpan` that does so raises `SYNTAX 40.19` too.
The alarm is set for the moment that many seconds after `init` read the clock,
to the microsecond. The example reads the scheduled time back, with the bounds a
clock cannot break, and prints only whether they hold; under NUMERIC DIGITS 3 it
shows the alarm of a number of five digits set in full.

```rexx
n = .note~new
call try ' 90 '
call try '1.5e2'
call try '+.5e3'
call try ''
call try '1x'
call try '-1'
call try '--1'
call try '2.6e11'
call try '-2e11'
call try '1e12'
call try '-1e12'
numeric digits 3
before = .datetime~new
a = .alarm~new('12345', n)
span = a~scheduledTime - before
say 'digits 3:' span~hours span~minutes span~seconds
a~cancel
exit
try:
  signal on syntax name bad
  before = .datetime~new
  a = .alarm~new(arg(1), n)
  span = a~scheduledTime - before
  secs = span~totalSeconds
  say '"'arg(1)'" => seconds' (secs > 0) (secs < 601)
  a~cancel
  return
bad: say '"'arg(1)'" =>' condition('o')~code
  return
::class note inherit AlarmNotification
::method triggered
  nop
```

```output
" 90 " => seconds 1 1
"1.5e2" => seconds 1 1
"+.5e3" => seconds 1 1
"" => 93.951
"1x" => 93.951
"-1" => 93.951
"--1" => 93.951
"2.6e11" => 40.19
"-2e11" => 40.19
"1e12" => 88.903
"-1e12" => 88.903
digits 3: 3 25 45
```

r[class.alarm.init.clock]
A time that is a string with a colon names a time of day, and may name a date.
The forms are `hh:mm:ss` and `hh:mm:ss d Mmm yyyy`, with exactly one blank
before the date, and nothing else is accepted: not a blank at either end, not
a time with fewer than the two digits of each part, not hours of 24 or more or
minutes or seconds of 60 or more, not a fraction of a second, not a date
alone, not an ISO date and not a year with fewer than four digits. `hh`, `mm`
and `ss` are two digits each; `d` is one or two digits, the day of the month,
which must exist in that month and year, and `Mmm` is the English name of the
month cut to three letters, written with a capital first and lower case after
it (`Jan`, `Feb`, `Mar`, `Apr`, `May`, `Jun`, `Jul`, `Aug`, `Sep`, `Oct`,
`Nov`, `Dec`): `jan`, `JAN` and `Sept` raise `SYNTAX 93.951`. Without a date
the alarm is set for the first moment after now at which the clock of the
local time zone shows that time, which is today or tomorrow. With a date it is
set for that moment, and a moment that is not after now raises
`SYNTAX 93.951`. The example prints the time of day of the scheduled time, the
whole moment of those with a date and, for the others, whether the scheduled
time is after now and within a day of it.

```rexx
n = .note~new
call try '12:00:00'
call try '00:00:00'
call try '23:59:59'
call try '12:00:00 1 Jan 2999'
call try '03:04:05 09 Feb 2996'
call try '12:00:00 29 Feb 2996'
call try '12:00:00 1 Jan 2000'
call try '12:00:00 31 Feb 2999'
call try '12:00:00 1 jan 2999'
call try '12:00:00 1 JAN 2999'
call try '12:00:00 1 Sept 2999'
call try '12:00:00  1 Jan 2999'
call try '12:00:00 1 Jan 2999 '
call try '12:00:00 1 Jan 99'
call try '12:00:00 2999-01-01'
call try '1 Jan 2999'
call try '12:00:00 '
call try ' 12:00:00'
call try '1:02:03'
call try '12:00'
call try '24:00:00'
call try '12:60:00'
call try '12:00:60'
call try '12:00:00.5'
line = ''
do m over 'Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec'~makeArray(' ')
  a = .alarm~new('12:00:00 5' m '2999', n)
  line = line a~scheduledTime~isoDate~substr(6, 2)
  a~cancel
end
say line~strip
exit
try:
  signal on syntax name bad
  before = .datetime~new
  a = .alarm~new(arg(1), n)
  t = a~scheduledTime
  if arg(1)~words > 1 then say '"'arg(1)'" =>' t~isoDate
  else say '"'arg(1)'" =>' t~isoDate~right(15) (t > before),
    ((t - before)~totalDays <= 1)
  a~cancel
  return
bad: say '"'arg(1)'" =>' condition('o')~code
  return
::class note inherit AlarmNotification
::method triggered
  nop
```

```output
"12:00:00" => 12:00:00.000000 1 1
"00:00:00" => 00:00:00.000000 1 1
"23:59:59" => 23:59:59.000000 1 1
"12:00:00 1 Jan 2999" => 2999-01-01T12:00:00.000000
"03:04:05 09 Feb 2996" => 2996-02-09T03:04:05.000000
"12:00:00 29 Feb 2996" => 2996-02-29T12:00:00.000000
"12:00:00 1 Jan 2000" => 93.951
"12:00:00 31 Feb 2999" => 93.951
"12:00:00 1 jan 2999" => 93.951
"12:00:00 1 JAN 2999" => 93.951
"12:00:00 1 Sept 2999" => 93.951
"12:00:00  1 Jan 2999" => 93.951
"12:00:00 1 Jan 2999 " => 93.951
"12:00:00 1 Jan 99" => 93.951
"12:00:00 2999-01-01" => 93.951
"1 Jan 2999" => 93.951
"12:00:00 " => 93.951
" 12:00:00" => 93.951
"1:02:03" => 93.951
"12:00" => 93.951
"24:00:00" => 93.951
"12:60:00" => 93.951
"12:00:60" => 93.951
"12:00:00.5" => 93.951
01 02 03 04 05 06 07 08 09 10 11 12
```

r[class.alarm.init.moment]
A time that is a `DateTime` sets the alarm for that moment, and it has to be
after the moment `init` reads the clock: a `DateTime` that is not after now
raises `SYNTAX 93.951`. A time that is a `TimeSpan` sets the alarm for that long
after `init` read the clock; it must not be negative, and a span of zero is
allowed. `.nil`, an array and an instance of `Object` raise `SYNTAX 93.951`.
`scheduledTime` answers the moment that was set, as
[its rule](r:class.alarm.scheduledtime) says. Only a string, a `DateTime` and a
`TimeSpan` are read as a time. A time of zero, whether it is the number 0 or a
span of zero, sets an alarm that goes off at once; the second example makes one
of each and waits until its target has been sent `triggered` twice.

```rexx
n = .note~new
now = .datetime~new
call try '.datetime~new~addSeconds(600)'
call try '.datetime~new~addSeconds(-600)'
call try '.datetime~new'
call try '.timespan~fromSeconds(600)'
call try '.timespan~fromSeconds(-600)'
call try '.nil'
call try '.array~new'
call try '.object~new'
exit
try:
  signal on syntax name bad
  interpret 'when =' arg(1)
  a = .alarm~new(when, n)
  say arg(1) '=> ok' (a~scheduledTime > now)
  a~cancel
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class note inherit AlarmNotification
::method triggered
  nop
```

```output
.datetime~new~addSeconds(600) => ok 1
.datetime~new~addSeconds(-600) => 93.951
.datetime~new => 93.951
.timespan~fromSeconds(600) => ok 1
.timespan~fromSeconds(-600) => 93.951
.nil => 93.951
.array~new => 93.951
.object~new => 93.951
```

```rexx
n = .note~new
a = .alarm~new(0, n, 'number')
b = .alarm~new(.timespan~fromSeconds(0), n, 'span')
n~awaitCount(2)
say a~triggered b~triggered n~log~words
::class note inherit AlarmNotification
::method init
  expose log count
  log = ''
  count = 0
::method triggered unguarded
  expose log count
  use arg src
  log = log src~attachment
  count = count + 1
::method awaitCount unguarded
  expose count
  use arg k
  guard on when count >= k
::method log unguarded
  expose log
  return log
```

```output
1 1 2
```

r[class.alarm.init.sends]
`init` sends messages that a program can override, in this order: first `isA`
to the target, with `AlarmNotification` as its one argument, once; the answer
decides whether the target is an implementation. Then, for a time that is a
`TimeSpan`, the span is sent `totalMicroseconds`, `sign`, `days` and
`totalMicroseconds` again, once each and in that order, and for a time that is
a `DateTime`, it is sent `-` once, with a `DateTime` of the moment of the call
as the argument. A time that is a string is sent nothing ([Deviation
109](r:class.alarm.deviation.109) covers a subclass of `String`). When the
span is negative `init` stops after `sign` and when the `DateTime` is not
after now it stops after `-`; the further messages that build the text of the
error are not specified. An alarm that is refused for its target sends the
time nothing. `init` sends no message to the attachment, and none to the alarm
itself, so an override of `attachment` or `scheduledTime` does not run. The
example logs the sends of an override of each message in a subclass of
`TimeSpan` and `DateTime`.

```rexx
.environment~log = .list~new
n = .note~new
ts = .span~fromSeconds(600)
dt = .moment~new~addSeconds(600)
.environment~log~empty
a = .alarm~new(ts, n)
say 'span:' .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
b = .alarm~new(dt, n)
say 'moment:' .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
c = .alarm~new(600, n, .tracked~new)
say 'seconds:' .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
signal on syntax name bad
d = .alarm~new(.span~fromSeconds(-600), n)
bad: say 'negative:' condition('o')~code,
  .environment~log~makeArray~makeString('l', ' ')~subword(1, 4)
.environment~log~empty
signal on syntax name bad2
e = .alarm~new(ts, .object~new)
bad2: say 'refused:' condition('o')~code .environment~log~items
a~cancel
b~cancel
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
::class moment subclass datetime
::method '-'
  .environment~log~append('-' arg(1)~class~id)
  forward class (super)
::class tracked
::method string
  .environment~log~append('string')
  return 'tracked'
```

```output
span: isA AlarmNotification totalMicroseconds sign days totalMicroseconds
moment: isA AlarmNotification - DateTime
seconds: isA AlarmNotification
negative: 93.951 isA AlarmNotification totalMicroseconds sign
refused: 88.914 0
```

## What the alarm answers

r[class.alarm.attachment]
`attachment` answers the object that `new` was given as the third argument,
the very object, and `.nil` when there was none. It takes no argument
(`SYNTAX 93.902`) and sends no message. It answers the same after the alarm
has gone off and after it has been cancelled.

```rexx
n = .note~new
o = .object~new
a = .alarm~new(60, n, o)
b = .alarm~new(60, n)
c = .alarm~new(60, n, 'text')
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

r[class.alarm.scheduledtime]
`scheduledTime` answers a `DateTime`, the moment for which the alarm was set,
and it takes no argument (`SYNTAX 93.902`). For a time that was a `DateTime`
it is that moment, equal to the object that was given. For a time that was a
number of seconds or a `TimeSpan`, it is the moment `init` read the clock plus
the span, to the microsecond; for a time of day, the moment that [the clock
rule](r:class.alarm.init.clock) finds. It does not change when the alarm goes
off or is cancelled, and it sends no message. The first example shows that a
`DateTime` is answered as it was given, and its `==` is true ([the compare
rule](r:class.datetime.compare)); the second shows the sum.

```rexx
n = .note~new
dt = .datetime~new~addSeconds(600)
a = .alarm~new(dt, n)
say (a~scheduledTime == dt) a~scheduledTime~class~id
a~cancel
say (a~scheduledTime == dt)
signal on syntax name bad
say a~scheduledTime(1)
bad: say condition('o')~code
::class note inherit AlarmNotification
::method triggered
  nop
```

```output
1 DateTime
1
93.902
```

```rexx
n = .note~new
before = .datetime~new
a = .alarm~new(.timespan~fromSeconds(500), n)
after = .datetime~new
t = a~scheduledTime
say (t >= before + .timespan~fromSeconds(500)),
  (t <= after + .timespan~fromSeconds(500))
b = .alarm~new('500', n)
say ((b~scheduledTime - a~scheduledTime)~totalSeconds < 1)
a~cancel
b~cancel
::class note inherit AlarmNotification
::method triggered
  nop
```

```output
1 1
1
```

r[class.alarm.triggered]
`triggered` answers 1 when the alarm has gone off and 0 otherwise, so 0 for an
alarm that is still waiting and for one that was cancelled before it went off.
The flag is already 1 when the target is sent `triggered`, so the target's
method sees it. It takes no argument (`SYNTAX 93.902`) and sends no message. The
method is a reader of that flag; `AlarmNotification`'s `triggered` is another
method, which the target has
([the notification rule](r:class.alarmnotification.triggered)). The library
sends `triggered` to the target only, never to the alarm, so an override of
`triggered` in a subclass of `Alarm` does not run when it goes off.

```rexx
n = .note~new
a = .alarm~new(0.05, n, 'A')
say 'made:' a~triggered
n~awaitCount(1)
say 'target saw:' n~log~strip
say 'after:' a~triggered a~canceled
c = .alarm~new(60, n, 'C')
c~cancel
say 'cancelled:' c~triggered c~canceled
signal on syntax name bad
say a~triggered(1)
bad: say condition('o')~code
::class note inherit AlarmNotification
::method init
  expose log count
  log = ''
  count = 0
::method triggered unguarded
  expose log count
  use arg src
  log = log src~attachment src~triggered src~canceled
  count = count + 1
::method cancel unguarded
  nop
::method awaitCount unguarded
  expose count
  use arg k
  guard on when count >= k
::method log unguarded
  expose log
  return log
```

```output
made: 0
target saw: A 1 0
after: 1 0
cancelled: 0 1
93.902
```

## Cancelling

r[class.alarm.cancel]
`cancel` takes no argument (`SYNTAX 93.902`) and answers nothing, so using its
answer as a value raises `SYNTAX 91.999`. Sent to an alarm that is still waiting
it ends the wait: the alarm will not go off, `canceled` answers 1 from then on
and `triggered` stays 0, and the target is sent `cancel` with the alarm as its
argument, as [the notification rule](r:class.alarmnotification.cancel) says. The
flag is already 1 when the target receives `cancel`. A second `cancel` of an
alarm that was cancelled sends the target nothing
([Deviation 119](r:class.alarmnotification.deviation.119)).

```rexx
n = .note~new
a = .alarm~new(60, n, 'waiting')
a~cancel
n~awaitCancel
say n~log~strip
say a~canceled a~cancelled a~triggered
z = .alarm~new(60, n)
y = .alarm~new(60, n)
signal on syntax name e1
x = z~cancel
e1: say condition('o')~code
signal on syntax name e2
y~cancel(1)
e2: say condition('o')~code
y~cancel
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
1 1 0
91.999
93.902
```

r[class.alarm.canceled]
`canceled` and `cancelled` answer 1 when the alarm was cancelled while it was
still waiting and 0 otherwise, so 0 for an alarm that has gone off. The two
names have one meaning and are two methods: neither sends the other, and a
subclass that overrides one leaves the other as it was. Each takes no argument
(`SYNTAX 93.902`).

```rexx
n = .note~new
a = .alarm~new(60, n)
b = .alarm~new(60, n)
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
::class sub subclass alarm
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

r[class.alarm.cancel.sends]
`cancel` sends the target `cancel` with the alarm as the argument, once and
only when the alarm was still waiting. It sends no message to the alarm itself
and none to the attachment, so overrides of `canceled`, `cancelled`,
`attachment` and `triggered` do not run when the alarm is cancelled. The
reading methods `attachment`, `canceled`, `cancelled`, `scheduledTime` and
`triggered` send no message at all. The example overrides each of them in a
subclass, cancels the alarm, waits until the target has been sent `cancel`,
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
x = a~scheduledTime
x = a~triggered
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
::class sub subclass alarm
::method attachment
  .environment~log~append('attachment')
  forward class (super)
::method canceled
  .environment~log~append('canceled')
  forward class (super)
::method cancelled
  .environment~log~append('cancelled')
  forward class (super)
::method scheduledTime
  .environment~log~append('scheduledTime')
  forward class (super)
::method triggered
  .environment~log~append('triggered')
  forward class (super)
```

```output
cancelled
target-cancel attachment canceled cancelled scheduledTime triggered
```

## Deviations

r[class.alarm.deviation.109]
A time that is an instance of a subclass of `String` is read by its own
characters, and `request`, `makeString` and `string` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an instance
`request` with `STRING`, so an override can change what is read. In the
example the overrides would make the time `-1`; they count their sends, and
the string is read as 60 seconds without one.

```rexx
.environment~sent = 0
n = .note~new
a = .alarm~new(.digits~new('60'), n)
say (a~scheduledTime > .datetime~new) 'sends' .environment~sent
a~cancel
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
1 sends 0
```

r[class.alarm.deviation.119]
A second `cancel` of an alarm that was cancelled sends its target nothing, as
the notification page states ([Deviation
119](r:class.alarmnotification.deviation.119)). ooRexx sends the target
`cancel` once or twice for it, depending on timing, so the example is not run
against it; the target prints a line for each `cancel` it is sent.

```rexx
n = .n~new
a = .alarm~new(60, n)
a~cancel
a~cancel
call syssleep 0.3
say 'done'
exit
::class n inherit alarmnotification
::method cancel unguarded
  say 'cancel sent'
```

```output defect=#150
cancel sent
done
```
