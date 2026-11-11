---
element: "SysFormatMessage"
kind: function
syntax: "SysFormatMessage(message, substitutions)"
rexxref: "Rexx Utilities (RexxUtil) > SysFormatMessage"
---

# SysFormatMessage

SysFormatMessage answers a text in which the places for inserts are filled
with the strings that the call gives. It needs no message catalog: the text is
the first argument. The places and what becomes of them are the same as for
the message functions, [SysGetMessage](sysgetmessage.md) and
[SysGetMessageX](sysgetmessagex.md), whose
[inserts rule](r:util.sysgetmessage.inserts) says them in full.

## Arguments

r[util.sysformatmessage.args]
`message` and `substitutions` are required arguments. A call with either
missing or omitted raises `SYNTAX 88.901`, and a third argument raises `SYNTAX
88.922`; an omitted place at the end of the call adds nothing to the count
([the argument rule](r:util.index.args), and for the order of the checks
[Deviation 101](r:util.index.deviation.101)). `message` is checked before
`substitutions`, and a third argument is reported before either. `message` is
read as a string as [the argument rule](r:util.index.args) says: `.nil`, and
an object that has no `MAKESTRING` method, raise `SYNTAX 88.909`, and an
object with only a `STRING` method is such an object. A mutable buffer is read
as the string that it holds.

```rexx
call try "SysFormatMessage()"
call try "SysFormatMessage(,)"
call try "SysFormatMessage('a')"
call try "SysFormatMessage('a', )"
call try "SysFormatMessage(, 'x')"
call try "SysFormatMessage('a', 'b', 'c')"
call try "SysFormatMessage('[&1]', 'x', )"
call try "SysFormatMessage(.nil, 'x')"
call try "SysFormatMessage(.nil, .nil)"
call try "SysFormatMessage(.object~new, 'x')"
call try "SysFormatMessage(.c6zqs~new, 'x')"
call try "SysFormatMessage(.c6zqt~new, 'x')"
call try "SysFormatMessage(.mutablebuffer~new('[&1]'), 'X')"
call try "SysFormatMessage(12, 12)"
call try "SysFormatMessage('', 'x') == ''"
call try "SysFormatMessage(unsetvariable, 'x')"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::class c6zqs
::method string
  return 'S &1'
::class c6zqt
::method makestring
  return 'T &1'
```

```output
88.901
88.901
88.901
88.901
88.901
88.922
[x]
88.909
88.909
88.909
88.909
T x
[X]
12
1
UNSETVARIABLE
```

## The substitutions

r[util.sysformatmessage.substitutions]
`substitutions` gives the strings that fill the places `&1` to `&9`. It is
sent `MAKEARRAY`, and the items of the array that it answers, in order, are
the inserts: the first item is for `&1`, the second for `&2`, and so on. A
string is split at its line ends, as [its
`makeArray`](r:class.string.makearray) splits it. So a string with no line end
is one insert, the one for `&1`, and `'a' || '0a'x || 'b'` gives two. A
subclass whose `MAKEARRAY` is overridden, of String or of Array, gives the
items of the override. An object that has no `MAKEARRAY` method, `.nil`
included, raises `SYNTAX 98.913`, and so does an array of more than one
dimension. An item that is not a string is replaced by
the string that its `request` with the argument `STRING` answers. When that
answer is not a string, the item is replaced by the answer of its `STRING`
method, as with an insert of [SysGetMessage](r:util.sysgetmessage.args). So
`.nil` is `The NIL object`, and an array that holds `1` is `1`. A place for
which the array has no item gives the null string, and so does every place
when the array is empty. An array whose size is more than nine raises `SYNTAX
40.1`, even when fewer than ten of its places hold an item, and one of size
nine does not. The size is the array's own, so an override of `SIZE` in a
subclass of Array does not change it.

