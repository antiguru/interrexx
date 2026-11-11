---
element: "Locks"
kind: concept
rexxref: "Concurrency > Default Concurrency; Concurrency > Sending Messages within an Activity; Concurrency > Using Additional Concurrency Mechanisms; Concurrency > Using Additional Concurrency Mechanisms > SETUNGUARDED Method and UNGUARDED Option; Concurrency > Using Additional Concurrency Mechanisms > GUARD ON and GUARD OFF; Concurrency > Using Additional Concurrency Mechanisms > Guarded Methods"
deviations: [14, 20]
---

# Locks

Methods of one object can run on several activities at once, and a guarded
method must not run at the same time as another guarded method that works on
the same object variables. The interpreter keeps them apart with locks. This
page specifies what a lock belongs to, which methods take one and when, what
an activity that already holds a lock may do, how often a lock is held, and
what happens when activities wait for each other in a ring. The [GUARD
instruction](r:instr.guard.exclusion) owns the instruction and its WHEN form,
the [`::METHOD` directive](r:dir.method.guard) owns the options, and the
[Method class](r:class.method.setunguarded) owns the methods that change them;
this page links those rules and says what they mean for the locks.

The page says only what is guaranteed. Which of several activities that wait
for the same lock gets it first is not specified, and neither is how long a
wait lasts; the examples are arranged so that nothing in their output depends
on either. An example that must show that a method can run while another holds
a lock makes the program wait for it: if the method could not run, the example
would not end. An example that must show that a method cannot run starts it,
lets the holder end, and shows the order in which the two ended, which the
rule forces. How activities are started and how they end is the matter of the
activities page of this chapter; here every activity is one that the program
starts with `start`.

## The lock of a scope

r[conc.locks.scope]
An object has one lock for each of its scopes, and the lock of a scope is held
by at most one activity at a time. A guarded method holds the lock of its
scope from its first clause until it ends or gives it up, and an activity that
asks for a lock that another activity holds waits until the holder lets it go.
The scope of a method is the class that gave the method to the object, as [the
scope rule](r:class.method.scope) says, and the object variables of that class
are the ones its methods share ([the scope rule of
EXPOSE](r:instr.expose.scope)). So two guarded methods of the same class run
one after the other when they work on one object, and run together when they
work on two objects. Guarded methods of different scopes of one object run
together as well, whether the scopes are a class and its subclass, a class and
a mixin class that it inherits, or a class and one of its superclasses. In the
example every trial has a holder, which stays inside a guarded method until
the program lets it go, and a probe, which appends a word to a log. Where the
probe can run at once the program waits for it before it lets the holder go,
and the log shows the probe first. Where it cannot, the log shows the holder's
end first.

```rexx
call trial 'sub holds, sub probes', 'subHold', 'subProbe', 'wait'
call trial 'sub holds, base probes', 'subHold', 'baseProbe', 'free'
call trial 'sub holds, mixin probes', 'subHold', 'mixProbe', 'free'
call trial 'base holds, sub probes', 'baseHold', 'subProbe', 'free'
call trial 'base holds, base probes', 'baseHold', 'baseProbe', 'wait'
call trial 'mixin holds, mixin probes', 'mixHold', 'mixProbe', 'wait'
call trial 'sub holds, other object probes', 'subHold', 'subProbe', 'free', 1
exit
trial: procedure
  use arg title, hold, probe, how, other = 0
  .local~log = .array~new
  .local~inside = .false; .local~go = .false
  o = .sub~new
  target = o
  if other then target = .sub~new
  h = o~start(hold)
  do until .local~inside; end
  p = target~start(probe)
  if how = 'free' then p~result
  .local~go = .true
  h~result; p~result
  say title':' .local~log~makeString('l', ' ')
  return
::class mix mixinclass object
::method mixHold
  call hold
  return 'mixHold'
::method mixProbe
  .local~log~append('probe')
  return 'mixProbe'
::class base
::method baseHold
  call hold
  return 'baseHold'
::method baseProbe
  .local~log~append('probe')
  return 'baseProbe'
::class sub subclass base inherit mix
::method subHold
  call hold
  return 'subHold'
::method subProbe
  .local~log~append('probe')
  return 'subProbe'
::routine hold
  .local~inside = .true
  do until .local~go; end
  .local~log~append('hold-end')
```

