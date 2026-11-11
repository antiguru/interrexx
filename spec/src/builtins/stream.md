---
element: STREAM
kind: builtin
syntax: "STREAM(name [, operation [, command]])"
ansi: "9.7.8"
rexxref: "Functions > Built-in Functions > STREAM"
deviations: [34, 35, 49, 55, 57, 59, 62, 74, 79, 98, 100, 145]
---

# STREAM

STREAM asks a [stream](r:bif.index.stream.name) for its state, describes the
state, or carries out a command on the stream: opening and closing it,
moving its positions, and asking about the file behind it. The other stream
builtins open a stream when they need it, and the program that needs nothing
STREAM offers does not call it.

> **Note.** What a command answers about the host, a size or a time or a
> handle, is the host's. The examples here read back the file they make, and
> check the shape of an answer where its value belongs to the host.

## Arguments

r[bif.stream.args]
`name` is a [string](r:conv.string.null) that names the stream, and is
required: it must be neither omitted nor null. `operation` is a string that
chooses what STREAM does, and `command` is a string, the command that the
operation `C` carries out. STREAM is not a [String
method](r:class.string.builtins). It takes three arguments at the most, by the
[call rules](r:bif.index.call.too-many). The call is checked in this order.
The count of arguments comes first, by the [call
rules](r:bif.index.call.count). An omitted `name` then raises `SYNTAX 40.5`
and a null one `SYNTAX 40.27`; an `operation` that is [none of those of the
next rule](r:bif.stream.operation) raises `SYNTAX 40.904`. Last, a `command`
that is missing for the operation `C` raises `SYNTAX 40.3`, and a `command`
given for any other operation raises `SYNTAX 40.4`.

```rexx
f = 'spec-stream-args.tmp'
do e over "stream()", "stream(,'s')", "stream('','s')", "stream(f,'x')", "stream('','x')",,
          "stream(f,'c')", "stream(f,'s','close')", "stream(f,'d','close')",,
          "stream(f,'c','close','x')", "stream(f,,'close')", "stream(f,'x','close')",,
          "stream(f,'c','query exists')"
  say left(e, 28) '=>' try(e)
end
exit
try: signal on syntax name oops
  interpret 'r =' arg(1)
  return 'no error'
oops: return condition('o')~code
```

```output
stream()                     => 40.3
stream(,'s')                 => 40.5
stream('','s')               => 40.27
stream(f,'x')                => 40.904
stream('','x')               => 40.27
stream(f,'c')                => 40.3
stream(f,'s','close')        => 40.4
stream(f,'d','close')        => 40.4
stream(f,'c','close','x')    => 40.4
stream(f,,'close')           => 40.4
stream(f,'x','close')        => 40.904
stream(f,'c','query exists') => no error
```

r[bif.stream.operation]
The operation is `S` for the state of the stream, `D` for its description and
`C` for a command, and it is the first character of `operation`, in either
case, that decides: the characters after it are ignored. An omitted
`operation` is `S`.

```rexx
f = 'spec-stream-op.tmp'
say stream(f) stream(f, ) stream(f, 'S') stream(f, 'state') stream(f, 'Sxyz')
say '['stream(f, 'D')']' '['stream(f, 'description')']' '['stream(f, 'c', 'query exists')']'
```

```output
UNKNOWN UNKNOWN UNKNOWN UNKNOWN UNKNOWN
[UNKNOWN:] [UNKNOWN:] []
```

## State and description

r[bif.stream.state]
The state of a stream is one of four words. `UNKNOWN` is the state of a stream
that is not open. `READY` is that of an open stream whose last operation went
as asked. `NOTREADY` is that of an open stream whose last read or write could
not do all it was asked to, at the end of the data for example. `ERROR` is
that of a stream whose last read or write failed, such as a read of a name
that could not be opened, or a write to a stream that was opened for reading
only. The state belongs to the one stream: an operation on another changes
nothing in it.

```rexx
a = 'spec-stream-state-a.tmp'
b = 'spec-stream-state-b.tmp'
call sysfiledelete a
call sysfiledelete b
say stream(a) stream(b)
call charout a, 'xy'
call charout b, 'pq'
say stream(a) stream(b)
x = charin(a, 1, 5)
say stream(a) stream(b)
x = charin(b, 1, 1)
say stream(a) stream(b)
say stream('spec-stream-state-none.tmp', 'c', 'open read')
x = charin('spec-stream-state-none.tmp')
say stream('spec-stream-state-none.tmp')
call charout a
call charout b
call sysfiledelete a
call sysfiledelete b
```

```output
UNKNOWN UNKNOWN
READY READY
NOTREADY READY
NOTREADY READY
ERROR:2
ERROR
```

> **Note.** The number after `ERROR:` is the host's number for the failure. It
> is 2 where a file does not exist, on a Unix-like system. The
> [next rule](r:bif.stream.description) gives the form.

r[bif.stream.change]
Opening a stream, and every read, write, and `SEEK` or `POSITION` that
succeeds, leave it `READY`, or `NOTREADY` when a read or write could not do it
all, and a read or write that fails leaves `ERROR`. `CLOSE` leaves it
`UNKNOWN`. `FLUSH` and `QUERY` leave the state as it was.

