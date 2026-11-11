---
element: "RexxUtil"
kind: chapter
rexxref: "Rexx Utilities (RexxUtil); Rexx Utilities (RexxUtil) > A Note on Error Codes"
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
* [SysStemCopy](sysstemcopy.md), [SysStemDelete](sysstemdelete.md),
  [SysStemInsert](syssteminsert.md) and [SysStemSort](sysstemsort.md): working
  on a stem array, and [SysDumpVariables](sysdumpvariables.md): writing the
  variables of the current scope.

The rules state the behavior of a Unix-like system. The examples make their
own files and directories, whose names start with `c1zq`, and remove them
again. What a function answers for a failure is what
[the answer rule](r:util.index.codes) gives: an error number of Linux for the
functions that change files, and a fixed value for the functions that the rule
names.

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
function in the current directory is not run in its place. Among the routines
that are not labels, the RexxUtil functions are where
[the search order](r:env.search.order) puts the functions that RXFUNCADD made
callable. A routine of the program, a public routine of a package that it
requires, and a macro added with the order `B` are found before a RexxUtil
function of the same name. A file and a macro added with the order `A` are
not.

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

```rexx
call lineout 'c1zqidxm.rex', 'return "macro"'
call lineout 'c1zqidxm.rex'
call SysAddRexxMacro 'SYSISFILE', 'c1zqidxm.rex', 'B'
call SysAddRexxMacro 'SYSISFILELINK', 'c1zqidxm.rex', 'A'
say SysIsFile('/') SysIsFileLink('/') 'SysIsFile'('/')
call SysDropRexxMacro 'SYSISFILE'
call SysDropRexxMacro 'SYSISFILELINK'
say SysIsFile('/') SysIsFileLink('/')
call SysFileDelete 'c1zqidxm.rex'
```

```output
macro 0 macro
0 0
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
characters of the name: no function except SysFileTree expands a pattern, so
a name that holds one names a file that has them or no file. A name of the
null string names nothing. SysFileTree has exceptions to this rule, which
[its rule for the `filespec`](r:util.sysfiletree.spec) states.

```rexx
call lineout 'c1zqidxn', 'x'
call lineout 'c1zqidxn'
call SysMkDir 'c1zqidxd'
say SysIsFile('./c1zqidxn') SysIsFile('c1zqidxd/../c1zqidxn')
say SysIsFile('c1zqidxn/') SysIsFile('c1zqidxnone/../c1zqidxn')
say SysIsFile('c1zqidxd/./../c1zqidxn') SysFileExists('./')
say SysFileExists('c1zqidx*') SysFileExists('c1zqidxn?') SysFileExists('')
say SysFileTree('c1zqidx?', 's.', 'FO') s.0 SysGetFileDateTime('c1zqidx?') SysFileTree('c1zqidxd/', 's.') s.0
signal on syntax name e
say SysFileTree('', 's.')
e: say condition('o')~code
signal off syntax
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
0 1 -1 0 0
40.21
1 1
```

r[util.index.home]
A name that is `~` alone, or that starts with `~/`, has the `~` replaced by
the value of the environment variable `HOME` as it is. When `HOME` is the null
string, `~` alone names nothing and `~/` is replaced by `/`. When it is any
other value that does not start with a slash, no name that starts with `~/`,
or is `~` alone, names an entry. A name that starts with `~` and a user's
name, up to the first slash or the end, stands for the home directory of that
user, as [it does for DIRECTORY](r:bif.directory.user); when there is no such
user, no file has that name, and the name is never read as the name of a file
of the current directory, which `./~name` reaches. SysFileTree has exceptions
to this rule, which [its rule for the `filespec`](r:util.sysfiletree.spec.dir)
states.

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
call value 'HOME', '', 'ENVIRONMENT'
say SysIsFileDirectory('~') SysIsFileDirectory('~/')
call value 'HOME', 'c1zqidxh', 'ENVIRONMENT'
say SysIsFile('~/f') SysIsFileDirectory('~')
call value 'HOME', old, 'ENVIRONMENT'
call SysMkDir './~c1zqidxv'
say SysFileTree('~c1zqidxv', 's.', 'DO') s.0 SysIsFileDirectory('~c1zqidxv')
call SysRmDir './~c1zqidxv'
call SysFileDelete './~c1zqidxu'
call SysFileDelete 'c1zqidxh/f'
call SysRmDir 'c1zqidxh'
```