```output
sub holds, sub probes: hold-end probe
sub holds, base probes: probe hold-end
sub holds, mixin probes: probe hold-end
base holds, sub probes: probe hold-end
base holds, base probes: hold-end probe
mixin holds, mixin probes: hold-end probe
sub holds, other object probes: probe hold-end
```

r[conc.locks.scope.owner]
A method has the scope that [the scope rule of `Method`](r:class.method.scope)
gives it: the class whose directive or `define` gave it to the class, which an
inherited method keeps, so a method of a mixin class has the lock of the mixin
class. A method that `setMethod` adds to one object with `OBJECT` takes the
place of a method of the object's class and has the lock of that class. A
method that it adds with `FLOAT`, which is the default, and a method that
`enhanced` adds belong to no class: the object has one lock of its own for all
of them, so that two of them run one after the other, and each runs together
with a method of any class. A method that [`run`](r:class.object.run) runs,
guarded unless it was made unguarded, has that lock as well. The example uses
the trials of the rule above.

```rexx
.sub~define('defHold', holder())
call trial 'defined holds, own probes', 'defHold', 'subProbe', 'wait'
call trial 'defined holds, base probes', 'defHold', 'baseProbe', 'free'
call trial 'OBJECT holds, own probes', 'objHold', 'subProbe', 'wait'
call trial 'OBJECT holds, base probes', 'objHold', 'baseProbe', 'free'
call trial 'FLOAT holds, own probes', 'fltHold', 'subProbe', 'free'
call trial 'own holds, FLOAT probes', 'subHold', 'fltProbe', 'free'
call trial 'FLOAT holds, FLOAT probes', 'fltHold', 'fltProbe', 'wait'
call trial 'enhanced holds, own probes', 'enhHold', 'subProbe', 'free', 'enh'
call trial 'enhanced holds, enhanced probes', 'enhHold', 'enhProbe',,
  'wait', 'enh'
call trial 'run holds, own probes', 'runHold', 'subProbe', 'free'
call trial 'run holds, FLOAT probes', 'runHold', 'fltProbe', 'wait'
exit
trial: procedure
  use arg title, hold, probe, how, kind = 'plain'
  .local~log = .array~new
  .local~inside = .false; .local~go = .false
  if kind = 'enh' then do
    t = .table~new
    t['enhHold'] = holder()
    t['enhProbe'] = prober()
    o = .sub~enhanced(t)
  end
  else do
    o = .sub~new
    o~put('fltHold', holder())
    o~put('fltProbe', prober())
    o~put('objHold', holder(), 'OBJECT')
  end
  h = o~start(hold)
  do until .local~inside; end
  p = o~start(probe)
  if how = 'free' then p~result
  .local~go = .true
  h~result; p~result
  say title':' .local~log~makeString('l', ' ')
  return
holder: return .method~new('h', .array~of('.local~inside = .true',,
  'do until .local~go; end', '.local~log~append("hold-end")'))
prober: return .method~new('p', '.local~log~append("probe")')
::class base
::method baseProbe
  .local~log~append('probe')
::class sub subclass base
::method put
  use arg name, method
  if arg(3, 'E') then self~setMethod(name, method, arg(3))
  else self~setMethod(name, method)
::method subProbe
  .local~log~append('probe')
::method subHold
  .local~inside = .true
  do until .local~go; end
  .local~log~append('hold-end')
::method runHold unguarded
  self~run(.method~new('h', .array~of('.local~inside = .true',,
    'do until .local~go; end', '.local~log~append("hold-end")')))
```

```output
defined holds, own probes: hold-end probe
defined holds, base probes: probe hold-end
OBJECT holds, own probes: hold-end probe
OBJECT holds, base probes: probe hold-end
FLOAT holds, own probes: probe hold-end
own holds, FLOAT probes: probe hold-end
FLOAT holds, FLOAT probes: hold-end probe
enhanced holds, own probes: probe hold-end
enhanced holds, enhanced probes: hold-end probe
run holds, own probes: probe hold-end
run holds, FLOAT probes: hold-end probe
```

