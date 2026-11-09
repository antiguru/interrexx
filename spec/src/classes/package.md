---
element: "Package"
kind: class
rexxref: "Builtin Classes > Fundamental Classes > Package Class; Builtin Classes > Fundamental Classes > Package Class > new (Class Method); Builtin Classes > Fundamental Classes > Package Class > defaultOptions (Class Method); Builtin Classes > Fundamental Classes > Package Class > addClass; Builtin Classes > Fundamental Classes > Package Class > addPackage; Builtin Classes > Fundamental Classes > Package Class > addPublicClass; Builtin Classes > Fundamental Classes > Package Class > addPublicRoutine; Builtin Classes > Fundamental Classes > Package Class > addRoutine; Builtin Classes > Fundamental Classes > Package Class > annotation; Builtin Classes > Fundamental Classes > Package Class > annotations; Builtin Classes > Fundamental Classes > Package Class > classes; Builtin Classes > Fundamental Classes > Package Class > definedMethods; Builtin Classes > Fundamental Classes > Package Class > digits; Builtin Classes > Fundamental Classes > Package Class > findClass; Builtin Classes > Fundamental Classes > Package Class > findNamespace; Builtin Classes > Fundamental Classes > Package Class > findProgram; Builtin Classes > Fundamental Classes > Package Class > findPublicClass; Builtin Classes > Fundamental Classes > Package Class > findPublicRoutine; Builtin Classes > Fundamental Classes > Package Class > findRoutine; Builtin Classes > Fundamental Classes > Package Class > form; Builtin Classes > Fundamental Classes > Package Class > fuzz; Builtin Classes > Fundamental Classes > Package Class > importedClasses; Builtin Classes > Fundamental Classes > Package Class > importedPackages; Builtin Classes > Fundamental Classes > Package Class > importedRoutines; Builtin Classes > Fundamental Classes > Package Class > loadLibrary; Builtin Classes > Fundamental Classes > Package Class > loadPackage; Builtin Classes > Fundamental Classes > Package Class > local; Builtin Classes > Fundamental Classes > Package Class > name; Builtin Classes > Fundamental Classes > Package Class > namespaces; Builtin Classes > Fundamental Classes > Package Class > options; Builtin Classes > Fundamental Classes > Package Class > prolog; Builtin Classes > Fundamental Classes > Package Class > publicClasses; Builtin Classes > Fundamental Classes > Package Class > publicRoutines; Builtin Classes > Fundamental Classes > Package Class > resource; Builtin Classes > Fundamental Classes > Package Class > resources; Builtin Classes > Fundamental Classes > Package Class > routines; Builtin Classes > Fundamental Classes > Package Class > setSecurityManager; Builtin Classes > Fundamental Classes > Package Class > source; Builtin Classes > Fundamental Classes > Package Class > sourceLine; Builtin Classes > Fundamental Classes > Package Class > sourceSize; Builtin Classes > Fundamental Classes > Package Class > trace"
deviations: [41, 109]
---

# Package

A `Package` object is a unit of code that the interpreter loads: the program
that a user runs, a file that `::REQUIRES` loads, a source that `new` is
given. It holds the source text, the classes, routines and unattached methods
that its directives declare, the resources and annotations, the packages it
imports and the namespaces it names them by, the initial settings of the
numeric and condition options and of tracing, and the code that comes before
its first directive, which is called its prolog. The classes and routines that
its code can name are found through the package.

The page specifies the class methods that make a package and set the default
options of the packages that are made afterwards, and the methods that ask a
package what it holds, find a class, a routine or a namespace in it, add to
it, and read or change its options. The directives that fill a package are
specified by [`::CLASS`](../directives/class.md),
[`::ROUTINE`](../directives/routine.md),
[`::METHOD`](../directives/method.md),
[`::REQUIRES`](../directives/requires.md),
[`::OPTIONS`](../directives/options.md),
[`::RESOURCE`](../directives/resource.md) and
[`::ANNOTATE`](../directives/annotate.md), and the page links them instead of
repeating them. A package's source is the source of a method or routine made
in the way [the Method page](method.md) describes, and its rules for the
source apply here where this page says so. The search for a file or a library
that `new`, `loadPackage`, `loadLibrary` and `findProgram` start is specified
in the environment chapter; the examples here name files that they create
themselves, or a library that every installation has. The message a package
sends to a security manager is specified on [the Routine page](routine.md),
which also specifies the checkpoints.

The examples that show an error use a routine `try` that sends the message in
its argument and prints the answer, or the error code when the message raises
one. A message that answers `.nil` prints `The NIL object`.

## The class

r[class.package.creation]
A package is made by `new`, by the interpreter for the program that runs, by
`::REQUIRES` and by `loadPackage`, and the library classes belong to a package
named `REXX`. A package made by `new` is an ordinary object otherwise: it has
the default name `a Package`, and the class `Package` has the superclass
`Object` and no mixin class. The package of the library, which
`.object~package` answers, holds the classes of the library, is named `REXX`,
has no source, and is found by `findNamespace` under that name from any
package.

```rexx
p = .package~new('first', 'return 5')
say p~class~id p~defaultName p~name p~source[1]
say .package~superclass~id .package~superclasses~items
rp = .object~package
say rp~name .array~package~name (rp == .string~package)
say rp~source~items rp~sourceSize rp~publicClasses~hasIndex('STRING')
say (p~findNamespace('REXX') == rp) (p~findNamespace('rexx') == rp)
```

```output
Package a Package first return 5
Object 1
REXX REXX 1
0 0 1
1 1
```

## Class methods

### new

r[class.package.new]
`new` makes a package from a name and, optionally, a source and a context. The
name is required (`SYNTAX 88.901`), and is a string or the `MAKESTRING` answer
of its argument: an object with no `MAKESTRING` method, `.nil` among them,
raises `88.909`. With a source, the name is only the name of the package, and
the package is always a new object. With no source, the name is the name of a
file, which `new` looks for and reads, as
[the file rule](r:class.package.new.file) says, and there is no context: the
arguments from the third on go to `init`. The context is the third argument.
It is a `Method`, a `Routine` or a `Package` object, or omitted; any other
value, `.nil` and `PROGRAMSCOPE` included, raises `SYNTAX 93.953`. The checks
run in this order: the name, the context, the source (a source that is `.nil`
raises `93.961`), and then the source is translated, so that a source that
does not translate raises its translation error, and `new` then installs the
package as [the install rule](r:class.package.new.install) says. The arguments
from the fourth on are passed to `init` of the new package, and so are the
arguments from the third on when the source is omitted; the `init` of `Object`
accepts none, and one argument raises `SYNTAX 93.902` after the package is
installed. A subclass of `Package` makes instances of itself. The answer is
the new package.

```rexx
p = .package~new('named', 'return 1')
say p~class~id p~name
call try '.package~new()'
call try '.package~new(.nil, "nop")'
call try '.package~new(.list~new, "nop")'
call try '.package~new(.onlystr~new, "nop")'
call try '.package~new("n", .nil)'
call try '.package~new("n", "nop", .nil)'
call try '.package~new("n", "nop", "PROGRAMSCOPE")'
call try '.package~new("n", "nop", 5)'
call try '.package~new("n", "nop", .object~new)'
call try '.package~new("n", "nop", .routine~new("r", "nop"))~class~id'
call try '.package~new("n", "say (")'
call try '.package~new("n", "nop", , 1)'
call try '.package~new("n", "nop", .package~new("c", "nop"), 1)'
call try '.package~new("n", .nil, 5)'
say .mine~new('n', 'nop', , 'a', 'b')~class~id
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class mine subclass package
::method init
  say 'init' arg() arg(1) arg(2)
::class onlystr
::method string
  return 'onlystr'
```

```output
Package named
.package~new() => 88.901
.package~new(.nil, "nop") => 88.909
.package~new(.list~new, "nop") => 88.909
.package~new(.onlystr~new, "nop") => 88.909
.package~new("n", .nil) => 93.961
.package~new("n", "nop", .nil) => 93.953
.package~new("n", "nop", "PROGRAMSCOPE") => 93.953
.package~new("n", "nop", 5) => 93.953
.package~new("n", "nop", .object~new) => 93.953
.package~new("n", "nop", .routine~new("r", "nop"))~class~id => Package
.package~new("n", "say (") => 35.1
.package~new("n", "nop", , 1) => 93.902
.package~new("n", "nop", .package~new("c", "nop"), 1) => 93.902
.package~new("n", .nil, 5) => 93.953
init 2 a b
MINE
```

r[class.package.new.sends]
`new` sends the messages that
[the sends rule of `Method`](r:class.method.new.sends) states for a source
that is not a string and not an array, `request` with `ARRAY` and then with
`STRING` (the line is a string answer itself, and any other answer's
`objectName`), and then sends the new package `init`, after the package has
been installed, with the arguments from the fourth on, or from the third on
when the source is omitted. The name is converted by the string conversion
when it has a `MAKESTRING` method, once.

```rexx
f = 'class_package_sends_zq.tmp'
call sysfiledelete f
call lineout f, 'say "file prolog"'
call lineout f
say .mine~new('n', .src~new('say "source prolog"'), , 'a', 'b')~class~id
say .mine~new(f, , , 'c')~class~id
call sysfiledelete f
say .mine~new(.named~new('m'), 'nop')~name
::class src
::method init
  expose line
  use arg line
::method request
  expose line
  say 'request' arg(1)
  if arg(1) == 'ARRAY' then return .nil
  return line
::class mine subclass package
::method init
  say 'init' arg() arg(1) arg(2)
::class named
::method init
  expose n
  use arg n
::method makestring
  expose n
  say 'makestring'
  return n
```

```output
request ARRAY
request STRING
source prolog
init 2 a b
MINE
file prolog
init 2  c
MINE
makestring
init 0  
m
```

r[class.package.new.source]
The source is a string or an array of strings, read as
[the source rule of `Method`](r:class.method.new.source) says: a string is one
line and is not split at line ends, an array holds one line for each item,
converted by `MAKESTRING`, a line end character (`'0A'x`, `'0D'x` or `'1A'x`)
in a line raises `SYNTAX 13.1`, an item that has no `MAKESTRING` method,
`.nil` among them, or a hole before an item, raises `93.952`, and any other
object, an instance of a subclass of `String` included, is converted by
[the sends rule](r:class.method.new.sends): `request` with `ARRAY`, then with
`STRING`. What differs is that the lines of a package are the whole text,
directives included, so `source` answers the same lines, and that an empty
string is one empty line where an empty array is none. A line that the package
does not translate raises its error from `new`.

```rexx
call try '.package~new("n", "return 1")~source~items'
call try '.package~new("n", .array~of("a = 1", "return a"))~source[2]'
call try '.package~new("n", "")~source~items'
call try '.package~new("n", .array~new)~source~items'
call try '.package~new("n", .array~of("::class k", "::method m"))~sourceSize'
call try '.package~new("n", .array~of("return 1", .object~new))'
call try '.package~new("n", "a" || "0A"x || "b")'
call try '.package~new("n", .array~of("a" || "0D"x))'
call try '.package~new("n", .src~new(.array~of("return 2")))~source[1]'
call try '.package~new("n", .array~of(.onlystr~new))'
ssrc = .ss~new("nop" || "0A"x || "return 3")
call try '.package~new("n", ssrc)~sourceSize'
call try '.package~new("n", .ss~new(""))~sourceSize'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class src
::method init
  expose arr
  use arg arr
::method request
  expose arr
  say 'request' arg(1)
  return arr
::class onlystr
::method string
  return 'onlystr'
::class ss subclass string
```

