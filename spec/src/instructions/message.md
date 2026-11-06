---
element: "Message instructions"
kind: concept
ansi: "6.3.2"
rexxref: "General Concepts > Clauses and Instructions > Message Instructions"
---

# Message instructions

A message instruction is a clause that sends a message to an object for what
the object does, where a message term in an expression is evaluated for what
it answers. The clause is a message term with nothing after it, or a message
term followed by `=` and an expression, which sets a value on the receiver.
This page specifies how the interpreter treats the answer of a message
instruction, how a message assignment is turned into a message, the order in
which its parts are evaluated, and the errors. The syntax of a message term
and the messages that strings and operators understand are in [the
expressions chapter](../expressions/messages.md), and the classes specify what
their methods do.

## The clause

r[instr.message.clause]
A clause is a message instruction when it is a single message term: a term
followed by one or more messages or subscripts, with no operator or other
term after the last of them. The term in front may be a symbol, a string, a
function call, a class name or an expression in parentheses. The message is
sent exactly as it is in an expression. A clause that is only a function
call, a string or an expression in parentheses is a [command](command.md), and
so is one with an operator after the message, because the whole expression
is then not a message term. A message instruction does not set `RC`, and a
command does not set `RESULT`.

```rexx
address system
o = .box~new
call t 'o~answer'
call t 'o~answer~~class'
call t '(o)~answer'
call t 'o[1]'
call t '(o~answer)'
call t "o~answer ''"
call t 'f(1)'
exit
t:
  drop result
  rc = 'unset'
  interpret arg(1)
  say left(arg(1), 16) symbol('result') rc
  return
f: return 'exit 0'
::class box
::method answer
  return 'exit 0'
::method '[]'
  return 'exit 0'
```

```output
o~answer         VAR unset
o~answer~~class  VAR unset
(o)~answer       VAR unset
o[1]             VAR unset
(o~answer)       LIT 0
o~answer ''      LIT 0
f(1)             LIT 0
```

## The answer

r[instr.message.result]
After a message instruction, `RESULT` holds the object that the message
answered, and it is dropped when the message answered nothing, so that the
earlier value is gone either way. A message instruction whose last operator is
`~~` answers its receiver whatever the message did, so `RESULT` holds the
receiver. A message to a string is a message instruction like any other, so
`'abc'~length` sets `RESULT` to 3.

```rexx
o = .box~new
result = 'old'
o~answer
say 'A' result
o~silent
say 'B' symbol('result') result
o~answer(1, 2)
say 'C' result
'abc'~length
say 'D' result
o~~answer
say 'E' result~class~id
o~answer~~length
say 'F' result
::class box
::method answer
  return 'ans' arg()
::method silent
  return
```

```output
A ans 0
B LIT RESULT
C ans 2
D 3
E BOX
F ans 0
```

## Message assignments

r[instr.message.assign]
The clause `term~name = expression` sends the message `name=`, which is the
name with an equal sign added, to the value of the term. The value of the
expression is the first argument. A name written as a string gets the equal
sign in the same way, and the case of the name does not matter. The clause
`term[i, j] = expression` sends the message `[]=` the same way, with the value
of the expression first and the values of the index expressions after it,
and the brackets may hold no expression at all. When the message has its own
arguments, as in `term~name(a) = expression`, the value comes first and the
arguments follow. The answer of the message is treated as for any message
instruction, so it is assigned to `RESULT`, or `RESULT` is dropped.

```rexx
o = .rec~new
result = 'old'
o~x = 5
say 'A' result
o~"X" = 6
o~y(7, 8) = 9
say 'B' symbol('result')
o[10, 11] = 12
o[] = 13
::class rec
::method 'x='
  say 'x= got' arg() arg(1)
  return 'ignored'
::method 'y='
  say 'y= got' arg() arg(1) arg(2) arg(3)
::method '[]='
  say '[]= got' arg() arg(1)'|'arg(2)'|'arg(3)
```

```output
x= got 1 5
A ignored
x= got 1 6
y= got 3 9 7 8
B LIT
[]= got 3 12|10|11
[]= got 1 13||
```

r[instr.message.assign.order]
In a message assignment the receiver term is evaluated first, then the
expression after the `=`, and then the argument expressions of the message
or of the brackets from left to right. This differs from the order of the
message term in an expression, where the arguments follow the receiver
directly.

```rexx
n(.rec~new)~z(n(1), n(2)) = n(3)
say '--'
n(.rec~new)[n(4), n(5)] = n(6)
exit
n: say 'eval' arg(1)~class~id arg(1); return arg(1)
::class rec
::method 'z='
  say 'z= got' arg(1) arg(2) arg(3)
::method '[]='
  say '[]= got' arg(1) arg(2) arg(3)
```

```output
eval REC a REC
eval String 3
eval String 1
eval String 2
z= got 3 1 2
--
eval REC a REC
eval String 6
eval String 4
eval String 5
[]= got 6 4 5
```

r[instr.message.assign.extended]
An extended assignment on a message term, such as `term~name += expression`
or `term[i] ||= expression`, sends two messages: the term is read with `name`
or `[]`, the operator is applied to the answer and the value of the
expression, and the result is set with `name=` or `[]=`. The receiver term is
evaluated twice, both times before the read. The argument expressions of a
subscript are evaluated once for the read and a second time, after the
expression, for the assignment, so a term or an argument that has an effect
has it twice.

```rexx
n(.rec~new)~x += n(1)
say '--'
n(.rec~new)[n(2)] ||= n(3)
exit
n: say 'eval' arg(1)~class~id arg(1); return arg(1)
::class rec
::method 'x='
  say 'x= got' arg(1)
::method 'x'
  say 'x read'; return 42
::method '[]='
  say '[]= got' arg(1)
::method '[]'
  say '[] read' arg(1); return 100
```

```output
eval REC a REC
eval REC a REC
x read
eval String 1
x= got 43
--
eval REC a REC
eval REC a REC
eval String 2
[] read 2
eval String 3
eval String 2
[]= got 1003
```

## Errors

r[instr.message.error]
A message instruction whose receiver does not understand the message raises
`SYNTAX 97.1`, and so does a message assignment, since it needs a method
whose name ends in an equal sign. A message assignment with no expression
after the `=` raises `SYNTAX 35.1`, not the `35.918` of an assignment to a
variable. A message operator with no name raises `SYNTAX 19.909`, as [the
expression rules](r:expr.index.syntax) say. A string understands no `[]=`
and no name that ends in an equal sign.

```rexx
o = .rec~new
call try "o~nosuch"
call try "o~nosuch = 1"
call try "o~x = "
call try "o~"
call try "'abc'~foo = 1"
call try "'abc'[1] = 1"
call try "o~x += 1"
exit
try:
  signal on syntax name e
  interpret arg(1)
e: say arg(1) '=>' condition('o')~code
  return
::class rec
::method 'x='
  nop
```

```output
o~nosuch => 97.1
o~nosuch = 1 => 97.1
o~x =  => 35.1
o~ => 19.909
'abc'~foo = 1 => 97.1
'abc'[1] = 1 => 97.1
o~x += 1 => 97.1
```
