---
element: "CommandOutput"
kind: class
deviations: [109]
rexxref: "Builtin Classes > Utility Classes > CommandOutput Class"
---

# CommandOutput

A `CommandOutput` is the stream to which a handler written in Rexx writes the
output or the error output of a command that was sent with `ADDRESS ... WITH
OUTPUT` or `WITH ERROR`. The handler gets it from
[`output`](r:class.commandcontext.output) or
[`error`](r:class.commandcontext.error) of the
[`CommandContext`](commandcontext.md) of the command. What it is given goes to
the target of the redirection, a stem, a stream or the object of a `USING`, as
lines, in the way that [the redirection rules](r:instr.address.with.syntax)
say for any command. A command that is sent without a redirection has no such
stream; a handler that is to write then uses `SAY` or a stream of its own. The
class inherits the mixin class `OutputStream`, whose other methods, `arrayOut`
among them, are specified with the [stream classes](outputstream.md). This
page states the four methods that the class defines, how text becomes lines,
and when a line that is not finished is written.

## The class

r[class.commandoutput.creation]
`CommandOutput` has the instance methods `charOut`, `close`, `flush` and
`lineOut` besides those that it inherits. Its superclass is `Object`, and it
inherits the mixin class `OutputStream`: its `superClasses` are `Object` and
`OutputStream` in that order ([the hierarchy rule](r:class.index.hierarchy)).
The interpreter makes the objects for a command and hands them out, one for the
output and one for the error output; a program does not make one. The `new` of a
program with no arguments, or with one, raises `SYNTAX 93.901`, with more than
two `SYNTAX 93.902`, and what an object made with two arguments is good for is
not specified, since it belongs to no command. The class has other methods,
`init` among them, that the interpreter uses; they are not part of this
specification.

```rexx
s = ''
do c over .commandoutput~superclasses
  s = s c~id
end
say .commandoutput~id .commandoutput~superclasses~items s~strip
have = .set~new
m = .commandoutput~methods(.commandoutput)
do while m~available
  have~put(m~index)
  m~next
end
names = 'charOut close flush lineOut'~makeArray(' ')
count = 0
do name over names
  if have~hasItem(name~upper) then count = count + 1
end
say count 'of' names~items
call try '.commandoutput~new'
call try '.commandoutput~new("x")'
call try '.commandoutput~new("x", "y", "z")'
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
```

```output
CommandOutput 2 Object OutputStream
4 of 4
.commandoutput~new 93.901
.commandoutput~new("x") 93.901
.commandoutput~new("x", "y", "z") 93.902
```

## Writing

r[class.commandoutput.lineout]
`lineOut(string)` finishes a line and answers `0`. The line is the text that
`charOut` has written since the last line ended, followed by `string`; with no
`string`, or with an omitted place, the line is that text alone and nothing more
is added, so that `lineOut` alone ends a line that `charOut` began, and writes
nothing when no line is begun. A `string` that is not a string is replaced by
the answer of its `string` method, and not of its `makeString` or its `request`;
an instance of a subclass of `String` is a string here ([the deviation
rule](r:class.commandoutput.deviation.109)); `.nil` is `The NIL object`. A line
end inside `string` does not split the line: it is written as one line, with the
line end in it. `lineOut` takes one argument (`SYNTAX 93.902`). The example
gives the lines to a stem.

```rexx
x = .commandhandler~register('spec_co_lineout', .writer~new)
address spec_co_lineout 'go' with output stem o.
do i = 1 to o.0
  say i '<'o.i~changestr('0a'x, '\n')~changestr('0d'x, '\r')'>'
end
signal on syntax name bad
address spec_co_lineout 'bad' with output stem o.
bad: say condition('o')~code
x = .commandhandler~deregister('spec_co_lineout')
::class writer inherit CommandHandler
::method command
  use arg command, caller
  o = caller~output
  if command = 'bad' then return o~lineout('a', 'b')
  say o~lineout('first') o~lineout('') o~lineout o~lineout()
  o~charout('begun')
  say o~lineout
  o~charout('begun and ')
  say o~lineout('finished')
  say o~lineout('two' || '0a'x || 'lines') o~lineout('crlf' || '0d0a'x)
  say o~lineout(12.50) o~lineout(.nil) o~lineout(.both~new),
    o~lineout(.made~new)
  return 0
::class both
::method makestring
  return 'from makestring'
::method string
  return 'from string'
::class made
::method makestring
  return 'only makestring'
```

