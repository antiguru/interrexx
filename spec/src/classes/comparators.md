---
element: "Comparator"
kind: class
rexxref: "Builtin Classes > Utility Classes > Comparator Classes; Builtin Classes > Utility Classes > Comparator Classes > Comparator Class; Builtin Classes > Utility Classes > Comparator Classes > Comparator Class > compare; Builtin Classes > Utility Classes > Comparator Classes > CaselessComparator Class; Builtin Classes > Utility Classes > Comparator Classes > CaselessComparator Class > compare; Builtin Classes > Utility Classes > Comparator Classes > ColumnComparator Class; Builtin Classes > Utility Classes > Comparator Classes > ColumnComparator Class > compare; Builtin Classes > Utility Classes > Comparator Classes > ColumnComparator Class > init; Builtin Classes > Utility Classes > Comparator Classes > CaselessColumnComparator Class; Builtin Classes > Utility Classes > Comparator Classes > CaselessColumnComparator Class > compare; Builtin Classes > Utility Classes > Comparator Classes > CaselessColumnComparator Class > init; Builtin Classes > Utility Classes > Comparator Classes > DescendingComparator Class; Builtin Classes > Utility Classes > Comparator Classes > DescendingComparator Class > compare; Builtin Classes > Utility Classes > Comparator Classes > CaselessDescendingComparator Class; Builtin Classes > Utility Classes > Comparator Classes > CaselessDescendingComparator Class > compare; Builtin Classes > Utility Classes > Comparator Classes > InvertingComparator Class; Builtin Classes > Utility Classes > Comparator Classes > InvertingComparator Class > compare; Builtin Classes > Utility Classes > Comparator Classes > InvertingComparator Class > init; Builtin Classes > Utility Classes > Comparator Classes > NumericComparator Class; Builtin Classes > Utility Classes > Comparator Classes > NumericComparator Class > compare; Builtin Classes > Utility Classes > Comparator Classes > NumericComparator Class > init"
deviations: [109]
---

# Comparators

A comparator is an object that orders two others. It has one method,
`compare`, which takes two objects and answers a number: negative when the
first is to come before the second, 0 when the order does not tell them apart,
and positive when the first is to come after. The sorting methods that take a
comparator, such as [`sortWith`](r:class.array.sort), send it `compare` and
need nothing else of it, so any object with such a method will do. This page
describes `Comparator` and the seven classes below it: `CaselessComparator`,
`ColumnComparator`, `CaselessColumnComparator`, `DescendingComparator`,
`CaselessDescendingComparator`, `InvertingComparator` and `NumericComparator`.
Each is made by `new` like any class; the `compare` of each hands the two
objects to a method of the first or to a comparator that the instance holds,
and says what it answers, so what a comparator answers for an object that is
not a string depends on that object's own methods.

## The classes

r[class.comparators.creation]
`Comparator` is a mixin class ([the mixin rule](r:class.index.mixins)) whose
only superclass is `Object`; each of the other seven classes is a mixin class
whose only superclass is `Comparator`. Every one of the eight defines its own
`compare`. `ColumnComparator`, `CaselessColumnComparator`,
`InvertingComparator` and `NumericComparator` define `init` as well; the other
four have the `init` of `Object` and take no argument in `new`, so that an
argument raises `SYNTAX 93.902` ([the new rule](r:class.index.new)). An
instance of each of the eight can be made by `new`. An instance of one of them
is not an instance of `Comparable` or `Orderable`, and a program's own
comparator needs to inherit nothing: any class with a `compare` method works
where a comparator is wanted.

```rexx
names = 'Comparator CaselessComparator ColumnComparator',
  'CaselessColumnComparator DescendingComparator',
  'CaselessDescendingComparator InvertingComparator NumericComparator'
do n over names~makearray(' ')
  c = .environment~at(n~upper)
  line = n~left(28)
  do s over c~superclasses
    line = line s~id
  end
  say line~left(46) owns(c, 'compare') owns(c, 'init')
end
c = .comparator~new
n = .numericComparator~new
say c~isA(.comparable) c~isA(.orderable) n~isA(.comparator)
signal on syntax name bad
say .caselessComparator~new(1)
bad: say condition('o')~code
exit
owns: procedure
  signal on syntax name no
  return arg(1)~method(arg(2))~scope == arg(1)
no: return 0
```

