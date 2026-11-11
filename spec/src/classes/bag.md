---
element: "Bag"
kind: class
rexxref: "Builtin Classes > Collection Classes > Bag Class; Builtin Classes > Collection Classes > Bag Class > new (Class Method); Builtin Classes > Collection Classes > Bag Class > of (Class Method); Builtin Classes > Collection Classes > Bag Class > []; Builtin Classes > Collection Classes > Bag Class > []=; Builtin Classes > Collection Classes > Bag Class > allAt; Builtin Classes > Collection Classes > Bag Class > allIndex; Builtin Classes > Collection Classes > Bag Class > allIndexes; Builtin Classes > Collection Classes > Bag Class > allItems; Builtin Classes > Collection Classes > Bag Class > at; Builtin Classes > Collection Classes > Bag Class > difference; Builtin Classes > Collection Classes > Bag Class > empty; Builtin Classes > Collection Classes > Bag Class > hasIndex; Builtin Classes > Collection Classes > Bag Class > hasItem; Builtin Classes > Collection Classes > Bag Class > index; Builtin Classes > Collection Classes > Bag Class > intersection; Builtin Classes > Collection Classes > Bag Class > isEmpty; Builtin Classes > Collection Classes > Bag Class > items; Builtin Classes > Collection Classes > Bag Class > makeArray; Builtin Classes > Collection Classes > Bag Class > put; Builtin Classes > Collection Classes > Bag Class > putAll; Builtin Classes > Collection Classes > Bag Class > remove; Builtin Classes > Collection Classes > Bag Class > removeAll; Builtin Classes > Collection Classes > Bag Class > removeItem; Builtin Classes > Collection Classes > Bag Class > subset; Builtin Classes > Collection Classes > Bag Class > supplier; Builtin Classes > Collection Classes > Bag Class > union; Builtin Classes > Collection Classes > Bag Class > uniqueIndexes; Builtin Classes > Collection Classes > Bag Class > xor"
deviations: [33, 45, 78, 84, 109, 125]
---

# Bag

A `Bag` holds members, each an object that is its own index, and keeps every
object that is put: `put` always adds, so one object can be a member many
times, and `items` counts them all. Members are found as a
[relation](relation.md) finds its indexes, by the `hashCode` and the `==` that
they answer to; the methods that take an item compare by `==` as well. The
class inherits [`MapCollection`](mapcollection.md) and
[`SetCollection`](setcollection.md), and through them
[`Collection`](collection.md); [the hierarchy rule](r:class.index.hierarchy)
lists what it inherits. Like [`Set`](set.md) it declares its own set
operations, which read their argument as a list of members and not as a list
of entries, and which count how often a member occurs.

A bag has no `unknown` method: a message that [the method
search](r:class.methods.unknown) does not resolve raises
[NOMETHOD](r:cond.nomethod.raise), as the error rule of [the
chapter](r:class.index.errors) says. Examples that show an error use a small
routine `try`, as the builtin pages do. Where several members qualify for a
method that answers one, which one it answers is not specified, and the examples
then hold one member or sort the answers ([the hash order
rule](r:class.bag.deviation.33)).

## The class

r[class.bag.creation]
`Bag` declares the instance methods `[]`, `[]=`, `allAt`, `allIndex`,
`allIndexes`, `allItems`, `at`, `difference`, `empty`, `hasIndex`, `hasItem`,
`index`, `intersection`, `isEmpty`, `items`, `makeArray`, `put`, `putAll`,
`remove`, `removeAll`, `removeItem`, `subset`, `supplier`, `union`,
`uniqueIndexes` and `xor`, the `init` that `new` runs ([the new
rule](r:class.bag.new)) and the class methods `new` and `of`; the `copy` of
`Object` copies a bag as [the copy rule](r:class.bag.copy) says. The instance
methods `disjoint` and `equivalent` are those of [`Collection`](collection.md)
([one rule](r:class.bag.inherited) covers them). A bag has no `makeString`, so
it is not a string ([the string rule](r:class.bag.string)), and no `unknown`.
Its superclasses are `Object`, `MapCollection` and `SetCollection`, in that
order.

```rexx
b = .bag~new
n = 0
m = .bag~methods(.nil)
do while m~available
  n += 1
  m~next
end
say b~class~id n 'own methods' .bag~hasMethod('OF') b~hasMethod('OF')
do c over b~class~superclasses
  say c~id
end
say b~hasMethod('UNION') b~hasMethod('DISJOINT') b~hasMethod('EQUIVALENT')
say b~hasMethod('MAKESTRING') b~hasMethod('UNKNOWN') b~hasMethod('ALLAT')
```

```output
Bag 27 own methods 1 0
Object
MapCollection
SetCollection
1 1 1
0 0 1
```

## Making a bag

r[class.bag.new]
`new` makes an empty bag. Its optional argument is a hint for how large the
bag will grow; no method that this page specifies answers differently for it.
It must be a whole number that is not negative, or `SYNTAX 93.923` is raised.
A second argument raises `SYNTAX 93.902`, and omitted arguments at the end
count for nothing. `new` sends `init` to the new object with the arguments it
received, so a subclass's `init` sees them ([the new
rule](r:class.index.new)). The checks are those of the `init` that `Bag`
supplies: they come after the `init` of a subclass has run, and a subclass
whose `init` does not pass the arguments on is not refused for them.

```rexx
say .bag~new~items .bag~new(0)~items .bag~new(3, )~items
call try '.bag~new(-1)'
call try '.bag~new("x")'
call try '.bag~new(.nil)'
call try '.bag~new(1.5)'
call try '.bag~new(1, 2)'
x = .watch~new(7)
call try '.watch~new("x")'
say .quiet~new("x", 2, 3)~items
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class watch subclass bag
::method init
  say 'init' arg()
  forward class (super)
::class quiet subclass bag
::method init
  return
```

```output
0 0 0
.bag~new(-1) => 93.923
.bag~new("x") => 93.923
.bag~new(.nil) => 93.923
.bag~new(1.5) => 93.923
.bag~new(1, 2) => 93.902
init 1
init 1
.watch~new("x") => 93.923
0
```

r[class.bag.of]
`of(item, ...)` answers a new bag of the class that received it that holds the
arguments, each as often as it is given, and with no argument an empty bag. An
argument that is omitted before the last one raises `SYNTAX 93.903`, and a
trailing omitted argument counts for nothing. The method does not send `new`
to the class and does not send `put` to the bag: it sends `init` with no
argument to the new object and stores the arguments itself, so an override of
`new` or `put` in a subclass is not used.

