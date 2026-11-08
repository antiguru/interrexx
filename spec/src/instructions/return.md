---
element: RETURN
kind: instruction
syntax: "RETURN [expression [, expression]...]"
ansi: "6.3.2.80, 8.3.22"
rexxref: "Keyword Instructions > RETURN"
---

# RETURN

RETURN ends the routine, method or program that is running and hands an
optional value back to whoever started it. What the starter does with the
value depends on how the code was started: a CALL stores it in `RESULT`, a
function call uses it as the value of the call, a message answers it, and
for the main program it becomes the program's exit code. This page specifies
RETURN in each of these places, the list form with several expressions, and
the errors of the clause. [EXIT](exit.md) is the instruction that ends the
whole program from any depth, and the page of the exit code is that one.

## Returning from a routine

r[instr.return.routine]
RETURN in an internal routine, which is the code from a label that a CALL or a
function call started, ends the routine and continues with the caller, even
when the RETURN is nested inside DO, IF or SELECT constructs, which it
abandons. A RETURN with an expression hands its value to the caller, and a
CALL stores it in `RESULT` as [the CALL rule](r:instr.call.result) says. A
RETURN without an expression hands over no value, so a CALL drops `RESULT`,
and a function call that has no value raises `SYNTAX 44.1`, as [the rule for a
call in an expression](r:expr.terms.call.result) says. The caller's settings
come back as [the CALL rule](r:instr.call.restore) says.

```rexx
call find 6
say 'found' result
call find 99
say 'found' result
say 'direct' find(4) 'then' find(5)
result = 'old'
call quiet
say symbol('result')
signal on syntax name e
say quiet()
e: say condition('o')~code
exit
find: procedure
  arg target
  do i = 1 to 3
    do j = 1 to 3
      if i * j = target then return i'x'j
    end
  end
  return 'none'
quiet: return
```

```output
found 2x3
found none
direct 2x2 then none
LIT
44.1
```

r[instr.return.value]
The expression is evaluated before the routine is left, with the variables
and the settings that the routine has. When the routine started with
PROCEDURE, its variables are dropped only after the expression has been
evaluated, so the value may use them, and the caller's variable of the same
name is untouched.

```rexx
total = 'caller total'
say sum(3, 4) total
exit
sum: procedure
  total = arg(1) + arg(2)
  return total * 2
```

```output
14 caller total
```

## Returning from a program, a method or a routine directive

r[instr.return.program]
A RETURN in the code of the main program, outside any internal routine,
method or routine directive, ends the program, as [EXIT](exit.md) does, and
its value is the program's value, whose use as the exit code is [specified
with EXIT](r:instr.exit.code). This holds for a RETURN inside a
DO, IF or SELECT, and for a RETURN after a label that the program reached by
running into it, since no CALL started that code. A RETURN without an
expression gives no value and the exit code 0.

```rexx
say 'main'
helper:
say 'at helper'
return 7
say 'not reached'
```

```output rc=7
main
at helper
```

r[instr.return.unit]
RETURN in a method or in a routine that a [`::ROUTINE`](r:dir.routine.create)
directive defines ends that method or routine and hands the value to whoever
sent the message or made the call, whatever the program as a whole is doing. A
message instruction stores the value in `RESULT` as [the message
rule](r:instr.message.result) says, and a message that has no value and is
used as a term raises `SYNTAX 91.999`, as [the cascade
rule](r:expr.messages.cascade) says. A CALL of the routine stores the value
as for an internal routine.

```rexx
o = .box~new
o~note
say 'A' result
o~quiet
say 'B' symbol('result')
call twice 4
say 'C' result
say 'D' twice(5)
exit 2
::class box
::method note
  return 'noted'
::method quiet
  return
::routine twice
  return arg(1) * 2
```

```output rc=2
A noted
B LIT
C 8
D 10
```

r[instr.return.interpret]
A RETURN in the text of an INTERPRET returns from the routine, method or
program that holds the INTERPRET, as [the rule on leaving the
text](r:instr.interpret.transfer) says. The rest of the text and the clauses
after the INTERPRET do not run.

```rexx
say pick(1) pick(2)
exit
pick:
  interpret 'if arg(1) = 1 then return "one"'
  return 'other'
```

```output
one other
```

## Several expressions

r[instr.return.list]
RETURN may have several expressions separated by commas, and the value it
hands over is then a new Array that holds the values in order. An expression
that is left out leaves its position empty, so the Array counts it in its
size but not in its items. A comma at the end of a line is not a separator
but a continuation, as [the continuation
rule](r:lex.clauses.continuation.comma) says. For the main program the value
is an Array, which is not a number, so the exit code is 0.

```rexx
call pair 'x', 'y'
say result~class~id result~items result[1] result[2]
call gap
say result~items result~size
exit
pair: return arg(1), arg(2)
gap: return 1,,3
```

```output
Array 2 x y
2 3
```

## Errors

r[instr.return.syntax]
A malformed expression in a RETURN raises the `SYNTAX` error of the
expression, `35.1` for an operator with nothing after it. Like any fault of
the clause, it is found when the program is read, so no clause runs and the
exit code is 221, and the text of an INTERPRET raises it when the INTERPRET
runs.

```rexx
say 'before'
return 1 +
```

```output rc=221
```

```rexx
signal on syntax name e
interpret 'say "first"; return 1 +'
e: say condition('o')~code
```

```output
35.1
```
