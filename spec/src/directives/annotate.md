---
element: "::ANNOTATE"
kind: directive
syntax: "::ANNOTATE PACKAGE | CLASS name | METHOD name | ATTRIBUTE name | CONSTANT name | ROUTINE name {annotation value}..."
rexxref: "Directives > ::ANNOTATE"
---

# ::ANNOTATE

::ANNOTATE attaches notes to the parts of a package. A note is an annotation,
a name with a text as its value, and a program that holds the object of a
package, class, method or routine can read the annotations that were declared
for it. The interpreter does nothing with them: no annotation name has a
meaning of its own, so they carry information for tools and for the program
itself. This page specifies what the directive may name, the form of the
names and values, and which annotations a program finds. The methods that
read and change annotations belong to the classes chapter, and the examples
use only `ANNOTATION` and `ANNOTATIONS`, and the methods that reach an
object.

## The form

r[dir.annotate.syntax]
After the directive word comes a kind word, in any case: PACKAGE, CLASS,
METHOD, ATTRIBUTE, CONSTANT or ROUTINE. Every kind but PACKAGE then has a
target name, a symbol or a string, and the rest of the clause is a list of
annotations, each a name and a value. The list may be empty. A missing kind
raises `SYNTAX 20.924`, a word that is no kind `25.928`, and a missing target
name `19.925`. The directive takes no code: a clause after it that is not a
null clause raises `SYNTAX 99.916`, as [the section rule](r:dir.index.section)
says, so that the directive ends the code of the method or routine before it.
These faults are found when the program is read, so [as the read
rule says](r:lex.index.read-first) a program file with one writes nothing and
exits with 256 minus the error number, 236 for `20.924`.

```rexx
call try '::annotate'
call try '::annotate frob x a b'
call try '::annotate class'
call try '::annotate package'
call try '::annotate package a b', '', '-- a comment', '::class k'
call try '::class k', '::annotate CLASS K a b; ::annotate class k c d'
call try '::class k', '::method m', '  return 1', '::annotate method m a b'
call try '::class k', '::method m', '::annotate method m a b', '  return 1'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-annotate-syntax', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::annotate => 20.924
::annotate frob x a b => 25.928
::annotate class => 19.925
::annotate package => ok
::annotate package a b /  / -- a comment / ::class k => ok
::class k / ::annotate CLASS K a b; ::annotate class k c d => ok
::class k / ::method m /   return 1 / ::annotate method m a b => ok
::class k / ::method m / ::annotate method m a b /   return 1 => 99.916
```

```rexx
say 'before'
::annotate
```

```output rc=236
```

r[dir.annotate.value]
Each annotation is a name followed by a value. The name is a symbol, which is
taken in upper case and never looked up as a variable. A name that is not a
symbol, a string or an operator for instance, raises `SYNTAX 20.919`. The
value is a string, a symbol or a number with an optional sign, and is kept as
text: a string as written and a symbol in upper case, which includes a number
that is written as a symbol such as `1e3`. A sign before a number is kept and
joined to it, also when a blank stands between them, and the number is kept as
written. A symbol may hold periods, as `.abc` and `a.b` do. A value in
parentheses raises `19.923`, and so does a sign before anything but a
number: before a sign, a string or a symbol that is no number. A list with an
unpaired word raises `19.924`.
A name given twice, in one directive or in several, keeps the value that comes
last in text order, and the other names are kept. The names and values do not
depend on one another or on any variable of the program.

```rexx
p = .context~package
a = p~annotations
do name over a~allindexes~sort
  say name '=' '['a[name]']'
end
::annotate package maint "Q. Rivera" year 2024 neg -5 plus +3.5 spaced - 7
::annotate package sym abc env .abc dotted a.b num 007 exp 1e3 empty ""
::annotate package year 2025 Maint 'it''s'
```

```output
DOTTED = [A.B]
EMPTY = []
ENV = [.ABC]
EXP = [1E3]
MAINT = [it's]
NEG = [-5]
NUM = [007]
PLUS = [+3.5]
SPACED = [-7]
SYM = [ABC]
YEAR = [2025]
```

