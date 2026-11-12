---
element: "CircularQueue"
kind: class
rexxref: "Builtin Classes > Collection Classes > CircularQueue Class; Builtin Classes > Collection Classes > CircularQueue Class > of (Class Method); Builtin Classes > Collection Classes > CircularQueue Class > init; Builtin Classes > Collection Classes > CircularQueue Class > append; Builtin Classes > Collection Classes > CircularQueue Class > insert; Builtin Classes > Collection Classes > CircularQueue Class > intersection; Builtin Classes > Collection Classes > CircularQueue Class > makeArray; Builtin Classes > Collection Classes > CircularQueue Class > makeString; Builtin Classes > Collection Classes > CircularQueue Class > push; Builtin Classes > Collection Classes > CircularQueue Class > queue; Builtin Classes > Collection Classes > CircularQueue Class > resize; Builtin Classes > Collection Classes > CircularQueue Class > section; Builtin Classes > Collection Classes > CircularQueue Class > size; Builtin Classes > Collection Classes > CircularQueue Class > string; Builtin Classes > Collection Classes > CircularQueue Class > supplier; Builtin Classes > Collection Classes > CircularQueue Class > union; Builtin Classes > Collection Classes > CircularQueue Class > xor"
deviations: [78, 109, 121]
---

# CircularQueue

A `CircularQueue` is a [`Queue`](queue.md) that has a size, fixed when it is
made and changed by `resize`: it never holds more items than that. An item
added to a full queue pushes another out at the opposite end, so a circular
queue keeps the latest items, as a log of the last few commands does.
Everything that this page does not specify is the behaviour of `Queue`: the
indexes are positions from 1 at the head, and the methods that read, look up and
remove
items are those of the `Queue` page. The class inherits `Queue`, and through
it [`OrderedCollection`](orderedcollection.md) and
[`Collection`](collection.md); [the hierarchy rule](r:class.index.hierarchy)
lists what it inherits.

The page follows the order of the rexxref entry. Examples that show an error
use a small routine `try`, as the builtin pages do.

## The class

r[class.circularqueue.creation]
`CircularQueue` declares the class method `of` and the instance methods
`init`, `append`, `insert`, `intersection`, `makeArray`, `makeString`, `push`,
`queue`, `resize`, `section`, `size`, `string`, `supplier`, `union` and `xor`,
and a `copy` that this page specifies ([the copy
rule](r:class.circularqueue.copy)). All other methods are those of `Queue`
([the Queue creation rule](r:class.queue.creation)) and of the mixins; the
rules below say where one of them behaves differently for a circular queue. A
circular queue is a `Queue`: it has `peek`, `pull` and `sort`, and `Queue` has
no `resize` and no `makeString`.

```rexx
q = .circularqueue~new(3)
say q~class~id q~class~superclass~id q~isA(.queue) q~isA(.orderedcollection)
say q~hasMethod('RESIZE') q~hasMethod('MAKESTRING')
say .queue~new~hasMethod('RESIZE')
say .queue~new~hasMethod('MAKESTRING') .queue~new~hasMethod('SIZE')
say q~hasMethod('PEEK') q~hasMethod('PULL') q~hasMethod('SORT')
```

```output
CircularQueue Queue 1 1
1 1
0
0 1
1 1 1
```

## Making a circular queue

r[class.circularqueue.new]
`new` needs the size of the queue: no argument raises `SYNTAX 93.901` and a
second one `SYNTAX 93.902`. The size must be a whole number from 0 to
999999999, read from its string value as a whole-number argument of a builtin
is ([the form rule](r:conv.whole.form), [the digits
rule](r:conv.whole.digits)): a negative number, a number that is not whole,
text that is not a number and a number above that bound raise `SYNTAX 93.906`.
A size of 0 makes a queue that holds nothing. A new queue is empty. `new`
sends `init` to the new object with the arguments it received, so a subclass's
`init` sees them ([the new rule](r:class.index.new)); the checks are those of
the `init` that `CircularQueue` supplies.

```rexx
q = .circularqueue~new(3)
say q~size q~items q~isEmpty
say .circularqueue~new(0)~size .circularqueue~new(999999999)~size
say .circularqueue~new(' 3 ')~size '|' .circularqueue~new(3.0)~size
call try '.circularqueue~new'
call try '.circularqueue~new(-1)'
call try '.circularqueue~new("x")'
call try '.circularqueue~new(1.5)'
call try '.circularqueue~new(1000000000)'
call try '.circularqueue~new(3, 4)'
call try '.circularqueue~new(.nil)'
call try '.circularqueue~new(.object~new)'
x = .watch~new(7)
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class watch subclass circularqueue
::method init
  say 'init' arg() '('arg(1)')'
  forward class (super)
```

```output
3 0 1
0 999999999
 3  | 3.0
.circularqueue~new => 93.901
.circularqueue~new(-1) => 93.906
.circularqueue~new("x") => 93.906
.circularqueue~new(1.5) => 93.906
.circularqueue~new(1000000000) => 93.906
.circularqueue~new(3, 4) => 93.902
.circularqueue~new(.nil) => 97.1
.circularqueue~new(.object~new) => 97.1
init 1 (7)
```

r[class.circularqueue.new.sends]
`init` first reads the string value of the size, as [the conversion
rule](r:conv.string.object) says, and then sends it `<` with the argument 0,
reading the answer as a logical value: an answer that is neither 0 nor 1
raises `SYNTAX 34.901`. `SYNTAX 93.906` is raised when `<` answered 1 or when
the string value is not a whole number or is above 999999999; only `<`
decides whether the size is below 0, so a size whose string value is `-1` and
whose `<` answers 0 is not refused. An object that does not understand `<`,
`.nil` included, raises `SYNTAX 97.1`. The queue then keeps the very object
it was given as its size, `size` answers it ([the size
rule](r:class.circularqueue.size)), and its string value is read again each
time a method compares the number of items with the size.

```rexx
q = .circularqueue~new(.pretend~new(0))
say q~size~class~id q~size
do i = 1 to 6
  q~queue(i)
end
say q~allItems~makeString('L', ',')
call try '.circularqueue~new(.pretend~new(1))'
call try '.circularqueue~new(.pretend~new(3))'
call try '.circularqueue~new(.nil)'
w = .circularqueue~new(.loud~new(0, '-1'))
say 'made' w~size~class~id
w~queue('a')
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class pretend
::method init
  expose answer
  use arg answer
::method makeString
  return 4
::method string
  return 4
::method '<'
  expose answer
  use arg limit
  say 'lt' limit
  return answer
::class loud
::method init
  expose answer text
  use arg answer, text
::method makeString
  expose text
  say 'makestring'
  return text
::method '<'
  expose answer
  use arg limit
  say 'lt' limit
  return answer
```

