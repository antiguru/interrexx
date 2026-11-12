---
element: "SysCreatePipe"
kind: function
syntax: "SysCreatePipe()"
rexxref: "Rexx Utilities (RexxUtil) > SysCreatePipe (Unix-like systems only)"
---

# SysCreatePipe

SysCreatePipe makes a pipe, a channel through which what is written at one end
can be read at the other, and answers the two numbers that name its ends.

## Arguments

r[util.syscreatepipe.args]
SysCreatePipe takes no argument. One argument that is a string raises `SYNTAX
40.920`, an object that has no string value raises `SYNTAX 88.909`, and two
arguments or more raise `SYNTAX 88.922`; an omitted place that is followed by
no argument is not one. ([The argument rule](r:util.index.args) says that an
argument past the last one of a function raises `88.922`; [Deviation
101](r:util.index.deviation.101) is about the order of that check.)

```rexx
call try "SysCreatePipe('x')"
call try "SysCreatePipe('')"
call try "SysCreatePipe(.nil)"
call try "SysCreatePipe(1, 2)"
call try "SysCreatePipe(, 1)"
say words(SysCreatePipe(,))
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
```

```output
40.920
40.920
88.909
88.922
88.922
2
```

## The answer

r[util.syscreatepipe.answer]
The answer is two whole numbers, a blank between them: the first names the end
of the pipe that is read, the second the end that is written. They are numbers
of open files of the program, which stay open for the rest of its run: a
command that ADDRESS starts has them too, and can write to the second number
and read from the first, as a shell does with `>&` and `<&` and a number (the example runs its
commands through `bash`, because some shells, `dash` among them, refuse a
number of 10 or more). What
is written is read in the order in which it was written. Two calls make two
pipes, whose four numbers are all different, and what is written to one is not
read from the other. Which numbers a call answers is not specified.

```rexx
parse value SysCreatePipe() with r w
parse value SysCreatePipe() with r2 w2
say words(r w) datatype(r, 'W') datatype(w, 'W') (r \= w)
say (r2 \= r) (r2 \= w) (w2 \= r) (w2 \= w) (r2 \= w2)
address system "bash -c 'echo through >&"w"'"
address system "bash -c 'echo other >&"w2"'"
address system "bash -c 'read x <&"r2"; echo got $x'" with output stem out.
say out.0 out.1
address system "bash -c 'read x <&"r"; echo got $x'" with output stem out.
say out.0 out.1
address system "bash -c 'echo first >&"w"'"
address system "bash -c 'echo second >&"w"'"
address system "bash -c 'read x <&"r"; echo got $x'" with output stem out.
say out.1
address system "bash -c 'read x <&"r"; echo got $x'" with output stem out.
say out.1
```

```output
2 1 1 1
1 1 1 1 1
1 got other
1 got through
got first
got second
```
