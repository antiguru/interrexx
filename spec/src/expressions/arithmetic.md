---
element: "Arithmetic operators"
kind: concept
ansi: "7.4.2, 7.4.3, 7.4.4, 7.4.5"
rexxref: "General Concepts > Terms, Expressions, and Operators > Operators > Arithmetic"
---

# Arithmetic operators

The arithmetic operators are `+` `-` `*` `/` `%` `//` and `**`, with `+` and
`-` also usable in front of one term. This page describes them as part of an
expression: which operands they take, in what order the operands are evaluated
and checked, and which errors an operand that is not a number causes. What the
operators compute, to how many digits and with which rounding, is in [the
arithmetic page of the numbers chapter](../numbers/arithmetic.md), and this
page links its rules rather than stating them again. Where an operator ranks
among the others is in [the precedence rule](r:expr.index.precedence).

## Operators and operands

r[expr.arithmetic.infix]
An infix arithmetic operator takes a term on each side, and its value is the
number that [the numbers rules](r:num.arithmetic.digits) give for the two
operand values: the sum, the difference, the product, the quotient, the
whole-number quotient and the remainder of `+` `-` `*` `/` `%` and `//`, and
the power of `**`. The result of a power is the left operand raised to the
right one. Blanks around an operator have no effect, as [the blank
rule](r:lex.operators.blank) says. The result is a string, written in the form
[the result rule](r:num.formats.result) gives, so `2 * 3` is the string `6`
and `1.50 + 1.50` is `3.00`.

```rexx
say 7 + 2 7 - 2 7 * 2 7 / 2 7 % 2 7 // 2 7 ** 2
say 7+2 7 +2 7+ 2 7  +  2
say 1.50 + 1.50 (1.50 * 2) (6 / 3) (2 ** 3)~length
```

```output
9 5 14 3.5 3 1 49
9 9 9 9
3.00 3.00 2 1
```

r[expr.arithmetic.prefix]
A prefix `+` or `-` takes the term that follows it, with the messages sent to
that term, as in [the prefix rule](r:expr.index.prefix), and its value is the
number that [the numbers rule](r:num.arithmetic.prefix) gives: `-` negates the
operand and `+` keeps its sign. Both also round the operand to NUMERIC DIGITS.
A prefix operator may follow another operator, infix or prefix, with or
without a blank between them, as in `5 - -3`, `5 * - 3` and `- - 3`. Two minus
signs written together are a [line comment](r:lex.comments.line), so `5 --3`
is `5` followed by a comment, and `5 - -3` is eight. A minus sign that is the
last token of a line is a [continuation](r:lex.clauses.continuation) and acts
as a blank, so `5 -` at the end of a line followed by `3` on the next is the
two terms `5` and `3` joined by a blank and not a subtraction.

```rexx
say - 3 + 3 (- - 3) (+ - + 3) (5 - -3) (5 * - 3) (5 ** - 1)
say 5 --3
x = 5 -
    3
say x
say (5 -
    3)
```

```output
0 3 -3 8 -15 0.2
5
5 3
5 3
```

r[expr.arithmetic.exponent]
The operator `**` takes a whole number as its right operand, and this number
may be negative, so that `2 ** -2` is `0.25`. The other operators take any
number. A right operand of `**` that is not a whole number raises
`SYNTAX 26.8`, as [the exponent rule](r:num.arithmetic.exponent) says. Because
`**` is applied from left to right, `2 ** 3 ** 2` is `(2 ** 3) ** 2`, which is
64.

```rexx
say (2 ** 3 ** 2) (2 ** -2) (-2 ** 2) (- 2 ** 3) (2 ** 0)
call try '2 ** 1.5'
call try '2 ** "x"'
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
64 0.25 4 -8 1
2 ** 1.5 => 26.8
2 ** "x" => 26.8
```

## Operands that are not numbers

r[expr.arithmetic.number]
Both operands of an infix operator, and the operand of a prefix one, must be
numbers, and a string that is not one raises `SYNTAX 41.1`, as [the operand
rule](r:num.arithmetic.operand) says. A symbol with no value is such a string,
because its value is its own name. Which operand is wrong makes no difference
to the code, and the base of `**` raises `41.1` like any left operand, where
its exponent raises `26.8`.

```rexx
call try '"ten" + 1'
call try '1 - "ten"'
call try '- "ten"'
call try '"" + 1'
call try '" " * 2'
call try 'unset + 1'
call try '"12ab" - 1'
call try '"ten" ** 2'
call try '2 ** "ten"'
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
"ten" + 1 => 41.1
1 - "ten" => 41.1
- "ten" => 41.1
"" + 1 => 41.1
" " * 2 => 41.1
unset + 1 => 41.1
"12ab" - 1 => 41.1
"ten" ** 2 => 41.1
2 ** "ten" => 26.8
```

