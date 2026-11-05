---
element: LENGTH
kind: builtin
syntax: "LENGTH(string)"
ansi: "9.3.14"
rexxref: "Functions > Built-in Functions > LENGTH"
deviations: [4]
---

# LENGTH

LENGTH answers the number of characters in a string.

## Arguments

r[bif.length.method]
LENGTH is a builtin that runs as a String method, and `string`, its only
argument, is the receiver; the message takes no other argument. `string` is
a [string](r:conv.string.null), so no value of it raises an error. The
argument count is checked by the [builtin call rules](r:bif.index.call.count):
one is required and one is the most.

```rexx
say length('abc def') 'abc def'~length
signal on syntax name a
say length()
a: say condition('o')~code
signal on syntax name b
say length('a', 'b')
b: say condition('o')~code
signal on syntax name c
say length(,)
c: say condition('o')~code
signal on syntax name d
say length(, 'b')
d: say condition('o')~code
```

```output
7 7
40.3
40.4
40.3
40.4
```

## The result

r[bif.length.result]
LENGTH returns the number of characters in `string`, which is its
[length](r:conv.string.chars) in bytes. The null string has length 0. The
answer is [written as a count](r:bif.index.count.answer), and whether it is a new
object is [not specified](r:bif.length.deviation.4).

```rexx
say length('abcdefgh') length('abc defg') length('')
numeric digits 3
say length(copies('x', 1234))
```

```output
8 8 0
1234
```

r[bif.length.bytes]
Every character counts once, whatever it is: a blank, a tab, a line feed or
`'00'x` at any place in the string, leading and trailing blanks included, and
each byte of what an encoding writes as several.

```rexx
say length(' ') length(' a  ') length('00'x) length('0a'x || 'b' || '09'x),
    length('c3a9'x)
```

```output
1 4 1 3 2
```

r[bif.length.string-form]
When `string` is a number, LENGTH counts the characters of the string the
number is, as it is written: a literal keeps its spelling, and the result of
arithmetic is the string that [NUMERIC DIGITS](r:conv.whole.digits) and the
arithmetic rules make of it. When `string` is an object other than a string,
it is [replaced by its string](r:conv.string.object).

```rexx
say length(12.00) length(-5) length(1 + 1) length(.object~new)
numeric digits 5
say length(1/3) length(1e20 + 0)
numeric digits 9
say length(1/3)
```

```output
5 2 1 9
7 5
11
```

## Deviations

r[bif.length.deviation.4]
interrexx does not specify whether the result of LENGTH is the same object as
another string of equal value: as an equal number the program already holds,
or a literal such as `'1234'`, or the equal answer of another call. Through
`identityHash`, or as a key of an `IdentityTable` or `IdentitySet`, the answer
may be either, and a program can rely only on the value, which is equal under
`==`. ooRexx differs: it often answers a new object where interrexx answers
the existing one.

```rexx
h = copies('x', 1234)
n = 1233 + 1
say (length(h) == n) (length(h) == '1234') (length(h) == length(h))
```

```output
1 1 1
```

> **Note.** The ANSI draft (9.3.14) defines LENGTH as the number of characters in the
> argument and adds an error for an argument that is not a character string,
> which cannot occur here: every value is a string.
