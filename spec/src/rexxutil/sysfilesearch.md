---
element: "SysFileSearch"
kind: function
syntax: "SysFileSearch(target, file, stem, [options])"
rexxref: "Rexx Utilities (RexxUtil) > SysFileSearch"
---

# SysFileSearch

SysFileSearch reads a text file and stores the lines that contain a string in
a stem or an array, in the order of the file, with their line numbers when
asked. It answers 0, or 3 when the file cannot be read.

> **Note.** The examples write the files they search themselves and remove
> them.

## Arguments

r[util.sysfilesearch.args]
`target`, `file` and `stem` are required, and `options` is optional. The call
follows [the argument rule](r:util.index.args): a missing required
argument raises `SYNTAX 88.901`, a fifth argument raises `SYNTAX 88.922`, and
an argument that is an object other than a string is read by its string value,
or raises `SYNTAX 88.909` when it has none. `stem` is the exception to the
last point: an array object or a stem object is the stem itself, and any other
object that has no string value raises `SYNTAX 40.925`, as a name that is not
a stem name does. The null string is a value for `target` and `file`: for
`file` it names no file that can be read, and for `target` it matches no line.

```rexx
call try "sysfilesearch()"
call try "sysfilesearch('a')"
call try "sysfilesearch('a', 'c2zqfs01.txt')"
call try "sysfilesearch(, 'c2zqfs01.txt', 'f.')"
call try "sysfilesearch('a', , 'f.')"
call try "sysfilesearch('a', 'c2zqfs01.txt', , 'N')"
call try "sysfilesearch('a', 'c2zqfs01.txt', 'f.', 'N', 'x')"
call try "sysfilesearch(.nil, 'c2zqfs01.txt', 'f.')"
call try "sysfilesearch('a', .nil, 'f.')"
call try "sysfilesearch('a', 'c2zqfs01.txt', .nil)"
call try "sysfilesearch('a', 'c2zqfs01.txt', 'f.', .nil)"
call try "sysfilesearch('a', 'c2zqfs01.txt', 'a b', 'X')"
call try "sysfilesearch('a', 'c2zqfs01.txt', 'a b', 'N')"
call try "sysfilesearch('a', 'c2zqfs01.txt', 'f.', 'X')"
call try "sysfilesearch('a', 'c2zqfs01.txt', 'f.', 'NX')"
call try "sysfilesearch('a', 'c2zqfs01.txt', 'f.', ' ')"
call try "sysfilesearch('a', 'c2zqfs01.txt', 'f.', 'N ')"
call try "sysfilesearch('a', 'c2zqfs01.txt', 'f.', 'S')"
call try "sysfilesearch('a', 'c2zqfs01.txt', 'f.', 1)"
call try "sysfilesearch('a', 'c2zqfs01.txt', 'f.', '')"
call try "sysfilesearch('a', 'c2zqfs01.txt', 'f.', 'n')"
call try "sysfilesearch('a', 'c2zqfs01.txt', '1.')"
call try "sysfilesearch('a', 'c2zqfs01.txt', '')"
exit
try: procedure
  parse arg ex
  signal on syntax name bad
  interpret 'r = ' ex
  say left(r, 6) ex
  return
bad: say left(condition('o')~code, 6) ex
return
```

```output
88.901 sysfilesearch()
88.901 sysfilesearch('a')
88.901 sysfilesearch('a', 'c2zqfs01.txt')
88.901 sysfilesearch(, 'c2zqfs01.txt', 'f.')
88.901 sysfilesearch('a', , 'f.')
88.901 sysfilesearch('a', 'c2zqfs01.txt', , 'N')
88.922 sysfilesearch('a', 'c2zqfs01.txt', 'f.', 'N', 'x')
88.909 sysfilesearch(.nil, 'c2zqfs01.txt', 'f.')
88.909 sysfilesearch('a', .nil, 'f.')
40.925 sysfilesearch('a', 'c2zqfs01.txt', .nil)
88.909 sysfilesearch('a', 'c2zqfs01.txt', 'f.', .nil)
40.900 sysfilesearch('a', 'c2zqfs01.txt', 'a b', 'X')
40.925 sysfilesearch('a', 'c2zqfs01.txt', 'a b', 'N')
40.900 sysfilesearch('a', 'c2zqfs01.txt', 'f.', 'X')
40.900 sysfilesearch('a', 'c2zqfs01.txt', 'f.', 'NX')
40.900 sysfilesearch('a', 'c2zqfs01.txt', 'f.', ' ')
40.900 sysfilesearch('a', 'c2zqfs01.txt', 'f.', 'N ')
40.900 sysfilesearch('a', 'c2zqfs01.txt', 'f.', 'S')
40.900 sysfilesearch('a', 'c2zqfs01.txt', 'f.', 1)
3      sysfilesearch('a', 'c2zqfs01.txt', 'f.', '')
3      sysfilesearch('a', 'c2zqfs01.txt', 'f.', 'n')
40.925 sysfilesearch('a', 'c2zqfs01.txt', '1.')
40.925 sysfilesearch('a', 'c2zqfs01.txt', '')
```

