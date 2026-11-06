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

The keyword instructions are ADDRESS, ARG, CALL, DO, DROP, EXIT, EXPOSE,
FORWARD, GUARD, IF, INTERPRET, ITERATE, LEAVE, LOOP, NOP, NUMERIC, OPTIONS,
PARSE, PROCEDURE, PULL, PUSH, QUEUE, RAISE, REPLY, RETURN, SAY, SELECT,
SIGNAL, TRACE and USE. The words THEN, ELSE, WHEN, OTHERWISE and END are
not instructions: they are parts of IF, SELECT and DO. The directives, which
start with two colons, are not instructions either, and their own chapter
specifies them.

## Kinds of instruction

r[instr.index.kinds]
A clause that is not a null clause, a label or a directive is an
instruction of one of four kinds: an assignment, a message instruction, a
keyword instruction or a command. [The lexical
rule](r:lex.clauses.kinds) decides which from the first tokens of the clause,
so a symbol followed by `=` is an assignment before any keyword is looked at.
A clause that is a single message term, with nothing after it, is a message
instruction. A clause of the form `term~name = expression` or
`term[...] = expression` is a message instruction too, the one that [the
message page](message.md) calls a message assignment. A clause that begins
with a keyword and is not an assignment is a keyword instruction. Every
other clause is a command, which includes a clause that holds only a
function call, a string, or a parenthesized expression.

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
A clause that is not an assignment and whose first token is one of the
keyword instruction names is that instruction, whatever the case of the
name. A name that is not on that list starts no keyword instruction. A
clause that begins with such a name is a command, so that the name is
evaluated as an expression like any other symbol. The names of directives
and of builtin functions are not keywords: `endlocal` as a clause is a
command, and so are `class` and `method` without colons.

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
