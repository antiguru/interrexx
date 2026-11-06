---
element: "Characters"
kind: concept
ansi: "6.2.1.3"
rexxref: "General Concepts > Structure and General Syntax > Characters"
---

# Characters

A program is a sequence of bytes, and this page says which bytes it may hold
and what each one means to the reader of the program. The set of characters
that may stand between tokens is small: letters, digits, a few marks and the
blank. Strings and comments may hold any byte at all. Case matters inside
strings only, and blanks matter only in where they separate tokens.

r[lex.characters.bytes]
The text of a program is read as bytes, with no decoding. A character in a
string or in a comment is one byte, so a character that UTF-8 writes as two
bytes is [two characters](r:conv.string.chars) in a string, and `'00'x` and
the values from `'80'x` upward may appear there. In a program file, a
comment may hold any byte except `'1A'x`, and a string any byte except
`'1A'x` and the line-end bytes `'0A'x` and `'0D'x`, as
[line ends](r:lex.characters.line-end) and
[the end-of-file byte](r:lex.characters.eof) say. In the text given to
INTERPRET a string or comment may hold any byte, as
[its rule](r:lex.characters.interpret) says.

```rexx
say length('é') c2x('é')
interpret "say length('a" || '00'x || "b')"
interpret "say c2x('" || '80ff'x || "') length(/* " || '00'x || " */'')"
```

```output
2 C3A9
3
80FF 0
```

r[lex.characters.set]
Outside strings and comments, a program may hold only these characters: the
letters `A` to `Z` and `a` to `z`, the digits `0` to `9`, the marks
`.` `!` `?` `_`, the blank and the horizontal tab, the quote marks `'` and
`"`, the bytes `'AA'x` and `'AC'x` ([the not sign](r:lex.characters.not)),
and the operator and special characters
`& ( ) * + , - / : ; < = > \ | % ~ [ ]`. Any other byte there, among them
`#`, `$`, `@`, `^`, the backquote, the braces, `'7F'x` and the control
characters other than the tab, raises `SYNTAX 13.1`. So does every byte from
`'80'x` upward other than `'AA'x` and `'AC'x`. Inside a string or a comment
none of them is an error. In a program file the line ends and `'1A'x` are
not characters of the text but [line ends](r:lex.characters.line-end) and
[its end](r:lex.characters.eof). In INTERPRET text they are outside the set
[like any other control character](r:lex.characters.interpret).

```rexx
x.1 = 'v'; a! = 1; b? = 2; c_ = 3
say (7 + 1) * 2 - 9 % 4 // 5 / 2 || 'x'
say 1 < 2 & 2 > 1 | 0
say \0 = 1 \== 2
say 'abc'~length .array~of(5, 6)[2]
say x.1 a! b? c_,
    'end'
say '# $ @ ^ { }' /* # $ @ ^ { } */
```

```output
15x
1
1
3 6
v 1 2 3 end
# $ @ ^ { }
```

```rexx
signal on syntax name a
interpret "say 1 #"
a: say condition('o')~code
signal on syntax name b
interpret "x = a@b"
b: say condition('o')~code
signal on syntax name c
interpret "say 1" || '80'x
c: say condition('o')~code
signal on syntax name d
interpret "say 1" || '00'x
d: say condition('o')~code
signal on syntax name e
interpret "say 'é' é"
e: say condition('o')~code
```

```output
13.1
13.1
13.1
13.1
13.1
```

