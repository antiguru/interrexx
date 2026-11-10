---
element: "Relation"
kind: class
rexxref: "Builtin Classes > Collection Classes > Relation Class; Builtin Classes > Collection Classes > Relation Class > new (Class Method); Builtin Classes > Collection Classes > Relation Class > []; Builtin Classes > Collection Classes > Relation Class > []=; Builtin Classes > Collection Classes > Relation Class > allAt; Builtin Classes > Collection Classes > Relation Class > allIndex; Builtin Classes > Collection Classes > Relation Class > allIndexes; Builtin Classes > Collection Classes > Relation Class > allItems; Builtin Classes > Collection Classes > Relation Class > at; Builtin Classes > Collection Classes > Relation Class > empty; Builtin Classes > Collection Classes > Relation Class > hasIndex; Builtin Classes > Collection Classes > Relation Class > hasItem; Builtin Classes > Collection Classes > Relation Class > index; Builtin Classes > Collection Classes > Relation Class > isEmpty; Builtin Classes > Collection Classes > Relation Class > items; Builtin Classes > Collection Classes > Relation Class > makeArray; Builtin Classes > Collection Classes > Relation Class > put; Builtin Classes > Collection Classes > Relation Class > remove; Builtin Classes > Collection Classes > Relation Class > removeAll; Builtin Classes > Collection Classes > Relation Class > removeItem; Builtin Classes > Collection Classes > Relation Class > supplier; Builtin Classes > Collection Classes > Relation Class > uniqueIndexes"
deviations: [33, 78, 109]
---

# Relation

A `Relation` is a [`Table`](table.md) that keeps every item put at an index:
it holds entries, each a pair of an index and an item, and `put` always adds
an entry, so one index, or one item, can occur in many entries, and the same
pair can occur twice. Indexes are found as a table finds them, by the
`hashCode` and the `==` that they answer to; items are found by `==`. The
class inherits [`MapCollection`](mapcollection.md), and through it
[`Collection`](collection.md); [the hierarchy rule](r:class.index.hierarchy)
lists what it inherits.

A relation has no `unknown` method: a message that [the method
search](r:class.methods.unknown) does not resolve raises
[NOMETHOD](r:cond.nomethod.raise), as the error rule of [the
chapter](r:class.index.errors) says. The page follows the order of the rexxref
entry. Examples that show an error use a small routine `try`, as the builtin
pages do. Where several entries qualify for a method that answers one, which
entry it answers is not specified, and the examples then hold one entry or
sort the answers ([the hash order rule](r:class.relation.deviation.33)).

## The class

r[class.relation.creation]
`Relation` declares the instance methods `[]`, `[]=`, `allAt`, `allIndex`,
`allIndexes`, `allItems`, `at`, `empty`, `hasIndex`, `hasItem`, `index`,
`isEmpty`, `items`, `makeArray`, `put`, `remove`, `removeAll`, `removeItem`,
`supplier` and `uniqueIndexes`, the `init` that `new` runs ([the new
rule](r:class.relation.new)) and the class method `new`; the `copy` of
`Object` copies a relation as [the copy rule](r:class.relation.copy) says. The
class method `of` and the instance methods `putAll`, `union`, `difference`,
`intersection`, `xor`, `subset`, `disjoint` and `equivalent` are those of the
mixins ([one rule](r:class.relation.inherited) lists them). A relation has no
`makeString`, so it is not a string ([the string
rule](r:class.relation.string)).

```rexx
r = .relation~new
own = '[] []= ALLAT ALLINDEX ALLINDEXES ALLITEMS AT EMPTY HASINDEX HASITEM',
  'INDEX ISEMPTY ITEMS MAKEARRAY PUT REMOVE REMOVEALL REMOVEITEM SUPPLIER',
  'UNIQUEINDEXES'
n = 0
do m over own~makearray(' ')
  if r~hasMethod(m) then n += 1
end
say r~class~id n .relation~hasMethod('OF') r~hasMethod('OF')
say r~hasMethod('UNION') r~hasMethod('PUTALL') r~hasMethod('APPENDALL')
say r~hasMethod('MAKESTRING') r~hasMethod('UNKNOWN') r~hasMethod('ENTRY')
```

```output
Relation 20 1 0
1 1 0
0 0 0
```

## Making a relation

