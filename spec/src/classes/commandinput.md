---
element: "CommandInput"
kind: class
rexxref: "Builtin Classes > Utility Classes > CommandInput Class"
---

# CommandInput

A `CommandInput` is the stream from which a handler written in Rexx reads the
input of a command that was sent with `ADDRESS ... WITH INPUT`. The handler
gets it from [`input`](r:class.commandcontext.input) of the
[`CommandContext`](commandcontext.md) of the command. It reads the source of
the redirection, a stem, a stream or the object of a `USING` ([the redirection
rules](r:instr.address.with.syntax)), as a list of lines, one after the other
and once. It is a stream of the input kind: its class inherits the mixin class
`InputStream`, whose other methods, `arrayIn` among them, are specified with
the [stream classes](inputstream.md). This page states the four methods that
the class defines for itself, the text that they read, and what they do at the
end of the input. The input of a command is read by the handler, or by nobody;
which commands of the host read their standard input from it is the business
of the environment chapter.

## The class

r[class.commandinput.creation]
`CommandInput` has the instance methods `charIn`, `chars`, `lineIn` and `lines`
besides those that it inherits. Its superclass is `Object`, and it inherits the
mixin class `InputStream`: its `superClasses` are `Object` and `InputStream` in
that order ([the hierarchy rule](r:class.index.hierarchy)). The interpreter
makes the object for a command and hands it out; a program does not make one.
The `new` of a program raises `SYNTAX 93.901` with no argument and
`SYNTAX 93.902` with more than one, and what an object made with one is good for
is not specified, since it belongs to no command. The class has other methods,
`init` among them, that the interpreter uses; they are not part of this
specification.

```rexx
s = ''
do c over .commandinput~superclasses
  s = s c~id
end
say .commandinput~id .commandinput~superclasses~items s~strip
have = .set~new
m = .commandinput~methods(.commandinput)
do while m~available
  have~put(m~index)
  m~next
end
names = 'charIn chars lineIn lines'~makeArray(' ')
count = 0
do name over names
  if have~hasItem(name~upper) then count = count + 1
end
say count 'of' names~items
call try '.commandinput~new'
call try '.commandinput~new(1, 2)'
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
CommandInput 2 Object InputStream
4 of 4
.commandinput~new 93.901
.commandinput~new(1, 2) 93.902
```

## The text that it reads

r[class.commandinput.text]
The input is a list of lines. A source that is a stem gives the lines `stem.1`
to `stem.n` that `stem.0` counts, a stream gives its lines, and an object of a
`USING` gives the lines that it supplies, an array's items for one. Each line is
followed by a line end of one character, `'0A'x`: the characters that the stream
reads are the characters of the lines, each followed by that line end, the last
line too. A line that holds a line end is one line, and it is not split where
the line end is. Reading goes from the first line to the last and never back:
the first argument of `charIn` and `lineIn`, which would give a position, is
refused with `SYNTAX 93.963`. The example shows one source of each kind.

```rexx
x = .commandhandler~register('spec_ci_text', .reader~new)
i. = ''; i.0 = 2; i.1 = 'stem one'; i.2 = 'stem two'
f = 'spec_ci_text.tmp'
call lineout f, 'file one'
call lineout f, 'file two'
call lineout f
address spec_ci_text 'stem' with input stem i.
address spec_ci_text 'stream' with input stream (f)
address spec_ci_text 'array',
  with input using (.array~of('array one', '', 'array three'))
address spec_ci_text 'list' with input using (.list~of('list one'))
address spec_ci_text 'string' with input using ('one' || '0a'x || 'line')
x = .file~new(f)~delete
x = .commandhandler~deregister('spec_ci_text')
::class reader inherit CommandHandler
::method command
  use arg command, caller
  in = caller~input
  text = ''
  do while in~lines > 0
    text = text '<'in~linein~changestr('0a'x, '\n')'>'
  end
  say command~left(7) text
  return 0
```

```output
stem     <stem one> <stem two>
stream   <file one> <file two>
array    <array one> <> <array three>
list     <list one>
string   <one\nline>
```

