---
element: FORMAT
kind: builtin
syntax: "FORMAT(number [, before [, after [, expp [, expt]]]])"
ansi: "9.4.2"
rexxref: "Functions > Built-in Functions > FORMAT"
deviations: [4]
---

# FORMAT

FORMAT writes a number in a chosen layout: a fixed width for the integer
part, a fixed count of decimal places, and a choice of when the exponent form
is used and how wide its exponent is. Called with only a number it writes the
number the way arithmetic would.

## Arguments

r[bif.format.method]
FORMAT is a builtin that runs as a String method, and `number`, its first
argument, is the receiver; `before`, `after`, `expp` and `expt` become the
message's arguments. `number` is a [number](r:bif.index.num.arg), and each of
the others is a [whole-number](r:conv.whole.form) argument that is a count of
character places. The argument count is checked by the [builtin call
rules](r:bif.index.call.count): one is required and five is the most.

```rexx
say '['format(3.5, 4)']' '['3.5~format(4)']' '['3.5~format(4, 2)']'
signal on syntax name a
say format()
a: say condition('o')~code
signal on syntax name b
say format(, 4)
b: say condition('o')~code
signal on syntax name c
say format(1, 2, 3, 4, 5, 6)
c: say condition('o')~code
```

```output
[   3.5] [   3.5] [   3.50]
40.3
40.5
40.4
```

r[bif.format.errors]
The checks run in this order, and the first that fails is raised: the
[form](r:conv.whole.form) and [value](r:conv.whole.digits) of `before`,
`after`, `expp` and `expt`, from left to right, which raise `SYNTAX 40.12`;
that `number` is a [number](r:bif.index.num.arg), `SYNTAX 93.943`; that none
of the four counts is negative, from left to right, `SYNTAX 93.906`; and
last the fit of the result: that the exponent has at most `expp` digits,
`SYNTAX 93.941`, and then that the integer part has at most `before`
characters, `SYNTAX 93.942`. The null string is not [omitted](r:conv.arg.empty)
and raises 40.12, and `-0` is 0.

```rexx
say '['format(5, ' 2 ', '-0')']'
signal on syntax name a
say format(5, , '')
a: say condition('o')~code
signal on syntax name b
say format('x', 'y', -1)
b: say condition('o')~code
signal on syntax name c
say format('x', -1)
c: say condition('o')~code
signal on syntax name d
say format(5, 1, -1, -1)
d: say condition('o')~code
signal on syntax name e
say format(5, , 1.5)
e: say condition('o')~code
signal on syntax name f
say format('1e15', 0, , 1)
f: say condition('o')~code
signal on syntax name g
say format('1e15', 1, , 0)
g: say condition('o')~code
```

```output
[ 5]
40.12
40.12
93.943
93.906
40.12
93.941
93.942
```

## The result

r[bif.format.default]
When only `number` is given, or `before`, `after`, `expp` and `expt` are
all omitted, FORMAT returns the number [rounded](r:bif.index.num.round) to
NUMERIC DIGITS digits and [written](r:bif.index.num.write) with NUMERIC
DIGITS as the trigger, which is what the arithmetic of `number + 0` gives.

```rexx
say format('3') format(' - 48.06') format('1.50') format('00.00') format('-0')
say format('1234567890') format('0.0000000000000000001') format(5, , , , )
numeric digits 3
say format('1.2345') format('-1.2355') format('12345')
```

```output
3 -48.06 1.50 0 0
1.23456789E+9 1E-19 5
1.23 -1.24 1.23E+4
```

r[bif.format.before]
`before` is the width of the integer part. The integer part of the result,
with its `-` when the number is negative, is padded on the left with
blanks to `before` characters; when `before` is omitted it is not padded.
The integer part is the characters before the point, or before the `E`
when there is no point, so in plain notation a magnitude below 1 has the
integer part `0`.
When the integer part, with its sign, is longer than `before`, the call raises
`SYNTAX 93.942`. Since the integer part is never empty, a `before` of 0
always raises it.

