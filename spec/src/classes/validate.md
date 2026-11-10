---
element: "Validate"
kind: class
rexxref: "Builtin Classes > Utility Classes > Validate Class; Builtin Classes > Utility Classes > Validate Class > classType (Class Method); Builtin Classes > Utility Classes > Validate Class > length (Class Method); Builtin Classes > Utility Classes > Validate Class > logical (Class Method); Builtin Classes > Utility Classes > Validate Class > nonNegativeNumber (Class Method); Builtin Classes > Utility Classes > Validate Class > nonNegativeWholeNumber (Class Method); Builtin Classes > Utility Classes > Validate Class > number (Class Method); Builtin Classes > Utility Classes > Validate Class > numberRange (Class Method); Builtin Classes > Utility Classes > Validate Class > position (Class Method); Builtin Classes > Utility Classes > Validate Class > positiveNumber (Class Method); Builtin Classes > Utility Classes > Validate Class > positiveWholeNumber (Class Method); Builtin Classes > Utility Classes > Validate Class > requestClassType (Class Method); Builtin Classes > Utility Classes > Validate Class > wholeNumber (Class Method); Builtin Classes > Utility Classes > Validate Class > wholeNumberRange (Class Method)"
deviations: [109]
---

# Validate

`Validate` is a set of class methods that check the arguments of a method
written in Rexx: that an argument is a number, a whole number in a range, a
position, a logical value or an instance of a class. A check that fails raises
the `SYNTAX` condition that the interpreter itself raises for such a
mistake, naming the argument, so that a method of the program refuses a bad
argument the way the methods of the library do. A check that passes answers
no value, except `requestClassType`, which answers the converted object.

[Deviation 109](r:class.index.deviation.109) applies to every argument below
that is read as a string or as a number.

## The class

r[class.validate.creation]
`Validate` is a class whose only superclass is `Object`, and it inherits no
mixin class ([the hierarchy rule](r:class.index.hierarchy)). Its instance side
defines no method, and its class side answers `classType`, `length`,
`logical`, `nonNegativeNumber`, `nonNegativeWholeNumber`, `number`,
`numberRange`, `position`, `positiveNumber`, `positiveWholeNumber`,
`requestClassType`, `wholeNumber` and `wholeNumberRange`. `new` makes an
object, which answers none of them: the methods are sent to the class.

```rexx
v = .validate
say v~superclasses~items v~superclass~id v~class~id
list = 'classType length logical nonNegativeNumber nonNegativeWholeNumber',
  'number numberRange position positiveNumber positiveWholeNumber',
  'requestClassType wholeNumber wholeNumberRange nosuchcheck'
do n over list~makearray(' ')
  say n~upper v~hasMethod(n)
end
s = v~methods(.nil)
say s~available
o = v~new
say o~class~id
signal on syntax name bad
o~number('x', 5)
bad: say condition('o')~code
```

```output
1 Object Class
CLASSTYPE 1
LENGTH 1
LOGICAL 1
NONNEGATIVENUMBER 1
NONNEGATIVEWHOLENUMBER 1
NUMBER 1
NUMBERRANGE 1
POSITION 1
POSITIVENUMBER 1
POSITIVEWHOLENUMBER 1
REQUESTCLASSTYPE 1
WHOLENUMBER 1
WHOLENUMBERRANGE 1
NOSUCHCHECK 0
0
Validate
97.1
```

## Arguments

r[class.validate.arguments]
Every check takes the name of the argument as its first argument, the value as
its second, and then its own further arguments. The name is only used in the
error: it may be any string, a name or a text, and it is the first entry of the
condition's `ADDITIONAL` array, whose other entries each rule below gives. The
counts are fixed: two arguments for `number` and `logical`, two and an optional
`digits` for the other number checks, three for `classType` and
`requestClassType`, and four and an optional `digits` for the range checks. A
call with fewer raises `SYNTAX 93.901`, one with more `93.902`, and one that
leaves out a required argument before the last one given `93.903`; omitted
arguments at the end count as not given. The `ADDITIONAL` array of the first
holds the least number of arguments, that of the second the most, and that of
the third the position of the first argument left out. The argument count is
checked first, then `digits`, and the value last. Every value and bound is read
as a string as [the conversion rule](r:conv.string.object) says. A check that
fails raises the `SYNTAX` error of its method, which the rules below give.

