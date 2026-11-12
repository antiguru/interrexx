---
element: "Monitor"
kind: class
rexxref: "Builtin Classes > Utility Classes > Monitor Class; Builtin Classes > Utility Classes > Monitor Class > current; Builtin Classes > Utility Classes > Monitor Class > destination; Builtin Classes > Utility Classes > Monitor Class > init; Builtin Classes > Utility Classes > Monitor Class > unknown"
deviations: [118, 124]
---

# Monitor

A `Monitor` stands in for another object. A message that the monitor does not
have a method for is sent on to the object that is the monitor's current
destination, with the same name and the same arguments, and what that answers
is what the monitor answers. The destination can be changed while the program
runs, and put back to the one before: a program that holds a monitor, or that
has made one the value of a name such as `.output` that other code reads,
changes where the messages go without that code noticing. A monitor is
transparent only for messages that it does not itself understand: the methods
of `Object` are the monitor's own, and [the rule on
them](r:class.monitor.object) says which messages are not forwarded. The
forwarding is the `unknown` method, which [the method
search](r:class.methods.unknown) runs when it finds no method.

## The class

r[class.monitor.creation]
`Monitor` has the instance methods `current`, `destination`, `init` and
`unknown`, besides the `new` that every class has. It may have private methods
besides these; they are not part of this specification. Its only superclass is
`Object` and it inherits no mixin class ([the hierarchy
rule](r:class.index.hierarchy)). `new` is the one [that `Class`
supplies](r:class.index.new): it sends `init` the arguments of `new`, and
answers an instance of the class that received it, so a subclass of `Monitor`
makes instances of itself; methods that a subclass declares are found before
`unknown`, so a message for which it has a method is not forwarded.

```rexx
say .monitor~id .monitor~superclasses~items .monitor~superclass~id,
  .monitor~queryMixinClass
have = .set~new
s = .monitor~methods(.nil)
do while s~available
  have~put(s~index)
  s~next
end
names = 'current destination init unknown'~makeArray(' ')
count = 0
do name over names
  if have~hasItem(name~upper) then count = count + 1
end
say count 'of 4'
t = .target~new('T')
m = .monitor~new(t)
x = .sub~new(t)
say m~class~id x~class~id
say m~hello x~hello x~own
::class target
::method init
  expose name
  use arg name
::method hello
  expose name
  return name 'hello'
::class sub subclass monitor
::method hello
  return 'own hello'
::method own
  return 'own'
```

```output
Monitor 1 Object 0
4 of 4
Monitor SUB
T hello own hello own
```

## The destination

r[class.monitor.init]
`init` sets the destination to its argument. It takes at most one argument
(`SYNTAX 93.902`); the argument can be any object, strings, the empty string
and `.nil` among them, and when it is omitted the destination is `.nil`. It
replaces every destination that was set before, including those that
[`destination`](r:class.monitor.destination) put on top of the first, so that
the destination that comes back after the last one has been dropped is `.nil`.
`init` answers nothing. Because `new` sends `init` the arguments of `new`,
`new` takes the same argument and raises the same error, and an instance made
by `new` with no argument has no destination until one is set. A message that
is forwarded while the destination is `.nil` raises `SYNTAX 97.1`, as [the
NOMETHOD rule](r:class.index.errors) says of a message that nothing
understands.

```rexx
a = .target~new('A')
b = .target~new('B')
m = .monitor~new
say (m~current == .nil) .monitor~new(a)~current~id
call try "m~hello"
call try ".monitor~new(a, b)"
call try "m~init(a, b)"
call try "m~init"
call try "m~init(.nil)"
say (m~current == .nil)
call try "m~init(a)"
say m~hello m~current~id
call try "m~init('')"
say '['m~current']' m~current~class~id
call try "m~init(a)"
r = m~destination(b)
call try "m~init(a)"
r = m~destination
say (r == .nil) (m~current == .nil)
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=>' x
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class target
::method init
  expose name
  use arg name
::method id
  expose name
  return name
::method hello
  expose name
  return name 'hello'
```

```output
1 A
m~hello => 97.1
.monitor~new(a, b) => 93.902
m~init(a, b) => 93.902
m~init => 91.999
m~init(.nil) => 91.999
1
m~init(a) => 91.999
A hello A
m~init('') => 91.999
[] String
m~init(a) => 91.999
m~init(a) => 91.999
1 1
```

r[class.monitor.current]
`current` answers the destination that messages are forwarded to now: the
object `init` was given, or the object that the last
[`destination`](r:class.monitor.destination) put on top, or `.nil` when there
is none. It takes no argument (`SYNTAX 93.902`), changes nothing and sends no
message.

