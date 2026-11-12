---
element: "TimeSpan"
kind: class
rexxref: "Builtin Classes > Utility Classes > TimeSpan Class; Builtin Classes > Utility Classes > TimeSpan Class > init; Builtin Classes > Utility Classes > TimeSpan Class > fromCivilTime (Class Method); Builtin Classes > Utility Classes > TimeSpan Class > fromDays (Class Method); Builtin Classes > Utility Classes > TimeSpan Class > fromHours (Class Method); Builtin Classes > Utility Classes > TimeSpan Class > fromLongTime (Class Method); Builtin Classes > Utility Classes > TimeSpan Class > fromMicroseconds (Class Method); Builtin Classes > Utility Classes > TimeSpan Class > fromMinutes (Class Method); Builtin Classes > Utility Classes > TimeSpan Class > fromNormalTime (Class Method); Builtin Classes > Utility Classes > TimeSpan Class > fromSeconds (Class Method); Builtin Classes > Utility Classes > TimeSpan Class > fromStringFormat (Class Method); Builtin Classes > Utility Classes > TimeSpan Class > Arithmetic Methods; Builtin Classes > Utility Classes > TimeSpan Class > addDays; Builtin Classes > Utility Classes > TimeSpan Class > addHours; Builtin Classes > Utility Classes > TimeSpan Class > addMicroseconds; Builtin Classes > Utility Classes > TimeSpan Class > addMinutes; Builtin Classes > Utility Classes > TimeSpan Class > addSeconds; Builtin Classes > Utility Classes > TimeSpan Class > addWeeks; Builtin Classes > Utility Classes > TimeSpan Class > compareTo; Builtin Classes > Utility Classes > TimeSpan Class > days; Builtin Classes > Utility Classes > TimeSpan Class > duration; Builtin Classes > Utility Classes > TimeSpan Class > hashCode; Builtin Classes > Utility Classes > TimeSpan Class > hours; Builtin Classes > Utility Classes > TimeSpan Class > makeString; Builtin Classes > Utility Classes > TimeSpan Class > microseconds; Builtin Classes > Utility Classes > TimeSpan Class > minutes; Builtin Classes > Utility Classes > TimeSpan Class > seconds; Builtin Classes > Utility Classes > TimeSpan Class > sign; Builtin Classes > Utility Classes > TimeSpan Class > string; Builtin Classes > Utility Classes > TimeSpan Class > totalDays; Builtin Classes > Utility Classes > TimeSpan Class > totalHours; Builtin Classes > Utility Classes > TimeSpan Class > totalMicroseconds; Builtin Classes > Utility Classes > TimeSpan Class > totalMinutes; Builtin Classes > Utility Classes > TimeSpan Class > totalSeconds"
deviations: [109, 126, 127, 130, 131, 132, 133, 163]
---

# TimeSpan

A `TimeSpan` is a length of time: a signed count of microseconds, at most
3652058 days and 23:59:59.999999 either way, read as days, hours, minutes,
seconds and microseconds. It is the difference of two [`DateTime`
moments](r:class.datetime.minus) and what is added to one ([the `+`
rule](r:class.datetime.plus)), and it can stand for the offset of a time zone
([the offset rule](r:class.datetime.offset)). Its methods answer the parts and
the totals of the span, write it as text, make another span from it by
addition, multiplication and division, and compare it with another. The class
inherits `Comparable` and `Orderable`, which give it the comparison operators
on top of its own `compareTo`. Every method that
answers another span answers a new instance; only `init`, sent again, changes
one ([the `new` rule](r:class.timespan.new)). The examples build their spans
from fixed values.

## The class

r[class.timespan.creation]
`TimeSpan` has the instance methods `+`, `-`, `*`, `/`, `//`, `%`, `addDays`,
`addHours`, `addMicroseconds`, `addMinutes`, `addSeconds`, `addWeeks`,
`compareTo`, `days`, `duration`, `hashCode`, `hours`, `init`, `makeString`,
`microseconds`, `minutes`, `seconds`, `sign`, `string`, `totalDays`,
`totalHours`, `totalMicroseconds`, `totalMinutes` and `totalSeconds`, and the
class methods `fromCivilTime`, `fromDays`, `fromHours`, `fromLongTime`,
`fromMicroseconds`, `fromMinutes`, `fromNormalTime`, `fromSeconds` and
`fromStringFormat`, besides the `new` that every class has. It may have
private methods besides these; they are not part of this specification. Its
superclasses are `Object`, `Comparable` and `Orderable`, in that order. The
comparison operators of an instance are those of `Orderable`, which send
`compareTo` to the receiver ([the comparison rule](r:class.timespan.compare)).
An instance has no `unknown` method, so a message that [the method
search](r:class.methods.unknown) does not resolve raises
[NOMETHOD](r:cond.nomethod.raise), as the error rule of [the
chapter](r:class.index.errors) says; it is not a string either, so a string
method such as `length` raises `SYNTAX 97.1`.

```rexx
do c over .timespan~superclasses
  say c~id
end
names = '+ - * / // % addDays addHours addMicroseconds addMinutes addSeconds',
  'addWeeks compareTo days duration hashCode hours init makeString',
  'microseconds minutes seconds sign string totalDays totalHours',
  'totalMicroseconds totalMinutes totalSeconds'
found = 0
do n over names~makearray(' ')
  found += (.timespan~method(n)~scope == .timespan)
end
say found 'of' names~words
cnames = 'fromCivilTime fromDays fromHours fromLongTime fromMicroseconds',
  'fromMinutes fromNormalTime fromSeconds fromStringFormat new'
found = 0
do n over cnames~makearray(' ')
  found += .timespan~hasMethod(n)
end
say found 'of' cnames~words
t = .timespan~new(1)
say t~hasMethod('FROMDAYS') t~hasMethod('UNKNOWN')
signal on syntax name bad
say .timespan~new(1)~length
bad: say condition('o')~code
```

```output
Object
Comparable
Orderable
29 of 29
10 of 10
0 0
97.1
```

r[class.timespan.value]
An instance holds a count of microseconds, a whole number from
-315537897599999999 to 315537897599999999, which is the span from the first to
the last microsecond of the years that `DateTime` covers; no span is outside
this range, and a method that would make one raises `SYNTAX 40.19` unless its
rule gives another error. The count is read as days, hours, minutes, seconds and
microseconds by [the parts rule](r:class.timespan.parts), and a negative count
is a negative span in all of them. The arithmetic of the class is done in 18
significant digits, whatever `NUMERIC DIGITS` the program that sends the message
has set ([the total rule](r:class.timespan.total) and the rules of the
arithmetic methods say where it shows). The string value of an instance is [its
`string` text](r:class.timespan.string): an instance written by `say`, or joined
to a string by `||`, is that text, as [the string-value
rule](r:class.index.string) says. An instance on which the `init` of `TimeSpan`
never ran, because the `init` of a subclass does not forward to it, is the span
of 0 ([Deviation 131](r:class.timespan.deviation.131)).

```rexx
say .timespan~new(0)
say .timespan~new(315537897599999999) .timespan~new('-315537897599999999')
t = .timespan~new(1, 2, 3, 4, 5)
say t
say 'at' t || '!'
signal on syntax name bad
say .timespan~new(315537897600000000)
bad: say condition('o')~code
```

```output
00:00:00.000000
3652058.23:59:59.999999 -3652058.23:59:59.999999
1.02:03:04.000005
at 1.02:03:04.000005!
40.19
```

## Making an instance

r[class.timespan.new]
`new` makes an instance and sends `init` to it with the arguments it received
([the new rule](r:class.index.new)); the `init` of `TimeSpan` takes from one
argument to five, and the number of them that are given, counted as [the count
rule](r:class.index.args.count) says, picks the form. With one argument it is
the number of microseconds of the span, a whole number as [the form
rule](r:conv.whole.form) and [the digits rule](r:conv.whole.digits) read it
([Deviation 130](r:class.timespan.deviation.130)), positive or negative. With
two arguments they are hours and minutes, with three hours, minutes and seconds,
with four days, hours, minutes and seconds, and with five days, hours, minutes,
seconds and microseconds; a part that the form does not name is 0. The parts of
the multi-part forms are never negative: a span below zero is made by the
one-argument form or by [negating](r:class.timespan.minus) a span. No argument
raises `SYNTAX 93.901`, and more than five `SYNTAX 93.902`. An omitted argument
inside the list, before a given one, raises `SYNTAX 93.903`. `init` is an
ordinary message that can be sent again ([the init
rule](r:class.index.new.init-again)): it gives the instance the span that `new`
with the same arguments would make, after the same checks.

```rexx
say .timespan~new(1234567)
say .timespan~new('-1234567')
say .timespan~new(1, 2)
say .timespan~new(1, 2, 3)
say .timespan~new(1, 2, 3, 4)
say .timespan~new(1, 2, 3, 4, 5)
say .timespan~new(3, 7, 41, 52, 250)
say .timespan~new(0)
say .timespan~new(1, 2, )
t = .timespan~new(1, 2)
t~init(7)
say t
call try '.timespan~new'
call try '.timespan~new(1, 2, 3, 4, 5, 6)'
call try '.timespan~new(, 2)'
call try '.timespan~new(1, , 3)'
call try '.timespan~new(1, 2, 3, , 5)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
00:00:01.234567
-00:00:01.234567
01:02:00.000000
01:02:03.000000
1.02:03:04.000000
1.02:03:04.000005
3.07:41:52.000250
00:00:00.000000
01:02:00.000000
00:00:00.000007
.timespan~new => 93.901
.timespan~new(1, 2, 3, 4, 5, 6) => 93.902
.timespan~new(, 2) => 93.903
.timespan~new(1, , 3) => 93.903
.timespan~new(1, 2, 3, , 5) => 93.903
```

r[class.timespan.new.checks]
The values are checked in this order, and the first that fails raises its error.
With one argument it must be a whole number, read here and below by [the form
rule](r:conv.whole.form) and [the digits rule](r:conv.whole.digits) with the
codes of this rule in place of SYNTAX 40.12 ([Deviation
130](r:class.timespan.deviation.130)): another value raises `SYNTAX 88.903`, and
so does a count of 1000000000000000000 or more, or of that size below zero; a
count of a smaller size that is outside the range of [the value
rule](r:class.timespan.value) raises `SYNTAX 40.19`. In the forms of two to five
arguments the days, when given, are checked first: they must be a whole number
from 0 to 999999999, or `SYNTAX 88.904` is raised. Hours (0 to 23), minutes (0
to 59), seconds (0 to 59) and microseconds (0 to 999999) are checked next, in
the order of the arguments, each a whole number in its range; a value outside
it, one that is no number and one that is not whole, such as 1.5, raise `SYNTAX
88.907`. Last, a total outside the range of the class raises `SYNTAX 40.19`:
3652059 days with every other part in range is refused so, and the same days
with an hour of 99 are refused for the hour.

