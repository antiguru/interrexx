---
element: "Array"
kind: class
rexxref: "Builtin Classes > Collection Classes > Array Class; Builtin Classes > Collection Classes > Array Class > new (Class Method); Builtin Classes > Collection Classes > Array Class > of (Class Method); Builtin Classes > Collection Classes > Array Class > []; Builtin Classes > Collection Classes > Array Class > []=; Builtin Classes > Collection Classes > Array Class > allIndexes; Builtin Classes > Collection Classes > Array Class > allItems; Builtin Classes > Collection Classes > Array Class > append; Builtin Classes > Collection Classes > Array Class > at; Builtin Classes > Collection Classes > Array Class > delete; Builtin Classes > Collection Classes > Array Class > dimension; Builtin Classes > Collection Classes > Array Class > dimensions; Builtin Classes > Collection Classes > Array Class > empty; Builtin Classes > Collection Classes > Array Class > fill; Builtin Classes > Collection Classes > Array Class > first; Builtin Classes > Collection Classes > Array Class > firstItem; Builtin Classes > Collection Classes > Array Class > hasIndex; Builtin Classes > Collection Classes > Array Class > hasItem; Builtin Classes > Collection Classes > Array Class > index; Builtin Classes > Collection Classes > Array Class > insert; Builtin Classes > Collection Classes > Array Class > isEmpty; Builtin Classes > Collection Classes > Array Class > items; Builtin Classes > Collection Classes > Array Class > last; Builtin Classes > Collection Classes > Array Class > lastItem; Builtin Classes > Collection Classes > Array Class > makeArray; Builtin Classes > Collection Classes > Array Class > makeString; Builtin Classes > Collection Classes > Array Class > next; Builtin Classes > Collection Classes > Array Class > previous; Builtin Classes > Collection Classes > Array Class > put; Builtin Classes > Collection Classes > Array Class > remove; Builtin Classes > Collection Classes > Array Class > removeItem; Builtin Classes > Collection Classes > Array Class > section; Builtin Classes > Collection Classes > Array Class > size; Builtin Classes > Collection Classes > Array Class > sort; Builtin Classes > Collection Classes > Array Class > sortWith; Builtin Classes > Collection Classes > Array Class > stableSort; Builtin Classes > Collection Classes > Array Class > stableSortWith; Builtin Classes > Collection Classes > Array Class > supplier; Builtin Classes > Collection Classes > Array Class > toString"
deviations: [107, 109]
---

# Array

An `Array` is an ordered collection whose entries are addressed by positive
whole numbers. Its indexes may have gaps, so an array can hold items at 1 and
5 and nothing between, and it may have more than one dimension, in which case
an entry is addressed by one whole number for each dimension. The class
inherits [`OrderedCollection`](orderedcollection.md), and through it
[`Collection`](collection.md), and declares most of their methods itself; [the
hierarchy rule](r:class.index.hierarchy) lists what it inherits.

The page follows the order of the rexxref entry: the class, the way indexes
are read, the methods that read and write entries, the questions about the
contents, the methods that make other collections, and the ones that sort or
print. Examples that show an error use a small routine `try`, as the builtin
pages do. No method that the class declares sends another message to its
receiver; [the sends rule](r:class.array.sends) lists the messages that
methods do send, and to whom.

## The class

r[class.array.creation]
`Array` declares thirty-six instance methods and two class methods, `new` and
`of`. The instance methods are `[]`, `[]=`, `allIndexes`, `allItems`,
`append`, `at`, `delete`, `dimension`, `dimensions`, `empty`, `fill`, `first`,
`firstItem`, `hasIndex`, `hasItem`, `index`, `insert`, `isEmpty`, `items`,
`last`, `lastItem`, `makeArray`, `makeString`, `next`, `previous`, `put`,
`remove`, `removeItem`, `section`, `size`, `sort`, `sortWith`, `stableSort`,
`stableSortWith`, `supplier` and `toString`, and a `copy` that this page
specifies ([the copy rule](r:class.array.copy)). The set operations `union`,
`difference`, `intersection`, `xor`, `subset`, `disjoint` and `equivalent` and
the method `appendAll` are those of the mixins, and [one
rule](r:class.array.inherited) lists them. A program subclasses `Array` as it
does any class; the rules below say what each method answers for an instance
of a subclass.

```rexx
a = .array~new
own = 'AT PUT APPEND INSERT SECTION SORT TOSTRING SUPPLIER DIMENSIONS'
n = 0
do m over own~makearray(' ')
  if a~hasMethod(m) then n += 1
end
say a~class~id n .array~hasMethod('OF') a~hasMethod('OF')
say a~hasMethod('UNION') a~hasMethod('APPENDALL') a~hasMethod('SIZE')
```

```output
Array 9 1 0
1 1 1
```

## Making an array

r[class.array.new]
`new` makes an empty array. The arguments say how large the dimensions are
expected to be, and they are hints only: the array grows as entries are put
into it. With no argument the array has no dimension yet; its `dimension` is 0
and its `size` is 0, and the first `put` gives it as many dimensions as the
put has indexes. With one or more arguments, each a whole number that is not
negative, the array has one dimension for each argument, of that size, and its
`size` is the product of them, though it holds no item (`items` is 0). One
argument that is an `Array` of such numbers gives the same dimensions as
arguments would; an instance of a subclass is not such an array. An argument
that is not a whole number that is not negative raises `SYNTAX 93.906`, and so
does an array of sizes that holds one, or an instance of a subclass; an
omitted argument followed by a given one raises `SYNTAX 93.903`, while omitted
arguments at the end count for nothing. A size of 0 is allowed and makes a
dimension with no room: `new(0)` has `dimension` 1 and `size` 0. A size is
read as [the index rule](r:class.array.indexes) reads an index.

```rexx
a = .array~new
say a~dimension a~size a~items
b = .array~new(3)
say b~dimension b~size b~items
c = .array~new(2, 3)
say c~dimension c~size c~items c~dimensions~makeString('L', 'x')
d = .array~new(.array~of(2, 3, 4))
say d~dimension d~size
say .array~new(0)~dimension .array~new(0)~size .array~new(2, )~dimension
call try '.array~new(-1)'
call try '.array~new("x")'
call try '.array~new(1.5)'
call try '.array~new(.nil)'
call try '.array~new(.array~of(2, "x"))'
call try '.array~new(, 2)'
call try '.array~new(.pair~of(2, 3))'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class pair subclass array
```

```output
0 0 0
1 3 0
2 6 0 2x3
3 24
1 0 1
.array~new(-1) => 93.906
.array~new("x") => 93.906
.array~new(1.5) => 93.906
.array~new(.nil) => 93.906
.array~new(.array~of(2, "x")) => 93.906
.array~new(, 2) => 93.903
.array~new(.pair~of(2, 3)) => 93.906
```

