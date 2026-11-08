---
element: "Conditions"
kind: chapter
ansi: "8.2.1, 8.4.1"
rexxref: "Conditions and Condition Traps; Conditions and Condition Traps > Action Taken when a Condition Is Not Trapped"
---

# Conditions

A condition is an event that a program can ask to be told about. The
interpreter raises one when a command fails, when an unset variable is read,
when an error is found in a running clause, when a stream cannot deliver, and
when the program asks for one itself. What happens next depends on the trap
for that condition: a program switches a trap on or off with the instructions
[SIGNAL](../instructions/signal.md) and [CALL](../instructions/call.md),
whose pages specify the syntax of those forms. This chapter specifies the
conditions, what a trap does when its condition is raised, and what a program
can learn about the condition afterwards.

The pages of this chapter are:

* [Condition traps](traps.md): which trap answers a condition, what SIGNAL
  ON and CALL ON each do, the delayed state, and how far a trap reaches.
* [Condition information](information.md): the condition object, the
  variables `RC` and `SIGL`, and how an error reaches the caller of the code
  that raised it.

The conditions themselves are specified one per page, named here in plain
text: SYNTAX, ERROR, FAILURE, HALT, NOVALUE, NOSTRING, NOMETHOD, NOTREADY,
LOSTDIGITS and the user conditions. The RAISE instruction, which raises a
condition on purpose, has a page of its own among the instructions. The
[CONDITION](../builtins/condition.md) builtin owns what a program reads back
from a trapped condition, and this chapter links its rules.

## The conditions

r[cond.index.names]
The conditions are named ANY, ERROR, FAILURE, HALT, LOSTDIGITS, NOMETHOD,
NOSTRING, NOTREADY, NOVALUE, SYNTAX and USER. A USER condition is the word
USER with a name, and it is a condition of its own, whatever its name: `user
error` is not ERROR, and an ERROR trap does not catch it. The form of the name
and which of the two instructions accepts which condition are in [the ON rule
of SIGNAL](r:instr.signal.on) and [that of CALL](r:instr.call.on). ANY is not
raised by anything. It is a trap that catches the others, as [the choice
rule](r:cond.traps.choice) says.

```rexx
call on user Foo name h1
call on user error name h2
call on any name h3
signal on error name er
call r1
call r2
call r3
exit
r1: raise user foo return
r2: raise user error return
r3: raise user baz return
h1: say 'h1' condition('c'); return
h2: say 'h2' condition('c'); return
h3: say 'h3' condition('c'); return
er: say 'ERROR trap'
```

```output
h1 USER FOO
h2 USER ERROR
h3 USER BAZ
```

## Trap states

r[cond.index.state]
Every condition has a trap with one of three states: `OFF`, `ON` and `DELAY`.
A program starts with every trap `OFF`, and the instructions SIGNAL ON, CALL
ON, SIGNAL OFF and CALL OFF set the state of the trap of their condition,
which is what the `S` option of [CONDITION](r:bif.condition.status) reads
back. `DELAY` is the state of a CALL trap while its handler runs, and [the
delay rule](r:cond.traps.delay) specifies it. The state of each condition is
its own, and the trap of ANY has a state of its own beside those of the
conditions it catches. A condition whose trap is `OFF`, and which no ANY trap
catches, is untrapped, and [the untrapped rule](r:cond.index.untrapped) says
what then happens.

```rexx
address system
x = undefinedvar
'exit 3'
say 'ignored' rc
signal on novalue name n
y = undefinedvar2
n: say 'trapped' condition('c') condition('s')
signal off novalue
z = undefinedvar3
say 'off again' condition('s')
```

```output
ignored 3
trapped NOVALUE OFF
off again OFF
```

## When nothing traps a condition

r[cond.index.untrapped]
When a condition is raised and neither its trap nor the ANY trap is `ON` or
`DELAY`, ERROR, FAILURE, LOSTDIGITS, NOSTRING, NOTREADY, NOVALUE and the user
conditions are ignored: the clause goes on as if nothing had happened, and
the trap stays `OFF`. The exception is a FAILURE that neither its trap nor
ANY answers: it is raised as ERROR, and the ERROR trap answers it when that
trap is `ON` or `DELAY`. HALT and NOMETHOD are not ignored. An untrapped HALT
raises `SYNTAX 4.1` instead, and an untrapped NOMETHOD `SYNTAX 97.1`, and
that SYNTAX condition is raised at the same place, so a SYNTAX trap catches
it with `CONDITION('C')` answering `SYNTAX`. An ANY trap catches HALT and
NOMETHOD before they would become SYNTAX, so the program sees their own names
then. SYNTAX itself is not ignored, and [the next rule](r:cond.index.syntax)
says what it does.

```rexx
address system
numeric digits 3
'exit 3'
address nosuch
'exit 3'
x = 12345 + 1
y = 'a' || .nostr~new
z = nosuchvar
w = linein()
call r
say 'ignored' x '['w']'
signal on syntax name s1
call rh
s1: say condition('c') condition('o')~code rc
signal on syntax name s2
x = .nil~nosuchmethod
s2: say condition('c') condition('o')~code rc
signal on any name a
call rh
a: say condition('c')
signal on any name b
x = .nil~nosuchmethod
b: say condition('c')
exit
r: raise user foo return
rh: raise halt return
::class nostr
::method makestring
  return .nil
```

```output
ignored 1.23E+4 []
SYNTAX 4.1 4
SYNTAX 97.1 97
HALT
NOMETHOD
```

```rexx
address nosuch
signal on error name e
'exit 3'
say 'not reached'
e: say 'e' condition('c') rc condition('d')
call on error name h
'exit 4'
say 'after' rc
exit
h: say 'h' condition('c') rc; return
```

```output
e ERROR 30 exit 3
h ERROR 30
after 30
```

r[cond.index.syntax]
A SYNTAX condition that no trap catches ends the clause in which it was
raised, and then every clause that is running a call, a function or a
message at that moment, back to the main program, as [the propagation
rule](r:cond.information.propagate) says. The program then ends. It writes a
report of the error to its standard error stream, whose text is not
specified, and its exit code is `256` minus the number of the error, so that
the error `97.1` gives 159 and `42.3` gives 214. The output that the clauses
before the error wrote stays written. The subcode does not enter the exit
code.

```rexx
say 'before'
x = .nil~nosuchmethod
say 'not reached'
```

```output rc=159
before
```

```rexx
say 'before'
call mid
say 'not reached'
exit
mid:
  say 'in mid'
  x = 1 % 0
  say 'not reached in mid'
```

```output rc=214
before
in mid
```

r[cond.index.read]
An error that is found while a program is read, before any of its clauses
runs, is not a condition that a trap can see, except in the text of an
INTERPRET instruction, where the same fault is an ordinary SYNTAX condition.
[The reading rule of the lexical chapter](r:lex.index.read-first) lists those
errors and gives the exit code.

```rexx
signal on syntax name e
say 'before'
interpret "say 'x'; say 1 #"
e: say condition('c') condition('o')~code
```

```output
before
SYNTAX 13.1
```