r[class.relation.new]
`new` makes an empty relation. Its optional argument is a hint for how large
the relation will grow; no method that this page specifies answers differently
for it. It must be a whole number that is not negative, or `SYNTAX 93.923` is
raised. A second argument raises `SYNTAX 93.902`, and omitted arguments at the
end count for nothing. `new` sends `init` to the new object with the
arguments it received, so a subclass's `init` sees them ([the new
rule](r:class.index.new)). The checks are those of the `init` that `Relation`
supplies: they come after the `init` of a subclass has run, and a subclass
whose `init` does not pass the arguments on is not refused for them.

```rexx
say .relation~new~items .relation~new(0)~items .relation~new(3, )~items
call try '.relation~new(-1)'
call try '.relation~new("x")'
call try '.relation~new(.nil)'
call try '.relation~new(1.5)'
call try '.relation~new(1, 2)'
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
::class watch subclass relation
::method init
  say 'init' arg()
  forward class (super)
::class quiet subclass relation
::method init
  return
```

```output
0 0 0
.relation~new(-1) => 93.923
.relation~new("x") => 93.923
.relation~new(.nil) => 93.923
.relation~new(1.5) => 93.923
.relation~new(1, 2) => 93.902
init 1
init 1
.watch~new("x") => 93.923
0
```

## Entries and indexes

r[class.relation.indexes]
An index is any object, `.nil` and the null string included, and it is found
as [a table finds it](r:class.table.indexes): by `hashCode` and `==`, so a
string is the index of its characters, strictly compared, and an object that
declares neither method is its own index only. A relation does not merge equal
indexes: each entry keeps the index object that was put with it, so two
entries can hold two distinct objects that are equal as indexes, and a method
that takes an index answers for all of the entries whose index is equal to the
argument.

```rexx
r = .relation~new
r~put('v1', 'Key')
r~put('v2', 'KEY')
r~put('v3', 5)
r~put('v4', 5.0)
r~put('v5', .nil)
r~put('v6', '')
a = .array~of(1)
r~put('arr', a)
say r~items r~at('Key') r~at('key') r~at(5) r~at('5.0') r~at('') r~at(.nil)
say r~at(a) (r~at(.array~of(1)) == .nil) r~hasIndex(.sub~new('Key'))
r~put('v7', .eq~new('x', 'first'))
r~put('v8', .eq~new('x', 'second'))
p = .eq~new('x', 'probe')
say r~items r~items(p) r~hasIndex(p) r~allAt(p)~sort~makeString('L', ',')
ix = r~allIndexes
n = 0
do i over ix
  if i~isA(.eq) then n += 1
end
say n 'indexes are .eq objects'
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
7 v1 The NIL object v3 v4 v6 v5
arr 1 1
9 2 1 v7,v8
2 indexes are .eq objects
```

r[class.relation.indexes.sends]
`at`, `[]`, `hasIndex`, `remove` and `items` with an index look the index up
by sending `hashCode` to the index argument, once, and then `==` to the index
argument with a stored index as the argument, for each stored index that has
the same hash code, until one answers true. Further stored indexes may be sent
`==` as well, which ones is not specified ([the hash order
rule](r:class.relation.deviation.33)); when the relation is empty, the only
message is `hashCode`. `put` and `[]=` send
`hashCode` to the index and nothing else: no `==` is sent, as an entry is
added whether or not an equal index is there. The very object that is stored
is no exception to the `==` ([the deviation
rule](r:class.relation.deviation.78)). In the example every index hashes to
the same string, so each stored index is compared.

```rexx
.local~log = .array~new
say 'empty:' .relation~new~at(.k~new('p')) .local~log~makeString('L', ' | ')
calls = 'r~at(.k~new("p"))|r[.k~new("p")]|r~hasIndex(.k~new("p"))',
  '|r~items(.k~new("p"))|r~remove(.k~new("p"))|r~remove(.k~new("s1"))',
  '|r~put("v2", .k~new("p"))|r[.k~new("p")] = "v2"',
  '|r~put("v3", .k~new("s1"))'
do c over calls~makeArray('|')
  r = .relation~new
  r~put('v1', .k~new('s1'))
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
r~at(.k~new("p")) => hash p | == p s1
r[.k~new("p")] => hash p | == p s1
r~hasIndex(.k~new("p"))  => hash p | == p s1
r~items(.k~new("p")) => hash p | == p s1
r~remove(.k~new("p")) => hash p | == p s1
r~remove(.k~new("s1"))  => hash s1 | == s1 s1
r~put("v2", .k~new("p")) => hash p
r[.k~new("p")] = "v2"  => hash p
r~put("v3", .k~new("s1")) => hash s1
```

