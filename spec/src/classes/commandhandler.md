---
element: "CommandHandler"
kind: class
deviations: [109]
rexxref: "Builtin Classes > Utility Classes > CommandHandler Class; Builtin Classes > Utility Classes > CommandHandler Class > deregister (Class Method); Builtin Classes > Utility Classes > CommandHandler Class > environments (Class Method); Builtin Classes > Utility Classes > CommandHandler Class > handler (Class Method); Builtin Classes > Utility Classes > CommandHandler Class > handlers (Class Method); Builtin Classes > Utility Classes > CommandHandler Class > register (Class Method); Builtin Classes > Utility Classes > CommandHandler Class > command"
---

# CommandHandler

A command that a program sends to an environment is run by whatever the
environment is. `CommandHandler` is the mixin class for environments written
in Rexx: an object whose class inherits it, registered under a name, receives
every command that is sent to that name as a message, and answers with the
return code of the command. The class methods register such an object, remove
it, and list the environments of the interpreter, the native ones that run
commands in the host's shell among them; the objects that stand for those are
the [`NativeCommandHandler`](nativecommandhandler.md) objects. While a command
runs, the handler gets a [`CommandContext`](commandcontext.md) that describes
it.

This page states the class, how a command reaches the method `command` of the
handler and what the handler answers, and the class methods. The environment
chapter specifies the environments of the interpreter, how a command to a name
that no environment has fails and what the native environments do; this page
says only where the class takes part. The examples register environments under
names of their own, and use no environment that runs commands in the host.

## The class

r[class.commandhandler.creation]
`CommandHandler` is a mixin class: it has the one instance method `command`
and the class methods `deregister`, `environments`, `handler`, `handlers` and
`register`, besides the methods that every class has. Its superclass is
`Object`, and it inherits no mixin of its own ([the hierarchy
rule](r:class.index.hierarchy)). A class that inherits it gets the instance
method `command` and the class methods too, so a program may send `register`
to its own handler class as well as to `CommandHandler`, and all of them work
with the same environments. A class method is not an instance method: an
instance of such a class does not understand `register`. `new` sent to
`CommandHandler` makes an instance, as it does for any class that is not
abstract, and `command` sent to that instance is the abstract method that [the
`command` rule](r:class.commandhandler.command) states. The class may have
private methods besides these; they are not part of this specification.

```rexx
say .commandhandler~id .commandhandler~superclasses~items,
  .commandhandler~superclass~id .commandhandler~queryMixinClass
h = .commandhandler~new
say h~class~id h~hasMethod('COMMAND') h~hasMethod('REGISTER')
say .commandhandler~hasMethod('REGISTER') .commandhandler~hasMethod('COMMAND')
say .echo~hasMethod('REGISTER') .echo~hasMethod('HANDLERS'),
  .echo~hasMethod('COMMAND')
say .echo~new~hasMethod('REGISTER') .echo~new~hasMethod('COMMAND')
say .echo~register('spec_ch_creation', .echo~new)~isNil
say .commandhandler~handler('spec_ch_creation')~class~id,
  .echo~handler('SPEC_CH_CREATION')~class~id
say .echo~deregister('spec_ch_creation')~class~id
::class echo inherit CommandHandler
::method command
  return 0
```

```output
CommandHandler 1 Object 1
CommandHandler 1 0
1 0
1 1 0
0 1
1
ECHO ECHO
ECHO
```

## The method command

r[class.commandhandler.command]
`command(command, caller)` is the method that a class implements to run
commands. `CommandHandler` defines it as an abstract method: sent to an
instance whose class has not replaced it, it raises `SYNTAX 93.965`, and so
does a command that is sent to a handler like that. A command is therefore an
ordinary message to the handler object, answered by the first `command` method
that the [method search](methods.md) finds for it, a subclass's before the
class it extends, and `command` of a superclass reached with
`self~command:super` as for any method. The arguments and what the method
answers are stated in [the dispatch rule](r:class.commandhandler.dispatch) and
[the return code rule](r:class.commandhandler.rc).

```rexx
.commandhandler~register('spec_ch_abstract', .lazy~new)
signal on syntax name direct
.commandhandler~new~command('x', .nil)
direct: say 'direct' condition('o')~code
signal on syntax name sent
address spec_ch_abstract 'x'
sent: say 'sent' condition('o')~code
signal off syntax
.commandhandler~register('spec_ch_sub', .sub~new)
address spec_ch_sub 'abc'
say rc
x = .commandhandler~deregister('spec_ch_abstract')
x = .commandhandler~deregister('spec_ch_sub')
::class lazy inherit CommandHandler
::class base inherit CommandHandler
::method command
  use arg command, caller
  return command~length
::class sub subclass base
::method command
  use arg command, caller
  return self~command:super(command command, caller) + 100
```

