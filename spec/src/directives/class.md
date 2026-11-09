---
element: "::CLASS"
kind: directive
syntax: "::CLASS name [METACLASS m] [SUBCLASS s | MIXINCLASS s] [PUBLIC | PRIVATE] [ABSTRACT] [INHERIT i ...]"
rexxref: "Directives > ::CLASS"
---

# ::CLASS

::CLASS declares a class. The class comes into being when the package is
installed, before the prolog runs, and the methods that the ::METHOD
directives after it declare belong to it until the next ::CLASS. This page
specifies the name of the class, the options that place it in a hierarchy and
make it visible to other packages, and the order in which the classes of a
package are made. What a class object does once it exists, the methods of
`Class` such as `new`, `id` and `superclass` among them, is specified in the
classes chapter; the examples here use those methods only to show what the
directive made.

## The class and its name

r[dir.class.create]
::CLASS creates one class, named by a symbol or a string, and the class is
reachable from every code of the package as the environment symbol that is a
period followed by its name, from the point that [the installation
rule](r:dir.index.install) gives. The class's `id` is the name that [the name
rule](r:dir.index.names) gives, and `.shown` and `.SHOWN` are one class. A
class declared with no SUBCLASS or MIXINCLASS option is a subclass of
`Object`, and a class is a mixin class only when MIXINCLASS says so. A program
may declare a class named like a built-in one, and its own then hides the
built-in one in every code of the package.

```rexx
say .plain~superclass~id .plain~new~class~id
say .Shown~id .ALSO~id .also~id
say .array~new~hi .array~new~class~id
::class plain
::class Shown
::class "Also"
::class array
::method hi
  return 'my array'
```

```output
Object PLAIN
SHOWN Also Also
my array ARRAY
```

r[dir.class.duplicate]
A package may declare a class name once. A second ::CLASS that names it again
raises `SYNTAX 99.901`, whatever the case of either spelling, whether the
directive is a symbol or a string and whether the first one was PUBLIC. The
error is found when the program is read, as [the directive
rule](r:dir.index.directive) says for the other faults of the text.

```rexx
call try '::class Foo', '::class FOO'
call try '::class Foo', '::class "Foo"'
call try '::class Foo public', '::class foo private'
call try '::class Foo', '::class Bar'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-class-dup', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::class Foo / ::class FOO => 99.901
::class Foo / ::class "Foo" => 99.901
::class Foo public / ::class foo private => 99.901
::class Foo / ::class Bar => ok
```

## Options

r[dir.class.options]
The keywords after the name are METACLASS, SUBCLASS, MIXINCLASS, PUBLIC,
PRIVATE, ABSTRACT and INHERIT, in any case. They may stand in any order,
except that INHERIT takes every word that follows it as the name of a class,
so it comes last. Each of the four groups METACLASS, SUBCLASS or MIXINCLASS,
PUBLIC or PRIVATE and ABSTRACT may appear once. A word that is none of the
keywords, a keyword repeated, and a keyword that excludes one already given,
such as PRIVATE after PUBLIC or SUBCLASS after MIXINCLASS, raise
`SYNTAX 25.901`. METACLASS, SUBCLASS, MIXINCLASS and INHERIT without a name
after them raise `19.906`, `19.907`, `19.913` and `19.908`, and a ::CLASS with
no name raises `19.901`. Every one of these is found when the program is read.

```rexx
call try '::class a public private'
call try '::class a public public'
call try '::class a mixinclass object subclass object'
call try '::class a abstract abstract'
call try '::class a frob'
call try '::class a inherit m public'
call try '::class A SubClass Object Public Abstract'
call try '::class a public abstract subclass object metaclass class'
call try '::class'
call try '::class a metaclass'
call try '::class a subclass'
call try '::class a mixinclass'
call try '::class a inherit'
exit
try:
  signal on syntax name e
  p = .package~new('spec-dir-class-opt', .array~of(arg(1), -
    '::class m mixinclass object'))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
::class a public private => 25.901
::class a public public => 25.901
::class a mixinclass object subclass object => 25.901
::class a abstract abstract => 25.901
::class a frob => 25.901
::class a inherit m public => 98.909
::class A SubClass Object Public Abstract => ok
::class a public abstract subclass object metaclass class => ok
::class => 19.901
::class a metaclass => 19.906
::class a subclass => 19.907
::class a mixinclass => 19.913
::class a inherit => 19.908
```

> **Note.** The `inherit m public` case reads `public` as the name of a second
> mixin class, and the package fails to find it, `98.909`, when it is
> installed.

