---
element: "StringTable"
kind: class
rexxref: "Builtin Classes > Collection Classes > StringTable Class; Builtin Classes > Collection Classes > StringTable Class > new (Class Method); Builtin Classes > Collection Classes > StringTable Class > []; Builtin Classes > Collection Classes > StringTable Class > []=; Builtin Classes > Collection Classes > StringTable Class > allIndexes; Builtin Classes > Collection Classes > StringTable Class > allItems; Builtin Classes > Collection Classes > StringTable Class > at; Builtin Classes > Collection Classes > StringTable Class > empty; Builtin Classes > Collection Classes > StringTable Class > entry; Builtin Classes > Collection Classes > StringTable Class > hasEntry; Builtin Classes > Collection Classes > StringTable Class > hasIndex; Builtin Classes > Collection Classes > StringTable Class > hasItem; Builtin Classes > Collection Classes > StringTable Class > index; Builtin Classes > Collection Classes > StringTable Class > isEmpty; Builtin Classes > Collection Classes > StringTable Class > items; Builtin Classes > Collection Classes > StringTable Class > makeArray; Builtin Classes > Collection Classes > StringTable Class > put; Builtin Classes > Collection Classes > StringTable Class > remove; Builtin Classes > Collection Classes > StringTable Class > removeEntry; Builtin Classes > Collection Classes > StringTable Class > removeItem; Builtin Classes > Collection Classes > StringTable Class > setEntry; Builtin Classes > Collection Classes > StringTable Class > supplier; Builtin Classes > Collection Classes > StringTable Class > unknown"
deviations: [29, 33]
---

# StringTable

A `StringTable` maps string indexes to items, as a [`Directory`](directory.md)
does, and has the same entry methods, but it holds nothing else: no method can
be associated with a name, and there is no `UNKNOWN` method. An index is a
string compared in full and by its exact characters, and an item is any
object. The class inherits [`MapCollection`](mapcollection.md), and through it
[`Collection`](collection.md); [the hierarchy rule](r:class.index.hierarchy)
lists what it inherits.

A message that no method answers, such as `t~name`, reaches the `unknown`
method of the class, which reads or sets the entry of that name in upper case
([the unknown rule](r:class.stringtable.unknown)). The page follows the order
of the rexxref entry. Examples that show an error use a small routine `try`,
as the builtin pages do. The page says nothing about which of two items equal
to the argument `removeItem` removes ([the hash order
rule](r:class.stringtable.deviation.33)).

## The class

r[class.stringtable.creation]
`StringTable` declares the instance methods `[]`, `[]=`, `allIndexes`,
`allItems`, `at`, `empty`, `entry`, `hasEntry`, `hasIndex`, `hasItem`,
`index`, `isEmpty`, `items`, `makeArray`, `put`, `remove`, `removeEntry`,
`removeItem`, `setEntry`, `supplier` and `unknown`, a `copy` that this page
specifies ([the copy rule](r:class.stringtable.copy)), and the class method
`new`. The class method `of` and the instance methods `putAll`, `union`,
`difference`, `intersection`, `xor`, `subset`, `disjoint` and `equivalent` are
those of the mixins ([one rule](r:class.stringtable.inherited) lists them). It
has no `setMethod` and no `unsetMethod` of the kind a directory has ([the
method rule](r:class.stringtable.setmethod)), and no `makeString`, so a string
table is not a string ([the string rule](r:class.stringtable.string)).

```rexx
t = .stringtable~new
own = '[] []= ALLINDEXES ALLITEMS AT EMPTY ENTRY HASENTRY HASINDEX HASITEM',
  'INDEX ISEMPTY ITEMS MAKEARRAY PUT REMOVE REMOVEENTRY REMOVEITEM SETENTRY',
  'SUPPLIER UNKNOWN'
n = 0
do m over own~makearray(' ')
  if t~hasMethod(m) then n += 1
end
say t~class~id n .stringtable~hasMethod('OF') t~hasMethod('OF')
say t~hasMethod('UNION') t~hasMethod('PUTALL') t~hasMethod('APPENDALL')
say t~hasMethod('MAKESTRING') (t~makeString == .nil) t~hasMethod('FIRST')
```

```output
StringTable 21 1 0
1 1 0
0 1 0
```

## Making a string table

