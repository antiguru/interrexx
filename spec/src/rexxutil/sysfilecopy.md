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
names no file, so the call answers a number that is not zero.

```rexx
call try "SysFileCopy()"
call try "SysFileCopy('c1zqcpa')"
call try "SysFileCopy('c1zqcpa', , )"
call try "SysFileCopy(, 'c1zqcpb')"
call try "SysFileCopy('c1zqcpa', 'c1zqcpb', 'y')"
call try "SysFileCopy(.nil, 'c1zqcpb')"
call try "SysFileCopy('c1zqcpa', .nil)"
call try "SysFileCopy('', 'c1zqcpb') > 0"
call try "SysFileCopy('c1zqcpnone', '') > 0"
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
A copy has the modification time of its source, to the second. A `target` that
the call makes has the permission bits of its source; a `target` that was
there keeps its own.

```rexx
call charout 'c1zqcta', 'x'
call charout 'c1zqcta'
call charout 'c1zqctb', 'y'
call charout 'c1zqctb'
address system 'chmod 640 c1zqcta; chmod 600 c1zqctb; touch -d @1000000000 c1zqcta'
say SysFileCopy('c1zqcta', 'c1zqctn') SysFileCopy('c1zqcta', 'c1zqctb')
address system 'stat -c "%a %Y" c1zqcta c1zqctn c1zqctb'
address system 'rm -f c1zqcta c1zqctb c1zqctn'
```

```output
0 0
640 1000000000
640 1000000000
600 1000000000
```

r[util.sysfilecopy.fail]
SysFileCopy answers a number that is not zero when it cannot copy. That is so
when `source` does not name a file, when it is a directory, and when the
program may not read it; and a `target` that is not there is not made when
`source` names nothing or may not be read. It is also so when the directory of
`target` does not exist or may not be changed, when `target` is a directory,
when `target` is a file that may not be written, and when `target` is the same
file as `source`, under the same name, under another name or through a link.
A file that a failed copy found at `target` keeps its bytes.

```rexx
call charout 'c1zqcfa', 'AAA'
call charout 'c1zqcfa'
call charout 'c1zqcfb', 'BBBBBB'
call charout 'c1zqcfb'
call SysMkDir 'c1zqcfd'
address system 'ln -s c1zqcfa c1zqcfl'
say (SysFileCopy('c1zqcfnone', 'c1zqcfn') > 0) SysFileExists('c1zqcfn')
say (SysFileCopy('c1zqcfd', 'c1zqcfd2') > 0)
say (SysFileCopy('c1zqcfa', 'c1zqcfnone/n') > 0) SysFileExists('c1zqcfnone')
say (SysFileCopy('c1zqcfa', 'c1zqcfd') > 0) SysIsFileDirectory('c1zqcfd') SysFileExists('c1zqcfd/c1zqcfa')
say (SysFileCopy('c1zqcfa', 'c1zqcfa') > 0) (SysFileCopy('c1zqcfa', './c1zqcfa') > 0) (SysFileCopy('c1zqcfa', 'c1zqcfl') > 0)
say chars('c1zqcfa')
call stream 'c1zqcfa', 'c', 'close'
address system 'chmod 000 c1zqcfa'
say (SysFileCopy('c1zqcfa', 'c1zqcfn') > 0) SysFileExists('c1zqcfn')
address system 'chmod 644 c1zqcfa; chmod 444 c1zqcfb'
say (SysFileCopy('c1zqcfa', 'c1zqcfb') > 0) chars('c1zqcfb')
call stream 'c1zqcfb', 'c', 'close'
address system 'chmod 644 c1zqcfb; rm -f c1zqcfa c1zqcfb c1zqcfl c1zqcfd2'
call SysRmDir 'c1zqcfd'
```

```output
1 0
1
1 0
1 1 0
1 1 1
3
1 0
1 6
```
