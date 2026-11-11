---
element: "SysSearchPath"
kind: function
syntax: "SysSearchPath(path, name [, option])"
rexxref: "Rexx Utilities (RexxUtil) > SysSearchPath"
---

# SysSearchPath

SysSearchPath finds a file along a list of directories that an environment
variable holds, and answers the full name of the first one it finds, or the
null string when it finds none. The variable is `path`, and the file is
`name`.

The examples set an environment variable of their own for the run of the
example, and remove it at the end.

## Arguments

r[util.syssearchpath.args]
`path` and `name` are required strings, and `option` is an optional string.
The count and the omission of the arguments, and an object in their place,
follow [the argument rule](r:util.index.args): a call with fewer than two
arguments, or with an omitted one, raises `SYNTAX 88.901`; a fourth argument
raises `SYNTAX 88.922`; and `.nil` for any of them raises `SYNTAX 88.909`.
`option` is read as [the option rule](r:util.syssearchpath.option) says.

```rexx
call try "SysSearchPath()"
call try "SysSearchPath('C1ZQSPX')"
call try "SysSearchPath(, 'c1zqspn')"
call try "SysSearchPath('C1ZQSPX', )"
call try "SysSearchPath('C1ZQSPX', 'c1zqspn', 'N', 'y')"
call try "SysSearchPath(.nil, 'c1zqspn')"
call try "SysSearchPath('C1ZQSPX', .nil)"
call try "SysSearchPath('C1ZQSPX', 'c1zqspn', .nil)"
call try "'[' || SysSearchPath('C1ZQSPX', 'c1zqspn', ) || ']'"
call try "'[' || SysSearchPath(.mutablebuffer~new('C1ZQSPX'), .mutablebuffer~new('c1zqspn'), .mutablebuffer~new('N')) || ']'"
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
88.909
[]
[]
```

r[util.syssearchpath.option]
`option` is `C`, to look in the current directory first and then along the
list, or `N`, to look along the list only, in upper or in lower case. It is
`C` when it is omitted. Only the first character of `option` is read, so
`Nope` is `N`; any other first character, and the null string, raises `SYNTAX
40.920`.

```rexx
here = directory()
call lineout 'c1zqspf', 'x'
call lineout 'c1zqspf'
call SysMkDir 'c1zqspd'
call lineout 'c1zqspd/g', 'x'
call lineout 'c1zqspd/g'
call value 'C1ZQSP', here || '/c1zqspd', 'ENVIRONMENT'
say SysSearchPath('C1ZQSP', 'c1zqspf') == here || '/c1zqspf'
say SysSearchPath('C1ZQSP', 'c1zqspf', 'C') == here || '/c1zqspf'
say SysSearchPath('C1ZQSP', 'c1zqspf', 'c') == here || '/c1zqspf'
say SysSearchPath('C1ZQSP', 'c1zqspf', 'CN') == here || '/c1zqspf'
say '[' || SysSearchPath('C1ZQSP', 'c1zqspf', 'N') || ']' '[' || SysSearchPath('C1ZQSP', 'c1zqspf', 'n') || ']'
say '[' || SysSearchPath('C1ZQSP', 'c1zqspf', 'Nope') || ']'
say SysSearchPath('C1ZQSP', 'g', 'n') == here || '/c1zqspd/g'
call try "SysSearchPath('C1ZQSP', 'g', 'X')"
call try "SysSearchPath('C1ZQSP', 'g', 'Q')"
call try "SysSearchPath('C1ZQSP', 'g', '')"
call try "SysSearchPath('C1ZQSP', 'g', ' N')"
call try "SysSearchPath('C1ZQSP', 'g', 1)"
call value 'C1ZQSP', .nil, 'ENVIRONMENT'
call SysFileDelete 'c1zqspf'
call SysFileDelete 'c1zqspd/g'
call SysRmDir 'c1zqspd'
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
1
1
1
1
[] []
[]
1
40.920
40.920
40.920
40.920
40.920
```

## The list

