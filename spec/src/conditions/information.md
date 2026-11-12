---
element: "Condition information"
kind: concept
ansi: "8.4.1, 9.5.3"
rexxref: "Conditions and Condition Traps > Condition Information; Conditions and Condition Traps > Condition Information > Condition Object"
---

# Condition information

When a trap answers a condition, the interpreter records what it knows about
the condition, and the code that the trap reaches can read it. The record
holds the name of the condition, the kind of the trap, its state, a
description, and an object that some conditions carry. It is read with the
[CONDITION](../builtins/condition.md) builtin, which owns what each option
answers, and as a condition object, a directory with one entry for each
piece of information. The trap also sets the variables `RC` and `SIGL`, and an
error that no trap in the code that raised it answers goes on to the code
that called it. This page specifies the record, the condition object, the two
variables and that journey, and [the traps page](traps.md) specifies when a
trap answers.

## The record

r[cond.information.record]
When a SIGNAL trap or a CALL trap answers a condition, the condition is
recorded for the code that the trap reaches: the label's code for a SIGNAL
trap, and the routine that a CALL trap calls. The record holds the name of the
condition, the instruction of the trap, SIGNAL or CALL, the state of the trap,
the description and the additional object, which
[CONDITION](r:bif.condition.name) answers with its options. [The scope
rule](r:bif.condition.scope) of CONDITION also says which code holds a record
and how long it stays. A condition that no trap answers
records nothing: it is ignored, or it becomes a SYNTAX condition, as
[the untrapped rule](r:cond.index.untrapped) says.

```rexx
address system
x = undefinedvar
'exit 3'
say '['condition('c')']' (condition('o') == .nil)
signal on novalue name n
y = undefinedvar2
n: say condition('c') condition('i') condition('s') condition('d')
```

```output
[] 1
NOVALUE SIGNAL OFF UNDEFINEDVAR2
```

r[cond.information.argument]
A CALL trap calls its routine with one argument, the condition object of the
condition that it answers, so `ARG()` is 1 there and `ARG(1)` is a directory
with the entries of [the entries rule](r:cond.information.object). A SIGNAL
trap passes no argument: `ARG()` at its label is what it is at any other place
of the code, which is the arguments of the routine that holds the label. That
directory is the condition object itself, and none of the copies that
[the object rule of CONDITION](r:bif.condition.object) says `CONDITION('O')`
answers is the argument. The options `C`, `D`, `I` and `A` of `CONDITION` read
the current entries of the object itself, so a change that the routine makes
to its argument shows in what they answer, and in the entries of a copy made
after it. The option `S` reads no entry, and
[the status rule](r:bif.condition.status) says what it answers.

```rexx
address system
call on error name h
call sub 'p', 'q'
signal on novalue name n
call sub2 'p', 'q'
exit
sub:
  'exit 3'
  return
h: say 'handler' arg() arg(1)~class arg(1)~condition arg(1)~rc
  return
sub2:
  y = undefinedvar
n: say 'label' arg() arg(1) condition('c')
```

```output
handler 1 The Directory class ERROR 3
label 2 p NOVALUE
```

```rexx
call on user foo name h
call r
say 'done'
exit
h: o = arg(1)
  say 'same' (condition('o') == o) (condition('o') == condition('o'))
  o~description = 'changed'
  say condition('d') condition('o')~description
  o~extra = 'x'
  say condition('o')~extra
  c = condition('o')
  c~description = 'copy changed'
  say condition('d') o~description
  o~condition = 'USER BAR'
  o~instruction = 'SIGNAL'
  say condition('c') condition('i')
  return
::routine r
  raise user foo description 'raised'
```

```output
same 0 0
changed changed
x
changed changed
USER BAR SIGNAL
done
```

## The condition object

r[cond.information.object]
The condition object is a directory with these entries, whose names are in
capital letters. `CONDITION`, `DESCRIPTION`, `INSTRUCTION` and `ADDITIONAL`
are [the entries that the object rule of CONDITION](r:bif.condition.object)
names. `PACKAGE` is the package that holds the code that raised the condition,
and `PROGRAM` its name. `POSITION` is the number of the line on which the
clause that raised the condition starts, in that code, and a NOTREADY that the
interpreter raised has none. `PROPAGATED` is `1` when the condition left the
level that raised it and `0` otherwise, as
[the propagated rule](r:cond.information.propagated) specifies. `STACKFRAMES`
and `TRACEBACK` are lists with one item for each active level of code at that
time, the main program's included, and the text of a traceback line is not
specified. `ADDITIONAL` is present for a SYNTAX, NOMETHOD, NOSTRING or
NOTREADY that the interpreter raised. A condition that a RAISE instruction
raised has `POSITION`, and has `ADDITIONAL` exactly when the instruction gave
one, whatever the name of the condition, as
[the options rule of RAISE](r:instr.raise.additional) says. The exception is a
SYNTAX, which always has an `ADDITIONAL` array, empty when the instruction
gave none, as [the SYNTAX rule of RAISE](r:instr.raise.syntax) says. SYNTAX
adds `CODE`, the error number with its subcode as `41.1`, `RC`, its major
number, `ERRORTEXT`, the text of the error that the `ERRORTEXT` builtin
answers for `RC`, and `MESSAGE`, the text with the details of this occurrence.
ERROR and FAILURE have `RC`, the return code of the command. The texts of the
messages are not specified, and the entry `RESULT` is not specified here.

