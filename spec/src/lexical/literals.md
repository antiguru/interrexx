---
element: "Literals"
kind: concept
ansi: "6.2.2, 6.2.1.1"
rexxref: "General Concepts > Structure and General Syntax > Tokens"
---

# Literals

A literal is a token whose value is written in the program: a string in
quotes, or a hexadecimal or binary string, which spells a string by the
codes of its characters. All three have a string as their value, and none is
a number or a name. A string never changes when the program runs, whatever
characters it holds. This page gives the forms, the faults that the reader of
a program finds in them, and how a literal sits next to the tokens around it.

## Strings

r[lex.literals.string]
A string is any characters between a pair of the same quote mark, either
`'` or `"`. The value is those characters unchanged: case, blanks and any
byte stay as written. To put the delimiting quote mark inside a string it is
written twice, which stands for one; the other quote mark needs no doubling.
In a program file a string cannot contain a line end.

```rexx
say 'It''s' "say ""hi""" 'say "hi"' "it's"
say length('''') length("""") length('''''')
say 'a  b' '  lead' 'trail  '||'|'
say 'ABC abc'
```

```output
It's say "hi" say "hi" it's
1 1 2
a  b   lead trail  |
ABC abc
```

r[lex.literals.null]
Two quote marks side by side, `''` or `""`, are the null string, a string of
length 0.

```rexx
say length('') length("") '['||''||']'
```

```output
0 0 []
```

r[lex.literals.unmatched]
A string whose closing quote mark is not found before the end of the line is
an error. In the text given to INTERPRET the end of the text takes the place
of the end of the line, as [the rule on its text](r:lex.characters.interpret)
says. It raises `SYNTAX 6.2` for a string opened with `'` and
`SYNTAX 6.3` for one opened with `"`. The fault is lexical, so it is found
when the program is read, as [the chapter](r:lex.index.read-first) says. A
string in a program file never goes on into the next line.

```rexx
say 'before'
say 'abc
say 'after'
```

```output rc=250
```

```rexx
signal on syntax name a
interpret "say 'abc"
a: say condition('o')~code
signal on syntax name b
interpret 'say "abc'
b: say condition('o')~code
signal on syntax name c
interpret "say 'it''"
c: say condition('o')~code
```

```output
6.2
6.3
6.2
```

## Hexadecimal strings

r[lex.literals.hex]
A hexadecimal string is a string of hexadecimal digits, `0` to `9`, `a` to
`f` and `A` to `F`, between quote marks of the same kind, with `x` or `X`
directly after the closing quote mark. Its value is the string of the
characters whose codes the digits spell, two digits to a character, read
from the left. When the digits are odd in number, a `0` is taken before the
first one. The value is a string and not a number: `'31'x` is the character
`1`, and `'41'x` is the character `A`. A hexadecimal string with no digits
is the null string.

```rexx
say '41'x "42"X '4142'x '['||''x||']'
say c2x('123'x) c2x('f'x) c2x('abc'X) c2x('ABC'x)
say '31'x + 1
say length(''x) length(""X)
```

```output
A B AB []
0123 0F 0ABC 0ABC
2
0 0
```

r[lex.literals.hex.blank]
Blanks and horizontal tabs may separate the digits of a hexadecimal string
into groups. Every group after the first must have an even number of digits,
and the first may have any number. The value is that of the digits with the
separators removed. A separator may not come first or last in the string. A
group of digits that must be even and is not raises `SYNTAX 15.5`, and a
separator at the start or the end raises `SYNTAX 15.1`.

```rexx
say c2x('41 42'x) c2x('4 1242'x) c2x('41     42'x)
interpret "say c2x('4" || '09'x || "12'x)"
signal on syntax name a
interpret "say '41 4'x"
a: say condition('o')~code
signal on syntax name b
interpret "say '4 1'x"
b: say condition('o')~code
signal on syntax name c
interpret "say ' 41'x"
c: say condition('o')~code
signal on syntax name d
interpret "say '41 'x"
d: say condition('o')~code
```

```output
4142 041242 4142
0412
15.5
15.5
15.1
15.1
```

r[lex.literals.hex.digit]
A character in a hexadecimal string that is neither a hexadecimal digit nor
a blank or tab raises `SYNTAX 15.3`. The form feed and the vertical tab are
not blanks here either.

```rexx
signal on syntax name a
interpret "say 'g1'x"
a: say condition('o')~code
signal on syntax name b
interpret "say '41" || '0c'x || "42'x"
b: say condition('o')~code
signal on syntax name c
interpret "say '41-42'x"
c: say condition('o')~code
```

