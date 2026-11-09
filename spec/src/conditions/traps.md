---
element: "Condition traps"
kind: concept
ansi: "8.2.4, 8.4.1"
rexxref: "Conditions and Condition Traps > Action Taken when a Condition Is Trapped"
deviations: [64, 75, 104]
---

# Condition traps

A trap is the setting that tells the interpreter what to do when a condition
is raised, and there is one for each condition. A SIGNAL trap moves control
to a label as SIGNAL does and abandons the clause that raised the condition.
A CALL trap lets the clause finish and then calls a routine, which returns to
the clause after it. This page specifies which trap answers a condition, what
each kind of trap does, the state that keeps a CALL trap from calling itself,
and the code that a trap reaches. What the program learns about the condition
is on [the information page](information.md), and the syntax that sets a trap
is on the pages of [SIGNAL](../instructions/signal.md) and
[CALL](../instructions/call.md).

## Which trap answers

r[cond.traps.choice]
When a condition is raised, the trap of that condition answers it if its state
is `ON` or `DELAY`. When it is `OFF`, the ANY trap answers instead if its
state is `ON` or `DELAY`. When neither is on, the condition is
[untrapped](r:cond.index.untrapped), except that the FAILURE of a command then
goes to the ERROR trap, as [the fallback rule](r:cond.error.fallback) says. A
trap that answers a condition changes only its own state, so every other trap,
ANY included, stays as it was. The two kinds of trap are equal in this: a CALL
trap for the condition wins over a SIGNAL trap for ANY, and a SIGNAL trap for
the condition wins over a CALL trap for ANY. A condition that the ANY trap
catches is still itself: `CONDITION('C')` answers its own name, and the
default label or routine of an ANY trap is `ANY`. A condition whose own trap
was switched off explicitly does not keep ANY from answering. A CALL trap for
ANY answers only the conditions that CALL can trap, and a SIGNAL trap for ANY
answers all of them. A user condition is answered by the trap of its own name,
and the ANY trap catches the user conditions that have none.

```rexx
call on any name a
call on user foo name f
signal on novalue name nv
call r1
call r2
call r3
x = undefinedvar
exit
r1: raise user foo return
r2: raise user bar return
r3: raise halt return
a: say 'a' condition('c'); return
f: say 'f' condition('c'); return
nv: say 'nv' condition('c')
```

```output
f USER FOO
a USER BAR
a HALT
nv NOVALUE
```

```rexx
signal on any name a
call on user foo name f
call r1
call r2
exit
r1: raise user foo return
r2: raise user bar return
f: say 'f' condition('c'); return
a: say 'a' condition('c')
```

```output
f USER FOO
a USER BAR
```

```rexx
address system
call on any name a
x = undefinedvar
say 'ignored'
signal on any
signal off novalue
y = undefinedvar2
any: say 'any' condition('c') condition('s')
signal on any name other
z = 'a' + 'b'
other: say 'other' condition('c')
exit
a: say 'a' condition('c'); return
```

```output
ignored
any NOVALUE OFF
other SYNTAX
```

r[cond.traps.replace]
A trap has one setting at a time, and the latest SIGNAL ON, SIGNAL OFF, CALL
ON or CALL OFF for its condition is that setting: the state, the kind (SIGNAL
or CALL) and the name come from that instruction alone. A CALL ON and a
SIGNAL ON for the same condition therefore do not coexist, whichever came
first, and an ON with no NAME has the default name, not the name of an
earlier ON. The traps of other conditions are not touched.

```rexx
address system
call on error name viaCall
signal on error name viaSignal
'exit 1'
say 'not reached'
viaSignal: say 'viaSignal' condition('i')
call on error name viaCall
'exit 2'
say 'after 2'
call on error name other
call on error
'exit 3'
say 'after 3'
call off error
'exit 4'
say 'after 4'
exit
viaCall: say 'viaCall' rc condition('i'); return
other: say 'other'; return
error: say 'error' rc; return
```

```output
viaSignal SIGNAL
viaCall 2 CALL
after 2
error 3
after 3
after 4
```

## SIGNAL traps

r[cond.traps.signal]
When a SIGNAL trap answers a condition, the clause that raised it ends at
once, so that an assignment is not made, and a command whose string was being
evaluated is not sent. A command whose return code raised ERROR or FAILURE has
already run, and `RC` holds its return code. The state of the trap becomes
`OFF`, the condition is recorded as [the information
page](r:cond.information.record) says, and control goes to the label of the
trap as [SIGNAL](r:instr.signal.label) goes to a label, ending the open DO,
LOOP, SELECT and IF constructs of the routine as [that
rule](r:instr.signal.ends) says. The label is the one that the NAME option
gave, and the name of the condition when there was none. The trap is `OFF`
at the label, so a second condition of the same kind that the code there
raises is not trapped unless that code switches the trap on again or an ANY
trap is `ON`, which the answer did not change.

