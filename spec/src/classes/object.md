---
element: "Object"
kind: class
rexxref: "Builtin Classes > Fundamental Classes > Object Class; Builtin Classes > Fundamental Classes > Object Class > new (Class Method); Builtin Classes > Fundamental Classes > Object Class > Comparison Methods; Builtin Classes > Fundamental Classes > Object Class > Concatenation Methods; Builtin Classes > Fundamental Classes > Object Class > class; Builtin Classes > Fundamental Classes > Object Class > copy; Builtin Classes > Fundamental Classes > Object Class > defaultName; Builtin Classes > Fundamental Classes > Object Class > hashCode; Builtin Classes > Fundamental Classes > Object Class > hasMethod; Builtin Classes > Fundamental Classes > Object Class > identityHash; Builtin Classes > Fundamental Classes > Object Class > init; Builtin Classes > Fundamental Classes > Object Class > instanceMethod; Builtin Classes > Fundamental Classes > Object Class > instanceMethods; Builtin Classes > Fundamental Classes > Object Class > isA; Builtin Classes > Fundamental Classes > Object Class > isInstanceOf; Builtin Classes > Fundamental Classes > Object Class > isNil; Builtin Classes > Fundamental Classes > Object Class > objectName; Builtin Classes > Fundamental Classes > Object Class > objectName=; Builtin Classes > Fundamental Classes > Object Class > request; Builtin Classes > Fundamental Classes > Object Class > run (Private Method); Builtin Classes > Fundamental Classes > Object Class > send; Builtin Classes > Fundamental Classes > Object Class > sendWith; Builtin Classes > Fundamental Classes > Object Class > setMethod (Private Method); Builtin Classes > Fundamental Classes > Object Class > start; Builtin Classes > Fundamental Classes > Object Class > startWith; Builtin Classes > Fundamental Classes > Object Class > string; Builtin Classes > Fundamental Classes > Object Class > unsetMethod (Private Method)"
deviations: [4, 28, 30, 32, 33, 36, 105]
---

# Object

`Object` is the root of the class hierarchy: every class has it among its
ancestors, so every object understands the methods of this page, and every
class but those that `Class~new` makes understands its class method `new`. The
page specifies each of them: what it accepts, in which order it checks, what
it answers, and which messages it sends to its receiver or to its arguments,
so that a class which overrides a method that another one sends can predict
what happens.

The names that a program gives an object, its string value and its default
name belong to [the chapter index](index.md); the rules below link to them.
How a message finds a method, and what `setMethod` and `unsetMethod` do to
that search, belong to [the method search](methods.md). The examples that show
an error use a small routine `try` that sends the message in its argument and
prints either the answer or the error code, as the builtin pages do.

## Class methods

r[class.object.new]
The class method `new` makes an object of the class that received the message
and sends it `init` with the arguments of `new`, as [the new
rule](r:class.index.new) says. The arguments keep their places: an argument
that is omitted in the middle of the list is omitted in the message to `init`,
and omitted places at the end add nothing to the argument count, as [the count
rule](r:class.index.args.count) says. The answer of `new` is the new object,
whatever `init` answers. Every class that a directive, `subclass` or
`mixinClass` made has this method, mixin classes and abstract classes
included, and an abstract class refuses it as [the abstract
rule](r:dir.class.abstract) says. The class that `Class~new` makes is the one
exception, and [its page rule](r:class.class.new) says what it answers.

```rexx
o = .dd~new(1, , 3)
o = .dd~new(1, )
o = .dd~new(, )
o = .dd~new(, 2)
say .dd~new(7)~class~id .object~new~class~id
call try '.object~new(1)'
call try '.abs~new'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r~class~id
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class dd
::method init
  say 'init' arg() arg(1, 'e') arg(2, 'e') arg(3, 'e')
::class abs abstract
```

```output
init 3 1 0 1
init 1 1 0 0
init 0 0 0 0
init 2 0 1 0
init 1 1 0 0
DD Object
.object~new(1) => 93.902
.abs~new => 98.989
```

## Methods

### Making and copying

r[class.object.init]
The `init` method of `Object` accepts no argument and answers no value; it
changes nothing. A class that declares its own `init` replaces it, and reaches
it with a scoped send to `super`. `new` sends `init` as [the new
rule](r:class.index.new) says, and a program may send it again at any time, as
[the init rule](r:class.index.new.init-again) says. With any argument the
method raises `SYNTAX 93.902`.

```rexx
call try '.object~new~init'
call try '.object~new~init(1)'
call try '.object~new~init(, )'
call try 'r = .object~new~init'
say .keep~new(5)~kept
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class keep
::attribute kept
::method init
  expose kept
  use arg kept
  self~init:super
```

```output
.object~new~init => no error
.object~new~init(1) => 93.902
.object~new~init(, ) => no error
r = .object~new~init => 91.999
5
```

r[class.object.copy]
`copy` answers a new object of the same class. The copy has the object
variables of the receiver, each holding the same object that the receiver's
variable holds, so the copy is shallow, and it has the same methods, the
receiver's own methods included, and the same name. The copy has a table of
its own methods of its own, holding the same method objects, and the variables
of those methods are copied as the others are, so a change to one object's own
methods or to those variables does not reach the other. It does not send
`init`, and takes no argument: with one it raises `SYNTAX 93.902`. A subclass
of `Object` whose instances hold more than variables, as the collection
classes do, defines its own `copy` on its page. A class object cannot be
copied, as [the copy rule](r:class.class.copy) says.

```rexx
a = .box~new
a~items = .array~of(1, 2)
a~n = 5
a~objectName = 'boxed'
a~give('GREET', 'return "hello from" self~objectName')
b = a~copy
say b~class~id (b == a) (b~items == a~items) b~n b~objectName b~greet
b~n = 6
b~items~append(3)
say a~n b~n a~items~items
b~give('ONLYB', 'return "only b"')
say a~hasMethod('onlyb') b~hasMethod('onlyb')
a~give('SETF', 'expose f; f = arg(1)')
a~give('GETF', 'expose f; return f')
a~setf('fa')
c = a~copy
c~setf('fc')
say a~getf c~getf
call try 'a~copy(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class box
::attribute items
::attribute n
::method init
  say 'init'
::method give
  use arg name, src
  self~setMethod(name, src)
```

```output
init
BOX 0 1 5 boxed hello from boxed
5 6 3
0 1
fa fc
a~copy(1) => 93.902
```

### Comparison and joining

r[class.object.eq]
The methods `=` and `==` answer 1 when the argument is the receiver itself and
0 otherwise. They take one argument: none raises `SYNTAX 93.903` and two raise
`93.902`. Neither is defined in terms of another message: a class that
declares `==` changes what `==` answers and leaves `=`, `\=`, `\==`, `<>` and
`><` as they were, as [the default rule](r:expr.messages.default) says of the
equality messages. The operators `=` and `==` of an expression send these
messages ([the operator rule](r:expr.messages.operator)).

```rexx
a = .box~new
b = .box~new
say (a = a) (a == a) (a = b) (a == b) (a~"="(a)) (a~"=="(b))
x = .strict~new
y = .strict~new
say (x = y) (x == y) (x \== y) (x \= y) (x <> y) (x >< y)
call try 'a~"="'
call try 'a~"=="(1, 2)'
call try 'a~"="(,)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class box
::class strict
::method "=="
  return 1
```

```output
1 1 0 0 1 0
0 1 1 1 1 1
a~"=" => 93.903
a~"=="(1, 2) => 93.902
a~"="(,) => 93.903
```

