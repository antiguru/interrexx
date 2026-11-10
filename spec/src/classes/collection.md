---
element: "Collection"
kind: class
rexxref: "Builtin Classes > Collection Classes > Collection Class; Builtin Classes > Collection Classes > Collection Class > [] (Abstract Method); Builtin Classes > Collection Classes > Collection Class > []= (Abstract Method); Builtin Classes > Collection Classes > Collection Class > allIndexes (Abstract Method); Builtin Classes > Collection Classes > Collection Class > allItems (Abstract Method); Builtin Classes > Collection Classes > Collection Class > at (Abstract Method); Builtin Classes > Collection Classes > Collection Class > difference; Builtin Classes > Collection Classes > Collection Class > disjoint; Builtin Classes > Collection Classes > Collection Class > equivalent; Builtin Classes > Collection Classes > Collection Class > hasIndex; Builtin Classes > Collection Classes > Collection Class > hasItem; Builtin Classes > Collection Classes > Collection Class > index (Abstract Method); Builtin Classes > Collection Classes > Collection Class > intersection; Builtin Classes > Collection Classes > Collection Class > items; Builtin Classes > Collection Classes > Collection Class > makeArray; Builtin Classes > Collection Classes > Collection Class > put (Abstract Method); Builtin Classes > Collection Classes > Collection Class > subset; Builtin Classes > Collection Classes > Collection Class > supplier; Builtin Classes > Collection Classes > Collection Class > union; Builtin Classes > Collection Classes > Collection Class > xor"
deviations: [78, 109, 118]
---

# Collection

`Collection` is the mixin class that all the collection classes have in
common. It defines nineteen methods. Seven of them are abstract: they name
what a collection has to be able to do, reading and writing an entry by its
index and listing its indexes and items, and a class that inherits
`Collection` declares them. The other twelve are written in terms of those
seven and of the messages a collection answers for itself. They are the
questions about an index, an item, the count and a supplier, and the set
operations that combine two collections. A class that inherits only
`Collection` gets all twelve working as the rules below say; a concrete class
of the library may declare a method of the same name instead, and its page
says what that method does.

The page specifies the methods in that order. Where a method sends a message
that a program can override, its `.sends` rule gives the message, the
receiver and the order, as [the sends rule](r:class.index.sends) asks. The
argument of a set operation is read by a rule of its own, shared by the seven
operations. [`OrderedCollection`](orderedcollection.md) declares its own
versions of five of the set operations, and
[`MapCollection`](mapcollection.md) its own `makeArray`; the pages of
the concrete collections link to this page for what they do not declare.

The examples use the class `box`, a collection of strings in which the item
and the index of an entry are the same string. It keeps its entries in a
directory, answers the indexes and the items in sorted order, and declares
`put`, `remove` and `copy` for the set operations; the rest it gets from
`Collection`.

## The class

r[class.collection.creation]
`Collection` is a mixin class whose superclass is `Object`, and
`queryMixinClass` answers 1 for it ([the mixin rule](r:class.index.mixins)).
`new` is not refused: it makes an object of the class that received it, as
[the new rule](r:class.index.new) says, and that object understands the
nineteen methods of the class and no others beyond those of `Object`. A class
that inherits `Collection`, or a subclass of such a class, understands them
all and gets each from this page unless it or a class between it and
`Collection` declares a method of the same name.

```rexx
c = .collection~new
say c~class~id .collection~queryMixinClass .collection~superclass~id
names = 'allIndexes allItems at difference disjoint equivalent hasIndex',
    'hasItem index intersection items makeArray put subset supplier union',
    'xor [] []='
n = 0
do m over names~makearray(' ')
  if c~hasMethod(m) then n += 1
end
say n c~hasMethod('OF') c~hasMethod('APPEND') c~hasMethod('PUTALL')
say .box~new~hasMethod('UNION') .box~new~hasMethod('FIRST')
::class box inherit collection
```

```output
Collection 1 Object
19 0 0 0
1 0
```

## The abstract methods

r[class.collection.abstract]
The methods `[]`, `[]=`, `at`, `put`, `allIndexes`, `allItems` and `index` are
abstract. Sent to an object whose class has none of its own, each raises
`SYNTAX 93.965` whatever arguments the message has, none or too many
included, and the raising comes before any check of the arguments, so that
`at` with no argument raises 93.965 and not a missing-argument error
([the abstract rule](r:dir.method.abstract)). The assignment `c[i] = v` is
the message `[]=`. A class that declares one of them answers the message
itself. The methods that this page defines build on these: `allIndexes` and
`allItems` are what `hasIndex`, `hasItem`, `items`, `makeArray` and
`supplier` use, and `put` is what the set operations use to store.

```rexx
c = .collection~new
call try 'c~at'
call try 'c~at(1, 2, 3)'
call try 'c~put(1, 2)'
call try 'c~allIndexes'
call try 'c~allItems(7)'
call try 'c~index("x")'
call try 'c[1]'
call try 'c[1] = "x"'
say .part~new~at(5)
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class part inherit collection
::method at
  return 'at' arg(1)
```

```output
c~at => 93.965
c~at(1, 2, 3) => 93.965
c~put(1, 2) => 93.965
c~allIndexes => 93.965
c~allItems(7) => 93.965
c~index("x") => 93.965
c[1] => 93.965
c[1] = "x" => 93.965
at 5
```

## Questions about the contents

r[class.collection.hasindex]
`hasIndex(index)` answers what the answer of `allIndexes` answers to
`hasItem` for the same argument, unchanged. It takes exactly one argument:
none raises `SYNTAX 93.901` and a second raises `SYNTAX 93.902`, in both
cases before `allIndexes` is sent, so that a collection that leaves
`allIndexes` abstract raises 93.965 only for a call with one argument. A
`.nil` index is a valid argument.

```rexx
c = .collection~new
call try 'c~hasIndex'
call try 'c~hasIndex(1, 2)'
call try 'c~hasIndex(1)'
b = .box~new(.array~of('x', 'y'))
say b~hasIndex('x') b~hasIndex('z') b~hasIndex(.nil)
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class box inherit collection
::method init
  expose d
  use arg list
  d = .directory~new
  do i over list; d[i] = i; end
::method allindexes
  expose d
  return d~allindexes~sort
```

```output
c~hasIndex => 93.901
c~hasIndex(1, 2) => 93.902
c~hasIndex(1) => 93.965
1 0 0
```

r[class.collection.hasindex.sends]
`hasIndex` sends `allIndexes` to the receiver, with no argument, and then
sends `hasItem` with the index to the object that `allIndexes` answered, and
answers what that answers. That object need not be an array: any object that
understands `hasItem` does. Nothing else is sent, to the receiver or to the
answer.

