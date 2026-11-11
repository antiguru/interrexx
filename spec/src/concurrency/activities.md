---
element: "Activities"
kind: concept
rexxref: "Concurrency; Concurrency > Early Reply; Concurrency > Message Objects"
deviations: [20]
---

# Activities

A program is run by an activity: a thread of control that runs the clauses of
the program one after another, keeps the calls it has made and gives each call
its own variables. A program starts with one activity. It gets more by asking
for them: it starts a message, replies early from a method, or sets an alarm
or a ticker. The activities that exist at one time run side by side. They
share every object they can reach, and each keeps its own calls, its own
variables and its own traps. This page specifies what an activity is, where
each kind of new activity comes from and which code it runs, how the answer or
the error of a message that another activity ran reaches the program, and what
a program may count on about the order in which activities do things.

The lock that an object has, the methods that are guarded or unguarded and the
`GUARD` instruction are specified on the locks page of this chapter, and
`HALT` and the end of a program that still has activities on the halt page;
this page uses `GUARD ... WHEN` only as the way a program makes one activity
wait for another. The methods of `Message`, `Alarm` and `Ticker` and the
instruction `REPLY` are specified on their own pages, which this page links:
it states what they do to activities, and not what they check or answer.

The examples force the order of everything they print. An activity that has to
wait for the program does so on a `GUARD ... WHEN` that the program opens when
it is ready, and a program that needs a result asks for it with `result` or
`wait`. A method that another activity waits for, or that waits for the
program, is `UNGUARDED`, so that the program can still use the object
meanwhile ([the exclusion rule](r:instr.guard.exclusion)).

## What an activity is

r[conc.activities.model]
An activity runs one clause at a time, in the order the program gives. Two
activities share the objects they can both reach: an object that one of them
changes is changed for the other, and so are the entries of `.local` and
`.environment`. They do not share the variables of a call. The variables of a
method or routine call belong to the activity that made the call, so a method
that another activity runs starts with its own variables, even when the same
name is set in the code that started it. A condition trap is set for the
activity that ran the `SIGNAL ON` or `CALL ON`, and does not catch a condition
of another.

```rexx
shared = .directory~new
.local~put('orig', 'SHARED')
.environment~put('orig', 'ENVSHARED')
loc = 'main variable'
m = .worker~new~start('work', shared)
m~wait
say shared~at('seen') '|' shared~at('own')
say .local~at('SHARED') '|' .environment~at('ENVSHARED') '|' loc
::class worker
::method work unguarded
  use arg dir
  dir~put('written by the other activity', 'seen')
  dir~put(symbol('loc'), 'own')
  say 'the other activity read:' .local~at('SHARED')
  .local~put('changed', 'SHARED')
  .environment~put('changed', 'ENVSHARED')
  loc = 'worker variable'
```

```output
the other activity read: orig
written by the other activity | LIT
changed | changed | main variable
```

```rexx
signal on syntax name caught
m = .worker~new~start('fail')
m~wait
say 'no trap taken here; the error is the message''s:' m~errorCondition~code
exit
caught: say 'trapped in the main activity:' condition('o')~code
::class worker
::method fail unguarded
  return 1/0
```

```output
no trap taken here; the error is the message's: 42.3
```

## Where activities come from

r[conc.activities.creation]
A program makes a new activity by each of the following, and by nothing else
(the activity that runs the `UNINIT` methods of objects is not one a program
makes, and belongs to the halt page of this chapter):
[`start`](r:class.object.start) and `startWith` of `Object`, and
[`start`](r:class.message.start) and `startWith` of a `Message`, which run the
message on a new activity; [`reply`](r:class.message.reply) and `replyWith` of
a `Message`, which run a copy of the message on a new activity; the
[`REPLY`](r:instr.reply.early) instruction, which runs the rest of a method on
a new activity; an [`Alarm`](r:class.alarmnotification.triggered) or a
[`Ticker`](r:class.ticker.triggered), which sends its target `triggered` on an
activity of the library; and a message object that receives `messageComplete`
or `triggered`, which runs itself on a new activity ([the
rule](r:class.message.messagecomplete)). The call that makes the activity does
not wait for the code the activity runs: it returns, and that code runs next
to the program. `send` and `sendWith` of a `Message` and an ordinary message
to an object are not among the list. The method they run is run by the
activity that sent the message, which goes on when the method has ended.

