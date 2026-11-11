---
element: "OutputStream"
kind: class
rexxref: "Builtin Classes > Stream Classes > OutputStream Class; Builtin Classes > Stream Classes > OutputStream Class > arrayOut; Builtin Classes > Stream Classes > OutputStream Class > charIn; Builtin Classes > Stream Classes > OutputStream Class > charOut (Abstract Method); Builtin Classes > Stream Classes > OutputStream Class > chars; Builtin Classes > Stream Classes > OutputStream Class > close; Builtin Classes > Stream Classes > OutputStream Class > lineIn; Builtin Classes > Stream Classes > OutputStream Class > lineOut (Abstract Method); Builtin Classes > Stream Classes > OutputStream Class > lines; Builtin Classes > Stream Classes > OutputStream Class > open; Builtin Classes > Stream Classes > OutputStream Class > position"
---

# OutputStream

`OutputStream` is a mixin class that gives a class the methods of a stream
that is written: the ones that write, and the ones that a stream of that kind
does not have. A class that inherits it, or a subclass of it, writes the
methods that write, `charOut` and `lineOut`; the rest the mixin answers in a
way a program can rely on: an error for the methods that make no sense, a
method that does nothing for `open` and `close`, and `arrayOut`, which writes
with the `lineOut` of the class. The classes of the library that inherit it
are [`CommandOutput`](commandoutput.md),
[`InputOutputStream`](inputoutputstream.md) and, through that,
[`Stream`](stream.md). What the standard output and error are, and what the
`.output` and `.error` of a program are bound to, is specified in [the
bindings rule](r:env.objects.bindings). The examples define their own classes.

## The class

r[class.outputstream.creation]
`OutputStream` has the instance methods `arrayOut`, `charIn`, `charOut`,
`chars`, `close`, `lineIn`, `lineOut`, `lines`, `open` and `position`, and no
class method besides the `new` that every class has. It may have private
methods besides these; they are not part of this specification. Its only
superclass is `Object`. It is a mixin class (see [the mixin
rule](r:class.index.mixins)), and a class that inherits it has it among its
superclasses, in the place that the `INHERIT` of its directive gives it ([the
hierarchy rule](r:class.index.hierarchy)). `new` makes an instance, an object
with these methods and no state, whose string value is `an OutputStream`; the
`init` of `Object` accepts no argument, so `new` with one raises `SYNTAX
93.902` ([the new rule](r:class.index.new)).

```rexx
say .outputstream~superclasses~items .outputstream~superclass~id,
  .outputstream~queryMixinClass
names = 'arrayOut charIn charOut chars close lineIn lineOut lines open position'
found = 0
do n over names~makearray(' ')
  found += .outputstream~method(n)~isA(.method)
end
say found 'of' names~words
o = .outputstream~new
say o~class~id o~string
say o~hasMethod('charOut') o~hasMethod('state'),
  o~hasMethod('arrayIn') o~hasMethod('say')
signal on syntax name bad
x = .outputstream~new(1)
bad: say condition('o')~code
say .writer~superclasses~makeString('L', ' ') .writer~new~isA(.outputstream)
::class writer inherit outputstream
```

```output
1 Object 1
10 of 10
OutputStream an OutputStream
1 0 0 0
93.902
The Object class The OutputStream class 1
```

## What the methods do

r[class.outputstream.abstract]
`charOut` and `lineOut` are abstract: sent to an instance of `OutputStream`,
or to an instance of a class that inherits it and does not override them, each
raises `SYNTAX 93.965`. They take any number of arguments and read none of
them; the error is the same whatever they are. A class that has them overrides
them.

```rexx
o = .writer~new
do m over 'charOut lineOut'~makearray(' ')
  call try 'o~' || m
  call try 'o~' || m || '("text")'
  call try 'o~' || m || '(1, 2, 3, 4, 5, 6)'
  call try 'o~' || m || '(.nil, , "x")'
end
call try '.outputstream~new~lineOut("x")'
say .better~new~lineOut('x') .better~new~charOut('y')
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> [' || r || ']'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class writer inherit outputstream
::class better inherit outputstream
::method lineOut
  return 0
::method charOut
  return 0
```

```output
o~charOut => 93.965
o~charOut("text") => 93.965
o~charOut(1, 2, 3, 4, 5, 6) => 93.965
o~charOut(.nil, , "x") => 93.965
o~lineOut => 93.965
o~lineOut("text") => 93.965
o~lineOut(1, 2, 3, 4, 5, 6) => 93.965
o~lineOut(.nil, , "x") => 93.965
.outputstream~new~lineOut("x") => 93.965
0 0
```

r[class.outputstream.unsupported]
`charIn`, `chars`, `lineIn`, `lines` and `position` make no sense for a stream
that is written: sent to an instance of `OutputStream`, or of a class that
inherits it and does not override them, each raises `SYNTAX 93.963`, whatever
the number and the values of the arguments are, none of which is read.

```rexx
o = .writer~new
do m over 'charIn chars lineIn lines position'~makearray(' ')
  call try 'o~' || m
  call try 'o~' || m || '(1)'
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
::class writer inherit outputstream
```

