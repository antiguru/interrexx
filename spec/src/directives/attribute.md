---
element: "::ATTRIBUTE"
kind: directive
syntax: "::ATTRIBUTE name [GET | SET] [CLASS] [PUBLIC | PRIVATE | PACKAGE] [GUARDED | UNGUARDED] [PROTECTED | UNPROTECTED] [ABSTRACT | EXTERNAL spec | DELEGATE var]"
rexxref: "Directives > ::ATTRIBUTE"
---

# ::ATTRIBUTE

::ATTRIBUTE declares the accessor methods of an object variable: a getter that
returns the variable and a setter that assigns it, or only one of the two. A
directive with no code of its own generates both methods, and a directive with
GET or SET takes the code of that one method from its section. The options
that set who may send the messages and how they are guarded are the ones of
[::METHOD](method.md), and this page names them without restating them. What a
method object does once it exists, and the methods of `Class` that add one to
a class, belong to the classes chapter.

## The generated methods

r[dir.attribute.create]
::ATTRIBUTE with no GET and no SET declares two methods of the class that the
most recent [::CLASS](class.md) declared: a method of the attribute's name and
a method of that name followed by an equals sign. The first takes no argument
and returns the value of the object variable of that name, which is the
variable's own name, spelled as [the name rule](r:dir.attribute.name) gives
it, while it is unset. The second takes exactly one argument and assigns it to
the variable; it is what a message assignment `obj~name = value` sends, and it
returns no value, so a message whose result is used raises `SYNTAX 91.999`. An
extra argument raises `SYNTAX 93.902` and a missing one `93.903`. A clause
after the directive is refused as [the section rule](r:dir.attribute.section)
says. The form of [::METHOD](r:dir.method.attribute) with the keyword
ATTRIBUTE declares the same two methods.

```rexx
o = .box~new
say o~size
o~size = 4
say o~size
say o~peek
call try 'say o~size(1)'
call try 'o~"SIZE="()'
call try 'o~"SIZE="(1, 2)'
call try 'say o~"SIZE="(1)'
src = .array~of('::class k', '::attribute a', 'say 1')
signal on syntax name e
p = .package~new('spec-dir-attribute-create', src)
e: say condition('o')~code
exit
try:
  signal on syntax name e2
  interpret arg(1)
  return
e2: say arg(1) '=>' condition('o')~code
  return
::class box
::attribute size
::method peek
  expose size
  return size
```

```output
SIZE
4
4
say o~size(1) => 93.902
o~"SIZE="() => 93.903
o~"SIZE="(1, 2) => 93.902
say o~"SIZE="(1) => 91.999
99.937
```

r[dir.attribute.name]
The name is a symbol or a string. As [the names rule](r:dir.index.names) says,
a symbol names the variable in upper case and a string names it exactly as
written, so the variable of `::attribute "Mixed"` is not the one that the
symbol `mixed` names in an EXPOSE, and while it is unset the getter answers
`Mixed`, although the messages `mixed` and `mixed=` reach it, because the
lookup of a method by name [ignores case](r:dir.method.name). A name that
cannot be the name of a variable, such as a string with a blank or a number,
raises `SYNTAX 99.925`, and a directive whose name is missing or is neither a
symbol nor a string raises `19.914`. Both are found when the program is read.

```rexx
o = .box~new
o~"Mixed" = 5
o~lower = 6
say o~mixed o~lower
say o~peek
o2 = .box~new
say o2~mixed
call try '::attribute "a b"'
call try '::attribute 1'
call try '::attribute'
call try '::attribute +'
exit
try:
  signal on syntax name e
  p = .package~new('spec-dir-attribute-name', .array~of('::class k', arg(1)))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
::class box
::attribute "Mixed"
::attribute lower
::method peek
  expose Mixed lower
  return Mixed lower
```

```output
5 6
MIXED 6
Mixed
::attribute "a b" => 99.925
::attribute 1 => 99.925
::attribute => 19.914
::attribute + => 19.914
```

## GET and SET

