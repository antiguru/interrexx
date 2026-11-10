---
element: "IdentityTable"
kind: class
rexxref: "Builtin Classes > Collection Classes > IdentityTable Class; Builtin Classes > Collection Classes > IdentityTable Class > new (Class Method); Builtin Classes > Collection Classes > IdentityTable Class > []; Builtin Classes > Collection Classes > IdentityTable Class > []=; Builtin Classes > Collection Classes > IdentityTable Class > allIndexes; Builtin Classes > Collection Classes > IdentityTable Class > allItems; Builtin Classes > Collection Classes > IdentityTable Class > at; Builtin Classes > Collection Classes > IdentityTable Class > empty; Builtin Classes > Collection Classes > IdentityTable Class > hasIndex; Builtin Classes > Collection Classes > IdentityTable Class > hasItem; Builtin Classes > Collection Classes > IdentityTable Class > index; Builtin Classes > Collection Classes > IdentityTable Class > isEmpty; Builtin Classes > Collection Classes > IdentityTable Class > items; Builtin Classes > Collection Classes > IdentityTable Class > makeArray; Builtin Classes > Collection Classes > IdentityTable Class > put; Builtin Classes > Collection Classes > IdentityTable Class > remove; Builtin Classes > Collection Classes > IdentityTable Class > removeItem; Builtin Classes > Collection Classes > IdentityTable Class > supplier"
deviations: [4, 33, 109]
---

# IdentityTable

An `IdentityTable` maps indexes to items, as a [`Table`](table.md) does, but
two indexes are one entry only when they are the very same object: the
`hashCode` and the `==` of an index are never asked. Both indexes and items
are any objects, and an item is found by its identity too. The table holds at
most one item for each index. The class inherits
[`MapCollection`](mapcollection.md), and through it
[`Collection`](collection.md); [the hierarchy rule](r:class.index.hierarchy)
lists what it inherits.

An `IdentityTable` has no `unknown` method: a message that [the method
search](r:class.methods.unknown) does not resolve raises
[NOMETHOD](r:cond.nomethod.raise), as the error rule of [the
chapter](r:class.index.errors) says. The page follows the order of the rexxref
entry. Examples that show an error use a small routine `try`, as the builtin
pages do. Whether two equal strings or numbers are one object is not specified
([the identity rule](r:class.identitytable.deviation.4)), so the examples use
other objects, or one variable read twice, for indexes and items. The page
says nothing about which of two items found by the same object `removeItem`
removes ([the hash order rule](r:class.identitytable.deviation.33)).

## The class

r[class.identitytable.creation]
`IdentityTable` declares the instance methods `[]`, `[]=`, `allIndexes`,
`allItems`, `at`, `empty`, `hasIndex`, `hasItem`, `index`, `isEmpty`, `items`,
`makeArray`, `put`, `remove`, `removeItem` and `supplier`, the `init` that
`new` runs ([the new rule](r:class.identitytable.new)) and the class method
`new`; the `copy` of `Object` copies a table as [the copy
rule](r:class.identitytable.copy) says. The class method `of` and the instance
methods `putAll`, `union`, `difference`, `intersection`, `xor`, `subset`,
`disjoint` and `equivalent` are those of the mixins ([one
rule](r:class.identitytable.inherited) lists them). A table has no
`makeString`, so it is not a string ([the string
rule](r:class.identitytable.string)), and no `unknown` and no `allAt`.

```rexx
t = .identitytable~new
own = '[] []= ALLINDEXES ALLITEMS AT EMPTY HASINDEX HASITEM INDEX ISEMPTY',
  'ITEMS MAKEARRAY PUT REMOVE REMOVEITEM SUPPLIER'
n = 0
do m over own~makearray(' ')
  if t~hasMethod(m) then n += 1
end
say t~class~id n .identitytable~hasMethod('OF') t~hasMethod('OF')
say t~hasMethod('UNION') t~hasMethod('PUTALL') t~hasMethod('APPENDALL')
say t~hasMethod('MAKESTRING') t~hasMethod('UNKNOWN') t~hasMethod('ALLAT')
```

```output
IdentityTable 16 1 0
1 1 0
0 0 0
```

## Making a table

r[class.identitytable.new]
`new` makes an empty table. Its optional argument is a hint for how large the
table will grow; no method that this page specifies answers differently for
it. It must be a whole number that is not negative, or `SYNTAX 93.923` is
raised. A second argument raises `SYNTAX 93.902`, and omitted arguments at the
end count for nothing. `new` sends `init` to the new object with the arguments
it received, so a subclass's `init` sees them ([the new
rule](r:class.index.new)). The checks are those of the `init` that
`IdentityTable` supplies: they come after the `init` of a subclass has run,
and a subclass whose `init` does not pass the arguments on is not refused for
them.

