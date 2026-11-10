---
element: "Properties"
kind: class
rexxref: "Builtin Classes > Collection Classes > Properties Class; Builtin Classes > Collection Classes > Properties Class > load (Class method); Builtin Classes > Collection Classes > Properties Class > new (Class method); Builtin Classes > Collection Classes > Properties Class > []=; Builtin Classes > Collection Classes > Properties Class > getLogical; Builtin Classes > Collection Classes > Properties Class > getProperty; Builtin Classes > Collection Classes > Properties Class > getWhole; Builtin Classes > Collection Classes > Properties Class > load; Builtin Classes > Collection Classes > Properties Class > put; Builtin Classes > Collection Classes > Properties Class > save; Builtin Classes > Collection Classes > Properties Class > setLogical; Builtin Classes > Collection Classes > Properties Class > setProperty; Builtin Classes > Collection Classes > Properties Class > setWhole"
deviations: [29, 33, 78, 109, 122, 123, 124]
---

# Properties

A `Properties` is a [`Directory`](directory.md) whose items are strings, with
methods to read and write them as text, as logical values and as whole
numbers, and to load and save them as lines of a file. It inherits
`Directory`, and through it [`MapCollection`](mapcollection.md) and
[`Collection`](collection.md); [the hierarchy rule](r:class.index.hierarchy)
lists what it inherits. Everything that the page of `Directory` says holds for
a `Properties`, the entries read by upper-case name, the methods associated
with names, `unknown` and the lists of the entries included; this page gives
what differs and what is added.

The page follows the order of the rexxref entry. Examples that show an error
use a small routine `try`, as the builtin pages do. Names and values of the
examples are strings, and the examples that use files create them under a
distinctive name and delete them. The page says nothing about what `load` does
with a file that cannot be read, other than one that does not exist.

## The class

r[class.properties.creation]
`Properties` declares the instance methods `[]=`, `getLogical`, `getProperty`,
`getWhole`, `load`, `put`, `save`, `setLogical`, `setProperty` and `setWhole`,
and the class method `load`; its `new` is that of `Directory` ([the new
rule](r:class.properties.new)). The class method `of` and the other
instance methods are those of `Directory` and the mixins. A properties object
answers the same `of` and set operations, `hasEntry`, `setMethod` and
`unsetMethod` as a directory ([the inheritance
rule](r:class.properties.inherited)).

```rexx
p = .properties~new
own = '[]= GETLOGICAL GETPROPERTY GETWHOLE LOAD PUT SAVE SETLOGICAL',
  'SETPROPERTY SETWHOLE'
n = 0
do m over own~makearray(' ')
  if p~hasMethod(m) then n += 1
end
say p~class~id n p~class~superclass~id .properties~hasMethod('LOAD')
say p~hasMethod('HASENTRY') p~hasMethod('SETMETHOD') p~hasMethod('UNION')
```

```output
Properties 10 Directory 1
1 1 1
```

## Making a properties object

r[class.properties.new]
`new` makes an empty properties object and takes the optional size hint of
[the `new` of `Directory`](r:class.directory.new): a whole number that is not
negative, or `SYNTAX 93.923`; a second argument raises `SYNTAX 93.902`; `init`
is sent with the arguments, and the checks come after the `init` of a
subclass.

```rexx
say .properties~new~items .properties~new(10)~items
call try '.properties~new(-1)'
call try '.properties~new("x")'
call try '.properties~new(1, 2)'
x = .watch~new(7)
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class watch subclass properties
::method init
  say 'init' arg()
  forward class (super)
```

```output
0 0
.properties~new(-1) => 93.923
.properties~new("x") => 93.923
.properties~new(1, 2) => 93.902
init 1
```

## Writing values

r[class.properties.put]
`put(value, name)`, `[name] = value` and `setProperty(name, value)` store a
string at the name. A value that is not a string is replaced by its string
value as [the string-value rule](r:conv.string.object) says, which sends it
`request` with `STRING`, so a number is stored as its digits, an object whose
`makeString` answers a string is stored as that answer, and an array as the
text that its `makeString` gives; `.nil` and an object with no `makeString`
raise `SYNTAX 93.938`. An instance of a subclass of `String` is stored as its
own characters ([the deviation rule](r:class.properties.deviation.109)). The
name is read as a string as for a directory ([the
index rule](r:class.directory.indexes)), and `.nil` as the name raises
`SYNTAX 88.909`. The three answer nothing, and none of them sends `put`, `[]=`
or `setProperty` to the receiver. `put` and `setProperty` take exactly two
arguments and `[]=` is the assignment `[name] = value`: fewer raises
`SYNTAX 93.901`, an omitted place before the last argument given raises
`SYNTAX 93.903`, and a third argument raises `SYNTAX 93.902`. The count is
checked first, then the value, then the name.

