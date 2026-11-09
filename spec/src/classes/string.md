---
element: "String"
kind: class
rexxref: "Builtin Classes > Fundamental Classes > String Class; Builtin Classes > Fundamental Classes > String Class > new (Class Method); Builtin Classes > Fundamental Classes > String Class > alnum (Class Method); Builtin Classes > Fundamental Classes > String Class > alpha (Class Method); Builtin Classes > Fundamental Classes > String Class > blank (Class Method); Builtin Classes > Fundamental Classes > String Class > cntrl (Class Method); Builtin Classes > Fundamental Classes > String Class > cr (Class Method); Builtin Classes > Fundamental Classes > String Class > digit (Class Method); Builtin Classes > Fundamental Classes > String Class > graph (Class Method); Builtin Classes > Fundamental Classes > String Class > lower (Class Method); Builtin Classes > Fundamental Classes > String Class > nl (Class Method); Builtin Classes > Fundamental Classes > String Class > null (Class Method); Builtin Classes > Fundamental Classes > String Class > print (Class Method); Builtin Classes > Fundamental Classes > String Class > punct (Class Method); Builtin Classes > Fundamental Classes > String Class > space (Class Method); Builtin Classes > Fundamental Classes > String Class > tab (Class Method); Builtin Classes > Fundamental Classes > String Class > upper (Class Method); Builtin Classes > Fundamental Classes > String Class > xdigit (Class Method); Builtin Classes > Fundamental Classes > String Class > Arithmetic Methods; Builtin Classes > Fundamental Classes > String Class > Comparison Methods; Builtin Classes > Fundamental Classes > String Class > Concatenation Methods; Builtin Classes > Fundamental Classes > String Class > Logical Methods; Builtin Classes > Fundamental Classes > String Class > []; Builtin Classes > Fundamental Classes > String Class > ? (inline if); Builtin Classes > Fundamental Classes > String Class > abbrev; Builtin Classes > Fundamental Classes > String Class > abs; Builtin Classes > Fundamental Classes > String Class > append; Builtin Classes > Fundamental Classes > String Class > b2x; Builtin Classes > Fundamental Classes > String Class > bitAnd; Builtin Classes > Fundamental Classes > String Class > bitOr; Builtin Classes > Fundamental Classes > String Class > bitXor; Builtin Classes > Fundamental Classes > String Class > c2d; Builtin Classes > Fundamental Classes > String Class > c2x; Builtin Classes > Fundamental Classes > String Class > caselessAbbrev; Builtin Classes > Fundamental Classes > String Class > caselessChangeStr; Builtin Classes > Fundamental Classes > String Class > caselessCompare; Builtin Classes > Fundamental Classes > String Class > caselessCompareTo; Builtin Classes > Fundamental Classes > String Class > caselessContains; Builtin Classes > Fundamental Classes > String Class > caselessContainsWord; Builtin Classes > Fundamental Classes > String Class > caselessCountStr; Builtin Classes > Fundamental Classes > String Class > caselessEndsWith; Builtin Classes > Fundamental Classes > String Class > caselessEquals; Builtin Classes > Fundamental Classes > String Class > caselessLastPos; Builtin Classes > Fundamental Classes > String Class > caselessMatch; Builtin Classes > Fundamental Classes > String Class > caselessMatchChar; Builtin Classes > Fundamental Classes > String Class > caselessPos; Builtin Classes > Fundamental Classes > String Class > caselessStartsWith; Builtin Classes > Fundamental Classes > String Class > caselessWordPos; Builtin Classes > Fundamental Classes > String Class > ceiling; Builtin Classes > Fundamental Classes > String Class > center/centre; Builtin Classes > Fundamental Classes > String Class > changeStr; Builtin Classes > Fundamental Classes > String Class > compare; Builtin Classes > Fundamental Classes > String Class > compareTo; Builtin Classes > Fundamental Classes > String Class > contains; Builtin Classes > Fundamental Classes > String Class > containsWord; Builtin Classes > Fundamental Classes > String Class > copies; Builtin Classes > Fundamental Classes > String Class > countStr; Builtin Classes > Fundamental Classes > String Class > d2c; Builtin Classes > Fundamental Classes > String Class > d2x; Builtin Classes > Fundamental Classes > String Class > dataType; Builtin Classes > Fundamental Classes > String Class > decodeBase64; Builtin Classes > Fundamental Classes > String Class > delStr; Builtin Classes > Fundamental Classes > String Class > delWord; Builtin Classes > Fundamental Classes > String Class > encodeBase64; Builtin Classes > Fundamental Classes > String Class > endsWith; Builtin Classes > Fundamental Classes > String Class > equals; Builtin Classes > Fundamental Classes > String Class > floor; Builtin Classes > Fundamental Classes > String Class > format; Builtin Classes > Fundamental Classes > String Class > hashCode; Builtin Classes > Fundamental Classes > String Class > insert; Builtin Classes > Fundamental Classes > String Class > lastPos; Builtin Classes > Fundamental Classes > String Class > left; Builtin Classes > Fundamental Classes > String Class > length; Builtin Classes > Fundamental Classes > String Class > lower; Builtin Classes > Fundamental Classes > String Class > makeArray; Builtin Classes > Fundamental Classes > String Class > makeString; Builtin Classes > Fundamental Classes > String Class > match; Builtin Classes > Fundamental Classes > String Class > matchChar; Builtin Classes > Fundamental Classes > String Class > max; Builtin Classes > Fundamental Classes > String Class > min; Builtin Classes > Fundamental Classes > String Class > modulo; Builtin Classes > Fundamental Classes > String Class > overlay; Builtin Classes > Fundamental Classes > String Class > pos; Builtin Classes > Fundamental Classes > String Class > replaceAt; Builtin Classes > Fundamental Classes > String Class > reverse; Builtin Classes > Fundamental Classes > String Class > right; Builtin Classes > Fundamental Classes > String Class > round; Builtin Classes > Fundamental Classes > String Class > sign; Builtin Classes > Fundamental Classes > String Class > space; Builtin Classes > Fundamental Classes > String Class > startsWith; Builtin Classes > Fundamental Classes > String Class > strip; Builtin Classes > Fundamental Classes > String Class > subChar; Builtin Classes > Fundamental Classes > String Class > substr; Builtin Classes > Fundamental Classes > String Class > subWord; Builtin Classes > Fundamental Classes > String Class > subWords; Builtin Classes > Fundamental Classes > String Class > translate; Builtin Classes > Fundamental Classes > String Class > trunc; Builtin Classes > Fundamental Classes > String Class > upper; Builtin Classes > Fundamental Classes > String Class > verify; Builtin Classes > Fundamental Classes > String Class > word; Builtin Classes > Fundamental Classes > String Class > wordIndex; Builtin Classes > Fundamental Classes > String Class > wordLength; Builtin Classes > Fundamental Classes > String Class > wordPos; Builtin Classes > Fundamental Classes > String Class > words; Builtin Classes > Fundamental Classes > String Class > x2b; Builtin Classes > Fundamental Classes > String Class > x2c; Builtin Classes > Fundamental Classes > String Class > x2d"
deviations: [4, 36, 109]
---

# String

`String` is the class of every string of characters: a literal, a number, and
the result of any operation that makes text. A program never needs to make
one with `new`, but it can, and it can subclass `String` to give strings
more behavior. `String` is a subclass of `Object` and inherits
`Comparable`, so a string also understands the methods of
[Object](object.md).

Most methods of `String` are the methods that a built-in function runs, such
as `left` and `pos`. The behavior of each belongs to the page of its function,
and [one rule](r:class.string.builtins) lists them, with what changes when one
is sent as a message. The operator methods, such as `+` and `||`, belong to
the expressions chapter, and [one rule](r:class.string.operators) lists them
too. The class methods that name sets of characters and make strings, and the
methods that only the class has, such as `caselessEquals`, `makeArray` and
`[]`, are specified in full below. Examples that show an error use a small
routine `try`, as the builtin pages do.

## The class

r[class.string.value]
Every string is an instance of `String`, whether it is a literal, a number,
the result of a calculation or of a comparison, or the answer of a method:
`class` answers `String` for all of them, and there is no separate class for
numbers. A string is its own string value, so `string` and `makeString` answer
a string with the same characters.

```rexx
say 'abc'~class~id 12~class~id (3 + 4)~class~id (1 = 1)~class~id
say ('a' || 'b')~class~id 'a'~c2x~class~id .string~new('x')~class~id
say 12~isA(.string) 'abc'~isInstanceOf(.string) 12~string 12~makeString
```

```output
String String String String
String String String
1 1 12 12
```

r[class.string.hierarchy]
The superclass of `String` is `Object`, and its list of superclasses ends with
the mixin class `Comparable`, as [the hierarchy
rule](r:class.index.hierarchy) lists. A string is therefore an instance of
`Comparable` too. `String` is a class of the library, which accepts no change
to its methods or superclasses ([the library
rule](r:class.methods.library)).

