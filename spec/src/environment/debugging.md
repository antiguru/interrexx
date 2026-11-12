---
element: "Debugging"
kind: concept
deviations: [150, 151, 152]
rexxref: "Debugging Aids; Debugging Aids > Interactive Debugging of Programs; Debugging Aids > RXTRACE Variable"
---

# Debugging

Interactive tracing lets a person, or a program that stands in for one, step
through another program: after each clause that is traced the interpreter
stops, reads a line, and runs it or goes on. This page specifies how that
state is entered and left, when the interpreter stops, what it reads and from
where, what each kind of line does, and where its own lines are written. It
leaves to other pages what they already own: the settings and the question
mark prefix are those of [the TRACE function](r:bif.trace.prefix), the format
of the lines that show clauses is that of [the TRACE
instruction](r:instr.trace.line), and the objects `.debuginput` and
`.traceoutput` are among [the environment objects](r:env.objects.local).

> **Note.** The examples feed the lines that a person would type from objects
> of their own, which a program installs as the debug input, and never from
> the terminal. The class `in` that most of them define answers its lines one
> at a time and then the empty line, and writes each line it answers in
> brackets, so that the output shows when the interpreter stops. The class
> `show` that most of them define receives the lines of the trace; it leaves
> out the line that holds the program's source string, which names the host's
> file. The line that [the opening rule](r:env.debugging.opening) describes is
> checked in its own example.

## Starting

r[env.debugging.start]
Interactive tracing starts when a [TRACE
instruction](r:instr.trace.form.interactive) or a call of the [TRACE
function](r:bif.trace.prefix) gives a setting that turns it on, and it starts
with whatever letter that setting has. The clause that turns it on is traced
under the old setting, as [the effect rule](r:instr.trace.effect) says, so it
is shown only when the old setting shows it. A TRACE instruction, and an
assignment that calls the function, are followed by no stop: the first stop
follows the next clause. Any other clause that calls the function and turns it
on, a CALL, a SAY, an IF or a PARSE VALUE for instance, is followed by a stop
at once, after the lines it caused; [the notice](r:env.debugging.notice) comes
before that stop and [the source line](r:env.debugging.opening) before the
next clause that is shown. How a package asks for it with `::OPTIONS TRACE` is
the business of [the directive](r:dir.options.trace); the environment variable
`RXTRACE` is described [below](r:env.debugging.rxtrace). In the first example
the first clause that is shown is `say 'one'`, the one after `trace ?a`, and
the stop follows it, after what it wrote. In the second, the function turns it
on from inside an assignment, and the stop follows the clause after that one.
In the third, a CALL of the function turns it on, and the stop follows the
CALL at once, so that `say 'b'` is shown only after it. In the fourth, the old
setting is `R`, so the clause that turns interactive tracing on is shown.

```rexx
.debuginput~destination(.in~new(.array~of('', '')))
.traceoutput~destination(.show~new)
say 'before'
trace ?a
say 'one'
say 'two'
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
before
     5 *-* say 'one'
one
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: ]
     6 *-* say 'two'
two
[read 2: ]
     7 *-* exit
```

```rexx
.debuginput~destination(.in~new(.array~of('say "typed"', '')))
.traceoutput~destination(.show~new)
say 'a'
x = trace('?r')
say 'b'
call trace 'o'
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
a
     5 *-* say 'b'
       >>>   "b"
b
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: say "typed"]
typed
[read 2: ]
     6 *-* call trace 'o'
```

```rexx
.debuginput~destination(.in~new(.array~of('say "t1"', '', '', '', '')))
.traceoutput~destination(.show~new)
say 'a'
call trace '?r'
say 'b'
say 'c'
call trace 'o'
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output defect=#222
a
       >>>   "N"
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: say "t1"]
t1
[read 2: ]
     5 *-* say 'b'
       >>>   "b"
b
[read 3: ]
     6 *-* say 'c'
       >>>   "c"
c
[read 4: ]
     7 *-* call trace 'o'
```

```rexx
.debuginput~destination(.in~new(.array~of('', '')))
.traceoutput~destination(.show~new)
trace r
trace ?r
say 'b'
call trace 'o'
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     4 *-* trace ?r
     5 *-* say 'b'
       >>>   "b"
b
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: ]
     6 *-* call trace 'o'
```

r[env.debugging.opening]
When interactive tracing is turned on in code that has not yet written it, a
line is written before the next clause that is shown: seven blanks, `+++`, a
blank, and in double quotation marks the string that [PARSE
SOURCE](r:instr.parse.source) gives for the program. The code is the main code
of the program or a run of an internal routine; a routine starts as having
written the line when its caller has, and what it writes is not handed back,
so a routine that turns interactive tracing on before its caller ever has
writes it, and so does the caller later. In the first example the class `show`
replaces the source line by a marker, after checking that it has the form and
the content the rule gives. In the second the routine and then the caller
write it, and the second call of the routine, made after the caller had
written it, does not; the class `show` there also leaves out the notice.

```rexx
parse source src
.traceoutput~destination(.show~new(src))
.debuginput~destination(.in~new(.array~of('say "typed"', '')))
say 'before'
trace ?a
say 'one'
say 'two'
exit
::class show
::method init
  expose src
  use arg src
::method lineout
  expose src
  use arg l
  l = '' || l
  if l = '       +++ "' || src || '"' then l = '<source line>'
  .stdout~lineout(l)
  return 0
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  say '[read' i']'
  if i > a~items then return ''
  return a[i]
```