r[class.commandinput.linein]
`lineIn` answers the next line, without its line end, and moves the reading past
it, so that the next call answers the line after. After some of a line has been
read with `charIn`, it answers the rest of that line, which is the null string
when only the line end is left. When all the input has been read it answers the
null string and raises NOTREADY ([the NOTREADY rule](r:cond.notready.call)).
`lineIn` takes the arguments `line` and `count`: `line` is the position and must
be omitted (`SYNTAX 93.963`), and `count` is `1`, the default, or `0`, which
reads nothing and answers the null string, and it is compared by its characters:
any other value, `2`, `-1`, `1.0`, `01` and `x` among them, raises
`SYNTAX 93.963`. A third argument raises `SYNTAX 93.902`. A line that is the
null string and the end of the input look alike in the answer, and `lines` tells
them apart.

```rexx
x = .commandhandler~register('spec_ci_linein', .reader~new)
address spec_ci_linein 'go',
  with input using (.array~of('one', 'two', 'three'))
x = .commandhandler~deregister('spec_ci_linein')
::class reader inherit CommandHandler
::method command
  use arg command, caller
  in = caller~input
  say '<'in~linein'>' in~lines
  say '<'in~charin(,2)'>' '<'in~linein'>' in~lines
  say '<'in~linein(,0)'>' in~lines
  say '<'in~linein'>' in~lines
  call on notready name nr
  say 'at the end <'in~linein'>' in~lines
  call off notready
  do e over .array~of("in~linein(1)", "in~linein(, 2)", "in~linein(, 1.0)",,
      "in~linein(, -1)", "in~linein(1, 1)", "in~linein(, 1, 1)",,
      "in~linein(.nil)")
    self~try(in, e)
  end
  return 0
nr: say '   NOTREADY' condition('c')
  return
::method try
  use arg in, e
  signal on syntax name bad
  interpret 'x =' e
  say e 'no error'
  return
bad: say e condition('o')~code
  return
```

```output
<one> 1
<tw> <o> 1
<> 1
<three> 0
at the end <> 0
   NOTREADY NOTREADY
in~linein(1) 93.963
in~linein(, 2) 93.963
in~linein(, 1.0) 93.963
in~linein(, -1) 93.963
in~linein(1, 1) 93.963
in~linein(, 1, 1) 93.902
in~linein(.nil) 93.963
```

r[class.commandinput.charin]
`charIn(start, count)` answers the next `count` characters of the text that
[the text rule](r:class.commandinput.text) describes, the line ends among
them, and moves the reading past them. `count` is a whole number of at least 0
and defaults to `1`; with `0` it answers the null string and reads nothing;
when fewer than `count` characters are left, it answers those that are. A line
that was partly read goes on from where it stopped. At the end of the input it
answers the null string and raises NOTREADY. `start`, the position, must be
omitted (`SYNTAX 93.963`). A `count` that is no number raises `SYNTAX 93.943`
and one that is a negative number or not whole raises `SYNTAX 93.923`; a third
argument raises `SYNTAX 93.902`. In the example the input is two lines, `ab`
and `cd`.

```rexx
x = .commandhandler~register('spec_ci_charin', .reader~new)
address spec_ci_charin 'go' with input using (.array~of('ab', 'cd'))
x = .commandhandler~deregister('spec_ci_charin')
::class reader inherit CommandHandler
::method command
  use arg command, caller
  in = caller~input
  do e over .array~of("in~charin(1)", "in~charin(1, 1)", "in~charin(, 'x')",,
      "in~charin(, .nil)", "in~charin(, -1)", "in~charin(, 1.5)",,
      "in~charin(, 1, 1)")
    self~try(in, e)
  end
  say c2x(in~charin(,3)) '<'in~charin'>' '<'in~charin(,0)'>' in~lines
  say c2x(in~charin(,100)) in~lines
  call on notready name nr
  say 'at the end <'in~charin'>' in~lines
  return 0
nr: say '   NOTREADY' condition('c')
  return
::method try
  use arg in, e
  signal on syntax name bad
  interpret 'x =' e
  say e 'no error'
  return
bad: say e condition('o')~code
  return
```

```output
in~charin(1) 93.963
in~charin(1, 1) 93.963
in~charin(, 'x') 93.943
in~charin(, .nil) 93.943
in~charin(, -1) 93.923
in~charin(, 1.5) 93.923
in~charin(, 1, 1) 93.902
61620A <c> <> 1
640A 0
at the end <> 0
   NOTREADY NOTREADY
```

