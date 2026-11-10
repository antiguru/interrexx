---
element: "List"
kind: class
rexxref: "Builtin Classes > Collection Classes > List Class; Builtin Classes > Collection Classes > List Class > new (Class Method); Builtin Classes > Collection Classes > List Class > of (Class Method); Builtin Classes > Collection Classes > List Class > []; Builtin Classes > Collection Classes > List Class > []=; Builtin Classes > Collection Classes > List Class > allIndexes; Builtin Classes > Collection Classes > List Class > allItems; Builtin Classes > Collection Classes > List Class > append; Builtin Classes > Collection Classes > List Class > at; Builtin Classes > Collection Classes > List Class > delete; Builtin Classes > Collection Classes > List Class > empty; Builtin Classes > Collection Classes > List Class > first; Builtin Classes > Collection Classes > List Class > firstItem; Builtin Classes > Collection Classes > List Class > hasIndex; Builtin Classes > Collection Classes > List Class > hasItem; Builtin Classes > Collection Classes > List Class > index; Builtin Classes > Collection Classes > List Class > insert; Builtin Classes > Collection Classes > List Class > isEmpty; Builtin Classes > Collection Classes > List Class > items; Builtin Classes > Collection Classes > List Class > last; Builtin Classes > Collection Classes > List Class > lastItem; Builtin Classes > Collection Classes > List Class > makeArray; Builtin Classes > Collection Classes > List Class > next; Builtin Classes > Collection Classes > List Class > previous; Builtin Classes > Collection Classes > List Class > put; Builtin Classes > Collection Classes > List Class > remove; Builtin Classes > Collection Classes > List Class > removeItem; Builtin Classes > Collection Classes > List Class > section; Builtin Classes > Collection Classes > List Class > supplier"
deviations: [109, 118]
---

# List

A `List` is an ordered collection that grows and shrinks at any point. Its
entries are addressed by indexes that the list chooses and hands back when an
item is added: an index names an entry for as long as the entry is in the
list, whatever is inserted or removed around it, so it is a handle on the
entry, not a position. The order of the entries is a separate matter, and the
list keeps it. The class inherits [`OrderedCollection`](orderedcollection.md),
and through it [`Collection`](collection.md); [the hierarchy
rule](r:class.index.hierarchy) lists what it inherits.

The page follows the order of the rexxref entry. Examples that show an error
use a small routine `try`, as the builtin pages do. The page says nothing
about an entry whose item was omitted in a `put` or an `insert`.

## The class

r[class.list.creation]
`List` declares twenty-six instance methods and two class methods, `new` and
`of`. The instance methods are `[]`, `[]=`, `allIndexes`, `allItems`,
`append`, `at`, `delete`, `empty`, `first`, `firstItem`, `hasIndex`,
`hasItem`, `index`, `insert`, `isEmpty`, `items`, `last`, `lastItem`,
`makeArray`, `next`, `previous`, `put`, `remove`, `removeItem`, `section` and
`supplier`, and a `copy` that this page specifies ([the copy
rule](r:class.list.copy)). A list has no `makeString`, no `toString`, no
`size`, no `fill` and no dimensions, and no method of its own that sorts:
`sort`, `sortWith`, `stableSort` and `stableSortWith` are those of
`OrderedCollection`, which sort a list through its `makeArray`, `first`,
`next` and `[]=` ([the sort rule](r:class.list.sort)). The set operations and
`appendAll` are those of the mixins, and [one rule](r:class.list.inherited)
lists them.

```rexx
l = .list~new
own = 'AT PUT APPEND INSERT SECTION SUPPLIER REMOVEITEM FIRSTITEM'
n = 0
do m over own~makearray(' ')
  if l~hasMethod(m) then n += 1
end
say l~class~id n .list~hasMethod('OF') l~hasMethod('OF')
say l~hasMethod('SORT') l~hasMethod('UNION') l~hasMethod('APPENDALL')
say l~hasMethod('MAKESTRING') l~hasMethod('TOSTRING') l~hasMethod('SIZE')
say l~hasMethod('FILL') l~hasMethod('DIMENSION')
```

```output
List 8 1 0
1 1 1
0 0 0
0 0
```

## Making a list

