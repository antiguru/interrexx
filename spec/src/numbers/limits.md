---
element: "Number limits"
kind: concept
ansi: "4.2, 7.4, 7.4.10"
rexxref: "Numbers and Arithmetic > Arithmetic Errors"
---

# Number limits

A number has no limit on its digits but a limit on its exponent: every
nonzero number must lie within a fixed range of powers of ten, and every
result that leaves the range is an error and never a wrapped or flushed
value. This page gives the range, what a string outside it is, and the
conditions that `+`, `-`, `*`, `/` and `**` raise when a result leaves it.
The integer division and the remainder have [their own
conditions](r:num.arithmetic.integer). The page also separates this range
from the smaller limit that a whole-number argument has.

## The exponent range

r[num.limits.range]
A nonzero number is within the range under the three conditions of
[the builtin rule](r:bif.index.num.limit), which the arithmetic operators
apply in the same way. The page adds two points about the last written digit:
it counts whether or not it is zero, so the zeros that end a number after the
point count, and a number with a long fraction reaches the lower bound sooner
than its exponent says. Leading zeros of the exponent do not count toward its
value.

```rexx
say datatype('1e999999999', 'N') datatype('1e-999999999', 'N'),
    datatype('4.5e999999998', 'N') datatype('0.0000001e999999999', 'N'),
    datatype('1e0000000000000000000005', 'N')
say datatype('1e1000000000', 'N') datatype('45e999999999', 'N'),
    datatype('1.5e-999999999', 'N') datatype('0.0000001e1000000000', 'N'),
    datatype('1e99999999999999999999', 'N') datatype('1e-1000000000', 'N')
say (1e999999999 + 0) (1e-999999999 + 0) (4.5e999999998 + 0)
say (12345e-999999999 + 0) (0.0000001e999999999 + 0)
```

```output
1 1 1 1 1
0 0 0 0 0 0
1E+999999999 1E-999999999 4.5E+999999998
1.2345E-999999995 1E+999999992
```

r[num.limits.not-number]
A string whose exponent is out of range has [the form of a
number](r:num.index.form) but is not a number, whatever its other
characters. DATATYPE with type `N` answers `0` for it, an
arithmetic operator raises `SYNTAX 41.1`, and a number argument of a builtin
raises `SYNTAX 93.943`, as [the use rule](r:num.index.use) says of every
string that is not a number. A comparison raises no error: it treats such a
string as it treats any other that is not a number, and compares it as text.

```rexx
signal on syntax name a
say '1e1000000000' + 1
a: say condition('o')~code
signal on syntax name b
say -1.5e-999999999
b: say condition('o')~code
signal on syntax name c
say abs('45e999999999')
c: say condition('o')~code
say ('1e1000000000' = '1e1000000000') ('1e1000000000' = 1) (1e1000000000 = 1)
say ('1e1000000000' = '1.0e1000000000') ('1e999999999' = '1.0e999999999')
```

```output
41.1
41.1
93.943
1 0 0
0 1
```

r[num.limits.digits]
The number of digits of a number is not limited, and neither is the number of
zeros before its first nonzero digit. NUMERIC DIGITS plays no part in whether
a string is a number: it rounds a number only when an operation uses it,
and the exponent range applies to the number as it was written, before that
rounding.

```rexx
big = copies('9', 400)
say datatype(big, 'N') length(big) length(big + 0)
say (big + 0)
say (big * big)
lead = copies('0', 400) || '7.50'
tiny = '0.' || copies('0', 400) || '1'
say datatype(lead, 'N') (lead + 0) (tiny + 0)
numeric digits 3
say (big + 0) (lead + 0) (tiny + 0)
say datatype('1.00000e-999999998', 'N') datatype('1.00e-999999997', 'N')
```

```output
1 400 15
1.00000000E+400
1.00000000E+800
1 7.50 1E-401
1.00E+400 7.50 1E-401
0 1
```

## Overflow and underflow

r[num.limits.overflow]
A result of `+`, `-`, `*` or `/`, or of the prefix `+` or `-`, whose first
digit stands at a power of ten above 999999999 raises `SYNTAX 42.901`, and no
value results. The test is made on the result after it is
[rounded](r:bif.index.num.round) to NUMERIC DIGITS, so a result that carries
into a digit above the range when it is rounded raises it too. The operands
may all be in range and the result still out of it. A prefix operator can
only overflow, because its rounding shortens the digits and never moves them
below the range, and a prefix operator on an operand that is out of range
raises `SYNTAX 41.1` for the operand.

```rexx
signal on syntax name a
say 1e999999999 * 10
a: say condition('o')~code
signal on syntax name b
say 9e999999999 + 9e999999999
b: say condition('o')~code
signal on syntax name c
say 1e999999999 / 0.1
c: say condition('o')~code
signal on syntax name d
say 9.99999999e999999999 + 5e999999990
d: say condition('o')~code
numeric digits 3
signal on syntax name e
say 99950e999999995 + 0
e: say condition('o')~code
say (1e999999999 * 9) (1e999999999 + 1e999999999) (99949e999999995 + 0)
say (1.0004e999999999 + 0)
numeric digits 9
say (9.99999999e999999999 + 4e999999990) (9.99999999e999999999 + 5e999999989)
signal on syntax name f
say -9.9999999995e999999999
f: say condition('o')~code
signal on syntax name g
say +9.9999999995e999999999
g: say condition('o')~code
numeric digits 1
signal on syntax name h
say -9.5e999999999
h: say condition('o')~code
say (-9.4e999999999) (+9.4e999999999)
```

```output
42.901
42.901
42.901
42.901
42.901
9E+999999999 2E+999999999 9.99E+999999999
1.00E+999999999
9.99999999E+999999999 9.99999999E+999999999
42.901
42.901
42.901
-9E+999999999 9E+999999999
```

