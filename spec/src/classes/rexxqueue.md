---
element: "RexxQueue"
kind: class
deviations: [109, 135]
rexxref: "Builtin Classes > Utility Classes > RexxQueue Class; Builtin Classes > Utility Classes > RexxQueue Class > new (Inherited Class Method); Builtin Classes > Utility Classes > RexxQueue Class > create (Class Method); Builtin Classes > Utility Classes > RexxQueue Class > delete (Class Method); Builtin Classes > Utility Classes > RexxQueue Class > exists (Class Method); Builtin Classes > Utility Classes > RexxQueue Class > open (Class Method); Builtin Classes > Utility Classes > RexxQueue Class > delete; Builtin Classes > Utility Classes > RexxQueue Class > empty; Builtin Classes > Utility Classes > RexxQueue Class > get; Builtin Classes > Utility Classes > RexxQueue Class > init; Builtin Classes > Utility Classes > RexxQueue Class > lineIn; Builtin Classes > Utility Classes > RexxQueue Class > lineOut; Builtin Classes > Utility Classes > RexxQueue Class > makeArray; Builtin Classes > Utility Classes > RexxQueue Class > pull; Builtin Classes > Utility Classes > RexxQueue Class > push; Builtin Classes > Utility Classes > RexxQueue Class > queue; Builtin Classes > Utility Classes > RexxQueue Class > queued; Builtin Classes > Utility Classes > RexxQueue Class > say; Builtin Classes > Utility Classes > RexxQueue Class > set"
---

# RexxQueue

A `RexxQueue` is an object that works with one of the external data queues,
the lists of lines that are kept outside the program in a service and that the
[QUEUE](r:instr.queue.fifo) and [PUSH](r:instr.push.lifo) instructions fill
and the [PULL](r:instr.pull.source) instruction empties. The object holds the
name of a queue, adds lines to the queue and takes them out, and asks how many
there are. Besides the instance methods the class has methods that make, open,
delete and ask after a queue by its name. They are the same queues as those of
[the RXQUEUE function](../builtins/rxqueue.md), and the rules that function
states for the names of queues, for the queue `SESSION` and for a queue that
does not exist hold here too. A program has a current queue that the
instructions use; the object has a queue of its own, and the two are the same
queue only when they have the same name. How programs and sessions share the
queues is specified by [the session rule](r:env.commands.rxqueue.session) and
[the child rule](r:env.commands.rxqueue.child).

> **Note.** The examples create a queue under a name that no other example
> uses and delete it when they are done, so they leave nothing behind. An
> example never waits for a line that nothing will add, because a program that
> does that does not end.

## The class

r[class.rexxqueue.creation]
`RexxQueue` has the class methods `create`, `delete`, `exists` and `open`, and
the instance methods `delete`, `empty`, `get`, `init`, `lineIn`, `lineOut`,
`makeArray`, `pull`, `push`, `queue`, `queued`, `say` and `set`, besides the
methods that every class has, `new` among them. A class method is not an
instance method and an instance method is not a class method, so the two
`delete` methods do not meet. The class may have private methods besides
these; they are not part of this specification. Its only superclass is
`Object` and it inherits no mixin class ([the hierarchy
rule](r:class.index.hierarchy)).

```rexx
say .rexxqueue~id .rexxqueue~superclasses~items,
  .rexxqueue~superclass~id .rexxqueue~queryMixinClass
have = .set~new
s = .rexxqueue~methods(.nil)
do while s~available
  have~put(s~index)
  s~next
end
names = ('delete empty get init lineIn lineOut makeArray pull push queue',
  'queued say set')~makeArray(' ')
count = 0
do name over names
  if have~hasItem(name~upper) then count = count + 1
end
say count 'of' names~items
say .rexxqueue~hasMethod('CREATE') .rexxqueue~hasMethod('DELETE'),
  .rexxqueue~hasMethod('EXISTS') .rexxqueue~hasMethod('OPEN')
q = .rexxqueue~new('spec_rq_creation')
say q~hasMethod('CREATE') q~hasMethod('OPEN') q~hasMethod('EXISTS'),
  q~hasMethod('NEW') q~hasMethod('DELETE')
say q~delete
```

```output
RexxQueue 1 Object 0
13 of 13
1 1 1 1
0 0 0 0 1
0
```

r[class.rexxqueue.string]
The string value of a `RexxQueue` is the name of its queue in upper case: the
class has no `string` method of its own, and the `string` of `Object` answers
the `objectName`, which [the `init` rule](r:class.rexxqueue.init) sets to the
queue name and [the `set` rule](r:class.rexxqueue.set) sets again to the name
of the queue it switches to. The default name of the object stays
`a RexxQueue`. A program may give the object another name with `objectName=`
([the name rule](r:class.index.objectname)); that changes the string value and
nothing else, for the name of the queue that the object works with is `get`'s
answer.

```rexx
q = .rexxqueue~new('spec_rq_Mixed_Case')
say q~string q~get q~defaultName
q~objectName = 'a nickname'
say q~string '|' q~get '|' q~objectName
say q~set('spec_rq_string_other') q~string q~get
say q~set('spec_rq_Mixed_Case') q~string
say q~delete
```