## Guarded and unguarded methods

r[conc.locks.default]
A method is guarded unless it is unguarded, and a guarded method takes the
lock of its scope when it starts. A method that a `::METHOD` directive
declares is guarded unless the directive says `UNGUARDED`, class methods
included. The two methods of an `::ATTRIBUTE` directive take the option of the
directive, so both are guarded or both unguarded. A `::CONSTANT` directive
makes an unguarded method. A method of the name `UNKNOWN`, a method that
`define` gives to a class and a method that `new` makes are guarded unless
they were made unguarded, whatever the code they run. [The method
options](r:dir.method.guard) and [`isGuarded`](r:class.method.isguarded) say
the same of the options and of what a `Method` object answers; this rule says
what it means for the locks. In the example a guarded holder keeps the lock of
the class, and an unguarded `relay` method sends each probe to the object and
appends a word to the log when the probe has answered, so the log shows
whether the probe had to wait. The probes that wait for the holder are the
guarded methods, the accessors of a guarded attribute, and the `UNKNOWN`
method that answers a message the object has no method for; the others run at
once.

```rexx
.k~define('made', .method~new('made', 'return 1'))
call trial 'plain method', 'plain', 'wait'
call trial 'GUARDED method', 'grd', 'wait'
call trial 'UNGUARDED method', 'un', 'free'
call trial 'attribute get', 'at', 'wait'
call trial 'attribute set', 'at=', 'wait', 1
call trial 'UNGUARDED attribute get', 'au', 'free'
call trial 'UNGUARDED attribute set', 'au=', 'free', 1
call trial 'constant', 'cc', 'free'
call trial 'UNKNOWN', 'nosuch', 'wait'
call trial 'defined by define', 'made', 'wait'
exit
trial: procedure
  use arg title, probe, how, value
  .local~log = .array~new
  .local~inside = .false; .local~go = .false
  o = .k~new
  h = o~start('holder')
  do until .local~inside; end
  if arg(4, 'E') then p = o~start('relay', probe, value)
  else p = o~start('relay', probe)
  if how = 'free' then p~result
  .local~go = .true
  h~result; p~result
  say title':' .local~log~makeString('l', ' ')
  return
::class k
::method holder
  .local~inside = .true
  do until .local~go; end
  .local~log~append('hold-end')
::method relay unguarded
  if arg(2, 'E') then self~send(arg(1), arg(2))
  else self~send(arg(1))
  .local~log~append('probe')
::method plain
  return 1
::method grd guarded
  return 1
::method un unguarded
  return 1
::attribute at
::attribute au unguarded
::constant cc 5
::method unknown
  return 1
```

```output
plain method: hold-end probe
GUARDED method: hold-end probe
UNGUARDED method: probe hold-end
attribute get: hold-end probe
attribute set: hold-end probe
UNGUARDED attribute get: probe hold-end
UNGUARDED attribute set: probe hold-end
constant: probe hold-end
UNKNOWN: hold-end probe
defined by define: hold-end probe
```

r[conc.locks.unguarded]
An unguarded method holds no lock when it starts, so it runs while another
activity holds the lock of its scope, and it reads and changes the object
variables of its scope at its own risk. It can take the lock with `GUARD ON`
and give it up again, as [the exclusion rule](r:instr.guard.exclusion) says,
and a guarded method that it sends a message to holds its own lock for the
length of that call and no longer.
[`setUnguarded`](r:class.method.setunguarded) makes a method unguarded and
[`setGuarded`](r:class.method.setguarded) makes it guarded again; a message
sent to the method afterwards takes the lock, or does not, as the method now
is. In the example the method object that the class holds is changed between
the trials.

