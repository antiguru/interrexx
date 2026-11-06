---
element: "Assignments"
kind: concept
ansi: "6.3.2.8, 8.3.3"
rexxref: "General Concepts > Clauses and Instructions > Assignments; General Concepts > Assignments and Symbols; General Concepts > Assignments and Symbols > Extended Assignments"
---

# Assignments

An assignment gives a variable a new value. Its form is a symbol, an equal
sign and an expression, and the symbol decides which variable changes: a
simple symbol names one variable, a stem symbol names a stem, and a compound
symbol names the variable that its tail works out to. This page specifies
what the assignment does with the value, the order in which its parts are
evaluated, what it does to a stem, the extended forms that combine an
operator with the assignment, and the errors it raises. How a clause is
recognized as an assignment is the business of [the lexical
rules](r:lex.clauses.assignment), and the variables themselves are specified
in [the expressions chapter](../expressions/variables.md).

## The value

r[instr.assignment.value]
An assignment evaluates its expression and makes the result the value of the
variable. The expression is everything after the first `=`, so a further `=`
in it is a comparison. The variable holds the object that the expression
answered and not a copy: two variables assigned from one object hold the same
object. A symbol with no value that stands alone as the expression has its
name in upper case as value, as [the variable rule](r:expr.variables.value)
says, and the assignment gives that to the variable.

```rexx
a = 3; b = a = 3; c = (a = 4)
say b c
o = .array~new; p = o; p~append('x')
say o~items (o == p)
y = unknown_name
say y
```

```output
1 0
1 1
UNKNOWN_NAME
```

r[instr.assignment.order]
The expression is evaluated before the variable is chosen. For a compound
symbol this matters when evaluating the expression changes a variable that
its tail uses: the tail is read after the expression has run, so the
assignment goes to the variable that the changed value names. The terms of
the expression itself are evaluated as [the expression
rules](r:expr.index.order) say.

```rexx
i = 1
a.i = f()
say a.1 a.2
i = 1
b.i.i = f()
say symbol('b.1.1') symbol('b.2.2')
exit
f: i = 2; say 'f called'; return 'v'
```

```output
f called
A.1 v
f called
LIT VAR
```

r[instr.assignment.error]
An error raised while the expression is evaluated ends the clause, and the
variable keeps the value it had: nothing is assigned, and the stem or the
compound variable that the target names is left as it was. The error is the
one the expression raises.

```rexx
x = 'old'; x.1 = 'keep'
call try 'x = 1 / 0'
call try 'x.1 = 1 / 0'
call try 'x. = 1 / 0'
call try 'x = .nosuch~foo'
say x x.1
exit
try:
  signal on syntax name e
  interpret arg(1)
e: say arg(1) '=>' condition('o')~code
  return
```

```output
x = 1 / 0 => 42.3
x.1 = 1 / 0 => 42.3
x. = 1 / 0 => 42.3
x = .nosuch~foo => 97.1
old keep
```

## Stems and compound variables

r[instr.assignment.stem]
An assignment to a stem symbol gives the stem a new default value, and every
compound variable of the stem that had its own value loses it, as [the stem
rule](r:expr.variables.stem) says. The expression is evaluated first, so it
can still read the compound variables that the assignment is about to
discard. The simple variable that has the stem's name does not change, as
[the separation rule](r:expr.variables.separate) says.

```rexx
s.1 = 'x'; s.2 = 'y'; s = 'simple'
s. = s.1 s.2
say s. s.1 s.2 s.3 s
```

```output
x y x y x y x y simple
```

r[instr.assignment.compound]
An assignment to a compound symbol sets the one variable that [its derived
name](r:expr.variables.compound) names. It leaves the default of the stem and
every other compound variable as they were, so [the default
rule](r:expr.variables.default) still gives the other tails the default.

```rexx
k = 'TWO'
f. = 'dflt'
f.1 = 'one'
f.k = 'second'
say f.1 f.2 f.two f.k f.three
```

```output
one dflt second second dflt
```

## Special names

r[instr.assignment.special]
The names that the interpreter sets itself, `RC`, `RESULT` and `SIGL`, are
ordinary variable symbols, and an assignment to one of them is an assignment
like any other. So are `TRUE` and `NIL`, which differ from the [environment
symbols](r:lex.symbols.environment) `.TRUE` and `.NIL` in the period. The
interpreter assigns the three special variables again when the event that
sets them happens, [as a command](r:instr.command.rc) sets `RC`.

