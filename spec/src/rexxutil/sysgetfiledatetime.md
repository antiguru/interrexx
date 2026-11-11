---
element: "SysGetFileDateTime"
kind: function
syntax: "SysGetFileDateTime(filename, [timesel])"
rexxref: "Rexx Utilities (RexxUtil) > SysGetFileDateTime"
---

# SysGetFileDateTime

SysGetFileDateTime answers the time of the last modification or of the last
access of a file or directory, as text, or -1 when there is no such time to
give. [SysSetFileDateTime](syssetfiledatetime.md) sets the time of the last
modification.

> **Note.** The time of the last access is changed by the host when a file is
> read, so the examples show it only by its form and by its being unchanged by
> SysSetFileDateTime. Times are local times of the host.

## Arguments

r[util.sysgetfiledatetime.args]
`filename` is required and `timesel` is optional. The call follows [the native
call rule](r:bif.index.call.native): a missing `filename` raises `SYNTAX
88.901`, a third argument raises `SYNTAX 88.922`, and an argument that is an
object other than a string is read by its string value, or raises `SYNTAX
88.909` when it has none. A `timesel` whose place is empty, as in
`SysGetFileDateTime(f, )`, is an omitted one.

```rexx
d = 'c2zqgt02'
call sysmkdir d
call lineout d'/f.txt', 'x'; call lineout d'/f.txt'
f = d'/f.txt'
call syssetfiledatetime f, '2031-03-07', '14:05:09'
call try "sysgetfiledatetime()"
call try "sysgetfiledatetime(f, 'W', 'x')"
call try "sysgetfiledatetime(.nil)"
call try "sysgetfiledatetime(f, .nil)"
call try "sysgetfiledatetime(f, , )"
call try "sysgetfiledatetime(.mutablebuffer~new(f))"
call try "sysgetfiledatetime(f, .mutablebuffer~new('write'))"
call sysfiledelete f
call sysrmdir d
exit
try: procedure expose f d
  parse arg ex
  signal on syntax name bad
  interpret 'r = ' ex
  say ex '->' r
  return
bad: say ex '->' condition('o')~code
return
```

```output
sysgetfiledatetime() -> 88.901
sysgetfiledatetime(f, 'W', 'x') -> 88.922
sysgetfiledatetime(.nil) -> 88.909
sysgetfiledatetime(f, .nil) -> 88.909
sysgetfiledatetime(f, , ) -> 2031-03-07 14:05:09
sysgetfiledatetime(.mutablebuffer~new(f)) -> 2031-03-07 14:05:09
sysgetfiledatetime(f, .mutablebuffer~new('write')) -> 2031-03-07 14:05:09
```

## The answer

r[util.sysgetfiledatetime.answer]
The answer for a file or directory that exists is its time in the form
`YYYY-MM-DD HH:MM:SS`, in local time on a 24-hour clock, with the year in four
digits and every other field in two. The file may be a regular file, a
directory or any other kind of entry; a symbolic link is followed, so the
answer is the time of its target. A name that is `~` or starts with `~/`
starts at the directory named by the environment variable `HOME`.

```rexx
d = 'c2zqgt01'
call sysmkdir d
call lineout d'/f.txt', 'x'; call lineout d'/f.txt'
f = d'/f.txt'
call sysmkdir d'/sub'
call syssetfiledatetime f, '2031-03-07', '14:05:09'
call syssetfiledatetime d'/sub', '1999-12-31', '23:59:58'
say syssetfiledatetime(d'/sub', '1999-12-31', '23:59:58')
say sysgetfiledatetime(f)
say sysgetfiledatetime(f, 'W') sysgetfiledatetime(f, 'w') sysgetfiledatetime(f, 'Write')
say sysgetfiledatetime(f, 'WRITEX') sysgetfiledatetime(f, 'Wz') sysgetfiledatetime(f, 'W ')
say sysgetfiledatetime(d'/sub')
say sysgetfiledatetime(d'/sub/')
a = sysgetfiledatetime(f, 'A')
say translate(a, '##########', '0123456789')
say (a == sysgetfiledatetime(f, 'a')) (a == sysgetfiledatetime(f, 'ACCESS'))
call syssetfiledatetime f, '2040-01-02', '03:04:05'
say (sysgetfiledatetime(f, 'A') == a) sysgetfiledatetime(f, 'W')
call sysfiledelete f
call sysrmdir d'/sub'
call sysrmdir d
```

