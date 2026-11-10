---
element: "Set"
kind: class
rexxref: "Builtin Classes > Collection Classes > Set Class; Builtin Classes > Collection Classes > Set Class > new (Class Method); Builtin Classes > Collection Classes > Set Class > of (Class Method); Builtin Classes > Collection Classes > Set Class > []; Builtin Classes > Collection Classes > Set Class > []=; Builtin Classes > Collection Classes > Set Class > allIndexes; Builtin Classes > Collection Classes > Set Class > allItems; Builtin Classes > Collection Classes > Set Class > at; Builtin Classes > Collection Classes > Set Class > empty; Builtin Classes > Collection Classes > Set Class > hasIndex; Builtin Classes > Collection Classes > Set Class > hasItem; Builtin Classes > Collection Classes > Set Class > index; Builtin Classes > Collection Classes > Set Class > intersection; Builtin Classes > Collection Classes > Set Class > isEmpty; Builtin Classes > Collection Classes > Set Class > items; Builtin Classes > Collection Classes > Set Class > makeArray; Builtin Classes > Collection Classes > Set Class > put; Builtin Classes > Collection Classes > Set Class > putAll; Builtin Classes > Collection Classes > Set Class > remove; Builtin Classes > Collection Classes > Set Class > removeItem; Builtin Classes > Collection Classes > Set Class > subset; Builtin Classes > Collection Classes > Set Class > supplier; Builtin Classes > Collection Classes > Set Class > union; Builtin Classes > Collection Classes > Set Class > xor"
deviations: [33, 78, 84, 109]
---

# Set

A `Set` holds members, each an object that is its own index: `put` stores an
item and the item is the index it is found by, so a member occurs once however
often it is put. Members are found as a [table](table.md) finds its indexes,
by the `hashCode` and the `==` that they answer to. The class inherits
[`MapCollection`](mapcollection.md) and [`SetCollection`](setcollection.md),
and through them [`Collection`](collection.md); [the hierarchy
rule](r:class.index.hierarchy) lists what it inherits. Unlike a table it
declares its own set operations, which read their argument as a list of
members and not as a list of entries.

A set has no `unknown` method: a message that [the method
search](r:class.methods.unknown) does not resolve raises
[NOMETHOD](r:cond.nomethod.raise), as the error rule of [the
chapter](r:class.index.errors) says. Examples that show an error use a small
routine `try`, as the builtin pages do. Where several members qualify for a
method that answers one, which one it answers is not specified, and the examples
then hold one member or sort the answers ([the hash order
rule](r:class.set.deviation.33)).

## The class

r[class.set.creation]
`Set` declares the instance methods `[]`, `[]=`, `allIndexes`, `allItems`,
`at`, `empty`, `hasIndex`, `hasItem`, `index`, `intersection`, `isEmpty`,
`items`, `makeArray`, `put`, `putAll`, `remove`, `removeItem`, `subset`,
`supplier`, `union` and `xor`, the `init` that `new` runs ([the new
rule](r:class.set.new)) and the class methods `new` and `of`; the `copy` of
`Object` copies a set as [the copy rule](r:class.set.copy) says. The instance
methods `difference`, `disjoint` and `equivalent` are those of
[`Collection`](collection.md) ([one rule](r:class.set.inherited) covers them).
A set has no `makeString`, so it is not a string ([the string
rule](r:class.set.string)), and no `unknown`, `allAt`, `allIndex`, `removeAll`
or `uniqueIndexes`. Its superclasses are `Object`, `MapCollection` and
`SetCollection`, in that order.

```rexx
s = .set~new
n = 0
m = .set~methods(.nil)
do while m~available
  n += 1
  m~next
end
say s~class~id n 'own methods' .set~hasMethod('OF') s~hasMethod('OF')
do c over s~class~superclasses
  say c~id
end
say s~hasMethod('UNION') s~hasMethod('DIFFERENCE') s~hasMethod('DISJOINT')
say s~hasMethod('MAKESTRING') s~hasMethod('UNKNOWN') s~hasMethod('REMOVEALL')
say s~hasMethod('ALLAT') s~hasMethod('ALLINDEX') s~hasMethod('UNIQUEINDEXES')
```

```output
Set 22 own methods 1 0
Object
MapCollection
SetCollection
1 1 1
0 0 0
0 0 0
```

## Making a set

r[class.set.new]
`new` makes an empty set. Its optional argument is a hint for how large the
set will grow; no method that this page specifies answers differently for it.
It must be a whole number that is not negative, or `SYNTAX 93.923` is raised.
A second argument raises `SYNTAX 93.902`, and omitted arguments at the end
count for nothing. `new` sends `init` to the new object with the arguments it
received, so a subclass's `init` sees them ([the new
rule](r:class.index.new)). The checks are those of the `init` that `Set`
supplies: they come after the `init` of a subclass has run, and a subclass
whose `init` does not pass the arguments on is not refused for them.

