---
element: "Conventions"
kind: concept
ansi: "9.2.1"
rexxref: "Functions > Built-in Functions"
deviations: [109]
---

# Conventions

The rules on this page hold on every page that links to them. A builtin
page states what its arguments are (a string, a position, a length, a pad)
and links here for what that means, instead of restating it.

## Strings and characters

r[conv.string.chars]
A string is a finite sequence of characters, and a character is one byte:
any of the 256 values from `'00'x` to `'FF'x`. The length of a string is
the number of bytes in it. No rule in this book decodes a string in UTF-8
or any other encoding, so a character that such an encoding writes as two
bytes is two characters here.

```rexx
say length('c3a9'x)
say c2x(substr('c3a9'x, 2))
```

```output
2
A9
```

r[conv.string.null]
The null string is the string of length 0, written `''`. A builtin makes no
check of an argument its page calls a string: no string there raises an
error, so the null string and a string of any bytes are as valid as any
other.

```rexx
say length(upper(''))
say c2x(upper('61007a'x))
```

```output
0
41005A
```

r[conv.string.object]
An argument that a page calls a string and that is an object other than a
string is replaced by a string, and the builtin works on that string. The
replacement is found by sending the object the message `REQUEST` with the
argument `STRING`, and nothing else is sent first. When the answer is a string,
it is the replacement. When the answer is `.nil` or another object that is not
a string, the replacement is the answer of the object's public
[`STRING`](r:class.object.string) method, and when that answer has no string
value either, [the answer rule](r:conv.string.answer) gives the replacement. The
[`REQUEST`](r:class.object.request) of `Object` sends the object `MAKESTRING`
when it has one and answers what that answers, and answers `.nil` when it has
none. So an object with a `MAKESTRING` method is replaced by its answer, as for
an [`Array`](r:class.array.makestring) or a
[`MutableBuffer`](r:class.mutablebuffer.makestring), and an object with only a
`STRING` method by the answer of that method. A class that declares its own
`REQUEST` method decides the answer itself: its `MAKESTRING` method is not used
unless its `REQUEST` sends it, and a `REQUEST` that answers a string is used
even when the class has a `STRING` method. A `MAKESTRING` that answers `.nil`
or an object that is not a string is like a `REQUEST` that does. An instance of
a subclass of `String` is a string here, whether it is passed directly or is
what `REQUEST` answers: the builtin works on its own characters, and no message
is sent to it for that ([Deviation 109](r:class.string.deviation.109)). A
builtin that reads several such arguments reads them from left to right, before
it checks any of them, when the call has no fault ([the conversion
rule](r:bif.index.call.convert)). An argument that a builtin reads
as a whole number or as a number is read this way too, and the replacement must
then be a number. A builtin whose page links [the native call
rule](r:bif.index.call.native) has no fallback to `STRING`, as that rule says,
and the same holds for the string arguments of methods ([the method argument
rule](r:class.index.string)). A `REQUEST`, `MAKESTRING` or `STRING` method that
is private and would be sent is not one the replacement can use: the call
raises `SYNTAX 97.2`. A comparison operator converts its right operand once, as
[the comparison rule](r:expr.comparison.object) says.

```rexx
say upper(.object~new) upper(.both~new) upper(.mutablebuffer~new('mb'))
say c2x(.array~of(1, 2)) length(.array~of(1, 2)~string)
signal on syntax name a
say upper(.hidden~new)
a: say condition('o')~code
signal on syntax name b
say upper(.hiddenstr~new)
b: say condition('o')~code
::class both
::method makestring
  return 'made'
::method string
  return 'told'
::class hidden
::method makestring private
  return 'secret'
::class hiddenstr
::method string private
  return 'secret'
```

```output
AN OBJECT MADE MB
310A32 8
97.2
97.2
```

```rexx
say upper(.mk~new) upper(.strsub~new('abc'))
::class strsub subclass string
::class mk
::method makestring
  return .strsub~new('ss')
```

```output
SS ABC
```

```rexx
say length(.asks~new)
say length(.asknil~new)
say length(.askobj~new)
say length(.askdef~new)
say length(.askup~new)
say copies('ab', .two~new) 1 + .two~new 'x' || .two~new
signal on syntax name p
say length(.askpriv~new)
p: say condition('o')~code
::class asks
::method request
  say 'request' arg(1) arg()
  return 'abcd'
::method makestring
  say 'makestring'
  return 'xx'
::method string
  say 'string'
  return 'yy'
::class asknil
::method request
  return .nil
::method makestring
  say 'makestring'
  return 'xx'
::method string
  return 'abc'
::class askobj
::method request
  return .object~new
::method string
  return 'ab'
::class askdef
::method request
  return .nil
::class askup
::method request
  say 'sent' arg(1)
  return self~request:super(arg(1))
::method makestring
  say 'makestring'
  return 'm'
::class two
::method request
  return 2
::class askpriv
::method request private
  return 'abc'
::method string
  return 'abc'
```

