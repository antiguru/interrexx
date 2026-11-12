---
element: "DateTime"
kind: class
rexxref: "Builtin Classes > Utility Classes > DateTime Class; Builtin Classes > Utility Classes > DateTime Class > init; Builtin Classes > Utility Classes > DateTime Class > fromBaseDate (Class Method); Builtin Classes > Utility Classes > DateTime Class > fromCivilTime (Class Method); Builtin Classes > Utility Classes > DateTime Class > fromEuropeanDate (Class Method); Builtin Classes > Utility Classes > DateTime Class > fromIsoDate (Class Method); Builtin Classes > Utility Classes > DateTime Class > fromLongTime (Class Method); Builtin Classes > Utility Classes > DateTime Class > fromNormalDate (Class Method); Builtin Classes > Utility Classes > DateTime Class > fromNormalTime (Class Method); Builtin Classes > Utility Classes > DateTime Class > fromOrderedDate (Class Method); Builtin Classes > Utility Classes > DateTime Class > fromOrdinalDate (Class Method); Builtin Classes > Utility Classes > DateTime Class > fromStandardDate (Class Method); Builtin Classes > Utility Classes > DateTime Class > fromTicks (Class Method); Builtin Classes > Utility Classes > DateTime Class > fromUsaDate (Class Method); Builtin Classes > Utility Classes > DateTime Class > fromUtcIsoDate (Class Method); Builtin Classes > Utility Classes > DateTime Class > fromWeekNumberDate (Class Method); Builtin Classes > Utility Classes > DateTime Class > maxDate (Class Method); Builtin Classes > Utility Classes > DateTime Class > minDate (Class Method); Builtin Classes > Utility Classes > DateTime Class > today (Class Method); Builtin Classes > Utility Classes > DateTime Class > Arithmetic Methods; Builtin Classes > Utility Classes > DateTime Class > addDays; Builtin Classes > Utility Classes > DateTime Class > addHours; Builtin Classes > Utility Classes > DateTime Class > addMicroseconds; Builtin Classes > Utility Classes > DateTime Class > addMinutes; Builtin Classes > Utility Classes > DateTime Class > addSeconds; Builtin Classes > Utility Classes > DateTime Class > addWeeks; Builtin Classes > Utility Classes > DateTime Class > addYears; Builtin Classes > Utility Classes > DateTime Class > baseDate; Builtin Classes > Utility Classes > DateTime Class > civilTime; Builtin Classes > Utility Classes > DateTime Class > compareTo; Builtin Classes > Utility Classes > DateTime Class > date; Builtin Classes > Utility Classes > DateTime Class > day; Builtin Classes > Utility Classes > DateTime Class > dayMicroseconds; Builtin Classes > Utility Classes > DateTime Class > dayMinutes; Builtin Classes > Utility Classes > DateTime Class > dayName; Builtin Classes > Utility Classes > DateTime Class > daySeconds; Builtin Classes > Utility Classes > DateTime Class > daysInMonth; Builtin Classes > Utility Classes > DateTime Class > daysInYear; Builtin Classes > Utility Classes > DateTime Class > elapsed; Builtin Classes > Utility Classes > DateTime Class > europeanDate; Builtin Classes > Utility Classes > DateTime Class > fullDate; Builtin Classes > Utility Classes > DateTime Class > hashCode; Builtin Classes > Utility Classes > DateTime Class > hours; Builtin Classes > Utility Classes > DateTime Class > isLeapYear; Builtin Classes > Utility Classes > DateTime Class > isoDate; Builtin Classes > Utility Classes > DateTime Class > languageDate; Builtin Classes > Utility Classes > DateTime Class > longTime; Builtin Classes > Utility Classes > DateTime Class > makeString; Builtin Classes > Utility Classes > DateTime Class > microseconds; Builtin Classes > Utility Classes > DateTime Class > minutes; Builtin Classes > Utility Classes > DateTime Class > month; Builtin Classes > Utility Classes > DateTime Class > monthName; Builtin Classes > Utility Classes > DateTime Class > normalDate; Builtin Classes > Utility Classes > DateTime Class > normalTime; Builtin Classes > Utility Classes > DateTime Class > offset; Builtin Classes > Utility Classes > DateTime Class > orderedDate; Builtin Classes > Utility Classes > DateTime Class > ordinalDate; Builtin Classes > Utility Classes > DateTime Class > seconds; Builtin Classes > Utility Classes > DateTime Class > standardDate; Builtin Classes > Utility Classes > DateTime Class > string; Builtin Classes > Utility Classes > DateTime Class > ticks; Builtin Classes > Utility Classes > DateTime Class > timeOfDay; Builtin Classes > Utility Classes > DateTime Class > toLocalTime; Builtin Classes > Utility Classes > DateTime Class > toTimezone; Builtin Classes > Utility Classes > DateTime Class > toUtcTime; Builtin Classes > Utility Classes > DateTime Class > usaDate; Builtin Classes > Utility Classes > DateTime Class > utcDate; Builtin Classes > Utility Classes > DateTime Class > utcIsoDate; Builtin Classes > Utility Classes > DateTime Class > weekDay; Builtin Classes > Utility Classes > DateTime Class > weekNumber; Builtin Classes > Utility Classes > DateTime Class > weekNumberDate; Builtin Classes > Utility Classes > DateTime Class > weekNumberYear; Builtin Classes > Utility Classes > DateTime Class > weeksInYear; Builtin Classes > Utility Classes > DateTime Class > year; Builtin Classes > Utility Classes > DateTime Class > yearDay"
deviations: [109, 126, 127, 128, 129, 130, 131, 132, 163]
---

# DateTime

A `DateTime` is one moment of the Gregorian calendar, from the first
microsecond of 1 January 0001 to the last of 31 December 9999, that carries
the offset of the time zone it is read in. Its methods answer the parts of the
date and the time of day, write the moment in the formats of
[DATE](../builtins/date.md) and [TIME](../builtins/time.md) and in ISO 8601
forms, make the moment that lies a given span away, and convert it to another
zone. The class inherits `Comparable` and `Orderable`, which give it the
comparison operators on top of its own `compareTo`; the span of time between
two moments is a [`TimeSpan`](r:class.timespan.value). The current moment and
the host's time zone are not specified, so the examples build their instances
from fixed values and give them an explicit offset wherever a result depends
on one.

## The class

r[class.datetime.creation]
`DateTime` has the instance methods `+`, `-`, `addDays`, `addHours`,
`addMicroseconds`, `addMinutes`, `addSeconds`, `addWeeks`, `addYears`,
`baseDate`, `civilTime`, `compareTo`, `date`, `day`, `dayMicroseconds`,
`dayMinutes`, `dayName`, `daySeconds`, `daysInMonth`, `daysInYear`, `elapsed`,
`europeanDate`, `fullDate`, `hashCode`, `hours`, `init`, `isLeapYear`,
`isoDate`, `languageDate`, `longTime`, `makeString`, `microseconds`,
`minutes`, `month`, `monthName`, `normalDate`, `normalTime`, `offset`,
`orderedDate`, `ordinalDate`, `seconds`, `standardDate`, `string`, `ticks`,
`timeOfDay`, `toLocalTime`, `toTimezone`, `toUtcTime`, `usaDate`, `utcDate`,
`utcIsoDate`, `weekDay`, `weekNumber`, `weekNumberDate`, `weekNumberYear`,
`weeksInYear`, `year` and `yearDay`, and the class methods `fromBaseDate`,
`fromCivilTime`, `fromEuropeanDate`, `fromIsoDate`, `fromLongTime`,
`fromNormalDate`, `fromNormalTime`, `fromOrderedDate`, `fromOrdinalDate`,
`fromStandardDate`, `fromTicks`, `fromUsaDate`, `fromUtcIsoDate`,
`fromWeekNumberDate`, `maxDate`, `minDate` and `today`, besides the `new` that
every class has. It may have private methods besides these; they are not part
of this specification. Its superclasses are `Object`, `Comparable` and
`Orderable`, in that order. The comparison operators of an instance are those
of `Orderable`, which send `compareTo` to the receiver ([the comparison
rule](r:class.datetime.compare)). An instance has no `unknown` method, so a
message that [the method search](r:class.methods.unknown) does not resolve
raises [NOMETHOD](r:cond.nomethod.raise), as the error rule of [the
chapter](r:class.index.errors) says; it is not a string either, so a string
method such as `length` raises `SYNTAX 97.1`.

```rexx
do c over .datetime~superclasses
  say c~id
end
names = '+ - addDays addHours addMicroseconds addMinutes addSeconds addWeeks',
  'addYears baseDate civilTime compareTo date day dayMicroseconds dayMinutes',
  'dayName daySeconds daysInMonth daysInYear elapsed europeanDate fullDate',
  'hashCode hours init isLeapYear isoDate languageDate longTime makeString',
  'microseconds minutes month monthName normalDate normalTime offset',
  'orderedDate ordinalDate seconds standardDate string ticks timeOfDay',
  'toLocalTime toTimezone toUtcTime usaDate utcDate utcIsoDate weekDay',
  'weekNumber weekNumberDate weekNumberYear weeksInYear year yearDay'
found = 0
do n over names~makearray(' ')
  found += .datetime~method(n)~isA(.method)
end
say found 'of' names~words
cnames = 'fromBaseDate fromCivilTime fromEuropeanDate fromIsoDate',
  'fromLongTime fromNormalDate fromNormalTime fromOrderedDate',
  'fromOrdinalDate fromStandardDate fromTicks fromUsaDate fromUtcIsoDate',
  'fromWeekNumberDate maxDate minDate today new'
found = 0
do n over cnames~makearray(' ')
  found += .datetime~hasMethod(n)
end
say found 'of' cnames~words
say .datetime~new~hasMethod('FROMTICKS') .datetime~new~hasMethod('UNKNOWN')
signal on syntax name bad
say .datetime~new~length
bad: say condition('o')~code
```

```output
Object
Comparable
Orderable
58 of 58
18 of 18
0 0
97.1
```

r[class.datetime.value]
An instance holds a date and a time of day as the fields year (1 to 9999),
month, day, hours, minutes, seconds and microseconds (0 to 999999), and an
[offset](r:class.datetime.offset) from UTC. The fields are the time as the
people in that zone read it. No method changes an instance: every method that
answers a different moment answers a new instance. Leap seconds do not exist, so
a seconds value is at most 59, and a day has exactly 86400 seconds. The string
value of an instance is its `isoDate` ([the string
rule](r:class.datetime.string)); an instance written by `say` or joined to a
string by `||` is that text, as [the string-value rule](r:class.index.string)
says. An instance on which the `init` of `DateTime` never ran, because the
`init` of a subclass does not forward to it, has the value of
[`minDate`](r:class.datetime.limits) ([Deviation
131](r:class.datetime.deviation.131)).

```rexx
say .datetime~minDate .datetime~maxDate
d = .datetime~new(2016, 12, 31, 23, 59, 59, 999999, 0)
say d
say d~addMicroseconds(1)
say 'at' d || '!'
signal on syntax name bad
say .datetime~new(2016, 12, 31, 23, 59, 60, 0, 0)
bad: say condition('o')~code
```

```output
0001-01-01T00:00:00.000000 9999-12-31T23:59:59.999999
2016-12-31T23:59:59.999999
2017-01-01T00:00:00.000000
at 2016-12-31T23:59:59.999999!
88.907
```

## Offsets

r[class.datetime.offset]
The offset of an instance is the distance of its zone east of UTC, in whole
minutes from -900 to 900. The method `offset` answers it as a
[`TimeSpan`](r:class.timespan.value), so an instance in a zone 5 hours and 30
minutes east of UTC answers `05:30:00.000000` and one in a zone 90 minutes west
answers `-01:30:00.000000`. Every place that takes an offset takes the same two
forms: a number of whole minutes in that range, or a `TimeSpan` that is a whole
number of minutes in that range. The argument is sent `isA` with the class
`TimeSpan`; only a `TimeSpan` is then sent `totalMicroseconds` and is converted
to minutes, and any other value is read as a number ([the argument
rule](r:class.index.string)) and sent the messages that [the sends
rule](r:class.datetime.sends.new) lists. A value outside the range, a number
that is not whole as [the form rule](r:conv.whole.form) and [the digits
rule](r:conv.whole.digits) read it ([Deviation
130](r:class.datetime.deviation.130)), such as 1.5, and a span that is not whole
minutes raise `SYNTAX 88.907` for `new` and, with a number or a span, for
`toTimezone`, and `SYNTAX 88.918` for `today` and for the `from` methods that
read an offset, which are all but `fromWeekNumberDate` (it [ignores the
offset](r:class.datetime.fromweek)) and `fromUtcIsoDate` (it has none); a value
that is no number at all gets the same code from `new`, `today` and those `from`
methods, and what `toTimezone` raises for it is in [the zone
rule](r:class.datetime.zone).

```rexx
d = .datetime~new(2017, 6, 30, 5, 6, 7, 8, 330)
say d~offset d~addDays(1)~offset d~toTimezone(-90)~offset
say .datetime~new(2017, 6, 30, 0, 0, 0, 0, .timespan~fromHours(2))~offset
say .datetime~new(2017, 6, 30, 0, 0, 0, 0, 900)~offset,
    .datetime~new(2017, 6, 30, 0, 0, 0, 0, -900)~offset
call try '.datetime~new(2017, 6, 30, 0, 0, 0, 0, 901)'
call try '.datetime~new(2017, 6, 30, 0, 0, 0, 0, 1.5)'
call try '.datetime~new(2017, 6, 30, 0, 0, 0, 0, .timespan~fromSeconds(90))'
call try 'd~toTimezone(-901)'
call try 'd~toTimezone(.timespan~fromSeconds(90))'
call try '.datetime~today(901)'
call try '.datetime~fromBaseDate(5, 901)'
call try '.datetime~fromTicks(5, 1.5)'
call try '.datetime~fromTicks(5, "x")'
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
05:30:00.000000 05:30:00.000000 -01:30:00.000000
02:00:00.000000
15:00:00.000000 -15:00:00.000000
.datetime~new(2017, 6, 30, 0, 0, 0, 0, 901) => 88.907
.datetime~new(2017, 6, 30, 0, 0, 0, 0, 1.5) => 88.907
.datetime~new(2017, 6, 30, 0, 0, 0, 0, .timespan~fromSeconds(90)) => 88.907
d~toTimezone(-901) => 88.907
d~toTimezone(.timespan~fromSeconds(90)) => 88.907
.datetime~today(901) => 88.918
.datetime~fromBaseDate(5, 901) => 88.918
.datetime~fromTicks(5, 1.5) => 88.918
.datetime~fromTicks(5, "x") => 88.918
```

r[class.datetime.offset.default]
An instance made without an offset gets the offset of the host's zone as it is
when the instance is made: the number of microseconds that
[`TIME('O')`](r:bif.time.zone) answers, as a span. It is one offset for every
date: an instance for a date of summer and one for a date of winter get the
same offset, the one the host has at the moment of the call. Which zone the
host is in is not specified, so the examples compare with the offset of
another instance or with `TIME('O')`. The class methods `new`, `today` and
every `from` method take it, except `fromUtcIsoDate`, which takes the offset
that its string gives, and `minDate` and `maxDate`, whose offset is 0. The
methods that answer a moment of a new zone, `toLocalTime` and `toUtcTime`, are
described with [the zone rule](r:class.datetime.zone). When the host's offset
is not a whole number of minutes from -900 to 900, every method that takes the
default offset fails: `new` and `toLocalTime` raise `SYNTAX 88.907`, and
`today` and every `from` method other than `fromUtcIsoDate` raise
`SYNTAX 88.918`.