```rexx
say .set~new~items .set~new(0)~items .set~new(3, )~items
call try '.set~new(-1)'
call try '.set~new("x")'
call try '.set~new(.nil)'
call try '.set~new(1.5)'
call try '.set~new(1, 2)'
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
::class watch subclass set
::method init
  say 'init' arg()
  forward class (super)
::class quiet subclass set
::method init
  return
```

```output
0 0 0
.set~new(-1) => 93.923
.set~new("x") => 93.923
.set~new(.nil) => 93.923
.set~new(1.5) => 93.923
.set~new(1, 2) => 93.902
init 1
init 1
.watch~new("x") => 93.923
0
```

r[class.set.of]
`of(item, ...)` answers a new set of the class that received it that holds the
arguments, each member once, and with no argument an empty set. An argument
that is omitted before the last one raises `SYNTAX 93.903`, and a trailing
omitted argument counts for nothing. The method does not send `new` to the
class and does not send `put` to the set: it sends `init` with no argument to
the new object and stores the arguments itself, so an override of `new` or
`put` in a subclass is not used. Two arguments are one member when they are
equal as [members are](r:class.set.members), so `1` and `1.0` are two.

```rexx
s = .set~of('a', 'b', 'a', 1, 1.0, .nil)
say s~items s~hasItem('a') s~hasItem(1.0) s~hasItem(.nil) .set~of~items
x = .spy~of('p', 'q', 'p')
say x~class~id x~items
call try '.set~of(1, , 2)'
call try '.set~of(, 1)'
say .set~of(1, )~items
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class spy subclass set
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
5 1 1 1 0
init 0
SPY 2
.set~of(1, , 2) => 93.903
.set~of(, 1) => 93.903
1
```

## Members

r[class.set.members]
A member is any object, `.nil` and the null string included, and the set
stores the object that was put. A member is found by [the
lookup](r:class.set.members.sends): `hashCode` answers the same string for the
argument and for the stored object, and `==`, sent to the argument with the
stored object as argument, answers true. For an object that declares neither
method that is the same object, and an `Array` is its own member only. A
string is a member of its characters, compared by [the strict
comparison](r:expr.comparison.strict): case counts, a leading or trailing
blank counts, and `5` and `5.0` are two members. A class that declares `==`
has to declare `hashCode` so that equal objects hash alike, as [the hash code
rule](r:class.object.hashcode) says. When `put` finds an equal member, the set
keeps the object it has as the index of the entry, and takes the new object as
its item: `at` and `allItems` then answer the new object, and `allIndexes` and
`makeArray` the old one.

```rexx
s = .set~new
do x over .array~of('Key', 'KEY', 5, 5.0, ' a', '', .nil)
  s~put(x)
end
a = .array~of(1)
s~put(a)
say s~items s~hasItem('Key') s~hasItem('key') s~hasItem('5.0') s~hasItem('a')
say s~hasItem(a) s~hasItem(.array~of(1)) s~hasItem('') s~hasItem(.nil)
s~put(.eq~new('x', 'first'))
s~put(.eq~new('x', 'second'))
say s~items s~at(.eq~new('x', 'probe'))~tag
say s~allItems~items s~allIndexes~items
do i over s~allIndexes
  if i~isA(.eq) then say 'index' i~tag
end
do i over s~allItems
  if i~isA(.eq) then say 'item' i~tag
end
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
8 1 0 1 0
1 0 1 1
9 second
9 9
index first
item second
```

r[class.set.members.sends]
`at`, `[]`, `hasIndex`, `hasItem`, `index`, `put`, `[]=`, `remove` and
`removeItem` look a member up by sending `hashCode` to the argument, once, and
then `==` to the argument with a stored object as the argument, for stored
objects that have the same hash code, until one answers true; each of those is
compared before the lookup gives up. Which further stored objects are compared,
by `==` or `hashCode`, and how often, is not specified ([the hash order
rule](r:class.set.deviation.33)); when the set is empty, the only message is
`hashCode`. The very object that is stored is no exception to the `==` ([the
deviation rule](r:class.set.deviation.78)). `put` and `[]=` with an index first
send `==` to the item with the index as argument ([the put
rule](r:class.set.put)). In the example every member hashes to the same string,
so each stored member is compared.

