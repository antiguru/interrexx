---
element: "StackFrame"
kind: class
rexxref: "Builtin Classes > Utility Classes > StackFrame Class; Builtin Classes > Utility Classes > StackFrame Class > arguments; Builtin Classes > Utility Classes > StackFrame Class > context; Builtin Classes > Utility Classes > StackFrame Class > executable; Builtin Classes > Utility Classes > StackFrame Class > invocation; Builtin Classes > Utility Classes > StackFrame Class > line; Builtin Classes > Utility Classes > StackFrame Class > makeString; Builtin Classes > Utility Classes > StackFrame Class > name; Builtin Classes > Utility Classes > StackFrame Class > string; Builtin Classes > Utility Classes > StackFrame Class > target; Builtin Classes > Utility Classes > StackFrame Class > traceLine; Builtin Classes > Utility Classes > StackFrame Class > type"
deviations: [0, 66]
---

# StackFrame

A `StackFrame` describes one level of the call stack of an activity: the main
part of a program, a routine, a method, an internal routine or the text of an
INTERPRET that was running when the frame was made. A program gets frames from
[the `stackFrames` method of `RexxContext`](r:class.rexxcontext.stackframes)
and from [the condition object](r:cond.information.object); it never makes
one. A frame answers where its level was, what it was called with and what
runs it, and it keeps answering after the level has ended.

The CALL, INTERPRET and SIGNAL instructions, the conditions and TRACE are
specified in their own chapters; this page links their rules. Two Deviations
touch frames, 0 for the indentation of a `traceLine` and 66 for the name of
a frame, and the last section states them.

## The class

r[class.stackframe.creation]
`StackFrame` is a class whose only superclass is `Object`, and it inherits no
mixin class ([the hierarchy rule](r:class.index.hierarchy)). Its instance side
defines `arguments`, `context`, `executable`, `invocation`, `line`,
`makeString`, `name`, `string`, `target`, `traceLine` and `type`, and no other
method. The class method `new` raises `SYNTAX 93.967` whatever the arguments
are, so no program makes a frame. No instance method takes an argument, and one
sent with an argument raises `SYNTAX 93.902`, `copy` included.

```rexx
s = .stackframe~methods(.nil)
names = .array~new
do while s~available
  names~append(s~index)
  s~next
end
names = names~sort
do i = 1 to names~items by 4
  say names~section(i, min(4, names~items - i + 1))~makeString('l', ' ')
end
say .stackframe~superclasses~items .stackframe~superclass~id
call try '.stackframe~new'
call try '.stackframe~new(1)'
call try '.context~stackFrames[1]~type(1)'
call try '.context~stackFrames[1]~copy(1)'
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
ARGUMENTS CONTEXT EXECUTABLE INVOCATION
LINE MAKESTRING NAME STRING
TARGET TRACELINE TYPE
1 Object
.stackframe~new => 93.967
.stackframe~new(1) => 93.967
.context~stackFrames[1]~type(1) => 93.902
.context~stackFrames[1]~copy(1) => 93.902
```

r[class.stackframe.copy]
`copy` answers a new frame that is not the receiver and has the same `type`,
`name` and `line`. Every other method of the copy answers what the receiver's
answers, the same objects included: the `arguments`, the `context` and the
`executable` of the two are the same objects.

```rexx
call r 'a', 'b'
exit
r:
  f = .context~stackFrames[1]
  g = f~copy
  say g~class~id (g == f) g~type g~name g~line (g~arguments == f~arguments)
  say (g~context == f~context) (g~executable == f~executable),
      (g~traceLine == f~traceLine) (g~invocation == f~invocation)
  return
```

```output
StackFrame 0 INTERNALCALL R 4 1
1 1 1 1
```

r[class.stackframe.sources]
A frame comes from the array that `stackFrames` of a context answers or from
the list that a condition's `STACKFRAMES` entry holds. In both the first frame
is the level that was running, or that raised the condition, and each next one
is the level that called the one before, with the main part of the program
last. The array of a context has the indexes `1` and up and the list of a
condition has the indexes `0` and up. The levels are those of the activity
that asked, so the list of a message that another activity runs ends with the
method that the message started. A condition answers the same list each time.