```output
SPEC_RQ_MIXED_CASE SPEC_RQ_MIXED_CASE a RexxQueue
a nickname | SPEC_RQ_MIXED_CASE | a nickname
SPEC_RQ_MIXED_CASE SPEC_RQ_STRING_OTHER SPEC_RQ_STRING_OTHER
SPEC_RQ_STRING_OTHER SPEC_RQ_MIXED_CASE
0
```

## Making a queue object

r[class.rexxqueue.new]
`new` is the `new` that `Class` supplies ([the new rule](r:class.index.new)): it
makes an object of the class that received the message, so a subclass makes
instances of itself, and sends it `init` with the arguments of `new`, in order.
The queue that the object works with is therefore the business of `init`, and
`new` with more than one argument raises `SYNTAX 93.902`, because `init` takes
at most one. A subclass whose `init` does not reach the `init` of `RexxQueue`,
by `forward` or by `self~init:super`, makes an object that is not connected to a
queue ([Deviation 135](r:class.rexxqueue.deviation.135)).

```rexx
q = .rexxqueue~new('spec_rq_new')
say q~class~id q~get
s = .tagged~new('spec_rq_new_sub', 'my tag')
say s~class~id s~get s~tag
say q~delete s~delete
signal on syntax name bad
x = .rexxqueue~new('spec_rq_new_x', 'y')
bad: say condition('o')~code
::class tagged subclass rexxqueue
::method init
  expose tag
  use arg name, tag
  self~init:super(name)
::method tag
  expose tag
  return tag
```

```output
RexxQueue SPEC_RQ_NEW
TAGGED SPEC_RQ_NEW_SUB my tag
0 0
93.902
```

r[class.rexxqueue.init]
`init(name)` connects the object to a queue and takes at most one argument
(`SYNTAX 93.902`). It decides the name in one of three ways. With no argument
the name is `SESSION`. With `.nil` it makes a queue whose name the service
picks, as the class method `create` does with no argument, and uses that name;
no program can know the name beforehand. With any other argument it sends the
argument the message `upper`, and the answer is the name: a string in upper
case, a number such as `12` or a `MutableBuffer` gives its upper-case string,
and an object that does not understand `upper`, an `Object` or an `Array` for
one, raises `SYNTAX 97.1`. For a name that was given, `init` then sends the
class of the receiver the message `open` with the name, as [the class
method](r:class.rexxqueue.open.class-side) says: a queue of that name is kept
as it is with its lines, or made when there is none. An `init` does not look
at the answer of `open`, so a name that no queue can have raises no error
here: the object is made, and the methods that need the queue fail later, as
[the missing queue rule](r:class.rexxqueue.missing) says. At last `init` sets
the `objectName` of the object to the name, as [the string
rule](r:class.rexxqueue.string) says. The messages that `init` sends are
stated in [the next rule](r:class.rexxqueue.init.sends).

```rexx
a = .rexxqueue~new
b = .rexxqueue~new(.nil)
say a~get '|' (b~get~length > 0) (b~get \== 'SESSION')
say .rexxqueue~exists(b~get) b~delete .rexxqueue~exists(b~get)
c = .rexxqueue~new('spec_rq_init_Kept')
c~queue('kept line')
d = .rexxqueue~new('SPEC_RQ_INIT_KEPT')
say d~get d~queued d~pull
say .rexxqueue~new(12)~get '|' .rexxqueue~new(.mutablebuffer~new('mb'))~get
say .rexxqueue~exists(12) .rexxqueue~exists('MB') .rexxqueue~delete(12),
  .rexxqueue~delete('mb')
say c~delete
e = .rexxqueue~new('not a valid name')
say e~get '|' .rexxqueue~exists(e~get)
call try .object~new
call try .array~new
exit
try:
  signal on syntax name bad
  x = .rexxqueue~new(arg(1))
  say 'no error'
  return
bad: say condition('o')~code
  return
```

```output
SESSION | 1 1
1 0 0
SPEC_RQ_INIT_KEPT 1 kept line
12 | MB
1 1 0 0
0
NOT A VALID NAME | 0
97.1
97.1
```

r[class.rexxqueue.init.sends]
`init` sends messages in this order. With a name given, `upper` goes to the
argument, once and before anything else. With `.nil`, `create` goes to the
class of the receiver with no argument, and then `open` with the name that
`create` answered. With a name given, `open` goes to the class of the
receiver, with the upper-case name. With no argument, nothing is sent to the
class. Then `objectName=` goes to the receiver itself with the name. A
subclass of `RexxQueue` therefore sees its own class-side `create` and `open`
and its own `objectName=`. In the example the subclass overrides them and
writes what they receive.

```rexx
.environment~made = ''
say '-- named'
q = .spy~new('spec_rq_sends_init')
say '-- nil'
r = .spy~new(.nil)
say '-- none'
s = .spy~new
say '-- upper'
t = .spy~new(.shout~new)
say q~delete r~delete t~delete
::class spy subclass rexxqueue
::method create class
  say 'create' arg()
  .environment~made = self~create:super
  return .environment~made
::method open class
  if arg(1) == .environment~made~upper then
    say 'open the name that create answered'
  else say 'open' arg(1)
  forward class (super)
::method 'objectName='
  if arg(1) == .environment~made~upper then
    say 'objectName= the name that create answered'
  else say 'objectName=' arg(1)
  forward class (super)
::class shout
::method upper
  say 'upper'
  return 'SPEC_RQ_SENDS_UPPER'
```

