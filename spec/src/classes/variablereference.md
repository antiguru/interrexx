---
element: "VariableReference"
kind: class
rexxref: "Builtin Classes > Utility Classes > VariableReference Class; Builtin Classes > Utility Classes > VariableReference Class > name; Builtin Classes > Utility Classes > VariableReference Class > request; Builtin Classes > Utility Classes > VariableReference Class > unknown; Builtin Classes > Utility Classes > VariableReference Class > value (Attribute)"
deviations: [72, 118, 124]
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
inherits no mixin class ([the hierarchy rule](r:class.index.hierarchy)). A term
`>name` or `<name`, where `name` is the name of a simple variable or of a stem,
makes a reference to that variable, in the routine or method where the term is
evaluated, and the variable need not have a value. A reference to a variable of
a method can be an object variable that the method exposes. A compound symbol, a
constant symbol or an environment symbol after the `>` or `<` is refused with
`SYNTAX 20.930`, when the program is read; the first example reaches the error
through `INTERPRET`, and the second is a program that ends before anything of it
runs. The instance side of the class defines `name`, `request`, `unknown`,
`value` and `value=`; the comparison methods of `Object` are not among those
that an instance understands, and the messages `=`, `==`, `\=`, `\==`, `<>` and
`><` are handled as the other messages that the class does not define ([the
unknown rule](r:class.variablereference.unknown)).

```rexx
x = 'one'
r = >x
say r~class~id r~name r~value (r~value == x)
z = <x
say z~class~id z~name
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
VariableReference X
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
A message that the reference does not understand is sent on to the value of the
variable, with the same arguments, and its answer is the answer of the
reference: the reference stands in for its value, so `r~length` sends `length`
to the string that the variable holds, and `r~append(4)` appends to the array it
holds. This holds for the messages `=`, `==`, `\=`, `\==`, `<>` and `><` too,
which the reference, unlike other objects, does not understand itself, so they
compare the value of the variable with the argument as given, and for the
operators of arithmetic and comparison, which are messages. The methods that the
reference does understand, `class`, `copy`, `hasMethod` and the others of
`Object`, `name`, `request`, `value` and `value=`, answer for the reference
itself. `unknown` is the method that does the sending: `unknown(name,
arguments)` sends the message `name` to the value with the arguments that
`arguments` holds. It takes exactly two arguments, and a third raises `SYNTAX
93.902` before anything else is checked. `name` is checked next: an omitted one
raises `SYNTAX 93.903`, and one with no string value, `.nil` included, `SYNTAX
88.909`. The message name is used as written: a name that is not in upper case
names no method of the value, which handles it as a message it does not
understand, and the null string is a name like any other, the abuttal's ([the
joining rule](r:class.object.concat)). `arguments` is checked last: an omitted
one raises `SYNTAX 93.903`; any other is sent `request` with `ARRAY`, and an
answer that is an instance of `Array`, or of a subclass of it, with one
dimension is the array of arguments, while any other answer raises `SYNTAX
98.913`, so `.nil`, an `Object` and an array of two dimensions raise it and a
string gives an array of its lines. A position that holds nothing is an omitted
argument. An instance of a subclass of `List` given as `arguments` is sent
`request` as any other object is ([Deviation
118](r:class.variablereference.deviation.118)). A value that does not understand
the message raises `SYNTAX 97.1`, as it would for any other sender.

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
say r~unknown('', .array~of('!')) r~unknown('LEFT', '3')
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
call try "r~unknown('LENGTH', .nil)"
call try "r~unknown(.nil, .nil, 1)"
call try 'r~unknown(.nil)'
call try "r~unknown('LENGTH', .object~new)"
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
hello world! hel
6 10 -5 57 1 1 1 0
1 0 1
r~nosuchmethodzz => 97.1
r~unknown => 93.903
r~unknown('LENGTH') => 93.903
r~unknown('LENGTH', .array~new, 1) => 93.902
r~unknown(.nil, .array~new) => 88.909
r~unknown('', .array~new) => 93.903
r~unknown('LENGTH', .nil) => 98.913
r~unknown(.nil, .nil, 1) => 93.902
r~unknown(.nil) => 88.909
r~unknown('LENGTH', .object~new) => 98.913
```

```rexx
v = 'hello'
r = >v
signal on syntax name bad
say r~unknown('LENGTH', .array~new)
say r~unknown('length', .array~new)
exit
bad: say condition('o')~code
```

```output
5
97.1
```

r[class.variablereference.request]
`request` sends `request` to the value of the variable with its argument and
answers what the value answers, so a value that declares its own `request` is
asked, and a string answers the conversions that `request` of `String` answers.
Sent with no argument, or with an omitted one, it sends the value one omitted
argument: the `request` of the value sees `ARG()` as 1 and `ARG(1, 'E')` as 0.
Two arguments raise `SYNTAX 93.902`.

```rexx
o = .ask~new
r = >o
say r~request('FOO') r~request('STRING')
say r~request r~request()
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
  return 'asked' arg() || ':' || arg(1) || ':' || arg(1, 'E')
```

```output defect=#199
asked 1:FOO:1 asked 1:STRING:1
asked 1::0 asked 1::0
a b 1 The NIL object
r~request('A', 'B') => 93.902
```

## Passing a variable

r[class.variablereference.arguments]
A term `>name` or `<name` has as its value a reference to the variable,
wherever it stands; given as an argument of a call or a message it passes that
reference: the receiver gets a `VariableReference` as the argument, whichever
way it reads its arguments ([the reference rule](r:instr.use.reference) of
`USE ARG` states the `<` target that makes the receiver's variable the
caller's). A plain target of `USE ARG`, or the function `ARG`, shows the
object, and reads and assigns the variable of the caller through it. A
reference to a variable of a routine or method keeps the variable alive after
the routine or method returned, so `value` and `value=` go on working on the
variable it had.

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
builtin function, the string value of a reference is the value of the variable,
and `NOSTRING` is raised only when that value has no string. ooRexx uses the
object name of the referenced value, its default name unless one was set, in
most of these positions and raises `SYNTAX 88.909` for an argument of a method
([Deviation 72](../deviations.md)). The example gives references as the needle
of `pos` and `changeStr` and joins them to strings; the variable holds `one` and
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

r[class.variablereference.deviation.118]
An instance of a subclass of `List` that is given to `unknown` as the array of
arguments is sent `request` with `ARRAY`, as [the unknown
rule](r:class.variablereference.unknown) says of any other object, so that an
override is used. See [Deviation 118](../deviations.md). ooRexx sends the
instance no `request` and uses the items it holds, so the example answers `b`
on ooRexx. The `request` of the class `args` answers one item.

```rexx
x = 'abcdef'
r = >x
say r~unknown('SUBSTR', .args~of(2, 1))
::class args subclass list
::method request
  return .array~of(4)
```

```output
def
```

r[class.variablereference.deviation.124]
An answer of `request` with `ARRAY` that is not an `Array` or an instance of a
subclass of `Array` is refused as the array of arguments of `unknown`, which
raises `SYNTAX 98.913`, whatever collection the answer is. ooRexx accepts a
`Queue` or a `CircularQueue` as such an answer and uses its items, and crashes
for a `List`, a `Bag`, a `Table` or a `Directory`, among others. In the example
the answer is a queue.

```rexx
x = 'abcdef'
ref = >x
call try "ref~unknown('SUBSTR', .ans~new)"
exit
try:
  signal on syntax name bad
  interpret 'v =' arg(1)
  say arg(1) '=>' v
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class ans
::method request
  return .queue~of('x')
```

```output
ref~unknown('SUBSTR', .ans~new) => 98.913
```
