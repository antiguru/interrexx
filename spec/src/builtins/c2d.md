---
element: C2D
kind: builtin
syntax: "C2D(string [, n])"
ansi: "9.6.5"
rexxref: "Functions > Built-in Functions > C2D (Character to Decimal)"
deviations: [4]
---

# C2D

C2D answers the number that the characters of a string stand for, when each
character is a digit of base 256, the first one the most significant. With a
length `n` it reads the string as a signed number of `n` characters.

## Arguments

r[bif.c2d.method]
C2D is a builtin that runs as a [String method](r:class.string.builtins), and
`string`, its first argument, is the receiver; `n` becomes the message's
argument. `string` is a [string](r:conv.string.null), and `n` is a
[length](r:bif.index.conv.length). The argument count is checked by the
[builtin call rules](r:bif.index.call.count): one is required and two is the
most.

```rexx
say c2d('ff'x) 'ff'x~c2d 'ff'x~c2d(1)
signal on syntax name a
say c2d()
a: say condition('o')~code
signal on syntax name b
say c2d(, 1)
b: say condition('o')~code
signal on syntax name c
say c2d('a', 1, 1)
c: say condition('o')~code
```

```output
255 255 -1
40.3
40.5
40.4
```

r[bif.c2d.n.default]
When `n` is omitted, the string is read as a number with no sign whatever its
leftmost bit is, and the answer is never negative. An omitted `n` is not an
`n` of the length of `string`: with that `n` a string whose leftmost bit is
set is negative.

```rexx
say c2d('80'x) c2d('80'x, ) c2d('ff'x) c2d('ff'x, , )
say c2d('80'x, 1) c2d('ff'x, 1)
```

```output
128 128 255 255
-128 -1
```

## The result

r[bif.c2d.unsigned]
Without `n`, the answer is the sum, over the characters of `string`, of the
character's code times 256 to the power of the number of characters
after it. Leading `'00'x` characters therefore change nothing.

```rexx
say c2d('0c'x) c2d('0100'x) c2d('e3a1'x) c2d('00000007'x) c2d('abc')
```

```output
12 256 58273 7 6382179
```

r[bif.c2d.signed]
With `n`, the string is read as [a signed number of `n`
characters](r:bif.index.conv.signed.read): it is brought to `n` characters
by adding `'00'x` characters on its left or removing characters from there,
and the answer is negative when the leftmost bit of the result is set.
Padding with
`'00'x` means that a string shorter than `n` is never negative, and that
only the last `n` characters of a longer string matter.

```rexx
say c2d('93'x, 1) c2d('93'x, 2) c2d('ffa5'x, 2) c2d('ffa5'x, 1) c2d('ff5a'x, 1)
say c2d('e3a1'x, 2) c2d('0107'x, 0) c2d('ff'x, 5) c2d('80'x, 1) c2d('7f'x, 1)
```

```output
-109 147 -91 -91 90
-7263 0 255 -128 127
```

r[bif.c2d.null]
When `string` is the null string, the answer is 0, with an `n` or without
one.

```rexx
say c2d('') c2d('', 0) c2d('', 1) c2d('', 8)
```

```output
0 0 0 0
```

r[bif.c2d.characters]
C2D reads the characters of `string`, whatever they are, and not their value
as a number: a string of digits is a number in base 256 of the codes of its
digits.

```rexx
say c2d('1') c2d('12') c2d(12) c2d(1.5)
```

```output
49 12594 12594 3223093
```

r[bif.c2d.digits]
The answer must fit in the caller's NUMERIC DIGITS setting, by
[the rule for the answer](r:bif.index.conv.digits-limit): when it has more
digits than that, the call raises `SYNTAX 93.936`. The caller's setting
decides, not the length of the string: a string can be longer than that when
its leading characters are `'00'x`, or when `n` cuts it short.

```rexx
say c2d('ffffffff'x, 4) c2d(copies('00'x, 30) || 'ff'x) c2d('ffffffffffffffffff'x, 1)
signal on syntax name a
say c2d('ffffffff'x)
a: say condition('o')~code
signal on syntax name b
say c2d('8000000000'x, 5)
b: say condition('o')~code
numeric digits 20
say c2d('ffffffffffffffff'x) c2d('8000000000000000'x, 8)
signal on syntax name c
say c2d('ffffffffffffffffff'x)
c: say condition('o')~code
```

```output
-1 255 -1
93.936
93.936
18446744073709551615 -9223372036854775808
93.936
```

r[bif.c2d.errors]
The checks of the call run in this order: the form of `n` as a
[whole number](r:conv.whole.form), `SYNTAX 40.12`; then the sign of `n`,
`SYNTAX 93.923`; and then the number of digits of the answer, `SYNTAX
93.936`. A negative `n` is therefore raised even when the answer would not
fit either.

```rexx
signal on syntax name a
say c2d('ffffffffffff'x, 'x')
a: say condition('o')~code
signal on syntax name b
say c2d('ffffffffffff'x, -1)
b: say condition('o')~code
signal on syntax name c
say c2d('ffffffffffff'x)
c: say condition('o')~code
```

```output
40.12
93.923
93.936
```

## Deviations

r[bif.c2d.deviation.4]
interrexx does not specify whether the answer of C2D is the same object as
another string of equal value, as [the shared
rule](r:bif.index.conv.deviation.4) says for every conversion function. A
program can rely only on the value, which is equal under `==`.

```rexx
say (c2d('') == 0) (c2d('a') == 97) (c2d('00'x) == '0')
```

```output
1 1 1
```
