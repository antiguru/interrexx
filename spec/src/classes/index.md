---
element: "Classes"
kind: chapter
rexxref: "Builtin Classes"
---

# Classes

This chapter specifies the classes that every program can use without
declaring them: the classes of the class library. It has a page for each
class, or for a few classes that belong together, and a page, [the method
search](methods.md), for how a message finds the method that runs it. The
chapter index holds what is true of all of them: how a class page is laid
out, how the classes are related, what a method does with a wrong argument,
what `new` does, and what the string value of an object is.

Where this chapter says that a program declares a class, the declaring is
done by the [`::CLASS`](../directives/class.md) directive, and the methods
of that class by [`::METHOD`](../directives/method.md). The chapter does not
repeat what those pages say about the options of the directives. It
specifies the methods of `Class` that make and change classes while a program
runs.

## How a class page is laid out

A class page opens with its front matter, then a title and an overview
paragraph, and then the sections below, in this order. A page leaves out a
section it has nothing for.

* **The class**: rules about the class as a whole: how an instance is made,
  its string value, how two instances compare, what it inherits.
* **Class methods**: the methods of the class object itself, the class side,
  such as `new` and `of`. Each is a rule group.
* **Methods**: the methods of an instance, the instance side, grouped by
  what they do. Each method is a rule group, and a group may cover several
  methods that share a rule.
* **Deviations**: the rules that state a [Deviation](../deviations.md), as
  every page does.

A method that the class inherits from a superclass or a mixin class is
specified on the page of that class. The page of a class links to it and
states only what the class adds or changes. A method whose behavior is shared
by the classes of a family, such as the collection classes, is specified once
on the page of the class it belongs to, and the pages of the other classes
link to it.

The front matter names the element (the class, by its id), `kind: class`, the
sections of the reference manual that describe it, the ANSI section when the
ANSI draft has the class, and the Deviations that touch the page.

## How rules are named

A rule on a class page is named `class.` followed by the page, which is the
class name in lower case, and then the following words.

* `class.<page>.<method>` is the main rule of a method, with the method name
  in lower case: `class.array.append`.
* `class.<page>.<method>.<aspect>` is a further rule of the same method, such
  as an error or a default.
* `class.<page>.<topic>` is a rule about the class as a whole, such as
  `creation`, `string` or `comparison`.
* A class method and an instance method of one class never share an
  identifier. When a name exists on both sides, the instance method has the
  plain name, and the class method has the aspect `class-side`.
* A method that is written as a symbol takes a word in place of the symbol:
  `eq` for `=`, `seq` for `==`, `ne` for `\=`, `sne` for `\==`, `ltgt` for
  `<>`, `gtlt` for `><`, `gt`, `lt`, `ge` and `le` for `>`, `<`, `>=` and
  `<=`, `ngt` and `nlt` for `\>` and `\<`, `sgt`, `slt`, `sge` and `sle` for
  `>>`, `<<`, `>>=` and `<<=`, `nsgt` and `nslt` for `\>>` and `\<<`, `plus`,
  `minus`, `times`, `divide`, `intdiv`, `remainder` and `power` for `+`, `-`,
  `*`, `/`, `%`, `//` and `**`, `concat`, `blank` and `abuttal` for `||`, the
  blank and the abuttal, `and`, `or`, `xor` and `not` for `&`, `|`, `&&` and
  `\`, `brackets` for `[]`, and `iif` for `?`.
* A method whose name ends in `=`, an attribute that is assigned, takes the
  name without it and `-set`: `objectname-set` for `objectName=`, and
  `brackets-set` for `[]=`.

Every method of a class has a rule on its page, and the identifiers never
change once they are in the book.

## The two sides of a class

r[class.index.sides]
A class is an object, and so has methods of its own beside those of its
instances. The methods an instance understands are the instance side, and the
methods the class object itself understands, such as `new` and `of`, are the
class side. A message to the class finds a class-side method and never an
instance-side one; a message to an instance finds an instance-side method and
never a class-side one. `hasMethod` shows which side a name is on.

```rexx
say .array~hasMethod('OF') .array~new~hasMethod('OF')
say .array~hasMethod('APPEND') .array~new~hasMethod('APPEND')
say .array~of(4, 5)~items
signal on syntax name e
say .array~append(1)
e: say condition('o')~code
```

```output
1 0
0 1
2
97.1
```

## The class hierarchy

r[class.index.root]
Every class is an instance of `Class`. `Object` is the root of the
hierarchy: it has no superclass, and every other class has `Object` among its
ancestors. A class has one superclass and may inherit any number of mixin
classes. A mixin class is a class that exists to be inherited; it is also a
class with a superclass, and it can make instances.

```rexx
say .object~superclass~isNil .object~class~id .array~class~id
say .circularqueue~isSubclassOf(.object) .circularqueue~isSubclassOf(.array)
say .comparable~isSubclassOf(.object) .comparable~new~class~id
```

```output
1 Class Class
1 0
1 Comparable
```

r[class.index.hierarchy]
The `superClasses` method of a class answers an array that holds its
superclass first, then the mixin classes it inherits in the order it
inherits them. The four listings below show the library classes related by
it, each line giving the superclasses of the class before the colon. The
first listing also shows the class of each class, which is `Class` for every
one of them, `Singleton` included. A class that is not in a listing is
described on its own page.

```rexx
say .object~superclasses~items .object~class~id
call show 'Class String Method Routine Package Message',
  'MessageNotification AlarmNotification Singleton'