```rexx
b = .bag~of('a', 'b', 'a', 1, 1.0, .nil)
say b~items b~items('a') b~items(1.0) b~items(.nil) .bag~of~items
x = .spy~of('p', 'q', 'p')
say x~class~id x~items
call try '.bag~of(1, , 2)'
call try '.bag~of(, 1)'
say .bag~of(1, )~items .bag~of(, )~items
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class spy subclass bag
::method new class
  say 'new'
  forward class (super)
::method init
  say 'init' arg()
  forward class (super)
::method put
  say 'put'
  forward class (super)
```

```output
6 2 1 1 0
init 0
SPY 3
.bag~of(1, , 2) => 93.903
.bag~of(, 1) => 93.903
1 0
```

## Members

r[class.bag.members]
A member is any object, `.nil` and the null string included, and the bag
stores every object that was put, whether or not an equal one is there. A
member is found by [the lookup](r:class.bag.members.sends): `hashCode` answers
the same string for the argument and for the stored object, and `==`, sent to
the argument with the stored object as argument, answers true. For an object
that declares neither method that is the same object, and an `Array` is its
own member only. A string is a member of its characters, compared by [the
strict comparison](r:expr.comparison.strict): case counts, a leading or
trailing blank counts, and `5` and `5.0` are two members. A class that
declares `==` has to declare `hashCode` so that equal objects hash alike, as
[the hash code rule](r:class.object.hashcode) says. A method that takes an
index considers every member that is equal to the argument, and the bag does
not merge them: the objects stay distinct, and `allItems` answers each of
them.

```rexx
b = .bag~new
do x over .array~of('Key', 'KEY', 5, 5.0, ' a', '', .nil, 'Key')
  b~put(x)
end
a = .array~of(1)
b~put(a)
say b~items b~items('Key') b~items('key') b~items('5.0') b~items('a')
say b~items(a) b~items(.array~of(1)) b~items('') b~items(.nil)
b~put(.eq~new('x', 'first'))
b~put(.eq~new('x', 'second'))
say b~items b~items(.eq~new('x', 'probe')),
  b~allAt(.eq~new('x', 'probe'))~items
t = .array~new
do i over b~allAt(.eq~new('x', 'probe'))
  t~append(i~tag)
end
say t~sort~makeString('L', ' ')
::class eq
::attribute tag
::method init
  expose n tag
  use arg n, tag
::method '=='
  expose n
  use arg o
  if \o~isA(.eq) then return 0
  return n == o~n
::method n
  expose n
  return n
::method hashCode
  expose n
  return n~hashCode
```

```output
9 2 0 1 0
1 0 1 1
11 2 2
first second
```

r[class.bag.members.sends]
`at`, `[]`, `hasIndex`, `hasItem`, `index`, `remove`, `removeItem` and `items`
with an index look a member up by sending `hashCode` to the argument, once, and
then `==` to the argument with a stored object as the argument, for stored
objects that have the same hash code, until one answers true; each of those is
compared before the lookup gives up. Which further stored objects are compared,
by `==` or `hashCode`, and how often, is not specified ([the hash order
rule](r:class.bag.deviation.33)); when the bag is empty, the only message is
`hashCode`. `allAt`, `removeAll` and `supplier` with an index send `hashCode`
once and `==` to the stored objects of the same hash code, how often to each
being left open by [the hash order rule](r:class.bag.deviation.33). `put` and
`[]=` send `hashCode` to the item and nothing else: no `==` is sent, as a member
is added whether or not an equal one is there; with an index they send `==` to
the item first ([the put rule](r:class.bag.put)). The very object that is stored
is no exception to the `==` ([the deviation rule](r:class.bag.deviation.78)). In
the example every member hashes to the same string, so each stored member is
compared; `index` is shown by [the item rule](r:class.bag.hasitem.sends), and
the messages of `uniqueIndexes` are left open by [the hash order
rule](r:class.bag.deviation.33).

```rexx
.local~log = .array~new
say 'empty:' .bag~new~at(.k~new('p')) .local~log~makeString('L', ' | ')
calls = 'b~at(.k~new("p"))|b[.k~new("p")]|b~hasIndex(.k~new("p"))',
  '|b~hasItem(.k~new("p"))|b~items(.k~new("p"))|b~remove(.k~new("p"))',
  '|b~removeItem(.k~new("p"))|b~put(.k~new("p"))',
  '|b[.k~new("p")] = .k~new("p")',
  '|b~remove(.k~new("s1"))|b~put(.k~new("s1"))|b~allAt(.k~new("p"))~items'
do c over calls~makeArray('|')
  b = .bag~new
  b~put(.k~new('s1'))
  .local~log~empty
  interpret c
  say c~strip '=>' .local~log~makeString('L', ' | ')
end
exit
::class k
::method init
  expose tag
  use arg tag
::method tag
  expose tag
  return tag
::method '=='
  expose tag
  use arg o
  .local~log~append('==' tag o~tag)
  return tag == o~tag
::method hashCode
  expose tag
  .local~log~append('hash' tag)
  return 'same'
```

```output
empty: The NIL object hash p
b~at(.k~new("p")) => hash p | == p s1
b[.k~new("p")] => hash p | == p s1
b~hasIndex(.k~new("p")) => hash p | == p s1
b~hasItem(.k~new("p")) => hash p | == p s1
b~items(.k~new("p")) => hash p | == p s1
b~remove(.k~new("p")) => hash p | == p s1
b~removeItem(.k~new("p")) => hash p | == p s1
b~put(.k~new("p")) => hash p
b[.k~new("p")] = .k~new("p") => == p p | hash p
b~remove(.k~new("s1")) => hash s1 | == s1 s1
b~put(.k~new("s1")) => hash s1
b~allAt(.k~new("p"))~items => hash p | == p s1
```

```rexx
.local~log = .array~new
do c over 'removeAll allAt supplier'~makeArray(' ')
  b = .bag~new
  b~put(.k~new('s1'))
  b~put(.k~new('s1'))
  .local~log~empty
  x = b~send(c, .k~new('s1'))
  say c b~items .local~log~makeString('L', ' | ')
end
::class k
::method init
  expose tag
  use arg tag
::method '=='
  expose tag
  use arg o
  return tag == o~tag
::method tag
  expose tag
  return tag
::method hashCode
  expose tag
  .local~log~append('hash' tag)
  return 'same'
```

```output
removeAll 0 hash s1
allAt 2 hash s1
supplier 2 hash s1
```

## Adding and reading members

r[class.bag.put]
`put(item)` and `put(item, index)` add the item and answer nothing;
`[index] = item` is the same as `put(item, index)`, and `[] = item` the same
as `put(item)`. An index may be given only as another name for the item: when
one is, and the item is not the very object, `==` is sent to the item with the
index as argument first, and an answer that is not true raises `SYNTAX 93.949`
and leaves the bag unchanged. An index that is omitted counts for nothing. The
methods take one or two arguments: none, or an omitted item, raises
`SYNTAX 88.901`, and a third argument raises `SYNTAX 93.902`. The item may be
any object, `.nil` included, and is added also when an equal member is held,
so `items` grows by one.

