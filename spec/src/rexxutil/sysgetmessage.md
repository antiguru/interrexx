---
element: "SysGetMessage"
kind: function
syntax: "SysGetMessage(num [, filename [, str]...])"
rexxref: "Rexx Utilities (RexxUtil) > SysGetMessage (Unix-like systems only)"
---

# SysGetMessage

SysGetMessage answers a message from a message catalog, with the places in it
for inserts filled with the strings that the call gives. The catalog is either
the interpreter's own, which holds the messages of the interpreter, or a Unix
message catalog that the program names. The call takes its message from the
first message set of the catalog; [SysGetMessageX](sysgetmessagex.md) takes a
set as an argument. Which numbers the interpreter's own catalog has a message
for, and what the messages say, is not part of the book (see
[the rule on error messages](r:conv.error.name)), so the examples of this page
that use it look only at the inserts and at the answers for numbers that have
no message, and the examples that print a message make a catalog of their own.

## Arguments

r[util.sysgetmessage.args]
`num` is a required argument. `filename` is an optional string, and the
arguments after it are optional strings, the inserts. A call with no `num`, or
an omitted `num`, raises `SYNTAX 88.901`. `num` is read as a number, and a
string that is not a number, a number that is not whole and a whole number
that is not above zero raise `SYNTAX 88.905`; a sign, leading zeros, blanks
around the number and an exponent are allowed when the value is a whole number
above zero. A `filename` of `.nil` raises `SYNTAX 88.909`. A call with more
than nine inserts raises `SYNTAX 40.1`. An omitted place at the end of the
call adds nothing to the count. A string is read from an object that is not a
string as [the argument rule](r:util.index.args) says.

```rexx
call try "SysGetMessage()"
call try "SysGetMessage(,'rexx.cat')"
call try "SysGetMessage(0)"
call try "SysGetMessage(-3)"
call try "SysGetMessage(1.5)"
call try "SysGetMessage('x')"
call try "SysGetMessage('')"
call try "SysGetMessage(.nil)"
call try "SysGetMessage(100, .nil)"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j')"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', , )"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i') == SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c')"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c') == SysGetMessage('100.0', , 'a', 'b', 'c')"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c') == SysGetMessage(' +0100 ', , 'a', 'b', 'c')"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c') == SysGetMessage('1e2', , 'a', 'b', 'c')"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c') == SysGetMessage(.mutablebuffer~new(100), .mutablebuffer~new('rexx.cat'), 'a', 'b', 'c')"
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
88.905
88.905
88.905
88.905
88.905
88.905
88.909
40.1
Error a running b, line c:.
1
1
1
1
1
```

## The catalog

r[util.sysgetmessage.catalog]
A `filename` that is omitted, or is `rexx.cat` exactly as written, names the
interpreter's own catalog. Any other `filename` names a Unix message catalog,
as `gencat` makes it. A name with a slash in it is the path of the file, taken
from the current directory when it does not start with a slash. A name with no
slash is searched for along the environment variable `NLSPATH`, in which `%N`
stands for the name; when `NLSPATH` does not name it, no catalog is found. Any
other name, `REXX.CAT`, `./rexx.cat` and the null string included, is looked
for as the name of a Unix catalog. A catalog that is not found, and a message
that the catalog does not have, are not errors: the call answers [a text for
each](r:util.sysgetmessage.answer).

```rexx
say (SysGetMessage(100, 'rexx.cat', 'c6zqa', 'c6zqb', 'c6zqc') == SysGetMessage(100, , 'c6zqa', 'c6zqb', 'c6zqc'))
m = SysGetMessage(100, , 'c6zqa', 'c6zqb', 'c6zqc')
say (pos('c6zqa', m) > 0) (pos('c6zqb', m) > 0) (pos('c6zqc', m) > 0) (pos('&', m) = 0)
say '['SysGetMessage(100, 'REXX.CAT')']'
say '['SysGetMessage(100, 'rexx.cat ')']'
say '['SysGetMessage(100, './rexx.cat')']'
say '['SysGetMessage(100, '')']'
say '['SysGetMessage(100, 'c6zqnone/x.cat')']'
say '['SysGetMessage(100, 'c6zqnone.cat')']'
```