```rexx
c = .spy~new
say c~hasIndex('k')
::class spy inherit collection
::method allindexes
  say 'allIndexes' arg()
  return .answer~new
::class answer
::method hasitem
  say 'hasItem' arg(1)
  return 'any object'
::method isa
  say 'isA'
  return 0
::method items
  say 'items'
  return 0
::method makearray
  say 'makeArray'
  return .array~new
::method '[]'
  say '[]'
  return 0
```

```output
allIndexes 0
hasItem k
any object
```

r[class.collection.hasitem]
`hasItem(item)` answers what the answer of `allItems` answers to `hasItem` for
the same argument, unchanged, and takes exactly one argument, as `hasIndex`
does: none raises `SYNTAX 93.901` and a second raises `SYNTAX 93.902`, in
both cases before `allItems` is sent. Whether two items count as the same is
decided by the object `allItems` answers: for an array that is the strict
equality of `==`, under which `1` and `1.0` are different items (for the very
object that is stored, see [the Array deviation
rule](r:class.array.deviation.78)).

```rexx
b = .box~new(.array~of('1', 'Ab', 'x'))
say b~hasItem(1) b~hasItem(1.0) b~hasItem('ab') b~hasItem('Ab')
signal on syntax name bad
say b~hasItem
bad: say condition('o')~code
::class box inherit collection
::method init
  expose items
  use arg items
::method allitems
  expose items
  return items
```

```output
1 0 0 1
93.901
```

r[class.collection.hasitem.sends]
`hasItem` sends `allItems` to the receiver, with no argument, and then sends
`hasItem` with the item to the object that `allItems` answered, and answers
what that answers. Nothing else is sent, to the receiver or to the answer.

```rexx
c = .spy~new
say c~hasItem('v')
::class spy inherit collection
::method allitems
  say 'allItems' arg()
  return .answer~new
::class answer
::method hasitem
  say 'hasItem' arg(1)
  return 'any object'
::method isa
  say 'isA'
  return 0
::method items
  say 'items'
  return 0
::method makearray
  say 'makeArray'
  return .array~new
::method '[]'
  say '[]'
  return 0
```

```output
allItems 0
hasItem v
any object
```

r[class.collection.items]
`items` answers what the answer of `allIndexes` answers to `items`,
unchanged. It does not check its arguments: any arguments are ignored, so
that `items(1, 2)` answers like `items`.

```rexx
b = .box~new(.array~of('p', 'q', 'r'))
say b~items b~items(1, 2)
::class box inherit collection
::method init
  expose list
  use arg list
::method allindexes
  expose list
  return list
```

```output
3 3
```

r[class.collection.items.sends]
`items` sends `allIndexes` to the receiver, with no argument, and then sends
`items` to the object that `allIndexes` answered, and answers what that
answers. Nothing else is sent; in particular `allItems` is not, and the answer
is sent nothing but `items`.

```rexx
c = .spy~new
say c~items
::class spy inherit collection
::method allindexes
  say 'allIndexes' arg()
  return .answer~new
::method allitems
  say 'allItems'
  return .array~new
::class answer
::method items
  say 'items' arg()
  return 'a count'
::method isa
  say 'isA'
  return 0
::method makearray
  say 'makeArray'
  return .array~new
::method hasitem
  say 'hasItem'
  return 0
```

```output
allIndexes 0
items 0
a count
```

r[class.collection.makearray]
`makeArray` answers what `allItems` answers, and the very object: it is not
copied, and what it is is not checked. Arguments are ignored. A collection
that wants `makeArray` to answer something other than its items declares it;
the [`MapCollection`](mapcollection.md) mixin answers the indexes instead.

```rexx
mine = .array~of('u', 'v')
b = .box~new(mine)
r = b~makeArray
say r~class~id r~items (r == mine)
say b~makeArray(1, 2, 3)~items
::class box inherit collection
::method init
  expose list
  use arg list
::method allitems
  expose list
  return list
```

```output
Array 2 1
2
```

r[class.collection.makearray.sends]
`makeArray` sends `allItems` to the receiver, with no argument, and answers
what it answers. Nothing else is sent, to the receiver or to the answer.

```rexx
c = .spy~new
say c~makeArray~class~id
::class spy inherit collection
::method allitems
  say 'allItems' arg()
  return .answer~new
::method allindexes
  say 'allIndexes'
  return .array~new
::class answer
::method isa
  say 'isA'
  return 0
::method makearray
  say 'makeArray'
  return .array~new
::method items
  say 'items'
  return 0
::method '[]'
  say '[]'
  return 0
```

```output
allItems 0
ANSWER
```

r[class.collection.supplier]
`supplier` answers a `Supplier` whose items are the objects that `allItems`
answered and whose indexes are the objects that `allIndexes` answered, the
entry at the same position of the two being a pair. It takes its arguments as
`items` does, that is, ignores them. The two answers are taken once, when
`supplier` is sent: the supplier is not a view on the collection. The
`Supplier` class has its own page.

```rexx
b = .box~new
s = b~supplier
b~put('late', 'k3')
do while s~available
  say s~index '=>' s~item
  s~next
end
say b~supplier(1)~class~id
::class box inherit collection
::method init
  expose d
  d = .directory~new
  d['k1'] = 'one'
  d['k2'] = 'two'
::method put
  expose d
  use arg item, index
  d[index] = item
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
```

```output
k1 => one
k2 => two
Supplier
```

r[class.collection.supplier.sends]
`supplier` sends `allItems` to the receiver, then `allIndexes`, both with no
argument, in that order, and makes the supplier from the two answers. The
supplier is made by `new` of the `Supplier` class, which sends `request` with
`ARRAY` once to the answer of `allItems` and then once to the answer of
`allIndexes` ([the conversion rule](r:class.supplier.init.convert)); the
`request` of `Object` sends `makeArray`. Nothing else is sent.

```rexx
c = .spy~new
s = c~supplier
say s~index s~item
::class spy inherit collection
::method allitems
  say 'allItems' arg()
  return .answer~new('items')
::method allindexes
  say 'allIndexes' arg()
  return .answer~new('indexes')
::class answer
::method init
  expose kind
  use arg kind
::method makearray
  expose kind
  say kind 'answer: makeArray'
  return .array~of('the' kind)
::method isa
  say 'isA'
  return 0
::method items
  say 'items'
  return 0
```

```output
allItems 0
allIndexes 0
items answer: makeArray
indexes answer: makeArray
the indexes the items
```

## The set operations

