---
element: "Collection"
kind: class
rexxref: "Builtin Classes > Collection Classes > Collection Class; Builtin Classes > Collection Classes > Collection Class > [] (Abstract Method); Builtin Classes > Collection Classes > Collection Class > []= (Abstract Method); Builtin Classes > Collection Classes > Collection Class > allIndexes (Abstract Method); Builtin Classes > Collection Classes > Collection Class > allItems (Abstract Method); Builtin Classes > Collection Classes > Collection Class > at (Abstract Method); Builtin Classes > Collection Classes > Collection Class > difference; Builtin Classes > Collection Classes > Collection Class > disjoint; Builtin Classes > Collection Classes > Collection Class > equivalent; Builtin Classes > Collection Classes > Collection Class > hasIndex; Builtin Classes > Collection Classes > Collection Class > hasItem; Builtin Classes > Collection Classes > Collection Class > index (Abstract Method); Builtin Classes > Collection Classes > Collection Class > intersection; Builtin Classes > Collection Classes > Collection Class > items; Builtin Classes > Collection Classes > Collection Class > makeArray; Builtin Classes > Collection Classes > Collection Class > put (Abstract Method); Builtin Classes > Collection Classes > Collection Class > subset; Builtin Classes > Collection Classes > Collection Class > supplier; Builtin Classes > Collection Classes > Collection Class > union; Builtin Classes > Collection Classes > Collection Class > xor"
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
sends `hasItem` with the index to the object that `allIndexes` answered. That
object need not be an array: any object that understands `hasItem` does, and
what it answers is what `hasIndex` answers. Nothing else is sent.

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
equality of `==`, under which `1` and `1.0` are different items.

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
`hasItem` with the item to the object that `allItems` answered. Nothing else
is sent.

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
`items` sends `allIndexes` to the receiver, with no argument, and then
sends `items` to the object that `allIndexes` answered. Nothing else is sent;
in particular `allItems` is not.

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
```

```output
allIndexes 0
items 0
a count
```

r[class.collection.makearray]
`makeArray` answers what `allItems` answers, and the very object: it is not
copied, and what it is is not checked. Arguments are ignored. A collection that wants
`makeArray` to answer something other than its items declares it; the
[`MapCollection`](mapcollection.md) mixin answers the indexes instead.

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
`makeArray` sends `allItems` to the receiver, with no argument, and nothing
else.

```rexx
c = .spy~new
say c~makeArray
::class spy inherit collection
::method allitems
  say 'allItems' arg()
  return 'whatever allItems answers'
::method allindexes
  say 'allIndexes'
  return .array~new
```

```output
allItems 0
whatever allItems answers
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
argument, in that order, and makes the supplier from the two answers.
Nothing else is sent.

```rexx
c = .spy~new
s = c~supplier
say s~index s~item
::class spy inherit collection
::method allitems
  say 'allItems' arg()
  return .array~of('the item')
::method allindexes
  say 'allIndexes' arg()
  return .array~of('the index')
```

```output
allItems 0
allIndexes 0
the index the item
```

## The set operations

r[class.collection.operand]
The argument of `union`, `difference`, `intersection`, `xor`, `subset`,
`disjoint` and `equivalent` is read as a list of entries, and what is sent to
it depends on what it is. An instance of a class that inherits
[`OrderedCollection`](orderedcollection.md) is sent `allItems`. Any other
instance of a class that inherits [`MapCollection`](mapcollection.md) is
sent `supplier` by `union`, `intersection`, `xor` and `disjoint`, and
`allIndexes` by `difference` and `subset`. Every other object is sent
`makeArray`. An object that inherits both mixins counts as ordered. An
element of an ordered or plain list is an index and an item at once; an entry
of a map is its own index with its own item. `disjoint` reads its argument
through `intersection`, and `equivalent` through `subset`.

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

r[class.collection.operand.error]
Each of the seven operations takes exactly one argument: none raises
`SYNTAX 93.901` and a second raises `SYNTAX 93.902`. An argument that is not
a collection and does not understand `makeArray`, `.nil` and an object of
a class with no such method included, raises `SYNTAX 93.948` from `union`,
`difference`, `intersection`, `xor`, `subset` and `disjoint`. `equivalent`
sends `items` to the argument first, and raises `SYNTAX 97.1` for the same
objects. A string is read by its `makeArray`, so a string without line ends is
a list of one element.

```rexx
a = .box~new('a b')
ops = 'union difference intersection xor subset disjoint equivalent'
do op over ops~makearray(' ')
  call try 'a~'op'(.nil)'
  call try 'a~'op
end
call try 'a~union(a, a)'
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
`union` sends `copy` to the receiver, with no argument. It then reads the
argument as [the operand rule](r:class.collection.operand) says, and for each
element or entry in the order the argument gives them sends `hasIndex` to the
copy with the index, and, when that answers 0, `put` to the copy with the item
and then the index. Nothing else is sent. The copy is made before the argument
is read, so an argument that cannot be read leaves the `copy` already sent.

```rexx
a = .lbox~new('a b')
say 'start'
r = a~union(.array~of('b', 'x'))
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
```

```output
start
copy
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
`difference` first reads the argument as [the operand
rule](r:class.collection.operand) says, then sends `copy` to the receiver, and
then sends `items` to the argument, with no argument. It then sends `remove`
to the copy once for each of the positions from 1 up to the number that
`items` answered, with the element or index of the list that the argument
gave at that position, or `.nil` for a position beyond its end. The answer of
`remove` is not used.