r[class.commandinput.lines]
`lines` answers `1` when some of the input has not been read, and `0` when all
has been, whatever the number of lines left: it is no count. An input with no
line answers `0` from the start, one whose only line is the null string
answers `1` until that line is read, and a line that was partly read counts as
unread. Arguments are ignored. The method does not read and does not raise
NOTREADY.

```rexx
x = .commandhandler~register('spec_ci_lines', .reader~new)
address spec_ci_lines 'none' with input using (.array~new)
address spec_ci_lines 'empty' with input using (.array~of(''))
address spec_ci_lines 'three' with input using (.array~of('a', 'b', 'c'))
x = .commandhandler~deregister('spec_ci_lines')
::class reader inherit CommandHandler
::method command
  use arg command, caller
  in = caller~input
  say command~left(6) in~lines in~lines('N') in~lines(1, 2)
  do while in~lines > 0
    x = in~charin
    say '   after' c2x(x) 'lines' in~lines
  end
  return 0
```

```output
none   0 0 0
empty  1 1 1
   after 0A lines 0
three  1 1 1
   after 61 lines 1
   after 0A lines 1
   after 62 lines 1
   after 0A lines 1
   after 63 lines 1
   after 0A lines 0
```

r[class.commandinput.chars]
`chars` answers the number of characters that are left of the line that the
next read starts in, the line end counted as one character: for a line that
was not read at all, its length and one more. It is `0` at the end of the
input, and `1` when the next line is the null string. It counts one line,
never the rest of the input. Arguments are ignored, and it reads nothing.

```rexx
x = .commandhandler~register('spec_ci_chars', .reader~new)
address spec_ci_chars 'go' with input using (.array~of('abc', '', 'de'))
x = .commandhandler~deregister('spec_ci_chars')
::class reader inherit CommandHandler
::method command
  use arg command, caller
  in = caller~input
  say in~chars in~chars('N')
  x = in~charin(,2)
  say in~chars
  x = in~linein
  say in~chars
  x = in~linein
  say in~chars
  x = in~linein
  say in~chars
  return 0
```

```output
4 4
2
1
3
0
```

## Copying and the end of the command

r[class.commandinput.copy]
`copy` answers a new `CommandInput` that reads from the same source and shares
the position of the reading with the original: a line that one has read is
gone for the other, and `lines` and `chars` of both agree. The command that
the original belongs to is the command of the copy too.

```rexx
x = .commandhandler~register('spec_ci_copy', .reader~new)
address spec_ci_copy 'go',
  with input using (.array~of('first', 'second', 'third'))
x = .commandhandler~deregister('spec_ci_copy')
::class reader inherit CommandHandler
::method command
  use arg command, caller
  in = caller~input
  c = in~copy
  say c~class~id (c == in) c~lines c~chars
  say c~linein in~linein c~linein in~lines c~lines
  return 0
```

```output
CommandInput 0 1 6
first second third 0 0
```

r[class.commandinput.valid]
A `CommandInput` belongs to the command it was made for, as its
[context](r:class.commandcontext.valid) does. When the command is over, or on
another activity, `charIn`, `chars`, `lineIn` and `lines` raise
`SYNTAX 98.900`.

```rexx
x = .commandhandler~register('spec_ci_valid', .reader~new)
address spec_ci_valid 'go' with input using (.array~of('a'))
in = .environment~input
x = .commandhandler~deregister('spec_ci_valid')
do m over 'lineIn lines chars charIn'~makeArray(' ')
  call try m
end
exit
try:
  signal on syntax name bad
  x = in~send(arg(1))
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
::class reader inherit CommandHandler
::method command
  use arg command, caller
  .environment~input = caller~input
  m = .message~new(.other~new, 'ask', 'I', caller~input)
  m~start
  say 'other activity:' m~result
  return 0
::class other
::method ask unguarded
  use arg in
  signal on syntax name bad
  x = in~lines
  return 'no error'
bad: return condition('o')~code
```

```output
other activity: 98.900
lineIn 98.900
lines 98.900
chars 98.900
charIn 98.900
```