```rexx
b = .bag~new
b~put('a')
b~put('a', 'a')
b['b'] = 'b'
b[] = 'a'
b~put(.nil)
b~put('d', )
say b~items b~items('a') b~items('d') b~items(.nil)
call try 'b~put("a", "A")'
call try 'b~put(1, 2)'
call try 'b["x"] = "y"'
call try 'b~put'
call try 'b~put(, "a")'
call try 'b~put("a", "a", "a")'
say b~items
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
6 3 1 1
b~put("a", "A") => 93.949
b~put(1, 2) => 93.949
b["x"] = "y" => 93.949
b~put => 88.901
b~put(, "a") => 88.901
b~put("a", "a", "a") => 93.902
6
```

r[class.bag.at]
`at(index)` and `[index]` answer the stored object that is equal to the index,
and `.nil` when there is none; when several members are equal, which one is
answered is not specified ([the hash order rule](r:class.bag.deviation.33)).
The methods take exactly one argument: none, or an omitted one, raises
`SYNTAX 88.901`, and a second raises `SYNTAX 93.902`; the number of arguments
is checked before the index is looked up. An error that the `hashCode` of the
index raises is the error of the message.

```rexx
b = .bag~new
b~put('item')
b~put('item')
say b~at('item') b['item'] b~at('ITEM') (b~at('zz') == .nil),
  (b~at(.nil) == .nil)
call try 'b~at'
call try 'b~at(, "x")'
call try 'b~at("a", "b")'
call try 'b[]'
call try 'b~at(.hostile~new)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class hostile
::method hashCode
  raise syntax 93.900 array ('stop')
```

```output
item item The NIL object 1 1
b~at => 88.901
b~at(, "x") => 93.902
b~at("a", "b") => 93.902
b[] => 88.901
b~at(.hostile~new) => 93.900
```

r[class.bag.allat]
`allAt(index)` answers a new `Array` of the stored objects that are equal to the
argument, one for each member, in an order that is not specified ([the hash
order rule](r:class.bag.deviation.33)), and an empty `Array` when there is none.
`allIndex(item)` is the same question asked with the item: it answers a new
`Array` of the stored objects equal to it, and for a bag the two answers are
alike. Both are instances of `Array` for a subclass receiver too, and do not
follow later changes. Each takes exactly one argument: none, or an omitted one,
raises `SYNTAX 93.903`, and a second raises `SYNTAX 93.902`. `allAt` looks the
argument up as [the lookup rule](r:class.bag.members.sends) says; `allIndex`
sends `==` to the argument with the stored objects as the argument, and no
`hashCode`; how often is left open by [the hash order
rule](r:class.bag.deviation.33).

```rexx
b = .many~of('a', 'b', 'a', 'a')
x = b~allAt('a')
y = b~allIndex('a')
say x~class~id x~items y~class~id y~items b~allAt('zz')~items,
  b~allIndex('zz')~items
say x~makeString('L', ',') y~makeString('L', ',')
b~put('a')
say x~items b~allAt('a')~items
call try 'b~allAt'
call try 'b~allIndex'
call try 'b~allAt(, 1)'
call try 'b~allIndex(1, 2)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass bag
```

```output
Array 3 Array 3 0 0
a,a,a a,a,a
3 4
b~allAt => 93.903
b~allIndex => 93.903
b~allAt(, 1) => 93.902
b~allIndex(1, 2) => 93.902
```

## Questions about the contents

r[class.bag.hasindex]
`hasIndex(index)` answers 1 when a member is equal to the argument and 0
otherwise. It takes exactly one argument: none, or an omitted one, raises
`SYNTAX 88.901`, and a second raises `SYNTAX 93.902`.

```rexx
b = .bag~of('k', .nil)
say b~hasIndex('k') b~hasIndex('K') b~hasIndex('zz') b~hasIndex(.nil)
call try 'b~hasIndex'
call try 'b~hasIndex("a", "b")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 0 0 1
b~hasIndex => 88.901
b~hasIndex("a", "b") => 93.902
```

r[class.bag.items]
`items` answers the number of members, each counted as often as it was put,
and `isEmpty` answers 1 when there is none and 0 otherwise. `items(index)`
answers how many members are equal to the index, 0 when none is. `items` takes
at most one argument and `isEmpty` none: a surplus argument raises
`SYNTAX 93.902`; an index that is omitted counts for nothing.

```rexx
b = .bag~new
say b~items b~isEmpty
b~put('a')
b~put(.nil)
b~put('a')
say b~items b~isEmpty b~items('a') b~items(.nil) b~items('zz') b~items('a', )
call try 'b~items(1, 2)'
call try 'b~isEmpty(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
0 1
3 0 2 1 0 2
b~items(1, 2) => 93.902
b~isEmpty(1) => 93.902
```

r[class.bag.hasitem]
`hasItem(item)` answers 1 when a member is equal to the item and 0 otherwise;
for a bag it is the same question as `hasIndex`. `index(item)` looks the item up
as an index, as [the lookup rule](r:class.bag.members.sends) says, and answers
the stored object it finds, and `.nil` when there is none. `index` takes exactly
one argument and `hasItem` one or two: for each, none, or an omitted first one,
raises `SYNTAX 93.903`, and a surplus argument raises `SYNTAX 93.902`; the count
is checked first, so `index(, 1)` raises `SYNTAX 93.902`. The second argument of
`hasItem` is an index, which may be given only as another name for the item; an
index equal to the item gives the answer of `hasItem(item)`, and an unequal one
answers 0 ([the deviation rule](r:class.bag.deviation.45)).

```rexx
b = .bag~of('x', .nil, 'x')
say b~hasItem('x') b~hasItem('X') b~hasItem(.nil) b~hasItem('zz')
say b~index('x') (b~index('q') == .nil) b~hasItem('x', 'x') b~hasItem('x', )
call try 'b~hasItem'
call try 'b~index'
call try 'b~hasItem(, 1)'
call try 'b~hasItem(1, 1, 1)'
call try 'b~index(1, 2)'
call try 'b~index(, 1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 0 1 0
x 1 1 1
b~hasItem => 93.903
b~index => 93.903
b~hasItem(, 1) => 93.903
b~hasItem(1, 1, 1) => 93.902
b~index(1, 2) => 93.902
b~index(, 1) => 93.902
```

r[class.bag.hasitem.sends]
`hasItem(item)` and `removeItem(item)` look the item up as [the lookup
rule](r:class.bag.members.sends) says, with `hashCode` and `==` to the argument,
and `hasItem(item, index)` and `removeItem(item, index)` send `==` to the item
with the index as argument first. A search with the very object that is stored
is no exception: its `==` is sent as for any other item ([the deviation
rule](r:class.bag.deviation.78)). `index(item)` sends what `hasItem(item)`
sends.