r[dir.class.visibility]
A class is private unless PUBLIC says otherwise. A private class is known only
to the package that declares it. A public class is also known to every package
that requires the package, directly or through a chain of ::REQUIRES, and a
class name is looked up first among the classes that the package declares,
then among the public classes of the packages it requires, taking the first of
those in the order of the ::REQUIRES directives. The order is depth first: a
required package contributes its own public classes, then those it makes
visible by its own ::REQUIRES, and all of these come before the classes of the
next ::REQUIRES directive of the requiring package. A private class of a
required package does not hide a public class of the same name that the
package makes visible. When none of these has the name, the lookup goes on as
[the environment rule](r:lex.symbols.environment) says, which finds the
built-in classes and any class object that the program has stored in
`.environment` under the name.
This book does not specify which other directories that lookup searches. The
same lookup finds the class that SUBCLASS, MIXINCLASS, METACLASS and INHERIT
name. A name that no step finds is the environment symbol's own text, and the
options that need a class raise the error that their rule gives.

```rexx
f = 'spec-dir-class-lib.tmp'
g = 'spec-dir-class-lib2.tmp'
call sysfiledelete f
call sysfiledelete g
call lineout f, '::class shown public'
call lineout f, '::method hi'
call lineout f, '  return "shown, built from" .hidden~new~hi'
call lineout f, '::class hidden'
call lineout f, '::method hi'
call lineout f, '  return "hidden"'
call lineout f, '::class box public'
call lineout f, '::method hi'
call lineout f, '  return "first box"'
call lineout f
call lineout g, '::class box public'
call lineout g, '::method hi'
call lineout g, '  return "second box"'
call lineout g
src = .array~of('say .shown~new~hi', -
  'say .hidden~class~id', -
  'say .box~new~hi', -
  '::requires "'f'"', -
  '::requires "'g'"')
p = .package~new('spec-dir-class-vis', src)
src = .array~of('say .box~new~hi', -
  '::requires "'g'"', -
  '::requires "'f'"')
p = .package~new('spec-dir-class-vis2', src)
src = .array~of('say .box~new~hi', -
  '::requires "'f'"', -
  '::class box', -
  '::method hi', -
  '  return "local box"')
p = .package~new('spec-dir-class-vis3', src)
src = .array~of('::requires "'f'"', '::class sub subclass hidden')
signal on syntax name e
p = .package~new('spec-dir-class-vis4', src)
e: say condition('o')~code
h = 'spec-dir-class-lib3.tmp'
call sysfiledelete h
call lineout h, '::requires "'f'"'
call lineout h
src = .array~of('say .shown~id', 'say .hidden', -
  '::requires "'h'"')
p = .package~new('spec-dir-class-vis5', src)
src = .array~of('say .box~new~hi', '::requires "'h'"', '::requires "'g'"')
p = .package~new('spec-dir-class-vis7', src)
k = 'spec-dir-class-lib4.tmp'
call sysfiledelete k
call lineout k, '::requires "'f'"'
call lineout k, '::class box'
call lineout k, '::method hi'
call lineout k, '  return "private box"'
call lineout k
src = .array~of('say .box~new~hi', '::requires "'k'"')
p = .package~new('spec-dir-class-vis8', src)
call sysfiledelete k
.environment['SPEC_DIR_CLASS_ENVCLASS'] = .queue
src = .array~of('say .envsub~superclass~id', -
  '::class envsub subclass spec_dir_class_envclass')
p = .package~new('spec-dir-class-vis6', src)
call sysfiledelete f
call sysfiledelete g
call sysfiledelete h
```

```output
shown, built from hidden
String
first box
second box
local box
98.909
SHOWN
.HIDDEN
first box
first box
Queue
```

## The hierarchy

r[dir.class.subclass]
SUBCLASS names the class's superclass, by a symbol or a string that the lookup
of [the visibility rule](r:dir.class.visibility) resolves. The new class
inherits the instance methods and the class methods of the superclass, and the
class that the program names may be a built-in one. A name written with a
leading period is looked up as written: `subclass .object` looks for a class
named `.OBJECT`, which no ::CLASS declares unless it spells its own name that
way, so the built-in `Object` is not found. A name that is found
nowhere raises `SYNTAX 98.909` when the package is installed, and a class that
is its own superclass, directly or through the classes it names, raises
`98.911`.

```rexx
call try '::class a subclass array'
call try '::class a subclass "String"'
call try '::class a subclass .object'
call try '::class a subclass nosuchsuperclass'
call try '::class a subclass a'
call try '::class a subclass b', '::class b subclass a'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-class-sub', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::class a subclass array => ok
::class a subclass "String" => ok
::class a subclass .object => 98.909
::class a subclass nosuchsuperclass => 98.909
::class a subclass a => 98.911
::class a subclass b / ::class b subclass a => 98.911
```