r[class.object.ne]
The methods `\=`, `\==`, `<>` and `><` answer 1 when the argument is not the
receiver itself and 0 when it is. They take one argument, as [the equality
rule](r:class.object.eq) says of `=`, and none of them sends `=` or `==`, nor
is any of them changed by a class that declares `=` or `==`.

```rexx
a = .box~new
b = .box~new
say (a \= b) (a \== b) (a <> b) (a >< b)
say (a \= a) (a \== a) (a <> a) (a >< a)
c = .loose~new
say (c \= c) (c \== c) (c <> c) (c >< c) (c = c) (c == c)
call try 'a~"\="'
call try 'a~"<>"(1, 2)'
call try 'a~"><"(,)'
call try 'a~"\=="'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class box
::class loose
::method "="
  return 0
::method "=="
  return 0
```

```output
1 1 1 1
0 0 0 0
0 0 0 0 0 0
a~"\=" => 93.903
a~"<>"(1, 2) => 93.902
a~"><"(,) => 93.903
a~"\==" => 93.903
```

r[class.object.concat]
The methods `||`, the one blank and the null string answer a new string: the
string value of the receiver, then (for the blank method) one blank, then the
string value of the argument. The string value of the receiver is the answer
of its `MAKESTRING` method when it has one, and otherwise of its `string`
method, as [the string rule](r:class.index.string) says, and the same goes for
the argument; `.nil` has the string value `The NIL object`. The three methods
take one argument: none raises `SYNTAX 93.903` and two raise `93.902`. They
are the messages that the concatenation operators send ([the operator
rule](r:expr.messages.operator)), and a class that declares one of them
changes only that operator.

```rexx
a = .box~new
a~objectName = 'bx'
s = .shown~new
say a~"||"('!') '|' a~" "('!') '|' a~""('!') '|'
say a~"||"(s) '|' s~" "(a) '|' s~""(.nil)
say .both~new~"||"('.') '|' .both~new~" "(.both~new)
o = .own~new
say (o || 'x') '|' (o 'x') '|' (o'x') '|' o~"||"('y') '|' o~" "('y')
call try 'a~"||"'
call try 'a~" "'
call try 'a~""'
call try 'a~"||"(1, 2)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class box
::class shown
::method string
  return 'S'
::class own
::method "||"
  return 'OVERRIDE'
::class both
::method string
  return 'from string'
::method makestring
  return 'from makestring'
```

```output
bx! | bx ! | bx! |
bxS | S bx | SThe NIL object
from makestring. | from makestring from makestring
OVERRIDE | an OWN x | an OWNx | OVERRIDE | an OWN y
a~"||" => 93.903
a~" " => 93.903
a~"" => 93.903
a~"||"(1, 2) => 93.902
```

### Class, names and strings

r[class.object.class]
`class` answers the class of the receiver: the class that made it, or, for an
object that `enhanced` made, the class that received `enhanced`. For a class
object it answers the metaclass of the class (see [the class-side
rule](r:class.methods.class-side)). The method takes no argument and raises
`SYNTAX 93.902` with one. `isA`, `isInstanceOf` and `request` do not send it,
so a class that declares `class` changes none of them.

```rexx
a = .box~new
say a~class~id (a~class == .box) .box~class~id .meta~class~id
say 'abc'~class~id (12)~class~id .array~new~class~id .sub~new~class~id
say .metaed~class~id .metaed~class~class~id
e = .box~enhanced(.directory~new)
say e~class~id (e~class == .box)
say .liar~new~class~id .liar~new~isA(.liar) .liar~new~isInstanceOf(.liar)
call try 'a~class(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class box
::class sub subclass box
::class meta subclass class
::class metaed metaclass meta
::class liar
::method class
  return .box
```

```output
BOX 1 Class Class
String String Array SUB
META Class
BOX 1
BOX 1 1
a~class(1) => 93.902
```

r[class.object.defaultname]
`defaultName` answers the default name of the receiver and takes no argument,
as [the default name rule](r:class.index.defaultname) says. An object that
`enhanced` made has the default name `enhanced`, one blank and the id of its
class, in the case the id was written in: `enhanced K`. That name is also its
`objectName` and its string value until a name is set, and a copy of the
object keeps both names.

```rexx
e = .k~enhanced(.directory~new)
say e~defaultName '|' e~objectName '|' e
say .k~new~defaultName '|' .sub~enhanced(.directory~new)~defaultName
e~objectName = 'chosen'
say e~defaultName '|' e~objectName '|' e~copy~objectName
say e~copy~defaultName
::class k
::class sub subclass k
```

```output
enhanced K | enhanced K | enhanced K
a K | enhanced SUB
enhanced K | chosen | chosen
enhanced K
```

r[class.object.objectname]
`objectName` answers the name of the receiver: the name set with
`objectName=`, and the answer of `defaultName` when no name was set, as [the
name rule](r:class.index.objectname) says. It takes no argument. The `string`
method of an object sends it `objectName` ([the sends
rule](r:class.index.sends)), so an override shows in the string value of an
object that has no `MAKESTRING` method; `objectName` sends `defaultName` only
while no name has been set.

```rexx
a = .plain~new
b = .named~new
say a~objectName '|' b~objectName '|' a '|' b
a~objectName = 'chosen'
say a~objectName '|' a~defaultName '|' a
call try 'a~objectName(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class named
::method defaultName
  say 'defaultName sent'
  return 'dflt'
```

```output
defaultName sent
defaultName sent
a PLAIN | dflt | a PLAIN | dflt
chosen | a PLAIN | chosen
a~objectName(1) => 93.902
```

r[class.object.objectname-set]
`objectName=` sets the name of the receiver to the string value of its
argument, under the checks of [the name rule](r:class.index.objectname): an
argument that is not a string, or has no `MAKESTRING` method that answers one,
raises `SYNTAX 88.909`, and one argument is required (`93.903`, none). Sent as
a message it answers no value. Only the receiver gets the name, and the name
survives `copy`. A String that is no instance of a subclass and no enhancement
is refused, as [Deviation 36](r:class.object.deviation.36) states.

```rexx
a = .plain~new
b = .plain~new
a~objectName = 'first'
c = a~copy
c~objectName = 'second'
say a~objectName b~objectName c~objectName
call try 'a~"OBJECTNAME="(.nil)'
call try 'a~"OBJECTNAME="'
call try 'a~"OBJECTNAME="("x")'
call try 'a~"OBJECTNAME="("x", "y")'
call try 'a~"OBJECTNAME="(.ms~new)'
say a~objectName
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class ms
::method makestring
  return 'madestring'
```

```output
first a PLAIN second
a~"OBJECTNAME="(.nil) => 88.909
a~"OBJECTNAME=" => 93.903
a~"OBJECTNAME="("x") => 91.999
a~"OBJECTNAME="("x", "y") => 93.902
a~"OBJECTNAME="(.ms~new) => 91.999
madestring
```

r[class.object.string]
`string` answers a string that shows the receiver, which is its `objectName`,
as [the string rule](r:class.index.string) says, and takes no argument. `say`,
the joining operators and the builtin functions use it for an object that has
no `MAKESTRING` method. A class that declares `string` shows its instances in
another way without changing their names.

```rexx
a = .plain~new
b = .shown~new
say a~string '|' b~string '|' a '|' b '|' b~objectName
a~objectName = 'nm'
say a~string '|' a
call try 'a~string(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class shown
::method string
  return 'shown!'
```