r[class.collection.operand]
The argument of `union`, `difference`, `intersection`, `xor`, `subset`,
`disjoint` and `equivalent` is read as a list of entries, and what is sent to
it depends on what it is, which the object itself decides by its answers to
`isA` ([the isA rule](r:class.collection.operand.isa)). An ordered argument,
one for which `isA` says it is an [`OrderedCollection`](orderedcollection.md),
is sent `allItems`. Any other object for which `isA` says it is a
[`MapCollection`](mapcollection.md) is sent `supplier` by `union`,
`intersection`, `xor` and `disjoint`, and `allIndexes` by `difference` and
`subset`. Every other object is sent `makeArray`. An object that is both
counts as ordered. An element of an ordered or plain list is an index and an
item at once; an entry of a map is its own index with its own item. `disjoint`
reads its argument through `intersection`, and `equivalent` through `subset`.
What is done with the answer is the subject of [the list
rule](r:class.collection.operand.list), [the map
rule](r:class.collection.operand.map) and [the answer
rule](r:class.collection.operand.answer).

```rexx
a = .box~new('a b')
do k over .array~of(.ordered~new, .mapped~new, .plain~new, .both~new)
  say '==' k~class~id
  r = a~union(k)
  r = a~difference(k)
  r = a~subset(k)
end
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  return self~class~new
::method allitems
  return .array~new
::method allindexes
  return .array~new
::class ordered inherit orderedcollection
::method allitems
  say '  allItems'
  return .array~new
::method makearray
  say '  makeArray'
  return .array~new
::method items
  return 0
::class mapped inherit mapcollection
::method supplier
  say '  supplier'
  return .supplier~new(.array~new, .array~new)
::method allindexes
  say '  allIndexes'
  return .array~new
::method makearray
  say '  makeArray'
  return .array~new
::method items
  return 0
::class both inherit mapcollection orderedcollection
::method allitems
  say '  allItems'
  return .array~new
::method supplier
  say '  supplier'
  return .supplier~new(.array~new, .array~new)
::method items
  return 0
::class plain inherit collection
::method makearray
  say '  makeArray'
  return .array~new
::method items
  return 0
```

```output
== ORDERED
  allItems
  allItems
  allItems
== MAPPED
  supplier
  allIndexes
  allIndexes
== PLAIN
  makeArray
  makeArray
  makeArray
== BOTH
  allItems
  allItems
  allItems
```

r[class.collection.operand.isa]
Each of the seven operations asks the argument what it is by sending
[`isA`](r:class.object.isa) with the class `OrderedCollection` and, when the
answer is not 1, `isA` with the class `MapCollection`. An answer of 1 to the
first question ends the questions. The answers are used as logical values, so
what an object answers to `isA` is what decides, not the classes it inherits:
an object that inherits no mixin and answers 1 is read as an ordered argument,
and an instance of a class that inherits `OrderedCollection` and answers 0 to
both is read as a plain one. An answer that is neither 0 nor 1 raises
`SYNTAX 34.1`, and so does `.nil`. `equivalent` asks after its two `items`
messages, as part of the `subset` it sends, and `subset` asks once more, with
the class `Set`, as [its rule](r:class.collection.subset) says.

```rexx
a = .box~new('a')
say a~union(.liar~new)
say a~union(.shy~new)
signal on syntax name bad
say a~union(.odd~new)
bad: say condition('o')~code
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method allindexes
  expose d
  return d~allindexes~sort
::class liar
::method isa
  say '  isA' arg(1)~id
  return 1
::method allitems
  say '  allItems'
  return .array~of('from allItems')
::method makearray
  say '  makeArray'
  return .array~of('from makeArray')
::class shy inherit orderedcollection
::method isa
  say '  isA' arg(1)~id
  return 0
::method allitems
  say '  allItems'
  return .array~of('from allItems')
::method makearray
  say '  makeArray'
  return .array~of('from makeArray')
::class odd
::method isa
  return 2
```

```output
  isA OrderedCollection
  allItems
a from allItems
  isA OrderedCollection
  isA MapCollection
  makeArray
a from makeArray
34.1
```

r[class.collection.operand.list]
The answer of `allItems` or `makeArray` is read as two arrays, the items and the
indexes, and the operation takes the entries from position 1 of the items up to
its size, an entry being the item at the position and the index at the same
position, `.nil` when there is none. An answer that is an `Array` of that class
itself is both arrays: it is read position by position, so an element is its own
index, and a position that holds nothing is an entry whose item and index are
`.nil`. Any other answer, an instance of a subclass of `Array` included, is sent
`makeArray` twice, and the first answer is the items and the second the indexes;
for a `List`, a `Queue`, a `Set` or a string the two are the same. An instance
of a subclass of `List` is such an answer, and is sent `makeArray` like the
instance of any other class that is not `Array` itself ([Deviation
118](r:class.collection.deviation.118)). Messages that the operation sends to
the entries are those of its own rule.

```rexx
h = .array~new(3)
h[2] = 'two'
r = .nb~new~union(.pk~new(h))
say '--'
r = .nb~new~union(.pk~new(.twice~new(2, 2)))
say '--'
r = .nb~new~union(.pk~new(.twice~new(3, 2)))
say '--'
r = .nb~new~union(.pk~new(.sub~of('s1', 's2')))
say '--'
r = .nb~new~union(.pk~new(.twice~new(1, 3)))
::class nb inherit collection
::method copy
  return self
::method hasindex
  say 'hasIndex' arg(1)
  return 0
::method put
  say 'put' arg(1) arg(2)
::class pk inherit collection
::method init
  expose answer
  use arg answer
::method makearray
  expose answer
  return answer
::class twice
::method init
  expose n m k
  use arg n, m
  k = 0
::method makearray
  expose n m k
  k += 1
  a = .array~new
  if k = 1 then do i = 1 to n; a~append('item' i); end
  else do i = 1 to m; a~append('index' i); end
  return a
::class sub subclass array
::method makearray
  say 'makeArray of the subclass'
  return .array~of('<'self[1]'>')
```

```output
hasIndex The NIL object
put The NIL object The NIL object
hasIndex two
put two two
hasIndex The NIL object
put The NIL object The NIL object
--
hasIndex index 1
put item 1 index 1
hasIndex index 2
put item 2 index 2
--
hasIndex index 1
put item 1 index 1
hasIndex index 2
put item 2 index 2
hasIndex The NIL object
put item 3 The NIL object
--
makeArray of the subclass
makeArray of the subclass
hasIndex <s1>
put <s1> <s1>
--
hasIndex index 1
put item 1 index 1
```

r[class.collection.operand.list.subclass]
Each of the two answers of `makeArray` that [the list
rule](r:class.collection.operand.list) reads is used as it is when it is an
`Array` or an instance of a subclass of `Array`: it is read position by position
and is sent nothing, so a subclass's own `makeArray`, `[]` or `items` is not
used.

