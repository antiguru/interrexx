---
element: SIGN
kind: builtin
syntax: "SIGN(number)"
ansi: "9.4.5"
rexxref: "Functions > Built-in Functions > SIGN"
---

# SIGN

SIGN answers which side of zero a number lies on: `-1` below it, `0` at it
and `1` above it.

## Arguments

r[bif.sign.method]
SIGN is a builtin that runs as a String method, and `number`, its only
argument, is the receiver. `number` is a [number](r:bif.index.num.arg). The
argument count is checked by the [builtin call rules](r:bif.index.call.count):
one is required and one is the most.

```rexx
say sign(-3) (-3)~sign '-3'~sign
signal on syntax name a
say sign()
a: say condition('o')~code
signal on syntax name b
say sign(1, 2)
b: say condition('o')~code
signal on syntax name c
say sign('')
c: say condition('o')~code
signal on syntax name d
say sign(.nil)
d: say condition('o')~code
```

```output
-1 -1 -1
40.3
40.4
93.943
93.943
```

## The result

r[bif.sign.result]
SIGN answers `-1` for a negative number, `1` for a positive one and `0`
for zero. The written form does not matter: a
leading `+`, whitespace, trailing zeros and an exponent change nothing, and
neither does a zero written with a minus sign.

```rexx
say sign('7.25') sign(' -0.041 ') sign(0.00) sign('-0') sign('+5') sign('- 5'),
    sign('0e5') sign('-0.00E-9') sign('1E3') sign('-1e-3')
```

```output
1 -1 0 0 1 -1 0 0 1 -1
```

r[bif.sign.digits]
NUMERIC DIGITS plays no part in the result, except that a number whose
rounding to NUMERIC DIGITS [overflows](r:bif.index.num.overflow) raises an
error. Rounding a number to fewer digits never changes it from nonzero to
zero, so a number far below the precision still has its sign, and so does
a number with an exponent at either end of the range.

```rexx
numeric digits 1
say sign('0.0000000001') sign('-123456789') sign('-1e-999999999'),
    sign('1e999999999')
```

```output
1 -1 -1 1
```