```rexx
call try '.timespan~new(3652058, 23, 59, 59, 999999)'
call try '.timespan~new(3652059, 0, 0, 0, 0)'
call try '.timespan~new(3652059, 99, 0, 0, 0)'
call try '.timespan~new(x, 99, 0, 0, 0)'
call try '.timespan~new(-1, 0, 0, 0, 0)'
call try '.timespan~new(1.5, 0, 0, 0, 0)'
call try '.timespan~new(999999999, 0, 0, 0)'
call try '.timespan~new(1000000000, 0, 0, 0)'
call try '.timespan~new(0, 24, 0, 0, 0)'
call try '.timespan~new(0, 0, 60, 0, 0)'
call try '.timespan~new(0, 0, 0, 60, 0)'
call try '.timespan~new(0, 0, 0, 0, 1000000)'
call try '.timespan~new(0, 0, 0, 0, -1)'
call try '.timespan~new(24, 0, 0)'
call try '.timespan~new(1, "x", 0)'
call try '.timespan~new(1, 1.5)'
call try '.timespan~new(315537897599999999)'
call try '.timespan~new(315537897600000000)'
call try '.timespan~new("-315537897600000000")'
call try '.timespan~new(999999999999999999)'
call try '.timespan~new(1000000000000000000)'
call try '.timespan~new(1.5)'
call try '.timespan~new("x")'
call try '.timespan~new("")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
.timespan~new(3652058, 23, 59, 59, 999999) => 3652058.23:59:59.999999
.timespan~new(3652059, 0, 0, 0, 0) => 40.19
.timespan~new(3652059, 99, 0, 0, 0) => 88.907
.timespan~new(x, 99, 0, 0, 0) => 88.904
.timespan~new(-1, 0, 0, 0, 0) => 88.904
.timespan~new(1.5, 0, 0, 0, 0) => 88.904
.timespan~new(999999999, 0, 0, 0) => 40.19
.timespan~new(1000000000, 0, 0, 0) => 88.904
.timespan~new(0, 24, 0, 0, 0) => 88.907
.timespan~new(0, 0, 60, 0, 0) => 88.907
.timespan~new(0, 0, 0, 60, 0) => 88.907
.timespan~new(0, 0, 0, 0, 1000000) => 88.907
.timespan~new(0, 0, 0, 0, -1) => 88.907
.timespan~new(24, 0, 0) => 88.907
.timespan~new(1, "x", 0) => 88.907
.timespan~new(1, 1.5) => 88.907
.timespan~new(315537897599999999) => 3652058.23:59:59.999999
.timespan~new(315537897600000000) => 40.19
.timespan~new("-315537897600000000") => 40.19
.timespan~new(999999999999999999) => 40.19
.timespan~new(1000000000000000000) => 88.903
.timespan~new(1.5) => 88.903
.timespan~new("x") => 88.903
.timespan~new("") => 88.903
```

r[class.timespan.from]
`fromDays`, `fromHours`, `fromMinutes`, `fromSeconds` and `fromMicroseconds`
take one number, which may have a fraction and a sign, and answer the span of
that many days, hours, minutes, seconds or microseconds. The count of
microseconds is the number multiplied by the microseconds of the unit, rounded
to 18 significant digits whatever `NUMERIC DIGITS` the program has set, and
with a part of a microsecond that is left dropped toward zero: `fromSeconds`
of 0.0000019 is 1 microsecond and of -0.0000019 is -1. Each method sends `new`
to the receiver with that count as its only argument ([the sends
rule](r:class.timespan.sends.new)), so that a count outside the range raises
`SYNTAX 40.19` and one of 1000000000000000000 or more in size raises
`SYNTAX 88.903`, as [the checks of `new`](r:class.timespan.new.checks) say. A
value that is no number raises `SYNTAX 88.902`; a missing argument raises
`SYNTAX 93.901` and a second one `SYNTAX 93.902`.

```rexx
say .timespan~fromDays(1.5) .timespan~fromDays('-1.5')
say .timespan~fromHours(1.5) .timespan~fromMinutes(0.5),
  .timespan~fromSeconds(90)
say .timespan~fromMicroseconds(1234567) .timespan~fromMicroseconds(1.6),
  .timespan~fromMicroseconds('-1.6')
say .timespan~fromSeconds(0.0000019) .timespan~fromSeconds('-0.0000019')
say .timespan~fromMinutes(0.00000099) .timespan~fromDays(0.0000001)
say .timespan~fromSeconds(123456789012.1234567)
say .timespan~fromMicroseconds(123456789012345678.9)
say .timespan~fromSeconds(1e3) .timespan~fromDays(' 2 ') .timespan~fromDays(0)
numeric digits 4
say .timespan~fromSeconds(1234567.1234567) .timespan~fromHours(0.00123456)
numeric digits 9
call try '.timespan~fromDays(3652058)'
call try '.timespan~fromDays(3652059)'
call try '.timespan~fromDays("-3652059")'
call try '.timespan~fromDays(11574074)'
call try '.timespan~fromDays(11574075)'
call try '.timespan~fromSeconds(999999999999)'
call try '.timespan~fromSeconds(1000000000000)'
call try '.timespan~fromDays("x")'
call try '.timespan~fromHours("")'
call try '.timespan~fromMinutes(.nil)'
call try '.timespan~fromSeconds'
call try '.timespan~fromMicroseconds(1, 2)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1.12:00:00.000000 -1.12:00:00.000000
01:30:00.000000 00:00:30.000000 00:01:30.000000
00:00:01.234567 00:00:00.000001 -00:00:00.000001
00:00:00.000001 -00:00:00.000001
00:00:00.000059 00:00:00.008640
1428898.00:30:12.123457
1428898.00:30:12.345679
00:16:40.000000 2.00:00:00.000000 00:00:00.000000
14.06:56:07.123456 00:00:04.444416
.timespan~fromDays(3652058) => 3652058.00:00:00.000000
.timespan~fromDays(3652059) => 40.19
.timespan~fromDays("-3652059") => 40.19
.timespan~fromDays(11574074) => 40.19
.timespan~fromDays(11574075) => 88.903
.timespan~fromSeconds(999999999999) => 40.19
.timespan~fromSeconds(1000000000000) => 88.903
.timespan~fromDays("x") => 88.902
.timespan~fromHours("") => 88.902
.timespan~fromMinutes(.nil) => 88.902
.timespan~fromSeconds => 93.901
.timespan~fromMicroseconds(1, 2) => 93.902
```

r[class.timespan.fromtime]
`fromNormalTime`, `fromCivilTime` and `fromLongTime` read a time of day that
is written in the form of the option N, C or L of
[TIME](r:bif.time.out.clock): `hh:mm:ss`, `h:mmam` or `h:mmpm`, and
`hh:mm:ss.f` where `f` is one to six digits, taken as the leading digits of
the microseconds. The hour of a civil time has one or two digits and is from 1
to 12; `am` and `pm` are lower case. The answer is the span from midnight to
that time of day, so it is never negative and less than a day. Unlike the
methods of `DateTime` they take no offset. A string that is not of the form,
an hour past 23 (past 12 for the civil form, and 0 is no civil hour), and
minutes or seconds past 59 raise `SYNTAX 88.918`. Without the string each
method raises `SYNTAX 93.901`, and a second argument raises `SYNTAX 93.902`.
Each sends `new` to the receiver with the count of microseconds as its only
argument ([the sends rule](r:class.timespan.sends.new)).

```rexx
say .timespan~fromNormalTime('13:45:09') .timespan~fromNormalTime('00:00:00')
say .timespan~fromCivilTime('1:45pm') .timespan~fromCivilTime('01:45pm')
say .timespan~fromCivilTime('12:00am') .timespan~fromCivilTime('12:59pm')
say .timespan~fromLongTime('13:45:09.123456'),
    .timespan~fromLongTime('13:45:09.1'),
    .timespan~fromLongTime('13:45:09.0001')
call try '.timespan~fromNormalTime("24:00:00")'
call try '.timespan~fromNormalTime("12:60:00")'
call try '.timespan~fromNormalTime("1:00:00")'
call try '.timespan~fromNormalTime("13:45")'
call try '.timespan~fromCivilTime("0:30am")'
call try '.timespan~fromCivilTime("13:30pm")'
call try '.timespan~fromCivilTime("1:45PM")'
call try '.timespan~fromCivilTime("1:45 pm")'
call try '.timespan~fromLongTime("13:45:09")'
call try '.timespan~fromLongTime("13:45:09.1234567")'
call try '.timespan~fromNormalTime("")'
call try '.timespan~fromNormalTime'
call try '.timespan~fromCivilTime'
call try '.timespan~fromLongTime'
call try '.timespan~fromNormalTime("13:45:09", 0)'
call try '.timespan~fromCivilTime("1:45pm", 0)'
call try '.timespan~fromLongTime("13:45:09.1", 0)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
13:45:09.000000 00:00:00.000000
13:45:00.000000 13:45:00.000000
00:00:00.000000 12:59:00.000000
13:45:09.123456 13:45:09.100000 13:45:09.000100
.timespan~fromNormalTime("24:00:00") => 88.918
.timespan~fromNormalTime("12:60:00") => 88.918
.timespan~fromNormalTime("1:00:00") => 88.918
.timespan~fromNormalTime("13:45") => 88.918
.timespan~fromCivilTime("0:30am") => 88.918
.timespan~fromCivilTime("13:30pm") => 88.918
.timespan~fromCivilTime("1:45PM") => 88.918
.timespan~fromCivilTime("1:45 pm") => 88.918
.timespan~fromLongTime("13:45:09") => 88.918
.timespan~fromLongTime("13:45:09.1234567") => 88.918
.timespan~fromNormalTime("") => 88.918
.timespan~fromNormalTime => 93.901
.timespan~fromCivilTime => 93.901
.timespan~fromLongTime => 93.901
.timespan~fromNormalTime("13:45:09", 0) => 93.902
.timespan~fromCivilTime("1:45pm", 0) => 93.902
.timespan~fromLongTime("13:45:09.1", 0) => 93.902
```

