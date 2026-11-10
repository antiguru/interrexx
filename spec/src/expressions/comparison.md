---
element: "Comparison operators"
kind: concept
ansi: "7.4.7"
rexxref: "General Concepts > Terms, Expressions, and Operators > Operators > Comparison"
deviations: [122]
---

# Comparison operators

A comparison operator compares the values of two terms and answers `1` when
the relation holds and `0` when it does not. Rexx has two families of them:
the normal comparisons, which compare numbers as numbers and anything else as
text with some leniency about blanks, and the strict comparisons, which always
compare the characters exactly. This page lists the operators, gives the rules
for comparing text in both families, and ties them to the rules for numbers.
How two numbers compare, including NUMERIC FUZZ, is in [the numeric comparison
page](../numbers/comparison.md).

## The operators

r[expr.comparison.operators]
The normal comparison operators are `=` `\=` `<>` `><` `<` `>` `<=` `>=` `\<`
and `\>`. The strict ones are `==` `\==` `<<` `>>` `<<=` `>>=` `\<<` and
`\>>`, all of them tokens of [the operator set](r:lex.operators.set). Each
answers the string `1` or the string `0`. The comparisons rank [below the
concatenation and above `&`](r:expr.index.precedence), normal and strict ones
together. Some of them are spelled in more than one way: `\=`, `<>` and `><`
are one operator, which is true when the operands are not equal, `<=` and `\>`
are one, and so are `>=` and `\<`. The strict operators pair up the same way,
as `\==` is the opposite of `==`, `<<=` is `\>>` and `>>=` is `\<<`.

```rexx
say (3 = 3) (3 \= 3) (3 <> 4) (3 >< 4) (3 < 4) (3 > 4)
say (3 <= 4) (3 \> 4) (3 >= 4) (3 \< 4) (4 >= 4) (4 \< 4)
say ('a' == 'a') ('a' \== 'a') ('a' << 'b') ('a' >> 'b')
say ('a' <<= 'b') ('a' \>> 'b') ('b' >>= 'b') ('b' \<< 'b') ('b' \<< 'a')
say (3 < 4)~length (3 > 4)~length
```

```output
1 0 1 1 1 0
1 1 0 0 1 1
1 0 1 0
1 1 1 1 1
1 1
```

