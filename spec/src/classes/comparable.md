---
element: "Comparable"
kind: class
rexxref: "Builtin Classes > Utility Classes > Comparable Class; Builtin Classes > Utility Classes > Comparable Class > compareTo"
---

# Comparable

`Comparable` is a mixin class that stands for the objects that can be put in
order. It holds one method, `compareTo`, which compares the receiver with
another object and answers a number whose sign tells which one is the greater.
A class that inherits it overrides `compareTo` with its own ordering; what the
class holds itself is only a default that orders objects by their identity,
which no program should rely on. The comparison operators are not part of it:
[`Orderable`](r:class.orderable.creation) has them. The class is the base of
the ordering that [`sort`](r:class.array.sort) uses and that [the default
comparator](r:class.comparators.comparator.compare) sends.

## The class

r[class.comparable.creation]
`Comparable` is a mixin class ([the mixin rule](r:class.index.mixins)) whose
only superclass is `Object`. It defines one instance method, `compareTo`, no
other method of its own, and no class method besides the `new` that every class
has. `new` takes no argument, and any argument raises `SYNTAX 93.902` ([the new
rule](r:class.index.new)). An instance that is a plain `Comparable` can be made.
`String`, `DateTime`, `TimeSpan` and `File` inherit `Comparable`, and the last
three inherit `Orderable` as well ([the hierarchy](r:class.index.hierarchy)); an
instance of `Orderable` is not an instance of `Comparable`.

```rexx
do c over .comparable~superclasses
  say c~id
end
do n over 'compareTo = < hashCode string'~makearray(' ')
  say n owns(n)
end
say .comparable~new~class~id .comparable~new~isA(.orderable)
say .timespan~new(1)~isA(.comparable) 'x'~isA(.comparable)
signal on syntax name bad
say .comparable~new(1)
bad: say condition('o')~code
exit
owns: procedure
  signal on syntax name no
  return .comparable~method(arg(1))~scope == .comparable
no: return 0
```

```output
Object
compareTo 1
= 0
< 0
hashCode 0
string 0
Comparable 0
1 1
93.902
```

r[class.comparable.compareto]
`compareTo` takes one argument. The default `compareTo` sends `identityHash` to
the receiver and then to the argument, and answers -1, 0 or 1 as the number that
the first answered is less than, equal to or greater than the one that the
second answered, compared to 18 significant digits whatever `NUMERIC DIGITS` the
program has set. Which of two objects is the lesser is not specified, but the
same object is always equal to itself, and so are two objects whose
`identityHash` answers are equal numbers. An answer of `identityHash` that is no
number raises `SYNTAX 41.1`. A missing argument raises `SYNTAX 93.901` and a
second one `SYNTAX 93.902`. Nothing but `identityHash` is sent to the argument.
The example gives its objects an `identityHash` of its own, so that the order is
known; the line that ends in `R R` shows the two sends.

```rexx
a = .item~new(3)
b = .item~new(9)
c = .item~new(3)
say a~compareTo(b) b~compareTo(a) a~compareTo(c) a~compareTo(a)
say .item~new(10)~compareTo(.item~new(9)) .item~new(2.50)~compareTo(c)
x = .comparable~new
say x~compareTo(x)
say .item~new(123456789012345678)~compareTo(.item~new(123456789012345679)),
  .item~new(1234567890123456789)~compareTo(.item~new(1234567890123456788))
numeric digits 3
say .item~new(1000)~compareTo(.item~new(1001))
numeric digits 9
.local~log = ''
say .item~new(1)~compareTo(.item~new(2)) .local~log
call try 'a~compareTo'
call try 'a~compareTo(b, c)'
call try '.item~new("abc")~compareTo(b)'
call try 'a~compareTo(.item~new(""))'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class item subclass comparable
::method init
  expose h tag
  use arg h
  tag = 'x'
::method identityHash
  expose h
  .local~log = .local~log 'R'
  return h
```

```output
-1 1 0 0
1 -1
0
-1 0
-1
-1  R R
a~compareTo => 93.901
a~compareTo(b, c) => 93.902
.item~new("abc")~compareTo(b) => 41.1
a~compareTo(.item~new("")) => 41.1
```