```rexx
.local~log = .array~new
b = .bag~new
b~put(.k~new('s1'))
calls = 'b~hasItem(.k~new("p"))|b~hasItem(.k~new("A"), .k~new("A"))',
  '|b~removeItem(.k~new("p"))'
do c over calls~makeArray('|')
  .local~log~empty
  interpret 'x =' c
  say c~strip '=>' .local~log~makeString('L', ' | ')
end
::class k
::method init
  expose tag
  use arg tag
::method tag
  expose tag
  return tag
::method '=='
  expose tag
  use arg o
  .local~log~append('==' tag o~tag)
  return tag == o~tag
::method hashCode
  expose tag
  .local~log~append('hash' tag)
  return 'same'
```

```output
b~hasItem(.k~new("p")) => hash p | == p s1
b~hasItem(.k~new("A"), .k~new("A")) => == A A | hash A | == A s1
b~removeItem(.k~new("p")) => hash p | == p s1
```

```rexx
.local~log = .array~new
b = .bag~new
b~put(.k~new('s1'))
.local~log~empty
say b~index(.k~new('s1'))~tag '|' .local~log~makeString('L', ' | ')
.local~log~empty
say (.nil == b~index(.k~new('p'))) '|' .local~log~makeString('L', ' | ')
::class k
::method init
  expose tag
  use arg tag
::method tag
  expose tag
  return tag
::method '=='
  expose tag
  use arg o
  .local~log~append('==' tag o~tag)
  return tag == o~tag
::method hashCode
  expose tag
  .local~log~append('hash' tag)
  return 'same'
```

```output
s1 | hash s1 | == s1 s1
1 | hash p | == p s1
```

```rexx
.local~log = .array~new
b = .bag~new
b~put(.k~new('s1'))
.local~log~empty
say (.nil == b~removeItem(.k~new('A'), .k~new('B'))) '|',
  .local~log~makeString('L', ' | ')
.local~log~empty
say b~removeItem(.k~new('s1'), .k~new('s1'))~tag '|',
  .local~log~makeString('L', ' | ')
::class k
::method init
  expose tag
  use arg tag
::method tag
  expose tag
  return tag
::method '=='
  expose tag
  use arg o
  .local~log~append('==' tag o~tag)
  return tag == o~tag
::method hashCode
  expose tag
  .local~log~append('hash' tag)
  return 'same'
```

```output defect=#170
1 | == A B
s1 | == s1 s1 | hash s1 | == s1 s1
```

## Removing members

r[class.bag.remove]
`remove(index)` removes one member that is equal to the argument and answers
the stored object, and `.nil` when there is none; the other equal members
stay, and which of them is removed is not specified ([the hash order
rule](r:class.bag.deviation.33)). It takes exactly one argument: none, or an
omitted one, raises `SYNTAX 88.901`, and a second raises `SYNTAX 93.902`.

```rexx
b = .bag~of('v', 'v', .nil)
say b~remove('v') b~items('v') b~remove('v') (b~remove('v') == .nil) b~items
say (b~remove(.nil) == .nil) (b~remove(.nil) == .nil) b~items
call try 'b~remove'
call try 'b~remove("a", "b")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
v 1 v 1 1
1 1 0
b~remove => 88.901
b~remove("a", "b") => 93.902
```

r[class.bag.removeall]
`removeAll(index)` removes every member that is equal to the argument and
answers them in a new `Array`, one object for each, in an order that is not
specified ([the hash order rule](r:class.bag.deviation.33)); it answers an empty
`Array` when there is none, and the answer is an instance of `Array` for a
subclass receiver too. It takes exactly one argument: none, or an omitted one,
raises `SYNTAX 93.903`, and a second raises `SYNTAX 93.902`.

```rexx
b = .many~of('a', 'b', 'a', 'a')
x = b~removeAll('a')
say x~class~id x~items x~makeString('L', ',') b~items b~hasIndex('a')
say b~removeAll('zz')~items b~removeAll('a')~items b~items
call try 'b~removeAll'
call try 'b~removeAll(, 1)'
call try 'b~removeAll("a", "b")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass bag
```

```output
Array 3 a,a,a 1 0
0 0 1
b~removeAll => 93.903
b~removeAll(, 1) => 93.902
b~removeAll("a", "b") => 93.902
```

r[class.bag.removeitem]
`removeItem(item)` removes one member that is equal to the argument and
answers the stored object, and `.nil` when there is none; it does what
`remove` does for a bag. `removeItem(item, index)` answers `.nil` and removes
nothing when the index is not equal to the item, and otherwise does what
`removeItem(item)` does. It takes one or two arguments: none, or an omitted
one, raises `SYNTAX 93.903`, and a third raises `SYNTAX 93.902`.

```rexx
b = .bag~of('one', 'one', 'two')
say b~removeItem('one') b~items b~removeItem('one', 'one') b~items
say (b~removeItem('zz') == .nil) (b~removeItem('two', 'other') == .nil),
  b~items
say b~removeItem('two', ) b~items
call try 'b~removeItem'
call try 'b~removeItem(, 1)'
call try 'b~removeItem(1, 2, 3)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
one 2 one 1
1 1 1
two 0
b~removeItem => 93.903
b~removeItem(, 1) => 93.903
b~removeItem(1, 2, 3) => 93.902
```

r[class.bag.empty]
`empty` removes every member and answers the bag itself. It takes no argument:
one raises `SYNTAX 93.902`.

```rexx
b = .bag~of('v', 'v')
e = b~empty
say (e == b) b~items b~isEmpty
call try 'b~empty(1)'
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
1 0 1
b~empty(1) => 93.902
```

## Lists of the members

r[class.bag.makearray]
`allIndexes` and `makeArray` answer a new `Array` that holds each member as
often as it occurs, and `allItems` does the same: for a bag index and item are
the same object, so the three answers are alike. `uniqueIndexes` answers a new
`Array` that holds one object for each group of equal members, and so as many
objects as there are different members. All four are instances of `Array` for a
subclass receiver too, never the receiver, and do not follow later changes. They
take no argument: one raises `SYNTAX 93.902`. The order is not specified ([the
hash order rule](r:class.bag.deviation.33)), and is the same for `allIndexes`,
`makeArray`, `allItems` and the supplier of one unchanged bag. `allIndexes`,
`allItems` and `makeArray` send no message to the bag or to the members; the
messages that `uniqueIndexes` sends to the members are left open by [the hash
order rule](r:class.bag.deviation.33).

