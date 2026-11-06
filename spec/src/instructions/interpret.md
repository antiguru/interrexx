---
element: INTERPRET
kind: instruction
syntax: "INTERPRET expression"
ansi: "6.3.2.62, 8.3.10"
rexxref: "Keyword Instructions > INTERPRET"
deviations: [51, 76]
---

# INTERPRET

INTERPRET evaluates an expression and runs the string that results as
instructions, at the place where the INTERPRET stands. The instructions of the
text are those of the clause's own routine, method or program: they read and
assign its variables, take its arguments and use its settings. The text is
program text without a file around it, so it has no lines and no labels, and
it must hold only whole constructs. This page specifies what the text may
hold, how it is run, how control leaves it and what errors it raises.

## Running the text

r[instr.interpret.run]
INTERPRET evaluates its expression, takes the string value of the result and
runs it as a sequence of clauses, from the first to the last. When the last
has run, the clause after the INTERPRET runs. The text may hold any
instruction, including another INTERPRET, and the instruction may be written
in any case, over any number of clauses separated by semicolons. The
expression is an ordinary [expression](r:expr.index.value), so a text is
often built from pieces.

```rexx
slot = 'total'
interpret slot '= 6 * 7'
say total
interpret 'p = 1; q = p + 1'
say p q
interpret 'say' 1 + 1
interpret 'do 2; say "loop"; end'
interpret 'interpret "z = 9"'
say z
```

```output
42
1 2
2
loop
loop
9
```

r[instr.interpret.empty]
A text that holds no clause, because it is the null string, blanks, one or
more comments or a bare semicolon, does nothing. An INTERPRET with no
expression after the keyword raises `SYNTAX 35.912`, which a program file
raises when it is read, so that it runs nothing and exits with 221. An error
in the expression is raised before any text is read.

```rexx
interpret ''
interpret '   '
interpret '/* only a comment */'
interpret '-- a line comment'
interpret ';'
say 'nothing happened'
signal on syntax name e
interpret 1 / 0
e: say condition('o')~code
call try 'interpret'
exit
try:
  signal on syntax name e2
  interpret arg(1)
  say 'ok' arg(1)
  return
e2: say condition('o')~code '<=' arg(1)
  return
```

```output
nothing happened
42.3
35.912 <= interpret
```

```rexx
say 'before'
interpret
```

```output rc=221
```

## Scope

r[instr.interpret.scope]
The text runs in the scope of the code that holds the INTERPRET. A variable
that the text sets is set for the code after the INTERPRET, a variable that
the text drops is dropped, and the text reads the arguments of the routine
through `ARG` and `PARSE ARG`. In a method, the text sees the object variables
that the method exposed and the method's `SELF`. A NUMERIC setting that the
text makes stays in force for the code after the INTERPRET, and so does
[an ADDRESS setting](r:bif.address.scope.interpret).

```rexx
numeric digits 12
interpret 'numeric digits 5; v = 1 / 3'
say digits() v
call sub 'one', 'two'
say digits() v
o = .box~new
o~go
exit
sub:
  interpret 'say arg(1) arg(2) arg()'
  interpret 'parse arg p, q; w = p || q; numeric digits 3'
  say p q w digits()
  interpret 'drop p'
  say symbol('p') symbol('w')
return
::class box
::method init
  expose n
  n = 5
::method go
  expose n
  interpret 'n = n + 1; say n self~class~id'
  say n
```

```output
5 0.33333
one two 2
one two onetwo 3
LIT VAR
5 0.33333
6 BOX
6
```

## Translation and running

r[instr.interpret.whole]
The text is read as a whole before its first clause runs. An error of the kind
that [a program file](r:lex.index.read-first) raises when it is read, such as
an expression that cannot be read, an unterminated string or comment, a label,
a directive or an unbalanced construct, is raised before any clause of the
text runs, so the clauses before the faulty one do nothing. An error that a
clause raises when it runs, such as a zero divisor or a missing routine, is
raised when that clause runs, and the clauses before it have run.

