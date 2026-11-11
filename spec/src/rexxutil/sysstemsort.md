---
element: "SysStemSort"
kind: function
syntax: "SysStemSort(stem, [order], [type], [start], [end], [firstcol], [lastcol])"
rexxref: "Rexx Utilities (RexxUtil) > SysStemSort"
---

# SysStemSort

SysStemSort puts the items of a stem array, or of a part of it, in order. The
order is that of the items as strings of bytes, read from a column of each
item if the call says so, and the call can ignore the case of letters and
reverse the order. It answers `0` when it has done so. A call that cannot be
done raises a `SYNTAX` condition and never answers another number. The stem
array is the one that [the chapter page](index.md) describes.

> **Note.** Most examples of this page put the items of a stem `s.` in a
> routine called `fill`, whose arguments are the items, and print them with a
> routine called `dump`, which writes each item in square brackets. Both are
> shown in the first example that uses them.

## Arguments

r[util.sysstemsort.args]
`stem` is required, and the six other arguments are optional. An omitted
argument takes its default: `order` is `A`, `type` is `C`, `start` is `1`,
`end` is the count of the stem, `firstcol` is `1` and `lastcol` is the end of
each item. `stem` names a stem as [the stem name rule](r:util.index.stemname)
says, with the addition of [the next rule](r:util.sysstemsort.name). The call
follows [the argument rule](r:util.index.args): a missing `stem` raises
`SYNTAX 88.901` and an eighth argument raises `SYNTAX 88.922`
([Deviation 101](r:util.index.deviation.101) is about which of the two comes
first). `start`, `end`, `firstcol` and `lastcol` are read as whole numbers
that are at least `1`, with their blanks, a leading plus sign, a decimal point
and an exponent as any whole number is. An argument that is `0`, negative,
fractional, text, the null string or `.nil` raises `SYNTAX 88.905`. `order` is
read by its first character, in either case: `A` is ascending and `D` is
descending, and the rest of the argument is not read, so `Descending` is `D`.
`type` is read the same way: `C` respects the case of letters and `I` ignores
it, and `Ignore` is `I`. Anything else in `order` or `type` raises
`SYNTAX 40.920`: the null string, a blank before the letter, another letter.
An `order` or `type` that is `.nil` raises `SYNTAX 88.909`, and a
MutableBuffer that holds `D` is `D`.

```rexx
call try "SysStemSort()"
call try "SysStemSort(,'D')"
call try "SysStemSort('s.', , , , , , , 1)"
call try "SysStemSort('s.', 'x')"
call try "SysStemSort('s.', '')"
call try "SysStemSort('s.', ' D')"
call try "SysStemSort('s.', .nil)"
call try "SysStemSort('s.', , 'x')"
call try "SysStemSort('s.', , '')"
call try "SysStemSort('s.', , .nil)"
call try "SysStemSort('s.', , , 0)"
call try "SysStemSort('s.', , , -1)"
call try "SysStemSort('s.', , , 1.5)"
call try "SysStemSort('s.', , , 'x')"
call try "SysStemSort('s.', , , '')"
call try "SysStemSort('s.', , , .nil)"
call try "SysStemSort('s.', , , , 0)"
call try "SysStemSort('s.', , , , , 0)"
call try "SysStemSort('s.', , , , , , 0)"
call try "SysStemSort('s.', , , 1, ' 3 ', +1, 1e1)"
call try "SysStemSort('s.', 'Descending', 'Ignore')"
call try "SysStemSort('s.', .mutablebuffer~new('d'))"
call try "SysStemSort('s.', , , , , , , )"
exit
try:
  parse arg ex
  s.0 = 3; s.1 = 'b'; s.2 = 'c'; s.3 = 'a'
  signal on syntax name e
  interpret 'r = ' ex
  say r s.1 s.2 s.3
  return
e: say condition('o')~code
  return
```

```output
88.901
88.901
88.922
40.920
40.920
40.920
88.909
40.920
40.920
88.909
88.905
88.905
88.905
88.905
88.905
88.905
88.905
88.905
88.905
0 a b c
0 c b a
0 c b a
0 a b c
```

