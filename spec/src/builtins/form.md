---
element: FORM
kind: builtin
syntax: "FORM()"
ansi: "9.5.6"
rexxref: "Functions > Built-in Functions > FORM"
---

# FORM

FORM answers the setting of NUMERIC FORM in force where it is called: how a
number written in exponent notation presents its exponent.

## Arguments

r[bif.form.args]
FORM takes no arguments. The argument count is checked by the [builtin call
rules](r:bif.index.call.count): none is required and none is the most, so a
call with an argument raises `SYNTAX 40.4`, whatever its value, while a call
with only omitted places counts as a call with no arguments. FORM does not
run as a String method: sending the message `FORM` to a string raises
`SYNTAX 97.1`.

```rexx
say form() form( ) form(,)
signal on syntax name a
say form(1)
a: say condition('o')~code
signal on syntax name b
say form('')
b: say condition('o')~code
signal on syntax name c
say form(, 'SCIENTIFIC')
c: say condition('o')~code
signal on syntax name d
say 'x'~form
d: say condition('o')~code
```

```output
SCIENTIFIC SCIENTIFIC SCIENTIFIC
40.4
40.4
40.4
97.1
```

## The result

r[bif.form.result]
FORM answers `SCIENTIFIC` or `ENGINEERING`, in capital letters, whatever
case the NUMERIC FORM instruction wrote the keyword in. The setting at the start
of a program is the [default](r:bif.form.default) of the package. The
instruction can also take its keyword from
the value of an expression, with `VALUE`, where the value must be exactly
`SCIENTIFIC` or `ENGINEERING`, in capital letters and without blanks around
it, or `SYNTAX 25.11` is raised and the setting stays as it was.

```rexx
say form()
numeric form engineering
say form()
numeric form scientific
say form()
numeric form value 'ENGINEERING'
say form()
k = 'SCIENTIFIC'
numeric form value k
say form()
numeric form Engineering
say form()
signal on syntax name a
numeric form value ' SCIENTIFIC '
a: say condition('o')~code form()
signal on syntax name b
numeric form value 'scientific'
b: say condition('o')~code form()
```

```output
SCIENTIFIC
ENGINEERING
SCIENTIFIC
ENGINEERING
SCIENTIFIC
ENGINEERING
25.11 ENGINEERING
25.11 ENGINEERING
```

r[bif.form.default]
The default of a package is `SCIENTIFIC` unless a `::OPTIONS` directive of
the package names another value with its `FORM` keyword. Where it does, that
value is the setting at
the start of the program, the setting at the start of each of its
`::ROUTINE` routines and methods, and the setting that the NUMERIC
FORM instruction without an expression returns to.

```rexx
say 'start' form()
numeric form scientific
say 'set' form()
numeric form
say 'reset' form() later() .holder~new~m
exit
::options form engineering
::routine later
numeric form scientific
numeric form
return form()
::class holder
::method m
return form()
```

```output
start ENGINEERING
set SCIENTIFIC
reset ENGINEERING ENGINEERING ENGINEERING
```

r[bif.form.scope]
The setting belongs to the routine or method that is running. An internal
routine, which CALL or a function call reaches by its label, starts with the
setting in force at the call, and the caller's setting is unchanged when it
returns, whatever it set meanwhile. A routine of a `::ROUTINE` directive,
public or private, a method (also one that another method sends the message
to), and an external routine in another file all start with the
[default](r:bif.form.default) of their own package, whatever the setting of the
caller. The NUMERIC instruction in an INTERPRET string changes the setting of
the running routine, and the change stays after the INTERPRET.

```rexx
numeric form engineering
say 'main' form() inner() later() later2() .holder~new~m
call setsci
say 'main' form()
call later3
call hidden
say .holder~new~n
interpret 'numeric form scientific'
say 'main' form()
exit
inner: return form()
later: numeric form scientific; return form()
setsci: numeric form scientific; say 'setsci' form(); return
::routine later2
return form()
::routine later3
say 'later3' form()
::routine hidden private
say 'hidden' form()
::class holder
::method m
return form()
::method n
numeric form engineering
return self~m form()
```

```output
main ENGINEERING ENGINEERING SCIENTIFIC SCIENTIFIC SCIENTIFIC
setsci SCIENTIFIC
main ENGINEERING
later3 SCIENTIFIC
hidden SCIENTIFIC
SCIENTIFIC ENGINEERING
main SCIENTIFIC
```
