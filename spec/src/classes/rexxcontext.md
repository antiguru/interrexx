---
element: "RexxContext"
kind: class
rexxref: "Builtin Classes > Utility Classes > RexxContext Class; Builtin Classes > Utility Classes > RexxContext Class > args; Builtin Classes > Utility Classes > RexxContext Class > condition; Builtin Classes > Utility Classes > RexxContext Class > digits; Builtin Classes > Utility Classes > RexxContext Class > executable; Builtin Classes > Utility Classes > RexxContext Class > form; Builtin Classes > Utility Classes > RexxContext Class > fuzz; Builtin Classes > Utility Classes > RexxContext Class > interpreter; Builtin Classes > Utility Classes > RexxContext Class > invocation; Builtin Classes > Utility Classes > RexxContext Class > line; Builtin Classes > Utility Classes > RexxContext Class > name; Builtin Classes > Utility Classes > RexxContext Class > package; Builtin Classes > Utility Classes > RexxContext Class > rs; Builtin Classes > Utility Classes > RexxContext Class > stackFrames; Builtin Classes > Utility Classes > RexxContext Class > thread; Builtin Classes > Utility Classes > RexxContext Class > variables"
deviations: [51, 63, 71]
---

# RexxContext

A `RexxContext` describes one piece of running code: the main part of a
program, a routine, a method or the text of an INTERPRET. The environment
symbol `.CONTEXT` answers the context of the code that names it. Its methods
answer what the code was started with (its arguments, name and executable),
what it has set (the NUMERIC settings, the status of the last command, the
variables), where it is (the line, the call stack) and which run it belongs
to. A program cannot make a `RexxContext`; it only receives them.

Scheduling between activities belongs to [the concurrency
chapter](r:conc.activities.order), and the
examples here use a second activity only to show that its context is another
one. The condition object, TRACE and the CALL and SIGNAL instructions are
specified in their own chapters, and this page links their rules.

## The class

r[class.rexxcontext.creation]
`RexxContext` is a class whose only superclass is `Object`, and it inherits no
mixin class ([the hierarchy rule](r:class.index.hierarchy)). Its instance side
defines `args`, `condition`, `copy`, `digits`, `executable`, `form`, `fuzz`,
`interpreter`, `invocation`, `line`, `name`, `package`, `rs`, `stackFrames`,
`thread` and `variables`, and no other method. The class method `new` raises
`SYNTAX 93.967` whatever the arguments are, so no program makes an instance. No
method takes an argument, and one sent with an argument raises `SYNTAX 93.902`,
`copy` included. `copy` without an argument raises `SYNTAX 93.970`; [Deviation
63](r:class.rexxcontext.deviation.63) states the one route on which it differs.

```rexx
s = .rexxcontext~methods(.nil)
names = .array~new
do while s~available
  names~append(s~index)
  s~next
end
names = names~sort
do i = 1 to names~items by 5
  say names~section(i, min(5, names~items - i + 1))~makeString('l', ' ')
end
say .rexxcontext~superclasses~items .rexxcontext~superclass~id
call try '.rexxcontext~new'
call try '.rexxcontext~new(1, 2)'
call try '.context~copy'
call try '.context~name(1)'
call try '.context~copy(1)'
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
ARGS CONDITION COPY DIGITS EXECUTABLE
FORM FUZZ INTERPRETER INVOCATION LINE
NAME PACKAGE RS STACKFRAMES THREAD
VARIABLES
1 Object
.rexxcontext~new => 93.967
.rexxcontext~new(1, 2) => 93.967
.context~copy => 93.970
.context~name(1) => 93.902
.context~copy(1) => 93.902
```

r[class.rexxcontext.symbol]
The environment symbol `.CONTEXT` answers the `RexxContext` of the code that
is running, unless an entry or a class of the same name is found first ([the
order](r:env.objects.search.symbols)). Within one run of that code it answers
the same object each time, and the code of a called routine, of a method and
of a message that another activity runs has another context. An INTERPRET runs
in the context of the code that holds it. ([The lookup of environment
symbols](r:env.objects.search) is specified with the environment objects.)

```rexx
c = .context
say c~class~id (c == .context)
call inner c
interpret 'say (c == .context)'
.o~new~m(c)
.o~new~start('m', c)~result
exit
inner: procedure
  say (arg(1) == .context) (arg(1) == .context)
  return
::class o
::method m
  say (arg(1) == .context)
  return 1
```

```output
RexxContext 1
0 0
1
0
0
```

