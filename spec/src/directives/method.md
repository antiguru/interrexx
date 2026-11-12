---
element: "::METHOD"
kind: directive
syntax: "::METHOD name [CLASS] [PUBLIC | PRIVATE | PACKAGE] [GUARDED | UNGUARDED] [PROTECTED | UNPROTECTED] [ATTRIBUTE | ABSTRACT | EXTERNAL spec | DELEGATE var]"
rexxref: "Directives > ::METHOD"
deviations: [105]
---

# ::METHOD

::METHOD declares a method. Its section, the clauses up to the next directive,
is the method's code, and the method belongs to the class of the most recent
::CLASS directive. A ::METHOD before any ::CLASS belongs to no class. This
page specifies the name of a method, the options that set who may send it and
how it is guarded, the forms that have no code of their own, and the method
that belongs to no class. What a [method object](r:class.method.creation) does
once it exists, and the methods of `Class` that [add one to a
class](r:class.class.define), belong to the classes chapter.

## The method and its name

r[dir.method.create]
The section of a ::METHOD is the code of an instance method of the class that
the most recent ::CLASS declared, and the directive clause itself is not part
of it. The method runs with `SELF` as the object it was sent to, and it reads
and writes the object's variables through [EXPOSE](r:instr.expose.object). The
most recent ::CLASS is the one that counts even when other directives, such as
a ::ROUTINE, come between. A method with no clause in its section returns no
value, so a message whose result is used raises `SYNTAX 91.999`. The code is a
[unit of its own](r:dir.index.unit): a label or variable of another section is
not known in it.

```rexx
o = .counter~new
say o~add(2) o~add(3)
say o~total
o~empty
signal on syntax name e
x = o~empty
e: say condition('o')~code
::class counter
::routine helper
  return 'a routine between'
::method add
  expose n
  use arg step
  if \datatype(n, 'W') then n = 0
  n += step
  return n
::method total
  expose n
  return n
::method empty
```

```output
2 5
5
91.999
```

r[dir.method.name]
The name of a method is a symbol or a string, and [the name
rule](r:dir.index.names) gives the name, so `::method "Mixed"` is found by
`mixed` and by `MIXED`. A string may hold any characters, so it can name an
operator method such as `"+"`; a symbol cannot, and a `::method` followed by
something that is not a symbol or a string raises `SYNTAX 19.902`. A class
declares one method of a name: a second ::METHOD of the same name, in any
case, raises `SYNTAX 99.902`. An instance method and a class method of the
same name are two methods.

```rexx
o = .box~new
say o~mixed o~MIXED o~"MiXeD"
say o~"+"(1) o~"a b"
say .box~mixed
call try '::method +'
call try '::class k', '::method m', '::method M'
call try '::class k', '::method m', '::method "m"'
call try '::class k', '::method m', '::method m class'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-method-name', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
::class box
::method "Mixed"
  return 'instance'
::method mixed class
  return 'class'
::method "+"
  use arg x
  return x + 1
::method "a b"
  return 'blank'
```

```output
instance instance instance
2 blank
class
::method + => 19.902
::class k / ::method m / ::method M => 99.902
::class k / ::method m / ::method "m" => 99.902
::class k / ::method m / ::method m class => ok
```

## Options

r[dir.method.options]
The keywords after the name are CLASS, PUBLIC, PRIVATE, PACKAGE, GUARDED,
UNGUARDED, PROTECTED, UNPROTECTED, ATTRIBUTE, ABSTRACT, EXTERNAL and DELEGATE,
in any case and in any order. A method names at most one keyword of each of
these groups: access (PUBLIC, PRIVATE, PACKAGE), guarding (GUARDED,
UNGUARDED), protection (PROTECTED, UNPROTECTED) and the kind of code
(ABSTRACT, EXTERNAL, DELEGATE), and CLASS and ATTRIBUTE at most once each.
ATTRIBUTE goes with any one keyword of the kind group. A word that is none of
the keywords, a keyword repeated or a second keyword of one group raises
`SYNTAX 25.902`. EXTERNAL and DELEGATE take an operand, and the other keywords
do not. The errors of the directive are found when the program is read. A
clause after an ATTRIBUTE method raises the error that [the attribute
rule](r:dir.method.attribute) gives, except with DELEGATE, where it raises the
error of [the delegate rule](r:dir.method.delegate).

