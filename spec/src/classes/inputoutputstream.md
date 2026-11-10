---
element: "InputOutputStream"
kind: class
rexxref: "Builtin Classes > Stream Classes > InputOutputStream Class; Builtin Classes > Stream Classes > InputStream Class; Builtin Classes > Stream Classes > OutputStream Class"
---

# InputOutputStream

`InputOutputStream` is a mixin class for a stream that is both read and written.
It defines no method of its own: it inherits the mixin classes
[`InputStream`](inputstream.md) and [`OutputStream`](outputstream.md), in that
order, and so has the methods of both, and the stream classes of the library
that read and write, [`Stream`](stream.md) among them, are built on it. This
page states what the order means for a class that inherits it, since the two
mixin classes define the same names. The examples define their own classes.

## The class

r[class.inputoutputstream.creation]
`InputOutputStream` has no instance method of its own and no class method
besides the `new` that every class has. Its superclasses are `Object`,
`InputStream` and `OutputStream`, in that order ([the hierarchy
rule](r:class.index.hierarchy)). It is a mixin class (see [the mixin
rule](r:class.index.mixins)), and a class that inherits it has it among its
superclasses, in the place that the `INHERIT` of its directive gives it. `new`
makes an instance, an object with the methods of the two mixin classes and no
state, whose string value is `an InputOutputStream`; the `init` of `Object`
accepts no argument, so `new` with one raises `SYNTAX 93.902` ([the new
rule](r:class.index.new)).

```rexx
s = ''
do c over .inputoutputstream~superclasses
  s = s c~id
end
say s~strip .inputoutputstream~queryMixinClass
n = 0
m = .inputoutputstream~methods(.nil)
do while m~available
  n += 1
  m~next
end
say n
o = .inputoutputstream~new
say o~class~id o~string
say o~isA(.inputstream) o~isA(.outputstream) o~hasMethod('arrayIn'),
  o~hasMethod('arrayOut')
signal on syntax name bad
x = .inputoutputstream~new(1)
bad: say condition('o')~code
s = ''
do c over .duplex~superclasses
  s = s c~id
end
say s~strip
::class duplex inherit inputoutputstream
```

```output
Object InputStream OutputStream 1
0
InputOutputStream an InputOutputStream
1 1 1 1
93.902
Object InputOutputStream
```

r[class.inputoutputstream.methods]
A message to an instance of `InputOutputStream`, or of a class that inherits it
and does not override the method, reaches the method of `InputStream` when
`InputStream` has one of that name, and only then the method of `OutputStream`,
because `InputStream` comes first. So `charIn`, `chars`, `lineIn` and `lines`
raise `SYNTAX 93.965`, as [the abstract methods of
`InputStream`](r:class.inputstream.abstract) do; `charOut`, `lineOut` and
`position` raise `SYNTAX 93.963`, as [the methods of `InputStream` that make no
sense](r:class.inputstream.unsupported) do, not `93.965`, which the abstract
`charOut` and `lineOut` of `OutputStream` would raise; `open` and `close` do
nothing, as [those of `InputStream`](r:class.inputstream.nop) do; `arrayIn` is
[the `arrayIn` of `InputStream`](r:class.inputstream.arrayin) and `arrayOut`
[the `arrayOut` of `OutputStream`](r:class.outputstream.arrayout). A class that
inherits `InputOutputStream` therefore overrides the six methods that read and
write, `charIn`, `charOut`, `chars`, `lineIn`, `lineOut` and `lines`, and
`position` where it can be positioned.

```rexx
o = .duplex~new
names = 'arrayIn arrayOut charIn charOut chars close lineIn lineOut lines',
  'open position'
do m over names~makearray(' ')
  if m = 'arrayOut' then call try 'o~arrayOut(.array~of("x"))'
  else call try 'o~' || m
end
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> [' || r || ']'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class duplex inherit inputoutputstream
```

```output
o~arrayIn => 93.965
o~arrayOut(.array~of("x")) => 93.963
o~charIn => 93.965
o~charOut => 93.963
o~chars => 93.965
o~close => 91.999
o~lineIn => 93.965
o~lineOut => 93.963
o~lines => 93.965
o~open => 91.999
o~position => 93.963
```

r[class.inputoutputstream.complete]
A class that inherits `InputOutputStream` and overrides `lineIn` and `lineOut`
has working `arrayIn` and `arrayOut`: `arrayIn` reads with its `lineIn` as [the
rule of `InputStream`](r:class.inputstream.arrayin.sends) says, and `arrayOut`
writes with its `lineOut` as [the rule of
`OutputStream`](r:class.outputstream.arrayout.sends) says. The two methods do
not know of each other.

```rexx
t = .tape~new
t~arrayout(.array~of('first', 'second', 'third'))
a = t~arrayin
say a~items a~makeString('L', ',')
say t~log
::class tape inherit inputoutputstream
::attribute log
::attribute data
::attribute pos
::method init
  self~log = ''
  self~data = .array~new
  self~pos = 0
::method lineOut
  self~log = self~log 'out'
  self~data~append(arg(1))
  return 0
::method lineIn
  self~log = self~log 'in'
  self~pos = self~pos + 1
  if self~pos > self~data~items then raise notready
  return self~data[self~pos]
```

```output
3 first,second,third
 out out out in in in in
```