```output
lt 0
PRETEND 4
3,4,5,6
lt 0
.circularqueue~new(.pretend~new(1)) => 93.906
lt 0
.circularqueue~new(.pretend~new(3)) => 34.901
.circularqueue~new(.nil) => 97.1
makestring
lt 0
made LOUD
makestring
```

r[class.circularqueue.of]
`of` makes a circular queue that holds its arguments in the order given, with
the size that is the number of arguments, an argument left out before a given
one counting, one left out at the end not. An item that was left out is the
null string. The result is an instance of the class that received the message.
`of` sends `new` to that class with the size as the only argument, which sends
`init` as [the new rule](r:class.circularqueue.new) says, and then sends
`queue` once for each argument, in order, with that argument, the null string
for an omitted one, as the only argument. `of` with no argument makes a queue
of size 0.

```rexx
q = .circularqueue~of('a', 'b', 'c')
say q~class~id q~size q~items q~allItems~makeString('L', ',')
q = .circularqueue~of('a', , 'c')
say q~size q~items '['q~at(2)']' q~at(2)~class~id
say .circularqueue~of~size .circularqueue~of('x', )~size
say .circularqueue~of(, )~size .circularqueue~of(.nil, .nil)~items
x = .watch~of(1, , 3)
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class watch subclass circularqueue
::method new class
  say 'new' arg(1)
  forward class (super)
::method init
  say 'init' arg(1)
  forward class (super)
::method queue
  say 'queue' arg()
  forward class (super)
```

```output
CircularQueue 3 3 a,b,c
3 3 [] String
0 1
0 2
new 3
init 3
queue 1
queue 1
queue 1
```

r[class.circularqueue.size]
`size` answers the size of the queue: the object that `new` was given, or that
`resize` was given last, as it was given, so that the size `' 3 '` is answered
as `' 3 '` and `3.0` as `3.0`. The size is not the number of items, which
`items` answers, and it does not change when items come and go. `size` takes
no argument: one raises `SYNTAX 93.902`.

```rexx
q = .circularqueue~new(' 3 ')
say '['q~size']' q~items
q~queue(1); q~queue(2)
say '['q~size']' q~items
q~resize(4.0)
say '['q~size']' (q~size == 4.0) q~items
say q~empty~size
call try 'q~size(1)'
call try '.circularqueue~new(2)~size(1)'
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
[ 3 ] 0
[ 3 ] 2
[4.0] 1 2
4.0
q~size(1) => 93.902
.circularqueue~new(2)~size(1) => 93.902
```

## Adding items

r[class.circularqueue.queue]
`queue(item, option)` adds the item at the tail and `push(item, option)` at
the head. When the queue already holds as many items as its size, the item at
the opposite end is taken out first: `queue` takes out the item at the head
and answers it, `push` takes out the item at the tail and answers it; when
nothing was taken out, both answer `.nil`. A queue of size 0 holds nothing.
The `option` is `Normal` by default; one whose first character is `U`, in
either case, is `Unique`: before the item is added, every item that is equal
to it by the strict `==` is taken out, and when that made room nothing else
is. Any other option, a null string and one that begins with a blank included,
is `Normal`, and `option` is read as [the option
rule](r:class.circularqueue.queue.sends) says. No argument at all raises
`SYNTAX 93.901`; an omitted item before an option raises `SYNTAX 93.903`; a
third argument raises `SYNTAX 93.902`.

```rexx
q = .circularqueue~new(3)
say (q~queue('a') == .nil) (q~queue('b') == .nil) (q~queue('c') == .nil)
say q~queue('d') q~allItems~makeString('L', ',') q~items
say q~push('z') q~allItems~makeString('L', ',') q~items
say q~queue('e') q~push('f') q~allItems~makeString('L', ',')
call try 'q~queue'
call try 'q~push'
call try 'q~queue(, "U")'
call try 'q~push(, "U")'
call try 'q~queue(1, "U", 3)'
call try 'q~push(1, "U", 3)'
say q~allItems~makeString('L', ',')
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
1 1 1
a b,c,d 3
d z,b,c 3
z e f,b,c
q~queue => 93.901
q~push => 93.901
q~queue(, "U") => 93.903
q~push(, "U") => 93.903
q~queue(1, "U", 3) => 93.902
q~push(1, "U", 3) => 93.902
f,b,c
```

```rexx
r = .circularqueue~new(4)
r~queue('a'); r~queue('b'); r~queue('c')
say (r~queue('a', 'U') == .nil) r~allItems~makeString('L', ',')
say (r~push('c', 'Unique') == .nil) r~allItems~makeString('L', ',')
say (r~queue('q', 'N') == .nil) r~allItems~makeString('L', ',')
say r~queue('x', 'Normal') r~allItems~makeString('L', ',')
say (r~queue('x', 'unique') == .nil) r~allItems~makeString('L', ',')
say (r~push('x', 'Unique') == .nil) r~allItems~makeString('L', ',')
say r~queue('y', 'Z') r~queue('y', ' U') r~allItems~makeString('L', ',')
say r~queue(1.0, 'U') r~queue(1, 'U') r~allItems~makeString('L', ',')
say r~queue('Y', 'U') r~allItems~makeString('L', ',')
say r~queue('b', 'U') r~push(.nil) r~queue(.nil, 'U') r~items
call try 'r~queue("n", .nil)'
call try 'r~push("n", .nil)'
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
1 b,c,a
1 c,b,a
1 c,b,a,q
c b,a,q,x
1 b,a,q,x
1 x,b,a,q
x b a,q,y,y
a q y,y,1.0,1
y y,1.0,1,Y
y b The NIL object 4
r~queue("n", .nil) => 97.1
r~push("n", .nil) => 97.1
```

r[class.circularqueue.queue.sends]
`queue` and `push` send, to the receiver, `items` and then, when the answer is
greater than the size, `pull` for `queue` or, for `push`, `items` again and
`remove` with the answer as the index. With the option `Unique` they send
`hasItem` with the item first and, for as long as it answers true, `removeItem`
with the item and `hasItem` again, and only then the `items`. The strict `==` of
that search is the one of [the Queue search
rule](r:class.queue.removeitem.sends): the first example shows the order. For
the very object that is stored, see [the deviation
rule](r:class.circularqueue.deviation.78). The option is sent
`caselessMatchChar` with the arguments 1 and `u`, whose answer is read as a
logical value: neither 0 nor 1 raises `SYNTAX 34.1`, and an option that does not
understand the message raises `SYNTAX 97.1`, `.nil` included. The second example
shows these.

```rexx
q = .circularqueue~of(.thing~new('p'), .thing~new('q'), .thing~new('p'))
q~resize(4)
q~queue(.thing~new('p'), 'U')
say q~items
q~push(.thing~new('q'), 'U')
say q~items
exit
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
p == p
p == p
p == q
p == p
p == q
p == p
p == q
2
q == q
q == q
q == p
2
```

