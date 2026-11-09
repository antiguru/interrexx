---
element: "Classes"
kind: chapter
rexxref: "Builtin Classes; Objects and Classes > Mixin Classes; Objects and Classes > Overview of Classes Provided by Rexx; Fundamental Classes > Object Class > defaultName; Fundamental Classes > Object Class > objectName; Fundamental Classes > Object Class > objectName=; Fundamental Classes > Object Class > string"
deviations: [30, 32, 36]
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
class name in lower case, and then the following words. On a page that holds
several classes, the class name in lower case follows the page, as in
`class.comparators.numeric.compare`.

* `class.<page>.<method>` is the main rule of a method, with the method name
  in lower case: `class.array.append`.
* `class.<page>.<method>.<aspect>` is a further rule of the same method, such
  as an error or a default.
* `class.<page>.<topic>` is a rule about the class as a whole, such as
  `creation` or `comparison`. A topic never takes the name of a method of
  the class: when the class has a method of that name, the main rule of the
  method is the rule about it, so for a class with a `string` method the
  rule about its string value is `class.<page>.string`.
* `class.<page>.<method>.sends` is the rule that states the messages the
  method sends, as [the sends rule](r:class.index.sends) says.
* A class method and an instance method of one class never share an
  identifier. When a name exists on both sides, the instance method has the
  plain name, and the class method has the aspect `class-side`; a further
  aspect of the class method follows it, as in
  `class.properties.load.class-side.error`.
* The prefix form of `-` and `+`, which is a method of its own with no
  argument, takes the aspect `prefix`: `class.<page>.minus.prefix`.
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

Every method that a class defines has a rule on its page, and the
identifiers never change once they are in the book. A method that the class
only inherits has no rule on its page: the page links to the rule on the page
that defines it, in a sentence that is not itself a rule and has no
identifier.

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
classes. A mixin class is a class that exists to be inherited. Nothing stops a
program from sending `new` to a mixin class: it makes an instance like any
other class.

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
superclass first, then its mixin classes in the order of its inherit list,
into which the `inherit` method of `Class` can insert more (see the page of
`Class`). The four listings below show the library classes related by it, each
line giving the superclasses of the class before the colon. The first listing
also shows the class of each class, which is `Class` for every one of them,
`Singleton` included. A class has everything its superclass and its mixin
classes inherit, so `CircularQueue` has `OrderedCollection` through `Queue`,
though only `Queue` is among its superclasses. Apart from `ArgUtil`, which
[a rule of its own](r:class.index.argutil) covers, a class that is not in a
listing is described on its own page. `RexxInfo`, whose class is reached only
as `.rexxinfo~class`, and `RegularExpression`, which a program gets with
`::REQUIRES`, are among those.

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

```rexx
call show
.c~inherit(.m3)
call show
.c~inherit(.m4, .m1)
call show
say .circularqueue~isSubclassOf(.orderedcollection),
  .circularqueue~superclasses~items
say .rexxinfo~class~id .rexxinfo~class~superclass~id
exit
show:
  s = ''
  do x over .c~superclasses
    s = s x~id
  end
  say s~strip
  return
::class c inherit m2 m1
::class m1 mixinclass object
::class m2 mixinclass object
::class m3 mixinclass object
::class m4 mixinclass object
```

```output
Object M2 M1
Object M2 M1 M3
Object M2 M4 M1 M3
1 1
RexxInfo Object
```

r[class.index.mixins]
The library classes that are mixin classes are the ones for which
`queryMixinClass` answers 1: the two notification classes, the four
collection mixins `Collection`, `MapCollection`, `OrderedCollection` and
`SetCollection`, `Comparable`, `Orderable`, `Comparator` and every class
below it, `CommandHandler`, `Singleton`, and the stream classes
`InputStream`, `OutputStream`, `InputOutputStream` and `Stream`. Every other
class listed in the hierarchy is not a mixin class.

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

r[class.index.argutil]
The class `ArgUtil` is among the classes of `.environment`. It is a class
whose superclass is `Object`, and it is not a mixin class. It is deprecated
and has no page in this chapter; its methods are not specified.

```rexx
c = .environment~at('ARGUTIL')
say c~class~id c~superclass~id c~queryMixinClass
```

```output
Class Object 0
```

## Arguments and the errors of methods

r[class.index.errors]
A message to an object of a library class can fail in three ways: no method of
that name is found and the class has no `UNKNOWN` method (`Directory`,
`StringTable` and their subclasses have one, and their pages say what it
does), the arguments do not fit the method, or an argument has a value the
method refuses. The first raises [NOMETHOD](r:cond.nomethod.raise), which
[untrapped](r:cond.nomethod.untrapped) ends as `SYNTAX 97.1`, and its private
and package forms are those of [the access rule](r:dir.method.access). The
other two raise SYNTAX at the clause that sent the message, as
[the SYNTAX rule](r:cond.syntax.raise) says, with the error number and subcode
that the method gives. A rule of this chapter that says a method "raises
`SYNTAX m.n`" means what [that convention](r:conv.error.name) means: the
message returns no value, a trap sees `m.n` as the code and `m` as `RC`, and
the text of the error message is not specified.