```rexx
signal on syntax
do i = 1 to 3
  say 'loop' i
  if i = 2 then x = 1 + 'a'
end
say 'not reached'
exit
syntax:
say 'trapped' i symbol('x') sigl rc condition('s')
x = 1 + 'b'
say 'not reached either'
```

```output rc=215
loop 1
loop 2
trapped 2 LIT 4 41 OFF
```

```rexx
address system
signal on error name e
'echo sent; exit 3'
say 'not reached'
e: say 'e' rc
signal on novalue name n
'echo' unsetvar
say 'not reached'
n: say 'n' condition('d')
```

```output
sent
e 3
n UNSETVAR
```

```rexx
signal on syntax name s
signal on any name a
x = 1 + 'a'
s: say 's' condition('c') condition('s')
y = 1 + 'b'
say 'not reached'
a: say 'a' condition('c') condition('s')
```

```output defect=#110
s SYNTAX OFF
a SYNTAX OFF
```

r[cond.traps.signal.label]
The label of a SIGNAL trap is looked for when the condition is raised, not
when the trap is switched on. When there is no such label among the labels of
the code that holds the trap, the trap, which is already `OFF`, raises `SYNTAX
16.1` at the same clause, and a SYNTAX trap that is `ON` catches that error
as any other. A routine or a method looks among its own labels, so a name
that is a label of the main program only is no label there.

```rexx
signal on syntax name e
signal on novalue name nolabel
x = undefinedvar
say 'not reached'
exit
e: say condition('o')~code sigl condition('c')
```

```output
16.1 3 SYNTAX
```

```rexx
signal on syntax name lbl
call r
exit
lbl: say 'main lbl' condition('o')~code sigl
::routine r
  signal on syntax name lbl
  x = 1 + 'a'
```

```output
main lbl 16.1 2
```

```rexx
say 'before'
signal on syntax name nolabel
x = 1 + 'a'
```

```output rc=240
before
```

## CALL traps

r[cond.traps.call]
When a CALL trap answers a condition, the clause that raised it finishes
first, so that a command has run and an assignment has been made. The state of
the trap becomes `DELAY`, the condition is recorded, and a routine is called
before the next clause starts. The next clause is the one after it in the same
place, so the call is made inside a DO, a SELECT or an IF that holds the
clause, and between the clauses of the text of an INTERPRET. When the clause
that raised the condition ends the program, as an EXIT in the main program or
the last clause does, the routine of the trap is called before the program
ends. When it is the RETURN of a routine, the routine of the trap is called
after the value of the RETURN is computed and before the caller goes on, and
it sees the variables of the routine that returned. The caller then goes on
with the clause after the CALL, or with the rest of the expression of the
clause that invoked the routine as a function. What an EXIT inside an internal
routine does with a pending CALL trap is
[a Deviation](r:cond.traps.deviation.104). The routine is the one
that [CALL](r:instr.call.search) finds for the name of the trap, which is the
name of the condition when there was no NAME. It gets one argument, the
[condition object](r:cond.information.argument). The call does not change
`RESULT`, and what the routine returns is dropped. When the routine returns,
the trap is `ON` again, and the clause after the one that raised the condition
runs. When the routine is an internal routine, `SIGL` is set as
[a CALL](r:instr.call.sigl) sets it, to the line of the clause that raised the
condition, which for a RETURN or an EXIT is that clause itself and not the
CALL. A routine without PROCEDURE shares the variables of the caller, `RC`
among them. When the routine is a `::ROUTINE`, a built-in function or an
external routine, `SIGL` is not set, and it keeps the value it had.

```rexx
address system
call on error name h
say 'start'
if rc = 'RC' then
  'exit 3'
say 'after if' rc sigl
call one
'exit 4'
say 'result' result
do 2
  select
    when rc = 4 then 'exit 5'
    otherwise nop
  end
  say 'in loop' rc
end
interpret "'exit 6'; say 'in text' rc; say 'text end'"
say 'after interpret'
exit
one: return 'kept'
h: say 'handler' arg() arg(1)~class condition('c') condition('i'),
    condition('s') rc sigl
  return 'dropped'
```

```output
start
handler 1 The Directory class ERROR CALL DELAY 3 5
after if 3 5
handler 1 The Directory class ERROR CALL DELAY 4 8
result kept
handler 1 The Directory class ERROR CALL DELAY 5 12
in loop 5
in loop 5
handler 1 The Directory class ERROR CALL DELAY 6 17
in text 6
text end
after interpret
```

