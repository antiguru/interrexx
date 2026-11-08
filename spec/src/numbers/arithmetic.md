---
element: "Arithmetic"
kind: concept
ansi: "7.4.2, 7.4.3, 7.4.4, 7.4.5, 7.4.10"
rexxref: "Numbers and Arithmetic > Arithmetic"
deviations: [53, 102]
---

# Arithmetic

The arithmetic operators are the infix `+`, `-`, `*`, `/`, `%`, `//` and `**`,
and the prefix `+` and `-`. Each reads its operands as numbers, works to the
precision that NUMERIC DIGITS sets, and answers a new number. This page says
how many digits an operation looks at in its operands, how it rounds, what
each operator answers, and which errors an operation raises. How an answer is
written is the business of [the formats page](formats.md), the size limits of
[the limits page](limits.md), and the settings of [the settings
page](settings.md). Where the operators stand in an expression is left to [the
expressions chapter](../expressions/arithmetic.md).

## Operands and precision

r[num.arithmetic.operand]
Each operand of an arithmetic operator must be a [number](r:num.index.form),
and a string that is not one raises `SYNTAX 41.1`, as [the use
rule](r:num.index.use) says. Both operands, and the operand of a prefix
operator, are checked before the operator does anything else, so a bad operand
wins over a zero divisor or any other error that the operator would otherwise
find. The right operand of `**` is the exception: it is the exponent and is
checked by [its own rule](r:num.arithmetic.exponent). A string whose exponent
is out of [range](r:num.limits.not-number) is not a number either.

```rexx
signal on syntax name a
say 'ten' / 0
a: say condition('o')~code
signal on syntax name b
say 10 % 'x'
b: say condition('o')~code
signal on syntax name c
say -'1 2'
c: say condition('o')~code
signal on syntax name d
say 1 + ''
d: say condition('o')~code
```

```output
41.1
41.1
41.1
41.1
```

r[num.arithmetic.digits]
An operation works at NUMERIC DIGITS significant digits and rounds its result
to that many, as [the rounding rule](r:bif.index.num.round) of the builtins
says and as [the result rule](r:num.formats.result) applies to every operator.
An operand is not rounded before the operation: it takes part with the digits
it has, apart from the cut that the next two rules describe for the digits
beyond DIGITS plus one. Under NUMERIC DIGITS 1 the product of 2 and 1.5 is
therefore 3, where rounding 1.5 to 2 first would give 4, and rounding happens
once, on the result.

```rexx
numeric digits 1
say (2 * 1.5) (1.5 + 1.5) (6 / 4) (0.4 + 0.4)
numeric digits 3
say (1 / 3) (2 / 3) (100 / 3) (1000 / 3) (10000 / 3) (2.5 * 2.5)
numeric digits 12
say (1 / 3) (2 / 3) (1 / 7)
```

```output
3 3 2 0.8
0.333 0.667 33.3 333 3.33E+3 6.25
0.333333333333 0.666666666667 0.142857142857
```

r[num.arithmetic.cut]
Before `*`, `/`, `%` or `//` uses an operand that has more than DIGITS plus
one significant digits, it cuts the operand to its first DIGITS plus one
digits, counted from the first nonzero digit. The cut drops the later digits
and does not round them. The digit kept beyond DIGITS takes part in the
operation, so it shapes the digit on which the result is rounded. The two
products and the two quotients below therefore differ from the exact ones,
which would round to 4, 4, 1.0 and 0.32.

```rexx
numeric digits 1
say (1.19 * 3) (1.19 * 3.19)
numeric digits 2
say (1.499 / 1.5) (2.999 / 9.5)
```

```output
3 3
0.99 0.31
```

r[num.arithmetic.addsub]
The result of `+` and `-` is rounded to DIGITS digits counted from the first
digit of the operand whose first digit stands higher, and not from the first
digit of the result. Let that higher digit stand at power of ten `H`, taking
no account of an operand with the value zero. Both operands are first cut
after the digit at `H - DIGITS`, which is one digit more than the rounding
keeps. Their exact sum or difference is then rounded once, half up in
magnitude, at the coarser of `H - DIGITS + 1` and the DIGITS significant
digits of that sum. So two close numbers can cancel down to zero, and a sum
that carries into a new leading digit keeps one digit less after the point.

