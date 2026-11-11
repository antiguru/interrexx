---
element: LINEOUT
kind: builtin
syntax: "LINEOUT([name] [, [string] [, line]])"
ansi: "9.7.5"
rexxref: "Functions > Built-in Functions > LINEOUT (Line Output)"
deviations: [46, 56, 60]
---

# LINEOUT

LINEOUT writes a string and a line end to a [stream](r:bif.index.stream.name)
and answers 0 when it did, or 1 when it could not. Given a line number it
writes there; given neither a string nor a line number it closes the
stream.

## Arguments

r[bif.lineout.args]
`name` is a [string](r:conv.string.null) that names the stream; when it is
omitted or null, the [standard output](r:bif.index.stream.std) is written.
`string` is a string too. `line` is a line number: a [whole-number
argument](r:bif.index.stream.args) whose [value](r:bif.index.stream.value) the
stream rules give, and a [valid start](r:bif.index.stream.start). LINEOUT is
not a [String method](r:class.string.builtins). It takes three arguments at
the most, by the [call rules](r:bif.index.call.too-many), and none is
required.

```rexx
f = 'spec-lineout-args.tmp'
call sysfiledelete f
signal on syntax name a
say lineout(f, 'abc', 1, 1)
a: say condition('o')~code
signal on syntax name b
say lineout(f, 'abc', 'two')
b: say condition('o')~code
signal on syntax name c
say lineout(f, 'abc', 0)
c: say condition('o')~code
signal on syntax name d
say lineout(f, , 0)
d: say condition('o')~code
signal off syntax
say lineout(f, 12, ) lineout(f, 'x', 1.0)
call lineout f
say c2x(charin(f, 1, 9))
call sysfiledelete f
```

```output
40.4
40.12
93.907
93.907
0 0
780A0A
```

## The result

r[bif.lineout.result]
LINEOUT writes the characters of `string` and then a line end, in order,
from the write position, and moves the write position past them. A
character written at a position inside the data replaces the one there; a
character written after the last one extends the data. The answer is 0 when
all of it was written. The line end written is the one the platform uses: on
a Unix-like system, a line feed (`'0a'x`). A `string` that is the null
string writes a line end alone, and a `string` that holds line ends of its
own makes more than one line.

```rexx
f = 'spec-lineout-result.tmp'
call sysfiledelete f
say lineout(f, 'one') lineout(f, '') lineout(f, 12 * 2) lineout(f, 'a' || '0a'x || 'b')
call lineout f
say c2x(charin(f, 1, 100))
say linein(f, 1) '[' || linein(f) || ']' linein(f) linein(f) linein(f)
call lineout f
call sysfiledelete f
```

```output
0 0 0 0
6F6E650A0A32340A610A620A
one [] 24 a b
```

> **Note.** The line end is data of the stream like any other character, and
> [it counts as one](r:bif.index.stream.line-end) however the platform writes
> it.

r[bif.lineout.append]
The write position of a stream just opened is after the last character, so
that a LINEOUT without a `line` adds a line to the end of the data, and
the next one adds another after it. When the data ends without a line end,
because [characters were written](r:bif.charout.result) to it, LINEOUT
continues that last line.

```rexx
f = 'spec-lineout-append.tmp'
call sysfiledelete f
call lineout f, 'a'
call lineout f
call lineout f, 'b'
call charout f, 'c'
call lineout f, 'd'
call lineout f
say c2x(charin(f, 1, 100))
call lineout f
call sysfiledelete f
```

```output
610A620A63640A
```

## The line argument

r[bif.lineout.line]
When `line` is given, LINEOUT first sets the write position to the start of
line number `line`, the first line of the stream being line 1 and a line
being [counted by its line end](r:bif.index.stream.line-end), and then writes
as it would without it. A line number is counted from the start of the
stream, so it names the same line wherever the write position was. The
string replaces characters, not a line: a string that is shorter than the
line leaves the rest of its characters where they are, and a longer one
overwrites the beginning of the line that follows.

```rexx
f = 'spec-lineout-line.tmp'
call sysfiledelete f
call lineout f, 'abcdef'
call lineout f, 'ghi'
call lineout f, 'jkl'
call lineout f
call lineout f, 'XY', 1
call lineout f
say c2x(charin(f, 1, 100))
call lineout f, 'MNOPQRSTUV', 3
call lineout f
say c2x(charin(f, 1, 100))
call lineout f
call sysfiledelete f
```

```output
58590A6465660A6768690A6A6B6C0A
58590A6465660A4D4E4F505152535455560A
```

> **Note.** The first write puts `XY` and a line feed over `abc`, so that
> `def` is now line 2 and `ghi` line 3. The second write puts its ten letters
> and a line feed over `ghi` and `jkl` and their line ends, and goes on three
> characters past them.

r[bif.lineout.line.end]
A `line` that is past the last line of the data, one more than the number of
lines or more, sets the write position to the end of the data: nothing is
filled in, whatever the number. If the last line has no line end, the string
therefore continues it. LINEOUT raises no condition for such a `line`.

```rexx
f = 'spec-lineout-lineend.tmp'
call sysfiledelete f
call lineout f, 'a'
call lineout f, 'b'
call lineout f
call lineout f, 'c', 3
call lineout f, 'd', 99
call lineout f
call charout f, 'e'
call charout f
call lineout f, 'f', 9
call lineout f
say c2x(charin(f, 1, 100))
call lineout f
call sysfiledelete f
```

```output
610A620A630A640A65660A
```