exit
show:
  do n over arg(1)~makearray(' ')
    c = .environment~at(n~upper)
    s = ''
    do x over c~superclasses
      s = s x~id
    end
    say n':' s~strip c~class~id
  end
```

```output
0 Class
Class: Object Class
String: Object Comparable Class
Method: Object Class
Routine: Object Class
Package: Object Class
Message: Object MessageNotification AlarmNotification Class
MessageNotification: Object Class
AlarmNotification: Object Class
Singleton: Class Class
```

```rexx
call show 'Collection MapCollection OrderedCollection SetCollection',
  'Array List Queue CircularQueue Directory StringTable Properties',
  'Stem Table IdentityTable Relation Bag Set'
exit
show:
  do n over arg(1)~makearray(' ')
    c = .environment~at(n~upper)
    s = ''
    do x over c~superclasses
      s = s x~id
    end
    say n':' s~strip
  end
```

```output
Collection: Object
MapCollection: Collection
OrderedCollection: Collection
SetCollection: Collection
Array: Object OrderedCollection
List: Object OrderedCollection
Queue: Object OrderedCollection
CircularQueue: Queue
Directory: Object MapCollection
StringTable: Object MapCollection
Properties: Directory
Stem: Object MapCollection
Table: Object MapCollection
IdentityTable: Object MapCollection
Relation: Object MapCollection
Bag: Object MapCollection SetCollection
Set: Object MapCollection SetCollection
```

```rexx
call show 'DateTime TimeSpan Comparable Orderable Comparator',
  'CaselessComparator ColumnComparator CaselessColumnComparator',
  'DescendingComparator CaselessDescendingComparator',
  'InvertingComparator NumericComparator MutableBuffer Buffer Pointer',
  'WeakReference VariableReference File'
exit
show:
  do n over arg(1)~makearray(' ')
    c = .environment~at(n~upper)
    s = ''
    do x over c~superclasses
      s = s x~id
    end
    say n':' s~strip
  end
```

```output
DateTime: Object Comparable Orderable
TimeSpan: Object Comparable Orderable
Comparable: Object
Orderable: Object
Comparator: Object
CaselessComparator: Comparator
ColumnComparator: Comparator
CaselessColumnComparator: Comparator
DescendingComparator: Comparator
CaselessDescendingComparator: Comparator
InvertingComparator: Comparator
NumericComparator: Comparator
MutableBuffer: Object
Buffer: Object
Pointer: Object
WeakReference: Object
VariableReference: Object
File: Object Comparable Orderable
```

```rexx
call show 'RexxContext StackFrame TraceObject Validate Alarm Ticker',
  'Monitor EventSemaphore MutexSemaphore CommandHandler',
  'NativeCommandHandler CommandContext CommandInput CommandOutput',
  'RexxQueue Supplier StreamSupplier InputStream OutputStream',
  'InputOutputStream Stream'