```output
Comparator                   Object            1 0
CaselessComparator           Comparator        1 0
ColumnComparator             Comparator        1 1
CaselessColumnComparator     Comparator        1 1
DescendingComparator         Comparator        1 0
CaselessDescendingComparator Comparator        1 0
InvertingComparator          Comparator        1 1
NumericComparator            Comparator        1 1
0 0 1
93.902
```

## Comparator

r[class.comparators.comparator.compare]
The `compare` of `Comparator` takes two arguments and sends `compareTo` to the
first, with the second as its only argument, and answers what `compareTo`
answers, unchanged and without examining it. So two strings are ordered by
[the string rule](r:class.string.compareto), two `DateTime` or two `TimeSpan`
instances by theirs, and any other object by its own `compareTo`; a first
argument that has no `compareTo` raises `SYNTAX 97.1`, and a second one that
the `compareTo` of a string cannot read raises that method's error. A missing
argument raises `SYNTAX 93.901`, an omitted first argument before a given
second `SYNTAX 93.903`, and a third argument `SYNTAX 93.902`; the arguments
are checked before anything is sent. Nothing is sent to the second argument
by `compare` itself.

```rexx
c = .comparator~new
say c~compare('a', 'b') c~compare('b', 'a') c~compare('b', 'b')
say c~compare('10', '9') c~compare(10, 9) c~compare('A', 'a')
say c~compare(.timespan~new(1), .timespan~new(2)),
  c~compare(.timespan~new(2), .timespan~new(1))
say c~compare(.pad~new(5), .pad~new(2)) c~compare(.pad~new('x'), 1)
call try 'c~compare("a")'
call try 'c~compare(, "a")'
call try 'c~compare'
call try 'c~compare("a", "b", "c")'
call try 'c~compare(.nil, "a")'
call try 'c~compare(.object~new, "a")'
call try 'c~compare("a", .nil)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class pad
::method init
  expose v
  use arg v
::method compareTo
  expose v
  say 'compareTo' arg(1)~class~id
  return v
```

```output
-1 1 0
-1 -1 -1
-1 1
compareTo PAD
compareTo String
5 x
c~compare("a") => 93.901
c~compare(, "a") => 93.903
c~compare => 93.901
c~compare("a", "b", "c") => 93.902
c~compare(.nil, "a") => 97.1
c~compare(.object~new, "a") => 97.1
c~compare("a", .nil) => 88.909
```

## The caseless and descending comparators

r[class.comparators.caseless.compare]
The `compare` of `CaselessComparator` takes two arguments and sends
`caselessCompareTo` to the first, with the second as its only argument, and
answers what it answers ([the string rule](r:class.string.caselesscompareto)):
0 for two strings that differ in the case of letters alone. A first argument
that has no `caselessCompareTo` raises `SYNTAX 97.1`; the second is read by
that method as a string. Argument errors are those of
[`Comparator`](r:class.comparators.comparator.compare).

```rexx
c = .caselessComparator~new
say c~compare('a', 'A') c~compare('a', 'B') c~compare('B', 'a')
say c~compare('Rye', 'ryegrass') c~compare(10, 9) c~compare('', '')
say c~compare('a', .pad~new)
call try 'c~compare("a")'
call try 'c~compare("a", "b", "c")'
call try 'c~compare(.pad~new, "a")'
call try 'c~compare(.nil, "a")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class pad
::method makestring
  return 'A'
```

```output
0 -1 1
-1 -1 0
0
c~compare("a") => 93.901
c~compare("a", "b", "c") => 93.902
c~compare(.pad~new, "a") => 97.1
c~compare(.nil, "a") => 97.1
```

r[class.comparators.descending.compare]
The `compare` of `DescendingComparator` takes two arguments, sends `compareTo`
to the first with the second as its only argument, sends the prefix `-` to the
answer, and answers what that answers: the negated number, so that the order
is the reverse of the one `Comparator` gives. An answer that is no number
raises the error of `-` for it, `SYNTAX 41.1` for a string, and one that does
not understand `-` raises `SYNTAX 97.1`. Argument errors are those of
[`Comparator`](r:class.comparators.comparator.compare).