```rexx
p = .properties~new
p~put('v1', 'a')
p['b'] = 7
p~setProperty('c', .made~new)
p~put(.array~of(1, 2), 'd')
do i over p~allIndexes~sort
  say i p[i]~class~id '['p[i]~changeStr('0a'x, '/')']'
end
call try 'p~put(.nil, "k")'
call try 'p~put(.plain~new, "k")'
call try 'p["k"] = .nil'
call try 'p~setProperty("k", .plain~new)'
call try 'p~put("x", .nil)'
call try 'p~put("x")'
call try 'p~put(, "x")'
call try 'p~setProperty("x")'
call try 'p~setProperty(, "x")'
call try 'p~put("x", "y", "z")'
call try 'p~put(.nil, .nil)'
call try 'p~setProperty(.nil, .nil)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class made
::method makestring
  return 'made'
::class plain
```

```output
a String [v1]
b String [7]
c String [made]
d String [1/2]
p~put(.nil, "k") => 93.938
p~put(.plain~new, "k") => 93.938
p["k"] = .nil => 93.938
p~setProperty("k", .plain~new) => 93.938
p~put("x", .nil) => 88.909
p~put("x") => 93.901
p~put(, "x") => 93.903
p~setProperty("x") => 93.901
p~setProperty(, "x") => 93.903
p~put("x", "y", "z") => 93.902
p~put(.nil, .nil) => 93.938
p~setProperty(.nil, .nil) => 93.938
```

r[class.properties.unchecked]
`setEntry`, the assignment through `unknown` (`p~name = x`) and the methods
that `setMethod` associates do not check the value: an item stored by
`setEntry` or through `unknown` keeps its class and may be `.nil`. `putAll`
and `of` store with `put`, so they check.

```rexx
p = .properties~new
p~setEntry('e1', 5)
p~setEntry('e2', .nil)
p~setEntry('e3', .array~of(1))
p~name = .array~of(2)
say p~entry('E1')~class~id (p~entry('E2') == .nil) p~entry('E3')~class~id
say p~name~class~id p~items
call try 'p~putAll(.directory~of(.array~of("k", .nil)))'
call try '.properties~of(.array~of("k", .nil))'
x = .properties~of(.array~of('k', 5))
say x['k']~class~id
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
String 1 Array
Array 4
p~putAll(.directory~of(.array~of("k", .nil))) => 93.938
.properties~of(.array~of("k", .nil)) => 93.938
String
```

## Reading values

r[class.properties.getproperty]
`getProperty(name, default)` answers the item at the name, read with `at`
([the `at` rule](r:class.directory.at)), and when the directory has none the
default, or `.nil` when the default is omitted. The default is any object and
is not converted. It sends `at` to the receiver with the name. It takes one or
two arguments: none raises `SYNTAX 93.901`, an omitted name before a default
raises `SYNTAX 93.903` and a third raises `SYNTAX 93.902`.

```rexx
p = .properties~new
p~setProperty('a', 'x')
say p~getProperty('a') p~getProperty('A', 'dflt') p~getProperty('a', 'dflt')
say (p~getProperty('zz') == .nil) (p~getProperty('zz', ) == .nil)
say p~getProperty('zz', 5)~class~id p~getProperty('zz', .array~new)~class~id
call try 'p~getProperty'
call try 'p~getProperty(, 1)'
call try 'p~getProperty("a", "b", "c")'
call try 'p~getProperty(.nil)'
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
x dflt x
1 1
String Array
p~getProperty => 93.901
p~getProperty(, 1) => 93.903
p~getProperty("a", "b", "c") => 93.902
p~getProperty(.nil) => 88.909
```

r[class.properties.getlogical]
`getLogical(name, default)` reads the value as
[`getProperty`](r:class.properties.getproperty) does, with the same arguments,
which it sends to the receiver, and answers 1 for the value `1` or `true` and
0 for `0` or `false`. Case counts, and no other text is accepted:
`SYNTAX 34.904` is raised. The default is used when the name has no value and
is checked in the same way, so it must be one of those four strings (or a
logical value), or the error is raised; when the name has a value, the default
is not looked at. A missing name with no default raises `SYNTAX 34.904` too.
A value that is not a string is not converted and raises `SYNTAX 34.904`. The
arguments are counted as for `getProperty`.

