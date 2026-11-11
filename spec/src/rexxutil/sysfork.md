---
element: "SysFork"
kind: function
syntax: "SysFork()"
deviations: [42]
rexxref: "Rexx Utilities (RexxUtil) > SysFork (Unix-like systems only)"
---

# SysFork

SysFork asks for a copy of the running program. ooRexx makes one, and
interrexx refuses the call ([Deviation 42](r:util.sysfork.deviation.42)).

## Arguments

r[util.sysfork.args]
SysFork takes no argument. Any argument raises `SYNTAX 88.922`, and the call
is made, or refused, only when it has none. The same holds for an omitted
place that is followed by an argument.

```rexx
call try "SysFork(1)"
call try "SysFork('')"
call try "SysFork(.nil)"
call try "SysFork(, 1)"
call try "SysFork(1, 2)"
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

r[util.sysfork.deviation.42]
A call of SysFork with no argument ends the program at once, with exit code
`120` and a message on the error stream, and makes no copy: nothing that
follows the call runs. ooRexx makes a copy of the process, in which both
programs go on after the call. The call answers `0` in the copy and the number
of the new process in the original, and [SysWait](syswait.md) in the original
answers, for a copy that ended with EXIT and a whole number from `0` to `255`,
256 times that number. interrexx can show none of this: the example shows the
program that the refusal ends.

```rexx
say 'before'
pid = SysFork()
say 'after' pid
```

```output rc=120
before
```
