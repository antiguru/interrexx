---
element: "SysIsFile"
kind: function
syntax: "SysIsFile(name)"
rexxref: "Rexx Utilities (RexxUtil) > SysIsFile"
---

# SysIsFile

SysIsFile answers `1` when the name is that of a regular file, and `0` when it
is not. The name is read as [the chapter page](index.md) says for every name.

## Arguments

r[util.sysisfile.args]
`name` is a required string. The count and the omission of the argument, and
an object in its place, follow [the argument rule](r:util.index.args): no
argument raises `SYNTAX 88.901`, a second argument raises `SYNTAX 88.922`, and
`.nil` raises `SYNTAX 88.909`. A name that is the null string is not that of a
file, and the answer is `0`.

```rexx
call try "SysIsFile()"
call try "SysIsFile('c1zqifx', 'y')"
call try "SysIsFile(.nil)"
call try "SysIsFile('')"
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
0
```

## The result

r[util.sysisfile.answer]
The answer is `1` for a regular file, and for a symbolic link whose target is
a regular file. It is `0` for a directory, a fifo and a character device such
as `/dev/null`, for a name that nothing has, for a symbolic link that points
to nothing, and for a name that the program may not look up. The call never
raises a condition for a name. This book does not specify the answer for a
block device.

```rexx
call lineout 'c1zqiff', 'x'
call lineout 'c1zqiff'
call SysMkDir 'c1zqifd'
address system 'ln -s c1zqiff c1zqifl; ln -s c1zqifd c1zqifk; ln -s c1zqifnowhere c1zqifb; mkfifo c1zqifp'
say SysIsFile('c1zqiff') SysIsFile('c1zqifl') SysIsFile('c1zqiff/')
say SysIsFile('c1zqifd') SysIsFile('c1zqifk') SysIsFile('c1zqifp')
say SysIsFile('c1zqifb') SysIsFile('c1zqifnone') SysIsFile('/dev/null')
address system 'rm c1zqifl c1zqifk c1zqifb c1zqifp c1zqiff'
call SysRmDir 'c1zqifd'
```

```output
1 1 1
0 0 0
0 0 0
```