```output
before
<source line>
     6 *-* say 'one'
one
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1]
typed
[read 2]
     7 *-* say 'two'
two
[read 3]
     8 *-* exit
```

```rexx
parse source src
.traceoutput~destination(.show~new(src))
.debuginput~destination(.in~new(.array~of('', '', '', '', '', '', '', '')))
call r1
trace ?a
say 'main'
call trace 'o'
call r1
exit
r1:
  trace ?a
  nop
  return
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method init
  expose src
  use arg src
::method lineout
  expose src
  use arg l
  l = '' || l
  if l = '       +++ "' || src || '"' then l = '<source line>'
  if l~word(2) \= 'Interactive' then .stdout~lineout(l)
  return 0
```

```output
<source line>
    12 *-*   nop
[read 1: ]
    13 *-*   return
<source line>
     6 *-* say 'main'
main
[read 2: ]
     7 *-* call trace 'o'
    12 *-*   nop
[read 3: ]
    13 *-*   return
```

r[env.debugging.notice]
Each time interactive tracing is turned on where it was off, the line
`+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++` is
written just before the first stop that follows. Turning it off and on again
in the same code, in the program or by a line typed at a stop, writes it
again. In the example it is written before the first stop of each of the two
periods in which interactive tracing is on.

```rexx
.debuginput~destination(.in~new(.array~of('', '', '', '', '', '')))
.traceoutput~destination(.show~new)
trace ?a
say 'p1'
call trace 'o'
say 'p2'
trace ?a
say 'p3'
say 'p4'
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output defect=#221
     4 *-* say 'p1'
p1
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: ]
     5 *-* call trace 'o'
p2
     8 *-* say 'p3'
p3
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 2: ]
     9 *-* say 'p4'
p4
[read 3: ]
    10 *-* exit
```

## Where the lines go

r[env.debugging.output]
Every line that interactive tracing writes, as well as the shown clauses and
their values, reaches the destination of `.traceoutput` as a `LINEOUT` message
whose argument is a [`TraceObject`](r:class.traceobject.creation), as [the
destination rule](r:instr.trace.destination) says for the lines of a trace:
the line with the source string, the notice, the lines that report an error in
what was typed, and the shown clauses. The example sorts what its destination
receives into those kinds, and every line is one of them; no other line, a
prompt for instance, is written there.

```rexx
.debuginput~destination(.in~new(.array~of('say 1 +', 'say 7', '')))
.traceoutput~destination(.catalog~new)
trace ?r
say 'one'
say 'two'
call trace 'o'
::class catalog
::method lineout
  use arg l
  s = '' || l
  select
    when s~strip~left(5) = '+++ "' then kind = 'source line'
    when s~word(2) = 'Interactive' & s~word(4) = '"Trace' then kind = 'notice'
    when s~word(2) = 'Interactive' & s~word(4) = 'Error' then kind = 'error line'
    when s~pos('*-*') > 0 then kind = 'clause'
    when s~left(10) = '       >>>' then kind = 'value'
    otherwise kind = '?'
  end
  say kind~left(11) l~class~id
  return 0
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  return a[i]
```

```output
source line TraceObject
clause      TraceObject
value       TraceObject
one
notice      TraceObject
error line  TraceObject
error line  TraceObject
7
clause      TraceObject
value       TraceObject
two
clause      TraceObject
```

## When the interpreter stops

r[env.debugging.pause]
A stop follows a traced clause, after the clause has run and after the lines
that it caused have been written, and before the next clause starts. A CALL
clause that calls a routine is followed by one stop, after the routine has
returned and its RETURN has been shown, besides the stops inside the routine,
which come while interactive tracing is on in it. A `LEAVE` is followed by a
stop, and so is a label, however it is reached. In the example the stops are
the ones that follow `n = 1`, the label `sub:`, the clause in the routine, and
the CALL, which is stopped at after the `return` is shown.

```rexx
.debuginput~destination(.in~new(.array~of('', '', '', '', '')))
.traceoutput~destination(.show~new)
trace ?r
n = 1
call sub
say 'back'
exit
sub: say 'in sub'
  return
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     4 *-* n = 1
       >>>   "1"
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: ]
     5 *-* call sub
     8 *-*   sub:
[read 2: ]
     8 *-*   say 'in sub'
       >>>     "in sub"
in sub
[read 3: ]
     9 *-*   return
[read 4: ]
     6 *-* say 'back'
       >>>   "back"
back
[read 5: ]
     7 *-* exit
```

A `DO` or `LOOP` clause is stopped at after its header has been evaluated,
and a condition that the evaluation queued is not delivered until that stop is
over. In the example the `NOTREADY` that the `LINEIN` call raises in the header
is handled after the stop that follows the header: the label `h:` and the
clauses of the handler are shown and stopped at after it.

```rexx
.debuginput~destination(.in~new(.array~of('', '', '', '', '', '', '', '', '')))
.traceoutput~destination(.show~new)
call on notready name h
trace ?a
do i = 1 to length(linein('/nonexistent/dh')) + 1
  say 'body' i
end
say 'after'
exit
h: say 'h' sigl
  return
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     5 *-* do i = 1 to length(linein('/nonexistent/dh')) + 1
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: ]
    10 *-*     h:
[read 2: ]
    10 *-*     say 'h' sigl
h 5
[read 3: ]
    11 *-*     return
     6 *-*   say 'body' i
body 1
[read 4: ]
     7 *-* end
     5 *-* do i = 1 to length(linein('/nonexistent/dh')) + 1
     8 *-* say 'after'
after
[read 5: ]
     9 *-* exit
```

