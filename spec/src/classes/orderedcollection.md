---
element: "OrderedCollection"
kind: class
rexxref: "Builtin Classes > Collection Classes > OrderedCollection Class; Builtin Classes > Collection Classes > OrderedCollection Class > append (Abstract Method); Builtin Classes > Collection Classes > OrderedCollection Class > appendAll; Builtin Classes > Collection Classes > OrderedCollection Class > delete (Abstract Method); Builtin Classes > Collection Classes > OrderedCollection Class > difference; Builtin Classes > Collection Classes > OrderedCollection Class > first (Abstract Method); Builtin Classes > Collection Classes > OrderedCollection Class > firstItem (Abstract Method); Builtin Classes > Collection Classes > OrderedCollection Class > insert (Abstract Method); Builtin Classes > Collection Classes > OrderedCollection Class > intersection; Builtin Classes > Collection Classes > OrderedCollection Class > last (Abstract Method); Builtin Classes > Collection Classes > OrderedCollection Class > lastItem (Abstract Method); Builtin Classes > Collection Classes > OrderedCollection Class > next (Abstract Method); Builtin Classes > Collection Classes > OrderedCollection Class > previous (Abstract Method); Builtin Classes > Collection Classes > OrderedCollection Class > section (Abstract Method); Builtin Classes > Collection Classes > OrderedCollection Class > sort; Builtin Classes > Collection Classes > OrderedCollection Class > sortWith; Builtin Classes > Collection Classes > OrderedCollection Class > stableSort; Builtin Classes > Collection Classes > OrderedCollection Class > stableSortWith; Builtin Classes > Collection Classes > OrderedCollection Class > subset; Builtin Classes > Collection Classes > OrderedCollection Class > union; Builtin Classes > Collection Classes > OrderedCollection Class > xor"
deviations: [109, 118]
---

# OrderedCollection

`OrderedCollection` is the mixin class of the collections whose entries have
an order of their own: the arrays, the lists, the queues and the circular
queues. It inherits [`Collection`](collection.md) and declares twenty methods
of its own. The ones that walk and change a sequence, `first`, `firstItem`,
`last`, `lastItem`, `next`, `previous`, `append`, `insert`, `delete` and
`section`, are abstract, and the four sorts are written in terms of other
messages. The rest are `appendAll` and
the set operations `union`, `difference`, `intersection`, `xor` and `subset`,
which replace the versions of [`Collection`](collection.md) for a class that
inherits this mixin. `disjoint` and `equivalent` are not replaced: they are
the methods of that page, and they use the set operations of this one through
the messages they send.

The set operations here treat the collection as a sequence that may repeat
an item: a repeated item counts as many times as it occurs. They are built on
messages that the receiver has to understand: `removeItem`, which no mixin
defines, `append`, which this mixin declares abstract, and `hasItem`, which
[`Collection`](collection.md) defines. The examples use the class `seq`,
which keeps its items in an array and declares `init`, `string`, `append`,
`removeItem`, `allItems`, `items`, `isEmpty` and `copy`; `hasItem` it gets
from `Collection`.

## The class

r[class.orderedcollection.creation]
`OrderedCollection` is a mixin class whose superclass is `Collection`, and
`queryMixinClass` answers 1 for it ([the mixin rule](r:class.index.mixins)).
`new` is not refused and makes an object of the class that received it, as
[the new rule](r:class.index.new) says. The instance methods that the class
declares are `append`, `appendAll`, `delete`, `difference`, `first`,
`firstItem`, `insert`, `intersection`, `last`, `lastItem`, `next`,
`previous`, `section`, `sort`, `sortWith`, `stableSort`, `stableSortWith`,
`subset`, `union` and `xor`; there is no class method. A class that inherits
the mixin gets these before the methods of `Collection` of the same name,
and gets `disjoint`, `equivalent` and the rest from
[`Collection`](collection.md).

```rexx
c = .orderedcollection~new
say c~class~id c~class~queryMixinClass c~class~superclass~id
names = 'append appendAll delete difference first firstItem insert',
  'intersection last lastItem next previous section sort sortWith',
  'stableSort stableSortWith subset union xor'
n = 0
do m over names~makearray(' ')
  if c~hasMethod(m) then n += 1
end
say n c~hasMethod('DISJOINT') c~hasMethod('HASINDEX') c~hasMethod('OF')
```

