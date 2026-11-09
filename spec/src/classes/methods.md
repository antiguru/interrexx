---
element: "Method search"
kind: concept
rexxref: "Objects and Classes > Default Search Order for Method Selection; Objects and Classes > Changing the Search Order for Methods; Objects and Classes > Defining an UNKNOWN Method; Objects and Classes > Mixin Classes; Objects and Classes > Metaclasses; Objects and Classes > Defining Instance Methods with SETMETHOD or ENHANCED; Objects and Classes > Public, Package-Scope, and Private Methods; Special Variables > SUPER; Builtin Classes > Fundamental Classes > Object Class > setMethod; Builtin Classes > Fundamental Classes > Object Class > unsetMethod; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > define; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > defineMethods; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > delete; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > enhanced; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > inherit; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > uninherit"
deviations: [32, 105]
---

# Method search

Every message goes to an object, and the object's class decides which method
answers it. This page specifies the whole search: which methods an object
has of its own, in which order its class and the classes it descends from are
looked at, how a message can start the search part of the way down, what
`UNKNOWN` receives when nothing is found, how a class object finds the class
methods it answers, and what a change to a class does to objects that already
exist. The pages of the classes `Object` and `Class` specify the arguments of
the methods that make these changes; this page specifies what the changes do
to the search. The [classes chapter](index.md) lists which class inherits
which. The directives that create classes and methods are in
[`::CLASS`](../directives/class.md) and
[`::METHOD`](../directives/method.md), and
[the search rule](r:dir.class.search) of the first gives the simple cases.

Two words are used throughout. The **method table** of a class is the list of
names the class itself holds: the methods that its `::METHOD` directives
declare and that `define` and `defineMethods` add. An entry of a table is
either a method or a mark that hides the name (see [hidden
names](r:class.methods.hidden)). The **order** of a class is the list of
classes whose tables a message to one of its instances is looked up in.

## The order of a class

r[class.methods.order]
The order of `Object` holds only `Object`. The order of any other class `C`
starts with `C` itself and goes on with the orders of the classes that its
`superClasses` method answers, one after the other: the superclass first, then
the mixin classes in the order they were inherited. A class that this makes
occur more than once keeps only its last place. The order therefore ends with
`Object`, and a class stands after every class that has it as a superclass or
inherits it.

> **Note.** The example prints the order of two classes. Each `who` method
> names its own class and sends `who` on to the class that follows it, using
> the special variable `SUPER`, which [the SUPER rule](r:class.methods.super)
> specifies, so that the line shows the classes of the order in turn, `Object`
> left out. `LEAF2` lists the same mixin classes as `LEAF` the other way
> round.

```rexx
call show .leaf, .leaf2
exit
show:
  do c over arg(1, 'A')
    say c~id':' c~new~who
  end
  return
::routine step
  use arg id, obj, scope
  signal on nomethod name last
  return id obj~who:scope
last: return id
::class top
::method who
  return step('TOP', self, super)
::class mid subclass top
::method who
  return step('MID', self, super)
::class ma mixinclass object
::method who
  return step('MA', self, super)
::class mb mixinclass object
::method who
  return step('MB', self, super)
::class leaf subclass mid inherit ma mb
::method who
  return step('LEAF', self, super)
::class leaf2 subclass mid inherit mb ma
::method who
  return step('LEAF2', self, super)
```

```output
LEAF: LEAF MID TOP MA MB
LEAF2: LEAF2 MID TOP MB MA
```

r[class.methods.order.base]
A mixin class has a base class, the first class among its ancestors that is
not a mixin class, and the base class comes behind it in its own order. By
[the order rule](r:class.methods.order) a mixin class that a class inherits
therefore stands in front of its base class and of every class above that,
and behind the classes of the superclass's order that are not among those. A
mixin class based on `Object` stands behind every class of the superclass's
order but `Object`, so its methods hide only those of `Object` and of mixin
classes listed after it; this is the case [the search
rule](r:dir.class.search) of the directive describes. A mixin class based on
an ancestor of the superclass hides the methods of that ancestor and of the
classes above it, and loses to the classes of the chain below that ancestor.
The mixin classes of one list keep the order of the list, so a mixin class
based on `Object` that is listed before one based on a lower class also stands
in front of that lower class. In the example, `LEAF` lists the `Object`-based
mixin class `OM` first, so it stands in front of `BASE`, and `LEAF2` lists it
last, so it stays behind.

```rexx
call show .leaf, .leaf2, .leaf3
exit
show:
  do c over arg(1, 'A')
    say c~id':' c~new~who
  end
  return
::routine step
  use arg id, obj, scope
  signal on nomethod name last
  return id obj~who:scope
last: return id
::class base
::method who
  return step('BASE', self, super)
::class mid subclass base
::method who
  return step('MID', self, super)
::class mx mixinclass base
::method who
  return step('MX', self, super)
::class om mixinclass object
::method who
  return step('OM', self, super)
::class leaf subclass mid inherit om mx
::method who
  return step('LEAF', self, super)
::class leaf2 subclass mid inherit mx om
::method who
  return step('LEAF2', self, super)
::class leaf3 subclass mid inherit mx
::method who
  return step('LEAF3', self, super)
```

