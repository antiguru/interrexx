---
element: WORDS
kind: builtin
syntax: "WORDS(string)"
ansi: "9.3.29"
rexxref: "Functions > Built-in Functions > WORDS"
---

# WORDS

WORDS answers how many words a string has. Words are those of
[the word rule](r:bif.index.words.split).

## Arguments

r[bif.words.method]
WORDS is a builtin that runs as a [String method](r:class.string.builtins),
and `string`, its only argument, is the receiver; the message takes no other
argument. `string` is a [string](r:conv.string.null), so no value of it raises
an error. The argument count is checked by the [builtin call
rules](r:bif.index.call.count): one is required and one is the most.

```rexx
say words('abc def') 'abc def'~words
signal on syntax name a
say words()
a: say condition('o')~code
signal on syntax name b
say words('a', 'b')
b: say condition('o')~code
signal on syntax name c
say words(,)
c: say condition('o')~code
signal on syntax name d
say words(, 'b')
d: say condition('o')~code
```

```output
2 2
40.3
40.4
40.3
40.4
```

## The result

r[bif.words.result]
WORDS returns how many words `string` has, which is the largest word number
that [the word rule](r:bif.index.words.split) gives it. The number is
[written as a count](r:bif.index.count.answer), and whether it is a new
object is [not specified](r:bif.index.count.answer.deviation.4).

```rexx
numeric digits 3
say words('four short words here') words(copies('w ', 1500)) words(' a ' || '09'x)
```

```output
4 1500 1
```

r[bif.words.none]
A string with no words, the null string or one of only whitespace, has 0
words.

```rexx
say words('') words('  ') words('09'x || ' ' || '09'x)
```

```output
0 0 0
```

r[bif.words.separators]
Only whitespace, the blank and the tab, separates two words, however many
of them are in a run. A line feed, a carriage return, a form feed, a
vertical tab and `'00'x` are characters of a word, so they join the words
beside them instead of splitting them, and a string of only such characters
has one word.

```rexx
say words('a  b' || '09'x || '09'x || 'c') words('a' || '0a'x || 'b'),
    words('a' || '0d0c0b00'x || 'b') words('0a'x)
```

```output
3 1 1 1
```

r[bif.words.object]
An argument that is not a string is first replaced by its string value, as
[the string rule](r:conv.string.object) says, and WORDS counts the words of
that.

```rexx
say words(.object~new) words(.object~new~string)
```

```output
2 2
```

> **Note.** The ANSI draft (9.3.29) defines WORDS as the number of leading
> word numbers `k`, counting from 1, for which `SUBWORD(string, k)` is not the
> null string; it is 0 when there is no word. This counts the same words.

> **Note.** An argument whose `STRING` method answers an object that is not a
> string falls under [Deviation 30](r:conv.string.deviation.30), as it does
> for every builtin.
