---
element: SETLOCAL
kind: builtin
syntax: "SETLOCAL()"
rexxref: "Functions > Built-in Functions > SETLOCAL"
---

# SETLOCAL

SETLOCAL saves the current directory and the values of the environment
variables, so that [ENDLOCAL](endlocal.md) can put them back later. It answers
`1`. The save itself changes nothing the program can see; what the program does
afterwards to the current directory and to the environment variables, for
instance with [DIRECTORY](directory.md) and [VALUE](value.md), is what ENDLOCAL
undoes.

The environment variables are those of the process, which the `ENVIRONMENT`
selector of VALUE reads and writes and which the commands the program starts
inherit. Variables of the program itself are not among them.

> **Note.** The examples change only variables whose names start with
> `SPEC_SETLOCAL`, and compare the current directory with the one the program
> started in, so they do not depend on what the program starts with. A program
> that calls SETLOCAL and no ENDLOCAL leaves nothing for the other examples to
> see.

## Arguments

r[bif.setlocal.args]
SETLOCAL takes no argument. A call with an argument raises `SYNTAX 40.4`, by
the [call rules](r:bif.index.call.too-many). A trailing omitted place adds
nothing to the count, so `SETLOCAL(,)` is a call without an argument.
SETLOCAL does not run as a String method: sending the message `SETLOCAL` to a
string raises `SYNTAX 97.1`.

```rexx
signal on syntax name a
say setlocal(1)
a: say condition('o')~code
signal on syntax name b
say setlocal(, 'x')
b: say condition('o')~code
signal on syntax name c
say 'x'~setlocal
c: say condition('o')~code
signal off syntax
say setlocal(,)
```

```output
40.4
40.4
97.1
1
```

## The result

r[bif.setlocal.answer]
A call answers `1` once the directory and the environment variables are saved,
and a call as a subroutine sets `RESULT` to it. A program can call SETLOCAL any
number of times; each call saves the state as it is then, and
[ENDLOCAL](r:bif.endlocal.restore) takes the saves back in the opposite order.

```rexx
say setlocal() datatype(setlocal(), 'W')
call setlocal
say result
```

```output
1 1
1
```

r[bif.setlocal.nothing]
SETLOCAL does not change the current directory, the environment variables or
any variable of the program: they have the same values after the call as
before it.

```rexx
here = directory()
call value 'SPEC_SETLOCAL_A', 'a0', 'ENVIRONMENT'
v = 'kept'
call setlocal
say (directory() == here) value('SPEC_SETLOCAL_A',, 'ENVIRONMENT') v
```

```output
1 a0 kept
```

r[bif.setlocal.return]
A routine's return does not take a save back. The save made inside a routine is
still there after the routine has returned, and the changes made after it stay
in force. Only [ENDLOCAL](endlocal.md) takes a save back.

```rexx
call value 'SPEC_SETLOCAL_B', 'before', 'ENVIRONMENT'
call sub
say value('SPEC_SETLOCAL_B',, 'ENVIRONMENT')
exit
sub:
call setlocal
call value 'SPEC_SETLOCAL_B', 'inside', 'ENVIRONMENT'
return
```

```output
inside
```