```output
-- named
open SPEC_RQ_SENDS_INIT
objectName= SPEC_RQ_SENDS_INIT
-- nil
create 0
open the name that create answered
objectName= the name that create answered
-- none
objectName= SESSION
-- upper
upper
open SPEC_RQ_SENDS_UPPER
objectName= SPEC_RQ_SENDS_UPPER
0 0 0
```

## Class methods

r[class.rexxqueue.create.class-side]
`create(name)` makes a queue and answers its name. It takes at most one argument
(`SYNTAX 88.922`). With a name that no queue has, the queue gets that name and
the answer is the name as it was given, in the case it was given in. When a
queue of that name exists, or with no argument, the queue is made under a name
that the service picks, which the answer reports, so the answer is a name of a
new queue: [the `Create` function](r:bif.rxqueue.create) states that. The name
`SESSION` and a name that no queue can have ([the queue name
rule](r:bif.rxqueue.name)), the null string, a name with a blank or a `-`, and a
name of more than 250 characters among them, make no queue and answer the null
string, without an error. The name is read as [the name
rule](r:class.rexxqueue.name) says.

```rexx
a = .rexxqueue~create('spec_rq_Create')
b = .rexxqueue~create('SPEC_RQ_CREATE')
c = .rexxqueue~create
say a (b \== 'SPEC_RQ_CREATE') (c \== a) (c \== b)
say .rexxqueue~exists(a) .rexxqueue~exists(b) .rexxqueue~exists(c)
say .rexxqueue~delete(a) .rexxqueue~delete(b) .rexxqueue~delete(c)
say '['.rexxqueue~create('session')']' '['.rexxqueue~create('a b')']',
  '['.rexxqueue~create('a-b')']' '['.rexxqueue~create('')']',
  '['.rexxqueue~create(copies('z', 251))']'
signal on syntax name bad
x = .rexxqueue~create('spec_rq_create_x', 'y')
bad: say condition('o')~code
```

```output
spec_rq_Create 1 1 1
1 1 1
0 0 0
[] [] [] [] []
88.922
```

r[class.rexxqueue.open.class-side]
`open(name)` makes sure that a queue of the name exists and answers `0`: it
keeps an existing queue with its lines, and makes the queue when there is none,
as [the `Open` function](r:bif.rxqueue.open) does. The queue `SESSION` answers
`0` too. A name that no queue can have ([the queue name
rule](r:bif.rxqueue.name)), as for `create`, makes nothing and answers `5`. The
name is required (`SYNTAX 88.901`) and the method takes no other argument
(`SYNTAX 88.922`).

```rexx
say .rexxqueue~exists('spec_rq_open')
say .rexxqueue~open('spec_rq_open') .rexxqueue~exists('spec_rq_open')
q = .rexxqueue~new('spec_rq_open')
q~queue('stays')
say .rexxqueue~open('SPEC_RQ_OPEN') q~queued
say .rexxqueue~open('session') .rexxqueue~open('a b') .rexxqueue~open('')
say q~delete
call try 'open'
call try 'open("x", "y")'
exit
try:
  signal on syntax name bad
  interpret 'x = .rexxqueue~' || arg(1)
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
```

```output
0
0 1
0 1
0 5 5
0
open 88.901
open("x", "y") 88.922
```

r[class.rexxqueue.exists.class-side]
`exists(name)` answers `1` when a queue of the name exists and `0` when it
does not. The case of the name plays no part, the queue `SESSION` always
exists, and a name that no queue can have answers `0` and raises no error. The
name is required (`SYNTAX 88.901`) and the method takes no other argument
(`SYNTAX 88.922`).

```rexx
q = .rexxqueue~new('spec_rq_Exists')
say .rexxqueue~exists('spec_rq_exists') .rexxqueue~exists('SPEC_RQ_EXISTS'),
  .rexxqueue~exists('session')
say .rexxqueue~exists('spec_rq_exists_none') .rexxqueue~exists('a b'),
  .rexxqueue~exists('')
say q~delete .rexxqueue~exists('spec_rq_exists')
call try 'exists'
call try 'exists("x", "y")'
exit
try:
  signal on syntax name bad
  interpret 'x = .rexxqueue~' || arg(1)
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
```

```output
1 1 1
0 0 0
0 0
exists 88.901
exists("x", "y") 88.922
```

r[class.rexxqueue.delete.class-side]
`delete(name)` removes the queue of the name with its lines, as [the `Delete`
function](r:bif.rxqueue.delete) does, and answers a number. The answer is `0`
when the queue was deleted and `9` when no queue has the name. The queue
`SESSION` is not deleted, and the answer is `5`; `5` is also the answer for a
name that no queue can have. The name is required (`SYNTAX 88.901`) and the
method takes no other argument (`SYNTAX 88.922`). An object that works with
the queue goes on holding its name, and [the missing queue
rule](r:class.rexxqueue.missing) says what its methods do.

