---
element: "SysFileMove"
kind: function
syntax: "SysFileMove(source, target)"
rexxref: "Rexx Utilities (RexxUtil) > SysFileMove"
---

# SysFileMove

SysFileMove gives a file another name. It answers `0` when the file was moved
and a number that is not zero when it was not. Both names are read as [the
chapter page](index.md) says for every name.

## Arguments

r[util.sysfilemove.args]
`source` and `target` are required strings. The count and the omission of the
arguments, and an object in their place, follow [the argument
rule](r:util.index.args): a call with fewer than two arguments, or with an
omitted one, raises `SYNTAX 88.901`; a third argument raises `SYNTAX 88.922`;
and `.nil` for either raises `SYNTAX 88.909`. A name that is the null string
names no file, so the call answers `2`.

```rexx
call try "SysFileMove()"
call try "SysFileMove('c1zqmva')"
call try "SysFileMove('c1zqmva', , )"
call try "SysFileMove(, 'c1zqmvb')"
call try "SysFileMove('c1zqmva', 'c1zqmvb', 'y')"
call try "SysFileMove(.nil, 'c1zqmvb')"
call try "SysFileMove('c1zqmva', .nil)"
call try "SysFileMove('', 'c1zqmvb')"
call try "SysFileMove('c1zqmvnone', '')"
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
88.922
88.909
88.909
2
2
```

## The result

r[util.sysfilemove.answer]
When `source` is the name of an entry that exists and no entry has the name
`target`, in a directory that exists and may be changed, the entry gets the
name `target`, `source` names nothing any more, and the call answers `0`. The
bytes, the permission bits and the times of the file are those it had; a
symbolic link is moved as a link and not read through, so a link that points
to nothing can be moved. A directory can be moved to a name that no entry has
in the same file system. This book does not specify a move from one file
system to another. The names are not patterns.

```rexx
call charout 'c1zqmva', 'AAA'
call charout 'c1zqmva'
call SysMkDir 'c1zqmvd'
call charout 'c1zqmvd/in', 'I'
call charout 'c1zqmvd/in'
address system 'ln -s c1zqmva c1zqmvl; ln -s c1zqmvnowhere c1zqmvb; chmod 640 c1zqmva; touch -d @1000000000 c1zqmva'
say SysFileMove('c1zqmva', 'c1zqmvn') SysFileExists('c1zqmva') chars('c1zqmvn')
address system 'stat -c "%a %Y" c1zqmvn'
say SysFileMove('c1zqmvl', 'c1zqmvl2') SysIsFileLink('c1zqmvl2') SysIsFileLink('c1zqmvl')
say SysFileMove('c1zqmvb', 'c1zqmvb2') SysIsFileLink('c1zqmvb2') SysIsFileLink('c1zqmvb')
say SysFileMove('c1zqmvd', 'c1zqmvd2') SysIsFileDirectory('c1zqmvd') SysIsFile('c1zqmvd2/in')
say SysFileMove('c1zqmv*', 'c1zqmvw') > 0
call stream 'c1zqmvn', 'c', 'close'
address system 'rm -f c1zqmvn c1zqmvl2 c1zqmvb2 c1zqmvd2/in'
call SysRmDir 'c1zqmvd2'
```

```output
0 0 3
640 1000000000
0 1 0
0 1 0
0 0 1
1
```

r[util.sysfilemove.replace]
Some entries at `target` are replaced by the move: a symbolic link that points
to nothing, or to a directory, when `source` is not a directory, and a
directory with no entries when `source` is a directory. The link itself, or
the empty directory, goes, and what a link pointed to is not changed.