```rexx
f = 'spec-stream-change.tmp'
call sysfiledelete f
call charout f, 'abc'
x = charin(f, 1, 9)
say stream(f) stream(f, 'c', 'query size') stream(f) stream(f, 'c', 'flush') stream(f)
say stream(f, 'c', 'seek 2') stream(f)
x = charin(f, 1, 1)
say stream(f) stream(f, 'c', 'close') stream(f)
call sysfiledelete f
```

```output
NOTREADY 3 NOTREADY READY: NOTREADY
2 READY
READY READY: UNKNOWN
```

r[bif.stream.description]
The description of a stream is its state, a colon, and more information about
the state when there is any. `READY:` and `UNKNOWN:` have none. A stream whose
last read stopped at the end of the data is `NOTREADY:EOF`. A stream in the
`ERROR` state has the host's number for the failure, and may have words
after it; in the examples that follow the number is not shown.

```rexx
f = 'spec-stream-desc.tmp'
call sysfiledelete f
say stream(f, 'd')
call charout f, 'ab'
say stream(f, 'd')
x = charin(f, 1, 5)
say stream(f, 'd')
call charout f
say stream(f, 'c', 'open read')
x = charout(f, 'q')
d = stream(f, 'd')
say left(d, 6) datatype(substr(d, 7, 2), 'W')
call charout f
x = charin('spec-stream-desc-none.tmp')
d = stream('spec-stream-desc-none.tmp', 'd')
say left(d, 6) datatype(substr(d, 7, 1), 'W')
call sysfiledelete f
```

```output
UNKNOWN:
READY:
NOTREADY:EOF
READY:
ERROR: 1
ERROR: 1
```

## Commands

r[bif.stream.command]
With the operation `C`, STREAM carries out `command`, which begins with a
word that names the command: `OPEN`, `CLOSE`, `FLUSH`, `SEEK`, `POSITION`
or `QUERY`. The word may be in any case, and may be shortened to any of its
leading letters down to the first. What is after the word is the command's
own, and the rules for each say what it is. A first word
that no command begins with raises `SYNTAX 93.914`, and anything the
command does not take after it, `SYNTAX 93.0`. A `command` that is the null
string, or only blanks, has no first word, and what it does is not specified.

```rexx
f = 'spec-stream-command.tmp'
call sysfiledelete f
call charout f, 'abc'
say stream(f, 'c', 'seek 2') stream(f, 'c', 'SEEK 2') stream(f, 'c', 's 2') stream(f, 'c', 'se 2')
say stream(f, 'c', 'position 3') stream(f, 'c', 'p 3') stream(f, 'c', 'f') stream(f, 'c', 'q size')
signal on syntax name a
say stream(f, 'c', 'seeker 2')
a: say condition('o')~code
signal on syntax name b
say stream(f, 'c', 'flush now')
b: say condition('o')~code
signal on syntax name c
say stream(f, 'c', 'frobnicate')
c: say condition('o')~code
call stream f, 'c', 'close'
call sysfiledelete f
```

```output
2 2 2 2
3 3 READY: 3
93.914
93.0
93.914
```

### OPEN

r[bif.stream.open.result]
`OPEN` opens the stream and answers `READY:`. If it cannot, it answers
`ERROR:` and the host's number for the failure, and raises
[NOTREADY](r:bif.index.stream.notready). A name that is a directory, or is in
a directory that does not exist, cannot be opened, and the stream is then not
open: its state is `UNKNOWN`. With no option the stream is opened for reading
and writing, and the `OPEN` of a file that does not allow both fails, where a
read alone would open it. A file that does not exist is created by `OPEN`,
unless it is opened for reading only. What follows `OPEN` is a list of
options, [the next rules](r:bif.stream.open.mode) give them. The words
`SHARED`, `SHAREREAD`, `SHAREWRITE` and `NOBUFFER` are accepted, and what they
do is not specified; so are their shortenings as far as `SHARER`, `SHAREW` and
`NOB`. Any other word in the list raises `SYNTAX 93.0`, a shortening of
`SHARED` among them.

```rexx
f = 'spec-stream-open.tmp'
call sysfiledelete f
n = 0
call on notready name nr
say stream(f, 'c', 'open read') (stream(f, 'c', 'query exists') == '')
say n
say stream(f, 'c', 'open') (stream(f, 'c', 'query exists') \== '')
say stream(f, 'c', 'close') stream(f, 'c', 'open nobuffer')
say stream(f, 'c', 'close')
say stream('.', 'c', 'open')
say n
say stream('spec-stream-nodir/x.tmp', 'c', 'open write')
say n
say stream('.') stream('spec-stream-nodir/x.tmp')
signal on syntax name a
say stream(f, 'c', 'open reading')
a: say condition('o')~code
call sysfiledelete f
exit
nr: n = n + 1
return
```

```output
ERROR:2 1
1
READY: 1
READY: READY:
READY:
ERROR:21
2
ERROR:2
3
UNKNOWN UNKNOWN
93.0
```