```rexx
call trial 'guarded', 'wait'
.k~method('plain')~setUnguarded
call trial 'after setUnguarded', 'free'
.k~method('plain')~setGuarded
call trial 'after setGuarded', 'wait'
exit
trial: procedure
  use arg title, how
  .local~log = .array~new
  .local~inside = .false; .local~go = .false
  o = .k~new
  h = o~start('holder')
  do until .local~inside; end
  p = o~start('plain')
  if how = 'free' then p~result
  .local~go = .true
  h~result; p~result
  say title':' .local~log~makeString('l', ' ')
  return
::class k
::method holder
  .local~inside = .true
  do until .local~go; end
  .local~log~append('hold-end')
::method plain
  .local~log~append('probe')
```

```output
guarded: hold-end probe
after setUnguarded: probe hold-end
after setGuarded: hold-end probe
```

## One activity

r[conc.locks.nested]
An activity that holds the lock of a scope of an object can send a message to
a method of that scope of that object, and the method runs at once, as a call
inside the activity: the activity holds the lock already and holds it once
more for the length of the call. It does not matter who sends the message: the
object to itself, a method of another object that the first called, or code
outside any object that a method of the first called. A message to a method of
another scope, or of another object, takes the lock of that scope or object,
and waits for it if another activity holds it. A message that is started, as
`start` does, is run by another activity, which holds nothing of the sender's,
so it can wait for a lock that the sender holds. In the example an object
sends a message to itself, and it sends a message to another object whose
method sends a message back to the first. Both run on the one activity and
both end.

```rexx
a = .node~new('a')
b = .node~new('b')
a~peer = b
b~peer = a
say a~go
::class node
::attribute peer
::attribute name
::method init
  expose name
  use arg name
::method go
  return self~name 'sends to' self~peer~relay(self) 'and' self~again
::method again
  return 'to itself'
::method relay
  use arg origin
  return self~name 'sends back to' origin~name
```

```output
a sends to b sends back to a and to itself
```

r[conc.locks.count]
An activity holds the lock of a scope as many times as it has entered methods
of that scope that hold it: a guarded method that starts while the activity
holds the lock adds one hold to the activity, and a `GUARD ON` in a method
that does not hold the lock adds one; the end of the method or a `GUARD OFF`
in it takes away the hold that the method added, and no other. A `GUARD ON` in
a method that holds the lock already, and a `GUARD OFF` in a method that does
not, change nothing, so repeated ones do not count. The lock is free for the
other activities when the activity holds it no more. A method that gives up
its own hold with `GUARD OFF` therefore lets another activity in only when no
method below it, on the same activity, still holds the lock. A method that
replies keeps its hold on the new activity, or, when a guarded method below it
on the old activity holds the lock as well, gives up its hold on the old
activity and asks for the lock again on the new one, as [the REPLY
rule](r:instr.reply.guard) says. A line that
[TRACE](r:instr.trace.destination) writes in a method shows the count in the
entries `HASSCOPELOCK` and `SCOPELOCKCOUNT` of its [trace
object](r:class.traceobject.entries). The first example prints them for the
lines of three methods that call a method which does a `GUARD OFF`; where an
outer method holds the lock, the inner one counts two holds and, after its
`GUARD OFF`, one. The methods have no `EXPOSE`, whose effect on the entries is
not specified. The second example starts a method that repeats `GUARD ON` or
`GUARD OFF` and shows with a probe whether the lock was free while the method
waited: repeated instructions leave the lock as the first one did.

```rexx
.traceoutput~destination(.sink~new)
.traceobject~notify = .rec~new
o = .k~new
r = o~guarded
r = o~unguarded
r = o~guardOn
.traceobject~notify = .nil
exit
::class sink
::method lineout
  return 0
::class rec
::method append
  use arg t
  if \t~hasIndex('SCOPELOCKCOUNT') then return
  parse value t~traceline with . '*-*' clause
  if clause = '' then return
  say clause~strip~left(16) 'holds' t~hasScopeLock 'count' t~scopeLockCount
::class k
::method guarded
  trace i
  x = 'guarded'
  y = self~inner
  return 1
::method unguarded unguarded
  trace i
  x = 'unguarded'
  y = self~inner
  return 1
::method guardOn unguarded
  trace i
  x = 'guardOn'
  guard on
  y = self~inner
  guard off
  z = 'released'
  return 1
::method inner
  trace i
  a = 'inner'
  guard off
  b = 'inner off'
  return 1
```

