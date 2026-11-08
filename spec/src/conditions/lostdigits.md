---
element: "LOSTDIGITS"
kind: concept
ansi: "8.4.1"
rexxref: "Conditions and Condition Traps; Conditions and Condition Traps > Condition Information > Descriptive Strings; Conditions and Condition Traps > Condition Information > Additional Object Information; Conditions and Condition Traps > Action Taken when a Condition Is Trapped"
---

# LOSTDIGITS

LOSTDIGITS is the condition of a number that has more digits than the
arithmetic can keep. NUMERIC DIGITS sets how many digits an operation works
with, and an operand written with more of them is cut before the operation
starts, so the program's result is not the one its text suggests. Nothing is
wrong enough to stop the program, and the condition is how a program that
cares about exact input finds out. The rules for which operands raise it, and
how the cut is made, belong to the chapter on numbers: [the arithmetic
rule](r:num.arithmetic.lostdigits) and [the comparison
rule](r:num.comparison.lostdigits). This page specifies what the condition
holds, which traps answer it and how far it reaches.

## What it holds

r[cond.lostdigits.information]
The description of a LOSTDIGITS condition is the operand that has too many
digits, and the condition is raised in the clause that holds the operation.
The condition object has the entries that every condition has and `POSITION`,
and none of `ADDITIONAL`, `RC` and `CODE`, as [the entries
rule](r:cond.information.object) lists them. The form in which the operand is
written in the description is the numbers chapter's to specify, in [the
arithmetic deviation](r:num.arithmetic.deviation.53) and in [the
comparison rule](r:num.comparison.lostdigits).

```rexx
numeric digits 4
signal on lostdigits name n
y = 98765 * 2
say 'not reached'
n: o = condition('o')
say condition('c') '['condition('d')']' condition('i') sigl condition('s')
say o~hasindex('POSITION') o~hasindex('ADDITIONAL') o~hasindex('CODE'),
    o~hasindex('RC')
signal on lostdigits name n2
say 1234 < 98765
n2: say 'comparison' '['condition('d')']'
```

```output
LOSTDIGITS [98765] SIGNAL 3 OFF
1 0 0 0
comparison [98765]
```

## Which traps answer it

r[cond.lostdigits.trap]
LOSTDIGITS is answered by SIGNAL ON LOSTDIGITS and by SIGNAL ON ANY, and by no
CALL trap: CALL ON LOSTDIGITS is `SYNTAX 25.1`, as [the CALL
rule](r:instr.call.on) says, and a CALL ON ANY trap does not see it, as [the
choice rule](r:cond.traps.choice) says. A SIGNAL trap ends the clause that
held the operation, so the clause assigns nothing. Whether a LOSTDIGITS that a trap is
to answer comes before the SYNTAX error that the same operation would raise
is [the arithmetic chapter's](r:num.arithmetic.deviation.102) to specify.

```rexx
numeric digits 4
call on any name h
x = 123456 + 1
say 'call any did not answer:' x
signal on lostdigits name n
y = 98765 * 2
say 'not reached'
n: say condition('c') '['y']'
numeric digits 5
signal on any name a
z = 12345678 - 1
a: say 'ANY' condition('c') condition('d')
signal on syntax name s
interpret 'call on lostdigits name h'
s: say condition('c') condition('o')~code
exit
h: say 'not called'
  return
```

```output
call any did not answer: 1.235E+5
LOSTDIGITS [Y]
ANY LOSTDIGITS 12345678
SYNTAX 25.1
```

## How far it reaches

r[cond.lostdigits.scope]
LOSTDIGITS is raised in the code that holds the operation, which counts the
digits against its own NUMERIC DIGITS, and goes no further. A trap in the
caller does not answer a LOSTDIGITS that a routine raised, as [the propagation
rule](r:cond.information.propagate.only) says, and the routine's own traps do.
When no trap answers, the operation goes on with the digits it has, as [the
arithmetic rule](r:num.arithmetic.lostdigits) says.

```rexx
signal on lostdigits name main
call r
say 'back from r'
exit
main: say 'main trap' condition('d')
::routine r
  numeric digits 3
  say 'r' 1234 + 0
  signal on lostdigits name mine
  say 'not reached' 5678 + 0
  mine: say 'r trap' condition('d')
  return
```

```output
r 1.23E+3
r trap 5678
back from r
```
