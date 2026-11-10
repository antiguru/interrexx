---
element: "TraceObject"
kind: class
rexxref: "Builtin Classes > Utility Classes > TraceObject Class; Builtin Classes > Utility Classes > TraceObject Class > activate (Class Method); Builtin Classes > Utility Classes > TraceObject Class > collector (Class Method); Builtin Classes > Utility Classes > TraceObject Class > collector= (Class Method); Builtin Classes > Utility Classes > TraceObject Class > counter (Class Method); Builtin Classes > Utility Classes > TraceObject Class > makeStringImpl (Class Method); Builtin Classes > Utility Classes > TraceObject Class > new (Class Method); Builtin Classes > Utility Classes > TraceObject Class > notify (Class Method); Builtin Classes > Utility Classes > TraceObject Class > notify= (Class Method); Builtin Classes > Utility Classes > TraceObject Class > option (Class Method); Builtin Classes > Utility Classes > TraceObject Class > option= (Class Method); Builtin Classes > Utility Classes > TraceObject Class > setMakeString (Class Method); Builtin Classes > Utility Classes > TraceObject Class > unsetMakeString (Class Method); Builtin Classes > Utility Classes > TraceObject Class > compareTo; Builtin Classes > Utility Classes > TraceObject Class > makeString"
deviations: [109]
---

# TraceObject

Every line that [TRACE](r:instr.trace.destination) writes is first an object,
a `TraceObject`: a [`StringTable`](stringtable.md) whose entries describe the
line, the code that produced it and the moment, and whose string value is the
text of the line. The object, and not its text, is what the trace destination
receives. The class lets a program choose how the text is formatted (the
`option`), see every line as an object while the program runs (`notify`) or
afterwards (`collector`), and replace the formatting altogether
(`setMakeString`).

The class methods live on the class object, as the attributes of a singleton
do: `.traceobject~option = 'S'` sets one. The instance methods are
`makeString` and `compareTo`; everything else an instance answers is that of
`StringTable` and its entries. This page links the rules of [the TRACE
instruction](r:instr.trace.line) for what a trace line says and for where it
goes. [Deviation 7](r:instr.trace.deviation.7) covers the order in which
concurrent activities' lines arrive, and [Deviation
50](r:instr.trace.deviation.50) a line the destination refuses.
[Deviation 109](r:class.index.deviation.109) applies to every argument below
that is read as a string.

## The class

r[class.traceobject.creation]
`TraceObject` is a class whose only superclass is `StringTable`; it inherits
no mixin class of its own ([the hierarchy rule](r:class.index.hierarchy)), and
no class of the library is its subclass. Its instance side defines
`compareTo` and `makeString`. The class side answers `activate`, `collector`,
`collector=`, `counter`, `makeStringImpl`, `new`, `notify`, `notify=`,
`option`, `option=`, `setMakeString` and `unsetMakeString`.

```rexx
t = .traceobject
say t~superclasses~items t~superclass~id t~subclasses~items
say t~isSubclassOf(.stringtable) t~isSubclassOf(.directory)
s = t~methods(.nil)
names = .array~new
do while s~available
  names~append(s~index)
  s~next
end
say names~sort~makeString('l', ' ')
list = 'ACTIVATE COLLECTOR COLLECTOR= COUNTER MAKESTRINGIMPL NEW NOTIFY',
  'NOTIFY= OPTION OPTION= SETMAKESTRING UNSETMAKESTRING NOSUCH'
do n over list~makearray(' ')
  say n t~hasMethod(n)
end
```

```output
1 StringTable 0
1 0
COMPARETO MAKESTRING
ACTIVATE 1
COLLECTOR 1
COLLECTOR= 1
COUNTER 1
MAKESTRINGIMPL 1
NEW 1
NOTIFY 1
NOTIFY= 1
OPTION 1
OPTION= 1
SETMAKESTRING 1
UNSETMAKESTRING 1
NOSUCH 0
```

r[class.traceobject.new]
`new` makes a `TraceObject` with three entries: `NUMBER`, the number of the
object, which is the `counter` after the count has gone up by one, `OPTION`,
the `option` at that moment, and `TIMESTAMP`, a `DateTime`. The interpreter
makes the objects for trace lines in the same way and then adds more entries,
as [the entries rule](r:class.traceobject.entries) says. When a `collector`
is set, `new` sends it `append` with the new object. `new` takes no
required argument and accepts one, which it does not use; two raise
`SYNTAX 93.902`.

