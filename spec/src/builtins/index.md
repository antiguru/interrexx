---
element: "Built-in functions"
kind: chapter
ansi: "9.2.1"
rexxref: "Functions > Built-in Functions"
deviations: [4]
---

# Built-in functions

This chapter has a page for each builtin function, and each page specifies
the function call. The rules here hold for every builtin call; the pages
link to them, and to the [conventions](../conventions.md), instead of
restating them.

A page's syntax line names the arguments in order. A name in square
brackets is an optional argument; every other argument is required. The
number of names on the line is the most arguments the builtin takes, and
the number of required ones is the fewest.

r[bif.index.call.count]
The argument count of a call is the position of the last argument in it
that is not [omitted](r:conv.arg.omitted). Omitted places at the end of the
list therefore add nothing to the count, and a call with only omitted
places counts as a call with no arguments, a count of 0.

```rexx
say substr('abc', 2, , ) word('a b', 2, , , )
```

```output
bc b
```

r[bif.index.call.too-few]
A call whose argument count is less than the number of required arguments
raises `SYNTAX 40.3`.

```rexx
signal on syntax name a
say substr('abc')
a: say condition('o')~code
signal on syntax name b
say substr('abc', )
b: say condition('o')~code
signal on syntax name c
say word(, , )
c: say condition('o')~code
```

```output
40.3
40.3
40.3
```

r[bif.index.call.too-many]
A call whose argument count is greater than the number of arguments on the
syntax line raises `SYNTAX 40.4`, whatever the values of the arguments,
and even when the arguments before the last are omitted.

```rexx
signal on syntax name a
say word('a b', 1, 2)
a: say condition('o')~code
signal on syntax name b
say substr(, , , , 1)
b: say condition('o')~code
```

```output
40.4
40.4
```

r[bif.index.call.missing]
A required argument that is omitted, in a call whose argument count reaches
past it, raises `SYNTAX 40.5`.

```rexx
signal on syntax name a
say substr(, 1)
a: say condition('o')~code
signal on syntax name b
say wordpos('a', , 1)
b: say condition('o')~code
```

```output
40.5
40.5
```

r[bif.index.call.order.count-before-missing]
A call whose argument count is wrong raises the [too-few](r:bif.index.call.too-few)
or [too-many](r:bif.index.call.too-many) error even when it also omits a
[required argument](r:bif.index.call.missing).

```rexx
signal on syntax name a
say word(, 1, 2)
a: say condition('o')~code
```

```output
40.4
```

r[bif.index.call.order.count-before-values]
A call whose argument count is wrong raises the too-few or too-many error
even when an argument it does pass would fail its own check.

```rexx
signal on syntax name a
say substr('abc', 'x', 1, '**', 5)
a: say condition('o')~code
signal on syntax name b
say word('a b', 0, 2)
b: say condition('o')~code
```

```output
40.4
40.4
```

r[bif.index.call.order.missing-before-values]
A call whose argument count is right but that omits a
[required argument](r:bif.index.call.missing) raises `SYNTAX 40.5` even when an
argument it passes after the omitted one would fail its own check, a
[form check](r:conv.check.order) or a range check.

```rexx
signal on syntax name a
say substr(, 'x')
a: say condition('o')~code
signal on syntax name b
say subword(, 0)
b: say condition('o')~code
signal on syntax name c
say wordpos(, 'a', 0)
c: say condition('o')~code
signal on syntax name d
say verify('abc', , 'Y')
d: say condition('o')~code
```

```output
40.5
40.5
40.5
40.5
```

> **Note.** The ANSI draft (9.2.1) checks the arguments from left to right, raising
> 40.5 at an omitted required argument before it reaches the ones after it,
> and makes the range checks only once every argument is present.

## Words

