---
element: X2C
kind: builtin
syntax: "X2C(hex_string)"
ansi: "9.6.10"
rexxref: "Functions > Built-in Functions > X2C (Hexadecimal to Character)"
deviations: [4]
---

# X2C

X2C answers the string whose characters have the codes that a string of
hexadecimal digits writes: one character for each two digits.

## Arguments

r[bif.x2c.method]
X2C is a builtin that runs as a String method, and `hex_string`, its only
argument, is the receiver. The argument count is checked by the
[builtin call rules](r:bif.index.call.count): one is required and one is the
most.

```rexx
say x2c('41') '41'~x2c
signal on syntax name a
say x2c()
a: say condition('o')~code
signal on syntax name b
say x2c('41', '42')
b: say condition('o')~code
```

```output
A A
40.3
40.4
```

r[bif.x2c.digits]
`hex_string` is a [string of hexadecimal digits](r:bif.index.conv.digits),
which may have [whitespace](r:bif.index.conv.groups) between its digits
and not at [its ends](r:bif.index.conv.groups.edge), and which may be
[null](r:bif.index.conv.digits.null). The errors those rules give are
those of a hexadecimal string: `SYNTAX 93.933` for a character that is not a
digit, `SYNTAX 93.931` for whitespace at an end and `SYNTAX 93.976` for a gap
at a place that is not a multiple of 2 digits from the end.

```rexx
say x2c('41 42') x2c('41' || '09'x || '42')
signal on syntax name a
say x2c('4g')
a: say condition('o')~code
signal on syntax name b
say x2c('41 ')
b: say condition('o')~code
signal on syntax name c
say x2c('4 1')
c: say condition('o')~code
```

```output
AB AB
93.933
93.931
93.976
```

## The result

r[bif.x2c.result]
The answer has one character for each 2 digits of `hex_string`, taken from the
right: the last 2 digits, ignoring whitespace, make the last character, the 2
before them the one before that, and so on. The character's code is the value
of its 2 digits, from `'00'x` to `'FF'x`, and a letter is read in either
case.

```rexx
say x2c('48656c6C6f') c2x(x2c('00ff7F80')) length(x2c('41 42 43'))
say x2c('4a 4B') x2c('4A4b')
```

```output
Hello 00FF7F80 3
JK JK
```

r[bif.x2c.partial]
When the number of digits is odd, the leftmost group has one digit and is
read as if a `0` digit stood on its left. The answer has `ceil(n/2)`
characters for `n` digits.

```rexx
say c2x(x2c('f')) c2x(x2c('abc')) c2x(x2c('1 4142')) c2x(x2c('0'))
say length(x2c('12345'))
```

```output
0F 0ABC 014142 00
3
```

r[bif.x2c.null]
When `hex_string` is the null string, the answer is the null string.

```rexx
say '['x2c('')']' length(x2c(''))
```

```output
[] 0
```

r[bif.x2c.not-number]
X2C reads the characters of its argument and not its value as a number, so
an argument written as a number is treated as the digits it is written with
and NUMERIC DIGITS plays no part.

```rexx
numeric digits 2
say x2c(41) x2c(414243) c2x(x2c(0010))
```

```output
A ABC 0010
```

r[bif.x2c.inverse]
C2X undoes X2C when the number of digits is even: the answer of X2C, given to
C2X, is `hex_string` with its letters in capitals and its whitespace removed.
When the number is odd, it has a `0` on its left as well.

```rexx
say c2x(x2c('a5f0')) c2x(x2c('00 Ff')) c2x(x2c('a5f')) '['c2x(x2c(''))']'
```

```output
A5F0 00FF 0A5F []
```

r[bif.x2c.deviation.4]
interrexx does not specify whether the answer of X2C is the same object as
another string of equal value, as [the shared rule](r:bif.index.conv.deviation.4)
says for every conversion function. A program can rely only on the value,
which is equal under `==`.

```rexx
say (x2c('61') == 'a') (x2c('') == '') (x2c('6162') == 'ab')
```

```output
1 1 1
```