```rexx
t = .traceobject
say t~counter
n = t~new
say n~class~id n~items t~counter n~allIndexes~sort~makeString('l', ' ')
say n~number n~option n~timestamp~class~id
m = t~new(7)
say m~number t~counter
a = .array~new
t~collector = a
x = t~new
say a~items (a[1] == x) t~counter
t~collector = .nil
signal on syntax name bad
y = t~new(1, 2)
say 'not reached'
bad: say condition('o')~code
```

```output
0
TraceObject 3 1 NUMBER OPTION TIMESTAMP
1 N DateTime
2 2
1 1 3
93.902
```

r[class.traceobject.lines]
A line of trace output is made like this. The interpreter makes the object
with `new`, so that a `collector` sees it first, with only the three entries
of `new`; it adds the other entries; a `notify` object then receives the
complete object with `append`; and last the object is sent to the trace
destination with `lineout`. The destination reads the line through
`makeString`, which is what the string value of the object means for every
use that needs a string. The same `TraceObject` is delivered to all three.

```rexx
.traceoutput~destination(.sink~new)
a = .array~new
b = .array~new
.environment['COLLECTED'] = a
.environment['NOTIFIED'] = b
.traceobject~collector = .rec~new('collect', 'COLLECTED')
.traceobject~notify = .rec~new('notify', 'NOTIFIED')
trace r
x = 1
trace off
.traceobject~collector = .nil
.traceobject~notify = .nil
n = .traceobject~new
say 'say gives' '['n~~put('shown', 'TRACELINE')']' '['n~string']'
::class rec
::method init
  expose name key
  use arg name, key
::method append
  expose name key
  use arg o
  .environment[key]~append(o)
  .output~lineout(name 'append' o~items o~number)
::class sink
::method lineout
  use arg line
  a = .environment['COLLECTED']
  b = .environment['NOTIFIED']
  .output~lineout('destination' line~class~id '['line~makeString']',
    (line == a~lastItem) (line == b~lastItem))
  return 0
```

```output
collect append 3 1
notify append 8 1
destination TraceObject [     9 *-* x = 1] 1 1
collect append 3 2
notify append 8 2
destination TraceObject [       >>>   "1"] 1 1
collect append 3 3
notify append 8 3
destination TraceObject [    10 *-* trace off] 1 1
say gives [shown] [a TraceObject]
```

r[class.traceobject.entries]
A traced line has the entries `INTERPRETER`, `INVOCATION` and `THREAD`, the
numbers of the interpreter, the invocation and the activity of the traced
code as [`RexxContext`](r:class.rexxcontext.numbers) gives them, `NUMBER`,
`OPTION` and `TIMESTAMP` as [`new`](r:class.traceobject.new) makes them,
`TRACELINE`, the text of the line in the `N` format, and `STACKFRAME`, a
`StringTable` that describes the level that produced the line with the
entries `ARGUMENTS`, `EXECUTABLE`, `INVOCATION`, `LINE`, `NAME`, `TARGET`,
`TRACELINE` and `TYPE`, which answer what the [`StackFrame`](stackframe.md)
methods of the same names answer. A line traced in a method adds
`ATTRIBUTEPOOL`, `HASSCOPELOCK`, `ISGUARDED`, `RECEIVER` and
`SCOPELOCKCOUNT`, and the line that announces the method, flagged `>I>`, adds
`CALLERSTACKFRAME`, the frame of the code that called it. A line that shows a
variable being read or assigned adds `VARIABLE`, a `StringTable` with the
entries `NAME`, `VALUE` and `ASSIGNMENT`, which is `1` for an assignment and
`0` for a read. The example shows each set of entries once, with the first
word of the line that has it.

