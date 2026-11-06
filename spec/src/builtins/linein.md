---
element: LINEIN
kind: builtin
syntax: "LINEIN([name] [, [line] [, count]])"
ansi: "9.7.4"
rexxref: "Functions > Built-in Functions > LINEIN (Line Input)"
---

# LINEIN

LINEIN reads a line from a [stream](r:bif.index.stream.name) and answers it
without its [line end](r:bif.index.stream.line-end). It reads from the
stream's read position, which a `line` argument can set first to the start of
a given line, and its `count` of 0 turns the read into a positioning.

## Arguments

r[bif.linein.args]
`name` is a [string](r:conv.string.null) that names the stream; when it is
omitted or null, the [standard input](r:bif.index.stream.std) is read. `line`
is a line number, and `count` a count: both are
[whole-number arguments](r:bif.index.stream.args), the line number a
[valid start](r:bif.index.stream.start). LINEIN is not a String method. It
takes three arguments at the most, by the [call rules](r:bif.index.call.too-many),
and none is required.

```rexx
f = 'spec-linein-args.tmp'
call sysfiledelete f
call lineout f, 'one'
call lineout f, 'two'
call lineout f
say linein(f, 2, ) linein(f, , 0)'|' linein(f, 1)
signal on syntax name a
say linein(f, 1, 1, 1)
a: say condition('o')~code
signal on syntax name b
say linein(f, 'x')
b: say condition('o')~code
signal on syntax name c
say linein(f, 0)
c: say condition('o')~code
call sysfiledelete f
```

```output
two | one
40.4
40.12
93.907
```

r[bif.linein.count]
`count` is 0 or 1, and 1 when it is omitted. A count of 1 reads a line and a
count of 0 [reads nothing](r:bif.linein.zero). A `count` that is greater than 1
raises `SYNTAX 93.0`, before the `line` is checked; a negative `count` raises
`SYNTAX 88.907`, as [the stream rule](r:bif.index.stream.args) says.

```rexx
f = 'spec-linein-count.tmp'
call sysfiledelete f
call lineout f, 'one'
call lineout f
signal on syntax name a
say linein(f, 1, 2)
a: say condition('o')~code
signal on syntax name b
say linein(f, 0, 2)
b: say condition('o')~code
signal on syntax name c
say linein(f, 1, -1)
c: say condition('o')~code
say linein(f, , 1) linein(f, 1, '1.0')
call sysfiledelete f
```

```output
93.0
93.0
88.907
one one
```

## The result

r[bif.linein.result]
LINEIN returns the characters from the read position up to the end of the
line, without the [line end](r:bif.index.stream.line-end), and moves the read
position to the start of the next line. A line of no characters is returned
as the null string and raises no condition. A line that ends the data without
a line end is returned whole and raises none either; the call after it finds
no line.

```rexx
f = 'spec-linein-result.tmp'
call sysfiledelete f
call charout f, 'a' || '0a'x || '0a'x || 'c d' || '0d0a'x || 'e' || '0d'x || 'f'
call charout f
n = 0
call on notready name nr
do 4
  x = linein(f)
  say length(x) c2x(x) n
end
x = linein(f)
say length(x) c2x(x) n
call sysfiledelete f
exit
nr: n = n + 1
return
```

```output
1 61 0
0  0
3 632064 0
3 650D66 0
0  1
```

> **Note.** The fourth line is `e`, a carriage return and `f`: the carriage return
> has no line feed after it, so the line does not end there.

r[bif.linein.partial]
A line that a read of characters has begun is read from where that read
stopped: LINEIN returns the rest of the line, and the read position moves to
the start of the next line.

```rexx
f = 'spec-linein-partial.tmp'
call sysfiledelete f
call lineout f, 'abcdef'
call lineout f, 'ghi'
call lineout f
say charin(f, , 2) linein(f) linein(f)
say charin(f, 9, 1) linein(f)
call sysfiledelete f
```

```output
ab cdef ghi
h i
```

r[bif.linein.line]
When `line` is given, LINEIN first sets the read position to the start of
line number `line`, the first line of the stream being line 1, and then
reads as it would without it. A line number is counted from the start of the
stream, so it names the same line wherever the read position was.

```rexx
f = 'spec-linein-line.tmp'
call sysfiledelete f
do i = 1 to 4
  call lineout f, 'line' i
end
call lineout f
say linein(f, 3) linein(f) linein(f, 1) linein(f, 2)
say linein(f, 4) linein(f, 1)
call sysfiledelete f
```

```output
line 3 line 4 line 1 line 2
line 4 line 1
```

r[bif.linein.zero]
When `count` is 0, LINEIN reads nothing and returns the null string. If
`line` is given, it still sets the read position to the start of that line;
the next read goes on from there.

```rexx
f = 'spec-linein-zero.tmp'
call sysfiledelete f
call lineout f, 'ab'
call lineout f, 'cd'
call lineout f, 'ef'
call lineout f
say '['linein(f, 3, 0)']' charin(f, , 2) '['linein(f, 2, 0)']' linein(f)
say '['linein(f, , 0)']' linein(f)
call sysfiledelete f
```

```output
[] ef [] cd
[] ef
```

## The end of the data

r[bif.linein.end]
When the read position is at the end of the data, or when `line` names a line
that the stream does not have, LINEIN returns the null string and raises
NOTREADY. A line number one more than the number of lines is the case for a
stream whose last line has a line end. A stream with no characters has no
line. The read position is then at the end of the data.

```rexx
f = 'spec-linein-end.tmp'
call sysfiledelete f
call lineout f, 'only'
call lineout f
n = 0
call on notready name nr
x = linein(f)
say '[' || x || ']' n
x = linein(f)
say '[' || x || ']' n
x = linein(f, 2)
say '[' || x || ']' n
x = linein(f, 99)
say '[' || x || ']' n chars(f)
x = linein(f, 1)
say '[' || x || ']' n
call sysfiledelete f
call charout f
x = linein(f)
say '[' || x || ']' n
call sysfiledelete f
exit
nr: n = n + 1
return
```

```output
[only] 0
[] 1
[] 2
[] 3 0
[only] 3
[] 4
```

r[bif.linein.missing]
A name that cannot be opened for reading, a file that does not exist or a
directory for example, has no lines: LINEIN returns the null string and raises
NOTREADY, and creates no file. Its `line` is not checked (see
[the start rule](r:bif.index.stream.start)), but a `count` that is not
valid is.

```rexx
n = 0
call on notready name nr
x = linein('spec-linein-nofile.tmp')
say '[' || x || ']' n
x = linein('spec-linein-nofile.tmp', 0, 1)
say '[' || x || ']' n
x = linein('.', 1)
say '[' || x || ']' n
say stream('spec-linein-nofile.tmp', 'c', 'query exists') == ''
signal on syntax name a
say linein('spec-linein-nofile.tmp', 1, 2)
a: say condition('o')~code
exit
nr: n = n + 1
return
```

```output
[] 1
[] 2
[] 3
1
93.0
```

r[bif.linein.stdin]
Without a `name`, LINEIN reads a line of the standard input. At the end of
the input it returns the null string and raises NOTREADY.

```rexx
call on notready name nr
x = linein()
say '[' || x || ']' '[' || linein(, , 0) || ']'
exit
nr: say 'notready' condition('d')
return
```

```output
notready STDIN
[] []
```