r[env.debugging.pause.skipped]
Some traced clauses are not followed by a stop, because they cannot be run
again safely: a DO or LOOP clause on its second and later turns, which
includes the turn that finds the loop at its end, an END, a THEN, an ELSE, an
OTHERWISE, an ITERATE, a SIGNAL of any kind, a CALL ON or a CALL OFF, a
RETURN, whose stop is the one that follows the clause that made the call,
whether a CALL, a function call or a message in an expression, and there is
none when that clause has no stop, an EXIT, and a clause that raises a syntax
error. A label that [SIGNAL ON](r:cond.traps.signal) transfers to after an
error is followed by a stop like any label, and so is a command whose return
code raises ERROR or FAILURE, a stop that comes before a trap for the
condition runs. The example prints a number at each stop. It shows each clause
of the rule without a stop: the second and third turns of the loop header, the
two `end` clauses, `then`, `else` and `otherwise`, `iterate`, the CALL ON, the
SIGNAL ON and the plain SIGNAL, and `x = 1 / 0`, after which the label `oops`
is the next thing that is shown and stopped at.

```rexx
.debuginput~destination(.in~new(.array~of('', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '')))
.traceoutput~destination(.show~new)
trace ?a
n = 1
if n = 1 then nop
if n = 2 then nop
else nop
select
  when n = 5 then nop
  otherwise nop
end
do i = 1 to 2
  if i = 1 then iterate
  nop
end
do forever
  leave
end
call sub
call on error name ce
signal on syntax name oops
x = 1 / 0
oops: signal fin
fin: say 'done'
exit
sub: return
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     4 *-* n = 1
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: ]
     5 *-* if n = 1 
[read 2: ]
     5 *-*   then
     5 *-*     nop
[read 3: ]
     6 *-* if n = 2 
[read 4: ]
     7 *-*   else
     7 *-*     nop
[read 5: ]
     8 *-* select
[read 6: ]
     9 *-*   when n = 5 
[read 7: ]
    10 *-*   otherwise
    10 *-*     nop
[read 8: ]
    11 *-* end
    12 *-* do i = 1 to 2
[read 9: ]
    13 *-*   if i = 1 
[read 10: ]
    13 *-*     then
    13 *-*       iterate
    12 *-* do i = 1 to 2
[read 11: ]
    13 *-*   if i = 1 
[read 12: ]
    14 *-*   nop
[read 13: ]
    15 *-* end
    12 *-* do i = 1 to 2
    16 *-* do forever
[read 14: ]
    17 *-*   leave
[read 15: ]
    19 *-* call sub
    26 *-*   sub:
[read 16: ]
    26 *-*   return
[read 17: ]
    20 *-* call on error name ce
    21 *-* signal on syntax name oops
    22 *-* x = 1 / 0
    23 *-* oops:
[read 18: ]
    23 *-* signal fin
    24 *-* fin:
[read 19: ]
    24 *-* say 'done'
done
[read 20: ]
    25 *-* exit
```

r[env.debugging.interpret]
The clauses of a string that INTERPRET runs are traced and stopped after as
the clauses of the program are, and the INTERPRET clause itself is followed by
a stop that comes before the first clause of its string. A string of two
clauses therefore causes three stops, and an empty string one.

```rexx
.debuginput~destination(.in~new(.array~of('', '', '', '', '')))
.traceoutput~destination(.show~new)
trace ?a
interpret 'say 1; say 2'
interpret ''
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(3) \= '+++' then .stdout~lineout(l)
  return 0
```

```output
     4 *-* interpret 'say 1; say 2'
[read 1: ]
     4 *-* say 1;
1
[read 2: ]
     4 *-* say 2
2
[read 3: ]
     5 *-* interpret ''
[read 4: ]
     6 *-* exit
```

## What is read

r[env.debugging.read]
At a stop the interpreter sends the message `LINEIN`, with no argument, to the
object that the entry `DEBUGINPUT` of `.local` holds, and takes the answer as
the line that was typed. When the line is the empty string, the stop is over
and the program goes on, so an input that has run dry and answers the empty
string lets the program run on. The example installs an object of its own on
`.debuginput`: it is sent `LINEIN` three times, with no argument each time,
for the line it answers, the empty line that follows, and the stop after the
second `nop`.

```rexx
.traceoutput~destination(.quiet~new)
a = .in~new('say "from debuginput"')
.debuginput~destination(a)
trace ?a
nop
nop
call trace 'o'
say 'LINEIN sent' a~calls 'times, with' a~args 'arguments'
::class quiet
::method lineout
  return 0
::class in
::attribute calls
::attribute args
::method init
  expose line
  use arg line
  self~calls = 0
  self~args = 0
::method linein
  expose line
  self~calls += 1
  self~args = arg()
  if self~calls = 1 then return line
  return ''
```

```output
from debuginput
LINEIN sent 3 times, with 0 arguments
```

r[env.debugging.read.fallback]
When nothing was installed, the object that `.debuginput` holds is a monitor
whose destination is the monitor `.input`, as [the bindings
rule](r:env.objects.bindings) says, so the stops read from whatever `.input`
reads from, which is the standard input unless a program changes it. The
example makes `.debuginput` the way it was by taking the destination that it
installed off, and then sets the destination of `.input`: the stops read from
that object.

