---
element: "Native routines and methods"
kind: concept
---

# Native routines and methods

A routine or method of a library with a package entry declares the types of
its arguments and of its result. The interpreter converts each argument to its
type before the code runs, and converts the result back to a Rexx object after
it. While it runs, the code can see the caller's variables and settings, raise
conditions, and send messages. The examples use [the fixture library
`specfixture`](index.md#the-fixture-libraries).

## Arguments

r[native.calls.conversion]
The arguments are converted in order, before the routine runs, and the first
that does not convert raises the error of its type; the routine does not run.
A 32-bit integer is a whole number from `-2147483648` to `2147483647`, written
in any way Rexx accepts a number, with blanks around it, a decimal point or an
exponent, and anything else raises `SYNTAX 88.907`. The argument is the string
the caller passed, so an expression whose result NUMERIC DIGITS rounds is
compared as rounded. A positive whole number that is not above 0 raises
`88.905`, and a nonnegative whole number below 0 raises `88.904`. A binary
floating-point number is read from all the digits of a Rexx number, or is
`nan`, `+infinity` or `-infinity`, and anything else raises `88.921`. A
logical value is `0` or `1`, and anything else raises `34.901`. A string is
the [string value](r:conv.string.object) of the argument, and `.nil` raises
`88.909`. An Array is the argument when it is one, and otherwise the Array that
its `makeArray` method answers, so a string is an Array of its lines.

```rexx
call t "fxadd(2147483647, 0) fxadd(' 7 ', 0) fxadd(1.0, 2E0)"
call t "fxadd('a', 1)"
call t "fxadd(2147483648, 0)"
call t "fxadd(-2147483648, 0)"
call t "fxadd(1, 1.5)"
call t "fxpositive(5) fxpositive(0)"
call t "fxnonnegative(0) fxnonnegative(-1)"
call t "fxdouble(2.5) fxdouble('x')"
call t "fxlogical(1) fxlogical(0) fxlogical('true')"
call t "fxstring(12) fxstring(.nil)"
call t "fxitems(.array~of(1, , 3)) fxitems('a' || '0a'x || 'b')"
exit
t: parse arg expr
  signal on syntax name e
  interpret 'v =' expr
  say expr '=>' v
  return
e: say expr '=>' condition('o')~code
  return
::requires 'specfixture' LIBRARY
```

```output
fxadd(2147483647, 0) fxadd(' 7 ', 0) fxadd(1.0, 2E0) => 2147483647 7 3
fxadd('a', 1) => 88.907
fxadd(2147483648, 0) => 88.907
fxadd(-2147483648, 0) => 88.907
fxadd(1, 1.5) => 88.907
fxpositive(5) fxpositive(0) => 88.905
fxnonnegative(0) fxnonnegative(-1) => 88.904
fxdouble(2.5) fxdouble('x') => 88.921
fxlogical(1) fxlogical(0) fxlogical('true') => 34.901
fxstring(12) fxstring(.nil) => 88.909
fxitems(.array~of(1, , 3)) fxitems('a' || '0a'x || 'b') => 2 2
```

r[native.calls.count]
A required argument that is omitted raises `SYNTAX 88.901`, and more arguments
than the routine declares raise `88.922`. An optional argument that is omitted
is seen as omitted by the routine, and one that is given is converted as a
required one is.

```rexx
call t "fxadd(1)"
call t "fxadd(, 1)"
call t "fxadd(1, 2, 3)"
call t "fxoptional() fxoptional(,) fxoptional(3)"
call t "fxoptional('x')"
exit
t: parse arg expr
  signal on syntax name e
  interpret 'v =' expr
  say expr '=>' v
  return
e: say expr '=>' condition('o')~code condition('o')~message
  return
::requires 'specfixture' LIBRARY
```

```output
fxadd(1) => 88.901 Missing argument; argument 2 is required.
fxadd(, 1) => 88.901 Missing argument; argument 1 is required.
fxadd(1, 2, 3) => 88.922 Too many arguments in invocation; 2 expected.
fxoptional() fxoptional(,) fxoptional(3) => omitted omitted 3
fxoptional('x') => 88.907 Argument 1 must be in the range -2147483648 to 2147483647; found "x".
```

r[native.calls.arglist]
A routine may take its arguments as one Array, as `ARG(1, 'A')` would give
them: its size counts the places up to the last argument given, and its items
the arguments given.

```rexx
say fxargs(1, , 3) '/' fxargs() '/' fxargs(, ) '/' fxargs(1, )
::requires 'specfixture' LIBRARY
```

```output
3 2 / 0 0 / 0 0 / 1 1
```

r[native.calls.name]
A routine may ask for the name it was called by, which is the name as the call
wrote it: in capitals for a symbol, and as spelled for a quoted name.

```rexx
say fxname() FxName() 'fXnAmE'()
::requires 'specfixture' LIBRARY
```

```output
FXNAME FXNAME fXnAmE
```

## Results

r[native.calls.result]
An object result is the object itself. A routine that answers no result makes
a function call raise `SYNTAX 44.1`, and makes a CALL drop the variable
`RESULT`. A 32-bit integer, a whole number or a logical value is written as a
Rexx whole number. A binary floating-point number is rounded to 9 significant
digits and written as Rexx arithmetic writes a result under NUMERIC DIGITS 9
and NUMERIC FORM SCIENTIFIC, whatever the caller's settings are; a value that
is not a number is `nan`, `+infinity` or `-infinity`.

```rexx
a = .array~new
say (fxecho(a) == a) fxlogical(1) fxadd(-3, 1)
result = 'set'
call fxnothing
say symbol('RESULT')
signal on syntax name e
say fxnothing()
e: say condition('o')~code
numeric digits 20
numeric form engineering
say fxdouble(1/3) fxdouble(0.0000001) fxdouble(1E-20) fxdouble(1E9)
say fxdouble(12345678901) fxdouble(1E300 * 1E300) fxdouble('nan')
::requires 'specfixture' LIBRARY
```

```output
1 1 -2
LIT
44.1
0.333333333 0.0000001 1E-20 1.00000000E+9
1.23456789E+10 +infinity nan
```

## Conditions

r[native.calls.raise]
A routine or method may raise a `SYNTAX` error with a number and its
substitutions. The code goes on running after it has raised the error, and the
error is raised in the caller when the call returns, with the message that
`RAISE SYNTAX` with that number and those substitutions would give; the
result of the call is not delivered. The fixture's routine raises `40.900` and
then sets a variable of the caller, which shows that it ran on; its method
raises `93.900`. An error that nothing traps ends the program as any `SYNTAX`
error does, and the traceback names the routine.

```rexx
after = 'unset'
signal on syntax name s
say fxraise('boom')
s: say condition('o')~code '['condition('o')~message']' after symbol('RESULT')
o = .n2zqk~new
signal on syntax name m
say o~bad('bad thing')
m: say condition('o')~code '['condition('o')~message']'
::requires 'specfixture' LIBRARY
::class n2zqk
::method bad external "LIBRARY specfixture FxMethodRaise"
```

```output
40.900 [boom.] ran on LIT
93.900 [bad thing.]
```

```rexx
say fxraise('final')
::requires 'specfixture' LIBRARY
```

```output rc=216
```

r[native.calls.condition]
A routine may raise a condition by name, with a description and an extra
object. The routine's result is delivered and the clause that called it goes
on; the condition is then raised as one that the clause raised, so a `CALL
ON` handler runs after the clause, and `CONDITION` answers the name, the
description and, for `'A'`, the extra object.

```rexx
call on user n2zqfoo name h
say fxcondition('USER N2ZQFOO', 'desc', 'extra')
say 'after'
exit
h: say 'handler' condition('c') condition('d') condition('a') condition('i')
  return
::requires 'specfixture' LIBRARY
```

```output
returned
handler USER N2ZQFOO desc extra CALL
after
```

r[native.calls.send]
A routine may send a message to an object and receive its result. A condition
that the message raises and does not trap ends the message and is held for the
routine, which can read it and clear it; a condition that it clears is not
raised in the caller.

```rexx
o = .n2zqs~new
say fxsend(o, 'ok') '/' fxsend(o, 'none') '/' fxsend(o, 'bad')
say 'still running'
::requires 'specfixture' LIBRARY
::class n2zqs
::method ok
  return 'fine'
::method none
  return
::method bad
  raise syntax 40.900 array('in bad')
```

```output
fine / no result / condition 40.900
still running
```

## The caller's state

r[native.calls.variables]
A routine may set, read and drop the variables of its caller, by name, in any
case, compound names included. They are the variables that the caller's
clauses see, so in a procedure they are the procedure's own. A variable that
has no value reads as none.

```rexx
x = 'old'
call fxsetvar 'X', 5
call fxsetvar 'y', 6
call fxsetvar 's.1', 7
say x y s.1 fxgetvar('x') fxgetvar('S.1') fxgetvar('nosuch')
call fxdropvar 'X'
say symbol('X') fxgetvar('X')
call p
exit
p: procedure
  call fxsetvar 'Z', 'inner'
  say z symbol('Y')
  return
::requires 'specfixture' LIBRARY
```

```output
5 6 7 5 7 (unset)
LIT (unset)
inner LIT
```

r[native.calls.numeric]
A routine may read the caller's NUMERIC DIGITS, FUZZ and FORM.

```rexx
say fxnumeric()
numeric digits 20
numeric fuzz 2
numeric form engineering
say fxnumeric()
::requires 'specfixture' LIBRARY
```

```output
9 0 SCIENTIFIC
20 2 ENGINEERING
```

## Methods

r[native.calls.method]
A native method may ask for its receiver, the name of the message, and its
scope, the class whose definition holds it. It may set and read the object
variables of its scope, which are the ones that a Rexx method of the same
scope exposes; a subclass's method works on the subclass's variables of the
same name. A variable that has no value reads as none.

```rexx
o = .n2zqsub~new
say (o~self == o) o~msg o~scope~id
o~oset('V', 1)
o~soset('V', 2)
say o~peek o~speek o~oget('V') o~soget('V') o~oget('W')
::requires 'specfixture' LIBRARY
::class n2zqbase
::method self external "LIBRARY specfixture FxSelf"
::method msg external "LIBRARY specfixture FxMessage"
::method scope external "LIBRARY specfixture FxScope"
::method oset external "LIBRARY specfixture FxObjSet"
::method oget external "LIBRARY specfixture FxObjGet"
::method peek
  expose v
  return v
::class n2zqsub subclass n2zqbase
::method soset external "LIBRARY specfixture FxObjSet"
::method soget external "LIBRARY specfixture FxObjGet"
::method speek
  expose v
  return v
```

```output
1 MSG N2ZQBASE
1 2 1 2 (unset)
```
