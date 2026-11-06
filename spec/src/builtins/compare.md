---
element: COMPARE
kind: builtin
syntax: "COMPARE(string1, string2 [, pad])"
ansi: "9.3.5"
rexxref: "Functions > Built-in Functions > COMPARE"
---

# COMPARE

COMPARE tests two strings for equality and, when they differ, answers the
position of the first character where they do. The shorter string is
treated as if a pad character followed it.

## Arguments

r[bif.compare.method]
COMPARE is a builtin that runs as a String method, and `string1`, its first
argument, is the receiver; `string2` and `pad` become the message's arguments,
in that order. `string1` and `string2` are [strings](r:conv.string.null), and
`pad` is a [pad](r:conv.pad). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): two are required and three
is the most.

```rexx
say compare('abc', 'abd') 'abc'~compare('abd') 'abc'~compare('ab', 'c')
signal on syntax name a
say compare('abc', 'ab', 'cc')
a: say condition('o')~code
signal on syntax name b
say compare('abc', 'ab', '')
b: say condition('o')~code
signal on syntax name c
say compare('abc')
c: say condition('o')~code
signal on syntax name d
say compare('abc', 'ab', 'c', 1)
d: say condition('o')~code
signal on syntax name e
say compare(, 'ab')
e: say condition('o')~code
signal on syntax name f
say compare('abc', , '')
f: say condition('o')~code
```

```output
3 3 0
40.23
40.23
40.3
40.4
40.5
40.5
```

r[bif.compare.pad.default]
When `pad` is omitted, it is the blank, `'20'x`.

```rexx
say compare('xy ', 'xy') compare('xy ', 'xy', ) compare('xy  q', 'xy')
```

```output
0 0 5
```

r[bif.compare.pad.checked]
`pad` is checked as a [pad](r:conv.pad) always is, whatever the two
strings are: it raises `SYNTAX 40.23` when it is not one character even if
the strings are equal or null and so no padding takes place.

```rexx
signal on syntax name a
say compare('abc', 'abc', 'xy')
a: say condition('o')~code
signal on syntax name b
say compare('', '', '')
b: say condition('o')~code
```

```output
40.23
40.23
```

## The result

r[bif.compare.result]
Let `p` be `pad`, and let the two strings be made equally long by
appending as many copies of `p` to the shorter one as are missing. COMPARE
then returns `0` when the two lengthened strings are the same string, and
otherwise the position of the first character at which they differ: the
smallest `i` for which their characters at position `i` are not the same.
The answer is [written as a count](r:bif.index.count.answer), and whether it
is a new object is [not specified](r:bif.index.count.answer.deviation.4).

```rexx
say compare('tuv', 'tuv') compare('tuv', 'tx') compare('abc', 'xbc'),
    compare('abc', 'abd') compare('abc', 'ab') compare('ab', 'abc')
```

```output
0 2 1 3 3 3
```

r[bif.compare.padding]
The padding makes a string equal to itself followed by any number of
copies of `pad`, and only that. A string that is the other one followed by
characters that are not all `pad` differs from it at the first such
character; a longer string whose extra characters are all `pad` does not
differ from the shorter one.

```rexx
say compare('xy ', 'xy') compare('xy ', 'xy', ' ') compare('xy ', 'xy', 'z'),
    compare('xy== ', 'xy', '=') compare('xyzz', 'xy', 'z') compare('xyzzq', 'xy', 'z')
```

```output
0 0 3 5 0 5
```

r[bif.compare.empty]
The null string is a string of no characters, so it takes the padding
alone: it is equal to the other string when that one is empty or all `pad`,
and otherwise differs from it at the position of the first character of the
other string that is not `pad`.

```rexx
say compare('', '') compare('', 'a') compare('a', '') compare('', '  ') compare('', '  a')
say compare('', '  ', 'x') compare('', 'xx', 'x') compare('xx', '', 'x'),
    compare('xxy', '', 'x')
```

```output
0 1 1 0 3
1 0 0 3
```

r[bif.compare.exact]
Characters are compared for equality exactly, one by one. Case matters, no
numeric comparison is made, and whitespace and every other character,
`'00'x` among them, count like any other.

```rexx
say compare('abc', 'ABC') compare('1', '1.0') compare(1, 1) compare('a b', 'a  b'),
    compare('a' || '00'x, 'a') compare('a' || '00'x, 'a', '00'x)
```

```output
1 2 0 3 2 0
```

> **Note.** The ANSI draft (9.3.5) pads the shorter string with `LEFT` and
> compares character by character. That gives the answers above.
