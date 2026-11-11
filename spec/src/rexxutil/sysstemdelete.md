---
element: "SysStemDelete"
kind: function
syntax: "SysStemDelete(stem, start, [count])"
rexxref: "Rexx Utilities (RexxUtil) > SysStemDelete"
---

# SysStemDelete

SysStemDelete removes one or more items from a stem array and closes the gap:
the items after them move down, and the count of the stem gets smaller. It
answers `0` when it has done so. A call that cannot be done raises a `SYNTAX`
condition and never answers another number. The stem array is the one that
[the chapter page](index.md) describes.

> **Note.** Most examples of this page run in a routine that sets the stem to
> the four items `a b c d` before every call, and that prints the answer or
> the error code, and then the stem as its count, a colon and its items.

## Arguments

r[util.sysstemdelete.args]
`stem` and `start` are required, and `count` is optional and is `1` when it is
left out. `stem` names a stem as [the stem name rule](r:util.index.stemname)
says. The call follows [the argument rule](r:util.index.args): a missing
required argument raises `SYNTAX 88.901` and a fourth argument raises
`SYNTAX 88.922` ([Deviation 101](r:util.index.deviation.101) is about which of
the two comes first). `start` and `count` are read as whole numbers that are
at least `1`, with their blanks, a leading plus sign, a decimal point and an
exponent as any whole number is, so that `' 2 '` and `1e0` are numbers. An
argument that is `0`, negative, fractional, text, the null string, blank or
`.nil` raises `SYNTAX 88.905`, and a count of `0` is such an argument too: the
call cannot be used to delete nothing.

```rexx
call try "SysStemDelete()"
call try "SysStemDelete('s.')"
call try "SysStemDelete('s.', )"
call try "SysStemDelete('s.', 1, 1, 1)"
call try "SysStemDelete('s.', 0)"
call try "SysStemDelete('s.', -1)"
call try "SysStemDelete('s.', 1.5)"
call try "SysStemDelete('s.', 'x')"
call try "SysStemDelete('s.', '')"
call try "SysStemDelete('s.', ' ')"
call try "SysStemDelete('s.', .nil)"
call try "SysStemDelete('s.', 2, 0)"
call try "SysStemDelete('s.', 2, -1)"
call try "SysStemDelete('s.', 2, 1.5)"
call try "SysStemDelete('s.', 2, '')"
call try "SysStemDelete('s.', 2, .nil)"
call try "SysStemDelete('s.', ' 2 ', ' 1 ')"
call try "SysStemDelete('s.', 1e0, +2)"
call try "SysStemDelete('s.', 2, )"
call try "SysStemDelete(.mutablebuffer~new('s.'), .mutablebuffer~new('2'))"
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
  return
show: procedure expose s.
  out = s.0 ':'
  do i = 1 to s.0; out = out s.i; end
  return out
```

```output
88.901 4 : a b c d
88.901 4 : a b c d
88.901 4 : a b c d
88.922 4 : a b c d
88.905 4 : a b c d
88.905 4 : a b c d
88.905 4 : a b c d
88.905 4 : a b c d
88.905 4 : a b c d
88.905 4 : a b c d
88.905 4 : a b c d
88.905 4 : a b c d
88.905 4 : a b c d
88.905 4 : a b c d
88.905 4 : a b c d
88.905 4 : a b c d
0 3 : a c d
0 2 : c d
0 3 : a c d
0 3 : a c d
```

## What is removed

r[util.sysstemdelete.delete]
The call removes the items `start` to `start + count - 1`. The items above
them move down by `count` places, one at a time and in rising order, so that
each takes the place of the item `count` below it, and the last `count` tails
of the array are dropped: after the call they have no value and are no longer
variables. The count of the stem is its old count less `count`, written as a
plain whole number. The items that are removed are not read, and may be
missing. The tails above the old count are not touched. The stem may be
emptied, and the count is then `0`.

