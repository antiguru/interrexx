---
element: BITAND
kind: builtin
syntax: "BITAND(string1 [, string2] [, pad])"
ansi: "9.6.2"
rexxref: "Functions > Built-in Functions > BITAND (Bit by Bit AND)"
deviations: [4]
---

# BITAND

BITAND combines two strings bit by bit, keeping a bit set only where both
strings have it set, and answers the string of the combined characters. How
the two strings are lined up and what happens when their lengths differ are
the rules of [the bit-by-bit functions](r:bif.index.bit.bytes), shared with
BITOR and BITXOR.

## Arguments

r[bif.bitand.method]
BITAND is a builtin that runs as a [String method](r:class.string.builtins),
and `string1`, its first argument, is the receiver; `string2` and `pad` become
the message's arguments, in that order. Its arguments, their checks and their
defaults are those of [the bit-by-bit functions](r:bif.index.bit.args):
`string1` is required, `string2` defaults to the null string, and `pad` has
[its own check](r:bif.index.bit.pad.check) and no default.

```rexx
say c2x(bitand('f0'x, '3c'x)) c2x('f0'x~bitand('3c'x)) c2x('f0'x~bitand('3c'x, '00'x))
signal on syntax name a
say bitand('a', 'b', 'xy')
a: say condition('o')~code
```

```output
30 30 30
40.23
```

## The result

r[bif.bitand.result]
A bit of the answer is set when the corresponding bit of both operands is
set, and clear otherwise, for every character of the answer that both
operands have. The characters that only the longer operand has follow
[the rules for unequal lengths](r:bif.index.bit.nopad) and
[for a pad](r:bif.index.bit.pad).

```rexx
say c2x(bitand('f0'x, '3c'x)) c2x(bitand('ff'x, '5a'x)) c2x(bitand('00'x, '5a'x))
say c2x(bitand('7f'x, 'c3'x)) c2x(bitand('12'x || '34'x, 'f0'x || '0f'x))
```

```output
30 5A 00
43 1004
```

r[bif.bitand.mask]
A character ANDed with `'FF'x` is unchanged, and one ANDed with `'00'x`
becomes `'00'x`. So a `pad` of `'FF'x` keeps the characters of the longer
operand that lie past the end of the shorter, and a `pad` of `'00'x` clears
them.

```rexx
say c2x(bitand('abc', 'a', 'ff'x)) c2x(bitand('abc', 'a', '00'x))
say bitand('abc', , 'ff'x) c2x(bitand('abc', , '00'x))
```

```output
616263 610000
abc 000000
```

## Deviations

r[bif.bitand.deviation.4]
interrexx does not specify whether the answer of BITAND is the same object as
another string of equal value, as [the shared
rule](r:bif.index.bit.deviation.4) says for every bit-by-bit function. A
program can rely only on the value, which is equal under `==`.

```rexx
say (bitand('ab', 'ab') == 'ab') (bitand('x', , 'ff'x) == 'x')
```

```output
1 1
```