```rexx
say .string~superclasses~items .string~superclass~id
say .string~superclasses~at(2)~id
say 'a'~isA(.comparable) 'a'~isA(.object) 12~isA(.comparable)
signal on syntax name e
.string~inherit(.comparable)
e: say condition('o')~code
```

```output
2 Object
Comparable
1 1 1
98.985
```

r[class.string.subclass]
A program makes a subclass of `String` with
[the directive](r:dir.class.subclass) or with the `subclass` method of
`Class`. An instance is made with [`new`](r:class.string.new), takes the
characters given there, and has the class of the receiver of `new`. It is a
string wherever a string is wanted, and the class-side methods of `String` are
understood by the subclass. A String method sent to the instance works on the
characters of the instance and answers a new string of class `String`, not an
instance of the subclass, and `copy`, which `Object` supplies, answers another
instance of the subclass. Wherever a string, a number, a logical value or a
whole number is read from the instance, as the receiver or as an argument, its
own characters are read and neither `MAKESTRING` nor `STRING` is sent to it
([Deviation 109](r:class.string.deviation.109)). The instance may have
attributes and methods of its own, and an object name
([Deviation 36](r:class.string.deviation.36)).

```rexx
x = .shout~new('abc')
say x~class~id x~length x~left(2) x~left(2)~class~id x~shout
say .shout~nl~c2x (x || 'd')~class~id (x = 'abc') x~copy~class~id
::class shout subclass string
::method shout
  return self~upper'!'
```

```output
SHOUT 3 ab String ABC!
0A String 1 SHOUT
```

r[class.string.args]
A position, length or count argument of a method of `String` is read as
[a whole-number argument](r:conv.whole.form) of a builtin function is, with
the value that [the digits rule](r:conv.whole.digits) gives and
[its limit](r:num.limits.whole). This holds for the methods that only `String`
has as well as for the builtin ones. An instance of a subclass of `String`
given as such an argument is read by its own characters
([Deviation 109](r:class.string.deviation.109)). A failure of either raises
the range code of the argument (`SYNTAX 93.924`, `93.923` or `93.906`), not
40.12.

```rexx
say 'abc'~"[]"('1.0000000000000000001') 'abc'~subChar(' +2 ')
say '['||'abc'~"[]"(1000)']'
numeric digits 3
say '['||'abcdef'~"[]"(1000)']' 'abcdef'~"[]"('2e0')
signal on syntax name a
say 'abc'~"[]"(1000000000000000000)
a: say condition('o')~code
signal on syntax name b
say 'abc'~"[]"(1, 1000000000000000000)
b: say condition('o')~code
signal on syntax name c
say 'abc'~match(1.5, 'a')
c: say condition('o')~code
```

```output
a b
[]
[] b
93.924
93.923
93.924
```

## Class methods

r[class.string.new]
`new` makes a string. Its first argument is required, and gives the
characters: the string value of the argument, which is the argument itself for
a string, a number and an instance of a subclass of `String` included, and its
`MAKESTRING` answer for another object. For another object `new` uses
`MAKESTRING` only, as the methods that take a string argument do
([the string-value rule](r:class.index.string)). Without an argument `new`
raises `SYNTAX 93.903`, and with an argument that has no `MAKESTRING` method,
`.nil` and an object with only a `string` method included, `SYNTAX 88.909`.
The object that `new` makes has the class of the receiver, and the characters
are in place before `init` runs, so an `init` of a subclass can use them.
`init` is sent the arguments after the first, as
[the new rule](r:class.index.new) says, so a second argument raises
`SYNTAX 93.902` for a class whose `init` is that of `Object`.

```rexx
say .string~new('abc') .string~new(12)~class~id .string~new('')~length
call try ".string~new"
call try ".string~new(.nil)"
call try ".string~new(.object~new)"
call try ".string~new(.st~new)"
call try ".string~new('a', 'b')"
call try ".string~new('a', )"
x = .si~new('abc', 1, 2)
say x
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class si subclass string
::method init
  say 'init' self~length self 'with' arg() 'arguments'
::class st
::method string
  return 'q'
```

```output
abc String 0
.string~new => 93.903
.string~new(.nil) => 88.909
.string~new(.object~new) => 88.909
.string~new(.st~new) => 88.909
.string~new('a', 'b') => 93.902
.string~new('a', ) => a
init 3 abc with 2 arguments
abc
```

r[class.string.new.makestring]
An argument whose `MAKESTRING` answers an instance of a subclass of `String`
gives that instance's characters to [`new`](r:class.string.new), as it does to
the other methods that take a string.

```rexx
say .string~new(.mk~new)
say .string~new(.mk~new)~class~id
::class strsub subclass string
::class mk
::method makestring
  return .strsub~new('ss')
```

```output defect=#89
ss
String
```

r[class.string.constants]
The class-side methods `alnum`, `alpha`, `blank`, `cntrl`, `cr`, `digit`,
`graph`, `lower`, `nl`, `null`, `print`, `punct`, `space`, `tab`, `upper` and
`xdigit` each answer a string of characters, a character class. None takes an
argument (`SYNTAX 93.902`), and an instance of `String` does not understand
them (`SYNTAX 97.1`), though a subclass of `String` understands them as the
class does. Only ASCII characters are in any class, and they are written here
by code, in ascending order unless the list says otherwise. `alnum` is the
digits `0` to `9`, then `A` to `Z`, then `a` to `z`; `alpha` is `A` to `Z`,
then `a` to `z`; `blank` is the tab `'09'x` and the blank `'20'x`; `cntrl` is
`'00'x` to `'1F'x` and `'7F'x`; `cr` is `'0D'x`, `nl` is `'0A'x`, `null` is
`'00'x` and `tab` is `'09'x`; `digit` is `0` to `9`, and `xdigit` is `0` to
`9`, `A` to `F`, then `a` to `f`; `graph` is `'21'x` to `'7E'x` and `print` is
`'20'x` to `'7E'x`; `lower` is `a` to `z` and `upper` is `A` to `Z`; `punct`
is the characters of `graph` that are not in `alnum`; and `space` is `'09'x`
to `'0D'x` and `'20'x`. The instance methods `lower` and `upper` are different
methods that [a builtin](r:bif.lower.method) runs.

```rexx
say .string~alnum
say .string~alpha
say .string~digit .string~xdigit
say .string~lower .string~upper
say .string~punct
say .string~cntrl~length .string~cntrl~left(2)~c2x .string~cntrl~right(1)~c2x
say .string~graph~length .string~print~length .string~print~left(2)~c2x
say .string~blank~c2x .string~space~c2x .string~tab~c2x
say .string~cr~c2x .string~nl~c2x .string~null~c2x
say .shout~tab~c2x 'ABC'~lower
call try ".string~nl(1)"
call try "'abc'~nl"
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class shout subclass string
```

```output
0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz
ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz
0123456789 0123456789ABCDEFabcdef
abcdefghijklmnopqrstuvwxyz ABCDEFGHIJKLMNOPQRSTUVWXYZ
!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~
33 0001 7F
94 95 2021
0920 090A0B0C0D20 09
0D 0A 00
09 abc
.string~nl(1) => 93.902
'abc'~nl => 97.1
```

## Methods

### Making other objects and numbers of a string

r[class.string.makestring]
`makeString` answers a string with the characters of the receiver, and takes
no argument (`SYNTAX 93.902`). For an instance of a subclass the answer has
the class `String`. Whether the answer for a string of class `String` is the
receiver itself or another string is not specified ([Deviation
4](r:class.string.deviation.4)).

```rexx
x = .shout~new('abc')
y = x~makeString
say y y~class~id x~class~id 12~makeString '['||''~makeString||']'
call try "'abc'~makeString(1)"
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class shout subclass string
```

```output
abc String SHOUT 12 []
'abc'~makeString(1) => 93.902
```

r[class.string.makearray]
`makeArray` answers a new array of strings, the parts of the receiver, and
takes at most one argument, the separator, whose string value is taken like
that of an argument of [`new`](r:class.string.new) (`SYNTAX 88.909` for an
object with none). Without a separator, or when it is omitted, the parts are
the lines: the text is cut at each line feed, `'0A'x`, and a carriage return
`'0D'x` right before a line feed is not part of the line. A carriage return
alone is part of its line. With a separator the text is cut at each occurrence
of the whole separator, found from the left and without overlap, and the case
of letters matters; the null string as the separator makes each character a
part. The text before the first cut is the first part, so a text that starts
with a separator has an empty first part, and a text that ends with one has no
empty part after it. A receiver that is the null string answers an array with
no part. A subclass instance and a plain string answer the same.