```rexx
q = .rexxqueue~new('spec_rq_delete_cls')
q~queue('lost')
say .rexxqueue~delete('spec_rq_delete_CLS'),
  .rexxqueue~exists('spec_rq_delete_cls')
say .rexxqueue~delete('spec_rq_delete_cls') .rexxqueue~delete('spec_rq_none')
say .rexxqueue~delete('session') .rexxqueue~delete('a b'),
  .rexxqueue~exists('session')
say q~get q~queued
call try 'delete'
call try 'delete("x", "y")'
exit
try:
  signal on syntax name bad
  interpret 'x = .rexxqueue~' || arg(1)
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
```

```output
0 0
9 9
5 5 1
SPEC_RQ_DELETE_CLS 0
delete 88.901
delete("x", "y") 88.922
```

r[class.rexxqueue.name]
The `name` that `create`, `open`, `exists` and `delete` take, and the `line`
that `queue` and `push` take, are read in the same way, as a string. It is a
string, an instance of a subclass of `String` included, or an object that
answers a string to the message `request` with the argument `STRING`, which an
object with a `makeString` method does by the rule of `Object` ([the
conversion rule](r:conv.string.object)); the answer is used as the name or the
line. Any other object, `.nil` and an `Object` among them, raises
`SYNTAX 88.909`, and the object is read before the number of arguments is
checked, so a call with two arguments whose first is `.nil` raises `88.909`
and not `88.922`. A number is read as the string that it is, and an `Array` is
read as the string that its `makeString` gives: its items, one to a line.

```rexx
q = .rexxqueue~new('spec_rq_name')
n = .named~new('SPEC_RQ_NAME')
say .rexxqueue~exists(n) .rexxqueue~exists(12),
  .rexxqueue~exists(.array~of('spec_rq_name'))
q~queue(.named~new('from request'))
q~queue(.array~of('a', 'b'))
say '<'q~pull'>' '<'q~pull~changestr('0a'x, '|')'>'
say .rexxqueue~open(n) .rexxqueue~delete(n) .rexxqueue~exists(n)
call try '.rexxqueue~exists(.nil)'
call try '.rexxqueue~open(.object~new)'
call try '.rexxqueue~delete(.nil, 2)'
call try '.rexxqueue~create(.nil)'
q = .rexxqueue~new('spec_rq_name')
call try 'q~queue(.nil)'
call try 'q~push(.object~new)'
call try 'q~queue(.nil, 2)'
say q~delete
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
::class named
::method init
  expose name
  use arg name
::method request
  expose name
  return name
```

```output defect=#164
1 0 1
<from request> <a|b>
0 0 0
.rexxqueue~exists(.nil) 88.909
.rexxqueue~open(.object~new) 88.909
.rexxqueue~delete(.nil, 2) 88.909
.rexxqueue~create(.nil) 88.909
q~queue(.nil) 88.909
q~push(.object~new) 88.909
q~queue(.nil, 2) 88.909
0
```

## Adding lines

r[class.rexxqueue.queue]
`queue(line)` adds one line at the end of the queue, so that it is taken after
the lines that are there, and answers `0`. `line` is the line, read as a
string by [the name rule](r:class.rexxqueue.name); without it, or with an
omitted place, the line is the null string. The method takes no other argument
(`SYNTAX 88.922`). A line is kept as it is, with its case, its blanks and any
characters it holds, a line end among them: one call adds one line, and a line
end inside it does not split it. A queue that does not exist, the queue of a
name that no queue can have included, cannot take the line: `SYNTAX 48.1` is
raised and the queue stays as it was. [The `QUEUE`
instruction](r:instr.queue.fifo) adds a line to the queue of the program in
the same order.

```rexx
q = .rexxqueue~new('spec_rq_queue')
say q~queue('first') q~queue('Second  line') q~queue q~queue(,)
say q~queue(12.50) q~queue(.mutablebuffer~new('mb')) q~queue('a'||'0a'x||'b')
say q~queued
do while q~queued > 0
  say '<'q~pull~changestr('0a'x, '\n')'>'
end
signal on syntax name bad
x = q~queue('a', 'b')
bad: say condition('o')~code
q~delete
signal on syntax name gone
x = q~queue('lost')
gone: say condition('o')~code q~queued
```

```output
0 0 0 0
0 0 0
7
<first>
<Second  line>
<>
<>
<12.50>
<mb>
<a\nb>
88.922
48.1 0
```

r[class.rexxqueue.push]
`push(line)` adds one line at the front of the queue, so that it is the next
line taken, and answers `0`. The line, its default, its errors and the
behaviour for a missing queue are those of [`queue`](r:class.rexxqueue.queue).
[The `PUSH` instruction](r:instr.push.lifo) does the same for the queue of the
program.

```rexx
q = .rexxqueue~new('spec_rq_push')
say q~queue('b') q~push('a') q~push q~queue('c')
do while q~queued > 0
  say '<'q~pull'>'
end
signal on syntax name bad
x = q~push('a', 'b')
bad: say condition('o')~code
q~delete
signal on syntax name gone
x = q~push('lost')
gone: say condition('o')~code
```

