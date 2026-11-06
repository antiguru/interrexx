---
element: TIME
kind: builtin
syntax: "TIME([option [, [string] [, informat]]])"
ansi: "9.8.5"
rexxref: "Functions > Built-in Functions > TIME"
---

# TIME

TIME answers a time of day. With no `string` it answers the current local time,
or the reading of an elapsed-time clock that the program can start and reset;
with a `string` it reads a time written in one format and writes it in
another. The current time changes from moment to moment, so the examples of this
page convert fixed times, or check only the form of the current time and how its
formats agree with each other, and the elapsed-time clock is checked against
pauses and not against figures.

## Arguments

r[bif.time.args]
TIME takes up to three arguments and requires none: `option`, `string` and
`informat`. The argument count is checked by the [builtin call
rules](r:bif.index.call.count): a call with more than three arguments
raises `SYNTAX 40.4`, and a call with only omitted places counts as a call
with no arguments. TIME does not run as a String method: sending the
message `TIME` to a string raises `SYNTAX 97.1`. `option` and `informat` name
formats by [their first letter](r:bif.index.dt.letter).

```rexx
say time('C', '13:05:09') time(, '13:05:09') time('N', '13:05:09', 'N') time('N', '13:05:09', )
signal on syntax name a
say time('N', '13:05:09', 'N', 1)
a: say condition('o')~code
signal on syntax name b
say '13:05:09'~time
b: say condition('o')~code
```

```output
1:05pm 13:05:09 13:05:09 13:05:09
40.4
97.1
```

r[bif.time.default]
An omitted `option` is N, and so is an omitted `informat`. A call with no
`string` answers the current time, in the format `option` names; a call with
a `string` is a [conversion](r:bif.index.dt.convert).

```rexx
say time(, '07:08:09') time('N', '07:08:09') time(, '07:08:09', 'N')
say (time() == time('N')) (length(time()) == 8)
```

```output
07:08:09 07:08:09 07:08:09
1 1
```

## Formats written

r[bif.time.out.clock]
Three options write the time of day as hours, minutes and seconds. N writes
`hh:mm:ss`, with the hour from 00 to 23, and L writes
`hh:mm:ss.uuuuuu`, which adds six digits for the microseconds. C writes
`h:mmam` or `h:mmpm`, a civil time: the hour is from 1 to 12 with no
leading zero, the minutes have two digits and the seconds are not written. The
hour 0 is 12 with `am`, the hours 1 to 11 have `am`, the hour 12 is 12 with
`pm`, and the hours 13 to 23 are 1 to 11 with `pm`.

```rexx
say time('N', '00:00:00') time('N', '09:08:07') time('N', '23:59:59')
say time('L', '09:08:07') time('L', '09:08:07.5', 'L') time('L', '23:59:59.999999', 'L')
say time('C', '00:00:00') time('C', '00:59:59') time('C', '01:00:00') time('C', '11:59:59')
say time('C', '12:00:00') time('C', '12:59:59') time('C', '13:00:00') time('C', '23:59:59')
```

```output
00:00:00 09:08:07 23:59:59
09:08:07.000000 09:08:07.500000 23:59:59.999999
12:00am 12:59am 1:00am 11:59am
12:00pm 12:59pm 1:00pm 11:59pm
```

r[bif.time.out.counts]
Three options write the time of day as a count since midnight: H the whole
hours, M the whole minutes and S the whole seconds. Each is written with
no leading zero, point or exponent, as `0` for zero, and a part of a unit that
is not complete is dropped, never rounded. NUMERIC DIGITS plays no part.

```rexx
numeric digits 2
say time('H', '00:00:00') time('H', '09:59:59') time('H', '23:59:59')
say time('M', '00:00:59') time('M', '01:00:00') time('M', '23:59:59')
say time('S', '00:00:01') time('S', '01:00:00') time('S', '23:59:59')
```

```output
0 9 23
0 60 1439
1 3600 86399
```

