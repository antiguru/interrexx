---
element: SUBSTR
kind: builtin
syntax: "SUBSTR(string, n [, length] [, pad])"
ansi: "9.3.21"
rexxref: "Functions > Built-in Functions > SUBSTR (Substring)"
deviations: [4]
---

# SUBSTR

SUBSTR selects a run of characters from a string by position, and pads the
run when it reaches past the end of the string.

## Arguments

r[bif.substr.method]
SUBSTR is a builtin that runs as a String method, and `string`, its first
argument, is the receiver. `string` is a [string](r:conv.string.null), `n`
is a [position](r:conv.position.check), `length` is a
[length](r:conv.length.check), and `pad` is a [pad](r:conv.pad). The
arguments are checked in the [order](r:conv.check.order) the conventions
give, and the argument count by the [builtin call rules](r:bif.call.count):
two are required and four is the most.

```rexx
signal on syntax name a
say substr('abc', 0)
a: say condition('o')~code
signal on syntax name b
say substr('abc', 2, -1)
b: say condition('o')~code
signal on syntax name c
say substr('abc', 1, 2, '..')
c: say condition('o')~code
signal on syntax name d
say substr('abc', 1, 1, '.', 1)
d: say condition('o')~code
```

```output
93.924
93.923
40.23
40.4
```

r[bif.substr.length.default]
When `length` is omitted, it is the number of characters of `string` at
position `n` and after it: `LENGTH(string) - n + 1` when `n` is at most
`LENGTH(string)`, and 0 otherwise.

```rexx
say '['substr('abcde', 2)']' '['substr('abcde', 5)']',
    '['substr('abcde', 6)']' '['substr('abcde', 100)']'
```

```output
[bcde] [e] [] []
```

r[bif.substr.pad.default]
When `pad` is omitted, it is the blank, `'20'x`.

```rexx
say '['substr('abc', 2, 4)']'
```

```output
[bc  ]
```

## The result

r[bif.substr.result]
SUBSTR returns a string of exactly `length` characters. Its character at
position `i`, for each `i` from 1 to `length`, is the character of `string`
at position `n + i - 1` when `string` has a character there, and `pad`
otherwise. So the result is the characters of `string` from position `n`
on, at most `length` of them, followed by as many copies of `pad` as make
up the length.

```rexx
say '['substr('abcdef', 2, 3)']' '['substr('abcdef', 4, 5, '.')']',
    '['substr('abcdef', 1, 6)']'
```

```output
[bcd] [def..] [abcdef]
```

r[bif.substr.past-end]
When `n` is greater than `LENGTH(string)`, no character of `string` is
selected: the result is `length` copies of `pad`, and the null string when
`length` is omitted. This holds for every `n` past the end, however large.

```rexx
say '['substr('abc', 4, 2, '*')']' '['substr('abc', 999999999999999999, 3, '*')']',
    '['substr('abc', 4)']'
```

```output
[**] [***] []
```

r[bif.substr.length.zero]
When `length` is 0, the result is the null string, whatever `n` is. `pad`
is still checked, as a [pad](r:conv.pad) always is.

```rexx
say '['substr('abc', 2, 0)']' '['substr('abc', 10, 0, '*')']'
signal on syntax name e
say substr('abc', 2, 0, '')
e: say condition('o')~code
```

```output
[] []
40.23
```

r[bif.substr.empty]
When `string` is the null string, every position is past its end, so the
result is `length` copies of `pad`, or the null string when `length` is
omitted.

```rexx
say '['substr('', 1)']' '['substr('', 1, 3)']' '['substr('', 5, 2, '-')']'
```

```output
[] [   ] [--]
```

r[bif.substr.bytes]
SUBSTR selects and pads by [character](r:conv.string.chars), one byte each,
and copies every byte unchanged: whitespace, control characters and
`'00'x` are characters like any other, in `string` and as `pad`.

```rexx
say c2x(substr('61000962'x, 2, 4, '00'x))
```

```output
00096200
```

## Deviations

r[bif.substr.deviation.4]
When SUBSTR returns all of `string` unchanged, as `substr(s, 1)` does,
interrexx does not specify whether the result is the same object as
`string` or another object with the same value; a program can rely only on
the value, which is equal under `==`. Through `identityHash`, or as a key of
an `IdentityTable` or `IdentitySet`, the answer may be either. The oracle
differs: it returns a new object each time, so there the two are always
distinct objects.

```rexx
s = 'abc' || 'def'
say substr(s, 1) == s
```

```output
1
```
