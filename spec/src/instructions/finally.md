---
element: FINALLY
kind: instruction
syntax: "DO ... [CATCH ...]... [FINALLY [instructions]] END [name]"
rexxref: "CATCH and FINALLY Blocks > FINALLY Clauses"
deviations: [179]
---

# FINALLY

FINALLY is a clause at the end of a DO, LOOP or SELECT block. The instructions
that follow it, up to the END of the block, form the FINALLY clause of the
block. This page specifies where a FINALLY may stand, which instructions its
clause may hold, and the errors that the program raises when it is read. The
blocks are those of [DO](do.md), [LOOP](loop.md) and [SELECT](select.md).

## Where it stands

r[instr.finally.form]
A FINALLY belongs to the innermost DO, LOOP or SELECT block that is open where
it stands, and a block has at most one. It comes after the instructions of the
block, and after its CATCH clauses when it has any, so it is the last clause
before the END. In a SELECT it follows the last WHEN or OTHERWISE and the
instructions of that branch. Like OTHERWISE, the keyword ends a clause of its
own, so an instruction may follow it on the same line without a semicolon, and
the FINALLY clause may be empty. The END of the block closes the FINALLY
clause with it: the END name and the error for a block without an END are
those of the block. A clause that starts with FINALLY and goes on with `=` or
`:` is an assignment or a label, as for any keyword
([the keyword rule](r:instr.index.keywords)).

```rexx
call try 'do; nop; finally; say 1; end'
call try 'do; nop; catch syntax; nop; finally say 1; end'
call try 'loop label lp i = 1 to 2; nop; finally; nop; end lp'
call try 'select; when 1 then nop; otherwise nop; finally; end'
call try 'select case 1; when 1 then nop; finally; nop; end'
call try 'do; finally; end'
call try 'finally = 6; say finally'
call try 'signal finally; finally: say 1'
call try 'do label a; nop; finally; nop; end b'
call try 'loop 2; nop; finally; nop'
exit
try:
  signal on syntax name e
  r = .routine~new('t', arg(1))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
do; nop; finally; say 1; end => ok
do; nop; catch syntax; nop; finally say 1; end => ok
loop label lp i = 1 to 2; nop; finally; nop; end lp => ok
select; when 1 then nop; otherwise nop; finally; end => ok
select case 1; when 1 then nop; finally; nop; end => ok
do; finally; end => ok
finally = 6; say finally => ok
signal finally; finally: say 1 => ok
do label a; nop; finally; nop; end b => 10.2
loop 2; nop; finally; nop => 14.5
```

r[instr.finally.errors]
A FINALLY with no DO, LOOP or SELECT open around it raises `SYNTAX 10.12`, one
that is the instruction of a THEN raises `10.13`, and one that is the
instruction of an ELSE raises `10.14`. A CATCH or a second FINALLY after the
FINALLY of a block raises `10.15` at that clause, naming the line of the
FINALLY. The FINALLY clause is part of its block for the other rules: a label
among its instructions raises `47.2`, in a SELECT as well, and a WHEN or an
OTHERWISE after the FINALLY of a SELECT belongs to no SELECT, so it raises
`9.1` or `9.2`. A SELECT whose FINALLY comes before any WHEN raises `7.1`.

```rexx
call try 'finally; nop'
call try 'do; if 1 then finally; end'
call try 'select; when 1 then finally; end'
call try 'do; if 1 then nop; else finally; end'
call try 'do; nop; finally; nop; catch syntax; end'
call try 'do; nop; finally; nop; finally; end'
call try 'do; nop; end; finally; nop'
call try 'do; nop; finally; l: nop; end'
call try 'select; when 1 then nop; finally; l: nop; end'
call try 'select; when 1 then nop; finally; when 2 then nop; end'
call try 'select; finally; nop; end'
exit
try:
  signal on syntax name e
  r = .routine~new('t', arg(1))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
finally; nop => 10.12
do; if 1 then finally; end => 10.13
select; when 1 then finally; end => 10.13
do; if 1 then nop; else finally; end => 10.14
do; nop; finally; nop; catch syntax; end => 10.15
do; nop; finally; nop; finally; end => 10.15
do; nop; end; finally; nop => 10.12
do; nop; finally; l: nop; end => 47.2
select; when 1 then nop; finally; l: nop; end => 47.2
select; when 1 then nop; finally; when 2 then nop; end => 9.1
select; finally; nop; end => 7.1
```

## What its clause may hold