```output
OrderedCollection 1 Collection
20 1 1 0
```

## The abstract methods

r[class.orderedcollection.abstract]
The methods `append`, `delete`, `first`, `firstItem`, `insert`, `last`,
`lastItem`, `next`, `previous` and `section` are abstract. Sent to an object
whose class has none of its own, each raises `SYNTAX 93.965` whatever
arguments the message has, none or too many included
([the abstract rule](r:dir.method.abstract)). A class that declares one
answers the message itself.

```rexx
c = .orderedcollection~new
call try 'c~append(1)'
call try 'c~append'
call try 'c~append(1, 2, 3)'
call try 'c~delete(1)'
call try 'c~first'
call try 'c~firstItem(7)'
call try 'c~insert(1)'
call try 'c~insert(1, 2)'
call try 'c~last'
call try 'c~lastItem'
call try 'c~next(1)'
call try 'c~next'
call try 'c~previous(1)'
call try 'c~section(1)'
call try 'c~section(1, 2)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
c~append(1) => 93.965
c~append => 93.965
c~append(1, 2, 3) => 93.965
c~delete(1) => 93.965
c~first => 93.965
c~firstItem(7) => 93.965
c~insert(1) => 93.965
c~insert(1, 2) => 93.965
c~last => 93.965
c~lastItem => 93.965
c~next(1) => 93.965
c~next => 93.965
c~previous(1) => 93.965
c~section(1) => 93.965
c~section(1, 2) => 93.965
```

r[class.orderedcollection.sort]
The methods `sort`, `stableSort`, `sortWith` and `stableSortWith` sort any
receiver through other messages; a class that inherits the mixin and has no
sorts of its own, such as `List`, uses them. They check the number of their
arguments first: `sort` and `stableSort` take none, and an argument raises
`SYNTAX 93.902`; `sortWith` and `stableSortWith` take exactly one, none raises
`SYNTAX 93.901` and a second `SYNTAX 93.902`, and the argument is not checked
to be a comparator. `sort` sends `stableSort` to the receiver, and `sortWith`
sends `stableSortWith` with its argument. `stableSort` sends `makeArray` to
the receiver and `sort` to the answer; `stableSortWith` sends that answer
`sortWith` with the comparator. Then the receiver is sent `first`, the sorted
array `size`, and for each place from 1 to that size the array is sent `[]`
with the place, the receiver `[]=` with that item and the current index, and
then `next` with that index, which gives the index for the next item. The
receiver answers itself. A receiver whose `makeArray` reaches an abstract
method, an instance of the mixin itself among them, raises `SYNTAX 93.965`.

```rexx
c = .orderedcollection~new
call try 'c~sort'
call try 'c~sort(1)'
call try 'c~stableSort'
call try 'c~stableSort(1)'
call try 'c~sortWith'
call try 'c~sortWith(.numericComparator~new)'
call try 'c~sortWith(.nil)'
call try 'c~sortWith(1, 2)'
call try 'c~stableSortWith'
call try 'c~stableSortWith("not a comparator")'
call try 'c~stableSortWith(1, 2)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
c~sort => 93.965
c~sort(1) => 93.902
c~stableSort => 93.965
c~stableSort(1) => 93.902
c~sortWith => 93.901
c~sortWith(.numericComparator~new) => 93.965
c~sortWith(.nil) => 93.965
c~sortWith(1, 2) => 93.902
c~stableSortWith => 93.901
c~stableSortWith("not a comparator") => 93.965
c~stableSortWith(1, 2) => 93.902
```

```rexx
c = .oc~new
say 'sort' c~sort~class~id
say c~string
call try 'c~sort(1)'
call try '.oc2~new~stableSortWith'
call try '.oc2~new~stableSort'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class oc inherit orderedcollection
::method init
  expose a
  a = .array~of('c', 'a', 'b')
::method makearray
  expose a
  say 'makeArray'
  return a~copy
::method first
  say 'first'
  return 1
::method next
  use arg i
  say 'next' i
  if i < 3 then return i + 1
  return .nil
::method '[]='
  expose a
  use arg v, i
  say '[]=' v i
  a[i] = v
::method string
  expose a
  return a~makestring('L', ',')
::class oc2 inherit orderedcollection
```

