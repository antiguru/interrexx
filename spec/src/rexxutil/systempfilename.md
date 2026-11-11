---
element: "SysTempFileName"
kind: function
syntax: "SysTempFileName(template [, filler])"
rexxref: "Rexx Utilities (RexxUtil) > SysTempFileName"
---

# SysTempFileName

SysTempFileName invents the name of a file that does not exist, from a
template in which some characters stand for digits. It makes nothing: it only
finds a name. The template is read as [the chapter page](index.md) says for
every name, except that the filler characters are counted in it as it is
written.

## Arguments

r[util.systempfilename.args]
`template` is a required string and `filler` is an optional string. The count
and the omission of `template`, and an object in its place, follow [the
argument rule](r:util.index.args): no argument raises `SYNTAX 88.901`, a third
argument raises `SYNTAX 88.922`, and `.nil` for either raises `SYNTAX 88.909`.
`filler` is one character, and it is `?` when it is omitted; a `filler` of
the null string, or of more than one character, raises `SYNTAX 40.23`.

```rexx
call try "SysTempFileName()"
call try "SysTempFileName('c1zqtfx?', '?', 'y')"
call try "SysTempFileName(.nil)"
call try "SysTempFileName('c1zqtfx?', .nil)"
call try "SysTempFileName('c1zqtfx?', '')"
call try "SysTempFileName('c1zqtfx?', '??')"
call try "SysTempFileName(, '?')"
call try "SysTempFileName('c1zqtfx?', ) \= ''"
call try "SysTempFileName('c1zqtfx?', '?') \= ''"
call try "SysTempFileName(.mutablebuffer~new('c1zqtfx?'), .mutablebuffer~new('?')) \= ''"
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
88.922
88.909
88.909
40.23
40.23
88.901
1
1
1
```

## The result

r[util.systempfilename.answer]
Every `filler` character of `template` is replaced by a digit, chosen at
random, and the answer is the full name of the result: when the template is
relative, the name is taken from the current directory, and it is cleaned up
as [for every name](r:util.index.names). A `filler` in the directory part of
the template is replaced as well, and the directory need not exist. Which
digits are chosen is not specified, and two calls with one template can answer
different names. The call makes no file, so a name that it answers is free
until a program takes it.

```rexx
here = directory()
p = here || '/c1zqtf'
n = SysTempFileName('c1zqtf???')
parse var n (p) d
say digits(d, 3) SysFileExists(n) SysIsFile(n)
n = SysTempFileName('./c1zqtfd/../c1zqtf?.x')
parse var n (p) d '.x' rest
say digits(d, 1) '[' || rest || ']'
n = SysTempFileName('c1zqtfQQ', 'Q')
parse var n (p) d
say digits(d, 2)
n = SysTempFileName('c1zqtf?Q', 'Q')
parse var n (p) '?' d
say digits(d, 1)
n = SysTempFileName('c1zqtf   ', ' ')
parse var n (p) d
say digits(d, 3)
n = SysTempFileName('c1zqtfaQ/f??', 'Q')
parse var n (p) 'a' d '/f??' rest
say digits(d, 1) '[' || rest || ']'
n = SysTempFileName('c1zqtfaQ/fQQ', 'Q')
parse var n (p) 'a' d '/f' e
say digits(d, 1) digits(e, 2)
exit
digits: procedure
  parse arg s, k
  return length(s) == k & verify(s, '0123456789') == 0
```

```output
1 0 0
1 []
1
1
1
1 []
1 1
```

r[util.systempfilename.count]
A `template` has one to nine `filler` characters, counting the ones in the
directory part. A `template` with none, or with more than nine, has no
answer: the call answers the null string.

```rexx
say '[' || SysTempFileName('c1zqtfn') || ']' '[' || SysTempFileName('c1zqtfn/') || ']'
say '[' || SysTempFileName('c1zqtfn', 'Q') || ']' '[' || SysTempFileName('') || ']'
say (SysTempFileName('c1zqtfQ', 'Q') \= '') (SysTempFileName('c1zqtfQQQQQQQQQ', 'Q') \= '')
say '[' || SysTempFileName('c1zqtfQQQQQQQQQQ', 'Q') || ']'
say (SysTempFileName('QQQQQ/c1zqtfQQQQ', 'Q') \= '') '[' || SysTempFileName('QQQQQ/c1zqtfQQQQQ', 'Q') || ']'
```

```output
[] []
[] []
1 1
[]
1 []
```

r[util.systempfilename.free]
The answer is a name for which [SysFileExists](r:util.sysfileexists.answer)
answers `0`. When the digits that were chosen make a name for which it answers
`1`, other digits are tried, and the call answers the null string only when
every name that the `template` can make is such a name. A name is taken when a
file, a directory or any other entry has it, through a symbolic link too. A
symbolic link that points to nothing does not take its name, so the answer can
be the name of such a link.

```rexx
here = directory()
call SysMkDir 'c1zqtfd'
do i = 0 to 8
  call lineout 'c1zqtfd/t' || i, 'x'
  call lineout 'c1zqtfd/t' || i
end
say SysTempFileName('c1zqtfd/t?') == here || '/c1zqtfd/t9'
say SysTempFileName('./c1zqtfd/../c1zqtfd/t?') == here || '/c1zqtfd/t9'
say SysTempFileName('c1zqtfd/tQ', 'Q') == here || '/c1zqtfd/t9'
call SysMkDir 'c1zqtfd/t9'
say '[' || SysTempFileName('c1zqtfd/t?') || ']'
call SysRmDir 'c1zqtfd/t9'
say SysTempFileName('c1zqtfd/t?') == here || '/c1zqtfd/t9'
do i = 0 to 8
  call SysFileDelete 'c1zqtfd/t' || i
end
call SysRmDir 'c1zqtfd'
```

```output
1
1
1
[]
1
```

```rexx
here = directory()
call SysMkDir 'c1zqtfd'
call SysMkDir 'c1zqtfd/t9'
do i = 0 to 8
  address system 'ln -s c1zqtfnowhere c1zqtfd/t' || i
end
n = SysTempFileName('c1zqtfd/t?')
say (left(n, length(here) + 10) == here || '/c1zqtfd/t') SysIsFileLink(n) SysFileExists(n)
do i = 0 to 8
  address system 'ln -sfn .. c1zqtfd/t' || i
end
say '[' || SysTempFileName('c1zqtfd/t?') || ']'
do i = 0 to 8
  call SysFileDelete 'c1zqtfd/t' || i
end
call SysRmDir 'c1zqtfd/t9'
call SysRmDir 'c1zqtfd'
```

```output
1 1 0
[]
```