r[bif.time.out.stamp]
Two options write a count that includes the date. F writes the number of
microseconds since the start of 1 January 0001, and T the number of seconds
since the start of 1 January 1970, which is negative before it. T is F
divided by 1000000, rounded down, less 62135596800. The string of a conversion
has no date, so its date is 1 January 0001: F is then the time of day in
microseconds, and T is that time of day in seconds less 62135596800, a negative
number. NUMERIC DIGITS plays no part in how they are written.

```rexx
say time('F', '00:00:00') time('F', '00:00:01') time('F', '23:59:59.999999', 'L')
say time('T', '00:00:00') time('T', '00:00:01') time('T', '23:59:59.999999', 'L')
numeric digits 3
say time('F', '13:05:09') time('T', '13:05:09')
say time('F', '13:05:09') + 0
```

```output
0 1000000 86399999999
-62135596800 -62135596799 -62135510401
47109000000 -62135549691
4.71E+10
```

## The current time

r[bif.time.current]
A call with no `string` answers the current time in the local time zone, in
the format `option` names, as of the [moment](r:bif.index.dt.stamp) the clause
reads the clock. Every format of the current time is the same time: converting
the current time in L form to N, C, H, M, S or L gives what that option gives
for the current time. F and T include the date, and F with its whole days taken
off is what the conversion of the L form gives for F.

```rexx
numeric digits 20
say (time('N') == time('N', time('L'), 'L')) (time('C') == time('C', time('L'), 'L')),
    (time('H') == time('H', time('L'), 'L')) (time('M') == time('M', time('L'), 'L')),
    (time('S') == time('S', time('L'), 'L')) (time('L') == time('L', time('L'), 'L'))
say (time('F') // 86400000000 == time('F', time('L'), 'L')),
    (time('T') == date('T'))
say length(time('L')) length(time('N')) datatype(time('S'), 'W') datatype(time('F'), 'W') datatype(time('T'), 'W')
```

```output
1 1 1 1 1 1
1 1
15 8 1 1 1
```

## The time zone

r[bif.time.zone]
The current time of every format is that of the local time zone, and the
option O writes how far that zone is from UTC: a whole number of microseconds,
positive east of UTC and negative to the west, and less than a day. F and T are
local like the rest: the UTC reading of F is `F - O`. A call that gives O
with a `string` reads the string as usual and answers the same offset. This book
does not specify which time zone a program runs in.

```rexx
numeric digits 20
o = time('O')
say datatype(o, 'W') (abs(o) < 86400000000) (time('O') == time('O', '12:00:00')),
    (time('T') == time('F') % 1000000 - 62135596800) (time('S') == time('T') // 86400)
say time('O', '00:00:00') == time('O', '23:59:59.5', 'L')
signal on syntax name a
say time('O', 'noon')
a: say condition('o')~code
```

```output
1 1 1 1 1
1
40.19
```

## Reading a time

r[bif.time.in.text]
The informats N, L and C read the formats that the options of the same letters
[write](r:bif.time.out.clock), and accept nothing else: no blank or other
character before, after or inside the time, and no other number of digits in
a field. N has two-digit hours from 00 to 23, minutes from 00 to 59 and
seconds from 00 to 59. L has the same, followed by a point and from one to six
digits, which are the microseconds when read as if completed on the right with
zeros. C has an hour from 1 to 12 written with or without a leading zero,
a colon, minutes of two digits from 00 to 59, and `am` or `pm` in lower case; 12 with
`am` is midnight, 12 with `pm` is noon.

