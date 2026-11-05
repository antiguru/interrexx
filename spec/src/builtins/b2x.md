---
element: B2X
kind: builtin
syntax: "B2X(binary_string)"
ansi: "9.6.1"
rexxref: "Functions > Built-in Functions > B2X (Binary to Hexadecimal)"
deviations: [4]
---

# B2X

B2X answers the hexadecimal digits that stand for a string of binary
digits: four binary digits for each hexadecimal digit, in capital letters.

## Arguments

r[bif.b2x.method]
B2X is a builtin that runs as a String method, and `binary_string`, its
only argument, is the receiver. The argument count is checked by the
[builtin call rules](r:bif.index.call.count): one is required and one is the
most.

```rexx
say b2x('1010') '1010'~b2x
signal on syntax name a
say b2x()
a: say condition('o')~code
signal on syntax name b
say b2x('1', '1')
b: say condition('o')~code
```

```output
A A
40.3
40.4
```

r[bif.b2x.digits]
`binary_string` is a [string of binary digits](r:bif.index.conv.digits),
which may have [whitespace](r:bif.index.conv.groups) between its digits
and not at [its ends](r:bif.index.conv.groups.edge), and which may be
[null](r:bif.index.conv.digits.null). The errors those rules give are
those of a binary string: `SYNTAX 93.934` for a character that is not a
digit, `SYNTAX 93.932` for whitespace at an end and `SYNTAX 93.977` for a gap
at a place that is not a multiple of 4 digits from the end.

```rexx
say b2x('1 1111 0000') b2x('0001' || '09'x || '1111')
signal on syntax name a
say b2x('12')
a: say condition('o')~code
signal on syntax name b
say b2x('1111 ')
b: say condition('o')~code
signal on syntax name c
say b2x('1 11')
c: say condition('o')~code
```

```output
1F0 1F
93.934
93.932
93.977
```

## The result

r[bif.b2x.result]
The answer has one hexadecimal digit for each 4 binary digits of
`binary_string`, taken from the right: the last 4 digits, ignoring
whitespace, make the last hexadecimal digit, the 4 before them the one
before that, and so on. A group of 4 digits is written as the hexadecimal
digit of its value, `0` to `9` and `A` to `F`, with the letters in
capitals. Leading zero digits are kept.

```rexx
say b2x('11000011') b2x('00001010') b2x('0000') b2x('1111 0001 0010')
say b2x('0000000011111111')
```

```output
C3 0A 0 F12
00FF
```

r[bif.b2x.partial]
When the number of binary digits is not a multiple of 4, the leftmost group
has fewer than 4 digits and is read as if enough `0` digits stood on its
left to make it 4. The answer has `ceil(n/4)` hexadecimal digits for `n`
digits.

```rexx
say b2x('1') b2x('10') b2x('101') b2x('10111') b2x('1 0000 0000')
say length(b2x(copies('1', 9)))
```

```output
1 2 5 17 100
3
```

r[bif.b2x.null]
When `binary_string` is the null string, the answer is the null string.

```rexx
say '['b2x('')']' length(b2x(''))
```

```output
[] 0
```

r[bif.b2x.not-number]
B2X reads the characters of its argument and not its value as a number, so
an argument written as a number is treated as the digits it is written with
and NUMERIC DIGITS plays no part.

```rexx
numeric digits 2
say b2x(101) b2x(1111) b2x(0) b2x(0001)
```

```output
5 F 0 1
```

r[bif.b2x.deviation.4]
interrexx does not specify whether the answer of B2X is the same object as
another string of equal value, as [the shared rule](r:bif.index.conv.deviation.4)
says for every conversion function. A program can rely only on the value,
which is equal under `==`.

```rexx
say (b2x('0110') == '6') (b2x('') == '') (b2x('1' || '0000') == '10')
```

```output
1 1 1
```