r[class.list.new]
`new` makes an empty list. Its optional argument is a hint for how large the
list will grow and changes nothing the program can see; it is read as [the
index rule](r:class.list.indexes) reads an index and must be a whole number
that is not negative, or `SYNTAX 93.923` is raised. A second argument
raises `SYNTAX 93.902`, and omitted arguments at the end count for nothing.
`new` sends `init` to the new object with the arguments it received, so a
subclass's `init` sees the hint, unlike the `init` of an array ([the new
rule](r:class.index.new)).

```rexx
l = .list~new
say l~items l~isEmpty l~first
say .list~new(0)~items .list~new(1000)~items .list~new(3, )~items
call try '.list~new(-1)'
call try '.list~new("x")'
call try '.list~new(.nil)'
call try '.list~new(1.5)'
call try '.list~new(1, 2)'
x = .watch~new(7)
y = .watch~new
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class watch subclass list
::method init
  say 'init' arg()
  forward class (super)
```

```output
0 1 The NIL object
0 0 0
.list~new(-1) => 93.923
.list~new("x") => 93.923
.list~new(.nil) => 93.923
.list~new(1.5) => 93.923
.list~new(1, 2) => 93.902
init 1
init 0
```

r[class.list.of]
`of` makes a list that holds its arguments in the order given: the first is
the first entry. An argument left out at the end counts for nothing, so
`of(1, )` holds one item; an argument left out before a given one raises
`SYNTAX 93.903`. `.nil` is an item like any other. The result is an instance
of the class that received the message. `of` sends `init` to the new object
with no argument and not `new`, and adds the items itself: the `append`,
`insert` and `put` of a subclass are not sent for them. The indexes are those
that a fresh list hands out, [the index rule](r:class.list.indexes) says
which.

```rexx
l = .list~of('a', 'b', 'c')
say l~items l~allItems~makeString('L', ',') l~allIndexes~makeString('L', ',')
say .list~of~items .list~of('x', )~items .list~of(.nil, .nil)~items
call try '.list~of(1, , 3)'
call try '.list~of(, 1)'
x = .watch~of(1, 2)
say x~items
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class watch subclass list
::method new class
  say 'new'
  forward class (super)
::method init
  say 'init' arg()
  forward class (super)
::method append
  say 'append'
  forward class (super)
::method insert
  say 'insert'
  forward class (super)
::method put
  say 'put'
  forward class (super)
```

```output
3 a,b,c 0,1,2
0 1 2
.list~of(1, , 3) => 93.903
.list~of(, 1) => 93.903
init 0
2
```

## Indexes

r[class.list.indexes]
An index of a list is a whole number of 0 or more, chosen by the list when an
entry is created and kept until the entry is removed. A method that takes an
index reads it from its string value ([the conversion
rule](r:conv.string.object)) as a whole-number argument of a builtin is ([the
form rule](r:conv.whole.form), [the digits rule](r:conv.whole.digits)): `"1"`,
`' + 1 '` and `1.0` are as good as `1`, NUMERIC DIGITS plays no part, and
anything that is not a whole number of 0 or more, `.nil` included, or that
those rules refuse, raises `SYNTAX 93.918`. A valid index that names no entry is
not an error for the
methods that look entries up: `at`, `next`, `previous`, `delete` and `remove`
answer `.nil` and `hasIndex` answers 0; the methods that must have an entry,
`put`, `insert` and `section`, raise `SYNTAX 93.918`. When an entry is removed
its index becomes free. A new entry takes the index that was freed most
recently, when one is free, and otherwise the lowest whole number that has not
been used since the list was made or last emptied. A list that only grows
therefore hands out 0, 1, 2 and so on; `empty` makes the list start again at
0. The number of an entry never changes, wherever it is inserted. A `copy` has
the same indexes, and the same state, as the list it is made from.

```rexx
l = .list~new
a = l~append('a'); b = l~append('b'); c = l~append('c')
say a b c
l~remove(a)
l~remove(c)
say l~append('x') l~append('y') l~append('z')
say l~allIndexes~makeString('L', ',')
say l~allItems~makeString('L', ',')
m = .list~of('p', 'q')
m~empty
say m~append('r') m~append('s')
say m~at('1') m~at(1.0) m~hasIndex(' 1 ') m~hasIndex(7) (m~at(7) == .nil)
call try 'm~at(-1)'
call try 'm~at("x")'
call try 'm~at(1.5)'
call try 'm~at(.nil)'
call try 'm~at(1000000000000000000)'
call try 'm~put("t", 9)'
call try 'm~insert("t", 9)'
call try 'm~section(9)'
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
0 1 2
2 0 3
1,2,0,3
b,x,y,z
0 1
s s 1 0 1
m~at(-1) => 93.918
m~at("x") => 93.918
m~at(1.5) => 93.918
m~at(.nil) => 93.918
m~at(1000000000000000000) => 93.918
m~put("t", 9) => 93.918
m~insert("t", 9) => 93.918
m~section(9) => 93.918
```
## Reading and writing entries