```output
a PLAIN | shown! | a PLAIN | shown! | a SHOWN
nm | nm
a~string(1) => 93.902
```

### Identity

r[class.object.hashcode]
`hashCode` answers a string, the hash code of the receiver, and takes no
argument. It answers the same string every time for one object. The collection
classes that hash their indexes use it together with `==`, as their pages
specify, so a class which declares `==` has to declare `hashCode` so that
objects which are equal under `==` have equal hash codes. For `Object` the
hash code belongs to the identity of the receiver; its form and its digits are
not specified, and two different objects may have the same one. Equal strings
have equal hash codes ([Deviation 4](r:class.object.deviation.4)).

```rexx
a = .box~new
b = .box~new
h = a~hashCode
say h~class~id (h == a~hashCode)
say .fixed~new~hashCode '|' .fixed~new~hashCode
say ('ab' || 'c')~hashCode == 'abc'~hashCode
call try 'a~hashCode(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class box
::class fixed
::method hashCode
  return 'ABCD'
```

```output
String 1
ABCD | ABCD
1
a~hashCode(1) => 93.902
```

r[class.object.identityhash]
`identityHash` answers an integer that identifies the receiver, written as
digits with an optional leading minus sign, that may have more digits than
`NUMERIC DIGITS` allows, and takes no argument. One object answers the same
number every time, and objects that exist at the same time answer different
numbers. The number is not specified further: its size and its sign are not.
The method is not changed by a `hashCode` or `==` that the class declares. The
identity collection classes compare objects by the identity that it reports,
without sending it, so a class that declares `identityHash` does not change
them. Whether equal strings or equal numbers that were built separately have
one identity is not specified ([Deviation 4](r:class.object.deviation.4)).

```rexx
a = .fixed~new
b = .fixed~new
x = a~identityHash
say (x == a~identityHash) (x == b~identityHash) (a~copy~identityHash == x)
say (a~identityHash == b~identityHash) (verify(x, '-0123456789') == 0)
call try 'a~identityHash(1)'
it = .identitytable~new
k = .key~new
it[k] = 1
say it[k] it~hasIndex(.key~new)
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class fixed
::method hashCode
  return 'ABCD'
::method "=="
  return 1
::class key
::method identityHash
  say 'identityHash sent'
  return 7
```

```output
1 0 0
0 1
a~identityHash(1) => 93.902
1 0
```

r[class.object.isnil]
`isNil` answers 1 for the object `.nil` and 0 for every other object, and
takes no argument. It does not compare the receiver with `=` or `==`, so a
class that declares those does not change it.

```rexx
say .nil~isNil .object~new~isNil 'x'~isNil .object~isNil
say .noisy~new~isNil
call try '.nil~isNil(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class noisy
::method "="
  say 'sent ='
  return 1
::method "=="
  say 'sent =='
  return 1
```

```output
1 0 0 0
0
.nil~isNil(1) => 93.902
```

### Asking about methods and classes

r[class.object.hasmethod]
`hasMethod` answers 1 when a message with the given name finds a method of the
receiver, and 0 when it does not. The name is compared in upper case. The
search is the one of [the search rule](r:class.methods.search) made for the
name, over the receiver's own methods and the classes of its order, with these
differences: it makes no check of access, so a `PRIVATE` or `PACKAGE` method
counts; a name that is hidden counts as not found; and `UNKNOWN` plays no
part, so an object whose `UNKNOWN` answers every message still answers 0 for a
name that has no method. A class receiver is searched in its class-side order,
so `hasMethod` shows [which side](r:class.index.sides) a name is on. The
argument is a string or an object whose `MAKESTRING` method answers one; any
other object, `.nil` included, raises `SYNTAX 88.909`, and no argument raises
`93.903`. A second argument raises `93.902`.

```rexx
a = .box~new
say a~hasMethod('copy') a~hasMethod('COPY') a~hasMethod('Nosuchzz')
say a~hasMethod('secret') a~hasMethod('visible') a~hasMethod('init')
say a~hasMethod(.name~new) .catchall~new~hasMethod('anything')
a~give('OWN', 'return 1')
say a~hasMethod('own') .box~new~hasMethod('own')
a~hide('OWN')
say a~hasMethod('own')
say .box~hasMethod('new') .box~new~hasMethod('new') .box~hasMethod('give')
call try 'a~hasMethod'
call try 'a~hasMethod(.nil)'
call try 'a~hasMethod(.object~new)'
call try 'a~hasMethod("A", "B")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class box
::method secret private
  return 1
::method visible
  return 1
::method give
  use arg name, src
  self~setMethod(name, src)
::method hide
  use arg name
  self~setMethod(name)
::class name
::method makestring
  return 'visible'
::class catchall
::method unknown
  return 1
```

```output
1 1 0
1 1 1
1 0
1 0
0
1 0 0
a~hasMethod => 93.903
a~hasMethod(.nil) => 88.909
a~hasMethod(.object~new) => 88.909
a~hasMethod("A", "B") => 93.902
```

r[class.object.instancemethod]
`instanceMethod` answers the method object that a message of the given name
finds for the receiver, and `.nil` when the search finds no method or finds a
name that is hidden. The search is that of
[`hasMethod`](r:class.object.hasmethod): the receiver's own methods first,
then the classes of its order, in upper case, with no access check. The answer
is the method object that the class holds, so two requests for one method
answer the same object, and a method object of a class table is the one
`method` of `Class` answers. A name that finds nothing is not an error. The
argument is checked as for `hasMethod`: none raises `SYNTAX 93.903`, `.nil`
raises `88.909`, and a second argument raises `93.902`.

```rexx
a = .box~new
m = a~instanceMethod('hello')
say m~class~id m~source[1]~strip
say (m == a~instanceMethod('HELLO')) (m == .box~method('hello'))
say a~instanceMethod('secret')~isPrivate a~instanceMethod('nosuchzz')~isNil
say a~instanceMethod('copy')~class~id,
  (a~instanceMethod('copy')~scope == .object)
say .sub~new~instanceMethod('hello')~source[1]~strip
a~give('OWN', 'return "own"')
say a~instanceMethod('own')~source[1] .box~new~instanceMethod('own')~isNil
a~hide('OWN')
say a~instanceMethod('own')~isNil
.sub~define('HELLO')
say .sub~new~instanceMethod('hello')~isNil .sub~new~hasMethod('hello')
call try 'a~instanceMethod'
call try 'a~instanceMethod(.nil)'
call try 'a~instanceMethod("hello", 1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class box
::method hello
  return 'hello'
::method secret private
  return 1
::method give
  use arg name, src
  self~setMethod(name, src)
::method hide
  use arg name
  self~setMethod(name)
::class sub subclass box
```

```output
Method return 'hello'
1 1
1 1
Method 1
return 'hello'
return "own" 1
1
1 0
a~instanceMethod => 93.903
a~instanceMethod(.nil) => 88.909
a~instanceMethod("hello", 1) => 93.902
```

r[class.object.instancemethods]
`instanceMethods` answers a supplier whose indexes are method names in upper
case and whose items are method objects. It lists names that exist when it is
sent, and a later change to a class or to the receiver does not reach the
supplier. The order of its entries is not specified
([Deviation 33](../deviations.md)). Without an argument it lists every method
of the receiver's own methods and of every class of its order, so a name that
more than one of them holds is listed once for each. With a class that is in
the order of the receiver as its argument, it lists the methods of that
class's table, and with `.nil` the receiver's own methods. With any other
argument, a number or a string or a class outside the order included, the
supplier has no entries. A table entry that hides a name is not listed; the
name is still listed from every other table of the order that holds a method
for it, so the listing does not show what the search finds. The argument may
be given once: a second raises `SYNTAX 93.902`. For a class receiver the order
is the [class-side one](r:class.methods.class-side).

