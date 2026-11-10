---
element: "Supplier"
kind: class
rexxref: "Builtin Classes > Utility Classes > Supplier Class; Builtin Classes > Utility Classes > Supplier Class > new (Class Method); Builtin Classes > Utility Classes > Supplier Class > allIndexes; Builtin Classes > Utility Classes > Supplier Class > allItems; Builtin Classes > Utility Classes > Supplier Class > available; Builtin Classes > Utility Classes > Supplier Class > index; Builtin Classes > Utility Classes > Supplier Class > item; Builtin Classes > Utility Classes > Supplier Class > next; Builtin Classes > Utility Classes > Supplier Class > supplier"
---

# Supplier

A `Supplier` walks over a series of entries, one at a time. An entry is an
index and an item. The `supplier` method of every collection class answers a
supplier that holds the entries of the collection; the page of each collection
class says which entries those are and in what order. A program can also make
a supplier of its own from two arrays, one for the items and one for the
indexes. The loop `DO WITH INDEX name ITEM name OVER collection` walks the
supplier of a collection ([the loop rule](r:instr.do.with)). This page states
the class: how a supplier is made, how it is read and advanced, what it
answers at its end, and how copies and subclasses behave. The class
`StreamSupplier`, a supplier over the lines of a stream, has [a page of its
own](streamsupplier.md).

## The class

r[class.supplier.creation]
`Supplier` has the class method `new` and the instance methods `allIndexes`,
`allItems`, `available`, `index`, `item`, `next` and `supplier`, besides the
methods that every class has, and an instance method `init` that
[`new`](r:class.supplier.new) sends. It may have private methods besides
these; they are not part of this specification. Its only superclass is
`Object` and it inherits no mixin class ([the hierarchy
rule](r:class.index.hierarchy)). Its string value is that of any object that
does not define `string`, `a Supplier`.

```rexx
say .supplier~id .supplier~superclasses~items .supplier~superclass~id,
  .supplier~queryMixinClass
have = .set~new
s = .supplier~methods(.nil)
do while s~available
  have~put(s~index)
  s~next
end
names = 'allIndexes allItems available index item next supplier init'
count = 0
do name over names~makeArray(' ')
  if have~hasItem(name~upper) then count = count + 1
end
say count 'of 8'
t = .supplier~new(.array~of(1), .array~of(2))
say t~string '|' t~class~id
```

```output
Supplier 1 Object 0
8 of 8
a Supplier | Supplier
```

## Making a supplier

r[class.supplier.new]
`new(items, indexes)` answers a new supplier of the class that received it, so
a subclass makes instances of itself. It makes the object and then sends
`init` the arguments of `new`, as [the new rule](r:class.index.new) says, so
the arrays are read and checked by [`init`](r:class.supplier.init), whatever
the class. A subclass that declares its own `init` forwards to the `init` of
`Supplier` with the two arrays; an instance whose `init` never reached that
method is not specified.

```rexx
s = .supplier~new(.array~of('a', 'b'), .array~of(10, 20))
say s~class~id s~index s~item
t = .stamped~new('first')
say t~class~id t~label t~index t~item
call try '.supplier~new'
call try '.supplier~new(.array~of(1))'
call try '.supplier~new(.array~of(1), .array~of(2), 3)'
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=> made'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class stamped subclass supplier
::method init
  expose label
  use arg label
  self~init:super(.array~of('p', 'q'), .array~of(1, 2))
::method label
  expose label
  return label
```

```output
Supplier 10 a
STAMPED first 1 p
.supplier~new => 93.903
.supplier~new(.array~of(1)) => 93.903
.supplier~new(.array~of(1), .array~of(2), 3) => 93.902
```

r[class.supplier.init]
`init(items, indexes)` fills the supplier and puts it at its first entry. It
checks in this order: more than two arguments raise `SYNTAX 93.902` before
anything is read; a missing first argument raises `93.903`; the first argument
is converted, as the next rule says, and an answer that is not accepted raises
`98.913`; a missing second argument raises `93.903`; the second argument is
converted the same way. So a call with one argument that cannot be converted
raises `98.913`, and one with an argument that can raises `93.903`. Each
answer must be an `Array` of one dimension, whatever its items; `.nil`, a
string and an array of more than one dimension raise `98.913`, and what
another kind of object does is not specified; the supplier then has as many
entries as the first array has slots ([the walking
rules](r:class.supplier.available)). `init` sent to a supplier that has been
made already does the same again with new arrays and starts it over at its
first entry; the arguments are checked then as they are by `new`.

