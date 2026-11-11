---
element: "SysStemCopy"
kind: function
syntax: "SysStemCopy(fromstem, tostem, [from], [to], [count], [option])"
rexxref: "Rexx Utilities (RexxUtil) > SysStemCopy"
---

# SysStemCopy

SysStemCopy copies items of one stem array to another stem array, either over
the items that are there or between them. It answers `0` when it has done so.
A call that cannot be done raises a `SYNTAX` condition and never answers
another number. The stem arrays are the ones that
[the chapter page](index.md) describes: the tail `0` holds the number of items
and the tails `1` and up hold the items.

> **Note.** Most examples of this page run in a routine that sets the source
> to the four items `a b c d` and the target to the two items `x y` before
> every call, and that prints the answer or the error code, and then the
> target as its count, a colon and its items.

## Arguments

r[util.sysstemcopy.args]
`fromstem` and `tostem` are required. They name stems as
[the stem name rule](r:util.index.stemname) says, and may name the same stem.
`from`, `to`, `count` and `option` are optional, and an omitted one takes its
default: `from` and `to` are `1`, `count` is the number of items of the source
from `from` to its end, and `option` is `O`. The call follows
[the argument rule](r:util.index.args): a missing required argument raises
`SYNTAX 88.901` and an argument past the sixth raises `SYNTAX 88.922`
([Deviation 101](r:util.index.deviation.101) is about which of the two
comes first). `from` and `to` are read as whole numbers that are at least `1`,
and an argument that is `0`, negative, fractional, text, the null string or
`.nil` raises `SYNTAX 88.905`. `count` is read as a whole number that is at
least `0`, and an argument that is negative, fractional, text, the null string
or `.nil` raises `SYNTAX 88.904`. A number is read with its blanks, a leading
plus sign, a decimal point and an exponent as any whole number is, so that
`' 2 '` and `1e0` are numbers. `option` is read by its first character, in
either case: `I` inserts the items, `O` overwrites the items, and the rest of
the argument is not read, so `Insert` is `I`. Anything else raises
`SYNTAX 40.920`: the null string, a blank before the letter, another letter.
An `option` that is `.nil` raises `SYNTAX 88.909`, and a MutableBuffer that
holds `I` is `I`.

```rexx
call try "SysStemCopy()"
call try "SysStemCopy('s.')"
call try "SysStemCopy('s.', 't.', 1, 1, 1, 'I', 1)"
call try "SysStemCopy('s.', 't.', 0)"
call try "SysStemCopy('s.', 't.', 1, 0)"
call try "SysStemCopy('s.', 't.', 1, 1, -1)"
call try "SysStemCopy('s.', 't.', 1.5)"
call try "SysStemCopy('s.', 't.', 'x')"
call try "SysStemCopy('s.', 't.', '')"
call try "SysStemCopy('s.', 't.', .nil)"
call try "SysStemCopy('s.', 't.', 1, 1, '')"
call try "SysStemCopy('s.', 't.', 1, 1, .nil)"
call try "SysStemCopy('s.', 't.', 1, 1, 1, 'x')"
call try "SysStemCopy('s.', 't.', 1, 1, 1, '')"
call try "SysStemCopy('s.', 't.', 1, 1, 1, ' I')"
call try "SysStemCopy('s.', 't.', 1, 1, 1, .nil)"
call try "SysStemCopy('s.', 't.', ' 2 ', 1e0, +1)"
call try "SysStemCopy('s.', 't.', 1, 1, 1, 'insert')"
call try "SysStemCopy('s.', 't.', 1, 1, 1, .mutablebuffer~new('i'))"
call try "SysStemCopy('s.', 't.', 1, 1, 1, 'overwrite')"
exit
try:
  parse arg ex
  call reset
  signal on syntax name e
  interpret 'r = ' ex
  say r show()
  return
e: say condition('o')~code show()
  return
reset:
  s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
  drop t.
  t.0 = 2; t.1 = 'x'; t.2 = 'y'
  return
show: procedure expose t.
  out = t.0 ':'
  do i = 1 to t.0; out = out t.i; end
  return out
```

```output
88.901 2 : x y
88.901 2 : x y
88.922 2 : x y
88.905 2 : x y
88.905 2 : x y
88.904 2 : x y
88.905 2 : x y
88.905 2 : x y
88.905 2 : x y
88.905 2 : x y
88.904 2 : x y
88.904 2 : x y
40.920 2 : x y
40.920 2 : x y
40.920 2 : x y
88.909 2 : x y
0 2 : b y
0 3 : a x y
0 3 : a x y
0 2 : a y
```

## What is copied