```output
direct 93.965
sent 93.965
107
```

r[class.commandhandler.dispatch]
When a command is sent to an environment name for which a handler is in use, by
[`ADDRESS name expression`](r:instr.address.command) or by a [command
clause](r:instr.command.send) while that name is the current environment, the
handler is sent the message `command` with two arguments: the command, as the
string that [the command rule](r:instr.command.string) makes of the value of the
expression, and a new `CommandContext` for this command. The message is sent
from the activity that sends the command, and the clause that sent it waits for
the answer. The name of the environment may be written in any case: the handler
gets the same command either way, and the context names the environment as [the
environment rule](r:class.commandcontext.environment) says. A command that the
handler sends from inside `command`, whichever environment it names, is
dispatched by this same rule: it gets a context of its own, and one handler can
be running several commands at once, one inside another. A value that is the
null string is sent as a command, and arrives as the null string. In the example
the handler registered under two names tells them apart by `caller~environment`,
runs a command inside a command, and shows the arguments it was sent.

```rexx
h = .count~new
x = .commandhandler~register('spec_ch_dispatch', h)
x = .commandhandler~register('spec_ch_other', h)
address spec_ch_dispatch 'first command'
address SPEC_CH_OTHER 'second' 1 + 2
address spec_ch_other ''
address spec_ch_dispatch .text~new
address spec_ch_dispatch
'a clause'
say 'rc' rc address()
address spec_ch_dispatch 'nest 2'
x = .commandhandler~deregister('spec_ch_dispatch')
x = .commandhandler~deregister('spec_ch_other')
::class count inherit CommandHandler
::method command
  use arg command, caller
  say 'got' arg() '<'command'>' 'for' caller~environment caller~isValid
  parse var command 'nest' n .
  if n \== '' & n > 0 then do
    address spec_ch_other 'nest' n - 1
    say 'back in nest' n 'rc' rc
  end
  return arg()
::class text
::method makestring
  return 'from makestring'
```

```output
got 2 <first command> for SPEC_CH_DISPATCH 1
got 2 <second 3> for SPEC_CH_OTHER 1
got 2 <> for SPEC_CH_OTHER 1
got 2 <from makestring> for SPEC_CH_DISPATCH 1
got 2 <a clause> for SPEC_CH_DISPATCH 1
rc 2 SPEC_CH_DISPATCH
got 2 <nest 2> for SPEC_CH_DISPATCH 1
got 2 <nest 1> for SPEC_CH_OTHER 1
got 2 <nest 0> for SPEC_CH_OTHER 1
back in nest 1 rc 2
back in nest 2 rc 2
```

r[class.commandhandler.rc]
What `command` answers is the return code of the command: the object itself is
the value of `RC` after the clause, a string, a number, any object or `.nil`,
and a method that answers nothing leaves `RC` at `0`. Whatever object `command`
answers, a negative number included, the clause raises no condition for it, so a
return code other than `0` from a handler is neither ERROR nor FAILURE, and
`.RS` holds `0` after the command ([the status rule](r:instr.command.status));
the answer is no more than a value. Only the methods of the context
([`raise`](r:class.commandcontext.raise) and the rest) make a command raise
one, and so does a native environment that the handler forwards the command to
([the forwarding rule](r:class.nativecommandhandler.command)). How `RC` is
assigned otherwise is stated in [the return code rule](r:instr.command.rc).

```rexx
x = .commandhandler~register('spec_ch_rc', .answers~new)
signal on error name caught
signal on failure name caught
do word over 'ZERO ONE NEGATIVE TEXT NOTHING NIL ARRAY BIG'~makearray(' ')
  address spec_ch_rc word
  say word 'rc' '<'rc'>' rc~class~id .rs
end
x = .commandhandler~deregister('spec_ch_rc')
exit
caught:
  say 'trapped' condition('c') rc
::class answers inherit CommandHandler
::method command
  use arg command, caller
  select
    when command = 'ZERO' then return 0
    when command = 'ONE' then return 1
    when command = 'NEGATIVE' then return -1
    when command = 'TEXT' then return 'some text'
    when command = 'NOTHING' then return
    when command = 'NIL' then return .nil
    when command = 'ARRAY' then return .array~of(1)
    when command = 'BIG' then return 123456789012
  end
```