```output
request STRING 1
4
3
2
9
sent STRING
makestring
1
abab 3 x2
97.2
```

r[conv.string.answer]
An answer that has no string value counts as no answer, so that the
conversion of an object always ends in a string and never raises an error of
its own. The answer of a `STRING` method has a string value when it is a
string, a number or an instance of a subclass of `String`, when it is an array,
whose items joined by a newline are its string value, or a `MutableBuffer`,
and when its own `MAKESTRING` answers one of these. The answer of an
`OBJECTNAME` method of the object's own and of a `MAKESTRING` method has one
when it is one of the first five; no message is sent to it. Any other answer,
such as `.nil`, a `Directory`, a `Queue`, an `Object` or the object itself, has
none. Where the `STRING` answer has none, the object's string value is the one
the `STRING` method of `Object` answers for it, its `OBJECTNAME`. Where the
`OBJECTNAME` answer has none, it is the default name of the object's class: `a`
or `an` and the class's id, whatever name was set on the object and whatever
its `DEFAULTNAME` answers. Where the `MAKESTRING` answer, or the answer of a
`REQUEST` method of the object's own, has none, or a `REQUEST` method answers
nothing, the object has not converted, and its `STRING` method is sent as
[the string rule](r:conv.string.object) says. Every clause and every argument
of a builtin reads that string as it reads any string: a builtin checks it as
the argument's kind requires, `PARSE VALUE` parses it, and an operator, a
comparison and a truth value read it. A `NOSTRING` trap takes the conversion
as it takes that of any object converted by its `STRING` method, with that
string as its description ([the information rule](r:cond.nostring.information)).

```rexx
o = .weird~new
say length(o) words(o) 'a'||o
parse value o with p q
say q ('a WEIRD' == o) changestr('E', 'xEy', o)
signal on syntax name s
say left('abc', o)
s: say condition('o')~code
say ('a' || .named~new) ('a' || .badname~new) ('a' || .made~new)
::class weird
::method string
  return .directory~new
::class named
::method string
  return .nil
::method objectname
  return 'NAMED'
::class badname
::method objectname
  return .directory~new
::class made
::method string
  return .maker~new
::class maker
::method makestring
  return 'made'
```

```output
7 2 aa WEIRD
WEIRD 1 xa WEIRDy
40.12
aNAMED aa BADNAME amade
```

r[conv.string.whitespace]
Whitespace is the blank, `'20'x`, and the horizontal tab, `'09'x`, and no
other character. Line feed, carriage return, form feed and vertical tab are
not whitespace.

```rexx
signal on syntax name e
say substr('abc', '09'x || '2 ')
say substr('abc', '2' || '0a'x)
e: say condition('o')~code
```

```output
bc
40.12
```

## Positions

r[conv.position.origin]
Positions count from 1. Position 1 is the first character of a string, or
its first word where a page counts words, position 2 the one after it, and
so on. No character or word is at position 0.

```rexx
say substr('abc', 1, 1) substr('abc', 3, 1)
say word('x y z', 2)
```

```output
a c
y
```

## Whole-number arguments

> **Note.** Position and length arguments are whole numbers. The two rules
> below say which strings a whole-number argument accepts and what value it
> then has.

r[conv.whole.form]
A whole-number argument must be a string of this form: optional whitespace;
an optional sign, `+` or `-`, which may be followed by whitespace; one or
more digits, with at most one period among or around them; optionally an
exponent, which is `E` or `e` directly after the last digit or the period,
an optional sign, and one or more digits; optional whitespace. Any other
string raises `SYNTAX 40.12`, the null string and a string of only
whitespace included.

```rexx
say substr('abcdef', ' + 2 ', 1) substr('abcdef', '2.', 1),
    substr('abcdef', '.2E1', 1) substr('abcdef', '20e-1', 1),
    substr('abcdef', '2.E0', 1)
signal on syntax name a
say substr('abc', '')
a: say condition('o')~code
signal on syntax name b
say substr('abc', '1 E1')
b: say condition('o')~code
signal on syntax name c
say substr('abc', '+-2')
c: say condition('o')~code
signal on syntax name d
say substr('abc', '0x2')
d: say condition('o')~code
```

```output
b b b b b
40.12
40.12
40.12
40.12
```

