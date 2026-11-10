---
element: "Environment objects"
kind: concept
rexxref: "Rexx Runtime Objects; Rexx Runtime Objects > The Environment Directory (.ENVIRONMENT); Rexx Runtime Objects > The Local Directory (.LOCAL); Rexx Runtime Objects > The Input Monitor (.INPUT); Rexx Runtime Objects > The Output Monitor (.OUTPUT); Rexx Runtime Objects > The Error Monitor (.ERROR); Rexx Runtime Objects > The Trace Output Monitor (.TRACEOUTPUT); Rexx Runtime Objects > The Debug Input Monitor (.DEBUGINPUT); Rexx Runtime Objects > The STDIN Stream (.STDIN); Rexx Runtime Objects > The STDOUT Stream (.STDOUT); Rexx Runtime Objects > The STDERR Stream (.STDERR); Rexx Runtime Objects > The STDQUE Queue (.STDQUE); Rexx Runtime Objects > The SYSCARGS Array (.SYSCARGS); Rexx Runtime Objects > The ENDOFLINE Constant (.ENDOFLINE); Rexx Runtime Objects > The NIL Object (.NIL); Rexx Runtime Objects > The TRUE Constant (.TRUE); Rexx Runtime Objects > The FALSE Constant (.FALSE); Rexx Runtime Objects > The RexxInfo Object (.RexxInfo); Rexx Runtime Objects > The Rexx Context (.CONTEXT); Rexx Runtime Objects > The Line Number (.LINE)"
---

# Environment objects

A program reaches the objects that the language provides through environment
symbols: a period followed by a name, such as `.output`, `.true` or `.array`.
The value of such a symbol is found by a lookup that this page specifies:
first which objects exist and what they are, then the places the lookup goes
through and their order, and last what a program may change. The lookup that
reads the symbol is [the environment rule](r:lex.symbols.environment) of the
lexical chapter; the order in which it consults the places is the rule of this
page.

Two directories hold most of the objects. `.environment` holds the objects
that the whole program can reach, and `.local` the ones that belong to one
interpreter instance, such as the standard streams. A program adds its own
objects to them as it adds entries to any other `Directory`.

