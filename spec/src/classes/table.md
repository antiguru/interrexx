---
element: "Table"
kind: class
rexxref: "Builtin Classes > Collection Classes > Table Class; Builtin Classes > Collection Classes > Table Class > new (Class Method); Builtin Classes > Collection Classes > Table Class > []; Builtin Classes > Collection Classes > Table Class > []=; Builtin Classes > Collection Classes > Table Class > allIndexes; Builtin Classes > Collection Classes > Table Class > allItems; Builtin Classes > Collection Classes > Table Class > at; Builtin Classes > Collection Classes > Table Class > empty; Builtin Classes > Collection Classes > Table Class > hasIndex; Builtin Classes > Collection Classes > Table Class > hasItem; Builtin Classes > Collection Classes > Table Class > index; Builtin Classes > Collection Classes > Table Class > isEmpty; Builtin Classes > Collection Classes > Table Class > items; Builtin Classes > Collection Classes > Table Class > makeArray; Builtin Classes > Collection Classes > Table Class > put; Builtin Classes > Collection Classes > Table Class > remove; Builtin Classes > Collection Classes > Table Class > removeItem; Builtin Classes > Collection Classes > Table Class > supplier"
deviations: [33, 78, 84, 109]
---

# Table

A `Table` maps indexes to items. Both are any objects: an index is found by
the `hashCode` and the `==` that it answers to, so two indexes are one entry
when they hash alike and compare equal, and a table holds at most one item for
each index. The class inherits [`MapCollection`](mapcollection.md), and
through it [`Collection`](collection.md); [the hierarchy
rule](r:class.index.hierarchy) lists what it inherits.
[`Directory`](directory.md) and [`StringTable`](stringtable.md) are tables
whose indexes are strings, [`IdentityTable`](identitytable.md) is the table
whose indexes are compared by identity, and [`Relation`](relation.md) is the
table that keeps several items for an index.

A table has no `unknown` method: a message that [the method
search](r:class.methods.unknown) does not resolve raises
[NOMETHOD](r:cond.nomethod.raise), as the error rule of [the
chapter](r:class.index.errors) says. The page follows the order of the rexxref
entry. Examples that show an error use a small routine `try`, as the builtin
pages do. The page says nothing about which of two items equal to the argument
`removeItem` removes ([the hash order rule](r:class.table.deviation.33)).

## The class

r[class.table.creation]
`Table` declares the instance methods `[]`, `[]=`, `allIndexes`, `allItems`,
`at`, `empty`, `hasIndex`, `hasItem`, `index`, `isEmpty`, `items`,
`makeArray`, `put`, `remove`, `removeItem` and `supplier`, the `init` that
`new` runs ([the new rule](r:class.table.new)) and the class method `new`; the
`copy` of `Object` copies a table as [the copy rule](r:class.table.copy) says.
The class method `of` and the instance methods `putAll`, `union`,
`difference`, `intersection`, `xor`, `subset`, `disjoint` and `equivalent` are
those of the mixins ([one rule](r:class.table.inherited) lists them). A table
has no `makeString`, so it is not a string ([the string
rule](r:class.table.string)), and no `unknown` and no `allAt`.

```rexx
t = .table~new
own = '[] []= ALLINDEXES ALLITEMS AT EMPTY HASINDEX HASITEM INDEX ISEMPTY',
  'ITEMS MAKEARRAY PUT REMOVE REMOVEITEM SUPPLIER'
n = 0
do m over own~makearray(' ')
  if t~hasMethod(m) then n += 1
end
say t~class~id n .table~hasMethod('OF') t~hasMethod('OF')
say t~hasMethod('UNION') t~hasMethod('PUTALL') t~hasMethod('APPENDALL')
say t~hasMethod('MAKESTRING') t~hasMethod('UNKNOWN') t~hasMethod('ALLAT')
```

```output
Table 16 1 0
1 1 0
0 0 0
```

## Making a table

r[class.table.new]
`new` makes an empty table. Its optional argument is a hint for how large the
table will grow; no method that this page specifies answers differently for
it. It must be a whole number that is not negative, or `SYNTAX 93.923` is
raised. A second argument raises `SYNTAX 93.902`, and omitted arguments at the
end count for nothing. `new` sends `init` to the new object with the
arguments it received, so a subclass's `init` sees them ([the new
rule](r:class.index.new)). The checks are those of the `init` that `Table`
supplies: they come after the `init` of a subclass has run, and a subclass
whose `init` does not pass the arguments on is not refused for them.