```rexx
r = .nb~new~union(.pk~new(.twoans~new))
::class nb inherit collection
::method copy
  return self
::method hasindex
  say 'hasIndex' arg(1)
  return 0
::method put
  say 'put' arg(1) arg(2)
::class pk inherit collection
::method init
  expose answer
  use arg answer
::method makearray
  expose answer
  return answer
::class twoans
::method makearray
  expose k
  if var('k') then return .lsub~of('i1', 'i2')
  k = 1
  return .lsub~of('c', 'q')
::class lsub subclass array
::method makearray
  say 'makeArray of the subclass'
  forward class (super)
::method '[]'
  say '[] of the subclass'
  forward class (super)
::method items
  say 'items of the subclass'
  forward class (super)
```

```output defect=#153
hasIndex i1
put c i1
hasIndex i2
put q i2
```

r[class.collection.operand.map]
A map argument of `union`, `intersection`, `xor` and `disjoint` is read from
the answer of `supplier`, and the operation does not need that to be a
`Supplier`: it sends `available`, and while the answer is true sends `index`,
then the operation's own message to the receiver or the copy with the index,
then `item` only when the operation has an item to store, then `next`, and
asks `available` again. An object that answers those four messages is read as
a supplier is. An answer that does not understand one of them raises
`SYNTAX 93.948` when the message is sent, so after the entries before it have
been handled.

```rexx
s = .sp~new(.array~of('v1', 'v2', 'v3'), .array~of('a', 'b', 'c'))
r = .nb~new~union(.mp~new(s))
::class nb inherit collection
::method copy
  return self
::method hasindex
  say 'hasIndex' arg(1)
  return arg(1) == 'b'
::method put
  say 'put' arg(1) arg(2)
::class mp inherit mapcollection
::method init
  expose answer
  use arg answer
::method supplier
  expose answer
  return answer
::class sp subclass supplier
::method available
  say '  available'
  forward class (super)
::method index
  say '  index'
  forward class (super)
::method item
  say '  item'
  forward class (super)
::method next
  say '  next'
  forward class (super)
```

```output
  available
  index
hasIndex a
  item
put v1 a
  next
  available
  index
hasIndex b
  next
  available
  index
hasIndex c
  item
put v3 c
  next
  available
```

r[class.collection.operand.answer]
The object that `makeArray`, `allItems` or `allIndexes` answered is not
required to be a collection, only to understand the messages the operation
sends it, and what it lacks is found out when the message is due. The four
operations that read a list send `makeArray` to an answer that is not an
`Array` of that class itself, and one that has no `makeArray` method raises
`SYNTAX 98.913`; an `UNKNOWN` method does not count as one. `difference` sends
`[]` to the answer and `subset` sends `hasItem`, and an answer that does not
understand one raises `SYNTAX 93.948`, an `UNKNOWN` method that answers the
message counting as understanding it. `difference` and `subset` also send
`items` to the argument itself, after `copy`, and what that answers is how
many positions `difference` takes: an answer that is not a number raises
`SYNTAX 41.1`, and `.nil` raises `SYNTAX 93.948`, and a position beyond the
end of the answer reads `.nil`.

```rexx
a = .box~new('a b')
call try 'union', .pk~new(.object~new), 'plain object'
call try 'union', .pk~new(.unk~new), 'UNKNOWN only'
call try 'union', .mp~new(.nil), 'supplier .nil'
call try 'union', .mp~new(.noindex~new), 'no index'
call try 'difference', .pk~new(.object~new), 'plain object'
call try 'difference', .pk~new(.unk~new), 'UNKNOWN only'
call try 'subset', .pk~new(.object~new), 'plain object'
call try 'subset', .pk~new(.unk~new), 'UNKNOWN only'
call try 'difference', .counted~new('abc'), 'items abc'
call try 'difference', .counted~new(.nil), 'items .nil'
call try 'difference', .counted~new(3), 'items 3'
exit
try:
  signal on syntax name bad
  r = .message~new(a, arg(1), 'I', arg(2))~send
  say arg(1) arg(3) '->' r~class~id
  return
bad: say arg(1) arg(3) '=>' condition('o')~code
  return
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes
::method makearray
  return .array~of('a')
::method items
  return 2
::class pk inherit collection
::method init
  expose answer
  use arg answer
::method makearray
  expose answer
  return answer
::method items
  return 1
::class mp inherit mapcollection
::method init
  expose answer
  use arg answer
::method supplier
  expose answer
  return answer
::class unk
::method unknown
  return 0
::class noindex
::method available
  return 1
::method item
  return 'v'
::method next
  nop
::class counted inherit collection
::method init
  expose n
  use arg n
::method makearray
  return .array~of('b', 'a')
::method items
  expose n
  return n
```

```output
union plain object => 98.913
union UNKNOWN only => 98.913
union supplier .nil => 93.948
union no index => 93.948
difference plain object => 93.948
difference UNKNOWN only -> BOX
subset plain object => 93.948
subset UNKNOWN only -> String
difference items abc => 41.1
difference items .nil => 93.948
difference items 3 -> BOX
```

r[class.collection.operand.logical]
`union`, `intersection` and `xor` use what `hasIndex` answers as a logical
value. `union` refuses an answer that is not a string, `.nil` included, with
`SYNTAX 93.948` whatever its string value, and a string that is neither 0 nor 1
with `SYNTAX 34.901`. `intersection` and `xor` read an answer that is not a
string by its string value, and an answer whose value is neither 0 nor 1, `.nil`
included, raises `SYNTAX 34.1` there, and so from `disjoint`, which reads its
argument through `intersection`. The answer of `isA` is used in the same way, as
[the isA rule](r:class.collection.operand.isa) says.

```rexx
do v over .array~of('maybe', .nil, .yes~new)
  do op over 'union intersection xor disjoint'~makearray(' ')
    call try op, v
  end
end
exit
try:
  signal on syntax name bad
  r = .message~new(.hb~new(arg(2)), arg(1), 'I', .array~of('p'))~send
  say arg(1) arg(2)~string '=> ok'
  return
bad: say arg(1) arg(2)~string '=>' condition('o')~code
  return
::class hb inherit collection
::method init
  expose v
  use arg v
::method copy
  return self
::method hasindex
  expose v
  return v
::method put
  nop
::method remove
  nop
::method isempty
  return 1
::class yes
::method makestring
  return 1
```

```output
union maybe => 34.901
intersection maybe => 34.1
xor maybe => 34.1
disjoint maybe => 34.1
union The NIL object => 93.948
intersection The NIL object => 34.1
xor The NIL object => 34.1
disjoint The NIL object => 34.1
union a YES => 93.948
intersection a YES => ok
xor a YES => ok
disjoint a YES => ok
```

