---
element: "Stem"
kind: class
rexxref: "Builtin Classes > Collection Classes > Stem Class; Builtin Classes > Collection Classes > Stem Class > new (Class Method); Builtin Classes > Collection Classes > Stem Class > []; Builtin Classes > Collection Classes > Stem Class > []=; Builtin Classes > Collection Classes > Stem Class > allIndexes; Builtin Classes > Collection Classes > Stem Class > allItems; Builtin Classes > Collection Classes > Stem Class > at; Builtin Classes > Collection Classes > Stem Class > empty; Builtin Classes > Collection Classes > Stem Class > hasIndex; Builtin Classes > Collection Classes > Stem Class > hasItem; Builtin Classes > Collection Classes > Stem Class > index; Builtin Classes > Collection Classes > Stem Class > isEmpty; Builtin Classes > Collection Classes > Stem Class > items; Builtin Classes > Collection Classes > Stem Class > makeArray; Builtin Classes > Collection Classes > Stem Class > put; Builtin Classes > Collection Classes > Stem Class > remove; Builtin Classes > Collection Classes > Stem Class > removeItem; Builtin Classes > Collection Classes > Stem Class > request; Builtin Classes > Collection Classes > Stem Class > supplier; Builtin Classes > Collection Classes > Stem Class > toDirectory; Builtin Classes > Collection Classes > Stem Class > unknown"
deviations: [1, 68, 78, 109, 118, 124]
---

# Stem

A `Stem` is the collection that a stem symbol names: [the value of a stem
symbol](r:expr.variables.stem) is a `Stem` object, and the compound variables
of the stem are its entries. An entry is a tail, a string, and the item stored
at it; the stem also has a default, the value that every tail without an item
has, and a name, which is the text that stands for the stem when it has no
default. A `Stem` can be made without a variable, by `new`, and then behaves
alike: the methods below read and write the same entries that `s.tail` reads
and writes ([the compound rule](r:expr.variables.compound), [the default
rule](r:expr.variables.default)). The class inherits
[`MapCollection`](mapcollection.md), and through it
[`Collection`](collection.md); [the hierarchy rule](r:class.index.hierarchy)
lists what it inherits.

A stem has an `unknown` method, and the page begins with what the other methods
do; [the unknown rule](r:class.stem.unknown) says what `unknown` does with the
messages that no method answers. Examples that show an error use a small routine
`try`, as the builtin pages do. The order in which the tails are listed is not
specified, as [the DO OVER rule](r:instr.do.deviation.1) says of a loop, and the
examples then hold one tail or sort the answers ([the order
rule](r:class.stem.deviation.1)).

## The class

r[class.stem.creation]
`Stem` declares the instance methods `[]`, `[]=`, `allIndexes`, `allItems`,
`at`, `empty`, `hasIndex`, `hasItem`, `index`, `isEmpty`, `items`,
`makeArray`, `put`, `remove`, `removeItem`, `request`, `supplier`,
`toDirectory` and `unknown`, and the class method `new`; the `copy` of
`Object` copies a stem as [the copy rule](r:class.stem.copy) says. It has no
`init` of its own: `new` makes the stem as [the new rule](r:class.stem.new)
says. The class method `of` and the instance methods `putAll`, `union`,
`difference`, `intersection`, `xor`, `subset`, `disjoint` and `equivalent` are
those of the mixins ([one rule](r:class.stem.inherited) lists them). A stem
has no `makeString`, and none of the comparison methods `=`, `==`, `\=`,
`\==`, `<>` and `><`: a message with one of those names is passed on by [the
unknown rule](r:class.stem.unknown). Its superclasses are `Object` and
`MapCollection`, in that order.

```rexx
s = .stem~new
n = 0
m = .stem~methods(.nil)
do while m~available
  n += 1
  m~next
end
say s~class~id n 'own methods' .stem~hasMethod('OF') s~hasMethod('OF')
do c over s~class~superclasses
  say c~id
end
say s~hasMethod('UNION') s~hasMethod('PUTALL') s~hasMethod('DISJOINT')
say s~hasMethod('MAKESTRING') s~hasMethod('UNKNOWN') s~hasMethod('INIT')
say s~hasMethod('=') s~hasMethod('==') s~hasMethod('\=') s~hasMethod('\==')
say s~hasMethod('<>') s~hasMethod('><')
```