```output
0 0 0 0
<>
<a>
<b>
<c>
88.922
48.1
```

r[class.rexxqueue.lineout]
`lineOut(line)` adds a line at the end of the queue exactly as
[`queue`](r:class.rexxqueue.queue) does and answers what `queue` answers: it
sends `queue` to the receiver with the argument it was given, the way [the
`lineOut.sends` rule](r:class.rexxqueue.lineout.sends) states. A `lineOut`
with no argument adds the null string.

```rexx
q = .rexxqueue~new('spec_rq_lineout')
say q~lineOut('one') q~lineOut q~lineOut('') q~lineOut(3)
say q~queued '<'q~pull'><'q~pull'><'q~pull'><'q~pull'>'
signal on syntax name bad
x = q~lineOut('a', 'b')
bad: say condition('o')~code
q~delete
```

```output
0 0 0 0
4 <one><><><3>
88.922
```

r[class.rexxqueue.say]
`say(line)` adds a line at the end of the queue exactly as
[`queue`](r:class.rexxqueue.queue) does and answers what `queue` answers: it
sends `queue` to the receiver with the argument it was given, as [the
`say.sends` rule](r:class.rexxqueue.say.sends) states. It writes nothing to
the output of the program; the name is that of the method that [the SAY
instruction](r:instr.say.write) uses to write to an output stream.

```rexx
q = .rexxqueue~new('spec_rq_say')
say q~say('x y') q~say
say q~queued '<'q~pull'><'q~pull'>'
signal on syntax name bad
x = q~say('a', 'b')
bad: say condition('o')~code
q~delete
```

```output
0 0
2 <x y><>
88.922
```

r[class.rexxqueue.lineout.sends]
`lineOut` sends the receiver the message `queue` with its own arguments, as
many as it got: none when it got none, and the argument unchanged, not read as
a string, otherwise. It sends nothing else, and what `lineOut` answers is what
that `queue` answers, so an override of `queue` that answers another value
changes the answer of `lineOut`, and one that stores nothing leaves the queue
as it was. The example overrides `queue`.

```rexx
q = .spy~new('spec_rq_lineout_sends')
say q~lineOut
say q~lineOut('text')
say q~lineOut(.mutablebuffer~new('buf'))
say q~lineOut(,)
say q~queued
say q~delete
::class spy subclass rexxqueue
::method queue
  say 'queue' arg() arg(1)~class~id
  return 'answered by queue'
```

```output
queue 0 String
answered by queue
queue 1 String
answered by queue
queue 1 MutableBuffer
answered by queue
queue 0 String
answered by queue
0
0
```

r[class.rexxqueue.say.sends]
`say` sends the receiver the message `queue` with its own arguments, exactly
as [`lineOut` does](r:class.rexxqueue.lineout.sends), sends nothing else, and
answers what that `queue` answers.

```rexx
q = .spy~new('spec_rq_say_sends')
say q~say
say q~say(12.50)
say q~say(.mutablebuffer~new('buf'))
say q~queued
say q~delete
::class spy subclass rexxqueue
::method queue
  say 'queue' arg() arg(1)~class~id
  return 'answered by queue'
```

```output
queue 0 String
answered by queue
queue 1 String
answered by queue
queue 1 MutableBuffer
answered by queue
0
0
```

## Taking lines

r[class.rexxqueue.pull]
`pull` takes the first line of the queue away and answers it as a string, or
answers `.nil` when the queue holds no line, at once and without waiting. The
line is as it was added, in the same case: nothing is changed to upper case as
the [`PULL` instruction](r:instr.pull.source) does. The method takes no
argument (`SYNTAX 88.922`). A queue that does not exist has no lines, so it
answers `.nil`.

```rexx
q = .rexxqueue~new('spec_rq_pull')
say q~pull~isNil
q~queue('Mixed Case')
q~queue('')
say '<'q~pull'>' '<'q~pull'>' q~pull~isNil q~queued
q~delete
say q~pull~isNil
signal on syntax name bad
x = q~pull(1)
bad: say condition('o')~code
```

```output
1
<Mixed Case> <> 1 0
1
88.922
```

r[class.rexxqueue.linein]
`lineIn` takes the first line of the queue away and answers it, as
[`pull`](r:class.rexxqueue.pull) does, when the queue holds a line. When the
queue exists and holds no line, `lineIn` does not answer at once: it waits
until some activity adds a line, and answers that. Which of several activities
that wait gets a line is specified by [the waiting
rule](r:conc.activities.waiting) of the concurrency chapter. A queue that
does not exist makes `lineIn` answer `.nil` at once, as `pull` does. The
method takes no argument (`SYNTAX 88.922`). In the example a message that the
program starts waits on an empty queue, and the program adds the line from the
other activity; whichever of the two comes first, the line goes to the waiting
call.

```rexx
q = .rexxqueue~new('spec_rq_linein')
q~queue('ready')
say q~lineIn q~queued
m = .message~new(q, 'lineIn')
m~start
say q~queue('late')
say m~result
say q~queued
q2 = .rexxqueue~new('spec_rq_linein_other')
w = .message~new(q2, 'lineIn')~~start
say q2~push('pushed')
say w~result
say q~delete q2~delete
say q~lineIn~isNil
signal on syntax name bad
x = q~lineIn(1)
bad: say condition('o')~code
```