r[bif.index.words.split]
The builtins that count words split a string into words the same way. A
word is a run of one or more characters, none of them
[whitespace](r:conv.string.whitespace), that has whitespace or an end of
the string on each side. The words of a string are numbered from 1, left to
right. Whitespace before the first word, between two words and after the
last belongs to no word, so the null string and a string of only whitespace
have no words. Every other character, line feed and `'00'x` among them, can
be part of a word.

```rexx
say words(' a'||'09'x||'b  c ') words('') words('  '||'09'x)
say word('one  two', 2) c2x(word('a'||'0a000c'x||'b c', 1))
```

```output
3 0 0
two 610A000C62
```

> **Note.** The ANSI draft (9.3.26 and the other word builtins) separates words by a
> set of blank characters that it leaves to the configuration: the blank and
> any others the configuration names. ooRexx fixes that set as the blank and
> the horizontal tab, and so does this book.

r[bif.index.words.number]
Where a word builtin's page calls an argument a word number, that argument
is a [position](r:conv.position.check) that counts words: the word number
`n` names word `n` of the string as [the word rule](r:bif.index.words.split)
numbers them. A word number greater than the number of words in the string
is not an error, for every value that passes its check; it names no word,
and the builtin's page says what the builtin answers then.

```rexx
say '['word('a b', 3)']' wordindex('a b', 3) wordlength('a b', 999999999999999999),
    wordpos('a', 'a b', 3)
signal on syntax name e
say wordindex('a b', 0)
e: say condition('o')~code
```

```output
[] 0 0 0
93.924
```

## Answers that are counts

r[bif.index.count.answer]
A builtin whose page links this rule answers a count: of characters, of
words or of occurrences, or a position, which counts characters or words up
to the one it names. Such a builtin writes it as a whole
number in decimal digits only: no sign, no leading zero, no period and no exponent, and `0`
for zero. The caller's NUMERIC DIGITS setting plays no part, so the answer
is exact however many digits it has.

```rexx
numeric digits 3
say wordindex(copies(' ', 1000) || 'a', 1) wordlength(copies('z', 1234), 1),
    wordpos('y', copies('x ', 1200) || 'y') wordpos('q', 'a b')
say compare(copies('a', 1500) || 'b', copies('a', 1500) || 'c') countstr('a', copies('a', 1500))
say verify(copies('a', 1200) || 'b', 'a') verify(copies('a', 1300), '', 'N', 1234),
    verify('ab', 'ab')
```

```output
1001 1234 1201 0
1501 1500
1201 1234 0
```

> **Note.** The reference manual does not say how these answers are written or whether
> NUMERIC DIGITS applies to them; this rule matches ooRexx.

r[bif.index.count.answer.deviation.4]
interrexx does not specify whether the answer of a builtin that answers a
count is the same object as another string of equal value: as an equal
number the program already holds, such as the `start` argument of a call
that answers it, or a literal such as `'1201'`, or the equal answer of
another call. Through `identityHash`, or as a key of an `IdentityTable` or
`IdentitySet`, the answer may be either, and a program can rely only on the
value, which is equal under `==`. ooRexx differs: it often answers a
new object where interrexx answers the existing one.

```rexx
w = copies('x ', 1200) || 'y'
n = 1200 + 1
say (wordpos('y', w, n) == n) (wordindex(copies(' ', 1200) || 'a', 1) == '1201'),
    (wordlength(copies('z', 1201), 1) == wordpos('y', w))
v = copies('a', 1300)
say (verify(v, '', 'N', n) == n) (verify(copies('a', 1200) || 'b', 'a') == '1201')
```

```output
1 1 1
1 1
```

## Bit-by-bit functions

r[bif.index.bit.args]
BITAND, BITOR and BITXOR take the same arguments: `string1`, `string2` and
`pad`. Each runs as a String method, and `string1`, its first argument, is
the receiver; `string2` and `pad` become the message's arguments, in that
order. `string1` and `string2` are [strings](r:conv.string.null) and `pad` is
a [pad](r:conv.pad). Only `string1` is required, `string2` and `pad` are
optional, and the argument count is checked by the
[builtin call rules](r:bif.index.call.count): one is required and three is the
most.