r[class.collection.operand.error]
Each of the seven operations takes exactly one argument: none raises
`SYNTAX 93.901` and a second raises `SYNTAX 93.902`. An argument that `isA`
does not report as ordered or as a map and that has no `makeArray` method,
`.nil` and an object of a class with no such method included, raises
`SYNTAX 93.948` from `union`, `difference`, `intersection`, `xor`, `subset`
and `disjoint`. `equivalent` sends `items` to the argument first, and raises
`SYNTAX 97.1` for the same objects. A string is read by its `makeArray`, so
`union`, `intersection`, `xor` and `disjoint` read a string without line ends
as a list of one element. A string has no `items` method, which `difference`
and `subset` need after they have read the argument and `equivalent` needs
first: a string raises `SYNTAX 93.948` from the first two and `SYNTAX 97.1`
from `equivalent`.

```rexx
a = .box~new('a b')
ops = 'union difference intersection xor subset disjoint equivalent'
do op over ops~makearray(' ')
  call try 'a~'op'(.nil)'
  call try 'a~'op
end
call try 'a~union(a, a)'
call try 'a~difference("b c")'
call try 'a~subset("b")'
call try 'a~equivalent("b")'
say a~union('x y')~allitems~items
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  return self~class~new(self~string)
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
```

```output
a~union(.nil) => 93.948
a~union => 93.901
a~difference(.nil) => 93.948
a~difference => 93.901
a~intersection(.nil) => 93.948
a~intersection => 93.901
a~xor(.nil) => 93.948
a~xor => 93.901
a~subset(.nil) => 93.948
a~subset => 93.901
a~disjoint(.nil) => 93.948
a~disjoint => 93.901
a~equivalent(.nil) => 97.1
a~equivalent => 93.901
a~union(a, a) => 93.902
a~difference("b c") => 93.948
a~subset("b") => 93.948
a~equivalent("b") => 97.1
3
```

r[class.collection.union]
`union(other)` answers a new collection: the object that the receiver's `copy`
answers, with the entries of the argument added whose index it lacks. An
argument that is read as a list ([the operand
rule](r:class.collection.operand)) adds each element as an item stored at an
index equal to it; an argument that is a map adds each of its entries, with
its own item, at its own index. An entry whose index the copy has is not
added, and the receiver is not changed.

```rexx
a = .box~new('a b c')
b = .box~new('b c d')
u = a~union(b)
say u '|' a '|' (u == a)
d = .directory~new
d['b'] = 'B'
d['z'] = 'Z'
say a~union(d)
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
::method isempty
  expose d
  return d~items = 0
::method '[]'
  expose d
  return d[arg(1)]
```

```output
a b c d | a b c | 0
Z a b c
```

r[class.collection.union.sends]
`union` sends `copy` to the receiver, with no argument, and then reads the
argument: it sends `isA` with `OrderedCollection` and, when that does not
answer 1, `isA` with `MapCollection`, then `allItems`, `supplier` or
`makeArray` as [the operand rule](r:class.collection.operand) says, and reads
the answer by [the list rule](r:class.collection.operand.list) or [the map
rule](r:class.collection.operand.map). For each entry in the order the
argument gives them it sends `hasIndex` to the copy with the index, and, when
that answers 0, `put` to the copy with the item and then the index. The answer
of `hasIndex` is used as [a logical
value](r:class.collection.operand.logical). Nothing else is sent. The copy is
made before the argument is read, so an argument that cannot be read leaves
the `copy` already sent.

```rexx
a = .lbox~new('a b')
say 'start'
r = a~union(.tr~new)
say r
signal on syntax name bad
r = a~union(.nil)
bad: say condition('o')~code
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
::method isempty
  expose d
  return d~items = 0
::method '[]'
  expose d
  return d[arg(1)]
::class lbox subclass box
::method copy
  say 'copy'
  return self~class~new(self~string)
::method hasindex
  say 'hasIndex' arg(1)
  forward class (super)
::method put
  say 'put' arg(1) arg(2)
  forward class (super)
::method remove
  say 'remove' arg(1)
  forward class (super)
::class tr
::method isa
  say 'isA' arg(1)~id
  return 0
::method makearray
  say 'makeArray'
  return .array~of('b', 'x')
```

```output
start
copy
isA OrderedCollection
isA MapCollection
makeArray
hasIndex b
hasIndex x
put x x
a b x
copy
93.948
```

r[class.collection.difference]
`difference(other)` answers a new collection: the object that the receiver's
`copy` answers, from which the entry at each index the argument lists has
been removed. For an argument read as a list ([the operand
rule](r:class.collection.operand)) an element names the index to remove, and
for a map the indexes of its entries do. Indexes that the copy lacks are
skipped, and the receiver is not changed.

```rexx
a = .box~new('a b c')
say a~difference(.box~new('b c d')) '|' a
d = .directory~new
d['a'] = 'unrelated item'
say a~difference(d)
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
::method isempty
  expose d
  return d~items = 0
::method '[]'
  expose d
  return d[arg(1)]
```

```output
a | a b c
b c
```

r[class.collection.difference.sends]
`difference` first reads the argument: it sends `isA` with `OrderedCollection`
and, when that does not answer 1, `isA` with `MapCollection`, then `allItems`,
`allIndexes` or `makeArray` as [the operand rule](r:class.collection.operand)
says. It then sends `copy` to the receiver and `items` to the argument, with
no argument. It then does the following for each of the positions from 1 up to
the number that `items` answered: it sends `[]` with the position to the
answer that the first step gave, and `remove` to the copy with what `[]`
answered. A position beyond the end of the answer reads `.nil`, and the answer
of `remove` is not used. Nothing else is sent. `remove` is not defined by
`Collection`, so a copy that lacks it raises `SYNTAX 93.948` at the first
position, and an argument whose `items` answers 0 never sends it.

```rexx
a = .lbox~new('a b')
r = a~difference(.tr~new)
say r
r = .bare~new~difference(.tr~new('none'))
say 'no position'
signal on syntax name bad
say .bare~new~difference(.tr~new)
bad: say condition('o')~code
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
::method isempty
  expose d
  return d~items = 0
::method '[]'
  expose d
  return d[arg(1)]
::class lbox subclass box
::method copy
  say 'copy'
  return self~class~new(self~string)
::method hasindex
  say 'hasIndex' arg(1)
  forward class (super)
::method put
  say 'put' arg(1) arg(2)
  forward class (super)
::method remove
  say 'remove' arg(1)
  forward class (super)
::class bare inherit collection
::method copy
  return self
::method put
  nop
::class tr
::method init
  expose kind
  if arg(1, 'E') then kind = 'none'
  else kind = 'some'
::method isa
  say 'isA' arg(1)~id
  return 0
::method makearray
  say 'makeArray'
  return .ans~new
::method items
  expose kind
  say 'items'
  if kind = 'none' then return 0
  return 2
::class ans
::method '[]'
  say '[]' arg(1)
  return .array~of('b', 'x')[arg(1)]
```

