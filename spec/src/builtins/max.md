---
element: MAX
kind: builtin
syntax: "MAX(number [, number]...)"
ansi: "9.4.3"
rexxref: "Functions > Built-in Functions > MAX (Maximum)"
deviations: [44, 53]
---

# MAX

MAX answers the greatest of a list of numbers, rounded to NUMERIC DIGITS
digits.

## Arguments

r[bif.max.method]
MAX is a builtin that runs as a String method, and its first number is the
receiver; the other numbers become the message's arguments, in their order.
The numbers, their checks and the argument count are those of
[a list of numbers](r:bif.index.num.list): at least one number is required
and there is no most.

```rexx
say max(3, 8, 5) 3~max(8, 5) '3'~max('8', '5') (3)~max
signal on syntax name a
say max()
a: say condition('o')~code
signal on syntax name b
say '3'~max(8, 'five')
b: say condition('o')~code
signal on syntax name c
say 'three'~max(8)
c: say condition('o')~code
```

```output
8 8 8 3
40.3
93.904
93.943
```

r[bif.max.omitted]
Sent as a message to a receiver that is a number, MAX raises `SYNTAX 93.903`
when an argument is omitted and a later argument is passed, whether the
omitted place is the first argument after the receiver or a later one. Omitted
places after the last number passed are no arguments at all. A receiver that
is not a number raises `SYNTAX 93.943` first. (This is the message form; a
function call raises `SYNTAX 40.5` for the same mistake, as [Deviation
53](r:bif.max.deviation.53) says.)

```rexx
say (4)~max(2,,)
signal on syntax name a
say (1)~max(,3)
a: say condition('o')~code
signal on syntax name b
say (1)~max(2,,3)
b: say condition('o')~code
signal on syntax name c
say ('x')~max(,3)
c: say condition('o')~code
```

```output
4
93.903
93.903
93.943
```

## The result

r[bif.max.result]
MAX answers the number of the list that no other number of the list is greater
than, picked as the [list rule](r:bif.index.num.pick) says with a later number
replacing the candidate when the candidate is less than it. The answer is
therefore [rounded](r:bif.index.num.round) to NUMERIC DIGITS digits and
[written](r:bif.index.num.write) in plain or scientific notation, and its sign
is kept. A whole number wider than NUMERIC DIGITS is rounded too (see
[Deviation 44](r:bif.max.deviation.44)).

```rexx
say max(15, 4, 8, 11) max(-9, -2, -6.5) max('+2', '1E1', '09')
say max(' 0.0 ', '-0.000') max(-0.5)
numeric digits 3
say max(123456, 5) max(0.01234, 0.01233) max(1000, 999.9)
numeric form engineering
say max(123456, 5) max(-123456, -1234567)
```

```output
15 -2 10
0 -0.5
1.23E+5 0.0123 1.00E+3
123E+3 -123E+3
```

r[bif.max.rounding]
The numbers are rounded before they are compared, so two numbers that round
to the same value are equal, and MAX answers the earlier of them. This is
seen in the written form, which keeps the decimal places of the number
picked.

```rexx
numeric digits 3
say max(1.00, 1.0004) max(1.0004, 1.00) max(1.0, 1.0004) max(1.0004, 1.0)
say max(1.234, 1.236) max(1.2349, 1.2351)
```

```output
1.00 1.00 1.0 1.00
1.24 1.24
```

r[bif.max.fuzz]
Numbers that NUMERIC FUZZ makes equal are equal to MAX, which then answers the
earlier of them, as the [list rule](r:bif.index.num.pick) says; whole numbers
are no exception (see [Deviation 53](r:bif.max.deviation.53)). With NUMERIC
FUZZ 0, the default, only numbers of the same value are equal.

```rexx
numeric digits 3
numeric fuzz 1
say max(1.04, 1.01) max(1.01, 1.04) max(1.04, 1.09) max(1.09, 1.04)
say max(1.04, 1.01, 1.09) max(1.09, 1.04, 1.01)
numeric fuzz 0
say max(1.04, 1.01) max(1.01, 1.04)
```

```output
1.04 1.01 1.09 1.09
1.09 1.09
1.04 1.04
```

## Deviations

r[bif.max.deviation.44]
MAX rounds a whole number to NUMERIC DIGITS digits as it does any other
number, in the function form and in the method form, with or without
arguments: a whole-number argument wider than NUMERIC DIGITS is rounded before
it is compared, and so is the answer. ooRexx answers some such integers
unrounded, depending on how the number is held.

```rexx
numeric digits 3
say max(1, 12345) max(12345, 1) max(5, 12345, 3)
```

```output
1.23E+4 1.23E+4 1.23E+4
```

r[bif.max.deviation.53]
MAX applies NUMERIC FUZZ to whole numbers as it does to any other numbers, so
`max(100, 104)` answers 100 under NUMERIC DIGITS 3 and NUMERIC FUZZ 1. A
function call that omits an argument and passes a later one raises
`SYNTAX 40.5`, whether the omitted argument is the first or a later one; the
message form raises `SYNTAX 93.903`, as [its rule](r:bif.max.omitted) says.
ooRexx departs from both: it compares two whole numbers written without a
decimal point as if NUMERIC FUZZ were 0, and its function call raises 93.903
when the omitted argument is not the first.

```rexx
signal on syntax name a
say max(1, , 3)
a: say condition('o')~code
signal on syntax name b
say max(, 3)
b: say condition('o')~code
signal off syntax
numeric digits 3
numeric fuzz 1
say max(100, 104) max(104, 100) (100)~max(104)
```

```output
40.5
40.5
100 104 100
```
