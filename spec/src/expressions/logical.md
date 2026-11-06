---
element: "Logical operators"
kind: concept
ansi: "7.4.2, 7.4.8, 7.4.9"
rexxref: "Terms, Expressions, and Operators > Operators > Logical"
---

# Logical operators

The logical operators combine the values `0` and `1`, which are the values
that a comparison answers and that `IF` and `WHILE` test for. There are three
infix operators, `&`, `|` and `&&`, and one prefix operator, `\`. They work on
strings, and a string is a logical value only when it is exactly `0` or
exactly `1`. This page gives their truth tables, the error for any other
operand, and the order in which their operands are evaluated and checked.

## Operators

r[expr.logical.operators]
The operator `&` is true when both operands are `1`, the operator `|` is true
when at least one is, and `&&` is true when exactly one is. The prefix `\` is
true when its operand is `0`. Each answers the string `1` or `0`. The three
infix operators rank in [two levels](r:expr.index.precedence), `&` above `|`
and `&&`, and `|` and `&&` share one level.

```rexx
say '& ' (0 & 0) (0 & 1) (1 & 0) (1 & 1)
say '| ' (0 | 0) (0 | 1) (1 | 0) (1 | 1)
say '&&' (0 && 0) (0 && 1) (1 && 0) (1 && 1)
say '\ ' (\ 0) (\ 1) (\ \ 1)
say (1 | 1 & 0) ((1 | 1) & 0) (1 && 1 | 1) (1 | 1 && 1)
```

```output
&  0 0 0 1
|  0 1 1 1
&& 0 1 1 0
\  1 0 1
1 0 1 0
```

r[expr.logical.value]
An operand of a logical operator must be a string that is exactly `0` or
exactly `1`, with no other characters in it. Any other operand raises
`SYNTAX 34.901`, and the check does not take the number form into account:
`01`, `1.0`, `+1`, `1e0`, `0e0`, a string with a blank before or after the
digit, the null string and every other number and text are refused. The
results of comparisons and of the logical operators are logical values, and so
is the value of an arithmetic operator when it comes out as `0` or `1`.

```rexx
call try '1 & 1'
call try '1 & 2'
call try '"01" | 0'
call try '"1.0" | 0'
call try '" 1" & 1'
call try '"" | 1'
call try '"a" && 1'
call try '\ 2'
call try '\ "0.0"'
call try '(1 - 1) | (3 - 2)'
call try '(1.0 - 1) | (1.0 - 0.0)'
call try '(2 > 1) & (0 = 0.0)'
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
1 & 1 => 1
1 & 2 => 34.901
"01" | 0 => 34.901
"1.0" | 0 => 34.901
" 1" & 1 => 34.901
"" | 1 => 34.901
"a" && 1 => 34.901
\ 2 => 34.901
\ "0.0" => 34.901
(1 - 1) | (3 - 2) => 1
(1.0 - 1) | (1.0 - 0.0) => 34.901
(2 > 1) & (0 = 0.0) => 1
```

## Evaluation

r[expr.logical.order]
Both operands of `&`, `|` and `&&` are evaluated, from left to right, before
either is checked, so no operator stops early. A function call in the right
operand runs even when the left operand has already decided the answer, and an
error in the right operand is raised before a left operand that is not a
logical value is found out. The check then goes from left to right, so that
for two bad operands the left one is the one the error names, as the message
of the last example shows.

```rexx
say (0 & f(1)) (1 | f(2)) (1 && f(3))
call try '"x" & 1/0'
call try '"x" & f(4)'
call try '"x" | "y"'
call try '1 | "y"'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
f: say 'f' arg(1); return arg(1) > 0
```

```output
f 1
f 2
f 3
0 1 0
"x" & 1/0 => 42.3
f 4
"x" & f(4) => 34.901
"x" | "y" => 34.901
1 | "y" => 34.901
```

```rexx
signal on syntax name e
say "x" | "y"
e: say condition('o')~message
```

```output
Logical value must be exactly "0" or "1"; found "x".
```

r[expr.logical.prefix]
The prefix `\` takes the one term after it, as [the prefix
rule](r:expr.index.prefix) says, so `\ 1 = 1` is `(\ 1) = 1`, and a comparison
to be negated is put in parentheses. A `\` with no term after it raises
`SYNTAX 35.901`. The character is an operator only in front of a term or as
the first character of a comparison operator, so an infix `\` such as in
`1 \ 1` raises `SYNTAX 35.1`. The sign may be written `'AA'x` or `'AC'x`, as
[the not rule](r:lex.operators.not) says. Both errors are found when the
program is read, so a program file with one of them exits with `256` minus the
error number before it runs a clause, and the text of an INTERPRET instruction
raises them as ordinary conditions.

```rexx
say (\ 1 = 1) (\ (1 = 1)) (\ 0 > 1) (\ \ 0 = 0)
call try 'say \'
call try 'say 1 \ 1'
exit
try:
  signal on syntax name bad
  interpret arg(1)
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
0 0 0 1
say \ => 35.901
say 1 \ 1 => 35.1
```

```rexx
say 'not reached'
say \
```

```output rc=221
```

```rexx
say 'not reached'
say 1 \ 1
```

```output rc=221
```

r[expr.logical.object]
A logical operator is a message to its left operand, as [the operator
rule](r:expr.messages.operator) says, so an object on the left that does not
define it raises `SYNTAX 97.1`, and a string on the left checks the string
that [the conversion rule](r:conv.string.object) makes of the right operand as
a logical value. A right operand that is an object is therefore refused with
`34.901` unless that string is `0` or `1`.

```rexx
call try '.nil & 1'
call try '1 | .nil'
call try '\ .nil'
call try '1 & .one~new'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class one
::method makestring
  return '1'
```

```output
.nil & 1 => 97.1
1 | .nil => 34.901
\ .nil => 97.1
1 & .one~new => 1
```
