---
element: "HALT"
kind: concept
ansi: "8.4.1"
rexxref: "Conditions and Condition Traps; Conditions and Condition Traps > Condition Information > Descriptive Strings; Conditions and Condition Traps > Action Taken when a Condition Is Trapped; Conditions and Condition Traps > Action Taken when a Condition Is Not Trapped"
---

# HALT

HALT is the condition of a request to stop the program from outside its own
flow of clauses. A user of a terminal asks for it with an interrupt key, and a
host program asks for it through an interface, and the interpreter raises it
between two clauses. A program that traps HALT can tidy up and end on its
own terms. This page specifies what a program sees of HALT, the effect of each
kind of trap and what happens when none is on. The interrupt keys, the
signals and the interfaces are of the host and not specified, so every example
on this page raises HALT with a RAISE instruction, which has its own page in
the instructions chapter.

## Raising it

r[cond.halt.raise]
HALT is raised by a request that the host makes, and by a RAISE instruction
that names it. Which host requests exist, and at which clause boundary one is
noticed, is not specified. The condition has no description unless the RAISE
instruction gave one with DESCRIPTION, so `CONDITION('D')` answers the null
string otherwise. Its condition object has the entries that every condition
has and `POSITION`, and neither `RC`, `CODE` nor `ADDITIONAL`, as [the entries
rule](r:cond.information.object) lists them. A HALT that the host raises is
not passed on to the caller of the code that was running, as [the propagation
rule](r:cond.information.propagate.only) says for every condition but SYNTAX.
Which level a RAISE instruction raises HALT in, the one that runs it or the
caller of that one, is specified with that instruction. The examples whose
trap answers HALT raise it with `RETURN`, which raises it in the caller.

```rexx
call on halt name h
call plain
call with
exit
h: o = arg(1)
  say 'HALT' condition('c') '['condition('d')']' condition('i'),
      condition('s'),
      o~hasindex('POSITION') o~hasindex('RC') o~hasindex('CODE'),
      o~hasindex('ADDITIONAL')
  return
plain: raise halt return
with: raise halt description 'asked to stop' return
```

```output
HALT HALT [] CALL DELAY 1 0 0 0
HALT HALT [asked to stop] CALL DELAY 1 0 0 0
```

## The traps

r[cond.halt.signal]
A SIGNAL trap for HALT ends the clause that was running and goes to the label,
ending the open DO and SELECT constructs as [the SIGNAL trap
rule](r:cond.traps.signal) says. A program uses it to leave a loop that has no
other end. The trap is `OFF` at the label, and a second HALT is untrapped
unless the program enables the trap again.

```rexx
signal on halt
n = 0
do forever
  n = n + 1
  call check n
end
halt: say 'stopped at' n '['condition('d')']' condition('c') condition('s')
::routine check
  use arg n
  if n = 3 then raise halt description 'enough' return
```

```output
stopped at 3 [enough] HALT OFF
```

```rexx
n = 0
signal on halt name h
call stop
say 'not reached'
h: n = n + 1
say 'halt' n
if n = 1 then signal on halt name h
call stop
say 'not reached either'
::routine stop
  raise halt return
```

```output rc=252
halt 1
halt 2
```

r[cond.halt.call]
A CALL trap for HALT lets the clause that was running finish, calls its
routine, and goes on with the next clause when the routine returns. The state
of the trap is `DELAY` in the routine, as [the delay rule](r:cond.traps.delay)
says. A program that wants to stop in the routine ends it with
EXIT.

```rexx
call on halt name h
say 'one'
call stop 'first'
say 'two'
call stop 'second'
say 'three'
exit
h: say 'handler' condition('d') condition('s')
  if condition('d') = 'second' then exit
  return
stop: d = arg(1)
  raise halt description (d) return
```

```output
one
handler first DELAY
two
handler second DELAY
```

## When nothing traps it

r[cond.halt.untrapped]
A HALT that neither its own trap nor the ANY trap answers is not ignored. As
[the untrapped rule](r:cond.index.untrapped) says, it becomes the SYNTAX
condition `4.1` at the same clause, which a SYNTAX trap sees under the name
SYNTAX, and [the SYNTAX rule](r:cond.index.syntax) says what follows when none
answers. That condition has `CODE` `4.1`, `RC` 4 and no description, whatever
description the RAISE instruction gave.

```rexx
say 'before'
raise halt
say 'not reached'
```

```output rc=252
before
```

```rexx
signal on syntax name s
raise halt description 'ignored text'
s: o = condition('o')
say condition('c') o~code o~rc '['condition('d')']'
```

```output
SYNTAX 4.1 4 []
```
