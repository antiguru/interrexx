---
element: "Directory"
kind: class
rexxref: "Builtin Classes > Collection Classes > Directory Class; Builtin Classes > Collection Classes > Directory Class > new (Class Method); Builtin Classes > Collection Classes > Directory Class > []; Builtin Classes > Collection Classes > Directory Class > []=; Builtin Classes > Collection Classes > Directory Class > allIndexes; Builtin Classes > Collection Classes > Directory Class > allItems; Builtin Classes > Collection Classes > Directory Class > at; Builtin Classes > Collection Classes > Directory Class > empty; Builtin Classes > Collection Classes > Directory Class > entry; Builtin Classes > Collection Classes > Directory Class > hasEntry; Builtin Classes > Collection Classes > Directory Class > hasIndex; Builtin Classes > Collection Classes > Directory Class > hasItem; Builtin Classes > Collection Classes > Directory Class > index; Builtin Classes > Collection Classes > Directory Class > isEmpty; Builtin Classes > Collection Classes > Directory Class > items; Builtin Classes > Collection Classes > Directory Class > makeArray; Builtin Classes > Collection Classes > Directory Class > put; Builtin Classes > Collection Classes > Directory Class > remove; Builtin Classes > Collection Classes > Directory Class > removeEntry; Builtin Classes > Collection Classes > Directory Class > removeItem; Builtin Classes > Collection Classes > Directory Class > setEntry; Builtin Classes > Collection Classes > Directory Class > setMethod; Builtin Classes > Collection Classes > Directory Class > supplier; Builtin Classes > Collection Classes > Directory Class > unknown; Builtin Classes > Collection Classes > Directory Class > unsetMethod"
deviations: [29, 33, 78, 109, 123]
---

# Directory

A `Directory` maps string indexes to items. An index is a string compared in
full and by its exact characters, so `ab` and `AB` are two indexes, and an
item is any object. A directory can also hold methods: a method that
`setMethod` associates with a name runs when the name is read, and so makes a
computed entry; a method named `UNKNOWN` answers every read of a name that has
no entry. The class inherits [`MapCollection`](mapcollection.md), and through
it [`Collection`](collection.md); [the hierarchy
rule](r:class.index.hierarchy) lists what it inherits.
[`Properties`](properties.md) is its subclass, and
[`StringTable`](stringtable.md) is a sibling that has the same entry methods
without the methods of a directory.

A directory also gives its entries a second face: a message for which [the
method search](r:class.methods.unknown) finds no method, such as `d~name`,
reaches the `unknown` method of the class, which reads or sets the entry of that
name in upper case ([the unknown rule](r:class.directory.unknown)). The page
follows the order of the rexxref entry. Examples that show an error use a small
routine `try`, as the builtin pages do. The page says nothing about which of
two items equal to the argument `removeItem` removes ([the hash order
rule](r:class.directory.deviation.33)).

## The class

r[class.directory.creation]
`Directory` declares the instance methods `[]`, `[]=`, `allIndexes`,
`allItems`, `at`, `empty`, `entry`, `hasEntry`, `hasIndex`, `hasItem`,
`index`, `isEmpty`, `items`, `makeArray`, `put`, `remove`, `removeEntry`,
`removeItem`, `setEntry`, `setMethod`, `supplier`, `unknown` and
`unsetMethod`, the `init` that `new` runs ([the new
rule](r:class.directory.new)) and the class method `new`; the `copy` of
`Object` copies a directory as [the copy rule](r:class.directory.copy) says.
The class method `of` and the instance methods `putAll`, `union`,
`difference`, `intersection`, `xor`, `subset`, `disjoint` and `equivalent` are
those of the mixins ([one rule](r:class.directory.inherited) lists them). A
directory has no `makeString`: the message reaches `unknown`, so it answers the
entry `MAKESTRING`, `.nil` when there is none, and a directory is not a string
([the string rule](r:class.directory.string)).

```rexx
d = .directory~new
own = '[] []= ALLINDEXES ALLITEMS AT EMPTY ENTRY HASENTRY HASINDEX HASITEM',
  'INDEX ISEMPTY ITEMS MAKEARRAY PUT REMOVE REMOVEENTRY REMOVEITEM SETENTRY',
  'SETMETHOD SUPPLIER UNKNOWN UNSETMETHOD'
n = 0
do m over own~makearray(' ')
  if d~hasMethod(m) then n += 1
end
say d~class~id n .directory~hasMethod('OF') d~hasMethod('OF')
say d~hasMethod('UNION') d~hasMethod('PUTALL') d~hasMethod('APPENDALL')
say d~hasMethod('MAKESTRING') (d~makeString == .nil) d~hasMethod('FIRST')
```