```rexx
.traceoutput~destination(.quiet~new)
.debuginput~destination(.in~new('say "never read"'))
.debuginput~destination
say .debuginput~current~class~id
b = .in~new('say "from input"')
.input~destination(b)
trace ?a
nop
nop
call trace 'o'
say 'LINEIN sent' b~calls 'times'
::class quiet
::method lineout
  return 0
::class in
::attribute calls
::method init
  expose line
  use arg line
  self~calls = 0
::method linein
  expose line
  self~calls += 1
  if self~calls = 1 then return line
  return ''
```

```output
Monitor
from input
LINEIN sent 3 times
```

r[env.debugging.read.entry]
It is the entry `DEBUGINPUT` of `.local` that is read, as for [the other
objects](r:env.objects.replace), and nothing else. Replacing the entry `INPUT`
of `.local` by another object changes what `.input` finds and not the monitor
that `.debuginput` is connected to, so the stops do not read from it; and an
object that is in `.environment` under the name `DEBUGINPUT` is not read when
the entry of `.local` is gone. When the entry is gone, a stop reads nothing
and the program goes on; interactive tracing stays on. In the first example
the stops read from the object that is the destination of `.input`, and the
object that then replaces `INPUT` is never sent `LINEIN`; in the second the
object in `.environment` is never sent it, and `trace()` still answers `?A`
after the stop that read nothing.

```rexx
.traceoutput~destination(.quiet~new)
b = .in~new('say "from b"')
.input~destination(b)
c = .in~new('say "from c"')
.local~input = c
trace ?a
nop
call trace 'o'
say 'b read' b~calls 'times, c read' c~calls 'times'
::class quiet
::method lineout
  return 0
::class in
::attribute calls
::method init
  expose line
  use arg line
  self~calls = 0
::method linein
  expose line
  self~calls += 1
  if self~calls = 1 then return line
  return ''
```

```output
from b
b read 2 times, c read 0 times
```

```rexx
.traceoutput~destination(.quiet~new)
a = .in~new
.environment~debuginput = a
.local~remove('DEBUGINPUT')
trace ?a
nop
say 'during' trace()
nop
call trace 'o'
say 'sent' a~calls 'times, trace' trace()
::class quiet
::method lineout
  return 0
::class in
::attribute calls
::method init
  self~calls = 0
::method linein
  self~calls += 1
  return ''
```

```output
during ?A
sent 0 times, trace O
```

r[env.debugging.read.refused]
An object that the stops read from, and that does not understand `LINEIN`,
raises `SYNTAX 97.1` at the first stop, an ordinary condition of the clause
that the stop follows: a trap for SYNTAX in force catches it. The example is
ended by that error, with the return code that goes with it, after the clause
that came before the stop has written its line.

```rexx
.debuginput~destination(.mute~new)
trace ?a
say 'one'
say 'two'
::class mute
```

```output rc=159
one
```

## What a line does

r[env.debugging.input.clause]
A line that is not empty and not `=` is run as the clauses it holds, in the
way an INTERPRET clause would run it: it can hold several clauses, separated
by semicolons, and a group such as a DO must be complete in it. Variables that
it sets stay set. The clauses of the line are not traced, so they write no
line, and no stop follows any of them. After the line has run the interpreter
stops again: it reads another line, and goes on to the next clause of the
program only when the line is empty or does one of the things that the rules
below give. A line that holds only blanks, only a comment, only a semicolon or
only `nop` does nothing and is followed by another read. A LEAVE or an ITERATE
in a line does not reach the loops of the program: it raises error 28, which
is reported as [the next rule](r:env.debugging.input.error) says. An INTERPRET
in a line runs its string untraced too ([Deviation
150](r:env.debugging.deviation.150)). In the
example the interpreter reads eight lines after `say 'prog' x`, the last one
the empty line.

```rexx
.debuginput~destination(.in~new(.array~of('x = 5; y = x * 2', 'say x y', 'do k = 1 to 2; say k; end', '   ', ';', 'nop', '/* just a comment */', '')))
.traceoutput~destination(.show~new)
x = 1
trace ?r
say 'prog' x
say 'prog' x
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     5 *-* say 'prog' x
       >>>   "prog 1"
prog 1
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: x = 5; y = x * 2]
[read 2: say x y]
5 10
[read 3: do k = 1 to 2; say k; end]
1
2
[read 4:    ]
[read 5: ;]
[read 6: nop]
[read 7: /* just a comment */]
[read 8: ]
     6 *-* say 'prog' x
       >>>   "prog 5"
prog 5
     7 *-* exit
```

r[env.debugging.input.error]
An error in a line does not end the program. It is reported by two lines that
reach `.traceoutput` like the others, which start with
`+++ Interactive trace.  Error`, the number and then the subcode of the error,
followed by text that is not specified; the interpreter then stops again and
reads another line. Errors that are found when the line is read, such as an
expression that cannot be parsed or a DO that is not closed, and errors that
happen when it runs, such as a division by zero or a call of a routine that
does not exist, are all reported that way, and so is one that RAISE SYNTAX
in a line causes ([Deviation 151](r:env.debugging.deviation.151)); the empty
line after any of them is read as any other ([Deviation
152](r:env.debugging.deviation.152)). The error is not
[trapped](r:cond.traps.signal), even when the program has a trap for its
condition in force; the example would branch to `trapped` otherwise. The
example prints only the code of each reported error.