```rexx
signal on syntax name h
call outer
exit
outer: procedure
  .o~new~m
  return
h:
  fs = condition('o')~stackFrames
  say fs~class~id fs~first fs~last fs~items (fs == condition('o')~stackFrames)
  do i over fs~allIndexes
    say i fs[i]~type fs[i]~line
  end
  .o~new~start('act')~result
exit
::class o
::method m
  say 1 + 'a'
::method act
  fs = .context~stackFrames
  say fs~class~id fs~first fs~last fs[1]~type fs[1]~name
  return 1
```

```output
List 0 2 3 1
0 METHOD 17
1 INTERNALCALL 5
2 PROGRAM 2
Array 1 1 METHOD ACT
```

## Methods

r[class.stackframe.type]
`type` answers one of `PROGRAM` for the main part of a program, `ROUTINE`
for a routine, `METHOD` for a method, `INTERNALCALL` for an internal routine,
`INTERPRET` for the text of an INTERPRET and `COMPILE` for text that was being
translated when a SYNTAX condition was raised. A frame of type `COMPILE` is
the first of that condition's list. A built-in method that runs a program's
code, such as `send`, is a level of type `METHOD`.

```rexx
call ic
exit
ic:
  .o~new~send('go')
  return
::routine rt
  call dump
::class o
::method go
  interpret 'call rt'
::routine dump
  do f over .context~stackFrames
    say f~type
  end
```

```output
ROUTINE
ROUTINE
INTERPRET
METHOD
METHOD
INTERNALCALL
PROGRAM
```

```rexx
signal on syntax name h1
r = .routine~new('badone', 'x = = 1')
h1: call first condition('o')
signal on syntax name h2
interpret 'x = = 1'
h2: call first condition('o')
exit
first:
  fs = arg(1)~stackFrames
  say fs~items fs~firstItem~type fs~firstItem~line fs~lastItem~type
  return
```

```output
3 COMPILE 1 PROGRAM
2 COMPILE 5 PROGRAM
```

r[class.stackframe.name]
`name` answers the name of the level. For `PROGRAM` it is the name of the
package ([the name rule of `Package`](r:class.package.name)). For `ROUTINE` it
is the name by which the routine was called: for a `::ROUTINE` in upper case,
for a routine that `Routine~new` made the name it was given. For `METHOD` it
is the name of the method in upper case, which is the message name, and for
`INTERNALCALL` the label in upper case. For `INTERPRET` it is the null string.
A frame of type `COMPILE` has the name of the text that was being translated;
for the text of an INTERPRET that is the name of the package.

```rexx
call outer
.routine~new('anon', 'call dump')~call
.o~new~m
signal on syntax name h1
r = .routine~new('badone', 'x = = 1')
h1: f = condition('o')~stackFrames~firstItem
say f~type '['f~name']'
signal on syntax name h2
interpret 'x = = 1'
h2: f = condition('o')~stackFrames~firstItem
say f~type (f~name == .context~package~name)
exit
outer:
  call rt
  return
::routine rt
  call dump
::class o
::method m
  interpret 'call dump'
::routine dump
  do f over .context~stackFrames
    if f~type == 'PROGRAM' then say f~type (f~name == .context~package~name)
    else say f~type '['f~name']'
  end
  say '--'
```

```output
ROUTINE [DUMP]
ROUTINE [RT]
INTERNALCALL [OUTER]
PROGRAM 1
--
ROUTINE [DUMP]
ROUTINE [anon]
METHOD [CALL]
PROGRAM 1
--
ROUTINE [DUMP]
INTERPRET []
METHOD [M]
PROGRAM 1
--
COMPILE [badone]
COMPILE 1
```

r[class.stackframe.line]
`line` answers the number of the line on which the clause that the level was
running starts, and for an INTERPRET level the number of the line of the
clause that holds the text, as [the line of a trace line](r:instr.trace.line)
is. A frame is a snapshot of the moment it was made: the running code goes on,
and the line of the frame's context, [asked with
`line`](r:class.rexxcontext.line), moves with it. `line` answers `.nil` for a
built-in method. The number that it answers for a method of the class library
is not specified, as [the library rule](r:cond.information.library) says.

```rexx
call r
exit
r:
  f = .context~stackFrames[1]
  say f~line .context~line f~context~line
  say f~line .context~line f~context~line
  g = .context~stackFrames[2]
  say g~line g~context~line
  .o~new~send('m')
  return
::class o
::method m
  say .context~stackFrames[2]~line~isNil .context~stackFrames[2]~type
  interpret 'say .context~stackFrames[1]~type .context~stackFrames[1]~line'
```