```output
Stem 19 own methods 1 0
Object
MapCollection
1 1 1
0 1 1
0 0 0 0
0 0
```

## Making a stem

r[class.stem.new]
`new` makes an empty stem, with no default. Its first argument is the name:
the text that the stem has as its value while it has no default, and the
start of every tail that is read from it with no item. The name is the null
string when it is omitted, and is not changed to end with a period. It must
have a string value, found as [the conversion rule](r:conv.string.object)
says for a `request` that answers a string; an object whose `request` with
`STRING` does not answer a string, `.nil` among them, raises `SYNTAX 88.909`
before `init` is sent. `new` then sends `init` to the new stem with the
arguments that follow the name ([the new rule](r:class.index.new)). A stem
has the `init` of [`Object`](r:class.object.init), which takes no argument,
so a second argument to `new` raises `SYNTAX 93.902` when that `init` runs:
after the `init` of a subclass has run, and not at all when the `init` of a
subclass does not pass its arguments on.

```rexx
s = .stem~new('abc.')
say s~string s['x'] s~items
say '['.stem~new~string']' '['.stem~new('')~string']' .stem~new(7)~string
say .stem~new('N', )~string .stem~new(.array~new)~string'|'
say .tracked~new('T.')~string
call try '.stem~new(.nil)'
call try '.stem~new(.object~new)'
call try '.stem~new("a", "b")'
call try '.tracked~new(.nil)'
call try '.tracked~new("T.", "x")'
say .quiet~new('Q.', 'x', 'y')~string .quiet~new(, 'x')~string'|'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class tracked subclass stem
::method init
  say 'init' arg()
  forward class (super) continue
::class quiet subclass stem
::method init
  return
```

```output
abc. abc.x 0
[] [] 7
N |
init 0
T.
.stem~new(.nil) => 88.909
.stem~new(.object~new) => 88.909
.stem~new("a", "b") => 93.902
.tracked~new(.nil) => 88.909
init 1
.tracked~new("T.", "x") => 93.902
Q. |
```

## Tails

r[class.stem.tails]
The methods that take indexes take any number of them, and make one tail of
them: each index argument is replaced by its string value, an omitted argument
by the null string, and the strings are joined with a period between them,
trailing omitted arguments being dropped. So `s[1, 2]` and `s['1.2']` are the
same tail, and `s[1, , 3]` is the tail `1..3`. The tail is used as written:
case counts, `01` and `1` are two tails, and blanks stay ([the tail
rule](r:expr.variables.tail)). An index that is not a string is replaced as
[the conversion rule](r:conv.string.object) says: the answer of its `request`
with the argument `STRING`, and when that is not a string, of its `string`
method; so `.nil` is the tail `The NIL object`. With no index at all the
methods name the default of the stem and not a tail.

```rexx
s = .stem~new('N.')
s['a'] = 1
s['A'] = 2
s['01'] = 3
s[1] = 4
s[1, 2] = 5
s[1, , 3] = 6
s['x.y'] = 7
s[.mk~new] = 8
s[.nil] = 9
say s~items s['a'] s['A'] s['01'] s[1] s['1'] s['1.2'] s[1, 2],
  s['1..3'] s['x', 'y']
say s[.mk~new] s['MK'] s[.nil] s['The NIL object']
say s~allIndexes~sort~makeString('L', ',')
::class mk
::method makestring
  return 'MK'
```

```output
9 1 2 3 4 4 5 5 6 7
8 8 9 9
01,1,1..3,1.2,A,MK,The NIL object,a,x.y
```

```rexx
s = .stem~new('N.')
s[.asker~new] = 'v'
say s~allIndexes[1] s['REQ']
::class asker
::method request
  return 'REQ'
```

```output defect=#164
REQ v
```

```rexx
s = .stem~new('N.')
s[] = 'D'
s[''] = 'null'
say s[,] s~at(,)
```

```output defect=#174
D D
```

## Reading and writing items

r[class.stem.at]
`at(index, ...)` and `[index, ...]` answer the item stored at the tail that
the indexes make ([the tails rule](r:class.stem.tails)); when no item is
stored there, the default of the stem; and when the stem has no default, the
name of the stem followed by the tail, which is what the compound symbol would
have as its value. With no index they answer the default, or the name when
there is none. Reading stores nothing.