r[dir.class.mixin]
MIXINCLASS makes the class a mixin class and names its superclass as SUBCLASS
does, so that the name is required. Only a mixin class may be inherited by
INHERIT, which takes a list of class names, and the classes of the list are
inherited in the order written. INHERIT of a class that is not a mixin class
raises `SYNTAX 98.942`. A class that appears twice in the list, or is both the
superclass and in the list, raises `98.944`. A class that would become its own
ancestor through INHERIT, itself included, raises `98.911`. The class that
MIXINCLASS names for a mixin class that is inherited must be the superclass of
the inheriting class or one of the superclass's ancestors, or INHERIT raises
`SYNTAX 98.943` when the package is installed. When that class is itself a
mixin class, the test applies to the class its own MIXINCLASS names, and so on
down to the first class that is not a mixin class. A mixin class can itself be
a subclass and inherit other mixin classes.

```rexx
call try '::class c inherit m1 m2'
call try '::class c inherit plain'
call try '::class c inherit m1 m1'
call try '::class c subclass m1 inherit m1'
call try '::class c inherit c'
call try '::class m3 mixinclass m1 inherit m2'
exit
try:
  signal on syntax name e
  src = .array~of('::class m1 mixinclass object', -
    '::class m2 mixinclass object', '::class plain', arg(1))
  p = .package~new('spec-dir-class-mix', src)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
::class c inherit m1 m2 => ok
::class c inherit plain => 98.942
::class c inherit m1 m1 => 98.944
::class c subclass m1 inherit m1 => 98.944
::class c inherit c => 98.911
::class m3 mixinclass m1 inherit m2 => ok
```

```rexx
call try '::class ms mixinclass string', '::class c inherit ms'
call try '::class ms mixinclass string', -
  '::class c subclass string inherit ms'
call try '::class ms mixinclass object', -
  '::class c subclass string inherit ms'
call try '::class ms mixinclass array', -
  '::class c subclass string inherit ms'
call try '::class ms mixinclass object', '::class m2 mixinclass ms', -
  '::class c subclass string inherit m2'
call try '::class ms mixinclass string', '::class m2 mixinclass ms', -
  '::class c subclass string inherit m2'
call try '::class ms mixinclass string', '::class m2 mixinclass ms', -
  '::class c inherit m2'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-class-mix2', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::class ms mixinclass string / ::class c inherit ms => 98.943
::class ms mixinclass string / ::class c subclass string inherit ms => ok
::class ms mixinclass object / ::class c subclass string inherit ms => ok
::class ms mixinclass array / ::class c subclass string inherit ms => 98.943
::class ms mixinclass object / ::class m2 mixinclass ms / ::class c subclass string inherit m2 => ok
::class ms mixinclass string / ::class m2 mixinclass ms / ::class c subclass string inherit m2 => ok
::class ms mixinclass string / ::class m2 mixinclass ms / ::class c inherit m2 => 98.943
```

r[dir.class.search]
A message sent to an instance finds its method by looking first at the methods
the class itself declares and then at those of the classes it descends from
through SUBCLASS and INHERIT, in the order that
[the order rule](r:class.methods.order) of the
[method search](../classes/methods.md) gives; the first method found runs.
When the superclass and its ancestors inherit no mixin classes and each class
that INHERIT names is a mixin class based on `Object` that inherits none
itself, as in the example, that order is the class, its superclass and the
superclass's ancestors other than `Object`, the mixin classes in the order
written, and `Object` last. A superclass then hides a method of a mixin class
that the subclass inherits, and an earlier mixin class hides a later one. A
mixin class with another base class, and a class that more than one listed
class reaches, stand where [the base rule](r:class.methods.order.base) and
[the shared-class rule](r:class.methods.order.shared) say.

```rexx
o = .c~new
say o~who o~only1 o~only2 o~deep o~own
::class s0
::method deep
  return 's0 deep'
::class s subclass s0
::method who
  return 's who'
::method only1
  return 's only1'
::class m1 mixinclass object
::method who
  return 'm1 who'
::method only1
  return 'm1 only1'
::method only2
  return 'm1 only2'
::method deep
  return 'm1 deep'
::class m2 mixinclass object
::method only2
  return 'm2 only2'
::class c subclass s inherit m1 m2
::method own
  return 'c own'
```

```output
s who s only1 m1 only2 s0 deep c own
```

