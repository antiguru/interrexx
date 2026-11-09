---
element: "::CONSTANT"
kind: directive
syntax: "::CONSTANT name [value | (expression)]"
rexxref: "Directives > ::CONSTANT"
deviations: [69]
---

# ::CONSTANT

::CONSTANT declares a name that answers one value for the class and for each
of its instances. The value is a literal, a symbol taken as text or an
expression in parentheses that is evaluated once, when the package is
installed. The directive has no code section and no setter. This page
specifies what the directive declares, the forms of the value, when an
expression runs and what it can see, and the names that a constant takes. What
a method object does once it exists belongs to the classes chapter.

## What a constant is

r[dir.constant.create]
::CONSTANT declares one method of the class that the most recent
[::CLASS](class.md) declared, and the method is both an instance method and a
class method, so the class and every instance of it, subclasses included,
answer the name with the value. A subclass that declares a method of the name
answers it instead. The method takes no argument, and an argument raises
`SYNTAX 93.902`. There is no method that assigns, so a message assignment
raises `SYNTAX 97.1`. A value left out makes the constant answer its own name,
in upper case for a symbol and as written for a string. A constant needs no
class before it, as the rule for [constants of no
class](r:dir.constant.floating) says.

```rexx
say .shape~sides .shape~new~sides
say .shape~kind .shape~new~kind
say .shape~"Quiet" .shape~new~quiet
say .square~sides .square~new~sides
say .triangle~sides
call try 'say .shape~sides(1)'
call try '.shape~new~sides = 4'
call try '.shape~sides = 4'
exit
try:
  signal on syntax name e
  interpret arg(1)
  return
e: say arg(1) '=>' condition('o')~code
  return
::class shape
::constant sides 0
::constant kind
::constant "Quiet"
::class square subclass shape
::class triangle subclass shape
::method sides class
  return 3
```

```output
0 0
KIND KIND
Quiet Quiet
0 0
3
say .shape~sides(1) => 93.902
.shape~new~sides = 4 => 97.1
.shape~sides = 4 => 97.1
```

r[dir.constant.name]
The name of a constant is a symbol or a string, with the rules that
[::METHOD](r:dir.method.name) gives for the name of a method, and
[the names rule](r:dir.index.names) gives for its spelling. A missing
name raises `SYNTAX 19.915`. A class declares one method of a name, and the
constant is both an instance and a class method, so a second declaration of
its name in the same class raises a `SYNTAX` error when the program is read,
also when the other one is a [::METHOD](r:dir.method.name) of the class kind
or an [::ATTRIBUTE](r:dir.attribute.duplicate). The code is that of the later
directive: `99.932` for a constant, `99.902` for a ::METHOD and `99.931` for
an ::ATTRIBUTE. Two classes may each have a constant of one name.

```rexx
say .k~Lower .k~"LOWER" .k~"+"
call try '::constant a', '::constant A'
call try '::constant a', '::method a'
call try '::constant a', '::method a class'
call try '::constant a', '::attribute a'
call try '::method a', '::constant a'
call try '::method a class', '::constant a'
call try '::attribute a class', '::constant a'
call try '::constant a', '::class l', '::constant a'
call try '::constant'
exit
try:
  signal on syntax name e
  src = .array~new
  src~append('::class k')
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-constant-name', src)
  say arg(1) '/' arg(2) '=> ok'
  return
e: say arg(1) '/' arg(2) '=>' condition('o')~code
  return
::class k
::constant "Lower" Abc
::constant "+" 1
```

```output
ABC ABC 1
::constant a / ::constant A => 99.932
::constant a / ::method a => 99.902
::constant a / ::method a class => 99.902
::constant a / ::attribute a => 99.931
::method a / ::constant a => 99.932
::method a class / ::constant a => 99.932
::attribute a class / ::constant a => 99.932
::constant a / ::class l => ok
::constant /  => 19.915
```

## The value

