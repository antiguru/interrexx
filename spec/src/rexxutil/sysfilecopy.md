---
element: "SysFileCopy"
kind: function
syntax: "SysFileCopy(source, target)"
rexxref: "Rexx Utilities (RexxUtil) > SysFileCopy"
---

# SysFileCopy

SysFileCopy copies the bytes of a file to another file. It answers `0` when
the copy was made and a number that is not zero when it was not. Both names
are read as [the chapter page](index.md) says for every name.

## Arguments

r[util.sysfilecopy.args]
`source` and `target` are required strings. The count and the omission of the
arguments, and an object in their place, follow [the argument
rule](r:util.index.args): a call with fewer than two arguments, or with an
omitted one, raises `SYNTAX 88.901`; a third argument raises `SYNTAX 88.922`;
and `.nil` for either raises `SYNTAX 88.909`. A name that is the null string
names no file, so the call answers `2`.

```rexx
call try "SysFileCopy()"
call try "SysFileCopy('c1zqcpa')"
call try "SysFileCopy('c1zqcpa', , )"
call try "SysFileCopy(, 'c1zqcpb')"
call try "SysFileCopy('c1zqcpa', 'c1zqcpb', 'y')"
call try "SysFileCopy(.nil, 'c1zqcpb')"
call try "SysFileCopy('c1zqcpa', .nil)"
call try "SysFileCopy('', 'c1zqcpb')"
call try "SysFileCopy('c1zqcpnone', '')"
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

r[util.sysfilecopy.answer]
When `source` is a file that may be read, and `target` is a name that a file
may be written under, the call writes the bytes of `source` to `target` and
answers `0`. `target` is made when no entry has the name; when a file has it,
its bytes are replaced and none of the old ones are left. A `target` that
names a symbolic link is written through the link: the link stays a link and
the file it points to gets the bytes, and a link that points to nothing has
that file made. A `source` that is a symbolic link is read through the link,
so the copy is a file of its own. The bytes are copied as they are, so a file
with no bytes copies to a file with none, and `source` is not changed. The
names are not patterns.

```rexx
call charout 'c1zqcpa', 'a' || '00ff0d0a'x || 'bc'
call charout 'c1zqcpa'
call charout 'c1zqcpb', 'a long old content'
call charout 'c1zqcpb'
call charout 'c1zqcpe'
address system 'ln -s c1zqcpb c1zqcpl; ln -s c1zqcpnowhere c1zqcpd; ln -s c1zqcpa c1zqcps'
say SysFileCopy('c1zqcpa', 'c1zqcpn') chars('c1zqcpn') c2x(charin('c1zqcpn', 1, 6))
say SysFileCopy('c1zqcpa', 'c1zqcpb') chars('c1zqcpb') chars('c1zqcpa')
say SysFileCopy('c1zqcpa', 'c1zqcpl') SysIsFileLink('c1zqcpl') chars('c1zqcpb')
say SysFileCopy('c1zqcpa', 'c1zqcpd') SysIsFileLink('c1zqcpd') chars('c1zqcpnowhere')
say SysFileCopy('c1zqcps', 'c1zqcpc') SysIsFileLink('c1zqcpc') chars('c1zqcpc')
say SysFileCopy('c1zqcpe', 'c1zqcpf') SysFileExists('c1zqcpf') chars('c1zqcpf')
say SysFileCopy('c1zqcp*', 'c1zqcpg') > 0
call stream 'c1zqcpn', 'c', 'close'
call stream 'c1zqcpb', 'c', 'close'
call stream 'c1zqcpnowhere', 'c', 'close'
call stream 'c1zqcpc', 'c', 'close'
address system 'rm -f c1zqcp?  c1zqcpnowhere'
```

```output
0 7 6100FF0D0A62
0 7 7
0 1 7
0 1 7
0 0 7
0 1 0
1
```

r[util.sysfilecopy.attributes]
A copy has the modification time of its source in whole seconds: a fraction
of a second is dropped. A `target` that the call makes has the permission bits
of its source, whatever the process mask; a `target` that was there keeps its
own. This book does not specify the access time of the copy.

```rexx
call charout 'c1zqcta', 'x'
call charout 'c1zqcta'
call charout 'c1zqctb', 'y'
call charout 'c1zqctb'
address system 'chmod 640 c1zqcta; chmod 600 c1zqctb; touch -d @1000000000.5 c1zqcta'
say SysFileCopy('c1zqcta', 'c1zqctn') SysFileCopy('c1zqcta', 'c1zqctb')
address system 'stat -c "%a %.1Y" c1zqcta c1zqctn c1zqctb'
address system 'rm -f c1zqcta c1zqctb c1zqctn'
```

```output
0 0
640 1000000000.5
640 1000000000.0
600 1000000000.0
```

r[util.sysfilecopy.fail]
SysFileCopy answers a number that is not zero when it cannot copy. When
`source` names an entry and `target` names the same one, under the same name,
under another spelling of its name, such as `./` before it, or through a
symbolic link, the call answers `17` and changes nothing. Otherwise `source`
is checked first: the call answers `2` when `source` names nothing and `13`
when the program may not read it, and a `target` that is not there is not made
then. `target` is checked next: the call answers `2` when the directory of
`target` does not exist, `21` when `target` is a directory, and `13` when
`target` is a file that may not be written or a name in a directory that may
not be changed. A `source` that is a directory answers `21` only when none of
these checks fails.

```rexx
call charout 'c1zqcfa', 'AAA'
call charout 'c1zqcfa'
call charout 'c1zqcfb', 'BBBBBB'
call charout 'c1zqcfb'
call SysMkDir 'c1zqcfd'
address system 'ln -s c1zqcfa c1zqcfl'
say SysFileCopy('c1zqcfnone', 'c1zqcfn') SysFileExists('c1zqcfn')
say SysFileCopy('c1zqcfa', 'c1zqcfnone/n') SysFileExists('c1zqcfnone')
say SysFileCopy('c1zqcfa', 'c1zqcfd') SysIsFileDirectory('c1zqcfd') SysFileExists('c1zqcfd/c1zqcfa')
say SysFileCopy('c1zqcfa', 'c1zqcfa') SysFileCopy('c1zqcfa', './c1zqcfa') SysFileCopy('c1zqcfa', 'c1zqcfl') SysFileCopy('c1zqcfnone', 'c1zqcfnone')
say chars('c1zqcfa')
call stream 'c1zqcfa', 'c', 'close'
address system 'chmod 000 c1zqcfa'
say SysFileCopy('c1zqcfa', 'c1zqcfn') SysFileExists('c1zqcfn')
address system 'chmod 644 c1zqcfa; chmod 444 c1zqcfb'
say SysFileCopy('c1zqcfa', 'c1zqcfb') chars('c1zqcfb')
call stream 'c1zqcfb', 'c', 'close'
address system 'chmod 555 c1zqcfd'
say SysFileCopy('c1zqcfa', 'c1zqcfd/n') SysFileExists('c1zqcfd/n')
address system 'chmod 755 c1zqcfd; chmod 644 c1zqcfb; rm -f c1zqcfa c1zqcfb c1zqcfl'
call SysRmDir 'c1zqcfd'
call SysMkDir 'c1zqcod'
call charout 'c1zqcob', 'BBBB'
call charout 'c1zqcob'
call charout 'c1zqcou', 'UU'
call charout 'c1zqcou'
address system 'chmod 444 c1zqcob; chmod 000 c1zqcou'
say SysFileCopy('c1zqconone', 'c1zqcod') SysFileCopy('c1zqcou', 'c1zqconone/n') SysFileCopy('c1zqcou', 'c1zqcod')
say SysFileCopy('c1zqcod', 'c1zqconone/n') SysFileCopy('c1zqcod', 'c1zqcob')
address system 'chmod 644 c1zqcob c1zqcou; rm -f c1zqcob c1zqcou'
call SysRmDir 'c1zqcod'
```

```output
2 0
2 0
21 1 0
17 17 17 2
3
13 0
13 6
13 0
2 13 13
2 13
```