```rexx
call t 'number', 'N', 'x'
call t 'number', 'my arg', 'x'
call t 'number', 'N'
call t 'number', 'N', 5, 6
call t 'number', , 5
call t 'positiveNumber', 'N', , 9
call t 'numberRange', 'N', 5, , 9
call t 'classType', 'N', , .string
call t 'numberRange', 'N', 5, 1
call t 'numberRange', 'N', 5, 1, 9, 9, 9
call t 'wholeNumber', 'N', 'x', 0
call t 'wholeNumber', 'N', 'x', 5, 6
call t 'classType', 'N', 1
call u 'number("N", )'
call u 'numberRange("N", 5, 1, )'
exit
t: procedure
  use arg m
  signal on syntax name bad
  args = arg(2, 'A')
  .validate~sendWith(m, args~section(1, args~size))
  say m 'ok'
  return
bad: c = condition('o')
  say m c~code c~additional~items c~additional[1]
  return
u: procedure
  m = arg(1)
  signal on syntax name bad2
  interpret '.validate~'m
  say m 'ok'
  return
bad2: c = condition('o')
  say m c~code c~additional~items c~additional[1]
  return
```

```output
number 88.902 2 N
number 88.902 2 my arg
number 93.901 1 2
number 93.902 1 2
number 93.903 1 1
positiveNumber 93.903 1 2
numberRange 93.903 1 3
classType 93.903 1 2
numberRange 93.901 1 4
numberRange 93.902 1 5
wholeNumber 26.5 1 0
wholeNumber 93.902 1 3
classType 93.901 1 3
number("N", ) 93.901 1 2
numberRange("N", 5, 1, ) 93.901 1 4
```

r[class.validate.digits]
The checks that take `digits` compare and round under that many digits and not
under the NUMERIC DIGITS of the running code. `digits` is a whole number of at
least 1 and at most 999999999 and defaults to `9`, except for `length` and
`position`, whose default is [the number of digits that whole numbers have
internally](r:class.rexxinfo.limits). Any other value, a string that is no
number included, raises `SYNTAX 26.5` before the value is read; its `ADDITIONAL`
array holds the value of `digits`.

```rexx
numeric digits 20
call t 'wholeNumber', '1234567890'
call t 'wholeNumber', '1234567890', 9
call t 'wholeNumber', '1234567890', 10
call t 'wholeNumber', '12345678901234567890123', 25
call t 'wholeNumber', '1e12', 12
call t 'wholeNumber', '1e12', 13
call t 'wholeNumber', '5', 0
call t 'wholeNumber', '5', -1
call t 'wholeNumber', '5', 'x'
call t 'wholeNumber', '5', ''
call t 'wholeNumber', '5', 1.5
call t 'wholeNumber', '5', 999999999
call t 'wholeNumber', '5', 1000000000
call t 'wholeNumber', 'abc', 0
n = .rexxinfo~internalDigits
say chk('position', copies(9, n)) chk('position', copies(9, n + 1)),
    chk('position', copies(9, n + 1), n + 1)
exit
t: procedure
  use arg m, v, d
  signal on syntax name bad
  if arg(3, 'E') then .validate~send(m, 'N', v, d)
  else .validate~send(m, 'N', v)
  say m v '['arg(3)'] => ok'
  return
bad: say m v '['arg(3)'] =>' condition('o')~code
  return
chk: procedure
  use arg m, v, d
  signal on syntax name bad2
  if arg(3, 'E') then .validate~send(m, 'N', v, d)
  else .validate~send(m, 'N', v)
  return 'ok'
bad2: return condition('o')~code
```