r[class.array.new.sends]
`new` sends `init` to the new object with no argument, whatever arguments the
`new` message had: the sizes are read by `new` itself and `init` never sees
them. A subclass that overrides the class method `new` is sent it with the
arguments as given, as for any class ([the new rule](r:class.index.new)). `of`
is a class method that does not send `new`; it sends `init` with no argument
and then places the items itself, so that `put` and `append` of a subclass are
not sent for them.

```rexx
a = .watch~new(5)
b = .watch~of(1, 2)
say b~items
::class watch subclass array
::method new class
  say 'new' arg()
  forward class (super)
::method init
  say 'init' arg()
  forward class (super)
::method put
  say 'put'
  forward class (super)
::method append
  say 'append'
  forward class (super)
```

```output
new 1
init 0
init 0
2
```

r[class.array.of]
`of` makes a one-dimensional array that holds its arguments, the first at
index 1, the second at index 2 and so on. An argument left out leaves its
index without an item, and arguments left out at the end count for nothing, so
`of(1, , 3, )` has three indexes. `of` with no argument makes an empty array
that has one dimension, unlike `new`. An argument that is `.nil` is an item
like any other. The result is an instance of the class that received the
message.

```rexx
a = .array~of('x', , 'z', )
say a~size a~items a~dimension a~hasIndex(2)
b = .array~of
say b~size b~dimension
c = .array~of(.nil, .nil)
say c~items (c~at(1) == .nil)
say .array~of(, 'b')~first .array~of(, 'b')~size
```

```output
3 2 1 0
0 1
2 1
2 2
```
## Indexes

r[class.array.indexes]
A method that takes an index takes it as one argument for each dimension, or
as one `Array` that holds them all; `at(2, 3)` and `at(.array~of(2, 3))` name
the same entry. Only an instance of `Array` itself is such a list: an instance
of a subclass is one argument like any other object. Each index is a whole
number of at least 1, read from the string value of the argument ([the
conversion rule](r:conv.string.object)) as a whole-number argument of a builtin
is ([the form rule](r:conv.whole.form), [the digits
rule](r:conv.whole.digits)), so `"2"`, `' + 2 '` and `2.0` are as good as `2`
and NUMERIC DIGITS plays no part; an argument those rules refuse raises the
error this rule names, not `SYNTAX 40.12`. The number of indexes must be the
number of dimensions of the array: too
few raise `SYNTAX 93.925` and too many `SYNTAX 93.926`. An index that is not a
whole number of at least 1 raises `SYNTAX 93.907` when the array has one
dimension, and `SYNTAX 93.924` when it has more. No index at all, or an empty
array of indexes, raises `SYNTAX 93.901`. An array that has no dimension yet
(one made by `new` without arguments and not yet put into) has no entries, and
a message that reads, with two or more indexes, answers as for an index that
has no entry without checking them; with one index it is checked as for a
one-dimensional array. The sections below say where a method checks fewer or
more than this.

```rexx
a = .array~new(2, 2)
a[2, 1] = 'x'
say a[2, 1] a[.array~of(2, 1)] a~at('2', 1.0)
call try 'a~at(1)'
call try 'a~at(1, 1, 1)'
call try 'a~at(0, 1)'
call try 'a~at(1, "q")'
call try '.array~of(7)~at(0)'
call try '.array~of(7)~at(.nil)'
call try '.array~of(7)~at(1, 1)'
call try '.array~of(7)~at'
call try '.array~of(7)~at(.array~of)'
call try '.array~of(7)~at(1000000000000000000)'
say .array~new~at(0, 0) .array~new~at(5, 5, 5)
call try 'a~at(.pair~of(2, 1))'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class pair subclass array
```

```output
x x x
a~at(1) => 93.925
a~at(1, 1, 1) => 93.926
a~at(0, 1) => 93.924
a~at(1, "q") => 93.924
.array~of(7)~at(0) => 93.907
.array~of(7)~at(.nil) => 93.907
.array~of(7)~at(1, 1) => 93.926
.array~of(7)~at => 93.901
.array~of(7)~at(.array~of) => 93.901
.array~of(7)~at(1000000000000000000) => 93.907
The NIL object The NIL object
a~at(.pair~of(2, 1)) => 93.925
```

## Reading and writing entries

r[class.array.at]
`at(index)` answers the item at the index, and `.nil` when the index has no
item, whether the slot lies beyond the array or inside it. The indexes are
read as [the index rule](r:class.array.indexes) says. `[]` is the same method
under another name: `a[i]` is the message `[]` with the indexes as arguments,
and `a~"[]"(i)` the same message written out. A subclass that overrides `at`
is not asked when `[]` is sent, nor the other way round.

```rexx
a = .array~of('a', , 'c')
say a~at(1) a[3] (a~at(2) == .nil) (a[9] == .nil)
b = .twin~new
b~put('p', 1)
say b~at(1) b[1]
::class twin subclass array
::method at
  return 'at override'
::method '[]'
  return 'bracket override'
```

```output
a c 1 1
at override bracket override
```

r[class.array.put]
`put(item, index)` makes the item the entry at the index, replacing any item
there, and answers no value. `a[i] = v` is the message `[]=` with the item
first and the indexes after, and it is `put` under another name. The method
needs at least the item and one index: fewer arguments raise `SYNTAX 93.901`,
and an omitted item raises `SYNTAX 93.903`; both come before any check of the
indexes. The indexes are read as [the index rule](r:class.array.indexes) says.
A put beyond the present size of a dimension makes the dimension as large as
the index, and the array grows. The first put into an array that has no
dimension yet gives it as many dimensions as it has indexes, each as large as
its index. `.nil` is an item like any other. A subclass's own `put` is not
asked for `[]=`, nor the other way round.

```rexx
a = .array~new
a~put('x', 3)
say a~dimension a~size a~items
b = .array~new
b[2, 3] = 'y'
say b~dimension b~dimensions~makeString('L', 'x') b~items b[2, 3]
b~put('z', 4, 1)
say b~dimensions~makeString('L', 'x') b~items
c = .array~of(1, 2)
c~put(.nil, 1)
say c~items (c[1] == .nil)
c~put(9, 2)
say c[2]
call try 'c~put'
call try 'c~put(1)'
call try 'c~put(, 1)'
call try 'c~put(, 0)'
call try 'c~put(1, 0)'
call try 'c~put(1, 1, 1)'
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
1 3 1
2 2x3 1 y
4x3 2
2 1
9
c~put => 93.901
c~put(1) => 93.901
c~put(, 1) => 93.903
c~put(, 0) => 93.903
c~put(1, 0) => 93.907
c~put(1, 1, 1) => 93.926
```
## Adding and removing items

