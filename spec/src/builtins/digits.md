---
element: DIGITS
kind: builtin
syntax: "DIGITS()"
ansi: "9.5.4"
rexxref: "Functions > Built-in Functions > DIGITS"
---

# DIGITS

DIGITS answers the setting of NUMERIC DIGITS in force where it is called: the
count of significant digits that arithmetic keeps.

## Arguments

r[bif.digits.args]
DIGITS takes no arguments. The argument count is checked by the [builtin call
rules](r:bif.index.call.count): none is required and none is the most, so a
call with an argument raises `SYNTAX 40.4`, whatever its value, while a call
with only omitted places counts as a call with no arguments. DIGITS does not
run as a String method: sending the message `DIGITS` to a string raises
`SYNTAX 97.1`.

```rexx
say digits() digits( ) digits(,)
signal on syntax name a
say digits(1)
a: say condition('o')~code
signal on syntax name b
say digits('')
b: say condition('o')~code
signal on syntax name c
say digits(, 9)
c: say condition('o')~code
signal on syntax name d
say 9~digits
d: say condition('o')~code
```

```output
9 9 9
40.4
40.4
40.4
97.1
```

## The result

r[bif.digits.result]
DIGITS answers the setting as a whole number written without a sign, a
point, an exponent or leading zeros. At the start of a program the setting
is the [default](r:bif.digits.default) of the package, and the NUMERIC
DIGITS instruction changes it, to the value of its expression as a whole
number.

```rexx
say digits()
numeric digits 1e3
say digits() datatype(digits(), 'W')
numeric digits ' 0015.0 '
say digits()
numeric digits 12 + 3 * 2
say digits()
```

```output
9
1000 1
15
18
```

r[bif.digits.default]
The default of a package is 9 unless a `::OPTIONS` directive of the package
names another value with its `DIGITS` keyword. Where it does, that value is
the setting at the start of the program, the setting at the start of each of
its `::ROUTINE` routines and methods, and the setting that the NUMERIC DIGITS
instruction without an expression returns to.

```rexx
say 'start' digits()
numeric digits 20
say 'set' digits()
numeric digits
say 'reset' digits() later() .holder~new~m
exit
::options digits 12
::routine later
numeric digits 30
numeric digits
return digits()
::class holder
::method m
return digits()
```

```output
start 12
set 20
reset 12 12 12
```

r[bif.digits.scope]
The setting belongs to the routine or method that is running. An internal
routine, which CALL or a function call reaches by its label, starts with the
setting in force at the call, and the caller's setting is unchanged when it
returns, whatever it set meanwhile. A routine of a `::ROUTINE` directive,
public or private, a method (also one that another method sends the message
to), and an external routine in another file all start with the
[default](r:bif.digits.default) of their own package, whatever the setting of
the caller. The NUMERIC instruction in an INTERPRET string changes the setting
of the running routine, and the change stays after the INTERPRET.

```rexx
numeric digits 15
say 'main' digits() inner() later() later2() .holder~new~m
call set30
say 'main' digits()
call later3
call hidden
say .holder~new~n
interpret 'numeric digits 17'
say 'main' digits()
exit
inner: return digits()
later: numeric digits 21; return digits()
set30: numeric digits 30; say 'set30' digits(); return
::routine later2
return digits()
::routine later3
say 'later3' digits()
::routine hidden private
say 'hidden' digits()
::class holder
::method m
return digits()
::method n
numeric digits 40
return self~m digits()
```

```output
main 15 15 21 9 9
set30 30
main 15
later3 9
hidden 9
9 40
main 17
```