```rexx
.traceoutput~destination(.sink~new)
.traceobject~notify = .rec~new
trace i
x = 1
y = x + 1
call plain
z = .o~new~m
trace off
.traceobject~notify = .nil
exit
::routine plain
  trace i
  q = 2
::class o
::method m
  trace i
  w = 3
  return 1
::class sink
::method lineout
  return 0
::class rec
::method init
  expose seen
  seen = .stringtable~new
::method append
  expose seen
  use arg o
  key = o~allIndexes~sort~makeString('l', ' ')
  if seen~hasIndex(key) then return
  seen[key] = 1
  say o~traceline~word(1)
  line = '  '
  do w over key~makearray(' ')
    if line~length + w~length > 60 then do
      say line
      line = '  '
    end
    line = line w
  end
  say line
  sf = o~stackframe
  say '  STACKFRAME' sf~class~id sf~type
  if o~traceline~word(1) \= '>I>' then say '  line' sf~line
  if o~hasIndex('VARIABLE') then do
    v = o~variable
    say '  VARIABLE' v~allIndexes~sort~makeString('l', ' '),
        v~name v~assignment
  end
  if o~hasIndex('CALLERSTACKFRAME') then do
    c = o~callerstackframe
    say '  CALLERSTACKFRAME' c~class~id c~type c~line
  end
```

```output
4
   INTERPRETER INVOCATION NUMBER OPTION STACKFRAME THREAD
   TIMESTAMP TRACELINE
  STACKFRAME StringTable PROGRAM
  line 4
>=>
   INTERPRETER INVOCATION NUMBER OPTION STACKFRAME THREAD
   TIMESTAMP TRACELINE VARIABLE
  STACKFRAME StringTable PROGRAM
  line 4
  VARIABLE ASSIGNMENT NAME VALUE X 1
>I>
   CALLERSTACKFRAME INTERPRETER INVOCATION NUMBER OPTION
   STACKFRAME THREAD TIMESTAMP TRACELINE
  STACKFRAME StringTable ROUTINE
  CALLERSTACKFRAME StringTable PROGRAM 6
>I>
   ATTRIBUTEPOOL CALLERSTACKFRAME HASSCOPELOCK INTERPRETER
   INVOCATION ISGUARDED NUMBER OPTION RECEIVER SCOPELOCKCOUNT
   STACKFRAME THREAD TIMESTAMP TRACELINE
  STACKFRAME StringTable METHOD
  CALLERSTACKFRAME StringTable PROGRAM 7
17
   ATTRIBUTEPOOL HASSCOPELOCK INTERPRETER INVOCATION
   ISGUARDED NUMBER OPTION RECEIVER SCOPELOCKCOUNT STACKFRAME
   THREAD TIMESTAMP TRACELINE
  STACKFRAME StringTable METHOD
  line 17
>=>
   ATTRIBUTEPOOL HASSCOPELOCK INTERPRETER INVOCATION
   ISGUARDED NUMBER OPTION RECEIVER SCOPELOCKCOUNT STACKFRAME
   THREAD TIMESTAMP TRACELINE VARIABLE
  STACKFRAME StringTable METHOD
  line 17
  VARIABLE ASSIGNMENT NAME VALUE W 1
```

## Class attributes

r[class.traceobject.collector]
`collector` answers the object that is sent `append` with each `TraceObject`
at the moment it is made, `.nil` when there is none, which is the setting at
the start. `collector=` takes that object, any object, or `.nil` to stop the
collecting. It takes exactly one argument: none raises `SYNTAX 93.903` and two
`93.902`. A collector sees the object with the three entries of `new` only;
the others are added afterwards and are there when the program reads the
object later.

```rexx
t = .traceobject
say t~collector~isNil
a = .array~new
t~collector = a
.traceoutput~destination(.output)
trace r
x = 1
trace off
t~collector = .nil
say a~items (t~collector == .nil) a[1]~items
do o over a
  say '['o~traceline']' o~option
end
call try 't~collector(1)'
call try 't~"COLLECTOR="'
call try 't~"COLLECTOR="(1, 2)'
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
1
     7 *-* x = 1
       >>>   "1"
     8 *-* trace off
3 1 8
[     7 *-* x = 1] N
[       >>>   "1"] N
[     8 *-* trace off] N
t~collector(1) => 93.902
t~"COLLECTOR=" => 93.903
t~"COLLECTOR="(1, 2) => 93.902
```

r[class.traceobject.notify]
`notify` and `notify=` work like `collector` and `collector=`, with the
difference that the object is sent `append` when the `TraceObject` is
complete, after any collector and before the line goes to the destination, as
[the lines rule](r:class.traceobject.lines) says. Its default is `.nil`.