```output
.package~new("n", "return 1")~source~items => 1
.package~new("n", .array~of("a = 1", "return a"))~source[2] => return a
.package~new("n", "")~source~items => 1
.package~new("n", .array~new)~source~items => 0
.package~new("n", .array~of("::class k", "::method m"))~sourceSize => 2
.package~new("n", .array~of("return 1", .object~new)) => 93.952
.package~new("n", "a" || "0A"x || "b") => 13.1
.package~new("n", .array~of("a" || "0D"x)) => 13.1
request ARRAY
.package~new("n", .src~new(.array~of("return 2")))~source[1] => return 2
.package~new("n", .array~of(.onlystr~new)) => 93.952
.package~new("n", ssrc)~sourceSize => 2
.package~new("n", .ss~new(""))~sourceSize => 0
```

```rexx
call try '.package~new("n", .array~of("nop", .mk~new))~source[2]'
call try '.package~new("n", .array~of(.ss~new("nop")))~source[1]'
call try '.package~new("n", .array~of(.mutablebuffer~new("nop")))~source[1]'
call try '.package~new("n", .array~of(.array~of("nop", "nop")))'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class mk
::method makestring
  return 'return 1'
::class ss subclass string
```

```output defect=#142
.package~new("n", .array~of("nop", .mk~new))~source[2] => return 1
.package~new("n", .array~of(.ss~new("nop")))~source[1] => nop
.package~new("n", .array~of(.mutablebuffer~new("nop")))~source[1] => nop
.package~new("n", .array~of(.array~of("nop", "nop"))) => 13.1
```

r[class.package.new.context]
The context decides which classes and routines the code of the package can
name besides its own and those of the library. A package made with no context
sees nothing of the package of the code that made it: a class name that it
does not hold is the string of that name, and a routine that it does not hold
raises `SYNTAX 43.1`. With a `Package` object as the context the package
finds, after its own, the classes and routines that the context package finds,
its own, those it imports and what its own context finds, and with a `Routine`
or a `Method` it finds those of the package of that object. A context that is
a `Method` or a `Routine` whose `package` is `.nil`, one that
`loadExternalMethod` or `loadExternalRoutine` made, gives a package whose
lookup through the context is not specified. The context package is not one of
the `importedPackages` of the new package, and its classes and routines are
not among the `classes` and `routines` of the new package.

```rexx
src = .array~of('return 1', '::class there', '::routine thereproc')
src~append('  return "there routine"')
p = .package~new('other', src)
r = .routine~new('r', 'return 1', p)
none = .package~new('a', 'say .here~class~id')
via = .package~new('b', 'say .there~class~id thereproc()', p)
viar = .package~new('c', 'say .there~class~id thereproc()', r)
call try '.package~new("d", "say here()")'
call try '.package~new("d", "say here()", p)'
c = .package~new('e', 'return 1', p)
say c~classes~items c~routines~items c~importedPackages~items
say c~findClass('THERE')~id (c~findClass('HERE') == .nil)
say c~findRoutine('THEREPROC')~class~id
src = .array~of('nop', '::class c2cls', '::routine c2r', '  return "c2"')
c2 = .package~new('c2', src)
c1 = .package~new('c1', 'nop', c2)
pc = .package~new('pc', 'nop', c1)
say pc~findClass('C2CLS')~isNil pc~findRoutine('C2R')~isNil
src = .array~of('return 1', '::class libpub public')
src~append('::routine libproc public')
src~append('  return "lib routine"')
p~addPackage(.package~new('lib', src))
imp = .package~new('f', 'say .libpub~class~id libproc()', p)
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class here
::routine here
  return 'here routine'
```

```output
String
Class there routine
Class there routine
.package~new("d", "say here()") => 43.1
.package~new("d", "say here()", p) => 43.1
0 0 0
THERE 1
Routine
0 0
Class lib routine
```

r[class.package.new.file]
With no source, the name is the name of a file, which `new` looks for as
[the search rule](r:dir.requires.create) of `::REQUIRES` does, reads and
installs. A file that cannot be found raises `SYNTAX 43.901`. The name of the
package is the name of the file as the search resolved it, an absolute name,
and not the name that was given. A file that was loaded before, by `new`, by
`loadPackage` or by a `::REQUIRES`, is not read or run again: `new` answers
the same package, which is also the answer when the file is given by another
name that resolves to the same file. A package that `new` made from a source
is never found that way, even when it has the name of a file; a package that
[`loadPackage`](r:class.package.loadpackage) installed from a source is found
by its name, as the rule of `loadPackage` says. How a file is searched for is
specified in the environment chapter.

```rexx
f = 'class_package_file_zq.tmp'
call sysfiledelete f
call lineout f, 'say "file body ran"'
call lineout f, '::class fromfile public'
call lineout f
p = .package~new(f)
say p~class~id p~classes~items (p~name == qualify(f)) (p~name == f)
q = .package~new(f)
say (p == q) (p == .package~new('./' || f))
say (p == p~loadPackage(f)) (p == .package~new(f, 'say "from source"'))
call sysfiledelete f
call try '.package~new("class_package_file_none.tmp")'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
file body ran
Package 1 1 0
1 1
from source
1 0
.package~new("class_package_file_none.tmp") => 43.901
```

r[class.package.new.install]
When `new` has read the source it installs the package as the program is
installed when it runs. The packages of its `::REQUIRES` directives are loaded
first, each as this rule says, then the classes of its `::CLASS` directives
are made, and then, unless `::OPTIONS NOPROLOG` was given, the prolog runs,
which is the code before the first directive. An error in any of these is
raised from `new`, and a package whose prolog ended with `EXIT` or `RETURN` is
made as it is; the value is dropped. A package that was made has been
installed, so a class of the package has had its class `init` run before the
prolog starts. The installation of the directives is specified by [the
directive pages](../directives/index.md).

```rexx
f = 'class_package_order_zq.tmp'
call sysfiledelete f
call lineout f, 'say "required prolog"'
call lineout f, '::class reqclass public'
call lineout f, '::method init class'
call lineout f, '  say "required class init"'
call lineout f
src = .array~of('say "prolog"', '::requires "' || f || '"')
src~append('::class own')
src~append('::method init class')
src~append('  say "own class init"')
p = .package~new('order', src)
say 'made' p~classes~items
q = .package~new('quiet', .array~of('say "never"', '::options noprolog'))
say 'quiet made' q~prolog~class~id
call try '.package~new("e1", "say 1; say 1/0")'
src = .array~of('say 2', '::requires "class_package_none.cls"')
call try '.package~new("e2", src)'
call try '.package~new("e3", "exit 5")~class~id'
call try '.package~new("e4", "return 5")~class~id'
call sysfiledelete f
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
required class init
required prolog
own class init
prolog
made 1
quiet made Routine
1
.package~new("e1", "say 1; say 1/0") => 42.3
.package~new("e2", src) => 43.901
.package~new("e3", "exit 5")~class~id => Package
.package~new("e4", "return 5")~class~id => Package
```

### defaultOptions

r[class.package.defaultoptions]
`defaultOptions` reads and sets the options that a package gets when it is
made without the corresponding `::OPTIONS`. It takes an option name and
optionally a value, and no further argument (`SYNTAX 93.902`). The name is
required (`88.901`), an object with no `MAKESTRING` method, `.nil` among them,
raises `88.909`, and only its first letter counts: `D` names
`DefineDefaultOptions` and `C` names `CountOverrides`, in any case; another
name raises `SYNTAX 93.914`. With no value the call answers the current value,
and with a value it sets it and answers the previous one.
`DefineDefaultOptions` is a string in the form of an `::OPTIONS` directive,
which is the same form that [`options`](r:class.package.options.settings)
answers; a string that is not one `::OPTIONS` directive, in any case and
after optional blanks, raises `93.900`, as a string of two directives does, a
string whose options the directive refuses raises the error of the directive
(`25.924` for an unknown option), and a directive with no options leaves every
option at its initial value. The initial value names `DIGITS 9`,
`FORM SCIENTIFIC`, `FUZZ 0`, `NUMERIC NOINHERIT`, each of the six condition
options with `CONDITION`, `PROLOG` and `TRACE NORMAL`, as
[`options`](r:class.package.options) lists them. `CountOverrides` is a whole
number whose initial value is `0`; a value that is not a whole number raises
`93.905`. While it is positive, every package that is made, by `new` and by
the `new` and `newFile` of `Method` and of `Routine`, which make a package of
their own, gets the default options in place of the initial ones and reduces
the count by one; a package that gives an option in a directive keeps its own
value for that option. A count that is `0` leaves the packages that are made
with the initial options, and a negative count applies the defaults to every
package that is made, and is reduced by one for each, so that it stays
negative.

```rexx
say .package~defaultOptions('CountOverrides')
say .package~defaultOptions('DefineDefaultOptions')~subword(1, 3)
old = .package~defaultOptions('DefineDefaultOptions', '::OPTIONS DIGITS 20')
say old~subword(1, 3)
say .package~defaultOptions('D')~subword(1, 3)
say .package~defaultOptions('CountOverrides', 3)
p = .package~new('first', 'nop')
q = .package~new('second', .array~of('nop', '::options digits 11'))
r = .routine~new('third', 'return digits()')
say p~digits q~digits r~call r~package~digits
say .package~defaultOptions('CountOverrides')
s = .package~new('fourth', 'nop')
say s~digits .package~defaultOptions('CountOverrides')
f = 'class_package_defaults_zq.tmp'
call sysfiledelete f
call lineout f, 'return 1'
call lineout f
.package~defaultOptions('CountOverrides', 2)
say .routine~newFile(f)~package~digits .method~newFile(f)~package~digits
say .package~defaultOptions('CountOverrides')
call sysfiledelete f
.package~defaultOptions('CountOverrides', -1)
t = .package~new('fifth', 'nop')
u = .package~new('sixth', 'nop')
say t~digits u~digits .package~defaultOptions('CountOverrides')
.package~defaultOptions('CountOverrides', 0)
say .package~defaultOptions('DefineDefaultOptions', '::OPTIONS')~subword(1, 3)
say .package~defaultOptions('DefineDefaultOptions')~subword(1, 3)
call try '.package~defaultOptions'
call try '.package~defaultOptions(.nil)'
call try '.package~defaultOptions("Other")'
call try '.package~defaultOptions("D", "text")'
call try '.package~defaultOptions("D", "OPTIONS DIGITS 5")'
two = '::OPTIONS DIGITS 7; ::OPTIONS FORM ENGINEERING'
call try '.package~defaultOptions("D", two)'
call try '.package~defaultOptions("D", "::OPTIONS BADWORD")'
call try '.package~defaultOptions("C", "text")'
call try '.package~defaultOptions("C", 1.5)'
call try '.package~defaultOptions("C", 0, 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
0
::OPTIONS DIGITS 9
::OPTIONS DIGITS 9
::OPTIONS DIGITS 20
0
20 11 20 20
0
9 0
20 20
0
20 20 -3
::OPTIONS DIGITS 20
::OPTIONS DIGITS 9
.package~defaultOptions => 88.901
.package~defaultOptions(.nil) => 88.909
.package~defaultOptions("Other") => 93.914
.package~defaultOptions("D", "text") => 93.900
.package~defaultOptions("D", "OPTIONS DIGITS 5") => 93.900
.package~defaultOptions("D", two) => 93.900
.package~defaultOptions("D", "::OPTIONS BADWORD") => 25.924
.package~defaultOptions("C", "text") => 93.905
.package~defaultOptions("C", 1.5) => 93.905
.package~defaultOptions("C", 0, 1) => 93.902
```

