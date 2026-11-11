---
element: LINES
kind: builtin
syntax: "LINES([name] [, option])"
ansi: "9.7.6"
rexxref: "Functions > Built-in Functions > LINES (Lines Remaining)"
deviations: [54, 58]
---

# LINES

LINES tells whether a [stream](r:bif.index.stream.name) has lines left to read,
or, when asked, how many. It reads nothing and moves nothing.

## Arguments

r[bif.lines.args]
`name` is a [string](r:conv.string.null) that names the stream; when it is
omitted or null, the [standard input](r:bif.index.stream.std) is asked.
`option` is a string whose first character, in either case, is `N` or `C`; the
characters after it are ignored. An omitted `option` is `N`. An `option` that
is the null string, or begins with any other character, raises
`SYNTAX 40.904`. LINES is not a [String method](r:class.string.builtins). It
takes two arguments at the most, by the [call
rules](r:bif.index.call.too-many), and none is required.

```rexx
f = 'spec-lines-args.tmp'
call sysfiledelete f
call lineout f, 'a'
call lineout f, 'b'
call lineout f
say lines(f) lines(f, ) lines(f, 'n') lines(f, 'Normal') lines(f, 'c') lines(f, 'Count') lines(f, 'cx')
signal on syntax name a
say lines(f, 'x')
a: say condition('o')~code
signal on syntax name b
say lines(f, '')
b: say condition('o')~code
signal on syntax name c
say lines(f, 'N', 1)
c: say condition('o')~code
call sysfiledelete f
```

```output
1 1 1 1 2 2 2
40.904
40.904
40.4
```

## The result

r[bif.lines.normal]
With the option `N`, LINES answers 1 when a read would find a line, that is
when there is a character between the read position and the end of the
data, and 0 when there is not. It answers 0 at the end of the data without
raising a condition.

```rexx
f = 'spec-lines-normal.tmp'
call sysfiledelete f
call lineout f, 'a'
call lineout f
say lines(f) linein(f) lines(f)
call charout f, 'xy'
say lines(f) chars(f) charin(f, , 1) lines(f) charin(f, , 1) lines(f) chars(f)
call sysfiledelete f
```

```output
1 a 0
1 2 x 1 y 0 0
```

> **Note.** The characters `xy` were added after the line feed, and so after the
> read position; LINES counts them at once, like [CHARS](r:bif.chars.result).

r[bif.lines.count]
With the option `C`, LINES answers the number of lines that the stream has
from the read position to the end of the data, the number of line ends
there is, plus one when the data ends without a line end after a character.
It is the number of calls of [LINEIN](r:bif.linein.result) that find a line. A
read position inside a line counts the rest of that line as one. The answer
is [written as a count](r:bif.index.count.answer), and whether it is a new
object is [not specified](r:bif.index.count.answer.deviation.4).

```rexx
f = 'spec-lines-count.tmp'
call sysfiledelete f
call charout f, 'a' || '0a'x || '0a'x || 'cd' || '0d0a'x || 'ef'
call charout f
say lines(f, 'C')
say linein(f) lines(f, 'C')
say linein(f) lines(f, 'C')
say charin(f, , 1) lines(f, 'C')
say c2x(charin(f, , 3)) lines(f, 'C')
say charin(f, , 1) lines(f, 'C')
say charin(f, 9, 1) lines(f, 'C')
call sysfiledelete f
```

```output
4
a 3
 2
c 2
640D0A 1
e 1
f 0
```

> **Note.** After `c` is read, the rest of its line, `d` and the line end,
> still counts as one line, so the count stays 2. The next three characters
> are `d` and the two characters of the line end.

r[bif.lines.nothing]
LINES neither reads nor moves anything: the read position and the write
position are as they were, and the next read starts where it would have
started without the call.

```rexx
f = 'spec-lines-nothing.tmp'
call sysfiledelete f
call lineout f, 'one'
call lineout f, 'two'
call lineout f
say lines(f, 'C') lines(f, 'C') lines(f)
say linein(f) lines(f, 'C') lines(f, 'C') linein(f)
say lines(f, 'C') stream(f, 'c', 'query position read') stream(f, 'c', 'query position write')
call sysfiledelete f
```

```output
2 2 1
one 1 1 two
0 9 9
```

## The end of the data

r[bif.lines.missing]
A name that cannot be opened for reading, a file that does not exist or a
directory for example, has no lines: LINES answers 0, raises
[NOTREADY](r:bif.index.stream.notready) and creates no file. LINES raises no
condition at the end of the data of a stream that is open.

```rexx
n = 0
call on notready name nr
x = lines('spec-lines-nofile.tmp')
say x n (stream('spec-lines-nofile.tmp', 'c', 'query exists') == '')
x = lines('spec-lines-nofile.tmp', 'C')
say x n
x = lines('.', 'C')
say x n
f = 'spec-lines-end.tmp'
call sysfiledelete f
call charout f, 'a'
call charout f
x = charin(f)
x = lines(f)
say x n
call sysfiledelete f
exit
nr: n = n + 1
return
```

```output
0 1 1
0 2
0 3
0 3
```

## Deviations

r[bif.lines.deviation.54]
LINES of the standard input, or of any other transient stream, answers
whether input waits to be read, 0 for none and otherwise a positive number
that is not specified, and consumes none. When the input is empty, as it is
for the examples of this book, the answer is 0. ooRexx answers 1 to some of
these questions when the standard input is `/dev/null`, which holds no
line. For an empty pipe, a pipe that holds data, and a file, it answers as
this rule says.

```rexx
say lines() lines('stdin') lines(, 'C')
```

```output
0 0 0
```

r[bif.lines.deviation.58]
With the option `C`, LINES answers the lines that remain after the read
position, whatever was done to the stream before. ooRexx's count can be
wrong after a character read or a line positioning, and negative.

```rexx
f = 'spec-lines-dev58.tmp'
call sysfiledelete f
do i = 1 to 5
  call lineout f, 'line' i
end
call lineout f
say linein(f, 2) lines(f, 'C')
call charin f, 1, 0
say lines(f, 'C')
call sysfiledelete f
```

```output
line 2 3
5
```