```rexx
t = .many~new
do i = 1 to 20
  t~put('item'||(i // 5))
end
ix = t~allIndexes
it = t~allItems
say ix~class~id it~class~id t~makeArray~class~id t~uniqueIndexes~class~id
say ix~items it~items t~makeArray~items t~uniqueIndexes~items
say (ix~sort~makeString('L', ',') == it~sort~makeString('L', ','))
say t~uniqueIndexes~sort~makeString('L', ',')
a = t~allItems
t~put('x')
say a~items t~items
call try 't~allIndexes(1)'
call try 't~allItems(1)'
call try 't~makeArray(1)'
call try 't~uniqueIndexes(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass bag
```

```output
Array Array Array Array
20 20 20 5
1
item0,item1,item2,item3,item4
20 21
t~allIndexes(1) => 93.902
t~allItems(1) => 93.902
t~makeArray(1) => 93.902
t~uniqueIndexes(1) => 93.902
```

r[class.bag.supplier]
`supplier` answers a `Supplier` over the members, the index and the item of
each being the same object, once for each time the member occurs.
`supplier(index)` answers one over the members equal to the argument only. It
is a snapshot: later changes do not reach it. The order of `supplier` is that
of `allIndexes` ([the order rule](r:class.bag.makearray)). A supplier over no
member is not available. It takes at most one argument: a second raises
`SYNTAX 93.902`.

```rexx
b = .bag~of('a', 'a', 'b')
p = b~supplier
b~put('c')
b~remove('a')
say '['listed(p)']' '['listed(b~supplier('a'))']',
  '['listed(b~supplier('zz'))']'
say b~supplier~available .bag~new~supplier~available
call try 'b~supplier("a", "b")'
exit
listed: procedure
  use arg s
  l = .array~new
  do while s~available
    l~append(s~index'='s~item)
    s~next
  end
  return l~sort~makeString('L', ' ')
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
[a=a a=a b=b] [a=a] []
1 0
b~supplier("a", "b") => 93.902
```

## Copies and the string value

r[class.bag.copy]
`copy` answers a new bag of the class of the receiver with the same members,
each as often as before. The members are the same objects, and the two bags do
not see each other's changes. As [the copy rule](r:class.object.copy) says of
every object, the variables of a subclass instance are copied as they are, and
`init` is not sent. It takes no argument: one raises `SYNTAX 93.902`.

```rexx
t = .many~new
t~tag = 'T'
t~put('v1')
t~put('v1')
a = .array~new
t~put(a)
c = t~copy
say c~class~id c~items c~tag c~items('v1') (c == t) c~hasItem(a)
c~put('v2')
c~remove('v1')
say t~items c~items t~items('v1') c~items('v1')
call try 't~copy(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass bag
::attribute tag
::method init
  say 'init'
  forward class (super)
```

```output
init
MANY 3 T 2 0 1
3 3 2 1
t~copy(1) => 93.902
```

r[class.bag.string]
A bag has no `makeString` method, so its string value is the one of `Object`:
its name, `a Bag`, as [the string rule](r:class.index.string) says, whatever
the bag holds, and `request('STRING')` answers `.nil`.

```rexx
b = .bag~of('x')
say b '|' b~string '|' b 'x' b~hasMethod('MAKESTRING')
say (b~request('STRING') == .nil) .bag~new
```

```output
a Bag | a Bag | a Bag x 0
1 a Bag
```

## Adding the members of a collection

r[class.bag.putall]
`putAll(collection)` adds the members of the argument and answers the bag. It
takes exactly one argument: none raises `SYNTAX 93.901` and a second raises
`SYNTAX 93.902`. The argument is read as a list of members ([the operand
rule](r:class.bag.operand)), and each is added by sending `put` with it, as
the only argument, to the bag, so an override of `put` in a subclass is used
and every member counts, equal ones included. Unlike
[`MapCollection`](mapcollection.md), which stores the entries of a supplier,
it reads items only: the indexes of a map argument are not added.

```rexx
b = .many~new
r = b~putAll(.array~of('a', 'b', 'a'))
say (r == b) b~items b~items('a')
b~putAll(.directory~new~~put('v', 'k'))
b~putAll(.bag~of('z', 'a'))
say b~items b~items('v') b~items('k') b~items('a') b~items('z')
call try 'b~putAll'
call try 'b~putAll(1, 2)'
call try 'b~putAll(.nil)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass bag
::method put
  say 'put' arg(1)~class~id arg()
  forward class (super)
```

```output
put String 1
put String 1
put String 1
1 3 2
put String 1
put String 1
put String 1
6 1 0 3 1
b~putAll => 93.901
b~putAll(1, 2) => 93.902
b~putAll(.nil) => 93.948
```

## Combining bags

r[class.bag.operand]
`putAll`, `union`, `difference`, `intersection`, `xor` and `subset` read their
argument by sending it `isA` with the class `Collection`: an answer of 1 means
the argument is sent `allItems`, an answer of 0 means it is sent `makeArray`,
and any other answer, `.nil` among them, raises `SYNTAX 34.1`. An argument
that does not understand the message it is sent, `.nil` among them, raises
`SYNTAX 93.948`. The operations check the number of arguments first: none
raises `SYNTAX 93.901` and a second raises `SYNTAX 93.902`. What the argument
answers is read as [the list rule](r:class.bag.operand.list) says. Every
member of the argument counts, so an object that the argument holds twice is
read twice. The example records the messages that each kind of argument is
sent.

```rexx
.local~log = .array~new
do op over 'putAll union difference intersection xor subset'~makeArray(' ')
  line = op':'
  do k over .array~of(.coll~new, .plain~new(0), .plain~new(2), .bare~new)
    call try op, k
    line = line result
  end
  say line
end
exit
try: procedure expose line
  use arg op, k
  .local~log~empty
  signal on syntax name bad
  x = .bag~of('x')~send(op, k)
  return .local~log~makeString('L', '+')
bad: return condition('o')~code
::class coll inherit collection
::method isa
  .local~log~append('isA')
  forward class (super)
::method allitems
  .local~log~append('allItems')
  return .array~new
::method makearray
  .local~log~append('makeArray')
  return .array~new
::class plain
::method init
  expose a
  use arg a
::method isa
  expose a
  .local~log~append('isA')
  return a
::method allitems
  .local~log~append('allItems')
  return .array~new
::method makearray
  .local~log~append('makeArray')
  return .array~new
::class bare
```

```output
putAll: isA+allItems isA+makeArray 34.1 93.948
union: isA+allItems isA+makeArray 34.1 93.948
difference: isA+allItems isA+makeArray 34.1 93.948
intersection: isA+allItems isA+makeArray 34.1 93.948
xor: isA+allItems isA+makeArray 34.1 93.948
subset: isA+allItems isA+makeArray 34.1 93.948
```