```output
Directory 23 1 0
1 1 0
0 1 0
```

## Making a directory

r[class.directory.new]
`new` makes an empty directory. Its optional argument is a hint for how large
the directory will grow; no method that this page specifies answers
differently for it. It must be a whole number that is not negative, or
`SYNTAX 93.923` is raised. A second argument raises `SYNTAX 93.902`, and
omitted arguments at the end count for nothing. `new` sends `init` to the new
object with the arguments it received, so a subclass's `init` sees them ([the
new rule](r:class.index.new)). The checks are those of the `init` that
`Directory` supplies: they come after the `init` of a subclass has run, and a
subclass whose `init` does not pass the arguments on is not refused for them.

```rexx
say .directory~new~items .directory~new(0)~items .directory~new(3, )~items
call try '.directory~new(-1)'
call try '.directory~new("x")'
call try '.directory~new(.nil)'
call try '.directory~new(1.5)'
call try '.directory~new(1, 2)'
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
::class watch subclass directory
::method init
  say 'init' arg()
  forward class (super)
::class quiet subclass directory
::method init
  return
```

```output
0 0 0
.directory~new(-1) => 93.923
.directory~new("x") => 93.923
.directory~new(.nil) => 93.923
.directory~new(1.5) => 93.923
.directory~new(1, 2) => 93.902
init 1
init 1
.watch~new("x") => 93.923
0
```

## Indexes

r[class.directory.indexes]
An index is read as a string: a string is used as it is, and any other object
by [the string-value rule](r:conv.string.object), which sends it `request`
with the argument `STRING`, so an object whose `makeString` answers a string
is used through that answer, and an object that has neither gives
`SYNTAX 88.909`, as `.nil` does. The index is stored as that string, and two
indexes are the same when their characters are the same: case counts, `5` and
`5.0` are different, and the null string is an index. A method of this page
that takes a name reads it this way, `at`, `[]`, `put`, `[]=`, `hasIndex`,
`remove` and the methods whose name is changed to upper case by [the entry
rule](r:class.directory.entry). An instance of a subclass of `String` is read
by its own characters ([the deviation rule](r:class.directory.deviation.109)).

```rexx
d = .directory~new
d~put('v1', 'Key')
d~put('v2', 5)
d~put('v3', .made~new)
d~put('v4', '')
say d~at('Key') d~at('KEY') d~at('key') d~at(5) d~at('5') d~at(5.0)
say d~at('made') d~at(.made~new) d~hasIndex(.made~new)
say d~at('') d~items
call try 'd~at(.nil)'
call try 'd~put("x", .plain~new)'
call try 'd~at(.plain~new)'
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
v1 The NIL object The NIL object v2 v2 The NIL object
v3 v3 1
v4 4
d~at(.nil) => 88.909
d~put("x", .plain~new) => 88.909
d~at(.plain~new) => 88.909
```

r[class.directory.indexes.request]
An object whose own `request` method answers a string is read through that
answer: `request` with the argument `STRING` is the message that asks any
object for its string value, and `makeString` is used only when `request` of
`Object` sends it.

```rexx
d = .directory~new
d~put('v', 'req')
say d~at(.asker~new)
::class asker
::method request
  return 'req'
```

```output defect=#164
v
```

## Reading and writing entries

r[class.directory.at]
`at(name)` and `[name]` answer the item stored at the index, and `.nil` when
there is none. When the name is the name of a method that
[`setMethod`](r:class.directory.setmethod) associated, the method runs, with
the directory as `self` and no argument, and the answer is what it returns, so
it is run again for each read, and `.nil` when it returns nothing. When there
is neither an item nor a method and the directory has an `UNKNOWN` method,
that method runs with the name as its only argument, as the name was given,
and its answer is the answer of `at`. The methods take exactly one argument:
none, or an omitted one, raises `SYNTAX 88.901`, and a second raises
`SYNTAX 93.902`; the number of arguments is checked before the name is read.

```rexx
d = .directory~new
d~put('item', 'k')
d~setMethod('calc', .method~new('', 'return self~class~id arg()'))
say d~at('k') d['k'] d~at('K') d~at('CALC') d~at('calc')
d~setMethod('unknown', .method~new('', 'return "none:" arg(1)'))
say d~at('missing') d['MISSING'] d~at('k')
call try 'd~at'
call try 'd~at(, "x")'
call try 'd~at("a", "b")'
call try 'd[]'
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
item item The NIL object Directory 0 The NIL object
none: missing none: MISSING item
d~at => 88.901
d~at(, "x") => 93.902
d~at("a", "b") => 93.902
d[] => 88.901
```