```rexx
.local~log = .array~new
say 'empty:' .set~new~at(.k~new('p')) .local~log~makeString('L', ' | ')
calls = 's~at(.k~new("p"))|s[.k~new("p")]|s~hasIndex(.k~new("p"))',
  '|s~hasItem(.k~new("p"))|s~put(.k~new("p"))|s[.k~new("p")] = .k~new("p")',
  '|s~remove(.k~new("p"))|s~removeItem(.k~new("p"))',
  '|s~put(.k~new("s1"))|s~remove(.k~new("s1"))'
do c over calls~makeArray('|')
  s = .set~new
  s~put(.k~new('s1'))
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
s~at(.k~new("p")) => hash p | == p s1
s[.k~new("p")] => hash p | == p s1
s~hasIndex(.k~new("p")) => hash p | == p s1
s~hasItem(.k~new("p")) => hash p | == p s1
s~put(.k~new("p")) => hash p | == p s1
s[.k~new("p")] = .k~new("p") => == p p | hash p | == p s1
s~remove(.k~new("p")) => hash p | == p s1
s~removeItem(.k~new("p")) => hash p | == p s1
s~put(.k~new("s1")) => hash s1 | == s1 s1
s~remove(.k~new("s1")) => hash s1 | == s1 s1
```

## Adding and reading members

r[class.set.put]
`put(item)` and `put(item, index)` add the item and answer nothing;
`[index] = item` is the same as `put(item, index)`, and `[] = item` the same
as `put(item)`. An index may be given only as another name for the item: when
one is, and the item is not the very object, `==` is sent to the item with the
index as argument first, and an answer that is not true raises `SYNTAX 93.949`
and leaves the set unchanged. An index that is omitted counts for nothing. The
methods take one or two arguments: none, or an omitted item, raises
`SYNTAX 88.901`, and a third argument raises `SYNTAX 93.902`. The item may be
any object, `.nil` included. The check of the index is made before the item is
looked up ([the lookup rule](r:class.set.members.sends)).

```rexx
s = .set~new
s~put('a')
s~put('a', 'a')
s['b'] = 'b'
s[] = 'c'
s~put(.nil)
s~put('d', )
say s~items s~hasItem('a') s~hasItem('d') s~hasItem(.nil) s~hasItem('e')
call try 's~put("a", "A")'
call try 's~put(1, 2)'
call try 's["x"] = "y"'
call try 's~put'
call try 's~put(, "a")'
call try 's~put("a", "a", "a")'
say s~items
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
5 1 1 1 0
s~put("a", "A") => 93.949
s~put(1, 2) => 93.949
s["x"] = "y" => 93.949
s~put => 88.901
s~put(, "a") => 88.901
s~put("a", "a", "a") => 93.902
5
```

r[class.set.at]
`at(index)` and `[index]` answer the stored item that is equal to the index,
and `.nil` when there is none. The methods take exactly one argument: none, or
an omitted one, raises `SYNTAX 88.901`, and a second raises `SYNTAX 93.902`;
the number of arguments is checked before the index is looked up. An error
that the `hashCode` of the index raises is the error of the message.

```rexx
s = .set~new
s~put('item')
say s~at('item') s['item'] s~at('ITEM') (s~at('zz') == .nil),
  (s~at(.nil) == .nil)
call try 's~at'
call try 's~at(, "x")'
call try 's~at("a", "b")'
call try 's[]'
call try 's~at(.hostile~new)'
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
s~at => 88.901
s~at(, "x") => 93.902
s~at("a", "b") => 93.902
s[] => 88.901
s~at(.hostile~new) => 93.900
```

## Questions about the contents

r[class.set.hasindex]
`hasIndex(index)` answers 1 when a member is equal to the argument and 0
otherwise. It takes exactly one argument: none, or an omitted one, raises
`SYNTAX 88.901`, and a second raises `SYNTAX 93.902`.

```rexx
s = .set~of('k', .nil)
say s~hasIndex('k') s~hasIndex('K') s~hasIndex('zz') s~hasIndex(.nil)
call try 's~hasIndex'
call try 's~hasIndex("a", "b")'
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
s~hasIndex => 88.901
s~hasIndex("a", "b") => 93.902
```

r[class.set.hasitem]
`hasItem(item)` answers 1 when a member is equal to the item and 0 otherwise;
for a set it is the same question as `hasIndex`. `index(item)` looks the item up
as an index, as [the lookup rule](r:class.set.members.sends) says, and answers
the item of the entry it finds, which is the object put last for that member
([the members rule](r:class.set.members)), and `.nil` when there is none. Both
take exactly one argument: for `hasItem` none, or an omitted one, raises
`SYNTAX 88.901`; for `index` it raises `SYNTAX 93.903`; a second argument raises
`SYNTAX 93.902` for both, and an argument omitted at the end counts for nothing.

```rexx
s = .set~of('x', .nil)
say s~hasItem('x') s~hasItem('X') s~hasItem(.nil) s~hasItem('zz')
say s~index('x') (s~index('q') == .nil) s~hasItem('x', )
call try 's~hasItem'
call try 's~index'
call try 's~hasItem(, 1)'
call try 's~index(1, 2)'
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
x 1 1
s~hasItem => 88.901
s~index => 93.903
s~hasItem(, 1) => 93.902
s~index(1, 2) => 93.902
```