```rexx
a = ('one' || '0A'x || 'two' || '0D0A'x || 'three' || '0D'x)~makeArray
say a~items a~at(1) a~at(2) a~at(3)~c2x
b = ('a' || '0A'x || '0A'x || 'b' || '0A'x)~makeArray
say b~items '['||b~at(2)||']' b~at(3)
c = 'abXYcdXYXYef'~makeArray('XY')
say c~items c~at(1) c~at(2) '['||c~at(3)||']' c~at(4)
d = 'XYab'~makeArray('XY')
say d~items '['||d~at(1)||']' d~at(2)
say 'a,b;c'~makeArray(',;')~items 'abc'~makeArray('')~items
say 'aaa'~makeArray('aa')~items 'AbA'~makeArray('a')~items
say ''~makeArray~items ''~makeArray('x')~items
say .shout~new('a b')~makeArray~items
call try "'abc'~makeArray(.nil)"
call try "'abc'~makeArray(',', 1)"
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class shout subclass string
```

```output
3 one two 74687265650D
3 [] b
4 ab cd [] ef
2 [] ab
1 3
2 1
0 0
1
'abc'~makeArray(.nil) => 88.909
'abc'~makeArray(',', 1) => 93.902
```

r[class.string.hashcode]
`hashCode` answers a string, the hash code of the receiver's characters, and
takes no argument (`SYNTAX 93.902`). Strings with the same characters have the
same hash code, including an instance of a subclass and a plain string, and
the collection classes that hash their indexes use it with `==`, as [the
object rule](r:class.object.hashcode) says. Strings with different characters
may have equal codes. The characters of the answer are not specified.

```rexx
a = 'ab' || 'c'
say (a~hashCode == 'abc'~hashCode) (a~hashCode == 'abd'~hashCode)
say (.shout~new('abc')~hashCode == 'abc'~hashCode)
say ('ABC'~hashCode == a~hashCode)
say (12~hashCode == '12'~hashCode) ('1'~hashCode == '1.0'~hashCode)
signal on syntax name e
say a~hashCode(1)
e: say condition('o')~code
::class shout subclass string
```

```output
1 0
1
0
1 0
93.902
```

### Joining and picking

r[class.string.append]
`append` answers a new string, the characters of the receiver followed by the
string value of its argument. The argument is required (`SYNTAX 93.903`) and a
second raises `93.902`. The string value is found as the joining operators
find it, so an object with no `MAKESTRING` method gives its `string` answer,
and `.nil` gives `The NIL object`: no error is raised for an argument that has
no `MAKESTRING` method. The answer has the class `String` even for an instance
of a subclass.

```rexx
say 'abc'~append('def') 'abc'~append(12) '['||'abc'~append('')']'
say 'abc'~append(.nil) '|' 'abc'~append(.object~new) '|' 'x'~append(.st~new)
say .shout~new('ab')~append('c')~class~id
call try "'abc'~append"
call try "'abc'~append('d', 'e')"
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class shout subclass string
::class st
::method string
  return 'told'
```

```output
abcdef abc12 [abc]
abcThe NIL object | abcan Object | xtold
String
'abc'~append => 93.903
'abc'~append('d', 'e') => 93.902
```

r[class.string.brackets]
The method `[]`, which the subscript syntax `string[n]` and `string[n, len]`
send ([the subscript rule](r:expr.messages.subscript)), answers part of the
receiver. With one argument `n` it answers the character at position `n`, and
with two it answers the `len` characters from position `n`. The part is cut at
the end of the receiver and is not padded, so a position past the end answers
the null string. `n` must be a positive whole number (`SYNTAX 93.924`) and
`len` a non-negative whole number (`93.923`); a value that is not a number at
all raises the same error as one out of range, and the arguments are checked
from left to right. An omitted `n` raises `93.903`, an omitted `len` is as no
`len`, and a third argument raises `93.902`.

```rexx
say 'abcdef'[2] 'abcdef'[2, 3] '['||'abcdef'[7]']' 'abcdef'[5, 9] 12345[2, 3]
say '['||'abcdef'[2, 0]']' 'abcdef'~"[]"(1, ) 'abcdef'[2.0, '2']
call try "'abc'[0]"
call try "'abc'[0, -1]"
call try "'abc'[1, -1]"
call try "'abc'['x']"
call try "'abc'[1, 'x']"
call try "'abc'[1000000000000000000]"
call try "'abc'~'[]'"
call try "'abc'~'[]'(, 1)"
call try "'abc'~'[]'(1, 2, 3)"
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
b bcd [] ef 234
[] a bc
'abc'[0] => 93.924
'abc'[0, -1] => 93.924
'abc'[1, -1] => 93.923
'abc'['x'] => 93.924
'abc'[1, 'x'] => 93.923
'abc'[1000000000000000000] => 93.924
'abc'~'[]' => 93.903
'abc'~'[]'(, 1) => 93.903
'abc'~'[]'(1, 2, 3) => 93.902
```

r[class.string.iif]
`?`, the inline if, answers its first argument when the receiver is `1` and
its second when the receiver is `0`; the receiver must be a [logical
value](r:expr.logical.value) (`SYNTAX 34.901`). Both arguments are required,
and a missing or omitted one raises `SYNTAX 88.901` whichever the receiver is;
a third raises `93.902`. The argument not chosen is not looked at, so it may
be any object. The answer is the argument itself. When the receiver is an
instance of a subclass of `String`, `?` tests its own characters and sends
it nothing ([Deviation 109](r:class.string.deviation.109)).

```rexx
say (1 = 1)~?('yes', 'no') (1 = 2)~?('yes', 'no') '1'~"?"('a', 'b')
say '0'~?(.nil, 'second')~class~id (1 = 1)~?('first', .nil)
call try "'1'~?('only')"
call try "'0'~?(, 'b')"
call try "'1'~?('a', 'b', 'c')"
call try "'2'~?('a', 'b')"
call try "'1'~?"
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
yes no a
String first
'1'~?('only') => 88.901
'0'~?(, 'b') => 88.901
'1'~?('a', 'b', 'c') => 93.902
'2'~?('a', 'b') => 34.901
'1'~? => 88.901
```

### Equality and order

r[class.string.equals]
`equals` answers 1 when the argument has exactly the characters of the
receiver, and 0 otherwise. It is a strict comparison of text: case matters, a
trailing blank matters, and numbers are not compared as numbers, so `1` and
`1.0` are not equal. The argument is required (`SYNTAX 93.903`) and a second
raises `93.902`. The argument's string value is found as the joining operators
find it, so an object with no `MAKESTRING` method is compared by its `string`
text and `.nil` as the text `The NIL object`; no argument is refused.

```rexx
say 'abc'~equals('abc') 'abc'~equals('ABC') 'abc'~equals('abc ')
say '1'~equals('1.0') 12~equals('12') 'abc'~equals(.nil)
say 'an Object'~equals(.object~new)
say .shout~new('abc')~equals('abc') 'abc'~equals(.shout~new('abc'))
call try "'abc'~equals"
call try "'abc'~equals('a', 'b')"
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class shout subclass string
```

```output
1 0 0
0 1 0
1
1 1
'abc'~equals => 93.903
'abc'~equals('a', 'b') => 93.902
```

r[class.string.caselessequals]
`caselessEquals` is [`equals`](r:class.string.equals) with upper and lower
case letters taken as equal. Only the letters `A` to `Z` have a lower-case
form; a character of another script is compared as it is.

```rexx
say 'abc'~caselessEquals('ABC') 'abc'~caselessEquals('ABD')
say 'abc'~caselessEquals('abc ') 'a1'~caselessEquals('A1')
say 'é'~caselessEquals('É') 'abc'~caselessEquals(.nil)
signal on syntax name e
say 'abc'~caselessEquals
e: say condition('o')~code
```

```output
1 0
0 1
0 0
93.903
```

r[class.string.compareto]
`compareTo` compares the receiver with its first argument and answers `0` when
they are equal, `1` when the receiver is the greater and `-1` when it is the
lesser. The comparison is of characters by their codes, unsigned, so case
matters and `'FF'x` is greater than `'00'x`, and a string that is a leading
part of another is the lesser, whatever follows; numbers are not compared as
numbers. The second argument `start` is the position, 1 by default, from which
both strings are compared, and the third `length` is the number of characters
compared of each, by default all that are left. A part that runs past the end
of a string is cut there. The arguments are checked from left to right: the
first, `other`, is required (`SYNTAX 93.903`) and has a string value as the
conversion rule says (`88.909`, as for `.nil`; its `MAKESTRING` only is used,
and then only for an object that is no string); `start` must be a positive
whole number (`93.924`) and `length` a non-negative whole number (`93.923`); a
fourth argument raises `93.902`.