```output
1 1 1
1 1 0
0 1
0 1
0 0
0 1 0
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

## Stem arrays

r[util.index.stemname]
The name of a stem that SysStemCopy, SysStemDelete and SysStemInsert take is
a string, read as [the conventions](r:conv.string.object) read any string
argument, so that a MutableBuffer that holds `s.` names the stem `s.`, and an
object that has only a `STRING` method names the stem that the method
answers. The name is a symbol of letters, digits and the characters `!`, `?`
and `_` that does not start with a digit, with one period after it or
without, in any case. The name, with the period that a name without one is
given, has at most 250 characters. Any other name raises `SYNTAX 40.919`: the
null string, a name that starts with a digit or a period, a name with a
period before its last character, one that holds a blank, a character that no
symbol has, a name that is too long, or the text of `.nil`. The stem is the
one of that name among the variables of the code that makes the call, so that
in a procedure it is the procedure's own stem unless the procedure exposes it.
[SysStemSort](r:util.sysstemsort.name) reads a name that has more after its
first period differently, and reads an object that is not a string
differently too.

```rexx
call p
say s.0 s.1
exit
p: procedure
  s.0 = 0; ab_1.0 = 0; q?.0 = 0
  say SysStemInsert('s', 1, 'a') SysStemInsert('S.', 1, 'b'),
    SysStemInsert('Ab_1.', 1, 'c') SysStemInsert('q?', 1, 'd')
  say s.0 s.1 s.2 ab_1.0 ab_1.1 q?.0 q?.1
  call SysStemInsert .mutablebuffer~new('s.'), 1, 'm'
  say s.0 s.1
  call SysStemInsert .sonly~new, 1, 'z'
  call SysStemDelete .sonly~new, 2
  call SysStemCopy .sonly~new, 'u.', 1, 1, 1
  say s.0 s.1 s.2 u.0 u.1
  call try "SysStemInsert('', 1, 'x')"
  call try "SysStemInsert('1s.', 1, 'x')"
  call try "SysStemInsert('.s', 1, 'x')"
  call try "SysStemInsert('s.a', 1, 'x')"
  call try "SysStemInsert('s..', 1, 'x')"
  call try "SysStemInsert('s t.', 1, 'x')"
  call try "SysStemInsert('s-.', 1, 'x')"
  call try "SysStemInsert(.nil, 1, 'x')"
  call SysStemCopy 's.', copies('a', 249), 1, 1, 1
  call try "SysStemInsert(copies('a', 249), 1, 'x')"
  call try "SysStemInsert(copies('a', 249) || '.', 1, 'x')"
  call try "SysStemInsert(copies('a', 250), 1, 'x')"
  call try "SysStemInsert(copies('a', 250) || '.', 1, 'x')"
  call try "SysStemInsert(copies('a', 251) || '.', 1, 'x')"
  return
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::class sonly
::method string
  return 's.'