```rexx
say .table~new~items .table~new(0)~items .table~new(3, )~items
call try '.table~new(-1)'
call try '.table~new("x")'
call try '.table~new(.nil)'
call try '.table~new(1.5)'
call try '.table~new(1, 2)'
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
::class watch subclass table
::method init
  say 'init' arg()
  forward class (super)
::class quiet subclass table
::method init
  return
```

```output
0 0 0
.table~new(-1) => 93.923
.table~new("x") => 93.923
.table~new(.nil) => 93.923
.table~new(1.5) => 93.923
.table~new(1, 2) => 93.902
init 1
init 1
.watch~new("x") => 93.923
0
```

## Indexes

r[class.table.indexes]
An index is any object, `.nil` and the null string included, and is stored as
it is: `allIndexes` answers the very objects that were stored. Two indexes are
one entry when [the lookup](r:class.table.indexes.sends) finds that `hashCode`
answers the same string for both and that `==`, sent to one with the other as
argument, answers true. For an object that declares neither method that is the
same object, and an `Array` is its own index only. A string is the index of
its characters, compared by [the strict comparison](r:expr.comparison.strict):
case counts, a leading or trailing blank counts, and `5` and `5.0` are two
indexes. An instance of a subclass of `String` that declares neither method is
the index of its characters like any string. A class that declares `==` has to
declare `hashCode` so that equal objects hash alike, as [the hash code
rule](r:class.object.hashcode) says. When a `put` finds the index already in
the table, the entry keeps the index object it has and takes the new item.

```rexx
t = .table~new
t~put('v1', 'Key')
t~put('v2', 'KEY')
t~put('v3', 5)
t~put('v4', 5.0)
t~put('v5', ' a')
t~put('v6', '')
t~put('v7', .nil)
a = .array~of(1)
t~put('arr', a)
say t~items t~at('Key') t~at('key') t~at(5) t~at('5.0') t~at('a') t~at('')
say t~at(.nil) t~at(a) (t~at(.array~of(1)) == .nil) t~at(.sub~new('Key'))
t~put('v8', .eq~new('x', 'first'))
t~put('v9', .eq~new('x', 'second'))
say t~items t~at(.eq~new('x', 'probe')) t~allIndexes~items
ix = t~allIndexes
do i over ix
  if i~isA(.eq) then say i~tag
end
::class sub subclass string
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
8 v1 The NIL object v3 v4 The NIL object v6
v7 arr 1 v1
9 v9 9
first
```

r[class.table.indexes.sends]
`at`, `[]`, `hasIndex`, `put`, `[]=` and `remove` look the index up by sending
`hashCode` to the index argument, once, and then `==` to the index argument
with a stored index as the argument, for stored indexes that have the same hash
code, until one answers true; each of those is compared before the lookup gives
up. Which further stored indexes are compared, by `==` or `hashCode`, and how
often, is not specified ([the hash order rule](r:class.table.deviation.33));
when the table is empty, the only message is `hashCode`. The very object that
is stored is no exception to the `==` ([the deviation
rule](r:class.table.deviation.78)). In the example every index hashes to the
same string, so each stored index is compared.

```rexx
.local~log = .array~new
say 'empty:' .table~new~at(.k~new('p')) .local~log~makeString('L', ' | ')
calls = 't~at(.k~new("p"))|t[.k~new("p")]|t~hasIndex(.k~new("p"))',
  '|t~put("v2", .k~new("p"))|t[.k~new("p")] = "v2"',
  '|t~put("v3", .k~new("s1"))|t~remove(.k~new("s1"))'
do c over calls~makeArray('|')
  t = .table~new
  t~put('v1', .k~new('s1'))
  .local~log~empty
  interpret c
  say c '=>' .local~log~makeString('L', ' | ')
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
t~at(.k~new("p")) => hash p | == p s1
t[.k~new("p")] => hash p | == p s1
t~hasIndex(.k~new("p"))  => hash p | == p s1
t~put("v2", .k~new("p")) => hash p | == p s1
t[.k~new("p")] = "v2"  => hash p | == p s1
t~put("v3", .k~new("s1")) => hash s1 | == s1 s1
t~remove(.k~new("s1")) => hash s1 | == s1 s1
```

