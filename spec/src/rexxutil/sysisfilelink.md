---
element: "SysIsFileLink"
kind: function
syntax: "SysIsFileLink(name)"
rexxref: "Rexx Utilities (RexxUtil) > SysIsFileLink"
---

# SysIsFileLink

SysIsFileLink answers `1` when the name is that of a symbolic link, and `0`
when it is not. The name is read as [the chapter page](index.md) says for
every name.

## Arguments

r[util.sysisfilelink.args]
`name` is a required string. The count and the omission of the argument, and
an object in its place, follow [the argument rule](r:util.index.args): no
argument raises `SYNTAX 88.901`, a second argument raises `SYNTAX 88.922`, and
`.nil` raises `SYNTAX 88.909`. A name that is the null string is not that of a
link, and the answer is `0`.

```rexx
call try "SysIsFileLink()"
call try "SysIsFileLink('c1zqilx', 'y')"
call try "SysIsFileLink(.nil)"
call try "SysIsFileLink('')"
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

r[util.sysisfilelink.answer]
The answer is `1` for the name of a symbolic link, whatever the link points
to: a file, a directory, another link or nothing. The link itself is looked
at, not its target, so the answer is `0` for the name of the target and for
any other entry that is not a link, and a link to a link is a link. A name
with a `..` component is cleaned up first, as for every name. The call never
raises a condition for a name.

```rexx
call lineout 'c1zqilf', 'x'
call lineout 'c1zqilf'
call SysMkDir 'c1zqild'
address system 'ln -s c1zqilf c1zqill; ln -s c1zqild c1zqilk; ln -s c1zqilnowhere c1zqilb; ln -s c1zqill c1zqilm'
say SysIsFileLink('c1zqill') SysIsFileLink('c1zqilk') SysIsFileLink('c1zqilb')
say SysIsFileLink('c1zqilm') SysIsFileLink('c1zqilk/') SysIsFileLink('c1zqild/../c1zqill')
say SysIsFileLink('c1zqilf') SysIsFileLink('c1zqild') SysIsFileLink('c1zqilnone')
address system 'rm c1zqill c1zqilk c1zqilb c1zqilm c1zqilf'
call SysRmDir 'c1zqild'
```

```output
1 1 1
1 1 1
0 0 0
```