```rexx
call try '::method m public private'
call try '::method m private package'
call try '::method m guarded unguarded'
call try '::method m protected protected'
call try '::method m class class'
call try '::method m abstract delegate d'
call try '::method m frob'
call try '::method m abstract external "library x"'
call try '::method m abstract attribute'
call try '::method m attribute delegate d'
call try '::method m Unguarded PRIVATE Protected'
exit
try:
  signal on syntax name e
  p = .package~new('spec-dir-method-opt', .array~of(arg(1)))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
::method m public private => 25.902
::method m private package => 25.902
::method m guarded unguarded => 25.902
::method m protected protected => 25.902
::method m class class => 25.902
::method m abstract delegate d => 25.902
::method m frob => 25.902
::method m abstract external "library x" => 25.902
::method m abstract attribute => ok
::method m attribute delegate d => ok
::method m Unguarded PRIVATE Protected => ok
```

r[dir.method.class]
CLASS makes the method a class method: the section is code that answers
messages sent to the class object itself, with `SELF` the class, and not
messages sent to its instances. The subclasses of the class inherit it as they
inherit any class method, and an instance does not see it. CLASS requires a
::CLASS directive before the ::METHOD, and without one the directive raises
`SYNTAX 99.905`, found when the program is read.

```rexx
say .shape~describe
say .square~describe
say .square~new~respond
signal on syntax name e
say .square~new~describe
e: say condition('o')~code
call try '::method m class'
exit
try:
  signal on syntax name f
  p = .package~new('spec-dir-method-cls', .array~of(arg(1)))
  say arg(1) '=> ok'
  return
f: say arg(1) '=>' condition('o')~code
  return
::class shape
::method describe class
  return 'class' self~id
::method respond
  return 'instance of' self~class~id
::class square subclass shape
```

```output
class SHAPE
class SQUARE
instance of SQUARE
97.1
::method m class => 99.905
```

r[dir.method.access]
PUBLIC is the default, and a public method accepts a message from any sender.
A PRIVATE method, an instance method or a class method, accepts a message
only from a method whose `SELF`, the sender, is the class that declares it or
a subclass of it, or is an instance of one of those; a class that inherits a
mixin class counts as a subclass of it. The sender may be another
object, so objects of one class can send each other private messages, and a
class method of the class can send a private instance method. A message from
the prolog, from a routine, or from a method of an unrelated class raises
`SYNTAX 97.2`. The test is on `SELF`, not on the class whose method runs, so a
method of a superclass that runs with an instance of the superclass as
`SELF` cannot send a private method of a subclass, and the same method running
with an instance of the subclass as `SELF` can. A PACKAGE method accepts a
message from any code of the package that declares it and from no other
package, which raises `SYNTAX 97.3`. A message sent with START is checked as
[a Deviation](r:dir.method.deviation.105) states.

```rexx
a = .acct~new
b = .acct~new
c = .child~new
say a~take(b) a~take(c) c~take(a) .mixed~new~take(.mixed~new)
say .acct~ccall .child~cvia(a) a~clspriv
call try 'say a~secret'
call try 'say .other~new~take(a)'
call try 'say .other~cvia(a)'
call try 'say .acct~csecret'
call try 'say a~down(c)'
call try 'say c~down(c)'
call try 'say peek(a)'
exit
try:
  signal on syntax name e
  interpret arg(1)
  return
e: say arg(1) '=>' condition('o')~code
  return
::routine peek
  use arg o
  return o~secret
::class acct
::method secret private
  return 'secret'
::method csecret class private
  return 'csecret'
::method ccall class
  return self~csecret
::method cvia class
  use arg o
  return o~secret
::method clspriv
  return self~class~csecret
::method take
  use arg o
  return o~secret
::method down
  use arg o
  return o~childsecret
::class child subclass acct
::method childsecret private
  return 'childsecret'
::class other
::method take
  use arg o
  return o~secret
::method cvia class
  use arg o
  return o~secret
::class tag mixinclass object
::method hidden private
  return 'hidden'
::method take
  use arg o
  return o~hidden
::class mixed inherit tag
```

```output
secret secret secret hidden
csecret secret csecret
say a~secret => 97.2
say .other~new~take(a) => 97.2
say .other~cvia(a) => 97.2
say .acct~csecret => 97.2
say a~down(c) => 97.2
childsecret
say peek(a) => 97.2
```

