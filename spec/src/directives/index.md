---
element: "Directives"
kind: chapter
rexxref: "Directives"
---

# Directives

A directive is a clause that starts with two colons. It does not run: it
declares a piece of the program, such as a class, a method or a routine, and
the interpreter acts on it before the program's own instructions run. A
program is a package: a set of declared pieces, and the instructions that come
before the first directive. This chapter specifies how a program is split by
its directives, and the pages that follow specify each directive.

The directives are ANNOTATE, ATTRIBUTE, CLASS, CONSTANT, METHOD, OPTIONS,
REQUIRES, RESOURCE and ROUTINE. The pages of this chapter are:

* [::CLASS](class.md): a class, its superclass, metaclass and inheritance, and
  the order in which the classes of a package come into being.
* [::METHOD](method.md): a method of a class, the options that set its access
  and its kind, and a method that belongs to no class.

The directives that have no page yet are named in plain text where a rule
needs them. The classes that the directives create are objects too, and what
their own methods do is specified in the classes chapter. This chapter says
only what a directive makes and what the options on it mean.

A note on the examples. A directive cannot be run by INTERPRET, and a fault in
one of them ends a program file before it writes anything. Where an example
shows which condition a directive raises, it builds the text of a package with
the `Package` class and catches the condition that building it raises. That
class has its own page in the classes chapter, so the examples use it for one
thing only: `.package~new(name, lines)` translates the lines as a package,
installs it as [the installation rule](r:dir.index.install) says and runs its
instructions before the next clause of the caller.

## What a directive is

r[dir.index.directive]
A clause is a directive when its first token is `::`, which may be followed
by blanks or comments before the keyword. The keyword is not case sensitive.
A directive begins its clause at the start of a line or after a semicolon;
after a label on the same line the `::` is read as part of an expression and
raises `SYNTAX 35.1`. A keyword that names no directive raises
`SYNTAX 99.916`, and `::` followed by nothing that is a symbol raises
`SYNTAX 20.916`. The error is found when the program is read, so that a
program file with such a clause writes nothing and exits with 256 minus the
error number, 157 for 99.916, as [the reading rule](r:lex.index.read-first)
describes for any fault of the text. A directive cannot be part of the text of
[INTERPRET](r:instr.interpret.labels), which refuses it.

```rexx
call try '::class k1'
call try ':: /* a comment */ Class k2'
call try '::CLASS k3'
call try '::frob k4'
call try '::'
call try '::"class" k5'
call try 'lab: ::class k6'
exit
try:
  signal on syntax name e
  p = .package~new('spec-dir-index-kw', .array~of(arg(1)))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
::class k1 => ok
:: /* a comment */ Class k2 => ok
::CLASS k3 => ok
::frob k4 => 99.916
:: => 20.916
::"class" k5 => 20.916
lab: ::class k6 => 35.1
```

```rexx
say 'before'
::frob k
```

```output rc=157
```

r[dir.index.end]
A directive ends where a clause ends: at the end of the line or at a
semicolon, and a comma at the end of a line continues it on the next line, as
[the clause rules](r:lex.clauses.end) say for any clause. A comment after the
directive is part of the clause's white space. What follows the semicolon is
the next clause, so a directive and the first clause of its body can share a
line.

```rexx
src = .array~of('::class k', '::method one; return 1; ::method two; return 2')
p = .package~new('spec-dir-index-end', src)
src = .array~of('say .box~new~m', '::class box,', '  public -- a comment', -
  '::method m', '  return 3')
p = .package~new('spec-dir-index-end2', src)
```

```output
3
```

## The prolog and the sections

r[dir.index.prolog]
The clauses before the first directive are the prolog, the main code of the
package. It runs once the package is installed, and its end is the end of the
program: the interpreter never goes on into the code that follows the first
directive, which is reached only through the routine or method that it
declares. A file without directives is all prolog, and a file whose first
clause is a directive has no prolog. An `exit` or `return` in the prolog ends
the program there, and the directives have been installed all the same.

```rexx
say 'start'
say .box~new~m
say 'end of the prolog'
::class box
::method m
  say 'running m'
  return 'result of m'
say 'this clause is part of m'
```

```output
start
running m
result of m
end of the prolog
```

```rexx
say 'start'
exit
::class box
::method activate class
  say 'activated'
```

```output
activated
start
```

r[dir.index.section]
The clauses after a directive, up to the next directive or the end of the
program, are that directive's section. The section is the body of the method
or routine that the directive declares, and the directive clause itself is not
part of it. A directive that declares no code, such as ::CLASS, allows only
comments and empty lines between it and the next directive. Any other clause
there raises a `SYNTAX` error found when the program is read: `99.916` after
`::class`. The
pages of the directives that take a body or are refused one say which subcode
their own case raises.

```rexx
call try '::class k', '-- a comment', '', '/* another */', '::method m'
call try '::class k', 'say 1', '::method m'
call try '::class k', 'x = 2'
call try '::class k', '::method m', 'say 1', 'x = 2'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-index-sec', src)
  say src~makestring('l', ' / ') '=> ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
```

```output
::class k / -- a comment /  / /* another */ / ::method m => ok
::class k / say 1 / ::method m => 99.916
::class k / x = 2 => 99.916
::class k / ::method m / say 1 / x = 2 => ok
```