```rexx
s = .stem~new('N.')
say s~at s[] s~at(1) s[1, 2] s~items
s['k'] = 'item'
say s~at('k') s['k'] s['K']
s[] = 'dflt'
say s~at s['K'] s['k'] s[3, 4] s~items
```

```output
N. N. N.1 N.1.2 0
item item N.K
dflt dflt dflt dflt 0
```

r[class.stem.put]
`put(item, index, ...)` and `[index, ...] = item` store the item at the tail
that the indexes make ([the tails rule](r:class.stem.tails)), replacing the item
that was there, and answer nothing. The item may be any object, `.nil` included.
With no index they store the item as the new default of the stem and remove
every item from that same object, so the variable and every other reference to
the stem see no entries. The item is required: none, or an omitted one, raises
`SYNTAX 93.903` for `put`.

```rexx
s = .stem~new('N.')
s~put('one', 1)
s['two', 2] = 'x'
s[3] = .nil
say s~items s[1] s['two.2'] (s[3] == .nil) s~hasIndex(3)
s~put('d')
say s~items s[1] s~string
s[] = 'e'
say s~string s['zz']
call try 's~put'
call try 's~put(, 1)'
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
3 one x 1 1
0 d d
e e
s~put => 93.903
s~put(, 1) => 93.903
```

## Questions about the contents

r[class.stem.hasindex]
`hasIndex(index, ...)` answers 1 when an item is stored at the tail that the
indexes make ([the tails rule](r:class.stem.tails)) and 0 otherwise; a tail
that has only the default, or that was dropped, has none. With no index it
answers 1 whatever the stem holds.

```rexx
s = .stem~new('N.')
s['a', 'b'] = 'x'
say s~hasIndex('a.b') s~hasIndex('a', 'b') s~hasIndex('a'),
  s~hasIndex('A', 'b')
say s~hasIndex s~hasIndex() .stem~new~hasIndex
s[] = 'dflt'
say s~hasIndex('a', 'b') s~hasIndex(5) s~hasIndex
```

```output
1 1 0 0
1 1 1
0 0 1
```

r[class.stem.hasitem]
`hasItem(item)` answers 1 when an item stored at some tail is equal to the
argument, and `index(item)` answers the tail of such an item, as a string, and
`.nil` when there is none. The default of the stem is not an item. Both take
exactly one argument: a second raises `SYNTAX 93.902`. Which tail `index`
answers when several items qualify is not specified ([the order
rule](r:class.stem.deviation.1)).

```rexx
s = .stem~new('N.')
s['k1'] = 'x'
s['k2'] = 'y'
say s~hasItem('x') s~hasItem('X') s~hasItem('zz') s~hasItem('N.')
say s~index('x') s~index('y') (s~index('q') == .nil)
s[] = 'dflt'
say s~hasItem('dflt') s~hasItem('x')
t = .stem~new
t['p'] = 'v'
say t~hasItem('v') t~index('v')
call try 't~hasItem("a", "b")'
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
1 0 0 0
k1 k2 1
0 0
1 p
t~hasItem("a", "b") => 93.902
t~index(1, 2) => 93.902
```

r[class.stem.items]
`items` answers the number of tails that have an item, the default not
counted, and `isEmpty` answers 1 when there is none and 0 otherwise. Neither
takes an argument: one raises `SYNTAX 93.902`.

```rexx
s = .stem~new('N.')
say s~items s~isEmpty
s[1] = 'a'
s[2] = .nil
s[1] = 'b'
say s~items s~isEmpty
s[] = 'dflt'
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
0 1
s~items(1) => 93.902
s~isEmpty(1) => 93.902
```

## Removing items

r[class.stem.remove]
`remove(index, ...)` removes the item at the tail that the indexes make ([the
tails rule](r:class.stem.tails)) and answers it, and answers `.nil` when no
item is stored there; the tail then has the default again, or the derived
name. With no index it removes the default and answers it, and answers the
name of the stem when there is none, so that a second call answers the name.
The other items stay.

