---
element: "MessageNotification"
kind: class
rexxref: "Builtin Classes > Utility Classes > MessageNotification Class; Builtin Classes > Utility Classes > MessageNotification Class > messageComplete (Abstract Method)"
---

# MessageNotification

`MessageNotification` is an interface class: a class that a program's class
inherits to say that its instances can be told when a message has ended. An
object that was given to the `notify` method of a [`Message`](message.md) as
the target of a notification has to be an instance of it, and the library
then sends that object the one message the interface names,
`messageComplete`, each time the message ends. The class itself implements
nothing: its one method is abstract, and an implementation is a class that
declares `messageComplete`.

The page specifies what the class is, what an implementation has to answer
to, and when and how the library sends to it. How the activities that run a
message and its notifications are scheduled is specified in the environment
chapter.

## The class

r[class.messagenotification.creation]
`MessageNotification` is a mixin class whose superclass is `Object`, and
`queryMixinClass` answers 1 for it ([the mixin rule](r:class.index.mixins)).
A class implements the interface by inheriting it with the `INHERIT` option of
[`::CLASS`](../directives/class.md), or by being a subclass of such a class; an
object is an implementation when it is an instance of one of these, which is
what `isInstanceOf` tests, and nothing else counts: an object whose class has a
method `messageComplete` and does not inherit the interface is not accepted by
`notify`, which raises `SYNTAX 88.914` for it. `new` is not refused, and makes
an object of the class that received it. `Message` is itself an
implementation, so a message object can be the target of a notification.

```rexx
say .messagenotification~queryMixinClass .messagenotification~superclass~id
say .messagenotification~new~class~id .messagenotification~new~string
say .impl~new~isInstanceOf(.messagenotification),
  .sub~new~isInstanceOf(.messagenotification),
  .plain~new~isInstanceOf(.messagenotification),
  .message~new(.object~new, 'string')~isInstanceOf(.messagenotification)
m = .message~new(.plain~new, 'messageComplete')
call try 'm~notify(.plain~new)'
call try 'm~notify(.sub~new)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class impl inherit MessageNotification
::method messageComplete
  nop
::class sub subclass impl
::class plain
::method messageComplete
  nop
```

```output
1 Object
MessageNotification a MessageNotification
1 1 0 1
m~notify(.plain~new) => 88.914
m~notify(.sub~new) => ok
```

## Methods

### messageComplete

r[class.messagenotification.messagecomplete]
`messageComplete(source)` is abstract. Sent to an object whose class has no
`messageComplete` of its own, whatever arguments it has, it raises
`SYNTAX 93.965`, as [the abstract rule](r:dir.method.abstract) says of any
abstract method. An implementation declares the method and takes one
argument, the message object that has ended, as `source`: the library sends
it after the run has ended, so `completed` answers 1 for it, and the answer
of the message is there for `result` to give, or its error for
`errorCondition`. What an implementation answers is not used. The library
sends the message in the circumstances that
[the `notify` rule](r:class.message.notify) lists: once for each `notify` to
the target, at each end of the message and in the order of the calls; at once
when the message had already ended; and in the activity that ran the message
for a message that was started. A `messageComplete` that raises an error does
not change the message; where the error shows is also stated there.

```rexx
t = .t~new
m = .message~new(t, 'show', 'I', 'a')
log = .logger~new
m~notify(log)
say m~send
say log~lines~strip
g = .message~new(t, 'show', 'I', 'b')
g~notify(log)
g~start
log~await(2)
say log~lines~strip
call try '.noimpl~new~messageComplete(1)'
call try '.noimpl~new~messageComplete'
call try '.messagenotification~new~messageComplete(1, 2, 3)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method show unguarded
  return 'shown' arg(1)
::class logger inherit MessageNotification
::method init
  expose lines
  lines = ''
::method messageComplete unguarded
  expose lines
  use arg msg
  lines = lines msg~class~id msg~completed msg~result'.'
::method lines unguarded
  expose lines
  return lines
::method await unguarded
  expose lines
  use arg n
  guard on when lines~words >= n * 3
::class noimpl inherit MessageNotification
```

```output
shown a
Message 1 shown a.
Message 1 shown a. Message 1 shown b.
.noimpl~new~messageComplete(1) => 93.965
.noimpl~new~messageComplete => 93.965
.messagenotification~new~messageComplete(1, 2, 3) => 93.965
```