## Methods

### What the package is

r[class.package.name]
`name` answers the name of the package: the name that `new` was given for a
package made from a source, the name of the source for a package that
`loadPackage` loaded from a source, the resolved file name for a package made
from a file or loaded by `::REQUIRES`, and `REXX` for the package of the
library classes. The package of a method or a routine that `new` or `newFile`
made has the name that was given to the method or routine, as [the context
rule](r:class.method.new.context) says. The method takes no argument
(`SYNTAX 93.902`).

```rexx
say .package~new('given', 'nop')~name .package~new('1', 'nop')~name
f = 'class_package_name_zq.tmp'
call sysfiledelete f
call lineout f, 'nop'
call lineout f
say (.package~new(f)~name == qualify(f))
say (.routine~newFile(f)~package~name == f)
call sysfiledelete f
say .object~package~name .routine~new('rt', 'nop')~package~name
call try '.package~new("n", "nop")~name(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
given 1
1
1
REXX rt
.package~new("n", "nop")~name(1) => 93.902
```

r[class.package.source]
`source` answers a new array that holds the lines of the package's source, one
string for each line, directives included, so changing the array changes
nothing in the package. A package whose source is not available, such as the
package of the library classes, answers an array of no items. The method takes
no argument (`SYNTAX 93.902`).

```rexx
p = .package~new('n', .array~of('nop', '::class k', '::method m'))
a = p~source
say a~class~id a~items (a == p~source) a[2]
a~put('changed', 1)
say p~source[1]
say .object~package~source~items
call try 'p~source(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
Array 3 0 ::class k
nop
0
p~source(1) => 93.902
```

r[class.package.sourceline]
`sourceLine` answers the line of the source that has the number it is given,
counted from 1. A number beyond the last line answers the null string, as
does any line of a package whose source is not available. The argument is
required (`SYNTAX 93.903`) and must be a positive whole number: zero, a
negative number, a number with a fraction and a value that is not a number,
`.nil` included, raise `SYNTAX 93.924`; a second argument raises `93.902`.

```rexx
p = .package~new('n', .array~of('nop', '', '  nop  /* a', ' b */'))
say '['p~sourceLine(1)']' '['p~sourceLine(2)']' '['p~sourceLine(3)']'
say '['p~sourceLine(4)']' '['p~sourceLine(5)']' '['p~sourceLine(500)']'
say '['.object~package~sourceLine(1)']'
call try 'p~sourceLine'
call try 'p~sourceLine(0)'
call try 'p~sourceLine(-1)'
call try 'p~sourceLine(1.5)'
call try 'p~sourceLine("a")'
call try 'p~sourceLine(.nil)'
call try 'p~sourceLine(1, 2)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
[nop] [] [  nop  /* a]
[ b */] [] []
[]
p~sourceLine => 93.903
p~sourceLine(0) => 93.924
p~sourceLine(-1) => 93.924
p~sourceLine(1.5) => 93.924
p~sourceLine("a") => 93.924
p~sourceLine(.nil) => 93.924
p~sourceLine(1, 2) => 93.902
```

r[class.package.sourcesize]
`sourceSize` answers the number of lines of the source, which is the number of
items that `source` answers, and `0` for a package whose source is not
available. The method takes no argument (`SYNTAX 93.902`).

```rexx
say .package~new('n', .array~of('a = 1', 'b = 2'))~sourceSize
say .package~new('n', 'nop')~sourceSize .package~new('n', '')~sourceSize
say .package~new('n', .array~new)~sourceSize .object~package~sourceSize
call try '.package~new("n", "nop")~sourceSize(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
2
1 1
0 0
.package~new("n", "nop")~sourceSize(1) => 93.902
```

r[class.package.prolog]
`prolog` answers a `Routine` that holds the code of the package that comes
before its first directive: its source is those lines, and its package is the
package. It is the same object every time. Sending `call` to it runs the
prolog again, as `call` of a routine does, with the answer of the code. The
routine exists also for a package with no code before its first directive,
whose source is empty, and for a package with `NOPROLOG`, whose prolog did not
run when the package was made. A first line that begins with `#!` is not part
of the prolog's source, though it stays in the package's own `source`. The
library package, and the package of a method or routine that `new` or
`newFile` of `Method` or `Routine` made, have no prolog: `prolog` answers
`.nil`. The method takes no argument (`SYNTAX 93.902`).

```rexx
p = .package~new('n', .array~of('say "prolog ran"', 'return 9', '::class k'))
r = p~prolog
say r~class~id r~source~items (r == p~prolog) (r~package == p)
say r~call
q = .package~new('q', .array~of('::class k'))
say q~prolog~class~id q~prolog~source~items
src = .array~of('say "not run at make"', 'return 8', '::options noprolog')
n = .package~new('n2', src)
say n~prolog~call
say .object~package~prolog
say .method~new('mm', 'return 2')~package~prolog~isNil
say .routine~new('rr', 'return 7')~package~prolog~isNil
call try 'p~prolog(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
prolog ran
Routine 2 1 1
prolog ran
9
Routine 0
not run at make
8
The NIL object
1
1
p~prolog(1) => 93.902
```

```rexx
f = 'class_package_shebang_zq.tmp'
call sysfiledelete f
call lineout f, '#!/usr/bin/rexx'
call lineout f, 'return "after the first line"'
call lineout f
p = .package~new(f)
say p~prolog~source~items p~prolog~source[1]
say p~source~items p~source[1] p~sourceSize '['p~sourceLine(1)']'
call sysfiledelete f
```

```output defect=#144
1 return "after the first line"
2 #!/usr/bin/rexx 2 [#!/usr/bin/rexx]
```

### What the package holds

r[class.package.classes]
`classes` answers a new `StringTable` of the classes that the package's
`::CLASS` directives made, public or not, indexed by the upper case class
name, together with the classes that `addClass` and `addPublicClass` added.
The classes that the package imports are not among them. Changing the table
does not change the package. The method takes no argument (`SYNTAX 93.902`).

```rexx
src = .array~of('return 1', '::class pub public', '::class priv')
p = .package~new('n', src)
t = p~classes
say t~class~id t~items t['PUB']~id t['PRIV']~id
t~put(.object, 'EXTRA')
say p~classes~items (p~classes == p~classes)
say .package~new('empty', 'nop')~classes~items
call try 'p~classes(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
StringTable 2 PUB PRIV
2 0
0
p~classes(1) => 93.902
```

r[class.package.publicclasses]
`publicClasses` answers a new `StringTable` of the classes of the package that
are public, whether a `::CLASS ... PUBLIC` directive or `addPublicClass` made
them public, indexed by the upper case class name. The method takes no
argument (`SYNTAX 93.902`).

```rexx
src = .array~of('return 1', '::class pub public', '::class priv')
p = .package~new('n', src)
t = p~publicClasses
say t~class~id t~items t['PUB']~id (t~hasIndex('PRIV'))
p~addPublicClass('Added', .object)
say p~publicClasses~allIndexes~sort~makeArray~makeString('l', ',')
say p~classes~allIndexes~sort~makeArray~makeString('l', ',')
say .object~package~publicClasses~hasIndex('ARRAY')
call try 'p~publicClasses(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
StringTable 1 PUB 0
ADDED,PUB
ADDED,PRIV,PUB
1
p~publicClasses(1) => 93.902
```

r[class.package.routines]
`routines` answers a new `StringTable` of the routines of the package,
public or not, indexed by the upper case routine name: those that its
`::ROUTINE` directives made and those that `addRoutine` and `addPublicRoutine`
added. The routines that the package imports are not among them. The method
takes no argument (`SYNTAX 93.902`).

```rexx
src = .array~of('return 1', '::routine pubr public', '  return "p"')
src~append('::routine privr')
src~append('  return "q"')
p = .package~new('n', src)
t = p~routines
say t~class~id t~items t['PUBR']~call t['PRIVR']~call
say (p~routines == p~routines)
say .package~new('empty', 'nop')~routines~items
call try 'p~routines(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
StringTable 2 p q
0
0
p~routines(1) => 93.902
```

r[class.package.publicroutines]
`publicRoutines` answers a new `StringTable` of the routines of the package
that are public, indexed by the upper case routine name: those that a
`::ROUTINE ... PUBLIC` directive or `addPublicRoutine` made public. The method
takes no argument (`SYNTAX 93.902`).

```rexx
src = .array~of('return 1', '::routine pubr public', '  return "p"')
src~append('::routine privr')
src~append('  return "q"')
p = .package~new('n', src)
t = p~publicRoutines
say t~class~id t~items t['PUBR']~call (t~hasIndex('PRIVR'))
p~addPublicRoutine('Added', t['PUBR'])
say p~publicRoutines~allIndexes~sort~makeArray~makeString('l', ',')
say .object~package~publicRoutines~items
call try 'p~publicRoutines(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
StringTable 1 p 0
ADDED,PUBR
0
p~publicRoutines(1) => 93.902
```

r[class.package.definedmethods]
`definedMethods` answers a new `StringTable` of the methods that the package's
`::METHOD`, `::ATTRIBUTE` and `::CONSTANT` directives declared outside any
`::CLASS`, the methods that belong to no class, indexed by the upper case
method name; they are there also when a `::ROUTINE` directive comes before
them, and an attribute makes the two methods of its name and its name with an
equals sign. A method that a class holds is not among them. The method takes
no argument (`SYNTAX 93.902`).

```rexx
src = .array~of('::method fm1', '  return 1', '::method fm2 private')
src~append('::class k')
src~append('::method inclass')
p = .package~new('n', src)
t = p~definedMethods
say t~class~id t~items t['FM1']~class~id t['FM2']~isPrivate
say t['FM1']~scope~isNil (t['FM1']~package == p)
say p~classes['K']~method('INCLASS')~scope~id
say .package~new('empty', 'nop')~definedMethods~items
src = .array~of('nop', '::routine r1', '  return 1', '::method afterroutine')
src~append('::attribute fa')
src~append('::constant fc 3')
src~append('::class k')
src~append('::method inclass')
t = .package~new('n2', src)~definedMethods
say t~items t~allIndexes~sort~makeArray~makeString('l', ',')
call try 'p~definedMethods(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
StringTable 2 Method 1
1 1
K
0
4 AFTERROUTINE,FA,FA=,FC
p~definedMethods(1) => 93.902
```

r[class.package.resources]
`resources` answers a new `StringTable` of the data resources that the
package's `::RESOURCE` directives made, as [the
directive](r:dir.resource.access) says, indexed by the upper case resource
name, each value an array of the lines. A package with none answers an empty
table. The method takes no argument (`SYNTAX 93.902`).

```rexx
src = .array~of('return 1', '::resource first', 'line a', 'line b', '::END')
src~append('::resource "second one"')
src~append('only line')
src~append('::END')
p = .package~new('n', src)
t = p~resources
say t~class~id t~items
say t~allIndexes~sort~makeArray~makeString('l', ',')
say t['FIRST']~class~id t['FIRST']~items t['FIRST'][2]
say .package~new('none', 'nop')~resources~items
call try 'p~resources(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
StringTable 2
FIRST,SECOND ONE
Array 2 line b
0
p~resources(1) => 93.902
```

