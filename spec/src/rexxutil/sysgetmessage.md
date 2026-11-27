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
string that is not a number, a number that is not whole, a whole number that is
not above zero and a whole number of more than 18 digits raise `SYNTAX 88.905`.
A sign, leading zeros, blanks around the number and an exponent are allowed
when the value is a whole number above zero. The value is read as [the
whole-number rule](r:util.index.whole) says. The number of the message is the
remainder of `num` divided by 4294967296. So `4294967297` names message `1`,
and a multiple of 4294967296 names no message, which [the answer
rule](r:util.sysgetmessage.answer) covers. A `filename` of `.nil` raises
`SYNTAX 88.909`. A call with more than nine inserts raises `SYNTAX 40.1`. A
call whose `num` and `filename` are both in error raises the error of one of
them, as [the precedence rule](r:conv.error.precedence) says, and so does one
with too many inserts and another fault. An omitted place at the end of the
call adds nothing to the count. `filename` is read from an object that is not a
string as [the argument rule](r:util.index.args) says, so an object whose
`request` answers no string raises `SYNTAX 88.909`. `num` and the inserts have
a fallback. An object that is not a string is replaced by the string that its
`request` with the argument `STRING` answers. When that answer is not a string,
the object is replaced by the answer of its `STRING` method. So `.object~new`,
whose `STRING` answers `an Object`, raises `SYNTAX 88.905` as `num` and is put
in as `an Object` as an insert, and an object whose only method is a `STRING`
that answers `2` names message `2`. The inserts are converted in order, and a
conversion that raises ends the call: no later insert is converted.

```rexx
call try "SysGetMessage()"
call try "SysGetMessage(,'rexx.cat')"
call try "SysGetMessage(0)"
call try "SysGetMessage(-3)"
call try "SysGetMessage(1.5)"
call try "SysGetMessage('x')"
call try "SysGetMessage('')"
call try "SysGetMessage(.nil)"
call try "SysGetMessage(.object~new)"
call try "SysGetMessage(100, .object~new)"
call try "SysGetMessage(.c6zqs~new, 'rexx.cat') == SysGetMessage(2, 'rexx.cat')"
call try "SysGetMessage(100, .nil)"
call try "SysGetMessage('1000000000000000000')"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j')"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', , ) == SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c')"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i') == SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c')"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c') == SysGetMessage('100.0', , 'a', 'b', 'c')"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c') == SysGetMessage(' +0100 ', , 'a', 'b', 'c')"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c') == SysGetMessage('1e2', , 'a', 'b', 'c')"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c') == SysGetMessage(.mutablebuffer~new(100), .mutablebuffer~new('rexx.cat'), 'a', 'b', 'c')"
call try "SysGetMessage(100, 'rexx.cat', 'a', 'b', 'c') == SysGetMessage('100.00000000000000000001', , 'a', 'b', 'c')"
call try "SysGetMessage('1.0000000000000000001')"
call try "SysGetMessage(5, 'rexx.cat', .c6zql~new, .c6zqr~new, .c6zql~new)"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::class c6zqs
::method string
  return '2'
::class c6zql
::method string
  say 'converted'
  return 'L'
::class c6zqr
::method string
  raise syntax 93.900 array('r')
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
88.905
88.909
1
88.909
88.905
40.1
1
1
1
1
1
1
1
88.905
converted
93.900
```

## The catalog

r[util.sysgetmessage.catalog]
A `filename` that is omitted, or is `rexx.cat` exactly as written, names the
interpreter's own catalog. Any other `filename` names a Unix message catalog,
as `gencat` makes it. A name with a slash in it is the path of the file, taken
from the current directory when it does not start with a slash. A name with no
slash is searched for along the environment variable `NLSPATH`, a list of
patterns separated by colons, in which `%N` stands for the name, and `%L` and
`%l` stand for the locale that `LANG` names when the call is made, and for its
language part. Which other places the system searches when no pattern names a
file is not specified. Any other name, `REXX.CAT`, `./rexx.cat` and the null
string included, is looked for as the name of a Unix catalog. A catalog that
is not found, and a message that the catalog does not have, are not errors:
the call answers [a text for each](r:util.sysgetmessage.answer), and so is
[a damaged catalog](r:util.sysgetmessage.damaged).

```rexx
say (SysGetMessage(100, 'rexx.cat', 'c6zqa', 'c6zqb', 'c6zqc') == SysGetMessage(100, , 'c6zqa', 'c6zqb', 'c6zqc'))
m = SysGetMessage(100, , 'c6zqa', 'c6zqb', 'c6zqc')
say (pos('c6zqa', m) > 0) (pos('c6zqb', m) > 0) (pos('c6zqc', m) > 0) (pos('&', m) = 0)
nc = SysGetMessage(100, 'c6zqnone.cat')
say (nc \== '') (nc \== m) (nc \== SysGetMessage(999999))
say (SysGetMessage(100, 'REXX.CAT') == nc) (SysGetMessage(100, 'rexx.cat ') == nc),
    (SysGetMessage(100, './rexx.cat') == nc) (SysGetMessage(100, '') == nc),
    (SysGetMessage(100, 'c6zqnone/x.cat') == nc)
```

```output
1
1 1 1 1
1 1 1
1 1 1 1 1
```

## A damaged catalog