```rexx
a = .lbox~new('a b')
r = a~difference(.short~new)
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
::class short inherit collection
::method makearray
  return .array~of('b', 'x')
::method items
  say 'items'
  return 3
```

```output
copy
items
remove b
remove x
remove The NIL object
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
which makes the result; it then reads the argument as [the operand
rule](r:class.collection.operand) says, and for each element or entry sends
`hasIndex` to the receiver, not to the result, with the index, and when that
answers 1 sends `put` to the result with the item and then the index. No
`copy` is sent.

```rexx
a = .nbox~new('a b')
say 'start'
r = a~intersection(.array~of('b', 'x'))
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
::class nbox subclass lbox
::method new class
  say 'new' arg()
  forward class (super)
```

```output
new 1
start
new 0
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
`xor` sends `copy` to the receiver, then reads the argument as [the operand
rule](r:class.collection.operand) says, and for each element or entry in
order sends `hasIndex` to the copy with the index. When that answers 1 it
sends `remove` to the copy with the index, and otherwise `put` with the item
and then the index. An index that occurs twice in the argument is toggled
twice.

```rexx
a = .lbox~new('a b')
r = a~xor(.array~of('b', 'x'))
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
```

```output
copy
hasIndex b
remove b
hasIndex x
put x x
a x
```

r[class.collection.subset]
`subset(other)` answers 1 when each item that the receiver's `makeArray`
answers is equal to an element that the argument supplies, and 0 otherwise.
The argument is read as [the operand rule](r:class.collection.operand) says,
so for a plain or ordered argument the elements are its items, and a
receiver with no items is a subset of every collection. An item and an
element are equal as `==` compares strings: `1` and `1.0` differ, and so do
`a` and `A`. For an argument that is read through `allIndexes`, the elements
are its indexes, and an item of the receiver that is one of them must also
have an equal entry: the receiver's entry at that index and the argument's
are compared by `=`, so `1` and `1.0` are equal there and `a` and `A` are not.

```rexx
a = .box~new('a b')
say a~subset(.box~new('a b c')) a~subset(.box~new('a')) a~subset(a)
say .box~new~subset(.box~new)
say .box~new('a')~subset(.array~of('A')) .box~new('1')~subset(.array~of(1.0))
x = .map~new('k', 1, 'm', 'a')
say x~subset(.map~new('k', 1.0, 'm', 'a', 'n', 5))
say x~subset(.map~new('k', 1, 'm', 'A'))
say x~subset(.map~new('k', 1))
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
```

r[class.collection.subset.sends]
`subset` reads the argument as [the operand rule](r:class.collection.operand)
says, then sends `copy` to the receiver, then `items` to the argument, and
then `makeArray` to the receiver, each with no argument. The answers of `copy`
and `items` are not used. When the argument is read through `allIndexes`, `[]`
is then sent for each item of the receiver that is among the argument's
indexes, in the order of `makeArray`: first to the receiver and then to the
argument, with the item as the argument. The first item that is not an
element, or whose entries differ, ends the operation with 0 and sends nothing
more.

```rexx
a = .lmap~new('me', 'k', 1, 'm', 2)
say a~subset(.lmap~new('arg', 'k', 1, 'm', 2, 'n', 3))
say a~subset(.lmap~new('arg', 'k', 1, 'm', 9))
::class lmap inherit mapcollection
::method init
  expose d name
  name = arg(1)
  d = .directory~new
  do i = 2 to arg() by 2; d[arg(i)] = arg(i + 1); end
::method allindexes
  expose d
  return d~allindexes~sort
::method makearray
  say 'makeArray'
  forward class (.mapcollection)
::method copy
  say 'copy'
  return self
::method items
  expose d
  say 'items'
  return d~items
::method '[]'
  expose d name
  say name'[]' arg(1) '->' d[arg(1)]
  return d[arg(1)]
```

```output
copy
items
makeArray
me[] k -> 1
arg[] k -> 1
me[] m -> 2
arg[] m -> 2
1
copy
items
makeArray
me[] k -> 1
arg[] k -> 1
me[] m -> 2
arg[] m -> 9
0
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
nothing else itself: the `new`, `hasIndex` and `put` messages are those of
[the intersection rule](r:class.collection.intersection.sends).

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
each with no argument. When `=` finds the two answers different it answers 0
and sends nothing more. Otherwise it sends `subset` to the receiver with the
argument, and answers what that answers.

```rexx
a = .lbox2~new('a b')
say a~equivalent(.other~new(3))
say a~equivalent(.other~new(2))
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
```
