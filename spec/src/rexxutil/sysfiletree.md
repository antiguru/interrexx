---
element: "SysFileTree"
kind: function
syntax: "SysFileTree(filespec, stem, [options], [tattrib], [nattrib])"
rexxref: "Rexx Utilities (RexxUtil) > SysFileTree"
---

# SysFileTree

SysFileTree looks for files and directories whose names match a pattern and
stores what it finds in a stem or an array: one text line per match, and in
the stem a count. It answers 0.

The rules here are those of a Unix-like system. The attribute arguments exist
for the sake of other systems and have no effect here.

> **Note.** Times, the sizes of directories, the permission bits a new file
> gets and the order in which matches are stored belong to the host or to the
> file system. The examples build a tree of their own, give its files fixed
> times with [SysSetFileDateTime](syssetfiledatetime.md), print matches in a
> sorted order and show host-dependent values only by their form.

## Arguments

r[util.sysfiletree.args]
`filespec` and `stem` are required; `options`, `tattrib` and `nattrib` are
optional. The call follows the argument rule (rule `util.index.args`): a
required argument that is missing raises `SYNTAX 88.901`, a sixth argument
raises `SYNTAX 88.922`, and an argument that is an object other than a string
is read by its string value, or raises `SYNTAX 88.909` when it has none. The
`stem` argument is the exception to the last point: an array object or a stem
object is the stem itself, [as the next rule](r:util.sysfiletree.stem) says,
and any other object that has no string value raises `SYNTAX 40.925`, as a
name that is not a stem name does.

```rexx
signal on syntax name a
say sysfiletree()
a: say condition('o')~code
signal on syntax name b
say sysfiletree('c2zqft01*')
b: say condition('o')~code
signal on syntax name c
say sysfiletree('c2zqft01*', 'f.', 'D', '*****', '*****', 'x')
c: say condition('o')~code
signal on syntax name d
say sysfiletree(, 'f.')
d: say condition('o')~code
signal on syntax name e
say sysfiletree('c2zqft01*', , 'D')
e: say condition('o')~code
signal on syntax name f
say sysfiletree(.nil, 'f.')
f: say condition('o')~code
signal on syntax name g
say sysfiletree('c2zqft01*', 'f.', .nil)
g: say condition('o')~code
signal off syntax
say sysfiletree('c2zqft01*', 'f.', 'D', , )
say sysfiletree(.mutablebuffer~new('c2zqft01*'), 'f.', .array~of('D'))
```

```output
88.901
88.901
88.922
88.901
88.901
88.909
88.909
0
0
```

r[util.sysfiletree.order]
The arguments are checked in this order, and the first failure decides the
code: the number of arguments and the conversion of each argument to a string
(`88.9xx`); the name of the stem (`SYNTAX 40.925`); a `filespec` that is the
null string (`SYNTAX 40.21`); `options` (`SYNTAX 40.21` for the null string,
`SYNTAX 40.900` for a letter that is not an option); and last the two
attribute masks (`SYNTAX 40.900`). Nothing is searched until every check has
passed.

```rexx
call try "sysfiletree('', 'a b', 'X', 'zz', 'qq')"
call try "sysfiletree('', 'f.', 'X', 'zz', 'qq')"
call try "sysfiletree('', 'f.', '', 'zz', 'qq')"
call try "sysfiletree('c2zqft04*', 'f.', 'X', 'zz', 'qq')"
call try "sysfiletree('c2zqft04*', 'f.', '', 'zz', 'qq')"
call try "sysfiletree('c2zqft04*', 'f.', 'D', 'zz', 'qq')"
call try "sysfiletree('c2zqft04*', 'a b', 'X', 'zz', .nil)"
call try "sysfiletree('c2zqft04*', .object~new)"
call try "sysfiletree(.nil, 'a b', 'X', 'zz', 'qq')"
call try "sysfiletree('c2zqft04*', 'f.', 'D', '*****', 'qq')"
call try "sysfiletree('c2zqft04*', 'f.', 'D', 'zz', '*****')"
exit
try: procedure
  parse arg ex
  signal on syntax name bad
  interpret 'r = ' ex
  say r
  return
bad: say condition('o')~code
return
```

```output
40.925
40.21
40.21
40.900
40.21
40.900
88.909
40.925
88.909
40.900
40.900
```

## The stem

r[util.sysfiletree.stem]
When the call returns, the stem holds the number of matches in `.0` and the
matches in `.1` to `.n` (as every function that fills a stem does; rule
`util.index.stems`). No other tail is assigned or dropped: a call that finds
nothing sets `.0` to 0 and leaves the entries of an earlier call in place, and
a call that finds fewer matches than an earlier one leaves its surplus
entries. The `stem` argument names a stem variable, with or without the final
period and in any letter case (as for every stem argument; rule
`util.index.stems`), and an argument that is a stem object is filled in the
same way. An argument that is an array object is filled differently: it holds
the matches at the indexes 1 to n and no item at any other index. The order of
the matches is not specified.