r[util.sysfilesearch.order]
The arguments are checked in this order: the count and the conversion of the
arguments to strings (`88.9xx`); `options` (`SYNTAX 40.900` for a character
that is not an option letter, a blank included); the name of the stem (`SYNTAX
40.925`, with the names that [SysFileTree](r:util.sysfiletree.stem.name)
rejects). Only then is the file opened, so a stem or an option that is wrong
raises its error whether or not the file exists. An `options` that is the null
string is allowed and means no option.

```rexx
call try "sysfilesearch('a', 'c2zqfs06.none', 'a b', 'X')"
call try "sysfilesearch('a', 'c2zqfs06.none', 'a b', 'N')"
call try "sysfilesearch('a', 'c2zqfs06.none', 'f.', 'X')"
call try "sysfilesearch('a', 'c2zqfs06.none', 'f.', 'N')"
call try "sysfilesearch('a', 'c2zqfs06.none', 'f.', '')"
call try "sysfilesearch('a', 'c2zqfs06.none', 'f.', ' ')"
call try "sysfilesearch('a', 'c2zqfs06.none', .nil, 'X')"
call try "sysfilesearch('a', 'c2zqfs06.none', 'a b', .nil)"
call try "sysfilesearch('a', 'c2zqfs06.none', '')"
exit
try: procedure
  parse arg ex
  signal on syntax name bad
  interpret 'r = ' ex
  say ex '->' r
  return
bad: say ex '->' condition('o')~code
return
```

```output
sysfilesearch('a', 'c2zqfs06.none', 'a b', 'X') -> 40.900
sysfilesearch('a', 'c2zqfs06.none', 'a b', 'N') -> 40.925
sysfilesearch('a', 'c2zqfs06.none', 'f.', 'X') -> 40.900
sysfilesearch('a', 'c2zqfs06.none', 'f.', 'N') -> 3
sysfilesearch('a', 'c2zqfs06.none', 'f.', '') -> 3
sysfilesearch('a', 'c2zqfs06.none', 'f.', ' ') -> 40.900
sysfilesearch('a', 'c2zqfs06.none', .nil, 'X') -> 40.900
sysfilesearch('a', 'c2zqfs06.none', 'a b', .nil) -> 88.909
sysfilesearch('a', 'c2zqfs06.none', '') -> 40.925
```

## What is stored

r[util.sysfilesearch.stem]
The stem is filled as [SysFileTree fills it](r:util.sysfiletree.stem): the
number of lines stored in `.0`, the lines in `.1` to `.n` in the order of the
file, no other tail assigned or dropped, so entries of an earlier call stay
when this call stores fewer, and an array object holds the lines at the
indexes 1 to n and nothing else. A call that stores nothing sets `.0` to 0.

```rexx
d = 'c2zqfs04'
call sysmkdir d
f = d'/in.txt'
call lineout f, 'hit one'
call lineout f, 'miss'
call lineout f, 'hit two'
call lineout f
say sysfilesearch('hit', f, 'f.') f.0 f.1 f.2 symbol('f.3')
f.3 = 'old3'
say sysfilesearch('hit', f, 'f.') f.0 f.3
say sysfilesearch('nothing', f, 'f.') f.0 f.1 f.3
say sysfilesearch('hit', f, 'g') g.0 g.1 symbol('g')
s = .stem~new
say sysfilesearch('hit', f, s, 'N') s[0] s[1] s[2]
a = .array~of('k1', 'k2', 'k3')
say sysfilesearch('hit one', f, a) a~items a[1] a~hasindex(2) a~hasindex(3)
say sysfilesearch('hit', f, a) a~items a[1] a[2]
b = .array~new
say sysfilesearch('nothing', f, b) b~items
call sysfiledelete f
call sysrmdir d
```

