---
element: "Orderable"
kind: class
rexxref: "Builtin Classes > Utility Classes > Orderable Class; Builtin Classes > Utility Classes > Orderable Class > compareTo; Builtin Classes > Utility Classes > Orderable Class > Comparison Methods"
---

# Orderable

`Orderable` is a mixin class that gives a class the eighteen comparison
operators as methods, all of them answered from one method, `compareTo`. A class
that inherits it and overrides `compareTo` with its own ordering can then be
written on both sides of `<`, `=`, `>>=` and the rest, and sorted by
[`sort`](r:class.array.sort). `DateTime`, `TimeSpan` and `File` are such
classes; the [comparison rule of `DateTime`](r:class.datetime.compare) and [the
one of `TimeSpan`](r:class.timespan.compare) say what their `compareTo` answers.
The page of [`Comparable`](r:class.comparable.creation) describes the class that
holds the `compareTo` alone.

## The class

r[class.orderable.creation]
`Orderable` is a mixin class ([the mixin rule](r:class.index.mixins)) whose
only superclass is `Object`. It defines the instance method `compareTo` and
the eighteen comparison methods `=`, `==`, `<`, `<=`, `<<`, `<<=`, `<>`, `>`,
`>=`, `><`, `>>`, `>>=`, `\=`, `\==`, `\<`, `\<<`, `\>` and `\>>`; it has no
other method of its own, and no class method besides the `new` that every
class has. It does not inherit [`Comparable`](r:class.comparable.creation): an
instance of `Orderable` is not an instance of `Comparable`, and a class that
wants both inherits both. `new` takes no argument, and any argument raises
`SYNTAX 93.902` ([the new rule](r:class.index.new)). An instance that is a
plain `Orderable` can be made: its `compareTo` is the default one of [the next
rule](r:class.orderable.compareto).

```rexx
do c over .orderable~superclasses
  say c~id
end
names = 'compareTo = == < <= << <<= <> > >= >< >> >>= \= \== \< \<< \> \>>'
found = 0
do n over names~makearray(' ')
  found += (.orderable~method(n)~scope == .orderable)
end
say found 'of' names~words
say .orderable~new~class~id .orderable~new~isA(.comparable)
say .timespan~new(1)~isA(.orderable) .datetime~new(1)~isA(.orderable)
signal on syntax name bad
say .orderable~new(1)
bad: say condition('o')~code
```

```output
Object
19 of 19
Orderable 0
1 1
93.902
```

r[class.orderable.compareto]
`compareTo` takes one argument and answers what [`compareTo` of
`Comparable`](r:class.comparable.compareto) answers, from the `identityHash`
answers of the receiver and the argument, with the same errors.

```rexx
a = .item~new(3)
b = .item~new(9)
c = .item~new(3)
say a~compareTo(b) b~compareTo(a) a~compareTo(c) a~compareTo(a)
say .item~new(10)~compareTo(.item~new(9)) .item~new(2.50)~compareTo(c)
x = .orderable~new
say x~compareTo(x)
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
::class item subclass orderable
::method init
  expose h
  use arg h
::method identityHash
  expose h
  return h
```

```output
-1 1 0 0
1 -1
0
a~compareTo => 93.901
a~compareTo(b, c) => 93.902
.item~new("abc")~compareTo(b) => 41.1
a~compareTo(.item~new("")) => 41.1
```

## The comparison operators

r[class.orderable.compare]
Each of the eighteen comparison methods takes one argument, sends `compareTo`
to the receiver with that argument, and answers what the answer of `compareTo`
answers to a comparison message that has the number 0 as its argument. That
message is `=` for `=` and `==`, `<` for `<` and `<<`, `>` for `>` and `>>`,
`<=` for `<=` and `<<=`, `>=` for `>=` and `>>=`, and `\=` for `\=` and `\==`.
So a strict operator is not strict here: the answer of `compareTo` is compared
as the non-strict operator compares, and an answer of ` 1 ` (with blanks) is
greater than 0 for `>>` as for `>`, one of `00` is equal to 0 for `==`. An
answer that is no number is compared as a string with `0`, so that `abc` is
greater and the empty string is less. What a method answers is whatever the
sent message answers; an answer of `compareTo` that does not understand the
message raises `SYNTAX 97.1`, as `.nil` does for every message here but `=`
and `\=`. Four methods hand the work to another method of the receiver: `\<`,
`\>`, `\<<` and `\>>` send `>=`, `<=`, `>>=` and `<<=` with their argument and
answer what it answers, and `<>` and `><` send `\=` the same way. So a class
that overrides `>=` changes `\<` as well. The example prints, for answers of
`compareTo` of -1, 0, 5, `abc`, the empty string, ` 1 `, `00` and `.nil`, what
the eighteen methods answer, in the order of the list in [the creation
rule](r:class.orderable.creation) with `compareTo` left out.

