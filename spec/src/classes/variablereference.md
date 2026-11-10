---
element: "VariableReference"
kind: class
rexxref: "Builtin Classes > Utility Classes > VariableReference Class; Builtin Classes > Utility Classes > VariableReference Class > name; Builtin Classes > Utility Classes > VariableReference Class > request; Builtin Classes > Utility Classes > VariableReference Class > unknown; Builtin Classes > Utility Classes > VariableReference Class > value (Attribute)"
deviations: [72]
---

# VariableReference

A `VariableReference` stands for a variable, not for the value the variable
holds at some moment: it can be read and assigned through the reference at any
later time, after the variable has changed and after the routine or method
that owns the variable has returned. A program makes one with a variable
reference term, a `>` before the name of a variable, and receives one when a
caller passes a variable by reference ([`USE ARG`](r:instr.use.reference)).
This book does not specify the term in the chapter on expressions, so this
page states what the object does, from its creation by the term on.

## The class

r[class.variablereference.creation]
`VariableReference` is a class whose only superclass is `Object`, and it
inherits no mixin class ([the hierarchy rule](r:class.index.hierarchy)). A
term `>name`, where `name` is the name of a simple variable or of a stem,
makes a reference to that variable, in the routine or method where the term is
evaluated, and the variable need not have a value. A reference to a variable
of a method can be an object variable that the method exposes. A compound
symbol after the `>` is refused with `SYNTAX 20.930`, when the program is
read; the first example reaches the error through `INTERPRET`, and the second
is a program that ends before anything of it runs. The instance side of the
class defines `name`, `request`, `unknown`, `value` and `value=`; the
comparison methods of `Object` are not among those that an instance
understands, and the messages `=`, `==`, `\=`, `\==`, `<>` and `><` are
handled as the other messages that the class does not define ([the unknown
rule](r:class.variablereference.unknown)).

```rexx
x = 'one'
r = >x
say r~class~id r~name r~value (r~value == x)
o = .box~new
say o~ref~name o~ref~value
do c over .variablereference~superclasses
  say c~id
end
do n over '= == \= \== <> ><'~makeArray(' ')
  say n .object~new~hasMethod(n) r~hasMethod(n)
end
signal on syntax name bad
interpret 'z = >a.b'
say 'not reached'
bad: say condition('o')~code
::class box
::method init
  expose count
  count = 5
::method ref
  expose count
  return >count
```

```output
VariableReference X one 1
COUNT 5
Object
= 1 0
== 1 0
\= 1 0
\== 1 0
<> 1 0
>< 1 0
20.930
```

```rexx
x = >a.b
```

```output rc=236
```

r[class.variablereference.new]
`new` raises `SYNTAX 93.967` whatever the arguments are, and makes no object:
a reference exists only where a variable reference term, or the passing of a
variable by reference, made it. A subclass that a program declares has the
same `new`; a `new` that the subclass declares itself runs first, and the call
of the inherited one from it raises the same error.

```rexx
call try '.variablereference~new'
call try '.variablereference~new(1)'
call try '.variablereference~new(1, 2)'
call try '.kept~new'
call try '.loud~new'
exit
try:
  signal on syntax name bad
  interpret 'rr =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class kept subclass variablereference
::class loud subclass variablereference
::method new class
  say 'new runs'
  return self~new:super
```

```output
.variablereference~new => 93.967
.variablereference~new(1) => 93.967
.variablereference~new(1, 2) => 93.967
.kept~new => 93.967
new runs
.loud~new => 93.967
```

## Methods

r[class.variablereference.name]
`name` answers the name of the variable the reference stands for, as a string
in upper case, with the final period of a stem, as in `S.`. It does not change
when the variable changes or is dropped, and takes no argument
(`SYNTAX 93.902`). There is no `name=`: assigning to it raises `SYNTAX 97.1`.

```rexx
Abc = 5
say (>abc)~name (>ABC)~name
s. = 'dflt'
st = >s.
say st~name st~value~class~id
x = 1
r = >x
drop x
say r~name
call try 'rr = r~name(1)'
call try "r~name = 'zz'"
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
ABC ABC
S. Stem
X
rr = r~name(1) => 93.902
r~name = 'zz' => 97.1
```