```rexx
say 'abc'~compareTo('abc') 'b'~compareTo('a') 'a'~compareTo('b')
say 'abc'~compareTo('aBc') 'abc'~compareTo('abcd') 'abcd'~compareTo('abc')
say '10'~compareTo('9') ('FF'x)~compareTo('00'x) ''~compareTo('')
say '000abc000'~compareTo('111abc111', 4, 3) 'abc'~compareTo('abd', 1, 2)
say 'abc'~compareTo('abd', 3) 'abc'~compareTo('abd', 4)
say 'abc'~compareTo('abd', 1, 0)
call try "'abc'~compareTo"
call try "'abc'~compareTo(.nil, 0, -1)"
call try "'abc'~compareTo('a', 0, -1)"
call try "'abc'~compareTo('a', 1, -1)"
call try "'abc'~compareTo('a', 1, 1, 1)"
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
0 1 -1
1 -1 1
-1 1 0
0 0
-1 0
0
'abc'~compareTo => 93.903
'abc'~compareTo(.nil, 0, -1) => 88.909
'abc'~compareTo('a', 0, -1) => 93.924
'abc'~compareTo('a', 1, -1) => 93.923
'abc'~compareTo('a', 1, 1, 1) => 93.902
```

r[class.string.caselesscompareto]
`caselessCompareTo` is [`compareTo`](r:class.string.compareto) with upper and
lower case letters taken as equal. A lower-case letter is compared as its
upper-case form, so the case of a letter never decides the answer. A
character that lies between `Z` and `a` in code order, such as `[`, is the
greater of any letter here, though it is the lesser of a lower-case letter
in `compareTo`.

```rexx
say 'abc'~caselessCompareTo('ABC') 'abd'~caselessCompareTo('ABC')
say 'abb'~caselessCompareTo('ABC')
say '000abc000'~caselessCompareTo('111ABC111', 4, 3)
say '['~caselessCompareTo('a') '['~compareTo('a')
signal on syntax name e
say 'abc'~caselessCompareTo('ABD', 0)
e: say condition('o')~code
```

```output
0 1
-1
0
1 -1
93.924
```

r[class.string.caselesscompare]
`caselessCompare` is [`compare`](r:bif.compare.method) with upper and lower
case letters taken as equal: it answers 0 when the strings are the same apart
from case, and otherwise the position of the first character that differs
once case is ignored. The shorter string is padded with the `pad` argument
before they are compared, a blank by default, and the pad is compared without
regard to case too. The pad must be a single character (`SYNTAX 93.922`); the
other is required (`93.903`), and a third argument raises `93.902`.

```rexx
say 'Hello'~caselessCompare('hELLO') 'Hello'~caselessCompare('hELLo!')
say 'Hello'~caselessCompare('hELL', 'O') 'Hello'~caselessCompare('hELL', 'x')
say 'Hello'~caselessCompare('hELLx', 'o') 'Hello'~compare('hello')
call try "'Hello'~caselessCompare"
call try "'Hello'~caselessCompare('a', '**')"
call try "'Hello'~caselessCompare('a', '')"
call try "'Hello'~caselessCompare('a', 'b', 'c')"
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
0 6
0 5
5 1
'Hello'~caselessCompare => 93.903
'Hello'~caselessCompare('a', '**') => 93.922
'Hello'~caselessCompare('a', '') => 93.922
'Hello'~caselessCompare('a', 'b', 'c') => 93.902
```

### Containing and matching

r[class.string.contains]
`contains` answers 1 when its first argument, the needle, occurs in the
receiver, and 0 otherwise; a needle that is the null string is never found.
The search covers the whole receiver unless `start`, a position, and `length`,
a number of characters from it, limit it, and then the needle must lie within
that part. A `start` or a `length` that reaches past the end is cut there. The
needle is required (`SYNTAX 93.903`) and is a string or else uses its
`MAKESTRING` answer (`88.909` for `.nil`), `start` must be a positive whole
number (`93.924`) and `length` a non-negative whole number (`93.923`); the
arguments are checked from left to right, and a fourth argument raises
`93.902`. Case matters.

```rexx
t = '-abcdef-'
say t~contains('ef') t~contains('-', 2) t~contains('-', 2, 6)
say t~contains('-', 2, 7)
say 'abc'~contains('c', 3, 0) 'abc'~contains('c', 9)
say 'abc'~contains('c', 3, 99)
say 'abc'~contains('') 'abc'~contains('B') '12345'~contains(23)
call try "'abc'~contains"
call try "'abc'~contains('b', 0)"
call try "'abc'~contains('b', 1, -1)"
call try "'abc'~contains(.nil, 0, -1)"
call try "'abc'~contains('b', 1, 1, 1)"
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
1 1 0
1
0 0
1
0 0 1
'abc'~contains => 93.903
'abc'~contains('b', 0) => 93.924
'abc'~contains('b', 1, -1) => 93.923
'abc'~contains(.nil, 0, -1) => 88.909
'abc'~contains('b', 1, 1, 1) => 93.902
```

r[class.string.caselesscontains]
`caselessContains` is [`contains`](r:class.string.contains) with upper and
lower case letters taken as equal.

```rexx
say 'abc'~caselessContains('B') 'abc'~caselessContains('B', 3)
say 'abc'~caselessContains('') 'ABC'~caselessContains('bc', 2, 2)
signal on syntax name e
say 'abc'~caselessContains
e: say condition('o')~code
```

```output
1 0
0 1
93.903
```

r[class.string.startswith]
`startsWith` answers 1 when the receiver begins with its argument, and 0
otherwise; the null string is never a prefix, not even of the null string. The
argument is required, and without it `startsWith` raises `SYNTAX 88.901`; a
second raises `93.902`. The argument is a string or else uses its `MAKESTRING`
answer (`88.909` for `.nil`). Case matters.

```rexx
say 'abc'~startsWith('ab') 'abc'~startsWith('AB') 'abc'~startsWith('abc')
say 'abc'~startsWith('abcd') 'abc'~startsWith('') ''~startsWith('')
say '12abc'~startsWith(12) 'abc'~startsWith(12)
call try "'abc'~startsWith"
call try "'abc'~startsWith(.nil)"
call try "'abc'~startsWith('a', 'b')"
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
1 0 1
0 0 0
1 0
'abc'~startsWith => 88.901
'abc'~startsWith(.nil) => 88.909
'abc'~startsWith('a', 'b') => 93.902
```

r[class.string.caselessstartswith]
`caselessStartsWith` is [`startsWith`](r:class.string.startswith) with upper
and lower case letters taken as equal, with the same arguments and errors.

```rexx
say 'abc'~caselessStartsWith('AB') 'abc'~caselessStartsWith('')
signal on syntax name e
say 'abc'~caselessStartsWith
e: say condition('o')~code
```

```output
1 0
88.901
```

r[class.string.endswith]
`endsWith` answers 1 when the receiver ends with its argument, and 0
otherwise; the null string is never a suffix. It takes its argument and raises
its errors as [`startsWith`](r:class.string.startswith) does. Case matters.

```rexx
say 'abc'~endsWith('bc') 'abc'~endsWith('BC') 'abc'~endsWith('abc')
say 'abc'~endsWith('xabc') 'abc'~endsWith('') 'a12'~endsWith(12)
call try "'abc'~endsWith"
call try "'abc'~endsWith(.nil)"
call try "'abc'~endsWith('a', 'b')"
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
1 0 1
0 0 1
'abc'~endsWith => 88.901
'abc'~endsWith(.nil) => 88.909
'abc'~endsWith('a', 'b') => 93.902
```

r[class.string.caselessendswith]
`caselessEndsWith` is [`endsWith`](r:class.string.endswith) with upper and
lower case letters taken as equal, with the same arguments and errors.

```rexx
say 'abc'~caselessEndsWith('BC') 'abc'~caselessEndsWith('')
signal on syntax name e
say 'abc'~caselessEndsWith
e: say condition('o')~code
```

```output
1 0
88.901
```

r[class.string.containsword]
`containsWord` answers 1 when the words of its first argument, the phrase,
occur in the receiver as whole words in the same order, and 0 otherwise. The
phrase and the receiver are cut into [words](r:bif.index.words.split) in the
same way, so the blanks between the words of the phrase need not equal those
of the receiver. A phrase with no words is never found. The search begins at
the word `start`, 1 by default, which is a
[word number](r:bif.index.words.number): a phrase that begins before it is not
found. The phrase is required (`SYNTAX 93.903`) and is a string or else uses
its `MAKESTRING` answer (`88.909`), `start` must be a positive whole number
(`93.924`), and a third argument raises `93.902`. Case matters.

```rexx
t = 'the quick fox'
say t~containsWord('quick') t~containsWord('qui') t~containsWord('QUICK')
say t~containsWord('  quick   fox ') t~containsWord('fox quick')
say t~containsWord('') t~containsWord('quick', 2) t~containsWord('quick', 3)
say t~containsWord('fox', 4)
say ('a' || '0A'x || 'b')~containsWord('b') 'a 12 b'~containsWord(12)
call try "t~containsWord"
call try "t~containsWord('fox', 0)"
call try "t~containsWord(.nil)"
call try "t~containsWord('fox', 1, 1)"
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
1 0 0
1 0
0 1 0
0
0 1
t~containsWord => 93.903
t~containsWord('fox', 0) => 93.924
t~containsWord(.nil) => 88.909
t~containsWord('fox', 1, 1) => 93.902
```

