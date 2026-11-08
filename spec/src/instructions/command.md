---
element: "Commands"
kind: concept
ansi: "6.3.2.10, 8.3.5"
rexxref: "General Concepts > Clauses and Instructions > Commands; General Concepts > Commands to External Environments > Commands"
deviations: [40]
---

# Commands

A command hands work to something outside the program. The clause is a
bare expression, and the interpreter works out its value as a string and
sends that string to an environment, which runs it and answers with a return
code. This page specifies what is sent, where the return code
goes and what else a command leaves behind. Which clauses are commands is
decided by [the lexical rules](r:lex.clauses.command). Which environment is
current, how the ADDRESS instruction changes it and what the environments are
belong to the ADDRESS instruction and to the environment chapter, so
the examples here use the environment named `system`, which runs its
commands in the host's shell, and the name that [the ADDRESS
function](r:bif.address.name) reports is the one the ADDRESS instruction set.
What a command does is the business of the environment, and the examples need
of it only that `exit n` for a small `n` answers the return code `n` and that
`echo` writes its arguments.

## What is sent

r[instr.command.send]
The expression of a command is evaluated like any other, and its string value
is sent to the current environment. Symbols in it are replaced by their
values, and a symbol that has none contributes its own name in upper case, so
a word that is meant as it is must be a string. The whole value is one
command string: the blanks between its parts are the single blanks that the
expression joined them with. The string may be null, and a null string is sent
like any other.

```rexx
address system
verb = 'echo'
verb 'one' 1 + 1
'echo' 'two'
signal on failure name f
address nosuch
echo three
f: say condition('d')
```

```output
one 2
two
ECHO THREE
```

```rexx
address system
''
say 'A' rc .rs
address nosuch
signal on failure name f
''
f: say 'B' condition('c') rc .rs
```

```output
A 0 0
B FAILURE 30 -1
```

r[instr.command.string]
A value that is not a string is turned into one before it is sent, as [the
conversion rule](r:conv.string.object) says: an object with a `MAKESTRING`
method contributes its answer. A number is sent in the form that the
operation gave it, so `'exit' 2 + 3` sends `exit 5`.

```rexx
address system
'exit' 2 + 3
say rc
cmd = .cmd~new
cmd
say rc
::class cmd
::method makestring
  return 'exit 6'
```

```output
5
6
```

r[instr.command.error]
An error raised while the expression is evaluated ends the clause before
anything is sent. The error is the one the expression raises, and the
environment does not see a command.

```rexx
address system
say 'before'
'echo sent' + 1
```

```output rc=215
before
```

## The return code

r[instr.command.rc]
When the environment has run a command, the variable `RC` is assigned the
return code it answered, also when that is 0, and the new value replaces
whatever `RC` held, an assignment of the program included. Until the first
command or the first trapped condition that sets it, `RC` has no value. No
instruction other than a command assigns `RC` on its own account. A trapped
`SYNTAX`, `ERROR` or `FAILURE` condition sets it as [the error
rule](r:conv.error.name) and the conditions chapter say, so a trapped `SYNTAX`
error sets it to the error number.

```rexx
address system
say symbol('rc') rc
'exit 3'
say rc
'exit 0'
say rc
rc = 'mine'
x = 1
say rc
'exit 2'
say rc
.array~new~items
say rc
signal on syntax name s
x = 1 / 0
s: say rc
```

```output
LIT RC
3
0
mine
2
2
42
```

r[instr.command.scope]
`RC` is a variable like any other, so the command sets it in the variables of
the code that runs it. A routine that the program calls by its label, with no
PROCEDURE instruction, shares its variables with the caller, so the caller
sees the `RC` that the routine's commands set. A routine with PROCEDURE and a
method have variables of their own, so they leave the `RC` of their caller as
it was, and an INTERPRET instruction shares the variables of the code that
ran it.

```rexx
address system
'exit 1'
call plain
say 'A' rc
call hidden
say 'B' rc
interpret "'exit 4'"
say 'C' rc
say 'D' .o~new~go rc
exit
plain: 'exit 2'; return
hidden: procedure; 'exit 3'; say 'in hidden' rc; return
::class o
::method go
  'exit 5'
  return rc
```

```output
A 2
in hidden 3
B 2
C 4
D 5 4
```

r[instr.command.status]
The environment symbol `.RS` holds the status of the last command: `0` when
the command raised neither ERROR nor FAILURE, `1` when it raised ERROR and
`-1` when it raised FAILURE. It holds a value only after a command has run,
and before that it is an [environment symbol](r:lex.symbols.environment) that
finds no object, so its value is its own characters. Which return codes and
which environments raise the two conditions is part of
the conditions chapter and of the environment chapter. For the environment
`system`, `exit 3` is an error and a command that is not sent to an existing
environment is a failure.

```rexx
say .rs
address system
'exit 0'
say rc .rs
'exit 3'
say rc .rs
'exit 0'
say rc .rs
address nosuch
'exit 0'
say .rs
```

```output
.RS
0 0
3 1
0 0
-1
```

r[instr.command.status.scope]
`.RS` follows a different rule from `RC`. A routine that the program calls by
its label starts with the `.RS` of its caller, with or without PROCEDURE, and
the commands it runs change only its own `.RS`, so the caller's value is the
same on return, where a routine without PROCEDURE shares `RC`. A method, and a
routine of a `::routine` directive, starts with no `.RS` and sets its own. An
INTERPRET instruction shares the `.RS` of the code that ran it.

```rexx
address system
'exit 3'
say 'A' .rs rc
call hidden
say 'B' .rs rc
interpret "say 'I' .rs; 'exit 0'; say 'I' .rs"
say 'C' .rs rc
say 'D' .o~new~go
call plain
say 'E' .rs rc
exit
hidden: procedure
  say 'hidden' .rs rc
  'exit 5'
  say 'hidden' .rs rc
  return
plain:
  say 'plain' .rs rc
  address nosuch
  'exit 0'
  say 'plain' .rs rc
  return
::class o
::method go
  say 'method' .rs
  'exit 4'
  return .rs
```

```output
A 1 3
hidden 1 RC
hidden 1 5
B 1 3
I 1
I 0
C 0 0
method .RS
D 1
plain 0 0
plain -1 30
E 0 30
```

## Conditions

r[instr.command.conditions]
A command can raise the conditions ERROR and FAILURE, which a program traps
with `SIGNAL ON` or `CALL ON` as the conditions chapter specifies. When
neither is trapped, the clause ends and the program goes on with the next
clause, whatever the return code was.

```rexx
address system
'exit 3'
say 'went on' rc
signal on error name caught
'exit 4'
say 'not reached'
caught: say 'trapped' rc condition('c')
```

```output
went on 3
trapped 4 ERROR
```

## Deviations

r[instr.command.deviation.40]
The examples of this page write with `say` between commands and show their
output in program order. That holds because a standard output that is a pipe
or a file is handed over before a command is sent, as [Deviation
40](../deviations.md) says. A program that relies on the order of its own
output and a command's output in one stream gets it from the hand-over and
not from each line being written at once.

```rexx
address system
say 'one'
'echo two'
say 'three'
```

```output
one
two
three
```
