---
element: "SysSetFileDateTime"
kind: function
syntax: "SysSetFileDateTime(filename, [newdate], [newtime])"
rexxref: "Rexx Utilities (RexxUtil) > SysSetFileDateTime"
---

# SysSetFileDateTime

SysSetFileDateTime sets the time of the last modification of a file or
directory, to a date and a time given as text or to the present time. It
answers 0, or -1 when it did not set the time.

> **Note.** The examples set times in the middle of a day and a month, far
> from the changes between summer and winter time, because the time is a local
> time of the host. They read the result back with
> [SysGetFileDateTime](sysgetfiledatetime.md).

## Arguments

r[util.syssetfiledatetime.args]
`filename` is required; `newdate` and `newtime` are optional. The call follows
the argument rule (rule `util.index.args`): a missing `filename` raises
`SYNTAX 88.901`, a fourth argument raises `SYNTAX 88.922`, and an argument
that is an object other than a string is read by its string value, or raises
`SYNTAX 88.909` when it has none.

```rexx
d = 'c2zqst05'
call sysmkdir d
call lineout d'/f.txt', 'x'; call lineout d'/f.txt'
f = d'/f.txt'
call syssetfiledatetime f, '2031-01-15', '12:30:45'
call try "syssetfiledatetime()"
call try "syssetfiledatetime(f, '2032-01-01', '01:01:01', 'x')"
call try "syssetfiledatetime(.nil)"
call try "syssetfiledatetime(f, .nil)"
call try "syssetfiledatetime(f, '2032-01-01', .nil)"
call try "syssetfiledatetime(.mutablebuffer~new(f), .mutablebuffer~new('2032-01-01'), .mutablebuffer~new('01:01:01'))"
call try "sysgetfiledatetime(f)"
call sysfiledelete f
call sysrmdir d
exit
try: procedure expose f
  parse arg ex
  signal on syntax name bad
  interpret 'r = ' ex
  say ex '->' r
  return
bad: say ex '->' condition('o')~code
return
```

```output
syssetfiledatetime() -> 88.901
syssetfiledatetime(f, '2032-01-01', '01:01:01', 'x') -> 88.922
syssetfiledatetime(.nil) -> 88.909
syssetfiledatetime(f, .nil) -> 88.909
syssetfiledatetime(f, '2032-01-01', .nil) -> 88.909
syssetfiledatetime(.mutablebuffer~new(f), .mutablebuffer~new('2032-01-01'), .mutablebuffer~new('01:01:01')) -> 0
sysgetfiledatetime(f) -> 2032-01-01 01:01:01
```

## What is set

r[util.syssetfiledatetime.set]
A call with a `newdate`, a `newtime`, or both sets the time of the last
modification of `filename` to the date and the time given, local time, and
answers 0. An omitted `newdate` leaves the date of the present modification
time, and an omitted `newtime` leaves its time of day; an argument whose place
is empty is omitted. The time of the last access is not changed. `filename`
may be a directory, and a symbolic link is followed, so that the time of its
target is set. A name is read as every name is (rule `util.index.names`;
Deviation 34, rule `util.index.deviation.34`, says where ooRexx removes a
different component for a `..`), and a name that starts with `~` as rule
`util.index.home` says.

```rexx
d = 'c2zqst01'
call sysmkdir d
call sysmkdir d'/sub'
call lineout d'/f.txt', 'x'; call lineout d'/f.txt'
f = d'/f.txt'
say syssetfiledatetime(f, '2031-01-15', '12:30:45') sysgetfiledatetime(f)
say syssetfiledatetime(f, '2032-02-16') sysgetfiledatetime(f)
say syssetfiledatetime(f, , '01:02:03') sysgetfiledatetime(f)
say syssetfiledatetime(f, '2033-03-17', ) sysgetfiledatetime(f)
say syssetfiledatetime(f,, '04:05:06') sysgetfiledatetime(f)
say syssetfiledatetime(d'/sub', '1998-07-08', '09:10:11') sysgetfiledatetime(d'/sub')
say syssetfiledatetime(f, '1969-12-31', '23:59:59') sysgetfiledatetime(f)
say syssetfiledatetime(f, '2037-12-31', '12:00:00') sysgetfiledatetime(f)
say syssetfiledatetime(f, '1970-01-01', '12:00:00') sysgetfiledatetime(f)
a1 = sysgetfiledatetime(f, 'A')
call syssetfiledatetime f, '2031-01-15', '12:30:45'
say (sysgetfiledatetime(f, 'A') == a1) sysgetfiledatetime(f)
call sysfiledelete f
call sysrmdir d'/sub'
call sysrmdir d
```

