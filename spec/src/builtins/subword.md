---
element: SUBWORD
kind: builtin
syntax: "SUBWORD(string, n [, length])"
ansi: "9.3.22"
rexxref: "Functions > Built-in Functions > SUBWORD"
deviations: [4]
---

# SUBWORD

SUBWORD selects a run of consecutive words from a string, keeping the
whitespace between them as it is. Words are those of
[the word rule](r:bif.words.split).

## Arguments

r[bif.subword.method]
SUBWORD is a builtin that runs as a String method, and `string`, its first
argument, is the receiver. `string` is a [string](r:conv.string.null), `n`
is a [position](r:conv.position.check) that counts words, and `length` is a
[length](r:conv.length.check) that counts words. The arguments are checked
in the [order](r:conv.check.order) the conventions give, and the argument
count by the [builtin call rules](r:bif.call.count): two are required and
three is the most.

```rexx
signal on syntax name a
say subword('a b', 0)
a: say condition('o')~code
signal on syntax name b
say subword('a b', 1, -1)
b: say condition('o')~code
signal on syntax name c
say subword('a b', 0, 'x')
c: say condition('o')~code
signal on syntax name d
say subword('a b', 1, 1, 1)
d: say condition('o')~code
```

```output
93.924
93.923
40.12
40.4
```

r[bif.subword.length.default]
When `length` is omitted, it selects every word from word `n` to the last
word of `string`.

```rexx
say '['subword('one two  three', 2)']' '['subword('one two  three', 3)']'
```

```output
[two  three] [three]
```

## The result

r[bif.subword.result]
SUBWORD returns the part of `string` that begins with the first character
of word `n` and ends with the last character of word `n + length - 1`, or
of the last word of `string` when it has fewer words than that. It is a
slice of `string` and nothing is rewritten in it, so a run of blanks or
tabs separating two selected words keeps its length and its characters.
Since the slice starts and ends on word characters, whitespace before word
`n`, and after the last selected word, falls outside it.

```rexx
say '['subword('one two  three   four', 2, 2)']'
say '['subword('  one  two  ', 1)']' '['subword('a b c', 2, 100)']'
say c2x(subword('09'x||'a'||'0909'x||'b'||'09'x, 1))
```

```output
[two  three]
[one  two] [b c]
61090962
```

r[bif.subword.past-end]
When `n` is greater than the number of words in `string`, the result is the
null string. This holds for every `n` past the last word, however large.

```rexx
say '['subword('a b c', 4)']' '['subword('a b c', 999999999999999999, 2)']'
```

```output
[] []
```

r[bif.subword.length.zero]
When `length` is 0, the result is the null string, whatever `n` is.

```rexx
say '['subword('a b c', 2, 0)']' '['subword('a b c', 9, 0)']'
```

```output
[] []
```

r[bif.subword.empty]
When `string` has no words, the null string or a string of only whitespace
among them, the result is the null string for every `n` and `length` that
pass their checks.

```rexx
say '['subword('', 1)']' '['subword(' '||'09'x||' ', 1, 2)']'
```

```output
[] []
```

r[bif.subword.nonblank]
A character that [the word rule](r:bif.words.split) places inside a word,
such as a line feed or a carriage return, is copied into the result as it
stands, and SUBWORD ends the result after it when it is the last character
of the last selected word.

```rexx
say c2x(subword('a'||'0a'x||'b c', 1, 1)) c2x(subword('a'||'0d'x||' b', 1, 1))
```

```output
610A62 610D
```

## Deviations

r[bif.subword.deviation.4]
interrexx does not specify whether the result of SUBWORD is the same object
as another string of equal value: as `string` itself, when the call returns
all of it unchanged as `subword(w, 1)` does for a `w` with no whitespace
around its words, or as an equal string produced some other way, such as
the literal `'a b'` beside `subword('a b c', 1, 2)`. Through
`identityHash`, or as a key of an `IdentityTable` or `IdentitySet`, the
answer may be either, and a program can rely only on the value, which is
equal under `==`. The oracle differs: it often answers a new object where
interrexx answers the existing one.

```rexx
w = 'a' || ' b'
say (subword(w, 1) == w) (subword('a b c', 1, 2) == 'a b')
```

```output
1 1
```