r[class.timespan.fromstringformat]
`fromStringFormat` reads a span written in the form of [the string
rule](r:class.timespan.string): an optional minus sign, an optional count of
days and a period, hours, minutes and seconds with colons between them, a
period, and microseconds, as in `1.02:03:04.000005` or `-23:59:59.500000`. The
text that `string` answers for a span is read back as the same span, a negative
span under an hour included ([Deviation 133](r:class.timespan.deviation.133)).
The text is read strictly ([Deviation 132](r:class.timespan.deviation.132)):
only a text that `string` can answer is accepted, so that a string without the
microseconds, as `01:02:03`, one that is not of the form at all, such as `x` or
the empty string, one with a blank, a plus sign or a value out of range, and one
for a span outside the range of [the value rule](r:class.timespan.value) raise
`SYNTAX 88.918`. The method sends no `new`, and answers a plain `TimeSpan`
whatever its receiver is. A missing argument raises `SYNTAX 93.901` and a second
one `SYNTAX 93.902`.

```rexx
say .timespan~fromStringFormat('1.02:03:04.000005')
say .timespan~fromStringFormat('-1.02:03:04.000005')
say .timespan~fromStringFormat('01:02:03.000005')
say .timespan~fromStringFormat('-23:59:59.500000')
say .timespan~fromStringFormat('00:00:00.000000')
say .timespan~fromStringFormat('3652058.23:59:59.999999')
say .timespan~fromStringFormat('-3652058.23:59:59.999999')
do s over .array~of(0, 1, 90000000, 86400000000, '-86400000000',,
    '-3600000000', '-5400000001', 1234567890123)
  t = .timespan~new(s)
  say t~string (.timespan~fromStringFormat(t~string) = t)
end
call try '.timespan~fromStringFormat("01:02:03")'
call try '.timespan~fromStringFormat("1.02:03")'
call try '.timespan~fromStringFormat("x")'
call try '.timespan~fromStringFormat("")'
call try '.timespan~fromStringFormat("3652059.00:00:00.000000")'
call try '.timespan~fromStringFormat'
call try '.timespan~fromStringFormat("01:02:03.000005", 1)'
call try '.sub~fromStringFormat("1.02:03:04.000005")~class~id'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class sub subclass timespan
```

```output
1.02:03:04.000005
-1.02:03:04.000005
01:02:03.000005
-23:59:59.500000
00:00:00.000000
3652058.23:59:59.999999
-3652058.23:59:59.999999
00:00:00.000000 1
00:00:00.000001 1
00:01:30.000000 1
1.00:00:00.000000 1
-1.00:00:00.000000 1
-01:00:00.000000 1
-01:30:00.000001 1
14.06:56:07.890123 1
.timespan~fromStringFormat("01:02:03") => 88.918
.timespan~fromStringFormat("1.02:03") => 88.918
.timespan~fromStringFormat("x") => 88.918
.timespan~fromStringFormat("") => 88.918
.timespan~fromStringFormat("3652059.00:00:00.000000") => 88.918
.timespan~fromStringFormat => 93.901
.timespan~fromStringFormat("01:02:03.000005", 1) => 93.902
.sub~fromStringFormat("1.02:03:04.000005")~class~id => TimeSpan
```

## Reading a span

r[class.timespan.parts]
`days`, `hours`, `minutes`, `seconds` and `microseconds` take no argument and
answer the parts of the span as whole numbers that are never negative: they
describe its size, not its sign. `hours` is from 0 to 23, `minutes` and
`seconds` from 0 to 59, `microseconds` from 0 to 999999, and `days` the whole
days. A span of 90 seconds has 0 days, 0 hours, 1 minute and 30 seconds, and
the span of -90 seconds has the same parts. `sign` answers -1, 0 or 1 as the
span is below, at or above zero. `duration` answers a new span that is not
below zero and has the size of the receiver; this is stated for a span shorter
than 1000 seconds, one whose count has at most nine digits, and what
`duration` answers for a longer one is not stated here. It makes its answer by
sending `new` to the class of the receiver ([the sends
rule](r:class.timespan.sends.parts)). Any argument raises `SYNTAX 93.902` for
each of these methods.

```rexx
t = .timespan~new(3, 7, 41, 52, 250)
n = .timespan~new('-286912000250')
z = .timespan~new(0)
do s over .array~of(t, n, z)
  say s~days s~hours s~minutes s~seconds s~microseconds '|' s~sign
end
say t~days~class~id t~sign~class~id t~duration~class~id
do s over .array~of('325000100', '-325000100', 0, '-1', 1, '-86400000')
  say .timespan~new(s)~duration .timespan~new(s)~sign
end
say .timespan~new('-90000000')~minutes .timespan~new('-90000000')~seconds
say .timespan~new('-86400000000')~days .timespan~new('-86399999999')~days
say .timespan~new(315537897599999999)~days,
  .timespan~new('-315537897599999999')~hours
call try 't~days(1)'
call try 't~hours(1)'
call try 't~minutes(1)'
call try 't~seconds(1)'
call try 't~microseconds(1)'
call try 't~sign(1)'
call try 't~duration(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
3 7 41 52 250 | 1
3 7 41 52 250 | -1
0 0 0 0 0 | 0
String String TimeSpan
00:05:25.000100 1
00:05:25.000100 -1
00:00:00.000000 0
00:00:00.000001 -1
00:00:00.000001 1
00:01:26.400000 -1
1 30
1 0
3652058 23
t~days(1) => 93.902
t~hours(1) => 93.902
t~minutes(1) => 93.902
t~seconds(1) => 93.902
t~microseconds(1) => 93.902
t~sign(1) => 93.902
t~duration(1) => 93.902
```

r[class.timespan.total]
`totalDays`, `totalHours`, `totalMinutes`, `totalSeconds` and
`totalMicroseconds` answer the whole span in that unit, with a fraction when
it does not come out even and with the sign of the span; `totalMicroseconds`,
the count itself, is always a whole number. The others are the count divided
by the microseconds of the unit, in 18 significant digits, rounded, whatever
`NUMERIC DIGITS` the program has set, so that a long span has fewer decimals
and a short one more, and the same figure is written without an exponent and
without trailing zeros. Any argument raises `SYNTAX 93.902`.

```rexx
t = .timespan~new(3, 7, 41, 52, 250)
n = .timespan~new('-286912000250')
do s over .array~of(t, n, .timespan~new(0), .timespan~new(1))
  say s~totalDays s~totalHours
  say s~totalMinutes s~totalSeconds s~totalMicroseconds
end
numeric digits 4
say t~totalDays t~totalSeconds
numeric digits 9
m = .timespan~new(315537897599999999)
say m~totalDays m~totalHours m~totalSeconds
say .timespan~fromDays(1.5)~totalDays .timespan~fromHours(36)~totalDays
say .timespan~fromSeconds(1)~totalMinutes,
  .timespan~fromSeconds(100)~totalHours
call try 't~totalDays(1)'
call try 't~totalHours(1)'
call try 't~totalMinutes(1)'
call try 't~totalSeconds(1)'
call try 't~totalMicroseconds(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
3.32074074363425926 79.6977778472222222
4781.86667083333333 286912.00025 286912000250
-3.32074074363425926 -79.6977778472222222
-4781.86667083333333 -286912.00025 -286912000250
0 0
0 0 0
0.0000000000115740740740740741 0.000000000277777777777777778
0.0000000166666666666666667 0.000001 1
3.32074074363425926 286912.00025
3652058.99999999999 87649415.9999999997 315537897599.999999
1.5 1.5
0.0166666666666666667 0.0277777777777777778
t~totalDays(1) => 93.902
t~totalHours(1) => 93.902
t~totalMinutes(1) => 93.902
t~totalSeconds(1) => 93.902
t~totalMicroseconds(1) => 93.902
```

r[class.timespan.string]
`string` and `makeString` take no argument and answer the span as text in the
form `-ddddd.hh:mm:ss.uuuuuu`: a minus sign when the span is below zero, the
days and a period only when the span has days, with no leading zeros, then
hours, minutes and seconds with two digits each, and the microseconds with
six. So a span of no days and no minus sign is 15 characters long, and the
string value of a span is this text ([the string-value
rule](r:class.index.string)). `makeString` answers what `string` answers, and
both answer a `String`. Any argument raises `SYNTAX 93.902`. The messages that
they send are in [the sends rule](r:class.timespan.sends.parts).

```rexx
do s over .array~of(0, 1, 90000000, '-90000000', 86400000000,,
    '-86400000000', 93784000005, 1234567890123, 315537897599999999)
  t = .timespan~new(s)
  say t~string t~makeString t~string~class~id (t~string == t~makeString)
end
say .timespan~new(0)~string~length .timespan~new(1, 2, 3, 4, 5)~string~length
say .timespan~new(10, 2, 3)~string .timespan~new(0, 2, 3)~makeString
call try '.timespan~new(1)~string(1)'
call try '.timespan~new(1)~makeString(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
00:00:00.000000 00:00:00.000000 String 1
00:00:00.000001 00:00:00.000001 String 1
00:01:30.000000 00:01:30.000000 String 1
-00:01:30.000000 -00:01:30.000000 String 1
1.00:00:00.000000 1.00:00:00.000000 String 1
-1.00:00:00.000000 -1.00:00:00.000000 String 1
1.02:03:04.000005 1.02:03:04.000005 String 1
14.06:56:07.890123 14.06:56:07.890123 String 1
3652058.23:59:59.999999 3652058.23:59:59.999999 String 1
15 17
10:02:03.000000 00:02:03.000000
.timespan~new(1)~string(1) => 93.902
.timespan~new(1)~makeString(1) => 93.902
```

## Arithmetic

r[class.timespan.add]
Each of `addWeeks`, `addDays`, `addHours`, `addMinutes`, `addSeconds` and
`addMicroseconds` takes one number and answers a new span, of the class of the
receiver, that is the receiver lengthened by that many weeks, days, hours,
minutes, seconds or microseconds, and shortened by a negative number; the
receiver is not changed. A fraction is allowed: `addDays(1.5)` is a day and
twelve hours. The new count of microseconds is the receiver's count plus the
amount in microseconds, the sum rounded to 18 significant digits, and a part
of a microsecond that is left is dropped toward zero, whatever the sign of the
sum (a span of -5 microseconds that is given 0.5 microseconds becomes -4, and
one of 5 given -5.5 becomes 0). A value that is no number raises
`SYNTAX 88.902`, a sum outside the range of [the value
rule](r:class.timespan.value) raises `SYNTAX 40.19`, and an amount of
1000000000000000000 microseconds or more in size raises `SYNTAX 88.903`. A
missing argument raises `SYNTAX 93.901` and a second `SYNTAX 93.902`.

