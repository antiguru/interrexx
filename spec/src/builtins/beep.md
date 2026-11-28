---
element: BEEP
kind: builtin
syntax: "BEEP(frequency, duration)"
rexxref: "Functions > Built-in Functions > BEEP"
deviations: [179]
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
raises `SYNTAX 97.1`. For the number of arguments it follows [the native call
rule](r:bif.index.call.native), so `BEEP(100, 0, )` is a call with two
arguments. The null string is not an omitted argument; it is a value that
[fails the checks below](r:bif.beep.range).

```rexx
signal on syntax name a
say beep()
a: say condition('c')
signal on syntax name b
say beep(100, 0, 1)
b: say condition('c')
signal on syntax name e
say '100'~beep(0)
e: say condition('o')~code
signal off syntax
say 'done'
```

```output
SYNTAX
SYNTAX
97.1
done
```

r[bif.beep.range]
`frequency` and `duration` are whole numbers in the [form](r:conv.whole.form)
that rule gives. The value of each is its number rounded to 20 significant
digits, whatever the NUMERIC DIGITS setting; when the first digit dropped is 5
or more, the magnitude rounds up, otherwise down. This is not the rule of [the
conventions](r:conv.whole.digits), which rounds to 18 digits and raises
`SYNTAX 40.12`. The frequency must be from 37 to 32767 and the duration from 0
to 60000. An argument that is not in that form, whose rounded value has a
fractional part other than zero, or that is the null string raises `SYNTAX 88.903`,
as does an object whose string value fails these checks, such as `.nil`.
A rounded value whose magnitude is 10\*\*18 or more also fails the whole-number
check. A whole number outside the frequency or duration range raises
`SYNTAX 88.907`.

```rexx
call try 36, 0
call try 32768, 0
call try 100, 60001
call try 100, -1
call try 1.5, 0
call try 100, ''
call try .nil, 0
call try 36, 'x'
exit
try:
  signal on syntax name e
  call beep arg(1), arg(2)
  say 'accepted'
  return
e: say condition('c')
  return
```

```output
SYNTAX
SYNTAX
SYNTAX
SYNTAX
SYNTAX
SYNTAX
SYNTAX
SYNTAX
```

r[bif.beep.order]
A call with more than one fault, such as a third argument together with a
frequency out of range, or a frequency that is not a whole number together
with a missing duration, raises the error of one of them, as [the precedence
rule](r:conv.error.precedence) says.

```rexx
signal on syntax name d
x = beep(100, 0, 1)
d: say condition('c')
signal on syntax name e
x = beep(36, 0, 1)
e: say condition('c')
signal on syntax name f
x = beep('x', )
f: say condition('c')
```

```output
SYNTAX
SYNTAX
SYNTAX
```

r[bif.beep.native-order]
The `Beep` routine of the REXX package checks the frequency's conversion and
range before checking the duration. A range error reports the supplied value,
including any surrounding blanks in a string.

```rexx
call try "pb(1)"
call try "pb(5,)"
call try "pb(32768, 'x')"
call try "pb(' 7 ', 0)"
call try "pb(40, ' -1 ')"
call try "pb(4, .duration~new)"
call try "pb(40,)"
call try "pb(40, 'x')"
exit
try:
  signal on syntax name failed
  interpret 'answer =' arg(1)
  say 'accepted'
  return
failed:
  o = condition('o')
  if o~code = 88.907 then say o~code '[' || o~additional[4] || ']'
  else say o~code
  return
::routine pb external 'LIBRARY REXX Beep'
::class duration
::method request
  say 'duration touched'
  return '0'
```

```output
88.907 [1]
88.907 [5]
88.907 [32768]
88.907 [ 7 ]
88.907 [ -1 ]
88.907 [4]
88.901
88.903
```

## The result

r[bif.beep.answer]
BEEP answers the null string, which is a string of length 0, for every
accepted pair of arguments. A call as a subroutine therefore sets `RESULT` to
the null string rather than dropping it. The test that cites this rule checks
it on interrexx; the example under [the deviation](r:bif.beep.deviation.179)
shows it with interrexx's bells.


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

## Deviations

r[bif.beep.deviation.179]
BEEP is a routine of the REXX package, as at language level 6.06
([Deviation 179](../deviations.md)). Its argument errors are those of [the
native call rule](r:bif.index.call.native) and of [the range
rule](r:bif.beep.range), of the 88.9xx family, and when no tone can be sounded
an accepted call writes a bell character to standard output, as [the bell
rule](r:bif.beep.bell) says. ooRexx, at level 6.07, makes BEEP a builtin: it
raises errors of the 40.xxx family for the same faults, accepts some values
that interrexx refuses, such as `36.9999999999999999994` and
`60000.000000000000001`, and writes no bell. The examples are those of the
rules above with the codes and the bells of interrexx.

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
signal on syntax name l
call beep '*', 1
l: say condition('o')~code
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
88.903
88.903
88.903
88.907
88.903
88.903
88.903
88.903
\a\a\a\a\a\a\a\adone
```

```rexx
signal on syntax name d
x = beep(100, 0, 1)
d: say condition('o')~code
signal on syntax name e
x = beep(36, 0, 1)
e: say wordpos(condition('o')~code, '88.922 88.907') > 0
signal on syntax name f
x = beep('x', )
f: say wordpos(condition('o')~code, '88.903 88.901') > 0
```

```output
88.922
1
1
```

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

```rexx
say '[' || beep(100, 0) || ']' length(beep(37, 0))
call beep 32767, 0
say symbol('RESULT') length(result)
```

```output escaped
\a\a[] 0
\aVAR 0
```