r[class.list.at]
`at(index)` answers the item of the entry with that index, and `.nil` when no
entry has it. `[]` is the same method under another name: `l[i]` is the
message `[]`. The method takes exactly one argument: none raises
`SYNTAX 93.903` and a second raises `SYNTAX 93.902`. The index is read as [the
index rule](r:class.list.indexes) says. An override of `at` is not asked when
`[]` is sent, nor the other way round.

```rexx
l = .list~of('a', 'b')
say l~at(0) l[1] (l~at(5) == .nil)
call try 'l~at'
call try 'l~at(0, 1)'
call try 'l[]'
call try 'l[0, 1]'
x = .twin~new
x~append('p')
say x~at(0) x[0]
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class twin subclass list
::method at
  return 'at override'
::method '[]'
  return 'bracket override'
```

```output
a b 1
l~at => 93.903
l~at(0, 1) => 93.902
l[] => 93.903
l[0, 1] => 93.902
at override bracket override
```

r[class.list.put]
`put(item, index)` replaces the item of the entry with that index and answers
no value; the order of the list and the indexes do not change. `l[i] = v` is
the message `[]=` and the same as `put`. The index must name an entry, or
`SYNTAX 93.918` is raised, as for any index that is not a whole number of 0 or
more ([the index rule](r:class.list.indexes)). The method takes two arguments:
fewer, or an omitted index, raise `SYNTAX 93.903`, and a third raises
`SYNTAX 93.902`. A `put` of a subclass is not asked for `[]=`, nor the other
way round.

```rexx
l = .list~of('a', 'b', 'c')
l~put('B', 1)
l[2] = 'C'
say l~allItems~makeString('L', ',') l~allIndexes~makeString('L', ',')
call try 'l~put("x", 7)'
call try 'l~put("x", .nil)'
call try 'l~put("x")'
call try 'l~put'
call try 'l~put("x", 0, 0)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
a,B,C 0,1,2
l~put("x", 7) => 93.918
l~put("x", .nil) => 93.918
l~put("x") => 93.903
l~put => 93.903
l~put("x", 0, 0) => 93.902
```

r[class.list.append]
`append(item)` adds the item as the last entry and answers its index. The
method takes exactly one argument: none, or an omitted one, raises
`SYNTAX 93.903` and a second raises `SYNTAX 93.902`. `.nil` is an item like
any other.

```rexx
l = .list~new
say l~append('a') l~append('b') l~append(.nil)
say l~items l~lastItem~class~id (l~lastItem == .nil)
call try 'l~append'
call try 'l~append(, 1)'
call try 'l~append(1, 2)'
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
0 1 2
3 Object 1
l~append => 93.903
l~append(, 1) => 93.902
l~append(1, 2) => 93.902
```

r[class.list.insert]
`insert(item, index)` adds the item as a new entry directly after the entry
with that index and answers the new index. An `index` of `.nil` makes the item
the first entry, and an omitted `index` makes it the last. The index must name
an entry, or `SYNTAX 93.918` is raised. The method takes one or two arguments:
a third raises `SYNTAX 93.902`. The indexes of the other entries do not
change; only the order does.

```rexx
l = .list~of('a', 'b', 'c')
say l~insert('x', 0) l~allItems~makeString('L', ',')
say l~insert('y', .nil) l~allItems~makeString('L', ',')
say l~insert('z') l~allItems~makeString('L', ',')
say l~insert('w', l~last) l~allItems~makeString('L', ',')
say l~allIndexes~makeString('L', ',')
call try 'l~insert("q", 99)'
call try 'l~insert("q", "x")'
call try 'l~insert("q", 0, 1)'
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
3 a,x,b,c
4 y,a,x,b,c
5 y,a,x,b,c,z
6 y,a,x,b,c,z,w
4,0,3,1,2,5,6
l~insert("q", 99) => 93.918
l~insert("q", "x") => 93.918
l~insert("q", 0, 1) => 93.902
```

