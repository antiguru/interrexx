---
element: "Message"
kind: class
rexxref: "Builtin Classes > Fundamental Classes > Message Class; Builtin Classes > Fundamental Classes > Message Class > new (Class Method); Builtin Classes > Fundamental Classes > Message Class > arguments; Builtin Classes > Fundamental Classes > Message Class > completed; Builtin Classes > Fundamental Classes > Message Class > errorCondition; Builtin Classes > Fundamental Classes > Message Class > halt; Builtin Classes > Fundamental Classes > Message Class > hasError; Builtin Classes > Fundamental Classes > Message Class > hasResult; Builtin Classes > Fundamental Classes > Message Class > messageComplete; Builtin Classes > Fundamental Classes > Message Class > messageName; Builtin Classes > Fundamental Classes > Message Class > notify; Builtin Classes > Fundamental Classes > Message Class > reply; Builtin Classes > Fundamental Classes > Message Class > replyWith; Builtin Classes > Fundamental Classes > Message Class > result; Builtin Classes > Fundamental Classes > Message Class > send; Builtin Classes > Fundamental Classes > Message Class > sendWith; Builtin Classes > Fundamental Classes > Message Class > start; Builtin Classes > Fundamental Classes > Message Class > startWith; Builtin Classes > Fundamental Classes > Message Class > target; Builtin Classes > Fundamental Classes > Message Class > triggered; Builtin Classes > Fundamental Classes > Message Class > wait; Builtin Classes > Utility Classes > AlarmNotification Class > cancel"
deviations: [105, 107, 109, 113, 116, 118, 124]
---

# Message

A `Message` object is a message that has been written down as an object: a
target, the name of a message and its arguments. A program makes one with
`new`, or gets one from the `start` and `startWith` methods of
[`Object`](object.md), and then decides when the message is sent. It can send
the message and wait for the answer, start it so that another activity runs
it while the sender goes on, or start a copy of it. Afterwards the object
keeps what the message answered, or the error that ended it, and a program
can ask for them, wait for them, or be told when they are there.

The page specifies the class method that makes a message object, the methods
that read what it holds, the methods that run it, the methods that wait for
it, stop it or ask for a notification when it is done, and the three methods
the class has because it implements the notification interfaces of
[`MessageNotification`](messagenotification.md) and
[`AlarmNotification`](alarmnotification.md). How an activity is scheduled,
when a started message begins to run, and how it interleaves with the activity
that started it is specified in the environment chapter. The examples on this
page therefore show only results that no scheduling can change: they wait for
a message with `result` or `wait`, or park the activity of the message on a
guard that the example opens, before they print anything that depends on it.
The methods that an example waits for, or that the waiting activity calls
meanwhile, are `UNGUARDED`, so that the waiting activity and the activity of
the message can both use the object ([the guard
rule](r:instr.guard.exclusion)).

The examples that show an error use a routine `try` that interprets the clause
in its argument and prints the clause with the error code, or with the class
of the answer when no error is raised.

## The class

r[class.message.creation]
`Message` is a class whose superclass is `Object` and which inherits the
mixin classes `MessageNotification` and `AlarmNotification`; it is not itself
a mixin class. An instance made by `new` has the default name `a Message`, and
so has its string value ([the string rule](r:class.index.string)). A message
object that has not been run holds its target, its name and its arguments, has
not completed, has no result and has no error.

```rexx
m = .message~new(.object~new, 'string')
say m~class~id '|' m~string '|' m~defaultName
say .message~superclass~id .message~superclasses~items,
  .message~queryMixinClass
say m~isInstanceOf(.messagenotification) m~isInstanceOf(.alarmnotification)
say m~completed m~hasResult m~hasError m~errorCondition~isNil
say .object~new~start('string')~class~id
```

```output
Message | a Message | a Message
Object 3 0
1 1
0 0 0 1
Message
```

r[class.message.started]
A message object that has been given `start` or `startWith` is a started
message. Every later `send`, `sendWith`, `start`, `startWith`, `reply`,
`replyWith`, `messageComplete` or `triggered` to it raises `SYNTAX 98.915`,
whether or not the first run has ended. No other method refuses it. A message
object that has only been sent, that has only received `reply` or `replyWith`
(its copy was started, and not it), or that has only been run by
`messageComplete` or `triggered`, is not a started message, and these methods
do not refuse it. The copy that `reply` and `replyWith` answer is a started
message.

```rexx
t = .t~new
a = .message~new(t, 'hello')
a~start
a~wait
call try 'a~send'
call try 'a~sendWith(, .array~new)'
call try 'a~start'
call try 'a~startWith(, .array~new)'
call try 'a~reply'
call try 'a~replyWith(, .array~new)'
call try 'a~messageComplete(1)'
call try 'a~triggered(1)'
say a~completed a~hasResult a~result a~target~class~id
b = .message~new(t, 'hello')
b~send
b~start
b~wait
say b~result
call try 'b~send'
c = .message~new(t, 'hello')
r = c~reply
call try 'r~send'
call try 'r~start'
call try 'r~reply'
call try 'r~messageComplete(1)'
c~send
c~reply
d = .message~new(t, 'hello')
d~messageComplete
say d~result
d~send
say 'unmarked'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method hello unguarded
  return 'hello'
```

```output
a~send => 98.915
a~sendWith(, .array~new) => 98.915
a~start => 98.915
a~startWith(, .array~new) => 98.915
a~reply => 98.915
a~replyWith(, .array~new) => 98.915
a~messageComplete(1) => 98.915
a~triggered(1) => 98.915
1 1 hello T
hello
b~send => 98.915
r~send => 98.915
r~start => 98.915
r~reply => 98.915
r~messageComplete(1) => 98.915
hello
unmarked
```

## Class methods

### new

r[class.message.new]
`new(target, name, option, ...)` makes a message object that has not been run.
It needs at least two arguments: fewer raise `SYNTAX 93.901`, and an omitted
target raises `88.901`. The target is any object, `.nil` included. The name is
checked next, after the target, as [the `send` rule](r:class.object.send)
checks it: a string,
or an array of two items, the name and a class that starts the search ([the
scoped send rule](r:class.methods.scoped.send)). A name that is not a
string is read as [the `send` rule](r:class.object.send) reads it, through
`request` with `ARRAY`; `.nil` and an object that answers `.nil` raise
`93.972`, an array of another size `93.946`, an
array whose first item is missing `88.901`, an array with a first item that
has no string value `88.909` and one with a
second item that is not a class `88.914`. The null string is a name. The
option that follows is the third argument, which says how the remaining
arguments are read, as the next rule states. With no option, the message has
no arguments. The answer is the new object, and what the object holds is shown
by [`messageName`](r:class.message.messagename),
[`target`](r:class.message.target) and
[`arguments`](r:class.message.arguments).

```rexx
t = .t~new
call try '.message~new(t, "hello")'
call try '.message~new(.nil, "hello")'
call try '.message~new(t, "")'
call try '.message~new(t, .array~of("hello", .t))'
call try '.message~new'
call try '.message~new(t)'
call try '.message~new(, "hello")'
call try '.message~new(t, .nil)'
call try '.message~new(t, .array~of("hello"))'
call try '.message~new(t, .array~of(.nil, .t))'
call try '.message~new(t, .array~of("hello", 3))'
call try '.message~new(t, .nil, "Z")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r~class~id
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method hello
  return 'hello'
```