```output
LEAF: LEAF MID OM MX BASE
LEAF2: LEAF2 MID MX BASE OM
LEAF3: LEAF3 MID MX BASE
```

r[class.methods.order.shared]
A class that can be reached by more than one path stands in the order once,
behind every class that reaches it. Two mixin classes that inherit the same
mixin class, and a subclass that inherits a mixin class whose own mixin class
its superclass already has, are the usual cases. A class cannot be inherited
twice by naming it twice or by inheriting it when it is already an ancestor:
[the mixin rule](r:dir.class.mixin) gives `SYNTAX 98.944` for a class that is
listed twice or is the superclass, and the same code is raised when the class
named is an ancestor of the receiving class at its turn in the list. Listing a
mixin class after one that inherits it is therefore the error
(`INHERIT left shared` in the second example), and listing it in front of that
one is not (`INHERIT shared left`).

```rexx
call show .both, .kid, .late
exit
show:
  do c over arg(1, 'A')
    say c~id':' c~new~who
  end
  return
::routine step
  use arg id, obj, scope
  signal on nomethod name last
  return id obj~who:scope
last: return id
::class shared mixinclass object
::method who
  return step('SHARED', self, super)
::class left mixinclass object inherit shared
::method who
  return step('LEFT', self, super)
::class right mixinclass object inherit shared
::method who
  return step('RIGHT', self, super)
::class both inherit left right
::method who
  return step('BOTH', self, super)
::class both2 inherit left
::method who
  return step('BOTH2', self, super)
::class kid subclass both2 inherit right
::method who
  return step('KID', self, super)
::class late inherit shared left
::method who
  return step('LATE', self, super)
```

```output
BOTH: BOTH LEFT RIGHT SHARED
KID: KID BOTH2 LEFT RIGHT SHARED
LATE: LATE LEFT SHARED
```

```rexx
call try 'inherit left shared'
call try 'inherit shared left'
call try 'inherit left left'
call try 'subclass both inherit left'
exit
try:
  signal on syntax name e
  src = .array~of('::class shared mixinclass object', -
    '::class left mixinclass object inherit shared', -
    '::class both inherit left', '::class c' arg(1))
  p = .package~new('spec-methods-inherit', src)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
inherit left shared => 98.944
inherit shared left => ok
inherit left left => 98.944
subclass both inherit left => 98.944
```

r[class.methods.order.change]
The order of a class is worked out again whenever its list of superclasses
changes, and so is the order of every class below it. The `inherit` method of
`Class` adds a mixin class at the end of the list, or immediately in front of
the class that its second argument names, which may be the superclass; the
`uninherit` method takes one out. The example prints the order of `C` and of
its subclass `SUB` and the list of superclasses after each change.

```rexx
call show 'start'
.c~inherit(.mb)
call show 'inherit mb'
.c~inherit(.mx, .p1)
call show 'inherit mx before p1'
.c~uninherit(.ma)
call show 'uninherit ma'
.c~inherit(.ma, .mx)
call show 'inherit ma before mx'
.c~uninherit(.mb)
call show 'uninherit mb'
exit
show:
  s = ''
  do x over .c~superclasses
    s = s x~id
  end
  say arg(1)':' s~strip '/' .c~new~who '/' .sub~new~who
  return
::routine step
  use arg id, obj, scope
  signal on nomethod name last
  return id obj~who:scope
last: return id
::class p1
::method who
  return step('P1', self, super)
::class ma mixinclass object
::method who
  return step('MA', self, super)
::class mb mixinclass object
::method who
  return step('MB', self, super)
::class mx mixinclass p1
::method who
  return step('MX', self, super)
::class c subclass p1 inherit ma
::method who
  return step('C', self, super)
::class sub subclass c
::method who
  return step('SUB', self, super)
```

```output
start: P1 MA / C P1 MA / SUB C P1 MA
inherit mb: P1 MA MB / C P1 MA MB / SUB C P1 MA MB
inherit mx before p1: MX P1 MA MB / C MX P1 MA MB / SUB C MX P1 MA MB
uninherit ma: MX P1 MB / C MX P1 MB / SUB C MX P1 MB
inherit ma before mx: MA MX P1 MB / C MA MX P1 MB / SUB C MA MX P1 MB
uninherit mb: MA MX P1 / C MA MX P1 / SUB C MA MX P1
```

## The search for a message

