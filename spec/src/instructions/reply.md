---
element: REPLY
kind: instruction
syntax: "REPLY [expression]"
rexxref: "Keyword Instructions > REPLY"
deviations: [15]
---

# REPLY

REPLY ends a method call early. The caller gets the answer at once and goes
on, and the method goes on with the clauses after the REPLY, on an activity
of its own. It is how a method hands out a result and then does the slow part
of its work while the caller carries on. This page specifies what the caller
and the continuation each see, what the continuation may and may not do, the
places where REPLY may stand, and the errors.

The order in which the caller's later clauses and the continuation's clauses
run is not specified, unless a wait, a guard or the instruction itself puts
them in order. Every example below puts them in order, so that its output does
not depend on it.

## The early answer

r[instr.reply.early]
REPLY evaluates its expression and hands the value to whoever sent the
message, as RETURN does, and the caller continues with it. A REPLY with no
expression hands over no value, and the caller sees what it sees after a
RETURN with none, as [the message rule](r:instr.message.result) says. The
method does not end. The clause after the REPLY runs next, on a new
activity, with the variables of the method as they were, its arguments, and
its object variables. For a message sent with `~start`, the value of the
REPLY is the `~result` of the message object. The method call that sent the
message is over for the caller, so the method's later `RETURN` or `EXIT` has
no value to hand over.

```rexx
log = .array~new
o = .rep~new(log)
m = o~start('work', 'a1', 'a2')
log~append('caller got' m~result)
o~go
o~wait
say log~makeString('l', '|')
o~noval
say symbol('result')
exit
::class rep
::method init
  expose log go done
  use arg log
  go = .false; done = .false
::method go unguarded
  expose go
  go = .true
::method wait unguarded
  expose done
  guard on when done
::method work
  expose log go done
  use arg a, b
  loc = 'local'
  reply 'early'
  guard on when go
  log~append('cont' a b loc arg() self~class~id)
  done = .true
::method noval
  reply
```

```output
caller got early|cont a1 a2 local 2 REP
LIT
```

r[instr.reply.guard]
A method that holds its object's guard when it replies keeps it on the new
activity, so the continuation still has exclusive use of the object variables,
and a guarded method that another activity sends to the object waits until
the continuation has ended, as [the exclusion rule](r:instr.guard.exclusion)
of GUARD says.

```rexx
log = .array~new
o = .vault~new(log)
say o~holder
p = o~start('probe')
o~release
p~result
say log~makeString('l', ' ')
::class vault
::method init
  expose log go
  use arg log
  go = .false
::method holder
  expose log go
  reply 'replied'
  do until go; end
  log~append('continued')
::method probe
  expose log
  log~append('probe')
  return 'p'
::method release unguarded
  expose go
  go = .true
```

```output
replied
continued probe
```

## After the REPLY

r[instr.reply.once]
A method can execute one REPLY for each call. A second REPLY raises
`SYNTAX 98.935`. A RETURN with an expression after a REPLY raises
`SYNTAX 98.936`, and an EXIT with an expression `SYNTAX 98.937`, because the
value has nowhere to go. A RETURN or an EXIT without an expression ends the
continuation, and the method is over. These errors arise in the continuation,
so a trap in the method sees them. An error that no trap handles ends the
continuation only, with its message on standard error: the caller has the
answer already, and the program goes on.

```rexx
log = .array~new
o = .rep~new(log)
do m over .array~of('twice', 'retval', 'exitval', 'plain', 'plainexit')
  log~append(m 'got' o~send(m))
  o~go
  o~wait
end
say log~makeString('l', '|')
say o~lost
say 'main continues'
exit
::class rep
::method init
  expose log go done
  use arg log
  go = .false; done = .false
::method go unguarded
  expose go
  go = .true
::method wait unguarded
  expose done
  guard on when done
  done = .false
::method twice
  expose log go done
  signal on syntax name oops
  reply 'one'
  guard on when go
  go = .false
  reply 'two'
oops: log~append('twice' condition('o')~code)
  done = .true
::method retval
  expose log go done
  signal on syntax name oops
  reply 'one'
  guard on when go
  go = .false
  return 5
oops: log~append('retval' condition('o')~code)
  done = .true
::method exitval
  expose log go done
  signal on syntax name oops
  reply 'one'
  guard on when go
  go = .false
  exit 5
oops: log~append('exitval' condition('o')~code)
  done = .true
::method plain
  expose log go done
  reply 'one'
  guard on when go
  go = .false
  log~append('plain continued')
  done = .true
  return
::method plainexit
  expose log go done
  reply 'one'
  guard on when go
  go = .false
  log~append('plainexit continued')
  done = .true
  exit
::method lost
  reply 'lost answer'
  x = 1 / 0
```

```output
twice got one|twice 98.935|retval got one|retval 98.936|exitval got one|exitval 98.937|plain got one|plain continued|plainexit got one|plainexit continued
lost answer
main continues
```

r[instr.reply.value]
An error in the expression of the REPLY is raised before the method is
split: the caller gets the error, as it would for a RETURN, and no
continuation runs.

```rexx
o = .rep~new
signal on syntax name e
say o~bad
say 'not reached'
e: say 'caller trapped' condition('o')~code
::class rep
::method bad
  reply 1/0
  say 'the continuation must not run'
```

```output
caller trapped 42.3
```

## Where REPLY may appear

r[instr.reply.method]
REPLY works in the code of a method, anywhere among its own clauses, in a DO,
an IF or a SELECT as well. In the main program, in a routine of a `::ROUTINE`
directive and in an internal routine, including one that a method calls, it
raises `SYNTAX 99.919` when it runs. The text of an INTERPRET raises `SYNTAX
99.924` when the INTERPRET runs, so no clause of the text runs.

```rexx
o = .box~new
say 'inside =>' o~inside
call try 'main'
call try 'internal'
call try 'routine'
say o~nested
exit
try:
  what = arg(1)
  signal on syntax name e
  if what = 'main' then reply 1
  if what = 'internal' then do; call lab; return; end
  if what = 'routine' then call r
  say what '=> ok'
  return
e: say what '=>' condition('o')~code
  return
lab:
  reply 1
::routine r
  reply 1
::class box
::method inside
  signal on syntax name h
  interpret 'reply 1'
  return 'no'
h: return condition('o')~code
::method nested
  do i = 1 to 2
    if i = 2 then reply 'second round'
  end
  return
```

```output
inside => 99.924
main => 99.919
internal => 99.919
routine => 99.919
second round
```

## Deviations

r[instr.reply.deviation.15]
A REPLY executed inside a loop that interrexx does not flatten, such as a
DO group that has a label and no repetition (`DO LABEL blk`), is refused:
interrexx ends the program with exit code 120 and a message on standard error
that says it is not implemented. ooRexx continues the method on a new
thread, as the rules above describe. The same holds for the same REPLY inside
a nested DO of that group. The REPLY in the text of an INTERPRET is the plain
rule of [the placement rule](r:instr.reply.method), and raises `SYNTAX 99.924`
on both. The handler of a CALL ON is an internal routine, so a REPLY there is
misplaced, as the same rule says.

```rexx
o = .rep~new
say 'before'
say o~m
::class rep
::method m
  do label blk
    reply 'from block'
  end
  return
```

```output rc=120
before
```
