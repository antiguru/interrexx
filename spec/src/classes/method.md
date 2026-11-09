---
element: "Method"
kind: class
rexxref: "Builtin Classes > Fundamental Classes > Method Class; Builtin Classes > Fundamental Classes > Method Class > new (Class Method); Builtin Classes > Fundamental Classes > Method Class > newFile (Class Method); Builtin Classes > Fundamental Classes > Method Class > loadExternalMethod (Class Method); Builtin Classes > Fundamental Classes > Method Class > annotation; Builtin Classes > Fundamental Classes > Method Class > annotations; Builtin Classes > Fundamental Classes > Method Class > isAbstract; Builtin Classes > Fundamental Classes > Method Class > isAttribute; Builtin Classes > Fundamental Classes > Method Class > isConstant; Builtin Classes > Fundamental Classes > Method Class > isGuarded; Builtin Classes > Fundamental Classes > Method Class > isPackage; Builtin Classes > Fundamental Classes > Method Class > isPrivate; Builtin Classes > Fundamental Classes > Method Class > isProtected; Builtin Classes > Fundamental Classes > Method Class > package; Builtin Classes > Fundamental Classes > Method Class > scope; Builtin Classes > Fundamental Classes > Method Class > setGuarded; Builtin Classes > Fundamental Classes > Method Class > setPrivate; Builtin Classes > Fundamental Classes > Method Class > setProtected; Builtin Classes > Fundamental Classes > Method Class > setSecurityManager; Builtin Classes > Fundamental Classes > Method Class > setUnguarded; Builtin Classes > Fundamental Classes > Method Class > source"
deviations: [41]
---

# Method

A `Method` object is the code of one method together with the facts that
decide who may send it and how it runs: whether it is private, package-scope,
protected or guarded, which class it belongs to, and the package whose classes
and routines its code can name. A class holds its methods as `Method` objects,
`::METHOD` makes one for each directive, and a program makes one of its own
from source text with `new`, from a file with `newFile`, or from a native
library with `loadExternalMethod`.

The page specifies those three class methods, with the checks they make, the
messages they send and what the new method may see, and the methods that ask
or change what a method object is. The directive that makes a method object
is [`::METHOD`](../directives/method.md), the annotations come from
[`::ANNOTATE`](../directives/annotate.md), how a message finds a method is
[the method search](methods.md), and the methods of `Class` that install a
method object, [`define`](r:class.class.define) and
[`setMethod`](r:class.object.setmethod), are on the pages of `Object` and
`Class`. The messages every object understands are those of
[`Object`](object.md). The search for a file or a library that `newFile` and
`loadExternalMethod` start is specified in the environment chapter; the
examples here name a file that they create themselves, or a library that every
installation has.

The examples that show an error use a routine `try` that sends the message in
its argument and prints the answer, or the error code when the message raises
one, as the pages of the other classes do.

## The class

r[class.method.creation]
A method object made by `new` or `newFile` is public, guarded and not
protected, belongs to no class (its `scope` is `.nil`), has no annotations,
and holds the source it was made from. It has no code that runs by itself: it
runs when it is installed in a class or an object, and a message that finds it
there runs its code. A `Method` object is an ordinary object otherwise, so it
understands the methods of [`Object`](object.md) and has the default name
`a Method`. The class `Method` has the superclass `Object` and no mixin class.

```rexx
m = .method~new('greet', 'return "hello" arg(1)')
say m~class~id m~isGuarded m~isPrivate m~isPackage m~isProtected
say m~isAbstract m~isAttribute m~isConstant m~scope~isNil
say m~source~items m~annotations~items m~annotation('x')~isNil
say m~defaultName m .method~superclass~id .method~superclasses~items
.holder~define('GREET', m)
say .holder~new~greet('world')
::class holder
```

```output
Method 1 0 0 0
0 0 0 1
1 0 1
a Method a Method Object 1
hello world
```

## Class methods

### new

r[class.method.new]
`new` makes a method object from a name, a source and a context. The name is
required (`SYNTAX 88.901`), and is a string, or the `MAKESTRING` answer of its
argument: an object with no `MAKESTRING` method, `.nil` among them, raises
`88.909`, and a number is a name like any other string. The name becomes the
name of the method's package, which `package` shows. The source is required
too (`88.901`) and is read as [the source rule](r:class.method.new.source)
says. The context is `PROGRAMSCOPE` in any case, or omitted, or a `Method`, a
`Routine` or a `Package` object, and decides what the code can see, as
[the context rule](r:class.method.new.context) says; another value raises
`40.904`. The checks run in this order: the name, the source, the context, and
then the source is translated, so a source of `.nil` raises `93.961` whatever
the context is, and a source that does not translate raises its translation
error (`35.1` for an invalid expression, `13.1` for an invalid character)
after the context has been accepted, and `new` runs none of the code. The
arguments from the fourth on are passed to `init` as
[the sends rule](r:class.method.new.sends) says. The answer is the new method
object.

