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
is the [default](r:bif.fuzz.default) of the package, and the NUMERIC FUZZ
instruction changes it, to the value of its expression as a whole number. A
setting is a count of digits, so it must stay below the setting of NUMERIC
DIGITS: both NUMERIC FUZZ and NUMERIC DIGITS raise `SYNTAX 33.1` for a
value that breaks this, and the settings stay as they were.

```rexx
say fuzz()
numeric fuzz 3
say fuzz() datatype(fuzz(), 'W')
numeric fuzz ' 02.0 '
say fuzz()
numeric fuzz 1e1 - 7
say fuzz()
signal on syntax name a
numeric fuzz 9
a: say condition('o')~code fuzz()
signal on syntax name b
numeric digits 3
b: say condition('o')~code digits() fuzz()
```

```output
0
3 1
2
3
33.1 3
33.1 9 3
```

r[bif.fuzz.default]
The default of a package is 0 unless a `::OPTIONS` directive of
the package names another value with its `FUZZ` keyword. Where it does, that value is the setting at
the start of the program, the setting at the start of each of its
`::ROUTINE` routines and methods, and the setting that the NUMERIC
FUZZ instruction without an expression returns to.

```rexx
say 'start' fuzz()
numeric fuzz 4
say 'set' fuzz()
numeric fuzz
say 'reset' fuzz() later() .holder~new~m
exit
::options fuzz 2
::routine later
numeric fuzz 5
numeric fuzz
return fuzz()
::class holder
::method m
return fuzz()
```

```output
start 2
set 4
reset 2 2 2
```

r[bif.fuzz.scope]
The setting belongs to the routine or method that is running. An internal
routine, which CALL or a function call reaches by its label, starts with the
setting in force at the call, and the caller's setting is unchanged when it
returns, whatever it set meanwhile. A routine of a `::ROUTINE` directive,
public or private, a method (also one that another method sends the message
to), and an external routine in another file all start with the
[default](r:bif.fuzz.default) of their own package, whatever the setting of the
caller. The NUMERIC instruction in an INTERPRET string changes the setting of
the running routine, and the change stays after the INTERPRET.

```rexx
numeric fuzz 2
say 'main' fuzz() inner() later() later2() .holder~new~m
call set4
say 'main' fuzz()
call later3
call hidden
say .holder~new~n
interpret 'numeric fuzz 3'
say 'main' fuzz()
exit
inner: return fuzz()
later: numeric fuzz 1; return fuzz()
set4: numeric fuzz 4; say 'set4' fuzz(); return
::routine later2
return fuzz()
::routine later3
say 'later3' fuzz()
::routine hidden private
say 'hidden' fuzz()
::class holder
::method m
return fuzz()
::method n
numeric fuzz 4
return self~m fuzz()
```

```output
main 2 2 1 0 0
set4 4
main 2
later3 0
hidden 0
0 4
main 3
```