r[class.bag.operand.list]
The answer of `allItems` or `makeArray` is read by sending it `items`, with no
argument, and then `[]` with each position from 1 up to the number that `items`
answered, in that order; each answer of `[]` is a member of the argument. The
answer need not be an `Array`: an object that understands the two messages will
do, and an answer that lacks one raises `SYNTAX 93.948`. A subclass of `Array`
is sent the two messages too, and no other. A position that holds nothing reads
`.nil`, which is then a member, so an `Array` with a hole is read for as many
positions as it has items, not as its size. An answer that is a `List`, or an
instance of a subclass of `List`, is the one kind read otherwise: its members
are all its items, in the order of the list, not the entries at the positions 1
up to its number of items ([Deviation 125](r:class.bag.deviation.125)). The
answer of `items` is read as a number: one that is not a number raises `SYNTAX
41.1`, and a negative one reads no position. An error that `[]` raises is the
error of the operation, after the members before it have been handled.

```rexx
a = .array~new(5)
a[2] = 'two'
b = .bag~new
b~putAll(.pk~new(a))
say b~items b~items('two') b~items(.nil)
b = .bag~new
b~putAll(.pk~new(.walk~new(3, 'e')))
say b~items b~allItems~sort~makeString('L', ',')
b = .bag~new
b~putAll(.pk~new(.logarr~of('p', 'q')))
say b~allItems~sort~makeString('L', ',')
call try 'b~putAll(.pk~new(.walk~new("x", "e")))'
b = .bag~new
call try 'b~putAll(.pk~new(.walk~new(2, "boom")))'
say b~items b~hasItem('boom1')
call try 'b~putAll(.pk~new(.object~new))'
say .bag~new~~putAll(.pk~new(.walk~new(-1, 'e')))~items
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class pk inherit collection
::method init
  expose answer
  use arg answer
::method allitems
  expose answer
  return answer
::class walk
::method init
  expose n kind
  use arg n, kind
::method items
  expose n
  return n
::method '[]'
  expose kind
  if kind = 'boom' & arg(1) = 2 then raise syntax 93.900 array ('stop')
  return kind || arg(1)
::class logarr subclass array
::method items
  say 'items'
  forward class (super)
::method '[]'
  say '[]' arg(1)
  forward class (super)
::method makearray
  say 'makeArray'
  forward class (super)
```

```output
1 0 1
3 e1,e2,e3
items
[] 1
[] 2
p,q
b~putAll(.pk~new(.walk~new("x", "e"))) => 41.1
b~putAll(.pk~new(.walk~new(2, "boom"))) => 93.900
1 1
b~putAll(.pk~new(.object~new)) => 93.948
0
```

r[class.bag.union]
`union(other)` answers the object that `copy` of the receiver answers, after
every member of the argument has been added to it: for each member, in the
order of [the list rule](r:class.bag.operand.list), `put` is sent to the copy
with the member as the only argument. The answer holds as many members as the
receiver and the argument have together, and is an instance of the class of
the receiver. The receiver and the argument are not changed.

```rexx
a = .bag~of('a', 'a', 'b')
b = .bag~of('b', 'c', 'c')
u = a~union(b)
say u~class~id u~items (u \== a) a~items b~items sorted(u)
say sorted(a~union(.array~of('x', 'x', 'a')))
say sorted(a~union(.directory~new~~put('v', 'k'))),
  sorted(a~union(.set~of('b', 'z')))
say sorted(a~union('xy')) sorted(.bag~new~union(a)) sorted(a~union(a))
exit
sorted: procedure
  use arg c
  return c~allItems~sort~makeString('L', ',')
```

```output
Bag 6 1 3 3 a,a,b,b,c,c
a,a,a,b,x,x
a,a,b,v a,a,b,b,z
a,a,b,xy a,a,b a,a,a,a,b,b
```

r[class.bag.difference]
`difference(other)` answers the object that `copy` of the receiver answers,
after one member equal to each member of the argument has been removed from
it: for each member, in the order of [the list
rule](r:class.bag.operand.list), `remove` is sent to the copy with the member.
A member that the receiver holds twice and the argument once stays once, and a
member the copy does not hold removes nothing. The receiver and the argument
are not changed.

```rexx
a = .bag~of('a', 'a', 'b', 'c')
b = .bag~of('a', 'c', 'c', 'd')
d = a~difference(b)
say d~class~id d~items (d \== a) a~items sorted(d)
say sorted(a~difference(.array~of('a', 'a', 'a', 'q')))
say sorted(a~difference(.directory~new~~put('b', 'k'))),
  '['sorted(a~difference(a))']'
exit
sorted: procedure
  use arg c
  return c~allItems~sort~makeString('L', ',')
```

```output
Bag 2 1 4 a,b
b,c
a,a,c []
```

r[class.bag.intersection]
`intersection(other)` answers a new bag of the class of the receiver, made by
`new` with no argument, that holds each member as often as the receiver and
the argument both hold it. For each member of the argument, in the order of
[the list rule](r:class.bag.operand.list), `hasIndex` is sent to the copy of
the receiver with the member and, when that answers true, `put` with the
member as the only argument is sent to the new bag and `remove` with the
member to the copy. The receiver and the argument are not changed, and the
answer is not the copy.

```rexx
a = .bag~of('a', 'a', 'b', 'c')
b = .bag~of('a', 'c', 'c', 'd', 'a', 'a')
i = a~intersection(b)
say i~class~id i~items (i \== a) a~items sorted(i)
say sorted(a~intersection(.array~of('a', 'q', 'b'))) sorted(a~intersection(a))
say '['sorted(a~intersection(.bag~new))']',
  '['sorted(.bag~new~intersection(a))']'
exit
sorted: procedure
  use arg c
  return c~allItems~sort~makeString('L', ',')
```

```output
Bag 3 1 4 a,a,c
a,b a,a,b,c
[] []
```

r[class.bag.xor]
`xor(other)` answers the object that `copy` of the receiver answers, holding
each member as many times as the receiver and the argument differ in how often
they hold it. For each member of the argument, in the order of [the list
rule](r:class.bag.operand.list), `hasIndex` is sent to the copy with the
member and then either `remove` with the member to the copy, when that
answered true, or `put` with the member as the only argument to a new bag made
by `new` with no argument. Last, the copy is sent `putAll` with the new bag as
the argument, also when the new bag is empty. A member that the argument holds
more often than the receiver is therefore in the answer for the surplus, and
one that the receiver holds more often for its surplus. The receiver and the
argument are not changed.

```rexx
a = .bag~of('a', 'a', 'b', 'c')
b = .bag~of('a', 'c', 'c', 'd')
x = a~xor(b)
say x~class~id x~items (x \== a) a~items b~items sorted(x)
say sorted(a~xor(.array~of('a', 'z', 'z')))
say sorted(a~xor(a)) '|' sorted(a~xor(.bag~new)) '|' sorted(.bag~new~xor(a))
exit
sorted: procedure
  use arg c
  return c~allItems~sort~makeString('L', ',')
```

