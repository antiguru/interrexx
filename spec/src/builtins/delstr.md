---
element: DELSTR
kind: builtin
syntax: "DELSTR(string [, n [, length]])"
ansi: "9.3.9"
rexxref: "Functions > Built-in Functions > DELSTR (Delete String)"
deviations: [4]
---

# DELSTR

DELSTR deletes a run of characters from a string, selected by position and
length, and answers what is left.

## Arguments

r[bif.delstr.method]
DELSTR is a builtin that runs as a String method, and `string`, its first
argument, is the receiver; `n` and `length` become the message's arguments,
in that order. `string` is a [string](r:conv.string.null), `n` is a
[position](r:conv.position.check), and `length` is a
[length](r:conv.length.check). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): one is required and three
is the most.

```rexx
say delstr('fghij', 3, 2) 'fghij'~delstr(3, 2) 'fghij'~delstr(3) '[' || 'fghij'~delstr || ']'
signal on syntax name a
say delstr('fghij', 0)
a: say condition('o')~code
signal on syntax name b
say delstr('fghij', 2, -1)
b: say condition('o')~code
signal on syntax name c
say delstr()
c: say condition('o')~code
signal on syntax name d
say delstr('fghij', 2, 1, 1)
d: say condition('o')~code
signal on syntax name e
say delstr(, 2)
e: say condition('o')~code
signal on syntax name f
say delstr('fghij', 'x', -1)
f: say condition('o')~code
signal on syntax name g
say delstr('fghij', 0, -1)
g: say condition('o')~code
```

```output
fgj fgj fg []
93.924
93.923
40.3
40.4
40.5
40.12
93.924
```

r[bif.delstr.n.default]
When `n` is omitted, it is 1: the deletion begins at the first character,
and with `length` omitted too the result is the null string.

```rexx
say '['delstr('fghij')']' '['delstr('fghij', )']' delstr('fghij', , 2)
```

```output
[] [] hij
```

> **Note.** The ANSI draft (9.3.9) requires `n`. The reference manual and ooRexx make it
> optional, with the default above.

r[bif.delstr.length.default]
When `length` is omitted, it is as if it were large enough to reach the end
of `string`: the deletion takes the character at position `n` and every
character after it.

```rexx
say delstr('fghi', 3) delstr('fghi', 4) '['delstr('fghi', 1)']'
```

```output
fg fgh []
```

## The result

r[bif.delstr.result]
When `n` is at most `LENGTH(string)`, DELSTR returns the characters of
`string` before position `n`, followed by the characters after the deleted
run; the deleted run is the characters at positions `n` through
`n + length - 1`, or those that exist, when `string` ends sooner.

```rexx
say delstr('fghij', 3, 2) delstr('fghij', 2, 1) delstr('fghij', 1, 2),
    delstr('fghij', 5, 1) delstr('fghij', 3, 3) delstr('fghij', 3, 100),
    delstr('fghij', 3, 999999999999999999)
```

```output
fgj fhij hij fghi fg fg fg
```

r[bif.delstr.past-end]
When `n` is greater than `LENGTH(string)`, nothing is deleted and the result
is `string` unchanged, for every `n` that passes its check and every
`length` that passes its.

```rexx
say delstr('fghij', 6) delstr('fghij', 6, 2) delstr('fghij', 999999999999999999)
say '['delstr('', 1)']' '['delstr('', 5, 2)']'
```

```output
fghij fghij fghij
[] []
```

r[bif.delstr.length.zero]
When `length` is 0, nothing is deleted and the result is `string`
unchanged, for every `n` that passes its check.

```rexx
say delstr('fghij', 3, 0) delstr('fghij', 1, 0) delstr('fghij', 9, 0)
```

```output
fghij fghij fghij
```

r[bif.delstr.bytes]
DELSTR deletes by [character](r:conv.string.chars), one byte each, and
copies every byte it keeps unchanged: whitespace, control characters and
`'00'x` are characters like any other.

```rexx
say c2x(delstr('61000962'x, 2, 1)) c2x(delstr('c3a9'x, 2))
```

```output
610962 C3
```

## Deviations

r[bif.delstr.deviation.4]
interrexx does not specify whether the result of DELSTR is the same object
as another string of equal value: as `string` itself, when the call deletes
nothing, as `delstr(s, length(s) + 1)` does, or as an equal string produced
some other way, such as the literal `'abc'`. Through `identityHash`, or as a
key of an `IdentityTable` or `IdentitySet`, the answer may be either, and a
program can rely only on the value, which is equal under `==`. ooRexx
differs: it often answers a new object where interrexx answers the existing
one.

```rexx
s = 'ab' || 'c'
say (delstr(s, 4) == s) (delstr('abcde', 4) == 'abc')
```

```output
1 1
```