```rexx
m = .method~new('greet', 'return "hello" arg(1)')
say m~class~id m~package~name m~source[1]
call try '.method~new()'
call try '.method~new("n")'
call try '.method~new(.nil, "nop")'
call try '.method~new(12, "nop")~package~name'
call try '.method~new("n", "nop", "programscope")~class~id'
call try '.method~new("n", "nop", "other")'
call try '.method~new("n", "nop", .object~new)'
call try '.method~new("n", "nop", .nil)'
call try '.method~new("n", "say (")'
call try '.method~new("n", "nop", "PROGRAMSCOPE", 1)'
call try '.method~new(.list~new, "nop")'
call try '.method~new(.onlystr~new, "nop")'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class onlystr
::method string
  return 'onlystr'
```

```output
Method greet return "hello" arg(1)
.method~new() => 88.901
.method~new("n") => 88.901
.method~new(.nil, "nop") => 88.909
.method~new(12, "nop")~package~name => 12
.method~new("n", "nop", "programscope")~class~id => Method
.method~new("n", "nop", "other") => 40.904
.method~new("n", "nop", .object~new) => 40.904
.method~new("n", "nop", .nil) => 40.904
.method~new("n", "say (") => 35.1
.method~new("n", "nop", "PROGRAMSCOPE", 1) => 93.902
.method~new(.list~new, "nop") => 88.909
.method~new(.onlystr~new, "nop") => 88.909
```

```rexx
call try '.method~new("n", .nil, 7)'
call try '.method~new("n", "nop", 7)'
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
.method~new("n", .nil, 7) => 93.961
.method~new("n", "nop", 7) => 40.904
```

r[class.method.new.source]
The source is one string, or an array of strings. A string is a single line of
code, and the method's `source` is an array of that one line; a string is not
split at line ends, and a line end character in a line, `'0A'x`, `'0D'x` or
`'1A'x`, makes the translation raise `SYNTAX 13.1`, while a tab is an ordinary
character. An empty string is a source of no lines. An array holds the lines
in the order of its indexes. Each item is converted by `MAKESTRING` and the
answer is the line: a string is its own line, a number is a line like any
string, a `MutableBuffer` is the line of its text, and so is an instance of a
subclass of `String`; an item that has no `MAKESTRING` method, `.nil` among
them, or a hole that is followed by an item, raises `SYNTAX 93.952`. The
answer of `MAKESTRING` is a line like any other, so an item whose answer has a
line end, such as an array (its `makeString` joins its items with line ends),
raises `13.1`. An empty array is a source of no lines. A source that is
neither a string nor an array is converted as
[the sends rule](r:class.method.new.sends) says; `.nil` raises `93.961`. An
instance of a subclass of `String` is such a source: it is not used as the one
line that a string is, but is converted by
[its `makeArray`](r:class.string.makearray), so it is split at its line ends,
and a null one is a source of no lines.

```rexx
call try '.method~new("n", "return 1")~source~items'
call try '.method~new("n", .array~of("a = 1", "return a"))~source~items'
call try '.method~new("n", .array~of("a = 1", "return a"))~source[2]'
call try '.method~new("n", .array~new)~source~items'
call try '.method~new("n", "")~source~items'
call try '.method~new("n", 5)~source[1]'
call try '.method~new("n", .array~of("x", 3))~source[2]'
call try '.method~new("n", .array~of("x", .object~new))'
call try '.method~new("n", .array~new(2)~~put("nop", 2))'
call try '.method~new("n", "a" || "09"x || "b")~source~items'
call try '.method~new("n", "a" || "0A"x || "b")'
call try '.method~new("n", "a" || "0D"x)'
call try '.method~new("n", "a" || "1A"x)'
call try '.method~new("n", .nil)'
call try '.method~new("n", .array~of("a" || "0A"x || "b"))'
call try '.method~new("n", .array~of(.onlystr~new))'
ssrc = .ss~new("a = 1" || "0A"x || "return a")
call try '.method~new("n", ssrc)~source~items'
call try '.method~new("n", .ss~new(""))~source~items'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class onlystr
::method string
  return 'onlystr'
::class ss subclass string
```

```output
.method~new("n", "return 1")~source~items => 1
.method~new("n", .array~of("a = 1", "return a"))~source~items => 2
.method~new("n", .array~of("a = 1", "return a"))~source[2] => return a
.method~new("n", .array~new)~source~items => 0
.method~new("n", "")~source~items => 0
.method~new("n", 5)~source[1] => 5
.method~new("n", .array~of("x", 3))~source[2] => 3
.method~new("n", .array~of("x", .object~new)) => 93.952
.method~new("n", .array~new(2)~~put("nop", 2)) => 93.952
.method~new("n", "a" || "09"x || "b")~source~items => 1
.method~new("n", "a" || "0A"x || "b") => 13.1
.method~new("n", "a" || "0D"x) => 13.1
.method~new("n", "a" || "1A"x) => 13.1
.method~new("n", .nil) => 93.961
.method~new("n", .array~of("a" || "0A"x || "b")) => 13.1
.method~new("n", .array~of(.onlystr~new)) => 93.952
.method~new("n", ssrc)~source~items => 2
.method~new("n", .ss~new(""))~source~items => 0
```

