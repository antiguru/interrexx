---
element: "RexxUtil"
kind: chapter
rexxref: "Rexx Utilities (RexxUtil)"
deviations: [34, 101]
---

# RexxUtil

RexxUtil is the set of functions that a program calls to work with files,
directories, stems, the macrospace, processes, time and the terminal. This
chapter specifies the functions that exist on a Unix-like system, one page per
function. This page holds what every function shares: how the functions are
made available, how their arguments are read, what a name of a file is, and
what the numbers they answer mean.

The pages of this chapter so far are:

* [SysFileDelete](sysfiledelete.md), [SysFileExists](sysfileexists.md),
  [SysIsFile](sysisfile.md), [SysIsFileDirectory](sysisfiledirectory.md) and
  [SysIsFileLink](sysisfilelink.md): removing a file and asking what a name
  is.
* [SysMkDir](sysmkdir.md) and [SysRmDir](sysrmdir.md): making and removing a
  directory.
* [SysFileCopy](sysfilecopy.md) and [SysFileMove](sysfilemove.md): copying and
  moving a file.
* [SysTempFileName](systempfilename.md) and
  [SysSearchPath](syssearchpath.md): inventing a name that is not in use, and
  finding a file along a list of directories.

The rules state the behavior of a Unix-like system. The examples make their
own files and directories, whose names start with `c1zq`, and remove them
again. A number that is a system's error number is stated by its form: the
examples show that it is not zero, never which number it is.

## Availability

r[util.index.available]
Every RexxUtil function is provided by the interpreter from the start of a
program: it needs no registration, no library directive and no file. It is
called like any other function, as `name(arguments)` or by CALL, and its name
is read in any case. It is not a method of `String`: sending its name to a
string as a message raises `SYNTAX 97.1`. A label of the code that makes the
call, and a routine of the program, are found before it as
[the search rule](r:instr.call.search) says, and so a name written as a
string reaches the function and not the label; a file of the name of the
function in the current directory is not run in its place.

```rexx
say SysIsFileLink('/') sysisfilelink('/') 'SYSISFILELINK'('/')
call SysIsFileDirectory '/'
say result
signal on syntax name e
say '/'~SysIsFileDirectory
e: say condition('o')~code
signal off syntax
call lineout 'sysfileexists.rex', 'return "file"'
call lineout 'sysfileexists.rex'
say SysFileExists('/') SysIsFile('/') SysIsFileLink('/')
call SysFileDelete 'sysfileexists.rex'
exit
SYSISFILELINK: return 'label'
::routine SysIsFile
return 'routine'
```

```output
label label 0
1
97.1
1 routine label
```

r[util.index.load]
`SysLoadFuncs` and `SysDropFuncs` take no argument, and each answers `0` and
changes nothing: the functions are callable before the first call of either,
and after any number of calls. They exist for programs that were written for
a system where the functions had to be loaded. Such a program may also ask
for them with [RXFUNCADD](r:bif.rxfuncadd.provided), which answers `0`
whatever its module and procedure are, and RXFUNCDROP does not remove them
([RXFUNCDROP](r:bif.rxfuncdrop.provided)). More than no argument raises
`SYNTAX 88.922`.

```rexx
say rxfuncquery('SysFileExists') rxfuncquery('SysLoadFuncs')
say rxfuncadd('SysLoadFuncs', 'rexxutil', 'SysLoadFuncs')
call SysLoadFuncs
say result
say SysDropFuncs() SysDropFuncs()
say SysFileExists('/') rxfuncquery('SysFileExists')
say rxfuncdrop('SysFileExists') SysFileExists('/')
signal on syntax name a
call SysLoadFuncs 'x'
a: say condition('o')~code
signal on syntax name b
say SysDropFuncs(,'x')
b: say condition('o')~code
```

```output
0 0
0
0
0 0
1 0
1 1
88.922
88.922
```

## Arguments

r[util.index.args]
The RexxUtil functions follow [the native call
rule](r:bif.index.call.native), not the rules for the number of arguments of
the other builtins: a required argument that is not given, or is omitted,
raises `SYNTAX 88.901`; an argument past the last one of the function raises
`SYNTAX 88.922`; an argument that is an object other than a string is
replaced by the string that its `request` with the argument `STRING` answers,
and an object that has none raises `SYNTAX 88.909`. An omitted place at the
end of the call adds nothing to the count, and an optional argument that is
omitted takes the default that the function's page states. Each page says
what its function reads as a string and what it reads as a number, and which
other error codes it raises. The order of these checks is the subject of
[Deviation 101](r:util.index.deviation.101).

