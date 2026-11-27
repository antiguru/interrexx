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
Both arguments are required. BEEP does not run as a [String
method](r:class.string.builtins): sending the message `BEEP` to a string
raises `SYNTAX 97.1`. For the number of arguments it follows [the rules for
every builtin](r:bif.index.call.count), so `BEEP(100, 0, )` is a call with two
arguments and `BEEP(100, )` a call with one. The null string is not an omitted
argument; it is a value that [fails the checks below](r:bif.beep.range).

```rexx
signal on syntax name a
say beep()
a: say condition('o')~code
signal on syntax name b
say beep(100)
b: say condition('o')~code
signal on syntax name c
say beep(, 0)
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
40.3
40.3
40.5
40.4
97.1
done
```

r[bif.beep.range]
`frequency` and `duration` are [whole numbers](r:conv.whole.digits): each is
read in the form [that rule](r:conv.whole.form) gives, rounded to 18
significant digits whatever the NUMERIC DIGITS setting, and an argument that
is not a whole number in that sense, the null string and `.nil` included,
raises `SYNTAX 40.12`. The frequency must then be from 37 to 32767 and the
duration from 0 to 60000; a whole number outside its range raises `SYNTAX
40.927`.

```rexx
call try 36, 0
call try 32768, 0
call try 100, 60001
call try 100, -1
call try 1.5, 0
call try 100, ''
call try .nil, 0
call try 'x', 0
call try '36.9999999999999999994', 0
call try 100, '60000.000000000000001'
exit
try:
  signal on syntax name e
  call beep arg(1), arg(2)
  say 'accepted'
  return
e: say condition('o')~code
  return
```

```output escaped
40.927
40.927
40.927
40.927
40.12
40.12
40.12
40.12
\aaccepted
\aaccepted
```

r[bif.beep.order]
A call with more than one fault, such as a frequency that is out of range
together with a duration that is not a number, raises the error of one of them,
as [the precedence rule](r:conv.error.precedence) says.

```rexx
signal on syntax name d
x = beep(100, 0, 1)
d: say condition('o')~code
signal on syntax name e
x = beep(36, 'x')
e: say wordpos(condition('o')~code, '40.927 40.12') > 0
signal on syntax name f
x = beep('x', )
f: say wordpos(condition('o')~code, '40.3 40.12') > 0
```

```output
40.4
1
1
```

## The result

r[bif.beep.answer]
BEEP answers the null string, which is a string of length 0, for every
accepted pair of arguments. A call as a subroutine therefore sets `RESULT` to
the null string rather than dropping it. The example has the bells of [the
rule below](r:bif.beep.bell) in its output.

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
signal on syntax name a
call beep 100, 60001
a: say 'after' condition('c')
```

```output
after SYNTAX
```