r[class.directory.put]
`put(item, name)` and `[name] = item` store the item at the index, replacing
the item or the method that was there, and answer nothing. The item may be any
object, `.nil` included. A method named `UNKNOWN` is not replaced by an item
of that name: the two are kept apart. The methods take exactly two arguments:
a missing or omitted one raises `SYNTAX 88.901`, and a third raises
`SYNTAX 93.902`.

```rexx
d = .directory~new
d~put('a', 'k')
d['k'] = 'b'
d~put(.nil, 'n')
say d~at('k') d~hasIndex('n') (d~at('n') == .nil) d~items
d~setMethod('m', 'return "method"')
d['M'] = 'replaced'
say d~at('M') d~items
d~setMethod('unknown', 'return "unk" arg(1)')
d~put('own', 'UNKNOWN')
say d~at('UNKNOWN') d~at('other') d~items
call try 'd~put("x")'
call try 'd~put(, "x")'
call try 'd~put("x", )'
call try 'd~put("x", "y", "z")'
call try 'd~put("x", .nil)'
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
replaced 3
own unk other 4
d~put("x") => 88.901
d~put(, "x") => 88.901
d~put("x", ) => 88.901
d~put("x", "y", "z") => 93.902
d~put("x", .nil) => 88.909
```

## Entries by upper-case name

r[class.directory.entry]
`entry(name)`, `hasEntry(name)`, `setEntry(name, item)` and
`removeEntry(name)` work like `at`, `hasIndex`, `put` and `remove` with the
name changed to upper case first, so `setEntry('abc', 1)` stores the index
`ABC`, and `entry('abc')` reads it, while `at('abc')` does not find it.
`entry` falls back to a method and to the `UNKNOWN` method as `at` does, with
the upper-case name as the argument of the latter. `hasEntry` is 1 when an
item or a method has that name and does not count `UNKNOWN`; `setEntry` with
the item omitted removes the entry, and with the item present stores it,
replacing a method of that name. `setEntry` answers nothing and does not check
the item. `entry`, `hasEntry` and `removeEntry` take exactly one argument, and
`setEntry` one or two: a missing name raises `SYNTAX 88.901`, and one argument
too many raises `SYNTAX 93.902`.

```rexx
d = .directory~new
d~setEntry('abc', 1)
say d~allIndexes~makeString('L', ',') d~entry('abc') d~entry('ABC')
say d~at('abc')
say d~hasEntry('abc') d~hasEntry('ABC') d~hasIndex('abc') d~hasIndex('ABC')
d~put('lower', 'low')
say d~entry('low') d~hasEntry('low') d~at('low') d~removeEntry('low') d~items
d~setEntry('abc', 'x')
d~setEntry('abc')
say d~items d~hasEntry('abc')
d~setMethod('calc', 'return "computed"')
say d~entry('calc') d~hasEntry('calc')
d~setEntry('calc', 'replaced')
say d~at('CALC') d~items
d~setMethod('unknown', 'return "unk" arg(1)')
say d~entry('missing') d~removeEntry('missing') d~hasEntry('unknown')
call try 'd~entry'
call try 'd~setEntry'
call try 'd~setEntry("a", 1, 2)'
call try 'd~setEntry(.nil, 1)'
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
computed 1
replaced 2
unk MISSING unk MISSING 0
d~entry => 88.901
d~setEntry => 88.901
d~setEntry("a", 1, 2) => 93.902
d~setEntry(.nil, 1) => 88.909
```

## Questions about the contents

r[class.directory.hasindex]
`hasIndex(name)` answers 1 when an item or a method has that index, and 0
otherwise; the `UNKNOWN` method is not counted, and the name is not changed to
upper case. It takes exactly one argument: none, or an omitted one, raises
`SYNTAX 88.901`, and a second raises `SYNTAX 93.902`.

```rexx
d = .directory~new
d~put(.nil, 'k')
d~setMethod('calc', 'return 1')
d~setMethod('unknown', 'return 2')
say d~hasIndex('k') d~hasIndex('K') d~hasIndex('CALC') d~hasIndex('calc')
say d~hasIndex('UNKNOWN') d~hasIndex('zz')
call try 'd~hasIndex'
call try 'd~hasIndex("a", "b")'
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
0 0
d~hasIndex => 88.901
d~hasIndex("a", "b") => 93.902
```