```rexx
h = .datetime~new(2017, 1, 5)~offset
say h~totalMicroseconds == time('O')
say (.datetime~new(2017, 7, 5)~offset == h),
    (.datetime~new(1, 1, 1)~offset == h)
say (.datetime~today~offset == h) (.datetime~fromTicks(0)~offset == h)
say (.datetime~new~offset == h) (.datetime~fromBaseDate(5)~offset == h)
say .datetime~fromUtcIsoDate('2017-01-05T01:02:03+00:00')~offset
say .datetime~minDate~offset .datetime~maxDate~offset
say (abs(h~totalMinutes) <= 900)
```

```output
1
1 1
1 1
1 1
00:00:00.000000
00:00:00.000000 00:00:00.000000
1
```

## Making an instance

r[class.datetime.new]
`new` makes an instance and sends `init` to it with the arguments it received
([the new rule](r:class.index.new)); the `init` of `DateTime` takes up to
eight arguments, and the number of them that are given, counted as [the count
rule](r:class.index.args.count) says, picks the form. With no argument the
moment is that of the host's clock, whose value is not specified. With one
argument or two, the first is the number of microseconds from 00:00:00.000000
on 1 January 0001 to the moment, a whole number from 0 to 315537897599999999,
and the second, if present, is the offset. With three arguments or four they
are year, month and day, the time of day is 00:00:00.000000, and the fourth,
if present, is the offset: the four-argument form is not the three-argument
form with an hour. With from five to eight arguments they are year, month,
day, hours and minutes, then seconds, microseconds and the offset, where an
omitted one of these three is 0 (the offset: [the
default](r:class.datetime.offset.default)). Without an offset argument the
instance has the default offset. An argument that the form needs and that is
omitted raises `SYNTAX 93.903`; more than eight arguments raise
`SYNTAX 93.902`.

```rexx
say .datetime~new(2017, 6, 30, 5, 6, 7, 8, 0)
say .datetime~new(2017, 6, 30, 5, 6, 0)
say .datetime~new(2017, 6, 30, 5, 6, , , 60)~utcIsoDate
say .datetime~new(2017, 6, 30, 0)~utcIsoDate
say .datetime~new(2017, 6, 30, 5)~utcIsoDate
say .datetime~new(63634417730123456, 0)
say .datetime~new(63634417730123456, 120)~utcIsoDate
say .datetime~new(12)~minutes .datetime~new(0)
say (.datetime~new~year >= 2026) (.datetime~new() < .datetime~maxDate)
call try '.datetime~new(2017, , 30)'
call try '.datetime~new(, 2)'
call try '.datetime~new(2017, 6, 30, , 2)'
call try '.datetime~new(2017, 6, 30, 5, , 7)'
call try '.datetime~new(1, 2, 3, 4, 5, 6, 7, 8, 9)'
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
2017-06-30T05:06:07.000008
2017-06-30T05:06:00.000000
2017-06-30T05:06:00.000000+01:00
2017-06-30T00:00:00.000000Z
2017-06-30T00:00:00.000000+00:05
2017-06-30T11:08:50.123456
2017-06-30T11:08:50.123456+02:00
0 0001-01-01T00:00:00.000000
1 1
.datetime~new(2017, , 30) => 93.903
.datetime~new(, 2) => 93.903
.datetime~new(2017, 6, 30, , 2) => 93.903
.datetime~new(2017, 6, 30, 5, , 7) => 93.903
.datetime~new(1, 2, 3, 4, 5, 6, 7, 8, 9) => 93.902
```

r[class.datetime.new.checks]
The values are checked in this order, and the first that fails raises its error.
In the one-argument and two-argument form the number of microseconds must be a
whole number in its range, or `SYNTAX 40.19` is raised, and then the offset is
checked as [the offset rule](r:class.datetime.offset) says. Each whole number,
here and below, is read as a whole-number argument of a builtin is, by [the form
rule](r:conv.whole.form) and [the digits rule](r:conv.whole.digits), with the
codes of this rule in place of SYNTAX 40.12 ([Deviation
130](r:class.datetime.deviation.130)). In the other forms year, month and day
are whole numbers, in the ranges 1 to 9999, 1 to 12 and 1 to 31; one outside its
range, one that is no number and one that is not whole, such as 1.5, raise
`SYNTAX 88.907`. A day that the month does not have, such as 30 February, then
raises `SYNTAX 40.19`. Hours (0 to 23), minutes (0 to 59), seconds (0 to 59) and
microseconds (0 to 999999) are checked last, each a whole number in its range,
and then the offset, as [the offset rule](r:class.datetime.offset) says; a value
outside its range, one that is no number and one that is not whole raise `SYNTAX
88.907`. A day that does not exist is thus reported before an hour that is out
of range, and a day of 32 before one that does not exist.

```rexx
call try '.datetime~new(2017, 2, 30, 25, 0, 0, 0)'
call try '.datetime~new(2017, 2, 32)'
call try '.datetime~new(2017, 2, 30, 0, 0, 0, 0, 901)'
call try '.datetime~new(2017, 2, 29)'
call try '.datetime~new(2016, 2, 29)'
call try '.datetime~new(2017, 6, 30, 24, 0, 0, 0)'
call try '.datetime~new(2017, 6, 30, 0, 60, 0, 0)'
call try '.datetime~new(2017, 6, 30, 0, 0, 60, 0)'
call try '.datetime~new(2017, 6, 30, 0, 0, 0, 1000000)'
call try '.datetime~new(2017, 6, 30, -1, 0, 0, 0)'
call try '.datetime~new(10000, 1, 1)'
call try '.datetime~new(0, 1, 1)'
call try '.datetime~new(2017, "x", 1)'
call try '.datetime~new(2017, 6, 30, 1.5, 0, 0, 0)'
call try '.datetime~new(-1)'
call try '.datetime~new(315537897600000000)'
call try '.datetime~new(1.5)'
call try '.datetime~new("x")'
call try '.datetime~new(-5, 901)'
call try '.datetime~new(5, 901)'
call try '.datetime~new(9999, 12, 31, 23, 59, 59, 999999)'
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
.datetime~new(2017, 2, 30, 25, 0, 0, 0) => 40.19
.datetime~new(2017, 2, 32) => 88.907
.datetime~new(2017, 2, 30, 0, 0, 0, 0, 901) => 40.19
.datetime~new(2017, 2, 29) => 40.19
.datetime~new(2016, 2, 29) => 2016-02-29T00:00:00.000000
.datetime~new(2017, 6, 30, 24, 0, 0, 0) => 88.907
.datetime~new(2017, 6, 30, 0, 60, 0, 0) => 88.907
.datetime~new(2017, 6, 30, 0, 0, 60, 0) => 88.907
.datetime~new(2017, 6, 30, 0, 0, 0, 1000000) => 88.907
.datetime~new(2017, 6, 30, -1, 0, 0, 0) => 88.907
.datetime~new(10000, 1, 1) => 88.907
.datetime~new(0, 1, 1) => 88.907
.datetime~new(2017, "x", 1) => 88.907
.datetime~new(2017, 6, 30, 1.5, 0, 0, 0) => 88.907
.datetime~new(-1) => 40.19
.datetime~new(315537897600000000) => 40.19
.datetime~new(1.5) => 40.19
.datetime~new("x") => 40.19
.datetime~new(-5, 901) => 40.19
.datetime~new(5, 901) => 88.907
.datetime~new(9999, 12, 31, 23, 59, 59, 999999) => 9999-12-31T23:59:59.999999
```

r[class.datetime.new.init-again]
`init` is an ordinary message that can be sent again ([the init
rule](r:class.index.new.init-again)): it gives the instance the moment that
`new` with the same arguments would make, with the same forms, the same checks
in the same order and the same codes, and answers nothing. The offset is the
one the arguments give, or the default offset when they give none, whatever
the instance had. What an instance holds after an `init` that raised an error
is not specified.

```rexx
d = .datetime~new(2017, 6, 30, 5, 6, 7, 8, 60)
d~init(2018, 1, 2)
say d (d~offset == .datetime~new~offset)
d~init(2019, 3, 4, 5, 6, 7, 8, 120)
say d d~offset
d~init(5, 30)
say d d~offset
d~init(2020, 2, 29, 0, 0, , , .timespan~fromHours(-1))
say d d~offset
call try 'd~init(1, 2, 3, 4, 5, 6, 7, 8, 9)'
call try 'd~init(2017, , 30)'
call try 'd~init(2017, 13, 1)'
call try 'd~init(2017, 2, 30)'
call try 'd~init(-1)'
call try 'd~init(2017, 6, 30, 0, 0, 0, 0, 901)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
2018-01-02T00:00:00.000000 1
2019-03-04T05:06:07.000008 02:00:00.000000
0001-01-01T00:00:00.000005 00:30:00.000000
2020-02-29T00:00:00.000000 -01:00:00.000000
d~init(1, 2, 3, 4, 5, 6, 7, 8, 9) => 93.902
d~init(2017, , 30) => 93.903
d~init(2017, 13, 1) => 88.907
d~init(2017, 2, 30) => 40.19
d~init(-1) => 40.19
d~init(2017, 6, 30, 0, 0, 0, 0, 901) => 88.907
```

## Class methods

r[class.datetime.limits]
`minDate` answers the first moment, 1 January 0001 at 00:00:00.000000, and
`maxDate` the last, 31 December 9999 at 23:59:59.999999. Both have the offset
0, so their `fullDate` is 0 and 315537897599999999. Each takes no argument;
one raises `SYNTAX 93.902`. A subclass receiver gets an instance of itself, as
[the sends rule](r:class.datetime.sends.new) says.

```rexx
say .datetime~minDate .datetime~minDate~utcIsoDate .datetime~minDate~fullDate
say .datetime~maxDate .datetime~maxDate~utcIsoDate .datetime~maxDate~fullDate
say .datetime~minDate < .datetime~maxDate
call try '.datetime~minDate(1)'
call try '.datetime~maxDate(1)'
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
0001-01-01T00:00:00.000000 0001-01-01T00:00:00.000000Z 0
9999-12-31T23:59:59.999999 9999-12-31T23:59:59.999999Z 315537897599999999
1
.datetime~minDate(1) => 93.902
.datetime~maxDate(1) => 93.902
```

r[class.datetime.today]
`today` answers the current day of the host, at 00:00:00.000000, with the
default offset or the one its optional argument gives ([the offset
rule](r:class.datetime.offset)). The day is the current one and is not
specified, so the example checks the time of day and the offset only. It takes
at most one argument: a second raises `SYNTAX 93.902`, whatever the first is.

```rexx
t = .datetime~today
say t~normalTime t~microseconds (t~offset == .datetime~new~offset)
say .datetime~today(90)~offset .datetime~today(.timespan~fromHours(-3))~offset
signal on syntax name bad
say .datetime~today(901)
bad: say condition('o')~code
signal on syntax name bad2
say .datetime~today(901, 1)
bad2: say condition('o')~code
```

```output
00:00:00 0 1
01:30:00.000000 -03:00:00.000000
88.918
93.902
```

r[class.datetime.fromdate]
`fromBaseDate`, `fromEuropeanDate`, `fromNormalDate`, `fromOrderedDate`,
`fromStandardDate` and `fromUsaDate` read a date that is written in the form
that the option B ([DATE](r:bif.date.out.counts)) or E, N, O, S or U
([DATE](r:bif.date.out.text)) writes it, and answer that day at 00:00:00.000000.
`fromBaseDate` takes a whole number of days from 0 (1 January 0001) to 3652058
(31 December 9999) as `baseDate` answers it, read by [the form
rule](r:conv.whole.form) and [the digits rule](r:conv.whole.digits) ([Deviation
130](r:class.datetime.deviation.130)), so `5.0` is day 5 and `5.0000001` is
refused. The other five take the forms `dd/mm/yy`, `d mon yyyy` (the day has one
or two digits, the month is the three letters of [its name](r:bif.date.out.text)
with a capital first letter), `yy/mm/dd`, `yyyymmdd` and `mm/dd/yy`. A two-digit
year is the one that [the century rule](r:bif.date.in.century) of `DATE` gives,
so it depends on the current year. Each of the other five takes the field
separator as its optional second argument, in place of the default that [the
output separator](r:bif.date.sep.out) of the option has: the null string or one
character that is no letter and no digit. The offset ([the offset
rule](r:class.datetime.offset)) is the last optional argument, the second one of
`fromBaseDate` and the third of the others. A string that is not of the form, a
month or a day that does not exist, a year 0000, and a separator that does not
fit raise `SYNTAX 88.918`. Without a string the method raises `SYNTAX 93.901`,
and an argument after the offset raises `SYNTAX 93.902`.

```rexx
say .datetime~fromBaseDate(0) .datetime~fromBaseDate(736509)
say .datetime~fromBaseDate(' 12 ') .datetime~fromBaseDate('1e1')
say .datetime~fromEuropeanDate('30/06/17'),
    .datetime~fromOrderedDate('17/06/30')
say .datetime~fromUsaDate('06/30/17') .datetime~fromStandardDate('20170630')
say .datetime~fromNormalDate('30 Jun 2017'),
    .datetime~fromNormalDate('3 Jun 2017')
say .datetime~fromEuropeanDate('30-06-17', '-'),
    .datetime~fromEuropeanDate('300617', '')
say .datetime~fromStandardDate('2017.06.30', '.'),
    .datetime~fromNormalDate('30Jun2017', '')
say .datetime~fromUsaDate('06/30/17', , 90)~utcIsoDate
say .datetime~fromStandardDate('00010101'),
    .datetime~fromStandardDate('99991231')
ok = 1
do n = 0 to 99
  y = .datetime~fromEuropeanDate('01/01/' || right(n, 2, 0))~year
  now = .datetime~new~year
  ok = ok & (y >= now - 50) & (y <= now + 49) &,
      (right(y, 2) == right(n, 2, 0))
end
say ok
call try '.datetime~fromBaseDate(-1)'
call try '.datetime~fromBaseDate(3652059)'
call try '.datetime~fromBaseDate(1.5)'
call try '.datetime~fromBaseDate(5.0000001)'
call try '.datetime~fromEuropeanDate("30/06/17", "")'
call try '.datetime~fromEuropeanDate("30/06/17", "//")'
call try '.datetime~fromEuropeanDate("30x06x17", "x")'
call try '.datetime~fromEuropeanDate("31/06/17")'
call try '.datetime~fromEuropeanDate("29/02/17")'
call try '.datetime~fromEuropeanDate("1/6/17")'
call try '.datetime~fromEuropeanDate("30/06/2017")'
call try '.datetime~fromNormalDate("30 jun 2017")'
call try '.datetime~fromNormalDate("30 June 2017")'
call try '.datetime~fromNormalDate("30 Jun 0000")'
call try '.datetime~fromStandardDate("2017/06/30")'
call try '.datetime~fromBaseDate'
call try '.datetime~fromBaseDate(5, 0, 1)'
call try '.datetime~fromEuropeanDate("30/06/17", "/", 0, 1)'
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
0001-01-01T00:00:00.000000 2017-06-30T00:00:00.000000
0001-01-13T00:00:00.000000 0001-01-11T00:00:00.000000
2017-06-30T00:00:00.000000 2017-06-30T00:00:00.000000
2017-06-30T00:00:00.000000 2017-06-30T00:00:00.000000
2017-06-30T00:00:00.000000 2017-06-03T00:00:00.000000
2017-06-30T00:00:00.000000 2017-06-30T00:00:00.000000
2017-06-30T00:00:00.000000 2017-06-30T00:00:00.000000
2017-06-30T00:00:00.000000+01:30
0001-01-01T00:00:00.000000 9999-12-31T00:00:00.000000
1
.datetime~fromBaseDate(-1) => 88.918
.datetime~fromBaseDate(3652059) => 88.918
.datetime~fromBaseDate(1.5) => 88.918
.datetime~fromBaseDate(5.0000001) => 88.918
.datetime~fromEuropeanDate("30/06/17", "") => 88.918
.datetime~fromEuropeanDate("30/06/17", "//") => 88.918
.datetime~fromEuropeanDate("30x06x17", "x") => 88.918
.datetime~fromEuropeanDate("31/06/17") => 88.918
.datetime~fromEuropeanDate("29/02/17") => 88.918
.datetime~fromEuropeanDate("1/6/17") => 88.918
.datetime~fromEuropeanDate("30/06/2017") => 88.918
.datetime~fromNormalDate("30 jun 2017") => 88.918
.datetime~fromNormalDate("30 June 2017") => 88.918
.datetime~fromNormalDate("30 Jun 0000") => 88.918
.datetime~fromStandardDate("2017/06/30") => 88.918
.datetime~fromBaseDate => 93.901
.datetime~fromBaseDate(5, 0, 1) => 93.902
.datetime~fromEuropeanDate("30/06/17", "/", 0, 1) => 93.902
```

