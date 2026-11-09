---
element: "Queue"
kind: class
rexxref: "Builtin Classes > Collection Classes > Queue Class; Builtin Classes > Collection Classes > Queue Class > new (Class Method); Builtin Classes > Collection Classes > Queue Class > of (Class Method); Builtin Classes > Collection Classes > Queue Class > []; Builtin Classes > Collection Classes > Queue Class > []=; Builtin Classes > Collection Classes > Queue Class > allIndexes; Builtin Classes > Collection Classes > Queue Class > allItems; Builtin Classes > Collection Classes > Queue Class > append; Builtin Classes > Collection Classes > Queue Class > at; Builtin Classes > Collection Classes > Queue Class > delete; Builtin Classes > Collection Classes > Queue Class > empty; Builtin Classes > Collection Classes > Queue Class > first; Builtin Classes > Collection Classes > Queue Class > firstItem; Builtin Classes > Collection Classes > Queue Class > hasIndex; Builtin Classes > Collection Classes > Queue Class > hasItem; Builtin Classes > Collection Classes > Queue Class > index; Builtin Classes > Collection Classes > Queue Class > insert; Builtin Classes > Collection Classes > Queue Class > isEmpty; Builtin Classes > Collection Classes > Queue Class > items; Builtin Classes > Collection Classes > Queue Class > last; Builtin Classes > Collection Classes > Queue Class > lastItem; Builtin Classes > Collection Classes > Queue Class > makeArray; Builtin Classes > Collection Classes > Queue Class > next; Builtin Classes > Collection Classes > Queue Class > peek; Builtin Classes > Collection Classes > Queue Class > previous; Builtin Classes > Collection Classes > Queue Class > pull; Builtin Classes > Collection Classes > Queue Class > push; Builtin Classes > Collection Classes > Queue Class > put; Builtin Classes > Collection Classes > Queue Class > queue; Builtin Classes > Collection Classes > Queue Class > remove; Builtin Classes > Collection Classes > Queue Class > removeItem; Builtin Classes > Collection Classes > Queue Class > section; Builtin Classes > Collection Classes > Queue Class > size; Builtin Classes > Collection Classes > Queue Class > supplier"
deviations: [109]
---

# Queue

A `Queue` is an ordered collection that is addressed by position: the item at
the head is at index 1, the next at index 2, and so on up to the number of
items, so an index is where an item stands now, not a name for it. Adding an
item at the head or in the middle, or taking one out, moves the items behind
it to other indexes. The class inherits
[`OrderedCollection`](orderedcollection.md), and through it
[`Collection`](collection.md); [the hierarchy rule](r:class.index.hierarchy)
lists what it inherits. [`CircularQueue`](circularqueue.md) is its subclass.

The page follows the order of the rexxref entry. Examples that show an error
use a small routine `try`, as the builtin pages do. The page says nothing
about an entry whose item was omitted in a `put` or an `insert`.

## The class

r[class.queue.creation]
`Queue` declares the instance methods `[]`, `[]=`, `allIndexes`, `allItems`,
`append`, `at`, `delete`, `empty`, `first`, `firstItem`, `hasIndex`,
`hasItem`, `index`, `insert`, `isEmpty`, `items`, `last`, `lastItem`,
`makeArray`, `next`, `peek`, `previous`, `pull`, `push`, `put`, `queue`,
`remove`, `removeItem`, `section`, `size` and `supplier`, a `copy` that this
page specifies ([the copy rule](r:class.queue.copy)), and the class methods
`new` and `of`. A queue has no `makeString`, no `toString`, no `fill` and no
dimensions, and no method of its own that sorts: `sort`, `sortWith`,
`stableSort` and `stableSortWith` are those of `OrderedCollection`, which sort
a queue through its `makeArray`, `first`, `next` and `[]=` ([the sort
rule](r:class.queue.sort)). The set operations and `appendAll` are those of
the mixins, and [one rule](r:class.queue.inherited) lists them.

```rexx
q = .queue~new
own = 'AT PUT APPEND INSERT SECTION SUPPLIER REMOVEITEM FIRSTITEM PEEK PULL',
  'PUSH QUEUE SIZE'
n = 0
do m over own~makearray(' ')
  if q~hasMethod(m) then n += 1
end
say q~class~id n .queue~hasMethod('OF') q~hasMethod('OF')
say q~hasMethod('SORT') q~hasMethod('UNION') q~hasMethod('APPENDALL')
say q~hasMethod('MAKESTRING') q~hasMethod('TOSTRING') q~hasMethod('FILL')
say q~hasMethod('DIMENSION') q~hasMethod('SIZE')
```

```output
Queue 13 1 0
1 1 1
0 0 0
0 1
```

## Making a queue

