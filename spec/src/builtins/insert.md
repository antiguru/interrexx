---
element: INSERT
kind: builtin
syntax: "INSERT(new, target [, n [, length [, pad]]])"
ansi: "9.3.11"
rexxref: "Functions > Built-in Functions > INSERT"
deviations: [4]
---

# INSERT

INSERT puts a string into another string after a given character, adjusting
the inserted string to a given length, and answers the combined string.

## Arguments

r[bif.insert.method]
INSERT is a builtin that runs as a String method, and `target`, its
*second* argument, is the receiver; `new`, `n`, `length` and `pad` become
the message's arguments, in that order. `new` and `target` are
[strings](r:conv.string.null), `n` is a [non-negative position](r:bif.insert.n.check),
`length` is a [length](r:conv.length.check), and `pad` is a
[pad](r:conv.pad). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): two are required and five
is the most.

```rexx
say insert('123', 'abc', 1) 'abc'~insert('123', 1) '|' 'abc'~insert('123', 1, 2, '.')
signal on syntax name a
say insert('123')
a: say condition('o')~code
signal on syntax name b
say insert('123', 'abc', 1, 1, '.', 1)
b: say condition('o')~code
signal on syntax name c
say insert(, 'abc')
c: say condition('o')~code
signal on syntax name d
say insert('123', , 1)
d: say condition('o')~code
signal on syntax name e
say insert('123', 'abc', 'x')
e: say condition('o')~code
signal on syntax name f
say insert('123', 'abc', 1, 1, '')
f: say condition('o')~code
```

```output
a123bc a123bc | a12bc
40.3
40.4
40.5
40.5
40.12
40.23
```

r[bif.insert.n.check]
In INSERT, `n` is a whole-number argument whose value must not be negative.
A negative value raises `SYNTAX 93.906`. An `n` of 0 is allowed, and so is
`-0`, which is 0. (A [position](r:conv.position.check) elsewhere must be
positive; `n` here counts the characters before the insertion, so 0 is its
first value.) The check is a range check, and falls with those of
[the conventions](r:conv.check.order), from left to right with the check of
`length`.

```rexx
say insert('X', 'abc', 0) insert('X', 'abc', '-0')
signal on syntax name a
say insert('X', 'abc', -1)
a: say condition('o')~code
signal on syntax name b
say insert('X', 'abc', -1, -1)
b: say condition('o')~code
signal on syntax name c
say insert('X', 'abc', 0, -1)
c: say condition('o')~code
signal on syntax name d
say insert('X', 'abc', -1, 'x')
d: say condition('o')~code
signal on syntax name e
say insert('X', 'abc', -1, 1, '**')
e: say condition('o')~code
```

```output
Xabc Xabc
93.906
93.906
93.923
40.12
40.23
```

r[bif.insert.n.default]
When `n` is omitted, it is 0: `new` goes before the first character of
`target`.

```rexx
say insert('123', 'abc') insert('123', 'abc', ) insert('123', 'abc', , 2)
```

```output
123abc 123abc 12abc
```

r[bif.insert.length.default]
When `length` is omitted, it is the length of `new`, so `new` is inserted
whole.

```rexx
say insert('123', 'abc', 1) insert('', 'abc', 1) insert('123', 'abc', 1, )
```

```output
a123bc abc a123bc
```

r[bif.insert.pad.default]
When `pad` is omitted, it is the blank, `'20'x`.

```rexx
say '['insert('1', 'abc', 5, 3)']' '['insert('1', 'abc', 5, 3, )']'
```

```output
[abc  1  ] [abc  1  ]
```

## The result

r[bif.insert.result]
INSERT returns the concatenation of three strings, in this order. The first
is the first `n` characters of `target`, followed by as many copies of `pad`
as it takes to make `n` characters when `target` has fewer. The second is
`new` adjusted to exactly `length` characters: cut at the right when it has
more, and followed by copies of `pad` when it has fewer. The third is the
characters of `target` after position `n`, which is the null string when
`target` has `n` characters or fewer. So the length of the result is `length`
plus the larger of `n` and the length of `target`.