r[class.list.delete]
`delete(index)` and `remove(index)` are the same method: they take the entry
with that index out of the list and answer its item, and answer `.nil` and
change nothing when no entry has the index. The index of the removed entry
becomes free ([the index rule](r:class.list.indexes)), and the indexes of the
other entries do not change. The methods take exactly one argument: none
raises `SYNTAX 93.903` and a second raises `SYNTAX 93.902`. An index that is
not a whole number of 0 or more raises `SYNTAX 93.918`.

```rexx
l = .list~of('a', 'b', 'c')
say l~delete(1) l~items l~allIndexes~makeString('L', ',')
say l~remove(0) l~items
say (l~remove(0) == .nil) (l~delete(9) == .nil) l~items
call try 'l~delete'
call try 'l~remove(1, 2)'
call try 'l~delete(.nil)'
call try 'l~remove("x")'
call try 'l~remove(-1)'
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
b 2 0,2
a 1
1 1 1
l~delete => 93.903
l~remove(1, 2) => 93.902
l~delete(.nil) => 93.918
l~remove("x") => 93.918
l~remove(-1) => 93.918
```

r[class.list.removeitem]
`removeItem(item)` removes the first entry, in list order, whose item is equal
to the argument by the strict `==`, and answers the item that was stored, or
`.nil` when there is none. Its search is the one of `hasItem` and `index`
([the search rule](r:class.list.removeitem.sends)). The method takes exactly
one argument: none, or an omitted one, raises `SYNTAX 93.903` and a second
raises `SYNTAX 93.902`.

```rexx
l = .list~of('a', 'b', 'a', 1)
say l~removeItem('a') l~allIndexes~makeString('L', ',')
say l~removeItem('a') l~allIndexes~makeString('L', ',')
say (l~removeItem('a') == .nil) l~items (l~removeItem(1.0) == .nil)
call try 'l~removeItem'
call try 'l~removeItem(, 1)'
call try 'l~removeItem(1, 2)'
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
a 1,2,3
a 1,3
1 2 1
l~removeItem => 93.903
l~removeItem(, 1) => 93.902
l~removeItem(1, 2) => 93.902
```

r[class.list.removeitem.sends]
`removeItem` sends `==` to the argument, once for each item in list order,
with that item as the argument, until one answers true, and sends nothing
else. `hasItem` and `index` make the same search.

```rexx
l = .list~of(.thing~new('p'), .thing~new('q'), .thing~new('r'))
x = l~removeItem(.thing~new('q'))
say x~name l~items
::class thing
::method init
  expose name
  use arg name
::method name
  expose name
  return name
::method '=='
  expose name
  use arg other
  say name '==' other~name
  return name = other~name
```

```output
q == p
q == q
q 2
```
## Questions about the contents

r[class.list.hasindex]
`hasIndex(index)` answers 1 when an entry has that index and 0 when none has,
including a valid index that no entry has. The method takes exactly one
argument: none raises `SYNTAX 93.903` and a second raises `SYNTAX 93.902`. An
index that is not a whole number of 0 or more raises `SYNTAX 93.918` ([the
index rule](r:class.list.indexes)).

```rexx
l = .list~of('a', 'b')
l~remove(0)
say l~hasIndex(0) l~hasIndex(1) l~hasIndex(7) l~hasIndex('1')
call try 'l~hasIndex'
call try 'l~hasIndex(0, 1)'
call try 'l~hasIndex(-1)'
call try 'l~hasIndex(.nil)'
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
0 1 0 1
l~hasIndex => 93.903
l~hasIndex(0, 1) => 93.902
l~hasIndex(-1) => 93.918
l~hasIndex(.nil) => 93.918
```

r[class.list.hasitem]
`hasItem(item)` answers 1 when some item of the list is equal to the argument
by the strict `==`, as [the search rule](r:class.list.removeitem.sends) says,
and 0 otherwise. The method takes exactly one argument: none, or an omitted
one, raises `SYNTAX 93.903` and a second raises `SYNTAX 93.902`.

```rexx
l = .list~of(1, 'x', .nil)
say l~hasItem(1) l~hasItem('1') l~hasItem(1.0) l~hasItem('X') l~hasItem(.nil)
say .list~of(1)~hasItem(.nil)
call try 'l~hasItem'
call try 'l~hasItem(, 1)'
call try 'l~hasItem(1, 2)'
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
1 1 0 0 1
0
l~hasItem => 93.903
l~hasItem(, 1) => 93.902
l~hasItem(1, 2) => 93.902
```