```output
.message~new(t, "hello") => Message
.message~new(.nil, "hello") => Message
.message~new(t, "") => Message
.message~new(t, .array~of("hello", .t)) => Message
.message~new => 93.901
.message~new(t) => 93.901
.message~new(, "hello") => 88.901
.message~new(t, .nil) => 93.972
.message~new(t, .array~of("hello")) => 93.946
.message~new(t, .array~of(.nil, .t)) => 88.909
.message~new(t, .array~of("hello", 3)) => 88.914
.message~new(t, .nil, "Z") => 93.972
```

r[class.message.new.options]
The option is a string whose first letter, in either case, is `I` (individual)
or `A` (array); the other letters are not looked at, so `Individual` and `array`
are options. Any other string, the null string, and a number raise
`SYNTAX 93.915`, and an option that has no string value raises `88.909`; the
option is checked after the name. An omitted option with an argument after it
raises `88.901`. With `I`, every argument after the option is an argument of the
message, in order, and an omitted place stays omitted; omitted places at the end
add nothing to the count, as [the count rule](r:class.index.args.count) says,
and an argument may be any object, `.nil` included. With `A`, the fourth
argument is required (`SYNTAX 93.901`) and is the only argument that may follow:
a fifth raises `93.902`, also when the fourth is omitted. It holds the arguments
of the message: it is an array, or an object that answers an array to the
message `request` with the argument `ARRAY`, and the items of the array are the
arguments in order. The message keeps a copy of the array, made when `new` runs,
so that a change that the caller makes to the array afterwards does not reach
the message, and the number of arguments is the position of the last item
present, as [the count rule](r:class.index.args.count) says ([Deviation
113](r:class.message.deviation.113)). An object that answers no array, `.nil`
and an array of more than one dimension among them, raises `SYNTAX 88.913`.

```rexx
t = .t~new
say .message~new(t, 'show')~send
say .message~new(t, 'show', 'I', 1, , 3)~send
say .message~new(t, 'show', 'individual', 1, )~send
say .message~new(t, 'show', 'I', , )~send
say .message~new(t, 'show', 'I', 'x', .nil)~send
say .message~new(t, 'show', 'Array', .array~of(4, 5, 6))~send
say .message~new(t, 'show', 'a', .list~of(7, 8))~send
say .message~new(t, 'show', 'A', 'xyz')~send
call try '.message~new(t, "show", "Z")'
call try '.message~new(t, "show", "")'
call try '.message~new(t, "show", 1)'
call try '.message~new(t, "show", .nil)'
call try '.message~new(t, "show", "Z", .nil)'
call try '.message~new(t, "show", , 1)'
call try '.message~new(t, "show", "A")'
call try '.message~new(t, "show", "A", .nil)'
call try '.message~new(t, "show", "A", .array~new(2, 2))'
call try '.message~new(t, "show", "A", .array~of(1), 2)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r~class~id
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method show
  s = arg()':'
  do i = 1 to arg()
    if arg(i, 'e') then s = s arg(i)
    else s = s '.'
  end
  return s
```

```output
0:
3: 1 . 3
1: 1
0:
2: x The NIL object
3: 4 5 6
2: 7 8
1: xyz
.message~new(t, "show", "Z") => 93.915
.message~new(t, "show", "") => 93.915
.message~new(t, "show", 1) => 93.915
.message~new(t, "show", .nil) => 88.909
.message~new(t, "show", "Z", .nil) => 93.915
.message~new(t, "show", , 1) => 88.901
.message~new(t, "show", "A") => 93.901
.message~new(t, "show", "A", .nil) => 88.913
.message~new(t, "show", "A", .array~new(2, 2)) => 88.913
.message~new(t, "show", "A", .array~of(1), 2) => 93.902
```

r[class.message.new.sends]
`new` sends the message `request` with the argument `ARRAY` to the fourth
argument of the option `A`, once, before it sends `init`, and uses the array
that `request` answers. It sends no message to the target or to the arguments
of `I`, and to the name only the `request` that [the Object sends
rule](r:class.object.send.sends) states, before the option is read. It then
sends the new object the message `init` with no argument, whatever arguments
`new` had, so a subclass of `Message`
that declares `init` sees none of them; the object that `init` runs on already
holds its target, its name and its arguments. The answer of `init` is not
used, and `new` answers the new object, as [the new
rule](r:class.index.new) says for every class. `new` makes an instance of the
class that received it, so a subclass of `Message` makes instances of itself. A
name that is no string is sent `request` once, at `new`; the message then holds
the name the answer gives, and running it sends nothing more. The second
example shows it.

```rexx
t = .t~new
m = .mine~new(t, 'show', 'A', .logged~new)
say 'new answered' m~class~id
m = .mine~new(t, 'show', 'I', 'q', 'r')
say m~send
m = .mine~new(.logged~new, 'show', 'I', .logged~new)
say 'no message to the target or the argument'

::class t
::method show
  return 'show' arg()
::class logged
::method request
  say 'request' arg(1)
  forward class (super)
::method makearray
  say 'makearray'
  return .array~of(1, 2)
::class mine subclass message
::method init
  say 'init' arg() self~messageName self~target~class~id self~arguments~items
  forward class (super) continue
```

```output
request ARRAY
makearray
init 0 SHOW T 2
new answered MINE
init 0 SHOW T 2
show 2
init 0 SHOW LOGGED 1
no message to the target or the argument
```

```rexx
o = .w~new
m = .message~new(o, .duck~new, 'I', 5)
say 'made'
say m~send
say m~messageName
::class w
::method count
  return 'count got' arg()
::class duck
::method request
  say 'duck request' arg(1)
  return .array~of('count', .w)
```

```output
duck request ARRAY
made
count got 1
COUNT
```

## Methods

### What the object holds

r[class.message.messagename]
`messageName` answers the name of the message as a string, in upper case. For
a name that was given as an array of two items, it is the first item. It does
not change when the message is run, and it takes no argument (`SYNTAX 93.902`
for one).

```rexx
t = .t~new
say .message~new(t, 'show')~messageName
say .message~new(t, 'MixedCase')~messageName
say .message~new(t, .array~of('inLeaf', .t))~messageName
say '['.message~new(t, '')~messageName']'
m = .message~new(t, 'show')
m~send
say m~messageName
call try 'm~messageName(1)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method show
  return 1
```

```output
SHOW
MIXEDCASE
INLEAF
[]
SHOW
m~messageName(1) => 93.902
```

r[class.message.target]
`target` answers the object that the message is sent to, the object that
`new` was given as the target. A message that has a scoped name sends to this
object too, and its target is not the class of the scope. The methods that
run the message change the target when they are given one, as the rules for
[`send`](r:class.message.send) and the others state. `target` takes no
argument.

```rexx
t = .t~new
u = .u~new
m = .message~new(t, .array~of('show', .t))
say (m~target == t) (m~target == u)
say .message~new(.nil, 'show')~target~isNil
m~send(u)
say (m~target == t) (m~target == u)
call try 'm~target(1)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method show
  return 1
::class u subclass t
```

```output
1 0
1
0 1
m~target(1) => 93.902
```

r[class.message.arguments]
`arguments` answers an array that holds the arguments of the message, in
order, with the places of omitted arguments empty; it is empty when the
message has no arguments. The array is a new one each time the method is
sent, so changing it changes nothing in the message. The methods that run the
message replace the arguments when they are given some, and the array
answers the new ones afterwards. `arguments` takes no argument.