r[class.methods.search]
A message to an object is looked up in this sequence. First come the
object's own methods, those that `setMethod` gave it and those that
`enhanced` made it with. Then come the tables of the classes of the order of
the object's class, one after another. The name of the message is compared in
upper case. The first table that has an entry for the name decides: when the
entry is a method that the sender may run, that method runs; when the entry
hides the name, or the method is one the sender may not run, the name counts
as not found and the search for the name ends, without going on to the classes
behind. A name that counts as not found goes to `UNKNOWN`, as
[the unknown rule](r:class.methods.unknown) says. The same sequence serves
`send` and the other methods that send a message, and the operators, which
are messages. The example shows each step: a table that is
reached first wins, a superclass before a mixin class, and an object's own
method before all of them. The message name is written in several cases.

```rexx
o = .sub~new
say o~one '|' o~TWO '|' o~Three '|' o~four
o~put('one', 'return "object one"')
o~put('four', 'return "object four"')
say o~one '|' o~two '|' o~three '|' o~four
say .sub~new~one '|' .sub~new~four
::class base
::method one
  return 'base one'
::method two
  return 'base two'
::method three
  return 'base three'
::class mx mixinclass object
::method two
  return 'mx two'
::method three
  return 'mx three'
::method four
  return 'mx four'
::class sub subclass base inherit mx
::method one
  return 'sub one'
::method put
  self~setMethod(arg(1), arg(2))
```

```output
sub one | base two | base three | mx four
object one | base two | base three | object four
sub one | mx four
```

r[class.methods.search.access]
A method that the sender may not run, because it is `PRIVATE` or `PACKAGE`
as [the access rule](r:dir.method.access) says, does not let the search go on:
the name counts as not found, even when a class behind has a public method of
that name. The message goes to `UNKNOWN` when there is one, and otherwise
[raises](r:cond.nomethod.raise) the error that the access rule gives, `97.2`
for the private method of the example. The sender that may run the method
still finds it, and a scoped send from the method reaches a method behind it.
A message that START sends is checked as [Deviation
105](r:dir.method.deviation.105) states.

```rexx
o = .sub~new
call try 'say o~m'
call try 'say o~inside'
call try 'say o~behind'
call try 'say .withunk~new~m'
exit
try:
  signal on syntax name e
  interpret arg(1)
  return
e: say arg(1) '=>' condition('o')~code
  return
::class base
::method m
  return 'base m'
::class sub subclass base
::method m private
  return 'sub m'
::method inside
  return self~m
::method behind
  return self~m:super
::class withunk subclass sub
::method unknown
  use arg name
  return 'unknown took' name
```

```output
say o~m => 97.2
sub m
base m
unknown took M
```

r[class.methods.hidden]
A table can hold a mark that hides a name instead of a method: the search
stops at it, and the name counts as not found. The marks are made by `define`
with no method, by an item `.nil` in the collection that `defineMethods` or
`enhanced` receives, and, for the object's own methods, by `setMethod` with no
method. The `delete` method of `Class` removes the entry of a name from the
table of the class it is sent to, whether it is a method or a mark, and the
name is then looked up in the classes behind. A name the table does not hold
is left as it is; `delete` does not remove a method that only a class behind
holds. The
example hides a method that the superclass holds, shows that `UNKNOWN`
receives the hidden name, and brings the method back with `delete`.

```rexx
.sub~define('m')
.sub~defineMethods(.directory~new~~put(.nil, 'N'))
o = .sub~new
say t(o, 'm') '|' t(o, 'n') '|' t(o, 'k') '|' o~hasMethod('m')
e = .sub~enhanced(.directory~new~~put(.nil, 'K'))
say t(e, 'k') '|' t(e, 'm')
.sub~delete('m')
.sub~delete('k')
say t(.sub~new, 'm') '|' t(.sub~new, 'n') '|' t(.sub~new, 'k')
say t(.sub~new, 'zz')
exit
t:
  return arg(1)~send(arg(2))
::class base
::method m
  return 'base m'
::method n
  return 'base n'
::method k
  return 'base k'
::class sub subclass base
::method unknown
  return 'unknown' arg(1)
```

```output
unknown M | unknown N | base k | 0
unknown K | unknown M
base m | unknown N | base k
unknown ZZ
```

## The object's own methods

r[class.methods.object]
`setMethod` gives the receiving object a method of its own, found before any
class. Its first argument is the name, which is used in upper case, and its
second is the method: a method object, a string that is the source of the
method, or an array of strings, one for each line, from which a method is
made. A second `setMethod` of the same name replaces the method; the call with
no second argument hides the name for the object, so that the search stops
there and the message goes to `UNKNOWN`. `unsetMethod` takes the name off the
object again: the name is looked up in the classes as if `setMethod` had never
been sent, and `unsetMethod` of a name the object has not set is not an error.
`setMethod` and `unsetMethod` are private methods with a further restriction,
which [the next rule](r:class.methods.object.access) gives.

