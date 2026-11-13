---
element: "rxunixsys"
kind: chapter
deviations: [164]
rexxref: "Unix extensions"
---

# The rxunixsys library

The library `rxunixsys` gives a program the calls of a Unix-like system that
RexxUtil leaves out: the identity of the process, signals, links and
permissions of files, the user, group and service databases, and the errors
of these calls. This page holds what every function of the library shares:
how the functions are made available, how their arguments are read, what they
answer, and how a program learns why a call failed. [The files
page](files.md) holds the functions that work on files and directories, [the
system page](system.md) those that work on the process and the system, and [the
databases page](databases.md) the lookups of users, groups and services.

The rules state the behavior of Linux. Values that depend on the process or on
the system, such as a process number, are stated as rules that hold wherever the
program runs, and the examples show only what holds in every run: that a process
number is a whole number, and not which one. The examples make their own files
and directories, whose names start with `c`, a digit and `zq`, and remove them
again, and they call the functions after one of them has been made available, as
[the availability rule](r:rxunixsys.index.available) says.

## Availability

r[rxunixsys.index.available]
The functions of the library are not provided from the start of a program. They
are made available by [RXFUNCADD](r:bif.rxfuncadd.library) of the name of any
one of them with the library name `rxunixsys`, or by a
[`::requires`](r:dir.requires.library) directive with the `LIBRARY` keyword.
From then on every function of the library can be called, as `name(arguments)` or
by CALL, with its name in any case. A name is not a method of `String`: sending
it to a string raises `SYNTAX 97.1`. A call before the library is available
raises `SYNTAX 43.1`. RXFUNCDROP of a name removes nothing: the library stays
available until the program ends. The functions are `SysGetpid`, `SysGetppid`,
`SysGettid`, `SysGetuid`, `SysGeteuid`, `SysGetgid`, `SysGetegid`, `SysSetuid`,
`SysSeteuid`, `SysSetgid`, `SysSetegid`, `SysSetsid`, `SysGetsid`, `SysSetpgrp`,
`SysGetpgrp`, `SysSetpgid`, `SysGetpgid`, `SysKill`, `SysSignal`, `SysUmask`,
`SysSymlink`, `SysLink`, `SysUnlink`, `SysChown`, `SysLchown`, `SysChroot`,
`SysClose`, `SysAccess`, `SysEuidaccess`, `SysMkdirUnix`, `SysRmdirUnix`,
`SysChmod`, `SysStat`, `SysGetdirlist`, `SysGetpwnam`, `SysGetpwuid`,
`SysGetgrnam`, `SysGetgrgid`, `SysGetservbyname`, `SysGetservbyport`,
`SysUname`, `SysGethostname`, `SysGetsizeofptr`, `SysGettzname1`,
`SysGettzname2`, `SysWordexp`, `SysSetxattr`, `SysGetxattr`, `SysListxattr`,
`SysRemovexattr`, `SysGeterrno` and `SysGeterrnomsg`. The directory functions
carry the suffix `Unix` because [RexxUtil](../rexxutil/index.md) has functions
of the names `SysMkDir` and `SysRmDir`.

```rexx
say rxfuncquery('SysUnlink') rxfuncquery('SysGetpid')
say rxfuncadd('SysUnlink', 'rxunixsys')
say rxfuncquery('SysUnlink') rxfuncquery('SysGetpid') rxfuncquery('SysMkdirUnix')
say datatype(SysGetpid(), 'W') datatype(SYSGETPID(), 'W') datatype('sysgetpid'(), 'W')
call SysGetpid
say datatype(result, 'W')
say rxfuncdrop('SysGetpid') rxfuncquery('SysGetpid') datatype(SysGetpid(), 'W')
signal on syntax name a
say 'x'~SysGetpid
a: say condition('o')~code
```

```output
1 1
0
0 0 0
1 1 1
1
1 0 1
97.1
```

```rexx
say datatype(SysGetpid(), 'W')
::requires 'rxunixsys' library
```

```output
1
```

```rexx
signal on syntax name a
say SysGetpid()
a: say condition('o')~code
```

```output
43.1
```

## Arguments

r[rxunixsys.index.args]
The functions follow [the native call rule](r:bif.index.call.native), as the
[RexxUtil](../rexxutil/index.md) functions do. A required argument that is not
given, or is omitted, raises `SYNTAX 88.901`. An argument past the last one of
the function raises `SYNTAX 88.922`, and the count is checked before the
arguments are read. An omitted place at the end of the call adds nothing to the
count. An argument that is an object other than a string is replaced by the
string that its `request` with the argument `STRING` answers, and a name or an
option for which an object has none raises `SYNTAX 88.909`. A name of a file,
user or group is read as far as its first null character, so that a string whose
null character comes before the end names what the part before it names.

