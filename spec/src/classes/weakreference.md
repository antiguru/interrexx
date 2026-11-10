---
element: "WeakReference"
kind: class
rexxref: "Builtin Classes > Utility Classes > WeakReference Class; Builtin Classes > Utility Classes > WeakReference Class > new (Class Method); Builtin Classes > Utility Classes > WeakReference Class > value"
deviations: [6, 36]
---

# WeakReference

A `WeakReference` refers to an object without keeping it alive. As long as
anything else in the program refers to the object, the reference answers it;
once nothing else does, the object may be reclaimed, and from then on the
reference answers `.nil`. When the reclaiming happens is the business of the
interpreter and not of the program, so this page states what the reference
answers while the object is held and that it answers `.nil` for good after the
object has gone, and leaves the moment of the reclaiming unspecified.

## The class

r[class.weakreference.creation]
`WeakReference` is a class whose only superclass is `Object`, and it inherits
no mixin class ([the hierarchy rule](r:class.index.hierarchy)). Its instance
side defines `value` and no other method of its own. An instance is made with
`new`, [stated below](r:class.weakreference.new).

```rexx
say .weakreference~class~id .weakreference~superclasses~items
say .weakreference~superclasses~at(1)~id
s = .weakreference~methods(.nil)
names = .array~new
do while s~available
  names~append(s~index)
  s~next
end
say names~sort~makeString('l', ' ')
```

```output
Class 1
Object
VALUE
```

r[class.weakreference.new]
`new` makes a reference to the object that is its first argument, which is
required (`SYNTAX 93.903`). Any object can be referred to: `.nil`, a string, a
number, a class, another `WeakReference` or an ordinary object. The new object
is sent `init` with the arguments of `new` after the first, so that `new` with
a second argument raises `SYNTAX 93.902` from the `init` of `Object` unless a
subclass declares an `init` that takes it; `init` is sent after the reference
exists, and a subclass whose `init` does not call its superclass still has a
reference that answers the object. A subclass receiver gets an instance of
itself, and a subclass that declares its own `value` has it called instead of
the method of this class.

```rexx
o = .directory~new
w = .weakreference~new(o)
say w~class~id (w~value == o) (w~value~identityHash == o~identityHash)
say (.weakreference~new(.nil)~value == .nil)
say .weakreference~new('text')~value .weakreference~new(12)~value
say .weakreference~new(w)~value~class~id,
  (.weakreference~new(.object)~value == .object)
say w~hasMethod('VALUE') w~hasMethod('NEW') .weakreference~hasMethod('NEW')
call try '.weakreference~new'
call try '.weakreference~new(1, 2)'
call try 'w~value(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
WeakReference 1 1
1
text 12
WeakReference 1
1 0 1
.weakreference~new => 93.903
.weakreference~new(1, 2) => 93.902
w~value(1) => 93.902
```

```rexx
o = .directory~new
w = .noted~new(o)
say w~class~id (w~value == o) w~note
w = .noted~new(o, 'x', 'y')
say w~note
w = .loud~new(o)
say (w~value == o)
say .shadow~new(o)~value
call try ".weakreference~new(o, 'extra')"
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class noted subclass weakreference
::attribute note
::method init
  expose note
  note = 'init got' arg()
::class loud subclass weakreference
::method init
  say 'loud init runs'
::class shadow subclass weakreference
::method value
  return 'shadowed'
```

```output
NOTED 1 init got 0
init got 2
loud init runs
1
shadowed
.weakreference~new(o, 'extra') => 93.902
```

r[class.weakreference.value]
`value` answers the object the reference was made for, the very object and not
a copy, and takes no argument (`SYNTAX 93.902`). It answers it for as long as
the program holds the object in any other way, as in a variable, an attribute,
an array or another object that is itself held. When the program no longer
holds the object in any other way, the object can be reclaimed by a collection
that the program does not control; `value` then answers `.nil`, and keeps
answering `.nil` at every later call. When the collection happens is not
specified ([Deviation 6](r:class.weakreference.deviation.6)). The first
example holds the object while the program allocates; the others drop it and
allocate until the answer changes, with a bound that no program should reach,
and all print only answers that do not depend on when.

```rexx
o = .directory~new
w = .weakreference~new(o)
do 100000
  x = .array~new(10)
end
say (w~value == o) (w~copy~value == o)
c = w~copy
say (c~value == w~value)
```

```output
1 1
1
```

```rexx
o = .directory~new
w = .weakreference~new(o)
say (w~value == o)
drop o
do 2000000 while w~value \== .nil
  x = .array~new(10)
end
say (w~value == .nil) (w~value == .nil)
```

```output
1
1 1
```

```rexx
w = made()
do 2000000 while w~value \== .nil
  x = .array~new(10)
end
say (w~value == .nil) (w~value == .nil) (w~value == .nil)
exit
made: return .weakreference~new(.directory~new)
```

```output
1 1 1
```

r[class.weakreference.copy]
`copy` answers a new `WeakReference` of the same class that refers to the same
object, with the object variables of the receiver as [the copy
rule](r:class.object.copy) says, and takes no argument (`SYNTAX 93.902`). The
two references are different objects, so `==` of one with the other is false,
and both answer the object while it is held and `.nil` after it was reclaimed.

```rexx
o = .directory~new
w = .noted~new(o)
c = w~copy
say c~class~id c~note (c == w) (c~value == o) (w~value == c~value)
call try "w~copy(1)"
exit
try:
  signal on syntax name bad
  interpret 'rr =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class noted subclass weakreference
::attribute note
::method init
  expose note
  note = 'init got' arg()
```

```output
NOTED init got 0 0 1 1
w~copy(1) => 93.902
```

## Deviations

r[class.weakreference.deviation.6]
At which point of the program a reference to a dropped object starts to answer
`.nil` is not specified. A forced collection can reclaim an object dropped
just after it was made, so interrexx can clear the reference earlier than
ooRexx, which keeps its most recent allocations alive through a forced
collection ([Deviation 6](../deviations.md)). The difference is only the
moment; a reference answers its object for as long as the program holds it,
and answers `.nil` for good once it has cleared. The example allocates until
the answer changes and prints what does not depend on the moment.

```rexx
o = .array~new
w = .weakreference~new(o)
say (w~value == o)
drop o
do 2000000 while w~value \== .nil
  x = .directory~new
end
say (w~value == .nil)
```

```output
1
1
```

r[class.weakreference.deviation.36]
A string or a number that is not an instance of a subclass is value-based, and
whether equal values are one object is not specified, so when a reference to
one answers `.nil` is not specified either ([Deviation 36](../deviations.md)).
A reference to a string answers it for as long as the program holds that
string, which the example shows; whether an equal string held elsewhere keeps
the reference alive is not specified.

```rexx
s = 'abc' || 'def'
w = .weakreference~new(s)
n = 12 + 30
v = .weakreference~new(n)
say w~value v~value
```

```output
abcdef 42
```