r[util.sysgetmessage.damaged]
A Unix catalog file holds three 32-bit words (a magic number, the number of
buckets of its table and the number of planes of buckets), then two copies of
the table, one in each byte order, each entry of it three words (a set number,
a message number and the offset of the message), then the messages, each ended
by `'00'x`. A file that is shorter than the three words, or that does not start
with the magic number in either byte order, is a catalog that is not found. A
catalog is damaged when its table has no entries, when the two tables do not
fit in the file, when an entry of the table in the machine's byte order, one
that holds a message or not, gives an offset at or past the end of the file,
or when the last byte of the file is not `'00'x`. The other table is not read.
A call that names a damaged catalog answers a text that says so, for every set
and number it asks for and with the inserts of the call ignored. That text is
not null and differs from the two texts of [the answer
rule](r:util.sysgetmessage.answer), and its wording is not specified.

```rexx
call lineout 'c6zqdmg.msg', '$set 1'
call lineout 'c6zqdmg.msg', '1 first'
call lineout 'c6zqdmg.msg'
address system 'gencat c6zqdmg.cat c6zqdmg.msg'
s = .stream~new('c6zqdmg.cat'); sound = s~charin(1, s~chars); s~close
nm = SysGetMessage(2, './c6zqdmg.cat')
nc = SysGetMessage(1, './c6zqnone.cat')
call write sound || 'x'
bad = SysGetMessage(1, './c6zqdmg.cat')
say (bad \== '') (bad \== nm) (bad \== nc)
say (SysGetMessage(2, './c6zqdmg.cat', 'A') == bad) (SysGetMessageX(3, 1, './c6zqdmg.cat') == bad)
call write sound~left(sound~length - 1)
say (SysGetMessage(1, './c6zqdmg.cat') == bad)
call write sound || 'x' || '00'x
say SysGetMessage(1, './c6zqdmg.cat')
call write sound~left(8)
say (SysGetMessage(1, './c6zqdmg.cat') == nc)
call SysFileDelete 'c6zqdmg.msg'
call SysFileDelete 'c6zqdmg.cat'
exit
write: call SysFileDelete 'c6zqdmg.cat'
  call charout 'c6zqdmg.cat', arg(1)
  call charout 'c6zqdmg.cat'
  return
```

```output
1 1 1
1 1
1
first
1
```

## The answer

r[util.sysgetmessage.answer]
The answer is the text of message `num` of the first set of the catalog, with
the inserts put in. When the catalog has no such message the answer is a text
that says so, and when no catalog is found the answer is another text that says
so, each with the inserts of the call ignored. The two texts are not null and differ from each
other, and their wording is not specified, as [the rule on error
messages](r:conv.error.text) says. A `CALL` leaves the answer in `RESULT`.

```rexx
nm = SysGetMessage(999999)
nc = SysGetMessage(100, 'c6zqnone.cat')
say (nm \== '') (nc \== '') (nm \== nc)
say (SysGetMessage(999999, 'rexx.cat', 'c6zqa') == nm)
say (SysGetMessage(100, 'c6zqnone.cat', 'c6zqa') == nc)
call SysGetMessage 999999
say (result == nm)
```

```output
1 1 1
1
1
1
```

## The inserts

r[util.sysgetmessage.inserts]
In the text of a message, a place for an insert is an `&` and a digit from `1`
to `9`: the place `&n` is replaced by the `n`th string after `filename`. A
place whose string the call does not give, or leaves empty, becomes the null
string, and a string that no place names has no effect. An insert is put in as
it is: a `&` in it is not read as a place. Every other character of the text
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
nm = SysGetMessage(999999)
nc = SysGetMessage(100, 'c6zqnone.cat')
say SysGetMessage(1, './c6zqcat.cat', 'A', 'B')
say SysGetMessage(1, './c6zqcat.cat', 'A')
say SysGetMessage(1, './c6zqcat.cat')
say SysGetMessage(1, './c6zqcat.cat', 'A', 'B', 'C', 'D')
say SysGetMessage(1, './c6zqcat.cat', , 'B')
say SysGetMessage(1, './c6zqcat.cat', '&2', 'B')
say SysGetMessage(1, './c6zqcat.cat', '', '')
say SysGetMessage(1, './c6zqcat.cat', .c6zqs~new, .mutablebuffer~new('M'))
say SysGetMessage(7, './c6zqcat.cat', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I')
say (SysGetMessage(3, './c6zqcat.cat') == nm)
say (SysGetMessage(2, './c6zqcat.cat') == nm)
say SysGetMessage(1, directory() || '/c6zqcat.cat', 'A')
say SysGetMessage(4294967297, './c6zqcat.cat', 'A')
say (SysGetMessage(4294967296, './c6zqcat.cat', 'A') == nm)
say (SysGetMessage(1, 'c6zqcat.cat', 'A') == nc)
saved = value('NLSPATH', directory() || '/%N', 'ENVIRONMENT')
say SysGetMessage(1, 'c6zqcat.cat', 'A')
say (SysGetMessage(1, 'c6zqnone.cat', 'A') == nc)
if saved == '' then call value 'NLSPATH', .nil, 'ENVIRONMENT'
else call value 'NLSPATH', saved, 'ENVIRONMENT'
say (SysGetMessage(1, 'c6zqcat.cat', 'A') == nc)
call SysFileDelete 'c6zqcat.msg'
call SysFileDelete 'c6zqcat.cat'
exit
::class c6zqs
::method string
  return '2'
```

```output
first [A] [B] %s
first [A] [] %s
first [] [] %s
first [A] [B] %s
first [] [B] %s
first [&2] [B] %s
first [] [] %s
first [2] [M] %s
seven I A0 &0 &A &
1
1
first [A] [] %s
first [A] [] %s
1
1
first [A] [] %s
1
1
```