```

```output
0 0 0 0
2 b a 1 c 1 d
3 m
3 z b 1 z
40.919
40.919
40.919
40.919
40.919
40.919
40.919
40.919
0
0
40.919
40.919
40.919
S.0 S.1
```

r[util.index.stemarray]
A stem array is a stem whose tail `0` holds the number of items, `n`, and
whose tails `1` to `n` hold the items, written as plain whole numbers, so that
`s.01` is not an item. A function that reads a stem array reads the tail `0`
first. It is `n` when it is a whole number, written as `4`, ` 4 `, `04`,
`4.0`, `4e0` or `+4`. SysStemCopy, SysStemDelete and SysStemInsert read it
rounded to 18 significant digits and need a magnitude below 10\*\*18.
SysStemSort reads it rounded to 9 significant digits and needs a magnitude
below 10\*\*9. The NUMERIC DIGITS of the caller plays no part. A stem without
that tail, a tail that is not a whole number after that rounding, such as
`1.5` or a word, a value at or above that limit, and a stem that has only a
default value (`s. = 'x'`) raise `SYNTAX 40.922`: the default of a stem is
never the value of a tail that has none. A count below `0` is read as a
number, and each function says what it does with it. An item that the
function reads, and that has no value, because it was never assigned or was
dropped, raises `SYNTAX 40.924`, and again the stem's default does not stand
for it. A position or a number of items that is outside the array raises
`SYNTAX 40.923`. An item can be any object, and a function that moves an
item moves the object itself, so that the item is still the same object. A
function that changes the count writes it as a plain whole number.

```rexx
call try "SysStemDelete('s.', 1)", "drop s.0"
call try "SysStemDelete('s.', 1)", "s.0 = 'x'"
call try "SysStemDelete('s.', 1)", "s.0 = 1.5"
call try "SysStemDelete('s.', 1)", "s. = 'dflt'"
call try "SysStemDelete('s.', 1)", "s. = 'dflt'; s.0 = 3"
call try "SysStemDelete('s.', 1)", "drop s.2"
call try "SysStemDelete('s.', 1)", "s.2 = .array~of(7)"
call try "SysStemDelete('s.', 3)", "drop s.2"
call try "SysStemDelete('s.', 4)"
call try "SysStemDelete('s.', 1)", "s.0 = ' 3 '"
call try "SysStemDelete('s.', 1)", "s.0 = '+3'"
call try "SysStemDelete('s.', 1)", "s.0 = '3.0'"
call try "SysStemDelete('s.', 1)", "s.0 = '3e0'"
call try "SysStemDelete('s.', 1)", "s.0 = '03'"
call try "SysStemDelete('s.', 1)", "s.0 = '1.000000000000000001'"
call try "SysStemDelete('s.', 1)", "s.0 = '1.00000000000000001'"
call try "SysStemDelete('s.', 1)", "s.0 = '1e18'"
call try "SysStemDelete('s.', 1)", "s.0 = -1"
call try "SysStemSort('s.')", "s.0 = '1.000000001'"
call try "SysStemSort('s.')", "s.0 = '1.00000001'"
call try "SysStemSort('s.')", "s.0 = -1"
numeric digits 30
call try "SysStemDelete('s.', 1)", "s.0 = '1.000000000000000001'"
numeric digits 3
call try "SysStemSort('s.')", "s.0 = '1.0001'"
exit
try:
  s.0 = 3; s.1 = 'a'; s.2 = 'b'; s.3 = 'c'
  interpret arg(2)
  signal on syntax name e
  interpret 'r = ' arg(1)
  signal off syntax
  say r show()
  return
e: say condition('o')~code show()
  return
show: procedure expose s.
  out = ''
  do i = 0 to 3
    if symbol('s.' || i) == 'VAR' then out = out s.i
    else out = out '-'
  end
  return '[' || strip(out) || ']'