```rexx
f = 'spec-dir-method-lib.tmp'
call sysfiledelete f
call lineout f, '::class vault public'
call lineout f, '::method code package'
call lineout f, '  return "1234"'
call lineout f, '::method open'
call lineout f, '  return "open with" self~code'
call lineout f, '::routine peek public'
call lineout f, '  use arg v'
call lineout f, '  return v~code'
call lineout f
src = .array~of('v = .vault~new', -
  'say v~open', -
  'say peek(v)', -
  'signal on syntax name lost', -
  'say v~code', -
  'lost: say condition("o")~code', -
  'signal on syntax name lost2', -
  'say my(v)', -
  'lost2: say condition("o")~code', -
  '::requires "'f'"', -
  '::routine my', -
  '  use arg v', -
  '  return v~code')
p = .package~new('spec-dir-method-pkg', src)
call sysfiledelete f
```

```output
open with 1234
1234
97.3
97.3
```

r[dir.method.guard]
A method is guarded unless UNGUARDED says otherwise, and the instruction page
of [GUARD](r:instr.guard.exclusion) says what that means: a guarded method
starts with the exclusive use of the object variables of its scope, and an
unguarded one does not. An unguarded method can therefore run while another
method holds the guard. GUARDED may be written, and it is the same as writing
no option.

```rexx
log = .array~new
o = .box~new(log)
h = o~start('hold')
do until o~inside; end
g = o~start('probe')
u = o~start('quick')
u~result
o~release
h~result; g~result
say log~makestring('l', ' ')
exit
::class box
::method init
  expose log inside go
  use arg log
  inside = .false
  go = .false
::method hold
  expose log inside go
  inside = .true
  do until go; end
  log~append('hold-end')
::method probe guarded
  expose log
  log~append('probe')
::method quick unguarded
  expose log
  log~append('quick')
::method inside unguarded
  expose inside
  return inside
::method release unguarded
  expose go
  go = .true
```

```output
quick hold-end probe
```

r[dir.method.protected]
PROTECTED marks the method so that a security manager, when the program has
one, decides whether the call may run. UNPROTECTED is the default. A program
with no security manager runs a protected method like any other, and the
security manager is not specified here.

```rexx
o = .box~new
say o~locked o~open
::class box
::method locked protected
  return 'ran'
::method open unprotected
  return 'ran too'
```

```output
ran ran too
```

## Methods without code of their own

r[dir.method.attribute]
ATTRIBUTE declares two methods at once and takes no code: a method of the
given name that returns the value of the object variable of that name, and a
method `name=` that assigns its one argument to the variable and returns no
value, so that a message whose result is used raises `SYNTAX 91.999`. The
object variable is named as the directive spells the name: a symbol gives it
in upper case, and a string as written, so an attribute that a lower-case
string declares is not the variable that an EXPOSE of the symbol names. The
access, guarding and protection options apply to both methods, and so does
CLASS, which makes them class methods. An unset variable returns its own name
as spelled. The first method takes no argument, and the second takes exactly
one: `SYNTAX 93.902` for an extra argument and `93.903` for a missing one. A
clause after the directive that is not a null clause raises `SYNTAX 99.934`,
found when the program is read, because the directive has no section; that
holds for ATTRIBUTE with ABSTRACT too, which declares both methods abstract,
and for ATTRIBUTE with EXTERNAL, which looks for its library as [the external
rule](r:dir.method.external) says. ATTRIBUTE with DELEGATE raises `99.946`
instead, as [the delegate rule](r:dir.method.delegate) says.

```rexx
o = .box~new
say o~size
o~size = 4
say o~size
.box~count = 7
say .box~count
say .vault~new~safe_open
call try 'say o~size(1)'
call try 'o~"SIZE="()'
call try 'o~"SIZE="(1, 2)'
call try 'x = o~"SIZE="(4)'
call try 'say .box~new~count'
call try 'say .vault~new~pin'
call try '.vault~new~pin = 1'
call try 'say .blank~new~limit'
call try '.blank~new~limit = 1'
call pkg 'size attribute'
call pkg 'size abstract attribute'
call pkg 'size attribute external "library spec_no_such_lib_xyz"'
call pkg 'size attribute delegate d'
o = .low~new
say o~sz o~"Mx" o~mx
o~set
say o~sz
o~sz = 7
say o~sz o~get
exit
pkg:
  signal on syntax name e3
  src = .array~of('::class k', '::method' arg(1), 'say 1')
  p = .package~new('spec-dir-method-attr', src)
  say 'ok'
  return
e3: say arg(1) '=>' condition('o')~code
  return
try:
  signal on syntax name e2
  interpret arg(1)
  return
e2: say arg(1) '=>' condition('o')~code
  return
::class box
::method size attribute
::method count attribute class
::class blank
::method limit attribute abstract
::class low
::method "sz" attribute
::method "Mx" attribute
::method set
  expose sz
  sz = 5
::method get
  expose sz
  return sz
::class vault
::method pin attribute private
::method safe_open
  self~pin = 1234
  return self~pin
```