```rexx
s = .stem~new('N.')
s['a'] = 1
s['b', 'c'] = 2
say s~remove('a') (s~remove('a') == .nil) s~remove('b', 'c') s~items s['a']
s[] = 'd'
s['k'] = 'v'
say (s~remove('zz') == .nil) s~items
say s~remove s~remove s['k'] s~items
```

```output
1 1 2 0 N.a
1 1
d N. v 1
```

r[class.stem.removeitem]
`removeItem(item)` removes one item that is equal to the argument and answers
it, and answers `.nil` when there is none; when several tails hold an equal
item, which one is removed is not specified ([the order
rule](r:class.stem.deviation.1)). The default is not an item. It takes
exactly one argument: none raises `SYNTAX 93.903`, and a second raises
`SYNTAX 93.902`.

```rexx
s = .stem~new('N.')
s['a'] = 'x'
s['b'] = 'x'
s['c'] = 'y'
say s~removeItem('x') s~items s~removeItem('y') s~items
say (s~removeItem('zz') == .nil) s~items
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
x 2 y 1
1 1
s~removeItem => 93.903
s~removeItem(1, 2) => 93.902
```

r[class.stem.empty]
`empty` removes every item and answers the stem itself; the default and the
name stay. It takes no argument: one raises `SYNTAX 93.902`.

```rexx
s = .stem~new('N.')
s['a'] = 1
s[] = 'd'
s['a'] = 2
e = s~empty
say (e == s) s~items s~isEmpty s~string s['a']
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
1 0 1 d d
s~empty(1) => 93.902
```

## Lists of the items

r[class.stem.makearray]
`allIndexes` and `makeArray` answer a new `Array` of the tails that have an
item, as strings, and `allItems` a new `Array` of the items. The default is in
neither. They are instances of `Array` for a subclass receiver too, never the
receiver, and do not follow later changes. They take no argument: one raises
`SYNTAX 93.902`. The order is not specified ([the order
rule](r:class.stem.deviation.1)), and is the same for the three answers and
for the supplier of one unchanged stem, so the item at a position belongs to
the tail at the same position. The methods send no message to the stem,
`makeArray` excepted when it is asked through
[`request`](r:class.stem.request), or to the items.

```rexx
t = .many~new('N.')
do i = 1 to 40
  t['item'i] = .array~of(i)
end
t[] = 'dflt'
t['x', 'y'] = 'xy'
ix = t~allIndexes
it = t~allItems
bad = 0
do i = 1 to ix~items
  if t~at(ix[i]) \== it[i] then bad += 1
  if t~makeArray[i] \== ix[i] then bad += 1
end
say ix~class~id it~class~id t~makeArray~class~id ix~items it~items bad
a = t~allItems
t['extra'] = 'e'
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
::class many subclass stem
```

```output
Array Array Array 1 1 0
1 2 0
t~allIndexes(1) => 93.902
t~allItems(1) => 93.902
t~makeArray(1) => 93.902
```

r[class.stem.supplier]
`supplier` answers a `Supplier` over the tails that have an item, the tail as
the index and the item as the item. It is a snapshot: later changes do not
reach it. Its order is that of `allIndexes` ([the order
rule](r:class.stem.makearray)). A stem with no item gives a supplier that is
not available. It takes no argument: one raises `SYNTAX 93.902`.

```rexx
s = .stem~new('N.')
s['a', 'b'] = 'ab'
p = s~supplier
s['c'] = 'x'
s~remove('a', 'b')
n = 0
do while p~available
  say p~index p~item
  n += 1
  p~next
end
say n .stem~new~supplier~available
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
a.b ab
1 0
s~supplier(1) => 93.902
```

r[class.stem.todirectory]
`toDirectory` answers a new `Directory` with an entry for every tail that has
an item, the tail as the index and the item as the item; the default is not in
it. Later changes to either object do not reach the other. It takes no
argument: one raises `SYNTAX 93.902`.

```rexx
s = .stem~new('N.')
s['a'] = 1
s['b', 'c'] = 2
s[] = 'dflt'
s['k'] = 'v'
d = s~toDirectory
say d~class~id d~items d['k'] (d['zz'] == .nil)
s['m'] = 'later'
d['n'] = 'other'
say d~items s~items (d~at('m') == .nil) (s['n'])
t = .stem~new('N.')
t['a'] = 1
t['b', 'c'] = 2
say t~toDirectory~allIndexes~sort~makeString('L', ',') t~toDirectory['b.c']
call try 't~toDirectory(1)'
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
Directory 1 v 1
2 2 1 dflt
a,b.c 2
t~toDirectory(1) => 93.902
```