```rexx
say .identitytable~new~items .identitytable~new(0)~items,
  .identitytable~new(3, )~items
call try '.identitytable~new(-1)'
call try '.identitytable~new("x")'
call try '.identitytable~new(.nil)'
call try '.identitytable~new(1.5)'
call try '.identitytable~new(1, 2)'
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
::class watch subclass identitytable
::method init
  say 'init' arg()
  forward class (super)
::class quiet subclass identitytable
::method init
  return
```

```output
0 0 0
.identitytable~new(-1) => 93.923
.identitytable~new("x") => 93.923
.identitytable~new(.nil) => 93.923
.identitytable~new(1.5) => 93.923
.identitytable~new(1, 2) => 93.902
init 1
init 1
.watch~new("x") => 93.923
0
```

## Indexes

r[class.identitytable.indexes]
An index is any object, `.nil` included, and is stored as it is: `allIndexes`
answers the very objects that were stored. Two indexes are one entry when they
are the same object, and in no other case: an object that declares `==` and
`hashCode` to be equal to another is a different index from it, two arrays
with the same items are two indexes, and a string read from the same variable
twice is one index. Whether two strings or numbers with the same characters
that were made separately are one object is not specified ([the identity
rule](r:class.identitytable.deviation.4)). When a `put` finds the index
already in the table, the item is replaced. A lookup sends no message to the
index, `hashCode` and `==` included ([the lookup
rule](r:class.identitytable.indexes.sends)).

```rexx
t = .identitytable~new
s = 'Key'
a = .array~of(1)
b = .array~of(1)
o1 = .eq~new('x')
o2 = .eq~new('x')
t~put('v1', s)
t~put('v2', a)
t~put('v3', b)
t~put('v4', .nil)
t~put('v5', o1)
t~put('v6', o2)
s2 = s
say t~items t~at(s) t~at(s2) t~at(a) t~at(b) t~at(.nil)
say (t~at(.array~of(1)) == .nil) t~at(o1) t~at(o2),
  (t~at(.eq~new('x')) == .nil)
t~put('v7', a)
say t~items t~at(a) t~hasIndex(b) t~hasIndex(.array~of(1))
say (o1 == o2) (o1~hashCode == o2~hashCode)
::class eq
::method init
  expose n
  use arg n
::method n
  expose n
  return n
::method '=='
  expose n
  use arg o
  return n == o~n
::method hashCode
  expose n
  return n~hashCode
```

```output
6 v1 v1 v2 v3 v4
1 v5 v6 1
6 v7 1 0
1 1
```

r[class.identitytable.indexes.sends]
`at`, `[]`, `hasIndex`, `put`, `[]=` and `remove` send no message to the index
argument and to the stored indexes: not `hashCode`, not `==`, and nothing
else. The same holds for `hasItem`, `index` and `removeItem` and the items
([the search rule](r:class.identitytable.hasitem)). In the example every
object of the class records each message it is sent.

```rexx
.local~log = .array~new
t = .identitytable~new
p = .k~new('p')
s = .k~new('s1')
t~put('v1', s)
calls = 'x = t~at(p)|x = t[p]|x = t~hasIndex(p)|t~put("v2", p)',
  '|t[p] = "v3"|x = t~remove(p)|x = t~remove(s)|t~put(s, s)',
  '|x = t~hasItem(s)|x = t~index(s)|x = t~removeItem(p)|x = t~removeItem(s)'
do c over calls~makeArray('|')
  .local~log~empty
  interpret c
  say c '=>' .local~log~items
end
exit
::class k
::method init
  expose tag
  use arg tag
::method '=='
  .local~log~append('==')
  return 0
::method hashCode
  .local~log~append('hash')
  return 1
::method request
  .local~log~append('request')
  return .nil
```

```output
x = t~at(p) => 0
x = t[p] => 0
x = t~hasIndex(p) => 0
t~put("v2", p)  => 0
t[p] = "v3" => 0
x = t~remove(p) => 0
x = t~remove(s) => 0
t~put(s, s)  => 0
x = t~hasItem(s) => 0
x = t~index(s) => 0
x = t~removeItem(p) => 0
x = t~removeItem(s) => 0
```

## Reading and writing entries

r[class.identitytable.at]
`at(index)` and `[index]` answer the item stored at the index, and `.nil` when
there is none. The methods take exactly one argument: none, or an omitted one,
raises `SYNTAX 88.901`, and a second raises `SYNTAX 93.902`; the number of
arguments is checked before the index is looked up.

