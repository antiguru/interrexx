---
element: "Class"
kind: class
rexxref: "Builtin Classes > Fundamental Classes > Class Class (Metaclass); Builtin Classes > Fundamental Classes > Class Class (Metaclass) > Comparison Methods; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > activate; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > annotation; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > annotations; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > baseClass; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > defaultName; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > define; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > defineMethods; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > delete; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > enhanced; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > id; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > inherit; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > isAbstract; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > isMetaclass; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > isSubclassOf; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > metaClass; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > method; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > methods; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > mixinClass; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > new; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > package; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > queryMixinClass; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > subclass; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > subclasses; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > superClass; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > superClasses; Builtin Classes > Fundamental Classes > Class Class (Metaclass) > uninherit"
deviations: [32, 33, 63, 108, 109, 110, 114]
---

# Class

`Class` is the class of the class objects: every class is an instance of it,
or of a subclass of it, which is a metaclass. Its instance methods are
therefore what every class object understands: the methods that make and
change classes (`subclass`, `mixinClass`, `define`, `inherit` and the others),
that answer questions about a class (`id`, `superClasses`, `method`) and that
make objects (`enhanced`). `Class` is a subclass of `Object`, so a class also
understands the methods of [Object](object.md). Its class method `new` makes a
new class.

The chapter [index](index.md) lists how the library classes are related and
what `new` and `defaultName` do. [The method search](methods.md) specifies
what `define`, `inherit` and the other methods that change a class do to the
order in which methods are looked up; this page specifies their arguments,
their checks and their errors. A class of the class library accepts none of
the methods that change it, as [the library rule](r:class.methods.library)
says. The examples that show an error use a small routine `try`, as the
builtin pages do.

## Class methods

r[class.class.new]
The class method `new` of `Class` makes a class. It is a class method of
`Class` and so of every metaclass, a subclass of `Class`, and it is not the
`new` that makes the instances of an ordinary class ([the Object
rule](r:class.object.new) specifies that one). Its first argument is the id of
the new class: a string, or an object whose `request` with the argument `STRING`
answers one,
and the null string and a number are allowed. The new class has `Object` as
its superclass, is not a mixin class and not abstract, and is an instance of
the class that received `new`, which is also its metaclass; its package is the
package of that class, so `.class~new` gives a class whose package is the one
of the library and a user's metaclass gives one whose package is the package
that declares the metaclass. `new` sends the new class the message `init` with
the arguments that follow the id, in order, and answers the class. The class
answers no `new` of its own: a message `new` to it raises `SYNTAX 97.1`, but a
subclass of it, made with `subclass`, has the `new` of every class. The id is
required (`SYNTAX 93.901`, none), `.nil` and an object with no string value
raise `88.909`, and the arguments after the id are those of `init`, so `Class`
itself, whose `init` takes none, raises `93.902` for one.

```rexx
f = .class~new('Foo')
say f~id f~class~id f~superclass~id f~queryMixinClass f~isAbstract
call try 'f~new'
g = f~subclass('Bar')
say g~new~class~id g~superclass~id
m = .meta~new('Qux', 1, 2)
say m~id m~class~id m~extra m~metaclass~id
m = .meta~new('Quux')
say m~extra '|' .class~new('')~id '|' .class~new(12)~id '|',
  .class~new(.mk~new)~id
say (.class~new('P')~package == .array~package),
  (.meta~new('P')~package == .context~package)
call try '.class~new'
call try '.class~new(.nil)'
call try '.class~new(.object~new)'
call try '.class~new("Foo", 1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class meta subclass class
::attribute extra
::method init
  expose extra
  extra = arg() arg(1) arg(2)
  self~init:super
::class mk
::method makestring
  return 'MadeId'
```

```output
Foo Class Object 0 0
f~new => 97.1
Bar Foo
Qux META 2 1 2 META
0   |  | 12 | MadeId
1 1
.class~new => 93.901
.class~new(.nil) => 88.909
.class~new(.object~new) => 88.909
.class~new("Foo", 1) => 93.902
```

## Methods

### Comparison and copying

r[class.class.eq]
The methods `=` and `==` of a class answer 1 when the argument is the receiver
itself and 0 otherwise, and take one argument, as [the Object
rule](r:class.object.eq) says of every object. A class is the same class only
as itself: a subclass or a class of the same name is another one.

```rexx
say (.a = .a) (.a == .a) (.a = .b) (.a == .asub) (.a = 'x')
say (.a~"="(.a)) (.a~"=="(.b)) (.a = .a~class) (.class = .class)
x = .a~subclass('A')
say (x = .a) (x == x) (x~id == .a~id)
call try '.a~"="'
call try '.a~"=="(1, 2)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class a
::class b
::class asub subclass a
```

```output
1 1 0 0 0
1 0 0 1
0 1 1
.a~"=" => 93.903
.a~"=="(1, 2) => 93.902
```

r[class.class.ne]
The methods `\=`, `\==`, `<>` and `><` of a class answer 1 when the argument
is not the receiver itself, and 0 when it is, as [the Object
rule](r:class.object.ne) says. A class that declares a class method of one of
these names answers that method's result when it is the receiver, and the
other methods are not changed by it.

```rexx
say (.a \= .b) (.a \== .a) (.a <> .asub) (.a >< .a) (.a \= 'x')
say (.over \= .over) (.over = .over) (.over <> .over)
call try '.a~"\="'
call try '.a~"<>"(1, 2)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class a
::class b
::class asub subclass a
::class over
::method "\=" class
  return 'ne override'
```

```output
1 0 1 0 1
ne override 1 0
.a~"\=" => 93.903
.a~"<>"(1, 2) => 93.902
```

r[class.class.copy]
A class object cannot be copied: `copy`, sent to a class by any route, raises
`SYNTAX 93.970`. That holds for the library classes, for a metaclass, and for
`Class` itself, and also when the message is sent with `send` or `sendWith`.
[Deviation 63](r:class.class.deviation.63) states what is different for a
scoped send.

```rexx
call try '.a~copy'
call try '.class~copy'
call try '.a~class~copy'
call try '.object~copy'
call try '.a~send("COPY")'
call try '.a~sendWith("copy", .array~new)'
call try '.meta~copy'
call try '.a~subclass("Z")~copy'
say .a~new~copy~class~id
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class a
::class meta subclass class
```

```output
.a~copy => 93.970
.class~copy => 93.970
.a~class~copy => 93.970
.object~copy => 93.970
.a~send("COPY") => 93.970
.a~sendWith("copy", .array~new) => 93.970
.meta~copy => 93.970
.a~subclass("Z")~copy => 93.970
A
```

### Names and questions about the class

r[class.class.id]
`id` answers the id of the class as a string: the name of the class in the
case that the `::CLASS` directive wrote it, which is upper case for a name
written as a symbol, or the string that `subclass`, `mixinClass` or `new` got.
It takes no argument, and raises `SYNTAX 93.902` with one. The id is not
changed by `objectName=`, and two classes may have one id.

```rexx
say .plain~id .mixed~id .quoted~id .class~id .object~id
x = .plain~subclass('Dyn One')
y = .plain~subclass('Dyn One')
say x~id y~id (x == y)
x~objectName = 'renamed'
say x~id x~objectName x
call try '.plain~id(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class "Mixed" mixinclass object
::class "quoted"
```

```output
PLAIN Mixed quoted Class Object
Dyn One Dyn One 0
Dyn One renamed renamed
.plain~id(1) => 93.902
```

r[class.class.defaultname]
`defaultName` answers `The`, a blank, the id of the class and `class`, and
takes no argument, as [the default name rule](r:class.index.defaultname) says.
The name is that of the id, not of any name set with `objectName=`.

```rexx
say .array~defaultName '|' .plain~defaultName '|' .quoted~defaultName
.plain~objectName = 'chosen'
say .plain~defaultName '|' .plain~objectName '|' .plain
say .plain~subclass('Some Id')~defaultName
call try '.plain~defaultName(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class "quoted"
```

```output
The Array class | The PLAIN class | The quoted class
The PLAIN class | chosen | chosen
The Some Id class
.plain~defaultName(1) => 93.902
```

