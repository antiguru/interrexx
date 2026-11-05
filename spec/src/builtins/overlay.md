---
element: OVERLAY
kind: builtin
syntax: "OVERLAY(new, target [, n [, length [, pad]]])"
ansi: "9.3.15"
rexxref: "Functions > Built-in Functions > OVERLAY"
deviations: [4]
---

# OVERLAY

OVERLAY writes a string over a run of characters of another string, starting
at a given position and adjusting the written string to a given length, and
answers the changed string. The run may reach past the end of the target.

## Arguments

r[bif.overlay.method]
OVERLAY is a builtin that runs as a String method, and `target`, its
*second* argument, is the receiver; `new`, `n`, `length` and `pad` become
the message's arguments, in that order. `new` and `target` are
[strings](r:conv.string.null), `n` is a [position](r:conv.position.check),
`length` is a [length](r:conv.length.check), and `pad` is a
[pad](r:conv.pad). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): two are required and five
is the most.

```rexx
say overlay('XY', 'abcd', 2) 'abcd'~overlay('XY', 2) '|' 'abcd'~overlay('XY', 2, 3, '.')
signal on syntax name a
say overlay('XY')
a: say condition('o')~code
signal on syntax name b
say overlay('XY', 'abcd', 2, 1, '.', 1)
b: say condition('o')~code
signal on syntax name c
say overlay(, 'abcd')
c: say condition('o')~code
signal on syntax name d
say overlay('XY', , 2)
d: say condition('o')~code
signal on syntax name e
say overlay('XY', 'abcd', 0)
e: say condition('o')~code
signal on syntax name f
say overlay('XY', 'abcd', 1, -1)
f: say condition('o')~code
signal on syntax name g
say overlay('XY', 'abcd', 'x')
g: say condition('o')~code
signal on syntax name h
say overlay('XY', 'abcd', 0, -1, '**')
h: say condition('o')~code
signal on syntax name i
say overlay('XY', 'abcd', 0, 'x')
i: say condition('o')~code
```

```output
aXYd aXYd | aXY.
40.3
40.4
40.5
40.5
93.924
93.923
40.12
40.23
40.12
```

r[bif.overlay.n.default]
When `n` is omitted, it is 1: the overlay begins at the first character of
`target`.

```rexx
say overlay('qq', 'abcd') overlay('qq', 'abcd', ) overlay('qq', 'abcd', , 1)
```

```output
qqcd qqcd qbcd
```

r[bif.overlay.length.default]
When `length` is omitted, it is the length of `new`, so `new` is written
whole.

```rexx
say overlay('qq', 'abcd', 2) overlay('qq', 'abcd', 2, ) overlay('', 'abcd', 2)
```

```output
aqqd aqqd abcd
```

r[bif.overlay.pad.default]
When `pad` is omitted, it is the blank, `'20'x`.

```rexx
say '['overlay('1', 'abc', 5, 3)']' '['overlay('1', 'abc', 5, 3, )']'
```

```output
[abc 1  ] [abc 1  ]
```

## The result

r[bif.overlay.result]
OVERLAY returns the concatenation of three strings, in this order. The first
is the first `n - 1` characters of `target`, followed by as many copies of
`pad` as it takes to make `n - 1` characters when `target` has fewer. The
second is `new` adjusted to exactly `length` characters: cut at the right
when it has more, and followed by copies of `pad` when it has fewer. The
third is the characters of `target` from position `n + length` on, which is
the null string when `target` has fewer than that many characters. So the
result keeps every character of `target` outside the run of `length`
characters from position `n`, and the result is longer than `target` when
the run reaches past its end.

```rexx
say overlay(' ', 'abcdef', 3) overlay('.', 'abcdef', 3, 2) overlay('qq', 'abcd', 4)
say overlay('123', 'abc', 5, 6, '+') overlay('xyz', 'abc', 1, 2)
say '['overlay('xyz', 'abc', 2, 5)']'
say overlay('xyz', 'abc', 3) overlay('x', 'abc', 4) overlay('abc', 'abc', 2, 0)
```