```rexx
say insert('123', 'abc', 3) insert('123', 'abc', 1, 2) insert('123', 'abc', 1, 5, '+')
say insert('123', 'abc', 5, 6, '+') insert('123', 'abc', , 5, '-')
say '['insert('123', 'abc', 1, 0)']' insert('1', 'abc', 4, 3)'.'
```

```output
abc123 a12bc a123++bc
abc++123+++ 123--abc
[abc] abc 1  .
```

r[bif.insert.past-end]
When `n` is greater than the length of `target`, the padding goes to
`target`, not to `new`: `target` is extended on the right with copies of
`pad` to `n` characters before `new` is placed after them, and nothing
follows `new`. This holds when `length` is 0, so the result is then `target`
padded to `n` characters.

```rexx
say insert('X', 'ab', 4, 1, '.') insert('X', 'ab', 3, 0, '.') insert('X', 'ab', 2, 0, '.')
```

```output
ab..X ab. ab
```

r[bif.insert.empty]
When `target` is the null string, the first string is `n` copies of `pad` and
the third is empty, so the result is those copies followed by `new`
adjusted to `length`. When `new` is the null string and `length` is omitted,
the result is `target` padded to `n` characters, and when `length` is
given the null string `new` is padded to that length.

```rexx
say '['insert('X', '', 0)']' '['insert('X', '', 3, 2, '.')']'
say '['insert('', 'ab', 1)']' '['insert('', 'ab', 1, 2, '.')']' '['insert('', '', 3)']'
```

```output
[X] [...X.]
[ab] [a..b] [   ]
```

r[bif.insert.length.zero]
When `length` is 0, the second string is the null string, whatever `new`
is: the result is `target`, padded on the right to `n` characters when `n`
is more than its length. `pad` is still checked, as a [pad](r:conv.pad)
always is.

```rexx
say '['insert('XYZ', 'ab', 1, 0)']' '['insert('XYZ', 'ab', 5, 0, '.')']'
signal on syntax name a
say insert('XYZ', 'ab', 1, 0, '')
a: say condition('o')~code
```

```output
[ab] [ab...]
40.23
```

r[bif.insert.bytes]
INSERT cuts and pads by [character](r:conv.string.chars), one byte each, and
copies every byte of `new` and `target` that stays unchanged: whitespace,
control characters and `'00'x` are characters like any other, in the
strings and as `pad`.

```rexx
say c2x(insert('0061'x, '00'x, 1, 3, '00'x)) c2x(insert('c3a9'x, 'ff'x, 0, 1))
```

```output
00006100 C3FF
```

r[bif.insert.large]
An `n` or a `length` too large for the result to be built raises an error
from the interpreter running out of storage; which values those are is not
specified. A value that is a whole number below `10**18` is otherwise
valid, and a value of `10**18` or more raises `SYNTAX 40.12` like any
[whole number](r:conv.whole.digits) that is too large.

```rexx
signal on syntax name a
say insert('X', 'abc', 1000000000000000000)
a: say condition('o')~code
signal on syntax name b
say insert('X', 'abc', 1, 1000000000000000000)
b: say condition('o')~code
```

```output
40.12
40.12
```

## Deviations

r[bif.insert.deviation.4]
interrexx does not specify whether the result of INSERT is the same object
as another string of equal value: as `target` itself, when the call returns
all of it unchanged as `insert('', s)` does, or as an equal string produced
some other way, such as the literal `'abc'`. Through `identityHash`, or as
a key of an `IdentityTable` or `IdentitySet`, the answer may be either, and
a program can rely only on the value, which is equal under `==`. ooRexx
differs: it often answers a new object where interrexx answers the existing
one.

```rexx
s = 'ab' || 'c'
say (insert('', s) == s) (insert('', s, 3, 0) == 'abc')
```

```output
1 1
```