r[class.list.index]
`index(item)` answers the index of the first entry in list order whose item is
equal to the argument, and `.nil` when there is none. The method takes exactly
one argument: none, or an omitted one, raises `SYNTAX 93.903` and a second
raises `SYNTAX 93.902`.

```rexx
l = .list~of('p', 'q', 'q')
l~insert('q', .nil)
say l~index('q') l~index('p') (l~index('z') == .nil)
call try 'l~index'
call try 'l~index(1, 2)'
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
3 0 1
l~index => 93.903
l~index(1, 2) => 93.902
```

## Walking the list

r[class.list.first]
`first` answers the index of the first entry in list order, `last` that of the
last, and `firstItem` and `lastItem` their items; all four answer `.nil` for
an empty list. They take no argument: any argument raises `SYNTAX 93.902`.

```rexx
l = .list~of('a', 'b', 'c')
l~insert('z', .nil)
say l~first l~last l~firstItem l~lastItem
e = .list~new
say (e~first == .nil) (e~last == .nil) (e~firstItem == .nil)
say (e~lastItem == .nil)
call try 'l~first(1)'
call try 'l~lastItem(1)'
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
3 2 z c
1 1 1
1
l~first(1) => 93.902
l~lastItem(1) => 93.902
```

r[class.list.next]
`next(index)` answers the index of the entry after the one with that index, in
list order, and `previous(index)` the one before it; both answer `.nil` at the
end or beginning of the list, and for a valid index that no entry has. The
methods take exactly one argument: none raises `SYNTAX 93.903` and a second
raises `SYNTAX 93.902`. An index that is not a whole number of 0 or more
raises `SYNTAX 93.918`.

```rexx
l = .list~of('a', 'b', 'c')
x = l~insert('x', 0)
say l~first l~next(l~first) l~next(x) l~next(1) (l~next(2) == .nil)
say l~last l~previous(l~last) l~previous(1) (l~previous(0) == .nil)
say (l~next(99) == .nil) (l~previous(99) == .nil)
call try 'l~next'
call try 'l~previous(0, 1)'
call try 'l~next(-1)'
call try 'l~previous(.nil)'
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
0 3 1 2 1
2 1 3 1
1 1
l~next => 93.903
l~previous(0, 1) => 93.902
l~next(-1) => 93.918
l~previous(.nil) => 93.918
```

## Size and emptying

r[class.list.items]
`items` answers the number of entries and `isEmpty` answers 1 for a list
without entries and 0 otherwise. A list has no `size`. `empty` removes every
entry, answers the receiver, and makes the list hand out indexes from 0 again
([the index rule](r:class.list.indexes)). The three take no argument: any
argument raises `SYNTAX 93.902`.

```rexx
l = .list~of('a', 'b')
say l~items l~isEmpty
say (l~empty == l) l~items l~isEmpty l~append('c')
call try 'l~items(1)'
call try 'l~isEmpty(1)'
call try 'l~empty(1)'
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
2 0
1 0 1 0
l~items(1) => 93.902
l~isEmpty(1) => 93.902
l~empty(1) => 93.902
```

## Taking part of a list

r[class.list.section]
`section(start, count)` answers a new `List` that holds the entries from the
one with index `start` on, in list order: `count` of them, or all of them up
to the end when `count` is omitted or runs past the end. A `count` of 0 gives
an empty list. The new list has its own indexes, handed out from 0 as for any
list ([the index rule](r:class.list.indexes)), the items are the receiver's
own objects, and the receiver is not changed. The `start` must name an entry,
or `SYNTAX 93.918` is raised, as for any index that is not a whole number of 0
or more; the `count`, read as [the index rule](r:class.list.indexes) reads an
index,
must be a whole number that is not negative, or `SYNTAX 93.923`. No `start`, or
an omitted one, raises `SYNTAX 93.903`, and a
third argument `SYNTAX 93.902`. The answer is an instance of `List`, whatever
the class of the receiver is, and no `new` or `init` is sent for it.

```rexx
l = .list~of('a', 'b', 'c', 'd')
x = l~insert('x', 0)
s = l~section(l~first)
say s~class~id s~allItems~makeString('L', ',')
say s~allIndexes~makeString('L', ',')
s = l~section(x, 2)
say s~allItems~makeString('L', ',') s~allIndexes~makeString('L', ',')
say l~section(x, 99)~items l~section(x, 0)~items l~section(3)~items
call try 'l~section'
call try 'l~section(9)'
call try 'l~section(0, -1)'
call try 'l~section(0, "x")'
call try 'l~section(0, 1, 2)'
y = .many~of(1, 2, 3)
say y~section(0)~class~id
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass list
::method new class
  say 'new'
  forward class (super)
::method init
  say 'init' arg()
  forward class (super)
```