## Reading and writing entries

r[class.relation.put]
`put(item, index)` and `[index] = item` add an entry for the item at the index
and answer nothing. Entries already at the index, or already holding the item,
stay, and the same pair put twice makes two entries. The item may be any
object, `.nil` included. The methods take exactly two arguments: a missing or
omitted one raises `SYNTAX 88.901`, and a third raises `SYNTAX 93.902`.

```rexx
r = .relation~new
r~put('a', 'k')
r['k'] = 'b'
r~put('a', 'k')
r~put(.nil, 'n')
r~put('c', .nil)
say r~items r~items('k') r~items('n') r~items(.nil),
  r~allAt('k')~sort~makeString('L', ',')
call try 'r~put("x")'
call try 'r~put(, "x")'
call try 'r~put("x", )'
call try 'r~put("x", "y", "z")'
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
5 3 1 1 a,a,b
r~put("x") => 88.901
r~put(, "x") => 88.901
r~put("x", ) => 88.901
r~put("x", "y", "z") => 93.902
```

r[class.relation.at]
`at(index)` and `[index]` answer an item of an entry whose index is equal to
the argument, and `.nil` when there is none. When several entries qualify,
which item is answered is not specified. The methods take exactly one
argument: none, or an omitted one, raises `SYNTAX 88.901`, and a second raises
`SYNTAX 93.902`; the number of arguments is checked before the index is looked
up.

```rexx
r = .relation~new
r~put('item', 'k')
r~put('x', 'k2')
r~put('y', 'k2')
say r~at('k') r['k'] r~at('K') (r~at('zz') == .nil) (r~at(.nil) == .nil)
say r~at('k2')~length (r~at('k2') == 'x' | r~at('k2') == 'y')
call try 'r~at'
call try 'r~at(, "x")'
call try 'r~at("a", "b")'
call try 'r[]'
exit
try:
  signal on syntax name bad
  interpret 'r2 =' arg(1)
  say arg(1) '=>' r2
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
item item The NIL object 1 1
1 1
r~at => 88.901
r~at(, "x") => 93.902
r~at("a", "b") => 93.902
r[] => 88.901
```

r[class.relation.allat]
`allAt(index)` answers a new `Array` of the items of all the entries whose
index is equal to the argument, one for each entry, in an order that is not
specified, and an empty `Array` when there is none. It sends `hashCode` to the
index first, as [the lookup rule](r:class.relation.indexes.sends) says, and
then `==` to the index with stored indexes as the argument. The answer is an
instance of `Array` for a subclass receiver too, and does not follow later
changes. It takes exactly one argument: none, or an omitted one, raises
`SYNTAX 93.903`, and a second raises `SYNTAX 93.902`.

```rexx
.local~log = .array~new
r = .many~new
r~put('a', 'k')
r~put('b', 'k')
r~put('a', 'k')
r~put('c', 'j')
x = r~allAt('k')
r~put('d', 'k')
say x~sort~makeString('L', ',') x~class~id r~allAt('j')~items
e = r~allAt('zz')
say e~items e~class~id
call try 'r~allAt'
call try 'r~allAt(, "x")'
call try 'r~allAt("a", "b")'
r2 = .relation~new
r2~put('v', .k~new('s1'))
.local~log~empty
x = r2~allAt(.k~new('p'))
say .local~log[1] (.local~log~items >= 2)
exit
try:
  signal on syntax name bad
  interpret 'z =' arg(1)
  say arg(1) '=>' z
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass relation
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
a,a,b Array 1
0 Array
r~allAt => 93.903
r~allAt(, "x") => 93.902
r~allAt("a", "b") => 93.902
hash p 1
```

## Questions about the contents

r[class.relation.hasindex]
`hasIndex(index)` answers 1 when some entry has an index equal to the argument
and 0 otherwise. It takes exactly one argument: none, or an omitted one,
raises `SYNTAX 88.901`, and a second raises `SYNTAX 93.902`.

```rexx
r = .relation~new
r~put(.nil, 'k')
say r~hasIndex('k') r~hasIndex('K') r~hasIndex('zz') r~hasIndex(.nil)
r~remove('k')
say r~hasIndex('k')
call try 'r~hasIndex'
call try 'r~hasIndex("a", "b")'
exit
try:
  signal on syntax name bad
  interpret 'r2 =' arg(1)
  say arg(1) '=>' r2
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 0 0 0
0
r~hasIndex => 88.901
r~hasIndex("a", "b") => 93.902
```

