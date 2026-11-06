---
element: BITOR
kind: builtin
syntax: "BITOR(string1 [, string2] [, pad])"
ansi: "9.6.3"
rexxref: "Functions > Built-in Functions > BITOR (Bit by Bit OR)"
deviations: [4]
---

# BITOR

BITOR combines two strings bit by bit, setting a bit wherever either string has
it set, and answers the string of the combined characters. How the two
strings are lined up and what happens when their lengths differ are the
rules of [the bit-by-bit functions](r:bif.index.bit.bytes), shared with
BITAND and BITXOR.

## Arguments

r[bif.bitor.method]
BITOR is a builtin that runs as a String method, and `string1`, its first
argument, is the receiver; `string2` and `pad` become the message's
arguments, in that order. Its arguments, their checks and their defaults are
those of [the bit-by-bit functions](r:bif.index.bit.args): `string1` is
required, `string2` defaults to the null string, and `pad` has [its own
check](r:bif.index.bit.pad.check) and no default.

```rexx
say c2x(bitor('f0'x, '3c'x)) c2x('f0'x~bitor('3c'x)) c2x('f0'x~bitor('3c'x, '00'x))
signal on syntax name a
say bitor('a', 'b', 'xy')
a: say condition('o')~code
```

```output
FC FC FC
40.23
```

## The result

r[bif.bitor.result]
A bit of the answer is set when the corresponding bit of either operand, or
of both, is set, and clear otherwise, for every character of the answer
that both operands have. The characters that only the longer operand has
follow [the rules for unequal lengths](r:bif.index.bit.nopad) and
[for a pad](r:bif.index.bit.pad).

```rexx
say c2x(bitor('f0'x, '3c'x)) c2x(bitor('00'x, '5a'x)) c2x(bitor('ff'x, '5a'x))
say c2x(bitor('12'x || '34'x, 'f0'x || '0f'x)) c2x(bitor('a1'x, '1a'x))
```

```output
FC 5A FF
F23F BB
```

r[bif.bitor.mask]
A character ORed with `'00'x` is unchanged, and one ORed with `'FF'x`
becomes `'FF'x`. So a `pad` of `'00'x` keeps the characters of the longer
operand that lie past the end of the shorter, and a `pad` of `'FF'x` sets
every bit of them.

```rexx
say c2x(bitor('abc', 'a', '00'x)) c2x(bitor('abc', 'a', 'ff'x))
say bitor('abc', , '00'x) c2x(bitor('abc', , 'ff'x))
```

```output
616263 61FFFF
abc FFFFFF
```

## Deviations

r[bif.bitor.deviation.4]
interrexx does not specify whether the answer of BITOR is the same object
as another string of equal value, as
[the shared rule](r:bif.index.bit.deviation.4)
says for every bit-by-bit function. A program can rely only on the value,
which is equal under `==`.

```rexx
say (bitor('ab', 'ab') == 'ab') (bitor('x', , '00'x) == 'x')
```

```output
1 1
```
