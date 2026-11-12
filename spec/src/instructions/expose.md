---
element: EXPOSE
kind: instruction
syntax: "EXPOSE [name | (name)]..."
rexxref: "Keyword Instructions > EXPOSE"
deviations: [70]
---

# EXPOSE

EXPOSE lets a method use the variables that belong to its object. A method
starts with no variables but its own, and the variables it sets are gone
when it returns. The names that EXPOSE lists are the object's: they keep
their values from one method call to the next, and every method of the
object that exposes the same name sees the same variable. This page
specifies which variables a name shares, the forms of the names, the one
place where EXPOSE may stand, and the errors. The stems and compound names
are those of [the variables chapter](r:expr.variables.stem), and the
forms of the names are those of [PROCEDURE](procedure.md), which does the
same for the routine of a caller.

## Object variables

r[instr.expose.object]
A name that EXPOSE lists is a variable of the object that received the
message, not of the method call. The method reads its value, an assignment
changes it, and [DROP](r:instr.drop.exposed) unsets it, all for the object,
so the effect outlasts the call and other methods and later calls see it. A
name that is not listed is a variable of the call: it starts unset, and it is
gone when the method returns. Two objects of one class have separate object
variables. A name that no method has set is unset in the object, and
a method that exposes it finds it unset.

```rexx
c = .counter~new
c~inc; c~inc
say c~get c~peek
d = .counter~new
say d~get
::class counter
::method init
  expose n
  n = 0
::method inc
  expose n
  n += 1
  scratch = 'gone'
::method get
  expose n
  return n
::method peek
  return symbol('n') symbol('scratch')
```

```output
2 LIT LIT
0
```

r[instr.expose.scope]
The object variables of one object are divided by the class that defines the
method. Methods of one class that expose a name share a variable, and a
method of a subclass that exposes the same name has a variable of its own,
for the same object. A method inherited from a class works on the variables
of that class.

```rexx
o = .sub~new
say o~setBase('base') o~setSub('sub')
say o~getBase o~getSub
::class base
::method setBase
  expose v
  v = arg(1)
  return 'set'
::method getBase
  expose v
  return v
::class sub subclass base
::method setSub
  expose v
  v = arg(1)
  return 'set'
::method getSub
  expose v
  return v
```

```output
set set
base sub
```

r[instr.expose.callee]
The variables of a method are visible to what the method runs in its own
code. The text of an INTERPRET sees the exposed names, and so does an
internal routine that a CALL starts without PROCEDURE. A routine that begins
with [PROCEDURE](r:instr.procedure.scope) sees none of them unless it
exposes them with PROCEDURE EXPOSE.

```rexx
o = .box~new
say o~run
say o~after
::class box
::method run
  expose a
  a = 'A0'
  interpret 'a = a || "+i"'
  call bump
  call hid
  call hid2
  return a
bump:
  a = a'+b'
  return
hid: procedure
  say symbol('a')
  return
hid2: procedure expose a
  say symbol('a')
  a = a'+h'
  return
::method after
  expose a
  return a
```

```output
LIT
VAR
A0+i+b+h
A0+i+b+h
```

## The names

r[instr.expose.stem]
A stem symbol, a name that ends in a period, exposes the whole stem of the
object: every compound variable of it, those that are set and those that
are set later, and the stem's default value. A compound variable such as
`b.1` that the method uses without exposing the stem is the method's own.

```rexx
o = .box~new
say o~setup
say o~look
::class box
::method setup
  expose b.
  b.1 = 'one'
  b. = 'dflt'
  b.2 = 'two'
  return 'set'
::method look
  expose b.
  return b.1 b.2 b.9
```

```output
set
dflt two dflt
```

r[instr.expose.compound]
A compound symbol exposes the one variable that it names. Its tail is
replaced by the value that the tail variable has when the EXPOSE runs. A
tail variable is the method's own, and it has the value of the object only
if the same clause has exposed it before. A later change of the tail variable
does not move the exposed name, and a compound name with another tail is a
variable of the method.

```rexx
o = .box~new
say o~setup
say o~run
say o~look
::class box
::method setup
  expose i a.
  i = 1; a.1 = 'one'; a.2 = 'two'
  return 'set'
::method run
  expose i a.i
  say a.i
  i = 2
  say a.i
  a.i = 'local'
  return a.1 a.2 i
::method look
  expose a.
  return a.1 a.2
```

```output
set
one
A.2
one local 2
one two
```

## The list form

r[instr.expose.list]
A name in parentheses is read as [the list form of
PROCEDURE](r:instr.procedure.list) reads it, with [the string value of the
DROP list](r:instr.drop.list), and what differs in a method is the target: the
variable that holds the list is exposed first, so it is an object variable and
the list is its value in the object, and each word of the list names an object
variable. A list that is empty or all whitespace exposes the variable that
holds it and nothing else.

```rexx
o = .box~new
say o~setup
say o~run
say o~setBlank o~blank
::class box
::method init
  expose x y z.
  x = 'X'; y = 'Y'; z.1 = 'Z1'
::method setup
  expose lst
  lst = 'x z.'
  return 'set'
::method run
  expose (lst)
  x = 'x set'
  return symbol('x') symbol('y') symbol('z.1') symbol('lst')
::method setBlank
  expose lst
  lst = '  '
  return 'set'
::method blank
  expose (lst)
  return symbol('x') symbol('lst')
```

```output
set
VAR LIT VAR VAR
set LIT VAR
```