r[class.queue.new]
`new` makes an empty queue. Its optional argument is a hint for how large the
queue will grow and changes nothing the program can see; it must be a whole
number that is not negative, or `SYNTAX 93.923` is raised. A second argument
raises `SYNTAX 93.902`, and omitted arguments at the end count for nothing.
`new` sends `init` to the new object with the arguments it received, so a
subclass's `init` sees them ([the new rule](r:class.index.new)). The checks
are those of the `init` that `Queue` supplies: they come after the `init` of a
subclass has run, and a subclass whose `init` does not pass the arguments on
is not refused for them.

```rexx
q = .queue~new
say q~items q~isEmpty q~first
say .queue~new(0)~items .queue~new(1000)~items .queue~new(3, )~items
call try '.queue~new(-1)'
call try '.queue~new("x")'
call try '.queue~new(.nil)'
call try '.queue~new(1.5)'
call try '.queue~new(1, 2)'
x = .watch~new(7)
y = .watch~new
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
::class watch subclass queue
::method init
  say 'init' arg()
  forward class (super)
::class quiet subclass queue
::method init
  return
```

```output
0 1 The NIL object
0 0 0
.queue~new(-1) => 93.923
.queue~new("x") => 93.923
.queue~new(.nil) => 93.923
.queue~new(1.5) => 93.923
.queue~new(1, 2) => 93.902
init 1
init 0
init 1
.watch~new("x") => 93.923
0
```

r[class.queue.of]
`of` makes a queue that holds its arguments in the order given: the first is
at the head. An argument left out at the end counts for nothing, so `of(1, )`
holds one item; an argument left out before a given one raises
`SYNTAX 93.903`. `.nil` is an item like any other. The result is an instance
of the class that received the message. `of` sends `init` to the new object
with no argument and not `new`, and adds the items itself: the `append`,
`queue`, `push`, `insert` and `put` of a subclass are not sent for them.

```rexx
q = .queue~of('a', 'b', 'c')
say q~items q~allItems~makeString('L', ',') q~allIndexes~makeString('L', ',')
say .queue~of~items .queue~of('x', )~items .queue~of(.nil, .nil)~items
call try '.queue~of(1, , 3)'
call try '.queue~of(, 1)'
x = .watch~of(1, 2)
say x~class~id x~items
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class watch subclass queue
::method new class
  say 'new'
  forward class (super)
::method init
  say 'init' arg()
  forward class (super)
::method append
  say 'append'
  forward class (super)
::method queue
  say 'queue'
  forward class (super)
::method push
  say 'push'
  forward class (super)
::method insert
  say 'insert'
  forward class (super)
::method put
  say 'put'
  forward class (super)
```

```output
3 a,b,c 1,2,3
0 1 2
.queue~of(1, , 3) => 93.903
.queue~of(, 1) => 93.903
init 0
WATCH 2
```

r[class.queue.of.init]
The `init` has already been sent when the omitted argument is found, so a
subclass sees it before the error.

```rexx
call try '.watch~of(1, , 3)'
call try '.watch~of(, 3)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class watch subclass queue
::method init
  say 'init' arg()
  forward class (super)
```

```output defect=#9002
init 0
.watch~of(1, , 3) => 93.903
init 0
.watch~of(, 3) => 93.903
```

## Indexes

r[class.queue.indexes]
An index of a queue is a whole number of at least 1, the position of an item
counted from the head. A method that takes an index reads it from its string
value ([the conversion rule](r:conv.string.object)) as a whole-number argument
of a builtin is ([the form rule](r:conv.whole.form), [the digits
rule](r:conv.whole.digits)): `"2"`, `' 2 '` and `2.0` are as good as `2`,
NUMERIC DIGITS plays no part, and an argument that those rules refuse, a
number below 1, or `.nil` raises `SYNTAX 93.907`. An array, an instance of a
subclass of `Array` included, that holds exactly one such index is read as
that index; an array with none or with more than one raises `SYNTAX 93.907` as
does any other object, such as a list. An index above the number of items is
not an error for the methods that look an item up: `at` answers `.nil`,
`hasIndex` answers 0, `delete` and `remove` answer `.nil`, `next` answers
`.nil`, `previous` answers the last index and `section` an empty queue; the
methods that must reach an item, `put` and `insert`, raise a condition ([the
put rule](r:class.queue.put), [the insert rule](r:class.queue.insert)). Where
a method adds or removes an item in front of others, those others have other
indexes afterwards, and the examples of the methods show it.