```output
List a,x,b,c,d
0,1,2,3,4
x,b 0,1
4 0 1
l~section => 93.903
l~section(9) => 93.918
l~section(0, -1) => 93.923
l~section(0, "x") => 93.923
l~section(0, 1, 2) => 93.902
init 0
List
```
## The items, the indexes and a supplier

r[class.list.makearray]
`makeArray` and `allItems` answer a new `Array` that holds the items of the
list in list order, the first at index 1, and `allIndexes` a new `Array` of
the indexes of the entries in the same order. The answers are instances of
`Array` whatever the class of the receiver is, never the receiver, and do not
follow later changes to the list. They take no argument: any argument raises
`SYNTAX 93.902`.

```rexx
l = .list~of('a', 'b', 'c')
l~insert('x', 0)
say l~makeArray~makeString('L', ',') l~allItems~makeString('L', ',')
say l~allIndexes~makeString('L', ',') l~allItems~class~id
items = l~allItems
l~append('d')
say items~items l~items
say .many~of(1)~makeArray~class~id .many~of(1)~allIndexes~class~id
call try 'l~makeArray(1)'
call try 'l~allItems(1)'
call try 'l~allIndexes(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass list
```

```output
a,x,b,c a,x,b,c
0,3,1,2 Array
4 5
Array Array
l~makeArray(1) => 93.902
l~allItems(1) => 93.902
l~allIndexes(1) => 93.902
```

r[class.list.supplier]
`supplier` answers a `Supplier` whose indexes and items are those of the
entries in list order. It is a snapshot: entries added, removed or replaced
afterwards do not change it. An empty list gives a supplier that is not
available. The method takes no argument: any argument raises `SYNTAX 93.902`.
The `Supplier` class has its own page.

```rexx
l = .list~of('a', 'b')
s = l~supplier
l~append('c')
l~put('B', 1)
do while s~available
  say s~index s~item
  s~next
end
say .list~new~supplier~available
call try 'l~supplier(1)'
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
0 a
1 b
0
l~supplier(1) => 93.902
```

## Copies and the string value

r[class.list.copy]
`copy` answers a new list of the same class as the receiver, with the same
entries in the same order, the same indexes and the same state of free
indexes, so that the next `append` to either hands out the same index and
neither list sees the other's changes. The items are the same objects: the
copy is shallow. As [the copy rule](r:class.object.copy) says of every object,
the variables of a subclass instance are copied as they are and the name
stays; `init` is not sent. The method takes no argument: one raises
`SYNTAX 93.902`.

```rexx
l = .many~of('a', 'b', 'c')
l~tag = 'T'
l~remove(1)
c = l~copy
say c~class~id c~items c~tag c~allIndexes~makeString('L', ',') (c == l)
say c~append('n') l~append('m')
c~append('o')
say l~items c~items
::class many subclass list
::attribute tag
::method init
  say 'init'
  forward class (super)
```

```output
init
MANY 2 T 0,2 0
1 1
3 4
```

r[class.list.string]
A list has no `makeString` method, so its string value is the one of `Object`:
its name, `a List`, as [the string rule](r:class.index.string) says, whatever
the list holds; `request('STRING')` answers `.nil`. Use `makeArray` to turn
the items into an array.

```rexx
l = .list~of('a', 'b')
say l '|' l~string '|' l 'x'
say (l~request('STRING') == .nil) .list~new
```

```output
a List | a List | a List x
1 a List
```
## Sorting

r[class.list.sort]
`sort`, `stableSort`, `sortWith(comparator)` and `stableSortWith(comparator)`
of a list are the methods of [`OrderedCollection`](orderedcollection.md), and
they sort the list in place and answer the receiver. The entries keep their
indexes and their places in the list, and the items change places among them:
after a sort the first entry of the list holds the first item in the new
order, and so on down the list. They are written in terms of other messages, as
[the sort
rule](r:class.orderedcollection.sort) says: the list's `makeArray`, `first`,
`next` and `[]=`, and the `sort` or `sortWith`, `size` and `[]` of the array
that `makeArray` answers, which compares the items as [the array
rule](r:class.array.sort) says. If the answer holds fewer items than the list
has
entries the last entries are left as they were, and when it holds more the
`[]=` for the first item without an entry raises `SYNTAX 93.918`. The methods
check their arguments as [the sort rule](r:class.orderedcollection.sort) says:
`sort` and `stableSort` take
none, and `sortWith` and `stableSortWith` take one, `SYNTAX 93.901` being
raised when it is missing; the comparator is not looked at, so the string
`"x"` is accepted by a list of one item.