```output
wholeNumber 1234567890 [] => 88.903
wholeNumber 1234567890 [9] => 88.903
wholeNumber 1234567890 [10] => ok
wholeNumber 12345678901234567890123 [25] => ok
wholeNumber 1e12 [12] => 88.903
wholeNumber 1e12 [13] => ok
wholeNumber 5 [0] => 26.5
wholeNumber 5 [-1] => 26.5
wholeNumber 5 [x] => 26.5
wholeNumber 5 [] => 26.5
wholeNumber 5 [1.5] => 26.5
wholeNumber 5 [999999999] => ok
wholeNumber 5 [1000000000] => 26.5
wholeNumber abc [0] => 26.5
ok 88.912 ok
```

```rexx
call t 0
call t 'y'
exit
t: procedure
  signal on syntax name bad
  .validate~wholeNumber('N', 5, arg(1))
  return
bad: c = condition('o')
  say c~code c~additional~items c~additional[1]
  return
```

```output
26.5 1 0
26.5 1 y
```

## Numbers

r[class.validate.number]
`number` accepts a string that is a number as [DATATYPE with the option
`N`](r:bif.datatype.number) says, and raises `SYNTAX 88.902` for any other
value, an object whose string value, read as [the conversion
rule](r:conv.string.object) says, is not a number included. `positiveNumber`
also needs the value to be greater than zero (`SYNTAX 88.926`) and
`nonNegativeNumber` to be zero or greater (`SYNTAX 88.925`); a value that is no
number raises the error of the method asked, so `88.926` or `88.925`. Both take
`digits`, and negative zero and `0.00` count as zero. The `ADDITIONAL` array of
every error holds the name and the value.

```rexx
vals = '5 -5 0 -0 0.00 5.5 1e3 1e-3 +7 abc 1E 2e+'
meths = 'number positiveNumber nonNegativeNumber'
do v over vals~makearray(' ')
  s = v~left(5)
  do m over meths~makearray(' ')
    s = s t(m, v)~left(7)
  end
  say s~strip('T')
end
say t('number', ' + 2 ') t('number', '2 E1') t('number', ''),
    t('number', .object~new)
signal on syntax name done
.validate~positiveNumber('my argument', '-3')
done: say condition('o')~code condition('o')~additional~makeString('l', ' ; ')
exit
t: procedure
  use arg m, v
  signal on syntax name bad
  .validate~send(m, 'N', v)
  return 'ok'
bad: return condition('o')~code
```

```output
5     ok      ok      ok
-5    ok      88.926  88.925
0     ok      88.926  ok
-0    ok      88.926  ok
0.00  ok      88.926  ok
5.5   ok      ok      ok
1e3   ok      ok      ok
1e-3  ok      ok      ok
+7    ok      ok      ok
abc   88.902  88.926  88.925
1E    88.902  88.926  88.925
2e+   88.902  88.926  88.925
ok 88.902 88.902 88.902
88.926 my argument ; -3
```

```rexx
call t .withstr~new
call t .withms~new
call t .withreq~new
exit
t: procedure
  use arg o
  signal on syntax name bad
  .validate~wholeNumber('N', o)
  say o~class~id 'ok'
  return
bad: say o~class~id condition('o')~code
  return
::class withstr
::method string
  say '  string'
  return '5'
::class withms
::method makestring
  say '  makestring'
  return '5'
::class withreq
::method request
  say '  request' arg(1)
  return '5'
```

```output defect=#164
  string
WITHSTR ok
  makestring
WITHMS ok
  request STRING
WITHREQ ok
```

r[class.validate.wholenumber]
`wholeNumber` accepts a number whose value, under `digits`, is a whole number
and raises `SYNTAX 88.903` for any other value: one with a fraction, one with
more integer digits than `digits` allows, and a string that is no number.
`positiveWholeNumber` also needs the value to be greater than zero (`88.905`),
`nonNegativeWholeNumber` to be zero or greater (`88.904`), `position` to be
greater than zero (`88.912`) and `length` to be zero or greater (`88.911`).
All take `digits`, whose default [differs for `position` and
`length`](r:class.validate.digits). A value with a trailing period or zeros
after the point, such as `5.` or `5.00`, and a value written with an exponent
that makes it whole, such as `1e3`, count as whole; negative zero counts as
zero. The `ADDITIONAL` array of every error holds the name and the value.