```rexx
numeric digits 3
say (1234 - 1233) (1234 - 1200) (12.34 - 12.3) (1.0006 - 1)
say (1234 + 1) (1234 + 0) (0 + 1234) (1.234 - 0)
say (999 + 1) (995 + 9.5) (9.99 + 0.006) (99.5 + 0.5)
a = -1234
say (a + 1233) (a - 1) (1233 + a) (-1 + 1)
numeric digits 4
say (8909.31002 - 800.8374) (8909.31002 + 0) (800.8374 - 8909.31002)
```

```output
0 30 0 0
1.24E+3 1.23E+3 1.23E+3 1.23
1.00E+3 1.00E+3 10.0 100
0 -1.23E+3 0 0
8109 8909 -8109
```

r[num.arithmetic.prefix]
The prefix `+` and `-` round their operand to DIGITS significant digits as
`0 + x` and `0 - x` would, and `-` also flips the sign. The operand is a
number like any other, so a prefix applied to zero answers `0` without a
sign. The sign of a number in a program is a prefix operator applied to the
number without the sign, so a literal that is wider than DIGITS is rounded by
its own sign before the operator beside it sees it.

```rexx
numeric digits 3
say (-1.2345) (+2.4999) (-0.00012345) (+' 5.0 ') (-0) (-0.0)
numeric digits 1
x = '-1.5'
say (-1.5 + 2) (x + 2) (-1.5) x (+1.5) (+15)
```

```output
-1.23 2.50 -0.000123 5.0 0 0
0 1 -2 -1.5 2 2E+1
```

## Division

r[num.arithmetic.divide]
`/` answers the quotient of its operands, which are cut as the [cut
rule](r:num.arithmetic.cut) says, rounded to DIGITS digits. A dividend of zero
answers `0`, and the quotient has the [trailing zeros](r:num.formats.trailing)
removed. The quotient need not be whole, and a quotient with a repeating
expansion is the first DIGITS digits of it, rounded.

```rexx
say (1 / 3) (2 / 3) (10 / 4) (1 / 8) (6.0 / 3) (0 / 5) (-9 / 4) (7 / -2)
numeric digits 4
say (1 / 3) (2 / 3) (22 / 7) (1 / 80000) (12345 / 100) (1 / 800000000)
```

```output
0.333333333 0.666666667 2.5 0.125 2 0 -2.25 -3.5
0.3333 0.6667 3.143 0.0000125 123.5 1.25E-9
```

r[num.arithmetic.zero]
A zero divisor raises `SYNTAX 42.3` for `/`, `%` and `//`, whatever the
dividend is, zero included. A divisor is zero when it is a number of value
zero, whatever its written form. The [operand
check](r:num.arithmetic.operand) comes first, as the last example shows.

```rexx
signal on syntax name a
say 1 / 0
a: say condition('o')~code
signal on syntax name b
say 0 / 0
b: say condition('o')~code
signal on syntax name c
say 7 % 0.00
c: say condition('o')~code
signal on syntax name d
say 7 // '-0e5'
d: say condition('o')~code
signal on syntax name e
say 7 / 'x'
e: say condition('o')~code
```

```output
42.3
42.3
42.3
42.3
41.1
```

r[num.arithmetic.integer]
`%` answers the integer part of the quotient, its magnitude rounded down, with
the sign of the quotient, and `//` answers what remains of the dividend after
that many divisors are taken from it, which has the sign of the dividend and
is rounded to DIGITS digits. Both cut their operands as the [cut
rule](r:num.arithmetic.cut) says before they divide. The integer part must
fit in DIGITS digits, or `%` raises `SYNTAX 26.11` and `//` raises `SYNTAX
26.12`, so a remainder is refused when the quotient it belongs to is too wide.
A result of zero is `0` with no sign.

```rexx
say (7 % 2) (-7 % 2) (7 % -2) (7 // 2) (-7 // 2) (7 // -2) (-7 // -2)
say (7.5 // 2) (-7.5 % 2) (0.5 % 1) (-0.5 // 1) (-0.5 % 1) (6 // 3) (6 % 3)
numeric digits 3
say (999 % 1) (1234 // 1000) (1234 % 1000) (12.3456789 // 1) (9.9999 // 10)
signal on syntax name a
say 1000 % 1
a: say condition('o')~code
signal on syntax name b
say 1000 // 1
b: say condition('o')~code
signal on syntax name c
say 12.5 // 0.001
c: say condition('o')~code
```

```output
3 -3 -3 1 -1 1 -1
1.5 -3 0 -0.5 0 0 2
999 234 1 0.34 10.0
26.11
26.12
26.12
```

## Powers

