---
element: PULL
kind: instruction
syntax: "PULL [template_list]"
ansi: "6.3.2.77, 8.3.19"
rexxref: "Keyword Instructions > PULL"
---

# PULL

PULL takes a line from the external data queue, makes it upper case and
parses it. It is a short form of [PARSE](parse.md): `PULL template_list` does
what `PARSE UPPER PULL template_list` does. Lines get into the queue with
[PUSH](push.md) and [QUEUE](queue.md), and the builtins
[QUEUED](../builtins/queued.md) and [RXQUEUE](../builtins/rxqueue.md) count
the lines and choose the queue.

> **Note.** The examples create a queue of their own, make it the current
> queue, and delete it when they are done, so they do not depend on the
> `SESSION` queue. The standard input of the examples is empty.

r[instr.pull.short]
`PULL` followed by a template list is the instruction `PARSE UPPER PULL`
followed by the same list. The line is made upper case before the template
sees it, as [the case conversion](r:parse.multiple.convert) defines, so the
patterns of the template are written in upper case, and the case of the line
that was in the queue is not kept. A PULL with no template
still takes the line out of the queue and assigns nothing.

```rexx
q = rxqueue('Create', 'spec_instr_pull_short')
call rxqueue 'Set', q
queue 'Mixed Case=Value'
queue 'another line'
queue 'third'
pull a '=' b
say a '|' b queued()
pull
say queued()
parse upper pull c
say c queued()
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', q
```

```output
MIXED CASE | VALUE 2
1
THIRD 0
```

r[instr.pull.source]
PULL takes the line at the head of the current queue and removes it, so lines
come out in the order in which [PUSH](r:instr.push.lifo) and
[QUEUE](r:instr.queue.fifo) arranged them. Only when the queue holds no line
does PULL read the next line of the default input stream, as
[PARSE LINEIN](r:instr.parse.linein) does, and the end of that input ends
the same way. A line that is the null string is a line: a PULL that takes it
does not read the input.

```rexx
q = rxqueue('Create', 'spec_instr_pull_source')
call rxqueue 'Set', q
call on notready name nr
queue ''
queue 'x'
pull a
say '[' || a || ']' queued()
pull a
say '[' || a || ']' queued()
pull a
say '[' || a || ']' queued()
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', q
exit
nr: say 'notready' condition('d')
    return
```

```output
[] 1
[X] 0
notready STDIN
[] 0
```

r[instr.pull.current]
The queue is the current queue, the one that [RXQUEUE](r:bif.rxqueue.set)
last chose, or `SESSION` if none was chosen. A current queue that does not
exist counts as one with no line: PULL raises no error, and reads the default
input stream. The lines of another queue stay where they are.

```rexx
a = rxqueue('Create', 'spec_instr_pull_a')
b = rxqueue('Create', 'spec_instr_pull_b')
call rxqueue 'Set', a
queue 'in a'
call rxqueue 'Set', b
queue 'in b'
pull x
say x queued()
call rxqueue 'Set', a
pull x
say x queued()
call rxqueue 'Delete', a
say queued()
pull x
say '[' || x || ']'
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', b
```

```output
IN B 0
IN A 0
0
[]
```