r[util.sysstemcopy.copy]
With `O`, the call copies the items `from` to `from + count - 1` of the
source, one at a time and in rising order, to the tails `to` and up of the
target, over whatever those tails hold. When the last tail written is above
the count of the target, the tail `0` of the target is set to its number, and
otherwise the tail `0` is left as it is. A target that has no tail `0` counts
as `0`, so that it is made by the call, and a call that copies no item does
not change the target at all, not even to give it a tail `0`. The items of the
target below `to`, above the last tail written, and past its count are not
touched. An item is copied as it is, so that an object that is an item of the
source is then the item of the target too. The source is never changed, except
when the target is the same stem.

```rexx
call try "SysStemCopy('s.', 't.')"
call try "SysStemCopy('s.', 't.', 2)"
call try "SysStemCopy('s.', 't.', 2, 2)"
call try "SysStemCopy('s.', 't.', 2, 3)"
call try "SysStemCopy('s.', 't.', 2, 1, 1)"
call try "SysStemCopy('s.', 't.', 3, 3, 2)"
call try "SysStemCopy('s.', 't.', 1, 1, 0)"
call try "SysStemCopy('s.', 't.', 5)"
call try "SysStemCopy('s.', 't.', 1, 1, 1, 'O')"
t.0 = 2; t.1 = 'x'; t.2 = 'y'; t.3 = 'kept'
call SysStemCopy 's.', 't.', 1, 1, 1
say t.0 t.1 t.2 t.3
drop u.
call SysStemCopy 's.', 'u.', 2, 1, 2
say u.0 u.1 u.2
drop u.
call SysStemCopy 's.', 'u.', 1, 1, 0
say symbol('u.0') u.0
s.2 = .array~of(7)
call SysStemCopy 's.', 'u.', 2, 1, 1
say u.0 (u.1 == s.2) u.1~class~id
exit
try:
  parse arg ex
  call reset
  signal on syntax name e
  interpret 'r = ' ex
  say r show()
  return
e: say condition('o')~code show()
  return
reset:
  s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
  drop t.
  t.0 = 2; t.1 = 'x'; t.2 = 'y'
  return
show: procedure expose t.
  out = t.0 ':'
  do i = 1 to t.0; out = out t.i; end
  return out
```

```output
0 4 : a b c d
0 3 : b c d
0 4 : x b c d
0 5 : x y b c d
0 2 : b y
0 4 : x y c d
0 2 : x y
0 2 : x y
0 2 : a y
2 a y kept
2 b c
LIT U.0
1 1 Array
```

r[util.sysstemcopy.range]
The call raises `SYNTAX 40.923`, and changes nothing, when the items it
would read are not all in the source, that is when `from + count - 1` is
more than the count of the source, and when `to` is more than one past the
count of the target: a target of two items can take `to` up to `3`. `from`
may be one past the count of the source when no item is read, and a `count`
that is left out then is `0`; any other `from` past that point with a `count`
given raises the condition too.

```rexx
call try "SysStemCopy('s.', 't.', 1, 1, 5)"
call try "SysStemCopy('s.', 't.', 3, 1, 3)"
call try "SysStemCopy('s.', 't.', 4, 1, 1)"
call try "SysStemCopy('s.', 't.', 5, 1, 0)"
call try "SysStemCopy('s.', 't.', 5)"
call try "SysStemCopy('s.', 't.', 6, 1, 0)"
call try "SysStemCopy('s.', 't.', 1, 3)"
call try "SysStemCopy('s.', 't.', 1, 4)"
call try "SysStemCopy('s.', 't.', 1, 3, 0)"
call try "SysStemCopy('s.', 't.', 1, 4, 0)"
call try "SysStemCopy('s.', 't.', 1, 1, 99999999999)"
exit
try:
  parse arg ex
  call reset
  signal on syntax name e
  interpret 'r = ' ex
  say r show()
  return
e: say condition('o')~code show()
  return
reset:
  s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
  drop t.
  t.0 = 2; t.1 = 'x'; t.2 = 'y'
  return
show: procedure expose t.
  out = t.0 ':'
  do i = 1 to t.0; out = out t.i; end
  return out
```

```output
40.923 2 : x y
40.923 2 : x y
0 2 : d y
0 2 : x y
0 2 : x y
40.923 2 : x y
0 6 : x y a b c d
40.923 2 : x y
0 2 : x y
40.923 2 : x y
40.923 2 : x y
```

## Inserting

r[util.sysstemcopy.insert]
With `I`, the call first moves the items of the target from `to` to its count
up by `count` places, and then copies as `O` does. So the copied items stand
at the tails `to` to `to + count - 1`, the items that were there follow them,
and the count of the target is its old count plus `count`, written as a plain
whole number. `to` may be one past the count of the target, which appends the
items. With `I`, a target that has no tail `0` is given one, even when
`count` is `0`.