```rexx
q = .queue~of('a', 'b', 'c')
say q~at(1) q~at(3) (q~at(4) == .nil) q~hasIndex(4)
say q~at('2') q~at(2.0) q~at(' 2 ') q~at(.array~of(2)) q~hasIndex(' 3 ')
q~push('z')
say q~at(1) q~at(2) q~allIndexes~makeString('L', ',')
q~delete(2)
say q~allItems~makeString('L', ',') q~at(2) q~at(4)
q~insert('n', 1)
say q~allItems~makeString('L', ',') q~at(2)
call try 'q~at(0)'
call try 'q~at(-1)'
call try 'q~at("x")'
call try 'q~at(.nil)'
call try 'q~at(1.5)'
call try 'q~at(1000000000000000000)'
call try 'q~at(.array~of(2, 1))'
call try 'q~at(.array~of)'
call try 'q~at(.array~of(.array~of(2)))'
call try 'q~at(.list~of(2))'
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
a c 1 0
b b b b 1
z a 1,2,3,4
z,b,c b The NIL object
z,n,b,c n
q~at(0) => 93.907
q~at(-1) => 93.907
q~at("x") => 93.907
q~at(.nil) => 93.907
q~at(1.5) => 93.907
q~at(1000000000000000000) => 93.907
q~at(.array~of(2, 1)) => 93.907
q~at(.array~of) => 93.907
q~at(.array~of(.array~of(2))) => 93.907
q~at(.list~of(2)) => 93.907
```

## Reading and writing items

r[class.queue.at]
`at(index)` answers the item at the index, and `.nil` when the queue has no
item there. `[]` is the same method under another name: `q[i]` is the message
`[]`. The index is read as [the index rule](r:class.queue.indexes) says. The
methods take exactly one argument: none raises `SYNTAX 93.901` and a second
raises `SYNTAX 93.926`. An override of `at` is not asked when `[]` is sent,
nor the other way round.

```rexx
q = .queue~of('a', 'b')
say q~at(1) q[2] (q~at(5) == .nil) (q[5] == .nil)
call try 'q~at'
call try 'q~at(1, 1)'
call try 'q[]'
call try 'q[1, 1]'
x = .twin~new
x~append('p')
say x~at(1) x[1]
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class twin subclass queue
::method at
  return 'at override'
::method '[]'
  return 'bracket override'
```

```output
a b 1 1
q~at => 93.901
q~at(1, 1) => 93.926
q[] => 93.901
q[1, 1] => 93.926
at override bracket override
```

r[class.queue.put]
`put(item, index)` replaces the item at the index and answers no value; the
other items keep their indexes. `q[i] = v` is the message `[]=` and the same
as `put`. The index is read as [the index rule](r:class.queue.indexes) says.
An index above the number of items raises a condition and leaves the queue as
it was: `SYNTAX 93.966` or `SYNTAX 93.918`; which of the two is not specified,
since it depends on room the queue holds in advance. The method takes two
arguments: fewer, or an omitted item or index, raise `SYNTAX 93.903`, and a
third raises `SYNTAX 93.902`. A `put` of a subclass is not asked for `[]=`,
nor the other way round.

```rexx
q = .queue~of('a', 'b', 'c')
q~put('B', 2)
q[3] = 'C'
say q~allItems~makeString('L', ',') q~items
call try 'q~put("x", 0)'
call try 'q~put("x", .nil)'
call try 'q~put("x", "x")'
call try 'q~put("x", .array~of(1, 2))'
call try 'q~put("x")'
call try 'q~put'
call try 'q~put(, 1)'
call try 'q~put("x", 1, 2)'
call try 'q~put("x", 2)'
say q~allItems~makeString('L', ',')
code = .nil
signal on syntax name over
q~put('x', 4)
over: c = condition('o')~code
say (c = 93.966 | c = 93.918) q~items
signal on syntax name over2
.queue~new~put('x', 1)
over2: c = condition('o')~code
say (c = 93.966 | c = 93.918) q~items
signal on syntax name over3
q~put('x', 5000)
over3: c = condition('o')~code
say (c = 93.966 | c = 93.918) q~items
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
a,B,C 3
q~put("x", 0) => 93.907
q~put("x", .nil) => 93.907
q~put("x", "x") => 93.907
q~put("x", .array~of(1, 2)) => 93.907
q~put("x") => 93.903
q~put => 93.903
q~put(, 1) => 93.903
q~put("x", 1, 2) => 93.902
q~put("x", 2) => no error
a,x,C
1 3
1 3
1 3
```

r[class.queue.append]
`append(item)` adds the item at the tail and answers its index, which is the
new number of items. `queue(item)` does the same and answers no value, and
`push(item)` adds the item at the head, so that every item already in the
queue moves to the next index, and answers no value. `.nil` is an item like
any other. Each method takes exactly one argument: none, or an omitted one,
raises `SYNTAX 93.903` and a second raises `SYNTAX 93.902`.

```rexx
q = .queue~new
say q~append('a') q~append('b') q~append(.nil)
q~queue('c')
q~push('z')
say q~allItems~makeString('L', ',') q~items
call try 'x = q~queue("d")'
call try 'x = q~push("d")'
call try 'q~append'
call try 'q~append(, 1)'
call try 'q~append(1, 2)'
call try 'q~queue'
call try 'q~queue(, 1)'
call try 'q~queue(1, 2)'
call try 'q~push'
call try 'q~push(, 1)'
call try 'q~push(1, 2)'
say q~items
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
1 2 3
z,a,b,The NIL object,c 5
x = q~queue("d") => 91.999
x = q~push("d") => 91.999
q~append => 93.903
q~append(, 1) => 93.902
q~append(1, 2) => 93.902
q~queue => 93.903
q~queue(, 1) => 93.902
q~queue(1, 2) => 93.902
q~push => 93.903
q~push(, 1) => 93.902
q~push(1, 2) => 93.902
7
```

