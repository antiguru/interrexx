---
element: LASTPOS
kind: builtin
syntax: "LASTPOS(needle, haystack [, start [, length]])"
ansi: "9.3.12"
rexxref: "Functions > Built-in Functions > LASTPOS (Last Position)"
deviations: [4]
---

# LASTPOS

LASTPOS searches a string for another string, from the end toward the
beginning, and answers the position of the first character of the last
occurrence, or `0` when there is none. A starting position and a length can
confine the search to a stretch of the string. [POS](pos.md) searches from
the front.

## Arguments

r[bif.lastpos.method]
LASTPOS is a builtin that runs as a String method, and `haystack`, its second
argument, is the receiver; `needle`, `start` and `length` become the
message's arguments, in that order. `needle` and `haystack` are
[strings](r:conv.string.null), `start` is a [position](r:conv.position.check)
and `length` is a [length](r:conv.length.check). The arguments are checked in
the [order](r:conv.check.order) the conventions give, and the argument count
by the [builtin call rules](r:bif.index.call.count): two are required and
four is the most.

```rexx
say lastpos('/', 'a/b/c') 'a/b/c'~lastpos('/')
signal on syntax name a
say lastpos('a')
a: say condition('o')~code
signal on syntax name b
say lastpos('a', 'a', 1, 1, 1)
b: say condition('o')~code
signal on syntax name c
say lastpos(, 'a')
c: say condition('o')~code
signal on syntax name d
say lastpos('a', 'a', 0)
d: say condition('o')~code
signal on syntax name e
say lastpos('a', 'a', 1, -1)
e: say condition('o')~code
signal on syntax name f
say lastpos('a', 'a', 'x')
f: say condition('o')~code
signal on syntax name g
say lastpos('a', 'a', 0, 'x')
g: say condition('o')~code
signal on syntax name h
say lastpos('a', 'a', 0, -1)
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

r[bif.lastpos.start.default]
`start` is the length of `haystack` both when it is omitted and when it is
greater than that length: the search begins at the last character either way.

```rexx
say lastpos('a', 'banana') lastpos('a', 'banana', ) lastpos('a', 'banana', 6),
    lastpos('a', 'banana', 7) lastpos('a', 'banana', 999999999999999999)
```

```output
6 6 6 6 6
```

r[bif.lastpos.length.default]
When `length` is omitted, the search runs back to the first character of
`haystack`.

```rexx
say lastpos('b', 'banana', 6) lastpos('b', 'banana', 6, ) lastpos('b', 'banana', 6, 100)
```

```output
1 1 1
```

## The result

r[bif.lastpos.result]
Let `n` be the length of `haystack` and `m` the length of `needle`. Let `e` be
`start`, or `n` when `start` is omitted or greater than `n`. The characters
that the search covers are those at the positions from `e - length + 1` (or
from 1, when that is less, or when `length` is omitted) through `e`; this is
the window, and it holds no characters when `length` is 0. When `m` is at
least 1, LASTPOS returns the greatest position `k` such that the `m`
characters of `haystack` from position `k` on all lie in the window and are
equal under `==` to `needle`. When there is no such `k`, the result is `0`. A
`needle` with no characters is the case of [its own
rule](r:bif.lastpos.empty-needle). The answer is [written as a
count](r:bif.index.count.answer), and whether it is a new object is [not
specified](r:bif.lastpos.deviation.4).

```rexx
say lastpos('/', 'a/b/c') lastpos('/', 'abc') lastpos('ana', 'banana'),
    lastpos('aa', 'aaa') lastpos('an', 'banana')
numeric digits 3
say lastpos('x', copies('x', 1234) || 'y')
```

```output
4 0 4 2 4
1234
```

r[bif.lastpos.window]
An occurrence is found only when all of it lies in the window: a `start`
before the end of an occurrence hides it, even when the occurrence begins at
or before `start`, and one that begins before the window is not found. A
`length` that would reach before the first character is cut off there, and
`start` is limited to `n` before `length` is applied.

```rexx
say lastpos('an', 'banana', 5) lastpos('an', 'banana', 4) lastpos('an', 'banana', 3),
    lastpos('an', 'banana', 2)
