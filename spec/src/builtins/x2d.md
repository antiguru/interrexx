---
element: X2D
kind: builtin
syntax: "X2D(hex_string [, n])"
ansi: "9.6.11"
rexxref: "Functions > Built-in Functions > X2D (Hexadecimal to Decimal)"
deviations: [4]
---

# X2D

X2D answers the number that a string of hexadecimal digits writes, in
decimal. With a length `n` it reads the digits as a signed number of `n`
hexadecimal digits.

## Arguments

r[bif.x2d.method]
X2D is a builtin that runs as a String method, and `hex_string`, its first
argument, is the receiver; `n` becomes the message's argument. `hex_string`
is a [string of hexadecimal digits](r:bif.index.conv.digits) and `n` is a
[length](r:bif.index.conv.length). The argument count is checked by the
[builtin call rules](r:bif.index.call.count): one is required and two is the
most.

```rexx
say x2d('ff') 'ff'~x2d 'ff'~x2d(2)
signal on syntax name a
say x2d()
a: say condition('o')~code
signal on syntax name b
say x2d(, 1)
b: say condition('o')~code
signal on syntax name c
say x2d('a', 1, 1)
c: say condition('o')~code
```

```output
255 255 -1
40.3
40.5
40.4
```

r[bif.x2d.digits]
`hex_string` may have [whitespace](r:bif.index.conv.groups) between its
digits and not at [its ends](r:bif.index.conv.groups.edge), and may be
[null](r:bif.index.conv.digits.null). The errors those rules give are
those of a hexadecimal string: `SYNTAX 93.933` for a character that is not a
digit, `SYNTAX 93.931` for whitespace at an end and `SYNTAX 93.976` for a gap
at a place that is not a multiple of 2 digits from the end. A sign is not a
digit, so a negative number cannot be written in `hex_string`.

```rexx
say x2d('1 ff') x2d('1' || '09'x || 'ff') x2d('')
signal on syntax name a
say x2d('1g')
a: say condition('o')~code
signal on syntax name b
say x2d('ff ')
b: say condition('o')~code
signal on syntax name c
say x2d('f f')
c: say condition('o')~code
signal on syntax name d
say x2d('-1')
d: say condition('o')~code
```

```output
511 511 0
93.933
93.931
93.976
93.933
```

r[bif.x2d.errors]
The checks of the call run in this order: the argument count; the form of
`n` as a [whole number](r:conv.whole.form), `SYNTAX 40.12`; the sign of `n`,
`SYNTAX 93.923`; then the digits of `hex_string`; and last the number of
digits of the answer, `SYNTAX 93.935`. A negative `n` is therefore raised even
when `hex_string` has a character that is not a digit or the answer would not
fit, and a non-whole `n` before that.

```rexx
signal on syntax name a
say x2d('zz', 'x')
a: say condition('o')~code
signal on syntax name b
say x2d('zz', -1)
b: say condition('o')~code
signal on syntax name c
say x2d('ffffffffff', -1)
c: say condition('o')~code
signal on syntax name d
say x2d('zz')
d: say condition('o')~code
signal on syntax name e
say x2d('ffffffffff')
e: say condition('o')~code
```

```output
40.12
93.923
93.923
93.933
93.935
```

## The result

r[bif.x2d.unsigned]
Without `n`, the answer is the sum, over the digits of `hex_string`, of the
digit's value times 16 to the power of the number of digits after it, and
is never negative. A letter is read in either case, whitespace does not count
as a digit and leading zeros change nothing.

```rexx
say x2d('0') x2d('f') x2d('FF') x2d('0ff') x2d('1 00') x2d('aBcD') x2d('8000')
```

```output
0 15 255 255 256 43981 32768
```

r[bif.x2d.signed]
With `n`, the digits are read as [a signed number of `n`
hexadecimal digits](r:bif.index.conv.signed.read): they are made `n` digits
long, by dropping digits on the left or adding `0` digits there, and the
answer is negative when the leftmost bit of the result is set, which is
when its leftmost digit is `8` or more. Padding with `0` means that a
string shorter than `n` is never negative, and that only the last `n` digits
of a longer one matter.