```output
ab def ab. ef abcqq
abc+123+++ xyc
[axyz  ]
abxyz abcx abc
```

r[bif.overlay.past-end]
When `n` is greater than one more than the length of `target`, the padding
goes to `target`, not to `new`: `target` is extended on the right with
copies of `pad` to `n - 1` characters before the adjusted `new` follows.
When `n` is exactly one more than the length of `target`, `new` is appended
with no padding between. This holds when `length` is 0, so the result is
then `target` padded to `n - 1` characters.

```rexx
say overlay('xyz', 'abc', 5, 2, '-') overlay('xyz', 'abc', 4, 2, '-') overlay('xyz', 'abc', 5, , '-')
say overlay('xyz', 'abc', 6, 0, '-') overlay('xyz', 'abc', 4, 0, '-')'.'
```

```output
abc-xy abcxy abc-xyz
abc-- abc.
```

r[bif.overlay.length.zero]
When `length` is 0, the second string is the null string, whatever `new`
is, and the third starts at position `n`: so no character of `target` is
replaced, and the result is `target` padded to `n - 1` characters when `n`
is more than one past its end. `pad` is still checked, as a
[pad](r:conv.pad) always is.

```rexx
say '['overlay('XYZ', 'abc', 2, 0)']' '['overlay('XYZ', 'abc', 6, 0, '.')']'
signal on syntax name a
say overlay('XYZ', 'abc', 2, 0, '')
a: say condition('o')~code
```

```output
[abc] [abc..]
40.23
```

r[bif.overlay.empty]
When `target` is the null string, the first string is `n - 1` copies of
`pad` and the result is those copies followed by `new` adjusted to
`length`. When `new` is the null string and `length` is omitted, nothing is
written and the result is `target` padded to `n - 1` characters; when
`length` is given, the null string `new` is padded to that length.

```rexx
say '['overlay('x', '', 1)']' '['overlay('x', '', 3)']' '['overlay('x', '', 3, 3, '-')']'
say '['overlay('', 'abc', 2)']' '['overlay('', 'abc', 2, 2)']' '['overlay('', 'abc', 6)']'
```

```output
[x] [  x] [--x--]
[abc] [a  ] [abc  ]
```

r[bif.overlay.bytes]
OVERLAY cuts and pads by [character](r:conv.string.chars), one byte each,
and copies every byte of `new` and `target` that stays unchanged:
whitespace, control characters and `'00'x` are characters like any other,
in the strings and as `pad`.

```rexx
say c2x(overlay('6100'x, '0000'x, 2, 3, '00'x)) c2x(overlay('c3'x, 'a9a9'x, 2))
```

```output
00610000 A9C3
```

r[bif.overlay.large]
An `n` or a `length` too large for the result to be built raises an error
from the interpreter running out of storage; which values those are is not
specified. A value that is a whole number below `10**18` is otherwise
valid, and a value of `10**18` or more raises `SYNTAX 40.12` like any
[whole number](r:conv.whole.digits) that is too large.

```rexx
signal on syntax name a
say overlay('x', 'abc', 1000000000000000000)
a: say condition('o')~code
signal on syntax name b
say overlay('x', 'abc', 2, 1000000000000000000)
b: say condition('o')~code
```

```output
40.12
40.12
```

## Deviations

r[bif.overlay.deviation.4]
interrexx does not specify whether the result of OVERLAY is the same object
as another string of equal value: as `target` itself, when the call returns
all of it unchanged as `overlay('', s)` does, or as an equal string produced
some other way, such as the literal `'abc'`. Through `identityHash`, or as
a key of an `IdentityTable` or `IdentitySet`, the answer may be either, and
a program can rely only on the value, which is equal under `==`. ooRexx
differs: it often answers a new object where interrexx answers the existing
one.

```rexx
s = 'ab' || 'c'
say (overlay('', s) == s) (overlay('', s, 1, 0) == 'abc')
```

```output
1 1
```