r[util.sysstemsort.name]
A name that has characters after its first period names the stem array whose
count is the tail made of those characters and `.0`: the name `s.a` is the
array whose count is `s.a.0` and whose items are `s.a.1` and up, and `s.9.8`
is the one with the count `s.9.8.0`. The characters after the first period are
the tail as they are written, in upper case, and never the value of a
variable of that name. The part up to the first period is a name as
[the stem name rule](r:util.index.stemname) says. The other three functions
that take a stem name raise `SYNTAX 40.919` for such a name.

```rexx
s.9.0 = 2; s.9.1 = 'b'; s.9.2 = 'a'
say SysStemSort('s.9') s.9.1 s.9.2
s.9.8.0 = 2; s.9.8.1 = 'y'; s.9.8.2 = 'x'
say SysStemSort('S.9.8') s.9.8.1 s.9.8.2
say SysStemSort('s.9.8.') s.9.8.1 s.9.8.2
s.a.0 = 2; s.a.1 = 'q'; s.a.2 = 'p'
say SysStemSort('s.a') s.a.1 s.a.2
n = 9
signal on syntax name e
say SysStemSort('s.n')
e: say condition('o')~code
```

```output
0 a b
0 x y
0 x y
0 p q
40.922
```

## The order

r[util.sysstemsort.compare]
The items are compared as strings of bytes, from the first byte of each, so
that the order is that of the byte values. With `A` an item that compares
less comes first, and with `D` an item that compares more comes first. An
item that is the start of a longer one is less than it, and blanks count as
any other byte, so that `a` is less than `a ` and the null string is less than
a blank. The byte `00` is an ordinary byte too. The items are not read as
numbers: `10` is less than `9`, and `01` is less than `1`. Items that compare
as equal are put in no order that this book states.

```rexx
call fill 'b', 'a ', 'a', '', 'ab', '10', '9', 'B', '_', ' '
call SysStemSort 's.'
say dump()
call SysStemSort 's.', 'D'
say dump()
call fill 'a' || '00'x || 'b', 'a', '00'x, 'a' || '00'x
call SysStemSort 's.'
say c2x(s.1) c2x(s.2) c2x(s.3) c2x(s.4)
call fill '2', '10', '1.5', '01', '1e1'
call SysStemSort 's.'
say dump()
exit
fill: procedure expose s.
  do i = 1 to arg(); s.i = arg(i); end
  s.0 = arg()
  return
dump: procedure expose s.
  out = ''
  do i = 1 to s.0; out = out '[' || s.i || ']'; end
  return strip(out)
```

```output
[] [ ] [10] [9] [B] [_] [a] [a ] [ab] [b]
[b] [ab] [a ] [a] [_] [B] [9] [10] [ ] []
00 61 6100 610062
[01] [1.5] [10] [1e1] [2]
```

r[util.sysstemsort.case]
With `C`, the bytes are compared as they are, so that every upper case letter
comes before every lower case one, and the bytes between `Z` and `a`, such as
`[` and `_`, come between them. With `I`, the letters `a` to `z` are compared
as the letters `A` to `Z`, so that an item and the same item in another case
are equal and come in no order that this book states, and the bytes between
`Z` and `a` come after every letter. With `I`, the bytes `80` to `FF` are less
than the bytes `00` to `7F`, in the order of their values among themselves,
whereas with `C` they are greater. With `D` the order of both is reversed.

```rexx
call fill 'Zed', 'apple', 'Mango', '_x', '[y'
call SysStemSort 's.'
say dump()
call SysStemSort 's.', , 'I'
say dump()
call SysStemSort 's.', 'D', 'I'
say dump()
call fill '7f'x, '80'x, 'ff'x, 'a', 'e9'x
call SysStemSort 's.'
say c2x(s.1) c2x(s.2) c2x(s.3) c2x(s.4) c2x(s.5)
call SysStemSort 's.', , 'I'
say c2x(s.1) c2x(s.2) c2x(s.3) c2x(s.4) c2x(s.5)
call SysStemSort 's.', 'D', 'I'
say c2x(s.1) c2x(s.2) c2x(s.3) c2x(s.4) c2x(s.5)
exit
fill: procedure expose s.
  do i = 1 to arg(); s.i = arg(i); end
  s.0 = arg()
  return
dump: procedure expose s.
  out = ''
  do i = 1 to s.0; out = out '[' || s.i || ']'; end
  return strip(out)
```

