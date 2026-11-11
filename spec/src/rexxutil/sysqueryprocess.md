---
element: "SysQueryProcess"
kind: function
syntax: "SysQueryProcess([info])"
rexxref: "Rexx Utilities (RexxUtil) > SysQueryProcess"
---

# SysQueryProcess

SysQueryProcess answers a fact about the process that runs the program: its
number, its parent's number, its priority, the processor time it has used, the
memory it has used, or the number of signals it has received. The answer is a
number or a short line of text, as the page says for each `info`.

## Arguments

r[util.sysqueryprocess.args]
`info` is an optional string that says which fact
is wanted. When it is omitted, as in `SysQueryProcess()` and
`SysQueryProcess(,)`, the call answers the number of the process, as `PID`
does. A second argument raises `SYNTAX 88.922`, even where the first is
omitted or is not one of the words that [the next
rule](r:util.sysqueryprocess.info) knows. An object that is not a string is
replaced by its string value as [the argument rule](r:util.index.args) says,
and one that has none raises `SYNTAX 88.909`. When the first argument is an
object that has no string value, [Deviation 101](r:util.index.deviation.101)
says which error is raised with a second argument.

```rexx
pid = SysQueryProcess('PID')
say (SysQueryProcess() == pid) (SysQueryProcess(,) == pid)
say (SysQueryProcess('PID', ) == pid)
say (SysQueryProcess(.mutablebuffer~new('pid')) == pid)
call try "SysQueryProcess(.nil)"
call try "SysQueryProcess('PID', 1)"
call try "SysQueryProcess('x', 1)"
call try "SysQueryProcess(, 1)"
call try "SysQueryProcess('PID', 1, 2)"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
1 1
1
1
88.909
88.922
88.922
88.922
88.922
```

r[util.sysqueryprocess.info]
`info` is one of the words `PID`, `PPID`, `PPRIO`, `PTIME`, `PMEM` and
`PRCVDSIG`, in any mix of upper and lower case, and the whole of the string is
the word. Any other string raises `SYNTAX 40.1`: the null string, a word with
a blank before or after it, a part of a word such as `P`, a word with
something added, and the words `TID`, `TPRIO` and `TTIME` as well, which ask
for a fact about a thread that Linux has no answer for.

```rexx
do w over .array~of('PID', 'ppid', 'Pprio', 'ptime', 'PMEM', 'PrcvDsig')
  say w '->' (SysQueryProcess(w) \== '')
end
do w over .array~of('', ' PID', 'PID ', 'P', 'PIDX', 'PRIO', 'TID', 'TPRIO', 'TTIME', 'X', '*')
  call try w
end
exit
try:
  signal on syntax name e
  call SysQueryProcess arg(1)
  say 'no error for' arg(1)
  return
e: say '['arg(1)']' condition('o')~code
  return
```

```output
PID -> 1
ppid -> 1
Pprio -> 1
ptime -> 1
PMEM -> 1
PrcvDsig -> 1
[] 40.1
[ PID] 40.1
[PID ] 40.1
[P] 40.1
[PIDX] 40.1
[PRIO] 40.1
[TID] 40.1
[TPRIO] 40.1
[TTIME] 40.1
[X] 40.1
[*] 40.1
```

## The facts

r[util.sysqueryprocess.pid]
`PID` answers the number of the process that runs the program, a whole number
above `1`, and `PPID` the number of its parent. A command that ADDRESS runs
sees the process of the program as its parent. The number of the program is
the same at every call, and is not that of its parent. Which numbers they are
is not specified.

```rexx
pid = SysQueryProcess('PID')
ppid = SysQueryProcess('PPID')
say datatype(pid, 'W') (pid > 1) datatype(ppid, 'W') (ppid >= 0)
say (pid \= ppid) (pid == SysQueryProcess('PID'))
address system 'echo $PPID' with output stem a.
say 'parent of a command is the program:' (a.1 == pid)
address system "awk '/^PPid:/ {print $2}' /proc/$PPID/status" with output stem b.
say 'parent of the program:' (b.1 == ppid)
```

```output
1 1 1 1
1 1
parent of a command is the program: 1
parent of the program: 1
```