```rexx
call show '-1', '-1'
call show '0', '0'
call show '5', '5'
call show 'abc', 'abc'
call show 'empty', ''
call show 'blank 1', ' 1 '
call show '00', '00'
call show '.nil', .nil
exit
show:
  .fixed~answer = arg(2)
  line = arg(1)':'
  ops = '= == < <= << <<= <> > >= >< >> >>= \= \== \< \<< \> \>>'
  do op over ops~makearray(' ')
    line = line try(op)
  end
  say line
  return
try:
  signal on syntax name bad
  interpret 'r = .fixed~new' arg(1) '7'
  return r
bad: return condition('o')~code
::class fixed subclass object inherit orderable
::attribute answer class
::method compareTo
  return .fixed~answer
```

```output
-1: 0 0 1 1 1 1 1 0 0 1 0 0 1 1 0 0 1 1
0: 1 1 0 1 0 1 0 0 1 0 0 1 0 0 1 1 1 1
5: 0 0 0 0 0 0 1 1 1 1 1 1 1 1 1 1 0 0
abc: 0 0 0 0 0 0 1 1 1 1 1 1 1 1 1 1 0 0
empty: 0 0 1 1 1 1 1 0 0 1 0 0 1 1 0 0 1 1
blank 1: 0 0 0 0 0 0 1 1 1 1 1 1 1 1 1 1 0 0
00: 1 1 0 1 0 1 0 0 1 0 0 1 0 0 1 1 1 1
.nil: 0 0 97.1 97.1 97.1 97.1 1 97.1 97.1 1 97.1 97.1 1 1 97.1 97.1 97.1 97.1
```

r[class.orderable.compare.nil]
When the argument of `=`, `==`, `\=`, `\==`, `<>` or `><` is `.nil`, the
method answers without sending `compareTo`: 0 for `=` and `==`, and 1 for the
others (`<>` and `><` send `\=` to the receiver, which answers 1). Each of the
other twelve methods raises `SYNTAX 93.973` when its argument is `.nil`, and
sends `compareTo` to no one: `<`, `<=`, `<<`, `<<=`, `>`, `>=`, `>>` and `>>=`
raise it themselves, and `\<`, `\>`, `\<<` and `\>>` raise it by sending `>=`,
`<=`, `>>=` and `<<=` to the receiver, as [the comparison
rule](r:class.orderable.compare) says. The example prints each method, the
messages that the receiver got and the answer or the error code; the class of
the example logs `compareTo`, `>=`, `<=`, `>>=`, `<<=` and `\=`, so a method
of one of these names logs its own call too.

```rexx
ops = '= == < <= << <<= <> > >= >< >> >>= \= \== \< \<< \> \>>'
do op over ops~makearray(' ')
  call try op
end
exit
try:
  .local~log = ''
  signal on syntax name bad
  interpret 'r = .fixed~new' arg(1) '.nil'
  say arg(1)':' r'  sent:' .local~log
  return
bad: say arg(1)':' condition('o')~code'  sent:' .local~log
  return
::class fixed subclass object inherit orderable
::method compareTo
  .local~log = .local~log 'compareTo'
  return 0
::method '>='
  .local~log = .local~log '>='
  forward class (super)
::method '<='
  .local~log = .local~log '<='
  forward class (super)
::method '>>='
  .local~log = .local~log '>>='
  forward class (super)
::method '<<='
  .local~log = .local~log '<<='
  forward class (super)
::method '\='
  .local~log = .local~log '\='
  forward class (super)
```

```output
=: 0  sent: 
==: 0  sent: 
<: 93.973  sent: 
<=: 93.973  sent:  <=
<<: 93.973  sent: 
<<=: 93.973  sent:  <<=
<>: 1  sent:  \=
>: 93.973  sent: 
>=: 93.973  sent:  >=
><: 1  sent:  \=
>>: 93.973  sent: 
>>=: 93.973  sent:  >>=
\=: 1  sent:  \=
\==: 1  sent: 
\<: 93.973  sent:  >=
\<<: 93.973  sent:  >>=
\>: 93.973  sent:  <=
\>>: 93.973  sent:  <<=
```