```output
isA OrderedCollection
isA MapCollection
makeArray
copy
items
[] 1
remove b
[] 2
remove x
a
isA OrderedCollection
isA MapCollection
makeArray
items
no position
isA OrderedCollection
isA MapCollection
makeArray
items
[] 1
93.948
```

r[class.collection.intersection]
`intersection(other)` answers a new, empty collection that `new` of the
receiver's class makes, and stores into it, for each element of the argument
that the receiver has as an index, that element as item and as index; for a
map argument the argument's own item is stored at the entry's index. Nothing
is stored for an index the receiver lacks, and the receiver is not changed.

```rexx
a = .box~new('a b c')
say a~intersection(.box~new('b c d'))
d = .directory~new
d['b'] = 'item from the map'
d['q'] = 'never stored'
say a~intersection(d)
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
::method isempty
  expose d
  return d~items = 0
::method '[]'
  expose d
  return d[arg(1)]
```

```output
b c
item from the map
```

r[class.collection.intersection.sends]
`intersection` sends `new` to the class of the receiver, with no argument,
which makes the result. It then reads the argument: it sends `isA` with
`OrderedCollection` and, when that does not answer 1, `isA` with
`MapCollection`, then `allItems`, `supplier` or `makeArray` as [the operand
rule](r:class.collection.operand) says, and reads the answer by [the list
rule](r:class.collection.operand.list) or [the map
rule](r:class.collection.operand.map). For each entry in order it sends
`hasIndex` to the receiver, not to the result, with the index, and when that
answers 1 sends `put` to the result with the item and then the index. The
answer of `hasIndex` is used as [a logical
value](r:class.collection.operand.logical). No `copy` is sent.

```rexx
a = .nbox~new('a b')
say 'start'
r = a~intersection(.tr~new)
say r
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
::method isempty
  expose d
  return d~items = 0
::method '[]'
  expose d
  return d[arg(1)]
::class nbox subclass box
::method new class
  say 'new' arg()
  forward class (super)
::method hasindex
  say 'hasIndex' arg(1)
  forward class (super)
::method put
  say 'put' arg(1) arg(2)
  forward class (super)
::class tr
::method isa
  say 'isA' arg(1)~id
  return 0
::method makearray
  say 'makeArray'
  return .array~of('b', 'x')
```

```output
new 1
start
new 0
isA OrderedCollection
isA MapCollection
makeArray
hasIndex b
put b b
hasIndex x
b
```

r[class.collection.xor]
`xor(other)` answers a new collection: the object that the receiver's `copy`
answers, in which each entry of the argument toggles: an entry whose index the
copy has is removed, and one it lacks is added with its item. What remains is
the entries that exactly one of the two collections has. The receiver is not
changed.

```rexx
a = .box~new('a b c')
say a~xor(.box~new('b c d'))
say '['a~xor(a)']' a
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
::method isempty
  expose d
  return d~items = 0
::method '[]'
  expose d
  return d[arg(1)]
```

```output
a d
[] a b c
```

r[class.collection.xor.sends]
`xor` sends `copy` to the receiver, then reads the argument: it sends `isA`
with `OrderedCollection` and, when that does not answer 1, `isA` with
`MapCollection`, then `allItems`, `supplier` or `makeArray` as [the operand
rule](r:class.collection.operand) says, and reads the answer by [the list
rule](r:class.collection.operand.list) or [the map
rule](r:class.collection.operand.map). For each entry in order it sends
`hasIndex` to the copy with the index. When that answers 1 it sends `remove`
to the copy with the index, and otherwise `put` with the item and then the
index. The answer of `hasIndex` is used as [a logical
value](r:class.collection.operand.logical). An index that occurs twice in the
argument is toggled twice, and a copy that lacks `remove` raises
`SYNTAX 93.948` at the first index that is already present.

```rexx
a = .lbox~new('a b')
r = a~xor(.tr~new)
say r
signal on syntax name bad
r = .bare~new~xor(.tr~new)
bad: say condition('o')~code
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
::method isempty
  expose d
  return d~items = 0
::method '[]'
  expose d
  return d[arg(1)]
::class lbox subclass box
::method copy
  say 'copy'
  return self~class~new(self~string)
::method hasindex
  say 'hasIndex' arg(1)
  forward class (super)
::method put
  say 'put' arg(1) arg(2)
  forward class (super)
::method remove
  say 'remove' arg(1)
  forward class (super)
::class bare inherit collection
::method copy
  return self
::method hasindex
  return 1
::class tr
::method isa
  say 'isA' arg(1)~id
  return 0
::method makearray
  say 'makeArray'
  return .array~of('b', 'x', 'x')
```

```output
copy
isA OrderedCollection
isA MapCollection
makeArray
hasIndex b
remove b
hasIndex x
put x x
hasIndex x
remove x
a
isA OrderedCollection
isA MapCollection
makeArray
93.948
```

r[class.collection.subset]
`subset(other)` answers 1 when each item that the receiver's `makeArray` answers
is equal to an element that the argument supplies, and 0 otherwise. The argument
is read as [the operand rule](r:class.collection.operand) says, so for a plain
or ordered argument the elements are its items, and a receiver with no items is
a subset of every collection. An item and an element are equal as `==` compares
strings: `1` and `1.0` differ, and so do `a` and `A`. For an argument that is
read through `allIndexes`, the elements are its indexes, and an item of the
receiver that is one of them must also have a matching entry: the receiver's
entry at that index is sent `\=` with the argument's entry at that index, and an
answer of 1 makes the result 0, so a receiver whose entries do not override `\=`
compares them as `=` does, where `1` and `1.0` are equal and `a` and `A` are
not. When `isA` says that the argument is a `Set`, its entries are not compared,
because an index and its item are the same there. The answer of that `isA` is
used as a logical value: a string that is neither 0 nor 1 raises
`SYNTAX 34.901`, and an answer that is not a string, `.nil` included, raises
`SYNTAX 93.948` whatever its string value. The answer of `\=` is used as a
logical value: what is neither 0 nor 1 raises `SYNTAX 34.6`.