r[dir.attribute.getset]
GET declares only the getter and SET only the setter, so that the two can have
different options. GET and SET together raise `SYNTAX 25.925`. A GET or SET
directive without a keyword of the kind group (ABSTRACT, EXTERNAL, DELEGATE)
that has a section takes the section as the code of its method,
with the directive clause left out, and without a section it declares the
generated method. In the code of a setter the argument is the value that
`name = value` assigned, and the variable is reached through
[EXPOSE](r:instr.expose.object) as in any method.

```rexx
o = .box~new
o~size = 3
say o~size o~peek
o~tag = 'abc'
say o~tag
o~note = 7
say o~note
signal on syntax name e
o~count = 1
e: say condition('o')~code
::class box
::attribute size get
::attribute size set
  expose size
  use arg value
  size = value * 10
::attribute tag get
  expose tag
  return 'tag:' tag
::attribute tag set
::attribute note get
::attribute note set
::attribute count get
::method peek
  expose size
  return size
```

```output
30 30
tag: abc
7
97.1
```

r[dir.attribute.options]
The keywords after the name are GET, SET, CLASS, PUBLIC, PRIVATE, PACKAGE,
GUARDED, UNGUARDED, PROTECTED, UNPROTECTED, ABSTRACT, EXTERNAL and DELEGATE,
in any case and in any order. The groups and the rules about them are those of
[::METHOD](r:dir.method.options), except that ATTRIBUTE is not a keyword and
GET and SET make a group of their own: a word that is none of the keywords, a
keyword repeated or a second keyword of one group raises `SYNTAX 25.925`. The
access, guarding and protection options apply to every method that the
directive declares, as [access](r:dir.method.access),
[guarding](r:dir.method.guard) and [protection](r:dir.method.protected) say
for a method. A GET and a SET directive for one name may therefore differ: a
public getter and a private setter make an attribute that others read and only
the object itself writes.

```rexx
o = .box~new
o~inside
say o~level
call try 'o~level = 4'
call try 'say o~secret'
call try 'o~secret = 1'
call decl '::attribute a get set'
call decl '::attribute a public private'
call decl '::attribute a class class'
call decl '::attribute a frob'
call decl '::attribute a attribute'
call decl '::attribute a abstract delegate d'
call decl '::attribute a Package Get Class Guarded Protected'
exit
try:
  signal on syntax name e
  interpret arg(1)
  return
e: say arg(1) '=>' condition('o')~code
  return
decl:
  signal on syntax name e2
  p = .package~new('spec-dir-attribute-opt', .array~of('::class k', arg(1)))
  say arg(1) '=> ok'
  return
e2: say arg(1) '=>' condition('o')~code
  return
::class box
::attribute level get
::attribute level set private
::attribute secret private
::method inside
  self~level = 8
```

```output
8
o~level = 4 => 97.2
say o~secret => 97.2
o~secret = 1 => 97.2
::attribute a get set => 25.925
::attribute a public private => 25.925
::attribute a class class => 25.925
::attribute a frob => 25.925
::attribute a attribute => 25.925
::attribute a abstract delegate d => 25.925
::attribute a Package Get Class Guarded Protected => ok
```

## Class attributes

r[dir.attribute.class]
CLASS makes the declared methods class methods, as it does for
[::METHOD](r:dir.method.class), and the variable that they reach is an object
variable of the class object. An instance does not see the methods, and an
attribute of the same name without CLASS reaches a different variable, so a
class and its instances can hold a value each under one name. CLASS requires a
::CLASS directive before it, and without one the directive raises
`SYNTAX 99.905`.

```rexx
.box~count = 3
say .box~count
o = .box~new
say o~count
o~count = 'mine'
say o~count .box~count
call try 'say o~tally'
call try '.box~new~tally = 1'
src = .array~of('::attribute a class')
signal on syntax name e
p = .package~new('spec-dir-attribute-class', src)
e: say condition('o')~code
exit
try:
  signal on syntax name e2
  interpret arg(1)
  return
e2: say arg(1) '=>' condition('o')~code
  return
::class box
::attribute count class
::attribute count
::attribute tally class
```

```output
3
COUNT
mine 3
say o~tally => 97.1
.box~new~tally = 1 => 97.1
99.905
```

## Names that are taken