```rexx
call show .box~new~instanceMethods(.box), 'box'
call show .sub~new~instanceMethods(.sub), 'sub'
call show .sub~new~instanceMethods(.box), 'sub as box'
call show .sub~new~instanceMethods(.mix), 'sub as mix'
call show .box~new~instanceMethods(.sub), 'box as sub'
call show .box~new~instanceMethods(.nil), 'box own'
call show .box~new~instanceMethods(1), 'number'
call show .box~new~instanceMethods('BOX'), 'string'
call show .box~instanceMethods(.box), 'class box'
call show .cm~instanceMethods(.cm), 'class cm'
call show .cm~new~instanceMethods(.cm), 'cm object'
o = .sub~new
o~give('OWN', 'return 1')
call show o~instanceMethods(.nil), 'own'
t = o~instanceMethods(.sub)
.sub~define('M3', 'return 3')
call show t, 'snapshot'
call show o~instanceMethods(.sub), 'after'
n = 0
t = .sub~new~instanceMethods
do while t~available
  if t~index == 'HELLO' then n = n + 1
  t~next
end
say 'HELLO listed' n 'times'
h = .sub~new
h~hide('HELLO')
n = 0
t = h~instanceMethods
do while t~available
  if t~index == 'HELLO' then n = n + 1
  t~next
end
say 'hidden HELLO listed' n 'times' h~hasMethod('hello')
exit
show:
  t = arg(1)
  a = .array~new
  do while t~available
    a~append(t~index'='t~item~class~id)
    t~next
  end
  say arg(2)':' a~items a~sort~makestring('l', ' ')
  return
::class box
::method hello
  return 'box hello'
::method give
  use arg name, src
  self~setMethod(name, src)
::method hide
  self~setMethod(arg(1))
::class mix mixinclass object
::method mixed
  return 1
::class sub subclass box inherit mix
::method hello
  return 'sub hello'
::method m2
  return 2
::class cm
::method only class
  return 1
```

```output
box: 3 GIVE=Method HELLO=Method HIDE=Method
sub: 2 HELLO=Method M2=Method
sub as box: 3 GIVE=Method HELLO=Method HIDE=Method
sub as mix: 1 MIXED=Method
box as sub: 0 
box own: 0 
number: 0 
string: 0 
class box: 0 
class cm: 1 ONLY=Method
cm object: 0 
own: 1 OWN=Method
snapshot: 2 HELLO=Method M2=Method
after: 2 HELLO=Method M2=Method
HELLO listed 2 times
hidden HELLO listed 2 times 0
```

r[class.object.isa]
`isA` answers 1 when the argument is the class of the receiver, or a class in
the order of that class: a superclass, an ancestor, or an inherited mixin
class. The argument must be a class: another object raises `SYNTAX 88.914`, no
argument `88.901`, and a second argument `93.902`. For a class object the
order is its class-side order, so `.sub~isA(.class)` and `.sub~isA(.object)`
are 1, while `.sub~isA(.sub)` and `.sub~isA(.box)` are 0. The method is
independent of [`isInstanceOf`](r:class.object.isinstanceof): neither sends
the other.

```rexx
a = .sub~new
say a~isA(.sub) a~isA(.box) a~isA(.object) a~isA(.mix) a~isA(.other),
  a~isA(.class)
say .sub~isA(.class) .sub~isA(.object) .sub~isA(.sub) .sub~isA(.box)
say 'abc'~isA(.string) 'abc'~isA(.comparable),
  .array~new~isA(.orderedcollection)
say .twist~new~isA(.twist) .twist~new~isInstanceOf(.twist)
call try 'a~isA'
call try 'a~isA(1)'
call try 'a~isA(.nil)'
call try 'a~isA("SUB")'
call try 'a~isA(.box, 1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class box
::class mix mixinclass object
::class sub subclass box inherit mix
::class other
::class twist
::method isInstanceOf
  return 'overridden'
```

```output
1 1 1 1 0 0
1 1 0 0
1 1 1
1 overridden
a~isA => 88.901
a~isA(1) => 88.914
a~isA(.nil) => 88.914
a~isA("SUB") => 88.914
a~isA(.box, 1) => 93.902
```

r[class.object.isinstanceof]
`isInstanceOf` answers 1 under the same conditions as
[`isA`](r:class.object.isa), with the same checks of its argument, and is not
changed by a class that declares `isA`.

```rexx
a = .sub~new
say a~isInstanceOf(.sub) a~isInstanceOf(.box) a~isInstanceOf(.mix),
  a~isInstanceOf(.other)
say .sub~isInstanceOf(.class) .sub~isInstanceOf(.sub)
say .twist~new~isInstanceOf(.twist) .twist~new~isA(.twist)
call try 'a~isInstanceOf'
call try 'a~isInstanceOf(1)'
call try 'a~isInstanceOf(.box, 1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class box
::class mix mixinclass object
::class sub subclass box inherit mix
::class other
::class twist
::method isA
  return 'overridden'
```

```output
1 1 1 0
1 0
1 overridden
a~isInstanceOf => 88.901
a~isInstanceOf(1) => 88.914
a~isInstanceOf(.box, 1) => 93.902
```

### Conversion

r[class.object.request]
`request` answers the receiver converted to the kind of object that its
argument names, or `.nil`. The argument is a string, taken in upper case. The
method first asks whether the receiver has a method named `MAKE` followed by
that name; if it has, `request` sends that message to the receiver with no
argument and answers what it answers, whatever that is, and the answer is not
checked: a method may answer `.nil` or an object of another class. The message
is checked for access as if the code that sent `request` had sent it, as for
[`send`](r:class.object.send): a `PRIVATE` method runs when `request` is sent
from a method of its class and raises `SYNTAX 97.2` from elsewhere, and a
`PACKAGE` method raises `97.3` when `request` is sent from another package.
When there is no such method, `UNKNOWN` plays no part, the method compares the
name with the id of the receiver's class, without regard to case: when they
are equal it answers the receiver itself, and otherwise `.nil`. The id is that
of the class of the receiver only, not of its superclasses. The argument is
checked as for `hasMethod`: none raises `93.903`, `.nil` and objects without a
string value `88.909`, and a second argument `93.902`.

```rexx
o = .doc~new
say o~request('string') '|' o~request('Array')~class~id '|',
  o~request('doc')~class~id
say o~request('nosuchzz') '|' o~request('') '|' o~request('object')
say .sub~new~request('doc') '|' .sub~new~request('sub')~class~id
say .doc~new~request('plain') '|' .doc~new~request('lazy')
say .doc~new~request('wild')~class~id
say .catchall~new~request('anything')
say 'abc'~request('string') '|' 12~request('integer') '|',
  'x'~request('string')~class~id
call try 'o~request'
call try 'o~request(.nil)'
call try 'o~request(.object~new)'
call try 'o~request("string", 1)'
call try 'o~request("secret")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class doc
::method makestring
  return 'as string'
::method makearray
  return .array~of(1, 2)
::method makeplain
  return 'made plain'
::method makelazy
  return .nil
::method makewild
  return .object~new
::method makesecret private
  return 1
::class sub subclass doc
::class catchall
::method unknown
  return 'unknown was run'
```