r[class.package.resource]
`resource` answers the array of lines of the resource that has the name it is
given, as [the directive](r:dir.resource.access) says, and `.nil` when the
package has no resource of that name; the name is taken in upper case. The
name is required (`SYNTAX 88.901`), an object with no `MAKESTRING` method,
`.nil` among them, raises `88.909`, and a second argument raises `93.902`.

```rexx
src = .array~of('return 1', '::resource first', 'line a', 'line b', '::END')
p = .package~new('n', src)
say p~resource('first')~items p~resource('FIRST')[1] p~resource('First')[2]
say p~resource('nothere')~isNil
call try 'p~resource'
call try 'p~resource(.nil)'
call try 'p~resource("first", 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
2 line a line b
1
p~resource => 88.901
p~resource(.nil) => 88.909
p~resource("first", 1) => 93.902
```

r[class.package.annotation]
`annotation` answers the value of the annotation of the package that has the
name it is given, and `.nil` when there is none, as
[the annotation rule](r:dir.annotate.read) says: the name is taken in upper
case. The name is required (`SYNTAX 88.901`), an object with no `MAKESTRING`
method, `.nil` among them, raises `88.909`, and a second argument raises
`93.902`.

```rexx
src = .array~of('return 1', '::annotate package Version 3')
p = .package~new('n', src)
say p~annotation('Version') p~annotation('VERSION') p~annotation('none')~isNil
call try 'p~annotation'
call try 'p~annotation(.nil)'
call try 'p~annotation("a", "b")'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
3 3 1
p~annotation => 88.901
p~annotation(.nil) => 88.909
p~annotation("a", "b") => 93.902
```

r[class.package.annotations]
`annotations` answers the `StringTable` that holds every annotation of the
package, indexed by the upper case name, as
[the annotation rule](r:dir.annotate.read) says. It is the table that the
package keeps: the same object every time, so an entry put into it is an
annotation that `annotation` answers. A package with none has an empty table.
The method takes no argument (`SYNTAX 93.902`).

```rexx
src = .array~of('return 1', '::annotate package Version 3')
p = .package~new('n', src)
t = p~annotations
say t~class~id t~items t['VERSION'] (t == p~annotations)
t~put('later', 'ADDED')
say p~annotation('added') p~annotations~items
say .package~new('none', 'nop')~annotations~items
call try 'p~annotations(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
StringTable 1 3 1
later 2
0
p~annotations(1) => 93.902
```

r[class.package.local]
`local` answers a `Directory` that holds objects local to the package, the
same object every time, empty when the package is made. It is not `.local`. An
object that is put into it under an upper case name is found by the code of
the package when it names that object with `.name`, before `.local` and
`.environment` are searched, and the other packages do not see it. The method
takes no argument (`SYNTAX 93.902`).

```rexx
src = .array~of('return 1', '::routine rd', '  return .zqv')
p = .package~new('n', src)
q = .package~new('q', src)
say p~local~class~id p~local~items (p~local == p~local) (p~local == .local)
say p~findRoutine('RD')~call
p~local~put('package value', 'ZQV')
say p~findRoutine('RD')~call q~findRoutine('RD')~call
.local~zqv = 'in local'
say p~findRoutine('RD')~call q~findRoutine('RD')~call
call try 'p~local(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
Directory 0 1 0
.ZQV
package value .ZQV
package value in local
p~local(1) => 93.902
```

r[class.package.namespaces]
`namespaces` answers a new `StringTable` of the namespaces that the package
names, indexed by the upper case name, each value the package that was added
under the name by a `::REQUIRES ... NAMESPACE` directive or by `addPackage`
with a namespace. A package that names none answers an empty table: the `REXX`
namespace, which every package knows, is not in it unless `addPackage` was
given that name. The method takes no argument (`SYNTAX 93.902`).

```rexx
p = .package~new('n', 'return 1')
say p~namespaces~class~id p~namespaces~items
other = .package~new('other', 'return 2')
p~addPackage(other, 'tagged')
say p~namespaces~items p~namespaces~allIndexes~makeArray~makeString('l', ',')
say (p~namespaces['TAGGED'] == other) (p~namespaces == p~namespaces)
call try 'p~namespaces(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
StringTable 0
1 TAGGED
1 0
p~namespaces(1) => 93.902
```

r[class.package.importedclasses]
`importedClasses` answers a new `StringTable` of the public classes that the
package imports from the packages of its `::REQUIRES` directives, of
`loadPackage` and of `addPackage`, indexed by the upper case class name, and
among them the public classes of the packages that those import, as
[the visibility rule](r:dir.class.visibility) says. When two imported packages
have a public class of one name, the one of the package that was imported
first is the entry. The classes that the package itself made are not among
them. The method takes no argument (`SYNTAX 93.902`).

```rexx
src = .array~of('return 1', '::class shown public', '::class hidden')
lib = .package~new('lib', src)
p = .package~new('n', 'return 1')
say p~importedClasses~class~id p~importedClasses~items
p~addPackage(lib)
t = p~importedClasses
say t~items t~allIndexes~makeArray~makeString('l', ',') t['SHOWN']~id
say p~classes~items (p~importedClasses == p~importedClasses)
call try 'p~importedClasses(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
StringTable 0
1 SHOWN SHOWN
0 0
p~importedClasses(1) => 93.902
```

```rexx
src = .array~of('nop', '::class deep public', '::class dup public')
a = .package~new('a', src)
b = .package~new('b', .array~of('nop', '::class mid public'))
b~addPackage(a)
p = .package~new('p', 'nop')
p~addPackage(b)
say p~importedClasses~allIndexes~sort~makeArray~makeString('l', ',')
say p~importedPackages~items (p~importedPackages[1] == b)
x = .package~new('x', .array~of('nop', '::class dup public'))
y = .package~new('y', .array~of('nop', '::class dup public'))
q = .package~new('q', 'nop')
q~addPackage(x)
q~addPackage(y)
say (q~importedClasses['DUP'] == x~classes['DUP'])
```

```output
DEEP,DUP,MID
1 1
1
```

r[class.package.importedroutines]
`importedRoutines` answers a new `StringTable` of the public routines that the
package imports from the packages of its `::REQUIRES` directives, of
`loadPackage` and of `addPackage`, indexed by the upper case routine name, and
among them the public routines of the packages that those import. When two
imported packages have a public routine of one name, the one of the package
that was imported first is the entry. The routines that the package itself
made are not among them. The method takes no argument (`SYNTAX 93.902`).

```rexx
src = .array~of('return 1', '::routine shown public', '  return "s"')
src~append('::routine hidden')
src~append('  return "h"')
lib = .package~new('lib', src)
p = .package~new('n', 'return 1')
say p~importedRoutines~class~id p~importedRoutines~items
p~addPackage(lib)
t = p~importedRoutines
say t~items t~allIndexes~makeArray~makeString('l', ',') t['SHOWN']~call
say p~routines~items (p~importedRoutines == p~importedRoutines)
call try 'p~importedRoutines(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
StringTable 0
1 SHOWN s
0 0
p~importedRoutines(1) => 93.902
```

```rexx
src = .array~of('nop', '::routine rdeep public', '  return "d"')
a = .package~new('a', src)
b = .package~new('b', 'nop')
b~addPackage(a)
p = .package~new('p', 'nop')
p~addPackage(b)
say p~importedRoutines~allIndexes~makeArray~makeString('l', ',')
say p~importedPackages~items p~findRoutine('RDEEP')~call
src = .array~of('nop', '::routine rdup public', '  return "from x"')
x = .package~new('x', src)
src = .array~of('nop', '::routine rdup public', '  return "from y"')
y = .package~new('y', src)
q = .package~new('q', 'nop')
q~addPackage(x)
q~addPackage(y)
say q~importedRoutines['RDUP']~call q~findRoutine('RDUP')~call
```

```output
RDEEP
1 d
from x from x
```

r[class.package.importedpackages]
`importedPackages` answers a new `Array` of the packages that the package
imports, in the order that it imported them: those of its `::REQUIRES`
directives, of `loadPackage` and of `addPackage`. A package that is imported
twice is in the array once. The array holds only the packages that the
receiver imported itself: a package that one of those imports is not in it.
The package of the library classes is not among them. The method takes no
argument (`SYNTAX 93.902`).

```rexx
a = .package~new('a', 'return 1')
b = .package~new('b', 'return 1')
p = .package~new('n', 'return 1')
say p~importedPackages~class~id p~importedPackages~items
p~addPackage(b)
p~addPackage(a)
p~addPackage(b)
x = p~importedPackages
say x~items (x[1] == b) (x[2] == a) (p~importedPackages == x)
c = .package~new('c', 'return 1')
b~addPackage(c)
say p~importedPackages~items (b~importedPackages[1] == c)
call try 'p~importedPackages(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
Array 0
2 1 1 0
2 1
p~importedPackages(1) => 93.902
```

### Finding

r[class.package.findclass]
`findClass` looks a name up as the environment symbol of the same name is
looked up in the package's code, and answers what it finds, or `.nil`. The
name is taken in upper case. The places that are searched, in this order, are
the classes of the package, public or not; the classes that the package of its
context finds, which are its own, public or not, the public ones of the
packages it imports, and what its own context finds; the public classes of the
packages it imports, the one imported first winning among equal names; the
public classes of the library; the package's [`local`](r:class.package.local)
directory; and then `.local` and `.environment`, so the answer may be an
object that is not a class, such as `.nil`, `.true` or `.local`, and a name
that nothing holds answers `.nil`. The first place that holds the name
decides. The name is required (`SYNTAX 88.901`), an object with no
`MAKESTRING` method, `.nil` among them, raises `88.909`, and a second argument
raises `93.902`. Any string is a name: a name with a blank or a period is
found when `addClass`, `addPublicClass` or a directory holds it under that
name. A name that has a colon is not looked up through a namespace: `tg:shown`
and `rexx:string` answer `.nil`, even when a package that holds a public class
`SHOWN` was added under the namespace `TG`.

```rexx
src = .array~of('return 1', '::class own public', '::class priv')
p = .package~new('n', src)
say p~findClass('OWN')~id p~findClass('own')~id p~findClass('PRIV')~id
say p~findClass('String')~id p~findClass('RexxInfo')~class~id
say p~findClass('TRUE') p~findClass('nil')~isNil
say p~findClass('LOCAL')~class~id p~findClass('ENVIRONMENT')~class~id
p~addClass('a.b', .array)
say p~findClass('nothere')~isNil p~findClass('a.b')~id
.local~zqfind = 'from local'
say p~findClass('ZQFIND')
o = .package~new('o', .array~of('nop', '::class shown public'))
p~addPackage(o, 'tg')
say p~findClass('SHOWN')~id p~findClass('tg:shown')~isNil
say p~findClass('rexx:string')~isNil p~findClass('rexx:')~isNil
call try 'p~findClass'
call try 'p~findClass(.nil)'
call try 'p~findClass("x", 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
OWN OWN PRIV
String RexxInfo
1 1
Directory Directory
1 Array
from local
SHOWN 1
1 1
p~findClass => 88.901
p~findClass(.nil) => 88.909
p~findClass("x", 1) => 93.902
```

