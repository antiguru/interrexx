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
label of the code that holds the CALL, then a built-in function, then an
external routine. That code is the main program, or the body of the one
routine or method that the CALL is in, so the labels of the other parts of the
file are not searched. A label wins over a built-in function of the same name,
so a program can replace a built-in one. A name written as a string is used as
it is written and is not searched among the labels at all, so the search
starts with the built-in functions, whose names are in capitals, and goes on
to the external routines. A string that is not in capitals therefore never
reaches a built-in function, and a label of the same text is not found by it
either: with the labels `'myLab':` and `mylab:`, the string `'myLab'` finds
neither. The labels of a method are the ones that a CALL in it finds, as the
second example shows. How
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
call try '"myLab"'
call try '"MYLAB"'
call try '(n)'
call try 'mylab'
n = 'myLab'
call try '(n)'
exit
length: return 'label'
'myLab': return 'string label'
mylab: return 'symbol label'
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
"myLab" => 43.1
"MYLAB" => 43.1
(n) => 43.1
mylab => ok symbol label
(n) => ok string label
```

```rexx
.c~new~m
exit
mainlab: say 'main label'
::class c
::method m
  call inner 'x'
  say 'method' sigl result
  signal on syntax name e
  call mainlab
e: say 'method' condition('o')~code
  return
inner: return 'inner' arg(1)
```

```output
method 6 inner x
method 43.1
```

r[instr.call.expr]
`CALL (expression)` evaluates the expression first, and the arguments after
it, and uses the string the value converts to as the name, by [the conversion
rule](r:conv.string.object): the answer of its `MAKESTRING` method when it has
one and of its `STRING` method otherwise. The name is compared as it is, so
`GREET` matches a label in the example, `SUBSTR` the built-in function, and a
lower-case value matches neither. Unlike a name written as a string, the value
is searched among the labels first, so the order of [the search
rule](r:instr.call.search) holds for it.

```rexx
n = 'GREET'
call (n) 'world'
call ('SUBSTR') 'abcdef', 3, 2
say result
call try '"greet"'
call try 'n || "x"'
call (.z~new)
call (.y~new)
exit
greet: say 'hello' arg(1); return
cms: say 'makestring'; return
cst: say 'string'; return
try:
  signal on syntax name e
  interpret 'call (' || arg(1) || ') 1'
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
::class z
::method makestring
  return 'CMS'
::method string
  return 'CST'
::class y
::method string
  return 'CST'
```

```output
hello world
cd
"greet" => 43.1
n || "x" => 43.1
makestring
string
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

r[instr.call.restore]
The settings of an internal routine are its own. When NUMERIC settings, the
trace setting or the default environment of ADDRESS are changed inside the
routine, the caller has the ones it had before the CALL when the routine
returns, whether or not the routine starts with PROCEDURE. For NUMERIC, [the
scope rule](r:num.settings.call) says it, and for the trace setting [the
TRACE rule](r:bif.trace.scope) does. The same holds for the address
environment, which the ADDRESS page specifies, and the traps of conditions
that the conditions chapter specifies. The routine of the example traces, and
that trace output goes to standard error, so the output block shows only the
standard output.

```rexx
numeric digits 5
trace o
address foo
call s
say digits() 1/3 address()
exit
s: numeric digits 3; trace r; address bar
  return
```

```output
5 0.33333 FOO
```

r[instr.call.sigl]
A CALL of an internal routine sets `SIGL`, in the variable scope of the
caller, to the line number of the CALL clause, which is the line where the
clause starts when it runs over several lines. A routine that starts with
PROCEDURE has its own scope and sees `SIGL` only if it exposes it. A CALL of a
built-in function or of an external routine does not set it, and an external
routine has a scope of its own that holds no `SIGL`. [The SIGNAL
rule](r:instr.signal.sigl) says what SIGNAL sets.

```rexx
say 'start' sigl
call abs 1
say 'builtin' sigl
call ext
say 'external' sigl
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
::routine ext
  say 'in ext' symbol('sigl')
```

```output
start SIGL
builtin SIGL
in ext LIT
external SIGL
in sub 6
sub 6
in psub LIT
in esub 9
in sub 10
two lines 10
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
HALT or NOTREADY, or ANY, or USER followed by the name of a user condition, a
symbol read as a constant in upper case, not a string. ON may be followed by
`NAME` and a routine name, a symbol or a string, and the default name is the
name of the condition. The routine is not looked for when the trap is enabled,
so a CALL ON for a name that is not a routine is not an error then. What a
trap does when its condition occurs belongs to the chapter on conditions. The
conditions that only SIGNAL can trap are not accepted: NOVALUE, SYNTAX,
LOSTDIGITS, NOMETHOD and NOSTRING raise `SYNTAX 25.1` after ON and `25.2`
after OFF.

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
The errors of the ON and OFF forms are found when the program is read. A CALL
ON with no condition raises `SYNTAX 20.911` and an OFF `20.912`, and a USER
with no name, or with a string for the name, `20.915`. A word that is not a
condition raises `25.1` after ON and `25.2` after OFF. A NAME with no routine
name raises `19.3`, data after the name raises `21.903` on an ON, and any NAME
on an OFF raises `21.904`. A word after the condition that is not NAME raises
`25.914`.

```rexx
call try 'call on'
call try 'call off'
call try 'call on user'
call try 'call on user "foo"'
call try 'call off user "foo"'
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
call on user "foo" => 20.915
call off user "foo" => 20.915
call on syntax => 25.1
call off novalue => 25.2
call on foo => 25.1
call on error name => 19.3
call on error name lbl extra => 21.903
call off error name lbl => 21.904
call on error lbl => 25.914
```