```rexx
.local~log = .array~new
s = .set~new
s~put(.k~new('first'))
s~put(.k~new('second'))
.local~log~empty
say s~index(.k~new('probe'))~tag '|' .local~log~makeString('L', ' | ')
.local~log~empty
say (s~index(.k~new('miss', 'other')) == .nil) '|',
  .local~log~makeString('L', ' | ')
::class k
::method init
  expose tag key
  use arg tag, key = 'same'
::method tag
  expose tag
  return tag
::method key
  expose key
  return key
::method '=='
  expose tag key
  use arg o
  .local~log~append('==' tag o~tag)
  return key == o~key
::method hashCode
  expose tag key
  .local~log~append('hash' tag)
  return key
```

```output defect=#169
second | hash probe | == probe first
1 | hash miss
```

r[class.set.items]
`items` answers the number of members, and `isEmpty` answers 1 when there is
none and 0 otherwise. Neither takes an argument: one raises `SYNTAX 93.902`.
Members that are equal count once.

```rexx
s = .set~new
say s~items s~isEmpty
s~put('a')
s~put(.nil)
s~put('a')
say s~items s~isEmpty
call try 's~items(1)'
call try 's~isEmpty(1)'
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
2 0
s~items(1) => 93.902
s~isEmpty(1) => 93.902
```

## Removing members

r[class.set.remove]
`remove(index)` removes the member that is equal to the argument and answers
the item that was stored, and `.nil` when there is none. It takes exactly one
argument: none, or an omitted one, raises `SYNTAX 88.901`, and a second raises
`SYNTAX 93.902`.

```rexx
s = .set~of('v', .nil)
say s~remove('v') (s~remove('v') == .nil) s~items
say (s~remove(.nil) == .nil) (s~remove(.nil) == .nil) s~items
call try 's~remove'
call try 's~remove("a", "b")'
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
v 1 1
1 1 0
s~remove => 88.901
s~remove("a", "b") => 93.902
```

r[class.set.removeitem]
`removeItem(item)` removes the member that is equal to the argument and
answers the item that was stored, and `.nil` when there is none; for a set it
does what `remove` does. It takes exactly one argument: none, or an omitted
one, raises `SYNTAX 88.901`, and a second raises `SYNTAX 93.902`.

```rexx
s = .set~of('one', 'two')
say s~removeItem('one') s~items (s~removeItem('one') == .nil) s~items
say (s~removeItem('zz') == .nil) s~removeItem('two') s~items
call try 's~removeItem'
call try 's~removeItem(1, 2)'
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
one 1 1 1
1 two 0
s~removeItem => 88.901
s~removeItem(1, 2) => 93.902
```

r[class.set.empty]
`empty` removes every member and answers the set itself. It takes no argument:
one raises `SYNTAX 93.902`.

```rexx
s = .set~of('v')
e = s~empty
say (e == s) s~items s~isEmpty
call try 's~empty(1)'
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
s~empty(1) => 93.902
```

## Lists of the members

r[class.set.makearray]
`allIndexes` and `makeArray` answer a new `Array` of the indexes of the
entries, and `allItems` a new `Array` of their items. They are instances of
`Array` for a subclass receiver too, never the receiver, and do not follow
later changes. They take no argument: one raises `SYNTAX 93.902`. The order is
not specified ([the hash order rule](r:class.set.deviation.33)), and is the
same for the three answers and for the supplier of one unchanged set, so the
item at a position belongs to the index at the same position. The methods send
no message to the set or to the members.

```rexx
t = .many~new
do i = 1 to 40
  t~put('item'i)
end
ix = t~allIndexes
it = t~allItems
bad = 0
do i = 1 to ix~items
  if t~at(ix[i]) \== it[i] then bad += 1
  if t~makeArray[i] \== ix[i] then bad += 1
end
say ix~class~id it~class~id t~makeArray~class~id ix~items it~items bad
a = t~allItems
t~put('extra')
say a~items t~items (ix == t~allIndexes)
call try 't~allIndexes(1)'
call try 't~allItems(1)'
call try 't~makeArray(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass set
```

```output
Array Array Array 40 40 0
40 41 0
t~allIndexes(1) => 93.902
t~allItems(1) => 93.902
t~makeArray(1) => 93.902
```

r[class.set.supplier]
`supplier` answers a `Supplier` over the entries, the index and the item of
each. It is a snapshot: later changes do not reach it. Its order is that of
`allIndexes` ([the order rule](r:class.set.makearray)). An empty set gives a
supplier that is not available. It takes no argument: one raises
`SYNTAX 93.902`.

