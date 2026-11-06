---
element: DATE
kind: builtin
syntax: "DATE([option [, [string] [, [informat] [, [osep] [, isep]]]]])"
ansi: "9.8.1"
rexxref: "Functions > Built-in Functions > DATE"
---

# DATE

DATE answers a calendar date. With no `string` it answers the current local
date; with a `string` it reads a date written in one format and writes it in
another. The dates are those of the Gregorian calendar, extended back to
year 1 and forward to year 9999, and a date has no time zone. The current
date changes from day to day, so the examples of this page convert fixed
dates, or check only the form of the current one and how its formats agree
with each other.

## Arguments

r[bif.date.args]
DATE takes up to five arguments and requires none: `option`, `string`,
`informat`, `osep` and `isep`. The argument count is checked by the [builtin
call rules](r:bif.index.call.count): a call with more than five arguments
raises `SYNTAX 40.4`, and a call with only omitted places counts as a call
with no arguments. DATE does not run as a String method: sending the
message `DATE` to a string raises `SYNTAX 97.1`. `option` and `informat` name
formats by [their first letter](r:bif.index.dt.letter), and `osep` and `isep`
are [separators](r:bif.date.sep.char).

```rexx
say date('S', '1 Jan 2000') date(, '1 Jan 2000') date('S', '20000101', 'S', , )
signal on syntax name a
say date('S', '1 Jan 2000', 'N', '', '', 1)
a: say condition('o')~code
signal on syntax name b
say 'S'~date
b: say condition('o')~code
```

```output
20000101 1 Jan 2000 20000101
40.4
97.1
```

r[bif.date.default]
An omitted `option` is N, and so is an omitted `informat`. A call with no
`string` answers the current date, in the format `option` names; a call with
a `string` is a [conversion](r:bif.index.dt.convert).

```rexx
say date(, '29 Feb 2024') date('N', '29 Feb 2024') date(, '20240229', 'S')
say (date() == date('N')) (date(, , , '-') == date('N', , , '-'))
```

```output
29 Feb 2024 29 Feb 2024 29 Feb 2024
1 1
```

## Formats written

r[bif.date.out.text]
Six options write a date as its day, month and year in one string. N
writes the day without a leading zero, the month as the first three letters
of its English name with a capital (`Jan`, `Feb`, `Mar`, `Apr`, `May`, `Jun`,
`Jul`, `Aug`, `Sep`, `Oct`, `Nov`, `Dec`) and the year in four digits,
separated by blanks. E writes `dd/mm/yy`, O writes `yy/mm/dd` and U writes
`mm/dd/yy`; S writes `yyyymmdd` and I writes `yyyy-mm-dd`. A year is in four
digits with leading zeros, or in the last two digits of the year (E, O and
U), and a month or a day other than N's is in two digits with a leading zero.

```rexx
d = '5 Mar 2024'
say date('N', d) '|' date('E', d) '|' date('O', d) '|' date('U', d) '|' date('S', d) '|' date('I', d)
d = '0007-03-05'
say date('N', d, 'I') '|' date('E', d, 'I') '|' date('S', d, 'I') '|' date('I', d, 'I')
d = '31 Dec 9999'
say date('N', d) '|' date('E', d) '|' date('S', d) '|' date('I', d)
```

```output
5 Mar 2024 | 05/03/24 | 24/03/05 | 03/05/24 | 20240305 | 2024-03-05
5 Mar 0007 | 05/03/07 | 00070305 | 0007-03-05
31 Dec 9999 | 31/12/99 | 99991231 | 9999-12-31
```

r[bif.date.out.names]
Three options write a name. M writes the full English name of the month, W
the full English name of the day of the week, and L writes the day without a
leading zero, the full name of the month and the year in four digits,
separated by blanks. Both names begin with a capital letter.

```rexx
list = ''
do m = 1 to 12
  list = list date('M', right(m, 2, 0) || '/15/24', 'U')
end
say strip(list)
list = ''
do d = 1 to 7
  list = list date('W', d 'Jan 2024')
end
say strip(list)
say date('L', '5 Mar 2024') '|' date('L', '31 Dec 9999') '|' date('L', '1 Jan 0001')
```