> **Note.** The numbers are those of a Unix-like system: 2 for a file that
> does not exist, and 21 for a directory. Another host may count differently.

r[bif.stream.open.mode]
The options of `OPEN` say what the stream is for and where it begins. `READ`
opens it for reading only, `WRITE` for writing only, and `BOTH`, which is the
default, for both. `APPEND` puts the write position after the last character,
which is where a stream that is open for writing and neither `APPEND` nor
`REPLACE` has it as well, and keeps it there: the write position cannot be
moved into the data that the file held when it was opened, by `SEEK` or by a
start or line that a write gives; see [Deviation
79](r:bif.stream.deviation.79). A position at the end of that data, or past
it, is allowed, but a write still goes after the last character. On a file
that the stream can only write, see [Deviation 74](r:bif.stream.deviation.74).
`REPLACE` empties the file and begins the write position at its start. An
option may be shortened as far as to `REA`, `W`, `BO`, `AP` and `REP`, and its
case does not matter. A word that is given twice, two of `READ`, `WRITE` and
`BOTH`, `READ` with `APPEND` or `REPLACE`, and `APPEND` with `REPLACE` raise
`SYNTAX 93.0`. A stream for reading only refuses a write: it fails, and the
state is `ERROR`.

```rexx
f = 'spec-stream-mode.tmp'
call sysfiledelete f
call charout f, 'abcdef'
call charout f
say stream(f, 'c', 'open write') stream(f, 'c', 'query position write')
say charout(f, 'X') stream(f, 'c', 'close') c2x(charin(f, 1, 100))
say stream(f, 'c', 'close')
say stream(f, 'c', 'open both replace') stream(f, 'c', 'query position write') stream(f, 'c', 'query size')
say charout(f, 'YZ') stream(f, 'c', 'close') charin(f, 1, 100)
say stream(f, 'c', 'close')
say stream(f, 'c', 'open read') charout(f, 'Q') left(stream(f, 'd'), 6) charin(f, 1, 1)
say stream(f, 'c', 'close')
say stream(f, 'c', 'open ap') stream(f, 'c', 'query position write') stream(f, 'c', 'close')
say stream(f, 'c', 'open bo rep') stream(f, 'c', 'query size') stream(f, 'c', 'close')
signal on syntax name a
say stream(f, 'c', 'open read replace')
a: say condition('o')~code
signal on syntax name b
say stream(f, 'c', 'open re')
b: say condition('o')~code
signal on syntax name c
say stream(f, 'c', 'open write read')
c: say condition('o')~code
call sysfiledelete f
```

```output
READY: 7
0 READY: 61626364656658
READY:
READY: 1 0
0 READY: YZ
READY:
READY: 1 ERROR: Y
READY:
READY: 3 READY:
READY: 0 READY:
93.0
93.0
93.0
```

r[bif.stream.open.again]
`OPEN` of a stream that is open opens it afresh: the positions are those
of a stream just opened, the read position at the first character and the
write position after the last, and the state is `READY`.

```rexx
f = 'spec-stream-again.tmp'
call sysfiledelete f
call charout f, 'abcdef'
x = charin(f, , 4)
say stream(f, 'c', 'query position read') stream(f, 'c', 'query position write')
say stream(f, 'c', 'open') stream(f, 'c', 'query position read') stream(f, 'c', 'query position write')
x = charin(f, , 9)
say stream(f)
say stream(f, 'c', 'open') stream(f)
call charout f
call sysfiledelete f
```

```output
5 7
READY: 1 7
NOTREADY
READY: READY
```

r[bif.stream.open.binary]
The option `BINARY RECLENGTH n` opens the stream as a stream of records of `n`
characters, `n` being 1 or more: its lines are not delimited by line ends but
are consecutive `n` characters, whatever they are. The words may be shortened
as far as to `BI` and `REC`. A line end in the data is data.
[LINEIN](r:bif.linein.result) answers a record, and the last record may be
short. [LINES](r:bif.lines.count) counts records.
[LINEOUT](r:bif.lineout.result) writes its string and then blanks up to the
end of the record that the write position is in, with no line end; a string
that is longer than the rest of that record raises `SYNTAX 93.0` and writes
nothing, and the stream stays `READY`. The option `BINARY` without `RECLENGTH`
is not specified, and neither is `BINARY RECLENGTH` without `n`, except that
when `BINARY` without `RECLENGTH` raises `SYNTAX 93.0`, as it does for a file
that is missing or empty, the stream is `UNKNOWN` afterwards ([Deviation
145](r:bif.stream.deviation.145)).

```rexx
f = 'spec-stream-binary.tmp'
call sysfiledelete f
call charout f, 'abcdefghij' || '0a'x || 'kl'
call charout f
say stream(f, 'c', 'open binary reclength 4')
say lines(f) lines(f, 'C') chars(f)
say '[' || linein(f) || ']' '[' || linein(f) || ']' c2x(linein(f)) lines(f, 'C')
say '[' || linein(f, 2) || ']' '[' || linein(f, 1) || ']'
call lineout f, 'XY', 2
say stream(f, 'c', 'close')
say c2x(charin(f, 1, 100))
call sysfiledelete f
```