r[class.datetime.fromordinal]
`fromOrdinalDate` reads a year and the number of a day in it, written
`yyyyddd` or `yyyy-ddd`: four digits for the year, from 0001 to 9999, and
three for the day, from 001 to 365 or, in a leap year, 366. It answers that
day at 00:00:00.000000, and its optional second argument is the offset. Any
other string, a day number that the year does not have and a year 0000 raise
`SYNTAX 88.918`. Without the string the method raises `SYNTAX 93.901`, and a
third argument raises `SYNTAX 93.902`.

```rexx
say .datetime~fromOrdinalDate('2020-041') .datetime~fromOrdinalDate('2020041')
say .datetime~fromOrdinalDate('2020-366'),
    .datetime~fromOrdinalDate('2019-365')
say .datetime~fromOrdinalDate('0001-001'),
    .datetime~fromOrdinalDate('9999-365')
say .datetime~fromOrdinalDate('2020-041', 120)~utcIsoDate
call try '.datetime~fromOrdinalDate("2019-366")'
call try '.datetime~fromOrdinalDate("2019-000")'
call try '.datetime~fromOrdinalDate("2019-41")'
call try '.datetime~fromOrdinalDate("0000-001")'
call try '.datetime~fromOrdinalDate("2020/041")'
call try '.datetime~fromOrdinalDate("2020-041", 901)'
call try '.datetime~fromOrdinalDate'
call try '.datetime~fromOrdinalDate("2020-041", 0, 1)'
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
2020-02-10T00:00:00.000000 2020-02-10T00:00:00.000000
2020-12-31T00:00:00.000000 2019-12-31T00:00:00.000000
0001-01-01T00:00:00.000000 9999-12-31T00:00:00.000000
2020-02-10T00:00:00.000000+02:00
.datetime~fromOrdinalDate("2019-366") => 88.918
.datetime~fromOrdinalDate("2019-000") => 88.918
.datetime~fromOrdinalDate("2019-41") => 88.918
.datetime~fromOrdinalDate("0000-001") => 88.918
.datetime~fromOrdinalDate("2020/041") => 88.918
.datetime~fromOrdinalDate("2020-041", 901) => 88.918
.datetime~fromOrdinalDate => 93.901
.datetime~fromOrdinalDate("2020-041", 0, 1) => 93.902
```

r[class.datetime.fromweek]
`fromWeekNumberDate` reads an ISO week date, `yyyyWwwd` or `yyyy-Www-d`: the
year in four digits, a capital `W`, the number of the week in two digits and
the weekday from 1 (Monday) to 7 (Sunday). Week 1 of a year is the week that
holds its 4 January, and a year has 52 or 53 weeks, as
[`weeksInYear`](r:class.datetime.isoweek) says; the first week may begin in
the year before. The method answers that day at 00:00:00.000000. A string that
is not of the form, a week 00 or a week that the year does not have, a weekday
0 or 8, and a date outside the range of the class (the year 0000, or the week
date that would follow 31 December 9999) raise `SYNTAX 88.918`. The method is
given an optional second argument, the offset, but ignores it, so a value that
is no offset raises nothing: the answer always has the default offset ([the
default](r:class.datetime.offset.default)). Without the string the method
raises `SYNTAX 93.901`, and a third argument raises `SYNTAX 93.902`.

```rexx
say .datetime~fromWeekNumberDate('2020-W07-1'),
    .datetime~fromWeekNumberDate('2020W071')
say .datetime~fromWeekNumberDate('2020-W53-7'),
    .datetime~fromWeekNumberDate('2019-W52-7')
say .datetime~fromWeekNumberDate('2018-W01-1'),
    .datetime~fromWeekNumberDate('0001-W01-1')
say .datetime~fromWeekNumberDate('9999-W52-5')
say .datetime~fromWeekNumberDate('2017-W01-4', 120)~offset,
    == .datetime~new~offset
say .datetime~fromWeekNumberDate('2020-W07-1', 'x'),
    .datetime~fromWeekNumberDate('2020-W07-1', 901)
call try '.datetime~fromWeekNumberDate("2019-W53-1")'
call try '.datetime~fromWeekNumberDate("2020-W00-1")'
call try '.datetime~fromWeekNumberDate("2020-W01-0")'
call try '.datetime~fromWeekNumberDate("2020-W01-8")'
call try '.datetime~fromWeekNumberDate("2020-w07-1")'
call try '.datetime~fromWeekNumberDate("2020-W7-1")'
call try '.datetime~fromWeekNumberDate("2020-W07")'
call try '.datetime~fromWeekNumberDate("9999-W52-7")'
call try '.datetime~fromWeekNumberDate("0000-W52-7")'
call try '.datetime~fromWeekNumberDate'
call try '.datetime~fromWeekNumberDate("2020-W07-1", 0, 1)'
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
2020-02-10T00:00:00.000000 2020-02-10T00:00:00.000000
2021-01-03T00:00:00.000000 2019-12-29T00:00:00.000000
2018-01-01T00:00:00.000000 0001-01-01T00:00:00.000000
9999-12-31T00:00:00.000000
1
2020-02-10T00:00:00.000000 2020-02-10T00:00:00.000000
.datetime~fromWeekNumberDate("2019-W53-1") => 88.918
.datetime~fromWeekNumberDate("2020-W00-1") => 88.918
.datetime~fromWeekNumberDate("2020-W01-0") => 88.918
.datetime~fromWeekNumberDate("2020-W01-8") => 88.918
.datetime~fromWeekNumberDate("2020-w07-1") => 88.918
.datetime~fromWeekNumberDate("2020-W7-1") => 88.918
.datetime~fromWeekNumberDate("2020-W07") => 88.918
.datetime~fromWeekNumberDate("9999-W52-7") => 88.918
.datetime~fromWeekNumberDate("0000-W52-7") => 88.918
.datetime~fromWeekNumberDate => 93.901
.datetime~fromWeekNumberDate("2020-W07-1", 0, 1) => 93.902
```

r[class.datetime.fromdate.argument]
`fromOrdinalDate` and `fromWeekNumberDate` do not read their first argument as
a string at once. After the argument count is checked, each sends the argument
`contains` with the one argument `-`, once and before anything else, and uses
the answer to pick the form: true for the form with hyphens, false for the one
without. The answer must be exactly `0` or `1`; any other answer, `.nil`
included, raises `SYNTAX 34.1`, and an argument that does not understand
`contains` raises `SYNTAX 97.1` (an `Array`, a `List`, a `Set` or `.nil`; a
`Directory` answers `.nil` and so raises 34.1). Then the string value of the
argument is read, and a form that the answer did not pick, or a string that is
not a date, raises `SYNTAX 88.918`. A string that is a number or a `String`
subclass instance is sent `contains` like any other value, so a subclass that
overrides it decides the form: an instance of the string `2020041` whose
`contains` answers true raises 88.918, though the string is a date.

```rexx
say .datetime~fromOrdinalDate(.cs~new('2020-041'))
say .datetime~fromOrdinalDate(.cs~new('2020041'))
call try '.datetime~fromOrdinalDate(.array~of("2020-041"))'
call try '.datetime~fromWeekNumberDate(.nil)'
call try '.datetime~fromOrdinalDate(.directory~new)'
call try '.datetime~fromOrdinalDate(.spy~new(1))'
call try '.datetime~fromWeekNumberDate(.spy~new("maybe"))'
call try '.datetime~fromOrdinalDate(.no~new("2020-041"))'
call try '.datetime~fromWeekNumberDate(.yes~new("2020W071"))'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class spy
::method init
  expose answer
  use arg answer
::method unknown
  expose answer
  use arg name, args
  say name args~toString
  return answer
::class cs subclass string
::method contains
  say 'contains' arg(1)
  return self~string~contains(arg(1))
::class no subclass string
::method contains
  say 'contains' arg(1)
  return 0
::class yes subclass string
::method contains
  say 'contains' arg(1)
  return 1
```

```output
contains -
2020-02-10T00:00:00.000000
contains -
2020-02-10T00:00:00.000000
.datetime~fromOrdinalDate(.array~of("2020-041")) => 97.1
.datetime~fromWeekNumberDate(.nil) => 97.1
.datetime~fromOrdinalDate(.directory~new) => 34.1
CONTAINS -
.datetime~fromOrdinalDate(.spy~new(1)) => 88.918
CONTAINS -
.datetime~fromWeekNumberDate(.spy~new("maybe")) => 34.1
contains -
.datetime~fromOrdinalDate(.no~new("2020-041")) => 88.918
contains -
.datetime~fromWeekNumberDate(.yes~new("2020W071")) => 88.918
```

r[class.datetime.fromtime]
`fromNormalTime`, `fromCivilTime` and `fromLongTime` read a time of day that
is written in the form of the option N, C or L of
[TIME](r:bif.time.out.clock): `hh:mm:ss`, `h:mmam` or `h:mmpm`, and
`hh:mm:ss.f` where `f` is one to six digits, taken as the leading digits of
the microseconds. The hour of a civil time has one or two digits and is from 1
to 12; `am` and `pm` are lower case. The answer is that time on 1 January
0001, and the optional second argument is the offset. A string that is not of
the form, an hour past 23 (past 12 for the civil form, and 0 is no civil
hour), and minutes or seconds past 59 raise `SYNTAX 88.918`. Without the
string each method raises `SYNTAX 93.901`, and a third argument raises
`SYNTAX 93.902`.

```rexx
say .datetime~fromNormalTime('13:45:09') .datetime~fromNormalTime('00:00:00')
say .datetime~fromCivilTime('1:45pm') .datetime~fromCivilTime('01:45pm')
say .datetime~fromCivilTime('12:00am') .datetime~fromCivilTime('12:59pm')
say .datetime~fromLongTime('13:45:09.123456'),
    .datetime~fromLongTime('13:45:09.1')
say .datetime~fromLongTime('13:45:09.0001'),
    .datetime~fromNormalTime('23:59:59', 60)~utcIsoDate
call try '.datetime~fromNormalTime("24:00:00")'
call try '.datetime~fromNormalTime("12:60:00")'
call try '.datetime~fromNormalTime("1:00:00")'
call try '.datetime~fromNormalTime("13:45")'
call try '.datetime~fromCivilTime("0:30am")'
call try '.datetime~fromCivilTime("13:30pm")'
call try '.datetime~fromCivilTime("1:45PM")'
call try '.datetime~fromCivilTime("1:45 pm")'
call try '.datetime~fromLongTime("13:45:09")'
call try '.datetime~fromLongTime("13:45:09.1234567")'
call try '.datetime~fromNormalTime'
call try '.datetime~fromCivilTime'
call try '.datetime~fromLongTime'
call try '.datetime~fromNormalTime("13:45:09", 0, 1)'
call try '.datetime~fromCivilTime("1:45pm", 0, 1)'
call try '.datetime~fromLongTime("13:45:09.1", 0, 1)'
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
0001-01-01T13:45:09.000000 0001-01-01T00:00:00.000000
0001-01-01T13:45:00.000000 0001-01-01T13:45:00.000000
0001-01-01T00:00:00.000000 0001-01-01T12:59:00.000000
0001-01-01T13:45:09.123456 0001-01-01T13:45:09.100000
0001-01-01T13:45:09.000100 0001-01-01T23:59:59.000000+01:00
.datetime~fromNormalTime("24:00:00") => 88.918
.datetime~fromNormalTime("12:60:00") => 88.918
.datetime~fromNormalTime("1:00:00") => 88.918
.datetime~fromNormalTime("13:45") => 88.918
.datetime~fromCivilTime("0:30am") => 88.918
.datetime~fromCivilTime("13:30pm") => 88.918
.datetime~fromCivilTime("1:45PM") => 88.918
.datetime~fromCivilTime("1:45 pm") => 88.918
.datetime~fromLongTime("13:45:09") => 88.918
.datetime~fromLongTime("13:45:09.1234567") => 88.918
.datetime~fromNormalTime => 93.901
.datetime~fromCivilTime => 93.901
.datetime~fromLongTime => 93.901
.datetime~fromNormalTime("13:45:09", 0, 1) => 93.902
.datetime~fromCivilTime("1:45pm", 0, 1) => 93.902
.datetime~fromLongTime("13:45:09.1", 0, 1) => 93.902
```

r[class.datetime.fromticks]
`fromTicks` reads a whole number of seconds from 00:00:00 on 1 January 1970,
negative before it, as `ticks` answers it, and answers that moment with no
microseconds. The largest value is 253402300799 and the smallest is
-62135596800, the first moment of the class; up to 49 seconds below the
smallest are read as the first moment. Beyond those limits the method raises
`SYNTAX 88.918`, as it does for a value that is not a whole number. The
optional second argument is the offset. Without the value the method raises
`SYNTAX 93.901`, and a third argument raises `SYNTAX 93.902`.