```rexx
call try "SysStemDelete('s.', 1)"
call try "SysStemDelete('s.', 4)"
call try "SysStemDelete('s.', 2)"
call try "SysStemDelete('s.', 2, 2)"
call try "SysStemDelete('s.', 2, 3)"
call try "SysStemDelete('s.', 1, 4)"
call try "SysStemDelete('s.', 1)", "s.0 = '04'"
call try "SysStemDelete('s.', 3)", "drop s.3"
call try "SysStemDelete('s.', 2, 2)", "drop s.2"
call try "SysStemDelete('s.', 2, 2)", "s.5 = 'beyond'"
s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
call SysStemDelete 's.', 2, 2
say s.0 symbol('s.2') symbol('s.3') symbol('s.4')
exit
try:
  parse arg ex, setup
  s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
  interpret setup
  signal on syntax name e
  interpret 'r = ' ex
  say r show()
  return
e: say condition('o')~code show()
  return
show: procedure expose s.
  out = s.0 ':'
  do i = 1 to s.0; out = out s.i; end
  return out ':' symbol('s.5')
```

```output
0 3 : b c d : LIT
0 3 : a b c : LIT
0 3 : a c d : LIT
0 2 : a d : LIT
0 1 : a : LIT
0 0 : : LIT
0 3 : b c d : LIT
0 3 : a b d : LIT
0 2 : a d : LIT
0 2 : a d : VAR
2 VAR LIT LIT
```

r[util.sysstemdelete.range]
The call raises `SYNTAX 40.923`, and changes nothing, when the items to be
removed are not all in the stem, that is when `start + count - 1` is more
than its count. The check is made before any item is moved or read. An empty
stem has no item to remove, so that every `start` raises it.

```rexx
call try "SysStemDelete('s.', 4, 2)"
call try "SysStemDelete('s.', 5)"
call try "SysStemDelete('s.', 1, 5)"
call try "SysStemDelete('s.', 2, 4)"
call try "SysStemDelete('s.', 99999999999)"
call try "SysStemDelete('s.', 1, 99999999999)"
call try "SysStemDelete('s.', 1)", "s.0 = 0"
call try "SysStemDelete('s.', 4)", "s.0 = 3"
exit
try:
  parse arg ex, setup
  s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
  interpret setup
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
40.923 4 : a b c d
40.923 4 : a b c d
40.923 4 : a b c d
40.923 4 : a b c d
40.923 4 : a b c d
40.923 4 : a b c d
40.923 0 :
40.923 3 : a b c
```

## Order of the checks

r[util.sysstemdelete.order]
When several things are wrong, the call raises the condition of the first of
these checks that fails: the name of the stem raises `SYNTAX 40.919`; then
`start` and `count`, in that order, raise `SYNTAX 88.905`; then the tail `0`
of the stem raises `SYNTAX 40.922`; then the range raises
`SYNTAX 40.923`; and last an item that has to move and has no value raises
`SYNTAX 40.924`.

```rexx
call try "SysStemDelete('s.a', 'x')", ""
call try "SysStemDelete('s.', 0, -1)", ""
call try "SysStemDelete('s.', 1, 'x')", ""
call try "SysStemDelete('s.', 0)", "drop s.0"
call try "SysStemDelete('s.', 9)", "drop s.0"
call try "SysStemDelete('s.', 9)", "drop s.2"
call try "SysStemDelete('s.', 1)", "drop s.2"
exit
try:
  parse arg ex, setup
  s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
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
88.905
40.922
40.923
40.924
```

r[util.sysstemdelete.fail]
A call that raises `SYNTAX 40.924` has moved the items before the one that
has no value, and has not changed the count of the stem. Items that move are
read one at a time from the lowest, and a tail that is read and has no value
stops the call there.

```rexx
s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.4 = 'd'
signal on syntax name e
call SysStemDelete 's.', 1
e: say condition('o')~code s.0 s.1 s.2 symbol('s.3') s.4
```

```output
40.924 4 b b LIT d
```
