---
element: "StreamSupplier"
kind: class
rexxref: "Builtin Classes > Utility Classes > StreamSupplier Class; Builtin Classes > Utility Classes > StreamSupplier Class > available; Builtin Classes > Utility Classes > StreamSupplier Class > index; Builtin Classes > Utility Classes > StreamSupplier Class > init; Builtin Classes > Utility Classes > StreamSupplier Class > item; Builtin Classes > Utility Classes > StreamSupplier Class > next"
---

# StreamSupplier

A `StreamSupplier` is a [`Supplier`](supplier.md) over the lines of a stream.
Its entries are the lines that are left to read, the index of an entry is the
number of its line in the stream, and the item is the text of the line without
its line end, or the rest of it for the first entry ([Deviation
140](r:class.stream.deviation.140)). The `supplier` method of a stream answers
one; what a stream is and how it reads and writes is specified in the stream
chapter, and this page states only what an example on a file that the example
writes and deletes shows. The examples write a file under a name of their own,
and remove it.

## The class

r[class.streamsupplier.creation]
`StreamSupplier` is a subclass of `Supplier` and inherits no mixin class. It
has the instance methods `available`, `index`, `init`, `item` and `next`, and
inherits the class method `new` and the instance methods `allIndexes`,
`allItems` and `supplier` with the behaviour [the page of
`Supplier`](supplier.md) gives them, except as the rules below say. It may
have private methods besides these; they are not part of this specification.
`new(stream)` makes an instance of the class that received it and sends `init`
the arguments of `new`, as [the new rule](r:class.index.new) says. The `init`
takes the stream as its first argument and reads the first line from it; it
does not take the arrays of `Supplier`. An argument after the first is not
used. An argument that is not a stream, which includes a missing one, raises
`SYNTAX 97.1`, because the object does not answer the messages that `init`
sends it.

```rexx
f = 'spec-streamsupplier-creation-zq.tmp'
call sysfiledelete f
call lineout f, 'first'
call lineout f, 'second'
call lineout f
st = .stream~new(f)
s = .streamsupplier~new(st)
say s~class~id s~class~superclass~id .streamsupplier~queryMixinClass
say s~index s~item
t = .streamsupplier~new(.stream~new(f), 'ignored', 3)
say t~index t~item
u = .mine~new(.stream~new(f))
say u~class~id u~item
call try '.streamsupplier~new'
call try '.streamsupplier~new(1)'
call try '.streamsupplier~new(.nil)'
call try '.streamsupplier~new(.object~new)'
call sysfiledelete f
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) '=> made'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class mine subclass streamsupplier
```

```output
StreamSupplier Supplier 0
1 first
1 first
MINE first
.streamsupplier~new => 97.1
.streamsupplier~new(1) => 97.1
.streamsupplier~new(.nil) => 97.1
.streamsupplier~new(.object~new) => 97.1
```

## Walking the lines

r[class.streamsupplier.entry]
A new supplier is at the read position of the stream: line 1 for a stream that
was not read from yet, and the line after the last whole line read for one
that was. A stream that was read part of the way into a line gives a supplier
that is at that line, whose first item is the rest of it, as a plain `lineIn`
would answer ([Deviation 140](r:class.stream.deviation.140)). `index` answers
the number of the line, `item` its text, and `available` answers 1. Making the
supplier reads that line, so the stream's read position is after it. A stream
that has no line left, such as an empty file, and one that cannot be read,
such as a file that does not exist, gives a supplier that is not available,
and no error. An empty line is an item that is the empty string, and the last
line is an item whether or not it ends with a line end.

```rexx
f = 'spec-streamsupplier-entry-zq.tmp'
call sysfiledelete f
call charout f, 'one'||'0a'x||'0a'x||'three'||'0a'x||'last'
call lineout f
st = .stream~new(f)
say st~linein
s = st~supplier
say s~index s~item
say st~linein
call sysfiledelete f
call charout f, 'x'
call lineout f
all = .stream~new(f)~supplier
do while all~available
  say all~index '['all~item']'
  all~next
end
call sysfiledelete f
call charout f, ''
call lineout f
say .stream~new(f)~supplier~available
call sysfiledelete f
say .stream~new(f)~supplier~available
```

```output
one
2 
three
1 [x]
0
0
```

r[class.streamsupplier.next]
`next` moves to the line after the one the supplier is at: it reads the line
whose number is one more than the index, by that number, and the stream's read
position is after it. When the stream has no such line the supplier is at its
end: `available` answers 0, and `index`, `item` and `next` raise
`SYNTAX 93.937`, as for any [`Supplier`](r:class.supplier.next). `available`,
`index` and `item` read nothing from the stream, and each takes no argument:
an argument raises `SYNTAX 93.902`.

```rexx
f = 'spec-streamsupplier-next-zq.tmp'
call sysfiledelete f
do l over .array~of('l1', 'l2', 'l3', 'l4')
  call lineout f, l
end
call lineout f
st = .stream~new(f)
s = st~supplier
say s~index s~item st~lines
s~next
say s~index s~item st~lines
say st~linein
s~next; s~next
say s~index s~item s~available
s~next
say s~available
call try 's~item'
call try 's~index'
call try 's~next'
call try 's~available(1)'
call sysfiledelete f
exit
try:
  signal on syntax name bad
  interpret arg(1)
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1 l1 3
2 l2 2
l3
4 l4 1
0
s~item => 93.937
s~index => 93.937
s~next => 93.937
s~available(1) => 93.902
```

