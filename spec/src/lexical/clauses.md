---
element: "Clauses"
kind: concept
ansi: "6.2.3, 6.4.3"
rexxref: "General Concepts > Structure and General Syntax"
---

# Clauses

A clause is the unit that the interpreter runs: a sequence of tokens that
ends at a semicolon, which the reader supplies at most line ends. This page
says where a clause ends, which clauses do nothing, how a label is
recognized, how a line is continued, and how the first tokens of a clause
decide whether it is a label, an assignment, an instruction or a command.
The line ends of this page are those of a program file. The text given to
INTERPRET ends its lines in ways that this book does not specify, so
examples of INTERPRET here hold no line end.

r[lex.clauses.end]
A clause ends at a semicolon, at a line end, or at the end of the program.
A semicolon inside a string or a comment ends nothing. A line end inside a
comment ends nothing either, and neither does a line end after a
[continuation character](r:lex.clauses.continuation). Several clauses may
share a line when semicolons separate them.

```rexx
say 1; say 2; say ';' /* ; */
say 3
say 4 /* a comment
   across lines */ ; say 5
say 6
```

```output
1
2
;
3
4
5
6
```

r[lex.clauses.null]
A clause with no tokens is a null clause. Blank lines, lines that hold only
comments and repeated semicolons are null clauses, and a null clause does
nothing. Where an instruction is required after THEN, ELSE or OTHERWISE, the
null clauses in between are passed over and the next clause that has tokens
is that instruction.

```rexx
say 1;;;say 2

/* only a comment */
; ;
say 3
if 1 then ; say 4
if 0 then nop; else
  ; say 5
```

```output
1
2
3
4
5
```

r[lex.clauses.implied]
The keywords THEN, ELSE and OTHERWISE end the clause they are in, so that
the instruction after one may follow on the same line with no semicolon.
They have this effect only where they are keywords: THEN after the
expression of IF or WHEN, and ELSE and OTHERWISE as the first token of a
clause. Anywhere else they are ordinary symbols.

```rexx
if 1 then say 'a'; else say 'b'
if 0 then say 'c'
else say 'd'
select; when 0 then say 'e'; otherwise say 'f'; end
say 1 then say 2 else say 3 otherwise say 4
then = 5; else = 6; otherwise = 7
say then else otherwise
```

```output
a
d
f
1 THEN SAY 2 ELSE SAY 3 OTHERWISE SAY 4
5 6 7
```

## Labels

r[lex.clauses.label]
A clause that is a single symbol or a single string followed by a colon is a
label. Blanks and comments may stand before the colon, and the colon implies
a semicolon, so another clause, or another label, may follow on the same
line. A label does nothing when it is reached. A symbol label is read in
upper case, and a string label keeps its characters as written, so a label
written as a lower-case string is not found by a name that is written as a
symbol. Any kind of [symbol](r:lex.symbols.kinds) is a label, a number and a
symbol that starts with a period included. When two labels have the same
name, the first is the one that is found. The colon makes a label before it
can make an assignment, so `x: = 3` is the label `x` followed by the clause
`= 3`, which raises `SYNTAX 35.1` when the program is read, so that the
program produces no output and ends with the exit code 221.

```rexx
go : /* c */ say 'a'
1: say 'b'
.x: y.1: say 'c'
"quoted": say 'd'
signal on syntax name e
signal go2
say 'skipped'
"go2": say 'string label, not found'
Go2: say 'found'
d: say 'first'; d: say 'second'
signal stop
"Stop": say 'string label, as written'
e: say condition('o')~code
```

```output
a
b
c
d
found
first
second
16.1
```

```rexx
say 'before'
x: = 3
```

```output rc=221
```

## Continuation

r[lex.clauses.continuation]
A comma or a minus sign that is the last token on a line, followed by
nothing but blanks, tabs and comments, continues the clause on the next
line. The character is replaced by a blank, and no semicolon is implied. A
comment after it may span lines, and the line end after the comment is the
one that counts. The next line is joined whole, so when it is empty the
clause ends at its line end. At the end of a program, the character just
ends the clause. A `--` is a [line comment](r:lex.comments.line) and not a
continuation, and a minus sign that is not the last token is an operator.