r[class.variablereference.value]
`value` answers the value that the variable holds at the moment of the call,
and takes no argument (`SYNTAX 93.902`). A later assignment to the variable,
whoever makes it, shows at the next call. A variable that has no value answers
its own name, in upper case, as the variable would in an expression when no
`NOVALUE` condition is trapped; the call itself raises no `NOVALUE`. The value
of a reference to a stem is the `Stem` object.

```rexx
x = 'one'
r = >x
say r~value
x = 'two'
say r~value
q = >fresh
say q~name q~value
fresh = 'now set'
say q~value
drop fresh
say q~value
signal on novalue name trapped
say q~value
say 'no NOVALUE raised'
call try 'rr = r~value(1)'
exit
trapped: say 'trapped'
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
one
two
FRESH FRESH
now set
FRESH
FRESH
no NOVALUE raised
rr = r~value(1) => 93.902
```

r[class.variablereference.value-set]
`value=` assigns the object that is its argument to the variable, the same
assignment as in the routine or method that owns the variable. Called as a
message it takes exactly one argument (none raises `SYNTAX 88.901`, two
`SYNTAX 93.902`); the usual form is the assignment `r~value = object`. A
variable that had no value gets one, so a reference to a variable that was
never set makes it, and a variable that was dropped is set again. The setting
works on an object variable that a method exposed as on a variable of the
program.

```rexx
r = >fresh
r~value = 'first'
say fresh
r~value = 'again'
say fresh r~value
drop fresh
r~value = .nil
say (fresh == .nil)
o = .box~new
b = o~ref
b~value = 'set through the reference'
say o~count
call try "r~'VALUE='"
call try "r~'VALUE='(1, 2)"
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class box
::method init
  expose count
  count = 5
::method ref
  expose count
  return >count
::method count
  expose count
  return count
```

```output
first
again again
1
set through the reference
r~'VALUE=' => 88.901
r~'VALUE='(1, 2) => 93.902
```

r[class.variablereference.unknown]
A message that the reference does not understand is sent on to the value of
the variable, with the same arguments, and its answer is the answer of the
reference: the reference stands in for its value, so `r~length` sends `length`
to the string that the variable holds, and `r~append(4)` appends to the array
it holds. This holds for the messages `=`, `==`, `\=`, `\==`, `<>` and `><`
too, which the reference, unlike other objects, does not understand itself, so
they compare the value of the variable with the argument as given, and for the
operators of arithmetic and comparison, which are messages. The methods that
the reference does understand, `class`, `copy`, `hasMethod` and the others of
`Object`, `name`, `request`, `value` and `value=`, answer for the reference
itself. `unknown` is the method that does the sending: its first argument is
the name of the message as a string, in upper case, and its second an array of
the arguments, both required (`SYNTAX 93.903`), and a third argument raises
`SYNTAX 93.902`. A name that is no string raises `SYNTAX 88.909`. The value
gets the message with the name that the first argument gives, so a value that
does not understand it raises `SYNTAX 97.1`, as it would for any other sender,
and a name that is the null string raises `SYNTAX 93.903`.

```rexx
text = 'hello world'
r = >text
say r~length r~left(3) r~substr(7) r~words
say r~class~id r~hasMethod('LENGTH') r~hasMethod('NAME')
arr = .array~of(1, 2, 3)
ra = >arr
say ra~items ra~class~id
ra~append(4)
say arr~items
say r~unknown('LEFT', .array~of(2)) r~unknown('WORDS', .array~new)
x = 5
rx = >x
say (rx + 1) (rx * 2) (-rx) (rx || 7) (rx = 5) (rx == 5) (rx < 7) (rx == 6)
say (ra == arr) (ra == ra) (rx == >x)
call try 'r~nosuchmethodzz'
call try "r~unknown"
call try "r~unknown('LENGTH')"
call try "r~unknown('LENGTH', .array~new, 1)"
call try 'r~unknown(.nil, .array~new)'
call try "r~unknown('', .array~new)"
exit
try:
  signal on syntax name bad
  interpret 'rr =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
11 hel world 2
VariableReference 0 1
3 VariableReference
4
he 2
6 10 -5 57 1 1 1 0
1 0 1
r~nosuchmethodzz => 97.1
r~unknown => 93.903
r~unknown('LENGTH') => 93.903
r~unknown('LENGTH', .array~new, 1) => 93.902
r~unknown(.nil, .array~new) => 88.909
r~unknown('', .array~new) => 93.903
```

