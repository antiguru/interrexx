---
element: "SysAddRexxMacro"
kind: function
syntax: "SysAddRexxMacro(name, file [, order])"
rexxref: "Rexx Utilities (RexxUtil) > SysAddRexxMacro"
---

# SysAddRexxMacro

SysAddRexxMacro reads a Rexx program from a file and puts it into the
[macrospace](macrospace.md) under a name, as a macro that a call of that name
runs. It answers `0` when the macro is in, and a number that is not zero when
it is not.

## Arguments

r[util.sysaddrexxmacro.args]
`name` and `file` are required strings, and `order` is optional. The count and
the omission of an argument, and an object in the place of one, follow [the
argument rule](r:util.index.args): a call with no `name` or no `file`, or with
either one omitted, raises `SYNTAX 88.901`, a fourth argument raises `SYNTAX
88.922`, and `.nil` raises `SYNTAX 88.909`. An object that is not a string but
has a string value is read by that value. The rules for what a `name` is are
on [the page of the macrospace](r:util.macrospace.names). A `file` that is the
null string names no file that can be read, so the call answers `7`. When more
than one check fails, the count and the omission come first, then the `order`
(the next rule), and only then is the file read, so that a bad `order` is
reported for a file that is not there.

```rexx
call lineout 'c4zqargs.rex', 'return "x"'
call lineout 'c4zqargs.rex'
call try "SysAddRexxMacro()"
call try "SysAddRexxMacro('C4ZQA')"
call try "SysAddRexxMacro('C4ZQA',)"
call try "SysAddRexxMacro(,'c4zqargs.rex')"
call try "SysAddRexxMacro(,'c4zqargs.rex','x')"
call try "SysAddRexxMacro('C4ZQA','c4zqargs.rex','B','x')"
call try "SysAddRexxMacro(.nil,'c4zqargs.rex')"
call try "SysAddRexxMacro('C4ZQA',.nil)"
call try "SysAddRexxMacro('C4ZQA','c4zqargs.rex',.nil)"
call try "SysAddRexxMacro('C4ZQA','')"
call try "SysAddRexxMacro('C4ZQA','c4zqnone.rex','x')"
call try "SysAddRexxMacro('C4ZQA','c4zqnone.rex')"
call try "SysAddRexxMacro(.mutablebuffer~new('c4zqa'),.mutablebuffer~new('c4zqargs.rex'))"
say SysQueryRexxMacro('C4ZQA')
call SysDropRexxMacro 'C4ZQA'
call SysFileDelete 'c4zqargs.rex'
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
88.901
88.922
88.909
88.909
88.909
7
40.920
7
0
B
```

r[util.sysaddrexxmacro.order]
`order` says where a call looks for the macro among the other places it
searches, as [the search order rule](r:env.search.order) states: `B` puts the
macro before the files and the registered functions, and `A` after the files.
An `order` that is omitted, at the end or in its place, is `B`. Only the first
character of the string is read: it is `A` or `B` in either case, and the
characters after it do not matter. The null string, a string that starts with
a blank, and any other first character raise `SYNTAX 40.920`. When the macro
already exists, the order of this call replaces its order.
[SysQueryRexxMacro](sysqueryrexxmacro.md) answers the order that a macro has,
as `A` or `B`.

```rexx
call lineout 'c4zqord.rex', 'return "x"'
call lineout 'c4zqord.rex'
call try "'c4zqord.rex'"
call try "'c4zqord.rex', 'a'"
call try "'c4zqord.rex', 'B'"
call try "'c4zqord.rex', 'after'"
call try "'c4zqord.rex', 'BEFORE'"
call try "'c4zqord.rex', 'Ab'"
call try "'c4zqord.rex', 'B '"
call try "'c4zqord.rex', "
call try "'c4zqord.rex', ''"
call try "'c4zqord.rex', ' B'"
call try "'c4zqord.rex', 'C'"
call try "'c4zqord.rex', '0'"
call SysDropRexxMacro 'C4ZQO'
call SysFileDelete 'c4zqord.rex'
exit
try:
  signal on syntax name e
  r = SysAddRexxMacro('C4ZQO', 'c4zqord.rex')
  interpret 'r = SysAddRexxMacro("C4ZQO",' arg(1) ')'
  say arg(1) '->' r SysQueryRexxMacro('C4ZQO')
  return
e: say arg(1) '->' condition('o')~code
  return
```