```rexx
t = .traceobject
say t~notify~isNil
b = .array~new
t~notify = b
.traceoutput~destination(.output)
trace r
x = 1
trace off
t~notify = .nil
say b~items (t~notify == .nil) b[1]~items
call try 't~"NOTIFY="'
call try 't~"NOTIFY="(1, 2)'
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
1
     7 *-* x = 1
       >>>   "1"
     8 *-* trace off
3 1 8
t~"NOTIFY=" => 93.903
t~"NOTIFY="(1, 2) => 93.902
```

r[class.traceobject.counter]
`counter` answers how many `TraceObject` objects have been made, those that
`new` makes and those made for the lines of trace output, and is `0` before
the first. It takes no argument. `activate` answers no value: it sets the
`counter` to `0`, the `option` to `N` and the `collector` and `notify` to
`.nil`.

```rexx
t = .traceobject
say t~counter
.traceoutput~destination(.output)
trace r
x = 1
trace off
say t~counter
n = t~new
say t~counter n~number
t~option = 'S'
t~collector = .array~new
t~activate
say t~option t~counter t~collector~isNil t~notify~isNil
call try 't~counter(1)'
call try 'r = t~activate'
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
0
     5 *-* x = 1
       >>>   "1"
     6 *-* trace off
3
4 4
N 0 1 1
t~counter(1) => 93.902
r = t~activate => 91.999
```

r[class.traceobject.option]
`option` answers the setting that chooses the format of the trace lines, one
of `N`, `T`, `S`, `F` and `P`, and `N` at the start. `option=` reads its
argument as a string and keeps the first character in upper case if it is one
of `n`, `t`, `s`, `f` or `p`, so that `Normal`, `Thread`, `Standard`, `Full`
and `Profiling` or `Probing` can be written as words; the rest of the string
is not examined. A string that does not begin with one of the five letters,
the null string included, raises `SYNTAX 93.914`. `option=` takes no more
than its first argument into account.

```rexx
t = .traceobject
say t~option
do v over 'n Normal thread Standard FULL profiling probing tx'~makearray(' ')
  t~option = v
  say v '=>' t~option
end
call try '"x"'
call try '""'
call try '5'
call try '" s"'
say t~option
t~option = 'N'
exit
try:
  signal on syntax name bad
  interpret '.traceobject~option =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
N
n => N
Normal => N
thread => T
Standard => S
FULL => F
profiling => P
probing => P
tx => T
"x" => 93.914
"" => 93.914
5 => 93.914
" s" => 93.914
T
```

r[class.traceobject.makestringimpl]
`makeStringImpl` answers the text of one `TraceObject`, which it takes as its
one argument, in the format that the `option` names. With `N` it is the
`TRACELINE` entry, or `?` when the object has none. With `T` the number of
the thread is put inside the flag of the trace line: after the first two
characters of `*-*`, so that it reads `*-1*`, and likewise `>>1>`. With `S`
the line is preceded by `[T`, the thread number padded to three columns and
at least one blank, `I` and the invocation number padded to two columns, `]`
and fifteen blanks. With `F` the prefix is `[R`, the interpreter number
padded to four columns and at least one blank, `T`, the thread number in the
same way, `I`, the invocation number padded to four columns, `]` and eighteen
blanks. A `P` object is formatted like `N`. The line of a method may carry
more fields in the prefix, whose layout is not specified here.
`makeStringImpl` raises `SYNTAX 93.901` with no argument, `93.902` with two
and `93.924` for an argument that is not a collection; a `StringTable`
without entries gives `?`.