```rexx
t = .identitytable~new
k = 'k'
t~put('item', k)
say t~at(k) t[k] t~at('K') (t~at('zz') == .nil) (t~at(.nil) == .nil)
call try 't~at'
call try 't~at(, "x")'
call try 't~at("a", "b")'
call try 't[]'
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
item item The NIL object 1 1
t~at => 88.901
t~at(, "x") => 93.902
t~at("a", "b") => 93.902
t[] => 88.901
```

r[class.identitytable.put]
`put(item, index)` and `[index] = item` store the item at the index, replacing
the item that was there, and answer nothing. The item may be any object,
`.nil` included. The methods take exactly two arguments: a missing or omitted
one raises `SYNTAX 88.901`, and a third raises `SYNTAX 93.902`.

```rexx
t = .identitytable~new
k = 'k'
n = 'n'
t~put('a', k)
t[k] = 'b'
t~put(.nil, n)
t~put('c', .nil)
say t~at(k) t~hasIndex(n) (t~at(n) == .nil) t~at(.nil) t~items
call try 't~put("x")'
call try 't~put(, "x")'
call try 't~put("x", )'
call try 't~put("x", "y", "z")'
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
b 1 1 c 3
t~put("x") => 88.901
t~put(, "x") => 88.901
t~put("x", ) => 88.901
t~put("x", "y", "z") => 93.902
```

## Questions about the contents

r[class.identitytable.hasindex]
`hasIndex(index)` answers 1 when an item has that index and 0 otherwise. It
takes exactly one argument: none, or an omitted one, raises `SYNTAX 88.901`,
and a second raises `SYNTAX 93.902`.

```rexx
t = .identitytable~new
k = 'k'
t~put(.nil, k)
say t~hasIndex(k) t~hasIndex('K') t~hasIndex('zz') t~hasIndex(.nil)
call try 't~hasIndex'
call try 't~hasIndex("a", "b")'
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
1 0 0 0
t~hasIndex => 88.901
t~hasIndex("a", "b") => 93.902
```

r[class.identitytable.hasitem]
`hasItem(item)` answers 1 when some entry holds the argument itself as its
item and 0 otherwise, and `index(item)` answers the index of such an entry and
`.nil` when there is none. An object that is equal to an item under `==` but
is not that object is not found, and no message is sent to the argument or to
the items. Which index `index` answers when several entries hold the item is
not specified ([the hash order rule](r:class.identitytable.deviation.33)).
Both take exactly one argument: none, or an omitted one, raises
`SYNTAX 93.903`, and a second raises `SYNTAX 93.902`. `.nil` is an item like
any other.

```rexx
t = .identitytable~new
x = .array~of('x')
t~put(x, 'k1')
t~put(.nil, 'k2')
say t~hasItem(x) t~hasItem(.array~of('x')) t~hasItem(.nil) t~hasItem('zz')
say t~index(x) t~index(.nil) (t~index(.array~of('x')) == .nil)
call try 't~hasItem'
call try 't~index'
call try 't~hasItem(, 1)'
call try 't~index(1, 2)'
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
k1 k2 1
t~hasItem => 93.903
t~index => 93.903
t~hasItem(, 1) => 93.902
t~index(1, 2) => 93.902
```

r[class.identitytable.items]
`items` answers the number of entries, and `isEmpty` answers 1 when there is
none and 0 otherwise. Neither takes an argument: one raises `SYNTAX 93.902`.
The same index put twice counts once.

```rexx
t = .identitytable~new
say t~items t~isEmpty
k = 'k'
t~put('a', k)
t~put(.nil, 'n')
t~put('b', k)
say t~items t~isEmpty
call try 't~items(1)'
call try 't~isEmpty(1)'
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
t~items(1) => 93.902
t~isEmpty(1) => 93.902
```

## Removing entries

r[class.identitytable.remove]
`remove(index)` removes the entry with that index and answers its item, and
answers `.nil` when there is none. It takes exactly one argument: none, or an
omitted one, raises `SYNTAX 88.901`, and a second raises `SYNTAX 93.902`.

```rexx
t = .identitytable~new
k = 'k'
t~put('v', k)
t~put('w', .nil)
say t~remove(k) (t~remove(k) == .nil) t~items
say t~remove(.nil) (t~remove(.nil) == .nil) t~items
call try 't~remove'
call try 't~remove("a", "b")'
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
w 1 0
t~remove => 88.901
t~remove("a", "b") => 93.902
```