```rexx
t = .timespan~new(1, 2, 3, 4, 5)
say t~addDays(1) t~addDays(-1) t~addDays(0.5) t~addDays(0)
say t~addWeeks(1) t~addWeeks(-1) t~addWeeks(0.5)
say t~addHours(1) t~addHours(-26) t~addHours(0.5)
say t~addMinutes(1) t~addMinutes(-1500) t~addMinutes(0.5)
say t~addSeconds(1) t~addSeconds(-0.5) t~addSeconds(0.0000015)
say t~addMicroseconds(1) t~addMicroseconds(-6) t~addMicroseconds(1.5),
  t~addMicroseconds(-1.5)
say t~addMicroseconds(0.96) t~addMicroseconds(-0.96)
say .timespan~new(-5)~addMicroseconds(0.5),
  .timespan~new(-5)~addMicroseconds(-0.5)
say .timespan~new(5)~addMicroseconds(-5.5),
  .timespan~new(0)~addMicroseconds(-1.5)
say .timespan~new(0)~addDays(-1.5) .timespan~new(0)~addDays(1.5)
say t~addDays(1)~class~id (t~addDays(0) = t) (t~addDays(1) = t)
u = .timespan~new(0)~addMicroseconds(1e17)
say u~addMicroseconds(1e17)~addMicroseconds(1e17)
call try 't~addDays(3652058)'
call try 't~addDays("-3652059")'
call try 't~addDays(1e9)'
call try 't~addHours(1e8)'
call try 't~addMicroseconds(1e18)'
call try 't~addDays("x")'
call try 't~addWeeks("")'
call try 't~addMicroseconds(.nil)'
call try 't~addDays'
call try 't~addSeconds(1, 2)'
call try 't~addMonths(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
2.02:03:04.000005 02:03:04.000005 1.14:03:04.000005 1.02:03:04.000005
8.02:03:04.000005 -5.21:56:55.999995 4.14:03:04.000005
1.03:03:04.000005 00:03:04.000005 1.02:33:04.000005
1.02:04:04.000005 01:03:04.000005 1.02:03:34.000005
1.02:03:05.000005 1.02:03:03.500005 1.02:03:04.000006
1.02:03:04.000006 1.02:03:03.999999 1.02:03:04.000006 1.02:03:04.000003
1.02:03:04.000005 1.02:03:04.000004
-00:00:00.000004 -00:00:00.000005
00:00:00.000000 -00:00:00.000001
-1.12:00:00.000000 1.12:00:00.000000
TimeSpan 1 0
3472222.05:20:00.000000
t~addDays(3652058) => 40.19
t~addDays("-3652059") => -3652057.21:56:55.999995
t~addDays(1e9) => 88.903
t~addHours(1e8) => 40.19
t~addMicroseconds(1e18) => 88.903
t~addDays("x") => 88.902
t~addWeeks("") => 88.902
t~addMicroseconds(.nil) => 88.902
t~addDays => 93.901
t~addSeconds(1, 2) => 93.902
t~addMonths(1) => 97.1
```

r[class.timespan.plus]
`+` takes a `TimeSpan` or a `DateTime`. With a `TimeSpan` it answers a new span,
of the class of the receiver, whose count is the sum of the two counts. With a
`DateTime` it answers a new instance of the class of that argument: the moment
that is the argument's fields moved forward by the span (back for a negative
span), with the offset of the argument ([Deviation
127](r:class.timespan.deviation.127)), so that `span + moment` and [`moment +
span`](r:class.datetime.plus) are equal, with the same fields and offset. Any
other argument raises `SYNTAX 88.914`; a result outside the range of [the value
rule](r:class.timespan.value) or of the moments of `DateTime` raises `SYNTAX
40.19`. The prefix `+` has no argument and answers a new span of the class of
the receiver with the same count. A second argument raises `SYNTAX 93.902`; the
message `+` with no argument is the prefix form.

```rexx
a = .timespan~new(1, 2, 3, 4, 5)
b = .timespan~fromSeconds(90)
say a + b a + a~copy a + .timespan~new(0)
say (a + b)~class~id (a + b = b + a) (+a) (+a)~class~id (+a == a)
say b + .timespan~new('-90000000') .timespan~new('-5') + .timespan~new('-5')
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 120)
say a + d
say ((a + d)~string == (d + a)~string)
say (a + d)~class~id (a + .sub~new(2016, 1, 1))~class~id
say .timespan~fromDays(1) + .datetime~minDate
say .timespan~new('-86400000000') + .datetime~new(2016, 1, 1)
say a~'+'
call try 'a + 5'
call try 'a + "x"'
call try 'a + .nil'
call try 'a + a~class'
call try 'a + .timespan~new(315537897599999999)'
call try '.timespan~new(315537897599999999) + a'
call try 'a + .datetime~maxDate'
call try 'a~"+"(b, 1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class sub subclass datetime
```

```output
1.02:04:34.000005 2.04:06:08.000010 1.02:03:04.000005
TimeSpan 1 1.02:03:04.000005 TimeSpan 1
00:00:00.000000 -00:00:00.000010
2016-03-01T11:11:11.012350
1
DateTime SUB
0001-01-02T00:00:00.000000
2015-12-31T00:00:00.000000
1.02:03:04.000005
a + 5 => 88.914
a + "x" => 88.914
a + .nil => 88.914
a + a~class => 88.914
a + .timespan~new(315537897599999999) => 40.19
.timespan~new(315537897599999999) + a => 40.19
a + .datetime~maxDate => 40.19
a~"+"(b, 1) => 93.902
```

r[class.timespan.minus]
`-` has two forms. With a `TimeSpan` as its argument it answers a new span, of
the class of the receiver, whose count is the receiver's less the argument's.
With no argument, as the prefix `-`, it answers a new span of the class of the
receiver with the negated count; the negation of a span of 0 is 0. Any other
argument, a `DateTime` included, raises `SYNTAX 93.948`, and a result outside
the range of [the value rule](r:class.timespan.value) raises `SYNTAX 40.19`. A
second argument raises `SYNTAX 93.902`; the message `-` with no argument is
the prefix form.

```rexx
a = .timespan~new(1, 2, 3, 4, 5)
b = .timespan~fromSeconds(90)
say a - b b - a a - a
say (-a) (-b) (-.timespan~new(0))
say (-a)~class~id (a - b)~class~id (a - b + b = a) (- -a == a)
say .timespan~new(315537897599999999) - .timespan~new(1)
say -.timespan~new(315537897599999999)
say .timespan~new('-315537897599999999') - .timespan~new(0)
call try 'a - 5'
call try 'a - "x"'
call try 'a - .nil'
call try 'a - .datetime~new(2016, 1, 1)'
call try '.timespan~new(315537897599999999) - -a'
call try '.timespan~new("-315537897599999999") - a'
call try 'a~"-"(b, 1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1.02:01:34.000005 -1.02:01:34.000005 00:00:00.000000
-1.02:03:04.000005 -00:01:30.000000 00:00:00.000000
TimeSpan TimeSpan 1 1
3652058.23:59:59.999998
-3652058.23:59:59.999999
-3652058.23:59:59.999999
a - 5 => 93.948
a - "x" => 93.948
a - .nil => 93.948
a - .datetime~new(2016, 1, 1) => 93.948
.timespan~new(315537897599999999) - -a => 40.19
.timespan~new("-315537897599999999") - a => 40.19
a~"-"(b, 1) => 93.902
```

r[class.timespan.times]
`*` takes one number, which may have a fraction and a sign, and answers a new
span, of the class of the receiver, whose count is the receiver's count
multiplied by the number: the product is rounded to 18 significant digits,
whatever `NUMERIC DIGITS` the program has set, and a part of a microsecond
that is left is dropped toward zero. The argument is read as a number: a
string that is no number, `.nil` and another span raise `SYNTAX 41.1`. A
product outside the range of [the value rule](r:class.timespan.value) raises
`SYNTAX 40.19`, and one of 1000000000000000000 microseconds or more in size
raises `SYNTAX 88.903`. A missing argument raises `SYNTAX 93.901` and a second
one `SYNTAX 93.902`.

```rexx
a = .timespan~new(1, 2, 3, 4, 5)
b = .timespan~fromSeconds(90)
say a * 2 a * 0.5 a * -1
say a * 0 a * ' 2 ' a * '2e1'
say b * 1.5 b * 1.0000005 b * 0.0000005 b * -0.0000005
say .timespan~new(10) * 0.25 .timespan~new(10) * -0.25 .timespan~new(-7) * 0.9
say .timespan~new(100) * 1.005 .timespan~new(10) * 1e16
m = .timespan~new(315537897599999999)
say m * 0.5 m * 0.3 m * 0.99999999999999
say m * 1e-10 m * 1e-18
numeric digits 4
say .timespan~new(10000000000) * 1.23456789 b * 1.5
numeric digits 9
say (a * 2)~class~id (a * 2 * 2)
call try 'a * "x"'
call try 'a * ""'
call try 'a * b'
call try 'a * .nil'
call try 'm * 1.0000000000000001'
call try 'm * 2'
call try '.timespan~new(10) * 1e17'
call try 'a~"*"'
call try 'a~"*"(2, 3)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
2.04:06:08.000010 13:01:32.000002 -1.02:03:04.000005
00:00:00.000000 2.04:06:08.000010 21.17:01:20.000100
00:02:15.000000 00:01:30.000045 00:00:00.000045 -00:00:00.000045
00:00:00.000002 -00:00:00.000002 -00:00:00.000006
00:00:00.000100 1157407.09:46:40.000000
1826029.12:00:00.000000 1095617.16:47:59.999999 3652058.23:59:59.996844
00:00:31.553789 00:00:00.000000
03:25:45.678900 00:02:15.000000
TimeSpan 4.08:12:16.000020
a * "x" => 41.1
a * "" => 41.1
a * b => 41.1
a * .nil => 41.1
m * 1.0000000000000001 => 40.19
m * 2 => 40.19
.timespan~new(10) * 1e17 => 88.903
a~"*" => 93.901
a~"*"(2, 3) => 93.902
```

r[class.timespan.divide]
`/` and `%` take one number and answer the same new span, of the class of the
receiver, whose count is the integer part of the receiver's count divided by
the number: the quotient is cut off toward zero, not rounded. That is the
whole number that the operator `%` gives to the count, so it has at most 18
digits; a quotient with more raises `SYNTAX 26.11`. A number of 0 raises
`SYNTAX 42.3`, and a string that is no number, `.nil` or another span raises
`SYNTAX 41.1`. A quotient outside the range of [the value
rule](r:class.timespan.value) raises `SYNTAX 40.19`. A missing argument raises
`SYNTAX 93.901` and a second one `SYNTAX 93.902`. A subclass that overrides
`%` changes `/` too ([the sends rule](r:class.timespan.sends.arithmetic)).

