---
element: "AlarmNotification"
kind: class
deviations: [119]
rexxref: "Builtin Classes > Utility Classes > AlarmNotification Class; Builtin Classes > Utility Classes > AlarmNotification Class > cancel; Builtin Classes > Utility Classes > AlarmNotification Class > triggered (Abstract method)"
---

# AlarmNotification

`AlarmNotification` is an interface class: a class that a program's class
inherits to say that its instances can be told when an alarm goes off or is
cancelled. An object that is given to an `Alarm` or a `Ticker` as the target
of its notification has to be an instance of it, and the library sends that
object `triggered` when the time comes and `cancel` when the alarm is
cancelled. `Message` is an implementation, so a message object can be the
target of an alarm. The interface has two methods: `triggered` is abstract
and an implementation declares it, and `cancel` does nothing, so that an
implementation declares it only when it wants to hear of cancellations.

The page specifies what the class is, what an implementation has to answer to,
and when the library sends to it. The classes `Alarm` and `Ticker` are
specified apart from this page, the times at which an alarm or ticker goes off
are specified with their `init`, and which activity runs the target by [the
timer rule](r:conc.activities.timers) of the concurrency chapter; the examples
here wait on a guard that the notification opens before they print what it
recorded.

## The class

r[class.alarmnotification.creation]
`AlarmNotification` is a mixin class whose superclass is `Object`, and
`queryMixinClass` answers 1 for it ([the mixin rule](r:class.index.mixins)).
A class implements the interface by inheriting it with the `INHERIT` option of
[`::CLASS`](../directives/class.md), or by being a subclass of such a class;
an object is an implementation when it is an instance of one of these, as
`isInstanceOf` tests. Nothing else counts: an `Alarm` or a `Ticker` made with
a target that is not an implementation, `.nil` and an object whose class has
a method `triggered` but does not inherit the interface among them, raises
`SYNTAX 88.914`, and one made with no target raises `93.901`. `new` is not
refused for `AlarmNotification` itself, and makes an object of the class that
received it. A `Message` is an implementation.

```rexx
say .alarmnotification~queryMixinClass .alarmnotification~superclass~id
say .alarmnotification~new~class~id .alarmnotification~new~string
say .impl~new~isInstanceOf(.alarmnotification),
  .sub~new~isInstanceOf(.alarmnotification),
  .plain~new~isInstanceOf(.alarmnotification),
  .message~new(.object~new, 'string')~isInstanceOf(.alarmnotification)
call try '.alarm~new(60, .plain~new)'
call try '.alarm~new(60, .nil)'
call try '.alarm~new(60)'
call try '.alarm~new(60, .sub~new)~cancel'
call try '.ticker~new(60, .plain~new)'
call try '.ticker~new(60)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class impl inherit AlarmNotification
::method triggered
  nop
::class sub subclass impl
::class plain
::method triggered
  nop
```

```output
1 Object
AlarmNotification an AlarmNotification
1 1 0 1
.alarm~new(60, .plain~new) => 88.914
.alarm~new(60, .nil) => 88.914
.alarm~new(60) => 93.901
.alarm~new(60, .sub~new)~cancel => ok
.ticker~new(60, .plain~new) => 88.914
.ticker~new(60) => 93.901
```

## Methods

### triggered

r[class.alarmnotification.triggered]
`triggered(source)` is abstract. Sent to an object whose class has no
`triggered` of its own, whatever arguments it has, it raises `SYNTAX 93.965`,
as [the abstract rule](r:dir.method.abstract) says of any abstract method. An
implementation declares the method and takes one argument, the `Alarm` or
`Ticker` that went off, as `source`: the library sends it at the time of the
alarm, once for an `Alarm`, and for a `Ticker` at its times until it is
cancelled or stopped, as [the timer rule](r:conc.activities.timers) says. The
object that was given as the attachment when the alarm was made is
`source~attachment`. What an implementation answers is not used.