r[class.identitytable.removeitem]
`removeItem(item)` removes one entry whose item is the argument itself, as
found by [the search rule](r:class.identitytable.hasitem), and answers that
item, or `.nil` when no entry holds it. It removes one entry only. It takes
exactly one argument: none, or an omitted one, raises `SYNTAX 93.903`, and a
second raises `SYNTAX 93.902`.

```rexx
t = .identitytable~new
one = .array~of('one')
two = .array~of('two')
t~put(one, 'k1')
t~put(one, 'k2')
t~put(two, 'k3')
say (t~removeItem(one) == one) t~items (t~removeItem(two) == two) t~items
say (t~removeItem(.array~of('one')) == .nil) t~items
call try 't~removeItem'
call try 't~removeItem(1, 2)'
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
1 2 1 1
1 1
t~removeItem => 93.903
t~removeItem(1, 2) => 93.902
```

r[class.identitytable.empty]
`empty` removes every entry and answers the table itself. It takes no
argument: one raises `SYNTAX 93.902`.

```rexx
t = .identitytable~new
t~put('v', .object~new)
e = t~empty
say (e == t) t~items t~isEmpty
call try 't~empty(1)'
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
t~empty(1) => 93.902
```

## Lists of the entries

r[class.identitytable.makearray]
`allIndexes` and `makeArray` answer a new `Array` of the indexes of the
entries, and `allItems` a new `Array` of their items. They are instances of
`Array` for a subclass receiver too, never the receiver, and do not follow
later changes. They take no argument: one raises `SYNTAX 93.902`. The order is
not specified ([the hash order rule](r:class.identitytable.deviation.33)), and
is the same for the three answers and for the supplier of one unchanged table,
so the item at a position belongs to the index at the same position. The
methods send no message to the table or to the indexes and items.

```rexx
t = .many~new
do i = 1 to 40
  t~put('item'i, .array~of(i))
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
t~put('x', 'extra')
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
::class many subclass identitytable
```

```output
Array Array Array 40 40 0
40 41 0
t~allIndexes(1) => 93.902
t~allItems(1) => 93.902
t~makeArray(1) => 93.902
```

r[class.identitytable.supplier]
`supplier` answers a `Supplier` over the entries, the index and the item of
each. It is a snapshot: later changes do not reach it. Its order is that of
`allIndexes` ([the order rule](r:class.identitytable.makearray)). An empty
table gives a supplier that is not available. It takes no argument: one raises
`SYNTAX 93.902`.

```rexx
t = .identitytable~new
k = 'k'
t~put('a', k)
s = t~supplier
t~put('b', 'k2')
t~remove(k)
n = 0
do while s~available
  say s~index s~item
  n += 1
  s~next
end
say n .identitytable~new~supplier~available
call try 't~supplier(1)'
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
k a
1 0
t~supplier(1) => 93.902
```

## Copies and the string value

r[class.identitytable.copy]
`copy` answers a new table of the class of the receiver with the same entries.
The indexes and the items are the same objects, and the two tables do not see
each other's changes. As [the copy rule](r:class.object.copy) says of every
object, the variables of a subclass instance are copied as they are, and
`init` is not sent. It takes no argument: one raises `SYNTAX 93.902`.

```rexx
t = .many~new
t~tag = 'T'
k = 'k1'
t~put('v1', k)
a = .array~new
t~put(a, 'arr')
c = t~copy
say c~class~id c~items c~tag c~at(k) (c == t) (c~at('arr') == a)
c~put('v2', 'k2')
c~remove(k)
say t~items c~items t~at(k) (c~at(k) == .nil)
call try 't~copy(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass identitytable
::attribute tag
::method init
  say 'init'
  forward class (super)
```

```output
init
MANY 2 T v1 0 1
2 2 v1 1
t~copy(1) => 93.902
```

r[class.identitytable.string]
A table has no `makeString` method, so its string value is the one of
`Object`: its name, `an IdentityTable`, as [the string
rule](r:class.index.string) says, whatever the table holds, and
`request('STRING')` answers `.nil`.

```rexx
t = .identitytable~new
t~put('x', 'k')
say t '|' t~string '|' t 'x' t~hasMethod('MAKESTRING')
say (t~request('STRING') == .nil) .identitytable~new
```

```output
an IdentityTable | an IdentityTable | an IdentityTable x 0
1 an IdentityTable
```

## The methods of the mixins