```rexx
a = .timespan~new(1, 2, 3, 4, 5)
b = .timespan~fromSeconds(90)
say a / 2 a % 2 a / 0.5
say a / 3 a / 1000000
say b / 7 b % 7 (-b) / 7 (-b) % 7
say .timespan~new(10) / 4 .timespan~new(-7) / 2 .timespan~new(9) / 0.4
say a / ' 2 ' a / '2e0' (a / 2)~class~id (a / 2 = a % 2)
m = .timespan~new(315537897599999999)
say m / 3 m / 7 m / 1.00000000000001
say m / 1e17 m / 1e20
call try 'a / 0'
call try 'a % 0'
call try 'a / "x"'
call try 'a % .nil'
call try 'a / b'
call try 'm / 0.3'
call try 'm % 0.3'
call try 'm / 0.5'
call try 'a~"/"'
call try 'a~"%"(2, 3)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
13:01:32.000002 13:01:32.000002 2.04:06:08.000010
08:41:01.333335 00:00:00.093784
00:00:12.857142 00:00:12.857142 -00:00:12.857142 -00:00:12.857142
00:00:00.000002 -00:00:00.000003 00:00:00.000022
13:01:32.000002 13:01:32.000002 TimeSpan 1
1217352.23:59:59.999999 521722.17:08:34.285714 3652058.23:59:59.996843
00:00:00.000003 00:00:00.000000
a / 0 => 42.3
a % 0 => 42.3
a / "x" => 41.1
a % .nil => 41.1
a / b => 41.1
m / 0.3 => 26.11
m % 0.3 => 26.11
m / 0.5 => 40.19
a~"/" => 93.901
a~"%"(2, 3) => 93.902
```

r[class.timespan.remainder]
`//` takes one number and answers a new span, of the class of the receiver,
whose count is the remainder of the receiver's count divided by the number, as
the operator `//` gives it: the remainder has the sign of the receiver's
count, whatever the sign of the number. It must be a whole number of
microseconds: a divisor with a fraction that leaves another remainder raises
`SYNTAX 88.903`, and a quotient with more than 18 digits raises
`SYNTAX 26.12`. A number of 0 raises `SYNTAX 42.3`, and a string that is no
number, `.nil` or another span raises `SYNTAX 41.1`. A missing argument raises
`SYNTAX 93.901` and a second one `SYNTAX 93.902`.

```rexx
a = .timespan~new(1, 2, 3, 4, 5)
b = .timespan~fromSeconds(90)
say a // 7 a // -7 (-a) // 7 (-a) // -7
say b // 7 .timespan~new(7) // 2 .timespan~new(-7) // 2
say a // 1000000 a // 86400000000 (a // 7)~class~id
say .timespan~new(9) // 0.5 .timespan~new(9) // 4.5
m = .timespan~new(315537897599999999)
say m // 7 m // 1e18 m // 0.5
call try '.timespan~new(9) // 2.5'
call try '.timespan~new(9) // 0.4'
call try 'm // 0.3'
call try 'a // 0'
call try 'a // "x"'
call try 'a // .nil'
call try 'a // b'
call try 'a~"//"'
call try 'a~"//"(2, 3)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
00:00:00.000003 00:00:00.000003 -00:00:00.000003 -00:00:00.000003
00:00:00.000006 00:00:00.000001 -00:00:00.000001
00:00:00.000005 02:03:04.000005 TimeSpan
00:00:00.000000 00:00:00.000000
00:00:00.000001 3652058.23:59:59.999999 00:00:00.000000
.timespan~new(9) // 2.5 => 88.903
.timespan~new(9) // 0.4 => 88.903
m // 0.3 => 26.12
a // 0 => 42.3
a // "x" => 41.1
a // .nil => 41.1
a // b => 41.1
a~"//" => 93.901
a~"//"(2, 3) => 93.902
```

## Comparing

r[class.timespan.compare]
`compareTo` takes one argument and answers -1 when the receiver is the shorter
(the lesser) of the two spans, 0 for two spans of the same count and 1 when it
is the longer, comparing the signed counts, so that a negative span is less than
any that is not. The argument is sent `isA` with `TimeSpan`, and a `TimeSpan` is
then sent `totalMicroseconds`; the receiver is sent nothing. An argument that is
no `TimeSpan` raises `SYNTAX 88.914` as soon as `isA` has answered ([Deviation
126](r:class.timespan.deviation.126)), and the comparison operators raise it for
such an argument too, `.nil` excepted (see below). The comparison operators `=`,
`==`, `<`, `<=`, `>`, `>=`, `<>`, `><`, `\=`, `\==`, `<<`, `<<=`, `>>`, `>>=`,
`\<`, `\>`, `\<<` and `\>>` are those of
[`Orderable`](r:class.orderable.compare), which answer from `compareTo` of the
receiver, strict and non-strict ones alike: `==` is as `=`, and two spans of the
same count are equal however they were made. The text of the instances plays no
part. A missing argument raises `SYNTAX 93.901` and a second one `SYNTAX
93.902`; an operator whose argument is `.nil` does as [the `.nil`
rule](r:class.orderable.compare.nil) of `Orderable` says, and `compareTo(.nil)`
raises `SYNTAX 88.914`. The sort methods order spans by this method ([the sort
rule](r:class.array.sort)).

```rexx
a = .timespan~new(1, 2, 3, 4, 5)
b = .timespan~fromMicroseconds(a~totalMicroseconds)
c = .timespan~new(1, 2, 3, 4, 6)
n = .timespan~new('-5')
z = .timespan~new(0)
say a~compareTo(c) c~compareTo(a) a~compareTo(b) a~compareTo(a)
say n~compareTo(z) z~compareTo(n) n~compareTo(.timespan~new('-6'))
say (a = b) (a == b) (a \= b) (a < c) (a > c) (a <= b) (a >= c) (a <> c)
say (a >< b) (a << c) (a >> c) (a <<= b) (a >>= c) (a \== c)
say (a \< c) (a \> c) (a \<< b) (a \>> b)
say (n < z) (n << z) (z > n) (n = z) (z = .timespan~fromMicroseconds(0))
say (a = .nil) (a \= .nil) (a == .nil) (a <> .nil)
l = .array~of(c, a, z, n, .timespan~fromDays(1))
say show(l~copy~~sort)
say show(l~copy~~sortWith(.descendingComparator~new))
say show(l~copy~~sortWith(.comparator~new))
call try 'a~compareTo'
call try 'a~compareTo(b, c)'
call try 'a < .nil'
call try 'a \>> .nil'
exit
show: procedure
  s = ''
  do i over arg(1)
    s = s i~totalMicroseconds
  end
  return s~strip
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
-1 1 0 0
-1 1 1
1 1 0 1 0 1 0 1
0 1 0 1 0 1
0 1 1 1
1 1 1 0 1
0 1 0 1
-5 0 86400000000 93784000005 93784000006
93784000006 93784000005 86400000000 0 -5
-5 0 86400000000 93784000005 93784000006
a~compareTo => 93.901
a~compareTo(b, c) => 93.902
a < .nil => 93.973
a \>> .nil => 93.973
```

r[class.timespan.hashcode]
`hashCode` takes no argument and answers a string of 8 bytes that is computed
from the count of the span. Two spans of the same count have the same hash
code, so that equal spans are the same entry of a `Set` or `Table`, and an
instance has the hash code of its copy; spans of different counts may have
different ones. The bytes are not specified. Any argument raises
`SYNTAX 93.902`.

```rexx
a = .timespan~new(1, 2, 3, 4, 5)
b = .timespan~fromMicroseconds(a~totalMicroseconds)
c = .timespan~new(1, 2, 3, 4, 6)
say (a~hashCode == b~hashCode) (a~hashCode == a~copy~hashCode)
say a~hashCode~length .timespan~new(0)~hashCode~length
s = .set~new
s~put(a)
s~put(b)
s~put(c)
say s~items
t = .table~new
t[a] = 'x'
say t[b] t~items
call try 'a~hashCode(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 1
8 8
2
x 1
a~hashCode(1) => 93.902
```

## Messages that a program can override

r[class.timespan.sends.new]
The class methods `fromDays`, `fromHours`, `fromMinutes`, `fromSeconds`,
`fromMicroseconds`, `fromNormalTime`, `fromCivilTime` and `fromLongTime` send
`new` to the receiver with one argument, the count of microseconds, so that a
class that overrides its `new` sees them, and `new` sends `init` to the new
instance with the arguments it got ([the new rule](r:class.index.new)); the
`init` of `TimeSpan` sends nothing to the instance. `fromStringFormat` sends
nothing to the receiver and answers a plain `TimeSpan`. The count is checked by
`init` ([the checks of `new`](r:class.timespan.new.checks)), so a count outside
the range raises its error after `new` and `init` were sent. The first example
prints, for each call, the messages sent in order, with the number of arguments
after `NEW` and `INIT`. The numbers that a program passes are read for their
string value, and are sent further messages ([the argument
rule](r:class.index.string)). The one-argument form of `new` and
`fromMicroseconds` read the string value of their argument three times and
twice. In the other forms of `new` each argument is read, in the order of the
arguments, and sent `>=` with 0 and, but for the days, `<=` with the greatest
value of its range (23, 59, 59 and 999999), after which the days are sent `*`
with 24, the minutes `*` with 60 and, in the form of two or three arguments, the
hours `*` with 3600; the hours of the forms with days, the seconds and the
microseconds are read for their string value once more. A value that is not a
whole number, or a value for the days with more than nine digits, is refused
after it is read and before `>=` is sent; a count of the one-argument form is
refused after its first reading when it is no whole number or
1000000000000000000 or more in size, and after its second when it is outside the
range. `fromDays`, `fromHours`, `fromMinutes` and `fromSeconds` read their
argument and send it `*` with 24, 3600, 60 and 1000000. `fromNormalTime`,
`fromCivilTime` and `fromLongTime` read the string once, and `fromStringFormat`
once, and twice when it is refused. The second example passes objects whose
`makeString` answers a number, and whose `unknown` sends every other message to
that number, and prints the messages each object got.

