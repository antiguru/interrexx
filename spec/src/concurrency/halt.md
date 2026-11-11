---
element: "Halt and program end"
kind: concept
rexxref: "Concurrency; Builtin Classes > Fundamental Classes > Message Class > halt; Objects and Classes > Object Destruction and Uninitialization"
deviations: [10, 21, 162]
---

# Halt and program end

This page specifies three things that reach an activity from outside its own
clauses: a HALT that another activity or the host asks for, the end of the
main program while other activities still run, and the message `UNINIT` that
the interpreter sends to an object that nothing can reach any more. The
condition HALT itself, its traps and what an untrapped one does are on [the
HALT page](r:cond.halt.raise); [the `halt` method](r:class.message.halt) of
`Message` owns what the method checks and answers; the status of the command
is [the invocation rule](r:env.invocation.status), and when `UNINIT` is sent,
and to what, is [the rule of the invocation page](r:env.invocation.uninit).
This page says where in a program with several activities each of these
lands, and what the others do meanwhile.

The examples force the order of what they print, as
[the chapter](index.md) says. An example that has to show that a wait was not
ended by a HALT cannot rely on a program having reached the wait before it
sends the HALT: each such waiter tells the program that it is about to wait
while it evaluates the clause that waits, by a call of a built-in method in
the last clause of the method that marks it, so that no clause boundary is
left between the mark and the wait, and a HALT is taken only at a clause
boundary.

## HALT sent to an activity