```output
1
1 1 1 1
[Error: Message catalog not found]
[Error: Message catalog not found]
[Error: Message catalog not found]
[Error: Message catalog not found]
[Error: Message catalog not found]
[Error: Message catalog not found]
```

## The answer

r[util.sysgetmessage.answer]
The answer is the text of message `num` of the first set of the catalog, with
the inserts put in. When the catalog has no such message the answer is
`Error: Message not found`, and when no catalog is found the answer is
`Error: Message catalog not found`, each with the inserts of the call ignored.
A `CALL` leaves the answer in `RESULT`.

```rexx
say '['SysGetMessage(999999)']'
say '['SysGetMessage(999999, 'rexx.cat', 'c6zqa')']'
say '['SysGetMessage(100, 'c6zqnone.cat', 'c6zqa')']'
call SysGetMessage 999999
say '['result']'
```

```output
[Error: Message not found]
[Error: Message not found]
[Error: Message catalog not found]
[Error: Message not found]
```

## The inserts

r[util.sysgetmessage.inserts]
In the text of a message, a place for an insert is an `&` and a digit from `1`
to `9`: the place `&n` is replaced by the `n`th string after `filename`. A
place whose string was not given, or is omitted, is replaced by the null
string, and a string that no place uses is ignored. An insert is put in as it
is: a `&` in it is not read as a place. Every other character of the text
stays as it is: a `%` and a letter, an `&` with a `0` after it, an `&` at the
end, and an `&` that is followed by another `&`. A digit after a place is a
character of the text, so that `&10` is the first insert and a `0`. The
example makes a catalog with `gencat` in the current directory and shows these
cases.

```rexx
call lineout 'c6zqcat.msg', '$set 1'
call lineout 'c6zqcat.msg', '1 first [&1] [&2] %s'
call lineout 'c6zqcat.msg', '7 seven &9 &10 &0 &&1 &'
call lineout 'c6zqcat.msg', '$set 2'
call lineout 'c6zqcat.msg', '1 other [&1]'
call lineout 'c6zqcat.msg', '3 third'
call lineout 'c6zqcat.msg'
address system 'gencat c6zqcat.cat c6zqcat.msg'
say SysGetMessage(1, './c6zqcat.cat', 'A', 'B')
say SysGetMessage(1, './c6zqcat.cat', 'A')
say SysGetMessage(1, './c6zqcat.cat')
say SysGetMessage(1, './c6zqcat.cat', 'A', 'B', 'C', 'D')
say SysGetMessage(1, './c6zqcat.cat', , 'B')
say SysGetMessage(1, './c6zqcat.cat', '&2', 'B')
say SysGetMessage(1, './c6zqcat.cat', '', '')
say SysGetMessage(7, './c6zqcat.cat', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I')
say SysGetMessage(3, './c6zqcat.cat')
say SysGetMessage(2, './c6zqcat.cat')
say SysGetMessage(1, directory() || '/c6zqcat.cat', 'A')
say SysGetMessage(1, 'c6zqcat.cat', 'A')
saved = value('NLSPATH', directory() || '/%N', 'ENVIRONMENT')
say SysGetMessage(1, 'c6zqcat.cat', 'A')
say SysGetMessage(1, 'c6zqnone.cat', 'A')
if saved == '' then call value 'NLSPATH', .nil, 'ENVIRONMENT'
else call value 'NLSPATH', saved, 'ENVIRONMENT'
say SysGetMessage(1, 'c6zqcat.cat', 'A')
call SysFileDelete 'c6zqcat.msg'
call SysFileDelete 'c6zqcat.cat'
```

```output
first [A] [B] %s
first [A] [] %s
first [] [] %s
first [A] [B] %s
first [] [B] %s
first [&2] [B] %s
first [] [] %s
seven I A0 &0 &A &
Error: Message not found
Error: Message not found
first [A] [] %s
Error: Message catalog not found
first [A] [] %s
Error: Message catalog not found
Error: Message catalog not found
```
