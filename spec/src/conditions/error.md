---
element: "ERROR and FAILURE"
kind: concept
ansi: "8.4.1"
rexxref: "Conditions and Condition Traps; Conditions and Condition Traps > Condition Information > Descriptive Strings; Conditions and Condition Traps > Action Taken when a Condition Is Trapped; Conditions and Condition Traps > Action Taken when a Condition Is Not Trapped"
---

# ERROR and FAILURE

ERROR and FAILURE are the two conditions of a command. After an environment
has run a command, or has failed to, the interpreter looks at the return code
and raises one of them. ERROR means that the command ran and answered with a
return code that is not zero, and FAILURE means that the command could not be
run at all. A program that wants to react to a failing command traps one or
both. This page specifies when each is raised, what the condition records, and
the way a FAILURE that nothing answers reaches an ERROR trap. How a command is
sent, and how it sets `RC`, is on [the command
page](../instructions/command.md).

## ERROR

r[cond.error.raise]
A command that the environment has run, and whose return code is not zero,
raises ERROR at the end of the clause that sent it, so that the command has
run and `RC` holds its return code before the trap answers. A return code of
zero raises nothing. The record of the condition has the command string as its
description, and the `RC` entry of the condition object holds the return code,
which is also the value of `RC` in the code of the trap. The `POSITION` entry
and `SIGL` are the line of the command clause. A SIGNAL trap goes to its label
after the command has run, and a CALL trap calls its routine and then goes on
with the clause after the command. The status `.RS` of the command is `1`
for a command that raised ERROR, as [the status rule](r:instr.command.status)
says.

```rexx
address system
call on error name h
'exit 0'
say 'zero' rc .rs
'exit 3'
say 'after' rc .rs
cmd = 'exit' 2 + 5
cmd
signal on error name s
'echo ran; exit 4'
say 'not reached'
s: say 'signal' rc condition('c') condition('s') condition('i')
exit
h: o = arg(1)
  say 'handler' rc '['condition('d')']' o~rc o~condition sigl o~position
  return
```

```output
zero 0 0
handler 3 [exit 3] 3 ERROR 5 5
after 3 1
handler 7 [exit 7] 7 ERROR 8 8
ran
signal 4 ERROR OFF SIGNAL
```

r[cond.error.untrapped]
An ERROR that no trap answers is ignored, as [the untrapped
rule](r:cond.index.untrapped) says: the clause is over, `RC` holds the return
code, `.RS` is `1`, and the program goes on. The traps that answer it are the
ones of the code that sent the command, which for an internal routine are its
copy of the traps of its caller, as [the scope rule](r:cond.traps.scope) says.
The condition is not passed on to the next level, as [the propagation
rule](r:cond.information.propagate.only) says, so a routine that has its trap
off ignores the ERROR although its caller has the trap on.

```rexx
address system
'exit 3'
say 'went on' rc .rs
call on error name caught
call fresh
say 'back' rc
call copy
say 'back again' rc
call quiet
say 'and again' rc
exit
caught: say 'caught' condition('c') rc sigl; return
copy:
  'exit 6'
  return
quiet:
  call off error
  'exit 7'
  say 'quiet went on' rc
  return
::routine fresh
  'exit 5'
  say 'fresh went on' rc
```

```output
went on 3 1
fresh went on 5
back 3
caught ERROR 6 14
back again 6
quiet went on 7
and again 7
```

## FAILURE

r[cond.error.failure]
A command raises FAILURE, and not ERROR, when it could not be run. That is the
case when the current environment does not exist, as [the status
rule](r:instr.command.status) says, whatever the command is. On a POSIX host
it is also the case when the return code is `127`, in the command
environments `system`, `command`, `sh`, `bash` and `path`, whether the command
was not found or exited with that code. Whether other environments of a host
do the same is not specified. No other return code is a failure: `126`
raises ERROR, and so does a negative one, which is not the failure of classic
Rexx here. The condition is raised at the end of the clause, and the record
has the return code in `RC`, the command string as its description, and `.RS`
is `-1`. A trap for ERROR does not answer a FAILURE as such, so it needs the
fallback of [the next rule](r:cond.error.fallback), and a trap for FAILURE
does not answer an ERROR.

```rexx
address system
call on failure name f
signal on error name e
'exit 127'
say 'after' rc .rs
address nosuch
'exit 0'
say 'after env' rc .rs
address system
'exit 126'
say 'not reached'
f: say 'F' rc '['condition('d')']' condition('o')~rc sigl,
     condition('c')
  return
e: say 'E' rc .rs condition('c')
```

```output
F 127 [exit 127] 127 4 FAILURE
after 127 -1
F 30 [exit 0] 30 7 FAILURE
after env 30 -1
E 126 1 ERROR
```

```rexx
address bash
call on failure name f
call on error name e
'exit 127'
address sh 'exit 127'
address command 'exit 127'
address path 'nosuchprogram_interrexx_spec'
'kill -9 $$'
exit
f: say 'F' rc .rs; return
e: say 'E' rc .rs; return
```

```output
F 127 -1
F 127 -1
F 127 -1
F 127 -1
E -9 1
```

r[cond.error.fallback]
The FAILURE of a command that its own trap and the ANY trap do not answer is
answered as an ERROR by the ERROR trap, when that is on, as [the untrapped
rule](r:cond.index.untrapped) and [the choice rule](r:cond.traps.choice) also
say. The condition is then ERROR for everything that reads it:
`CONDITION('C')` answers `ERROR`, and the record holds the same return code
and command string. `.RS` stays `-1`, because the command failed. A FAILURE
trap that was switched off explicitly does not prevent this. An ANY trap that
is on answers the FAILURE under its own name, so the ERROR trap is reached
only when neither of the two is on. When no trap of the three is on, the
FAILURE is ignored and nothing else is raised. A FAILURE that the RAISE
instruction raised does not fall back: when its own trap and the ANY trap do
not answer it, it is ignored, and the ERROR trap does not see it, as [the
untrapped rule of RAISE](r:instr.raise.untrapped) says. The second example
shows both.

```rexx
address system
call on error name h
'exit 127'
call off failure
'exit 127'
call on failure name f
'exit 127'
say 'a' rc
call off error
call off failure
'exit 127'
say 'ignored' rc .rs
signal on any name any
signal on error name er
'exit 127'
any: say 'ANY' condition('c') rc
exit
er: say 'not reached'
h: say 'ERROR trap' condition('c') condition('i') condition('s') rc .rs
  return
f: say 'FAILURE trap' condition('c') rc; return
```

```output
ERROR trap ERROR CALL DELAY 127 -1
ERROR trap ERROR CALL DELAY 127 -1
FAILURE trap FAILURE 127
a 127
ignored 127 -1
ANY FAILURE 127
```

```rexx
address system
call on error name h
call r1
say 'raise failure ignored'
call r2
call r3
exit
r1: raise failure 5 return
r2: raise error 6 return
r3: 'exit 127'; return
h: say 'h' condition('c') rc; return
```

```output
raise failure ignored
h ERROR 6
h ERROR 127
```