```output
[Mango] [Zed] [[y] [_x] [apple]
[apple] [Mango] [Zed] [[y] [_x]
[_x] [[y] [Zed] [Mango] [apple]
61 7F 80 E9 FF
80 E9 FF 61 7F
7F 61 FF E9 80
```

## Columns

r[util.sysstemsort.cols]
`firstcol` and `lastcol` choose the bytes of each item that are compared: the
bytes from the column `firstcol` to the column `lastcol` of the item, the
first byte being column `1`. An item that has fewer bytes than the column
`firstcol` compares as the null string, so that it is less than any item that
has a byte there; `lastcol` may be past the end of an item, and then the
comparison reaches the end of the item. The columns change only what is
compared: the whole items are what the call puts in order.

```rexx
call fill 'b2', 'a9', 'c1'
call SysStemSort 's.', , , , , 2
say dump()
call SysStemSort 's.', 'D', , , , 2
say dump()
call fill 'xb2', 'xa9', 'xc1'
call SysStemSort 's.', , , , , 2, 2
say dump()
call fill 'xb9', 'xa8', 'xc7'
call SysStemSort 's.', , , , , 3, 99
say dump()
call fill 'b', 'abc', 'ab'
call SysStemSort 's.', , , , , 2
say dump()
call SysStemSort 's.', 'D', , , , 2, 3
say dump()
call fill 'XB', 'xa', 'Xc'
call SysStemSort 's.', , 'I', , , 2
say dump()
call SysStemSort 's.', , 'C', , , 2
say dump()
exit
fill: procedure expose s.
  do i = 1 to arg(); s.i = arg(i); end
  s.0 = arg()
  return
dump: procedure expose s.
  out = ''
  do i = 1 to s.0; out = out '[' || s.i || ']'; end
  return strip(out)
```

```output
[c1] [b2] [a9]
[a9] [b2] [c1]
[xa9] [xb2] [xc1]
[xc7] [xa8] [xb9]
[b] [ab] [abc]
[abc] [ab] [b]
[xa] [XB] [Xc]
[XB] [xa] [Xc]
```

## Which items

r[util.sysstemsort.range]
`start` and `end` choose the items that are put in order, the items from
`start` to `end`; the items outside them keep their places and their values.
An item that is alone in the range, as with `start` equal to `end`, is still
read. The call raises `SYNTAX 40.923`, and changes nothing, when `start` or
`end` is more than the count of the stem. A stem with the count `0` has no
item, and then no range is checked: the call answers `0` and does nothing, for
every `start` and `end` that are allowed. The tail `0` is never changed, and
the tails above the count are not touched.

```rexx
call fill 'f', 'e', 'd', 'c', 'b', 'a'
call SysStemSort 's.', , , 2, 4
say dump()
call SysStemSort 's.', , , 5
say dump()
call SysStemSort 's.', , , , 2
say dump()
call SysStemSort 's.', , , 3, 3
say dump()
call try "SysStemSort('s.', , , 7)"
call try "SysStemSort('s.', , , 1, 7)"
call try "SysStemSort('s.', , , 7, 7)"
s.0 = 0
call try "SysStemSort('s.', , , 2)"
call try "SysStemSort('s.', , , 2, 9)"
s.0 = 3; s.1 = 'c'; s.2 = 'b'; s.3 = 'a'; s.4 = 'beyond'
call SysStemSort 's.'
say dump() s.4
exit
fill: procedure expose s.
  do i = 1 to arg(); s.i = arg(i); end
  s.0 = arg()
  return
dump: procedure expose s.
  out = ''
  do i = 1 to s.0; out = out '[' || s.i || ']'; end
  return strip(out)
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
[f] [c] [d] [e] [b] [a]
[f] [c] [d] [e] [a] [b]
[c] [f] [d] [e] [a] [b]
[c] [f] [d] [e] [a] [b]
40.923
40.923
40.923
0
0
[a] [b] [c] beyond
```