```rexx
say c2x(bitor('a0'x)) c2x('a0'x~bitxor('01'x)) c2x('a0'x~bitand('f0'x, '00'x))
signal on syntax name a
say bitand()
a: say condition('o')~code
signal on syntax name b
say bitor('a', 'b', 'c', 'd')
b: say condition('o')~code
signal on syntax name c
say bitxor(, 'b')
c: say condition('o')~code
```

```output
A0 A1 A0
40.3
40.4
40.5
```

r[bif.index.bit.pad.check]
A `pad` that is present is checked as [a pad](r:conv.pad) whether or not the
call needs it: a `pad` that is not one character long raises `SYNTAX 40.23`
even when the operands are equally long or both null, so that no character
of it would be used, and the null string is not a pad, as [an empty argument](r:conv.arg.empty) is not
omitted.

```rexx
signal on syntax name a
say bitand('a', 'b', '')
a: say condition('o')~code
signal on syntax name b
say bitor('', '', '')
b: say condition('o')~code
signal on syntax name c
say bitxor('ab', 'cd', 'xy')
c: say condition('o')~code
say c2x(bitor('ab', 'cd', 'x')) '['bitxor('', '', 'x')']'
```

```output
40.23
40.23
40.23
6366 []
```

r[bif.index.bit.bytes]
Each character of the result is made from the characters at the same
position in the two operands: bit `k` of it is the function's operation on
bit `k` of each, for each of the eight bits of a character, and nothing else
about the operands matters. Every character code is an operand like any
other. A string of digits is the characters of its digits, not a number, so
NUMERIC DIGITS plays no part.

```rexx
numeric digits 1
say c2x(bitxor(12, 5)) c2x(bitand('ff80'x, '80ff'x)) c2x(bitor('00ff'x, 'f00f'x))
```

```output
0432 8080 F0FF
```

r[bif.index.bit.nopad]
When `pad` is omitted, the answer is as long as the longer operand. The
characters of the shorter one are combined with the characters at the same
positions of the longer, and the characters of the longer beyond the end of
the shorter are copied into the answer unchanged, whichever of the two is
the longer.

```rexx
say c2x(bitand('ff00ff'x, '0f'x)) c2x(bitand('0f'x, 'ff00ff'x))
say c2x(bitxor('ab', 'a')) c2x(bitor('a', 'ab'))
```

```output
0F00FF 0F00FF
0062 6162
```

r[bif.index.bit.pad]
When `pad` is present, copies of `pad` are appended to the shorter operand
until the two are equally long, and the two are then combined character by
character. The answer is as long as the longer
operand, and each character of it beyond the end of the shorter operand is
the operation applied to the longer's character and `pad`.

```rexx
say c2x(bitand('ff00ff'x, '0f'x, 'f0'x)) c2x(bitand('0f'x, 'ff00ff'x, 'f0'x))
say c2x(bitxor('ab', 'a', ' ')) c2x(bitor('a', 'ab', ' '))
```

```output
0F00F0 0F00F0
0042 6162
```

r[bif.index.bit.string2-default]
An omitted `string2` is the null string, and a `string2` given as the null
string behaves the same. The answer is then `string1` itself when `pad` is omitted, and with
a `pad` it is each character of `string1` combined with `pad`.

```rexx
say c2x(bitand('abc')) c2x(bitor('abc', '')) c2x(bitxor('abc', , 'ff'x))
say c2x(bitand('abc', , '0f'x)) c2x(bitor('abc', '', '0f'x))
```

```output
616263 616263 9E9D9C
010203 6F6F6F
```

r[bif.index.bit.commutative]
Exchanging the first two arguments does not change the answer, with a `pad`
or without one.

```rexx
say (bitand('abcd', 'ef') == bitand('ef', 'abcd')),
    (bitor('abcd', 'ef', 'x') == bitor('ef', 'abcd', 'x'))
say (bitxor('abc', 'z', 'q') == bitxor('z', 'abc', 'q')) (bitxor('', 'z') == bitxor('z', ''))
```