r[class.relation.items]
`items` with no argument answers the number of entries, and `items(index)` the
number of entries whose index is equal to the argument, 0 when there is none.
`isEmpty` answers 1 when there is no entry and 0 otherwise. `items` takes at
most one argument and an omitted argument is no argument: a second raises
`SYNTAX 93.902`. `isEmpty` takes none: one raises `SYNTAX 93.902`.

```rexx
r = .relation~new
say r~items r~items('k') r~isEmpty
r~put('a', 'k')
r~put('a', 'k')
r~put('b', 'j')
say r~items r~items('k') r~items('j') r~items('zz') r~items(.nil) r~items(,),
  r~isEmpty
call try 'r~items("a", "b")'
call try 'r~items(, "b")'
call try 'r~isEmpty(1)'
exit
try:
  signal on syntax name bad
  interpret 'r2 =' arg(1)
  say arg(1) '=>' r2
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
0 0 1
3 2 1 0 0 3 0
r~items("a", "b") => 93.902
r~items(, "b") => 93.902
r~isEmpty(1) => 93.902
```

r[class.relation.hasitem]
`hasItem(item)` answers 1 when some entry has an item equal to the argument
and 0 otherwise; with a second argument, `hasItem(item, index)` answers 1 only
when one entry has both an item equal to the first argument and an index equal
to the second, so an item that exists under other indexes only gives 0. An
omitted second argument is no argument. The methods take one or two arguments:
a missing or omitted item raises `SYNTAX 93.903`, and a third argument raises
`SYNTAX 93.902`. `.nil` is an item like any other.

```rexx
r = .relation~new
r~put('x', 'k1')
r~put('x', 'k2')
r~put(.nil, 'k2')
say r~hasItem('x') r~hasItem('X') r~hasItem(.nil) r~hasItem('zz')
say r~hasItem('x', 'k1') r~hasItem('x', 'k2') r~hasItem('x', 'k3')
say r~hasItem(.nil, 'k1') r~hasItem(.nil, 'k2') r~hasItem('x', )
call try 'r~hasItem'
call try 'r~hasItem(, "k1")'
call try 'r~hasItem("x", "k1", 3)'
exit
try:
  signal on syntax name bad
  interpret 'r2 =' arg(1)
  say arg(1) '=>' r2
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 0 1 0
1 1 0
0 1 1
r~hasItem => 93.903
r~hasItem(, "k1") => 93.903
r~hasItem("x", "k1", 3) => 93.902
```

r[class.relation.hasitem.sends]
`hasItem(item)`, `index`, `allIndex` and `removeItem(item)` send `==` to the
item argument, once for each entry that is searched, with the stored item as
the argument, until one answers true, and send nothing else, `hashCode`
included. The entries are visited in an order that is not specified, so the
number of messages is specified only when no item matches, as then every entry
is visited. `hasItem(item, index)` and `removeItem(item, index)` first look
the index up as [the lookup rule](r:class.relation.indexes.sends) says, and
then send `==` to the item argument with the item of each entry whose index
matches, and no message when no index matches. The search with the very object
that is stored is no exception: its `==` is sent as for any other item ([the
deviation rule](r:class.relation.deviation.78)).

```rexx
.local~n = 0
r = .relation~new
r~put('s', 'k1')
r~put('t', 'k2')
r~put('u', 'k3')
q = .thing~new
calls = 'r~hasItem(q)|r~index(q)|r~allIndex(q)|r~removeItem(q)',
  '|r~hasItem(q, "none")',
  '|r~hasItem(q, "k1")|r~removeItem(q, "none")|r~removeItem(q, "k1")'
do c over calls~makeArray('|')
  .local~n = 0
  interpret 'x =' c
  say c '=>' .local~n
end
say r~items
::class thing
::method '=='
  .local~n += 1
  return 0
::method hashCode
  .local~n += 100
  return 1
```

```output
r~hasItem(q) => 3
r~index(q) => 3
r~allIndex(q) => 3
r~removeItem(q)  => 3
r~hasItem(q, "none")  => 0
r~hasItem(q, "k1") => 1
r~removeItem(q, "none") => 0
r~removeItem(q, "k1") => 1
3
```