```output
makeArray
first
[]= a 1
next 1
[]= b 2
next 2
[]= c 3
next 3
sort OC
a,b,c
c~sort(1) => 93.902
.oc2~new~stableSortWith => 93.901
.oc2~new~stableSort => 93.965
```

## Adding

r[class.orderedcollection.appendall]
`appendAll(source)` adds every item of `source` to the end of the receiver,
in the order the source gives them, and answers the receiver. The items are
those that the source answers to `allItems`, an array or any collection. The
source need not be a collection: any object that understands `allItems` is
accepted, and an answer that is a string counts as one item. The answer of
`allItems` is taken before the first `append`, so a collection can append its
own items. The method takes exactly one argument: none raises `SYNTAX 93.901`
and a second raises `SYNTAX 93.902`. A `source` that does not understand
`allItems` raises `SYNTAX 93.948`, and an answer of `.nil` raises
`SYNTAX 98.913`; both are raised before anything is appended.

```rexx
s = .seq~new('a')
r = s~appendAll(.list~of('b', 'c'))
say (r == s) '|' s
r = s~appendAll(s)
say s
r = s~appendAll(.array~new)
say s
say s~appendAll(.words~new)
call try 's~appendAll'
call try 's~appendAll(1, 2)'
call try 's~appendAll("text")'
call try 's~appendAll(.nil)'
call try 's~appendAll(.nothing~new)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class words
::method allitems
  return 'one string'
::class nothing
::method allitems
  return .nil
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
```

```output
1 | a b c
a b c a b c
a b c a b c
a b c a b c one string
s~appendAll => 93.901
s~appendAll(1, 2) => 93.902
s~appendAll("text") => 93.948
s~appendAll(.nil) => 93.948
s~appendAll(.nothing~new) => 98.913
```

r[class.orderedcollection.appendall.sends]
`appendAll` sends `allItems` to `source`, with no argument, and then sends
`makeArray` once to the answer, as [`DO OVER`](r:instr.do.over) does. It sends
`append` to the receiver once for each item present in the array that came
back, in order, and nothing else: `source` is not asked `isA`, and no `copy`
or `items` is sent. An answer that has no `makeArray` method raises
`SYNTAX 98.913`.

```rexx
s = .lseq~new('x')
say s~appendAll(.src~new)
::class src inherit collection
::method allitems
  say 'allItems' arg()
  return .la~of('p', 'q')
::method isa
  say 'isA'
  return 0
::method items
  say 'items'
  return 0
::method makearray
  say 'makeArray of the source'
  return .array~new
::class la subclass array
::method makearray
  say 'makeArray of the answer'
  forward class (super)
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
::class lseq subclass seq
::method copy
  say 'copy'
  forward class (super)
::method append
  say 'append' arg(1)
  forward class (super)
::method removeitem
  say 'removeItem' arg(1)
  forward class (super)
::method hasitem
  say 'hasItem' arg(1)
  forward class (super)
::method items
  say 'items'
  forward class (super)
```

```output
allItems 0
makeArray of the answer
append p
append q
x p q
```

## Set operations

r[class.orderedcollection.operand]
The argument of `union`, `difference`, `intersection`, `xor` and `subset` is
read as a list of items, and so is that of `disjoint` and `equivalent` through
the methods that they send. The argument is asked `isA` with the class
`Collection` ([the isA rule](r:class.orderedcollection.operand.isa)). When
that answers 1, whichever of the mixins the object has, it is sent `allItems`,
with no argument. Any other object is sent `makeArray`, and what that answers
is sent `allItems`: an object that does not understand `makeArray`, `.nil`
included, raises `SYNTAX 93.948`, and so does an answer of `makeArray` that
does not understand `allItems`, a string included. What `allItems` answers is
read by [the answer rule](r:class.orderedcollection.operand.answer). Each of
the five operations takes exactly one argument: none raises `SYNTAX 93.901`
and a second raises `SYNTAX 93.902`. A string is read by its `makeArray`,
which answers an array of its lines, so a string without line ends is a list
of one item.

