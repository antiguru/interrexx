---
element: EXIT
kind: instruction
syntax: "EXIT [expression [, expression]...]"
ansi: "6.3.2.60, 8.3.8"
rexxref: "Keyword Instructions > EXIT"
---

# EXIT

EXIT ends the program at once, however many routines, loops and INTERPRET
texts it is in, and the value of its expression is the program's value. In a
method or in a routine of a `::ROUTINE` directive it ends only that unit, as
RETURN would. This page specifies where EXIT stops, the end of the program as
an implied EXIT, what the expression is, and how the value of the main
program becomes the exit code of the process.

## Where it stops

r[instr.exit.program]
EXIT in the main program, or in an internal routine of it that a CALL or a
function call started, ends the program. No clause after the EXIT runs, the
routines that were being called are abandoned without returning, and a
function call that is still waiting for its value does not raise `SYNTAX
44.1`. An EXIT in the text of an INTERPRET ends the program too, as [the rule
on leaving the text](r:instr.interpret.transfer) says.

```rexx
say 'start'
call outer
say 'not reached'
exit 1
outer: procedure
  do i = 1 to 3
    say 'outer' i
    if i = 2 then call inner
  end
  return
inner:
  say 'inner'
  exit 9
```

```output rc=9
start
outer 1
outer 2
inner
```

```rexx
say 'start'
x = quit()
say 'not reached'
quit: exit
```

```output
start
```

r[instr.exit.unit]
EXIT in a method or in a routine that a `::ROUTINE` directive defines ends
that unit, including the internal routines it called, and hands the value to
the sender of the message or to the caller of the routine in the way that
[RETURN](r:instr.return.unit) does. The program goes on, so the main program
is not ended by it.

```rexx
say 'start'
r = .box~new~go
say 'main goes on' r
call early
say 'and on' result
exit 3
::class box
::method go
  call helper
  say 'not reached'
helper:
  exit 'from method'
::routine early
  exit 'from routine'
```

```output rc=3
start
main goes on from method
and on from routine
```

r[instr.exit.end]
When the program runs past its last clause, it ends as if there were an EXIT
without an expression, so the program has no value and the exit code is 0.
This holds when the last clause is in the body of a routine that a CALL
started, because the routine does not return from the end of the program;
the caller does not continue.

```rexx
say 'a'
call f
say 'not reached'
f: say 'in f'
```

```output
a
in f
```

## The expression

r[instr.exit.value]
The expression is evaluated when the EXIT runs, with the variables and the
settings of the code that holds it. An error in it is raised before the
program ends, and a trap for the condition handles it as it would anywhere
else. EXIT without an expression has no value. Several expressions separated
by commas give an Array, as [for RETURN](r:instr.return.list) does, so that a
method which exits with two of them answers an Array, and the main program
has an Array for its value.

```rexx
signal on syntax name e
x = 5
exit x * 'a'
e: say condition('o')~code x
```

```output
41.1 5
```

```rexx
say .box~new~pair~class~id
exit
::class box
::method pair
  exit 4, 5
```

```output
Array
```

## The exit code

r[instr.exit.code]
The value of the main program, which is that of its EXIT or of a RETURN that
ends it, becomes the exit code of the process in this way. A string that has
the form of a [whole-number argument](r:conv.whole.form), and that is a
whole number from -2147483648 to 2147483647, is handed to the host as that
integer. Blanks around the number, an exponent, and a decimal part of zeros
do not matter. An exit code from 0 to 255 is the exit status itself. What a
host makes of an integer outside 0 to 255 is not specified: Unix-like hosts
keep its low eight bits, so that 257 and -1 give 1 and 255, and another host
may report the integer as it is. Any other value gives the exit code 0 and
raises no error: no value, a fraction, a string that is no number, a whole
number outside the 32-bit range, and an object whose string value is none of
these. The book does not specify a value with more than 17 significant
digits.

```rexx
say 'done'
exit 5
```

```output rc=5
done
```

```rexx
exit 255
```

```output rc=255
```

```rexx
exit '  + 7  '
```

```output rc=7
```

```rexx
exit 1.5e1
```

```output rc=15
```

```rexx
exit 7.000
```

```output rc=7
```

```rexx
exit 7.5
```

```output
```

```rexx
exit 'seven'
```

```output
```

```rexx
exit 2147483649
```

```output
```

```rexx
exit 4294967297
```

```output
```

```rexx
exit
```

```output
```

## Errors

r[instr.exit.syntax]
A malformed expression in an EXIT raises the `SYNTAX` error of the
expression, `35.1` for an operator with nothing after it, when the program is
read, so no clause runs and the exit code is 221. The text of an INTERPRET
raises it when the INTERPRET runs.

```rexx
say 'before'
exit 1 +
```

```output rc=221
```

```rexx
signal on syntax name e
interpret 'say "first"; exit 1 +'
e: say condition('o')~code
```

```output
35.1
```