```output
ZERO rc <0> String 0
ONE rc <1> String 0
NEGATIVE rc <-1> String 0
TEXT rc <some text> String 0
NOTHING rc <0> String 0
NIL rc <The NIL object> Object 0
ARRAY rc <1> Array 0
BIG rc <123456789012> String 0
```

r[class.commandhandler.conditions]
A condition that is raised in `command` and that the method does not trap
reaches the code that sent the command, as a condition raised in any called
code does: a `SYNTAX` error, a HALT, or any other condition raised by a
`RAISE` in the method, which the code that sent the command traps as it would
a condition raised at its own clause. An error in the method is therefore the
error of the clause that sent the command, with its error number. The
conditions that the command itself raises, by what the handler asks the
context to raise, are stated with [the context](r:class.commandcontext.raise).

```rexx
x = .commandhandler~register('spec_ch_conditions', .raiser~new)
call attempt 'syntax'
call attempt 'halt'
call attempt 'error'
call attempt 'user'
x = .commandhandler~deregister('spec_ch_conditions')
exit
attempt: procedure
  signal on syntax name s
  call on halt name c
  call on error name c
  call on user mine name c
  address spec_ch_conditions arg(1)
  say arg(1) 'went through, rc' rc
  return
s: say 'syntax' condition('o')~code
  return
c: say condition('c') condition('d') rc
  return
::class raiser inherit CommandHandler
::method command
  use arg command, caller
  select
    when command = 'syntax' then x = .nosuchclass~new
    when command = 'halt' then raise halt description 'stop it'
    when command = 'error' then raise error 3 description 'oops'
    when command = 'user' then raise user mine description 'mine'
  end
  return 0
```

```output
syntax 97.1
HALT stop it 0
halt went through, rc 0
ERROR oops 3
error went through, rc 3
USER MINE mine 0
user went through, rc 0
```

## Class methods

r[class.commandhandler.register]
`register(name, handler)` makes `handler` the handler in use for the
environment `name` and answers the handler that it covers: the one that was in
use for the name before, or `.nil` when there was none. Both arguments are
required (`SYNTAX 93.901`) and a third raises `SYNTAX 93.902`. The name is
read as [the name rule](r:class.commandhandler.name) says, and is kept in
upper case, so a handler is found by the name in any case; a name with a blank
is allowed, and the null string and an object that is not a string raise
`SYNTAX 93.900`. `handler` must be an object of a class that inherits
`CommandHandler`: the object is asked `isA` with `.CommandHandler` as the
argument, once, after the name was read, and it is accepted when the answer is
true; `.nil`, a class and an object of another class raise `SYNTAX 93.900`.
The handler it covers is not lost: it comes back into use when this one is
removed by [`deregister`](r:class.commandhandler.deregister), and `handlers`
lists it. The same object can be registered more than once, under several
names and under one name again. A handler is the object registered, not a
copy. The registration belongs to the interpreter instance, and a name that is
already the name of a native environment is covered like any other.

```rexx
a = .echo~new('a'); b = .echo~new('b')
say .commandhandler~register('spec_ch_register', a)~isNil
say .commandhandler~register('Spec_Ch_Register', b)~tag
say .commandhandler~handler('SPEC_CH_REGISTER')~tag
say .commandhandler~register('spec_ch_register', a)~tag
say .commandhandler~handlers('spec_ch_register')~items
say .commandhandler~register('a name', a)~isNil,
  .commandhandler~register(12, a)~isNil
x = .commandhandler~deregister('a name')
x = .commandhandler~deregister('12')
do while .commandhandler~handlers('spec_ch_register')~items > 0
  x = .commandhandler~deregister('spec_ch_register')
end
call try ".commandhandler~register('spec_ch_register')"
call try ".commandhandler~register"
call try ".commandhandler~register('spec_ch_register', a, 1)"
call try ".commandhandler~register('', a)"
call try ".commandhandler~register(.nil, a)"
call try ".commandhandler~register(.object~new, a)"
call try ".commandhandler~register('spec_ch_register', .nil)"
call try ".commandhandler~register('spec_ch_register', .object~new)"
call try ".commandhandler~register('spec_ch_register', .echo)"
say .commandhandler~handlers('spec_ch_register')~items
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
::class echo inherit CommandHandler
::method init
  expose tag
  use arg tag
::attribute tag get
::method command
  return 0
```