## Reading and writing entries

r[class.table.at]
`at(index)` and `[index]` answer the item stored at the index, and `.nil` when
there is none. The methods take exactly one argument: none, or an omitted one,
raises `SYNTAX 88.901`, and a second raises `SYNTAX 93.902`; the number of
arguments is checked before the index is looked up. An error that the
`hashCode` of the index raises is the error of the message.

```rexx
t = .table~new
t~put('item', 'k')
say t~at('k') t['k'] t~at('K') (t~at('zz') == .nil) (t~at(.nil) == .nil)
call try 't~at'
call try 't~at(, "x")'
call try 't~at("a", "b")'
call try 't[]'
call try 't~at(.hostile~new)'
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
t~at => 88.901
t~at(, "x") => 93.902
t~at("a", "b") => 93.902
t[] => 88.901
t~at(.hostile~new) => 93.900
```

r[class.table.put]
`put(item, index)` and `[index] = item` store the item at the index, replacing
the item that was there, and answer nothing. The item may be any object,
`.nil` included. The methods take exactly two arguments: a missing or omitted
one raises `SYNTAX 88.901`, and a third raises `SYNTAX 93.902`.

```rexx
t = .table~new
t~put('a', 'k')
t['k'] = 'b'
t~put(.nil, 'n')
t~put('c', .nil)
say t~at('k') t~hasIndex('n') (t~at('n') == .nil) t~at(.nil) t~items
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

r[class.table.hasindex]
`hasIndex(index)` answers 1 when an item has that index and 0 otherwise. It
takes exactly one argument: none, or an omitted one, raises `SYNTAX 88.901`,
and a second raises `SYNTAX 93.902`.

```rexx
t = .table~new
t~put(.nil, 'k')
say t~hasIndex('k') t~hasIndex('K') t~hasIndex('zz') t~hasIndex(.nil)
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

r[class.table.hasitem]
`hasItem(item)` answers 1 when some item is equal to the argument and 0
otherwise, and `index(item)` answers an index of such an item and `.nil` when
there is none. Which index `index` answers when several items qualify is not
specified ([the hash order rule](r:class.table.deviation.33)). Both take
exactly one argument: none, or an omitted one, raises `SYNTAX 93.903`, and a
second raises `SYNTAX 93.902`. `.nil` is an item like any other.

```rexx
t = .table~new
t~put('x', 'k1')
t~put(.nil, 'k2')
say t~hasItem('x') t~hasItem('X') t~hasItem(.nil) t~hasItem('zz')
say t~index('x') t~index(.nil) (t~index('q') == .nil)
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

r[class.table.hasitem.sends]
`hasItem`, `index` and `removeItem` send `==` to the argument with an item as
the argument, item by item, until one answers true, and send nothing else,
`hashCode` included. The items are visited in an order that is not specified;
every item is compared at least once when none matches, and how often `==` is
sent with the same item is not specified ([the hash order
rule](r:class.table.deviation.33)). The search with the very object that is
stored is no exception: its `==` is sent as for any other item ([the deviation
rule](r:class.table.deviation.78)).

```rexx
.local~n = 0
t = .table~new
t~put('s', 'k1')
t~put('t', 'k2')
t~put('u', 'k3')
q = .thing~new
.local~h = 0
say t~hasItem(q) (.local~n >= 3) .local~h
.local~n = 0
say (t~index(q) == .nil) (.local~n >= 3) .local~h
.local~n = 0
say (t~removeItem(q) == .nil) (.local~n >= 3) .local~h t~items
::class thing
::method '=='
  .local~n += 1
  return 0
::method hashCode
  .local~h += 1
  return 1
```

```output
0 1 0
1 1 0
1 1 0 3
```

r[class.table.items]
`items` answers the number of entries, and `isEmpty` answers 1 when there is
none and 0 otherwise. Neither takes an argument: one raises `SYNTAX 93.902`.
Two indexes that are one entry count once.

```rexx
t = .table~new
say t~items t~isEmpty
t~put('a', 'k')
t~put(.nil, 'n')
t~put('b', 'k')
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

