---
element: "Debugging"
kind: concept
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
is not shown and nothing stops after it: the first stop follows the first
clause after it. How a package asks for it with `::OPTIONS TRACE` is the
business of [the directive](r:dir.options.trace); the environment variable
`RXTRACE` is described [below](r:env.debugging.rxtrace). In the first example
the first clause that is shown is `say 'one'`, the one after `trace ?a`, and
the stop follows it, after what it wrote. In the second, the function turns it
on from inside an assignment, and the stop follows the clause after that one.

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

r[env.debugging.opening]
The first time that interactive tracing is turned on in a program, a line is
written before the first clause that is shown: seven blanks, `+++`, a blank,
and in double quotation marks the string that [PARSE
SOURCE](r:instr.parse.source) gives for the program. Just before the first
stop, after the clause that was shown first and what it wrote, comes a second
line,
`+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++`. In
the example the class `show` replaces the source line by a marker, after
checking that it has the form and the content the rule gives, and the notice
follows the first `one`.

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
clause is followed by one stop, after the routine has returned and its RETURN
has been shown, besides the stops inside the routine, which come while
interactive tracing is on in it. A `LEAVE` is followed by a stop, and so is a
label that a CALL or a SIGNAL has reached. In the example the stops are the
ones that follow `n = 1`, the label `sub:`, the clause in the routine, and the
CALL, which is stopped at after the `return` is shown.

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

r[env.debugging.pause.skipped]
Some traced clauses are not followed by a stop, because they cannot be run
again safely: a DO or LOOP clause on its second and later turns, which
includes the turn that finds the loop at its end, an END, a THEN, an ELSE, an
OTHERWISE, a SIGNAL of any kind, and a clause that raises an error. A label
that [SIGNAL ON](r:cond.traps.signal) transfers to after an error is followed
by a stop like any label. The example prints a number at each stop. It shows
each clause of the rule without a stop: the second and third turns of the loop
header, the two `end` clauses, `then`, `else` and `otherwise`, the SIGNAL ON
and the plain SIGNAL, and `x = 1 / 0`, after which the label `oops` is the
next thing that is shown and stopped at.

```rexx
.debuginput~destination(.in~new(.array~of('', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '')))
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
  nop
end
do forever
  leave
end
call sub
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
    13 *-*   nop
[read 10: ]
    14 *-* end
    12 *-* do i = 1 to 2
    13 *-*   nop
[read 11: ]
    14 *-* end
    12 *-* do i = 1 to 2
    15 *-* do forever
[read 12: ]
    16 *-*   leave
[read 13: ]
    18 *-* call sub
    24 *-*   sub:
[read 14: ]
    24 *-*   return
[read 15: ]
    19 *-* signal on syntax name oops
    20 *-* x = 1 / 0
    21 *-* oops:
[read 16: ]
    21 *-* signal fin
    22 *-* fin:
[read 17: ]
    22 *-* say 'done'
done
[read 18: ]
    23 *-* exit
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
the entry of `.local` is gone. In the first example the object that replaces
`INPUT` is never sent `LINEIN`; in the second the object in `.environment` is
never sent it.

```rexx
.traceoutput~destination(.quiet~new)
.local~input = .in~new('say "from the replaced entry INPUT"')
trace ?a
nop
call trace 'o'
say 'INPUT entry sent LINEIN' .local~input~calls 'times'
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
  return line
```

```output
INPUT entry sent LINEIN 0 times
```

```rexx
.traceoutput~destination(.quiet~new)
a = .in~new
.environment~debuginput = a
.local~remove('DEBUGINPUT')
trace ?a
nop
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
sent 0 times, trace O
```

r[env.debugging.read.refused]
An object that the stops read from, and that does not understand `LINEIN`,
raises `SYNTAX 97.1` at the first stop. The example is ended by that error,
with the return code that goes with it, after the clause that came before the
stop has written its line.

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
only `nop` does nothing and is followed by another read. In the example the
interpreter reads eight lines after `say 'prog' x`, the last one the empty
line.

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
does not exist, are all reported that way. The error is not
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
set `RC`. In the example the first line makes the program trap the syntax
error that its own `say 1 / 0` raises later, and the second runs a command
whose status would be 5, and `rc` still holds 7.

```rexx
.debuginput~destination(.in~new(.array~of('signal on syntax name hs', 'rc = 7; address system "exit 5"', 'say rc', '')))
.traceoutput~destination(.show~new)
trace ?a
say 'one'
say 1 / 0
exit
hs: say 'trapped' condition('c')
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
[read 1: signal on syntax name hs]
[read 2: rc = 7; address system "exit 5"]
[read 3: say rc]
7
[read 4: ]
     5 *-* say 1 / 0
     7 *-* hs:
     7 *-* say 'trapped' condition('c')