```rexx
call try '.method~new("n", .array~of("a", .mk~new))~source[2]'
call try '.method~new("n", .array~of(.ss~new("x")))~source[1]'
call try '.method~new("n", .array~of(.mutablebuffer~new("mb")))~source[1]'
call try '.method~new("n", .array~of(.array~of("a", "b")))'
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
  return 'from makestring'
::class ss subclass string
```

```output defect=#142
.method~new("n", .array~of("a", .mk~new))~source[2] => from makestring
.method~new("n", .array~of(.ss~new("x")))~source[1] => x
.method~new("n", .array~of(.mutablebuffer~new("mb")))~source[1] => mb
.method~new("n", .array~of(.array~of("a", "b"))) => 13.1
```

r[class.method.new.sends]
`new` sends messages to its source and to the object it makes. A source that
is not a string and not an array, and an instance of a subclass of `String` or
`Array`, is sent `request` with the argument `ARRAY`; when that answers an
array its items are the lines, read as
[the source rule](r:class.method.new.source) reads an array. When it answers
`.nil` the source is sent `request` with the argument `STRING`: an answer that
is a string is the one-line source, and for any other answer its `objectName`
is the line, with no `MAKESTRING` or `string` message sent to it; `.nil` from
both raises `SYNTAX 93.961`. What `request` answers when its answer to `ARRAY`
is neither an array nor `.nil` is not specified. A string that is an instance
of `String` itself and an array that is an instance of `Array` itself are used
directly, without a message. The `request` of `Object` sends `makeArray` for
the argument `ARRAY` and `makeString` for `STRING`, as
[its rule](r:class.object.request.sends) says, so an object that declares
`makeArray` or `makeString` is a source. When the method is made, `new` sends
the new object the message `init` with the arguments of `new` from the fourth
on, so that a subclass of `Method` that declares `init` receives them; the
`init` of `Object` accepts none, and a fourth argument raises `SYNTAX 93.902`
after the source has been translated. The class that received `new` decides
the class of the answer: a subclass of `Method` makes instances of itself.

```rexx
call try '.method~new("n", .src~new(.array~of("return 1"), .nil))'
call try '.method~new("n", .src~new(.nil, "return 2"))~source[1]'
call try '.method~new("n", .src~new(.nil, .nil))'
call try '.method~new("n", .arrsub~of("return 3"))~source[1]'
call try '.method~new("n", .src~new(.nil, .loud~new))~source[1]'
call try '.method~new("n", .src~new(.nil, .named~new))~source[1]'
call try '.method~new("n", .src~new(.nil, 12))~source[1]'
say .mine~new('n', 'return 5', 'PROGRAMSCOPE', 'a', 'b')~class~id
say .mine~new('n', 'return 5')~source[1]
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
::class arrsub subclass array
::method makearray
  say 'makearray'
  return .array~of('return 33')
::class loud
::method makestring
  say 'makestring'
  return 'return 1'
::method string
  say 'string'
  return 'return 1'
::class named
::method objectname
  return 'return 4'
::class mine subclass method
::method init
  say 'init' arg() arg(1) arg(2)
```

```output
request ARRAY
.method~new("n", .src~new(.array~of("return 1"), .nil)) => a Method
request ARRAY
request STRING
.method~new("n", .src~new(.nil, "return 2"))~source[1] => return 2
request ARRAY
request STRING
.method~new("n", .src~new(.nil, .nil)) => 93.961
makearray
.method~new("n", .arrsub~of("return 3"))~source[1] => return 33
request ARRAY
request STRING
.method~new("n", .src~new(.nil, .loud~new))~source[1] => a LOUD
request ARRAY
request STRING
.method~new("n", .src~new(.nil, .named~new))~source[1] => return 4
request ARRAY
request STRING
.method~new("n", .src~new(.nil, 12))~source[1] => 12
init 2 a b
MINE
init 0  
return 5
```

r[class.method.new.context]
The context decides which classes and routines the code of the method can
name, as the classes and routines of a package do. The method has a package of
its own, a new package named by the name argument, whose source is the source
of the method. That package has no classes or routines of its own, and a name
that it does not hold is looked up in the package of the context, among the
classes and routines that package finds: its own, public or not, and the
public ones of the packages it imports. With `PROGRAMSCOPE`, or with the
argument omitted, that is the package of the code that sent `new`: the method
sees the classes and routines of the program that made it. With a `Package`
object it is that package, with a `Routine` or a `Method` object it is the
package of that object, and the classes and routines of the program that sent
`new` are no longer visible: a class name that is not found is the string of
that name, and a routine that is not found raises `SYNTAX 43.1`. The package
of the context is not the package of the new method: `package` answers another
object. A `Method` or `Routine` context whose `package` is `.nil`, one that
`loadExternalMethod` or `loadExternalRoutine` made, gives a package whose
lookup through the context is not specified.