```rexx
say time('N', '23:59:59') time('N', '12:00am', 'C') time('N', '12:30pm', 'C') time('N', '01:30pm', 'C'),
    time('N', '1:30pm', 'C') time('N', '11:59pm', 'C') time('N', '00:30:00')
say time('L', '01:02:03.4', 'L') time('L', '01:02:03.45', 'L') time('L', '01:02:03.000001', 'L')
signal on syntax name a
say time('N', '1:02:03')
a: say condition('o')~code
signal on syntax name b
say time('N', '24:00:00')
b: say condition('o')~code
signal on syntax name c
say time('N', '23:59:60')
c: say condition('o')~code
signal on syntax name d
say time('N', '01:02')
d: say condition('o')~code
signal on syntax name e
say time('N', '01:02:03.5')
e: say condition('o')~code
signal on syntax name f
say time('L', '01:02:03', 'L')
f: say condition('o')~code
signal on syntax name g
say time('L', '01:02:03.1234567', 'L')
g: say condition('o')~code
signal on syntax name h
say time('N', '13:00pm', 'C')
h: say condition('o')~code
signal on syntax name i
say time('N', '0:30am', 'C')
i: say condition('o')~code
signal on syntax name j
say time('N', '12:30PM', 'C')
j: say condition('o')~code
signal on syntax name k
say time('N', '12:30 pm', 'C')
k: say condition('o')~code
signal on syntax name l
say time('N', '12:3pm', 'C')
l: say condition('o')~code
```

```output
23:59:59 00:00:00 12:30:00 13:30:00 13:30:00 23:59:00 00:30:00
01:02:03.400000 01:02:03.450000 01:02:03.000001
40.19
40.19
40.19
40.19
40.19
40.19
40.19
40.19
40.19
40.19
40.19
40.19
```

r[bif.time.in.number]
The informats H, M, S, F and T read a count: hours, minutes or seconds since
midnight, or the number of microseconds or seconds that F and T
[write](r:bif.time.out.stamp). The string may have any form of a number
whose value is a whole number, such as `5`, ` 5 `, `+5`, `5.0` and `5e0`, and
the number of digits it has plays no part in whether it is accepted; NUMERIC
DIGITS does not apply. H must be from 0 to 23, M from 0 to 1439 and S from 0 to
86399. F must be from 0 to 315537897599999999 and T from -62135596800 to
253402300799, which are the instants from 1 January 0001 to 31 December 9999.
An F or T names a date and a time, and a format other than F or T keeps only the
time of day; F and T keep both.

```rexx
say time('N', '5', 'H') time('N', ' 5 ', 'H') time('N', '+5', 'H') time('N', '5.0', 'H') time('N', '5e0', 'H')
say time('N', '23', 'H') time('N', '61', 'M') time('N', '1439', 'M') time('N', '3661', 'S') time('N', '86399', 'S')
say time('N', '86399999999', 'F') time('N', '86400000000', 'F') time('N', '172800000000', 'F')
say time('N', '0', 'T') time('N', '-1', 'T') time('N', '86400', 'T') time('N', '1480003888', 'T')
say time('F', '86400000001', 'F') time('T', '-1', 'T') time('T', '62135596799999999', 'F')
say time('F', '1', 'T') time('F', '86400', 'T') time('F', '-86400', 'T')
numeric digits 3
say time('N', '63326132161828000', 'F') time('F', '63326132161828000', 'F')
signal on syntax name a
say time('N', '24', 'H')
a: say condition('o')~code
signal on syntax name b
say time('N', '-1', 'H')
b: say condition('o')~code
signal on syntax name c
say time('N', '1440', 'M')
c: say condition('o')~code
signal on syntax name d
say time('N', '86400', 'S')
d: say condition('o')~code
signal on syntax name e
say time('N', '1.5', 'S')
e: say condition('o')~code
signal on syntax name f
say time('N', '315537897600000000', 'F')
f: say condition('o')~code
signal on syntax name g
say time('N', '253402300800', 'T')
g: say condition('o')~code
signal on syntax name h
say time('N', '', 'H')
h: say condition('o')~code
signal on syntax name i
say time('N', 'x', 'T')
i: say condition('o')~code
```