```rexx
call try '::annotate package "a" b'
call try '::annotate package a b+c'
call try '::annotate package a (b)'
call try '::annotate package a ++5'
call try '::annotate package a -b'
call try '::annotate package a +"x"'
call try '::annotate package a'
call try '::annotate package a b c'
call try '::annotate package a b c d'
call try '::annotate package a 5 b -3 c "x" d y'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-annotate-value', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::annotate package "a" b => 20.919
::annotate package a b+c => 20.919
::annotate package a (b) => 19.923
::annotate package a ++5 => 19.923
::annotate package a -b => 19.923
::annotate package a +"x" => 19.923
::annotate package a => 19.924
::annotate package a b c => 19.924
::annotate package a b c d => ok
::annotate package a 5 b -3 c "x" d y => ok
```

## What is annotated

r[dir.annotate.target]
The target of a CLASS, ROUTINE, METHOD, ATTRIBUTE or CONSTANT annotation is
an element that an earlier directive of the same package declared, found by
its name with case ignored, whether the target was written as a symbol or as
a string. A class or a routine is found among those declared before the
directive, however many directives lie between. A method, an attribute or a
constant is found only among those declared in the section of the most recent
::CLASS, or, before any class, among the methods that no class holds. A
target that is not found, which includes one that is declared later, in
another class, or by another package that was required or supplied by the
interpreter, raises `SYNTAX 99.945`. The directive with no annotation is
checked in the same way. The error is found when the program is read.

```rexx
call try '::annotate class k a b'
call try '::annotate class k'
call try '::class k', '::annotate class k a b'
call try '::class k', '::annotate class "K" a b'
call try '::annotate class k a b', '::class k'
call try '::class k', '::class l', '::annotate class k a b'
call try '::class k', '::annotate class k'
call try '::class k', '::annotate class .k a b'
call try '::annotate class object a b'
call try '::routine r', '::class k', '::annotate routine R a b'
call try '::annotate routine r a b', '::routine r'
call try '::class k', '::method m', '::annotate method M a b'
call try '::class k', '::method m', '::class l', '::annotate method m a b'
call try '::method m', '::annotate method m a b'
call try '::method m', '::class k', '::annotate method m a b'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-annotate-target', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::annotate class k a b => 99.945
::annotate class k => 99.945
::class k / ::annotate class k a b => ok
::class k / ::annotate class "K" a b => ok
::annotate class k a b / ::class k => 99.945
::class k / ::class l / ::annotate class k a b => ok
::class k / ::annotate class k => ok
::class k / ::annotate class .k a b => 99.945
::annotate class object a b => 99.945
::routine r / ::class k / ::annotate routine R a b => ok
::annotate routine r a b / ::routine r => 99.945
::class k / ::method m / ::annotate method M a b => ok
::class k / ::method m / ::class l / ::annotate method m a b => 99.945
::method m / ::annotate method m a b => ok
::method m / ::class k / ::annotate method m a b => 99.945
```

r[dir.annotate.kind]
The kind word says which declarations the name is looked for in. CLASS finds
a class, ROUTINE a routine and PACKAGE needs no name and annotates the
package itself. METHOD finds a method that a ::METHOD declared, and also the
method of an attribute or a constant, whose names are the attribute's name,
with an equals sign for a setter, and the constant's name. ATTRIBUTE finds an
attribute that ::ATTRIBUTE or the ATTRIBUTE option of ::METHOD declared and
no other method, and the annotation is attached to both of the methods of the
attribute that exist. CONSTANT finds a constant. An element is not found by
the wrong kind word, and the error is `SYNTAX 99.945`. A name that holds an
equals sign has to be written as a string. A method that was declared with the
CLASS option, and an attribute that was, are found in the same way as the
others. When a class has an instance method and a class method of one name,
the annotation goes to the instance method, whichever of the two directives
comes first, and the class method has none. A class method is annotated when
no instance method has the name.