```output
1
a
b
b
3
1 1
.commandhandler~register('spec_ch_register') 93.901
.commandhandler~register 93.901
.commandhandler~register('spec_ch_register', a, 1) 93.902
.commandhandler~register('', a) 93.900
.commandhandler~register(.nil, a) 93.900
.commandhandler~register(.object~new, a) 93.900
.commandhandler~register('spec_ch_register', .nil) 93.900
.commandhandler~register('spec_ch_register', .object~new) 93.900
.commandhandler~register('spec_ch_register', .echo) 93.900
0
```

r[class.commandhandler.deregister]
`deregister(name)` removes the handler in use for the environment `name`, and
answers it. The handler that it covered is in use again; when none is left, a
command that names the environment finds nothing to run it and raises FAILURE
with the return code `30`, as [the status rule](r:instr.command.status) shows
for a name that has no environment. A name that is not the name of an
environment, one that was already removed included, raises `SYNTAX 93.900`. The
name is required (`SYNTAX 93.901`) and a second argument raises `SYNTAX 93.902`;
it is read as [the name rule](r:class.commandhandler.name) says, and its case
plays no part. `deregister` always takes the handler in use; there is no way to
remove one that is covered by another. Registering the same handler again is
allowed. The null string raises `SYNTAX 93.900` as it does for `register`,
although `handler('')` answers an environment
([`environments`](r:class.commandhandler.environments)).

```rexx
a = .echo~new('a'); b = .echo~new('b')
x = .commandhandler~register('spec_ch_dereg', a)
x = .commandhandler~register('spec_ch_dereg', b)
say .commandhandler~deregister('SPEC_CH_DEREG')~tag,
  .commandhandler~handler('spec_ch_dereg')~tag
address spec_ch_dereg 'still there'
say 'rc' rc
say .commandhandler~deregister('spec_ch_dereg')~tag
say .commandhandler~handlers('spec_ch_dereg')~items,
  .commandhandler~handler('spec_ch_dereg')~isNil
signal on failure name gone
address spec_ch_dereg 'unknown now'
gone: say condition('c') rc
signal off failure
call try ".commandhandler~deregister('spec_ch_dereg')"
call try ".commandhandler~deregister"
call try ".commandhandler~deregister('x', 1)"
call try ".commandhandler~deregister(.nil)"
call try ".commandhandler~deregister('')"
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
::class echo inherit CommandHandler
::method init
  expose tag
  use arg tag
::attribute tag get
::method command
  say 'run by' self~tag
  return 0
```

```output
b a
run by a
rc 0
a
0 1
FAILURE 30
.commandhandler~deregister('spec_ch_dereg') 93.900
.commandhandler~deregister 93.901
.commandhandler~deregister('x', 1) 93.902
.commandhandler~deregister(.nil) 93.900
.commandhandler~deregister('') 93.900
```

r[class.commandhandler.handler]
`handler(name)` answers the handler in use for the environment `name`, the one
that a command sent to it would reach, and `.nil` when there is no environment
of that name. The case of the name plays no part. The name is required
(`SYNTAX 88.901`) and a second argument raises `SYNTAX 88.922`. The answer is
the object that was registered, so `==` finds the handler again, and a native
environment always answers the same
[`NativeCommandHandler`](nativecommandhandler.md) object. The name is read as
[the name rule](r:class.commandhandler.name) says.

```rexx
h = .echo~new
say .commandhandler~handler('spec_ch_handler')~isNil
x = .commandhandler~register('spec_ch_handler', h)
say (.commandhandler~handler('SPEC_CH_handler') == h)
say (.commandhandler~handler('system') == .commandhandler~handler('SYSTEM'))
x = .commandhandler~deregister('spec_ch_handler')
say .commandhandler~handler('spec_ch_handler')~isNil
call try ".commandhandler~handler"
call try ".commandhandler~handler('x', 1)"
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
::class echo inherit CommandHandler
::method command
  return 0
```

```output
1
1
1
1
.commandhandler~handler 88.901
.commandhandler~handler('x', 1) 88.922
```

r[class.commandhandler.handlers]
`handlers(name)` answers a new `Array` of the handlers that are registered for
the environment `name`, the one in use at index 1 and then the ones it covers,
each covered one after the one that covers it, down to the oldest. The array
is empty when there is no environment of that name. It is a copy: changing it
changes nothing, and a later `register` or `deregister` does not change an
array that was answered before. The name is required (`SYNTAX 88.901`) and a
second argument raises `SYNTAX 88.922`; it is read as [the name
rule](r:class.commandhandler.name) says.

