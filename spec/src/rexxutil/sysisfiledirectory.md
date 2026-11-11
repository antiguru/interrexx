---
element: "SysIsFileDirectory"
kind: function
syntax: "SysIsFileDirectory(name)"
rexxref: "Rexx Utilities (RexxUtil) > SysIsFileDirectory"
---

# SysIsFileDirectory

SysIsFileDirectory answers `1` when the name is that of a directory, and `0`
when it is not. The name is read as [the chapter page](index.md) says for
every name.

## Arguments

r[util.sysisfiledirectory.args]
`name` is a required string. The count and the omission of the argument, and
an object in its place, follow [the argument rule](r:util.index.args): no
argument raises `SYNTAX 88.901`, a second argument raises `SYNTAX 88.922`, and
`.nil` raises `SYNTAX 88.909`. A name that is the null string is not that of a
directory, and the answer is `0`.

```rexx
call try "SysIsFileDirectory()"
call try "SysIsFileDirectory('c1zqidx', 'y')"
call try "SysIsFileDirectory(.nil)"
call try "SysIsFileDirectory('')"
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

r[util.sysisfiledirectory.answer]
The answer is `1` for a directory, and for a symbolic link whose target is a
directory. It is `0` for every other kind of entry, for a name that nothing
has, for a symbolic link that points to nothing, and for a name that the
program may not look up. The names `.` and `..` and the root `/` are
directories. The call never raises a condition for a name.

```rexx
call lineout 'c1zqidf', 'x'
call lineout 'c1zqidf'
call SysMkDir 'c1zqidd'
address system 'ln -s c1zqidd c1zqidl; ln -s c1zqidf c1zqidk; ln -s c1zqidnowhere c1zqidb'
say SysIsFileDirectory('c1zqidd') SysIsFileDirectory('c1zqidd/') SysIsFileDirectory('c1zqidl')
say SysIsFileDirectory('c1zqidf') SysIsFileDirectory('c1zqidk')
say SysIsFileDirectory('c1zqidb') SysIsFileDirectory('c1zqidnone')
say SysIsFileDirectory('.') SysIsFileDirectory('..') SysIsFileDirectory('/')
address system 'rm c1zqidl c1zqidk c1zqidb c1zqidf'
call SysRmDir 'c1zqidd'
```

```output
1 1 1
0 0
0 0
1 1 1
```