exit
show:
  do n over arg(1)~makearray(' ')
    c = .environment~at(n~upper)
    s = ''
    do x over c~superclasses
      s = s x~id
    end
    say n':' s~strip
  end
```

```output
RexxContext: Object
StackFrame: Object
TraceObject: StringTable
Validate: Object
Alarm: Object
Ticker: Object
Monitor: Object
EventSemaphore: Object
MutexSemaphore: Object
CommandHandler: Object
NativeCommandHandler: Object CommandHandler
CommandContext: Object
CommandInput: Object InputStream
CommandOutput: Object OutputStream
RexxQueue: Object
Supplier: Object
StreamSupplier: Supplier
InputStream: Object
OutputStream: Object
InputOutputStream: Object InputStream OutputStream
Stream: InputOutputStream
```

r[class.index.mixins]
The library classes that are mixin classes are the ones for which
`queryMixinClass` answers 1: the two notification classes, the four
collection mixins `Collection`, `MapCollection`, `OrderedCollection` and
`SetCollection`, `Comparable`, `Orderable`, `Comparator` and every class
below it, `CommandHandler`, `Singleton`, and the stream classes
`InputStream`, `OutputStream`, `InputOutputStream` and `Stream`. Every other
class listed in the hierarchy is not a mixin class. A mixin class can
still make instances with `new`.

```rexx
mix = ''
other = ''
names = 'MessageNotification AlarmNotification Collection Array',
  'Comparator NumericComparator Singleton CommandHandler Stream String',
  'Message Directory'
do n over names~makearray(' ')
  if .environment~at(n~upper)~queryMixinClass then mix = mix n
  else other = other n
end
say 'mixin:' mix~word(1) mix~word(2) mix~word(3)
say 'more:' mix~subword(4)
say 'other:' other~strip
say .collection~new~class~id .comparator~new~class~id
```

```output
mixin: MessageNotification AlarmNotification Collection
more: Comparator NumericComparator Singleton CommandHandler Stream
other: Array String Message Directory
Collection Comparator
```

## Arguments and the errors of methods

r[class.index.errors]
A message to an object of a library class can fail in three ways: no method
of that name is found, the arguments do not fit the method, or an argument
has a value the method refuses. The first raises
[NOMETHOD](r:cond.nomethod.raise), which untrapped ends as `SYNTAX 97.1`,
and its private and package forms are those of
[the access rule](r:dir.method.access). The other two raise SYNTAX at the
clause that sent the message, as [the SYNTAX rule](r:cond.syntax.raise) says,
with the error number and subcode that the method gives. A rule of this
chapter that says a method "raises `SYNTAX m.n`" means what
[that convention](r:conv.error.name) means: the message returns no value, a
trap sees `m.n` as the code and `m` as `RC`, and the text of the error
message is not specified.

```rexx
call try '.array~new~nosuchzz'
call try '.array~new~items(1)'
call try '.array~new~at'
call try '.array~new~section("x")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('c') condition('o')~code rc
  return
