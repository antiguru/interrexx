---
element: DO
kind: instruction
syntax: "DO [LABEL name] [COUNTER name] [repetitor] [conditional]; instructions; END [name]"
ansi: "6.3.2.12, 8.3.6"
rexxref: "Keyword Instructions > DO"
deviations: [1, 25, 26, 38, 47, 118]
---

# DO

DO opens a block of instructions that ends at a matching END. Without
anything after the keyword it groups the clauses so that they stand where
the language expects one instruction. With a repetitor, a condition, or both,
it runs the block again and again: a number of times, for ever, with a
variable that steps through a range, or once for each item of a collection.
This page specifies each form, the order in which the parts of the clause
are evaluated, the tests that end a loop, the value that the control
variable has afterwards, and the errors. [LOOP](loop.md) is the same
instruction under another keyword, and [LEAVE](leave.md) and
[ITERATE](iterate.md) steer a loop from inside.

The clause has one of these shapes, where each part in brackets is optional.
The LABEL and COUNTER phrases come first, in either order. Then comes at
most one repetitor, which is a count, `FOREVER`, `name = expression` with
TO, BY and FOR phrases, `name OVER expression`, or `WITH` with an INDEX
and an ITEM phrase and `OVER expression`. Then comes at most one WHILE or
UNTIL phrase. The block follows, and `END` closes it.

## Groups

r[instr.do.group]
A DO with nothing after the keyword, or with only a LABEL phrase, is a simple
group. The interpreter runs the clauses between the DO and its END once, in
order, and no variable changes because of the DO. A group lets several clauses
stand in the place of one, as the branch of an IF does. It is not a loop, so a
LEAVE or an ITERATE with no name does not see it, as [the LEAVE
rule](r:instr.leave.target) says, while a LEAVE that names its label does
leave it.

```rexx
x = 1
if x = 1 then do
  say 'one'
  say 'still the first branch'
end
else say 'two'
do; say 'bare'; end
do label blk
  say 'labelled'
  leave blk
  say 'skipped'
end blk
say 'done'
```

```output
one
still the first branch
bare
labelled
done
```

## Counts and endless loops

r[instr.do.count]
`DO expression` runs the block a fixed number of times. The expression is
evaluated once, when the DO is reached, and its value is the number of passes,
so a later assignment to a variable of the expression changes nothing. The
value must be a whole number that is not negative. It may be written with a
decimal point or an exponent when that is its value, and it must be below 10
to the power of NUMERIC DIGITS however it is written: at DIGITS 5, `9.9999e4`
is a count and `1e5` is not. No count may reach 10 to the power of 18,
whatever DIGITS is. Any other value raises `SYNTAX 26.2`. A count of zero runs
no pass and does not raise an error. When
[LOSTDIGITS](r:cond.lostdigits.information) is trapped, a count written with
more than DIGITS digits raises that condition before the error, if any, as
[the lost-digits rule](r:num.arithmetic.lostdigits) describes for an operand.

```rexx
n = 3
do n
  n = 1
  say 'pass'
end
do 0
  say 'never'
end
k = 0
do 1e1; k = k + 1; end
say k n
call try 'do 1.5; end'
call try 'do -1; end'
call try 'do "x"; end'
call try 'do ""; end'
call try 'numeric digits 5; do 12345; leave; end'
call try 'numeric digits 5; do 123456; leave; end'
call try 'numeric digits 5; do 9.9999e4; leave; end'
call try 'numeric digits 5; do 1e5; leave; end'
call try 'numeric digits 20; do 999999999999999999; leave; end'
call try 'numeric digits 20; do 1000000000000000000; leave; end'
signal on lostdigits name wide
numeric digits 5
do 123456
  say 'never'
end
wide: say condition('c') condition('d')
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
pass
pass
pass
10 1
do 1.5; end => 26.2
do -1; end => 26.2
do "x"; end => 26.2
do ""; end => 26.2
numeric digits 5; do 12345; leave; end => ok
numeric digits 5; do 123456; leave; end => 26.2
numeric digits 5; do 9.9999e4; leave; end => ok
numeric digits 5; do 1e5; leave; end => 26.2
numeric digits 20; do 999999999999999999; leave; end => ok
numeric digits 20; do 1000000000000000000; leave; end => 26.2
LOSTDIGITS 123456
```