r[class.class.package]
`package` answers the package that the class belongs to: for a class that a
`::CLASS` directive made, the package of the program that declares it, and for
a class of the class library the package that holds the library, whose name is
`REXX`. A class made by `subclass` or `mixinClass` belongs to no package, and
`package` answers `.nil`; a class made by the `new` of a metaclass has the
package of that metaclass ([the new rule](r:class.class.new)). The method
takes no argument.

```rexx
p = .plain~package
say p~class~id (p == .context~package) (.sub~package == p)
say .array~package~name .string~package~name .class~package~name
say (.array~package == p) (.array~package == .string~package)
say .plain~subclass('Dyn')~package~isNil,
  .plain~mixinClass('Mix')~package~isNil
call try '.plain~package(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class sub subclass plain
```

```output
Package 1 1
REXX REXX REXX
0 1
1 1
.plain~package(1) => 93.902
```

r[class.class.superclass]
`superClass` answers the first class of the list of superclasses: the class
that was named as the superclass when the class was made, and `.nil` for
`Object`, which has none. When `inherit` has put a mixin class in front of the
superclass in the list, `superClass` answers that mixin class ([the change
rule](r:class.methods.order.change)). The method takes no argument.

```rexx
say .sub~superclass~id .plain~superclass~id .object~superclass~isNil
say .mixed~superclass~id .class~superclass~id .array~superclass~id
.c~inherit(.m1, .plain)
say .c~superclass~id .c~superclasses~items
call try '.sub~superclass(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class sub subclass plain
::class mixed mixinclass plain
::class m1 mixinclass plain
::class c subclass plain
```

```output
PLAIN Object 1
PLAIN Object Object
M1 2
.sub~superclass(1) => 93.902
```

r[class.class.superclasses]
`superClasses` answers a new array, of one dimension, of the immediate
superclasses of the class: the superclass first and then the mixin classes in
the order in which the class inherits them, after the changes that `inherit`
and `uninherit` make to the list. The array is new on every call, so changing
it changes nothing in the class. `Object` has an array with no item. The
method takes no argument.

```rexx
a = .c~superclasses
say a~class~id a~dimension a~items
do x over a
  call charout , x~id' '
end
say
a~append(.string)
say a~items .c~superclasses~items (.c~superclasses == .c~superclasses)
say .object~superclasses~items .plain~superclasses~items
call try '.c~superclasses(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class m1 mixinclass plain
::class m2 mixinclass plain
::class c subclass plain inherit m2 m1
```

```output
Array 1 3
PLAIN M2 M1 
4 3 0
0 1
.c~superclasses(1) => 93.902
```

r[class.class.subclasses]
`subclasses` answers a new array, of one dimension, of the classes that have
the class as their superclass, mixin classes that were made from it included.
Classes that only inherit it are not in the array. The items are in the order
of the classes' making, the newest class first; classes that directives make
count as made in the order of the directives. The array is new on every call.
The method takes no argument.

```rexx
a = .plain~subclasses
say a~class~id a~dimension a~items
ids = ''
do x over a
  ids = ids x~id
end
ids = ids~strip
say ids~words ids~makearray(' ')~sort~makestring('l', ' ')
a~append(.string)
say .plain~subclasses~items .sub~subclasses~items .mixin~subclasses~items
n = .sub~subclass('Dyn')
say .sub~subclasses~items .sub~subclasses[1]~id
call try '.plain~subclasses(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class sub subclass plain
::class mixin mixinclass plain
::class inheritor subclass plain inherit mixin
```

```output
Array 1 3
3 INHERITOR MIXIN SUB
3 0 1
1 Dyn
.plain~subclasses(1) => 93.902
```

```rexx
s = ''
do c over .top~subclasses
  s = s c~id
end
say s~strip
do i = 1 to 3
  keep.i = .top~subclass('D'i)
end
s = ''
do c over .top~subclasses
  s = s c~id
end
say s~strip
::class top
::class b1 subclass top
::class a1 subclass top
::class c1 subclass top
```

```output defect=#136
C1 A1 B1
D3 D2 D1 C1 A1 B1
```

r[class.class.baseclass]
`baseClass` answers the class itself when it is not a mixin class, and, for a
mixin class, the first class on the way up its superclasses that is not a
mixin class: the class its MIXINCLASS option or `mixinClass` named, or the
base class of that one when it is a mixin class too. It takes no argument.

```rexx
say .plain~baseClass~id .sub~baseClass~id .object~baseClass~id
say .m1~baseClass~id .m2~baseClass~id .m3~baseClass~id .mo~baseClass~id
say .sub~mixinClass('Dyn')~baseClass~id .m2~subclass('Sub2')~baseClass~id
call try '.plain~baseClass(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class sub subclass plain
::class m1 mixinclass plain
::class m2 mixinclass m1
::class m3 mixinclass m2
::class mo mixinclass object
```

```output
PLAIN SUB Object
PLAIN PLAIN PLAIN Object
SUB Sub2
.plain~baseClass(1) => 93.902
```

r[class.class.querymixinclass]
`queryMixinClass` answers 1 for a class that was made a mixin class, by the
MIXINCLASS option or by `mixinClass`, and 0 for any other class. A subclass of
a mixin class made by `subclass` is not a mixin class. The method takes no
argument.

```rexx
say .plain~queryMixinClass .m1~queryMixinClass,
  .m1~subclass('S')~queryMixinClass
say .plain~mixinClass('Dyn')~queryMixinClass .object~queryMixinClass
call try '.plain~queryMixinClass(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class m1 mixinclass plain
```

```output
0 1 0
1 0
.plain~queryMixinClass(1) => 93.902
```

r[class.class.isabstract]
`isAbstract` answers 1 for a class that the ABSTRACT option made abstract and
0 for any other class. The property is not passed to a subclass, to a class
made by `subclass` or to a mixin class made by `mixinClass`, which are not
abstract unless the directive says so. The method takes no argument.

```rexx
say .abs~isAbstract .abssub~isAbstract .plain~isAbstract .object~isAbstract
say .abs~subclass('Dyn')~isAbstract .abs~mixinClass('Dyn2')~isAbstract
say .abs2~isAbstract .abs2~queryMixinClass
call try '.abs~isAbstract(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class abs abstract
::class abssub subclass abs
::class plain
::class abs2 abstract mixinclass object
```

```output
1 0 0 0
0 0
1 1
.abs~isAbstract(1) => 93.902
```

r[class.class.ismetaclass]
`isMetaclass` answers 1 for `Class` and for every class that has `Class` among
its superclasses, whichever way it was made, and 0 for any other class, so
also for a class that is only an instance of a metaclass. It takes no
argument.

```rexx
say .class~isMetaclass .meta~isMetaclass .meta2~isMetaclass,
  .object~isMetaclass
say .plain~isMetaclass .withmeta~isMetaclass .meta~new('Q')~isMetaclass
say .class~subclass('Dyn')~isMetaclass .class~mixinClass('Mix')~isMetaclass
call try '.plain~isMetaclass(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class meta subclass class
::class meta2 subclass meta
::class plain
::class withmeta metaclass meta
```

```output
1 1 1 0
0 0 0
1 1
.plain~isMetaclass(1) => 93.902
```

r[class.class.issubclassof]
`isSubclassOf` answers 1 when the class is the argument, or when the argument
is among its superclasses, direct or further up, or among the mixin classes it
or its ancestors inherit; otherwise it answers 0. The argument must be a
class: another object, `.nil` and a string included, raises `SYNTAX 88.914`;
none raises `88.901`, and a second argument `93.902`. Only the instance
hierarchy is asked: that `.plain` is an instance of `Class` does not make it a
subclass of `Class`.

```rexx
say .sub~isSubclassOf(.sub) .sub~isSubclassOf(.plain),
  .sub~isSubclassOf(.object)
say .plain~isSubclassOf(.sub) .sub~isSubclassOf(.m2) .sub~isSubclassOf(.m1)
say .m2~isSubclassOf(.m1) .m1~isSubclassOf(.m2) .plain~isSubclassOf(.class)
say .plain~class~isSubclassOf(.object) .class~isSubclassOf(.object)
say .object~isSubclassOf(.object) .meta~isSubclassOf(.class),
  .m2~isSubclassOf(.plain)
call try '.plain~isSubclassOf'
call try '.plain~isSubclassOf(.nil)'
call try '.plain~isSubclassOf("PLAIN")'
call try '.plain~isSubclassOf(.plain~new)'
call try '.plain~isSubclassOf(.plain, 1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class m1 mixinclass plain
::class m2 mixinclass m1
::class sub subclass plain inherit m2
::class meta subclass class
```