```output
0 2031-01-15 12:30:45
0 2032-02-16 12:30:45
0 2032-02-16 01:02:03
0 2033-03-17 01:02:03
0 2033-03-17 04:05:06
0 1998-07-08 09:10:11
0 1969-12-31 23:59:59
0 2037-12-31 12:00:00
0 1970-01-01 12:00:00
1 2031-01-15 12:30:45
```

r[util.syssetfiledatetime.now]
A call with neither `newdate` nor `newtime` sets the time of the last
modification to the present time of the host and answers 0; as in [the rule
above](r:util.syssetfiledatetime.set), the time of the last access is not
changed. A `newdate` and a `newtime` that are both empty are omitted in the
same way.

```rexx
d = 'c2zqst06'
call sysmkdir d
call lineout d'/f.txt', 'x'; call lineout d'/f.txt'
call sysmkdir d'/sub'
f = d'/f.txt'
old = '2001-02-03 04:05:06'
do name over .array~of(f, d'/sub')
  call syssetfiledatetime name, '2001-02-03', '04:05:06'
  address system 'touch -a -d "2001-02-03 04:05:06" 'name
  a = sysgetfiledatetime(name, 'A')
  before = date('I')
  r = syssetfiledatetime(name)
  t = sysgetfiledatetime(name)
  after = date('I')
  say r (t \== old) (before <<= left(t, 10)) (left(t, 10) <<= after) length(t)
  say (sysgetfiledatetime(name, 'A') == a) a
  call syssetfiledatetime name, '2001-02-03', '04:05:06'
  r = syssetfiledatetime(name, , )
  t = sysgetfiledatetime(name)
  say r (t \== old) (substr(t, 5, 1) || substr(t, 11, 1) || substr(t, 14, 1))
end
call sysfiledelete f
call sysrmdir d'/sub'
call sysrmdir d
```

```output
0 1 1 1 19
1 2001-02-03 04:05:06
0 1 - :
0 1 1 1 19
1 2001-02-03 04:05:06
0 1 - :
```

r[util.syssetfiledatetime.form]
`newdate` is read as a year, a month and a day, three whole numbers in decimal
digits separated by hyphens, and `newtime` as an hour, a minute and a second,
separated by colons. Leading white space (blanks, tabs and the other space
characters of the C locale) before a number is skipped, but a blank before a
separator is not allowed. The year has at most four digits
and each other number at most two, and a separator has to follow a number
directly for the first two numbers of each; whatever follows the third number
is not read, so that `2031-01-155` is the fifteenth. A date or time that has
fewer than three numbers, other separators or a number that is too long for
its place gives the answer -1, and the file is not changed. The null string is
such a date or time, not an omitted one. A number may start with `+` or `-`;
the sign counts toward its length, and a negative number is borrowed from the
next larger field as [the next rule](r:util.syssetfiledatetime.carry) says.
How a year below 1000 and a year above 9999 are read is not specified.

```rexx
d = 'c2zqst03'
call sysmkdir d
call lineout d'/f.txt', 'x'; call lineout d'/f.txt'
f = d'/f.txt'
call t '2031-1-5', '2:3:4'
call t '2031-01-15 ', ' 12:30:45'
call t '2031- 01- 15', '12: 30: 45'
call t '2031-01-15xyz', '12:30:45abc'
call t '2031-01-15-99', '12:30:45:99'
call t '2031-01-155', '12:30:455'
call t '2031-01-1x', '12:30:4x'
call t '2031-011-15', '12:30:45'
call t '2031-01-15', '12:301:45'
call t '2031-01-15', '121:30:45'
call t '2031-01-15', '012:30:45'
call t '12345-01-01', '12:30:45'
call t '2031/01/15', '12:30:45'
call t '20310115', '12:30:45'
call t '2031-01', '12:30:45'
call t '2031', '12:30:45'
call t '', '12:30:45'
call t 'x', '12:30:45'
call t '2031 -01-15', '12:30:45'
call t '2031-01-15', '12:30'
call t '2031-01-15', '12'
call t '2031-01-15', '12.30.45'
call t '2031-01-15', '12:30:'
call t '2031-01-15', ''
call t '2031-01-15', 'x'
call t '2031-01-15', '12:30 :45'
call t 20310115, 123045
call t '2031-+1-15', '12:30:45'
call t '2031-01-15', '12:-1:45'
call t '2031-01-15', ' -1:30:45'
call t '-2031-01-15', '12:30:45'
call syssetfiledatetime f, '2031-06-15', '09:08:07'
say syssetfiledatetime(f, '2031-01-15', '09'x'12:30:45') sysgetfiledatetime(f)
call sysfiledelete f
call sysrmdir d
exit
t: procedure expose f
  parse arg dt, tm
  call syssetfiledatetime f, '2031-06-15', '09:08:07'
  r = syssetfiledatetime(f, dt, tm)
  say left('['dt']['tm']', 30) r sysgetfiledatetime(f)
return
```