```rexx
p = .properties~new
do v over .array~of('1', '0', 'true', 'false')
  p~setProperty('k', v)
  call charout , p~getLogical('k') ''
end
say
p~setProperty('k', 'true')
say p~getLogical('k', 'junk') p~getLogical('zz', 1)
say p~getLogical('zz', 'false')
say p~getLogical('zz', .true) p~getLogical('zz', .false)
do v over .array~of('TRUE', 'False', 'yes', '2', '-1', '00', '', ' 1')
  p~setProperty('k', v)
  call try 'p~getLogical("k")', '['v']'
end
p~setEntry('k', .text~new('true'))
call try 'p~getLogical("K")', 'object'
call try 'p~getLogical("zz")', 'missing'
call try 'p~getLogical("zz", "junk")', 'default junk'
call try 'p~getLogical("zz", .nil)', 'default nil'
call try 'p~getLogical', 'no name'
call try 'p~getLogical("a", 1, 2)', 'three'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(2) '=>' r
  return
bad: say arg(2) '=>' condition('o')~code
  return
::class text
::method init
  expose v
  use arg v
::method makestring
  expose v
  return v
::method request
  expose v
  if arg(1) = 'STRING' then return v
  return .nil
```

```output
1 0 1 0 
1 1
0
1 0
[TRUE] => 34.904
[False] => 34.904
[yes] => 34.904
[2] => 34.904
[-1] => 34.904
[00] => 34.904
[] => 34.904
[ 1] => 34.904
object => 34.904
missing => 34.904
default junk => 34.904
default nil => 34.904
no name => 93.901
three => 93.902
```

r[class.properties.getwhole]
`getWhole(name, default)` reads the value as `getProperty` does, with the same
arguments, which it sends to the receiver, and answers it as it is stored, not
reformatted, when it is a whole number, and raises `SYNTAX 26.901` when it is
not. A whole number is judged at `NUMERIC DIGITS` 9 whatever the caller's
setting: blanks around it are allowed, a sign, an exponent and a point
followed by zeros are allowed, and a value that needs more than nine digits is
not. The default is used when the name has no value and is checked in the same
way, and is not looked at when the name has a value. A missing name with no
default raises `SYNTAX 26.901` too. A value that is not a string is read
through [the string-value rule](r:conv.string.object), by one send of `request`
([Deviation 122](r:class.properties.deviation.122)); that holds for a default
as for a stored value.

```rexx
p = .properties~new
do v over .array~of('0', '12', '-5', '+7', '007', '1.0', '1e2', '3.', ' 8 ')
  p~setProperty('k', v)
  say '['v']' '['p~getWhole('k')']'
end
do v over .array~of('1.5', 'abc', '1000000000', '99999999999', '0.5', '')
  p~setProperty('k', v)
  call try 'p~getWhole("k")', '['v']'
end
p~setProperty('w', '12')
say p~getWhole('w', 'junk') p~getWhole('zz', 5) p~getWhole('zz', '2.0')
numeric digits 20
call try 'p~getWhole("zz", "12345678901")', 'digits 20'
numeric digits 3
say p~getWhole('zz', '123456')
call try 'p~getWhole("zz")', 'missing'
call try 'p~getWhole("zz", 1.5)', 'default 1.5'
call try 'p~getWhole("zz", .nil)', 'default nil'
call try 'p~getWhole', 'no name'
call try 'p~getWhole("a", 1, 2)', 'three'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(2) '=>' r
  return
bad: say arg(2) '=>' condition('o')~code
  return
```

```output
[0] [0]
[12] [12]
[-5] [-5]
[+7] [+7]
[007] [007]
[1.0] [1.0]
[1e2] [1e2]
[3.] [3.]
[ 8 ] [ 8 ]
[1.5] => 26.901
[abc] => 26.901
[1000000000] => 26.901
[99999999999] => 26.901
[0.5] => 26.901
[] => 26.901
12 5 2.0
digits 20 => 26.901
123456
missing => 26.901
default 1.5 => 26.901
default nil => 26.901
no name => 93.901
three => 93.902
```

```rexx
p = .properties~new
p~setEntry('b', .text~new('12'))
say p~getWhole('B')
::class text
::method init
  expose v
  use arg v
::method makestring
  return 'not read'
::method request
  expose v
  if arg(1) = 'STRING' then return v
  return .nil
```

```output defect=#164
12
```

## Setting logical and whole values

