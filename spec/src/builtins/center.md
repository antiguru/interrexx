---
element: CENTER
kind: builtin
syntax: "CENTER(string, length [, pad])"
ansi: "9.3.2"
rexxref: "Functions > Built-in Functions > CENTER (or CENTRE)"
deviations: [4]
---

# CENTER

CENTER returns a string of a given length with a string placed in the middle
of it: padded on both sides when the string is shorter than the length, and
cut at both ends when it is longer. CENTRE is another spelling of the same
builtin.

## Arguments

r[bif.center.method]
CENTER is a builtin that runs as a String method, and `string`, its first
argument, is the receiver; `length` and `pad` become the message's
arguments, in that order. `string` is a [string](r:conv.string.null),
`length` is a [length](r:conv.length.check), and `pad` is a
[pad](r:conv.pad). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): two are required and three
is the most.

```rexx
say center('pqr', 9) '|' 'pqr'~center(9, '.')
signal on syntax name a
say center('abc')
a: say condition('o')~code
signal on syntax name b
say center('abc', 5, '*', 1)
b: say condition('o')~code
signal on syntax name c
say center('abc', , '*')
c: say condition('o')~code
signal on syntax name d
say center('abc', 'x')
d: say condition('o')~code
signal on syntax name e
say center('abc', -1)
e: say condition('o')~code
signal on syntax name f
say center('abc', 5, '**')
f: say condition('o')~code
signal on syntax name g
say center('abc', -1, '**')
g: say condition('o')~code
signal on syntax name h
say center('abc', 1000000000000000000)
h: say condition('o')~code
```

```output
   pqr    | ...pqr...
40.3
40.4
40.5
40.12
93.923
40.23
40.23
40.12
```

r[bif.center.centre]
CENTRE is the same builtin as CENTER: it takes the same arguments, runs as
the String method of that name, and gives the same result and the same
errors for the same arguments.

```rexx
say centre('abc', 7, '.') 'abc'~centre(7, '.') center('abc', 7, '.')
signal on syntax name a
say centre('abc', -1)
a: say condition('o')~code
```

```output
..abc.. ..abc.. ..abc..
93.923
```

r[bif.center.pad.default]
When `pad` is omitted, it is the blank, `'20'x`.

```rexx
say '['center('ab', 6)']' '['center('ab', 6, )']'
```

```output
[  ab  ] [  ab  ]
```

## The result

r[bif.center.result]
CENTER returns a string of exactly `length` characters. Let `d` be
`length` less `LENGTH(string)`. When `d` is 0 the result is `string`. When
`d` is positive, the result is `string` with `d` copies of `pad` added,
`d` divided by 2 and rounded down on the left and the rest on the right, so
that the right-hand end gains the extra one when `d` is odd. When `d` is
negative, the result is `string` with its first `-d` divided by 2 (rounded down)
characters and its last `-d` less that many characters removed, so the
right-hand end loses the extra one when `-d` is odd.

```rexx
say '['center('xyz', 9)']' '['center('xyz', 10, '-')']' '['center('a', 4, '*')']'
say '['center('xyz', 3)']' '['center('Red green blue', 10)']' '['center('Red green blue', 9)']'
say '['center('abcde', 2)']' '['center('abcdef', 1)']' '['center('abcd', 3)']'
```

```output
[   xyz   ] [---xyz----] [*a**]
[xyz] [d green bl] [d green b]
[bc] [c] [abc]
```

r[bif.center.length.zero]
When `length` is 0, the result is the null string, for every `string`.
`pad` is still checked, as a [pad](r:conv.pad) always is.

```rexx
say '['center('abc', 0)']' '['center('', 0)']' '['center('abc', '-0', '.')']'
signal on syntax name a
say center('abc', 0, '')
a: say condition('o')~code
```

```output
[] [] []
40.23
```

r[bif.center.empty]
When `string` is the null string, the result is `length` copies of `pad`.

```rexx
say '['center('', 4)']' '['center('', 3, '*')']'
```

```output
[    ] [***]
```

r[bif.center.bytes]
CENTER pads and cuts by [character](r:conv.string.chars), one byte each,
and copies unchanged every byte of `string` that it keeps: whitespace, control
characters and `'00'x` are characters like any other, in `string` and as
`pad`.

```rexx
say c2x(center('610062'x, 6, '00'x)) c2x(center(' a ', 1))
```

```output
006100620000 61
```

> **Note.** Whether, when and with which code CENTER fails for a `length`
> whose result cannot be held in memory is not specified.

## Deviations

r[bif.center.deviation.4]
interrexx does not specify whether the result of CENTER is the same object
as another string of equal value: as `string` itself, when the call returns
all of it unchanged as `center(s, length(s))` does, or as an equal string
produced some other way, such as the literal `'abc'`. Through
`identityHash`, or as a key of an `IdentityTable` or `IdentitySet`, the
answer may be either, and a program can rely only on the value, which is
equal under `==`. ooRexx differs: it often answers a new object where
interrexx answers the existing one.

```rexx
s = 'ab' || 'c'
say (center(s, 3) == s) (center('xabcx', 3) == 'abc')
```

```output
1 1
```