```rexx
n = .note~new
a = .alarm~new(0.02, n, 'att')
n~awaitCount(1)
say n~log~strip
say a~attachment
t = .note~new(3)
k = .ticker~new(0.02, t, 'tick')
t~awaitCancel
say t~log~strip
signal on syntax name c1
.alarmnotification~new~triggered(1)
c1: say condition('o')~code
::class note inherit AlarmNotification
::method init
  expose log count cdone limit
  use arg limit = 0
  log = ''
  count = 0
  cdone = .false
::method triggered unguarded
  expose log count limit
  use arg src
  log = log 'triggered' src~attachment arg()
  count = count + 1
  if count = limit then src~cancel
::method cancel unguarded
  expose log cdone
  use arg src
  log = log 'cancel' src~attachment
  cdone = .true
::method awaitCount unguarded
  expose count
  use arg n
  guard on when count >= n
::method awaitCancel unguarded
  expose cdone
  guard on when cdone
::method log unguarded
  expose log
  return log
```

```output
triggered att 1
att
triggered tick 1 triggered tick 1 triggered tick 1 cancel tick
93.965
```

### cancel

r[class.alarmnotification.cancel]
`cancel(source)` does nothing and answers nothing, whatever arguments it is
sent, none included; an implementation that wants to hear of cancellations
declares it. The library sends it, with the `Alarm` or `Ticker` as the
argument, when the alarm or ticker is cancelled by its `cancel` method while
it is still waiting: an `Alarm` that has not gone off and was not cancelled
before, or a `Ticker` that has not been cancelled before. A `cancel` of an
`Alarm` that has already gone off and a second `cancel` of an `Alarm` or a
`Ticker` ([Deviation 119](r:class.alarmnotification.deviation.119)) send
nothing more.

```rexx
n = .note~new
a = .alarm~new(60, n, 'cancelled')
a~cancel
n~awaitCancel
say n~log~strip
t = .note~new(2)
k = .ticker~new(0.02, t, 'ticking')
t~awaitCancel
say t~log~words t~log~word(t~log~words)
.alarmnotification~new~cancel
.alarmnotification~new~cancel(1, 2, 3, 4)
signal on syntax name c1
x = .alarmnotification~new~cancel
c1: say 'answered nothing:' condition('o')~code
::class note inherit AlarmNotification
::method init
  expose log count cdone limit
  use arg limit = 0
  log = ''
  count = 0
  cdone = .false
::method triggered unguarded
  expose log count limit
  use arg src
  log = log 'triggered' src~attachment
  count = count + 1
  if count = limit then src~cancel
::method cancel unguarded
  expose log cdone
  use arg src
  log = log 'cancel' src~attachment
  cdone = .true
::method awaitCount unguarded
  expose count
  use arg n
  guard on when count >= n
::method awaitCancel unguarded
  expose cdone
  guard on when cdone
::method log unguarded
  expose log
  return log
```

```output
cancel cancelled
6 ticking
answered nothing: 91.999
```

r[class.alarmnotification.deviation.119]
A second `cancel` of an `Alarm` or a `Ticker` sends its target nothing
([Deviation 119](../deviations.md)). ooRexx sends the target `cancel` once or
twice for it, depending on timing, so the examples are not a test of the
owning rule on ooRexx. The first sends `cancel` twice to a `Ticker` that is
still five seconds away, the second to such an `Alarm`, and the implementation
prints a line for each message it receives.

```rexx
n = .n~new
t = .ticker~new(5, n)
t~cancel
t~cancel
call syssleep 0.2
say 'done'
exit
::class n inherit alarmnotification
::method cancel unguarded
  say 'cancel sent'
```

```output
cancel sent
done
```

```rexx
n = .n~new
a = .alarm~new(5, n)
a~cancel
a~cancel
call syssleep 0.3
say 'done'
exit
::class n inherit alarmnotification
::method cancel unguarded
  say 'cancel sent'
```

```output
cancel sent
done
```