```rexx
call probe 'new', '.spy~new(1, 2, 3)'
call probe 'new 1', '.spy~new(5)'
call probe 'new 5', '.spy~new(1, 2, 3, 4, 5)'
call probe 'fromDays', '.spy~fromDays(1.5)'
call probe 'fromHours', '.spy~fromHours(1.5)'
call probe 'fromMinutes', '.spy~fromMinutes(1.5)'
call probe 'fromSeconds', '.spy~fromSeconds(1.5)'
call probe 'fromMicroseconds', '.spy~fromMicroseconds(15)'
call probe 'fromNormalTime', '.spy~fromNormalTime("01:02:03")'
call probe 'fromCivilTime', '.spy~fromCivilTime("1:02pm")'
call probe 'fromLongTime', '.spy~fromLongTime("01:02:03.4")'
call probe 'fromStringFormat', '.spy~fromStringFormat("1.01:02:03.000004")'
call probe 'fromDays big', '.spy~fromDays(4000000)'
call probe 'fromDays x', '.spy~fromDays("x")'
exit
probe:
  .local~log = ''
  signal on syntax name bad
  interpret 'x =' arg(2)
  say arg(1)':' .local~log '=>' x~class~id
  return
bad: say arg(1)':' .local~log '=>' condition('o')~code
  return
::class spy subclass timespan
::method init
  .local~log = .local~log 'INIT'arg()
  forward class (super)
::method new class
  .local~log = .local~log 'NEW'arg()
  forward class (super)
```

```output
new:  NEW3 INIT3 => SPY
new 1:  NEW1 INIT1 => SPY
new 5:  NEW5 INIT5 => SPY
fromDays:  NEW1 INIT1 => SPY
fromHours:  NEW1 INIT1 => SPY
fromMinutes:  NEW1 INIT1 => SPY
fromSeconds:  NEW1 INIT1 => SPY
fromMicroseconds:  NEW1 INIT1 => SPY
fromNormalTime:  NEW1 INIT1 => SPY
fromCivilTime:  NEW1 INIT1 => SPY
fromLongTime:  NEW1 INIT1 => SPY
fromStringFormat:  => TimeSpan
fromDays big:  NEW1 INIT1 => 40.19
fromDays x:  => 88.902
```


```rexx
d = .num~new('d', 3)
h = .num~new('h', 3)
m = .num~new('m', 3)
s = .num~new('s', 3)
u = .num~new('u', 3)
call probe '2 args', '.timespan~new(h, m)'
call probe '3 args', '.timespan~new(h, m, s)'
call probe '4 args', '.timespan~new(d, h, m, s)'
call probe '5 args', '.timespan~new(d, h, m, s, u)'
call probe '1 arg', '.timespan~new(u)'
call probe 'big days', '.timespan~new(.num~new("d", 99999999999), h, m, s)'
call probe 'neg days', '.timespan~new(.num~new("d", -1), h, m, s)'
call probe 'big hours', '.timespan~new(d, .num~new("h", 99), m, s)'
n = .num~new('n', 3)
call probe 'fromDays', '.timespan~fromDays(n)'
call probe 'fromHours', '.timespan~fromHours(n)'
call probe 'fromMinutes', '.timespan~fromMinutes(n)'
call probe 'fromSeconds', '.timespan~fromSeconds(n)'
call probe 'fromMicroseconds', '.timespan~fromMicroseconds(n)'
call probe 'fromNormalTime',,
  '.timespan~fromNormalTime(.num~new("n", "01:02:03"))'
call probe 'fromStringFormat',,
  '.timespan~fromStringFormat(.num~new("n", "1.01:02:03.000004"))'
call probe 'fromStringFormat x',,
  '.timespan~fromStringFormat(.num~new("n", "x"))'
exit
probe:
  .local~log = ''
  signal on syntax name bad
  interpret 'x =' arg(2)
  call wrap arg(1)':' strip(.local~log)
  return
bad: call wrap arg(1)':' strip(.local~log) '=>' condition('o')~code
  return
wrap:
  line = word(arg(1), 1)
  do i = 2 to words(arg(1))
    if length(line word(arg(1), i)) > 76 then do
      say line
      line = '  'word(arg(1), i)
    end
    else line = line word(arg(1), i)
  end
  say line
  return
::class num
::method init
  expose v t
  use arg t, v
::method makeString
  expose v t
  .local~log = .local~log t':MAKESTRING'
  return v
::method request
  expose t
  .local~log = .local~log t':REQUEST'
  forward class (super)
::method unknown
  expose v t
  use arg name, args
  .local~log = .local~log t':'name'/'args[1]
  forward to (v) message (name) arguments (args)
```

```output defect=#164
2 args: h:REQUEST h:MAKESTRING h:>=/0 h:<=/23 m:REQUEST m:MAKESTRING m:>=/0
  m:<=/59 h:*/3600 m:*/60
3 args: h:REQUEST h:MAKESTRING h:>=/0 h:<=/23 m:REQUEST m:MAKESTRING m:>=/0
  m:<=/59 s:REQUEST s:MAKESTRING s:>=/0 s:<=/59 h:*/3600 m:*/60 s:REQUEST
  s:MAKESTRING
4 args: d:REQUEST d:MAKESTRING d:>=/0 h:REQUEST h:MAKESTRING h:>=/0 h:<=/23
  m:REQUEST m:MAKESTRING m:>=/0 m:<=/59 s:REQUEST s:MAKESTRING s:>=/0
  s:<=/59 d:*/24 h:REQUEST h:MAKESTRING m:*/60 s:REQUEST s:MAKESTRING
5 args: d:REQUEST d:MAKESTRING d:>=/0 h:REQUEST h:MAKESTRING h:>=/0 h:<=/23
  m:REQUEST m:MAKESTRING m:>=/0 m:<=/59 s:REQUEST s:MAKESTRING s:>=/0
  s:<=/59 u:REQUEST u:MAKESTRING u:>=/0 u:<=/999999 d:*/24 h:REQUEST
  h:MAKESTRING m:*/60 s:REQUEST s:MAKESTRING u:REQUEST u:MAKESTRING
1 arg: u:REQUEST u:MAKESTRING u:REQUEST u:MAKESTRING u:REQUEST u:MAKESTRING
big days: d:REQUEST d:MAKESTRING => 88.904
neg days: d:REQUEST d:MAKESTRING d:>=/0 => 88.904
big hours: d:REQUEST d:MAKESTRING d:>=/0 h:REQUEST h:MAKESTRING h:>=/0
  h:<=/23 => 88.907
fromDays: n:REQUEST n:MAKESTRING n:*/24
fromHours: n:REQUEST n:MAKESTRING n:*/3600
fromMinutes: n:REQUEST n:MAKESTRING n:*/60
fromSeconds: n:REQUEST n:MAKESTRING n:*/1000000
fromMicroseconds: n:REQUEST n:MAKESTRING n:REQUEST n:MAKESTRING
fromNormalTime: n:REQUEST n:MAKESTRING
fromStringFormat: n:REQUEST n:MAKESTRING
fromStringFormat x: n:REQUEST n:MAKESTRING n:REQUEST n:MAKESTRING => 88.918
```

r[class.timespan.sends.parts]
`string` sends the receiver `days`, `hours`, `minutes`, `seconds` and
`microseconds`, in this order, and `makeString` sends `string`. `duration`
sends `new` to the class of the receiver with one argument, the count of
microseconds, and no other message. `days`, `hours`, `minutes`, `seconds`,
`microseconds`, `sign`, `totalDays`, `totalHours`, `totalMinutes`,
`totalSeconds`, `totalMicroseconds` and `hashCode` send nothing. The example
gives a subclass a method for every one of these messages that logs it, and
prints what each call logged: the message itself first, then what it sent.

```rexx
names = 'DAYS HOURS MINUTES SECONDS MICROSECONDS SIGN DURATION TOTALDAYS',
  'TOTALHOURS TOTALMINUTES TOTALSECONDS TOTALMICROSECONDS STRING MAKESTRING',
  'HASHCODE'
do n over names~makearray(' ')
  body = .array~of(".local~log = .local~log '" || n || "'",,
    'forward class (super)')
  .spy~define(n, .method~new(n, body))
end
a = .spy~new(1, 2, 3, 4, 5)
calls = 'string makeString hashCode days hours minutes seconds',
  'microseconds sign duration totalDays totalHours totalMinutes',
  'totalSeconds totalMicroseconds'
do n over calls~makearray(' ')
  .local~log = ''
  interpret 'x = a~'n
  say n':' .local~log
end
say a~duration~class~id a~string~class~id
exit
::class spy subclass timespan
::method init
  .local~log = .local~log 'INIT'arg()
  forward class (super)
::method new class
  .local~log = .local~log 'NEW'arg()
  forward class (super)
```

```output
string:  STRING DAYS HOURS MINUTES SECONDS MICROSECONDS
makeString:  MAKESTRING STRING DAYS HOURS MINUTES SECONDS MICROSECONDS
hashCode:  HASHCODE
days:  DAYS
hours:  HOURS
minutes:  MINUTES
seconds:  SECONDS
microseconds:  MICROSECONDS
sign:  SIGN
duration:  DURATION NEW1 INIT1
totalDays:  TOTALDAYS
totalHours:  TOTALHOURS
totalMinutes:  TOTALMINUTES
totalSeconds:  TOTALSECONDS
totalMicroseconds:  TOTALMICROSECONDS
SPY String
```

r[class.timespan.sends.arithmetic]
`addDays`, `addHours`, `addMicroseconds`, `addMinutes`, `addSeconds`,
`addWeeks`, `*`, `%`, `//`, `+` and `-` with one argument, and the prefix `+`
and `-`, send `new` to the class of the receiver with one argument, the count of
microseconds of the answer, and nothing else to the receiver; `/` sends `%` to
the receiver with the argument it got and answers what that answers, so that a
subclass that overrides `%` changes `/`. The argument of `+` and `-` is sent
`isA` with `TimeSpan`, and a `TimeSpan` is then sent `totalMicroseconds`; when
the answer of `isA` is false, `-` raises its error, and `+` sends `isA` with
`DateTime` to the argument, and when that answers true sends it `offset` and
then sends `new` to the class of the argument with two arguments, the count of
microseconds of the moment and the answer of `offset` ([Deviation
127](r:class.timespan.deviation.127)); when it answers false `+` raises its
error. The number that the other methods take is read for its string value: once
by `*`, `/`, `%` and `//`, and twice by `addMicroseconds`; `addDays`,
`addHours`, `addMinutes`, `addSeconds` and `addWeeks` read it once and send it
`*` with the microseconds of a day, an hour, a minute, a second and a week,
86400000000, 3600000000, 60000000, 1000000 and 604800000000. The first example
gives a receiver and the span that it is added to methods that log every
message, and prints what each call logged: the message itself first, then what
it sent; the second passes an object that logs the messages that it got in the
place of a number.