```rexx
src = .array~of('return 1', '::class cx', '::class same', '::class array')
ctx = .package~new('ctx', src)
src = .array~of('return 1', '::class same public', '::class array public')
lib = .package~new('lib', src)
lib2 = .package~new('lib2', .array~of('return 1', '::class libc public'))
ctx~addPackage(lib2)
p = .package~new('own', .array~of('return 1', '::class mine'), ctx)
p~addPackage(lib)
say (p~findClass('MINE') == p~classes['MINE'])
say (p~findClass('CX') == ctx~classes['CX'])
say (p~findClass('SAME') == ctx~classes['SAME'])
say (p~findClass('ARRAY') == ctx~classes['ARRAY'])
say (p~findClass('ARRAY') == .array)
q = .package~new('q', 'return 1')
q~addPackage(lib)
say (q~findClass('ARRAY') == lib~classes['ARRAY'])
say (q~findClass('STRING') == .string) (q~findClass('ARRAY') == .array)
say (p~findClass('LIBC') == lib2~classes['LIBC'])
say .package~new('z', 'nop', .package~new('y', 'nop', p))~findClass('MINE')~id
lib3 = .package~new('lib3', .array~of('return 1', '::class same public'))
q~addPackage(lib3)
say (q~findClass('SAME') == lib~classes['SAME'])
q~local~put('package local', 'ZQSTR')
.local~zqstr = 'in local'
.local~zqonly = 'only in local'
.environment~zqonly = 'in environment'
say q~findClass('ZQSTR') '/' q~findClass('ZQONLY')
```

```output
1
1
1
1
0
1
1 0
1
MINE
1
package local / only in local
```

r[class.package.findpublicclass]
`findPublicClass` answers the public class that has the name it is given, or
`.nil`: a public class of the package, a public class that the package of its
context finds, one of the packages it imports (the one imported first winning
among equal names), and the classes of the library that are public; a class
that is not public, a library class that is not public, such as `RexxInfo`,
and a name that is not a class answer `.nil`. The name is taken in upper case.
The name is required (`SYNTAX 88.901`), an object with no `MAKESTRING` method,
`.nil` among them, raises `88.909`, and a second argument raises `93.902`. A
name that has a colon is not looked up through a namespace, as for
[`findClass`](r:class.package.findclass): `tg:shown` answers `.nil`.

```rexx
src = .array~of('return 1', '::class own public', '::class priv')
p = .package~new('n', src)
say p~findPublicClass('own')~id p~findPublicClass('String')~id
say p~findPublicClass('PRIV')~isNil p~findPublicClass('RexxInfo')~isNil
say p~findPublicClass('NIL')~isNil p~findPublicClass('nothere')~isNil
q = .package~new('q', 'return 1')
q~addPackage(p)
say q~findPublicClass('OWN')~id q~findPublicClass('PRIV')~isNil
src = .array~of('nop', '::class cpub public', '::class cpriv')
c = .package~new('c', 'nop', .package~new('ctx', src))
say c~findPublicClass('CPUB')~id c~findPublicClass('CPRIV')~isNil
src = .array~of('nop', '::class deep public', '::routine deepr public')
src~append('  return "deep"')
d2 = .package~new('d2', src)
d = .package~new('d', 'nop', .package~new('d1', 'nop', d2))
say d~findPublicClass('DEEP')~id d~findPublicRoutine('DEEPR')~call
o = .package~new('o', .array~of('nop', '::class shown public'))
p~addPackage(o, 'tg')
say p~findPublicClass('tg:shown')~isNil p~findPublicClass('shown')~id
p~addPublicClass('c:d', .array)
say p~findPublicClass('c:d')~id
call try 'p~findPublicClass'
call try 'p~findPublicClass(.nil)'
call try 'p~findPublicClass("x", 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
OWN String
1 1
1 1
OWN 1
CPUB 1
DEEP deep
1 SHOWN
Array
p~findPublicClass => 88.901
p~findPublicClass(.nil) => 88.909
p~findPublicClass("x", 1) => 93.902
```

r[class.package.findroutine]
`findRoutine` answers the routine that has the name it is given, or `.nil`.
The name is taken in upper case. The routines it looks through, in this order,
are those of the package, public or not, those that were added to it, the
routines that the package of its context finds, which are its own, public or
not, the public ones of the packages it imports, and what its own context
finds, and the public routines of the packages it imports, the one imported
first winning among equal names; the first that holds the name decides.
Built-in functions and the functions of native libraries are not routine
objects: they answer `.nil`. A name that has a colon is not looked up through
a namespace: `tg:rr` answers `.nil` even when a package that holds a public
routine `RR` was added under the namespace `TG`; a routine that was added
under a name with a colon is found by that name. The name is required
(`SYNTAX 88.901`), an object with no `MAKESTRING` method, `.nil` among them,
raises `88.909`, and a second argument raises `93.902`.

```rexx
src = .array~of('return 1', '::routine pubr public', '  return "p"')
src~append('::routine privr')
src~append('  return "q"')
p = .package~new('n', src)
say p~findRoutine('PUBR')~call p~findRoutine('privr')~call
say (p~findRoutine('pubr') == p~routines['PUBR'])
say p~findRoutine('nothere')~isNil p~findRoutine('')~isNil
o = .package~new('o', .array~of('nop', '::routine rr public'))
p~addPackage(o, 'tg')
say p~findRoutine('RR')~class~id p~findRoutine('tg:rr')~isNil
say p~findRoutine('LEFT')~isNil p~findRoutine('SysSleep')~isNil
p~addRoutine('e:f', .routine~new('x', 'return "ef"'))
say p~findRoutine('e:f')~call
q = .package~new('q', 'return 1')
q~addPackage(p)
say q~findRoutine('PUBR')~call q~findRoutine('PRIVR')~isNil
call try 'p~findRoutine'
call try 'p~findRoutine(.nil)'
call try 'p~findRoutine("x", 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
p q
1
1 1
Routine 1
1 1
ef
p 1
p~findRoutine => 88.901
p~findRoutine(.nil) => 88.909
p~findRoutine("x", 1) => 93.902
```

```rexx
src = .array~of('return 1', '::routine same', '  return "ctx"')
src~append('::routine s2')
src~append('  return "ctx2"')
ctx = .package~new('ctx', src)
src = .array~of('return 1', '::routine same public', '  return "lib"')
src~append('::routine s2 public')
src~append('  return "lib2"')
src~append('::routine libonly public')
src~append('  return "libonly"')
lib = .package~new('lib', src)
src = .array~of('return 1', '::routine ctxlib public', '  return "ctxlib"')
ctx~addPackage(.package~new('lib2', src))
src = .array~of('return 1', '::routine same', '  return "own"')
p = .package~new('own', src, ctx)
p~addPackage(lib)
say p~findRoutine('SAME')~call p~findRoutine('S2')~call
say p~findRoutine('LIBONLY')~call p~findPublicRoutine('S2')~call
say p~findPublicRoutine('SAME')~call p~findRoutine('CTXLIB')~call
src = .array~of('nop', '::routine rdup public', '  return "from x"')
x = .package~new('x', src)
src = .array~of('nop', '::routine rdup public', '  return "from y"')
y = .package~new('y', src)
q = .package~new('q', 'nop')
q~addPackage(x)
q~addPackage(y)
say q~findRoutine('RDUP')~call
```

```output
own ctx2
libonly ctx2
own ctxlib
from x
```

r[class.package.findpublicroutine]
`findPublicRoutine` answers the same routine as
[`findRoutine`](r:class.package.findroutine) does for the name it is given, or
`.nil`: a routine that the package holds or that its context finds answers
public or not, and a routine that an imported package holds answers only when
it is public there. The name is required (`SYNTAX 88.901`), an object with no
`MAKESTRING` method, `.nil` among them, raises `88.909`, and a second argument
raises `93.902`.

```rexx
src = .array~of('return 1', '::routine pubr public', '  return "p"')
src~append('::routine privr')
src~append('  return "q"')
p = .package~new('n', src)
say p~findPublicRoutine('PUBR')~call p~findPublicRoutine('privr')~call
say p~findPublicRoutine('nothere')~isNil
q = .package~new('q', 'return 1')
q~addPackage(p)
say q~findPublicRoutine('PUBR')~call q~findPublicRoutine('PRIVR')~isNil
call try 'p~findPublicRoutine'
call try 'p~findPublicRoutine(.nil)'
call try 'p~findPublicRoutine("x", 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
p q
1
p 1
p~findPublicRoutine => 88.901
p~findPublicRoutine(.nil) => 88.909
p~findPublicRoutine("x", 1) => 93.902
```

r[class.package.findnamespace]
`findNamespace` answers the package that is tagged with the namespace name it
is given, or `.nil`. The name is taken in upper case. The namespace `REXX` is
the package of the library classes in every package; other namespaces are
those that `namespaces` lists. The name is required (`SYNTAX 88.901`), an
object with no `MAKESTRING` method, `.nil` among them, raises `88.909`, and a
second argument raises `93.902`.

```rexx
p = .package~new('n', 'return 1')
other = .package~new('other', 'return 2')
p~addPackage(other, 'Tagged')
say (p~findNamespace('TAGGED') == other) (p~findNamespace('tagged') == other)
say (p~findNamespace('rexx') == .object~package) p~findNamespace('none')~isNil
call try 'p~findNamespace'
call try 'p~findNamespace(.nil)'
call try 'p~findNamespace("x", 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 1
1 1
p~findNamespace => 88.901
p~findNamespace(.nil) => 88.909
p~findNamespace("x", 1) => 93.902
```

r[class.package.findprogram]
`findProgram` answers the file name that the search for the program that is
named finds, in the absolute form that `new` gives a package for the file, or
`.nil` when the file is not found. The name is required (`SYNTAX 88.901`), an
object with no `MAKESTRING` method, `.nil` among them, raises `88.909`, and a
second argument raises `93.902`. The search is specified in the environment
chapter; it starts from the directory of the receiver's file, so packages that
were loaded from different directories can answer differently.

```rexx
f = 'class_package_find_zq.tmp'
call sysfiledelete f
call lineout f, 'nop'
call lineout f
p = .package~new('n', 'return 1')
say (p~findProgram(f) == qualify(f)) (p~findProgram('./' || f) == qualify(f))
say p~findProgram('class_package_find_none.tmp')~isNil
say (p~findProgram(f) == .package~new('other', 'nop')~findProgram(f))
call sysfiledelete f
d = 'class_package_dir_zq'
call sysmkdir d
call lineout d'/class_package_dir_in.tmp', 'nop'
call lineout d'/class_package_dir_in.tmp'
call lineout d'/class_package_dir_pk.cls', '::routine x public'
call lineout d'/class_package_dir_pk.cls'
sub = .package~new(d'/class_package_dir_pk.cls')
say sub~findProgram('class_package_dir_in.tmp')~isNil
say .context~package~findProgram('class_package_dir_in.tmp')~isNil
found = sub~findProgram('class_package_dir_in.tmp')
say (found == qualify(d'/class_package_dir_in.tmp'))
call sysfiledelete d'/class_package_dir_in.tmp'
call sysfiledelete d'/class_package_dir_pk.cls'
call sysrmdir d
call try 'p~findProgram'
call try 'p~findProgram(.nil)'
call try 'p~findProgram(f, 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 1
1
1
0
1
1
p~findProgram => 88.901
p~findProgram(.nil) => 88.909
p~findProgram(f, 1) => 93.902
```

