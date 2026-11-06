---
element: ABS
kind: builtin
syntax: "ABS(number)"
ansi: "9.4.1"
rexxref: "Functions > Built-in Functions > ABS (Absolute Value)"
deviations: [4]
---

# ABS

ABS answers the magnitude of a number: the number without its minus sign,
rounded to NUMERIC DIGITS digits.

## Arguments

r[bif.abs.method]
ABS is a builtin that runs as a String method, and `number`, its only
argument, is the receiver. `number` is a [number](r:bif.index.num.arg). The
argument count is checked by the [builtin call rules](r:bif.index.call.count):
one is required and one is the most.

```rexx
say abs(-3) (-3)~abs '-3'~abs
signal on syntax name a
say abs()
a: say condition('o')~code
signal on syntax name b
say abs(,)
b: say condition('o')~code
signal on syntax name c
say abs(1, 2)
c: say condition('o')~code
signal on syntax name d
say abs('three')
d: say condition('o')~code
```

```output
3 3 3
40.3
40.3
40.4
93.943
```

## The result

r[bif.abs.result]
ABS returns the number [rounded](r:bif.index.num.round) to NUMERIC DIGITS
digits with its sign removed, and [written](r:bif.index.num.write) in plain
or scientific notation as that rule says, with NUMERIC DIGITS as the
trigger. So the result has the value of the number when it is not negative, and the
value of its negation when it is.

```rexx
say abs('12.3') abs(' -0.307') abs('-1.500') abs('+4') abs('-1E2')
numeric digits 4
say abs('-123456') abs('123.456') abs('-0.00012345')
numeric form engineering
say abs('-123456')
```

```output
12.3 0.307 1.500 4 100
1.235E+5 123.5 0.0001235
123.5E+3
```

r[bif.abs.zero]
ABS of a number whose value is zero is `0`, whatever its written form and
sign.

```rexx
say abs('-0') abs('0.00') abs('-0.0E5') abs(' +0 ')
```

```output
0 0 0 0
```

r[bif.abs.rounding]
The rounding is done before the sign is removed, and does not depend on the
sign. It is the only change ABS makes to the magnitude.

```rexx
numeric digits 3
say abs('-1.235') abs('1.235') abs('-1.2349') abs('9996') abs('-0.9995')
```

```output
1.24 1.24 1.23 1.00E+4 1.00
```

## Deviations

r[bif.abs.deviation.4]
interrexx does not specify whether the result of ABS is the same object as
another string of equal value: as `number` itself, when the call leaves it
unchanged, or as an equal string produced some other way. Through
`identityHash`, or as a key of an `IdentityTable` or `IdentitySet`, the
answer may be either, and a program can rely only on the value, which is
equal under `==`. ooRexx differs: it answers the argument itself in a case
where interrexx may answer an equal new string.

```rexx
s = '12' || '5'
say (abs(s) == s) (abs('125') == '125') (abs(-s) == s)
```

```output
1 1 1
```