r[util.syssearchpath.list]
The environment variable named `path` holds a list of directories, separated
by colons. Its name is read as it is written: a name in another case is
another variable. A directory of the list is read as [every name
is](r:util.index.names), so a relative one is taken from the current
directory, and `~` is the home directory. An entry that is the null string
stands for no directory. A variable that has no value, or the null string, is
a list of no directory, and so is the name of no variable, which includes the
null string; `option` `C` still finds files of the current directory then. The
directories are tried in the order of the list.

```rexx
here = directory()
call SysMkDir 'c1zqsla'
call SysMkDir 'c1zqslb'
call lineout 'c1zqsla/f', 'x'
call lineout 'c1zqsla/f'
call lineout 'c1zqslb/f', 'x'
call lineout 'c1zqslb/f'
call lineout 'c1zqslb/h', 'x'
call lineout 'c1zqslb/h'
call lineout 'c1zqslc', 'x'
call lineout 'c1zqslc'
call value 'C1ZQSL', here || '/c1zqsla:' || here || '/c1zqslb', 'ENVIRONMENT'
oldhome = value('HOME', , 'ENVIRONMENT')
say SysSearchPath('C1ZQSL', 'f') == here || '/c1zqsla/f'
say SysSearchPath('C1ZQSL', 'h') == here || '/c1zqslb/h'
call value 'C1ZQSL', here || '/c1zqslb:' || here || '/c1zqsla', 'ENVIRONMENT'
say SysSearchPath('C1ZQSL', 'f') == here || '/c1zqslb/f'
call value 'C1ZQSL', 'c1zqslb:c1zqsla', 'ENVIRONMENT'
say SysSearchPath('C1ZQSL', 'f', 'N') == here || '/c1zqslb/f'
call value 'C1ZQSL', here || '/c1zqslx::' || here || '/c1zqslb', 'ENVIRONMENT'
say SysSearchPath('C1ZQSL', 'h', 'N') == here || '/c1zqslb/h'
call value 'C1ZQSL', ':' || here || '/c1zqslb', 'ENVIRONMENT'
say '[' || SysSearchPath('C1ZQSL', 'c1zqslc', 'N') || ']'
call value 'C1ZQSL', here || '/c1zqsla;' || here || '/c1zqslb', 'ENVIRONMENT'
say '[' || SysSearchPath('C1ZQSL', 'h', 'N') || ']'
call value 'C1ZQSL', here || '/c1zqslb/', 'ENVIRONMENT'
say SysSearchPath('C1ZQSL', 'h', 'N') == here || '/c1zqslb/h'
call value 'HOME', here || '/c1zqslb', 'ENVIRONMENT'
call value 'C1ZQSL', '~', 'ENVIRONMENT'
say SysSearchPath('C1ZQSL', 'h', 'N') == here || '/c1zqslb/h'
call value 'C1ZQSL', '', 'ENVIRONMENT'
say '[' || SysSearchPath('C1ZQSL', 'c1zqslc', 'N') || ']' (SysSearchPath('C1ZQSL', 'c1zqslc') == here || '/c1zqslc')
say '[' || SysSearchPath('C1ZQSLNONE', 'c1zqslc', 'N') || ']' (SysSearchPath('C1ZQSLNONE', 'c1zqslc') == here || '/c1zqslc')
say '[' || SysSearchPath('c1zqsl', 'f', 'N') || ']' (SysSearchPath('', 'c1zqslc') == here || '/c1zqslc')
call value 'C1ZQSL', .nil, 'ENVIRONMENT'
call value 'HOME', oldhome, 'ENVIRONMENT'
call SysFileDelete 'c1zqsla/f'
call SysFileDelete 'c1zqslb/f'
call SysFileDelete 'c1zqslb/h'
call SysFileDelete 'c1zqslc'
call SysRmDir 'c1zqsla'
call SysRmDir 'c1zqslb'
```

```output
1
1
1
1
1
[]
[]
1
1
[] 1
[] 1
[] 1
```

## The result