r[instr.do.forever]
`DO FOREVER` has no end test of its own, so the block runs until a LEAVE, an
EXIT, a RETURN, a SIGNAL or an error ends it. It may be followed by a WHILE
or an UNTIL phrase, which [the condition rules](r:instr.do.while) cover.
Anything else after FOREVER raises `SYNTAX 27.901`, because the clause is
read before it runs. [LOOP](loop.md) with nothing after it is the same
endless loop.

```rexx
n = 0
do forever
  n = n + 1
  if n = 4 then leave
end
say n
do forever while n < 6
  n = n + 1
end
say n
call try 'do forever forever; end'
call try 'do forever 2; end'
call try 'do forever to 2; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
4
6
do forever forever; end => 27.901
do forever 2; end => 27.901
do forever to 2; end => 27.901
```

## Controlled loops

r[instr.do.control]
`DO name = initial [TO limit] [BY step] [FOR count]` is a controlled loop. The
three phrases come in any order after the initial expression, each at most
once, and a repeated phrase raises `SYNTAX 27.902`. The interpreter evaluates
the initial expression and then the phrases in the order written, each exactly
once, and only after all of them does it assign the initial value to the
control variable. A phrase therefore sees the variables as they were before
the loop, even `name` itself, and nothing the block does to a variable of a
phrase affects the loop. A missing STEP is 1. A missing LIMIT means that no
limit ends the loop, and a missing COUNT means that no count does. A control
variable written as a string is not specified.

```rexx
do i = f('init', 1) to f('to', 3) by f('by', 1) for f('for', 5)
  say 'body' i
end
say '--'
do i = f('init', 1) for f('for', 2) to f('to', 9) by f('by', 1)
  say 'body' i
end
say '--'
i = 3
do i = 1 to i + 2 by i - 1
  say 'i' i
end
say '--'
t = 3
do j = 1 to t
  t = 10
  say 'j' j 't' t
end
call try 'do i = 1 to 3 by 1 to 5; end'
call try 'do i = 1 for 2 for 3; end'
exit
f: say 'eval' arg(1); return arg(2)
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
eval init
eval to
eval by
eval for
body 1
body 2
body 3
--
eval init
eval for
eval to
eval by
body 1
body 2
--
i 1
i 3
i 5
--
j 1 t 10
j 2 t 10
j 3 t 10
do i = 1 to 3 by 1 to 5; end => 27.902
do i = 1 for 2 for 3; end => 27.902
```

r[instr.do.control.check]
The interpreter checks each value when it has evaluated that expression, and
before it evaluates the next, so the first fault in the written order is the
one that is reported. The initial value, the LIMIT and the STEP must be
[numbers](r:num.index.form), and the first that is not raises `SYNTAX 41.1`.
The COUNT must be a value that [the count rule](r:instr.do.count) accepts, and
raises `SYNTAX 26.3` when it is not. The initial value is taken as the prefix
`+` would answer it, so it is [rounded and written](r:num.arithmetic.prefix)
like the result of an operation: blanks go, an exponent is written out as a
result would show it, and the digits are cut to NUMERIC DIGITS, but a trailing
zero after the point stays.

```rexx
call try 'do i = "a" to "b"; end'
call try 'do i = 1 to "b" by 1/0; end'
call try 'do i = 1 by 1/0 to "b"; end'
call try 'do i = 1 to 2 by "c"; end'
call try 'do i = 1 for 1.5; end'
call try 'do i = 1 for -1 to "b"; end'
call try 'do i = 1 for "d" by "c"; end'
do i = ' 5 ' to 5; say '['i']'; end
do i = 1.00 to 1; say i; end
do i = '1e1' to 10; say i; end
numeric digits 3
do i = 1234567 for 1; say i; end
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
do i = "a" to "b"; end => 41.1
do i = 1 to "b" by 1/0; end => 41.1
do i = 1 by 1/0 to "b"; end => 42.3
do i = 1 to 2 by "c"; end => 41.1
do i = 1 for 1.5; end => 26.3
do i = 1 for -1 to "b"; end => 26.3
do i = 1 for "d" by "c"; end => 26.3
[5]
1.00
10
1.23E+6
```