```output
0 0 0 0
0
0
0 0
0 0 0 0
1 <first>
2 <>
3 <begun>
4 <begun and finished>
5 <two\nlines>
6 <crlf\r\n>
7 <12.50>
8 <The NIL object>
9 <from string>
10 <a MADE>
93.902
```

r[class.commandoutput.charout]
`charOut(string)` writes text that need not be a whole line, and answers `0`.
A line end in the text, the character `'0A'x`, ends the line before it, and
the text after the last line end, which may be none, is kept as the beginning
of the next line, so that a line is made of what several calls of `charOut`
wrote and of the `lineOut` that ends it. A `'0D'x` is a character of the text
like another. Without `string`, with an omitted place, or with the null
string, `charOut` writes nothing. A `string` that is not a string is replaced
by its string value as for [`lineOut`](r:class.commandoutput.lineout). The
position that the stream method has as its second argument must be omitted
(`SYNTAX 93.963`), and a third argument raises `SYNTAX 93.902`. What is kept
is written when [the pending text rule](r:class.commandoutput.pending) says.

```rexx
x = .commandhandler~register('spec_co_charout', .writer~new)
address spec_co_charout 'go' with output stem o.
do i = 1 to o.0
  say i '<'o.i~changestr('0a'x, '\n')~changestr('0d'x, '\r')'>'
end
x = .commandhandler~deregister('spec_co_charout')
::class writer inherit CommandHandler
::method command
  use arg command, caller
  o = caller~output
  say o~charout('ab') o~charout o~charout('') o~charout('cd')
  say o~charout('0a'x),
    o~charout('ef' || '0a'x || 'gh' || '0a'x || '0a'x || 'ij')
  say o~charout('a' || '0d0a'x || 'b' || '0d'x || 'c')
  say o~charout(12.50) o~charout(.nil)
  do e over .array~of("o~charout('a', 1)", "o~charout(, 1)",,
      "o~charout('a', , 3)")
    self~try(o, e)
  end
  return 0
::method try
  use arg o, e
  signal on syntax name bad
  interpret 'x =' e
  say e 'no error'
  return
bad: say e condition('o')~code
  return
```

```output
0 0 0 0
0 0
0
0 0
o~charout('a', 1) 93.963
o~charout(, 1) 93.963
o~charout('a', , 3) 93.902
1 <abcd>
2 <ef>
3 <gh>
4 <>
5 <ija\r>
6 <b\rc12.50The NIL object>
```

r[class.commandoutput.pending]
The text that `charOut` has written after the last line end is not yet a line,
and it is kept. It is written as a line by the next `lineOut`, which appends
to it its own string, by [`flush`](r:class.commandoutput.flush), by
[`close`](r:class.commandoutput.close) and when the method `command` of the
handler returns, so that nothing is lost at the end of a command. The text
that was kept is written once, and it ends a line each time: after `flush` or
`close` a later `charOut` begins a new line.