```rexx
vals = '5 -5 0 -0 5. 5.00 5.5 1e3 1e-3 +7 abc 1234567890 1.0e9'~makearray(' ')
vals~append('')
meths = 'wholeNumber positiveWholeNumber nonNegativeWholeNumber position',
  'length'
do v over vals
  s = ('['v']')~left(14)
  do m over meths~makearray(' ')
    s = s t(m, v)~left(7)
  end
  say s~strip('T')
end
exit
t: procedure
  use arg m, v
  signal on syntax name bad
  .validate~send(m, 'N', v)
  return 'ok'
bad: return condition('o')~code
```

```output
[5]            ok      ok      ok      ok      ok
[-5]           ok      88.905  88.904  88.912  88.911
[0]            ok      88.905  ok      88.912  ok
[-0]           ok      88.905  ok      88.912  ok
[5.]           ok      ok      ok      ok      ok
[5.00]         ok      ok      ok      ok      ok
[5.5]          88.903  88.905  88.904  88.912  88.911
[1e3]          ok      ok      ok      ok      ok
[1e-3]         88.903  88.905  88.904  88.912  88.911
[+7]           ok      ok      ok      ok      ok
[abc]          88.903  88.905  88.904  88.912  88.911
[1234567890]   88.903  88.905  88.904  ok      ok
[1.0e9]        88.903  88.905  88.904  ok      ok
[]             88.903  88.905  88.904  88.912  88.911
```

r[class.validate.range]
`numberRange(name, number, min, max, digits)` accepts a number from `min` up to
and including `max`, compared under `digits`, and raises `SYNTAX 88.907` for one
that is no number and for a value that compares below `min` or above `max` as
the `<` and `>` operators compare them. A `min` above `max` so refuses every
value, and needs no check of its own. A bound is read as [the conversion
rule](r:conv.string.object) says and need not be a number: a bound that is not a
number takes part in the comparison as a string, so that `5` is below `x` and
above `4x`, and a bound that is `.nil` raises `88.907`. `wholeNumberRange` has
the same arguments and also needs the value to be a whole number as
`wholeNumber` does; its bounds may have a fraction. The `ADDITIONAL` array of
the error holds the name, `min`, `max` and the value.

```rexx
call t 'numberRange', 5, 1, 10
call t 'numberRange', 1, 1, 10
call t 'numberRange', 10, 1, 10
call t 'numberRange', 0, 1, 10
call t 'numberRange', 11, 1, 10
call t 'numberRange', 5.5, 1, 10
call t 'numberRange', 'x', 1, 10
call t 'numberRange', 5, 10, 1
call t 'numberRange', 5, 5, 5
call t 'wholeNumberRange', 5, 1, 10
call t 'wholeNumberRange', 5.5, 1, 10
call t 'wholeNumberRange', 11, 1, 10
call t 'wholeNumberRange', 'x', 1, 10
call t 'wholeNumberRange', 5, 10, 1
call t 'numberRange', 1234567891, 1234567890, 1234567890, 9
call t 'numberRange', 1234567891, 1234567890, 1234567890, 12
call t 'numberRange', 5, 1, 'x'
call t 'numberRange', 5, 'x', 9
call t 'numberRange', 5, 1, '4x'
call t 'numberRange', 5, 1, .nil
call t 'wholeNumberRange', 5, 1.5, 9.5
exit
t: procedure
  use arg m, v, lo, hi, d
  signal on syntax name bad
  if arg(5, 'E') then .validate~send(m, 'N', v, lo, hi, d)
  else do
    d = '-'
    .validate~send(m, 'N', v, lo, hi)
  end
  say m v lo hi d '=> ok'
  return
bad: c = condition('o')
  if arg(5, 'E') then say m v lo hi d '=>' c~code
  else say m v lo hi '- =>' c~code c~additional~makeString('l', ' ; ')
  return
```