r[instr.do.control.step]
After each pass the interpreter adds STEP to the control variable, with [the
rules of addition](r:num.arithmetic.addsub) at the current NUMERIC settings,
and then decides whether to run another pass. No pass runs when FOR has
counted out. No pass runs either when the control variable has passed LIMIT,
which for a STEP that is zero or positive means that it is greater, and for a
negative STEP that it is smaller. The comparison is [a numeric
one](r:num.settings.effect), so NUMERIC FUZZ applies to it. A STEP of zero is
treated as a positive one, so it never moves the variable and ends the loop
only through FOR, a condition, a LEAVE, or an initial value beyond LIMIT. The
test happens before the first pass too, so a loop can run no pass at all.

```rexx
do i = 1 to 2 by 0.5; say i; end
do i = 5 to 1 by -2; say i; end
do i = 3 to 1; say 'never' i; end
do i = 1 to 3 by -1; say 'never' i; end
do i = 3 to 1 by 0; say 'never' i; end
do i = 1 to 1 by 0 for 3; say 'zero step' i; end
do i = 1 to 3 for 2; say 'for' i; end
numeric digits 3
do i = 998 for 4; say 'digits' i; end
numeric digits 4
numeric fuzz 1
do i = 3 to 2.999; say 'fuzz' i; end
```

```output
1
1.5
2.0
5
3
1
zero step 1
zero step 1
zero step 1
for 1
for 2
digits 998
digits 999
digits 1.00E+3
digits 1.00E+3
fuzz 3
```

r[instr.do.control.after]
When the loop ends, the control variable holds the value that it last had. If
a pass ran to its end and the step and the tests ended the loop, that is the
value after the last step, which is one step beyond the last value that the
block saw. If a LEAVE ended the loop, it is the value the block saw. If the
tests ended the loop before the first pass, it is the initial value. A
WHILE that fails ends the loop with the stepped value, and an UNTIL that
holds ends it before the step, so it keeps the value of the last pass.

```rexx
do i = 1 to 3; end; say i
do i = 1 to 3; leave; end; say i
do i = 5 to 1; end; say i
do i = 1 for 2; end; say i
do i = 1 to 3 while i < 2; end; say i
do i = 1 to 3 until i = 2; end; say i
do i = 1 to 3 until 1; end; say i
```

```output
4
1
5
3
2
2
1
```

r[instr.do.control.body]
The control variable is an ordinary variable that the block can read and
assign. The step adds to the value that the variable has then, so an
assignment in the block moves the loop, and a value that is not a number, or a
variable that was dropped, raises `SYNTAX 41.1` at the step. When the control
variable is a compound symbol, its tail is worked out again each time that the
loop uses the variable, from the values of the tail variables at that moment,
so the block can change which variable the loop steps.

```rexx
do i = 1 to 5
  say i
  if i = 2 then i = 4
end
say 'end' i
k = 1
do s.k = 1 to 3
  say 'k' k 's.1' s.1
  if k = 1 then do; s.2 = 10; k = 2; end
end
say 'after' k s.1 s.2
signal on syntax name e1
do i = 1 to 3
  i = 'abc'
end
e1: say condition('o')~code i
signal on syntax name e2
do i = 1 to 3
  drop i
end
e2: say condition('o')~code symbol('i')
```

```output
1
2
5
end 6
k 1 s.1 1
after 2 1 11
41.1 abc
41.1 LIT
```

## Collections