```output
[2031-1-5][2:3:4]              0 2031-01-05 02:03:04
[2031-01-15 ][ 12:30:45]       0 2031-01-15 12:30:45
[2031- 01- 15][12: 30: 45]     0 2031-01-15 12:30:45
[2031-01-15xyz][12:30:45abc]   0 2031-01-15 12:30:45
[2031-01-15-99][12:30:45:99]   0 2031-01-15 12:30:45
[2031-01-155][12:30:455]       0 2031-01-15 12:30:45
[2031-01-1x][12:30:4x]         0 2031-01-01 12:30:04
[2031-011-15][12:30:45]        -1 2031-06-15 09:08:07
[2031-01-15][12:301:45]        -1 2031-06-15 09:08:07
[2031-01-15][121:30:45]        -1 2031-06-15 09:08:07
[2031-01-15][012:30:45]        -1 2031-06-15 09:08:07
[12345-01-01][12:30:45]        -1 2031-06-15 09:08:07
[2031/01/15][12:30:45]         -1 2031-06-15 09:08:07
[20310115][12:30:45]           -1 2031-06-15 09:08:07
[2031-01][12:30:45]            -1 2031-06-15 09:08:07
[2031][12:30:45]               -1 2031-06-15 09:08:07
[][12:30:45]                   -1 2031-06-15 09:08:07
[x][12:30:45]                  -1 2031-06-15 09:08:07
[2031 -01-15][12:30:45]        -1 2031-06-15 09:08:07
[2031-01-15][12:30]            -1 2031-06-15 09:08:07
[2031-01-15][12]               -1 2031-06-15 09:08:07
[2031-01-15][12.30.45]         -1 2031-06-15 09:08:07
[2031-01-15][12:30:]           -1 2031-06-15 09:08:07
[2031-01-15][]                 -1 2031-06-15 09:08:07
[2031-01-15][x]                -1 2031-06-15 09:08:07
[2031-01-15][12:30 :45]        -1 2031-06-15 09:08:07
[20310115][123045]             -1 2031-06-15 09:08:07
[2031-+1-15][12:30:45]         0 2031-01-15 12:30:45
[2031-01-15][12:-1:45]         0 2031-01-15 11:59:45
[2031-01-15][ -1:30:45]        0 2031-01-14 23:30:45
[-2031-01-15][12:30:45]        -1 2031-06-15 09:08:07
0 2031-01-15 12:30:45
```

r[util.syssetfiledatetime.carry]
A number above the largest value of its field is carried into the next larger
field, and one below the smallest value is borrowed from it: month 13 is
January of the next year, month 0 is December of the previous year, day 0 is
the last day of the previous month, day 31 of a month with 30 days is the
first of the next, hour 24 is midnight of the next day, and minute and second
60 are the next hour and the next minute. The answer is 0 and the time set is
the one reached.

```rexx
d = 'c2zqst02'
call sysmkdir d
call lineout d'/f.txt', 'x'; call lineout d'/f.txt'
f = d'/f.txt'
call t '2031-13-15', '12:30:45'
call t '2031-00-15', '12:30:45'
call t '2031-02-30', '12:30:45'
call t '2031-01-32', '12:30:45'
call t '2031-01-00', '12:30:45'
call t '2031-01-15', '24:00:00'
call t '2031-01-15', '12:60:00'
call t '2031-01-15', '12:30:60'
call t '2031-01-15', '25:61:61'
call t '2031-15-45', '26:75:80'
call sysfiledelete f
call sysrmdir d
exit
t: procedure expose f
  parse arg dt, tm
  call syssetfiledatetime f, '2031-06-15', '09:08:07'
  r = syssetfiledatetime(f, dt, tm)
  say left('['dt']['tm']', 30) r sysgetfiledatetime(f)
return
```

```output
[2031-13-15][12:30:45]         0 2032-01-15 12:30:45
[2031-00-15][12:30:45]         0 2030-12-15 12:30:45
[2031-02-30][12:30:45]         0 2031-03-02 12:30:45
[2031-01-32][12:30:45]         0 2031-02-01 12:30:45
[2031-01-00][12:30:45]         0 2030-12-31 12:30:45
[2031-01-15][24:00:00]         0 2031-01-16 00:00:00
[2031-01-15][12:60:00]         0 2031-01-15 13:00:00
[2031-01-15][12:30:60]         0 2031-01-15 12:31:00
[2031-01-15][25:61:61]         0 2031-01-16 02:02:01
[2031-15-45][26:75:80]         0 2032-04-15 03:16:20
```

## Failing

