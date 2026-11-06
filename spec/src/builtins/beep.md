---
element: BEEP
kind: builtin
syntax: "BEEP(frequency, duration)"
rexxref: "Functions > Built-in Functions > BEEP"
---

# BEEP

BEEP asks the system to sound a tone of a given pitch and length, and answers
the null string. It is meant to be used as a subroutine, for its effect.

Whether a tone is heard depends on the system and on the terminal the program
runs on, and this book does not specify it. It specifies what a program can
check: the arguments that are accepted, the answer, and what is written to
standard output when no tone can be sounded.

> **Note.** The examples use a duration of 0 only, so that they do not wait.
> They run with their output captured, where no tone can be sounded, so every
> accepted call in them writes the bell character, which the output blocks
> show as a raw character of code 7 and which is invisible when the page is
> read.

## Arguments

r[bif.beep.args]
Both arguments are required. BEEP does not run as a String method: sending
the message `BEEP` to a string raises `SYNTAX 97.1`. It does not follow the
[call rules](r:bif.index.call.count) for the number of arguments, because
its errors are the 88.9xx family. An argument that is missing, whether the
call stops short of it or leaves its place empty, raises `SYNTAX 88.901`,
and a third argument raises `SYNTAX 88.922`. A trailing omitted place adds
nothing to the count, so `BEEP(100, 0, )` is a call with two arguments. The
null string is not an omitted argument; it is a value that
[fails the checks below](r:bif.beep.range).

```rexx
signal on syntax name a
say beep()
a: say condition('o')~code
signal on syntax name b
say beep(, 0)
b: say condition('o')~code
signal on syntax name c
say beep(100, )
c: say condition('o')~code
signal on syntax name d
say beep(100, 0, 1)
d: say condition('o')~code
signal on syntax name e
say '100'~beep(0)
e: say condition('o')~code
signal off syntax
say 'done'
```

```output
88.901
88.901
88.901
88.922
97.1
done
```

r[bif.beep.range]
`frequency` and `duration` are whole numbers, in the [form](r:conv.whole.form)
and with the [value](r:conv.whole.digits) that those rules give, whatever the
NUMERIC DIGITS setting. The frequency must be from 37 to 32767 and the
duration from 0 to 60000. An argument that is not a whole number, that is out
of its range, or that is the null string raises `SYNTAX 88.907`, and so does
an object with no string value, such as `.nil`. Both arguments are first
checked to be whole numbers, the frequency before the duration, and only then
is each checked against its range, again the frequency first.

```rexx
signal on syntax name a
call beep 36, 0
a: say condition('o')~code
signal on syntax name b
call beep 32768, 0
b: say condition('o')~code
signal on syntax name c
call beep 100, 60001
c: say condition('o')~code
signal on syntax name d
call beep 100, -1
d: say condition('o')~code
signal on syntax name e
call beep 1.5, 0
e: say condition('o')~code
signal on syntax name f
call beep 100, ''
f: say condition('o')~code
signal on syntax name g
call beep .nil, 0
g: say condition('o')~code
signal on syntax name h
call beep 36, 'x'
h: say condition('o')~code
signal off syntax
numeric digits 3
call beep ' + 100 ', '-0'
call beep '1e+2', '0.0e1'
call beep 99.99999999999999999999, 60000.0000000000000000001
call beep 37, 0
call beep 32767, 0
call beep .mutablebuffer~new('12345'), 0
say 'done'
```

```output
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
done
```

## The result

r[bif.beep.answer]
BEEP answers the null string, which is a string of length 0, for every
accepted pair of arguments. A call as a subroutine therefore sets `RESULT` to
the null string rather than dropping it.

```rexx
say '[' || beep(100, 0) || ']' length(beep(37, 0))
call beep 32767, 0
say symbol('RESULT') length(result)
```

```output
[] 0
VAR 0
```

r[bif.beep.bell]
When no tone can be sounded, as when the output of the program is not a
console, an accepted call writes one bell character, code 7, to standard
output, at the point of the call, and nothing else. A call that raises an
error writes nothing. When a tone can be sounded the program cannot rely on
anything being written.

```rexx
call beep 100, 0
say 'after'
signal on syntax name a
call beep 100, 60001
a: say condition('o')~code
```

```output
after
88.907
```