```output
Bag 4 1 4 4 a,b,c,d
a,b,c,z,z
 | a,a,b,c | a,a,b,c
```

r[class.bag.subset]
`subset(other)` answers 1 when every member of the receiver is held by the
argument at least as often, and 0 otherwise, so an empty bag is a subset of
every collection. It sends `remove` with each member of the argument, in the
order of [the list rule](r:class.bag.operand.list), to the copy of the
receiver, and answers what `isEmpty` of the copy answers. The receiver and the
argument are not changed.

```rexx
a = .bag~of('a', 'a', 'b')
say a~subset(.bag~of('a', 'a', 'b', 'c')) a~subset(.bag~of('a', 'b')),
  a~subset(a~copy)
say .bag~new~subset(a) .bag~new~subset(.array~new),
  a~subset(.array~of('b', 'a', 'a'))
say a~subset(.set~of('a', 'b')) a~subset(.list~of('a', 'b', 'a', 'q')) a~items
```

```output
1 0 1
1 1 1
0 1 3
```

## The methods of the mixins

r[class.bag.inherited]
The instance methods `disjoint` and `equivalent` are not declared by `Bag`;
they are those of [`Collection`](collection.md). `disjoint` sends
`intersection` to the receiver and answers what `isEmpty` of that answer says
([the disjoint rule](r:class.collection.disjoint)), so it asks what [the
intersection rule](r:class.bag.intersection) selects; `equivalent` sends
`items` to the receiver and to the argument, then `subset` to the receiver
([the equivalent rule](r:class.collection.equivalent)), so it compares bags by
[the subset rule](r:class.bag.subset), and two bags are equivalent when they
hold the same members equally often. The class method `of` is declared by
`Bag` itself ([the of rule](r:class.bag.of)), and `putAll` by [its own
rule](r:class.bag.putall).

```rexx
a = .many~of('a', 'a', 'b')
say a~disjoint(.bag~of('x')) a~disjoint(.bag~of('x', 'b'))
say a~equivalent(.bag~of('b', 'a', 'a')) a~equivalent(.bag~of('a', 'b'))
say a~equivalent(.bag~of('a', 'b', 'b')),
  a~equivalent(.array~of('a', 'a', 'b'))
::class many subclass bag
```

```output
1 0
1 0
0 1
```

## Messages the methods send

r[class.bag.sends]
`put` and `[]=` send `==` to the item with the index as argument when an index
is given, `hashCode` to the item and nothing to the receiver. The other methods
that look a member up, `at`, `[]`, `hasIndex`, `hasItem`, `index`, `remove`,
`removeItem`, `allAt`, `removeAll`, `items` and `supplier` with an index, send
`hashCode` and `==` to the argument and nothing to the receiver ([the lookup
rule](r:class.bag.members.sends)); `allIndex` sends `==` to the argument ([the
allAt rule](r:class.bag.allat)). `putAll` sends `put` to the receiver for each
member it reads ([the putAll rule](r:class.bag.putall)). The set operations send
to the receiver, the argument, the copy and the new bag the messages that [their
rule](r:class.bag.setops.sends) lists. `allIndexes`, `allItems`, `makeArray`,
`supplier`, `copy`, `items`, `isEmpty` and `empty` send nothing to the members.
No other method asks an override of `at`, `put` or any of the methods, and `[]`
and `[]=` do not use `at` and `put`. In the first example a subclass has every
method overridden to record its name, and each call records only its own; in the
second the members record every message a program is likely to override.

```rexx
own = '[] []= allAt allIndex allIndexes allItems at empty hasIndex hasItem',
  'index isEmpty items makeArray put remove removeAll removeItem supplier',
  'uniqueIndexes copy string'
.local~log = .array~new
.local~on = 0
do m over own~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"'); forward class (super)"
  .spy~define(m, src)
end
a = .spy~new
calls = 'x = a[1]|a["x"] = "x"|x = a~allAt("k1")|x = a~allIndex("q")',
  '|x = a~allIndexes|x = a~allItems|x = a~at("k1")|x = a~hasIndex("k1")',
  '|x = a~hasItem("q")|x = a~index("q")|x = a~isEmpty|x = a~items',
  '|x = a~items("k1")|x = a~makeArray|a~put(1)|a~put(1, 1)',
  '|x = a~remove("k9")',
  '|x = a~removeAll("k9")|x = a~removeItem("q")|x = a~supplier',
  '|x = a~supplier("k1")|x = a~uniqueIndexes|x = a~copy|x = a~string|a~empty'
n = 0
extra = 0
do c over calls~makeArray('|')
  .local~log~empty
  .local~on = 1
  interpret c
  .local~on = 0
  n += 1
  if .local~log~items <> 1 then do
    extra += 1
    say c '=>' .local~log~makeString('L', ' ')
  end
end
say n 'calls,' extra 'sent more than one message to the receiver'
::class spy subclass bag
```

```output
25 calls, 0 sent more than one message to the receiver
```

```rexx
.local~log = .array~new
ms = 'hashCode == = request makeString string objectName defaultName'
do m over ms~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"'); forward class (super)"
  .rec~define(m, src)
end
.local~on = 0
t = .bag~new
do i = 1 to 3
  t~put(.rec~new(i))
end
.local~on = 1
a = t~allIndexes
a = t~allItems
a = t~makeArray
a = t~supplier
c = t~copy
a = t~items
a = t~isEmpty
say .local~log~items
t~empty
say .local~log~items
::class rec
::method init
  expose n
  use arg n
```

```output
0
0
```

r[class.bag.setops.sends]
The set operations send these messages, in this order, to the receiver, to the
argument, to the object that `copy` of the receiver answers (the copy) and to
the new bag that `new` with no argument, sent to the class of the receiver,
answers. The argument is read first as [the operand rule](r:class.bag.operand)
says: "reading" below stands for the `isA` and the `allItems` or `makeArray`
that follows it, and "for each member" means once for each position that [the
list rule](r:class.bag.operand.list) reads. `union` sends `copy` to the
receiver, reads, and for each member sends `put(member)` to the copy; it
answers the copy. `difference` sends `copy` to the receiver, reads, and for
each member sends `remove(member)` to the copy; it answers the copy.
`intersection` sends `new` to the class, `copy` to the receiver, reads, and
for each member sends `hasIndex(member)` to the copy and, when that answers
true, `put(member)` to the new bag and `remove(member)` to the copy; it
answers the new bag. `xor` sends `copy` to the receiver, `new` to the class,
reads, and for each member sends `hasIndex(member)` to the copy and then
either `remove(member)` to the copy, when that answered true, or `put(member)`
to the new bag; then it sends `putAll` to the copy with the new bag as the
argument, whose answer it does not use, and answers the copy; the `putAll` of
`Bag` then sends `isA` and `allItems` to the new bag and `put` to the copy for
each member ([its rule](r:class.bag.putall)). `subset` sends `copy` to the
receiver, reads, sends `remove(member)` to the copy for each member and then
`isEmpty` to the copy; it answers what `isEmpty` answered. Nothing else is
sent to the receiver, the argument or either bag. In the example the receiver
is `R`, the argument `A`, the copy `C` and the new bag `W`; the last field is
the identity of the answer, `-` for none of them.