r[dir.attribute.duplicate]
A class declares one method of a name, and a second declaration of one of the
attribute's method names in the same class raises a `SYNTAX` error when the
program is read. The names are the getter's name and the setter's name with
the equals sign, so a GET directive and a SET directive of one name do not
collide, while two of the same half, or a plain directive and a GET or SET
one, do. The code is that of the later directive: `99.931` for an attribute,
`99.902` for a [::METHOD](r:dir.method.name) and `99.932` for a
[::CONSTANT](constant.md). An attribute with CLASS and one without it are two
methods of different kinds, so they do not collide.

```rexx
call try '::attribute a', '::attribute A'
call try '::attribute a get', '::attribute a set'
call try '::attribute a get', '::attribute a get'
call try '::attribute a', '::attribute a get'
call try '::attribute a', '::attribute a class'
call try '::method a', '::attribute a'
call try '::method a', '::attribute a set'
call try '::method "a="', '::attribute a'
call try '::attribute a', '::method a'
call try '::constant a', '::attribute a'
call try '::attribute a', '::constant a'
exit
try:
  signal on syntax name e
  src = .array~of('::class k', arg(1), arg(2))
  p = .package~new('spec-dir-attribute-dup', src)
  say arg(1) '/' arg(2) '=> ok'
  return
e: say arg(1) '/' arg(2) '=>' condition('o')~code
  return
```

```output
::attribute a / ::attribute A => 99.931
::attribute a get / ::attribute a set => ok
::attribute a get / ::attribute a get => 99.931
::attribute a / ::attribute a get => 99.931
::attribute a / ::attribute a class => ok
::method a / ::attribute a => 99.931
::method a / ::attribute a set => ok
::method "a=" / ::attribute a => 99.931
::attribute a / ::method a => 99.902
::constant a / ::attribute a => 99.931
::attribute a / ::constant a => 99.932
```

## Methods without code

r[dir.attribute.abstract]
ABSTRACT declares both methods abstract, or the one that GET or SET names.
A message that reaches an abstract method raises `SYNTAX 93.965`, as
[::METHOD](r:dir.method.abstract) specifies, and a subclass that declares the
attribute again, or a method of the name, answers the message instead.

```rexx
call try 'say .base~new~value'
call try '.base~new~value = 1'
call try 'say .base~new~read'
call try '.base~new~read = 1'
say .sub~new~value
o = .sub~new
o~value = 6
say o~value
exit
try:
  signal on syntax name e
  interpret arg(1)
  return
e: say arg(1) '=>' condition('o')~code
  return
::class base
::attribute value abstract
::attribute read get abstract
::class sub subclass base
::attribute value
```

```output
say .base~new~value => 93.965
.base~new~value = 1 => 93.965
say .base~new~read => 93.965
.base~new~read = 1 => 97.1
VALUE
6
```

r[dir.attribute.delegate]
DELEGATE takes the name of an object variable as a symbol, as it does for
[::METHOD](r:dir.method.delegate), and forwards the message of each method
that the directive declares to the object that the variable holds. The getter
forwards `name` and the setter forwards `name=` with its argument, so
assigning to the attribute assigns to the attribute of the same name of the
delegate. A missing operand or a string raises `SYNTAX 20.926` and a symbol
that cannot name a variable raises `99.925`.

```rexx
t = .target~new
f = .front~new(t)
say f~level
f~level = 7
say f~level t~level
say f~part
f~sink = 5
say t~sink
call try '::attribute a delegate'
call try '::attribute a delegate "d"'
call try '::attribute a delegate 1'
exit
try:
  signal on syntax name e
  p = .package~new('spec-dir-attribute-del', .array~of('::class k', arg(1)))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
::class target
::attribute level
::attribute part
::attribute sink
::class front
::method init
  expose d
  use arg d
::attribute level delegate d
::attribute part get delegate d
::attribute sink set delegate d
```

```output
LEVEL
7 7
PART
5
::attribute a delegate => 20.926
::attribute a delegate "d" => 20.926
::attribute a delegate 1 => 99.925
```

