---
element: TRUNC
kind: builtin
syntax: "TRUNC(number [, n])"
ansi: "9.4.6"
rexxref: "Functions > Built-in Functions > TRUNC (Truncate)"
deviations: [4]
---

# TRUNC

TRUNC cuts a number off after a given number of decimal places, without
rounding at the cut, and writes the result in plain notation with exactly
that many places.

## Arguments

r[bif.trunc.method]
TRUNC is a builtin that runs as a String method, and `number`, its first
argument, is the receiver; `n` becomes the message's argument. `number` is a
[number](r:bif.index.num.arg) and `n` is a [whole-number](r:conv.whole.form)
argument that is the count of decimal places. The argument count is checked
by the [builtin call rules](r:bif.index.call.count): one is required and two
is the most.

```rexx
say trunc(12.37) 12.37~trunc 12.37~trunc(1) trunc(12.37, 1)
signal on syntax name a
say trunc()
a: say condition('o')~code
signal on syntax name b
say trunc(, 1)
b: say condition('o')~code
signal on syntax name c
say trunc(1, 1, 1)
c: say condition('o')~code
```

```output
12 12 12.3 12.3
40.3
40.5
40.4
```

r[bif.trunc.n.default]
When `n` is omitted, it is 0.

```rexx
say trunc(7.9) trunc(7.9, ) trunc(7.9, 0)
```

```output
7 7 7
```

r[bif.trunc.errors]
The checks run in this order, and the first that fails is raised: the
[form](r:conv.whole.form) and [value](r:conv.whole.digits) of `n`, which
raise `SYNTAX 40.12`; that `number` is a [number](r:bif.index.num.arg),
`SYNTAX 93.943`; and last that `n` is not negative, `SYNTAX 93.906`. `n` is
a whole-number argument, so the [null string](r:conv.arg.empty) raises 40.12
and `-0` is 0. A string that is not a number is therefore raised even when `n` is
negative, and a negative `n` is raised only for a number that is valid.

```rexx
say trunc(5, '-0') trunc(5, ' 2 ') trunc(5, '1e1')
signal on syntax name a
say trunc(5, '')
a: say condition('o')~code
signal on syntax name b
say trunc('x', 'y')
b: say condition('o')~code
signal on syntax name c
say trunc('x', -1)
c: say condition('o')~code
signal on syntax name d
say trunc(5, -1)
d: say condition('o')~code
signal on syntax name e
say trunc(5, 1.5)
e: say condition('o')~code
```

```output
5 5.00 5.0000000000
40.12
40.12
93.943
93.906
40.12
```

## The result

r[bif.trunc.result]
TRUNC first [rounds](r:bif.index.num.round) the number to NUMERIC DIGITS
significant digits. It then keeps the digits of its integer part, and
`n` digits after the point, and drops the rest, so the magnitude is cut, not
rounded: a decimal place beyond `n` has no effect on the ones before it.
When the number has fewer than `n` decimal places, zeros are added to make
`n`. With `n` of 0 the result is the integer part alone, with no point.

```rexx
say trunc(358.46913, 2) trunc(358.4, 3) trunc(358, 1) trunc(-12.79, 1),
    trunc(0.999, 0) trunc(-5.55, 1) trunc(5.55, 1)
```

```output
358.46 358.400 358.0 -12.7 0 -5.5 5.5
```

r[bif.trunc.plain]
The result is always written in plain notation, never with an exponent,
however many digits that takes and whatever NUMERIC FORM is. A number
that has an exponent is written out in full, and the digits dropped by the
rounding to NUMERIC DIGITS are written as zeros, and a decimal place cut off
after that rounding is not rounded again.

```rexx
say trunc('1.5e3') trunc('1e20') trunc('1.5e-3', 5) trunc('1e-20', 3)
numeric form engineering
say trunc('1.2345E+11', 2) trunc('1e-7', 8)
numeric digits 5
say trunc('1234567.89', 1) trunc('99999.9') trunc('0.000012345678', 8)
```

```output
1500 100000000000000000000 0.00150 0.000
123450000000.00 0.00000010
1234600.0 100000 0.00001234
```

r[bif.trunc.sign]
A negative number keeps its `-` when the result has a nonzero digit, and
loses it when every digit of the result is zero, so TRUNC never answers a
negative zero.

```rexx
say trunc(-0.5) trunc(-0.5, 2) trunc(-0.001, 2) trunc(-0.001, 3),
    trunc('-0') trunc('-0.0', 1) trunc(-1.2, 0)
```

```output
0 -0.50 0.00 -0.001 0 0.0 -1
```

> **Note.** The ANSI draft (9.4.6) checks `number` before an `n` that is not
> a whole number, where ooRexx, which this book follows, raises 40.12 for `n`
> first (for a negative `n` both check `number` first). It raises
> `SYNTAX 40.13` for a negative `n`, where ooRexx raises 93.906.

## Deviations

r[bif.trunc.deviation.4]
interrexx does not specify whether the result of TRUNC is the same object as
another string of equal value: as `number` itself, when the call leaves it
unchanged, or as an equal string produced some other way. Through
`identityHash`, or as a key of an `IdentityTable` or `IdentitySet`, the
answer may be either, and a program can rely only on the value, which is
equal under `==`. ooRexx differs: it answers the argument itself in a case
where interrexx may answer an equal new string.

```rexx
s = '12' || '5'
say (trunc(s) == s) (trunc('125') == '125') (trunc(s, 1) == '125.0')
```

```output
1 1 1
```