r[expr.comparison.not]
The not sign in `\=` and in the other operators that start with it may be
written as `'AA'x` or `'AC'x`, as [the not rule](r:lex.operators.not) says, so
the spellings above have more than one form. A `\` that does not start one of
these operators is a [logical prefix](r:expr.logical.prefix). A comparison
operator that stands where its first operand should be, as in `(= 1)`, raises
`SYNTAX 35.1`. A program file with that error runs no clause and exits with
`256` minus the error number, as [the read-first rule](r:lex.index.read-first)
says and the second example shows, and the text of an [INTERPRET
instruction](r:instr.interpret.whole) raises it as an ordinary condition, as
the last one shows.

```rexx
say (1 \= 2) (1 \== 1) (1 \< 2) (1 \>> 2)
n = 'aa'x
interpret 'say (1' n'= 2) (1' n'== 1)'
```

```output
1 0 0 1
1 0
```

```rexx
say 'not reached'
say (= 1)
```

```output rc=221
```

```rexx
signal on syntax name e
interpret 'say (= 1)'
e: say condition('o')~code
```

```output
35.1
```

## Numbers or text

r[expr.comparison.kind]
A normal comparison compares its operands as numbers when both are numbers and
as text when either one is not, as [the numeric comparison
rule](r:num.comparison.when) says, and a strict comparison always compares
text. The same two strings may therefore compare unequal as text and equal
as numbers, such as `'1.0'` and `'1'`, and the operator decides which of the
two comparisons is made.

```rexx
say ('1.0' = '1') ('1.0' == '1') ('1.0' = 'x') ('1 2' = '1 2')
say (1 < 'abc') ('abc' < 1) (2 < 10) ('2' << '10')
```

```output
1 0 0 1
1 0 1 0
```

## Text

r[expr.comparison.blanks]
A normal comparison of text first removes the blanks and tabs at the start and
at the end of each operand, and the blanks and tabs inside an operand stay.
The shorter of the two is then extended with blanks to the length of the
longer, so a trailing blank never decides the result. Other characters that
look like blank space, such as the line feed or the carriage return, are not
whitespace and take part like any character, as [the whitespace
rule](r:conv.string.whitespace) says. A string of nothing but blanks and tabs
is equal to the null string.

```rexx
say ('a' = 'a ') ('a' = ' a') ('  a  ' = 'a') ('a b' = 'a  b')
say ('a' = 'a' || '09'x) ('a' = '09'x || 'a') ('a' = 'a' || '0a'x)
say ('' = ' ') ('' = ' ' || '09'x) ('' = '0') ('' = 'a')
say ('ab' < 'ab ') ('ab' < 'ab!') ('ab' < 'ab' || '01'x)
```

```output
1 1 1 0
1 1 0
1 1 0 0
0 1 0
```

r[expr.comparison.bytes]
Text compares character by character, left to right, and the first position at
which the characters differ decides which operand is the lesser, by comparing
the codes of the two characters as unsigned numbers from `0` to `255`. So a
capital letter is lesser than any small one, a character with code above 127
is greater than any code below it, and a comparison does not fold case. The
padding blank of a normal comparison is the character `'20'x`, and it is
compared like any other character, so it is greater than a control character
and lesser than a digit.

```rexx
say ('a' = 'A') ('B' < 'a') ('a' < 'B') ('Z' < 'a')
say ('7F'x < '80'x) ('c3a9'x > 'z') ('ab' > 'ab' || '00'x)
say ('a' < '1') (' ' < '1') ('a1' < 'a2') ('a10' < 'a9')
```

```output
0 1 0 1
1 1 1
0 1 1 1
```

r[expr.comparison.strict]
A strict comparison of two strings neither strips blanks nor pads: the strings
are compared character by character as [the byte
rule](r:expr.comparison.bytes) says, and when one string is a leading part of
the other, the shorter one is the lesser. `==` is true only for two strings of
the same characters, so a blank, a tab and a case difference all make a
difference. The null string is lesser than any other string, a blank included.

```rexx
say ('a' == 'a') ('a' == 'a ') ('a' == 'A') ('' == ' ') ('' == '')
say ('a' << 'a ') ('a ' >> 'a') ('a' << 'ab') ('ab' >> 'a')
say ('' << ' ') ('' << '00'x) ('' <<= '') ('' >>= '') ('' >> 'a')
say ('abc' << 'abd') ('b' << 'ab') ('B' << 'a') ('a' << 'B')
```

```output
1 0 0 0 1
1 1 1 1
1 1 1 1 0
1 0 1 0
```

## Operands that are objects

r[expr.comparison.object]
A comparison is a message to its left operand, as [the operator
rule](r:expr.messages.operator) says, and an object that does not define the
ordering messages raises `SYNTAX 97.1` for `<` and for the other ordering
operators, normal and strict. The equality messages that every object
understands are those of [the default rule](r:expr.messages.default). A string
on the left compares the string that [the conversion
rule](r:conv.string.object) makes of the right operand, which for an instance
of a program's own class that declares no `MAKESTRING`, `request` or `string`
method is the text such as
`an O`. This book does not specify the comparison with `.nil`, because
`'The NIL object' = .nil` is `0` although `.nil` writes that text. The right
operand that is not a string is read once, by the normal comparisons as by the
strict ones ([Deviation 122](r:expr.comparison.deviation.122)).

```rexx
a = .array~new
b = a
say (a = b) (a = .array~new) (a \== b) (a <> .array~new)
call try 'a < b'
call try 'a >> b'
call try '"an O" = .o~new'
call try '"an O" < .o~new'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(1) '=>' r
  return
bad: say arg(1) '=>' condition('o')~code
  return
::class o
```

```output
1 0 0 1
a < b => 97.1
a >> b => 97.1
"an O" = .o~new => 1
"an O" < .o~new => 0
```

r[expr.comparison.deviation.122]
A right operand that is not a string is converted once by every comparison
operator, [the conversion rule](r:conv.string.object) applying to that one
conversion, so that an object counting the conversions is asked once by `=`,
by `<` and by `==` alike. ooRexx converts it twice for the normal comparisons
`=`, `\=`, `<>`, `><`, `<`, `>`, `<=`, `>=`, `\<` and `\>` when the two
operands are not both numbers, and once when they are and for the strict ones.

```rexx
o = .counted~new
.local~n = 0
r = 'a' = o
say .local~n
.local~n = 0
r = 'a' < o
say .local~n
.local~n = 0
r = 'a' == o
say .local~n
.local~n = 0
r = 'a'~">="(o)
say .local~n
::class counted
::method makeString
  .local~n += 1
  return 'x'
```

```output
1
1
1
1
```