```output
READY:
1 4 13
[abcd] [efgh] 696A0A6B 1
[efgh] [abcd]
READY:
6162636458592020696A0A6B6C
```

```rexx
f = 'spec-stream-binary2.tmp'
call sysfiledelete f
call charout f, 'ab'
call charout f
say stream(f, 'c', 'open binary reclength 4')
say lineout(f, '') lineout(f, 'xy')
signal on syntax name a
say lineout(f, 'abcde', 1)
a: say condition('o')~code stream(f)
say stream(f, 'c', 'close')
say '[' || translate(charin(f, 1, 100), '_', ' ') || ']'
call sysfiledelete f
```

```output
READY:
0 0
93.0 READY
READY:
[ab__xy__]
```

## Closing and flushing

r[bif.stream.close]
`CLOSE` closes the stream and answers `READY:`; its state is then `UNKNOWN`.
The characters written are in the file, and the next use of the name
[opens the stream afresh](r:bif.index.stream.lifetime). A stream that is not
open has nothing to close, and the answer is the null string. A standard
stream is closed in the same way: `CLOSE` of the standard input, output or
error answers `READY:` and leaves its state `UNKNOWN`, and the next stream
builtin that reads from it, writes to it, counts what is left in it (CHARS
or LINES) or opens it opens it again on the same input or output, so that
later reads and writes carry on, as [the lifetime
rule](r:bif.index.stream.lifetime) says. A `QUERY` or a `FLUSH` leaves the
state as it was.

```rexx
f = 'spec-stream-close.tmp'
call sysfiledelete f
say '['stream(f, 'c', 'close')']' stream(f)
call charout f, 'abc'
say stream(f) stream(f, 'c', 'close') stream(f) '['stream(f, 'c', 'close')']'
say chars(f) charin(f, , 1)
call charout f
call sysfiledelete f
```

```output
[] UNKNOWN
READY READY: UNKNOWN []
3 a
```

```rexx
say stream('stdin', 'c', 'close') stream('stdin')
say '['linein()']' stream('stdin')
```

```output
READY: UNKNOWN
[] NOTREADY
```

r[bif.stream.flush]
`FLUSH` answers `READY:`. A file can be asked about, or read back, by the
program through the stream functions as it is, whether `FLUSH` was used or
not, so that through the functions `FLUSH` has no other effect; a `Stream`
instance on the same file [may not see](r:class.stream.flush) what was
written before it.

```rexx
f = 'spec-stream-flush.tmp'
call sysfiledelete f
call charout f, 'abc'
say stream(f, 'c', 'flush') stream(f, 'c', 'query size') stream(f)
call charout f, 'd'
say stream(f, 'c', 'flush') stream(f, 'c', 'query size') chars(f)
call charout f, 'e'
s = .stream~new(f)
say s~chars
x = s~close
say stream(f, 'c', 'flush')
s = .stream~new(f)
say s~chars
x = s~close
call charout f
call sysfiledelete f
```

```output
READY: 3 READY
READY: 4 4
4
READY:
5
```

## Positions

r[bif.stream.seek]
`SEEK` sets a position of the stream and answers the new one. `POSITION` is
the same command. After the word there is an offset, and then options in any
order. The offset is a whole number, which may have a prefix: `=` or no prefix
for a position counted from the start of the data, 1 being its first
character; `<` for a position counted back from the end, `<0` being the one
past the last character; `+` and `-` for a position counted forward or back
from the read position with `READ`, and from the write position with `WRITE`
(see [Deviation 49](r:bif.stream.deviation.49)). `READ` sets the read
position, `WRITE` the write position, and when neither is given a stream open
for reading and writing has both set to the answer, and a `+` or `-` then
counts from the read position (see [Deviation 49](r:bif.stream.deviation.49)).
`CHAR` counts characters and
is the default; for `LINE` see [the next rule](r:bif.stream.seek.line). A
position past the end of the data is allowed: a read there finds nothing, and
a write there [fills the gap](r:bif.charout.start), except on a stream opened
`APPEND`, where [a write goes after the last
character](r:bif.stream.deviation.79) wherever the write position is. A stream
that is not open is opened by `SEEK`; when it cannot be, `SEEK` answers 0,
raises [NOTREADY](r:bif.index.stream.notready), leaves the state `ERROR`, and
creates nothing. An offset that is not a whole number or an option that is not
one of these raises `SYNTAX 93.0`, and a missing offset `SYNTAX 93.903`.