```

```output
.array~new~nosuchzz => SYNTAX 97.1 97
.array~new~items(1) => SYNTAX 93.902 93
.array~new~at => SYNTAX 93.901 93
.array~new~section("x") => SYNTAX 93.907 93
```

r[class.index.args.count]
The argument count of a message is the position of the last argument that is
not [omitted](r:conv.arg.omitted), as it is for [a built-in
function](r:bif.index.call.count), so omitted places at the end add nothing.
A message with more arguments than the method accepts raises `SYNTAX 93.902`,
whatever the values of the arguments are, and an omitted place before the
last argument still counts.

```rexx
o = .object~new
call try 'o~hasMethod("A", "B")'
call try 'o~hasMethod(, "B")'
call try 'o~hasMethod("A", )'
call try 'o~copy(1)'
call try 'o~class(, )~id'
call try '.array~of(1)~items(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
o~hasMethod("A", "B") => 93.902
o~hasMethod(, "B") => 93.902
o~hasMethod("A", ) => 0
o~copy(1) => 93.902
o~class(, )~id => Object
.array~of(1)~items(1) => 93.902
```

r[class.index.args.missing]
A message that leaves out an argument the method requires raises a SYNTAX
error, and the subcode belongs to the method: the page of a method gives
it. Among those that occur are `93.903` for a method such as `append`,
`hasMethod` or `left`, `93.901` for the index methods such as `at` and
`put`, and `88.901` for `isA` and `put` of a `Directory`.

```rexx
o = .object~new
call try 'o~hasMethod'
call try '.array~new~append'
call try '"abc"~left'
call try '.array~new~at'
call try '.array~new~put(1)'
call try 'o~isA'
call try '.directory~new~put(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
o~hasMethod => 93.903
.array~new~append => 93.903
"abc"~left => 93.903
.array~new~at => 93.901
.array~new~put(1) => 93.901
o~isA => 88.901
.directory~new~put(1) => 88.901
```

## Making an object with new

r[class.index.new]
The message `new` to a class makes an object of the class that received it,
and sends the new object the message `init` with the arguments of the `new`
message, in order. The result of `new` is that object, whatever `init`
answers. A class that inherits `new` from a superclass still makes an object
of the class that received the message, so a subclass of a class with an
`init` runs that `init` on its own instances. An `init` that a class does not
declare is that of `Object`, which accepts no argument: `new` with an argument
raises `SYNTAX 93.902` for such a class. A class that is
[abstract](r:dir.class.abstract) makes no object with the `new` that `Class`
supplies.

```rexx
p = .point~new(3, 4)
say p~class~id p~x p~y
q = .point3~new(1, 2)
say q~class~id q~x q~y q~z
say .ret~new~class~id
call try '.plain~new(1)'
call try '.plain~new'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r~class~id
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class point
::attribute x
::attribute y
::method init
  expose x y
  use arg x, y
::class point3 subclass point
::attribute z
::method init
  expose z
  forward class (super) continue
  z = 'z'
::class ret
::method init
  return 'ignored'
::class plain
```

```output
POINT 3 4
POINT3 1 2 z
RET
.plain~new(1) => 93.902
.plain~new => PLAIN
```

r[class.index.new.init-once]
A message `init` to an object that already exists runs the method it finds and
then raises `SYNTAX 91.999`, whoever sends it, and so does an `init` that the
object's own methods send to `self` or by `send` once the object is made.
The `init` that `new` sends, and a FORWARD to the `init` of a superclass
from inside an `init`, are not subject to this.

```rexx
call try '.noisy~new(1)~init(2)'
call try '.noisy~new(1)~redo'
call try '.object~new~init'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class noisy
::method init
  use arg n
  say 'init' n
::method redo
  self~init(9)
```

```output
init 1
init 2
.noisy~new(1)~init(2) => 91.999
init 1
init 9
.noisy~new(1)~redo => 91.999
.object~new~init => 91.999
```

r[class.index.new.override]
A class method named `new` that a class declares answers the message `new`
in place of the one `Class` supplies, as the
[metaclass rule](r:dir.class.metaclass) says for a class method that hides
another. The one `Class` supplies is reached from it with FORWARD CLASS to
`super`. The override may answer any object.

```rexx
say .counted~new~class~id .counted~new~class~id .counted~made
say .fixed~new
::class counted
::method made class
  expose n
  return n
::method new class
  expose n
  n = n + 1
  forward class (super)
::method init class
  expose n
  n = 0
::class fixed
::method new class
  return 'not an object of FIXED'
