---
element: IF
kind: instruction
syntax: "IF expression [;] THEN [;] instruction [[;] ELSE [;] instruction]"
ansi: "6.3.2.14, 8.3.9"
rexxref: "Keyword Instructions > IF"
deviations: [39]
---

# IF

IF runs one instruction when a test is true and, when the program says so,
another when it is false. The test is an expression whose value must be
`0` or `1`, and each branch is a single instruction, which is a DO group when
a branch needs several. The keywords THEN and ELSE end the clause they are in,
so a branch may follow on the same line, as [the lexical
rule](r:lex.clauses.implied) says. This page specifies how the test is read,
which ELSE pairs with which IF, what a branch may be, and the errors. The
[SELECT](select.md) page is the instruction for more than two branches.

## What it does

r[instr.if.run]
IF evaluates its expression once. When the value is `1`, the instruction after
THEN runs, and when it is `0`, the instruction after ELSE runs, if there is
one. Control then continues with the clause after the whole IF, which is after
the ELSE instruction when there is an ELSE. The branch is one instruction: a
second instruction on the same line or the next line is not part of it, so it
runs whatever the test said. A DO group, a SELECT or another IF is a single
instruction, so a branch that holds one of them can hold many clauses.
THEN need not stand in the clause of the expression: [the rule for
THEN](r:instr.if.then) says where it may begin.

```rexx
n = 3
if n > 2 then say 'big'; else say 'small'
if n > 5 then say 'huge'
else say 'not huge'
if n = 3 then say 'a'; say 'b'
if n = 4 then say 'c'; say 'd'
if n = 3 then do
  say 'e1'
  say 'e2'
end
else say 'no'
say 'end'
```

```output
big
not huge
a
b
d
e1
e2
end
```

r[instr.if.then]
THEN may begin a later clause than the expression. Any number of null clauses
may stand between the expression and THEN, so a line end, a semicolon, blank
lines and comments there do not end the IF, and a THEN on the next line
belongs to the IF above it. The same holds for the WHEN of a
[SELECT](r:instr.select.run). The null clauses after THEN are the subject of
[the rule for branches](r:instr.if.branch).

```rexx
if 1
then say 'a'
if 1; then say 'b'
if 1;; then say 'c'
if 1 /* c */

  /* c2 */
then say 'd'
```

```output
a
b
c
d
```

## The value

r[instr.if.value]
The value of the expression must be a string that is exactly `0` or exactly
`1`, with the rules that [the logical operators](r:expr.logical.value) give
for their operands: `01`, `1.0`, a blank around the digit, the null string and
every other text are refused. An IF that gets another value raises
`SYNTAX 34.1`, and no branch runs. A value that is an object and not a string
counts as the string it converts to, which [the conversion
rule](r:conv.string.object) gives: the answer of its `MAKESTRING` method when
it has one, and the answer of its `STRING` method otherwise. An object that
converts to `1` takes the THEN branch, and one that converts to any other
string, such as `.nil`, raises `34.1`. The test does not look at the number
form: `1 + 0` and `5 - 5` are logical because they come out as `1` and `0`,
and `01` is not logical because it is the text `01`.

```rexx
call try '1 + 0'
call try '5 - 5'
call try '2'
call try '"yes"'
call try '""'
call try '1.0'
call try '01'
call try '.nil'
call try '(1 = 1)'
call try '.probe~new'
call try '.both~new'
exit
try:
  signal on syntax name e
  interpret 'if' arg(1) 'then r = "then"; else r = "else"'
  say arg(1) '=>' r
  return
e: say arg(1) '=>' condition('o')~code
  return
::class probe
::method string
  return 1
::class both
::method makestring
  return 0
::method string
  return 1
```

```output
1 + 0 => then
5 - 5 => else
2 => 34.1
"yes" => 34.1
"" => 34.1
1.0 => 34.1
01 => 34.1
.nil => 34.1
(1 = 1) => then
.probe~new => then
.both~new => else
```

r[instr.if.list]
The expression can be a list of expressions separated by commas. The list is
true when every expression is, and the expressions are evaluated from left to
right until one is false, so the rest are not evaluated. The value of an
expression of the list that is evaluated must be `0` or `1`, and any other
value raises `SYNTAX 34.6`, which is the code for the list form, where a
single expression raises `34.1`. [WHILE](r:instr.do.while) takes the same
list.