r[util.sysqueryprocess.pprio]
`PPRIO` answers the priority of the process as a whole number, the niceness
that Linux gives it: a larger number is a lower priority. A process starts
with the niceness of the process that started it, normally 0.
[SysSetPriority](syssetpriority.md) changes it ([Deviation
156](r:util.syssetpriority.deviation.156) is about the activities of the
program), and a command that ADDRESS starts has the niceness that the program
has. In the example the program lowers its priority by three steps, or by as
many as are left below the greatest niceness, and the command sees the new
number.

```rexx
n = SysQueryProcess('PPRIO')
say datatype(n, 'W')
d = min(3, 19 - n)
call SysSetPriority 1, -(n + d)
say (SysQueryProcess('PPRIO') - n == d)
address system 'nice' with output stem c.
say (c.1 - n == d)
```

```output
1
1
1
```

r[util.sysqueryprocess.ptime]
`PTIME` answers a line of text of the form `CPU_Time Summary:  `, a time, `
Kernel:  `, a time, `  User:  ` and a time, with two blanks after each colon
and between the times and the words. Each time is written `h:mm:ss:mmm` in
hours, minutes, seconds and milliseconds, the hours as a whole number and the
other fields with two, two and three digits. They are the processor time the
process has used so far in all, in the part of the system and in the program
itself. The first time is not less than either of the others, and no time
becomes smaller from one call to the next. The time of a command that ADDRESS
runs is not counted.

```rexx
t1 = SysQueryProcess('PTIME')
x = 0
do i = 1 to 20000; x = x + i; end
t2 = SysQueryProcess('PTIME')
do t over .array~of(t1, t2)
  parse var t 'CPU_Time Summary:' s 'Kernel:' k 'User:' u
  say (t == 'CPU_Time Summary:  ' || strip(s) || '  Kernel:  ' || strip(k) || '  User:  ' || strip(u))
  say (ms(s) >= ms(k)) (ms(s) >= ms(u))
end
parse var t1 'Summary:' s1 'Kernel:' k1 'User:' u1
parse var t2 'Summary:' s2 'Kernel:' k2 'User:' u2
say (ms(s2) >= ms(s1)) (ms(k2) >= ms(k1)) (ms(u2) >= ms(u1))
exit
ms: procedure
  parse arg h ':' m ':' s ':' z
  h = strip(h); z = strip(z)
  if \datatype(h, 'W') | length(m) \= 2 | length(s) \= 2 | length(z) \= 3 then return 'bad'
  return ((h * 60 + m) * 60 + s) * 1000 + z
```

```output
1
1 1
1
1 1
1 1 1
```

r[util.sysqueryprocess.pmem]
`PMEM` answers a line of text of the form `Max_Memory_RSS: ` and a whole
number: the most main memory, counted in kilobytes, that the process has held
at one time so far. Memory that a command that ADDRESS runs holds is not
counted: in the example a command holds 256 megabytes, and the number grows by
far less.

```rexx
t = SysQueryProcess('PMEM')
parse var t head n extra
say head '['extra']' datatype(n, 'W') (n > 0)
address system "awk '/^VmHWM:/ {print $2}' /proc/$PPID/status" with output stem h.
say (n * 2 > h.1) (n < h.1 * 2)
address system "awk 'BEGIN { s = ""x""; for (i = 0; i < 28; i++) s = s s; print length(s) }'" with output stem a.
parse value SysQueryProcess('PMEM') with . n2
say a.1 (n2 - n < 150000)
```

```output
Max_Memory_RSS: [] 1 1
1 1
268435456 1
```

r[util.sysqueryprocess.prcvdsig]
`PRCVDSIG` answers a line of text of the form `Received_signals: ` and a whole
number. On Linux that number is `0` for every process: it does not count the
signals the program was sent. In the example a command sends the program a
signal that is ignored, one that continues a stopped process and one that
carries urgent data, and the count stays `0`.

```rexx
address system 'kill -WINCH $PPID; kill -CONT $PPID; kill -URG $PPID'
t = SysQueryProcess('PRCVDSIG')
parse var t head n extra
say head n '['extra']'
```

```output
Received_signals: 0 []
```
