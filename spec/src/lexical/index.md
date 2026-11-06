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

A token belongs to one of these classes: a literal string, a hexadecimal
string, a binary string, a symbol, a number, an operator character or a
special character. Blanks and comments sit between tokens and are not tokens
themselves. The pages of this chapter take the classes in turn:

* [Characters](characters.md): the characters a program may use, case, blanks
  and line ends.
* [Comments](comments.md): block comments, nesting and line comments.
* [Literals](literals.md): strings, hexadecimal strings and binary strings.
* Symbols and numbers, operator characters and special characters, and the
  clause level (clause ends, labels, continuation) have pages of their own.

r[lex.index.read-first]
A program is read in full before any clause of it runs. A fault in the
lexical form of the text, such as an unmatched quote or comment delimiter
(`SYNTAX 6.1`, `6.2` and `6.3`), a character outside the
[character set](r:lex.characters.set) (`SYNTAX 13.1`) or a malformed
hexadecimal or binary string (`SYNTAX 15.1` to `15.6`), is raised before the
first clause runs, so the program produces no output and ends with the exit
code `256` minus the error number. The text given to INTERPRET is read when
the INTERPRET instruction runs, so the same fault there is an ordinary
condition that a trap can handle, and the instructions before it in the
string never run.

```rexx
say 'before'
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
joins a following operator character. Blanks are needed only where two
tokens would otherwise read as one, so `1+2` and `1 + 2` are the same three
tokens, and a string may touch the symbol after it.

```rexx
y = 'v'
say 1+2 1 + 2
say 'x'y 'p'q'r'
```

```output
3 3
xv pQr
```