```output
05:00:00 05:00:00 05:00:00 05:00:00 05:00:00
23:00:00 01:01:00 23:59:00 01:01:01 23:59:59
23:59:59 00:00:00 00:00:00
00:00:00 23:59:59 00:00:00 16:11:28
86400000001 -1 -1
62135596801000000 62135683200000000 62135510400000000
08:16:01 63326132161828000
40.19
40.19
40.19
40.19
40.19
40.19
40.19
40.19
40.19
```

r[bif.time.in.truncate]
A conversion answers the earliest time that fits the string. A part of the
time that the string gives and the format of the answer has no place for is
dropped, never rounded: N, C, H, M and S drop the microseconds, C, H and M drop
the seconds as well, and H drops the minutes. A part that the answer needs and
the string lacks is zero.

```rexx
say time('N', '12:00:59.999999', 'L') time('C', '23:59:59.999999', 'L') time('M', '00:00:59.999999', 'L')
say time('S', '12:00:00.999999', 'L') time('H', '12:59:59', 'N') time('L', '12:34:56')
say time('N', '5', 'H') time('N', '90', 'M') time('F', '2', 'H') time('F', '2', 'M')
```

```output
12:00:59 11:59pm 0
43200 12 12:34:56.000000
05:00:00 01:30:00 7200000000 120000000
```

r[bif.time.in.offset]
The informat O reads a whole number of microseconds, in the forms that the
[other counts](r:bif.time.in.number) accept, as an offset from UTC. The
answer is the current time in a zone that far from UTC, as of the
[moment](r:bif.index.dt.stamp) of the clause: with the offset of the local zone,
the current local time. This book does not specify the range of the offset.

```rexx
numeric digits 20
say (time('F', time('O'), 'O') == time('F')) (time('N', time('O'), 'O') == time('N')),
    (time('F', 3600000000, 'O') - time('F', 0, 'O') == 3600000000),
    (time('T', -7200000000, 'O') - time('T', 0, 'O') == -7200)
signal on syntax name a
say time('N', '13:05:09', 'O')
a: say condition('o')~code
```

```output
1 1 1 1
40.19
```

r[bif.time.in.letter]
The informat is one of C, F, H, L, M, N, O, S and T. The reading of the clock,
E and R, cannot be read, and an informat that begins with any other character
raises `SYNTAX 40.904`.

```rexx
signal on syntax name a
say time('N', '5', 'E')
a: say condition('o')~code
signal on syntax name b
say time('N', '5', 'R')
b: say condition('o')~code
signal on syntax name c
say time('N', '5', 'X')
c: say condition('o')~code
signal on syntax name d
say time('N', '5', '')
d: say condition('o')~code
```

```output
40.904
40.904
40.904
40.904
```

## The elapsed-time clock

r[bif.time.elapsed.start]
The options E and R read an elapsed-time clock, which starts at the first call
of E or R in the code that owns the clock. That call answers `0`, and the
clock then counts time from it. Other options do not start the clock and are not
affected by it, and neither is DATE.

```rexx
x = time('N') time('L') time('F') time('O') date('S')
call syssleep 0.3
say time('R')
call syssleep 0.3
say time('E') >= 0.25
```

```output
0
1
```

r[bif.time.elapsed.value]
After the clock has started, E answers the time since the start or the last
reset in seconds with a fraction of six digits: whole seconds with no leading zeros,
a point, and the six digits of the microseconds, `0.300268` and `12.000001`
being examples. The figure is written whatever NUMERIC DIGITS is.

```rexx
numeric digits 3
x = time('E')
call syssleep 0.3
e = time('E')
say datatype(e, 'N') (e >= 0.25) (e < 100) length(e) - pos('.', e) left(e, 2)
call syssleep 1.1
e = time('E')
say (e >= 1.35) (e < 100) length(e) - pos('.', e) pos('.', e) verify(e, '0123456789.')
```

```output
1 1 1 6 0.
1 1 6 2 0
```

r[bif.time.elapsed.reset]
R answers what E would, and also resets the clock: the next call of E or R
counts from the moment of this call.