```rexx
a = .echo~new('a'); b = .echo~new('b'); c = .echo~new('c')
say .commandhandler~handlers('spec_ch_handlers')~items
x = .commandhandler~register('spec_ch_handlers', a)
x = .commandhandler~register('spec_ch_handlers', b)
x = .commandhandler~register('spec_ch_handlers', c)
l = .commandhandler~handlers('SPEC_CH_HANDLERS')
say l~class~id l~items l[1]~tag l[2]~tag l[3]~tag
l~append('extra')
x = .commandhandler~deregister('spec_ch_handlers')
say l~items .commandhandler~handlers('spec_ch_handlers')~items
say (l[1] == c) (.commandhandler~handlers('spec_ch_handlers')[1] == b)
x = .commandhandler~deregister('spec_ch_handlers')
x = .commandhandler~deregister('spec_ch_handlers')
call try ".commandhandler~handlers"
call try ".commandhandler~handlers('x', 1)"
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
::class echo inherit CommandHandler
::method init
  expose tag
  use arg tag
::attribute tag get
::method command
  return 0
```

```output
0
Array 3 c b a
4 2
1 1
.commandhandler~handlers 88.901
.commandhandler~handlers('x', 1) 88.922
```

r[class.commandhandler.environments]
`environments` answers a new `StringTable` of the environments of the
interpreter instance. Its indexes are the names of the environments in upper
case, and the item of each is the handler in use for it, the object that
`handler` answers: an object of a class that inherits `CommandHandler` for an
environment that a program registered, and a
[`NativeCommandHandler`](nativecommandhandler.md) for a native one. The
environment `SYSTEM`, which runs commands in the host's shell, is among them.
Which other native environments exist depends on the host: on Linux they
are the ones that [the names rule](r:env.commands.names) lists. They include
one under the null string, which `register` and `deregister` refuse. The table is a snapshot: a later `register` or
`deregister` does not change it, and changing it does not change the
environments. The method takes no argument (`SYNTAX 88.922`).

```rexx
before = .commandhandler~environments
say before~class~id before~hasIndex('SYSTEM') before~hasIndex('system'),
  before~hasIndex('SPEC_CH_ENVIRONMENTS')
say before~hasIndex('') before['']~kind
h = .echo~new
x = .commandhandler~register('spec_ch_environments', h)
now = .commandhandler~environments
say now~hasIndex('SPEC_CH_ENVIRONMENTS'),
  before~hasIndex('SPEC_CH_ENVIRONMENTS'),
  (now['SPEC_CH_ENVIRONMENTS'] == h)
say now['SYSTEM']~class~id
now['SPEC_CH_ENVIRONMENTS'] = .nil
now['SPEC_CH_NEW'] = h
say .commandhandler~handler('spec_ch_environments')~class~id,
  .commandhandler~handler('spec_ch_new')~isNil
x = .commandhandler~deregister('spec_ch_environments')
say .commandhandler~environments~hasIndex('SPEC_CH_ENVIRONMENTS'),
  (now~items > 0)
signal on syntax name bad
x = .commandhandler~environments(1)
bad: say condition('o')~code
::class echo inherit CommandHandler
::method command
  return 0
```

```output
StringTable 1 0 0
1 REDIRECTING
1 0 1
NativeCommandHandler
ECHO 1
0 1
88.922
```

## Reading names and what is sent

r[class.commandhandler.name]
The `name` that `register`, `deregister`, `handler` and `handlers` take is read
as a string, and the environment is the one with that name in upper case. A
string is the name as it is. An object that is not a string, an instance of a
subclass of `String` apart ([the deviation
rule](r:class.commandhandler.deviation.109)), is replaced by what it answers to
the message `request`: `register` and `deregister` send it with the argument
`String`, and `handler` and `handlers` send it with the argument `STRING`, and
the answer, when it is a string, is the name; the `request` of `Object` answers
what `makeString` does, so an object that has `makeString`, an `Array` or a
`MutableBuffer` is read through it. A number is read as the string that it is.
An object that does not answer a string is no name: `register` and `deregister`
raise `SYNTAX 93.900` for it, and `handler` and `handlers` raise
`SYNTAX 88.909`.

