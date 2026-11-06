---
element: DATATYPE
kind: builtin
syntax: "DATATYPE(string [, type])"
ansi: "9.3.8"
rexxref: "Functions > Built-in Functions > DATATYPE"
---

# DATATYPE

DATATYPE examines the characters of a string. Called with the string alone,
it names the string `NUM` when it is a number and `CHAR` when it is not;
called with a type, it answers `1` when the string is of that type and `0`
when it is not. The types are about the form of the string, never about its
meaning: they ask whether it is a number, a whole number, a string of
hexadecimal or of binary digits, a string of letters or of letters and
digits, a symbol, or a logical value.

## Arguments

r[bif.datatype.method]
DATATYPE is a builtin that runs as a String method, and `string`, its first
argument, is the receiver; `type` becomes the message's argument. `string` is
a [string](r:conv.string.null) and may be the null string. The argument count
is checked by the [builtin call rules](r:bif.index.call.count): one argument
is required and two is the most.

```rexx
say datatype('12') '12'~datatype '12'~datatype('W') datatype('', 'X')
signal on syntax name a
say datatype()
a: say condition('o')~code
signal on syntax name b
say datatype(, 'N')
b: say condition('o')~code
signal on syntax name c
say datatype('1', 'N', 2)
c: say condition('o')~code
```

```output
NUM NUM 1 1
40.3
40.5
40.4
```

r[bif.datatype.type]
`type` selects the test by its first character, in either case, and the
characters after the first are not looked at. The letters that select a
test are `A`, `B`, `I`, `L`, `M`, `N`, `O`, `S`, `U`, `V`, `W` and `X`, and
the digit `9`. A `type` whose first character is another character,
including a blank, or a `type` that is the null string, raises
`SYNTAX 93.915`. The `type` is checked before the string is examined, so it
raises the error for every `string`, the null string included.

```rexx
say datatype('12', 'n') datatype('12', 'Number') datatype('12', 'Nonsense'),
    datatype('abc', 'Xyz')
signal on syntax name a
say datatype('', 'Q')
a: say condition('o')~code
signal on syntax name b
say datatype('1', ' N')
b: say condition('o')~code
signal on syntax name c
say datatype('1', '')
c: say condition('o')~code
signal on syntax name d
say datatype('1', 'C')
d: say condition('o')~code
```

```output
1 1 1 1
93.915
93.915
93.915
93.915
```

r[bif.datatype.omitted]
When `type` is omitted, DATATYPE answers `NUM` if the string is a number
as [the number test](r:bif.datatype.number) defines it, and `CHAR` if it is
not; a `type` that is [omitted](r:conv.arg.omitted) behaves as an absent one.

```rexx
say datatype('  7.5 ') datatype('') datatype('1 2') datatype('12', ) datatype('x1', )
```

```output
NUM CHAR CHAR NUM CHAR
```

## Numbers

r[bif.datatype.number]
Type `N` answers `1` when the string is a number: a string of the
[whole-number form](r:conv.whole.form) whose value is within the [exponent
range](r:bif.index.num.limit) of the language. The strings that
[a number argument](r:bif.index.num.arg) rejects with `SYNTAX 93.943` are
exactly the strings for which `N` answers `0`: the null string, a string of
only whitespace, a string with a character that the form does
not allow, and a number whose exponent is out of range. The digits of the
string are not rounded and NUMERIC DIGITS plays no part.

```rexx
say datatype(' + 2.50E1 ', 'N') datatype('.5', 'N') datatype('5.', 'N'),
    datatype('- 7', 'N')
say datatype('', 'N') datatype(' ', 'N') datatype('1 2', 'N') datatype('1e', 'N'),
    datatype('0x1', 'N') datatype('..5', 'N') datatype('1' || '0a'x, 'N')
say datatype('1e999999999', 'N') datatype('1e1000000000', 'N'),
    datatype('12.3e999999999', 'N') datatype('1.1e-999999999', 'N')
numeric digits 2
say datatype('123456789.123', 'N')
```

```output
1 1 1 1
0 0 0 0 0 0 0
1 0 0 0
1
```

r[bif.datatype.whole]
Type `W` answers `1` when the string is a number that is a whole number at
the current NUMERIC DIGITS. The number is [rounded](r:bif.index.num.round) to
NUMERIC DIGITS significant digits first. It is whole when the rounded value has
no decimal places other than zeros, which are ignored, and its integer part has
no more digits than NUMERIC DIGITS, so that a whole value too large to be
written in full at that setting is not whole. A string that is not a number is
not whole. Zero, where it is a number, is whole however it is written.

