---
element: "SysStemInsert"
kind: function
syntax: "SysStemInsert(stem, position, value)"
rexxref: "Rexx Utilities (RexxUtil) > SysStemInsert"
---

# SysStemInsert

SysStemInsert puts one new item into a stem array at a place of the array:
the items from that place move up, and the count of the stem gets larger by
one. It answers `0` when it has done so. A call that cannot be done raises a
`SYNTAX` condition and never answers another number. The stem array is the one
that [the chapter page](index.md) describes.

> **Note.** Most examples of this page run in a routine that sets the stem to
> the four items `a b c d` before every call, and that prints the answer or
> the error code, and then the stem as its count, a colon and its items.

## Arguments

r[util.syssteminsert.args]
`stem`, `position` and `value` are all required. `stem` names a stem as
[the stem name rule](r:util.index.stemname) says. The call follows
[the argument rule](r:util.index.args): an argument that is missing, or is
omitted, raises `SYNTAX 88.901`, and a fourth argument raises
`SYNTAX 88.922` ([Deviation 101](r:util.index.deviation.101) is about which
of the two comes first). `position` is read as a whole number that is at
least `1`, with its blanks, a leading plus sign, a decimal point and an
exponent as any whole number is, so that `' 2 '` and `1e0` are numbers. A
`position` that is `0`, negative, fractional, text, the null string or `.nil`
raises `SYNTAX 88.905`. `value` is any object, and the null string and a
string of blanks are values.

```rexx
call try "SysStemInsert()"
call try "SysStemInsert('s.')"
call try "SysStemInsert('s.', 2)"
call try "SysStemInsert('s.', 2, )"
call try "SysStemInsert('s.', , 'v')"
call try "SysStemInsert('s.', 2, 'v', 'w')"
call try "SysStemInsert('s.', 0, 'v')"
call try "SysStemInsert('s.', -1, 'v')"
call try "SysStemInsert('s.', 1.5, 'v')"
call try "SysStemInsert('s.', 'x', 'v')"
call try "SysStemInsert('s.', '', 'v')"
call try "SysStemInsert('s.', .nil, 'v')"
call try "SysStemInsert('s.', ' 2 ', 'v')"
call try "SysStemInsert('s.', 1e0, 'v')"
call try "SysStemInsert('s.', +1, 'v')"
call try "SysStemInsert('s.', 2, '')"
call try "SysStemInsert('s.', 2, '  pad  ')"
call try "SysStemInsert('s.', 2, 12)"
call try "SysStemInsert(.mutablebuffer~new('s.'), .mutablebuffer~new('2'), 'v')"
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
  do i = 1 to s.0; out = out '[' || s.i || ']'; end
  return out
```

```output
88.901 4 : [a] [b] [c] [d]
88.901 4 : [a] [b] [c] [d]
88.901 4 : [a] [b] [c] [d]
88.901 4 : [a] [b] [c] [d]
88.901 4 : [a] [b] [c] [d]
88.922 4 : [a] [b] [c] [d]
88.905 4 : [a] [b] [c] [d]
88.905 4 : [a] [b] [c] [d]
88.905 4 : [a] [b] [c] [d]
88.905 4 : [a] [b] [c] [d]
88.905 4 : [a] [b] [c] [d]
88.905 4 : [a] [b] [c] [d]
0 5 : [a] [v] [b] [c] [d]
0 5 : [v] [a] [b] [c] [d]
0 5 : [v] [a] [b] [c] [d]
0 5 : [a] [] [b] [c] [d]
0 5 : [a] [  pad  ] [b] [c] [d]
0 5 : [a] [12] [b] [c] [d]
0 5 : [a] [v] [b] [c] [d]
```

## What is inserted

r[util.syssteminsert.insert]
The call moves the items `position` to the count of the stem up by one place,
one at a time from the last, and stores `value` in the tail `position`. The
count of the stem is then its old count plus one, written as a plain whole
number. `position` may be one past the count, and the value is then appended;
the stem may be empty, and `position` is then `1`. A tail above the old count
is overwritten when an item moves into it. The value is stored as it is, so
that an object is the item itself and not its string, and `.nil` is stored as
`.nil`. The items below `position` are not read or changed.