```rexx
call try 'say "first"; say 2 +'
call try 'say "first"; say "open'
call try 'say "first"; say 1 2 )'
call try 'say "first"; end'
call try 'say "first"; x = 1 / 0'
call try 'say "first"; call nosuchroutine'
call try 'say "first"; signal nolabel'
call try 'say "first"; if 2 then nop'
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
35.1 <= say "first"; say 2 +
6.3 <= say "first"; say "open
37.2 <= say "first"; say 1 2 )
10.1 <= say "first"; end
first
42.3 <= say "first"; x = 1 / 0
first
43.1 <= say "first"; call nosuchroutine
first
16.1 <= say "first"; signal nolabel
first
34.1 <= say "first"; if 2 then nop
```

r[instr.interpret.lines]
The text has no line ends. A line feed or a carriage return in it, which is
what [a program file](r:lex.index.read-first) would read as the end of a
line, raises `SYNTAX 13.1` when the text is read, and so does a result that
the conversion makes into several lines, such as an `Array`. A `--` comment
therefore runs to the end of the whole text and hides what follows it, even
after a semicolon, and a comma at the very end of the text is accepted.

```rexx
call try 'say 1' || '0a'x || 'say 2'
call try 'say 1' || '0d'x || 'say 2'
call try 'say 3 -- c; say 4'
call try 'say 5,'
call try 'interpret .array~of("say 6", "say 7")'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say 'ok' arg(1)
  return
e: say condition('o')~code
  return
```

```output
13.1
13.1
3
ok say 3 -- c; say 4
5
ok say 5,
13.1
```

r[instr.interpret.complete]
Every DO, IF and SELECT that the text opens must end inside it, and the text
may not hold the end of a construct that began outside it. A simple DO with no
END raises `SYNTAX 14.1`, a SELECT with no END `14.2`, and a loop with no END
`14.5`. An IF with no THEN raises `18.1`, a THEN with no instruction after it
`14.3` and an ELSE with none `14.4`. An END with no construct to end raises
`10.1`, including an END that would close a loop around the INTERPRET. A THEN, an ELSE, a WHEN and an OTHERWISE with no
IF or SELECT before it raise `8.1`, `8.2`, `9.1` and `9.2`, in that order.
Complete constructs run as they do in a program.

```rexx
interpret 'do 2; say "a"; end'
interpret 'if 1 then say "t"; else say "e"'
interpret 'select; when 0 then say 0; otherwise say "o"; end'
signal on syntax name outer
do 2
  interpret 'end'
end
outer: say condition('o')~code
call try 'do; say 1'
call try 'select; when 1 then say 1'
call try 'do 2'
call try 'do i = 1 to 2; say 1'
call try 'if 1'
call try 'if 1 then'
call try 'if 1 then say 1; else'
call try 'end'
call try 'then say 1'
call try 'else say 1'
call try 'when 1 then say 1'
call try 'otherwise say 1'
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
a
a
t
o
10.1
14.1 <= do; say 1
14.2 <= select; when 1 then say 1
14.5 <= do 2
14.5 <= do i = 1 to 2; say 1
18.1 <= if 1
14.3 <= if 1 then
14.4 <= if 1 then say 1; else
10.1 <= end
8.1 <= then say 1
8.2 <= else say 1
9.1 <= when 1 then say 1
9.2 <= otherwise say 1
```

r[instr.interpret.labels]
The text may not hold a label or a directive. A label raises `SYNTAX 47.1`, as
[the label rule](r:lex.clauses.label) says, and a clause that starts with `::`
raises `SYNTAX 99.914`, wherever in the text it stands. Both are raised before
any clause of the text runs.

```rexx
call try 'say "first"; lab: say 2'
call try 'lab:'
call try '::class foo'
call try 'say "first"; ::method m'
call try '::requires "x.rex"'
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
47.1 <= say "first"; lab: say 2
47.1 <= lab:
99.914 <= ::class foo
99.914 <= say "first"; ::method m
99.914 <= ::requires "x.rex"
```

## Leaving the text

r[instr.interpret.transfer]
An instruction of the text that moves control moves it for the code that holds
the INTERPRET. A `RETURN` returns from that routine or method with its value,
or ends the program, and the clauses after the INTERPRET do not run. An `EXIT`
ends the program with its value as the exit code. A `SIGNAL` to a label of the
program leaves the text and any active DO, IF or SELECT, and sets `SIGL` to
the number of the line of the INTERPRET clause, as it would for a SIGNAL
written there.