```rexx
s = .set~new
s~put('a')
p = s~supplier
s~put('b')
s~remove('a')
n = 0
do while p~available
  say p~index p~item
  n += 1
  p~next
end
say n .set~new~supplier~available
call try 's~supplier(1)'
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
a a
1 0
s~supplier(1) => 93.902
```

## Copies and the string value

r[class.set.copy]
`copy` answers a new set of the class of the receiver with the same entries.
The indexes and the items are the same objects, and the two sets do not see
each other's changes. As [the copy rule](r:class.object.copy) says of every
object, the variables of a subclass instance are copied as they are, and
`init` is not sent. It takes no argument: one raises `SYNTAX 93.902`.

```rexx
t = .many~new
t~tag = 'T'
t~put('v1')
a = .array~new
t~put(a)
c = t~copy
say c~class~id c~items c~tag c~hasItem('v1') (c == t) c~hasItem(a)
c~put('v2')
c~remove('v1')
say t~items c~items t~hasItem('v1') c~hasItem('v1')
call try 't~copy(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass set
::attribute tag
::method init
  say 'init'
  forward class (super)
```

```output
init
MANY 2 T 1 0 1
2 2 1 0
t~copy(1) => 93.902
```

r[class.set.string]
A set has no `makeString` method, so its string value is the one of `Object`:
its name, `a Set`, as [the string rule](r:class.index.string) says, whatever
the set holds, and `request('STRING')` answers `.nil`.

```rexx
s = .set~of('x')
say s '|' s~string '|' s 'x' s~hasMethod('MAKESTRING')
say (s~request('STRING') == .nil) .set~new
```

```output
a Set | a Set | a Set x 0
1 a Set
```

## Adding the members of a collection

r[class.set.putall]
`putAll(collection)` adds the members of the argument and answers the set. It
takes exactly one argument: none raises `SYNTAX 93.901` and a second raises
`SYNTAX 93.902`. The argument is read as a list of members ([the operand
rule](r:class.set.operand)), and each is added by sending `put` with it, as
the only argument, to the set, so an override of `put` in a subclass is used
and members that are equal count once. Unlike
[`MapCollection`](mapcollection.md), which stores the entries of a supplier,
it reads items only: the indexes of a map argument are not added.

```rexx
s = .many~new
r = s~putAll(.array~of('a', 'b', 'a'))
say (r == s) s~items
s~putAll(.directory~new~~put('v', 'k'))
s~putAll(.set~of('z', 'a'))
say s~items s~hasItem('v') s~hasItem('k') s~hasItem('z')
call try 's~putAll'
call try 's~putAll(1, 2)'
call try 's~putAll(.nil)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass set
::method put
  say 'put' arg(1)~class~id arg()
  forward class (super)
```

```output
put String 1
put String 1
put String 1
1 2
put String 1
put String 1
put String 1
4 1 0 1
s~putAll => 93.901
s~putAll(1, 2) => 93.902
s~putAll(.nil) => 93.948
```

## Combining sets

r[class.set.operand]
`putAll`, `union`, `intersection` and `xor` read their argument by sending it
`isA` with the class `Collection`: an answer of 1 means the argument is sent
`allItems`, an answer of 0 means it is sent `makeArray`, and any other answer,
`.nil` among them, raises `SYNTAX 34.1`. An argument that does not understand
the message it is sent, `.nil` among them, raises `SYNTAX 93.948`. `subset`
sends no `isA`: it sends `allItems` to its argument whatever it is.
`difference` is not declared by `Set`; [the operand rule of
`Collection`](r:class.collection.operand) reads its argument. The operations
check the number of arguments first: none raises `SYNTAX 93.901` and a second
raises `SYNTAX 93.902`. What the argument answers is read as [the list
rule](r:class.set.operand.list) says. The example records the messages that
each kind of argument is sent.

```rexx
.local~log = .array~new
do op over 'putAll union intersection xor subset'~makeArray(' ')
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
  x = .set~of('x')~send(op, k)
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
intersection: isA+allItems isA+makeArray 34.1 93.948
xor: isA+allItems isA+makeArray 34.1 93.948
subset: allItems allItems allItems 93.948
```

r[class.set.operand.list]
The answer of `allItems` or `makeArray` is read by sending it `items`, with no
argument, and then `[]` with each position from 1 up to the number that
`items` answered, in that order; each answer of `[]` is a member of the
argument. The answer need not be an `Array`: an object that understands the
two messages will do, and an answer that lacks one raises `SYNTAX 93.948`. A
subclass of `Array` is sent the two messages too, and no other. A position
that holds nothing reads `.nil`, which is then a member, so an `Array` with a
hole is read for as many positions as it has items, not as its size. An answer
that is a `List` is the one kind read otherwise: its members are all its items,
in the order of the list, not the entries at the positions 1 up to its number
of items. The
answer of `items` is read as a number: one that is not a number raises
`SYNTAX 41.1`, and a negative one reads no position. An error that `[]` raises
is the error of the operation, after the members before it have been handled.