```rexx
a = .box~new('a b')
say a~subset(.box~new('a b c')) a~subset(.box~new('a')) a~subset(a)
say .box~new~subset(.box~new)
say .box~new('a')~subset(.array~of('A')) .box~new('1')~subset(.array~of(1.0))
x = .map~new('k', 1, 'm', 'a')
say x~subset(.map~new('k', 1.0, 'm', 'a', 'n', 5))
say x~subset(.map~new('k', 1, 'm', 'A'))
say x~subset(.map~new('k', 1))
say x~subset(.set~of('k', 'm'))
t = .table~new
t['k'] = 'k'
t['m'] = 'm'
say x~subset(t)
signal on syntax name bad
say x~subset(.setliar~new(2))
bad: say condition('o')~code
signal on syntax name bad2
say x~subset(.setliar~new(.nil))
bad2: say condition('o')~code
signal on syntax name bad3
say x~subset(.setliar~new(.yes~new))
bad3: say condition('o')~code
exit
::class yes
::method makestring
  return 1
::class setliar inherit mapcollection
::method init
  expose answer
  use arg answer
::method isa
  expose answer
  if arg(1) == .set then return answer
  forward class (super)
::method allindexes
  return .array~of('k')
::method items
  return 1
::class map inherit mapcollection
::method init
  expose d
  d = .directory~new
  do i = 1 to arg() by 2; d[arg(i)] = arg(i + 1); end
::method allindexes
  expose d
  return d~allindexes~sort
::method '[]'
  expose d
  return d[arg(1)]
::method copy
  return self
::method items
  expose d
  return d~items
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
::method isempty
  expose d
  return d~items = 0
::method '[]'
  expose d
  return d[arg(1)]
```

```output
1 0 1
1
0 0
1
0
0
1
0
34.901
93.948
93.948
```

r[class.collection.subset.sends]
`subset` first reads the argument: it sends `isA` with `OrderedCollection` and,
when that does not answer 1, `isA` with `MapCollection`, then `allItems`,
`allIndexes` or `makeArray` as [the operand rule](r:class.collection.operand)
says. For a map it then sends `isA` with `Set`. Then it sends `copy` to the
receiver, `items` to the argument and `makeArray` to the receiver, each with no
argument, and uses none of the first two answers. A `copy` that answers anything
and an `items` that answers anything do not matter, but an argument that does
not understand `items` raises `SYNTAX 93.948`. The answer of the receiver's
`makeArray` has to be an `Array` of that class itself: an instance of a subclass
of `Array` raises `SYNTAX 98.913`, and nothing is sent to it. `subset` reads its
positions from 1 up to its `last`, a position that holds nothing giving the item
`.nil`, and for each item, in order, `subset` sends `hasItem` with the item to
the answer of the first step, and answers 0 at the first item for which that
answers 0, sending nothing more. For a map that is not a `Set`, when `hasItem`
answers 1 it sends `[]` with the item to the receiver and then `[]` with the
item to the argument itself, and then `\=` to the receiver's entry with the
argument's entry as the argument. An answer of 1 ends the operation with 0. When
every item has passed the answer is 1.

```rexx
a = .lmap~new('me', 'k', .e~new('one'), 'm', .e~new('two'))
b = .lmap~new('arg', 'k', .e~new('one'), 'm', .e~new('two'), 'n', .e~new('x'))
say a~subset(b)
say a~subset(.lmap~new('arg', 'k', .e~new('one'), 'm', .e~new('2')))
::class lmap inherit mapcollection
::method init
  expose d name
  name = arg(1)
  d = .directory~new
  do i = 2 to arg() by 2; d[arg(i)] = arg(i + 1); end
::method allindexes
  expose d name
  say name'.allIndexes'
  return d~allindexes~sort
::method makearray
  expose name
  say name'.makeArray'
  forward class (.mapcollection)
::method copy
  expose name
  say name'.copy'
  return self
::method items
  expose d name
  say name'.items'
  return d~items
::method isa
  expose name
  say name'.isA' arg(1)~id
  forward class (super)
::method '[]'
  expose d name
  say name'.[]' arg(1)
  return d[arg(1)]
::class e
::method init
  expose s
  use arg s
::method '\='
  expose s
  say '  \=' s arg(1)~string
  return s \= arg(1)~string
::method string
  expose s
  return s
```

```output
arg.isA OrderedCollection
arg.isA MapCollection
arg.allIndexes
arg.isA Set
me.copy
arg.items
me.makeArray
me.allIndexes
me.[] k
arg.[] k
  \= one one
me.[] m
arg.[] m
  \= two two
1
arg.isA OrderedCollection
arg.isA MapCollection
arg.allIndexes
arg.isA Set
me.copy
arg.items
me.makeArray
me.allIndexes
me.[] k
arg.[] k
  \= one one
me.[] m
arg.[] m
  \= two 2
0
```

r[class.collection.subset.receiver]
The array that the receiver's `makeArray` answers is read by position, as [the
sends rule](r:class.collection.subset.sends) says: an empty position before the
last item is the item `.nil`, and `hasItem` is sent with it; an instance of a
subclass of `Array` is refused with `SYNTAX 98.913` before anything is sent to
it.

```rexx
say .hole~new~subset(.pa~new)
signal on syntax name bad
say .sub~new~subset(.pa~new)
bad: say condition('o')~code
::class hole inherit collection
::method copy
  return self
::method makearray
  h = .array~new(4)
  h[1] = 'a'
  h[3] = 'b'
  return h
::class sub subclass hole
::method makearray
  return .larr~of('a')
::class larr subclass array
::method makearray
  say 'makeArray of the subclass'
  forward class (super)
::class pa
::method isa
  return 0
::method items
  return 0
::method makearray
  return .ans~new
::class ans
::method hasitem
  say 'hasItem' arg(1)~string
  return 1
```

```output defect=#154
hasItem a
hasItem The NIL object
hasItem b
1
98.913
```

r[class.collection.disjoint]
`disjoint(other)` answers 1 when the receiver and the argument have no entry
in common and 0 when they have one. It is the answer of `isEmpty` for what
`intersection` answers, so what the two have in common is what
[`intersection`](r:class.collection.intersection) selects. `isEmpty` is not
defined by `Collection`: a result that does not understand it raises
`SYNTAX 97.1`.

```rexx
a = .box~new('a b')
say a~disjoint(.box~new('x y')) a~disjoint(.box~new('b y'))
say a~disjoint(.array~new)
r = .noempty~new
signal on syntax name bad
say r~disjoint(.array~of('q'))
bad: say condition('o')~code
::class noempty inherit collection
::method hasindex
  return 1
::method put
  nop
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
::method isempty
  expose d
  return d~items = 0
::method '[]'
  expose d
  return d[arg(1)]
```

```output
1 0
1
97.1
```

r[class.collection.disjoint.sends]
`disjoint` sends `intersection` to the receiver with the argument, and then
`isEmpty` to the object `intersection` answered, with no argument. It sends
nothing else itself: the `new`, `isA`, `hasIndex` and `put` messages are those
of [the intersection rule](r:class.collection.intersection.sends), and an
answer of `isEmpty` is returned as it is.