r[class.array.append]
`append(item)` puts the item at the index after the highest index that holds
an item, and answers that index. In an array with no item the index is 1, and
an array that has no dimension yet becomes one-dimensional. The size grows
when the index is beyond it. The method takes exactly one argument: none, or
an omitted one, raises `SYNTAX 93.903` and a second argument raises
`SYNTAX 93.902`; an array of more than one dimension raises `SYNTAX 93.954`,
after the argument count has been checked. `.nil` is an item like any other.

```rexx
a = .array~of('a', 'b', , )
say a~append('c') a~size
a~remove(3)
say a~append('d') a~allItems~makeString('L', ',')
b = .array~new(5)
say b~append('x') b~size b~dimension
c = .array~new
say c~append(.nil) c~dimension c~items
call try '.array~new~append'
call try '.array~new~append(1, 2)'
call try '.array~new(2, 2)~append(1)'
call try '.array~new(2, 2)~append'
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
3 3
3 a,b,d
1 5 1
1 1 1
.array~new~append => 93.903
.array~new~append(1, 2) => 93.902
.array~new(2, 2)~append(1) => 93.954
.array~new(2, 2)~append => 93.903
```

r[class.array.insert]
`insert(item, index)` puts the item at the index after `index`: it takes the
place after the entry at `index`, every entry after that moves up by one
index, and the method answers the index of the new item. An `index` of `.nil`
puts the item first, at index 1, and an omitted `index` puts it after the
highest index that holds an item. An `index` beyond the highest index of the
array is allowed: the item goes to the index after it. The size of the array
becomes the larger of what it was and the highest index the insert uses, the
new one's or that of an entry it moved, so it grows only when an entry is
pushed past its last place. An omitted item, or no argument at all, makes the
new slot empty, and the answer, the size and the moving of the entries after
it are as for an item ([the omitted item rule](r:class.array.insert.omitted)).
The index is read as [the index rule](r:class.array.indexes) says, so an array
of one index is accepted. The
method takes one or two arguments: a third raises `SYNTAX 93.902`; an array of
more than one dimension raises `SYNTAX 93.954` whatever the index is, after
the argument count has been checked. An index that is not a whole number of at
least 1 and is not `.nil` raises `SYNTAX 93.907`.

```rexx
a = .array~of('a', 'b', 'c')
say a~insert('x', 1) a~allItems~makeString('L', ',')
say a~insert('y', .nil) a~allItems~makeString('L', ',')
say a~insert('z') a~allItems~makeString('L', ',')
say a~insert('w', 9) a~size a~allIndexes~makeString('L', ',')
b = .array~new(5)
b[1] = 'p'
b[2] = 'q'
say b~insert('r', 1) b~size b~allItems~makeString('L', ',')
c = .array~new
say c~insert('s', 1) c~size c~dimension
call try '.array~of(1)~insert(1, 0)'
call try '.array~of(1)~insert(1, "x")'
call try '.array~of(1)~insert(1, 1, 1)'
call try '.array~new(2, 2)~insert(1, 0)'
call try '.array~new(2, 2)~insert(1, 1, 1)'
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
10 10 1,2,3,4,5,6,10
2 5 p,r,q
2 2 1
.array~of(1)~insert(1, 0) => 93.907
.array~of(1)~insert(1, "x") => 93.907
.array~of(1)~insert(1, 1, 1) => 93.902
.array~new(2, 2)~insert(1, 0) => 93.954
.array~new(2, 2)~insert(1, 1, 1) => 93.902
```

r[class.array.insert.omitted]
An `insert` whose item is omitted makes an empty slot where the item would
go: the entries after it move up, the answer is the index of the empty slot,
and the size grows to that index when it lies beyond the size.

```rexx
a = .array~of('a')
say a~insert(, 2) a~size a~items a~hasIndex(3)
b = .array~of('a', 'b')
say b~insert(, 1) b~size b~items b~hasIndex(2) b[3]
c = .array~of('a')
say c~insert() c~size c~items c~hasIndex(2)
```

```output defect=#161
3 3 1 0
2 3 2 0 b
2 2 1 0
```

r[class.array.delete]
`delete(index)` takes the entry at the index out of the array and answers its
item; every entry after it moves down by one index and the size drops by one.
When the slot at the index is empty the answer is `.nil` and the slot is still
taken out. An index above the size answers `.nil` and changes nothing. The
method takes one argument, which may be an array of one index: none, or an
omitted one, raises `SYNTAX 93.903` and a second raises `SYNTAX 93.902`. An
array of more than one dimension raises `SYNTAX 93.954` whatever the index is,
after the argument count has been checked; otherwise the index is read as [the
index rule](r:class.array.indexes) says, with `SYNTAX 93.907` for an index
that is not a whole number of at least 1 and `SYNTAX 93.926` for an array of
two.

```rexx
a = .array~of('a', 'b', 'c', 'd')
say a~delete(2) a~size a~allItems~makeString('L', ',')
say a~delete(9) a~size
b = .array~of('p', , 'r')
say (b~delete(2) == .nil) b~size b~items
c = .array~new(5)
say (c~delete(2) == .nil) c~size (c~delete(6) == .nil) c~size
say a~delete(.array~of(1)) a~allItems~makeString('L', ',')
call try 'a~delete'
call try 'a~delete(1, 2)'
call try 'a~delete(0)'
call try 'a~delete("x")'
call try 'a~delete(.array~of(1, 2))'
call try '.array~new(2, 2)~delete(1)'
call try '.array~new(2, 2)~delete(1, 2)'
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
The NIL object 3
1 2 2
1 4 1 4
a c,d
a~delete => 93.903
a~delete(1, 2) => 93.902
a~delete(0) => 93.907
a~delete("x") => 93.907
a~delete(.array~of(1, 2)) => 93.926
.array~new(2, 2)~delete(1) => 93.954
.array~new(2, 2)~delete(1, 2) => 93.902
```

r[class.array.remove]
`remove(index)` takes the item at the index out of its slot and answers it,
and leaves the slot empty: no other index changes and the size does not. When
the index has no item, because the slot is empty or lies beyond the array, the
answer is `.nil` and nothing changes. It works on arrays of any number of
dimensions, so the indexes are read as [the index rule](r:class.array.indexes)
says, and it checks no argument count of its own: no argument raises
`SYNTAX 93.901`, and two on a one-dimensional array raise `SYNTAX 93.926`.

```rexx
a = .array~of('a', 'b', 'c')
say a~remove(2) a~size a~items a~hasIndex(2) (a~remove(2) == .nil)
say a~remove(9) a~size
b = .array~new(2, 2)
b[2, 1] = 'x'
say b~remove(.array~of(2, 1)) b~items b~size
call try 'a~remove'
call try 'a~remove(1, 2)'
call try 'a~remove(0)'
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
b 3 2 0 1
The NIL object 3
x 0 4
a~remove => 93.901
a~remove(1, 2) => 93.926
a~remove(0) => 93.907
```

