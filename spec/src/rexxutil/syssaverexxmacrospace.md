---
element: "SysSaveRexxMacroSpace"
kind: function
syntax: "SysSaveRexxMacroSpace(file)"
rexxref: "Rexx Utilities (RexxUtil) > SysSaveRexxMacroSpace"
---

# SysSaveRexxMacroSpace

SysSaveRexxMacroSpace writes the [macrospace](macrospace.md) to a file, from
which [SysLoadRexxMacroSpace](sysloadrexxmacrospace.md) can bring it back. It
answers `0` when the file was written, and `5` when it was not.

## Arguments

r[util.syssaverexxmacrospace.args]
`file` is a required string. The count and the omission of the argument, and
an object in its place, follow [the argument rule](r:util.index.args): no
argument raises `SYNTAX 88.901`, a second argument raises `SYNTAX 88.922`, and
`.nil` raises `SYNTAX 88.909`. An object that is not a string but has a string
value is read by that value. A `file` that is the null string names no file
that can be written, so the call answers `5`.

```rexx
call lineout 'c4zqsav.rex', 'return "x"'
call lineout 'c4zqsav.rex'
call SysAddRexxMacro 'C4ZQSAV', 'c4zqsav.rex'
call try "SysSaveRexxMacroSpace()"
call try "SysSaveRexxMacroSpace(, )"
call try "SysSaveRexxMacroSpace('c4zqsavfile', 'x')"
call try "SysSaveRexxMacroSpace(.nil)"
call try "SysSaveRexxMacroSpace('')"
call try "SysSaveRexxMacroSpace(.mutablebuffer~new('c4zqsavfile'))"
say SysFileExists('c4zqsavfile')
call SysDropRexxMacro 'C4ZQSAV'
call SysFileDelete 'c4zqsavfile'
call SysFileDelete 'c4zqsav.rex'
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
5
0
1
```

## The result

r[util.syssaverexxmacrospace.answer]
When the macrospace holds at least one macro and the file can be written, the
call writes every macro to it, with its order, and answers `0`. A relative
`file` is taken from the current directory, and the name is cleaned as [the
name rule](r:util.index.names) says. A file that is already there is replaced,
whole: what it held before does not come back with a later load. The
macrospace itself is not changed, and the saved macros are the programs as
they were added, so the files they were made from are not needed to load them.
The form of the file is not specified: it is for
[SysLoadRexxMacroSpace](r:util.sysloadrexxmacrospace.answer) to read.

```rexx
call lineout 'c4zqsa1.rex', 'return "one"'
call lineout 'c4zqsa1.rex'
call lineout 'c4zqsa2.rex', 'return "two"'
call lineout 'c4zqsa2.rex'
call SysMkDir 'c4zqsadir'
call SysAddRexxMacro 'C4ZQSA1', 'c4zqsa1.rex', 'A'
call SysAddRexxMacro 'C4ZQSA2', 'c4zqsa2.rex', 'B'
say SysSaveRexxMacroSpace('c4zqsadir/../c4zqsadir/saved') SysFileExists('c4zqsadir/saved')
say SysQueryRexxMacro('C4ZQSA1') SysQueryRexxMacro('C4ZQSA2') C4ZQSA1() C4ZQSA2()
call SysFileDelete 'c4zqsa1.rex'
call SysDropRexxMacro 'C4ZQSA2'
say SysSaveRexxMacroSpace('c4zqsadir/saved')
call SysClearRexxMacroSpace
say SysLoadRexxMacroSpace('c4zqsadir/saved') '[' || SysQueryRexxMacro('C4ZQSA1') || '] [' || SysQueryRexxMacro('C4ZQSA2') || ']' C4ZQSA1()
call SysDropRexxMacro 'C4ZQSA1'
call SysFileDelete 'c4zqsadir/saved'
call SysRmDir 'c4zqsadir'
call SysFileDelete 'c4zqsa2.rex'
```

```output
0 1
A B one two
0
0 [A] [] one
```

r[util.syssaverexxmacrospace.fail]
The call answers `5`, and writes nothing, when the macrospace holds no macro:
there is then no file to load, and a file of that name that was there is left
as it was. It also answers `5` when the file cannot be written: when a
directory in its name is not there, when a component before the last is a
file, and when the name is that of a directory.

```rexx
call lineout 'c4zqsfail.rex', 'return "x"'
call lineout 'c4zqsfail.rex'
call SysMkDir 'c4zqsfdir'
call lineout 'c4zqsffile', 'text'
call lineout 'c4zqsffile'
say SysSaveRexxMacroSpace('c4zqsfempty') SysFileExists('c4zqsfempty')
say SysSaveRexxMacroSpace('c4zqsffile') charin('c4zqsffile', 1, 4)
call SysAddRexxMacro 'C4ZQSF', 'c4zqsfail.rex'
say SysSaveRexxMacroSpace('c4zqsfnone/saved') SysSaveRexxMacroSpace('c4zqsfdir') SysSaveRexxMacroSpace('c4zqsffile/saved')
say SysFileExists('c4zqsfnone') SysFileExists('c4zqsfdir/saved')
call SysDropRexxMacro 'C4ZQSF'
call SysFileDelete 'c4zqsffile'
call SysFileDelete 'c4zqsfail.rex'
call SysRmDir 'c4zqsfdir'
```

```output
5 0
5 text
5 5 5
0 0
```
