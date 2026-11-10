---
element: "Message operators"
kind: concept
rexxref: "General Concepts > Terms, Expressions, and Operators > Operators"
---

# Message operators

ooRexx is object based, and the operators of an expression are part of that.
The tilde `~` sends a message to the value of the term before it, the double
tilde `~~` does the same and keeps the receiver as its value, and the brackets
`[ ]` send the message `[]`. An arithmetic, comparison, logical or
concatenation operator is a message too, sent to its left operand with the
right operand as its argument. This page gives the rules that make the
operators and the message terms one mechanism. The syntax of a message term is
in [the terms page](terms.md), and the classes, their methods and the messages
every class understands are specified in [the chapters about
classes](../classes/index.md). The ANSI standard has no message operators.

## Message terms

r[expr.messages.name]
The name of a message may be written as a symbol or as a string, and a message
is understood without regard to the case of its name. A message to an object
that has no [method](r:class.methods.search) of that name raises
`SYNTAX 97.1`. A string answers the messages of [the string
methods](r:class.string.builtins), such as `length` and `left`, and its
answers are specified with [the builtins that run as these
methods](r:conv.method).

```rexx
say 'abc'~length 'abc'~LENGTH 'abc'~"Length" 'abc'~"left"(2)
signal on syntax name e
say 'abc'~nosuch
e: say condition('o')~code
```

```output
3 3 3 ab
97.1
```

r[expr.messages.cascade]
The operator `~~` sends the message and then discards the answer, so that the
value of the term is the receiver. It is how a message that answers nothing is
used inside an expression, and how a chain of messages goes on with the same
receiver. A message with `~` that answers nothing and is used as a term raises
`SYNTAX 91.999`, as [a function call that returns no
value](r:expr.terms.call.result) raises `SYNTAX 44.1`.

```rexx
o = .box~new
say o~~note~~note~class~id
signal on syntax name e
say o~note~class~id
e: say condition('o')~code
::class box
::method note
  say 'note'
```

```output
note
note
BOX
note
91.999
```

r[expr.messages.subscript]
A subscript `[ ]` after a term sends the message `[]` to the value of the term
with the expressions between the brackets as its arguments, which may be none.
The `[` must follow the term with no blank between them, as [the adjacency
rule](r:lex.operators.adjacent) says. The brackets are the same operator as
`~` followed by the name `[]` written as a string, and they share the
precedence of the message operators.

```rexx
o = .box~new
say o[3] o[3, 4] o[] o~"[]"(5)
say o[1 + 1]~length
::class box
::method '[]'
  return arg()
```

```output
1 2 0 1
1
```

## Operators as messages

r[expr.messages.operator]
An infix operator sends the message that has its characters as its name to the
value of its left operand, with the value of the right operand as the only
argument, and the value of the operation is what the receiver answers. A
prefix operator sends the message of its own character with no arguments. The
names are `+` `-` `*` `/` `%` `//` `**` `||` `&` `|` `&&`, the normal
comparisons `=` `\=` `<>` `><` `<` `>` `<=` `>=` `\<` and `\>`, and the strict
comparisons `==` `\==` `<<` `>>` `<<=` `>>=` `\<<` and `\>>`. The prefix
messages are `+`, `-` and `\`. The blank operator sends the message whose name
is one blank, and [the abuttal](r:expr.messages.abuttal) sends the message
whose name is the null string. The one name `+` and the one name `-` serve
both the infix and the prefix form, and they are told apart by the number of
arguments.

```rexx
say (.t~new + 1) (.t~new - 1) (- .t~new) (\ .t~new)
say (.t~new = 1) (.t~new << 1) (.t~new & 1)
say (.t~new 1) (.t~new || 1)
::class t
::method '+'
  return 'plus' arg()
::method '-'
  return 'minus' arg()
::method '\'
  return 'not' arg()
::method '='
  return 'eq' arg()
::method '<<'
  return 'strictlt' arg()