r[class.array.removeitem]
`removeItem(item)` takes out the first item that is equal to the argument, in
index order, leaves its slot empty as `remove` does, and answers the item that
was stored, which is `.nil` when there is none. Equality is the strict `==`,
so `1` and `1.0` differ. The method takes one argument: none, or an omitted
one, raises `SYNTAX 93.903` and a second raises `SYNTAX 93.902`; it works on
arrays of any number of dimensions.

```rexx
a = .array~of('a', 'b', 'a', 1)
say a~removeItem('a') a~allIndexes~makeString('L', ',')
say a~removeItem('a') a~allIndexes~makeString('L', ',')
say (a~removeItem('a') == .nil) a~size
say (a~removeItem(1.0) == .nil) a~items
call try 'a~removeItem'
call try 'a~removeItem(1, 2)'
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
a 2,3,4
a 2,4
1 4
1 2
a~removeItem => 93.903
a~removeItem(1, 2) => 93.902
```

r[class.array.removeitem.sends]
`removeItem` sends `==` to the argument, once for each item in index order,
with that item as the argument, until one answers true, and sends nothing
else. `hasItem` and `index` make the same search, and their rules say so.

```rexx
a = .array~of(.thing~new('p'), .thing~new('q'), .thing~new('r'))
x = a~removeItem(.thing~new('q'))
say x~name a~items
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

r[class.array.hasindex]
`hasIndex(index)` answers 1 when the index holds an item and 0 when it does
not, because the slot is empty, lies beyond the array, or the array has no
dimension yet. A slot that holds `.nil` holds an item. The indexes are read as
[the index rule](r:class.array.indexes) says, so the method takes no argument
count of its own: no argument raises `SYNTAX 93.901`, and an index that is not
a whole number of at least 1 raises `SYNTAX 93.907`, or `SYNTAX 93.924` in a
multi-dimensional array.

```rexx
a = .array~of('a', , .nil)
say a~hasIndex(1) a~hasIndex(2) a~hasIndex(3) a~hasIndex(4)
say .array~new~hasIndex(1) .array~new(3)~hasIndex(1)
b = .array~new(2, 3)
b[2, 3] = 'x'
say b~hasIndex(2, 3) b~hasIndex(.array~of(2, 3)) b~hasIndex(1, 1)
call try 'a~hasIndex'
call try 'a~hasIndex(0)'
call try 'a~hasIndex(.nil)'
call try 'b~hasIndex(1)'
call try 'b~hasIndex(0, 1)'
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
1 1 0
a~hasIndex => 93.901
a~hasIndex(0) => 93.907
a~hasIndex(.nil) => 93.907
b~hasIndex(1) => 93.925
b~hasIndex(0, 1) => 93.924
```

r[class.array.hasitem]
`hasItem(item)` answers 1 when some item of the array is equal to the argument
by the strict `==`, as [the search rule](r:class.array.removeitem.sends) says,
and 0 otherwise. An empty slot is not an item, so `.nil` is found only where
it was stored. The method takes exactly one argument: none, or an omitted one,
raises `SYNTAX 93.903` and a second raises `SYNTAX 93.902`.

```rexx
a = .array~of(1, 'x', , .nil)
say a~hasItem(1) a~hasItem('1') a~hasItem(1.0) a~hasItem('X') a~hasItem(.nil)
say .array~of(1, , 3)~hasItem(.nil)
call try 'a~hasItem'
call try 'a~hasItem(, 1)'
call try 'a~hasItem(1, 2)'
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
a~hasItem => 93.903
a~hasItem(, 1) => 93.902
a~hasItem(1, 2) => 93.902
```

r[class.array.index]
`index(item)` answers the index of the first item that is equal to the
argument, searching in index order as `hasItem` does, and `.nil` when there is
none. In a multi-dimensional array the answer is an array that holds one index
for each dimension. The method takes exactly one argument: none, or an omitted
one, raises `SYNTAX 93.903` and a second raises `SYNTAX 93.902`.

```rexx
a = .array~of('p', 'q', 'q', .nil)
say a~index('q') a~index(.nil) (a~index('z') == .nil)
b = .array~new(2, 3)
b[2, 3] = 'x'
say b~index('x')~class~id b~index('x')~makeString('L', ',')
call try 'a~index'
call try 'a~index(1, 2)'
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
2 4 1
Array 2,3
a~index => 93.903
a~index(1, 2) => 93.902
```

## Walking the array

r[class.array.order]
The entries of an array of more than one dimension have one order: the order
of their slots with the first dimension changing fastest, so that in a 2 by 2
array (1,1) comes first, then (2,1), (1,2) and (2,2). Every method that walks
the entries follows it, for the indexes as for the items: `allItems`,
`makeArray` and `allIndexes`, `supplier`, `first`, `last`, `firstItem` and
`lastItem`, `next` and `previous`, the search of `hasItem`, `index` and
`removeItem`, `makeString` and the sorts.

```rexx
a = .array~new(2, 2)
a[1, 2] = 'r'; a[2, 1] = 'q'
say a~allItems~makeString('L', ',')
do i over a~allIndexes
  say i~makeString('L', ',') a[i]
