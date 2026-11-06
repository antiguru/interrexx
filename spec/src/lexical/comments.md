---
element: "Comments"
kind: concept
ansi: "6.2.2"
rexxref: "General Concepts > Structure and General Syntax > Comments"
---

# Comments

A comment is text that the reader of a program skips. Rexx has two kinds: a
block comment, written between `/*` and `*/`, which may span lines and nest,
and a line comment, which starts with `--` and runs to the end of the line.
A comment is not a token, but it ends the token before it, so it separates
a symbol, a number or a string from what follows. Comments never reach the
later stages of the interpreter, and no rule on another page depends on
their text.

r[lex.comments.block]
A block comment starts at `/*` and ends at the matching `*/`. It may hold
any characters, and it may sit between any two tokens,
or at the start or end of a line, or on a line of its own.

```rexx
/* first */ say 'a' /* second */ ; /* third */
say 'b' /* the comment */
/* only a comment */
say 'c'
```

```output
a
b
c
```

r[lex.comments.nest]
Block comments nest. Every `/*` inside a block comment opens a comment of
its own, and the outer comment ends only at the `*/` that closes the
outermost level.

```rexx
say 1 /* outer /* inner */ still outer */ + 2
say 3 /* a /* b /* c */ d */ e */
```

```output
3
3
```

r[lex.comments.unmatched]
A block comment that is still open at the end of the program is an error:
`SYNTAX 6.1`. Because of nesting, a comment with one more `/*` than `*/` is
such a case. The fault is lexical, so it is found when the program is read,
as [the chapter](r:lex.index.read-first) says.

```rexx
say 'before'
say 'x' /* open /* nested */ still open
say 'after'
```

```output rc=250
```

```rexx
signal on syntax name e
interpret "say 1 /* open"
e: say condition('o')~code
```

```output
6.1
```

r[lex.comments.separator]
A comment ends the symbol, number or string before it and begins the one
after it, and it is not a blank. Two terms with only a comment between them
are therefore joined without a blank, and a blank on either side of the
comment, or on both, is the blank of the join. A comment cannot sit inside a
symbol, a number or a string: it splits a symbol or number that surrounds it
into two. Between two operator characters a comment is dropped as a blank
is, so the characters still form one operator.

```rexx
ab = 'whole'; a = 'p'; b = 'q'
say a/**/b a /**/b a/**/ b a /**/ b
say ab 1/**/2 1 /**/ 2
say 1 |/**/| 2 2 */**/* 3
```

```output
pq p q p q p q
whole 12 1 2
12 8
```

r[lex.comments.string]
Inside a string, `/*` and `--` are ordinary characters and start no
comment. Inside a block comment, quote marks, `--` and the characters of a
string are ordinary characters, so a quote mark never has to be balanced in
a comment.

```rexx
say '/* not a comment */' '--' "-- no"
say 'a' /* it's "quoted" -- and a line comment */ 'b'
```

```output
/* not a comment */ -- -- no
a b
```

r[lex.comments.span]
In a program file, a block comment may span lines, and a line end inside a
comment is not the end of a clause. The clause that was open before the
comment goes on after its `*/`. A line end after the `*/` ends the clause
as usual.

```rexx
say 1 /* the comment
   goes on */ say 2
say 3 /* one line */
say 4
```

```output
1 SAY 2
3
4
```

r[lex.comments.line]
A line comment starts at `--` and ends at the end of the line. The line end
that follows it is still the end of the clause. In the text given to
[INTERPRET](r:lex.characters.interpret) there are no lines, and the comment
runs to the end of the text. Within the comment a `/*`
starts no block comment. Where a block comment is open, a `--` in it starts
nothing, and the comment goes on to its `*/` even on the same line. Two
minus signs that are separated, even by a blank, are not a line comment.

```rexx
say 'a' -- the rest /* is ignored
say 'b' /* in the comment -- */ 'c'
say 5 - - 2
say 5 --2
interpret "say 6 -- c" || '0a'x || "say 7"
```

```output
a
b c
7
5
6
```

r[lex.comments.shebang]
When a program file starts with the two bytes `#!`, its first line is
ignored, as a line comment is. The two bytes are recognized only at the very
start of the file. In any other place a `#` is outside the
[character set](r:lex.characters.set).

```rexx
#!/usr/bin/env rexx
say 'ok'
```

```output
ok
```