```rexx
say f(5)
call sub
say 'sigl' sigl result
interpret 'signal lab; say "no"'
say 'not run'
lab: say 'at lab' sigl
interpret 'exit 3'
say 'not run'
f: interpret 'return arg(1) * 2'
say 'not run'
sub: return 'from sub'
```

```output rc=3
10
sigl 2 from sub
at lab 4
```

r[instr.interpret.result]
INTERPRET does not set `RESULT` and does not drop it, and an instruction of
the text that sets or drops `RESULT`, such as a CALL, does it for the code
that holds the INTERPRET.

```rexx
result = 'keep'
interpret 'nop'
interpret 'z = 1'
say result
interpret 'call giver "x"'
say result
interpret 'call quiet'
say symbol('result')
exit
giver: return arg(1)'!'
quiet: return
```

```output
keep
x!
LIT
```

r[instr.interpret.loop]
A LEAVE or ITERATE in the text refers to a loop that the text itself holds. A
loop of the code around the INTERPRET is out of reach: the instruction raises
`SYNTAX 28.1` for a LEAVE and `28.2` for an ITERATE with no name, and `28.3`
and `28.4` when the instruction names a loop, even in the body of a loop. A
simple DO group is not a loop. A PROCEDURE in the text raises `SYNTAX 17.1`,
even when the INTERPRET is the first instruction of the routine, and an EXPOSE
raises `99.908` and a REPLY `99.924` when the text is read, so that no clause
of the text runs.

```rexx
signal on syntax name e1
do i = 1 to 3
  say 'loop' i
  interpret 'if i = 2 then leave'
end
e1: say condition('o')~code i
signal on syntax name e2
do j = 1 to 3
  say 'loop' j
  interpret 'if j = 2 then iterate j'
end
e2: say condition('o')~code j
signal on syntax name e3
call r1
e3: say condition('o')~code
exit
r1: interpret 'procedure expose x'
```

```output
loop 1
loop 2
28.1 2
loop 1
loop 2
28.4 2
17.1
```

```rexx
call try 'do 3; if 1 then leave; end; say "after leave"'
call try 'do k = 1 to 2; interpret "iterate k"; say "k" k; end'
call try 'do i = 1 to 2; interpret "leave i"; end'
call try 'do; leave; end'
call try 'iterate'
call try 'say "first"; expose x'
call try 'say "first"; reply 1'
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
after leave
ok do 3; if 1 then leave; end; say "after leave"
28.4 <= do k = 1 to 2; interpret "iterate k"; say "k" k; end
28.3 <= do i = 1 to 2; interpret "leave i"; end
28.1 <= do; leave; end
28.2 <= iterate
99.908 <= say "first"; expose x
99.924 <= say "first"; reply 1
```

## Errors

r[instr.interpret.error]
An error in the text is a `SYNTAX` condition raised at the INTERPRET clause.
`SIGL` is the number of the line on which that clause begins, whichever line
of the text the error is in and however many lines the clause spans. A
program that does not trap the condition ends with the exit code that the
error gives, as for any other error.

```rexx
signal on syntax name e1
interpret 'say 1; x = 1 / 0'
e1: say condition('o')~code sigl
signal on syntax name e2
interpret 'say "x"' ,
  'say 2 +'
e2: say condition('o')~code sigl
signal on syntax name e3
interpret 'say 1 2 )'
e3: say condition('o')~code sigl
```

```output
1
42.3 2
35.1 5
37.2 9
```

```rexx
say 'before'
interpret 'x = 1 / 0'
```

```output rc=214
before
```

## Deviations

r[instr.interpret.deviation.76]
The condition object of a `SYNTAX` error that arises while the text is
translated names the package that holds the INTERPRET. The object that its
`PACKAGE` method answers is the one that `.context~package` answers in the
code. In ooRexx the object is another package that has the same name.

```rexx
signal on syntax name e
interpret 'say 1 +'
e: c = condition('o')
say c~code (c~package == .context~package)
```

```output
35.1 1
```

r[instr.interpret.deviation.51]
Inside the text, `.RS` and `.CONTEXT~RS` answer the same value: the return
code that a command in the text has just set. In ooRexx the method answers
the value that the caller had before the INTERPRET until the text ends, and
`.RS` answers the new value at once.

```rexx
'true'
interpret '"false"; say .rs .context~rs'
say .rs .context~rs
```

```output
1 1
1 1
```
