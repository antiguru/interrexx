---
element: "Concatenation"
kind: concept
ansi: "7.4.6"
rexxref: "General Concepts > Terms, Expressions, and Operators > String Concatenation"
---

# Concatenation

Concatenation joins two values into one string. Rexx has three forms of it,
which differ in what they put between the two values: the blank operator
puts one blank, and the abuttal and the operator `||` put nothing. The blank
is the form a program uses without noticing, as when it writes
`say 'total:' x`. This page says when two terms are concatenated, what the
joined string holds, and how concatenation ranks among the other operators.

r[expr.concatenation.blank]
Two terms that are separated by one or more blanks or tabs, with no operator
between them, are concatenated with exactly one blank in the result. The
blanks of the program are not kept, and a tab becomes a blank, but the blanks
that belong to the values are: a value that ends in a blank or begins with
one keeps it. The null string takes part like any other value, so a blank
remains on its side of the result. A blank next to an operator is no
concatenation, as [the blank rule](r:lex.operators.blank) says.

```rexx
say 'a'      'b'	'c'
say 'a ' 'b' ' c'
say '[' '' 'a' '' ']'
```

```output
a b c
a  b  c
[  a  ]
```

r[expr.concatenation.abuttal]
Two terms with nothing between them, or with only a comment between them, are
concatenated without a blank. This is the abuttal. It joins a string and a
symbol, as in `x'%'`, two parenthesized terms, and the value of a function
call with the term after it. A comment is
[no blank](r:lex.comments.separator), so it does not stop the abuttal.

```rexx
x = 5
say x'%' x'<' (x)(x) x/**/x
say abs(-2)abs(-3) (1 + 2)(3)
say x /**/ x
```

```output
5% 5< 55 55
23 33
5 5
```

r[expr.concatenation.operator]
The operator `||` concatenates the two terms with no blank, and blanks around
it are dropped. A line end ends the clause even after the operator, so a `||`
at the end of a line without a [continuation](r:lex.clauses.continuation)
leaves it without a right operand and raises `SYNTAX 35.1`. The second example
shows that in a program file, which [exits with `256` minus the error
number](r:lex.index.read-first), and the third shows it in the text of an
INTERPRET instruction, where the end of the text ends the clause and the error
is an ordinary condition.

```rexx
say 'a' || 'b' 'c'||'d'
say 'a' ||,
    'b'
```

```output
ab cd
ab
```

```rexx
say 'before'
say 'a' ||
```

```output rc=221
```

```rexx
signal on syntax name e
interpret "say 'a' ||"
e: say condition('o')~code
```

```output
35.1
```

r[expr.concatenation.precedence]
The three forms are on one level of [precedence](r:expr.index.precedence),
below the arithmetic operators and above the comparisons. The operands of an
arithmetic operator are therefore evaluated before the result is joined to a
neighbour, and the concatenations of a run are applied from left to right.

```rexx
say 1+2 3*4
say 1 + 1 || 2 * 3
say (1 || 2 = 12) (1 2 = '1 2')
```

```output
3 12
26
1 1
```

r[expr.concatenation.operands]
The operands are joined as they are, so a number is not put into its normal
form, and `1.50 || 2.0` is `1.502.0`. An operand that is an object other than
a string is replaced by a string first, on either side of the operator, by
[the conversion rule](r:conv.string.object). The result is a `String` also
when an operand is an instance of a subclass of `String`, and that operand
contributes its own characters.

```rexx
say 1.50 || 2.0 007 || 7
arr = .array~of(1, 2)
say ('x' arr) ('x' .both~new) (.both~new 'x') (.nil 'x')
say ('x' .strsub~new('sub'))~class~id
signal on syntax name e
say 'x' .hidden~new
e: say condition('o')~code
::class both
::method makestring
  return 'made'
::method string
  return 'told'
::class strsub subclass string
::method string
  return 'overridden'
::class hidden
::method makestring private
  return 'secret'
```

```output
1.502.0 0077
x 1
2 x made made x The NIL object x
String
97.2
```

r[expr.concatenation.sign]
A blank before `+` or `-` does not start a concatenation, because the sign is
an infix operator there and the blank next to it has no effect. To join a
signed number to the term before it with a blank, the number goes in
parentheses, and `||` takes it directly.

```rexx
say (3 -1) (3 - 1) (3 (-1)) (3 || -1)
```

```output
2 2 3 -1 3-1
```