```output
1 1 1
0 1 1
1 0 0
1 1
1 1 1
.plain~isSubclassOf => 88.901
.plain~isSubclassOf(.nil) => 88.914
.plain~isSubclassOf("PLAIN") => 88.914
.plain~isSubclassOf(.plain~new) => 88.914
.plain~isSubclassOf(.plain, 1) => 93.902
```

r[class.class.metaclass]
`metaClass` answers the metaclass of the class: the class that its `METACLASS`
option or the metaclass argument of `subclass` or `mixinClass` named, or, for
a class that the class method `new` of a metaclass made, that metaclass;
otherwise the metaclass of its superclass, otherwise `Class`. It is the class
that `subclass` and `mixinClass` use for a new class when they get no
metaclass, and the instance methods of that metaclass are the class methods
the class has ([the class-side rule](r:class.methods.class-side)). The method
takes no argument.

```rexx
say .plain~metaclass~id .withmeta~metaclass~id .withsub~metaclass~id
say .class~metaclass~id .object~metaclass~id .meta~metaclass~id
x = .withmeta~subclass('Dyn')
say x~metaclass~id x~class~id
y = .plain~subclass('Dyn2', .meta)
say y~metaclass~id y~subclass('Dyn3')~metaclass~id,
  y~mixinclass('Dyn4')~metaclass~id
say .meta~new('Q')~metaclass~id
call try '.plain~metaclass(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class meta subclass class
::class plain
::class withmeta metaclass meta
::class withsub subclass withmeta
```

```output
Class META META
Class Class Class
META META
META META META
META
.plain~metaclass(1) => 93.902
```

### Methods and annotations of the class

r[class.class.method]
`method` answers the method object that the class's own table holds for the
name, taken in upper case. The tables of the superclasses and mixin classes
are not searched: a name that only a class behind holds raises `SYNTAX 97.1`,
as does a name that the table does not hold. A name that `define` hid by its
one argument form answers `.nil`; a name that `define` hid with `.nil` as the
method has no entry that `method` could answer and raises `97.1`. The answer
is the object that the class holds, so the same method answers every time, and
it is the object that [`instanceMethod`](r:class.object.instancemethod) of an
instance answers. The name is a string or an object whose `request` with the
argument `STRING` answers one: none raises `88.901`, and `.nil` raises `88.909`,
and a second
argument raises `93.902`.

```rexx
m = .plain~method('foo')
say m~class~id m~source[1]~strip (m == .plain~method('FOO'))
say (m == .plain~new~instanceMethod('foo')) (m~scope == .plain)
say .plain~method('priv')~isPrivate .plain~method('att')~class~id
call try '.sub~method("foo")'
call try '.plain~method("nosuchzz")'
call try '.plain~method("COPY")'
call try '.plain~method(.mk~new)~class~id'
.plain~define('HIDDEN')
call try '.plain~method("hidden")~isNil'
.plain~define('GONE', .nil)
call try '.plain~method("gone")'
call try '.plain~method'
call try '.plain~method(.nil)'
call try '.plain~method("foo", 1)'
call try '.plain~class~method("define")~class~id'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::method foo
  return 'foo'
::method hidden
  return 1
::method gone
  return 1
::method priv private
  return 1
::attribute att
::class sub subclass plain
::class mk
::method makestring
  return 'FOO'
```

```output
Method return 'foo' 1
1 1
1 Method
.sub~method("foo") => 97.1
.plain~method("nosuchzz") => 97.1
.plain~method("COPY") => 97.1
.plain~method(.mk~new)~class~id => Method
.plain~method("hidden")~isNil => 1
.plain~method("gone") => 97.1
.plain~method => 88.901
.plain~method(.nil) => 88.909
.plain~method("foo", 1) => 93.902
.plain~class~method("define")~class~id => Method
```

r[class.class.methods]
`methods` answers a supplier whose indexes are method names in upper case and
whose items are method objects, for the instance methods of the class. The
supplier lists the names that exist when it is made, and a later change does
not reach it. The order of its entries is not specified
([Deviation 33](../deviations.md)). Without an argument it lists the table of
the class and the tables of every class behind it in its order, so a name that
more than one of them holds is listed once for each. With a class that is in
the order of the receiver, the class itself included, it lists the methods of
that class's table, and with `.nil` the methods of the receiver's own table
only. With any other argument, a number, a string or a class that is not in
the order, the supplier has no entries. A table entry that hides a name is not
listed; the name is still listed from every other table of the order that
holds a method for it, so the listing does not show what the search finds. A
second argument raises `SYNTAX 93.902`.

```rexx
call show .sub~methods(.nil), 'sub own'
call show .sub~methods(.sub), 'sub(.sub)'
call show .sub~methods(.plain), 'sub(.plain)'
call show .sub~methods(.mix), 'sub(.mix)'
call show .plain~methods(.sub), 'plain(.sub)'
call show .sub~methods(.string), 'sub(.string)'
call show .sub~methods(1), 'sub(1)'
t = .sub~methods(.nil)
.sub~define('LATE', 'return 1')
call show t, 'snapshot'
call show .sub~methods(.nil), 'after'
n = 0
t = .sub~methods
do while t~available
  if t~index == 'HELLO' then n = n + 1
  t~next
end
say 'HELLO listed' n 'times'
.sub~define('PLAINONLY')
call show .sub~methods(.nil), 'hidden'
n = 0
t = .sub~methods
do while t~available
  if t~index == 'PLAINONLY' then n = n + 1
  t~next
end
say 'PLAINONLY listed' n 'times' .sub~new~hasMethod('plainonly')
call try '.sub~methods(.sub, 1)'
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
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::method hello
  return 'plain hello'
::method plainonly
  return 1
::class mix mixinclass object
::method mixed
  return 1
::class sub subclass plain inherit mix
::method hello
  return 'sub hello'
::method own
  return 1
```

```output
sub own: 2 HELLO=Method OWN=Method
sub(.sub): 2 HELLO=Method OWN=Method
sub(.plain): 2 HELLO=Method PLAINONLY=Method
sub(.mix): 1 MIXED=Method
plain(.sub): 0 
sub(.string): 0 
sub(1): 0 
snapshot: 2 HELLO=Method OWN=Method
after: 3 HELLO=Method LATE=Method OWN=Method
HELLO listed 2 times
hidden: 3 HELLO=Method LATE=Method OWN=Method
PLAINONLY listed 1 times 0
.sub~methods(.sub, 1) => 93.902
```

r[class.class.annotation]
`annotation` answers the value of the annotation of the class with the given
name, as a string, as [the `::ANNOTATE` rules](r:dir.annotate.value) keep it,
and `.nil` when the class has no annotation of that name. The name is compared
in upper case. The annotations of the superclasses are not inherited. The
argument is a string or an object whose `request` with the argument `STRING`
answers one: none
raises `SYNTAX 88.901`, `.nil` raises `88.909`, and a second argument raises
`93.902`.

```rexx
say .a~annotation('author') '|' .a~annotation('AUTHOR') '|',
  .a~annotation('Ver')
say .a~annotation('none')~isNil .asub~annotation('author')~isNil
say .a~annotation(.mk~new) '|' .a~annotation(12)~isNil
say .a~annotation('author')~class~id
call try '.a~annotation'
call try '.a~annotation(.nil)'
call try '.a~annotation(.object~new)'
call try '.a~annotation("a", "b")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class a
::annotate class a author "Jo" ver 1.0 other +3
::class asub subclass a
::class mk
::method makestring
  return 'AUTHOR'
```

```output
Jo | Jo | 1.0
1 1
Jo | 1
String
.a~annotation => 88.901
.a~annotation(.nil) => 88.909
.a~annotation(.object~new) => 88.909
.a~annotation("a", "b") => 93.902
```

r[class.class.annotations]
`annotations` answers a `StringTable` whose indexes are the annotation names
in upper case and whose items are their values. It is the class's own table,
the same object on every call, so a change to it is a change to the
annotations of the class that `annotation` shows. A class with no annotations
answers an empty table. The method takes no argument.