```output
ready 0
0
late
0
0
pushed
0 0
1
88.922
```

r[class.rexxqueue.queued]
`queued` answers the number of lines in the queue, and `0` for a queue that
does not exist. It takes no argument (`SYNTAX 88.922`). [The `QUEUED`
function](r:bif.queued.count) counts the lines of the queue of the program in
the same way.

```rexx
q = .rexxqueue~new('spec_rq_queued')
say q~queued
q~queue('a'); q~push('b'); q~queue('')
say q~queued
x = q~pull
say q~queued
say q~delete q~queued
signal on syntax name bad
x = q~queued(1)
bad: say condition('o')~code
```

```output
0
3
2
0 0
88.922
```

r[class.rexxqueue.makearray]
`makeArray` takes all the lines out of the queue and answers them as a new
`Array` of strings, the first line of the queue at index 1, in the order in
which `pull` would have answered them; the queue is empty afterwards. A queue
with no lines, or one that does not exist, gives an empty array. The method
takes no argument (`SYNTAX 93.902`), and the messages it sends are stated in
[the next rule](r:class.rexxqueue.makearray.sends).

```rexx
q = .rexxqueue~new('spec_rq_makearray')
say q~makearray~items
q~queue('b'); q~queue(''); q~push('a'); q~queue('c')
a = q~makearray
say a~class~id a~items q~queued
do i = 1 to a~items
  say i '<'a[i]'>'
end
say (q~makearray == q~makearray)
q~delete
say q~makearray~items
signal on syntax name bad
x = q~makearray(1)
bad: say condition('o')~code
```

```output
0
Array 4 0
1 <a>
2 <b>
3 <>
4 <c>
0
0
93.902
```

r[class.rexxqueue.makearray.sends]
`makeArray` sends the receiver `queued`, once, and then sends it `pull`, with
no argument, as often as `queued` answered, but it stops after the first
`pull` that answers `.nil`, and that answer is not put in the array. It sends
nothing else. An override of `queued` that answers a number larger than the
number of lines gives all the lines, and one that answers a smaller number
leaves the lines that it does not count in the queue; an override of `pull`
that answers `.nil` ends the array there. The example logs the messages.

```rexx
.environment~log = ''
.environment~nilpull = 0
q = .spy~new('spec_rq_makearray_sends')
p = .rexxqueue~new('spec_rq_makearray_sends')
q~queue('a'); q~queue('b'); q~queue('c')
.environment~count = 2
a = q~makearray
say a~items p~queued 'sent' .environment~log
.environment~log = ''
.environment~count = 9
a = q~makearray
say a~items p~queued 'sent' .environment~log
p~queue('x'); p~queue('y')
.environment~log = ''
.environment~count = 5
.environment~nilpull = 1
a = q~makearray
say a~items p~queued 'sent' .environment~log
.environment~nilpull = 0
say q~delete
::class spy subclass rexxqueue
::method queued
  .environment~log = .environment~log 'queued'
  return .environment~count
::method pull
  .environment~log = .environment~log 'pull'
  if .environment~nilpull then return .nil
  forward class (super)
```

```output
2 1 sent  queued pull pull
1 0 sent  queued pull pull
0 2 sent  queued pull
0
```

r[class.rexxqueue.empty]
`empty` removes every line from the queue and answers `0`; the queue stays,
with no line. The answer is `9` for a queue that does not exist and `5` for a
name that no queue can have. The method takes no argument (`SYNTAX 88.922`).

```rexx
q = .rexxqueue~new('spec_rq_empty')
say q~empty q~queued
q~queue('a'); q~queue('b')
say q~empty q~queued .rexxqueue~exists('spec_rq_empty')
say q~delete q~empty
signal on syntax name bad
x = q~empty(1)
bad: say condition('o')~code
```

```output
0 0
0 0 1
0 9
88.922
```

## The name and the switch

r[class.rexxqueue.get]
`get` answers the name of the queue that the object works with, in upper case.
It is the name that `init` or the last `set` fixed, whether or not a queue by
that name exists; deleting the queue does not change it. The method takes no
argument (`SYNTAX 93.902`).

```rexx
q = .rexxqueue~new('spec_rq_Get')
say q~get
say .rexxqueue~delete('spec_rq_get') q~get
say .rexxqueue~new~get
signal on syntax name bad
x = q~get(1)
bad: say condition('o')~code
```

```output
SPEC_RQ_GET
0 SPEC_RQ_GET
SESSION
93.902
```

r[class.rexxqueue.set]
`set(name)` switches the object to the queue of that name and answers the name
of the queue it worked with before, in upper case. `name` is required
(`SYNTAX 93.901`) and the method takes no other argument (`SYNTAX 93.902`),
both checked before anything else. `set` does not look for the queue: a name
of a queue that does not exist is taken, and [the missing queue
rule](r:class.rexxqueue.missing) says what the object then does. The name is
decided as `init` does: the argument is sent `upper`, and an object without
that method raises `SYNTAX 97.1`, `.nil` and an `Object` among them. Then
`objectName=` is sent to the receiver with the upper-case name ([the sends
rule](r:class.rexxqueue.set.sends)). Only the object changes: the queue that
the program uses with its instructions is not switched, and another object
that holds the same name is not changed.

