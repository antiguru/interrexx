---
element: FUZZ
kind: builtin
syntax: "FUZZ()"
ansi: "9.5.7"
rexxref: "Functions > Built-in Functions > FUZZ"
---

# FUZZ

FUZZ answers the setting of NUMERIC FUZZ in force where it is called: the
count of low-order digits that a numeric comparison ignores.

## Arguments

r[bif.fuzz.args]
FUZZ takes no arguments. The argument count is checked by the [builtin call
rules](r:bif.index.call.count): none is required and none is the most, so a
call with an argument raises `SYNTAX 40.4`, whatever its value, while a call
with only omitted places counts as a call with no arguments. FUZZ does not
run as a String method: sending the message `FUZZ` to a string raises
`SYNTAX 97.1`.

```rexx
say fuzz() fuzz( ) fuzz(,)
signal on syntax name a
say fuzz(1)
a: say condition('o')~code
signal on syntax name b
say fuzz('')
b: say condition('o')~code
signal on syntax name c
say fuzz(, 0)
c: say condition('o')~code
signal on syntax name d
say 0~fuzz
d: say condition('o')~code
```

```output
0 0 0
40.4
40.4
40.4
97.1
```

## The result

r[bif.fuzz.result]
FUZZ answers the setting as a whole number written without a sign, a
point, an exponent or leading zeros. At the start of a program the setting
is 0, and the NUMERIC FUZZ instruction changes it, to the value of its
expression as a whole number. A setting is a count of digits, so it must
stay below the setting of NUMERIC DIGITS, as the instruction checks.

```rexx
say fuzz()
numeric fuzz 3
say fuzz() datatype(fuzz(), 'W')
numeric fuzz ' 02.0 '
say fuzz()
numeric fuzz 1e1 - 7
say fuzz()
```

```output
0
3 1
2
3
```

r[bif.fuzz.scope]
The setting belongs to the routine or method that is running. A routine
called with CALL or as a function starts with the setting in force at the
call and gives it back when it returns, whatever it set meanwhile; the
instruction does not change the setting of the caller. A public routine or a
method starts with 0, whatever the setting of its caller.

```rexx
numeric fuzz 2
say 'main' fuzz() inner() later() later2() .holder~new~m
call set4
say 'main' fuzz()
exit
inner: return fuzz()
later: numeric fuzz 1; return fuzz()
set4: numeric fuzz 4; say 'set4' fuzz(); return
::routine later2
return fuzz()
::class holder
::method m
return fuzz()
```

```output
main 2 2 1 0 0
set4 4
main 2
```