```rexx
a = .seq~new('a b')
say a~union(.array~of('x'))
say a~union(.mapped~new)
say a~union(.plain~new)
say a~union('one two')
say a~union(.viamake~new)
call try 'a~union(.nil)'
call try 'a~union(.object~new)'
call try 'a~union(.badmake~new)'
call try 'a~union'
call try 'a~xor(1, 2)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class mapped inherit mapcollection
::method allitems
  return .array~of('m')
::method supplier
  return .supplier~new(.array~of('wrong'), .array~of('index'))
::class plain inherit collection
::method allitems
  return .array~of('p')
::method makearray
  return .array~of('wrong')
::class viamake
::method makearray
  return .list~of('v1', 'v2')
::class badmake
::method makearray
  return 'a string'
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
```

```output
a b x
a b m
a b p
a b one two
a b v1 v2
a~union(.nil) => 93.948
a~union(.object~new) => 93.948
a~union(.badmake~new) => 93.948
a~union => 93.901
a~xor(1, 2) => 93.902
```

r[class.orderedcollection.operand.isa]
Each of the five operations asks the argument what it is by sending
[`isA`](r:class.object.isa) with the class `Collection`, and nothing else: an
answer of 1 makes the argument a list that is sent `allItems`, and an answer of
0 makes it an object that is sent `makeArray`. The answer decides, not the
classes the object inherits: an object that inherits no mixin and answers 1 is
read through `allItems`, and an instance of a class that inherits `Collection`
and answers 0 is read through `makeArray`. The answer is used as a logical
value: a string that is neither 0 nor 1 raises `SYNTAX 34.901`, and an answer
that is not a string, `.nil` included, raises `SYNTAX 93.948` whatever its
string value. `appendAll` does not send `isA`.

```rexx
a = .seq~new('a')
say a~union(.liar~new)
say a~union(.shy~new)
signal on syntax name bad
say a~union(.odd~new)
bad: say condition('o')~code
signal on syntax name bad2
say a~union(.vague~new)
bad2: say condition('o')~code
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
::class shy inherit collection
::method isa
  say '  isA' arg(1)~id
  return 0
::method allitems
  say '  allItems'
  return .array~of('from allItems')
::method makearray
  say '  makeArray'
  return .shy~new
::class odd
::method isa
  return 2
::class vague
::method isa
  return .yes~new
::class yes
::method makestring
  return 1
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
```

```output
  isA Collection
  allItems
a from allItems
  isA Collection
  makeArray
  allItems
a from allItems
34.901
93.948
```

r[class.orderedcollection.operand.answer]
The answer of `allItems` is read as [`DO OVER`](r:instr.do.over) reads a
collection: `makeArray` is sent to it once and the items present in the array
that comes back are the items of the list, in order, a position that holds
nothing being skipped. An answer that is an array, a list, any collection or a
string is read in this way, an instance of a subclass of `List` among them,
which is sent `makeArray` like any other ([Deviation
118](r:class.orderedcollection.deviation.118)); one that has no `makeArray`
method, `.nil` included, raises `SYNTAX 98.913`. Whether the argument was asked
`isA` or `makeArray` first makes no difference to this.

```rexx
h = .array~new(4)
h[2] = 'h2'
h[4] = 'h4'
say .seq~new('a')~union(.cA~new(h))
say .seq~new('a')~union(.cA~new('w1 w2'))
say .seq~new('a')~union(.cA~new(.list~of('l1', 'l2')))
say .seq~new('a')~union(.cA~new(.la~of('q')))
signal on syntax name bad
say .seq~new('a')~union(.cA~new(.nil))
bad: say condition('o')~code
::class cA inherit collection
::method init
  expose ans
  use arg ans
::method isa
  return 1
::method allitems
  expose ans
  return ans
::class la subclass array
::method makearray
  say 'makeArray of the answer'
  forward class (super)
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
```

```output
a h2 h4
a w1 w2
a l1 l2
makeArray of the answer
a q
98.913
```