```rexx
a = .array~of('p', 'q', 'r')
s = .supplier~new(a, .array~of(1, 2, 3))
s~next; s~next
say s~item
s~init(.array~of('x'), .array~of('y'))
say s~item s~index s~available
call try 's~init'
call try 's~init(.array~of(1))'
call try 's~init(.nil, .array~of(1))'
call try 's~init(.array~of(1), .nil)'
call try 's~init(.array~of(1), .object~new)'
call try 's~init(.array~new(2, 2), .array~of(1))'
call try 's~init(.array~of(1), .array~new(2, 2))'
call try 's~init(.array~of(1), "text")'
call try 's~init(.nil)'
call try 's~init(.nil, .array~of(1), 1)'
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
r
x y 1
s~init => 93.903
s~init(.array~of(1)) => 93.903
s~init(.nil, .array~of(1)) => 98.913
s~init(.array~of(1), .nil) => 98.913
s~init(.array~of(1), .object~new) => 98.913
s~init(.array~new(2, 2), .array~of(1)) => 98.913
s~init(.array~of(1), .array~new(2, 2)) => 98.913
s~init(.array~of(1), "text") => ok
s~init(.nil) => 98.913
s~init(.nil, .array~of(1), 1) => 93.902
```

r[class.supplier.init.convert]
Each argument is converted by sending it `request` with the argument `ARRAY`
([the request rule](r:class.object.request)), the first argument before the
second; nothing else is sent to either. An `Array` of one dimension answers
itself, so the supplier holds that array and not a copy. An instance of a
subclass of `Array`, a `List` and a `String` answer through their `makeArray`
method ([the `makeArray` rule](r:class.array.makearray)) when they have one,
so the supplier holds the array that method answers; a string gives an array
of its lines. What the supplier holds is read by the walking methods directly,
and nothing is sent to the arrays after `init` has returned. A `request` that
answers an instance of a subclass of `Array` is accepted, and the supplier
holds that instance.

```rexx
.local~log = .array~new
x = .watched~of('p', 'q')
y = .watched~of(1, 2)
s = .supplier~new(x, y)
say .log~makeString('L', ' ')
.log~empty
say s~item s~index
s~next
say s~item s~index s~available
say '['.log~makeString('L', ' ')']'
l = .list~of('u', 'v')
t = .supplier~new(l, .array~of(7, 8))
say t~item
t~next
say t~item t~index
r = .supplier~new('one'||'0a'x||'two', .array~of(1, 2))
say r~item
r~next
say r~item
::class watched subclass array
::method request
  .log~append('request' arg(1))
  forward class (super)
::method makeArray
  .log~append('makeArray')
  forward class (super)
::method at
  .log~append('at')
  forward class (super)
::method size
  .log~append('size')
  forward class (super)
```

```output
request ARRAY makeArray request ARRAY makeArray
p 1
q 2 1
[]
u
v 8
one
two
```



```rexx
s = .supplier~new(.answers~new(.mine~of('m1', 'm2')), .array~of(1, 2))
say s~item s~index
s~next
say s~item s~index
call try '.supplier~new(.answers~new("x"), .array~of(1))'
call try '.supplier~new(.answers~new(.nil), .array~of(1))'
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=> made'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class mine subclass array
::class answers
::method init
  expose a
  use arg a
::method request
  expose a
  return a
```

```output defect=#201
m1 1
m2 2
.supplier~new(.answers~new("x"), .array~of(1)) => 98.913
.supplier~new(.answers~new(.nil), .array~of(1)) => 98.913
```

## Walking

r[class.supplier.available]
`available` answers 1 when the supplier is at an entry and 0 when it has
walked past the last one. A supplier starts at its first entry. The entries
are numbered from 1 and their number is the number of slots of the items array
(its `size`) at the moment of the call, so an array with empty slots has an
entry for each of them, and a slot that is removed with `delete` at the end of
the array takes its entry away. The supplier reads the arrays when it is
asked, and does not copy them, so a change made to an array that the supplier
holds shows in what it answers; [the next rule](r:class.supplier.item) gives
the entry. The method takes no argument: any argument raises `SYNTAX 93.902`,
also at the end.