r[num.arithmetic.exponent]
The right operand of `**` must be a whole number of at most DIGITS digits,
after it is rounded to DIGITS digits as `exponent + 0` would round it, and
its value may be negative. A string that is not a number at all, a number
with a fraction after that rounding, and a number of more than DIGITS digits
raise `SYNTAX 26.8` where the other operators raise `SYNTAX 41.1`. The
base is checked first, so a base that is not a number raises 41.1 even when
the exponent is bad too.

```rexx
numeric digits 3
say (2 ** 3.0) (2 ** 3.001) (2 ** 010) (2 ** 1e2) (2 ** -2) (2 ** 999)
signal on syntax name a
say 2 ** 1.5
a: say condition('o')~code
signal on syntax name b
say 2 ** 'x'
b: say condition('o')~code
signal on syntax name c
say 2 ** 1000
c: say condition('o')~code
signal on syntax name d
say 'x' ** 'y'
d: say condition('o')~code
signal on syntax name e
say 2 ** ''
e: say condition('o')~code
```

```output
8 8 1.02E+3 1.27E+30 0.25 5.36E+300
26.8
26.8
26.8
41.1
26.8
```

r[num.arithmetic.power]
A power answers the base raised to the exponent, rounded to DIGITS digits and
written without the zeros that end its decimal places, as [the trailing-zero
rule](r:num.formats.trailing) says. Any base to the power 0 is 1, zero
included, a base to a positive exponent is that many factors, and a base to a
negative exponent is the reciprocal of the power with the exponent's
magnitude. A negative base keeps its sign for an odd exponent. The base is cut
to DIGITS plus one digits first, and the factors are multiplied under rounding
of their own, so a power whose exact value does not fit in DIGITS digits may
differ from that value rounded in its last digit, and this book does not
specify which.

```rexx
say (2 ** 10) (2 ** -3) ((-2) ** 3) ((-2) ** 2) (1.5 ** 2) (0 ** 3) (0 ** 0)
say (5.0 ** 1) (1.50 ** 2) (10 ** -2) (2.5 ** 0) (0.5 ** 3) (3 ** 20)
numeric digits 3
say (1.01 ** 100) (1.5 ** 10) (7 ** 7) (2 ** -999)
numeric digits 1
say (3 ** 3) (7 ** 2) (2 ** 9)
```

```output
1024 0.125 -8 4 2.25 0 1
5 2.25 0.01 1 0.125 3.4867844E+9
2.7 57.7 8.24E+5 1.87E-301
3E+1 5E+1 5E+2
```

r[num.arithmetic.power.error]
Only a zero base raises `SYNTAX 42.903`, and it does so for a negative
exponent. A power raises `SYNTAX 42.1` when the first digit of the base, at
power of ten `A`, and the exponent `N` have a product `A * N` that is more
than 999999999 in magnitude. The test is made before any digit is computed and
does not look at the value, so it also refuses a power whose value is within
the [exponent range](r:num.limits.range). A power that passes the test is
computed. A product of the computation whose first digit stands above the
range raises `SYNTAX 42.901`, whatever the sign of the exponent. The last
digit of a product is not tested, so an intermediate value that is out of
range only at its last digit raises nothing. The final result, which for a
negative exponent is the reciprocal of the unrounded product, is tested as
[the power rule](r:num.limits.power) and [the underflow
rule](r:num.limits.underflow) say. A base such as 100 therefore raises `42.1`
at an exponent for which 11 raises `42.901`.

```rexx
signal on syntax name a
say 0 ** -1
a: say condition('o')~code
signal on syntax name b
say 100 ** 999999999
b: say condition('o')~code
signal on syntax name c
say 11 ** 999999999
c: say condition('o')~code
signal on syntax name d
say 3e333333333 ** 3
d: say condition('o')~code
signal on syntax name e
say 0.1000000001 ** 999999999
e: say condition('o')~code
signal on syntax name f
say 3e-333333334 ** 3
f: say condition('o')~code
signal on syntax name h
say 11 ** -999999999
h: say condition('o')~code
numeric digits 12
signal on syntax name g
say 0.5 ** 1000000000
g: say condition('o')~code
say (0.5 ** 999999999) (1.5 ** 1000000000) (0 ** 0)
numeric digits 9
say (10 ** 999999999) (2e333333333 ** 3) (3e-333333333 ** 3)
```

```output
42.903
42.1
42.901
42.901
42.902
42.1
42.901
42.1
4.33559593523E-301029996 1.13679261102E+176091259 1
1E+999999999 8E+999999999 2.7E-999999998
```

