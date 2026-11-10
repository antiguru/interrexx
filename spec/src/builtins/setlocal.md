---
element: SETLOCAL
kind: builtin
syntax: "SETLOCAL()"
rexxref: "Functions > Built-in Functions > SETLOCAL (Linux only)"
---

# SETLOCAL

SETLOCAL records where the program is and what every environment variable
holds, so that [ENDLOCAL](endlocal.md) can put both back later. It answers
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
> see. Each example takes at most one save back, counting the one that the
> reference implementation puts back at the end of a
> [`::routine`](r:dir.routine.create), a method or the program, because that
> implementation can fail in a program that takes a save back a second time.

## Arguments

r[bif.setlocal.args]
SETLOCAL takes no argument. A call with an argument raises `SYNTAX 40.4`, by
the [call rules](r:bif.index.call.too-many). A trailing omitted place adds
nothing to the count, so `SETLOCAL(,)` is a call without an argument. SETLOCAL
does not run as a [String method](r:class.string.builtins): sending the
message `SETLOCAL` to a string raises `SYNTAX 97.1`.

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
number of times; each call saves the state as it is then, and ENDLOCAL takes
the saves back [in the opposite order](r:bif.endlocal.nested).

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

r[bif.setlocal.owner]
A save belongs to the code that is running when SETLOCAL is called: the
program, a `::routine` or a method. An internal routine, with or without
[PROCEDURE](r:instr.procedure.scope), and the code that
[INTERPRET](r:instr.interpret.scope) runs are part of the code that runs them.
A save made in one of them is therefore still outstanding after it has
returned, the changes made after the save stay in force, and an ENDLOCAL in
the code that ran it takes the save back; a save made by that code can
likewise be taken back by an ENDLOCAL in one of them. A `::routine` or a
method sees only its own saves, as [ENDLOCAL in a
routine](r:bif.endlocal.restore) shows.

```rexx
call value 'SPEC_SETLOCAL_D', 'd0', 'ENVIRONMENT'
call sub
say value('SPEC_SETLOCAL_D',, 'ENVIRONMENT')
interpret "say endlocal()"
say value('SPEC_SETLOCAL_D',, 'ENVIRONMENT')
exit
sub: procedure
call setlocal
call value 'SPEC_SETLOCAL_D', 'd1', 'ENVIRONMENT'
return
```

```output
d1
1
d0
```

```rexx
call value 'SPEC_SETLOCAL_E', 'e0', 'ENVIRONMENT'
interpret "call setlocal; call value 'SPEC_SETLOCAL_E', 'e1', 'ENVIRONMENT'"
say value('SPEC_SETLOCAL_E',, 'ENVIRONMENT')
call sub
say value('SPEC_SETLOCAL_E',, 'ENVIRONMENT')
exit
sub:
say endlocal()
return
```

```output
e1
1
e0
```

r[bif.setlocal.return]
When a `::routine` or a method ends with saves of its own outstanding, the
oldest of them is put back: the code that called it sees the current
directory of that save, and each environment variable the save held has the
value it held, as [an ENDLOCAL](r:bif.endlocal.restore) would give. A
variable the program created after the save keeps its value, as
[ENDLOCAL](r:bif.endlocal.scope) says. The return of an internal routine
puts nothing back, by [the rule above](r:bif.setlocal.owner). What the end of
the program does with saves outstanding is not specified.

```rexx
call value 'SPEC_SETLOCAL_B', 'before', 'ENVIRONMENT'
here = directory()
call r
say value('SPEC_SETLOCAL_B',, 'ENVIRONMENT') (directory() == here),
    value('SPEC_SETLOCAL_N',, 'ENVIRONMENT')
exit
::routine r
call setlocal
call value 'SPEC_SETLOCAL_B', 'inside', 'ENVIRONMENT'
call value 'SPEC_SETLOCAL_N', 'new', 'ENVIRONMENT'
call directory '/'
say value('SPEC_SETLOCAL_B',, 'ENVIRONMENT') directory()
return
```

```output
inside /
before 1 new
```

```rexx
call value 'SPEC_SETLOCAL_C', 'c0', 'ENVIRONMENT'
here = directory()
.t~new~m
say value('SPEC_SETLOCAL_C',, 'ENVIRONMENT') (directory() == here)
exit
::class t
::method m
call setlocal
call value 'SPEC_SETLOCAL_C', 'c1', 'ENVIRONMENT'
call directory '/'
call setlocal
call value 'SPEC_SETLOCAL_C', 'c2', 'ENVIRONMENT'
say value('SPEC_SETLOCAL_C',, 'ENVIRONMENT') directory()
```

```output
c2 /
c0 1
```
