---
element: "SysRmDir"
kind: function
syntax: "SysRmDir(name)"
rexxref: "Rexx Utilities (RexxUtil) > SysRmDir"
---

# SysRmDir

SysRmDir removes a directory that has nothing in it. It answers `0` when the
directory is gone and a number that is not zero when it is not. The name is
read as [the chapter page](index.md) says for every name.

## Arguments

r[util.sysrmdir.args]
`name` is a required string. The count and the omission of the argument, and
an object in its place, follow [the argument rule](r:util.index.args): no
argument raises `SYNTAX 88.901`, a second argument raises `SYNTAX 88.922`, and
`.nil` raises `SYNTAX 88.909`. A name that is the null string names no
directory, so the call answers `2`.

```rexx
call try "SysRmDir()"
call try "SysRmDir('c1zqrmx', 'y')"
call try "SysRmDir(.nil)"
call try "SysRmDir('')"
call try "SysRmDir(.mutablebuffer~new('c1zqrmnone'))"
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
2
2
```

## The result

r[util.sysrmdir.answer]
When the name is that of a directory with no entry in it, and the directory
that holds it may be changed, the directory is removed and the call answers
`0`. A slash at the end of the name does not matter. The call removes one
entry: the characters `*` and `?` are characters of the name.

```rexx
call SysMkDir 'c1zqrmd'
call SysMkDir 'c1zqrmd/sub'
call SysMkDir 'c1zqrm*'
say SysRmDir('c1zqrmd/sub/') SysIsFileDirectory('c1zqrmd/sub') SysIsFileDirectory('c1zqrmd')
say SysRmDir('c1zqrm*') SysIsFileDirectory('c1zqrm*') SysIsFileDirectory('c1zqrmd')
say SysRmDir('c1zqrmd') SysIsFileDirectory('c1zqrmd')
```

```output
0 0 1
0 0 1
0 0
```

r[util.sysrmdir.fail]
SysRmDir removes nothing and answers `2` when no entry has the name, `20` when
a component before the last is a file, `39` when the directory has an entry in
it, even one that is a directory again, and `13` when the system refuses, as
it does for a directory in a directory that the program may not change.

```rexx
call SysMkDir 'c1zqrmd'
call SysMkDir 'c1zqrmd/sub'
call lineout 'c1zqrmf', 'x'
call lineout 'c1zqrmf'
say SysRmDir('c1zqrmnone') SysRmDir('c1zqrmnone/sub') SysRmDir('c1zqrmf/sub')
say SysRmDir('c1zqrmd') SysIsFileDirectory('c1zqrmd/sub')
address system 'chmod 555 c1zqrmd'
say SysRmDir('c1zqrmd/sub') SysIsFileDirectory('c1zqrmd/sub')
address system 'chmod 755 c1zqrmd'
say SysRmDir('c1zqrmd/sub') SysRmDir('c1zqrmd')
call SysFileDelete 'c1zqrmf'
```

```output
2 2 20
39 1
13 1
0 0
```

r[util.sysrmdir.entry]
SysRmDir does not look at what kind of entry the name is: a name that is not
that of a directory is removed too, so a file, a fifo or a symbolic link goes
the way a directory does. A symbolic link to a directory is removed, and the
directory it points to stays with what it holds.

```rexx
call SysMkDir 'c1zqrmd'
call lineout 'c1zqrmd/in', 'x'
call lineout 'c1zqrmd/in'
call lineout 'c1zqrmf', 'x'
call lineout 'c1zqrmf'
address system 'ln -s c1zqrmd c1zqrml; mkfifo c1zqrmp'
say SysRmDir('c1zqrml') SysIsFileLink('c1zqrml') SysIsFile('c1zqrmd/in')
say SysRmDir('c1zqrmf') SysFileExists('c1zqrmf')
say SysRmDir('c1zqrmp') SysFileExists('c1zqrmp')
call SysFileDelete 'c1zqrmd/in'
call SysRmDir 'c1zqrmd'
```

```output
0 0 1
0 0
0 0
```
