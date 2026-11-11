---
element: WORDINDEX
kind: builtin
syntax: "WORDINDEX(string, n)"
ansi: "9.3.26"
rexxref: "Functions > Built-in Functions > WORDINDEX"
---

# WORDINDEX

WORDINDEX answers where in a string one of its words begins, as a
character position. Words are those of [the word rule](r:bif.index.words.split).

## Arguments

r[bif.wordindex.method]
WORDINDEX is a builtin that runs as a [String
method](r:class.string.builtins), and `string`, its first argument, is the
receiver. `string` is a [string](r:conv.string.null) and `n` is a [word
number](r:bif.index.words.number). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): both are required and two is
the most.

```rexx
signal on syntax name a
say wordindex('a b', -2)
a: say condition('o')~code
signal on syntax name b
say wordindex('a b', '')
b: say condition('o')~code
signal on syntax name c
say wordindex(, 1)
c: say condition('o')~code
signal on syntax name d
say wordindex('a b', 1, 1)
d: say condition('o')~code
```

```output
93.924
40.12
40.5
40.4
```

## The result

r[bif.wordindex.result]
When `string` has a word `n`, WORDINDEX returns the
[position](r:conv.position.origin) in `string` of the first character of that
word. The position counts characters of `string` from its first, so whitespace
before the word, and the words and whitespace before that, all count toward
it. The answer is [written as a count](r:bif.index.count.answer), and whether
it is a new object is [not specified](r:bif.index.count.answer.deviation.4).

```rexx
say wordindex('red  green blue', 2) wordindex('  red', 1),
    wordindex('09'x || 'red', 1) wordindex('a  bb  ccc', 3)
```

```output
6 3 2 8
```

r[bif.wordindex.past-end]
When `n` is greater than the number of words in `string`, the result is
`0`. This holds for every `n` past the last word that passes its check.

```rexx
say wordindex('red green', 3) wordindex('red green', 999999999999999999)
```

```output
0 0
```

r[bif.wordindex.empty]
When `string` has no words, the null string or a string of only whitespace
among them, the result is `0` for every `n` that passes its check.

```rexx
say wordindex('', 1) wordindex('  ' || '09'x, 1)
```

```output
0 0
```