r[dir.class.metaclass]
METACLASS names a class that is a subclass of `Class`, and the instance
methods of that class become class methods of the new class, so that they
answer messages sent to the class itself. A class method that the new class
declares with the CLASS option hides a metaclass method of the same name, and
both hide a class method that the new class inherits. A subclass has its
superclass's metaclass unless it names its own. A metaclass name that is found
nowhere raises `SYNTAX 98.908`, where SUBCLASS, MIXINCLASS and INHERIT raise
`98.909`. A name that is found and is not a subclass of `Class` raises
`SYNTAX 99.927` when the package is installed, after the classes created
before it have received INIT as [the failure rule](r:dir.index.failure) says,
and a metaclass that is the class itself raises `98.911`.

```rexx
say .k~hello .k~both
say .sub~hello .sub~both .sub~metaclass~id
say .sub2~hello .sub2~metaclass~id
say .plain~metaclass~id
say .k4~hello .k5~hello .k6~hello
call try '::class a metaclass object'
call try '::class a metaclass a'
call try '::class a metaclass spec_dir_nometa'
exit
try:
  signal on syntax name e
  p = .package~new('spec-dir-class-meta', .array~of(arg(1)))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
::class meta subclass class
::method hello
  return 'meta hello'
::method both
  return 'meta both'
::class k metaclass meta
::method both class
  return 'k both'
::class sub subclass k
::class meta2 subclass class
::method hello
  return 'meta2 hello'
::class sub2 subclass k metaclass meta2
::class plain
::class base
::method hello class
  return 'base hello'
::class k4 subclass base
::class k5 subclass base metaclass meta
::class k6 subclass base metaclass meta
::method hello class
  return 'k6 hello'
```

```output
meta hello k both
meta hello k both META
meta2 hello META2
Class
base hello meta hello k6 hello
::class a metaclass object => 99.927
::class a metaclass a => 98.911
::class a metaclass spec_dir_nometa => 98.908
```

```rexx
say 'prolog'
::class k
::method init class
  say 'init k'
::class notmeta
::class a metaclass notmeta
```

```output rc=157
init k
```

r[dir.class.abstract]
ABSTRACT marks the class as one that has no instances of its own: the NEW
method that `Class` supplies raises `SYNTAX 98.989` for it, before any INIT
runs, and a class method NEW that the abstract class declares itself runs
instead. A class that has the abstract class as a
superclass, or that inherits it, is not abstract unless it says so, and makes
instances. A class can be abstract and a mixin class at once.

```rexx
call try '.base~new'
call try '.sub~new'
call try '.sub2~new'
call try '.user~new'
call try '.mixed~new'
call try '.own~new'
exit
try:
  signal on syntax name e
  interpret 'say arg(1) "=>"' arg(1)'~class~id'
  return
e: say arg(1) '=>' condition('o')~code
  return
::class base abstract
::class sub subclass base
::class sub2 subclass sub abstract
::class mix mixinclass object abstract
::class user inherit mix
::class mixed abstract mixinclass object
::class own abstract
::method new class
  return 'made'
```

```output
.base~new => 98.989
.sub~new => SUB
.sub2~new => 98.989
.user~new => USER
.mixed~new => 98.989
.own~new => String
```

## Order of creation

r[dir.class.forward]
A class may name a class that the package declares further on, in SUBCLASS,
MIXINCLASS, METACLASS or INHERIT. The classes are created in text order,
except that a class waits until every class it names has been created, and is
then created at once, before the next class in the text. The order of creation
is the order in which the classes receive their INIT and ACTIVATE messages, as
[the installation rule](r:dir.index.install) says. A class that names itself,
directly or through the classes it names, raises `SYNTAX 98.911` before any
class of the package is created, so that no INIT has run, not even for the
classes that come before it in the text and take no part in the cycle.

```rexx
say 'prolog'
::class x subclass y
::method init class
  say 'init x'
::class w
::method init class
  say 'init w'
::class y subclass z
::method init class
  say 'init y'
::class z
::method init class
  say 'init z'
::class v
::method init class
  say 'init v'
```

```output
init w
init z
init y
init x
init v
prolog
```

```rexx
say 'prolog'
::class a metaclass b
::method init class
  say 'init a'
::class c inherit d
::method init class
  say 'init c'
::class e mixinclass f
::method init class
  say 'init e'
::class g
::method init class
  say 'init g'
::class b subclass class
::method init class
  say 'init b'
::class d mixinclass object
::method init class
  say 'init d'
::class f mixinclass object
::method init class
  say 'init f'
```

```output
init g
init b
init a
init d
init c
init f
init e
prolog
```

```rexx
say 'prolog'
exit
::class k
::method init class
  say 'init k'
::class a subclass b
::class b subclass a
::class z
::method init class
  say 'init z'
```

```output rc=158
```