r[class.stringtable.new]
`new` makes an empty string table. Its optional argument is a hint for how
large the table will grow; no method that this page specifies answers
differently for it. It must be a whole number that is not negative, or
`SYNTAX 93.923` is raised. A second argument raises `SYNTAX 93.902`, and
omitted arguments at the end count for nothing. `new` sends `init` to the new
object with the arguments it received, so a subclass's `init` sees them ([the
new rule](r:class.index.new)). The checks are those of the `init` that
`StringTable` supplies: they come after the `init` of a subclass has run, and
a subclass whose `init` does not pass the arguments on is not refused for
them.

```rexx
say .stringtable~new~items .stringtable~new(0)~items
say .stringtable~new(3, )~items
call try '.stringtable~new(-1)'
call try '.stringtable~new("x")'
call try '.stringtable~new(.nil)'
call try '.stringtable~new(1.5)'
call try '.stringtable~new(1, 2)'
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
::class watch subclass stringtable
::method init
  say 'init' arg()
  forward class (super)
::class quiet subclass stringtable
::method init
  return
```

```output
0 0
0
.stringtable~new(-1) => 93.923
.stringtable~new("x") => 93.923
.stringtable~new(.nil) => 93.923
.stringtable~new(1.5) => 93.923
.stringtable~new(1, 2) => 93.902
init 1
init 1
.watch~new("x") => 93.923
0
```

## Indexes

r[class.stringtable.indexes]
An index is read as a string, exactly as [a directory reads
it](r:class.directory.indexes): a string is used as it is, any other object by
[the string-value rule](r:conv.string.object), so an object whose `makeString`
answers a string is used through that answer, and an object that has neither,
`.nil` included, gives `SYNTAX 88.909`. The index is stored as that string,
case counts, `5` and `5.0` are different indexes, and the null string is an
index. Every method of this page that takes a name reads it this way, those
that change it to upper case afterwards included ([the entry
rule](r:class.stringtable.entry)).

```rexx
t = .stringtable~new
t~put('v1', 'Key')
t~put('v2', 5)
t~put('v3', .made~new)
t~put('v4', '')
say t~at('Key') t~at('KEY') t~at(5) t~at('5') t~at(5.0)
say t~at('made') t~at(.made~new) t~hasIndex(.made~new) t~at('') t~items
call try 't~at(.nil)'
call try 't~put("x", .plain~new)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class made
::method makestring
  return 'made'
::class plain
```

```output
v1 The NIL object v2 v2 The NIL object
v3 v3 1 v4 4
t~at(.nil) => 88.909
t~put("x", .plain~new) => 88.909
```

## Reading and writing entries

r[class.stringtable.at]
`at(name)` and `[name]` answer the item stored at the index, and `.nil` when
there is none. The methods take exactly one argument: none, or an omitted one,
raises `SYNTAX 88.901`, and a second raises `SYNTAX 93.902`; the number of
arguments is checked before the name is read.

```rexx
t = .stringtable~new
t~put('item', 'k')
say t~at('k') t['k'] t~at('K') (t~at('zz') == .nil)
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
item item The NIL object 1
t~at => 88.901
t~at(, "x") => 93.902
t~at("a", "b") => 93.902
t[] => 88.901
```

r[class.stringtable.put]
`put(item, name)` and `[name] = item` store the item at the index, replacing
the item that was there, and answer nothing. The item may be any object,
`.nil` included. The methods take exactly two arguments: a missing or omitted
one raises `SYNTAX 88.901`, and a third raises `SYNTAX 93.902`.

```rexx
t = .stringtable~new
t~put('a', 'k')
t['k'] = 'b'
t~put(.nil, 'n')
say t~at('k') t~hasIndex('n') (t~at('n') == .nil) t~items
call try 't~put("x")'
call try 't~put(, "x")'
call try 't~put("x", )'
call try 't~put("x", "y", "z")'
call try 't~put("x", .nil)'
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
b 1 1 2
t~put("x") => 88.901
t~put(, "x") => 88.901
t~put("x", ) => 88.901
t~put("x", "y", "z") => 93.902
t~put("x", .nil) => 88.909
```

## Entries by upper-case name

r[class.stringtable.entry]
`entry(name)`, `hasEntry(name)`, `setEntry(name, item)` and
`removeEntry(name)` work like `at`, `hasIndex`, `put` and `remove` with the
name changed to upper case first, so `setEntry('abc', 1)` stores the index
`ABC` and `entry('abc')` reads it, while `at('abc')` does not find it.
`setEntry` with the item omitted removes the entry, and answers nothing, and
it does not check the item. `entry`, `hasEntry` and `removeEntry` take exactly
one argument and `setEntry` one or two: a missing name raises `SYNTAX 88.901`,
and one argument too many raises `SYNTAX 93.902`.

