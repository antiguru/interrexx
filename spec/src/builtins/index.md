---
element: "Built-in functions"
kind: chapter
ansi: "9.2.1"
rexxref: "Functions > Built-in Functions"
---

# Built-in functions

This chapter has a page for each builtin function, and each page specifies
the function call. The rules here hold for every builtin call; the pages
link to them, and to the [conventions](../conventions.md), instead of
restating them.

A page's syntax line names the arguments in order. A name in square
brackets is an optional argument; every other argument is required. The
number of names on the line is the most arguments the builtin takes, and
the number of required ones is the fewest.

r[bif.call.count]
The argument count of a call is the position of the last argument in it
that is not [omitted](r:conv.arg.omitted). Omitted places at the end of the
list therefore add nothing to the count, and a call with only omitted
places counts as a call with no arguments, a count of 0.

```rexx
say substr('abc', 2, , ) word('a b', 2, , , )
```

```output
bc b
```

r[bif.call.too-few]
A call whose argument count is less than the number of required arguments
raises `SYNTAX 40.3`.

```rexx
signal on syntax name a
say substr('abc')
a: say condition('o')~code
signal on syntax name b
say substr('abc', )
b: say condition('o')~code
signal on syntax name c
say word(, , )
c: say condition('o')~code
```

```output
40.3
40.3
40.3
```

r[bif.call.too-many]
A call whose argument count is greater than the number of arguments on the
syntax line raises `SYNTAX 40.4`, whatever the values of the arguments,
and even when the arguments before the last are omitted.

```rexx
signal on syntax name a
say word('a b', 1, 2)
a: say condition('o')~code
signal on syntax name b
say substr(, , , , 1)
b: say condition('o')~code
```

```output
40.4
40.4
```

r[bif.call.missing]
A required argument that is omitted, in a call whose argument count reaches
past it, raises `SYNTAX 40.5`.

```rexx
signal on syntax name a
say substr(, 1)
a: say condition('o')~code
signal on syntax name b
say wordpos('a', , 1)
b: say condition('o')~code
```

```output
40.5
40.5
```

r[bif.call.order.count-before-missing]
A call whose argument count is wrong raises the [too-few](r:bif.call.too-few)
or [too-many](r:bif.call.too-many) error even when it also omits a
[required argument](r:bif.call.missing).

```rexx
signal on syntax name a
say word(, 1, 2)
a: say condition('o')~code
```

```output
40.4
```

r[bif.call.order.count-before-values]
A call whose argument count is wrong raises the too-few or too-many error
even when an argument it does pass would fail its own check.

```rexx
signal on syntax name a
say substr('abc', 'x', 1, '**', 5)
a: say condition('o')~code
signal on syntax name b
say word('a b', 0, 2)
b: say condition('o')~code
```

```output
40.4
40.4
```

## Words

r[bif.words.split]
The builtins that count words split a string into words the same way. A
word is a run of one or more characters, none of them
[whitespace](r:conv.string.whitespace), that has whitespace or an end of
the string on each side. The words of a string are numbered from 1, left to
right. Whitespace before the first word, between two words and after the
last belongs to no word, so the null string and a string of only whitespace
have no words. Every other character, line feed and `'00'x` among them, can
be part of a word.

```rexx
say words(' a'||'09'x||'b  c ') words('') words('  '||'09'x)
say word('one  two', 2) c2x(word('a'||'0a000c'x||'b c', 1))
```

```output
3 0 0
two 610A000C62
```

The ANSI draft (9.3.26 and the other word builtins) separates words by a
set of blank characters that it leaves to the configuration: the blank and
any others the configuration names. ooRexx fixes that set as the blank and
the horizontal tab, and so does this book.

r[bif.words.number]
Where a word builtin's page calls an argument a word number, that argument
is a [position](r:conv.position.check) that counts words: the word number
`n` names word `n` of the string as [the word rule](r:bif.words.split)
numbers them. A word number greater than the number of words in the string
is not an error, however large it is; it names no word, and the builtin's
page says what the builtin answers then.

```rexx
say '['word('a b', 3)']' wordindex('a b', 3) wordlength('a b', 999999999999999999),
    wordpos('a', 'a b', 3)
signal on syntax name e
say wordindex('a b', 0)
e: say condition('o')~code
```

```output
[] 0 0 0
93.924
```

r[bif.words.answer]
WORDINDEX, WORDLENGTH and WORDPOS each answer a count, of characters or of
words, and they write it as a whole number in decimal digits only: no sign,
no leading zero, no period and no exponent, and `0` for zero. The caller's
NUMERIC DIGITS setting plays no part, so the answer is exact however many
digits it has.

```rexx
numeric digits 3
say wordindex(copies(' ', 1000) || 'a', 1) wordlength(copies('z', 1234), 1),
    wordpos('y', copies('x ', 1200) || 'y') wordpos('q', 'a b')
```

```output
1001 1234 1201 0
```

The reference manual does not say how these answers are written or whether
NUMERIC DIGITS applies to them; this follows the observed behaviour.
