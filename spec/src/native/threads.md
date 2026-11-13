---
element: "Threads and halting"
kind: concept
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
and it leaves the interpreter when it detaches. The fixture's thread posts an
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

```output defect=#628
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