```output
4 5 5
4 6 6
1 1
1 METHOD
INTERPRET 14
```

r[class.stackframe.traceline]
`traceLine`, `string` and `makeString` all answer the same string, which is the
[trace line](r:instr.trace.line) of the clause that the level was running: the
number of its line in six columns, `*-*` and the clause, indented as [the
indentation rule](r:instr.trace.indent) says, which counts one level for each
active internal call. [Deviation 0](r:class.stackframe.deviation.0) covers the
indentation that a program reads back. The string of a built-in method's frame
is seven blanks, `*-*`, a blank and `Compiled method "NAME" with scope
"Class".`, with the method's name in upper case and the id of the class that
defines it.
The number in the line of a method of the class library, and its
indentation, are not specified, as [the library rule](r:cond.information.library)
says.

```rexx
call show
do i = 1 to 1
  call show
end
call outer
call show 'multi',
   'line'
exit
outer:
  call show
  return
show:
  f = .context~stackFrames[2]
  say '['f~traceLine']' (f~string == f~traceLine),
      (f~makeString == f~traceLine)
  return
```

```output
[     1 *-* call show] 1 1
[     3 *-*   call show] 1 1
[    10 *-*   call show] 1 1
[     6 *-* call show 'multi',   'line'] 1 1
```

```rexx
.o~new~send('m')
.o~new~sendWith('m', .array~new)
.message~new(.o~new, 'm')~send
exit
::class o
::method m
  say '['.context~stackFrames[2]~traceLine']'
```

```output
[       *-* Compiled method "SEND" with scope "Object".]
[       *-* Compiled method "SENDWITH" with scope "Object".]
[       *-* Compiled method "SEND" with scope "Message".]
```

r[class.stackframe.arguments]
`arguments` answers an array of the arguments that the level was given, as
[ARG with the option `A`](r:bif.arg.option.array) answers for it: an omitted
argument is a place with no item. The frame keeps the array, so asking again
answers the same object, but it is a copy: changing it changes nothing the
level sees and nothing that another frame of the level answers. A built-in
method's frame holds the arguments of its message, and a frame of the main
part of a program the arguments the program was started with.

```rexx
call r 'a1', , 'a3'
exit
r:
  f = .context~stackFrames[1]
  a = f~arguments
  say a~class~id a~size a~items (a == f~arguments)
  a[1] = 'changed'
  a~append('x')
  say f~arguments~items arg(1) arg() (a == .context~stackFrames[1]~arguments),
      .context~stackFrames[1]~arguments[1]
  .o~new~send('m', 7, 8)
  return
::class o
::method m
  fs = .context~stackFrames
  say fs[1]~arguments~items fs[2]~arguments~items fs[2]~arguments[1],
      fs[2]~name fs[3]~arguments~items
```

```output
Array 3 2 1
3 a1 3 0 a1
2 3 m SEND 2
```

r[class.stackframe.target]
`target` answers the object that received the message for a frame of type
`METHOD`, a built-in method's frame included, and `.nil` for the frame of any
other type.

```rexx
x = .o~new
x~go
.o~classgo
exit
::class o
::method go
  call dump
::method classgo class
  call dump
::routine dump
  do f over .context~stackFrames
    if f~type == 'METHOD' then say f~name f~target~string
    else say f~type f~target~isNil
  end
  say '--'
```

```output
ROUTINE 1
GO an O
PROGRAM 1
--
ROUTINE 1
CLASSGO The O class
PROGRAM 1
--
```

r[class.stackframe.executable]
`executable` answers the object that holds the code of the level: a `Method`
for a frame of type `METHOD` and a `Routine` for `ROUTINE`, `PROGRAM` and
`INTERNALCALL`, the last two being parts of the program's own routine. A frame
of type `INTERPRET` has the executable of the level that holds the text. A
frame of type `COMPILE` has no executable and answers `.nil`.

```rexx
call ic
exit
ic:
  call rt
  return
::routine rt
  .o~new~m
::class o
::method m
  interpret 'call dump'
::routine dump
  fs = .context~stackFrames
  do f over fs
    say f~type f~executable~class~id
  end
  say (fs[1]~executable == .context~package~findRoutine('DUMP')),
      (fs[3]~executable == fs[2]~executable),
      (fs[5]~executable == fs[6]~executable)
```