```rexx
src = .array~of('return 1', '::class there', '::routine thereproc')
src~append('  return "there routine"')
p = .package~new('other', src)
rout = .routine~new('rr', 'return 1', p)
default = .method~new('m1', 'return .here~class~id here()')
inpkg = .method~new('m2', 'return .there~class~id thereproc()', p)
inrout = .method~new('m3', 'return .there~class~id', rout)
inmeth = .method~new('m4', 'return .there~class~id', inpkg)
lost = .method~new('m5', 'return .here~class~id', p)
nohere = .method~new('m6', 'return here()', p)
say run(default)
say run(inpkg)
say run(inrout) run(inmeth)
say run(lost)
say run(nohere)
say (default~package == .context~package) (inpkg~package == p)
say default~package~name inpkg~package~name
exit
run:
  signal on syntax name bad
  .runner~define('GO', arg(1))
  return .runner~new~go
bad: return 'error' condition('o')~code
::class here
::routine here
  return 'here routine'
::class runner
```

```output
Class here routine
Class there routine
Class Class
String
error 43.1
0 0
m1 m2
```

### newFile

r[class.method.newfile]
`newFile` makes a method object from the lines of a file. The file name is
required (`SYNTAX 88.901`) and is a string, or an object with a `MAKESTRING`
method (`.nil` and others with none raise `88.909`), and the context is the
second argument, with the same values and the same error `40.904` as for
[`new`](r:class.method.new.context). `newFile` takes no other argument: a
third raises `SYNTAX 93.902` before the file is looked for. The checks run in
this order: the number of arguments, the name, the context, and then the file
is found and read; a file that cannot be read raises `SYNTAX 3.1`, and a file
that does not translate raises its translation error. The method's package is
named by the file name as it was given, and its source is the lines of the
file that come before its first directive, except that a first line that
begins with `#!` is not part of the source. A directive in the file still
makes the classes and routines of the method's package, which the package
answers to `classes` and `routines`, and the code of the method sees them. The
context is used as for `new`. How the file is searched for is specified in the
environment chapter. `newFile` sends `init` with no argument to the new
object, and a subclass of `Method` makes instances of itself.

```rexx
f = 'class_method_newfile_zq.tmp'
call sysfiledelete f
call lineout f, 'say "file method" arg(1)'
call lineout f, 'return .fromfile~new~hi'
call lineout f, '::class fromfile'
call lineout f, '::method hi'
call lineout f, '  return "hi from the file"'
call lineout f
m = .method~newFile(f)
say m~class~id m~source~items m~source[2] (m~package~name == f)
say m~package~classes~items m~package~classes~allindexes~makearray~items
.runner~define('GO', m)
say .runner~new~go('x')
say .mine~newFile(f)~class~id
call sysfiledelete f
call try '.method~newFile("class_method_newfile_none.tmp")'
call try '.method~newFile()'
call try '.method~newFile(.nil)'
call try '.method~newFile(f, "PROGRAMSCOPE", 1)'
call try '.method~newFile("class_method_newfile_none.tmp", "bad")'
call try '.method~newFile("class_method_newfile_none.tmp", "PROGRAMSCOPE")'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class runner
::class mine subclass method
::method init
  say 'init' arg()
```

```output
Method 2 return .fromfile~new~hi 1
1 1
file method x
hi from the file
init 0
MINE
.method~newFile("class_method_newfile_none.tmp") => 3.1
.method~newFile() => 88.901
.method~newFile(.nil) => 88.909
.method~newFile(f, "PROGRAMSCOPE", 1) => 93.902
.method~newFile("class_method_newfile_none.tmp", "bad") => 40.904
.method~newFile("class_method_newfile_none.tmp", "PROGRAMSCOPE") => 3.1
```

```rexx
f = 'class_method_shebang_zq.tmp'
call sysfiledelete f
call lineout f, '#!/usr/bin/rexx'
call lineout f, 'return "after the first line"'
call lineout f
m = .method~newFile(f)
say m~source~items m~source[1]
call sysfiledelete f
```

```output defect=#144
1 return "after the first line"
```

r[class.method.newfile.sends]
`newFile` sends the new object the message `init` with no argument, after the
file has been read and translated, so that a subclass of `Method` that
declares `init` runs it once for each `newFile`. A file name that is an object
with a `MAKESTRING` method is sent that message for its string value, once,
before `init`; no other message is sent. The arguments of `newFile` are only
the name and the context, so no argument reaches `init`.