```rexx
o = .k~new
o~put('m', 'return "from a string"')
o~put('a', .array~of('parse arg x', 'return x * 2'))
o~put('n', .method~new('n', 'return "from a method object"'))
say o~m '|' o~a(21) '|' o~n '|' o~old
o~put('m', 'return "replaced"')
say o~m '|' o~old
o~put('old')
say t(o, 'old')
o~unset('old')
o~unset('m')
o~unset('never_set')
say o~old '|' t(o, 'm')
exit
t:
  return arg(1)~send(arg(2))
::class k
::method old
  return 'k old'
::method m
  return 'k m'
::method put
  if arg(2, 'E') then self~setMethod(arg(1), arg(2))
  else self~setMethod(arg(1))
::method unset
  self~unsetMethod(arg(1))
::method unknown
  return 'unknown' arg(1)
```

```output
from a string | 42 | from a method object | k old
replaced | k old
unknown OLD
k old | k m
```

r[class.methods.object.access]
`setMethod` and `unsetMethod` are accepted from a method that runs with the
object itself as `SELF`, and from a method that runs with a class as `SELF`
when the object is an instance of that class, directly or through a subclass,
which includes a class method that the object's class inherits.
A message from the prolog or from a routine raises `SYNTAX 97.2`, as for any
private method, and from any other method it raises `98.991`. A method that
`setMethod` receives with a scope argument other than `FLOAT` or `OBJECT`, in
either case, raises `88.916`.

```rexx
x = .k~new
y = .k~new
call try 'x~setMethod("a", "return 1")'
call try 'x~self'
call try 'x~other(y)'
call try 'x~other(x)'
call try '.sub~cset(x)'
call try '.k~cset(x)'
call try '.k~cset(.sub~new)'
call try '.sub~cset(.sub~new)'
call try '.sub~cset(.other~new)'
call try '.sub~cset(.k~new)'
call try 'x~scoped("bogus")'
call try 'x~scoped("object")'
call try 'x~scoped("FLOAT")'
call try 'p()'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
::routine p
  .k~new~setMethod('z', 'return 1')
::class k
::method self
  self~setMethod('z', 'return 1')
::method other
  use arg o
  o~setMethod('z', 'return 1')
::method scoped
  self~setMethod('z', 'return 1', arg(1))
::method cset class
  use arg o
  o~setMethod('z', 'return 1')
::class sub subclass k
::class other
```

```output
x~setMethod("a", "return 1") => 97.2
x~self => ok
x~other(y) => 98.991
x~other(x) => ok
.sub~cset(x) => 98.991
.k~cset(x) => ok
.k~cset(.sub~new) => ok
.sub~cset(.sub~new) => ok
.sub~cset(.other~new) => 98.991
.sub~cset(.k~new) => 98.991
x~scoped("bogus") => 88.916
x~scoped("object") => ok
x~scoped("FLOAT") => ok
p() => 97.2
```

r[class.methods.object.scope]
The scope argument of `setMethod` decides which class a scoped send from the
method starts behind. A method set with the scope `FLOAT`, the default,
belongs to no class of the order, and `SUPER` in it is the object's own class,
so that `self~m:super` runs the class's `m`. A method set with `OBJECT` takes
the place of a method of the object's class, and `SUPER` in it is the class
that follows the object's class in the order. A method that `enhanced` adds
behaves as one with `FLOAT`. The scoped send is specified by
[the scoped send rule](r:class.methods.scoped).

```rexx
o = .k~new
o~put('m', 'return "float m >" self~m:super', 'FLOAT')
o~put('n', 'return "object n >" self~n:super', 'OBJECT')
o~put('s1', 'return super', 'FLOAT')
o~put('s2', 'return super', 'OBJECT')
say o~m '|' o~n
say o~s1~id o~s2~id
m = .directory~new~~put('return "enhanced m >" self~m:super', 'M')
e = .k~enhanced(m)
say e~m
::class base
::method m
  return 'base m'
::method n
  return 'base n'
::class k subclass base
::method m
  return 'k m'
::method n
  return 'k n'
::method put
  self~setMethod(arg(1), arg(2), arg(3))
```

```output
float m > k m | object n > base n
K BASE
enhanced m > k m
```

r[class.methods.enhanced]
`enhanced` answers a new object of the receiving class that has, in addition
to the class's methods, the methods of the collection it receives, whose
indexes are the names and whose items are method objects, strings or arrays,
or `.nil` to hide a name. It sends the message `new` to the receiving class
with the arguments that follow the collection, so a class method `new` of the
class runs and `init` receives those arguments, and it gives the object that
`new` answers the added methods. Its class is the receiving class. The
added methods are the object's own methods, so they are found before the
class's, and `unsetMethod` leaves them: after a `setMethod` has replaced one,
`unsetMethod` brings the added method back. The object follows the later
changes to the class, to its superclasses and to its mixin classes, and
`inherit` and `uninherit`, unlike an object that `new` made, which [stops
following](r:class.methods.late) them at the next `define` of its class.