```output
SIZE
4
7
1234
say o~size(1) => 93.902
o~"SIZE="() => 93.903
o~"SIZE="(1, 2) => 93.902
x = o~"SIZE="(4) => 91.999
say .box~new~count => 97.1
say .vault~new~pin => 97.2
.vault~new~pin = 1 => 97.2
say .blank~new~limit => 93.965
.blank~new~limit = 1 => 93.965
size attribute => 99.934
size abstract attribute => 99.934
size attribute external "library spec_no_such_lib_xyz" => 99.934
size attribute delegate d => 99.946
sz Mx Mx
sz
7 5
```

r[dir.method.abstract]
ABSTRACT declares a method that has no code, a method a subclass declares in
full. A message that reaches an abstract method raises
`SYNTAX 93.965`, and a subclass that declares a method of the same name
answers the message instead. The class that declares an abstract method is not
itself abstract. A clause after the directive that is not a null clause raises
`SYNTAX 99.933`, found when the program is read.

```rexx
call try '.base~new~area'
call try '.shape~new~area'
exit
try:
  signal on syntax name e
  interpret 'say arg(1) "=>"' arg(1)
  return
e: say arg(1) '=>' condition('o')~code
  return
::class base
::method area abstract
::class shape subclass base
::method area
  return 'filled in'
```

```output
.base~new~area => 93.965
.shape~new~area => filled in
```

r[dir.method.external]
EXTERNAL names a method that a native library supplies, and takes no code. Its
operand is a string whose first word is LIBRARY, in any case, followed by the
name of the library and optionally one more word, the entry in it. A missing
operand, or one that is not a string, raises `SYNTAX 19.905`. A string that
does not have that form, whose first word is another word, or that has no
library name or more than two words after LIBRARY, raises `99.917`; both are
found when the program is read. A clause after the directive that is not a
null clause raises `99.936`. A library that cannot be loaded raises
`SYNTAX 98.903` when the directive's section ends, which is before the next
directive is read. It therefore comes before every fault raised when the
package is installed and before a fault found when a later directive is read,
while a fault found when an earlier directive is read comes first, and a
clause in the section raises `99.936` and not `98.903`. With ATTRIBUTE the
library is looked for in the same way, and a clause after the directive raises
`99.934`, as [the attribute rule](r:dir.method.attribute) says. How a library
is searched for is [the library rule](r:env.search.library); which routine of
it the method runs is not specified here.

```rexx
call try '::method m external "library spec_no_such_lib_xyz"'
call try '::method m external "Library spec_no_such_lib_xyz entry"'
call try '::method m external "registered spec_no_such_lib_xyz"'
call try '::method m external "library a b c"'
call try '::method m external'
call try '::method m external library'
call try '::method m external ""'
call try '::method m external "library"'
call try '::method m external "library spec_no_such_lib_xyz"', '::frob x'
call try '::method m external "library spec_no_such_lib_xyz"', 'say 1'
call try '::class a subclass spec_no_such_super_xyz', -
  '::method m external "library spec_no_such_lib_xyz"'
call try '::frob x', '::method m external "library spec_no_such_lib_xyz"'
call try '::method m attribute external "library spec_no_such_lib_xyz"'
call try '::method m attribute external "library spec_no_such_lib_xyz"', -
  '::frob x'
exit
try:
  signal on syntax name e
  p = .package~new('spec-dir-method-ext', -
    .array~of('::class k', arg(1), arg(2)))
  say strip(arg(1) arg(2)) '=> ok'
  return
e: say strip(arg(1) arg(2)) '=>' condition('o')~code
  return
```

