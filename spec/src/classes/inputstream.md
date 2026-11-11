---
element: "InputStream"
kind: class
rexxref: "Builtin Classes > Stream Classes > InputStream Class; Builtin Classes > Stream Classes > InputStream Class > arrayIn; Builtin Classes > Stream Classes > InputStream Class > charIn (Abstract Method); Builtin Classes > Stream Classes > InputStream Class > charOut; Builtin Classes > Stream Classes > InputStream Class > chars (Abstract Method); Builtin Classes > Stream Classes > InputStream Class > close; Builtin Classes > Stream Classes > InputStream Class > lineIn (Abstract Method); Builtin Classes > Stream Classes > InputStream Class > lineOut; Builtin Classes > Stream Classes > InputStream Class > lines (Abstract Method); Builtin Classes > Stream Classes > InputStream Class > open; Builtin Classes > Stream Classes > InputStream Class > position"
---

# InputStream

`InputStream` is a mixin class that gives a class the methods of a stream that
is read: the ones that read, and the ones that a stream of that kind does not
have. A class that inherits it, or a subclass of it, writes the methods that
read, `charIn`, `chars`, `lineIn` and `lines`; the rest the mixin answers in a
way a program can rely on: an error for the methods that make no sense, a
method that does nothing for `open` and `close`, and `arrayIn`, which reads
with the `lineIn` of the class. The classes of the library that inherit it are
[`CommandInput`](commandinput.md), [`InputOutputStream`](inputoutputstream.md)
and, through that, [`Stream`](stream.md). What the standard input is, and what
the `.input` of a program is bound to, is specified in [the bindings
rule](r:env.objects.bindings). The examples define their own classes.

## The class

r[class.inputstream.creation]
`InputStream` has the instance methods `arrayIn`, `charIn`, `charOut`,
`chars`, `close`, `lineIn`, `lineOut`, `lines`, `open` and `position`, and no
class method besides the `new` that every class has. It may have private
methods besides these; they are not part of this specification. Its only
superclass is `Object`. It is a mixin class (see [the mixin
rule](r:class.index.mixins)), and a class that inherits it has it among its
superclasses, in the place that the `INHERIT` of its directive gives it ([the
hierarchy rule](r:class.index.hierarchy)). `new` makes an instance, an object
with these methods and no state, whose string value is `an InputStream`; the
`init` of `Object` accepts no argument, so `new` with one raises `SYNTAX
93.902` ([the new rule](r:class.index.new)).

```rexx
say .inputstream~superclasses~items .inputstream~superclass~id,
  .inputstream~queryMixinClass
names = 'arrayIn charIn charOut chars close lineIn lineOut lines open position'
found = 0
do n over names~makearray(' ')
  found += .inputstream~method(n)~isA(.method)
end
say found 'of' names~words
o = .inputstream~new
say o~class~id o~string
say o~hasMethod('charIn') o~hasMethod('state'),
  o~hasMethod('makeArray') o~hasMethod('say')
signal on syntax name bad
x = .inputstream~new(1)
bad: say condition('o')~code
say .reader~superclasses~makeString('L', ' ') .reader~new~isA(.inputstream)
::class reader inherit inputstream
```

```output
1 Object 1
10 of 10
InputStream an InputStream
1 0 0 0
93.902
The Object class The InputStream class 1
```

## What the methods do

r[class.inputstream.abstract]
`charIn`, `chars`, `lineIn` and `lines` are abstract: sent to an instance of
`InputStream`, or to an instance of a class that inherits it and does not
override them, each raises `SYNTAX 93.965`. They take any number of arguments
and read none of them; the error is the same whatever they are. A class that
has them overrides them.

```rexx
o = .reader~new
do m over 'charIn chars lineIn lines'~makearray(' ')
  call try 'o~' || m
  call try 'o~' || m || '(1, 2, 3, 4, 5, 6)'
  call try 'o~' || m || '(.nil, , "x")'
end
call try '.inputstream~new~lineIn'
say .better~new~lineIn .better~new~lines
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> [' || r || ']'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class reader inherit inputstream
::class better inherit inputstream
::method lineIn
  return 'a line'
::method lines
  return 1
```

```output
o~charIn => 93.965
o~charIn(1, 2, 3, 4, 5, 6) => 93.965
o~charIn(.nil, , "x") => 93.965
o~chars => 93.965
o~chars(1, 2, 3, 4, 5, 6) => 93.965
o~chars(.nil, , "x") => 93.965
o~lineIn => 93.965
o~lineIn(1, 2, 3, 4, 5, 6) => 93.965
o~lineIn(.nil, , "x") => 93.965
o~lines => 93.965
o~lines(1, 2, 3, 4, 5, 6) => 93.965
o~lines(.nil, , "x") => 93.965
.inputstream~new~lineIn => 93.965
a line 1
```