r[class.properties.setlogical]
`setLogical(name, value)` sends `datatype` with the argument `O` to the value
and, when that answers true, stores `true` for the value 1 and `false` for 0
with `setProperty`, which it sends to the receiver with the name and the new
text. A string is accepted only as `0` or `1`, and a logical value `.true` or
`.false` too: `true`, `2`, `00`, a blank around the digit and the null string
raise `SYNTAX 34.901`. A value that has no `datatype` method, `.nil` and an
instance of a class that does not understand the message among them, raises
`SYNTAX 97.1`; one whose `datatype` answers 1 but that is not a logical value
raises `SYNTAX 34.1`, and one that answers 0 raises `SYNTAX 34.901`. The
method answers nothing and takes exactly two arguments, counted as for `put`:
fewer raises `SYNTAX 93.901`, an omitted name before the value `SYNTAX 93.903`
and a third `SYNTAX 93.902`; the value is checked before the name.

```rexx
p = .properties~new
do v over .array~of('0', '1', .true, .false)
  p~setLogical('k', v)
  call charout , p['k'] ''
end
say
do v over .array~of('true', '2', '00', 'yes', '1.0', '', ' 1')
  call try 'p~setLogical("k", v)', '['v']'
end
call try 'p~setLogical("k", .nil)', 'nil'
call try 'p~setLogical("k", .object~new)', 'object'
call try 'p~setLogical("k", .answers~new(1))', 'answers 1'
call try 'p~setLogical("k", .answers~new(0))', 'answers 0'
call try 'p~setLogical("k")', 'one argument'
call try 'p~setLogical("k", 1, 2)', 'three'
call try 'p~setLogical(, 1)', 'omitted name'
call try 'p~setLogical(.nil, .nil)', 'both nil'
call try 'p~setLogical(.nil, 1)', 'name nil'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(2) '=> ok'
  return
bad: say arg(2) '=>' condition('o')~code
  return
::class answers
::method init
  expose r
  use arg r
::method datatype
  expose r
  say '  datatype' arg(1)
  return r
```

```output
false true true false 
[true] => 34.901
[2] => 34.901
[00] => 34.901
[yes] => 34.901
[1.0] => 34.901
[] => 34.901
[ 1] => 34.901
nil => 97.1
object => 97.1
  datatype O
answers 1 => 34.1
  datatype O
answers 0 => 34.901
one argument => 93.901
three => 93.902
omitted name => 93.903
both nil => 97.1
name nil => 88.909
```

r[class.properties.setwhole]
`setWhole(name, value)` sends `datatype` with the argument `W` to the value
and, when that answers true, stores the value as its string with
`setProperty`, which it sends to the receiver with the name and the text, as
it is, not reformatted. A string that is a whole number at `NUMERIC DIGITS` 9
is accepted: `12`, `+7`, `007`, `1.0`, `1e2`, `3.` and a blank around the
number. Anything else that answers a string, such as `1.5`, `abc`, the null
string or a number of more than nine digits, raises `SYNTAX 93.905`. A value
that has no `datatype` method raises `SYNTAX 97.1`; one whose `datatype`
answers 1 but that is not a string raises `SYNTAX 93.938`, and one that
answers 0 raises `SYNTAX 93.905`. The method answers nothing and takes exactly
two arguments, counted and checked as for `setLogical`.

```rexx
p = .properties~new
do v over .array~of('0', '12', '-5', '+7', '007', '1.0', '1e2', '3.', ' 8 ')
  p~setWhole('k', v)
  call charout , '['p['k']'] '
end
say
do v over .array~of('1.5', 'abc', '1000000000', '99999999999', '1e9', '')
  call try 'p~setWhole("k", v)', '['v']'
end
call try 'p~setWhole("k", .nil)', 'nil'
call try 'p~setWhole("k", .answers~new(1))', 'answers 1'
call try 'p~setWhole("k", .answers~new(0))', 'answers 0'
call try 'p~setWhole("k")', 'one argument'
call try 'p~setWhole("k", 1, 2)', 'three'
call try 'p~setWhole(, 1)', 'omitted name'
call try 'p~setWhole(.nil, .nil)', 'both nil'
call try 'p~setWhole(.nil, 1)', 'name nil'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(2) '=> ok'
  return
bad: say arg(2) '=>' condition('o')~code
  return
::class answers
::method init
  expose r
  use arg r
::method datatype
  expose r
  say '  datatype' arg(1)
  return r
```

```output
[0] [12] [-5] [+7] [007] [1.0] [1e2] [3.] [ 8 ] 
[1.5] => 93.905
[abc] => 93.905
[1000000000] => 93.905
[99999999999] => 93.905
[1e9] => 93.905
[] => 93.905
nil => 97.1
  datatype W
answers 1 => 93.938
  datatype W
answers 0 => 93.905
one argument => 93.901
three => 93.902
omitted name => 93.903
both nil => 97.1
name nil => 88.909
```

## Loading and saving

