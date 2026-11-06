---
element: ENDLOCAL
kind: builtin
syntax: "ENDLOCAL()"
rexxref: "Functions > Built-in Functions > ENDLOCAL"
---

# ENDLOCAL

ENDLOCAL takes back the most recent save that [SETLOCAL](setlocal.md) made: it
makes the directory the current one again, and gives the environment
variables the values they had when the save was made. It answers `1` when
there was a save to take back and `0` when there was none.

> **Note.** The examples use names that start with `SPEC_ENDLOCAL` for the
> environment variables they change. Each example calls ENDLOCAL at most once
> after a SETLOCAL that it made, because the reference implementation can fail
> in a program that restores a second time. The rule on nested saves
> therefore has no example of its own.

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
> after a SETLOCAL has run in the program crashes the interpreter. It is
> specified here as it is before the first SETLOCAL, and the example does not
> go beyond that.

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
environment variable that the save held has the value it held; the
environment variables that the program changed in between are therefore
undone, and so is a null-string value given to one of them. The save is
gone: the next call takes back the one made before it, or, when there is none,
answers `0`. Nested saves are taken back one at a time, the innermost first.
The call can be in another routine than the SETLOCAL call.

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
call endlocal
say x digits() value('SPEC_ENDLOCAL_E',, 'ENVIRONMENT')
address system 'echo "$SPEC_ENDLOCAL_D"' with output stem out.
say out.1
```

```output
5 12 e1
d0
```