## Conversions and unknown messages

r[class.stem.request]
`request(class)` with the argument `ARRAY`, in any case, answers what
`makeArray` answers, after sending `makeArray` to the stem, so an override of
`makeArray` in a subclass answers. Any other class name is passed on: `request`
is sent to the default of the stem with the same argument, or to the name when
there is no default, and its answer is the answer. It takes exactly one
argument: none raises `SYNTAX 93.903` and a second raises `SYNTAX 93.902`. The
argument must have a string value: `.nil` raises `SYNTAX 88.909`.

```rexx
s = .stem~new('N.')
s['a'] = 'x'
s[] = 'dflt'
s['b'] = 'y'
say s~request('ARRAY')~items s~request('array')~class~id s~request('Array')[1]
say s~request('STRING') s~request('string')
say (s~request('INTEGER') == .nil) (s~request('STEM') == .nil)
say .stem~new('M.')~request('STRING')
t = .stem~new
t[] = .directory~new
say t~request('DIRECTORY')~class~id
call try 's~request'
call try 's~request("a", "b")'
call try 's~request(.nil)'
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
1 Array b
dflt dflt
1 1
M.
Directory
s~request => 93.903
s~request("a", "b") => 93.902
s~request(.nil) => 88.909
```

r[class.stem.unknown]
A message that the stem does not understand, `makeString` and the comparison
methods among them, is sent to the default of the stem with the same arguments,
or to the name of the stem when there is no default, and what that answers is
the answer. So a stem stands for its default in an expression: `s~length`,
`s + 1` and `s = 'abc'` are the messages `length`, `+` and `=` sent to the
default ([the extended assignment rule](r:instr.assignment.extended.stem) shows
the operators); a default that does not understand the message is sent
`unknown` by [the method search](r:class.methods.unknown), and one with no
`unknown` method raises `SYNTAX 97.1`, as does `.nil`. The message
`unknown(name, arguments)` does the same for the message `name` and the
arguments that `arguments` holds: `arguments` is sent `request` with `ARRAY`,
and an answer that is an instance of `Array`, or of a subclass of it, with one
dimension is the array of arguments, while `.nil` and an object that is no
collection raise `SYNTAX 98.913`; a position that holds nothing is an omitted
argument. `name` must have a string value (`.nil` raises `SYNTAX 88.909`), and
the message name is used as written: a name that is not in upper case names no
method of the default, which handles it as a message it does not understand.
An instance of a subclass of `List` given as `arguments` is sent `request` as
any other object is ([Deviation 118](r:class.stem.deviation.118)).
It takes exactly two arguments: fewer raises `SYNTAX 93.903` and more
`SYNTAX 93.902`.

```rexx
s = .stem~new('N.')
say s~length s~left(1) (s = 'N.') (s == 'N.') (s \= 'N.')
s[] = 'abc'
say s~length s~substr(2) s~makeString (s = 'abc') (s <> 'abc') s || 'x'
t = .stem~new('M.')
t[] = 'abc'
say (s == t) (s = t) (s == s)
n = .stem~new('N.')
n[] = 5
say n + 2 (n > 4) n~unknown('+', .array~of(1)) n~unknown('ABS', .array~new)
say s~unknown('SUBSTR', .array~of(2, 1)) s~unknown('LENGTH', .list~new)
say s~unknown('SUBSTR', .ask~new) s~unknown('LENGTH', .array~new(3))
call try 's~foo'
call try 's~unknown'
call try 's~unknown("LENGTH")'
call try 's~unknown("LENGTH", .array~new, 1)'
call try 's~unknown(.nil, .array~new)'
call try 's~unknown("LENGTH", .nil)'
call try 's~unknown("LENGTH", .array~new(2, 2))'
u = .stem~new
u[] = .nil
call try 'u~length'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class ask
::method request
  say 'request' arg(1)
  return .array~of(2, 1)
```