r[class.properties.load.format]
`load` reads its source line by line as `linein` does ([the line
rule](r:bif.linein.result)) and enters one property for each line that carries
one. A line that is empty or holds only blanks and tab characters is skipped,
and so is a line whose first characters after blanks and tab characters are
`--`; a `--` anywhere else is text. Any other line is split at its first `=`:
the name is the text before it with the blanks and tab characters removed from
both ends, and the value is all the text after it, blanks included. A line
with no `=` is a name with the null string as its value, and a line that
starts with `=` has the null string as its name. A later line with a name that
an earlier line or the properties object already has replaces the value.

```rexx
f = 'spec-properties-format.tmp'
call sysfiledelete f
t = '09'x
n = '0a'x
text = 'alpha=1'||n||'  beta = two words  '||n||'-- comment'||n,
  || t||'  -- indented comment'||n||n||' '||t||' '||n||'gamma'||n,
  || 'delta=x=y=z'||n||'=nokey'||n||'empty='||n||'alpha=3'||n,
  || 'Mixed=Case'||n||'k -- c=v'||n||'-'||n,
  || t||t||'tab'||t||'='||t||'v'||t||n||'last=noeol'
call charout f, text
call stream f, 'c', 'close'
p = .properties~load(f)
say p~items
do i over p~allIndexes~sort
  say '['i']' '=' '['p[i]']'
end
call sysfiledelete f
```

```output
11
[] = [nokey]
[-] = []
[Mixed] = [Case]
[alpha] = [3]
[beta] = [ two words  ]
[delta] = [x=y=z]
[empty] = []
[gamma] = []
[k -- c] = [v]
[last] = [noeol]
[tab] = [	v	]
```

r[class.properties.load.class]
The class method `load(source)` answers a new `Properties` that holds what the
source gives, an instance of `Properties` whatever class received the message,
and `new` is not sent. `source` is the name of a file or a `File` object,
which is read through a `Stream` made for it, or any other object, a `Stream`
included, which is sent `linein` with no argument until it raises the NOTREADY
condition, so a stream is read from its read position to its end, is opened by
its first `linein` when it was not open, and is not closed ([the instance
rule](r:class.properties.load)). A file that does not exist gives an empty
object and no error, and an object that does not understand `linein` raises
`SYNTAX 97.1`. The method takes exactly one argument: none raises `SYNTAX
93.901`, a second raises `SYNTAX 93.902`.

```rexx
f = 'spec-properties-load.tmp'
call sysfiledelete f
call lineout f, 'a=1'
call lineout f, 'b=2'
call lineout f
p = .properties~load(f)
say p~class~id p~items p['a']
p = .properties~load(.file~new(f))
say p~items p['b']
s = .stream~new(f)
s~open('read')
say s~linein
p = .properties~load(s)
say p~items p['a'] p['b']
s~close
q = .sub~load(f)
s2 = .stream~new(f)
say .properties~load(s2)~items s2~state
s2~close
say q~class~id q~items
say .properties~load('spec-properties-nothing-here.tmp')~items
p = .properties~load(.lines~new)
say p~items p['x'] p['y']
.local~k = 0
.local~a = 0
s3 = .counting~new(f)
say .properties~load(s3)~items .local~k .local~a s3~state
s3~close
call try '.properties~load(.nil)'
call try '.properties~load'
call try '.properties~load(f, 1)'
call sysfiledelete f
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class sub subclass properties
::method new class
  say 'new'
  forward class (super)
::class lines
::method init
  expose n
  n = 0
::method linein
  expose n
  n += 1
  if n = 1 then return 'x=1'
  if n = 2 then return 'y=2'
  raise notready
::class counting subclass stream
::method linein
  .local~k += 1
  .local~a += arg()
  forward class (super)
```

```output
Properties 2 1
2 2
a=1
1 The NIL object 2
2 NOTREADY
Properties 2
0
2 1 2
2 3 0 NOTREADY
.properties~load(.nil) => 97.1
.properties~load => 93.901
.properties~load(f, 1) => 93.902
```

r[class.properties.load]
The instance method `load(source)` reads the source as [the class
method](r:class.properties.load.class) does and adds what it reads to the
receiver, replacing the values of names that it already has and leaving the
others; it answers nothing. When the source is a name or a `File`, it first
sends `load` to the receiver with a `Stream` made for it, and that message
does the reading, so an override of `load` sees the message twice. For each
line that carries a property it then sends `put` to the receiver with the
value and the name, in the order of the lines, so an override of `put` sees
them and its checks apply. It takes exactly one argument, as the class method
does.

