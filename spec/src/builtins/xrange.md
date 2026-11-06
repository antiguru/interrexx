---
element: XRANGE
kind: builtin
syntax: "XRANGE([start] [, end] ...)"
ansi: "9.3.30"
rexxref: "Functions > Built-in Functions > XRANGE (Hexadecimal Range)"
deviations: [4]
---

# XRANGE

XRANGE builds a string out of characters picked by their codes: every
character from a start character to an end character, or every character of
a named class, such as the digits. Any number of ranges and class names can
be given, and the answer joins what each of them selects.

## Arguments

r[bif.xrange.method]
XRANGE takes any number of arguments, none included, and every argument may
be omitted. It does not run as a String method: a string has no message
named `XRANGE`, and sending it raises `SYNTAX 97.1`. Each argument is one
character, an omitted place or the name of a class, as the rules below read
them; an argument that is not a string is first converted
[as any string argument is](r:bif.xrange.object).

```rexx
say length(xrange()) length(xrange('a')) length(xrange('a', 'c', 'digit', 'x'))
signal on syntax name e
say 'a'~xrange
e: say condition('o')~code
```

```output
256 159 149
97.1
```

r[bif.xrange.groups]
XRANGE reads its arguments from left to right as a series of groups, and
the answer is what the groups select, joined in that order. A group starts
at an argument that is [omitted](r:conv.arg.omitted) or one character long:
that argument is the group's `start` and the argument after it is its
`end`, so the group takes two arguments and selects a
[range](r:bif.xrange.range). A group starts at any other argument, the null
string among them, and takes that argument alone: the argument names a
[class](r:bif.xrange.class). The argument after a group starts the next
group. The arguments read are the first `count` of them, where `count` is the
[argument count](r:bif.index.call.count) of the call: the omitted places
after the last argument that is not omitted are not read and start no group.

```rexx
say xrange('a', 'c', 'x', 'z') xrange('a', 'b', 'digit', 'c', 'd')
say xrange('digit', 'upper') length(xrange('digit', 'a')) xrange('digit', 'a', 'a')
say length(xrange('a', , , )) length(xrange('a', , , 'b'))
```

```output
abcxyz ab0123456789cd
0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ 169 0123456789a
159 258
```

r[bif.xrange.default]
A call whose [argument count](r:bif.index.call.count) is 0, such as
`xrange()` or `xrange(, )`, reads no group by
[the groups rule](r:bif.xrange.groups) and selects every character instead:
the range from `'00'x` to `'FF'x`, 256 characters in ascending order of
code. A group whose start and end are both omitted selects the same 256
characters, but only when an argument that is not omitted comes after it;
omitted places after the last such argument start no group, so they add
nothing to a call that has one.

```rexx
say c2x(left(xrange(), 4)) length(xrange()) length(xrange(, ))
say length(xrange('digit', , )) length(xrange('digit', , , )) length(xrange('digit', , , 'a'))
```

```output
00010203 256 256
10 10 425
```

## Ranges

r[bif.xrange.range]
When the code of `start` is not above the code of `end`, a range selects
the character with the code of `start`, the one with the code of `end`, and
every character whose code lies between them, from the lowest code up. A
start equal to the end selects that one character. An omitted `start` is
`'00'x`, and an omitted `end`, or an end the call has no argument for, is
`'FF'x`. A `start` and an `end` are single characters, as
[the groups rule](r:bif.xrange.groups) reads them.

```rexx
say xrange('m', 'q') xrange('k', 'k')
say c2x(xrange(, '03'x)) c2x(xrange('FD'x)) c2x(xrange('FD'x, ))
```

```output
mnopq k
00010203 FDFEFF FDFEFF
```

r[bif.xrange.wrap]
When the code of `start` is above the code of `end`, the range runs
around the end of the code space: it selects the characters from `start` to
`'FF'x` and then those from `'00'x` to `end`, in that order.

```rexx
say c2x(xrange('FD'x, '01'x)) c2x(xrange('ff'x, '00'x)) length(xrange('b', 'a'))
```

```output
FDFEFF0001 FF00 256
```

r[bif.xrange.end]
An `end` that is present must be one character long: an end that is the
null string, or longer than one character, a class name among them, raises
`SYNTAX 40.23`. A character of any code can be an end, and a start.

```rexx
signal on syntax name a
say xrange('a', '')
a: say condition('o')~code
signal on syntax name b
say xrange('a', 'digit')
b: say condition('o')~code
signal on syntax name c
say xrange('a', 'zz')
c: say condition('o')~code
say c2x(xrange('00'x, '00'x)) c2x(xrange('ff'x, 'ff'x))
```

```output
40.23
40.23
40.23
00 FF
```

## Classes

r[bif.xrange.class]
An argument that names a class selects the characters of that class in
ascending order of code. The twelve names are `ALNUM`, `ALPHA`, `BLANK`,
`CNTRL`, `DIGIT`, `GRAPH`, `LOWER`, `PRINT`, `PUNCT`, `SPACE`, `UPPER` and
`XDIGIT`, in any mix of upper and lower case. The whole argument must be the
name: an abbreviation, a name with a blank or any other character before or
after it, a plural, and the null string name no class, and an argument that
names no class and is not one character long raises `SYNTAX 40.28`.