r[util.sysstemsort.items]
An item that is not a string takes part by its string value, found
[as the conventions say](r:conv.string.object), and after the call the items
in the range are strings: each is replaced by that string value, whether it
moved or not. The items outside the range keep their objects. A call that
raises `SYNTAX 40.924` has changed no item, not even to a string, so that it
leaves every object in its place. The call raises it when an item of the
range has no value, and this holds for a range of one item as well.

```rexx
s.0 = 4; s.1 = 'd'; s.2 = .k~new('c'); s.3 = 'b'; s.4 = .k~new('a')
call SysStemSort 's.', , , 2, 3
say s.1 s.2~class~id s.3~class~id s.4~class~id s.2
call SysStemSort 's.'
say s.1~class~id s.2~class~id s.3~class~id s.4~class~id s.1 s.2 s.3 s.4
s.0 = 2; s.1 = .k~new('a'); s.2 = 'b'
call SysStemSort 's.'
say s.1~class~id s.1
s.0 = 3; s.1 = .k~new('c'); s.2 = 'a'; drop s.3
signal on syntax name e
call SysStemSort 's.'
e: say condition('o')~code s.1~class~id s.2 symbol('s.3')
signal off syntax
s.3 = 'b'; drop s.2
signal on syntax name f
call SysStemSort 's.', , , 2, 2
f: say condition('o')~code
::class k
::method init
  expose v
  use arg v
::method makestring
  expose v
  return v
```

```output
d String String K b
String String String String a b c d
String a
40.924 K a LIT
40.924
```

## Order of the checks

r[util.sysstemsort.order]
When several things are wrong, the call raises the condition of the first of
these checks that fails: an `order` or a `type` that is `.nil` raises
`SYNTAX 88.909`; then `start`, `end`, `firstcol` and `lastcol` raise
`SYNTAX 88.905`; then the name of the stem raises `SYNTAX 40.919`; then
`order` and `type` raise `SYNTAX 40.920`; then a `start` that is more than an
`end` that is given, and a `firstcol` that is more than a `lastcol` that is
given, raise `SYNTAX 40.921`; then the tail `0` of the stem raises
`SYNTAX 40.922`; then the range raises `SYNTAX 40.923`; and last an item
without a value raises `SYNTAX 40.924`. A `start` that is more than the
default `end`, the count of the stem, is a range that is outside the stem and
raises `SYNTAX 40.923`, not `SYNTAX 40.921`. The other three functions that
take a stem name check the name before any number.

```rexx
call try "SysStemSort('s-.', .nil, , 'x')", ""
call try "SysStemSort('s-.', 'x', , 'x')", ""
call try "SysStemSort('s-.', 'x')", ""
call try "SysStemSort('s.', 'x', , 4, 2)", ""
call try "SysStemSort('s.', , , 4, 2)", "drop s.0"
call try "SysStemSort('s.', , , , , 3, 2)", "drop s.0"
call try "SysStemSort('s.', , , 9, 8)", ""
call try "SysStemSort('s.', , , 7)", ""
call try "SysStemSort('s.')", "drop s.0"
call try "SysStemSort('s.', , , , 9)", "drop s.3"
call try "SysStemSort('s.')", "drop s.3"
call try "SysStemInsert('s-.', 'x', 'v')", ""
exit
try:
  parse arg ex, setup
  s.0 = 6; s.1 = 'f'; s.2 = 'e'; s.3 = 'd'; s.4 = 'c'; s.5 = 'b'; s.6 = 'a'
  interpret setup
  signal on syntax name e
  interpret 'r = ' ex
  say r
  return
e: say condition('o')~code
  return
```

```output
88.909
88.905
40.919
40.920
40.921
40.921
40.921
40.923
40.922
40.923
40.924
40.919
```