```rexx
f = 'spec-properties-iload.tmp'
g = 'spec-properties-isave.tmp'
call sysfiledelete f
call sysfiledelete g
call lineout f, 'n=1'
call lineout f, 'm=2'
call lineout f
.local~on = 0
p = .spy~new
p~put('keep', 'k')
p~put('old', 'n')
.local~on = 1
say 'by name'
p~load(f)
say p~items p['k'] p['n'] p['m']
say 'by stream'
p~load(.stream~new(f))
say 'save by name'
p~save(g)
.local~on = 0
call sysfiledelete f
call sysfiledelete g
call try 'p~load'
call try 'p~load(f, 1)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class spy subclass properties
::method load
  if .local~on then say 'load' arg(1)~class~id
  forward class (super)
::method put
  if .local~on then say 'put' arg(1) arg(2)
  forward class (super)
::method save
  if .local~on then say 'save' arg(1)~class~id
  forward class (super)
```

```output
by name
load String
load Stream
put 1 n
put 2 m
3 keep 1 2
by stream
load Stream
put 1 n
put 2 m
save by name
save String
save Stream
p~load => 93.901
p~load(f, 1) => 93.902
```

r[class.properties.save]
`save(target)` writes one line `name=value` for each entry, each ended by a
line feed, in an order that is not specified ([the hash order
rule](r:class.properties.deviation.33)); the name and the value are written as
they are, a value that is not a string through [the string-value
rule](r:conv.string.object), with no escaping, so a name that holds `=` or a
value that holds a line feed does not load back as it was. `target` is the name
of a file, whose old contents are replaced, a `File` object, treated the same,
or an object that understands `lineout`, such as a `Stream`, which is written at
its write position and is not closed. When the target is a name or a `File`,
`save` first sends `save` to the receiver with a `Stream` made for it, and that
message does the writing. The writing sends `supplier` to the receiver, and
`lineout` with the line as the only argument to the target for each entry. The
answer is nothing, and an object that does not understand `lineout` raises
`SYNTAX 97.1`. It takes exactly one argument: none raises `SYNTAX 93.901`, a
second raises `SYNTAX 93.902`.

```rexx
f = 'spec-properties-save.tmp'
call sysfiledelete f
p = .properties~new
p~put('a much longer value', 'k')
p~save(f)
say c2x(charin(f, 1, chars(f)))
call stream f, 'c', 'close'
p~put('x', 'k')
p~save(f)
say c2x(charin(f, 1, chars(f)))
call stream f, 'c', 'close'
p~save(.file~new(f))
say chars(f)
call stream f, 'c', 'close'
s = .stream~new(f)
s~open('write append')
p~save(s)
say s~state
s~close
say chars(f)
call stream f, 'c', 'close'
p~put('two', 'k2')
t = .recorder~new
p~save(t)
say t~lines~sort~makeString('L', '|')
q = .properties~new
q~put('', 'k=eq')
q~put('v', '')
q~save(f)
r = .properties~load(f)
say (r~at('k=eq') == .nil) r~at('k') r~items
call sysfiledelete f
call try 'p~save'
call try 'p~save(f, 1)'
call try 'p~save(.nil)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> ok'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class recorder
::method init
  expose lines
  lines = .array~new
::method lineout
  expose lines
  lines~append(arg(1))
  return 0
::method lines
  expose lines
  return lines
```

```output
6B3D61206D756368206C6F6E6765722076616C75650A
6B3D780A
4
READY
8
k2=two|k=x
1 eq= 2
p~save => 93.901
p~save(f, 1) => 93.902
p~save(.nil) => 97.1
```

```rexx
p = .properties~new
p~setEntry('o', .text~new('val'))
t = .recorder~new
p~save(t)
say t~lines~makeString('L', '|')
::class text
::method init
  expose v
  use arg v
::method makestring
  return 'not read'
::method request
  expose v
  if arg(1) = 'STRING' then return v
  return .nil
::class recorder
::method init
  expose lines
  lines = .array~new
::method lineout
  expose lines
  lines~append(arg(1))
  return 0
::method lines
  expose lines
  return lines
```

```output defect=#164
O=val
```

## What a properties object inherits

r[class.properties.inherited]
The methods that `Properties` does not declare are those of `Directory`: the
entries read by upper-case name (`entry`, `hasEntry`, `setEntry`,
`removeEntry`), `at`, `remove`, `removeItem`, `hasIndex`, `hasItem`, `index`,
`items`, `isEmpty`, `empty`, `copy`, `supplier`, `allIndexes`, `allItems`,
`makeArray`, `unknown` and `setMethod` and `unsetMethod`, and those of the
mixins, `of`, `putAll` and the set operations, and each behaves as the page of
`Directory` says, of a properties object and of an instance of a subclass of
it. The copy is a `Properties`. The `unknown` that reads and sets the entries
by message name does not check the value ([the unchecked
rule](r:class.properties.unchecked)). A search for the very item stored sends
its `==` as for any other item ([the Properties
rule](r:class.properties.deviation.78)). Its string value is `a Properties`.