```output
numberRange 5 1 10 - => ok
numberRange 1 1 10 - => ok
numberRange 10 1 10 - => ok
numberRange 0 1 10 - => 88.907 N ; 1 ; 10 ; 0
numberRange 11 1 10 - => 88.907 N ; 1 ; 10 ; 11
numberRange 5.5 1 10 - => ok
numberRange x 1 10 - => 88.907 N ; 1 ; 10 ; x
numberRange 5 10 1 - => 88.907 N ; 10 ; 1 ; 5
numberRange 5 5 5 - => ok
wholeNumberRange 5 1 10 - => ok
wholeNumberRange 5.5 1 10 - => 88.907 N ; 1 ; 10 ; 5.5
wholeNumberRange 11 1 10 - => 88.907 N ; 1 ; 10 ; 11
wholeNumberRange x 1 10 - => 88.907 N ; 1 ; 10 ; x
wholeNumberRange 5 10 1 - => 88.907 N ; 10 ; 1 ; 5
numberRange 1234567891 1234567890 1234567890 9 => ok
numberRange 1234567891 1234567890 1234567890 12 => 88.907
numberRange 5 1 x - => ok
numberRange 5 x 9 - => 88.907 N ; x ; 9 ; 5
numberRange 5 1 4x - => 88.907 N ; 1 ; 4x ; 5
numberRange 5 1 The NIL object - => 88.907 N ; 1 ; The NIL object ; 5
wholeNumberRange 5 1.5 9.5 - => ok
```

## Logical values and classes

r[class.validate.logical]
`logical` accepts exactly the strings `0` and `1`, or the objects `.true` and
`.false`, and raises `SYNTAX 88.927` for any other value, such as `2`, a
number written otherwise such as `1.0` or `01`, a string with blanks and the
null string; the `ADDITIONAL` array holds the name and the value.

```rexx
call show .array~of('0', '1', .true, .false, '2')
call show .array~of('1.0', '01', ' 1', 'yes', '', .nil)
exit
show:
  do v over arg(1)
    say '['v~string']' t(v)
  end
  return
t: procedure
  use arg v
  signal on syntax name bad
  .validate~logical('flag', v)
  return 'ok'
bad: return condition('o')~code
```

```output
[0] ok
[1] ok
[1] ok
[0] ok
[2] 88.927
[1.0] 88.927
[01] 88.927
[ 1] 88.927
[yes] 88.927
[] 88.927
[The NIL object] 88.927
```

r[class.validate.classtype]
`classType(name, object, class)` accepts an object that is an instance of
`class`, a subclass or a class it inherits from included, as `isA` says, and
raises `SYNTAX 88.914` otherwise, with `ADDITIONAL` holding the name and the id
of the class. An argument `class` that is not a class raises `88.914` too,
naming the argument `class`; [the sends rule](r:class.validate.classtype.sends)
says where that error comes from. `requestClassType(name, object, class)` sends
the object the message `request` with the id of the class as its argument, and
answers the object that `request` answers when that is an instance of the class,
and raises `SYNTAX 93.978` for an answer that is not, `.nil` being an instance
of `Object` only; `ADDITIONAL` then holds the name, the id of the class and the
object.

