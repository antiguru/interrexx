---
element: "Numeric settings"
kind: concept
ansi: "8.3.15"
rexxref: "Numbers and Arithmetic > Arithmetic > Precision"
---

# Numeric settings

Three settings steer every numeric operation: NUMERIC DIGITS, NUMERIC FUZZ
and NUMERIC FORM. They are not arguments of the operations and no number
carries them, so the same expression gives different answers under different
settings. This page says what each setting controls, how it is checked when a
program changes it, and which part of a program a change reaches. The syntax
of the NUMERIC instruction that changes a setting belongs to the instructions
chapter, and the examples here use it only in its plain form, `numeric digits
expression` and its two siblings.

## What the settings control

r[num.settings.effect]
NUMERIC DIGITS is the count of significant digits that arithmetic and numeric
comparison keep, and it is also read by the builtins that [round their
numbers](r:bif.index.num.round). NUMERIC FUZZ is the count of those digits
that a numeric comparison ignores, so it acts only on comparison, including
the comparisons of the builtins that [pick](r:bif.index.num.pick) a number.
NUMERIC FORM decides only how an exponent is written in a result that needs
one, as [the notation rule](r:bif.index.num.sci) says. Each setting has a
[default](r:bif.digits.default) that a program starts with: 9, 0 and
SCIENTIFIC, unless a directive of the package gives another value. The
builtins [DIGITS](r:bif.digits.result), [FUZZ](r:bif.fuzz.result) and
[FORM](r:bif.form.result) answer the setting in force where they are called.

```rexx
say digits() fuzz() form()
say (1 / 3) (12345678 * 1000) (1.0004 = 1) (1.006 = 1)
numeric digits 4
numeric fuzz 1
numeric form engineering
say digits() fuzz() form()
say (1 / 3) (12345678 * 1000) (1.0004 = 1) (1.006 = 1)
say (1.0004 + 1) (1.0004 - 1)
```

```output
9 0 SCIENTIFIC
0.333333333 1.23456780E+10 0 0
4 1 ENGINEERING
0.3333 12.35E+9 1 0
2.000 0
```

r[num.settings.written]
A setting acts when an operation runs and never afterwards. A string that an
operation answered keeps the digits and the notation it was written with, and
a later change of a setting leaves it as it is, even though a later operation
that uses it works under the new setting. This is the rule that
[a number is a string](r:num.index.string) applied to settings.

```rexx
numeric digits 3
x = 1 / 3
y = 12345 + 0
numeric digits 9
say x y (x + 0) (x * 3) (x * 3.0) (y + 0)
numeric digits 3
numeric form engineering
say x y (y + 0) (y * 1)
```

```output
0.333 1.23E+4 0.333 0.999 0.9990 12300
0.333 1.23E+4 12.3E+3 12.3E+3
```

## Changing a setting

r[num.settings.digits]
The value that NUMERIC DIGITS sets must be a whole number of at least 1 and of
at most as many digits as the setting in force, after it is rounded to that
many digits as `value + 0` would round it. Any other value raises `SYNTAX
26.5`, a value that is not a number, a fraction, zero and a negative number
among them, and the setting stays as it was. The check is made under the old
setting, so a setting of 2 cannot be changed to a value of 100, while a
setting of 3 can be changed to 999. Writing no value returns the setting to
its default. The new setting must stay above NUMERIC FUZZ, or the instruction
raises `SYNTAX 33.1` as [the fuzz rule](r:bif.fuzz.result) says.

```rexx
numeric digits 20
say digits()
numeric digits 1e1 + 5
say digits()
numeric digits 3
numeric digits 999.4
say digits()
numeric digits 2
numeric digits 99
say digits()
numeric digits 2
signal on syntax name a
numeric digits 100
a: say condition('o')~code digits()
signal on syntax name b
numeric digits 0
b: say condition('o')~code digits()
signal on syntax name c
numeric digits 2.5
c: say condition('o')~code digits()
signal on syntax name d
numeric digits 'x'
d: say condition('o')~code digits()
signal on syntax name e
numeric digits -3
e: say condition('o')~code digits()
```

```output
20
15
999
99
26.5 2
26.5 2
26.5 2
26.5 2
26.5 2
```

r[num.settings.fuzz]
The value that NUMERIC FUZZ sets must be a whole number of at least 0 and of
at most as many digits as NUMERIC DIGITS, after it is rounded to that many
digits. Any other value raises `SYNTAX 26.6`, and so does a value that is not
a number, and the setting stays as it was. A value that is not below NUMERIC
DIGITS raises `SYNTAX 33.1`, and so does a return of NUMERIC DIGITS to its
default when that default is not above the fuzz.

```rexx
numeric digits 20
numeric fuzz 15
signal on syntax name a
numeric digits
a: say condition('o')~code digits() fuzz()
signal on syntax name b
numeric fuzz 2.5
b: say condition('o')~code fuzz()
signal on syntax name c
numeric fuzz 'x'
c: say condition('o')~code fuzz()
signal on syntax name d
numeric fuzz -1
d: say condition('o')~code fuzz()
signal on syntax name e
numeric fuzz 20
e: say condition('o')~code fuzz()
numeric fuzz 0.0
say fuzz()
```

```output
33.1 20 15
26.6 15
26.6 15
26.6 15
33.1 15
0
```

## Where a setting reaches

r[num.settings.block]
A setting belongs to the routine that is running and not to a block of it. A
NUMERIC instruction in the body of a DO loop, an IF or a SELECT changes the
setting for the rest of the routine, including the clauses after the block,
and a trapped condition that transfers control to a label of the same routine
finds the setting as it was.

```rexx
numeric digits 4
numeric fuzz 1
do 2
  numeric digits 5
end
if 1 then numeric fuzz 2
select
  when 1 then numeric form engineering
  otherwise nop
end
say digits() fuzz() form()
signal on syntax name h
x = 'a' + 1
h: say condition('o')~code digits() fuzz() form()
```

```output
5 2 ENGINEERING
41.1 5 2 ENGINEERING
```

r[num.settings.call]
An internal routine, which a CALL or a function call reaches by its label,
starts with the three settings that the caller has at the call. It may change
them, and the caller has its own settings again when the routine returns,
whether the routine is written with PROCEDURE or not. The value that the
routine returns is a string like any other, written under the routine's
settings, and the caller finds it as it was written. Routines of a directive,
methods and routines of other files do not start from the caller's settings,
as [the scope rule](r:bif.digits.scope) says of NUMERIC DIGITS and as
[the same rule](r:bif.fuzz.scope) says of FUZZ.

```rexx
numeric digits 4
numeric fuzz 1
numeric form engineering
call inner
say digits() fuzz() form()
say outer() digits() (1 / 3)
numeric digits 9
say passed(1 / 3)
exit
inner: procedure
  numeric digits 12
  numeric fuzz 0
  numeric form scientific
  say digits() fuzz() form() (1 / 3)
  return
outer:
  numeric digits 3
  return 1 / 3
passed: procedure
  return arg(1) digits() (arg(1) + 0)
```

```output
12 0 SCIENTIFIC 0.333333333333
4 1 ENGINEERING
0.333 4 0.3333
0.333333333 9 0.333333333
```
