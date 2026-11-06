---
element: CALL
kind: instruction
syntax: "CALL {name | (expression)} [argument] [, [argument]]... | CALL ON condition [NAME label] | CALL OFF condition"
ansi: "6.3.2.37, 8.3.4"
rexxref: "Keyword Instructions > CALL"
---

# CALL

CALL runs a routine as a subroutine and goes on with the clause after it when
the routine returns. The routine is an internal one, which is a label of the
same program, a built-in function, or an external routine, and the name of the
CALL decides where the search for it looks. The routine may return a value,
which the CALL leaves in the variable `RESULT`. The ON and OFF forms do not
call anything: they switch the trap of a condition. This page specifies the
arguments, the search order as far as it separates internal routines from
built-in ones, `RESULT` and `SIGL`, and the errors. How a routine is found
outside the program belongs to the environment chapter, and what a trap does
when its condition occurs to the chapter on conditions.

## Running a routine

r[instr.call.run]
`CALL name` evaluates the arguments from left to right and runs the routine
with them. When the routine returns, control continues with the clause after
the CALL, whether the routine is an internal one, a built-in function or an
external routine. The routine reads its arguments with the [ARG built-in
function](r:bif.arg.args), the ARG and PARSE ARG instructions or USE ARG. A
symbol that is the name is a constant, so a variable of the same
name has no part in the call.

```rexx
hello = 'a variable'
call hello 'x', 1 + 1
call length 'abc'
say 'length gave' result
exit
hello: say 'hello' arg() arg(1) arg(2); return
```

```output
hello 2 x 2
length gave 3
```

r[instr.call.args]
The arguments are expressions separated by commas, and a position that has no
expression is an omitted argument, which the routine sees as absent. An
omitted argument in front of a given one counts as a position, so
`call f ,, 3` passes three arguments of which the first two are absent. An
omitted argument after the last given one is dropped and does not count, so
`call f 1,;` passes one argument. A comma at the end of a line is not a
separator but a continuation, as [the continuation
rule](r:lex.clauses.continuation.comma) says. A parenthesis that follows the
name directly or after a blank begins the first argument as a parenthesized
expression, and the CALL does not read the name as a function call, so
`call f(1)` and `call f (1)` both pass one argument, the value `1`.

```rexx
call show
call show 1,;
call show ,, 3
call show , 2
call show(1)
call show (1)
call show 1 2, 3
call show a,
  b
exit
show: say arg() '['arg(1)']' '['arg(2)']' 'present' arg(1, 'e') arg(2, 'e')
return
```

```output
0 [] [] present 0 0
1 [1] [] present 1 0
3 [] [] present 0 0
2 [] [2] present 0 1
1 [1] [] present 1 0
1 [1] [] present 1 0
2 [1 2] [3] present 1 1
1 [A B] [] present 1 0
```

## Which routine

r[instr.call.search]
A name written as a symbol is read in upper case and searched in this order: a
label of the program with that name, then a built-in function, then an
external routine. A label wins over a built-in function of the same name, so
a program can replace a built-in one. A name written as a string is used as it
is written and is not searched among the labels at all, so the search starts
with the built-in functions, whose names are in capitals, and goes on to the
external routines. A string that is not in capitals therefore never reaches a
built-in function, and a label of the same text is not found by it either. How
external routines are found is the subject of the environment chapter. A name
that nothing answers to raises `SYNTAX 43.1` when the CALL runs.

```rexx
call length 'abc'
say 'symbol:' result
call 'LENGTH' 'abc'
say 'string:' result
call ('LENGTH') 'abc'
say 'expression:' result
call try '"length"'
call try '("length")'
call try 'nosuch'
exit
length: return 'label'
try:
  signal on syntax name e
  interpret 'call' arg(1) '"abc"'
  say arg(1) '=> ok' result
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
symbol: label
string: 3
expression: label
"length" => 43.1
("length") => 43.1
nosuch => 43.1
```

r[instr.call.expr]
`CALL (expression)` evaluates the expression before any argument and uses its
string value as the name. The text is not changed to upper case, so it has to
match the label or the built-in function exactly: `GREET` matches a label in
the example, `SUBSTR` the built-in function, and a lower-case value matches
neither.
Unlike a name written as a string, the value is searched among the labels
first, so the order of [the search rule](r:instr.call.search) holds for it.

```rexx
n = 'GREET'
call (n) 'world'
call ('SUBSTR') 'abcdef', 3, 2
say result
call try '"greet"'
call try 'n || "x"'
exit
greet: say 'hello' arg(1); return
try:
  signal on syntax name e
  interpret 'call (' || arg(1) || ') 1'
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
hello world
cd
"greet" => 43.1
n || "x" => 43.1
```

## What the call leaves