r[conv.whole.digits]
On a 64-bit host, which is what interrexx targets, the value of a
whole-number argument is its number rounded to 18 significant digits; when
the first digit dropped is 5 or more, the magnitude rounds up, otherwise
down. The caller's NUMERIC DIGITS setting plays no part. If the rounded
value has a fractional part other than zero, or its magnitude is 10\*\*18 or
more, the argument raises `SYNTAX 40.12`; otherwise its value is that
integer. (The reference manual gives 9 digits for 32-bit hosts; this book
does not specify those.)

```rexx
say substr('abcdef', '2.000', 1) substr('abcdef', '1.0000000000000000001', 1)
say '['word('a b', 999999999999999999)']'
numeric digits 3
signal on syntax name a
say substr('abcdef', 2, 1.0001)
a: say condition('o')~code
signal on syntax name b
say substr('abc', '2.5')
b: say condition('o')~code
signal on syntax name c
say word('a b', 1000000000000000000)
c: say condition('o')~code
signal on syntax name d
say word('a b', '999999999999999999.5')
d: say condition('o')~code
```

```output
b a
[]
40.12
40.12
40.12
40.12
```

## Builtins that run as String methods

r[conv.method]
A builtin that runs as a String method does its work by sending the
[`String`](r:class.string.builtins) message of the same name. One of its
arguments, usually the first and for a few builtins the second, is the
receiver; the others become the message's arguments, in their order. The
builtin and the message then give the same result for the same arguments. A
builtin page says when its builtin is one of these. The range checks below,
and [the checks they belong to](r:conv.check.order), are rules for these
builtins only; any other builtin's page states its own range checks. Where one
of those checks finds a fault, or the [number](r:conv.method.number) or the
[option](r:conv.method.option) check below does, the builtin raises an error
40 of its own; the same check reached by sending the message raises the
method's error 93, as [the message rule](r:class.string.builtins.errors) gives
it.

```rexx
say substr('abcd', 2, 2) 'abcd'~substr(2, 2) upper('ab', 2) 'ab'~upper(2)
```

```output
bc bc aB aB
```

> **Note.** The ooRexx reference manual (Functions > Built-in Functions)
> names the builtins that run this way; the ANSI standard, which has no
> String methods, does not.

r[conv.position.check]
In a builtin that runs as a String method, a position argument is a
whole-number argument whose value must be positive. A value of 0 or less
raises `SYNTAX 40.14`; `-0` is 0. Sent as a message, the same value raises
`SYNTAX 93.924`.

```rexx
signal on syntax name a
say substr('abc', 0)
a: say condition('o')~code
signal on syntax name b
say word('a b', '-0')
b: say condition('o')~code
signal on syntax name c
say substr('abc', -1)
c: say condition('o')~code
signal on syntax name d
say 'abc'~substr(0)
d: say condition('o')~code
```

```output
40.14
40.14
40.14
93.924
```

r[conv.length.check]
In a builtin that runs as a String method, a length argument is a
whole-number argument whose value must not be negative. A negative value
raises `SYNTAX 40.13`. A length of 0 is allowed, and so is `-0`, which is
0. A count, such as the `n` of `COPIES` or the count of `CHANGESTR`, is
checked the same way. Sent as a message, a negative length raises `SYNTAX
93.923` and a negative count `SYNTAX 93.906`.

```rexx
say '['substr('abc', 2, 0)']' '['substr('abc', 2, '-0')']'
signal on syntax name a
say substr('abc', 1, -1)
a: say condition('o')~code
signal on syntax name b
say copies('ab', -1)
b: say condition('o')~code
signal on syntax name c
say 'abc'~substr(1, -1)
c: say condition('o')~code
signal on syntax name d
say 'ab'~copies(-1)
d: say condition('o')~code
```

```output
[] []
40.13
40.13
93.923
93.906
```

r[conv.method.number]
In a builtin that runs as a String method, an argument that the builtin reads
as a number and that is not one raises `SYNTAX 40.11`. Such an argument is the
number of `ABS`, `FORMAT`, `SIGN` and `TRUNC`, and each number of `MAX` and
`MIN`, the first and the others alike. Sent as a message, a receiver that is
not a number raises `SYNTAX 93.943`, and a further number of `MAX` or `MIN`
that is not one `SYNTAX 93.904`.

```rexx
signal on syntax name a
say abs('x')
a: say condition('o')~code
signal on syntax name b
say max(1, 'x')
b: say condition('o')~code
signal on syntax name c
say 'x'~abs
c: say condition('o')~code
signal on syntax name d
say 1~max('x')
d: say condition('o')~code
```

```output
40.11
40.11
93.943
93.904
```