```rexx
c = .k~new
e = .k~enhanced(.directory~new~~put('return "added"', 'EXTRA'), 7, 8)
say e~class~id (e~class == c~class) e~extra e~args
say c~hasMethod('EXTRA') e~hasMethod('EXTRA')
e~set('extra', 'return "set"')
say e~extra
e~unset('extra')
say e~extra
e~unset('extra')
say e~extra
f = .k~enhanced(.directory~new)
.k~define('later', 'return "later"')
.base~define('earlier', 'return "earlier"')
say f~later f~earlier '|' c~hasMethod('LATER') c~hasMethod('EARLIER')
::class base
::class k subclass base
::method new class
  if arg() > 0 then say 'new' arg(1, 'A')~makestring('l', ',')
  forward class (super)
::method init
  expose args
  args = arg(1, 'A')~makestring('l', ',')
::method args
  expose args
  return args
::method set
  self~setMethod(arg(1), arg(2))
::method unset
  self~unsetMethod(arg(1))
```

```output
new 7,8
K 1 added 7,8
0 1
set
added
added
later earlier | 0 0
```

## Changes after objects exist

r[class.methods.late]
`define`, `defineMethods` and `delete`, sent to a class `C`, change the table
of `C` for the objects made after them. An instance of `C` that exists
already keeps the methods it had: the change does not reach it, and after that
no later change reaches it either, neither a further change to the table of
`C`, nor one to its superclasses or its mixin classes, nor `inherit` or
`uninherit` sent to `C`. Until then, such an instance does follow the
changes to the tables of the classes behind its class, and `inherit` and
`uninherit`. The instances of subclasses of `C`, the instances of classes that
inherit `C` and enhanced objects follow every change to `C` whenever they
were made.

> **Note.** In the example, `A` is made first and the first `define` of `ONE`
> cuts it off from the class. `B`, made after that `define`, follows the
> changes to the superclass and to the mixin classes and `inherit` and
> `uninherit`, until the second `define` of `ONE` cuts it off too. `S` is an
> instance of a subclass, so it follows everything. Each line gives, for each
> name, what `A`, `B` and `S` answer, with `-` where the name is not found.

```rexx
a = .k~new
s = .sub~new
call show 'start'
.k~define('one', 'return "o1"')
b = .k~new
call show 'k define one'
.base~define('two', 'return "t"')
call show 'base define two'
.mx~define('three', 'return "h"')
call show 'mixin define three'
.k~inherit(.my)
call show 'k inherit my'
.k~uninherit(.mx)
call show 'k uninherit mx'
.k~define('one', 'return "o2"')
.base~define('two', 'return "T"')
call show 'k define one again'
exit
show:
  say arg(1)':'
  do n over 'ONE TWO THREE FOUR'~makearray(' ')
    say ' ' n f(a, n)'/'f(b, n)'/'f(s, n)
  end
  return
f:
  signal on syntax name e
  return arg(1)~send(arg(2))
e: return '-'
::class base
::class mx mixinclass object
::class my mixinclass object
::method four
  return 'f'
::class k subclass base inherit mx
::class sub subclass k
```

```output
start:
  ONE -/-/-
  TWO -/-/-
  THREE -/-/-
  FOUR -/-/-
k define one:
  ONE -/o1/o1
  TWO -/-/-
  THREE -/-/-
  FOUR -/-/-
base define two:
  ONE -/o1/o1
  TWO -/t/t
  THREE -/-/-
  FOUR -/-/-
mixin define three:
  ONE -/o1/o1
  TWO -/t/t
  THREE -/h/h
  FOUR -/-/-
k inherit my:
  ONE -/o1/o1
  TWO -/t/t
  THREE -/h/h
  FOUR -/f/f
k uninherit mx:
  ONE -/o1/o1
  TWO -/t/t
  THREE -/-/-
  FOUR -/f/f
k define one again:
  ONE -/o1/o2
  TWO -/t/T
  THREE -/-/-
  FOUR -/f/f
```

r[class.methods.library]
The classes of the class library accept no change to their methods or their
lists of superclasses: `define`, `defineMethods`, `delete`, `inherit` and
`uninherit` sent to one of them raise `SYNTAX 98.985`, so their orders and
tables stay as the library made them. Sending `inherit` to another class with
a library class that is not a mixin class raises `98.942`, as
[the mixin rule](r:dir.class.mixin) says for the directive.

```rexx
call try '.array~define("zork", "return 1")'
call try '.array~defineMethods(.directory~new)'
call try '.array~delete("zork")'
call try '.array~inherit(.mm)'
call try '.array~uninherit(.orderedcollection)'
call try '.object~define("zork", "return 1")'
call try '.mm~inherit(.array)'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
::class mm mixinclass object
```

```output
.array~define("zork", "return 1") => 98.985
.array~defineMethods(.directory~new) => 98.985
.array~delete("zork") => 98.985
.array~inherit(.mm) => 98.985
.array~uninherit(.orderedcollection) => 98.985
.object~define("zork", "return 1") => 98.985
.mm~inherit(.array) => 98.942
```

## Starting the search at a class