```rexx
.debuginput~destination(.in~new(.array~of('say 1 +', 'do', 'say "inside"', 'say 1/0', 'call nosuchroutine_a6zq', 'say 2', '')))
.traceoutput~destination(.show~new)
signal on syntax name trapped
trace ?a
say 'one'
say 'two'
exit
trapped: say 'trapped' condition('c')
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  select
    when l~strip~left(5) = '+++ "' then nop
    when l~word(2) = 'Interactive' & l~word(4) = 'Error' then do
      say 'error' l~word(5)~strip('T', ':')
    end
    otherwise .stdout~lineout(l)
  end
  return 0
```

```output
     5 *-* say 'one'
one
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: say 1 +]
error 35
error 35.1
[read 2: do]
error 14
error 14.1
[read 3: say "inside"]
inside
[read 4: say 1/0]
error 42
error 42.3
[read 5: call nosuchroutine_a6zq]
error 43
error 43.1
[read 6: say 2]
2
[read 7: ]
     6 *-* say 'two'
two
     7 *-* exit
```

r[env.debugging.input.condition]
A line runs in the variables and the condition traps of the program, with two
differences. A `SIGNAL ON` that a line holds sets a trap of the program, and
the trap stays when the line is over; but no trap is used while the line
itself runs, as the rule before says. And a command that a line runs does not
set `RC`; nothing is written about it, whatever its return code, and it raises
no ERROR or FAILURE condition that a trap of the program would catch. In the
example the first line makes the program trap the syntax error that its own
`say 1 / 0` raises later, and the second runs a command whose status would be
5: `rc` still holds 7, nothing is written, and the trap for ERROR is not
called.

```rexx
.debuginput~destination(.in~new(.array~of('signal on syntax name hs', 'rc = 7; address system "exit 5"', 'say rc', '')))
.traceoutput~destination(.show~new)
call on error name ce
trace ?a
say 'one'
say 1 / 0
exit
hs: say 'trapped' condition('c')
exit
ce: say 'ce'
return
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     5 *-* say 'one'
one
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: signal on syntax name hs]
[read 2: rc = 7; address system "exit 5"]
[read 3: say rc]
7
[read 4: ]
     6 *-* say 1 / 0
     8 *-* hs:
     8 *-* say 'trapped' condition('c')
trapped SYNTAX
     9 *-* exit
```

r[env.debugging.input.transfer]
A line that runs EXIT, RETURN or SIGNAL carries it out as the program would at
that point: EXIT ends the program with its value as the return code, RETURN
returns from the code that is stopped, with its value, and SIGNAL goes to the
label of the program that it names. The stop is over and what follows in the
line is not run. In the example each routine `body` is stopped after its
`nop`; `return 7` and `say 1; return 8` return from it, `signal lab` goes on
at its label, and `exit 4` ends the program.

```rexx
.traceoutput~destination(.quiet~new)
call probe 'return 7'
call probe 'signal lab'
call probe 'say 1; return 8'
call probe 'exit 4'
say 'not reached'
exit
probe: procedure
  .debuginput~destination(.feed~new(arg(1)))
  call body
  say 'probe' arg(1) 'result' result
  return
body:
  trace ?a
  nop
  say 'after the stop'
  return 'normal'
lab: say 'at lab'
  return 'from lab'
::class quiet
::method lineout
  return 0
::class feed
::attribute count class
::method init
  expose line
  use arg line
  .feed~count = 0
::method linein
  expose line
  .feed~count += 1
  if .feed~count = 1 then return line
  return ''
```

```output rc=4
probe return 7 result 7
at lab
probe signal lab result from lab
1
probe say 1; return 8 result 8
```

A line is run at the line of the clause that the stop follows, so a routine
that a `CALL` in it enters reads that line in `SIGL`. In the example `SIGL`
is 4, the line of the `nop`.

```rexx
.traceoutput~destination(.quiet~new)
.debuginput~destination(.feed~new('call sub'))
trace ?a
nop
say 'done'
exit
sub: say 'sigl' sigl
return
::class quiet
::method lineout
  return 0
::class feed
::method init
  expose line n
  use arg line
  n = 0
::method linein
  expose line n
  n += 1
  if n = 1 then return line
  return ''
```

```output
sigl 4
done
```

r[env.debugging.input.equals]
The line `=`, with nothing before or after it, runs again the clause that the
stop follows, shows it again with the lines that it caused, and stops after it
again; at the stop that follows a CALL that clause is the CALL, and the
routine runs again. The clause that is repeated is one of the program, never a
line that was typed. In the last example the stop after `call sub` is followed
by `=`, so that `n` is incremented twice. With a blank or any other character
beside it the line is not `=` and is run as a clause: `=` has no meaning as a
clause and raises an error, which is reported as the rule before says.

```rexx
.debuginput~destination(.in~new(.array~of('=', '=', 'say n', '=', 'say n', '')))
.traceoutput~destination(.show~new)
n = 0
trace ?r
n = n + 1
say 'n' n
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     5 *-* n = n + 1
       >>>   "1"
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: =]
     5 *-* n = n + 1
       >>>   "2"
[read 2: =]
     5 *-* n = n + 1
       >>>   "3"
[read 3: say n]
3
[read 4: =]
     5 *-* n = n + 1
       >>>   "4"
[read 5: say n]
4
[read 6: ]
     6 *-* say 'n' n
       >>>   "n 4"
n 4
     7 *-* exit
```