### Adding

r[class.package.addclass]
`addClass` adds a class to the package under a name, as a class that is not
public, and answers the package. The name is required (`SYNTAX 88.901`), an
object with no `MAKESTRING` method, `.nil` among them, raises `88.909`, and
the class is required too (`88.901`) and must be a class (`SYNTAX 88.914`;
`.nil`, a number and an instance that is not a class do not qualify); a third
argument raises `93.902`. The checks run in that order. The name is taken in
upper case, any string is a name, and a class that is added under a name that
the package holds replaces the class that was there. The class is in
`classes`, `findClass` finds it, and the code of the package names it with
`.name`; it is not in `publicClasses`.

```rexx
src = .array~of('return 1', '::routine r', '  return .zk~id')
p = .package~new('n', src)
say (p~addClass('zk', .object) == p)
say p~findClass('ZK')~id p~classes['ZK']~id p~publicClasses~items
say p~findRoutine('R')~call
p~addClass('zk', .array)
say p~findClass('ZK')~id p~classes~items
p~addClass('has blank', .object)
say p~classes~items p~classes~hasIndex('HAS BLANK')
call try 'p~addClass("a")'
call try 'p~addClass(, .object)'
call try 'p~addClass(.nil, .object)'
call try 'p~addClass("a", .nil)'
call try 'p~addClass("a", 5)'
call try 'p~addClass("a", .object~new)'
call try 'p~addClass("a", .object, 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1
Object Object 0
Object
Array 1
2 1
p~addClass("a") => 88.901
p~addClass(, .object) => 88.901
p~addClass(.nil, .object) => 88.909
p~addClass("a", .nil) => 88.914
p~addClass("a", 5) => 88.914
p~addClass("a", .object~new) => 88.914
p~addClass("a", .object, 1) => 93.902
```

r[class.package.addpublicclass]
`addPublicClass` adds a class to the package under a name, as a public class,
with the same arguments, checks and errors as
[`addClass`](r:class.package.addclass), and answers the package. The class is
in `classes` and in `publicClasses`, and a package that imports this one
afterwards finds it with `findClass` and `findPublicClass`; one that imported
this package before does not.

```rexx
p = .package~new('n', 'return 1')
say (p~addPublicClass('shown', .array) == p)
say p~classes~items p~publicClasses~items p~publicClasses['SHOWN']~id
say p~findPublicClass('SHOWN')~id
q = .package~new('q', 'return 1')
q~addPackage(p)
say q~findClass('SHOWN')~id q~findPublicClass('SHOWN')~id
p~addPublicClass('late', .array)
say q~findClass('LATE')~isNil
q2 = .package~new('q2', 'return 1')
q2~addPackage(p)
say q2~findClass('LATE')~id
call try 'p~addPublicClass("a")'
call try 'p~addPublicClass(.nil, .object)'
call try 'p~addPublicClass("a", 5)'
call try 'p~addPublicClass("a", .object, 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1
1 1 Array
Array
Array Array
1
Array
p~addPublicClass("a") => 88.901
p~addPublicClass(.nil, .object) => 88.909
p~addPublicClass("a", 5) => 88.914
p~addPublicClass("a", .object, 1) => 93.902
```

r[class.package.addroutine]
`addRoutine` adds a routine to the package under a name, as a routine that is
not public, and answers the package. The name is required (`SYNTAX 88.901`),
an object with no `MAKESTRING` method, `.nil` among them, raises `88.909`, and
the routine is required too (`88.901`) and must be a `Routine`
(`SYNTAX 88.914`; a method, `.nil` or another object do not qualify); a third
argument raises `93.902`. The name is taken in upper case, any string is a
name, and a routine that is added under a name that the package holds replaces
the routine that was there. The routine is in `routines`, `findRoutine` finds
it, and the code of the package calls it by name; it is not in
`publicRoutines`.

```rexx
src = .array~of('return 1', '::routine caller', '  return xr()')
p = .package~new('n', src)
x = .routine~new('xr', 'return "xr called"')
say (p~addRoutine('xr', x) == p)
say (p~findRoutine('XR') == x) (p~routines['XR'] == x) p~publicRoutines~items
say p~findRoutine('CALLER')~call
p~addRoutine('xr', .routine~new('other', 'return "replaced"'))
say p~routines['XR']~call p~findRoutine('XR')~call p~routines~items
call try 'p~addRoutine("a")'
call try 'p~addRoutine(, x)'
call try 'p~addRoutine(.nil, x)'
call try 'p~addRoutine("a", .nil)'
call try 'p~addRoutine("a", .method~new("m", "nop"))'
call try 'p~addRoutine("a", x, 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1
1 1 0
xr called
replaced replaced 2
p~addRoutine("a") => 88.901
p~addRoutine(, x) => 88.901
p~addRoutine(.nil, x) => 88.909
p~addRoutine("a", .nil) => 88.914
p~addRoutine("a", .method~new("m", "nop")) => 88.914
p~addRoutine("a", x, 1) => 93.902
```

r[class.package.addpublicroutine]
`addPublicRoutine` adds a routine to the package under a name, as a public
routine, with the same arguments, checks and errors as
[`addRoutine`](r:class.package.addroutine), and answers the package. The
routine is in `routines` and in `publicRoutines`, and a package that imports
this one afterwards finds it; one that imported this package before does not.
Adding a routine with `addRoutine` under a name that was added as public
leaves the public table unchanged, so `routines` and `publicRoutines` can then
hold different routines for the name.

```rexx
p = .package~new('n', 'return 1')
x = .routine~new('xr', 'return "first"')
say (p~addPublicRoutine('pr', x) == p)
say p~routines~items p~publicRoutines~items p~publicRoutines['PR']~call
q = .package~new('q', 'return 1')
q~addPackage(p)
say q~findRoutine('PR')~call q~importedRoutines~items
p~addRoutine('pr', .routine~new('y', 'return "second"'))
say p~routines['PR']~call p~publicRoutines['PR']~call
p~addPublicRoutine('later', .routine~new('z', 'return "z"'))
say q~findRoutine('LATER')~isNil q~importedRoutines~items
q2 = .package~new('q2', 'return 1')
q2~addPackage(p)
say q2~findRoutine('LATER')~call
call try 'p~addPublicRoutine("a")'
call try 'p~addPublicRoutine(.nil, x)'
call try 'p~addPublicRoutine("a", 4)'
call try 'p~addPublicRoutine("a", x, 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1
1 1 first
first 1
second first
1 1
z
p~addPublicRoutine("a") => 88.901
p~addPublicRoutine(.nil, x) => 88.909
p~addPublicRoutine("a", 4) => 88.914
p~addPublicRoutine("a", x, 1) => 93.902
```

r[class.package.addpackage]
`addPackage` adds a package to the packages that the receiver imports, as a
`::REQUIRES` of it would, and answers the receiver. The public classes and
routines of the added package are then found by the receiver, and appear in
`importedClasses`, `importedRoutines` and `importedPackages`. The package is
required (`SYNTAX 88.901`) and must be a `Package` (`SYNTAX 88.914`; `.nil`
and other objects do not qualify). The second argument is a namespace name, a
string or an object with a `MAKESTRING` method (`88.909` for `.nil` and for an
object with none), which names the package in `namespaces` and for
`findNamespace`; a third argument raises `SYNTAX 93.902`. A package that was
already added is not added again, so it is in `importedPackages` once, and a
package may be added to itself. The namespace is taken in upper case. The
namespace may be `REXX` or the null string: `addPackage` accepts both and
`namespaces` lists them, but `findNamespace` with `REXX` still answers the
package of the library classes. A `::REQUIRES` that gives the namespace `REXX`
raises `99.944`, as [the namespace rule](r:dir.requires.namespace) says.

```rexx
src = .array~of('return 1', '::routine shown public', '  return "s"')
lib = .package~new('lib', src)
p = .package~new('n', 'return 1')
say (p~addPackage(lib) == p) p~importedPackages~items
say p~findRoutine('SHOWN')~call p~importedRoutines~items
p~addPackage(lib)
say p~importedPackages~items p~namespaces~items
p~addPackage(lib, 'ns1')
say p~importedPackages~items p~namespaces~allIndexes~makeArray~makeString
say (p~findNamespace('NS1') == lib)
p~addPackage(p)
say p~importedPackages~items
q = .package~new('q', 'return 1')
say (q~addPackage(lib, 'rexx') == q) (q~addPackage(lib, '') == q)
say q~namespaces~allIndexes~sort~makeArray~makeString('l', ',')
say (q~findNamespace('REXX') == .object~package) (q~findNamespace('') == lib)
call try 'p~addPackage'
call try 'p~addPackage(.nil)'
call try 'p~addPackage(5)'
call try 'p~addPackage(.object~new)'
call try 'p~addPackage(lib, .nil)'
call try 'p~addPackage(lib, "x", 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 1
s 1
1 0
1 NS1
1
2
1 1
,REXX
1 1
p~addPackage => 88.901
p~addPackage(.nil) => 88.914
p~addPackage(5) => 88.914
p~addPackage(.object~new) => 88.914
p~addPackage(lib, .nil) => 88.909
p~addPackage(lib, "x", 1) => 93.902
```

r[class.package.loadpackage]
`loadPackage` loads a package by name as a `::REQUIRES` of the receiver would,
adds it to the packages that the receiver imports, as
[`addPackage`](r:class.package.addpackage) does, and answers the loaded
package. The name is required (`SYNTAX 93.903`); an object with no
`MAKESTRING` method, `.nil` among them, raises `88.909`. With no second
argument the name is the name of a file, which is found, read and installed as
[the file rule](r:class.package.new.file) of `new` says, so a file that was
loaded before is not read or run again: the answer is the same package. A file
that cannot be found raises `SYNTAX 43.901`. The second argument is a source,
which is installed under the name in place of the file. A source that is not
an instance of `Array` itself, a string included, is sent `request` with the
argument `ARRAY`, once, and an `Array` is used as it is, with no message. A
string is therefore split at its line ends by
[its `makeArray`](r:class.string.makearray), and the null string is a source
of no lines, where for `new` a string is one line. The items of an array are
the lines, and a string item is its line. An object for which `request`
answers `.nil`, `.nil` itself among them, raises `SYNTAX 88.913`, and
`request` is not sent with `STRING`. A source that does not translate raises
its translation error. A package that is installed from a source is found
afterwards by its name: a `loadPackage` of that name with no source, the `new`
of `Package` with that name and a `::REQUIRES` of it answer that package and
run nothing. A `loadPackage` with a source for a name that was already loaded,
from a file or from a source, answers the package that was loaded first: the
new source is not read, translated or run, and no class or routine of it is
added. A third argument raises `93.902`. How the file is searched for is
specified in the environment chapter. The checkpoint that a security manager
sees is that of
[the `REQUIRES` rule](r:class.routine.setsecuritymanager.requires).

