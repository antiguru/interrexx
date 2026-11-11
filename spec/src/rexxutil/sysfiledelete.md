---
element: "SysFileDelete"
kind: function
syntax: "SysFileDelete(name)"
rexxref: "Rexx Utilities (RexxUtil) > SysFileDelete"
---

# SysFileDelete

SysFileDelete removes a file. It answers `0` when the file is gone and a
number that is not zero when it could not remove it. The name is read as
[the chapter page](index.md) says for every name.

## Arguments

r[util.sysfiledelete.args]
`name` is a required string. The count and the omission of the argument, and
an object in its place, follow [the argument rule](r:util.index.args): a call
with no argument raises `SYNTAX 88.901`, a second argument raises
`SYNTAX 88.922`, and `.nil` raises `SYNTAX 88.909`. A name that is the null
string names no file, so the call answers a number that is not zero.

```rexx
call try "SysFileDelete()"
call try "SysFileDelete('c1zqfdx', 'y')"
call try "SysFileDelete(.nil)"
call try "SysFileDelete('') > 0"
call try "SysFileDelete(.mutablebuffer~new('c1zqfdnone')) > 0"
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
1
```

## The result

r[util.sysfiledelete.answer]
When the name is that of a file that exists and the directory that holds it
may be changed, the file is removed and the call answers `0`. The name of a
symbolic link removes the link and leaves what it points to. A name of a
fifo is removed the same way.
The call removes one entry: a name with a `*` or a `?` in it is not a pattern.

```rexx
call lineout 'c1zqfdf', 'x'
call lineout 'c1zqfdf'
call lineout 'c1zqfdg', 'x'
call lineout 'c1zqfdg'
address system 'ln -s c1zqfdf c1zqfdl'
say SysFileDelete('c1zqfdl') SysIsFileLink('c1zqfdl') SysIsFile('c1zqfdf')
say (SysFileDelete('c1zqfd*') > 0) SysIsFile('c1zqfdf') SysIsFile('c1zqfdg')
say SysFileDelete('c1zqfdf') SysIsFile('c1zqfdf')
say SysFileDelete('c1zqfdg') SysFileExists('c1zqfdg')
```

```output
0 0 1
1 1 1
0 0
0 0
```

r[util.sysfiledelete.fail]
The call answers a number that is not zero, and removes nothing, when no file
has the name, including a name in a directory that does not exist and a
symbolic link that points to nothing; when the name is that of a directory,
which SysFileDelete does not remove; and when the system refuses the removal,
as it does for a file in a directory that the program may not change.

```rexx
call SysMkDir 'c1zqfdd'
call lineout 'c1zqfdd/in', 'x'
call lineout 'c1zqfdd/in'
address system 'ln -s c1zqfdnowhere c1zqfdl'
say (SysFileDelete('c1zqfdnone') > 0) (SysFileDelete('c1zqfdnone/in') > 0)
say (SysFileDelete('c1zqfdl') > 0) SysIsFileLink('c1zqfdl')
say (SysFileDelete('c1zqfdd') > 0) SysIsFileDirectory('c1zqfdd')
address system 'chmod 555 c1zqfdd'
say (SysFileDelete('c1zqfdd/in') > 0) SysIsFile('c1zqfdd/in')
address system 'chmod 755 c1zqfdd'
say SysFileDelete('c1zqfdd/in') SysIsFile('c1zqfdd/in')
address system 'rm c1zqfdl'
call SysRmDir 'c1zqfdd'
```

```output
1 1
1 1
1 1
1 1
0 0
```