r[class.variablereference.request]
`request` sends `request` to the value of the variable with its argument and
answers what the value answers, so a value that declares its own `request` is
asked, and a string answers the conversions that `request` of `String`
answers. The reference accepts at most one argument (`SYNTAX 93.902`).

```rexx
o = .ask~new
r = >o
say r~request('FOO') r~request('STRING')
t = 'a b'
rt = >t
say rt~request('STRING') rt~request('ARRAY')~items rt~request('FOO')
call try "r~request('A', 'B')"
exit
try:
  signal on syntax name bad
  interpret 'rr =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class ask
::method request
  return 'asked' arg() || ':' || arg(1)
```

```output
asked 1:FOO asked 1:STRING
a b 1 The NIL object
r~request('A', 'B') => 93.902
```

## Passing a variable

r[class.variablereference.arguments]
A term `>name` or `<name` given as an argument of a call or a message passes a
reference to the variable: the receiver gets a `VariableReference` as the
argument, whichever way it reads its arguments ([the reference
rule](r:instr.use.reference) of `USE ARG` states the `<` target that makes the
receiver's variable the caller's). A plain target of `USE ARG`, or the
function `ARG`, shows the object, and reads and assigns the variable of the
caller through it. A reference to a variable of a routine or method keeps the
variable alive after the routine or method returned, so `value` and `value=`
go on working on the variable it had.

```rexx
v = 'orig'
call show <v
call show >v
call keep >v
say v
call keep <v
say v
call alias <v
say v
o = .box~new
say o~show(>v)
o~keep(<v)
say v
r = made()
say r~name r~value
r~value = 'changed after return'
say r~value
exit
show: procedure
  say arg(1)~class~id arg(1)~name arg(1)~value
  return
keep: procedure
  use arg ref
  say ref~class~id ref~value
  ref~value = ref~value || '+kept'
  return
alias: procedure
  use arg <p
  p = p || '+alias'
  return
made: procedure
  local = 'local value'
  return >local
::class box
::method show
  use arg r
  return r~class~id r~value
::method keep
  use arg r
  r~value = r~value || '+m'
```

```output
VariableReference V orig
VariableReference V orig
VariableReference orig
orig+kept
VariableReference orig+kept
orig+kept+kept
orig+kept+kept+alias
VariableReference orig+kept+kept+alias
orig+kept+kept+alias+m
LOCAL local value
changed after return
```

r[class.variablereference.copy]
`copy` answers a new `VariableReference` that stands for the same variable as
the receiver, with the same name: a value assigned through either of the two
shows through both and in the variable. `copy` takes no argument
(`SYNTAX 93.902`) and sends no `init`, as [the copy rule](r:class.object.copy)
says.

```rexx
x = 'start'
r = >x
c = r~copy
say c~class~id c~name c~value (c~identityHash == r~identityHash),
  (c~value == r~value)
c~value = 'set through the copy'
say x r~value
call try 'r~copy(1)'
exit
try:
  signal on syntax name bad
  interpret 'rr =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
VariableReference X start 0 1
set through the copy set through the copy
r~copy(1) => 93.902
```

## Deviations

r[class.variablereference.deviation.72]
In every position that needs a string, as an argument of a string method or a
builtin function, the string value of a reference is the value of the
variable, and `NOSTRING` is raised only when that value has no string. ooRexx
uses the default name of the reference in most of these positions and raises
`SYNTAX 88.909` for an argument of a method ([Deviation
72](../deviations.md)). The example gives references as the needle of `pos`
and `changeStr` and joins them to strings; the variable holds `one` and
`zero`.

```rexx
x = 'one'
y = 'zero'
rx = >x
ry = >y
say 'xonex'~pos(rx) 'xzerox'~pos(ry)
say 'a' || rx || 'b' length('a' || ry) 'abc'~left(rx~length)
say 'xonex'~changeStr(rx, '1')
```

```output
2 2
aoneb 5 abc
x1x
```
