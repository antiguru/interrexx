---
element: "Terms"
kind: concept
ansi: "7.4.1, 7.4.9"
rexxref: "Terms, Expressions, and Operators > Terms and Expressions"
---

# Terms

A term is the unit that an operator works on: a piece of a program that has a
value without an operator being applied. A program writes a term as a string,
a symbol, a function call, a message, an expression in parentheses or a list
in parentheses. The kind is set by the first token of the term and by the
token after it, and not by the value. This page says what each kind of term
looks like and what its value is, and leaves the way that terms are combined
to [the overview](index.md).

r[expr.terms.kinds]
A term is one of these: a [literal string](r:lex.literals.string), a
[symbol](r:lex.symbols.kinds), a [function call](r:expr.terms.call), a
[message term](r:expr.terms.message), an expression in
[parentheses](r:expr.terms.paren) or an [array term](r:expr.terms.array).
Each has a value, and that value is an object. This book does not specify the
variable reference terms, which are written with a `>` or `<` before a
symbol.

```rexx
v = 'var'
say 'str' 12 v length('abc') 'abc'~length (1 + 1) (7, 8)~items
```

```output
str 12 var 3 3 2 2
```

r[expr.terms.string]
The value of a literal string is the one that [the lexical
rule](r:lex.literals.string) gives, and that of a [hexadecimal](r:lex.literals.hex)
or [binary](r:lex.literals.bin) string is the characters it spells. A literal
is not a number until an operation reads it as one, so `'007'` keeps its
zeros.

```rexx
say 'Don''t' "say ""no""" '41 42'x '01000001'b
say '007' 007 + 0
```

```output
Don't say "no" AB A
007 7
```

r[expr.terms.symbol]
The value of a symbol term follows from the kind of symbol that
[the lexical rules](r:lex.symbols.kinds) set. The value of a [constant
symbol](r:lex.symbols.constant) and of an [environment
symbol](r:lex.symbols.environment) is given there. A variable symbol has the
value of the variable it names, which [the variables
page](r:expr.variables.value) describes.

```rexx
x = 'val'
say 1e3 x X .nil~string .nosuch
```

```output
1E3 val val The NIL object .NOSUCH
```

r[expr.terms.call]
A function call is a symbol or a string that is directly followed by `(`,
with no blank between them, and a list of arguments up to the matching `)`.
The arguments are expressions separated by commas, and an argument may be
left empty, as [the builtin rules](r:bif.index.call.count) describe. The
arguments are evaluated from left to right before the function is called,
and the value of the call is the value the function returns. A blank between
the name and the `(` makes the name an ordinary term, as
[the adjacency rule](r:lex.operators.adjacent) says, so that `n (1)` is a
symbol and a parenthesized expression, concatenated.

```rexx
say left(n('abc'), n(2)) max(n(3), n(9), n(2))
say n(1)(2) n (1)
exit
n: say 'eval' arg(1); return arg(1)
```

```output
eval abc
eval 2
eval 3
eval 9
eval 2
ab 9
eval 1
12 N 1
```

r[expr.terms.call.name]
The name of a call that is a symbol is read in upper case, and the name of a
call that is a string is used as [the lexical rule](r:lex.literals.function)
says, as written. The function that is called is looked up by that name, and
any name that no routine has raises `SYNTAX 43.1`. When a name could be more
than one routine, which one is called is specified with the instructions that
define routines and is not part of this page.

```rexx
say abs(-1) Abs(-2) 'ABS'(-3)
signal on syntax name b
say nosuch(1)
b: say condition('o')~code
```

```output
1 2 3
43.1
```

r[expr.terms.call.result]
A call in an expression must return a value. A routine that returns with a
`RETURN` that has no expression raises `SYNTAX 44.1` when the call is a
term.

```rexx
signal on syntax name e
say 'before'
say none()
say 'not reached'
e: say condition('o')~code
exit
none: return
```

```output
before
44.1
```

r[expr.terms.paren]
An expression in parentheses is a term whose value is the value of the
expression. It may be the receiver of a message, as in `(1 + 2)~length`,
and a parenthesis that follows another term with no blank between them is
a second term, which is concatenated to the first without a blank, as
[the abuttal rule](r:expr.concatenation.abuttal) says. A pair of parentheses
with nothing between them is not a term and raises `SYNTAX 35.1`.

```rexx
say (1 + 2)~length (4)(5)
signal on syntax name e
interpret 'say ()'
e: say condition('o')~code
```

```output
1 45
35.1
```

r[expr.terms.message]
A message term is a term followed by `~` and the name of a message, written
as a symbol or a string and optionally followed by `(` and arguments with no
blank between them. The name is used in upper case, whichever way it is
written, so `'length'` and `length` are one message. Its value is what the
receiver answers to the message.
Message terms chain, each applying to the value of the one before it, and a
subscript `[ ]` after a term sends the message `[]` to it. A `~~` instead of
a `~` discards the answer and has the receiver as its value. The messages
that a string answers to are the string methods, such as
[`length`](r:bif.length.result), and the messages to every other class are
specified with that class. The chapter's `messages` page treats the operators
as messages and does not repeat the term syntax.

```rexx
x = 'abc'
say x~length x~left(2)~upper x~~reverse x~length~length
say (10, 20, 30)[2] 'abc'~"LENGTH" 'abc'~'length'
say .point~new~'Shown' .point~new~shown
::class point
::method shown
  return 'shown'
```

```output
3 AB abc 1
20 3 3
shown shown
```

r[expr.terms.array]
A list of two or more expressions separated by commas, in parentheses, is
an array term. Its value is a new array that holds the values of the
expressions, in the order they are written, at the positions 1, 2 and so on,
and the expressions are evaluated from left to right. An array term can be
the receiver of a message, and it is how a call passes one array as an
argument: `f((1, 2))` has one argument where `f(1, 2)` has two. The array
term with an omitted item is not specified by this book.

```rexx
a = (n(5), n(6) + 1, 'x')
say a~class~id a~items a[1] a[2] a[3]
say arg_count((1, 2)) arg_count(1, 2)
exit
n: say 'eval' arg(1); return arg(1)
arg_count: return arg()
```

```output
eval 5
eval 6
Array 3 5 7 x
1 2
```