```rexx
f = 'class_package_load_zq.tmp'
call sysfiledelete f
call lineout f, 'say "loaded file"'
call lineout f, '::class loaded public'
call lineout f, '::routine shown public'
call lineout f, '  return "shown"'
call lineout f
p = .package~new('n', 'return 1')
l = p~loadPackage(f)
say l~class~id (l~name == qualify(f)) p~importedPackages~items
say p~findClass('LOADED')~id p~findRoutine('SHOWN')~call
say (p~loadPackage(f) == l) p~importedPackages~items
say (.package~new(f) == l)
s = p~loadPackage('class_package_src_zq.cls', 'say "from source"; return 1')
say s~name s~sourceSize p~importedPackages~items
a = p~loadPackage('class_package_arr_zq.cls', .array~of('say "array source"'))
say a~name a~source~items
call sysfiledelete f
call try 'p~loadPackage'
call try 'p~loadPackage(.nil)'
call try 'p~loadPackage("class_package_load_none.tmp")'
call try 'p~loadPackage("class_package_x1.cls", .nil)'
call try 'p~loadPackage("class_package_x2.cls", .object~new)'
call try 'p~loadPackage("class_package_x3.cls", "say (")'
call try 'p~loadPackage("class_package_x4.cls", "nop", 1)'
call try 'p~loadPackage("class_package_x5.cls", .mk~new)~source[1]'
call try 'p~loadPackage("class_package_x6.cls", .src~new(.nil, "nop"))'
call try 'p~loadPackage("class_package_x7.cls", .ms~new)'
call try 'p~loadPackage("class_package_x8.cls", .ssub~new("nop"))~source[1]'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class mk
::method makearray
  say 'mk makearray'
  return .array~of('return 1')
::class src
::method init
  expose arr str
  use arg arr, str
::method request
  expose arr str
  say 'request' arg(1)
  if arg(1) == 'ARRAY' then return arr
  return str
::class ms
::method makestring
  say 'ms makestring'
  return 'return 1'
::class ssub subclass string
```

```output
loaded file
Package 1 1
LOADED shown
1 1
1
from source
class_package_src_zq.cls 1 2
array source
class_package_arr_zq.cls 1
p~loadPackage => 93.903
p~loadPackage(.nil) => 88.909
p~loadPackage("class_package_load_none.tmp") => 43.901
p~loadPackage("class_package_x1.cls", .nil) => 88.913
p~loadPackage("class_package_x2.cls", .object~new) => 88.913
p~loadPackage("class_package_x3.cls", "say (") => 35.1
p~loadPackage("class_package_x4.cls", "nop", 1) => 93.902
mk makearray
p~loadPackage("class_package_x5.cls", .mk~new)~source[1] => return 1
request ARRAY
p~loadPackage("class_package_x6.cls", .src~new(.nil, "nop")) => 88.913
p~loadPackage("class_package_x7.cls", .ms~new) => 88.913
p~loadPackage("class_package_x8.cls", .ssub~new("nop"))~source[1] => nop
```

```rexx
p = .package~new('n', 'nop')
body = 'say "first body"; ::class first public'
a = p~loadPackage('class_package_found_zq.cls', body)
src = .array~of('say "second body"', '::class second public')
b = p~loadPackage('class_package_found_zq.cls', src)
say (a == b) p~findClass('SECOND')~isNil p~importedPackages~items
say (a == p~loadPackage('class_package_found_zq.cls'))
say (a == .package~new('class_package_found_zq.cls'))
src = .array~of('nop', '::requires "class_package_found_zq.cls"')
q = .package~new('q', src)
say (q~importedPackages[1] == a)
```

```output defect=#145
first body
1 1 1
1
1
1
```

```rexx
p = .package~new('n', 'nop')
say p~loadPackage('class_package_empty_zq.cls', '')~sourceSize
```

```output defect=#147
0
```

```rexx
p = .package~new('n', 'nop')
src = 'nop' || '0A'x || 'return 1'
say p~loadPackage('class_package_split_zq.cls', src)~sourceSize
```

```output defect=#148
2
```

r[class.package.loadpackage.sends]
`loadPackage` sends a source that is not an instance of `Array` itself the
message `request` with the argument `ARRAY`, once, and uses the array that is
answered; an `Array` is sent nothing, and the array answered is sent nothing
either. An instance of a subclass of `Array` is sent `request`, and what that
sends in turn is stated by [the `request` rule](r:class.object.request.sends).
The message is sent after the name has been read and before the source is
translated. The example uses an instance of an `Array` subclass, an object
that answers an array of its own, and an `Array`.

```rexx
p = .package~new('n', 'nop')
q = p~loadPackage('class_package_sends_a_zq.cls', .arr~of('nop', 'return 1'))
say q~source~items
q = p~loadPackage('class_package_sends_b_zq.cls', .own~new)
say q~source[1]
q = p~loadPackage('class_package_sends_c_zq.cls', .array~of('nop'))
say q~source[1]
::class arr subclass array
::method request
  say 'arr request' arg(1)
  forward class (super)
::method makearray
  say 'arr makearray'
  forward class (super)
::class own
::method request
  say 'own request' arg(1)
  return .array~of('return 2')
```

```output
arr request ARRAY
arr makearray
2
own request ARRAY
return 2
nop
```

r[class.package.loadlibrary]
`loadLibrary` loads a native library by name as a `::REQUIRES ... LIBRARY`
does, and answers `1` when the library was loaded and `0` when it was not. A
library that cannot be loaded does not raise an error. The name is required
(`SYNTAX 88.901`), an object with no `MAKESTRING` method, `.nil` among them,
raises `88.909`, and a second argument raises `93.902`. The library is loaded
for the interpreter, not for the package, and adds no routine to
`importedRoutines`. Where libraries are searched for, and whether the name is
case sensitive, is specified in the environment chapter.

```rexx
p = .package~new('n', 'return 1')
say p~loadLibrary('rexxutil') p~loadLibrary('rexxutil')
say p~loadLibrary('nosuchlibraryzq') p~loadLibrary('')
say p~importedRoutines~items p~importedPackages~items
call try 'p~loadLibrary'
call try 'p~loadLibrary(.nil)'
call try 'p~loadLibrary("rexxutil", 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 1
0 0
0 0
p~loadLibrary => 88.901
p~loadLibrary(.nil) => 88.909
p~loadLibrary("rexxutil", 1) => 93.902
```

### Settings

r[class.package.digits]
`digits` answers the initial `NUMERIC DIGITS` setting of the code of the
package, as a whole number string: `9` unless the package gave another by an
`::OPTIONS`, by [`options`](r:class.package.options.settings) or by
[`defaultOptions`](r:class.package.defaultoptions). The method takes no
argument (`SYNTAX 93.902`).

```rexx
p = .package~new('n', .array~of('return 1', '::options digits 12'))
say p~digits .package~new('m', 'return 1')~digits
p~options('Digits', 20)
say p~digits
say .object~package~digits
call try 'p~digits(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
12 9
20
9
p~digits(1) => 93.902
```

r[class.package.form]
`form` answers the initial `NUMERIC FORM` setting of the code of the package,
`SCIENTIFIC` unless the package gave `ENGINEERING`. The method takes no
argument (`SYNTAX 93.902`).

```rexx
p = .package~new('n', .array~of('return 1', '::options form engineering'))
say p~form .package~new('m', 'return 1')~form
p~options('Form', 'Scientific')
say p~form
call try 'p~form(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
ENGINEERING SCIENTIFIC
SCIENTIFIC
p~form(1) => 93.902
```

r[class.package.fuzz]
`fuzz` answers the initial `NUMERIC FUZZ` setting of the code of the package,
`0` unless the package gave another. The method takes no argument
(`SYNTAX 93.902`).

```rexx
p = .package~new('n', .array~of('return 1', '::options fuzz 2'))
say p~fuzz .package~new('m', 'return 1')~fuzz
p~options('Fuzz', 3)
say p~fuzz
call try 'p~fuzz(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
2 0
3
p~fuzz(1) => 93.902
```

r[class.package.trace]
`trace` answers the initial trace setting of the code of the package as one
letter, the first letter of the setting: `N` for `NORMAL`, which a package
has unless it gave another, and `A`, `C`, `E`, `F`, `I`, `L`, `O` and `R` for
`ALL`, `COMMANDS`, `ERROR`, `FAILURE`, `INTERMEDIATES`, `LABELS`, `OFF` and
`RESULTS`. The package of the library classes answers the null string. The
method takes no argument (`SYNTAX 93.902`).

```rexx
p = .package~new('n', 'return 1')
say p~trace
p~options('Trace', 'Results')
say p~trace
p~options('Trace', 'Off')
say p~trace
say '['.object~package~trace']'
call try 'p~trace(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
N
R
O
[]
p~trace(1) => 93.902
```

r[class.package.options]
`options` reads and changes the settings of the package that the `::OPTIONS`
directive gives, while the program runs. With no argument it answers the
current settings as a string in the form of an `::OPTIONS` directive. The
string starts with
`::OPTIONS DIGITS 9 FORM SCIENTIFIC FUZZ 0 NUMERIC NOINHERIT` for a package
that has the initial settings, goes on with the six condition options `ERROR`,
`FAILURE`, `LOSTDIGITS`, `NOSTRING`, `NOTREADY` and `NOVALUE`, each followed
by `CONDITION` or `SYNTAX`, and ends with `PROLOG` or `NOPROLOG` and `TRACE`
and its setting, so the initial string ends with `PROLOG TRACE NORMAL`. With
one argument, the option, it answers the current setting of that option, and
with a second argument it sets the option to that value and answers the
previous setting. More than two arguments raise `SYNTAX 93.902`. The option
name is a string, an object with no `MAKESTRING` method, `.nil` among them,
raises `88.909`, and a name that is not an option raises `SYNTAX 93.914`.
[The names rule](r:class.package.options.names) says which strings name which
option, and [the settings rule](r:class.package.options.settings) says what
the settings are. A change takes effect for the code of the package that
starts afterwards: a routine or a method of the package that runs next runs
under the new settings.

```rexx
src = .array~of('return 1', '::routine rd')
src~append('  return digits() form() fuzz() trace()')
p = .package~new('n', src)
say p~options~subword(1, 7)
rd = p~findRoutine('RD')
say rd~call
say p~options('Digits', 20) p~options('Digits')
say rd~call
say p~options('Form', 'Engineering') p~options('Form')
say rd~call
say p~options('Trace', 'Results') p~options('Trace')
call try 'p~options("Zzz")'
call try 'p~options(.nil)'
call try 'p~options("Digits", 5, 6)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
::OPTIONS DIGITS 9 FORM SCIENTIFIC FUZZ 0
9 SCIENTIFIC 0 N
9 20
20 SCIENTIFIC 0 N
SCIENTIFIC ENGINEERING
20 ENGINEERING 0 N
NORMAL RESULTS
p~options("Zzz") => 93.914
p~options(.nil) => 88.909
p~options("Digits", 5, 6) => 93.902
```

r[class.package.options.names]
The option name is matched by its first letters, in any case, and the letters
after them are ignored. The names are `Digits` (`D`), `Form` (`FO`), `Fuzz`
(`FU`), `Trace` (`T`), `Numeric` (`NU`), `Error` (`E`), `Failure` (`FA`),
`Lostdigits` (`L`), `NOString` (`NOS`), `NOTready` (`NOT`), `NOValue` (`NOV`),
`Prolog` (`P`), and the names that are not settings, `All` (`A`),
`InitialPackageSettings` (`I`), `Reset` (`R`), `SetPackageSettings` (`S`) and
`X` (`X`). A name that stops after `F` or `N`, or after `NO`, is not enough to
choose, and raises `SYNTAX 93.914`, as the other strings that are not a name
do. A name that has more letters than the ones that choose is taken by the
choosing ones: `Digitz` names `Digits`.