> **Note.** Each interpreter instance has its own `.environment`
> ([Deviation 85](../deviations.md#85-environment-per-interpreter)). A program
> that `rexx` runs has one instance, and so has every example in this book.

## The directories

r[env.objects.directories]
`.environment` and `.local` are each a `Directory`, the same object every time
the symbol is read. `.environment` holds an entry `ENVIRONMENT` for itself and
an entry `LOCAL` for `.local`, so that the symbols `.environment` and `.local`
are found in `.environment` like every other. `.local` holds neither of those
entries. An entry is read with the methods of `Directory`, by its name in
upper case, and the entries of this page are the ones that exist before the
program changes anything.

```rexx
say .environment~class~id .local~class~id (.environment == .environment),
    (.local == .local)
say (.environment~at('ENVIRONMENT') == .environment)
say (.environment~at('LOCAL') == .local)
say .local~hasIndex('LOCAL') .local~hasIndex('ENVIRONMENT')
```

```output
Directory Directory 1 1
1
1
0 0
```

## What `.environment` holds

r[env.objects.entries]
`.environment` holds the class objects of the [classes
chapter](../classes/index.md), each under its class name in upper case, with
two differences: `RegularExpression` is not among them, because it belongs to
a package that a program [requires](../classes/regularexpression.md), and
`ArgUtil` is ([the rule on it](r:class.index.argutil)). The entries that are
not classes are `NIL`, `TRUE`, `FALSE`, `ENDOFLINE`, `REXXINFO`, `LOCAL` and
`ENVIRONMENT`, and there are no others. `.true` is the string `1`, `.false`
the string `0`, and `.endofline` the string of the one character `'0A'x`, the
line end of the platform. `.nil` is the object that stands for the absence of
an object (`isNil` [answers 1 for it](r:class.object.isnil)). `.rexxinfo` is
the one `RexxInfo` object ([its rule](r:class.rexxinfo.creation)). The example
prints the class entries in rows, after checking that the upper case id of
each is its index, and then the entries that are none.

```rexx
others = .array~new
row = ''
do i over .environment~allIndexes~sort
  o = .environment~at(i)
  if \o~isA(.class) then others~append(i)
  else do
    if o~id~upper \== i then say 'id differs:' i
    if length(row i) > 78 then do
      say row~strip
      row = ''
    end
    row = row i
  end
end
say row~strip
say others~makeString('l', ' ')
say .true .false c2x(.endofline) .nil~isNil .rexxinfo~class~id
say .environment~hasIndex('REGULAREXPRESSION')
```

```output
ALARM ALARMNOTIFICATION ARGUTIL ARRAY BAG BUFFER CASELESSCOLUMNCOMPARATOR
CASELESSCOMPARATOR CASELESSDESCENDINGCOMPARATOR CIRCULARQUEUE CLASS
COLLECTION COLUMNCOMPARATOR COMMANDCONTEXT COMMANDHANDLER COMMANDINPUT
COMMANDOUTPUT COMPARABLE COMPARATOR DATETIME DESCENDINGCOMPARATOR DIRECTORY
EVENTSEMAPHORE FILE IDENTITYTABLE INPUTOUTPUTSTREAM INPUTSTREAM
INVERTINGCOMPARATOR LIST MAPCOLLECTION MESSAGE MESSAGENOTIFICATION METHOD
MONITOR MUTABLEBUFFER MUTEXSEMAPHORE NATIVECOMMANDHANDLER NUMERICCOMPARATOR
OBJECT ORDERABLE ORDEREDCOLLECTION OUTPUTSTREAM PACKAGE POINTER PROPERTIES
QUEUE RELATION REXXCONTEXT REXXQUEUE ROUTINE SET SETCOLLECTION SINGLETON
STACKFRAME STEM STREAM STREAMSUPPLIER STRING STRINGTABLE SUPPLIER TABLE
TICKER TIMESPAN TRACEOBJECT VALIDATE VARIABLEREFERENCE WEAKREFERENCE
ENDOFLINE ENVIRONMENT FALSE LOCAL NIL REXXINFO TRUE
1 0 0A 1 RexxInfo
0
```

## What `.local` holds

r[env.objects.local]
`.local` holds, when a program starts, the five monitors `INPUT`, `OUTPUT`,
`ERROR`, `TRACEOUTPUT` and `DEBUGINPUT`, each a
[`Monitor`](../classes/monitor.md); the three streams `STDIN`, `STDOUT` and
`STDERR`, each a [`Stream`](../classes/stream.md); `STDQUE`, a
[`RexxQueue`](../classes/rexxqueue.md); and `SYSCARGS`, an `Array`. The
symbols `.input`, `.output`, `.error`, `.traceoutput`, `.debuginput`,
`.stdin`, `.stdout`, `.stderr`, `.stdque` and `.syscargs` are found there.
`.syscargs` is the array of the command line arguments that follow the program
name; a program started without any has none, as the example shows, and how a
command line becomes those arguments is part of starting a program.

```rexx
do i over .local~allIndexes~sort
  say i~left(12) .local~at(i)~class~id
end
say .syscargs~items
```

```output
DEBUGINPUT   Monitor
ERROR        Monitor
INPUT        Monitor
OUTPUT       Monitor
STDERR       Stream
STDIN        Stream
STDOUT       Stream
STDQUE       RexxQueue
SYSCARGS     Array
TRACEOUTPUT  Monitor
0
```

r[env.objects.bindings]
The monitors start out connected to each other and to the streams: the current
destination of `.output` is `.stdout`, that of `.error` is `.stderr`, that of
`.input` is `.stdin`, that of `.traceoutput` is the monitor `.error` and that
of `.debuginput` is the monitor `.input`, each the same object and not a copy
([the rule on `current`](r:class.monitor.current)). A message that a monitor
does not understand goes to its current destination, so a destination that a
program sets on `.error` is also where the trace lines go, as the [trace
rule](r:instr.trace.destination) says for `.traceoutput`. `.stdque` is the
queue `SESSION`, which holds no line.

```rexx
say (.output~current == .stdout) (.error~current == .stderr),
    (.input~current == .stdin) (.traceoutput~current == .error),
    (.debuginput~current == .input)
say .stdque~class~id .stdque~get .stdque~queued
```

```output
1 1 1 1 1
RexxQueue SESSION 0
```

## The order of the lookup

r[env.objects.search]
A program that reads the environment symbol `.name` gets the first object that
the following places answer to the name, taken in upper case, and the symbol's
own text when none does ([the last rule of this
section](r:env.objects.search.none)). The places, in order, are: the three
fixed symbols `.nil`, `.true` and `.false`; the classes that the code can see,
beginning with those its own package declares; the `local` directory of the
package that holds the code; `.local`; `.environment`; and the symbols
`.line`, `.context`, `.methods`, `.routines`, `.resources` and `.rs`, which
the interpreter answers for the code that reads them. The rules that follow
state each step. In the example, `.zqa` is found as a class, `.zqb` in
`.local` although `.environment` has it too, `.zqc` in `.environment`, and
`.zqd` nowhere.

```rexx
.local~zqa = 'in local'
.environment~zqa = 'in environment'
.local~zqb = 'in local'
.environment~zqb = 'in environment'
.environment~zqc = 'in environment'
say .zqa~id '|' .zqb '|' .zqc '|' .zqd
::class zqa
```

```output
ZQA | in local | in environment | .ZQD
```

r[env.objects.search.fixed]
The symbols `.nil`, `.true` and `.false` always answer the `.nil` object, the
string `1` and the string `0`. A program that stores an entry of the same name
in `.local` or `.environment`, or that declares a class of that name, changes
that entry and that class and nothing else: the entry is still there to read
with `at`, and the symbol still answers the fixed object.

```rexx
.environment~nil = 'N'
.local~true = 'T'
.environment~false = 'F'
say .nil~isNil .true .false
say .environment~nil .local~true .environment~false
say .environment~at('TRUE') .environment~hasIndex('TRUE')
::class true
```

```output
1 1 0
N T F
1 1
```

r[env.objects.search.classes]
A name is looked up among the classes that the code can see before any
directory is: the classes of its package, the public classes of the packages
it requires, and the classes of the class library, in the order that [the
class rule](r:dir.class.visibility) and
[`findClass`](r:class.package.findclass) give. A library class is found
whatever `.local` and `.environment` hold under its name, and it stays found
when the entry is removed from `.environment`; a class that the package itself
declares hides a library class of the same name.

```rexx
.local~object = 'in local'
.environment~array = 'in environment'
say .object~id .array~id .string~id
say .environment~remove('LIST')~id .environment~hasIndex('LIST')
say .list~id
say .queue~new~hi
::class queue
::method hi
  return 'the queue of this package'
```

```output
Object Array String
List 0
List
the queue of this package
```

r[env.objects.search.local]
When no class has the name, the `local` directory of the package that holds
the code is searched ([its rule](r:class.package.local)), then `.local`, then
`.environment`. An entry of `.local` hides an entry of the same name in
`.environment`, and removing it from `.local` makes the other one visible
again.

```rexx
.environment~zqv = 'environment'
say .zqv
.local~zqv = 'local'
say .zqv '|' .environment~zqv '|' .local~zqv
.local~remove('ZQV')
say .zqv
```

```output
environment
local | environment | local
environment
```

r[env.objects.search.symbols]
`.line`, `.context`, `.methods`, `.routines`, `.resources` and `.rs` are
looked up last. An entry of one of those names in the `local` directory of the
package, in `.local` or in `.environment`, and a class of the name, is the
value of the symbol, and the interpreter's own answer is given only when none
exists. The interpreter answers `.line` with the same number as
[`.context~line`](r:class.rexxcontext.line) and `.context` as [the context
rule](r:class.rexxcontext.symbol) says. The rules for `.methods` ([the
floating methods](r:dir.method.floating)), `.routines` ([the table of
routines](r:dir.routine.table)), `.resources` ([the resource
rule](r:dir.resource.access)) and `.rs` ([the command
status](r:instr.command.status)) say when each exists; when it does not, the
symbol is not found and has its own text. The first example shows `.line` and
`.context`, and the second a `LINE` in the `local` directory of a package and
a class named `rs`.

```rexx
say .line .context~line .rs
.environment~line = 'mine'
say .line
.environment~remove('LINE')
say .line
.local~context = 'also mine'
say .context
.local~remove('CONTEXT')
say .context~class~id
```

```output
1 1 .RS
mine
5
also mine
RexxContext
```

```rexx
src = .array~of('::routine r', '  return .line')
p = .package~new('zq-env-line', src)
say p~findRoutine('R')~call
p~local~put('own', 'LINE')
say p~findRoutine('R')~call
src = .array~of('::routine r', '  return .rs~id', '::class rs')
say .package~new('zq-env-rs', src)~findRoutine('R')~call
```

```output
2
own
RS
```

r[env.objects.search.name]
The name that is looked up is the characters after the period, in upper case,
and the lookup is by that exact index: an entry whose index has a lower case
letter is not found by any symbol, because the symbol's name has none. A
period inside the name is part of it, so `.my.name` looks up the index
`MY.NAME`, which is also how `.environment~my.name` reads it. A name that
nothing holds is not an error and does not raise `NOVALUE` ([the
rule](r:cond.novalue.raise)).

```rexx
.environment['mixed'] = 'lower index'
.environment['MIXED2'] = 'upper index'
.environment['MY.NAME'] = 'dotted'
say .mixed .Mixed2 .my.name .MY.name
say .environment~at('mixed') '|' .environment~mixed2 '|' .environment~my.name
```

```output
.MIXED upper index dotted dotted
lower index | upper index | dotted
```

r[env.objects.search.none]
When no place holds the name, the value of the symbol is its own characters in
upper case, period included ([the lexical rule](r:lex.symbols.environment)).
An entry whose value is `.nil` is held: the symbol answers `.nil` and not its
own text.

```rexx
.environment~zqnil = .nil
say .zqnil~isNil .zqnone .Zq.None .zqnil2
```

```output
1 .ZQNONE .ZQ.NONE .ZQNIL2
```

## What a program may change

r[env.objects.change.entries]
A program may add to `.local` and `.environment` an entry of any name and any
object, replace an entry, and remove one, with the methods of `Directory`. The
symbol then finds the object, the new one after a replacement and the one
below it, or none, after a removal, as the order above says. The entries that
the language put there are entries like the others, except where a rule of
this page says that a symbol does not read them: the classes of the library,
and `.nil`, `.true` and `.false`.

```rexx
.local~zqx = 'first'
say .zqx
.local~zqx = .array~of(1, 2)
say .zqx~items
.local~remove('ZQX')
say .zqx
.environment~endofline = 'eol'
say .endofline
```

```output
first
2
.ZQX
eol
```

r[env.objects.change.scope]
An entry that a program adds or changes is seen by everything that runs in the
same interpreter instance afterwards: by the code of other packages, a and by
the other activities. The `local` directory of a package is not shared ([its
rule](r:class.package.local)). In the example, a package that the program
makes reads `.zqenv` and `.zqloc` and sets `.zqfrom`, which the main program
reads, and the method of a started message reads `.zqenv` and sets `.zqact`.

```rexx
.environment~zqenv = 'env'
.local~zqloc = 'loc'
src = .array~of('::routine see', '  return .zqenv .zqloc .zqfrom',,
  '::routine put', '  .environment~zqfrom = "set by the package"',,
  '  return 0')
p = .package~new('zq-env-scope', src)
say p~findRoutine('SEE')~call
x = p~findRoutine('PUT')~call
say .zqfrom '|' p~findRoutine('SEE')~call
say .zqo~new~start('put')~result
say .zqact
::class zqo
::method put
  .local~zqact = 'set by an activity'
  return .zqenv .zqfrom
```

```output
env loc .ZQFROM
set by the package | env loc set by the package
env set by the package
set by an activity
```

## Where input and output go

> **Note.** The monitors in `.local` are the one place through which the
> language reads and writes the standard streams, so a program that gives one
> a new destination, or that puts another object in its place, changes what
> other code reads and writes without that code noticing. The rules below
> state which messages the language sends to each of these objects. The
> examples use a spy, an object that logs the name of each message it is sent
> and answers a value of its own.

r[env.objects.output]
SAY sends the message `SAY` to the object that `.output` finds, with the text
it writes. The builtins CHAROUT and LINEOUT, when their stream name is null,
omitted or one of `STDOUT` and `STDOUT:` in any case ([the
rule](r:bif.index.stream.std)), send it `CHAROUT` and `LINEOUT` instead, and
the builtin answers what the object answers. A message that a program sends to
`.stdout` itself is not redirected: it writes to the stream whatever `.output`
holds. The monitor sends all of these on to its current destination, which is
`.stdout` until a program sets another.

```rexx
s = .spy~new
.output~destination(s)
say 'one'
call charout , 'two'
call lineout , 'three'
x = lineout('STDOUT', 'four')
call charout 'stdout:', 'five'
.stdout~lineout('direct')
.output~destination
say 'restored'
say s~log~makeString('l', ' | ') x
::class spy
::method init
  expose log
  log = .array~new
::attribute log get
::method unknown
  expose log
  use arg name
  log~append(name)
  return 7
```

```output
direct
restored
SAY | CHAROUT | LINEOUT | LINEOUT | CHAROUT 7
```

r[env.objects.error]
The builtins CHAROUT and LINEOUT, when their stream name is `STDERR` or
`STDERR:` in any case, send `CHAROUT` and `LINEOUT` to the object that
`.error` finds. The lines of a trace go to `.traceoutput` ([the trace
rule](r:instr.trace.destination)), whose monitor passes them on to `.error`,
so a destination that a program sets on `.error` receives them as `LINEOUT`
messages whose argument is a `TraceObject`. A message sent to `.stderr`
directly is not redirected. The example logs the name of each message and the
class of its argument.

```rexx
s = .spy~new
.error~destination(s)
call lineout 'STDERR', 'one'
call charout 'stderr:', 'two'
trace r
x = 1
trace o
.error~destination
u = .set~new
do e over s~log
  u~put(e)
end
say u~makeArray~sort~makeString('l', ' | ')
::class spy
::method init
  expose log
  log = .array~new
::attribute log get
::method unknown
  expose log
  use arg name, args
  log~append(name args[1]~class~id)
  return 0
```

```output
CHAROUT String | LINEOUT String | LINEOUT TraceObject
```

r[env.objects.input]
The builtins LINEIN, CHARIN, LINES and CHARS, when their stream name is null,
omitted or one of `STDIN` and `STDIN:` in any case, send `LINEIN`, `CHARIN`,
`LINES` and `CHARS` to the object that `.input` finds. LINEIN and CHARIN
answer what the object answers. PARSE LINEIN does the same as LINEIN with no
name, and a message that a program sends to `.stdin` itself is not redirected.
The example is run with an empty standard input, which the last read, of
`.stdin`, then meets.

```rexx
s = .spy~new
.input~destination(s)
x = linein()
y = linein('STDIN')
z = charin()
n = chars('stdin:')
n = lines()
parse linein v
n = .input~linein
.input~destination
n = .stdin~linein
say s~log~makeString('l', ' ') x y z v
::class spy
::method init
  expose log
  log = .array~new
::attribute log get
::method unknown
  expose log
  use arg name
  log~append(name)
  return 7
```

```output
LINEIN LINEIN CHARIN CHARS LINES LINEIN LINEIN 7 7 7 7
```

r[env.objects.queue]
PUSH and QUEUE send `PUSH` and `QUEUE` with the line to the object that
`.stdque` finds, and the builtin QUEUED sends it `QUEUED`. The builtin RXQUEUE
sends `GET` for its option `GET` and `SET` for its option `SET`. PULL and
PARSE PULL send `PULL`; when the answer is `.nil`, they send `LINEIN` to the
object that `.input` finds and use its answer as the line; otherwise the
answer is the line ([the pull rule](r:instr.pull.source)). The example
restores `.stdque` and the destination of `.input` before it prints.

```rexx
old = .stdque
q = .spy~new('Q', .nil)
i = .spy~new('I', 'from input')
.local~stdque = q
.input~destination(i)
push 'a'
queue 'b'
x = queued()
x = rxqueue('get')
x = rxqueue('set', 'zq')
pull v
.local~stdque = old
.input~destination
say q~log~makeString('l', ' ') '|' i~log~makeString('l', ' ') '|' v
::class spy
::method init
  expose log answer
  use arg , answer
  log = .array~new
::attribute log get
::method unknown
  expose log answer
  use arg name
  log~append(name)
  if name == 'PULL' | name == 'LINEIN' then return answer
  return 0
```

```output
PUSH QUEUE QUEUED GET SET PULL | LINEIN | FROM INPUT
```

r[env.objects.replace]
The language finds `.output`, `.error`, `.input`, `.traceoutput` and `.stdque`
each time it needs one, as the lookup of this page does, so a program that
stores another object under one of those names in `.local` makes that object
the one that the rules above send their messages to, from then on. An entry
that `.environment` holds under the same name is hidden by the one in
`.local`. In the example, the object that replaces `.output` is sent `SAY`,
`LINEOUT` and `CHAROUT`, the one that replaces `.input` is sent `LINEIN`, the
one that replaces `.error` is sent `LINEOUT`, the one that replaces
`.traceoutput` is sent the trace lines, and the one that replaces `.stdque` is
sent `PUSH`.

```rexx
old = .local~output
.local~output = .cap~new('out')
say 'one'
call lineout , 'two'
call charout , 'three'
.environment~output = .cap~new('env')
say 'four'
.local~output = old
.environment~remove('OUTPUT')
old = .local~input
.local~input = .cap~new('in')
parse linein v
.local~input = old
say 'read' v
old = .local~error
.local~error = .cap~new('err')
call lineout 'STDERR', 'five'
.local~error = old
old = .local~traceoutput
.local~traceoutput = .cap~new('trace')
trace r
x = 1
trace o
.local~traceoutput = old
old = .local~stdque
.local~stdque = .cap~new('que')
push 'six'
.local~stdque = old
::class cap
::method init
  expose tag
  use arg tag
::method say
  expose tag
  .stdout~lineout(tag 'say' arg(1))
  return 0
::method lineout
  expose tag
  .stdout~lineout(tag 'lineout' arg(1)~class~id)
  return 0
::method charout
  expose tag
  .stdout~lineout(tag 'charout' arg(1))
  return 0
::method linein
  expose tag
  return tag
::method push
  expose tag
  .stdout~lineout(tag 'push' arg(1))
  return 0
```

```output
out say one
out lineout String
out charout three
out say four
read in
err lineout String
trace lineout TraceObject
trace lineout TraceObject
trace lineout TraceObject
que push six
```