```rexx
call try "SysStemCopy('s.', 't.', 2, 2, 2, 'I')"
call try "SysStemCopy('s.', 't.', 1, 1, 1, 'I')"
call try "SysStemCopy('s.', 't.', 1, 3, 3, 'I')"
call try "SysStemCopy('s.', 't.', 1, 1, , 'I')"
call try "SysStemCopy('s.', 't.', 3, 2, , 'I')"
call try "SysStemCopy('s.', 't.', 1, 2, 0, 'I')"
call try "SysStemCopy('s.', 't.', 1, 4, 2, 'I')"
t.0 = '02'
call SysStemCopy 's.', 't.', 1, 1, 1, 'I'
say t.0
drop u.
call SysStemCopy 's.', 'u.', 1, 1, 0, 'I'
say symbol('u.0') u.0
exit
try:
  parse arg ex
  call reset
  signal on syntax name e
  interpret 'r = ' ex
  say r show()
  return
e: say condition('o')~code show()
  return
reset:
  s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
  drop t.
  t.0 = 2; t.1 = 'x'; t.2 = 'y'
  return
show: procedure expose t.
  out = t.0 ':'
  do i = 1 to t.0; out = out t.i; end
  return out
```

```output
0 4 : x b c y
0 3 : a x y
0 5 : x y a b c
0 6 : a b c d x y
0 4 : x c d y
0 2 : x y
40.923 2 : x y
3
VAR 0
```

## One stem for both

r[util.sysstemcopy.same]
When `fromstem` and `tostem` name the same stem, the call copies the items
one at a time in rising order and reads each item at the moment it copies it,
so that an item written earlier in the call can be read again later. A copy
over the items that it reads therefore repeats the first of them, and a copy
to a place that does not overlap what is read is a plain copy.

```rexx
call try "SysStemCopy('s.', 's.', 1, 2, 2)"
call try "SysStemCopy('s.', 's.', 2, 1, 2)"
call try "SysStemCopy('s.', 's.', 1, 2)"
call try "SysStemCopy('s.', 's.', 1, 5, 2)"
call try "SysStemCopy('s.', 's.', 1, 3, 2, 'I')"
exit
try:
  parse arg ex
  s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
  signal on syntax name e
  interpret 'r = ' ex
  say r show()
  return
e: say condition('o')~code show()
  return
show: procedure expose s.
  out = s.0 ':'
  do i = 1 to s.0; out = out s.i; end
  return out
```

```output
0 4 : a a a d
0 4 : b c c d
0 5 : a a a a a
0 6 : a b c d a b
0 6 : a b a b c d
```

## Order of the checks

r[util.sysstemcopy.order]
When several things are wrong, the call raises the condition of the first of
these checks that fails. The names of the stems raise `SYNTAX 40.919`; then
`from`, `to` and `count`, in that order, raise `SYNTAX 88.905` or
`SYNTAX 88.904`; then `option` raises `SYNTAX 40.920`; then the tail `0` of
the source raises `SYNTAX 40.922`; then the items to be read raise
`SYNTAX 40.923` when they are not all in the source; then the tail `0` of the
target raises `SYNTAX 40.922`; then `to` raises `SYNTAX 40.923` when it is
more than one past the count of the target; and last the items themselves
raise `SYNTAX 40.924` when one has no value, as they are copied.

```rexx
call try "SysStemCopy('s.a', 't.a', 'x')", ""
call try "SysStemCopy('s.', 't.', 0, 0, -1)", ""
call try "SysStemCopy('s.', 't.', 1, 'x', 'x')", ""
call try "SysStemCopy('s.', 't.', 1, 1, -1, 'x')", ""
call try "SysStemCopy('s.', 't.', 1, 1, 1, 'x')", "drop s.0"
call try "SysStemCopy('s.', 't.', 1, 1, 9)", "drop s.0"
call try "SysStemCopy('s.', 't.', 1, 1, 9)", "t.0 = 'x'"
call try "SysStemCopy('s.', 't.', 1, 9, 1)", "t.0 = 'x'"
call try "SysStemCopy('s.', 't.', 1, 9, 4)", "drop s.3"
call try "SysStemCopy('s.', 't.', 1, 1, 4)", "drop s.3; t.0 = 'x'"
call try "SysStemCopy('s.', 't.', 1, 1, 4)", "drop s.3"
exit
try:
  parse arg ex, setup
  s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
  t.0 = 2; t.1 = 'x'; t.2 = 'y'
  interpret setup
  signal on syntax name e
  interpret 'r = ' ex
  say r
  return
e: say condition('o')~code
  return
```

```output
40.919
88.905
88.905
88.904
40.920
40.922
40.923
40.922
40.923
40.922
40.924
```

r[util.sysstemcopy.fail]
A call that raises `SYNTAX 40.924` has copied the items before the one that
has no value: they are in the target, but the count of the target has not been
changed, even where the copy was to make it larger.

```rexx
s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.4 = 'd'
t.0 = 2; t.1 = 'x'; t.2 = 'y'
signal on syntax name e
call SysStemCopy 's.', 't.', 1, 2, 4
e: say condition('o')~code t.0 t.1 t.2 t.3
```

```output
40.924 2 x a b
```
