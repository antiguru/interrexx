---
element: LEFT
kind: builtin
syntax: "LEFT(string, length [, pad])"
ansi: "9.3.13"
rexxref: "Functions > Built-in Functions > LEFT"
deviations: [4]
---

# LEFT

LEFT answers the first `length` characters of a string, adding `pad` on the
right when the string runs out.

## Arguments

r[bif.left.method]
LEFT is a builtin that runs as a String method, and `string`, its first
argument, is the receiver; `length` and `pad` become the message's
arguments, in that order. `string` is a [string](r:conv.string.null),
`length` is a [length](r:conv.length.check), and `pad` is a
[pad](r:conv.pad). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): two are required and three
is the most. Unlike [SUBSTR](substr.md), LEFT does not default `length`: it
is a required argument, so a call without it fails by the
[call rules](r:bif.index.call.too-few), with `SYNTAX 40.3` when nothing
follows its place and `SYNTAX 40.5` when a pad does.

```rexx
say left('hi yo', 7) '|' 'hi yo'~left(7, '.')
signal on syntax name a
say left('abc')
a: say condition('o')~code
signal on syntax name b
say left('abc', 2, '.', 1)
b: say condition('o')~code
signal on syntax name c
say left('abc', , '.')
c: say condition('o')~code
signal on syntax name d
say left('abc', 'x')
d: say condition('o')~code
signal on syntax name e
say left('abc', -1)
e: say condition('o')~code
signal on syntax name f
say left('abc', 2, '..')
f: say condition('o')~code
signal on syntax name g
say left('abc', -1, '..')
g: say condition('o')~code
signal on syntax name h
say left()
h: say condition('o')~code
```

```output
hi yo   | hi yo..
40.3
40.4
40.5
40.12
93.923
40.23
40.23
40.3
```

r[bif.left.pad.default]
When `pad` is omitted, it is the blank, `'20'x`.

```rexx
say '['left('ab', 4)']' '['left('ab', 4, )']'
```

```output
[ab  ] [ab  ]
```

## The result

r[bif.left.result]
LEFT returns a string of exactly `length` characters, the same as
`SUBSTR(string, 1, length, pad)` returns. Its character at position `i`, for
each `i` from 1 to `length`, is the character of `string` at position `i`
when `string` has one, and `pad` otherwise. So the result is the first
`length` characters of `string`, followed by as many copies of `pad` as
make up the length.

```rexx
say '['left('xy  zw', 5)']' '['left('xy  zw', 9, '.')']' '['left('pq rs', 3)']'
say '['left('abc', 3)']' '['left('abc', 1)']' '['left('abc', 5, '*')']'
say (left('abc', 5, '.') == substr('abc', 1, 5, '.')) (left('abc', 2) == substr('abc', 1, 2))
```

```output
[xy  z] [xy  zw...] [pq ]
[abc] [a] [abc**]
1 1
```

r[bif.left.length.zero]
When `length` is 0, the result is the null string, for every `string`. `pad`
is still checked, as a [pad](r:conv.pad) always is.

```rexx
say '['left('abc', 0)']' '['left('', 0)']' '['left('abc', '-0', '.')']'
signal on syntax name a
say left('abc', 0, '')
a: say condition('o')~code
```

```output
[] [] []
40.23
```

r[bif.left.empty]
When `string` is the null string, the result is `length` copies of `pad`.

```rexx
say '['left('', 3)']' '['left('', 3, '*')']'
```

```output
[   ] [***]
```

r[bif.left.bytes]
LEFT cuts and pads by [character](r:conv.string.chars), one byte each, and
copies every byte of `string` that stays unchanged: whitespace, control
characters and `'00'x` are characters like any other, in `string` and as
`pad`.

```rexx
say c2x(left('0061'x, 4, '00'x)) c2x(left('c3a9'x, 1))
```

```output
00610000 C3
```

r[bif.left.large]
A `length` that is a whole number below `10**18` is valid, and a `length`
of `10**18` or more raises `SYNTAX 40.12` like any
[whole number](r:conv.whole.digits) that is too large. The caller's NUMERIC
DIGITS setting plays no part.

```rexx
numeric digits 3
say '['left('abc', 1E1)']'
signal on syntax name a
say left('abc', 1000000000000000000)
a: say condition('o')~code
```

```output
[abc       ]
40.12
```

> **Note.** Whether, when and with which code LEFT fails for a `length`
> whose result cannot be held in memory is not specified.
## Deviations

r[bif.left.deviation.4]
interrexx does not specify whether the result of LEFT is the same object as
another string of equal value: as `string` itself, when the call returns all
of it unchanged as `left(s, length(s))` does, or as an equal string produced
some other way, such as the literal `'abc'`. Through `identityHash`, or as a
key of an `IdentityTable` or `IdentitySet`, the answer may be either, and a
program can rely only on the value, which is equal under `==`. ooRexx
differs: it often answers a new object where interrexx answers the existing
one.

```rexx
s = 'ab' || 'c'
say (left(s, 3) == s) (left('abcde', 3) == 'abc')
```

```output
1 1
```
