---
element: "Lexical structure"
kind: chapter
ansi: "6.2"
rexxref: "General Concepts > Structure and General Syntax"
---

# Lexical structure

This chapter specifies how the text of a program becomes tokens. A program is
a sequence of clauses, and a clause is a sequence of tokens. The chapter
covers the level below the clause: which characters a program may hold, how
comments and strings are written, and where one token ends and the next
begins. The meaning of a clause, and of the expressions inside it, is the
business of the chapters that follow.

Every token is a string in quotes, a symbol or number, an operator of one to
three characters, or a special character, and strings carry an optional
radix suffix that makes them hexadecimal or binary. Blanks and comments sit
between tokens and are not tokens themselves. The chapter has these pages:

* [Characters](characters.md): the characters a program may use, case, blanks
  and line ends.
* [Comments](comments.md): block comments, nesting and line comments.
* [Literals](literals.md): strings, hexadecimal strings and binary strings.
* [Symbols](symbols.md): the characters of a symbol, constant, simple, stem
  and compound symbols, and environment symbols.
* [Operators](operators.md): special characters and how operator tokens form.
* [Clauses](clauses.md): where a clause ends, null clauses, labels and
  continuation.

r[lex.index.read-first]
A program is read in full before any clause of it runs, so a fault anywhere in
the text, even after an `exit` or inside a routine that is never called, is
raised before the first clause runs. A fault in the lexical form of the text
is an unmatched quote or comment delimiter (`SYNTAX 6.1`, `6.2` and `6.3`), a
character outside the [character set](r:lex.characters.set) (`SYNTAX 13.1`) or
a malformed hexadecimal or binary string (`SYNTAX 15.1` to `15.6`). The
program then produces no output, and the interpreter exits with the negated
error number, so that the exit status is `256` minus the error number: 243 for
13.1. The text given to INTERPRET is read when the [INTERPRET
instruction](r:instr.interpret.whole) runs, so the same fault there is an
ordinary [SYNTAX](r:cond.syntax.raise) condition that a trap can handle, and
the instructions before it in the string never run.

```rexx
say 'before'; exit
say 1 #
```

```output rc=243
```

```rexx
signal on syntax name e
say 'before'
interpret "say 'x'; say 1 #"
say 'not reached'
e: say condition('o')~code
```

```output
before
13.1
```

r[lex.index.token-end]
A token ends at the first character that cannot be part of it. A string
ends at its closing quote, a symbol at the first character that is not a
symbol character, and an operator character is a token of its own or
joins a following operator character where the two form one operator, also
across blanks and comments, as [the gap rule](r:lex.operators.gap) says.
Blanks are needed only where two tokens would otherwise read as one, so
`1+2` and `1 + 2` are the same three tokens. A string may also
[touch the symbol after it](r:lex.literals.adjacent).

```rexx
say 1+2 1 + 2
say 1 | | 2 1 |/**/| 2
```

```output
3 3
12 12
```