```rexx
t = .a~annotations
say t~class~id t~items (t == .a~annotations)
say t['AUTHOR'] t['VER'] t['OTHER']
t['NEW'] = 'x'
t['AUTHOR'] = 'changed'
say .a~annotations~items .a~annotation('new') .a~annotation('author')
say .asub~annotations~class~id .asub~annotations~items
call try '.a~annotations(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class a
::annotate class a author "Jo" ver 1.0 other +3
::class asub subclass a
```

```output
StringTable 3 1
Jo 1.0 +3
4 x changed
StringTable 0
.a~annotations(1) => 93.902
```

r[class.class.activate]
`activate` is sent by the interpreter to each class that the classes of a
program declare, after all of them exist and before the program's main code
runs, once for each class and in the order in which the classes were made: a
class comes after its superclass and its inherited mixin classes, and
otherwise in the order of the directives. A class made at run time by
`subclass`, `mixinClass` or `new` is not activated. A program declares
`activate` as a class method to set a class up; the method that `Class`
supplies does nothing and answers no value. Sent by hand, it takes no argument
(`SYNTAX 93.902` with one).

```rexx
say 'main'
.plain~activate
x = .first~subclass('Dyn')
say 'dyn made'
call try '.plain~activate(1)'
call try 'r = .plain~activate'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class first subclass second
::method activate class
  say 'activate' self~id 'sees' .third~id
::class plain
::class second
::method activate class
  say 'activate' self~id
::class third inherit mix
::method activate class
  say 'activate' self~id
::class mix mixinclass object
::method activate class
  say 'activate' self~id
```

```output
activate SECOND
activate FIRST sees THIRD
activate MIX
activate THIRD
main
dyn made
.plain~activate(1) => 93.902
r = .plain~activate => 91.999
```

### Changing the class

r[class.class.define]
`define` gives the class an instance method, or hides a name, for every
instance of the class, those that exist already included, as
[the rule on changes](r:class.methods.late) and
[the hidden-name rule](r:class.methods.hidden) say, and the search finds it as
[the search rule](r:class.methods.search) says, and answers no value. The
first argument is the name, taken in upper case, and may be any string, a
number included. The second is the method, in the forms that `setMethod`
takes: a method object, a string that is the source of a method, or an array
of strings, one for each line, converted as
[the `setMethod` rule](r:class.object.setmethod) says and with the messages of
[the sends rule](r:class.class.define.sends). The method that the class holds
runs the code of the method object that is given, with the class as its scope.
It is not specified whether the method object that the class holds is the
argument itself or a copy of it, and what scope the argument object answers
afterwards is not specified either. A method replaces what the table held for
the name. With no second argument the name is hidden; with `.nil` as the
second argument the name is hidden from the search as well, but the table no
longer holds an entry for it that `method` could answer. An object whose
`request` with `STRING` answers `.nil`, as an instance of a subclass of
`Method` that declares neither `MAKEARRAY` nor `MAKESTRING` does, raises
`SYNTAX 93.974`, and a source that does not translate raises its translation
error from `define`. A name is required (`88.901`), a name that is no string
and has no string value raises `88.909`, and a third argument raises `93.902`.
The number of arguments is checked first, then the class, then the name, then
the method. The classes of the class library accept no `define`
([the library rule](r:class.methods.library)), which raises `98.985` before
any argument is checked. The examples use objects made after the change.

```rexx
.k~define('A1', 'return 1')
say .k~new~a1
.k~define('a1', 'return 2')
say .k~new~a1
.k~define('A2', .method~new('', 'return 3'))
say .k~new~a2
.k~define('A3', .array~of('say "in a3"', 'return 33'))
say .k~new~a3
.k~define('12', 'return "number name"')
say .k~new~'12'
.k~define('foo')
say .k~new~hasMethod('foo') .k~method('foo')~isNil
.k~define('foo', 'return "back"')
say .k~new~foo
.k~define('bar', .nil)
say .k~new~hasMethod('bar')
call try 'x = .k~method("bar")'
call try '.k~define("x", 1, 2)'
call try '.k~define("x", .object~new)'
call try '.k~define(.nil, .object~new)'
call try '.k~define("x", "if then")'
call try '.k~define'
call try '.k~define(.nil)'
call try '.k~define(.nil, "return 1")'
call try 'r = .k~define("y", "return 1")'
call try '.string~define("y", "return 1")'
call try '.string~define("y", 1, 2)'
call try '.string~define(.nil)'
call try '.string~define'
call try '.k~define("z", .lmeth~new("", "return 1"))'
m = .method~new('', 'return "mobj"')
.k~define('M2', m)
.k~define('M3', m)
say .k~new~m2 .k~new~m3 (.k~method('M2')~scope == .k),
  (.k~method('M3')~scope == .k)
n = .method~new('', 'return "n"')
x = .k~new~go(n)
.k~define('M4', n)
say .k~new~m4 (.k~method('M4')~scope == .k)
c = .method~new('', 'return 5')~copy
.k~define('M5', c)
say .k~new~m5 (.k~method('M5')~scope == .k)
o = .k~new
o~sm('IM', .method~new('', 'return 6'))
im = o~instanceMethod('IM')
.k~define('M6', im)
say .k~new~m6 (.k~method('M6')~scope == .k)
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method foo
  return 'foo'
::method bar
  return 'bar'
::method go
  return self~run(arg(1))
::method sm
  self~setMethod(arg(1), arg(2))
::class lmeth subclass method
```

```output
1
2
3
in a3
33
number name
0 1
back
0
x = .k~method("bar") => 97.1
.k~define("x", 1, 2) => 93.902
.k~define("x", .object~new) => 93.974
.k~define(.nil, .object~new) => 88.909
.k~define("x", "if then") => 35.929
.k~define => 88.901
.k~define(.nil) => 88.909
.k~define(.nil, "return 1") => 88.909
r = .k~define("y", "return 1") => 91.999
.string~define("y", "return 1") => 98.985
.string~define("y", 1, 2) => 93.902
.string~define(.nil) => 98.985
.string~define => 98.985
.k~define("z", .lmeth~new("", "return 1")) => 93.974
mobj mobj 1 1
n 1
5 1
6 1
```

r[class.class.define.sends]
`define` sends the message `request` with the argument `ARRAY`, once, to a
second argument that is not `.nil` and not an instance of `Method`, `String`
or `Array` itself, and uses the array it answers. When that answers no array,
it sends `request` with the argument `STRING`, once, and uses a string answer
as the source; one whose answer is `.nil` raises `SYNTAX 93.974`. It sends
nothing to the class.

```rexx
.k~define('M', .logged~new)
say .k~new~m
.k~define('N', .src~new)
say .k~new~n
::class k
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

r[class.class.definemethods]
`defineMethods` gives the class every method of a collection, as `define`
gives one. The argument is a collection whose indexes are the method names and
whose items are method objects, strings or arrays of strings, as `define`
takes them, or `.nil` to hide the name, converted as
[`define`](r:class.class.define) converts its method, with the same errors for
an item that is no method (`SYNTAX 93.974`). A method object item is installed
as [`define`](r:class.class.define) installs a method object: the class holds
a method that runs the code of the item with the class as its scope, and it is
not specified whether that is the item itself or a copy, or what scope the
item answers afterwards. Every item is made into a method before any is
defined, so an error leaves the class unchanged. The names are taken in upper
case. A name that the class holds is replaced. The method sends the argument
the message `supplier`, with no argument, once, and takes the pairs of what
the supplier answers one after the other, so any object that answers a
supplier will do, and the indexes of an array are the names `1`, `2` and so
on. An object that has no `supplier` method raises `SYNTAX 97.1`, as `.nil`
does. The object that `supplier` answers need not be a `Supplier`
([Deviation 108](r:class.class.deviation.108)). The argument is required
(`88.901`) and a second raises `93.902`. The number of arguments is checked
first, then the class, then the argument. The method answers no value, and the
classes of the class library accept it as little as `define`: they raise
`98.985` before the argument is checked.

```rexx
t = .directory~new
t['A1'] = 'return "a1"'
t['a2'] = .method~new('', 'return "a2"')
t['A3'] = .array~of('x = 3', 'return "a3"')
t['A4'] = .nil
t['foo'] = 'return "new foo"'
.k~defineMethods(t)
o = .k~new
say o~a1 o~a2 o~a3 o~foo o~hasMethod('a4')
.k~defineMethods(.array~of('return "one"', 'return "two"'))
say .k~new~'1' .k~new~'2'
.k~defineMethods(.stringtable~new~~put('return "from stringtable"', 'ST'))
say .k~new~st
.k~defineMethods(.sp~new)
say .k~new~spa
call try '.k~defineMethods'
call try '.k~defineMethods(.nil)'
call try '.k~defineMethods(1)'
call try '.k~defineMethods(.object~new)'
call try '.k~defineMethods(.directory~new, 1)'
u = .directory~new
u['U1'] = .object~new
call try '.k~defineMethods(u)'
call try '.object~defineMethods(.directory~new)'
call try '.object~defineMethods'
call try '.object~defineMethods(.nil, 2)'
x = .array~new(4)~~put("return 3", 3)
call try '.k~defineMethods(x~~put(.object~new, 4))'
say .k~new~hasMethod('3')
call try '.k~defineMethods(x~~put("if then", 4))'
say .k~new~hasMethod('3')
n = .method~new('', 'return "n"')
.k~defineMethods(.directory~new~~put(n, 'N'))
say .k~new~n (.k~method('N')~scope == .k)
h = .j~method('FOO')
.k~defineMethods(.directory~new~~put(h, 'H'))
say .k~new~h (.k~method('H')~scope == .k)
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method foo
  return 'foo'