r[class.orderedcollection.union]
`union(other)` answers a new collection: the object that the receiver's `copy`
answers, with every item of the argument appended to it, in the order the
argument gives them. An item that the receiver already has is appended again,
so the result has as many copies of an item as the receiver and the argument
have together. The receiver is not changed. When the last item has been
appended, the copy is sent `isA` with the class `Array`; when it answers 1 the
answer is what the copy answers to `makeArray`, sent with no argument, and
otherwise the answer is the copy.

```rexx
a = .seq~new('a b b')
u = a~union(.seq~new('b c'))
say u '|' a '|' (u == a)
say a~union(a)
say a~union(.array~new)
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
```

```output
a b b b c | a b b | 0
a b b a b b
a b b
```

r[class.orderedcollection.union.sends]
`union` sends `copy` to the receiver, with no argument. It then reads the
argument by [the operand rule](r:class.orderedcollection.operand): `isA` with
the class `Collection`, then `allItems` to the argument when that answered 1,
and otherwise `makeArray` to the argument and `allItems` to what that
answered. It sends `makeArray` once to the answer of `allItems`, as
[`DO OVER`](r:instr.do.over) does, and then sends `append` to the copy once
for each item present in the array that came back, in order. Then it sends
`isA` with `Array` to the copy, and `makeArray` to it when that answered 1;
nothing else is sent: no item is tested before it is appended.

```rexx
a = .lseq~new('a b')
r = a~union(.tr~new)
say r
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
::class lseq subclass seq
::method copy
  say 'copy'
  forward class (super)
::method append
  say 'append' arg(1)
  forward class (super)
::method removeitem
  say 'removeItem' arg(1)
  forward class (super)
::method hasitem
  say 'hasItem' arg(1)
  forward class (super)
::method items
  say 'items'
  forward class (super)
::class tr
::method isa
  say 'isA' arg(1)~id
  return 0
::method makearray
  say 'makeArray'
  return .ans~new
::class ans
::method allitems
  say 'allItems of the answer'
  return .la~of('b', 'x')
::class la subclass array
::method makearray
  say 'makeArray of the allItems answer'
  forward class (super)
```

```output
copy
isA Collection
makeArray
allItems of the answer
makeArray of the allItems answer
append b
append x
a b b x
```

```rexx
a = .isaseq~new('a b')
r = a~union(.array~of('b', 'x'))
say r~class~id r
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
::class isaseq subclass seq
::method isa
  say 'isA' arg(1)~id
  forward class (super) continue
  return 1
::method makearray
  say 'makeArray'
  return .array~of('MA')
::method append
  say 'append' arg(1)
  forward class (super)
::method removeitem
  say 'removeItem' arg(1)
  forward class (super)
```

```output
append b
append x
isA Array
makeArray
Array MA
```

r[class.orderedcollection.difference]
`difference(other)` answers a new collection: the object that the receiver's
`copy` answers, from which, for each item of the argument in order, one
occurrence of the item has been removed by `removeItem`. An item of the
argument that the copy no longer has removes nothing. The receiver is not
changed. `removeItem` is not declared by the mixin: a receiver without it
raises `SYNTAX 93.948` at the first removal. When the last item has been
removed, the copy is sent `isA` with the class `Array`; when it answers 1 the
answer is what the copy answers to `makeArray`, sent with no argument, and
otherwise the answer is the copy.

```rexx
a = .seq~new('a b b c')
say a~difference(.seq~new('b c c d')) '|' a
say a~difference(.array~new)
call try '.bare~new~difference(.array~of(1))'
call try '.bare~new~difference(.array~new)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class bare inherit orderedcollection
::method copy
  return self
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
```

```output
a b | a b b c
a b b c
.bare~new~difference(.array~of(1)) => 93.948
.bare~new~difference(.array~new) => ok
```

r[class.orderedcollection.difference.sends]
`difference` sends `copy` to the receiver, then reads the argument as [the
operand rule](r:class.orderedcollection.operand) says, and sends `makeArray`
once to the answer of `allItems`, as [`DO OVER`](r:instr.do.over) does. It
then sends `removeItem` to the copy with each item present in the array that
came back, in order, whether or not the copy has it. Then it sends `isA` with
`Array` to the copy, and `makeArray` to it when that answered 1; nothing else
is sent.