```rexx
log = .array~new
g = .gate~new(log)
g~start('viaStart', 'start')
log~append('returned start')
call proceed 1
.message~new(g, 'viaStart', 'I', 'message start')~start
log~append('returned message start')
call proceed 2
g~startWith('viaStart', .array~of('startWith'))
log~append('returned startWith')
call proceed 3
.message~new(g, 'viaStart', 'I', 'message reply')~reply
log~append('returned message reply')
call proceed 4
g~viaReply
log~append('returned REPLY')
call proceed 5
a = .alarm~new(0.01, g, 'alarm')
log~append('returned alarm')
call proceed 6
t = .ticker~new(0.01, g, 'ticker')
log~append('returned ticker')
call proceed 7
g~open
.message~new(g, 'viaStart', 'I', 'send')~send
log~append('returned send')
do line over log
  say line
end
exit
proceed:
  g~open
  g~wait(arg(1))
  g~close
  return
::class gate inherit AlarmNotification
::method init
  expose log up done
  use arg log
  up = .false
  done = 0
::method open unguarded
  expose up
  up = .true
::method close unguarded
  expose up
  up = .false
::method wait unguarded
  expose done
  use arg k
  guard on when done >= k
::method viaStart unguarded
  expose log up done
  guard on when up
  log~append('ran' arg(1))
  done = done + 1
::method viaReply unguarded
  expose log up done
  reply 'early'
  guard on when up
  log~append('ran REPLY')
  done = done + 1
::method triggered unguarded
  expose log up done
  use arg src
  guard on when up
  log~append('ran' src~attachment)
  done = done + 1
  if src~attachment == 'ticker' then src~cancel
::method cancel unguarded
  nop
```

```output
returned start
ran start
returned message start
ran message start
returned startWith
ran startWith
returned message reply
ran message reply
returned REPLY
ran REPLY
returned alarm
ran alarm
returned ticker
ran ticker
ran send
returned send
```

> **Note.** The example parks the code of each new activity on a gate that the
> program opens only after the call that made the activity has returned, so
> each line that starts with `ran` follows the line that starts with
> `returned` for the same call. The last line shows that a sent message has
> run when `send` returns.

r[conc.activities.timers]
An alarm and a ticker run their target's `triggered` on an activity of the
library, and never on the activity that made them, which can be waiting for
the target meanwhile. The activities of different alarms and tickers are
different: a `triggered` that is still running for one alarm does not delay
that of another. The calls that one ticker makes are one after the other and
in order: the ticker sends `triggered` again only after the call before it has
returned, so a target that stays inside `triggered` delays the next one, and
how many are sent after it returns is not specified. When a program cancels an
alarm or a ticker that is still waiting, the target is sent `cancel` on the
activity that sent `cancel` to the alarm or ticker, and the sender of `cancel`
goes on when the target's method has returned ([the cancel
rule](r:class.alarmnotification.cancel)).

```rexx
n = .note~new
a = .alarm~new(0.01, n, 'first')
b = .alarm~new(0.01, n, 'second')
n~awaitBoth
say 'the first alarm waited for the second, so they ran side by side'
::class note inherit AlarmNotification
::method init
  expose second
  second = .false
::method triggered unguarded
  expose second
  use arg al
  if al~attachment == 'first' then guard on when second
  else second = .true
::method cancel unguarded
  nop
::method awaitBoth unguarded
  expose second
  guard on when second
```

```output
the first alarm waited for the second, so they ran side by side
```

```rexx
n = .slow~new
t = .ticker~new(0.01, n)
n~awaitFirst
call syssleep 0.3
say 'while the first call has not returned:' n~count
n~release
n~awaitSecond
t~cancel
say 'after it returned:' (n~count >= 2)
::class slow inherit AlarmNotification
::method init
  expose count go
  count = 0
  go = .false
::method count unguarded
  expose count
  return count
::method triggered unguarded
  expose count go
  count = count + 1
  if count = 1 then guard on when go
::method cancel unguarded
  nop
::method release unguarded
  expose go
  go = .true
::method awaitFirst unguarded
  expose count
  guard on when count >= 1
::method awaitSecond unguarded
  expose count
  guard on when count >= 2
```