```rexx
say .datetime~fromTicks(0) .datetime~fromTicks(1498820930)
say .datetime~fromTicks(-1)
say .datetime~fromTicks(253402300799) .datetime~fromTicks(-62135596800)
say .datetime~fromTicks(-62135596849)
say .datetime~fromTicks(5, 0)~utcIsoDate .datetime~fromTicks(5, 90)~utcIsoDate
call try '.datetime~fromTicks(253402300800)'
call try '.datetime~fromTicks(-62135596850)'
call try '.datetime~fromTicks(1.5)'
call try '.datetime~fromTicks("x")'
call try '.datetime~fromTicks'
call try '.datetime~fromTicks(5, 0, 1)'
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
1970-01-01T00:00:00.000000 2017-06-30T11:08:50.000000
1969-12-31T23:59:59.000000
9999-12-31T23:59:59.000000 0001-01-01T00:00:00.000000
0001-01-01T00:00:00.000000
1970-01-01T00:00:05.000000Z 1970-01-01T00:00:05.000000+01:30
.datetime~fromTicks(253402300800) => 88.918
.datetime~fromTicks(-62135596850) => 88.918
.datetime~fromTicks(1.5) => 88.918
.datetime~fromTicks("x") => 88.918
.datetime~fromTicks => 93.901
.datetime~fromTicks(5, 0, 1) => 93.902
```

r[class.datetime.fromiso]
`fromIsoDate` reads the string that `isoDate` writes, `yyyy-mm-ddThh:mm:ss.f`,
and answers that moment: the fields are the string's, and the offset is the
optional second argument or the default. The fraction `f` has one to six
digits, the leading digits of the microseconds, and is required; the `T` is a
capital letter. A string that is not of the form, or names an hour past 23, a
minute or second past 59 or a day that does not exist, raises `SYNTAX 88.918`.
Without the string the method raises `SYNTAX 93.901`, and a third argument
raises `SYNTAX 93.902`.

```rexx
say .datetime~fromIsoDate('2017-06-30T11:08:50.123456')
say .datetime~fromIsoDate('2017-06-30T11:08:50.12345')
say .datetime~fromIsoDate('2017-06-30T11:08:50.1', 30)~utcIsoDate
say .datetime~fromIsoDate('0001-01-01T00:00:00.000000')
say .datetime~fromIsoDate('9999-12-31T23:59:59.999999')
call try '.datetime~fromIsoDate("2017-06-30T11:08:50")'
call try '.datetime~fromIsoDate("2017-06-30")'
call try '.datetime~fromIsoDate("2017-06-30 11:08:50.123456")'
call try '.datetime~fromIsoDate("2017-06-30t11:08:50.123456")'
call try '.datetime~fromIsoDate("2017-06-30T11:08:50.123456Z")'
call try '.datetime~fromIsoDate("2017-06-30T11:08:50.1234567")'
call try '.datetime~fromIsoDate("2017-06-30T24:00:00.000000")'
call try '.datetime~fromIsoDate("2017-02-30T00:00:00.000000")'
call try '.datetime~fromIsoDate("20170630T110850.123456")'
call try '.datetime~fromIsoDate'
call try '.datetime~fromIsoDate("2017-06-30T11:08:50.1", 0, 1)'
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
2017-06-30T11:08:50.123456
2017-06-30T11:08:50.123450
2017-06-30T11:08:50.100000+00:30
0001-01-01T00:00:00.000000
9999-12-31T23:59:59.999999
.datetime~fromIsoDate("2017-06-30T11:08:50") => 88.918
.datetime~fromIsoDate("2017-06-30") => 88.918
.datetime~fromIsoDate("2017-06-30 11:08:50.123456") => 88.918
.datetime~fromIsoDate("2017-06-30t11:08:50.123456") => 88.918
.datetime~fromIsoDate("2017-06-30T11:08:50.123456Z") => 88.918
.datetime~fromIsoDate("2017-06-30T11:08:50.1234567") => 88.918
.datetime~fromIsoDate("2017-06-30T24:00:00.000000") => 88.918
.datetime~fromIsoDate("2017-02-30T00:00:00.000000") => 88.918
.datetime~fromIsoDate("20170630T110850.123456") => 88.918
.datetime~fromIsoDate => 93.901
.datetime~fromIsoDate("2017-06-30T11:08:50.1", 0, 1) => 93.902
```

r[class.datetime.fromutciso]
`fromUtcIsoDate` reads the string that `utcIsoDate` writes: the date and time of
`fromIsoDate`, with a fraction of one to six digits that may also be left out
together with its point, followed by the offset of the string, either `Z`
(offset 0) or a sign and `hh:mm`, `+02:30` and `-03:30` for instance, up to
15:00 in either direction. The answer has the fields of the string and that
offset; `+00:00` and `-00:00` are the offset 0. The method takes no second
argument (`SYNTAX 93.902`) and, without the string, raises `SYNTAX 93.901`. The
string is read strictly ([Deviation 132](r:class.datetime.deviation.132)): it
has no blank, the hour and the minutes of the offset have two digits each with
minutes up to 59, and nothing follows the `Z` or the offset. A string that is
not of this form raises `SYNTAX 88.918`: one that ends with no `Z` or offset,
that ends in a lower-case `z`, in an offset in the forms `+0200`, `+02`, `+02:`
or `+:30` or in an offset past 15:00, or that ends in a letter other than `Z`
after the offset, and the other spellings that the deviation rule shows.

```rexx
say .datetime~fromUtcIsoDate('2017-06-30T11:08:50.123456+02:00')~utcIsoDate
say .datetime~fromUtcIsoDate('2017-06-30T11:08:50.123456-03:30')~utcIsoDate
say .datetime~fromUtcIsoDate('2017-06-30T11:08:50.123456Z')~utcIsoDate
say .datetime~fromUtcIsoDate('2017-06-30T11:08:50.1-00:00')~utcIsoDate
say .datetime~fromUtcIsoDate('2017-06-30T11:08:50.123456+15:00')~offset
say .datetime~fromUtcIsoDate('2017-06-30T11:08:50+02:00')~utcIsoDate
call try '.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456")'
call try '.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456z")'
call try '.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+0200")'
call try '.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+02")'
call try '.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+15:01")'
call try '.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+02:")'
call try '.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+:30")'
call try '.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+16:00")'
call try '.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456Zx")'
call try '.datetime~fromUtcIsoDate'
call try '.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+02:00", 5)'
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
2017-06-30T11:08:50.123456+02:00
2017-06-30T11:08:50.123456-03:30
2017-06-30T11:08:50.123456Z
2017-06-30T11:08:50.100000Z
15:00:00.000000
2017-06-30T11:08:50.000000+02:00
.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456") => 88.918
.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456z") => 88.918
.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+0200") => 88.918
.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+02") => 88.918
.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+15:01") => 88.918
.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+02:") => 88.918
.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+:30") => 88.918
.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+16:00") => 88.918
.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456Zx") => 88.918
.datetime~fromUtcIsoDate => 93.901
.datetime~fromUtcIsoDate("2017-06-30T11:08:50.123456+02:00", 5) => 93.902
```

## Reading the fields

r[class.datetime.fields]
`year` answers the year in four digits, with leading zeros for a year before
1000, so the year of `minDate` is `0001`. `month`, `day`, `hours`, `minutes`,
`seconds` and `microseconds` answer the field of that name as a whole number
without leading zeros, so the microseconds of `10:00:00.012345` are 12345 and
the month of 5 March is 3. `dayMinutes`, `daySeconds` and
`dayMicroseconds` answer how many whole minutes, whole seconds and
microseconds the time of day is past 00:00:00.000000. The offset plays no part
in any of them. Each method takes no argument: one raises `SYNTAX 93.902`.

```rexx
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 120)
say d~year d~month d~day d~hours d~minutes d~seconds d~microseconds
say d~dayMinutes d~daySeconds d~dayMicroseconds
say .datetime~minDate~dayMicroseconds .datetime~maxDate~dayMicroseconds
say d~toUtcTime~hours d~toTimezone(-600)~day
say .datetime~minDate~year .datetime~new(5, 3, 5)~year .datetime~new(5, 3, 5)~month
signal on syntax name bad
say d~year(1)
bad: say condition('o')~code
```

```output
2016 2 29 9 8 7 12345
548 32887 32887012345
0 86399999999
7 28
0001 0005 3
93.902
```

r[class.datetime.calendar]
`weekDay` answers the day of the week from 1 for Monday to 7 for Sunday, and
`dayName` its English name; `monthName` answers the English name of the month.
Both names begin with a capital letter. `yearDay` answers the number of the
day in its year, from 1, `baseDate` the number of days from 1 January 0001
(day 0) to the day, and `isLeapYear` answers 1 when the year is divisible by 4
and not by 100, or is divisible by 400, and 0 otherwise. `daysInMonth` answers
the length of the month in days and `daysInYear` the length of the year, 365
or 366. They agree with [the options](r:bif.date.out.counts) B and D of `DATE`
and with [the names](r:bif.date.out.names) of M and W. Each takes no argument.

```rexx
d = .datetime~new(2017, 6, 5, 0, 0, 0, 0, 0)
say d~weekDay d~dayName d~monthName d~yearDay d~baseDate
say .datetime~new(1, 1, 1)~dayName .datetime~new(9999, 12, 31)~dayName
do y over .array~of(1900, 2000, 2016, 2100, 2400, 4, 100)
  x = .datetime~new(y, 2, 1, 0, 0, 0, 0, 0)
  say y x~isLeapYear x~daysInMonth x~daysInYear .datetime~new(y, 3, 1)~yearDay
end
say .datetime~new(2017, 12, 1)~daysInMonth,
    .datetime~new(2017, 4, 1)~daysInMonth
say .datetime~new(2017, 6, 5)~baseDate == date('B', '5 Jun 2017')
say .datetime~new(2017, 6, 5)~yearDay == date('D', '5 Jun 2017')
call try 'd~weekDay(1)'
call try 'd~isLeapYear(1)'
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
1 Monday June 156 736484
Monday Friday
1900 0 28 365 60
2000 1 29 366 61
2016 1 29 366 61
2100 0 28 365 60
2400 1 29 366 61
4 1 29 366 61
100 0 28 365 60
31 30
1
1
d~weekDay(1) => 93.902
d~isLeapYear(1) => 93.902
```

r[class.datetime.isoweek]
The ISO 8601 week of a day is numbered by the week that holds the Thursday of
it. `weekNumber` answers the number of the week, 1 to 53, `weekNumberYear` the
year that week belongs to, which is the year before or after the day's own at
the turn of a year, and `weekNumberDate` the week date `yyyy-Www-d`, with the
weekday from 1 (Monday) to 7 (Sunday) as
[`fromWeekNumberDate`](r:class.datetime.fromweek) reads it. `weeksInYear`
answers 52 or 53, the number of weeks of the day's own calendar year, not of
the year of its week. `ordinalDate` answers `yyyy-ddd`, the year and the day
of the year in three digits. Each takes no argument.

```rexx
do d over .array~of(.datetime~new(2018, 12, 31), .datetime~new(2019, 1, 1), -
    .datetime~new(2020, 12, 31), .datetime~new(2021, 1, 1), -
    .datetime~new(2016, 1, 1))
  say d~isoDate~left(10) d~weekNumber d~weekNumberYear d~weekNumberDate,
      d~weeksInYear d~ordinalDate
end
say .datetime~new(1, 1, 1)~weekNumberDate,
    .datetime~new(9999, 12, 31)~weekNumberDate
say .datetime~new(5, 1, 1)~weekNumberDate .datetime~new(5, 1, 1)~ordinalDate
do y = 2010 to 2030
  w = .datetime~new(y, 6, 30)~weeksInYear
  if w \= 52 then say y w
end
call try '.datetime~new(2017, 1, 1)~weekNumber(1)'
call try '.datetime~new(2017, 1, 1)~ordinalDate(1)'
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
2018-12-31 1 2019 2019-W01-1 52 2018-365
2019-01-01 1 2019 2019-W01-2 52 2019-001
2020-12-31 53 2020 2020-W53-4 53 2020-366
2021-01-01 53 2020 2020-W53-5 52 2021-001
2016-01-01 53 2015 2015-W53-5 52 2016-001
0001-W01-1 9999-W52-5
0004-W53-6 0005-001
2015 53
2020 53
2026 53
.datetime~new(2017, 1, 1)~weekNumber(1) => 93.902
.datetime~new(2017, 1, 1)~ordinalDate(1) => 93.902
```

## Writing a moment

r[class.datetime.dateformats]
`europeanDate`, `normalDate`, `orderedDate`, `standardDate` and `usaDate`
write the date part in the form that the option E, N, O, S or U of
[DATE](r:bif.date.out.text) writes it: `dd/mm/yy`, `d mon yyyy`, `yy/mm/dd`,
`yyyymmdd` and `mm/dd/yy`. Each takes an optional separator, in place of the
default of its form ([the output separator](r:bif.date.sep.out)), which is the
null string or one character that is no letter and no digit ([the separator
rule](r:bif.date.sep.char)); any other value raises `SYNTAX 40.43`.
`languageDate` writes the day, the full name of the month and the year, as the
option L of `DATE` does, and takes no argument.

```rexx
d = .datetime~new(2017, 6, 5, 9, 8, 7, 12345, 0)
say d~europeanDate d~normalDate d~orderedDate,
    d~standardDate d~usaDate d~languageDate
say d~europeanDate('-') d~normalDate('-'),
    d~orderedDate('.') d~standardDate('-')
say d~usaDate('') d~europeanDate(' ') d~normalDate('') d~standardDate('')
say .datetime~new(5, 1, 1)~europeanDate .datetime~new(2100, 1, 1)~europeanDate
say .datetime~new(5, 1, 1)~normalDate .datetime~new(5, 1, 1)~languageDate
call try 'd~europeanDate("x")'
call try 'd~normalDate("5")'
call try 'd~usaDate("//")'
call try 'd~languageDate(1)'
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
05/06/17 5 Jun 2017 17/06/05 20170605 06/05/17 5 June 2017
05-06-17 5-Jun-2017 17.06.05 2017-06-05
060517 05 06 17 5Jun2017 20170605
01/01/05 01/01/00
1 Jan 0005 1 January 0005
d~europeanDate("x") => 40.43
d~normalDate("5") => 40.43
d~usaDate("//") => 40.43
d~languageDate(1) => 93.902
```

r[class.datetime.timeformats]
`civilTime`, `normalTime` and `longTime` write the time of day in the form
that the option C, N or L of [TIME](r:bif.time.out.clock) writes it: `h:mmam`
or `h:mmpm`, `hh:mm:ss` and `hh:mm:ss.uuuuuu`. The seconds of the civil form
are not written, and the microseconds of the normal form are dropped, not
rounded. Each takes no argument.

```rexx
say .datetime~new(2017, 6, 30, 0, 0, 0, 0, 0)~civilTime,
    .datetime~new(2017, 6, 30, 0, 5)~civilTime
say .datetime~new(2017, 6, 30, 12, 5)~civilTime,
    .datetime~new(2017, 6, 30, 13, 0)~civilTime
say .datetime~new(2017, 6, 30, 23, 59, 59, 999999)~civilTime
say .datetime~new(2017, 6, 30, 23, 59, 59, 999999)~normalTime
say .datetime~new(2017, 6, 30, 23, 59, 59, 999999)~longTime
say .datetime~new(2017, 6, 30, 1, 2, 3, 4)~longTime,
    .datetime~new(2017, 6, 30)~longTime
call try '.datetime~new(2017, 6, 30)~civilTime(1)'
call try '.datetime~new(2017, 6, 30)~longTime(1)'
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
12:00am 12:05am
12:05pm 1:00pm
11:59pm
23:59:59
23:59:59.999999
01:02:03.000004 00:00:00.000000
.datetime~new(2017, 6, 30)~civilTime(1) => 93.902
.datetime~new(2017, 6, 30)~longTime(1) => 93.902
```