```rexx
a = .array~of('p', 'q')
s = .supplier~new(a, .array~of(1, 2))
say s~available
a~append('r')
s~next; s~next
say s~available s~item
s~next
say s~available
a~delete(3)
say s~available
say .supplier~new(.array~new(2), .array~of(1, 2))~available
say .supplier~new(.array~of(), .array~of(1, 2))~available
call try 's~available(1)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1
1 r
0
0
1
0
s~available(1) => 93.902
```

r[class.supplier.item]
`item` answers the item of the current entry: the item of the items array at
the position of the entry, or `.nil` when that slot is empty. `index` answers
the index of the current entry: the item of the indexes array at the same
position, or `.nil` when that array has no item there, as it does when it is
shorter than the items array. When the supplier is not available both raise
`SYNTAX 93.937`. An argument raises `93.902` whether the supplier is available
or not, before the error of the end.

```rexx
s = .supplier~new(.array~of('p', , 'r'), .array~of(1, 2))
do while s~available
  say s~index s~item
  s~next
end
call try 's~item'
call try 's~index'
call try 's~item(1)'
call try 's~index(1)'
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=>' x
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 p
2 The NIL object
The NIL object r
s~item => 93.937
s~index => 93.937
s~item(1) => 93.902
s~index(1) => 93.902
```

r[class.supplier.next]
`next` moves the supplier to the following entry and answers no value. A
supplier that has moved past the last entry is at its end: `next` there raises
`SYNTAX 93.937`, as `index` and `item` do. An argument raises `93.902` first.
The method sends nothing to the arrays and nothing to the supplier itself.

```rexx
s = .supplier~new(.array~of('p', 'q'), .array~of(1, 2))
s~next
say s~item
s~next
say s~available
call try 's~next'
call try 's~next(1)'
call try 'y = .supplier~new(.array~of(1), .array~of(2))~next'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
q
0
s~next => 93.937
s~next(1) => 93.902
y = .supplier~new(.array~of(1), .array~of(2))~next => 91.999
```

## The remaining entries as arrays

r[class.supplier.allitems]
`allItems` answers an `Array` of the items of the entries from the current one
to the last, in order, and `allIndexes` an `Array` of their indexes; an empty
slot or a missing index is `.nil`, and the array has as many items as there
are entries left. The answers are new arrays of the class `Array`, never the
arrays the supplier holds. Both methods move the supplier to its end, so
`available` answers 0 afterwards and `index`, `item` and `next` raise
`SYNTAX 93.937`. Whichever of the two is sent first collects both arrays, at
the entry where the supplier was then; every later call of either answers the
very same `Array` objects as the first (they compare equal under `==`), and a
later change to the arrays that the supplier holds does not change them. A
copy of the supplier made after the first call answers them too. A supplier
that has walked to its end with `next` and has had neither method sent answers
empty arrays. Any argument raises `93.902`.

```rexx
items = .array~of(10, 20, 30)
s = .supplier~new(items, .array~of('a', 'b', 'c'))
s~next
say s~allItems~makeString('L', ',') s~available
items[3] = 'z'
say s~allItems~makeString('L', ',')
say s~allIndexes~makeString('L', ',') (s~allItems == s~allItems)
call try 's~item'
call try 's~next'
t = .supplier~new(.array~of(10, 20, 30), .array~of('a', 'b', 'c'))
say t~allIndexes~makeString('L', ',') t~available
say t~allItems~makeString('L', ',') t~allItems~class~id
u = .supplier~new(.array~of(10, , 30), .array~of('a'))
x = u~allItems
y = u~allIndexes
say x~items x~hasIndex(2) y~items y~hasIndex(3)
v = .supplier~new(.array~of(10, 20), .array~of('a', 'b'))
do while v~available
  v~next
end
say v~allItems~items v~allIndexes~items
c = .supplier~new(.array~of(10, 20), .array~of('a', 'b'))
c~allItems
d = c~copy
say d~available (d~allItems == c~allItems)
call try 'c~allItems(1)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output defect=#201
20,30 0
20,30
b,c 1
s~item => 93.937
s~next => 93.937
a,b,c 0
10,20,30 Array
3 1 3 1
0 0
0 1
c~allItems(1) => 93.902
```