r[dir.attribute.external]
EXTERNAL takes the operand that [::METHOD](r:dir.method.external) describes
and declares the method or methods from a native library. A missing operand,
or one that is not a string, raises `SYNTAX 19.905`, and a string that does
not start with the word LIBRARY or has too many words raises `99.917`. A
library that cannot be loaded raises `98.903`, and a library that loads but
has no method entry of the given name raises `90.998`. Both are raised when
the section of the directive ends, as [the rule for external
methods](r:dir.method.external) describes for the faults of other directives,
so that a clause in the section is refused first, as the section rule below
says.

```rexx
call try '::attribute a external'
call try '::attribute a external library'
call try '::attribute a external "library spec_no_such_lib_xyz"'
call try '::attribute a get external "library spec_no_such_lib_xyz"'
call try '::attribute a external "registered spec_no_such_lib_xyz"'
call try '::attribute a external "library a b c"'
call try '::attribute a external "library rexxutil sysfiledelete"'
call try '::attribute a external "library spec_no_such_lib_xyz"', '::frob'
exit
try:
  signal on syntax name e
  src = .array~of('::class k')
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-attribute-ext', src)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
::attribute a external => 19.905
::attribute a external library => 19.905
::attribute a external "library spec_no_such_lib_xyz" => 98.903
::attribute a get external "library spec_no_such_lib_xyz" => 98.903
::attribute a external "registered spec_no_such_lib_xyz" => 99.917
::attribute a external "library a b c" => 99.917
::attribute a external "library rexxutil sysfiledelete" => 90.998
::attribute a external "library spec_no_such_lib_xyz" => 98.903
```

## The section

r[dir.attribute.section]
A directive that declares the generated methods takes no code, and any clause
after it that is not a null clause raises a `SYNTAX` error when the program is
read. The code is `99.937` after a directive with neither GET nor SET,
whatever its kind keyword. After GET or SET it is `99.940` with ABSTRACT,
`99.935` with EXTERNAL and `99.947` with DELEGATE. GET or SET with no kind
keyword is the form whose section is code, so no clause is refused there.
Comments and blank lines between the directive and the next one are null
clauses.

```rexx
call try '::attribute a', 'say 1'
call try '::attribute a abstract', 'say 1'
call try '::attribute a delegate d', 'say 1'
call try '::attribute a get', 'say 1'
call try '::attribute a set', 'say 1'
call try '::attribute a get abstract', 'say 1'
call try '::attribute a set abstract', 'say 1'
call try '::attribute a get external "library spec_no_such_lib_xyz"', 'say 1'
call try '::attribute a get delegate d', 'say 1'
call try '::attribute a', '-- a comment', '', '/* more */', '::attribute b'
exit
try:
  signal on syntax name e
  src = .array~of('::class k')
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-attribute-sec', src)
  say arg(1) '/' arg(2) '=> ok'
  return
e: say arg(1) '/' arg(2) '=>' condition('o')~code
  return
```

```output
::attribute a / say 1 => 99.937
::attribute a abstract / say 1 => 99.937
::attribute a delegate d / say 1 => 99.937
::attribute a get / say 1 => ok
::attribute a set / say 1 => ok
::attribute a get abstract / say 1 => 99.940
::attribute a set abstract / say 1 => 99.940
::attribute a get external "library spec_no_such_lib_xyz" / say 1 => 99.935
::attribute a get delegate d / say 1 => 99.947
::attribute a / -- a comment => ok
```

## Attributes of no class

r[dir.attribute.floating]
An ::ATTRIBUTE before every ::CLASS declares methods of no class, which the
package keeps as method objects of its own, as [floating
methods](r:dir.method.floating) are kept. The methods are in the table that
`.METHODS` names, under the names of the getter and of the setter in upper
case, and a GET or SET directive adds only its own. Duplicate names raise
`SYNTAX 99.931` as they do in a class, and CLASS raises `99.905`.

```rexx
say .methods~items
say .methods~allindexes~sort~makearray~makestring('l', ' ')
.box~define('LEVEL', .methods['LEVEL'])
.box~define('LEVEL=', .methods['LEVEL='])
o = .box~new
say o~level
o~level = 8
say o~level
call try '::attribute a class'
call try '::attribute a', '::attribute a'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-attribute-fl', src)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
::attribute level
::attribute other get
::class box
```

```output
3
LEVEL LEVEL= OTHER
LEVEL
8
::attribute a class => 99.905
::attribute a => 99.931
```