```rexx
p = .many~new
p~put('v', 'k')
p~setMethod('m', 'return "computed"')
p~title = 'x'
c = p~copy
say c~class~id c~items c~at('M') c~entry('TITLE') p~hasEntry('title')
say p p~string (p~request('STRING') == .nil)
say p~removeEntry('title') p~items p~union(c)~class~id
say .properties~new~isEmpty
::class many subclass properties
```

```output
MANY 3 computed x 1
a MANY a MANY 1
x 2 MANY
1
```

## Messages the methods send

r[class.properties.sends]
The messages that `Properties` methods send to the receiver, to the values and
to the source or target, are these. `getProperty` sends `at` with the name;
`getLogical` and `getWhole` send `getProperty` with their own arguments;
`setLogical` and `setWhole` send `datatype` to the value, and `setProperty`
with the name and the text; `setProperty`, `put` and `[]=` send nothing to the
receiver; `load` and `save` send what [the load rule](r:class.properties.load)
and [the save rule](r:class.properties.save) say, `linein` with no argument to
a source that is not a name or a `File` and `lineout` with the line to a
target that is not a name or a `File`. The methods `put`, `setProperty`, `[]=`
and the setters read a value that is not a string with `request` and the
argument `STRING` ([the put rule](r:class.properties.put)), and `save` sends
`request` with `STRING` to a stored value that is not a string, and `getWhole`
sends it once ([the getWhole rule](r:class.properties.getwhole) and [the save
rule](r:class.properties.save)).
In the example each method of a subclass records its name, the number of its
arguments and its first argument, and each call shows the records it caused.

```rexx
f = 'spec-properties-sends.tmp'
own = '[]= put at getProperty setProperty getLogical getWhole setLogical',
  'setWhole unknown'
.local~log = .array~new
.local~on = 0
do m over own~makeArray(' ')
  src = "if .local~on then .local~log~append('"m"' arg()':'arg(1));",
    "forward class (super)"
  .spy~define(m, src)
end
a = .spy~new
a~put('1', 'n')
a~put('true', 't')
calls = 'a~setProperty("k", "v")|x = a~getProperty("k")',
  '|x = a~getProperty("zz", 1)',
  '|x = a~getLogical("t")|x = a~getLogical("zz", 1)|x = a~getWhole("n")',
  '|x = a~getWhole("zz", 5)|a~setLogical("k", 1)|a~setWhole("k", 3)',
  '|a["k"] = "v"|a~put("v", "k")|a~name = "v"|x = a~name'
do c over calls~makeArray('|')
  .local~log~empty
  .local~on = 1
  interpret c
  .local~on = 0
  say c '=>' .local~log~makeString('L', ' ')
end
::class spy subclass properties
```

```output
a~setProperty("k", "v") => setProperty 2:k
x = a~getProperty("k")  => getProperty 1:k at 1:k
x = a~getProperty("zz", 1)  => getProperty 2:zz at 1:zz
x = a~getLogical("t") => getLogical 1:t getProperty 1:t at 1:t
x = a~getLogical("zz", 1) => getLogical 2:zz getProperty 2:zz at 1:zz
x = a~getWhole("n")  => getWhole 1:n getProperty 1:n at 1:n
x = a~getWhole("zz", 5) => getWhole 2:zz getProperty 2:zz at 1:zz
a~setLogical("k", 1) => setLogical 2:k setProperty 2:k
a~setWhole("k", 3)  => setWhole 2:k setProperty 2:k
a["k"] = "v" => []= 2:v
a~put("v", "k") => put 2:v
a~name = "v" => unknown 2:NAME=
x = a~name => unknown 2:NAME
```

## Deviations

r[class.properties.deviation.29]
An assignment through `unknown` that gives no value, by message or by a direct
call, is refused as it is for a directory ([the Directory
rule](r:class.directory.deviation.29)): interrexx ends the program with exit
code 120, and ooRexx stores an unrelated leftover object for the message and
removes the entry for the direct call. See [Deviation 29](../deviations.md).

```rexx
say 'before'
p = .properties~new
p~"A="()
say 'after'
```

```output rc=120
before
```