```rexx
call try '1, f(1)'
call try '0, f(1)'
call try '1, f(0), f(1)'
call try '0, 5'
call try '1, 5'
call try '5, 1'
exit
f: say 'f('arg(1)')'; return arg(1)
try:
  signal on syntax name e
  interpret 'if' arg(1) 'then r = "then"; else r = "else"'
  say arg(1) '=>' r
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
f(1)
1, f(1) => then
0, f(1) => else
f(0)
1, f(0), f(1) => else
0, 5 => else
1, 5 => 34.6
5, 1 => 34.6
```

## The ELSE

r[instr.if.else]
An ELSE belongs to the nearest IF before it that has a THEN instruction and no
ELSE yet, so in a nested IF the first ELSE pairs with the inner IF and the
second with the outer one. The ELSE may be on the line of the THEN instruction
after a semicolon, or on a later line, with blank lines and comments between,
since the line end after the instruction is an implied semicolon. Nothing but
null clauses may stand between the THEN instruction and its ELSE: another
instruction there ends the IF, and the ELSE after it has no IF and raises
`SYNTAX 8.2`.

```rexx
y = 0
do x = 1 to 0 by -1
  if x then if y then say 'both'; else say 'x only'
  else say 'x false'
end
if 0 then say 'a'

/* a comment and a blank line do not end the IF */
else say 'b'
if 1 then say 'c';; else say 'd'
call try 'if 1 then say 1; nop; else say 2'
call try 'if 1 then say 1; else say 2; else say 3'
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
x only
x false
b
c
if 1 then say 1; nop; else say 2 => 8.2
if 1 then say 1; else say 2; else say 3 => 8.2
```

## What a branch is

r[instr.if.branch]
The null clauses after THEN or after ELSE are passed over, as [the null-clause
rule](r:lex.clauses.null) says, so `then;` followed by a line holds that
line's instruction. A branch with no instruction is an error, and the
do-nothing instruction is [NOP](nop.md). An ELSE directly after THEN, or after
THEN and null clauses, leaves the THEN without an instruction, so the ELSE has
no IF waiting for it and raises `SYNTAX 8.2`: `if 1 then ; else say 'f'`
raises `8.2` and the same IF with `nop` after THEN does not. The word ELSE
after THEN, with a `=` next to it, is a variable assignment, as [the lexical
rule](r:lex.clauses.assignment) says. A label cannot be a branch: [the label
rule](r:lex.clauses.label) gives its error.

```rexx
if 1 then
  ; say 'second line is the branch'
if 0 then nop; else ; say 'else branch'
if 1 then else = 5; say else
call try 'if 1 then ; else say "f"'
call try 'if 1 then nop; else say "f"'
call try 'if 1 then say 1;; else say 2'
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
second line is the branch
else branch
5
if 1 then ; else say "f" => 8.2
if 1 then nop; else say "f" => ok
1
if 1 then say 1;; else say 2 => ok
```

## Errors

r[instr.if.syntax]
Errors in the shape of an IF are found when the program is read, so a program
that has one runs none of its clauses. An IF with no THEN raises
`SYNTAX 18.1`. A THEN with no instruction after it raises `14.3`, and an ELSE
with no instruction `14.4`. A THEN that no IF waits for, an ELSE with no IF,
and an END or a WHEN where the instruction of a branch should be, raise the
codes of [a part out of place](r:instr.index.parts). An error in the
expression is raised when the clause runs, as it is for any expression, and
when a branch is not taken, its expression is not evaluated.

```rexx
say 'before'
if 1 say 'no then'
```

```output rc=238
```

```rexx
call try 'if 1'
call try 'if 1 then'
call try 'if 0 then nop; else'
call try 'then say 1'
call try 'else say 1'
call try 'if 1 then end'
call try 'if 1 then when 1'
call try 'if 0 then say 1/0; else say "other branch"'
call try 'if 1/0 then say 1'
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
if 1 => 18.1
if 1 then => 14.3
if 0 then nop; else => 14.4
then say 1 => 8.1
else say 1 => 8.2
if 1 then end => 10.1
if 1 then when 1 => 9.1
other branch
if 0 then say 1/0; else say "other branch" => ok
if 1/0 then say 1 => 42.3
```

## Deviations

r[instr.if.deviation.39]
A label spelled THEN that stands as a clause of its own after the expression
of an IF raises `SYNTAX 18.1`, because the IF is still waiting for its THEN.
ooRexx raises `35.1` at the colon. When the `then:` follows the expression in
the same clause, both raise `35.1`.

```rexx
call try 'if 1 then: say 1'
call try 'if 1; then: say 1'
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
if 1 then: say 1 => 35.1
if 1; then: say 1 => 18.1
```
