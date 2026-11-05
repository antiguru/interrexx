---
element: TRANSLATE
kind: builtin
syntax: "TRANSLATE(string [, tableo] [, tablei] [, pad] [, pos] [, length])"
ansi: "9.3.23"
rexxref: "Functions > Built-in Functions > TRANSLATE"
deviations: [4, 94]
---

# TRANSLATE

TRANSLATE replaces characters of a string, or of a run of its characters,
through a pair of tables: a character found in the input table `tablei` is
replaced by the character at the same position in the output table
`tableo`. Called with no table and no pad, it uppercases instead.

## Arguments

r[bif.translate.method]
TRANSLATE is a builtin that runs as a String method, and `string`, its
first argument, is the receiver; the other five become the message's
arguments, in their order. `string`, `tableo` and `tablei` are
[strings](r:conv.string.null), `pad` is a [pad](r:conv.pad), `pos` is a
[position](r:conv.position.check), and `length` is a
[length](r:conv.length.check). The arguments are checked in the
[order](r:conv.check.order) the conventions give, and the argument count by
the [builtin call rules](r:bif.call.count): one is required and six is the
most.

```rexx
say translate('tab', 'TB', 'tb') 'tab'~translate('TB', 'tb', , 2)
signal on syntax name a
say translate('abc', , , , 0)
a: say condition('o')~code
signal on syntax name b
say translate('abc', , , , 1, -3)
b: say condition('o')~code
signal on syntax name c
say translate('abc', 'x', 'a', '--')
c: say condition('o')~code
signal on syntax name d
say translate('abc', , , , 'one')
d: say condition('o')~code
signal on syntax name f
say translate('abc', , , , 1, 1, 1)
f: say condition('o')~code
signal on syntax name g
say translate(, 'x')
g: say condition('o')~code
```

```output
TaB taB
93.924
93.923
40.23
40.12
40.4
40.5
```

r[bif.translate.tablei.default]
When `tablei` is omitted, it is the string of all 256 characters in order,
from `'00'x` to `'FF'x`. Each character is then found in it at the
position one more than its byte value, `C2D(c) + 1`, so `tableo` is indexed
by byte value.

```rexx
say translate('0a0102'x, 'abc' || copies('-', 7) || 'k')
```

```output
kbc
```

r[bif.translate.tableo.default]
When `tableo` is omitted, it is the null string, so every character found
in `tablei` is replaced by `pad`, unless the
[uppercasing rule](r:bif.translate.upper) applies.

```rexx
say '['translate('abcabc', , 'ca')']' translate('abcabc', , 'ca', '#')
```

```output
[ b  b ] #b##b#
```

r[bif.translate.pad.default]
When `pad` is omitted, it is the blank, `'20'x`.

```rexx
say '['translate('pq', 'x', 'pq')']'
```

```output
[x ]
```

r[bif.translate.pos.default]
When `pos` is omitted, it is 1: the run of characters translated starts at
the first character of `string`.

```rexx
say translate('aaa', 'b', 'a', , , 2) translate('aaa', 'b', 'a', , 1, 2)
```

```output
bba bba
```

r[bif.translate.length.default]
When `length` is omitted, the run is every character of `string` at
position `pos` and after it, and no character when `pos` is greater than
`LENGTH(string)`.

```rexx
say translate('aaaa', 'b', 'a', , 3) translate('aaaa', 'b', 'a', , 5)
```

```output
aabb aaaa
```

## The result

r[bif.translate.upper]
When `tableo`, `tablei` and `pad` are all omitted, TRANSLATE returns
`string` with each character of the run from `pos` for `length` characters
[uppercased as UPPER does](r:bif.upper.case), and every other character
unchanged. The result is then `UPPER(string, pos, length)`, with the same
defaults for `pos` and `length`.

```rexx
say translate('hello, world') translate('hello, world', , , , 8, 3)
```

```output
HELLO, WORLD hello, WORld
```

r[bif.translate.result]
When at least one of `tableo`, `tablei` and `pad` is given, TRANSLATE
returns a string of `LENGTH(string)` characters. Each character `c` of
`string` at a position from `pos` to `pos + length - 1` is looked up in
`tablei`. When `c` occurs in it, let `k` be the position of its first
occurrence: the result has, in place of `c`, the character of `tableo` at
position `k` when `LENGTH(tableo)` is at least `k`, and `pad` otherwise.
When `c` does not occur in `tablei`, it is copied unchanged, and so is
every character of `string` outside the run.

```rexx
say translate('a1b2c3', 'xyz', 'abc') translate('a1b2c3', 'xyz', '123', , 3)
```

```output
x1y2z3 a1bycz
```