```

```output
40.922 [- a b c]
40.922 [x a b c]
40.922 [1.5 a b c]
40.922 [dflt dflt dflt dflt]
40.924 [3 dflt dflt dflt]
40.924 [3 a - c]
0 [2 7 c -]
0 [2 a - -]
40.923 [3 a b c]
0 [2 b c -]
0 [2 b c -]
0 [2 b c -]
0 [2 b c -]
0 [2 b c -]
0 [0 - b c]
40.922 [1.00000000000000001 a b c]
40.922 [1e18 a b c]
40.922 [-1 a b c]
0 [1.000000001 a b c]
40.922 [1.00000001 a b c]
40.922 [-1 a b c]
0 [0 - b c]
40.922 [1.0001 a b c]
```

```rexx
o = .array~of(7)
s.0 = 2; s.1 = 'a'; s.2 = o; s.01 = 'zero-one'
call SysStemDelete 's.', 1
say s.0 (s.1 == o) s.1~class~id s.01
```

```output
1 1 Array zero-one
```

## Answers

r[util.index.codes]
A function that works on the system answers what it did, and a failure of the
system is an answer, never a `SYNTAX` condition. A missing file, a directory
in place of a file, a refused permission and a name that is already taken
leave the program running. What a function raises is limited to bad
arguments, which the page of each function lists. SysFileDelete, SysMkDir,
SysRmDir, SysFileCopy and SysFileMove answer `0` when they did it. Otherwise
they answer the system's error number for the failure: `2` for a name that
nothing has or a directory in it that does not exist, `13` for a permission
that the system refuses, `17` for a name that is taken, `20` for a name that
must be a directory and is not (a component before the last, or the target
of a directory that is moved), `21` for a directory where a file is needed,
`22` for a directory moved into itself, and `39` for a directory that is not
empty. Other failures answer the system's other numbers, such as `36` for a
name that is too long. The page of each of these five functions gives the
number of every failure it lists, and says where the function departs from
the system's number. The other functions answer fixed values, which their
pages state. SysFileTree answers `0` whether or not it could read what it was
asked for. SysFileSearch answers `0`, or `3` for a file that it cannot read.
SysSetFileDateTime answers `0`, or `-1` for every failure.
SysGetFileDateTime answers a time, or `-1`. SysTempFileName and
SysSearchPath answer a name, or the null string. A function that answers a
state, such as `SysFileExists`, answers `0` or `1` and has no failure to
report.

```rexx
say SysFileDelete('c1zqidxnone') SysMkDir('c1zqidxnone/sub') SysRmDir('c1zqidxnone')
say SysFileCopy('c1zqidxnone', 'c1zqidxcopy') SysFileMove('c1zqidxnone', 'c1zqidxmoved')
say SysFileExists('c1zqidxcopy') SysFileExists('c1zqidxmoved')
call SysMkDir 'c1zqidxd'
say SysMkDir('c1zqidxd') SysFileDelete('c1zqidxd') SysRmDir('c1zqidxd')
say SysFileTree('c1zqidxnone/*', 's.') SysFileSearch('x', 'c1zqidxnone', 's.') SysGetFileDateTime('c1zqidxnone') SysSetFileDateTime('c1zqidxnone')
```

```output
13 2 2
2 2
0 0
17 21 0
0 3 -1 -1
```

## Functions that exist only on Windows

r[util.index.windows]
A RexxUtil function that exists only on Windows, such as `SysWinVer`,
`SysDriveMap` or `RxMessageBox`, is not provided on a Unix-like system.
Calling one raises `SYNTAX 43.1`, as a call of any name that no routine has
does, and [RXFUNCQUERY](r:bif.rxfuncquery.answer) and
[RXFUNCADD](r:bif.rxfuncadd.module) answer `1` for it.

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
string, a stem name of SysStemCopy, SysStemDelete or SysStemInsert that
fails its check (SysStemSort checks its stem after the count, as interrexx
does), a whole-number argument of
SysStemCopy, SysStemDelete, SysStemInsert or SysStemSort that fails its check,
and a `mode` of SysMkDir that fails any of its checks before the count, and
the count before a `filler` or an `option` that it does not allow.

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
signal on syntax name d
say SysMkDir('c1zqidxdm', 'x', 'y')
d: say condition('o')~code
signal on syntax name f
say SysTempFileName('c1zqidx?', .nil, 'y')
f: say condition('o')~code
signal on syntax name g
say SysStemDelete('s.a', 1, 1, 1)
g: say condition('o')~code
signal on syntax name h
say SysStemDelete('s.', 'x', 1, 1)
h: say condition('o')~code
signal on syntax name i
say SysStemDelete('s.', 0, 1, 1)
i: say condition('o')~code
say SysIsFileDirectory('c1zqidxdm')
```

```output
88.922
88.922
88.922
88.922
88.922
88.922
88.922
88.922
0
```
