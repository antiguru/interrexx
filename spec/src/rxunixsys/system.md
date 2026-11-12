---
element: "SysUname"
kind: chapter
syntax: "SysGetpid(); SysGetppid(); SysGettid(); SysGetuid(); SysGeteuid(); SysGetgid(); SysGetegid(); SysGetpgrp(); SysGetpgid(pid); SysGetsid(pid); SysKill(pid, signal); SysSignal(signal, action); SysUname([option]); SysGethostname(); SysGetsizeofptr(); SysGettzname1(); SysGettzname2(); SysSetxattr(file, name, value); SysGetxattr(file, name); SysListxattr(file); SysRemovexattr(file, name)"
rexxref: "Unix extensions > Process Functions; Unix extensions > Miscellaneous Functions"
---

# Processes, the system and file attributes

The functions on this page tell a program who and where it is, send signals, and
read the names the system gives itself and the extended attributes of files.
What every function of [the rxunixsys library](index.md) shares holds here too.
The numbers that identify a process, a group, a session, a user and a thread
depend on the run, so a rule states what holds for them in every run and an
example shows only that.

## Identity

r[rxunixsys.system.ids]
`SysGetpid()`, `SysGetppid()` and `SysGettid()` answer the number of the
process, of its parent and of the calling thread, `SysGetuid()` and `SysGeteuid()` the
real and the effective user number of the process, `SysGetgid()` and
`SysGetegid()` the real and the effective group number, `SysGetpgrp()` the number
of the process group of the process, and `SysGetpgid(pid)` and `SysGetsid(pid)` the
number of the process group and of the session of the process `pid`, where `0` names
the process itself. Each answers a whole number with no sign, except that the
process number of a process that has no parent is `0`. The number of a thread can have more digits than `NUMERIC DIGITS` allows a whole number to have. `SysGetpgid(pid)` and
`SysGetsid(pid)` answer `-1` with the error `3` for a process that does not
exist. A file made by the process belongs to its effective user.

```rexx
say datatype(SysGetpid(), 'W') datatype(SysGetppid(), 'W') (verify(SysGettid(), '0123456789') == 0)
say datatype(SysGetuid(), 'W') datatype(SysGeteuid(), 'W') datatype(SysGetgid(), 'W') datatype(SysGetegid(), 'W')
say (SysGetpid() > 0) (SysGetppid() >= 0) (SysGetpid() <> SysGetppid())
say (SysGetpgrp() > 0) (SysGetpgid(0) == SysGetpgrp()) (SysGetpgid(SysGetpid()) == SysGetpgrp())
say (SysGetsid(0) > 0) (SysGetsid(SysGetpid()) == SysGetsid(0))
say SysGetpgid(999999999) SysGeterrno() SysGetsid(999999999) SysGeterrno()
say SysGetpgid(2147483647) SysGeterrno()
call lineout 'c1zqf1', 'x'
call lineout 'c1zqf1'
say (SysStat('c1zqf1', 'u') == SysGeteuid())
call SysFileDelete 'c1zqf1'
::requires 'rxunixsys' library
```

```output
1 1 1
1 1 1 1
1 1 1
1 1 1
1 1
-1 3 -1 3
-1 3
1
```

r[rxunixsys.system.set]
`SysSetuid(uid)`, `SysSeteuid(uid)`, `SysSetgid(gid)`, `SysSetegid(gid)`,
`SysSetsid()`, `SysSetpgrp()` and `SysSetpgid(pid, group)` change the identity of the
process. Each answers `0` where the system made the change, and `-1` where
it refused with the error `1` for a process that may not change it and `3` for a
process that does not exist; `SysSetsid()` answers the number of the new session
instead of `0`. The change of a user or group number asks for a privilege, and
a process without it, as a program that runs for an ordinary user, is refused. The
functions of the group and session of a process answer as they do for a call that
needs no privilege: the process of an ordinary user is refused a `SysSetpgid` for
a process that does not exist, and `SysSetsid()` for a process that already leads
its group. The examples show only the refusals that hold in every run.