::class j
::method foo
  return 'from j'
::class sp
::method supplier
  say 'supplier sent' arg()
  return .directory~new~~put('return "spa"', 'SPA')~supplier
```

```output
a1 a2 a3 new foo 0
one two
from stringtable
supplier sent 0
spa
.k~defineMethods => 88.901
.k~defineMethods(.nil) => 97.1
.k~defineMethods(1) => 97.1
.k~defineMethods(.object~new) => 97.1
.k~defineMethods(.directory~new, 1) => 93.902
.k~defineMethods(u) => 93.974
.object~defineMethods(.directory~new) => 98.985
.object~defineMethods => 98.985
.object~defineMethods(.nil, 2) => 93.902
.k~defineMethods(x~~put(.object~new, 4)) => 93.974
0
.k~defineMethods(x~~put("if then", 4)) => 35.929
0
n 1
from j 1
```

r[class.class.definemethods.sends]
`defineMethods` sends its argument the message `supplier`, with no argument,
once, and nothing else to the argument; it sends nothing to the class. The
indexes and items it takes are those that the supplier answers in turn. A
`Supplier` is read without sending it messages: overrides of `available`,
`index`, `item` and `next` do not run. Any other object that `supplier`
answers is used through those four messages, as
[Deviation 108](r:class.class.deviation.108) states.

```rexx
.k~defineMethods(.sp~new)
say .k~new~one .k~new~two
::class k
::class sp
::method supplier
  say 'supplier sent' arg()
  src = .array~of('return 1', 'return 2')
  return .lsup~new(src, .array~of('ONE', 'TWO'))
::method allIndexes
  say 'allIndexes sent'
::method allItems
  say 'allItems sent'
::method items
  say 'items sent'
::class lsup subclass supplier
::method available
  say 'available sent'
  forward class (super)
::method index
  say 'index sent'
  forward class (super)
::method item
  say 'item sent'
  forward class (super)
::method next
  say 'next sent'
  forward class (super)
```

```output
supplier sent 0
1 2
```

r[class.class.delete]
`delete` removes the entry that the class's own table holds for a name, taken in
upper case, whether the entry is a method or a mark that hides the name, and
answers no value. A name that `define` hid with `.nil` as the method is hidden
by such a mark too, and `delete` ends the hiding at once ([Deviation
117](r:class.methods.deviation.117)). The name is then looked up in the classes
behind, as [the hidden-name rule](r:class.methods.hidden) says. A name that the
table does not hold is not an error and changes nothing, and `delete` does not
remove a method that only a class behind holds. The name is required
(`SYNTAX 88.901`), `.nil` raises `88.909`, and a second argument raises
`93.902`. The number of arguments is checked first, then the class, then the
name. The classes of the class library accept no `delete` ([the library
rule](r:class.methods.library)); they raise `98.985` before the name is checked.

```rexx
.sub~define('M2', 'return "sub m2"')
.sub~define('M1')
say .sub~new~m2 .sub~new~hasMethod('m1')
.sub~delete('m2')
.sub~delete('M1')
say .sub~new~m2 .sub~new~m1
.sub~delete('m1')
.sub~delete('nosuchzz')
.sub~delete('')
.sub~delete('base only')
call try '.sub~delete("base")'
say .sub~new~base .base~new~base
.base~delete('BASE')
say .sub~new~hasMethod('base')
call try 'r = .sub~delete("m2")'
call try '.sub~delete'
call try '.sub~delete(.nil)'
call try '.sub~delete("m2", 1)'
call try '.array~delete("items")'
call try '.array~delete("items", 1)'
call try '.array~delete'
call try '.sub~delete(, 1)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class base
::method base
  return 'base'
::method m1
  return 'base m1'
::method m2
  return 'base m2'