```rexx
names = 'ADDDAYS ADDHOURS ADDMINUTES ADDSECONDS ADDMICROSECONDS ADDWEEKS',
  '+ - * / // % TOTALMICROSECONDS'
do n over names~makearray(' ')
  body = .array~of(".local~log = .local~log '" || n || "'",,
    'forward class (super)')
  .spy~define(n, .method~new(n, body))
end
a = .spy~new(1, 2, 3, 4, 5)
b = .arg~new(0, 0, 0, 1, 0)
d = .dt~new(2016, 2, 29, 9, 8, 7, 12345, 120)
o = .obj~new
call probe 'a + b', 'a + b'
call probe 'a - b', 'a - b'
call probe 'a + o', 'a + o'
call probe 'a - o', 'a - o'
call probe 'a - d', 'a - d'
call probe '+a', '+a'
call probe '-a', '-a'
call probe 'a * 2', 'a * 2'
call probe 'a / 2', 'a / 2'
call probe 'a % 2', 'a % 2'
call probe 'a // 2', 'a // 2'
call probe 'addDays', 'a~addDays(1)'
call probe 'addHours', 'a~addHours(1)'
call probe 'addMinutes', 'a~addMinutes(1)'
call probe 'addSeconds', 'a~addSeconds(1)'
call probe 'addMicroseconds', 'a~addMicroseconds(1)'
call probe 'addWeeks', 'a~addWeeks(1)'
exit
probe:
  .local~log = ''
  signal on syntax name bad
  interpret 'x =' arg(2)
  say arg(1)':' .local~log '=>' x~class~id
  return
bad: say arg(1)':' .local~log '=>' condition('o')~code
  return
::class spy subclass timespan
::method init
  .local~log = .local~log 'INIT'arg()
  forward class (super)
::method new class
  .local~log = .local~log 'NEW'arg()
  forward class (super)
::class arg subclass timespan
::method totalMicroseconds
  .local~log = .local~log 'ARG:TOTALMICROSECONDS'
  forward class (super)
::class dt subclass datetime
::method isA
  .local~log = .local~log 'dt:ISA('arg(1)~id')'
  forward class (super)
::method new class
  .local~log = .local~log 'dt:NEW'arg()
  forward class (super)
::class obj
::method isA
  .local~log = .local~log 'obj:ISA('arg(1)~id')'
  return 0
```

```output
a + b:  + ARG:TOTALMICROSECONDS NEW1 INIT1 => SPY
a - b:  - ARG:TOTALMICROSECONDS NEW1 INIT1 => SPY
a + o:  + obj:ISA(TimeSpan) obj:ISA(DateTime) => 88.914
a - o:  - obj:ISA(TimeSpan) => 93.948
a - d:  - dt:ISA(TimeSpan) => 93.948
+a:  + NEW1 INIT1 => SPY
-a:  - NEW1 INIT1 => SPY
a * 2:  * NEW1 INIT1 => SPY
a / 2:  / % NEW1 INIT1 => SPY
a % 2:  % NEW1 INIT1 => SPY
a // 2:  // NEW1 INIT1 => SPY
addDays:  ADDDAYS NEW1 INIT1 => SPY
addHours:  ADDHOURS NEW1 INIT1 => SPY
addMinutes:  ADDMINUTES NEW1 INIT1 => SPY
addSeconds:  ADDSECONDS NEW1 INIT1 => SPY
addMicroseconds:  ADDMICROSECONDS NEW1 INIT1 => SPY
addWeeks:  ADDWEEKS NEW1 INIT1 => SPY
```

```rexx
n = .num~new(3)
t = .timespan~new(1, 2, 3)
call probe 'addDays', 't~addDays(n)'
call probe 'addWeeks', 't~addWeeks(n)'
call probe 'addHours', 't~addHours(n)'
call probe 'addMinutes', 't~addMinutes(n)'
call probe 'addSeconds', 't~addSeconds(n)'
call probe 'addMicroseconds', 't~addMicroseconds(n)'
call probe '*', 't * n'
call probe '/', 't / n'
call probe '%', 't % n'
call probe '//', 't // n'
exit
probe:
  .local~log = ''
  interpret 'x =' arg(2)
  say arg(1)':' strip(.local~log)
  return
::class num
::method init
  expose v
  use arg v
::method makeString
  expose v
  .local~log = .local~log 'MAKESTRING'
  return v
::method request
  .local~log = .local~log 'REQUEST:'arg(1)
  forward class (super)
::method unknown
  expose v
  use arg name, args
  .local~log = .local~log name'/'args[1]
  forward to (v) message (name) arguments (args)
```

```output defect=#164
addDays: REQUEST:STRING MAKESTRING */86400000000
addWeeks: REQUEST:STRING MAKESTRING */604800000000
addHours: REQUEST:STRING MAKESTRING */3600000000
addMinutes: REQUEST:STRING MAKESTRING */60000000
addSeconds: REQUEST:STRING MAKESTRING */1000000
addMicroseconds: REQUEST:STRING MAKESTRING REQUEST:STRING MAKESTRING
*: REQUEST:STRING MAKESTRING
/: REQUEST:STRING MAKESTRING
%: REQUEST:STRING MAKESTRING
//: REQUEST:STRING MAKESTRING
```

r[class.timespan.sends.compare]
`compareTo` sends its argument `isA` with `TimeSpan` and, for a `TimeSpan`,
`totalMicroseconds`; the receiver is sent nothing, and an argument that is no
`TimeSpan` is sent nothing but `isA` ([Deviation
126](r:class.timespan.deviation.126)). A comparison operator sends `compareTo`
to the receiver with the argument it got, as [the sends rule of
`Orderable`](r:class.orderable.compare.sends) says in full; `<>`, `><`, `\<`,
`\>`, `\<<` and `\>>` send another operator first.

```rexx
names = 'COMPARETO = == < > <= >= <> >< << >> <<= >>= \= \== \< \> \<<',
  '\>>'
do n over names~makearray(' ')
  body = .array~of(".local~log = .local~log '" || n || "'",,
    'forward class (super)')
  .spy~define(n, .method~new(n, body))
end
a = .spy~new(1, 2, 3, 4, 5)
b = .arg~new(1, 2, 3, 4, 6)
do n over names~makearray(' ')
  .local~log = ''
  if n = 'COMPARETO' then x = a~compareTo(b)
  else interpret 'x = a' n 'b'
  say n':' .local~log '=>' x
end
exit
::class spy subclass timespan
::class arg subclass timespan
::method isA
  .local~log = .local~log 'ARG:ISA('arg(1)~id')'
  forward class (super)
::method totalMicroseconds
  .local~log = .local~log 'ARG:TOTALMICROSECONDS'
  forward class (super)
```

```output
COMPARETO:  COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => -1
=:  = COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 0
==:  == COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 0
<:  < COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 1
>:  > COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 0
<=:  <= COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 1
>=:  >= COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 0
<>:  <> \= COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 1
><:  >< \= COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 1
<<:  << COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 1
>>:  >> COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 0
<<=:  <<= COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 1
>>=:  >>= COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 0
\=:  \= COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 1
\==:  \== COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 1
\<:  \< >= COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 0
\>:  \> <= COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 1
\<<:  \<< >>= COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 0
\>>:  \>> <<= COMPARETO ARG:ISA(TimeSpan) ARG:TOTALMICROSECONDS => 1
```

## Deviations

r[class.timespan.deviation.109]
An instance of a subclass of `String` that a method takes as a string or a
number is read by its own characters, and `REQUEST`, `MAKESTRING` and `STRING`
are not sent to it ([Deviation 109](r:class.string.deviation.109)). ooRexx
sends such an instance `request` with `STRING` and then `makeString`, so an
override can change what is read. In the example `makeString` and `request`
answer a different string, and the instances still stand for their own
characters.

```rexx
.local~sent = 0
a = .digits~new('5')
b = .digits~new('01:02:03')
c = .digits~new('2')
call show '.timespan~fromDays(a) .timespan~new(a)'
call show '.timespan~fromNormalTime(b) .timespan~new(c, c, c)'
call show '.timespan~fromHours(c) * a .timespan~new(1, 2, 3)~addDays(c)'
call show ".timespan~fromStringFormat(.digits~new('1.02:03:04.000005'))"
say 'sends' .local~sent
exit
show:
  signal on syntax name failed
  interpret 'say' arg(1)
  return
failed: say '=>' condition('o')~code
  return
::class digits subclass string
::method makestring
  .local~sent = .local~sent + 1
  return 'zzz'
::method request
  .local~sent = .local~sent + 1
  return 'zzz'
```

```output defect=#140
5.00:00:00.000000 00:00:00.000005
01:02:03.000000 02:02:02.000000
10:00:00.000000 2.01:02:03.000000
1.02:03:04.000005
sends 0
```

r[class.timespan.deviation.126]
`compareTo` of a `TimeSpan` with an argument that is not a `TimeSpan` (an
instance of a subclass is one), and a comparison operator with such an argument,
raise `SYNTAX 88.914`. The error follows the `isA` that the method sends to the
argument and precedes `totalMicroseconds` and every other message; an operator
whose argument is `.nil` sends no `compareTo` and does as [the `.nil`
rule](r:class.orderable.compare.nil) says, and `compareTo` itself raises for
`.nil`. ooRexx answers 1 or -1 for such an argument, whatever the receiver is,
so the answer says nothing about the two values. In the example the last line is
a probe that answers false to `isA` and gets nothing else.

```rexx
a = .timespan~new(1, 2, 3)
call try 'a~compareTo(.datetime~new(2017, 6, 30, 5, 6, 7, 8, 0))'
call try 'a~compareTo(.file~new("/t/a"))'
call try 'a~compareTo("01:02:03.000000")'
call try 'a~compareTo(5)'
call try 'a~compareTo(.nil)'
call try 'a < .datetime~minDate'
call try 'a >>= "x"'
call try 'a = 3723000000'
call try 'a \== .object~new'
call try 'a = .nil'
call try 'a < .nil'
call try 'a~compareTo(.sub~new(1, 2, 3))'
call try 'a~compareTo(a~copy)'
.local~log = ''
call try 'a~compareTo(.probe~new)'
say 'sent:' .local~log
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class sub subclass timespan
::class probe
::method isA
  .local~log = .local~log 'ISA('arg(1)~id')'
  return 0
::method totalMicroseconds
  .local~log = .local~log 'TOTALMICROSECONDS'
  return 0
```

```output
a~compareTo(.datetime~new(2017, 6, 30, 5, 6, 7, 8, 0)) => 88.914
a~compareTo(.file~new("/t/a")) => 88.914
a~compareTo("01:02:03.000000") => 88.914
a~compareTo(5) => 88.914
a~compareTo(.nil) => 88.914
a < .datetime~minDate => 88.914
a >>= "x" => 88.914
a = 3723000000 => 88.914
a \== .object~new => 88.914
a = .nil => 0
a < .nil => 93.973
a~compareTo(.sub~new(1, 2, 3)) => 0
a~compareTo(a~copy) => 0
a~compareTo(.probe~new) => 88.914
sent:  ISA(TimeSpan)
```