```output
1 1
1 1
```

r[bif.index.bit.deviation.4]
interrexx does not specify whether the answer of a bit-by-bit function is
the same object as another string of equal value: as `string1` itself when
the answer equals it, as in `bitor(s)`, or as an equal string produced some
other way, such as a literal. Through `identityHash`, or as a key of an
`IdentityTable` or `IdentitySet`, the answer may be either, and a program can
rely only on the value, which is equal under `==`. ooRexx differs: it often
answers a new object where interrexx answers the existing one.

```rexx
s = 'ab' || 'cd'
say (bitor(s) == s) (bitand('', s) == 'abcd') (bitand('a', 'a') == 'a')
```

```output
1 1 1
```

## Conversion functions

> **Note.** B2X, C2D, C2X, D2C, D2X, X2B, X2C and X2D convert between the characters
> of a string, a number written in decimal, and a string of hexadecimal or
> binary digits. The rules below are the ones that more than one of them
> follows; a function's page links the ones it uses and states the rest.

r[bif.index.conv.digits]
In an argument that is a [string](r:conv.string.object) of binary digits,
the digits are `0` and `1`. In an argument that is a string of
hexadecimal digits, the digits are
`0` to `9` and the letters `A` to `F`, each in either case, and the two
cases may be mixed. Any character other than a digit that is not
[whitespace](r:conv.string.whitespace) raises `SYNTAX 93.934` in a binary
string and `SYNTAX 93.933` in a hexadecimal string. A sign, a period, a
letter past `F`, a decimal digit above 1 in a binary string, line feed and
`'00'x` are all such characters.

```rexx
say b2x('0110') x2c('4a4B') x2b('aF')
signal on syntax name a
say b2x('102')
a: say condition('o')~code
signal on syntax name b
say b2x('1' || '0a'x)
b: say condition('o')~code
signal on syntax name c
say x2c('4G')
c: say condition('o')~code
signal on syntax name d
say x2c('-4')
d: say condition('o')~code
```

```output
6 JK 10101111
93.934
93.934
93.933
93.933
```

r[bif.index.conv.digits.null]
The null string is a valid digit string, binary or hexadecimal: it has no
digits. B2X, X2B and X2C answer the null string for it, and X2D answers 0.

```rexx
say '['b2x('')']' '['x2b('')']' '['x2c('')']' x2d('')
```

```output
[] [] [] 0
```

r[bif.index.conv.groups]
A digit string may have whitespace between its digits, which is not read as
part of the number. A run of any number of blanks and tabs, in any mix,
counts as one gap. A gap is allowed only where the digits that follow it,
up to the end of the string, can be read in whole units:
a multiple of 4 binary digits (the digits of one hexadecimal digit) in a
binary string, and a multiple of 2 hexadecimal digits (the digits of one
character) in a hexadecimal string. The first group of digits, before the
first gap, may have any number of digits. A gap that does not fall at such a
boundary raises `SYNTAX 93.977` in a binary string and `SYNTAX 93.976` in a
hexadecimal string.

```rexx
say b2x('10 0101 1100') b2x('1  1111') b2x('1111' || '09'x || ' 0000')
say x2c('4a 4B') c2x(x2c('4 4a4b')) x2d('1 23 45')
signal on syntax name a
say b2x('11 11')
a: say condition('o')~code
signal on syntax name b
say b2x('1 111 1')
b: say condition('o')~code
signal on syntax name c
say x2c('4A 4')
c: say condition('o')~code
signal on syntax name d
say x2c('1 2 3')
d: say condition('o')~code
```

```output
25C 1F F0
JK 044A4B 74565
93.977
93.977
93.976
93.976
```