```rexx
call try "SysStemInsert('s.', 1, 'n')"
call try "SysStemInsert('s.', 2, 'n')"
call try "SysStemInsert('s.', 4, 'n')"
call try "SysStemInsert('s.', 5, 'n')"
call try "SysStemInsert('s.', 1, 'n')", "s.0 = '04'"
call try "SysStemInsert('s.', 1, 'n')", "s.0 = 0"
call try "SysStemInsert('s.', 1, 'n')", "drop s.; s.0 = 0"
call try "SysStemInsert('s.', 1, 'n')", "s.5 = 'beyond'"
call try "SysStemInsert('s.', 5, 'n')", "s.5 = 'beyond'"
call try "SysStemInsert('s.', 5, 'n')", "s.6 = 'beyond'"
s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
o = .array~of(7)
call SysStemInsert 's.', 1, o
say s.0 (s.1 == o) s.1~class~id s.2
call SysStemInsert 's.', 1, .nil
say s.0 (s.1 == .nil) s.2~class~id
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
  return out '|' s.5 s.6
```

```output
0 5 : n a b c d | d S.6
0 5 : a n b c d | d S.6
0 5 : a b c n d | d S.6
0 5 : a b c d n | n S.6
0 5 : n a b c d | d S.6
0 1 : n | d S.6
0 1 : n | S.5 S.6
0 5 : n a b c d | d S.6
0 5 : a b c d n | n S.6
0 5 : a b c d n | n beyond
5 1 Array a
6 1 Array
```

r[util.syssteminsert.range]
The call raises `SYNTAX 40.923`, and changes nothing, when `position` is more
than one past the count of the stem: an array of four items takes the
positions `1` to `5`, and an empty one takes only `1`.

```rexx
call try "SysStemInsert('s.', 6, 'n')"
call try "SysStemInsert('s.', 99999999999, 'n')"
call try "SysStemInsert('s.', 2, 'n')", "s.0 = 0"
call try "SysStemInsert('s.', 3, 'n')", "s.0 = 1"
exit
try:
  parse arg ex, setup
  s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
  interpret setup
  signal on syntax name e
  interpret 'r = ' ex
  say r s.0
  return
e: say condition('o')~code s.0
  return
```

```output
40.923 4
40.923 4
40.923 0
40.923 1
```

## Order of the checks

r[util.syssteminsert.order]
When several things are wrong, the call raises the condition of the first of
these checks that fails: the name of the stem raises `SYNTAX 40.919`; then
`position` raises `SYNTAX 88.905`; then the tail `0` of the stem raises
`SYNTAX 40.922`; then `position` raises `SYNTAX 40.923` when it is past the
count; and last an item that has to move and has no value raises
`SYNTAX 40.924`, with the count left as it was.

```rexx
call try "SysStemInsert('s.a', 'x', 'v')", ""
call try "SysStemInsert('s.', 0, 'v')", "drop s.0"
call try "SysStemInsert('s.', 9, 'v')", "drop s.0"
call try "SysStemInsert('s.', 9, 'v')", "drop s.2"
call try "SysStemInsert('s.', 1, 'v')", "drop s.2"
call try "SysStemInsert('s.', 5, 'v')", "drop s.2"
call try "SysStemInsert('s.', 3, 'v')", "drop s.4"
call try "SysStemInsert('s.', 4, 'v')", "drop s.3"
exit
try:
  parse arg ex, setup
  s.0 = 4; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'; s.4 = 'd'
  interpret setup
  signal on syntax name e
  interpret 'r = ' ex
  say r s.0
  return
e: say condition('o')~code s.0
  return
```

```output
40.919 4
88.905 S.0
40.922 S.0
40.923 4
40.924 4
0 5
40.924 4
0 5
```
