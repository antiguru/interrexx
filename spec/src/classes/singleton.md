---
element: "Singleton"
kind: class
rexxref: "Builtin Classes > Utility Classes > Singleton Class (Metaclass); Builtin Classes > Utility Classes > Singleton Class (Metaclass) > new (Class Method)"
---

# Singleton

`Singleton` is a metaclass for the programs that want a class with one
instance. A class that names it in the `METACLASS` option of its
[`::CLASS`](../directives/class.md) directive makes one object, the first
time it is sent `new`, and answers that same object to every later `new`.
The class has no other use: a program that needs the object only has to ask
the class for a new one.

The place of `Singleton` in the hierarchy is stated in [the hierarchy
rule](r:class.index.hierarchy), and [the mixin rule](r:class.index.mixins)
lists it among the mixin classes. This page specifies what it does for the
classes that use it.

## The class

r[class.singleton.creation]
`Singleton` is a metaclass: `isMetaclass` answers `1`, and its only instance
method is `new`, the class-side `new` of every class whose metaclass it is.
A class made with `METACLASS Singleton` has `Singleton` as its metaclass and
as its class, and is not itself a metaclass.

```rexx
s = .singleton
say s~isMetaclass s~queryMixinClass s~class~id
m = s~methods(.nil)
n = 0
do while m~available
  say m~index
  n += 1
  m~next
end
say n
say .one~class~id .one~metaclass~id (.one~class == .singleton)
say .one~isMetaclass .one~new~class~id
::class one metaclass singleton
```

```output
1 1 Class
NEW
1
Singleton Singleton 1
0 ONE
```

## Methods

r[class.singleton.new]
The first `new` that a class with the metaclass `Singleton` receives makes
the object as `Class` does ([the `new` rule](r:class.index.new)): it makes an
instance and sends it `init` with the arguments of `new`. The class keeps
the object, and every later `new`, with any arguments, answers that object
and sends no `init`. When `init` raises an error, no object is kept, and the
next `new` makes one again and sends `init` again.

```rexx
.environment['MADE'] = 0
a = .one~new(7, 8)
b = .one~new
c = .one~new(9)
say a~args (a == b) (a == c) .environment~made
call try '.three~new'
call try '.three~new'
say .four~new~n .four~new~n .four~new~n
exit
try:
  signal on syntax name bad
  interpret 'rr =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class one metaclass singleton
::method init
  expose args
  use arg first, second
  args = first second
  .environment['MADE'] = .environment['MADE'] + 1
::method args
  expose args
  return args
::class three metaclass singleton
::method init
  raise syntax 93.900 array ('boom')
::class four metaclass singleton
::method n
  expose count
  if \var('count') then count = 0
  count += 1
  return count
```

```output
7 8 1 1 1
.three~new => 93.900
.three~new => 93.900
1 2 3
```

r[class.singleton.new.direct]
`new` sent to `Singleton` itself, with a name, answers an object that has that
name for its `id` and `Singleton` for its class, and that is not a class made by
a `::CLASS` directive with the metaclass `Singleton`: sent `new`, with any
arguments, it raises `SYNTAX 97.1`. `new` with no argument raises `SYNTAX
93.901`.

```rexx
call try 'c = .singleton~new("x"); say c~class~id c~id; x = c~new'
call try 'c = .singleton~new("y"); x = c~new(1, 2)'
call try 'c = .singleton~new'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
Singleton x
c = .singleton~new("x"); say c~class~id c~id; x = c~new => 97.1
c = .singleton~new("y"); x = c~new(1, 2) => 97.1
c = .singleton~new => 93.901
```

r[class.singleton.inherit]
A subclass of a class with the metaclass `Singleton`, whether a `::CLASS`
directive or `subclass` makes it, has the metaclass too and so has its own
object: its `new` does not answer the object of the superclass, and the
superclass does not answer the subclass's.

```rexx
a = .one~new
s = .sub~new
say s~class~id (a == s) (s == .sub~new) .sub~class~id .sub~metaclass~id
t = .one~subclass('Late')
say t~class~id (t~new == t~new) (t~new == a) (t~new == s)
::class one metaclass singleton
::class sub subclass one
```

```output
SUB 0 1 Singleton Singleton
Singleton 1 0 0
```

r[class.singleton.copy]
Before it sends `init`, the first `new` of a class with the metaclass
`Singleton` defines on that class an instance method `COPY` that answers `self`,
in place of any `copy` method the class defines itself. So the object answers
itself to `copy`, with or without arguments. A `copy` sent with a scope
override, such as `copy:.object`, bypasses that method and makes a second
instance. An object of a class without the metaclass is copied as usual.

```rexx
a = .one~new
b = .mine~new
say (a~copy == a) (a~copy~copy == a) (a~copy(1) == a) (a~copy('x', 2) == a)
say (b~copy == b) b~copy~class~id
c = .plain~new
say (c~copy == c) c~copy~class~id
::class one metaclass singleton
::class mine metaclass singleton
::method copy
  return 'mine copy'
::class plain
```

```output
1 1 1 1
1 MINE
0 PLAIN
```

```rexx
call own .plain1
x = .plain1~new
call own .plain1
say .plain1~methods(.nil)~allItems[1]~source~makeString('l', '|')
say (x~copy == x) (x~copy:.object == x) x~copy:.object~class~id
exit
own: procedure
  say arg(1)~id '['arg(1)~methods(.nil)~allIndexes~makeString('l', ' ')']'
  return
::class plain1 metaclass singleton
```

```output
PLAIN1 []
PLAIN1 [COPY]
return self
1 0 PLAIN1
```