end
s = a~supplier
say s~index~makeString('L', ',') s~item
say a~first~makeString('L', ',') a~last~makeString('L', ',')
say a~next(2, 1)~makeString('L', ',') a~previous(1, 2)~makeString('L', ',')
b = .array~new(2, 2)
b[2, 1] = 7; b[1, 2] = 7
say b~index(7)~makeString('L', ',')
b~removeItem(7)
say b~first~makeString('L', ',')
```

```output defect=#159
q,r
2,1 q
1,2 r
2,1 q
2,1 1,2
1,2 2,1
2,1
1,2
```

r[class.array.first]
`first` answers the lowest index that holds an item and `last` the highest,
and `firstItem` and `lastItem` answer the items there; all four answer `.nil`
for an array without an item. For a multi-dimensional array the indexes are
arrays that hold one index for each dimension. The four methods take no
argument: any argument raises `SYNTAX 93.902`.

```rexx
a = .array~of(, 'b', , 'd', )
say a~first a~last a~firstItem a~lastItem a~size
say (.array~new(3)~first == .nil) (.array~new~lastItem == .nil)
b = .array~new(2, 3)
b[2, 3] = 'x'
say b~first~makeString('L', ',') b~last~makeString('L', ',') b~firstItem
call try 'a~first(1)'
call try 'a~lastItem(1)'
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
2 4 b d 4
1 1
2,3 2,3 x
a~first(1) => 93.902
a~lastItem(1) => 93.902
```

r[class.array.next]
`next(index)` answers the index of the first item after the index, passing
over empty slots, and `.nil` when there is none; `previous(index)` answers the
index of the last item before it. The index need not hold an item, or exist:
`next` of an index beyond the array answers `.nil`, and `previous` of one
beyond the array answers as [the rule for an index beyond
the array](r:class.array.next.beyond) says. The index is
read as [the index rule](r:class.array.indexes) says, with `SYNTAX 93.901` for
no index and `SYNTAX 93.907` for an index that is not a whole number of at
least 1; so index 0 is an error, and `previous(1)` is `.nil`.

```rexx
a = .array~of('a', , , 'd', 'e')
say a~next(1) a~next(2) a~next(4) (a~next(5) == .nil) (a~next(9) == .nil)
say a~previous(5) a~previous(4) (a~previous(1) == .nil) a~previous(3)
say a~previous(9)
call try 'a~next'
call try 'a~next(0)'
call try 'a~previous(.nil)'
call try 'a~next(1, 2)'
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
4 4 5 1 1
4 1 1 1
5
a~next => 93.901
a~next(0) => 93.907
a~previous(.nil) => 93.907
a~next(1, 2) => 93.926
```

r[class.array.next.beyond]
`previous` of an index that has a component beyond its dimension answers the
index of the last item that lies before a position, or `.nil` when there is
none. In an array of one dimension the position is the index, so an index
beyond the array answers the last item. In an array of more than one
dimension the slots are numbered with the first dimension running fastest: the
slot at `i, j` is number `i + (j - 1) * n`, where `n` is the size of the first
dimension, and likewise for more dimensions. The position is the value of the
first component, in the order of the dimensions, that is beyond its dimension;
the other components do not matter.

```rexx
a = .array~new(3, 3)
do i = 1 to 3; do j = 1 to 3; a[i, j] = i||j; end; end
say a~previous(4, 1)~makeString('L', ',') a~previous(4, 7)~makeString('L', ',')
say a~previous(1, 5)~makeString('L', ',') a~previous(10, 1)~makeString('L', ',')
b = .array~new(2, 2)
b[2, 2] = 'x'
say (b~previous(3, 1) == .nil) b~previous(5, 1)~makeString('L', ',')
say .array~of('x', 'y')~previous(9)
```

```output defect=#160
3,1 3,1
1,2 3,3
1 2,2
2
```

## Size, dimensions and filling

r[class.array.items]
`items` answers the number of items the array holds, `size` the number of
places it has, and `isEmpty` answers 1 when it holds no item and 0 when it
does. The size of an array only grows when an entry is put beyond it, or an
item inserted or appended past it, and falls only by one for each `delete`;
`remove` and `empty` leave it as it is. The size of a multi-dimensional array
is the product of its dimensions. The three methods take no argument: any
argument raises `SYNTAX 93.902`.

```rexx
a = .array~of('a', , 'c')
say a~items a~size a~isEmpty
a~remove(3)
say a~items a~size a~isEmpty
b = .array~new(4)
say b~items b~size b~isEmpty
b[6] = 'x'
say b~items b~size b~isEmpty
say .array~new(2, 3)~size .array~new~size
call try 'a~items(1)'
call try 'a~size(1)'
call try 'a~isEmpty(1)'
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
2 3 0
1 3 0
0 4 1
1 6 0
6 0
a~items(1) => 93.902
a~size(1) => 93.902
a~isEmpty(1) => 93.902
```

r[class.array.empty]
`empty` removes every item and answers the receiver. The size and the
dimensions stay as they were, so an emptied array of size 3 still has size 3.
The method takes no argument: one raises `SYNTAX 93.902`.

```rexx
a = .array~of(1, 2, 3)
say (a~empty == a) a~items a~size a~dimension
b = .array~new(2, 2)
b[1, 1] = 'x'
b~empty
say b~items b~dimensions~makeString('L', 'x')
call try 'a~empty(1)'
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
1 0 3 1
0 2x2
a~empty(1) => 93.902
```

r[class.array.fill]
`fill(item)` puts the item at every index of the array, from 1 to its size in
each dimension, replacing the items that are there, and answers the receiver.
An array whose size is 0 stays empty. The method takes exactly one argument:
none, or an omitted one, raises `SYNTAX 93.903` and a second raises
`SYNTAX 93.902`.

```rexx
a = .array~of('x', , 'z')
say (a~fill(0) == a) a~items a~allItems~makeString('L', ',')
b = .array~new(2, 2)~fill('k')
say b~items b[2, 2]
say .array~new~fill('q')~items
call try 'a~fill'
call try 'a~fill(, 1)'
call try 'a~fill(1, 2)'
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
1 3 0,0,0
4 k
0
a~fill => 93.903
a~fill(, 1) => 93.902
a~fill(1, 2) => 93.902
```

r[class.array.dimension]
`dimension` with no argument answers the number of dimensions: 0 for an array
that has none yet, 1 for a one-dimensional array. `dimension(n)` answers the
size of dimension `n`, and 0 when the array has fewer dimensions than `n`; `n`
is read as [the index rule](r:class.array.indexes) reads an index and must be
a whole number of at least 1, or `SYNTAX 93.907` is raised. `dimensions` answers
a new `Array` that holds the
size of each dimension, in order, and for an array that has no dimension yet
an array that holds 0. A second argument to `dimension`, or any argument to
`dimensions`, raises `SYNTAX 93.902`. The size of a dimension is the highest
index in use or the size it was made with, as for `size`.

```rexx
a = .array~new(2, 3)
say a~dimension a~dimension(1) a~dimension(2) a~dimension(3) a~dimension('2')
say a~dimensions~class~id a~dimensions~makeString('L', ',')
b = .array~new
say b~dimension b~dimension(1) b~dimensions~makeString('L', ',')
c = .array~of(1, 2, 3)
c~delete(1)
say c~dimension c~dimension(1) c~dimensions[1]
call try 'a~dimension(0)'
call try 'a~dimension(.nil)'
call try 'a~dimension(1, 2)'
call try 'a~dimensions(1)'
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
2 2 3 0 3
Array 2,3
0 0 0
1 2 2
a~dimension(0) => 93.907
a~dimension(.nil) => 93.907
a~dimension(1, 2) => 93.902
a~dimensions(1) => 93.902
```
## Taking part of an array

r[class.array.section]
`section(start, count)` answers a new array that holds the entries from the
index `start` on: `count` of them, or all of them up to the end of the array
when `count` is omitted or runs past the end. The first entry of the section
is at index 1, and an empty slot of the receiver stays an empty slot. The
items are the receiver's own objects, and the receiver is not changed. A
`start` beyond the size of the receiver, or a `count` of 0, gives an array
with no entries and no dimension. The index is read as [the index
rule](r:class.array.indexes) says, so an array of one index is accepted, and
`SYNTAX 93.907` is raised for a `start` that is not a whole number of at least
1. The `count` is read as [the index rule](r:class.array.indexes) reads an
index and must be a whole number of 0 or more, or `SYNTAX 93.923` is raised. No
`start`, or an omitted one, raises `SYNTAX 93.903`, and a third
argument `SYNTAX 93.902`; for an array of more than one dimension
`SYNTAX 93.954` is raised after the count of arguments has been checked,
whatever they are. The answer is an instance of the class of the receiver.

```rexx
a = .array~of('a', 'b', , 'd', 'e')
s = a~section(2)
say s~size s~items s~allItems~makeString('L', ',')
s = a~section(2, 3)
say s~size s~items (s[2] == .nil)
say a~section(4, 99)~allItems~makeString('L', ',')
say a~section(.array~of(5))~allItems~makeString('L', ',')
say a~section(6)~size a~section(6)~dimension a~section(2, 0)~dimension
say a~items a~size
call try 'a~section'
call try 'a~section(0)'
call try 'a~section(1, -1)'
call try 'a~section(1, "x")'
call try 'a~section(1, 2, 3)'
call try '.array~new(2, 2)~section(1)'
call try '.array~new(2, 2)~section'
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
4 3 b,d,e
3 2 1
d,e
e
0 0 0
4 5
a~section => 93.903
a~section(0) => 93.907
a~section(1, -1) => 93.923
a~section(1, "x") => 93.923
a~section(1, 2, 3) => 93.902
.array~new(2, 2)~section(1) => 93.954
.array~new(2, 2)~section => 93.954
```

r[class.array.section.sends]
`section` makes its answer by sending `new` to the class of the receiver with
one argument, the number of places the section covers, which is 0 for an empty
section, and then the new object is sent `init` with no argument, as [the new
rule](r:class.array.new.sends) says. A subclass that overrides `new` or `init`
therefore sees both, and the answer is an instance of the subclass.

```rexx
a = .many~of('a', 'b', 'c', 'd')
s = a~section(2, 2)
say s~class~id s~items s~allItems~makeString('L', ',')
s = a~section(9)
say s~items
::class many subclass array
::method new class
  say 'new' arg(1)
  forward class (super)