```rexx
a = .target~new('A')
m = .monitor~new(a)
say m~current~id (m~current == a)
signal on syntax name bad
say m~current(1)
bad: say condition('o')~code
::class target
::method init
  expose name
  use arg name
::method id
  expose name
  return name
```

```output
A 1
93.902
```

r[class.monitor.destination]
`destination` changes the destination, and answers the one that is current
afterwards. The destinations form a stack, with the one that `init` set at the
bottom. With an argument, `destination` puts that object on top: from then on
messages go to it, and the earlier destinations are kept below it. The
argument can be any object, `.nil` and the empty string among them, and what
`destination` answers is that same object. Without an argument it takes the
top one off, so that the destination that was current before the last change
is current again, and answers that one; when no destination is left under the
one it took off it answers `.nil`, and the destination is `.nil` and stays so,
for a further `destination` without an argument takes nothing more off and
answers `.nil` again. A destination put on top after that is the only one.
More than one argument raises `SYNTAX 93.902`. `destination` sends no message,
not `current` and not `init`.

```rexx
a = .target~new('A')
b = .target~new('B')
c = .target~new('C')
m = .monitor~new(a)
say m~destination(b)~id m~destination(c)~id m~hello
say m~destination~id m~hello
say m~destination~id m~hello
r = m~destination
say (r == .nil) (m~current == .nil)
r = m~destination
say (r == .nil) (m~current == .nil)
say m~destination(c)~id m~hello
r = m~destination
say (r == .nil)
n = .monitor~new
say (n~destination == .nil) (n~destination(a) == a) n~current~id
say '['n~destination('')']' n~length
signal on syntax name bad
r = n~destination(a, b)
bad: say condition('o')~code
::class target
::method init
  expose name
  use arg name
::method id
  expose name
  return name
::method hello
  expose name
  return name 'hello'
```

```output
B C C hello
B B hello
A A hello
1 1
1 1
C C hello
1
1 1 A
[] 0
93.902
```

## Forwarding

r[class.monitor.unknown]
`unknown(name, arguments)` forwards a message. It is run by [the method
search](r:class.methods.unknown) for a message the monitor has no method for,
with the name in upper case and the array of the arguments that the search
builds, and it can be sent directly with the same two arguments. It sends the
current destination a message with that name and the items of the array as its
arguments, and answers what that message answers; when the message answers
nothing, `unknown` answers nothing. The arguments are passed on as they were
sent: an omitted argument stays omitted, so that the destination counts the
arguments by the last one present, and a trailing omitted argument is not
counted. A condition that the destination's method raises is raised for the
message to the monitor in the same way, an error number and subcode included.
When the destination does not understand the name, or is `.nil`, `SYNTAX 97.1`
is raised; and a method of the destination that is `PRIVATE` or `PACKAGE` is
not available to the monitor, which is not the destination and has not the
destination's package: the message raises `SYNTAX 97.2` or `97.3` as it would
for any other sender ([the access rule](r:dir.method.access)). A method that
the destination has on its class side is not reached. A destination that has
an `unknown` method of its own gets that message in turn, with the name in
upper case. Called directly, `unknown` needs both arguments (`SYNTAX 93.901`),
takes no more (`SYNTAX 93.902`), and needs the name to be present
(`SYNTAX 93.903`). The second argument of a direct call is any object, and it
is sent `request` with `ARRAY` before anything is forwarded; [the arguments
rule](r:class.monitor.unknown.arguments) says what that makes of it.

```rexx
a = .target~new('A')
m = .monitor~new(a)
say m~hello(1, 2)
say m~hello(, 2)
say m~hello(1, , 3, )
say m~hello
say m~Hello(1) m~'hello'(1)
say m~unknown('hello', .array~of(7, 8))
call try "m~fail"
call try "m~priv"
call try "m~pk"
call try "m~classside"
call try "m~nosuchzz"
call try "m~silent"
call try "m~unknown('hello')"
call try "m~unknown"
call try "m~unknown('hello', .array~of(1), 'x')"
call try "m~unknown(, .array~of(1))"
call try ".monitor~new~hello"
t = .monitor~new(.catcher~new)
say t~whatsoever(4, 5) t~Mixed(6)
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=>' x
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class target
::method init
  expose name
  use arg name
::method hello
  expose name
  return name 'hello' arg() arg(1, 'E') arg(2, 'E') arg(3, 'E')
::method fail
  raise syntax 40.900 array('boom')
::method priv private
  return 'private'
::method pk package
  return 'package'
::method classside class
  return 'class side'
::method silent
  return
::class catcher
::method unknown
  use arg name, args
  return 'caught' name args~items
```