```rexx
c = .descendingComparator~new
say c~compare('a', 'b') c~compare('b', 'a') c~compare('b', 'b')
say c~compare('10', '9') c~compare('A', 'a')
say c~compare(.pad~new(7), .pad~new(1)) c~compare(.pad~new(0), 1),
  c~compare(.pad~new(0.5), 1) c~compare(.pad~new(' 4 '), 1)
call try 'c~compare(.pad~new("x"), 1)'
call try 'c~compare(.pad~new(.nil), 1)'
call try 'c~compare("a")'
call try 'c~compare("a", "b", "c")'
call try 'c~compare(.nil, "a")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class pad
::method init
  expose v
  use arg v
::method compareTo
  expose v
  return v
```

```output
1 -1 0
1 1
-7 0 -0.5 -4
c~compare(.pad~new("x"), 1) => 41.1
c~compare(.pad~new(.nil), 1) => 97.1
c~compare("a") => 93.901
c~compare("a", "b", "c") => 93.902
c~compare(.nil, "a") => 97.1
```

r[class.comparators.caselessdescending.compare]
The `compare` of `CaselessDescendingComparator` is that of
[`DescendingComparator`](r:class.comparators.descending.compare) with
`caselessCompareTo` ([the string rule](r:class.string.caselesscompareto)) in
place of `compareTo`: it sends `caselessCompareTo` to the first argument with
the second as its only argument, then the prefix `-` to the answer, and
answers what that answers. Strings that differ in the case of letters alone
answer 0. A first argument without `caselessCompareTo` raises `SYNTAX 97.1`.

```rexx
c = .caselessDescendingComparator~new
say c~compare('a', 'B') c~compare('B', 'a') c~compare('a', 'A')
say c~compare('Rye', 'ryegrass') c~compare('', '')
call try 'c~compare("a")'
call try 'c~compare("a", "b", "c")'
call try 'c~compare(.pad~new, "a")'
call try 'c~compare(.nil, "a")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class pad
```

```output
1 -1 0
1 0
c~compare("a") => 93.901
c~compare("a", "b", "c") => 93.902
c~compare(.pad~new, "a") => 97.1
c~compare(.nil, "a") => 97.1
```

## The column comparators

r[class.comparators.column.init]
`init` of `ColumnComparator` takes two arguments, `start` and `length`, and
keeps them as they are: it examines neither, so that `new` accepts any two
objects, including ones that no `compare` can use, and an invalid value shows
only when `compare` is sent ([the compare
rule](r:class.comparators.column.compare)). Both arguments are required: `new`
with none or one raises `SYNTAX 93.901`, one with only the second given raises
`SYNTAX 93.903`, and a third argument raises `SYNTAX 93.902`. `init` is an
ordinary message that can be sent again ([the init
rule](r:class.index.new.init-again)), and then replaces both values.

```rexx
call try '.columnComparator~new(3, 2)~class~id'
call try '.columnComparator~new("x", .nil)~class~id'
call try '.columnComparator~new(0, -1)~class~id'
call try '.columnComparator~new'
call try '.columnComparator~new(3)'
call try '.columnComparator~new(3, )'
call try '.columnComparator~new(, 2)'
call try '.columnComparator~new(3, 2, 1)'
c = .columnComparator~new(3, 2)
say c~compare('12ab', '99ac')
c~init(1, 1)
say c~compare('12ab', '99ac') c~compare('12ab', '19ac')
call try 'c~init(1)'
call try 'c~init(1, 2, 3)'
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
.columnComparator~new(3, 2)~class~id => ColumnComparator
.columnComparator~new("x", .nil)~class~id => ColumnComparator
.columnComparator~new(0, -1)~class~id => ColumnComparator
.columnComparator~new => 93.901
.columnComparator~new(3) => 93.901
.columnComparator~new(3, ) => 93.901
.columnComparator~new(, 2) => 93.903
.columnComparator~new(3, 2, 1) => 93.902
-1
-1 0
c~init(1) => 93.901
c~init(1, 2, 3) => 93.902
```