```rexx
f = 'spec-stream-seek.tmp'
call sysfiledelete f
call charout f, '0123456789'
say stream(f, 'c', 'seek 3') stream(f, 'c', 'query position read') stream(f, 'c', 'query position write')
say stream(f, 'c', 'seek =5 read') stream(f, 'c', 'query position read') stream(f, 'c', 'query position write')
say stream(f, 'c', 'seek +2 read') charin(f, , 1)
say stream(f, 'c', 'seek -2 read') charin(f, , 1)
say stream(f, 'c', 'seek <2 read') charin(f, , 1)
say stream(f, 'c', 'seek <0 read') stream(f, 'c', 'seek <1 write') stream(f, 'c', 'position 4 read')
say stream(f, 'c', 'seek read 4') stream(f, 'c', 'seek char write 6') stream(f, 'c', 'query position write')
say stream(f, 'c', 'seek 2 write') charout(f, 'W') c2x(charin(f, 1, 10))
say stream(f, 'c', 'seek 20 read') stream(f, 'c', 'query position read') stream(f)
say '[' || charin(f, , 2) || ']'
signal on syntax name a
say stream(f, 'c', 'seek x read')
a: say condition('o')~code
signal on syntax name b
say stream(f, 'c', 'seek 2 read write')
b: say condition('o')~code
signal on syntax name c
say stream(f, 'c', 'seek')
c: say condition('o')~code
call stream f, 'c', 'close'
call sysfiledelete f
```

```output
3 3 3
5 5 3
7 6
6 5
9 8
11 10 4
4 6 6
2 0 30573233343536373839
20 20 READY
[]
93.0
93.0
93.903
```

> **Note.** The data is `0123456789`. The character written after
> `seek 2 write` is `W`, in place of the `1`, as the hex string shows. The
> position `<2` is the one two characters back from the end of the data, the
> character `8`.

r[bif.stream.seek.line]
With `LINE`, the offset counts [lines](r:bif.index.stream.line-end) and not
characters: `=` counts from the first line, being line 1, and `+` and `-`
from the line the position is in. The position is set to the start of the
line, and the answer is the line number. A line number past the last line
sets the position to the end of the data, and the answer is the number of
the line that position is in. The prefix `<` with `LINE` is not specified.
A `SEEK` by line of a text stream that was opened for writing only answers 0
and raises NOTREADY; see [Deviation 55](r:bif.stream.deviation.55).

```rexx
f = 'spec-stream-line.tmp'
call sysfiledelete f
call lineout f, 'ab'
call lineout f, 'cd'
call lineout f, 'ef'
call lineout f
say stream(f, 'c', 'seek =2 read line') linein(f)
say stream(f, 'c', 'seek =3 read line') stream(f, 'c', 'seek -1 read line') linein(f)
say stream(f, 'c', 'seek =1 read line') stream(f, 'c', 'seek +2 read line') linein(f)
say stream(f, 'c', 'seek 9 read line') stream(f, 'c', 'query position read') chars(f)
say stream(f, 'c', 'seek 2 write line') charout(f, 'X') stream(f, 'c', 'close')
say c2x(charin(f, 1, 100))
call lineout f
call sysfiledelete f
```

```output
2 cd
3 2 cd
1 3 ef
4 10 0
2 0 READY:
61620A58640A65660A
```

r[bif.stream.query.position]
`QUERY POSITION` answers the read position of the stream, or, with the option
`WRITE`, the write position, with `READ` the read position; with no option a
stream open for reading and writing answers its read position, and one open
for one thing the position that it has. The option `CHAR` counts characters
and is the default; `LINE` counts lines, and the answer is the number of the
line that the position is in. Each option may be shortened to its first
letter. The option `SYS` may be given alone, and what it answers is not
specified. A stream that is not open has no positions, and the answer is the
null string; so is the answer for a standard stream that is
[transient](r:bif.index.stream.transient) (see [Deviation
100](r:bif.stream.deviation.100)). On a stream opened `APPEND`, the write
position after a write is just after the last character, wherever the write
was asked to go (see [Deviation 79](r:bif.stream.deviation.79)).

```rexx
f = 'spec-stream-qpos.tmp'
call sysfiledelete f
call charout f, 'ab' || '0a'x || 'cd' || '0a'x || 'ef'
call charout f
say '['stream(f, 'c', 'query position')']'
say stream(f, 'c', 'open')
say stream(f, 'c', 'query position') stream(f, 'c', 'query position read') stream(f, 'c', 'query position write')
x = charin(f, , 4)
say stream(f, 'c', 'query position') stream(f, 'c', 'query position read line') stream(f, 'c', 'query pos r c')
say stream(f, 'c', 'seek 1 write') stream(f, 'c', 'q pos w')
say stream(f, 'c', 'close')
say stream(f, 'c', 'open read') stream(f, 'c', 'query position')
call stream f, 'c', 'close'
call sysfiledelete f
```

```output
[]
READY:
1 1 9
5 2 5
1 1
READY:
READY: 1
```

## Asking about the file

r[bif.stream.query.exists]
`QUERY EXISTS` answers the full name of the file, as
[QUALIFY](r:bif.qualify.relative) makes it (see [Deviation
34](r:bif.stream.deviation.34)), if it exists, and the null string if not. A
directory is not a file for it, and the answer for one is the null string. It
does not create the file, and it does not depend on the stream being open. For
a standard stream the answer is its name: `STDIN`, `STDOUT` or `STDERR`.

```rexx
f = 'spec-stream-exists.tmp'
call sysfiledelete f
say '['stream(f, 'c', 'query exists')']' '['stream('.', 'c', 'query exists')']'
call charout f, 'abc'
say stream(f, 'c', 'query exists') == qualify(f)
call charout f
say stream(f, 'c', 'query exists') == qualify(f)
call sysfiledelete f
say '['stream(f, 'c', 'query exists')']'
say stream('stdin', 'c', 'query exists') stream('STDERR:', 'c', 'query exists')
```