```rexx
call try "SysFormatMessage('[&1][&2]', 'X')"
call try "SysFormatMessage('[&1][&2]', .array~of('X', 'Y'))"
call try "SysFormatMessage('[&2][&1]', .list~of('a', 'b'))"
call try "SysFormatMessage('[&1][&2]', .array~new)"
call try "SysFormatMessage('[&1][&2][&3]', .array~of('X', , 'Z'))"
call try "SysFormatMessage('[&1][&2]', .c6zqu~new)"
call try "SysFormatMessage('[&1]', .array~of(.c6zqs~new))"
call try "SysFormatMessage('[&1]', .array~of(.nil))"
call try "SysFormatMessage('[&1]', .array~of(.object~new))"
call try "SysFormatMessage('[&1]', .array~of(12))"
call try "SysFormatMessage('[&1]', '')"
call try "SysFormatMessage('[&1]', ' x ')"
call try "SysFormatMessage('[&1]', .nil)"
call try "SysFormatMessage('[&1]', .object~new)"
call try "SysFormatMessage('[&1][&9]', .array~of(1, 2, 3, 4, 5, 6, 7, 8, 9))"
call try "SysFormatMessage('[&1]', .array~of(1, 2, 3, 4, 5, 6, 7, 8, 9, 10))"
call try "SysFormatMessage('[&1]', .array~of(1, 2, 3, 4, 5, 6, 7, 8, 9)~~put(10, 10))"
call try "SysFormatMessage('[&1][&2]', 'a' || '0a'x || 'b')"
call try "SysFormatMessage('[&1][&2]', .c6zqo~new('zz'))"
call try "SysFormatMessage('[&1]', .array~of(.c6zqm~new))"
call try "SysFormatMessage('[&1]', .array~of(.array~of(1)))"
call try "SysFormatMessage('[&1]', .array~new(9))"
call try "SysFormatMessage('[&1]', .array~new(10))"
call try "SysFormatMessage('[&1]', .array~new(2, 2)~~put('m', 1, 1))"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::class c6zqs
::method string
  return 'S'
::class c6zqu
::method makearray
  return .array~of('u1', 'u2')
::class c6zqo subclass string
::method makearray
  return .array~of('o1', 'o2')
::class c6zqm
::method makestring
  return 'M'
::method string
  return 'S'
```

```output
[X][]
[X][Y]
[b][a]
[][]
[X][][Z]
[u1][u2]
[S]
[The NIL object]
[an Object]
[12]
[]
[ x ]
98.913
98.913
[1][9]
40.1
40.1
[a][b]
[o1][o2]
[M]
[1]
[]
40.1
98.913
```

## The answer

r[util.sysformatmessage.answer]
The answer is `message` with each place `&n` replaced by insert `n`, as [the
inserts rule](r:util.sysgetmessage.inserts) of SysGetMessage says: a place
with no insert becomes the null string, an insert is put in as it is, so that
a `&` in it is not read as a place, and a `%`, an `&` with a `0` after it, an
`&` at the end and an `&` before another `&` stay in the text. A digit after a
place is a character of the text, so `&10` is insert `1` and a `0`. The length
of the text is not limited. `CALL` leaves the answer in `RESULT`.

```rexx
say SysFormatMessage('a [&1] b [&2] &10 &&1', .array~of('X', 'Y'))
say SysFormatMessage('[&1]&1', .array~of('&2', 'B'))
say SysFormatMessage('[&1] [&9] [&0] [&] x& 100% %s', .array~of(1, 2, 3, 4, 5, 6, 7, 8, 9))
say SysFormatMessage('&1&1&1', 'ab')
say SysFormatMessage('a&', 'ab')
say SysFormatMessage('&', 'ab')
say SysFormatMessage('&&', 'ab')
say SysFormatMessage('&&1', 'ab')
say SysFormatMessage('&0&1', 'ab')
say length(SysFormatMessage(copies('a', 5000) || '&1', copies('b', 3000)))
say SysFormatMessage('a', 'b') == 'a'
call SysFormatMessage 'm [&1]', 'v'
say result
```

```output
a [X] b [Y] X0 &X
[&2]&2
[1] [9] [&0] [&] x& 100% %s
ababab
a&
&
&&
&ab
&0ab
8000
1
m [v]
```