r[class.orderable.compare.args]
Each of the comparison methods takes exactly one argument. A missing argument
and one that is omitted raise `SYNTAX 93.901`, and a second argument raises
`SYNTAX 93.902`, before `compareTo` is sent ([the count
rule](r:class.index.args.count)). Any object but `.nil` is accepted as the
argument, and is not examined by the method: it is only passed on to
`compareTo`.

```rexx
.local~log = ''
call try '.fixed~new~"<"'
call try '.fixed~new~"<"()'
call try '.fixed~new~"<"(1, 2)'
call try '.fixed~new~"="'
call try '.fixed~new~"\=="(1, 2)'
call try '.fixed~new~"<"(.object~new)'
call try '.fixed~new~">="("abc")'
say 'sent:' .local~log
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class fixed subclass object inherit orderable
::method compareTo
  .local~log = .local~log 'compareTo/'arg()
  return 1
```

```output
.fixed~new~"<" => 93.901
.fixed~new~"<"() => 93.901
.fixed~new~"<"(1, 2) => 93.902
.fixed~new~"=" => 93.901
.fixed~new~"\=="(1, 2) => 93.902
.fixed~new~"<"(.object~new) => 0
.fixed~new~">="("abc") => 1
sent:  compareTo/1 compareTo/1
```

r[class.orderable.compare.sends]
A comparison method sends the messages in this order and no other: the
receiver is sent first the method that the table of [the comparison
rule](r:class.orderable.compare) names for `<>`, `><`, `\<`, `\>`, `\<<` and
`\>>` (`\=`, `\=`, `>=`, `<=`, `>>=`, `<<=`), which then goes on as a method
of the other kind does; then `compareTo` is sent to the receiver with the
argument itself and no other argument; then the comparison message is sent to
the answer of `compareTo`, with the number 0 as its single argument. Nothing
is sent to the argument of the method (`isA`, `makeString`, `request` and
`identityHash` included), and `\=` and `\==` send `\=`, not `=`, to the
answer. The example gives the answer of `compareTo` a method for every
comparison message, and the receiver's class a method for every comparison
message too, that log themselves; so each line starts with the method itself,
then has what it sent, `compareTo` with its argument count and the class of
its argument, and the message that the answer got with its argument.

```rexx
ops = '= == < <= << <<= <> > >= >< >> >>= \= \== \< \<< \> \>>'
do op over ops~makearray(' ')
  body = .array~of(".local~log = .local~log '" || op || "/'arg(1)",,
    'return 1')
  .rec~define(op, .method~new(op, body))
  body = .array~of(".local~log = .local~log '" || op || " '",,
    'forward class (super)')
  .spy~define(op, .method~new(op, body))
end
do op over ops~makearray(' ')
  call try op
end
exit
try:
  .local~log = ''
  interpret 'r = .spy~new' arg(1) '.probe~new'
  say arg(1)':' .local~log
  return
::class spy subclass object inherit orderable
::method compareTo
  .local~log = .local~log 'compareTo/'arg()'/'arg(1)~class~id' '
  return .rec~new
::class rec
::class probe
::method unknown
  .local~log = .local~log 'PROBE!'
  return 1
```

```output
=:  =  compareTo/1/PROBE  =/0
==:  ==  compareTo/1/PROBE  =/0
<:  <  compareTo/1/PROBE  </0
<=:  <=  compareTo/1/PROBE  <=/0
<<:  <<  compareTo/1/PROBE  </0
<<=:  <<=  compareTo/1/PROBE  <=/0
<>:  <>  \=  compareTo/1/PROBE  \=/0
>:  >  compareTo/1/PROBE  >/0
>=:  >=  compareTo/1/PROBE  >=/0
><:  ><  \=  compareTo/1/PROBE  \=/0
>>:  >>  compareTo/1/PROBE  >/0
>>=:  >>=  compareTo/1/PROBE  >=/0
\=:  \=  compareTo/1/PROBE  \=/0
\==:  \==  compareTo/1/PROBE  \=/0
\<:  \<  >=  compareTo/1/PROBE  >=/0
\<<:  \<<  >>=  compareTo/1/PROBE  >=/0
\>:  \>  <=  compareTo/1/PROBE  <=/0
\>>:  \>>  <<=  compareTo/1/PROBE  <=/0
```
