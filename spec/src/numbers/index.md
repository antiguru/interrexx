---
element: "Numbers"
kind: chapter
ansi: "6.2.2.35, 7"
rexxref: "Numbers and Arithmetic > Numbers"
---

# Numbers

This chapter specifies what a number is, how a number is written when an
operation produces one, and the limits on its size. Rexx has no numeric
type: a number is a string whose characters have a certain form, and the
same string is a number or not depending only on that form. Operations that
need a number read the string, compute on its value, and write the answer
as a new string. The chapter describes the strings and the answers, and
leaves the operators and the settings that steer them to the pages named at
the end.

A value is checked against the form each time an operation needs a number,
and no check or conversion happens before that. The pages of this chapter
are:

* [Formats](formats.md): how an arithmetic result is written, with
  significant digits, zeros, exponential notation and engineering notation.
* [Limits](limits.md): the range of exponents, overflow and underflow, and
  the limits that whole numbers have.
* [Arithmetic](arithmetic.md): the operators, the rounding to NUMERIC DIGITS,
  integer division, powers and the errors of each.
* [Settings](settings.md): NUMERIC DIGITS, FUZZ and FORM, how a program
  changes them and which part of the program a change reaches.
* [Comparison](comparison.md): when two values compare as numbers, and how
  NUMERIC FUZZ widens equality.

## What a number is

r[num.index.string]
A number is a string, and it stays the string it was written as until an
operation answers with a new one. A variable that holds `1.50` holds those
four characters, and so does a string that spells `007`. Only the form of
the characters makes either one a number, and a string that does not have
the form is an ordinary string that happens to hold digits or letters.

```rexx
x = 1.50
y = '007'
say x y length(x) length(y)
say x + 0 y + 0
say datatype(x) datatype(y) datatype('seven')
```

```output
1.50 007 4 3
1.50 7
NUM NUM CHAR
```

r[num.index.form]
A string is a number when it consists of these parts in this order. First
comes optional whitespace, then an optional sign, `+` or `-`, which may be
followed by whitespace. Next come the digits `0` to `9`, with at most one
period before, among or after them and at least one digit in all, and last an
optional exponent, which is `E` or `e` directly after the digits, then an
optional sign, then one or more digits, with nothing between those parts.
Optional whitespace may end the string. [Whitespace](r:conv.string.whitespace)
is blanks and horizontal tabs only, so no other control character may stand
anywhere in a number, and no whitespace may stand between two digits, before
the `E`, or inside the exponent.

```rexx
say datatype(' 7.25 ', 'N') datatype('+ 0.5', 'N'),
    datatype('-' || '09'x || '12', 'N') datatype('3.', 'N'),
    datatype('.8', 'N') datatype('6E2', 'N') datatype('0.4e+3', 'N'),
    datatype('2e-0', 'N') datatype('00', 'N')
say datatype('', 'N') datatype(' ', 'N') datatype('.', 'N'),
    datatype('4 5', 'N') datatype('1e', 'N') datatype('1e+', 'N'),
    datatype('1 e3', 'N') datatype('--3', 'N') datatype('3-', 'N'),
    datatype('1.2.3', 'N')
say datatype('0b101', 'N') datatype('1,5', 'N') datatype('1_000', 'N'),
    datatype('1e3.5', 'N') datatype('e5', 'N') datatype('+ -3', 'N'),
    datatype('1' || '0a'x, 'N') datatype('1' || '0d'x, 'N'),
    datatype('1' || 'a0'x, 'N') datatype('1' || '0b'x, 'N')
```

```output
1 1 1 1 1 1 1 1 1
0 0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0 0
```

> **Note.** The same form is the form of a [whole-number
> argument](r:conv.whole.form), which has a further condition on its value,
> and the form that DATATYPE tests with type `N`, as
> [its rule](r:bif.datatype.number) says. A number written without quotes
> in a program is a constant symbol, and
> [the symbol rules](r:lex.symbols.number) say which constant symbols have
> this form. A number out of the range of
> [exponents](r:num.limits.range) has the form but is not a number. The ANSI
> draft (6.2.2.35 and 7) allows blanks only as whitespace, where ooRexx also
> allows the tab.

r[num.index.value]
The value of a number is its digits read as a decimal numeral, with the
period as the decimal point, multiplied by ten to the power of the exponent,
and negated when the sign is `-`. Leading zeros, zeros after the point, the
choice between `1e2` and `100`, and whitespace around the sign change the
text but not the value. A number whose digits are all zero has the value
zero, however it is written and whatever its sign.

```rexx
say ('0012.500' = '125e-1') ('.125e2' = '12.5') ('-0' = '0'),
    ('1e0' = '1.000')
say ('+ 5' = '5') ('- 5' = '-5') ('-' || '09'x || '5' = '-5')
say ('1e2' = '100') ('1e2' = '1e+2') ('5e-1' = '0.5')
```

```output
1 1 1 1
1 1 1
1 1 1
```

r[num.index.use]
An operation that needs a number checks its operand against
[the form](r:num.index.form) when it runs, and the error depends on the
operation. An operand of an arithmetic operator that is not a number raises
`SYNTAX 41.1`, the null string and a string of only whitespace included. A
[number argument](r:bif.index.num.arg) of a builtin raises `SYNTAX 93.943`
for the same strings, and a [whole-number argument](r:conv.whole.form) raises
`SYNTAX 40.12`. A string that is not used as a number is never checked.

```rexx
signal on syntax name a
say 'seven' + 1
a: say condition('o')~code
signal on syntax name b
say abs('seven')
b: say condition('o')~code
signal on syntax name c
say substr('abc', 'seven')
c: say condition('o')~code
signal on syntax name d
say 4 * '4 5'
d: say condition('o')~code
signal on syntax name e
say -''
e: say condition('o')~code
signal on syntax name f
say 5 + ' '
f: say condition('o')~code
say length('seven') 'seven' || 3
```

```output
41.1
93.943
40.12
41.1
41.1
41.1
5 seven3
```