```rexx
q = .rexxqueue~new('spec_rq_set_a')
b = .rexxqueue~new('spec_rq_set_b')
q~queue('in a')
b~queue('in b')
say q~set('spec_rq_Set_B') q~get q~queued '<'q~pull'>'
say q~set('spec_rq_set_a') q~queued
say q~set('spec_rq_set_none') q~get q~queued
say q~set('spec_rq_set_a') b~get
say q~delete b~delete
call try 'set'
call try 'set("x", "y")'
call try 'set(.nil)'
call try 'set(.object~new)'
exit
try:
  signal on syntax name bad
  interpret 'x = q~' || arg(1)
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
```

```output
SPEC_RQ_SET_A SPEC_RQ_SET_B 1 <in b>
SPEC_RQ_SET_B 1
SPEC_RQ_SET_A SPEC_RQ_SET_NONE 0
SPEC_RQ_SET_NONE SPEC_RQ_SET_B
0 0
set 93.901
set("x", "y") 93.902
set(.nil) 97.1
set(.object~new) 97.1
```

r[class.rexxqueue.set.sends]
`set` sends the argument the message `upper`, once, and then sends the
receiver `objectName=` with the upper-case name, whether or not the name
changes, and answers the name that `get` answered before; it sends nothing
else. `get` is not sent. The example overrides `objectName=` and gives an
argument that has its own `upper`.

```rexx
q = .spy~new('spec_rq_set_sends')
say '-- string'
say q~set('spec_rq_set_sends_b')
say '-- object'
say q~set(.shout~new)
say '-- same name'
say q~set('SPEC_RQ_SET_SENDS_Z')
say .rexxqueue~delete('spec_rq_set_sends')
::class spy subclass rexxqueue
::method 'objectName='
  say 'objectName=' arg(1)
  forward class (super)
::class shout
::method upper
  say 'upper'
  return 'SPEC_RQ_SET_SENDS_Z'
```

```output
objectName= SPEC_RQ_SET_SENDS
-- string
objectName= SPEC_RQ_SET_SENDS_B
SPEC_RQ_SET_SENDS
-- object
upper
objectName= SPEC_RQ_SET_SENDS_Z
SPEC_RQ_SET_SENDS_B
-- same name
objectName= SPEC_RQ_SET_SENDS_Z
SPEC_RQ_SET_SENDS_Z
0
```

## Deleting

r[class.rexxqueue.delete]
`delete` deletes the queue that the object works with, with its lines, and
answers the number that the class method `delete` answers for its name: `0`
when the queue was deleted, `9` when it did not exist, and `5` for `SESSION`
and for a name that no queue can have. It takes no argument (`SYNTAX 93.902`),
and [the next rule](r:class.rexxqueue.delete.sends) states what it sends. The
object keeps its name, so that the queue is made again by any object that is
made for that name; until then [the missing queue
rule](r:class.rexxqueue.missing) holds for it.

```rexx
q = .rexxqueue~new('spec_rq_delete')
q~queue('gone')
say q~delete .rexxqueue~exists('spec_rq_delete') q~get
say q~delete
say .rexxqueue~new~delete .rexxqueue~exists('session')
r = .rexxqueue~new('spec_rq_delete')
say r~queued .rexxqueue~exists('spec_rq_delete')
say r~delete
signal on syntax name bad
x = r~delete(1)
bad: say condition('o')~code
```

```output
0 0 SPEC_RQ_DELETE
9
5 1
0 1
0
93.902
```

r[class.rexxqueue.delete.sends]
`delete` sends the class of the receiver the message `delete`, with the name
that `get` would answer, in upper case, and answers what that class method
answers; it sends nothing else, `get` not among them. A subclass sees its own
class-side `delete`.

```rexx
q = .spy~new('spec_rq_delete_sends')
q~objectName = 'another name'
say q~delete
say q~delete
::class spy subclass rexxqueue
::method delete class
  say 'delete' arg()  arg(1)
  forward class (super)
```

```output
delete 1 SPEC_RQ_DELETE_SENDS
0
delete 1 SPEC_RQ_DELETE_SENDS
9
```

## A queue that is not there

r[class.rexxqueue.missing]
The methods of an object whose queue does not exist, because it was deleted,
because the name was never made or because no queue can have the name, behave
as follows. `queue`, `push`, `lineOut` and `say` raise `SYNTAX 48.1`, and the
queue is not made. `pull` and `lineIn` answer `.nil` at once, `queued` answers
`0`, `makeArray` answers an empty array, and `empty` and `delete` answer `9`
for a queue that is gone and `5` for a name that no queue can have. `get` and
`set` work as usual. The queue exists again as soon as any object or class
method makes it, by `new` with the name, `open` or `create`.