r[class.queue.insert]
`insert(item, index)` adds the item as a new entry directly after the item at
the index and answers the index of the new entry. An `index` of `.nil` makes
the item the first, and an omitted `index` the last. The index is read as [the
index rule](r:class.queue.indexes) says, and it must be one of an item: a
number above the number of items raises `SYNTAX 93.966`, and the queue keeps
its items. A third argument raises `SYNTAX 93.902`. The items behind the new
one move to the next index.

```rexx
q = .queue~of('a', 'b', 'c')
say q~insert('x', 1) q~allItems~makeString('L', ',')
say q~insert('y', .nil) q~allItems~makeString('L', ',')
say q~insert('z') q~allItems~makeString('L', ',')
say q~insert('w', q~last) q~allItems~makeString('L', ',')
say q~insert('v', 3) q~allItems~makeString('L', ',')
say q~insert('q', '2') q~insert('q', .array~of(1)) q~items
e = .queue~new
say e~insert('only', .nil) e~insert('end') e~allItems~makeString('L', ',')
n = q~items
call try 'q~insert("q", 0)'
call try 'q~insert("q", "x")'
call try 'q~insert("q", -1)'
call try 'q~insert("q", .array~of(1, 2))'
call try 'q~insert("q", 1, 1)'
call try 'q~insert("q", q~items + 1)'
call try 'q~insert("q", 99)'
call try '.queue~new~insert("q", 1)'
say (q~items == n) q~allItems~makeString('L', ',')
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
2 a,x,b,c
1 y,a,x,b,c
6 y,a,x,b,c,z
7 y,a,x,b,c,z,w
4 y,a,x,v,b,c,z,w
3 2 10
1 2 only,end
q~insert("q", 0) => 93.907
q~insert("q", "x") => 93.907
q~insert("q", -1) => 93.907
q~insert("q", .array~of(1, 2)) => 93.907
q~insert("q", 1, 1) => 93.902
q~insert("q", q~items + 1) => 93.966
q~insert("q", 99) => 93.966
.queue~new~insert("q", 1) => 93.966
1 y,q,a,q,x,v,b,c,z,w
```

r[class.queue.delete]
`delete(index)` and `remove(index)` are the same method, an override of one
not being asked for the other: they take the item at the index out of the
queue and answer it, and the items behind it move to the next lower index. An
index above the number of items answers `.nil` and changes nothing. The
methods take exactly one argument: none raises `SYNTAX 93.903` and a second
raises `SYNTAX 93.902`. An index that [the index rule](r:class.queue.indexes)
refuses raises `SYNTAX 93.907`.

```rexx
q = .queue~of('a', 'b', 'c', 'd')
say q~delete(2) q~items q~allItems~makeString('L', ',')
say q~remove(1) q~items q~allItems~makeString('L', ',')
say q~delete('2') q~remove(.array~of(1)) q~items
say (q~remove(1) == .nil) (q~delete(9) == .nil) q~items
call try 'q~delete'
call try 'q~remove(1, 2)'
call try 'q~delete(.nil)'
call try 'q~remove("x")'
call try 'q~remove(-1)'
call try 'q~delete(0)'
call try 'q~delete(.array~of(1, 2))'
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
b 3 a,c,d
a 2 c,d
d c 0
1 1 0
q~delete => 93.903
q~remove(1, 2) => 93.902
q~delete(.nil) => 93.907
q~remove("x") => 93.907
q~remove(-1) => 93.907
q~delete(0) => 93.907
q~delete(.array~of(1, 2)) => 93.907
```

r[class.queue.removeitem]
`removeItem(item)` removes the first item, from the head, that is equal to the
argument by the strict `==`, and answers the item that was stored, or `.nil`
when there is none; the items behind it move to the next lower index. Its
search is the one of `hasItem` and `index` ([the search
rule](r:class.queue.removeitem.sends)). The method takes exactly one argument:
none, or an omitted one, raises `SYNTAX 93.903` and a second raises
`SYNTAX 93.902`.

```rexx
q = .queue~of('a', 'b', 'a', 1)
say q~removeItem('a') q~allItems~makeString('L', ',')
say q~removeItem('a') q~allItems~makeString('L', ',')
say (q~removeItem('a') == .nil) q~items (q~removeItem(1.0) == .nil)
call try 'q~removeItem'
call try 'q~removeItem(, 1)'
call try 'q~removeItem(1, 2)'
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
a b,a,1
a b,1
1 2 1
q~removeItem => 93.903
q~removeItem(, 1) => 93.902
q~removeItem(1, 2) => 93.902
```

r[class.queue.removeitem.sends]
`removeItem` sends `==` to the argument, once for each item from the head,
with that item as the argument, until one answers true, and sends nothing
else. `hasItem` and `index` make the same search.

