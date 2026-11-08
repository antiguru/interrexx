---
element: "NOTREADY"
kind: concept
ansi: "8.4.1"
rexxref: "Conditions and Condition Traps; Conditions and Condition Traps > Condition Information > Descriptive Strings; Conditions and Condition Traps > Condition Information > Additional Object Information; Conditions and Condition Traps > Action Taken when a Condition Is Trapped"
---

# NOTREADY

NOTREADY is the condition of a stream operation that could not do what it was
asked to. A read that reaches the end of the data, a name that cannot be
opened and a write that fails all leave the program with an answer that is
shorter than it expected, and the condition is how a program finds out without
testing every call. Unlike an error it does not stop anything: a stream call
answers as its page says whether or not a trap exists. The builtin pages and
[the NOTREADY rule of the builtins](r:bif.index.stream.notready) say which
calls raise it and when, and this page specifies what the condition holds,
which traps answer it and how far it reaches.

## What it holds

r[cond.notready.information]
The description of a NOTREADY condition is the name of the stream, spelled as
[the stream rule](r:bif.index.stream.notready) says, and its `ADDITIONAL`
entry is the stream object, whose string value is that name. The condition
object has the entries that every condition has and `ADDITIONAL`, and no
`POSITION`, `RC` or `CODE`, as [the entries rule](r:cond.information.object)
lists them. The whole clause that held the call has finished
by the time a CALL trap runs, as [the CALL trap rule](r:cond.traps.call) says.

```rexx
f = 'spec-notready-info.tmp'
call sysfiledelete f
call lineout f, 'one'
call lineout f
say linein(f)
call on notready name h
x = linein(f)
say 'x is [' || x || ']'
call sysfiledelete f
exit
h: o = condition('o')
  say condition('c') '[' || condition('d') || ']' condition('i') sigl,
      condition('s')
  say o~hasindex('ADDITIONAL') o~hasindex('POSITION') o~hasindex('CODE'),
      o~hasindex('RC') '[' || o~additional || ']'
  return
```

```output
one
NOTREADY [spec-notready-info.tmp] CALL 7 DELAY
1 0 0 0 [spec-notready-info.tmp]
x is []
```

## Which traps answer it

r[cond.notready.call]
NOTREADY is answered by CALL ON NOTREADY, by SIGNAL ON NOTREADY and by the ANY
traps of both kinds. A CALL trap lets the stream call finish, and the
clause that held the call finishes too. The routine is called before the next
clause, with the state `DELAY` in the routine and `SIGL` the line of that
clause, as [the CALL trap rule](r:cond.traps.call) says. The clause has then
assigned what its page says the stream call answers, so the routine sees the
assignment already made and the clause is not resumed after the routine.

```rexx
f = 'spec-notready-call.tmp'
call sysfiledelete f
call lineout f, 'only line'
call lineout f
call on notready name h
a = linein(f)
b = 'old'
b = linein(f) || mark('second operand')
say 'a=' || a 'b=' || b
call off notready
call on any name h2
c = linein(f)
say 'c=' || c
call sysfiledelete f
exit
mark: say 'evaluating' arg(1)
  return '+'
h: say 'h' sigl condition('c') condition('s') 'b=[' || b || ']'
  return
h2: say 'h2' sigl condition('c')
  return
```

```output
evaluating second operand
h 8 NOTREADY DELAY b=[+]
a=only line b=+
h2 12 NOTREADY
c=
```

r[cond.notready.signal]
A SIGNAL trap for NOTREADY ends the clause that made the stream call and goes
to the label, ending the open DO and SELECT constructs as [the SIGNAL trap
rule](r:cond.traps.signal) says, so the clause makes none of its assignments.
`SIGL` is the line of that clause and the trap is `OFF` at the label. A SIGNAL
trap for ANY answers it too, and the name stays NOTREADY.

```rexx
f = 'spec-notready-sig.tmp'
call sysfiledelete f
call lineout f, 'only line'
call lineout f
x = 'kept'
signal on notready name n
x = linein(f)
x = linein(f)
say 'not reached'
n: say 'trapped' x condition('c') sigl condition('s')
signal on any name a
x = linein(f)
a: say 'ANY' condition('c') sigl
call sysfiledelete f
```

```output
trapped only line NOTREADY 8 OFF
ANY NOTREADY 12
```

## How far it reaches

r[cond.notready.scope]
NOTREADY is raised in the code that made the stream call, and goes no further.
A trap in the caller of a routine does not answer a NOTREADY that a stream
call in the routine raised, as [the propagation
rule](r:cond.information.propagate.only) says, and the routine's own traps do.

```rexx
f = 'spec-notready-scope.tmp'
call sysfiledelete f
call lineout f, 'only line'
call lineout f
call on notready name h
call rd
say 'back from rd'
x = linein(f)
say 'main read done'
call sysfiledelete f
exit
h: say 'main handler' sigl '[' || condition('d') || ']'
  return
::routine rd
  call on notready name inner
  x = linein('spec-notready-scope.tmp')
  y = linein('spec-notready-scope.tmp')
  say 'rd done'
  return
inner: say 'rd handler' sigl
  return
```

```output
rd handler 17
rd done
back from rd
main handler 8 [spec-notready-scope.tmp]
main read done
```

## When nothing traps it

r[cond.notready.untrapped]
A NOTREADY that neither its own trap nor the ANY trap answers is ignored, as
[the untrapped rule](r:cond.index.untrapped) says, and the stream call answers
as its page says. The directive `::OPTIONS` can turn an untrapped NOTREADY
into a SYNTAX condition, and the page of the directive specifies that.

```rexx
f = 'spec-notready-none.tmp'
call sysfiledelete f
call lineout f, 'only line'
call lineout f
say linein(f) '[' || linein(f) || ']' '[' || linein(f) || ']'
say lines(f)
call sysfiledelete f
```

```output
only line [] []
0
```