```rexx
address system
call on error name h
call on failure name h
call on notready name h
call on halt name h
call on user foo name h
call on user bar name h
signal on novalue name n1
'exit 3'
address nosuch
'exit 3'
y = linein()
call rh
call rf
call rb
x = undefinedvar
n1: call show condition('o')
signal on syntax name n2
x = 1 + 'a'
n2: call show condition('o')
signal on nostring name n3
z = 'a' || .nostr~new
n3: call show condition('o')
signal on nomethod name n4
.nil~foo
n4: call show condition('o')
numeric digits 3
signal on lostdigits name n5
z = 12345 + 1
n5: call show condition('o')
exit
h: call show arg(1)
  return
rh: raise halt return
rf: raise user foo return
rb: raise user bar additional ('x') return
show: procedure
  o = arg(1)
  all = 1
  do k over .array~of('CONDITION', 'DESCRIPTION', 'INSTRUCTION',,
      'PACKAGE', 'PROGRAM', 'PROPAGATED', 'STACKFRAMES', 'TRACEBACK')
    if \o~hasIndex(k) then all = 0
  end
  line = left(o~condition, 10) 'common='all
  do k over .array~of('ADDITIONAL', 'CODE', 'ERRORTEXT', 'MESSAGE',,
      'POSITION', 'RC')
    if o~hasIndex(k) then line = line k
  end
  say line
  return
::class nostr
::method makestring
  return .nil
```

```output
ERROR      common=1 POSITION RC
FAILURE    common=1 POSITION RC
NOTREADY   common=1 ADDITIONAL
HALT       common=1 POSITION
USER FOO   common=1 POSITION
USER BAR   common=1 ADDITIONAL POSITION
NOVALUE    common=1 POSITION
SYNTAX     common=1 ADDITIONAL CODE ERRORTEXT MESSAGE POSITION RC
NOSTRING   common=1 ADDITIONAL POSITION
NOMETHOD   common=1 ADDITIONAL POSITION
LOSTDIGITS common=1 POSITION
```

```rexx
call on notready name h
call r
call r2
call on error name h2
call r3
exit
r: raise notready return
r2: raise notready additional ('z') description 'd' return
r3: raise error 3 additional ('x') return
h: o = arg(1)
  say o~hasindex('POSITION') o~hasindex('ADDITIONAL') '['o~description']'
  return
h2: say arg(1)~condition arg(1)~hasindex('ADDITIONAL') arg(1)~rc
  return
```

```output
1 0 []
1 1 [d]
ERROR 1 3
```

```rexx
signal on syntax name s
call r
s: o = condition('o')
say o~hasindex('ADDITIONAL') o~additional~items
call on user foo name h
call r2
exit
h: say arg(1)~hasindex('ADDITIONAL')
  return
::routine r
  raise syntax 40.1 return
::routine r2
  raise user foo return
```

```output
1 0
0
```

r[cond.information.library]
A level that runs a method of the class library, the code of package `REXX`,
has a line number that is not specified. That holds for `POSITION` of a
condition that was raised there, for the `line` of its
[StackFrame](r:class.stackframe.line), for the number in its
[traceback line](r:class.stackframe.traceline) and for the line that the
[report of an error](r:cond.index.syntax) names. The number of such levels
between the level that sent the message and the level that raised the
condition is not specified either, so the number of items of `STACKFRAMES` and
`TRACEBACK` is not specified where library levels are among them. The code
and subcode of an error that a library method raises are specified, as the
page of the class says; its text is not, as [the object
rule](r:cond.information.object) says for every message.

```rexx
signal on syntax name h
x = .Validate~number('N', 'x')
say 'not reached'
exit
h: say condition('o')~code
```

```output
88.902
```