```rexx
t = .traceobject
do o over 'N T S F P'~makearray(' ')
  t~option = o
  say 'option' o
  do th over '1 12 123 1234'~makearray(' ')
    do line over '    11 *-* x = 1', '       >>>   "1"'
      n = t~new
      n~traceline = line
      n~thread = th; n~invocation = th; n~interpreter = th
      say '['t~makeStringImpl(n)']'
    end
  end
end
t~option = 'N'
n = t~new
say '['t~makeStringImpl(n)']' '['t~makeStringImpl(.stringtable~new)']'
call try 't~makeStringImpl'
call try 't~makeStringImpl(n, n)'
call try 't~makeStringImpl(1)'
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
option N
[    11 *-* x = 1]
[       >>>   "1"]
[    11 *-* x = 1]
[       >>>   "1"]
[    11 *-* x = 1]
[       >>>   "1"]
[    11 *-* x = 1]
[       >>>   "1"]
option T
[    11 *-1* x = 1]
[       >>1>   "1"]
[    11 *-12* x = 1]
[       >>12>   "1"]
[    11 *-123* x = 1]
[       >>123>   "1"]
[    11 *-1234* x = 1]
[       >>1234>   "1"]
option S
[[T1  I1 ]                   11 *-* x = 1]
[[T1  I1 ]                      >>>   "1"]
[[T12 I12]                   11 *-* x = 1]
[[T12 I12]                      >>>   "1"]
[[T123 I123]                   11 *-* x = 1]
[[T123 I123]                      >>>   "1"]
[[T1234 I1234]                   11 *-* x = 1]
[[T1234 I1234]                      >>>   "1"]
option F
[[R1   T1   I1   ]                      11 *-* x = 1]
[[R1   T1   I1   ]                         >>>   "1"]
[[R12  T12  I12  ]                      11 *-* x = 1]
[[R12  T12  I12  ]                         >>>   "1"]
[[R123 T123 I123 ]                      11 *-* x = 1]
[[R123 T123 I123 ]                         >>>   "1"]
[[R1234 T1234 I1234]                      11 *-* x = 1]
[[R1234 T1234 I1234]                         >>>   "1"]
option P
[    11 *-* x = 1]
[       >>>   "1"]
[    11 *-* x = 1]
[       >>>   "1"]
[    11 *-* x = 1]
[       >>>   "1"]
[    11 *-* x = 1]
[       >>>   "1"]
[?] [?]
t~makeStringImpl => 93.901
t~makeStringImpl(n, n) => 93.902
t~makeStringImpl(1) => 93.924
```

> **Note.** The option applies to the lines a program traces, as the next
> example shows for each setting, with the numbers of the first level of the
> program.

```rexx
.traceoutput~destination(.output)
do o over 'N T S F P'~makearray(' ')
  .traceobject~option = o
  say 'option' o
  trace r
  x = 1
  trace off
end
.traceobject~option = 'N'
```

```output
option N
     6 *-*   x = 1
       >>>     "1"
     7 *-*   trace off
option T
     6 *-1*   x = 1
       >>1>     "1"
     7 *-1*   trace off
option S
[T1  I1 ]                    6 *-*   x = 1
[T1  I1 ]                      >>>     "1"
[T1  I1 ]                    7 *-*   trace off
option F
[R1   T1   I1   ]                       6 *-*   x = 1
[R1   T1   I1   ]                         >>>     "1"
[R1   T1   I1   ]                       7 *-*   trace off
option P
```

> **Note.** The `P` setting writes no line to the destination, but the objects
> are still made, collected and notified, as the next example shows.

```rexx
.traceoutput~destination(.output)
a = .array~new
.traceobject~collector = a
.traceobject~notify = .array~new
.traceobject~option = 'P'
trace r
x = 1
trace off
.traceobject~option = 'N'
.traceobject~collector = .nil
n = .traceobject~notify
.traceobject~notify = .nil
say a~items n~items .traceobject~counter
do o over a
  say '['o~traceline']' o~option '['o~makeString']'
end
```

```output
3 3 3
[     7 *-* x = 1] P [     7 *-* x = 1]
[       >>>   "1"] P [       >>>   "1"]
[     8 *-* trace off] P [     8 *-* trace off]
```

r[class.traceobject.makestring]
`makeString` answers what `makeStringImpl` of the class answers for the
object, so it follows the `option` at the time it is asked and not the option
that the object was made with. That string is the value of the object wherever
a string is needed, such as SAY, the concatenation operators and the trace
destination; the `string` method of the object is the default one. An
argument of `makeString` is not used.

```rexx
t = .traceobject
n = t~new
say '['n~makeString']' '['n']'
n~traceline = 'hello'
say '['n~makeString']' '['n']' '['n~string']' length(n) (n || '!'),
    '['n~makeString(1)']'
t~option = 'S'
n~thread = 4; n~invocation = 5
say '['n~makeString']'
t~option = 'N'
say '['n~makeString']'
n~traceline = .nil
say '['n~makeString']'
```

```output
[?] [?]
[hello] [hello] [a TraceObject] 5 hello! [hello]
[[T4  I5 ]               hello]
[hello]
[?]
```