r[bif.translate.first]
When a character occurs more than once in `tablei`, its first occurrence
alone decides what it becomes; the characters of `tableo` at the positions
of the later occurrences are never used.

```rexx
say translate('mom', 'abc', 'mmo') translate('mom', '123', 'omo')
```

```output
aca 212
```

r[bif.translate.tableo.length]
`tableo` need not be as long as `tablei`. When it is shorter, a
character found in `tablei` at a position past the end of `tableo` is
replaced by `pad`; when it is longer, its characters past the length of
`tablei` are never used.

```rexx
say translate('wxyz', 'WX', 'wxyz', '+') translate('wxyz', 'WXYZ123', 'wx')
```

```output
WX++ WXyz
```

r[bif.translate.pad-only]
When both tables are omitted and `pad` is given, every character of the run
is replaced by `pad`, since every character is found in the
[default `tablei`](r:bif.translate.tablei.default) and the
[default `tableo`](r:bif.translate.tableo.default) has no characters.

```rexx
say translate('secret', , , '*') translate('secret', , , '*', 3, 2)
```

```output
****** se**et
```

The ANSI draft (9.3.23) uppercases whenever both tables are omitted and
ignores `pad` then. This page follows ooRexx, where the reference manual
says a given pad replaces the whole string.

r[bif.translate.tablei.empty]
When `tablei` is given as the null string, no character is found in it, and
the result is `string` unchanged, for every `tableo`, `pad`, `pos` and
`length` that pass their checks.

```rexx
say translate('abc', 'xyz', '') translate('abc', , '', '*') translate('abc', '', '')
```

```output
abc abc abc
```

This holds however the null string was produced, which the oracle does not
do: see [Deviation 94](r:bif.translate.deviation.94).

r[bif.translate.past-end]
When `pos` is greater than `LENGTH(string)`, no character is translated and
the result is `string`, for every `pos` past the end that passes its check.
When the run reaches past the end of `string`, it stops there, and nothing
is added.

```rexx
say translate('abc', , , '*', 4) translate('abc', , , , 999999999999999999),
    translate('abc', , , '*', 2, 50)
```

```output
abc abc a**
```

r[bif.translate.length.zero]
When `length` is 0, no character is translated and the result is `string`,
for every `pos` that passes its check. `pad` is still checked, as a
[pad](r:conv.pad) always is.

```rexx
say translate('abc', , , '*', 2, 0) translate('abc', , , , 1, 0)
signal on syntax name e
say translate('abc', , , '', 1, 0)
e: say condition('o')~code
```

```output
abc abc
40.23
```

r[bif.translate.empty]
When `string` is the null string, the result is the null string, for every
table, pad, `pos` and `length` that pass their checks.

```rexx
say '['translate('')']' '['translate('', 'a', 'b', '*', 3, 4)']'
```

```output
[] []
```

r[bif.translate.bytes]
TRANSLATE looks characters up and replaces them by
[character](r:conv.string.chars), one byte each: whitespace, control
characters and `'00'x` are found, replaced and used as replacements like
any other character, in `string`, in both tables and as `pad`.

```rexx
say c2x(translate('610009'x, '0920'x, '0009'x)) c2x(translate('ab', , 'b', '00'x))
```

```output
610920 6100
```

## Deviations

r[bif.translate.deviation.4]
interrexx does not specify whether the result of TRANSLATE is the same
object as another string of equal value: as `string` itself, when no
character of it changes, as in `translate(s, '', '')`, or as an equal
string produced some other way, such as the literal `'MNOP'` beside
`translate('mn' || 'op')`. Through `identityHash`, or as a key of an
`IdentityTable` or `IdentitySet`, the answer may be either, and a program
can rely only on the value, which is equal under `==`. The oracle differs:
it often answers a new object where interrexx answers the existing one.

```rexx
s = 'mn' || 'op'
say (translate(s, '', '') == s) (translate('mn' || 'op') == 'MNOP')
```

```output
1 1
```

r[bif.translate.deviation.94]
A `tablei` given as the null string is an [empty table](r:bif.translate.tablei.empty)
whether it was written as `''` or answered by an expression, such as
`LEFT(s, 0)`, `COPIES(s, 0)` or `STRIP` of a string of blanks. The oracle
differs: a null string that some of its builtins answer counts as given, so
it does not uppercase, but it is searched as if it were the
[default `tablei`](r:bif.translate.tablei.default), so every character of
the run is translated. There `translate('abc', , LEFT('abc', 0))` answers
three blanks. The same holds for the String method's `tablei`. A null
`tableo` produced the same way agrees on both.

```rexx
z = left('abc', 0)
say '['translate('abc', , z)']' translate('abc', , z, '*') 'ab'~translate(z, z)
```

```output
[abc] abc ab
```
