---
element: "SYNTAX"
kind: concept
ansi: "8.4.1, 8.4.2, 9.2.4"
rexxref: "Conditions and Condition Traps; Conditions and Condition Traps > Condition Information > Descriptive Strings; Conditions and Condition Traps > Condition Information > Condition Object; Conditions and Condition Traps > Condition Information > The Special Variable RC"
---

# SYNTAX

SYNTAX is the condition of an error in the program as it runs. It is raised by
a clause that cannot be carried out: arithmetic on a value that is no number,
a call with a wrong argument, a routine that does not exist, and the other
faults that the error numbers describe. It is also the condition that HALT and
NOMETHOD become when nothing answers them, and it is never ignored: a trap
answers it, or the program ends. This page specifies what raises SYNTAX, what
identifies the error, which traps can answer it and what a program sees after
one has. The numbers of the errors are listed on the pages of the clauses that
raise them and in the error numbers chapter, and [the traps page](traps.md)
and [the information page](information.md) specify what any trap does and what
it records.

## What raises it

r[cond.syntax.raise]
An error that is found while a clause runs raises SYNTAX at that clause. The
clause ends at the point of the error, so that an assignment it would have
made is not made, a command it would have sent is not sent, and output that it
would have written after the point is not written, while the effects it had
before the point, such as the output of a function that the expression called,
stay. The error can come from an expression, from a builtin function or a
method, from a call that finds no routine, from the text of an INTERPRET
instruction, from a trap that has no label, and from a RAISE instruction that
names SYNTAX, as the last three examples show. A fault in the text of an
INTERPRET is found when that text is read as a whole, as [the whole-text
rule](r:instr.interpret.whole) says, so none of its clauses has run then. An
error found while the program itself is read is not a condition, as [the
reading rule](r:cond.index.read) says.

```rexx
total = 10
signal on syntax name s
say 'before'
total = noisy() + 'x'
say 'not reached'
s: say 'total' total condition('c') condition('o')~code
::routine noisy
  say 'noisy ran'
  return 5
```

```output
before
noisy ran
total 10 SYNTAX 41.1
```

```rexx
say 'start'
call try "total = total + 'x'"
call try "total = 9 // 0"
call try "total = nosuchroutinexyz(1)"
call try "total = abs()"
call try "total = substr('abc', 0)"
call try "total = 3 +"
call try "say 'seen'; total = 4 * 'y'; say 'late'"
call try "say 'fine'"
exit
::routine try
  use arg text
  signal on syntax name e
  interpret text
  say 'no error:' text
  return
e: o = condition('o')
  say left(text, 28) o~code o~rc
```

```output
start
total = total + 'x'          41.1 41
total = 9 // 0               42.3 42
total = nosuchroutinexyz(1)  43.1 43
total = abs()                40.3 40
total = substr('abc', 0)     93.924 93
total = 3 +                  35.1 35
seen
say 'seen'; total = 4 * 'y'; 41.1 41
fine
no error: say 'fine'
```

```rexx
address system
signal on syntax name s
'echo sent' 1 + 'a'
say 'not reached'
s: say 'syntax' condition('o')~code
```

```output
syntax 41.1
```

```rexx
address system
signal on syntax name s
signal on error name nolabel
'exit 3'
say 'not reached'
s: say 'syntax' condition('o')~code sigl
```

```output
syntax 16.1 4
```

```rexx
signal on syntax name s
raise syntax 93.900 additional ('boom')
say 'not reached'
s: o = condition('o')
say o~code o~rc
```

```output
93.900 93
```

## What identifies the error

r[cond.syntax.code]
An error has a number and a subcode. The number names the kind of error and
the subcode the particular fault, so errors of one kind share an `RC` and
differ in `CODE`, as [the values rule](r:cond.information.object.values) says
of the entries, and as [the RC rule](r:cond.information.rc) says of `RC` in
the code of a trap. A program tells the errors apart with `CODE`, never with
the text of the message, which is not specified.

```rexx
call try "x = abs()"
call try "x = abs(1, 2)"
call try "x = left('abc', -2)"
call try "x = copies('ab', -2)"
exit
::routine try
  use arg text
  signal on syntax name e
  interpret text
e: o = condition('o')
  say left(text, 22) o~code o~rc,
      (o~rc == word(translate(o~code, ' ', '.'), 1)),
      (o~errortext == errortext(o~rc))
```

```output
x = abs()              40.3 40 1 1
x = abs(1, 2)          40.4 40 1 1
x = left('abc', -2)    93.923 93 1 1
x = copies('ab', -2)   93.906 93 1 1
```

## Which traps answer it

r[cond.syntax.trap]
SYNTAX is answered by SIGNAL ON SYNTAX and by SIGNAL ON ANY, and by no CALL
trap: CALL ON SYNTAX is itself an error, as [the CALL ON
rule](r:instr.call.on) says, and a CALL ON ANY trap does not see a SYNTAX
condition, as [the choice rule](r:cond.traps.choice) says for the conditions
that CALL cannot trap. A SYNTAX that no SIGNAL trap answers ends the program
as [the untrapped rule](r:cond.index.syntax) says, whatever other traps were
on, and the exit code is the one of that rule.

```rexx
call on any name h
call on error name h
x = 1 + 'a'
say 'not reached'
h: say 'not called'
```

```output rc=215
```

```rexx
signal on any name a
x = 1 + 'a'
a: say 'ANY' condition('c') condition('o')~code
```

```output
ANY SYNTAX 41.1
```

```rexx
call on syntax name h
h: say 'not reached'
```

```output rc=231
```

## After the error

r[cond.syntax.handler]
The SIGNAL trap that answers a SYNTAX condition is `OFF` in the code that it
reaches, as [the SIGNAL trap rule](r:cond.traps.signal) says, so an error
that code makes before it enables the trap again is not answered by it. Unless
another trap is on, that second error is untrapped and ends the program with
the exit code of the second error. A handler that enables the
trap again at its start can catch the errors that follow.

```rexx
signal on syntax name s
say 'one'
x = 1 + 'a'
s: say 'caught' condition('o')~code condition('s')
y = 1 % 0
say 'not reached'
```

```output rc=214
one
caught 41.1 OFF
```

```rexx
signal on syntax name s
say 'one'
x = 1 + 'a'
s: say 'caught' condition('o')~code
signal on syntax name t
y = 2 // 0
say 'not reached'
t: say 'caught again' condition('o')~code
```

```output
one
caught 41.1
caught again 42.3
```