```output
as string | Array | DOC
The NIL object | The NIL object | The NIL object
The NIL object | SUB
made plain | The NIL object
Object
The NIL object
abc | The NIL object | String
o~request => 93.903
o~request(.nil) => 88.909
o~request(.object~new) => 88.909
o~request("string", 1) => 93.902
o~request("secret") => 97.2
```

r[class.object.request.access]
The access check of `request` is that of the code that sent it. In the
example, a class in a package of its own has a method that sends `request`
and reaches the `PRIVATE` and `PACKAGE` methods, and the program of another
package does not.

```rexx
f = 'spec-class-object-request-zq.tmp'
call sysfiledelete f
call lineout f, '::class doc public'
call lineout f, '::method makepk package'
call lineout f, '  return "pk ran"'
call lineout f, '::method makesecret private'
call lineout f, '  return "secret ran"'
call lineout f, '::method inner'
call lineout f, '  signal on syntax name b'
call lineout f, '  return self~request(arg(1))'
call lineout f, 'b: return condition("o")~code'
call lineout f
src = .array~of('o = .doc~new', -
  'say o~inner("pk") o~inner("secret")', -
  'signal on syntax name b', -
  'say o~request("pk")', -
  'b: say condition("o")~code', -
  'signal on syntax name c', -
  'say o~request("secret")', -
  'c: say condition("o")~code', -
  '::requires "'f'"')
p = .package~new('spec-class-object-request-main', src)
call sysfiledelete f
```

```output
pk ran secret ran
97.3
97.2
```

r[class.object.request.sends]
`request` sends one message to its receiver: `MAKE` followed by the upper case
name of its argument, with no argument, when the receiver has a method of that
name (see [`hasMethod`](r:class.object.hasmethod)). It sends nothing else to
the receiver: it does not send `class`, so a class that declares `class` does
not change which id is compared, and it sends no `UNKNOWN`. `request` sends
nothing to the argument other than what the
[string conversion](r:conv.string.object) sends.

```rexx
o = .watch~new
r = o~request('watch')
say (r == o)
r = o~request('abc')
say r
r = o~request('other')
say r
l = .liar~new
r = l~request('liar')
say (r == l)
r = l~request('box')
say r
::class watch
::method makeabc package
  say 'makeabc sent' arg()
  return 'ABC'
::method class
  say 'class sent'
  return .watch
::method unknown
  say 'unknown sent' arg(1)
  return 'u'
::class liar
::method class
  return .box
::class box
```

```output
1
makeabc sent 0
ABC
The NIL object
1
The NIL object
```

### Sending messages

r[class.object.send]
`send` sends the message that its first argument names to the receiver, with
the remaining arguments as the arguments of that message, in order, and
answers what the message answers. An omitted argument stays omitted in the
message, and omitted places at the end add nothing to the argument count. The
message is an ordinary one: it is found by [the search
rule](r:class.methods.search) and so also goes to `UNKNOWN` when nothing is
found, raises NOMETHOD (`SYNTAX 97.1` when untrapped) when there is no
`UNKNOWN`, and is checked for access as if the code that called `send` had
sent it, so `send('secret')` from outside the class raises `97.2` for a
private method that no `UNKNOWN` takes. The name is taken in upper case. The
first argument is a string, or an array of two items, the name and a class,
which starts the search at that class as [the scoped send
rule](r:class.methods.scoped.send) says; `.nil` raises `SYNTAX 93.972`, an
array of another size `93.946`, an array whose first item is missing
`88.901` and one whose first item has no string value `88.909`, and no
argument `88.901`. The null string is a name like any other: it names the
abuttal method ([the joining rule](r:class.object.concat)), so
`send('', 'x')` joins and `send('')` raises that method's `93.903`. An array
whose second item is not a class raises `88.914`. A message
that answers nothing makes `send` answer nothing.

```rexx
o = .w~new
say o~send('hello', 'W', 'F')
say o~send('HeLLo', 'W', 'F')
say o~send('count') '|' o~send('count', 1, , 3) '|' o~send('count', 1, )
say o~send(.array~of('hello', .w), 'A', 'B')
u = .watcher~new
say u~send('nosuchzz', 1, 2)
say o~send('inside', 'secret') o~send('pack')
call try 'o~send'
call try 'o~send(.nil)'
call try 'o~send("", "x")'
call try 'o~send("")'
call try 'o~send("nosuchzz")'
call try 'o~send(.array~of("count"))'
call try 'o~send(.array~of("count", .w, 3))'
call try 'o~send(.array~of(.nil, .w))'
call try 'o~send(.array~of(.object~new, .w))'
call try 'o~send(.array~new(2)~~put(.w, 2))'
call try 'o~send("secret")'
call try 'o~send("nothing")'
call try 'o~send(.array~of("count", 1))'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class w
::method hello
  use strict arg place, name
  return 'Hello' place', I am' name
::method count
  return arg() '|' arg(1, 'e') arg(2, 'e') arg(3, 'e')
::method secret private
  return 'secret ran'
::method pack package
  return 'pack ran'
::method inside
  return self~send(arg(1))
::method nothing
::class watcher
::method unknown
  return 'unknown' arg(1) arg(2)~items
```

```output
Hello W, I am F
Hello W, I am F
0 | 0 0 0 | 3 | 1 0 1 | 1 | 1 0 0
Hello A, I am B
unknown NOSUCHZZ 2
secret ran pack ran
o~send => 88.901
o~send(.nil) => 93.972
o~send("", "x") => a Wx
o~send("") => 93.903
o~send("nosuchzz") => 97.1
o~send(.array~of("count")) => 93.946
o~send(.array~of("count", .w, 3)) => 93.946
o~send(.array~of(.nil, .w)) => 88.909
o~send(.array~of(.object~new, .w)) => 88.909
o~send(.array~new(2)~~put(.w, 2)) => 88.901
o~send("secret") => 97.2
o~send("nothing") => 91.999
o~send(.array~of("count", 1)) => 88.914
```

r[class.object.sendwith]
`sendWith` is `send` with the arguments of the message in an array. The first
argument names the message as for [`send`](r:class.object.send), and is
checked first. The second argument is an array, or an object that answers an
array to the message `request` with the argument `ARRAY`, which is what
`sendWith` sends it; an object that answers anything but an array, including
`.nil`, and an array of more than one dimension, raise `SYNTAX 88.913`. The
items of the array are the arguments in order; a place of a sparse array that
holds nothing is an omitted argument, and the argument count is the position
of the last item that is present. Both arguments are required (`88.901`, one
missing) and a third raises `93.902`.

```rexx
o = .w~new
say o~sendWith('hello', .array~of('W', 'F'))
say o~sendWith('count', .array~of(1, , 3))
say o~sendWith('count', .array~new(3)~~put(5, 2))
say o~sendWith('count', .array~new(3))
say o~sendWith('count', .list~of(1, 2))
say o~sendWith('count', .holder~new)
say o~sendWith('count', 'abc')
call try 'o~sendWith'
call try 'o~sendWith("count")'
call try 'o~sendWith("count", .nil)'
call try 'o~sendWith("count", .object~new)'
call try 'o~sendWith("count", .array~new(2, 2))'
call try 'o~sendWith("count", .array~new, 3)'
call try 'o~sendWith(.nil, .nil)'
call try 'o~sendWith("nosuchzz", .nil)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class w
::method hello
  use strict arg place, name
  return 'Hello' place', I am' name
::method count
  return arg() '|' arg(1, 'e') arg(2, 'e') arg(3, 'e')
::class holder
::method makearray
  return .array~of('x', 'y', 'z')
```