```rexx
.local~log = .array~new
.local~on = 0
own = 'put hasIndex remove isEmpty putAll allItems isA'
do m over own~makeArray(' ')
  src = "if .local~on then .local~log~append(who(self)'"m"');"
  src = src "forward class (super)"
  .spy~define(m, src)
end
cp = "r = self~copy:super; .local~cp = r;"
cp = cp "if .local~on then .local~log~append('copy'); return r"
.spy~define('copy', cp)
a = .spy~new
a~put('A1'); a~put('B1'); a~put('B1')
b = .spy~new
b~put('B1'); b~put('C')
.local~ra = a
.local~rb = b
do op over 'union difference intersection xor subset'~makeArray(' ')
  .local~log~empty
  .local~on = 1
  x = a~send(op, b)
  .local~on = 0
  line = op'=>'
  do msg over .local~log
    if line~length + msg~length > 66 then do
      say line
      line = ' '
    end
    line = line msg
  end
  say line '|' who(x)
end
::routine who public
  use arg o
  if o == .local~ra then return 'R.'
  if o == .local~rb then return 'A.'
  if o == .local~cp then return 'C.'
  if o == .local~nw then return 'W.'
  return '-'
::class spy subclass bag
::method new class
  r = self~new:super
  .local~nw = r
  if .local~on then .local~log~append('class.new' arg())
  return r
```

```output
union=> copy A.isA A.allItems C.put C.put | C.
difference=> copy A.isA A.allItems C.remove C.remove | C.
intersection=> class.new 0 copy A.isA A.allItems C.hasIndex W.put
  C.remove C.hasIndex | W.
xor=> copy class.new 0 A.isA A.allItems C.hasIndex C.remove
  C.hasIndex W.put C.putAll W.isA W.allItems C.put | C.
subset=> copy A.isA A.allItems C.remove C.remove C.isEmpty | -
```

## Deviations

r[class.bag.deviation.33]
The order of the answers of `allIndexes`, `allItems`, `makeArray`, `allAt`,
`allIndex`, `removeAll`, `uniqueIndexes` and `supplier`, which of several equal
members `at`, `index`, `remove` and `removeItem` choose, and which stored
objects a lookup or a search compares, by `==` or `hashCode`, beyond those it
must compare, and how often, and which members `uniqueIndexes` sends `hashCode`
or `==`, are not specified, and may differ from ooRexx's. Which members are
produced, and how often, is specified. In the example the answers are sorted.

```rexx
b = .bag~of(1, 2, 2, 3)
say b~allIndexes~sort~makeString('L', ',')
say b~allItems~sort~makeString('L', ',')
say b~uniqueIndexes~sort~makeString('L', ',')
say b~items b~makeArray~items
```

```output
1,2,2,3
1,2,2,3
1,2,3
4 4
```

r[class.bag.deviation.45]
`hasItem(item, index)` with an index that is not equal to the item answers 0.
ooRexx answers `.nil`. See [Deviation 45](../deviations.md).

```rexx
say .bag~of(1, 1, 1)~hasItem(1, 2) .bag~of('a')~hasItem('a', 'A')
```

```output defect=#172
0 0
```

r[class.bag.deviation.78]
A lookup, a `put` or a `remove` with the very object that is stored sends that
object's `==` as for any other argument, and so does a search with the very
object that is stored by `hasItem`, `index`, `allIndex` or `removeItem`, so an
object whose `==` answers false is not found even at its own entry. A `put` or
`[]=` that names an index sends `==` to the item with the index as argument
also when the two are the same object. ooRexx compares identity first, finds
the object and sends nothing but `hashCode` for the lookup, and nothing for
the index. See [Deviation 78](../deviations.md).

```rexx
.local~n = 0
b = .bag~new
q = .thing~new
b~put(q)
say b~hasItem(q) .local~n
say (b~removeItem(q) == .nil) .local~n b~items
.local~n = 0
say b~at(q) .local~n
.local~n = 0
x = .loud~new
.bag~new~put(x, x)
say .local~n
::class thing
::method '=='
  .local~n += 1
  return 0
::method hashCode
  return 1
::class loud
::method '=='
  .local~n += 1
  return 1
```

```output
0 1
1 2 1
The NIL object 1
1
```

r[class.bag.deviation.84]
When the `==` that `remove` or `removeItem` sends to its argument, or that
`removeAll` or `allAt` sends, empties the bag, the search ends and `items`
answers 0 afterwards; `remove` and `removeItem` answer `.nil`. ooRexx's `items`
answers -1 after `remove` and `removeItem`, and its `removeAll` and `allAt` go
on comparing with an object that is no stored member. See [Deviation
84](../deviations.md).

```rexx
b = .bag~new
a = .c~new('a')
b~put(a)
.local~b = b
say b~remove(.c~new('a'))
say b~items
::class c
::method init
  expose n
  use arg n
::method n
  expose n
  return n
::method '=='
  expose n
  use arg o
  .local~b~empty
  return n == o~n
::method hashCode
  expose n
  return n~hashCode
```

```output
The NIL object
0
```

r[class.bag.deviation.109]
The size argument of `new` that is an instance of a subclass of `String` is
read by its own characters, and `REQUEST` and `MAKESTRING` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an instance
`request` with `STRING` or `MAKESTRING`, so an override of either changes the
size that `new` checks. In the example `MAKESTRING` answers another text.

```rexx
say .bag~new(.sub~new('2'))~items
::class sub subclass string
::method makestring
  return 'other'
```

```output
0
```

r[class.bag.deviation.125]
An answer of `allItems` or `makeArray` that is a `List` gives its items in list
order, as [the list rule](r:class.bag.operand.list) says, for `putAll`,
`union`, `intersection`, `xor`, `subset` and `difference`. ooRexx reads it by
the positions 1 up to its number of items, so in the example an item is missed
and `.nil` is a member.

```rexx
b = .bag~new
b~putAll(.pk~new(.list~of('p', 'q', 'r')))
say b~items b~hasItem('p') b~hasItem('q') b~hasItem('r') b~hasItem(.nil)
say .bag~of('q', 'r')~subset(.pk~new(.list~of('q', 'r')))
d = .bag~of('p', 'q', 'r', 'z')~difference(.pk~new(.list~of('p', 'q', 'r')))
say d~items
::class pk inherit collection
::method init
  expose answer
  use arg answer
::method allitems
  expose answer
  return answer
```

```output defect=#187
3 1 1 1 0
1
1
```