r[class.methods.scoped]
A message written `object~name:scope` starts the search at a class. The
`scope` is a variable symbol or an environment symbol such as `.base`, and its
value is evaluated like that of any term; a string or a number there makes the
program unreadable with `SYNTAX 20.917`. The value must be a class that stands
in the order of the object's class, that is the class itself, a superclass, an
inherited mixin class or `Object`. Another value raises `SYNTAX 88.914` when
it is not a class, and `93.957` when it is a class that does not stand in the
order. The object's own methods are not looked at. The search begins with the
table of the named class and goes on through the classes that follow it in the
order of the object's class, whichever class the sending method belongs to. It
ends as [the search rule](r:class.methods.search) says, and a name that is not
found goes to `UNKNOWN` through the plain search for that name. A scoped
message can be sent from any code, and the object need not be `SELF`.

> **Note.** In the first example `o` has a method of its own, and the scoped
> messages skip it. `ONLYMX` is held by a mixin class that stands behind
> `BASE`, so a search that starts at `BASE` reaches it, and `ONLYBASE` is held
> by a class in front of `MX`, so a search that starts at `MX` does not.

```rexx
o = .leaf~new
o~put('hello', 'return "object hello"')
say o~hello '|' o~hello:.leaf '|' o~hello:.mid '|' o~hello:.base
say o~hello:.mx '|' o~onlymx:.base '|' o~onlymx:.mx
call try 'say o~onlybase:.mx'
call try 'say o~onlybase:.base'
exit
try:
  signal on syntax name e
  interpret arg(1)
  return
e: say arg(1) '=>' condition('o')~code
  return
::class base
::method hello
  return 'base hello'
::method onlybase
  return 'only base'
::class mid subclass base
::method hello
  return 'mid hello'
::class mx mixinclass object
::method hello
  return 'mx hello'
::method onlymx
  return 'only mx'
::class leaf subclass mid inherit mx
::method hello
  return 'leaf hello'
::method put
  self~setMethod(arg(1), arg(2))
```

```output
object hello | leaf hello | mid hello | base hello
mx hello | only mx | only mx
say o~onlybase:.mx => 97.1
only base
```

```rexx
o = .leaf~new
name = 'mid'
call try 'say o~hello:.other'
call try 'say o~hello:.nil'
call try 'say o~hello:nosuchvariable'
call try 'say o~hello:name'
call try 'say o~hello:o'
call try 'say o~hello:.object'
exit
try:
  signal on syntax name e
  interpret arg(1)
  return
e: say arg(1) '=>' condition('o')~code
  return
::class mid
::method hello
  return 'mid hello'
::class leaf subclass mid
::class other
::method hello
  return 'other hello'
```

```output
say o~hello:.other => 93.957
say o~hello:.nil => 88.914
say o~hello:nosuchvariable => 88.914
say o~hello:name => 88.914
say o~hello:o => 88.914
say o~hello:.object => 97.1
```

```rexx
o = .mid~new
say o~hello:"MID"
::class mid
::method hello
  return 'mid hello'
```

```output rc=236
```

r[class.methods.scoped.send]
The message name that `send` and `sendWith` receive may be an array of two
items instead of a string: the name, and a class that starts the search the
way the `scope` of a scoped message does, with the same errors for a value
that is not a class (`88.914`) or a class outside the order (`93.957`). An
array of any other size raises `SYNTAX 93.946`.

```rexx
o = .leaf~new
say o~send(.array~of('hello', .leaf)) '|' o~send(.array~of('hello', .mid))
say o~sendWith(.array~of('hello', .mid), .array~new)
call try 'o~send(.array~of("hello"))'
call try 'o~send(.array~of("hello", .mid, 3))'
call try 'o~send(.array~new)'
call try 'o~send(.array~of("hello", 3))'
call try 'o~send(.array~of("hello", .other))'
exit
try:
  signal on syntax name e
  interpret 'r =' arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
::class mid
::method hello
  return 'mid hello'
::class leaf subclass mid
::method hello
  return 'leaf hello'
::class other
::method hello
  return 'other hello'
```

```output
leaf hello | mid hello
mid hello
o~send(.array~of("hello")) => 93.946
o~send(.array~of("hello", .mid, 3)) => 93.946
o~send(.array~new) => 93.946
o~send(.array~of("hello", 3)) => 88.914
o~send(.array~of("hello", .other)) => 93.957
```

r[class.methods.super]
`SUPER` is a variable that every method starts with. Its value is the class
that follows, in the order of the receiver's class, the class whose table the
running method came from; that is `Object` for the last class. It is the
usual scope of a message that runs the method an override replaces. The
value belongs to the activation and depends on the receiver: one mixin class
method sees a different class behind it for receivers of different orders.
A method that was set with `setMethod` and the scope `FLOAT`, and a method
that `enhanced` added, has the receiver's own class as `SUPER`, and one set
with `OBJECT` has the class that follows the receiver's class, as
[the scope rule](r:class.methods.object.scope) says. `SUPER` can be assigned
like any variable, and the new value counts for the rest of the activation. In
the prolog and in a routine `SUPER` is not set, so that it holds its own name,
and a message with the scope `super` there raises `SYNTAX 88.914`, because
that value is not a class.

