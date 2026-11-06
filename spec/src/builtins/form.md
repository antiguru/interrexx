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
FORM answers `SCIENTIFIC` or `ENGINEERING`, in capital letters, whichever
way the NUMERIC FORM instruction wrote the keyword. The setting at the start
of a program is `SCIENTIFIC`. The instruction can also take its keyword from
the value of an expression, with `VALUE`, where the keyword must be written
in capital letters.

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
```

```output
SCIENTIFIC
ENGINEERING
SCIENTIFIC
ENGINEERING
SCIENTIFIC
ENGINEERING
```

r[bif.form.scope]
The setting belongs to the routine or method that is running. A routine
called with CALL or as a function starts with the setting in force at the
call and gives it back when it returns, whatever it set meanwhile; the
instruction does not change the setting of the caller. A public routine or a
method starts with `SCIENTIFIC`, whatever the setting of its caller.

```rexx
numeric form engineering
say 'main' form() inner() later() later2() .holder~new~m
call setsci
say 'main' form()
exit
inner: return form()
later: numeric form scientific; return form()
setsci: numeric form scientific; say 'setsci' form(); return
::routine later2
return form()
::class holder
::method m
return form()
```

```output
main ENGINEERING ENGINEERING SCIENTIFIC SCIENTIFIC SCIENTIFIC
setsci SCIENTIFIC
main ENGINEERING
```