r[class.relation.index]
`index(item)` answers the index of an entry whose item is equal to the
argument, and `.nil` when there is none; when several entries qualify, which
index is answered is not specified. `allIndex(item)` answers a new `Array` of
the indexes of all such entries, one for each entry, in an order that is not
specified, and an empty `Array` when there is none; the same index twice when
two entries hold the item at it. The answer of `allIndex` is an instance of
`Array` for a subclass receiver too, and does not follow later changes. Both
take exactly one argument: none, or an omitted one, raises `SYNTAX 93.903`,
and a second raises `SYNTAX 93.902`. The messages sent are those of [the
search rule](r:class.relation.hasitem.sends).

```rexx
r = .many~new
r~put('a', 'k')
r~put('a', 'k')
r~put('a', 'j')
r~put('b', 'm')
say r~index('b') (r~index('zz') == .nil) r~index('a')~length
x = r~allIndex('a')
say x~sort~makeString('L', ',') x~class~id r~allIndex('b')~items,
  r~allIndex('zz')~items
call try 'r~index'
call try 'r~allIndex'
call try 'r~index(, "x")'
call try 'r~allIndex(1, 2)'
exit
try:
  signal on syntax name bad
  interpret 'r2 =' arg(1)
  say arg(1) '=>' r2
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass relation
```

```output
m 1 1
j,k,k Array 1 0
r~index => 93.903
r~allIndex => 93.903
r~index(, "x") => 93.902
r~allIndex(1, 2) => 93.902
```

## Removing entries

r[class.relation.remove]
`remove(index)` removes one entry whose index is equal to the argument and
answers its item, and answers `.nil` when there is none; when several entries
qualify, which one is removed is not specified, and the others stay. The index
stays in the relation while an entry has it. It takes exactly one argument:
none, or an omitted one, raises `SYNTAX 88.901`, and a second raises `SYNTAX
93.902`.

```rexx
r = .relation~new
r~put('v', 'k')
r~put('w', 'k')
r~put('z', 'j')
a = r~remove('k')
say (a == 'v' | a == 'w') r~items r~items('k') r~hasIndex('k')
b = r~remove('k')
say (b == 'v' | b == 'w') (a \== b) r~items r~hasIndex('k')
say (r~remove('k') == .nil) r~remove('j') r~isEmpty
call try 'r~remove'
call try 'r~remove("a", "b")'
exit
try:
  signal on syntax name bad
  interpret 'r2 =' arg(1)
  say arg(1) '=>' r2
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 2 1 1
1 1 1 0
1 z 1
r~remove => 88.901
r~remove("a", "b") => 93.902
```

r[class.relation.removeall]
`removeAll(index)` removes every entry whose index is equal to the argument
and answers a new `Array` of their items, one for each entry, in an order that
is not specified; the answer is an empty `Array` when there is none. It sends
`hashCode` to the index first, then `==` as [the lookup
rule](r:class.relation.indexes.sends) says. The answer is an instance of
`Array` for a subclass receiver too. It takes exactly one argument: none, or
an omitted one, raises `SYNTAX 93.903`, and a second raises `SYNTAX 93.902`.

```rexx
.local~log = .array~new
r = .many~new
r~put('a', 'k')
r~put('b', 'k')
r~put('a', 'k')
r~put('c', 'j')
x = r~removeAll('k')
say x~sort~makeString('L', ',') x~class~id r~items r~hasIndex('k')
e = r~removeAll('k')
say e~items e~class~id r~items
call try 'r~removeAll'
call try 'r~removeAll("a", "b")'
r2 = .relation~new
r2~put('v', .k~new('s1'))
.local~log~empty
x = r2~removeAll(.k~new('p'))
say .local~log[1] (.local~log~items >= 2)
exit
try:
  signal on syntax name bad
  interpret 'z =' arg(1)
  say arg(1) '=>' z
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass relation
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
a,a,b Array 1 0
0 Array 1
r~removeAll => 93.903
r~removeAll("a", "b") => 93.902
hash p 1
```

r[class.relation.removeitem]
`removeItem(item)` removes one entry whose item is equal to the argument, by
the search of [the search rule](r:class.relation.hasitem.sends), and answers
the item that was stored, or `.nil` when no entry matches; when several
entries qualify, which one is removed is not specified. `removeItem(item,
index)` removes one entry that has both an item equal to the first argument
and an index equal to the second, and answers `.nil` when there is none. An
omitted second argument is no argument. The index stays in the relation while
an entry has it, and it is gone when the entry was the last one. The methods
take one or two arguments: a missing or omitted item raises `SYNTAX 93.903`,
and a third argument raises `SYNTAX 93.902`.