r[instr.call.result]
When the routine returns a value, `RESULT` is set to it, and when the routine
returns no value, `RESULT` is dropped and is unset afterwards. This holds for
every kind of routine, so a built-in function that answers a value sets
`RESULT` like any other, and a routine that returns nothing drops the value
that an earlier call left. A function call in an expression does not set it.
Where a message instruction sets the variable, [the message
rule](r:instr.message.result) says.

```rexx
call giver 2
say 'giver:' result
call quiet
say 'quiet:' symbol('result')
result = 'old'
call abs -4
say 'abs:' result
result = 'old'
call quiet
say 'quiet:' symbol('result') result
result = 'kept'
x = giver(1)
say 'function:' result
exit
giver: return arg(1) * 21
quiet: return
```

```output
giver: 42
quiet: LIT
abs: 4
quiet: LIT RESULT
function: kept
```

r[instr.call.sigl]
A CALL of an internal routine sets `SIGL`, in the variable scope of the
caller, to the line number of the CALL clause, which is the line where the
clause starts when it runs over several lines. A routine that starts with
PROCEDURE has its own scope and sees `SIGL` only if it exposes it. A CALL of a
built-in function or of an external routine does not set it. [The SIGNAL
rule](r:instr.signal.sigl) says what SIGNAL sets.

```rexx
say 'start' sigl
call abs 1
say 'builtin' sigl
call sub
say 'sub' sigl
call psub
call esub
call sub 1,
  2
say 'two lines' sigl
exit
sub: say 'in sub' sigl; return
psub: procedure
  say 'in psub' symbol('sigl')
  return
esub: procedure expose sigl
  say 'in esub' sigl
  return
```

```output
start SIGL
builtin SIGL
in sub 4
sub 4
in psub LIT
in esub 7
in sub 8
two lines 8
```

## Errors

r[instr.call.error]
A CALL with no name raises `SYNTAX 19.2`, and a name written as a symbol with
a colon after it raises `20.922`, because the colon form is the call of a
routine of a namespace, which this chapter does not specify. Both are found
when the program is read, so none of its clauses run. A routine that is not
found raises `43.1` when the CALL runs, as the search rule says, and an error
that a built-in function or a routine raises is raised at the CALL. A
parenthesized name that is not closed raises `36.901`.

```rexx
say 'before'
call
```

```output rc=237
```

```rexx
call try 'call'
call try 'call hello:'
call try 'call abs'
call try 'call (n'
call try 'call nosuch 1'
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
call => 19.2
call hello: => 20.922
call abs => 40.3
call (n => 36.901
call nosuch 1 => 43.1
```

## ON and OFF

r[instr.call.on]
`CALL ON condition` enables the trap of a condition and `CALL OFF condition`
disables it, and neither calls a routine. The condition is ERROR, FAILURE,
HALT or NOTREADY, or ANY, or USER followed by the name of a user condition,
a symbol taken as written. ON may be followed by `NAME` and a routine name,
a symbol or a string, and the default name is the name of the condition. The
routine is not looked for when the trap is enabled, so a CALL ON for a name
that is not a routine is not an error then. What a trap does when its
condition occurs belongs to the chapter on conditions. The conditions that
only SIGNAL can trap are not accepted: NOVALUE, SYNTAX, LOSTDIGITS, NOMETHOD
and NOSTRING raise `SYNTAX 25.1` after ON and `25.2` after OFF.

```rexx
do c over .array~of('error', 'failure', 'halt', 'notready', 'any')
  interpret 'call on' c
  interpret 'call off' c
  say c 'on and off'
end
call on user foo name lbl
call off user foo
call on error name nolabel
call off error
say 'done'
exit
lbl: return
```

```output
error on and off
failure on and off
halt on and off
notready on and off
any on and off
done
```

r[instr.call.on.error]
The errors of the ON and OFF forms are found when the program is read. A
CALL ON with no condition raises `SYNTAX 20.911` and an OFF `20.912`, and a
USER with no name `20.915`. A word that is not a condition raises `25.1` after
ON and `25.2` after OFF. A NAME with no routine name raises `19.3`, data after
the name raises `21.903` on an ON, and any NAME on an OFF raises `21.904`. A
word after the condition that is not NAME raises `25.914`.

```rexx
call try 'call on'
call try 'call off'
call try 'call on user'
call try 'call on syntax'
call try 'call off novalue'
call try 'call on foo'
call try 'call on error name'
call try 'call on error name lbl extra'
call try 'call off error name lbl'
call try 'call on error lbl'
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
call on => 20.911
call off => 20.912
call on user => 20.915
call on syntax => 25.1
call off novalue => 25.2
call on foo => 25.1
call on error name => 19.3
call on error name lbl extra => 21.903
call off error name lbl => 21.904
call on error lbl => 25.914
```