```output
January February March April May June July August September October November December
Monday Tuesday Wednesday Thursday Friday Saturday Sunday
5 March 2024 | 31 December 9999 | 1 January 0001
```

r[bif.date.out.counts]
Four options write a whole number. B writes the number of days from 1 January
0001 up to the day, so that 1 January 0001 is 0, and the remainder of that
number divided by 7 is 0 for a Monday and 6 for a Sunday. D writes the number
of the day in its year, from 1. F writes the number of microseconds from the
start of 1 January 0001 to the start of the day, and T the number of seconds
from the start of 1 January 1970, negative before it. A number is written
with no sign unless it is negative, no leading zero, point or exponent, however many
digits it has: the setting of NUMERIC DIGITS plays no part. Arithmetic on
such a number is subject to NUMERIC DIGITS like any other.

```rexx
say date('B', '1 Jan 0001') date('B', '1 Jan 1970') date('B', '31 Dec 9999')
say date('B', '2 Jan 2000') // 7 date('B', '3 Jan 2000') // 7 date('B', '9 Jan 2000') // 7
say date('D', '1 Jan 2001') date('D', '1 Mar 2001') date('D', '31 Dec 2001')
say date('T', '1 Jan 1970') date('T', '2 Jan 1970') date('T', '31 Dec 1969') date('T', '1 Jan 0001')
numeric digits 3
say date('F', '1 Jan 0001') date('F', '2 Jan 0001') date('F', '31 Dec 9999') date('T', '31 Dec 9999')
say date('F', '2 Jan 0001') + 0
```

```output
0 719162 3652058
6 0 6
1 60 365
0 86400 -86400 -62135596800
0 86400000000 315537811200000000 253402214400
8.64E+10
```

r[bif.date.calendar]
A date is valid for the year 1 to the year 9999. A year is a leap year, with a
29 February, when it is divisible by 4 but not by 100, or when it is
divisible by 400. A string names a valid date only when its day exists in its
month of that year, and the days of the year number 366 in a leap year and 365 in another.

```rexx
say date('S', '29 Feb 2000') date('S', '29 Feb 1904') date('S', '29 Feb 2024')
say date('D', '31 Dec 2000') date('D', '31 Dec 1900') date('D', '31 Dec 2100')
signal on syntax name a
say date('S', '29 Feb 1900')
a: say condition('o')~code
signal on syntax name b
say date('S', '29 Feb 2100')
b: say condition('o')~code
signal on syntax name c
say date('S', '31 Apr 2000')
c: say condition('o')~code
```

```output
20000229 19040229 20240229
366 365 365
40.19
40.19
40.19
```

## The current date

r[bif.date.current]
A call with no `string` answers the current date in the local time zone,
written in the format `option` names, as of the [moment](r:bif.index.dt.stamp)
the clause reads the clock. Every format of the current date is the same date:
converting the current date in S form to any format other than F and T gives
the answer that format gives. F and T include the time of day: each is the
figure that the conversion of the same date gives, which is for the start of
the day, plus the time since then, a number of microseconds (F) or seconds (T)
that is not negative and is less than a day.

```rexx
numeric digits 20
say (date('B') == date('B', date('S'), 'S')) (date('D') == date('D', date('S'), 'S')),
    (date('E') == date('E', date('S'), 'S')) (date('I') == date('I', date('S'), 'S')),
    (date('L') == date('L', date('S'), 'S')) (date('M') == date('M', date('S'), 'S')),
    (date('N') == date('N', date('S'), 'S')) (date('O') == date('O', date('S'), 'S')),
    (date('U') == date('U', date('S'), 'S')) (date('W') == date('W', date('S'), 'S'))
f = (date('F') - date('F', date('S'), 'S') >= 0) & (date('F') - date('F', date('S'), 'S') < 86400000000)
t = (date('T') - date('T', date('S'), 'S') >= 0) & (date('T') - date('T', date('S'), 'S') < 86400)
say f t
say length(date('S')) length(date('I')) length(date('E')) datatype(date('B'), 'W') datatype(date('D'), 'W'),
    words(date('N')) words(date('L'))
```

