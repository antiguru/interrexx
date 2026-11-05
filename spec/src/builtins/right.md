---
element: RIGHT
kind: builtin
syntax: "RIGHT(string, length [, pad])"
ansi: "9.3.18"
rexxref: "Functions > Built-in Functions > RIGHT"
deviations: [4]
---

# RIGHT

RIGHT returns a string of a given length made of the rightmost characters of
a string: cut at the left when the string is longer than the length, and
padded at the left when it is shorter.

## Arguments

r[bif.right.method]
RIGHT is a builtin that runs as a String method, and `string`, its first
argument, is the receiver; `length` and `pad` become the message's
arguments, in that order. `string` is a [string](r:conv.string.null),
`length` is a [length](r:conv.length.check), and `pad` is a
[pad](r:conv.pad). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): two are required and three
is the most. Like [LEFT](left.md) and unlike [SUBSTR](substr.md), RIGHT does
not default `length`: omitting it raises `SYNTAX 40.5`.

```rexx
say right('abc  d', 8) '|' 'abc'~right(5, '.')
signal on syntax name a
say right('abc')
a: say condition('o')~code
signal on syntax name b
say right('abc', 2, '.', 1)
b: say condition('o')~code
signal on syntax name c
say right('abc', , '.')
c: say condition('o')~code
signal on syntax name d
say right('abc', 'x')
d: say condition('o')~code
signal on syntax name e
say right('abc', -1)
e: say condition('o')~code
signal on syntax name f
say right('abc', 2, '..')
f: say condition('o')~code
signal on syntax name g
say right('abc', -1, '..')
g: say condition('o')~code
signal on syntax name h
say right()
h: say condition('o')~code
```

```output
  abc  d | ..abc
40.3
40.4
40.5
40.12
93.923
40.23
40.23
40.3
```

r[bif.right.pad.default]
When `pad` is omitted, it is the blank, `'20'x`.

```rexx
say '['right('ab', 4)']' '['right('ab', 4, )']'
```

```output
[  ab] [  ab]
```

## The result

r[bif.right.result]
RIGHT returns a string of exactly `length` characters. When `string` has
`length` characters or more, the result is its last `length` characters.
Otherwise the result is `string` preceded by `length` less `LENGTH(string)`
copies of `pad`.

```rexx
say '['right('abc  d', 8)']' '['right('abc def', 5)']' '['right('12', 5, '0')']'
say '['right('abc', 3)']' '['right('abc', 1)']' '['right('abc', 4, '.')']'
```

```output
[  abc  d] [c def] [00012]
[abc] [c] [.abc]
```

r[bif.right.length.zero]
When `length` is 0, the result is the null string, for every `string`. `pad`
is still checked, as a [pad](r:conv.pad) always is.

```rexx
say '['right('abc', 0)']' '['right('', 0)']' '['right('abc', '-0', '.')']'
signal on syntax name a
say right('abc', 0, '')
a: say condition('o')~code
```

```output
[] [] []
40.23
```

r[bif.right.empty]
When `string` is the null string, the result is `length` copies of `pad`.

```rexx
say '['right('', 3)']' '['right('', 3, '*')']'
```

```output
[   ] [***]
```

r[bif.right.bytes]
RIGHT cuts and pads by [character](r:conv.string.chars), one byte each, and
copies every byte of `string` that stays unchanged: whitespace, control
characters and `'00'x` are characters like any other, in `string` and as
`pad`.

```rexx
say c2x(right('0061'x, 4, '00'x)) c2x(right('c3a9'x, 1))
```

```output
00000061 A9
```

r[bif.right.large]
A `length` too large for the result to be built raises an error from the
interpreter running out of storage; which values those are is not
specified. A `length` that is a whole number below `10**18` is otherwise
valid, and a `length` of `10**18` or more raises `SYNTAX 40.12` like any
[whole number](r:conv.whole.digits) that is too large. The caller's NUMERIC
DIGITS setting plays no part.

```rexx
numeric digits 3
say '['right('abc', 1E1)']'
signal on syntax name a
say right('abc', 1000000000000000000)
a: say condition('o')~code
```

```output
[       abc]
40.12
```

## Deviations

r[bif.right.deviation.4]
interrexx does not specify whether the result of RIGHT is the same object as
another string of equal value: as `string` itself, when the call returns all
of it unchanged as `right(s, length(s))` does, or as an equal string produced
some other way, such as the literal `'abc'`. Through `identityHash`, or as a
key of an `IdentityTable` or `IdentitySet`, the answer may be either, and a
program can rely only on the value, which is equal under `==`. ooRexx
differs: it often answers a new object where interrexx answers the existing
one.

```rexx
s = 'ab' || 'c'
say (right(s, 3) == s) (right('xabc', 3) == 'abc')
```

```output
1 1
```
