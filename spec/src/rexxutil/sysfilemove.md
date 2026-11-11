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
names no file, so the call answers a number that is not zero.

```rexx
call try "SysFileMove()"
call try "SysFileMove('c1zqmva')"
call try "SysFileMove('c1zqmva', , )"
call try "SysFileMove(, 'c1zqmvb')"
call try "SysFileMove('c1zqmva', 'c1zqmvb', 'y')"
call try "SysFileMove(.nil, 'c1zqmvb')"
call try "SysFileMove('c1zqmva', .nil)"
call try "SysFileMove('', 'c1zqmvb') > 0"
call try "SysFileMove('c1zqmvnone', '') > 0"
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
1
1
```

## The result

r[util.sysfilemove.answer]
When `source` is the name of an entry that exists and no entry has the name
`target`, in a directory that exists and may be changed, the entry gets the
name `target`, `source` names nothing any more, and the call answers `0`. The
bytes, the permission bits and the times of the file are those it had; a
symbolic link is moved as a link and not read through, so a link that points
to nothing can be moved. A directory can be moved to a name that no entry has.
The names are not patterns.

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

r[util.sysfilemove.fail]
SysFileMove answers a number that is not zero when it cannot move, and
nothing changes: neither name is changed and the files keep their bytes. That
is so when `source` names nothing, when `target` is the name of a file that
exists, which a move does not replace, and so when it is `source` itself, when
`target` is a directory that a file is moved to, when the directory of
`target` does not exist, and when the system refuses, as it does for a
directory that may not be changed.

```rexx
call charout 'c1zqmfa', 'AAA'
call charout 'c1zqmfa'
call charout 'c1zqmfb', 'BBBBBB'
call charout 'c1zqmfb'
call SysMkDir 'c1zqmfd'
say (SysFileMove('c1zqmfnone', 'c1zqmfn') > 0) SysFileExists('c1zqmfn')
say (SysFileMove('c1zqmfa', 'c1zqmfb') > 0) chars('c1zqmfa') chars('c1zqmfb')
say (SysFileMove('c1zqmfa', 'c1zqmfa') > 0) (SysFileMove('c1zqmfa', './c1zqmfa') > 0) chars('c1zqmfa')
say (SysFileMove('c1zqmfa', 'c1zqmfd') > 0) SysIsFile('c1zqmfa') SysIsFileDirectory('c1zqmfd')
say (SysFileMove('c1zqmfa', 'c1zqmfd/') > 0) SysFileExists('c1zqmfd/c1zqmfa')
say (SysFileMove('c1zqmfa', 'c1zqmfnone/a') > 0) SysIsFile('c1zqmfa')
address system 'chmod 555 c1zqmfd'
say (SysFileMove('c1zqmfa', 'c1zqmfd/a') > 0) SysIsFile('c1zqmfa') SysFileExists('c1zqmfd/a')
address system 'chmod 755 c1zqmfd'
call stream 'c1zqmfa', 'c', 'close'
call stream 'c1zqmfb', 'c', 'close'
call SysFileDelete 'c1zqmfa'
call SysFileDelete 'c1zqmfb'
call SysRmDir 'c1zqmfd'
```

```output
1 0
1 3 6
1 1 3
1 1 1
1 0
1 1
1 1 0
```
