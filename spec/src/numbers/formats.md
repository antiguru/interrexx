---
element: "Number formats"
kind: concept
ansi: "7.4.10"
rexxref: "Numbers and Arithmetic > Exponential Notation"
---

# Number formats

An operation that answers with a number writes it as a new string, and that
string is the only trace of the computation: nothing remembers how many
digits the operands had or which operation made the result. This page says
what the written string looks like. The rules for rounding a number to a
count of significant digits and for choosing plain or exponential notation
are stated once, with the built-in functions, and this page applies them to
the results of arithmetic and adds what is particular to those results.

## Which strings are rewritten

r[num.formats.result]
Every result of an arithmetic operator, the prefix operators `+` and `-`
included, is a number that is [rounded](r:bif.index.num.round) to NUMERIC
DIGITS significant digits and then [written](r:bif.index.num.write) with
NUMERIC DIGITS as the trigger value, in the NUMERIC FORM in force when the
operation runs, which is the form of the operation and not the form that the
operands were written under. The result of the integer division `%` and of
the remainder `//` is written the same way.

```rexx
numeric digits 4
say (12345 + 1) (-12345 - 1) (2 * 0.00000123) (1 / 7) (22 / 7) (3 ** 15),
    (17 % 5) (17 // 5)
say (+0.00012345) (-98765.4)
numeric form engineering
say (12345 + 1) (-12345 - 1) (2 * 0.00000123) (1 / 7000000) (3 ** 15)
numeric digits 9
x = 1e11 + 0
say x (x + 0) (x * 1) (-x) (x / 1) (x ** 1)
numeric form scientific
say x (x + 0) (x * 1) (-x) (x / 1) (x ** 1)
```

```output
1.235E+4 -1.235E+4 0.00000246 0.1429 3.143 1.435E+7 3 2
0.0001235 -9.877E+4
12.35E+3 -12.35E+3 0.00000246 142.9E-9 14.35E+6
100E+9 100E+9 100E+9 -100E+9 100E+9 100E+9
100E+9 1E+11 1E+11 -1E+11 1E+11 1E+11
```

r[num.formats.operand]
A string that is not the result of an operation keeps the characters it has,
whether it is a literal, the value of a variable or the answer of a
concatenation. Only an arithmetic operator writes a number anew, so adding 0
to a number or multiplying it by 1 brings it into the written form.

```rexx
a = 1.50; b = '0012'; c = '-0'
say a b c a||b
say (a + 0) (b + 0) (c + 0) (-c) (+b)
```

```output
1.50 0012 -0 1.500012
1.50 12 0 0 12
```

## The characters of a written number

r[num.formats.canonical]
A written number has no whitespace and never starts with `+`. It has no
leading zero, except the single `0` before the point of a magnitude below 1
and the number zero itself, and its decimal point stands between two digits,
so it is never the first or the last character. Exponential notation is as
[the notation rule](r:bif.index.num.sci) gives it. The leading zeros, the
leading or trailing point, the plus signs and the whitespace that the operand
had do not survive.

```rexx
say (' +007.250 ' + 0) ('.5' + 0) ('5.' + 0) ('0.5e1' + 0) ('-.5E+0' + 0),
    ('+ 3' + 0)
say ('00.00100' + 0) ('1e0' + 0) ('12E+01' + 0) ('125E-3' + 0) ('-1e-2' + 0)
```

```output
7.250 0.5 5 5 -0.5 3
0.00100 1 120 0.125 -0.01
```

r[num.formats.zero]
A result that has the value zero is written as the single character `0`,
with no sign, no decimal places and no exponent. This holds however the
zero came about: a product with a zero operand, a difference of equal
numbers, a negation, or a zero written with decimal places or an exponent.
No operation answers a negative zero.

```rexx
say (5 - 5.00) (0.00 * 7) (0 * -3) (-0.0 + 0) (-(0.00)) (1e5 - 1e5),
    ('-0' + 0) ('0e9' * 1)
say (0 / -5) (-5 * 0) (0.0 - 0.0) (-0.5 * 0)
```

```output
0 0 0 0 0 0 0 0
0 0 0 0
```

## Digits and zeros

r[num.formats.rounded]
When the exact result has more significant digits than NUMERIC DIGITS, the
digits of the rounded result number exactly NUMERIC DIGITS, and the zeros
that end them are kept, as a number that rounds up to a power of ten shows.
A result too wide for [plain notation](r:bif.index.num.write) shows those
zeros before its exponent. A result with fewer digits than NUMERIC DIGITS is
not extended. A result of `/` or `**` is the exception, as
[the next rule](r:num.formats.trailing) says.

```rexx
numeric digits 4
say (1234567 + 0) (1.00000 + 0) (99999 + 0) (0.99996 + 0) (12.34996 + 0)
say (1.5 + 0) (1.50 + 0) (1500 + 0) (15000 + 0) (123456 * 1)
numeric digits 9
say (999999999 + 1) (123456789 + 1) (1234567895 + 0) (1.00000000000 + 0)
say (100000000 * 10) (1e9 + 0) (3e9 * 1) (1000000000 * 3)
```