```rexx
l = .list~of('c', 'a', 'b')
first = l~first
say (l~sort == l) l~allItems~makeString('L', ',')
say l~allIndexes~makeString('L', ',')
say l~at(first) l~sortWith(.desc~new)~allItems~makeString('L', ',')
say .list~new~sort~items .list~of('z')~sortWith('x')~items
call try 'l~sort(1)'
call try 'l~stableSort(1)'
call try 'l~sortWith'
call try 'l~stableSortWith(1, 2)'
call try 'l~sortWith(.nil)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class desc
::method compare
  use arg x, y
  return y~compareTo(x)
```

```output
1 a,b,c
0,1,2
a c,b,a
0 1
l~sort(1) => 93.902
l~stableSort(1) => 93.902
l~sortWith => 93.901
l~stableSortWith(1, 2) => 93.902
l~sortWith(.nil) => 97.1
```

```rexx
l = .few~of('c', 'b', 'a')
l~sort
say l~allItems~makeString('L', ',')
m = .more~of('c', 'b')
signal on syntax name e
m~sort
exit
e: say condition('o')~code m~allItems~makeString('L', ',')
::class few subclass list
::method makeArray
  return .array~of('z', 'y')
::class more subclass list
::method makeArray
  return .array~of('z', 'y', 'x')
```

```output
y,z,a
93.918 x,y
```

r[class.list.sort.sends]
The messages of [the sort rule](r:class.list.sort) go to the receiver and to
the array that `makeArray` answered, in the order that the example shows.

```rexx
.local~on = 1
l = .spy~of('b', 'a')
l~sort
say '--'
l~sortWith(.cmp~new)
::class cmp
::method compare
  use arg x, y
  return x~compareTo(y)
::class spy subclass list
::method sort
  if .local~on then say 'sort' arg()
  forward class (super)
::method sortWith
  if .local~on then say 'sortWith' arg(1)~class~id
  forward class (super)
::method stableSort
  if .local~on then say 'stableSort' arg()
  forward class (super)
::method stableSortWith
  if .local~on then say 'stableSortWith' arg(1)~class~id
  forward class (super)
::method makeArray
  if .local~on then say 'makeArray' arg()
  forward class (super) continue
  return .probe~new(result)
::method first
  if .local~on then say 'first' arg()
  forward class (super)
::method next
  if .local~on then say 'next' arg(1)
  forward class (super)
::method '[]='
  if .local~on then say '[]=' arg(1) arg(2)
  forward class (super)
::class probe
::method init
  expose target
  use arg target
::method unknown
  expose target
  use arg name, args
  say 'array' name'('args~makeString('L', ',')')'
  forward to (target) message (name) arguments (args)
```

```output
sort 0
stableSort 0
makeArray 0
array SORT()
first 0
array SIZE()
array [](1)
[]= a 0
next 0
array [](2)
[]= b 1
next 1
--
sortWith CMP
stableSortWith CMP
makeArray 0
array SORTWITH(a CMP)
first 0
array SIZE()
array [](1)
[]= a 0
next 0
array [](2)
[]= b 1
next 1
```
## The methods of the mixins

r[class.list.inherited]
The methods `union`, `difference`, `intersection`, `xor`, `subset`, `disjoint`
and `equivalent` and `appendAll` are not declared by `List`. The methods
`union`, `difference`, `intersection`, `xor`, `subset` and `appendAll` are
those of [`OrderedCollection`](orderedcollection.md), and `disjoint` and
`equivalent` those of [`Collection`](collection.md), as the pages of the
mixins say, and a
list is one of the collections they take as an argument, so that
`.list~of(1)~union(.array~of(2))` is a list of 1 and 2. A list answers 0 to
`isA` with `Array`, so `union`, `difference` and `xor` answer the copies they
work on, and `intersection` the collection that `new` of the receiver's class
made; all four are instances of the class of the receiver, and `appendAll`
answers the receiver.