```output
[] []
1
1
[]
STDIN STDERR
```

r[bif.stream.query.size]
`QUERY SIZE` answers the number of characters of the file, 0 for
an empty file and for a directory, and the null string for a name that is not
a file. Characters written are counted at once, whether or not the stream is
open.

```rexx
f = 'spec-stream-size.tmp'
call sysfiledelete f
say '['stream(f, 'c', 'query size')']' stream('.', 'c', 'query size')
call charout f
say stream(f, 'c', 'query size')
call charout f, 'a' || '0d0a'x
say stream(f, 'c', 'query size')
call charout f, 'bc'
say stream(f, 'c', 'query size') chars(f)
call charout f
call sysfiledelete f
```

```output
[] 0
0
3
5 5
```

r[bif.stream.query.type]
`QUERY STREAMTYPE` answers `PERSISTENT` for an open file, a stream that can be
positioned, `TRANSIENT` for an open stream that cannot, such as the standard
input when it is a pipe, and `UNKNOWN` for a stream that is not open: a file
that is closed, a name that is not a file, a directory.

```rexx
f = 'spec-stream-type.tmp'
call sysfiledelete f
say stream(f, 'c', 'query streamtype') stream('.', 'c', 'query streamtype')
call charout f, 'a'
say stream(f, 'c', 'query streamtype') stream('stdin', 'c', 'query streamtype')
call charout f
say stream(f, 'c', 'query streamtype')
call sysfiledelete f
```

```output
UNKNOWN UNKNOWN
PERSISTENT TRANSIENT
UNKNOWN
```

r[bif.stream.query.time]
`QUERY TIMESTAMP` answers the date and time that the file was last changed
as `yyyy-mm-dd hh:mm:ss`, and `QUERY DATETIME` the same moment as
`mm-dd-yy hh:mm:ss`; the null string for a name that is not a file. For a
directory they answer its time. They hold the host's time zone, which
is not specified.

```rexx
f = 'spec-stream-time.tmp'
call sysfiledelete f
say '['stream(f, 'c', 'query timestamp')']' '['stream(f, 'c', 'query datetime')']'
call charout f, 'a'
call charout f
ts = stream(f, 'c', 'query timestamp')
dt = stream(f, 'c', 'query datetime')
say translate(ts, copies('9', 10), '0123456789') translate(dt, copies('9', 10), '0123456789')
say (substr(dt, 1, 2) == substr(ts, 6, 2)) (substr(dt, 4, 2) == substr(ts, 9, 2)),
    (substr(dt, 7, 2) == substr(ts, 3, 2)) (substr(dt, 10) == substr(ts, 12))
say length(stream('.', 'c', 'query timestamp')) length(stream('.', 'c', 'query datetime'))
call sysfiledelete f
```

```output
[] []
9999-99-99 99:99:99 99-99-99 99:99:99
1 1 1 1
19 17
```

r[bif.stream.query.handle]
`QUERY HANDLE` answers the host's handle for a stream that is open, a whole
number, and the null string for one that is not.

```rexx
f = 'spec-stream-handle.tmp'
call sysfiledelete f
say '['stream(f, 'c', 'query handle')']'
call charout f, 'a'
say datatype(stream(f, 'c', 'query handle'), 'W')
call charout f
say '['stream(f, 'c', 'query handle')']'
call sysfiledelete f
```

```output
[]
1
[]
```

r[bif.stream.query.words]
The word after `QUERY` is `EXISTS`, `SIZE`, `STREAMTYPE`, `TIMESTAMP`,
`DATETIME`, `HANDLE` or `POSITION`, in either case. It may be shortened to any
of its leading letters, as far as to its first letter for `EXISTS`, `HANDLE`,
`TIMESTAMP` and `DATETIME`, to `SI` for `SIZE`, to `ST` for `STREAMTYPE` and
to `P` for `POSITION`; what a shorter word answers is not specified. A `QUERY`
with no word is not specified. A word that is none of them, and words after it
that the query does not take, raise `SYNTAX 93.0`.

```rexx
f = 'spec-stream-words.tmp'
call sysfiledelete f
call charout f, 'abc'
say stream(f, 'c', 'query size') stream(f, 'c', 'q si') stream(f, 'c', 'QUERY ST') stream(f, 'c', 'query streamtype')
say (stream(f, 'c', 'query e') == qualify(f)) length(stream(f, 'c', 'query t')) length(stream(f, 'c', 'query d'))
signal on syntax name a
say stream(f, 'c', 'query streamtypes')
a: say condition('o')~code
signal on syntax name b
say stream(f, 'c', 'query size 1')
b: say condition('o')~code
call charout f
call sysfiledelete f
```

```output
3 3 PERSISTENT PERSISTENT
1 19 17
93.0
93.0
```

## The standard streams

r[bif.stream.std]
The standard input, output and error streams are open from the start of the
program, and their state is `READY`. Their names are those of
[the standard names](r:bif.index.stream.std). The standard input is
transient when it is a pipe, and the commands that need positions are not for
a transient stream: a `SEEK` or `POSITION` of it raises `SYNTAX 93.958`.