```rexx
a = .array~new(5)
a[2] = 'two'
s = .set~new
s~putAll(.pk~new(a))
say s~items s~hasItem('two') s~hasItem(.nil)
s = .set~new
s~putAll(.pk~new(.walk~new(3, 'e')))
say s~items s~allItems~sort~makeString('L', ',')
s = .set~new
s~putAll(.pk~new(.logarr~of('p', 'q')))
say s~allItems~sort~makeString('L', ',')
call try 's~putAll(.pk~new(.walk~new("x", "e")))'
s = .set~new
call try 's~putAll(.pk~new(.walk~new(2, "boom")))'
say s~items s~hasItem('boom1')
call try 's~putAll(.pk~new(.object~new))'
say .set~new~~putAll(.pk~new(.walk~new(-1, 'e')))~items
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
s~putAll(.pk~new(.walk~new("x", "e"))) => 41.1
s~putAll(.pk~new(.walk~new(2, "boom"))) => 93.900
1 1
s~putAll(.pk~new(.object~new)) => 93.948
0
```

r[class.set.union]
`union(other)` answers the object that `copy` of the receiver answers, after
each member of the argument has been added to it: for each member, in the
order of [the list rule](r:class.set.operand.list), `hasIndex` is sent to the
copy with the member and, when that answers false, `put` with the member as
the only argument. The receiver and the argument are not changed, and the copy
is an instance of the class of the receiver.

```rexx
a = .set~of('a', 'b')
b = .set~of('b', 'c')
u = a~union(b)
say u~class~id u~items (u \== a) a~items b~items
say sorted(u) '|' sorted(a~union(.array~of('x', 'x', 'a')))
say sorted(a~union(.directory~new~~put('v', 'k'))),
  sorted(a~union(.bag~of('b', 'b', 'z')))
say sorted(a~union('xy')) sorted(.set~new~union(a))
exit
sorted: procedure
  use arg c
  return c~allItems~sort~makeString('L', ',')
```

```output
Set 3 1 2 2
a,b,c | a,b,x
a,b,v a,b,z
a,b,xy a,b
```

r[class.set.intersection]
`intersection(other)` answers a new set of the class of the receiver, made by
`new` with no argument, that holds each member that the receiver and the
argument have in common. For each member of the argument, in the order of [the
list rule](r:class.set.operand.list), `hasIndex` is sent to the receiver with
the member and, when that answers true, `put` with the member as the only
argument to the new set. No copy is made, and neither the receiver nor the
argument is changed.

```rexx
a = .set~of('a', 'b', 'c')
b = .set~of('b', 'c', 'd')
i = a~intersection(b)
say i~class~id i~items (i \== a) (i \== b) sorted(i)
say sorted(a~intersection(.array~of('c', 'c', 'q', 'a')))
say sorted(a~intersection(.set~new)) '|' a~items b~items
exit
sorted: procedure
  use arg c
  return c~allItems~sort~makeString('L', ',')
```

```output
Set 2 1 1 b,c
a,c
 | 3 3
```

r[class.set.xor]
`xor(other)` answers the object that `copy` of the receiver answers, holding
the members that only one of the receiver and the argument holds. For each
member of the argument, in the order of [the list
rule](r:class.set.operand.list), `hasIndex` is sent to the receiver with the
member and then either `remove` with the member to the copy, when that
answered true, or `put` with the member as the only argument to the copy. The
receiver and the argument are not changed.

```rexx
a = .set~of('a', 'b', 'c')
b = .set~of('b', 'c', 'd')
x = a~xor(b)
say x~class~id x~items (x \== a) sorted(x) '|' a~items b~items
say sorted(a~xor(.array~of('a', 'z', 'z')))
say sorted(a~xor(a)) '|' sorted(a~xor(.set~new))
exit
sorted: procedure
  use arg c
  return c~allItems~sort~makeString('L', ',')
```

```output
Set 2 1 a,d | 3 3
b,c,z
 | a,b,c
```

r[class.set.subset]
`subset(other)` answers 1 when every member of the receiver is a member of the
argument, and 0 otherwise, so an empty set is a subset of every collection. It
sends `copy` to the receiver, then `allItems` to the argument, and sends
`remove` with each member of the argument, in the order of [the list
rule](r:class.set.operand.list), to the copy, and last `isEmpty` to the copy;
the answer is what `isEmpty` answered. The receiver and the argument are not
changed.

