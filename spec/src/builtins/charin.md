---
element: CHARIN
kind: builtin
syntax: "CHARIN([name] [, [start] [, length]])"
ansi: "9.7.1"
rexxref: "Functions > Built-in Functions > CHARIN (Character Input)"
deviations: [62]
---

# CHARIN

CHARIN reads characters from a [stream](r:bif.index.stream.name) and answers
them as a string. It reads from the stream's read position, which a `start`
argument can set first, and it reads as many characters as `length` asks
for, one when `length` is omitted.

## Arguments

r[bif.charin.args]
`name` is a [string](r:conv.string.null) that names the stream; when it is
omitted or null, the [standard input](r:bif.index.stream.std) is read.
`start` and `length` are [whole-number arguments](r:bif.index.stream.args)
whose [value](r:bif.index.stream.value) the stream rules give.
CHARIN is not a String method. It takes three arguments at the most, by the
[call rules](r:bif.index.call.too-many), and none is required. `start` is
checked as the [stream rules](r:bif.index.stream.start) say, and `length` must
not be negative.

```rexx
f = 'spec-charin-args.tmp'
call sysfiledelete f
call charout f, 'abcdef'
call charout f
say charin(f, 2, ) charin(f, , 2) charin(, , 0)'|'
signal on syntax name a
say charin(f, 1, 1, 1)
a: say condition('o')~code
signal on syntax name b
say charin(f, 1, 'two')
b: say condition('o')~code
signal on syntax name c
say charin(f, 1, -1)
c: say condition('o')~code
signal on syntax name d
say charin(f, 0)
d: say condition('o')~code
call sysfiledelete f
```

```output
b cd |
40.4
40.12
88.907
93.907
```

r[bif.charin.length.default]
When `length` is omitted, it is 1.

```rexx
f = 'spec-charin-default.tmp'
call sysfiledelete f
call charout f, 'xyz'
call charout f
say charin(f) charin(f, ) charin(f, 3)
call sysfiledelete f
```

```output
x y z
```

## The result

r[bif.charin.result]
CHARIN returns the `length` characters that follow the read position, the
characters in the order they are in the stream, and moves the read position
past them. If fewer characters are left, it returns the ones that are left
and raises the [NOTREADY condition](r:bif.index.stream.notready). Every
character is data, a [line end](r:bif.index.stream.line-end) included, and so
is `'1a'x`.

```rexx
f = 'spec-charin-result.tmp'
call sysfiledelete f
call charout f, 'ab' || '0d0a'x || 'c' || '1a'x || 'd'
call charout f
say c2x(charin(f, 1, 2)) c2x(charin(f, , 2)) c2x(charin(f, , 3))
say c2x(charin(f, 1, 100))
call sysfiledelete f
```

```output
6162 0D0A 631A64
61620D0A631A64
```

> **Note.** The second line shows the short read at the end of the data: 100
> characters were asked for and the 7 that the stream has were returned.

r[bif.charin.start]
When `start` is given, it is the read position for this call: CHARIN reads
from the character at that position, the first of the stream being at 1,
and leaves the read position after the last character read.

```rexx
f = 'spec-charin-start.tmp'
call sysfiledelete f
call charout f, '0123456789'
call charout f
say charin(f, 4, 3) charin(f, , 2) charin(f, 1, 2) charin(f)
say charin(f, 10, 1) charin(f, 9, 5)
call sysfiledelete f
```

```output
345 67 01 2
9 89
```

## The end of the data

r[bif.charin.end]
When the read position is after the last character, CHARIN returns the null
string and raises the NOTREADY condition. The same holds for a `start` that
is after the last character, one past it or further, and the read position
is then left where it was.

```rexx
f = 'spec-charin-end.tmp'
call sysfiledelete f
call charout f, 'abc'
call charout f
n = 0
call on notready name nr
x = charin(f, 4)
say 'one' '[' || x || ']' n chars(f)
x = charin(f, 9, 2)
say 'two' '[' || x || ']' n
x = charin(f, 3)
say 'three' '[' || x || ']' n
x = charin(f)
say 'four' '[' || x || ']' n
call sysfiledelete f
exit
nr: n = n + 1
return
```

```output
one [] 1 3
two [] 2
three [c] 2
four [] 3
```

> **Note.** A call that reads exactly the characters that are left raises
> nothing; only the next call, which finds none, does. The example shows the
> condition raised after the clause that made the call, which is when a `CALL
> ON` trap runs.

r[bif.charin.zero]
When `length` is 0, CHARIN reads nothing and returns the null string. If
`start` is given and names a character of the stream, it still sets the read
position. A `length` of 0 raises NOTREADY only when `start` is after the last
character, and that is the [case of the end](r:bif.charin.end).

```rexx
f = 'spec-charin-zero.tmp'
call sysfiledelete f
call charout f, 'abcdef'
call charout f
n = 0
call on notready name nr
x = charin(f, 3, 0)
say '[' || x || ']' n charin(f, , 2)
x = charin(f, 7, 0)
say '[' || x || ']' n chars(f)
x = charin(f, 4, 0)
say '[' || x || ']' n chars(f)
call sysfiledelete f
exit
nr: n = n + 1
return
```

```output
[] 0 cd
[] 1 2
[] 1 3
```

r[bif.charin.missing]
A file that does not exist has no characters: CHARIN returns the null string
and raises NOTREADY, whatever `start` is and whatever the valid `length` is,
and creates no file. A name that cannot be opened for another reason, a
directory for example, is answered the same way. The `start` of such a call is
not checked (see [the start rule](r:bif.index.stream.start)), but a `length`
that is not valid is.

```rexx
n = 0
call on notready name nr
x = charin('spec-charin-nofile.tmp')
say '[' || x || ']' n
x = charin('spec-charin-nofile.tmp', 0, 5)
say '[' || x || ']' n
x = charin('.', 1, 1)
say '[' || x || ']' n
say stream('spec-charin-nofile.tmp', 'c', 'query exists') == ''
signal on syntax name a
say charin('spec-charin-nofile.tmp', 1, -1)
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
88.907
```

r[bif.charin.stdin]
Without a `name`, CHARIN reads the standard input. It returns up to
`length` characters of the input, and no more than the input holds. When
there is no character left in the input, it returns the null string and
raises NOTREADY.

```rexx
call on notready name nr
x = charin()
y = charin(, , 7)
say '[' || x || ']' '[' || y || ']' '[' || charin(, , 0) || ']'
exit
nr: say 'notready' condition('d')
return
```

```output
notready STDIN
notready STDIN
[] [] []
```

## Deviations

r[bif.charin.deviation.62]
A CHARIN that reads no character because the data is at its end leaves the
stream in the state `NOTREADY`, with the description `NOTREADY:EOF` (STREAM
answers them with its `S` and `D` options). ooRexx leaves the state `ERROR`,
with the description `ERROR:0`.

```rexx
f = 'spec-charin-dev62.tmp'
call sysfiledelete f
call charout f, 'ab'
call charout f
say charin(f, 1, 2) '['charin(f)']' stream(f, 's') stream(f, 'd')
call sysfiledelete f
```

```output
ab [] NOTREADY NOTREADY:EOF
```
