---
element: FORWARD
kind: instruction
syntax: "FORWARD [TO expr] [MESSAGE expr] [ARGUMENTS expr | ARRAY (expr, ...)] [CLASS expr] [CONTINUE]"
rexxref: "Keyword Instructions > FORWARD"
deviations: [118, 124]
---

# FORWARD

FORWARD sends again the message that started the running method, with parts
of it changed: the object that receives it, the name, the arguments, or the
class where the search for the method starts. It is the way for a method to
hand its work to another object or to the method it overrides. Without
CONTINUE the running method is over once the message has been sent, and
the answer of the forwarded message is the answer of the first one. This
page specifies the options, what is kept of the original message, what comes
back and where errors go, the places where FORWARD may stand, and the errors
of the clause.

## The message that is sent

r[instr.forward.options]
The options may come in any order, and each may be given once. TO gives the
object that receives the message, and without it the receiver is the object
that received the original message. MESSAGE gives the name of the message,
which is its string value made upper case, and without it the name is the one
of the original message. ARGUMENTS gives the arguments, and ARRAY gives them
as a list of expressions in parentheses, where an expression left out is an
argument that is left empty, and without both the arguments are the ones the
original message had. CLASS gives a class at which the search for the method
starts, so that the methods of the classes below it are passed over, and
without it the search starts at the class of the receiver. CONTINUE is
described by [its own rule](r:instr.forward.continue). TO, MESSAGE, ARGUMENTS
and CLASS are followed by a literal string, a constant symbol such as a
number, an environment symbol such as `.nil`, or an expression in parentheses.
A variable symbol there is not enough.

```rexx
f = .front~new
say f~hello('x', 'y')
say f~named('p', 'q')
say f~mixed('p', 'q')
say f~withArgs('z')
say f~withArray
say f~noArgs('k')
::class target
::method hello
  return 'hello' arg() arg(1) arg(2)
::method recv
  s = arg()
  do i = 1 to arg()
    if arg(i, 'E') then s = s i'='arg(i)
    else s = s i'=-'
  end
  return s
::class front
::method init
  expose t
  t = .target~new
::method hello
  expose t
  forward to (t)
::method named
  expose t
  forward to (t) message 'hello'
::method mixed
  expose t
  forward message 'HeLLo' to (t)
::method withArgs
  expose t
  forward to (t) message 'recv' arguments (.array~of('A1', 'A2', 'A3'))
::method withArray
  expose t
  forward to (t) message 'recv' array ('R1', , 'R3')
::method noArgs
  expose t
  forward to (t) message 'recv' array ()
```

```output
hello 2 x y
hello 2 p q
hello 2 p q
3 1=A1 2=A2 3=A3
3 1=R1 2=- 3=R3
0
```

```rexx
t = .target~new
call try 'hello'
call try .nil
exit
try:
  signal on syntax name e
  say arg(1) '=>' t~fwd(arg(1))
  return
e: say arg(1) '=>' condition('o')~code
  return
::class target
::method hello
  return 'hello'
::method fwd
  use arg name
  forward message (name)
```

```output
hello => hello
The NIL object => 97.1
```

r[instr.forward.nullname]
A message name that is the null string is accepted, whether it is written as a
literal such as `('')` or comes from a variable that holds the null string.
The message with that name takes exactly one argument and answers the string
value of the receiver followed by the string value of that argument. A FORWARD
that sends it with no argument raises `SYNTAX 93.903`, and one that sends it
with two or more raises `93.902`, so the literal and the variable behave alike
and only the number of arguments decides. Because the forwarding method has
ended, a trap in the method does not see the error: the sender of the original
message does. With CONTINUE the method is still running, and its own trap
sees it. The arguments are those that [the rule for the original
message](r:instr.forward.original) names unless ARGUMENTS or ARRAY gives
others.