r[bif.index.conv.groups.edge]
A digit string that starts or ends with whitespace raises `SYNTAX 93.932` in
a binary string and `SYNTAX 93.931` in a hexadecimal string, as does a
string that has only whitespace and so no digit. The null string is not
such a string.

```rexx
signal on syntax name a
say b2x(' 1111')
a: say condition('o')~code
signal on syntax name b
say b2x('1111 ')
b: say condition('o')~code
signal on syntax name c
say b2x('  ')
c: say condition('o')~code
signal on syntax name d
say x2c(' 41')
d: say condition('o')~code
signal on syntax name e
say x2b('41' || '09'x)
e: say condition('o')~code
signal on syntax name f
say x2d('  ')
f: say condition('o')~code
```

```output
93.932
93.932
93.932
93.931
93.931
93.931
```

> **Note.** The reference manual says whitespace may not lead or trail, and
> that groups are at 4-digit boundaries for binary strings and at byte boundaries for
> hexadecimal ones. When a string has more than one of the faults above
> (such as a non-digit and a misplaced gap), this book does not say which of
> the errors is raised; the examples above have one fault each.

r[bif.index.conv.length]
Where a conversion function takes a length `n`, it is an optional
[whole-number argument](r:conv.whole.form) whose value must not be negative
([a length](r:conv.length.check)): a negative `n` raises `SYNTAX 93.923`, and
an `n` of 0 is allowed. An omitted `n` is not the same as an `n` of 0, and
the null string is not omitted. Its unit is the character in C2D and D2C and the
hexadecimal digit in D2X and X2D.

```rexx
signal on syntax name a
say c2d('a', -1)
a: say condition('o')~code
signal on syntax name b
say d2x(65, -1)
b: say condition('o')~code
signal on syntax name c
say c2d('a', '')
c: say condition('o')~code
signal on syntax name d
say d2c(65, 'x')
d: say condition('o')~code
say c2d('a') c2d('a', 0) c2d('a', '-0') '['d2c(65, 0)']' '['d2x(65, 0)']'
```

```output
93.923
93.923
40.12
40.12
97 0 0 [] []
```

r[bif.index.conv.signed.read]
When C2D or X2D is given an `n`, the string is read as a signed number of
exactly `n` units. It is first made `n` units long: units are removed from
its left when it is longer, and `'00'x` units (zero digits in X2D) are
added there when it is shorter. The result is then
read as a two's-complement number: when the leftmost bit of the leftmost
unit is set, the number is negative, and its value is the unsigned value minus
256\*\*`n` in C2D and 16\*\*`n` in X2D. An `n` of 0 answers 0. Without an `n`,
the string is read as a number with no sign, whatever its leftmost bit,
and the answer is never negative.

```rexx
say c2d('93'x) c2d('93'x, 1) c2d('93'x, 2) c2d('e3a1'x, 1) c2d('e3a1'x, 2)
say c2d('ff5a'x, 1) c2d('0107'x, 0) c2d('', 3)
say x2d('93') x2d('93', 2) x2d('93', 3)
say x2d('e3a1', 2) x2d('e3a1', 3) x2d('e3a1', 4)
```

```output
147 -109 147 -95 -7263
90 0 0
147 -109 147
-95 929 -7263
```

r[bif.index.conv.signed.write]
When D2C or D2X is given an `n`, the answer is exactly `n` units long. It
is the number's remainder on division by 256\*\*`n` in D2C and 16\*\*`n` in
D2X, a remainder that is never negative, written with `'00'x` units (`0`
digits in D2X) on its left to make `n` units. A small negative number
therefore has `'FF'x` units (`F` digits in D2X) on its left, and for a
number that needs more than `n` units the leftmost ones are lost, so -257
with an `n` of 1 is `'FF'x`. An `n` of 0 answers the null string.
Without an `n`, a non-negative number is written in as few units as it
takes, and at least one, so that no `'00'x` (or `0`) stands on its left
unless the number is 0; a negative number raises `SYNTAX 93.927`.

