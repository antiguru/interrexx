---
element: SPACE
kind: builtin
syntax: "SPACE(string [, n] [, pad])"
ansi: "9.3.19"
rexxref: "Functions > Built-in Functions > SPACE"
deviations: [4]
---

# SPACE

SPACE rebuilds a string from its words with the same fill between each two
of them, and none before the first or after the last. Words are those of
[the word rule](r:bif.index.words.split).

## Arguments

r[bif.space.method]
SPACE is a builtin that runs as a String method, and `string`, its first
argument, is the receiver; `n` and `pad` become the message's arguments, in
that order. `string` is a [string](r:conv.string.null), `n` is a
[length](r:conv.length.check), the number of fill characters between two
words, and `pad` is a [pad](r:conv.pad). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): one is required and three is
the most.

```rexx
say space('a  b') 'a  b'~space(2, '.')
signal on syntax name a
say space('a b', -1)
a: say condition('o')~code
signal on syntax name b
say space('a b', 'two')
b: say condition('o')~code
signal on syntax name c
say space('a b', 1, '..')
c: say condition('o')~code
signal on syntax name d
say space('a b', 1, '.', 1)
d: say condition('o')~code
signal on syntax name f
say space(, 2)
f: say condition('o')~code
```

```output
a b a..b
93.923
40.12
40.23
40.4
40.5
```

r[bif.space.n.default]
When `n` is omitted, it is 1: one fill character goes between each two
words.

```rexx
say space('a   b    c') space('a b c', , '+')
```

```output
a b c a+b+c
```

r[bif.space.pad.default]
When `pad` is omitted, it is the blank, `'20'x`, whatever `n` is, and an
omitted `n` does not change that.

```rexx
say space('a b c', 2) space('a b c', 0) space('a b c', , '+')
```

```output
a  b  c abc a+b+c
```

## The result

r[bif.space.result]
When `string` has at least one word, SPACE returns its words in order,
each unchanged, with each pair of neighbouring words separated by `n`
copies of `pad`. There is nothing before the first word and nothing after
the last. The characters of a word are copied as they are, including any
that [the word rule](r:bif.index.words.split) places inside it, such as a
line feed. The whitespace of `string` is not part of the result, whether it
was blanks or tabs and however long it ran.

```rexx
say '['space('  one two' || '09'x || '09'x || 'three ', 3, '-')']'
say '['space('a' || '0a'x || 'b  c')']'
say c2x(space('x' || '00'x || 'y  z', 2, '00'x))
```

```output
[one---two---three]
[a
b c]
78007900007A
```

r[bif.space.zero]
When `n` is 0, no fill is put between the words, and the result is the
words joined. `pad` plays no part, but it is still checked.

```rexx
say space(' a b  c ', 0) space(' a b  c ', 0, '*')
signal on syntax name a
say space(' a b ', 0, '**')
a: say condition('o')~code
```

```output
abc abc
40.23
```

r[bif.space.pad]
`pad` may be any one character, the blank and the tab included, and a
pad that is whitespace is only a fill: it does not make the words of the
result differ from those of `string`.

```rexx
say '['space('a  b', 2, ' ')']' c2x(space('a b', 2, '09'x))
say words(space('a b c', 3, '09'x))
```

```output
[a  b] 61090962
3
```

r[bif.space.few-words]
When `string` has no word, the result is the null string, and when it has
exactly one word, the result is that word without its surrounding
whitespace. There is no between-words fill then, so `n` and `pad` change
nothing, however large `n` is, but both are still checked.

```rexx
say '['space('')']' '['space('  ' || '09'x, 5, '*')']' '['space('  w  ', 5, '*')']'
say '['space('w', 999999999999999999)']'
signal on syntax name a
say space('', 1, 'ab')
a: say condition('o')~code
signal on syntax name b
say space('w', 'x')
b: say condition('o')~code
signal on syntax name c
say space('', -1)
c: say condition('o')~code
```

```output
[] [] [w]
[w]
40.23
40.12
93.923
```

r[bif.space.check-order]
Of the checks that `n` and `pad` can fail, the form checks of `n` and of
`pad` come first, from left to right, and only then the range check of `n`,
as [the order of the checks](r:conv.check.order) gives: a negative `n` with a
bad `pad` raises the `pad` error, and an `n` that is not a number raises its
own error before a bad `pad` is looked at.

```rexx
signal on syntax name a
say space('a b', -1, '**')
a: say condition('o')~code
signal on syntax name b
say space('a b', 'x', '**')
b: say condition('o')~code
signal on syntax name c
say space('a b', -1, '.')
c: say condition('o')~code
```

```output
40.23
40.12
93.923
```

> **Note.** A result of many characters needs memory for them; a request
> larger than the machine can serve ends in an error that this book does not
> specify. The ANSI draft (9.3.19) defines SPACE through `COPIES` and
> `SUBWORD`, with the same answers, and checks `n` for a value below 0 under
> error 40.13, where ooRexx raises 93.923.

## Deviations

> **Note.** An argument whose `STRING` method answers an object that is not a
> string falls under [Deviation 30](r:conv.string.deviation.30), as it does
> for every builtin.

r[bif.space.deviation.4]
interrexx does not specify whether the result of SPACE is the same object
as another string of equal value: as `string` itself, when it is already
in the form SPACE gives it, as in `space(s)` for an `s` of one blank
between each word and none outside, or as an equal string produced some
other way, such as the literal `'x y'` beside `space('x  y')`. Through
`identityHash`, or as a key of an `IdentityTable` or `IdentitySet`, the
answer may be either, and a program can rely only on the value, which is
equal under `==`. ooRexx differs: it often answers a new object where
interrexx answers the existing one.

```rexx
s = 'p' || ' ' || 'q'
say (space(s) == s) (space('x  y') == 'x y')
```

```output
1 1
```