r[instr.expose.list.error]
A word in a list that is not a variable symbol raises the error that [the DROP
list](r:instr.drop.list.error) gives, when the EXPOSE runs. In the example the
handler runs in the internal routine that called the method, so its RETURN
leaves the routine and the loop goes on.

```rexx
o = .box~new
signal on syntax name e
do w over .array~of('c 3', 'c .x', 'c (d)', 'c "d"', 'c d+e', 'c')
  call charout , w '=> '
  o~set(w)
  call go
end
exit
go:
  say o~go
  return
e: say condition('o')~code
  signal on syntax name e
  return
::class box
::method set
  expose lst
  use arg lst
::method go
  expose (lst)
  return 'ok'
```

```output
c 3 => 31.2
c .x => 31.3
c (d) => 20.928
c "d" => 20.928
c d+e => 20.928
c => ok
```

> The length limit of [the DROP list](r:instr.drop.list.error) holds here too.

```rexx
o = .box~new
signal on syntax name e
o~set(copies('a', 250))
say o~go
o~set(copies('a', 251))
say o~go
exit
e: say condition('o')~code
::class box
::method set
  expose lst
  use arg lst
::method go
  expose (lst)
  return 'ok'
```

```output
ok
20.928
```

## Where EXPOSE may appear

r[instr.expose.first]
EXPOSE must be the first instruction of the code of a method. Only comments
and null clauses may come before it in the method, so a label does not, and
neither does another instruction (a NOP, a USE or a second EXPOSE included),
and a position inside a DO, an IF or a SELECT is not the first. A misplaced
EXPOSE raises `SYNTAX 99.907` when the program is read, so no clause of the
program runs. The text of an INTERPRET raises `SYNTAX 99.908` when the
INTERPRET runs, whether or not the method started with an EXPOSE. A program
that reads the same source with [`.method~new`](r:class.method.new.source)
shows the code, because the error is raised at that call.

```rexx
say 'before'
::method m
  nop
  expose a
```

```output rc=157
```

```rexx
call try 'expose a'
call try '/* c */ ; expose a'
call try 'nop; expose a'
call try 'use arg a; expose b'
call try 'expose a; expose b'
call try 'x: expose a'
call try 'do 1; expose a; end'
call try 'if 1 then expose a'
call try 'interpret "expose a"'
call try 'return 1; expose a'
exit
try:
  signal on syntax name e
  m = .method~new('m', arg(1))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
expose a => ok
/* c */ ; expose a => ok
nop; expose a => 99.907
use arg a; expose b => 99.907
expose a; expose b => 99.907
x: expose a => 99.907
do 1; expose a; end => 99.907
if 1 then expose a => 99.907
interpret "expose a" => ok
return 1; expose a => 99.907
```

```rexx
o = .box~new
signal on syntax name e
o~run
e: say condition('o')~code
::class box
::method run
  interpret 'expose a'
```

```output
99.908
```

r[instr.expose.method]
EXPOSE belongs to a method. As the first instruction of the main program, or
of a routine of a [`::ROUTINE`](r:dir.routine.create) directive, it is
accepted when the program is read and raises `SYNTAX 98.992` when it runs.
Anywhere else in the main program or in a routine it is misplaced, as [the
rule on the first instruction](r:instr.expose.first) says, and raises `99.907`
when the program is read. That includes the first clause after the label of an
internal routine, also of one that a method calls, since a label does not
start the code of a method.

```rexx
say 'not run'
nop
expose a
```

```output rc=157
```

```rexx
expose a
```

```output rc=158
```

```rexx
say 'not run'
exit
r:
  expose a
  return
```

```output rc=157
```

```rexx
signal on syntax name e
r = .routine~new('r', 'expose a')
r~call
e: say condition('o')~code
```

```output
98.992
```

## Errors

r[instr.expose.syntax]
A fault in the form of the clause is raised when the program is read, so no
clause runs: `20.902` for an EXPOSE with no name and for text where a name
should be, such as a string, a comma or an `=`; `20.906` for an empty
parenthesis; `46.1` for two names in one parenthesis; `46.901` for a
parenthesis that is not closed; and `31.2` or `31.3` for a name that starts
with a digit or a period. The text of an INTERPRET raises the errors when
the INTERPRET runs.

```rexx
call try 'expose'
call try 'expose "a"'
call try 'expose a, b'
call try 'expose a = 1'
call try 'expose ()'
call try 'expose (a b)'
call try 'expose (a'
call try 'expose 3a'
call try 'expose .a'
call try 'expose a+b'
exit
try:
  signal on syntax name e
  m = .method~new('m', arg(1))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
expose => 20.902
expose "a" => 20.902
expose a, b => 20.902
expose a = 1 => 20.902
expose () => 20.906
expose (a b) => 46.1
expose (a => 46.901
expose 3a => 31.2
expose .a => 31.3
expose a+b => 20.902
```

## Deviations

r[instr.expose.deviation.70]
When a clause holds both parenthesised lists and other names, EXPOSE
processes every name and every list from left to right, so a variable that an
earlier name has exposed is already the object's when a later list's own
variable is read. ooRexx processes the parenthesised lists first, in reverse
order, and then the other names, so the variable of a list is read before any
name of the clause is exposed. In the example, ooRexx reads `a.i` while `i`
is still the method's own unset variable, and so exposes another name than
`x`.

```rexx
o = .box~new
say o~setup
say o~run
::class box
::method setup
  expose i a. x
  i = 1; a.1 = 'x'; x = 'X'
  return 'set'
::method run
  expose i (a.i)
  return symbol('x')
```

```output
set
VAR
```