```rexx
a = .many~of(1, 2, 2)
b = .array~of(2, 3)
say a~union(b)~class~id a~union(b)~makeArray~makeString('L', ',')
say a~difference(b)~class~id a~difference(b)~makeArray~makeString('L', ',')
i = a~intersection(b)
say i~class~id i~makeArray~makeString('L', ',')
say a~xor(b)~class~id a~xor(b)~makeArray~makeString('L', ',')
say a~subset(b) a~disjoint(b) a~equivalent(b) a~equivalent(.list~of(2, 1, 2))
say (a~appendAll(b) == a) a~makeArray~makeString('L', ',')
say .list~of(1)~union(.array~of(2))~makeArray~makeString('L', ',')
::class many subclass list
```

```output
MANY 1,2,2,2,3
MANY 1,2
MANY 2
MANY 1,2,3
0 0 0 1
1 1,2,2,2,3
1,2
```

## Messages the methods send

r[class.list.sends]
No method that `List` declares sends a message to its receiver: an override of
`at`, `put`, `append` or any other of the methods is never asked by another
method, and `[]` and `[]=` do not use `at` and `put`. The messages the methods
do send go to other objects, and each has a rule of its own: `init` to the new
object by `new` and `of` ([the new rule](r:class.list.new), [the of
rule](r:class.list.of)); `==` to the argument by `removeItem`, `hasItem` and
`index` ([the search rule](r:class.list.removeitem.sends)). The sorts, which
are not methods of the class, send messages to the receiver and to the array
of `makeArray`, as [their rule](r:class.list.sort.sends) says. The methods
`string` and `copy` of an instance and the set operations send what the pages
that own them say. No argument is asked `isA`, and a string value is read as
[the conversion rule](r:conv.string.object) says. In the example a subclass
has every method of `List` overridden to record its name, and each call
records only its own.

```rexx
own = '[] []= allIndexes allItems append at delete empty first firstItem',
  'hasIndex hasItem index insert isEmpty items last lastItem makeArray next',
  'previous put remove removeItem section supplier copy'
.local~log = .array~new
do m over own~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"'); forward class (super)"
  .spy~define(m, src)
end
a = .spy~of('c', 'b', 'a')
calls = 'x = a[1]|a[1] = "x"|a~allIndexes|a~allItems|a~append("d")|a~at(1)',
  '|a~insert("e", 1)|a~hasIndex(1)|a~hasItem("q")|a~index("q")|a~isEmpty',
  '|a~items|a~first|a~last|a~firstItem|a~lastItem|a~makeArray|a~next(1)',
  '|a~previous(2)|a~put("y", 1)|a~section(0)|a~supplier|a~copy',
  '|a~removeItem("q")|a~remove(0)|a~delete(1)|a~empty'
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
::class spy subclass list
```

```output
27 calls, 0 sent more than one message to the receiver
```

## Deviations

r[class.list.deviation.109]
An index, a count or the size hint of `new` that is an instance of a subclass of
`String` is read by its own
characters, and `MAKESTRING` is not sent to it ([Deviation
109](r:class.string.deviation.109)). ooRexx sends such an instance as an
index `request` with `STRING` or `MAKESTRING`, so an override of either can
change what is read. In the example `MAKESTRING` answers another
index.

```rexx
l = .list~of('first', 'second')
say l~at(.num~new('1'))
::class num subclass string
::method makestring
  return '0'
```

```output
second
```

r[class.list.deviation.118]
An instance of a subclass of `List` is read as any other collection is
wherever an array is wanted. `DO OVER` sends it `makeArray`, as [the loop
rule](r:instr.do.over) says, and the methods that read an argument array with
`request('ARRAY')` send it `request`, among them `sendWith`, so an override of
either is used ([Deviation 118](r:instr.do.deviation.118)). ooRexx sends such
an instance neither message and uses the items it holds; it does send them to
the subclasses of the other collection classes. The method `makeArray` itself
is the one this page specifies, and a program that sends it gets the override.
In the example the subclasses answer other items than they hold.

```rexx
l = .mlist~of('held')
do x over l
  say x
end
r = .rlist~of('held')
say .object~new~sendWith('HASMETHOD', r)
::class mlist subclass list
::method makearray
  say 'makearray sent'
  return .array~of('answered')
::class rlist subclass list
::method request
  say 'request' arg(1)
  return .array~of('NOSUCHMETHODZQ')
```

```output
makearray sent
answered
request ARRAY
0
```