```output
0
2031-03-07 14:05:09
2031-03-07 14:05:09 2031-03-07 14:05:09 2031-03-07 14:05:09
2031-03-07 14:05:09 2031-03-07 14:05:09 2031-03-07 14:05:09
1999-12-31 23:59:58
1999-12-31 23:59:58
####-##-## ##:##:##
1 1
1 2040-01-02 03:04:05
```

r[util.sysgetfiledatetime.timesel]
`timesel` selects the time, by its first character alone: `W` for the time of
the last modification, which is the default, and `A` for the time of the last
access, in either letter case; the characters after the first are not looked
at. A `timesel` that does not start with `W` or `A`, including the null
string, a letter such as `C` that selects a time this host does not keep, a
number and a value that starts with a blank, raises `SYNTAX 40.920`.

```rexx
d = 'c2zqgt03'
call sysmkdir d
call lineout d'/f.txt', 'x'; call lineout d'/f.txt'
f = d'/f.txt'
call syssetfiledatetime f, '2031-03-07', '14:05:09'
call try "sysgetfiledatetime(f, 'C')"
call try "sysgetfiledatetime(f, 'CREATE')"
call try "sysgetfiledatetime(f, 'X')"
call try "sysgetfiledatetime(f, '')"
call try "sysgetfiledatetime(f, ' W')"
call try "sysgetfiledatetime(f, 1)"
call try "sysgetfiledatetime(f, 'wX')"
call sysfiledelete f
call sysrmdir d
exit
try: procedure expose f d
  parse arg ex
  signal on syntax name bad
  interpret 'r = ' ex
  say ex '->' r
  return
bad: say ex '->' condition('o')~code
return
```

```output
sysgetfiledatetime(f, 'C') -> 40.920
sysgetfiledatetime(f, 'CREATE') -> 40.920
sysgetfiledatetime(f, 'X') -> 40.920
sysgetfiledatetime(f, '') -> 40.920
sysgetfiledatetime(f, ' W') -> 40.920
sysgetfiledatetime(f, 1) -> 40.920
sysgetfiledatetime(f, 'wX') -> 2031-03-07 14:05:09
```

r[util.sysgetfiledatetime.fail]
The answer is -1 when `filename` names nothing: a name that no file has, a
path through a file, a path through a directory the program may not search,
the null string, and a name with wildcards, which is not expanded and is taken
as a name. Blanks at the start or the end of `filename` are part of the name.
No error is raised.

```rexx
d = 'c2zqgt04'
call sysmkdir d
call lineout d'/f.txt', 'x'; call lineout d'/f.txt'
f = d'/f.txt'
call syssetfiledatetime f, '2031-03-07', '14:05:09'
old = value('HOME', directory()'/'d, 'ENVIRONMENT')
call try "sysgetfiledatetime('')"
call try "sysgetfiledatetime(d'/none')"
call try "sysgetfiledatetime(d'/none/f.txt')"
call try "sysgetfiledatetime(f'/x')"
call try "sysgetfiledatetime(f' ')"
call try "sysgetfiledatetime(' 'f)"
call try "sysgetfiledatetime(d'/*.txt')"
call try "sysgetfiledatetime(d'/f.tx?')"
call try "sysgetfiledatetime('~/f.txt')"
call try "sysgetfiledatetime('~/none')"
call try "sysgetfiledatetime(d'/sub')"
call value 'HOME', old, 'ENVIRONMENT'
address system 'id -u' with output stem u.
address system 'chmod 000 'd
r = sysgetfiledatetime(f)
address system 'chmod 755 'd
say 'in an unsearchable directory:' (r == -1 | u.1 == 0)
call sysfiledelete f
call sysrmdir d
exit
try: procedure expose f d
  parse arg ex
  signal on syntax name bad
  interpret 'r = ' ex
  say ex '->' r
  return
bad: say ex '->' condition('o')~code
return
```

```output
sysgetfiledatetime('') -> -1
sysgetfiledatetime(d'/none') -> -1
sysgetfiledatetime(d'/none/f.txt') -> -1
sysgetfiledatetime(f'/x') -> -1
sysgetfiledatetime(f' ') -> -1
sysgetfiledatetime(' 'f) -> -1
sysgetfiledatetime(d'/*.txt') -> -1
sysgetfiledatetime(d'/f.tx?') -> -1
sysgetfiledatetime('~/f.txt') -> 2031-03-07 14:05:09
sysgetfiledatetime('~/none') -> -1
sysgetfiledatetime(d'/sub') -> -1
in an unsearchable directory: 1
```