r[cond.information.object.values]
For a SYNTAX condition, `CODE` is the string `m.n` of the error number and
subcode, `RC` is `m`, `ERRORTEXT` equals what the `ERRORTEXT` builtin answers
for `RC`, `PACKAGE` is the package that `.CONTEXT~PACKAGE` answers in the code
that raised it, `PROGRAM` is the name of that package, and `POSITION` is the
line of the clause, and `PROPAGATED` is `0` when the code that raised it
answers it, as [the propagated rule](r:cond.information.propagated) says for
the other case. `ADDITIONAL` is an array of the
values that the error message names, `a` for the nonnumeric value `'a'` of
error `41.1`. The description of a SYNTAX condition is the null string.

```rexx
signal on syntax name s
x = 1 + 'a'
s: o = condition('o')
say o~code o~rc o~position o~propagated '['o~description']'
say (o~errortext == errortext(o~rc)) (o~program == .context~package~name),
    (o~package == .context~package)
say o~additional~class o~additional~items o~additional~at(1)
say o~stackframes~class o~traceback~class
```

```output
41.1 41 2 0 []
1 1 1
The Array class 1 a
The List class The List class
```

r[cond.information.context]
In a trapped condition's code, `.CONTEXT~CONDITION` answers a new
[directory](r:class.directory.creation) that holds the same entries as
`CONDITION('O')`, and `.nil` when the code holds no record. Every call answers
a new directory, so two answers are not identical and a change to one is seen
by neither the other nor the builtin. The method reads the record that [the
scope rule](r:bif.condition.scope) of CONDITION describes, so the option `R`
of CONDITION clears what the method reads.

```rexx
say '[' || .context~condition || ']' (.context~condition == .nil)
signal on syntax name h
x = 1 + 'a'
h: c = .context~condition
say c~class c~condition c~code (c == condition('o')),
    (c~code == condition('o')~code)
say (c == .context~condition) (c~condition == .context~condition~condition)
c~condition = 'changed'
say condition('c') condition('o')~condition .context~condition~condition
say '['condition('r')']' (.context~condition == .nil) sub()
return
sub: return (.context~condition == .nil)
```

```output
[The NIL object] 1
The Directory class SYNTAX 41.1 0 1
0 1
SYNTAX SYNTAX SYNTAX
[] 1 1
```

## The variables RC and SIGL

r[cond.information.rc]
Before control reaches the code of a trap, the trap sets `RC` for some
conditions. After a SYNTAX condition it is the major number of the error, so
`35` for `35.1`, and it equals the `RC` entry of the object. After ERROR and
FAILURE it is the return code of the command, which [the command
rule](r:instr.command.rc) already sets. Every other condition leaves `RC`
as it was.

```rexx
rc = 'before'
signal on novalue name h1
x = undefinedvar
h1: say 'novalue' rc
signal on syntax name h2
x = 1 + 'a'
h2: say 'syntax' rc condition('o')~code condition('o')~rc
rc = 'again'
signal on syntax name h3
interpret 'say 1 +'
h3: say 'syntax' rc condition('o')~code
rc = 'u'
call on user foo name u
call r
say 'after user' rc
address system
signal on error name h4
'exit 7'
h4: say 'error' rc condition('o')~rc
exit
r: raise user foo return
u: say 'user' rc
  return
```

```output
novalue before
syntax 41 41.1 41
syntax 35 35.1
user u
after user u
error 7 7
```

r[cond.information.sigl]
When a trap answers a condition, `SIGL` is the number of the line on which
starts the clause that was running in the code of the trap, in the level that
the trap belongs to. A level is the main program or a running routine, method
or internal routine. When the condition is raised in the level whose trap
answers, the clause is the one that raised it, and when it is raised deeper,
in a level that has no trap of its own, the clause is the one that called that
level. A clause that spans several lines counts at its first, and the clause
of an INTERPRET is the INTERPRET instruction, not a clause of its text. The
`POSITION` entry is the line of the clause that raised the condition, in the
code that raised it, so it differs from `SIGL` when the condition was raised
in a deeper level that has no trap. The `SIGL` of a CALL trap is set as
[the CALL trap rule](r:cond.traps.call) says.

```rexx
signal on syntax name h
say 'a',
    'b' + 'c',
    'd'
h: say 'multi-line' sigl condition('o')~position
signal on syntax name h2
call inner
h2: say 'inner trap' sigl condition('o')~position
signal on syntax name h3
call outer
h3: say 'outer' sigl condition('o')~position
signal on syntax name h4
say .o~new~go
h4: say 'method' sigl condition('o')~position
signal on syntax name h5
interpret 'say 1;',
  'x = 1 + "a"'
h5: say 'interpret' sigl condition('o')~position
exit
inner:
  x = 1 + 'a'
outer: procedure
  signal off syntax
  x = 1 + 'a'
::class o
::method go
  return 1 + 'x'
```

