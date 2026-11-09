---
element: "Routine"
kind: class
rexxref: "Builtin Classes > Fundamental Classes > Routine Class; Builtin Classes > Fundamental Classes > Routine Class > new (Class Method); Builtin Classes > Fundamental Classes > Routine Class > newFile (Class Method); Builtin Classes > Fundamental Classes > Routine Class > loadExternalRoutine (Class method); Builtin Classes > Fundamental Classes > Routine Class > []; Builtin Classes > Fundamental Classes > Routine Class > annotation; Builtin Classes > Fundamental Classes > Routine Class > annotations; Builtin Classes > Fundamental Classes > Routine Class > call; Builtin Classes > Fundamental Classes > Routine Class > callWith; Builtin Classes > Fundamental Classes > Routine Class > package; Builtin Classes > Fundamental Classes > Routine Class > setSecurityManager; Builtin Classes > Fundamental Classes > Routine Class > source; The Security Manager"
deviations: [41, 109, 115]
---

# Routine

A `Routine` object is a piece of code that is called with arguments and
answers a value, as a routine of a program is. A `::ROUTINE` directive makes
one for each directive, and a program makes one of its own from source text
with `new`, from a file with `newFile`, or from a native library with
`loadExternalRoutine`. A routine object is called by the messages `call`,
`callWith` and `[]`, and every call is a call of an external routine: the code
runs in a variable pool of its own, sees nothing of its caller's variables,
and finds classes and routines through its package.

The page specifies the class methods that make a routine, the messages that
call it and ask about it, and the security manager that a routine can be put
under. The routine that a directive makes is specified by
[`::ROUTINE`](../directives/routine.md), the annotations by
[`::ANNOTATE`](../directives/annotate.md), and the instruction that calls a
routine by name by [CALL](../instructions/call.md). A routine is made from
source in the way a method is, so the rules of [the Method
page](method.md) for the source and the context apply to `new` and `newFile`
here, and this page states what is its own. The search for a file or a library
that `newFile` and `loadExternalRoutine` start is specified in the
environment chapter; the examples here name a file that they create
themselves, or a library that every installation has.

The examples that show an error use a routine `try` that sends the message in
its argument and prints the answer, or the error code when the message raises
one.

## The class

r[class.routine.creation]
A routine object made by `new` or `newFile` is called by sending it `call`,
and holds the source it was made from. It has no annotations and belongs to a
package of its own, as [the context rule](r:class.method.new.context) says for
a method. A `Routine` object is an ordinary object otherwise, so it
understands the methods of [`Object`](object.md) and has the default name
`a Routine`. The class `Routine` has the superclass `Object` and no mixin
class.

```rexx
r = .routine~new('adder', 'use arg a, b; return a + b')
say r~class~id r~defaultName r~call(2, 3)
say r~source~items r~annotations~items r~package~class~id r~package~name
say .routine~superclass~id .routine~superclasses~items
```

```output
Routine a Routine 5
1 0 Package adder
Object 1
```

## Class methods

### new

r[class.routine.new]
`new` makes a routine from a name, a source and a context, with the same
arguments, the same checks, in the same order, and the same errors as
[the `new` of `Method`](r:class.method.new): the name and the source are
required (`SYNTAX 88.901`), a name with no `MAKESTRING` method, `.nil` among
them, raises `88.909`, a context that is not `PROGRAMSCOPE`, a `Method`, a
`Routine` or a `Package` raises `40.904`, and a source that does not translate
raises its translation error, and the code does not run. The source is read as
[the source rule](r:class.method.new.source) says, with the messages of
[the sends rule](r:class.method.new.sends), and the context decides what the
routine finds as [the context rule](r:class.method.new.context) says. The
arguments from the fourth on are passed to `init` of the new object, and the
class that received `new` is the class of the answer, as for a method. The
name is the name of the routine's package.

```rexx
r = .routine~new('who', 'return "I am" .context~executable~class~id')
say r~call r~package~name
d = .routine~new('d', 'return helper() .here~class~id')
say d~call
call try '.routine~new()'
call try '.routine~new("n")'
call try '.routine~new(.nil, "nop")'
call try '.routine~new("n", .nil)'
call try '.routine~new("n", "nop", "other")'
call try '.routine~new("n", "say (")'
call try '.routine~new("n", "nop", "PROGRAMSCOPE", 1)'
call try '.routine~new("n", .array~of("return 7"))~call'
call try '.routine~new("n", "return" || "0A"x)'
say .mine~new('n', 'nop', , 'a', 'b')~class~id
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::routine helper
  return 'help'
::class here
::class mine subclass routine
::method init
  say 'init' arg() arg(1) arg(2)
```

```output
I am Routine who
help Class
.routine~new() => 88.901
.routine~new("n") => 88.901
.routine~new(.nil, "nop") => 88.909
.routine~new("n", .nil) => 93.961
.routine~new("n", "nop", "other") => 40.904
.routine~new("n", "say (") => 35.1
.routine~new("n", "nop", "PROGRAMSCOPE", 1) => 93.902
.routine~new("n", .array~of("return 7"))~call => 7
.routine~new("n", "return" || "0A"x) => 13.1
init 2 a b
MINE
```

```rexx
call try '.routine~new("n", .nil, 7)'
call try '.routine~new("n", "nop", 7)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output defect=#143
.routine~new("n", .nil, 7) => 93.961
.routine~new("n", "nop", 7) => 40.904
```

