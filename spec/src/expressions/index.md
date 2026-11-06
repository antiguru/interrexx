---
element: "Expressions"
kind: chapter
ansi: "7.4, 7.4.1, 7.4.9"
rexxref: "General Concepts > Terms, Expressions, and Operators"
---

# Expressions

This chapter specifies how an expression is read and evaluated. An
expression is one term, or several terms joined by operators, and it has
one value when it has been evaluated. A term is a piece of the program that
has a value of its own: a string, a symbol, a function call, a message, a
parenthesized expression. The chapter covers what the terms are and how
variables give a symbol its value, and then how operators combine terms. This
page holds the rules that every operator shares: the precedence of the
operators and the order in which an expression is evaluated.

The pages of this chapter are:

* [Terms](terms.md): the kinds of term and the value of each.
* [Variables](variables.md): simple, stem and compound variables, and the
  value of a symbol that has none.
* [Concatenation](concatenation.md): the blank, the abuttal and `||`.

The operators that compute are described in four more pages of this chapter.
The [arithmetic](arithmetic.md) page covers `+` `-` `*` `/` `%` `//` and
`**`, the [comparison](comparison.md) page the normal and strict comparisons,
the [logical](logical.md) page `&` `|` `&&` and prefix `\`, and the
[messages](messages.md) page the message operators `~` and `~~`, with the
rule that every operator is a message. The characters of
the operators and the way a program is cut into tokens are in the
[lexical chapter](../lexical/index.md), and the numbers that arithmetic works
on are in the [numbers chapter](../numbers/index.md).

## Precedence

r[expr.index.value]
An expression has one value, which is the value of its only term when it has
one, and otherwise the result of the operator that is applied last. A value
is an object. The arithmetic, concatenation, comparison and logical operators
on strings answer strings, so the value of such an expression is a string,
written as [the number rules](r:num.formats.result) say when the operator
is an arithmetic one. An expression is evaluated in full: no term of it is
left unevaluated unless an error ends the evaluation first.

```rexx
say 7
say 2 * 3 'and' 1 + 1
say 'a' = 'a'
say 'a'~length
```

```output
7
6 and 2
1
1
```

r[expr.index.precedence]
Operators bind in nine levels. From the tightest to the loosest they are:
the message operators `~` and `~~` with the subscript `[ ]`; the prefix
operators `+`, `-` and `\`; the power `**`; the multiplicative operators
`*`, `/`, `%` and `//`; the additive operators `+` and `-` between two
terms; the concatenation operators, which are the blank, the abuttal and
`||`; the comparison operators, normal and strict ones together on one
level; `&`; and, loosest, `|` and `&&`. An operator takes as its operands
the nearest terms to its sides that no tighter operator has claimed.

```rexx
say (2 + 3 * 4) (2 * 3 ** 2) (1 + 1~length)
say (1 + 1 || 2) (1 = 1 || 0)
say (1 | 0 & 0) (0 & 0 = 0)
```

```output
14 18 2
22 0
1 0
```

r[expr.index.assoc]
Operators of the same level are applied from left to right, whatever the
operator: `-` and `/` are, and so is `**`, where `2**3**2` is `(2**3)**2`.
A chain of comparisons compares the result of the first with the next
operand, and `|` and `&&` share one level, so a chain of them is applied in
the order written.

```rexx
say (7 - 2 - 1) (100 / 10 / 5) (2 ** 3 ** 2)
say (2 = 2 = 2) (2 = 2 = 1)
say (1 | 1 && 1) (1 && 0 | 1)
```

```output
4 2 64
0 1
0 1
```

r[expr.index.prefix]
A prefix operator takes the term after it, with the messages sent to that
term, as its only operand and binds tighter than every infix operator but the
message operators. So `-2**2` is `(-2)**2`, which is 4, and a prefix operator
before a comparison applies to the left operand alone. The right operand of
an infix operator may itself start with a prefix operator, as in `2 ** -1`,
and prefix operators may follow one another, as in `- - 1`.

```rexx
say (-2 ** 2) (- 5 ** 2) (2 ** -1) (- - 1)
say (-'abc'~length) (\ 1 = 2) (\ (1 = 2))
```

```output
4 25 0.5 1
-3 0 1
```

r[expr.index.parens]
A pair of parentheses groups the expression between them, whatever the
operators around it bind. The grouped expression is evaluated as soon as it
is reached, and its value is the value of one term in the expression that
holds it. The parentheses that follow the name of a [function
call](r:expr.terms.call) or of a [message](r:expr.terms.message) are not a
group: they hold the arguments.

```rexx
say (2 + 3) * 4 (2 + 3 * 4) ((1 + 2) * (3 + 4))
say (7 - (2 - 1)) ((7 - 2) - 1)
```

```output
20 14 21
6 4
```

r[expr.index.syntax]
An expression that cannot be read raises a `SYNTAX` error: a `(` with no
matching `)` raises `36.901`, a `)` with no `(` raises `37.2`, and an
infix operator without the operand it needs, which includes an empty pair of
parentheses, raises `35.1`. A prefix operator without an operand raises
`35.901` instead, as [the arithmetic](r:expr.arithmetic.missing) and
[the logical](r:expr.logical.prefix) rules say, and a message operator
without a message name raises `19.909`. The fault is found when the program
is read, before any clause of it runs, so the program writes nothing and exits
with `256` minus the error number, as [the read rule](r:lex.index.read-first)
says. In the text of an INTERPRET instruction the same errors are ordinary
conditions that a trap can handle.

```rexx
say 'not reached'
say (1 + 2
```

```output rc=220
```

```rexx
say 'not reached'
say 1 + 2)
```

```output rc=219
```

```rexx
say 'not reached'
say 1 +
```

```output rc=221
```

```rexx
call try 'say (1 + 2'
call try 'say 1 + 2)'
call try 'say 1 +'
call try 'say ()'
call try 'say 1 + \'
call try 'say 1 ~'
exit
try:
  signal on syntax name bad
  interpret arg(1)
bad: say condition('o')~code
  return
```

```output
36.901
37.2
35.1
35.1
35.901
19.909
```

## Evaluation order

r[expr.index.order]
The terms of an expression are evaluated from left to right, each one when
the evaluation reaches it. An operator is applied as soon as its right
operand is known and the operator after it does not bind tighter, so a
tighter operator further right is applied first and the terms around it are
still evaluated in the order written. This holds for a term that has an
effect, such as a function call, and for an operator that has one, such as a
method of a class named `+`. Parentheses change which operators apply to which
terms and never the order of the terms.

```rexx
say (n(1) + n(2) * n(3))~v
say '--'
say ((n(1) + n(2)) * n(3))~v
exit
n: procedure
  say 'term' arg(1)
  return .t~new(arg(1))
::class t
::method init
  expose v
  use arg v
::method v attribute
::method '+'
  expose v
  use arg o
  say 'add' v o~v
  return .t~new(v + o~v)
::method '*'
  expose v
  use arg o
  say 'mul' v o~v
  return .t~new(v * o~v)
```

```output
term 1
term 2
term 3
mul 2 3
add 1 6
7
--
term 1
term 2
add 1 2
term 3
mul 3 3
9
```

r[expr.index.error]
An error raised while an expression is evaluated ends the evaluation: the
terms to the right of the failing operator are not evaluated. The error is
raised as soon as the operator is applied, so a term after it has no effect
even when it is a function call.

```rexx
signal on syntax name e
say n(1) + 'x' + n(2)
e: say condition('o')~code
exit
n: say 'term' arg(1); return arg(1)
```

```output
term 1
41.1
```
