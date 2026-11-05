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
when the operands are equally long, and when both are null, and the null
string is not a pad, as [an empty argument](r:conv.arg.empty) is not
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
When `pad` is present, the shorter operand is first extended on the right
with copies of `pad` until it is as long as the longer one, and the two are
then combined character by character. The answer is as long as the longer
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
An omitted `string2` is the null string, and so is one that is the null
string. The answer is then `string1` itself when `pad` is omitted, and with
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
say (bitand('abcd', 'ef') == bitand('ef', 'abcd')) (bitor('abcd', 'ef', 'x') == bitor('ef', 'abcd', 'x'))
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