```rexx
t = .t~new
m = .message~new(t, 'show', 'I', 'a', , 'c')
a = m~arguments
say a~class~id a~items a~hasIndex(2) a[1] a[3]
say (a == m~arguments)
a~append('d')
say m~arguments~items m~send
say .message~new(t, 'show')~arguments~items
m~send(, 'x')
say m~arguments~items m~arguments[1]
call try 'm~arguments(1)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method show
  return arg() arg(1, 'e') arg(2, 'e') arg(3, 'e')
```

```output
Array 2 0 a c
0
2 3 1 0 1
0
1 x
m~arguments(1) => 93.902
```

### What became of the message

r[class.message.completed]
`completed` answers 1 when the message has been run and has ended, whether it
answered, answered nothing or raised an error, and 0 when it has not been run,
or its latest run has not ended. It takes no argument.

```rexx
g = .gate~new
m = .message~new(g, 'work')
say m~completed
m~start
g~await
say m~completed
g~release
m~wait
say m~completed
e = .message~new(g, 'boom')
signal on syntax name caught
e~send
caught: say e~completed
::class gate
::method init
  expose up go
  up = .false
  go = .false
::method await unguarded
  expose up
  guard on when up
::method release unguarded
  expose go
  go = .true
::method work unguarded
  expose up go
  up = .true
  guard on when go
::method boom
  return 1/0
```

```output
0
0
1
1
```

r[class.message.hasresult]
`hasResult` answers 1 when the message has been run, has ended without an
error and answered a value, and 0 when it has not ended, ended with an error,
or answered nothing. The value `.nil` and the null string are values. It
takes no argument. The answer is that of the latest run.

```rexx
t = .t~new
do n over 'yes nil none empty boom'~makearray(' ')
  m = .message~new(t, n)
  call attempt m
  say n m~hasResult m~completed
end
say .message~new(t, 'yes')~hasResult
exit
attempt:
  signal on syntax name caught
  arg(1)~send
caught: return
::class t
::method yes
  return 'yes'
::method nil
  return .nil
::method none
  return
::method empty
  return ''
::method boom
  return 1/0
```

```output
yes 1 1
nil 1 1
none 0 1
empty 1 1
boom 0 1
0
```

r[class.message.haserror]
`hasError` answers 1 when the latest run ended with an error condition and 0
when the message has not been run, has not ended, or ended without one. The
error is a SYNTAX condition that the code of the message raised and nothing
trapped there, including the `97.1` of a message that no method of the target
understands, and a HALT that nothing trapped, which is the error `4.1`
([halt](r:class.message.halt)). Any other condition that nothing traps ends
the method as [the RAISE rule](r:instr.raise.exit) says and is not an error:
the message has completed, with no result and no error. The error of an
argument that `sendWith` or the other methods refuse is raised to their caller
and is not an error of the message, which is then not run. It takes no
argument. The answer, like those of `completed`, `hasResult` and
`errorCondition`, is that of the latest run: a message that is sent again
starts from the state of one that has not been run.

```rexx
t = .t~new
m = .message~new(t, 'ok')
say m~hasError
m~send
say m~hasError
m = .message~new(t, 'boom')
call attempt m
say m~hasError
m = .message~new(t, 'nosuchzz')
call attempt m
say m~hasError
m = t~start('boom')
m~wait
say m~hasError
m = .message~new(t, 'usr')
call attempt m
say m~completed m~hasError m~hasResult
f = .message~new(t, 'flip')
do 3
  call attempt f
  say f~completed f~hasError f~hasResult f~errorCondition~isNil
end
g = .message~new(t, 'ok')
call try 'g~sendWith(t, .nil)'
say g~completed g~hasError
exit
attempt:
  signal on syntax name caught
  arg(1)~send
caught: return
try:
  signal on syntax name bad
  interpret arg(1)
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method init
  expose n
  n = 0
::method ok
  return 1
::method boom
  return 1/0
::method usr
  raise user foo
  return 'not reached'
::method flip
  expose n
  n = n + 1
  if n = 2 then return 1/0
  return n
```

```output
0
0
1
1
1
1 0 0
1 0 1 1
1 1 0 0
1 0 1 1
g~sendWith(t, .nil) => 88.913
0 0
```

r[class.message.errorcondition]
`errorCondition` answers the condition object of the error that ended the
latest run, and `.nil` when there is none: when the message has not been run,
has not ended, or ended without an error. The object has the entries of a
SYNTAX condition, as [the entries rule](r:cond.information.object) lists
them: for the division by zero of the example `CODE` is `42.3`, `RC` is `42`
and `CONDITION` is `SYNTAX`; for a message that no method of the target
understands `CODE` is `97.1` and the entry `ADDITIONAL` holds the string value
of the target and the name. It takes no argument.

```rexx
t = .t~new
m = .message~new(t, 'boom')
say m~errorCondition~isNil
signal on syntax name c1
m~send
c1: c = m~errorCondition
say c~class~id c~code c~rc c~condition
n = t~start('nosuchzz')
n~wait
say n~errorCondition~code n~errorCondition~additional[1],
  n~errorCondition~additional[2]
say .message~new(t, 'ok')~~send~errorCondition~isNil
::class t
::method boom
  return 1/0
::method ok
  return 1
```

```output
1
Directory 42.3 42 SYNTAX
97.1 a T NOSUCHZZ
1
```

### Running the message

r[class.message.send]
`send(target, argument, ...)` sends the message in the activity of the sender
and does not answer until the message has ended. It answers what the message
answers, and nothing when the message answers nothing, so that a use of the
answer as a value then raises `SYNTAX 91.999`. An error that the message
raises is raised at the clause that sent `send`, as the error of an ordinary
message would be, and is also kept: `hasError` answers 1 and `result` raises
it again. The first argument, when it is not omitted, replaces the target of
the message for this and every later run, and it may be `.nil`. When the call
has more than one argument, as [the count rule](r:class.index.args.count)
counts them, the arguments after the first replace the arguments of the
message likewise; with only a target, or none, the message keeps its
arguments. The message is found by [the search rule](r:class.methods.search)
in the target, goes to `UNKNOWN` when nothing is found, raises NOMETHOD
(`SYNTAX 97.1` when untrapped) when there is none, and is checked for
`PRIVATE` and `PACKAGE` access as if the code that called `send` had sent it.
A scoped name starts the search at its class, and raises `93.957` when the
target is not an instance of that class; that error is raised to the caller
of `send`, `start` or the other run methods before the message runs, and is
not an error of the message. Each `send` is a new run: the message
can be sent again, and the result, the error and the completion are those of
the latest run.

```rexx
t = .t~new
m = .message~new(t, 'show', 'I', 'a', 'b')
say m~send
say m~send(, 'X')
say m~send(.u~new)
say m~target~class~id m~arguments~items m~arguments[1]
call try 'm~send(.nil)'
say m~target~isNil
call try 'x = .message~new(t, "none")~send'
call try 'x = .message~new(t, "boom")~send'
call try 'x = .message~new(t, "secret")~send'
call try 'x = .message~new(t, "nosuchzz")~send'
call try 'x = .message~new(.u~new, .array~of("show", .t), "I", 7)~send'
call try 'x = .message~new(t, .array~of("show", .u))~send'
say .message~new(.v~new, 'nosuchzz', 'I', 1, 2)~send
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok' x
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method show
  return self~class~id arg() arg(1)
::method none
::method boom
  return 1/0
::method secret private
  return 'secret'
::class u subclass t
::method show
  return 'u' arg() arg(1)
::class v
::method unknown
  return 'unknown' arg(1) arg(2)~items
```