```rexx
say datatype('12', 'W') datatype('1.0', 'W') datatype('1e3', 'W'),
    datatype('100e-2', 'W') datatype('-0.0', 'W') datatype('12.5', 'W'),
    datatype('1e-3', 'W') datatype('abc', 'W') datatype('', 'W')
numeric digits 3
say datatype('1.0004', 'W') datatype('123.4', 'W') datatype('999.6', 'W'),
    datatype('1234', 'W') datatype('12345', 'W') datatype('1e5', 'W'),
    datatype('0.001E4', 'W')
numeric digits 1
say datatype('9', 'W') datatype('10', 'W') datatype('0.99', 'W'),
    datatype('15e-1', 'W') datatype('0.5', 'W')
```

```output
1 1 1 1 1 0 0 0 0
1 1 0 0 0 0 1
1 0 1 1 0
```

r[bif.datatype.nine]
Type `9` is type `W` with NUMERIC DIGITS taken as 9, whatever the setting in
force: it answers `1` when the string is a number that rounds to a whole
number of at most nine digits.

```rexx
say datatype('123456789', '9') datatype('1234567890', '9') datatype('999999999.4', '9'),
    datatype('999999999.5', '9') datatype('1e8', '9') datatype('1e9', '9'),
    datatype('1.5', '9')
numeric digits 3
say datatype('123456', '9') datatype('123456', 'W')
numeric digits 30
say datatype('1234567890', '9') datatype('1234567890', 'W')
```

```output
1 0 1 0 1 0 0
1 0
0 1
```

r[bif.datatype.internal]
Type `I` answers `1` when the string is acceptable as a whole-number argument,
as [the conventions' rule](r:conv.whole.digits) gives it (stream positions and
BEEP have their own rules). The caller's NUMERIC DIGITS setting plays no part
in that rule, so `I` can answer `1` for a number that `W` finds too large for
the current setting, and `0` for a number that `W` finds whole because of
rounding to a short setting.

```rexx
say datatype('123456789012345678', 'I') datatype('1000000000000000000', 'I'),
    datatype('-5.00', 'I') datatype('1e17', 'I') datatype('1.0004', 'I'),
    datatype('x', 'I') datatype('', 'I')
numeric digits 3
say datatype('12345678901', 'I') datatype('12345678901', 'W'),
    datatype('1.0004', 'I') datatype('1.0004', 'W')
```

```output
1 0 1 1 0 0 0
1 0 0 1
```

## Digit strings

r[bif.datatype.hex]
Type `X` answers `1` when the string is a string of hexadecimal digits, as
[the digit rule](r:bif.index.conv.digits) defines the digits, and where a
conversion function would raise an error for the string, `X` answers `0`:
whitespace in the string only where [the group rule](r:bif.index.conv.groups)
allows a gap, so that every group after the first has an even count of digits;
no whitespace at the start or the end; and a character that is neither a digit
nor whitespace. The null string is a string of hexadecimal digits, while a
string of only whitespace is not. A string with an odd number of digits is
hexadecimal when it has no gap.

```rexx
say datatype('4a4B', 'X') datatype('4a 4B', 'X') datatype('4 4a4b', 'X'),
    datatype('abcdef 12', 'X') datatype('abc', 'X') datatype('', 'X')
say datatype('a bcde', 'X') datatype('abc def', 'X') datatype('a b', 'X'),
    datatype(' 4a', 'X') datatype('4a ', 'X') datatype(' ', 'X') datatype('4g', 'X'),
    datatype('4a' || '0a'x || '4b', 'X') datatype('4a' || '09'x || '4b', 'X')
say datatype('ab cdef', 'X') datatype('a bcdef', 'X')
```

```output
1 1 1 1 1 1
1 0 0 0 0 0 0 0 1
1 0
```

r[bif.datatype.binary]
Type `B` answers `1` when the string is a string of binary digits, with the
same allowances as `X`: whitespace only where [the group
rule](r:bif.index.conv.groups) allows a gap, so that every group after the
first has a count of digits that is a multiple of four, none at the start or
the end, and no other character. The null string is a string of binary digits,
a string of only whitespace is not. A string with a count of digits that is
not a multiple of four is binary when it has no gap.