r[class.comparators.column.compare]
The `compare` of `ColumnComparator` takes two arguments and sends `compareTo` to
the first with three arguments, the second argument, `start` and `length`, and
answers what it answers. For two strings that is [the string
rule](r:class.string.compareto), with `start` and `length` checked as it checks
them when `compare` is sent; so `init` accepts values that the first `compare`
refuses. A first argument whose `compareTo` takes one argument only, as the
default one of [`Comparable`](r:class.comparable.compareto) does, raises
`SYNTAX 93.902`, and one with no `compareTo` raises `SYNTAX 97.1`. Other
argument errors are those of
[`Comparator`](r:class.comparators.comparator.compare).

```rexx
c = .columnComparator~new(3, 2)
say c~compare('12abc', '99abd') c~compare('12abd', '99abc')
say c~compare('12abc', '99aBc') c~compare('12', '99') c~compare('12', '99a')
say c~compare('12ab', '99a') c~compare('12 ', '12') c~compare('12 ', '99')
say .columnComparator~new(' 3 ', ' 2 ')~compare('12abc', '99abd')
say .columnComparator~new(1, 1)~compare('ab', 'ac')
say .columnComparator~new(2, 100)~compare('xab', 'yac')
say .columnComparator~new(1, 0)~compare('abc', 'xyz')
call try '.columnComparator~new(0, 2)~compare("ab", "ab")'
call try '.columnComparator~new(1.5, 2)~compare("ab", "ab")'
call try '.columnComparator~new("x", 2)~compare("ab", "ab")'
call try '.columnComparator~new(1, -1)~compare("ab", "ab")'
call try '.columnComparator~new(1, "x")~compare("ab", "ab")'
say c~compare(.pad~new, "ab")
call try 'c~compare(.comparable~new, "ab")'
call try 'c~compare(.object~new, "ab")'
call try 'c~compare("ab")'
call try 'c~compare("ab", "cd", "ef")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class pad
::method compareTo
  return 0
```

```output
0 0
1 0 -1
1 1 1
0
0
-1
0
.columnComparator~new(0, 2)~compare("ab", "ab") => 93.924
.columnComparator~new(1.5, 2)~compare("ab", "ab") => 93.924
.columnComparator~new("x", 2)~compare("ab", "ab") => 93.924
.columnComparator~new(1, -1)~compare("ab", "ab") => 93.923
.columnComparator~new(1, "x")~compare("ab", "ab") => 93.923
0
c~compare(.comparable~new, "ab") => 93.902
c~compare(.object~new, "ab") => 97.1
c~compare("ab") => 93.901
c~compare("ab", "cd", "ef") => 93.902
```

r[class.comparators.caselesscolumn.init]
`init` of `CaselessColumnComparator` is that of
[`ColumnComparator`](r:class.comparators.column.init): two required arguments
that are kept without being examined, `SYNTAX 93.901` when one or none is
given, `SYNTAX 93.903` when only the second is, and `SYNTAX 93.902` for a
third.

```rexx
call try '.caselessColumnComparator~new(3, 2)~class~id'
call try '.caselessColumnComparator~new("x", .nil)~class~id'
call try '.caselessColumnComparator~new'
call try '.caselessColumnComparator~new(3)'
call try '.caselessColumnComparator~new(, 2)'
call try '.caselessColumnComparator~new(3, 2, 1)'
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
.caselessColumnComparator~new(3, 2)~class~id => CaselessColumnComparator
.caselessColumnComparator~new("x", .nil)~class~id => CaselessColumnComparator
.caselessColumnComparator~new => 93.901
.caselessColumnComparator~new(3) => 93.901
.caselessColumnComparator~new(, 2) => 93.903
.caselessColumnComparator~new(3, 2, 1) => 93.902
```