```output
A hello 2 1 1 0
A hello 2 0 1 0
A hello 3 1 0 1
A hello 0 0 0 0
A hello 1 1 0 0 A hello 1 1 0 0
A hello 2 1 1 0
m~fail => 40.900
m~priv => 97.2
m~pk => 97.3
m~classside => 97.1
m~nosuchzz => 97.1
m~silent => 91.999
m~unknown('hello') => 93.901
m~unknown => 93.901
m~unknown('hello', .array~of(1), 'x') => 93.902
m~unknown(, .array~of(1)) => 93.903
.monitor~new~hello => 97.1
caught WHATSOEVER 2 caught MIXED 1
```

r[class.monitor.unknown.arguments]
The second argument of a direct `unknown` need not be an `Array`: it is any
object, and it is sent `request` with `ARRAY` before anything is forwarded, as
[the unknown rule](r:class.monitor.unknown) says. The items of what that
answers, in order and with their holes, are the arguments of the forwarded
message. A string answers the array of its lines, so that a string without a
line end is one argument, the null string is none, and a string with line ends
is as many arguments as it has lines; a `MutableBuffer` likewise, by its
string; a number and `.true` are one argument each, as their strings;
a `List` or a `Set` answers its items;
and an object that answers no array to `request`, `.nil` and an `Object` among
them, raises `SYNTAX 98.946` and forwards nothing.

```rexx
m = .monitor~new(.target~new)
call try "m~unknown('echo', 'abc')"
call try "m~unknown('echo', 'a b  c')"
call try "m~unknown('echo', '')"
call try "m~unknown('echo', 5)"
call try "m~unknown('echo', .list~of('x', 'y'))"
call try "m~unknown('echo', .set~of('x'))"
call try "m~unknown('echo', .object~new)"
call try "m~unknown('echo', .nil)"
call try "m~unknown('echo', .true)"
call try "m~unknown('echo', .array~of('p', , 'r'))"
call try "m~unknown('echo', .mutablebuffer~new('q'))"
call try "m~unknown('echo', 'a'||'0a'x||'b')"
call try "m~unknown('echo', .mutablebuffer~new('c'||'0d0a'x||'d'))"
call try "m~unknown('echo', 'a'||'0a'x)"
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=>' x
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class target
::method echo
  return arg() '['arg(1)']' '['arg(2)']' '['arg(3)']'
```

```output
m~unknown('echo', 'abc') => 1 [abc] [] []
m~unknown('echo', 'a b  c') => 1 [a b  c] [] []
m~unknown('echo', '') => 0 [] [] []
m~unknown('echo', 5) => 1 [5] [] []
m~unknown('echo', .list~of('x', 'y')) => 2 [x] [y] []
m~unknown('echo', .set~of('x')) => 1 [x] [] []
m~unknown('echo', .object~new) => 98.946
m~unknown('echo', .nil) => 98.946
m~unknown('echo', .true) => 1 [1] [] []
m~unknown('echo', .array~of('p', , 'r')) => 3 [p] [] [r]
m~unknown('echo', .mutablebuffer~new('q')) => 1 [q] [] []
m~unknown('echo', 'a'||'0a'x||'b') => 2 [a] [b] []
m~unknown('echo', .mutablebuffer~new('c'||'0d0a'x||'d')) => 2 [c] [d] []
m~unknown('echo', 'a'||'0a'x) => 1 [a] [] []
```

r[class.monitor.object]
Every instance method that `Object` defines ([the Object page](object.md)) is
the monitor's own, so a message with the name of one is not forwarded: it
answers as it would for any object, about the monitor and not about the
destination. That includes `class`, `copy`, `defaultName`, `hasMethod`, `isA`,
`isInstanceOf`, `objectName` and `objectName=`, `send` and `sendWith`,
`string` and the comparison operators. A program that wants the destination's
own answer to one of them sends it to `current`. `send` and `sendWith` send
the message to the monitor itself, so they reach `unknown` for a name that the
monitor has no method for. Which names a subclass or an enhanced monitor has
methods for, besides these, is found by [the method
search](r:class.methods.search) in the usual way.