```rexx
call c 'classType', 'abc', .string
call c 'classType', 'abc', .array
call c 'classType', .array~new, .collection
call c 'classType', .base~new, .mix
call c 'classType', .withmix~new, .mix
call c 'classType', .sub~new, .base
call c 'classType', .base~new, .sub
call c 'classType', .nil, .object
call c 'classType', 1, 'string'
call c 'requestClassType', 'abc', .string
call c 'requestClassType', .conv~new, .array
call c 'requestClassType', .conv~new, .string
call c 'requestClassType', .conv~new, .list
call c 'requestClassType', .conv~new, .object
call c 'requestClassType', 'abc', .array
call c 'requestClassType', .object~new, .string
exit
c: procedure
  use arg m, o, k
  signal on syntax name bad
  if m == 'classType' then do
    .validate~classType('N', o, k)
    say m k~string 'ok'
  end
  else do
    r = .validate~requestClassType('N', o, k)
    say m k~string 'answers' r~class~id
  end
  return
bad: cd = condition('o')
  say m k~string '=>' cd~code cd~additional~items cd~additional[1],
      cd~additional[2]~string
  return
::class base
::class sub subclass base
::class mix mixinclass object
::class withmix inherit mix
::class conv
::method request
  use arg c
  .output~lineout('  request' c)
  if c == 'Array' then return .array~of('converted')
  if c == 'String' then return 'convstr'
  return .nil
```

```output
classType The String class ok
classType The Array class => 88.914 2 N Array
classType The Collection class ok
classType The MIX class => 88.914 2 N MIX
classType The MIX class ok
classType The BASE class ok
classType The SUB class => 88.914 2 N SUB
classType The Object class ok
classType string => 88.914 2 class Class
requestClassType The String class answers String
  request Array
requestClassType The Array class answers Array
  request String
requestClassType The String class answers String
  request List
requestClassType The List class => 93.978 3 N List
  request Object
requestClassType The Object class answers Object
requestClassType The Array class answers Array
requestClassType The String class => 93.978 3 N String
```

r[class.validate.classtype.sends]
`classType` sends the object `isA` with `class` and accepts it when the answer
is `1`. So the `88.914` that a `class` that is not a class raises, naming the
argument `class`, is that of `isA` of `Object`, and an object whose own `isA`
answers `1` is accepted whatever `class` is. For any other answer it sends
`class` the message `id` for the error, so that a `class` that is not a class
raises the error of that message, `97.1` for a string and for `.nil`.
`requestClassType` sends `class` the message `id` first, then sends the object
`request` with that id, and last sends the answer `isA` with `class`, accepting
it when that answers `1`.

```rexx
call c 'classType', .liar~new, .array
call c 'classType', .liar~new, .liar
call c 'classType', .liar~new, 'string'
call c 'requestClassType', .conv~new, .array
call c 'requestClassType', .conv~new, 'string'
call c 'requestClassType', .conv~new, .nil
call c 'classType', .nolie~new, 'string'
call c 'classType', 'x', 'string'
exit
c: procedure
  use arg m, o, k
  signal on syntax name bad
  .validate~send(m, 'N', o, k)
  say m 'ok'
  return
bad: say m '=>' condition('o')~code
  return
::class liar
::method isA
  say '  isA' arg(1)~string
  return 1
::class nolie
::method isA
  return 0
::class conv
::method request
  use arg c
  say '  request' c
  return .liar~new
```

```output
  isA The Array class
classType ok
  isA The LIAR class
classType ok
  isA string
classType ok
  request Array
  isA The Array class
requestClassType ok
requestClassType => 97.1
requestClassType => 97.1
classType => 97.1
classType => 88.914
```

## Deviations

r[class.validate.deviation.109]
A value, a `digits` or a bound that is an instance of a subclass of `String`
is read by its own characters, and `REQUEST` and `MAKESTRING` are not sent to
it ([Deviation 109](r:class.string.deviation.109)). ooRexx sends such an
instance `request` with `STRING` or `MAKESTRING`, so an override of either
can change what is checked. In the example both methods answer `x`, and the
characters are a whole number.

```rexx
signal on syntax name bad
.validate~wholeNumber('N', .num~new('5'))
say 'accepted'
.validate~numberRange('N', .num~new('5'), 1, 10)
say 'in range'
.validate~number('N', .num~new('abc'))
say 'not reached'
bad: say condition('o')~code
::class num subclass string
::method makestring
  return 'x'
::method request
  use arg c
  return 'x'
```

```output defect=#140
accepted
in range
88.902
```
