---
element: D2X
kind: builtin
syntax: "D2X(wholenumber [, n])"
ansi: "9.6.8"
rexxref: "Functions > Built-in Functions > D2X (Decimal to Hexadecimal)"
deviations: [4]
---

# D2X

D2X answers the hexadecimal digits that stand for a whole number, in
capital letters: `41` for 65 and `100` for 256. With a length `n` it answers
exactly `n` hexadecimal digits, and can write a negative number.

## Arguments

r[bif.d2x.method]
D2X is a builtin that runs as a String method, and `wholenumber`, its first
argument, is the receiver; `n` becomes the message's argument.
`wholenumber` is a [decimal number](r:bif.index.conv.decimal) and `n` is a
[length](r:bif.index.conv.length). The argument count is checked by the
[builtin call rules](r:bif.index.call.count): one is required and two is the
most.

```rexx
say d2x(65) 65~d2x 65~d2x(4)
signal on syntax name a
say d2x()
a: say condition('o')~code
signal on syntax name b
say d2x(, 1)
b: say condition('o')~code
signal on syntax name c
say d2x(65, 1, 1)
c: say condition('o')~code
```

```output
41 41 0041
40.3
40.5
40.4
```

r[bif.d2x.errors]
The errors of the arguments are [those of the decimal
argument](r:bif.index.conv.decimal), which raises `SYNTAX 93.928` in D2X, and
those of [the length](r:bif.index.conv.length), and they are checked in
[the order](r:bif.index.conv.decimal.order) the shared rules give.

```rexx
signal on syntax name a
say d2x('65.5')
a: say condition('o')~code
signal on syntax name b
say d2x(65, '1.5')
b: say condition('o')~code
signal on syntax name c
say d2x(65, -1)
c: say condition('o')~code
signal on syntax name d
say d2x('abc', -1)
d: say condition('o')~code
```

```output
93.928
40.12
93.923
93.928
```

## The result

r[bif.d2x.unsigned]
Without `n`, a negative `wholenumber` raises `SYNTAX 93.927`. The answer
for any other number is its base-16 form, with the digits `0` to `9` and
`A` to `F`, in as few digits as it needs and at least one: 0 is the single
digit `0`, and no other answer starts with `0`.

```rexx
say d2x(65) d2x(0) d2x(255) d2x(256) d2x(65535) d2x(1234567) d2x('0065')
signal on syntax name a
say d2x(-1)
a: say condition('o')~code
signal on syntax name b
say d2x(-65)
b: say condition('o')~code
```

```output
41 0 FF 100 FFFF 12D687 41
93.927
93.927
```

r[bif.d2x.digits]
Letters in the answer are always capitals, whatever the number, and a
whole number written in another way, with an exponent or a fractional part of
zeros, has the answer of its value.

```rexx
say d2x(171) d2x(48879) d2x('1.71E2') d2x('171.000') d2x(' +171 ') d2x('-0')
```

```output
AB BEEF AB AB AB 0
```

r[bif.d2x.n]
With `n`, the answer is exactly `n` hexadecimal digits, as [the signed-writing
rule](r:bif.index.conv.signed.write) says. A non-negative number has `0`
digits added on the left of its own digits, and one that needs more than `n`
digits is cut to its last `n`; a negative number is the next rule's.

```rexx
say d2x(65, 1) d2x(65, 2) d2x(65, 5) d2x(76, 3) d2x(-1, 1) d2x(-1, 3)
say d2x(321, 1) d2x(321, 2) d2x(4095, 2) d2x(65535, 1) d2x(1234567, 4)
```

```output
1 41 00041 04C F FFF
1 41 FF F D687
```

r[bif.d2x.n.negative]
A negative number with an `n` is written in two's complement: the answer
is the number's remainder on division by 16 to the power of `n`, taken as
a non-negative number and written in exactly `n` digits, with `0` digits
on its left. For `n` of 1, -17 leaves the remainder 15, so its answer is
`F`; a number that needs more than `n` digits is cut like a positive one.

```rexx
say d2x(-16, 1) d2x(-17, 1) d2x(-128, 2) d2x(-129, 2) d2x(-129, 3)
say d2x(-256, 2) d2x(-256, 3) d2x(-257, 3) d2x(-65536, 4) d2x(-65536, 5)
say d2x(-257, 1) d2x(-4097, 2) d2x(-32769, 4)
```

```output
0 F 80 7F F7F
00 F00 EFF 0000 F0000
F FF 7FFF
```

r[bif.d2x.n.zero]
With an `n` of 0 the answer is the null string, for every `wholenumber` that
passes its checks, and a negative `wholenumber` is accepted.

```rexx
say '['d2x(65, 0)']' '['d2x(-65, 0)']' '['d2x(0, 0)']' length(d2x(1, 0))
```

```output
[] [] [] 0
```

r[bif.d2x.digits-limit]
NUMERIC DIGITS bounds the number, as [the decimal-argument
rule](r:bif.index.conv.decimal) says, and not the answer: the answer of a
number with at most that many digits has no more hexadecimal digits than it has
decimal ones when it is written without an `n`, and an `n` may be
larger than NUMERIC DIGITS.

```rexx
say d2x(999999999) length(d2x(999999999)) d2x(5, 12) length(d2x(-1, 30))
numeric digits 20
say d2x(18446744073709551615) d2x(-1, 16) d2x(2**64)
```

```output
3B9AC9FF 8 000000000005 30
FFFFFFFFFFFFFFFF FFFFFFFFFFFFFFFF 10000000000000000
```

r[bif.d2x.inverse]
X2D undoes D2X. For a non-negative whole number with at most NUMERIC DIGITS
digits, X2D of the answer of D2X without an `n` is the number. When both
calls are given the same `n` it holds as well for every such number from
-8\*16\*\*(`n`-1) up to 8\*16\*\*(`n`-1)-1, the numbers that `n` digits hold
as a signed number, the negative ones included.

```rexx
say x2d(d2x(300)) x2d(d2x(0)) x2d(d2x(65535)) x2d(d2x(300, 4), 4),
    x2d(d2x(-300, 4), 4)
say x2d(d2x(7, 1), 1) x2d(d2x(-8, 1), 1) x2d(d2x(8, 1), 1)
```

```output
300 0 65535 300 -300
7 -8 -8
```

## Deviations

r[bif.d2x.deviation.4]
interrexx does not specify whether the answer of D2X is the same object as
another string of equal value, as
[the shared rule](r:bif.index.conv.deviation.4) says for every conversion
function. A program can rely only on the value,
which is equal under `==`.

```rexx
say (d2x(97) == '61') (d2x(0, 0) == '') (d2x(10, 1) == 'A')
```

```output
1 1 1
```
