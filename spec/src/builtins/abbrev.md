---
element: ABBREV
kind: builtin
syntax: "ABBREV(information, info [, length])"
ansi: "9.3.1"
rexxref: "Functions > Built-in Functions > ABBREV (Abbreviation)"
---

# ABBREV

ABBREV tests whether one string is an abbreviation of another: whether it
is the beginning of that string and, when the call asks for it, long enough.
The answer is `1` or `0`.

## Arguments

r[bif.abbrev.method]
ABBREV is a builtin that runs as a String method, and `information`, its
first argument, is the receiver; `info` and `length` become the message's
arguments, in that order. `information` and `info` are
[strings](r:conv.string.null), and `length` is a
[length](r:conv.length.check). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): two are required and three
is the most.

```rexx
say abbrev('Print', 'Pri') 'Print'~abbrev('Pri') 'Print'~abbrev('Pri', 4)
signal on syntax name a
say abbrev('abc', 'ab', -1)
a: say condition('o')~code
signal on syntax name b
say abbrev('abc', 'ab', 'x')
b: say condition('o')~code
signal on syntax name c
say abbrev('abc', 'ab', )
say abbrev('abc')
c: say condition('o')~code
signal on syntax name d
say abbrev('abc', 'ab', 2, 1)
d: say condition('o')~code
signal on syntax name e
say abbrev(, 'ab')
e: say condition('o')~code
signal on syntax name f
say abbrev('abc', 'a', 'x', 1)
f: say condition('o')~code
signal on syntax name g
say abbrev('abc', , -1)
g: say condition('o')~code
```

```output
1 1 0
93.923
40.12
1
40.3
40.4
40.5
40.4
40.5
```

r[bif.abbrev.length.default]
When `length` is omitted, it is `LENGTH(info)`: the whole of `info` takes
part, and no `info` is too short.

```rexx
say abbrev('PRINT', 'PRI') abbrev('PRINT', 'PRI', ) abbrev('PRINT', 'PRI', 3),
    abbrev('PRINT', 'PRI', 4)
```

```output
1 1 1 0
```

## The result

r[bif.abbrev.result]
ABBREV returns `1` when all three of these hold, and `0` when any does not:
`info` is no longer than `information`; the characters of `info` are equal
to the first `LENGTH(info)` characters of `information`, in order; and
`LENGTH(info)` is at least `length`. The answer is the single digit `1` or
`0`, for every call.

```rexx
say abbrev('PRINT', 'PRINT') abbrev('PRINT', 'PRINT', 5) abbrev('PRINT', 'PRI'),
    abbrev('PRINT', 'RIN') abbrev('PRINT', 'PRINTS') abbrev('PRINT', 'PRY')
```

```output
1 1 1 0 0 0
```

r[bif.abbrev.length.minimum]
`length` is the least number of characters `info` may have. A `length`
greater than `LENGTH(info)` therefore gives `0`, for every `length` that
passes its check, a very large one included, and even when `info` is the
beginning of `information`. A `length` of 0, or any `length` up to
`LENGTH(info)`, leaves the answer to the comparison of the characters.

```rexx
say abbrev('abc', 'ab', 0) abbrev('abc', 'ab', 2) abbrev('abc', 'ab', 3),
    abbrev('abc', 'ab', 999999999999999999) abbrev('abc', 'xy', 0)
```

```output
1 1 0 0 0
```

r[bif.abbrev.empty]
When `info` is the null string, it is the beginning of every
`information`, the null string included, so the answer is `1` when `length`
is omitted or 0, and `0` when `length` is 1 or more. When `information` is
the null string, only a null `info` can be a beginning of it.

```rexx
say abbrev('PRINT', '') abbrev('PRINT', '', 0) abbrev('PRINT', '', 1),
    abbrev('', '') abbrev('', '', 0) abbrev('', '', 1) abbrev('', 'a')
```

```output
1 1 0 1 1 0 0
```

r[bif.abbrev.exact]
The characters are compared for equality exactly, one by one. Case matters,
no numeric comparison is made, and whitespace and every other character,
`'00'x` among them, count like any other.

```rexx
say abbrev('Print', 'pri') abbrev('1.50', '1.5') abbrev('1.50', '1.500'),
    abbrev('01', '1') abbrev('a b', 'a ') abbrev('a b', 'a') abbrev('a b', 'a  ')
say abbrev('a' || '00'x || 'b', 'a' || '00'x) abbrev('a' || '00'x, 'a' || '01'x)
```

```output
0 1 0 0 1 1 0
1 0
```