```rexx
q = .circularqueue~of('a', 'b')
q~resize(4)
q~queue('c', .opt~new(0))
q~queue('c', .opt~new(1))
q~push('c', .opt~new(1))
say q~allItems~makeString('L', ',')
call try 'q~queue("s", .opt~new("x"))'
call try 'q~push("s", .opt~new(.nil))'
call try 'q~queue("s", .object~new)'
say q~allItems~makeString('L', ',')
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class opt
::method init
  expose answer
  use arg answer
::method caselessMatchChar
  expose answer
  use arg pos, set
  say 'caselessMatchChar' pos set
  return answer
```

```output
caselessMatchChar 1 u
caselessMatchChar 1 u
caselessMatchChar 1 u
c,a,b
caselessMatchChar 1 u
q~queue("s", .opt~new("x")) => 34.1
caselessMatchChar 1 u
q~push("s", .opt~new(.nil)) => 34.1
q~queue("s", .object~new) => 97.1
c,a,b
```

r[class.circularqueue.append]
`append(item)` adds the item at the tail and answers its index, which is the
number of items afterwards; a queue of size 0 holds nothing, and `append` then
answers `.nil`. A full queue takes out the item at its head first, so the
index is the size then. It takes exactly one argument: none raises
`SYNTAX 93.901`, and a second raises `SYNTAX 93.902`. After the item is added,
`append` sends `items` to the receiver, and `pull` when the answer is greater
than the size.

```rexx
q = .circularqueue~new(3)
say q~append('a') q~append('b') q~append('c')
say q~append('d') q~allItems~makeString('L', ',') q~items
say q~append('e') q~allItems~makeString('L', ',') q~last
say (.circularqueue~new(0)~append('a') == .nil)
call try 'q~append'
call try 'q~append(, 1)'
call try 'q~append(1, 2)'
say q~allItems~makeString('L', ',')
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
1 2 3
3 b,c,d 3
3 c,d,e 3
1
q~append => 93.901
q~append(, 1) => 93.902
q~append(1, 2) => 93.902
c,d,e
```

r[class.circularqueue.insert]
`insert(item, index)` adds the item directly after the item at the index, as
[the Queue insert rule](r:class.queue.insert) says of any queue, and answers
the index of the new item, which is the index it had when it was added. When
that makes the queue hold more items than its size, the item at the tail is
taken out afterwards: so an item that was added after the last item, by the
omitted `index` or by the index of the last item, is taken out at once from a
full queue, and the answer is then the index above the size. The receiver is
sent `items` after the item is added, and when its answer is greater than the
size `last` and then `remove` with the answer of `last`.

```rexx
q = .circularqueue~of('a', 'b')
q~resize(4)
say q~insert('x', 1) q~allItems~makeString('L', ',')
say q~insert('y', .nil) q~allItems~makeString('L', ',')
say q~insert('z', 2) q~allItems~makeString('L', ',') q~items
say q~insert('w', .nil) q~allItems~makeString('L', ',') q~items
say q~insert('v', 2) q~allItems~makeString('L', ',') q~items
say q~insert('u') q~allItems~makeString('L', ',') q~items
say q~insert('t', q~last) q~allItems~makeString('L', ',') q~items
call try 'q~insert("q", 0)'
call try 'q~insert("q", 99)'
call try 'q~insert("q", 1, 1)'
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
2 a,x,b
1 y,a,x,b
3 y,a,z,x 4
1 w,y,a,z 4
3 w,y,v,a 4
5 w,y,v,a 4
5 w,y,v,a 4
q~insert("q", 0) => 93.907
q~insert("q", 99) => 93.966
q~insert("q", 1, 1) => 93.902
```

## Resizing

r[class.circularqueue.resize]
`resize(newSize, order)` changes the size to `newSize` and answers `newSize`
as it was given. When the queue holds more items than the new size, the
surplus is taken out, from the head for the order `Fifo`, which is the
default, and from the tail for `Lifo`, so that the other end is kept. The
`newSize` is read as the size of [the new rule](r:class.circularqueue.new) is,
with the message `<` and the errors that rule names; a missing `newSize`
raises `SYNTAX 93.901`, an omitted one before an `order` raises
`SYNTAX 93.903`, and a third argument `SYNTAX 93.902`; the arguments are
counted first, then the `newSize` is read, then the order. The first character
of the order, in either case, is `F` or `L`, and an order whose first
character is another, the null string included, raises `SYNTAX 93.914`; the
order is read by [the order rule](r:class.circularqueue.order). A size 0
empties the queue.

```rexx
q = .circularqueue~of(1, 2, 3)
say q~resize(5) q~size q~items q~allItems~makeString('L', ',')
say q~resize(2) q~size q~items q~allItems~makeString('L', ',')
q = .circularqueue~of(1, 2, 3)
say q~resize(2, 'L') q~allItems~makeString('L', ',')
q = .circularqueue~of(1, 2, 3)
say q~resize(2, 'Fifo') q~allItems~makeString('L', ',')
q = .circularqueue~of(1, 2, 3)
say q~resize(0) q~size q~items
q = .circularqueue~of(1, 2, 3)
say q~resize(' 4 ')'|' q~size'|' q~resize(4.0)
say q~queue(4) q~queue(5) q~allItems~makeString('L', ',')
call try 'q~resize'
call try 'q~resize(, "L")'
call try 'q~resize(-1)'
call try 'q~resize("x")'
call try 'q~resize(1.5)'
call try 'q~resize(1000000000)'
call try 'q~resize(.nil)'
call try 'q~resize(1, "x")'
call try 'q~resize(1, "")'
call try 'q~resize(1, .nil)'
call try 'q~resize(1, "L", 1)'
say q~size q~allItems~makeString('L', ',')
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
5 5 3 1,2,3
2 2 2 2,3
2 1,2
2 2,3
0 0 0
 4 |  4 | 4.0
The NIL object 1 2,3,4,5
q~resize => 93.901
q~resize(, "L") => 93.903
q~resize(-1) => 93.906
q~resize("x") => 93.906
q~resize(1.5) => 93.906
q~resize(1000000000) => 93.906
q~resize(.nil) => 97.1
q~resize(1, "x") => 93.914
q~resize(1, "") => 93.914
q~resize(1, .nil) => 97.1
q~resize(1, "L", 1) => 93.902
4.0 2,3,4,5
```

r[class.circularqueue.resize.sends]
`resize` sends `items` to the receiver. When the answer is greater than
`newSize` it sends `items` again, and then `remove` once for each surplus
item: with the index 1 for the order `Fifo`, and with the index `newSize + 1`
for `Lifo`, which is the index of the last item as the removals go on. The
`newSize` and then the order, when given, are read first, the order as [the
order rule](r:class.circularqueue.order) says.

