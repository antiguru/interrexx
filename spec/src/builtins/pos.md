---
element: POS
kind: builtin
syntax: "POS(needle, haystack [, start [, length]])"
ansi: "9.3.16"
rexxref: "Functions > Built-in Functions > POS (Position)"
deviations: [4]
---

# POS

POS searches a string for another string and answers the position of the
first character of the first occurrence, or `0` when there is none. A
starting position and a length can confine the search to a stretch of the
string. [LASTPOS](lastpos.md) searches from the other end.

## Arguments

r[bif.pos.method]
POS is a builtin that runs as a String method, and `haystack`, its second
argument, is the receiver; `needle`, `start` and `length` become the
message's arguments, in that order. `needle` and `haystack` are
[strings](r:conv.string.null), `start` is a [position](r:conv.position.check)
and `length` is a [length](r:conv.length.check). The arguments are checked in
the [order](r:conv.check.order) the conventions give, and the argument count
by the [builtin call rules](r:bif.index.call.count): two are required and
four is the most.

```rexx
say pos('ter', 'winter') 'winter'~pos('ter')
signal on syntax name a
say pos('a')
a: say condition('o')~code
signal on syntax name b
say pos('a', 'a', 1, 1, 1)
b: say condition('o')~code
signal on syntax name c
say pos(, 'a')
c: say condition('o')~code
signal on syntax name d
say pos('a', 'a', 0)
d: say condition('o')~code
signal on syntax name e
say pos('a', 'a', 1, -1)
e: say condition('o')~code
signal on syntax name f
say pos('a', 'a', 'x')
f: say condition('o')~code
signal on syntax name g
say pos('a', 'a', 0, 'x')
g: say condition('o')~code
signal on syntax name h
say pos('a', 'a', 0, -1)
h: say condition('o')~code
```

```output
4 4
40.3
40.4
40.5
93.924
93.923
40.12
40.12
93.924
```

r[bif.pos.start.default]
When `start` is omitted, it is 1: the search begins at the first character of
`haystack`.

```rexx
say pos('a', 'banana') pos('a', 'banana', ) pos('a', 'banana', 3)
```

```output
2 2 4
```

r[bif.pos.length.default]
When `length` is omitted, the search runs to the end of `haystack`.

```rexx
say pos('a', 'banana', 3) pos('a', 'banana', 3, ) pos('a', 'banana', 3, 100)
```

```output
4 4 4
```

## The result

r[bif.pos.result]
Let `n` be the length of `haystack` and `m` the length of `needle`. The
characters that the search covers are those at the positions from `start`
through `start + length - 1` (through `n` when `length` is omitted), and at no
position beyond `n`; this is the
window, and it holds no characters when `start` is greater than `n` or
`length` is 0. When `m` is at least 1, POS returns the smallest position `k`
such that the `m` characters of `haystack` from position `k` on all lie in
the window and are equal under `==` to `needle`. When there is no such `k`,
the result is `0`. A `needle` with no characters is the case of
[its own rule](r:bif.pos.empty-needle). The answer is
[written as a count](r:bif.index.count.answer), and whether it is a new object
is [not specified](r:bif.pos.deviation.4).

```rexx
say pos('ter', 'winter') pos('q', 'winter') pos('i', 'winter'),
    pos('winter', 'winter')
say pos('aa', 'aaa') pos('aa', 'aaa', 2) pos('ana', 'banana') pos('ana', 'banana', 3)
numeric digits 3
say pos('y', copies('x', 1234) || 'y')
```

```output
4 0 2 1
1 2 2 4
1235
```

r[bif.pos.window]
An occurrence is found only when all of it lies in the window: one that
begins in the window but ends past its last position is not found, and
neither is one that begins before `start`. So `length` bounds where the
occurrence ends as well as where it begins.

```rexx
say pos('an', 'banana', 1, 3) pos('an', 'banana', 1, 2) pos('an', 'banana', 2, 2),
    pos('an', 'banana', 3, 3)
say pos('ana', 'banana', 2, 3) pos('ana', 'banana', 2, 2) pos('ana', 'banana', 3, 5),
    pos('na', 'banana', 5, 2) pos('na', 'banana', 5, 1)
say pos('n', 'banana', 3, 1) pos('n', 'banana', 2, 1) pos('n', 'banana', 1, 2),
    pos('n', 'banana', 1, 3)
```