## Errors in the multiplicative and additive operators

r[num.arithmetic.range]
A result of `+`, `-`, `*` or `/` that is out of the exponent range raises the
errors that [the limits page](limits.md) gives, `SYNTAX 42.901` for an
overflow and `SYNTAX 42.902` for an underflow, and the integer division
operators raise [the errors above](r:num.arithmetic.integer) for a result that
is too wide. No operator answers a value outside the range, and [the underflow
rule](r:num.limits.underflow) says that a result that is too small is not
replaced by zero.

```rexx
signal on syntax name a
say 5e999999999 * 20
a: say condition('o')~code
signal on syntax name b
say 1e-999999999 / 10
b: say condition('o')~code
signal on syntax name c
say 1e999999999 % 7
c: say condition('o')~code
```

```output
42.901
42.902
26.11
```

## Lost digits

r[num.arithmetic.lostdigits]
An operand of an arithmetic operator that has more digits than NUMERIC DIGITS
raises the LOSTDIGITS condition, and so does the base of `**` but not its
exponent. The digits counted are those from the first nonzero digit to the
last one written, so zeros that end the operand count, and the exponent and
the zeros that start the operand do not. An operand of value zero has no
digits. The condition is raised for the left operand first and only after both
operands were found to be numbers, and the builtins raise no such condition.
When the condition is not trapped the operation goes on with the digits it
has, cut as the [cut rule](r:num.arithmetic.cut) says. Both
operands are checked for lost digits before the operator looks at their
values, so a wide left operand raises LOSTDIGITS before `SYNTAX 42.3` for a
zero divisor of `/`, `%` and `//` ([the zero rule](r:num.arithmetic.zero))
and before `SYNTAX 26.8` for an exponent of `**` that is a number but not a
whole number of at most DIGITS digits ([the exponent
rule](r:num.arithmetic.exponent)).

```rexx
numeric digits 3
say 1234 + 1
signal on lostdigits name a
say 1234 + 1
a: say condition('c') condition('d')
signal on lostdigits name b
say 5 * 1.200
b: say condition('d')
signal on lostdigits name c
say 99 * 1000
c: say condition('d')
signal on lostdigits name d
say 1234 * 5678
d: say condition('d')
signal on syntax name e
say 1234 + 'x'
e: say condition('o')~code
signal on syntax name f
say 2 ** 1234
f: say condition('o')~code
signal on lostdigits name g
say (100 + 1e10 + 0.0000 + 1.00e-5) (0.0123 * 2) (1234 == 1234) abs(1234)
g: say 'end'
```

```output
1.24E+3
LOSTDIGITS 1234
1.200
1000
1234
41.1
26.8
1E+10 0.0246 1 1.23E+3
end
```

## Deviations

r[num.arithmetic.deviation.53]
A divisor that is wider than NUMERIC DIGITS raises LOSTDIGITS even when the
dividend is zero, and the description of the condition is the text of the
operand as it was written, whichever operator produced the condition. ooRexx
does not check the divisor when the dividend is zero and answers `0`. It
describes the operand of `*`, `/`, `%`, `//` and `**` and the operand of a
prefix operator by its value, written as [the notation
rule](r:bif.index.num.write) writes a result under the settings in force, with
every digit kept and none rounded, so `1234.50` becomes `1.23450E+3`. An
operand whose last digit stands at the units is written without an exponent
however wide it is. It keeps the text as written for the operands of `+`, `-`
and the comparisons.

```rexx
numeric digits 3
signal on lostdigits name a
say 0 / 1.234
a: say condition('d')
z = '1.234E3'
signal on lostdigits name b
say -z
b: say condition('d')
signal on lostdigits name c
say z * 1
c: say condition('d')
signal on lostdigits name d
say 1 * '1234.5'
d: say condition('d')
```

```output
1.234
1.234E3
1.234E3
1234.5
```

r[num.arithmetic.deviation.102]
When LOSTDIGITS is trapped, an operand that is wider than NUMERIC DIGITS
raises it before the operator checks a zero divisor or an exponent that is
not a whole number. ooRexx checks the right operand first and raises
`SYNTAX 42.3` or `SYNTAX 26.8` instead.

```rexx
numeric digits 3
signal on lostdigits name a
signal on syntax name a
say 1234 / 0
a: say condition('c')
signal on lostdigits name b
signal on syntax name b
say 1234 ** 0.5
b: say condition('c')
```

```output
LOSTDIGITS
LOSTDIGITS
```