```rexx
say .one~new~who '|' .two~new~who
say .c~new~plain '|' .c~new~skip
say .last~new~up
signal on syntax name e
say .last~new~who:super
e: say condition('o')~code
say 'super is' super rr()
::routine rr
  return super
::routine step
  use arg id, obj, scope
  signal on nomethod name last
  return id obj~who:scope
last: return id
::class mx mixinclass object
::method who
  return step('MX', self, super)
::class m1 mixinclass object
::method who
  return step('M1', self, super)
::class m2 mixinclass object
::method who
  return step('M2', self, super)
::class one inherit mx m1
::method who
  return step('ONE', self, super)
::class two inherit mx m2
::method who
  return step('TWO', self, super)
::class a
::method go
  return 'a go'
::class b subclass a
::method go
  return 'b go'
::class c subclass b
::method plain
  return self~go:super
::method skip
  super = .a
  return self~go:super
::class last
::method up
  return super~id
```

```output
ONE MX M1 | TWO MX M2
b go | a go
Object
88.914
super is SUPER SUPER
```

## Forwarding to a class

r[class.methods.forward]
The `CLASS` option of [FORWARD](r:instr.forward.class) names the class that
starts the search, and it starts it as the scope of a scoped message does:
the object's own methods are skipped, the table of the named class comes
first, and the classes behind it in the order of the receiver's class follow.
`forward class (super)` therefore runs the method that the running one
overrides. The class
must be one the receiver is an instance of, or the instruction raises
`93.957` as the FORWARD page says. In the example, `GO3` of the first mixin
class forwards to the class behind it in the order of `LEAF`, which is the
second mixin class, and `GO4` of `LEAF` forwards to the second mixin class by
name.

```rexx
o = .leaf~new
say o~go '|' o~go3 '|' o~go4
::class base
::method go
  return 'base go'
::class m1 mixinclass object
::method go
  return 'm1 go'
::method go3
  forward class (super)
::class m2 mixinclass object
::method go
  return 'm2 go'
::method go3
  return 'm2 go3'
::method go4
  return 'm2 go4'
::class leaf subclass base inherit m1 m2
::method go4
  forward class (.m2) message 'go4'
```

```output
base go | m2 go3 | m2 go4
```

## When nothing is found

r[class.methods.unknown]
When the search for a name ends without a method, because no table holds it,
a mark hides it, or the sender may not run the method that was found, the
search is made again for the name `UNKNOWN`, over the object's own methods and
then the order of its class, whatever scope the first message had. The first
`UNKNOWN` found runs, with two arguments: the name of the message in upper
case, and a new array that holds the arguments of the message at their
positions. The array has as many places as the position of the last argument
that was not [omitted](r:conv.arg.omitted), and a place of an omitted argument
holds nothing. The answer of `UNKNOWN` is the answer of the message, and a
message to an `UNKNOWN` that answers nothing answers nothing. A class in front
of another in the order wins also here, so an `UNKNOWN` of a superclass hides
the one of a mixin class and the object's own `UNKNOWN` hides both. When no
`UNKNOWN` is found, [NOMETHOD is raised](r:cond.nomethod.raise). A message to
a class object is looked up in the class-side order the same way.

```rexx
o = .c~new
say o~foo(1, , 3, )
say o~FoO '|' o~"mixed Name"(.nil)
say o~foo:.p(1) '|' o~viasuper
s = .sub~new
say s~zz '|' s~m
o~put('unknown', 'return "own unknown" arg(1)')
say o~foo '|' o~nothing:.p
signal on syntax name e
say .quiet~new~anything
e: say condition('o')~code
say .meta~whatever
::class p
::method unknown
  use arg n, a
  return 'p.unk' n 'size' a~size 'items' a~items 'has2' a~hasIndex(2)
::class c subclass p
::method viasuper
  return self~nothing:super
::method put
  self~setMethod(arg(1), arg(2))
::class mix mixinclass object
::method unknown
  return 'mix.unknown' arg(1)
::method m
  return 'mix m'
::class base
::method unknown
  return 'base.unknown' arg(1)
::class sub subclass base inherit mix
::class quiet
::method unknown
::class meta
::method unknown class
  return 'class-side unknown' arg(1)
```

```output
p.unk FOO size 3 items 2 has2 0
p.unk FOO size 0 items 0 has2 0 | p.unk MIXED NAME size 1 items 1 has2 0
p.unk FOO size 1 items 1 has2 0 | p.unk NOTHING size 0 items 0 has2 0
base.unknown ZZ | mix m
own unknown FOO | own unknown NOTHING
91.999
class-side unknown WHATEVER
```

## Messages to a class