r[class.string.caselesscontainsword]
`caselessContainsWord` is [`containsWord`](r:class.string.containsword) with
upper and lower case letters taken as equal.

```rexx
t = 'the quick fox'
say t~caselessContainsWord('QUICK') t~caselessContainsWord('QUICK', 3)
say t~caselessContainsWord('') t~caselessContainsWord('Quick FOX')
signal on syntax name e
say t~caselessContainsWord
e: say condition('o')~code
```

```output
1 0
0 1
93.903
```

r[class.string.match]
`match` tests whether a part of its second argument occurs at a given place in
the receiver. The arguments are `start`, the position in the receiver,
`other`, the string, `n`, the position in `other` where the part begins (1 by
default), and `length`, the number of characters in the part (by default all
that are left from `n`). The answer is 1 when the part is not empty, lies
wholly within `other` and wholly within the receiver counted from `start`, and
has the characters that the receiver has there, and 0 otherwise. A part that
is empty, begins after the end of `other`, or runs past the end of either
string is not an error: the answer is 0. The arguments are checked from left
to right: `start` and `n` must be positive whole numbers (`SYNTAX 93.924`),
`other` is required, as `start` is (`93.903`), and is a string or else uses
its `MAKESTRING` answer (`88.909`), and `length` must be a non-negative whole
number (`93.923`); a fifth argument raises `93.902`. Case matters.

```rexx
t = 'Wednesday'
say t~match(4, 'nes') t~match(4, 'NES') t~match(4, 'honest', 3, 3)
say t~match(4, 'nesting', 1, 3) t~match(4, 'nesting', 1, 4)
say t~match(4, 'nesday')
say t~match(9, 'y') t~match(9, 'yz') t~match(1, '') t~match(1, 'W', 2)
say t~match(1, 'W', 1, 0) t~match(1, 'W', 1, 2) t~match(1, 'W', , 1)
call try "t~match"
call try "t~match(1)"
call try "t~match(0, 'W')"
call try "t~match(1, .nil, 0, -1)"
call try "t~match(1, 'W', 0, -1)"
call try "t~match(1, 'W', 1, -1)"
call try "t~match(1, 'W', 1, 1, 1)"
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
1 0 1
1 0
1
1 0 0 0
0 0 1
t~match => 93.903
t~match(1) => 93.903
t~match(0, 'W') => 93.924
t~match(1, .nil, 0, -1) => 88.909
t~match(1, 'W', 0, -1) => 93.924
t~match(1, 'W', 1, -1) => 93.923
t~match(1, 'W', 1, 1, 1) => 93.902
```

r[class.string.caselessmatch]
`caselessMatch` is [`match`](r:class.string.match) with upper and lower case
letters taken as equal.

```rexx
t = 'Wednesday'
say t~caselessMatch(1, 'wed') t~caselessMatch(4, 'HONEST', 3, 3)
say t~caselessMatch(1, '') t~match(1, 'wed')
signal on syntax name e
say t~caselessMatch(0, 'W')
e: say condition('o')~code
```

```output
1 1
0 0
93.924
```

r[class.string.matchchar]
`matchChar` answers 1 when the character of the receiver at position `n` is
one of the characters of its second argument, the set, and 0 otherwise: when
it is not in the set, when the set is the null string, and when `n` is past
the end of the receiver. Both arguments are required (`SYNTAX 93.903`), `n`
must be a positive whole number (`93.924`) and is checked first, the set is a
string or else uses its `MAKESTRING` answer (`88.909`), and a third argument
raises `93.902`. Case matters.

```rexx
t = 'Saturday'
say t~matchChar(1, 'xyzS') t~matchChar(2, 'A') t~matchChar(2, 'a')
say t~matchChar(3, '')
say t~matchChar(9, 'y') t~matchChar(8, 'y') '12'~matchChar(2, 12)
call try "t~matchChar(1)"
call try "t~matchChar(0, .nil)"
call try "t~matchChar(1, .nil)"
call try "t~matchChar(1, 'a', 'b')"
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
1 0 1
0
0 1 1
t~matchChar(1) => 93.903
t~matchChar(0, .nil) => 93.924
t~matchChar(1, .nil) => 88.909
t~matchChar(1, 'a', 'b') => 93.902
```

r[class.string.caselessmatchchar]
`caselessMatchChar` is [`matchChar`](r:class.string.matchchar) with upper and
lower case letters taken as equal.

```rexx
t = 'Saturday'
say t~caselessMatchChar(2, 'A') t~caselessMatchChar(1, 's')
say t~caselessMatchChar(9, 's')
signal on syntax name e
say t~caselessMatchChar(0, 's')
e: say condition('o')~code
```

```output
1 1
0
93.924
```

### Searching without regard to case

r[class.string.caselesspos]
`caselessPos` is [`pos`](r:bif.pos.method) with upper and lower case letters
taken as equal: it takes the same needle, `start` and `length`, defaults and
checks, answers the position of the first occurrence in the receiver, and 0
when there is none. The receiver is the haystack.

```rexx
say 'Hello'~caselessPos('L') 'Hello'~caselessPos('L', 4)
say 'Hello'~caselessPos('L', 5)
say 'Hello'~caselessPos('L', 1, 3) 'Hello'~caselessPos('L', 1, 2)
say 'Hello'~caselessPos('') 'Hello'~pos('L')
signal on syntax name e
say 'Hello'~caselessPos('L', 0)
e: say condition('o')~code
```

```output
3 4
0
3 0
0 0
93.924
```

r[class.string.caselesslastpos]
`caselessLastPos` is [`lastPos`](r:bif.lastpos.method) with upper and lower
case letters taken as equal, with the same arguments, defaults and checks. It
answers the position of the last occurrence, as `lastPos` does.

```rexx
say 'Hello'~caselessLastPos('L') 'Hello'~caselessLastPos('L', 3)
say 'Hello'~caselessLastPos('L', 3, 2) 'Hello'~caselessLastPos('')
say 'Hello'~lastPos('L')
signal on syntax name e
say 'Hello'~caselessLastPos('L', 1, -1)
e: say condition('o')~code
```

```output
4 3
3 0
0
93.923
```

r[class.string.caselesscountstr]
`caselessCountStr` is [`countStr`](r:bif.countstr.method) with upper and lower
case letters taken as equal: it answers how many times its argument occurs in
the receiver without overlap. It takes one argument (`SYNTAX 93.903` if
missing, `93.902` if two).

```rexx
say 'Hello'~caselessCountStr('L') 'Hello'~caselessCountStr('LL')
say 'aaa'~caselessCountStr('AA') 'Hello'~caselessCountStr('')
say 'Hello'~countStr('L')
call try "'Hello'~caselessCountStr"
call try "'Hello'~caselessCountStr('l', 1)"
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
2 1
1 0
0
'Hello'~caselessCountStr => 93.903
'Hello'~caselessCountStr('l', 1) => 93.902
```

r[class.string.caselesschangestr]
`caselessChangeStr` is [`changeStr`](r:bif.changestr.method) with upper and
lower case letters taken as equal in the search: it answers the receiver with
occurrences of the needle replaced by the new string, the first `count` of
them when `count` is given. The replacement is put in as written, whatever
case the text it replaces had. A needle that is the null string changes
nothing. `count` must not be negative (`SYNTAX 93.906`); the arguments are
checked from left to right, the first two are required (`93.903`), and a
fourth raises `93.902`.

```rexx
say 'Hello'~caselessChangeStr('L', 'x') 'Hello'~caselessChangeStr('L', 'x', 1)
say 'Hello'~caselessChangeStr('L', 'x', 0) 'Hello'~caselessChangeStr('', 'x')
say 'Hello'~caselessChangeStr('L', '') 'Hello'~changeStr('L', 'x')
call try "'Hello'~caselessChangeStr('L')"
call try "'Hello'~caselessChangeStr('L', 'x', -1)"
call try "'Hello'~caselessChangeStr(.nil, 'x', -1)"
call try "'Hello'~caselessChangeStr('L', 'x', 1, 1)"
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
Hexxo Hexlo
Hello Hello
Heo Hello
'Hello'~caselessChangeStr('L') => 93.903
'Hello'~caselessChangeStr('L', 'x', -1) => 93.906
'Hello'~caselessChangeStr(.nil, 'x', -1) => 88.909
'Hello'~caselessChangeStr('L', 'x', 1, 1) => 93.902
```

r[class.string.caselessabbrev]
`caselessAbbrev` is [`abbrev`](r:bif.abbrev.method) with upper and lower case
letters taken as equal: it answers 1 when its first argument, the information,
is the beginning of the receiver and is at least `length` characters long, 0
otherwise, with the defaults and checks of `abbrev`. The receiver is the full
word.