```output
::method m external "library spec_no_such_lib_xyz" => 98.903
::method m external "Library spec_no_such_lib_xyz entry" => 98.903
::method m external "registered spec_no_such_lib_xyz" => 99.917
::method m external "library a b c" => 99.917
::method m external => 19.905
::method m external library => 19.905
::method m external "" => 99.917
::method m external "library" => 99.917
::method m external "library spec_no_such_lib_xyz" ::frob x => 98.903
::method m external "library spec_no_such_lib_xyz" say 1 => 99.936
::class a subclass spec_no_such_super_xyz ::method m external "library spec_no_such_lib_xyz" => 98.903
::frob x ::method m external "library spec_no_such_lib_xyz" => 99.916
::method m attribute external "library spec_no_such_lib_xyz" => 98.903
::method m attribute external "library spec_no_such_lib_xyz" ::frob x => 98.903
```

r[dir.method.delegate]
DELEGATE takes the name of an object variable as a symbol and no code. A
message sent to the method is forwarded, with its arguments, to the object
that the variable holds, as if the section were `expose var` followed by
`forward to(var)`. The operand must be a symbol, and a missing operand or a
string raises `SYNTAX 20.926`; a symbol that cannot name a variable raises
`99.925`. A clause after the directive that is not a null clause raises
`99.946`. All three are found when the program is read.

```rexx
o = .front~new(.target~new)
say o~hello(1)
say o~size
call try '::method m delegate'
call try '::method m delegate "d"'
call try '::method m delegate 1'
exit
try:
  signal on syntax name e
  p = .package~new('spec-dir-method-del', .array~of('::class k', arg(1)))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
::class target
::method hello
  use arg x
  return 'target hello' x self~class~id
::method size
  return 42
::class front
::method init
  expose d
  use arg d
::method hello delegate d
::method size delegate d
```

```output
target hello 1 TARGET
42
::method m delegate => 20.926
::method m delegate "d" => 20.926
::method m delegate 1 => 99.925
```

## Methods of no class

r[dir.method.floating]
A ::METHOD that comes before every ::CLASS belongs to no class, and the
package keeps it as a method object of its own. Unless an entry or a class of
the same name is found first ([the order](r:env.objects.search.symbols)), the
environment symbol `.METHODS` holds a table of the package's floating methods,
whose indexes are the method names in upper case, and a lookup in it is by
exact index, so `.methods['greet']` is the `.nil` object. The symbol exists
only in a package that has a floating method; for any other package it is the
symbol's own text. Each package has its own table. A floating method may not
use CLASS, which raises `SYNTAX 99.905`, and a name declared twice raises
`99.902`, as the methods of a class do.

```rexx
say .methods~items
say .methods~allindexes~sort~makearray~makestring('l', ' ')
.box~define('GREET', .methods['GREET'])
say .box~new~greet('world')
say .methods['greet'] .methods['GREET']~class~id
call try '::method m class'
call try '::method m', '::method M'
call try 'say .methods', '::class k', '::method m'
call try 'say .methods~items', '::method a', '::method b', '::method c'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-method-fl', src)
  say 'ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
::method greet
  use arg who
  return 'hello' who
::method unused
  return 1
::class box
```

```output
2
GREET UNUSED
hello world
The NIL object Method
::method m class => 99.905
::method m / ::method M => 99.902
.METHODS
ok
3
ok
```

## Deviations

r[dir.method.deviation.105]
A message sent with START is checked for PRIVATE and PACKAGE access as the
same message sent directly by the code that starts it. A method of a class may
start a private method of its own class, code of a package may start a PACKAGE
method of that package, and a PACKAGE method started from another package
raises `SYNTAX 97.3`. ooRexx raises `97.2` for every started private message
and `97.1` for every started PACKAGE message, whichever code starts it.

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

```rexx
f = 'spec-dir-method-lib2.tmp'
call sysfiledelete f
call lineout f, '::class vault public'
call lineout f, '::method code package'
call lineout f, '  return "1234"'
call lineout f
src = .array~of('v = .vault~new', -
  'signal on syntax name lost', -
  'm = v~start("code")', -
  'say m~result', -
  'exit', -
  'lost: say condition("o")~code', -
  '::requires "'f'"')
p = .package~new('spec-dir-method-pkg2', src)
call sysfiledelete f
```

```output
97.3
```

```rexx
signal on syntax name lost
say .vault2~new~start('code')~result
exit
lost: say condition('o')~code
::class vault2
::method code package
  return 'package answer'
```

```output
package answer
```
