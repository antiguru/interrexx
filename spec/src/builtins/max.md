---
element: MAX
kind: builtin
syntax: "MAX(number [, number]...)"
ansi: "9.4.3"
rexxref: "Functions > Built-in Functions > MAX (Maximum)"
---

# MAX

MAX answers the greatest of a list of numbers, rounded to NUMERIC DIGITS
digits.

## Arguments

r[bif.max.method]
MAX is a builtin that runs as a String method, and its first number is the
receiver; the other numbers become the message's arguments, in their order.
The numbers, their checks and the argument count are those of
[a list of numbers](r:bif.index.num.list): at least one number is required
and there is no most.

```rexx
say max(3, 8, 5) 3~max(8, 5) '3'~max('8', '5') (3)~max
signal on syntax name a
say max()
a: say condition('o')~code
signal on syntax name b
say '3'~max(8, 'five')
b: say condition('o')~code
signal on syntax name c
say 'three'~max(8)
c: say condition('o')~code
```

```output
8 8 8 3
40.3
93.904
93.943
```

## The result

r[bif.max.result]
MAX answers the number of the list that no other number of the list is
greater than, picked as the [list rule](r:bif.index.num.pick) says with a
later number replacing the candidate when the candidate is less than it. The
answer is therefore [rounded](r:bif.index.num.round) to NUMERIC DIGITS digits
and [written](r:bif.index.num.write) in plain or scientific notation, and its
sign is kept.

```rexx
say max(12, 6, 7, 9) max(-7, -3, -4.3) max('+2', '1E1', '09')
say max(' 0.0 ', '-0.000') max(-0.5)
numeric digits 3
say max(123456, 5) max(0.01234, 0.01233) max(1000, 999.9)
numeric form engineering
say max(123456, 5) max(-123456, -1234567)
```

```output
12 -3 10
0 -0.5
1.23E+5 0.0123 1.00E+3
123E+3 -123E+3
```

r[bif.max.rounding]
The numbers are rounded before they are compared, so two numbers that round
to the same value are equal, and MAX answers the earlier of them. This is
seen in the written form, which keeps the decimal places of the number
picked.

```rexx
numeric digits 3
say max(1.00, 1.0004) max(1.0004, 1.00) max(1.0, 1.0004) max(1.0004, 1.0)
say max(1.234, 1.236) max(1.2349, 1.2351)
```

```output
1.00 1.00 1.0 1.00
1.24 1.24
```

r[bif.max.fuzz]
Numbers that NUMERIC FUZZ makes equal are equal to MAX, which then answers
the earlier of them, as the [list rule](r:bif.index.num.pick) says. With
NUMERIC FUZZ 0, the setting at the start of a program, only numbers of the
same value are equal.

```rexx
numeric digits 3
numeric fuzz 1
say max(1.04, 1.01) max(1.01, 1.04) max(1.04, 1.09) max(1.09, 1.04)
say max(1.04, 1.01, 1.09) max(1.09, 1.04, 1.01)
numeric fuzz 0
say max(1.04, 1.01) max(1.01, 1.04)
```

```output
1.04 1.01 1.09 1.09
1.09 1.09
1.04 1.04
```