r[class.comparators.caselesscolumn.compare]
The `compare` of `CaselessColumnComparator` is that of
[`ColumnComparator`](r:class.comparators.column.compare) with
`caselessCompareTo` in place of `compareTo`: it sends `caselessCompareTo` to
the first argument with three arguments, the second argument, `start` and
`length`, and answers what it answers ([the string
rule](r:class.string.caselesscompareto)), so that the substrings are compared
without regard to the case of letters. The errors for `start` and `length`,
for a first argument without `caselessCompareTo` (`SYNTAX 97.1`) and for the
number of arguments are those of the other rule.

```rexx
c = .caselessColumnComparator~new(3, 2)
say c~compare('12abc', '99ABD') c~compare('12ABd', '99abc')
say c~compare('12abc', '99ABC') c~compare('12', '99') c~compare('12', '99a')
say c~compare('12ab', '99A') c~compare('ab', 'AB')
say .caselessColumnComparator~new(1, 1)~compare('Ab', 'aC')
call try '.caselessColumnComparator~new(0, 2)~compare("ab", "ab")'
call try '.caselessColumnComparator~new(1, -1)~compare("ab", "ab")'
call try 'c~compare(.pad~new, "ab")'
call try 'c~compare(.nil, "ab")'
call try 'c~compare("ab")'
call try 'c~compare("ab", "cd", "ef")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class pad
```

```output
0 0
0 0 -1
1 0
0
.caselessColumnComparator~new(0, 2)~compare("ab", "ab") => 93.924
.caselessColumnComparator~new(1, -1)~compare("ab", "ab") => 93.923
c~compare(.pad~new, "ab") => 97.1
c~compare(.nil, "ab") => 97.1
c~compare("ab") => 93.901
c~compare("ab", "cd", "ef") => 93.902
```

## The inverting comparator

r[class.comparators.inverting.init]
`init` of `InvertingComparator` takes one argument, the comparator to invert,
and keeps it as it is, without examining it: any object, `.nil` and a string
included, is accepted by `new`, and a value that cannot be used shows only
when `compare` is sent ([the compare
rule](r:class.comparators.inverting.compare)). The argument is required: `new`
with none, or with an omitted one, raises `SYNTAX 93.901`, and a second
argument raises `SYNTAX 93.902`. A later `init` sent to the instance replaces
the comparator.

```rexx
say .invertingComparator~new(.comparator~new)~class~id
say .invertingComparator~new(.nil)~class~id,
  .invertingComparator~new(1)~class~id
call try '.invertingComparator~new'
call try '.invertingComparator~new()'
call try '.invertingComparator~new(.comparator~new, 2)'
i = .invertingComparator~new(.comparator~new)
say i~compare('a', 'B')
i~init(.caselessComparator~new)
say i~compare('a', 'B')
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
InvertingComparator
InvertingComparator InvertingComparator
.invertingComparator~new => 93.901
.invertingComparator~new() => 93.901
.invertingComparator~new(.comparator~new, 2) => 93.902
-1
1
```

r[class.comparators.inverting.compare]
The `compare` of `InvertingComparator` takes two arguments, sends `compare`
with both of them to the comparator that `init` was given, sends the prefix
`-` to the answer, and answers what that answers: the negated number, so that
the order is the reverse of the one the held comparator gives. Inverting an
inverting comparator gives the order of the comparator inside. An answer of
the held comparator that is no number raises the error of `-` for it,
`SYNTAX 41.1` for a string, and a held object that has no `compare`, `.nil`
included, raises `SYNTAX 97.1`. Argument errors are those of
[`Comparator`](r:class.comparators.comparator.compare).

```rexx
by_size = .sizecomparator~new
i = .invertingComparator~new(by_size)
say by_size~compare('aaa', 'b') i~compare('aaa', 'b')
say by_size~compare('b', 'c') i~compare('b', 'c')
j = .invertingComparator~new(i)
say j~compare('aaa', 'b') j~compare('b', 'aaa')
say .invertingComparator~new(.comparator~new)~compare('a', 'b'),
  .invertingComparator~new(.caselessComparator~new)~compare('a', 'A')
call try '.invertingComparator~new(.pad~new("x"))~compare(1, 2)'
call try '.invertingComparator~new(.pad~new(0.5))~compare(1, 2)'
call try '.invertingComparator~new(.pad~new(5))~compare(1, 2)'
call try '.invertingComparator~new(.nil)~compare(1, 2)'
call try '.invertingComparator~new(.object~new)~compare(1, 2)'
call try 'i~compare("a")'
call try 'i~compare("a", "b", "c")'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class sizecomparator subclass comparator
::method compare
  use arg a, b
  return (a~length - b~length)~sign
::class pad
::method init
  expose v
  use arg v
::method compare
  expose v
  return v
```