```rexx
a = .lseq~new('a b')
r = a~difference(.tr~new)
say r
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
::class lseq subclass seq
::method copy
  say 'copy'
  forward class (super)
::method append
  say 'append' arg(1)
  forward class (super)
::method removeitem
  say 'removeItem' arg(1)
  forward class (super)
::method hasitem
  say 'hasItem' arg(1)
  forward class (super)
::method items
  say 'items'
  forward class (super)
::class tr
::method isa
  say 'isA' arg(1)~id
  return 0
::method makearray
  say 'makeArray'
  return .ans~new
::class ans
::method allitems
  say 'allItems of the answer'
  return .la~of('b', 'x')
::class la subclass array
::method makearray
  say 'makeArray of the allItems answer'
  forward class (super)
```

```output
copy
isA Collection
makeArray
allItems of the answer
makeArray of the allItems answer
removeItem b
removeItem x
a
```

```rexx
a = .isaseq~new('a b')
r = a~difference(.array~of('b', 'x'))
say r~class~id r
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
::class isaseq subclass seq
::method isa
  say 'isA' arg(1)~id
  forward class (super) continue
  return 1
::method makearray
  say 'makeArray'
  return .array~of('MA')
::method append
  say 'append' arg(1)
  forward class (super)
::method removeitem
  say 'removeItem' arg(1)
  forward class (super)
```

```output
removeItem b
removeItem x
isA Array
makeArray
Array MA
```

r[class.orderedcollection.intersection]
`intersection(other)` answers a new, empty collection that `new` of the
receiver's class makes, and appends to it, for each item of the argument in
order, that item when a working copy of the receiver still has it, taking the
item out of the working copy, so that an item counts at most as many times as
the receiver has it. The result is in the order of the argument. The receiver
is not changed.

```rexx
a = .seq~new('a b b c')
say a~intersection(.seq~new('b c c d'))
say a~intersection(.array~of('c', 'a', 'b', 'b', 'b'))
say '['a~intersection(.array~of('x'))']' a
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
```

```output
b c
c a b b
[] a b b c
```

r[class.orderedcollection.intersection.sends]
`intersection` sends `new` to the class of the receiver, with no argument,
which makes the result, and then `copy` to the receiver, which makes the
working copy. It then reads the argument as [the operand
rule](r:class.orderedcollection.operand) says, and sends `makeArray` once to
the answer of `allItems`, as [`DO OVER`](r:instr.do.over) does. For each item
present in the array that came back, in order, it sends `hasItem` to the
working copy and, when that answers 1, `append` to the result with the item
and then `removeItem` to the working copy with the item. Nothing else is sent.

```rexx
a = .nseq~new('a b')
say 'start'
r = a~intersection(.tr~new)
say r
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
::class lseq subclass seq
::method copy
  say 'copy'
  forward class (super)
::method append
  say 'append' arg(1)
  forward class (super)
::method removeitem
  say 'removeItem' arg(1)
  forward class (super)
::method hasitem
  say 'hasItem' arg(1)
  forward class (super)
::method items
  say 'items'
  forward class (super)
::class nseq subclass lseq
::method new class
  say 'new' arg()
  forward class (super)
::class tr
::method isa
  say 'isA' arg(1)~id
  return 0
::method makearray
  say 'makeArray'
  return .ans~new
::class ans
::method allitems
  say 'allItems of the answer'
  return .la~of('b', 'x')
::class la subclass array
::method makearray
  say 'makeArray of the allItems answer'
  forward class (super)
```

```output
new 1
start
new 0
copy
new 1
isA Collection
makeArray
allItems of the answer
makeArray of the allItems answer
hasItem b
append b
removeItem b
hasItem x
b
```

