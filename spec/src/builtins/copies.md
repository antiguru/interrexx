---
element: COPIES
kind: builtin
syntax: "COPIES(string, n)"
ansi: "9.3.6"
rexxref: "Functions > Built-in Functions > COPIES"
deviations: [4]
---

# COPIES

COPIES repeats a string: it answers the string written one after another
`n` times.

## Arguments

r[bif.copies.method]
COPIES is a builtin that runs as a String method, and `string`, its first
argument, is the receiver; `n` becomes the message's argument. `string` is a
[string](r:conv.string.null) and `n` is a
[whole number](r:conv.whole.form) of the kind
[the next rule](r:bif.copies.n.check) describes. The argument count is
checked by the [builtin call rules](r:bif.index.call.count): both arguments
are required and two is the most.

```rexx
say copies('abc', 2) 'abc'~copies(2)
signal on syntax name a
say copies('abc')
a: say condition('o')~code
signal on syntax name b
say copies('abc', 2, 1)
b: say condition('o')~code
signal on syntax name c
say copies(, 2)
c: say condition('o')~code
signal on syntax name d
say copies('abc', )
d: say condition('o')~code
signal on syntax name e
say copies('abc', 'x', 1)
e: say condition('o')~code
```

```output
abcabc abcabc
40.3
40.4
40.5
40.3
40.4
```

r[bif.copies.n.check]
`n` must be a [whole number](r:conv.whole.form) of the
[value](r:conv.whole.digits) the conventions describe, and not negative.
An `n` that is not a whole number, including the null string, raises
`SYNTAX 40.12`; an `n` that is a whole number and negative raises
`SYNTAX 93.906`. An `n` of 0 is allowed, and so is `-0`, which is 0. The form
check comes before the range check, as in
[the order of the checks](r:conv.check.order).

```rexx
say '['copies('rst', 0)']' '['copies('abc', '-0')']' copies('ab', ' + 2. ')
signal on syntax name a
say copies('abc', -1)
a: say condition('o')~code
signal on syntax name b
say copies('abc', '')
b: say condition('o')~code
signal on syntax name c
say copies('abc', '2.5')
c: say condition('o')~code
signal on syntax name d
say copies('abc', 'x')
d: say condition('o')~code
signal on syntax name e
say copies('abc', '-1E0')
e: say condition('o')~code
signal on syntax name f
say copies('abc', 1000000000000000000)
f: say condition('o')~code
```

```output
[] [] abab
93.906
40.12
40.12
40.12
93.906
40.12
```

> **Note.** The code is not 93.923, which [the length rule](r:conv.length.check)
> gives to the length of other builtins: COPIES' count is not a length
> argument.

## The result

r[bif.copies.result]
COPIES returns the characters of `string` written `n` times in a row: a
string of `n * LENGTH(string)` characters whose characters at positions
`(j - 1) * LENGTH(string) + 1` through `j * LENGTH(string)`, for each `j`
from 1 to `n`, are the characters of `string`.

```rexx
say copies('rst', 4) copies('ab', 1) copies('a', 5) copies(12, 2)
say length(copies('ab', 100000)) '['copies(' ', 2)']'
```

```output
rstrstrstrst ab aaaaa 1212
200000 [  ]
```

r[bif.copies.zero]
When `n` is 0, the result is the null string, for every `string`.

```rexx
say '['copies('rst', 0)']' '['copies('', 0)']' '['copies(' ', 0)']'
```

```output
[] [] []
```

r[bif.copies.empty]
When `string` is the null string, the result is the null string, for every
`n` that passes its check, a very large one included.

```rexx
say '['copies('', 5)']' length(copies('', 999999999999999999))
```

```output
[] 0
```

r[bif.copies.bytes]
COPIES copies [characters](r:conv.string.chars) of `string` unchanged:
whitespace, control characters and `'00'x` are characters like any other.

```rexx
say c2x(copies('a' || '00'x, 3)) c2x(copies('09'x || 'c3a9'x, 2))
```

```output
610061006100 09C3A909C3A9
```

> **Note.** Whether, when and with which code COPIES fails for a count whose
> result cannot be held in memory is not specified.

## Deviations

r[bif.copies.deviation.4]
interrexx does not specify whether the result of COPIES is the same object
as another string of equal value: as `string` itself, when the call leaves
it unchanged as `copies(s, 1)` does, or as an equal string produced some
other way, such as the literal `'abab'` beside `copies('ab', 2)`, or the
null string answered when `n` is 0 or `string` is null. Through
`identityHash`, or as a key of an `IdentityTable` or `IdentitySet`, the
answer may be either, and a program can rely only on the value, which is
equal under `==`. ooRexx differs: it often answers a new object where
interrexx answers the existing one.

```rexx
s = 'ab' || 'cd'
say (copies(s, 1) == s) (copies('ab', 2) == 'abab') (copies(s, 0) == '')
```

```output
1 1 1
```