r[instr.do.over]
`DO name OVER expression` runs the block once for each item that a collection
yields, and assigns the item to `name` before the pass. The expression is
evaluated once, before `name` is assigned, and the loop sends it the message
`MAKEARRAY`, whatever the class of the object is: an instance of a subclass of
`List` is sent it as any other collection is ([Deviation
118](r:instr.do.deviation.118)). The items are those of the Array that comes
back, in its order, and they are taken when the loop starts, so a change to the
collection during the loop does not add, remove or alter an item that the loop
visits. An Array yields its items in index order and skips an index that holds
none, and a List and a Queue yield theirs from first to last. A String yields
its lines, so a null string yields none. A Stem, a Directory and a Table yield
their indexes, which for a Stem are the tails of the existing compound
variables, and a Set yields its members. The order of those four is not
specified, as [Deviation 1](r:instr.do.deviation.1) says for a Stem and
[Deviation 33](../deviations.md#33-hash-iteration-order) for the others. A
collection that yields nothing leaves `name` as it was, and otherwise `name`
holds the last item after the loop.

```rexx
a = .array~of('x', 'y', 'z')
do e over a
  say e
end
say 'e' e
do e over .list~of(1, 2)
  say 'list' e
end
do e over .queue~of('p', 'q')
  say 'queue' e
end
do e over .array~of('a', , 'c')
  say 'item' e
end
do e over 'l1' || '0a'x || 'l2'
  say '['e']'
end
do e over ''
  say 'never'
end
say 'e' e
b = .array~of(1, 2, 3)
do e over b
  say 'visit' e
  if e = 1 then b~append(4)
end
say b~items
s. = 'dflt'
s.beta = 2; s.alpha = 1; s.gamma = 3
n = 0; seen = .array~new
do t over s.
  n = n + s.t
  seen~append(t)
end
say n seen~sort~makeString('l', ' ')
d = .directory~new
d~put('v1', 'k1'); d~put('v2', 'k2')
seen = .array~new
do t over d
  seen~append(t)
end
say seen~sort~makeString('l', ' ')
t = .table~new
t['x'] = 1; t['y'] = 2
seen = .array~new
do e over t
  seen~append(e)
end
say seen~sort~makeString('l', ' ')
seen = .array~new
do e over .set~of('m', 'n')
  seen~append(e)
end
say seen~sort~makeString('l', ' ')
```

```output
x
y
z
e z
list 1
list 2
queue p
queue q
item a
item c
[l1]
[l2]
e l2
visit 1
visit 2
visit 3
4
6 ALPHA BETA GAMMA
k1 k2
x y
m n
```

r[instr.do.over.error]
The expression must answer a message `MAKEARRAY` with an Array. An object that
has no such method, or whose method answers something else, raises
`SYNTAX 98.913`. A FOR phrase may follow the expression, and its count is
checked as for a count loop, with `SYNTAX 26.3`. A TO or a BY phrase after the
FOR raises `SYNTAX 25.927`, as [Deviation 38](r:instr.do.deviation.38) says.

```rexx
a = .array~of(1, 2, 3)
do e over a for 2
  say 'for' e
end
call try 'do e over .object~new; end'
call try 'do e over .nilmaker~new; end'
call try 'do e over a for 1.5; end'
call try 'do e over a for -1; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
::class nilmaker
::method makearray
  return .nil
```

```output
for 1
for 2
do e over .object~new; end => 98.913
do e over .nilmaker~new; end => 98.913
do e over a for 1.5; end => 26.3
do e over a for -1; end => 26.3
```

r[instr.do.with]
`DO WITH INDEX name ITEM name OVER expression` runs the block once for each
entry of a collection, and assigns the index and the item of the entry. The
two phrases come in either order, and one or both may be left out. With
neither, the loop is an OVER loop that assigns nothing: it behaves as [the
OVER rules](r:instr.do.over) say, and the sentences below about `SUPPLIER` do
not apply to it. With INDEX or ITEM, the loop sends the value of the
expression the message `SUPPLIER` and walks the Supplier that comes back,
assigning the index first and the item after it, so that a variable named by
both ends up holding the item. The entries are taken when the loop starts. A
FOR phrase limits the number of entries as it does in an OVER loop. A phrase
repeated raises `SYNTAX 27.902`. WITH starts this form only when INDEX, ITEM
or OVER follows it, and such a clause with no OVER raises `SYNTAX 27.904`. An
object that does not answer `SUPPLIER` raises `SYNTAX 97.1`, and an answer
that is not a Supplier raises `SYNTAX 98.994`.

```rexx
a = .array~of('x', 'y', 'z')
do with index ix item it over a
  say ix it
end
do with item it index ix over a for 2
  say ix it
end
do with index ix over a
  say 'index' ix
end
do with index i item i over a
  say 'same' i
end
do with over a
  say 'bare'
end
b = .array~of(1, 2)
do with item it over b
  say 'it' it
  if it = 1 then b~append(3)
end
say b~items
call try 'do with item it; end'
call try 'do with index i index j over a; end'
call try 'do with item it over 5; end'
call try 'do with item it over .badsup~new; end'
call try 'do with over 5; say "five"; end'
call try 'do with over .onlysup~new; say "p"; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
::class badsup
::method supplier
  return 42
::class onlysup
::method supplier
  return .array~of(1)~supplier
```

```output
1 x
2 y
3 z
1 x
2 y
index 1
index 2
index 3
same x
same y
same z
bare
bare
bare
it 1
it 2
3
do with item it; end => 27.904
do with index i index j over a; end => 27.902
do with item it over 5; end => 97.1
do with item it over .badsup~new; end => 98.994
five
do with over 5; say "five"; end => ok
do with over .onlysup~new; say "p"; end => 98.913
```

## Conditions

r[instr.do.while]
A WHILE phrase ends the loop when its expression is false. It is tested before
every pass, after the step and the LIMIT and FOR tests of a controlled loop
and after the item of an OVER or WITH loop was assigned, so the expression
sees the control variable of the pass that it guards. A WHILE that is false
the first time runs no pass. The expression must be a [logical
value](r:expr.logical.value), and a value that is not `0` or `1` raises
`SYNTAX 34.3`. The list form `WHILE a, b` follows [the list rule of
IF](r:instr.if.list): the expressions are evaluated from left to right up to
the first that is false, and an expression that is evaluated and is not `0` or
`1` raises `SYNTAX 34.6`. WHILE and UNTIL cannot both appear in one clause,
and the attempt raises `SYNTAX 27.1`.

```rexx
n = 0
do while n < 3
  n = n + 1
  say 'w' n
end
do while 0
  say 'never'
end
do i = 1 to 5 while i < 3
  say 'wi' i
end
say 'i' i
do e over .array~of(1, 2, 3) while e < 3
  say 'w' e
end
say 'e' e
do 2 while 1, 1
  say 'list'
end
do while 0, 2
  say 'never'
end
call try 'do while 2; end'
call try 'do while "a"; end'
call try 'do while 1, 2; end'
call try 'do while 1 until 1; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
w 1
w 2
w 3
wi 1
wi 2
i 3
w 1
w 2
e 3
list
list
do while 2; end => 34.3
do while "a"; end => 34.3
do while 1, 2; end => 34.6
do while 1 until 1; end => 27.1
```

r[instr.do.until]
An UNTIL phrase ends the loop when its expression is true. It is tested after
every pass, so the block runs at least once, and it is tested before the
control variable steps. An ITERATE counts as the end of a pass, so it does
reach the test. The expression follows [the same rules](r:instr.do.while) as
for WHILE, and a value that is not `0` or `1` raises `SYNTAX 34.4`, or
`34.6` for an expression of the list form.

```rexx
do until 1
  say 'once'
end
do i = 1 to 5 until t(i)
  say 'body' i
  if i = 2 then iterate
  say 'tail' i
end
say 'i' i
do e over .array~of(1, 2, 3) until e = 2
  say 'u' e
end
say 'e' e
call try 'do until 2; end'
call try 'do until 1, 2; nop; end'
exit
t: say 'test' arg(1); return arg(1) = 3
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
once
body 1
tail 1
test 1
body 2
test 2
body 3
tail 3
test 3
i 3
u 1
u 2
e 2
do until 2; end => 34.4
do until 1, 2; nop; end => 34.6
```

## Counting passes

r[instr.do.counter]
A COUNTER phrase names a variable that holds the number of the current pass.
The variable is set to zero before the expressions of the repetitor are
evaluated, so they can read it, and then to 1, 2, and so on at the start of
each pass, once the tests have decided that the pass runs. A test of WHILE
therefore sees the number of the previous pass. An assignment to the variable
in the block does not disturb the count, which the loop keeps itself, so the
next pass sets the variable again. After the loop the variable holds the value
that the last pass left in it: the number of that pass unless the block
assigned the variable. The phrase belongs to a loop and not to a
simple group, and a COUNTER on a DO with nothing else raises `SYNTAX 27.905`.
The name may be a compound symbol, and it comes before any repetitor.

```rexx
do counter c i = 5 to 7
  say c i
end
do counter c while t(c) < 3
  say 'body' c
end
say 'c' c
c = 'x'
do counter c 0
end
say 'c' c
do counter c i = 1 to 3
  c = 100
  say c
end
say 'c' c
do counter c i = 1 to 3
  if i = 3 then c = 100
end
say 'c' c
c = 'old'
do counter c i = f(c) to f(c) + 1
end
do counter c.1 2
  say 'comp' c.1
end
do counter c e over .array~of('p', 'q', 'r')
  say c e
end
call try 'do counter c; end'
exit
t: say 'test' arg(1); return arg(1)
f: say 'f sees' arg(1); return 1
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
1 5
2 6
3 7
test 0
body 1
test 1
body 2
test 2
body 3
test 3
c 3
c 0
100
100
100
c 100
c 100
f sees 0
f sees 0
comp 1
comp 2
1 p
2 q
3 r
do counter c; end => 27.905
```

## Labels and the end of a block

r[instr.do.label]
A LABEL phrase gives the block a name that LEAVE, ITERATE and END can use. The
name is the label when the block has a LABEL phrase, and otherwise the control
variable of a controlled loop or of an OVER loop. A block with neither has no
name. The interpreter compares a name to the names in those clauses without
regard to case, and a compound symbol matches by its text, without working out
the tail. Once a block has a LABEL, the name of its control variable no longer
names it, so LEAVE, ITERATE and END have to use the label. When two active
blocks have the same name, the name means the innermost. The phrase works on
every form, a simple group included. It comes before any repetitor, and in
either order with COUNTER. A LABEL phrase does not create a label for CALL or
SIGNAL. A LABEL with no symbol after it, or with a string there, raises
`SYNTAX 20.918`.

```rexx
do label outer i = 1 to 2
  loop label inner j = 1 to 3
    if j = 2 then iterate outer
    say i j
  end inner
end OUTER
do label x 2
  do label x 2
    say 'inner'
    leave x
  end x
  say 'outer reached'
end x
call try 'do label a i = 1 to 2; end i'
call try 'do label a i = 1 to 2; leave i; end'
call try 'do label; end'
call try 'do label "q" 2; end'
call try 'do label a; say "simple"; end a'
signal on syntax name s
do label g
  say 'in'
end g
signal g
s: say condition('o')~code
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
1 1
2 1
inner
outer reached
inner
outer reached
do label a i = 1 to 2; end i => 10.2
do label a i = 1 to 2; leave i; end => 28.3
do label; end => 20.918
do label "q" 2; end => 20.918
simple
do label a; say "simple"; end a => ok
in
16.1
```

r[instr.do.end]
END closes the nearest DO or LOOP that is still open. It may be followed by
one symbol, which has to be [the name of the block](r:instr.do.label), so
`s.k` closes only `s.k` and not `s.j`. A symbol after the END of a block that
has no name, which includes a count, `FOREVER`, a bare condition, a WITH loop
and a loop that has only COUNTER, raises `SYNTAX 10.3`. A symbol that does not
match raises `SYNTAX 10.2`, and an END with no open block is [a part out of
place](r:instr.index.parts). A token after END that is not a symbol raises
`SYNTAX 20.909`, and a second token after the symbol `21.909`. A DO with no
END raises `SYNTAX 14.1` as a simple group and `14.5` as a loop. All of these
are found when the program is read, so no clause of the program runs.

```rexx
say 'before'
do i = 1 to 2; say i; end j
```

```output rc=246
```

```rexx
call try 'do Abc = 1 to 2; end aBC'
call try 'do s.k = 1 to 2; end S.K'
call try 'do s.k = 1 to 2; end s.j'
call try 'do e over .array~of(1); end e'
call try 'do 2; end foo'
call try 'do forever; leave; end 1'
call try 'do while 0; end x'
call try 'do with item it over .array~of(1); end it'
call try 'do counter c 1; end c'
call try 'do label a 1; end b'
call try 'do 1; end; end'
call try 'do 2; leave; end "x"'
call try 'do 2; leave; end (x)'
call try 'do i = 1 to 2; end i j'
call try 'do i = 1 to 2; end .i'
call try 'do; say "open"'
call try 'do 2; say "open"'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
do Abc = 1 to 2; end aBC => ok
do s.k = 1 to 2; end S.K => ok
do s.k = 1 to 2; end s.j => 10.2
do e over .array~of(1); end e => ok
do 2; end foo => 10.3
do forever; leave; end 1 => 10.3
do while 0; end x => 10.3
do with item it over .array~of(1); end it => 10.3
do counter c 1; end c => 10.3
do label a 1; end b => 10.2
do 1; end; end => 10.1
do 2; leave; end "x" => 20.909
do 2; leave; end (x) => 20.909
do i = 1 to 2; end i j => 21.909
do i = 1 to 2; end .i => 10.2
do; say "open" => 14.1
do 2; say "open" => 14.5
```

## Errors

r[instr.do.syntax]
Errors in the shape of the clause are found when the program is read, so a
program with one runs no clause at all, as [the read-first
rule](r:lex.index.read-first) says. The text of an INTERPRET is read in the
same way, as [the INTERPRET rule](r:instr.interpret.whole) says. A TO, BY,
FOR, INDEX or ITEM phrase given twice raises `SYNTAX 27.902`, an unwanted word
after FOREVER `27.901`, a WHILE and an UNTIL together `27.1`, an INDEX or ITEM
phrase with no OVER `27.904`, and a COUNTER on a simple group `27.905`. A
missing expression raises `SYNTAX 35.904` after the `=`, `35.905` after BY,
`35.906` after TO and `35.907` after FOR, `35.911` after OVER, and `35.929`
after WHILE or UNTIL. A LABEL with no symbol, or with a string there, raises
`SYNTAX 20.918`, a COUNTER with no symbol or with a string `20.934`, and an
INDEX or ITEM with no symbol `20.929`. A control, COUNTER, INDEX or ITEM name
that starts with a digit raises `31.2`, and one that starts with a period
`31.3`. Anything that follows a COUNTER or LABEL phrase out of its place is
part of the expression before it, so that the errors of that expression are
raised instead. The errors of a value, `26.2`, `26.3` and `41.1`, are raised
when the clause runs.

```rexx
say 'before'
do i = 1 to 3 by 1 to 5; end
```

```output rc=229
```

```rexx
say 'before'
do while 1 until 1; end
```

```output rc=229
```

```rexx
call try 'do i = 1 to by; end'
call try 'do i = 1 to 2 by; end'
call try 'do i = 1 for; end'
call try 'do i =; end'
call try 'do e over; end'
call try 'do with index i over; end'
call try 'do while; end'
call try 'do until; end'
call try 'do i = 1 to 2 while; end'
call try 'do with index; end'
call try 'do with item "i" over .array~of(1); end'
call try 'do .x = 1 to 2; end'
call try 'do 1 = 1 to 2; end'
call try 'do counter 1 2; end'
call try 'do counter; end'
call try 'do 2 label a; end'
call try 'do i = 1 to 2 counter c; end'
call try 'do 2 counter c; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
do i = 1 to by; end => 35.906
do i = 1 to 2 by; end => 35.905
do i = 1 for; end => 35.907
do i =; end => 35.904
do e over; end => 35.911
do with index i over; end => 35.911
do while; end => 35.929
do until; end => 35.929
do i = 1 to 2 while; end => 35.929
do with index; end => 20.929
do with item "i" over .array~of(1); end => 20.929
do .x = 1 to 2; end => 31.3
do 1 = 1 to 2; end => 31.2
do counter 1 2; end => 31.2
do counter; end => 20.934
do 2 label a; end => 26.2
do i = 1 to 2 counter c; end => 41.1
do 2 counter c; end => 26.2
```

## Deviations

r[instr.do.deviation.1]
The order in which a DO OVER over a Stem visits its tails is not specified.
ooRexx visits them in an order that its storage fixes, which is neither the
order of assignment nor a sorted order, and interrexx may visit them in
another. Which tails the loop visits is specified, as [the OVER
rule](r:instr.do.over) says.

```rexx
s. = 'dflt'
s.beta = 2; s.alpha = 1; s.gamma = 3
seen = .array~new
do t over s.
  seen~append(t)
end
say seen~items seen~sort~makeString('l', ' ')
```

```output
3 ALPHA BETA GAMMA
```

r[instr.do.deviation.25]
When the TO test or the direction test of a controlled loop sends a comparison
message to an object, the loop reads the answer as it reads a WHILE. The
answer `1` ends the loop, `0` lets it go on, and any other answer raises
`SYNTAX 34.901`. ooRexx ends the loop only when the answer is the `.true`
object itself, and goes on for every other answer, a computed `1` included, so
a loop that ends here can run for ever there. The direction test is the
message `<` that the loop sends to the step, and a computed `1` there marks a
negative step for interrexx and a positive one for ooRexx. The first example
shows the TO test, the second the answer `x`, and the third the direction
test, where interrexx ends the loop before the first pass and ooRexx runs four
passes.

```rexx
do i = .num~new(1) to 3
  say 'pass' i~v
end
say 'finished'
::class num
::attribute v
::method init
  expose v
  use arg v
::method '+'
  expose v
  if arg() = 0 then return self
  return .num~new(v + arg(1))
::method '>'
  expose v
  say 'compare' v arg(1)
  return (v > arg(1)) + 0
```

```output
compare 1 3
pass 1
compare 2 3
pass 2
compare 3 3
pass 3
compare 4 3
finished
```

```rexx
signal on syntax name e
do i = .num~new(1, 'x') to 3
  say 'never'
end
say 'unreached'
e: say 'error' condition('o')~code
::class num
::attribute v
::method init
  expose v ans
  use arg v, ans
::method '+'
  expose v ans
  if arg() = 0 then return self
  return .num~new(v + arg(1), ans)
::method '>'
  expose v ans
  return ans
```

```output
error 34.901
```

```rexx
do i = .num~new(1) to 3 by .num~new(1, 1)
  say 'pass' i~v
  if i~v > 3 then leave
end
say 'finished' i~v
::class num
::attribute v
::method init
  expose v ans
  use arg v, ans
  if arg() < 2 then ans = .nil
::method '+'
  expose v ans
  if arg() = 0 then return self
  return .num~new(v + arg(1)~v)
::method '>'
  expose v ans
  return (v > arg(1)) + 0
::method '<'
  expose v ans
  if ans \== .nil then return ans
  return (v < arg(1)) + 0
```

```output
finished 1
```

r[instr.do.deviation.26]
Entering `=` at the debug pause that follows a controlled loop with no pass
runs the loop clause again and goes on. ooRexx damages its record of open
blocks at that point, and raises `SYNTAX 10.1` later at an END that is valid.
In the example the answers come from a destination of `.DebugInput`, and
the trace output goes to standard error.

```rexx
.debuginput~destination(.answers~new)
trace ?a
do 2
  do i = 1 to 0
    say 'body'
  end
  say 'after inner'
end
say 'after'
exit
::class answers
::method init
  expose n
  n = 0
::method linein
  expose n
  n = n + 1
  if n = 2 then return '='
  return ''
```

```output
after inner
after inner
after
```

r[instr.do.deviation.38]
A TO or a BY phrase after the FOR phrase of a DO OVER or a DO WITH raises
`SYNTAX 25.927`, which names FOR and the word that followed. ooRexx raises
the internal error `49.2` for it, with a number of its own in the message.
Both read the clause before it runs, so no clause of the program runs.

```rexx
call try 'do e over .array~of(1) for 1 to 2; end'
call try 'do with item e over .array~of(1) for 1 by 2; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
do e over .array~of(1) for 1 to 2; end => 25.927
do with item e over .array~of(1) for 1 by 2; end => 25.927
```

r[instr.do.deviation.47]
A repetition count or a FOR count that has more digits than NUMERIC DIGITS
raises `SYNTAX 26.2` for the first and `26.3` for the second, or the
LOSTDIGITS condition first when that is trapped. ooRexx refuses such a count
too while NUMERIC DIGITS is below 9. At 9 or more it runs the loop when the
count is an unquoted integer literal of at most 18 digits, written without a
sign, a leading zero, a decimal point or an exponent, or that literal's own
value carried unchanged through variables, arguments, returns and collections.
A quoted string, a value that an operation produced, and a literal in any
other form that is wider than DIGITS are refused there as they are here, and
so is every count of 10 to the power of 18 or more.

```rexx
call try 'do 4294967296; leave; end'
call try 'do i = 1 for 1000000000; leave; end'
call try 'do 999999999; leave; end'
call try 'n = 4294967296; do n; leave; end'
call try 'do "4294967296"; leave; end'
call try 'do 4294967296 + 0; leave; end'
call try 'numeric digits 8; do 4294967296; leave; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
do 4294967296; leave; end => 26.2
do i = 1 for 1000000000; leave; end => 26.3
do 999999999; leave; end => ok
n = 4294967296; do n; leave; end => 26.2
do "4294967296"; leave; end => 26.2
do 4294967296 + 0; leave; end => 26.2
numeric digits 8; do 4294967296; leave; end => 26.2
```

r[instr.do.deviation.118]
`DO OVER` sends `MAKEARRAY` to an instance of a subclass of `List` as it does to
an instance of any other collection class, so that a `MAKEARRAY` of the subclass
is used and its answer decides the items. ooRexx reads such an instance as a
`List` directly and sends it no `MAKEARRAY`, unlike the subclasses of the other
collection classes. In the example the subclasses answer other items than they
hold.

```rexx
do x over .llist~of('held')
  say x
end
do x over .qsub~of('held')
  say x
end
::class llist subclass list
::method makearray
  say 'makearray sent'
  return .array~of('answered')
::class qsub subclass queue
::method makearray
  say 'makearray sent'
  return .array~of('answered')
```

```output
makearray sent
answered
makearray sent
answered
```