```rexx
call try '.array~new~nosuchzz'
call try '.array~new~items(1)'
call try '.array~new~at'
call try '.array~new~section("x")'
call try '.directory~new~nosuchzz'
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
.directory~new~nosuchzz => no error
```

r[class.index.args.count]
The argument count of a message is the position of the last argument that is
not [omitted](r:conv.arg.omitted), as it is for [a built-in
function](r:bif.index.call.count), so omitted places at the end add nothing.
A message with more arguments than the method accepts raises a SYNTAX error
whatever the values of the arguments are, and an omitted place before the
last argument still counts. The subcode is `93.902` unless the page of the
method gives another; the stream methods, such as `state`, raise `88.922`.

```rexx
o = .object~new
s = .stream~new('class_index_zzq_never_opened')
call try 'o~hasMethod("A", "B")'
call try 'o~hasMethod(, "B")'
call try 'o~hasMethod("A", )'
call try 'o~copy(1)'
call try 'o~class(, )~id'
call try '.array~of(1)~items(1)'
call try '"abc"~left(-1, "xx", 3)'
call try 's~state(1)'
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
"abc"~left(-1, "xx", 3) => 93.902
s~state(1) => 88.922
```

r[class.index.args.missing]
A message that leaves out an argument the method requires raises a SYNTAX
error. The subcode belongs to the method, and its page gives it. Among those
that occur are `93.903`, an argument at a given position is missing, as for
`append`, `hasMethod`, `left` and the `at` of `List`; `93.901`, fewer
arguments than the method needs, as for the `at` and `put` of `Array`; and
`88.901`, as for `isA` and the `at` and `put` of `Table` and `Directory`.

```rexx
o = .object~new
call try 'o~hasMethod'
call try '.array~new~append'
call try '"abc"~left'
call try '.array~new~at'
call try '.array~new~put(1)'
call try '.list~new~at'
call try '.table~new~at'
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
.list~new~at => 93.903
.table~new~at => 88.901
o~isA => 88.901
.directory~new~put(1) => 88.901
```

## Making an object with new

r[class.index.new]
The `new` that `Class` supplies makes an object of the class that received
it and sends the new object the message `init` with the arguments of `new`,
in order. A library class may have a class-side `new` of its own, which uses
the arguments itself and sends `init` with other arguments or none:
`.array~new(5)` sends `init` with no argument. The page of each such class
states what its `new` sends to `init`. The result of `new` is the new object,
whatever `init` answers. A class that inherits `new` from a superclass still
makes an object of the class that received the message, so a subclass of a
class with an `init` runs that `init` on its own instances. A class that
neither declares nor inherits an `init` other than that of `Object` has the
`init` of `Object`, which accepts no argument: `new` with an argument raises
`SYNTAX 93.902` for such a class. A class that is
[abstract](r:dir.class.abstract) makes no object with the `new` that `Class`
supplies: it raises `SYNTAX 98.989`.

```rexx
p = .point~new(3, 4)
say p~class~id p~x p~y
q = .point3~new(1, 2)
say q~class~id q~x q~y q~z
say .ret~new~class~id
a = .xa~new(5)
say a~class~id a~size
call try '.plain~new(1)'
call try '.plain~new'
call try '.abs~new'
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
::class xa subclass array
::method init
  say 'xa init' arg()
  forward class (super)
::class abs abstract
```

```output
POINT 3 4
POINT3 1 2 z
RET
xa init 0
XA 5
.plain~new(1) => 93.902
.plain~new => PLAIN
.abs~new => 98.989
```

r[class.index.new.init-once]
A message `init` sent to an object after the `new` that made it has returned
runs the method it finds and then raises `SYNTAX 91.999`, whoever sends it
and whether it is sent by `~`, by `send` or as a scoped `~init:super`. An
`init` sent while that first `init` is still running, by FORWARD, by
`self~init:super` or by `self~init`, raises nothing.

```rexx
call try '.noisy~new(1)~init(2)'
call try '.noisy~new(1)~redo'
call try '.noisy~new(1)~send("INIT", 3)'
call try '.viasuper~new(6)~class~id'
call try '.viaself~new(7)~class~id'
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
::class viasuper subclass noisy
::method init
  self~init:super(arg(1))
::class viaself subclass noisy
::method init
  if arg(1) = 7 then self~init(8)
```

```output
init 1
init 2
.noisy~new(1)~init(2) => 91.999
init 1
init 9
.noisy~new(1)~redo => 91.999
init 1
init 3
.noisy~new(1)~send("INIT", 3) => 91.999
init 6
.viasuper~new(6)~class~id => VIASUPER
.viaself~new(7)~class~id => VIASELF
.object~new~init => 91.999
```