::class sub subclass base
```

```output
sub m2 0
base m2 base m1
.sub~delete("base") => no error
base base
0
r = .sub~delete("m2") => 91.999
.sub~delete => 88.901
.sub~delete(.nil) => 88.909
.sub~delete("m2", 1) => 93.902
.array~delete("items") => 98.985
.array~delete("items", 1) => 93.902
.array~delete => 98.985
.sub~delete(, 1) => 93.902
```

r[class.class.inherit]
`inherit` makes the class inherit a mixin class, as the INHERIT option of the
directive does, and answers no value. The first argument is the mixin class.
The second, if given, is a class in the class's list of superclasses, and the
mixin class is put immediately in front of it; without it the mixin class is
put at the end of the list. How that changes the order of the class, and so
the search, is specified by [the change rule](r:class.methods.order.change).
An argument that is no mixin class, `.nil` and a string among them, raises
`SYNTAX 98.942`, and so does a class that is not a mixin class. A mixin class
that is already inherited, or that is an ancestor of the class or has the
class among its ancestors, raises `98.944`. A mixin class whose base class is
not the superclass of the class or one of its ancestors raises `98.943`. A
second argument that is not in the list of superclasses, `.nil` included,
raises `98.945`. The first argument is required (`88.901`) and a third raises
`93.902`. The checks run in this order: the number of arguments, the class,
then `98.942`, `98.944`, `98.943` and `98.945`. A class of the class library
accepts no `inherit` ([the library rule](r:class.methods.library)); it raises
`98.985` before the arguments are checked.

```rexx
call show 'start'
call try '.c~inherit(.m1)'
call show 'm1'
call try '.c~inherit(.m1)'
call try '.c~inherit(.m2, .plain)'
call show 'm2 before plain'
call try '.c~inherit(.m3, .m2)'
call show 'm3 before m2'
call try '.c~inherit(.m4, .nil)'
call try '.c~inherit(.m4, .m5)'
call try '.c~inherit(.m4, 1)'
call try '.c~inherit(.m5)'
call try '.c~inherit(.m6)'
call try '.c~inherit(.c)'
call try '.c~inherit(.plain)'
call try '.c~inherit(.nil)'
call try '.c~inherit(1)'
call try '.c~inherit(.object~new)'
call try '.c~inherit'
call try '.c~inherit(.m4, .plain, 1)'
call try '.c~inherit(.m1, .nil)'
call try '.c~inherit(.mstr, .nil)'
call try '.c~inherit(.m6, .mstr)'
call try '.c~inherit(.nil, .nil)'
call try '.string~inherit'
call try '.string~inherit(.nil, .nil, 3)'
call try '.c2~inherit(.m5)'
call try '.c2~inherit(.mstr)'
call try '.string~inherit(.m1)'
exit
show:
  s = ''
  do x over .c~superclasses
    s = s x~id
  end
  say arg(1)':' s~strip
  return
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class c subclass plain
::class c2 subclass plain
::class m1 mixinclass plain
::class m2 mixinclass plain
::class m3 mixinclass plain
::class m4 mixinclass plain
::class m5 mixinclass object
::class m6 mixinclass c
::class mstr mixinclass string
```

```output
start: PLAIN
.c~inherit(.m1) => no error
m1: PLAIN M1
.c~inherit(.m1) => 98.944
.c~inherit(.m2, .plain) => no error
m2 before plain: M2 PLAIN M1
.c~inherit(.m3, .m2) => no error
m3 before m2: M3 M2 PLAIN M1
.c~inherit(.m4, .nil) => 98.945
.c~inherit(.m4, .m5) => 98.945
.c~inherit(.m4, 1) => 98.945
.c~inherit(.m5) => no error
.c~inherit(.m6) => 98.944
.c~inherit(.c) => 98.942
.c~inherit(.plain) => 98.942
.c~inherit(.nil) => 98.942
.c~inherit(1) => 98.942
.c~inherit(.object~new) => 98.942
.c~inherit => 88.901
.c~inherit(.m4, .plain, 1) => 93.902
.c~inherit(.m1, .nil) => 98.944
.c~inherit(.mstr, .nil) => 98.943
.c~inherit(.m6, .mstr) => 98.944
.c~inherit(.nil, .nil) => 98.942
.string~inherit => 98.985
.string~inherit(.nil, .nil, 3) => 93.902
.c2~inherit(.m5) => no error
.c2~inherit(.mstr) => 98.943
.string~inherit(.m1) => 98.985
```

r[class.class.uninherit]
`uninherit` takes a mixin class out of the class's list of superclasses, and
answers no value. How that changes the order of the class is specified by [the
change rule](r:class.methods.order.change). An argument that is no mixin class
raises `SYNTAX 98.942`, and a mixin class that the list does not hold, or that
stands first in it, so that it is the superclass, raises `98.945`. The
argument is required (`88.901`) and a second raises `93.902`. The number of
arguments is checked first, then the class, then the argument. A class of the
class library accepts no `uninherit` ([the library
rule](r:class.methods.library)); it raises `98.985` before the argument is
checked.

```rexx
.c~inherit(.m1)
.c~inherit(.m2)
call show 'start'
call try '.c~uninherit(.m1)'
call show 'without m1'
call try '.c~uninherit(.m1)'
call try '.c~uninherit(.m3)'
call try '.c~uninherit(.plain)'
call try '.c~uninherit(.nil)'
call try '.c~uninherit(1)'
call try '.c~uninherit(.object)'
call try '.c~uninherit'
call try '.c~uninherit(.m2, 1)'
call try '.c~uninherit(.m2)'
call show 'end'
call try '.c~inherit(.m3, .plain)'
call show 'm3 first'
call try '.c~uninherit(.m3)'
call try '.string~uninherit(.m1)'
call try '.string~uninherit(.nil, 2)'
call try '.string~uninherit'
exit
show:
  s = ''
  do x over .c~superclasses
    s = s x~id
  end
  say arg(1)':' s~strip
  return
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class plain
::class c subclass plain
::class m1 mixinclass plain
::class m2 mixinclass plain
::class m3 mixinclass plain
```

```output
start: PLAIN M1 M2
.c~uninherit(.m1) => no error
without m1: PLAIN M2
.c~uninherit(.m1) => 98.945
.c~uninherit(.m3) => 98.945
.c~uninherit(.plain) => 98.942
.c~uninherit(.nil) => 98.942
.c~uninherit(1) => 98.942
.c~uninherit(.object) => 98.942
.c~uninherit => 88.901
.c~uninherit(.m2, 1) => 93.902
.c~uninherit(.m2) => no error
end: PLAIN
.c~inherit(.m3, .plain) => no error
m3 first: M3 PLAIN
.c~uninherit(.m3) => 98.945
.string~uninherit(.m1) => 98.985
.string~uninherit(.nil, 2) => 93.902
.string~uninherit => 98.985
```

### Making classes and objects

r[class.class.subclass]
`subclass` makes a new class that has the receiver as its superclass and
answers it. The first argument is the id of the class, which `id` answers, a
string or an object whose `request` with the argument `STRING` answers one: none
raises
`SYNTAX 88.901`, and `.nil` and an object with no string value raise `88.909`.
The null string and a number are allowed, the id need not be unique, and the
class is not entered in `.environment`. The second argument is the metaclass,
a class that is `Class` or a subclass of it; the new class is an instance of
it. When the argument is omitted, the metaclass is the one of the receiver, as
[`metaClass`](r:class.class.metaclass) says. Any other value, `.nil` and a
class that is no metaclass included, raises `SYNTAX 99.927`. The third
argument is a collection of class methods, given as `defineMethods` takes its
argument, and the new class gets them as class methods of its own, which its
subclasses inherit as they inherit any class method. The new class is not a
mixin class and not abstract, whatever the receiver is, and it has no package.
A fourth argument raises `93.902`. The arguments are checked in the order:
their number (`93.902`), the metaclass, the id, then the collection. When an
error is raised, the receiver has no new subclass. The method does not check
whether the receiver may have subclasses, so a subclass of a library class is
made like any other.

```rexx
s = .k~subclass('Sub One')
say s~id s~class~id s~superclass~id s~queryMixinClass s~isAbstract
say s~baseClass~id s~metaclass~id (s == .k) s~new~class~id s~new~foo
say .k~subclass('Sub One')~id (.k~subclasses~items)
say .environment~at('SUB ONE')~isNil
mm = .k~subclass('WithMeta', .meta)
say mm~id mm~class~id mm~metaclass~id mm~foo mm~new~foo
t = .directory~new~~put('return "class method of" self~id', 'CM')
cc = .k~subclass('WithCM', , t)
say cc~cm cc~class~id cc~subclass('Deeper')~cm
say .abs~subclass('FromAbs')~isAbstract .abs~subclass('FromAbs')~new~class~id
say .string~subclass('MyString')~new('abc')~class~id
say .k~subclass(12)~id .k~subclass('')~id .k~subclass(.mk~new)~id
call try '.k~subclass'
call try '.k~subclass(.nil)'
call try '.k~subclass(.nil, 1)'
call try '.k~subclass(.object~new)'
call try '.k~subclass("X", .nil)'
call try '.k~subclass("X", 1)'
call try '.k~subclass("X", .k)'
call try '.k~subclass("X", .object~new)'
call try '.k~subclass("X", .class, 1)'
call try '.k~subclass("X", .class, .nil)'
call try '.k~subclass("X", .class, .directory~new, 1)'
call try '.k~cm'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r~id
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method foo
  return 'foo'
::class meta subclass class
::method foo
  return 'meta foo'
::class abs abstract
::class mk
::method makestring
  return 'FromMk'
```

```output
Sub One Class K 0 0
Sub One Class 0 Sub One foo
Sub One 2
1
WithMeta META META meta foo foo
class method of WithCM Class class method of Deeper
0 FromAbs
MyString
12  FromMk
.k~subclass => 88.901
.k~subclass(.nil) => 88.909
.k~subclass(.nil, 1) => 99.927
.k~subclass(.object~new) => 88.909
.k~subclass("X", .nil) => 99.927
.k~subclass("X", 1) => 99.927
.k~subclass("X", .k) => 99.927
.k~subclass("X", .object~new) => 99.927
.k~subclass("X", .class, 1) => 97.1
.k~subclass("X", .class, .nil) => 97.1
.k~subclass("X", .class, .directory~new, 1) => 93.902
.k~cm => 97.1
```

r[class.class.subclass.failed]
A `subclass` that raises an error, whichever check raised it, leaves the
receiver with no new subclass. In the example a class method of the third
argument is no method.

```rexx
signal on syntax name d
x = .k~subclass('X', , .array~of('return 1', .object~new))
d: say condition('o')~code .k~subclasses~items
::class k
```

```output defect=#134
93.974 0
```

r[class.class.subclass.sends]
`subclass` sends the message `new` to the metaclass, which is the receiver's
metaclass when there is no metaclass argument, with the id as its only argument,
and the class that `new` builds for it is the new class: a class method `new` of
the metaclass runs ([the new rule](r:class.class.new) specifies the one that
`Class` supplies). The class that `new` answers receives `init`, from that
`new`, while it is still a plain class: its superclass is `Object` and it has
only the methods of its metaclass. `subclass` then sends the third argument
`supplier`, once, with no argument, as
[`defineMethods`](r:class.class.definemethods) does. It then makes the receiver
the superclass, adds the class methods, sets the package to `.nil`, and sends
the class `init` a second time, with no argument. A class method `init` that the
receiver declares is therefore run only by the second `init`. `activate` is not
sent. A `new` of the metaclass that answers a class that `subclass`,
`mixinclass` or a `::CLASS` directive has already completed, or a class of the
library, is treated as [Deviation 32](r:class.class.deviation.32) states. A
`new` of the metaclass that answers an object that is not a class raises
`SYNTAX 98.900` from `subclass` or `mixinclass`, in the program that sent it,
where it can be trapped ([Deviation 114](r:class.class.deviation.114)).

```rexx
x = .k~subclass('S1')
say 'plain done'
y = .k~subclass('S2', .meta)
say 'meta done'
z = .k~subclass('S3', , .sp~new)
say z~cm
::class k
::class meta subclass class
::method new class
  say 'new sent to' self~id 'with' arg() 'argument' arg(1)
  forward class (super)