```output
1 1 1 1 1 1 1 1 1 1
1 1
8 10 8 1 1 3 3
```

## Reading a date

r[bif.date.in.text]
The informats N, S, E, O, U and I read the formats that the options of the same
letters [write](r:bif.date.out.text), and they accept nothing else: no blank or
other character before, after or inside the date, no other number of digits in a
field, and a month name only as the three letters that N writes, with the same
capitals. The day of N may have one digit or two, and is read with or without a
leading zero; every other field has the digits that its format writes.

```rexx
say date('S', '1 Jan 2000', 'N') date('S', '01 Jan 2000', 'N') date('S', '20000101', 'S')
say date('S', '01/02/03', 'E') date('S', '03/02/01', 'O') date('S', '02/01/03', 'U'),
    date('S', '2003-02-01', 'I')
signal on syntax name a
say date('S', '1 jan 2000')
a: say condition('o')~code
signal on syntax name b
say date('S', '1/2/03', 'E')
b: say condition('o')~code
signal on syntax name c
say date('S', ' 20000101', 'S')
c: say condition('o')~code
signal on syntax name d
say date('S', '1 Jan 02000', 'N')
d: say condition('o')~code
signal on syntax name e
say date('S', '2003-2-1', 'I')
e: say condition('o')~code
signal on syntax name f
say date('S', '1 January 2000')
f: say condition('o')~code
```

```output
20000101 20000101 20000101
20030201 20030201 20030201 20030201
40.19
40.19
40.19
40.19
40.19
40.19
```

r[bif.date.in.number]
The informats B, F and T read a count: the number of days, of microseconds or
of seconds that B, F and T [write](r:bif.date.out.counts). The string may
have any form of a number, such as `5`, ` 5 `, `+5`, `5.0` and `5e0`, and NUMERIC
DIGITS does not apply. A string whose value is a whole number is read as
that number, however many leading or trailing zeros it has, and the number
must be in the range given below. A string whose value is not a
whole number raises `SYNTAX 40.19` when it has at most nine significant digits
for B, or at most 18 for F and T, its significant digits being those from the
first digit that is not 0 to the last digit that is not 0. This book does not
specify whether a string with more significant digits, whose value is not a
whole number, is accepted, nor what it then reads as. B must be from 0 to
3652058, F from 0 to 315537897599999999, and T from -62135596800 to
253402300799, which are 1 January 0001 to 31 December 9999. A count of F or T
that falls inside a day reads as that day. The informat D is read in the same way, [as the
day of the current year](r:bif.date.in.day).

```rexx
say date('S', '5', 'B') date('S', ' 5 ', 'B') date('S', '+5', 'B') date('S', '5.0', 'B'),
    date('S', '5e0', 'B') date('S', '000005', 'B')
say date('S', '0', 'B') date('S', '3652058', 'B')
say date('S', '86399999999', 'F') date('S', '86400000000', 'F') date('S', '63845193600000000', 'F')
say date('S', '-1', 'T') date('S', '86399', 'T') date('S', '86400', 'T') date('S', '1709643909', 'T')
say date('S', '5.' || copies('0', 30), 'B') date('S', copies('0', 30) || '5', 'B') date('S', '5000e-3', 'B')
numeric digits 3
say date('S', '63845193600000000', 'F') date('S', '1709643909', 'T')
signal on syntax name h
say date('S', '1.99999999', 'B')
h: say condition('o')~code
signal on syntax name i
say date('S', '1.5000000000000', 'B')
i: say condition('o')~code
signal on syntax name j
say date('S', '1.00000000000000001', 'F')
j: say condition('o')~code
signal on syntax name k
say date('S', '1709643909.00000001', 'T')
k: say condition('o')~code
signal on syntax name a
say date('S', '3652059', 'B')
a: say condition('o')~code
signal on syntax name b
say date('S', '-1', 'B')
b: say condition('o')~code
signal on syntax name c
say date('S', '1.5', 'B')
c: say condition('o')~code
signal on syntax name d
say date('S', '315537897600000000', 'F')
d: say condition('o')~code
signal on syntax name e
say date('S', '253402300800', 'T')
e: say condition('o')~code
signal on syntax name f
say date('S', '', 'B')
f: say condition('o')~code
signal on syntax name g
say date('S', 'x', 'T')
g: say condition('o')~code
```

