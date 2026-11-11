---
element: "SysMkDir"
kind: function
syntax: "SysMkDir(name [, mode])"
rexxref: "Rexx Utilities (RexxUtil) > SysMkDir"
---

# SysMkDir

SysMkDir makes a directory. It answers `0` when the directory was made and a
number that is not zero when it was not. The name is read as [the chapter
page](index.md) says for every name.

## Arguments

r[util.sysmkdir.args]
`name` is a required string, and `mode` is an optional number that is read as
[the mode rule](r:util.sysmkdir.mode) says. The count and the omission of
`name`, and an object in its place, follow [the argument
rule](r:util.index.args): no argument raises `SYNTAX 88.901`, a third argument
raises `SYNTAX 88.922`, and `.nil` as the name raises `SYNTAX 88.909`. A name
that is the null string names no directory, so the call answers `2`.

```rexx
call try "SysMkDir()"
call try "SysMkDir('c1zqmkx', 493, 'y')"
call try "SysMkDir(.nil)"
call try "SysMkDir('')"
call try "SysMkDir(, 493)"
call try "SysMkDir(.mutablebuffer~new('c1zqmka')) SysIsFileDirectory('c1zqmka')"
call SysRmDir 'c1zqmka'
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
88.901
0 1
```

## The result

r[util.sysmkdir.answer]
When no entry has the name, and the directory that is to hold the new one
exists and may be changed, SysMkDir makes the directory, empty, and answers
`0`. It makes only the last component of the name: every component before it
must already be a directory. A slash at the end of the name does not matter,
and the characters `*` and `?` are characters of the name, so that
`c1zqmk*` is made as a directory of that name.

```rexx
say SysMkDir('c1zqmkd') SysIsFileDirectory('c1zqmkd')
say SysMkDir('c1zqmkd/sub/') SysIsFileDirectory('c1zqmkd/sub')
say SysMkDir('c1zqmkd/./sub2') SysMkDir('c1zqmkd/sub/../sub3')
say SysIsFileDirectory('c1zqmkd/sub2') SysIsFileDirectory('c1zqmkd/sub3')
say SysMkDir('c1zqmk*') SysIsFileDirectory('c1zqmk*') SysIsFileDirectory('c1zqmkd')
call SysRmDir 'c1zqmk*'
call SysRmDir 'c1zqmkd/sub3'
call SysRmDir 'c1zqmkd/sub2'
call SysRmDir 'c1zqmkd/sub'
call SysRmDir 'c1zqmkd'
```

```output
0 1
0 1
0 0
1 1
0 1 1
```

r[util.sysmkdir.fail]
SysMkDir makes nothing and answers `17` when an entry of that name exists,
whether it is a directory, a file or a symbolic link, even one that points to
nothing, so that an existing directory is left as it was; `2` when a component
before the last is missing; `20` when one is a file; and `13` when the system
refuses, as it does for a directory that the program may not change.

```rexx
call SysMkDir 'c1zqmkd'
call lineout 'c1zqmkf', 'x'
call lineout 'c1zqmkf'
address system 'ln -s c1zqmknowhere c1zqmkl'
say SysMkDir('c1zqmkd') SysIsFileDirectory('c1zqmkd')
say SysMkDir('c1zqmkf') SysIsFile('c1zqmkf') SysMkDir('c1zqmkl') SysFileExists('c1zqmknowhere')
say SysMkDir('c1zqmknone/sub') SysFileExists('c1zqmknone')
say SysMkDir('c1zqmkf/sub') SysFileExists('c1zqmkf/sub')
address system 'chmod 555 c1zqmkd'
say SysMkDir('c1zqmkd/sub') SysFileExists('c1zqmkd/sub')
address system 'chmod 755 c1zqmkd'
address system 'rm c1zqmkl'
call SysRmDir 'c1zqmkd'
call SysFileDelete 'c1zqmkf'
```

```output
17 1
17 1 17 0
2 0
20 0
13 0
```