```rexx
q = .rexxqueue~new('spec_rq_missing')
say q~delete
call show 'deleted'
q = .rexxqueue~new('spec_rq_not valid')
say q~get
call show 'invalid'
exit
show:
  say arg(1) q~pull~isNil q~lineIn~isNil q~queued q~makearray~items,
    q~empty q~delete
  do m over 'queue push lineout say'~makearray(' ')
    call try m
  end
  return
try:
  signal on syntax name e
  x = q~send(arg(1), 'text')
  say arg(1) 'no error'
  return
e: say arg(1) condition('o')~code q~queued
  return
```

```output
0
deleted 1 1 0 0 9 9
queue 48.1 0
push 48.1 0
lineout 48.1 0
say 48.1 0
SPEC_RQ_NOT VALID
invalid 1 1 0 0 5 5
queue 48.1 0
push 48.1 0
lineout 48.1 0
say 48.1 0
```

## Copying

r[class.rexxqueue.copy]
`copy` answers a new object of the same class that holds the same queue name
and has the same `objectName`; no message is sent to the receiver. The queue
itself is not copied, for the two objects work with the one queue, so a line
that one adds is a line the other takes. From then on the two objects are
separate: `set` or `objectName=` on one does not change the other.

```rexx
q = .rexxqueue~new('spec_rq_copy')
q~objectName = 'nick'
c = q~copy
say c~class~id c~get c~string (c \== q)
say q~queue('via q') c~queued '<'c~pull'>' q~queued
say c~set('spec_rq_copy_other') c~get q~get
c~objectName = 'changed'
say q~string c~string
say q~delete
```

```output
RexxQueue SPEC_RQ_COPY nick 1
0 1 <via q> 0
SPEC_RQ_COPY SPEC_RQ_COPY_OTHER SPEC_RQ_COPY
nick changed
0
```

## The queue of the program

r[class.rexxqueue.program]
An object and the instructions of the program meet when they use the same
name. The queue that [`PUSH`](r:instr.push.lifo),
[`QUEUE`](r:instr.queue.fifo) and [`PULL`](r:instr.pull.source) use, and that
[`QUEUED`](r:bif.queued.count) counts, is the current queue that
[`RXQUEUE('Set', name)`](r:bif.rxqueue.set) chooses, and an object whose queue
has that name sees the same lines in the same order. Switching one of them,
the current queue by `RXQUEUE` or the object by `set`, does not switch the
other.

```rexx
q = .rexxqueue~new('spec_rq_program')
before = rxqueue('Set', 'spec_rq_program')
queue 'by queue'
push 'by push'
say q~queued queued()
say q~pull '|' q~lineIn
q~queue('from object')
pull line
say line queued()
say q~set('session') rxqueue('Get') q~get
call rxqueue 'Set', before
say .rexxqueue~delete('spec_rq_program')
```

```output
2 2
by push | by queue
FROM OBJECT 0
SPEC_RQ_PROGRAM SPEC_RQ_PROGRAM SESSION
0
```

## Deviations

r[class.rexxqueue.deviation.109]
A name or a line that is an instance of a subclass of `String` is read by its
own characters, and `request`, `makeString` and `string` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an instance
`request` with `STRING`, so an override can change what is read. In the
example the overrides would change the line and the name; they count their
sends and the string is read as it is without one. The `upper` that `init` and
`set` send to a name is sent in either case, because it is an ordinary message
to an ordinary object ([the `init` rule](r:class.rexxqueue.init)).

```rexx
.environment~sent = 0
q = .rexxqueue~new('spec_rq_dev109')
q~queue(.digits~new('own text'))
say q~pull .environment~sent
say .rexxqueue~exists(.digits~new('spec_rq_dev109')) .environment~sent
say q~delete
::class digits subclass string
::method makestring
  .environment~sent = .environment~sent + 1
  return 'other'
::method request
  .environment~sent = .environment~sent + 1
  return 'other'
```

```output
own text 0
1 0
0
```

r[class.rexxqueue.deviation.135]
A subclass of `RexxQueue` whose `init` does not send `init` to its superclass,
by `self~init:super` or `FORWARD`, makes an object that is not connected to a
queue, and every instance method of the class other than `init` sent to it
raises `SYNTAX 98.901`: `delete`, `empty`, `get`, `lineIn`, `lineOut`,
`makeArray`, `pull`, `push`, `queue`, `queued`, `say` and `set`. The example
gives each method the arguments it takes. The class methods do not depend on an
object and are not affected. ooRexx ends the interpreter with a segmentation
fault from `empty`, so the example is not run against it.

```rexx
o = .lazy~new
call try "o~delete"
call try "o~empty"
call try "o~get"
call try "o~lineIn"
call try "o~lineOut('x')"
call try "o~makeArray"
call try "o~pull"
call try "o~push('x')"
call try "o~queue('x')"
call try "o~queued"
call try "o~say('x')"
call try "o~set('x')"
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class lazy subclass rexxqueue
::method init
  nop
```

```output defect=#197
o~delete => 98.901
o~empty => 98.901
o~get => 98.901
o~lineIn => 98.901
o~lineOut('x') => 98.901
o~makeArray => 98.901
o~pull => 98.901
o~push('x') => 98.901
o~queue('x') => 98.901
o~queued => 98.901
o~say('x') => 98.901
o~set('x') => 98.901
```