```output
2 0 2 4
2 0 4 5 0
3 0 0 3
```

r[bif.pos.length.zero]
When `length` is 0, the window holds no characters and the result is `0`,
for every `needle` and `haystack`. `-0` is 0.

```rexx
say pos('a', 'banana', 1, 0) pos('a', 'banana', 2, '-0') pos('', 'banana', 1, 0)
```

```output
0 0 0
```

r[bif.pos.past-end]
When `start` is greater than the length of `haystack`, the result is `0`, for
every `start` that passes its check. A `start` equal to the length of
`haystack` searches the last character alone. A `length` that reaches past the
end of `haystack` is the same as one that reaches exactly to it.

```rexx
say pos('a', 'banana', 6) pos('a', 'banana', 7) pos('a', 'banana', 100),
    pos('a', 'banana', 999999999999999999)
say pos('a', 'banana', 6, 1) pos('a', 'banana', 7, 1) pos('a', 'banana', 6, 100),
    pos('a', 'banana', 1, 999999999999999999) pos('a', '', 1, 100)
```

```output
6 0 0 0
6 0 6 2 0
```

r[bif.pos.empty-needle]
When `needle` has no characters, the result is `0` for every `haystack`, the
null string included. `start` and `length` are still checked, as a position
and a length always are.

```rexx
say pos('', 'banana') pos('', '') pos('', 'banana', 3, 2) pos('', 'banana', 100)
signal on syntax name a
say pos('', 'banana', 0)
a: say condition('o')~code
signal on syntax name b
say pos('', 'banana', 1, -1)
b: say condition('o')~code
```

```output
0 0 0 0
93.924
93.923
```

r[bif.pos.empty-haystack]
When `haystack` is the null string, the result is `0` for every `needle`
and for every `start` and `length` that pass their checks.

```rexx
say pos('a', '') pos('a', '', 1) pos('a', '', 1, 0) pos('', '')
```

```output
0 0 0 0
```

r[bif.pos.exact]
Two characters are equal only when they are the same byte. Case matters,
whitespace in `needle` and `haystack` is compared like any other character,
no numeric comparison is made, and `'00'x` and the other control characters
take part like all the rest. A `needle` longer than `haystack` is never found.

```rexx
say pos('A', 'banana') pos('a b', 'x a b') pos('a  b', 'x a b'),
    pos('a' || '09'x || 'b', 'a b') pos('1', '01') pos('1.0', '1')
say pos('00'x, 'a' || '00'x || 'b') pos('0a'x, 'a' || '0a'x) pos('abc', 'ab')
```

```output
0 3 0 0 2 0
2 2 0
```

r[bif.pos.string-form]
A `needle` or `haystack` that is a number, or an object other than a string,
is [replaced by its string](r:conv.string.object), and POS works on the
characters of that string as they are written.

```rexx
say pos(1, 'a1') pos('.', 3.14) pos('an Object', .object~new)
```

```output
2 2 1
```

## Deviations

r[bif.pos.deviation.4]
interrexx does not specify whether the result of POS is the same object as
another string of equal value: as an equal number the program already holds,
such as the `start` argument of a call that answers it, or a literal such as
`'1235'`, or the equal answer of another call. Through `identityHash`, or as a
key of an `IdentityTable` or `IdentitySet`, the answer may be either, and a
program can rely only on the value, which is equal under `==`. ooRexx differs:
it often answers a new object where interrexx answers the existing one.

```rexx
h = copies('x', 1234) || 'y'
n = 1234 + 1
say (pos('y', h, n) == n) (pos('y', h) == '1235') (pos('y', h) == pos('y', h, 2))
```

```output
1 1 1
```

> **Note.** The ANSI draft (9.3.16) has no `length` argument, and defines POS by
> comparing the needle with each stretch of the haystack of its length from
> `start` on. Without `length` that gives the answers above, `0` for a
> `start` past the last place the needle fits included. It also returns `0` for
> a needle with no characters.