```output
00010106 00010106 00010106 00010106 00010106 00010106
00010101 99991231
00010101 00010102 20240305
19691231 19700101 19700102 20240305
00010106 00010106 00010106
20240305 20240305
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

r[bif.date.in.day]
The informat D reads a number from 1 to the number of days of the current year
(366 in a leap year, 365 in another), read as the informat B
[reads](r:bif.date.in.number) its count, nine significant digits included, as
the day of that year. The year is that of the [moment](r:bif.index.dt.stamp)
of the clause. A number outside that range raises `SYNTAX 40.19`.

```rexx
say (date('S', '1', 'D') == left(date('S'), 4) || '0101'),
    (date('S', '32', 'D') == left(date('S'), 4) || '0201'),
    (date('S', '365', 'D') == left(date('S'), 4) || '1231' | date('S', '365', 'D') == left(date('S'), 4) || '1230')
say (date('N', ' 1 ', 'D') == date('N', '1', 'D')) (date('E', '1.0', 'D') == date('E', '1', 'D'))
signal on syntax name a
say date('S', '0', 'D')
a: say condition('o')~code
signal on syntax name b
say date('S', '367', 'D')
b: say condition('o')~code
signal on syntax name c
say date('S', '1 Jan', 'D')
c: say condition('o')~code
signal on syntax name d
say date('S', '1.99999999', 'D')
d: say condition('o')~code
```

```output
1 1 1
1 1
40.19
40.19
40.19
40.19
```

r[bif.date.in.century]
A date read with E, O or U has a year of two digits. Its year is the one that
ends in those digits and falls in the 100 years that start 50 years before
the current year: the current year less 50 is the earliest, and the current
year plus 49 the latest. The current year is that of the [moment](r:bif.index.dt.stamp)
of the clause. Every other informat gives the year in full.

```rexx
ok = 1
do n = 0 to 99
  yy = right(n, 2, 0)
  r = left(date('S', yy || '/01/01', 'O'), 4) left(date('S', '01/01/' || yy, 'E'), 4),
      left(date('S', '01/01/' || yy, 'U'), 4) left(date('S'), 4)
  parse var r o e u y
  ok = ok & (o >= y - 50) & (o <= y + 49) & (right(o, 2) == yy) & (e == o) & (u == o)
end
say ok
```

```output
1
```

r[bif.date.in.invalid]
A string raises `SYNTAX 40.19` when it does not have the form of its
informat, and when it has that form but names a day that does not exist: a
month 00 or above 12, a day 00 or past the end of its month, or the year 0000.
A year from 0001 to 9999 is valid.

```rexx
say date('S', '1 Jan 0001') date('S', '31 Dec 9999') date('S', '0001-01-01', 'I')
signal on syntax name a
say date('S', '1 Jan 0000')
a: say condition('o')~code
signal on syntax name b
say date('S', '2000-13-01', 'I')
b: say condition('o')~code
signal on syntax name c
say date('S', '2000-00-10', 'I')
c: say condition('o')~code
signal on syntax name d
say date('S', '00 Jan 2000')
d: say condition('o')~code
signal on syntax name e
say date('S', '20000230', 'S')
e: say condition('o')~code
signal on syntax name f
say date('S', '31 Feb 2000')
f: say condition('o')~code
```

```output
00010101 99991231 00010101
40.19
40.19
40.19
40.19
40.19
40.19
```

r[bif.date.in.letter]
The informat is one of B, D, E, F, I, N, O, S, T and U. The formats that
write a name, L, M and W, cannot be read, and an informat that begins with
any other character raises `SYNTAX 40.904`.

```rexx
signal on syntax name a
say date('S', 'January', 'M')
a: say condition('o')~code
signal on syntax name b
say date('S', 'Monday', 'W')
b: say condition('o')~code
signal on syntax name c
say date('S', '1 January 2000', 'L')
c: say condition('o')~code
signal on syntax name d
say date('S', '1 Jan 2000', 'X')
d: say condition('o')~code
```

```output
40.904
40.904
40.904
40.904
```

## Separators

r[bif.date.sep.char]
A separator, `osep` or `isep`, is the null string or a single character
that is neither one of the 52 ASCII letters nor one of the ten digits. A blank
is a separator, and so are `$`, `'00'x` and `'E9'x`. Any other value raises
`SYNTAX 40.43`.

```rexx
say date('S', '5 Mar 2024', 'N', '$') date('S', '5 Mar 2024', 'N', ' ') date('S', '5 Mar 2024', 'N', '')
say date('S', '5$Mar$2024', 'N', , '$') date('S', '5Mar2024', 'N', , '')
say date('S', '5' || '00'x || 'Mar' || '00'x || '2024', 'N', , '00'x),
    date('S', '5' || 'E9'x || 'Mar' || 'E9'x || '2024', 'N', , 'E9'x) c2x(date('S', '5 Mar 2024', 'N', 'E9'x))