r[expr.arithmetic.order]
Both operands of an operator are evaluated before the operator checks either
one, so an error in the evaluation of the right operand is raised before a
left operand that is not a number is found out. The operand check then goes
from left to right, and it comes before any other error that the operator
itself would raise, as [the operand rule](r:num.arithmetic.operand) says. A
function call in an operand takes its effect before the operator is applied,
whatever the operator does. Which of several operators is applied first
follows precedence, so `1 / 0 + "ten"` raises `42.3` from the division before
the addition looks at its operands.

```rexx
call try '"ten" + 1/0'
call try '"ten" + f(2)'
call try 'f(1) + "ten" / 0'
call try '1 / 0 + "ten"'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
f: say 'f' arg(1); return arg(1)
```

```output
"ten" + 1/0 => 42.3
f 2
"ten" + f(2) => 41.1
f 1
f(1) + "ten" / 0 => 41.1
1 / 0 + "ten" => 42.3
```

r[expr.arithmetic.zero]
A divisor of zero in `/`, `%` or `//` raises `SYNTAX 42.3`, as [the zero
rule](r:num.arithmetic.zero) says. The value of `0 ** 0` is `1`, and a zero
base with a negative exponent raises `SYNTAX 42.903`, as [the power
rule](r:num.arithmetic.power.error) says. A result of `+`, `-`, `*`, `/` or a
prefix operator that is too large raises `SYNTAX 42.901`, as [the overflow
rule](r:num.limits.overflow) says; the errors of `**` are those of [the power
rule](r:num.arithmetic.power.error). An operand with more digits than NUMERIC
DIGITS raises the condition that [the lost digits
rule](r:num.arithmetic.lostdigits) describes.

```rexx
call try '7 / 0'
call try '7 % 0'
call try '0 // 0'
call try '0 ** -1'
call try '0 ** 0'
call try '1e999999999 * 10'
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
7 / 0 => 42.3
7 % 0 => 42.3
0 // 0 => 42.3
0 ** -1 => 42.903
0 ** 0 => 1
1e999999999 * 10 => 42.901
```

## Operands that are not strings

r[expr.arithmetic.object]
The operand on the left of an infix operator decides how the operator runs:
the operator is a message to it, as [the operator
rule](r:expr.messages.operator) says. A left operand that is an object that
does not understand the message raises `SYNTAX 97.1`, as `.nil + 1` does. A
right operand that is not a string is replaced by a string as [the conversion
rule](r:conv.string.object) says, and that string must be a number, so that `1
+ .nil` raises `SYNTAX 41.1`, because `The NIL object` is not a number. A
prefix operator sends its message with no arguments, so `-.nil` raises `97.1`
too.

```rexx
call try '.nil + 1'
call try '1 + .nil'
call try '-.nil'
call try '+.array~new'
call try '1 + .array~new'
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
.nil + 1 => 97.1
1 + .nil => 41.1
-.nil => 97.1
+.array~new => 97.1
1 + .array~new => 41.1
```

## Missing operands

r[expr.arithmetic.missing]
An infix operator that lacks an operand on either side raises `SYNTAX 35.1`,
whether the operand is missing at the start of the expression, before a
closing parenthesis, or at the end of the clause. A prefix `+` or `-` with
nothing after it raises `SYNTAX 35.901` instead. A `-` that is the last token
of a line is a [continuation](r:lex.clauses.continuation) and takes the line
end with it, in a program file and in the text of an [INTERPRET
instruction](r:instr.interpret.lines) alike, so it is never an operator
without a right operand and no example can show that error for `-`. The error
is found when the program is read, so a program file with such an error runs
no clause and [exits with `256` minus the error
number](r:lex.index.read-first).

```rexx
say 'not reached'
say 3 +
```

```output rc=221
```

```rexx
say 'not reached'
say 3 * (2 +)
```

```output rc=221
```

```rexx
say 'not reached'
say +
```

```output rc=221
```

```rexx
call try 'say 3 +'
call try 'say 3 * (2 +)'
call try 'say * 3'
call try 'say (3 /)'
call try 'say +'
call try 'say 4 - +'
call try 'say (-)'
exit
try:
  signal on syntax name bad
  interpret arg(1)
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
say 3 + => 35.1
say 3 * (2 +) => 35.1
say * 3 => 35.1
say (3 /) => 35.1
say + => 35.901
say 4 - + => 35.901
say (-) => 35.901
```
