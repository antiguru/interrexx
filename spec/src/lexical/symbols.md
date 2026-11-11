---
element: "Symbols"
kind: concept
ansi: "6.2.2, 6.2.1.1, 6.2.1.2, 7.2"
rexxref: "General Concepts > Structure and General Syntax > Tokens"
---

# Symbols

A symbol is a token written without quotes from letters, digits and a few
marks. It plays one of three roles: a name for a variable, a constant whose
value is its own text, or a name that the environment of the program resolves.
Which role a symbol plays follows from its first character and from where its
periods are, not from anything around it. This page gives the characters of a
symbol, the three roles, and the limit on its length. What a variable holds,
and how a compound symbol is turned into a name when the program runs, belongs
to [the variables page](../expressions/variables.md).

r[lex.symbols.chars]
A symbol is a run of symbol characters: the letters `A` to `Z` and `a` to
`z`, the digits, and the four marks `.` `!` `?` `_`. The run ends at the
first character that is not one of these, so a blank, an operator character,
a quote mark, a comma or a parenthesis ends a symbol, and so does a comment.
A symbol is read in upper case, as [the case rule](r:lex.characters.case)
says. The [SYMBOL function](r:bif.symbol.result) tests a string against the
same characters and the same length limit.

```rexx
a = 5; b = 2; a_b = 'joined'
say a-b a+b a_b a?b a!b
say a/**/b
```

```output
3 7 joined A?B A!B
52
```

r[lex.symbols.kinds]
A symbol has one of three roles, set by its first character. A symbol that
starts with a digit is a [constant symbol](r:lex.symbols.constant). One that
starts with a period is an [environment symbol](r:lex.symbols.environment),
or a constant symbol when it is just a period or a number. Any other symbol
is a [variable symbol](r:lex.symbols.variable).

```rexx
v = 'var'
say v 1v .v
```

```output
var 1V .V
```

r[lex.symbols.constant]
A symbol that starts with a digit is a constant symbol. It names no
variable: its value is its own characters in upper case and otherwise
unchanged, so it is not normalized as a number is. A clause that assigns to a
symbol that starts with a digit raises `SYNTAX 31.2`. Assigning to a
constant symbol that starts with a period raises `SYNTAX 31.3`, as
[the environment rule](r:lex.symbols.environment) says.

```rexx
say 1e3 007 1.50 1a 2b.c
say 007 + 0 1e3 + 0
signal on syntax name e
interpret "1a = 2"
e: say condition('o')~code
```

```output
1E3 007 1.50 1A 2B.C
7 1000
31.2
```

r[lex.symbols.number]
A constant symbol can be used as a number when it has the form of one. Its
digits have at most one period and at least one digit, and may be followed
by `E` or `e`, then one optional sign, then one or more digits, and nothing
more. A sign belongs to the symbol only in that position. In every other
case the symbol ends before the sign and the sign is an operator. A constant
symbol that does not have the form of a number raises `SYNTAX 41.1` when it
is used in arithmetic.

```rexx
say 1e+3 + 0
say 5.e-1 + 0 .5E2 + 0 12E+0 + 1
say (2x9e)(+4)
call try '2x9e+4'
call try '4..1e+2'
call try '..e+5'
call try '1e+3.5'
call try '1e+3a'
call try '1.2.3'
exit
try:
  signal on syntax name bad
  interpret 'say' arg(1) '+ 0'
  return
bad: say arg(1) '=>' condition('o')~code
  return
```

```output
1000
0.5 50 13
2X9E4
2x9e+4 => 41.1
4..1e+2 => 41.1
..e+5 => 41.1
1e+3.5 => 41.1
1e+3a => 41.1
1.2.3 => 41.1
```

r[lex.symbols.variable]
A symbol that starts with a letter, `!`, `?` or `_` names a variable. It is
a simple symbol when it holds no period, a stem symbol when its only period
is its last character, and a compound symbol otherwise, where the characters
before the first period are its stem and the rest is its tail. The simple
symbol `a`, the stem symbol `a.` and the compound symbol `a.b` name three
different variables, and since a symbol is [read in upper
case](r:lex.characters.case), `a.b` and `A.B` name the same one.

```rexx
a = 1; a. = 2; a.b = 3; A.B = 4
say a a. a.b
```

```output
1 2 4
```

r[lex.symbols.environment]
A symbol that starts with a period is an environment symbol, unless it is
just the period or has the form of a [number](r:lex.symbols.number), as `.5`
and `.5e1` do: those two are constant symbols. The characters after the
period, in upper case, are the name that the environment of the program
looks up. When the lookup finds an object, that is the value of the symbol,
and when it finds none, the value is the symbol's own characters in upper
case, period included. A clause that assigns to a symbol that starts with a
period raises `SYNTAX 31.3`, the constant symbols `.`, `.5` and `.5e1`
included. The places the lookup searches, and their order, are [the rule of
the environment chapter](r:env.objects.search); the directories in which
files are searched for are those of [the places rule](r:env.search.places).

```rexx
.environment['WIDGET'] = 'found'
say .widget .Widget .nosuch
say . .5 .5e1 + 0
call try ".widget = 2"
call try ".5 = 2"
call try ". = 2"
exit
try:
  signal on syntax name e
  interpret arg(1)
e: say condition('o')~code
  return
```

```output
found found .NOSUCH
. .5 5
31.3
31.3
31.3
```

r[lex.symbols.length]
A symbol is at most 250 characters, counted as it is written: the stem, the
periods and the tail of a compound symbol together, and the leading period
of an environment symbol. A longer symbol raises `SYNTAX 30.1`. The fault is
lexical, so it is found when the program is read, even in a clause that
never runs. The limit is on the text of the program. The name that a
compound symbol takes when the program runs is not limited by it, and a
string has no limit.

```rexx
call try 'x = ' || copies('a', 250)
call try 'x.' || copies('a', 248) '= 1'
call try 'x.' || copies('a', 249) '= 1'
call try 'x = 1' || copies('a', 249)
call try 'x = 1' || copies('a', 250)
call try 'x = .' || copies('a', 250)
call try 'x = ' || copies('a', 251)
call try "x = '" || copies('a', 300) || "'"
v = copies('a', 150); w = copies('b', 150)
x.v.w = 'long name'
say x.v.w
exit
try:
  signal on syntax name bad
  interpret arg(1)
  say 'ok'
  return
bad: say condition('o')~code
  return
```

```output
ok
ok
30.1
ok
30.1
30.1
30.1
ok
long name
```

```rexx
f = 'spec-lex-long.tmp'
call sysfiledelete f
call charout f, "say 'before'" || '0a'x || "if 0 then x = "
call charout f, copies('a', 251) || '0a'x
call charout f
signal on syntax name e
call (f)
e: say condition('o')~code
call sysfiledelete f
```

```output
30.1
```