r[class.properties.deviation.33]
The order of the lines that `save` writes, and the order of the lists and
searches that `Properties` inherits from `Directory`, is not specified, and
may differ from ooRexx's ([the Directory
rule](r:class.directory.deviation.33)). Which lines are written, and how
often, is specified. In the example the lines are sorted.

```rexx
p = .properties~new
p~put('1', 'one')
p~put('2', 'two')
p~put('3', 'three')
t = .recorder~new
p~save(t)
say t~lines~sort~makeString('L', '|')
::class recorder
::method init
  expose lines
  lines = .array~new
::method lineout
  expose lines
  lines~append(arg(1))
  return 0
::method lines
  expose lines
  return lines
```

```output
one=1|three=3|two=2
```

r[class.properties.deviation.78]
A search for an item that was stored by `setEntry` or through `unknown`, a
string or not, with the very object that is stored, by `hasItem`, `index` or
`removeItem`, sends that object's `==` as for any other item, so an object
whose `==` answers false is not found even at its own entry ([the Directory
rule](r:class.directory.deviation.78)). ooRexx compares identity first, finds
it and sends nothing. `put` stores a string of the value's characters, so the
object given to `put` is not the object stored. See [Deviation
78](../deviations.md).

```rexx
.local~n = 0
p = .properties~new
q = .thing~new
p~setEntry('k', q)
say p~hasItem(q) .local~n
say (p~index(q) == .nil) .local~n
say (.nil == p~removeItem(q)) .local~n p~items
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

```rexx
.local~n = 0
p = .properties~new
s = .ls~new('v')
p~setEntry('k', s)
say p~hasItem(s) .local~n
::class ls subclass string
::method '=='
  .local~n += 1
  return 0
```

```output
0 1
```

r[class.properties.deviation.109]
A value that is an instance of a subclass of `String` is read by its own
characters by `put`, `[]=`, `setProperty`, `setLogical` and `setWhole`, and
`REQUEST` and `MAKESTRING` are not sent to it ([Deviation
109](r:class.string.deviation.109)); a name is read as [a directory reads
it](r:class.directory.deviation.109). ooRexx sends such a value `request`
with `STRING` or `MAKESTRING`, so an override of either changes the text that
is stored, and can make `setLogical` raise `SYNTAX 34.1` and `setWhole` raise
`SYNTAX 93.905`. In the example `MAKESTRING` answers another text.

```rexx
p = .properties~new
p~put(.sub~new('abc'), 'k')
p~setProperty('k2', .sub~new('def'))
p~setWhole('w', .sub~new('12'))
p~setLogical('l', .sub~new('1'))
say p['k'] p['k2'] p['w'] p['l']
::class sub subclass string
::method makestring
  return 'other'
```

```output defect=#140
abc def 12 true
```

r[class.properties.deviation.122]
`getWhole` sends `request` with `STRING` once to a stored value or a default
that is not a string, as [the getWhole rule](r:class.properties.getwhole) says.
ooRexx sends it twice to either.

```rexx
.local~n = 0
p = .properties~new
p~setEntry('b', .text~new('12'))
say p~getWhole('B') .local~n
::class text
::method init
  expose v
  use arg v
::method request
  expose v
  .local~n += 1
  if arg(1) = 'STRING' then return v
  return .nil
```

```output defect=#164
12 1
```

r[class.properties.deviation.123]
`isEmpty` agrees with `items`, as [the directory
rule](r:class.directory.deviation.123) says: it answers 0 for a properties
object that holds a method of any name but `UNKNOWN`, also when that is all it
holds. ooRexx answers 1 for one that holds only methods.

```rexx
p = .properties~new
p~setMethod('m', 'return 1')
say p~items p~isEmpty
p~setMethod('unknown', 'return 2')
say p~items p~isEmpty
```

```output
1 0
1 0
```

r[class.properties.deviation.124]
An answer of `request` with `ARRAY` that is not an `Array` or an instance of a
subclass of `Array` is refused as the array of arguments of `unknown`, which
raises `SYNTAX 98.913`, whatever collection the answer is, also when the
message name ends in `=` or the arguments are otherwise ignored. ooRexx
accepts a `Queue` or a `CircularQueue` as such an answer and uses its items,
and crashes for a `List`, a `Bag`, a `Table` or a `Directory`, among others. In
the example the answer is a queue.

```rexx
p = .properties~new
call try "p~unknown('Q=', .ans~new)"
call try "p~unknown('Q', .ans~new)"
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say arg(1) '=> no error'
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class ans
::method request
  return .queue~of('x')
```

```output
p~unknown('Q=', .ans~new) => 98.913
p~unknown('Q', .ans~new) => 98.913
```