r[conv.method.option]
In a builtin that runs as a String method, an option argument whose first
character is not one of the letters the builtin's page lists, and one that is
the null string, raise `SYNTAX 40.904`. Such an argument is the option of
`STRIP` and of `VERIFY` and the type of `DATATYPE`. Sent as a message, the same
option raises `SYNTAX 93.915`.

```rexx
signal on syntax name a
say strip('abc', 'X')
a: say condition('o')~code
signal on syntax name b
say datatype('1', 'Q')
b: say condition('o')~code
signal on syntax name c
say 'abc'~strip('X')
c: say condition('o')~code
```

```output
40.904
40.904
93.915
```

> **Note.** The ANSI draft (9.2.1), which has no String methods, raises the
> same 40.14 for a position and 40.13 for a length.

## Pad arguments

r[conv.pad]
A pad argument must be a string of exactly one character. Any character may
be the pad, the blank and `'00'x` included. A pad of any other length raises
`SYNTAX 40.23`; so does the null string, and so does a character that an
encoding writes as two bytes ([two characters](r:conv.string.chars)).

```rexx
say substr('ab', 1, 4, '*') c2x(substr('ab', 1, 3, '00'x))
signal on syntax name a
say substr('abc', 1, 2, '**')
a: say condition('o')~code
signal on syntax name b
say substr('abc', 1, 2, '')
b: say condition('o')~code
signal on syntax name c
say substr('abc', 1, 2, 'c3a9'x)
c: say condition('o')~code
```

```output
ab** 616200
40.23
40.23
40.23
```

## Omitted and empty arguments

r[conv.arg.omitted]
An argument is omitted when its place in the argument list holds no
expression: nothing before the first comma, nothing between two commas, or
nothing after the last one. An omitted optional argument takes the default
that its page states.

```rexx
say substr('abc', 1, , '.') substr('abc', 2, )
```

```output
abc bc
```

r[conv.arg.empty]
An argument whose expression yields the null string is not omitted and never
takes the default. It is checked like any other value: as a whole number it
raises `SYNTAX 40.12` ([whole-number form](r:conv.whole.form)), as a pad
`SYNTAX 40.23` ([pad](r:conv.pad)), and as a string it is the null string.

```rexx
signal on syntax name a
say substr('abc', 1, '')
a: say condition('o')~code
signal on syntax name b
say substr('abc', 1, 5, '')
b: say condition('o')~code
```

```output
40.12
40.23
```

## Evaluation order and several faults

r[conv.error.precedence]
An operator evaluates both operands, the left before the right, before it
checks either, and a call evaluates its arguments from left to right before
the call is made. An error raised while an operand or an argument is evaluated
is raised at that point, and nothing after it is evaluated. Each control
expression of a DO is an operation of its own: the instruction evaluates and
checks them one at a time, in the order they are written ([the DO
rule](r:instr.do.control.check)). A builtin's
[conversion](r:bif.index.call.convert) of the arguments that are
objects, by sending them `STRING`, is part of its checks: a call that has no
fault sends `STRING` to each such argument, from left to right, before its
other checks. When the checks of one operation, which is one operator, one
call of a builtin or a method, or the checks of one instruction, find one
fault, the operation raises the error that the rule for that fault gives.
When they find more
than one, a wrong number of arguments or an omitted required argument among
them, the operation raises the error of one of them, and which one is
unspecified. Whether the checks after the first fault found are made, and so
whether the messages they would send are sent, is unspecified too, and for a
builtin call with a fault, whether and to how many arguments it sends `STRING`
is unspecified. When one operation finds a fault that raises a condition, such
as LOSTDIGITS, and another that raises an error, which of the two it raises
first is unspecified, and so whether a trap for the condition runs. A page
that lists the checks of an operation lists what is checked, not an order
among them.

```rexx
signal on syntax name a
say substr('abc', 0)
a: say condition('o')~code
signal on syntax name b
say substr('abc', 1, 'x')
b: say condition('o')~code
signal on syntax name c
say 'x' + f()
c: say condition('o')~code
signal on syntax name d
say substr('abc', 0, 'x')
d: say wordpos(condition('o')~code, '40.14 40.12') > 0
signal on syntax name e
say substr('abc', 'x', 1, ' ', 5)
e: say wordpos(condition('o')~code, '40.4 40.12') > 0
exit
f: say 'f ran'
return 'y'
```

```output
40.14
40.12
f ran
41.1
1
1
```