```rexx
call on notready name h
say 'start' sigl
call sub
say 'main after' sigl result
exit
sub:
  say 'in sub'
  return '[' || linein() || ']'
h: say 'h' sigl condition('s')
  return
```

```output
start SIGL
in sub
h 8 DELAY
main after 8 []
```

```rexx
call on notready name h
x = f() g()
say 'after' sigl x
exit
f: return '[' || linein() || ']'
g: say 'g'; return 'G'
h: say 'h' sigl condition('s')
  return
```

```output
h 5 DELAY
g
after 2 [] G
```

```rexx
call on notready name h
exit linein()
h: say 'h' sigl condition('s')
  return
```

```output
h 2 DELAY
```

```rexx
address system
call on error name h
say 'before' sigl
'exit 3'
say 'after' sigl
exit
::routine h
  say 'in h' sigl
```

```output defect=#111
before SIGL
in h SIGL
after SIGL
```

```rexx
address system
call on error name length
say 'before' sigl
'exit 3'
say 'after' sigl
```

```output defect=#111
before SIGL
after SIGL
```

r[cond.traps.call.state]
A handler that a CALL trap calls is a routine like any other, so the changes
it makes to the states of the traps are undone when it returns: a CALL OFF in
it does not leave the trap `OFF`, and a CALL ON with another name does not
change the name. While it runs, the state of the trap that called it is
`DELAY`, as [CONDITION](r:bif.condition.status) shows.

```rexx
n = 0
call on user foo name h
call r
call r
call r
exit
r: raise user foo return
h: n = n + 1
  say 'h' n condition('s')
  if n = 1 then call off user foo
  if n = 2 then call on user foo name h2
  say 'leaving' condition('s')
  return
h2: say 'not called'
  return
```

```output
h 1 DELAY
leaving OFF
h 2 DELAY
leaving ON
h 3 DELAY
leaving DELAY
```

r[cond.traps.call.label]
When the routine of a CALL trap is not found, the call is an ordinary CALL of
a name that is no routine, and raises `SYNTAX 43.1` after the clause that
raised the condition has finished.

```rexx
call on error name nohandlerxyz
address system 'exit 3'
```

```output rc=213
```

## The delayed state

r[cond.traps.delay]
While the state of a CALL trap is `DELAY`, a condition that is raised for it
again does not call the routine. ERROR, FAILURE and NOTREADY raised again are
ignored. Every other condition that is raised is held, and the held ones are
answered in the order raised when the state of the trap changes. A CALL OFF
or SIGNAL OFF for the condition drops them. A CALL ON or SIGNAL ON for the
condition answers them at the end of that instruction, with the trap that the
instruction set. When the routine returns, the CALL trap answers them by
calling the routine again. A SIGNAL trap is never `DELAY`.
The held conditions are [Deviation 75](r:cond.traps.deviation.75).

```rexx
address system
call on error name h
'exit 3'
say 'main' rc
exit
h: say 'enter h' rc
  'exit 4'
  say 'in h after' rc condition('s')
  return
```

```output
enter h 3
in h after 4 DELAY
main 4
```

```rexx
n = 0
call on notready name h
x = linein()
say 'main after' n
x = linein()
say 'main after' n
exit
h: n = n + 1
  say 'enter h' n condition('s')
  y = linein()
  say 'leave h' n
  return
```

```output
enter h 1 DELAY
leave h 1
main after 1
enter h 2 DELAY
leave h 2
main after 2
```

```rexx
call on user foo name h
call r
say 'main after'
exit
r: raise user foo description 'first' return
h: say 'enter h' condition('d') condition('s')
  if condition('d') = 'first' then do
    call r2
    say 'held, before off'
    call off user foo
    say 'after off' condition('s')
  end
  say 'leave h' condition('d')
  return
r2: raise user foo description 'second' return
```

```output
enter h first DELAY
held, before off
after off OFF
leave h first
main after
```

## Where a trap reaches

r[cond.traps.scope]
The states of the traps belong to the code that runs, which is the main
program, a routine, a method or an internal routine, and each starts with
states of its own. An internal routine, which a CALL or a function call
reaches by its label, starts with a copy of the states of its caller, names,
kinds and `DELAY` included, and what it does to the copy is not seen by the
caller after it returns. A `::ROUTINE`, a method and a routine made from
other source text start with every trap `OFF`. The text of an
[INTERPRET](r:instr.interpret.scope) runs in the code that holds it and
shares its states, so a trap that the text switches on stays on after it.
Because an internal routine has a copy, a SIGNAL trap that answers a
condition in it jumps to a label of the code that holds the routine, and a
RETURN there returns from the routine, as [the SIGNAL
rule](r:instr.signal.routine) says.