```rexx
call try '.box~new~lit'
call try ".box~new~lit('a')"
call try ".box~new~var('b')"
call try ".box~new~lit('a', 'b')"
call try '.box~new~var'
call try ".box~new~later('c')"
call try ".box~new~later('c', 'd')"
call try '.box~new~later'
exit
try:
  signal on syntax name e
  interpret 'say arg(1) "=>" 'arg(1)
  return
e: say arg(1) '=>' condition('o')~code
  return
::class box
::method string
  return 'B'
::method lit
  forward message ('')
::method var
  s = ''
  forward message (s)
::method later
  signal on syntax name e
  forward message ('') continue
  return 'after' result
e: return 'in method' condition('o')~code
```

```output
.box~new~lit => 93.903
.box~new~lit('a') => Ba
.box~new~var('b') => Bb
.box~new~lit('a', 'b') => 93.902
.box~new~var => 93.903
.box~new~later('c') => after Bc
.box~new~later('c', 'd') => in method 93.902
.box~new~later => in method 93.903
```

r[instr.forward.original]
When the arguments are not given, the forwarded message carries the
arguments that the original message had when the method was entered, in
the same positions, with the positions that the caller left empty still empty.
The method's own variables do not change that: an assignment to a variable
that USE ARG filled does not change what is forwarded. When no receiver is
given, the message goes to the object itself, so a name or a class that
differs from the original is needed to stop the message from coming back to
the method. A FORWARD that sends the same message to the same object with the
same class again sends the original message unchanged, so the method runs
again and forwards again; what happens then is not specified.

```rexx
d = .derived~new
say d~holes(, 'two', , 'four')
say d~keepOrig('orig1', 'orig2')
say d~viaSelf('q')
::class base
::method recv
  s = arg()
  do i = 1 to arg()
    if arg(i, 'E') then s = s i'='arg(i)
    else s = s i'=-'
  end
  return s
::method name
  return 'base name' arg(1)
::class derived subclass base
::method holes
  forward message 'recv'
::method keepOrig
  use arg a, b
  a = 'changed'
  forward message 'recv'
::method viaSelf
  forward message 'name'
```

```output
4 1=- 2=two 3=- 4=four
2 1=orig1 2=orig2
base name q
```

r[instr.forward.arguments]
The value of ARGUMENTS is turned into the list of arguments by sending it the
message `REQUEST` with the argument `ARRAY` (the
[`request`](r:class.object.request) of `Object` sends `MAKEARRAY`), so an
[array](r:class.array.makearray) supplies its items, an empty array no
argument, and an item that is missing in an array is an argument that is left
empty. A string that is not null and has no line ends is one argument, the
null string supplies none, a string of several lines has one argument for each
line, and a [list](r:class.list.makearray) supplies its items. A value that
cannot be made into an array raises `SYNTAX 98.946`. An instance of a subclass
of `List` is sent `REQUEST` as well ([Deviation
118](r:instr.forward.deviation.118)).

```rexx
t = .target~new
call try 'array', .array~of(1,,3)
call try 'empty', .array~new
call try 'string', 'a b c'
call try 'null', ''
call try 'lines', 'x'||'0a'x||'y'
call try 'list', .list~of(1, 2)
call try 'nil', .nil
call try 'object', .object~new
exit
try:
  signal on syntax name e
  say arg(1) '=>' t~recv(arg(2))
  return
e: say arg(1) '=>' condition('o')~code
  return
::class target
::method recv
  forward message 'show' arguments (arg(1))
::method show
  s = arg()
  do i = 1 to arg()
    if arg(i, 'E') then s = s i'='arg(i)
    else s = s i'=-'
  end
  return s
```

```output
array => 3 1=1 2=- 3=3
empty => 0
string => 1 1=a b c
null => 0
lines => 2 1=x 2=y
list => 2 1=1 2=2
nil => 98.946
object => 98.946
```

r[instr.forward.class]
CLASS is the class at which [the search](r:class.methods.forward) for the
method starts, so a method of the receiver's own class, or of a class between,
is passed over when the class named is above it. The class must be one that
the receiver is an instance of: the class of the receiver itself, one of its
superclasses or a mixin class that it inherits. A class of the program that
the receiver does not belong to raises `SYNTAX 93.957`, and a value that is
not a class raises `SYNTAX 88.914`. A common use is
[`forward class (super)`](r:class.methods.forward) to run the method that the
running one overrides.