```rexx
r = .relation~new
r~put('a', 'k')
r~put('a', 'k')
r~put('a', 'j')
r~put('b', 'k')
say r~removeItem('a', 'k') r~items r~items('k')
say r~removeItem('a', 'k') r~items r~items('k')
say (r~removeItem('a', 'k') == .nil) r~items
say (r~removeItem('b', 'j') == .nil) r~items
say r~removeItem('a') r~items r~hasIndex('j')
say (r~removeItem('a') == .nil) r~removeItem('b', ) r~items r~isEmpty
call try 'r~removeItem'
call try 'r~removeItem(, "k")'
call try 'r~removeItem("a", "k", 3)'
exit
try:
  signal on syntax name bad
  interpret 'r2 =' arg(1)
  say arg(1) '=>' r2
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
a 3 2
a 2 1
1 2
1 2
a 1 0
1 b 0 1
r~removeItem => 93.903
r~removeItem(, "k") => 93.903
r~removeItem("a", "k", 3) => 93.902
```

r[class.relation.empty]
`empty` removes every entry and answers the relation itself. It takes no
argument: one raises `SYNTAX 93.902`.

```rexx
r = .relation~new
r~put('v', 'k')
r~put('w', 'k')
e = r~empty
say (e == r) r~items r~isEmpty r~hasIndex('k')
call try 'r~empty(1)'
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
1 0 1 0
r~empty(1) => 93.902
```

## Lists of the entries

r[class.relation.makearray]
`allIndexes` and `makeArray` answer a new `Array` with the index of each
entry, so an index that is in several entries appears several times, and
`allItems` a new `Array` with the item of each entry. They are instances of
`Array` for a subclass receiver too, never the receiver, and do not follow
later changes. They take no argument: one raises `SYNTAX 93.902`. The order is
not specified ([the hash order rule](r:class.relation.deviation.33)), and is
the same for the three answers and for the supplier of one unchanged relation,
so the item at a position belongs to the index at the same position. The
methods send no message to the relation, the indexes or the items.

```rexx
r = .many~new
do i = 1 to 40
  r~put('item'i, 'key' || (i // 5))
end
ix = r~allIndexes
it = r~allItems
bad = 0
do i = 1 to ix~items
  if \r~allAt(ix[i])~hasItem(it[i]) then bad += 1
  if r~makeArray[i] \== ix[i] then bad += 1
end
say ix~class~id it~class~id r~makeArray~class~id ix~items it~items bad
a = r~allItems
r~put('x', 'extra')
say a~items r~items (ix == r~allIndexes)
call try 'r~allIndexes(1)'
call try 'r~allItems(1)'
call try 'r~makeArray(1)'
exit
try:
  signal on syntax name bad
  interpret 'r2 =' arg(1)
  say arg(1) '=>' r2
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass relation
```

```output
Array Array Array 40 40 0
40 41 0
r~allIndexes(1) => 93.902
r~allItems(1) => 93.902
r~makeArray(1) => 93.902
```

r[class.relation.uniqueindexes]
`uniqueIndexes` answers a new `Array` with one index for each group of equal
indexes in the relation, so an index that is in several entries appears once.
Which of several equal index objects is answered is not specified, and so is
the order. The answer is an instance of `Array` for a subclass receiver too,
and does not follow later changes. `uniqueIndexes` takes no argument: one
raises `SYNTAX 93.902`.

```rexx
r = .many~new
r~put('a', 'k')
r~put('b', 'k')
r~put('c', 'j')
r~put('d', 'm')
x = r~uniqueIndexes
r~put('e', 'zz')
say x~items x~class~id r~allIndexes~items r~uniqueIndexes~items
say x~sort~makeString('L', ',') .relation~new~uniqueIndexes~items
call try 'r~uniqueIndexes(1)'
exit
try:
  signal on syntax name bad
  interpret 'r2 =' arg(1)
  say arg(1) '=>' r2
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass relation
```

```output
3 Array 5 4
j,k,m 0
r~uniqueIndexes(1) => 93.902
```

