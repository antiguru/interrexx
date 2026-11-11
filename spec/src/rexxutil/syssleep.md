---
element: "SysSleep"
kind: function
syntax: "SysSleep(delay)"
deviations: [161]
rexxref: "Rexx Utilities (RexxUtil) > SysSleep"
---

# SysSleep

SysSleep pauses the program for a number of seconds, which may have a
fraction, and then answers `0`.

## Arguments

r[util.syssleep.args]
`delay` is a required number of seconds. The count and
the omission of the argument follow [the argument rule](r:util.index.args): a
call with no argument raises `SYNTAX 88.901` and a second argument raises
`SYNTAX 88.922`. The value is read as a 64-bit binary floating-point number,
whatever `NUMERIC DIGITS` is in force: a fraction of more digits than `NUMERIC
DIGITS` counts, but digits past the precision of that format do not, so that
`2147483.0000000001` is the delay `2147483`. Blanks before and after it, a
plus sign and an exponent are allowed. A value that is not a number, as `x`,
the null string and `0.01x` are, raises `SYNTAX 88.902`; so does a string with
a blank between two of its digits, as `1 0`, and a number written in the form
`0x10` or `1_0`, and so do the words `Inf`, `Infinity` and `NaN` in any case
and with or without a sign, and a number too large in size for a 64-bit binary
floating-point number, one of `1.797693134862316E+308` or more, or of
`-1.797693134862316E+308` or less, as `1e400`, `1e1000` and `-1e400` are; the
third example shows these. Whitespace after a sign is the subject of [Deviation
161](r:util.syssleep.deviation.161). Any other number below `0`, and any other
number above `2147483`, raises `SYNTAX 88.907`, and so does `2147483.4`: the
largest delay is `2147483`. A delay of `0`, or of a fraction too small to
count, is allowed, and so is a negative number too small in size for that
format, as `-1e-400`, which is the delay `0`. When an argument past the last
one comes with an omitted or wrong argument, [Deviation
101](r:util.index.deviation.101) says which error is raised. An object that
has no string value, as `.nil` is, raises `SYNTAX 88.909`, and so does an
object that has only a `STRING` method, and no `MAKESTRING` method; the second
and third examples show them.

```rexx
call try "SysSleep()"
call try "SysSleep(1, 2)"
call try "SysSleep('x')"
call try "SysSleep('')"
call try "SysSleep('0.01x')"
call try "SysSleep('1 0')"
call try "SysSleep('0x10')"
call try "SysSleep(-1)"
call try "SysSleep(-0.001)"
call try "SysSleep(2147484)"
call try "SysSleep(2147483.4)"
call try "SysSleep(0)"
call try "SysSleep(-0)"
call try "SysSleep(' 0.01 ')"
call try "SysSleep('+0.01')"
call try "SysSleep('1e-2')"
call try "SysSleep(.mutablebuffer~new('0.01'))"
call try "SysSleep(0.0000001)"
numeric digits 3
call try "SysSleep(2147483.4)"
call try "SysSleep('0.0123456')"
call try "SysSleep('-1e-400')"
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
88.902
88.902
88.902
88.902
88.902
88.907
88.907
88.907
88.907
0
0
0
0
0
0
0
88.907
0
0
```

```rexx
call try "SysSleep(.strval~new)"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::class strval
::method string
  return '0'
```

```output defect=#230
88.909
```

```rexx
call try "SysSleep(.nil)"
call try "SysSleep(.object~new)"
call try "SysSleep('Inf')"
call try "SysSleep('nan')"
call try "SysSleep('1e1000')"
call try "SysSleep('1e400')"
call try "SysSleep('-1e400')"
call try "SysSleep('-INF')"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output defect=#245
88.909
88.909
88.902
88.902
88.902
88.902
88.902
88.902
```

## The answer

r[util.syssleep.answer]
The call answers `0`, and not before at least `delay` seconds have gone by. It
is a lower bound: the program may be resumed later than that, and how much
later is not specified.

```rexx
call time 'R'
say SysSleep(0.3) (time('E') >= 0.3)
call time 'R'
say SysSleep('1.2e-1') (time('E') >= 0.12)
say SysSleep(0)
```

```output
0 1
0 1
0
```

r[util.syssleep.activity]
The pause is the pause of the activity that makes the call. The other
activities of the program keep running meanwhile, and a message that the
program has started goes on with its work during the delay (the activities
page of the concurrency chapter has the rules for an activity that waits). In
the example the started message posts a semaphore and ends after a pause of
its own, and the program, after its own pause, waits for the post and the
message's answer.

```rexx
f = .eventsemaphore~new
m = .w~new~start('run', f)
call SysSleep 0.3
f~wait
say 'posted' f~isPosted
say m~result
::class w
::method run unguarded
  use arg f
  f~post
  call SysSleep 0.05
  return 'worker done'
```

```output
posted 1
worker done
```

## Deviations

r[util.syssleep.deviation.161]
A delay is read as a Rexx number, in which
whitespace (a blank or a tab) may follow the sign: `'+ 0.5'` pauses for at
least half a second, and `'- 1'` is a negative delay, which raises `SYNTAX
88.907` as `-1` does. ooRexx reads a delay with whitespace after its sign as
no pause at all: the call answers `0` at once, whatever the number is, without
the checks of its range. In the example the first call pauses for at least
`0.3` seconds.

```rexx
call time 'R'
say SysSleep('+ 0.3') (time('E') >= 0.3)
call try "SysSleep('- 1')"
call try "SysSleep('- 0.5')"
call try "SysSleep(' - 2147484')"
call try "SysSleep('+ 2147484')"
call try "SysSleep('- 0')"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output defect=#241
0 1
88.907
88.907
88.907
88.907
0
```