```output
0 2 hit one hit two LIT
0 2 old3
0 0 hit one old3
0 2 hit one LIT
0 2 1 hit one 3 hit two
0 1 hit one 0 0
0 2 hit one hit two
0 0
```

r[util.sysfilesearch.match]
A line is stored when `target` occurs in it as a run of characters; there are
no wildcards, and blanks in `target` count. The search ignores case unless the
option `C` is given. The letters `C` and `I` select a case-sensitive and a
case-insensitive search and, when both are given, the last decides; `N` stores
each line as its number in the file, one blank and the line. The letters may
come in any order and in either case ([the option
rule](r:util.index.options)), and a repeated one does no harm. The number is
written without padding, and the lines of the file are counted from 1, empty
ones included, so the numbers of the stored lines need not be consecutive. A
`target` that is the null string, or that contains a line feed, matches no
line.

```rexx
d = 'c2zqfs02'
call sysmkdir d
f = d'/in.txt'
nl = '0A'x
call charout f, 'Alpha one' || nl || 'beta TWO' || nl || 'ALPHA three' || nl || '  gamma alpha  ' || nl || nl || 'a*b a?b'
call charout f
call t 'alpha'
call t 'alpha', 'I'
call t 'alpha', 'C'
call t 'ALPHA', 'C'
call t 'Alpha', 'C'
call t 'alpha', 'N'
call t 'alpha', 'CN'
call t 'alpha', 'NC'
call t 'alpha', 'NI'
call t 'alpha', 'CI'
call t 'alpha', 'IC'
call t 'alpha', 'c'
call t 'alpha', 'cn'
call t 'alpha', 'NN'
call t 'two'
call t 'TWO', 'C'
call t 'two', 'C'
call t 'a*b'
call t 'a?b'
call t 'a.b'
call t 'ph'
call t 'one'||nl||'beta'
call t '  gamma'
call t 'gamma a'
call t 'a '
call t ' '
call t ' ', 'N'
call t 'a', 'N'
call t ''
call t '', 'N'
call t 'zzz', 'N'
call sysfiledelete f
call sysrmdir d
exit
t: procedure expose f
  parse arg target, opts
  if opts == '' then r = sysfilesearch(target, f, 'f.')
  else r = sysfilesearch(target, f, 'f.', opts)
  line = left('['target']['opts']', 14) r f.0
  do i = 1 to f.0
    line = line '<'f.i'>'
  end
  say line
return
```

```output
[alpha][]      0 3 <Alpha one> <ALPHA three> <  gamma alpha  >
[alpha][I]     0 3 <Alpha one> <ALPHA three> <  gamma alpha  >
[alpha][C]     0 1 <  gamma alpha  >
[ALPHA][C]     0 1 <ALPHA three>
[Alpha][C]     0 1 <Alpha one>
[alpha][N]     0 3 <1 Alpha one> <3 ALPHA three> <4   gamma alpha  >
[alpha][CN]    0 1 <4   gamma alpha  >
[alpha][NC]    0 1 <4   gamma alpha  >
[alpha][NI]    0 3 <1 Alpha one> <3 ALPHA three> <4   gamma alpha  >
[alpha][CI]    0 3 <Alpha one> <ALPHA three> <  gamma alpha  >
[alpha][IC]    0 1 <  gamma alpha  >
[alpha][c]     0 1 <  gamma alpha  >
[alpha][cn]    0 1 <4   gamma alpha  >
[alpha][NN]    0 3 <1 Alpha one> <3 ALPHA three> <4   gamma alpha  >
[two][]        0 1 <beta TWO>
[TWO][C]       0 1 <beta TWO>
[two][C]       0 0
[a*b][]        0 1 <a*b a?b>
[a?b][]        0 1 <a*b a?b>
[a.b][]        0 0
[ph][]         0 3 <Alpha one> <ALPHA three> <  gamma alpha  >
[one
beta][]   0 0
[  gamma][]    0 1 <  gamma alpha  >
[gamma a][]    0 1 <  gamma alpha  >
[a ][]         0 4 <Alpha one> <beta TWO> <ALPHA three> <  gamma alpha  >
[ ][]          0 5 <Alpha one> <beta TWO> <ALPHA three> <  gamma alpha  > <a*b a?b>
[ ][N]         0 5 <1 Alpha one> <2 beta TWO> <3 ALPHA three> <4   gamma alpha  > <6 a*b a?b>
[a][N]         0 5 <1 Alpha one> <2 beta TWO> <3 ALPHA three> <4   gamma alpha  > <6 a*b a?b>
[][]           0 0
[][N]          0 0
[zzz][N]       0 0
```