```rexx
d = .derived~new
say d~name('arg')
say d~name2
say d~cont
call try 'unrel'
call try 'selfcls'
call try 'supcls'
call try 'strcls'
call try 'mixcls'
exit
try:
  signal on syntax name e
  say arg(1) '=>' d~send(arg(1))
  return
e: say arg(1) '=>' condition('o')~code
  return
::class other
::method hello
  return 'other hello'
::class base
::method name
  return 'base name' arg(1) self~class~id
::method hello
  return 'base hello'
::class mix mixinclass object
::method hi
  return 'mix hi'
::class derived subclass base inherit mix
::method name
  forward class (super)
  say 'never printed'
::method name2
  forward class (super) message 'name' arguments (.array~of('forwarded'))
::method cont
  forward class (super) message 'name' arguments (.array~of('c')) continue
  return 'cont after' result
::method hello
  return 'derived hello'
::method unrel
  forward class (.other) message 'hello'
::method selfcls
  forward class (.derived) message 'hello'
::method supcls
  forward class (.base) message 'hello'
::method strcls
  forward class ('base') message 'hello'
::method mixcls
  forward class (.mix) message 'hi'
```

```output
base name arg DERIVED
base name forwarded DERIVED
cont after base name c DERIVED
unrel => 93.957
selfcls => derived hello
supcls => base hello
strcls => 88.914
mixcls => mix hi
```

## What comes back

r[instr.forward.end]
Without CONTINUE the running method ends when the message has been sent, so
the clauses after the FORWARD do not run. The answer of the forwarded message
is the answer of the original message, and when it has none the original has
none. A condition that the forwarded message raises is raised in the caller of
the method, not in the method: a trap in the method does not see it. If the
receiver has no method of that name, the message is an error in the caller in
the same way.

```rexx
d = .d~new
say d~answer
signal on syntax name e
say d~plain
say 'not reached'
e: say 'caller saw' condition('o')~code
::class d
::method answer
  forward to ('abcde') message 'length'
  say 'not run'
::method plain
  signal on syntax name h
  forward to ('abc') message 'nosuch'
  say 'not run'
h: return 'method trapped'
```

```output
5
caller saw 97.1
```

r[instr.forward.continue]
With CONTINUE the running method does not end: it goes on with the clause
after the FORWARD when the forwarded message has been answered. The answer is
assigned to `RESULT`, and when the message has no answer `RESULT` is dropped,
as after [a message instruction](r:instr.message.result). A condition that the
forwarded message raises is raised in the running method, where a trap can see
it. The running method returns its own value, as any method does.

```rexx
d = .d~new
say d~after
say d~dropped
say d~trapped
::class d
::method after
  forward to ('abc') message 'reverse' continue
  return 'after:' result
::method none
  return
::method dropped
  forward message 'none' continue
  return 'result is' symbol('result')
::method trapped
  signal on syntax name h
  forward to ('abc') message 'nosuch' continue
  return 'not reached'
h: return 'method trapped' condition('o')~code
```

```output
after: cba
result is LIT
method trapped 97.1
```

## Where FORWARD may appear

r[instr.forward.method]
FORWARD works in the code of a method, anywhere among its own clauses, in a
DO, an IF or a SELECT as well. In the main program, in a routine of a
[`::ROUTINE`](r:dir.routine.create) directive and in an internal routine,
including one that a method calls, it raises `SYNTAX 98.947` when it runs. The
text of an INTERPRET raises `SYNTAX 99.923` when the INTERPRET runs, so no
clause of the text runs.

```rexx
o = .box~new
say o~deep
call try 'main'
call try 'internal'
call try 'routine'
say 'inside =>' o~inside
exit
try:
  what = arg(1)
  signal on syntax name e
  if what = 'main' then forward to ('abc') message 'length'
  if what = 'internal' then do; call lab; return; end
  if what = 'routine' then call r
  say what '=> ok'
  return
e: say what '=>' condition('o')~code
  return
lab:
  forward to ('abc') message 'length'
::routine r
  forward to ('abc') message 'length'
::class box
::method deep
  do 1
    if 1 then forward to ('abcd') message 'length'
  end
  return 'never'
::method inside
  signal on syntax name h
  interpret "forward to ('abc') message 'length'"
  return 'no'
h: return condition('o')~code
```