```output
T 2 a
T 1 X
u 1 X
U 1 X
m~send(.nil) => 97.1
1
x = .message~new(t, "none")~send => 91.999
x = .message~new(t, "boom")~send => 42.3
x = .message~new(t, "secret")~send => 97.2
x = .message~new(t, "nosuchzz")~send => 97.1
x = .message~new(.u~new, .array~of("show", .t), "I", 7)~send => ok U 1 7
x = .message~new(t, .array~of("show", .u))~send => 93.957
unknown NOSUCHZZ 2
```

r[class.message.sendwith]
`sendWith(target, array)` is `send` with the arguments of the message in an
array, and it replaces the arguments always. The array is required
(`SYNTAX 88.901` when it is missing), a third argument raises `93.902`, and
the first argument is the target, which is replaced as `send` does when it is
not omitted. The second argument is an array, or an object that answers an
array to the message `request` with the argument `ARRAY`; an object that
answers anything else, `.nil` included, and an array of more than one
dimension raise `SYNTAX 88.913`. The items are the arguments in order. The
message keeps a copy of the array, so a later change to it does not reach the
message, and the number of arguments is the position of the last item present,
as [the count rule](r:class.index.args.count) says
([Deviation 113](r:class.message.deviation.113)). The target is replaced
before the array is checked, so a call that is refused
leaves its target in the message; the arguments and what the latest run left
are unchanged. Apart from the arguments, `sendWith` runs the message as `send`
does, answers as it does, and a run replaces the state of the message in the
same way.

```rexx
t = .t~new
m = .message~new(t, 'show', 'I', 'old')
say m~sendWith(, .array~of(1, 2, 3))
say m~arguments~items m~arguments[3]
say m~sendWith(.u~new, .array~of(5))
say m~target~class~id
say m~sendWith(, .list~of(6, 7))
say m~sendWith(, 'abc')
call try 'm~sendWith'
call try 'm~sendWith(t)'
call try 'm~sendWith(t, .nil)'
call try 'm~sendWith(t, .array~new(2, 2))'
call try 'm~sendWith(t, .array~new, 1)'
r = .message~new(t, 'show')
call try 'r~sendWith(.u~new, .nil)'
say r~target~class~id
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method show
  s = arg()':'
  do i = 1 to arg()
    if arg(i, 'e') then s = s arg(i)
    else s = s '.'
  end
  return s
::class u subclass t
::method show
  return 'u' arg() arg(1)
```

```output
3: 1 2 3
3 3
u 1 5
U
u 2 6
u 1 abc
m~sendWith => 88.901
m~sendWith(t) => 88.901
m~sendWith(t, .nil) => 88.913
m~sendWith(t, .array~new(2, 2)) => 88.913
m~sendWith(t, .array~new, 1) => 93.902
r~sendWith(.u~new, .nil) => 88.913
U
```

r[class.message.sendwith.sends]
`sendWith` sends its second argument the message `request` with the argument
`ARRAY`, once, before it runs the message, and it uses the array that
`request` answers. An instance of a subclass of `List` is sent it as well
([Deviation 118](r:class.message.deviation.118)).

```rexx
t = .t~new
m = .message~new(t, 'count')
say m~sendWith(, .logged~new)
::class t
::method count
  return 'count got' arg()
::class logged
::method request
  say 'request' arg(1)
  forward class (super)
::method makearray
  say 'makearray'
  return .array~of(1, 2)
```

```output
request ARRAY
makearray
count got 2
```

r[class.message.start]
`start(target, argument, ...)` starts the message so that another activity
runs it, and answers nothing; it does not wait for the message to run. The
target and the arguments are taken as `send` takes them, and a message that
has been started is a started message
([the started rule](r:class.message.started)). Until the message ends,
`completed` and `hasResult` answer 0, and `hasError` answers 0 and
`errorCondition` `.nil`. When it ends, the message object holds its answer
or its error as `send` leaves them, but an error of the message is not
raised to the caller of `start`: it is kept, and raised to the caller of
[`result`](r:class.message.result). The notifications that `notify` asked for
are sent when the message has ended. The access check is that of
[Deviation 105](r:class.message.deviation.105).

```rexx
g = .gate~new
q = .message~new(g, 'quick')
signal on syntax name c1
x = q~start
c1: say 'start answered nothing:' condition('o')~code
q~wait
m = .message~new(g, 'work', 'I', 'job')
m~start
g~await
say m~completed m~hasResult m~hasError m~errorCondition~isNil
g~release
say m~result
say m~completed m~hasResult m~hasError
e = .message~new(g, 'boom')
e~start
e~wait
say e~completed e~hasError e~errorCondition~code
::class gate
::method init
  expose up go
  up = .false
  go = .false
::method await unguarded
  expose up
  guard on when up
::method release unguarded
  expose go
  go = .true
::method work unguarded
  expose up go
  up = .true
  guard on when go
  return 'worked on' arg(1)
::method quick unguarded
  return 'quick'
::method boom unguarded
  return 1/0
```

```output
start answered nothing: 91.999
0 0 0 1
worked on job
1 1 0
1 1 42.3
```

