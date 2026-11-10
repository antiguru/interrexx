---
element: "Buffer"
kind: class
rexxref: "Builtin Classes > Utility Classes > Buffer Class; Builtin Classes > Utility Classes > Buffer Class > new (Class Method)"
---

# Buffer

A `Buffer` is a block of storage that the interpreter manages for native code:
a routine or a method written against the native interface asks the
interpreter for one, fills it, and hands it back to the program. A program
written in Rexx cannot make a buffer, so the class holds no method that such a
program could send to an instance, and a Rexx program that meets one only
passes it on. What a program can observe of the class is stated here: where it
stands in the class hierarchy and that `new` is refused. The `MutableBuffer`
class is the buffer a Rexx program uses for text; [its page](mutablebuffer.md)
is the one to read for that.

## The class

r[class.buffer.creation]
`Buffer` is a class whose only superclass is `Object`, which inherits no mixin
class ([the hierarchy rule](r:class.index.hierarchy)). It defines no instance
method of its own, so an instance understands the methods of `Object` and no
others, and its class side has the methods that every class has. It is not a
subclass of `String` or of `MutableBuffer`, and has no subclass among the
classes of the library.

```rexx
say .buffer~id .buffer~class~id
do c over .buffer~superclasses
  say c~id
end
say .buffer~superclasses~items .buffer~subclasses~items
s = .buffer~methods(.nil)
say s~available
say .buffer~isSubclassOf(.string) .buffer~isSubclassOf(.mutablebuffer)
```

```output
Buffer Class
Object
1 0
0
0 0
```

r[class.buffer.new]
`new` raises `SYNTAX 93.967` whatever the arguments are, none or any number of
them, and makes no object. A subclass that a program declares has the same
`new`: a `new` that the subclass declares itself runs first, and the call of
the inherited `new` from it raises the same error. A buffer therefore exists
only where native code made one.

```rexx
call try '.buffer~new'
call try '.buffer~new(1)'
call try '.buffer~new(1, 2)'
call try '.kept~new'
call try '.loud~new'
say .buffer~hasMethod('NEW')
exit
try:
  signal on syntax name bad
  interpret 'rr =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class kept subclass buffer
::class loud subclass buffer
::method new class
  say 'new runs'
  return self~new:super
```

```output
.buffer~new => 93.967
.buffer~new(1) => 93.967
.buffer~new(1, 2) => 93.967
.kept~new => 93.967
new runs
.loud~new => 93.967
1
```