```rexx
q = .queue~of(.thing~new('p'), .thing~new('q'), .thing~new('r'))
x = q~removeItem(.thing~new('q'))
say x~name q~items
say q~hasItem(.thing~new('r'))
say q~index(.thing~new('r'))
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
r == p
r == r
1
r == p
r == r
2
```

## Questions about the contents

r[class.queue.hasindex]
`hasIndex(index)` answers 1 when the queue has an item at that index and 0
when it has not, including an index above the number of items. The index is
read as [the index rule](r:class.queue.indexes) says. The method takes exactly
one argument: none raises `SYNTAX 93.901` and a second raises `SYNTAX 93.926`.

```rexx
q = .queue~of('a', 'b')
say q~hasIndex(1) q~hasIndex(2) q~hasIndex(3) q~hasIndex('1')
say q~hasIndex(' 2 ') q~hasIndex(.array~of(2)) .queue~new~hasIndex(1)
call try 'q~hasIndex'
call try 'q~hasIndex(1, 1)'
call try 'q~hasIndex(0)'
call try 'q~hasIndex(-1)'
call try 'q~hasIndex(.nil)'
call try 'q~hasIndex("x")'
call try 'q~hasIndex(.array~of(1, 2))'
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
1 1 0 1
1 1 0
q~hasIndex => 93.901
q~hasIndex(1, 1) => 93.926
q~hasIndex(0) => 93.907
q~hasIndex(-1) => 93.907
q~hasIndex(.nil) => 93.907
q~hasIndex("x") => 93.907
q~hasIndex(.array~of(1, 2)) => 93.907
```

r[class.queue.hasitem]
`hasItem(item)` answers 1 when some item of the queue is equal to the argument
by the strict `==`, as [the search rule](r:class.queue.removeitem.sends) says,
and 0 otherwise. `index(item)` answers the index of the first item from the
head that is equal to the argument, and `.nil` when there is none. Both take
exactly one argument: none, or an omitted one, raises `SYNTAX 93.903` and a
second raises `SYNTAX 93.902`.

```rexx
q = .queue~of(1, 'x', .nil)
say q~hasItem(1) q~hasItem('1') q~hasItem(1.0) q~hasItem('X') q~hasItem(.nil)
say .queue~of(1)~hasItem(.nil)
say q~index(1) q~index('x') q~index(.nil) (q~index('zz') == .nil)
say .queue~of('p', 'q', 'q')~index('q')
call try 'q~hasItem'
call try 'q~hasItem(, 1)'
call try 'q~hasItem(1, 2)'
call try 'q~index'
call try 'q~index(1, 2)'
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
1 2 3 1
2
q~hasItem => 93.903
q~hasItem(, 1) => 93.902
q~hasItem(1, 2) => 93.902
q~index => 93.903
q~index(1, 2) => 93.902
```

## Walking the queue

r[class.queue.first]
`first` answers the index of the first item, which is 1, and `last` the index
of the last, which is the number of items; `firstItem` answers the item at the
head and `lastItem` the item at the tail. `peek` answers what `firstItem`
does. All five answer `.nil` for an empty queue and take no argument: any
argument raises `SYNTAX 93.902`. `pull` takes the item at the head out of the
queue and answers it, and answers `.nil` for an empty queue; it takes no
argument either. The items that were behind the head move to the next lower
index.

```rexx
q = .queue~of('a', 'b', 'c')
say q~first q~last q~firstItem q~lastItem q~peek
e = .queue~new
say (e~first == .nil) (e~last == .nil) (e~firstItem == .nil)
say (e~lastItem == .nil) (e~peek == .nil)
say q~peek q~items
say q~pull q~pull q~items q~allItems~makeString('L', ',')
say q~pull (q~pull == .nil) q~items
call try 'q~first(1)'
call try 'q~last(1)'
call try 'q~firstItem(1)'
call try 'q~lastItem(1)'
call try 'q~peek(1)'
call try 'q~pull(1)'
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
1 3 a c a
1 1 1
1 1
a 3
a b 1 c
c 1 0
q~first(1) => 93.902
q~last(1) => 93.902
q~firstItem(1) => 93.902
q~lastItem(1) => 93.902
q~peek(1) => 93.902
q~pull(1) => 93.902
```

r[class.queue.next]
`next(index)` answers the index after the given one, and `.nil` when that is
the last index or lies above the number of items. `previous(index)` answers
the index before the given one, `.nil` for the index 1, and the last index for
an index above the number of items. For an empty queue both answer `.nil`. The
index is read as [the index rule](r:class.queue.indexes) says. The methods
take exactly one argument: none raises `SYNTAX 93.901` and a second raises
`SYNTAX 93.926`.

