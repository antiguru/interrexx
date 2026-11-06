---
element: X2B
kind: builtin
syntax: "X2B(hex_string)"
ansi: "9.6.9"
rexxref: "Functions > Built-in Functions > X2B (Hexadecimal to Binary)"
deviations: [4]
---

# X2B

X2B answers the binary digits that stand for a string of hexadecimal
digits: four binary digits for each hexadecimal digit.

## Arguments

r[bif.x2b.method]
X2B is a builtin that runs as a String method, and `hex_string`, its only
argument, is the receiver. The argument count is checked by the
[builtin call rules](r:bif.index.call.count): one is required and one is the
most.

```rexx
say x2b('A') 'A'~x2b
signal on syntax name a
say x2b()
a: say condition('o')~code
signal on syntax name b
say x2b('a', 'b')
b: say condition('o')~code
```

```output
1010 1010
40.3
40.4
```

r[bif.x2b.digits]
`hex_string` is a [string of hexadecimal digits](r:bif.index.conv.digits),
which may have [whitespace](r:bif.index.conv.groups) between its digits
and not at [its ends](r:bif.index.conv.groups.edge), and which may be
[null](r:bif.index.conv.digits.null). The errors those rules give are
those of a hexadecimal string: `SYNTAX 93.933` for a character that is not a
digit, `SYNTAX 93.931` for whitespace at an end and `SYNTAX 93.976` for a gap
at a place that is not a multiple of 2 digits from the end.

```rexx
say x2b('4a 4B') x2b('4' || '09'x || '4a')
signal on syntax name a
say x2b('4g')
a: say condition('o')~code
signal on syntax name b
say x2b('4a ')
b: say condition('o')~code
signal on syntax name c
say x2b('4 a')
c: say condition('o')~code
```

```output
0100101001001011 010001001010
93.933
93.931
93.976
```

## The result

r[bif.x2b.result]
The answer has four binary digits for each hexadecimal digit of `hex_string`,
in order: the digits of the hexadecimal digit's value, with `0` digits on
the left to make four. Leading zeros are kept, a letter has the answer of its
capital, and whitespace in `hex_string` leaves no trace in the answer.

```rexx
say x2b('0') x2b('1') x2b('9') x2b('a') x2b('F') x2b('00ff') x2b('d9')
say length(x2b('1 23 45')) x2b('1 23')
```

```output
0000 0001 1001 1010 1111 0000000011111111 11011001
20 000100100011
```

r[bif.x2b.null]
When `hex_string` is the null string, the answer is the null string.

```rexx
say '['x2b('')']' length(x2b(''))
```

```output
[] 0
```

r[bif.x2b.not-number]
X2B reads the characters of its argument and not its value as a number, so
an argument written as a number is treated as the digits it is written with
and NUMERIC DIGITS plays no part.

```rexx
numeric digits 2
say x2b(101) x2b(10) x2b(0) x2b(0010)
```

```output
000100000001 00010000 0000 0000000000010000
```

r[bif.x2b.inverse]
B2X undoes X2B: the answer of X2B, given to B2X, is `hex_string` with its
letters in capitals and its whitespace removed.

```rexx
say b2x(x2b('a5f')) b2x(x2b('00 Ff')) '['b2x(x2b(''))']'
```

```output
A5F 00FF []
```

r[bif.x2b.deviation.4]
interrexx does not specify whether the answer of X2B is the same object as
another string of equal value, as
[the shared rule](r:bif.index.conv.deviation.4) says for every conversion
function. A program can rely only on the value,
which is equal under `==`.

```rexx
say (x2b('6') == '0110') (x2b('') == '') (x2b('f0') == '11110000')
```

```output
1 1 1
```