::method init
  say 'init sent to' self~id 'with' arg()
  forward class (super) continue
::method activate
  say 'activate sent'
::class sp
::method supplier
  say 'supplier sent' arg()
  return .directory~new~~put('return "cm ran"', 'CM')~supplier
```

```output
plain done
new sent to META with 1 argument S2
init sent to S2 with 0
init sent to S2 with 0
meta done
supplier sent 0
cm ran
```

r[class.class.subclass.sends.state]
The new class is a plain class at its first `init`: its superclass is
`Object`, it has no class methods from the third argument and none from the
receiver, and its package is that of the metaclass. At the second `init` its
superclass is the receiver, the class methods are in place and the package is
`.nil`.

```rexx
y = .k~subclass('S2', .meta, .sp~new)
z = .k~subclass('S4', .meta)
::class k
::method kcm class
  return 'kcm'
::class meta subclass class
::method init
  say 'meta init' self~id 'super' self~superclass~id,
    'cm' self~hasMethod('CM') 'kcm' self~hasMethod('KCM'),
    'pkg' self~package~isNil
  forward class (super) continue
::class sp
::method supplier
  say 'supplier sent'
  return .directory~new~~put('return 1', 'CM')~supplier
```

```output
meta init S2 super Object cm 0 kcm 0 pkg 0
supplier sent
meta init S2 super K cm 1 kcm 1 pkg 1
meta init S4 super Object cm 0 kcm 0 pkg 0
meta init S4 super K cm 0 kcm 1 pkg 1
```

r[class.class.subclass.sends.classinit]
A class method `init` that the receiver declares runs once for the new class,
from the second `init`.

```rexx
x = .k~subclass('S1')
::class k
::method init class
  say 'k class init' self~id self~superclass~id
  forward class (super) continue
```

```output
k class init K Object
k class init S1 K
```

r[class.class.mixinclass]
`mixinClass` makes a new mixin class that has the receiver as its base class
and answers it. Its arguments, their order, their checks and their errors are
those of [`subclass`](r:class.class.subclass), and so are the messages it
sends, in the same order ([the sends rule](r:class.class.subclass.sends)). The
new class becomes a mixin class, with its base class, only after the second
`init`: at both `init`s, `queryMixinClass` answers 0 and `baseClass` answers
the class itself. Afterwards it is a mixin class, which `queryMixinClass`
shows, its superclass is the receiver, and its `baseClass` is the receiver or,
when the receiver is a mixin class, the receiver's base class. Only a mixin
class can be inherited: the new class can be named in `inherit` of a class
whose superclass is the receiver or a descendant of it.

```rexx
m = .k~mixinClass('Mix')
say m~id m~queryMixinClass m~superclass~id m~baseClass~id m~class~id
say .k~subclasses~items m~isAbstract
c = .k~subclass('Heir')
c~inherit(m)
say c~superclasses~items c~superclasses[2]~id c~new~isA(m) c~new~isA(.k)
m2 = m~mixinClass('Mix2')
say m2~superclass~id m2~baseClass~id
mm = .k~mixinClass('WithMeta', .meta)
say mm~class~id mm~queryMixinClass
call try '.k~mixinClass'
call try '.k~mixinClass(.nil)'
call try '.k~mixinClass("X", 1)'
call try '.k~mixinClass("X", .k)'
call try '.k~mixinClass("X", .class, 1)'
call try '.k~mixinClass("X", , 1, 2)'
call try '.string~inherit(m)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r~id
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::class meta subclass class
```

```output
Mix 1 K K Class
1 0
2 Mix 1 1
Mix K
META 1
.k~mixinClass => 88.901
.k~mixinClass(.nil) => 88.909
.k~mixinClass("X", 1) => 99.927
.k~mixinClass("X", .k) => 99.927
.k~mixinClass("X", .class, 1) => 97.1
.k~mixinClass("X", , 1, 2) => 93.902
.string~inherit(m) => 98.985
```

```rexx
y = .k~mixinclass('M2', .meta)
say y~queryMixinClass y~baseclass~id
::class k
::class meta subclass class
::method init
  say self~id self~superclass~id self~queryMixinClass self~baseclass~id
  forward class (super) continue
```

```output defect=#135
M2 Object 0 M2
M2 K 0 M2
1 K
```

r[class.class.enhanced]
`enhanced` answers a new object of the receiver whose own methods are those of
a collection, added to the methods of the class. The first argument is the
collection: its indexes are the names and its items are method objects,
strings or arrays of strings, or `.nil` to hide a name, as
[`defineMethods`](r:class.class.definemethods) takes them, with the same
errors, except for method object items: the new object holds a method that
runs the code of the item, with `.nil` as its scope. It is not specified
whether that method object is the item itself or a copy, or what scope the
item answers afterwards. The items are made into methods before `new` is sent,
so an error in one sends no `new`; the arguments after it are the arguments of
the new object. The method sends the collection the message `supplier`, with
no argument, and then sends the receiver the message `new` with the remaining
arguments, so that the class method `new` that the class declares runs and
`init` gets those arguments. The answer of `supplier` is read as [the
`defineMethods` sends rule](r:class.class.definemethods.sends) says: a
`Supplier` directly, any other object through four messages ([Deviation
108](r:class.class.deviation.108)). The object that `new` answers gets the
methods and is the answer. Its class is the receiver; the object follows the
rules of [the enhanced rule](r:class.methods.enhanced) for the search, and its
default name is that of [the default name rule](r:class.object.defaultname).
The abstract property of the receiver is not checked, and the receiver need
not accept the arguments after the collection: that is the business of its
`new` and `init`. The collection is required (`SYNTAX 93.901`), and an object
with no `supplier` method, `.nil` included, raises `97.1`.

```rexx
d = .directory~new
d['HI'] = 'return "hi"'
d['FOO'] = 'return "enhanced foo"'
e = .k~enhanced(d, 'a', 'b')
say e~class~id e~hi e~foo e~args (e~class == .k)
say .k~new~hasMethod('hi') .k~new~foo
say .k~enhanced(d)~args .k~enhanced(d, 1, 2, 3)~args
say .sub~enhanced(d)~hi .sub~enhanced(d)~foo
say .abs~enhanced(d)~class~id .abs~enhanced(d)~hi
say .string~enhanced(d, 'abc')~length .string~enhanced(d, 'abc')~hi
say .array~enhanced(d, 3)~size .array~enhanced(d, 3)~hi
say .k~enhanced(.array~of('return "one"'))~'1'
say .k~enhanced(.sp~new)~sp
call try '.k~enhanced'
call try '.k~enhanced(.nil)'
call try '.k~enhanced(1)'
call try '.k~enhanced(.object~new)'
call try '.noargs~enhanced(d, 1)'
call try '.string~enhanced(d)'
call try '.class~enhanced(d)'
d['BAD'] = .object~new
call try '.k~enhanced(d)'
d~remove('BAD')
em = .method~new('', 'return "e"')
z = .k~enhanced(.directory~new~~put(em, 'E'))
say z~e z~instanceMethod('E')~scope~string
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r~class~id
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::attribute args
::method init
  expose args
  args = arg()
::method foo
  return 'foo'
