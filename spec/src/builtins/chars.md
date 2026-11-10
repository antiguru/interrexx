---
element: CHARS
kind: builtin
syntax: "CHARS([name])"
ansi: "9.7.3"
rexxref: "Functions > Built-in Functions > CHARS (Characters Remaining)"
deviations: [54]
---

# CHARS

CHARS answers how many characters a [stream](r:bif.index.stream.name) has left
to read. It reads nothing and moves nothing.

## Arguments

r[bif.chars.args]
`name` is a [string](r:conv.string.null) that names the stream; when it is
omitted or null, the [standard input](r:bif.index.stream.std) is asked. CHARS
is not a [String method](r:class.string.builtins). It takes one argument at
the most, by the [call rules](r:bif.index.call.too-many), and none is
required.

```rexx
f = 'spec-chars-args.tmp'
call sysfiledelete f
call charout f, 'abc'
call charout f
say chars(f) chars(f, )
signal on syntax name a
say chars(f, 1)
a: say condition('o')~code
signal on syntax name b
say chars(f, , 1)
b: say condition('o')~code
call sysfiledelete f
```

```output
3 3
40.4
40.4
```

## The result

r[bif.chars.result]
For a file, CHARS answers the number of characters from the read position to
the end of the data, the [line ends](r:bif.index.stream.line-end) among them.
It is 0 when no character is left. The answer is
[written as a count](r:bif.index.count.answer), and whether it is a new
object is [not specified](r:bif.index.count.answer.deviation.4).
Characters written to the stream are counted as soon as they are written, if
the read position is not past them.

```rexx
f = 'spec-chars-result.tmp'
call sysfiledelete f
call charout f, 'ab' || '0d0a'x || 'cd'
say chars(f)
say charin(f, , 2) chars(f)
call charout f, 'e'
say chars(f) '['charin(f, 7, 0)']' chars(f)
call sysfiledelete f
```

```output
6
ab 4
5 [] 1
```

r[bif.chars.missing]
A name that cannot be opened for reading, a file that does not exist or a
directory for example, has no characters: CHARS answers 0, raises
[NOTREADY](r:bif.index.stream.notready) and creates no file. CHARS raises no
condition at the end of the data.

```rexx
n = 0
call on notready name nr
x = chars('spec-chars-nofile.tmp')
say x n (stream('spec-chars-nofile.tmp', 'c', 'query exists') == '')
f = 'spec-chars-end.tmp'
call sysfiledelete f
call charout f, 'a'
call charout f
x = charin(f)
x = chars(f)
say x n
x = chars('.')
say x n
call sysfiledelete f
exit
nr: n = n + 1
return
```

```output
0 1 1
0 1
0 2
```

## Deviations

r[bif.chars.deviation.54]
CHARS of the standard input, or of any other transient stream, answers
whether characters wait to be read, 0 for none and a positive number
otherwise, and consumes none. The positive number is not specified. When the
input is empty, as it is for the examples of this book, the answer is 0.
ooRexx answers 1 to some of these questions when the standard input is
`/dev/null`, which holds no character. For an empty pipe, a pipe that holds
data, and a file, it answers as this rule says.

```rexx
say chars() chars('stdin')
```

```output
0 0
```