```rexx
call try '::class k', '::attribute m', '::annotate attribute m a b'
call try '::class k', '::attribute m', '::annotate method m a b'
call try '::class k', '::attribute m', '::annotate method "m=" a b'
call try '::class k', '::attribute m', '::annotate method m= a b'
call try '::class k', '::method m attribute', '::annotate attribute m a b'
call try '::class k', '::method m', '::annotate attribute m a b'
call try '::class k', '::constant m 1', '::annotate constant m a b'
call try '::class k', '::constant m 1', '::annotate method m a b'
call try '::class k', '::constant m 1', '::annotate attribute m a b'
call try '::class k', '::method m', '::annotate constant m a b'
call try '::routine r', '::annotate method r a b'
call try '::routine r', '::annotate class r a b'
call try '::routine r', '::annotate package r a b'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-annotate-kind', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::class k / ::attribute m / ::annotate attribute m a b => ok
::class k / ::attribute m / ::annotate method m a b => ok
::class k / ::attribute m / ::annotate method "m=" a b => ok
::class k / ::attribute m / ::annotate method m= a b => 20.919
::class k / ::method m attribute / ::annotate attribute m a b => ok
::class k / ::method m / ::annotate attribute m a b => 99.945
::class k / ::constant m 1 / ::annotate constant m a b => ok
::class k / ::constant m 1 / ::annotate method m a b => ok
::class k / ::constant m 1 / ::annotate attribute m a b => 99.945
::class k / ::method m / ::annotate constant m a b => 99.945
::routine r / ::annotate method r a b => 99.945
::routine r / ::annotate class r a b => 99.945
::routine r / ::annotate package r a b => 19.924
```

```rexx
c = .k
say 'instance m' c~method('M')~annotation('V')
say 'class m' c~instancemethod('M')~annotation('V')
say 'class n' c~instancemethod('N')~annotation('V')
say 'class attribute' c~instancemethod('CA')~annotation('V') -
  c~instancemethod('CA=')~annotation('V')
::class k
::method m
::method m class
::method n class
::attribute ca class
::annotate method m v one
::annotate method n v two
::annotate attribute ca v three
```

```output
instance m ONE
class m The NIL object
class n TWO
class attribute THREE THREE
```

## Reading the annotations

r[dir.annotate.read]
The annotations of an element are kept with its object. The `ANNOTATION`
message of the object of a [package](r:class.package.annotation),
[class](r:class.class.annotation), [method](r:class.method.annotation) or
[routine](r:class.routine.annotation), with an annotation name, returns the
value as a string, or `.nil` when the element has no annotation of that name,
and the name is looked up in upper case. The
[`ANNOTATIONS`](r:class.package.annotations) message returns a table of all of
them indexed by name. The annotation of an attribute is found on both of its
methods, and that of a constant on the method of the constant. An annotation
is not inherited: a class that names an annotated class as its superclass has
none of its annotations, and the same name on a class and on its method are
two separate annotations. A copy of an annotated routine or method keeps its
annotations.

```rexx
p = .context~package
say 'package' p~annotation('maint') p~annotations~items
say 'class' .k~annotation('X') .k~annotation('y') .k~annotations~items
say 'sub' .sub~annotations~items
m = .k~method('M')
say 'method' m~annotation('v') m~annotation('x') m~annotations~items
say 'copy' m~copy~annotation('V')
say 'attribute' .k~method('AT')~annotation('P') -
  .k~method('AT=')~annotation('P')
say 'constant' .k~method('C')~annotation('P')
r = p~findroutine('R')
say 'routine' r~annotation('p') r~copy~annotation('P') r~annotations~items
say 'none' (p~annotation('nothing') == .nil)
::annotate package maint "Q. Rivera"
::class k
::method m
::attribute at
::constant c 5
::annotate class k x 1
::annotate method m v kv x mx
::annotate attribute at p ap
::annotate constant c p cp
::annotate class k y 3
::class sub subclass k
::routine r
::annotate routine r p rp
```

```output
package Q. Rivera 1
class 1 3 2
sub 0
method KV MX 2
copy KV
attribute AP AP
constant CP
routine RP RP 1
none 1
```
