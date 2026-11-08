---
element: OPTIONS
kind: instruction
syntax: "OPTIONS expression"
ansi: "6.3.2.70, 8.3.16"
rexxref: "Keyword Instructions > OPTIONS"
---

# OPTIONS

OPTIONS passes requests to the language processor. It evaluates an expression,
reads the words of the result and obeys those it knows. interrexx knows none:
the instruction evaluates its operand, so that the operand can raise errors,
and then it does nothing with the words. This page states that, and states
what the operand must be. The book specifies no option word, so a program
that relies on a word having an effect is not portable.

## What it does

r[instr.options.ignored]
OPTIONS evaluates its expression, converts the result to a string and reads
no word of it as a request: no word changes the behavior of the program, and
no word is an error. Words such as `ETMODE`, `NOEXMODE` and `NOVALUE` are
ignored like any other, and so are the words that the NUMERIC and TRACE
instructions use, so OPTIONS does not change a setting. The instruction sets
no variable and leaves `RESULT` and `RC` as they were.

```rexx
result = 'keep'; rc = 'mine'
say digits() fuzz() form() trace()
options 'ETMODE NOEXMODE NOVALUE COBOL'
options 'DIGITS 5 FUZZ 1 FORM ENGINEERING TRACE R'
options '' 3 4.5 'junk'
say digits() fuzz() form() trace()
say result rc symbol('undefinedvar') undefinedvar
```

```output
9 0 SCIENTIFIC N
9 0 SCIENTIFIC N
keep mine LIT UNDEFINEDVAR
```

r[instr.options.expression]
The operand is a full expression, so a comparison, a call and a message are
evaluated, and an error that they raise is raised by OPTIONS. A clause that is
only the keyword raises `SYNTAX 35.913` when the program is read, so that the
program runs nothing and the exit code is 221. A clause `options x = 5` is the
instruction with the comparison `x = 5` as operand, which assigns nothing, and
`options = 5` is an assignment to a variable named OPTIONS, as
[the kind rule](r:lex.clauses.kinds) says.

```rexx
call try 'options 1 / 0'
call try 'options .nosuch~foo'
call try 'options'
call try 'options 1 2 )'
options x = 5
say symbol('x')
options = 5
say options
exit
try:
  signal on syntax name e
  interpret arg(1)
  say 'ok' arg(1)
  return
e: say condition('o')~code '<=' arg(1)
  return
```

```output
42.3 <= options 1 / 0
97.1 <= options .nosuch~foo
35.913 <= options
37.2 <= options 1 2 )
LIT
5
```

```rexx
say 'before'
options
```

```output rc=221
```