```rexx
a = .target~new('A')
m = .monitor~new(a)
say m~class~id m~string m~isA(.monitor) m~isA(.target) m~isInstanceOf(.target)
say m~hasMethod('hello') m~hasMethod('unknown') a~hasMethod('hello')
say (m = m) (m == m) (m = a) (m == a) (m \= a)
say m~objectName '/' a~objectName
m~objectName = 'the monitor'
say m~objectName '/' a~objectName '/' m~current~objectName
say m~send('hello') '/' m~sendWith('hello', .array~new)
say m~copy~class~id (m~copy~current == a) (m~copy == m)
say m~defaultName a~defaultName
say m~id
::class target
::method init
  expose name
  use arg name
::method id
  expose name
  return name
::method hello
  expose name
  return name 'hello'
```

```output
Monitor a Monitor 1 0 0
0 1 1
1 1 0 0 1
a Monitor / a TARGET
the monitor / a TARGET / a TARGET
A hello / A hello
Monitor 1 0
a Monitor a TARGET
A
```

r[class.monitor.copy]
`copy` answers a monitor that shares the stack of destinations with the
original, also when that stack holds only `.nil`: a `destination` sent to either
one, with an argument or without, changes where both forward. `init` gives the
monitor that receives it a stack of its own, and from then on the two are
independent.

```rexx
a = .target~new('A'); b = .target~new('B'); c = .target~new('C')
m = .monitor~new(a)
k = m~copy
x = k~destination(b)
say m~current~id k~current~id
x = m~destination
say m~current~id k~current~id
x = k~destination(b)
k~init(c)
say m~current~id k~current~id
x = k~destination(a)
say m~current~id k~current~id
x = m~destination(b)
say m~current~id k~current~id
n = .monitor~new
j = n~copy
x = j~destination(a)
say (n~current == .nil) j~current~id
::class target
::method init
  expose name
  use arg name
::method id
  expose name
  return name
```

```output
B B
A A
B C
B A
B A
0 A
```

r[class.monitor.sends]
The messages that a monitor sends are the one that `unknown` sends to the
destination, which [the forwarding rule](r:class.monitor.unknown) states, and
that destination's `unknown` when it has to answer for a name. The monitor
sends none to itself: `unknown` does not ask `current` for the destination,
`init` and `destination` do not send each other, and a subclass that overrides
`current` or `destination` sees them run only when the program sends them. The
method `unknown` of a subclass that forwards with
`self~unknown:super(name, arguments)` is found by [the method
search](r:class.methods.unknown) before the one of `Monitor`. The example
overrides each message of a subclass and shows them from a forwarded message,
`destination` and `current`.

```rexx
.environment~log = .list~new
a = .target~new('A')
m = .logging~new(a)
say 'init:' .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
say m~hello
say 'forward:' .environment~log~makeArray~makeString('l', ' ')
.environment~log~empty
x = m~destination(a)
x = m~destination
x = m~current
say 'direct:' .environment~log~makeArray~makeString('l', ' ')
::class target
::method init
  expose name
  use arg name
::method hello
  expose name
  return name 'hello'
::class logging subclass monitor
::method init
  .environment~log~append('init')
  forward class (super)
::method unknown
  .environment~log~append('unknown' arg(1))
  forward class (super)
::method current
  .environment~log~append('current')
  forward class (super)
::method destination
  .environment~log~append('destination')
  forward class (super)
```

```output
init: init
A hello
forward: unknown HELLO
direct: destination destination current
```

r[class.monitor.deviation.118]
An instance of a subclass of `List` that is given to `unknown` as the array of
arguments is sent `request` with `ARRAY`, as [the unknown
rule](r:class.monitor.unknown) says of any other object, so that an override
is used. See [Deviation 118](../deviations.md). ooRexx sends the instance no
`request` and passes the items it holds, so the example answers `hello 2 a` on
ooRexx. The `request` of the class `ml` answers one item.

```rexx
m = .monitor~new(.target~new)
say m~unknown('HELLO', .ml~of('a', 'b'))
::class target
::method hello
  return 'hello' arg() arg(1)
::class ml subclass list
::method request
  say 'request' arg(1)
  return .array~of('R')
```

```output
request ARRAY
hello 1 R
```

r[class.monitor.deviation.124]
An answer of `request` with `ARRAY` that is not an `Array` or an instance of a
subclass of `Array` is refused as the array of arguments of `unknown`, which
raises `SYNTAX 98.946`, whatever collection the answer is. ooRexx accepts a
`Queue` or a `CircularQueue` as such an answer and uses its items, and crashes
for a `List`, a `Bag`, a `Table` or a `Directory`, among others. In the example
the answer is a queue.

```rexx
m = .monitor~new(.target~new)
signal on syntax name bad
say m~unknown('HELLO', .ans~new)
exit
bad: say condition('o')~code
::class target
::method hello
  return 'hello' arg()
::class ans
::method request
  return .queue~of('x')
```

```output
98.946
```