r[util.syssearchpath.found]
The answer is the full name of the first file that the search finds, in the
form that [every name](r:util.index.names) is cleaned up to: it starts with a
slash and has no `.` or `..` component and no run of slashes. A file is a
regular file, or a symbolic link to one, and a link in the name is not
replaced by what it points to. The current directory, when `option` is `C`,
and then each directory of the list are tried by joining it and `name` with a
slash. The first of them in which that is the name of an entry, with symbolic
links followed, decides. When the entry is a file, the answer is its name.
When it is anything else, such as a directory or a fifo, the answer is the
null string, even when a later directory holds a file of the name. A symbolic
link that points to nothing, and a name in a directory that the program may
not search, are no entry, and the search goes on. A `name` that is not found,
or is the null string or `.`, has no answer: the call answers the null
string. Blanks, `*` and `?` are characters of the name, so `g ` is not `g` and
no pattern is expanded. The name is not found in another case.

```rexx
here = directory()
call SysMkDir 'c1zqsfa'
call SysMkDir 'c1zqsfb'
call SysMkDir 'c1zqsfb/sub'
call SysMkDir 'c1zqsfa/x'
call lineout 'c1zqsfb/x', 'x'
call lineout 'c1zqsfb/x'
call lineout 'c1zqsfa/g', 'x'
call lineout 'c1zqsfa/g'
call SysMkDir 'c1zqsfb/g'
call lineout 'c1zqsfb/sub/k', 'x'
call lineout 'c1zqsfb/sub/k'
call lineout 'c1zqsff', 'x'
call lineout 'c1zqsff'
address system 'ln -s ../c1zqsff c1zqsfb/lk; ln -s sub c1zqsfb/ls'
call value 'C1ZQSF', here || '/c1zqsfa:' || here || '/c1zqsfb', 'ENVIRONMENT'
say SysSearchPath('C1ZQSF', 'g', 'N') == here || '/c1zqsfa/g'
say '[' || SysSearchPath('C1ZQSF', 'x', 'N') || ']'
say SysSearchPath('C1ZQSF', 'lk', 'N') == here || '/c1zqsfb/lk'
say SysSearchPath('C1ZQSF', 'sub/k', 'N') == here || '/c1zqsfb/sub/k'
say SysSearchPath('C1ZQSF', 'ls/k', 'N') == here || '/c1zqsfb/ls/k'
say SysSearchPath('C1ZQSF', 'g/', 'N') == here || '/c1zqsfa/g'
say SysSearchPath('C1ZQSF', 'x/../sub/k', 'N') == here || '/c1zqsfb/sub/k'
say '[' || SysSearchPath('C1ZQSF', 'sub', 'N') || ']' '[' || SysSearchPath('C1ZQSF', '.', 'N') || ']' '[' || SysSearchPath('C1ZQSF', 'g ', 'N') || ']' '[' || SysSearchPath('C1ZQSF', '', 'N') || ']'
say '[' || SysSearchPath('C1ZQSF', 'G', 'N') || ']' '[' || SysSearchPath('C1ZQSF', 'g*', 'N') || ']' '[' || SysSearchPath('C1ZQSF', 'c1zqsfnone', 'N') || ']'
call value 'C1ZQSF', .nil, 'ENVIRONMENT'
address system 'rm c1zqsfb/lk c1zqsfb/ls c1zqsff c1zqsfa/g c1zqsfb/x c1zqsfb/sub/k'
call SysRmDir 'c1zqsfb/g'
call SysRmDir 'c1zqsfb/sub'
call SysRmDir 'c1zqsfa/x'
call SysRmDir 'c1zqsfa'
call SysRmDir 'c1zqsfb'
```

```output
1
[]
1
1
1
1
1
[] [] [] []
[] [] []
```