```rexx
x = time('R')
call syssleep 0.4
a = time('R')
b = time('E')
say (a >= 0.35) (b < a)
call syssleep 0.4
c = time('E')
say (c >= 0.35) (c < 100)
```

```output
1 1
1 1
```

r[bif.time.elapsed.clause]
All the calls of E and R in one [clause](r:bif.index.dt.stamp) answer the same
figure, whichever they are, and a reset by R in the clause takes effect for
the next clause and not for the calls after it in the same clause.

```rexx
x = time('R')
call syssleep 0.3
a = time('E') time('E') time('R') time('E') time('R')
say words(a) (word(a, 1) >= 0.25) (word(a, 1) == word(a, 2)) (word(a, 1) == word(a, 3)),
    (word(a, 1) == word(a, 4)) (word(a, 1) == word(a, 5))
b = time('E')
say b < word(a, 1)
```

```output
5 1 1 1 1 1
1
```

r[bif.time.elapsed.scope]
The clock belongs to the code that runs. A program shares its clock with the
code that INTERPRET runs in it. A routine of the same program that CALL or a
function call runs, a label of the program, begins with a copy of its caller's
clock as it stands at the call, running if the caller's is and not started if
not, and what it does to its copy is not seen by the caller. A `::ROUTINE`, a
method and a message sent with START each begin with no clock, which their first
call of E or R starts.

```rexx
call starter
call syssleep 0.3
say 'caller of a label that started a clock' kind(time('E'))
call syssleep 0.3
say 'label' kind(inside())
say 'main' kind(time('E'))
say 'routine' kind(outside())
say 'method' kind(.o~new~m)
say 'start' kind(.o~new~start('m')~result)
interpret 'y = time("R")'
say 'interpret' kind(time('E'))
exit
starter: return time('R')
inside: v = time('E'); z = time('R'); return v
kind: procedure
parse arg v
if v >= 0.25 then return 'inherited'
return 'fresh'
::routine outside
return time('E')
::class o
::method m
return time('E')
```

```output
caller of a label that started a clock fresh
label inherited
main inherited
routine fresh
method fresh
start fresh
interpret fresh
```

r[bif.time.elapsed.convert]
A call with a `string` and the option E or R raises `SYNTAX 40.29`, whatever
`string` and `informat` are.

```rexx
signal on syntax name a
say time('E', '13:05:09')
a: say condition('o')~code
signal on syntax name b
say time('R', 'junk', 'Q')
b: say condition('o')~code
signal on syntax name c
say time('r', 5, 'H')
c: say condition('o')~code
```

```output
40.29
40.29
40.29
```

## The order of the checks

r[bif.time.order]
A call is checked in this order, and the first failure is the one raised: the
[argument count](r:bif.index.call.count); a missing `string` that an `informat`
needs ([40.5](r:bif.index.dt.convert)); the options E and R with a `string`
([40.29](r:bif.time.elapsed.convert)); the letter of `informat`; the string
itself (`40.19`); and, last, the letter of `option`.

```rexx
signal on syntax name a
say time('X', 'junk', 'Q', 1)
a: say condition('o')~code
signal on syntax name b
say time('E', , 'N')
b: say condition('o')~code
signal on syntax name c
say time('E', 'junk', 'Q')
c: say condition('o')~code
signal on syntax name d
say time('X', 'junk', 'Q')
d: say condition('o')~code
signal on syntax name e
say time('X', 'junk')
e: say condition('o')~code
signal on syntax name f
say time('X', '13:05:09')
f: say condition('o')~code
```

```output
40.4
40.5
40.29
40.904
40.19
40.904
```

> **Note.** The ANSI draft (9.8.5) has neither the options F and T nor the informats F,
> O and T, and it raises `SYNTAX 40.29` for O as well as E and R as `option` with a
> `string`, where ooRexx answers the offset. The reference manual describes an error
> when the system clock is set back below the start of the elapsed-time clock, and
> one for an elapsed time of more than nine digits of seconds; this book specifies
> neither.