say lastpos('ana', 'banana', 6, 3) lastpos('ana', 'banana', 6, 2),
    lastpos('ana', 'banana', 5, 3) lastpos('ban', 'banana', 3, 2)
say lastpos('n', 'banana', 4, 2) lastpos('n', 'banana', 4, 1) lastpos('n', 'banana', 6, 3),
    lastpos('b', 'banana', 8, 6) lastpos('a', 'banana', 3, 100)
```

```output
4 2 2 0
4 0 0 0
3 0 5 1 2
```

r[bif.lastpos.length.zero]
When `length` is 0, the window holds no characters and the result is `0`,
for every `needle` and `haystack`. `-0` is 0.

```rexx
say lastpos('a', 'banana', 6, 0) lastpos('a', 'banana', , 0),
    lastpos('a', 'banana', 6, '-0')
```

```output
0 0 0
```

r[bif.lastpos.empty-needle]
When `needle` has no characters, the result is `0` for every `haystack`, the
null string included. `start` and `length` are still checked, as a position
and a length always are.

```rexx
say lastpos('', 'banana') lastpos('', '') lastpos('', 'banana', 3, 2)
signal on syntax name a
say lastpos('', 'banana', 0)
a: say condition('o')~code
signal on syntax name b
say lastpos('', 'banana', 1, -1)
b: say condition('o')~code
```

```output
0 0 0
93.924
93.923
```

r[bif.lastpos.empty-haystack]
When `haystack` is the null string, the result is `0` for every `needle` and
for every `start` and `length` that pass their checks.

```rexx
say lastpos('a', '') lastpos('a', '', 1) lastpos('a', '', 5, 3) lastpos('', '')
```

```output
0 0 0 0
```

r[bif.lastpos.exact]
Two characters are equal only when they are the same byte. Case matters,
whitespace in `needle` and `haystack` is compared like any other character,
no numeric comparison is made, and `'00'x` and the other control characters
take part like all the rest. A `needle` longer than `haystack` is never found,
and neither is one that is longer than the window.

```rexx
say lastpos('A', 'banana') lastpos('a b', 'a b a b') lastpos('a  b', 'a b'),
    lastpos('1', '1 01') lastpos('abc', 'ab') lastpos('ab', 'abab', 3, 2)
say lastpos('00'x, '00'x || 'a' || '00'x || 'b') lastpos('0a'x, '0a'x || 'a')
```

```output
0 5 0 4 0 0
3 1
```

r[bif.lastpos.string-form]
A `needle` or `haystack` that is a number, or an object other than a string,
is [replaced by its string](r:conv.string.object), and LASTPOS works on the
characters of that string as they are written.

```rexx
say lastpos(1, '1a1') lastpos('.', 3.14) lastpos('an Object', .object~new)
```

```output
3 2 1
```

## Deviations

r[bif.lastpos.deviation.4]
interrexx does not specify whether the result of LASTPOS is the same object as
another string of equal value: as an equal number the program already holds,
such as the `start` argument of a call that answers it, or a literal such as
`'1234'`, or the equal answer of another call. Through `identityHash`, or as a
key of an `IdentityTable` or `IdentitySet`, the answer may be either, and a
program can rely only on the value, which is equal under `==`. ooRexx differs:
it often answers a new object where interrexx answers the existing one.

```rexx
h = copies('x', 1234) || 'y'
n = 1233 + 1
say (lastpos('x', h, n) == n) (lastpos('x', h) == '1234') (lastpos('x', h) == lastpos('x', h, n))
```

```output
1 1 1
```

> **Note.** The ANSI draft (9.3.12) has no `length` argument, and defines
> LASTPOS by comparing the needle with each stretch of the haystack of its
> length that ends at or before `start`, working backward. It does not limit
> `start` to the length of the haystack: it reads the places past the end as
> blanks, so a needle of only blanks can be found there. ooRexx limits `start`
> first, as this book does.