```output
1.235E+6 1.000 1.000E+5 1.000 12.35
1.5 1.50 1500 1.500E+4 1.235E+5
1.00000000E+9 123456790 1.23456790E+9 1.00000000
1.00000000E+9 1E+9 3E+9 3.00000000E+9
```

r[num.formats.trailing]
The zeros after the decimal point of a result of `+`, `-`, `*` or `//` are the
zeros that its digits include, and they are kept. A result of `/` or `**`
has the zeros that end its digits removed after rounding, so no zero ends its
decimal places, and none ends the digits before the exponent when it is
written in exponential notation. The zeros of a whole number in plain
notation are not digits but place holders, and stay. How many decimal places
a sum, a difference or a product has is a matter of the arithmetic rules and
not of this page.

```rexx
say (1.25 + 1.75) (9.5 - 0.5) (1.5 * 4) (4.50 + 0)
say (7.50 / 2.5) (9.0 / 3) (5.0 ** 2) (0.5 ** 2) (7.5 / 3)
say (0.10 * 0.10) (2.5 * 2.5) (0.5 * 0.2) (7.50 // 2) (10.00 // 3.0)
say (10000000000 + 0) (10000000000 / 1) (1.0 / 1) (100 / 1) (5.00 ** 1)
say (1000000000 * 1) (1000000000 / 1) (2000000000 / 2) (4000000000 / 4)
numeric digits 3
say (1500 + 0) (1500 / 1) (1.20 + 0) (1.20 / 1) (1.20 * 1) (1.20 ** 1)
```

```output
3.00 9.0 6.0 4.50
3 3 25 0.25 2.5
0.0100 6.25 0.10 1.50 1.00
1.00000000E+10 1E+10 1 100 5
1.00000000E+9 1E+9 1E+9 1E+9
1.50E+3 1.5E+3 1.20 1.2 1.20 1.2
```

## Plain and exponential notation

r[num.formats.plain]
A result is written in plain or exponential notation by [the writing
rule](r:bif.index.num.write), with NUMERIC DIGITS as its trigger value. The
sign and the point are not counted, and the zeros between the point and the
first nonzero digit are. The count is made on the rounded result, so a result
that rounds up into a new digit before the point can move from plain to
exponential notation.

```rexx
say (0.000003 * 2) (0.000003 * 0.5) (0.0000000001 * 1) (0.00000000001 * 1)
say (-0.000000000000000001 * 1) (-0.0000000000000000001 * 1)
numeric digits 3
say (0.000012 * 1) (0.0000012 * 1) (-0.000012 * 1) (-0.0000012 * 1),
    (1.2e-6 * 1)
say (999 + 0) (1000 * 1) (-1000 * 1) (999.9 + 0) (99.99 + 0) (1e3 + 0)
numeric digits 1
say (9 + 0) (10 + 0) (0.01 * 1) (0.001 * 1) (0.5 + 0) (0.05 * 1) (0.005 * 1)
```

```output
0.000006 0.0000015 0.0000000001 0.00000000001
-0.000000000000000001 -1E-19
0.000012 1.2E-6 -0.000012 -1.2E-6 1.2E-6
999 1.00E+3 -1.00E+3 1.00E+3 100 1E+3
9 1E+1 0.01 1E-3 0.5 0.05 5E-3
```

> **Note.** The ANSI draft (7.4.10) tests the digits before the point as
> this rule does, and it removes trailing zeros after division and power as
> [the trailing-zero rule](r:num.formats.trailing) does. It differs for a
> magnitude below 1, as the note to [the writing rule](r:bif.index.num.write)
> says.

r[num.formats.engineering]
Under NUMERIC FORM ENGINEERING a number that is written in plain notation is
written exactly as under SCIENTIFIC. A number in exponential notation is
written as [the notation rule](r:bif.index.num.sci) gives it, and the zeros
that extend its digits to the point can make it longer than NUMERIC DIGITS.
Under both forms the same numbers have the same exponent when it is already a
multiple of 3. The form that counts is the one in force when the result is
written.

```rexx
say (1e11 + 0) (12345678901 + 0) (1.5e11 + 0) (1e-11 * 1) (1e12 + 0)
numeric form engineering
say (1e11 + 0) (12345678901 + 0) (1.5e11 + 0) (1.5e10 * 1) (1e-11 * 1),
    (1.5e-12 * 1)
say (1000 * 1) (1e12 + 0) (0 * 1e12) (123456789 * 1),
    (0.00000000000000000123 * 1)
numeric digits 1
say (5e11 * 1) (5e10 * 1) (5e12 * 1) (5e-20 * 1) (5e-21 * 1) (8e0 * 1)
numeric form scientific
say (5e11 * 1) (5e-20 * 1)
```

```output
1E+11 1.23456789E+10 1.5E+11 0.00000000001 1E+12
100E+9 12.3456789E+9 150E+9 15E+9 0.00000000001 0.0000000000015
1000 1E+12 0 123456789 1.23E-18
500E+9 50E+9 5E+12 50E-21 5E-21 8
5E+11 5E-20
```