r[class.table.remove]
`remove(index)` removes the entry with that index and answers its item, and
answers `.nil` when there is none. It takes exactly one argument: none, or an
omitted one, raises `SYNTAX 88.901`, and a second raises `SYNTAX 93.902`.

```rexx
t = .table~new
t~put('v', 'k')
t~put('w', .nil)
say t~remove('k') (t~remove('k') == .nil) t~items
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

r[class.table.removeitem]
`removeItem(item)` removes one entry whose item is equal to the argument, by
the search of [the search rule](r:class.table.hasitem.sends), and answers the
item that was stored, or `.nil` when no item matches. It removes one entry
only. It takes exactly one argument: none, or an omitted one, raises `SYNTAX
93.903`, and a second raises `SYNTAX 93.902`.

```rexx
t = .table~new
t~put('one', 'k1')
t~put('one', 'k2')
t~put('two', 'k3')
say t~removeItem('one') t~items t~removeItem('two') t~items
say (t~removeItem('zz') == .nil) t~items
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
one 2 two 1
1 1
t~removeItem => 93.903
t~removeItem(1, 2) => 93.902
```

r[class.table.empty]
`empty` removes every entry and answers the table itself. It takes no
argument: one raises `SYNTAX 93.902`.

```rexx
t = .table~new
t~put('v', 'k')
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

r[class.table.makearray]
`allIndexes` and `makeArray` answer a new `Array` of the indexes of the
entries, and `allItems` a new `Array` of their items. They are instances of
`Array` for a subclass receiver too, never the receiver, and do not follow
later changes. They take no argument: one raises `SYNTAX 93.902`. The order is
not specified ([the hash order rule](r:class.table.deviation.33)), and is the
same for the three answers and for the supplier of one unchanged table, so the
item at a position belongs to the index at the same position. The methods send
no message to the table or to the indexes and items.

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
::class many subclass table
```

```output
Array Array Array 40 40 0
40 41 0
t~allIndexes(1) => 93.902
t~allItems(1) => 93.902
t~makeArray(1) => 93.902
```

r[class.table.supplier]
`supplier` answers a `Supplier` over the entries, the index and the item of
each. It is a snapshot: later changes do not reach it. Its order is that of
`allIndexes` ([the order rule](r:class.table.makearray)). An empty table
gives a supplier that is not available. It takes no argument: one raises
`SYNTAX 93.902`.

```rexx
t = .table~new
t~put('a', 'k')
s = t~supplier
t~put('b', 'k2')
t~remove('k')
n = 0
do while s~available
  say s~index s~item
  n += 1
  s~next
end
say n .table~new~supplier~available
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

r[class.table.copy]
`copy` answers a new table of the class of the receiver with the same entries.
The indexes and the items are the same objects, and the two tables do not see
each other's changes. As [the copy rule](r:class.object.copy) says of every
object, the variables of a subclass instance are copied as they are, and
`init` is not sent. It takes no argument: one raises `SYNTAX 93.902`.

```rexx
t = .many~new
t~tag = 'T'
t~put('v1', 'k1')
a = .array~new
t~put(a, 'arr')
c = t~copy
say c~class~id c~items c~tag c~at('k1') (c == t) (c~at('arr') == a)
c~put('v2', 'k2')
c~remove('k1')
say t~items c~items t~at('k1') (c~at('k1') == .nil)
call try 't~copy(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass table
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

r[class.table.string]
A table has no `makeString` method, so its string value is the one of
`Object`: its name, `a Table`, as [the string rule](r:class.index.string)
says, whatever the table holds, and `request('STRING')` answers `.nil`.

```rexx
t = .table~new
t~put('x', 'k')
say t '|' t~string '|' t 'x' t~hasMethod('MAKESTRING')
say (t~request('STRING') == .nil) .table~new
```

```output
a Table | a Table | a Table x 0
1 a Table
```

## The methods of the mixins

r[class.table.inherited]
The class method `of` and the instance methods `putAll`, `union`,
`difference`, `intersection`, `xor`, `subset`, `disjoint` and `equivalent` are
not declared by `Table`. `of` and `putAll` are those of
[`MapCollection`](mapcollection.md), and the others those of
[`Collection`](collection.md); a table is one of the collections they take as
an argument, and the answers of the first four are instances of the class of
the receiver. `putAll` stores with the `put` of the receiver. In the example
the arguments are tables of pairs of an index and an item.

```rexx
a = .many~of(.array~of('a', 'A1'), .array~of('b', 'B1'))
b = .table~of(.array~of('b', 'B2'), .array~of('c', 'C2'))
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
::class many subclass table
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