```rexx
say '['format(7, 3)']' '['format(-7, 3)']' '['format(0.5, 3)']',
    '['format(-12.5, 3)']' '['format(1234, 4)']'
signal on syntax name a
say format(-1234, 4)
a: say condition('o')~code
signal on syntax name b
say format(0, 0)
b: say condition('o')~code
signal on syntax name c
say format(1, 0)
c: say condition('o')~code
say '['format(-0, 1)']' '['format(5, , 0)']'
```

```output
[  7] [ -7] [  0.5] [-12.5] [1234]
93.942
93.942
93.942
[0] [5]
```

r[bif.format.before.rounded]
The integer part that `before` is checked against is the one the result
has, after the rounding to NUMERIC DIGITS and to `after` places, so a number
whose rounding adds a digit to the integer part needs the room for it.

```rexx
say '['format(9.4, 1, 0)']'
signal on syntax name a
say format(9.6, 1, 0)
a: say condition('o')~code
say '['format(9.6, 2, 0)']'
```

```output
[9]
93.942
[10]
```

r[bif.format.after]
`after` is the count of decimal places. When it is given, the number is
rounded to that many places, the magnitude going up when the first dropped
digit is 5 or more, or is extended with zeros to that many; and a point
separates them from the integer part, unless `after` is 0, when there is
no point. This rounding follows the rounding to NUMERIC DIGITS, and it is
not limited by it, so `after` can ask for more places than NUMERIC DIGITS.
When `after` is omitted, the places are those the number has. A negative
number whose value rounds to zero loses its `-`. This book does not specify
the result when the rounding to `after` carries the first digit to a power
of ten above 999999999.

```rexx
say format(123.456, , 2) format(123.456, , 5) format(123.456, , 0),
    format(-1.235, , 2) format(9.99, , 1) format(0.5, , 0)
say format(-0.4, , 0) format(-0.04, , 1) format(-0.05, , 1) format(-0, , 2)
numeric digits 3
say format(1.2345, , 6) format(12345678, , 1) format(0.12345, , 4)
```

```output
123.46 123.45600 123 -1.24 10.0 1
0 0.0 -0.1 0.00
1.230000 1.2E+7 0.1230
```

r[bif.format.trigger]
`expp` and `expt` control the notation. The trigger `T` of the [writing
rule](r:bif.index.num.write) is `expt` when it is given and NUMERIC DIGITS
otherwise. The two counts of that rule are taken at different steps. `S`,
the count of decimal places, is that of the number
[rounded](r:bif.index.num.round) to NUMERIC DIGITS, so it does not depend on
`after`. `W`, the count of digits before the point, is that of the number
after the rounding to `after` as well, so a rounding that carries into a new
integer digit can make the number scientific. An `expp` of 0 overrides the
trigger: the result is then always in plain notation, whatever its size.
With `expp` omitted or above 0, the number is written in
[scientific notation](r:bif.index.num.sci) exactly when the writing rule says
so for `T`. A `T` of 0 therefore makes every number scientific, zero
included, unless `expp` is 0.

```rexx
say format('12345.5', , , , 5) format('12345.5', , , , 4),
    format('0.00001234', , , , 4) format('0.00001234', , , , 3)
say format('1e30', , , 0) format('1e5', , , , 5) format('1e5', , , , 6)
say format('1e5', , , 0, 5) format('1e5', , , 3, 5) format('0.5', , , , 0)
say format('99999.5', , 0, , 5) format('99999.4', , 0, , 5),
    format('9999.96', , 1, , 4) format('99999.5', , 0, 0, 5)
say format('0.000000123', , 8, , 4) format('0.00000012', , 9, , 4)
numeric digits 3
say format('1e5') format('1e5', , , , 6) format('1.2345', , , 1, 1)
```

```output
12345.5 1.23455E+4 0.00001234 1.234E-5
1000000000000000000000000000000 1E+5 100000
100000 1E+005 5E-1
1E+5 99999 1.0E+4 100000
1.23000000E-7 0.000000120
1E+5 100000 1.23
```

r[bif.format.sci]
In [scientific notation](r:bif.index.num.sci), `after` is the count of
digits after the point in the part before the `E`: the digits are rounded to
it or extended with zeros, as in the plain case, and with 0 there is no
point. A rounding that carries into a new leading digit raises the exponent,
so the part before the `E` keeps the form the notation gives. `before` is the
width of the part before the point. NUMERIC FORM decides the exponent as the
notation rule says.

