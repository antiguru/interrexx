---
element: GUARD
kind: instruction
syntax: "GUARD ON | OFF [WHEN expression [, expression]...]"
rexxref: "Keyword Instructions > GUARD"
---

# GUARD

A method of an object can run on several activities at once, for instance when
it was started with [`~start`](r:class.object.start). GUARD controls whether a
running method has exclusive use of its object's variables, and it can make
the method wait until an object variable has a given value. This page
specifies the exclusion, the WHEN form and its conditions, the places where
GUARD may stand, and the errors. Which methods start guarded or unguarded is
the matter of the `::METHOD` directive. The EXPOSE of the variables that a
WHEN expression reads is [the EXPOSE instruction](expose.md).

The page says only what the instruction guarantees. The order in which
activities that are not held apart by a guard or by a wait make progress is
not specified. Every example below is arranged so that its output does not
depend on it.

## Exclusion

r[instr.guard.exclusion]
While an activity runs a method of an object with the guard of the method's
scope held, no other activity runs a method of the same scope of that object
that also holds it ([the lock of a scope](r:conc.locks.scope)). A method that
is guarded starts with the guard held and keeps it until it ends or gives it
up, and a method that is unguarded starts without it. GUARD ON takes the guard
for the rest of the method, waiting until the activity that holds it has
released it. GUARD OFF gives it up for the rest of the method, or until the
next GUARD ON. An unguarded method can read and write the object variables
without it, so it can be called while another activity holds the guard.
Repeated GUARD ON or GUARD OFF instructions do not count: [the counting
rule](r:conc.locks.count) says how often a lock is held.

```rexx
log = .array~new
say 'keeping the guard'
o = .vault~new(log, .true)
h = o~start('holder')
do until o~inside; end
p = o~start('probe')
o~release
h~result; p~result
say log~makeString('l', ' ')

log = .array~new
say 'giving up the guard'
o = .vault~new(log, .false)
h = o~start('holder')
do until o~inside; end
p = o~start('probe')
p~result
o~release
h~result
say log~makeString('l', ' ')
exit
::class vault
::method init
  expose log keep inside go
  use arg log, keep
  inside = .false
  go = .false
::method holder
  expose log keep inside go
  inside = .true
  if \keep then guard off
  do until go; end
  log~append('holder-end')
  return 'h'
::method probe
  expose log
  log~append('probe')
  return 'p'
::method inside unguarded
  expose inside
  return inside
::method release unguarded
  expose go
  go = .true
```

```output
keeping the guard
holder-end probe
giving up the guard
probe holder-end
```

> **Note.** The example starts a method that holds the guard and a second
> guarded method while the first still runs. The first waits for a flag that
> only the main program sets, so the log shows whether the second could run
> before the first ended. In the second run the first method gives up the
> guard, and the second runs at once.

r[instr.guard.end]
A guard that a method holds when it ends is released with it, whether the
method held it from its start, took it with GUARD ON or kept it after
GUARD ON WHEN. A method that is waiting for the guard is not kept waiting by
a method that has ended.

```rexx
o = .box~new
a = o~start('takes')
say a~result
b = o~start('after')
say b~result
::class box
::method takes unguarded
  guard on
  return 'took the guard'
::method after
  return 'ran after'
```

```output
took the guard
ran after
```

## WHEN

r[instr.guard.when]
With WHEN the instruction waits until its condition holds. While it waits it
does not hold the guard, so other activities run methods of the object, and
the condition is tried again once another activity assigns or drops one of the
object variables that it names; when it is tried at other times is not
specified. GUARD ON WHEN takes the guard again before each evaluation; GUARD
OFF WHEN evaluates the condition without it ([the locks
page](r:conc.locks.when)). When the condition is true, GUARD ON WHEN goes on
with the guard held, and GUARD OFF WHEN goes on without it. A condition that
is true at once does not wait.

```rexx
c = .gate~new
r = c~start('waiter')
c~open
say r~result
say c~state
r2 = c~start('waiter2')
c~bump
c~bump
c~bump
say r2~result
::class gate
::method init
  expose open n
  open = .false
  n = 0
::method waiter
  expose open
  guard on when open
  return 'passed'
::method open
  expose open
  open = .true
::method state
  expose open
  return open
::method waiter2
  expose n
  guard off when n >= 3
  return 'n reached' n
::method bump
  expose n
  n += 1
```

```output
passed
1
n reached 3
```

r[instr.guard.when.value]
The condition is an expression whose value must be `0` or `1`, and any other
value raises `SYNTAX 34.902`. It can be a list of expressions separated by
commas, evaluated as [the list of an IF](r:instr.if.list) is, and a value
other than `0` or `1` in an expression that is evaluated raises `SYNTAX 34.6`.

