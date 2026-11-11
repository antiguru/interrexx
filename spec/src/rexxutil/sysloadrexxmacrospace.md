---
element: "SysLoadRexxMacroSpace"
kind: function
syntax: "SysLoadRexxMacroSpace(file)"
rexxref: "Rexx Utilities (RexxUtil) > SysLoadRexxMacroSpace"
---

# SysLoadRexxMacroSpace

SysLoadRexxMacroSpace reads a file that
[SysSaveRexxMacroSpace](syssaverexxmacrospace.md) wrote and puts the macros
that it holds into the [macrospace](macrospace.md). It answers `0` when the
macros are in, and `5` when the file could not be used.

## Arguments

r[util.sysloadrexxmacrospace.args]
`file` is a required string. The count and the omission of the argument, and
an object in its place, follow [the argument rule](r:util.index.args): no
argument raises `SYNTAX 88.901`, a second argument raises `SYNTAX 88.922`, and
`.nil` raises `SYNTAX 88.909`. An object that is not a string but has a string
value is read by that value. A `file` that is the null string names no file
that can be read, so the call answers `5`.

```rexx
call try "SysLoadRexxMacroSpace()"
call try "SysLoadRexxMacroSpace(, )"
call try "SysLoadRexxMacroSpace('c4zqloadnone', 'x')"
call try "SysLoadRexxMacroSpace(.nil)"
call try "SysLoadRexxMacroSpace('')"
call try "SysLoadRexxMacroSpace(.mutablebuffer~new('c4zqloadnone'))"
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
5
```

## The result

r[util.sysloadrexxmacrospace.answer]
When the file is one that SysSaveRexxMacroSpace wrote, the call answers `0`
and every macro that the file holds is in the macrospace, with the order and
the program that it had when it was saved. A relative `file` is taken from the
current directory, and the name is read as [the rule of
SysAddRexxMacro](r:util.sysaddrexxmacro.answer) reads its `file`: it goes to
the system as it is written. The macros that the file does not hold stay as
they were; a macro of the same name (in any case) as one of the file is
replaced by the one of the file, with its order. A macro that was loaded is a
macro like any other ([the macrospace page](r:util.macrospace.run)), and it
does not need the file it was first made from. Loading the same file twice
leaves the macrospace as the first load did.

```rexx
call lineout 'c4zqld1.rex', 'return "one"'
call lineout 'c4zqld1.rex'
call lineout 'c4zqld2.rex', 'return "two"'
call lineout 'c4zqld2.rex'
call lineout 'c4zqld3.rex', 'return "three"'
call lineout 'c4zqld3.rex'
call SysAddRexxMacro 'C4ZQLD1', 'c4zqld1.rex', 'A'
call SysAddRexxMacro 'C4ZQLD2', 'c4zqld2.rex', 'B'
call SysMkDir 'c4zqlddir'
call SysSaveRexxMacroSpace 'c4zqlddir/saved'
call SysClearRexxMacroSpace
call SysFileDelete 'c4zqld1.rex'
call SysFileDelete 'c4zqld2.rex'
say SysLoadRexxMacroSpace('c4zqlddir/saved') SysQueryRexxMacro('C4ZQLD1') SysQueryRexxMacro('C4ZQLD2') C4ZQLD1() C4ZQLD2()
say SysLoadRexxMacroSpace('./c4zqlddir/saved') SysQueryRexxMacro('C4ZQLD1') SysQueryRexxMacro('C4ZQLD2')
call SysAddRexxMacro 'C4ZQLD1', 'c4zqld3.rex', 'B'
call SysAddRexxMacro 'C4ZQLD3', 'c4zqld3.rex', 'B'
call SysReorderRexxMacro 'C4ZQLD2', 'A'
say SysQueryRexxMacro('C4ZQLD1') SysQueryRexxMacro('C4ZQLD2') SysQueryRexxMacro('C4ZQLD3') C4ZQLD1()
say SysLoadRexxMacroSpace('c4zqlddir/saved') SysQueryRexxMacro('C4ZQLD1') SysQueryRexxMacro('C4ZQLD2') SysQueryRexxMacro('C4ZQLD3') C4ZQLD1() C4ZQLD3()
call SysDropRexxMacro 'C4ZQLD1'
call SysDropRexxMacro 'C4ZQLD2'
call SysDropRexxMacro 'C4ZQLD3'
call SysFileDelete 'c4zqlddir/saved'
call SysRmDir 'c4zqlddir'
call SysFileDelete 'c4zqld3.rex'
```