r[class.inputstream.unsupported]
`charOut`, `lineOut` and `position` make no sense for a stream that is read:
sent to an instance of `InputStream`, or of a class that inherits it and does
not override them, each raises `SYNTAX 93.963`, whatever the number and the
values of the arguments are, none of which is read.

```rexx
o = .reader~new
do m over 'charOut lineOut position'~makearray(' ')
  call try 'o~' || m
  call try 'o~' || m || '("text")'
  call try 'o~' || m || '(1, 2, 3, 4, 5, 6)'
  call try 'o~' || m || '(.nil)'
end
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> [' || r || ']'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class reader inherit inputstream
```

```output
o~charOut => 93.963
o~charOut("text") => 93.963
o~charOut(1, 2, 3, 4, 5, 6) => 93.963
o~charOut(.nil) => 93.963
o~lineOut => 93.963
o~lineOut("text") => 93.963
o~lineOut(1, 2, 3, 4, 5, 6) => 93.963
o~lineOut(.nil) => 93.963
o~position => 93.963
o~position("text") => 93.963
o~position(1, 2, 3, 4, 5, 6) => 93.963
o~position(.nil) => 93.963
```

r[class.inputstream.nop]
`open` and `close` do nothing: they take any number of arguments, read none of
them, and answer no value, so that their answer as a term raises
`SYNTAX 91.999`.

```rexx
o = .reader~new
o~open
o~close
o~open('read', 2, 3, 4, 5)
o~close(.nil)
say 'statements ran'
call try 'x = o~open'
call try 'x = o~close'
call try 'x = o~close(1, 2)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class reader inherit inputstream
```

```output
statements ran
x = o~open => 91.999
x = o~close => 91.999
x = o~close(1, 2) => 91.999
```

r[class.inputstream.arrayin]
`arrayIn` answers a new `Array` of what `lineIn` answers: each answer is an
item as it is, an object that is not a string included, in the order of the
answers, until the NOTREADY condition ends the reading ([the rule of the
messages](r:class.inputstream.arrayin.sends) has the detail). `arrayIn` takes
any number of arguments and reads none; there is no option for characters. A
`lineIn` that answers no value raises `SYNTAX 91.999`, and the abstract
`lineIn` raises `SYNTAX 93.965`, so a class that is to use `arrayIn` overrides
`lineIn` and raises NOTREADY at the end of what it has.

```rexx
r = .reader~new
a = r~arrayin
say a~class~id a~items a~makeString('L', ',')
say .reader~new~arrayin('C', 2, 3)~items
o = .odd~new
b = o~arrayin
say b~items b[1]~class~id b[2] (b[3] == .nil) b[4]~class~id
call try '.inputstream~new~arrayin'
call try '.silent~new~arrayin'
call try '.empty~new~arrayin~items'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> [' || r || ']'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class reader inherit inputstream
::attribute pos
::method init
  self~pos = 0
::method lineIn
  self~pos = self~pos + 1
  if self~pos > 3 then raise notready
  return 'line' || self~pos
::class odd inherit inputstream
::attribute pos
::method init
  self~pos = 0
::method lineIn
  self~pos = self~pos + 1
  select
    when self~pos = 1 then return .object~new
    when self~pos = 2 then return 42
    when self~pos = 3 then return .nil
    when self~pos = 4 then return .array~of(1)
    otherwise raise notready return 'not kept'
  end
::class silent inherit inputstream
::method lineIn
  return
::class empty inherit inputstream
::method lineIn
  raise notready
```

```output
Array 3 line1,line2,line3
3
4 Object 42 1 Array
.inputstream~new~arrayin => 93.965
.silent~new~arrayin => 91.999
.empty~new~arrayin~items => [0]
```

r[class.inputstream.arrayin.sends]
`arrayIn` sends `lineIn` to the receiver, with no argument, once for each item
and once more: it stops when a `lineIn` raises the NOTREADY condition, and the
answer of that one, if it has one, is not an item. It sends no other message,
`lines` and `chars` included, and none to the answers. A `lineIn` that does
not raise NOTREADY is sent again, and again, so that a class must raise it.

```rexx
r = .reader~new
a = r~arrayin
say a~items r~log
say .reader~new~arrayin('x', 'y')~items
say .reader~new~arrayin~items
::class reader inherit inputstream
::attribute log
::attribute pos
::method init
  self~log = ''
  self~pos = 0
::method lineIn
  self~log = self~log 'lineIn#' || arg()
  self~pos = self~pos + 1
  if self~pos > 2 then raise notready return 'ignored'
  return 'line' || self~pos
::method lines
  self~log = self~log 'lines'
  return 99
::method chars
  self~log = self~log 'chars'
  return 99
::method charIn
  self~log = self~log 'charIn'
  return 'c'
::method close
  self~log = self~log 'close'
::method open
  self~log = self~log 'open'
```

```output
2  lineIn#0 lineIn#0 lineIn#0
2
2
```
