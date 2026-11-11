---
element: "SysSaveRexxMacroSpace"
kind: function
deviations: [159]
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
`file` is taken from the current directory, and the name is read as [the rule
of SysAddRexxMacro](r:util.sysaddrexxmacro.answer) reads its `file`: it goes
to the system as it is written. A file that is already there is replaced,
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
file, when the name is that of a directory, and when the file is there and may
not be written. A new file in a directory that may not be written is not
made, and the call answers `5`; a file that is already there and may be
written is replaced as above, also in such a directory.

```rexx
call SysClearRexxMacroSpace
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

> **Note.** The next example shows the two cases of a directory that may not
> be written.

```rexx
address system 'id -u' with output stem u.
root = (u.1 == 0)
call lineout 'c4zqsp.rex', 'return "x"'
call lineout 'c4zqsp.rex'
call SysMkDir 'c4zqspdir'
call lineout 'c4zqspdir/old', 'old'
call lineout 'c4zqspdir/old'
call SysAddRexxMacro 'C4ZQSP', 'c4zqsp.rex'
address system 'chmod 555 c4zqspdir'
say (SysSaveRexxMacroSpace('c4zqspdir/new') == 5 | root) (SysFileExists('c4zqspdir/new') == 0 | root)
say SysSaveRexxMacroSpace('c4zqspdir/old') (charin('c4zqspdir/old', 1, 3) \== 'old')
call stream 'c4zqspdir/old', 'c', 'close'
address system 'chmod 755 c4zqspdir'
call SysDropRexxMacro 'C4ZQSP'
call SysFileDelete 'c4zqspdir/old'
call SysFileDelete 'c4zqspdir/new'
call SysRmDir 'c4zqspdir'
call SysFileDelete 'c4zqsp.rex'
```

```output
1 1
0 1
```

## Deviations

r[util.syssaverexxmacrospace.deviation.159]
A save onto a file that is there and may not be written answers `5` and leaves
the file as it was, with its bytes and its permission. ooRexx also answers
`5`, but whether the file is still there afterwards depends on the calls the
program made earlier: it keeps the file in some histories and removes it in
others. interrexx always keeps it, so the example is not run against ooRexx.

> **Note.** The example shows that the file is still there, with its bytes,
> after the answer. A program that runs as the superuser may write the file,
> so its answer and its file are not looked at there.

```rexx
address system 'id -u' with output stem u.
root = (u.1 == 0)
call lineout 'c4zqsd.rex', 'return "x"'
call lineout 'c4zqsd.rex'
call lineout 'c4zqsdold', 'old'
call lineout 'c4zqsdold'
call SysAddRexxMacro 'C4ZQSD', 'c4zqsd.rex'
address system 'chmod 444 c4zqsdold'
answer = SysSaveRexxMacroSpace('c4zqsdold')
exists = SysFileExists('c4zqsdold')
kept = (charin('c4zqsdold', 1, 3) == 'old')
call stream 'c4zqsdold', 'c', 'close'
say (answer == 5 | root) (exists | root) (kept | root)
address system 'chmod 644 c4zqsdold'
call SysDropRexxMacro 'C4ZQSD'
call SysFileDelete 'c4zqsdold'
call SysFileDelete 'c4zqsd.rex'
```

```output
1 1 1
```