```rexx
.local~log = .array~new
.local~on = 0
a = .spy~new(3)
do i = 1 to 3; a~queue(i); end
call run 'a~resize(5)'
call run 'a~resize(3)'
call run 'a~resize(2)'
call run 'a~resize(2, "L")'
call run 'a~resize(0)'
call run 'a~resize(1, .opt~new)'
exit
run:
  use arg c
  a = .spy~new(3)
  do i = 1 to 3; a~queue(i); end
  .local~log~empty
  .local~on = 1
  interpret 'x =' c
  .local~on = 0
  say left(c, 20) '=>' .local~log~makeString('L', ' ')
  return
::class opt
::method unknown
  use arg name
  .local~log~append(name)
  return 'F'
::class spy subclass circularqueue
::method items
  if .local~on then .local~log~append('items')
  forward class (super)
::method remove
  if .local~on then .local~log~append('remove('arg(1)')')
  forward class (super)
```

```output
a~resize(5)          => items
a~resize(3)          => items
a~resize(2)          => items items remove(1)
a~resize(2, "L")     => items items remove(3)
a~resize(0)          => items items remove(1) remove(1) remove(1)
a~resize(1, .opt~new => UPPER items items remove(1) remove(1)
```

## Parts, arrays and strings

r[class.circularqueue.section]
`section(start, count)` answers a new circular queue of the same class as the
receiver, made by `new` with the size `count`, whose items are read from the
receiver by position: the first at index `start`, the next ones at the indexes
after it, wrapping from the size to the index 1, so that a section that runs
past the tail goes on at the head. A position that holds no item gives `.nil`.
`count` is at most the size of the receiver, and is the size of the receiver
when it is omitted or larger; the new queue holds `count` items, `.nil` ones
included. When `start` is above the size, the first entry is what `[]` answers
for `start`, which is `.nil`, and the positions after it are 1, 2 and so on.
The `start` is read as [the Queue index rule](r:class.queue.indexes) says;
`count` must be a whole number from 0 up, or `SYNTAX 93.906` is raised, `.nil`
raising `SYNTAX 97.1`. No `start` raises `SYNTAX 93.901`, and a third argument
`SYNTAX 93.902`. The arguments are counted first. `count` is then checked with
the messages and conditions of
[the new rule](r:class.circularqueue.new.sends), after which its string value
is read once more; that second value is the count, and one below 0 or not a
whole number raises `SYNTAX 93.906`, so a count whose string value is `-1` is
refused even when its `<` answers 0. `start` is read last.
The answer is not the receiver, and the receiver is not changed.

```rexx
q = .circularqueue~of('a', 'b', 'c', 'd')
call show q~section(2)
call show q~section(3, 2)
call show q~section(1, 2)
call show q~section(2, 0)
call show q~section(2, 6)
call show q~section('2', ' 2 ')
call show q~section(6, 2)
e = .circularqueue~new(5)
e~queue('a'); e~queue('b'); e~queue('c')
call show e~section(1)
call show e~section(3, 4)
call show e~section(2, 99)
say q~items q~size q~allItems~makeString('L', ',')
call try 'q~section'
call try 'q~section(0)'
call try 'q~section(.nil)'
call try 'q~section("x")'
call try 'q~section(0, -1)'
call try 'q~section(1, -1)'
call try 'q~section(1, "x")'
call try 'q~section(1, 1.5)'
call try 'q~section(1, .nil)'
call try 'q~section(2, .loud~new(0, "-1"))'
call try 'q~section(1, 1, 1)'
exit
show: procedure
  use arg s
  say s~class~id s~size s~items '['s~allItems~makeString('L', '|')']'
  return
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class loud
::method init
  expose answer text
  use arg answer, text
::method makeString
  expose text
  say 'makestring'
  return text
::method '<'
  expose answer
  use arg limit
  say 'lt' limit
  return answer
```

```output
CircularQueue 4 4 [b|c|d|a]
CircularQueue 2 2 [c|d]
CircularQueue 2 2 [a|b]
CircularQueue 0 0 []
CircularQueue 4 4 [b|c|d|a]
CircularQueue 2 2 [b|c]
CircularQueue 2 2 [The NIL object|a]
CircularQueue 5 5 [a|b|c|The NIL object|The NIL object]
CircularQueue 4 4 [c|The NIL object|The NIL object|a]
CircularQueue 5 5 [b|c|The NIL object|The NIL object|a]
4 4 a,b,c,d
q~section => 93.901
q~section(0) => 93.907
q~section(.nil) => 93.907
q~section("x") => 93.907
q~section(0, -1) => 93.906
q~section(1, -1) => 93.906
q~section(1, "x") => 93.906
q~section(1, 1.5) => 93.906
q~section(1, .nil) => 97.1
makestring
lt 0
makestring
q~section(2, .loud~new(0, "-1")) => 93.906
q~section(1, 1, 1) => 93.902
```

r[class.circularqueue.section.sends]
`section` sends `new` to the class of the receiver with the size `count`, and
then, for each position in the order of the section, `[]` to the receiver with
the position as the argument and `queue` to the new object with what `[]`
answered; the new object is sent `items` after each `queue`, as [the queue
rule](r:class.circularqueue.queue.sends) says. The `init` of the new object
receives the size.

```rexx
.local~on = 0
q = .spy~of('a', 'b', 'c')
.local~on = 1
s = q~section(2)
say '--'
s = q~section(3, 2)
say '--'
s = q~section(1, 0)
.local~on = 0
say s~class~id s~size
::class spy subclass circularqueue
::method new class
  if .local~on then say 'new' arg(1)
  forward class (super)
::method init
  if .local~on then say 'init' arg(1)
  forward class (super)
::method queue
  if .local~on then say 'queue' arg(1)
  forward class (super)
::method items
  if .local~on then say 'items'
  forward class (super)
::method '[]'
  if .local~on then say '[]' arg(1)
  forward class (super)
```

```output
new 3
init 3
[] 2
queue b
items
[] 3
queue c
items
[] 1
queue a
items
--
new 2
init 2
[] 3
queue c
items
[] 1
queue a
items
--
new 0
init 0
SPY 0
```

r[class.circularqueue.order]
`makeArray(order)` answers a new `Array` of the items, from the head for the
order `Fifo`, which is the default, and from the tail for `Lifo`; `allItems`
answers the items from the head, and `allIndexes` the indexes 1 to the number
of items, as for any queue ([the Queue rule](r:class.queue.makearray)).
`supplier(order)` answers a `Supplier` that goes over the same items, a
snapshot as for any queue. The first character of the `order`, in either case,
is `F` or `L`, the rest does not count, and anything else raises
`SYNTAX 93.914`; an order that is a null string, a number, or a string with a
leading blank is anything else. With `Lifo` the index of an item in the answer
of `supplier` is its index in the queue, so that the supplier counts down.
`makeArray` and `supplier` take at most one argument: a second raises
`SYNTAX 93.902`.