```rexx
a = .lbox2~new('a b')
say a~disjoint(.array~of('b'))
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
::method isempty
  expose d
  return d~items = 0
::method '[]'
  expose d
  return d[arg(1)]
::class lbox subclass box
::method copy
  say 'copy'
  return self~class~new(self~string)
::method hasindex
  say 'hasIndex' arg(1)
  forward class (super)
::method put
  say 'put' arg(1) arg(2)
  forward class (super)
::method remove
  say 'remove' arg(1)
  forward class (super)
::class lbox2 subclass lbox
::method intersection
  say 'intersection' arg(1)~class~id
  forward class (super)
::method isempty
  say 'isEmpty'
  forward class (super)
```

```output
intersection Array
hasIndex b
put b b
isEmpty
0
```

r[class.collection.equivalent]
`equivalent(other)` answers 1 when the receiver and the argument have the same
number of items and the receiver is a [subset](r:class.collection.subset) of
the argument, so that two collections that have the same entries are
equivalent, and 0 otherwise. The two counts are compared by `=`, so `3` and
`3.0` are the same count. A receiver and an argument with different counts
are not equivalent whatever they hold.

```rexx
a = .box~new('a b c')
say a~equivalent(.box~new('c b a')) a~equivalent(.box~new('a b d'))
say a~equivalent(.box~new('a b')) a~equivalent(.array~of('a', 'b', 'c'))
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
::method isempty
  expose d
  return d~items = 0
::method '[]'
  expose d
  return d[arg(1)]
```

```output
1 0
0 1
```

r[class.collection.equivalent.sends]
`equivalent` sends `items` to the receiver and then `items` to the argument,
each with no argument. It then sends `=` to the answer of the receiver's
`items`, with the answer of the argument's as the argument, and uses what that
answers as a logical value. When it is 0 `equivalent` answers 0 and sends
nothing more. Otherwise it sends `subset` to the receiver with the argument,
which sends what [its rule](r:class.collection.subset.sends) says, and answers
what that answers, which is a logical value as well. An answer of `=` or of
`subset` that is neither 0 nor 1 raises `SYNTAX 34.6`.

```rexx
a = .lbox2~new('a b')
say a~equivalent(.other~new(3))
say a~equivalent(.other~new(2))
call try .odd~new(2, 'maybe')
call try .odd~new('maybe', 1)
exit
try:
  signal on syntax name bad
  say arg(1)~equivalent(.other~new(2))
  return
bad: say condition('o')~code
  return
::class box inherit collection
::method init
  expose d
  d = .directory~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); d[w] = w; end
::method copy
  expose d
  c = self~class~new
  do i over d~allindexes; c~put(d[i], i); end
  return c
::method string
  expose d
  return d~allitems~sort~makestring('L', ' ')
::method put
  expose d
  d[arg(2)] = arg(1)
::method remove
  expose d
  if arg(1) == .nil then return .nil
  return d~remove(arg(1))
::method allindexes
  expose d
  return d~allindexes~sort
::method allitems
  expose d
  return d~allitems~sort
::method isempty
  expose d
  return d~items = 0
::method '[]'
  expose d
  return d[arg(1)]
::class lbox subclass box
::method copy
  say 'copy'
  return self~class~new(self~string)
::method hasindex
  say 'hasIndex' arg(1)
  forward class (super)
::method put
  say 'put' arg(1) arg(2)
  forward class (super)
::method remove
  say 'remove' arg(1)
  forward class (super)
::class lbox2 subclass lbox
::method items
  say 'items'
  forward class (super)
::method subset
  say 'subset'
  forward class (super)
::class other inherit collection
::method init
  expose n
  use arg n
::method items
  expose n
  say 'other items'
  return n
::method makearray
  return .array~of('a', 'b')
::class odd inherit collection
::method init
  expose count answer
  use arg count, answer
::method items
  expose count
  return .count~new(count)
::method subset
  expose answer
  return answer
::class count
::method init
  expose n
  use arg n
::method '='
  expose n
  say 'count =' arg(1)
  if n == 'maybe' then return n
  return n = arg(1)
```

```output
items
other items
0
items
other items
subset
copy
other items
1
other items
count = 2
34.6
other items
count = 2
34.6
```

## Deviations

r[class.collection.deviation.78]
`hasItem` forwards the argument to the answer of `allItems`, so a search for
the very object that is stored in that answer follows [the Array
rule](r:class.array.deviation.78): the object's `==` is sent as for any other
item, and an object whose `==` answers false is not found. ooRexx finds the
very object without sending it `==`. See [Deviation 78](../deviations.md).

```rexx
x = .liar~new
b = .box~new(.array~of('a', x, 'b'))
.local~n = 0
say b~hasItem(x) .local~n
::class box inherit collection
::method init
  expose a
  use arg a
::method allItems
  expose a
  return a
::class liar
::method '=='
  .local~n += 1
  return 0
```

```output
0 3
```

r[class.collection.deviation.109]
An instance of a subclass of `String` that is compared by `subset` or
`equivalent` is read by its own characters, and `REQUEST` and `MAKESTRING` are
not sent to it ([Deviation 109](r:class.string.deviation.109)). That reaches the
entry of the argument that `subset` gives to `\=` and the count of the argument
that `equivalent` gives to `=`. ooRexx sends such an instance `request` with
`STRING` or `MAKESTRING`, so an override of either can change what is read. In
the example `MAKESTRING` answers other characters.

```rexx
n = .name~new('9')
say .cnt~new('3')~equivalent(.cnt~new(n))
x = .map~new('k', 'a')
say x~subset(.map~new('k', .name~new('b')))
::class name subclass string
::method makestring
  if self == '9' then return '3'
  return 'a'
::class cnt inherit collection
::method init
  expose c
  use arg c
::method items
  expose c
  return c
::method subset
  return 1
::class map inherit mapcollection
::method init
  expose d
  d = .directory~new
  do i = 1 to arg() by 2; d[arg(i)] = arg(i + 1); end
::method allindexes
  expose d
  return d~allindexes~sort
::method '[]'
  expose d
  return d[arg(1)]
::method copy
  return self
::method items
  expose d
  return d~items
```

```output defect=#140
0
0
```

r[class.collection.deviation.118]
An instance of a subclass of `List` that `makeArray` answers to `union`,
`intersection`, `xor` or `disjoint` is sent `makeArray`, twice, so that the
`makeArray` of the subclass is used, as [the list
rule](r:class.collection.operand.list) says. ooRexx uses the items such an
instance holds and sends it no `makeArray`. In the example the subclass answers
other items than it holds.

```rexx
r = .nb~new~union(.pk~new(.llist~of('held')))
::class llist subclass list
::method makearray
  say 'makearray sent'
  return .array~of('answered')
::class nb inherit collection
::method copy
  return self
::method hasindex
  say 'hasIndex' arg(1)
  return 0
::method put
  say 'put' arg(1) arg(2)
::class pk inherit collection
::method init
  expose answer
  use arg answer
::method makearray
  expose answer
  return answer
```

```output
makearray sent
makearray sent
hasIndex answered
put answered answered
```
