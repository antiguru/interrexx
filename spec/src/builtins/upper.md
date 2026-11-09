---
element: UPPER
kind: builtin
syntax: "UPPER(string [, n] [, length])"
rexxref: "Functions > Built-in Functions > UPPER"
deviations: [4]
---

# UPPER

UPPER changes the lowercase letters of a string, or of a run of its
characters, to uppercase, and leaves every other character as it is.

## Arguments

r[bif.upper.method]
UPPER is a builtin that runs as a String method, and `string`, its first
argument, is the receiver; `n` and `length` become the message's
arguments, in that order. `string` is a [string](r:conv.string.null), `n`
is a [position](r:conv.position.check), and `length` is a
[length](r:conv.length.check). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): one is required and three is
the most.

```rexx
say upper('mixed Case') 'mixed Case'~upper(7)
signal on syntax name a
say upper('abc', 0)
a: say condition('o')~code
signal on syntax name b
say upper('abc', 1, -2)
b: say condition('o')~code
signal on syntax name c
say upper('abc', 0, 'two')
c: say condition('o')~code
signal on syntax name d
say upper('abc', 1, 1, 1)
d: say condition('o')~code
signal on syntax name f
say upper(, 2)
f: say condition('o')~code
```

```output
MIXED CASE mixed CASE
93.924
93.923
40.12
40.4
40.5
```

r[bif.upper.n.default]
When `n` is omitted, it is 1: the run starts at the first character of
`string`.

```rexx
say upper('xyz', , 2) upper('xyz', 1, 2)
```

```output
XYz XYz
```

r[bif.upper.length.default]
When `length` is omitted, the run is every character of `string` at
position `n` and after it, and no character when `n` is greater than
`LENGTH(string)`.

```rexx
say upper('lower', 3) upper('lower', 5) upper('lower', 6)
```

```output
loWER loweR lower
```

## The result

r[bif.upper.case]
Uppercasing changes exactly the 26 characters `'a'` to `'z'`, `'61'x` to
`'7A'x`, each to the letter `'A'` to `'Z'` with the same place in the
alphabet, `'41'x` to `'5A'x`. Every other character is left as it is: the
uppercase letters, the digits, whitespace, `'00'x`, and every character from
`'80'x` to `'FF'x`, including the bytes that an encoding such as Latin-1 or
UTF-8 uses for a lowercase letter.

```rexx
say upper('q-z 0?_') c2x(upper('e9c3a9ff'x)) c2x(upper('60617a7b'x))
all = xrange('00'x, 'ff'x)
say (upper(all) == translate(all, xrange('A', 'Z'), xrange('a', 'z')))
```

```output
Q-Z 0?_ E9C3A9FF 60415A7B
1
```

> **Note.** The reference manual's UPPER section does not name the characters it
> changes; its TRANSLATE section, which uppercases the same way, names the
> letters `a` to `z`, and this rule follows that. The ANSI draft has no UPPER
> builtin, and leaves the uppercase form of each character to the
> configuration (5.4.3).

r[bif.upper.result]
UPPER returns a string of `LENGTH(string)` characters. The character at
each position from `n` to `n + length - 1` that `string` has is
[uppercased](r:bif.upper.case), and every other character of `string` is
copied unchanged. A run that reaches past the end of `string` stops at the
end, and nothing is added.

```rexx
say '['upper('string', 2, 3)']' '['upper('string', 4, 100)']' '['upper('ab cd')']'
```

```output
[sTRIng] [strING] [AB CD]
```

r[bif.upper.past-end]
When `n` is greater than `LENGTH(string)`, no character is changed and the
result is `string`, for every `n` past the end that passes its check.

```rexx
say upper('abc', 4) upper('abc', 999999999999999999, 2)
```

```output
abc abc
```

r[bif.upper.length.zero]
When `length` is 0, no character is changed and the result is `string`,
for every `n` that passes its check.

```rexx
say upper('abc', 2, 0) upper('abc', 50, 0)
```

```output
abc abc
```

r[bif.upper.empty]
When `string` is the null string, the result is the null string, for every
`n` and `length` that pass their checks.

```rexx
say '['upper('')']' '['upper('', 3, 2)']'
```

```output
[] []
```

## Deviations

> **Note.** An argument whose `STRING` method answers an object that is not a
> string falls under [Deviation 30](r:conv.string.deviation.30), as it does
> for every builtin.

r[bif.upper.deviation.4]
interrexx does not specify whether the answer of UPPER is the same object as
another string of equal value: as `string` itself, when no character of it
changes, as in `upper(s)` for an `s` with no lowercase letter, or as an equal
string produced some other way, such as the literal `'GHIJ'` beside
`upper('gh' || 'ij')`. Through `identityHash`, or as a key of an
`IdentityTable`, the answer may be either, and a program can rely only on the
value, which is equal under `==`. ooRexx differs: it often answers a new
object where interrexx answers the existing one.

```rexx
s = 'PQ' || 'RS'
say (upper(s) == s) (upper('gh' || 'ij') == 'GHIJ')
```

```output
1 1
```
