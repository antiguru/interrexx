---
element: LOWER
kind: builtin
syntax: "LOWER(string [, n] [, length])"
rexxref: "Functions > Built-in Functions > LOWER"
deviations: [4]
---

# LOWER

LOWER changes the uppercase letters of a string, or of a run of its
characters, to lowercase, and leaves every other character as it is. It is
the counterpart of [UPPER](upper.md).

## Arguments

r[bif.lower.method]
LOWER is a builtin that runs as a String method, and `string`, its first
argument, is the receiver; `n` and `length` become the message's
arguments, in that order. `string` is a [string](r:conv.string.null), `n`
is a [position](r:conv.position.check), and `length` is a
[length](r:conv.length.check). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.index.call.count): one is required and three is
the most.

```rexx
say lower('Mixed Case') 'Mixed Case'~lower(7)
signal on syntax name a
say lower('ABC', 0)
a: say condition('o')~code
signal on syntax name b
say lower('ABC', 1, -2)
b: say condition('o')~code
signal on syntax name c
say lower('ABC', 0, 'two')
c: say condition('o')~code
signal on syntax name d
say lower('ABC', 1, 1, 1)
d: say condition('o')~code
signal on syntax name f
say lower(, 2)
f: say condition('o')~code
```

```output
mixed case Mixed case
93.924
93.923
40.12
40.4
40.5
```

r[bif.lower.n.default]
When `n` is omitted, it is 1: the run starts at the first character of
`string`.

```rexx
say lower('XYZ', , 2) lower('XYZ', 1, 2)
```

```output
xyZ xyZ
```

r[bif.lower.length.default]
When `length` is omitted, the run is every character of `string` at
position `n` and after it, and no character when `n` is greater than
`LENGTH(string)`.

```rexx
say lower('UPPER', 3) lower('UPPER', 5) lower('UPPER', 6)
```

```output
UPper UPPEr UPPER
```

## The result

r[bif.lower.case]
Lowercasing changes exactly the 26 characters `'A'` to `'Z'`, `'41'x` to
`'5A'x`, each to the letter `'a'` to `'z'` with the same place in the
alphabet, `'61'x` to `'7A'x`. Every other character is left as it is: the
lowercase letters, the digits, whitespace, `'00'x`, and every character from
`'80'x` to `'FF'x`, including the bytes that an encoding such as Latin-1 or
UTF-8 uses for an uppercase letter.

```rexx
say lower('Q-Z 0?_') c2x(lower('C9C3A9DE'x)) c2x(lower('40415a5b'x))
all = xrange('00'x, 'ff'x)
say (lower(all) == translate(all, xrange('a', 'z'), xrange('A', 'Z')))
```

```output
q-z 0?_ C9C3A9DE 40617A5B
1
```

> **Note.** The reference manual's LOWER section does not name the characters it
> changes; this rule is the mirror of [the UPPER rule](r:bif.upper.case),
> and ooRexx lowercases exactly those letters. The ANSI draft has no LOWER
> builtin.

r[bif.lower.result]
LOWER returns a string of `LENGTH(string)` characters. The character at
each position from `n` to `n + length - 1` that `string` has is
[lowercased](r:bif.lower.case), and every other character of `string` is
copied unchanged. A run that reaches past the end of `string` stops at the
end, and nothing is added.

```rexx
say '['lower('STRING', 2, 3)']' '['lower('STRING', 4, 100)']' '['lower('AB CD')']'
```

```output
[StriNG] [STRing] [ab cd]
```

r[bif.lower.past-end]
When `n` is greater than `LENGTH(string)`, no character is changed and the
result is `string`, for every `n` past the end that passes its check.

```rexx
say lower('ABC', 4) lower('ABC', 999999999999999999, 2)
```

```output
ABC ABC
```

r[bif.lower.length.zero]
When `length` is 0, no character is changed and the result is `string`,
for every `n` that passes its check.

```rexx
say lower('ABC', 2, 0) lower('ABC', 50, 0)
```

```output
ABC ABC
```

r[bif.lower.empty]
When `string` is the null string, the result is the null string, for every
`n` and `length` that pass their checks.

```rexx
say '['lower('')']' '['lower('', 3, 2)']'
```

```output
[] []
```

## Deviations

> **Note.** An argument whose `STRING` method answers an object that is not a string
> falls under [Deviation 30](r:conv.string.deviation.30), as it does for
> every builtin.

r[bif.lower.deviation.4]
interrexx does not specify whether the result of LOWER is the same object
as another string of equal value: as `string` itself, when no character of
it changes, as in `lower(s)` for an `s` with no uppercase letter, or as an
equal string produced some other way, such as the literal `'ghij'` beside
`lower('GH' || 'IJ')`. Through `identityHash`, or as a key of an
`IdentityTable` or `IdentitySet`, the answer may be either, and a program
can rely only on the value, which is equal under `==`. ooRexx differs:
it often answers a new object where interrexx answers the existing one.

```rexx
s = 'pq' || 'rs'
say (lower(s) == s) (lower('GH' || 'IJ') == 'ghij')
```

```output
1 1
```