r[class.datetime.counts]
`fullDate` answers the number of microseconds from 00:00:00.000000 on 1
January 0001 to the fields of the instance, as a whole number without a point
or exponent, and `utcDate` the same count for the instant in UTC, which is
`fullDate` less the offset in microseconds; it can be negative, or larger than
the count of `maxDate`, for an instant outside the range of the class. `ticks`
answers the whole seconds from 00:00:00 on 1 January 1970 to the fields,
negative before then and rounded down, so `ticks` of the last moment of 1969
is -1. The offset plays no part in `fullDate` and `ticks`. No setting of
NUMERIC DIGITS changes the answers ([the numeric
rule](r:class.datetime.numeric)). Each takes no argument.

```rexx
d = .datetime~new(2017, 6, 30, 23, 59, 59, 999999, 0)
say d~fullDate d~utcDate d~ticks
e = .datetime~new(2017, 6, 30, 23, 59, 59, 999999, 600)
say e~fullDate e~utcDate e~ticks
f = .datetime~new(2017, 6, 30, 23, 59, 59, 999999, -600)
say f~fullDate f~utcDate f~ticks
say .datetime~new(1969, 12, 31, 23, 59, 59, 500000, 0)~ticks,
    .datetime~new(1970, 1, 1, 0, 0, 0, 999999, 0)~ticks
say .datetime~new(1, 1, 1, 0, 0, 0, 0, 60)~utcDate,
    .datetime~new(9999, 12, 31, 23, 59, 59, 999999, -60)~utcDate
say .datetime~minDate~ticks
call try 'd~fullDate(1)'
call try 'd~ticks(1)'
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
63634463999999999 63634463999999999 1498867199
63634463999999999 63634427999999999 1498867199
63634463999999999 63634499999999999 1498867199
-1 0
-3600000000 315537901199999999
-62135596800
d~fullDate(1) => 93.902
d~ticks(1) => 93.902
```

r[class.datetime.string]
`isoDate`, `string` and `makeString` answer the fields in the extended ISO
8601 form `yyyy-mm-ddThh:mm:ss.uuuuuu`, whose year has four digits and whose
microseconds have six, so the instance 5 January of year 5 is `0005-01-05T`
followed by the time. `utcIsoDate` answers the same followed by the offset:
`Z` for the offset 0, otherwise a sign and `hh:mm`, `+` for the east and `-`
for the west of UTC. `fromIsoDate` and `fromUtcIsoDate` read these strings
back. Each takes no argument. The string value of an instance is this text.

```rexx
d = .datetime~new(2017, 6, 5, 9, 8, 7, 12345, 330)
say d~isoDate d~string
say d~makeString d
say d~utcIsoDate .datetime~new(2017, 6, 5, 9, 8, 7, 12345, -75)~utcIsoDate
say .datetime~new(5, 1, 5, 0, 0, 0, 0, 0)~utcIsoDate
say (.datetime~fromUtcIsoDate(d~utcIsoDate) == d),
    (.datetime~fromIsoDate(d~isoDate, 330) == d)
say d~isoDate~length d~utcIsoDate~length d~request('STRING')
call try 'd~isoDate(1)'
call try 'd~utcIsoDate(1)'
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
2017-06-05T09:08:07.012345 2017-06-05T09:08:07.012345
2017-06-05T09:08:07.012345 2017-06-05T09:08:07.012345
2017-06-05T09:08:07.012345+05:30 2017-06-05T09:08:07.012345-01:15
0005-01-05T00:00:00.000000Z
1 1
26 32 2017-06-05T09:08:07.012345
d~isoDate(1) => 93.902
d~utcIsoDate(1) => 93.902
```

## Arithmetic

r[class.datetime.add]
Each of `addYears`, `addWeeks`, `addDays`, `addHours`, `addMinutes`,
`addSeconds` and `addMicroseconds` answers a new instance, of the class of the
receiver, that is the receiver moved by its argument; the receiver is not
changed, and the offset is kept. `addYears` takes a whole number of years, read
as [the form rule](r:conv.whole.form) and [the digits rule](r:conv.whole.digits)
say ([Deviation 130](r:class.datetime.deviation.130)), and keeps month, day and
time of day, except that 29 February moves to 28 February when the year it lands
in is no leap year; a year past the range raises `SYNTAX 88.907`, and a number
of years of 1000000000 or more in magnitude, or a value such as 1.5 or `x`,
raises `SYNTAX 88.903`. The other six take a number, a
negative one moves back, and a fraction is allowed: `addDays(1.5)` is a day and
twelve hours. The new moment is the receiver's count of microseconds plus the
amount in microseconds, rounded to 18 significant digits; a part of a
microsecond that is left is dropped toward the earlier moment. So
`addMicroseconds` of 1.5 adds 1 and of -1.5 subtracts 2, and on a date of 2016,
whose count has 17 digits, of 0.96 adds 1 and of -0.04 adds nothing. A value
that is no number raises `SYNTAX 88.902`; a moment outside the range of the
class raises `SYNTAX 40.19`. A missing argument raises `SYNTAX 93.901` and a
second argument `SYNTAX 93.902`.

```rexx
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 120)
say d~addYears(1) d~addYears(4)
say d~addYears(-1) d~addYears(0)
say d~addWeeks(1) d~addWeeks(0.5)
say d~addWeeks(-1)
say d~addDays(1) d~addDays(1.5)
say d~addDays(-0.25) d~addDays(-60)
say d~addDays(365)
say d~addHours(1) d~addHours(-9.5)
say d~addHours(24)
say d~addMinutes(90.5) d~addMinutes(-548)
say d~addSeconds(0.5) d~addSeconds(86400)
say d~addMicroseconds(1) d~addMicroseconds(-12346)
say d~addMicroseconds(1e6)
say d~addDays(0.0000001)~microseconds d~addSeconds(0.0000015)~microseconds
e = .datetime~new(2016, 2, 29, 9, 8, 7, 500000, 0)
say e~addMicroseconds(1.5)~microseconds e~addMicroseconds(-1.5)~microseconds,
    e~addMicroseconds(-0.5)~microseconds
say d~addMicroseconds(0.96)~microseconds d~addMicroseconds(0.4)~microseconds
say .datetime~new(1, 1, 1, 0, 0, 0, 5)~addMicroseconds(0.96)~microseconds
say d~addSeconds(0.00000096)~microseconds,
    d~addMicroseconds(-0.04)~microseconds
say d~addDays(1)~utcIsoDate d~addYears(1)~class~id (d~addDays(1) == d)
say .datetime~new(2016, 2, 29)~addYears(-4),
    .datetime~new(2016, 2, 29)~addYears(100)
call try 'd~addYears(1.5)'
call try 'd~addYears("x")'
call try 'd~addYears(7984)'
call try 'd~addYears(-2016)'
call try 'd~addDays("x")'
call try 'd~addDays(.nil)'
call try 'd~addDays(3000000)'
call try '.datetime~maxDate~addMicroseconds(1)'
call try '.datetime~minDate~addMicroseconds(-1)'
call try 'd~addDays'
call try 'd~addDays(1, 2)'
call try 'd~addMonths(1)'
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
2017-02-28T09:08:07.012345 2020-02-29T09:08:07.012345
2015-02-28T09:08:07.012345 2016-02-29T09:08:07.012345
2016-03-07T09:08:07.012345 2016-03-03T21:08:07.012345
2016-02-22T09:08:07.012345
2016-03-01T09:08:07.012345 2016-03-01T21:08:07.012345
2016-02-29T03:08:07.012345 2015-12-31T09:08:07.012345
2017-02-28T09:08:07.012345
2016-02-29T10:08:07.012345 2016-02-28T23:38:07.012345
2016-03-01T09:08:07.012345
2016-02-29T10:38:37.012345 2016-02-29T00:00:07.012345
2016-02-29T09:08:07.512345 2016-03-01T09:08:07.012345
2016-02-29T09:08:07.012346 2016-02-29T09:08:06.999999
2016-02-29T09:08:08.012345
20985 12346
500001 499998 499999
12346 12345
5
12346 12345
2016-03-01T09:08:07.012345+02:00 DateTime 0
2012-02-29T00:00:00.000000 2116-02-29T00:00:00.000000
d~addYears(1.5) => 88.903
d~addYears("x") => 88.903
d~addYears(7984) => 88.907
d~addYears(-2016) => 88.907
d~addDays("x") => 88.902
d~addDays(.nil) => 88.902
d~addDays(3000000) => 40.19
.datetime~maxDate~addMicroseconds(1) => 40.19
.datetime~minDate~addMicroseconds(-1) => 40.19
d~addDays => 93.901
d~addDays(1, 2) => 93.902
d~addMonths(1) => 97.1
```

r[class.datetime.plus]
`+` takes a [`TimeSpan`](r:class.timespan.value) and answers a new instance,
of the class of the receiver, that is the receiver moved forward by the span
(back for a negative one); the offset is kept. Any other argument raises
`SYNTAX 93.948`, and a moment outside the range raises `SYNTAX 40.19`. The
prefix `+` has no argument and answers a new instance with the same fields and
the same offset.

```rexx
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 330)
t = .timespan~fromSeconds(90)
say d + t
say d + .timespan~fromSeconds(-90)
say (d + t)~offset (+d)~offset (+d == d)
say +d d~'+'
say d + .timespan~fromDays(0.5)
call try 'd + 5'
call try 'd + "x"'
call try 'd + d'
call try 'd + .nil'
call try '.datetime~maxDate + t'
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
2016-02-29T09:09:37.012345
2016-02-29T09:06:37.012345
05:30:00.000000 05:30:00.000000 1
2016-02-29T09:08:07.012345 2016-02-29T09:08:07.012345
2016-02-29T21:08:07.012345
d + 5 => 93.948
d + "x" => 93.948
d + d => 93.948
d + .nil => 93.948
.datetime~maxDate + t => 40.19
```

r[class.datetime.minus]
`-` has three forms. With a [`TimeSpan`](r:class.timespan.value) it answers a
new instance, of the class of the receiver, whose fields are the receiver's
moved back by the span (forward for a negative one) and whose offset is the
receiver's ([Deviation 127](r:class.datetime.deviation.127)). With a `DateTime`
it answers a `TimeSpan`, the difference of the two instants in UTC, negative
when the receiver is the earlier; the offsets of the two instances matter and
the fields alone do not decide it. With no argument, the prefix `-` raises
`SYNTAX 93.901`. Any other argument raises `SYNTAX 88.914`. A moment outside the
range of the class raises `SYNTAX 40.19`. The argument is sent `isA` with
`TimeSpan` first and then, when that answers false, `isA` with `DateTime`.

```rexx
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 120)
z = .datetime~new(2016, 3, 1, 10, 0, 0, 0, 0)
t = .timespan~fromSeconds(90)
say d - t
say (d + t)~offset == d~offset
say d - z
say z - d
say d - d
say (z - d)~class~id
say d + (z - d) == z
say .datetime~new(2017, 1, 1, 0, 0, 0, 0, 60),
    - .datetime~new(2017, 1, 1, 0, 0, 0, 0, 0)
say .datetime~maxDate - .datetime~minDate
call try 'd - 5'
call try 'd - "x"'
call try 'd - .nil'
call try '-d'
call try 'd~"-"'
call try '.datetime~minDate - t'
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
2016-02-29T09:06:37.012345
1
-1.02:51:52.987655
1.02:51:52.987655
00:00:00.000000
TimeSpan
1
-01:00:00.000000
3652058.23:59:59.999999
d - 5 => 88.914
d - "x" => 88.914
d - .nil => 88.914
-d => 93.901
d~"-" => 93.901
.datetime~minDate - t => 40.19
```

## Comparing

r[class.datetime.compare]
`compareTo` takes one argument and answers -1 when the receiver is the earlier
instant, 0 for the same instant and 1 when it is the later, comparing the
instants in UTC, so that two instances with different fields and different
offsets can be equal. The argument is sent `isA` with `DateTime`, and a
`DateTime` is then sent `utcDate`; an argument that is no `DateTime` raises
`SYNTAX 88.914` as soon as `isA` has answered ([Deviation
126](r:class.datetime.deviation.126)), and the comparison operators raise it for
such an argument too, except that an operator whose argument is `.nil` does as
[the `.nil` rule](r:class.orderable.compare.nil) says, without `compareTo`. The
comparison operators `=`, `==`, `<`, `<=`, `>`, `>=`, `<>`, `><`, `\=`, `\==`,
`<<`, `<<=`, `>>`, `>>=` and `\<`, `\>`, `\<<`, `\>>` are those of
[`Orderable`](r:class.orderable.compare), which answer from `compareTo` of the
receiver, strict and non-strict ones alike: `==` is as `=`. The text of the
instances plays no part.

```rexx
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 120)
z = .datetime~new(2016, 3, 1, 10, 0, 0, 0, 0)
q = .datetime~new(2016, 2, 29, 7, 8, 7, 12345, 0)
say d~compareTo(z) z~compareTo(d) d~compareTo(q) d~compareTo(d~copy)
say (d < z) (d > z) (d = q) (d == q) (d \= q) (d <> q) (d <= q) (d >= q)
say (d << z) (d >> z) (d <<= q) (d >>= z) (d \== q) (d >< q)
say (d~string == q~string) (d = q)
say (d \< z) (d \> z) (d \<< z) (d \>> z)
call try 'd~compareTo'
call try 'd~compareTo(z, 1)'
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
-1 1 0 0
1 0 1 1 0 0 1 1
1 0 1 0 0 0
0 1
0 1 0 1
d~compareTo => 93.901
d~compareTo(z, 1) => 93.902
```

r[class.datetime.hashcode]
`hashCode` takes no argument and answers a string of 8 bytes that is computed
from the instant of the instance. Two instances for the same instant, which `=`
finds equal, have the same hash code whatever their fields and offsets
([Deviation 128](r:class.datetime.deviation.128)); an instance and its copy have
the same one. The bytes are not specified, nor whether two different instants
have different ones.

```rexx
d = .datetime~new(2017, 6, 30, 5, 6, 7, 8, 0)
say (d~hashCode == d~copy~hashCode) d~hashCode~length
call try 'd~hashCode(1)'
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
1 8
d~hashCode(1) => 93.902
```

## Zones and parts

r[class.datetime.zone]
`toLocalTime`, `toUtcTime` and `toTimezone` answer a new instance, of the
class of the receiver, for the same instant with the fields of another zone:
the fields move by the difference of the offsets. `toLocalTime` takes the
default offset ([the default](r:class.datetime.offset.default)), `toUtcTime`
the offset 0, and `toTimezone` the offset that its argument gives ([the offset
rule](r:class.datetime.offset)), 0 when it has no argument. The instant does
not change, so `utcDate` and the comparisons see the same moment. When the new
fields fall outside the range of the class the methods raise `SYNTAX 40.19`.
`toLocalTime` and `toUtcTime` take no argument, and `toTimezone` at most one.
The argument of `toTimezone` that is not a
[`TimeSpan`](r:class.timespan.value) is read as a number of minutes by
multiplying it with 60000000 with `*`, so a string that is no number raises
`SYNTAX 41.1` and `.nil`, which has no `*`, `SYNTAX 97.1`; a number that is
not a whole number of minutes in the range raises `SYNTAX 88.907`.