```rexx
x = .commandhandler~register('spec_co_pending', .writer~new)
address spec_co_pending 'a' with output stem a.
address spec_co_pending 'b' with output stem b.
address spec_co_pending 'c' with output stem c.
address spec_co_pending 'd' with output stem d.
x = .commandhandler~deregister('spec_co_pending')
say a.0 '<'a.1'>' b.0 '<'b.1'>' '<'b.2'>'
say c.0 '<'c.1'>' '<'c.2'>' d.0 '<'d.1'>' '<'d.2'>'
::class writer inherit CommandHandler
::method command
  use arg command, caller
  o = caller~output
  select
    when command = 'a' then o~charout('left over')
    when command = 'b' then do
      o~charout('one '); x = o~lineout('two'); o~charout('three')
    end
    when command = 'c' then do
      o~charout('flushed'); o~flush; o~charout('then more')
    end
    when command = 'd' then do
      o~charout('closed'); o~close; o~charout('after close')
    end
  end
  return 0
```

```output
1 <left over> 2 <one two> <three>
2 <flushed> <then more> 2 <closed> <after close>
```

r[class.commandoutput.flush]
`flush` writes the text that `charOut` has kept as a line, as [the pending
text rule](r:class.commandoutput.pending) says, and does nothing when none is
kept. It answers nothing, so that using its answer as a value raises
`SYNTAX 91.999`. Arguments are ignored.

```rexx
x = .commandhandler~register('spec_co_flush', .writer~new)
address spec_co_flush 'go' with output stem o.
say o.0 '<'o.1'>' '<'o.2'>'
x = .commandhandler~deregister('spec_co_flush')
::class writer inherit CommandHandler
::method command
  use arg command, caller
  o = caller~output
  o~flush
  o~charout('kept')
  o~flush
  o~flush
  o~charout('kept again')
  o~flush(1)
  signal on syntax name bad
  x = o~flush
bad: say condition('o')~code
  return 0
```

```output
91.999
2 <kept> <kept again>
```

r[class.commandoutput.close]
`close` writes the text that `charOut` has kept as a line, as [the pending
text rule](r:class.commandoutput.pending) says, and does nothing when none is
kept. It answers nothing, and arguments are ignored. It does not end the
stream: the lines that the handler writes afterwards are written like the
earlier ones.

```rexx
x = .commandhandler~register('spec_co_close', .writer~new)
address spec_co_close 'go' with output stem o.
say o.0 '<'o.1'>' '<'o.2'>'
x = .commandhandler~deregister('spec_co_close')
::class writer inherit CommandHandler
::method command
  use arg command, caller
  o = caller~output
  o~close
  o~charout('kept')
  o~close
  o~lineout('written after close')
  o~close(1)
  signal on syntax name bad
  x = o~close
bad: say condition('o')~code
  return 0
```

```output
91.999
2 <kept> <written after close>
```

## Copying and the end of the command

r[class.commandoutput.copy]
`copy` answers a new `CommandOutput` that writes to the same target as the
original. The lines of the two go to the target in the order in which they are
written. Each of them keeps its own text that `charOut` wrote and that is not
yet a line: the `lineOut` of the copy does not finish what the original began,
and each is written, as [the pending text rule](r:class.commandoutput.pending)
says, when its own line ends or when the command is over. The command that the
original belongs to is the command of the copy too.

```rexx
x = .commandhandler~register('spec_co_copy', .writer~new)
address spec_co_copy 'go' with output stem o.
say o.0 '<'o.1'>' '<'o.2'>'
say '<'o.3'>' '<'o.4'>'
x = .commandhandler~deregister('spec_co_copy')
::class writer inherit CommandHandler
::method command
  use arg command, caller
  o = caller~output
  c = o~copy
  say c~class~id (c == o)
  o~lineout('from the original')
  c~lineout('from the copy')
  o~charout('begun by the original')
  c~charout('begun by the copy')
  c~lineout(', ended by the copy')
  return 0
```

```output
CommandOutput 0
4 <from the original> <from the copy>
<begun by the copy, ended by the copy> <begun by the original>
```