::class sub subclass k
::class abs abstract
::class noargs
::class sp
::method supplier
  return .directory~new~~put('return "sp ran"', 'SP')~supplier
```

```output
K hi enhanced foo 2 1
0 foo
0 3
hi enhanced foo
ABS hi
3 hi
3 hi
one
sp ran
.k~enhanced => 93.901
.k~enhanced(.nil) => 97.1
.k~enhanced(1) => 97.1
.k~enhanced(.object~new) => 97.1
.noargs~enhanced(d, 1) => 93.902
.string~enhanced(d) => 93.903
.class~enhanced(d) => 93.901
.k~enhanced(d) => 93.974
e The NIL object
```

r[class.class.enhanced.sends]
`enhanced` sends the collection `supplier` once, with no argument, before it
sends `new` to the receiver, and `new` is sent once. The `new` of the class
sends `init` as it always does; `enhanced` itself sends nothing else.

```rexx
o = .k~enhanced(.sp~new, 1)
say 'enhanced:' o~class~id
d = .directory~new
d['BAD'] = .object~new
signal on syntax name bad
x = .k~enhanced(d, 1)
say 'no error'
exit
bad: say condition('o')~code
::class k
::method new class
  say 'new sent' arg()
  forward class (super)
::method init
  say 'init sent' arg()
::class sp
::method supplier
  say 'supplier sent' arg()
  return .directory~new~supplier
```

```output
supplier sent 0
new sent 1
init sent 1
enhanced: K
93.974
```

## Deviations

r[class.class.deviation.32]
When the `new` of a metaclass answers a class that `subclass`,
`mixinclass` or a `::CLASS` directive has already completed, or a class of the
library such as `Array`, while `subclass` or
`mixinclass` makes a class with that metaclass, interrexx ends the program
with exit code 120; ooRexx rewires the existing class in place, so that `y~id`
in the examples answers `Array`. A `::CLASS` directive with METACLASS is
treated alike, as [the chapter rule](r:class.index.deviation.32) says. When a
class method `new` that `enhanced` reaches answers an object it did not build,
interrexx answers that object unchanged; ooRexx marks it enhanced, which
changes every object of its class, as [the chapter
rule](r:class.index.deviation.32) says. [The Object page
rule](r:class.object.deviation.32) has the example for that case.

```rexx
say 'before'
y = .k~subclass('Y')
say 'after' y~id
::class meta subclass class
::method new class
  if arg(1) = 'Y' then return .array
  forward class (super)
::class k metaclass meta
```

```output rc=120
before
```

```rexx
say 'before'
y = .k~mixinclass('Y')
say 'after' y~id
::class meta subclass class
::method new class
  if arg(1) = 'Y' then return .array
  forward class (super)
::class k metaclass meta
```

```output rc=120
before
```

```rexx
say 'before'
y = .object~subclass('Y', .meta)
say 'after' y~id
::class meta subclass class
::method new class
  return .array
```

```output rc=120
before
```

r[class.class.deviation.114]
A metaclass whose `new` answers an object that is not a class makes `subclass`
or `mixinclass` raise `SYNTAX 98.900`, whatever the object is, `.nil` and a
string included, and the error can be trapped by the program that sent the
message, as [the chapter rule](r:class.index.deviation.114) says. ooRexx
crashes. The metaclass is the second argument in the example.

```rexx
signal on syntax name e1
y = .object~subclass('Y', .meta)
say 'not reached'
e1: say 'subclass' condition('o')~code
signal on syntax name e2
z = .object~mixinclass('Z', .meta)
say 'not reached'
e2: say 'mixinclass' condition('o')~code
::class meta subclass class
::method new class
  return .nil
```

```output defect=#156
subclass 98.900
mixinclass 98.900
```

r[class.class.deviation.33]
The order of the entries that `methods` yields is not specified and may
differ from ooRexx's ([Deviation 33](../deviations.md)); which entries are
there, and how often, is specified.

```rexx
t = .top~methods(.top)
a = .array~new
do while t~available
  a~append(t~index)
  t~next
end
say a~sort~makestring('l', ' ')
::class top
::method hello
  return 1
::method goodbye
  return 2
```

```output
GOODBYE HELLO
```

r[class.class.deviation.63]
A class object is never copied: a `copy` that is sent with a scope, as
`.a~copy:.object` is, raises `SYNTAX 93.970` as every other route does ([the
copy rule](r:class.class.copy)). ooRexx copies the class in that case.

```rexx
call try '.a~copy:.object'
call try '.a~copy:.class'
call try '.object~copy:.object'
call try '.a~send(.array~of("copy", .object))'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r~id
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class a
```

```output
.a~copy:.object => 93.970
.a~copy:.class => 93.970
.object~copy:.object => 93.970
.a~send(.array~of("copy", .object)) => 93.970
```

r[class.class.deviation.108]
When `subclass`, `mixinclass`, `defineMethods` or `enhanced` takes the methods
of an object through its `supplier` method, the object that `supplier` answers
is used through the messages `available`, `index`, `item` and `next` when it
is not a `Supplier`; an object that does not answer one of them raises `SYNTAX
97.1` as for any message ([Deviation 108](../deviations.md)). A `Supplier`, an
instance of a subclass of `Supplier` included, is read directly, as [the sends
rule](r:class.class.definemethods.sends) says. ooRexx reads the answer as a
Supplier without checking it, and crashes when it is not one. In the first
example the `supplier` method of `holder` answers its argument, `lister` is no
`Supplier` and records the messages sent to it, and `broken` lacks `next`. The
second example gives `enhanced` such a collection.

```rexx
.local~seen = .directory~new
.k~defineMethods(.holder~new(.lister~new))
say .k~new~m1 .k~new~m2 .local~seen~allIndexes~sort~makestring('l', ' ')
x = .k~subclass('S', , .holder~new(.lister~new))
y = .k~mixinclass('M', , .holder~new(.lister~new))
say x~m1 y~m2
signal on syntax name bad
.k~defineMethods(.holder~new(.broken~new))
exit
bad: say condition('o')~code
::class k
::class holder
::method init
  expose obj
  use arg obj
::method supplier
  expose obj
  return obj
::class lister
::method init
  expose i
  i = 1
::method available
  expose i
  .local~seen['available'] = 1
  return i <= 2
::method index
  expose i
  .local~seen['index'] = 1
  return 'M'i
::method item
  expose i
  .local~seen['item'] = 1
  return 'return' i
::method next
  expose i
  .local~seen['next'] = 1
  i = i + 1
::class broken
::method available
  return 1
::method index
  return 'M9'
::method item
  return 'return 9'
```

```output defect=#139
1 2 available index item next
1 2
97.1
```

```rexx
e = .k~enhanced(.holder~new(.lister~new))
say e~m1 e~m2 .k~new~hasMethod('m1')
::class k
::class holder
::method init
  expose obj
  use arg obj
::method supplier
  expose obj
  return obj
::class lister
::method init
  expose i
  i = 1
::method available
  expose i
  return i <= 2
::method index
  expose i
  return 'M'i
::method item
  expose i
  return 'return' i
::method next
  expose i
  i = i + 1
```

```output defect=#139
1 2 0
```

r[class.class.deviation.109]
An instance of a subclass of `String` that is the id of `subclass` or
`mixinclass`, or the name given to `define`, `method` or `delete`, is read by
its own characters, and `MAKESTRING` and `STRING` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx reads it through its
`MAKESTRING` method. In the example `MAKESTRING` answers another name.

```rexx
n = .name~new('Foo')
k = .object~subclass(n)
say k~id
k~define(.name~new('hi'), 'return "hi"')
say k~method('HI')~class~id k~new~hi
::class name subclass string
::method makestring
  return 'other'
```

```output defect=#140
Foo
Method hi
```

r[class.class.deviation.110]
`define` reaches the instances of the class that exist already and the ones
that are made afterwards, as [the rule on changes](r:class.methods.late) says
and [Deviation 110](r:class.methods.deviation.110) states; ooRexx copies the
method table into each existing instance, which a `define` sent to the class
does not reach. In the example the `define` messages follow the instance, so
ooRexx raises `97.1` for the first message.

```rexx
o = .k~new
.k~define('ONE', 'return "one"')
.k~define('TWO', 'return "two"')
say o~one o~two
::class k
```

```output defect=#141
one two
```