```rexx
say xrange('Digit') xrange('xDIGIT')
signal on syntax name a
say xrange('digits')
a: say condition('o')~code
signal on syntax name b
say xrange('dig')
b: say condition('o')~code
signal on syntax name c
say xrange('digit ')
c: say condition('o')~code
signal on syntax name d
say xrange('')
d: say condition('o')~code
signal on syntax name e
say xrange('ab', 'c')
e: say condition('o')~code
```

```output
0123456789 0123456789ABCDEFabcdef
40.28
40.28
40.28
40.28
40.28
```

r[bif.xrange.class.letters]
The classes of letters and digits are these sets of characters, each in
ascending order of code: `DIGIT` is `0` to `9`; `UPPER` is `A` to `Z`;
`LOWER` is `a` to `z`; `ALPHA` is `UPPER` followed by `LOWER`; `ALNUM` is
`DIGIT`, `UPPER` and `LOWER` in that order; and `XDIGIT` is `0` to `9`, then
`A` to `F`, then `a` to `f`. No other character belongs to them.

```rexx
say (xrange('digit') == xrange('0', '9')) (xrange('upper') == xrange('A', 'Z')),
    (xrange('lower') == xrange('a', 'z'))
say (xrange('alpha') == xrange('A', 'Z') || xrange('a', 'z')),
    (xrange('alnum') == xrange('0', '9') || xrange('A', 'Z') || xrange('a', 'z'))
say (xrange('xdigit') == xrange('0', '9') || xrange('A', 'F') || xrange('a', 'f'))
```

```output
1 1 1
1 1
1
```

r[bif.xrange.class.space]
`BLANK` is the tab, `'09'x`, and the blank, `'20'x`, in that order. `SPACE`
is the characters `'09'x` to `'0D'x` and then the blank. `CNTRL` is the
characters `'00'x` to `'1F'x` and then `'7F'x`. `SPACE` is not the
[whitespace](r:conv.string.whitespace) of this book's other rules: it holds
`'0A'x` to `'0D'x`, which they do not.

```rexx
say c2x(xrange('blank')) c2x(xrange('space')) length(xrange('cntrl'))
say (xrange('cntrl') == xrange('00'x, '1f'x) || '7f'x)
```

```output
0920 090A0B0C0D20 33
1
```

r[bif.xrange.class.symbol]
`GRAPH` is the characters `'21'x` to `'7E'x`, the visible characters of
ASCII. `PRINT` is the same with the blank in front: `'20'x` to `'7E'x`.
`PUNCT` is the characters of `GRAPH` that are not in `ALNUM`: `'21'x` to
`'2F'x`, `'3A'x` to `'40'x`, `'5B'x` to `'60'x` and `'7B'x` to `'7E'x`.

```rexx
say (xrange('graph') == xrange('21'x, '7e'x)) (xrange('print') == xrange('20'x, '7e'x))
say xrange('punct')
say (xrange('punct') == xrange('21'x, '2f'x) || xrange(':', '@'),
    || xrange('[', '`') || xrange('{', '~'))
```

```output
1 1
!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~
1
```

## The result

r[bif.xrange.join]
The answer is the selections of the groups joined in the order of the
groups, with nothing between them. A character that two groups both select
appears once for each of them, so the answer can be longer than 256
characters, and a class given twice is selected twice.

```rexx
say xrange('digit', 'digit') length(xrange('a', 'c', 'b', 'd', 'digit', 'digit'))
say length(xrange(, , , )) length(xrange(, , 'a'))
```

```output
01234567890123456789 26
256 415
```

r[bif.xrange.errors]
The groups are read from left to right and the first group that has an
error decides: an error in a later argument is raised even when the groups
before it are valid, and no argument after the failing one is read.

```rexx
signal on syntax name a
say xrange('a', 'b', 'zz')
a: say condition('o')~code
signal on syntax name b
say xrange('a', '', 'zz')
b: say condition('o')~code
signal on syntax name c
say xrange('zz', 'a', '')
c: say condition('o')~code
signal on syntax name d
say xrange('digit', 'a', 'b', 'q', 'r', 'rs')
d: say condition('o')~code
```

```output
40.28
40.23
40.28
40.28
```

r[bif.xrange.object]
An argument that is not a string is replaced by the answer of its `STRING`
method before it is read, [as for any string argument](r:conv.string.object),
so a number is read by its characters and an object by the string it
answers.

```rexx
say xrange(1, 3) xrange(.mutablebuffer~new('c'), .mutablebuffer~new('e'))
signal on syntax name a
say xrange(.object~new)
a: say condition('o')~code
```

```output
123 cde
40.28
```

## Deviations

r[bif.xrange.deviation.4]
interrexx does not specify whether the answer of XRANGE is the same object
as another string of equal value, such as the literal `'a'` beside the answer
of `xrange('a', 'a')`, or the equal answer of another call. Through
`identityHash`, or as a key of an `IdentityTable` or `IdentitySet`, the
answer may be either, and a program can rely only on the value, which is
equal under `==`. ooRexx differs: it often answers a new object where
interrexx answers the existing one.

```rexx
say (xrange('a', 'a') == 'a') (xrange('a', 'b') == 'ab') (xrange('digit') == xrange('0', '9'))
```

```output
1 1 1
```

> **Note.** The ANSI draft (9.3.30) takes two arguments, `start` and `end`,
> checks each present one as a pad and raises `SYNTAX 40.23` when it is not
> one character long, the null string included; an omitted one is the lowest
> or highest character. It has no class names and no further groups; those
> are ooRexx's, and so is `SYNTAX 40.28` for an argument that starts a group
> and is neither one character long nor a class name.
