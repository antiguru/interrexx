---
element: "Numeric comparison"
kind: concept
ansi: "7.4.7, 7.4.10"
rexxref: "Numbers and Arithmetic > Numeric Comparisons"
deviations: [53]
---

# Numeric comparison

A comparison operator compares two values, and the language has two ways to do
it: as numbers or as text. This page says when the operator compares numbers,
how two numbers compare under NUMERIC DIGITS and NUMERIC FUZZ, and which
condition a comparison can raise. What the operators are called and how text
is compared, with its blank padding and its order of characters, belongs to
[the expressions chapter](../expressions/comparison.md).

## Numbers or text

r[num.comparison.when]
A normal comparison, which is `=`, `<`, `>` and the operators made from them
and from `\`, compares its operands as numbers when both are
[numbers](r:num.index.form) and as text when either is not. A string whose
exponent is out of [range](r:num.limits.not-number) is not a number, so it
compares as text. A strict comparison, which is `==`, `<<`, `>>` and the
operators made from them, always compares as text, so it tells two numbers of
equal value apart when their written forms differ. No comparison raises an
error for a string that is not a number.

```rexx
say (1 = 1.0) (1 == 1.0) ('1e2' = 100) ('1e2' == 100) (' 7 ' = 7) (' 7 ' == 7)
say ('10' < '9') ('10' << '9') ('10' < '9a') ('10' < 'x') (1 = 'x') ('' = 0)
say ('1e1000000000' < '2') (1e5 < 2) ('0x10' = 16) ('1,000' = 1000)
```

```output
1 0 1 0 1 0
0 1 1 1 0 0
1 0 0 0
```

## Comparing two numbers

r[num.comparison.numeric]
Two numbers compare as the sign of their difference. The difference is the one
that [the rule for `-`](r:num.arithmetic.addsub) gives, calculated at NUMERIC
DIGITS minus NUMERIC FUZZ digits, and the numbers are equal when it is zero,
the first is the lesser when it is negative and the greater when it is
positive. So the numbers are equal when that difference vanishes in the
rounding, which tolerates a small difference in the digits beyond the
precision, counted from the higher first digit of the two, and the values need
not be exactly equal. The difference is taken as the rule for `-` computes it
and its range is not checked, so a comparison raises no `SYNTAX 42.901` or
`SYNTAX 42.902` and an overflow of the difference does not change the answer.
Zero is equal to zero in any form, and no number is equal to zero except zero,
however small it is.

```rexx
say (1.000000001 = 1) (1.00000001 = 1) (1 < 1.00000001) (1 > 1.000000001)
say (123456789 = 123456790) (1234567890 = 1234567891),
    (1234567890 = 1234567895)
say (0 = -0) (0 = 0.00) (0 < 1e-999999999) (1e-999999999 = 0)
say (9e999999999 > -9e999999999) (9e999999999 = -9e999999999)
numeric digits 3
say (1234 = 1233) (1234 < 1233) (1234 <= 1233) (1234 \= 1233) (1234 == 1233)
say (1234 = 1240) (1234 = 1250) (1234 < 1250) (1234 < 1260)
say (1000 = 1004) (1000 = 1005) (100 = 100.4) (100 = 100.5),
    (0.001234 = 0.001233)
```

```output
1 0 1 0
0 1 0
1 1 1 0
1 0
1 0 1 0 0
0 0 1 1
1 0 1 0 1
```

## Fuzz

r[num.comparison.fuzz]
NUMERIC FUZZ widens the tolerance of [the comparison
rule](r:num.comparison.numeric): each unit of it lets a difference ten times
as large vanish in the rounding. It acts on comparisons alone: arithmetic
still works at NUMERIC DIGITS. Equality under a fuzz is not transitive, since
each pair is judged by the difference of the two and not through a third
number. The builtins that pick one of several numbers use [the same
test](r:bif.index.num.pick), and a strict comparison is unaffected because it
compares text.

```rexx
numeric digits 3
numeric fuzz 1
say (100 = 104) (100 = 105) (100 = 99) (1.00 = 1.04) (1.00 = 1.05)
say (100 < 104) (100 <= 104) (100 > 104) (100 >= 104) (100 == 104)
say (1000 = 1049) (1000 = 1050) (104 = 108) (100 = 108)
say (1.00 + 0.04) max(1.00, 1.04) (1.00 = 1.04)
numeric fuzz 0
say (100 = 104) (1.00 = 1.04)
```

```output
1 0 1 1 0
0 1 0 1 0
1 0 1 0
1.04 1.00 1
0 0
```

## Lost digits

r[num.comparison.lostdigits]
When a normal comparison compares two numbers, an operand with more digits
than NUMERIC DIGITS minus NUMERIC FUZZ raises the LOSTDIGITS condition. The
digits are counted as [the arithmetic rule](r:num.arithmetic.lostdigits)
counts them, and the condition is raised for the left operand first. A strict
comparison raises nothing, and neither does a comparison of text. When the
condition is not trapped the comparison goes on and gives its answer.

```rexx
numeric digits 3
signal on lostdigits name a
say 1234 = 1
a: say condition('c') condition('d')
numeric fuzz 1
signal on lostdigits name b
say 123 = 124
b: say condition('d')
signal on lostdigits name c
say 12 = 12
say (123 == 123) ('1234x' = '1234') (1234 < 'x')
c: say 'done'
```

```output
LOSTDIGITS 1234
123
1
1 0 1
done
```

## Deviations

r[num.comparison.deviation.53]
A comparison of two numbers under a NUMERIC FUZZ above zero checks both
operands for LOSTDIGITS whatever their signs and whether or not one of them is
zero. ooRexx does not check when the signs differ or when an operand is zero,
and answers the comparison. It checks every pair when NUMERIC FUZZ is 0.

```rexx
numeric digits 3
numeric fuzz 1
signal on lostdigits name a
say 123 = 0
a: say condition('d')
signal on lostdigits name b
say 123 > -1
b: say condition('d')
```

```output
123
123
```