```rexx
say stream('stdin') stream('STDOUT:') stream('stderr') stream('stdin', 'd')
signal on syntax name a
say stream('stdin', 'c', 'seek 1')
a: say condition('o')~code
```

```output
READY READY READY READY:
93.958
```

## Deviations

r[bif.stream.deviation.34]
In a stream's name, as in every name interrexx qualifies, each `..` removes
exactly the one component before it, however the component was written, so
`.b/../c` names the file `c` beside the directory `.b`. ooRexx removes none
after a doubled slash and two after a component that starts with a period, so
its stream can be a different file.

```rexx
address system 'mktemp -d' with output stem t.
base = t.1
call sysmkdir base || '/.b'
f = base || '/c'
call charout f, 'abc'
call charout f
g = base || '/.b/../c'
say (stream(g, 'c', 'query exists') == f) chars(g) charin(g, 1, 3)
call stream g, 'c', 'close'
call sysfiledelete f
call sysrmdir base || '/.b'
call sysrmdir base
```

```output
1 3 abc
```

r[bif.stream.deviation.35]
The names `HANDLE:0`, `HANDLE:1` and `HANDLE:2` denote the standard input,
output and error streams, and read and write them; a `HANDLE:` stream with any
other descriptor is refused when it is created. In ooRexx a `HANDLE:` stream
never reaches its descriptor: reads see end of file, and writes crash.

```rexx
call charout 'HANDLE:1', 'hello '
call charout 'handle:1', 'again'
say
say stream('HANDLE:1') '['charin('HANDLE:0')']'
```

```output
hello again
READY []
```

r[bif.stream.deviation.49]
On a stream open for reading and writing, a relative `SEEK` with `READ` moves
from the read position, and one with `WRITE` from the write position, whatever
operation came before. One with neither option moves from the read position.
ooRexx counts from a position that it tracks, which can be the other one: with
neither option, the position that the last read, write or `SEEK` used, and the
write position when there has been none since the stream was opened.

```rexx
f = 'spec-stream-dev49.tmp'
call sysfiledelete f
call lineout f, 'ab'
call lineout f, 'cd'
call lineout f, 'ef'
call lineout f
say stream(f, 'c', 'open')
say stream(f, 'c', 'seek +1 read') stream(f, 'c', 'query position read')
say stream(f, 'c', 'open')
x = charin(f, , 3)
say stream(f, 'c', 'seek +1 write') stream(f, 'c', 'query position read')
say stream(f, 'c', 'open')
say stream(f, 'c', 'seek +2') stream(f, 'c', 'query position write')
x = charin(f, , 2)
call charout f, 'X'
say stream(f, 'c', 'seek -1') stream(f, 'c', 'query position write')
call stream f, 'c', 'close'
call sysfiledelete f
```

```output
READY:
2 2
READY:
11 4
READY:
3 3
4 4
```

r[bif.stream.deviation.55]
A stream opened `WRITE` reads nothing: a read fails and the state is `ERROR`,
`CHARS` and `LINES` answer 0, and positioning by line is `NOTREADY`: a `SEEK`
by line answers 0 and a LINEOUT that gives a line answers 1. ooRexx opens it
for reading and writing.

```rexx
f = 'spec-stream-dev55.tmp'
call sysfiledelete f
call charout f, 'ab' || '0a'x || 'cd'
call charout f
say stream(f, 'c', 'open write')
x = charin(f, 1, 1)
say '[' || x || ']' stream(f) chars(f) lines(f) lines(f, 'C')
n = 0
call on notready name nr
say stream(f, 'c', 'seek 1 write line')
say n
say lineout(f, 'M', 1)
say n
call stream f, 'c', 'close'
call sysfiledelete f
exit
nr: n = n + 1
return
```

```output
READY:
[] ERROR 0 0 0
0
1
1
2
```

r[bif.stream.deviation.57]
`QUERY POSITION ... LINE` answers the line that the position is in, and 0 for
the write position of a stream open for reading only. ooRexx answers a line
number it tracks.

```rexx
f = 'spec-stream-dev57.tmp'
call sysfiledelete f
call charout f, 'ab' || '0a'x || 'cd' || '0a'x || 'ef'
call charout f
say stream(f, 'c', 'open')
say stream(f, 'c', 'query position write line') stream(f, 'c', 'query position write')
say stream(f, 'c', 'close')
say stream(f, 'c', 'open read')
say stream(f, 'c', 'query position write line') stream(f, 'c', 'query position read line')
call stream f, 'c', 'close'
call sysfiledelete f
```

```output
READY:
3 9
READY:
READY:
0 1
```

r[bif.stream.deviation.59]
A `SEEK` or `POSITION` to a position below 1 answers 0, raises NOTREADY and
moves nothing; so does one of the write position of a stream open for reading
only. ooRexx answers a position it has not checked.