```output
x = 'guarded'    holds 1 count 1
y = self~inner   holds 1 count 1
a = 'inner'      holds 1 count 2
guard off        holds 1 count 2
b = 'inner off'  holds 0 count 1
return 1         holds 0 count 1
return 1         holds 1 count 1
x = 'unguarded'  holds 0 count 0
y = self~inner   holds 0 count 0
a = 'inner'      holds 1 count 1
guard off        holds 1 count 1
b = 'inner off'  holds 0 count 0
return 1         holds 0 count 0
return 1         holds 0 count 0
x = 'guardOn'    holds 0 count 0
guard on         holds 0 count 0
y = self~inner   holds 1 count 1
a = 'inner'      holds 1 count 2
guard off        holds 1 count 2
b = 'inner off'  holds 0 count 1
return 1         holds 0 count 1
guard off        holds 1 count 1
z = 'released'   holds 0 count 0
return 1         holds 0 count 0
```

```rexx
call trial 'on on off', 'onOnOff', 'free'
call trial 'guarded off off on', 'offOffOn', 'wait'
call trial 'guarded on off', 'gOnOff', 'free'
exit
trial: procedure
  use arg title, m, how
  .local~log = .array~new
  .local~inside = .false; .local~go = .false
  o = .k~new
  h = o~start(m)
  do until .local~inside; end
  p = o~start('probe')
  if how = 'free' then p~result
  .local~go = .true
  h~result; p~result
  say title':' .local~log~makeString('l', ' ')
  return
::class k
::method onOnOff unguarded
  guard on
  guard on
  guard off
  .local~inside = .true
  do until .local~go; end
  .local~log~append('hold-end')
::method offOffOn
  guard off
  guard off
  guard on
  .local~inside = .true
  do until .local~go; end
  .local~log~append('hold-end')
::method gOnOff
  guard on
  guard off
  .local~inside = .true
  do until .local~go; end
  .local~log~append('hold-end')
::method probe
  .local~log~append('probe')
```

```output
on on off: probe hold-end
guarded off off on: hold-end probe
guarded on off: probe hold-end
```

r[conc.locks.count.off]
A `GUARD OFF` in a method gives up the hold of that method only. When a
guarded method on the same activity called it, that method still holds the
lock, and an activity that asks for the lock waits until it has ended; when
the caller is unguarded, the called method holds the lock alone, and its
`GUARD OFF` lets the other activity in. In the example the probe is started
while the inner method waits after its `GUARD OFF`: below a guarded method the
log shows the probe after the end of the inner method, and below an unguarded
one the program waits for the probe before it lets the inner method go.

```rexx
call trial 'outer guarded', 'outerG', 'wait'
call trial 'outer unguarded', 'outerU', 'free'
exit
trial: procedure
  use arg title, outer, how
  .local~log = .array~new
  .local~inside = .false; .local~go = .false
  o = .k~new
  h = o~start(outer)
  do until .local~inside; end
  p = o~start('probe')
  if how = 'free' then p~result
  .local~go = .true
  h~result; p~result
  say title':' .local~log~makeString('l', ' ')
  return
::class k
::method outerG
  return self~inner
::method outerU unguarded
  return self~inner
::method inner
  guard off
  .local~inside = .true
  do until .local~go; end
  .local~log~append('inner-end')
  return 1
::method probe
  .local~log~append('probe')
```

```output
outer guarded: inner-end probe
outer unguarded: probe inner-end
```

## Waiting inside a method

