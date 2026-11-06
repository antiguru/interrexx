---
element: SOURCELINE
kind: builtin
syntax: "SOURCELINE([n])"
ansi: "9.5.8"
rexxref: "Functions > Built-in Functions > SOURCELINE"
---

# SOURCELINE

SOURCELINE reads the text of the running program: how many lines it has, or
the text of one of them.

## Arguments

r[bif.sourceline.args]
SOURCELINE takes at most one argument, `n`, which is optional. The argument
count is checked by the [builtin call rules](r:bif.index.call.count): none is
required and one is the most, so a call with two arguments raises
`SYNTAX 40.4`, whatever they are, and a call with only omitted places counts as
a call with no arguments. SOURCELINE does not run as a String method: sending
the message `SOURCELINE` to a string raises `SYNTAX 97.1`.

```rexx
say sourceline() sourceline( ) sourceline(,)
signal on syntax name a
say sourceline(1, 2)
a: say condition('o')~code
signal on syntax name b
say 'x'~sourceline
b: say condition('o')~code
/* a last line */
```

```output
8 8 8
40.4
97.1
```

r[bif.sourceline.n.check]
`n` is a whole number that passes the
[whole-number rules](r:conv.whole.form) and whose value is at least 1. A value
of 0 or less raises `SYNTAX 40.14`; `-0` is 0. A string that is not a whole
number raises `SYNTAX 40.12`, the null string included, and so does a number
whose magnitude is [too large](r:conv.whole.digits). A whole number greater
than the number of lines is the case of
[its own rule](r:bif.sourceline.past-end). The caller's NUMERIC DIGITS plays no
part. These are not the range errors of the
[String-method builtins](r:conv.position.check).

```rexx
numeric digits 1
say length(sourceline(' 2 ')) length(sourceline(1e0)) length(sourceline('02.0'))
signal on syntax name a
say sourceline(0)
a: say condition('o')~code
signal on syntax name b
say sourceline('-0')
b: say condition('o')~code
signal on syntax name c
say sourceline('')
c: say condition('o')~code
signal on syntax name d
say sourceline(1.5)
d: say condition('o')~code
signal on syntax name e
say sourceline(1e30)
e: say condition('o')~code
signal on syntax name f
say sourceline('line')
f: say condition('o')~code
```

```output
80 16 80
40.14
40.14
40.12
40.12
40.12
40.12
```

## The result

r[bif.sourceline.count]
Without `n`, SOURCELINE answers the number of lines of the program whose
code is running. A line is a stretch of the program text between line ends, so
a clause that is continued, a comment that spans lines and a line that is empty
or only blanks each count by the lines they take, and the line end after the
last line starts no further line. The answer is
[written as a count](r:bif.index.count.answer), and whether it is a new object
is [not specified](r:bif.index.count.answer.deviation.4).

```rexx
say sourceline()
x = 'two',
  'lines'

/* a comment
   over
   three lines */

say sourceline()
```

```output
9
9
```

r[bif.sourceline.line]
With `n` not greater than the number of lines, SOURCELINE returns line `n` of
the program as it was written: all its characters, the blanks and tabs at its
start and at its end included, and no line end. An empty line answers the null
string. The example shows each tab as a period.

```rexx
src = .array~of('x = 1  ', '09'x || 'y = 2', '', ' ')
src~append('do i = 1 to sourceline() - 1; say i "["translate(sourceline(i), ".", "09"x)"]" length(sourceline(i)); end')
.routine~new('lines', src)~call
```

```output
1 [x = 1  ] 7
2 [.y = 2] 6
3 [] 0
4 [ ] 1
```

r[bif.sourceline.past-end]
When `n` is greater than the number of lines, SOURCELINE raises
`SYNTAX 40.34`, for every such `n` that passes its check. The last line is line
`SOURCELINE()`.

```rexx
last = sourceline()
say sourceline(last)
signal on syntax name a
say sourceline(last + 1)
a: say condition('o')~code
signal on syntax name b
say sourceline(999999999999999999)
b: say condition('o')~code
```

```output
b: say condition('o')~code
40.34
40.34
```

r[bif.sourceline.which]
SOURCELINE reads the program that holds the code that calls it. An internal
routine, a method, a `::ROUTINE` and an INTERPRET string read the program that
holds them; so does code in a required package, which reads that package's
source and not the requiring program's. A routine or package made from other
source text reads that text, which has one line for each item of the array it
was made from, an empty item included.

```rexx
say show() rt() .box~new~m
interpret 'say sourceline()'
r = .routine~new('other', .array~of('say sourceline()', '', 'say sourceline(1)', ''))
r~call
pk = .package~new('pk', .array~of('::routine pr public', 'return sourceline() sourceline(1)'))
say pk~findRoutine('PR')~call
exit
show: return sourceline()
::routine rt
return sourceline()
::class box
::method m
return sourceline()
```

```output
13 13 13
13
4
say sourceline()
2 ::routine pr public
```

> **Note.** The ANSI draft (9.5.8) defines SOURCELINE the same way, and raises
> 40.34 for a line past the end. The reference manual also mentions a program
> whose source is not available; this book does not specify that case. How a
> program file marks the end of a line, a carriage return before a line feed
> included, is not specified either.