```rexx
signal on syntax name h
call sub
say 'after call'
call psub
say 'after psub'
exit
sub:
  say 'in sub'
  x = 1 + 'a'
  say 'not reached'
  return
psub: procedure
  say 'in psub'
  x = 1 + 'a'
  return
h: say 'h' condition('s') sigl
  return
```

```output
in sub
h OFF 9
after call
in psub
h OFF 14
after psub
```

```rexx
address system
call on error name h
call inner
'exit 1'
say 'main 1' rc
call ext
say 'main 2'
say .o~new~go
say .routine~new('x', "'exit 7'; return 'made done'")~call
interpret "call off error; 'exit 2'; say 'in interpret'"
'exit 3'
say 'main 3'
call on error name h
interpret "'exit 4'"
exit
inner: procedure
  call off error
  'exit 9'
  say 'inner off'
  return
h: say 'h' rc condition('s'); return
::routine ext
  'exit 5'
  say 'ext done' condition('s')
::class o
::method go
  'exit 6'
  return 'method done'
```

```output
inner off
h 1 DELAY
main 1 1
ext done 
main 2
method done
made done
in interpret
main 3
h 4 DELAY
```

## Deviations

r[cond.traps.deviation.64]
When the CALL that a CALL trap makes fails, and a SYNTAX trap is `ON` in the
code that holds the CALL trap, that trap answers the SYNTAX condition at once,
before the next clause runs, and `SIGL` is the line of the clause that raised
the condition. The CALL fails when the routine is not found, which is `SYNTAX
43.1`, and when a SYNTAX error leaves the routine. In ooRexx the next clause
runs first, and `SIGL` is its line.

```rexx
address system
signal on syntax name s
call on error name nohandlerxyz
'exit 3'
say 'next clause'
exit
s: say 'syntax trap' condition('o')~code sigl
```

```output
syntax trap 43.1 4
```

```rexx
address system
signal on syntax name s
call on error name ext
'exit 3'
say 'next clause'
exit
s: say 's' condition('o')~code sigl condition('o')~position
::routine ext
  say 'ext'
  x = 1 + 'a'
```

```output
ext
s 41.1 4 10
```

r[cond.traps.deviation.75]
A condition that is raised for a CALL trap in the `DELAY` state, from a
routine that the handler calls, is held and answered as [the delay
rule](r:cond.traps.delay) says: the handler is called again when it returns,
or at once when a CALL ON or a SIGNAL ON for the condition runs, and several
such conditions are answered one after the other in the order raised. ooRexx
loses such a condition, and crashes in some programs.

```rexx
call on user foo name h
call r
say 'main after'
exit
r: raise user foo description 'first' return
h: say 'enter h' condition('d') condition('s')
  if condition('d') = 'first' then call r2
  say 'leave h' condition('d')
  return
r2: raise user foo description 'second' return
```

```output
enter h first DELAY
leave h first
enter h second DELAY
leave h second
main after
```

```rexx
call on user foo name h
call r
say 'main after'
exit
r: raise user foo description 'first' return
h: say 'enter h' condition('d') condition('s')
  if condition('d') = 'first' then do
    call r2
    say 'held, before on'
    call on user foo name h
    say 'after on'
  end
  say 'leave h' condition('d')
  return
r2: raise user foo description 'second' return
```

```output
enter h first DELAY
held, before on
enter h second DELAY
leave h second
after on
leave h first
main after
```

```rexx
call on user foo name h
call r
say 'main after'
exit
r: raise user foo description 'first' return
h: say 'enter h' condition('d') condition('s')
  if condition('d') = 'first' then do
    call r2
    say 'held, before signal on'
    signal on user foo name sg
    say 'not reached'
  end
  say 'leave h' condition('d')
  return
sg: say 'sg' condition('d') condition('i') condition('s')
  return
r2: raise user foo description 'second' return
```

```output
enter h first DELAY
held, before signal on
sg second SIGNAL OFF
main after
```

```rexx
call on user foo name h
call r 'first'
say 'main after'
exit
r: raise user foo description (arg(1)) return
h: say 'enter h' condition('d')
  if condition('d') = 'first' then do
    call r 'second'
    call r 'third'
  end
  say 'leave h' condition('d')
  return
```

```output
enter h first
leave h first
enter h second
leave h second
enter h third
leave h third
main after
```

r[cond.traps.deviation.104]
When an EXIT inside an internal routine ends the program while a CALL trap is
pending, the routine of the trap is called before the program ends, as for an
EXIT in the main program. ooRexx ends the program without calling it.

```rexx
call on notready name h
say 'start'
call r
say 'not reached'
exit
r: procedure
  exit linein('spec-cond-traps-nofile-xq7.tmp')
h: say 'handler' condition('c')
  return
```

```output
start
handler NOTREADY
```