```output
ROUTINE Routine
INTERPRET Method
METHOD Method
ROUTINE Routine
INTERNALCALL Routine
PROGRAM Routine
1 1 1
```

```rexx
signal on syntax name h1
r = .routine~new('badone', 'x = = 1')
h1: call show condition('o')~stackFrames~firstItem
signal on syntax name h2
interpret 'x = = 1'
h2: call show condition('o')~stackFrames~firstItem
exit
show:
  f = arg(1)
  say f~type f~executable~isNil
  return
```

```output
COMPILE 1
COMPILE 1
```

r[class.stackframe.context]
`context` answers the [`RexxContext`](rexxcontext.md) of the level: for the
first frame of a context's own `stackFrames` it is that context itself, and
asking one frame again answers the same object. A built-in method's frame and a
frame of type `COMPILE` have no context and answer `.nil` ([the example of
`invocation`](r:class.stackframe.invocation) shows the second). Once the level
has ended, its context answers no message, as [the active
rule](r:class.rexxcontext.active) says, while the frame itself goes on
answering.

```rexx
call outer 'p', 'q'
exit
outer: procedure
  outvar = 'outer value'
  call inner
  return
inner: procedure
  fs = .context~stackFrames
  say fs~items (fs[1]~context == .context) (fs[2]~context == fs[2]~context),
      (fs[2]~context == .context)
  say fs[2]~context~name fs[2]~context~args[2],
      fs[2]~context~variables['OUTVAR']
  say fs[3]~context~variables~hasIndex('OUTVAR')
  .o~new~send('m')
  call keep
  c = kept~context
  say kept~type kept~name
  signal on syntax name dead
  say c~name
  exit
dead: say 'context of a finished level' condition('o')~code
  return
keep: procedure expose kept
  kept = .context~stackFrames[1]
  return
::class o
::method m
  say .context~stackFrames[2]~context~isNil
```

```output
3 1 1 0
OUTER q outer value
0
1
INTERNALCALL KEEP
context of a finished level 98.981
```

r[class.stackframe.invocation]
`invocation` answers the number of the invocation of the level, which is the
number that [the `invocation` of its context](r:class.rexxcontext.numbers)
answers, and `.nil` for a built-in method's frame and for a frame of type
`COMPILE`. Making the array of frames numbers the levels that have no number
yet, the innermost first.

```rexx
call outer
exit
outer: procedure
  call inner
  return
inner: procedure
  fs = .context~stackFrames
  say fs[1]~invocation fs[2]~invocation fs[3]~invocation,
      (fs[1]~invocation == .context~invocation)
  .o~new~send('m')
  return
::class o
::method m
  say .context~stackFrames[2]~invocation~isNil
```

```output
1 2 3 1
1
```

```rexx
signal on syntax name h1
r = .routine~new('badone', 'x = = 1')
h1: call show condition('o')~stackFrames~firstItem
signal on syntax name h2
interpret 'x = = 1'
h2: call show condition('o')~stackFrames~firstItem
exit
show:
  f = arg(1)
  say f~type f~context~isNil f~invocation~isNil
  return
```

```output
COMPILE 1 1
COMPILE 1 1
```

## Deviations

r[class.stackframe.deviation.0]
The `traceLine` of a frame is indented by block structure alone, as [the
indentation rule](r:instr.trace.deviation.0) says of trace lines. ooRexx
drops one level too many when a loop ends because its test failed, so the
line of a clause after the loop stands one level short, as in the first
line of the example below where ooRexx shows `*-* call show` at the margin.

```rexx
do
  do i = 1 to 2
    nop
  end
  call show
end
call show
exit
show:
  f = .context~stackFrames[2]
  say '['f~traceLine']'
  return
```

```output
[     5 *-*   call show]
[     7 *-* call show]
```

r[class.stackframe.deviation.66]
A SYNTAX 16.1 that arises in the text of an INTERPRET, because the label of a
trap that `SIGNAL ON` set does not exist, has one frame, of type `PROGRAM`,
and its name is the name of the package, as [the INTERPRET
rule](r:instr.interpret.deviation.66) states. ooRexx names that frame with the
null string.

```rexx
signal on syntax name e
signal on novalue name nolabel
interpret 'a = undefinedvar'
e: c = condition('o')
f = c~stackFrames~firstItem
say c~code c~stackFrames~items f~type (f~name == .context~package~name)
```

```output
16.1 1 PROGRAM 1
```