r[class.directory.hasitem]
`hasItem(item)` answers 1 when some item is equal to the argument and 0
otherwise, and `index(item)` answers an index of such an item and `.nil` when
there is none. The stored items are searched first. When none of them is
equal, the methods are run one after the other, and the answer of each is
compared before the next one runs, so a method runs only when nothing searched
before it was equal, and at most once. The `UNKNOWN` method is not searched.
Which index `index` answers when several items qualify is not specified ([the
hash order rule](r:class.directory.deviation.33)). Both take exactly one
argument: none, or an omitted one, raises `SYNTAX 93.903`, and a second raises
`SYNTAX 93.902`. `.nil` is an item like any other.

```rexx
d = .directory~new
d~put('x', 'k1')
d~put(.nil, 'k2')
d~setMethod('calc', 'return "computed"')
say d~hasItem('x') d~hasItem('X') d~hasItem(.nil) d~hasItem('computed')
say d~index('x') d~index('computed') d~index(.nil) (d~index('q') == .nil)
call try 'd~hasItem'
call try 'd~index'
call try 'd~hasItem(, 1)'
call try 'd~index(1, 2)'
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
1 0 1 1
k1 CALC k2 1
d~hasItem => 93.903
d~index => 93.903
d~hasItem(, 1) => 93.902
d~index(1, 2) => 93.902
```

r[class.directory.hasitem.sends]
`hasItem`, `index` and `removeItem` send `==` to the argument, once for each
item that is searched, with that item as the argument, until one answers true,
and send nothing else; the stored items are visited first, in an order that is
not specified, and then the answers of the methods, in an order that is not
specified either, so the number of messages is specified only when no item
matches, as then every item is visited. The search with the very object that
is stored is no exception: its `==` is sent as for any other item ([the
deviation rule](r:class.directory.deviation.78)).

```rexx
.local~n = 0
d = .directory~new
d~put('s', 'k1')
d~put('t', 'k2')
d~put('u', 'k3')
q = .thing~new
say d~hasItem(q) .local~n
.local~n = 0
say (d~index(q) == .nil) .local~n
.local~n = 0
say (d~removeItem(q) == .nil) .local~n d~items
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

r[class.directory.items]
`items` answers the number of entries: the items and the methods, but not the
`UNKNOWN` method. `isEmpty` answers 1 when `items` answers 0 and 0 otherwise, so
that a directory that holds only methods is not empty, and one that holds only
the `UNKNOWN` method is ([Deviation 123](r:class.directory.deviation.123)).
Neither takes an argument: one raises `SYNTAX 93.902`.

```rexx
d = .directory~new
say d~items d~isEmpty
d~put('a', 'k')
d~setMethod('m', 'return 1')
d~setMethod('unknown', 'return 2')
say d~items d~isEmpty
call try 'd~items(1)'
call try 'd~isEmpty(1)'
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
d~items(1) => 93.902
d~isEmpty(1) => 93.902
```

## Removing entries

r[class.directory.remove]
`remove(name)` and `removeEntry(name)` remove the entry with that index (that
name in upper case for `removeEntry`) and answer its item. When the entry is a
method, the method is removed and the answer is what it returned when it ran.
When there is no entry, the answer is `.nil`, or the answer of the `UNKNOWN`
method run with the name as it was given (the upper-case name for
`removeEntry`) when the directory has one; the `UNKNOWN` method itself is not
removed by either. Each takes exactly one argument: none, or an omitted one,
raises `SYNTAX 88.901`, and a second raises `SYNTAX 93.902`.

```rexx
d = .directory~new
d~put('v', 'k')
d~setMethod('m', 'return "ran"')
say d~remove('k') d~remove('k') d~items
say d~remove('M') d~items d~hasEntry('m')
d~setMethod('m', 'return "ran"')
say d~removeEntry('m') d~items
d~setMethod('unknown', 'return "unk" arg(1)')
say d~remove('x') d~removeEntry('x') d~remove('UNKNOWN') d~at('y')
call try 'd~remove'
call try 'd~removeEntry'
call try 'd~remove("a", "b")'
call try 'd~removeEntry("a", "b")'
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
v The NIL object 1
ran 0 0
ran 0
unk x unk X unk UNKNOWN unk y
d~remove => 88.901
d~removeEntry => 88.901
d~remove("a", "b") => 93.902
d~removeEntry("a", "b") => 93.902
```

r[class.directory.removeitem]
`removeItem(item)` removes one entry whose item is equal to the argument, by
the search of [the search rule](r:class.directory.hasitem.sends), and answers
the item that was stored. When the equal item is the answer of a method, the
method is removed as [`remove`](r:class.directory.remove) removes it, so it
runs a second time, and that second answer is the answer of `removeItem`. It
answers `.nil` when no item matches. It removes one entry only. It takes
exactly one argument: none, or an omitted one, raises `SYNTAX 93.903`, and a
second raises `SYNTAX 93.902`.

```rexx
d = .directory~new
d~put('one', 'k1')
d~put('one', 'k2')
d~put('two', 'k3')
d~setMethod('m', 'return "ran"')
say d~removeItem('one') d~items d~removeItem('two') d~items
say d~removeItem('ran') d~items d~hasEntry('m')
say (d~removeItem('zz') == .nil) d~items
call try 'd~removeItem'
call try 'd~removeItem(1, 2)'
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
one 3 two 2
ran 1 0
1 1
d~removeItem => 93.903
d~removeItem(1, 2) => 93.902
```

```rexx
.local~c = 0
d = .directory~new
d~put('x', 'k')
src = '.local~c += 1; if .local~c = 1 then return "y";',
  'return "later"'