r[dir.index.unit]
Each section, and the prolog, is a code unit of its own. Its labels are known
only inside it, its variables are not shared with another unit, and a
construct that it opens must end inside it: a DO, IF or SELECT that is still
open at the next directive raises `SYNTAX 14.1`, `14.3` or `14.2`, found when
the program is read. A routine or method that needs a value of another unit
receives it as an argument or through an object variable.

```rexx
x = 'prolog variable'
say r1()
call try 'do', '::class k'
call try 'if 1 then', '::class k'
call try 'select', '::class k'
exit
try:
  signal on syntax name e
  src = .array~new
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-index-unit', src)
  say 'ok'
  return
e: say src~makestring('l', ' / ') '=>' condition('o')~code
  return
::routine r1
  signal on syntax name lost
  say symbol('x')
  call own
  call prolog
  return 'not reached'
own: say 'own label'
  return
lost: return 'r1 caught' condition('o')~code
```

```output
LIT
own label
r1 caught 43.1
do / ::class k => 14.1
if 1 then / ::class k => 14.3
select / ::class k => 14.2
```

## Installation

r[dir.index.install]
The interpreter installs a package before it runs the prolog, in four steps.
First the ::REQUIRES directives are processed in text order wherever they
stand, except that [the library rule](r:dir.requires.library) processes the
LIBRARY ones before the others, and each required package is installed and
runs its own prolog before the next step. Then the package's routines become
known and every class is created, in the order that [the class
rule](r:dir.class.forward) gives, and each new class object is sent an INIT
message. When all classes exist, each is sent an ACTIVATE message in the order
of creation. Only then does the prolog run, so a class method that ACTIVATE
runs sees every class and routine of the package, and the prolog sees every
class already activated.
A class is reachable by its name, as the environment symbol that is a period
followed by the name, once its INIT message has returned. During the INIT of
a class the classes created before it are reachable, and the class itself and
the classes created later are not, so that their symbols have the symbol's own
text as value. The routines of the package are known from the start. Every
class is reachable by the time the first ACTIVATE message is sent.

```rexx
f = 'spec-dir-index-lib.tmp'
call sysfiledelete f
call lineout f, 'say "lib prolog"'
call lineout f, '::class libc public'
call lineout f, '::method activate class'
call lineout f, '  say "lib activate"'
call lineout f
src = .array~of('say "prolog"', -
  '::class one', -
  '::method init class', -
  '  say "init one"', -
  '::method activate class', -
  '  say "activate one, routine says" r()', -
  '::requires "'f'"', -
  '::class two', -
  '::method init class', -
  '  say "init two"', -
  '::method activate class', -
  '  say "activate two"', -
  '::routine r', -
  '  return "r"')
p = .package~new('spec-dir-index-main', src)
call sysfiledelete f
```

```output
lib activate
lib prolog
init one
init two
activate one, routine says r
activate two
prolog
```

```rexx
say 'prolog'
::class a
::method init class
  say 'init a sees' .b r()
  signal on syntax name e
  say 'init a sees' .a~id
  return
e: say 'init a self' condition('o')~code
::class b
::method init class
  say 'init b sees' .a~id
  signal on syntax name e
  say 'init b sees' .b~id
  return
e: say 'init b self' condition('o')~code
::method activate class
  say 'activate b sees' .a~id .b~id
::routine r
  return 'r'
```

```output defect=#112
init a sees .B r
init a self 97.1
init b sees A
init b self 97.1
activate b sees A B
prolog
```

r[dir.index.failure]
A fault found while the package is read stops everything: nothing is
installed and no prolog runs. A fault raised while the package is installed,
such as a class that cannot be found or a required file that does not exist,
stops installation at that step, so that no class activates after it and the
prolog does not run. The condition is `SYNTAX` in both cases, and a program
file ends with 256 minus the error number as its exit status, 158 for
`98.909`. The classes created before such a fault have received their INIT
message, as the next example shows, and have received no ACTIVATE message.
A class that names itself, directly or through the classes it names, is an
exception: [the order rule](r:dir.class.forward) finds it before any class is
created, so no INIT message has been sent.

```rexx
call try '::class k public private'
call try '::class k', '::class l subclass nosuchsuperclass'
call try '::requires "spec-dir-index-nosuchfile-xyz.rex"'
exit
try:
  signal on syntax name e
  src = .array~of('say "prolog"')
  do i = 1 to arg()
    src~append(arg(i))
  end
  p = .package~new('spec-dir-index-fail', src)
  say 'ok'
  return
e: say 'caught' condition('o')~code
  return
```

```output
caught 25.901
caught 98.909
caught 43.901
```

```rexx
say 'prolog'
::class k
::method init class
  say 'init k'
::method activate class
  say 'activate k'
::class l subclass nosuchsuperclass
```

```output rc=158
init k
```

## Names

r[dir.index.names]
A name that a directive takes is a symbol or a string. A symbol names the
element in upper case, and a string names it as written, so `::class Box`
creates the class `BOX` and `::class "Box"` creates the class `Box`. A lookup
of a class or a method by name ignores case, whatever the spelling in the
directive, and two directives that name one class, or one method of a class,
in different cases declare it twice. The pages of the directives say what each
does with a duplicate.

```rexx
say .Widget~id .WIDGET~id .gadget~id .Gadget~id
::class Widget
::class "Gadget"
```

```output
WIDGET WIDGET Gadget Gadget
```
