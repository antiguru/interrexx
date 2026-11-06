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

> **Note.** The examples run with their output captured, where no tone can be
> sounded, so every accepted call in them writes the bell character, code 7.
> Their output blocks are marked `escaped` and show it as `\a`, as
> [Reading an example](../introduction.md#reading-an-example) describes.

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
`frequency` and `duration` are whole numbers in the [form](r:conv.whole.form)
that rule gives. The value of each is its number rounded to 20 significant
digits, whatever the NUMERIC DIGITS setting; when the first digit dropped is
5 or more, the magnitude rounds up, otherwise down. The frequency must be
from 37 to 32767 and the duration from 0 to 60000. An argument that is not in
that form, whose rounded value has a fractional part other than zero, that
is out of its range, or that is the null string raises `SYNTAX 88.907`, and
so does an object whose string value fails these checks, such as `.nil`.
A rounded value whose magnitude is 10\*\*18 or more is not a whole number for
this purpose, whichever sign it has: it fails the form check, which comes
before the range checks, as [the order rule](r:bif.beep.order) shows.

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
signal on syntax name i
call beep '37.000000000000000001', 0
i: say condition('o')~code
signal on syntax name j
call beep 100, '60000.000000000000001'
j: say condition('o')~code
signal on syntax name k
call beep '36.9999999999999999994', 0
k: say condition('o')~code
signal off syntax
numeric digits 3
call beep ' + 100 ', '-0'
call beep '1e+2', '0.0e1'
call beep 99.99999999999999999999, 60000.0000000000000000001
call beep '37.0000000000000000001', '60000.0000000000000001'
call beep '36.9999999999999999995', 0
call beep 37, 0
call beep 32767, 0
call beep .mutablebuffer~new('12345'), 0
say 'done'
```

```output escaped
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
88.907
\a\a\a\a\a\a\a\adone
```

r[bif.beep.order]
A call with no third argument checks the frequency first and the duration
next, each for being present and a whole number, and only then checks the
ranges: a frequency that is not a whole number is reported before a missing
duration, and a missing duration before a frequency out of range. A
frequency of magnitude 10\*\*18 or more is not a whole number, so it is
reported before a missing duration, while a frequency below that magnitude
that is out of range is reported after it. A third argument is reported before
an argument out of range. When a call has a
third argument and also a frequency or duration that is missing or not a
whole number, which of the two errors it raises is not specified.

```rexx
signal on syntax name a
x = beep('x', )
a: say condition('o')~code
signal on syntax name b
x = beep(36, )
b: say condition('o')~code
signal on syntax name c
x = beep(, 'x')
c: say condition('o')~code
signal on syntax name d
x = beep(36, 0, 1)
d: say condition('o')~code
signal on syntax name e
x = beep(100, 60001, 1)
e: say condition('o')~code
signal on syntax name f
x = beep(1E18, )
f: say condition('o')~code
signal on syntax name g
x = beep(999999999999999999, )
g: say condition('o')~code
signal on syntax name h
x = beep('-1E18')
h: say condition('o')~code
signal on syntax name i
x = beep(36)
i: say condition('o')~code
```

```output
88.907
88.901
88.901
88.922
88.922
88.907
88.901
88.907
88.901
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

```output escaped
\a\a[] 0
\aVAR 0
```

r[bif.beep.bell]
When no tone can be sounded, as when the output of the program is not a
console, an accepted call writes one bell character, code 7, to standard
output, at the point of the call, and nothing else. It is not written through
the `.OUTPUT` monitor: a program that has redirected `.OUTPUT` still has the
bell written to standard output. A call that raises an error writes nothing.
When a tone can be sounded the program cannot rely on anything being
written.

```rexx
call beep 100, 0
say 'after'
signal on syntax name a
call beep 100, 60001
a: say condition('o')~code
sink = .counter~new
.output~destination(sink)
say 'hidden'
call beep 100, 0
.output~destination
say 'shown' sink~count
exit
::class counter
::attribute count
::method init
  expose count
  count = 0
::method say
  expose count
  count += 1
```

```output escaped
\aafter
88.907
\ashown 1
```