```rexx
q = .queue~of('a', 'b', 'c')
say q~next(1) q~next(2) (q~next(3) == .nil) (q~next(4) == .nil)
say (q~next(99) == .nil) q~next('1') q~next(.array~of(2))
say (q~previous(1) == .nil) q~previous(2) q~previous(3)
say q~previous(4) q~previous(99)
e = .queue~new
say (e~next(1) == .nil) (e~previous(1) == .nil)
call try 'q~next'
call try 'q~previous'
call try 'q~next(1, 1)'
call try 'q~previous(1, 1)'
call try 'q~next(0)'
call try 'q~previous(0)'
call try 'q~next(-1)'
call try 'q~next(.nil)'
call try 'q~previous("x")'
call try 'q~previous(1.5)'
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
2 3 1 1
1 2 3
1 1 2
3 3
1 1
q~next => 93.901
q~previous => 93.901
q~next(1, 1) => 93.926
q~previous(1, 1) => 93.926
q~next(0) => 93.907
q~previous(0) => 93.907
q~next(-1) => 93.907
q~next(.nil) => 93.907
q~previous("x") => 93.907
q~previous(1.5) => 93.907
```

## Size and emptying

r[class.queue.items]
`items` answers the number of items, and `size` is the same method under
another name: an override of one is not asked for the other. `isEmpty` answers
1 for a queue without items and 0 otherwise. `empty` takes every item out and
answers the receiver. The four take no argument: any argument raises
`SYNTAX 93.902`.

```rexx
q = .queue~of('a', 'b')
say q~items q~size q~isEmpty
say (q~empty == q) q~items q~size q~isEmpty q~append('c')
call try 'q~items(1)'
call try 'q~size(1)'
call try 'q~isEmpty(1)'
call try 'q~empty(1)'
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
2 2 0
1 0 0 1 1
q~items(1) => 93.902
q~size(1) => 93.902
q~isEmpty(1) => 93.902
q~empty(1) => 93.902
```

## Taking part of a queue

r[class.queue.section]
`section(start, count)` answers a new queue that holds the items from the
index `start` on: `count` of them, or all of them up to the tail when `count`
is omitted or runs past the tail. A `count` of 0, or a `start` above the
number of items, gives an empty queue. The new queue has its own indexes from
1, the items are the receiver's own objects, and the receiver is not changed.
The `start` is read as [the index rule](r:class.queue.indexes) says; the
`count` must be a whole number that is not negative, read as the hint of
[`new`](r:class.queue.new) is, or `SYNTAX 93.923` is raised. No `start`, or an
omitted one, raises `SYNTAX 93.903`, and a third argument `SYNTAX 93.902`. The
answer is an instance of the class of the receiver, so that a subclass gets a
queue of its own class back.

```rexx
q = .queue~of('a', 'b', 'c', 'd')
s = q~section(2)
say s~class~id s~allItems~makeString('L', ',')
say s~allIndexes~makeString('L', ',')
say q~section(2, 2)~allItems~makeString('L', ',')
say q~section(2, 99)~items q~section(2, 0)~items q~section(4)~items
say q~section(5)~items q~section(9)~items q~section(9, 2)~items
say q~section('2', ' 2 ')~items q~section(.array~of(3))~items
s~append('z')
say q~items s~items
call try 'q~section'
call try 'q~section(0)'
call try 'q~section(.nil)'
call try 'q~section("x")'
call try 'q~section(1.5)'
call try 'q~section(.array~of(1, 2))'
call try 'q~section(1, -1)'
call try 'q~section(1, "x")'
call try 'q~section(1, .nil)'
call try 'q~section(1, 1.5)'
call try 'q~section(1, 1, 1)'
y = .many~of(1, 2, 3)
say y~section(2)~class~id y~section(2)~allItems~makeString('L', ',')
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass queue
```

```output
Queue b,c,d
1,2,3
b,c
3 0 1
0 0 0
2 2
4 4
q~section => 93.903
q~section(0) => 93.907
q~section(.nil) => 93.907
q~section("x") => 93.907
q~section(1.5) => 93.907
q~section(.array~of(1, 2)) => 93.907
q~section(1, -1) => 93.923
q~section(1, "x") => 93.923
q~section(1, .nil) => 93.923
q~section(1, 1.5) => 93.923
q~section(1, 1, 1) => 93.902
MANY 2,3
```

r[class.queue.section.new]
The answer is made by a `new` that the receiver's class is sent with one
argument, a whole number that is a hint of the size, and the `init` of that
new object receives it. The value of the hint is not specified.

```rexx
.local~on = 0
y = .many~of(1, 2, 3, 4, 5)
.local~on = 1
s = y~section(2)
say s~class~id s~items
s = y~section(2, 2)
say s~items
::class many subclass queue
::method new class
  if .local~on then say 'new' arg()
  forward class (super)
::method init
  if .local~on then say 'init' arg()
  forward class (super)
```

```output defect=#9001
new 1
init 1
MANY 4
new 1
init 1
2
```

## The items, the indexes and a supplier