d~setMethod('m', src)
say d~hasItem('x') .local~c
say d~removeItem('y') .local~c d~items
```

```output defect=#166
1 0
later 2 1
```

r[class.directory.empty]
`empty` removes every item and answers the directory itself. It does not
remove the methods that `setMethod` associated, `UNKNOWN` included. It takes
no argument: one raises `SYNTAX 93.902`.

```rexx
d = .directory~new
d~put('v', 'k')
d~setMethod('m', 'return "ran"')
d~setMethod('unknown', 'return "unk" arg(1)')
e = d~empty
say (e == d) d~hasIndex('k') d~items d~at('M') d~at('x')
call try 'd~empty(1)'
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
1 0 1 ran unk x
d~empty(1) => 93.902
```

## Methods as entries

r[class.directory.setmethod]
`setMethod(name, method)` associates the method with the name changed to upper
case, replacing the item or the method that was there, and answers nothing.
`method` is read as the `setMethod` of `Object` reads it ([the setMethod
rule](r:class.object.setmethod)): a `Method`, the source of a method as a string
or an array of strings, or any other object, which is sent `request` with
`ARRAY` and then with `STRING`. A source that does not translate raises its
syntax error ([the method rule](r:class.method.new)), `.nil` and an object that
answers neither raise `SYNTAX 93.974`. The items of a source array are read as
[the source rule](r:class.method.new.source) reads them, so an item that gives
no string raises `SYNTAX 93.952`. Without a method, or with it omitted, the
entry of that name is removed. The method runs when the name is read ([the read
rule](r:class.directory.at)). `setMethod('UNKNOWN', m)` sets the method that
answers the reads of names that have no entry; it replaces an item of that name
and is not an entry of the directory. `setMethod` gives the directory no method
of its own, unlike [the `setMethod` of `Object`](r:class.object.setmethod):
`hasMethod` of the name answers what it answered before, and a message of that
name runs the method of the directory that has the name when there is one, and
otherwise reaches [`unknown`](r:class.directory.unknown), which runs the method
with no arguments, whatever the message carried.

```rexx
d = .directory~new
d~setMethod('a', 'return "from a string"')
d~setMethod('b', .array~of('x = 40', 'return x + 2'))
d~setMethod('c', .method~new('', 'return arg()'))
say d~at('A') '|' d~at('B') '|' d~at('C') '|' d~hasIndex('a') d~items
d~put('item', 'D')
d~setMethod('d', 'return "method"')
say d~at('D') d~items
d~setMethod('d')
d~setMethod('c', )
say d~hasIndex('D') d~hasIndex('C') d~items
d~setMethod('e', .mk~new)
d~setMethod('f', .rq~new)
d~setMethod('g', .list~of('return 7'))
say d~at('E') d~at('F') d~at('G') d~items
d~setMethod('m', 'return "m" arg()')
say d~hasMethod('M') d~m(1, 2) d~at('M')
d~setMethod('items', 'return 99')
say d~hasMethod('ITEMS') d~items d~at('ITEMS')
call try 'd~setMethod'
call try 'd~setMethod("x", .nil)'
call try 'd~setMethod("x", .object~new)'
call try 'd~setMethod("x", .array~of(1, .object~new))'
call try 'd~setMethod("x", "this is not ((")'
call try 'd~setMethod("x", "y", "z")'
call try 'd~setMethod(.nil, "return 1")'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class mk
::method makestring
  return 'return 5'
::class rq
::method request
  say '  rq request' arg(1)
  if arg(1) = 'ARRAY' then return .nil
  return 'return "req"'
