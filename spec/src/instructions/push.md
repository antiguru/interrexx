---
element: PUSH
kind: instruction
syntax: "PUSH [expression]"
ansi: "6.3.2.78, 8.3.20"
rexxref: "Keyword Instructions > PUSH"
---

# PUSH

PUSH puts one line on the external data queue, at the head. The next
[PULL](pull.md) or `PARSE PULL` takes the line that was pushed last, so the
queue works as a stack. [QUEUE](queue.md) is the instruction that puts a line
at the other end. The queue is the current one, which the builtin
[RXQUEUE](../builtins/rxqueue.md) chooses and [QUEUED](../builtins/queued.md)
counts.

> **Note.** The examples create a queue of their own, make it the current
> queue, and delete it when they are done, so they do not depend on the
> `SESSION` queue.

r[instr.push.lifo]
PUSH adds a line at the head of the current queue, in front of every line that
is there. Lines pushed one after another come out in the opposite order, last
in and first out, and they come out before any line that QUEUE put at the
tail. A line that is the null string is a line like any other.

```rexx
q = rxqueue('Create', 'spec_instr_push_lifo')
call rxqueue 'Set', q
queue 'tail'
push 'one'
push 'two'
push ''
say queued()
do while queued() > 0
  parse pull line
  say '[' || line || ']'
end
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', q
```

```output
4
[]
[two]
[one]
[tail]
```

r[instr.push.value]
The expression is everything after the keyword. PUSH evaluates it once and
puts the string value of the result on the queue as one line, and nothing is
converted or trimmed, so a line keeps its blanks and its case. Without an
expression the line is the null string. A string that holds a line feed or
other control characters is still one line, and it comes back whole. If the
evaluation raises a condition, nothing is pushed. PUSH sets no variable, and
leaves `RC` and `RESULT` as they were.

```rexx
q = rxqueue('Create', 'spec_instr_push_value')
call rxqueue 'Set', q
numeric digits 3
push 1 / 3
push 'x' || '0a'x || 'y'
push
push '  Mixed  Case  '
say queued()
parse pull a
say '[' || a || ']'
parse pull a
say '[' || a || ']'
say queued()
pull a
say c2x(a)
signal on novalue name nv
push unset_name
say 'not reached'
nv: say condition('c') queued()
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', q
```

```output
4
[  Mixed  Case  ]
[]
2
580A59
NOVALUE 1
```

r[instr.push.queue]
The queue that receives the line is the current one at the time of the
instruction. It is `SESSION` when the program starts, until the builtin
[RXQUEUE](r:bif.rxqueue.set) chooses another. If the current queue does not
exist, PUSH raises `SYNTAX 48.1` and the line is lost. A fault in the
expression is the error of the expression, such as `35.1`.

```rexx
say rxqueue('Get')
a = rxqueue('Create', 'spec_instr_push_a')
b = rxqueue('Create', 'spec_instr_push_b')
call rxqueue 'Set', a
push 'for a'
call rxqueue 'Set', b
say queued()
push 'for b'
say queued()
call rxqueue 'Set', a
say queued()
call rxqueue 'Set', 'spec_instr_push_gone'
signal on syntax name e
push 'lost'
say 'not reached'
e: say condition('o')~code queued()
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', a
call rxqueue 'Delete', b
call try 'push 1 +'
call try 'push (1'
exit
try:
  signal on syntax name e2
  interpret arg(1)
  return
e2: say arg(1) '=>' condition('o')~code
  return
```

```output
SESSION
0
1
1
48.1 0
push 1 + => 35.1
push (1 => 36.901
```