```output
2 N 1 1 0
3 bc abc 1 0 abcx
1 1 1
7 1 6 5
b 3
request ARRAY
b 3
s~foo => 97.1
s~unknown => 93.903
s~unknown("LENGTH") => 93.903
s~unknown("LENGTH", .array~new, 1) => 93.902
s~unknown(.nil, .array~new) => 88.909
s~unknown("LENGTH", .nil) => 98.913
s~unknown("LENGTH", .array~new(2, 2)) => 98.913
u~length => 97.1
```

```rexx
s = .stem~new('N.')
s[] = 'abc'
signal on syntax name bad
say s~unknown('SUBSTR', .ans~new)
say s~unknown('SUBSTR', .object~new)
exit
bad: say condition('o')~code
::class mya subclass array
::class ans
::method request
  return .mya~of(2, 1)
```

```output defect=#173
b
98.913
```

```rexx
s = .stem~new('N.')
s[] = 'abc'
signal on syntax name bad
say s~unknown('LENGTH', .array~new)
say s~unknown('length', .array~new)
exit
bad: say condition('o')~code
```

```output defect=#171
3
97.1
```

## Copies and the string value

r[class.stem.copy]
`copy` answers a new stem of the class of the receiver with the same name, the
same default and the same items. The items are the same objects, and the two
stems do not see each other's changes. As [the copy rule](r:class.object.copy)
says of every object, the variables of a subclass instance are copied as they
are, and `init` is not sent. It takes no argument: one raises `SYNTAX 93.902`.

```rexx
t = .many~new('N.')
t[] = 'dflt'
t['k'] = 'v'
t~tag = 'T'
c = t~copy
say c~class~id c~string c~items c~tag c['k'],
  (c~identityHash == t~identityHash)
c['j'] = 'w'
c['k'] = 'z'
c[] = 'other'
say t~string t~items t['k'] t['j'] c~string c~items
u = .stem~new('M.')
say u~copy~string u~copy~items
call try 't~copy(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass stem
::attribute tag
```

```output
MANY dflt 1 T v 0
dflt 1 v dflt other 0
M. 0
t~copy(1) => 93.902
```

r[class.stem.string]
The string value of a stem is the string value of its default, and its name
when it has no default ([the stem rule](r:expr.variables.stem)); `string` and
`say` use it, and a stem has no `makeString` of its own, so `makeString` is
passed on by [the unknown rule](r:class.stem.unknown) with the same answer.
`objectName` is that of `Object`, `a Stem`, and is not the string value.

```rexx
s = .stem~new('N.')
say s '|' s~string '|' s~objectName
s[] = 'abc'
say s '|' s~string '|' s~makeString '|' s~objectName
s[] = 12
say s + 1 s~class~id s~hasMethod('MAKESTRING')
```

```output
N. | N. | a Stem
abc | abc | abc | a Stem
13 Stem 0
```

## The stem variable and the object

r[class.stem.variable]
The value of a stem symbol is a `Stem` object, so `a.` answers the object and
the methods of this page work on the compound variables of the stem: an item
put with `put` is the value of the compound variable, a compound variable that
is assigned is an item, and a variable that is dropped has none. The name of
such a stem is its symbol in upper case, with the period, and is the name that
an unset compound variable has as its value. Assigning the stem symbol
replaces the object ([the stem rule](r:expr.variables.stem)), so an earlier
value of the symbol is no longer the stem.

```rexx
a.1 = 'one'
a.x = 'ex'
say a.~items a.~string a.~hasIndex(1) a.~at('X') a.~class~id
a.~put('two', 2)
say a.2 a.~items
b.1 = 'kay'
b.2 = 'jay'
drop b.1
say b.~items b.~allIndexes b.1 b.~at(1)
old = b.
b. = 'new'
say old~items old[2] b.~items b.~string
a.~put('d')
say a.2 a.~items a.1
```

```output
2 A. 1 ex Stem
two 3
1 2 B.1 B.1
1 jay 0 new
d 0 d
```

## The methods of the mixins

