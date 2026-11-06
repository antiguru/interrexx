---
element: REVERSE
kind: builtin
syntax: "REVERSE(string)"
ansi: "9.3.17"
rexxref: "Functions > Built-in Functions > REVERSE"
deviations: [4]
---

# REVERSE

REVERSE answers a string with the characters of its argument in the opposite
order.

## Arguments

r[bif.reverse.method]
REVERSE is a builtin that runs as a String method, and `string`, its only
argument, is the receiver; the message takes no other argument. `string` is
a [string](r:conv.string.null), so no string raises an error. The
argument count is checked by the [builtin call rules](r:bif.index.call.count):
one is required and one is the most.

```rexx
say reverse('Xyz!') 'Xyz!'~reverse
signal on syntax name a
say reverse()
a: say condition('o')~code
signal on syntax name b
say reverse('a', 'b')
b: say condition('o')~code
signal on syntax name c
say reverse(,)
c: say condition('o')~code
```

```output
!zyX !zyX
40.3
40.4
40.3
```

## The result

r[bif.reverse.result]
Let `n` be the length of `string`. REVERSE returns a string of length `n`
whose character at position `i`, for each `i` from 1 to `n`, is the
character of `string` at position `n - i + 1`. The null string, and a string
of one character, are their own reverse. Reversing a string twice returns the
original characters.

```rexx
say reverse('Xyz!') reverse('QRS ')'|' '['reverse('')']' reverse('a')
say reverse(reverse('stressed')) reverse('stressed')
```

```output
!zyX  SRQ| [] a
stressed desserts
```

r[bif.reverse.bytes]
REVERSE reverses [characters](r:conv.string.chars), which are single bytes, so
the bytes of a character that an encoding writes as several come out in the
opposite order. Every character moves unchanged, whatever it is: blanks,
tabs, line feeds and `'00'x` included, with leading and trailing blanks
becoming trailing and leading ones.

```rexx
say c2x(reverse('c3a9'x)) c2x(reverse('0061'x)) c2x(reverse('0a'x || 'b' || '09'x))
say '['reverse('  ab ')']'
```

```output
A9C3 6100 09620A
[ ba  ]
```

r[bif.reverse.string-form]
When `string` is a number, REVERSE reverses the characters of the string the
number is, as it is written, and the result is a string of those characters
and nothing more: a sign, a period or an exponent moves like any other
character. A quoted literal keeps its spelling; an unquoted one is a symbol,
so its exponent letter is `E` whatever the source wrote. When `string` is an
object other than a string, it is [replaced by its
string](r:conv.string.object).

```rexx
say reverse(123) reverse(-1.50) reverse(.object~new) reverse(1e3) reverse('1e3')
numeric digits 5
say reverse(1/3)
```

```output
321 05.1- tcejbO na 3E1 3e1
33333.0
```

## Deviations

r[bif.reverse.deviation.4]
interrexx does not specify whether the result of REVERSE is the same object
as another string of equal value: as `string` itself, when the reversal leaves
it unchanged as it does for the null string, or as an equal string produced
some other way, such as the literal `'cba'`. Through `identityHash`, or as a
key of an `IdentityTable` or `IdentitySet`, the answer may be either, and a
program can rely only on the value, which is equal under `==`. ooRexx differs:
it often answers a new object where interrexx answers the existing one.

```rexx
s = 'a' || 'bc'
say (reverse(s) == 'cba') (reverse(reverse(s)) == s) (reverse('') == '')
```

```output
1 1 1
```

> **Note.** The ANSI draft (9.3.17) builds the result one character at a time,
> putting each character in front of those before it, which gives the string
> above.
