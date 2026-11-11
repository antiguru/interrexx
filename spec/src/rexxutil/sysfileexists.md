---
element: "SysFileExists"
kind: function
syntax: "SysFileExists(name)"
rexxref: "Rexx Utilities (RexxUtil) > SysFileExists"
---

# SysFileExists

SysFileExists answers `1` when something has the name, and `0` when nothing
does. The name is read as [the chapter page](index.md) says for every name.

## Arguments

r[util.sysfileexists.args]
`name` is a required string. The count and the omission of the argument, and
an object in its place, follow [the argument rule](r:util.index.args): no
argument raises `SYNTAX 88.901`, a second argument raises `SYNTAX 88.922`, and
`.nil` raises `SYNTAX 88.909`. A name that is the null string names nothing,
and the answer for it is `0`.

```rexx
call try "SysFileExists()"
call try "SysFileExists('c1zqfex', 'y')"
call try "SysFileExists(.nil)"
call try "SysFileExists('')"
call try "SysFileExists(.object~new)"
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
88.909
```

## The result

r[util.sysfileexists.answer]
The answer is `1` for a name that is that of a file of any kind: a regular
file, a directory, a fifo or a device, and a symbolic link whose target
exists. It is `0` for a name that nothing has, for a symbolic link that points
to nothing, and for a name that the program may not look up, such as one in
a directory it may not search. The call never raises a condition for a name,
whatever the name is.

```rexx
call lineout 'c1zqfef', 'x'
call lineout 'c1zqfef'
call SysMkDir 'c1zqfed'
address system 'ln -s c1zqfef c1zqfel; ln -s c1zqfenowhere c1zqfeb; mkfifo c1zqfep'
say SysFileExists('c1zqfef') SysFileExists('c1zqfed') SysFileExists('c1zqfep')
say SysFileExists('c1zqfel') SysFileExists('c1zqfeb') SysFileExists('c1zqfenone')
say SysFileExists('/dev/null') SysFileExists('/')
address system 'chmod 000 c1zqfed'
say SysFileExists('c1zqfed') SysFileExists('c1zqfed/x')
address system 'chmod 755 c1zqfed; rm c1zqfel c1zqfeb c1zqfep c1zqfef'
call SysRmDir 'c1zqfed'
```

```output
1 1 1
1 0 0
1 1
1 0
```