```rexx
x = .circularqueue~of('x', 'y', 'z')
say x~makeArray~makeString('L', ',') '|' x~makeArray('L')~makeString('L', ',')
say x~makeArray('lifo')~makeString('L', ',') '|' x~makeArray('Fifo')~items
say x~supplier~item x~supplier('L')~item x~supplier('Last')~item
s = x~supplier('L')
do while s~available
  say s~index s~item
  s~next
end
say x~allItems~makeString('L', ',') x~allIndexes~makeString('L', ',')
say x~makeArray~class~id x~makeArray('L')~class~id
call try 'x~makeArray("X")'
call try 'x~makeArray("")'
call try 'x~makeArray(" L")'
call try 'x~makeArray(1)'
call try 'x~makeArray(.nil)'
call try 'x~makeArray("L", 1)'
call try 'x~supplier("X")'
call try 'x~supplier(.nil)'
call try 'x~supplier("L", 1)'
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
x,y,z | z,y,x
z,y,x | 3
x z z
3 z
2 y
1 x
x,y,z 1,2,3
Array Array
x~makeArray("X") => 93.914
x~makeArray("") => 93.914
x~makeArray(" L") => 93.914
x~makeArray(1) => 93.914
x~makeArray(.nil) => 97.1
x~makeArray("L", 1) => 93.902
x~supplier("X") => 93.914
x~supplier(.nil) => 97.1
x~supplier("L", 1) => 93.902
```

r[class.circularqueue.order.sends]
`makeArray` and `supplier` read the order by sending it `upper` with no
argument, and use the first character of the answer. `string` and `makeString`
send it `caselessStartsWith` with `F` and, when the answer is not true, with
`L`, and raise `SYNTAX 93.914` when neither is; they pass the order on to
`makeArray`, which reads it as above. `resize` reads it as `makeArray` does.
For the order `Lifo`, `makeArray` and `supplier` send `copy` to the receiver,
then `items` to that copy, then `at` to the copy with each index from that
number down to 1, and the answer holds what `at` answered, in that order;
`Fifo` sends nothing to the receiver. `string` and `makeString` reach these
through the `makeArray` they send. In the first example the order is an object
that answers every message itself; the second shows the messages for `Lifo`.

```rexx
q = .circularqueue~of('a', 'b', 'c')
.local~up = 'LX'
say q~makeArray(.ord~new)~makeString('L', ',')
say q~supplier(.ord~new)~item
.local~up = 'F'
.local~cs = 1
say q~string(',', .ord~new)
.local~cs = 0
call try 'q~makeString(",", .ord~new)'
.local~up = 'Q'
call try 'q~makeArray(.ord~new)'
.local~up = 'L'
say q~resize(2, .ord~new) q~size q~allItems~makeString('L', ',')
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class ord
::method unknown
  use arg name, args
  say name args~makeString('L', ',')
  if name = 'UPPER' then return .local~up
  return .local~cs
```

```output
UPPER 
c,b,a
UPPER 
c
CASELESSSTARTSWITH F
UPPER 
a,b,c
CASELESSSTARTSWITH F
CASELESSSTARTSWITH L
q~makeString(",", .ord~new) => 93.914
UPPER 
q~makeArray(.ord~new) => 93.914
UPPER 
2 2 a,b
```

```rexx
q = .spy~of('a', 'b', 'c')
say '--'
say q~makeArray('L')~makeString('L', ',')
say q~supplier('L')~item
say q~string(',', 'L')
say q~makeArray~makeString('L', ',')
::class spy subclass circularqueue
::method copy
  say 'copy'
  forward class (super)
::method items
  say 'items'
  forward class (super)
::method at
  say 'at' arg(1)
  return 'X'arg(1)
```

```output
items
items
items
--
copy
items
at 3
at 2
at 1
X3,X2,X1
copy
items
at 3
at 2
at 1
X3
copy
items
at 3
at 2
at 1
X3,X2,X1
a,b,c
```

r[class.circularqueue.string]
`string(delimiter, order)` answers a string that joins the items in the order,
with the delimiter between one item and the next, and `makeString` answers
what `string` answers for the same arguments. The delimiter is `,` when it is
omitted, and may be the null string. The string value of a circular queue,
which `say` and the joining operators use, is what its `makeString` answers
with no argument, which is what `string` answers, so that `say` of a circular
queue writes its items separated by commas. An item that is a string is used as
it is and any other object, `.nil`
included, gives the answer of its `objectName`: [the Array
rule](r:class.array.makestring) says the same. A delimiter that is not a string
and [gives no string](r:class.index.string) raises `SYNTAX 88.909`; the order is
checked as
[the order
rule](r:class.circularqueue.order) says, and a third argument raises
`SYNTAX 93.902`. A queue without items answers the null string.

```rexx
x = .circularqueue~of('x', 'y', 'z')
say x '|' x~string '|' x~string(';') '|' x~string('/', 'L')
say x~makeString '|' x~makeString(';') '|' x~makeString('/', 'L')
say x~makeString(, 'L') '|' x~makeString('/', 'Last')
say x~makeString('', 'F')
say .circularqueue~of('a', , 'c')~string '|' .circularqueue~new(2)~string'|'
say .circularqueue~of(.nil, 1.0, 'a b')~makeString('|') x~request('STRING')
call try 'x~makeString("/", "X")'
call try 'x~makeString("/", "")'
call try 'x~makeString("/", .nil)'
call try 'x~makeString("/", "L", 1)'
call try 'x~makeString(.object~new)'
call try 'x~string("/", "X")'
call try 'x~string("/", "L", 1)'
call try 'x~string(.object~new)'
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
x,y,z | x,y,z | x;y;z | z/y/x
x,y,z | x;y;z | z/y/x
z,y,x | z/y/x
xyz
a,,c | |
The NIL object|1.0|a b x,y,z
x~makeString("/", "X") => 93.914
x~makeString("/", "") => 93.914
x~makeString("/", .nil) => 97.1
x~makeString("/", "L", 1) => 93.902
x~makeString(.object~new) => 88.909
x~string("/", "X") => 93.914
x~string("/", "L", 1) => 93.902
x~string(.object~new) => 88.909
```

r[class.circularqueue.string.sends]
`makeString` sends `string` to the receiver with its arguments, and `string`
sends `makeArray` with the order, `F` when it was omitted. The order is read
first, as [the order rule](r:class.circularqueue.order.sends) says. A string
item is sent nothing; each other item is sent `objectName`, from the one that
the answer of `makeArray` holds first, as [the Array
rule](r:class.array.makestring.sends) says of `makeString`; the delimiter is
sent its string value first. An item's own `string` and `makeString` are not
used.