r[class.relation.supplier]
`supplier` answers a `Supplier` over the entries, the index and the item of
each, and `supplier(index)` a `Supplier` over the entries whose index is equal
to the argument. It is a snapshot: later changes do not reach it. Its order is
that of `allIndexes` ([the order rule](r:class.relation.makearray)). A
relation with no entry, or none at the index, gives a supplier that is not
available. It takes at most one argument, and an omitted argument is no
argument: a second raises `SYNTAX 93.902`. With an index it sends `hashCode`
to it and then `==` to it with stored indexes as the argument, as [the lookup
rule](r:class.relation.indexes.sends) says.

```rexx
.local~log = .array~new
r = .relation~new
r~put('a', 'k')
r~put('b', 'k')
r~put('c', 'j')
s = r~supplier
t = r~supplier('k')
r~put('d', 'k')
r~remove('j')
n = 0
do while s~available
  n += 1
  s~next
end
m = 0
sum = ''
do while t~available
  m += 1
  sum = sum t~index || t~item
  t~next
end
say n m sum~words r~supplier('zz')~available .relation~new~supplier~available
say r~supplier~class~id r~supplier(,)~class~id
call try 'r~supplier("a", "b")'
r2 = .relation~new
r2~put('v', .k~new('s1'))
.local~log~empty
x = r2~supplier(.k~new('p'))
say .local~log[1] (.local~log~items >= 2)
exit
try:
  signal on syntax name bad
  interpret 'z =' arg(1)
  say arg(1) '=>' z
  return
bad: say arg(1) '=>' condition('o')~code
  return
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
3 2 2 0 0
Supplier Supplier
r~supplier("a", "b") => 93.902
hash p 1
```

## Copies and the string value

r[class.relation.copy]
`copy` answers a new relation of the class of the receiver with the same
entries. The indexes and the items are the same objects, and the two relations
do not see each other's changes. As [the copy rule](r:class.object.copy) says
of every object, the variables of a subclass instance are copied as they are,
and `init` is not sent. It takes no argument: one raises `SYNTAX 93.902`.

```rexx
t = .many~new
t~tag = 'T'
t~put('v1', 'k1')
t~put('v2', 'k1')
a = .array~new
t~put(a, 'arr')
c = t~copy
say c~class~id c~items c~tag c~items('k1') (c == t) (c~at('arr') == a)
c~put('v3', 'k2')
c~removeAll('k1')
say t~items c~items t~items('k1') c~items('k1')
call try 't~copy(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass relation
::attribute tag
::method init
  say 'init'
  forward class (super)
```

```output
init
MANY 3 T 2 0 1
3 2 2 0
t~copy(1) => 93.902
```

r[class.relation.string]
A relation has no `makeString` method, so its string value is the one of
`Object`: its name, `a Relation`, as [the string rule](r:class.index.string)
says, whatever the relation holds, and `request('STRING')` answers `.nil`.

```rexx
r = .relation~new
r~put('x', 'k')
say r '|' r~string '|' r 'x' r~hasMethod('MAKESTRING')
say (r~request('STRING') == .nil) .relation~new
```

```output
a Relation | a Relation | a Relation x 0
1 a Relation
```

## The methods of the mixins

r[class.relation.inherited]
The class method `of` and the instance methods `putAll`, `union`,
`difference`, `intersection`, `xor`, `subset`, `disjoint` and `equivalent` are
not declared by `Relation`. `of` and `putAll` are those of
[`MapCollection`](mapcollection.md), and the others those of
[`Collection`](collection.md); a relation is one of the collections they take
as an argument, and the answers of the first four are instances of the class
of the receiver. `putAll` stores with the `put` of the receiver, so an index
that the receiver and the argument share is in two entries afterwards. In the
example the arguments are relations of pairs of an index and an item.

```rexx
a = .many~of(.array~of('a', 'A1'), .array~of('b', 'B1'))
c3 = .array~of('c', 'C3')
b = .relation~of(.array~of('b', 'B2'), .array~of('c', 'C2'), c3)
do op over 'union difference intersection xor'~makeArray(' ')
  r = a~send(op, b)
  say op r~class~id r~items listed(r)
end
say a~subset(b) a~disjoint(b) a~equivalent(b) a~subset(a~copy)
c = .many~of(.array~of('b', 'B1'))
c~putAll(b)
say c~items c~items('b') listed(c)
exit
listed: procedure
  use arg r
  s = ''
  do i over r~uniqueIndexes~sort
    do j over r~allAt(i)~sort
      s = s i'='j
    end
  end
  return s~strip
::class many subclass relation
```