```rexx
x = .commandhandler~register('spec_ch_name', .echo~new)
n = .named~new
.environment~log = ''
say .commandhandler~handler(n)~class~id '|' .environment~log~strip
.environment~log = ''
say .commandhandler~handlers(n)~items '|' .environment~log~strip
.environment~log = ''
say .commandhandler~register(n, .echo~new)~class~id '|' .environment~log~strip
.environment~log = ''
say .commandhandler~deregister(n)~class~id '|' .environment~log~strip
say .commandhandler~handler(.array~of('spec_ch_name'))~class~id
x = .commandhandler~deregister('spec_ch_name')
call try ".commandhandler~handler(.nil)"
call try ".commandhandler~handlers(.object~new)"
call try ".commandhandler~register(.nil, .echo~new)"
call try ".commandhandler~deregister(.object~new)"
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
::class echo inherit CommandHandler
::method command
  return 0
::class named
::method request
  .environment~log = .environment~log 'request' arg(1)
  return 'SPEC_CH_NAME'
```

```output defect=#164
ECHO | request STRING
1 | request STRING
ECHO | request String
ECHO | request String
ECHO
.commandhandler~handler(.nil) 88.909
.commandhandler~handlers(.object~new) 88.909
.commandhandler~register(.nil, .echo~new) 93.900
.commandhandler~deregister(.object~new) 93.900
```

r[class.commandhandler.register.sends]
`register` sends the message `request` with the argument `String` to a name
that is not a string, and then `isA` with the class `CommandHandler` to the
handler, each once and in that order; it sends nothing else, and a handler
that does not answer true to `isA` is refused, whatever its class is. The
handler does not get `command` or any other message when it is registered. In
the example `isA` of a handler is overridden to say what it was asked.

```rexx
.environment~log = ''
n = .named~new
h = .picky~new
x = .commandhandler~register(n, h)
say .environment~log
x = .commandhandler~deregister('SPEC_CH_SENDS')
.environment~log = ''
signal on syntax name refused
x = .commandhandler~register('spec_ch_sends', .picky~new(0))
refused: say 'refused' condition('o')~code .environment~log
::class named
::method request
  .environment~log = .environment~log 'request' arg(1)
  return 'SPEC_CH_SENDS'
::class picky inherit CommandHandler
::method init
  expose answer
  use arg answer = 1
::method isA
  expose answer
  .environment~log = .environment~log 'isA' arg(1)~id
  return answer
::method command
  .environment~log = .environment~log 'command'
  return 0
```

```output
 request String isA CommandHandler
refused 93.900  isA CommandHandler
```

## Deviations

r[class.commandhandler.deviation.109]
A name that `register`, `deregister`, `handler` or `handlers` takes and that is
an instance of a subclass of `String` is read by its own characters, and
`request`, `makeString` and `string` are not sent to it ([Deviation
109](r:class.string.deviation.109)). ooRexx sends such an instance `request`,
with `String` from `register` and `deregister` and with `STRING` from `handler`
and `handlers`, so an override can change the name that is read. In the examples
the overrides would change the name; they count their sends, and the string is
read as it is without one.

```rexx
.environment~sent = 0
h = .echo~new
x = .commandhandler~register('spec_ch_dev109', h)
d = .digits~new('spec_ch_dev109')
say .commandhandler~handler(d)~class~id .environment~sent
say .commandhandler~handlers(d)~items .environment~sent
x = .commandhandler~deregister('spec_ch_dev109')
::class echo inherit CommandHandler
::method command
  return 0
::class digits subclass string
::method makestring
  .environment~sent = .environment~sent + 1
  return 'other'
::method request
  .environment~sent = .environment~sent + 1
  return 'other'
```

```output
ECHO 0
1 0
```

```rexx
.environment~sent = 0
h = .echo~new
d = .digits~new('spec_ch_dev109_reg')
say .commandhandler~register(d, h)~isNil .environment~sent
say .commandhandler~handler('spec_ch_dev109_reg')~class~id,
  .commandhandler~handler('other')~isNil
say .commandhandler~deregister(d)~class~id .environment~sent
say .commandhandler~handler('spec_ch_dev109_reg')~isNil
::class echo inherit CommandHandler
::method command
  return 0
::class digits subclass string
::method makestring
  .environment~sent = .environment~sent + 1
  return 'other'
::method request
  .environment~sent = .environment~sent + 1
  return 'other'
```

```output defect=#140
1 0
ECHO 1
ECHO 0
1
```
