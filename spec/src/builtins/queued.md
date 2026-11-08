---
element: QUEUED
kind: builtin
syntax: "QUEUED()"
ansi: "9.8.2"
rexxref: "Functions > Built-in Functions > QUEUED"
---

# QUEUED

QUEUED answers how many lines the current external data queue holds. It
takes no line out of the queue and puts none in.

The external data queue is a list of lines kept outside the program, where the
[QUEUE](r:instr.queue.fifo) and [PUSH](r:instr.push.lifo) instructions put
lines and the [PULL](r:instr.pull.source) and [PARSE PULL](r:instr.parse.pull)
instructions take them out. A program has one current queue at a time, which
[RXQUEUE](rxqueue.md) names and changes. The queue called `SESSION` is the
current queue when a program starts.

> **Note.** The examples create a queue of their own, make it the current
> queue, and delete it when they are done, so they leave nothing behind and do
> not depend on what the `SESSION` queue holds when they start. A queue
> can be reached by other programs, and the lines such a program puts into
> or takes out of it are not specified.

## Arguments

r[bif.queued.args]
QUEUED takes no argument. A call with an argument raises `SYNTAX 40.4`, by
the [call rules](r:bif.index.call.too-many). A trailing omitted place adds
nothing to the count, so `QUEUED(,)` is a call without an argument. QUEUED
does not run as a String method: sending the message `QUEUED` to a string
raises `SYNTAX 97.1`.

```rexx
signal on syntax name a
say queued(1)
a: say condition('o')~code
signal on syntax name b
say queued(, 'x')
b: say condition('o')~code
signal on syntax name c
say 'x'~queued
c: say condition('o')~code
signal off syntax
say queued(,) == queued()
```

```output
40.4
40.4
97.1
1
```

## The result

r[bif.queued.count]
QUEUED answers the number of lines in the current queue, written as [a
count](r:bif.index.count.answer). Every [QUEUE](r:instr.queue.fifo) or
[PUSH](r:instr.push.lifo) instruction adds one line, and a line that is the
null string is a line. Reading a line with [PULL](r:instr.pull.source), [PARSE
PULL](r:instr.parse.pull) or `LINEIN('QUEUE:')` removes it, and a queue with
no line left answers `0`.

```rexx
q = rxqueue('Create', 'spec_queued_count')
call rxqueue 'Set', q
say queued()
queue 'one'
queue ''
push 'zero'
say queued()
parse pull a
say a queued()
say linein('queue:') '[' || linein('queue:') || ']' queued()
do 1234
  queue 'x'
end
numeric digits 3
say queued() queued() + 0
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', q
```

```output
0
3
zero 2
one [] 0
1234 1.23E+3
```

r[bif.queued.current]
QUEUED counts the lines of the current queue only, so setting another queue
as the current queue with [RXQUEUE](rxqueue.md) changes the answer. When the
current queue is one that does not exist, because it was never created or
because it was deleted, QUEUED answers `0` and raises no condition.

```rexx
a = rxqueue('Create', 'spec_queued_a')
b = rxqueue('Create', 'spec_queued_b')
call rxqueue 'Set', a
queue 'x'
queue 'y'
call rxqueue 'Set', b
queue 'z'
say queued()
call rxqueue 'Set', a
say queued()
call rxqueue 'Delete', a
say queued()
call rxqueue 'Set', 'spec_queued_none'
say queued()
call rxqueue 'Set', b
say queued()
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', b
```

```output
1
2
0
0
1
```