```rexx
say 'Hello'~caselessAbbrev('HEL') 'Hello'~caselessAbbrev('HEL', 4)
say 'Hello'~caselessAbbrev('HEL', 3) 'Hello'~caselessAbbrev('HELLOO')
say 'Hello'~caselessAbbrev('') 'Hello'~caselessAbbrev('', 1)
say 'Hello'~abbrev('HEL')
signal on syntax name e
say 'Hello'~caselessAbbrev('HEL', -1)
e: say condition('o')~code
```

```output
1 0
1 0
1 0
0
93.923
```

r[class.string.caselesswordpos]
`caselessWordPos` is [`wordPos`](r:bif.wordpos.method) with upper and lower
case letters taken as equal: it answers the number of the word of the receiver
where its first argument, the phrase, begins, or 0, with the default and
checks of `wordPos`.

```rexx
say 'a b c'~caselessWordPos('B') 'a b c'~caselessWordPos('B C')
say 'a b c'~caselessWordPos('B', 3) 'a b c'~caselessWordPos('')
say 'a b c'~wordPos('B')
signal on syntax name e
say 'a b c'~caselessWordPos('B', 0)
e: say condition('o')~code
```

```output
2 2
0 0
0
93.924
```

### Parts of a string

r[class.string.subchar]
`subChar` answers the character at position `n` as a string of one character,
and the null string when `n` is past the end of the receiver. `n` is required
(`SYNTAX 93.903`) and must be a positive whole number (`93.924`), and a second
argument raises `93.902`. The answer has the class `String`, also for the
instance of a subclass.

```rexx
say 'Hello'~subChar(1) 'Hello'~subChar(5) '['||'Hello'~subChar(6)']'
say .shout~new('Hello')~subChar(2)~class~id ''~subChar(1)~length
call try "'Hello'~subChar"
call try "'Hello'~subChar(0)"
call try "'Hello'~subChar('x')"
call try "'Hello'~subChar(1, 1)"
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class shout subclass string
```

```output
H o []
String 0
'Hello'~subChar => 93.903
'Hello'~subChar(0) => 93.924
'Hello'~subChar('x') => 93.924
'Hello'~subChar(1, 1) => 93.902
```

r[class.string.subwords]
`subWords` answers a new array of strings, the words of the receiver from the
word `n`, 1 by default, up to `count` words, by default all that are left.
Words are those of [the word rule](r:bif.index.words.split): they are
separated by blanks and tabs, and the white space around them is not part of
them. A `n` past the last word, and a `count` of 0, answer an empty array;
a `count` larger than the number of words left is cut. `n` must be a positive
whole number (`SYNTAX 93.924`) and `count` a non-negative whole number
(`93.923`), checked in that order; a third argument raises `93.902`. The
items are strings of class `String`.

```rexx
t = 'a b c d'
say t~subWords~items t~subWords(2)~items t~subWords(2)~at(1)
say t~subWords(2, 2)~at(2)
say t~subWords(2, 0)~items t~subWords(5)~items t~subWords(2, 99)~items
say '  a   b  '~subWords~items '  a   b  '~subWords~at(1) ''~subWords~items
say t~subWords(,2)~items t~subWords(2)~at(1)~class~id
call try "t~subWords(0)"
call try "t~subWords(1, -1)"
call try "t~subWords('x', 'y')"
call try "t~subWords(1, 1, 1)"
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
4 3 b
c
0 0 3
2 a 0
2 String
t~subWords(0) => 93.924
t~subWords(1, -1) => 93.923
t~subWords('x', 'y') => 93.924
t~subWords(1, 1, 1) => 93.902
```

r[class.string.replaceat]
`replaceAt` answers a new string in which `length` characters of the receiver
from position `position` are replaced by the first argument, `newstr`. The
default `length` is the length of `newstr`, and a part that runs past the end
of the receiver is cut there. When `position` is more than one past the end of
the receiver, the gap is filled with the pad, a blank by default. `newstr` and
`position` are required (`SYNTAX 93.903`), and `newstr` is a string or else
uses its `MAKESTRING` answer (`88.909`), `position` must be a positive whole
number (`93.924`), `length` a non-negative whole number (`93.923`) and the pad
a single character (`93.922`); the arguments are checked from left to right,
and a fifth raises `93.902`. A `length` of 0 inserts `newstr`.

```rexx
s = 'abcdef'
say s~replaceAt('XY', 3) s~replaceAt('XY', 3, 1) s~replaceAt('XY', 3, 0)
say s~replaceAt('XY', 3, 3) s~replaceAt('XY', 3, 9) s~replaceAt('XY', 1, 2)
say s~replaceAt('XY', 7) s~replaceAt('', 3, 2) s~replaceAt(12, 3)
say s~replaceAt('XY', 9)'|' s~replaceAt('XY', 9, 1, '*')
call try "s~replaceAt"
call try "s~replaceAt('XY')"
call try "s~replaceAt(.nil, 0, -1, '**')"
call try "s~replaceAt('X', 0, -1, '**')"
call try "s~replaceAt('X', 1, -1, '**')"
call try "s~replaceAt('X', 1, 1, '**')"
call try "s~replaceAt('X', 1, 1, '')"
call try "s~replaceAt('X', 1, 1, '*', 1)"
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
abXYef abXYdef abXYcdef
abXYf abXY XYcdef
abcdefXY abef ab12ef
abcdef  XY| abcdef**XY
s~replaceAt => 93.903
s~replaceAt('XY') => 93.903
s~replaceAt(.nil, 0, -1, '**') => 88.909
s~replaceAt('X', 0, -1, '**') => 93.924
s~replaceAt('X', 1, -1, '**') => 93.923
s~replaceAt('X', 1, 1, '**') => 93.922
s~replaceAt('X', 1, 1, '') => 93.922
s~replaceAt('X', 1, 1, '*', 1) => 93.902
```

### Numbers

r[class.string.ceiling]
`ceiling` answers the smallest whole number that is not less than the
receiver, which must be a number (`SYNTAX 93.943`). The receiver is first
rounded to `NUMERIC DIGITS` digits, as the addition of 0 does, and the answer
is written without an exponent and without a decimal point. A result with no
non-zero digit is `0`, never `-0`. It takes no argument (`93.902`).

```rexx
say 12.3~ceiling '-12.3'~ceiling '-0.1'~ceiling '0.1'~ceiling
say 2~ceiling '1.2e3'~ceiling
say '-0.00'~ceiling ' 5.5 '~ceiling '99999999999999999999.9'~ceiling
numeric digits 3
say '12355'~ceiling '12301'~ceiling '1e10'~ceiling '-0.0004'~ceiling
call try "'abc'~ceiling"
call try "''~ceiling"
call try "2~ceiling(1)"
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
13 -12 0 1
2 1200
0 6 100000000000000000000
12400 12300 10000000000 0
'abc'~ceiling => 93.943
''~ceiling => 93.943
2~ceiling(1) => 93.902
```

r[class.string.floor]
`floor` answers the largest whole number that is not greater than the
receiver. It is otherwise as [`ceiling`](r:class.string.ceiling) is: a
receiver that is a number (`SYNTAX 93.943`), rounded first to `NUMERIC DIGITS`
digits, no exponent, no `-0`, and no argument (`93.902`).

```rexx
say 12.3~floor '-12.3'~floor '-0.1'~floor '0.1'~floor 2~floor '1e3'~floor
say '-0'~floor '-.5'~floor '123456789012'~floor '99999999999999999999.9'~floor
numeric digits 3
say '12355'~floor '1e10'~floor '1e-10'~floor '-0.0004'~floor
call try "'abc'~floor"
call try "2~floor(1)"
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
12 -13 -1 0 2 1000
0 -1 123456789000 100000000000000000000
12400 10000000000 0 -1
'abc'~floor => 93.943
2~floor(1) => 93.902
```

r[class.string.round]
`round` answers the whole number nearest to the receiver; a half is rounded
away from zero. It is otherwise as [`ceiling`](r:class.string.ceiling) is:
a receiver that is a number (`SYNTAX 93.943`), rounded first to `NUMERIC
DIGITS` digits, no exponent, no `-0`, and no argument (`93.902`).

```rexx
say 2.4~round 2.5~round '-2.4'~round '-2.5'~round
say '-0.1'~round '.5'~round '-.5'~round
say '123.456'~round '1e3'~round '0.49999999999'~round
numeric digits 3
say '1234.5'~round '-1234.5'~round '5e-1'~round
call try "'abc'~round"
call try "2~round(1)"
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
2 3 -2 -3
0 1 -1
123 1000 1
1230 -1230 1
'abc'~round => 93.943
2~round(1) => 93.902
```

r[class.string.modulo]
`modulo` answers the remainder of the receiver divided by its argument `n`.
The answer is never negative and is less than `n`, so for a negative receiver
`(-7)~modulo(4)` is 1. The receiver must be a number (`SYNTAX 93.943`) and a
whole number (`93.940`): a number written with no period and no exponent is a
whole number whatever its length, and any other number must be exactly whole,
without rounding, and its value must have at most `NUMERIC DIGITS` digits. The
argument is required (`93.903`) and must be a positive whole number (`93.907`)
in the same sense, checked after the receiver; a second argument raises
`93.902`. An argument that is not a string is converted to a number as
[the conversion rule](r:conv.string.object) says, as the other numeric
arguments are. The remainder is found as
[the `//` operator](r:num.arithmetic.integer) finds it, and written as that
operator writes it, so a quotient wider than `NUMERIC DIGITS` raises
`SYNTAX 26.12`, for a negative receiver too.