r[conc.locks.when]
`GUARD ON WHEN` and `GUARD OFF WHEN` give up the hold of their own method
while the condition is false. `GUARD ON WHEN` takes the lock before each
evaluation of the condition, so it evaluates it with the lock held and goes on
holding it, as [the WHEN rule](r:instr.guard.when) says. `GUARD OFF WHEN`
gives up its method's hold first and evaluates the condition without it each
time, while another activity may hold the lock. The condition is evaluated
again when another activity assigns a value to an object variable that the
condition names as a simple symbol, even the value that it had, or drops one,
whether or not the last evaluation reached that name; a variable that only a
method that the condition calls reads is not one of them. When the condition
is evaluated at other times is not specified. The wait gives up only that one
hold. When the method was called by a guarded method on the same activity,
which holds the lock too ([the counting rule](r:conc.locks.count)), the other
activities find the lock held while the condition waits, so that a guarded
method of another activity cannot change the variable that the condition
reads: only an unguarded method can. The example sends the change from an
unguarded and from a guarded method, to a waiting method that a guarded or an
unguarded method called. The three trials that end are the ones in which the
method that changes the variable can run; the fourth, where a guarded method
has to wait for the lock that its own caller's activity holds, is the case of
[Deviation 20](r:conc.locks.deviation.20), and the example does not run it.
The second example ends only if `GUARD OFF WHEN` evaluates its condition
without the lock: a guarded method waits in it, a guarded method of another
activity then takes the lock and keeps it until the waiting method has gone
on, and an unguarded method makes the condition true.

```rexx
call trial 'guarded caller, unguarded setter', 'outerG', 'setU'
call trial 'unguarded caller, guarded setter', 'outerU', 'setG'
call trial 'unguarded caller, unguarded setter', 'outerU', 'setU'
exit
trial: procedure
  use arg title, outer, setter
  o = .k~new
  m = o~start(outer)
  do until o~waiting; end
  s = o~start(setter)
  s~result
  say title':' m~result
  return
::class k
::method init
  expose flag waiting
  flag = .false; waiting = .false
::method waiting unguarded
  expose waiting
  return waiting
::method outerG
  return self~inner
::method outerU unguarded
  return self~inner
::method inner
  expose flag waiting
  waiting = .true
  guard off when flag
  return 'the condition held'
::method setG
  expose flag
  flag = .true
  return 1
::method setU unguarded
  expose flag
  flag = .true
  return 1
```

```output
guarded caller, unguarded setter: the condition held
unguarded caller, guarded setter: the condition held
unguarded caller, unguarded setter: the condition held
```

```rexx
.local~log = .array~new
o = .k~new
w = o~start('waiter')
do until o~waiting; end
h = o~start('holder')
do until o~inside; end
o~open
say w~result
say h~result
::class k
::method init
  expose x waiting inside
  x = 0; waiting = .false; inside = .false
::method waiting unguarded
  expose waiting
  return waiting
::method inside unguarded
  expose inside
  return inside
::method open unguarded
  expose x
  x = 1
::method holder
  expose inside
  inside = .true
  do until .local~log~items > 0; end
  return 'holder saw the waiter go on'
::method waiter
  expose x waiting
  waiting = .true
  guard off when x
  .local~log~append('w')
  return 'waiter went on'
```

```output
waiter went on
holder saw the waiter go on
```

## Class methods

r[conc.locks.class]
A class is an object, and its class methods are the methods of that object, so
the lock that a guarded class method takes is the lock of the class object for
the scope that defines the method. The class object of a subclass is another
object, and so is every instance. A guarded class method that runs on a class
therefore keeps another guarded class method of the same scope out of the same
class object, and does not keep out the same method on the subclass, the
instance methods of the class, or the class methods of other scopes.

```rexx
.local~log = .array~new
.base~go = .false; .base~inside = .false
h = .base~start('holder')
do until .base~inside; end
q = .base~start('probe')
r = .sub~start('probe')
say 'subclass:' r~result
say 'instance:' .base~new~probe
say 'superclass scope:' .base~tprobe
.base~go = .true
h~result; q~result
say .log~makeString('l', ' ')
::class top
::method tprobe class
  return 'ran'
::class base subclass top
::attribute go class unguarded
::attribute inside class unguarded
::method holder class
  .base~inside = .true
  do until .base~go; end
  .log~append('holder-end')
::method probe class
  .log~append('class-probe')
  return 'ran'
::method probe
  .log~append('instance-probe')
  return 'ran'
::class sub subclass base
```

```output
subclass: ran
instance: ran
superclass scope: ran
class-probe instance-probe holder-end class-probe
```

## Ending