```rexx
t = .stringtable~new
t~setEntry('abc', 1)
say t~allIndexes~makeString('L', ',') t~entry('abc') t~entry('ABC')
say t~at('abc')
say t~hasEntry('abc') t~hasEntry('ABC') t~hasIndex('abc') t~hasIndex('ABC')
t~put('lower', 'low')
say t~entry('low') t~hasEntry('low') t~at('low') t~removeEntry('low') t~items
t~setEntry('abc', 'x')
t~setEntry('abc')
say t~items t~hasEntry('abc')
call try 't~entry'
call try 't~setEntry'
call try 't~setEntry("a", 1, 2)'
call try 't~setEntry(.nil, 1)'
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
ABC 1 1
The NIL object
1 1 0 1
The NIL object 0 lower The NIL object 2
1 0
t~entry => 88.901
t~setEntry => 88.901
t~setEntry("a", 1, 2) => 93.902
t~setEntry(.nil, 1) => 88.909
```

r[class.stringtable.unknown]
`unknown(name, args)` runs when no method answers a message. When the name
does not end in `=`, it answers what `entry` of the name answers, and the
arguments of the message are ignored; when it ends in `=`, it stores the first
of the arguments at the name without the `=` with what `setEntry` does and
answers nothing, so `t~title = 'x'` and `t~name('y')` set and read the entries
`TITLE` and `NAME`. The message name can be any string. Neither `entry` nor
`setEntry` is sent to the table by it. Called directly, it takes exactly two
arguments: a missing name or a missing second argument raises `SYNTAX 93.903`,
a third raises `SYNTAX 93.902`, a name that cannot be read as a string raises
`SYNTAX 88.909`, and an arguments value that is `.nil` raises `SYNTAX 98.913`.
The page says nothing about an assignment that has no value, which [a
deviation](r:class.stringtable.deviation.29) covers.

```rexx
t = .spy~new
t~title = 'x'
say t~title t~at('TITLE') t~at('title')
say t~title(1, 2)
t~'two words' = 1
say t~hasEntry('two words')
say (t~nothing == .nil)
t~unknown('quoted=', .array~of('v', 'ignored'))
say t~quoted
call try 't~unknown'
call try 't~unknown("x")'
call try 't~unknown("x", .array~new, 3)'
call try 't~unknown(.nil, .array~new)'
call try 't~unknown("x", .nil)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class spy subclass stringtable
::method entry
  say '  entry sent'
  forward class (super)
::method setEntry
  say '  setEntry sent'
  forward class (super)
```

```output
x x The NIL object
x
1
1
v
t~unknown => 93.903
t~unknown("x") => 93.903
t~unknown("x", .array~new, 3) => 93.902
t~unknown(.nil, .array~new) => 88.909
t~unknown("x", .nil) => 98.913
```

r[class.stringtable.setmethod]
A string table has no `setMethod` and no `unsetMethod` of its own: the methods
of those names that `Object` has are private ([the private
rule](r:class.object.setmethod.private)), and a message from a program does
not reach them. The messages reach `unknown` like any other name, and answer
the entry `SETMETHOD` or `UNSETMETHOD`, `.nil` when there is none; nothing is
stored. An entry named `UNKNOWN` is an ordinary entry.

```rexx
t = .stringtable~new
r = t~setMethod('hi', 'return "method"')
say (r == .nil) t~items t~hasEntry('hi') (t~at('HI') == .nil)
t~setentry('setmethod', 'an entry')
say t~setMethod('hi', 'return "method"')
t~put('own', 'UNKNOWN')
say t~at('UNKNOWN') (t~at('other') == .nil) t~items
```

```output
1 0 0 1
an entry
own 1 2
```

## Questions about the contents

r[class.stringtable.hasindex]
`hasIndex(name)` answers 1 when an item has that index and 0 otherwise; the
name is not changed to upper case. It takes exactly one argument: none, or an
omitted one, raises `SYNTAX 88.901`, and a second raises `SYNTAX 93.902`.

```rexx
t = .stringtable~new
t~put(.nil, 'k')
say t~hasIndex('k') t~hasIndex('K') t~hasIndex('zz')
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
1 0 0
t~hasIndex => 88.901
t~hasIndex("a", "b") => 93.902
```