r[conc.halt.sent]
[`halt`](r:class.message.halt) of a message that another activity runs is the
way a program raises HALT in an activity other than its own. The condition is
raised in that activity, at a clause boundary, in the call that the activity
is running then: the method that the message names, or any method or
`::ROUTINE` that method has called and that is still running. Only a trap of
that call can answer it as HALT ([the scope rule](r:cond.traps.scope)). A call
that has no trap for it leaves it untrapped, and it becomes the SYNTAX error
`4.1` there ([the rule](r:cond.halt.untrapped)), which goes up to the callers
as any SYNTAX error does ([the rule](r:cond.information.propagate)): the HALT
trap of a caller is not taken, a SYNTAX trap of a caller is, with no
description, and an error that no call traps ends the message with `4.1`
([`hasError`](r:class.message.haserror)). A trap in the running call itself
sees HALT with the description that `halt` was given. Which clause of the
target is the first to see a HALT that is sent while the target runs is not
specified ([Deviation
22](../deviations.md#22-one-schedule-for-racing-activities)). When the
activity is running a nested region, such as a method that a built-in method
calls back, interrexx takes the HALT only when that region has ended
([Deviation
17](../deviations.md#17-halt-of-an-activity-with-a-nested-region)).

```rexx
g = .gate~new
m1 = g~start('own')
m2 = g~start('caller', 'halt')
m3 = g~start('caller', 'syntax')
g~allBusy
say 'halts answered:' m1~halt('one') m2~halt('two') m3~halt('three')
say m1~result
m2~wait
say m2~hasError m2~errorCondition~code m2~errorCondition~condition
say m3~result
::class gate
::method init
  expose busy
  busy = 0
::method mark unguarded
  expose busy
  busy += 1
::method allBusy unguarded
  expose busy
  guard on when busy = 3
::method own unguarded
  expose busy
  signal on halt
  busy += 1
  do forever
    nop
  end
halt:
  return 'the call that ran took' condition('c') '['condition('d')']'
::method caller unguarded
  use arg kind
  if kind = 'halt' then signal on halt name stopped
  else signal on syntax name stopped
  self~spin
  return 'not reached'
stopped:
  return 'the caller took' condition('c') '['condition('d')']'
::method spin unguarded
  self~mark
  do forever
    nop
  end
```

```output
halts answered: 1 1 1
the call that ran took HALT [one]
1 4.1 SYNTAX
the caller took SYNTAX []
```

> **Note.** The first message loops in the method that traps HALT, so it sees
> HALT and the description `one`. The other two loop in a method that has no
> trap, called by a method that has one: the first of them traps HALT, which
> the `4.1` that the loop's method leaves is not, so its message ends with the
> error; the second traps SYNTAX and takes it, and `CONDITION('D')` is empty.
> Each counts itself in before the loop, and the program sends the HALTs when
> all three have.

r[conc.halt.pending]
A HALT that `halt` sends does not end the waits of the activity that runs the
message. An activity that is waiting for a `GUARD ... WHEN` condition, for
the lock of an object that a guarded method or `GUARD ON` asks for, for the
post of an [`EventSemaphore`](r:class.eventsemaphore.wait), for a
[`MutexSemaphore`](r:class.mutexsemaphore.acquire) held by another activity,
for the [result](r:class.message.result) or the [end](r:class.message.wait)
of a message, in `SysSleep`, or for a [command](r:env.commands.rc) to end,
goes on only when what it waits for has happened, and takes the HALT at the
first clause boundary after the one that waited. `halt` answers 1 meanwhile
and the message is not completed. The clause that waited is finished when the
HALT is taken: a variable that it assigns has the answer of the wait.

```rexx
box = .box~new
box~mx~acquire
m = .array~new
do name over .array~of('byGuard', 'byEvent', 'byMutex', 'byResult', 'byEnd', 'bySleep')
  m~append(box~start(name))
end
box~allWaiting
say 'all wait:' states(m)
say 'halts answered:' halts(m)
call syssleep 0.3
say 'after the halts:' states(m)
box~open
say m[1]~result
box~ev~post
say m[2]~result
box~mx~release
say m[3]~result
box~finish
say m[4]~result
say m[5]~result
say m[6]~result
::routine states
  use arg m
  r = ''
  do x over m
    r = r x~completed
  end
  return r~strip
::routine halts
  use arg m
  r = ''
  do i = 1 to m~items
    r = r m[i]~halt(i)
  end
  return r~strip
::class box
::method init
  expose up fin ev mx marks helper
  up = .false
  fin = .false
  ev = .EventSemaphore~new
  mx = .MutexSemaphore~new
  marks = .array~new
  helper = .message~new(self, 'slow')
  helper~start
::method arm unguarded
  expose marks
  use arg name, seconds
  return seconds + 0 * marks~append(name)
::method idle unguarded
  expose marks
  return marks~append('guard') = 0
::method peer unguarded
  expose marks helper
  use arg name
  return .array~of(helper, marks~append(name))~at(1)
::method allWaiting unguarded
  expose marks
  do until marks~items = 6
    call syssleep 0.01
  end
::method open unguarded
  expose up
  up = .true
::method finish unguarded
  expose fin
  fin = .true
::method ev unguarded
  expose ev
  return ev
::method mx unguarded
  expose mx
  return mx
::method slow unguarded
  expose fin
  guard on when fin
  return 'slow done'
::method byGuard unguarded
  expose up
  signal on halt
  guard on when up | self~idle
  return 'not reached'
halt:
  return 'guard wait ended, then' condition('d')
::method byEvent unguarded
  expose ev
  signal on halt
  got = ev~wait(self~arm('event', 60))
  return 'not reached'
halt:
  return 'event wait answered' got 'then' condition('d')
::method byMutex unguarded
  expose mx
  signal on halt
  got = mx~acquire(self~arm('mutex', 60))
  return 'not reached'
halt:
  return 'acquire answered' got 'then' condition('d')
::method byResult unguarded
  signal on halt
  got = self~peer('result')~result
  return 'not reached'
halt:
  return 'result answered' got 'then' condition('d')
::method byEnd unguarded
  signal on halt
  self~peer('end')~wait
  return 'not reached'
halt:
  return 'the end wait ended, then' condition('d')
::method bySleep unguarded
  signal on halt
  call syssleep self~arm('sleep', 1)
  return 'not reached'
halt:
  return 'the sleep ended, then' condition('d')
```

```output
all wait: 0 0 0 0 0 0
halts answered: 1 1 1 1 1 1
after the halts: 0 0 0 0 0 0
guard wait ended, then 1
event wait answered 1 then 2
acquire answered 1 then 3
result answered slow done then 4
the end wait ended, then 5
the sleep ended, then 6
```

```rexx
box = .box~new
m1 = box~start('holder')
box~holding
m2 = box~start('byLock')
box~allWaiting
say 'halt answers' m2~halt('stop')
call syssleep 0.3
say 'while the lock is held: completed' m2~completed
box~ev~post
say m1~result
say m2~result
::class box
::method init
  expose held marks ev
  held = .false
  marks = .array~new
  ev = .EventSemaphore~new
::method ev unguarded
  expose ev
  return ev
::method arm unguarded
  expose marks
  return marks~append('lock')
::method holding unguarded
  expose held
  guard off when held
::method allWaiting unguarded
  expose marks
  do until marks~items = 1
    call syssleep 0.01
  end
::method holder
  expose held ev
  held = .true
  ev~wait
  return 'the holder ended'
::method byLock unguarded
  return self~locked(self~arm)
::method locked
  signal on halt
  return 'not reached'
halt:
  return 'the guarded method took' condition('c') 'then' condition('d')
```

```output
halt answers 1
while the lock is held: completed 0
the holder ended
the guarded method took HALT then stop
```

```rexx
stem = SysTempFileName('conc-halt-????')
up = stem'.up'; go = stem'.go'
m = .runner~new~start('work', up, go)
do until SysFileExists(up)
  call syssleep 0.01
end
say 'halt answers' m~halt('stop')
call syssleep 0.3
say 'while the command runs: completed' m~completed
call lineout go, 'now'
call lineout go
say m~result
call SysFileDelete up
call SysFileDelete go
::class runner
::method work unguarded
  use arg up, go
  signal on halt
  address system 'touch' up '; while [ ! -e' go ']; do sleep 0.01; done'
  return 'not reached'
halt:
  return 'the command ended with rc' rc', then' condition('d')
```

```output
halt answers 1
while the command runs: completed 0
the command ended with rc 0, then stop
```

> **Note.** The first example starts six messages that each wait in a
> different way: for a condition, for the post of an event, for the mutex,
> for the result of a message, for the end of a message and in `SysSleep`
> for a second. The clause that waits also marks the message: the `WHEN`
> condition, the argument of `wait`, `acquire` and `SysSleep`, and the
> receiver of `result` and `wait` each come from a method whose only clause
> after `expose` and `use arg` is a `RETURN` that appends the mark to an
> array (a time out of 60 seconds, which nothing uses up, is only an
> argument to give). No clause boundary is left between the mark and the
> wait, so the program knows that each message is in its waiting clause when
> it sends the HALTs. The program holds the mutex, and the fourth and fifth
> messages wait for a message that is itself waiting for the program. The
> 0.3 seconds are a window in which a wait that a HALT ended would show; it
> ends before the second of the sleep does. In the second example a guarded
> method holds the lock of the object while it waits for a post. The
> message asks for the same lock by sending a guarded method, whose argument
> carries the mark, so that the HALT is sent while the message waits for the
> lock. It is taken in the guarded method, at its first clause, after the
> program has let the holder go. The third example makes the command say that
> it has started by creating a file, so that the HALT is sent while the
> command runs, and the program ends the command by creating a second file.

r[conc.halt.reply]
A message whose method has replied has ended ([the result
rule](r:conc.activities.result)), and a HALT sent to it does not reach the
rest of the method, which runs on as an activity that no message object
stands for: `halt` answers 1, and the continuation goes on without a HALT,
whether it is running or waiting. For a message that was started, [the rule
of `halt`](r:class.message.halt) says that nothing happens; for one that
`send` ran, that rule raises the HALT in the activity that sent it. A HALT
that the host asks for does reach a continuation ([the next
section](r:conc.halt.signal)).

```rexx
t = .t~new
m = t~start('work')
say 'reply:' m~result
t~parked
say 'halt answers' m~halt('stop')
t~release
t~await
say t~log
::class t
::method init
  expose go done parked log
  go = .false
  done = .false
  parked = .false
  log = ''
::method note unguarded
  expose parked
  parked = .true
  return .false
::method parked unguarded
  expose parked
  guard on when parked
::method release unguarded
  expose go
  go = .true
::method await unguarded
  expose done
  guard on when done
::method log unguarded
  expose log
  return log
::method work unguarded
  expose go done log
  signal on halt
  reply 'replied'
  guard on when go | self~note
  log = 'the continuation went on'
  done = .true
  return
halt:
  log = 'the continuation was halted'
  done = .true
```

```output
reply: replied
halt answers 1
the continuation went on
```

## HALT from the host

r[conc.halt.signal]
A request for HALT that the host makes, which [the HALT
page](r:cond.halt.raise) leaves to the host, goes to every activity that is
running or waiting when it is made, the main program's included and the rest
of a method that has replied too; an activity that begins after the request
is not halted by it. Each takes it at a clause boundary of the call it is
running, as [the rule above](r:conc.halt.sent) says, with no description.
On Linux, SIGINT and SIGTERM make this request, and a program can send one to
itself with a command: `kill` of the shell's parent, which is the
interpreter. [Deviation 10](../deviations.md#10-signals-halt-and-wake) says
how the waits of the activities are treated. While one activity reads the
default input stream, interrexx runs no other, so that a request made then
can miss an activity that has not begun to run
([Deviation 13](../deviations.md#13-reading-the-default-input-stream)).

```rexx
f = SysTempFileName('conc-halt-????.rex')
do line over .resources~child
  call lineout f, line
end
call lineout f
address system 'rexx' f with output stem out.
do i = 1 to out.0
  say out.i
end
call SysFileDelete f
::resource child
g = .gate~new
a = g~start('spin', 'first')
b = g~start('spin', 'second')
g~allBusy
signal on halt
address system 'kill -INT $PPID'
do forever
  nop
end
halt:
say 'main took' condition('c') '['condition('d')']'
say a~result
say b~result
say g~start('late')~result
::class gate
::method init
  expose busy
  busy = 0
::method allBusy unguarded
  expose busy
  guard on when busy = 2
::method spin unguarded
  expose busy
  use arg name
  signal on halt
  busy += 1
  do forever
    nop
  end
halt:
  return name 'took' condition('c') '['condition('d')']'
::method late unguarded
  signal on halt
  do 1000
    nop
  end
  return 'a message started after the request ran on'
halt:
  return 'a message started after the request took' condition('c')
::END
```

```output
main took HALT []
first took HALT []
second took HALT []
a message started after the request ran on
```

```rexx
f = SysTempFileName('conc-halt-????.rex')
do line over .resources~child
  call lineout f, line
end
call lineout f
address system 'rexx' f with output stem out.
do i = 1 to out.0
  say out.i
end
call SysFileDelete f
::resource child
.local~busy = .false
t = .t~new
m = t~start('work')
say 'reply:' m~result
do until .local~busy
  call syssleep 0.01
end
signal on halt
address system 'kill -INT $PPID'
do forever
  nop
end
halt:
say 'main took' condition('c')
t~await
say t~log
::class t
::method init
  expose done log
  done = .false
  log = ''
::method await unguarded
  expose done
  guard on when done
::method log unguarded
  expose log
  return log
::method work unguarded
  expose done log
  signal on halt
  reply 'replied'
  .local~busy = .true
  do forever
    nop
  end
halt:
  log = 'the continuation took' condition('c')
  done = .true
::END
```

```output
reply: replied
main took HALT
the continuation took HALT
```

> **Note.** A signal that a command sends with `kill` to its parent must not
> reach the process that checks the examples, so each example writes the
> program that sends it to a file and runs it with a second interpreter,
> whose output it prints; the program is the text of the `::RESOURCE`. In
> the first example the signal is sent when both messages loop in a method
> that traps HALT, and each of the three activities takes it; the handler of
> the main program then starts a message, which the request does not reach.
> The second sends it while the rest of a method that has replied loops: the
> continuation takes it, which a `halt` of the message does not do.

## The end of the program

r[conc.halt.end.alive]
When the main program has ended, the program goes on until every other
activity has ended. It does not end while a started message has not ended,
while the rest of a method that has replied is still running, while a call of
the target of an alarm or a ticker is under way, while an alarm has neither
gone off nor been cancelled, or while a ticker has been neither cancelled nor
stopped. Each of them runs as it did before: the end of the main program is
not a HALT for them, and they may wait for each other and use the objects of
the program. An alarm that has gone off, an alarm or a ticker that was
cancelled, and a ticker that an error in its target's `triggered` stopped
([the timers rule](r:conc.activities.timers)) do not keep the program, so a
cancelled alarm or ticker with a delay of a minute does not make the program
last a minute. An activity that waits for something that nothing can bring
about any more keeps the program for ever: [Deviation
20](r:conc.activities.deviation.20) says that interrexx refuses such a wait
while the main program runs; after the main program has ended it does not yet,
and the program hangs (#249), which no example here tests. A main program
that interrexx ends with a refusal does not wait for any of them ([the
Deviation 21 rule](r:conc.halt.deviation.21)). The status of the command is
the main program's ([the next rule](r:conc.halt.end.status)).

```rexx
g = .gate~new
m1 = g~start('started')
m2 = g~start('continued')
a = .alarm~new(0.01, g, 'alarm')
t = .ticker~new(0.01, g, 'ticker')
say 'the main program ends'
g~open
exit 3
::class gate inherit AlarmNotification
::method init
  expose turn ticks
  turn = 0
  ticks = 0
::method open unguarded
  expose turn
  turn = 1
::method started unguarded
  expose turn
  guard on when turn = 1
  say 'a started message ran'
  turn = 2
::method continued unguarded
  expose turn
  reply 'replied'
  guard on when turn = 2
  say 'the rest of a method after REPLY ran'
  turn = 3
::method triggered unguarded
  expose turn ticks
  use arg src
  if src~attachment = 'alarm' then do
    guard on when turn = 3
    say 'an alarm went off'
    turn = 4
  end
  else do
    guard on when turn >= 4
    ticks += 1
    say 'a ticker went off' ticks
    if ticks = 3 then src~cancel
  end
::method cancel unguarded
  nop
```

```output rc=3
the main program ends
a started message ran
the rest of a method after REPLY ran
an alarm went off
a ticker went off 1
a ticker went off 2
a ticker went off 3
```

```rexx
f = SysTempFileName('conc-halt-????.rex')
do line over .resources~child
  call lineout f, line
end
call lineout f
call time 'R'
address system 'rexx' f with output stem out.
do i = 1 to out.0
  say out.i
end
say 'the program did not last long:' (time('E') < 30)
call SysFileDelete f
::resource child
n = .note~new
a = .alarm~new(60, n, 'long alarm')
a~cancel
t = .ticker~new(60, n, 'long ticker')
t~cancel
s = .alarm~new(0.01, n, 'short alarm')
n~await(1)
f = .ticker~new(0.01, n, 'failing ticker')
n~await(2)
say 'the main program ends'
::class note inherit AlarmNotification
::method init
  expose count
  count = 0
::method await unguarded
  expose count
  use arg k
  guard on when count >= k
::method triggered unguarded
  expose count
  use arg src
  say src~attachment 'went off'
  count += 1
  if src~attachment = 'failing ticker' then x = 1 / 0
::method cancel unguarded
  use arg src
  say src~attachment 'was cancelled'
::END
```

```output
long alarm was cancelled
long ticker was cancelled
short alarm went off
failing ticker went off
the main program ends
the program did not last long: 1
```

> **Note.** In the first example each activity waits for the gate, which the
> program opens as its last clause but one, and then for the one before it, so
> that the lines can only come in the order shown: the program prints, opens
> the gate and ends with the status 3, and the four activities run after it
> has ended. The ticker is cancelled by its own target at the third call,
> which ends it. The second example is the text of a `::RESOURCE` that a
> second interpreter runs, and the first one prints how long that took.
> Nothing is left to wait for when its program ends: the alarm and the ticker
> that were made with a delay of a minute were cancelled, the short alarm went
> off, and the ticker that fails has stopped; so the program ends long before
> a minute is up, and the example allows it 30 seconds.

r[conc.halt.end.status]
The status of the command is the main program's own ([the status
rule](r:env.invocation.status)), whatever the other activities do, whether the
main program ended by its last clause, by `EXIT` or by an error that nothing
traps. An `EXIT` and an error that nothing traps in the method of a started
message end that message ([the unasked rule](r:conc.activities.unasked)), not
the program, and change no status. The program does not end before the other
activities have, also when the main program ended with an error, except as
[the Deviation 21 rule](r:conc.halt.deviation.21) says. In the first
example the main program ends with its last clause, and in the second with an
error; in both the two messages run to their ends, one by `EXIT 7` and one by
an error, and the status is the main program's, 0 and 214.

```rexx
g = .gate~new
m1 = g~start('leaves')
m2 = g~start('fails')
say 'main: the last clause'
g~open
::class gate
::method init
  expose turn
  turn = 0
::method open unguarded
  expose turn
  turn = 1
::method leaves unguarded
  expose turn
  guard on when turn = 1
  say 'a message ran EXIT 7'
  turn = 2
  exit 7
::method fails unguarded
  expose turn
  guard on when turn = 2
  say 'a message ran a division by zero'
  return 1 / 0
```

```output
main: the last clause
a message ran EXIT 7
a message ran a division by zero
```

```rexx
g = .gate~new
m1 = g~start('leaves')
m2 = g~start('fails')
say 'main: before the error'
g~open
say 1 / 0
::class gate
::method init
  expose turn
  turn = 0
::method open unguarded
  expose turn
  turn = 1
::method leaves unguarded
  expose turn
  guard on when turn = 1
  say 'a message ran EXIT 7'
  turn = 2
  exit 7
::method fails unguarded
  expose turn
  guard on when turn = 2
  say 'a message ran a division by zero'
  return 1 / 0
```

```output rc=214
main: before the error
a message ran EXIT 7
a message ran a division by zero
```

> **Note.** The status 214 is `256` minus the major number 42 of the division
> ([the status rule](r:env.invocation.status)). What the program writes of the
> errors that the messages ended with is not specified ([the unasked
> rule](r:conc.activities.unasked)).

## UNINIT and activities

r[conc.halt.uninit.reach]
An object is reachable by the program when any activity can reach it, and
[the invocation rule](r:env.invocation.uninit) holds for all of them: an
object that a variable, an argument or a receiver of a call that another
activity is running still holds is not sent `UNINIT`, whichever activity
dropped its own references and however often storage is reclaimed. When that
activity has let go, the object is sent it at most once.

```rexx
.environment~count = 0
.environment~held = .false
w = .worker~new
x = .counted~new
m = w~start('hold', x)
do until .environment~held
  call syssleep 0.01
end
drop x
do 20
  call gc 'force'
end
say 'while the other activity holds it:' .environment~count
w~release
say m~result
do 20
  call gc 'force'
end
say 'after that activity let go:' (.environment~count <= 1)
::class worker
::method init
  expose go
  go = .false
::method release unguarded
  expose go
  go = .true
::method hold unguarded
  expose go
  use arg obj
  .environment~held = .true
  guard on when go
  return 'the other activity ended'
::class counted
::method uninit
  .environment~count += 1
```

```output
while the other activity holds it: 0
the other activity ended
after that activity let go: 1
```

r[conc.halt.uninit.runs]
The `UNINIT` method that a call of [`GC('FORCE')`](r:bif.gc.force) sends to an
object is run by the activity that made the call, and the call returns only
after the method has ended. No other activity is held up by it: while the
method waits, the others go on. Whether a call sends `UNINIT` to a particular
object is the matter of [the invocation rule](r:env.invocation.uninit), and
which activity runs the method when no call of `GC` asks for the collection,
and at the end of the program, is not specified. The first example has one
activity call `GC('FORCE')` until the method has begun, which waits for the
program to set a flag that only an activity that is not held in the call can
set. The second has a guarded method call `GC('FORCE')` while it holds the
lock of its object, and `UNINIT` sends a guarded method to the same object.
The lock is free for that message only when the forcing activity runs the
method, since an activity that holds a lock can enter it again ([the nesting
rule](r:conc.locks.nested)). Each example checks what was logged or counted if
the method began, and prints the same line if it never did.

```rexx
.environment~log = .array~new
.environment~go = .false
.environment~started = 0
x = .blocker~new
drop x
m = .worker~new~start('forcer')
do 600 until .environment~started > 0
  call syssleep 0.01
end
.environment~log~append('main goes on')
.environment~go = .true
say m~result
want = .array~of('uninit starts', 'main goes on', 'uninit ends', 'GC returns')
say 'the forcing activity waited for the method:',
  (.environment~started = 0 | .environment~log~makeString('l', '|') == want~makeString('l', '|'))
::class worker
::method forcer unguarded
  do 600 until .environment~started > 0
    call gc 'force'
  end
  .environment~log~append('GC returns')
  return 'worker done'
::class blocker
::method uninit
  .environment~log~append('uninit starts')
  .environment~started += 1
  do 300 until .environment~go
    call syssleep 0.01
  end
  .environment~log~append('uninit ends')
```

```output
worker done
the forcing activity waited for the method: 1
```

```rexx
.environment~started = 0
.environment~touched = 0
.environment~lk = .lk~new
x = .blocker~new
drop x
say .environment~lk~force
say 'the method used the lock that the forcing activity held:',
  (.environment~started = 0 | .environment~touched = 1)
::class lk
::method force
  do 300 until .environment~started > 0
    call gc 'force'
  end
  return 'GC returned'
::method touch
  .environment~touched += 1
::class blocker
::method uninit
  .environment~started += 1
  .environment~lk~touch
```

```output
GC returned
the method used the lock that the forcing activity held: 1
```

r[conc.halt.uninit.started]
An activity that an `UNINIT` method starts while the program runs, with
`start` or `REPLY`, is an ordinary one: the program does not end before it
has, as [the rule on the end](r:conc.halt.end.alive) says. In the example the
program collects until the method has begun, ends, and the message that the
method started, which waits for a gate that the program opens as its last
clause, writes its line after the main program has ended. When no collection
has sent `UNINIT` by then, the program starts the same message itself and
stops the method from starting a second one, so that the output does not
depend on [the invocation rule](r:env.invocation.uninit).

```rexx
.environment~started = 0
.environment~gate = .gate~new
x = .starter~new
drop x
do 600 until .environment~started > 0
  call gc 'force'
end
if .environment~started = 0 then do
  .environment~started = -1
  .message~new(.environment~gate, 'later')~start
end
say 'the main program ends'
.environment~gate~open
::class gate
::method init
  expose up
  up = .false
::method open unguarded
  expose up
  up = .true
::method later unguarded
  expose up
  guard on when up
  say 'the message that UNINIT started ran'
::class starter
::method uninit
  if .environment~started < 0 then return
  .environment~started += 1
  m = .message~new(.environment~gate, 'later')
  m~start
```

```output
the main program ends
the message that UNINIT started ran
```

r[conc.halt.uninit.once]
[The at-most-once rule](r:env.invocation.uninit) holds when several activities
call `GC('FORCE')` at the same time: an object is not sent `UNINIT` twice
because two of them find it. The method of the example waits a moment, so
that a second collection can find the object while the first has not ended
the method.

```rexx
.environment~count = 0
x = .counted~new
drop x
w = .forcer~new
a = w~start('run')
b = w~start('run')
c = w~start('run')
say a~result b~result c~result
say 'sent at most once:' (.environment~count <= 1)
::class forcer
::method run unguarded
  do 40
    call gc 'force'
  end
  return 'done'
::class counted
::method uninit
  .environment~count += 1
  call syssleep 0.01
```

```output
done done done
sent at most once: 1
```

## Deviations

r[conc.halt.deviation.10]
A HALT that a signal asks for ends the wait of an activity that waits for a
`GUARD ... WHEN` condition, for the post of an `EventSemaphore`, for a
`MutexSemaphore`, or in `SysSleep`. The clause that waited finishes, with
`wait` and `acquire` answering 0, and the activity takes the HALT at once
([Deviation 10](../deviations.md#10-signals-halt-and-wake)). A wait for the
result of a message is not ended. A `halt` of a message ends no wait ([the
rule](r:conc.halt.pending)). ooRexx ends none of these waits for a signal.
The activity goes on waiting until its condition holds, the post or the
release comes, or the time is up, and then takes the HALT, so that in the
example, where nothing posts, releases or opens, it waits for ever.

```rexx
f = SysTempFileName('conc-halt-????.rex')
do line over .resources~child
  call lineout f, line
end
call lineout f
address system 'rexx' f with output stem out.
do i = 1 to out.0
  say out.i
end
call SysFileDelete f
::resource child
box = .box~new
box~mx~acquire
a = box~start('byGuard')
b = box~start('byEvent')
c = box~start('byMutex')
d = box~start('byResult')
e = box~start('bySleep')
box~allWaiting
signal on halt
address system 'kill -INT $PPID'
do forever
  nop
end
halt:
say 'main took' condition('c')
box~finish
say a~result
say b~result
say c~result
say d~result
say e~result
::class box
::method init
  expose up fin ev mx marks helper
  up = .false
  fin = .false
  ev = .EventSemaphore~new
  mx = .MutexSemaphore~new
  marks = .array~new
  helper = .message~new(self, 'slow')
  helper~start
::method arm unguarded
  expose marks
  use arg name
  return 60 + 0 * marks~append(name)
::method idle unguarded
  expose marks
  return marks~append('guard') = 0
::method allWaiting unguarded
  expose marks
  do until marks~items = 5
    call syssleep 0.01
  end
::method finish unguarded
  expose fin
  fin = .true
::method ev unguarded
  expose ev
  return ev
::method mx unguarded
  expose mx
  return mx
::method slow unguarded
  expose fin
  signal on halt
  guard on when fin
  return 'the message ended'
halt:
  guard on when fin
  return 'the message ended after the HALT'
::method byGuard unguarded
  expose up
  signal on halt
  guard on when up | self~idle
  return 'not reached'
halt:
  return 'the guard wait ended with' condition('c')
::method byEvent unguarded
  expose ev
  signal on halt
  got = ev~wait(self~arm('event'))
  return 'not reached'
halt:
  return 'the event wait answered' got 'and ended with' condition('c')
::method byMutex unguarded
  expose mx
  signal on halt
  got = mx~acquire(self~arm('mutex'))
  return 'not reached'
halt:
  return 'the mutex wait answered' got 'and ended with' condition('c')
::method byResult unguarded
  expose helper
  signal on halt
  got = .array~of(helper, self~arm('result'))~at(1)~result
  return 'not reached'
halt:
  return 'the result wait answered' got 'and ended with' condition('c')
::method bySleep unguarded
  signal on halt
  call syssleep self~arm('sleep')
  return 'not reached'
halt:
  return 'the sleep ended with' condition('c')
::END
```

```output
main took HALT
the guard wait ended with HALT
the event wait answered 0 and ended with HALT
the mutex wait answered 0 and ended with HALT
the result wait answered the message ended after the HALT and ended with HALT
the sleep ended with HALT
```

> **Note.** As in the examples of [the signal rule](r:conc.halt.signal), a
> second interpreter runs the program that sends the signal; its text is the
> `::RESOURCE`. The handler of the main program does not post, release or open
> anything. It only lets the message end that the fourth activity waits for,
> which traps HALT as well, since the signal reaches it too, and waits on for
> its flag: the wait for its result is not one that the signal ended. In
> ooRexx that program writes its first line and then waits for ever for the
> result of the first message.

r[conc.halt.deviation.21]
The program does not wait for an activity that an `UNINIT` method starts
when the interpreter sends it at the end of the main program: a message that
the method starts, and the rest of a method that replies after `UNINIT`
called it, are not waited for, and the lines of the example that they would
write are not written. ooRexx does not cut them off, and often runs them, in
an order that varies ([Deviation
21](../deviations.md#21-activities-at-program-end)). The case is that of the
interpreter's own end of the program, for which `UNINIT` is not specified
([the invocation rule](r:env.invocation.end)); [the rule
above](r:conc.halt.uninit.started) holds for an `UNINIT` sent during the run.
Nor does interrexx wait for the other activities when the main program ends
with a refusal of one of the Deviations, exit code 120: they are cut off. In
the second example a started message waits for a gate that the program opens
just before the refusal, and never writes its line.

```rexx
x = .keeper~new
say 'the main program ends'
::class keeper
::method uninit
  say 'UNINIT ran'
  m = .message~new(self, 'later')
  m~start
  self~earlyReply
::method later unguarded
  say 'the message that UNINIT started ran'
::method earlyReply unguarded
  reply 'early'
  say 'the rest of the method after REPLY ran'
```

```output
the main program ends
UNINIT ran
```

```rexx
g = .gate~new
m = g~start('later')
say 'main: before the refusal'
g~open
say upper(.weird~new)
::class gate
::method init
  expose up
  up = .false
::method open unguarded
  expose up
  up = .true
::method later unguarded
  expose up
  guard on when up
  say 'the started message ran'
::class weird
::method string
  return .directory~new
```

```output rc=120
main: before the refusal
```

r[conc.halt.deviation.162]
An `init` that is sent to an `Alarm` or a `Ticker` after the one that `new`
sent raises `SYNTAX 98.915`, the error of a message that an object cannot take
in its state, and leaves the object as it was: an alarm or ticker that is set
stays set, and one that was cancelled stays cancelled, with its first time,
interval, target and attachment ([Deviation
162](../deviations.md#162-a-second-init-of-an-alarm-or-a-ticker)). ooRexx sets
the object up again without stopping the first setting, and a ticker that
runs crashes it, so the examples are not run on ooRexx. interrexx does not
refuse the second `init` yet (#248). The first example sends a second `init`
to an alarm that is set, cancels it and sends a third; the second does the
same with a ticker. They print which target hears the `cancel`, and what the
object still answers. The target of the ticker cancels it when it is called,
so that the example ends where the second `init` is not refused yet.

```rexx
n1 = .note~new('first target')
n2 = .note~new('second target')
dt = .datetime~new~addSeconds(2)
a = .alarm~new(dt, n1, 'first alarm')
call try 'a~init(0.01, n2, "second alarm")'
say a~attachment (a~scheduledTime == dt) a~canceled
a~cancel
call try 'a~init(0.01, n2, "third alarm")'
say a~attachment (a~scheduledTime == dt) a~canceled
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class note inherit AlarmNotification
::method init
  expose name
  use arg name
::method triggered
  nop
::method cancel
  expose name
  use arg src
  say name 'was sent cancel for' src~attachment
```

```output defect=#248
a~init(0.01, n2, "second alarm") => 98.915
first alarm 1 0
first target was sent cancel for first alarm
a~init(0.01, n2, "third alarm") => 98.915
first alarm 1 1
```

```rexx
n1 = .note~new('first target')
n2 = .note~new('second target')
t = .ticker~new(7200, n1, 'first ticker')
call try 't~init(30, n2, "second ticker")'
say t~attachment t~interval t~canceled
t~cancel
call try 't~init(30, n2, "third ticker")'
say t~attachment t~interval t~canceled
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class note inherit AlarmNotification
::method init
  expose name
  use arg name
::method triggered
  use arg src
  src~cancel
::method cancel
  expose name
  use arg src
  say name 'was sent cancel for' src~attachment
```

```output defect=#248
t~init(30, n2, "second ticker") => 98.915
first ticker 02:00:00.000000 0
first target was sent cancel for first ticker
t~init(30, n2, "third ticker") => 98.915
first ticker 02:00:00.000000 1
```