```rexx
here = directory()
call SysMkDir 'c1zqsfa'
call SysMkDir 'c1zqsfb'
address system 'mkfifo c1zqsfa/p; ln -s c1zqsfnowhere c1zqsfa/k'
call lineout 'c1zqsfb/p', 'x'
call lineout 'c1zqsfb/p'
call lineout 'c1zqsfb/k', 'x'
call lineout 'c1zqsfb/k'
call lineout 'c1zqsfb/b ', 'x'
call lineout 'c1zqsfb/b '
call value 'C1ZQSF', here || '/c1zqsfa:' || here || '/c1zqsfb', 'ENVIRONMENT'
say '[' || SysSearchPath('C1ZQSF', 'p', 'N') || ']'
say (SysSearchPath('C1ZQSF', 'k', 'N') == here || '/c1zqsfb/k')
say (SysSearchPath('C1ZQSF', 'b ', 'N') == here || '/c1zqsfb/b ') '[' || SysSearchPath('C1ZQSF', 'b', 'N') || ']'
call value 'C1ZQSF', .nil, 'ENVIRONMENT'
address system 'rm c1zqsfa/p c1zqsfa/k c1zqsfb/p c1zqsfb/k "c1zqsfb/b "'
call SysRmDir 'c1zqsfa'
call SysRmDir 'c1zqsfb'
```

```output
[]
1
1 []
```

r[util.syssearchpath.name]
A `name` is read as [every name is](r:util.index.names): a `..` component is
resolved first, and `~` is the home directory. A `name` that starts with a
slash is a full name, and is not searched for: the answer is the name, cleaned
up, when it is the name of a file, and the null string when it is not,
whatever `option` and the list are. A `name` that starts with `./` or with
`../` is taken from the current directory and is not searched for along the
list either, with `N` too. Any other `name` is joined to the directories of
the list, and to the current directory first when `option` is `C`, so that a
`name` with a slash in it, such as `sub/k`, is a path from each of them.

```rexx
here = directory()
call SysMkDir 'c1zqsna'
call SysMkDir 'c1zqsnh'
call lineout 'c1zqsna/c1zqsnf', 'x'
call lineout 'c1zqsna/c1zqsnf'
call lineout 'c1zqsnf', 'x'
call lineout 'c1zqsnf'
call lineout 'c1zqsnh/hf', 'x'
call lineout 'c1zqsnh/hf'
call value 'C1ZQSN', here || '/c1zqsna', 'ENVIRONMENT'
oldhome = value('HOME', here || '/c1zqsnh', 'ENVIRONMENT')
say SysSearchPath('C1ZQSN', here || '/c1zqsna/c1zqsnf', 'N') == here || '/c1zqsna/c1zqsnf'
say '[' || SysSearchPath('C1ZQSN', here || '/c1zqsna/none') || ']' '[' || SysSearchPath('C1ZQSN', here || '/c1zqsna') || ']'
say SysSearchPath('C1ZQSN', here || '/c1zqsna//./c1zqsnf') == here || '/c1zqsna/c1zqsnf'
say SysSearchPath('C1ZQSN', 'c1zqsnf', 'N') == here || '/c1zqsna/c1zqsnf'
say SysSearchPath('C1ZQSN', './c1zqsnf', 'N') == here || '/c1zqsnf'
say SysSearchPath('C1ZQSN', '../' || filespec('N', here) || '/c1zqsnf', 'N') == here || '/c1zqsnf'
say '[' || SysSearchPath('C1ZQSN', 'c1zqsna/c1zqsnf', 'N') || ']' (SysSearchPath('C1ZQSN', 'c1zqsna/c1zqsnf') == here || '/c1zqsna/c1zqsnf')
say SysSearchPath('C1ZQSN', 'c1zqsnh/../c1zqsnf', 'N') == here || '/c1zqsna/c1zqsnf'
say SysSearchPath('C1ZQSN', '~/hf', 'N') == here || '/c1zqsnh/hf'
call value 'C1ZQSN', .nil, 'ENVIRONMENT'
call value 'HOME', oldhome, 'ENVIRONMENT'
call SysFileDelete 'c1zqsna/c1zqsnf'
call SysFileDelete 'c1zqsnh/hf'
call SysFileDelete 'c1zqsnf'
call SysRmDir 'c1zqsna'
call SysRmDir 'c1zqsnh'
```

```output
1
[] []
1
1
1
1
[] 1
1
1
```