r[class.stringtable.hasitem]
`hasItem(item)` answers 1 when some item is equal to the argument and 0
otherwise, and `index(item)` answers an index of such an item and `.nil` when
there is none. Which index `index` answers when several items qualify is not
specified ([the hash order rule](r:class.stringtable.deviation.33)). Both take
exactly one argument: none, or an omitted one, raises `SYNTAX 93.903`, and a
second raises `SYNTAX 93.902`. `.nil` is an item like any other.

```rexx
t = .stringtable~new
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

r[class.stringtable.hasitem.sends]
`hasItem`, `index` and `removeItem` send `==` to the argument, once for each
item that is searched, with that item as the argument, until one answers true,
and send nothing else; the items are visited in an order that is not
specified, so the number of messages is specified only when no item matches,
as then every item is visited.

```rexx
.local~n = 0
t = .stringtable~new
t~put('s', 'k1')
t~put('t', 'k2')
t~put('u', 'k3')
q = .thing~new
say t~hasItem(q) .local~n
.local~n = 0
say (t~index(q) == .nil) .local~n
.local~n = 0
say (t~removeItem(q) == .nil) .local~n t~items
::class thing
::method '=='
  .local~n += 1
  return 0
```

```output
0 3
1 3
1 3 3
```

r[class.stringtable.items]
`items` answers the number of entries, and `isEmpty` answers 1 when there is
none and 0 otherwise. Neither takes an argument: one raises `SYNTAX 93.902`.

```rexx
t = .stringtable~new
say t~items t~isEmpty
t~put('a', 'k')
t~put(.nil, 'n')
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

r[class.stringtable.remove]
`remove(name)` and `removeEntry(name)` remove the entry with that index (that
name in upper case for `removeEntry`) and answer its item, and answer `.nil`
when there is none. Each takes exactly one argument: none, or an omitted one,
raises `SYNTAX 88.901`, and a second raises `SYNTAX 93.902`.

```rexx
t = .stringtable~new
t~put('v', 'k')
t~setEntry('e', 'w')
say t~remove('k') (t~remove('k') == .nil) t~items
say t~removeEntry('e') (t~removeEntry('e') == .nil) t~items
call try 't~remove'
call try 't~removeEntry'
call try 't~remove("a", "b")'
call try 't~removeEntry("a", "b")'
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
t~removeEntry => 88.901
t~remove("a", "b") => 93.902
t~removeEntry("a", "b") => 93.902
```

r[class.stringtable.removeitem]
`removeItem(item)` removes one entry whose item is equal to the argument, by
the search of [the search rule](r:class.stringtable.hasitem.sends), and
answers the item that was stored, or `.nil` when no item matches. It removes
one entry only. It takes exactly one argument: none, or an omitted one, raises
`SYNTAX 93.903`, and a second raises `SYNTAX 93.902`.

```rexx
t = .stringtable~new
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

r[class.stringtable.empty]
`empty` removes every entry and answers the table itself. It takes no
argument: one raises `SYNTAX 93.902`.

```rexx
t = .stringtable~new
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

r[class.stringtable.makearray]
`allIndexes` and `makeArray` answer a new `Array` of the indexes of the
entries, and `allItems` a new `Array` of their items. They are instances of
`Array` for a subclass receiver too, never the receiver, and do not follow
later changes. They take no argument: one raises `SYNTAX 93.902`. The order is
not specified ([the hash order rule](r:class.stringtable.deviation.33)), and
is the same for the three answers and for the supplier of one unchanged table,
so the item at a position belongs to the index at the same position.

```rexx
t = .many~new
do i = 1 to 40
  t~put('item'i, 'key'i)
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
::class many subclass stringtable
```

```output
Array Array Array 40 40 0
40 41 0
t~allIndexes(1) => 93.902
t~allItems(1) => 93.902
t~makeArray(1) => 93.902
```

r[class.stringtable.supplier]
`supplier` answers a `Supplier` over the entries, the index and the item of
each. It is a snapshot: later changes do not reach it. Its order is that of
`allIndexes` ([the order rule](r:class.stringtable.makearray)). An empty table
gives a supplier that is not available. It takes no argument: one raises
`SYNTAX 93.902`.

```rexx
t = .stringtable~new
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
say n .stringtable~new~supplier~available
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

r[class.stringtable.copy]
`copy` answers a new string table of the class of the receiver with the same
entries. The items are the same objects, and the two tables do not see each
other's changes. As [the copy rule](r:class.object.copy) says of every object,
the variables of a subclass instance are copied as they are, and `init` is not
sent. It takes no argument: one raises `SYNTAX 93.902`.

```rexx
t = .many~new
t~tag = 'T'
t~put('v1', 'k1')
c = t~copy
say c~class~id c~items c~tag c~at('k1') (c == t)
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
::class many subclass stringtable
::attribute tag
::method init
  say 'init'
  forward class (super)