```rexx
say SysSetpgid(999999999, 0) SysGeterrno()
say SysSetpgid(2147483647, 2147483647) SysGeterrno()
::requires 'rxunixsys' library
```

```output
-1 3
-1 3
```

## Signals

r[rxunixsys.system.kill]
`SysKill(pid, signal)` sends the signal with the number `signal` to the process
`pid`, as the `kill` command does, and answers `0`, or `-1` with the error `3` for
a process that does not exist, `1` for one that the process may not signal and
`22` for a signal number that does not exist. A `pid` of `0` names every process
of the process group of the process, itself included. The signal `0` sends nothing and
asks only whether the process could be signalled. The other signals do what the
process has set for them with [SysSignal](r:rxunixsys.system.signal), or what
the system does by default: a signal sent to the program itself can end it, which
the examples do not do.

```rexx
say SysKill(SysGetpid(), 0) SysKill(0, 0)
say SysKill(999999999, 0) SysGeterrno()
say SysKill(SysGetpid(), 99999) SysGeterrno()
say SysKill(SysGetpid(), '-1') SysGeterrno()
::requires 'rxunixsys' library
```

```output
0 0
-1 3
-1 22
-1 22
```

r[rxunixsys.system.signal]
`SysSignal(signal, action)` sets what the process does on receiving the signal with
the number `signal`: `action` is `D` for the default action of the system and `I`
for ignoring it, by its first character in either case, and any other first
character, and the null string, raise `SYNTAX 40.1`. The call answers `0` whether
the system took the change or not: a signal number that does not exist, `0`,
and the signals `9` and `19`, whose action cannot be set, are refused, and then
`SysGeterrno()` answers `22`. A signal that is ignored does not stop the
program when it is sent, as the example shows for `SysKill`. A `SysSignal` of the
signals that the interpreter handles itself, as `2` is, replaces its handling
for as long as the program runs.

```rexx
say SysSignal(12, 'I') SysKill(SysGetpid(), 12) 'survived'
say SysSignal(12, 'i') SysSignal(12, 'Ignore') SysSignal(12, 'D') SysSignal(12, 'default')
say SysSignal(99999, 'D') SysGeterrno()
say SysSignal(0, 'I') SysGeterrno()
say SysSignal(9, 'I') SysGeterrno()
say SysSignal(19, 'D') SysGeterrno()
call try "SysSignal(12, 'x')"
call try "SysSignal(12, '')"
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
0 0 survived
0 0 0 0
0 22
0 22
0 22
0 22
40.1
40.1
```

## The system

r[rxunixsys.system.uname]
`SysUname([option])` answers one item of the name of the system: `S` the
name of the operating system, `N` the name of the host, `R` its release, `V` its
version and `M` the type of the machine. `option` is read by its first character in
either case, and is `S` where omitted, so that `SysUname()` answers `Linux`
on Linux. An option that starts with another character, and the null string,
raise `SYNTAX 40.1`. `SysGethostname()` answers what `SysUname('N')` does.

```rexx
say SysUname() SysUname('s') SysUname('system') SysUname('SYSTEM')
say (SysUname('n') == SysGethostname()) (SysUname('Node') == SysGethostname()) (length(SysGethostname()) > 0)
say (length(SysUname('r')) > 0) (length(SysUname('Release')) > 0) (length(SysUname('v')) > 0) (length(SysUname('m')) > 0)
say (SysUname('r') == SysUname('R')) (SysUname('m') == SysUname('MACHINE'))
call try "SysUname('x')"
call try "SysUname('')"
call try "SysUname('s', 'x')"
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
Linux Linux Linux Linux
1 1 1
1 1 1 1
1 1
40.1
40.1
88.922
```

r[rxunixsys.system.pointer]
`SysGetsizeofptr()` answers the size of a pointer in bits, `32` or `64`. The
interpreters answer `64` on a 64-bit system.

```rexx
say (SysGetsizeofptr() == 32) | (SysGetsizeofptr() == 64)
::requires 'rxunixsys' library
```