r[class.supplier.allitems.sends]
For a receiver whose class is `Supplier` itself the two methods send nothing.
For an instance of a subclass they first send the receiver one private message
of their own, which is not part of this specification, and then walk the
remaining entries by messages to the receiver, so that an override of
`available`, `index`, `item` or `next` takes part: while `available` answers
true they send `index`, then `item`, then `next`, and when `available` answers
false the walk ends. The answers of `index` and of `item` are the elements of
the two arrays, whatever the overrides answer, and those arrays are the ones
that every later call answers. The two methods send nothing to the arrays and
nothing to the answers. A call that finds the arrays collected already sends
the private message only, and the same messages are sent whichever of the two
methods is the first call. In the example the first class counts the messages
it receives, the second changes what `item` and `index` answer, and the third
ends the walk early.

```rexx
.local~log = .array~new
.local~on = 0
s = .spy~new(.array~of(1, 2, 3), .array~of('a', 'b', 'c'))
s~next
.local~on = 1
x = s~allItems
.local~on = 0
say x~makeString('L', ',') s~available
say .log~items 'sends'
.log~empty
.local~on = 1
y = s~allIndexes
.local~on = 0
say y~makeString('L', ',') .log~items
u = .loud~new(.array~of(1, 2), .array~of('a', 'b'))
say u~allItems~makeString('L', ',') u~allIndexes~makeString('L', ',')
v = .short~new(.array~of(1, 2, 3, 4), .array~of('a', 'b', 'c', 'd'))
say v~allItems~makeString('L', ',') v~allIndexes~makeString('L', ',')
::class spy subclass supplier
::method available
  if .on then .log~append('available')
  forward class (super)
::method index
  if .on then .log~append('index')
  forward class (super)
::method item
  if .on then .log~append('item')
  forward class (super)
::method next
  if .on then .log~append('next')
  forward class (super)
::class loud subclass supplier
::method item
  return self~item:super~copies(2)
::method index
  return self~index:super~upper
::class short subclass supplier
::method init
  expose steps
  steps = 0
  forward class (super)
::method available
  expose steps
  return steps < 2 & self~available:super
::method next
  expose steps
  steps = steps + 1
  self~next:super
```

```output defect=#201
2,3 0
9 sends
b,c 0
11,22 A,B
1,2 a,b
```

r[class.supplier.supplier]
`supplier` answers the receiver itself, the same object and not a copy, so a
supplier can be given where a collection's `supplier` is wanted, as `DO WITH`
does ([the loop rule](r:instr.do.with)). It sends nothing. An argument raises
`SYNTAX 93.902`. A subclass instance answers itself, whatever its class.

```rexx
s = .supplier~new(.array~of('p', 'q'), .array~of(1, 2))
say (s~supplier == s) s~supplier~class~id
do with index i item x over s
  say i x
end
say s~available
t = .twin~new(.array~of('u'), .array~of(7))
say t~supplier~class~id (t~supplier == t)
signal on syntax name bad
say s~supplier(1)
bad: say condition('o')~code
::class twin subclass supplier
```

```output
1 Supplier
1 p
2 q
0
TWIN 1
93.902
```

r[class.supplier.copy]
`copy` answers a new supplier of the same class as the receiver, at the same
entry. The two are independent suppliers from then on, each with its own
entry, but they hold the same arrays and not copies of them: what is put into
an array shows in both. A copy sends nothing to the receiver, to `init` or to
the arrays, and an argument raises `SYNTAX 93.902`. A copy made after
[`allItems` or `allIndexes`](r:class.supplier.allitems) has been sent answers
the same arrays as the receiver does.

```rexx
s = .supplier~new(.array~of('p', 'q', 'r'), .array~of(1, 2, 3))
c = s~copy
s~next
say s~item c~item s~available c~available
c~next; c~next; c~next
say s~item c~available
say c~class~id (c == s) .twin~new(.array~of(1), .array~of(2))~copy~class~id
::class twin subclass supplier
```

```output
q p 1 1
q 0
Supplier 0 TWIN
```



```rexx
a = .array~of('p', 'q')
s = .supplier~new(a, .array~of(1, 2))
c = s~copy
a[2] = 'changed'
s~next; c~next
say s~item c~item
::class twin subclass supplier
```

```output defect=#201
changed changed
```