r[conc.locks.end]
A method that ends releases the hold that it still has, as [the end
rule](r:instr.guard.end) of GUARD says, whether it ends with RETURN or with an
error that nothing in the method traps, which the sender sees or, for a
started message, the message keeps. The next activity that waits for the lock
gets it, and the error holds nothing back. In the example the first message
holds the lock when it fails, and the second runs after it.

```rexx
o = .k~new
m1 = o~start('boom')
do until o~inside; end
m2 = o~start('after')
say m2~result
m1~wait
say m1~hasError m1~errorCondition~code
::class k
::method init
  expose inside
  inside = .false
::method inside unguarded
  expose inside
  return inside
::method boom
  expose inside
  inside = .true
  return 1/0
::method after
  return 'ran'
```

```output
ran
1 42.3
```

## Deadlock

r[conc.locks.deadlock]
An activity waits for another when it asks for a lock that the other holds, or
when it waits, with [`result`](r:class.message.result) or
[`wait`](r:class.message.wait), for a message that the other runs. A wait of
either kind that would close a ring of such waits, with the waiting activity
in it, could never end, and it is refused: it raises `SYNTAX 98.905` in the
activity that was about to wait, at the clause that asked for the lock or at
the `result` or `wait` of the message. A guarded method asks for its lock when
it starts, so a refused guarded method does not run: the sender sees the
error, or the message keeps it when the message was started, and
[`result`](r:class.message.result) raises it again. The lock that `GUARD ON`
asks for is refused in the same way. When the waits of two activities close a
ring at once, which of them is refused is not specified, and the other goes on
when the refused one has ended and let go of its locks. In the first example a
method holds the lock of its object and waits for the result of a message that
it started to the same object, whose method needs the lock. Whichever of the
two waits closes the ring is refused, the request for the lock or the
`result`, and either way the error reaches the trap of the main program. The
second is a ring of two objects, in which each of two activities holds the
lock of its own object and then asks for the other's; one of them is refused,
and the method that it asked for does not run. The third asks for the lock
with `GUARD ON` in an unguarded method. The fourth has no lock in the ring:
two started unguarded methods each wait for the result of the other's message;
the one that closes the ring is refused, and the result of the other raises
the same error again.

```rexx
o = .k~new
signal on syntax name refused
say o~outer
exit
refused: say 'refused' condition('o')~code
::class k
::method outer
  m = self~start('inner')
  return m~result
::method inner
  return 'inner'
```

```output
refused 98.905
```

```rexx
.local~ran = .array~new
a = .part~new('a'); b = .part~new('b')
a~other = b; b~other = a
m1 = a~start('cross')
m2 = b~start('cross')
r = .array~new
r~append(m1~result); r~append(m2~result)
do line over r~sort; say line; end
say 'touch ran' .local~ran~items 'times'
exit
::class part
::attribute other
::attribute name
::method init
  expose mine name
  use arg name
  mine = .false
::method holding unguarded
  expose mine
  return mine
::method cross
  expose mine
  mine = .true
  do until self~other~holding; end
  signal on syntax name refused
  return 'touched' self~other~touch
refused:
  return 'refused' condition('o')~code
::method touch
  .local~ran~append(self~name)
  return 'ok'
```

```output
refused 98.905
touched ok
touch ran 1 times
```

```rexx
o = .k~new
signal on syntax name refused
say o~outer
exit
refused: say 'refused' condition('o')~code
::class k
::method outer
  m = self~start('inner')
  return m~result
::method inner unguarded
  guard on
  return 'got the lock'
```

```output
refused 98.905
```

```rexx
.local~ready = .false
o = .k~new
m1 = o~start('a')
m2 = o~start('b')
.local~m1 = m1; .local~m2 = m2
.local~ready = .true
r = .array~new
r~append(get(m1)); r~append(get(m2))
do l over r~sort; say l; end
exit
get: procedure
  signal on syntax name bad
  return 'ok' arg(1)~result
bad: return 'err' condition('o')~code
::class k
::method a unguarded
  do until .local~ready; end
  return .local~m2~result
::method b unguarded
  do until .local~ready; end
  return .local~m1~result
```

