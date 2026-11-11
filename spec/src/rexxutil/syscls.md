---
element: "SysCls"
kind: function
syntax: "SysCls()"
rexxref: "Rexx Utilities (RexxUtil) > SysCls"
---

# SysCls

SysCls clears the screen of a terminal. It does so by running the program
`clear` of the system, and it answers what the system reports about that run.
What `clear` writes, and whether it writes anything, depends on the terminal
that the environment names; so the examples of this page put a `clear` of
their own in front of the system's, in a directory that they make and remove,
and print what that one does.

## Arguments

r[util.syscls.args]
SysCls takes no argument. An argument raises `SYNTAX 88.922`, as
[the argument rule](r:util.index.args) says for an argument past the last one;
an omitted place at the end of the call adds nothing to the count, so
`SysCls(,)` is a call without arguments.

```rexx
call try "SysCls('x')"
call try "SysCls(,'x')"
call try "SysCls(.nil)"
call SysMkDir 'c6zqcls'
call lineout 'c6zqcls/clear', '#!/bin/sh'
call lineout 'c6zqcls/clear', 'echo "clear with $# arguments"'
call lineout 'c6zqcls/clear'
address system 'chmod +x c6zqcls/clear'
saved = value('PATH', directory() || '/c6zqcls:' || value('PATH', , 'ENVIRONMENT'), 'ENVIRONMENT')
say '[' || SysCls(,) || ']'
call value 'PATH', saved, 'ENVIRONMENT'
call SysFileDelete 'c6zqcls/clear'
call SysRmDir 'c6zqcls'
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
clear with 0 arguments
[0]
```

## What it runs

r[util.syscls.run]
SysCls hands the command `clear`, with no argument, to the shell of the
system. The shell looks for `clear` along the environment variable `PATH` as
it is at the moment of the call, so a program that sets `PATH` with
[VALUE](r:bif.value.os) changes which `clear` runs. The command writes
to the standard output and the standard error of the interpreter's own
process: it does not write to `.output` or `.error`, even when the program has
redirected them. What the program wrote earlier to its standard output is
already written when the command starts, so it comes first. The command reads
the standard input of the interpreter's process, so what it reads is no longer
there for the program. SysCls sets no variable, `RC` included.

```rexx
call SysMkDir 'c6zqcls'
call lineout 'c6zqcls/clear', '#!/bin/sh'
call lineout 'c6zqcls/clear', 'echo from-clear'
call lineout 'c6zqcls/clear'
address system 'chmod +x c6zqcls/clear'
saved = value('PATH', directory() || '/c6zqcls:' || value('PATH', , 'ENVIRONMENT'), 'ENVIRONMENT')
call charout , 'before '
call SysCls
say 'after'
file = .stream~new('c6zqcls/log')
file~open('write replace')
.output~destination(file)
say 'to the file'
rc = 77
call SysCls
say 'also to the file' rc
.output~destination
file~close
call value 'PATH', saved, 'ENVIRONMENT'
say 'file:'
do while lines('c6zqcls/log') > 0
  say '>' linein('c6zqcls/log')
end
call stream 'c6zqcls/log', 'c', 'close'
call SysFileDelete 'c6zqcls/clear'
call SysFileDelete 'c6zqcls/log'
call SysRmDir 'c6zqcls'
```

```output
before from-clear
after
from-clear
file:
> to the file
> also to the file 77
```

```rexx
call SysMkDir 'c6zqcls'
call lineout 'c6zqcls/clear', '#!/bin/sh'
call lineout 'c6zqcls/clear', 'read line'
call lineout 'c6zqcls/clear', 'echo "clear read [$line]"'
call lineout 'c6zqcls/clear'
address system 'chmod +x c6zqcls/clear'
call lineout 'c6zqcls/child.rex', "call value 'PATH', directory() || '/c6zqcls:' || value('PATH', , 'ENVIRONMENT'), 'ENVIRONMENT'"
call lineout 'c6zqcls/child.rex', 'call SysCls'
call lineout 'c6zqcls/child.rex', "say 'program read [' || linein() || ']'"
call lineout 'c6zqcls/child.rex'
address system 'printf "ab\ncd\n" | rexx c6zqcls/child.rex' with output stem out.
do i = 1 to out.0
  say out.i
end
call SysFileDelete 'c6zqcls/child.rex'
call SysFileDelete 'c6zqcls/clear'
call SysRmDir 'c6zqcls'
```

```output
clear read [ab]
program read [cd]
```

## The answer

r[util.syscls.answer]
SysCls answers `0` when `clear` ran and ended with the status `0`. When
`clear` ended with another status from `1` to `255`, SysCls answers that
status multiplied by 256. When the shell found no `clear` along `PATH`, the
status of the shell is `127`, so the answer is `32512`. Nothing is raised
for a failure.

```rexx
call SysMkDir 'c6zqcls'
saved = value('PATH', directory() || '/c6zqcls:' || value('PATH', , 'ENVIRONMENT'), 'ENVIRONMENT')
do status over 0, 1, 2, 200
  call lineout 'c6zqcls/clear', '#!/bin/sh'
  call lineout 'c6zqcls/clear', 'echo "status" ' || status
  call lineout 'c6zqcls/clear', 'exit ' || status
  call lineout 'c6zqcls/clear'
  address system 'chmod +x c6zqcls/clear'
  say SysCls()
  call SysFileDelete 'c6zqcls/clear'
end
call value 'PATH', directory() || '/c6zqcls', 'ENVIRONMENT'
say SysCls()
call value 'PATH', saved, 'ENVIRONMENT'
call SysRmDir 'c6zqcls'
```

```output
status 0
0
status 1
256
status 2
512
status 200
51200
32512
```