r[class.rexxcontext.active]
A context answers the messages of its instance side only while the code it
belongs to is running. Once that code has returned, every one of them raises
`SYNTAX 98.981`. The context of code that is still running answers, as the
main program's does when a routine that it called asks.

```rexx
call keep
c = kept
names = 'args condition digits executable form fuzz interpreter invocation',
  'line name package rs stackFrames thread variables'
do m over names~makearray(' ')
  call try c, m
end
call alive .context
exit
keep: procedure expose kept
  kept = .context
  return
alive: procedure
  say 'main still answers' arg(1)~line
  return
try: procedure
  use arg o, m
  signal on syntax name bad
  r = o~send(m)
  say m 'answers'
  return
bad: say m '=>' condition('o')~code
  return
```

```output
args => 98.981
condition => 98.981
digits => 98.981
executable => 98.981
form => 98.981
fuzz => 98.981
interpreter => 98.981
invocation => 98.981
line => 98.981
name => 98.981
package => 98.981
rs => 98.981
stackFrames => 98.981
thread => 98.981
variables => 98.981
main still answers 8
```

## Methods

r[class.rexxcontext.args]
`args` answers a new array each time that holds the arguments of the code,
as [ARG with the option `A`](r:bif.arg.option.array) answers from the first:
an omitted argument is a place with no item. For a method they are the
arguments of the message, for a routine those of the call, and for a program
those it was started with. Changing the array changes nothing the code sees.

```rexx
call show 'a', , 'c'
call show
.o~new~m(1, 2)
exit
show:
  a = .context~args
  say a~class~id a~size a~items arg() (a == .context~args)
  do i = 1 to a~size
    if a~hasIndex(i) then say ' ' i a[i]
  end
  a~append('x')
  say a~items .context~args~items arg()
  return
::class o
::method m
  say .context~args~items .context~args[2]
```

```output
Array 3 2 3 0
  1 a
  3 c
3 2 3
Array 0 0 0 0
1 0 0
2 2
```

r[class.rexxcontext.condition]
`condition` answers what [the context rule of the conditions
chapter](r:cond.information.context) specifies: a new directory with the
entries of the condition that the code is handling, and `.nil` when it is
handling none.

```rexx
say .context~condition~isNil
signal on syntax name h
x = 1 + 'a'
h:
c = .context~condition
say c~code c~condition (c == condition('o'))
```

```output
1
41.1 SYNTAX 0
```

r[class.rexxcontext.numeric]
`digits`, `form` and `fuzz` answer the NUMERIC settings in force in the code
at that moment, as the builtins DIGITS, FORM and FUZZ do, and the scope of a
setting is the one that [the DIGITS rule](r:bif.digits.scope) gives, the
same for the other two: a routine called by its label starts with its
caller's settings and its own changes do not outlast it, and a method starts
with the settings of its package.

```rexx
numeric digits 12; numeric form engineering; numeric fuzz 2
say .context~digits .context~form .context~fuzz
call plain
say .context~digits
say .o~new~show
numeric digits 6
say .context~digits digits()
exit
plain:
  say .context~digits .context~form .context~fuzz
  numeric digits 20
  say .context~digits digits()
  return
::class o
::method show
  return .context~digits .context~form .context~fuzz
```

```output
12 ENGINEERING 2
12 ENGINEERING 2
20 20
12
9 SCIENTIFIC 0
6 6
```

r[class.rexxcontext.executable]
`executable` answers the object that holds the code: a `Routine` for a
program, for a `::ROUTINE` and for an internal routine, which is part of the
program, and a `Method` for a method. The routine of a `::ROUTINE` is the one
that `findRoutine` of its package finds. An INTERPRET has the executable of
the code that holds it.

```rexx
e = .context~executable
say e~class~id (e~package == .context~package)
call inner
interpret 'say (.context~executable == e)'
call rt
.o~new~m
exit
inner:
  say (.context~executable == e)
  return
::routine rt
  say .context~executable~class~id,
      (.context~executable == .context~package~findRoutine('RT'))
::class o
::method m
  say .context~executable~class~id (.context~executable~scope == .o)
```

```output
Routine 1
1
1
Routine 1
Method 1
```

r[class.rexxcontext.numbers]
`interpreter`, `thread` and `invocation` answer whole numbers that identify the
run: `interpreter` the interpreter instance, `thread` the activity that runs the
code and `invocation` this call of the code. The answers of one context never
change, a routine called from the code has the same interpreter and thread and
another invocation, and a message that another activity runs has another thread.
A number is given to an invocation when it is first needed, counting from `1`:
by this method; by `stackFrames` and by a condition, each of which numbers every
level of the stack that has none yet, the innermost first; by a trace line, for
the level it traces; and by `start`, for the level that sends it and then for
the level that runs the message. Scheduling is specified in the environment
chapter.