```

```output
from a string | 42 | 0 | 0 3
method 4
0 0 2
  rq request ARRAY
  rq request STRING
5 req 7 5
0 m 0 m 0
1 7 99
d~setMethod => 88.901
d~setMethod("x", .nil) => 93.974
d~setMethod("x", .object~new) => 93.974
d~setMethod("x", .array~of(1, .object~new)) => 93.952
d~setMethod("x", "this is not ((") => 35.1
d~setMethod("x", "y", "z") => 93.902
d~setMethod(.nil, "return 1") => 88.909
```

```rexx
d = .directory~new
d~setMethod('m', .array~of(.ms~new))
say d~m
::class ms
::method makestring
  return 'return "from ms"'
```

```output defect=#142
from ms
```

r[class.directory.unsetmethod]
`unsetMethod(name)` removes the method associated with the name, the name
changed to upper case, and answers nothing. It removes no item: an entry that
holds an item is left, and a name with no method is ignored. It takes exactly
one argument: none, or an omitted one, raises `SYNTAX 88.901`, and a second
raises `SYNTAX 93.902`.

```rexx
d = .directory~new
d~put('v', 'item')
d~setMethod('m', 'return 1')
d~unsetMethod('item')
d~unsetMethod('m')
d~unsetMethod('nothing')
say d~items d~at('item') d~hasIndex('M')
d~setMethod('unknown', 'return "unk"')
d~unsetMethod('unknown')
say (d~at('x') == .nil)
call try 'd~unsetMethod'
call try 'd~unsetMethod("a", "b")'
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
1 v 0
1
d~unsetMethod => 88.901
d~unsetMethod("a", "b") => 93.902
```

r[class.directory.unknown]
`unknown(name, args)` runs when [the method search](r:class.methods.unknown)
finds no method for a message. When the name
does not end in `=`, it answers what `entry` of the name answers, and the
arguments of the message are ignored; when it ends in `=`, it stores the first
of the arguments at the name without the `=` with what `setEntry` does and
answers nothing, so `d~title = 'x'` and `d~name('y')` set and read the entries
`TITLE` and `NAME`. The message name can be any string, so `d~'two words' = 1`
sets the entry `TWO WORDS`. Neither `entry` nor `setEntry` is sent to the
directory by it. Called directly, it takes exactly two arguments: a missing
name or a missing second argument raises `SYNTAX 93.903`, a third raises
`SYNTAX 93.902`, a name that cannot be read as a string raises
`SYNTAX 88.909`, and an arguments value that is `.nil` raises `SYNTAX 98.913`.
The page says nothing about an assignment that has no value, which [a
deviation](r:class.directory.deviation.29) covers.

```rexx
d = .spy~new
d~title = 'x'
say d~title d~at('TITLE') d~at('title')
say d~title(1, 2)
d~'two words' = 1
say d~hasEntry('two words')
say (d~nothing == .nil)
d~unknown('quoted=', .array~of('v', 'ignored'))
say d~quoted
call try 'd~unknown'
call try 'd~unknown("x")'
call try 'd~unknown("x", .array~new, 3)'
call try 'd~unknown(.nil, .array~new)'
call try 'd~unknown("x", .nil)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class spy subclass directory
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
d~unknown => 93.903
d~unknown("x") => 93.903
d~unknown("x", .array~new, 3) => 93.902
d~unknown(.nil, .array~new) => 88.909
d~unknown("x", .nil) => 98.913
```

## Lists of the entries

r[class.directory.makearray]
`allIndexes` and `makeArray` answer a new `Array` of the indexes of the
entries, and `allItems` a new `Array` of their items, with the answer of each
method (run once for each call of `allItems`; `makeArray` and `allIndexes` do
not run the methods). They are instances of `Array` for a subclass receiver
too, never the receiver, and do not follow later changes. They take no
argument: one raises `SYNTAX 93.902`. The order is not specified ([the hash
order rule](r:class.directory.deviation.33)), and is the same for the three
answers and for the supplier of one unchanged directory, so the item at a
position belongs to the index at the same position.

```rexx
d = .many~new
do i = 1 to 40
  d~put('item'i, 'key'i)
end
d~setMethod('calc', 'return "item0"')
ix = d~allIndexes
it = d~allItems
bad = 0
do i = 1 to ix~items
  if d~at(ix[i]) \== it[i] then bad += 1
  if d~makeArray[i] \== ix[i] then bad += 1