r[conv.check.order]
In a builtin that runs as a String method, the arguments have the form checks
of a [whole number](r:conv.whole.form), its [value](r:conv.whole.digits) and a
[pad](r:conv.pad), and the range checks of [positions](r:conv.position.check)
and [lengths](r:conv.length.check). A builtin page that has checks of its own
names them. A call with one argument in error raises that argument's error.
When more than one argument fails, or one argument fails more than one check,
the call raises one of their errors, as [the precedence
rule](r:conv.error.precedence) says.

```rexx
signal on syntax name a
say substr('abc', 1, 2, '**')
a: say condition('o')~code
signal on syntax name b
say translate('abc', , , '**')
b: say condition('o')~code
signal on syntax name f
say substr('abc', 1, -1)
f: say condition('o')~code
signal on syntax name g
say substr('abc', 'x', -1, '**')
g: say wordpos(condition('o')~code, '40.12 40.13 40.23') > 0
```

```output
40.23
40.23
40.13
1
```

> **Note.** ANSI (9.2.1) checks each argument's form and range together, one
> argument at a time, and gives which fault of several a call raises. ooRexx
> departs from that order, and interrexx keeps ooRexx's where they differ, so
> neither is part of this book.

## Errors and results

r[conv.error.name]
A rule that says something "raises `SYNTAX m.n`" means that the builtin
returns no value and the SYNTAX condition is raised with error number `m`
and subcode `n`. A trap for it sees `condition('C')` as `SYNTAX`,
`condition('O')~code` as the string `m.n`, and the variable `RC` set to
`m`. The error number and the subcode are both part of what such a rule
states. This book does not state the error message.

```rexx
signal on syntax name e
x = substr('abc', 0)
say 'not reached'
e: say condition('c') condition('o')~code rc
```

```output
SYNTAX 40.14 40
```

r[conv.error.text]
The wording of a message is unspecified wherever it reaches a program: the
result of the ERRORTEXT builtin, the `ERRORTEXT` and `MESSAGE` entries of a
condition object and the text that `CONDITION('D')` carries where it holds a
message, the report that an error writes to the error stream after its frame,
the notice and the error lines of interactive tracing, a trace line that
names a built-in method, which a build may not show at all ([the built-in
rule](r:class.stackframe.builtin)), the answers of SysGetMessage for the
interpreter's own catalog and of SysGeterrnomsg, the C API's functions that
answer the text of an error, the usage texts of the commands, and the messages
of the utility programs. What is specified is the code, and for the report
the frame that names the error number, the program and the line (`Error
N running F line L:`, or `Error N.M:` where no line applies). A message exists
for a number or it does not, as the [ERRORTEXT rules](r:bif.errortext.result)
say, and an example in this book shows a message only by whether it is the
null string or equals another message, never by its words. The words `Error`,
`running` and `line` of the frame and the names of the months are short names
that stay as they are, not wording that is free.

```rexx
signal on syntax name e
x = substr('abc', 0)
e: o = condition('o')
say (errortext(40) \== '') (o~errortext == errortext(40)) (o~message \== '')
```

```output
1 1 1
```

r[conv.error.additional]
For an error the interpreter raises, the `ADDITIONAL` entry of the condition
object is an Array, and the number, order and values of its items are
unspecified, except where a rule states an item. A rule states an item only where the item is
needed to understand the error and the program cannot get it otherwise, as with
a number parse error that carries the string that failed to parse. A rule that
states one says which item it is and what it holds. An error that `RAISE` gives rise to is
such an error; the objects the clause supplied are governed by the [RAISE
rules](r:instr.raise.untrapped).

```rexx
signal on syntax name e
x = substr('abc', 0)
e: say condition('o')~additional~isA(.array)
```

```output
1
```

r[conv.returns]
A rule that says a builtin "returns" a string means that the function
call's value is exactly that string: the same characters in the same order,
with no blanks added or removed.

```rexx
say '['substr('abc', 2)']' '['substr(' a ', 1)']'
```

```output
[bc] [ a ]
```

## Deviations

r[conv.string.deviation.109]
A builtin that takes a string works on the own characters of an instance of a
subclass of `String`, and sends it none of `REQUEST`, `MAKESTRING` and
`STRING`, as [the string rule](r:conv.string.object) says and
[Deviation 109](r:class.string.deviation.109) states; ooRexx sends the
instance `request` with `STRING` or `MAKESTRING` when it is an argument of a
builtin function, so an override of either can change what is read. In
the example the subclass answers another string from `MAKESTRING`.

```rexx
.local~sent = 0
s = .st~new('ab')
say length(s) upper(s)
say 'sends' .local~sent
::class st subclass string
::method makestring
  .local~sent = .local~sent + 1
  return 'MADE'
```

```output
2 AB
sends 0
```