r[class.commandoutput.valid]
A `CommandOutput` belongs to the command it was made for, as its
[context](r:class.commandcontext.valid) does. When the command is over, or on
another activity, `lineOut` with a string raises `SYNTAX 98.900` and writes
nothing. The other methods depend on the text that `charOut` has kept ([the
pending text rule](r:class.commandoutput.pending)). `charOut` of a text without
a line end answers `0` and keeps the text, and `flush`, `close` and `lineOut`
without a string do nothing when no text is kept. When text is kept they raise
`SYNTAX 98.900`, and the text stays kept. `charOut` of a text with a line end
raises `SYNTAX 98.900` too and writes nothing; it keeps the text after its last
line end in place of what was kept before. Text kept on another activity while
the command runs stays kept, and the next line that the handler writes begins
with it. The second example shows these calls after the command, and the text
from another activity.

```rexx
x = .commandhandler~register('spec_co_valid', .writer~new)
address spec_co_valid 'go' with output stem o.
say 'written:' o.0 '<'o.1'>'
out = .environment~output
x = .commandhandler~deregister('spec_co_valid')
signal on syntax name bad
x = out~lineOut('late')
bad: say 'after the end:' condition('o')~code
::class writer inherit CommandHandler
::method command
  use arg command, caller
  .environment~output = caller~output
  m = .message~new(.other~new, 'ask', 'I', caller~output)
  m~start
  say 'other activity:' m~result
  caller~output~lineout('from the handler')
  return 0
::class other
::method ask unguarded
  use arg out
  signal on syntax name bad
  x = out~lineout('from another activity')
  return 'no error'
bad: return condition('o')~code
```

```output
other activity: 98.900
written: 1 <from the handler>
after the end: 98.900
```

```rexx
x = .commandhandler~register('spec_co_late', .writer~new)
address spec_co_late 'go' with output stem o.
say 'written:' o.0 '<'o.1'>'
out = .environment~output
x = .commandhandler~deregister('spec_co_late')
say 'nothing kept:' check('flush') check('close') check('lineOut')
say 'kept:' check('charOut', 'abc') check('flush') check('lineOut'),
  check('lineOut', 'text')
say 'line end:' check('charOut', 'x' || '0a'x || 'y') check('lineOut'),
  check('charOut', 'z' || '0a'x) check('lineOut')
say 'stem' o.0
exit
check: procedure expose out
  signal on syntax name bad
  if arg() = 2 then out~send(arg(1), arg(2))
  else out~send(arg(1))
  return 'ok'
bad: return condition('o')~code
::class writer inherit CommandHandler
::method command
  use arg command, caller
  .environment~output = caller~output
  m = .message~new(.other~new, 'ask', 'I', caller~output)
  m~start
  x = m~result
  caller~output~lineout('own')
  return 0
::class other
::method ask unguarded
  use arg out
  return out~charout('t')
```

```output
written: 1 <town>
nothing kept: ok ok ok
kept: ok 98.900 98.900 98.900
line end: 98.900 98.900 98.900 ok
stem 1
```

## Deviations

r[class.commandoutput.deviation.109]
A `string` of `lineOut` or `charOut` that is an instance of a subclass of
`String` is written by its own characters, and `string`, `makeString` and
`request` are not sent to it ([Deviation 109](r:class.string.deviation.109)).
ooRexx sends it `string`, so an override changes what is written. In the example
the overrides would change what is written; they count their sends, and the
string is written as it is without one.

```rexx
x = .commandhandler~register('spec_co_dev109', .writer~new)
.environment~sent = 0
address spec_co_dev109 'go' with output stem o.
say o.0 '<'o.1'>' '<'o.2'>' .environment~sent
x = .commandhandler~deregister('spec_co_dev109')
::class writer inherit CommandHandler
::method command
  use arg command, caller
  o = caller~output
  s = .digits~new('own')
  x = o~lineOut(s)
  x = o~charOut(s)
  x = o~lineOut
  return 0
::class digits subclass string
::method string
  .environment~sent = .environment~sent + 1
  return 'other'
::method makestring
  .environment~sent = .environment~sent + 1
  return 'other'
::method request
  .environment~sent = .environment~sent + 1
  return 'other'
```

```output defect=#140
2 <own> <own> 0
```