```rexx
say 9~modulo(4) (-7)~modulo(4) 0~modulo(5) '1e2'~modulo(7) 12~modulo(' 5 ')
say '1.0'~modulo(5) 9~modulo(5.0) '11.0'~modulo(5) (9 // 4)
call try "10~modulo"
call try "10~modulo(0)"
call try "10~modulo(-3)"
call try "10~modulo(3.5)"
call try "10~modulo('x')"
call try "10.5~modulo(3)"
call try "10.5~modulo(0)"
call try "'x'~modulo('y')"
call try "'1E+30'~modulo(7)"
call try "10~modulo(3, 4)"
numeric digits 3
say '1000'~modulo(7) '1.0000'~modulo(7)
call try "'1E+3'~modulo(7)"
call try "'7000'~modulo(7)"
call try "'-7006'~modulo(7)"
call try "'1000000'~modulo('1E+3')"
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
1 1 0 2 2
1.0 4.0 1.0 1
10~modulo => 93.903
10~modulo(0) => 93.907
10~modulo(-3) => 93.907
10~modulo(3.5) => 93.907
10~modulo('x') => 93.907
10.5~modulo(3) => 93.940
10.5~modulo(0) => 93.940
'x'~modulo('y') => 93.943
'1E+30'~modulo(7) => 93.940
10~modulo(3, 4) => 93.902
6 1.00
'1E+3'~modulo(7) => 93.940
'7000'~modulo(7) => 26.12
'-7006'~modulo(7) => 26.12
'1000000'~modulo('1E+3') => 93.907
```

```rexx
say '17'~modulo(.five~new) '17'~modulo(.six~new)
::class five
::method makestring
  return 5
::class six
::method string
  return 6
```

```output defect=#138
2 5
```

### Base 64

r[class.string.encodebase64]
`encodeBase64` answers the receiver written in base 64: groups of three bytes,
taken as the receiver's characters, become four characters of the alphabet `A`
to `Z`, `a` to `z`, `0` to `9`, `+` and `/`, and a final group of one or two
bytes is completed with `=` signs to four characters. The null string gives
the null string. It takes no argument (`SYNTAX 93.902`).

```rexx
say 'Man'~encodeBase64 'Ma'~encodeBase64 'M'~encodeBase64
say '['||''~encodeBase64']'
say 'Hello, World!'~encodeBase64 ('FF00FE'x)~encodeBase64 12~encodeBase64
signal on syntax name e
say 'Man'~encodeBase64(1)
e: say condition('o')~code
```

```output
TWFu TWE= TQ==
[]
SGVsbG8sIFdvcmxkIQ== /wD+ MTI=
93.902
```

r[class.string.decodebase64]
`decodeBase64` undoes [`encodeBase64`](r:class.string.encodebase64): it
answers the bytes that the receiver, a text in base 64, stands for, and the
null string for the null string. The text must have a length that is a
multiple of 4, characters of that alphabet, and `=` signs only at the end, one
or two of them, to complete the last group; the bits of the last character
that the group does not need are ignored, so `'TR=='` decodes as `'TQ=='`
does. A blank, a line end, any other character, or `=` in another place raises
`SYNTAX 93.962`. It takes no argument (`93.902`).

```rexx
say 'TWFu'~decodeBase64 'TWE='~decodeBase64 'TQ=='~decodeBase64
say '['||''~decodeBase64']' 'TWFuTQ=='~decodeBase64 '/+8='~decodeBase64~c2x
say 'TR=='~decodeBase64~c2x 'TWF='~decodeBase64~c2x
say 'abc'~encodeBase64~decodeBase64
call try "'TWE'~decodeBase64"
call try "'TWE=='~decodeBase64"
call try "'TW=u'~decodeBase64"
call try "'TWFu TWFu'~decodeBase64"
call try "' TWFu'~decodeBase64"
call try "'TWFu='~decodeBase64"
call try "'TWFu'~decodeBase64(1)"
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
Man Ma M
[] ManM FFEF
4D 4D61
abc
'TWE'~decodeBase64 => 93.962
'TWE=='~decodeBase64 => 93.962
'TW=u'~decodeBase64 => 93.962
'TWFu TWFu'~decodeBase64 => 93.962
' TWFu'~decodeBase64 => 93.962
'TWFu='~decodeBase64 => 93.962
'TWFu'~decodeBase64(1) => 93.902
```

### Methods that a builtin function runs

r[class.string.builtins]
A built-in function that [runs as a String method](r:conv.method) is also a
method of `String` with the name of the function, and the behavior of the
method is stated on the page of the function: what it accepts, its defaults,
its results and the checks it makes. The pages say it as a function call and
show the method form beside it. The methods are
[`abbrev`](r:bif.abbrev.method), [`abs`](r:bif.abs.method),
[`b2x`](r:bif.b2x.method), [`bitAnd`](r:bif.bitand.method),
[`bitOr`](r:bif.bitor.method), [`bitXor`](r:bif.bitxor.method),
[`c2d`](r:bif.c2d.method), [`c2x`](r:bif.c2x.method),
[`center`](r:bif.center.method), [`centre`](r:bif.center.centre),
[`changeStr`](r:bif.changestr.method), [`compare`](r:bif.compare.method),
[`copies`](r:bif.copies.method), [`countStr`](r:bif.countstr.method),
[`d2c`](r:bif.d2c.method), [`d2x`](r:bif.d2x.method),
[`dataType`](r:bif.datatype.method), [`delStr`](r:bif.delstr.method),
[`delWord`](r:bif.delword.method), [`format`](r:bif.format.method),
[`insert`](r:bif.insert.method), [`lastPos`](r:bif.lastpos.method),
[`left`](r:bif.left.method), [`length`](r:bif.length.method),
[`lower`](r:bif.lower.method), [`max`](r:bif.max.method),
[`min`](r:bif.min.method), [`overlay`](r:bif.overlay.method),
[`pos`](r:bif.pos.method), [`reverse`](r:bif.reverse.method),
[`right`](r:bif.right.method), [`sign`](r:bif.sign.method),
[`space`](r:bif.space.method), [`strip`](r:bif.strip.method),
[`subWord`](r:bif.subword.method), [`substr`](r:bif.substr.method),
[`translate`](r:bif.translate.method), [`trunc`](r:bif.trunc.method),
[`upper`](r:bif.upper.method), [`verify`](r:bif.verify.method),
[`word`](r:bif.word.method), [`wordIndex`](r:bif.wordindex.method),
[`wordLength`](r:bif.wordlength.method), [`wordPos`](r:bif.wordpos.method),
[`words`](r:bif.words.method), [`x2b`](r:bif.x2b.method),
[`x2c`](r:bif.x2c.method) and [`x2d`](r:bif.x2d.method), each with the rule
that opens its page. A method of this kind needs no rule of its own here, but
the rules that follow state what is the same for all of them when they are
sent as messages: their errors ([the message
rule](r:class.string.builtins.errors)) and their receiver ([the receiver
rule](r:class.string.builtins.receiver)). Functions that a program calls but a
string does not understand as a message, such as `arg` and `xrange`, are not
among them: their pages say so.

```rexx
say 'Hello'~left(3) 'Hello'~pos('l') 'Hello'~reverse 'a b c'~words
say 'abc'~centre(7, '.') '255'~d2x 'ff'~x2d 'abc'~substr(2, 1) 'ABC'~lower
say '3.14159'~format(2, 2) '-4.7'~trunc '4'~max(9, 2) '-4'~sign 'abc'~length
say 'a-b'~translate('+', '-') 'hello world'~wordIndex(2) 'ab'~copies(3)
```

```output
Hel 3 olleH 3
..abc.. FF 255 b abc
 3.14 -4 9 -1 3
a+b 7 ababab
```

r[class.string.builtins.errors]
A method of the list is checked as the method itself checks it, argument by
argument and from left to right, without the earlier check that a function
call makes. The function form first checks the form of its arguments, as
[the order rule](r:conv.check.order) says, and raises the codes of
[the whole-number](r:conv.whole.form), [pad](r:conv.pad) and
[argument count](r:bif.index.call.count) rules. In the message form an
argument that fails its form raises the code of its range check. The message
form raises these errors: for a position that is not a positive whole number,
`SYNTAX 93.924` whatever the value is, `abc` or 0 or 1.5; for a length that is
not a non-negative whole number, `93.923`; for a pad that is not a single
character, `93.922`; for a missing argument, or an omitted one that is
followed by another, `93.903`; and for too many arguments, `93.902`, as
[the argument rules](r:class.index.args.count) say. A count or a number of
copies that is not a non-negative whole number raises `93.906`, and the other
arguments raise the code that the page of the builtin gives. Because the
message form checks one argument at a time, a bad position before a length
that is not a number is the error raised, where the function form raises 40.12
for the length first.