trapped SYNTAX
     8 *-* exit
```

r[env.debugging.input.equals]
The line `=`, with nothing before or after it, runs again the clause that the
program traced last, and shows it again, with the lines that it caused. A stop
follows it, as it follows a clause. The clause that is repeated is one of the
program, never a line that was typed. With a blank or any other character
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

r[env.debugging.input.trace]
A TRACE instruction, or a call of the TRACE function, that a line holds is not
ignored as it is in the program, and it ends the stop. `trace o`, `trace off`,
the bare `trace`, and a setting with a letter and no question mark, as
`trace r`, turn interactive tracing off and set the letter, as [the prefix
rule](r:bif.trace.prefix) says; `trace ?`, alone, turns it off and leaves the
letter. The program then runs on with no stop. A line that does something
else, such as `x = 1`, leaves interactive tracing on, and an input that has
run dry does too. The example runs a routine, `body`, that turns interactive
tracing on and runs three clauses, once for each line; each row shows the
line, the number of times the interpreter read, and the setting that `body`
answers at its end. Every row but the last reads once. The last reads four
times: the line, the empty line that follows it, and the stops after the
second and third `nop`.

```rexx
.traceoutput~destination(.quiet~new)
call try 'trace o'
call try 'trace'
call try 'trace n'
call try 'trace r'
call try 'trace ?'
call try 'trace off'
call try 'call trace "n"'
call try 'x = 1'
exit
try: procedure
  .debuginput~destination(.in~new(arg(1)))
  call body
  say arg(1)~left(16) 'reads' .in~count 'setting' result
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
trace o          reads 1 setting O
trace            reads 1 setting N
trace n          reads 1 setting N
trace r          reads 1 setting R
trace ?          reads 1 setting A
trace off        reads 1 setting O
call trace "n"   reads 1 setting N
x = 1            reads 4 setting ?A
```

r[env.debugging.input.number]
A TRACE instruction in a line whose operand is a whole number, [which a
program may not give](r:instr.trace.number), ends the stop. A positive number
`n` lets the program run until `n` further stops have been passed: the clauses
are shown as before, and the stops that would follow the next `n` of them are
left out. A negative number `-n` lets the next `n` clauses that would have
been traced run without being shown and without a stop, and the stops resume
after them. In the first example `trace 2` passes the stops after `say 'two'`
and `say 'three'`, so that the stop returns after `say 'four'`. In the second,
`trace -2` hides `say 'two'` and `say 'three'`.

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
The setting that interactive tracing belongs to is the
[scope](r:bif.trace.scope) of the TRACE function: an internal routine starts
with the interactive tracing of its caller and the caller has it back when the
routine returns, wherever the routine turned it on or off; a routine that a
`::ROUTINE` directive defines, and a method, start with no tracing at all, so
no stop follows any of their clauses. In the example the internal routine
`inner` stops, and so does the code after it; the routine `outer` is not
traced and has no stop, and the caller reads `?A` again after it.

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
When the interpreter starts a program, it reads the environment variable
`RXTRACE`. If its value is `ON`, in any combination of upper and lower case
and with nothing else in it, the program starts as if its first clause were
`TRACE ?R`: [`trace()`](r:bif.trace.setting) answers `?R` in it and its first
clause is shown. Any other value, among them one with a blank around `ON`, and
the empty value, is ignored. The variable is read once, when the interpreter
starts, so the rule is tested by starting the interpreter with the variable
set, which an example cannot do.