```output
15.3
15.3
15.3
```

## Binary strings

r[lex.literals.bin]
A binary string is a string of the digits `0` and `1` between quote marks of
the same kind, with `b` or `B` directly after the closing quote mark. Its
value is the string of the characters whose codes the digits spell, eight
digits to a character, read from the left. When the digits are not a
multiple of eight, zeros are taken before the first one. A binary string
with no digits is the null string.

```rexx
say c2x('11'b) c2x('10100110'b) c2x("01110001"B) c2x('1101'b)
say c2x('1000000000'b) c2x('1111 0000 1111'b)
say length(''b) length(""B)
```

```output
03 A6 71 0D
0200 0F0F
0 0
```

r[lex.literals.bin.blank]
Blanks and horizontal tabs may separate the digits of a binary string into
groups. Every group after the first must have a number of digits that is a
multiple of four, and the first may have any number. The value is that of
the digits with the separators removed. A separator may not come first or
last in the string. A group that must be a multiple of four and is not
raises `SYNTAX 15.6`, and a separator at the start or the end raises
`SYNTAX 15.2`.

```rexx
say c2x('1 0110'b) c2x('110 11001100'b) c2x('1 0011 0111'b)
interpret "say c2x('1" || '09'x || "0000'b)"
signal on syntax name a
interpret "say '1 1'b"
a: say condition('o')~code
signal on syntax name b
interpret "say '11 11'b"
b: say condition('o')~code
signal on syntax name c
interpret "say ' 1111'b"
c: say condition('o')~code
signal on syntax name d
interpret "say '1111 'b"
d: say condition('o')~code
```

```output
16 06CC 0137
10
15.6
15.6
15.2
15.2
```

r[lex.literals.bin.digit]
A character in a binary string that is neither `0`, `1`, a blank nor a tab
raises `SYNTAX 15.4`.

```rexx
signal on syntax name a
interpret "say '102'b"
a: say condition('o')~code
signal on syntax name b
interpret "say 'a'b"
b: say condition('o')~code
```

```output
15.4
15.4
```

r[lex.literals.digit.order]
When the digits of a hexadecimal or binary string have more than one fault,
the error raised is the first of these that applies: a separator at the start
or the end, then a group of the wrong size, then a character that is not a
digit. All three are found when the program is read.

```rexx
signal on syntax name a
interpret "say ' g'x"
a: say condition('o')~code
signal on syntax name b
interpret "say '4 g'x"
b: say condition('o')~code
signal on syntax name c
interpret "say '4 1g'x"
c: say condition('o')~code
signal on syntax name d
interpret "say '1 2'b"
d: say condition('o')~code
signal on syntax name e
interpret "say ' 2'b"
e: say condition('o')~code
signal on syntax name f
interpret "say '12'b"
f: say condition('o')~code
```

```output
15.1
15.5
15.3
15.6
15.2
15.4
```

## Next to other tokens

r[lex.literals.suffix]
The letter after the closing quote mark is the suffix of a hexadecimal or
binary string only when no symbol character follows it. The suffix `x` or
`X`, or `b` or `B`, with a symbol character after it, or a blank before it,
is a symbol or the start of one, and the quoted text before it is a plain
string. A comment between the closing quote mark and the letter has the same
effect, as it ends the string.

```rexx
x = 'v'
say '41'x '41' x '41'xx '41'x1 c2x('1'b) '1' b '1'bb
say '41'/**/x
```

```output
A 41 v 41XX 41X1 01 1 B 1BB
41v
```

r[lex.literals.adjacent]
A string, a hexadecimal string or a binary string that touches a symbol or a
number, on either side, is a token of its own. Nothing is added between the
two tokens, so they join as two terms with no blank between them.

```rexx
say 'a'b1 1'b' 'q'.5 x'y'
```

```output
aB1 1b q.5 Xy
```

r[lex.literals.function]
A string that is directly followed by `(`, with no blank between, is the
name of a function and not a value. The name is the characters of the string
as written, with no change of case, so the name of a built-in function is
found only when it is written in upper case. With a blank before the `(`
the string is a value, and the parentheses enclose a term of their own. A
comment between the string and the `(` does not matter, as for a symbol that
names a function.

```rexx
say 'ABS'(-3) 'ABS' (-3) 'ABS'/**/(-3) abs/**/(-3)
signal on syntax name e
say 'abs'(-3)
e: say condition('o')~code
```

```output
3 ABS -3 3 3
43.1
```
