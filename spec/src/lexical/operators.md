---
element: "Operators and special characters"
kind: concept
ansi: "6.2.2"
rexxref: "General Concepts > Structure and General Syntax > Tokens"
---

# Operators and special characters

An operator character is one that can be part of an operator, and a special
character is one that stands alone as a token, such as a parenthesis. Both
kinds end the token before them, so no blank is needed beside them. Several
operators are written with two or three characters, and the characters of
one operator may be spread over blanks and comments. This page gives the
operators as tokens, the way characters are joined into them, and the way
blanks next to these characters are treated. What an operator computes is
the business of the numbers and expressions chapters.

r[lex.operators.set]
The operators are these tokens. The arithmetic operators are `+` `-` `*`
`/` `%` `//` and `**`. The concatenation operator is `||`. The comparison
operators are `=` `\=` `>` `<` `>=` `<=` `<>` `><` `\>` and `\<`, and the
strict comparison operators `==` `\==` `>>` `<<` `>>=` `<<=` `\>>` and
`\<<`. The logical operators are `&` `|` and `&&`. The characters `+` `-`
and `\` are also prefix operators. The blank and the abuttal of two terms
are operators too, but they are not tokens, as
[the blank rule](r:lex.operators.blank) explains.

```rexx
say 7 + 2 7 - 2 7 * 2 7 / 2 7 % 2 7 // 2 7 ** 2
say (2>1) (2<1) (2>=1) (2<=1) (2<>1) (2><1) (2\>1) (2\<1) (2=2) (2\=2)
say (2==2) (2\==2) (2>>1) (2<<1) (2>>=1) (2<<=1) (2\>>1) (2\<<1)
say (1&0) (1|0) (1&&0) (\0) (-3) (+3) 'a'||'b'
```

```output
9 5 14 3.5 3 1 49
1 0 1 0 1 1 0 1 1 0
1 0 1 0 1 0 0 1
0 1 1 1 -3 3 ab
```

r[lex.operators.longest]
A run of operator characters is split from the left into the longest
operators of [the set](r:lex.operators.set) that fit. A run that cannot be
split that way, because a character other than `<` or `>` is left over with
no operand, such as `&&&`, `====`, `\>=` or `<==`, raises `SYNTAX 35.1`. A
single `<` or `>` that is left over begins a variable reference term
instead. That term takes a symbol, and the clause raises `SYNTAX 20.930` when
none follows. A leftover `<<` or `>>` is an operator with no left operand and
raises `SYNTAX 35.1`, as `1 =<< 2` does.

```rexx
x = 5
say (1<=-1) (1==-1) (2>-1) (1=\0) (1=>x)
call try '1 => 2'
call try '1 &&& 0'
call try '1 ==== 1'
call try '1 \>= 0'
call try '1 <== 0'
call try '1 =<< 2'
exit
try:
  signal on syntax name bad
  interpret 'say' arg(1)
  return
bad: say condition('o')~code
  return
```

```output
0 0 1 1 0
20.930
35.1
35.1
35.1
35.1
35.1
```

r[lex.operators.gap]
The characters of a two-character or three-character operator may be
separated by blanks, tabs and comments. The separators are dropped, and the
run of operator characters that remains is split as
[the longest rule](r:lex.operators.longest) says. So `> =` is `>=` and
`= =` is `==`, while `- -` is two prefix minus signs and `\ \` two nots,
because `--` and `\\` are no operators, and `1 > = - 1` is `>=` followed by
a prefix minus.

```rexx
say (2 > = 1) (2 = = 2) (2 \ = 1) (2 > < 1) (2 < /* c */ < 3)
say 2 | | 3 1 |/**/| 2
say (- - 1) (\ \ 0) (+ + 4) (1 > = - 1) (1 \ = \ 1)
```

```output
1 1 1 1 1
23 12
1 0 4 1 1
```

r[lex.operators.comment]
The two characters `/*` always open a comment, so an operator ends before
them. A `/` before a comment is the division operator, and `//*` is not the
remainder operator followed by `*`. The comment is then a separator between
characters as [the gap rule](r:lex.operators.gap) describes.

```rexx
say 3 */**/* 2
say 7 //**/ 2
say 7 / /* c */ / 2
```

```output
9
3.5
1
```

r[lex.operators.not]
Wherever the backslash starts an operator, as the prefix not or in `\=`,
`\==`, `\<`, `\>`, `\<<` and `\>>`, the byte `'AA'x` or `'AC'x` may stand
in its place, as [the not rule](r:lex.characters.not) says.

```rexx
n = 'aa'x 'ac'x
do i = 1 to 2
  c = word(n, i)
  interpret 'say (' || c || '0) (1' c'= 2) (1' c'== 2)',
            '(1' c'<< 2) (1' c'>> 2)'
end
```

```output
1 1 1 0 1
1 1 1 0 1
```

## Special characters

r[lex.operators.special]
The special characters are the comma, the semicolon, the colon, the
parentheses `(` and `)`, the brackets `[` and `]`, and the tilde. Each is a
token that ends the token before it. The tilde is the message operator `~`,
and two tildes `~~` are the cascade operator. As the characters of
[any operator](r:lex.operators.gap) may be, the two tildes may be separated
by blanks and comments.

```rexx
say 'abc'~length 'abc'~~length 'abc'~ ~length 'abc'~/**/~length
```

```output
3 abc abc abc
```

r[lex.operators.blank]
A blank, or a run of blanks and tabs, between two terms is the concatenation
operator. A blank next to an operator, a comma or a tilde has no effect. So
does a blank after an opening `(` or `[` and one before a closing `)` or `]`.
A comment is [no blank](r:lex.comments.separator).

```rexx
say 'a'   'b' 'a'||'b' 'a'/**/'b'
say 1+1 1  +  1
say left( 'abc' , 2 ) 'abc' ~ left( 2 )
a = .array~of(5, 6)
say a[ 2 ]
```

```output
a b ab ab
2 2
ab ab
6
```

r[lex.operators.adjacent]
A `(` that follows a symbol or a string with no blank between them opens the
arguments of a function call, and a `[` that follows a term with no blank
between them opens a subscript. A `(` that follows the name in a message term
in the same way holds the arguments of the message. A comment between them
is [no blank](r:lex.comments.separator). With a blank, a `(` starts a
parenthesized term that is concatenated with the term before it, so that a
message is sent without arguments, and a `[` is an error, `SYNTAX 35.1`. A
`(` after a `)` with no blank between them concatenates the two terms without
a blank.

```rexx
say abs(-1) abs/**/(-1) abs (-1) abs /**/(-1)
say (1)(2) (1) (2) (1)/**/(2)
a = .array~of(5, 6)
say a[2] a/**/[2] 'abc'~left(2) 'abc'~left/**/(2)
signal on syntax name e
interpret 'say a [2]'
e: say condition('o')~code
signal on syntax name f
interpret "say 'abc'~left (2)"
f: say condition('o')~code
```

```output
1 1 ABS -1 ABS -1
12 1 2 12
6 6 ab ab
35.1
93.903
```