r[dir.constant.value]
A value that is not in parentheses is one token. A string is the value as
written, with the quotes removed. A symbol is the value in upper case and it
is never looked up: `.nil` is the text `.NIL` and `Hello.World` is
`HELLO.WORLD`, whatever variable or environment symbol the name could be. A
number is not normalized, so `007` stays `007` and `1e3` is `1E3` as the
symbol it is. A sign, `+` or `-`, may come before a number, with blanks
between or not, and the value is the sign and the number joined, so `- 7` is
`-7` and `+5` is `+5`. A sign followed by anything other than a number, or by
nothing before a semicolon, raises `SYNTAX 19.916`, and a second token raises
`21.913`. These errors are found when the program is read. A sign that is the
last token on its line is a [continuation](r:lex.clauses.continuation) and not
a sign.

```rexx
say .k~lit .k~sym .k~nil .k~dotted
say .k~num .k~exp .k~neg .k~plus .k~spaced
call try '::constant a - b'
call try '::constant a +x'
call try '::constant a - -1'
call try '::constant a - "7"'
call try '::constant a - ;'
call try '::constant a -'
call try '::constant a 1 2'
call try '::constant a "x" "y"'
call try '::constant a (1) (2)'
exit
try:
  signal on syntax name e
  p = .package~new('spec-dir-constant-val', .array~of('::class k', arg(1)))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
::class k
::constant lit 'x  y'
::constant sym Hello
::constant nil .nil
::constant dotted Hello.World
::constant num 007
::constant exp 1e3
::constant neg -273.15
::constant plus +5
::constant spaced - 7
```

```output
x  y HELLO .NIL HELLO.WORLD
007 1E3 -273.15 +5 -7
::constant a - b => 19.916
::constant a +x => 19.916
::constant a - -1 => 19.916
::constant a - "7" => 19.916
::constant a - ; => 19.916
::constant a - => ok
::constant a 1 2 => 21.913
::constant a "x" "y" => 21.913
::constant a (1) (2) => 21.913
```

## Expressions

r[dir.constant.expression]
A value in parentheses is an expression, which the interpreter evaluates once
and keeps, so each message answers the same value that the evaluation
produced. The evaluation is part of [installation](r:dir.index.install): it
runs after every class of the package has been created and has received its
INIT message, and before any class receives ACTIVATE, so before the prolog
too. It runs the constants of the classes in the order in which the classes
were created, and in text order within a class. In the expression `SELF` is
the class object, whichever of the class and its instances later receives the
message, and the code is that of a method named `::CONSTANT` whose line is the
directive's. An error in the expression raises its own condition, and the
installation stops at that constant as [the failure rule](r:dir.index.failure)
says, so that no class activates and the prolog does not run. A syntax error
in the expression is found when the program is read.

```rexx
say 'prolog'
say .k~a .k~b
say .k~who
say .k~name .k~line
call try 'say "prolog"', '::class k', '::constant a (1/0)'
call try 'say "prolog"', '::class k', '::constant a (undefined_xyz())'
call try '::class k', '::constant a (1 +)'
call try '::class k', '::constant a (1))'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-constant-expr', src)
  say 'ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
::routine tr
  say 'evaluating' arg(1)
  return arg(1)
::class k
::method activate class
  say 'activate'
::constant a (tr('a'))
::constant b (tr('b') .k~a self~a)
::constant who (self~id)
::constant name (.context~name)
::constant line (.context~line)
```

```output
evaluating a
evaluating b
activate
prolog
a b a a
K
::CONSTANT 31
say "prolog" / ::class k / ::constant a (1/0) => 42.3
say "prolog" / ::class k / ::constant a (undefined_xyz()) => 43.1
::class k / ::constant a (1 +) => 35.1
::class k / ::constant a (1)) => 21.913
```

