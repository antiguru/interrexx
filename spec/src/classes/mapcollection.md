---
element: "MapCollection"
kind: class
rexxref: "Builtin Classes > Collection Classes > MapCollection Class; Builtin Classes > Collection Classes > MapCollection Class > of (Class Method); Builtin Classes > Collection Classes > MapCollection Class > makeArray; Builtin Classes > Collection Classes > MapCollection Class > putAll"
deviations: [33]
---

# MapCollection

`MapCollection` is the mixin class of the collections that map an index to an
item: the directories, the tables, the relations, the sets and the other
classes that
[the class hierarchy](index.md#the-class-hierarchy) lists as inheriting it. It
inherits [`Collection`](collection.md), and so has every method of that page,
and it adds three: the class method `of`, which builds a collection from
index and item pairs, and the instance methods `makeArray`, which answers the
indexes where `Collection` answers the items, and `putAll`, which copies
every entry of another collection in. The mixin also decides how a map is
read when it is the argument of a set operation, which
[the operand rule](r:class.collection.operand) states.

A class that inherits only `MapCollection` has to declare `put` and
`allIndexes`, which are abstract in `Collection`; the examples use the class
`store`, which keeps its entries in a directory and declares those two,
`allItems` and a `string` method that lists its entries in index order.

## The class

r[class.mapcollection.creation]
`MapCollection` is a mixin class whose superclass is `Collection`, and
`queryMixinClass` answers 1 for it ([the mixin rule](r:class.index.mixins)).
`new` is not refused and makes an object of the class that received it, as
[the new rule](r:class.index.new) says. The instance methods that the class
declares are `makeArray` and `putAll`, and `of` is declared on the class side:
the class and every class that inherits it understand `of`, and no instance
does, so `hasMethod('OF')` answers 1 for the class and 0 for an instance.
Everything else an instance understands it gets from
[`Collection`](collection.md).

```rexx
c = .mapcollection~new
say c~class~id .mapcollection~queryMixinClass .mapcollection~superclass~id
say .mapcollection~hasMethod('OF') c~hasMethod('OF')
say c~hasMethod('MAKEARRAY') c~hasMethod('PUTALL') c~hasMethod('UNION')
say .store~hasMethod('OF') .store~new~hasMethod('OF')
::class store inherit mapcollection
```

```output
MapCollection 1 Collection
1 0
1 1 1
1 0
```

## Class methods

### of

r[class.mapcollection.of]
`of(pair, ...)` answers a new object of the class that received it, made by
`new` and holding one entry for each argument: an argument is an array of
exactly two items, the index first and then the item, and the entry is the
second item stored at the first. The arguments are taken from left to right,
so a later pair with the same index replaces the item of an earlier one
when the class's `put` replaces. With no argument the result is an empty
collection. A trailing omitted argument counts for nothing. A class that
declares no `put` of its own gets the abstract `put` of `Collection`, so
`.mapcollection~of` with a pair raises `SYNTAX 93.965`.

```rexx
s = .store~of(.array~of('red', 1), .array~of('green', 2), .array~of('red', 3))
say s~class~id '|' s
say .store~of~class~id '['.store~of']'
say .store~of(.array~of('x', 'y'), )
say .mapcollection~of~class~id
signal on syntax name bad
say .mapcollection~of(.array~of('x', 'y'))
bad: say condition('o')~code
::class store inherit mapcollection
::method init
  expose d
  d = .directory~new
::method put
  expose d
  use arg item, index
  d[index] = item
::method allindexes
  expose d
  return d~allindexes~sort
::method string
  expose d
  s = ''
  do i over d~allindexes~sort; s = s i'='d[i]; end
  return s~strip
```

```output
STORE | green=2 red=3
STORE []
x=y
MapCollection
93.965
```

r[class.mapcollection.of.sends]
`of` sends `new` to the class that received it, with no argument, before it
looks at any argument. For each argument in order it then sends `put` to the
new object with the item and then the index. What `new` answers is the result,
and `init` is not sent again.

```rexx
s = .tstore~of(.array~of('a', 1), .array~of('b', 2))
say s~items
::class tstore inherit mapcollection
::method new class
  say 'new' arg()
  forward class (super)
::method init
  expose d
  say 'init' arg()
  d = .directory~new
::method put
  expose d
  use arg item, index
  say 'put' item index
  d[index] = item
::method allindexes
  expose d
  return d~allindexes
```

```output
new 0
init 0
put 1 a
put 2 b
2
```

r[class.mapcollection.of.error]
Each argument is checked when it is reached, after the pairs before it have
been stored. An argument that is not a single-dimensional array, `.nil` and a
string included, raises `SYNTAX 88.923`. An array that does not have exactly
two items raises `SYNTAX 88.924`, and an item counts only when it is present,
so `.array~of('a', )` is too short. An argument that is omitted and is
followed by another raises `SYNTAX 93.903`.

```rexx
call try ".store~of(.array~of('a', 1), 'x')"
call try ".store~of(.nil)"
call try ".store~of(.array~of('a'))"
call try ".store~of(.array~of('a', 1, 2))"
call try ".store~of(.array~of('a', ))"
call try ".store~of(.array~of('a', 1), , .array~of('b', 2))"
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class store inherit mapcollection
::method init
  expose d
  d = .directory~new
::method put
  expose d
  use arg item, index
  say '  put' item index
  d[index] = item
```

```output
  put 1 a
.store~of(.array~of('a', 1), 'x') => 88.923
.store~of(.nil) => 88.923
.store~of(.array~of('a')) => 88.924
.store~of(.array~of('a', 1, 2)) => 88.924
.store~of(.array~of('a', )) => 88.924
  put 1 a
.store~of(.array~of('a', 1), , .array~of('b', 2)) => 93.903
```

## Methods

### makeArray

r[class.mapcollection.makearray]
`makeArray` answers what `allIndexes` answers, and the very object: it is not
copied and its kind is not checked. Where `Collection` answers the items, a
map answers the indexes. Arguments are not checked.

```rexx
mine = .array~of('i1', 'i2')
s = .store~new(mine)
r = s~makeArray
say r~class~id r~items (r == mine)
say s~makeArray(1, 2)~items
::class store inherit mapcollection
::method init
  expose list
  use arg list
::method allindexes
  expose list
  return list
::method allitems
  return .array~of('item 1', 'item 2')
```

```output
Array 2 1
2
```

r[class.mapcollection.makearray.sends]
`makeArray` sends `allIndexes` to the receiver, with no argument, and nothing
else.

```rexx
s = .spy~new
say s~makeArray
::class spy inherit mapcollection
::method allindexes
  say 'allIndexes' arg()
  return 'the answer'
::method allitems
  say 'allItems'
  return .array~new
```

```output
allIndexes 0
the answer
```

### putAll

r[class.mapcollection.putall]
`putAll(source)` stores every entry of `source` into the receiver and answers
the receiver. The entries are those of the supplier that `source` answers to
`supplier`, in the order the supplier gives them, each stored by `put` with
the supplier's item and then its index; an entry of the source whose index the
receiver has is stored over by `put` in the way that `put` of the receiver
decides. `source` need not be a collection: any object that understands
`supplier` is accepted. The method takes exactly one argument: none raises
`SYNTAX 93.901` and a second raises `SYNTAX 93.902`.

```rexx
s = .store~new
src = .array~of('a', 'b', 'c')
r = s~putAll(src)
say (r == s) '|' s
say s~putAll(.array~of('X'))
call try 's~putAll'
call try 's~putAll(1, 2)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class store inherit mapcollection
::method init
  expose d
  d = .directory~new
::method put
  expose d
  use arg item, index
  d[index] = item
::method string
  expose d
  s = ''
  do i over d~allindexes~sort; s = s i'='d[i]; end
  return s~strip
```

```output
1 | 1=a 2=b 3=c
1=X 2=b 3=c
s~putAll => 93.901
s~putAll(1, 2) => 93.902
```

r[class.mapcollection.putall.sends]
`putAll` sends `supplier` to `source`, with no argument, and then reads the
answer by messages, whatever its class: it sends `available`, and while that
answers true sends `item`, then `index`, then `put` to the receiver with the
item and the index, then `next`, and asks `available` again. The answer of
`supplier` need not be a `Supplier`: an object that answers those four
messages does. Of the four, `available` is what the object has to understand
at once; a missing `item`, `index` or `next` raises `SYNTAX 97.1` when it is
sent, after the entries before it have been stored.

```rexx
s = .tstore~new
say s~putAll(.source~new)~class~id
::class source
::method supplier
  say 'supplier' arg()
  return .walker~new
::class walker
::method init
  expose n
  n = 1
::method available
  expose n
  say 'available' n
  return n <= 2
::method item
  expose n
  say 'item'
  return 'v' || n
::method index
  expose n
  say 'index'
  return 'i' || n
::method next
  expose n
  say 'next'
  n += 1
::class tstore inherit mapcollection
::method put
  say 'put' arg(1) arg(2)
```

```output
supplier 0
available 1
item
index
put v1 i1
next
available 2
item
index
put v2 i2
next
available 3
TSTORE
```

r[class.mapcollection.putall.error]
A `source` that does not understand `supplier` raises `SYNTAX 93.948`, and so
does an answer of `supplier` that does not understand `available`, as an
object does that has no such method and no `UNKNOWN` method. The check is for
understanding the message: an object whose `UNKNOWN` method handles it is
accepted. Both are raised before the receiver has been changed.

```rexx
s = .store~new
call try 's~putAll(.nil)'
call try 's~putAll(.object~new)'
call try 's~putAll(.deaf~new)'
call try 's~putAll(.lenient~new)'
call try 's~putAll(.array~of(1))'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> ok' s~items
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class deaf
::method supplier
  return .object~new
::class lenient
::method supplier
  return .quiet~new
::class quiet
::method unknown
  return 0
::class store inherit mapcollection
::method init
  expose d
  d = .directory~new
::method put
  expose d
  use arg item, index
  d[index] = item
::method allindexes
  expose d
  return d~allindexes
```

```output
s~putAll(.nil) => 93.948
s~putAll(.object~new) => 93.948
s~putAll(.deaf~new) => 93.948
s~putAll(.lenient~new) => ok 0
s~putAll(.array~of(1)) => ok 1
```

## Deviations

r[class.mapcollection.deviation.33]
The order in which a hash collection yields its entries is not specified
([Deviation 33](../deviations.md#33-hash-iteration-order)), and so is not the
order in which `putAll` stores the entries of a directory or another hash
collection given as `source`, nor the order of the indexes that `makeArray`
answers for one. The entries stored, and how often, are specified.

```rexx
s = .store~new
src = .directory~new
src['k1'] = 'first'
src['k2'] = 'second'
src['k3'] = 'third'
s~putAll(src)
say s~items
say s~allIndexes~sort~makeString('L', ',')
say s~allItems~sort~makeString('L', ',')
::class store inherit mapcollection
::method init
  expose d
  d = .directory~new
::method put
  expose d
  use arg item, index
  d[index] = item
::method allindexes
  expose d
  return d~allindexes
::method allitems
  expose d
  return d~allitems
```

```output
3
k1,k2,k3
first,second,third
```