```rexx
f = 'class_method_newfilesends_zq.tmp'
call sysfiledelete f
call lineout f, 'return 1'
call lineout f
say .mine~newFile(f)~class~id
say .mine~newFile(.named~new(f))~class~id
call sysfiledelete f
::class mine subclass method
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

### loadExternalMethod

r[class.method.loadexternalmethod]
`loadExternalMethod` makes a method object whose code is a routine of a native
library, the same kind of method that
[`::METHOD EXTERNAL`](r:dir.method.external) declares. It takes a name and a
descriptor and no other argument (a third raises `SYNTAX 93.902`). Both are
required (`88.901`), and each is a string or an object with a `MAKESTRING`
method (`88.909` for `.nil` and for an object with none). The descriptor has
the form that the directive's operand has: the word `LIBRARY` in any case, the
name of the library, and optionally the name of the entry in it, which is the
method name when it is left out; a descriptor of another form raises
`SYNTAX 99.917`. When the library cannot be loaded, or has no entry of that
name, the answer is `.nil` and nothing is raised. Otherwise the answer is a
method object that is public and guarded, whose `source` is an empty array and
whose `package` is `.nil`. Where the library is searched for, and whether its
name is case sensitive, is specified in the environment chapter. The result is
an instance of `Method` also when a subclass received the message, and no
`init` is sent.

```rexx
m = .method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Pos')
say m~class~id m~source~items m~package~isNil m~isGuarded m~isPrivate
m = .method~loadExternalMethod('m', 'library rxregexp regexp_pos')
say m~class~id
say .method~loadExternalMethod('m', 'LIBRARY rxregexp')~isNil
say .method~loadExternalMethod('m', 'LIBRARY rxregexp nosuchentry')~isNil
say .method~loadExternalMethod('m', 'LIBRARY nosuchlibraryzq nosuch')~isNil
say .mine~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Pos')~class~id
call try '.method~loadExternalMethod("m", "REGISTERED rxregexp")'
call try '.method~loadExternalMethod("m", "LIBRARY")'
call try '.method~loadExternalMethod("m", "")'
call try '.method~loadExternalMethod("m")'
call try '.method~loadExternalMethod()'
call try '.method~loadExternalMethod(.nil, "LIBRARY rxregexp RegExp_Pos")'
call try '.method~loadExternalMethod("m", .nil)'
call try '.method~loadExternalMethod("m", "LIBRARY rxregexp RegExp_Pos", 1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class mine subclass method
::method init
  say 'init'
```

```output
Method 0 1 1 0
Method
1
1
1
Method
.method~loadExternalMethod("m", "REGISTERED rxregexp") => 99.917
.method~loadExternalMethod("m", "LIBRARY") => 99.917
.method~loadExternalMethod("m", "") => 99.917
.method~loadExternalMethod("m") => 88.901
.method~loadExternalMethod() => 88.901
.method~loadExternalMethod(.nil, "LIBRARY rxregexp RegExp_Pos") => 88.909
.method~loadExternalMethod("m", .nil) => 88.909
.method~loadExternalMethod("m", "LIBRARY rxregexp RegExp_Pos", 1) => 93.902
```

## Methods

### What the method is

r[class.method.source]
`source` answers a new array that holds the lines of the method's source, one
string for each line, so changing the array changes nothing in the method. A
method made by `new` has the lines of its source argument, one made by
`newFile` the lines of its file before the first directive, without a first
line that begins with `#!`, and one that a `::METHOD` directive makes has the
lines of its section. Text that follows the `;` of the directive on its line
is the first line of the section, blank included, and the last line of a
section is left out when it is empty; a line of blanks is kept. A method that
has no Rexx code of its own, such as an attribute method, a constant method,
an abstract method, a method of a library class and one that
`loadExternalMethod` loaded, answers an array of no items. The method takes no
argument (`SYNTAX 93.902`).

```rexx
m = .method~new('n', .array~of('say 1', 'return 2'))
a = m~source
say a~class~id a~items (a == m~source)
a~put('changed', 1)
say m~source[1]
say .k~method('plain')~source~items .k~method('plain')~source[1]~strip
say .k~method('at')~source~items .k~method('abs')~source~items
say .k~method('cc')~source~items .array~method('append')~source~items
call try 'm~source(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method plain
  return 'plain'
::attribute at
::method abs abstract
::constant cc 5
```

```output
Array 2 0
say 1
1 return 'plain'
0 0
0 0
m~source(1) => 93.902
```

```rexx
src = .array~of('::method same; say "x"', '::method tail', '  say "a"', '')
src~append('::method blank')
src~append('  say "b"')
src~append('  ')
src~append('::method last')
p = .package~new('n', src)
t = p~definedMethods
do n over .array~of('SAME', 'TAIL', 'BLANK')
  say n t[n]~source~items '['t[n]~source~makeString('l', '|')']'
end
```

```output
SAME 1 [ say "x"]
TAIL 1 [  say "a"]
BLANK 2 [  say "b"|  ]
```

```rexx
p = .package~new('n', .array~of('nop', '::method f ;', '::method g'))
say p~definedMethods['F']~source~items
```

