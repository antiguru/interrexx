---
element: SAY
kind: instruction
syntax: "SAY [expression]"
ansi: "6.3.2.81, 8.3.23"
rexxref: "Keyword Instructions > SAY"
deviations: [40, 98]
---

# SAY

SAY writes one line of text to the standard output. It evaluates its
expression, converts the result to a string and writes that string followed by
a line end. Without an expression it writes an empty line. The instruction
does nothing else: it sets no variable, and it leaves the special variables
`RC` and `RESULT` as they were. The output goes to the same stream that the
builtins with no stream name write to, so SAY mixes with them in the order
the program ran them.

## What is written

r[instr.say.write]
The expression is everything after the keyword. SAY evaluates it, takes the
string value of the result and writes the string, then a line end. The
expression is an ordinary [expression](r:expr.index.value), so the operators
join, add and compare as they do anywhere else, and a number is written in
the form that the operation that made it gave it.

```rexx
numeric digits 4
x = 'abc'
say x 'def'
say 1 + 1 2 * 3
say 1 / 3
say (1 = 1) (1 = 2)
say 'a' || 'b' x
```

```output
abc def
2 6
0.3333
1 0
ab abc
```

r[instr.say.null]
A SAY with no expression writes a line end and nothing before it. So does a
SAY whose expression is the null string, and one whose value is a string of
blanks writes the blanks.

```rexx
say 'one'
say
say ''
say '  '
say 'two'
```

```output
one


  
two
```

r[instr.say.bytes]
SAY writes the characters of the string as they are. It does not change
control characters or other unprintable ones, it adds no padding and it does
not stop at a character that other contexts treat as an end of text, such as
`'00'x` or `'1A'x`. A line feed in the string is written as it is, so it
breaks the line where it stands, and the line end that SAY adds follows the
string. Which bytes make up the line end that SAY adds is not specified.

```rexx
say 'a' || '00'x || 'b' || '0D'x || '1A'x || '7F'x
say 'x' || '0A'x || 'y'
```

```output escaped
a\x00b\x0D\x1A\x7F
x
y
```

r[instr.say.object]
A result that is an object and not a string is turned into a string first,
as [the conversion rule](r:conv.string.object) says: the answer of its
`MAKESTRING` method when it has one, otherwise the answer of its `STRING`
method. The conversion happens before anything is written.

```rexx
say .viamake~new
say .viastring~new
say .both~new 'tail'
say .viamake~new||'!'
::class viastring
::method string
return 'viastring'
::class viamake
::method makestring
return 'made'
::class both
::method string
return 'plain'
::method makestring
return 'preferred'
```

```output
made
viastring
preferred tail
made!
```

## The output stream

r[instr.say.stream]
SAY writes to the standard output, the stream that [a builtin names with a
null name or none](r:bif.index.stream.std) when it writes. The line end is the
only thing SAY adds, so text that a CHAROUT call left unfinished on the
current line is followed by the SAY text on that same line, and a SAY is
followed by CHAROUT text on the next line. The text a program writes this
way appears in the order the program wrote it. The standard output is also
what `.OUTPUT` denotes by default, and what SAY does when a program has
redirected `.OUTPUT` is not specified here.

```rexx
call charout , 'p'
say 'c'
call charout , 'q'
call charout , 'r'
say 'd'
call lineout , 'e'
say 'f'
```

```output
pc
qrd
e
f
```

r[instr.say.result]
SAY does not set `RESULT` and does not drop it, and it does not change `RC`. A
routine that says something and then returns a value gives its caller that
value in `RESULT`, and the SAY has no part in it.

```rexx
result = 'keep'; rc = 'mine'
call giver
say 'got' result
say result rc
exit
giver: say 'in giver'
return 'given'
```

```output
in giver
got given
given mine
```

## Syntax

r[instr.say.keyword]
The word SAY starts the instruction in any case, and it may be followed
directly by a parenthesis, a quote or a sign, as [the keyword
rule](r:lex.clauses.keyword) says. A clause that begins with `say =` is an
assignment to a variable of that name and not a SAY, as [the
kind rule](r:lex.clauses.kinds) says, and so is a clause that begins with
`say.x =`, because a symbol with a period is not a keyword.

```rexx
Say(1+2)
SAY"q"
say-4
say = 'var'
say say
say.x = 'stem'
say say.x
```

```output
3
q
-4
var
stem
```

r[instr.say.error]
An error in the expression ends the SAY before anything is written, and the
clause writes no part of the line. The error is the one the expression raises,
for example `42.3` for a zero divisor, `41.1` for a name that is not a number
in arithmetic and `97.1` for a message that nothing answers. An expression
that cannot be read raises its `SYNTAX 35.1` or `37.2` when the program is
read, so a program with such a SAY runs nothing, as [the expression
rule](r:expr.index.syntax) says.

```rexx
call try 'say 1 / 0'
call try 'say "n" + u'
call try 'say .nosuch~foo'
call try 'say "partial" 1 / 0'
call try 'say 1 2 )'
call try 'say 1 +'
say 'done'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say 'ok' arg(1)
  return
e: say condition('o')~code '<=' arg(1)
  return
```

```output
42.3 <= say 1 / 0
41.1 <= say "n" + u
97.1 <= say .nosuch~foo
42.3 <= say "partial" 1 / 0
37.2 <= say 1 2 )
35.1 <= say 1 +
done
```

```rexx
say 'before'
say 1 +
```

```output rc=221
```

## Deviations

r[instr.say.deviation.40]
SAY and the builtins that write to the standard output share one stream, and
the text of all of them appears in the order the program wrote it. A standard
output that is a pipe or a file is buffered, and it is handed over at the
points that [Deviation 40](../deviations.md) names. ooRexx writes each line
at once, so text still buffered when the process is killed or crashes is lost
here and is not lost there.

```rexx
say 'one'
call charout , 'two'
call lineout , ' three'
say 'four'
```

```output
one
two three
four
```

r[instr.say.deviation.98]
After a close of the standard output (see [the stream lifetime
rule](r:bif.index.stream.lifetime.deviation.98)), SAY writes its line as
usual, and the output is opened again. In ooRexx the line does not
appear at once, and what is written after the last close, flush or open is
lost. SAY raises no `NOTREADY` condition in either, whatever the state of the
standard output, so the trap in the example does not run.

```rexx
say 'one'
call stream 'stdout', 'c', 'close'
signal on notready name nr
say 'two'
say 'three'
exit
nr: say 'notready'
```

```output
one
two
three
```