```output
while the first call has not returned: 1
after it returned: 1
```

```rexx
n = .note~new
a = .alarm~new(60, n, 'alarm')
a~cancel
say 'after the alarm''s cancel returned:' n~seen~strip
t = .ticker~new(60, n, 'ticker')
t~cancel
say 'after the ticker''s cancel returned:' n~seen~strip
::class note inherit AlarmNotification
::method init
  expose seen
  seen = ''
::method seen
  expose seen
  return seen
::method triggered
  nop
::method cancel
  expose seen
  use arg src
  seen = seen src~attachment
```

```output
after the alarm's cancel returned: alarm
after the ticker's cancel returned: alarm ticker
```

> **Note.** The first example has the target of the first alarm wait for the
> second alarm's call: it ends only because the two calls are on different
> activities. The second has a ticker's target stay in its first call; the
> program waits a while, and the ticker has not made a second call. The third
> shows that the target has been sent `cancel` when `cancel` returns, for an
> alarm and for a ticker.

## Messages and their results

r[conc.activities.result]
A message object is how the program keeps hold of what another activity does.
The activity that runs a started message holds nothing for its starter: the
object records what the message answered, and the program asks for it when it
wants it. [`result`](r:class.message.result) waits until the message has ended
and then answers; [`wait`](r:class.message.wait) waits and answers nothing;
`completed`, `hasResult`, `hasError` and `errorCondition` do not wait and
answer what is known when they are asked. Any number of activities may ask for
the same result, and each gets it, as the third example of [the order
rule](r:conc.activities.order) shows. A method that ends with `REPLY` ends the
message at the `REPLY`: the value of the `REPLY` is the result of the message,
`completed` answers 1 from then on, and the rest of the method runs on as an
activity that no message object waits for. The page of
[`Message`](r:class.message.start) specifies the methods; the examples show
what is common to them.

```rexx
o = .t~new
m = o~start('work')
say m~result
say m~completed m~hasResult m~hasError
o~release
o~await
say m~completed m~result
q = .t~new~start('replyOnly')
say q~result q~completed
::class t
::method init
  expose go done
  go = .false
  done = .false
::method release unguarded
  expose go
  go = .true
::method await unguarded
  expose done
  guard on when done
::method work unguarded
  expose go done
  reply 'early'
  guard on when go
  done = .true
::method replyOnly unguarded
  reply 'only'
```

```output
early
1 1 0
1 early
only 1
```

```rexx
log = .array~new
t = .team~new(log)
m1 = t~start('run', 1)
m2 = t~start('run', 2)
m3 = t~start('run', 3)
say m1~result m2~result m3~result
say 'finished in the order' log~makeString('l', ' ')
::class team
::method init
  expose log turn
  use arg log
  turn = 1
::method run unguarded
  expose log turn
  use arg k
  guard on when turn = 4 - k
  log~append(k)
  turn = turn + 1
  return 'result' k
```

```output
result 1 result 2 result 3
finished in the order 3 2 1
```

> **Note.** The first example asks for a message that replied early while its
> continuation is still waiting, and for one whose method does nothing after
> its `REPLY`. The second starts three messages that finish in the opposite
> order and asks for the results in the order of the starts.

r[conc.activities.error]
An error in the code of a started message does not reach the activity that
started it when it happens: nothing is raised at the call to `start`, a trap
in the starter is not taken, and the starter goes on. The error ends the
message, which keeps it: `hasError` answers 1 and `errorCondition` the
condition. It reaches the program when the program asks for
[`result`](r:class.message.result), which raises it again in the activity that
asked, with the same code, so that a trap in that activity handles it and a
trap in any other activity does not see it. Each activity that asks gets its
own raise, and asking again raises it again. `wait` returns without raising,
whether or not the message ended with an error. The errors that end a message
are those of [`hasError`](r:class.message.haserror).