```rexx
say c2x(d2c(72)) c2x(d2c(65, 3)) c2x(d2c(-65, 3)) c2x(d2c(321, 1))
say c2x(d2c(-129, 1))
say d2x(65) d2x(65, 4) d2x(-65, 4) d2x(321, 2) d2x(-129, 2)
say c2x(d2c(0)) d2x(0) c2x(d2c(256)) d2x(256) c2x(d2c(-256, 1)),
    c2x(d2c(-256, 2))
say c2x(d2c(-257, 1)) d2x(-4097, 2) c2x(d2c(-65537, 2))
signal on syntax name a
say d2c(-1)
a: say condition('o')~code
signal on syntax name b
say d2x(-65)
b: say condition('o')~code
```

```output
48 000041 FFFFBF 41
7F
41 0041 FFBF 41 7F
00 0 0100 100 00 FF00
FF FF FFFF
93.927
93.927
```

r[bif.index.conv.decimal]
The decimal argument of D2C and D2X is a [string](r:conv.string.object)
written as a Rexx number, with optional whitespace around it and after its
sign, a period and an exponent, whose value is a whole number of at most
NUMERIC DIGITS digits; leading zeros and zeros after the point are not
counted, and zeros at the end of the whole part are.
Each of these raises `SYNTAX 93.929` in D2C and `SYNTAX 93.928` in D2X: a
string that is not a number, the null string among them; a number with a
nonzero fractional part, when it is written with no more digits than
NUMERIC DIGITS; and a whole number with more digits than NUMERIC DIGITS.
The value is the number however it is written: `6.5E1` and `65.0` are 65.
This book does not specify a number written with more digits than
NUMERIC DIGITS and a nonzero fractional part.

```rexx
say c2x(d2c(' 65 ')) c2x(d2c('+65')) c2x(d2c('6.5E1')) c2x(d2c('65.0')),
    d2x('0.65E2')
signal on syntax name a
say d2c('x')
a: say condition('o')~code
signal on syntax name b
say d2x('')
b: say condition('o')~code
signal on syntax name c
say d2c('65.5')
c: say condition('o')~code
signal on syntax name d
say d2x('1234567890')
d: say condition('o')~code
numeric digits 3
signal on syntax name e
say d2c(1000)
e: say condition('o')~code
say d2x(999) d2x('1E2') d2x('-0')
```

```output
41 41 41 41 41
93.929
93.928
93.929
93.928
93.929
3E7 64 0
```

r[bif.index.conv.decimal.order]
The checks of a D2C or D2X call run in this order, and the first that fails
is raised: the form of `n` ([whole-number form](r:conv.whole.form), `SYNTAX
40.12`); whether the decimal argument is a number at all; the sign of `n`
(`SYNTAX 93.923`); and last whether the decimal argument is a whole number
with at most NUMERIC DIGITS digits. A decimal argument that is not a number
is therefore raised even when `n` is negative, and a number that is not
whole, or has too many digits, is raised only when `n` is not negative.

```rexx
signal on syntax name a
say d2c('x', 'y')
a: say condition('o')~code
signal on syntax name b
say d2c('x', -1)
b: say condition('o')~code
signal on syntax name c
say d2c('1234567890', -1)
c: say condition('o')~code
signal on syntax name d
say d2x('1.5', -1)
d: say condition('o')~code
signal on syntax name e
say d2x('x', -1)
e: say condition('o')~code
```

```output
40.12
93.929
93.923
93.923
93.928
```

r[bif.index.conv.digits-limit]
The answer of C2D and X2D is a whole number written in decimal digits, with
a leading `-` when it is negative, no leading zero, no period and no
exponent, and `0` for zero. If the answer has more digits than NUMERIC
DIGITS, the call raises `SYNTAX 93.936` in C2D and `SYNTAX 93.935` in X2D,
instead of rounding it; the sign is not a digit. An answer with exactly
NUMERIC DIGITS digits is allowed.