```rexx
call try "SysUnlink()"
call try "SysUnlink(,)"
call try "SysUnlink('c1zqnone', 'x')"
call try "SysUnlink(.nil)"
call try "SysUnlink(.object~new)"
call try "SysUnlink(.mutablebuffer~new('c1zqnone'))"
call try "SysUnlink('c1zqnone' || '00'x || 'x')"
call try "SysMkdirUnix('c1zqd1')"
call try "SysMkdirUnix('c1zqd1', 493, 1)"
call try "SysSymlink('c1zqa')"
call try "SysStat('c1zqa')"
call try "SysStat('c1zqa', 'p', 'x')"
call try "SysGetpid('')"
call try "SysGetppid(1)"
call try "SysGetuid(1)"
call try "SysSetsid(1)"
call try "SysGetsid()"
call try "SysSetpgid(1)"
call try "SysKill(0)"
call try "SysKill(0, 0, 0)"
call try "SysGeterrno(1)"
call try "SysGeterrnomsg()"
call try "SysGeterrnomsg(1, 2)"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::requires 'rxunixsys' library
```

```output
88.901
88.901
88.922
88.909
88.909
-1
-1
88.901
88.922
88.901
88.901
88.922
88.922
88.922
88.922
88.922
88.901
88.901
88.901
88.922
88.922
88.901
88.922
```

r[rxunixsys.index.whole]
An argument that a function reads as a whole number, such as a process number,
a signal, a mode or a file descriptor, is a number that has a whole value and
lies from `-2147483648` to `2147483647`. It may be written as any
[number](r:num.formats.operand) with a whole value, as `2.0`, `' 2 '`, `+2`
and `1E0` are, and with blanks around it. Anything else raises `SYNTAX 88.907`:
a number with a fraction, a number outside that range, the null string and a
string that is not a number. An object other than a string is replaced as [the
argument rule](r:rxunixsys.index.args) says, and one with no string value raises
`SYNTAX 88.907` here.

```rexx
call try "SysGeterrnomsg(2.0)"
call try "SysGeterrnomsg(' 2 ')"
call try "SysGeterrnomsg('+2')"
call try "SysGeterrnomsg('1E0')"
call try "SysGeterrnomsg(2*3)"
call try "SysGeterrnomsg(1.5)"
call try "SysGeterrnomsg(2147483647)"
call try "SysGeterrnomsg('-2147483647')"
call try "SysGeterrnomsg('-2147483648')"
call try "SysGeterrnomsg('2147483648')"
call try "SysGeterrnomsg('-2147483649')"
call try "SysGeterrnomsg('')"
call try "SysGeterrnomsg('x')"
call try "SysGeterrnomsg(.nil)"
call try "SysGeterrnomsg(.object~new)"
call try "SysGeterrnomsg(.mutablebuffer~new('2'))"
call try "SysKill('x', 0)"
call try "SysKill(0, 'x')"
call try "SysUmask('x')"
call try "SysGetpwuid('x', 'n')"
call try "SysClose('x')"
say (SysGeterrnomsg(2.0) == SysGeterrnomsg(2)) (SysGeterrnomsg(' 2 ') == SysGeterrnomsg(2)),
    (SysGeterrnomsg('+2') == SysGeterrnomsg(2)) (SysGeterrnomsg('1E0') == SysGeterrnomsg(1)),
    (SysGeterrnomsg(2*3) == SysGeterrnomsg(6)) (SysGeterrnomsg(.mutablebuffer~new('2')) == SysGeterrnomsg(2)),
    (SysGeterrnomsg(2) \== SysGeterrnomsg(1))
exit
try:
  signal on syntax name e
  interpret 'r =' arg(1)
  say arg(1) '=>' (r \== '')
  return
e: say arg(1) '->' condition('o')~code
  return
::requires 'rxunixsys' library
```