```output
o~charIn => 93.963
o~charIn(1) => 93.963
o~charIn(1, 2, 3, 4, 5, 6) => 93.963
o~charIn(.nil) => 93.963
o~chars => 93.963
o~chars(1) => 93.963
o~chars(1, 2, 3, 4, 5, 6) => 93.963
o~chars(.nil) => 93.963
o~lineIn => 93.963
o~lineIn(1) => 93.963
o~lineIn(1, 2, 3, 4, 5, 6) => 93.963
o~lineIn(.nil) => 93.963
o~lines => 93.963
o~lines(1) => 93.963
o~lines(1, 2, 3, 4, 5, 6) => 93.963
o~lines(.nil) => 93.963
o~position => 93.963
o~position(1) => 93.963
o~position(1, 2, 3, 4, 5, 6) => 93.963
o~position(.nil) => 93.963
```

r[class.outputstream.nop]
`open` and `close` do nothing: they take any number of arguments, read none of
them, and answer no value, so that their answer as a term raises
`SYNTAX 91.999`.

```rexx
o = .writer~new
o~open
o~close
o~open('write', 2, 3, 4, 5)
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
::class writer inherit outputstream
```

```output
statements ran
x = o~open => 91.999
x = o~close => 91.999
x = o~close(1, 2) => 91.999
```

r[class.outputstream.arrayout]
`arrayOut` writes the items of its one argument, a collection, one `lineOut`
for each item, and answers no value. The argument is sent `request` with the
argument `ARRAY`, which an `Array`, a `List` and a `Set` answer with an array
of their items, and the items are written in the order of that array, an empty
slot giving none; an argument whose answer is not an array, `.nil` among them,
raises `SYNTAX 98.913`. An item is sent to `lineOut` as it is, an object that
is not a string included, and what `lineOut` does with it is the business of
the class. A missing argument raises `SYNTAX 93.901` and more than one `SYNTAX
93.902`; there is no option for characters. An empty array writes nothing and
sends nothing, so it raises no error even where `lineOut` is abstract, and the
abstract `lineOut` raises `SYNTAX 93.965` for the first item otherwise.

```rexx
o = .sink~new
o~arrayout(.array~of('one', 'two'))
o~arrayout(.list~of('three'))
o~arrayout(.array~new(3)~~put('four', 1)~~put('six', 3))
o~arrayout(.array~new)
o~arrayout(.set~of('seven'))
o~arrayout(.array~of(.nil, 9))
say o~log
call try '.writer~new~arrayout(.array~new)'
call try '.writer~new~arrayout(.array~of(1))'
call try 'o~arrayout(.nil)'
call try 'o~arrayout(.object~new)'
call try 'o~arrayout'
call try 'o~arrayout(.array~of(1), "C")'
call try 'x = o~arrayout(.array~of("a"))'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class writer inherit outputstream
::class sink inherit outputstream
::attribute log
::method init
  self~log = ''
::method lineOut
  if arg(1) == .nil then self~log = self~log '.nil'
  else self~log = self~log arg(1)
  return 0
```

```output
 one two three four six seven .nil 9
.writer~new~arrayout(.array~new) => no error
.writer~new~arrayout(.array~of(1)) => 93.965
o~arrayout(.nil) => 98.913
o~arrayout(.object~new) => 98.913
o~arrayout => 93.901
o~arrayout(.array~of(1), "C") => 93.902
x = o~arrayout(.array~of("a")) => 91.999
```

r[class.outputstream.arrayout.sends]
`arrayOut` sends `request('ARRAY')` to its argument and then `lineOut` to the
receiver for each item, in order, with the item as the one argument and
nothing else. It sends no other message to the receiver, to the array or to an
item. The answer of a `lineOut` is not looked at, and neither is the NOTREADY
condition that a `lineOut` raises: the writing goes on with the next item.

```rexx
o = .sink~new
.local~log = ''
o~arrayout(.arr~of('a', 'b', 'c'))
say .local~log
say o~log
o = .sink~new
o~mode = 'answer'
o~arrayout(.array~of('x', 'y'))
say o~log
o = .sink~new
o~mode = 'notready'
o~arrayout(.array~of('p', 'q', 'r'))
say o~log
::class sink inherit outputstream
::attribute log
::attribute mode
::method init
  self~log = ''
  self~mode = ''
::method lineOut
  self~log = self~log 'lineOut(' || arg(1) || ')#' || arg()
  if self~mode = 'answer' then return 1
  if self~mode = 'notready' then raise notready
  return 0
::method charOut
  self~log = self~log 'charOut'
::method close
  self~log = self~log 'close'
::method open
  self~log = self~log 'open'
::class arr subclass array
::method request
  .local~log = .local~log 'request(' || arg(1) || ')'
  forward class (super)
```

```output
 request(ARRAY)
 lineOut(a)#1 lineOut(b)#1 lineOut(c)#1
 lineOut(x)#1 lineOut(y)#1
 lineOut(p)#1 lineOut(q)#1 lineOut(r)#1
```