```output
1 -1
0 0
1 -1
1 0
.invertingComparator~new(.pad~new("x"))~compare(1, 2) => 41.1
.invertingComparator~new(.pad~new(0.5))~compare(1, 2) => -0.5
.invertingComparator~new(.pad~new(5))~compare(1, 2) => -5
.invertingComparator~new(.nil)~compare(1, 2) => 97.1
.invertingComparator~new(.object~new)~compare(1, 2) => 97.1
i~compare("a") => 93.901
i~compare("a", "b", "c") => 93.902
```

## The numeric comparator

r[class.comparators.numeric.init]
`init` of `NumericComparator` takes one optional argument, the number of
digits that `compare` works to: a whole number from 1 to 999999999, 9 when the
argument is omitted. A value that is not such a whole number raises
`SYNTAX 26.5` in `new`, as it does for a `NUMERIC DIGITS` instruction; a
number with blanks around it is accepted. A second argument raises
`SYNTAX 93.902`. A later `init` sent to the instance replaces the digits.

```rexx
say .numericComparator~new~class~id .numericComparator~new(5)~class~id
say .numericComparator~new(1)~class~id,
  .numericComparator~new(999999999)~class~id
say .numericComparator~new(' 5 ')~class~id .numericComparator~new()~class~id
call try '.numericComparator~new(0)'
call try '.numericComparator~new(-1)'
call try '.numericComparator~new(1.5)'
call try '.numericComparator~new("x")'
call try '.numericComparator~new(.nil)'
call try '.numericComparator~new(1000000000)'
call try '.numericComparator~new(1, 2)'
n = .numericComparator~new
say n~compare(1000, 1001)
n~init(2)
say n~compare(1000, 1001)
call try 'n~init(0)'
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
NumericComparator NumericComparator
NumericComparator NumericComparator
NumericComparator NumericComparator
.numericComparator~new(0) => 26.5
.numericComparator~new(-1) => 26.5
.numericComparator~new(1.5) => 26.5
.numericComparator~new("x") => 26.5
.numericComparator~new(.nil) => 26.5
.numericComparator~new(1000000000) => 26.5
.numericComparator~new(1, 2) => 93.902
-1
0
n~init(0) => 26.5
```

r[class.comparators.numeric.compare]
The `compare` of `NumericComparator` takes two arguments, subtracts the second
from the first by sending `-` to the first with the second as its only argument,
while `NUMERIC DIGITS` is the number given to `init` (the digits of the program
that sent `compare` do not matter), and answers the sign of the difference: -1,
0 or 1. Two numbers whose difference is lost in the rounding to those digits are
equal, so a comparator of 3 digits finds 1000 and 1001 equal and 1000 and 1010
not. A difference that is no number raises `SYNTAX 93.943`; a string that is no
number raises `SYNTAX 41.1`; a first argument that has no `-` method raises
`SYNTAX 97.1`. Argument errors are those of
[`Comparator`](r:class.comparators.comparator.compare). The answer is read as a
number as [the argument rule](r:class.index.string) reads one, so an object
whose `makeString` or `string` answers a number is accepted, and the sign is
that number's.