signal on syntax name a
say date('S', '5 Mar 2024', 'N', 'x')
a: say condition('o')~code
signal on syntax name b
say date('S', '5 Mar 2024', 'N', '5')
b: say condition('o')~code
signal on syntax name c
say date('S', '5 Mar 2024', 'N', '--')
c: say condition('o')~code
signal on syntax name d
say date('S', '5xMar2024', 'N', , 'x')
d: say condition('o')~code
signal on syntax name e
say date('S', '5--Mar--2024', 'N', , '--')
e: say condition('o')~code
signal on syntax name f
say date('S', '5 Mar 2024', 'N', 'C3A9'x)
f: say condition('o')~code
```

```output
2024$03$05 2024 03 05 20240305
20240305 20240305
20240305 20240305 32303234E93033E93035
40.43
40.43
40.43
40.43
40.43
40.43
```

r[bif.date.sep.out]
The output separator `osep` replaces the separator that the format `option` has
by default, which is a blank for N, a `/` for E, O and U, a `-` for I and none for S. It
applies to E, I, N, O, S and U, and not to the other options. The null string
gives a format with no separator.

```rexx
d = '5 Mar 2024'
say date('N', d, 'N', '-') date('E', d, 'N', '.') date('O', d, 'N', '') date('U', d, 'N', ' ')
say date('S', d, 'N', '/') date('I', d, 'N', '') date('I', d, 'N', ',')
say date('N', d, 'N', ' ') date('E', d, 'N', '/') date('I', d, 'N', '-') date('S', d, 'N', '')
signal on syntax name a
say date('B', d, 'N', '-')
a: say condition('o')~code
signal on syntax name b
say date('M', d, 'N', '')
b: say condition('o')~code
signal on syntax name c
say date('W', d, 'N', '-')
c: say condition('o')~code
signal on syntax name d
say date('L', d, 'N', ' ')
d: say condition('o')~code
```

```output
5-Mar-2024 05.03.24 240305 03 05 24
2024/03/05 20240305 2024,03,05
5 Mar 2024 05/03/24 2024-03-05 20240305
40.44
40.44
40.44
40.44
```

r[bif.date.sep.in]
The input separator `isep` says which separator the `string` uses, in place
of the default separator of its format: it applies to N, E, O, U, S and I.
The string must then have exactly the form of its format with that
separator between the fields, or none for the null string, and a string that
does not is rejected with `SYNTAX 40.44` and not `40.19`, whatever is wrong
with it. With the null string, N reads the day as one or two digits.

```rexx
say date('S', '5-Mar-2024', 'N', , '-') date('S', '5Mar2024', 'N', , '') date('S', '15Mar2024', 'N', , '')
say date('S', '05.03.24', 'E', , '.') date('S', '050324', 'E', , '') date('S', '2024 03 05', 'I', , ' ')
say date('S', '20240305', 'I', , '') date('S', '2024/03/05', 'S', , '/') date('S', '05 Mar 2024', 'N', , ' ')
signal on syntax name a
say date('S', '5 Mar 2024', 'N', , '-')
a: say condition('o')~code
signal on syntax name b
say date('S', '30-Feb-2024', 'N', , '-')
b: say condition('o')~code
signal on syntax name c
say date('S', '5-Mar-2024', 'N', , '')
c: say condition('o')~code
signal on syntax name d
say date('S', '24035', 'E', , '')
d: say condition('o')~code
signal on syntax name e
say date('S', '5/3/24', 'E', , '/')
e: say condition('o')~code
```

```output
20240305 20240305 20240315
20240305 20240305 20240305
20240305 20240305 20240305
40.44
40.44
40.44
40.44
40.44
```

r[bif.date.sep.in.informat]
An `isep` with an `informat` that separators do not apply to, B, D, F, L, M, T
or W, raises `SYNTAX 40.44`, as does an `osep` with an `option` of the same
seven letters, or with a first character that names no format at all. An `isep` with an `informat` whose first character names no
format is the [letter error](r:bif.date.in.letter), `SYNTAX 40.904`.

```rexx
signal on syntax name a
say date('S', '5', 'B', , '-')
a: say condition('o')~code
signal on syntax name b
say date('S', '5', 'D', , '')
b: say condition('o')~code
signal on syntax name c
say date('S', '5', 'F', , ' ')
c: say condition('o')~code
signal on syntax name d
say date('S', '5', 'T', , '/')
d: say condition('o')~code
signal on syntax name e
say date('X', '5 Mar 2024', 'N', '-')
e: say condition('o')~code
signal on syntax name f
say date('S', '5-Mar-2024', 'L', , '-')
f: say condition('o')~code
signal on syntax name g
say date('S', '5-Mar-2024', 'W', , '-')
g: say condition('o')~code
signal on syntax name h
say date('S', '5-Mar-2024', 'Q', , '-')
h: say condition('o')~code
```

```output
40.44
40.44
40.44
40.44
40.44
40.44
40.44
40.904
```

## The order of the checks

r[bif.date.order]
A call is checked in this order, and the first failure is the one raised: the
[argument count](r:bif.index.call.count); a missing `string` that an `informat` or `isep`
needs ([40.5](r:bif.index.dt.convert)); an `osep` with an `option` that
[does not apply it](r:bif.date.sep.in.informat); the form of `osep`; an `isep` with an
`informat` that does not apply it; the form of `isep` ([40.43](r:bif.date.sep.char));
the letter of `informat`; the string itself (`40.19`, or `40.44` with an `isep`); and,
last, the letter of `option`. So an `option` that is not a letter is raised only when
the rest of the call is right.

```rexx
signal on syntax name a
say date('B', 'junk', 'Q', 'ab', '-')
a: say condition('o')~code
signal on syntax name b
say date('S', 'junk', 'Q', 'ab', '-')
b: say condition('o')~code
signal on syntax name c
say date('S', '5', 'B', 'ab', '-')
c: say condition('o')~code
signal on syntax name d
say date('S', '5', 'B', , 'ab')
d: say condition('o')~code
signal on syntax name e
say date('S', 'junk', 'Q', , 'ab')
e: say condition('o')~code
signal on syntax name f
say date('S', 'junk', 'Q')
f: say condition('o')~code
signal on syntax name g
say date('X', 'junk')
g: say condition('o')~code
signal on syntax name h
say date('X', '1 Jan 2000')
h: say condition('o')~code
signal on syntax name i
say date('X', , 'Q')
i: say condition('o')~code
signal on syntax name j
say date('S', 'junk', 'N', , '-')
j: say condition('o')~code
```

```output
40.44
40.43
40.43
40.44
40.43
40.904
40.19
40.904
40.5
40.44
```

> **Note.** The ANSI draft (9.8.1) has neither separators nor the options F, I, L and T.
> The reference manual does not give the order of the checks; this rule matches ooRexx.