r[util.sysfilesearch.lines]
A line of the file ends at a line feed (`'0A'x`). A carriage return that comes
directly before the line feed is not part of the line; a carriage return
anywhere else, a null character and a `'1A'x` character are. The last line of
the file counts, and is stored, when no line feed follows it, and a line feed
at the end of the file starts no further line. A file with no characters has
no lines.

```rexx
d = 'c2zqfs03'
call sysmkdir d
f = d'/in.txt'
nl = '0A'x
body = ''
do i = 1 to 9
  body = body || 'row' || i || nl
end
body = body || 'row10' || '0D0A'x || 'cr' || '0D'x || 'only' || nl || 'nul' || '00'x || 'byte' || nl
body = body || 'ctl' || '1A'x || 'z' || nl || '0D0D0A'x || 'last row, no end'
call charout f, body
call charout f
call t 'ow1', 'N'
call t '0D'x, 'N'
call t 'cr'
call t 'nul'
call t 'ctl'
call t 'last', 'N'
call t 'only'
call sysfiledelete f
f = d'/endnl.txt'
call charout f, 'one match' || nl || 'two match' || nl
call charout f
call t 'match', 'N'
call sysfiledelete f
f = d'/empty.txt'
call charout f, ''
call charout f
say sysfilesearch('', f, 'f.') f.0 sysfilesearch('x', f, 'f.') f.0
call sysfiledelete f
call sysrmdir d
exit
t: procedure expose f
  parse arg target, opts
  r = sysfilesearch(target, f, 'f.', opts)
  line = left('['show(target)']['opts']', 11) r f.0
  do i = 1 to f.0
    line = line '<'show(f.i)'>'
  end
  say line
return
show: procedure
  parse arg s
  s = changestr('0D'x, s, '<CR>')
  s = changestr('00'x, s, '<NUL>')
  s = changestr('1A'x, s, '<SUB>')
return s
```

```output
[ow1][N]    0 2 <1 row1> <10 row10>
[<CR>][N]   0 2 <11 cr<CR>only> <14 <CR>>
[cr][]      0 1 <cr<CR>only>
[nul][]     0 1 <nul<NUL>byte>
[ctl][]     0 1 <ctl<SUB>z>
[last][N]   0 1 <15 last row, no end>
[only][]    0 1 <cr<CR>only>
[match][N]  0 2 <1 one match> <2 two match>
0 0 0 0
```

## The answer

r[util.sysfilesearch.answer]
The answer is 0 when the file was read, whether or not a line was stored, and
3 when the file could not be opened for reading: a name that no file has, the
null string, a name that is a directory, a path through a file, or a file the
program may not read. In that case `.0` is set to 0 and the stem is otherwise
left as it was; an array object is left with no items. The file name is read
as [every name is](r:util.index.names) and as
[the home rule](r:util.index.home) says ([Deviation
34](r:util.index.deviation.34) says where ooRexx removes a different
component for a `..`).

```rexx
d = 'c2zqfs05'
call sysmkdir d
f = d'/in.txt'
call lineout f, 'hit one'
call lineout f
f.1 = 'stale1'
f.2 = 'stale2'
say sysfilesearch('hit', d'/nofile', 'f.') f.0 f.1 f.2
say sysfilesearch('hit', d, 'f.') f.0
say sysfilesearch('hit', f, 'f.') f.0 f.1
say sysfilesearch('hit', '', 'f.') f.0
say sysfilesearch('hit', d'/nodir/in.txt', 'f.') f.0
say sysfilesearch('hit', d'/in.txt/x', 'f.') f.0
a = .array~of('k1')
say sysfilesearch('hit', d'/nofile', a) a~items
address system 'id -u' with output stem u.
root = (u.1 == 0)
address system 'chmod 000 'f
r = sysfilesearch('hit', f, 'f.', 'N')
say (r == 3 | root) f.0 * (r == 3)
address system 'chmod 644 'f
old = value('HOME', directory()'/'d, 'ENVIRONMENT')
say sysfilesearch('hit', '~/in.txt', 'f.') f.0
call value 'HOME', old, 'ENVIRONMENT'
call sysfiledelete f
call sysrmdir d
```

```output
3 0 stale1 stale2
3 0
0 1 hit one
3 0
3 0
3 0
3 0
1 0
0 1
```