```output defect=#147
0
```

r[class.method.package]
`package` answers the package that holds the method's code, which decides
which classes and routines the code names. For a method that a `::METHOD`
directive makes it is the package of the program that holds the directive. For
a method made by `new` or `newFile` it is the package that `new` made, as
[the context rule](r:class.method.new.context) says, which is named by the
name argument or the file name and is the same object every time. For a method
of a library class it is the package named `REXX`, and for one that
`loadExternalMethod` made it is `.nil`. The method takes no argument.

```rexx
me = .context~package
m = .k~method('plain')
say (m~package == me) (m~package == m~package)
n = .method~new('named', 'nop')
say n~package~class~id n~package~name (n~package == n~package)
say .array~method('append')~package~name
ext = .method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Pos')
say ext~package~isNil
call try 'n~package(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method plain
```

```output
1 1
Package named 1
REXX
1
n~package(1) => 93.902
```

r[class.method.scope]
`scope` answers the class that the method belongs to, which is the class whose
`::METHOD` directive or `define` gave it to the class, and `.nil` for a method
that belongs to no class, such as one made by `new` or `newFile`. A method
that a class inherits from a superclass or a mixin class keeps the scope of
the class that defined it, so `instanceMethod` of an instance of a subclass
shows the superclass. The method that [`define`](r:class.class.define)
installs runs the code of the method object that is given, with the class as
its scope, so `method` of the class answers a method with that scope. The
method that [`setMethod`](r:class.object.setmethod) installs runs the code
with the scope that [the scope rule](r:class.methods.object.scope) says, which
is none for `FLOAT`. Whether the installed method is the argument or a copy of
it, and what scope the argument answers afterwards, is not specified. The
method takes no argument.

```rexx
say .sub~method('own')~scope~id .sub~new~instanceMethod('inherited')~scope~id
say .sub~new~instanceMethod('mixed')~scope~id
say .array~method('append')~scope~id
say .method~new('n', 'nop')~scope~isNil
m = .method~new('d', 'return 1')
.sub~define('D', m)
say .sub~method('D')~scope~id
n = .method~new('s', 'return 2')
o = .sub~new
o~put('S', n)
say o~instanceMethod('S')~scope~isNil
call try '.sub~method("own")~scope(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class mix mixinclass object
::method mixed
::class base
::method inherited
::class sub subclass base inherit mix
::method own
::method put
  use arg name, meth
  self~setMethod(name, meth)
```

```output
SUB BASE
MIX
Array
1
SUB
1
.sub~method("own")~scope(1) => 93.902
```

r[class.method.annotation]
`annotation` answers the value of the annotation of the method that has the
name it is given, and `.nil` when there is none, as
[the annotation rule](r:dir.annotate.read) says: the name is taken in upper
case, and a method made by `new`, `newFile` or `loadExternalMethod` has no
annotations: the answer for one is `.nil`, and its `annotations` table is
empty. The name is required (`SYNTAX 88.901`), an object with no `MAKESTRING`
method, `.nil` among them, raises `88.909`, and a second argument raises
`93.902`. A name that is a number or a string that names no annotation answers
`.nil`.

```rexx
m = .k~method('m')
say m~annotation('Version') m~annotation('VERSION') m~annotation('who')
say m~annotation('none')~isNil m~annotation(12)~isNil
say .method~new('n', 'nop')~annotation('version')~isNil
call try 'm~annotation'
call try 'm~annotation(.nil)'
call try 'm~annotation("a", "b")'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method m
::annotate method m Version 3
::annotate method m who 'me'
```

```output
3 3 me
1 1
1
m~annotation => 88.901
m~annotation(.nil) => 88.909
m~annotation("a", "b") => 93.902
```

```rexx
m = .method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Pos')
say m~annotation('x')~isNil m~annotations~items
```

```output defect=#146
1 0
```

r[class.method.annotations]
`annotations` answers the `StringTable` that holds every annotation of the
method, indexed by the upper case name, as [the annotation
rule](r:dir.annotate.read) says. It is the table that the method keeps: the
same object every time, so an entry put into it is an annotation that
`annotation` answers. A method with none has an empty table. The method takes
no argument (`SYNTAX 93.902`).

```rexx
t = .k~method('m')~annotations
say t~class~id t~items t['VERSION'] t['WHO']
say t~allIndexes~sort~makeArray~makeString('l', ',')
say (t == .k~method('m')~annotations)
t~put('later', 'ADDED')
say .k~method('m')~annotation('added') .k~method('m')~annotations~items
say .method~new('n', 'nop')~annotations~items
call try '.k~method("m")~annotations(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method m
::annotate method m Version 3
::annotate method m who 'me'
```

```output
StringTable 2 3 me
VERSION,WHO
1
later 3
0
.k~method("m")~annotations(1) => 93.902
```

### What the method is declared as