r[class.orderedcollection.xor]
`xor(other)` answers a new collection: the object that the receiver's `copy`
answers, in which each item of the argument, in order, cancels one occurrence
of itself when a working copy of the receiver still has one, and is appended
otherwise. Occurrences therefore cancel one against one: an item that the
argument has more often than the receiver has it is appended once for each
occurrence left over. The receiver is not changed. When the last item has been
appended or removed, the result is sent `isA` with the class `Array`; when it
answers 1 the answer is what the result answers to `makeArray`, sent with no
argument, and otherwise the answer is the result.

```rexx
a = .seq~new('a b b c')
say a~xor(.seq~new('b c c d'))
say a~xor(.array~of('b', 'b', 'b'))
say '['a~xor(a)']' a
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
```

```output
a b c d
a c b
[] a b b c
```

r[class.orderedcollection.xor.sends]
`xor` sends `copy` to the receiver twice: the first answer is the result and
the second is a working copy. It then reads the argument as [the operand
rule](r:class.orderedcollection.operand) says, and sends `makeArray` once to
the answer of `allItems`, as [`DO OVER`](r:instr.do.over) does. For each item
present in the array that came back, in order, it sends `hasItem` to the
working copy; when that answers 1 it sends `removeItem` with the item to the
result and then to the working copy, and otherwise it sends `append` with the
item to the result. Then it sends `isA` with `Array` to the result, and
`makeArray` to it when that answered 1; nothing else is sent.

```rexx
a = .lseq~new('a b')
r = a~xor(.tr~new)
say r
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
::class lseq subclass seq
::method copy
  say 'copy'
  forward class (super)
::method append
  say 'append' arg(1)
  forward class (super)
::method removeitem
  say 'removeItem' arg(1)
  forward class (super)
::method hasitem
  say 'hasItem' arg(1)
  forward class (super)
::method items
  say 'items'
  forward class (super)
::class tr
::method isa
  say 'isA' arg(1)~id
  return 0
::method makearray
  say 'makeArray'
  return .ans~new
::class ans
::method allitems
  say 'allItems of the answer'
  return .la~of('b', 'x')
::class la subclass array
::method makearray
  say 'makeArray of the allItems answer'
  forward class (super)
```

```output
copy
copy
isA Collection
makeArray
allItems of the answer
makeArray of the allItems answer
hasItem b
removeItem b
removeItem b
hasItem x
append x
a x
```

```rexx
a = .isaseq~new('a b')
r = a~xor(.array~of('b', 'x'))
say r~class~id r
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
::class isaseq subclass seq
::method isa
  say 'isA' arg(1)~id
  forward class (super) continue
  return 1
::method makearray
  say 'makeArray'
  return .array~of('MA')
::method append
  say 'append' arg(1)
  forward class (super)
::method removeitem
  say 'removeItem' arg(1)
  forward class (super)
```

```output
removeItem b
removeItem b
append x
isA Array
makeArray
Array MA
```

r[class.orderedcollection.subset]
`subset(other)` answers 1 when the receiver has no item that the argument
lacks, and 0 otherwise, counting repeats: a copy of the receiver has one
occurrence of each item of the argument removed, and the answer is 1 when what
is left has no items, that is, when `items` of the copy answers a number equal
to 0. The number 0 is compared with the answer by `=`, so `0.0` and ` 0 ` are
equal to it, and an answer that is an object is compared as the number 0 sees
it, not by its own `=`, `\=`, `==` or `\==`. An empty receiver is a subset of
every collection, and a receiver is a subset of itself.

```rexx
a = .seq~new('a b')
say a~subset(.seq~new('a b c')) a~subset(.array~of('b', 'a', 'b'))
say a~subset(.array~of('a')) .seq~new('b b')~subset(.array~of('b'))
say .seq~new~subset(.array~new) a~subset(a)
say .fake~new('0.0')~subset(.array~new) .fake~new(' 0 ')~subset(.array~new)
say .fake~new(.n~new('A'))~subset(.array~new)
::class fake inherit orderedcollection
::method init
  expose count
  use arg count
::method copy
  return self
::method removeitem
  nop
::method items
  expose count
  return count
::class n
::method init
  expose s
  use arg s
::method '='
  say 'n ='
  return 1
::method '\='
  say 'n \='
  return 0
::method '=='
  say 'n =='
  return 1
::method '\=='
  say 'n \=='
  return 0
::method string
  expose s
  return s
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
```