```output
Hello W, I am F
3 | 1 0 1
2 | 0 1 0
0 | 0 0 0
2 | 1 1 0
3 | 1 1 1
1 | 1 0 0
o~sendWith => 88.901
o~sendWith("count") => 88.901
o~sendWith("count", .nil) => 88.913
o~sendWith("count", .object~new) => 88.913
o~sendWith("count", .array~new(2, 2)) => 88.913
o~sendWith("count", .array~new, 3) => 93.902
o~sendWith(.nil, .nil) => 93.972
o~sendWith("nosuchzz", .nil) => 88.913
```

r[class.object.sendwith.sends]
`sendWith` sends its second argument the message `request` with the argument
`ARRAY`, once, before it sends the message that the first argument names. A
class whose `request` or `makeArray` is overridden shows it, and the answer of
`request` is checked as the previous rule says.

```rexx
o = .w~new
say o~sendWith('count', .logged~new)
::class w
::method count
  return 'count got' arg()
::class logged
::method request
  say 'request' arg(1)
  forward class (super)
::method makearray
  say 'makearray sent'
  return .array~of(1, 2)
```

```output
request ARRAY
makearray sent
count got 2
```

r[class.object.start]
`start` makes a message object for the message that its first argument names,
with the remaining arguments, and starts it so that it is run by another
activity, while the sender goes on. It answers the message object at once and
does not wait for the message to run. The name and the arguments are checked
by `start` as `send` does, with the same errors raised at the call to `start`,
except that the null string is a name like any other, so `start('')` answers a
message object; an error in the message itself, such as a name with no method
or the abuttal method's missing argument, is not raised by `start`: the
message object keeps it, and it is raised when the result is asked for. The
access check is that of [Deviation 105](r:dir.method.deviation.105). How the
message is scheduled and what the message object offers is specified in the
environment chapter and with the `Message` class; the examples show only
results that no scheduling changes.

```rexx
o = .w~new
m = o~start('hello', 'W', 'F')
say m~class~id m~result
m = o~start(.array~of('count', .w), 1, , 3)
say m~result
m = o~start('nothing')
say m~result~isNil
m = o~start('nosuchzz')
say m~class~id o~start('')~class~id
signal on syntax name late
x = m~result
say 'no error'
late: say 'raised by result:' condition('o')~code
signal off syntax
call try 'o~start'
call try 'o~start(.nil)'
call try 'o~start(.array~of("count"))'
call try 'o~start(.array~of("count", 1))'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r~class~id
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class w
::method hello
  use strict arg place, name
  return 'Hello' place', I am' name
::method count
  return arg() '|' arg(1, 'e') arg(2, 'e') arg(3, 'e')
::method nothing
```

```output
Message Hello W, I am F
3 | 1 0 1
1
Message Message
raised by result: 97.1
o~start => 88.901
o~start(.nil) => 93.972
o~start(.array~of("count")) => 93.946
o~start(.array~of("count", 1)) => 88.914
```

r[class.object.startwith]
`startWith` is `start` with the arguments of the message in an array: the
first argument names the message and is checked first, the second is an array
or an object that answers one to `request('ARRAY')`, which is how the
arguments are read, as for [`sendWith`](r:class.object.sendwith). It answers
the message object at once. The errors of the array are raised at the call,
with two differences from `sendWith`: an object that answers no array raises
`SYNTAX 98.913`, where `sendWith` raises `88.913`, and a missing array raises
`93.903`, where `sendWith` raises `88.901`. With no argument at all the error
is `88.901`.

```rexx
o = .w~new
m = o~startWith('hello', .array~of('W', 'F'))
say m~class~id m~result
m = o~startWith('count', .array~of(1, , 3))
say m~result
call try 'o~startWith'
call try 'o~startWith("count")'
call try 'o~startWith("count", .nil)'
call try 'o~startWith("count", 1, 2)'
call try 'o~startWith(.nil, .array~new)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r~class~id
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class w
::method hello
  use strict arg place, name
  return 'Hello' place', I am' name
::method count
  return arg() '|' arg(1, 'e') arg(2, 'e') arg(3, 'e')
```

```output
Message Hello W, I am F
3 | 1 0 1
o~startWith => 88.901
o~startWith("count") => 93.903
o~startWith("count", .nil) => 98.913
o~startWith("count", 1, 2) => 93.902
o~startWith(.nil, .array~new) => 93.972
```

r[class.object.startwith.sends]
`startWith` sends the message `request` with the argument `ARRAY` to its
second argument, once, at the call, and not when the message runs.

```rexx
o = .w~new
m = o~startWith('count', .logged~new)
say 'started'
say m~result
::class w
::method count
  return 'count got' arg()
::class logged
::method request
  say 'request' arg(1)
  forward class (super)
::method makearray
  say 'makearray sent'
  return .array~of(1, 2)
```

```output
request ARRAY
makearray sent
started
count got 2
```

### The receiver's own methods

r[class.object.setmethod]
`setMethod` gives the receiver a method of its own, as
[the object method rule](r:class.methods.object) says, and answers no value.
It is private, with the restriction that
[the access rule](r:class.methods.object.access) gives. The first argument is
the name and is required (`SYNTAX 88.901`); an object that is no string and
has no `MAKESTRING` method raises `88.909`. The second is the method: a method
object, a string that holds the source of a method, or an array of strings,
one for each line. These are instances of `Method`, `String` and `Array`
themselves; an instance of a subclass of one of them, `Method` included, is
converted as any other object is. Any other object is sent `request` with the
argument `ARRAY`, and the array it answers is used; when it answers no array,
it is sent `request` with the argument `STRING`, and a string answer is used
as the source ([the sends rule](r:class.object.setmethod.sends)). `.nil`, and
an object that answers neither, raise `SYNTAX 93.974`, so a subclass of
`Method` is refused. A method object is copied: the object gets a copy whose
scope is set as [the scope rule](r:class.methods.object.scope) says, and the
argument itself is not changed, so `instanceMethod` answers the copy. The
checks run in this order: the number of arguments, the name, the method, the
scope. With no second argument the name is hidden for the receiver. The third
argument is the scope, `FLOAT` or `OBJECT` as
[the scope rule](r:class.methods.object.scope) says; another value raises
`88.916`, and `.nil` raises `88.909`. A fourth argument raises `93.902`. A
source that does not translate raises the translation error from `setMethod`.
The code of a method made from a source runs with the scope `FLOAT`, whose
object variables are separate from those of any class: the methods that the
receiver got by `setMethod` with `FLOAT`, by `run` and by `enhanced` share one
set, which is kept for the receiver.