```output
err 98.905
err 98.905
```

> **Note.** The refusal is made when a wait is about to begin, so it needs the
> waits to be known to the interpreter: locks and results of messages are. A
> wait on a semaphore, on a condition of `GUARD WHEN`, on a message that is
> not sent, or in a loop that polls a variable is not one, and a ring that
> goes through such a wait is not refused: what a program does then is not
> specified, and [Deviation 20](r:conc.locks.deviation.20) says what interrexx
> does with a wait that nothing can end.

## Deviations

r[conc.locks.deviation.14]
An activity that waits inside a nested construct, such as the code of an
INTERPRET or a method that [`run`](r:class.object.run) runs, for something,
such as a lock, a result or the change of a variable, that only an activity
that is suspended beneath it can provide is refused, and the program ends with
exit code 120. ooRexx runs each activity on a thread of its own, so the
activity that waits is not in the way of the one that must go on, and the
program ends normally. In the first example the first message waits in an
INTERPRET until a flag is set; the second sets the flag and then waits, in an
INTERPRET, for the first one's result. In the second a method that `run` runs
keeps the lock of the object and waits for a flag; the method that sets the
flag then asks, in a method that `run` runs, for the same lock.

```rexx
say 'start'
f = .flag~new
a = f~start('waitForFlag')
s = f~start('setAndWait', a)
say s~result
::class flag
::attribute done unguarded
::method init
  expose done
  done = 0
::method waitForFlag unguarded
  interpret "do while \self~done; end"
  return 'A ended'
::method setAndWait unguarded
  use arg a
  self~done = 1
  interpret "r = a~result"
  return 'S saw' r
```

```output rc=120
start
```

```rexx
say 'start'
.local~inside = .false; .local~flag = .false
o = .k~new
h = o~start('holder')
do until .local~inside; end
s = o~start('asker')
say s~result
say h~result
::class k
::method holder unguarded
  self~run(.method~new('h', .array~of('.local~inside = .true',,
    'do until .local~flag; end', 'return "held"')))
  return 'H ended'
::method asker unguarded
  .local~flag = .true
  return self~run(.method~new('p', 'return "S ran"'))
```

```output rc=120
start
```

r[conc.locks.deviation.20]
A wait that no activity, timer or call can end is refused, and the program
ends with exit code 120. ooRexx waits for ever. This holds for the waits that
the refusal of a ring does not cover ([the rule](r:conc.locks.deadlock)): a
`GUARD WHEN` whose condition only a method that is held back by the lock of
the waiting activity's own caller could make true, and a method that holds a
lock and waits for a semaphore that only the activity that asks for the lock
posts. In the first example a guarded method calls the waiting method of the
rule on [WHEN](r:conc.locks.when), and the method that is to set the variable
is guarded too; in the second the method that holds the lock waits for an
event semaphore that the method that asks for the lock posts.

```rexx
say 'start'
o = .k~new
m = o~start('outer')
do until o~waiting; end
s = o~start('setter')
say s~result
::class k
::method init
  expose flag waiting
  flag = .false; waiting = .false
::method waiting unguarded
  expose waiting
  return waiting
::method outer
  return self~inner
::method inner
  expose flag waiting
  waiting = .true
  guard off when flag
  return 'the condition held'
::method setter
  expose flag
  flag = .true
  return 1
```

```output rc=120
start
```

```rexx
say 'start'
e = .eventsemaphore~new
o = .k~new
say o~outer(e)
::class k
::method outer
  use arg e
  m = self~start('inner', e)
  e~wait
  return 'outer done'
::method inner
  use arg e
  e~post
  return 'inner'
```

```output rc=120
start
```

> **Note.** Two more deviations touch what activities do while they wait.
> [Deviation 18](../deviations.md#18-nested-busy-waiters) says that two
> activities that busy-wait on each other inside nested constructs never end,
> where ooRexx preempts both; no example of it can end, so it has no rule
> here. [Deviation 22](../deviations.md#22-one-schedule-for-racing-activities)
> says that interrexx gives racing activities one fixed order, which ooRexx
> may give rarely or not at all.
