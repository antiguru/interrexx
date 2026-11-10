---
element: "Pointer"
kind: class
rexxref: "Builtin Classes > Utility Classes > Pointer Class; Builtin Classes > Utility Classes > Pointer Class > new (Class Method); Builtin Classes > Utility Classes > Pointer Class > Comparison Methods; Builtin Classes > Utility Classes > Pointer Class > isNull"
---

# Pointer

A program written in Rexx cannot make a `Pointer`: `new` is refused, so no
instance is reachable from a Rexx program, and what the instance methods do
cannot be observed. This page states what a program can observe of the class:
where it stands in the class hierarchy, which instance methods it defines, and
that `new` is refused.

## The class

r[class.pointer.creation]
`Pointer` is a class whose only superclass is `Object`, and it inherits no
mixin class ([the hierarchy rule](r:class.index.hierarchy)). Its instance side
defines `=`, `==`, `\=`, `\==` and `isNull`, and no other method. It is not a
subclass of `String` or of `Buffer`, and has no subclass among the classes of
the library.

```rexx
say .pointer~id .pointer~class~id
do c over .pointer~superclasses
  say c~id
end
say .pointer~superclasses~items .pointer~subclasses~items
s = .pointer~methods(.nil)
names = .array~new
do while s~available
  names~append(s~index)
  s~next
end
say names~sort~makeString('l', ' ')
say .pointer~isSubclassOf(.string) .pointer~isSubclassOf(.buffer)
```

```output
Pointer Class
Object
1 0
= == ISNULL \= \==
0 0
```

r[class.pointer.new]
`new` raises `SYNTAX 93.967` whatever the arguments are, none or any number of
them, and makes no object. A subclass that a program declares has the same
`new`: a `new` that the subclass declares itself runs first, and the call of
the inherited `new` from it raises the same error. No program written in
Rexx can therefore make a pointer.

```rexx
call try '.pointer~new'
call try '.pointer~new(1)'
call try '.pointer~new(1, 2)'
call try '.kept~new'
call try '.loud~new'
exit
try:
  signal on syntax name bad
  interpret 'rr =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class kept subclass pointer
::class loud subclass pointer
::method new class
  say 'new runs'
  return self~new:super
```

```output
.pointer~new => 93.967
.pointer~new(1) => 93.967
.pointer~new(1, 2) => 93.967
.kept~new => 93.967
new runs
.loud~new => 93.967
```

## Methods

r[class.pointer.comparison]
`Pointer` defines its own `=`, `==`, `\=` and `\==`, which take the place of
those of `Object` for an instance. This book states nothing about what they
answer, as no instance is reachable from a Rexx program ([the new
rule](r:class.pointer.new)). The example shows that the four methods belong
to the class.

```rexx
do n over '= == \= \== isNull'~makeArray(' ')
  m = .pointer~method(n)
  say n m~class~id (m~scope == .pointer)
end
```

```output
= Method 1
== Method 1
\= Method 1
\== Method 1
isNull Method 1
```

r[class.pointer.isnull]
`Pointer` defines `isNull`. This book states nothing about what it answers,
as no instance is reachable from a Rexx program ([the new
rule](r:class.pointer.new)). The example shows the method.

```rexx
say .pointer~method('ISNULL')~class~id
say .pointer~method('ISNULL')~scope == .pointer
```

```output
Method
1
```