```rexx
call try "SysFileExists()"
call try "SysFileExists(,)"
call try "SysFileExists('/', 'x')"
call try "SysFileExists(.nil)"
call try "SysFileExists(.object~new)"
call try "SysFileExists('/', )"
call try "SysFileExists(.mutablebuffer~new('/'))"
call try "SysFileCopy('c1zqidxa')"
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
88.922
88.909
88.909
1
1
88.901
```

r[util.index.names]
A name of a file or a directory is a string. A name that does not start with
a slash is taken from the current directory. The function cleans the name up
before it uses it, in the way [QUALIFY](r:bif.qualify.normal) does: a run of
slashes is one slash, a component that is a single period is dropped, a slash
at the end is dropped, and a component of two periods is dropped together with
the component before it, so that the name `c1zqidxd/../c1zqidxn` is the name
`c1zqidxn` even when `c1zqidxd` does not exist. The characters `*` and `?` are
characters of the name: no function expands a pattern, so a name that holds
one names a file that has them or no file. A name of the null string names
nothing.

```rexx
call lineout 'c1zqidxn', 'x'
call lineout 'c1zqidxn'
call SysMkDir 'c1zqidxd'
say SysIsFile('./c1zqidxn') SysIsFile('c1zqidxd/../c1zqidxn')
say SysIsFile('c1zqidxn/') SysIsFile('c1zqidxnone/../c1zqidxn')
say SysIsFile('c1zqidxd/./../c1zqidxn') SysFileExists('./')
say SysFileExists('c1zqidx*') SysFileExists('c1zqidxn?') SysFileExists('')
call SysFileDelete 'c1zqidx*'
say SysFileExists('c1zqidxn') SysIsFileDirectory('c1zqidxd')
call SysFileDelete 'c1zqidxn'
call SysRmDir 'c1zqidxd'
```

```output
1 1
1 1
1 1
0 0 0
1 1
```

r[util.index.home]
A name that is `~` alone, or that starts with `~/`, has the `~` replaced by
the value of the environment variable `HOME`, which must be the name of a
directory that starts with a slash. A name that starts with `~` and a user's
name, up to the first slash or the end, stands for the home directory of that
user, as [it does for DIRECTORY](r:bif.directory.user); when there is no such
user, no file has that name, and the name is never read as the name of a file
of the current directory, which `./~name` reaches.

```rexx
here = directory()
call SysMkDir 'c1zqidxh'
call lineout 'c1zqidxh/f', 'x'
call lineout 'c1zqidxh/f'
call lineout './~c1zqidxu', 'x'
call lineout './~c1zqidxu'
old = value('HOME', here || '/c1zqidxh', 'ENVIRONMENT')
say SysIsFile('~/f') SysIsFileDirectory('~') SysIsFileDirectory('~/')
say SysIsFile('~//f') SysIsFile('~/../c1zqidxh/f') SysIsFile('~/g')
say SysIsFile('~c1zqidxu') SysIsFile('./~c1zqidxu')
call value 'HOME', old, 'ENVIRONMENT'
call SysFileDelete './~c1zqidxu'
call SysFileDelete 'c1zqidxh/f'
call SysRmDir 'c1zqidxh'
```

```output
1 1 1
1 1 0
0 1
```

r[util.index.options]
A function that takes option letters reads each letter in upper or in lower
case, as the page of the function says which letters it knows and what an
option that it does not know raises.

```rexx
call lineout 'c1zqidxo', 'x'
call lineout 'c1zqidxo'
say '[' || SysSearchPath('c1zqidxnone', 'c1zqidxo', 'n') || ']'
say (SysSearchPath('c1zqidxnone', 'c1zqidxo', 'c') == directory() || '/c1zqidxo')
say (SysSearchPath('c1zqidxnone', 'c1zqidxo', 'C') == directory() || '/c1zqidxo')
call SysFileDelete 'c1zqidxo'
```

```output
[]
1
1
```

r[util.index.stems]
A function that is given a stem takes the name of a stem variable, with the
period at its end or without it, in any case. A function that fills a stem
sets the tail `0` to the number of entries and the tails `1` to that number to
the entries, and leaves every other tail of the stem as it was. A function
that reads the items of a stem needs the tail `0` to hold a count, and raises
`SYNTAX 40.922` when the stem has none. The functions of this chapter that
take a stem say which of the two they do.