r[dir.constant.order]
An expression that reads another constant sees its value only when that
constant is already evaluated. A constant whose value is a literal, a symbol
or a number is there from the start, so an expression may read it wherever it
is declared, in its own class or another. An expression constant is evaluated
in the order of [the expression rule](r:dir.constant.expression), so an
expression that reads one that comes after it, itself included, raises
`SYNTAX 97.4` during installation. The classes are created in the order that
[the class rule](r:dir.class.forward) gives, which a forward reference to a
superclass changes from the text order.

```rexx
call try '::class k', '::constant a (self~b)', '::constant b 1'
call try '::class k', '::constant a (self~b)', '::constant b (2)'
call try '::class k', '::constant a (self~a)'
call try '::class k', '::constant a (.l~x)', '::class l', '::constant x 5'
call try '::class k', '::constant a (.l~x)', '::class l', '::constant x (5)'
call try '::class k', '::constant a (1)', '::constant b (self~a + 1)'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-constant-order', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::class k / ::constant a (self~b) / ::constant b 1 => ok
::class k / ::constant a (self~b) / ::constant b (2) => 97.4
::class k / ::constant a (self~a) => 97.4
::class k / ::constant a (.l~x) / ::class l / ::constant x 5 => ok
::class k / ::constant a (.l~x) / ::class l / ::constant x (5) => 97.4
::class k / ::constant a (1) / ::constant b (self~a + 1) => ok
```

```rexx
say 'prolog'
exit
::routine tr
  say arg(1)
  return arg(1)
::class a subclass b
::method init class
  say 'init a'
::constant x (tr('x of a'))
::class b
::method init class
  say 'init b'
::constant y (tr('y of b'))
```

```output
init b
init a
y of b
x of a
prolog
```

## No code section

r[dir.constant.section]
::CONSTANT declares no code, and a clause after it that is not a null clause
raises `SYNTAX 99.938` when the program is read, whatever the form of its
value. Comments and blank lines between it and the next directive are null
clauses.

```rexx
call try '::constant a', 'say 1'
call try '::constant a 1', 'say 1'
call try '::constant a (1)', 'x = 2'
call try '::constant a (1)', '-- a comment', '', '/* more */', '::constant b'
exit
try:
  signal on syntax name e
  src = .array~new
  src~append('::class k')
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-constant-sec', src)
  say arg(1) '/' arg(2) '=> ok'
  return
e: say arg(1) '/' arg(2) '=>' condition('o')~code
  return
```

```output
::constant a / say 1 => 99.938
::constant a 1 / say 1 => 99.938
::constant a (1) / x = 2 => 99.938
::constant a (1) / -- a comment => ok
```

## Constants of no class

r[dir.constant.floating]
A ::CONSTANT before every ::CLASS declares a method of no class, which the
package keeps in the table that `.METHODS` names, as [floating
methods](r:dir.method.floating) are kept, under the name in upper case. Its
value is limited to the forms that are not expressions: a floating constant
whose value is in parentheses raises `SYNTAX 99.906` when the program is read.
A name declared twice raises `99.932`, as it does in a class.

```rexx
say .methods~items
say .methods~allindexes~sort~makearray~makestring('l', ' ')
.box~define('LIMIT', .methods['LIMIT'])
say .box~new~limit
call try '::constant a (1)'
call try '::constant a 1', '::constant A 2'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-constant-fl', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
::constant limit 99
::constant "Free"
::class box
```

```output
2
FREE LIMIT
99
::constant a (1) => 99.906
::constant a 1 / ::constant A 2 => 99.932
```

## Deviations

r[dir.constant.deviation.69]
The code of an expression constant is a method of its own, and the source that
it reports, through `.context~executable~source`, is the text of its own
directive. In ooRexx the expression constants of a class are joined into one
method, so the source of the one that is running is not its own directive.

```rexx
say .k~a
say .k~b
::class k
::constant a (.context~executable~source~makestring('l', '|'))
::constant b (.context~executable~source~makestring('l', '|'))
```

```output
::constant a (.context~executable~source~makestring('l', '|'))
::constant b (.context~executable~source~makestring('l', '|'))
```