```rexx
rc = 'mine'; result = 'mine'; sigl = 'mine'
say rc result sigl
true = 4
say true .true
```

```output
mine mine mine
4 1
```

r[instr.assignment.constant]
An assignment whose symbol is a constant symbol that starts with a digit
raises `SYNTAX 31.2`, and one whose symbol is an environment symbol, or a
constant symbol that starts with a period, raises `SYNTAX 31.3`, as [the
symbol rules](r:lex.symbols.constant) say. In a program file the fault is
found when the program is read, so that no clause runs, even one before the
assignment, and the exit code is 225, which is 256 minus 31. In the text of
INTERPRET the error is raised when the clause runs.

```rexx
say 'before'
exit
2b = 4
```

```output rc=225
```

```rexx
say 'before'
.widget = 4
```

```output rc=225
```

```rexx
call try '2b = 4'
call try '.widget = 4'
call try '1.5 = 4'
call try '.widget.name = 4'
exit
try:
  signal on syntax name e
  interpret arg(1)
e: say arg(1) '=>' condition('o')~code
  return
```

```output
2b = 4 => 31.2
.widget = 4 => 31.3
1.5 = 4 => 31.2
.widget.name = 4 => 31.3
```

r[instr.assignment.empty]
An assignment with no expression after the `=` raises `SYNTAX 35.918`, as
[the clause rule](r:lex.clauses.assignment) says. In a program file this is
found when the program is read, so the program writes nothing and exits with
221. An expression that cannot be read raises the error that [the expression
rules](r:expr.index.syntax) give, such as `35.1`, in the same way.

```rexx
say 'before'
x =
```

```output rc=221
```

```rexx
call try 'x ='
call try 'x +='
call try 'x = 1 +'
exit
try:
  signal on syntax name e
  interpret arg(1)
e: say arg(1) '=>' condition('o')~code
  return
```

```output
x = => 35.918
x += => 35.918
x = 1 + => 35.1
```

## Extended assignments

r[instr.assignment.extended]
In an extended assignment `target op= expression`, the clause is the plain
assignment `target = target op (expression)`, as [the clause
rule](r:lex.clauses.assignment.op) says, and the left operand is read before
the expression is evaluated. For a compound symbol the variable that is read
and the variable that is assigned are worked out separately: the first uses
the tail as it is before the expression runs, and the second uses it as the
expression left it.

```rexx
i = 1
a.1 = 10; a.2 = 20
a.i += f()
say a.1 a.2 i
exit
f: i = 2; return 1
```

```output
10 11 2
```

r[instr.assignment.extended.error]
An extended assignment raises what the operator raises for its operands, and
leaves the variable as it was. A variable that has no value contributes its
name, so `u += 1` on an unset variable raises `SYNTAX 41.1`, a zero divisor
raises `42.3` and a logical operator on a value that is not `0` or `1` raises
`34.901`. An empty expression raises `35.918`, as for a plain assignment.

```rexx
x = 5
call try 'u += 1'
call try 'x /= 0'
call try 'x %= 0'
call try 'x &= 2'
call try 'x ||= '
call try 'x -= "a"'
say x
exit
try:
  signal on syntax name e
  interpret arg(1)
e: say arg(1) '=>' condition('o')~code
  return
```

```output
u += 1 => 41.1
x /= 0 => 42.3
x %= 0 => 42.3
x &= 2 => 34.901
x ||=  => 35.918
x -= "a" => 41.1
5
```

r[instr.assignment.extended.stem]
When the target of an extended assignment is a stem symbol, the value of the
symbol is the left operand, and the result is assigned to the stem as any
assignment to a stem symbol does. The value is a `Stem` object, which passes
an operator message that it does not define on to its default value, so the
operator is in effect applied to that value and runs as that value's own
method when it has one. A stem that has no default has its name as the value
the operator meets, which no arithmetic operator accepts. A default that does
not understand the operator raises `97.1`.

```rexx
s. = 5
s.1 = 7
s. += 1
say s. s.1 s.2
signal on syntax name e
t. += 1
e: say condition('o')~code
```

```output
6 6 6
41.1
```

```rexx
s. = .t~new
s.1 = 'one'
s. += 1
say s. s.1
w. = .array~of(1)
signal on syntax name e
w. += 1
e: say condition('o')~code
::class t
::method '+'
  say 'plus called with' arg(1)
  return 'summed'
::method string
  return 'tstring'
```

```output
plus called with 1
summed summed
97.1
```
