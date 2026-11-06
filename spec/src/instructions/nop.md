---
element: NOP
kind: instruction
syntax: "NOP"
ansi: "6.3.2.65, 8.3.14"
rexxref: "Keyword Instructions > NOP"
---

# NOP

NOP is the instruction that does nothing. It exists for the places where the
language needs an instruction and the program has nothing to do, such as a
branch of an IF or a SELECT that is meant to stay empty. It takes no operand,
and any data after the keyword is an error.

## What it does

r[instr.nop.effect]
NOP changes no variable and no special variable, writes nothing and sends
nothing to an environment. A clause after it runs as if the NOP were absent,
and `RESULT` and `RC` keep the values that they had. A NOP that stands as the
body of a THEN, an ELSE, a WHEN or an OTHERWISE gives that branch an
instruction that does nothing.

```rexx
result = 'r0'; rc = 'c0'
call giver
nop
say result rc
if 1 then nop
else say 'not run'
if 0 then nop; else say 'else run'
select
  when 0 then nop
  otherwise nop
end
do 2; nop; end
say 'done'
exit
giver: return 'given'
```

```output
given c0
else run
done
```

r[instr.nop.keyword]
The word NOP starts the instruction in any case, and the clause ends where
the instruction does, so a semicolon, a line end or a comment may follow it
and another clause may follow on the same line. A clause that begins with
`nop =` is an assignment to a variable of that name, and one that begins with
`nop.x =` is an assignment to a compound variable, as [the kind
rule](r:lex.clauses.kinds) says.

```rexx
Nop; NOP /* comment */ ; nop -- more
nop = 'a variable'
say nop
nop.x = 'a compound'
say nop.x
```

```output
a variable
a compound
```

## Errors

r[instr.nop.error]
Anything after NOP in its clause, other than the end of the clause, raises
`SYNTAX 21.901`: a symbol, a number, a string, a parenthesis or a keyword
such as THEN. In a program file this is found when the program is read, so no
clause runs, not even one before the NOP, and the exit code is 235, which is
256 minus 21. In the text of INTERPRET it is raised when the clause is
translated.

```rexx
say 'before'
nop 1
```

```output rc=235
```

```rexx
call try 'nop 1'
call try 'nop x'
call try 'nop "s"'
call try 'nop(1)'
call try 'nop then'
call try 'nop end'
call try 'nop'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say 'ok' arg(1)
  return
e: say condition('o')~code '<=' arg(1)
  return
```

```output
21.901 <= nop 1
21.901 <= nop x
21.901 <= nop "s"
21.901 <= nop(1)
21.901 <= nop then
21.901 <= nop end
ok nop
```