```rexx
.debuginput~destination(.in~new(.array~of(' =', '= ', '==', 'say n', '')))
.traceoutput~destination(.show~new)
n = 0
trace ?a
n = n + 1
say 'n' n
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     5 *-* n = n + 1
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1:  =]
+++ Interactive trace.  Error 35:   Invalid expression.
+++ Interactive trace.  Error 35.1:  Incorrect expression detected at "=".
[read 2: = ]
+++ Interactive trace.  Error 35:   Invalid expression.
+++ Interactive trace.  Error 35.1:  Incorrect expression detected at "=".
[read 3: ==]
+++ Interactive trace.  Error 35:   Invalid expression.
+++ Interactive trace.  Error 35.1:  Incorrect expression detected at "==".
[read 4: say n]
1
[read 5: ]
     6 *-* say 'n' n
n 1
     7 *-* exit
```

```rexx
.debuginput~destination(.in~new(.array~of('', '', '', '=', '', '', '', '')))
.traceoutput~destination(.show~new)
trace ?a
n = 0
call sub
say 'n' n
exit
sub: n += 1
  return
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     4 *-* n = 0
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: ]
     5 *-* call sub
     8 *-*   sub:
[read 2: ]
     8 *-*   n += 1
[read 3: ]
     9 *-*   return
[read 4: =]
     5 *-* call sub
     8 *-*   sub:
[read 5: ]
     8 *-*   n += 1
[read 6: ]
     9 *-*   return
[read 7: ]
     6 *-* say 'n' n
n 2
[read 8: ]
     7 *-* exit
```

An ITERATE that `=` runs again is shown at the indent of the body of its
loop, not at the indent of its own nesting. In the example the `iterate` is
nested in an IF, and is shown again at the indent of the loop body.

```rexx
.debuginput~destination(.in~new(.array~of('', '', '=', '', '', '', '', '', '', '')))
.traceoutput~destination(.show~new)
trace ?a
do i = 1 to 3
  if i = 1 then iterate
  say i
end
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     4 *-* do i = 1 to 3
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: ]
     5 *-*   if i = 1 
[read 2: ]
     5 *-*     then
     5 *-*       iterate
     4 *-* do i = 1 to 3
[read 3: =]
     5 *-*   iterate
     4 *-* do i = 1 to 3
[read 4: ]
     5 *-*   if i = 1 
[read 5: ]
     6 *-*   say i
3
[read 6: ]
     7 *-* end
     4 *-* do i = 1 to 3
     8 *-* exit
```

r[env.debugging.input.trace]
A TRACE instruction that a line runs takes effect, unlike one in the
program, and so does a call of the TRACE function with an argument; either
ends the stop, whatever setting it leaves: the program runs on to its next
stop, if any. The rest of the line still runs. A call with no argument, and a
TRACE that the line holds but does not run, end nothing.
`trace o`, `trace off`,
the bare `trace`, and a setting with a letter and no question mark, as
`trace r`, turn interactive tracing off and set the letter, as [the prefix
rule](r:bif.trace.prefix) says; `trace ?`, alone, turns it off and leaves the
letter. The program then runs on with no stop. A line that does something
else, such as `x = 1`, leaves interactive tracing on, and an input that has
run dry does too. The example runs a routine, `body`, that turns interactive
tracing on and runs three clauses, once for each line; each row shows the
line, the number of times the interpreter read, and the setting that `body`
answers at its end. Every row but the last three reads once. The last
three, the call with no argument, the TRACE that is not run and the line that
is no TRACE, read four times: the line, the empty line that follows it, and
the stops after the second and third `nop`. In the second example each line
leaves interactive tracing on, and so each ends the stop and the stops after
the second and third `nop` follow: the routine reads three times, except in
the last row, where the letter `N` shows no clause and no stop follows.

```rexx
.traceoutput~destination(.quiet~new)
call try 'trace o'
call try 'trace'
call try 'trace n'
call try 'trace r'
call try 'trace ?'
call try 'trace off'
call try 'call trace "n"'
call try 'x = trace()'
call try 'if 0 then trace ?a'
call try 'x = 1'
exit
try: procedure
  .debuginput~destination(.in~new(arg(1)))
  call body
  say arg(1)~left(18) 'reads' .in~count 'setting' result
  return
body:
  trace ?a
  nop
  nop
  nop
  return trace()
::class quiet
::method lineout
  return 0
::class in
::attribute count class
::method init
  expose line
  use arg line
  .in~count = 0
::method linein
  expose line
  .in~count += 1
  if .in~count = 1 then return line
  return ''
```

```output
trace o            reads 1 setting O
trace              reads 1 setting N
trace n            reads 1 setting N
trace r            reads 1 setting R
trace ?            reads 1 setting A
trace off          reads 1 setting O
call trace "n"     reads 1 setting N
x = trace()        reads 4 setting ?A
if 0 then trace ?a reads 4 setting ?A
x = 1              reads 4 setting ?A
```

```rexx
.traceoutput~destination(.quiet~new)
call try 'trace ?a'
call try 'trace ??'
call try 'x = trace("?i")'
call try 'trace ?n'
exit
try: procedure
  .debuginput~destination(.in~new(arg(1)))
  call body
  say arg(1)~left(18) 'reads' .in~count 'setting' result
  return
body:
  trace ?a
  nop
  nop
  nop
  return trace()
::class quiet
::method lineout
  return 0
::class in
::attribute count class
::method init
  expose line
  use arg line
  .in~count = 0
::method linein
  expose line
  .in~count += 1
  if .in~count = 1 then return line
  return ''
```