end
say ix~class~id it~class~id d~makeArray~class~id ix~items it~items bad
a = d~allItems
d~put('x', 'extra')
say a~items d~items (ix == d~allIndexes)
call try 'd~allIndexes(1)'
call try 'd~allItems(1)'
call try 'd~makeArray(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass directory
```

```output
Array Array Array 41 41 0
41 42 0
d~allIndexes(1) => 93.902
d~allItems(1) => 93.902
d~makeArray(1) => 93.902
```

r[class.directory.supplier]
`supplier` answers a `Supplier` over the entries, the index and the item of
each, with the answer of each method as its item, run when the supplier is
made and not again when the item is read. It is a snapshot: later changes do
not reach it. Its order is that of `allIndexes` ([the order
rule](r:class.directory.makearray)). An empty directory gives a supplier that
is not available. It takes no argument: one raises `SYNTAX 93.902`.

```rexx
.local~runs = 0
d = .directory~new
d~put('a', 'k')
d~setMethod('m', .method~new('', '.local~runs += 1; return "computed"'))
s = d~supplier
d~put('b', 'k2')
d~remove('k')
n = 0
do while s~available
  if s~index == 'k' then say s~index s~item
  if s~index == 'M' then say s~index s~item
  n += 1
  s~next
end
say n .local~runs .directory~new~supplier~available
call try 'd~supplier(1)'
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
M computed
2 1 0
d~supplier(1) => 93.902
```

## Copies and the string value

r[class.directory.copy]
`copy` answers a new directory of the class of the receiver with the same
entries and the same methods, `UNKNOWN` included, which it does not run. The
items are the same objects, and the two directories do not see each other's
changes. As [the copy rule](r:class.object.copy) says of every object, the
variables of a subclass instance are copied as they are, and `init` is not
sent. It takes no argument: one raises `SYNTAX 93.902`.

```rexx
d = .many~new
d~tag = 'T'
d~put('v1', 'k1')
d~setMethod('m', 'return "ran"')
d~setMethod('unknown', 'return "unk" arg(1)')
a = .array~new
d~put(a, 'arr')
c = d~copy
say c~class~id c~items c~tag c~at('M') c~at('zz') (c == d) (c~at('arr') == a)
c~put('v2', 'k2')
c~unsetMethod('m')
c~unsetMethod('unknown')
say d~items c~items d~at('M') d~at('zz') (c~at('zz') == .nil)
call try 'd~copy(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass directory
::attribute tag
::method init
  say 'init'
  forward class (super)
```

```output
init
MANY 3 T ran unk zz 0 1
3 3 ran unk zz 1
d~copy(1) => 93.902
```

r[class.directory.string]
A directory has no `makeString` method, so its string value is the one of
`Object`: its name, `a Directory`, as [the string rule](r:class.index.string)
says, whatever the directory holds, and `request('STRING')` answers `.nil`.
Entries named `STRING`, `MAKESTRING` or `OBJECTNAME` do not change that: the
methods of those names that every object has are found before `unknown`.

```rexx
d = .directory~new
d~put('x', 'k')
d~makestring = 'text'
d~string = 'other'
say d '|' d~string '|' d 'x' d~makestring d~hasMethod('MAKESTRING')
say (d~request('STRING') == .nil) .directory~new
```

```output
a Directory | a Directory | a Directory x text 0
1 a Directory
```

## The methods of the mixins

r[class.directory.inherited]
The class method `of` and the instance methods `putAll`, `union`,
`difference`, `intersection`, `xor`, `subset`, `disjoint` and `equivalent` are
not declared by `Directory`. `of` and `putAll` are those of
[`MapCollection`](mapcollection.md), and the others those of
[`Collection`](collection.md); a directory is one of the collections they take
as an argument, and the answers of the first four are instances of the class
of the receiver. `putAll` stores with the `put` of the receiver. In the
example the arguments are directories of pairs of an index and an item.

```rexx
a = .many~of(.array~of('a', 'A1'), .array~of('b', 'B1'))
b = .directory~of(.array~of('b', 'B2'), .array~of('c', 'C2'))
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
::class many subclass directory
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

r[class.directory.sends]
No method that `Directory` declares sends a message to its receiver: an
override of `at`, `put`, `entry`, `setEntry` or any other of the methods is
never asked by another method, `[]` and `[]=` do not use `at` and `put`, and
`unknown` does not use `entry` and `setEntry`. The messages the methods do
send go to other objects and have a rule each: `init` to the new object by
`new` ([the new rule](r:class.directory.new)); `==` to the argument by
`hasItem`, `index` and `removeItem` ([the search
rule](r:class.directory.hasitem.sends)); `request` with `STRING` to a name
that is neither a string nor an instance of a subclass of `String` ([the index
rule](r:class.directory.indexes)); `request` with `ARRAY` and then `STRING` to
a method argument of `setMethod` that is not a `Method`, a string or an array
([the setMethod rule](r:class.directory.setmethod)). The set
operations send what the pages of the mixins say. In the example a subclass
has every method overridden to record its name, and each call records only its
own.

