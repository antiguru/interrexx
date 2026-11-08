---
element: "NOMETHOD"
kind: concept
rexxref: "Conditions and Condition Traps; Conditions and Condition Traps > Condition Information > Descriptive Strings; Conditions and Condition Traps > Condition Information > Additional Object Information; Conditions and Condition Traps > Action Taken when a Condition Is Trapped"
---

# NOMETHOD

NOMETHOD is the condition of a message that reaches an object with nothing to
run for it. Sending a message is the way Rexx objects are used, and a message
whose name is misspelled or meant for another class has no method to find. A
program traps NOMETHOD to learn which message was lost and to which object it
went. This page specifies when the condition is raised, what it holds, which
traps answer it and what becomes of it when none does.

## When it is raised

r[cond.nomethod.raise]
NOMETHOD is raised when a message is sent to an object, the object has no
method of that name that the sender may run, and the object has no `UNKNOWN`
method that takes the message instead. A private method that is sent from
outside its class is a method the sender may not run, so the message raises
NOMETHOD, with the name of the method, and an `UNKNOWN` method takes it as it
takes any lost message. The message may be written with `~`, with `~~`, or
with a scope override such as `~name:super`, as [the information
rule](r:cond.nomethod.information) shows, and it may be sent by the `SEND`
method of an object. The arguments of the message are evaluated before the
lookup, so the clause has evaluated them when the condition is raised. The
condition is raised in the code that sent the message, and the clause ends
there when a SIGNAL trap answers. The exception is a message that FORWARD
sends without CONTINUE, which [the FORWARD rule](r:instr.forward.end) raises
in the caller of the forwarding method. A class with an `UNKNOWN` method raises
nothing, because that method receives the message. The `UNKNOWN` method and
the way a class defines methods belong to the classes chapter.

```rexx
do k = 1 to 5
  call probe k
end
exit
::routine probe
  use arg k
  signal on nomethod name n
  select
    when k = 1 then x = .a~new~foo
    when k = 2 then x = .a~new~nothere(arg1())
    when k = 3 then x = 'abc'~~nothere
    when k = 4 then x = .a~new~send('Not Here')
    when k = 5 then x = .b~new~nothere
  end
  say k 'no condition'
  return
n: say k 'NOMETHOD' condition('d')
  return
::routine arg1
  say 'argument evaluated'
  return 1
::class a
::method foo
  return 1
::class b
::method unknown
  return 'unknown took it'
```

```output
1 no condition
argument evaluated
2 NOMETHOD NOTHERE
3 NOMETHOD NOTHERE
4 NOMETHOD NOT HERE
5 no condition
```

r[cond.nomethod.information]
The description of a NOMETHOD condition is the name of the message in upper
case, and its `ADDITIONAL` entry is the object that received the message. The
condition object has the entries that every condition has, `ADDITIONAL` and
`POSITION`, and neither `RC` nor `CODE`, as [the entries
rule](r:cond.information.object) lists them. For a message with a scope
override the receiver is the object that the message was sent to, not its
class.

```rexx
signal on nomethod name n
x = 'abc'~lost(1, 2)
n: o = condition('o')
say condition('c') '['condition('d')']' condition('i') sigl condition('s')
say o~hasindex('ADDITIONAL') o~hasindex('POSITION') o~hasindex('CODE'),
    o~hasindex('RC') '['o~additional']'
say .a~new~go
signal on nomethod name n3
x = .a~new~'Mixed case'
n3: say '['condition('d')']' condition('o')~additional~class
::class a
::method go
  signal on nomethod name inner
  x = self~missing:super
  inner: say 'in go' '['condition('d')']' condition('o')~additional~class
  return 1
```

```output
NOMETHOD [LOST] SIGNAL 2 OFF
1 1 0 0 [abc]
in go [MISSING] The A class
1
[MIXED CASE] The A class
```

## Which traps answer it

r[cond.nomethod.trap]
NOMETHOD is answered by SIGNAL ON NOMETHOD and by SIGNAL ON ANY, and by no
CALL trap: CALL ON NOMETHOD is `SYNTAX 25.1`, as [the CALL
rule](r:instr.call.on) says, and a CALL ON ANY trap does not see it, as [the
choice rule](r:cond.traps.choice) says. The trap is `OFF` at its label, as
[the SIGNAL trap rule](r:cond.traps.signal) says, so a second lost message is
untrapped until the program enables the trap again.

```rexx
signal on syntax name s
interpret 'call on nomethod name h'
s: say condition('c') condition('o')~code
signal on any name a
x = .nil~nothere
a: say 'ANY' condition('c') condition('d') condition('s')
signal on nomethod name n
x = .nil~first
n: say 'first' condition('d') condition('s')
signal on syntax name s2
x = .nil~second
s2: say 'second' condition('c') condition('o')~code
exit
h: say 'not called'
  return
```

```output
SYNTAX 25.1
ANY NOMETHOD NOTHERE OFF
first FIRST OFF
second SYNTAX 97.1
```

## When nothing traps it

r[cond.nomethod.untrapped]
A NOMETHOD that neither its own trap nor the ANY trap answers is not ignored.
It becomes the condition `SYNTAX 97.1` raised at the same clause, as [the
untrapped rule](r:cond.index.untrapped) says, and a SYNTAX trap answers it
under the name SYNTAX. A message to a private method that the sender may not
run becomes `SYNTAX 97.2` instead. Because that is a SYNTAX condition, it
leaves the code that sent the message and goes on to its caller, as [the
propagation rule](r:cond.information.propagate) says. So a NOMETHOD trap in
the caller of a method does not answer a message that the method lost, and a
SYNTAX trap in the caller does. Without a trap the program ends as [the SYNTAX
rule](r:cond.index.syntax) says.

```rexx
signal on nomethod name nm
signal on syntax name sy
.a~new~go
say 'not reached'
nm: say 'NOMETHOD in main' condition('d')
exit
sy: o = condition('o')
say 'SYNTAX in main' o~code o~rc '['condition('d')']'
::class a
::method go
  x = self~nothere
  say 'not reached'
```

```output
SYNTAX in main 97.1 97 []
```

```rexx
signal on syntax name s
x = .a~new~hidden
say 'not reached'
s: say condition('c') condition('o')~code
say 'before'
x = .a~new~hidden
say 'not reached'
::class a
::method hidden private
  return 'ran'
```

```output rc=159
SYNTAX 97.2
before
```

```rexx
say 'before'
x = 'abc'~nothere
say 'not reached'
```

```output rc=159
before
```