```output
'c4zqord.rex' -> 0 B
'c4zqord.rex', 'a' -> 0 A
'c4zqord.rex', 'B' -> 0 B
'c4zqord.rex', 'after' -> 0 A
'c4zqord.rex', 'BEFORE' -> 0 B
'c4zqord.rex', 'Ab' -> 0 A
'c4zqord.rex', 'B ' -> 0 B
'c4zqord.rex',  -> 0 B
'c4zqord.rex', '' -> 40.920
'c4zqord.rex', ' B' -> 40.920
'c4zqord.rex', 'C' -> 40.920
'c4zqord.rex', '0' -> 40.920
```

## The result

r[util.sysaddrexxmacro.answer]
When the file can be read and holds a Rexx program, the call answers `0`. The
program is read at that moment, and what the macro keeps is the program as it
was read ([the source rule](r:util.macrospace.source)). A relative `file` is
taken from the current directory at the moment of the call. The name goes to
the system as it is written, with none of the changes of [the name
rule](r:util.index.names) or [the home rule](r:util.index.home): a `~` at its
start is a character of the name, a component of two periods is the parent of
the directory that the components before it reach, so that those must name a
directory that exists (and the parent of a symbolic link is that of its
target), and a slash at the end names no file. SysSaveRexxMacroSpace and
SysLoadRexxMacroSpace read their `file` the same way. When a macro of the same
name (in any case) is there, the new one replaces it. The new macro takes the
`order` of the call, and not that of the macro it replaces.

```rexx
call lineout 'c4zqadd1.rex', 'return "first"'
call lineout 'c4zqadd1.rex'
call SysMkDir 'c4zqadddir'
call lineout 'c4zqadddir/c4zqadd2.rex', 'return "second"'
call lineout 'c4zqadddir/c4zqadd2.rex'
here = directory()
say SysAddRexxMacro('C4ZQADD', 'c4zqadd1.rex', 'A') SysQueryRexxMacro('C4ZQADD') C4ZQADD()
say SysAddRexxMacro('c4zqadd', './c4zqadddir/../c4zqadddir/c4zqadd2.rex') SysQueryRexxMacro('C4ZQADD') C4ZQADD()
call directory 'c4zqadddir'
say SysAddRexxMacro('C4ZQADD', 'c4zqadd2.rex', 'A') SysAddRexxMacro('C4ZQADE', here || '/c4zqadd1.rex')
call directory here
say SysQueryRexxMacro('C4ZQADD') C4ZQADD() SysQueryRexxMacro('C4ZQADE') C4ZQADE()
call SysDropRexxMacro 'C4ZQADD'
call SysDropRexxMacro 'C4ZQADE'
call SysFileDelete 'c4zqadddir/c4zqadd2.rex'
call SysRmDir 'c4zqadddir'
call SysFileDelete 'c4zqadd1.rex'
```

```output
0 A first
0 B second
0 0
A second B first
```

> **Note.** In the next example, `HOME` names a directory that holds a program
> of its own. A name that starts with `~/`, one with a `..` after a directory
> that is not there, and one with a slash at the end are not found, and the
> `~` of a name for a file to save is a directory of the name, which is not
> there.

```rexx
call SysMkDir 'c4zqhome'
call lineout 'c4zqhome/c4zqh.rex', 'return "home"'
call lineout 'c4zqhome/c4zqh.rex'
call lineout 'c4zqh.rex', 'return "here"'
call lineout 'c4zqh.rex'
old = value('HOME', directory() || '/c4zqhome', 'ENVIRONMENT')
say SysAddRexxMacro('C4ZQH', '~/c4zqh.rex') SysAddRexxMacro('C4ZQH', 'c4zqnone/../c4zqh.rex') SysAddRexxMacro('C4ZQH', 'c4zqh.rex/')
say SysAddRexxMacro('C4ZQH', 'c4zqh.rex') SysSaveRexxMacroSpace('~/c4zqhs') SysSaveRexxMacroSpace('c4zqnone/../c4zqhs')
say SysFileExists('c4zqhome/c4zqhs') SysFileExists('c4zqhs')
say SysSaveRexxMacroSpace('c4zqhome/c4zqhs') SysLoadRexxMacroSpace('~/c4zqhs') SysLoadRexxMacroSpace('c4zqhome/c4zqhs/')
call value 'HOME', old, 'ENVIRONMENT'
call SysDropRexxMacro 'C4ZQH'
call SysFileDelete 'c4zqhome/c4zqhs'
call SysFileDelete 'c4zqhome/c4zqh.rex'
call SysRmDir 'c4zqhome'
call SysFileDelete 'c4zqhs'
call SysFileDelete 'c4zqh.rex'
```