```

```output
init
MANY 1 T v1 0
1 1 v1 1
t~copy(1) => 93.902
```

r[class.stringtable.string]
A string table has no `makeString` method, so its string value is the one of
`Object`: its name, `a StringTable`, as [the string
rule](r:class.index.string) says, whatever the table holds, and
`request('STRING')` answers `.nil`. Entries named `STRING`, `MAKESTRING` or
`OBJECTNAME` do not change that: the methods of those names that every object
has are found before `unknown`.

```rexx
t = .stringtable~new
t~put('x', 'k')
t~makestring = 'text'
t~string = 'other'
say t '|' t~string '|' t 'x' t~makestring t~hasMethod('MAKESTRING')
say (t~request('STRING') == .nil) .stringtable~new
```

```output
a StringTable | a StringTable | a StringTable x text 0
1 a StringTable
```

## The methods of the mixins

r[class.stringtable.inherited]
The class method `of` and the instance methods `putAll`, `union`,
`difference`, `intersection`, `xor`, `subset`, `disjoint` and `equivalent` are
not declared by `StringTable`. `of` and `putAll` are those of
[`MapCollection`](mapcollection.md), and the others those of
[`Collection`](collection.md); a string table is one of the collections they
take as an argument, and the answers of the first four are instances of the
class of the receiver. `putAll` stores with the `put` of the receiver. In the
example the arguments are string tables of pairs of an index and an item.

```rexx
a = .many~of(.array~of('a', 'A1'), .array~of('b', 'B1'))
b = .stringtable~of(.array~of('b', 'B2'), .array~of('c', 'C2'))
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
::class many subclass stringtable
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

r[class.stringtable.sends]
No method that `StringTable` declares sends a message to its receiver: an
override of `at`, `put`, `entry`, `setEntry` or any other of the methods is
never asked by another method, `[]` and `[]=` do not use `at` and `put`, and
`unknown` does not use `entry` and `setEntry`. The messages the methods do
send go to other objects and have a rule each: `init` to the new object by
`new` ([the new rule](r:class.stringtable.new)); `==` to the argument by
`hasItem`, `index` and `removeItem` ([the search
rule](r:class.stringtable.hasitem.sends)); `request` with `STRING` to a name
that is not a string ([the index rule](r:class.stringtable.indexes)). The set
operations send what the pages of the mixins say. In the example a subclass
has every method overridden to record its name, and each call records only its
own.

```rexx
own = '[] []= allIndexes allItems at empty entry hasEntry hasIndex hasItem',
  'index isEmpty items makeArray put remove removeEntry removeItem setEntry',
  'supplier unknown copy string'
.local~log = .array~new
.local~on = 0
do m over own~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"'); forward class (super)"
  .spy~define(m, src)
end
a = .spy~new
calls = 'x = a[1]|a["k1"] = "x"|a~allIndexes|a~allItems|a~at("k1")',
  '|a~entry("k1")|a~hasEntry("k1")|a~hasIndex("k1")|a~hasItem("q")',
  '|a~index("q")|a~isEmpty|a~items|a~makeArray|a~put(1, "k9")',
  '|a~remove("k9")|a~removeEntry("k9")|a~removeItem("q")|a~setEntry("k8", 1)',
  '|a~supplier|a~copy|a~string|a~unknown("fooz", .array~new)|a~fooz',
  '|a~fooz = 1|a~empty'
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
::class spy subclass stringtable
```

```output
25 calls, 0 sent more than one message to the receiver
```

## Deviations

r[class.stringtable.deviation.29]
An assignment to a name that gives no value, `t~"A="()`, or the same through
`sendWith`, is refused: interrexx ends the program with exit code 120. ooRexx
stores an unrelated leftover object at the entry. The assignment with a value
is [the unknown rule](r:class.stringtable.unknown). See [Deviation
29](../deviations.md).

```rexx
say 'before'
t = .stringtable~new
t~"A="()
say 'after'
```

```output rc=120
before
```

r[class.stringtable.deviation.33]
The order of the answers of `allIndexes`, `allItems`, `makeArray` and
`supplier`, and of the search of `hasItem`, `index` and `removeItem`, is not
specified, and may differ from ooRexx's. Which entries are produced, and how
often, is specified. In the example the answers are sorted.

```rexx
t = .stringtable~new
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
