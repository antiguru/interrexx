---
element: MIN
kind: builtin
syntax: "MIN(number [, number]...)"
ansi: "9.4.4"
rexxref: "Functions > Built-in Functions > MIN (Minimum)"
deviations: [44, 53]
---

# MIN

MIN answers the least of a list of numbers, rounded to NUMERIC DIGITS
digits.

## Arguments

r[bif.min.method]
MIN is a builtin that runs as a String method, and its first number is the
receiver; the other numbers become the message's arguments, in their order.
The numbers, their checks and the argument count are those of
[a list of numbers](r:bif.index.num.list): at least one number is required
and there is no most.

```rexx
say min(3, 8, 5) 3~min(8, 5) '3'~min('8', '5') (9)~min
signal on syntax name a
say min()
a: say condition('o')~code
signal on syntax name b
say '3'~min(8, 'five')
b: say condition('o')~code
signal on syntax name c
say 'three'~min(8)
c: say condition('o')~code
```

```output
3 3 3 9
40.3
93.904
93.943
```

r[bif.min.omitted]
Sent as a message to a receiver that is a number, MIN raises `SYNTAX 93.903`
when an argument is omitted and a later argument is passed, whether the
omitted place is the first argument after the receiver or a later one. Omitted
places after the last number passed are no arguments at all. A receiver that
is not a number raises `SYNTAX 93.943` first. (This is the message form; a
function call raises `SYNTAX 40.5` for the same mistake, as [Deviation
53](r:bif.min.deviation.53) says.)

```rexx
say (4)~min(2,,)
signal on syntax name a
say (1)~min(,3)
a: say condition('o')~code
signal on syntax name b
say (5)~min(2,,3)
b: say condition('o')~code
signal on syntax name c
say ('x')~min(,3)
c: say condition('o')~code
```

```output
2
93.903
93.903
93.943
```

## The result

r[bif.min.result]
MIN answers the number of the list that no other number of the list is less
than, picked as the [list rule](r:bif.index.num.pick) says with a later number
replacing the candidate when the candidate is greater than it. The answer is
therefore [rounded](r:bif.index.num.round) to NUMERIC DIGITS digits and
[written](r:bif.index.num.write) in plain or scientific notation, and its sign
is kept. A whole number wider than NUMERIC DIGITS is rounded too (see
[Deviation 44](r:bif.min.deviation.44)).

```rexx
say min(15, 4, 8, 11) min(-9, -2, -6.5) min('+2', '1E1', '09')
say min(' 0.0 ', '-0.000') min(-0.5)
numeric digits 3
say min(123456, 5) min(0.01234, 0.01233) min(-1000, -999.9)
numeric form engineering
say min(123456, 1234567) min(-123456, -5)
```

```output
4 -9 2
0 -0.5
5 0.0123 -1.00E+3
123E+3 -123E+3
```

r[bif.min.rounding]
The numbers are rounded before they are compared, so two numbers that round
to the same value are equal, and MIN answers the earlier of them. This is
seen in the written form, which keeps the decimal places of the number
picked.

```rexx
numeric digits 3
say min(1.00, 1.0004) min(1.0004, 1.00) min(1.0, 1.0004) min(1.0004, 1.0)
say min(1.234, 1.236) min(1.2349, 1.2351)
```

```output
1.00 1.00 1.0 1.00
1.23 1.23
```

r[bif.min.fuzz]
Numbers that NUMERIC FUZZ makes equal are equal to MIN, which then answers the
earlier of them, as the [list rule](r:bif.index.num.pick) says; whole numbers
are no exception (see [Deviation 53](r:bif.min.deviation.53)). With NUMERIC
FUZZ 0, the default, only numbers of the same value are equal.

```rexx
numeric digits 3
numeric fuzz 1
say min(1.04, 1.01) min(1.01, 1.04) min(1.04, 1.09) min(1.09, 1.04)
say min(1.04, 1.01, 1.09) min(1.09, 1.04, 1.01)
numeric fuzz 0
say min(1.04, 1.01) min(1.01, 1.04)
```

```output
1.04 1.01 1.04 1.04
1.04 1.04
1.01 1.01
```

## Deviations

r[bif.min.deviation.44]
MIN rounds a whole number to NUMERIC DIGITS digits as it does any other
number, in the function form and in the method form, with or without
arguments: a whole-number argument wider than NUMERIC DIGITS is rounded before
it is compared, and so is the answer. ooRexx answers some such integers
unrounded, depending on how the number is held.

```rexx
numeric digits 3
say min(12345) 12345~min min(12345, 99999)
```

```output
1.23E+4 1.23E+4 1.23E+4
```

r[bif.min.deviation.53]
MIN applies NUMERIC FUZZ to whole numbers as it does to any other numbers, so
`min(104, 100)` answers 104 under NUMERIC DIGITS 3 and NUMERIC FUZZ 1. A
function call that omits an argument and passes a later one raises
`SYNTAX 40.5`, whether the omitted argument is the first or a later one; the
message form raises `SYNTAX 93.903`, as [its rule](r:bif.min.omitted) says.
ooRexx departs from both. It compares as if NUMERIC FUZZ were 0 when every
argument is a plain integer of at most NUMERIC DIGITS digits: one written as
digits alone, with no sign, blank, leading zero, decimal point or exponent, or
one that arithmetic produced. Its function call raises 93.903 when the
omitted argument is not the first.

```rexx
signal on syntax name a
say min(1, , 3)
a: say condition('o')~code
signal on syntax name b
say min(, 3)
b: say condition('o')~code
signal off syntax
numeric digits 3
numeric fuzz 1
say min(104, 100) min(100, 104) (104)~min(100)
```

```output
40.5
40.5
104 100 104
```