```rexx
own = '[] []= allIndexes allItems at empty entry hasEntry hasIndex hasItem',
  'index isEmpty items makeArray put remove removeEntry removeItem setEntry',
  'setMethod supplier unknown unsetMethod copy string'
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
  '|a~setMethod("k7", "return 1")|a~supplier|a~unsetMethod("k7")|a~copy',
  '|a~string|a~unknown("fooz", .array~new)|a~fooz|a~fooz = 1|a~empty'
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
::class spy subclass directory
```

```output
27 calls, 0 sent more than one message to the receiver
```

## Deviations

r[class.directory.deviation.29]
An assignment to a name that gives no value, `d~"A="()`, or the same through
`sendWith`, is refused: interrexx ends the program with exit code 120. ooRexx
stores an unrelated leftover object at the entry. A direct call of
[`unknown`](r:class.directory.unknown) with a name that ends in `=` and an
argument array that holds no item, `d~unknown('A=', .array~new(3))`, is
refused the same way; ooRexx removes the entry, as `setEntry` with the item
omitted does. The assignment with a value is [the unknown
rule](r:class.directory.unknown). See [Deviation 29](../deviations.md).

```rexx
say 'before'
d = .directory~new
d~"A="()
say 'after'
```

```output rc=120
before
```

```rexx
say 'before'
d = .directory~new
d~unknown('A=', .array~new(3))
say 'after'
```

```output rc=120
before
```

r[class.directory.deviation.33]
The order of the answers of `allIndexes`, `allItems`, `makeArray` and
`supplier`, and of the search of `hasItem`, `index` and `removeItem`, is not
specified, and may differ from ooRexx's. Which entries are produced, and how
often, is specified. In the example the answers are sorted.

```rexx
d = .directory~new
d~put(1, 'one')
d~put(2, 'two')
d~put(3, 'three')
say d~allIndexes~sort~makeString('L', ',')
say d~allItems~sort~makeString('L', ',')
say d~items d~makeArray~items
```

```output
one,three,two
1,2,3
3 3
```

r[class.directory.deviation.78]
A search for an item with the very object that is stored, by `hasItem`,
`index` or `removeItem`, sends that object's `==` as for any other item, so an
object whose `==` answers false is not found even at its own entry. ooRexx
compares identity first, finds it and sends nothing. See [Deviation
78](../deviations.md).

```rexx
.local~n = 0
d = .directory~new
q = .thing~new
d~put(q, 'k')
say d~hasItem(q) .local~n
say (d~index(q) == .nil) .local~n
say (d~removeItem(q) == .nil) .local~n d~items
::class thing
::method '=='
  .local~n += 1
  return 0
```

```output
0 1
1 2
1 3 1
```

r[class.directory.deviation.109]
A name, an index or the size argument of `new` that is an instance of a
subclass of `String` is read by its own characters, and `REQUEST` and
`MAKESTRING` are not sent to it ([Deviation
109](r:class.string.deviation.109)). ooRexx sends such an instance `request`
with `STRING` or `MAKESTRING`, so an override of either changes the index
that is stored or looked up and the size that `new` checks. In the example
`MAKESTRING` answers another text.

```rexx
say .directory~new(.sub~new('2'))~items
d = .directory~new
d~put('v', .sub~new('abc'))
say d~at('abc') (d~at('other') == .nil) d~hasIndex(.sub~new('abc'))
::class sub subclass string
::method makestring
  return 'other'
```

```output defect=#140
0
v 1 1
```

r[class.directory.deviation.123]
`isEmpty` agrees with `items`: it answers 0 for a directory that holds a
method of any name but `UNKNOWN`, also when that is all it holds, as `items`
answers 1 or more for it. ooRexx answers 1 for a directory that holds only
methods.

```rexx
d = .directory~new
d~setMethod('m', 'return 1')
say d~items d~isEmpty
d~setMethod('unknown', 'return 2')
say d~items d~isEmpty
e = .directory~new
e~setMethod('unknown', 'return 2')
say e~items e~isEmpty
d~unsetMethod('m')
say d~items d~isEmpty
```

```output
1 0
1 0
0 1
0 1
```
