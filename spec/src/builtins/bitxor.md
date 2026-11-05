---
element: BITXOR
kind: builtin
syntax: "BITXOR(string1 [, string2] [, pad])"
ansi: "9.6.4"
rexxref: "Functions > Built-in Functions > BITXOR (Bit by Bit Exclusive OR)"
deviations: [4]
---

# BITXOR

BITXOR combines two strings bit by bit, setting a bit where exactly one of the two
strings has it set, and answers the string of the combined characters. How
the two strings are lined up and what happens when their lengths differ are
the rules of [the bit-by-bit functions](r:bif.index.bit.bytes), shared with
BITAND and BITOR.

## Arguments

r[bif.bitxor.method]
BITXOR is a builtin that runs as a String method, and `string1`, its first
argument, is the receiver; `string2` and `pad` become the message's
arguments, in that order. Its arguments, their checks and their defaults are
those of [the bit-by-bit functions](r:bif.index.bit.args): `string1` is
required, `string2` defaults to the null string, and `pad` has [its own
check](r:bif.index.bit.pad.check) and no default.

```rexx
say c2x(bitxor('f0'x, '3c'x)) c2x('f0'x~bitxor('3c'x)) c2x('f0'x~bitxor('3c'x, '00'x))
signal on syntax name a
say bitxor('a', 'b', 'xy')
a: say condition('o')~code
```

```output
CC CC CC
40.23
```

## The result

r[bif.bitxor.result]
A bit of the answer is set when the corresponding bit of exactly one of the
two operands is set, and clear otherwise, for every character of the answer
that both operands have. The characters that only the longer operand has
follow [the rules for unequal lengths](r:bif.index.bit.nopad) and
[for a pad](r:bif.index.bit.pad).

```rexx
say c2x(bitxor('f0'x, '3c'x)) c2x(bitxor('00'x, '5a'x)) c2x(bitxor('ff'x, '5a'x))
say c2x(bitxor('12'x || '34'x, 'f0'x || '0f'x)) c2x(bitxor('a1'x, '1a'x))
```

```output
CC 5A A5
E23B BB
```

r[bif.bitxor.inverse]
A string combined with itself gives `'00'x` for every character, and
combining twice with the same operand, with a `pad` or without one,
restores the string when the operand is not longer than the string. A
character combined with `'FF'x` has every bit flipped, so a `pad` of `'FF'x`
flips the characters of the longer operand that lie past the end of the
shorter.

```rexx
s = 'secret'
say c2x(bitxor(s, s)) (bitxor(bitxor(s, 'key'), 'key') == s),
    (bitxor(bitxor(s, 'key', 'k'), 'key', 'k') == s)
say c2x(bitxor('a0'x, , 'ff'x)) c2x(bitxor('abc', 'a', 'ff'x))
```

```output
000000000000 1 1
5F 009D9C
```

r[bif.bitxor.deviation.4]
interrexx does not specify whether the answer of BITXOR is the same object
as another string of equal value, as [the shared rule](r:bif.index.bit.deviation.4)
says for every bit-by-bit function. A program can rely only on the value,
which is equal under `==`.

```rexx
say (bitxor('ab', '') == 'ab') (bitxor('', 'x', ) == 'x')
```

```output
1 1
```