```output
4
main => 98.947
internal => 98.947
routine => 98.947
inside => 99.923
```

## Errors of the clause

r[instr.forward.syntax]
A fault in the form of the clause is raised when the program is read. An
option given twice raises `SYNTAX 25.917` for TO, `25.922` for MESSAGE,
`25.921` for CLASS, `25.919` for CONTINUE, and `25.918` for ARGUMENTS and
ARRAY together or either one twice. A word that is not an option, or an item
after the last option, raises `25.916`. An option with no expression raises
`35.925` for TO, `35.927` for MESSAGE, `35.926` for ARGUMENTS, `35.928` for
CLASS and `35.924` for ARRAY, which also needs its expressions in parentheses.
A variable symbol where an expression is wanted raises `35.1`, and a
parenthesis that is not closed `36.901`. FORWARD with no option at all is
accepted.

```rexx
call try 'forward'
call try 'forward continue'
call try 'forward to (a) message (m) class (c) array (1) continue'
call try 'forward continue array (1) class (c) message (m) to (t)'
call try 'forward to (a) to (b)'
call try 'forward message (m) message (n)'
call try 'forward class (a) class (b)'
call try 'forward continue continue'
call try 'forward arguments (a) array (b)'
call try 'forward array (b) arguments (a)'
call try 'forward array (a) array (b)'
call try 'forward foo'
call try 'forward to (a) foo'
call try 'forward to'
call try 'forward message'
call try 'forward arguments'
call try 'forward class'
call try 'forward array'
call try 'forward array 1, 2'
call try 'forward to self'
call try 'forward message m'
call try 'forward to (a'
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
forward => ok
forward continue => ok
forward to (a) message (m) class (c) array (1) continue => ok
forward continue array (1) class (c) message (m) to (t) => ok
forward to (a) to (b) => 25.917
forward message (m) message (n) => 25.922
forward class (a) class (b) => 25.921
forward continue continue => 25.919
forward arguments (a) array (b) => 25.918
forward array (b) arguments (a) => 25.918
forward array (a) array (b) => 25.918
forward foo => 25.916
forward to (a) foo => 25.916
forward to => 35.925
forward message => 35.927
forward arguments => 35.926
forward class => 35.928
forward array => 35.924
forward array 1, 2 => 35.924
forward to self => 35.1
forward message m => 35.1
forward to (a => 36.901
```

## Deviations

r[instr.forward.deviation.118]
The value of ARGUMENTS that is an instance of a subclass of `List` is sent
the message `request` with the argument `ARRAY`, as for any other object that
is not an array ([Deviation 118](../deviations.md)), so that an
override is used. ooRexx sends the instance no `request` and uses the items it
holds, so the example answers 1 on ooRexx. The `request` of the class `ll`
answers two items.

```rexx
l = .ll~new
l~append('held')
say .w~new~relay(l)
::class ll subclass list
::method request
  say 'request' arg(1)
  return .array~of('a', 'b')
::class w
::method count
  return arg()
::method relay
  forward message 'count' arguments (arg(1))
```

```output
request ARRAY
2
```

r[instr.forward.deviation.124]
A value of ARGUMENTS whose `request` with `ARRAY` answers an object that is not
an `Array` or an instance of a subclass of `Array` raises `SYNTAX 98.946`,
whatever collection the answer is. ooRexx accepts a `Queue` or a `CircularQueue`
as such an answer and uses its items, and crashes for a `List`, a `Bag`, a
`Table` or a `Directory`, among others. In the example the answer is a queue.

```rexx
signal on syntax name bad
say .w~new~relay(.ans~new)
exit
bad: say condition('o')~code
::class w
::method count
  return arg()
::method relay
  forward message 'count' arguments (arg(1))
::class ans
::method request
  return .queue~of('x')
```

```output
98.946
```