```rexx
call lineout 'c1zqidxs1', 'x'
call lineout 'c1zqidxs1'
call lineout 'c1zqidxs2', 'x'
call lineout 'c1zqidxs2'
f.99 = 'old'
say SysFileTree('c1zqidxs*', 'f.', 'FO') f.0 (f.1 == f.2) f.99
drop f.
say SysFileTree('c1zqidxs*', 'F', 'FO') f.0 (f.1 == f.2) symbol('f.99')
s.0 = 2
s.1 = 'b'
s.2 = 'a'
say SysStemSort('S') s.1 s.2
signal on syntax name e
n.1 = 'x'
say SysStemSort('n.')
e: say condition('o')~code
call SysFileDelete 'c1zqidxs1'
call SysFileDelete 'c1zqidxs2'
```

```output
0 2 0 old
0 2 0 LIT
0 a b
40.922
```

## Answers

r[util.index.codes]
A function that works on the system answers a whole number for what it did:
`0` when it did it, and another number, the number the system gave for the
failure, when it could not. A failure of the system is an answer, never a
`SYNTAX` condition: a missing file, a directory in place of a file, a refused
permission and a name that is already taken all answer a number that is not
zero, and leave the program running. What a function raises is limited to bad
arguments, which the page of each function lists. Which number a particular
failure gets is a value of the host, so a program tests for zero and no more.
A function that answers a state, such as `SysFileExists`, answers `0` or `1`
and has no failure to report.

```rexx
say SysFileDelete('c1zqidxnone') > 0
say SysMkDir('c1zqidxnone/sub') > 0
say SysRmDir('c1zqidxnone') > 0
say SysFileCopy('c1zqidxnone', 'c1zqidxcopy') > 0
say SysFileMove('c1zqidxnone', 'c1zqidxmoved') > 0
say SysFileExists('c1zqidxcopy') SysFileExists('c1zqidxmoved')
say datatype(SysRmDir('c1zqidxnone'), 'W')
```

```output
1
1
1
1
1
0 0
1
```

## Functions that exist only on Windows

r[util.index.windows]
A RexxUtil function that exists only on Windows, such as `SysWinVer`,
`SysDriveMap` or `RxMessageBox`, is not provided on a Unix-like system.
Calling one raises `SYNTAX 43.1`, as a call of any name that no routine has
does, and [RXFUNCQUERY](r:bif.rxfuncquery.answer) and
[RXFUNCADD](r:bif.rxfuncadd.module) answer `1` for it. The list of these
functions is on the page of the functions of the host.

> **Note.** Support for the Windows-only functions on Windows is tracked in
> issue #215.

```rexx
call try "SysWinVer()"
call try "SysDriveMap()"
call try "RxMessageBox('text')"
say rxfuncquery('SysWinVer') rxfuncquery('SysDriveMap')
say rxfuncadd('SysWinVer', 'rexxutil', 'SysWinVer')
say rxfuncadd('RxMessageBox', 'rexxutil', 'RxMessageBox')
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
43.1
43.1
43.1
1 1
1
1
```

## Deviations

r[util.index.deviation.34]
In every name that a RexxUtil function takes, each `..` removes exactly the
one component before it, however the component was written ([Deviation
34](r:bif.qualify.deviation.34)). ooRexx removes none after a doubled slash
and two after a component that starts with a period, so a function can work on
a different file there.

```rexx
call lineout 'c1zqidxdv', 'x'
call lineout 'c1zqidxdv'
call SysMkDir 'c1zqidxdd'
say SysIsFile('c1zqidxdd//..//c1zqidxdv') SysIsFile('c1zqidxdd/.d/../../c1zqidxdv')
call SysFileDelete 'c1zqidxdv'
call SysRmDir 'c1zqidxdd'
```

```output
1 1
```

r[util.index.deviation.101]
The functions check their arguments in the order that every other builtin
follows: the number of arguments first, so that an argument past the last one
raises `SYNTAX 88.922` before an omitted or wrong argument is reported
([Deviation 101](r:bif.index.call.native.deviation.101)).
ooRexx reports an omitted argument, an object whose `request` answers no
string and an argument that is not a whole number before the count, and the
count before a value that is out of range.

```rexx
signal on syntax name a
say SysFileExists(, 'x')
a: say condition('o')~code
signal on syntax name b
say SysFileExists(.nil, 'x')
b: say condition('o')~code
signal on syntax name c
say SysFileCopy(, 'x', 'y')
c: say condition('o')~code
```

```output
88.922
88.922
88.922
```