```rexx
say c2d('3b9ac9ff'x) c2d(copies('00'x, 20) || '01'x) x2d('0' || 'ff'),
    x2d('f6', 2)
signal on syntax name a
say c2d('3b9aca00'x)
a: say condition('o')~code
signal on syntax name b
say x2d('3b9aca00')
b: say condition('o')~code
numeric digits 3
say c2d('03e7'x) c2d('fc19'x, 2) x2d('3e7') x2d('fc19', 4)
signal on syntax name c
say c2d('03e8'x)
c: say condition('o')~code
signal on syntax name d
say c2d('fc18'x, 2)
d: say condition('o')~code
```

```output
999999999 1 255 -10
93.936
93.935
999 -999 999 -999
93.936
93.936
```

> **Note.** The ANSI draft (9.6.1 to 9.6.11) differs from this book in the
> codes of the argument errors. It raises `40.24` for a binary string and
> `40.25` for a hexadecimal one that is not valid, where this book has the
> 93.9xx codes above; `40.12` for a decimal argument
> that is not a whole number, where D2C and D2X raise 93.929 and 93.928; and
> `40.13` for a negative `n`, or a negative decimal argument without an `n`,
> where this book raises 93.923 and 93.927. It raises `40.35` for an answer
> that does not fit NUMERIC DIGITS, where C2D raises 93.936 and X2D 93.935.
> The draft also checks the hexadecimal string of X2D when `n` is 0, which
> raises an error there for a string with a bad character; this book
> answers 0 without examining the string.

r[bif.index.conv.deviation.4]
interrexx does not specify whether the answer of a conversion function is
the same object as another string of equal value, such as a literal, the
argument itself when the answer equals it, or the equal answer of another
call. Through `identityHash`, or as a key of an `IdentityTable` or
`IdentitySet`, the answer may be either, and a program can rely only on the
value, which is equal under `==`. ooRexx differs: it often answers a new
object where interrexx answers the existing one.

```rexx
s = '6' || '1'
say (b2x('10000') == '10') (c2x('a') == s) (x2c(s) == 'a')
say (d2c(97) == 'a') (c2d('') == 0) (c2d(s) == 13873)
```

```output
1 1 1
1 1 1
```

## Numbers

> **Note.** ABS, SIGN, TRUNC and FORMAT take a number as their first argument, which is
> the receiver when they run as String methods. The rules below say which
> strings are numbers, how a number is rounded to NUMERIC DIGITS, and how a
> result is written; the pages link to them instead of restating them.

r[bif.index.num.arg]
The number argument must be a string of the [whole-number form](r:conv.whole.form),
which is also the form of a Rexx number, but its value need not be whole. Any
other string raises `SYNTAX 93.943`, the null string and a string of only
whitespace among them. An argument that is an object other than a string is
first replaced by the answer of its `STRING` method, as
[any string argument](r:conv.string.object) is.

```rexx
say abs(' + 2.50E1 ') abs('.5') abs('5.') abs('-' || '09'x || ' 7')
say abs(.mutablebuffer~new('-7'))
signal on syntax name a
say abs('')
a: say condition('o')~code
signal on syntax name b
say abs('1 E1')
b: say condition('o')~code
signal on syntax name c
say sign('0x1')
c: say condition('o')~code
signal on syntax name d
say trunc('5' || '0a'x)
d: say condition('o')~code
signal on syntax name e
say format(.array~new)
e: say condition('o')~code
```

```output
25.0 0.5 5 7
7
93.943
93.943
93.943
93.943
93.943
```

r[bif.index.num.limit]
A nonzero number is a number only within the exponent range of the
language, which the next three conditions describe. Each failing number
raises `SYNTAX 93.943`: one whose written exponent, the digits after `E`
without the sign, has a value above 999999999; one whose first nonzero digit
stands at a power of ten above 999999999; and one whose last written digit
stands at a power of ten below -999999999. This book does not specify a
zero written with an exponent above 999999999.