r[class.stem.inherited]
The class method `of` and the instance methods `putAll`, `union`,
`difference`, `intersection`, `xor`, `subset`, `disjoint` and `equivalent` are
not declared by `Stem`. `of` and `putAll` are those of
[`MapCollection`](mapcollection.md), and the others those of
[`Collection`](collection.md): a stem is one of the collections they take as
an argument and as a receiver, and its tails are the indexes they work with.
`of` and `putAll` store with `put(item, index)`, and the answers of `union`,
`difference` and `xor` are the object that `copy` of the stem answers, so they
keep its name and default, and the answer of `intersection` is a new stem that
`new` of the class makes, with no name and no default. `union` adds only the
tails that the copy has no item at. In the example the arguments are stems.

```rexx
a = .stem~new('A.')
a['p'] = 'P1'
a['q'] = 'Q1'
b = .stem~new('B.')
b['q'] = 'Q2'
b['r'] = 'R2'
do op over 'union difference intersection xor'~makeArray(' ')
  r = a~send(op, b)
  say op r~class~id r~items listed(r) '['r~string']'
end
say a~subset(b) b~subset(b) a~disjoint(b) a~equivalent(b) a~equivalent(a~copy)
s = .stem~of(.array~of('k1', 'x'), .array~of('k2', 'y'))
say s~items s['k1'] s['k2']
c = .stem~new('C.')
say (c~putAll(.array~of('m', 'n')) == c) c~items c[1] c[2]
exit
listed: procedure
  use arg r
  s = ''
  do i over r~allIndexes~sort
    s = s i'='r[i]
  end
  return s~strip
```

```output
union Stem 3 p=P1 q=Q1 r=R2 [A.]
difference Stem 1 p=P1 [A.]
intersection Stem 1 q=Q2 []
xor Stem 2 p=P1 r=R2 [A.]
0 1 0 0 1
2 x y
1 2 m n
```

## Messages the methods send

r[class.stem.sends]
No method that `Stem` declares sends a message to its receiver, except
`request`, which sends `makeArray` ([the request rule](r:class.stem.request)).
The set operations and `putAll` send what the pages of the mixins say, so
`union`, `difference`, `xor` and `subset` send `copy` to the receiver,
`intersection` sends `hasIndex` to it, `disjoint` sends `intersection`,
`equivalent` sends `items` and `putAll` sends `put`. An index argument that is
not a string is sent `request` with `STRING` ([the tails
rule](r:class.stem.tails)), and `hasItem`, `index` and `removeItem` send `==`
to the argument with the stored items as the argument ([the item
rule](r:class.stem.hasitem.sends)). The other methods send nothing to indexes
or items. In the example a subclass has every method overridden to record its
name, and each call records only its own.

```rexx
own = '[] []= allIndexes allItems at empty hasIndex hasItem index isEmpty',
  'items makeArray put remove removeItem supplier toDirectory copy string'
.local~log = .array~new
.local~on = 0
do m over own~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"'); forward class (super)"
  .spy~define(m, src)
end
a = .spy~new('N.')
calls = 'x = a[1]|a["k1"] = "x"|x = a~allIndexes|x = a~allItems',
  '|x = a~at("k1")',
  '|x = a~hasIndex("k1")|x = a~hasItem("q")|x = a~index("q")|x = a~isEmpty',
  '|x = a~items|x = a~makeArray|a~put(1, "k9")|x = a~remove("k9")',
  '|x = a~removeItem("q")|x = a~supplier|x = a~toDirectory|x = a~copy',
  '|x = a~string|a~empty'
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
::class spy subclass stem
```

```output
19 calls, 0 sent more than one message to the receiver
```

r[class.stem.hasitem.sends]
`hasItem(item)`, `index(item)` and `removeItem(item)` send `==` to the argument
with a stored item as the argument, once for each item, until one answers true,
and send nothing else, `hashCode` included; when none matches, every item is
compared. The order in which the items are compared is not specified ([the
order rule](r:class.stem.deviation.1)). The search with the very object that
is stored is no exception: its `==` is sent as for any other item ([the
deviation rule](r:class.stem.deviation.78)). The messages sent to an index are
those of [the tails rule](r:class.stem.tails).

```rexx
.local~n = 0
s = .stem~new('N.')
s['a'] = 's'
s['b'] = 't'
s['c'] = 'u'
q = .thing~new
say s~hasItem(q) .local~n
.local~n = 0
say (s~index(q) == .nil) .local~n
.local~n = 0
say (s~removeItem(q) == .nil) .local~n s~items
::class thing
::method '=='
  .local~n += 1
  return 0
::method hashCode
  .local~h += 1
  return 1
```