::method init
  say 'init' arg()
  forward class (super)
```

```output defect=#128
init 0
new 2
init 0
MANY 2 b,c
new 0
init 0
0
```

## The items, the indexes and a supplier

r[class.array.makearray]
`makeArray` and `allItems` answer a new `Array` that holds the items of the
receiver in the order of its indexes, without the empty slots; the first is at
index 1. In an array of more than one dimension that order is the one [the order
rule](r:class.array.order) gives. `allIndexes` answers a new
`Array` of the indexes that hold an item, in the same order: whole numbers, or
arrays of indexes in a multi-dimensional array. The three answer an `Array`
whatever the class of the receiver is, never the receiver, and do not follow
later changes to it. They take no argument: any argument raises
`SYNTAX 93.902`.

```rexx
a = .array~of('a', , 'c')
say a~makeArray~makeString('L', ',') a~makeArray~size
say a~allItems~makeString('L', ',') a~allIndexes~makeString('L', ',')
b = .array~new(2, 2)
b[1, 1] = 'p'; b[2, 1] = 'q'; b[1, 2] = 'r'; b[2, 2] = 's'
say b~allItems~makeString('L', ',') b~makeArray~dimension
c = .array~new(2, 3)
c[2, 3] = 'x'
say c~allIndexes~items c~allIndexes[1]~makeString('L', ',')
items = a~allItems
a~append('d')
say items~items a~items (items == a)
say .twin~of(1, 2)~makeArray~class~id .twin~of(1, 2)~allIndexes~class~id
call try 'a~makeArray(1)'
call try 'a~allItems(1)'
call try 'a~allIndexes(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class twin subclass array
```

```output
a,c 2
a,c 1,3
p,q,r,s 1
1 2,3
2 3 0
Array Array
a~makeArray(1) => 93.902
a~allItems(1) => 93.902
a~allIndexes(1) => 93.902
```

r[class.array.supplier]
`supplier` answers a `Supplier` whose indexes and items are those of the
entries that hold an item, in index order. It is a snapshot: entries put,
removed or deleted afterwards do not change it. An array without an item gives
a supplier that is not available. The method takes no argument: any argument
raises `SYNTAX 93.902`. The `Supplier` class has its own page; the supplier of
a multi-dimensional array answers an array of indexes as the `allIndexes` of
the array does.

```rexx
a = .array~of('a', , 'c')
s = a~supplier
a~put('z', 1)
a~append('d')
do while s~available
  say s~index s~item
  s~next
