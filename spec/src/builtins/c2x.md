---
element: C2X
kind: builtin
syntax: "C2X(string)"
ansi: "9.6.6"
rexxref: "Functions > Built-in Functions > C2X (Character to Hexadecimal)"
deviations: [4]
---

# C2X

C2X answers the hexadecimal digits that stand for the characters of a
string: two for each character, in capital letters.

## Arguments

r[bif.c2x.method]
C2X is a builtin that runs as a String method, and `string`, its only
argument, is the receiver. `string` is a [string](r:conv.string.null), so no
value of it raises an error. The argument count is checked by the
[builtin call rules](r:bif.index.call.count): one is required and one is the
most.

```rexx
say c2x('A') 'A'~c2x
signal on syntax name a
say c2x()
a: say condition('o')~code
signal on syntax name b
say c2x('a', 'b')
b: say condition('o')~code
```

```output
41 41
40.3
40.4
```

## The result

r[bif.c2x.result]
The answer has two hexadecimal digits for each character of `string`, in
order. The digits of a character are those of its code, from `'00'x` to
`'FF'x`: the code's value divided by 16, then its remainder, each as `0`
to `9` or `A` to `F` in capitals.

```rexx
say c2x('4567'x) c2x('7f80ff'x) c2x('Qz9') length(c2x('abc'))
```

```output
4567 7F80FF 517A39 6
```

r[bif.c2x.any-character]
Every character is converted like any other: `'00'x`, whitespace, line feed
and the characters above `'7F'x` included. A string of digits is converted as
the characters it consists of, not as a number, so NUMERIC DIGITS plays no
part and the answer for a number is that of the string it is written as.

```rexx
numeric digits 3
say c2x('00'x) c2x(' ' || '09'x || '0a'x) c2x(12345) c2x(-1) c2x(1.50) c2x(1e5)
```

```output
00 20090A 3132333435 2D31 312E3530 314535
```

r[bif.c2x.null]
When `string` is the null string, the answer is the null string.

```rexx
say '['c2x('')']' length(c2x(''))
```

```output
[] 0
```

r[bif.c2x.inverse]
X2C turns the answer back into `string`: converting the answer of C2X
with X2C gives `string` itself, for every string.

```rexx
s = 'Mixed' || '00ff'x || ' case'
say (x2c(c2x(s)) == s) (x2c(c2x('')) == '')
```

```output
1 1
```

## Deviations

r[bif.c2x.deviation.4]
interrexx does not specify whether the answer of C2X is the same object as
another string of equal value, as [the shared
rule](r:bif.index.conv.deviation.4) says for every conversion function. A
program can rely only on the value, which is equal under `==`.

```rexx
say (c2x('a') == '61') (c2x('') == '') (c2x('ab') == '6162')
```

```output
1 1 1
```