```

```output
COUNTED COUNTED 2
not an object of FIXED
```

## The string value of an object

r[class.index.defaultname]
The default name of an object is built from the id of its class. For an
ordinary object it is `a` or `an`, a blank and the id, with `an` when the
first letter of the id is `A`, `E`, `I`, `O` or `U` in either case and `a`
otherwise, and the id keeps its own case: `a XYLO`, `an apple`. The default
name of a class is `The`, a blank, the id and `class`, as in `The XYLO class`.
The `defaultName` method answers it, and takes no argument.

```rexx
say .object~new~defaultName '|' .array~new~defaultName
say .xylo~new~defaultName '|' .eagle~new~defaultName '|' .low~new~defaultName
say .apple~new~defaultName '|' .xylo~defaultName '|' .array~defaultName
call try '.xylo~new~defaultName(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class xylo
::class Eagle
::class "low"
::class "apple"
```

```output
an Object | an Array
a XYLO | an EAGLE | a low
an apple | The XYLO class | The Array class
.xylo~new~defaultName(1) => 93.902
```

r[class.index.objectname]
The `objectName` method of an object answers its name: the name set for it,
and when none was set, the object's `defaultName`. The `objectName=` method
sets the name. Its argument is converted to a string as [the conversion
rule](r:conv.string.object) says for an argument that is not a string, and it
is the name from then on, even when it is the null string. An argument that
cannot be converted raises `SYNTAX 88.909`. The object `.nil` has the name
`The NIL object`. The name belongs to the object
alone: it changes no other object, not the objects of its class, and not the
class's `id`. `objectName` takes no argument and `objectName=` takes one.

```rexx
o = .xylo~new
say o~objectName '|' o~defaultName
o~objectName = 'Fred'
say o~objectName '|' o~defaultName '|' .xylo~new~objectName
o~objectName = 12.50
say o~objectName '|' o~objectName~class~id
o~objectName = ''
say '['o~objectName']' .nil~objectName
.xylo~objectName = 'Cls'
say .xylo~objectName '|' .xylo~id '|' .xylo~defaultName
call try 'o~objectName = .object~new'
call try 'o~objectName = .nil'
call try 'o~objectName("x")'
call try 'o~"OBJECTNAME="()'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class xylo
```

```output
a XYLO | a XYLO
Fred | a XYLO | a XYLO
12.50 | String
[] The NIL object
Cls | XYLO | The XYLO class
o~objectName = .object~new => 88.909
o~objectName = .nil => 88.909
o~objectName("x") => 93.902
o~"OBJECTNAME="() => 93.903
```

r[class.index.string]
The string value of an object that is not a string is the answer of its
`string` method, and that of `Object` answers the `objectName` of the object.
So the string value is the name that was set, and otherwise the default name.
`say`, a concatenation with `||`, a blank or an abuttal, and every other use
that needs the string value of an object, as [the message
rule](r:expr.messages.default) says for the joining messages, take the answer
of `string`. A class whose instances should show differently declares `string`
itself.

```rexx
a = .plain~new
say a '|' a'!' '|' a~string
a~objectName = 'named'
say a '|' a'!' '|' a~string '|' a~defaultName
b = .shown~new
say b '|' b'!' '|' b~string '|' b~objectName
::class plain
::class shown
::method string
  return 'SHOWN: b'
```

```output
a PLAIN | a PLAIN! | a PLAIN
named | named! | named | a PLAIN
SHOWN: b | SHOWN: b! | SHOWN: b | a SHOWN
```

## Messages a method sends

r[class.index.sends]
A method of a library class whose result depends on messages it sends, to its
receiver or to its arguments, is specified with those messages: the page
names them and gives their order, because a subclass of the receiver's
class, or the class of an argument, can answer one of them differently, and
the result then differs. A method that sends no message that a class can
override has no such statement. The default of `string` above is such a
case: it sends `objectName`, and `objectName` sends `defaultName` when no name
was set. `appendAll` sends `allItems` to its argument and appends what it
answers, and `putAll` sends `supplier` to its argument and puts what it
answers.

```rexx
say .alpha~new~string '|' .beta~new~string
x = .many~of(1, 2)
t = .array~new
t~appendAll(x)
say t~items t~makeString('L', ',')
r = .relation~new
r~putAll(x)
say r~items r~allIndexes~items
::class alpha
::method defaultName
  return 'alpha-default'
::class beta
::method objectName
  return 'beta-name'
::class many subclass array
::method allItems
  return .array~of(7, 8, 9)
::method supplier
  return .array~of('a', 'b')~supplier
```

```output
alpha-default | beta-name
3 7,8,9
2 2
```