r[class.queue.makearray]
`makeArray` and `allItems` answer a new `Array` that holds the items of the
queue from the head, the first at index 1, and `allIndexes` a new `Array` of
the indexes 1 to the number of items. The answers are instances of `Array`
whatever the class of the receiver is, never the receiver, and do not follow
later changes to the queue. They take no argument: any argument raises
`SYNTAX 93.902`.

```rexx
q = .queue~of('a', 'b', 'c')
q~insert('x', 1)
say q~makeArray~makeString('L', ',') q~allItems~makeString('L', ',')
say q~allIndexes~makeString('L', ',') q~allItems~class~id
items = q~allItems
q~append('d')
say items~items q~items
say .many~of(1)~makeArray~class~id .many~of(1)~allIndexes~class~id
say .queue~new~makeArray~items .queue~new~allIndexes~items
call try 'q~makeArray(1)'
call try 'q~allItems(1)'
call try 'q~allIndexes(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass queue
```

```output
a,x,b,c a,x,b,c
1,2,3,4 Array
4 5
Array Array
0 0
q~makeArray(1) => 93.902
q~allItems(1) => 93.902
q~allIndexes(1) => 93.902
```

r[class.queue.supplier]
`supplier` answers a `Supplier` whose indexes and items are those of the queue
from the head. It is a snapshot: items added, removed or replaced afterwards
do not change it. An empty queue gives a supplier that is not available. The
method takes no argument: any argument raises `SYNTAX 93.902`. The `Supplier`
class has its own page.

```rexx
q = .queue~of('a', 'b')
s = q~supplier
q~append('c')
q~put('B', 2)
do while s~available
  say s~index s~item
  s~next
end
say .queue~new~supplier~available
call try 'q~supplier(1)'
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
1 a
2 b
0
q~supplier(1) => 93.902
```

## Copies and the string value

r[class.queue.copy]
`copy` answers a new queue of the same class as the receiver, with the same
items in the same order, so that neither queue sees the other's changes. The
items are the same objects: the copy is shallow. As [the copy
rule](r:class.object.copy) says of every object, the variables of a subclass
instance are copied as they are and the name stays; `init` is not sent. The
method takes no argument: one raises `SYNTAX 93.902`.

```rexx
q = .many~of('a', 'b', 'c')
q~tag = 'T'
q~remove(1)
c = q~copy
say c~class~id c~items c~tag c~allItems~makeString('L', ',') (c == q)
c~append('n')
q~append('m')
c~append('o')
say q~items c~items q~lastItem c~lastItem
call try 'q~copy(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass queue
::attribute tag
::method init
  say 'init'
  forward class (super)
```

```output
init
MANY 2 T b,c 0
3 4 m o
q~copy(1) => 93.902
```

r[class.queue.string]
A queue has no `makeString` method, so its string value is the one of
`Object`: its name, `a Queue`, as [the string rule](r:class.index.string)
says, whatever the queue holds; `request('STRING')` answers `.nil`. Use
`makeArray` to turn the items into an array.

```rexx
q = .queue~of('a', 'b')
say q '|' q~string '|' q 'x'
say (q~request('STRING') == .nil) .queue~new
```

```output
a Queue | a Queue | a Queue x
1 a Queue
```

## Sorting

r[class.queue.sort]
`sort`, `stableSort`, `sortWith(comparator)` and `stableSortWith(comparator)`
of a queue are the methods of [`OrderedCollection`](orderedcollection.md), and
they sort the queue in place and answer the receiver. The items change places
among the indexes, which stay 1 to the number of items: after a sort the head
holds the first item in the new order, and so on. They are written in terms of
other messages, as [the sort rule](r:class.orderedcollection.sort) says: the
queue's `makeArray`, `first`, `next` and `[]=`, and the `sort` or `sortWith`,
`size` and `[]` of the array that `makeArray` answers, which compares the
items as [the array rule](r:class.array.sort) says. The methods check their
arguments as [the sort rule](r:class.orderedcollection.sort) says: `sort` and
`stableSort` take none, and `sortWith` and `stableSortWith` take one,
`SYNTAX 93.901` being raised when it is missing; the comparator is not looked
at, so the string `"x"` is accepted by a queue of one item.

```rexx
q = .queue~of('c', 'a', 'b')
say (q~sort == q) q~allItems~makeString('L', ',')
say q~allIndexes~makeString('L', ',')
say q~sortWith(.desc~new)~allItems~makeString('L', ',') q~first
say (q~stableSort == q) (q~stableSortWith(.desc~new) == q) q~lastItem
say .queue~new~sort~items .queue~of('z')~sortWith('x')~items
call try 'q~sort(1)'
call try 'q~stableSort(1)'
call try 'q~sortWith'
call try 'q~stableSortWith(1, 2)'
call try 'q~sortWith(.nil)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
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
1,2,3
c,b,a 1
1 1 a
0 1
q~sort(1) => 93.902
q~stableSort(1) => 93.902
q~sortWith => 93.901
q~stableSortWith(1, 2) => 93.902
q~sortWith(.nil) => 97.1
```