end
say .array~new(3)~supplier~available
call try 'a~supplier(1)'
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
3 c
0
a~supplier(1) => 93.902
```
## The items as text

r[class.array.makestring]
`makeString(mode, separator)` answers a string made from the items of the
array in the order of the slots, with the first dimension changing fastest,
and `toString` is the same method under another name. The first letter of
`mode`, in either case, chooses the form and nothing after it counts: `L` puts
the separator between one item and the next, and is the form when `mode` is
omitted, and `C` joins the items directly. The separator is read from its
string value and is the line end `'0A'x` when it is omitted; it may be the
null string. A `mode` that does not begin with `L` or `C` raises
`SYNTAX 93.915`; a separator with `C`, even a null one, raises
`SYNTAX 93.902`, as a third argument does; a mode or a separator that has no
string value raises `SYNTAX 88.909`. The mode is checked first, its string
value and then its letter, and the separator after it. An item that is a string
is used as it is, and
any other object, `.nil` included, gives the answer of its `objectName`
message (a whole-number answer is written as its digits, [Deviation
107](r:class.array.deviation.107)). An empty slot gives nothing, and no
separator is added for it, but an
item that is the null string still takes its separator. An array with no item
answers the null string.

```rexx
a = .array~of(1, 2, 3)
say a~makeString('L', '-') a~makeString('Char') a~toString(, '+')
say a~makeString~c2x
say .array~of('x', , 'y')~makeString('l', '--')
say '['.array~of('x', '')~makeString('l', '|')']'
say '['.array~of('x', '', 'y')~makeString('L', '/')']'
say '['.array~new~makeString']' '['.array~new(3)~makeString('c')']'
say .array~of(.nil, 'n')~makeString('L', ' ')
b = .array~new(2, 2)
b[1, 1] = 'p'; b[2, 1] = 'q'; b[1, 2] = 'r'; b[2, 2] = 's'
say b~makeString('L', '')
call try 'a~makeString(.nil)'
call try 'a~makeString("x")'
call try 'a~makeString(" l")'
call try 'a~makeString("c", "-")'
call try 'a~makeString("l", "-", 1)'
call try 'a~makeString("l", .object~new)'
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
1-2-3 123 1+2+3
310A320A33
x--y
[x|]
[x//y]
[] []
The NIL object n
pqrs
a~makeString(.nil) => 88.909
a~makeString("x") => 93.915
a~makeString(" l") => 93.915
a~makeString("c", "-") => 93.902
a~makeString("l", "-", 1) => 93.902
a~makeString("l", .object~new) => 88.909
```

r[class.array.makestring.sends]
`makeString` sends `objectName`, with no argument, to each item that is not a
string, once, in the order of the slots, and nothing else: an item's own
`makeString` and `string` are not used. A string item is sent nothing.

```rexx
a = .array~of(.item~new('p'), 'plain', .item~new('q'))
say a~makeString('L', '-')
::class item
::method init
  expose name
  use arg name
::method objectName
  expose name
  say 'objectName' name
  return name
::method makeString
  say 'makeString'
  return 'MADE'
::method string
  say 'string'
  return 'STRING'
```

```output
objectName p
objectName q
p-plain-q
```

r[class.array.string]
An array has a `makeString` method, so its string value, the one that `say`
and the joining operators use, is what `makeString` answers with no argument:
the items one to a line. The `string` method is the one of `Object` and
answers the name of the array, `an Array`, as [the string
rule](r:class.index.string) says; `request('STRING')` answers the string
value. An array with no item has the null string for a string value.

```rexx
a = .array~of('one', 'two')
say a
say a~string
say 'x' || a~request('STRING')~length
say '['.array~new']'
```

```output
one
two
an Array
x7
[]
```

## Sorting

r[class.array.sort]
`sort` and `stableSort` put the items of the array in ascending order, in
place, and answer the receiver; `sortWith(comparator)` and
`stableSortWith(comparator)` do the same by the order the comparator gives.
Items are compared by `compareTo`, which for strings is [the string
rule](r:class.string.compareto): `10` sorts before `9`, and upper case before
lower case. The comparator is any object that understands `compare`, and when
it answers a negative number for two items the first sorts before the second,
when it answers 0 they are equal, and when it answers a positive number the
first sorts after. `stableSort` and `stableSortWith` keep items that are equal
in their order; whether `sort` and `sortWith` do is not specified. The sorted
items fill the slots from index 1 on in the order of the slots (in a
multi-dimensional array the first dimension changing fastest), so an array
whose items are not in the first slots, with an empty slot before an item,
raises `SYNTAX 98.975` before any comparison and stays as it was; empty slots
after the items stay empty, and the size does not change. An array of fewer
than two items is not compared; `sortWith` and `stableSortWith` then answer at
once, before the slot check and without looking at the comparator, while `sort`
and `stableSort` still make the slot check, so `.array~of(, 5)~sort` raises
`SYNTAX 98.975` and `.array~of(, 5)~sortWith(.nil)` does not. `sort` and
`stableSort` take no argument: any raises `SYNTAX 93.902`. `sortWith` and
`stableSortWith` take exactly one: none, or an omitted one, raises
`SYNTAX 93.903` and a second `SYNTAX 93.902`. An item or comparator that does
not understand its message raises `SYNTAX 97.1`. The comparator's answer must
be a whole number: another answer raises `SYNTAX 26.903`; the answer of an
item's `compareTo` raises `SYNTAX 26.902` in the same case.

```rexx
a = .array~of('b', 'A', 'a', '10', '9')
say (a~sort == a) a~allItems~makeString('L', ',')
a = .array~of(3, 1, 2)
a~sortWith(.cmp~new)
say a~allItems~makeString('L', ',')
a = .array~of('x', 'y', 'z')~~stableSortWith(.firstletter~new)
say a~allItems~makeString('L', ',')
b = .array~new(5)
b[1] = 'q'; b[2] = 'p'
b~stableSort
say b~size b~items b[1] b[2]
c = .array~new(2, 2)
c[1, 1] = 'd'; c[2, 1] = 'c'; c[1, 2] = 'b'; c[2, 2] = 'a'
c~sort
say c[1, 1] c[2, 1] c[1, 2] c[2, 2]
call try '.array~of(2, , 1)~sort'
call try '.array~of(, 5)~sort'
call try '.array~of(2, 1)~sort(1)'
call try '.array~of(2, 1)~sortWith'
call try '.array~of(2, 1)~sortWith(.cmp~new, 1)'
call try '.array~of(2, 1)~sortWith(.nil)'
call try '.array~of(2, 1)~sortWith(.badcmp~new)'
call try '.array~of(.badlike~new, .badlike~new)~sort'
say .array~of(5)~sortWith(.nil)~items
say .array~of(, 5)~sortWith(.nil)~items
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class cmp
::method compare
  use arg x, y
  return y - x
::class firstletter
::method compare
  return 0
::class badcmp
::method compare
  return 'x'
::class badlike
::method compareTo
  return 'x'
```

```output
1 10,9,A,a,b
3,2,1
x,y,z
5 2 p q
a b c d
.array~of(2, , 1)~sort => 98.975
.array~of(, 5)~sort => 98.975
.array~of(2, 1)~sort(1) => 93.902
.array~of(2, 1)~sortWith => 93.903
.array~of(2, 1)~sortWith(.cmp~new, 1) => 93.902
.array~of(2, 1)~sortWith(.nil) => 97.1
.array~of(2, 1)~sortWith(.badcmp~new) => 26.903
.array~of(.badlike~new, .badlike~new)~sort => 26.902
1
1
```
r[class.array.sort.sends]
`sort` and `stableSort` send `compareTo` to items, with another item of the
array as the argument, and `sortWith` and `stableSortWith` send `compare` to
the comparator, with two items as the arguments. Which items are compared,
which of two is the receiver or the first argument, how many comparisons are
made and in what order is not specified, except that an array of fewer than
two items is not compared at all, and that nothing but `compareTo` and
`compare` is sent to items or comparator; no argument is asked `isA`. The
example checks what is specified, that every argument is an item of the array.

```rexx
items = .array~of(.item~new(3), .item~new(1), .item~new(2), .item~new(5))
.local~calls = 0
.local~bad = 0
items~sort
say 'compareTo' (.local~calls > 0) .local~bad
.local~calls = 0
items~stableSortWith(.cmp~new)
say 'compare' (.local~calls > 0) .local~bad
.local~calls = 0
.array~of(.item~new(1))~sort
.array~of(.item~new(1))~sortWith(.cmp~new)
say 'single' .local~calls
out = ''
do it over items
  out = out it~n
end
say out~strip
::class item
::method init
  expose n
  use arg n
::method n
  expose n
  return n
::method compareTo
  expose n
  use arg other
  .local~calls = .local~calls + 1
  if \other~isA(.item) then .local~bad = .local~bad + 1
  return n - other~n
::class cmp
::method compare
  use arg x, y
  .local~calls = .local~calls + 1
  if \x~isA(.item) | \y~isA(.item) then .local~bad = .local~bad + 1
  return x~n - y~n
```

```output
compareTo 1 0
compare 1 0
single 0
1 2 3 5
```

## Copies and the methods of the mixins

r[class.array.copy]
`copy` answers a new array of the same class as the receiver, with the same
dimensions and size, the same items at the same indexes and the same empty
slots, so that adding to or removing from one does not change the other. The
items are the same objects: the copy is shallow. As [the copy
rule](r:class.object.copy) says of every object, the variables of a subclass
instance are copied as they are and the name stays; `init` is not sent. The
method takes no argument: one raises `SYNTAX 93.902`.

```rexx
a = .many~of('x', , 'z')
a~tag = 'T'
b = a~copy
say b~class~id b~size b~items b~tag (b == a) (b[1] == a[1])
b~append('w')
b[1] = 'new'
say a~items b~items a[1] b[1]
c = .array~new(2, 2)
c[2, 2] = 'q'
d = c~copy
say d~dimensions~makeString('L', 'x') d[2, 2] d~items
::class many subclass array
::attribute tag
::method init
  say 'init'
  forward class (super)
```

```output
init
MANY 3 2 T 0 1
2 3 x new
2x2 q 1
```

r[class.array.inherited]
The methods `union`, `difference`, `intersection`, `xor`, `subset`, `disjoint`
and `equivalent` and `appendAll` are not declared by `Array`. The methods
`union`, `difference`, `intersection`, `xor`, `subset` and `appendAll` are
those of [`OrderedCollection`](orderedcollection.md), and `disjoint` and
`equivalent` those of [`Collection`](collection.md), as the pages of the
mixins say, and an
array is one of the collections they take as an argument, so that
`.array~of(1)~union(.list~of(2))` is an array of 1 and 2. `union`,
`difference` and `xor` send the copy they work on `isA` with the class `Array`
and answer its `makeArray` when that answers 1, as their rules say, so for an
array, an instance of a subclass included, the answer is a new `Array`;
`intersection` answers the collection that `new` of the receiver's class made,
an instance of that class, and sends it neither message. `appendAll` answers
the receiver.

```rexx
a = .many~of(1, 2, 2)
b = .array~of(2, 3)
say a~union(b)~class~id a~union(b)~makeString('L', ',')
say a~difference(b)~class~id a~difference(b)~makeString('L', ',')
say a~intersection(b)~class~id a~intersection(b)~makeString('L', ',')
say a~xor(b)~class~id a~xor(b)~makeString('L', ',')
say a~subset(b) a~disjoint(b) a~equivalent(b) a~equivalent(.list~of(2, 1, 2))
say (a~appendAll(b) == a) a~makeString('L', ',')
say .array~of(1)~union(.list~of(2))~makeString('L', ',')
::class many subclass array
```

```output
Array 1,2,2,2,3
Array 1,2
MANY 2
Array 1,2,3
0 0 0 1
1 1,2,2,2,3
1,2
```
## Messages the methods send

r[class.array.sends]
No method that `Array` declares sends a message to its receiver: an override
of `at`, `put`, `append` or any other of the methods is never asked by another
method, and `[]` and `[]=` do not use `at` and `put`. The messages the methods
do send are to other objects, and each has a rule of its own: `new` and `init`
to the class and the new array by `new`, `of` and `section` ([the new
rule](r:class.array.new.sends), [the section
rule](r:class.array.section.sends)); `==` to the argument by `removeItem`,
`hasItem` and `index` ([the search rule](r:class.array.removeitem.sends));
`objectName` to the items by `makeString` and `toString` ([the text
rule](r:class.array.makestring.sends)); and `compareTo` to the items and
`compare` to the comparator by the sorts ([the sort
rule](r:class.array.sort)). No argument is asked `isA`; a string value is read
as [the conversion rule](r:conv.string.object) says. The methods `string` and
`copy` of an instance and the set operations send what the pages that own them
say. In the example a subclass has every method of `Array` overridden to
record its name, and each call records only its own.

```rexx
own = '[] []= allIndexes allItems append at delete dimension dimensions',
  'empty fill first firstItem hasIndex hasItem index insert isEmpty items',
  'last lastItem makeArray makeString next previous put remove removeItem',
  'section',
  'size sort sortWith stableSort stableSortWith supplier toString copy'
.local~log = .array~new
do m over own~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"'); forward class (super)"
  .spy~define(m, src)
end
a = .spy~of('c', 'b', 'a')
calls = 'x = a[1]|a[1] = "x"|a~allIndexes|a~allItems|a~append("d")|a~at(1)',
  '|a~sort|a~sortWith(.cmp~new)|a~stableSort|a~stableSortWith(.cmp~new)',
  '|a~delete(1)|a~dimension|a~dimensions|a~fill("q")|a~first|a~firstItem',
  '|a~hasIndex(1)|a~hasItem("q")|a~index("q")|a~insert("e", 1)|a~isEmpty',
  '|a~items|a~last|a~lastItem|a~makeArray|a~makeString|a~next(1)',
  '|a~previous(2)|a~put("y", 1)|a~remove(1)|a~removeItem("q")|a~section(1)',
  '|a~size|a~supplier|a~toString|a~copy|a~empty'
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
::class spy subclass array
::class cmp
::method compare
  return 0
```

```output
37 calls, 0 sent more than one message to the receiver
```
## Deviations

r[class.array.deviation.107]
An item whose `objectName` answers a string that names a number is written
by `makeString` and `toString` as the characters of that string, however the
number was written or computed, as [the Object
rule](r:class.object.deviation.107) says of every string. ooRexx writes bytes
of its stored form instead when the answer is a whole-number literal or the
result of arithmetic ([Deviation 107](../deviations.md)).

```rexx
say .array~of(.n3~new, 'x')~makeString('L', '-')
say .array~of(.n3~new)~toString~c2x
::class n3
::method objectName
  return 3
```

```output
3-x
33
```

r[class.array.deviation.109]
An index, a size, a count, the `n` of `dimension`, a mode or a separator that
is an instance of a subclass of `String` is read by its own characters, and
`REQUEST` and `MAKESTRING` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an instance
`request` with `STRING` or `MAKESTRING`, so an override of either can change
what is read. In the example `MAKESTRING` answers another number,
separator and mode.

```rexx
a = .array~of('first', 'second')
say a~at(.num~new('2'))
say a~makeString('L', .sep~new('-'))
say a~makeString(.mode~new('C'))
::class num subclass string
::method makestring
  return '1'
::class sep subclass string
::method makestring
  return '+'
::class mode subclass string
::method makestring
  return 'L'
```

```output defect=#140
second
first-second
firstsecond
```
