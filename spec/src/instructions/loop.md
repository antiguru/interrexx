---
element: LOOP
kind: instruction
syntax: "LOOP [LABEL name] [COUNTER name] [repetitor] [conditional]; instructions; END [name]"
ansi: "6.3.2.12, 8.3.6"
rexxref: "Keyword Instructions > LOOP"
---

# LOOP

LOOP is the repetitive form of [DO](do.md) under a second keyword. Every
clause that follows the keyword has the same meaning that it has after DO,
and the block ends at the same END. The one difference is the clause that
has nothing after the keyword: a DO with nothing is a group that runs once,
and a LOOP with nothing is a loop that never ends by itself. This page states
that difference and the places where the keyword matters. The standard
defines DO only, so LOOP is an addition of ooRexx.

## What LOOP is

r[instr.loop.same]
Every form of DO that has a repetitor or a condition has the same meaning
under the keyword LOOP, and the phrases LABEL, COUNTER, TO, BY, FOR, OVER,
WITH, WHILE and UNTIL are the same. The two keywords can be mixed, since
`LOOP` and `DO` both start a block that ends at an END, and an END closes
the nearest block of either kind. The rules of the DO page for the evaluation
of the phrases, the tests, the control variable, the errors and the END
symbol hold for LOOP without change. The keyword is not case sensitive.

```rexx
loop 2; say 'count'; end
loop i = 1 to 2; say 'control' i; end
loop e over .array~of('p', 'q'); say 'over' e; end
loop counter c while c < 2; say 'while' c; end
loop until 1; say 'until'; end
do label a
  loop label b i = 1 to 3
    if i = 2 then leave a
    say 'mixed' i
  end b
end a
LOOP I = 1 TO 2; SAY 'upper' I; END i
signal on syntax name e1
interpret 'loop i = 1 to 2; end j'
e1: say condition('o')~code
```

```output
count
count
control 1
control 2
over p
over q
while 1
while 2
until
mixed 1
upper 1
upper 2
10.2
```

r[instr.loop.forever]
A LOOP with no repetitor and no condition is an endless loop, as
[DO FOREVER](r:instr.do.forever) is. A LABEL phrase alone does not make it a
group: `LOOP LABEL name` is endless. A COUNTER phrase counts the passes of
such a loop, since a LOOP with nothing but a COUNTER is allowed where a DO is
not. Only a LEAVE, an EXIT, a RETURN, a SIGNAL or an error ends it.

```rexx
n = 0
loop
  n = n + 1
  if n = 3 then leave
end
say n
loop label top
  n = n + 1
  if n > 4 then leave top
end top
say n
loop counter c
  if c = 3 then leave
end
say c
signal on syntax name e1
do
  leave
end
e1: say condition('o')~code
```

```output
3
5
3
28.1
```

r[instr.loop.keyword]
LOOP is a keyword only at the start of a clause that is not an assignment.
A symbol `loop` that an `=` follows is an assignment to a variable of that
name, a stem or compound symbol that starts with `loop` is a variable, and
`loop(...)` inside an expression is a call of a function of that name, as
[the keyword rule](r:lex.clauses.keyword) and [the list of
keywords](r:instr.index.keywords) say for every keyword. A LOOP clause with an
opening parenthesis straight after the keyword is a loop whose count is that
parenthesized expression.

```rexx
loop = 3
say loop
loop. = 4
say loop.x
loop (2)
  say 'paren'
end
signal on syntax name e1
say loop(1)
e1: say condition('o')~code
```

```output
3
4
paren
paren
43.1
```