```rexx
q = .circularqueue~of(.it~new('p'), .it~new('q'), 'plain')
say q~string
say q~makeString(.it~new('D'), 'L')
::class it
::method init
  expose n
  use arg n
::method objectName
  expose n
  say '  objectName' n
  return n
::method string
  expose n
  say '  string' n
  return n
::method makeString
  expose n
  say '  makeString' n
  return n
```

```output
  objectName p
  objectName q
p,q,plain
  makeString D
  objectName q
  objectName p
plainDqDp
```

## The set operations

r[class.circularqueue.set]
`union`, `intersection` and `xor` answer a new circular queue of the same
class as the receiver, and the receiver is not changed. `union(other)` holds
the items of the receiver followed by every item of the argument, so an item
that both have is there twice; the size of the answer is the size of the
receiver, or, when the receiver's number of items plus the argument's is
greater, that number. `intersection(other)` holds the items of the argument in
its order that the receiver has, each counting as often as the receiver has
it, as [the OrderedCollection rule](r:class.orderedcollection.intersection)
says, and has the size of the receiver. `xor(other)` holds what [the
OrderedCollection rule](r:class.orderedcollection.xor) says of the items that
only one of the two has, receiver items first, and has the size of the
receiver, or, when it holds more items, their number. `difference`, `subset`,
`disjoint`, `equivalent` and `appendAll` are those of the mixins; `difference`
answers a circular queue of the receiver's class and size, and `appendAll`
answers the receiver and adds through `append`, so that the items that do not
fit push the first ones out. The argument is read as [the operand
rule](r:class.orderedcollection.operand) says, and each takes exactly one:
none raises `SYNTAX 93.901` and a second raises `SYNTAX 93.902`.

```rexx
a = .circularqueue~new(6)
a~queue(1); a~queue(2); a~queue(3)
call show a~union(.array~of(9))
call show a~union(.array~of(9, 8, 7, 6, 5, 4, 3, 2, 1))
call show a~union(.list~of(9))
call show a~union(.array~new)
call show a~intersection(.array~of(2, 2, 3))
call show a~intersection(.array~of(5))
call show a~xor(.array~of(2, 9))
call show a~xor(.array~of(1, 1, 9))
call show a~xor(.array~of(7, 8, 9, 10, 11, 12, 13))
call show a~difference(.array~of(2))
say a~size a~items a~allItems~makeString('L', ',')
b = .circularqueue~new(3)
b~queue(1); b~queue(2); b~queue(3)
call show b~union(.array~of(9))
call show b~union(.array~of(1, 2))
call show b~xor(.array~of(5, 6, 7, 8))
call show b~appendAll(.array~of(7, 8))
call try 'a~union'
call try 'a~xor(.array~of(1), 2)'
call try 'a~intersection(.nil)'
exit
show: procedure
  use arg s
  say s~class~id s~size s~items s~allItems~makeString('L', ',')
  return
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
CircularQueue 6 4 1,2,3,9
CircularQueue 12 12 1,2,3,9,8,7,6,5,4,3,2,1
CircularQueue 6 4 1,2,3,9
CircularQueue 6 3 1,2,3
CircularQueue 6 2 2,3
CircularQueue 6 0 
CircularQueue 6 3 1,3,9
CircularQueue 6 4 2,3,1,9
CircularQueue 10 10 1,2,3,7,8,9,10,11,12,13
CircularQueue 6 2 1,3
6 3 1,2,3
CircularQueue 4 4 1,2,3,9
CircularQueue 5 5 1,2,3,1,2
CircularQueue 7 7 1,2,3,5,6,7,8
CircularQueue 3 3 3,7,8
a~union => 93.901
a~xor(.array~of(1), 2) => 93.902
a~intersection(.nil) => 93.948
```

r[class.circularqueue.set.sends]
The three operations send the messages shown. `union` sends `copy` to the
receiver, which makes the answer, reads the argument as [the operand
rule](r:class.orderedcollection.operand) says, and then sends `items` to the
receiver and `items` to what the argument gave, the array that `makeArray`
answered or the argument itself when it is a collection; when their sum is
greater than the size it sends the two `items` again, and then `resize` with
the sum to the answer. It then sends `allItems` to the argument's list, and
`append` to the answer with each item in order. `intersection` and `xor` send
the argument's list `allItems`; `intersection` sends `copy` to the receiver,
which gives the working copy, `hasItem` to the working copy for each item of
the list, and, when that answered true, `append` to the answer, a new circular
queue, and then `removeItem` to the working copy. `xor` sends `allItems` to
the receiver and `copy`, `hasItem` and, when true, `removeItem` to the working
copy for each item, and then `append` to the answer for each item that is
left, from the receiver's, and for those of the argument that found none.
`difference` sends `copy` and then `removeItem` with each item of the list to
the copy. The sizes the answers get are made by `new`, as the second example
shows.

```rexx
.local~log = .array~new
.local~on = 0
call run 'a~union(.parg~new)', 3
call run 'a~union(.parg~new)', 6
call run 'a~intersection(.parg~new)', 3
call run 'a~xor(.parg~new)', 3
call run 'a~difference(.parg~new)', 3
exit
run:
  use arg c, n
  a = .spy~new(n)
  do i = 1 to 3; a~queue(i); end
  .local~rh = a~identityHash~c2x
  .local~log~empty
  .local~on = 1
  interpret 'x =' c
  .local~on = 0
  out = ''
  do e over .local~log
    parse var e name '@' h
    if h = '' then out = out e
    else if h = .local~rh then out = out name'@R'
    else if h = x~identityHash~c2x then out = out name'@A'
    else out = out name'@C'
  end
  say c n
  do e over out~makeArray(' ')
    say '  'e
  end
  return
::routine note
  use arg text, object
  .local~log~append(text'@'object~identityHash~c2x)
::class spy subclass circularqueue
::method items
  if .local~on then call note 'items', self
  forward class (super)
::method copy
  if .local~on then call note 'copy', self
  forward class (super)
::method resize
  if .local~on then call note 'resize('arg(1)')', self
  forward class (super)
::method append
  if .local~on then call note 'append('arg(1)')', self
  forward class (super)
::method hasItem
  if .local~on then call note 'hasItem('arg(1)')', self
  forward class (super)
::method removeItem
  if .local~on then call note 'removeItem('arg(1)')', self
  forward class (super)
::method allItems
  if .local~on then call note 'allItems', self
  forward class (super)
::class parg
::method isA
  use arg c
  .local~log~append('arg~isA('c~id')')
  return 0
::method makeArray
  .local~log~append('arg~makeArray')
  return .probe~new(.array~of(2, 9))
::class probe
::method init
  expose t
  use arg t
::method unknown
  expose t
  use arg n, a
  .local~log~append('answer~'n)
  forward to (t) message (n) arguments (a)
```