r[class.methods.class-side]
A message to a class object is looked up the same way as one to an instance,
with a class-side order in place of the order. The class-side order of a class
`C` starts with `C`, standing for the class methods that `C` declares with
`CLASS`. It goes on with the order of the metaclass of `C`, each class of it
standing for its instance methods, which are class methods of `C`; then the
class-side orders of the superclass of `C` and of its inherited mixin classes,
in the order of `superClasses`. A class that occurs more than once keeps its
last place. The metaclass of `C` is the one `METACLASS` names, otherwise that
of its superclass, otherwise `Class`, and the order of a metaclass is an
ordinary order that ends with `Class` and `Object`. When a subclass has
the metaclass of its superclass, the class methods of the subclass come first,
those of the superclass next, then the methods of the metaclass, and last
those of the inherited mixin classes. Naming another metaclass for the
subclass puts that metaclass in front of the class methods of the superclass
and keeps the metaclass of the superclass behind them. In the example, `SUB`
inherits `MIX` and has the metaclass of `BASE`, `SUB2` names a metaclass of
its own, and `SUB3` declares nothing, so it answers with the class method of
`BASE` and then the method of the metaclass.

```rexx
call show .sub, .sub2, .sub3
exit
show:
  do c over arg(1, 'A')
    say c~id':' c~who
  end
  return
::routine step
  use arg id, obj, scope
  signal on nomethod name last
  return id obj~who:scope
last: return id
::class meta subclass class
::method who
  return step('META', self, super)
::class meta2 subclass class
::method who
  return step('META2', self, super)
::class base metaclass meta
::method who class
  return step('BASE', self, super)
::class mix mixinclass object
::method who class
  return step('MIX', self, super)
::class sub subclass base inherit mix
::method who class
  return step('SUB', self, super)
::class sub2 subclass base metaclass meta2 inherit mix
::method who class
  return step('SUB2', self, super)
::class sub3 subclass base
```

```output
SUB: SUB BASE META MIX
SUB2: SUB2 META2 BASE META MIX
SUB3: BASE META
```

r[class.methods.class-side.change]
The class methods a class has follow the same changes as its order does. A
class that inherits a mixin class at run time gets the class methods of the
mixin class, and loses them with `uninherit`; the class methods of its
subclasses change with it. The instance methods of a metaclass can be changed
with `define` like those of any class: the classes that were made with that
metaclass before the `define` do not get the new method, and a class made
afterwards does, as an instance of the metaclass does in
[the rule on changes](r:class.methods.late).

```rexx
say 'start:' t(.c, 'who') t(.c, 'later') t(.d, 'later')
.meta~define('later', 'return "meta later"')
say 'meta define:' t(.c, 'later') t(.d, 'later')
say 'new class:' t(.meta~new('e'), 'later')
say 'before inherit:' t(.c, 'mixcls') t(.sub, 'mixcls')
.c~inherit(.mix)
say 'after inherit:' t(.c, 'mixcls') t(.sub, 'mixcls')
.c~uninherit(.mix)
say 'after uninherit:' t(.c, 'mixcls') t(.sub, 'mixcls')
exit
t:
  signal on syntax name e
  return arg(1)~send(arg(2))
e: return '-'
::class meta subclass class
::method who
  return 'meta who'
::class c metaclass meta
::class d metaclass meta
::class sub subclass c
::class mix mixinclass object
::method mixcls class
  return 'mixcls'
```

```output
start: meta who - -
meta define: - -
new class: meta later
before inherit: - -
after inherit: mixcls mixcls
after uninherit: - -
```

r[class.methods.class-side.roles]
A class that stands in a class-side order both as a metaclass and as a class
with class methods occupies one place, the last one, and stands there for its
class methods only. In the example, `T` is both the superclass and the
metaclass of `W`.

```rexx
say .w~who
signal on syntax name e
say .w~instmeth
e: say condition('o')~code
::class t subclass class
::method who
  return 'instance method of t'
::method instmeth
  return 'instance only'
::method who class
  return 'class method of t'
::class w subclass t metaclass t
```

```output
class method of t
97.1
```

## Deviations

r[class.methods.deviation.32]
When the `new` that `enhanced` sends answers an object the class did not
build, interrexx answers that object unchanged, as
[the Deviation 32 rule](r:class.index.deviation.32) states; ooRexx marks the
object enhanced, which changes every object of its class. No method is added
to the object, and the search for the messages to it is the search of its own
class.

```rexx
m = .table~new; m['HI'] = 'return "hi"'
q = .odd~enhanced(m)
say q~class~id q~hasMethod('HI')
::class odd
::method new class
  return 'abc'
```

```output
String 0
```

r[class.methods.deviation.105]
The access check of [the access rule](r:class.methods.search.access) for a
message that START sends is made as for the same message sent by the code that
starts it, as [the Deviation 105 rule](r:dir.method.deviation.105) states;
ooRexx raises `97.2` for every started private message and `97.1` for every
started PACKAGE message.

```rexx
say .vault~new~open
::class vault
::method open unguarded
  m = self~start('code')
  return m~result
::method code private
  return 'private answer'
```

```output
private answer
```