r[util.syssetfiledatetime.fail]
The answer is -1, and nothing is created or changed, when `filename` names
nothing, when the file's time may not be set, or when the date or time is not
of the form above. `filename` names nothing when no file has that name, when a
path goes through a file or a directory the program may not search, when it is
the null string, and when it has wildcards, which no function expands (rule
`util.index.names`). A file the program owns can be changed whether or not it
may be written. A `newdate` and `newtime` that are omitted do not make a
missing file exist.

```rexx
d = 'c2zqst04'
call sysmkdir d
call sysmkdir d'/ro'
call lineout d'/f.txt', 'x'; call lineout d'/f.txt'
call lineout d'/ro/g.txt', 'x'; call lineout d'/ro/g.txt'
f = d'/f.txt'
call syssetfiledatetime f, '2031-01-15', '12:30:45'
call try "syssetfiledatetime('')"
call try "syssetfiledatetime('', '2032-01-01', '01:01:01')"
call try "syssetfiledatetime(d'/none', '2032-01-01', '01:01:01')"
call try "syssetfiledatetime(d'/none')"
say 'not created:' (stream(d'/none', 'c', 'query exists') == '')
call try "syssetfiledatetime(d'/none/f.txt', '2032-01-01', '01:01:01')"
call try "syssetfiledatetime(f'/x', '2032-01-01', '01:01:01')"
call try "syssetfiledatetime(d'/*.txt', '2032-01-01', '01:01:01')"
call try "syssetfiledatetime(f' ', '2032-01-01', '01:01:01')"
call try "sysgetfiledatetime(f)"
call try "syssetfiledatetime(.mutablebuffer~new(f), .mutablebuffer~new('2032-01-01'), .mutablebuffer~new('01:01:01'))"
call try "sysgetfiledatetime(f)"
address system 'chmod 444 'f
call try "syssetfiledatetime(f, '2033-01-01', '01:01:01')"
call try "sysgetfiledatetime(f)"
address system 'chmod 555 'd
call try "syssetfiledatetime(f, '2034-01-01', '01:01:01')"
call try "sysgetfiledatetime(f)"
address system 'chmod 755 'd
address system 'ln -s f.txt 'd'/lnk'
call try "syssetfiledatetime(d'/lnk', '2035-01-01', '01:01:01')"
call try "sysgetfiledatetime(f)"
address system 'id -u' with output stem u.
root = (u.1 == 0)
r = syssetfiledatetime('/proc/version', '2036-01-01', '01:01:01')
say 'a file whose time may not be set:' (r == -1 | root)
address system 'chmod 000 'd'/ro'
r = syssetfiledatetime(d'/ro/g.txt', '2036-01-01', '01:01:01')
say 'in an unsearchable directory:' (r == -1 | root)
address system 'chmod 755 'd'/ro'
say 'and not changed:' (sysgetfiledatetime(d'/ro/g.txt') \== '2036-01-01 01:01:01')
old = value('HOME', directory()'/'d, 'ENVIRONMENT')
call try "syssetfiledatetime('~/f.txt', '2037-01-01', '01:01:01')"
call try "sysgetfiledatetime(f)"
call value 'HOME', old, 'ENVIRONMENT'
address system 'rm -r 'd
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
syssetfiledatetime('') -> -1
syssetfiledatetime('', '2032-01-01', '01:01:01') -> -1
syssetfiledatetime(d'/none', '2032-01-01', '01:01:01') -> -1
syssetfiledatetime(d'/none') -> -1
not created: 1
syssetfiledatetime(d'/none/f.txt', '2032-01-01', '01:01:01') -> -1
syssetfiledatetime(f'/x', '2032-01-01', '01:01:01') -> -1
syssetfiledatetime(d'/*.txt', '2032-01-01', '01:01:01') -> -1
syssetfiledatetime(f' ', '2032-01-01', '01:01:01') -> -1
sysgetfiledatetime(f) -> 2031-01-15 12:30:45
syssetfiledatetime(.mutablebuffer~new(f), .mutablebuffer~new('2032-01-01'), .mutablebuffer~new('01:01:01')) -> 0
sysgetfiledatetime(f) -> 2032-01-01 01:01:01
syssetfiledatetime(f, '2033-01-01', '01:01:01') -> 0
sysgetfiledatetime(f) -> 2033-01-01 01:01:01
syssetfiledatetime(f, '2034-01-01', '01:01:01') -> 0
sysgetfiledatetime(f) -> 2034-01-01 01:01:01
syssetfiledatetime(d'/lnk', '2035-01-01', '01:01:01') -> 0
sysgetfiledatetime(f) -> 2035-01-01 01:01:01
a file whose time may not be set: 1
in an unsearchable directory: 1
and not changed: 1
syssetfiledatetime('~/f.txt', '2037-01-01', '01:01:01') -> 0
sysgetfiledatetime(f) -> 2037-01-01 01:01:01
```
