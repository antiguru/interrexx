---
element: CHAROUT
kind: builtin
syntax: "CHAROUT([name] [, [string] [, start]])"
ansi: "9.7.2"
rexxref: "Functions > Built-in Functions > CHAROUT (Character Output)"
deviations: [60]
---

# CHAROUT

CHAROUT writes the characters of a string to a [stream](r:bif.index.stream.name)
and answers how many of them it could not write, which is 0 when the write
succeeded. Given a start it writes there; given no string it only sets the
write position or, with a name alone, closes the stream.

## Arguments

r[bif.charout.args]
`name` is a [string](r:conv.string.null) that names the stream; when it is
omitted or null, the [standard output](r:bif.index.stream.std) is written.
`string` is a string too. `start` is a [whole-number
argument](r:bif.index.stream.args) whose [value](r:bif.index.stream.value) the
stream rules give, and a [valid start](r:bif.index.stream.start). CHAROUT is
not a [String method](r:class.string.builtins). It takes three arguments at
the most, by the [call rules](r:bif.index.call.too-many), and none is
required.

```rexx
f = 'spec-charout-args.tmp'
call sysfiledelete f
signal on syntax name a
say charout(f, 'abc', 1, 1)
a: say condition('o')~code
signal on syntax name b
say charout(f, 'abc', 'two')
b: say condition('o')~code
signal on syntax name c
say charout(f, 'abc', 0)
c: say condition('o')~code
say charout(f, 12, ) charout(f, 'x', 1.0)
call charout f
say charin(f, 1, 9)
call sysfiledelete f
```

```output
40.4
40.12
93.907
0 0
x2
```

## The result

r[bif.charout.result]
CHAROUT writes the characters of `string`, in order, from the write position,
and moves the write position past them. A character written at a position
inside the data replaces the one there; written after the last character it
extends the data. The answer is
the number of characters of `string` not written, 0 when all were: it is
[written as a count](r:bif.index.count.answer), and whether it is a new
object is [not specified](r:bif.index.count.answer.deviation.4).

```rexx
f = 'spec-charout-result.tmp'
call sysfiledelete f
say charout(f, 'Jelly') charout(f, ' world') charout(f, 7 * 6)
call charout f, 'B', 1
say charin(f, 1, 20)
call sysfiledelete f
```

```output
0 0 0
Belly world42
```

r[bif.charout.start]
When `start` is given, it is the write position for this call: the first
character of the stream is at 1. If it is more than one past the last
character, the characters between the end of the data and `start` are
`'00'x`.

```rexx
f = 'spec-charout-start.tmp'
call sysfiledelete f
call charout f, 'ab'
call charout f, 'Q', 5
call charout f, 'R', 3
say c2x(charin(f, 1, 20))
call sysfiledelete f
```

```output
6162520051
```

r[bif.charout.position-only]
When `string` is omitted or null and `start` is given, CHAROUT writes
nothing, sets the write position to `start` and answers 0; the next write
that gives no start begins there. If both are omitted, the call is a
[close](r:bif.charout.close). A null `string` with no `start` writes nothing
and leaves the write position alone.

```rexx
f = 'spec-charout-pos.tmp'
call sysfiledelete f
call charout f, 'abcdef'
say charout(f, , 2) charout(f, '', 4)
call charout f, 'XY'
call charout f, ''
call charout f, 'Z'
say charin(f, 1, 20)
call sysfiledelete f
```

```output
0 0
abcXYZ
```

r[bif.charout.close]
When both `string` and `start` are omitted, CHAROUT closes the stream and
answers 0. The characters written are in the stream afterwards, and its
[next use](r:bif.index.stream.lifetime) opens it afresh. CHAROUT opens a stream
that is not open in order to close it, so on a file that does not exist
this creates it; the same is true of every call of CHAROUT, even one that
writes nothing.
A standard stream is closed in the same way, and its next use opens it again,
as [the lifetime rule](r:bif.index.stream.lifetime) says; ooRexx differs (see
[Deviation 98](r:bif.index.stream.lifetime.deviation.98)).

```rexx
f = 'spec-charout-close.tmp'
call sysfiledelete f
say stream(f, 'c', 'query exists') == ''
say charout(f)
say (stream(f, 'c', 'query exists') \== '') chars(f)
call charout f, 'abc'
say charout(f) charin(f, , 1)
call sysfiledelete f
```

```output
1
0
1 0
0 a
```

## When the write fails

r[bif.charout.fail]
When the stream cannot be written, CHAROUT writes nothing, answers the number
of characters of `string`, and raises [NOTREADY](r:bif.index.stream.notready).
That is the case when the name cannot be opened for writing, as a name in a
directory that does not exist, or a directory, and when the name is `STDIN`.
For a name that cannot be opened, a `start` is not checked (see [the start
rule](r:bif.index.stream.start)). Nor is a `start` given with the name
`STDIN` checked beyond its form, whatever its value (see [the transient
rule](r:bif.index.stream.transient)): the call fails in the same way, and
without a `string` it answers 0.

```rexx
n = 0
call on notready name nr
x = charout('spec-charout-nodir/x.tmp', 'abc')
say x n
x = charout('.', 'hello', 0)
say x n
x = charout('stdin', 'q')
say x n
exit
nr: n = n + 1
return
```

```output
3 1
5 2
1 3
```

```rexx
n = 0
call on notready name nr
say charout('stdin', 'a', 0) n
say charout('STDIN:', 'abc', -1) n
say charout('stdin', , 1) n
say n
exit
nr: n = n + 1
return
```

```output defect=#91
1 0
3 1
0 2
3
```

## The standard streams

r[bif.charout.std]
Without a `name`, CHAROUT writes to the standard output, and with the name
`STDERR` to the standard error. A `start` for the standard output or the
standard error, when it is [transient](r:bif.index.stream.transient), raises
`SYNTAX 93.958`.

```rexx
call charout , 'a'
call charout 'STDOUT:', 'b'
say charout(, 'c')
signal on syntax name a
call charout , 'd', 5
a: say condition('o')~code
signal on syntax name b
call charout 'STDERR', , 3
b: say condition('o')~code
signal off syntax
say charout('STDERR', 'x')
```

```output
abc0
93.958
93.958
0
```

## Deviations

r[bif.charout.deviation.60]
A `'1a'x` at the end of a stream is data like any other character: a write
without a `start` goes after it, and a write at its position replaces it.
ooRexx treats a final `'1a'x` as the end of the file, and a write there
begins at its position.

```rexx
f = 'spec-charout-dev60.tmp'
call sysfiledelete f
call charout f, 'ab' || '1a'x
call charout f
call charout f, 'cd'
call charout f
say c2x(charin(f, 1, 20))
call sysfiledelete f
```

```output
61621A6364
```