```rexx
o = .w~new
call try 'o~sm("A1", "return 1")'
call try 'x = o~smv'
say o~a1
call try 'o~sm("a2", .method~new("", "return 2"))'
say o~a2
call try 'o~sm("a3", .array~of("x = 3", "return x + 30"))'
say o~a3
call try 'o~sm("a4", "return 4", "float")'
call try 'o~sm("a5", "return 5", "Object")'
call try 'o~sm("a6", "return 6", "x")'
call try 'o~sm("a6", "return 6", .nil)'
call try 'o~sm("a7", .list~of("return 7"))'
say o~a7
call try 'o~sm("a8")'
say o~hasMethod('a8') o~hasMethod('a1')
call try 'o~sm("a9", .nil)'
call try 'o~sm("a9", .object~new)'
call try 'o~sm("a9", "return 1", "f", 4)'
call try 'o~sm'
call try 'o~sm(.nil)'
call try 'o~sm("a9", "if then")'
call try 'o~sm(.nil, .nil)'
call try 'o~sm("a9", .nil, "x")'
call try 'o~sm("a9", "if then", "x")'
call try 'o~sm("a9", .lmeth~new("", "return 1"))'
m = .method~new("", 'return "mobj"')
o~sm('M1', m)
o~sm('M1b', m)
say (o~instanceMethod('M1') == m),
  (o~instanceMethod('M1') == o~instanceMethod('M1b')) m~scope
o~sm('SETX', 'expose x; x = 41; return x')
o~sm('GETX', 'expose x; return x + 1')
say o~setx o~getx o~x
call try 'o~setMethod("outside", "return 1")'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class w
::attribute x
::method init
  expose x
  x = 'class x'
::method sm
  if arg() = 0 then self~setMethod
  else if arg() = 1 then self~setMethod(arg(1))
  else if arg() = 2 then self~setMethod(arg(1), arg(2))
  else if arg() = 3 then self~setMethod(arg(1), arg(2), arg(3))
  else self~setMethod(arg(1), arg(2), arg(3), arg(4))
::method smv
  return self~setMethod('V', 'return 1')
::class lmeth subclass method
```

```output
o~sm("A1", "return 1") => no error
x = o~smv => 91.999
1
o~sm("a2", .method~new("", "return 2")) => no error
2
o~sm("a3", .array~of("x = 3", "return x + 30")) => no error
33
o~sm("a4", "return 4", "float") => no error
o~sm("a5", "return 5", "Object") => no error
o~sm("a6", "return 6", "x") => 88.916
o~sm("a6", "return 6", .nil) => 88.909
o~sm("a7", .list~of("return 7")) => no error
7
o~sm("a8") => no error
0 1
o~sm("a9", .nil) => 93.974
o~sm("a9", .object~new) => 93.974
o~sm("a9", "return 1", "f", 4) => 93.902
o~sm => 88.901
o~sm(.nil) => 88.909
o~sm("a9", "if then") => 35.929
o~sm(.nil, .nil) => 88.909
o~sm("a9", .nil, "x") => 93.974
o~sm("a9", "if then", "x") => 35.929
o~sm("a9", .lmeth~new("", "return 1")) => 93.974
0 0 The NIL object
41 42 class x
o~setMethod("outside", "return 1") => 97.2
```

r[class.object.setmethod.private]
The method objects of `run`, `setMethod` and `unsetMethod`, which are
private, answer 1 to `isPrivate`.

```rexx
say .object~new~instanceMethod('RUN')~isPrivate,
  .object~method('SETMETHOD')~isPrivate,
  .object~method('UNSETMETHOD')~isPrivate
```

```output defect=#133
1 1 1
```

r[class.object.setmethod.sends]
`setMethod` sends the message `request` with the argument `ARRAY`, once, to a
second argument that is not an instance of `Method`, `String` or `Array`
itself, and uses the array it answers. When that answers no array, it sends
`request` with the argument `STRING`, once, and uses the string it answers as
the source. It sends nothing to the receiver.

```rexx
o = .w~new
o~sm('M', .logged~new)
say o~m
o~sm('N', .src~new)
say o~n
::class w
::method sm
  self~setMethod(arg(1), arg(2))
::class logged
::method request
  say 'request' arg(1)
  forward class (super)
::method makearray
  say 'makearray sent'
  return .array~of('return "made by array"')
::class src subclass logged
::method makestring
  say 'makestring sent'
  return 'return "made by string"'
::method makearray
  return .nil
```

```output
request ARRAY
makearray sent
made by array
request ARRAY
request STRING
makestring sent
made by string
```

r[class.object.unsetmethod]
`unsetMethod` takes the name off the receiver again, as
[the object method rule](r:class.methods.object) says, and answers no value.
It is private with the restriction of
[the access rule](r:class.methods.object.access). The argument is the name,
required (`SYNTAX 88.901`), and a value that is no string raises `88.909`. A
second argument raises `93.902`, and the number of arguments is checked first,
so a second argument raises it even when the name is missing. The name is
taken in upper case. A name for which the receiver has no method of its own is
not an error and changes nothing, and so does a name that only a class holds:
the class's method stays. A name that `setMethod` replaced twice is taken off
with one `unsetMethod`.

```rexx
o = .w~new
o~sm('A1', 'return "first"')
o~sm('A1', 'return "second"')
say o~a1
o~um('a1')
say o~hasMethod('a1')
call try 'o~um("a1")'
call try 'x = o~umv'
call try 'o~um("nosuchzz")'
call try 'o~um("base")'
say o~base
o~sm('BASE', 'return "own base"')
say o~base
o~um('BASE')
say o~base
call try 'o~um'
call try 'o~um(.nil)'
call try 'o~um("a1", 2)'
call try 'o~um(, 2)'
call try 'o~unsetMethod("outside")'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class w
::method base
  return 'class base'
::method sm
  self~setMethod(arg(1), arg(2))
::method um
  if arg() = 0 then self~unsetMethod
  else if arg() = 1 then self~unsetMethod(arg(1))
  else self~unsetMethod(arg(1), arg(2))
::method umv
  return self~unsetMethod('A1')
```

```output
second
0
o~um("a1") => no error
x = o~umv => 91.999
o~um("nosuchzz") => no error
o~um("base") => no error
class base
own base
class base
o~um => 88.901
o~um(.nil) => 88.909
o~um("a1", 2) => 93.902
o~um(, 2) => 93.902
o~unsetMethod("outside") => 97.2
```

r[class.object.run]
`run` runs a method on the receiver once and answers what that method answers.
It is private, with the restriction of
[the access rule](r:class.methods.object.access). The first argument is the
method: a method object, a string that holds the source of a method, or an
array of strings, one for each line, which `run` turns into a method as
`setMethod` does ([the argument rule](r:class.object.setmethod)), with the
same conversions and the same errors for a value that is no method or fails to
translate, and `SYNTAX 88.901` when the argument is missing. The method is not
kept. The second argument is an option, whose first letter counts and the rest
is ignored: `I` passes the remaining arguments to the method one for each, in
order, with omitted ones omitted, and `A` passes the items of the third
argument, converted as [`sendWith`](r:class.object.sendwith) converts its
array: a missing array raises `SYNTAX 88.901`, and one that converts to no
array, or to an array of more than one dimension, raises `88.913`. A sparse
array's empty places are omitted arguments. Without an option the method gets
no arguments, and arguments after an omitted option raise `88.901`. Any other
option, the null string included, raises `SYNTAX 93.915`; `.nil` raises
`88.909`. The checks run in this order: the method, the option, the array. The
code runs with the scope `FLOAT`: its object variables are those of
`setMethod`'s floating methods, not those of any class. When the method
answers nothing, `run` answers nothing.

