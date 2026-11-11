---
element: "SysWait"
kind: function
syntax: "SysWait()"
deviations: [157]
rexxref: "Rexx Utilities (RexxUtil) > SysWait (Unix-like systems only)"
---

# SysWait

SysWait waits for a child process that the program has made with
[SysFork](sysfork.md) to end, and answers what the system reports about it.
[SysFork](sysfork.md) is refused ([Deviation
42](r:util.sysfork.deviation.42)), so that a program has no child process to
wait for. A call answers `-1` when there is no child to wait for ([Deviation
157](r:util.syswait.deviation.157)).

## Arguments

r[util.syswait.args]
SysWait takes no argument. Any argument raises `SYNTAX 88.922`, whether it is
a string, an object that has no string value, or an omitted place that is
followed by another argument.

```rexx
call try "SysWait(1)"
call try "SysWait('')"
call try "SysWait(.nil)"
call try "SysWait(, 1)"
call try "SysWait(1, 2)"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
88.922
88.922
88.922
88.922
88.922
```

## Deviations

r[util.syswait.deviation.157]
A call of SysWait made when the program has no child process that it could
wait for answers `-1`. ooRexx answers a number that nothing set, which differs
from run to run. The example runs the calls in a program of its own, which has
no child.

```rexx
address system 'rexx -e "say SysWait(); say SysWait()"' with output stem r.
do i = 1 to r.0
  say r.i
end
```

```output
-1
-1
```