r[instr.finally.static]
No instruction of a FINALLY clause may transfer control out of it, and one
that would raises `SYNTAX 99.951` where it stands, naming its keyword and the
line of the FINALLY. The rule covers every instruction between the FINALLY and
its END, inside the blocks and CATCH clauses nested there as well, and these
instructions break it: RETURN, EXIT, REPLY, SIGNAL to a label and SIGNAL
VALUE, RAISE with RETURN or EXIT, and FORWARD without CONTINUE. A LEAVE or an
ITERATE breaks it unless the block it selects is open inside the FINALLY
clause: one with no name selects the innermost repetitive loop, one with a
name the innermost block of that name ([the LEAVE
rule](r:instr.leave.target)), and the search ends at the FINALLY, so the loop
that owns the FINALLY is outside, and a LEAVE that would select no block at
all breaks the rule too. Where FINALLY clauses are nested, the line named is
that of the innermost one. SIGNAL ON and OFF, CALL, CALL ON, RAISE without
RETURN or EXIT, FORWARD CONTINUE and INTERPRET are allowed.

```rexx
call try 'do; nop; finally; return; end'
call try 'do; nop; finally; exit 1; end'
call try 'do; nop; finally; signal x; end; x: nop'
call try 'do; nop; finally; signal value "X"; end; x: nop'
call try 'do; nop; finally; raise user x return; end'
call try 'do; nop; finally; reply; end'
call try 'do; nop; finally; forward to (.nil); end'
call try 'do; nop; finally; do 2; if 1 then return; end; end'
call try 'loop 2; nop; finally; leave; end'
call try 'loop label l 2; nop; finally; iterate l; end'
call try 'do; nop; finally; leave; end'
call try 'do; nop; finally; leave zz; end'
call try 'do; nop; finally; loop 3; leave; end; end'
call try 'do; nop; finally; do label q; leave q; end; end'
call try 'do; nop; finally; do i = 1 to 2; leave i; end; end'
call try 'do; nop; finally; signal on syntax; call f; raise user x; end'
call try 'do; nop; finally; forward continue to (.nil); end'
exit
try:
  signal on syntax name e
  r = .routine~new('t', arg(1))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
do; nop; finally; return; end => 99.951
do; nop; finally; exit 1; end => 99.951
do; nop; finally; signal x; end; x: nop => 99.951
do; nop; finally; signal value "X"; end; x: nop => 99.951
do; nop; finally; raise user x return; end => 99.951
do; nop; finally; reply; end => 99.951
do; nop; finally; forward to (.nil); end => 99.951
do; nop; finally; do 2; if 1 then return; end; end => 99.951
loop 2; nop; finally; leave; end => 99.951
loop label l 2; nop; finally; iterate l; end => 99.951
do; nop; finally; leave; end => 99.951
do; nop; finally; leave zz; end => 99.951
do; nop; finally; loop 3; leave; end; end => ok
do; nop; finally; do label q; leave q; end; end => ok
do; nop; finally; do i = 1 to 2; leave i; end; end => ok
do; nop; finally; signal on syntax; call f; raise user x; end => ok
do; nop; finally; forward continue to (.nil); end => ok
```

r[instr.finally.forward]
A FORWARD without CONTINUE must not stand in a block that has a FINALLY, in its
instructions, in its CATCH clauses or in a block nested in either. The FINALLY
of such a block raises `SYNTAX 99.952` where it stands, naming the line of the
last such FORWARD before it. The FINALLY is reached before any instruction of
its own clause, so this error comes before a `99.951` in that clause. A FORWARD
before the block, or one with CONTINUE, is no fault.

```rexx
call try 'do; forward to (.nil); finally; nop; end'
call try 'do; nop; catch syntax; do; forward; end; finally; end'
call try 'select; when 1 then forward; finally; end'
call try 'do; forward; finally; return; end'
call try 'forward to (.nil); do; nop; finally; nop; end'
call try 'do; forward continue to (.nil); finally; nop; end'
exit
try:
  signal on syntax name e
  r = .routine~new('t', arg(1))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
do; forward to (.nil); finally; nop; end => 99.952
do; nop; catch syntax; do; forward; end; finally; end => 99.952
select; when 1 then forward; finally; end => 99.952
do; forward; finally; return; end => 99.952
forward to (.nil); do; nop; finally; nop; end => ok
do; forward continue to (.nil); finally; nop; end => ok
```

r[instr.finally.read]
The errors of this page are found when the program is read, as [the rule for a
program that is read](r:lex.index.read-first) says, so no clause of the program
runs, and the exit code is that of the error. In the text of an INTERPRET they
are raised as `SYNTAX` conditions when the INTERPRET runs.

```rexx
say 'before'
do
  nop
finally
  return
end
```

```output rc=157
```

## Deviations

r[instr.finally.deviation.179]
interrexx reads FINALLY clauses as this page says, but it does not run them:
execution that reaches a FINALLY ends the program with a message naming it
([Deviation 179](../deviations.md)). The program ends with exit code 120 and
writes the message to standard error. ooRexx runs the clause whenever control
leaves the block.

```rexx
say 'before'
do
  say 'in the block'
finally
  say 'in the FINALLY clause'
end
say 'after'
```

```output rc=120
before
in the block
```