```rexx
say datatype('0101', 'B') datatype('1 0101', 'B') datatype('11111 1111', 'B'),
    datatype('010', 'B') datatype('', 'B')
say datatype('0101 11', 'B') datatype('01 01', 'B') datatype('0101 ', 'B'),
    datatype(' 0101', 'B') datatype('01012', 'B') datatype(' ', 'B'),
    datatype('0101' || '0a'x || '1111', 'B') datatype('0101' || '09'x || '1111', 'B')
say datatype('1 01010101', 'B') datatype('1 0101010', 'B')
```

```output
1 1 1 1 1
0 0 0 0 0 0 0 1
1 0
```

## Letters, digits and symbols

r[bif.datatype.letters]
The letters here are the 26 letters of the English alphabet in either case,
and the digits are `0` to `9`; no other character is a letter or a digit,
whatever the string holds, accents and the characters of other alphabets
included. Type `A` answers `1` when the string has at least one character
and only letters and digits. Type `M` answers `1` when it has at least one
character and only letters. Type `L` answers `1` when it has at least one
character and only lowercase letters, and type `U` when it has at least one
and only uppercase letters. The null string is therefore none of these.

```rexx
say datatype('aB3', 'A') datatype('aB', 'M') datatype('abc', 'L') datatype('ABC', 'U')
say datatype('aB3', 'M') datatype('aB', 'L') datatype('aB', 'U') datatype('a_b', 'A'),
    datatype('a b', 'M') datatype('A' || 'e9'x, 'A') datatype('é', 'M')
say datatype('', 'A') datatype('', 'M') datatype('', 'L') datatype('', 'U')
```

```output
1 1 1 1
0 0 0 0 0 0 0
0 0 0 0
```

r[bif.datatype.logical]
Type `O` answers `1` only for the strings `0` and `1`, each of one
character.

```rexx
say datatype('0', 'O') datatype('1', 'O') datatype('10', 'O') datatype('01', 'O'),
    datatype(' 1', 'O') datatype('', 'O') datatype('true', 'O')
```

```output
1 1 0 0 0 0 0
```

r[bif.datatype.symbol]
Type `S` answers `1` when the string is a symbol that
[SYMBOL](r:bif.symbol.result) does not call `BAD`: it has at least one
character, and each is a letter, a digit, or one of `.`, `!`, `?` and `_`, and
it has at most 250 characters; a longer string is not a symbol, whatever its
characters. The other characters are not symbol characters, among them `$`,
`@`, `#`, whitespace and the null character. Case does not matter, and the
string is not translated or looked up as a variable.

```rexx
say datatype('abc', 'S') datatype('A.b', 'S') datatype('5', 'S') datatype('.', 'S'),
    datatype('a!?_', 'S') datatype('1e3', 'S')
say datatype('', 'S') datatype('a b', 'S') datatype('a-b', 'S') datatype('$a', 'S'),
    datatype('a#', 'S') datatype('+5', 'S') datatype('"a"', 'S')
say datatype(copies('a', 250), 'S') datatype(copies('a', 251), 'S'),
    datatype(copies('a', 251), 'V')
```

```output
1 1 1 1 1 1
0 0 0 0 0 0 0
1 0 0
```

r[bif.datatype.variable]
Type `V` answers `1` when the string is a symbol, as type `S` says, whose
first character is neither a digit nor a period. A string that passes is a
symbol that can name a variable: a simple one, a stem such as `a.`, or a
compound.

```rexx
say datatype('abc', 'V') datatype('a1', 'V') datatype('a.b.c', 'V') datatype('a.', 'V'),
    datatype('_a', 'V') datatype('!a', 'V') datatype('?a', 'V')
say datatype('1a', 'V') datatype('.a', 'V') datatype('.', 'V') datatype('5.', 'V'),
    datatype('a b', 'V') datatype('', 'V')
```

```output
1 1 1 1 1 1 1
0 0 0 0 0 0
```

## Other forms of the arguments

r[bif.datatype.object]
A `string` that is an object other than a string is replaced by a string as
[any string argument](r:conv.string.object) says, and so is a `type`.

```rexx
say datatype(.mutablebuffer~new('12'), 'N') datatype('12', .mutablebuffer~new('Whole'))
say datatype(.array~of(1, 2), 'S') datatype(.array~of(1, 2), 'A')
```

```output
1 1
0 0
```