> **Note.** The error of a started message that nobody asks for is reported on
> the error stream when the message ends, and the line that the report names
> is the case of [Deviation
> 67](../deviations.md#67-the-line-a-started-messages-error-names): ooRexx's
> report often names the line of the activity that waits for the message, and
> the report of interrexx names the line that raised the error.

r[class.message.startwith]
`startWith(target, array)` is `start` with the arguments in an array, as
`sendWith` is `send`: the arguments always replace those of the message, the
array is required (`SYNTAX 88.901`), a third argument raises `93.902`, and an
array that is not one, `.nil` for instance, or has more than one dimension
raises `88.913`. These errors are raised by `startWith` itself, at the call,
and the message is not started when one is raised, though a target given to
the call has replaced the message's target. The answer is nothing, and
the rest is as for [`start`](r:class.message.start).

```rexx
t = .t~new
m = .message~new(t, 'show', 'I', 'old')
m~startWith(, .array~of(1, 2, 3))
say m~result
call try '.message~new(t, "show")~startWith'
call try '.message~new(t, "show")~startWith(t)'
call try '.message~new(t, "show")~startWith(t, .nil)'
call try '.message~new(t, "show")~startWith(t, .array~new(2, 2))'
call try '.message~new(t, "show")~startWith(t, .array~new, 1)'
n = .message~new(t, "show")
call try 'n~startWith(t, .nil)'
n~startWith(t, .array~of(9))
say n~result
r = .message~new(t, 'show')
call try 'r~startWith(.u~new, .nil)'
say r~target~class~id
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method show
  s = arg()':'
  do i = 1 to arg()
    if arg(i, 'e') then s = s arg(i)
    else s = s '.'
  end
  return s
::class u subclass t
```

```output
3: 1 2 3
.message~new(t, "show")~startWith => 88.901
.message~new(t, "show")~startWith(t) => 88.901
.message~new(t, "show")~startWith(t, .nil) => 88.913
.message~new(t, "show")~startWith(t, .array~new(2, 2)) => 88.913
.message~new(t, "show")~startWith(t, .array~new, 1) => 93.902
n~startWith(t, .nil) => 88.913
1: 9
r~startWith(.u~new, .nil) => 88.913
U
```

r[class.message.startwith.sends]
`startWith` sends its second argument the message `request` with the argument
`ARRAY`, once, at the call, before the message is started and not when it
runs.

```rexx
t = .t~new
m = .message~new(t, 'count')
m~startWith(, .logged~new)
say 'started'
say m~result
::class t
::method count
  return 'count got' arg()
::class logged
::method request
  say 'request' arg(1)
  forward class (super)
::method makearray
  say 'makearray'
  return .array~of(1, 2)
```

```output
request ARRAY
makearray
started
count got 2
```

r[class.message.reply]
`reply(target, argument, ...)` makes a copy of the message, starts the copy
as `start` does, and answers the copy at once, so that the sender goes on.
The target and arguments are taken as `send` takes them, and when they are
given they replace those of the message and of the copy alike. The copy has
the same name and notifications as the message, is a started message
([the started rule](r:class.message.started)), and is the object that
holds the answer or the error of the run: the message that received `reply`
is not run, and does not complete. It can therefore be given `reply` again,
which starts another copy, and it is [`send`](r:class.message.send) or
[`start`](r:class.message.start) that run the message object itself. A
program that waits for the answer asks the copy.

```rexx
t = .t~new
n = .message~new(t, 'show', 'I', 'b')
c = n~reply
say c~class~id (c == n) c~result
say n~completed n~hasResult c~completed c~hasResult
c2 = n~reply
say c2~result (c2 == c)
d = n~reply(.u~new, 'z')
say d~result n~target~class~id n~arguments[1] d~target~class~id
e = n~reply(, 'w1', 'w2')
say e~result n~arguments~items e~arguments~items
n~send
say n~result n~completed
::class t
::method show
  return self~class~id arg() arg(1)
::class u subclass t
```

```output
Message 0 T 1 b
0 0 1 1
T 1 b 0
U 1 z U z U
U 2 w1 2 2
U 2 w1 1
```

r[class.message.replywith]
`replyWith(target, array)` is `reply` with the arguments in an array, as
`startWith` is `start`: the array is required (`SYNTAX 88.901`), a third
argument raises `93.902`, and an array that is not one raises `88.913`, all at
the call. The arguments of the message and of the copy are those of the
array, copied and counted as for [`sendWith`](r:class.message.sendwith), and
the copy is answered.

```rexx
t = .t~new
m = .message~new(t, 'show', 'I', 'old')
c = m~replyWith(, .array~of('a', 'b'))
say c~result m~arguments~items
call try 'm~replyWith'
call try 'm~replyWith(t)'
call try 'm~replyWith(t, .nil)'
call try 'm~replyWith(t, .array~new, 1)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method show
  return arg() arg(1) arg(2)
```

```output
2 a b 2
m~replyWith => 88.901
m~replyWith(t) => 88.901
m~replyWith(t, .nil) => 88.913
m~replyWith(t, .array~new, 1) => 93.902
```

r[class.message.replywith.sends]
`replyWith` sends its second argument the message `request` with the argument
`ARRAY`, once, at the call. An instance of a subclass of `List` is sent it as
well ([Deviation 118](r:class.message.deviation.118)).

```rexx
t = .t~new
m = .message~new(t, 'count')
c = m~replyWith(, .logged~new)
say 'replied'
say c~result
::class t
::method count
  return 'count got' arg()
::class logged
::method request
  say 'request' arg(1)
  forward class (super)
::method makearray
  say 'makearray'
  return .array~of(1, 2)
```

```output
request ARRAY
makearray
replied
count got 2
```

### Waiting for the message

r[class.message.result]
`result` answers what the message answered, and waits first when the message
has been started and has not ended. When the message answered nothing it
answers `.nil`, so that `hasResult` is the way to tell the two apart. When the
latest run ended with an error, `result` raises that error again, in the
activity of its caller, with the code of the error that the message raised:
the trap there sees a new condition object each time, with the `CODE`, `RC`,
`CONDITION`, `DESCRIPTION`, `ADDITIONAL`, `ERRORTEXT` and `MESSAGE` of the
object that `errorCondition` answers, which is the same object each time, so
that the `SYNTAX 42.3` of a division by zero in the message is `42.3` at the
clause that asked for `result`, and asking again raises it again. `result`
takes no argument. What `result` does for a message that nothing will run,
such as one that has never been sent or the original of a `reply`, is the case
of [Deviation 20](../deviations.md#20-a-wait-nothing-can-end).

```rexx
t = .t~new
m = .message~new(t, 'show')
m~send
say m~result m~result
say .message~new(t, 'none')~~send~result~isNil
e = t~start('boom')
signal on syntax name c1
say e~result
c1: say 'raised' condition('o')~code
signal on syntax name c2
say e~result
c2: say 'again' condition('o')~code
n = t~start('nosuchzz')
signal on syntax name c3
say n~result
c3: say 'raised' condition('o')~code
call try 'm~result(1)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method show unguarded
  return 'shown'
::method none unguarded
::method boom unguarded
  return 1/0
```

```output
shown shown
1
raised 42.3
again 42.3
raised 97.1
m~result(1) => 93.902
```

r[class.message.wait]
`wait` waits until the message has ended, answers nothing and raises no error,
also when the message ended with one: the error is asked for with `hasError`
and `errorCondition`, or raised by `result`. It returns at once for a message
that has ended. It takes no argument. What `wait` does for a message that
nothing will run, such as one that has never been sent or the original of a
`reply`, is the case of
[Deviation 20](../deviations.md#20-a-wait-nothing-can-end).

```rexx
g = .gate~new
m = .message~new(g, 'work')
m~start
g~await
r = .message~new(g, 'release')
r~start
m~wait
say m~completed m~result
m~wait
e = g~start('boom')
e~wait
say e~hasError
signal on syntax name c1
x = m~wait
c1: say 'wait answered nothing:' condition('o')~code
call try 'm~wait(1)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class gate
::method init
  expose up go
  up = .false
  go = .false
::method await unguarded
  expose up
  guard on when up
::method release unguarded
  expose go
  go = .true
::method work unguarded
  expose up go
  up = .true
  guard on when go
  return 'worked'
::method boom unguarded
  return 1/0
```

```output
1 worked
1
wait answered nothing: 91.999
m~wait(1) => 93.902
```

r[class.message.halt]
`halt(description)` asks for the HALT condition to be raised in the activity
that is running the message, and answers 1 when a run of the message is in
progress, started or sent (a sent message runs in the sender's activity, which
then gets the HALT), and 0 when the message has not been run. The description
is optional and is a string, which the condition has as `CONDITION('D')`;
without one that is the null string. `.nil` as the description raises
`SYNTAX 88.909`, and a second argument `93.902`. Code of the message that
traps HALT, as [the HALT page](r:cond.halt.signal) describes, handles it as
for any HALT. A HALT nothing traps ends the message with the error `4.1`,
which `hasError`, `errorCondition` and `result` then show as for any other
error of a message. For a message that has ended, `halt` answers 1: a message
that `send` ran gets the HALT raised in the activity that sent it, with the
description, and that activity ends with error `4.1` when nothing traps it,
and for a message that was started nothing happens, so that `hasError`,
`hasResult` and `result` stay as they were. The second example shows both.

```rexx
g = .gate~new
m = .message~new(g, 'work')
say 'not started:' m~halt
m~start
g~await
call try 'm~halt(.nil)'
call try 'm~halt("a", "b")'
say 'halt:' m~halt('time is up')
say m~result
say m~completed m~hasError m~hasResult
h = .gate~new
w = .message~new(h, 'work')
w~start
h~await
w~halt
say w~result
n = .message~new(g, 'plain')
n~start
g~await2
say n~halt
n~wait
say n~hasError n~errorCondition~code n~errorCondition~condition
signal on syntax name c1
say n~result
c1: say 'result raises' condition('o')~code
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class gate
::method init
  expose up up2
  up = .false
  up2 = .false
::method await unguarded
  expose up
  guard on when up
::method await2 unguarded
  expose up2
  guard on when up2
::method work unguarded
  expose up
  signal on halt
  up = .true
  do forever
    nop
  end
  halt:
  return 'halted:' condition('d') condition('c')
::method plain unguarded
  expose up2
  up2 = .true
  do forever
    nop
  end
```

```output
not started: 0
m~halt(.nil) => 88.909
m~halt("a", "b") => 93.902
halt: 1
halted: time is up HALT
1 0 1
halted:  HALT
1
1 4.1 SYNTAX
result raises 4.1
```

```rexx
t = .t~new
m = .message~new(t, 'show')
m~send
signal on halt name caught
say 'sent:' m~halt('stop')
nop
say 'not reached'
caught:
say 'HALT' condition('d') m~hasError m~result
signal off halt
s = .message~new(t, 'show')
s~start
s~wait
say 'started:' s~halt('stop2')
nop
say 'continued' s~hasError s~result
::class t
::method show unguarded
  return 'shown'
```

```output
sent: 1
HALT stop 0 shown
started: 1
continued 0 shown
```

### Being told when the message has ended

r[class.message.notify]
`notify(target)` asks for a notification when the message ends: the object
`target` is sent the message `messageComplete` with the message object as its
only argument. The target must be an instance of
[`MessageNotification`](messagenotification.md), a class that inherits it or a
subclass of one; any other object, `.nil` included, raises `SYNTAX 88.914`, no
target `88.901` and a second argument `93.902`. `notify` answers nothing and can
be sent any number of times: the targets are kept in the order of the calls, and
each is sent `messageComplete` each time the message ends, in that order. A run
of `send` or `sendWith` sends them before it returns, in the activity of the
sender, and an error that a `messageComplete` raises is raised there, so that it
ends the `send` clause; the targets after the one that raised it are not sent
`messageComplete`, and when the message itself ended with an error, the sender
sees the error of the `messageComplete`, while `errorCondition` keeps that of
the message. The error does not become an error of a sent message, which has
ended without one. When a message that has ended is given a target, the target
is sent `messageComplete` at once, in the activity that called `notify`. For a
started message the targets are sent it in the activity that ran the message;
when that happens relative to the activity that waits for the message is a
matter of scheduling. For a started message, an error that a `messageComplete`
raises becomes the error of the message: `hasError` answers 1, `errorCondition`
answers the condition of that error and `result` raises it, while `hasResult`
keeps the 1 of the run. The targets after the one that raised it are not sent
`messageComplete`, and the target that raised it is sent it that once
([Deviation 116](r:class.message.deviation.116)). `hasError` answers 1 once the
error has been recorded, which may be after `wait` has returned. A copy made by
`reply` has the targets of the message, and so has the one that
[`copy`](r:class.message.copy) makes. The message the target is sent is the
completed message object, so `completed` answers 1 in it.

```rexx
t = .t~new
m = .message~new(t, 'show', 'I', 'a')
m~notify(.rec~new('one'))
m~notify(.rec~new('two'))
say m~send
say 'after send'
say m~send
f = .message~new(t, 'show', 'I', 'b')
f~notify(.rec~new('first'))
f~notify(.bad~new)
f~notify(.rec~new('third'))
signal on syntax name c1
f~send
say 'not reached'
c1: say 'handler:' condition('o')~code f~completed f~hasError f~result
g = .message~new(t, 'boom')
g~notify(.bad~new)
signal on syntax name c2
g~send
c2: say 'sender' condition('o')~code 'message' g~errorCondition~code
late = .message~new(t, 'show', 'I', 'c')
late~send
late~notify(.rec~new('late'))
say 'registered'
call try 'm~notify'
call try 'm~notify(.nil)'
call try 'm~notify(.object~new)'
call try 'm~notify(.t~new)'
call try 'm~notify(.rec)'
call try 'm~notify(.rec~new("x"), 1)'
call try 'm~notify(.sub~new("s"))'
nm = .message~new(t, "quiet")
call try 'm~notify(nm)'
nm~wait
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method show
  say 'running' arg(1)
  return 'shown' arg(1)
::method quiet unguarded
  return 'quiet'
::method boom
  return 1/0
::class rec inherit MessageNotification
::method init
  expose name
  use arg name
::method messageComplete
  expose name
  use arg msg
  say 'messageComplete' name msg~class~id msg~completed msg~result
::class sub subclass rec
::class bad inherit MessageNotification
::method messageComplete
  raise syntax 93.900 array('refused')
```

```output
running a
messageComplete one Message 1 shown a
messageComplete two Message 1 shown a
shown a
after send
running a
messageComplete one Message 1 shown a
messageComplete two Message 1 shown a
shown a
running b
messageComplete first Message 1 shown b
handler: 93.900 1 0 shown b
sender 93.900 message 42.3
running c
messageComplete late Message 1 shown c
registered
m~notify => 88.901
m~notify(.nil) => 88.914
m~notify(.object~new) => 88.914
m~notify(.t~new) => 88.914
m~notify(.rec) => 88.914
m~notify(.rec~new("x"), 1) => 93.902
messageComplete s Message 1 shown a
m~notify(.sub~new("s")) => ok
m~notify(nm) => ok
```

```rexx
t = .t~new
m = .message~new(t, 'show', 'I', 'a')
r = .rec~new('first')
m~notify(r)
d = m~copy
d~send
c = m~reply
r~await
say c~result
::class t
::method show unguarded
  return 'shown' arg(1)
::class rec inherit MessageNotification
::method init
  expose name count
  use arg name
  count = 0
::method messageComplete unguarded
  expose name count
  say 'messageComplete' name
  count = count + 1
::method await unguarded
  expose count
  guard on when count >= 2
```

```output
messageComplete first
messageComplete first
shown a
```

```rexx
t = .t~new
s = .message~new(t, 'show')
b = .bad~new
r = .rec~new
s~notify(b)
s~notify(r)
s~start
s~wait
do until s~hasError
  call syssleep 0.01
end
say 'failing target sent:' (b~count > 0) 'next target sent:' (r~count > 0)
say s~completed s~hasError s~hasResult
say 'error' s~errorCondition~code
signal on syntax name c1
say s~result
c1: say 'result raised' condition('o')~code
exit
::class t
::method show unguarded
  return 'shown'
::class bad inherit MessageNotification
::method init
  expose n
  n = 0
::method count unguarded
  expose n
  return n
::method messageComplete unguarded
  expose n
  n = n + 1
  raise syntax 93.900 array('refused')
::class rec inherit MessageNotification
::method init
  expose n
  n = 0
::method count unguarded
  expose n
  return n
::method messageComplete unguarded
  expose n
  n = n + 1
```

```output
failing target sent: 1 next target sent: 0
1 1 1
error 93.900
result raised 93.900
```

r[class.message.notify.sends]
`notify` sends the target no message: whether the target is a
`MessageNotification` is decided from its class, so an override of `isA` or
`isInstanceOf` in the target is not sent, and an object that only has a method
`messageComplete` raises `88.914`. The message `messageComplete` is sent to
each target, with the message object as its only argument, when the message
ends, as [the `notify` rule](r:class.message.notify) says, and at once by a
`notify` on a message that has ended. In the example `LIAR` answers 1 to `isA`
and does not inherit the class, and `HONEST` inherits it and answers 0.

```rexx
m = .message~new(.t~new, 'show', 'I', 'a')
call try 'm~notify(.liar~new)'
call try 'm~notify(.duck~new)'
call try 'm~notify(.honest~new)'
say 'before send'
say m~send
say 'ended'
m~notify(.honest~new)
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method show
  return 'shown' arg(1)
::class liar
::method isa
  say 'liar isA' arg(1)~id
  return 1
::method messagecomplete
  say 'liar messageComplete'
::class honest inherit messagenotification
::method isa
  say 'honest isA'
  return 0
::method isinstanceof
  say 'honest isInstanceOf'
  return 0
::method messagecomplete
  say 'honest messageComplete' arg(1)~class~id arg()
::class duck
::method messagecomplete
  say 'duck messageComplete'
```

```output
m~notify(.liar~new) => 88.914
m~notify(.duck~new) => 88.914
m~notify(.honest~new) => ok
before send
honest messageComplete Message 1
shown a
ended
honest messageComplete Message 1
```

r[class.message.messagecomplete]
`messageComplete(source)` is the method of the notification interface that a
message has itself: a message that is the target of a `notify` is sent
`messageComplete` when the message it waited for has ended, and starts. It
runs the message that received it, as `start` would, in another activity, and
answers nothing at once. The argument, the completed message, is not used and
may be omitted; two arguments raise `SYNTAX 93.902`. The message that
receives it is not made a started message, so `send` can run it afterwards,
and one that has been given `start` or `startWith` raises `98.915`, as
[the started rule](r:class.message.started) says.

```rexx
t = .t~new
a = .message~new(t, 'show', 'I', 'first')
b = .message~new(t, 'show', 'I', 'second')
a~notify(b)
a~send
say b~result
c = .message~new(t, 'show', 'I', 'third')
c~messageComplete
say c~result
say c~send
d = .message~new(t, 'show', 'I', 'fourth')
d~messageComplete(.object~new)
say d~result
signal on syntax name c1
d~messageComplete(1, 2)
c1: say condition('o')~code
e = .message~new(t, 'show', 'I', 'fifth')
signal on syntax name c2
x = e~messageComplete(1)
c2: say 'answered nothing:' condition('o')~code
::class t
::method show unguarded
  return 'shown' arg(1)
```

```output
shown second
shown third
shown third
shown fourth
93.902
answered nothing: 91.999
```

r[class.message.triggered]
`triggered(source)` is the method of the alarm notification interface that a
message has itself: an `Alarm` or `Ticker` whose target is the message calls
it when the alarm goes off. It does what
[`messageComplete`](r:class.message.messagecomplete) does: it starts the
message on another activity, answers nothing, takes one optional argument that
it does not use (`SYNTAX 93.902` for two), and raises `98.915` for a message
that was started with `start` or `startWith`. Scheduling, and the classes
`Alarm` and `Ticker`, are specified apart from this page.

```rexx
t = .t~new
m = .message~new(t, 'show', 'I', 'alarm')
m~triggered(.object~new)
say m~result
n = .message~new(t, 'show', 'I', 'byAlarm')
a = .alarm~new(0.02, n, 'att')
say n~result a~attachment
call try 'm~triggered(1, 2)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method show unguarded
  return 'shown' arg(1)
```

```output
shown alarm
shown byAlarm att
m~triggered(1, 2) => 93.902
```

r[class.message.cancel]
`cancel(source)` is the method of the alarm notification interface that is
called when an `Alarm` or `Ticker` is cancelled; it is the method that
[`AlarmNotification`](alarmnotification.md) defines, and a message has no
version of its own. It does nothing and answers nothing, whatever arguments it
is sent: the message is not started, and nothing about it changes.

```rexx
t = .t~new
m = .message~new(t, 'show')
m~cancel
m~cancel(1, 2, 3, 4)
say m~completed m~hasResult
b = .message~new(t, 'show')
a = .alarm~new(60, b)
a~cancel
say b~completed b~hasResult t~runs
signal on syntax name c1
x = m~cancel
c1: say 'answered nothing:' condition('o')~code
::class t
::method init
  expose n
  n = 0
::method show unguarded
  expose n
  n = n + 1
  return 'shown'
::method runs unguarded
  expose n
  return n
```

```output
0 0
0 0 0
answered nothing: 91.999
```

r[class.message.copy]
`copy` answers a message object with the target, the name, the arguments
(the same objects) and the notification targets of the message, as a message
that has not been run: it has not completed, has no result and no error, and
is not a started message even when the message was. A target that `notify`
later adds to one of the two is not added to the other.

```rexx
t = .t~new
s = .message~new(t, 'show', 'I', t)
s~start
s~wait
k = s~copy
say 'copy of started:' k~completed k~hasResult k~hasError
say (k~target == t) (k~arguments[1] == t) k~messageName
say k~send k~completed
m = .message~new(t, 'show', 'I', 1)
m~notify(.rec~new('original'))
c = m~copy
c~notify(.rec~new('copy'))
say '-- send copy'
c~send
say '-- send original'
m~send
::class t
::method show unguarded
  return 'shown'
::class rec inherit MessageNotification
::method init
  expose name
  use arg name
::method messageComplete unguarded
  expose name
  say 'messageComplete' name
```

```output
copy of started: 0 0 0
1 1 SHOW
shown 1
-- send copy
messageComplete original
messageComplete copy
-- send original
messageComplete original
```

## Sends

r[class.message.sends]
No method of `Message` sends a message to the message object itself, so a
subclass that overrides `send`, `start`, `startWith`, `reply`, `replyWith`,
`notify` or any other method sees it called only by a program:
`messageComplete` and `triggered` start the message without sending `start`,
`reply` starts a copy without sending `send` or `start`, and the notification
of a message sent with `notify` sends `messageComplete` to the target, which
is where an override of that method in the target shows. The messages that
methods of the class send to other objects are stated by the rules of
[`new`](r:class.message.new.sends),
[`sendWith`](r:class.message.sendwith.sends),
[`startWith`](r:class.message.startwith.sends),
[`replyWith`](r:class.message.replywith.sends) and
[`notify`](r:class.message.notify).

```rexx
t = .t~new
m = .mine~new(t, 'show', 'I', 'x')
m~messageComplete(1)
m~wait
say '--'
m~triggered(1)
m~wait
say '--'
m~send
m~sendWith(, .array~of(1))
say '--'
c = m~reply
c~wait
say '--'
s = .mine~new(t, 'show', 'I', 's')
s~startWith(, .array~of(1))
s~wait
say '--'
r = .mine~new(t, 'show', 'I', 'r')
c = r~replyWith(, .array~of(1))
c~wait
say '--'
q = .mine~new(t, 'show', 'I', 'q')
q~start
q~wait
say '--'
m~notify(.mine~new(t, 'show', 'I', 'notified'))
m~send
::class t
::method show unguarded
  return 'show' arg(1)
::class mine subclass message
::method send
  say 'send override'
  forward class (super)
::method sendWith
  say 'sendWith override'
  forward class (super)
::method start
  say 'start override'
  forward class (super)
::method startWith
  say 'startWith override'
  forward class (super)
::method reply
  say 'reply override'
  forward class (super)
::method replyWith
  say 'replyWith override'
  forward class (super)
::method notify
  say 'notify override'
  forward class (super)
::method messageComplete
  say 'messageComplete override'
  forward class (super)
::method triggered
  say 'triggered override'
  forward class (super)
```

```output
messageComplete override
--
triggered override
--
send override
sendWith override
--
reply override
--
startWith override
--
replyWith override
--
start override
--
notify override
send override
messageComplete override
```

## Deviations

r[class.message.deviation.105]
A message that `start`, `startWith`, `reply` or `replyWith` runs is checked
for `PRIVATE` and `PACKAGE` access as the same message sent by `send` from the
code that called it, as [the access rule](r:dir.method.access) states for a
started message: the code of a class can start a private message to its own
objects, and a `PACKAGE` message to an object of the same package. ooRexx
raises `97.2` for every private message and `97.1` for every `PACKAGE` message
that is started ([Deviation
105](../deviations.md#105-access-checks-of-started-messages)).

```rexx
o = .w~new
say o~viaStart('secret')
say o~viaStart('pack')
say o~viaReply('secret')
say .message~new(o, 'pack')~~start~result
::class w
::method viaStart unguarded
  m = .message~new(self, arg(1))
  m~start
  return m~result
::method viaReply unguarded
  return .message~new(self, arg(1))~reply~result
::method secret private
  return 'secret ran'
::method pack package
  return 'pack ran'
```

```output
secret ran
pack ran
secret ran
pack ran
```

r[class.message.deviation.107]
A string that names a number is the same string to every message, as
[the Object rule](r:class.object.deviation.107) states, and a number literal
such as `12` is accepted as the name of `new`: the message object has the
name `12`, and runs the method of that name. ooRexx raises `SYNTAX 93.946`
for a name stored as a whole number, as a literal or arithmetic gives, and for
the same characters in quotes once the literal occurs in the program
([Deviation 107](../deviations.md#107-number-literals-are-ordinary-strings)).

```rexx
o = .w~new
m = .message~new(o, 12, 'I', 'a')
say m~messageName m~send
say .message~new(o, 3 + 4, 'I', 'b')~send
::class w
::method 12
  return 'ran 12' arg(1)
::method 7
  return 'ran 7' arg(1)
```

```output
12 ran 12 a
ran 7 b
```

r[class.message.deviation.109]
An instance of a subclass of `String` that is the message name of `new` is
read by its own characters, and `MAKESTRING` and `STRING` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends it `request`
with the argument `ARRAY`, as
[the Object rule](r:class.object.deviation.109) states for `send`, and raises
`SYNTAX 93.946`. In the example `MAKESTRING`
answers another name.

```rexx
m = .message~new(.w~new, .name~new('count'))
say m~messageName m~send
::class w
::method count
  return 'count got' arg()
::class name subclass string
::method makestring
  return 'other'
```

```output defect=#140
COUNT count got 0
```

r[class.message.deviation.113]
`new` with the option `A` keeps a copy of the
array of arguments, so that a change to the caller's array after `new` does not
reach the message, and the number of arguments of the message is the position of
the last item present. The arrays of arguments that `sendWith`, `startWith` and
`replyWith` take are copied and counted the same way. ooRexx keeps the caller's
array itself, so that a change to it reaches the message, and passes the size of
the array as the number of arguments, with the empty places at its end counted,
for all four methods. In the example the array of the first message gets a new
first item and one more item after `new`, and the arrays of the second message
and of `sendWith` have room for four items and hold one.

```rexx
t = .t~new
a = .array~of('x', 'y')
m = .message~new(t, 'show', 'A', a)
a[1] = 'changed'
a~append('z')
say m~send
b = .array~new(4)
b[1] = 'p'
say .message~new(t, 'show', 'A', b)~send
say .message~new(t, 'show')~sendWith(t, b)
::class t
::method show
  s = arg()':'
  do i = 1 to arg()
    if arg(i, 'e') then s = s arg(i)
    else s = s '.'
  end
  return s
```

```output
2: x y
1: p
1: p
```

r[class.message.deviation.116]
A target of a started message whose `messageComplete` raises an error is sent
`messageComplete` once for that message, and no target after it is sent it.
ooRexx sends the target that raised the error `messageComplete` three times in
all, and none after it. The example counts the sends to the failing target after
the error has been recorded, and gives the activity that runs the notifications
time to send any further one.

```rexx
s = .message~new(.t~new, 'show')
b = .bad~new
s~notify(b)
s~start
s~wait
do until s~hasError
  call syssleep 0.01
end
call syssleep 0.3
say 'failing target sent' b~count
::class t
::method show unguarded
  return 'shown'
::class bad inherit MessageNotification
::method init
  expose n
  n = 0
::method count unguarded
  expose n
  return n
::method messageComplete unguarded
  expose n
  n = n + 1
  raise syntax 93.900 array('refused')
```

```output
failing target sent 1
```

r[class.message.deviation.118]
`new` with the option `A`, `sendWith`, `startWith` and `replyWith` send an
instance of a subclass of `List` given as the array of arguments the message
`request` with the argument `ARRAY`, as [the sends rule of
`sendWith`](r:class.message.sendwith.sends) states for any other object, so
that an override is used ([Deviation 118](../deviations.md)). ooRexx sends the
instance no `request` and uses the items it holds, so each line of the example
answers 1 on ooRexx. The `request` of the class `ll` answers two items.

```rexx
l = .ll~new
l~append('held')
t = .w~new
m = .message~new(t, 'count', 'A', l)
say m~send
m = .message~new(t, 'count')
say m~sendWith(t, l)
m = .message~new(t, 'count')
m~startWith(t, l)
say m~result
c = .message~new(t, 'count')~replyWith(t, l)
say c~result
::class ll subclass list
::method request
  say 'request' arg(1)
  return .array~of('a', 'b')
::class w
::method count
  return arg()
```

```output
request ARRAY
2
request ARRAY
2
request ARRAY
2
request ARRAY
2
```

r[class.message.deviation.124]
An answer of `request` with `ARRAY` that is not an `Array` or an instance of a
subclass of `Array` is refused as the array of arguments by `new` with the
option `A`, `sendWith`, `startWith` and `replyWith`, which raise `SYNTAX
88.913`, whatever collection the answer is. ooRexx accepts a `Queue` or a
`CircularQueue` as such an answer, but passes arguments by the storage the queue
holds rather than by its items: a new queue of two items gives the two items
followed by fourteen omitted arguments. It crashes for a `List`, a `Bag`, a
`Table` or a `Directory`, among others. In the example the answer is a queue.

```rexx
a = .ans~new
t = .w~new
call try ".message~new(t, 'count', 'A', a)"
call try ".message~new(t, 'count')~sendWith(t, a)"
call try ".message~new(t, 'count')~startWith(t, a)"
call try ".message~new(t, 'count')~replyWith(t, a)"
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class w
::method count
  return arg()
::class ans
::method request
  return .queue~of('x')
```

```output
.message~new(t, 'count', 'A', a) => 88.913
.message~new(t, 'count')~sendWith(t, a) => 88.913
.message~new(t, 'count')~startWith(t, a) => 88.913
.message~new(t, 'count')~replyWith(t, a) => 88.913
```