```output
0 3
1 3
1 3 3
```

## Deviations

r[class.stem.deviation.1]
The order of the answers of `allIndexes`, `allItems`, `makeArray` and
`supplier`, the order in which `hasItem`, `index` and `removeItem` compare
the items, and so which tail `index` and `removeItem` choose among equal
items, are not specified, and may differ from ooRexx's, which follows its
storage. Which tails are produced, and how often, is specified. [The DO OVER
rule](r:instr.do.deviation.1) says the same of a loop. See [Deviation
1](../deviations.md). In the example the answers are sorted.

```rexx
s = .stem~new
s['a'] = 3
s['b'] = 1
s['c'] = 2
say s~allIndexes~sort~makeString('L', ',')
say s~allItems~sort~makeString('L', ',')
say s~items s~makeArray~items
```

```output
a,b,c
1,2,3
3 3
```

r[class.stem.deviation.68]
The `UNKNOWN` method that a subclass of `Stem` declares runs for a message
that the subclass does not understand, in place of [the unknown
rule](r:class.stem.unknown) of the class. ooRexx forwards such a message to
the default of the stem and raises `SYNTAX 97.1` when that does not understand
it. See [Deviation 68](../deviations.md).

```rexx
s = .talker~new('N.')
s[] = 'abc'
say s~length s~foo(1, 2)
::class talker subclass stem
::method unknown
  use arg name, args
  return 'asked for' name args~items
```

```output
asked for LENGTH 0 asked for FOO 2
```

r[class.stem.deviation.78]
A search for an item with the very object that is stored, by `hasItem`,
`index` or `removeItem`, sends that object's `==` as for any other item, so an
object whose `==` answers false is not found even at its own tail. ooRexx
compares identity first, finds it and sends nothing. See [Deviation
78](../deviations.md).

```rexx
.local~n = 0
s = .stem~new('N.')
q = .thing~new
s['k'] = q
say s~hasItem(q) .local~n
say (s~index(q) == .nil) .local~n
say (s~removeItem(q) == .nil) .local~n s~items
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

r[class.stem.deviation.109]
A name given to `new` and an index that are instances of a subclass of
`String` are read by their own characters, and `REQUEST` and `MAKESTRING` are
not sent to them ([Deviation 109](r:class.string.deviation.109)). ooRexx sends
the name `request` with `STRING`, so an override of `request` changes the
name; it reads an index by its characters. In the example `MAKESTRING` answers
another text.

```rexx
s = .stem~new(.sub~new('N.'))
say s~string
s[.sub~new('i')] = 5
say s~allIndexes[1] s['i'] s['other']
::class sub subclass string
::method makestring
  return 'other'
```

```output defect=#140
N.
i 5 N.other
```

r[class.stem.deviation.118]
An instance of a subclass of `List` that is given to `unknown` as the array of
arguments is sent `request` with `ARRAY`, as [the unknown
rule](r:class.stem.unknown) says of any other object, so that an override is
used. See [Deviation 118](../deviations.md). ooRexx sends the instance no
`request` and uses the items it holds, so the example answers `c` on ooRexx. The
`request` of the class `ll` answers two items.

```rexx
l = .ll~new
l~append(3)
l~append(1)
s = .stem~new
s[] = 'abc'
say s~unknown('SUBSTR', l)
::class ll subclass list
::method request
  say 'request' arg(1)
  return .array~of(2, 1)
```

```output
request ARRAY
b
```

r[class.stem.deviation.124]
An answer of `request` with `ARRAY` that is not an `Array` or an instance of
a subclass of `Array` is refused as the array of arguments of `unknown`, which
raises `SYNTAX 98.913`, whatever collection the answer is. ooRexx accepts a `Queue` or a
`CircularQueue` as such an answer and uses its items, and crashes for a `List`,
a `Bag`, a `Table` or a `Directory`, among others. In the
example the answer is a queue.

```rexx
s = .stem~new
s[] = 'abc'
call try "s~unknown('SUBSTR', .ans~new)"
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class ans
::method request
  return .queue~of('x')
```

```output
s~unknown('SUBSTR', .ans~new) => 98.913
```