```rexx
o = .box~new
do v over .array~of('1', '2', 'yes', '', '1.0', .nil)
  call try 'one', v
  call try 'two', v
  call try 'three', v
end
l = .lazy~new
w = l~start('stops')
l~setn
say w~result
exit
try:
  signal on syntax name e
  o~set(arg(2))
  say arg(1) arg(2) '=>' o~send(arg(1))
  return
e: say arg(1) arg(2) '=>' condition('o')~code
  return
::class box
::method set
  expose x
  use arg x
::method one
  expose x
  guard on when x
  return 'passed'
::method two
  expose x
  guard on when x, 1
  return 'passed'
::method three
  expose x
  guard on when 1, x
  return 'passed'
::class lazy
::method init
  expose n
  n = 0
::method setn
  expose n
  n = 1
::method stops
  expose n
  guard on when n > 0, 10 // n = 0
  return 'passed'
```

```output
one 1 => passed
two 1 => passed
three 1 => passed
one 2 => 34.902
two 2 => 34.6
three 2 => 34.6
one yes => 34.902
two yes => 34.6
three yes => 34.6
one  => 34.902
two  => 34.6
three  => 34.6
one 1.0 => 34.902
two 1.0 => 34.6
three 1.0 => 34.6
one The NIL object => 34.902
two The NIL object => 34.6
three The NIL object => 34.6
passed
```

r[instr.guard.when.exposed]
The condition must name at least one variable that the method has exposed, as
a simple symbol or as a stem symbol of its own EXPOSE, or that is an object
variable because of USE LOCAL; a compound symbol exposed by its name is not
enough, and neither is a name that stands only inside the parentheses or only
in the list they hold. Otherwise the condition has nothing to wait for, and
the program raises `SYNTAX 99.913` when it is read. This holds for GUARD OFF
WHEN as well. The example creates methods from source text, so that the error
is raised at the call.

```rexx
call try 'expose x; guard on when x'
call try 'expose x; guard on when y, x'
call try 'expose x.; guard on when x.1'
call try 'use local y; guard on when x'
call try 'expose x; guard on when y'
call try 'expose x.1; guard on when x.1'
call try 'use local y; guard on when y'
call try 'guard on when 1'
call try 'expose x; guard off when y'
call try 'expose (lst); guard on when x'
call try 'expose (lst); guard on when lst'
exit
try:
  signal on syntax name e
  m = .method~new('m', arg(1))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
expose x; guard on when x => ok
expose x; guard on when y, x => ok
expose x.; guard on when x.1 => ok
use local y; guard on when x => ok
expose x; guard on when y => 99.913
expose x.1; guard on when x.1 => 99.913
use local y; guard on when y => 99.913
guard on when 1 => 99.913
expose x; guard off when y => 99.913
expose (lst); guard on when x => 99.913
expose (lst); guard on when lst => 99.913
```

> **Note.** A WHEN condition that stays false, while no other activity is
> able to change an object variable, never ends. What a program does then is
> not specified. interrexx ends it with an exit code of 120 and a message,
> as [Deviation 20](../deviations.md#20-a-wait-nothing-can-end) says, where
> ooRexx waits for ever.

## Where GUARD may appear

r[instr.guard.method]
GUARD works in the code of a method. In the main program, in a routine of a
[`::ROUTINE`](r:dir.routine.create) directive and in an internal routine,
including one that a method calls, it raises `SYNTAX 99.911` when it runs. In
a method it may stand anywhere in the method's own clauses, inside a DO, an IF
or a SELECT as well. The text of an INTERPRET raises `SYNTAX 99.912` when the
INTERPRET runs, so none of its clauses run.

```rexx
o = .box~new
say o~nested
say 'inside =>' o~inside
call try 'main'
call try 'internal'
call try 'routine'
exit
try:
  what = arg(1)
  signal on syntax name e
  if what = 'main' then guard on
  if what = 'internal' then do; call lab; return; end
  if what = 'routine' then call r
  say what '=> ok'
  return
e: say what '=>' condition('o')~code
  return
lab:
  guard off
  return
::routine r
  guard on
::class box
::method nested
  do 2
    guard off
  end
  if 1 then guard on
  select
    when 1 then guard off
  end
  return 'nested ok'
::method inside
  signal on syntax name h
  interpret 'guard on'
  return 'no'
h: return condition('o')~code
```

```output
nested ok
inside => 99.912
main => 99.911
internal => 99.911
routine => 99.911
```

## Errors of the clause

r[instr.guard.syntax]
A fault in the form of the clause is raised when the program is read. A
GUARD without ON or OFF, with another word, or with WHEN in their place raises
`SYNTAX 25.913`. A word after ON or OFF that is not WHEN raises `25.912`. A
WHEN with no expression raises `35.929`.

```rexx
call try 'expose x; guard'
call try 'expose x; guard foo'
call try 'expose x; guard when x'
call try 'expose x; guard on x'
call try 'expose x; guard off foo'
call try 'expose x; guard on when'
exit
try:
  signal on syntax name e
  m = .method~new('m', arg(1))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
expose x; guard => 25.913
expose x; guard foo => 25.913
expose x; guard when x => 25.913
expose x; guard on x => 25.912
expose x; guard off foo => 25.912
expose x; guard on when => 35.929
```