```rexx
say format('1e30', , 3) format('9.96e10', , 1) format('9.96e10', , 0),
    format('-1.2345e10', , 2)
say '['format('1.2345e10', 5, 1)']' '['format('-1.2345e-20', 3, 2)']'
numeric form engineering
say format('1.2345e10', , 1) format('1e11', 4, 1) format('9.96e10', , 1)
```

```output
1.000E+30 1.0E+11 1E+11 -1.23E+10
[    1.2E+10] [ -1.23E-20]
12.3E+9  100.0E+9 99.6E+9
```

r[bif.format.expp]
In scientific notation, `expp` is the width of the exponent: its digits are
padded on the left with zeros to `expp` places, and when it needs more
places, the call raises `SYNTAX 93.941`. The sign `+` or `-` is always
written. When `expp` is omitted the exponent has as many digits as it needs.
No such error comes from a number written in plain notation, whatever its
size.

```rexx
say format('1e15', , , 2) format('1e-25', , , 3) format('1e150', , , 3),
    format('1e15', , , 2, 16) format('5', , , 3, 6)
signal on syntax name a
say format('1e150', , , 2)
a: say condition('o')~code
signal on syntax name b
say format('1e-25', , , 1)
b: say condition('o')~code
signal on syntax name c
say format('1e999999999', , , 3)
c: say condition('o')~code
say format('1e999999999', , , 9)
```

```output
1E+15 1E-025 1E+150 1000000000000000 5
93.941
93.941
93.941
1E+999999999
```

r[bif.format.exponent-zero]
A number that is to be in scientific notation but whose exponent would be
0 is written without an exponent. That number is zero, or a result, after
the rounding to `after`, whose magnitude is at least 1 and below 10 (below
1000 under NUMERIC FORM ENGINEERING). When `expp` is above 0, the result then
ends with `expp + 2` blanks, the room the exponent would have taken; when
`expp` is omitted nothing follows. An `expp` of 0 never writes the exponent,
and adds no blanks.

```rexx
say '['format('1.2345', , 2, , 0)']' '['format('1.2345', , 2, 2, 0)']',
    '['format('1.2345', 5, 2, 3, 0)']'
say '['format(0, , , 1, 0)']' '['format(0, 2, 1, 0, 0)']' '['format(50, , , 2, 1)']'
say '['format('-5', , , 1, 0)']' '['format('9.996', , 2, 1, 0)']'
numeric form engineering
say '['format(123, , , 2, 2)']' '['format(123, , , , 2)']' '['format(1234, , , 2, 3)']'
say '['format('-999.4', , , 1, 2)']' '['format('999.6', , 0, 1, 2)']'
```

```output
[1.23] [1.23    ] [    1.23     ]
[0   ] [ 0.0] [5.0E+01]
[-5   ] [1.00E+1]
[123    ] [123] [1.234E+03]
[-999.4   ] [1E+3]
```

> **Note.** The ANSI draft (9.4.2) differs from ooRexx, which this book
> follows, in three ways. It checks the arguments from left to right, so a
> `number` that is not a number is raised before a bad count, and it raises
> `SYNTAX 40.13` for a negative count and `SYNTAX 40.38` for a result that
> does not fit, in the exponent or in the integer part. It decides the
> notation before it rounds to `after`, so there a carry out of the integer
> part does not make the number scientific: `format('99999.5', , 0, , 5)` is
> `100000`. And it tests the zeros after the point against a fixed limit, as
> the [note on the writing rule](r:bif.index.num.sci) says.

## Deviations

r[bif.format.deviation.4]
interrexx does not specify whether the result of FORMAT is the same object as
another string of equal value: as `number` itself, when the call leaves it
unchanged, or as an equal string produced some other way. Through
`identityHash`, or as a key of an `IdentityTable` or `IdentitySet`, the
answer may be either, and a program can rely only on the value, which is
equal under `==`. ooRexx differs: it answers the argument itself in a case
where interrexx may answer an equal new string.

```rexx
s = '12' || '5'
say (format(s) == s) (format('125') == '125') (format(s, 5) == '  125')
```

```output
1 1 1
```