```rexx
call a
say 'main' .context~invocation
call b
say 'main' .context~invocation
t = .context~thread
i = .context~interpreter
x = .o~new
say (t == x~start('t')~result) (t == .context~thread),
    (i == x~start('i')~result)
call same t, i
exit
a: call c
   say 'a' .context~invocation
   return
c: say 'c' .context~invocation
   return
b: say 'b' .context~invocation
   return
same: say (arg(1) == .context~thread) (arg(2) == .context~interpreter)
   return
::class o
::method t
  return .context~thread
::method i
  return .context~interpreter
```

```output
c 1
a 2
main 3
b 4
main 3
0 1 1
1 1
```

```rexx
call a
say 'main' .context~invocation
exit
a:
  call on error name e
  address system 'exit 2'
  call b
  return
e:
  return
b:
  signal on novalue name n
  y = zzz
n:
  m = .message~new(.o~new, 'm')
  m~send
  say 'b' .context~invocation
  call c
  return
c:
  x = .o~new
  r = x~start('m')
  r~result
  say 'c' .context~invocation
  return
::class o
::method m
  say 'm' .context~invocation
  return 1
```

```output
m 4
b 3
m 6
c 5
main 2
```

```rexx
call q
exit
q: call r
   return
r: f = .context~stackFrames
   say f[3]~invocation f[2]~invocation f[1]~invocation
   return
```

```output
3 2 1
```

r[class.rexxcontext.line]
`line` answers the number of the line on which the clause being run starts,
as [the line number of a trace line](r:instr.trace.line) is: a clause that
continues over several lines has the number of its first, and a clause that an
INTERPRET runs has the number of the INTERPRET clause.

```rexx
say .context~line
say 'a',
    .context~line
interpret 'say .context~line;' 'say .context~line'
x = 5; say .context~line; say .context~line
do i = 1 to 2
  say .context~line
end
call rt
exit
::routine rt
  say .context~line
```

```output
1
a 2
4
4
5
5
7
7
12
```

r[class.rexxcontext.name]
`name` answers the name by which the code was started. For the main part of a
program, and for an INTERPRET in it, it is the name of the package (see
[`.context~package~name`](r:class.package.name)). For a routine it is the name
as the CALL or the function call gave it: a name written as a symbol in upper
case, the characters of a string as they are written. For a routine run with
`call` of a `Routine`, it is the name of that routine. For a method it is the
name of the method that runs, in upper case, which is the message name, so
`UNKNOWN` when the `unknown` method of an object answers a message.

```rexx
say .context~name == .context~package~name
call rt
call 'rT'
x = rt()
x = 'Rt'()
say .routine~new('anon', 'return .context~name')~call
x = .context~package~findRoutine('RT')~call
call inner
x = .o~new
x~Mixed
x~send('bar')
x~missing
interpret 'say .context~name == .context~package~name'
exit
inner:
  say .context~name
  return
::routine rt
  say .context~name
  return 1
::class o
::method mixed
  say .context~name
::method bar
  say .context~name
::method unknown
  say .context~name
```

```output
1
RT
rT
RT
Rt
anon
RT
INNER
MIXED
BAR
UNKNOWN
1
```

r[class.rexxcontext.package]
`package` answers the [`Package`](package.md) that holds the code: the same
object as the package of its `executable`, and the same for every code of one
source file, a method and a routine of a `::ROUTINE` included.

```rexx
p = .context~package
say p~class~id (p == .context~executable~package)
call rt p
.o~new~m(p)
interpret 'say (p == .context~package)'
exit
::routine rt
  say (arg(1) == .context~package)
::class o
::method m
  say (arg(1) == .context~package),
      (.context~package == .context~executable~package)
```

```output
Package 1
1
1 1
1
```

r[class.rexxcontext.rs]
`rs` answers the status of the last command that the code ran, as [the
`.RS` rule](r:instr.command.status) says, and `.nil` before any command has
run. It follows [the scope rule of `.RS`](r:instr.command.status.scope): a
method starts with none, and a routine called by its label starts with its
caller's.

```rexx
say .context~rs
address system 'exit 3'
say .context~rs .rs
address system 'exit 0'
say .context~rs .rs
call inner
say .o~new~m
exit
inner:
  say .context~rs
  address system 'exit 4'
  return
::class o
::method m
  return .context~rs
```