::method '&'
  return 'and' arg()
::method ' '
  return 'blank' arg()
::method '||'
  return 'cc' arg()
```

```output
plus 1 minus 1 minus 0 not 0
eq 1 strictlt 1 and 1
blank 1 cc 1
```

r[expr.messages.abuttal]
The abuttal of two terms is a message like the blank and `||` are, with the
null string as its name. It is sent to the value of the left operand with the
value of the right operand as the only argument, so a class that defines a
method with the name `''` decides what the abuttal of its instances does, and
the methods named `||` and one blank have no part in it. An object whose class
defines no such method understands the message all the same, as [the default
rule](r:expr.messages.default) says.

```rexx
say (.t~new 1) (.t~new || 1)
say (.t~new'1') (.t~new/**/1) ((.t~new)(1))
say (.o~new'1') '3'~""(2)
::class t
::method ' '
  return 'blank'
::method '||'
  return 'cc'
::method ''
  return 'abut' arg()
::class o
```

```output
blank cc
abut 1 abut 1 abut 1
an O1 32
```

r[expr.messages.left]
The left operand alone decides which method runs. A right operand does not
choose the method: it is an argument, and what the receiver does with it is
the business of the method. A string as left operand runs its own methods for
every operator name, which compare, calculate or join with the string of the
right operand. The only messages that a string sends to a right operand that
is an object are the ones that [the conversion rule](r:conv.string.object)
describes, which ask for that string, and the operator message of the right
operand is never sent. An object that has no method for the operator raises
`SYNTAX 97.1`.

```rexx
call try '.t~new + 1'
call try '1 + .t~new'
call try '1 + .m~new'
call try '1 = .t~new'
call try '"a T" = .t~new'
call try '1 || .t~new'
call try '.t~new || 1'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class t
::method '+'
  return 'plus'
::class m
::method makestring
  say 'makestring called'
  return '5'
```

```output
.t~new + 1 => plus
1 + .t~new => 41.1
makestring called
1 + .m~new => 6
1 = .t~new => 0
"a T" = .t~new => 1
1 || .t~new => 1a T
.t~new || 1 => a T1
```

r[expr.messages.default]
Every object understands the equality messages `=`, `\=`, `==`, `\==`, `<>`
and `><`, and the messages `||`, one blank and the null string, whatever its
class. [The equality messages](r:class.object.eq) answer whether the receiver
and the argument are the same object by default, though a class may define its
own, as [the string class](r:class.string.operators) does. [The three joining
messages](r:class.object.concat) join the string value of the receiver to the
string value of the argument. Every other operator message, which means the
arithmetic ones, the logical ones and the ordering ones, normal and strict, is
understood only by a class that defines it.

```rexx
a = .array~new
say (a = a) (a == a) (a \= a) (a \== a) (a <> a) (a >< a)
say (a = .array~new) (a \= .array~new) (a == .array~new)
o = .o~new
say (o || 1) (o 1) (o~""(1))
signal on syntax name e
say a < a
e: say condition('o')~code
::class o
```

```output
1 1 0 0 0 0
0 1 0
an O1 an O 1 an O1
97.1
```

r[expr.messages.string]
A string understands each operator name as [a
method](r:class.string.operators). The infix names take exactly one argument
and raise `SYNTAX 93.903` when it is missing and `SYNTAX 93.902` when there
are more, and `+`, `-` and `\` take none for their prefix form. The method
does what the operator does, so a string used with an operator name behaves as
the expression would, including the errors for operands that are not numbers
or not logical values.

```rexx
say '3'~"+"(4) '3'~"-"(4) '3'~"-" '3'~"\=="(3) '1'~"\" '1'~"&&"(1)
call try '"3"~"+"(4, 5)'
call try '"3"~"+"("x")'
call try '"3"~"="()'
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
7 -1 -3 0 0 0
"3"~"+"(4, 5) => 93.902
"3"~"+"("x") => 41.1
"3"~"="() => 93.903
```