```rexx
m = .t~new~start('boom')
a = .asker~new~start('ask', m)
b = .asker~new~start('ask', m)
say a~result '|' b~result
m~wait
say 'wait raised nothing:' m~hasError m~errorCondition~code
signal on syntax name c
x = m~result
say 'not reached'
c: say 'main:' condition('o')~code
::class t
::method boom unguarded
  return 1/0
::class asker
::method ask unguarded
  use arg msg
  signal on syntax name c
  x = msg~result
  return 'no error'
c: return 'raised' condition('o')~code
```

```output
raised 42.3 | raised 42.3
wait raised nothing: 1 42.3
main: 42.3
```

> **Note.** The example has two activities ask for the result of a message
> that fails, each with a trap of its own, and then the program asks.

r[conc.activities.unasked]
An error of a started message that nothing asks about ends that message's
activity and nothing else: the program, and every other activity, goes on, and
the program's exit status is the one the program itself gives. The activity
that ran the message reports the error on the error stream when the message
ends, whether or not a later `result` raises it, and the text of the report is
not specified (the line it names is the case of [Deviation
67](../deviations.md#67-the-line-a-started-messages-error-names)). An error in
the continuation of a method that has replied ends the continuation in the
same way and is never an error of the message, which has ended with the
`REPLY`. A condition that is not an error, such as a `USER` condition that
nothing traps, ends the message without an error ([the
rule](r:class.message.haserror)).

```rexx
o = .t~new
m = o~start('boom')
m~wait
say 'after the error:' m~completed m~hasError
u = o~start('usr')
u~wait
say 'user condition:' u~completed u~hasError u~hasResult
r = o~start('late')
say r~result
o~release
o~await
say 'continuation error:' r~completed r~hasError
say 'the program goes on'
exit 3
::class t
::method init
  expose go done
  go = .false
  done = .false
::method release unguarded
  expose go
  go = .true
::method await unguarded
  expose done
  guard on when done
::method boom unguarded
  return 1/0
::method usr unguarded
  raise user myproblem
::method late unguarded
  expose go done
  reply 'replied'
  guard on when go
  done = .true
  x = 1/0
```

```output rc=3
after the error: 1 1
user condition: 1 0 0
replied
continuation error: 1 0
the program goes on
```

> **Note.** The example ends with the exit status 3 that it gives itself,
> after a message failed, one raised a `USER` condition, and a continuation
> failed.

## Order

r[conc.activities.order]
Two things done by different activities happen in a known order only when the
second activity can learn that the first has done its part. The ways it learns
are these, and nothing else orders two activities. One is the order of the
clauses of one activity: what an activity did before it started a message is
done before the first clause of that message runs, and what a message did is
done when `result` or `wait` returns for it, and before the targets of its
notifications are sent. Another is a `GUARD ... WHEN` whose condition is
false, which does not go on until another activity has made the condition
true, and what that activity did before is done when the first goes on. The
third is a wait on an `EventSemaphore`, which ends when another activity posts
it, and a `MutexSemaphore`, which is acquired only after its owner has
released it: what the other activity did before the post or the release is
done. Anything else is not ordered. The clauses of activities that nothing
orders may run in any interleaving that keeps the order of each activity's own
clauses, and which one runs may differ between runs and between
implementations ([Deviation
22](../deviations.md#22-one-schedule-for-racing-activities), and [Deviation
7](../deviations.md#7-order-of-concurrent-trace-lines) for trace lines). That
one message was started before another does not make it run first.

```rexx
log = .array~new
p = .pair~new(log)
a = p~start('play', 'ping', 'pong')
b = p~start('play', 'pong', 'ping')
a~wait; b~wait
say log~makeString('l', ' ')
::class pair
::method init
  expose log turn
  use arg log
  turn = 'ping'
::method play unguarded
  expose log turn
  use arg me, other
  do i = 1 to 3
    guard on when turn = me
    log~append(me || i)
    turn = other
    guard off
  end
```

```output
ping1 pong1 ping2 pong2 ping3 pong3
```

```rexx
log = .array~new
t = .writer~new(log)
a = t~start('put', 'a')
b = t~start('put', 'b')
a~wait; b~wait
say log~items
say (log~index('a2') > log~index('a1')) (log~index('a3') > log~index('a2')),
  (log~index('b2') > log~index('b1')) (log~index('b3') > log~index('b2'))
::class writer
::method init
  expose log
  use arg log
::method put unguarded
  expose log
  use arg tag
  do i = 1 to 3
    log~append(tag || i)
  end
```

```output
6
1 1 1 1
```

```rexx
box = .box~new
m = box~start('reader')
box~store('written before the post')
box~ready~post
say m~result
box~lock~acquire
w = box~start('taker')
box~store('written before the release')
box~lock~release
say w~result
a = .asker~new~start('ask', m)
b = .asker~new~start('ask', m)
say a~result
say b~result
::class box
::method init
  expose ready value lock
  ready = .EventSemaphore~new
  lock = .MutexSemaphore~new
  value = 'nothing yet'
::method ready unguarded
  expose ready
  return ready
::method lock unguarded
  expose lock
  return lock
::method store unguarded
  expose value
  use arg value
::method reader unguarded
  expose ready value
  ready~wait
  return 'the reader saw:' value
::method taker unguarded
  expose lock value
  lock~acquire
  r = 'the taker saw:' value
  lock~release
  return r
::class asker
::method ask unguarded
  use arg msg
  return msg~result
```

```output
the reader saw: written before the post
the taker saw: written before the release
the reader saw: written before the post
the reader saw: written before the post
```

> **Note.** The first example orders two activities that each take three
> turns: each waits for its turn on a `GUARD ... WHEN` and then gives the turn
> to the other, so the log alternates, whichever activity runs first. The
> second has two activities that nothing orders write three entries each: the
> log always has six entries, and the entries of each activity are in order.
> The third has the program store a value and then post a semaphore that a
> message waits for, store another and then release a mutex that another
> message acquires, and has two activities ask for the same result.

r[conc.activities.waiting]
An activity that waits does not hold up another. The waits are: `result` and
`wait` of a message, a `GUARD ... WHEN` whose condition is false, an
`EventSemaphore`'s `wait`, a `MutexSemaphore`'s `acquire` that has to wait,
and `SysSleep`; an activity in one of them runs no clauses until what it waits
for has happened, and every other activity that is not itself waiting makes
progress meanwhile. How fast each progresses, and whether the activities that
are not waiting share the time fairly, is not specified, and so is how soon an
activity whose wait has ended runs again. Some blocking operations of the
library hold every activity, and some busy-waiting code in nested constructs
does not end; they are the cases of [Deviation
16](../deviations.md#16-blocking-operations-hold-every-activity) and
[Deviation 18](../deviations.md#18-nested-busy-waiters).

```rexx
g = .gate~new
w = g~start('waiter')
say 'the program goes on while the waiter waits'
o = g~start('opener')
say w~result o~result
::class gate
::method init
  expose up
  up = .false
::method waiter unguarded
  expose up
  guard on when up
  return 'waited and passed'
::method opener unguarded
  expose up
  up = .true
  return 'opened'
```

```output
the program goes on while the waiter waits
waited and passed opened
```

> **Note.** The program prints a line and starts a second message while the
> first waits on a guard; the first can end only because the program and the
> second message ran while it waited.

## Deviations

r[conc.activities.deviation.20]
A wait that no other activity, alarm, ticker or call can end is refused, and
the program ends with exit code 120. It is the wait for the result of a
message that has never been sent, or that is the original of a `reply`, whose
copy ran; the same holds for `wait`. ooRexx waits for ever ([Deviation
20](../deviations.md#20-a-wait-nothing-can-end)). The case of a
`GUARD ... WHEN` that nothing can make true is the guard rule's.

```rexx
m = .message~new(.object~new, 'string')
c = m~reply
say 'a message that was never sent, and the original of a reply'
say 'waiting for the copy:' c~result
say 'waiting for the original:'
say m~wait
say 'not reached'
```

```output rc=120
a message that was never sent, and the original of a reply
waiting for the copy: an Object
waiting for the original:
```