```output defect=#223
trace ?a           reads 3 setting ?A
trace ??           reads 3 setting ?A
x = trace("?i")    reads 3 setting ?I
trace ?n           reads 1 setting ?N
```

r[env.debugging.input.number]
A TRACE instruction in a line whose operand is a whole number, [which a
program may not give](r:instr.trace.number), ends the stop. `trace 0` and
`trace -0` leave nothing out, like the empty line. A positive number `n`
leaves out the next `n` stops; the clauses are shown as before. A negative
number `-n` hides the clauses up to and including the one that the `n`th stop
would follow, and leaves out those `n` stops; the clauses in between that have
no stop are hidden too. In the first example `trace 2` passes the stops after
`say 'two'` and `say 'three'`, so that the stop returns after `say 'four'`. In
the second, `trace -2` hides `say 'two'` and `say 'three'`. In the third,
`trace -4` hides the loop header, both `nop` clauses, both `end` clauses and
`say 'x'`, eight clauses up to the one that the fourth stop would follow, and
`say 'y'` is shown with its stop. In the fourth, `trace 0` and `trace -0` end
the stop and leave nothing out.

```rexx
.debuginput~destination(.in~new(.array~of('trace 2', '', '', '')))
.traceoutput~destination(.show~new)
trace ?a
say 'one'
say 'two'
say 'three'
say 'four'
say 'five'
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     4 *-* say 'one'
one
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: trace 2]
     5 *-* say 'two'
two
     6 *-* say 'three'
three
     7 *-* say 'four'
four
[read 2: ]
     8 *-* say 'five'
five
[read 3: ]
     9 *-* exit
```

```rexx
.debuginput~destination(.in~new(.array~of('trace -2', '', '', '')))
.traceoutput~destination(.show~new)
trace ?a
say 'one'
say 'two'
say 'three'
say 'four'
say 'five'
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     4 *-* say 'one'
one
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: trace -2]
two
three
     7 *-* say 'four'
four
[read 2: ]
     8 *-* say 'five'
five
[read 3: ]
     9 *-* exit
```

```rexx
.debuginput~destination(.in~new(.array~of('trace -4', '', '', '', '', '', '', '', '', '')))
.traceoutput~destination(.show~new)
trace ?a
nop
do j = 1 to 2
  nop
end
say 'x'
say 'y'
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     4 *-* nop
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: trace -4]
x
     9 *-* say 'y'
y
[read 2: ]
    10 *-* exit
```

```rexx
.debuginput~destination(.in~new(.array~of('trace 0', 'trace -0', '', '', '')))
.traceoutput~destination(.show~new)
trace ?a
nop
say 'x'
say 'y'
say 'z'
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(5) \= '+++ "' then .stdout~lineout(l)
  return 0
```

```output
     4 *-* nop
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: trace 0]
     5 *-* say 'x'
x
[read 2: trace -0]
     6 *-* say 'y'
y
[read 3: ]
     7 *-* say 'z'
z
[read 4: ]
     8 *-* exit
```

## How it ends

r[env.debugging.end]
Interactive tracing ends when a line typed at a stop turns it off, as [the
rule for TRACE lines](r:env.debugging.input.trace) says; when the program
calls the TRACE function with a setting that turns it off, as [the prefix
rule](r:bif.trace.prefix) says, where a TRACE instruction does nothing, as
[its rule](r:bif.trace.instruction) says; when the code that turned it on
returns, as [the next rule](r:env.debugging.scope) says; and when the program
ends. An input that has run dry does not end it, as the last row of [the table
of the rule for TRACE lines](r:env.debugging.input.trace) shows. In the
example the first routine turns it on and returns after two stops, and the
second turns it off by a call of the function, so that the clauses after the
call have no stop; the count of reads shows it.

```rexx
.traceoutput~destination(.quiet~new)
.in~count = 0
.debuginput~destination(.in~new)
call routine1
say 'after routine1:' .in~count 'reads, setting' trace()
call routine2
say 'after routine2:' .in~count 'reads, setting' trace()
exit
routine1:
  trace ?a
  nop
  nop
  return
routine2:
  trace ?a
  nop
  call trace 'o'
  nop
  nop
  return
::class quiet
::method lineout
  return 0
::class in
::attribute count class
::method linein
  .in~count += 1
  return ''
```

```output
after routine1: 2 reads, setting N
after routine2: 3 reads, setting N
```

r[env.debugging.scope]
Interactive tracing is part of the setting and has [its
scope](r:bif.trace.scope); in particular a method has no stop after any of its
clauses until it turns interactive tracing on itself, and neither has a
`::ROUTINE`, unless [RXTRACE](r:env.debugging.rxtrace) started it. In the
example the internal routine `inner` stops, and so does the code after it; the
routine `outer` is not traced and has no stop, and the caller reads `?A` again
after it.

```rexx
.debuginput~destination(.in~new(.array~of('', '', '', '', '', '', '', '')))
.traceoutput~destination(.show~new)
trace ?a
call inner
say 'plain' trace()
call outer
say 'back' trace()
exit
inner:
  say 'inner' trace()
  return
::routine outer
  say 'outer' trace()
  return
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  if l~strip~left(3) \= '+++' then .stdout~lineout(l)
  return 0
```