```output
1 1
0 0
1 1
1 1
0
```

r[class.orderedcollection.subset.sends]
`subset` sends `isA` with the class `Collection` to the argument and, when
that does not answer 1, `makeArray` to it. It then sends `copy` to the
receiver, then `allItems` to the argument, or to what its `makeArray`
answered, and `makeArray` once to the answer of `allItems`, as
[`DO OVER`](r:instr.do.over) does. It sends `removeItem` to the copy with each
item present in the array that came back, in order, whether or not the copy
has it. Last it sends `items` to the copy and compares 0 with the answer by
`=`, so the `=` of the answer itself is never sent, and the result is 1 when
the two are equal as numbers. The order is that of a plain argument; for an
ordered one there is no `makeArray` before the `copy`.

```rexx
a = .lseq~new('a b')
say a~subset(.tr~new)
say a~subset(.oa~new)
::class seq inherit orderedcollection
::method init
  expose a
  a = .array~new
  if arg(1, 'E') then do w over arg(1)~makearray(' '); a~append(w); end
::method copy
  return self~class~new(self~string)
::method string
  expose a
  return a~makestring('L', ' ')
::method append
  expose a
  return a~append(arg(1))
::method removeitem
  expose a
  return a~removeitem(arg(1))
::method allitems
  expose a
  return a~allitems
::method items
  expose a
  return a~items
::method isempty
  expose a
  return a~items = 0
::class lseq subclass seq
::method copy
  say 'copy'
  forward class (super)
::method append
  say 'append' arg(1)
  forward class (super)
::method removeitem
  say 'removeItem' arg(1)
  forward class (super)
::method hasitem
  say 'hasItem' arg(1)
  forward class (super)
::method items
  say 'items'
  forward class (super)
::class tr
::method isa
  say 'isA' arg(1)~id
  return 0
::method makearray
  say 'makeArray'
  return .ans~new
::class ans
::method allitems
  say 'allItems of the answer'
  return .la~of('b', 'x')
::class la subclass array
::method makearray
  say 'makeArray of the allItems answer'
  forward class (super)
::class oa inherit orderedcollection
::method isa
  say 'isA' arg(1)~id
  forward class (super)
::method allitems
  say 'allItems'
  return .la~of('b', 'x')
```

```output
isA Collection
makeArray
copy
allItems of the answer
makeArray of the allItems answer
removeItem b
removeItem x
items
0
isA Collection
copy
allItems
makeArray of the allItems answer
removeItem b
removeItem x
items
0
```

## Deviations

r[class.orderedcollection.deviation.109]
An instance of a subclass of `String` that `subset` compares is read by its
own characters, and `MAKESTRING` is not sent to it ([Deviation
109](r:class.string.deviation.109)). That reaches the answer of `items` of the
copy, which 0 is compared with. ooRexx reads such an instance through its
`MAKESTRING` method. In the example `MAKESTRING` answers other characters.

```rexx
say .fake~new(.name~new('7'))~subset(.array~new)
::class name subclass string
::method makestring
  return '0'
::class fake inherit orderedcollection
::method init
  expose count
  use arg count
::method copy
  return self
::method removeitem
  nop
::method items
  expose count
  return count
```

```output defect=#140
0
```

r[class.orderedcollection.deviation.118]
An instance of a subclass of `List` that `appendAll` or one of the operations
that read a list gets as the answer of `allItems` is sent `makeArray`, so that
its own `makeArray` is used, as [the answer
rule](r:class.orderedcollection.operand.answer) says for any collection. ooRexx
uses the items such an instance holds and sends it no `makeArray`. In the
example the subclass answers other items than it holds.

```rexx
s = .seq~new
s~appendAll(.src~new(.llist~of('held')))
say s~items s~firstItem
s = .seq~new
say s~union(.src~new(.llist~of('held')))~items
::class llist subclass list
::method makearray
  say 'makearray sent'
  return .array~of('answered')
::class src inherit collection
::method init
  expose ans
  use arg ans
::method allitems
  expose ans
  return ans
::class seq subclass list
```

```output
makearray sent
1 answered
makearray sent
1
```