r[class.method.isabstract]
`isAbstract` answers `1` for a method that `::METHOD ... ABSTRACT` declared
and `0` for every other, a method made by `new` included. It takes no argument
(`SYNTAX 93.902`).

```rexx
say .k~method('absm')~isAbstract .k~method('real')~isAbstract
say .method~new('n', 'nop')~isAbstract
call try '.k~method("absm")~isAbstract(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method absm abstract
::method real
```

```output
1 0
0
.k~method("absm")~isAbstract(1) => 93.902
```

r[class.method.isattribute]
`isAttribute` answers `1` for a method that an `::ATTRIBUTE` directive made,
the one that reads and the one that assigns, and for the methods that
`::METHOD ... ATTRIBUTE` made, and `0` for every other. It takes no argument
(`SYNTAX 93.902`).

```rexx
say .k~method('at')~isAttribute .k~method('at=')~isAttribute
say .k~method('viaoption')~isAttribute .k~method('plain')~isAttribute
say .k~method('cc')~isAttribute
say .method~new('n', 'nop')~isAttribute
call try '.k~method("at")~isAttribute(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::attribute at
::method viaoption attribute
::method plain
::constant cc 5
```

```output
1 1
1 0
0
0
.k~method("at")~isAttribute(1) => 93.902
```

r[class.method.isconstant]
`isConstant` answers `1` for the method that a `::CONSTANT` directive made and
`0` for every other, an attribute method and a method made by `new` included.
It takes no argument (`SYNTAX 93.902`).

```rexx
say .k~method('cc')~isConstant .k~method('at')~isConstant
say .k~method('plain')~isConstant .method~new('n', 'nop')~isConstant
call try '.k~method("cc")~isConstant(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::constant cc 5
::attribute at
::method plain
```

```output
1 0
0 0
.k~method("cc")~isConstant(1) => 93.902
```

r[class.method.isguarded]
`isGuarded` answers `1` for a guarded method and `0` for an unguarded one. A
method is guarded unless `UNGUARDED` was declared, a `::CONSTANT` directive
made it, or [`setUnguarded`](r:class.method.setunguarded) was sent; a method
made by `new` is guarded, and so are an attribute method and an abstract
method. What guarding does when messages run at the same time is specified in
the environment chapter. The method takes no argument (`SYNTAX 93.902`).

```rexx
say .k~method('g')~isGuarded .k~method('u')~isGuarded
say .k~method('plain')~isGuarded .method~new('n', 'nop')~isGuarded
say .k~method('cc')~isGuarded .k~method('at')~isGuarded
say .k~method('abs')~isGuarded
call try '.k~method("g")~isGuarded(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method g guarded
::method u unguarded
::method plain
::constant cc 5
::attribute at
::method abs abstract
```

```output
1 0
1 1
0 1
1
.k~method("g")~isGuarded(1) => 93.902
```

r[class.method.ispackage]
`isPackage` answers `1` for a method declared `PACKAGE` and `0` for every
other. [The access rule](r:dir.method.access) says who may send a package
method. `isPackage` is independent of `isPrivate`: sending
[`setPrivate`](r:class.method.setprivate) to a package method makes both
answer `1`. The method takes no argument (`SYNTAX 93.902`).

```rexx
m = .k~method('pk')
say m~isPackage m~isPrivate
m~setPrivate
say m~isPackage m~isPrivate
say .k~method('plain')~isPackage .method~new('n', 'nop')~isPackage
call try '.k~method("plain")~isPackage(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method pk package
::method plain
```

```output
1 0
1 1
0 0
.k~method("plain")~isPackage(1) => 93.902
```

r[class.method.isprivate]
`isPrivate` answers `1` for a private method and `0` for every other. A method
is private when `PRIVATE` was declared or
[`setPrivate`](r:class.method.setprivate) was sent. [The access
rule](r:dir.method.access) says who may send a private method. The method
takes no argument (`SYNTAX 93.902`).

```rexx
say .k~method('pr')~isPrivate .k~method('plain')~isPrivate
say .method~new('n', 'nop')~isPrivate
say .k~method('pk')~isPrivate
call try '.k~method("pr")~isPrivate(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method pr private
::method plain
::method pk package
```

```output
1 0
0
0
.k~method("pr")~isPrivate(1) => 93.902
```

r[class.method.isprotected]
`isProtected` answers `1` for a protected method and `0` for every other. A
method is protected when `PROTECTED` was declared or
[`setProtected`](r:class.method.setprotected) was sent. The method takes no
argument (`SYNTAX 93.902`).

```rexx
say .k~method('pro')~isProtected .k~method('plain')~isProtected
say .method~new('n', 'nop')~isProtected
call try '.k~method("pro")~isProtected(1)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method pro protected
::method plain
```

```output
1 0
0
.k~method("pro")~isProtected(1) => 93.902
```

### Changing the method

