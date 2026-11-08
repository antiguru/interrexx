---
element: "NOTREADY"
kind: concept
ansi: "8.4.1"
rexxref: "Conditions and Condition Traps > Action Taken when a Condition Is Trapped"
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
lists them. The call that raised it has answered as its own page says by the
time a CALL trap runs.

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
traps of both kinds. A CALL trap lets the stream call finish and calls the
routine afterwards, and the clause that held the call goes on when the routine
returns, with the state `DELAY` in the routine and `SIGL` the line of that
clause, as [the CALL trap rule](r:cond.traps.call) says. The call has then
answered what its page says, so the clause sees the same answer as it would
without a trap.

```rexx
f = 'spec-notready-call.tmp'
call sysfiledelete f
call lineout f, 'only line'
call lineout f
call on notready name h
a = linein(f)
b = linein(f)
say 'a=' || a 'b=' || b
call off notready
call on any name h2
c = linein(f)
say 'c=' || c
call sysfiledelete f
exit
h: say 'h' sigl condition('c') condition('s')
  return
h2: say 'h2' sigl condition('c')
  return
```

```output
h 7 NOTREADY DELAY
a=only line b=
h2 11 NOTREADY
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
The state of a stream belongs to the stream and not to the code that used it,
so a read in one routine can leave the stream at its end for the next one.

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
