---
element: "SysGetMessageX"
kind: function
syntax: "SysGetMessageX(set, num [, filename [, str]...])"
rexxref: "Rexx Utilities (RexxUtil) > SysGetMessageX (Unix-like systems only)"
---

# SysGetMessageX

SysGetMessageX answers a message from a message catalog like
[SysGetMessage](sysgetmessage.md), but takes the number of the message set as
its first argument, where SysGetMessage always uses the first set. Every rule
of that page holds for the arguments after `set`: [the catalog
rule](r:util.sysgetmessage.catalog) says which catalog a `filename` names,
[the inserts rule](r:util.sysgetmessage.inserts) says how the inserts are put
in, and [the answer rule](r:util.sysgetmessage.answer) gives the two texts for
a message and a catalog that are not found.

## Arguments

r[util.sysgetmessagex.args]
`set` and `num` are required arguments, read as numbers like the `num` of
[SysGetMessage](r:util.sysgetmessage.args): a string that is not a number, a
number that is not whole and a whole number that is not above zero raise
`SYNTAX 88.905` for either of them. A call without `set` or without `num`, or
with either omitted, raises `SYNTAX 88.901`. `filename`, the inserts, the
`.nil` that `filename` may not be and the limit of nine inserts are as that
rule says.

```rexx
call try "SysGetMessageX()"
call try "SysGetMessageX(1)"
call try "SysGetMessageX(,100)"
call try "SysGetMessageX(1,)"
call try "SysGetMessageX(0, 100)"
call try "SysGetMessageX(-1, 100)"
call try "SysGetMessageX(1.5, 100)"
call try "SysGetMessageX('x', 100)"
call try "SysGetMessageX(1, 0)"
call try "SysGetMessageX(1, 'x')"
call try "SysGetMessageX(1, 100, .nil)"
call try "SysGetMessageX(1, 100, 'rexx.cat', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j')"
call try "SysGetMessageX(1, 100, 'rexx.cat', 'a', 'b', 'c') == SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c')"
call try "SysGetMessageX('1e0', ' +100 ', , 'a', 'b', 'c') == SysGetMessage(100, , 'a', 'b', 'c')"
call try "SysGetMessageX(1, 999999)"
call try "SysGetMessageX(1, 100, 'REXX.CAT')"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
88.901
88.901
88.901
88.901
88.905
88.905
88.905
88.905
88.905
88.905
88.909
40.1
1
1
Error: Message not found
Error: Message catalog not found
```

## The answer

r[util.sysgetmessagex.answer]
The answer is the text of message `num` of set `set` of the catalog, with the
inserts put in. A set that the catalog does not have is a message that it does
not have: the answer is `Error: Message not found`, as it is for a number that
the set has no message for. A call with the set `1` answers what SysGetMessage
answers for the same `num`, `filename` and inserts.

```rexx
call lineout 'c6zqcat.msg', '$set 1'
call lineout 'c6zqcat.msg', '1 first [&1] [&2]'
call lineout 'c6zqcat.msg', '$set 2'
call lineout 'c6zqcat.msg', '1 other [&1]'
call lineout 'c6zqcat.msg', '3 third'
call lineout 'c6zqcat.msg', '$set 5'
call lineout 'c6zqcat.msg', '9 fifth nine &2'
call lineout 'c6zqcat.msg'
address system 'gencat c6zqcat.cat c6zqcat.msg'
say SysGetMessageX(1, 1, './c6zqcat.cat', 'A', 'B')
say SysGetMessageX(2, 1, './c6zqcat.cat', 'A', 'B')
say SysGetMessageX(2, 3, './c6zqcat.cat')
say SysGetMessageX(5, 9, './c6zqcat.cat', 'A', 'B')
say SysGetMessageX(1, 3, './c6zqcat.cat')
say SysGetMessageX(2, 9, './c6zqcat.cat')
say SysGetMessageX(3, 1, './c6zqcat.cat')
say SysGetMessageX(1, 1, './c6zqcat.cat', 'A', 'B') == SysGetMessage(1, './c6zqcat.cat', 'A', 'B')
say SysGetMessageX(2, 1, './c6zqcat.cat', 'A') \== SysGetMessage(1, './c6zqcat.cat', 'A')
say SysGetMessageX(1, 1, './c6zqnone.cat', 'A')
call SysFileDelete 'c6zqcat.msg'
call SysFileDelete 'c6zqcat.cat'
```

```output
first [A] [B]
other [A]
third
fifth nine B
Error: Message not found
Error: Message not found
Error: Message not found
1
1
Error: Message catalog not found
```