```rexx
call try "'abc'~substr('x')"
call try "substr('abc', 'x')"
call try "'abc'~substr(0, 'x')"
call try "substr('abc', 0, 'x')"
call try "'abc'~left('x')"
call try "left('abc', 'x')"
call try "'abc'~left(1, '**')"
call try "left('abc', 1, '**')"
call try "'abc'~left"
call try "left('abc')"
call try "'abc'~left(, 2)"
call try "left('abc', , 2)"
call try "'abc'~left(1, 'x', 3)"
call try "left('abc', 1, 'x', 3)"
call try "'abc'~copies('x')"
call try "'abc'~word(1.5)"
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
'abc'~substr('x') => 93.924
substr('abc', 'x') => 40.12
'abc'~substr(0, 'x') => 93.924
substr('abc', 0, 'x') => 40.12
'abc'~left('x') => 93.923
left('abc', 'x') => 40.12
'abc'~left(1, '**') => 93.922
left('abc', 1, '**') => 40.23
'abc'~left => 93.903
left('abc') => 40.3
'abc'~left(, 2) => 93.903
left('abc', , 2) => 40.5
'abc'~left(1, 'x', 3) => 93.902
left('abc', 1, 'x', 3) => 40.4
'abc'~copies('x') => 93.906
'abc'~word(1.5) => 93.924
```

r[class.string.builtins.receiver]
When the receiver or an argument of such a method is an instance of a
subclass of `String`, the method works on the characters of the instance
itself, and answers a new string of class `String`. This holds for the message
form and for the function form, and for the methods that read a number, such
as `abs` and `max`, and `dataType` as well. The characters it reads are never
the answer of the instance's `MAKESTRING` or `STRING` method, even when the
subclass declares one, and neither is sent to the instance
([Deviation 109](r:class.string.deviation.109) has the example for the
arguments and the function form).

```rexx
x = .over~new('Hello')
say x~left(3) x~length x~pos('l') x~c2x~left(4) x~reverse x~left(3)~class~id
say x~makeString (x || '!')
::class over subclass string
::method makestring
  return 'MADE'
```

```output
Hel 5 3 4865 olleH String
MADE Hello!
```

### The operators

r[class.string.operators]
Each operator is also a method of `String` with the name of the operator, as
[the operator rule](r:expr.messages.operator) says, and what it does belongs
to the page of its operator: the arithmetic operators `+`, `-`, `*`, `/`, `%`,
`//` and `**` ([infix](r:expr.arithmetic.infix),
[prefix](r:expr.arithmetic.prefix) and
[exponent](r:expr.arithmetic.exponent)); the comparisons `=`, `\=`, `<>`,
`><`, `<`, `>`, `<=`, `>=`, `\<` and `\>` ([the normal
comparisons](r:expr.comparison.kind)) and `==`, `\==`, `<<`, `>>`, `<<=`,
`>>=`, `\<<` and `\>>` ([the strict ones](r:expr.comparison.strict)); the
logical operators `&`, `|`, `&&` and `\` ([the logical
rules](r:expr.logical.operators)); and the three joining methods `||`, the
blank and the null string, which is the abuttal ([the concatenation
rules](r:expr.concatenation.operator)). How many arguments each takes when it
is sent as a message, and the errors for too few or too many, are [the message
rule](r:expr.messages.string). An argument that is not a string is converted
as [the conversion rule](r:conv.string.object) says.

```rexx
say '3'~"+"(4) '3'~"-" '3'~"*"(4) '3'~"/"(4)
say '3'~"%"(2) '3'~"//"(2) '3'~"**"(2)
say '3'~"="(3.0) '3'~"=="(3.0) 'abc'~"<"('abd') 'abc'~">>"('ab') 'a'~"\="('a')
say '1'~"&"('1') '1'~"|"('0') '1'~"&&"('1') '1'~"\"
say 'ab'~"||"('c') 'ab'~" "('c') 'ab'~""('c')
call try "'3'~'+'(4, 5)"
call try "'3'~'*'"
call try "'3'~'**'(.5)"
call try "'3'~'/'(0)"
call try "'x'~'+'(1)"
call try "'2'~'&'('1')"
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
7 -3 12 0.75
1 1 9
1 0 1 1 0
1 1 0 0
abc ab c abc
'3'~'+'(4, 5) => 93.902
'3'~'*' => 93.903
'3'~'**'(.5) => 26.8
'3'~'/'(0) => 42.3
'x'~'+'(1) => 41.1
'2'~'&'('1') => 34.901
```

r[class.string.operators.receiver]
When the receiver or the argument of an operator method is an instance of a
subclass of `String`, the method uses the characters of the instance itself,
and not the answer of its `MAKESTRING` or `STRING` method, as
[the receiver rule](r:class.string.builtins.receiver) says of the builtin
methods. This holds for every operator: the joining methods and the strict
comparisons, which read a string, the arithmetic operators and the normal
comparisons, which read a number, and the logical operators, which read a
logical value. The joining methods answer a new string of class `String`.
[Deviation 109](r:class.string.deviation.109) has the example for the
arguments and the numbers.

```rexx
x = .over~new('Hello')
say x~"||"('!') x~"=="('Hello') x~"=="('MADE')
say x~" "('!') x~""('!') (x || '!')~class~id (x == 'Hello')
::class over subclass string
::method makestring
  return 'MADE'
```

```output
Hello! 1 0
Hello ! Hello! String 1
```

## Deviations

r[class.string.deviation.4]
Whether two strings with the same characters that were built separately are
one object, whether a method that leaves its receiver as it was, such as
`makeString`, `strip` of a string with nothing to strip, or `copy`, answers
the receiver or a new string, and whether a number and a string with its
digits are one object, are not specified, and show only through `identityHash`
and the identity collections ([Deviation 4](../deviations.md)). The characters
and the value comparisons are specified, and the hash codes of equal strings
are equal. ooRexx answers a new object in many cases where interrexx answers
the existing one.

```rexx
a = 'ab' || 'c'
b = 'a' || 'bc'
say (a == b) (a~hashCode == b~hashCode) (a~makeString == a) (a~copy == a)
say (12~makeString == '12') (12~hashCode == '12'~hashCode)
say ('abc'~strip == 'abc') ('abc'~substr(1) == 'abc')
```

```output
1 1 1 1
1 1
1 1
```

r[class.string.deviation.36]
`objectName=` sent to a string that is no instance of a subclass and not
enhanced is refused, and the program ends with exit code 120, as [the chapter
rule](r:class.index.deviation.36) says: a number is such a string, and so is a
string that `new` made. An instance of a subclass takes a name, and so does a
string that `enhanced` made. ooRexx gives the name to the string, and an equal
string elsewhere can show it.

```rexx
s = .sub~new('abc')
s~objectName = 'named'
say s~objectName s
say 'before'
t = .string~new('abc')
t~objectName = 'named'
say 'after'
::class sub subclass string
```

```output rc=120
named abc
before
```

r[class.string.deviation.109]
An instance of a subclass of `String` is read by its own characters wherever a
string, a number, a logical value or a whole number is needed, as the receiver
and as an argument, in the message form, in the function form and in the
operators; `MAKESTRING` and `STRING` are not sent to it for that
([Deviation 109](../deviations.md)). In the example the subclass declares both
methods and counts the sends: the instance `n` is the number 2, `b` is the
logical value 1 and `s` is the string `ab`, and no send is made. ooRexx reads
such an instance through its `MAKESTRING` method when it is an argument, of a
method, an operator or a builtin function, and when it is the receiver of the
numeric methods and operators, the logical operators, `dataType` and `?`, and
in `new`; as the receiver of the other methods, of the strict comparisons and
of the joining operators it reads the instance's own characters, so one
instance can answer differently to `=` and to `==`.

```rexx
.local~sent = 0
n = .num~new('2')
b = .num~new('1')
s = .num~new('ab')
say 'abcdef'~left(n) left('abcdef', n) 'abcdef'[n] 'a b c'~word(n)
say 'x' || s 'xaby'~pos(s) pos(s, 'xaby') compare(s, 'ab') (s == 'ab')
say 1 + n (n + 1) (n * n) ('2' = n) abs(n) max(n, 1)
say ('1' & b) (b | '0') (\b) b~?('y', 'n') n~dataType datatype(n)
say .string~new(s) .string~new(s)~class~id s
say 'sends' .local~sent
::class num subclass string
::method makestring
  .local~sent = .local~sent + 1
  return 7
::method string
  .local~sent = .local~sent + 1
  return 'str'
```

```output defect=#140
ab ab b b
xab 2 2 0 1
3 3 4 1 2 2
1 1 0 y NUM NUM
ab String ab
sends 0
```