r[class.queue.sort.sends]
The messages of [the sort rule](r:class.queue.sort) go to the receiver and to
the array that `makeArray` answered, in the order that the example shows.

```rexx
.local~on = 0
l = .spy~of('b', 'a')
.local~on = 1
l~sort
say '--'
l~sortWith(.cmp~new)
::class cmp
::method compare
  use arg x, y
  return x~compareTo(y)
::class spy subclass queue
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
[]= a 1
next 1
array [](2)
[]= b 2
next 2
--
sortWith CMP
stableSortWith CMP
makeArray 0
array SORTWITH(a CMP)
first 0
array SIZE()
array [](1)
[]= a 1
next 1
array [](2)
[]= b 2
next 2
```

## The methods of the mixins

r[class.queue.inherited]
The methods `union`, `difference`, `intersection`, `xor`, `subset`, `disjoint`
and `equivalent` and `appendAll` are not declared by `Queue`. The methods
`union`, `difference`, `intersection`, `xor`, `subset` and `appendAll` are
those of [`OrderedCollection`](orderedcollection.md), and `disjoint` and
`equivalent` those of [`Collection`](collection.md), as the pages of the
mixins say, and a queue is one of the collections they take as an argument, so
that `.queue~of(1)~union(.list~of(2))` is a queue of 1 and 2. A queue answers
0 to `isA` with `Array`, so `union`, `difference` and `xor` answer the copies
they work on, and `intersection` the collection that `new` of the receiver's
class made; all four are instances of the class of the receiver, and
`appendAll` answers the receiver.

```rexx
a = .many~of(1, 2, 2)
b = .array~of(2, 3)
say a~union(b)~class~id a~union(b)~makeArray~makeString('L', ',')
say a~difference(b)~class~id a~difference(b)~makeArray~makeString('L', ',')
i = a~intersection(b)
say i~class~id i~makeArray~makeString('L', ',')
say a~xor(b)~class~id a~xor(b)~makeArray~makeString('L', ',')
say a~subset(b) a~disjoint(b) a~equivalent(b) a~equivalent(.queue~of(2, 1, 2))
say (a~appendAll(b) == a) a~makeArray~makeString('L', ',')
say .queue~of(1)~union(.list~of(2))~makeArray~makeString('L', ',')
::class many subclass queue
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

r[class.queue.sends]
No method that `Queue` declares sends a message to its receiver: an override
of `at`, `put`, `append`, `queue` or any other of the methods is never asked
by another method, and `[]` and `[]=` do not use `at` and `put`. The messages
the methods do send go to other objects, and each has a rule of its own:
`init` to the new object by `new` and `of` ([the new rule](r:class.queue.new),
[the of rule](r:class.queue.of)), `new` and `init` by `section` ([the section
rule](r:class.queue.section)); `==` to the argument by `removeItem`, `hasItem`
and `index` ([the search rule](r:class.queue.removeitem.sends)). The sorts,
which are not methods of the class, send messages to the receiver and to the
array of `makeArray`, as [their rule](r:class.queue.sort.sends) says. The
methods `string` and `copy` of an instance and the set operations send what
the pages that own them say. No argument is asked `isA`, and a string value is
read as [the conversion rule](r:conv.string.object) says. In the example a
subclass has every method of `Queue` overridden to record its name, and each
call records only its own.

```rexx
own = '[] []= allIndexes allItems append at delete empty first firstItem',
  'hasIndex hasItem index insert isEmpty items last lastItem makeArray next',
  'previous put remove removeItem section supplier copy peek pull push queue',
  'size'
.local~log = .array~new
do m over own~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"'); forward class (super)"
  .spy~define(m, src)
end
a = .spy~of('c', 'b', 'a')
calls = 'x = a[1]|a[1] = "x"|a~allIndexes|a~allItems|a~append("d")|a~at(1)',
  '|a~insert("e", 1)|a~hasIndex(1)|a~hasItem("q")|a~index("q")|a~isEmpty',
  '|a~items|a~first|a~last|a~firstItem|a~lastItem|a~makeArray|a~next(1)',
  '|a~previous(2)|a~put("y", 1)|a~section(1)|a~supplier|a~copy|a~peek|a~size',
  '|a~queue("k")|a~push("j")|a~pull|a~removeItem("q")|a~remove(1)',
  '|a~delete(1)|a~empty'
n = 0
extra = 0
.local~on = 0
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
::class spy subclass queue
```

```output
32 calls, 0 sent more than one message to the receiver
```

## Deviations

r[class.queue.deviation.109]
An index or a count that is an instance of a subclass of `String` is read by
its own characters, and `MAKESTRING` is not sent to it ([Deviation
109](r:class.string.deviation.109)). ooRexx reads such an instance as an index
through its `MAKESTRING` method. In the example `MAKESTRING` answers another
index.

```rexx
q = .queue~of('first', 'second')
say q~at(.num~new('1'))
::class num subclass string
::method makestring
  return '2'
```

```output
first
```
