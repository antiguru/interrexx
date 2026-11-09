---
element: "SetCollection"
kind: class
rexxref: "Builtin Classes > Collection Classes > SetCollection Class"
---

# SetCollection

`SetCollection` is the mixin class that marks a collection as a set: one in
which the index of an entry and its item are the same object. It declares no
method. A class inherits it to say what kind of collection it is, and the
classes of the library that do are `Set` and `Bag`, each of which also
inherits [`MapCollection`](mapcollection.md). The page says what the mixin is
and what it changes for the programs that combine collections; the behavior
of a set or a bag is on the pages of those classes.

## The class

r[class.setcollection.creation]
`SetCollection` is a mixin class whose superclass is `Collection`, and
`queryMixinClass` answers 1 for it ([the mixin rule](r:class.index.mixins)).
`new` is not refused and makes an object of the class that received it, as
[the new rule](r:class.index.new) says. The class declares no method, on
either side: `methods` with `.nil` lists no entry, and an instance understands
the methods of [`Collection`](collection.md) and nothing more, so the abstract
`put`, `allIndexes` and `allItems` of that page are what an instance of
a class that inherits only this mixin has for them, and its constraint that
index and item are the same is a promise of the class that inherits, not
something the mixin enforces.

```rexx
c = .setcollection~new
say c~class~id .setcollection~queryMixinClass .setcollection~superclass~id
n = 0
s = .setcollection~methods(.nil)
do while s~available
  n += 1
  s~next
end
say n 'own methods'
say c~hasMethod('UNION') c~hasMethod('PUT')
say c~hasMethod('OF') c~hasMethod('APPENDALL')
signal on syntax name bad
c~put(1, 1)
bad: say condition('o')~code
```

```output
SetCollection 1 Collection
0 own methods
1 1
0 0
93.965
```

r[class.setcollection.operand]
Inheriting `SetCollection` does not change how an instance is read when it is
the argument of a [set operation](r:class.collection.operand). The operations
ask `isA` about `OrderedCollection` and `MapCollection`, and `subset` about
`Set` as well, which inheriting `SetCollection` does not make an instance ([the
isA rule](r:class.collection.operand.isa)), so an instance of a class that
inherits `SetCollection` and not `MapCollection` is read like any other
collection that is not ordered, through `makeArray`, and an instance of a class
that inherits both is read as a map, through `supplier` or `allIndexes`.

```rexx
a = .box~new
do k over .array~of(.settype~new, .both~new)
  say '==' k~class~id
  r = a~union(k)
  r = a~difference(k)
end
::class box inherit collection
::method copy
  return self~class~new
::method put
  nop
::method hasindex
  return 0
::method remove
  nop
::class settype inherit setcollection
::method makearray
  say '  makeArray'
  return .array~new
::method items
  return 0
::class both inherit mapcollection setcollection
::method supplier
  say '  supplier'
  return .supplier~new(.array~new, .array~new)
::method allindexes
  say '  allIndexes'
  return .array~new
::method items
  return 0
```

```output
== SETTYPE
  makeArray
  makeArray
== BOTH
  supplier
  allIndexes
```