```rexx
o = .w~new
say o~go('return "plain" arg()')
say o~go('return arg() "|" arg(1) arg(2)', 'I', 'a', 'b')
say o~go('return arg() "|" arg(1) arg(2)', 'individual', 'a', 'b')
say o~go('return arg() "|" arg(1) arg(2)', 'Array', .array~of('a', 'b'))
say o~go('return arg() "|" arg(1, "e") arg(2, "e") arg(3, "e")', 'a', -
  .array~of(1, , 3))
say o~go('return arg() "|" arg(1, "e") arg(2, "e")', 'i', 1, )
say o~go('return arg()', 'A', .array~new)
say o~go('return arg()', 'A', .list~of(1, 2, 3))
say o~go(.array~of('x = 5', 'return x * 2'))
say o~go(.method~new('', 'return "method object"'))
say o~go('expose v; v = 7; return v')
say o~go('expose v; return v + 1')
say o~v
say o~go('return self~class~id')
call try 'o~run("return 1")'
call try 'o~go'
call try 'o~go(.nil)'
call try 'o~go("return 1", "x")'
call try 'o~go("return 1", "")'
call try 'o~go("return 1", .nil)'
call try 'o~go("return 1", "A")'
call try 'o~go("return 1", "A", .nil)'
call try 'o~go("return 1", "A", .array~new(2, 2))'
call try 'o~go2("return arg()", 1)'
call try 'o~go(.object~new, "x")'
call try 'o~go("return 1", "x", .nil)'
call try 'o~go(.lmeth~new("", "return 1"))'
call try 'o~go("return 1", "I", 1, 2)'
call try 'o~go("if then")'
call try 'o~go("say 1")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class w
::attribute v
::method init
  expose v
  v = 'class v'
::method go
  if arg() = 0 then return self~run
  if arg() = 1 then return self~run(arg(1))
  if arg() = 2 then return self~run(arg(1), arg(2))
  if arg() = 3 then return self~run(arg(1), arg(2), arg(3))
  return self~run(arg(1), arg(2), arg(3), arg(4))
::method go2
  return self~run(arg(1), , arg(2))
::class lmeth subclass method
```

```output
plain 0
2 | a b
2 | a b
2 | a b
3 | 1 0 1
1 | 1 0
0
3
10
method object
7
8
class v
W
o~run("return 1") => 97.2
o~go => 88.901
o~go(.nil) => 93.974
o~go("return 1", "x") => 93.915
o~go("return 1", "") => 93.915
o~go("return 1", .nil) => 88.909
o~go("return 1", "A") => 88.901
o~go("return 1", "A", .nil) => 88.913
o~go("return 1", "A", .array~new(2, 2)) => 88.913
o~go2("return arg()", 1) => 88.901
o~go(.object~new, "x") => 93.974
o~go("return 1", "x", .nil) => 93.915
o~go(.lmeth~new("", "return 1")) => 93.974
o~go("return 1", "I", 1, 2) => 1
o~go("if then") => 35.929
1
o~go("say 1") => 91.999
```

r[class.object.run.sends]
`run` sends the message `request` with the argument `ARRAY`, once, to a method
argument that is not an instance of `Method`, `String` or `Array` itself, and
uses the array it answers; when that answers no array, it sends `request` with
the argument `STRING`, once, and uses the string it answers as the source, as
[`setMethod`](r:class.object.setmethod.sends) does. It also sends `request`
with the argument `ARRAY`, once, to the third argument of the option `A`, and
uses the array it answers. It sends nothing to the receiver.

```rexx
o = .w~new
say o~go(.logged~new)
say o~go('return arg()', 'A', .logged~new)
say o~go(.src~new)
::class w
::method go
  if arg() = 1 then return self~run(arg(1))
  return self~run(arg(1), arg(2), arg(3))
::class logged
::method request
  say 'request' arg(1)
  forward class (super)
::method makearray
  say 'makearray sent'
  return .array~of('return 1', 'return 2')
::class src subclass logged
::method makestring
  say 'makestring sent'
  return 'return "from string"'
::method makearray
  return .nil
```

```output
request ARRAY
makearray sent
1
request ARRAY
makearray sent
2
request ARRAY
request STRING
makestring sent
from string
```

## Deviations

r[class.object.deviation.4]
Whether two equal strings or numbers that were built separately are one object
is not specified, and shows only through `identityHash` and the identity
collections ([Deviation 4](../deviations.md)). The `hashCode` of equal strings
is equal, and so is any value comparison; ooRexx often answers a new object
where interrexx answers the existing one.

```rexx
a = 'ab' || 'c'
b = 'a' || 'bc'
say (a == b) (a~hashCode == b~hashCode)
say (a~hashCode == 'abc'~hashCode) (a~copy == a)
n = 10 + 2
say (n == 12) (n~hashCode == 12~hashCode)
```

```output
1 1
1 1
1 1
```

r[class.object.deviation.28]
A method of a built-in class that `setMethod` gives to an object of an
unrelated class, or `run` or `enhanced` runs on one, is refused: the program
ends with exit code 120. ooRexx reads the object with the layout of the other
class and answers garbage or crashes. The check happens when the method is
run, so `setMethod` itself still answers no value.

```rexx
say 'before'
m = .array~method('ITEMS')
o = .w~new
o~sm(m)
say 'after'
say o~items
::class w
::method sm
  self~setMethod('ITEMS', arg(1))
```

```output rc=120
before
after
```

r[class.object.deviation.30]
When the `string` method of an object answers an object that is not a string,
interrexx gives the answer ooRexx gives wherever that is defined, and
elsewhere ends the program with exit code 120, as [the chapter
rule](r:class.index.deviation.30) says. The `string` method of `Object` is not
affected: it answers the name.

```rexx
a = .weird~new
say a~string~class~id
say 'before'
say 'x' a
::class weird
::method string
  return .directory~new
```

```output rc=120
Directory
before
```

r[class.object.deviation.32]
When the class method `new` of a class answers an object that it did not
build, `enhanced` leaves that object as it is, as [the chapter
rule](r:class.index.deviation.32) says; ooRexx adds the methods to it and
marks it enhanced, which changes every object of its class.

```rexx
m = .table~new
m['HI'] = 'return "hi"'
q = .odd~enhanced(m)
say q~class~id q~hasMethod('HI')
say 'xyz'~class~id 'xyz'~hasMethod('HI')
::class odd
::method new class
  return 'abc'
```

```output
String 0
String 0
```

r[class.object.deviation.33]
The order of the entries that `instanceMethods` yields is not specified and
may differ from ooRexx's ([Deviation 33](../deviations.md)); which names are
listed, and how often, is specified.

```rexx
t = .box~new~instanceMethods(.box)
a = .array~new
do while t~available
  a~append(t~index)
  t~next
end
say a~sort~makestring('l', ' ')
::class box
::method hello
  return 1
::method goodbye
  return 2
```

```output
GOODBYE HELLO
```

r[class.object.deviation.105]
A message that `start` or `startWith` sends is checked for `PRIVATE` and
`PACKAGE` access as the same message sent directly by the code that started
it, as [the Deviation 105 rule](r:dir.method.deviation.105) states; ooRexx
raises `97.2` for every started private message and `97.1` for every started
`PACKAGE` message.

```rexx
say .safe~new~open
::class safe
::method open unguarded
  return self~startWith('inner', .array~of(7))~result
::method inner private
  return 'inner got' arg(1)
```

```output
inner got 7
```

r[class.object.deviation.36]
`objectName=` sent to a String that is no instance of a subclass and not
enhanced, a number included, ends the program with exit code 120, as [the
chapter rule](r:class.index.deviation.36) says; ooRexx gives the name to that
string. An instance of a subclass of `String` takes a name.

```rexx
s = .sub~new('abc')
s~objectName = 'named'
say s~objectName
say 'before'
t = 'abc'
t~objectName = 'named'
say 'after'
::class sub subclass string
```

```output rc=120
named
before
```
