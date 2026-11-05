---
element: SUBWORD
kind: builtin
syntax: "SUBWORD(string, n [, length])"
ansi: "9.3.22"
rexxref: "Functions > Built-in Functions > SUBWORD"
deviations: []
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
of the last word of `string` when it has fewer words than that. The
characters in between, the whitespace between the selected words included,
are copied unchanged: a run of blanks or tabs stays as long as it was, and
a tab stays a tab. The result therefore never begins or ends with
whitespace, whatever whitespace `string` has around the selected words.

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
Only blank and tab separate words. Any other character, a line feed, a
carriage return or `'00'x` included, is part of the word it is in, and is
copied into the result as part of that word.

```rexx
say c2x(subword('a'||'0a'x||'b c', 1, 1)) c2x(subword('a'||'0d'x||' b', 1, 1))
```

```output
610A62 610D
```