```rexx
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 120)
say d~toTimezone(0) d~toUtcTime
say d~toTimezone
say d~toTimezone(-90)~utcIsoDate d~toTimezone(90)~utcIsoDate
say d~toTimezone(120)~utcIsoDate d~toTimezone(900)~utcIsoDate
say d~toTimezone(.timespan~fromHours(-5))~utcIsoDate,
    d~toTimezone(.timespan~fromMinutes(-61))~utcIsoDate
say (d~toTimezone(-77) == d),
    (d~toTimezone(-77)~utcDate == d~utcDate) d~toTimezone(-77)~timeOfDay
l = d~toLocalTime
say (l == d) (l~offset == .datetime~new~offset) (l~utcDate == d~utcDate)
say d~toTimezone(60)~class~id d~toTimezone(60)~toUtcTime~isoDate
say .datetime~minDate~toTimezone(60) .datetime~maxDate~toTimezone(-60)
call try 'd~toTimezone(901)'
call try 'd~toTimezone(1.5)'
call try 'd~toTimezone(.timespan~fromSeconds(90))'
call try 'd~toTimezone("x")'
call try 'd~toTimezone(.nil)'
call try '.datetime~minDate~toTimezone(-60)'
call try '.datetime~maxDate~toTimezone(60)'
call try '.datetime~minDate~toUtcTime'
call try 'd~toUtcTime(1)'
call try 'd~toTimezone(1, 2)'
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
2016-02-29T07:08:07.012345 2016-02-29T07:08:07.012345
2016-02-29T07:08:07.012345
2016-02-29T05:38:07.012345-01:30 2016-02-29T08:38:07.012345+01:30
2016-02-29T09:08:07.012345+02:00 2016-02-29T22:08:07.012345+15:00
2016-02-29T02:08:07.012345-05:00 2016-02-29T06:07:07.012345-01:01
1 1 05:51:07.012345
1 1 1
DateTime 2016-02-29T07:08:07.012345
0001-01-01T01:00:00.000000 9999-12-31T22:59:59.999999
d~toTimezone(901) => 88.907
d~toTimezone(1.5) => 88.907
d~toTimezone(.timespan~fromSeconds(90)) => 88.907
d~toTimezone("x") => 41.1
d~toTimezone(.nil) => 97.1
.datetime~minDate~toTimezone(-60) => 40.19
.datetime~maxDate~toTimezone(60) => 40.19
.datetime~minDate~toUtcTime => 0001-01-01T00:00:00.000000
d~toUtcTime(1) => 93.902
d~toTimezone(1, 2) => 93.902
```

r[class.datetime.parts]
`date` answers a new instance, of the class of the receiver, for the same day at
00:00:00.000000 with the same offset. `timeOfDay` answers the time past
00:00:00.000000 as a [`TimeSpan`](r:class.timespan.value), the fields of the
time and not an instant in UTC. `elapsed` answers a `TimeSpan`, the current
instant less the instant of the receiver, so that the offset of the receiver
does not change it ([Deviation 129](r:class.datetime.deviation.129)); the span
is negative when the receiver lies ahead of the clock. The current time is not
specified, so the example uses instants that are far from it. Each takes no
argument.

```rexx
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 120)
say d~date d~date~offset d~timeOfDay d~timeOfDay~class~id
say d~toTimezone(-77)~timeOfDay d~toTimezone(-77)~date
say .datetime~minDate~elapsed~sign,
    .datetime~maxDate~elapsed~sign .datetime~minDate~elapsed~class~id
call try 'd~date(1)'
call try 'd~timeOfDay(1)'
call try 'd~elapsed(1)'
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
2016-02-29T00:00:00.000000 02:00:00.000000 09:08:07.012345 TimeSpan
05:51:07.012345 2016-02-29T00:00:00.000000
1 -1 TimeSpan
d~date(1) => 93.902
d~timeOfDay(1) => 93.902
d~elapsed(1) => 93.902
```

r[class.datetime.copy]
`copy` answers a new instance of the class of the receiver with the same
fields and offset, as [the copy rule](r:class.object.copy) of `Object` says,
without sending `init`. The copy is the same moment: `=` and `==` find it
equal to the original.

```rexx
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 330)
c = d~copy
say c c~offset (c == d) (c = d) (c~identityHash == d~identityHash) c~class~id
```

```output
2016-02-29T09:08:07.012345 05:30:00.000000 1 1 0 DateTime
```

r[class.datetime.numeric]
The answers of the methods of `DateTime` do not depend on the setting of
NUMERIC DIGITS of the caller: a count of microseconds with more digits than
the setting still has all its digits, and the amount that an `add` method
reads is not rounded to the setting.

```rexx
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 0)
numeric digits 3
say d~fullDate d~utcDate
say d~addDays(1.123456789012)~isoDate d~addMicroseconds(123456789012)
say .datetime~new(315537897599999999)~isoDate,
    .datetime~fromTicks(253402300799)~isoDate
numeric digits 20
say d~fullDate d~addDays(1.123456789012)~isoDate,
    d~addMicroseconds(123456789012)~isoDate
say d~fullDate + 1
```

```output
63592333687012345 63592333687012345
2016-03-01T12:05:53.678915 2016-03-01T19:25:43.801357
9999-12-31T23:59:59.999999 9999-12-31T23:59:59.000000
63592333687012345 2016-03-01T12:05:53.678915 2016-03-01T19:25:43.801357
63592333687012346
```

## Messages that a program can override

r[class.datetime.sends.new]
The class methods send `new` to the receiver, so a class that overrides its
`new` sees them, and `new` sends `init` to the new instance with the arguments
it got ([the new rule](r:class.index.new)); the `init` of `DateTime` sends
nothing. `minDate` sends `new` with the four arguments 1, 1, 1 and 0, and
`maxDate` with the eight arguments 9999, 12, 31, 23, 59, 59, 999999 and 0.
`today` and every `from` method other than `fromOrdinalDate` and
`fromWeekNumberDate` send `new` with two arguments: the number of microseconds
from the start of 0001 to the moment, and the offset argument exactly as the
caller gave it (a [`TimeSpan`](r:class.timespan.value) is not converted), or,
without one, the default offset in minutes; `fromUtcIsoDate`, which has no
offset argument, sends the offset of its string in minutes. `fromOrdinalDate`
sends nothing to the receiver and answers a plain `DateTime`, whatever the
receiver is. `fromWeekNumberDate` sends `new` with the three arguments year, 1
and 4, then sends that instance `weeksInYear` (which sends `year`), `weekDay`
and `addDays`, and the answer of `addDays` is the result. The first example
prints, for each call, the messages sent in order, with the number of
arguments after `NEW` and `INIT`. The second example prints the second
argument that `init` gets: a `TimeSpan` or a string that the caller gave
arrives as it is, the offset of the string of `fromUtcIsoDate` as a number of
minutes (150 for `+02:30`), and, without an offset argument, the default
offset in minutes. An argument that is read as a number is sent more messages
than the one that reads its string value ([the argument
rule](r:class.index.string)). A year, month, day, hours, minutes, seconds or
microseconds argument of `new` is sent `>=` and `<=` with the limits of its
range and then `right` with the width of its field, 4 for the year, 6 for the
microseconds and 2 for the others. An offset that is not a `TimeSpan`, the
argument of `new`, `today` or a `from` method other than `fromWeekNumberDate`,
is sent `isA`, read for its string value, and sent `>=` with -900, `<=` with
900 and `*` with 60000000. An amount of `addDays`, `addHours`, `addMinutes`,
`addSeconds` and `addWeeks` is sent `*`, and `addYears` and `addMicroseconds`
read the string value of their amount twice. A number argument of `toTimezone`
is sent `isA` and `*`, then read for its string value and sent `>=` and `<=`;
the `new` that `toTimezone` sends gets the argument itself, which is then
handled as the offset of `new` is. A count of `new` and the argument of
`fromTicks` and `fromBaseDate` are read for their string value only. The third
example passes an object whose `makeString` answers a number and whose
`unknown` sends every other message to that number, and prints the messages
the object got.

```rexx
names = 'WEEKSINYEAR YEAR WEEKDAY ADDDAYS OFFSET'
do n over names~makearray(' ')
  body = .array~of(".local~log = .local~log '" || n || "'",,
    'forward class (super)')
  .spy~define(n, .method~new(n, body))
end
call probe 'new', '.spy~new(2016, 2, 29)'
call probe 'minDate', '.spy~minDate'
call probe 'maxDate', '.spy~maxDate'
call probe 'today', '.spy~today'
call probe 'fromTicks', '.spy~fromTicks(5)'
call probe 'fromBaseDate', '.spy~fromBaseDate(5, 30)'
call probe 'fromIsoDate', '.spy~fromIsoDate("2017-01-05T01:02:03.4")'
call probe 'fromUtcIsoDate',,
  '.spy~fromUtcIsoDate("2017-01-05T01:02:03.4-02:30")'
call probe 'fromWeekNumberDate', '.spy~fromWeekNumberDate("2017-W01-4")'
call probe 'fromOrdinalDate', '.spy~fromOrdinalDate("2017-005")'
exit
probe:
  .local~log = ''
  interpret 'x =' arg(2)
  s = arg(1)':' .local~log '=>' x~class~id
  line = word(s, 1)
  do i = 2 to words(s)
    if length(line word(s, i)) > 76 then do
      say line
      line = '  'word(s, i)
    end
    else line = line word(s, i)
  end
  say line
  return
::class spy subclass datetime
::method init
  .local~log = .local~log 'INIT'arg()
  forward class (super)
::method new class
  .local~log = .local~log 'NEW'arg()
  forward class (super)
```

```output
new: NEW3 INIT3 => SPY
minDate: NEW4 INIT4 => SPY
maxDate: NEW8 INIT8 => SPY
today: NEW2 INIT2 => SPY
fromTicks: NEW2 INIT2 => SPY
fromBaseDate: NEW2 INIT2 => SPY
fromIsoDate: NEW2 INIT2 => SPY
fromUtcIsoDate: NEW2 INIT2 => SPY
fromWeekNumberDate: NEW3 INIT3 WEEKSINYEAR YEAR WEEKDAY ADDDAYS OFFSET NEW2
  INIT2 => SPY
fromOrdinalDate: => DateTime
```

```rexx
call probe 'fromTicks', '.spy~fromTicks(5, .timespan~fromMinutes(30))'
call probe 'fromTicks', '.spy~fromTicks(5, " 3E1 ")'
call probe 'fromIsoDate', '.spy~fromIsoDate("2017-01-05T01:02:03.4", 45)'
call probe 'today', '.spy~today(.timespan~fromMinutes(-90))'
call probe 'fromUtcIsoDate',,
  '.spy~fromUtcIsoDate("2017-01-05T01:02:03+02:30")'
.local~log = ''
x = .spy~fromTicks(5)
say 'default' (.local~log~word(3) == '[' || time('O') / 60000000 || ']')
exit
probe:
  .local~log = ''
  interpret 'x =' arg(2)
  say arg(1)':' .local~log
  return
::class spy subclass datetime
::method init
  .local~log = .local~log 'INIT'arg() '['arg(2)']'
  forward class (super)
::method new class
  .local~log = .local~log 'NEW'arg()
  forward class (super)
```

```output
fromTicks:  NEW2 INIT2 [00:30:00.000000]
fromTicks:  NEW2 INIT2 [ 3E1 ]
fromIsoDate:  NEW2 INIT2 [45]
today:  NEW2 INIT2 [-01:30:00.000000]
fromUtcIsoDate:  NEW2 INIT2 [150]
default 1
```

```rexx
n = .num~new(3)
call probe 'year', '.datetime~new(n, 1, 1)'
call probe 'offset', '.datetime~new(2017, 1, 1, 0, 0, 0, 0, n)'
call probe 'count', '.datetime~new(n)'
call probe 'fromTicks', '.datetime~fromTicks(n)'
call probe 'addDays', '.datetime~minDate~addDays(n)'
call probe 'addMicroseconds', '.datetime~minDate~addMicroseconds(n)'
call probe 'addYears', '.datetime~minDate~addYears(n)'
call probe 'toTimezone', '.datetime~minDate~toTimezone(n)'
exit
probe:
  .local~log = ''
  interpret 'x =' arg(2)
  say arg(1)':'
  say ' ' strip(.local~log)
  return
::class num
::method init
  expose v
  use arg v
::method isA
  .local~log = .local~log 'ISA'
  forward class (super)
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
  .local~log = .local~log name
  forward to (v) message (name) arguments (args)
```

```output defect=#164
year:
  REQUEST:STRING MAKESTRING >= <= RIGHT
offset:
  ISA REQUEST:STRING MAKESTRING >= <= *
count:
  REQUEST:STRING MAKESTRING
fromTicks:
  REQUEST:STRING MAKESTRING
addDays:
  REQUEST:STRING MAKESTRING *
addMicroseconds:
  REQUEST:STRING MAKESTRING REQUEST:STRING MAKESTRING
addYears:
  REQUEST:STRING MAKESTRING REQUEST:STRING MAKESTRING
toTimezone:
  ISA * REQUEST:STRING MAKESTRING >= <= ISA REQUEST:STRING MAKESTRING >= <= *
```

r[class.datetime.sends.add]
`addDays`, `addHours`, `addMicroseconds`, `addMinutes`, `addSeconds`, `addWeeks`
and `+` send `offset` to the receiver, then `new` with two arguments: the
microseconds of the new moment and the answer of `offset`. The prefix `+` is the
message `+` with no argument and sends the same. `-` with a
[`TimeSpan`](r:class.timespan.value) sends the same two messages ([Deviation
127](r:class.datetime.deviation.127)), and `-` with a `DateTime` sends the
receiver nothing. The argument of `+` is sent `isA` with `TimeSpan`, and a
`TimeSpan` then `totalMicroseconds`. The argument of `-` is sent `isA` with
`TimeSpan` and, when that is false, `isA` with `DateTime`; a `TimeSpan` is then
sent `totalMicroseconds` and a `DateTime` `utcDate`.

```rexx
names = '+ - OFFSET ADDDAYS ADDHOURS ADDMICROSECONDS ADDMINUTES ADDSECONDS',
  'ADDWEEKS UTCDATE'
do n over names~makearray(' ')
  body = .array~of(".local~log = .local~log '" || n || "'",,
    'forward class (super)')
  .spy~define(n, .method~new(n, body))
end
d = .spy~new(2016, 2, 29, 9, 8, 7, 12345, 330)
z = .spy~new(2016, 3, 1, 9, 8, 7, 12345, 330)
t = .timespan~fromSeconds(90)
.local~log = ''
call probe 'addDays', 'd~addDays(1)'
call probe 'addHours', 'd~addHours(1)'
call probe 'addMicroseconds', 'd~addMicroseconds(1)'
call probe 'addMinutes', 'd~addMinutes(1)'
call probe 'addSeconds', 'd~addSeconds(1)'
call probe 'addWeeks', 'd~addWeeks(1)'
call probe '+', 'd + t'
call probe 'prefix +', '+d'
call probe '- date', 'd - z'
exit
probe:
  .local~log = ''
  interpret 'x =' arg(2)
  say arg(1)':' .local~log '=>' x~class~id
  return
::class spy subclass datetime
::method init
  .local~log = .local~log 'INIT'arg()
  forward class (super)
::method new class
  .local~log = .local~log 'NEW'arg()
  forward class (super)
```

