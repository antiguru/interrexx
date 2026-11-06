---
element: "Instructions"
kind: chapter
ansi: "6.3.2, 8.3"
rexxref: "General Concepts > Clauses and Instructions"
---

# Instructions

An instruction is a clause that does something when the interpreter reaches
it. A program is a sequence of clauses, and the interpreter runs them in
the order written until an instruction moves control elsewhere or the
program ends. This chapter specifies what each kind of instruction does. The
[lexical chapter](../lexical/index.md) already says where a clause ends and
how its first tokens decide what kind it is, and the
[expressions chapter](../expressions/index.md) says how the expressions
inside an instruction are evaluated, so this chapter takes both as given.

The pages of this chapter are:

* [Assignments](assignment.md): `symbol = expression` and the extended
  forms such as `+=`.
* [Message instructions](message.md): a message term used as a clause, and
  the assignments `term~name = expression` and `term[...] = expression`.
* [Commands](command.md): an expression sent to the current environment,
  and the return code that comes back.

[The keyword rule](r:instr.index.keywords) below lists the keyword
instructions, with a link on each name whose page exists. The lexical chapter
calls a keyword instruction simply an instruction, and this chapter says
keyword instruction wherever the other three kinds are also in play. The
directives, which start with two colons, are not instructions, and their own
chapter specifies them.

## Kinds of instruction

r[instr.index.kinds]
A clause that is not a null clause, a label or a directive is an
instruction of one of four kinds: an assignment, a message instruction, a
keyword instruction or a command. [The lexical
rule](r:lex.clauses.kinds) decides from the first tokens of a clause which of
the four it is, so a symbol followed by `=` is an assignment before any
keyword is looked at. This chapter adds the naming and the pages. A message
assignment, which [the message page](message.md) specifies, is a message
instruction like the lone message term, and what the lexical chapter calls an
instruction is the keyword instruction here. Because every clause that is
none of the other three is a command, a clause that holds only a function
call, a string, or a parenthesized expression is a command, and so is a
message term that an operator or another term follows.

```rexx
address system
o = .box~new
o~tell
say 'keyword'
'exit 2'
say rc
::class box
::method tell
  say 'told'
```

```output
told
keyword
2
```

r[instr.index.keywords]
The keyword instruction names are ADDRESS, ARG, [CALL](call.md), [DO](do.md),
[DROP](drop.md), EXIT, EXPOSE, FORWARD, GUARD, [IF](if.md),
[INTERPRET](interpret.md), [ITERATE](iterate.md), [LEAVE](leave.md),
[LOOP](loop.md), [NOP](nop.md), NUMERIC, [OPTIONS](options.md), PARSE,
PROCEDURE, PULL, PUSH, QUEUE, RAISE, REPLY, RETURN, [SAY](say.md),
[SELECT](select.md), [SIGNAL](signal.md), TRACE and USE. A clause that is not
an assignment and whose first token is one of these names is that instruction,
whatever the case of the name. A name that is not on the list starts no
keyword instruction. A clause that begins with such a name is a command, so
that the name is evaluated as an expression like any other symbol. The names
of directives and of builtin functions are not keywords: `endlocal` as a
clause is a command, and so are `class` and `method` without colons. The words
THEN, ELSE, END, WHEN and OTHERWISE are the exception, and [the next
rule](r:instr.index.parts) covers them.

```rexx
address nosuch
call try 'nop'
call try 'NoP'
call try 'trace n'
call try 'push 1'
call try 'endlocal'
call try 'class'
call try 'method'
exit
try:
  signal on failure name sent
  interpret arg(1)
  say arg(1) '=> instruction'
  return
sent: say arg(1) '=> command' condition('d')
  return
```

```output
nop => instruction
NoP => instruction
trace n => instruction
push 1 => instruction
endlocal => command ENDLOCAL
class => command CLASS
method => command METHOD
```

r[instr.index.parts]
The words THEN, ELSE, WHEN, OTHERWISE and END are not instructions: they are
parts of IF, SELECT and DO, and the pages of those instructions say where each
may stand. A clause that begins with one of them where its construct does not
expect it is not a command either. It raises a `SYNTAX` error: `8.1` for
THEN, `8.2` for ELSE, `9.1` for WHEN, `9.2` for OTHERWISE and `10.1` for END.
In a program file the error is found when the program is read, as [the
expression rules](r:expr.index.syntax) say, and the program writes nothing.

```rexx
call try 'then'
call try 'else'
call try 'when 1'
call try 'otherwise'
call try 'end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
then => 8.1
else => 8.2
when 1 => 9.1
otherwise => 9.2
end => 10.1
```