```rexx
n = .numericComparator~new
say n~compare(2, 10) n~compare(10, 2) n~compare('10', '10.0')
say n~compare('1e1', '10') n~compare(' 5 ', '5') n~compare('-1', 1)
say n~compare(1000, 1001) n~compare(123456789, 123456788)
d3 = .numericComparator~new(3)
say d3~compare(1000, 1001) d3~compare(1000, 1010) d3~compare(1000, 1020)
numeric digits 3
say n~compare(1000, 1001) .numericComparator~new(9)~compare(1000, 1001)
numeric digits 9
call try 'n~compare("a", "b")'
call try 'n~compare(1, "")'
call try 'n~compare(.nil, 1)'
call try 'n~compare(1, .nil)'
call try 'n~compare(.pad~new("x"), 1)'
call try 'n~compare(.pad~new(.nil), 1)'
call try 'n~compare(.pad~new(40), 1)'
say n~compare(.pad~new(.ms~new), 1) n~compare(.pad~new(.st~new), 1)
call try 'n~compare(1)'
call try 'n~compare(1, 2, 3)'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class pad
::method init
  expose v
  use arg v
::method '-'
  expose v
  return v
::class ms
::method makeString
  return '-7'
::class st
::method string
  return '-7'
```

```output
-1 1 0
0 0 -1
-1 1
0 -1 -1
-1 -1
n~compare("a", "b") => 41.1
n~compare(1, "") => 41.1
n~compare(.nil, 1) => 97.1
n~compare(1, .nil) => 41.1
n~compare(.pad~new("x"), 1) => 93.943
n~compare(.pad~new(.nil), 1) => 93.943
n~compare(.pad~new(40), 1) => 1
-1 -1
n~compare(1) => 93.901
n~compare(1, 2, 3) => 93.902
```

## Sorting with a comparator

r[class.comparators.sort]
The comparators are what `sortWith` and `stableSortWith` take ([the sort
rule](r:class.array.sort)), and the order they give is the one that their
`compare` rules describe: `Comparator` the order of `compareTo`, in which for
strings upper case comes before lower case and `10` before `9`;
`CaselessComparator` the same without regard to case; `DescendingComparator`
and `CaselessDescendingComparator` the reverse of those; the two column
comparators the order of the part of each string that starts at `start`, with
items whose parts are equal keeping their order under `stableSortWith`;
`InvertingComparator` the reverse of the comparator it holds; and
`NumericComparator` the order of the numbers the items stand for. The example
sorts the same items with each, by `stableSortWith`, so that items that
compare as equal keep their places; `2e1` and `20.0` both stand for 20, and
keep their order for `NumericComparator`.

```rexx
w = .array~of('Rowan', 'elder', "hawk's eye", 'Birch', 'birch')
say show(w~copy~~stableSortWith(.comparator~new))
say show(w~copy~~stableSortWith(.caselessComparator~new))
say show(w~copy~~stableSortWith(.descendingComparator~new))
say show(w~copy~~stableSortWith(.caselessDescendingComparator~new))
say show(w~copy~~stableSortWith(.columnComparator~new(2, 3)))
say show(w~copy~~stableSortWith(.caselessColumnComparator~new(2, 3)))
say show(w~copy~~stableSortWith(.invertingComparator~new(.comparator~new)))
col = .columnComparator~new(2, 3)
say show(w~copy~~stableSortWith(.invertingComparator~new(col)))
n = .array~of(40, 8, 31, 6, 15, '2e1', 20.0)
say show(n~copy~~stableSortWith(.numericComparator~new))
say show(n~copy~~stableSortWith(.comparator~new))
exit
show: procedure
  s = ''
  do i over arg(1)
    s = s i
  end
  return s~strip
```

```output
Birch Rowan birch elder hawk's eye
Birch birch elder hawk's eye Rowan
hawk's eye elder birch Rowan Birch
Rowan hawk's eye elder Birch birch
hawk's eye Birch birch elder Rowan
hawk's eye Birch birch elder Rowan
hawk's eye elder birch Rowan Birch
Rowan elder Birch birch hawk's eye
6 8 15 2e1 20.0 31 40
15 20.0 2e1 31 40 6 8
```

## Messages that a program can override