```output
union MANY 5 a=A1 b=B1 b=B2 c=C2 c=C3
difference MANY 2 a=A1 b=B1
intersection MANY 0 
xor MANY 5 a=A1 b=B1 b=B2 c=C2 c=C3
0 1 0 1
4 2 b=B1 b=B2 c=C2 c=C3
```

## Messages the methods send

r[class.relation.sends]
No method that `Relation` declares sends a message to its receiver: an
override of `at`, `put` or any other of the methods is never asked by another
method, and `[]` and `[]=` do not use `at` and `put`. The messages the methods
do send go to other objects and have a rule each: `init` to the new object by
`new` ([the new rule](r:class.relation.new)); `hashCode`, and `==` where the
rule says so, to the index argument by `at`, `[]`, `put`, `[]=`, `hasIndex`,
`items`, `remove`, `allAt`, `removeAll`, `supplier`, `hasItem` and
`removeItem` with an index ([the lookup
rule](r:class.relation.indexes.sends)); `==` to the item argument by
`hasItem`, `index`, `allIndex` and `removeItem` ([the search
rule](r:class.relation.hasitem.sends)). `makeArray`, `allIndexes`, `allItems`,
`copy`, `empty`, `isEmpty` and `items` with no index send nothing to the
indexes and the items. The set operations send what the pages of the mixins
say. In the first example a subclass has every method overridden to record its
name, and each call records only its own; in the second the indexes and items
record every message a program is likely to override.

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
calls = 'x = a[1]|a["k1"] = "x"|x = a~allAt("k1")|x = a~allIndex("q")',
  '|x = a~allIndexes|x = a~allItems|x = a~at("k1")|x = a~hasIndex("k1")',
  '|x = a~hasItem("q")|x = a~index("q")|x = a~isEmpty|x = a~items',
  '|x = a~items("k1")|x = a~makeArray|a~put(1, "k9")|x = a~remove("k9")',
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
::class spy subclass relation
```

```output
24 calls, 0 sent more than one message to the receiver
```

```rexx
.local~log = .array~new
ms = 'hashCode == = request makeString string objectName defaultName'
do m over ms~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"'); forward class (super)"
  .rec~define(m, src)
end
.local~on = 0
r = .relation~new
do i = 1 to 3
  r~put(.rec~new(i), .rec~new(-i))
end
.local~on = 1
a = r~allIndexes
a = r~allItems
a = r~makeArray
a = r~supplier
c = r~copy
a = r~items
a = r~isEmpty
say .local~log~items
r~empty
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

r[class.relation.deviation.33]
The order of the answers of `allIndexes`, `allItems`, `makeArray`,
`uniqueIndexes`, `allAt`, `allIndex`, `removeAll` and `supplier`, the entry
that `at`, `index`, `remove` and `removeItem` choose among several, and the
stored indexes other than those with the same hash code that a lookup compares
with `==`, are not specified, and may differ from ooRexx's. Which entries are
produced, and how often, is specified. In the example the answers are sorted.

```rexx
r = .relation~new
r~put(1, 'one')
r~put(2, 'two')
r~put(3, 'three')
r~put(4, 'three')
say r~allIndexes~sort~makeString('L', ',')
say r~allItems~sort~makeString('L', ',')
say r~uniqueIndexes~items r~items r~makeArray~items
```

```output
one,three,three,two
1,2,3,4
3 4 4
```

r[class.relation.deviation.78]
A lookup or a `remove` with the very object that is stored as an index sends
that object's `==` as for any other index, and a search for an item with the
very object that is stored, by `hasItem`, `index`, `allIndex` or `removeItem`,
sends that object's `==` as for any other item, so an object whose `==`
answers false is not found even at its own entry. ooRexx compares identity
first, finds it and sends nothing but `hashCode`. See [Deviation
78](../deviations.md).

```rexx
.local~n = 0
r = .relation~new
q = .thing~new
r~put(q, 'k')
say r~hasItem(q) .local~n
say (r~index(q) == .nil) .local~n
say (r~removeItem(q) == .nil) .local~n r~items
r~put('v', q)
.local~n = 0
say r~at(q) .local~n
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

r[class.relation.deviation.109]
The size argument of `new` that is an instance of a subclass of `String` is
read by its own characters, and `REQUEST` and `MAKESTRING` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an
instance `request` with `STRING` or `MAKESTRING`, so an override of either
changes the size that `new` checks. In the example `MAKESTRING` answers
another text.

```rexx
say .relation~new(.sub~new('2'))~items
::class sub subclass string
::method makestring
  return 'other'
```

```output
0
```
