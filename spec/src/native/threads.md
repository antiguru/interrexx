---
element: "Threads and halting"
kind: concept
deviations: [166, 172, 173, 174]
---

# Threads and halting

Native code may start threads of its own, and may ask the interpreter to halt
activities. This page states what a program sees of both. The examples use
[the fixture library `specfixture`](index.md#the-fixture-libraries). The ones
that start a thread run their program as a separate `rexx` command, so that
the outcome is that program's output and return code.

## Attached threads

r[native.threads.attach]
A thread that native code starts may attach to the interpreter. It is then an
[activity](../concurrency/activities.md) of its own, which runs the Rexx code
of the messages it sends while the program's other activities run as well,
except the one each message interrupts, which [Deviation
174](r:native.threads.attach.deviation.174) stops, and it leaves the
interpreter when it detaches. The fixture's thread posts an
EventSemaphore that the main program waits on.

```rexx
src = .array~of( -
  "sem = .eventsemaphore~new", -
  "say fxonthread(sem, 'POST')", -
  "sem~wait", -
  "say 'posted'", -
  "::requires 'specfixture' LIBRARY")
call sysfiledelete 'n3zqattach.rex'
do line over src
  call lineout 'n3zqattach.rex', line
end
call lineout 'n3zqattach.rex'
address system 'rexx n3zqattach.rex 2>/dev/null' with output stem out.
say 'rc' rc
do i = 1 to out.0
  say out.i
end
call sysfiledelete 'n3zqattach.rex'
```

```output defect=#621
rc 0
1
posted
```

r[native.threads.attach.deviation.174]
While a message that an attached thread sends runs, the activity that was
running when it arrived makes no progress in Rexx clauses ([Deviation
174](../deviations.md#174-a-message-from-an-attached-thread-stops-the-activity-it-interrupts)).
The message runs at that activity's next clause boundary, or during its wait
or its call into a native library that is not built in, and the activity goes
on only once the message has finished. A message that arrives while another
runs is held until that one returns, waits or calls into such a library, the
earliest first. One that arrives while another waits runs on top of it, and the
earlier goes on only once the later has finished. A message the thread sends
with `SendMessage` or one of its variants, whose method needs at its entry a
guard lock a stopped activity holds, waits for it as in ooRexx, and `TRACE`
announces the method's entry once. Any other wait counts as one of a message
that has begun, including the wait of a routine the thread calls with
`CallRoutine` or of a method another API call sends to, such as the
`MAKESTRING` that `ObjectToString` sends.
Once a message has begun, a wait of it for something only a stopped activity
can give never ends. A wait for a guard lock such an activity holds and no
other message does, or for the completion of a message whose send it runs, is
refused at once, and the program ends with exit code 120. Any other, such as a
`GUARD WHEN`, goes on for ever, with a line on stderr once it has waited five
seconds and nothing but a stopped activity is left to end it. In
ooRexx the thread's messages run alongside the other activities. The tests of
this rule need an extension of their own that sends messages by name, so they
run outside this page.

r[native.threads.attach.deviation.173]
`DetachThread` called from a thread other than the one that attached the
context does nothing, and so does a second `DetachThread` from the thread that
attached it ([Deviation
173](../deviations.md#173-detachthread-from-the-wrong-thread-or-twice)): the
attachment stays as it was, and its next message runs. A message sent after
the thread did detach is not covered here but by [Deviation
19](../deviations.md#19-callback-from-a-foreign-thread), which refuses it. ooRexx
aborts the process on both misuses, so there is no example.

r[native.threads.end]
The end of a program waits for an attached thread only while that thread's
activity runs a message, as it waits for [any other
activity](r:conc.halt.end.alive), and the program's return code is that of its
main program. A thread that is attached but sends nothing does not keep the
program running: the program ends without it, and the thread does not get to
detach. In the first example the attached thread's message has started
before the main program ends, and finishes after it; in the second the thread
has finished its message and stays attached, idle, for three seconds, and the
program ends at once.

```rexx
src = .array~of( -
  "k = .n3zqk~new", -
  "call fxonthread k, 'RUN'", -
  "k~started~wait", -
  "::requires 'specfixture' LIBRARY", -
  "::class n3zqk", -
  "::attribute started", -
  "::method init", -
  "  expose started", -
  "  started = .eventsemaphore~new", -
  "::method run", -
  "  expose started", -
  "  started~post", -
  "  call syssleep 0.5", -
  "  say 'thread ran'")
call sysfiledelete 'n3zqend.rex'
do line over src
  call lineout 'n3zqend.rex', line
end
call lineout 'n3zqend.rex'
address system 'rexx n3zqend.rex 2>/dev/null' with output stem out.
say 'rc' rc
do i = 1 to out.0
  say out.i
end
call sysfiledelete 'n3zqend.rex'
```

```output defect=#621
rc 0
thread ran
```

```rexx
src = .array~of( -
  "sem = .eventsemaphore~new", -
  "call fxonthreadidle sem, 'POST', 3", -
  "sem~wait", -
  "say 'main ends'", -
  "::requires 'specfixture' LIBRARY")
call sysfiledelete 'n3zqidle.rex'
do line over src
  call lineout 'n3zqidle.rex', line
end
call lineout 'n3zqidle.rex'
call time 'R'
address system 'rexx n3zqidle.rex 2>/dev/null' with output stem out.
say 'rc' rc 'waited' (time('E') >= 2)
do i = 1 to out.0
  say out.i
end
call sysfiledelete 'n3zqidle.rex'
```

```output defect=#621
rc 0 waited 0
main ends
```

r[native.threads.end.deviation.172]
A thread that is still attached when the embedding calls `Terminate` can send
no message afterwards: each send through its context is refused, answers the
failure value of the member and runs nothing, and `DetachThread` from that
thread returns ([Deviation
172](../deviations.md#172-a-callback-after-the-embedding-ends-the-interpreter)).
ooRexx runs the message and answers its result. The example needs an
embedding that calls `Terminate`, which the `rexx` command does not, so the
tests of this rule run one instead.

r[native.threads.instance]
Every activity of one interpreter is given the same interpreter instance by
the native API: the main program's, one that [`REPLY`](../instructions/reply.md)
or [`start`](r:conc.activities.creation) makes, and an attached thread's. So
native code that keeps something for an interpreter finds it again from any
of its activities, and what native code asks of the instance, such as halting
every activity ([the halt rule](r:native.threads.halt)) or attaching a thread,
acts on the whole interpreter, not on the activity that asked. The fixture's
`FxInstance` writes the instance it is given as an address, which the example
compares.

```rexx
main = fxinstance()
o = .n3zqi~new
o~replied
o~done~wait
say 'main' (fxinstance() == main)
say 'reply' (o~seen == main)
say 'start' (o~start('plain')~result == main)
::requires 'specfixture' LIBRARY
::class n3zqi
::attribute done
::attribute seen
::method init
  expose done
  done = .eventsemaphore~new
::method replied
  expose done seen
  reply
  seen = fxinstance()
  done~post
::method plain
  return fxinstance()
```

```output
main 1
reply 1
start 1
```

> **Note.** A thread may use only the contexts it was given: the one that its attachment
> gave it, and, while a native call runs on it, that call's. A callback made
> through a call's context from another thread is [Deviation
> 19](../deviations.md), and a context kept from one call and used in another is
> [Deviation 24](../deviations.md); neither has an example, because ooRexx ends
> the process or hangs on both. Ending the interpreter belongs to the program
> that created it, which for a program run by `rexx` is `rexx` itself, so what
> native code that asks for the end achieves is not specified.

## Halting

r[native.threads.halt]
Native code may ask the interpreter to halt every activity, or only the
activity whose call is running it. Each activity that is asked takes the
[HALT condition](../conditions/halt.md) at its next clause, which for the
activity of the call is the first clause after the call returns, and an
activity that was not asked runs on. In the examples a second activity loops
with a short sleep and records whether it was halted.

```rexx
w = .n3zqw~new
w~start('spin')
w~ready~wait
signal on halt name h
call fxhaltall
do i = 1 to 1000000; nop; end
say 'main not halted'
h: w~done~wait
say 'main' condition('c') '/ worker' w~how
::requires 'specfixture' LIBRARY
::class n3zqw
::attribute ready
::attribute done
::attribute how
::method init
  expose ready done how
  ready = .eventsemaphore~new
  done = .eventsemaphore~new
  how = 'running'
::method spin unguarded
  expose ready done how
  signal on halt name h
  ready~post
  do 200
    call syssleep 0.01
  end
  how = 'not halted'
  done~post
  return
h: how = 'halted'
  done~post
```

```output
main HALT / worker halted
```

```rexx
w = .n3zqw~new
w~start('spin')
w~ready~wait
signal on halt name h
call fxhaltthis
do i = 1 to 1000000; nop; end
say 'main not halted'
h: w~done~wait
say 'main' condition('c') '/ worker' w~how
::requires 'specfixture' LIBRARY
::class n3zqw
::attribute ready
::attribute done
::attribute how
::method init
  expose ready done how
  ready = .eventsemaphore~new
  done = .eventsemaphore~new
  how = 'running'
::method spin unguarded
  expose ready done how
  signal on halt name h
  ready~post
  do 200
    call syssleep 0.01
  end
  how = 'not halted'
  done~post
  return
h: how = 'halted'
  done~post
```

```output
main HALT / worker not halted
```

## Tracing

r[native.threads.trace]
Native code may turn the external trace of the interpreter on or off from any
thread, whether or not that thread runs a call of the interpreter. Turned on,
every activity of the interpreter that runs Rexx code, the main one and the
ones that [`start`](r:conc.activities.creation) made, idle or not, traces from
its next clause as under `TRACE ?R`, with the pause of interactive trace, except
for the lines that [Deviation 166](r:native.threads.trace.deviation.166) leaves
out of code that was already running. The setting is that of the code each
activity is running: a method or a `::ROUTINE` it calls afterwards starts with
its own, as [the scope of the setting](r:bif.trace.scope) says. Turned off,
every activity runs on untraced from its next clause. A request made after the
interpreter has ended does nothing. The trace lines name the program's path,
so the tests that witness this rule compare them outside this page.

r[native.threads.trace.deviation.166]
Code that was already running when native code first turned the external trace
of the interpreter, or of one activity, on or off ([Deviation
166](../deviations.md#166-trace-lines-of-code-already-running-when-native-code-first-switches-the-trace))
may leave out some of the lines the trace asks for, up to the end of its run.
That code is each run of a program, method, `::ROUTINE` or internal routine
that an activity had begun, a caller that waits for a call to return included.
The lines it may leave out are the `>K>` lines of a `DO` or `LOOP` header for
`TO`, `BY`, `FOR`, `OVER` and `WITH`, and the `>L>`, `>V>`, `>E>`, `>O>`, `>P>`
and `>A>` lines that `TRACE I`, set at an interactive pause, asks for. Code
entered after that first request prints every line, as ooRexx does. ooRexx
prints every line in code that was already running too. The trace lines name
the program's path, so the tests that witness this rule compare them outside
this page.
