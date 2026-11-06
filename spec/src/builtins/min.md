---
element: MIN
kind: builtin
syntax: "MIN(number [, number]...)"
ansi: "9.4.4"
rexxref: "Functions > Built-in Functions > MIN (Minimum)"
---

# MIN

MIN answers the least of a list of numbers, rounded to NUMERIC DIGITS
digits.

## Arguments

r[bif.min.method]
MIN is a builtin that runs as a String method, and its first number is the
receiver; the other numbers become the message's arguments, in their order.
The numbers, their checks and the argument count are those of
[a list of numbers](r:bif.index.num.list): at least one number is required
and there is no most.

```rexx
say min(3, 8, 5) 3~min(8, 5) '3'~min('8', '5') (9)~min
signal on syntax name a
say min()
a: say condition('o')~code
signal on syntax name b
say '3'~min(8, 'five')
b: say condition('o')~code
signal on syntax name c
say 'three'~min(8)
c: say condition('o')~code
```

```output
3 3 3 9
40.3
93.904
93.943
```

## The result

r[bif.min.result]
MIN answers the number of the list that no other number of the list is
less than, picked as the [list rule](r:bif.index.num.pick) says with a
later number replacing the candidate when the candidate is greater than it.
The answer is therefore [rounded](r:bif.index.num.round) to NUMERIC DIGITS
digits and [written](r:bif.index.num.write) in plain or scientific notation,
and its sign is kept.

```rexx
say min(12, 6, 7, 9) min(-7, -3, -4.3) min('+2', '1E1', '09')
say min(' 0.0 ', '-0.000') min(-0.5)
numeric digits 3
say min(123456, 5) min(0.01234, 0.01233) min(-1000, -999.9)
numeric form engineering
say min(123456, 1234567) min(-123456, -5)
```

```output
6 -7 2
0 -0.5
5 0.0123 -1.00E+3
123E+3 -123E+3
```

r[bif.min.rounding]
The numbers are rounded before they are compared, so two numbers that round
to the same value are equal, and MIN answers the earlier of them. This is
seen in the written form, which keeps the decimal places of the number
picked.

```rexx
numeric digits 3
say min(1.00, 1.0004) min(1.0004, 1.00) min(1.0, 1.0004) min(1.0004, 1.0)
say min(1.234, 1.236) min(1.2349, 1.2351)
```

```output
1.00 1.00 1.0 1.00
1.23 1.23
```

r[bif.min.fuzz]
Numbers that NUMERIC FUZZ makes equal are equal to MIN, which then answers
the earlier of them, as the [list rule](r:bif.index.num.pick) says. With
NUMERIC FUZZ 0, the setting at the start of a program, only numbers of the
same value are equal.

```rexx
numeric digits 3
numeric fuzz 1
say min(1.04, 1.01) min(1.01, 1.04) min(1.04, 1.09) min(1.09, 1.04)
say min(1.04, 1.01, 1.09) min(1.09, 1.04, 1.01)
numeric fuzz 0
say min(1.04, 1.01) min(1.01, 1.04)
```

```output
1.04 1.01 1.04 1.04
1.04 1.04
1.01 1.01
```
