---
element: DELWORD
kind: builtin
syntax: "DELWORD(string, n [, length])"
ansi: "9.3.10"
rexxref: "Functions > Built-in Functions > DELWORD (Delete Word)"
deviations: [4]
---

# DELWORD

DELWORD deletes a run of words from a string, selected by word number and
word count, and answers what is left. Words are those of
[the word rule](r:bif.index.words.split).

## Arguments

r[bif.delword.method]
DELWORD is a builtin that runs as a String method, and `string`, its first
argument, is the receiver; `n` and `length` become the message's arguments,
in that order. `string` is a [string](r:conv.string.null), `n` is a
[word number](r:bif.index.words.number), and `length` is a
[length](r:conv.length.check) that counts words. The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): two are required and three
is the most.

```rexx
say delword('Now is the time', 2, 1) 'Now is the time'~delword(2, 1) 'Now is the time'~delword(2)
signal on syntax name a
say delword('a b', 0)
a: say condition('o')~code
signal on syntax name b
say delword('a b', 1, -1)
b: say condition('o')~code
signal on syntax name c
say delword('a b')
c: say condition('o')~code
signal on syntax name d
say delword('a b', 1, 1, 1)
d: say condition('o')~code
signal on syntax name e
say delword(, 1)
e: say condition('o')~code
signal on syntax name f
say delword('a b', 'x', -1)
f: say condition('o')~code
signal on syntax name g
say delword('a b', 0, -1)
g: say condition('o')~code
signal on syntax name h
say delword('a b', )
h: say condition('o')~code
```

```output
Now the time Now the time Now 
93.924
93.923
40.3
40.4
40.5
40.12
93.924
40.3
```

r[bif.delword.length.default]
When `length` is omitted, it is as if it were large enough to reach the
last word of `string`: the deletion takes word `n` and every word after it.

```rexx
say '['delword('Now is the time', 3)']' '['delword('Now is the time', 1)']'
```

```output
[Now is ] []
```

## The result

r[bif.delword.result]
When `n` is at most the number of words of `string`, the deleted run begins
at the first character of word `n` and extends over `length` words,
or over the words that exist when `string` has fewer after word `n - 1`,
and over the whitespace that follows the last of them; whitespace before
word `n` is kept. DELWORD returns the characters before the deleted run
followed by the characters after it.

```rexx
say '['delword('Now is the  time', 2, 2)']' '['delword('Now is   the time', 3, 1)']',
    '['delword('Now is the time', 2, 1)']'
say '['delword('  Now  is  the  time  ', 2, 1)']' '['delword('  Now  is  the  time  ', 1, 1)']'
```

```output
[Now time] [Now is   time] [Now the time]
[  Now  the  time  ] [  is  the  time  ]
```

r[bif.delword.trailing]
Whitespace that follows the last word of `string` belongs to the deleted run
when that word is the last one deleted, and whitespace before the first
deleted word never does. So deleting through the last word leaves the
whitespace before word `n` and nothing after it, and deleting the first word
leaves the whitespace before it.

```rexx
say '['delword('Now is the time ', 3)']' '['delword('Now is the time  ', 4, 1)']',
    '['delword('  a  b  ', 1)']' '['delword('  a  b  ', 2)']'
```

```output
[Now is ] [Now is the ] [  ] [  a  ]
```

r[bif.delword.whitespace]
Whitespace is [blank and tab](r:conv.string.whitespace), and the rules above
treat the two alike: a tab between words, or after the last deleted word, is
deleted or kept exactly as a blank would be, and the characters that stay are
copied unchanged. A character that is not whitespace, such as a line feed, is
part of a word.

```rexx
t = '09'x
say c2x(delword('a' || t || 'b' || t || 'c', 2, 1)) c2x(delword('a' || t || 'b' || t || 'c', 1, 1))
say c2x(delword('a' || '0a'x || 'b c', 1, 1)) c2x(delword('a b' || t || '  ' || t, 2, 1))
```

```output
610963 620963
63 6120
```

r[bif.delword.past-end]
When `n` is greater than the number of words of `string`, nothing is deleted
and the result is `string` unchanged, whitespace included, for every `n` that
passes its check and every `length` that passes its. So it is when `string`
has no words.

```rexx
say '['delword('Now is the time', 5)']' '['delword('  a  b  ', 3, 2)']',
    '['delword('', 1)']' '['delword('   ', 2)']' '['delword('   ', 1)']',
    '['delword('a b', 999999999999999999)']'
```

```output
[Now is the time] [  a  b  ] [] [   ] [   ] [a b]
```

r[bif.delword.length.zero]
When `length` is 0, nothing is deleted and the result is `string`
unchanged, for every `n` that passes its check.

```rexx
say '['delword('  a  b  ', 1, 0)']' '['delword('  a  b  ', 2, 0)']' '['delword('a b', 9, 0)']'
```

```output
[  a  b  ] [  a  b  ] [a b]
```

r[bif.delword.length.large]
A `length` greater than the number of words from word `n` to the last is
the same as the number of them, however large it is.

```rexx
say '['delword('a b c d', 2, 2)']' '['delword('a b c d', 2, 3)']' '['delword('a b c d', 2, 999999999999999999)']'
```

```output
[a d] [a ] [a ]
```

## Deviations

r[bif.delword.deviation.4]
interrexx does not specify whether the result of DELWORD is the same object
as another string of equal value: as `string` itself, when the call deletes
nothing, as `delword(s, 99)` does, or as an equal string produced some other
way, such as the literal `'a b'`. Through `identityHash`, or as a key of an
`IdentityTable` or `IdentitySet`, the answer may be either, and a program can
rely only on the value, which is equal under `==`. ooRexx differs: it often
answers a new object where interrexx answers the existing one.

```rexx
s = 'a ' || 'b'
say (delword(s, 3) == s) (delword('a b c', 3) == 'a b ')
```

```output
1 1
```