r[class.identitytable.inherited]
The class method `of` and the instance methods `putAll`, `union`,
`difference`, `intersection`, `xor`, `subset`, `disjoint` and `equivalent` are
not declared by `IdentityTable`. `of` and `putAll` are those of
[`MapCollection`](mapcollection.md), and the others those of
[`Collection`](collection.md); a table is one of the collections they take as
an argument, and the answers of the first four are instances of the class of
the receiver. They compare indexes as the table does, by identity, so the
indexes that the arguments share are shared objects. `putAll` stores with the
`put` of the receiver. In the example the arguments are tables of pairs of an
index and an item.

```rexx
ka = 'a'
kb = 'b'
kc = 'c'
a = .many~of(.array~of(ka, 'A1'), .array~of(kb, 'B1'))
b = .identitytable~of(.array~of(kb, 'B2'), .array~of(kc, 'C2'))
do op over 'union difference intersection xor'~makeArray(' ')
  r = a~send(op, b)
  say op r~class~id r~items listed(r)
end
say a~subset(b) a~disjoint(b) a~equivalent(b) a~subset(a~copy)
c = .many~new
c~putAll(b)
say c~items listed(c)
exit
listed: procedure
  use arg r
  s = ''
  do i over r~allIndexes~sort
    s = s i'='r[i]
  end
  return s~strip
::class many subclass identitytable
```

```output
union MANY 3 a=A1 b=B1 c=C2
difference MANY 1 a=A1
intersection MANY 1 b=B2
xor MANY 2 a=A1 c=C2
0 0 0 1
2 b=B2 c=C2
```

## Messages the methods send

r[class.identitytable.sends]
No method that `IdentityTable` declares sends a message to its receiver: an
override of `at`, `put` or any other of the methods is never asked by another
method, and `[]` and `[]=` do not use `at` and `put`. The messages the methods
do send go to other objects and have a rule: `init` to the new object by `new`
([the new rule](r:class.identitytable.new)). Neither the lookups by index nor
the searches by item send anything to the indexes, the items or the arguments
([the lookup rule](r:class.identitytable.indexes.sends)), and `makeArray`,
`allIndexes`, `allItems`, `supplier`, `copy`, `items`, `isEmpty` and `empty`
send nothing to the indexes and the items. The set operations send what the
pages of the mixins say. In the first example a subclass has every method
overridden to record its name, and each call records only its own; in the
second the indexes and items record every message a program is likely to
override.

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
calls = 'x = a[1]|a["k1"] = "x"|a~allIndexes|a~allItems|a~at("k1")',
  '|a~hasIndex("k1")|a~hasItem("q")|a~index("q")|a~isEmpty|a~items',
  '|a~makeArray|a~put(1, "k9")|a~remove("k9")|a~removeItem("q")',
  '|a~supplier|a~copy|a~string|a~empty'
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
::class spy subclass identitytable
```

```output
18 calls, 0 sent more than one message to the receiver
```

```rexx
.local~log = .array~new
ms = 'hashCode == = request makeString string objectName defaultName'
do m over ms~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"'); forward class (super)"
  .rec~define(m, src)
end
.local~on = 0
t = .identitytable~new
do i = 1 to 3
  t~put(.rec~new(i), .rec~new(-i))
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

## Deviations

r[class.identitytable.deviation.4]
Whether two strings or numbers with the same characters, produced separately,
are one object is not specified, so whether a lookup of one finds the entry
stored with the other is not specified either ([Deviation
4](r:class.object.deviation.4)). A value read from the same variable is one
object. In the example the two literals are either one index or two.

```rexx
t = .identitytable~new
t~put(1, 'abc')
t~put(2, 'abc')
say (t~items == 1) | (t~items == 2)
s = 'abc'
u = s
t~put(3, s)
say t~at(u)
```

```output
1
3
```

r[class.identitytable.deviation.33]
The order of the answers of `allIndexes`, `allItems`, `makeArray` and
`supplier`, and which of two entries that hold the same item `index` and
`removeItem` use, is not specified, and may differ from ooRexx's. Which
entries are produced, and how often, is specified. In the example the answers
are sorted.

```rexx
t = .identitytable~new
t~put(1, .array~of('one'))
t~put(2, .array~of('two'))
t~put(3, .array~of('three'))
say t~allItems~sort~makeString('L', ',')
say t~allIndexes~items t~items t~makeArray~items
```

```output
1,2,3
3 3 3
```

r[class.identitytable.deviation.109]
The size argument of `new` that is an instance of a subclass of `String` is
read by its own characters, and `REQUEST` and `MAKESTRING` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an
instance `request` with `STRING` or `MAKESTRING`, so an override of either
changes the size that `new` checks. In the example `MAKESTRING` answers
another text.

```rexx
say .identitytable~new(.sub~new('2'))~items
::class sub subclass string
::method makestring
  return 'other'
```

```output
0
```