```rexx
say abs('1e999999999') abs('12345e-999999999') abs('1e0000000001')
signal on syntax name a
say abs('1e1000000000')
a: say condition('o')~code
signal on syntax name b
say abs('12.3e999999999')
b: say condition('o')~code
signal on syntax name c
say abs('1.1e-999999999')
c: say condition('o')~code
signal on syntax name d
say abs('0.1e1000000000')
d: say condition('o')~code
```

```output
1E+999999999 1.2345E-999999995 10
93.943
93.943
93.943
93.943
```

r[bif.index.num.round]
Before it is used, the number is rounded to NUMERIC DIGITS significant
digits, as the sum `number + 0` would round it. Digits beyond the first
NUMERIC DIGITS significant ones are dropped, and the last kept digit is
raised by one when the first dropped digit is 5 or more, so the magnitude
rounds up and a negative number rounds away from zero. Leading zeros are not
significant, a number with fewer significant digits is not extended, and the
zeros it ends in after the point are kept. A number whose value is zero is
rounded to `0`, with no sign and no decimal places, however it was written.

```rexx
numeric digits 3
say abs('1.2349') abs('1.9996') abs('-0.0012350') abs(' 001.50 ')
say abs('-0.000') abs('0e5') abs('+0.00') abs('1234.5') abs('-125')
numeric digits 1
say abs('0.45') abs('0.55') abs('25') abs('-4.9')
```

```output
1.23 2.00 0.00124 1.50
0 0 0 1.23E+3 125
0.5 0.6 3E+1 5
```

r[bif.index.num.write]
A rounded number is written in plain or in scientific notation, depending on
a trigger value `T` that is NUMERIC DIGITS unless a page gives another. In
plain notation the digits are written in full, a point separates the integer
part from the decimal part when the number has decimal places, and a number
of magnitude below 1 starts `0.`. Let `W` be the count of digits before the
point in that notation, with a magnitude below 1 having `W` of 0 and zero
having 1, and `S` the count of digits after the point. A number of magnitude
1 or more is written plain when `W` is at most `T`; a number below 1 is
written plain when `S` is at most twice `T`; any other number is written in
[scientific notation](r:bif.index.num.sci). A negative number has a `-` before
its first character.

```rexx
say abs(123456789) abs(1234567890) abs('-0.000000000000000001'),
    abs('0.0000000000000000001')
numeric digits 3
say abs(123) abs(1234) abs('0.000001') abs('0.0000012') abs('0.000012')
say abs('-1234.5') abs('0.000000') abs('12.30')
```

```output
123456789 1.23456789E+9 0.000000000000000001 1E-19
123 1.23E+3 0.000001 1.2E-6 0.000012
1.23E+3 0 12.3
```

r[bif.index.num.sci]
Scientific notation writes the digits of the rounded number, the first
nonzero digit first, with a point after the first digit when there is a
second, then `E`, the sign of the exponent (`+` or `-`), and the exponent
as a number without leading zeros. The exponent is the power of ten of the
first digit. Under NUMERIC FORM SCIENTIFIC, which is the setting at the start
of a program, that is all. Under NUMERIC FORM ENGINEERING the exponent is
instead the multiple of 3 at or below the power of ten of the first digit, so
that one to three digits stand before the point, and the digits are
extended with zeros as far as the point needs.

```rexx
say abs('-1234567890') abs('1e11') abs('0.0000000123') abs('1e-19')
numeric form engineering
say abs('-1234567890') abs('2e11') abs('1.5e-19') abs('12345678e10')
```

```output
1.23456789E+9 1E+11 0.0000000123 1E-19
1.23456789E+9 200E+9 150E-21 123.45678E+15
```

> **Note.** The reference manual does not state when a number is written in scientific
> notation. The ANSI draft (9.4.2) decides it for FORMAT, and only there, from
> the count of digits before the point and a fixed count of zeros after it,
> where ooRexx, which this book follows, lets the limit on the zeros grow
> with the trigger.
