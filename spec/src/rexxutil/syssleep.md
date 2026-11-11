---
element: "SysSleep"
kind: function
syntax: "SysSleep(delay)"
rexxref: "Rexx Utilities (RexxUtil) > SysSleep"
---

# SysSleep

SysSleep pauses the program for a number of seconds, which may have a
fraction, and then answers `0`.

## Arguments

r[util.syssleep.args]
`delay` is a required number of seconds. The count and the omission of the
argument follow [the argument rule](r:util.index.args): a call with no
argument raises `SYNTAX 88.901` and a second argument raises `SYNTAX 88.922`.
The value is read as a number whatever `NUMERIC DIGITS` is in force, so that a
fraction of more digits than that is read in full. Blanks before and after it,
a plus sign and an exponent are allowed. A value that is not a number, as `x`,
the null string and `0.01x` are, raises `SYNTAX 88.902`; so does a string with
a blank inside it, and a number written in the form `0x10` or `1_0`. A number
below `0`, and a number above `2147483`, raises `SYNTAX 88.907`, and so does
`2147483.4`: the largest delay is `2147483`. A delay of `0`, or of a fraction
too small to count, is allowed.

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
call try "SysSleep('0.0123456')"
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
0
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
its own, and the program, sleeping longer, finds the semaphore posted and the
message's answer ready.

```rexx
f = .eventsemaphore~new
m = .w~new~start('run', f)
call SysSleep 0.3
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
