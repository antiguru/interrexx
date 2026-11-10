---
element: "Pointer"
kind: class
rexxref: "Builtin Classes > Utility Classes > Pointer Class; Builtin Classes > Utility Classes > Pointer Class > new (Class Method); Builtin Classes > Utility Classes > Pointer Class > Comparison Methods; Builtin Classes > Utility Classes > Pointer Class > isNull"
---

# Pointer

A `Pointer` wraps a native address, an address of memory outside the
interpreter, so that native code can hand it to a Rexx program and receive it
back. A program written in Rexx cannot make a pointer or look into one: it can
keep it in a variable, pass it on, and ask it two things, whether the address
is the null address and whether two pointers hold the same address. A routine
or a method of an external library is what makes the pointer, so what this
page states about the instance methods is what a program can ask of a pointer
that such a routine returned, and the examples can show only the class and its
methods, as a program has no way to make an instance.

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
the inherited `new` from it raises the same error. A pointer therefore exists
only where native code made one.

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
`=` and `==` answer 1 when the receiver and the argument are pointers that
hold the same address, and `\=` and `\==` answer 1 when they hold different
addresses and 0 when they hold the same one. The four methods are defined by
the class itself, not by `Object`, so a pointer is compared by its address and
not by its identity. The example shows that the four methods belong to the
class; a program has no pointer to compare, so it cannot show the answers.

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
`isNull` answers 1 when the address that the pointer holds is the null
address, which is 0, and 0 when it holds any other address. The example shows
the method; a program has no pointer to ask.

```rexx
say .pointer~method('ISNULL')~class~id
say .pointer~method('ISNULL')~scope == .pointer
```

```output
Method
1
```