r[class.comparators.sends]
Each `compare` sends the messages below, in this order, and no other: nothing is
sent to the second argument or to the comparators themselves, nothing is sent to
the first argument but what is listed, and no argument is asked `isA`.
`Comparator` sends `compareTo` to the first argument with one argument, the
second. `CaselessComparator` sends `caselessCompareTo` the same way.
`ColumnComparator` sends `compareTo`, and `CaselessColumnComparator`
`caselessCompareTo`, with three arguments: the second argument, `start` and
`length`. `DescendingComparator` sends `compareTo` with one argument and then
the prefix `-`, which is the message `-` with no argument, to the answer, and
`CaselessDescendingComparator` does the same with `caselessCompareTo`.
`InvertingComparator` sends `compare` with two arguments, the arguments it got,
to the comparator it holds and then the prefix `-` to the answer.
`NumericComparator` sends `-` to the first argument with one argument, the
second. The answer of the last message of each is the answer of `compare`,
except that `NumericComparator` answers the sign of it, reading it as a number
as [the argument rule](r:class.index.string) says. The example gives the objects
methods that log the messages they get, `isA` among them, and prints what each
`compare` sent.

```rexx
call p 'comparator', '.comparator~new~compare(.first~new, .second~new)'
call p 'caseless', '.caselessComparator~new~compare(.first~new, .second~new)'
call p 'column',,
  '.columnComparator~new(2, 3)~compare(.first~new, .second~new)'
call p 'caselesscolumn',,
  '.caselessColumnComparator~new(2, 3)~compare(.first~new, .second~new)'
call p 'descending',,
  '.descendingComparator~new~compare(.first~new, .second~new)'
call p 'caselessdescending',,
  '.caselessDescendingComparator~new~compare(.first~new, .second~new)'
call p 'inverting',,
  '.invertingComparator~new(.inner~new)~compare(.first~new, .second~new)'
call p 'numeric', '.numericComparator~new~compare(.first~new, .second~new)'
exit
p:
  .local~log = ''
  signal on syntax name bad
  interpret 'x =' arg(2)
  say arg(1)':' .local~log '=>' x~class~id
  return
bad: say arg(1)':' .local~log '=>' condition('o')~code
  return
::class first
::method isA
  .local~log = .local~log 'first:isA'
  forward class (super)
::method compareTo
  .local~log = .local~log 'compareTo/'arg()
  return .answer~new
::method caselessCompareTo
  .local~log = .local~log 'caselessCompareTo/'arg()
  return .answer~new
::method '-'
  .local~log = .local~log '-/'arg()
  return .answer~new
::class second
::method isA
  .local~log = .local~log 'second:isA'
  forward class (super)
::method unknown
  .local~log = .local~log 'SECOND!'
  return 1
::class inner
::method isA
  .local~log = .local~log 'inner:isA'
  forward class (super)
::method compare
  .local~log = .local~log 'compare/'arg()
  return .answer~new
::class answer
::method isA
  .local~log = .local~log 'answer:isA'
  forward class (super)
::method '-'
  .local~log = .local~log 'answer-/'arg()
  return 9
```

```output
comparator:  compareTo/1 => ANSWER
caseless:  caselessCompareTo/1 => ANSWER
column:  compareTo/3 => ANSWER
caselesscolumn:  caselessCompareTo/3 => ANSWER
descending:  compareTo/1 answer-/0 => String
caselessdescending:  caselessCompareTo/1 answer-/0 => String
inverting:  compare/2 answer-/0 => String
numeric:  -/1 => 93.943
```

## Deviations

r[class.comparators.deviation.109]
A string that a comparator reads from an argument is read by the string method
that `compare` sends, and an instance of a subclass of `String` is read by its
own characters, and `REQUEST`, `MAKESTRING` and `STRING` are not sent to it
([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an instance
`request` with `STRING` and then `makeString`, so an override can change what
is read. In the example `makeString` and `request` of the subclass answer
another string, and the instances still stand for their own characters.

```rexx
.local~sent = 0
x = .digits~new('b')
say .comparator~new~compare('a', x) .comparator~new~compare('c', x)
say .caselessComparator~new~compare('B', x)
say .columnComparator~new(1, 1)~compare('ab', .digits~new('ba'))
say 'sends' .local~sent
::class digits subclass string
::method makestring
  .local~sent = .local~sent + 1
  return 'zzz'
::method request
  .local~sent = .local~sent + 1
  return 'zzz'
```

```output defect=#140
-1 1
0
-1
sends 0
```