```output defect=#232
7 7 7
0 5 5
0 0
0 5 5
```

r[util.sysaddrexxmacro.fail]
The call answers `7`, and changes nothing, when it cannot make a macro of the
file. That is so when no file has the name, when the name is that of a
directory, when the file cannot be read (the permission is refused), when the
file is empty (has no bytes at all; a file of one empty line is a program that
does nothing), and when the file is not a valid Rexx program: an error in its
text, or in one of its directives, that is found while it is read. A macro of
the same name that was there is left as it was, with its order and its
program. Faults that appear only when the program runs are not found here: a
call to a routine that nothing has, and a `::REQUIRES` for a file that is not
there, add a macro without a fault.

```rexx
call lineout 'c4zqf0.rex', 'return "kept"'
call lineout 'c4zqf0.rex'
call lineout 'c4zqf1.rex', 'say 1 +'
call lineout 'c4zqf1.rex'
call lineout 'c4zqf2.rex', 'return 1'
call lineout 'c4zqf2.rex', '::routine'
call lineout 'c4zqf2.rex'
call lineout 'c4zqf3.rex', 'return 1'
call lineout 'c4zqf3.rex', '::class'
call lineout 'c4zqf3.rex'
call lineout 'c4zqf4.rex', 'if then'
call lineout 'c4zqf4.rex'
call lineout 'c4zqf5.rex', 'return c4zqnofunction()'
call lineout 'c4zqf5.rex'
call lineout 'c4zqf6.rex', 'return 1'
call lineout 'c4zqf6.rex', '::requires "c4zqnofile.rex"'
call lineout 'c4zqf6.rex'
call lineout 'c4zqf7.rex', 'return "unreadable"'
call lineout 'c4zqf7.rex'
call SysMkDir 'c4zqfdir'
address system 'touch c4zqf8.rex'
address system 'id -u' with output stem u.
root = (u.1 == 0)
address system 'chmod 000 c4zqf7.rex'
say SysAddRexxMacro('C4ZQF', 'c4zqf0.rex', 'A')
say SysAddRexxMacro('C4ZQF', 'c4zqfnone.rex', 'B') SysAddRexxMacro('C4ZQF', 'c4zqfdir', 'B')
say SysAddRexxMacro('C4ZQF', 'c4zqf1.rex', 'B') SysAddRexxMacro('C4ZQF', 'c4zqf2.rex', 'B')
say SysAddRexxMacro('C4ZQF', 'c4zqf3.rex', 'B') SysAddRexxMacro('C4ZQF', 'c4zqf4.rex', 'B')
say (SysAddRexxMacro('C4ZQF', 'c4zqf7.rex', 'B') == 7 | root) SysAddRexxMacro('C4ZQF', 'c4zqf8.rex', 'B')
say SysQueryRexxMacro('C4ZQF') C4ZQF()
say SysAddRexxMacro('C4ZQF5', 'c4zqf5.rex') SysAddRexxMacro('C4ZQF6', 'c4zqf6.rex')
say SysQueryRexxMacro('C4ZQF5') SysQueryRexxMacro('C4ZQF6')
call SysDropRexxMacro 'C4ZQF'
call SysDropRexxMacro 'C4ZQF5'
call SysDropRexxMacro 'C4ZQF6'
address system 'chmod 644 c4zqf7.rex'
do n = 0 to 8
  call SysFileDelete 'c4zqf' || n || '.rex'
end
call SysRmDir 'c4zqfdir'
```

```output
0
7 7
7 7
7 7
1 7
A kept
0 0
B B
```
