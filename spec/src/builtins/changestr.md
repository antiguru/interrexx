---
element: CHANGESTR
kind: builtin
syntax: "CHANGESTR(needle, haystack, newneedle [, count])"
ansi: "9.3.4"
rexxref: "Functions > Built-in Functions > CHANGESTR"
deviations: [4]
---

# CHANGESTR

CHANGESTR copies a string, replacing the occurrences of one string in it
with another. Occurrences that overlap are not both replaced, and a
`count` argument can limit how many are.

## Arguments

r[bif.changestr.method]
CHANGESTR is a builtin that runs as a String method, and `haystack`, its
second argument, is the receiver; `needle`, `newneedle` and `count` become
the message's arguments, in that order. `needle`, `haystack` and
`newneedle` are [strings](r:conv.string.null), and the function form takes
`needle` first. `count` is a whole number, described by
[its own rule](r:bif.changestr.count). The argument count is checked by the
[builtin call rules](r:bif.index.call.count): three arguments are required
and four is the most.

```rexx
say changestr('an', 'banana', '__') 'banana'~changestr('an', '__') 'banana'~changestr('an', '__', 1)
signal on syntax name a
say changestr('a', 'abc')
a: say condition('o')~code
signal on syntax name b
say changestr('a', 'abc', 'x', 1, 1)
b: say condition('o')~code
signal on syntax name c
say changestr('a', , 'x')
c: say condition('o')~code
signal on syntax name d
say changestr('a', 'abc', , 1)
d: say condition('o')~code
```

```output
b____a b____a b__ana
40.3
40.4
40.5
40.5
```

r[bif.changestr.count]
`count` is a [whole-number](r:conv.whole.form) argument, so it raises
`SYNTAX 40.12` when it does not have that form or has a
[value](r:conv.whole.digits) that is not a whole number. A whole number
below 0 raises `SYNTAX 93.906`; `-0` is 0. The null string is not a
whole number, so an empty `count` raises 40.12 and does not stand for an
omitted one. The form check comes first, so a `count` that fails it raises
40.12 whatever its sign.

```rexx
say changestr('a', 'banana', 'x', ' + 2 ') changestr('a', 'banana', 'x', '-0'),
    changestr('a', 'banana', 'x', '2.000')
signal on syntax name a
say changestr('a', 'banana', 'x', -1)
a: say condition('o')~code
signal on syntax name b
say changestr('a', 'banana', 'x', '')
b: say condition('o')~code
signal on syntax name c
say changestr('a', 'banana', 'x', '-0.4')
c: say condition('o')~code
signal on syntax name d
say changestr('a', 'banana', 'x', '-1e5')
d: say condition('o')~code
```

```output
bxnxna banana bxnxna
93.906
40.12
40.12
93.906
```

r[bif.changestr.count.default]
When `count` is omitted, every occurrence that
[the result rule](r:bif.changestr.result) selects is replaced. A `count` at
least the number of such occurrences has the same effect, however large it
is, up to the largest value the argument accepts.

```rexx
say changestr('a', 'banana', 'x') changestr('a', 'banana', 'x', ),
    changestr('a', 'banana', 'x', 3) changestr('a', 'banana', 'x', 999999999999999999)
```

```output
bxnxnx bxnxnx bxnxnx bxnxnx
```

## The result

r[bif.changestr.result]
CHANGESTR scans `haystack` from its first character to its last. At each
position where `needle` occurs, it appends `newneedle` to the result and
continues with the character after the occurrence; where `needle` does not
occur, it appends the character at that position and continues with the next
one. Every occurrence found this way is replaced, up to `count` of them
when `count` is given: after the `count`th replacement the rest of
`haystack` is appended unchanged. The match is exact, so case matters, and
every character of the strings, whitespace and `'00'x` included, is an
ordinary character.

```rexx
say changestr('o', 'foolish', '') changestr('o', 'foolish', '0') changestr('o', 'foolish', '0', 1)
say changestr('A', 'banana', 'x') changestr(' ', 'a b  c', '_')
say c2x(changestr('00'x, 'a' || '00'x || 'b', '0000'x))
```

```output
flish f00lish f0olish
banana a_b__c
61000062
```

r[bif.changestr.overlap]
Occurrences that overlap are not all replaced: an occurrence is replaced
only when it begins after the end of the one replaced before it, as
[COUNTSTR](r:bif.countstr.overlap) counts them. The text that
`newneedle` brings in is never searched again, so a `newneedle` that holds
`needle` does not make the call replace more.

```rexx
say changestr('aa', 'aaaaa', 'b') changestr('aba', 'ababababa', 'X'),
    changestr('a', 'aaaa', 'aa') changestr('a', 'aaaa', 'aa', 2)
```

```output
bba XbXba aaaaaaaa aaaaaa
```

r[bif.changestr.count.zero]
When `count` is 0, nothing is replaced and the result is `haystack`
unchanged. `count` is still checked, as it always is.

```rexx
say changestr('a', 'banana', 'x', 0) changestr('a', 'banana', 'x', '0.0')
signal on syntax name a
say changestr('a', 'banana', 'x', 'zero')
a: say condition('o')~code
```

```output
banana banana
40.12
```

r[bif.changestr.empty-needle]
When `needle` is the null string, it is taken to occur nowhere, so nothing
is replaced and the result is `haystack` unchanged, for every `newneedle`
and every `count` that passes its check.

```rexx
say '['changestr('', 'abc', 'x')']' '['changestr('', 'abc', 'x', 2)']'
say '['changestr('', '', 'x')']'
```

```output
[abc] [abc]
[]
```

r[bif.changestr.empty-haystack]
When `haystack` is the null string, `needle` occurs in it nowhere, and the
result is the null string, for every `count` that passes its check.

```rexx
say '['changestr('a', '', 'x')']' '['changestr('a', '', 'x', 3)']'
signal on syntax name a
say changestr('a', '', 'x', -1)
a: say condition('o')~code
```

```output
[] []
93.906
```

r[bif.changestr.empty-newneedle]
When `newneedle` is the null string, the occurrences replaced are deleted:
the result is `haystack` without them. This is not the same as omitting
`newneedle`, which [is an error](r:bif.index.call.missing).

```rexx
say changestr('a', 'banana', '') '['changestr('banana', 'banana', '')']'
say changestr('a', 'banana', '', 2)
```

```output
bnn []
bnna
```

## Deviations

r[bif.changestr.deviation.4]
interrexx does not specify whether the result of CHANGESTR is the same
object as another string of equal value: as `haystack` itself, when the call
replaces nothing, or as an equal string produced some other way, such as the
literal `'banana'`. Through `identityHash`, or as a key of an `IdentityTable`
or `IdentitySet`, the answer may be either, and a program can rely only on
the value, which is equal under `==`. ooRexx differs: it often answers a new
object where interrexx answers the existing one.

```rexx
s = 'ban' || 'ana'
say (changestr('x', s, 'y') == s) (changestr('', s, 'y') == 'banana')
```

```output
1 1
```

> **Note.** The ANSI draft (9.3.4) has no `count` argument; it replaces every
> occurrence, searching from the end of the previous one, which gives the
> rules above for the call without `count`. The `count` argument and its
> checks follow ooRexx.