```output
multi-line 2 2
inner trap 21 21
outer 10 24
method 13 27
1
interpret 16 16
```

## From the code that raised it to its caller

r[cond.information.propagate]
A SYNTAX condition that is raised in a level where no trap answers it, as [the
choice rule](r:cond.traps.choice) decides, ends the clause that raised it, and
then the clause of the level that called this one, so that no clause of the
first level runs on, and is raised again at the clause that made the call, in
the level above: a CALL, a function call, a message term or a message
instruction. A level above whose trap answers ends the chain there, and the
code of the trap runs in that level, with the record and the object of the
original condition: `POSITION` names the line where the error was found, and
`STACKFRAMES` lists the active levels as they were then. A level that does not
answer passes the condition on, up to the main program, where an untrapped
condition ends the program as [the SYNTAX rule](r:cond.index.syntax) says. A
level that starts with every trap `OFF`, a `::ROUTINE` or a method, passes it
on until it switches a trap on, and an internal routine does when its copy of
the trap states has no trap that answers.

```rexx
signal on syntax name top
say 'start'
say a()
say 'not reached'
exit
top: say 'top' condition('o')~code sigl rc condition('o')~position
say 'frames' condition('o')~stackFrames~items
::routine a
  say 'in a'
  return .o~new~m + 1
::class o
::method m
  say 'in m'
  signal on novalue name nope
  x = 1 + 'a'
  say 'not reached'
nope: nop
```

```output
start
in a
in m
top 41.1 3 41 15
frames 3
```

```rexx
signal on syntax name top
say 'start'
say a()
say 'caller goes on'
exit
top: say 'top'
::routine a
  say 'in a'
  signal on syntax name mine
  return .o~new~m + 1
mine: say 'a handles' condition('o')~code sigl
  return 'a done'
::class o
::method m
  say 'in m'
  x = 1 + 'a'
  say 'not reached'
```

```output
start
in a
in m
a handles 41.1 10
a done
caller goes on
```

r[cond.information.propagated]
The `PROPAGATED` entry of the object that the trap sees is `1` when the
condition left the level that raised it, which is the case for a SYNTAX that
reached a caller from a `::ROUTINE`, from a method or from an internal
routine, and after a RAISE with PROPAGATE. It is `0` for a condition that the
level which raised it answers, a SYNTAX or a user condition alike.

```rexx
signal on syntax name t
call r
t: say 'routine' condition('o')~propagated
signal on syntax name v
say .o~new~m
v: say 'method' condition('o')~propagated
signal on syntax name w
x = 1 % 0
w: say 'same level' condition('o')~propagated
exit
::routine r
  x = 1 % 0
::class o
::method m
  x = 1 % 0
```

```output
routine 1
method 1
same level 0
```

r[cond.information.propagated.internal]
An internal routine that switched the trap off passes the condition on in the
same way, and the object that its caller sees has `1`.

```rexx
signal on syntax name v
call iq
v: say 'caller' condition('o')~propagated condition('o')~code
exit
iq: procedure
  signal off syntax
  x = 1 % 0
```

```output
caller 1 42.3
```

```rexx
signal on syntax name v
call i1
v: say 'caller' condition('o')~propagated condition('o')~code
exit
i1: procedure
  signal off syntax
  call i2
  return
i2: procedure
  signal off syntax
  x = 1 % 0
```

```output
caller 1 42.3
```

r[cond.information.propagate.only]
Only a SYNTAX condition is passed on. Any other condition is answered or
ignored in the level that raised it, by the trap states that level has, so a
trap that the level above set for itself does not see it. An internal routine
has a copy of the trap states of its caller, as [the scope
rule](r:cond.traps.scope) says, so a trap that its caller switched on does
answer a condition raised in the routine, and it does so in the routine, not
in the caller. A HALT or NOMETHOD that nothing answers becomes a SYNTAX
condition first, as [the untrapped rule](r:cond.index.untrapped) says, and
that one is passed on.

```rexx
signal on novalue name nv
address system
call on error name er
say 'main' r()
exit
nv: say 'nv in main' sigl
er: say 'er in main'; return
::routine r
  x = unsetvar
  'exit 3'
  return 'r done'
```

```output
main r done
```

```rexx
signal on novalue name nv
address system
call on error name er
say 'main' r()
say 'after'
exit
nv: say 'nv' sigl; return 'from nv'
er: say 'er' sigl rc; return
r:
  'exit 3'
  say 'back in r'
  x = unsetvar
  say 'not reached'
  return 'r done'
```

```output
er 10 3
back in r
nv 12
main from nv
after
```
