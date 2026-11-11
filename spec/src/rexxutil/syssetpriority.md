---
element: "SysSetPriority"
kind: function
syntax: "SysSetPriority(class, delta)"
deviations: [156]
rexxref: "Rexx Utilities (RexxUtil) > SysSetPriority"
---

# SysSetPriority

SysSetPriority sets the priority of the program, the niceness that
[SysQueryProcess](sysqueryprocess.md) answers for `PPRIO`, and answers `0`
when it did, or a number that is not zero when the system refused.

## Arguments

r[util.syssetpriority.args]
`delta` is a required whole number. `class` is a whole number too, and may be
omitted; its value, and a place that is left empty, make no difference to the
call ([the next rule](r:util.syssetpriority.answer)). A number is a whole
number when it has no fraction or only zeros in it, as `1.0` and `1e0` have,
and it is read whatever `NUMERIC DIGITS` is in force, in the range
`-2147483648` to `2147483647`. A value that is not a whole number or is out of
that range raises `SYNTAX 88.907`: a fraction such as `1.5` or `.5`, the null
string, a word, and the words `IDLE`, `NORMAL`, `LOWEST` and the like, with
which some descriptions of the function name classes and deltas, are not
numbers here. A value that is given is checked before the omission of `delta`
is reported, so that `SysSetPriority('x')` raises `88.907` and
`SysSetPriority(1)` raises `SYNTAX 88.901`; three arguments raise `SYNTAX
88.922`. When an argument past the last one comes with an omitted or wrong
argument, [Deviation 101](r:util.index.deviation.101) says which error is
raised. An object that is not a string is read by the string that its `STRING`
method answers, not by its `request`, so that `.nil` raises `SYNTAX 88.907`
and an object whose `STRING` answers a whole number is that number; the second
example shows it.

```rexx
call try "SysSetPriority()"
call try "SysSetPriority(1)"
call try "SysSetPriority(,)"
call try "SysSetPriority(1,)"
call try "SysSetPriority(1, 2, 3)"
call try "SysSetPriority('x')"
call try "SysSetPriority(, 'x')"
call try "SysSetPriority(1, 'x')"
call try "SysSetPriority('x', 1)"
call try "SysSetPriority(1, '')"
call try "SysSetPriority('', 1)"
call try "SysSetPriority(1, 1.5)"
call try "SysSetPriority(1, .5)"
call try "SysSetPriority(1.5, 1)"
call try "SysSetPriority(1, IDLE)"
call try "SysSetPriority(1, 'LOWEST')"
call try "SysSetPriority('NORMAL', 0)"
call try "SysSetPriority(1, '2147483648')"
call try "SysSetPriority(2147483648, 1)"
call try "SysSetPriority(1, '-2147483649')"
call try "SysSetPriority('-2147483649', 1)"
call try "SysSetPriority(1, .nil)"
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
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
```

```rexx
b = -SysQueryProcess('PPRIO')
call try "SysSetPriority(1, .sv~new(b))"
call try "SysSetPriority(.sv~new(0), b)"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::class sv
::method init
  expose v
  use arg v
::method string
  expose v
  return v
```

```output defect=#230
0
0
```

## The answer

r[util.syssetpriority.answer]
The call sets the niceness of the program to the number `delta` with its sign
changed, so that `-4` makes it 4: a delta of `-19` or less makes the niceness
19, the greatest there is. `class` is not used: every whole number gives the
same result as any other, and so does its omission. The call answers `0` when
it set the niceness. A program that may not raise its own priority (the
superuser may) is refused a niceness lower than the one it has, and the call
then answers `13` and leaves the niceness as it was; a niceness that is the
same as the one it has, or higher, is always allowed, and a program that has
lowered its priority cannot raise it again unless the system grants it. A
command that ADDRESS starts afterwards has the niceness the call set;
[Deviation 156](r:util.syssetpriority.deviation.156) is about the other
activities of the program. The example guards the lines that the superuser's
permission changes.

```rexx
address system 'id -u' with output stem u.
root = (u.1 == 0)
base = SysQueryProcess('PPRIO')
want = min(base + 2, 19)
do class over .array~of(0, 1, 2, 3, 4, -1, 1.0, '2e0', ' 3 ')
  call SysSetPriority class, -want
  say '['class']' (SysQueryProcess('PPRIO') == want) (SysSetPriority(class, -want))
end
say SysSetPriority(, -want) SysSetPriority(.mutablebuffer~new(1), -want)
address system 'nice' with output stem c.
say (c.1 == want)
r = SysSetPriority(1, -base)
say (r == 13 | root | want == base) (SysQueryProcess('PPRIO') == want | root)
say SysSetPriority(1, -100) SysQueryProcess('PPRIO')
say SysSetPriority(1, -20) SysQueryProcess('PPRIO')
say SysSetPriority(1, -19) SysQueryProcess('PPRIO')
```

```output
[0] 1 0
[1] 1 0
[2] 1 0
[3] 1 0
[4] 1 0
[-1] 1 0
[1.0] 1 0
[2e0] 1 0
[ 3 ] 1 0
0 0
1
1 1
0 19
0 19
0 19
```

## Deviations

r[util.syssetpriority.deviation.156]
The call changes the priority of the
process, whichever activity of the program makes it: the number that
SysQueryProcess answers for `PPRIO` and the niceness of the commands that
ADDRESS starts are the new ones in every activity. ooRexx changes the priority
of the system thread that runs the calling activity: the activities that are
running on other threads, and the commands that they start, keep the priority
they had, and an activity started later has the new priority or the old one,
as the thread that ooRexx runs it on has. In the example a message that the
program started calls SysSetPriority, and the program sees the change.

```rexx
base = SysQueryProcess('PPRIO')
d = min(3, 19 - base)
m = .t~new~start('q', base + d)
say 'worker:' m~result
say 'program:' (SysQueryProcess('PPRIO') - base == d)
address system 'nice' with output stem c.
say 'command:' (c.1 - base == d)
::class t
::method q unguarded
  use arg delta
  return SysSetPriority(1, -delta)
```

```output
worker: 0
program: 1
command: 1
```
