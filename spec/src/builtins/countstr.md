---
element: COUNTSTR
kind: builtin
syntax: "COUNTSTR(needle, haystack)"
ansi: "9.3.7"
rexxref: "Functions > Built-in Functions > COUNTSTR"
---

# COUNTSTR

COUNTSTR counts how many times one string occurs in another, counting
occurrences that do not overlap.

## Arguments

r[bif.countstr.method]
COUNTSTR is a builtin that runs as a String method, and `haystack`, its
second argument, is the receiver; `needle` becomes the message's argument.
Both are [strings](r:conv.string.null), and the function form takes `needle`
first. The argument count is checked by the
[builtin call rules](r:bif.index.call.count): both arguments are required and
two is the most.

```rexx
say countstr('an', 'banana') 'banana'~countstr('an')
signal on syntax name a
say countstr('a')
a: say condition('o')~code
signal on syntax name b
say countstr('a', 'a', 1)
b: say condition('o')~code
signal on syntax name c
say countstr(, 'a')
c: say condition('o')~code
signal on syntax name d
say countstr('a', )
d: say condition('o')~code
```

```output
2 2
40.3
40.4
40.5
40.3
```

## The result

r[bif.countstr.result]
COUNTSTR scans `haystack` from its first character to its last. At each
position where `needle` occurs, it counts one and continues with the
character after the occurrence, so no character takes part in two counted
occurrences; where it does not occur, it continues with the next character.
The result is the number counted. It is [written as a count](r:bif.index.count.answer),
and whether it is a new object is
[not specified](r:bif.index.count.answer.deviation.4).

```rexx
say countstr('1', '101101') countstr('KK', 'J0KKK0') countstr('ab', 'xabyabab'),
    countstr('a', 'bcd')
```

```output
4 1 3 0
```

r[bif.countstr.overlap]
Occurrences that overlap are not both counted: an occurrence is counted
only when it begins after the end of the one counted before it. A run
such as `'aaa'` holds one `'aa'`, not two.

```rexx
say countstr('aa', 'aaa') countstr('aa', 'aaaa') countstr('aba', 'ababababa'),
    countstr('aba', 'abababa')
```

```output
1 2 2 2
```

r[bif.countstr.empty]
When `needle` is the null string, the result is `0`, for every `haystack`,
the null string included: the null string occurs nowhere that counts. When
`haystack` is the null string, the result is `0` for every `needle`.

```rexx
say countstr('', 'abc') countstr('', '') countstr('a', '') countstr('abc', 'ab')
```

```output
0 0 0 0
```

r[bif.countstr.exact]
Characters are compared for equality exactly, one by one. Case matters, no
numeric comparison is made, and whitespace and every other character,
`'00'x` among them, count like any other.

```rexx
say countstr('a', 'aA') countstr('1', '1.01 1') countstr(' ', 'a b  c'),
    countstr('a' || '00'x, 'a' || '00'x || 'a' || '00'x || 'a') countstr(1, 11211)
```

```output
1 3 3 2 4
```

> **Note.** The ANSI draft (9.3.7) defines COUNTSTR by repeated POS searches, each
> starting after the end of the previous match, and does not mention the
> case of a null needle, for which POS answers 0. The reference manual defines
> it only as the count of occurrences that do not overlap.