```output
0 A B one two
0 A B
B A B three
0 A B B one three
```

r[util.sysloadrexxmacrospace.fail]
The call answers `5`, and adds nothing, when no file has the name, when the
name is that of a directory, when the file is empty, and when a file that
SysSaveRexxMacroSpace did not write is shorter than 32 bytes. Such a file of
32 bytes or more, and a saved file whose first bytes were changed, answer `6`,
and nothing is added. A saved file with bytes added after its end loads as if
they were not there. A saved file that was cut short answers `5`: when it held
one macro nothing is added, and when it held several, the macros that the cut
left whole may have been added; which ones is not specified, as the form of
the file is not. A call that answers `5` or `6` changes no macro that was
already in the macrospace, except that a macro it adds from a cut file
replaces one of the same name, as a load that answers `0` does.

```rexx
call lineout 'c4zqlf.rex', 'return "x"'
call lineout 'c4zqlf.rex'
call lineout 'c4zqlftext', 'this is text'
call lineout 'c4zqlftext'
address system 'touch c4zqlfempty'
call SysMkDir 'c4zqlfdir'
say SysLoadRexxMacroSpace('c4zqlfnone') SysLoadRexxMacroSpace('c4zqlfdir') SysLoadRexxMacroSpace('.')
say SysLoadRexxMacroSpace('c4zqlftext') SysLoadRexxMacroSpace('c4zqlfempty') SysLoadRexxMacroSpace('c4zqlf.rex') chars('c4zqlfempty')
call SysAddRexxMacro 'C4ZQLF', 'c4zqlf.rex'
say SysLoadRexxMacroSpace('c4zqlftext') SysQueryRexxMacro('C4ZQLF') C4ZQLF()
call SysDropRexxMacro 'C4ZQLF'
call SysFileDelete 'c4zqlf.rex'
call SysFileDelete 'c4zqlftext'
call SysFileDelete 'c4zqlfempty'
call SysRmDir 'c4zqlfdir'
```

```output
5 5 5
5 5 5 0
5 B x
```

> **Note.** The next example saves a file of one macro, cuts it short in two
> ways, adds bytes after its end, changes its first byte, and loads a file of
> 31 bytes and one of 32. Only the load of the file with bytes added changes
> macro `C4ZQDM`; after each load, the example adds it again with order `B`.

```rexx
call SysClearRexxMacroSpace
call lineout 'c4zqdm1.rex', 'return "one"'
call lineout 'c4zqdm1.rex'
call SysAddRexxMacro 'C4ZQDM', 'c4zqdm1.rex', 'A'
call SysSaveRexxMacroSpace 'c4zqdmsaved'
call SysAddRexxMacro 'C4ZQDM', 'c4zqdm1.rex', 'B'
whole = charin('c4zqdmsaved', 1, chars('c4zqdmsaved'))
call stream 'c4zqdmsaved', 'c', 'close'
call try left(whole, length(whole) - 1)
call try left(whole, length(whole) % 2)
call try whole || 'more'
call try 'X' || substr(whole, 2)
call try copies('a', 31)
call try copies('a', 32)
call SysDropRexxMacro 'C4ZQDM'
call SysFileDelete 'c4zqdmsaved'
call SysFileDelete 'c4zqdm1.rex'
exit
try:
  call charout 'c4zqdm', arg(1)
  call stream 'c4zqdm', 'c', 'close'
  say SysLoadRexxMacroSpace('c4zqdm') SysQueryRexxMacro('C4ZQDM')
  call SysAddRexxMacro 'C4ZQDM', 'c4zqdm1.rex', 'B'
  call SysFileDelete 'c4zqdm'
  return
```

```output
5 B
5 B
0 A
6 B
5 B
6 B
```