r[class.streamsupplier.sends]
`init` sends the stream `state` twice and then sends the
receiver `next`, which sends the stream `lineIn` with the number of the line to
read, as [the rule of `supplier`](r:class.stream.supplier.sends) says of a
stream: line 1 unless the second answer of `state` is `READY`. So the receiver
is sent `next` once when it is made, and an override of `next` takes part
there. `init` also sends the stream private messages of its own, which are not
part of this specification; an argument that does not answer them raises
`SYNTAX 97.1` as [the creation rule](r:class.streamsupplier.creation) says, so
an object that is not a stream is accepted as the stream if it answers them,
for instance by an `UNKNOWN` method. Each later `next` sends the stream one
`lineIn`, whose argument is one more than the index, and nothing else.
`available`, `index`, `item` and `copy` send nothing, to the receiver or to the
stream, and `supplier` answers the receiver without a message. `allItems` and
`allIndexes` are those of [`Supplier`](r:class.supplier.allitems.sends) for an
instance of a subclass: they send the receiver `available`, `index`, `item` and
`next` until `available` answers false, so the stream is sent one `lineIn` for
every line that is left and one more, the one that ends the walk, and once a
call has collected the arrays, the other call sends the stream nothing. An entry is the answer of a `lineIn` that does not raise NOTREADY. A `lineIn`
that raises it ends the supplier, whether it was sent by `init` or by `next`:
the supplier is then not available, and what the `lineIn` answered is not an
item. A `lineIn` that answers no value raises `SYNTAX 91.999` from the method
that sent it, and any other error that the stream raises is raised by that
method as it is.

```rexx
.local~log = .array~new
s = .spy~new(.stand~new(3))
call show 'new'
s~next
call show 'next'
x = s~available; y = s~index; z = s~item
call show 'read'
c = s~copy
call show 'copy'
say c~class~id c~index c~item
.log~empty
t = .spy~new(.stand~new(3))
call show 'new'
a = t~allItems
call show 'allItems'
b = t~allIndexes
call show 'allIndexes'
say a~makeString('L', ',') b~makeString('L', ',') t~available
exit
show:
  say strip(arg(1)':' .log~makeString('L', ' '))
  .log~empty
  return
::class spy subclass streamsupplier
::method available
  .log~append('available'); return self~available:super
::method index
  .log~append('index'); return self~index:super
::method item
  .log~append('item'); return self~item:super
::method next
  .log~append('next'); self~next:super
::method copy
  .log~append('copy'); return self~copy:super
::class stand
::method init
  expose left
  use arg left
::method unknown
  use arg name
  if name == 'STATE' then do
    .log~append('state')
    return 'UNKNOWN'
  end
  return 1
::method linein
  expose left
  use arg number
  .log~append('lineIn('number')')
  if number > left then raise notready
  return 'line' number
```

```output
new: state state next lineIn(1)
next: next lineIn(2)
read: available index item
copy: copy
SPY 2 line 2
new: state state next lineIn(1)
allItems: available index item next lineIn(2) available index item next lineIn(3) available index item next lineIn(4) available
allIndexes:
line 1,line 2,line 3 1,2,3 0
```

```rexx
call try 'empty'
call try 'none'
call try 'error'
call try 'later'
call try 'ends'
exit
try:
  mode = arg(1)
  signal on syntax name bad
  s = .streamsupplier~new(.stand~new(mode))
  say mode 'made' s~available
  if s~available then do
    s~next
    say mode 'next' s~available
    s~next
    say mode 'again' s~available
  end
  return
bad: say mode '=>' condition('o')~code
  return
::class stand
::method init
  expose mode calls
  use arg mode
  calls = 0
::method unknown
  return 'UNKNOWN'
::method linein
  expose mode calls
  calls += 1
  select
    when mode == 'empty' then raise notready
    when mode == 'none' then return
    when mode == 'error' then raise syntax 40.900 array ('stand-in')
    when mode == 'ends' & calls > 1 then raise notready
    when mode == 'later' & calls > 1 then return
    otherwise return 'line' calls
  end
```

```output
empty made 0
none => 91.999
error => 40.900
later made 1
later => 91.999
ends made 1
ends next 0
ends => 93.937
```

r[class.streamsupplier.rest]
The methods that `StreamSupplier` inherits work on the lines that are left.
`allItems` answers an `Array` of the texts of the lines from the one the
supplier is at to the last, `allIndexes` an `Array` of their numbers, and both
leave the supplier at its end and answer the same arrays whenever they are
sent again, as [the rule of `Supplier`](r:class.supplier.allitems) says.
`supplier` answers the supplier itself. `copy` answers another supplier of the
same class at the same line, over the same stream: each of the two moves on
its own, and every line after the first is read by its number, so after the
copy has moved the original reads the line that follows its own, and not the
one after the copy's.

```rexx
f = 'spec-streamsupplier-rest-zq.tmp'
call sysfiledelete f
do l over .array~of('l1', 'l2', 'l3', 'l4')
  call lineout f, l
end
call lineout f
st = .stream~new(f)
say st~linein
s = st~supplier
say s~allIndexes~makeString('L', ',') s~allItems~makeString('L', ',')
say s~available (s~allItems == s~allItems)
st = .stream~new(f)
t = st~supplier
c = t~copy
say (t~supplier == t) (c == t) c~class~id
t~next; t~next
say t~item c~item
c~next
say c~item st~linein
t~next
say t~item c~item
call sysfiledelete f
```

```output
l1
2,3,4 l2,l3,l4
0 1
1 0 StreamSupplier
l3 l1
l2 l3
l4 l2
```