```rexx
f = 'spec-stream-dev59.tmp'
call sysfiledelete f
call charout f, 'abc'
call charout f
say stream(f, 'c', 'open read')
n = 0
call on notready name nr
say stream(f, 'c', 'seek 0 read') stream(f, 'c', 'query position read') n
say stream(f, 'c', 'seek -9 read') stream(f, 'c', 'query position read') n
say stream(f, 'c', 'seek 2 write') n
call stream f, 'c', 'close'
call sysfiledelete f
exit
nr: n = n + 1
return
```

```output
READY:
0 1 0
0 1 1
0 2
```

r[bif.stream.deviation.62]
A read that finds nothing at the end of the data leaves the stream `NOTREADY`
with the description `NOTREADY:EOF`. ooRexx leaves `ERROR:0` after a `CHARIN`
of nothing.

```rexx
f = 'spec-stream-dev62.tmp'
call sysfiledelete f
call charout f, 'ab'
call charout f
x = charin(f, 1, 2)
say stream(f, 'd')
x = charin(f, , 1)
say stream(f, 'd')
call charout f
call sysfiledelete f
```

```output
READY:
NOTREADY:EOF
```

r[bif.stream.deviation.74]
A stream opened `APPEND` on a file it can only write writes every line at the
file's current end, after what another writer appended since the stream was
opened. ooRexx writes at the end as it was when the stream was opened,
overwriting what another writer appended meanwhile.

```rexx
f = 'spec-stream-dev74.tmp'
call sysfiledelete f
call lineout f, 'one'
call lineout f
address system 'chmod 200' f
call stream f, 'c', 'open write append'
address system 'printf "two\n" >>' f
call lineout f, 'three'
call lineout f
address system 'chmod 600' f
do while lines(f) > 0
  say linein(f)
end
call stream f, 'c', 'close'
call sysfiledelete f
```

```output
one
two
three
```

r[bif.stream.deviation.79]
On a stream opened `APPEND`, every write goes after the last character,
wherever the write position has been set: a position past the end leaves no
gap, and a position in data written since the stream was opened does not
overwrite it. The write position cannot be moved into the data that the file
held when it was opened: a `SEEK` of it there answers 0, and a CHAROUT or
LINEOUT that names a position there writes nothing, all of them `NOTREADY`.
After a write, the write position is just after the last character, wherever
the write was asked to go. ooRexx differs there: it moves the position, and
the write goes after the last character as any other does; the write position
it reports afterwards is the one that a CHAROUT start named, or that a `SEEK`
set, moved past the characters written.

```rexx
f = 'spec-stream-dev79.tmp'
call sysfiledelete f
call charout f, 'abcdefgh'
call charout f
n = 0
call on notready name nr
say stream(f, 'c', 'open append')
say stream(f, 'c', 'seek 2 write')
say n stream(f, 'c', 'query position write')
say charout(f, 'Y', 1)
say n
say stream(f, 'c', 'seek 9 write') charout(f, 'Z', 9)
say stream(f, 'c', 'seek 20 write') charout(f, 'W') charout(f, 'V', 10) n
say stream(f, 'c', 'query position write') charout(f, 'U', 100),
    stream(f, 'c', 'query position write')
call stream f, 'c', 'close'
say charin(f, 1, 100)
call sysfiledelete f
exit
nr: n = n + 1
return
```

```output
READY:
0
1 9
1
2
9 0
20 0 0 2
12 0 13
abcdefghZWVU
```

r[bif.stream.deviation.98]
`CLOSE` of the standard output or error closes it as it
[closes any stream](r:bif.stream.close), and the output written after it
appears. In ooRexx, that output does not appear at once: it appears when the
program next closes, flushes or opens the stream, and what is written after
the last of these is lost.
[Deviation 98](r:bif.index.stream.lifetime.deviation.98) gives the whole of
the difference.

```rexx
say 'one'
r = stream('stdout', 'c', 'close')
s = stream('stdout')
call lineout 'stdout', r s
say 'two' stream('stdout')
```

```output
one
READY: UNKNOWN
two READY
```

r[bif.stream.deviation.100]
`QUERY POSITION` of a standard stream that is
[transient](r:bif.index.stream.transient) answers the null string, with any
option, because a transient stream has no position. ooRexx answers 1.

```rexx
say '[' || stream('stdin', 'c', 'query position') || ']',
    '[' || stream('stdout', 'c', 'query position write') || ']',
    '[' || stream('stderr', 'c', 'query position read line') || ']'
```

```output
[] [] []
```

r[bif.stream.deviation.145]
When `open` with the option `BINARY` and no `RECLENGTH` raises `SYNTAX 93.0`,
as it does for a file that is missing and for one that is empty, the stream is
`UNKNOWN` afterwards, as after any open that fails. ooRexx leaves it `READY`;
the example is not run against it.

```rexx
f = 'spec-stream-binary3-zq.tmp'
call sysfiledelete f
signal on syntax name a
say stream(f, 'c', 'open binary')
a: say 'missing' condition('o')~code stream(f)
call charout f
call charout f
s = .stream~new(f)
signal on syntax name b
say s~open('binary')
b: say 'empty' condition('o')~code s~state
signal off syntax
call sysfiledelete f
```

```output
missing 93.0 UNKNOWN
empty 93.0 UNKNOWN
```