r[class.table.sends]
No method that `Table` declares sends a message to its receiver: an override
of `at`, `put` or any other of the methods is never asked by another method,
and `[]` and `[]=` do not use `at` and `put`. The messages the methods do send
go to other objects and have a rule each: `init` to the new object by `new`
([the new rule](r:class.table.new)); `hashCode` and `==` to the index argument
by `at`, `[]`, `hasIndex`, `put`, `[]=` and `remove` ([the lookup
rule](r:class.table.indexes.sends)); `==` to the argument by `hasItem`,
`index` and `removeItem` ([the search rule](r:class.table.hasitem.sends)).
`makeArray`, `allIndexes`, `allItems`, `supplier`, `copy`, `items`,
`isEmpty` and `empty` send nothing to the indexes and the items. The set
operations send what the pages of the mixins say. In the first example a
subclass has every method overridden to record its name, and each call
records only its own; in the second the indexes and items record every message
a program is likely to override.

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
::class spy subclass table
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
t = .table~new
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

r[class.table.deviation.33]
The order of the answers of `allIndexes`, `allItems`, `makeArray` and
`supplier`, of the search of `hasItem`, `index` and `removeItem`, and which
stored indexes or items a lookup or a search compares, by `==` or `hashCode`,
beyond those it must compare, and how often, are not specified, and may differ
from ooRexx's. Which entries are produced, and how often, is specified. In the
example the answers are sorted.

```rexx
t = .table~new
t~put(1, 'one')
t~put(2, 'two')
t~put(3, 'three')
say t~allIndexes~sort~makeString('L', ',')
say t~allItems~sort~makeString('L', ',')
say t~items t~makeArray~items
```

```output
one,three,two
1,2,3
3 3
```

r[class.table.deviation.78]
A lookup, a `put` or a `remove` with the very object that is stored as the
index sends that object's `==` as for any other index, and a search for an
item with the very object that is stored, by `hasItem`, `index` or
`removeItem`, sends that object's `==` as for any other item, so an object
whose `==` answers false is not found even at its own entry. ooRexx compares
identity first, finds it and sends nothing but `hashCode`. See [Deviation
78](../deviations.md).

```rexx
.local~n = 0
t = .table~new
q = .thing~new
t~put(q, 'k')
say t~hasItem(q) .local~n
say (t~index(q) == .nil) .local~n
say (t~removeItem(q) == .nil) .local~n t~items
t~put('v', q)
.local~n = 0
say t~at(q) .local~n
::class thing
::method '=='
  .local~n += 1
  return 0
::method hashCode
  return 1
```

```output
0 1
1 2
1 3 1
The NIL object 1
```

r[class.table.deviation.84]
When the `==` that `remove` sends to an index, or that `removeItem` sends to
its argument, empties the table, `items` answers 0 afterwards and the method
answers `.nil`. ooRexx's `items` answers -1. See [Deviation
84](../deviations.md).

```rexx
t = .table~new
a = .c~new('a')
t~put('v', a)
.local~t = t
say t~remove(.c~new('a'))
say t~items
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
  .local~t~empty
  return n == o~n
::method hashCode
  expose n
  return n~hashCode
```

```output
The NIL object
0
```

```rexx
c = .table~new
.local~victim = c
c~put(.em~new, 'a')
say (c~removeItem(.em~new) == .nil) c~items
::class em
::method '=='
  .victim~empty
  return 1
::method hashCode
  return 'h'
```

```output
1 0
```

r[class.table.deviation.109]
The size argument of `new` that is an instance of a subclass of `String` is
read by its own characters, and `REQUEST` and `MAKESTRING` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an
instance `request` with `STRING` or `MAKESTRING`, so an override of either
changes the size that `new` checks. In the example `MAKESTRING` answers
another text.

```rexx
say .table~new(.sub~new('2'))~items
::class sub subclass string
::method makestring
  return 'other'
```

```output
0
```