```rexx
d = 'c2zqft02'
call sysmkdir d
call lineout d'/a.txt', 'x'; call lineout d'/a.txt'
call lineout d'/b.txt', 'x'; call lineout d'/b.txt'
spec = d'/*.txt'
say sysfiletree(spec, 'f.', 'FO') f.0 (f.1 \== f.2) symbol('f.3')
f.3 = 'old3'
f.9 = 'old9'
say sysfiletree(spec, 'f.', 'FO') f.0 f.3 f.9
say sysfiletree(d'/*.none', 'f.', 'FO') f.0 (f.1 \== '') f.3
say sysfiletree(spec, 'g', 'FO') g.0 symbol('g')
stemname = 'h.'
say sysfiletree(spec, stemname, 'FO') h.0
say sysfiletree(spec, 'K.', 'FO') k.0
s = .stem~new
say sysfiletree(spec, s, 'FO') s[0] s~items
arr = .array~of('p', 'q', 'r', 's')
say sysfiletree(spec, arr, 'FO') arr~items arr~hasindex(1) arr~hasindex(2) arr~hasindex(3)
say (arr[1] \== arr[2]) arr~hasindex(3)
arr2 = .array~new
say sysfiletree(d'/*.none', arr2, 'FO') arr2~items
call sysfiledelete d'/a.txt'
call sysfiledelete d'/b.txt'
call sysrmdir d
```

```output
0 2 1 LIT
0 2 old3 old9
0 0 1 old3
0 2 LIT
0 2
0 2
0 2 3
0 2 1 1 0
1 0
0 0
```

r[util.sysfiletree.stem.name]
A `stem` string that is not a stem name raises `SYNTAX 40.925`. A stem name is
a symbol that names a simple variable, with a period at its end or without; a
name with a period inside it, a name that starts with a digit or a period, a
name with a blank or an operator character in it, and the null string are not
stem names.

```rexx
names = .array~of('a b', '1.', '1', 'f.g', 'f.g.', 'f.1', 'f..', '.', '', '+', '.nil', 'f.x y', 'f', 'F.', '!x.', 'f_1.')
do n over names
  call try n
end
exit
try: procedure
  parse arg n
  signal on syntax name bad
  r = sysfiletree('c2zqft03*', n, 'FO')
  say '['n'] answered' r
  return
bad: say '['n'] raised' condition('o')~code
return
```

```output
[a b] raised 40.925
[1.] raised 40.925
[1] raised 40.925
[f.g] raised 40.925
[f.g.] raised 40.925
[f.1] raised 40.925
[f..] raised 40.925
[.] raised 40.925
[] raised 40.925
[+] raised 40.925
[.nil] raised 40.925
[f.x y] raised 40.925
[f] answered 0
[F.] answered 0
[!x.] answered 0
[f_1.] answered 0
```

## What is looked for