## Replacing the format

r[class.traceobject.setmakestring]
`setMakeString` replaces `makeStringImpl` with a method of the program: the
argument is a `Method` object, a string that holds the source of a method or
an array of strings that hold its lines. The method receives the `TraceObject`
and answers the text. From then on it supplies `makeStringImpl`, and with it
`makeString`, whatever the `option` is, until `unsetMakeString` takes it
away; a second `setMakeString` replaces the first. It answers no value. A
source that does not translate raises the error that `Method~new` raises for
it, `.nil` raises `SYNTAX 93.961`, no argument `93.901` and a second one
`93.902`. `unsetMakeString` takes no argument and answers no value.

```rexx
.traceoutput~destination(.output)
t = .traceobject
m = .method~new('mk', 'use arg o; return "<<" || o~traceline~strip || ">>"')
t~setMakeString(m)
trace r
x = 1
trace off
t~setMakeString('use arg o; return "S:" || o~number')
trace r
x = 1
trace off
t~setMakeString(.array~of('use arg o', 'return "A:" || o~option'))
trace r
x = 1
trace off
t~option = 'S'
n = t~new
n~traceline = 'abc'
say t~makeStringImpl(n) n~makeString
t~unsetMakeString
say '['t~makeStringImpl(n)']'
t~option = 'N'
say '['t~makeStringImpl(n)']'
call try 't~setMakeString(.nil)'
call try 't~setMakeString'
call try 't~setMakeString("x", "y")'
call try 't~setMakeString("not rexx )(")'
call try '.method~new("m", "not rexx )(")'
call try 't~unsetMakeString(1)'
call try 'x = t~setMakeString("return 1")'
call try 'x = t~unsetMakeString'
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
<<6 *-* x = 1>>
<<>>>   "1">>
<<7 *-* trace off>>
S:4
S:5
S:6
A:N
A:N
A:N
A:S A:S
[[T?  I? ]               abc]
[abc]
t~setMakeString(.nil) => 93.961
t~setMakeString => 93.901
t~setMakeString("x", "y") => 93.902
t~setMakeString("not rexx )(") => 37.2
.method~new("m", "not rexx )(") => 37.2
t~unsetMakeString(1) => 93.902
x = t~setMakeString("return 1") => 91.999
x = t~unsetMakeString => 91.999
```

## Instance methods

r[class.traceobject.compareto]
`compareTo` compares the receiver with another `StringTable` by their
`NUMBER` entries and answers `-1` when the receiver's is smaller, `0` when
they are equal and `1` when it is larger. It so sorts the objects in the
order they were made. An argument without a `NUMBER` entry counts as smaller
than any receiver, so the answer is `1`; an entry that is not a number raises
`SYNTAX 41.1`. A string raises `SYNTAX 93.924`, and so does a missing
argument.

```rexx
t = .traceobject
a = t~new
b = t~new
say a~number b~number a~compareTo(b) b~compareTo(a) a~compareTo(a)
say a~compareTo(.stringtable~new)
say a~compareTo(.stringtable~new~~put(5, 'NUMBER'))
say a~compareTo(.stringtable~new~~put(2, 'NUMBER'))
call try 'a~compareTo(.stringtable~new~~put("x", "NUMBER"))'
call try 'a~compareTo(1)'
call try 'a~compareTo'
y = .array~of(b, a)~sort
say y[1]~number y[2]~number
exit
try:
  signal on syntax name bad
  interpret 'rr =' arg(1)
  say arg(1) '=>' rr
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 2 -1 1 0
1
-1
-1
a~compareTo(.stringtable~new~~put("x", "NUMBER")) => 41.1
a~compareTo(1) => 93.924
a~compareTo => 93.924
1 2
```

## Deviations

r[class.traceobject.deviation.109]
The argument of `option=` is read by its own characters when it is an instance
of a subclass of `String`, and `REQUEST` and `MAKESTRING` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an instance
`request` with `STRING` or `MAKESTRING`, so an override of either can change
the option. In the example both methods answer `Full`, and the characters are
`Standard`.

```rexx
.traceobject~option = .opt~new('Standard')
say .traceobject~option
.traceobject~option = 'N'
::class opt subclass string
::method makestring
  return 'Full'
::method request
  use arg c
  return 'Full'
```

```output defect=#140
S
```