```rexx
say x2d('81', 2) x2d('81', 3) x2d('f081', 2) x2d('f081', 3) x2d('f081', 4)
say x2d('8', 1) x2d('7', 1) x2d('f', 2) x2d('ffff', 1) x2d('f ff', 2) x2d('8000', 4)
```

```output
-127 129 -127 129 -3967
-8 7 15 -1 -1 -32768
```

r[bif.x2d.n.default]
When `n` is omitted, the digits are read as an unsigned number whatever
the leftmost bit is. An omitted `n` is not an `n` of the number of digits:
with that `n` a string whose leftmost digit is `8` or more is negative.

```rexx
say x2d('80') x2d('80', ) x2d('ff') x2d('ff', , )
say x2d('80', 2) x2d('ff', 2)
```

```output
128 128 255 255
-128 -1
```

r[bif.x2d.n.zero]
With an `n` of 0 the answer is 0 and `hex_string` is not examined: a string
that is not a valid hexadecimal string, whether it has a character that is not
a digit or whitespace in the wrong place, raises no error. Only the checks of
`n` and of the argument count still apply.

```rexx
say x2d('ffff', 0) x2d('0', 0) x2d('', 0) x2d('zz', 0) x2d(' f f ', 0)
signal on syntax name a
say x2d('ff', -0 - 1)
a: say condition('o')~code
signal on syntax name b
say x2d('zz', 1)
b: say condition('o')~code
```

```output
0 0 0 0 0
93.923
93.933
```

r[bif.x2d.n.large]
An `n` larger than the number of digits just pads the digits with zeros, so
the answer is not negative; a large `n` is not an error and does not change
the answer.

```rexx
say x2d('8', 100) x2d('ff', 4) x2d('f', 99999999999) x2d('ffff', 5)
```

```output
8 255 15 65535
```

r[bif.x2d.null]
When `hex_string` is the null string, the answer is 0, with an `n` or without
one.

```rexx
say x2d('') x2d('', 0) x2d('', 1) x2d('', 8)
```

```output
0 0 0 0
```

r[bif.x2d.not-number]
X2D reads the characters of its argument and not its value as a number, so
an argument written as a number is treated as the digits it is written with:
NUMERIC DIGITS plays no part in reading it, and a number written with a period or
with a signed exponent has a character that is not a digit.

```rexx
say x2d(10) x2d(0010)
signal on syntax name a
say x2d(1.5)
a: say condition('o')~code
signal on syntax name b
say x2d(1e+3)
b: say condition('o')~code
```

```output
16 16
93.933
93.933
```

r[bif.x2d.digits-limit]
The answer must fit in the caller's NUMERIC DIGITS setting, by
[the rule for the answer](r:bif.index.conv.digits-limit): when it has more
digits than that, the call raises `SYNTAX 93.935`. The caller's setting
decides, not the length of `hex_string`: the string can be longer than that when
its leading digits are `0`, or when `n` cuts it short.

```rexx
say x2d('3b9ac9ff') x2d(copies('0', 30) || 'ff') x2d('ffffffffff', 2)
signal on syntax name a
say x2d('3b9aca00')
a: say condition('o')~code
signal on syntax name b
say x2d('8000000000', 10)
b: say condition('o')~code
numeric digits 20
say x2d('ffffffffffffffff') x2d('8000000000000000', 16)
```

```output
999999999 255 -1
93.935
93.935
18446744073709551615 -9223372036854775808
```

r[bif.x2d.inverse]
D2X undoes X2D. For a hexadecimal string without an `n`, D2X of the answer of
X2D is `hex_string` with its leading zeros and its whitespace removed, its
letters in capitals, and `0` for a string with no digit but `0`, or none at
all.

```rexx
say d2x(x2d('00ff')) d2x(x2d('aB 12')) d2x(x2d('0')) d2x(x2d(''))
```

```output
FF AB12 0 0
```

r[bif.x2d.deviation.4]
interrexx does not specify whether the answer of X2D is the same object as
another string of equal value, as [the shared rule](r:bif.index.conv.deviation.4)
says for every conversion function. A program can rely only on the value,
which is equal under `==`.

```rexx
say (x2d('') == 0) (x2d('61') == 97) (x2d('0a') == '10')
```

```output
1 1 1
```
