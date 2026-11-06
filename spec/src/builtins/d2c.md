---
element: D2C
kind: builtin
syntax: "D2C(wholenumber [, n])"
ansi: "9.6.7"
rexxref: "Functions > Built-in Functions > D2C (Decimal to Character)"
deviations: [4]
---

# D2C

D2C answers the string whose characters are the base-256 digits of a whole
number: the character with code 72 for the number 72, and two characters
for a number from 256 up. With a length `n` it answers exactly `n`
characters, and can write a negative number.

## Arguments

r[bif.d2c.method]
D2C is a builtin that runs as a String method, and `wholenumber`, its first
argument, is the receiver; `n` becomes the message's argument. `wholenumber`
is a [decimal number](r:bif.index.conv.decimal) and `n` is a
[length](r:bif.index.conv.length). The argument count is checked by the
[builtin call rules](r:bif.index.call.count): one is required and two is the
most.

```rexx
say d2c(72) 72~d2c c2x(72~d2c(2))
signal on syntax name a
say d2c()
a: say condition('o')~code
signal on syntax name b
say d2c(, 1)
b: say condition('o')~code
signal on syntax name c
say d2c(65, 1, 1)
c: say condition('o')~code
```

```output
H H 0048
40.3
40.5
40.4
```

r[bif.d2c.errors]
The errors of the arguments are [those of the decimal
argument](r:bif.index.conv.decimal), which raises `SYNTAX 93.929` in D2C, and
those of [the length](r:bif.index.conv.length), and they are checked in
[the order](r:bif.index.conv.decimal.order) the shared rules give.

```rexx
signal on syntax name a
say d2c('65.5')
a: say condition('o')~code
signal on syntax name b
say d2c(65, '1.5')
b: say condition('o')~code
signal on syntax name c
say d2c(65, -1)
c: say condition('o')~code
signal on syntax name d
say d2c('abc', -1)
d: say condition('o')~code
```

```output
93.929
40.12
93.923
93.929
```

## The result

r[bif.d2c.unsigned]
A negative `wholenumber` raises `SYNTAX 93.927` when there is no `n`. The
answer is the number written in base 256, most significant character first,
using as few characters as it can but at least one; `'00'x` begins it only
for the number 0, whose answer is the one character `'00'x`.

```rexx
say c2x(d2c(72)) c2x(d2c(0)) c2x(d2c(255)) c2x(d2c(256)) c2x(d2c(65535))
say c2x(d2c(65536)) c2x(d2c(1234567)) c2x(d2c('0065')) d2c(97)
signal on syntax name a
say d2c(-1)
a: say condition('o')~code
signal on syntax name b
say d2c(-65)
b: say condition('o')~code
```

```output
48 00 FF 0100 FFFF
010000 12D687 41 a
93.927
93.927
```

r[bif.d2c.n]
With `n`, the answer is exactly `n` characters, as [the signed-writing
rule](r:bif.index.conv.signed.write) says: the characters on the left of
the number's own are `'00'x` for a non-negative number and `'FF'x` for a
negative one, and a number that needs more than `n` characters is cut to
its last `n`.

```rexx
say c2x(d2c(72, 1)) c2x(d2c(72, 2)) c2x(d2c(72, 5)) c2x(d2c(83, 2))
say c2x(d2c(-173, 2)) c2x(d2c(-1, 1)) c2x(d2c(-1, 3)) c2x(d2c(-102, 1))
say c2x(d2c(1234567, 2)) c2x(d2c(256, 1)) c2x(d2c(255, 1)) c2x(d2c(65535, 1))
```

```output
48 0048 0000000048 0053
FF53 FF FFFFFF 9A
D687 00 FF FF
```

r[bif.d2c.n.negative]
A negative number with an `n` is written in two's complement: the answer
is the number's remainder on division by 256 to the power of `n`, taken as
a non-negative number and written in exactly `n` characters, with `'00'x`
characters on its left. For `n` of 1, -257 leaves the remainder 255, so
its answer is `'FF'x`; a number that needs more than `n` characters is cut
like a positive one.

```rexx
say c2x(d2c(-128, 1)) c2x(d2c(-129, 1)) c2x(d2c(-129, 2)) c2x(d2c(-256, 1))
say c2x(d2c(-256, 2)) c2x(d2c(-257, 2)) c2x(d2c(-65536, 2)) c2x(d2c(-65536, 3))
say c2x(d2c(-257, 1)) c2x(d2c(-32769, 1)) c2x(d2c(-65537, 2))
```

```output
80 7F FF7F 00
FF00 FEFF 0000 FF0000
FF FF FFFF
```

r[bif.d2c.n.zero]
With an `n` of 0 the answer is the null string, for every `wholenumber` that
passes its checks, and a negative `wholenumber` is accepted.

```rexx
say '['d2c(65, 0)']' '['d2c(-65, 0)']' '['d2c(0, 0)']' length(d2c(1, 0))
```

```output
[] [] [] 0
```

r[bif.d2c.inverse]
C2D undoes D2C. For a non-negative whole number with at most NUMERIC DIGITS
digits, C2D of the answer of D2C without an `n` is the number. When both calls
are given the same `n` it holds as well for every whole number from
-128\*256\*\*(`n`-1) up to 128\*256\*\*(`n`-1)-1, the numbers that `n`
characters hold as a signed number, the negative ones included.

```rexx
say c2d(d2c(300)) c2d(d2c(0)) c2d(d2c(65535)) c2d(d2c(300, 4), 4) c2d(d2c(-300, 4), 4)
say c2d(d2c(127, 1), 1) c2d(d2c(-128, 1), 1) c2d(d2c(128, 1), 1)
```

```output
300 0 65535 300 -300
127 -128 -128
```

## Deviations

r[bif.d2c.deviation.4]
interrexx does not specify whether the answer of D2C is the same object as
another string of equal value, as [the shared
rule](r:bif.index.conv.deviation.4) says for every conversion function. A
program can rely only on the value, which is equal under `==`.

```rexx
say (d2c(97) == 'a') (d2c(0, 0) == '') (d2c(98, 1) == 'b')
```

```output
1 1 1
```