```output
addDays:  ADDDAYS OFFSET NEW2 INIT2 => SPY
addHours:  ADDHOURS OFFSET NEW2 INIT2 => SPY
addMicroseconds:  ADDMICROSECONDS OFFSET NEW2 INIT2 => SPY
addMinutes:  ADDMINUTES OFFSET NEW2 INIT2 => SPY
addSeconds:  ADDSECONDS OFFSET NEW2 INIT2 => SPY
addWeeks:  ADDWEEKS OFFSET NEW2 INIT2 => SPY
+:  + OFFSET NEW2 INIT2 => SPY
prefix +:  + OFFSET NEW2 INIT2 => SPY
- date:  - UTCDATE => TimeSpan
```

r[class.datetime.sends.zone]
`addYears` sends the receiver `date`, `timeOfDay`, `year`, `month`, `day` and
`offset`, and then `new` with four arguments: the new year, the month, the day
(28 for a 29 February that the new year does not have) and the offset. It then
sends the new instance `+` with the answer of `timeOfDay`, and `+` sends what
it sends. `date` sends the receiver `standardDate` and `offset`, then `new`
with two arguments, and `timeOfDay` sends `longTime`. `toTimezone` sends `new`
with two arguments, the microseconds and the offset: a
[`TimeSpan`](r:class.timespan.value) argument converted to minutes, any other
argument as given; `toUtcTime` and `toLocalTime` send the receiver
`toTimezone`, with 0 and with the default offset in minutes, and so `new` once
more. `copy` sends nothing. The second example prints the argument of
`toTimezone` and the offset that `init` gets.

```rexx
names = 'ADDYEARS DATE STANDARDDATE OFFSET TIMEOFDAY LONGTIME YEAR MONTH DAY',
  '+ TOLOCALTIME TOTIMEZONE TOUTCTIME COPY'
do n over names~makearray(' ')
  body = .array~of(".local~log = .local~log '" || n || "'",,
    'forward class (super)')
  .spy~define(n, .method~new(n, body))
end
d = .spy~new(2016, 2, 29, 9, 8, 7, 12345, 330)
z = .spy~new(2016, 3, 1, 9, 8, 7, 12345, 330)
t = .timespan~fromSeconds(90)
.local~log = ''
call probe 'addYears', 'd~addYears(1)'
call probe 'date', 'd~date'
call probe 'timeOfDay', 'd~timeOfDay'
call probe 'toTimezone', 'd~toTimezone(30)'
call probe 'toUtcTime', 'd~toUtcTime'
call probe 'toLocalTime', 'd~toLocalTime'
call probe 'copy', 'd~copy'
exit
probe:
  .local~log = ''
  interpret 'x =' arg(2)
  s = arg(1)':' .local~log '=>' x~class~id
  line = word(s, 1)
  do i = 2 to words(s)
    if length(line word(s, i)) > 76 then do
      say line
      line = '  'word(s, i)
    end
    else line = line word(s, i)
  end
  say line
  return
::class spy subclass datetime
::method init
  .local~log = .local~log 'INIT'arg()
  forward class (super)
::method new class
  .local~log = .local~log 'NEW'arg()
  forward class (super)
```

```output
addYears: ADDYEARS DATE STANDARDDATE OFFSET NEW2 INIT2 TIMEOFDAY LONGTIME
  YEAR MONTH DAY OFFSET NEW4 INIT4 + OFFSET NEW2 INIT2 => SPY
date: DATE STANDARDDATE OFFSET NEW2 INIT2 => SPY
timeOfDay: TIMEOFDAY LONGTIME => TimeSpan
toTimezone: TOTIMEZONE NEW2 INIT2 => SPY
toUtcTime: TOUTCTIME TOTIMEZONE NEW2 INIT2 => SPY
toLocalTime: TOLOCALTIME TOTIMEZONE NEW2 INIT2 => SPY
copy: COPY => SPY
```

```rexx
d = .spy~new(2016, 2, 29, 9, 8, 7, 12345, 330)
call probe 'span', 'd~toTimezone(.timespan~fromMinutes(30))'
call probe 'number', 'd~toTimezone(30)'
call probe 'string', 'd~toTimezone(" 3E1 ")'
exit
probe:
  .local~log = ''
  interpret 'x =' arg(2)
  say arg(1)':' .local~log
  return
::class spy subclass datetime
::method init
  .local~log = .local~log 'INIT'arg() '['arg(2)']'
  forward class (super)
::method new class
  .local~log = .local~log 'NEW'arg()
  forward class (super)
::method toTimezone
  .local~log = .local~log 'TOTIMEZONE['arg(1)']'
  forward class (super)
```

```output
span:  TOTIMEZONE[00:30:00.000000] NEW2 INIT2 [30]
number:  TOTIMEZONE[30] NEW2 INIT2 [30]
string:  TOTIMEZONE[ 3E1 ] NEW2 INIT2 [ 3E1 ]
```

r[class.datetime.sends.compare]
`compareTo` sends its argument `isA` with `DateTime` and, for a `DateTime`,
`utcDate`; the receiver is sent nothing, and an argument that is no `DateTime`
is sent nothing but `isA` ([Deviation 126](r:class.datetime.deviation.126)). A
comparison operator sends the messages that [the sends rule of
`Orderable`](r:class.orderable.compare.sends) states: `compareTo` to the
receiver with the argument it got, after another operator for some of them.

```rexx
names = 'COMPARETO UTCDATE = == < > <= >= <> >< << >> <<= >>= \= \== \< \>',
  '\<< \>>'
do n over names~makearray(' ')
  body = .array~of(".local~log = .local~log '" || n || "'",,
    'forward class (super)')
  .spy~define(n, .method~new(n, body))
end
d = .spy~new(2016, 2, 29, 9, 8, 7, 12345, 330)
z = .spy~new(2016, 3, 1, 9, 8, 7, 12345, 330)
t = .timespan~fromSeconds(90)
.local~log = ''
call probe 'compareTo', 'd~compareTo(z)'
call probe '=', 'd = z'
call probe '==', 'd == z'
call probe '<', 'd < z'
call probe '<>', 'd <> z'
call probe '><', 'd >< z'
call probe '\=', 'd \= z'
call probe '\==', 'd \== z'
call probe '\<', 'd \< z'
call probe '\>', 'd \> z'
call probe '\<<', 'd \<< z'
call probe '\>>', 'd \>> z'
exit
probe:
  .local~log = ''
  interpret 'x =' arg(2)
  say arg(1)':' .local~log '=>' x~class~id
  return
::class spy subclass datetime
::method init
  .local~log = .local~log 'INIT'arg()
  forward class (super)
::method new class
  .local~log = .local~log 'NEW'arg()
  forward class (super)
```

```output
compareTo:  COMPARETO UTCDATE => String
=:  = COMPARETO UTCDATE => String
==:  == COMPARETO UTCDATE => String
<:  < COMPARETO UTCDATE => String
<>:  <> \= COMPARETO UTCDATE => String
><:  >< \= COMPARETO UTCDATE => String
\=:  \= COMPARETO UTCDATE => String
\==:  \== COMPARETO UTCDATE => String
\<:  \< >= COMPARETO UTCDATE => String
\>:  \> <= COMPARETO UTCDATE => String
\<<:  \<< >>= COMPARETO UTCDATE => String
\>>:  \>> <<= COMPARETO UTCDATE => String
```

r[class.datetime.sends.string]
`string` sends `isoDate`, and `makeString` sends `string`. `isLeapYear` sends
`year`; `daysInMonth` sends `month` and `isLeapYear`; `daysInYear` sends
`isLeapYear`; `ordinalDate` sends `year` and `yearDay`; `weekNumber` sends
`yearDay` and `weekDay`; `weekNumberYear` sends `year`, `weekNumber` and
`month`; `weekNumberDate` sends `weekNumberYear`, `weekNumber` and `weekDay`,
and each of these sends what it sends in turn; `weeksInYear` sends `year`;
`dayMicroseconds` sends `daySeconds` and `microseconds`. The other methods
that answer a field or a format send nothing.

```rexx
names = 'STRING ISODATE MAKESTRING ISLEAPYEAR YEAR MONTH DAYSINMONTH',
  'DAYSINYEAR ORDINALDATE YEARDAY WEEKNUMBER WEEKDAY WEEKNUMBERYEAR',
  'WEEKNUMBERDATE WEEKSINYEAR DAYMICROSECONDS DAYSECONDS MICROSECONDS'
do n over names~makearray(' ')
  body = .array~of(".local~log = .local~log '" || n || "'",,
    'forward class (super)')
  .spy~define(n, .method~new(n, body))
end
d = .spy~new(2016, 2, 29, 9, 8, 7, 12345, 330)
z = .spy~new(2016, 3, 1, 9, 8, 7, 12345, 330)
t = .timespan~fromSeconds(90)
.local~log = ''
call probe 'string', 'd~string'
call probe 'makeString', 'd~makeString'
call probe 'isLeapYear', 'd~isLeapYear'
call probe 'daysInMonth', 'd~daysInMonth'
call probe 'daysInYear', 'd~daysInYear'
call probe 'ordinalDate', 'd~ordinalDate'
call probe 'weekNumber', 'd~weekNumber'
call probe 'weekNumberYear', 'd~weekNumberYear'
call probe 'weekNumberDate', 'd~weekNumberDate'
call probe 'weeksInYear', 'd~weeksInYear'
call probe 'dayMicroseconds', 'd~dayMicroseconds'
exit
probe:
  .local~log = ''
  interpret 'x =' arg(2)
  s = arg(1)':' .local~log '=>' x~class~id
  line = word(s, 1)
  do i = 2 to words(s)
    if length(line word(s, i)) > 76 then do
      say line
      line = '  'word(s, i)
    end
    else line = line word(s, i)
  end
  say line
  return
::class spy subclass datetime
::method init
  .local~log = .local~log 'INIT'arg()
  forward class (super)
::method new class
  .local~log = .local~log 'NEW'arg()
  forward class (super)
```

```output
string: STRING ISODATE => String
makeString: MAKESTRING STRING ISODATE => String
isLeapYear: ISLEAPYEAR YEAR => String
daysInMonth: DAYSINMONTH MONTH ISLEAPYEAR YEAR => String
daysInYear: DAYSINYEAR ISLEAPYEAR YEAR => String
ordinalDate: ORDINALDATE YEAR YEARDAY => String
weekNumber: WEEKNUMBER YEARDAY WEEKDAY => String
weekNumberYear: WEEKNUMBERYEAR YEAR WEEKNUMBER YEARDAY WEEKDAY MONTH =>
  String
weekNumberDate: WEEKNUMBERDATE WEEKNUMBERYEAR YEAR WEEKNUMBER YEARDAY
  WEEKDAY MONTH WEEKNUMBER YEARDAY WEEKDAY WEEKDAY => String
weeksInYear: WEEKSINYEAR YEAR => String
dayMicroseconds: DAYMICROSECONDS DAYSECONDS MICROSECONDS => String
```

## Deviations

r[class.datetime.deviation.109]
An instance of a subclass of `String` that a method takes as a string, a
number or a separator is read by its own characters, and `REQUEST`,
`MAKESTRING` and `STRING` are not sent to it ([Deviation
109](r:class.string.deviation.109)). ooRexx sends such an instance `request`
with `STRING` and then `makeString`, so an override can change what is read.
In the example `makeString` answers a different string, and the instance still
stands for its own characters.

```rexx
.local~sent = 0
a = .digits~new('5')
b = .digits~new('05/01/17')
c = .digits~new('2017')
call show '.datetime~fromBaseDate(a) .datetime~fromEuropeanDate(b)'
call show '.datetime~new(c, 6, 3)'
call show '.datetime~fromTicks(a)',
  ".datetime~fromEuropeanDate('05-01-17', .digits~new('-'))"
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
0001-01-06T00:00:00.000000 2017-01-05T00:00:00.000000
2017-06-03T00:00:00.000000
1970-01-01T00:00:05.000000 2017-01-05T00:00:00.000000
sends 0
```

r[class.datetime.deviation.126]
`compareTo` of a `DateTime` with an argument that is not a `DateTime` (an
instance of a subclass is one), and a comparison operator with such an argument,
raise `SYNTAX 88.914`. The error follows the `isA` that the method sends to the
argument and precedes `utcDate` and every other message; an operator whose
argument is `.nil` sends no `compareTo` and does as [the `.nil`
rule](r:class.orderable.compare.nil) says, and `compareTo` itself raises for
`.nil`. ooRexx answers 1 or -1 for such an argument, whatever the receiver is,
so the answer says nothing about the two values. In the example the last line is
a probe that answers false to `isA` and gets nothing else.

```rexx
d = .datetime~new(2017, 6, 30, 5, 6, 7, 8, 0)
call try 'd~compareTo(.timespan~fromHours(2))'
call try 'd~compareTo(.file~new("/t/a"))'
call try 'd~compareTo("2017-06-30")'
call try 'd~compareTo(5)'
call try 'd~compareTo(.nil)'
call try 'd < .timespan~fromHours(2)'
call try 'd <<= "x"'
call try 'd = 5'
call try 'd \== .object~new'
call try 'd = .nil'
call try 'd < .nil'
call try 'd~compareTo(.sub~new(2017, 6, 30, 5, 6, 7, 8, 0))'
call try 'd~compareTo(d~copy)'
.local~log = ''
call try 'd~compareTo(.probe~new)'
say 'sent:' .local~log
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class sub subclass datetime
::class probe
::method isA
  .local~log = .local~log 'ISA('arg(1)~id')'
  return 0
::method utcDate
  .local~log = .local~log 'UTCDATE'
  return 0
```

```output
d~compareTo(.timespan~fromHours(2)) => 88.914
d~compareTo(.file~new("/t/a")) => 88.914
d~compareTo("2017-06-30") => 88.914
d~compareTo(5) => 88.914
d~compareTo(.nil) => 88.914
d < .timespan~fromHours(2) => 88.914
d <<= "x" => 88.914
d = 5 => 88.914
d \== .object~new => 88.914
d = .nil => 0
d < .nil => 93.973
d~compareTo(.sub~new(2017, 6, 30, 5, 6, 7, 8, 0)) => 0
d~compareTo(d~copy) => 0
d~compareTo(.probe~new) => 88.914
sent:  ISA(DateTime)
```

r[class.datetime.deviation.127]
`+` and `-` with a [`TimeSpan`](r:class.timespan.value), and `+` of a `TimeSpan`
with a `DateTime`, answer an instance that has the offset of the `DateTime`
operand: the fields move by the span and the offset stays, so the instant moves
by the span. `-` with a span sends the receiver `offset` and then `new` with the
new microseconds and the answer of `offset`, as `+` does, and a `TimeSpan` plus
a `DateTime` sends the argument `isA` with `TimeSpan` and `isA` with `DateTime`,
then `offset`, and then sends the class of the argument `new` with two
arguments. ooRexx gives the answer of `-` with a span, and of a span plus a
`DateTime`, the offset of the host's zone, and sends `new` with one argument.
The example gives the offsets with explicit values, so that it does not depend
on the host's zone.