r[class.timespan.deviation.127]
`+` of a `TimeSpan` with a `DateTime` answers an instance of the class of the
argument that has the offset of the argument, as [`+` of
`DateTime`](r:class.datetime.plus) does, so that `span + moment` and `moment +
span` are equal, with the same fields and offset. The method sends the argument
`isA` with `TimeSpan` and `isA` with `DateTime`, then `offset`, and then sends
the class of the argument `new` with two arguments, the microseconds of the
moment and the answer of `offset`. ooRexx gives the answer the offset of the
host's zone and sends `new` with one argument; see [Deviation
127](r:class.datetime.deviation.127) for `-` of a `DateTime`. The example gives
the offsets explicitly, so that it does not depend on the zone of the host.

```rexx
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 330)
t = .timespan~fromSeconds(90)
x = t + d
say x
say x~offset (x == d + t)
say (.timespan~fromDays(1) + d~toTimezone(-90))~utcIsoDate
say (t + .sub~new(2016, 2, 29))~class~id
m = .spy~new(2016, 2, 29, 9, 8, 7, 12345, 330)
.local~log = ''
x = t + m
say .local~log
exit
::class sub subclass datetime
::class spy subclass datetime
::method offset
  .local~log = .local~log 'OFFSET'
  forward class (super)
::method new class
  .local~log = .local~log 'NEW'arg()
  forward class (super)
::method isA
  .local~log = .local~log 'ISA('arg(1)~id')'
  forward class (super)
```

```output
2016-02-29T09:09:37.012345
05:30:00.000000 1
2016-03-01T02:08:07.012345-01:30
SUB
 ISA(TimeSpan) ISA(DateTime) OFFSET NEW2
```

r[class.timespan.deviation.130]
Every whole-number argument of `TimeSpan` is read as a whole-number argument of
a builtin is: its form by [the form rule](r:conv.whole.form) and its value by
[the digits rule](r:conv.whole.digits), so NUMERIC DIGITS of the caller plays no
part. A value that those rules refuse raises the error that the place gives for
a number that is not whole, in place of `SYNTAX 40.12`: `1.0000000001` and
`5.00000000000000001` are refused, and `2017.00000000000000000001` is 2017. The
places are the count of microseconds of the one-argument `new`, and its days,
hours, minutes, seconds and microseconds. ooRexx raises `SYNTAX 40.19` for a
fraction in the days, the hours, the minutes or the seconds that leaves a
fraction of a microsecond, such as `1.0000000001` days. It accepts a smaller
one, such as `4.00000000000001` seconds, and any fraction in the microseconds.
It reads the count as this rule does.

```rexx
say .timespan~new(1.0, 2E0, 3, 4.00, ' 5 ')
say .timespan~new(2E1) .timespan~new(' +7 ')
numeric digits 3
say .timespan~new(1234.0)
numeric digits 9
call try '.timespan~new(1.0000000001, 2, 3, 4, 5)'
call try '.timespan~new(1, 2.0000000001, 3, 4, 5)'
call try '.timespan~new(1, 2, 3, 4, 5.0000000001)'
call try '.timespan~new(1, 2, 3, 4.00000000000001, 5)'
call try '.timespan~new(5.0000000001)'
call try '.timespan~new(2017.00000000000000000001)'
call try '.timespan~new(5.00000000000000001)'
call try '.timespan~new(5.000000000000000001)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1.02:03:04.000005
00:00:00.000020 00:00:00.000007
00:00:00.001234
.timespan~new(1.0000000001, 2, 3, 4, 5) => 88.904
.timespan~new(1, 2.0000000001, 3, 4, 5) => 88.907
.timespan~new(1, 2, 3, 4, 5.0000000001) => 88.907
.timespan~new(1, 2, 3, 4.00000000000001, 5) => 88.907
.timespan~new(5.0000000001) => 88.903
.timespan~new(2017.00000000000000000001) => 00:00:00.002017
.timespan~new(5.00000000000000001) => 88.903
.timespan~new(5.000000000000000001) => 00:00:00.000005
```

r[class.timespan.deviation.131]
An instance of a subclass of `TimeSpan` whose `init` does not forward to the
`init` of `TimeSpan` is the span of 0, and every method works on it as on that
value; the class of an answer is the class of the receiver where the method says
so, and a copy of it is one too. ooRexx answers with errors from most methods,
or with text that is no value.

```rexx
t = .st~new
z = .timespan~new(0)
h = .timespan~fromHours(1)
call show 't t~totalMicroseconds t~sign t~string t~days t~hours t~duration'
call show '(t = z) (t == z) (t~compareTo(z)) (t~hashCode == z~hashCode)'
call show '(t + h) (t - h) (-t) (t * 3) (t / 3) (t // 3)'
call show '(t + .datetime~minDate) t~addDays(1) t~copy t~copy~class~id'
exit
show:
  signal on syntax name failed
  interpret 'say' arg(1)
  return
failed: say '=>' condition('o')~code
  return
::class st subclass timespan
::method init
  nop
```

```output
00:00:00.000000 0 0 00:00:00.000000 0 0 00:00:00.000000
1 1 0 1
01:00:00.000000 -01:00:00.000000 00:00:00.000000 00:00:00.000000 00:00:00.000000 00:00:00.000000
0001-01-01T00:00:00.000000 1.00:00:00.000000 00:00:00.000000 ST
```

r[class.timespan.deviation.132]
`fromStringFormat` refuses with `SYNTAX 88.918` every string that [the string
rule](r:class.timespan.string) cannot have written: no blank anywhere and no
plus sign; the days only when the span has days, written without leading zeros
and not as 0, and at most 3652058; hours (00 to 23), minutes and seconds (00 to
59) with two digits each, exactly six digits of microseconds, and no minus sign
before a span of 0. The first lines of the example show the accepted forms and
the rest the refused ones. ooRexx refuses a doubled minus sign, a point with no
digits after it, another separator between the parts or a fourth part and a span
past the last one; it accepts the others, reading some in its own way:
`00:61:30.000000` as one hour and 1 minute and 30 seconds, `24:00:00.000000` as
one day, and `00:01:30.5` as 5 microseconds.

```rexx
good = .array~of('1.02:03:04.000005', '-1.02:03:04.000005', '23:59:59.999999',,
  '-00:00:00.000001', '3652058.23:59:59.999999', '10.00:00:00.000000')
do s over good
  say s '=>' .timespan~fromStringFormat(s)~totalMicroseconds
end
bad = .array~of('+1.02:03:04.000005', ' 01:02:03.000005', '01:02:03.000005 ',,
  '- 01:02:03.000005', '--01:02:03.000005', '-00:00:00.000000',,
  '1.2:03:04.000005', '1.02:3:04.000005', '1.02:03:4.000005',,
  '1:02:03.000005', '00:01:30.0000005', '00:01:30.5', '00:01:30.',,
  '00:61:30.000000', '00:00:60.000000', '24:00:00.000000',,
  '1.24:00:00.000000', '1.02:03:04.000005.', '1. 02:03:04.000005',,
  '00001.02:03:04.000005', '0.02:03:04.000005', '-0.00:01:30.000000',,
  '1.02-03-04.000005', '01:02:03:04.000005', '3652058.24:00:00.000000')
do s over bad
  call try s
end
exit
try:
  signal on syntax name bad
  r = .timespan~fromStringFormat(arg(1))
  say '['arg(1)'] =>' r
  return
bad: say '['arg(1)'] =>' condition('o')~code
  return
```

```output
1.02:03:04.000005 => 93784000005
-1.02:03:04.000005 => -93784000005
23:59:59.999999 => 86399999999
-00:00:00.000001 => -1
3652058.23:59:59.999999 => 315537897599999999
10.00:00:00.000000 => 864000000000
[+1.02:03:04.000005] => 88.918
[ 01:02:03.000005] => 88.918
[01:02:03.000005 ] => 88.918
[- 01:02:03.000005] => 88.918
[--01:02:03.000005] => 88.918
[-00:00:00.000000] => 88.918
[1.2:03:04.000005] => 88.918
[1.02:3:04.000005] => 88.918
[1.02:03:4.000005] => 88.918
[1:02:03.000005] => 88.918
[00:01:30.0000005] => 88.918
[00:01:30.5] => 88.918
[00:01:30.] => 88.918
[00:61:30.000000] => 88.918
[00:00:60.000000] => 88.918
[24:00:00.000000] => 88.918
[1.24:00:00.000000] => 88.918
[1.02:03:04.000005.] => 88.918
[1. 02:03:04.000005] => 88.918
[00001.02:03:04.000005] => 88.918
[0.02:03:04.000005] => 88.918
[-0.00:01:30.000000] => 88.918
[1.02-03-04.000005] => 88.918
[01:02:03:04.000005] => 88.918
[3652058.24:00:00.000000] => 88.918
```

r[class.timespan.deviation.133]
`fromStringFormat` of the text of a negative span answers a negative span
whatever its size; one under an hour, which has no days and no hours, keeps its
sign too, and `sign` of the answer is -1. ooRexx answers the positive span for
such a text.

```rexx
do s over .array~of('-1', '-90000000', '-3599999999', '-3600000000',,
    '-5400000001', '-86400000000', '-86400000001')
  t = .timespan~new(s)
  r = .timespan~fromStringFormat(t~string)
  say t~string r~string r~sign (r = t)
end
say .timespan~fromStringFormat('-00:00:30.000000')~totalSeconds
say .timespan~fromStringFormat('-00:59:59.999999')~seconds
say .timespan~fromStringFormat('00:00:30.000000')~totalSeconds
```

```output
-00:00:00.000001 -00:00:00.000001 -1 1
-00:01:30.000000 -00:01:30.000000 -1 1
-00:59:59.999999 -00:59:59.999999 -1 1
-01:00:00.000000 -01:00:00.000000 -1 1
-01:30:00.000001 -01:30:00.000001 -1 1
-1.00:00:00.000000 -1.00:00:00.000000 -1 1
-1.00:00:00.000001 -1.00:00:00.000001 -1 1
-30
59
30
```

r[class.timespan.deviation.163]
An `init` sent to a `TimeSpan` that raises an error leaves the receiver as it
was: every argument is checked before the instance is changed ([Deviation
163](../deviations.md)). ooRexx stores an argument that is not a number as the
span before it raises the error, so that the span then raises errors of its
own, and the example is not run against it.

```rexx
t = .timespan~new(0, 1, 2, 3)
before = t~string
signal on syntax
t~init('x')
exit
syntax:
say condition('O')~code (t~string == before) t
```

```output defect=#441
88.903 1 01:02:03.000000
```