```rexx
a = .set~of('a', 'b')
say a~subset(.set~of('a', 'b', 'c')) a~subset(.set~of('a')) a~subset(a~copy)
say .set~new~subset(a) .set~new~subset(.array~new),
  a~subset(.array~of('b', 'a', 'a'))
say a~subset(.list~of('a', 'b', 'q')) a~subset(.bag~of('a')) a~items
call try 'a~subset("ab")'
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
1 0 1
1 1 1
1 0 2
a~subset("ab") => 93.948
```

## The methods of the mixins

r[class.set.inherited]
The instance methods `difference`, `disjoint` and `equivalent` are not
declared by `Set`; they are those of [`Collection`](collection.md).
`difference` answers the object that `copy` of the receiver answers with the
members removed that the argument lists, and is an instance of the class of
the receiver, so what [the difference rule](r:class.collection.difference)
says applies to a set. For a map argument it removes by the indexes of the
argument, not its items. `disjoint` sends `intersection` to the receiver and
answers what `isEmpty` of that answer says ([the disjoint
rule](r:class.collection.disjoint)), so it asks what [the intersection
rule](r:class.set.intersection) selects; `equivalent` sends `items` to the
receiver and to the argument, then `subset` to the receiver ([the equivalent
rule](r:class.collection.equivalent)), so it compares sets by [the subset
rule](r:class.set.subset). The class method `of` is declared by `Set` itself
([the of rule](r:class.set.of)), and `putAll` by [its own
rule](r:class.set.putall).

```rexx
a = .many~of('a', 'b', 'c')
d = a~difference(.set~of('b', 'z'))
say d~class~id d~items sorted(d)
say sorted(a~difference(.array~of('a', 'a'))),
  sorted(a~difference(.bag~of('c')))
say sorted(a~difference(.table~new~~put('a', 'c'))) a~items
say a~disjoint(.set~of('x')) a~disjoint(.set~of('x', 'c'))
say a~equivalent(.set~of('c', 'a', 'b')) a~equivalent(.set~of('a', 'b'))
say a~equivalent(.array~of('a', 'b', 'c'))
exit
sorted: procedure
  use arg c
  return c~allItems~sort~makeString('L', ',')
::class many subclass set
```

```output
MANY 2 a,c
b,c a,b
a,b 3
1 0
1 0
1
```

## Messages the methods send

r[class.set.sends]
`put` and `[]=` send `==` to the item with the index as argument when an index
is given, `hashCode` and `==` to the member that they look up, and nothing to
the receiver. The other methods that look a member up, `at`, `[]`, `hasIndex`,
`hasItem`, `index`, `remove` and `removeItem`, send `hashCode` and `==` to the
argument and nothing to the receiver ([the lookup
rule](r:class.set.members.sends)). `putAll` sends `put` to the receiver for each
member it reads ([the putAll rule](r:class.set.putall)). The set operations send
to the receiver, the argument, the copy and the new set the messages that [their
rule](r:class.set.setops.sends) lists. `makeArray`, `allIndexes`, `allItems`,
`supplier`, `copy`, `items`, `isEmpty` and `empty` send nothing to the members.
No other method asks an override of `at`, `put` or any of the methods, and `[]`
and `[]=` do not use `at` and `put`. In the first example a subclass has every
method overridden to record its name, and each call records only its own; in the
second the members record every message a program is likely to override.

```rexx
own = '[] []= allIndexes allItems at empty hasIndex hasItem index isEmpty',
  'items makeArray put remove removeItem supplier copy string'
.local~log = .array~new
.local~on = 0
do m over own~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"'); forward class (super)"
  .spy~define(m, src)
end
a = .spy~new
calls = 'x = a[1]|a["x"] = "x"|x = a~allIndexes|x = a~allItems',
  '|x = a~at("k1")',
  '|x = a~hasIndex("k1")|x = a~hasItem("q")|x = a~index("q")|x = a~isEmpty',
  '|x = a~items|x = a~makeArray|a~put(1)|a~put(1, 1)|x = a~remove("k9")',
  '|x = a~removeItem("q")|x = a~supplier|x = a~copy|x = a~string|a~empty'
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
::class spy subclass set
```

```output
19 calls, 0 sent more than one message to the receiver
```