```rexx
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 330)
t = .timespan~fromSeconds(90)
say d + t
say d - t
say t + d
say (d + t)~offset (d - t)~offset (t + d)~offset
say (d - t)~utcIsoDate
say (t + d)~utcIsoDate
say (d - .timespan~fromHours(10))~utcIsoDate
say (.timespan~fromDays(1) + d~toTimezone(-90))~utcIsoDate
say (d - t)~class~id (t + d)~class~id (t + .sub~new(2016, 2, 29))~class~id
m = .spy~new(2016, 2, 29, 9, 8, 7, 12345, 330)
.local~log = ''
x = m - t
say 'minus:' .local~log
.local~log = ''
x = t + m
say 'plus:' .local~log
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
  .local~log = .local~log 'ISA'
  forward class (super)
```

```output defect=#190
2016-02-29T09:09:37.012345
2016-02-29T09:06:37.012345
2016-02-29T09:09:37.012345
05:30:00.000000 05:30:00.000000 05:30:00.000000
2016-02-29T09:06:37.012345+05:30
2016-02-29T09:09:37.012345+05:30
2016-02-28T23:08:07.012345+05:30
2016-03-01T02:08:07.012345-01:30
DateTime DateTime SUB
minus:  OFFSET NEW2
plus:  ISA ISA OFFSET NEW2
```

r[class.datetime.deviation.128]
`hashCode` is computed from the instant, so instances that `=` finds equal have
equal hash codes whatever their offsets, and a `Set` or a `Table` finds one
instance by another of the same instant. The hash code of an instance whose
instant differs may or may not be different, and the example does not compare
it. ooRexx computes it from the fields: the instances of one instant at two
offsets have two hash codes, and a `Set` holds both.

```rexx
d = .datetime~new(2017, 6, 30, 5, 6, 7, 8, 0)
s = .datetime~new(2017, 6, 30, 6, 6, 7, 8, 60)
t = .datetime~new(2017, 6, 30, 5, 6, 7, 8, 60)
say (d = s) (d~hashCode == s~hashCode) (d = t)
say (d~hashCode == d~copy~hashCode) (d~hashCode == d~toTimezone(-300)~hashCode)
say d~hashCode~length
x = .set~new
x~put(d)
x~put(s)
x~put(t)
say x~items
y = .table~new
y[d] = 'found'
say y[s] y[t]
```

```output
1 1 0
1 1
8
2
found The NIL object
```

r[class.datetime.deviation.129]
`elapsed` answers the span from the instant of the receiver to the current
instant, so that instances of one instant answer the same span, apart from the
time between two calls, whatever their offsets and the zone of the host. ooRexx
subtracts the fields of the receiver from the host's local time, so the answer
shifts with the offset of the receiver. The example allows five seconds between
two answers, so it does not depend on the time it runs.

```rexx
d = .datetime~new(2016, 2, 29, 9, 8, 7, 12345, 120)
say (d == d~toTimezone(-77)) (d == d~toUtcTime)
say near(d~elapsed, d~toTimezone(-77)~elapsed)
say near(d~elapsed, d~toUtcTime~elapsed)
say near(d~elapsed, d~toTimezone(900)~elapsed)
say near(d~elapsed, .datetime~new - d)
say d~elapsed~sign .datetime~maxDate~elapsed~sign d~elapsed~class~id
exit
near: return abs(arg(1)~totalSeconds - arg(2)~totalSeconds) < 5
```

```output
1 1
1
1
1
1
1 -1 TimeSpan
```

r[class.datetime.deviation.130]
Every whole-number argument of `DateTime` is read as a whole-number argument of
a builtin is: its form by [the form rule](r:conv.whole.form) and its value by
[the digits rule](r:conv.whole.digits), so NUMERIC DIGITS of the caller plays no
part. A value that those rules refuse raises the error that the place gives for
a number that is not whole, in place of `SYNTAX 40.12`. So `2017.0`, `2.017E3`
and `'2017 '` are 2017, `2017.0000000001` and `5.00000000000000001` are refused,
and `2017.00000000000000000001`, whose fraction lies past the 18th significant
digit, is 2017. The places are the count of microseconds, the year, the month,
the day, the hours, the minutes, the seconds, the microseconds and the offset of
`new`, the offset of `today`, of `toTimezone` and of the `from` methods that
read one, the years of `addYears`, the days of `fromBaseDate` and the seconds of
`fromTicks`. ooRexx refuses some of these forms, `2017.0` as a year and `1.0` as
a number of years among them, reads `2017.0000000001` as year 1, `2.0000000001`
as month 1 and `1.0000000001` as an hour of 1, and reads the days of
`fromBaseDate` rounded to nine digits.

```rexx
call show '.datetime~new(2017.0, 6, 30)~isoDate'
call show '.datetime~new(2.017E3, 6.0, 30, 1.0, 0, 0, 0, 6E1)~isoDate'
call show ".datetime~new('2017 ', ' 6', '+3')~isoDate"
call show ".datetime~new(' 90.0 ')~isoDate"
call show '.datetime~new(5, 6E1)~utcIsoDate'
call show '.datetime~fromBaseDate(5.0, 60.0)~utcIsoDate'
call show ".datetime~fromTicks('5 ', 6E1)~utcIsoDate"
call show '.datetime~new(2017, 6, 30)~toTimezone(6E1)~offset'
numeric digits 3
call show '.datetime~new(2017.0, 6, 30)~isoDate'
numeric digits 9
call try '.datetime~new(2017.0000000001, 6, 30)'
call try '.datetime~new(2017.00000000000000000001, 6, 30)'
call try '.datetime~new(2017, 6.5E0, 30)'
call try '.datetime~new(2017, 6, 30, 1.0000000001, 0)'
call try '.datetime~new(2017, 6, 30, 0, 0, 0, 5.0000000001)'
call try '.datetime~new(2017, 6, 30, 0, 0, 0, 0, 60.0000000001)'
call try '.datetime~new(5.0000000001)'
call try '.datetime~new(5, 60.0000000001)'
call try '.datetime~today(60.0000000001)'
call try '.datetime~fromBaseDate(5.0000000001)'
call try '.datetime~fromBaseDate(5.00000000000000001)'
call try '.datetime~fromBaseDate(5.000000000000000001)'
call try '.datetime~fromBaseDate(5, 60.0000000001)'
call try '.datetime~fromTicks(5.0000000001)'
call try '.datetime~fromTicks(5, 60.0000000001)'
call try '.datetime~new(2017, 6, 30)~toTimezone(60.0000000001)'
call try '.datetime~new(2017, 6, 30)~addYears(1.0)'
call try '.datetime~new(2017, 6, 30)~addYears(1.0000000001)'
call try '.datetime~new(2017, 6, 30)~addYears(1000000000)'
exit
show:
  signal on syntax name failed
  interpret 'say' arg(1)
  return
failed: say '=>' condition('o')~code
  return
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
2017-06-30T00:00:00.000000
2017-06-30T01:00:00.000000
2017-06-03T00:00:00.000000
0001-01-01T00:00:00.000090
0001-01-01T00:00:00.000005+01:00
0001-01-06T00:00:00.000000+01:00
1970-01-01T00:00:05.000000+01:00
01:00:00.000000
2017-06-30T00:00:00.000000
.datetime~new(2017.0000000001, 6, 30) => 88.907
.datetime~new(2017.00000000000000000001, 6, 30) => 2017-06-30T00:00:00.000000
.datetime~new(2017, 6.5E0, 30) => 88.907
.datetime~new(2017, 6, 30, 1.0000000001, 0) => 88.907
.datetime~new(2017, 6, 30, 0, 0, 0, 5.0000000001) => 88.907
.datetime~new(2017, 6, 30, 0, 0, 0, 0, 60.0000000001) => 88.907
.datetime~new(5.0000000001) => 40.19
.datetime~new(5, 60.0000000001) => 88.907
.datetime~today(60.0000000001) => 88.918
.datetime~fromBaseDate(5.0000000001) => 88.918
.datetime~fromBaseDate(5.00000000000000001) => 88.918
.datetime~fromBaseDate(5.000000000000000001) => 0001-01-06T00:00:00.000000
.datetime~fromBaseDate(5, 60.0000000001) => 88.918
.datetime~fromTicks(5.0000000001) => 88.918
.datetime~fromTicks(5, 60.0000000001) => 88.918
.datetime~new(2017, 6, 30)~toTimezone(60.0000000001) => 88.907
.datetime~new(2017, 6, 30)~addYears(1.0) => 2018-06-30T00:00:00.000000
.datetime~new(2017, 6, 30)~addYears(1.0000000001) => 88.903
.datetime~new(2017, 6, 30)~addYears(1000000000) => 88.903
```

r[class.datetime.deviation.131]
An instance of a subclass of `DateTime` whose `init` does not forward to the
`init` of `DateTime` has the value of `minDate`: 1 January 0001 at
00:00:00.000000 with the offset 0. Every method works on it as on that value;
the class of an answer is the class of the receiver where the method says so,
and a copy of it is one too. ooRexx answers with errors from most methods, or
with text that is no value.

```rexx
d = .sd~new
m = .datetime~minDate
call show 'd d~offset d~fullDate d~isoDate d~utcIsoDate'
call show '(d = m) (d == m) (d~compareTo(m)) (d~hashCode == m~hashCode)'
call show 'd~year d~month d~day d~hours d~minutes d~seconds d~microseconds'
call show 'd~dayName d~weekDay d~yearDay d~ticks d~baseDate d~timeOfDay'
call show 'd~addDays(1) d~addYears(1) d~toTimezone(60)'
call show 'd + .timespan~fromHours(1)'
call show '(d - m) (d - .timespan~new(0))'
call show 'd~date d~toUtcTime d~copy d~makeString'
call show 'd~addDays(1)~class~id d~copy~class~id (d~elapsed~sign)'
exit
show:
  signal on syntax name failed
  interpret 'say' arg(1)
  return
failed: say '=>' condition('o')~code
  return
::class sd subclass datetime
::method init
  nop
```

```output
0001-01-01T00:00:00.000000 00:00:00.000000 0 0001-01-01T00:00:00.000000 0001-01-01T00:00:00.000000Z
1 1 0 1
0001 1 1 0 0 0 0
Monday 1 1 -62135596800 0 00:00:00.000000
0001-01-02T00:00:00.000000 0002-01-01T00:00:00.000000 0001-01-01T01:00:00.000000
0001-01-01T01:00:00.000000
00:00:00.000000 0001-01-01T00:00:00.000000
0001-01-01T00:00:00.000000 0001-01-01T00:00:00.000000 0001-01-01T00:00:00.000000 0001-01-01T00:00:00.000000
SD SD 1
```

r[class.datetime.deviation.132]
`fromUtcIsoDate` refuses with `SYNTAX 88.918` every string that is not exactly
one of the forms that [its rule](r:class.datetime.fromutciso) gives: no blank
anywhere, a point only with one to six digits after it, an offset of the form
`+hh:mm` or `-hh:mm` with both parts of two digits, minutes up to 59 and the
whole at most 15:00, and nothing after the `Z` or the offset. The example shows
the accepted extremes and, after them, spellings that are refused. ooRexx
refuses a blank at the start or after a `Z`, a blank before a `Z` that follows
no fraction, and the string that ends in ` x`; it accepts the others, reading
some in its own way: `+02:60` as `+03:00`, a seventh digit of the fraction as
nothing, and `Z` after a point as a time with no fraction.

```rexx
base = '2017-06-30T11:08:50'
say .datetime~fromUtcIsoDate(base'Z')~utcIsoDate
say .datetime~fromUtcIsoDate(base'.1Z')~utcIsoDate
say .datetime~fromUtcIsoDate(base'.123456-15:00')~utcIsoDate
say .datetime~fromUtcIsoDate(base'.5+14:59')~utcIsoDate
say .datetime~fromUtcIsoDate(base'-00:00')~utcIsoDate
bad = .array~of(' 'base'Z', base'Z ', base' Z', base'.123456 Z', base'.123456 +02:00',,
  base'.123456+02:60', base'.123456+14:60', base'.123456+02:00 ', base'.1234567Z',,
  base'.1234567+01:00', base'.Z', base'.+02:00', base'.123456+2:00',,
  base'.123456+02:0', base'.123456+02:1', base'.123456+ 02:00',,
  base'.123456+02 :00', base'.123456+-02:00', base'.123456+0:00',,
  base'.123456+02:00Z', base'.123456ZZ', base'.123456%Z', base'.123456+02:00 x')
do s over bad
  call try s
end
exit
try:
  signal on syntax name bad
  r = .datetime~fromUtcIsoDate(arg(1))
  say '['arg(1)'] =>' r~utcIsoDate
  return
bad: say '['arg(1)'] =>' condition('o')~code
  return
```

```output
2017-06-30T11:08:50.000000Z
2017-06-30T11:08:50.100000Z
2017-06-30T11:08:50.123456-15:00
2017-06-30T11:08:50.500000+14:59
2017-06-30T11:08:50.000000Z
[ 2017-06-30T11:08:50Z] => 88.918
[2017-06-30T11:08:50Z ] => 88.918
[2017-06-30T11:08:50 Z] => 88.918
[2017-06-30T11:08:50.123456 Z] => 88.918
[2017-06-30T11:08:50.123456 +02:00] => 88.918
[2017-06-30T11:08:50.123456+02:60] => 88.918
[2017-06-30T11:08:50.123456+14:60] => 88.918
[2017-06-30T11:08:50.123456+02:00 ] => 88.918
[2017-06-30T11:08:50.1234567Z] => 88.918
[2017-06-30T11:08:50.1234567+01:00] => 88.918
[2017-06-30T11:08:50.Z] => 88.918
[2017-06-30T11:08:50.+02:00] => 88.918
[2017-06-30T11:08:50.123456+2:00] => 88.918
[2017-06-30T11:08:50.123456+02:0] => 88.918
[2017-06-30T11:08:50.123456+02:1] => 88.918
[2017-06-30T11:08:50.123456+ 02:00] => 88.918
[2017-06-30T11:08:50.123456+02 :00] => 88.918
[2017-06-30T11:08:50.123456+-02:00] => 88.918
[2017-06-30T11:08:50.123456+0:00] => 88.918
[2017-06-30T11:08:50.123456+02:00Z] => 88.918
[2017-06-30T11:08:50.123456ZZ] => 88.918
[2017-06-30T11:08:50.123456%Z] => 88.918
[2017-06-30T11:08:50.123456+02:00 x] => 88.918
```

r[class.datetime.deviation.163]
An `init` sent to a `DateTime` that raises an error leaves the receiver as it
was: every argument is checked before the instance is changed ([Deviation
163](../deviations.md)). ooRexx stores the date of the arguments before it
checks the offset, so the example is not run against it.

```rexx
d = .datetime~new(2020, 1, 1)
before = d~isoDate d~offset
signal on syntax
d~init(2018, 2, 3, 0, 0, 0, 0, 901)
exit
syntax:
say condition('O')~code (d~isoDate d~offset == before) d~isoDate
```

```output
88.907 1 2020-01-01T00:00:00.000000
```
