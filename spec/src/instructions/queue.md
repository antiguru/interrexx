---
element: QUEUE
kind: instruction
syntax: "QUEUE [expression]"
ansi: "6.3.2.79, 8.3.21"
rexxref: "Keyword Instructions > QUEUE"
---

# QUEUE

QUEUE puts one line on the external data queue, at the tail. The next
[PULL](pull.md) or `PARSE PULL` takes the line that has waited longest, so
the queue works as a line of waiting lines. [PUSH](push.md) is the
instruction that puts a line at the other end. The queue is the current one,
which the builtin [RXQUEUE](../builtins/rxqueue.md) chooses and
[QUEUED](../builtins/queued.md) counts.

> **Note.** The examples create a queue of their own, make it the current
> queue, and delete it when they are done, so they do not depend on the
> `SESSION` queue.

r[instr.queue.fifo]
QUEUE adds a line at the tail of the current queue, behind every line that is
there. Lines queued one after another come out in the same order, first in
and first out. A line that a PUSH put at the head comes out before them, and a
line that is the null string is a line like any other.

```rexx
q = rxqueue('Create', 'spec_instr_queue_fifo')
call rxqueue 'Set', q
queue 'one'
queue 'two'
queue ''
push 'head'
queue 'three'
say queued()
do while queued() > 0
  parse pull line
  say '[' || line || ']'
end
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', q
```

```output
5
[head]
[one]
[two]
[]
[three]
```

r[instr.queue.value]
The expression is everything after the keyword. QUEUE evaluates it once and
puts the string value of the result on the queue as one line, without any
conversion or trimming. Without an expression the line is the null string. A
string with a line feed is one line. If the evaluation raises a condition,
nothing is queued. QUEUE sets no variable, and leaves `RC` and `RESULT` as
they were. [PUSH](r:instr.push.value) says the same of the head of the queue.

```rexx
q = rxqueue('Create', 'spec_instr_queue_value')
call rxqueue 'Set', q
result = 'kept'
numeric digits 3
queue 1 / 3 'and' 2 * 5
queue 'a' || '0a'x || 'b'
queue
queue ' Mixed  Case '
say queued() result
parse pull a
say '[' || a || ']'
pull a
say c2x(a)
parse pull a
say '[' || a || ']'
parse pull a
say '[' || a || ']'
signal on novalue name nv
queue unset_name
say 'not reached'
nv: say condition('c') queued()
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', q
```

```output
4 kept
[0.333 and 10]
410A42
[]
[ Mixed  Case ]
NOVALUE 0
```

r[instr.queue.queue]
The queue that receives the line is the current one at the time of the
instruction, `SESSION` unless the builtin [RXQUEUE](r:bif.rxqueue.set) chose
another. If that queue does not exist, QUEUE raises `SYNTAX 48.1` and the line
is lost. A fault in the expression is the error of the expression, such as
`35.1`.

```rexx
a = rxqueue('Create', 'spec_instr_queue_a')
b = rxqueue('Create', 'spec_instr_queue_b')
call rxqueue 'Set', a
queue 'for a'
call rxqueue 'Set', b
say queued()
queue 'for b'
say queued()
call rxqueue 'Set', a
say queued()
call rxqueue 'Set', 'spec_instr_queue_gone'
signal on syntax name e
queue 'lost'
say 'not reached'
e: say condition('o')~code queued()
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', a
call rxqueue 'Delete', b
call try 'queue 1 +'
call try 'queue (1'
exit
try:
  signal on syntax name e2
  interpret arg(1)
  return
e2: say arg(1) '=>' condition('o')~code
  return
```

```output
0
1
1
48.1 0
queue 1 + => 35.1
queue (1 => 36.901
```
