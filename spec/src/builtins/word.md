---
element: WORD
kind: builtin
syntax: "WORD(string, n)"
ansi: "9.3.25"
rexxref: "Functions > Built-in Functions > WORD"
deviations: [4]
---

# WORD

WORD selects one word of a string by its number. Words are those of
[the word rule](r:bif.index.words.split).

## Arguments

r[bif.word.method]
WORD is a builtin that runs as a String method, and `string`, its first
argument, is the receiver. `string` is a [string](r:conv.string.null) and
`n` is a [word number](r:bif.index.words.number). The arguments are checked in
the [order](r:conv.check.order) the conventions give, and the argument
count by the [builtin call rules](r:bif.index.call.count): both are required and
two is the most.

```rexx
signal on syntax name a
say word('a b', 0)
a: say condition('o')~code
signal on syntax name b
say word('a b', 'two')
b: say condition('o')~code
signal on syntax name c
say word('a b')
c: say condition('o')~code
signal on syntax name d
say word('a b', 1, 1)
d: say condition('o')~code
```

```output
93.924
40.12
40.3
40.4
```

## The result

r[bif.word.result]
When `string` has a word `n`, WORD returns that word: its characters
exactly as they stand in `string`, and none of the whitespace before or
after it. So then `word(string, n) == subword(string, n, 1)` holds.

```rexx
s = ' red  green' || '09'x || 'blue '
say '['word(s, 2)']' '['word(s, 3)']' (word(s, 2) == subword(s, 2, 1))
```

```output
[green] [blue] 1
```

r[bif.word.past-end]
When `n` is greater than the number of words in `string`, the result is the
null string. This holds for every `n` past the last word that passes its
check.

```rexx
say '['word('red green', 3)']' '['word('red green', 999999999999999999)']'
```

```output
[] []
```

r[bif.word.empty]
When `string` has no words, the null string or a string of only whitespace
among them, the result is the null string for every `n` that passes its
check.

```rexx
say '['word('', 1)']' '['word(' ' || '09'x, 2)']'
```

```output
[] []
```

r[bif.word.nonblank]
A character that [the word rule](r:bif.index.words.split) places inside a word,
such as a carriage return or a line feed, is part of the result, at the
end of the word as anywhere else in it.

```rexx
say c2x(word('x' || '0d'x || ' y', 1)) c2x(word('p q' || '0a'x || 'r', 2))
```

```output
780D 710A72
```

## Deviations

r[bif.word.deviation.4]
interrexx does not specify whether the answer of WORD is the same object as
another string of equal value: as `string` itself, when the call returns all
of it unchanged as `word(s, 1)` does for an `s` that is one word with no
whitespace around it, or as an equal string produced some other way, such as
the literal `'abc'` beside `word('abc def', 1)`. Through `identityHash`, or as
a key of an `IdentityTable`, the answer may be either, and a program can rely
only on the value, which is equal under `==`. ooRexx differs: it often answers
a new object where interrexx answers the existing one.

```rexx
s = 'ab' || 'c'
say (word(s, 1) == s) (word('abc def', 1) == 'abc')
```

```output
1 1
```