```output
a~union(.parg~new) 3
  
  copy@R
  arg~isA(Collection)
  arg~makeArray
  items@R
  answer~ITEMS
  items@R
  answer~ITEMS
  resize(5)@A
  items@A
  answer~ALLITEMS
  append(2)@A
  items@A
  append(9)@A
  items@A
a~union(.parg~new) 6
  
  copy@R
  arg~isA(Collection)
  arg~makeArray
  items@R
  answer~ITEMS
  answer~ALLITEMS
  append(2)@A
  items@A
  append(9)@A
  items@A
a~intersection(.parg~new) 3
  
  copy@R
  arg~isA(Collection)
  arg~makeArray
  answer~ALLITEMS
  hasItem(2)@C
  append(2)@A
  items@A
  removeItem(2)@C
  hasItem(9)@C
a~xor(.parg~new) 3
  
  allItems@R
  copy@R
  arg~isA(Collection)
  arg~makeArray
  answer~ALLITEMS
  hasItem(2)@C
  removeItem(2)@C
  hasItem(9)@C
  append(1)@A
  items@A
  append(3)@A
  items@A
  append(9)@A
  items@A
a~difference(.parg~new) 3
  
  copy@R
  arg~isA(Collection)
  arg~makeArray
  answer~ALLITEMS
  removeItem(2)@A
  removeItem(9)@A
```

```rexx
.local~on = 0
a = .watch~new(3)
a~queue(1); a~queue(2); a~queue(3)
.local~on = 1
calls = 'union(.array~of(9))|intersection(.array~of(2))|xor(.array~of(2, 9))',
  '|xor(.array~of(5, 6, 7, 8))|difference(.array~of(2))|section(2)|copy'
do c over calls~makeArray('|')
  say c
  interpret 'x = a~'c
  say '  ->' x~class~id x~size x~items
end
::class watch subclass circularqueue
::method new class
  if .local~on then say '  new' arg(1)
  forward class (super)
::method init
  if .local~on then say '  init' arg(1)
  forward class (super)
```

```output
union(.array~of(9))
  -> WATCH 4 4
intersection(.array~of(2))
  new 3
  init 3
  -> WATCH 3 1
xor(.array~of(2, 9)) 
  new 3
  init 3
  -> WATCH 3 3
xor(.array~of(5, 6, 7, 8))
  new 7
  init 7
  -> WATCH 7 7
difference(.array~of(2))
  -> WATCH 3 2
section(2)
  new 3
  init 3
  -> WATCH 3 3
copy
  -> WATCH 3 3
```

## Copies

r[class.circularqueue.copy]
`copy` answers a new circular queue of the same class as the receiver with the
same size and the same items, so that neither sees the other's changes; the
items are the same objects. As [the Queue copy rule](r:class.queue.copy) says,
the variables of a subclass instance are copied as they are, and `init` is not
sent. The method takes no argument: one raises `SYNTAX 93.902`.

```rexx
q = .many~new(4)
q~queue('a'); q~queue('b'); q~queue('c')
q~tag = 'T'
c = q~copy
say c~class~id c~size c~items c~tag c~allItems~makeString('L', ',') (c == q)
c~queue('n'); c~queue('o')
say q~items c~items q~allItems~makeString('L', ',')
say c~allItems~makeString('L', ',')
call try 'q~copy(1)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class many subclass circularqueue
::attribute tag
::method init
  say 'init' arg(1)
  forward class (super)
```

```output
init 4
MANY 4 3 T a,b,c 0
3 4 a,b,c
b,c,n,o
q~copy(1) => 93.902
```

## The methods of Queue

r[class.circularqueue.inherited]
The other methods are those of `Queue`, with the same arguments and answers
for the items that are there: `at` and `[]` ([the at rule](r:class.queue.at)),
`put` and `[]=`, `hasIndex`, `hasItem`, `index`, `removeItem`, `delete` and
`remove`, `first`, `firstItem`, `last`, `lastItem`, `peek`, `pull`, `next`,
`previous`, `items`, `isEmpty`, `empty`, `allItems`, `allIndexes` and the
sorts, and the methods of the mixins that this page does not list. The
differences are these. The size of the queue does not widen the indexes that
`put` and `insert` accept: either raises `SYNTAX 93.966` with an index above
the number of items, whatever the size is ([Deviation
121](r:class.circularqueue.deviation.121)). `empty` keeps the size. Each of
the methods sends only the messages that [the Queue rule](r:class.queue.sends)
lists. The example shows these.

```rexx
q = .circularqueue~new(5)
q~queue('c'); q~queue('a'); q~queue('b')
say q~at(2) q[3] q~first q~last q~firstItem q~lastItem q~peek q~index('b')
say q~hasItem('a') q~hasIndex(4) q~next(1) q~previous(3) q~items q~isEmpty
say q~removeItem('a') q~delete(1) q~remove(9) q~allItems~makeString('L', ',')
q~put('z', 1)
say q~sort~allItems~makeString('L', ',') q~size
say q~pull q~pull (q~pull == .nil) q~items q~size
q~queue('p'); q~queue('q')
say (q~empty == q) q~items q~size
call try 'q~insert("x", 1)'
call try 'q~at(0)'
call try 'q~hasIndex'
call try 'q~items(1)'
call try 'q~peek(1)'
call try 'q~pull(1)'
say q~allItems~class~id q~class~id
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
a b 1 3 c b c 3
1 0 2 2 3 0
a c The NIL object b
z 5
z The NIL object 1 0 5
1 0 5
q~insert("x", 1) => 93.966
q~at(0) => 93.907
q~hasIndex => 93.901
q~items(1) => 93.902
q~peek(1) => 93.902
q~pull(1) => 93.902
Array CircularQueue
```

## Messages the methods send

r[class.circularqueue.sends]
The methods that `CircularQueue` declares send these messages to the receiver:
`items`, `pull`, `last`, `remove`, `hasItem`, `removeItem`, `copy`, `allItems`,
`[]`, `makeArray` and `string`, and `items` and `at` to the copy that
`makeArray`
and `supplier` make for the order `Lifo`; the rules above say which each one
sends, in which order and with which arguments, and [the set
rule](r:class.circularqueue.set.sends) those of the three set operations.
`append`, `insert`, `push` and `queue` add the item themselves: none of them
sends `append`, `queue` or `push` to the receiver. The example records, for
each method, what it sends to a receiver whose methods are overridden to
record themselves, with a queue that has room for one more item and with one
that is full.