```rexx
call charout 'c1zqmra', 'AAA'
call charout 'c1zqmra'
call charout 'c1zqmrb', 'BB'
call charout 'c1zqmrb'
call SysMkDir 'c1zqmrd'
call SysMkDir 'c1zqmre'
call SysMkDir 'c1zqmrf'
call SysMkDir 'c1zqmrf/in'
call SysMkDir 'c1zqmrg'
address system 'ln -s c1zqmrnowhere c1zqmrl; ln -s c1zqmrg c1zqmrk'
say SysFileMove('c1zqmra', 'c1zqmrl') SysIsFileLink('c1zqmrl') chars('c1zqmrl')
say SysFileMove('c1zqmrb', 'c1zqmrk') SysIsFileLink('c1zqmrk') chars('c1zqmrk') SysIsFileDirectory('c1zqmrg')
say SysFileMove('c1zqmrd', 'c1zqmre') SysIsFileDirectory('c1zqmrd') SysIsFileDirectory('c1zqmre')
say SysFileMove('c1zqmre', 'c1zqmrf') SysFileMove('c1zqmre', 'c1zqmrl') SysFileMove('c1zqmre', 'c1zqmre/sub')
say SysIsFileDirectory('c1zqmre') SysIsFileDirectory('c1zqmrf/in')
call stream 'c1zqmrl', 'c', 'close'
call stream 'c1zqmrk', 'c', 'close'
call SysFileDelete 'c1zqmrl'
call SysFileDelete 'c1zqmrk'
call SysRmDir 'c1zqmre'
call SysRmDir 'c1zqmrf/in'
call SysRmDir 'c1zqmrf'
call SysRmDir 'c1zqmrg'
```

```output
0 0 3
0 0 2 1
0 0 1
39 17 22
1 1
```

r[util.sysfilemove.fail]
SysFileMove answers a number that is not zero when it cannot move, and nothing
changes: neither name is changed and the files keep their bytes. It answers
`17` when `source` names an entry and `target` names the same one, under the
same name, under another name or through a symbolic link; and when `target`,
with symbolic links followed, names an entry that exists and is not a
directory, whether `source` exists or not, since a move does not replace a
file. Otherwise it answers `2` when `source` names nothing or the directory of
`target` does not exist, `21` when a file is moved to a directory, `39` when a
directory is moved to a directory that has entries, `20` when a directory is
moved to a symbolic link, `22` when a directory is moved into itself, and `13`
when the system refuses, as it does for a directory that may not be changed.

```rexx
call charout 'c1zqmfa', 'AAA'
call charout 'c1zqmfa'
call charout 'c1zqmfb', 'BBBBBB'
call charout 'c1zqmfb'
call SysMkDir 'c1zqmfd'
say SysFileMove('c1zqmfnone', 'c1zqmfn') SysFileExists('c1zqmfn') SysFileMove('c1zqmfnone', 'c1zqmfb')
say SysFileMove('c1zqmfa', 'c1zqmfb') chars('c1zqmfa') chars('c1zqmfb')
say SysFileMove('c1zqmfa', 'c1zqmfa') SysFileMove('c1zqmfa', './c1zqmfa') chars('c1zqmfa')
say SysFileMove('c1zqmfa', 'c1zqmfd') SysIsFile('c1zqmfa') SysIsFileDirectory('c1zqmfd')
say SysFileMove('c1zqmfa', 'c1zqmfd/') SysFileExists('c1zqmfd/c1zqmfa')
say SysFileMove('c1zqmfa', 'c1zqmfnone/a') SysIsFile('c1zqmfa')
address system 'chmod 555 c1zqmfd'
say SysFileMove('c1zqmfa', 'c1zqmfd/a') SysIsFile('c1zqmfa') SysFileExists('c1zqmfd/a')
address system 'chmod 755 c1zqmfd'
call stream 'c1zqmfa', 'c', 'close'
call stream 'c1zqmfb', 'c', 'close'
call SysFileDelete 'c1zqmfa'
call SysFileDelete 'c1zqmfb'
call SysRmDir 'c1zqmfd'
```

```output
2 0 17
17 3 6
17 17 3
21 1 1
21 0
2 1
13 1 0
```
