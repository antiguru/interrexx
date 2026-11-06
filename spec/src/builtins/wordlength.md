---
element: WORDLENGTH
kind: builtin
syntax: "WORDLENGTH(string, n)"
ansi: "9.3.27"
rexxref: "Functions > Built-in Functions > WORDLENGTH"
---

# WORDLENGTH

WORDLENGTH answers how many characters one word of a string has. Words are
those of [the word rule](r:bif.index.words.split).

## Arguments

r[bif.wordlength.method]
WORDLENGTH is a builtin that runs as a String method, and `string`, its first
argument, is the receiver. `string` is a [string](r:conv.string.null) and `n`
is a [word number](r:bif.index.words.number). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): both are required and two is
the most.

```rexx
signal on syntax name a
say wordlength('a b', 0)
a: say condition('o')~code
signal on syntax name b
say wordlength('a b', 1.5)
b: say condition('o')~code
signal on syntax name c
say wordlength('a b', )
c: say condition('o')~code
signal on syntax name d
say wordlength('a b', 1, 2)
d: say condition('o')~code
```

```output
93.924
40.12
40.3
40.4
```

## The result

r[bif.wordlength.result]
When `string` has a word `n`, WORDLENGTH returns the number of
[characters](r:conv.string.chars) in that word, which is
`LENGTH(WORD(string, n))`. Whitespace around the word does not count, and a
character that [the word rule](r:bif.index.words.split) places inside a word,
such as a line feed, counts as one. The answer is [written as a
count](r:bif.index.count.answer), and whether it is a new object is [not
specified](r:bif.index.count.answer.deviation.4).

```rexx
say wordlength('red  green blue', 2) wordlength('  abc  ', 1),
    wordlength(' x' || '0a'x || 'y ', 1)
```

```output
5 3 3
```

r[bif.wordlength.past-end]
When `n` is greater than the number of words in `string`, the result is
`0`. This holds for every `n` past the last word that passes its check.

```rexx
say wordlength('red green', 3) wordlength('red green', 999999999999999999)
```

```output
0 0
```

r[bif.wordlength.empty]
When `string` has no words, the null string or a string of only whitespace
among them, the result is `0` for every `n` that passes its check.

```rexx
say wordlength('', 1) wordlength('09'x || ' ', 1)
```

```output
0 0
```