r[util.sysmkdir.mode]
`mode` is a whole number in the [form](r:conv.whole.form) that the conventions
give. Its value is its number rounded to 20 significant digits, whatever the
NUMERIC DIGITS setting, as for [BEEP](r:bif.beep.range), and that value must
be from `-2147483648` to `2147483647`. A `mode` that is not in that form, the
null string, a value with a fractional part other than zero, a value out of
that range, and an object whose string fails these checks, `.nil` among them,
raise `SYNTAX 88.907`. An object other than a string is first replaced by the
string that its `request` with the argument `STRING` answers, and when that
answer is not a string, by the answer of its `STRING` method; an object with a
`STRING` method that answers `'0'` is therefore the mode `0`. This differs from
`name`, which has no such fallback ([the argument rule](r:util.sysmkdir.args)).
When `mode` is omitted it is `511`.

```rexx
say try('x') try('') try(1.5) try(.nil) try(.object~new)
say try('2147483648') try('-2147483649') try('7.5')
say try('1e2') try(' + 7 ') try('7.000') try('-2147483648') try('2147483647')
exit
try:
  signal on syntax name e
  r = SysMkDir('c1zqmkt', arg(1))
  call SysRmDir 'c1zqmkt'
  return r
e: return condition('o')~code
```

```output defect=#229
88.907 88.907 88.907 88.907 88.907
88.907 88.907 88.907
0 0 0 0 0
```

```rexx
say try(.s0~new) try(.sx~new) try(.r0~new) try(.m0~new)
exit
try:
  signal on syntax name e
  r = SysMkDir('c1zqmku', arg(1))
  call SysRmDir 'c1zqmku'
  return r
e: return condition('o')~code
::class s0
::method string
  return '0'
::class sx
::method string
  return 'x'
::class r0
::method request
  use arg kind
  if kind == 'STRING' then return '0'
  return .nil
::class m0
::method makestring
  return '0'
```

```output defect=#230
0 88.907 0 0
```

r[util.sysmkdir.bits]
The permission bits of the new directory come from the remainder of `mode`
divided by `1024`, taken as zero or positive, so that `-1` gives `1023`. The
lowest nine bits of the remainder are the permission bits (`448` is the octal
`700`: the owner may do everything and nobody else anything), and its bit of
`512` is the sticky bit; the higher bits of `mode` are dropped. A permission
bit that the process mask forbids is never set, so the directory has the bits
that the remainder has and the mask leaves, and no other: a mode of `0` makes
a directory with no permission at all.

```rexx
call SysMkDir 'c1zqmkm0', 0
call SysMkDir 'c1zqmkm1', 448
call SysMkDir 'c1zqmkm2', 511
call SysMkDir 'c1zqmkm3'
address system 'stat -c %a c1zqmkm0 c1zqmkm1 c1zqmkm2 c1zqmkm3' with output stem s.
say s.1 right(s.2, 2) (s.3 == s.4)
address system 'chmod 755 c1zqmkm0'
call SysRmDir 'c1zqmkm0'
call SysRmDir 'c1zqmkm1'
call SysRmDir 'c1zqmkm2'
call SysRmDir 'c1zqmkm3'
```

```output
0 00 1
```

```rexx
call SysMkDir 'c1zqmkb0', 512
call SysMkDir 'c1zqmkb1', -1
call SysMkDir 'c1zqmkb2', 1024 + 448
call SysMkDir 'c1zqmkb3', 1000
call SysMkDir 'c1zqmkb4', '-2147483648'
address system 'stat -c %a c1zqmkb0 c1zqmkb1 c1zqmkb2 c1zqmkb3 c1zqmkb4' with output stem s.
say s.1 left(s.2, 2) s.3 left(s.4, 2) s.5
do i = 0 to 4
  address system 'chmod 755 c1zqmkb'i
  call SysRmDir 'c1zqmkb'i
end
```

```output
1000 17 700 17 0
```
