---
element: WORDPOS
kind: builtin
syntax: "WORDPOS(phrase, string [, start])"
ansi: "9.3.28"
rexxref: "Functions > Built-in Functions > WORDPOS (Word Position)"
---

# WORDPOS

WORDPOS searches a string for a sequence of words and answers the number
of the word where the sequence begins. Words are those of
[the word rule](r:bif.index.words.split), in the phrase and in the string alike.

## Arguments

r[bif.wordpos.method]
WORDPOS is a builtin that runs as a String method, and `string`, its
second argument, is the receiver; `phrase` and `start` become the
message's arguments, in that order. `phrase` and `string` are
[strings](r:conv.string.null), and `start` is a
[word number](r:bif.index.words.number). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): two are required and three is
the most.

```rexx
say wordpos('b c', 'a b c') 'a b c'~wordpos('b c')
signal on syntax name a
say wordpos('a', 'a', 0)
a: say condition('o')~code
signal on syntax name b
say wordpos('a', 'a', 'one')
b: say condition('o')~code
signal on syntax name c
say wordpos(, 'a')
c: say condition('o')~code
signal on syntax name d
say wordpos('a', 'a', 1, 1)
d: say condition('o')~code
```

```output
2 2
93.924
40.12
40.5
40.4
```

r[bif.wordpos.start.default]
When `start` is omitted, it is 1: the search begins at the first word of
`string`.

```rexx
say wordpos('a', 'a b a') wordpos('a', 'a b a', ) wordpos('a', 'a b a', 2)
```

```output
1 1 3
```

## The result

r[bif.wordpos.result]
When `phrase` has at least one word, let `m` be the number of its words.
WORDPOS then returns the smallest word number `k`, with `k` at least `start`,
such that `string` has a word `k + m - 1` and, for each `i` from 1 to `m`,
word `k + i - 1` of `string` is equal under `==` to word `i` of `phrase`. When
there is no such `k`, the result is `0`. A phrase with no words is the case of
[its own rule](r:bif.wordpos.empty-phrase). The answer is [written as a
count](r:bif.index.count.answer), and whether it is a new object is [not
specified](r:bif.index.count.answer.deviation.4).

```rexx
s = 'one two three two three four'
say wordpos('two', s) wordpos('two three', s) wordpos('two three', s, 3),
    wordpos('two three', s, 5) wordpos(s, s)
```

```output
2 2 4 0 1
```

r[bif.wordpos.whitespace]
Only the words of `phrase` and of `string` take part in the search. The
whitespace before the first word, between two words and after the last, in
either argument, plays no part: neither how long a run of it is nor
whether it is blanks or tabs changes the answer.

```rexx
s = 'one two three two three four'
say wordpos('two' || '09'x || '  three', s) wordpos(' two three ', s),
    wordpos('two three', 'one two' || '0909'x || 'three')
```

```output
2 2 2
```

r[bif.wordpos.exact]
Two words are equal only when they have the same characters in the same
order. Case matters, no numeric comparison is made, and a word of `phrase`
never matches only a part of a word of `string`. A character that
[the word rule](r:bif.index.words.split) places inside a word, such as a line
feed, must match like any other.

```rexx
s = 'one two three two three four'
say wordpos('Two', s) wordpos('tw', s) wordpos('wo', s) wordpos('two thr', s)
say wordpos('1', '01 1.0 1') wordpos('a b', 'a' || '0a'x || 'b'),
    wordpos('a' || '0a'x || 'b', 'a b a' || '0a'x || 'b')
```

```output
0 0 0 0
3 0 3
```

r[bif.wordpos.overlap]
Every word number from `start` on is a candidate for `k`, including a word
inside an earlier candidate that matched in part. A partial match does not
make the search skip past the words it covered, so a sequence found can
overlap one that was tried before it.

```rexx
say wordpos('a a', 'a a a', 2) wordpos('a b a', 'a b a b a', 2),
    wordpos('a a b', 'a a a b')
```

```output
2 3 2
```

r[bif.wordpos.partial]
A sequence must lie wholly within `string`. When the words of `phrase`
match the last words of `string` but `phrase` has more words after them,
the phrase is not found there, and the result is `0` unless it is found
earlier.

```rexx
s = 'one two three two three four'
say wordpos('four five', s) wordpos('three four five', s) wordpos('x y', 'x')
```

```output
0 0 0
```

r[bif.wordpos.past-end]
When `start` is greater than the number of words in `string`, the result is
`0`, for every `start` that passes its check. A `start` equal to the number of
words searches the last word alone.

```rexx
s = 'one two three two three four'
say wordpos('four', s, 6) wordpos('four', s, 7) wordpos('one', s, 999999999999999999)
```

```output
6 0 0
```

r[bif.wordpos.empty-phrase]
When `phrase` has no words, the null string or a string of only whitespace
among them, the result is `0` for every `string`, the null string
included. `start` is still checked, as a word number always is.

```rexx
say wordpos('', 'a b') wordpos(' ' || '09'x, 'a b') wordpos('', '')
signal on syntax name e
say wordpos('', '', 0)
e: say condition('o')~code
```

```output
0 0 0
93.924
```

r[bif.wordpos.empty-string]
When `string` has no words, the result is `0` for every `phrase` and every
`start` that passes its check.

```rexx
say wordpos('a', '') wordpos('a', '  ' || '09'x, 1)
```

```output
0 0
```

> **Note.** The ANSI draft (9.3.28) defines WORDPOS by comparing the phrase,
> with its whitespace reduced to single blanks, against each run of the same
> number of words of the string, reduced the same way, from `start` on. That
> gives the answers above.
