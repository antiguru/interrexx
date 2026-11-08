---
element: ENDLOCAL
kind: builtin
syntax: "ENDLOCAL()"
rexxref: "Functions > Built-in Functions > ENDLOCAL (Linux only)"
---

# ENDLOCAL

ENDLOCAL takes back the most recent save that [SETLOCAL](setlocal.md) made: it
makes the directory the current one again, and gives the environment
variables the values they had when the save was made. It answers `1` when
there was a save to take back and `0` when there was none.

> **Note.** The examples use names that start with `SPEC_ENDLOCAL` for the
> environment variables they change. Each example takes at most one save
> back, counting the one that the reference implementation puts back at the
> end of a `::routine`, a method or the program, because that implementation
> can fail in a program that takes a save back a second time. The rule on
> nested saves therefore has no example; a test of interrexx backs it.

## Arguments

r[bif.endlocal.args]
ENDLOCAL takes no argument. A call with an argument raises `SYNTAX 40.4`, by
the [call rules](r:bif.index.call.too-many). A trailing omitted place adds
nothing to the count, so `ENDLOCAL(,)` is a call without an argument.
ENDLOCAL does not run as a String method: sending the message `ENDLOCAL` to a
string raises `SYNTAX 97.1`.

```rexx
signal on syntax name a
say endlocal(1)
a: say condition('o')~code
signal on syntax name b
say endlocal(, 'x')
b: say condition('o')~code
signal on syntax name c
say 'x'~endlocal
c: say condition('o')~code
signal off syntax
say endlocal(,)
```

```output
40.4
40.4
97.1
0
```

## The result

r[bif.endlocal.none]
A call when no save is outstanding answers `0` and changes nothing: not the
current directory, not an environment variable. A program that has never
called SETLOCAL is in this state.

> **Note.** In the reference implementation, a call with no save outstanding
> crashes the interpreter when SETLOCAL has been called earlier by the same
> program, `::routine` or method, its internal routines and
> [INTERPRET](r:instr.interpret.scope) included. It is specified here as it is
> before the first SETLOCAL, and the example does not go beyond that.

```rexx
here = directory()
call value 'SPEC_ENDLOCAL_A', 'a0', 'ENVIRONMENT'
say endlocal() endlocal()
say (directory() == here) value('SPEC_ENDLOCAL_A',, 'ENVIRONMENT')
```

```output
0 0
1 a0
```

r[bif.endlocal.restore]
A call when a save is outstanding takes the most recent one back and answers
`1`. The current directory is then the directory of the save, and each
environment variable that the save held has the value it held; the environment
variables that the program changed in between are therefore undone, and so is
a null-string value given to one of them. The call can be in an internal
routine or in code that [INTERPRET](r:instr.interpret.scope) runs, as [the
owner of a save](r:bif.setlocal.owner) allows. A call in a `::routine` or a
method does not see the saves of the code that called it: with no save of its
own outstanding it answers `0`, and the caller's save stays outstanding.

```rexx
here = directory()
call value 'SPEC_ENDLOCAL_B', 'b0', 'ENVIRONMENT'
call value 'SPEC_ENDLOCAL_C', 'c0', 'ENVIRONMENT'
call setlocal
call value 'SPEC_ENDLOCAL_B', 'b1', 'ENVIRONMENT'
call value 'SPEC_ENDLOCAL_C', '', 'ENVIRONMENT'
call directory '/'
say directory() value('SPEC_ENDLOCAL_B',, 'ENVIRONMENT')
call sub
say (directory() == here) value('SPEC_ENDLOCAL_B',, 'ENVIRONMENT'),
    value('SPEC_ENDLOCAL_C',, 'ENVIRONMENT')
exit
sub:
say endlocal()
return
```

```output
/ b1
1
1 b0 c0
```

```rexx
call value 'SPEC_ENDLOCAL_F', 'f0', 'ENVIRONMENT'
call setlocal
call value 'SPEC_ENDLOCAL_F', 'f1', 'ENVIRONMENT'
call r
.t~new~m
say value('SPEC_ENDLOCAL_F',, 'ENVIRONMENT')
exit
::routine r
say 'routine' endlocal()
::class t
::method m
say 'method' endlocal()
```

```output
routine 0
method 0
f1
```

r[bif.endlocal.nested]
Saves are taken back one at a time, the most recent first, and a save that
has been taken back is no longer outstanding: with two saves outstanding, a
call takes back the second, the next call takes back the first, and a third
call answers `0`.

> **Note.** The reference implementation fails on the second of these calls,
> so this rule has no example.

r[bif.endlocal.scope]
ENDLOCAL restores only the directory and the environment variables. A
variable of the program, the NUMERIC settings and the trace setting keep the
values they have when ENDLOCAL is called, and an environment variable that
did not exist when the save was made is not removed: it keeps the value the
program gave it. The commands that the program starts afterwards see the
restored environment.

```rexx
call value 'SPEC_ENDLOCAL_D', 'd0', 'ENVIRONMENT'
call setlocal
call value 'SPEC_ENDLOCAL_D', 'd1', 'ENVIRONMENT'
call value 'SPEC_ENDLOCAL_E', 'e1', 'ENVIRONMENT'
x = 5
numeric digits 12
trace o
call endlocal
say x digits() trace() value('SPEC_ENDLOCAL_E',, 'ENVIRONMENT')
address system 'echo "$SPEC_ENDLOCAL_D"' with output stem out.
say out.1
```

```output
5 12 O e1
d0
```