```rexx
p = .package~new('n', 'return 1')
names = 'D d DI Digitz FO FU T Tr NU Num E Err FA Fail L Lo NOS NOT NOV P'
do n over names~makeArray(' ')
  say n~left(6) p~options(n)
end
call try 'p~options("F")'
call try 'p~options("N")'
call try 'p~options("NO")'
call try 'p~options("")'
call try 'p~options("Digit5")'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
D      9
d      9
DI     9
Digitz 9
FO     SCIENTIFIC
FU     0
T      NORMAL
Tr     NORMAL
NU     NOINHERIT
Num    NOINHERIT
E      CONDITION
Err    CONDITION
FA     CONDITION
Fail   CONDITION
L      CONDITION
Lo     CONDITION
NOS    CONDITION
NOT    CONDITION
NOV    CONDITION
P      PROLOG
p~options("F") => 93.914
p~options("N") => 93.914
p~options("NO") => 93.914
p~options("") => 93.914
p~options("Digit5") => 9
```

r[class.package.options.settings]
The settings that `options` reads and sets are these, each with the values
that the second argument may take, matched by their first letters and in any
case: `Digits`, a positive whole number, and `Fuzz`, a whole number that is
not negative and is less than `Digits`; `Form`, `Scientific` or `Engineering`;
`Numeric`, `Inherit` or `Noinherit`; `Error`, `Failure`, `Lostdigits`,
`NOString`, `NOTready` and `NOValue`, each `Condition` or `Syntax`, which is
what the directive [`::OPTIONS`](r:dir.options.condition) gives; `Prolog`,
`Prolog` or `Noprolog`; and `Trace`, one of the settings of
[the trace rule](r:dir.options.trace). The answer for an option is the string
of its setting in upper case, `INHERIT`, `NOINHERIT`, `CONDITION`, `SYNTAX`,
`PROLOG` or `NOPROLOG`, and the full name of a trace setting; a number is a
decimal string. The second argument raises an error when it is not an
acceptable value: `SYNTAX 93.900` for the null string, `88.909` for `.nil`,
`93.905` for a `Digits` value that is not a whole number, `93.907` when it is
a whole number that is less than 1, `93.906` for a `Fuzz` value that is not a
whole number that is not negative, `26.900` when a `Fuzz` value would not be
less than `Digits`, and `33.1` when a `Digits` value would not be greater than
`Fuzz` (the numeric setting that would be invalid is refused), `93.914` for
another value of an option that takes words, and `24.1` for a trace setting
that [the directive](r:dir.options.trace) refuses, a number included. The
`All` setting sets the six condition options at once, to `Condition` or
`Syntax`, and answers the previous six as a string of six names and values,
such as `ERROR CONDITION FAILURE CONDITION` and so on; `All` cannot be read
(`SYNTAX 93.903`).

```rexx
p = .package~new('n', 'return 1')
say p~options('Digits', 12)~right(3) p~options('Digits', ' 7.0 ')~right(3)
say p~options('Digits')~right(3) p~options('Digits', 1E1)~right(3)
say p~options('Fuzz', 3) p~options('Fuzz', 9)
p~options('Fuzz', 0)
say p~options('Form', 'e') p~options('Form', 'Scientific')
say p~options('Numeric', 'Inherit') p~options('Numeric', 'No')
say p~options('Error', 'syntax') p~options('Error', 'C')
say p~options('Prolog', 'Noprolog') p~options('Prolog', 'P')
say p~options('Trace', 'Results') p~options('Trace', 'Off')
say p~options('All', 'Syntax')~subword(1, 4)
say p~options('All', 'Condition')~subword(1, 4)
say p~options('All', 'Syntax')~words p~options('Novalue')
call try 'p~options("Digits", "abc")'
call try 'p~options("Digits", 0)'
call try 'p~options("Digits", 1.5)'
call try 'p~options("Digits", "")'
call try 'p~options("Digits", .nil)'
call try 'p~options("Digits", 3)'
call try 'p~options("Fuzz", -1)'
call try 'p~options("Fuzz", 1.5)'
call try 'p~options("Fuzz", 5)'
p~~options('Digits', 20)
p~~options('Fuzz', 5)
call try 'p~options("Digits", 5)'
call try 'p~options("Digits", 4)'
call try 'p~options("Form", "zzz")'
call try 'p~options("Error", "zzz")'
call try 'p~options("Numeric", "zzz")'
call try 'p~options("Prolog", "zzz")'
call try 'p~options("Trace", "zzz")'
call try 'p~options("Trace", "")'
call try 'p~options("All")'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
  9  12
  7   7
0 3
SCIENTIFIC ENGINEERING
NOINHERIT INHERIT
CONDITION SYNTAX
PROLOG NOPROLOG
NORMAL RESULTS
ERROR CONDITION FAILURE CONDITION
ERROR SYNTAX FAILURE SYNTAX
12 SYNTAX
p~options("Digits", "abc") => 93.905
p~options("Digits", 0) => 93.907
p~options("Digits", 1.5) => 93.905
p~options("Digits", "") => 93.900
p~options("Digits", .nil) => 88.909
p~options("Digits", 3) => 10
p~options("Fuzz", -1) => 93.906
p~options("Fuzz", 1.5) => 93.906
p~options("Fuzz", 5) => 26.900
p~options("Digits", 5) => 33.1
p~options("Digits", 4) => 33.1
p~options("Form", "zzz") => 93.914
p~options("Error", "zzz") => 93.914
p~options("Numeric", "zzz") => 93.914
p~options("Prolog", "zzz") => 93.914
p~options("Trace", "zzz") => 24.1
p~options("Trace", "") => 93.900
p~options("All") => 93.903
```

r[class.package.options.special]
Four names of `options` take a string of the directive form and not a setting.
`X` takes no second argument and answers the options that the package gave
explicitly by an `::OPTIONS` directive, as their names separated by blanks, in
the order `DIGITS`, `FORM`, `FUZZ`, `NUMERIC`, `ERROR`, `FAILURE`,
`LOSTDIGITS`, `NOSTRING`, `NOTREADY`, `NOVALUE`, `PROLOG` or `NOPROLOG`, and
`TRACE`, and answers the null string when it gave none; a setting that
`options` or `defaultOptions` gave is not explicit. `InitialPackageSettings`
takes no second argument and answers the settings the package had after it was
made, in the form of the directive. `Reset` takes no second argument, answers
the current settings in that form, and sets every option back to what
`InitialPackageSettings` answers. `SetPackageSettings` takes the directive
text as its argument, which is required (`SYNTAX 93.901`) and must begin with
`::OPTIONS` (`SYNTAX 93.900` otherwise); it sets the options that the text
gives, resets every other option to its initial value (`DIGITS 9`,
`FORM SCIENTIFIC`, `FUZZ 0` and so on to `TRACE NORMAL`, as the initial string
of [`options`](r:class.package.options) lists them), and answers the previous
settings. A second argument on the three raises `93.902`.

```rexx
p = .package~new('n', .array~of('return 1', '::options digits 15 trace r'))
say p~options('X')
p~options('Digits', 20)
say p~options('Digits') '|' p~options('X')
say p~options('InitialPackageSettings')~subword(1, 3)
say p~options('Reset')~subword(1, 3)
say p~options('Digits') p~options('Trace')
say .package~new('m', 'return 1')~options('X')'|'
say p~options('SetPackageSettings', '::OPTIONS DIGITS 7 FUZZ 2')~subword(1, 3)
say p~options~subword(1, 7) p~options('Trace')
say p~options('SetPackageSettings', '::OPTIONS')~subword(1, 3)
say p~options~subword(1, 7)
call try 'p~options("SetPackageSettings")'
call try 'p~options("SetPackageSettings", "DIGITS 7")'
call try 'p~options("SetPackageSettings", "")'
call try 'p~options("X", 1)'
call try 'p~options("Reset", 1)'
call try 'p~options("InitialPackageSettings", 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
DIGITS TRACE
20 | DIGITS TRACE
::OPTIONS DIGITS 15
::OPTIONS DIGITS 20
15 RESULTS
|
::OPTIONS DIGITS 15
::OPTIONS DIGITS 7 FORM SCIENTIFIC FUZZ 2 NORMAL
::OPTIONS DIGITS 7
::OPTIONS DIGITS 9 FORM SCIENTIFIC FUZZ 0
p~options("SetPackageSettings") => 93.901
p~options("SetPackageSettings", "DIGITS 7") => 93.900
p~options("SetPackageSettings", "") => 93.900
p~options("X", 1) => 93.902
p~options("Reset", 1) => 93.902
p~options("InitialPackageSettings", 1) => 93.902
```

r[class.package.setsecuritymanager]
`setSecurityManager` gives the package a security manager, or with no argument
removes the one it has, and answers `1`. It is a protected method. The manager
is any object, `.nil` as well, which is an object like another and is sent the
checkpoints, as [the Routine rule](r:class.routine.setsecuritymanager) says;
it is not checked when it is set. A second argument raises `SYNTAX 93.902`.
The checkpoints that [the checkpoint
rules](r:class.routine.setsecuritymanager.checkpoints) name are offered to the
manager for the code of the package that runs after the manager was set, the
routines and methods of the package included, and not for the code of other
packages.

```rexx
src = .array~of('return 1', '::routine agent')
src~append('  address system "echo hidden"')
src~append('  return "done"')
p = .package~new('n', src)
say p~setSecurityManager(.guard~new)
say p~findRoutine('AGENT')~call
say p~setSecurityManager
say p~findRoutine('AGENT')~call
say p~setSecurityManager(.nil) p~setSecurityManager
call try 'p~setSecurityManager(.guard~new, 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class guard
::method command
  use arg info
  say 'manager saw' info~command
  info~rc = 0
  return 1
```

```output
1
manager saw echo hidden
done
1
hidden
done
1 1
p~setSecurityManager(.guard~new, 1) => 93.902
```

## Deviations

r[class.package.deviation.41]
A source that is an encoded program image whose decoded bytes lack the tag of
an image raises `SYNTAX 3.903` from `new`, as [the rule of
`Method`](r:class.method.deviation.41) says; ooRexx translates the partly
decoded bytes as source.

```rexx
img = .array~of('#!/usr/bin/env rexx', '/**/@REXX@', 'c2F5ICdkZWNvZGVkJw==')
signal on syntax name bad
p = .package~new('n', img)
say 'made'
exit
bad: say condition('o')~code
```

```output
3.903
```

r[class.package.deviation.109]
An instance of a subclass of `String` that is the name of `new`, of
`addRoutine`, `addPublicRoutine`, `addClass` or `addPublicClass`, or the
source of `new`, is read by its own characters, and `MAKESTRING` and `STRING`
are not sent to it ([Deviation 109](r:class.string.deviation.109)). ooRexx
reads it through its `MAKESTRING` method. In the example `MAKESTRING` answers
another name.

```rexx
p = .package~new(.name~new('Foo'), 'nop')
say p~name
p~addRoutine(.name~new('Bar'), .routine~new('r', 'return 1'))
say p~findRoutine('BAR')~isNil
say .package~new('q', .name~new('nop'))~source[1]
::class name subclass string
::method makestring
  return 'other'
```

```output defect=#140
Foo
0
nop
```
