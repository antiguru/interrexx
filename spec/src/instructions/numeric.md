---
element: NUMERIC
kind: instruction
syntax: "NUMERIC {DIGITS [expression] | FUZZ [expression] | FORM [SCIENTIFIC | ENGINEERING | [VALUE] expression]}"
ansi: "6.3.2.66, 8.3.15"
rexxref: "Keyword Instructions > NUMERIC"
---

# NUMERIC

NUMERIC changes one of the three settings that steer arithmetic: the number
of digits kept, the number of digits a comparison ignores, and the notation
of an exponent. The meaning of each setting, the values it accepts, its
default and the part of the program that a change reaches belong to [the
numeric settings](r:num.settings.effect), and this page does not repeat them.
It specifies the syntax of the instruction, when its expression is evaluated,
what leaving the expression out means, the forms of FORM, and the errors of
the clause.

## DIGITS and FUZZ

r[instr.numeric.value]
NUMERIC DIGITS and NUMERIC FUZZ take an optional expression. The expression
is an ordinary one, so it may use operators, variables and function calls,
and it is evaluated before the setting changes, with the setting in force.
Its value is then checked and stored as [the DIGITS](r:num.settings.digits)
and [the FUZZ](r:num.settings.fuzz) rules say. The keywords may be written in
any case, and a comment may stand between the words.

```rexx
numeric digits 5 + 5
say digits()
numeric digits digits() * 2
say digits()
Numeric /* now */ DIGITS 3
say digits()
numeric digits 100 + 0.4
say digits()
numeric fuzz 1 + 1
say fuzz()
```

```output
10
20
3
100
2
```

r[instr.numeric.default]
NUMERIC DIGITS or NUMERIC FUZZ with no expression returns the setting to its
default, which is 9 for DIGITS and 0 for FUZZ unless the package names
another one, as [DIGITS](r:bif.digits.default) and [FUZZ](r:bif.fuzz.default)
say. The clause ends at the semicolon or the line end that follows the
keyword.

```rexx
numeric digits 20
numeric fuzz 3
numeric digits; numeric fuzz
say digits() fuzz()
```

```output
9 0
```

## FORM

r[instr.numeric.form]
NUMERIC FORM has four shapes. The symbol SCIENTIFIC or ENGINEERING, in any
case, sets the form directly. The keyword `VALUE` followed by an expression
sets the form to the value of the expression. An expression that does not
start with a symbol, such as a string or a parenthesized expression, needs no
`VALUE`. With nothing after FORM the form returns to its
[default](r:bif.form.default). The value of an expression has to be exactly
`SCIENTIFIC` or `ENGINEERING`, in capitals, with no blanks round it and no
abbreviation.

```rexx
numeric form engineering
say form()
numeric form Scientific
say form()
f = 'ENGINEERING'
numeric form value f
say form()
numeric form ('SCIENTIFIC')
say form()
numeric form value 'ENG' || 'INEERING'
say form()
numeric form
say form()
numeric form 'ENGINEERING'
say form()
numeric form
say form()
```

```output
ENGINEERING
SCIENTIFIC
ENGINEERING
SCIENTIFIC
ENGINEERING
SCIENTIFIC
ENGINEERING
SCIENTIFIC
```

r[instr.numeric.form.error]
A value that is not exactly one of the two words raises `SYNTAX 25.11` when
the instruction runs, and the form stays as it was. Lower case, mixed case,
blanks round the word, an abbreviation and a number are all such values. A
symbol after FORM that is neither of the two words nor VALUE is read as an
expression without `VALUE`, and the program is refused with `25.11` when it
is read.

```rexx
call try "numeric form 'engineering'"
call try "numeric form value 'Engineering'"
call try "numeric form value ' ENGINEERING '"
call try "numeric form value 'ENG'"
call try "numeric form value 5"
call try "numeric form value nosuch"
call try "numeric form + 1"
call try "numeric form nosuch"
exit
try:
  signal on syntax name e
  numeric form scientific
  interpret arg(1)
  say arg(1) '=>' form()
  return
e: say arg(1) '=>' condition('o')~code form()
  return
```

```output
numeric form 'engineering' => 25.11 SCIENTIFIC
numeric form value 'Engineering' => 25.11 SCIENTIFIC
numeric form value ' ENGINEERING ' => 25.11 SCIENTIFIC
numeric form value 'ENG' => 25.11 SCIENTIFIC
numeric form value 5 => 25.11 SCIENTIFIC
numeric form value nosuch => 25.11 SCIENTIFIC
numeric form + 1 => 25.11 SCIENTIFIC
numeric form nosuch => 25.11 SCIENTIFIC
```

```rexx
say 'before'
numeric form nosuch
```

```output rc=231
```

## Errors

r[instr.numeric.syntax]
A fault in the form of the clause is raised when the program is read, so no
clause runs: `20.905` for NUMERIC alone or followed by something that is not
a symbol; `25.15` for a symbol that is not DIGITS, FUZZ or FORM; `21.911` for
anything after SCIENTIFIC or ENGINEERING; and `35.917` for FORM VALUE with no
expression. A fault in the expression is the error of the expression, such as
`35.1` for an operator with nothing after it. A value that the setting does
not accept is raised when the instruction runs, with the error that [the
settings rules](r:num.settings.digits) give: `26.5`, `26.6` or `33.1`. An
INTERPRET raises the same errors when it runs.

```rexx
say 'before'
numeric
```

```output rc=236
```

```rexx
call try 'numeric'
call try "numeric 'digits' 5"
call try 'numeric foo'
call try 'numeric forms scientific'
call try 'numeric form engineering extra'
call try 'numeric form scientific 1'
call try 'numeric form value'
call try 'numeric digits 5 +'
call try 'numeric digits 5 6'
call try 'numeric fuzz 1 2'
call try 'numeric digits nosuch'
call try 'numeric digits 0'
call try 'numeric fuzz 9'
exit
try:
  signal on syntax name e
  numeric digits 9
  numeric fuzz 0
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
numeric => 20.905
numeric 'digits' 5 => 20.905
numeric foo => 25.15
numeric forms scientific => 25.15
numeric form engineering extra => 21.911
numeric form scientific 1 => 21.911
numeric form value => 35.917
numeric digits 5 + => 35.1
numeric digits 5 6 => 26.5
numeric fuzz 1 2 => 26.6
numeric digits nosuch => 26.5
numeric digits 0 => 26.5
numeric fuzz 9 => 33.1
```