```output
SysGeterrnomsg(2.0) => 1
SysGeterrnomsg(' 2 ') => 1
SysGeterrnomsg('+2') => 1
SysGeterrnomsg('1E0') => 1
SysGeterrnomsg(2*3) => 1
SysGeterrnomsg(1.5) -> 88.907
SysGeterrnomsg(2147483647) => 1
SysGeterrnomsg('-2147483647') => 1
SysGeterrnomsg('-2147483648') => 1
SysGeterrnomsg('2147483648') -> 88.907
SysGeterrnomsg('-2147483649') -> 88.907
SysGeterrnomsg('') -> 88.907
SysGeterrnomsg('x') -> 88.907
SysGeterrnomsg(.nil) -> 88.907
SysGeterrnomsg(.object~new) -> 88.907
SysGeterrnomsg(.mutablebuffer~new('2')) => 1
SysKill('x', 0) -> 88.907
SysKill(0, 'x') -> 88.907
SysUmask('x') -> 88.907
SysGetpwuid('x', 'n') -> 88.907
SysClose('x') -> 88.907
1 1 1 1 1 1 1
```

r[rxunixsys.index.option]
An option of a function, such as the second argument of `SysStat`, is read by
its first character alone, in either case, so that `p`, `P` and `Permissions`
are one option. A call whose option has a first character that the function
does not know, or is the null string, raises `SYNTAX 40.1`, and so does a call
of `SysStat`, `SysGetpwnam` or `SysGetgrnam` whose name is the null string: the
call fails and nothing is made, whatever the other arguments are. The count and
the whole numbers of the arguments are checked first. A name that is not the
null string and that the system does not know does not raise: it is a failed
call, as [the answer rule](r:rxunixsys.index.answer) says.

```rexx
call lineout 'c1zqf1', 'x'
call lineout 'c1zqf1'
call SysChmod 'c1zqf1', 'rw-r-----'
call try "SysStat('c1zqf1', 'x')"
call try "SysStat('c1zqf1', '')"
call try "SysStat('', 'p')"
call try "SysStat('x', 'x', 3)"
call try "SysStat('c1zqf1', 'p')"
call try "SysStat('c1zqf1', 'Perm')"
call try "SysStat('c1zqf1', 'PX')"
call try "SysUname('x')"
call try "SysUname('')"
call try "SysChmod('', 'rw-rw-rw-')"
call try "SysChmod('c1zqnone', '')"
call try "SysSignal(0, 'x')"
call try "SysSignal(0, '')"
call try "SysSignal('x', 'x')"
call try "SysGetpwnam('', 'n')"
call try "SysGetpwnam('root', '')"
call try "SysGetpwnam('root', 'x')"
call SysFileDelete 'c1zqf1'
exit
try:
  signal on syntax name e
  interpret 'say arg(1) ''=>'' ' arg(1)
  return
e: say arg(1) '->' condition('o')~code
  return
::requires 'rxunixsys' library
```

```output
SysStat('c1zqf1', 'x') -> 40.1
SysStat('c1zqf1', '') -> 40.1
SysStat('', 'p') -> 40.1
SysStat('x', 'x', 3) -> 88.922
SysStat('c1zqf1', 'p') => -rw-r-----
SysStat('c1zqf1', 'Perm') => -rw-r-----
SysStat('c1zqf1', 'PX') => -rw-r-----
SysUname('x') -> 40.1
SysUname('') -> 40.1
SysChmod('', 'rw-rw-rw-') => -1
SysChmod('c1zqnone', '') -> 40.1
SysSignal(0, 'x') -> 40.1
SysSignal(0, '') -> 40.1
SysSignal('x', 'x') -> 88.907
SysGetpwnam('', 'n') -> 40.1
SysGetpwnam('root', '') -> 40.1
SysGetpwnam('root', 'x') -> 40.1
```

## What the functions answer

r[rxunixsys.index.answer]
A function that makes a change answers `0` when it was made and `-1` when the
system refused it. A function that finds out a value answers the value, and
for a number that is `-1` where the system refused. A function that finds out a
string, or a lookup, answers the null string where the system refused or the
name is not known, and for a lookup of a user the same. Nothing that the system
refuses raises a condition: the program reads the answer, and
[SysGeterrno](r:rxunixsys.index.errno) tells why. The answers are written as
whole numbers with no sign unless they are `-1`, no blanks and no decimal point.

```rexx
say SysUnlink('c1zqnone') SysMkdirUnix('c1zqnone/c1zqnone', 493) SysRmdirUnix('c1zqnone')
say SysChmod('c1zqnone', 'rw-rw-rw-') SysLink('c1zqnone', 'c1zqnone2') SysSymlink('c1zqnone', '')
say '['SysStat('c1zqnone', 'p')']' '['SysGetpwnam('c1zqnone', 'u')']' '['SysGetpwuid(2000000000, 'n')']'
say length(SysUnlink('c1zqnone'))
::requires 'rxunixsys' library
```