r[class.routine.new.sends]
`new` sends the messages that [the sends rule of
`Method`](r:class.method.new.sends) states, in the same order: `request` with
`ARRAY` and then with `STRING` to a source that is not a string and not an array
(the line is a string answer itself, and any other answer's `objectName`), and,
when the routine has been made, `init` with the arguments of `new` from the
fourth on. An answer to `request` with `ARRAY` that is neither an array nor
`.nil` raises `SYNTAX 88.913`, and no `request` with `STRING` follows it
([Deviation 115](r:class.routine.deviation.115)).

```rexx
call try '.routine~new("n", .src~new(.array~of("return 1"), .nil))~call'
call try '.routine~new("n", .src~new(.nil, "return 2"))~call'
call try '.routine~new("n", .src~new(.nil, .nil))'
say .mine~new('n', 'return 5', 'PROGRAMSCOPE', 'a', 'b')~class~id
say .mine~new('n', 'return 5')~call
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
  expose arr str
  use arg arr, str
::method request
  expose arr str
  say 'request' arg(1)
  if arg(1) == 'ARRAY' then return arr
  return str
::class mine subclass routine
::method init
  say 'init' arg() arg(1) arg(2)
```

```output
request ARRAY
.routine~new("n", .src~new(.array~of("return 1"), .nil))~call => 1
request ARRAY
request STRING
.routine~new("n", .src~new(.nil, "return 2"))~call => 2
request ARRAY
request STRING
.routine~new("n", .src~new(.nil, .nil)) => 93.961
init 2 a b
MINE
init 0  
5
```

### newFile

r[class.routine.newfile]
`newFile` makes a routine from the lines of a file, with the arguments, the
checks, their order and the errors of
[the `newFile` of `Method`](r:class.method.newfile): the file name and an
optional context, no third argument (`SYNTAX 93.902`), `.nil` or an object
with no `MAKESTRING` method as the name raises `88.909`, a context that does
not fit raises `40.904` before the file is looked for, and a file that cannot
be read raises `SYNTAX 3.1`. The routine's package is named by the file name
as it was given. Its source is the lines of the file before the first
directive, except that a first line that begins with `#!` is not part of it,
and the directives of the file still make the classes and routines of its
package, which its code can name. `call` runs the lines before the first
directive. How the file is searched for is specified in the environment
chapter. `newFile` sends `init` with no argument to the new object, and a
subclass of `Routine` makes instances of itself.

```rexx
f = 'class_routine_newfile_zq.tmp'
call sysfiledelete f
call lineout f, 'say "file routine" arg(1)'
call lineout f, 'return .fromfile~new~hi'
call lineout f, '::class fromfile'
call lineout f, '::method hi'
call lineout f, '  return "hi from the file"'
call lineout f
r = .routine~newFile(f)
say r~class~id r~source~items (r~package~name == f)
say r~package~classes~items r~package~routines~items
say r~call('x')
say .mine~newFile(f)~class~id
call sysfiledelete f
call try '.routine~newFile("class_routine_newfile_none.tmp")'
call try '.routine~newFile()'
call try '.routine~newFile(.nil)'
call try '.routine~newFile(f, "PROGRAMSCOPE", 1)'
call try '.routine~newFile("class_routine_newfile_none.tmp", "bad")'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class mine subclass routine
::method init
  say 'init' arg()
```

```output
Routine 2 1
1 0
file routine x
hi from the file
init 0
MINE
.routine~newFile("class_routine_newfile_none.tmp") => 3.1
.routine~newFile() => 88.901
.routine~newFile(.nil) => 88.909
.routine~newFile(f, "PROGRAMSCOPE", 1) => 93.902
.routine~newFile("class_routine_newfile_none.tmp", "bad") => 40.904
```

```rexx
f = 'class_routine_shebang_zq.tmp'
call sysfiledelete f
call lineout f, '#!/usr/bin/rexx'
call lineout f, 'return "after the first line"'
call lineout f
r = .routine~newFile(f)
say r~source~items r~source[1]
say r~call
call sysfiledelete f
```

```output defect=#144
1 return "after the first line"
after the first line
```

r[class.routine.newfile.sends]
`newFile` sends the new object the message `init` with no argument, after the
file has been read and translated, and a file name that is an object with a
`MAKESTRING` method is sent that message once, before `init`; no other message
is sent, as [the rule of `Method`](r:class.method.newfile.sends) says.

```rexx
f = 'class_routine_newfilesends_zq.tmp'
call sysfiledelete f
call lineout f, 'return 1'
call lineout f
say .mine~newFile(f)~class~id
say .mine~newFile(.named~new(f))~class~id
call sysfiledelete f
::class mine subclass routine
::method init
  say 'init' arg()
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
init 0
MINE
makestring
init 0
MINE
```

### loadExternalRoutine

r[class.routine.loadexternalroutine]
`loadExternalRoutine` makes a routine object whose code is a routine of a
native library, the same kind of routine that
[`::ROUTINE EXTERNAL`](r:dir.routine.external) declares. It takes a name and a
descriptor, with the checks, the descriptor form and the errors of
[`loadExternalMethod`](r:class.method.loadexternalmethod): the word `LIBRARY`
in any case, the library, and optionally the entry in it, which is the name of
the routine when it is left out. A third argument raises `SYNTAX 93.902`, a
missing argument `88.901`, a `.nil` argument or one with no `MAKESTRING`
method `88.909` and a descriptor of another form `99.917`. When the library
cannot be loaded or has no entry of that name, the answer is `.nil`. The
answer is an instance of `Routine` also when a subclass received the message,
and no `init` is sent. The routine can be called, with the arguments that the
library routine takes, and its `source` is an empty array and its `package` is
`.nil`. Where the library is searched for, and whether its name is case
sensitive, is specified in the environment chapter.

```rexx
p = .routine~loadExternalRoutine('pi', 'LIBRARY rxmath rxcalcpi')
say p~class~id p~call(5) p~callWith(.array~of(8)) p[3]
say p~source~items p~package~isNil
q = .routine~loadExternalRoutine('rxcalcpi', 'library rxmath')
say q~call(4)
say .routine~loadExternalRoutine('x', 'LIBRARY rxmath')~isNil
say .routine~loadExternalRoutine('x', 'LIBRARY rxmath nosuchentry')~isNil
say .routine~loadExternalRoutine('x', 'LIBRARY nosuchlibraryzq x')~isNil
say .mine~loadExternalRoutine('pi', 'LIBRARY rxmath rxcalcpi')~class~id
call try '.routine~loadExternalRoutine("x", "REGISTERED rxmath")'
call try '.routine~loadExternalRoutine("x", "LIBRARY")'
call try '.routine~loadExternalRoutine("x", "")'
call try '.routine~loadExternalRoutine("x")'
call try '.routine~loadExternalRoutine()'
call try '.routine~loadExternalRoutine(.nil, "LIBRARY rxmath")'
call try '.routine~loadExternalRoutine("x", .nil)'
call try '.routine~loadExternalRoutine("x", "LIBRARY rxmath x", 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class mine subclass routine
::method init
  say 'init'
```

```output
Routine 3.1416 3.1415927 3.14
0 1
3.142
1
1
1
Routine
.routine~loadExternalRoutine("x", "REGISTERED rxmath") => 99.917
.routine~loadExternalRoutine("x", "LIBRARY") => 99.917
.routine~loadExternalRoutine("x", "") => 99.917
.routine~loadExternalRoutine("x") => 88.901
.routine~loadExternalRoutine() => 88.901
.routine~loadExternalRoutine(.nil, "LIBRARY rxmath") => 88.909
.routine~loadExternalRoutine("x", .nil) => 88.909
.routine~loadExternalRoutine("x", "LIBRARY rxmath x", 1) => 93.902
```

## Methods

### Calling the routine

r[class.routine.call]
`call` runs the routine with its arguments and answers what it returns. The
arguments are the arguments of the message, and the argument count is the
position of the last one that is not [omitted](r:conv.arg.omitted): an
omitted place before it stays omitted for `ARG` and `use arg`, and omitted
places at the end add nothing. The code runs as the code of an external
routine does: it has variables of its own and cannot see those of the program
that called it, the classes and routines it names are those of its package,
and `exit` ends the routine and answers the value it gives. A routine that
returns no value answers no value, so using the answer of its `call` as a term
raises `SYNTAX 91.999`. A condition that the routine raises and does not trap
reaches the sender of `call` as it would reach the caller of an external
routine; the example shows `42.3` from a division by zero. A routine that was
made by `loadExternalRoutine` calls the library routine with the arguments.

```rexx
r = .routine~new('n', 'return arg() "<"arg(1)"><"arg(2)"><"arg(3)">"')
call try 'r~call'
call try 'r~call(1)'
call try 'r~call(, 2)'
call try 'r~call(1, , 3)'
call try 'r~call(1, )'
call try 'r~call(, )'
call try '.routine~new("n", "return arg() arg(1, ""e"")")~call(, "x")'
x = 'caller variable'
call try '.routine~new("n", "return symbol(""X"") var(""X"")")~call'
call try '.routine~new("n", "x = 5; return x")~call'
say x
call try '.routine~new("n", "exit 6")~call'
call try '.routine~new("n", "return 1/0")~call'
call try '.routine~new("n", "nop")~call'
call try '.routine~new("n", "return")~call'
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
r~call => 0 <><><>
r~call(1) => 1 <1><><>
r~call(, 2) => 2 <><2><>
r~call(1, , 3) => 3 <1><><3>
r~call(1, ) => 1 <1><><>
r~call(, ) => 0 <><><>
.routine~new("n", "return arg() arg(1, ""e"")")~call(, "x") => 2 0
.routine~new("n", "return symbol(""X"") var(""X"")")~call => LIT 0
.routine~new("n", "x = 5; return x")~call => 5
caller variable
.routine~new("n", "exit 6")~call => 6
.routine~new("n", "return 1/0")~call => 42.3
.routine~new("n", "nop")~call => 91.999
.routine~new("n", "return")~call => 91.999
```

r[class.routine.brackets]
`[]` is the same message as `call` and takes the same arguments: the
expression `r[1, 2]` and the message `r~'[]'(1, 2)` and the message
`r~call(1, 2)` answer the same, with the same omitted places and the same
errors. The routine has both methods.

```rexx
r = .routine~new('n', 'return arg() "<"arg(1)"><"arg(2)">"')
say r[1, 2] r~'[]'(1, 2) r~call(1, 2)
say r[, 5] r~'[]'(, 5)
say r[] r~'[]'
say r~hasMethod('[]') r~hasMethod('CALL')
call try '.routine~new("n", "return 1/0")[]'
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
2 <1><2> 2 <1><2> 2 <1><2>
2 <><5> 2 <><5>
0 <><> 0 <><>
1 1
.routine~new("n", "return 1/0")[] => 42.3
```

r[class.routine.callwith]
`callWith` runs the routine as `call` does with the arguments that its one
argument holds. The argument is required (`SYNTAX 93.903`) and a second raises
`93.902`. An instance of `Array` is used as it is. Any other object, an
instance of a subclass of `Array` included, is sent `request` with the
argument `ARRAY`, and the array that answers is used; an object for which that
answers `.nil`, and `.nil` itself, and an array of more than one dimension
raise `SYNTAX 98.913`. A string is converted by its
[`makeArray`](r:class.string.makearray), one argument for each line, so a
string with no line end is one argument and the null string has none; the
empty `StringTable` and `Directory` have no items. The arguments are the items
of the array by position: a hole is an omitted argument, and the argument
count is the highest index that holds an item, as the count of a message is
for [`call`](r:class.routine.call), so an array with no items gives a routine
with no arguments, however large it was made.

```rexx
r = .routine~new('n', 'return arg() "<"arg(1)"><"arg(2)"><"arg(3)">"')
call try 'r~callWith(.array~of(1, 2))'
call try 'r~callWith(.array~new)'
call try 'r~callWith(.array~new(3))'
call try 'r~callWith(.array~new(3)~~put("a", 3))'
call try 'r~callWith(.array~of(1, 2)~~put("z", 5))'
call try 'r~callWith("a b")'
call try 'r~callWith("a" || "0A"x || "b")'
call try 'r~callWith("")'
call try 'r~callWith(.stringtable~new)'
call try 'r~callWith(.holder~new)'
call try 'r~callWith(.arrsub~of(1, 2))'
call try 'r~callWith'
call try 'r~callWith(.nil)'
call try 'r~callWith(.object~new)'
call try 'r~callWith(.array~new(1, 1))'
call try 'r~callWith(.array~of(1), 2)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class holder
::method request
  say 'request' arg(1)
  return .array~of('r1', 'r2')
::class arrsub subclass array
::method makearray
  say 'makearray'
  return .array~of('z')
```

```output
r~callWith(.array~of(1, 2)) => 2 <1><2><>
r~callWith(.array~new) => 0 <><><>
r~callWith(.array~new(3)) => 0 <><><>
r~callWith(.array~new(3)~~put("a", 3)) => 3 <><><a>
r~callWith(.array~of(1, 2)~~put("z", 5)) => 5 <1><2><>
r~callWith("a b") => 1 <a b><><>
r~callWith("a" || "0A"x || "b") => 2 <a><b><>
r~callWith("") => 0 <><><>
r~callWith(.stringtable~new) => 0 <><><>
request ARRAY
r~callWith(.holder~new) => 2 <r1><r2><>
makearray
r~callWith(.arrsub~of(1, 2)) => 1 <z><><>
r~callWith => 93.903
r~callWith(.nil) => 98.913
r~callWith(.object~new) => 98.913
r~callWith(.array~new(1, 1)) => 98.913
r~callWith(.array~of(1), 2) => 93.902
```

r[class.routine.callwith.sends]
`callWith` sends `request` with the argument `ARRAY` to its argument when that
is not an instance of `Array` itself, once, before the routine runs, and no
other message to it; an instance of `Array` is not sent any message, and the
routine's code is run with the items of the array that was answered. An
override of `makeArray` shows through the `request` of `Object`, which sends
it.

```rexx
r = .routine~new('n', 'return arg() "<"arg(1)"><"arg(2)">"')
say r~callWith(.holder~new)
say r~callWith(.maker~new)
say r~callWith(.array~of('plain', 'array'))
::class holder
::method request
  say 'request' arg(1)
  return .array~of('r1', 'r2')
::class maker
::method makearray
  say 'makearray'
  return .array~of('m1', 'm2')
```

```output
request ARRAY
2 <r1><r2>
makearray
2 <m1><m2>
2 <plain><array>
```

### What the routine is

r[class.routine.source]
`source` answers a new array that holds the lines of the routine's source, one
string for each line, so changing the array changes nothing in the routine. A
routine made by `new` has the lines of its source argument (an empty string
and an empty array are a source of no lines), one made by `newFile` the lines
of its file before the first directive, without a first line that begins with
`#!`, and one that a `::ROUTINE` directive makes has the lines of its section,
which [the rule of `Method`](r:class.method.source) reads as it does for a
method: the text after the `;` of the directive is the first line, and an
empty last line is left out, so a directive with nothing after its `;` and
no lines has an empty source. A routine that `loadExternalRoutine` loaded, and
an external routine, answer an array of no items. The method takes no argument
(`SYNTAX 93.902`).

```rexx
r = .routine~new('n', .array~of('say 1', 'return 2'))
a = r~source
say a~class~id a~items (a == r~source)
a~put('changed', 1)
say r~source[1]
say .routine~new('n', '')~source~items,
  .routine~new('n', .array~new)~source~items
say .context~package~findRoutine('PLAIN')~source[1]~strip
say .context~package~findRoutine('PLAIN')~source~items
call try 'r~source(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::routine plain
  return 'plain'
```

```output
Array 2 0
say 1
0 0
return 'plain'
1
r~source(1) => 93.902
```

```rexx
p = .package~new('n', .array~of('nop', '::routine r ;', '::routine s'))
say p~routines['R']~source~items
```

```output defect=#147
0
```

r[class.routine.package]
`package` answers the package that holds the routine's code, which decides
which classes and routines the code names. For a routine that a `::ROUTINE`
directive makes it is the package of the program that holds the directive. For
a routine made by `new` or `newFile` it is the package that `new` made, as
[the context rule](r:class.method.new.context) says, which is named by the
name argument or the file name, and is the same object every time. For
`loadExternalRoutine` it is `.nil`. The method takes no argument.

```rexx
me = .context~package
r = me~findRoutine('PLAIN')
say (r~package == me) (r~package == r~package)
n = .routine~new('named', 'nop')
say n~package~class~id n~package~name (n~package == n~package)
p = .routine~loadExternalRoutine('pi', 'LIBRARY rxmath rxcalcpi')
say p~package~isNil
call try 'n~package(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::routine plain
```

```output
1 1
Package named 1
1
n~package(1) => 93.902
```

r[class.routine.annotation]
`annotation` answers the value of the annotation of the routine that has the
name it is given, and `.nil` when there is none, as
[the annotation rule](r:dir.annotate.read) says: the name is taken in upper
case, and a routine made by `new`, `newFile` or `loadExternalRoutine` has no
annotations: the answer for one is `.nil`, and its `annotations` table is
empty. The name is required (`SYNTAX 88.901`), an object with no `MAKESTRING`
method, `.nil` among them, raises `88.909`, and a second argument raises
`93.902`. A number or a string that names no annotation answers `.nil`.

```rexx
r = .context~package~findRoutine('NOTED')
say r~annotation('Version') r~annotation('VERSION') r~annotation('who')
say r~annotation('none')~isNil r~annotation(12)~isNil
say .routine~new('n', 'nop')~annotation('version')~isNil
call try 'r~annotation'
call try 'r~annotation(.nil)'
call try 'r~annotation("a", "b")'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::routine noted
::annotate routine noted Version 3
::annotate routine noted who 'me'
```

```output
3 3 me
1 1
1
r~annotation => 88.901
r~annotation(.nil) => 88.909
r~annotation("a", "b") => 93.902
```

```rexx
r = .routine~loadExternalRoutine('pi', 'LIBRARY rxmath rxcalcpi')
say r~annotation('x')~isNil r~annotations~items
```

```output
1 0
```

r[class.routine.annotations]
`annotations` answers the `StringTable` that holds every annotation of the
routine, indexed by the upper case name, as [the annotation
rule](r:dir.annotate.read) says. It is the table that the routine keeps: the
same object every time, so an entry put into it is an annotation that
`annotation` answers. A routine with none has an empty table. The method takes
no argument (`SYNTAX 93.902`).

```rexx
t = .context~package~findRoutine('NOTED')~annotations
say t~class~id t~items t['VERSION'] t['WHO']
say t~allIndexes~sort~makeArray~makeString('l', ',')
say (t == .context~package~findRoutine('NOTED')~annotations)
t~put('later', 'ADDED')
say .context~package~findRoutine('NOTED')~annotation('added')
say .routine~new('n', 'nop')~annotations~items
call try '.context~package~findRoutine("NOTED")~annotations(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::routine noted
::annotate routine noted Version 3
::annotate routine noted who 'me'
```

```output
StringTable 2 3 me
VERSION,WHO
1
later
0
.context~package~findRoutine("NOTED")~annotations(1) => 93.902
```

### The security manager

r[class.routine.setsecuritymanager]
`setSecurityManager` gives the routine a security manager, or with no argument
removes the one it has, and answers `1`. It is a protected method. The manager
can be any object, `.nil` as well, and it is not checked when it is set;
`.nil` is an object like any other and is sent the checkpoints, which it does
not understand (`SYNTAX 97.1`), so only a call with no argument removes the
manager. A second argument raises `SYNTAX 93.902`. While the code of the
routine runs, and in the internal routines of that code, the actions that [the
checkpoint rules](r:class.routine.setsecuritymanager.checkpoints) name are
offered to the manager first. The code of other packages, those of the classes
and routines that the routine calls, and the packages that it makes, are not
under the manager. A manager that is removed no longer sees the actions.

```rexx
src = 'address system "echo hidden"; return "done"'
r = .routine~new('agent', src)
say r~setSecurityManager(.guard~new)
say r~call
say r~setSecurityManager
say r~call
say r~setSecurityManager(.nil)
call try 'r~call'
call try 'r~setSecurityManager(.guard~new, 1)'
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
1
r~call => 97.1
r~setSecurityManager(.guard~new, 1) => 93.902
```

r[class.routine.setsecuritymanager.checkpoints]
The actions that are offered to the manager are the checkpoints: an external
routine call (`CALL`), a command (`COMMAND`), a `loadPackage` (`REQUIRES`), a
name looked up in `.local` (`LOCAL`) or in `.environment` (`ENVIRONMENT`), a
stream name of a stream function (`STREAM`) and a message to a protected
method (`METHOD`). The rules for each name the entries of its information
directory. The interpreter sends the manager the message of the checkpoint's
name with one argument, a `Directory` that holds the entries; the manager
answers `1` to say it handled the action, and the entries that the checkpoint
reads from the directory are used, or `0` to let the action go on as it would.
An answer that is neither `0` nor `1` raises `SYNTAX 34.903`, and an answer of
no value raises `SYNTAX 91.999`. A manager that has no method of the
checkpoint's name is sent `UNKNOWN` as any object is, with the name and an
array that holds the directory; when it has none either, the message raises
`SYNTAX 97.1` at the clause that caused the action. A manager may raise a
condition instead of answering: the usual one is `SYNTAX 98.948`, and it
reaches the code of the routine as any error of its clause does.

```rexx
src = 'signal on syntax; address system "true"; return "ran"; '
src = src || 'syntax: return "error" condition("o")~code'
mgrs = .array~of(.none~new, .zero~new, .one~new, .five~new, .nothing~new)
mgrs~append(.raiser~new)
mgrs~append(.generic~new)
do mgr over mgrs
  r = .routine~new('agent', src)
  r~setSecurityManager(mgr)
  say mgr~class~id '=>' r~call
end
exit
::class none
::class zero
::method command
  return 0
::class one
::method command
  return 1
::class five
::method command
  return 5
::class nothing
::method command
  nop
::class raiser
::method command
  raise syntax 98.948 array('no way')
::class generic
::method unknown
  use arg name, args
  say 'unknown got' name args[1]~class~id
  return 1
```

```output
NONE => error 97.1
ZERO => ran
ONE => ran
FIVE => error 34.903
NOTHING => error 91.999
RAISER => error 98.948
unknown got COMMAND Directory
GENERIC => ran
```

r[class.routine.setsecuritymanager.call]
The `CALL` checkpoint is offered for the call of an external routine: one that
is not a label in the code, not a built-in function and not a routine that the
routine's package or its context package finds, its own and those it imports;
a function of a native library is offered. The directory holds `NAME`, the
upper case name of the routine, and `ARGUMENTS`, an array of the arguments. A
manager that answers `1` takes the call over: the entry `RESULT`, when the
manager set one, is the value of the call, and with no `RESULT` the call
answers no value, which raises `SYNTAX 44.1` where the value is used. A
manager that answers `0` lets the call go on, which raises `SYNTAX 43.1` for a
routine that does not exist. A routine that the package or its context finds
is not offered, as the call of `known` in the example shows, and neither is
one that they import, as the second example shows.

```rexx
src = 'say left("abc", 2) lab(1) known(); say zqprovided(4, 5); '
src = src || 'say zqtwice(); say "end"; exit; lab: return "lab" arg(1)'
r = .routine~new('agent', src)
r~setSecurityManager(.mgr~new)
call try 'r~call'
r = .routine~new('agent', 'return zqmissing()')
r~setSecurityManager(.mgr~new)
call try 'r~call'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::routine known
  return 'known'
::class mgr
::method call
  use arg info
  say 'call' info~name info~arguments~items
  if info~name == 'ZQPROVIDED' then do
    info~result = 'replaced' info~arguments[2]
    return 1
  end
  if info~name == 'ZQTWICE' then return 1
  return 0
```

```output
ab lab 1 known
call ZQPROVIDED 2
replaced 5
call ZQTWICE 0
r~call => 44.1
call ZQMISSING 0
r~call => 43.1
```

```rexx
src = .array~of('nop', '::routine zqpub public', '  return "pub"')
lib = .package~new('zqlib', src)
ctx = .package~new('zqctx', 'nop')
ctx~addPackage(lib)
src = 'say zqpub(); say sysfileexists("zq_nofile_zz")'
r = .routine~new('agent', src, ctx)
r~setSecurityManager(.mgr~new)
r~call
exit
::class mgr
::method call
  use arg info
  say 'call' info~name
  return 0
```

```output
pub
call SYSFILEEXISTS
0
```

r[class.routine.setsecuritymanager.command]
The `COMMAND` checkpoint is offered for a command that goes to an `ADDRESS`
environment. The directory holds `COMMAND`, the command string, and `ADDRESS`,
the name of the environment in upper case. A manager that answers `1` has run
the command itself: the entry `RC`, when it is set, is the return code, and
`0` is used when it is not. The entry `FAILURE`, when it is set alone, raises
the FAILURE condition for the command, and the entry `ERROR`, when it is set
alone, raises ERROR; when both are set, FAILURE is raised and ERROR is not.
The manager answers `0` to let the interpreter run the command. The default
environment that a command goes to when the code names no `ADDRESS` is that of
the host and is not specified.

```rexx
src = 'address system "echo one"; say "rc" rc; signal on failure name fail; '
src = src || 'address system "echo two"; say "not here"; '
src = src || 'fail: say "failure" rc; signal on error name err; '
src = src || 'address system "echo three"; say "not here"; '
src = src || 'err: say "error" rc; signal on failure name fail2; '
src = src || 'signal on error name err2; address system "echo four"; '
src = src || 'say "not here"; fail2: say "both failure" rc; exit; '
src = src || 'err2: say "both error" rc'
r = .routine~new('agent', src)
r~setSecurityManager(.mgr~new)
r~call
exit
::class mgr
::method command
  use arg info
  say 'command' info~address '>'info~command'<'
  say '  entries' info~allIndexes~sort~makearray~makestring('l', ',')
  select
    when info~command = 'echo one' then info~rc = 11
    when info~command = 'echo two' then do
      info~rc = 12
      info~failure = 1
    end
    when info~command = 'echo three' then do
      info~rc = 13
      info~error = 1
    end
    when info~command = 'echo four' then do
      info~rc = 14
      info~failure = 1
      info~error = 1
    end
  end
  return 1
```

```output
command SYSTEM >echo one<
  entries ADDRESS,COMMAND
rc 11
command SYSTEM >echo two<
  entries ADDRESS,COMMAND
failure 12
command SYSTEM >echo three<
  entries ADDRESS,COMMAND
error 13
command SYSTEM >echo four<
  entries ADDRESS,COMMAND
both failure 14
```

r[class.routine.setsecuritymanager.local]
The `LOCAL` checkpoint is offered when the code names an object of `.local`,
with the entry `NAME`, the upper case name, before `.local` is searched. It is
offered for a name that [`findClass`](r:class.package.findclass) takes from
`.local` or `.environment` or does not find, except `.nil`, `.true` and
`.false`, which are never offered, not for a name that a class of the package,
of its context, of the packages it imports or of the library gives, and not
for a name in the `local` directory of the package, which is searched first. A
manager that answers `1` with an entry `RESULT` provides the object; one that
answers `1` with no `RESULT` is read as the name not being found, and the
lookup goes on with the `ENVIRONMENT` checkpoint. The code of the routine is
the code that is under the manager; the program that called it is not.

```rexx
.local~zqvalue = 'in local'
r = .routine~new('agent', 'return .zqvalue "|" .zqother "|" .zqnone')
r~setSecurityManager(.mgr~new)
say r~call
exit
::class mgr
::method local
  use arg info
  say 'local' info~name
  if info~name = 'ZQOTHER' then do
    info~result = 'provided'
    return 1
  end
  if info~name = 'ZQNONE' then return 1
  return 0
::method environment
  use arg info
  say 'environment' info~name
  return 0
```

```output
local ZQVALUE
local ZQOTHER
local ZQNONE
environment ZQNONE
in local | provided | .ZQNONE
```

```rexx
src = .array~of('nop', '::class libpub public', '::class libpriv')
lib = .package~new('lib', src)
ctx = .package~new('ctx', .array~of('nop', '::class ctxpriv'))
ctx~addPackage(lib)
src = 'x = .nil; x = .false; x = .ctxpriv; x = .libpub; x = .string; '
src = src || 'x = .zqpkg; x = .libpriv; x = .rexxinfo; x = .stdout; '
src = src || 'x = .local'
r = .routine~new('agent', src, ctx)
r~package~local~put('in package', 'ZQPKG')
r~setSecurityManager(.mgr~new)
r~call
exit
::class mgr
::method unknown
  use arg name, args
  say name args[1]~name
  return 0
```

```output
LOCAL LIBPRIV
ENVIRONMENT LIBPRIV
LOCAL REXXINFO
ENVIRONMENT REXXINFO
LOCAL STDOUT
LOCAL LOCAL
ENVIRONMENT LOCAL
```

r[class.routine.setsecuritymanager.environment]
The `ENVIRONMENT` checkpoint is offered, after the `LOCAL` one answered `0` or
did not provide the name, for a name that is looked up in `.environment`: the
entry `NAME` is the upper case name, `RESULT` and the answer have the same
meaning as for `LOCAL`. A name that `LOCAL` provided is not offered to
`ENVIRONMENT`, and neither is a name that `.local` holds, such as `.stdout`,
when `LOCAL` answered `0`.

```rexx
.environment~zqenvvalue = 'in environment'
r = .routine~new('agent', 'return .zqenvvalue "|" .zqprov "|" .zqloc')
r~setSecurityManager(.mgr~new)
say r~call
exit
::class mgr
::method local
  use arg info
  say 'local' info~name
  if info~name = 'ZQLOC' then do
    info~result = 'from local'
    return 1
  end
  return 0
::method environment
  use arg info
  say 'environment' info~name
  if info~name = 'ZQPROV' then do
    info~result = 'from environment'
    return 1
  end
  return 0
```

```output
local ZQENVVALUE
environment ZQENVVALUE
local ZQPROV
environment ZQPROV
local ZQLOC
in environment | from environment | from local
```

r[class.routine.setsecuritymanager.stream]
The `STREAM` checkpoint is offered when a stream function (`CHARIN`,
`CHAROUT`, `CHARS`, `LINEIN`, `LINEOUT`, `LINES` or `STREAM`) resolves a
stream name, the first time that the code of the routine names it in a call.
The directory holds `NAME`, the name. A manager that answers `1` has chosen
the stream: the entry `STREAM` is the object that the function then uses, and
it must understand the messages of a stream. A manager that answers `0` lets
the function open the name itself.

```rexx
f = 'class_routine_stream_zq.tmp'
call sysfiledelete f
call lineout f, 'real line one'
call lineout f, 'real line two'
call lineout f
src = 'say linein("zq_virtual_name"); say linein("zq_virtual_name"); ' ||,
  'say lines("zq_virtual_name"); return "end"'
r = .routine~new('agent', src)
r~setSecurityManager(.mgr~new(f))
say r~call
call sysfiledelete f
exit
::class mgr
::method init
  expose file
  use arg file
::method stream
  expose file
  use arg info
  say 'stream asked for' info~name~right(15)
  info~stream = .stream~new(file)
  return 1
```

```output
stream asked for zq_virtual_name
real line one
real line two
0
end
```

r[class.routine.setsecuritymanager.requires]
The `REQUIRES` checkpoint is offered when the code of the routine loads a
package from a file, with `loadPackage` or with the `new` of `Package` and a
file name. The directory holds `NAME`, the name of the file. The checkpoint is
offered twice for a file that is found, first with the name as it was given
and then with its absolute name, and once, with the name as it was given, for
a file that is not found, before the load raises `SYNTAX 43.901`. A package
that was loaded before is not loaded again and is not offered, and neither is
a `loadPackage` that has a source, and a `::REQUIRES` of a package that the
code makes from a source is not offered either. A manager that answers `0`
lets the load go on, and so does one that answers `1` and leaves `NAME` as it
is.

```rexx
f = 'class_routine_requires_zq.tmp'
call sysfiledelete f
call lineout f, 'say "loaded body"'
call lineout f
g = 'class_routine_requires_n_zq.tmp'
call sysfiledelete g
call lineout g, 'say "new body"'
call lineout g
m = 'class_routine_requires_m_zq.tmp'
call sysfiledelete m
call lineout m, 'say "m body"'
call lineout m
h = 'class_routine_requires_b_zq.tmp'
call sysfiledelete h
call lineout h, 'say "required body"'
call lineout h
src = 'p = .package~new("p", "nop"); signal on syntax name bad; '
src = src || 'say p~loadPackage("class_routine_requires_zq.tmp")~class~id; '
src = src || 'say p~loadPackage("class_routine_requires_zq.tmp")~class~id; '
src = src || 'say p~loadPackage("class_routine_requires_src_zq.tmp", '
src = src || '"nop")~class~id; '
src = src || 'say .package~new("class_routine_requires_n_zq.tmp")~class~id; '
src = src || 'x = .package~new("x", .array~of("nop", '
src = src || '"::requires ""class_routine_requires_b_zq.tmp"""));'
src = src || 'say p~loadPackage("class_routine_requires_gone_zq.tmp"); '
src = src || 'bad: say condition("o")~code'
r = .routine~new('agent', src)
r~setSecurityManager(.mgr~new)
r~call
src = 'say .package~new("class_routine_requires_m_zq.tmp")~class~id'
r = .routine~new('agent2', src)
r~setSecurityManager(.mgr1~new)
r~call
call sysfiledelete f
call sysfiledelete g
call sysfiledelete h
call sysfiledelete m
exit
::class mgr
::method requires
  use arg info
  say 'requires, absolute name:' (info~name == qualify(info~name))
  return 0
::class mgr1
::method requires
  use arg info
  say 'requires, answer 1:' (info~name == qualify(info~name))
  return 1
```

```output
requires, absolute name: 0
requires, absolute name: 1
loaded body
Package
Package
Package
requires, absolute name: 0
requires, absolute name: 1
new body
Package
required body
requires, absolute name: 0
43.901
requires, answer 1: 0
requires, answer 1: 1
m body
Package
```

r[class.routine.setsecuritymanager.method]
The `METHOD` checkpoint is offered when the code of the routine sends a
message to a [protected method](r:dir.method.protected), before the method
runs. The directory holds `OBJECT`, the receiver, `NAME`, the upper case name
of the method, and `ARGUMENTS`, an array of the arguments. A manager that
answers `1` has run the message itself: the entry `RESULT`, when it is set, is
what the message answers, and with no `RESULT` the message answers no value,
so that using the answer raises `SYNTAX 91.999`. A manager that answers `0`
lets the method run. A message to a protected method from code that is not
under a manager runs the method, and so does a message to a method that is not
protected.

```rexx
r = .routine~new('agent', 'o = .box~new; return o~locked(1, 2) o~open')
r~setSecurityManager(.mgr~new)
say r~call
say .box~new~locked(1, 2)
exit
::class box
::method locked protected
  return 'locked ran'
::method open
  return 'open ran'
::class mgr
::method method
  use arg info
  say 'method' info~name info~object~class~id info~arguments~items
  info~result = 'replaced'
  return 1
```

```output
method LOCKED BOX 2
replaced open ran
locked ran
```

```rexx
r = .routine~new('agent', 'return .box~new~locked')
r~setSecurityManager(.mgr~new)
signal on syntax name bad
say r~call
exit
bad: say condition('o')~code
::class box
::method locked protected
  return 'locked ran'
::class mgr
::method method
  return 1
```

```output
91.999
```

## Deviations

r[class.routine.deviation.41]
A source that is an encoded program image whose decoded bytes lack the tag of
an image raises `SYNTAX 3.903` from `new`, as [the rule of
`Method`](r:class.method.deviation.41) says; ooRexx translates the partly
decoded bytes as source.

```rexx
img = .array~of('#!/usr/bin/env rexx', '/**/@REXX@', 'c2F5ICdkZWNvZGVkJw==')
signal on syntax name bad
r = .routine~new('n', img)
say 'made'
exit
bad: say condition('o')~code
```

```output
3.903
```

r[class.routine.deviation.109]
An instance of a subclass of `String` that is the name or the source of `new`
is read by its own characters, and `MAKESTRING` and `STRING` are not sent to
it ([Deviation 109](r:class.string.deviation.109)). ooRexx reads the name
through its `MAKESTRING` method; the source is read as the instance's own
characters on both. In the example `MAKESTRING` answers another name.

```rexx
r = .routine~new(.name~new('Foo'), .name~new('say "src"'))
say r~package~name r~source[1]
::class name subclass string
::method makestring
  return 'other'
```

```output defect=#140
Foo say "src"
```

r[class.routine.deviation.115]
A `new` whose source answers `request` with `ARRAY` by an object that is neither
an array nor `.nil` raises `SYNTAX 88.913`, as [the sends rule of
`Method`](r:class.method.deviation.115) states, and no `request` with `STRING`
is sent. ooRexx reads the answer as an array without checking it and crashes.
The examples have `request` answer a string, a list and an object that is none
of them.

```rexx
call try '.routine~new("n", .src~new("return 1"))'
call try '.routine~new("n", .src~new(.list~of("return 1")))'
call try '.routine~new("n", .src~new(.object~new))'
call try '.routine~new("n", .src~new(.array~of("return 1")))~call'
call try '.routine~new("n", .src~new(.nil))~call'
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
  expose ans
  use arg ans
::method request
  expose ans
  say 'request' arg(1)
  if arg(1) == 'ARRAY' then return ans
  return 'return 2'
```

```output defect=#157
request ARRAY
.routine~new("n", .src~new("return 1")) => 88.913
request ARRAY
.routine~new("n", .src~new(.list~of("return 1"))) => 88.913
request ARRAY
.routine~new("n", .src~new(.object~new)) => 88.913
request ARRAY
.routine~new("n", .src~new(.array~of("return 1")))~call => 1
request ARRAY
request STRING
.routine~new("n", .src~new(.nil))~call => 2
```