```output
1
```

r[rxunixsys.system.tzname]
`SysGettzname1()` and `SysGettzname2()` answer the abbreviations of the name of
the time zone of the program, the first for standard time and the second for
daylight saving time. The zone is the one the environment variable `TZ` names
at the time of the call, or the local zone of the system where it has none,
and the two are the same where the zone has no daylight saving time. A call
reads the variable again.

```rexx
call value 'TZ', 'EST5EDT', 'ENVIRONMENT'
say SysGettzname1() SysGettzname2()
call value 'TZ', 'UTC', 'ENVIRONMENT'
say SysGettzname1() SysGettzname2()
call value 'TZ', 'XYZ-3', 'ENVIRONMENT'
say SysGettzname1() SysGettzname2()
say (length(SysGettzname1()) > 0) (length(SysGettzname2()) > 0)
::requires 'rxunixsys' library
```

```output
EST EDT
UTC UTC
XYZ XYZ
1 1
```

## Extended attributes

r[rxunixsys.system.xattr]
An extended attribute is a named value that the file system keeps with a file. The
name of a user attribute starts with `user.`. `SysSetxattr(file, name, value)`
makes or replaces the attribute `name` of the file `file` and answers `0`, or `-1`;
`SysGetxattr(file, name)` answers the value, up to its first null character,
or the null string where there is none; `SysListxattr(file)` answers an `Array` of
the names of the attributes, empty for a file with none, and `SysRemovexattr(file,
name)` removes the attribute and answers `0`, or `-1`. The errors are `2` for a file
that does not exist, `61` for an attribute that the file does not have, `95` for a
name that does not start with a known prefix such as `user.` and `34` for a name
that is only a prefix; where the file does not exist, `SysListxattr` answers the
null string and not an array. The examples need a file system that keeps user attributes.

```rexx
call lineout 'c1zqf1', 'x'
call lineout 'c1zqf1'
list = SysListxattr('c1zqf1')
say list~class~id list~items
say '['SysGetxattr('c1zqf1', 'user.c1zqa')']' SysGeterrno()
say SysSetxattr('c1zqf1', 'user.c1zqa', 'val') SysGetxattr('c1zqf1', 'user.c1zqa')
say SysSetxattr('c1zqf1', 'user.c1zqb', '') '['SysGetxattr('c1zqf1', 'user.c1zqb')']'
say SysSetxattr('c1zqf1', 'user.c1zqc', 'a' || '00'x || 'b') c2x(SysGetxattr('c1zqf1', 'user.c1zqc'))
list = SysListxattr('c1zqf1')
say list~class~id list~items
do name over list~sort~allItems
  say name
end
say SysSetxattr('c1zqf1', 'user.c1zqa', 'new') SysGetxattr('c1zqf1', 'user.c1zqa')
say SysSetxattr('c1zqf1', 'c1zqa', 'v') SysGeterrno()
say SysSetxattr('c1zqf1', 'user.', 'v') SysGeterrno()
say SysRemovexattr('c1zqf1', 'user.c1zqnone') SysGeterrno()
say SysRemovexattr('c1zqf1', 'user.c1zqa') SysRemovexattr('c1zqf1', 'user.c1zqb') SysRemovexattr('c1zqf1', 'user.c1zqc')
say SysListxattr('c1zqf1')~items
call SysFileDelete 'c1zqf1'
say SysSetxattr('c1zqnone', 'user.c1zqa', 'v') SysGeterrno()
say '['SysGetxattr('c1zqnone', 'user.c1zqa')']' SysGeterrno()
say SysRemovexattr('c1zqnone', 'user.c1zqa') SysGeterrno()
say '['SysListxattr('c1zqnone')']' SysGeterrno()
::requires 'rxunixsys' library
```

```output
Array 0
[] 61
0 val
0 []
0 61
Array 3
user.c1zqa
user.c1zqb
user.c1zqc
0 new
-1 95
-1 22
-1 61
0 0 0
0
-1 2
[] 2
-1 2
[] 2
```