```output
-1 -1 -1
-1 -1 -1
[] [] []
2
```

## Errors

r[rxunixsys.index.errno]
`SysGeterrno()` answers the number of the error of the call of the library that
failed last, and `SysGeterrnomsg(number)` answers the text of an error number.
The number is the one Linux gives the error: `2` for a name that does not
exist, `17` for one that does, `20` for a name that is not a directory, `21` for
a directory where none is wanted, `39` for a directory that is not empty, `13`
for a refused permission and `1` for an operation the process may not do. A
call that fails sets it; a call that does not fail leaves it as it is, and the
lookups of users, groups and services set it to `0` before they look. The
number is that of the failed call when `SysGeterrno` is called in the same
clause, after the failed call and after nothing else that fails, as in the
examples; what a later clause finds, and what another activity does to it, is
the subject of [Deviation 164](r:rxunixsys.index.deviation.164).
`SysGeterrnomsg` takes the number as a [whole
number](r:rxunixsys.index.whole) and answers a string that is not null for every
number, `0` and a number that no error has included: the system's description
of the error, as C's `strerror` gives it at run time, so that two numbers of
different errors answer different strings. The wording is that of the platform
and is not specified (see [the rule on error messages](r:conv.error.text)), and
the examples look only at whether strings are alike.

```rexx
say SysUnlink('c1zqnone') SysGeterrno()
say SysMkdirUnix('c1zqnone/c1zqnone', 493) SysGeterrno()
say SysMkdirUnix('c1zqd1', 493) SysMkdirUnix('c1zqd1', 493) SysGeterrno()
say SysRmdirUnix('c1zqd1') SysRmdirUnix('c1zqd1') SysGeterrno()
call lineout 'c1zqf1', 'x'
call lineout 'c1zqf1'
say SysMkdirUnix('c1zqf1', 493) SysGeterrno() SysRmdirUnix('c1zqf1') SysGeterrno()
call SysFileDelete 'c1zqf1'
say (SysGeterrnomsg(0) \== '') (SysGeterrnomsg(2147483647) \== '') (SysGeterrnomsg(1000) \== '')
do n over .array~of(1, 2, 13, 17, 20, 21, 22, 39)
  do m over .array~of(1, 2, 13, 17, 20, 21, 22, 39)
    if n \= m, SysGeterrnomsg(n) == SysGeterrnomsg(m) then say 'alike' n m
  end
end
say 'checked'
::requires 'rxunixsys' library
```

```output
-1 2
-1 2
0 -1 17
0 -1 2
-1 17 -1 20
1 1 1
checked
```

## Deviations

r[rxunixsys.index.deviation.164]
The error number belongs to the activity that made the call: each activity has
its own, which is `0` until a call of the library fails on it. A call of the
library that fails sets the number of its activity; a call that does not fail
leaves it as it is, except that a lookup of a user, a group or a service sets
it to `0` before it looks. No other clause changes it, whether it fails or
not, and no call on another activity does: after a failed call, a later clause
of the same activity reads the number of that call, however many clauses that
do not set it come between, and however many other activities make calls of
the library meanwhile. A program therefore cannot lose the number to another
activity between the failed call and the clause that reads it. In the example
the main activity's call fails with `2`; the method that `start` runs begins
with `0`, fails with `17`, and keeps `17` after a call that does not fail,
while the main activity still reads `2` and then `0` after a lookup.

```rexx
call SysMkdirUnix 'c1zqd1', 493
say SysUnlink('c1zqnone') SysGeterrno()
x = 1 + 2
say 'later' SysGeterrno()
m = .k~new~start('work')
say 'result' m~result
say 'after' SysGeterrno()
call SysGetpwnam 'c1zqnouser', 'n'
say 'lookup' SysGeterrno()
call SysRmdirUnix 'c1zqd1'
exit
::requires 'rxunixsys' library
::class k
::method work
  say 'b start' SysGeterrno()
  say SysMkdirUnix('c1zqd1', 493) SysGeterrno()
  call SysMkdirUnix 'c1zqd1x', 493
  say 'b ok' SysGeterrno()
  call SysRmdirUnix 'c1zqd1x'
  return SysGeterrno()
```

```output
-1 2
later 2
b start 0
-1 17
b ok 17
result 17
after 2
lookup 0
```