```rexx
.local~log = .array~new
ms = 'hashCode == = request makeString string objectName defaultName'
do m over ms~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"'); forward class (super)"
  .rec~define(m, src)
end
.local~on = 0
t = .set~new
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

r[class.set.setops.sends]
The set operations send these messages, in this order, to the receiver, to the
argument, to the object that `copy` of the receiver answers (the copy) and to
the new set that `new` with no argument, sent to the class of the receiver,
answers. The argument is read first as [the operand rule](r:class.set.operand)
says: "reading" below stands for the `isA` that `union`, `intersection` and
`xor` send and for the `allItems` or `makeArray` that follows, and "for each
member" means once for each position that [the list
rule](r:class.set.operand.list) reads. `union` sends `copy` to the receiver,
reads, and for each member sends `hasIndex(member)` to the copy and, when that
answers false, `put(member)` to the copy; it answers the copy. `intersection`
sends `new` to the class, reads, and for each member sends `hasIndex(member)`
to the receiver and, when that answers true, `put(member)` to the new set; it
answers the new set, and no copy is made. `xor` sends `copy` to the receiver,
reads, and for each member sends `hasIndex(member)` to the receiver and then
either `remove(member)` to the copy, when that answered true, or `put(member)`
to the copy; it answers the copy. `subset` sends `copy` to the receiver, sends
`allItems` to the argument, sends `remove(member)` to the copy for each member
and then `isEmpty` to the copy; it answers what `isEmpty` answered. Nothing
else is sent to the receiver, the argument or either set. In the example the
receiver is `R`, the argument `A`, the copy `C` and the new set `W`; the last
field is the identity of the answer, `-` for none of them.

```rexx
.local~log = .array~new
.local~on = 0
own = 'put hasIndex remove isEmpty allItems isA'
do m over own~makeArray(' ')
  src = "if .local~on then .local~log~append(who(self)'"m"');"
  src = src "forward class (super)"
  .spy~define(m, src)
end
cp = "r = self~copy:super; .local~cp = r;"
cp = cp "if .local~on then .local~log~append('copy'); return r"
.spy~define('copy', cp)
a = .spy~new
a~put('A1'); a~put('B1')
b = .spy~new
b~put('B1'); b~put('C')
.local~ra = a
.local~rb = b
do op over 'union intersection xor subset'~makeArray(' ')
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
::class spy subclass set
::method new class
  r = self~new:super
  .local~nw = r
  if .local~on then .local~log~append('class.new' arg())
  return r
```

```output
union=> copy A.isA A.allItems C.hasIndex C.hasIndex C.put | C.
intersection=> class.new 0 A.isA A.allItems R.hasIndex W.put
  R.hasIndex | W.
xor=> copy A.isA A.allItems R.hasIndex C.remove R.hasIndex C.put | C.
subset=> copy A.allItems C.remove C.remove C.isEmpty | -
```

## Deviations

r[class.set.deviation.33]
The order of the answers of `allIndexes`, `allItems`, `makeArray` and
`supplier`, and which stored objects a lookup compares, by `==` or `hashCode`,
beyond those it must compare, and how often, are not specified, and may differ
from ooRexx's. Which members are produced, and how often, is specified. In the
example the answers are sorted.

```rexx
s = .set~of(1, 2, 3)
say s~allIndexes~sort~makeString('L', ',')
say s~allItems~sort~makeString('L', ',')
say s~items s~makeArray~items
```

```output
1,2,3
1,2,3
3 3
```

r[class.set.deviation.78]
A lookup, a `put` or a `remove` with the very object that is stored sends that
object's `==` as for any other argument, and so does a search with the very
object that is stored by `hasItem`, `index` or `removeItem`, so an object
whose `==` answers false is not found even at its own entry. A `put` or `[]=`
that names an index sends `==` to the item with the index as argument also
when the two are the same object. ooRexx compares identity first, finds the
object and sends nothing but `hashCode` for the lookup, and nothing for the
index. See [Deviation 78](../deviations.md).

```rexx
.local~n = 0
s = .set~new
q = .thing~new
s~put(q)
say s~hasItem(q) .local~n
say (s~removeItem(q) == .nil) .local~n s~items
.local~n = 0
say s~at(q) .local~n
.local~n = 0
x = .loud~new
.set~new~put(x, x)
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

r[class.set.deviation.84]
When the `==` that `remove` or `removeItem` sends to its argument empties the
set, the search ends: the method answers `.nil` and `items` answers 0
afterwards. ooRexx answers -1 for `items`. See [Deviation
84](../deviations.md).

```rexx
s = .set~new
a = .c~new('a')
s~put(a)
.local~s = s
say s~remove(.c~new('a'))
say s~items
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
  .local~s~empty
  return n == o~n
::method hashCode
  expose n
  return n~hashCode
```

```output
The NIL object
0
```

r[class.set.deviation.109]
The size argument of `new` that is an instance of a subclass of `String` is
read by its own characters, and `REQUEST` and `MAKESTRING` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an instance
`request` with `STRING` or `MAKESTRING`, so an override of either changes the
size that `new` checks. In the example `MAKESTRING` answers another text.

```rexx
say .set~new(.sub~new('2'))~items
::class sub subclass string
::method makestring
  return 'other'
```

```output
0
```