```rexx
own = 'append at empty items last makeArray remove copy pull push queue',
  'resize hasItem removeItem supplier string makeString insert'
.local~log = .array~new
do m over own~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"('arg(1)')');",
    'forward class (super)'
  .spy~define(m, src)
end
.local~on = 0
say 'room for one more'
call run 4, 'a~append("d")'
call run 4, 'a~queue("d")'
call run 4, 'a~push("d")'
call run 4, 'a~insert("d", 1)'
say 'full'
call run 3, 'a~append("d")'
call run 3, 'a~queue("d")'
call run 3, 'a~push("d")'
call run 3, 'a~insert("d", 1)'
call run 3, 'a~insert("d", .nil)'
call run 3, 'a~insert("d")'
call run 3, 'a~queue("2", "U")'
call run 3, 'a~push("2", "U")'
call run 3, 'a~resize(5)'
call run 3, 'a~resize(2)'
call run 3, 'a~resize(2, "L")'
call run 3, 'a~resize(0)'
exit
run:
  use arg n, c
  a = .spy~new(n)
  do i = 1 to 3; a~queue(i); end
  .local~log~empty
  .local~on = 1
  interpret 'x =' c
  .local~on = 0
  say c n
  do e over .local~log
    say '  'e
  end
  return
::class spy subclass circularqueue
```

```output
room for one more
a~append("d") 4
  append(d)
  items()
a~queue("d") 4
  queue(d)
  items()
a~push("d") 4
  push(d)
  items()
a~insert("d", 1) 4
  insert(d)
  items()
full
a~append("d") 3
  append(d)
  items()
  pull()
a~queue("d") 3
  queue(d)
  items()
  pull()
a~push("d") 3
  push(d)
  items()
  items()
  remove(3)
a~insert("d", 1) 3
  insert(d)
  items()
  last()
  remove(4)
a~insert("d", .nil) 3
  insert(d)
  items()
  last()
  remove(4)
a~insert("d") 3
  insert(d)
  items()
  last()
  remove(4)
a~queue("2", "U") 3
  queue(2)
  hasItem(2)
  removeItem(2)
  hasItem(2)
  items()
a~push("2", "U") 3
  push(2)
  hasItem(2)
  removeItem(2)
  hasItem(2)
  items()
a~resize(5) 3
  resize(5)
  items()
a~resize(2) 3
  resize(2)
  items()
  items()
  remove(1)
a~resize(2, "L") 3
  resize(2)
  items()
  items()
  remove(3)
a~resize(0) 3
  resize(0)
  items()
  items()
  remove(1)
  remove(1)
  remove(1)
```

```rexx
own = 'allItems append at empty items last makeArray remove copy pull push',
  'queue resize hasItem removeItem supplier string makeString first next'
.local~log = .array~new
do m over own~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"('arg(1)')');",
    'forward class (super)'
  .spy~define(m, src)
end
.local~on = 0
call run 'a~makeArray'
call run 'a~makeArray("F")'
call run 'a~makeArray("L")'
call run 'a~supplier'
call run 'a~supplier("L")'
call run 'a~string'
call run 'a~string(",", "L")'
call run 'a~makeString'
call run 'a~makeString(",", "L")'
call run 'a~section(2)'
call run 'a~allItems'
exit
run:
  use arg c
  a = .spy~new(3)
  do i = 1 to 3; a~queue(i); end
  .local~log~empty
  .local~on = 1
  interpret 'x =' c
  .local~on = 0
  say c
  do e over .local~log
    say '  'e
  end
  return
::class spy subclass circularqueue
```

```output
a~makeArray
  makeArray()
a~makeArray("F")
  makeArray(F)
a~makeArray("L")
  makeArray(L)
  copy()
  items()
  at(3)
  at(2)
  at(1)
a~supplier
  supplier()
a~supplier("L")
  supplier(L)
  copy()
  items()
  at(3)
  at(2)
  at(1)
a~string
  string()
  makeArray(F)
a~string(",", "L")
  string(,)
  makeArray(L)
  copy()
  items()
  at(3)
  at(2)
  at(1)
a~makeString
  makeString()
  string()
  makeArray(F)
a~makeString(",", "L")
  makeString(,)
  string(,)
  makeArray(L)
  copy()
  items()
  at(3)
  at(2)
  at(1)
a~section(2)
  queue(2)
  items()
  queue(3)
  items()
  queue(1)
  items()
a~allItems
  allItems()
```

## Deviations

r[class.circularqueue.deviation.78]
A search for an item with the very object that is stored, by `hasItem`,
`index` or `removeItem`, sends that object's `==` as for any other item, so
an object whose `==` answers false is not found even at its own entry.
ooRexx finds the very object without sending it `==`, and sends `==` only
for the items before it. See [Deviation 78](../deviations.md).

```rexx
.local~n = 0
x = .liar~new
c = .circularqueue~new(5)
c~append('a')
c~append(x)
c~append('b')
say c~hasItem(x) .n
.local~n = 0
say (c~index(x) == .nil) .n
.local~n = 0
say (.nil == c~removeItem(x)) c~items .n
::class liar
::method '=='
  .local~n += 1
  return 0
```

```output
0 3
1 3
1 3 3
```

r[class.circularqueue.deviation.109]
An index, a count, a size given to `new` or `resize`, or a delimiter that is
an instance of a subclass of `String` is read by its own characters, and
`REQUEST` and `MAKESTRING` are not sent to it ([Deviation
109](r:class.string.deviation.109)). ooRexx sends such an instance
`request` with `STRING` or `MAKESTRING`, so an override of either can change
what is read. In the example `MAKESTRING` answers another index,
count, size and delimiter.

```rexx
q = .circularqueue~of('first', 'second', 'third')
say q~at(.num~new('1'))
say q~section(1, .num~new('1'))~items
r = .circularqueue~new(.num~new('1'))
r~queue('a'); r~queue('b')
say r~items
r~resize(.num~new('1'))
say r~items
say q~makeString(.sep~new('-'))
::class num subclass string
::method makestring
  return '2'
::class sep subclass string
::method makestring
  return '+'
```

```output defect=#140
first
1
1
1
first-second-third
```

r[class.circularqueue.deviation.121]
`put` with an index above the number of items raises `SYNTAX 93.966` however
far above it the index is and whatever the size of the queue is, and the queue
keeps its items. ooRexx raises `93.966` or `93.918` according to the storage
the queue holds beyond its items, which its earlier history decides: an index
just above the last item can answer either, and a much larger one answers
`93.918`.

```rexx
q = .circularqueue~new(30)
q~queue('a'); q~queue('b'); q~queue('c')
do i over .array~of(4, 20, 1000)
  call try 'q~put("x", 'i')'
end
say q~allItems~makeString('L', ',')
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
q~put("x", 4) => 93.966
q~put("x", 20) => 93.966
q~put("x", 1000) => 93.966
a,b,c
```