r[util.sysfiletree.spec]
`filespec` is a path whose last component is the pattern. `*` matches any run
of characters, including none and including a leading period, `?` matches one
character, `[ ]` matches one of the characters listed in it or in one of its
ranges such as `a-o`, and `[! ]` or `[^ ]` one that is not. The entries `.`
and `..` that every directory holds are never matches. In a class, a `]` that
comes first and a `-` that comes first or last stand for themselves, a range
whose ends are in the wrong order matches nothing, and a name such as
`[:upper:]` or `[:digit:]` stands for the characters of that kind. A `[` with
no `]` after it is an ordinary character, and so is `\`: nothing in a pattern
is escaped. Only the last component is a pattern. In an earlier component `*`,
`?` and `[` are ordinary characters of a directory's name, so `s*/x` looks in
a directory whose name is `s*`. The match is case-sensitive, and the option
letter `I` makes it ignore case for the last component only: the directory
part of a path always has to be spelled as the directory is. A pattern with no
wildcards matches the entry of that name.

```rexx
d = 'c2zqft05'
call sysmkdir d
call sysmkdir d'/Sub'
call sysmkdir d'/sub2'
call touch d'/one.txt'
call touch d'/Two.TXT'
call touch d'/.hid'
call touch d'/o-e.dat'
call touch d'/Sub/in.txt'
call touch d'/sub2/in2.txt'
call t d'/*', 'B'
call t d'/.*', 'B'
call t d'/o??.*', 'F'
call t d'/[a-o]*', 'F'
call t d'/[!a-o]*', 'F'
call t d'/[ot][nw]*', 'F'
call t d'/*.txt', 'F'
call t d'/*.txt', 'FI'
call t d'/ONE.TXT', 'FI'
call t d'/ONE.TXT', 'F'
call t d'/one.tx', 'F'
call t d'/s*/*.txt', 'F'
call t d'/Su?/*', 'B'
call t d'/*', 'D'
call t d'/Sub*', 'DI'
call t d'/SUB*', 'DI'
call t d'/sub*', 'D'
call t d'/Sub/*', 'BI'
call t d'/SUB/*', 'BI'
call sysfiledelete d'/Sub/in.txt'
call sysfiledelete d'/sub2/in2.txt'
call sysfiledelete d'/one.txt'
call sysfiledelete d'/Two.TXT'
call sysfiledelete d'/.hid'
call sysfiledelete d'/o-e.dat'
call sysrmdir d'/Sub'
call sysrmdir d'/sub2'
call sysrmdir d
exit
touch: procedure
  parse arg name
  call lineout name, 'x'
  call lineout name
return
t: procedure
  parse arg spec, opts
  call sysfiletree spec, 'f.', opts'O'
  a = .array~new
  do i = 1 to f.0
    a~append(changestr(directory()'/c2zqft05/', f.i, ''))
  end
  say left(substr(spec, 10)'  'opts, 12) a~sort~makestring('l', ' ')
return
```

```output
*  B         .hid Sub Two.TXT o-e.dat one.txt sub2
.*  B        .hid
o??.*  F     o-e.dat one.txt
[a-o]*  F    o-e.dat one.txt
[!a-o]*  F   .hid Two.TXT
[ot][nw]*  F one.txt
*.txt  F     one.txt
*.txt  FI    Two.TXT one.txt
ONE.TXT  FI  one.txt
ONE.TXT  F   
one.tx  F    
s*/*.txt  F  
Su?/*  B     
*  D         Sub sub2
Sub*  DI     Sub sub2
SUB*  DI     Sub sub2
sub*  D      sub2
Sub/*  BI    Sub/in.txt
SUB/*  BI    
```

```rexx
d = 'c2zqft17'
call sysmkdir d
call sysmkdir d'/s*'
call touch d'/s*/in.txt'
do n over .array~of('\x', '*x', '^q', 'bq', 'aq', ']z', 'b]', '-a', 'a-', 'ab', 'Xb', 'a[')
  call touch d'/'n
end
call t d'/\*', 'F'
call t d'/[^a]q', 'F'
call t d'/[!a]q', 'F'
call t d'/[]b]*', 'F'
call t d'/[a-]*', 'F'
call t d'/[-a]*', 'F'
call t d'/[b-a]*', 'F'
call t d'/[[:upper:]]*', 'F'
call t d'/[[:digit:]]*', 'F'
call t d'/a[', 'F'
call t d'/a[*', 'F'
call t d'/[ab', 'F'
call t d'/s*/in.txt', 'F'
call t d'/s*/*', 'F'
call t d'/s?/*', 'F'
address system 'rm -r 'd
exit
touch: procedure
  parse arg name
  call lineout name, 'x'
  call lineout name
return
t: procedure
  parse arg spec, opts
  call sysfiletree spec, 'f.', opts'O'
  a = .array~new
  do i = 1 to f.0
    a~append(changestr(directory()'/c2zqft17/', f.i, ''))
  end
  say left(substr(spec, 10)'  'opts, 16) a~sort~makestring('l', ' ')
return
```

```output
\*  F            \x
[^a]q  F         ^q bq
[!a]q  F         ^q bq
[]b]*  F         ]z b] bq
[a-]*  F         -a a- a[ ab aq
[-a]*  F         -a a- a[ ab aq
[b-a]*  F        
[[:upper:]]*  F  Xb
[[:digit:]]*  F  
a[  F            a[
a[*  F           a[
[ab  F           
s*/in.txt  F     s*/in.txt
s*/*  F          s*/in.txt
s?/*  F          
```

r[util.sysfiletree.spec.dir]
A `filespec` that names a directory without a slash at its end matches that
directory itself, and one with a slash at its end, or with `*` after it,
matches the entries in it. A `filespec` whose last component is `.` or `..`
names a directory and not a pattern: it matches the entries of that directory,
as the same path with a slash at its end does, so `dir/sub/..` and `dir/.`
both match the entries of `dir`. A path that has a component that is a file, a
component that does not exist, or a directory the program may not read, on the
way to the last component or as the directory whose entries are wanted,
matches nothing and is not an error; the unreadable directory itself is still
matched by its own name. A relative path is read as every name is (rule
`util.index.names`). A `filespec` that starts with `~` is read as every name
is (rule `util.index.home`), with one exception: a `filespec` that is `~name`
alone, where no user has that name, is a pattern for the entries of the
current directory, so it matches a directory `~name` there. Each name stored
is the full name of the match, formed as [QUALIFY](r:bif.qualify.normal) forms
a name: made absolute, with the repeated slashes, the periods and the `..`
components of the path taken out (Deviation 34, rule
`util.index.deviation.34`, says where ooRexx takes out a different component).
Symbolic links in the path are not replaced.

```rexx
here = directory()
d = 'c2zqft06'
call sysmkdir d
call sysmkdir d'/sub'
call lineout d'/one.txt', 'x'; call lineout d'/one.txt'
call lineout d'/sub/in.txt', 'x'; call lineout d'/sub/in.txt'
call t d, 'BO'
call t d'/', 'BO'
call t d'//', 'BO'
call t d'/sub', 'BO'
call t d'/sub/', 'BO'
call t d'/sub/', 'DO'
call t d'/sub/..', 'BO'
call t d'/.', 'DO'
call t d'/one.txt/', 'BO'
call t d'/one.txt', 'BO'
call t d'/one.txt/in.txt', 'BO'
call t d'/./one.txt', 'FO'
call t d'/sub/../one.txt', 'FO'
call t d'//sub///in.txt', 'FO'
call t here'/'d'/sub/in.txt', 'FO'
call t d'/nodir/in.txt', 'FO'
call t d'/nodir/*', 'BO'
call t d'/nodir/', 'BO'
call t 'c2zqft06/s*', 'DO'
old = value('HOME', here'/'d, 'ENVIRONMENT')
call t '~', 'DO'
call t '~/', 'FO'
call t '~/one.txt', 'FO'
call t '~/sub/*', 'FO'
call value 'HOME', old, 'ENVIRONMENT'
call sysmkdir d'/~c2zqnouser'
call lineout d'/~c2zqnouser/in.txt', 'x'; call lineout d'/~c2zqnouser/in.txt'
call directory d
call t '~c2zqnouser', 'DO'
call t '~c2zqnouser/*', 'FO'
call t './~c2zqnouser/*', 'FO'
call directory here
call sysfiledelete d'/~c2zqnouser/in.txt'
call sysrmdir d'/~c2zqnouser'
call sysfiledelete d'/sub/in.txt'
call sysfiledelete d'/one.txt'
call sysrmdir d'/sub'
call sysrmdir d
exit
t: procedure expose here
  parse arg spec, opts
  call sysfiletree spec, 'f.', opts
  a = .array~new
  do i = 1 to f.0
    a~append(changestr(here'/', f.i, '@/'))
  end
  say left(changestr(here'/', spec, '@/')' 'opts, 30) f.0 a~sort~makestring('l', ' ')
return
```

```output
c2zqft06 BO                    1 @/c2zqft06
c2zqft06/ BO                   2 @/c2zqft06/one.txt @/c2zqft06/sub
c2zqft06// BO                  2 @/c2zqft06/one.txt @/c2zqft06/sub
c2zqft06/sub BO                1 @/c2zqft06/sub
c2zqft06/sub/ BO               1 @/c2zqft06/sub/in.txt
c2zqft06/sub/ DO               0 
c2zqft06/sub/.. BO             2 @/c2zqft06/one.txt @/c2zqft06/sub
c2zqft06/. DO                  1 @/c2zqft06/sub
c2zqft06/one.txt/ BO           0 
c2zqft06/one.txt BO            1 @/c2zqft06/one.txt
c2zqft06/one.txt/in.txt BO     0 
c2zqft06/./one.txt FO          1 @/c2zqft06/one.txt
c2zqft06/sub/../one.txt FO     1 @/c2zqft06/one.txt
c2zqft06//sub///in.txt FO      1 @/c2zqft06/sub/in.txt
@/c2zqft06/sub/in.txt FO       1 @/c2zqft06/sub/in.txt
c2zqft06/nodir/in.txt FO       0 
c2zqft06/nodir/* BO            0 
c2zqft06/nodir/ BO             0 
c2zqft06/s* DO                 1 @/c2zqft06/sub
~ DO                           1 @/c2zqft06
~/ FO                          1 @/c2zqft06/one.txt
~/one.txt FO                   1 @/c2zqft06/one.txt
~/sub/* FO                     1 @/c2zqft06/sub/in.txt
~c2zqnouser DO                 1 @/c2zqft06/~c2zqnouser
~c2zqnouser/* FO               0 
./~c2zqnouser/* FO             1 @/c2zqft06/~c2zqnouser/in.txt
```

r[util.sysfiletree.kinds]
A directory is an entry that is a directory once symbolic links have been
followed; every other entry is a file, whether it is a regular file, a pipe, a
device or a link to one of them. A link is therefore described by what it
points to.

```rexx
d = 'c2zqft15'
call sysmkdir d
call sysmkdir d'/dir'
call lineout d'/file', 'x'; call lineout d'/file'
address system 'ln -s file 'd'/flink'
address system 'ln -s dir 'd'/dlink'
address system 'mkfifo 'd'/pipe'
do kind over .array~of('F', 'D', 'B')
  call sysfiletree d'/*', 'f.', kind'O'
  a = .array~new
  do i = 1 to f.0
    a~append(substr(f.i, lastpos('/', f.i) + 1))
  end
  say kind a~sort~makestring('l', ' ')
end
address system 'rm -r 'd
```

```output
F file flink pipe
D dir dlink
B dir dlink file flink pipe
```

## Options

r[util.sysfiletree.options]
`options` is a string of letters, read in either case (rule
`util.index.options`), with no blanks. `B` selects files and directories, `D`
directories only and `F` files only; when more than one of them is given, the
last decides, and when none is given, `B` applies. `S`, `I`, `O`, `L`, `T` and
`H` change what is searched, how names are compared and how a match is
written, and are described below; a letter may be repeated and the letters may
come in any order. Omitting `options`, or leaving its place empty, is the same
as `B`. `options` that is the null string raises `SYNTAX 40.21`.

```rexx
d = 'c2zqft07'
call sysmkdir d
call sysmkdir d'/sub'
call lineout d'/one.txt', 'x'; call lineout d'/one.txt'
call sysfiletree d'/*', 'f.', 'O'
say 'with O only:' f.0
call sysfiletree d'/*', 'f.'
say 'omitted:' f.0
do opts over .array~of('BO', 'DO', 'FO', 'bo', 'do', 'fo', 'DBO', 'BDO', 'DFO', 'FDO', 'FBO', 'BFO', 'DDO', 'FFO', 'ODB', 'OFD')
  call sysfiletree d'/*', 'f.', opts
  a = .array~new
  do i = 1 to f.0
    a~append(substr(f.i, lastpos('/', f.i) + 1))
  end
  say left(opts, 4) a~sort~makestring('l', ' ')
end
call sysfiledelete d'/one.txt'
call sysrmdir d'/sub'
call sysrmdir d
```

```output
with O only: 2
omitted: 2
BO   one.txt sub
DO   sub
FO   one.txt
bo   one.txt sub
do   sub
fo   one.txt
DBO  one.txt sub
BDO  sub
DFO  one.txt
FDO  sub
FBO  one.txt sub
BFO  one.txt
DDO  sub
FFO  one.txt
ODB  one.txt sub
OFD  sub
```

r[util.sysfiletree.options.error]
A character in `options` that is not one of `B`, `D`, `F`, `S`, `I`, `O`, `L`,
`T` and `H`, whether another letter, a blank, a digit or a punctuation
character, raises `SYNTAX 40.900`.

```rexx
do opts over .array~of('X', 'BX', 'DFZ', ' ', 'D ', ' D', 'D F', 'D,F', 'D-', '1', '0', 'DOT', 'DD1', '', 'dofl', 'bdfsilotH', 'LT', 'TL', 'tl')
  call try opts
end
exit
try: procedure
  parse arg opts
  signal on syntax name bad
  call sysfiletree 'c2zqft08*', 'f.', opts
  say '['opts'] ok' f.0
  return
bad: say '['opts'] 'condition('o')~code
return
```

```output
[X] 40.900
[BX] 40.900
[DFZ] 40.900
[ ] 40.900
[D ] 40.900
[ D] 40.900
[D F] 40.900
[D,F] 40.900
[D-] 40.900
[1] 40.900
[0] 40.900
[DOT] ok 0
[DD1] 40.900
[] 40.21
[dofl] ok 0
[bdfsilotH] ok 0
[LT] ok 0
[TL] ok 0
[tl] ok 0
```

r[util.sysfiletree.recurse]
With `S` the search goes through every directory below the one `filespec`
names, however deep, including a directory reached through a symbolic link,
which is searched under the link's name, and applies the pattern to the
entries of each. What a search with `S` does when a link leads back to a
directory above it is not specified. A directory is entered whether or not its
own name matches the pattern. The match is also made in the directory of
`filespec` itself, and an entry found at a deeper level is stored with its
full name.

```rexx
d = 'c2zqft09'
call sysmkdir d
call sysmkdir d'/a'
call sysmkdir d'/a/b'
call sysmkdir d'/c'
call touch d'/top.txt'
call touch d'/a/mid.txt'
call touch d'/a/b/low.txt'
call touch d'/a/b/low.dat'
call touch d'/c/other.dat'
address system 'ln -s ../c 'd'/a/lnk'
call t d'/*', 'BO'
call t d'/*', 'BSO'
call t d'/*', 'FSO'
call t d'/*', 'DSO'
call t d'/*.txt', 'FSO'
call t d'/*.txt', 'BSO'
call t d'/low*', 'BSO'
call t d'/a', 'DSO'
call t d'/b', 'DSO'
call t d'/a/', 'BSO'
call t d'/a/*', 'DSO'
call t d'/a/*', 'DO'
call t d'/*', 'sdo'
call t d'/X*', 'DSOI'
call t d'/A*', 'DSOI'
call t d'/*.TXT', 'FSOI'
call t d'/*.TXT', 'FSO'
call sysfiledelete d'/top.txt'
call sysfiledelete d'/a/mid.txt'
call sysfiledelete d'/a/b/low.txt'
call sysfiledelete d'/a/b/low.dat'
call sysfiledelete d'/c/other.dat'
address system 'rm 'd'/a/lnk'
call sysrmdir d'/a/b'
call sysrmdir d'/a'
call sysrmdir d'/c'
call sysrmdir d
exit
touch: procedure
  parse arg name
  call lineout name, 'x'
  call lineout name
return
t: procedure
  parse arg spec, opts
  call sysfiletree spec, 'f.', opts
  a = .array~new
  do i = 1 to f.0
    a~append(changestr(directory()'/c2zqft09/', f.i, ''))
  end
  say left(substr(spec, 10)' 'opts, 14) a~sort~makestring('l', ' ')
return
```

```output
* BO           a c top.txt
* BSO          a a/b a/b/low.dat a/b/low.txt a/lnk a/lnk/other.dat a/mid.txt c c/other.dat top.txt
* FSO          a/b/low.dat a/b/low.txt a/lnk/other.dat a/mid.txt c/other.dat top.txt
* DSO          a a/b a/lnk c
*.txt FSO      a/b/low.txt a/mid.txt top.txt
*.txt BSO      a/b/low.txt a/mid.txt top.txt
low* BSO       a/b/low.dat a/b/low.txt
a DSO          a
b DSO          a/b
a/ BSO         a/b a/b/low.dat a/b/low.txt a/lnk a/lnk/other.dat a/mid.txt
a/* DSO        a/b a/lnk
a/* DO         a/b a/lnk
* sdo          a a/b a/lnk c
X* DSOI        
A* DSOI        a
*.TXT FSOI     a/b/low.txt a/mid.txt top.txt
*.TXT FSO      
```

## What is stored

r[util.sysfiletree.line]
Unless `O` is given, a match is stored as one line of four fields: the time of
the last modification, the size, the attribute string and the full name, the
date and the time separated by two blanks. The time is written in the form
`M/DD/YY` for the date and `H:MMa` or `H:MMp` for the time on a twelve-hour
clock, where the month and the hour are right-aligned in two columns, the year
is the last two digits of the year, and midnight and noon are 12 with `a` and
`p`. `T` writes the time as `YY/MM/DD/HH/MM`, and `L` as `YYYY-MM-DD HH:MM:SS`
on a 24-hour clock; `L` wins when both are given. The time is the local time
of the host. The fields after the time are the size in a field of ten columns,
the attribute string and the name, each preceded by two blanks.

```rexx
d = 'c2zqft10'
call sysmkdir d
call charout d'/f.txt', 'twelve bytes'
call charout d'/f.txt'
address system 'chmod 640 'd'/f.txt'
call settime '2031-03-07', '14:05:09'
call t 'F'
call t 'FL'
call t 'FT'
call t 'FH'
call t 'FLH'
call t 'FTH'
call t 'FLT'
call t 'FO'
call t 'FOL'
call t 'FOH'
call settime '2031-03-07', '00:05:09'
call t 'F'
call settime '2031-03-07', '12:05:09'
call t 'F'
call settime '2031-03-07', '11:59:59'
call t 'F'
call settime '2031-03-07', '23:59:59'
call t 'FT'
call settime '2009-11-30', '09:08:07'
call t 'F'
call t 'FL'
call t 'FT'
call settime '2000-01-02', '10:00:00'
call t 'F'
call t 'FL'
call t 'FT'
call settime '1969-01-02', '10:00:00'
call t 'F'
call t 'FL'
call t 'FT'
call sysfiledelete d'/f.txt'
call sysrmdir d
exit
settime:
  parse arg dt, tm
  call syssetfiledatetime d'/f.txt', dt, tm
return
t: procedure
  parse arg opts
  call sysfiletree 'c2zqft10/f.txt', 'f.', opts
  say '['opts']' '['changestr(directory()'/c2zqft10/', f.1, '@/')']'
return
```

```output
[F] [ 3/07/31   2:05p          12  -rw-r-----  @/f.txt]
[FL] [2031-03-07 14:05:09          12  -rw-r-----  @/f.txt]
[FT] [31/03/07/14/05          12  -rw-r-----  @/f.txt]
[FH] [ 3/07/31   2:05p                    12  -rw-r-----  @/f.txt]
[FLH] [2031-03-07 14:05:09                    12  -rw-r-----  @/f.txt]
[FTH] [31/03/07/14/05                    12  -rw-r-----  @/f.txt]
[FLT] [2031-03-07 14:05:09          12  -rw-r-----  @/f.txt]
[FO] [@/f.txt]
[FOL] [@/f.txt]
[FOH] [@/f.txt]
[F] [ 3/07/31  12:05a          12  -rw-r-----  @/f.txt]
[F] [ 3/07/31  12:05p          12  -rw-r-----  @/f.txt]
[F] [ 3/07/31  11:59a          12  -rw-r-----  @/f.txt]
[FT] [31/03/07/23/59          12  -rw-r-----  @/f.txt]
[F] [11/30/09   9:08a          12  -rw-r-----  @/f.txt]
[FL] [2009-11-30 09:08:07          12  -rw-r-----  @/f.txt]
[FT] [09/11/30/09/08          12  -rw-r-----  @/f.txt]
[F] [ 1/02/00  10:00a          12  -rw-r-----  @/f.txt]
[FL] [2000-01-02 10:00:00          12  -rw-r-----  @/f.txt]
[FT] [00/01/02/10/00          12  -rw-r-----  @/f.txt]
[F] [ 1/02/69  10:00a          12  -rw-r-----  @/f.txt]
[FL] [1969-01-02 10:00:00          12  -rw-r-----  @/f.txt]
[FT] [69/01/02/10/00          12  -rw-r-----  @/f.txt]
```

r[util.sysfiletree.line.name]
With `O`, a match is stored as its full name only, and `O` takes precedence
over `L`, `T` and `H`, in whatever order the letters come.

```rexx
d = 'c2zqft16'
call sysmkdir d
call lineout d'/f.txt', 'x'; call lineout d'/f.txt'
address system 'chmod 640 'd'/f.txt'
call syssetfiledatetime d'/f.txt', '2031-03-07', '14:05:09'
do opts over .array~of('FO', 'FOL', 'FLO', 'OFL', 'FOT', 'TOF', 'FOH', 'HFO', 'OLTH', 'HTLOF', 'FL')
  call sysfiletree d'/f.txt', 'f.', opts
  say left(opts, 6) changestr(directory()'/c2zqft16/', f.1, '@/')
end
call sysfiledelete d'/f.txt'
call sysrmdir d
```

```output
FO     @/f.txt
FOL    @/f.txt
FLO    @/f.txt
OFL    @/f.txt
FOT    @/f.txt
TOF    @/f.txt
FOH    @/f.txt
HFO    @/f.txt
OLTH   @/f.txt
HTLOF  @/f.txt
FL     2031-03-07 14:05:09           2  -rw-r-----  @/f.txt
```

r[util.sysfiletree.size]
The size is the size of the file in bytes, right-aligned in ten columns. A
size above 9999999999 is stored as 9999999999, unless `H` is given: `H` makes
the field twenty columns wide and stores the true size. The size of a
directory is a value of the host.

```rexx
d = 'c2zqft11'
call sysmkdir d
call lineout d'/small.txt', 'x'; call lineout d'/small.txt'
address system 'truncate -s 0 'd'/empty.txt'
address system 'truncate -s 9999999999 'd'/ten-.dat'
address system 'truncate -s 10000000000 'd'/ten.dat'
address system 'truncate -s 12345678901 'd'/big.dat'
do name over .array~of('empty.txt', 'small.txt', 'ten-.dat', 'ten.dat', 'big.dat')
  call sysfiletree d'/'name, 'f.', 'F'
  call sysfiletree d'/'name, 'h.', 'FH'
  parse var f.1 . . size . .
  parse var h.1 . . hsize . .
  say left(name, 9) size hsize
end
address system 'rm -r 'd
```

```output
empty.txt 0 0
small.txt 2 2
ten-.dat  9999999999 9999999999
ten.dat   9999999999 10000000000
big.dat   9999999999 12345678901
```

r[util.sysfiletree.attr]
The attribute string has ten characters, written as a long listing of the host
writes the mode of a file. The first is the kind of the entry: `-` for a
regular file, `d` for a directory, `p` for a pipe, `c` for a character device,
`b` for a block device and `s` for a socket. The other nine are three groups
of `r`, `w` and `x` for the owner, the group and the others, each `-` where
the permission is not given, with `s` in place of the owner's or group's `x`
when the set-user-id or set-group-id bit is set, `S` where that bit is set and
`x` is not, and `t` or `T` in the same way for the sticky bit at the end. The
permissions of a file are those of the file the entry names, so a link shows
those of its target.

```rexx
d = 'c2zqft12'
call sysmkdir d
call sysmkdir d'/dir'
call lineout d'/f', 'x'; call lineout d'/f'
address system 'mkfifo -m 600 'd'/fifo'
address system 'ln -s f 'd'/link'
address system 'ln -s dir 'd'/dlink'
address system 'chmod 640 'd'/f'
address system 'chmod 750 'd'/dir'
call show 'f'
call show 'dir'
call show 'fifo'
call show 'link'
call show 'dlink'
do m over .array~of('0000', '0777', '4644', '2644', '1644', '4755', '2755', '1755', '7777', '4000', '2010', '1000', '0007')
  address system 'chmod' m d'/f'
  call show 'f', m
end
address system 'chmod 2750 'd'/dir'
call show 'dir', '2750'
call sysmkdir d'/stick'
address system 'chmod 1755 'd'/stick'
call show 'stick'
call sysfiletree '/dev/null', 'n.', 'B'
parse var n.1 . . . attr .
say 'device' n.0 left(attr, 1) length(attr)
address system 'rm -r 'd
exit
show: procedure
  parse arg name, tag
  call sysfiletree 'c2zqft12/'name, 'f.', 'B'
  parse var f.1 . . . attr .
  if tag == '' then tag = name
  say left(tag, 6) f.0 attr
return
```

```output
f      1 -rw-r-----
dir    1 drwxr-x---
fifo   1 prw-------
link   1 -rw-r-----
dlink  1 drwxr-x---
0000   1 ----------
0777   1 -rwxrwxrwx
4644   1 -rwSr--r--
2644   1 -rw-r-Sr--
1644   1 -rw-r--r-T
4755   1 -rwsr-xr-x
2755   1 -rwxr-sr-x
1755   1 -rwxr-xr-t
7777   1 -rwsrwsrwt
4000   1 ---S------
2010   1 ------s---
1000   1 ---------T
0007   1 -------rwx
2750   1 drwxr-s---
stick  1 drwxr-xr-t
device 1 c 10
```

## The attribute masks

r[util.sysfiletree.masks]
`tattrib` and `nattrib` are strings of up to five characters, each `*`, `+` or
`-`, or the null string. They are checked, and an argument that has any other
character or is longer than five characters raises `SYNTAX 40.900`. They have
no other effect: a mask does not select, and does not change a file or what is
stored for it.

```rexx
d = 'c2zqft13'
call sysmkdir d
call lineout d'/f.txt', 'x'; call lineout d'/f.txt'
address system 'chmod 444 'd'/f.txt'
call sysfiletree d'/f.txt', 'f.', 'F'
parse var f.1 . . . plain .
say plain
do m over .array~of('*****', '+++++', '-----', '***+*', '+**+*', '*++**', '---+-', '*', '', '++', '-+-+-')
  call sysfiletree d'/f.txt', 'f.', 'F', m
  parse var f.1 . . . attr .
  say left('['m']', 9) f.0 attr
end
do m over .array~of('*****', '+++++', '-----', '***+*', '-**+*', '+*+++', '')
  call sysfiletree d'/f.txt', 'f.', 'F', , m
  parse var f.1 . . . attr .
  say left('['m']', 9) f.0 attr
end
call sysfiletree d'/f.txt', 'f.', 'F', '---+-', '+++++'
say f.0
do m over .array~of('ADHRS', '*****x', '******', '*a***', '+ +++', 'adhrs', '12345')
  call try m
end
address system 'rm -r 'd
exit
try: procedure
  parse arg m
  signal on syntax name bad1
  call sysfiletree 'c2zqft13*', 'f.', 'D', m
  say '['m'] as target mask: ok'
  signal off syntax
  go: signal on syntax name bad2
  call sysfiletree 'c2zqft13*', 'f.', 'D', , m
  say '['m'] as new mask: ok'
  return
bad1: say '['m'] as target mask:' condition('o')~code
  signal go
bad2: say '['m'] as new mask:' condition('o')~code
return
```

```output
-r--r--r--
[*****]   1 -r--r--r--
[+++++]   1 -r--r--r--
[-----]   1 -r--r--r--
[***+*]   1 -r--r--r--
[+**+*]   1 -r--r--r--
[*++**]   1 -r--r--r--
[---+-]   1 -r--r--r--
[*]       1 -r--r--r--
[]        1 -r--r--r--
[++]      1 -r--r--r--
[-+-+-]   1 -r--r--r--
[*****]   1 -r--r--r--
[+++++]   1 -r--r--r--
[-----]   1 -r--r--r--
[***+*]   1 -r--r--r--
[-**+*]   1 -r--r--r--
[+*+++]   1 -r--r--r--
[]        1 -r--r--r--
1
[ADHRS] as target mask: 40.900
[ADHRS] as new mask: 40.900
[*****x] as target mask: 40.900
[*****x] as new mask: 40.900
[******] as target mask: 40.900
[******] as new mask: 40.900
[*a***] as target mask: 40.900
[*a***] as new mask: 40.900
[+ +++] as target mask: 40.900
[+ +++] as new mask: 40.900
[adhrs] as target mask: 40.900
[adhrs] as new mask: 40.900
[12345] as target mask: 40.900
[12345] as new mask: 40.900
```

## Matches that cannot be found

r[util.sysfiletree.none]
A path that matches nothing gives `.0` equal to 0 and the answer 0, whatever
the reason: a pattern that no entry fits, a directory or file that is not
there, a path through a file, a directory whose entries the program may not
read. Where a directory is unreadable, a search with `S` stores what it finds
elsewhere and does not report it.

```rexx
d = 'c2zqft14'
call sysmkdir d
call sysmkdir d'/locked'
call lineout d'/locked/in.txt', 'x'; call lineout d'/locked/in.txt'
call lineout d'/f.txt', 'x'; call lineout d'/f.txt'
address system 'id -u' with output stem u.
root = (u.1 == 0)
address system 'chmod 000 'd'/locked'
say 'missing directory:  ' sysfiletree(d'/nodir/*', 'f.') f.0
say 'missing file:       ' sysfiletree(d'/nofile', 'f.') f.0
say 'no match:           ' sysfiletree(d'/*.none', 'f.') f.0
say 'below a file:       ' sysfiletree(d'/f.txt/in.txt', 'f.') f.0
say 'below a file, slash:' sysfiletree(d'/f.txt/', 'f.') f.0
say 'unreadable listed:  ' sysfiletree(d'/locked', 'f.', 'DO') f.0
say 'unreadable content: ' sysfiletree(d'/locked/*', 'f.') (f.0 == 0 | root)
say 'unreadable named:   ' sysfiletree(d'/locked/in.txt', 'f.') (f.0 == 0 | root)
say 'unreadable, S:      ' sysfiletree(d'/*', 'f.', 'FSO') (f.0 == 1 | root)
address system 'chmod 755 'd'/locked'
address system 'rm -r 'd
```

```output
missing directory:   0 0
missing file:        0 0
no match:            0 0
below a file:        0 0
below a file, slash: 0 0
unreadable listed:   0 1
unreadable content:  0 1
unreadable named:    0 1
unreadable, S:       0 1
```