r[class.index.new.override]
A class method named `new` that a class declares answers the message `new`
in place of the one `Class` supplies, as the
[metaclass rule](r:dir.class.metaclass) says for a class method that hides
another. The one `Class` supplies is reached from it with FORWARD CLASS to
`super`. The override may answer any object, except as
[Deviation 32](r:class.index.deviation.32) states.

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
sets the name. Its argument must be a string or an object with a `MAKESTRING`
method that answers one, which is then the name; an instance of a subclass of
`String` is a string here. The name is kept from then on, even when it is the
null string. Any other argument, `.nil` and an object that has only a `string`
method included, raises `SYNTAX 88.909`. The object `.nil` has the name
`The NIL object`. The name belongs to the object alone: it changes no other
object, not the objects of its class, and not the class's `id`. `objectName=`
on a string is refused, as [Deviation 36](r:class.index.deviation.36) states.
`objectName` takes no argument and `objectName=` takes one.

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
call try 'o~objectName = .ms~new'
say o~objectName
call try 'o~objectName = .st~new'
call try 'o~objectName = .sub~new("subval")'
say o~objectName
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
::class ms
::method makestring
  return 'from-makestring'
::class st
::method string
  return 'from-string'
::class sub subclass string
```

```output
a XYLO | a XYLO
Fred | a XYLO | a XYLO
12.50 | String
[] The NIL object
Cls | XYLO | The XYLO class
o~objectName = .object~new => 88.909
o~objectName = .nil => 88.909
o~objectName = .ms~new => no error
from-makestring
o~objectName = .st~new => 88.909
o~objectName = .sub~new("subval") => no error
subval
o~objectName("x") => 93.902
o~"OBJECTNAME="() => 93.903
```

r[class.index.string]
The string value of an object that is not a string is found as [the
conversion rule](r:conv.string.object) says: the answer of its `MAKESTRING`
method when it has one, and otherwise the answer of its `string` method.
`say` and the joining operators use it, as [the SAY
rule](r:instr.say.object) and [the concatenation
rule](r:expr.concatenation.operands) say. The `string` method of `Object`
answers the object's `objectName`, so an object with no `MAKESTRING` method
shows the name that was set, and otherwise its default name. A class whose
instances should show differently declares `string`, or `makeString` when
builtins too should see that value. A `string` method that answers an object
that is not a string is covered by [Deviation 30](r:class.index.deviation.30).

```rexx
a = .plain~new
say a '|' a'!' '|' a~string
a~objectName = 'named'
say a '|' a'!' '|' a~string '|' a~defaultName
b = .shown~new
say b '|' b'!' '|' b~string '|' b~objectName
c = .made~new
say c '|' c'!' '|' c~string '|' c~objectName
::class plain
::class shown
::method string
  return 'SHOWN: b'
::class made
::method makestring
  return 'M: c'
```

```output
a PLAIN | a PLAIN! | a PLAIN
named | named! | named | a PLAIN
SHOWN: b | SHOWN: b! | SHOWN: b | a SHOWN
M: c | M: c! | a MADE | a MADE
```

## Messages a method sends

r[class.index.sends]
A method of a library class that sends a message a program can override, to
its receiver, to the receiver's class, to an argument, to an item it holds or
to an object it makes, has a rule `class.<page>.<method>.sends`. That rule
names the messages, their receivers, their arguments and their order, and
says whether the number of times each is sent is specified, with an example
in which an override shows each message. A method with no such rule sends no
message that a program can override, other than what [the conversion
rule](r:conv.string.object) sends to its string arguments. The methods named
here are stated in this rule instead: the `string` method of `Object` sends
`objectName` to the object, and `objectName` sends `defaultName` when no name
was set; `appendAll` sends `allItems` to its argument and appends what it
answers; and `putAll` sends `supplier` to its argument and puts what it
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

## Deviations

r[class.index.deviation.30]
When the `string` method of an object answers an object that is not a
string, a builtin or an operator that needs the string value of that object
ends the program with exit code 120, as [the conventions
rule](r:conv.string.deviation.30) states for builtin arguments and
[the NOSTRING rule](r:cond.nostring.deviation.30) for the condition.

```rexx
say 'before'
say 'x' .weird~new
::class weird
::method string
  return .directory~new
```

```output rc=120
before
```

r[class.index.deviation.32]
A class-side `new` that answers an object other than one it built is
subject to [Deviation 32](../deviations.md): a metaclass `new` that answers
an existing class while `subclass` makes a class ends the program with exit
code 120.

```rexx
say 'before'
y = .k~subclass('Y')
say 'after'
::class meta subclass class
::method new class
  return .array
::class k metaclass meta
```

```output rc=120
```

r[class.index.deviation.36]
`objectName=` on a string ends the program with exit code 120, because a
string is value-based, as [Deviation 36](../deviations.md) says; the
[message assignment rule](r:instr.message.deviation.36) states it for the
instruction.

```rexx
say 'before'
a = 'abc'
a~objectName = 'named'
say 'after'
```

```output rc=120
before
```