```output
The NIL object
1 1
0 0
0
The NIL object
```

r[class.rexxcontext.stackframes]
`stackFrames` answers a new array of new `StackFrame` objects, one for each
active level of code: the first item is the code that asked, the next its
caller, and so on, with the main part of the program last. Each call of an
internal routine, a routine, a method and an INTERPRET is a level, and so is a
built-in method that runs code of the program, such as `send`. A condition's
list is [specified with the condition object](r:cond.information.object); the
methods of a frame are on [the `StackFrame` page](stackframe.md).

```rexx
call outer
exit
outer: procedure
  .o~new~send('m')
  return
::class o
::method m
  interpret 'call dump'
::routine dump
  fs = .context~stackFrames
  say fs~class~id fs~items (fs == .context~stackFrames)
  do i = 1 to fs~items
    say i fs[i]~type fs[i]~line
  end
  say (fs[1]~context == .context) (fs[1] == .context~stackFrames[1])
```

```output
Array 6 0
1 ROUTINE 10
2 INTERPRET 8
3 METHOD 8
4 METHOD The NIL object
5 INTERNALCALL 4
6 PROGRAM 1
1 0
```

r[class.rexxcontext.variables]
`variables` answers a new directory of the variables of the code: the simple
variables that have a value and the stems, each under its name in upper case,
with a stem's name ending in a period and its value the stem object. A
compound variable is not an entry; it is reached through its stem. The
directory is a snapshot: later assignments do not change it, and a change to
it assigns nothing, but a stem in it is the stem of the code. A variable that
was dropped is not listed. A routine with `PROCEDURE` lists only its own
variables, and a method lists its exposed variables with its own and also
`SELF` and `SUPER`. [Deviation 71](r:instr.use.deviation.71) covers the
alias that `USE ARG` makes.

```rexx
a = 1; b. = 'dflt'; b.x = 2; c.d = 5
aa = .nil
e = 'gone'; drop e
v = .context~variables
say v~class~id (v == .context~variables)
say v~allIndexes~sort~makeString('l', ' ')
say v['A'] v['AA']~isNil v['B.']~class~id v['B.']['X'] v~hasIndex('B.X')
a = 99
say v['A'] .context~variables['A']
v['NEWVAR'] = 'put'
say var('NEWVAR') .context~variables~hasIndex('NEWVAR')
v['B.']['Y'] = 7
say b.y
x = 'main'
call proc
.o~new~m
exit
proc: procedure
  y = 1
  v = .context~variables
  say 'proc' v~hasIndex('Y') v~hasIndex('X') v~hasIndex('A')
  return
::class o
::method m
  expose q
  q = 1
  u = 2
  v = .context~variables
  say 'm' v~hasIndex('Q') v~hasIndex('U') v~hasIndex('SELF'),
      v~hasIndex('SUPER') v~hasIndex('X')
```

```output
Directory 0
A AA B. C.
1 1 Stem 2 0
1 99
0 0
7
proc 1 0 0
m 1 1 1 1 0
```

## Deviations

r[class.rexxcontext.deviation.51]
Inside an INTERPRET, `.context~rs` answers the same value as `.RS`, as
[the INTERPRET rule](r:instr.interpret.deviation.51) states for the text it
runs. ooRexx answers the value that the code had before the INTERPRET until
the text ends.

```rexx
'true'
interpret '"false"; say .rs .context~rs'
```

```output
1 1
```

r[class.rexxcontext.deviation.63]
A `RexxContext` is never copied: a `copy` sent with a scope, as
`.context~copy:.object` is, raises `SYNTAX 93.970` as every other route
does ([the creation rule](r:class.rexxcontext.creation)). ooRexx copies the
context in that case.

```rexx
call try '.context~copy:.object'
call try '.context~copy'
call try '.context~send(.array~of("copy", .object))'
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
.context~copy:.object => 93.970
.context~copy => 93.970
.context~send(.array~of("copy", .object)) => 93.970
```

r[class.rexxcontext.deviation.71]
`variables` lists the alias that `USE ARG` made with a reference under the
name by which the routine reaches it, as [the USE
rule](r:instr.use.deviation.71) states. ooRexx lists it under the name of the
variable that it aliases, so its listing holds the caller's name only.

```rexx
v = 'orig'
call s <v
exit
s:
  use arg <x
  say .context~variables~allIndexes~sort~makeString('l', ' ')
  return
```

```output
SIGL V X
```