r[class.method.setguarded]
`setGuarded` makes the method guarded and answers no value, so using its
answer as a term raises `SYNTAX 91.999`. It has no effect on a method that is
guarded already. It takes no argument (`SYNTAX 93.902`). The method object
that `method` of a class answers is the one the class holds, so the example
sees the change through the class.

```rexx
m = .k~method('u')
say m~isGuarded
m~setGuarded
say m~isGuarded .k~method('u')~isGuarded
m~setGuarded
say m~isGuarded
call try 'm~setGuarded(1)'
call try 'x = m~setGuarded'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method u unguarded
```

```output
0
1 1
1
m~setGuarded(1) => 93.902
x = m~setGuarded => 91.999
```

r[class.method.setunguarded]
`setUnguarded` makes the method unguarded and answers no value, so using its
answer raises `SYNTAX 91.999`. It has no effect on a method that is unguarded
already. It takes no argument (`SYNTAX 93.902`).

```rexx
m = .method~new('n', 'nop')
say m~isGuarded
m~setUnguarded
say m~isGuarded
m~setUnguarded
say m~isGuarded
call try 'm~setUnguarded(1)'
call try 'x = m~setUnguarded'
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
0
0
m~setUnguarded(1) => 93.902
x = m~setUnguarded => 91.999
```

r[class.method.setprivate]
`setPrivate` makes the method private and answers no value, so using its
answer raises `SYNTAX 91.999`. It changes nothing else, so a package method
stays a package method. It takes no argument (`SYNTAX 93.902`). A message to
an installed method that was private raises the error of
[the access rule](r:dir.method.access) from then on, as the example shows by
sending `setPrivate` to the method that a class holds.

```rexx
o = .k~new
say o~hello
.k~method('hello')~setPrivate
say .k~method('hello')~isPrivate
call try 'o~hello'
call try '.k~method("hello")~setPrivate(1)'
call try 'x = .method~new("n", "nop")~setPrivate'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::method hello
  return 'hello'
```

```output
hello
1
o~hello => 97.2
.k~method("hello")~setPrivate(1) => 93.902
x = .method~new("n", "nop")~setPrivate => 91.999
```

r[class.method.setprotected]
`setProtected` makes the method protected and answers no value, so using its
answer raises `SYNTAX 91.999`. It takes no argument (`SYNTAX 93.902`). A
protected method runs like any other until the code that sends the message to
it is under a security manager: the manager is then sent `METHOD` before the
method runs, as [the checkpoint
rule](r:class.routine.setsecuritymanager.method) says.

```rexx
m = .method~new('n', 'return "ran"')
say m~isProtected
m~setProtected
say m~isProtected
.k~define('PROT', m)
say .k~new~prot
call try 'm~setProtected(1)'
call try 'x = m~setProtected'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
```

```output
0
1
ran
m~setProtected(1) => 93.902
x = m~setProtected => 91.999
```

r[class.method.setsecuritymanager]
`setSecurityManager` gives the method a security manager, or with no argument
removes the one it has, and answers `1`. It is a protected method. The manager
is any object, `.nil` as well, and it is not checked when it is set: `.nil` is
an object like any other, which is sent the checkpoints and does not
understand them (`SYNTAX 97.1`), so only a call with no argument removes the
manager. A second argument raises `SYNTAX 93.902`. While the code of the
method runs, and in the internal routines of that code, the actions that
[the checkpoint rules](r:class.routine.setsecuritymanager.checkpoints) name
are offered to the manager first; the code of other packages that the method
calls is not under the manager. A manager that was set before
[`define`](r:class.class.define) installed the method stays with the
installed method, as the example shows.

```rexx
m = .method~new('n', 'address system "echo not run"; return "ran"')
say m~setSecurityManager(.guard~new)
.k~define('GO', m)
say .k~new~go
say m~setSecurityManager
say .method~new('x', 'nop')~setSecurityManager(.nil)
call try 'm~setSecurityManager(1, 2)'
exit
try:
  signal on syntax name bad
  interpret 'zres =' arg(1)
  say arg(1) '=>' zres
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class k
::class guard
::method command
  use arg info
  say 'command asked:' info~command
  info~rc = 3
  return 1
```

```output
1
command asked: echo not run
ran
1
1
m~setSecurityManager(1, 2) => 93.902
```

## Deviations

r[class.method.deviation.41]
A source that is an encoded program image, which has the line
`#!/usr/bin/env rexx`, the marker line `/**/@REXX@` and then encoded text, and
whose decoded bytes lack the tag of an image, raises `SYNTAX 3.903` from
`new`, as [the deviation](../deviations.md#41-a-damaged-compiled-image) says
for a program. ooRexx translates the partly decoded bytes as source, with
results that are not predictable: it raises `13.1` for the source of the
example.

```rexx
img = .array~of('#!/usr/bin/env rexx', '/**/@REXX@', 'c2F5ICdkZWNvZGVkJw==')
signal on syntax name bad
m = .method~new('n', img)
say 'made'
exit
bad: say condition('o')~code
```

```output
3.903
```