```rexx
say 1,
    2
say 3,    /* c */   -- c
    4
say 5 -
    6
say 7 - -
    8
say 'a'-
'b'
say 9 --
say 10,

say 11
say 12 -  /* spans
   lines */
   13
say 14,
```

```output
1 2
3 4
5 6
-1
a b
9
10
11
12 13
14
```

r[lex.clauses.continuation.comma]
A comma that continues a line is not an argument separator: it is replaced
by a blank like any other continuation. To continue a call after a
separator, the separator is written first, so that the comma at the end of
the line is the continuation. Two commas in a row at the end of a line are
a separator and a continuation.

```rexx
say words(1,
          2)
say right('a', 3,,
          '-')
call show 1,
          2
call show 1,,
          2
exit
show: say arg() '['arg(1)']' '['arg(2)']'
return
```

```output
2
--a
1 [1 2] []
2 [1] [2]
```

## What a clause is

r[lex.clauses.kinds]
A clause that is not a null clause or a label is an assignment, an
instruction or a command, and the first tokens decide which. If the first
token is a symbol and the second is `=`, the clause is an
[assignment](r:lex.clauses.assignment). Otherwise, if the first token is a
symbol that is an [instruction keyword](r:lex.clauses.keyword), the clause
is that instruction. Any other clause is a [command](r:lex.clauses.command).

```rexx
signal on failure name f
address nosuch
say = 'assigned'
say say
say(2)
'a command'
f: say condition('d')
```

```output
assigned
2
a command
```
r[lex.clauses.assignment]
A clause whose first token is a symbol and whose second token is `=` is an
assignment, whatever the symbol is, so the names of instructions and the
keywords THEN, ELSE and OTHERWISE can be variables. Blanks and comments may
stand around the `=`. The expression is everything after it up to the end of
the clause, and an empty one raises `SYNTAX 35.918`.

```rexx
say = 3; say say
do = 'd'; if = 'i'; then = 't'; else = 'e'; otherwise = 'o'
say do if then else otherwise
x. = 0; x.say = 5; say x.say
x=/**/ 7; say x
signal on syntax name e
interpret 'v ='
e: say condition('o')~code
```

```output
3
d i t e o
5
7
35.918
```

r[lex.clauses.assignment.op]
The tokens `+=` `-=` `*=` `/=` `%=` `//=` `||=` `&=` `|=` `&&=` and `**=`
start an assignment too, when a symbol comes first. `x op= expression` assigns
the result of `x op (expression)`, so the whole expression is evaluated before
the operator is applied. The characters of the token may be separated as those
of [any operator](r:lex.operators.gap) may be.

```rexx
x = 10; x += 5; x -= 3; x *= 2; x /= 4; x %= 2; say x
x //= 2; say x
x = 2; x **= 3; say x
y = 'a'; y ||= 'b' 'c'; say y
n = 2; n *= 2 + 3; say n
t = 1; t + = 1; say t
b = 1; b &= 0; say b; b |= 1; say b; b &&= 1; say b
```

```output
3
1
8
ab c
10
2
0
1
0
```

r[lex.clauses.keyword]
When a clause is not an assignment and its first token is a symbol without a
period that is the name of an instruction, in either case, the clause is
that instruction. The keyword may be followed directly by a parenthesis, a
quote mark or a sign. A symbol with a period, as `say.1` is, is not a
keyword. The instructions and their keywords are the subject of the
instructions chapter.

```rexx
say(1)
say"a"
say-1
Say 'mixed case'
say.1 = 'stem'; say say.1
```

```output
1
a
-1
mixed case
stem
```

r[lex.clauses.command]
A clause that is not a label, an assignment or an instruction is a command.
Its expression is evaluated, and the resulting string is handed to the
current environment. How that is done is the business of the ADDRESS
instruction, and here only the recognition matters.

```rexx
signal on failure name f
address nosuch
x = 4
x + 1
f: say condition('c') condition('d')
```

```output
FAILURE 5
```