r[num.limits.underflow]
A result of `+`, `-`, `*` or `/` whose last digit stands at a power of ten
below -999999999 raises `SYNTAX 42.902`, and no value results: a result that
is too small is never replaced by zero. The last digit is that of the
result rounded to NUMERIC DIGITS digits, taken before the zeros that end a
division or power result are removed. The test is therefore not made on the
written result: `1 / 9.99999999e999999998` is written `1E-999999999` when it
answers, yet under DIGITS 2 to 12 the rounded value, such as `1.0` for DIGITS
2, ends at -1000000000 and the division raises. A division whose digits run on
can underflow when its first digit is still in range. The test follows the
rounding, so a result whose exact value would end below the range survives
when rounding to few digits removes the digits that fall below it: under
DIGITS 1 the same division answers.

```rexx
signal on syntax name a
say 1e-999999999 / 10
a: say condition('o')~code
signal on syntax name b
say 1e-999999999 * 0.1
b: say condition('o')~code
signal on syntax name c
say 1e-999999991 / 3
c: say condition('o')~code
signal on syntax name d
say 1 / 3e999999999
d: say condition('o')~code
signal on syntax name e
say 1 / 9.99999999e999999998
e: say condition('o')~code
numeric digits 2
signal on syntax name f
say 1 / 9.99999999e999999998
f: say condition('o')~code
numeric digits 9
say (1e-999999998 / 10) (1e-999999990 / 3) (1e-999999999 * 1)
numeric digits 1
say (1.5e-999999998 * 0.1) (1.4e-999999998 * 0.1) (1e-999999998 / 3),
    (1 / 9.99999999e999999998)
```

```output
42.902
42.902
42.902
42.902
42.902
42.902
1E-999999999 3.33333333E-999999991 1E-999999999
2E-999999999 1E-999999999 3E-999999999 1E-999999999
```

r[num.limits.zero]
A result of zero never overflows or underflows, whatever the exponents of
its operands. Such a result is written as [zero](r:num.formats.zero).

```rexx
say (1e999999999 - 1e999999999) (1e-999999999 * 0) (0 / 1e999999999),
    (0 * 1e999999999) (1e-999999999 - 1e-999999999)
say (-1e999999999 + 1e999999999) (0e5 + 0)
```

```output
0 0 0 0 0
0 0
```

r[num.limits.power]
A power raises `SYNTAX 42.1` when [the pre-test of the power
rule](r:num.arithmetic.power.error) refuses it, and that test can refuse a
power whose value is within the range. A power that passes the test raises
`SYNTAX 42.901` or `SYNTAX 42.902`, as the four operators above do, when its
result is out of range, and raises nothing when its result is within the
range, the largest and the smallest included. The result is the value rounded
to NUMERIC DIGITS digits, as [the underflow rule](r:num.limits.underflow)
tests it, so `9.99999999e999999998 ** -1` raises `42.902` under DIGITS 9
although its written result `1E-999999999` is within the range. The code is
that of the result, except that a base raised to a positive exponent that
overflows gives `42.901` even when the exponent is negative and the result is
too small: `11 ** -999999999` raises `42.901`, not `42.902`. Only the result
decides: an intermediate value of the computation that is out of range raises
nothing when the result is in range. The powers of the second example pass the
pre-test and have results within the range, although the base to the positive
exponent has its last digit below the range, and interrexx raises `SYNTAX
42.902` for that intermediate value.

```rexx
signal on syntax name a
say 1e500000000 ** 2
a: say condition('o')~code
signal on syntax name b
say 1e-500000000 ** 2
b: say condition('o')~code
signal on syntax name c
say 1e500000000 ** -2
c: say condition('o')~code
signal on syntax name d
say 99 ** 600000000
d: say condition('o')~code
signal on syntax name e
say 11 ** -999999999
e: say condition('o')~code
signal on syntax name f
say 9.99999999e999999998 ** -1
f: say condition('o')~code
say (1e300000000 ** 3) (1e-300000000 ** 3) (1e999999999 ** 1),
    (1e999999999 ** -1)
```

```output
42.1
42.1
42.1
42.901
42.901
42.902
1E+900000000 1E-900000000 1E+999999999 1E-999999999
```

```rexx
say 1.1e-333333333 ** -3
say 1.01e-333333333 ** -3
say 1.1e-499999999 ** -2
```

```output defect=#95
7.51314801E+999999998
9.70590148E+999999998
8.26446281E+999999997
```

> **Note.** The ANSI draft (4.2 and 7.4.10) requires an exponent limit of at
> least 999999999, as here, and raises `SYNTAX 42.1` for every overflow and
> `42.2` for every underflow, and `41.7` for an operand whose exponent is out
> of range. ooRexx, which this book follows, raises 42.901 and 42.902 for
> overflow and underflow of the four operators above and of a power that
> passes the pre-test, 42.1 for a power that fails it, and 41.1 for an
> operand.

## Whole-number limits

r[num.limits.whole]
The exponent range does not make a number usable as a whole number. A
[whole-number argument](r:conv.whole.digits) is limited to values below
10\*\*18 in magnitude whatever NUMERIC DIGITS is, so a number that is far
inside the exponent range, and is a number for every other purpose, raises
`SYNTAX 40.12` as such an argument when its value is 10\*\*18 or more.

```rexx
say '['substr('abc', '1e17')']' '['substr('abc', '99999999999999999e1')']'
signal on syntax name a
say substr('abc', '1e18')
a: say condition('o')~code
signal on syntax name b
say substr('abc', '1e999999999')
b: say condition('o')~code
say datatype('1e18', 'N') abs('1e999999999')
```

```output
[] []
40.12
40.12
1 1E+999999999
```