r[lex.characters.not]
The bytes `'AA'x` and `'AC'x` are accepted wherever the not operator `\` is,
and have its meaning. The two-byte UTF-8 form of the not sign, `'C2AC'x`, is
not a not operator: its first byte is outside the
[character set](r:lex.characters.set).

```rexx
interpret "say 1" || 'aa'x || "= 2"
interpret "say 1" || 'ac'x || "= 2"
interpret "say" 'ac'x || "0"
signal on syntax name e
interpret "say 1 ¬= 2"
e: say condition('o')~code
```

```output
1
1
1
13.1
```

r[lex.characters.case]
A symbol is translated to upper case when the program is read, so its
letters may be written in either case. A number is a symbol and is only
translated, not normalised: `1e3` becomes `1E3`, not `1000`. The same holds
for the keywords of
an instruction, for labels and for the names of variables. The characters of
a string are never translated.

```rexx
Abc = 'Val'
say ABC abc aBc
SAY 'Abc'
x.Y = 1
say X.y
say 'abc' == 'ABC'
say 1e3 1.5e2 .5e1
signal LaBel
say 'skipped'
label: say 'at the label'
```

```output
Val Val Val
Abc
1
0
1E3 1.5E2 .5E1
at the label
```

r[lex.characters.blank]
The blank, `'20'x`, and the horizontal tab, `'09'x`, are the whitespace
characters of a program, as [elsewhere](r:conv.string.whitespace), and a tab
is read exactly like a blank. Any number of them between two tokens has the
effect of one. Within a string they are ordinary characters. The form feed
and the vertical tab are not whitespace: outside a string or comment they
raise `SYNTAX 13.1`, as every character outside the
[character set](r:lex.characters.set) does.

```rexx
interpret "say 1" || '09'x || "+" || '09'x || "2"
interpret "say 'a'" || '09'x || '09'x || "'b'"
say length('a' || '09'x || 'b')
signal on syntax name e
interpret "say 1" || '0c'x || "+ 2"
e: say condition('o')~code
signal on syntax name f
interpret "say 1" || '0b'x || "+ 2"
f: say condition('o')~code
```

```output
3
a b
3
13.1
13.1
```

r[lex.characters.line-end]
In a program file, a line ends at a line feed (`'0A'x`), at a carriage
return (`'0D'x`), or at a carriage return followed by a line feed, which is
one line end. The last line need not end. A line end inside a string is the
fault `SYNTAX 6.2` or `6.3`, and a comment may hold line ends. The text
given to INTERPRET has no line ends, as
[its rule](r:lex.characters.interpret) says.

```rexx
f = 'spec-lex-lines.tmp'
call sysfiledelete f
call charout f, "say 'a'" || '0d'x || "say 'b'" || '0d0a'x || "say 'c'"
call charout f, '0a'x || "say 'd'"
call charout f
call (f)
call sysfiledelete f
```

```output
a
b
c
d
```

r[lex.characters.eof]
In a program file, the byte `'1A'x` ends the program: it and every byte
after it are not read, even inside a string or a comment, so a string or
comment still open at that byte is unmatched. The byte also ends a clause
that it follows directly.

```rexx
f = 'spec-lex-eof.tmp'
call sysfiledelete f
call charout f, "say 'a'" || '0a'x || '1a'x || "say 'b'" || '0a'x
call charout f
call (f)
call sysfiledelete f
```

```output
a
```

```rexx
f = 'spec-lex-eof2.tmp'
call sysfiledelete f
call charout f, "say 'a" || '1a'x || "b'" || '0a'x
call charout f
call (f)
call sysfiledelete f
```

```output rc=250
```

```rexx
f = 'spec-lex-eof3.tmp'
call sysfiledelete f
call charout f, "say 'a'" || '1a'x || "say 'b'" || '0a'x
call charout f
call (f)
call sysfiledelete f
```

```output
a
```

r[lex.characters.interpret]
The text given to INTERPRET is not a file and has no lines. In it the bytes
`'0A'x`, `'0D'x` and `'1A'x` are not line ends and not an end of text.
Outside a string or a comment they are outside the
[character set](r:lex.characters.set) and raise `SYNTAX 13.1`, as does any
other control character. Inside a string or a comment they are ordinary
characters, so a string there may hold them and a line comment runs to the
end of the text. A string or comment that is still open at the end of the
text is unmatched.

```rexx
lf = '0a'x
say length('a' || lf || 'b')
interpret "say 'a" || lf || "b'"
interpret "say 1 /* x" || lf || "y */ + 1"
interpret "say 2 -- c" || lf || "say 3"
signal on syntax name a
interpret "say 1" || lf || "say 2"
a: say condition('o')~code
signal on syntax name b
interpret "say 1" || '0d'x || "say 2"
b: say condition('o')~code
signal on syntax name c
interpret "say 1" || '1a'x
c: say condition('o')~code
```

```output
3
a
b
2
2
13.1
13.1
13.1
```