r[bif.lineout.position-only]
When `string` is omitted and `line` is given, LINEOUT writes nothing, sets
the write position to the start of that line and answers 0; the next write
that gives no `line` begins there. A `string` that is the null string is
not omitted: it writes a line end at the start of the line.

```rexx
f = 'spec-lineout-pos.tmp'
call sysfiledelete f
call lineout f, 'abc'
call lineout f, 'def'
call lineout f
say lineout(f, , 2)
call lineout f, 'XYZ'
call lineout f
say c2x(charin(f, 1, 100))
call lineout f
say lineout(f, '', 1)
call lineout f
say c2x(charin(f, 1, 100))
call lineout f
call sysfiledelete f
```

```output
0
6162630A58595A0A
0
0A62630A58595A0A
```

r[bif.lineout.close]
When both `string` and `line` are omitted, LINEOUT closes the stream and
answers 0, as [CHAROUT given only a name](r:bif.charout.close) does. A
file that does not exist is created by it, and the characters written are in
the file afterwards.
A standard stream is closed in the same way, and its next use opens it again,
as [the lifetime rule](r:bif.index.stream.lifetime) says; ooRexx differs (see
[Deviation 98](r:bif.index.stream.lifetime.deviation.98)).

```rexx
f = 'spec-lineout-close.tmp'
call sysfiledelete f
say lineout(f) (stream(f, 'c', 'query exists') \== '') chars(f)
call lineout f, 'abc'
say lineout(f) lineout(f, , ) stream(f)
say chars(f) linein(f)
call lineout f
call sysfiledelete f
```

```output
0 1 0
0 0 UNKNOWN
4 abc
```

## When the write fails

r[bif.lineout.fail]
When the stream cannot be written, LINEOUT writes nothing, answers 1 and
raises [NOTREADY](r:bif.index.stream.notready). That is the case when the name
cannot be opened for writing, as a name in a directory that does not exist, or
a directory; when the stream was opened for reading only; and when the name is
`STDIN` and a `string` is given. For a name that cannot be opened, a `line` is
not checked (see [the start rule](r:bif.index.stream.start)). What a `line`
does together with the name `STDIN` is [not
specified](r:bif.index.stream.transient), and so is a call with that name and
neither a `string` nor a `line`.

```rexx
f = 'spec-lineout-fail.tmp'
call sysfiledelete f
call charout f, 'abc'
call charout f
n = 0
call on notready name nr
x = lineout('spec-lineout-nodir/x.tmp', 'abc')
say x n
x = lineout('spec-lineout-nodir/x.tmp', , 0)
say x n
x = lineout('.', 'hello')
say x n
x = lineout('stdin', 'q')
say x n
say stream(f, 'c', 'open read')
x = lineout(f, 'q')
say x n left(stream(f, 'd'), 6)
say stream(f, 'c', 'close') chars(f)
call sysfiledelete f
exit
nr: n = n + 1
return
```

```output
1 1
1 2
1 3
1 4
READY:
1 5 ERROR:
READY: 3
```

## The standard streams

r[bif.lineout.std]
Without a `name`, LINEOUT writes to the standard output, and with the name
`STDERR` to the standard error. A `line` for the standard output or the
standard error, when it is [transient](r:bif.index.stream.transient), raises
`SYNTAX 93.958`.

```rexx
call lineout , 'a'
call lineout 'STDOUT:', 'b'
say lineout(, 'c')
signal on syntax name a
call lineout , 'd', 5
a: say condition('o')~code
```

```output
a
b
c
0
93.958
```

## Deviations

r[bif.lineout.deviation.46]
On a stream opened `BINARY RECLENGTH n`, a LINEOUT that only positions, or
that closes the stream, clears no record, and only a LINEOUT with a
`string` pads its record. ooRexx clears or pads a record when LINEOUT
positions or closes.

```rexx
f = 'spec-lineout-dev46.tmp'
call sysfiledelete f
call charout f, 'abcdefgh'
call charout f
say stream(f, 'c', 'open binary reclength 4')
say lineout(f, , 1) lineout(f)
say '[' || charin(f, 1, 8) || ']'
call lineout f
call sysfiledelete f
```

```output
READY:
0 0
[abcdefgh]
```

r[bif.lineout.deviation.56]
A `line` counts from the start of the data, as [the line
rule](r:bif.lineout.line) says, by the line ends that are there, whichever
operation touched the stream before. ooRexx counts from a line number that it
tracks, which an operation on characters resets: a LINEOUT that gives a line
past the last one then writes past the end of the data, and the characters
between are `'00'x`.

```rexx
f = 'spec-lineout-dev56.tmp'
call sysfiledelete f
call lineout f, 'x'
call lineout f, 'y'
call lineout f, 'z', 2
call charin f, 1, 100
call lineout f, 'z', 3
call lineout f
say c2x(charin(f, 1, 100))
call lineout f
call sysfiledelete f
```

```output
780A7A0A7A0A
```

r[bif.lineout.deviation.60]
A `'1a'x` at the end of a stream is data like any other character: a write
without a `line` goes after it. ooRexx treats a final `'1a'x` as the end of
the file, and a write there begins at its position.

```rexx
f = 'spec-lineout-dev60.tmp'
call sysfiledelete f
call charout f, 'ab' || '1a'x
call charout f
call lineout f, 'cd'
call lineout f
say c2x(charin(f, 1, 100))
call lineout f
call sysfiledelete f
```

```output
61621A63640A
```