```output
     4 *-* call inner
     9 *-*   inner:
[read 1: ]
    10 *-*   say 'inner' trace()
inner ?A
[read 2: ]
    11 *-*   return
[read 3: ]
     5 *-* say 'plain' trace()
plain ?A
[read 4: ]
     6 *-* call outer
outer N
[read 5: ]
     7 *-* say 'back' trace()
back ?A
[read 6: ]
     8 *-* exit
```

## The RXTRACE variable

r[env.debugging.rxtrace]
When the interpreter starts, it reads the environment variable `RXTRACE`
once. If its value is `ON`, in any combination of upper and lower case
and with nothing else in it, the program starts as if its first clause were
`TRACE ?R`, and so does each program that it calls by name and each routine
object, whether a `::ROUTINE` directive defines it or `new` or `newFile` of
`Routine` makes it: [`trace()`](r:bif.trace.setting)
answers `?R` in it and its first clause is shown. A method starts with `N` as
without the variable, and an internal routine starts with its caller's
setting, as [the scope rule](r:bif.trace.scope) says. Any other value, among
them one with a blank around `ON`, and the empty value, is ignored. A
later change of the variable changes nothing. An example cannot start the
interpreter with the variable set, so the rule is tested by the cases of the
command-line tests, which set it and show the main program and a program
called by name starting with `?R`, a routine called by name, as a function
and as a routine object starting with `?R`, a method with `N`, and every
other value ignored. The example shows the later change: a program called by
name before and after the variable is set to `ON` starts with `N` both times.

```rexx
.traceoutput~destination(.quiet~new)
call lineout 'a6rrzqrx.rex', 'return trace()'
call lineout 'a6rrzqrx.rex'
say 'before' a6rrzqrx()
old = value('RXTRACE', 'ON', 'ENVIRONMENT')
say 'after' a6rrzqrx()
call value 'RXTRACE', old, 'ENVIRONMENT'
call sysfiledelete 'a6rrzqrx.rex'
exit
::class quiet
::method lineout
  return 0
```

```output
before N
after N
```

## Deviations

r[env.debugging.deviation.150]
An INTERPRET in a line runs the clauses of its string as the rest of the line
runs: they are not traced and no stop follows them, so that the line
`interpret 'say 7'` writes 7 and the interpreter then reads the next line.
ooRexx traces the clauses of the string: it writes the line of `say 7` and
then stops once more before it reads the next line, so that it reads one line
more than interrexx does; a string of several clauses causes a stop after
each. The example is not run against it.

```rexx
.debuginput~destination(.in~new(.array~of("interpret 'say 7'", '', '', '', '')))
.traceoutput~destination(.show~new)
trace ?a
say 'one'
say 'two'
say 'three'
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  select
    when l~strip~left(5) = '+++ "' then nop
    when l~word(2) = 'Interactive' & l~word(4) = 'Error' then do
      say 'error' l~word(5)~strip('T', ':')
    end
    otherwise .stdout~lineout(l)
  end
  return 0
```

```output
     4 *-* say 'one'
one
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: interpret 'say 7']
7
[read 2: ]
     5 *-* say 'two'
two
[read 3: ]
     6 *-* say 'three'
three
[read 4: ]
     7 *-* exit
```

r[env.debugging.deviation.151]
A line that runs RAISE SYNTAX is an error of the line like any other
([the rule](r:env.debugging.input.error) gives them): it is reported, and the
tracing and the stops go on as before. After the line ooRexx writes no more
lines of clauses, and in the example it does not stop again either, so that
the program runs to its end without a read. The example is not run against it.

```rexx
.debuginput~destination(.in~new(.array~of('raise syntax 93.900', '', '', '', '')))
.traceoutput~destination(.show~new)
trace ?a
say 'one'
say 'two'
say 'three'
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  select
    when l~strip~left(5) = '+++ "' then nop
    when l~word(2) = 'Interactive' & l~word(4) = 'Error' then do
      say 'error' l~word(5)~strip('T', ':')
    end
    otherwise .stdout~lineout(l)
  end
  return 0
```

```output
     4 *-* say 'one'
one
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: raise syntax 93.900]
error 93
error 93.900
[read 2: ]
     5 *-* say 'two'
two
[read 3: ]
     6 *-* say 'three'
three
[read 4: ]
     7 *-* exit
```

r[env.debugging.deviation.152]
An empty line that comes after a line that failed as it was read, such as
`trace 1.5`, whose setting does not exist, or an expression that cannot be
parsed, ends the stop and goes on to the next clause of the
program, as an empty line always does. ooRexx ends interactive tracing at that
empty line: the clauses that follow are run without being traced and without
a stop. The example is not run against it.

```rexx
.debuginput~destination(.in~new(.array~of('trace 1.5', '', '', '', '')))
.traceoutput~destination(.show~new)
trace ?a
say 'one'
say 'two'
say 'three'
exit
::class in
::method init
  expose a i
  use arg a
  i = 0
::method linein
  expose a i
  i += 1
  if i > a~items then return ''
  say '[read' i':' a[i]']'
  return a[i]
::class show
::method lineout
  use arg l
  l = '' || l
  select
    when l~strip~left(5) = '+++ "' then nop
    when l~word(2) = 'Interactive' & l~word(4) = 'Error' then do
      say 'error' l~word(5)~strip('T', ':')
    end
    otherwise .stdout~lineout(l)
  end
  return 0
```

```output
     4 *-* say 'one'
one
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
[read 1: trace 1.5]
error 24
error 24.1
[read 2: ]
     5 *-* say 'two'
two
[read 3: ]
     6 *-* say 'three'
three
[read 4: ]
     7 *-* exit
```
