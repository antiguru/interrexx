---
element: "SysMkDir"
kind: function
syntax: "SysMkDir(name [, mode])"
rexxref: "Rexx Utilities (RexxUtil) > SysMkDir"
---

# SysMkDir

SysMkDir makes a directory. It answers `0` when the directory was made and a
number that is not zero when it was not. The name is read as [the chapter
page](index.md) says for every name.

## Arguments

r[util.sysmkdir.args]
`name` is a required string, and `mode` is an optional number that is read as
[the mode rule](r:util.sysmkdir.mode) says. The count and the omission of
`name`, and an object in its place, follow [the argument
rule](r:util.index.args): no argument raises `SYNTAX 88.901`, a third argument
raises `SYNTAX 88.922`, and `.nil` as the name raises `SYNTAX 88.909`. A name
that is the null string names no directory, so the call answers a number that
is not zero.

```rexx
call try "SysMkDir()"
call try "SysMkDir('c1zqmkx', 493, 'y')"
call try "SysMkDir(.nil)"
call try "SysMkDir('') > 0"
call try "SysMkDir(, 493)"
call try "SysMkDir(.mutablebuffer~new('c1zqmka')) SysIsFileDirectory('c1zqmka')"
call SysRmDir 'c1zqmka'
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
1
88.901
0 1
```

## The result

r[util.sysmkdir.answer]
When no entry has the name, and the directory that is to hold the new one
exists and may be changed, SysMkDir makes the directory, empty, and answers
`0`. It makes only the last component of the name: every component before it
must already be a directory. A slash at the end of the name does not matter,
and the characters `*` and `?` are characters of the name, so that
`c1zqmk*` is made as a directory of that name.

```rexx
say SysMkDir('c1zqmkd') SysIsFileDirectory('c1zqmkd')
say SysMkDir('c1zqmkd/sub/') SysIsFileDirectory('c1zqmkd/sub')
say SysMkDir('c1zqmkd/./sub2') SysMkDir('c1zqmkd/sub/../sub3')
say SysIsFileDirectory('c1zqmkd/sub2') SysIsFileDirectory('c1zqmkd/sub3')
say SysMkDir('c1zqmk*') SysIsFileDirectory('c1zqmk*') SysIsFileDirectory('c1zqmkd')
call SysRmDir 'c1zqmk*'
call SysRmDir 'c1zqmkd/sub3'
call SysRmDir 'c1zqmkd/sub2'
call SysRmDir 'c1zqmkd/sub'
call SysRmDir 'c1zqmkd'
```

```output
0 1
0 1
0 0
1 1
0 1 1
```

r[util.sysmkdir.fail]
SysMkDir answers a number that is not zero, and makes nothing, when an entry
of that name exists, whether it is a directory or a file, so that an existing
directory is left as it was; when a component before the last is missing, or
is a file; and when the system refuses, as it does for a directory that the
program may not change.

```rexx
call SysMkDir 'c1zqmkd'
call lineout 'c1zqmkf', 'x'
call lineout 'c1zqmkf'
say (SysMkDir('c1zqmkd') > 0) SysIsFileDirectory('c1zqmkd')
say (SysMkDir('c1zqmkf') > 0) SysIsFile('c1zqmkf')
say (SysMkDir('c1zqmknone/sub') > 0) SysFileExists('c1zqmknone')
say (SysMkDir('c1zqmkf/sub') > 0) SysFileExists('c1zqmkf/sub')
address system 'chmod 555 c1zqmkd'
say (SysMkDir('c1zqmkd/sub') > 0) SysFileExists('c1zqmkd/sub')
address system 'chmod 755 c1zqmkd'
call SysRmDir 'c1zqmkd'
call SysFileDelete 'c1zqmkf'
```

```output
1 1
1 1
1 0
1 0
1 0
```

r[util.sysmkdir.mode]
`mode` is a whole number from `0` to `511`, written in decimal, that gives the
permission bits of the new directory: `448` is the octal `700`, the owner may
do everything and nobody else anything. When it is omitted it is `511`, which
allows everything. The bits that the process mask forbids are never set, so
the directory has every bit of its permissions that `mode` has and the mask
leaves, and no other: a mode of `0` makes a directory with no permission at
all.

```rexx
call SysMkDir 'c1zqmkm0', 0
call SysMkDir 'c1zqmkm1', 448
call SysMkDir 'c1zqmkm2', 511
call SysMkDir 'c1zqmkm3'
address system 'stat -c %a c1zqmkm0 c1zqmkm1 c1zqmkm2 c1zqmkm3' with output stem s.
say s.1 right(s.2, 2) (s.3 == s.4)
address system 'chmod 755 c1zqmkm0'
call SysRmDir 'c1zqmkm0'
call SysRmDir 'c1zqmkm1'
call SysRmDir 'c1zqmkm2'
call SysRmDir 'c1zqmkm3'
```

```output
0 00 1
```
