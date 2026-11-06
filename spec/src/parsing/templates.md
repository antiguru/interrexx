---
element: "Templates"
kind: concept
ansi: "8.3.17"
rexxref: "Parsing > Simple Templates for Parsing into Words, Parsing > The Period as a Placeholder, Parsing > Parse with Message Terms Assignments"
---

# Templates

A template with no trigger cuts the source into words. This page specifies
that case, and every other template reduces to it: [the pieces
rule](r:parse.index.pieces) hands each piece of a source to the targets that
stand next to it, and what those targets do with a piece is what this page
says. The examples use `parse var` and `parse value … with`.

## Words

r[parse.templates.words]
Each target of a piece, except the last, receives one word of it. To find
the word, the parse skips the whitespace at the start of what remains of the
piece, takes the characters up to the next whitespace character or the end,
and then removes one whitespace character after the word, if there is one.
A word is what [the builtins](r:bif.index.words.split) call one, and
[whitespace](r:conv.string.whitespace) is as the conventions define it, so a
line feed can be part of a word.

```rexx
s = '  alpha   beta'||'09'x||'gamma delta  '
parse var s u v w
say u'|'v'|'w'|'
parse value 'ab'||'0a'x||'cd ef' with u v
say c2x(u) v
```

```output
alpha|beta|gamma delta  |
61620A6364 ef
```

r[parse.templates.last]
The last target of a piece receives all that remains of the piece after the
words of the targets before it, and nothing is removed from it: its leading
whitespace beyond the one character that ended the previous word, its
interior whitespace and its trailing whitespace all stay. A piece with a
single target therefore gives that target the whole piece, with the
whitespace at both ends. Words beyond the number of targets are therefore in
the last target, unless that is a [period](r:parse.templates.period), which
discards them.

```rexx
s = '  alpha   beta  gamma delta  '
parse var s u
say '['u']'
parse var s u v
say '['u']['v']'
parse var s . v
say '['v']'
parse value 'one  ' with u v
say '['u']['v']'
```

```output
[  alpha   beta  gamma delta  ]
[alpha][  beta  gamma delta  ]
[  beta  gamma delta  ]
[one][ ]
```

r[parse.templates.fewer]
When the words of a piece run out, each target that remains receives the null
string, except that the last target of the piece receives what is left, as
[the previous rule](r:parse.templates.last) says, and that is the null string
only when nothing is left. So a null source, and a source of whitespace only,
give two targets the null string, while a single target receives the
whitespace itself. Every target is assigned: a variable that held a value
before the parse does not keep it.

```rexx
u = 'old'; v = 'old'; w = 'old'
parse value 'one' with u v w
say '['u']['v']['w']'
u = 'old'; v = 'old'
parse value '' with u v
say '['u']['v']'
u = 'old'; v = 'old'
parse value '   ' with u v
say '['u']['v']'
parse value '   ' with u
say '['u']'
```

```output
[one][][]
[][]
[][]
[   ]
```

## The period

r[parse.templates.period]
A period stands where a variable could and discards what that variable would
receive. It takes its place in the sequence of targets as a variable would,
so it consumes the next word, or the rest of the piece when it is the last,
and assigns nothing. A period at the end of a template is the
usual way to ignore the surplus of the source.

```rexx
parse value 'red green blue amber' with . second . fourth
say second fourth
parse value 'red green blue amber' with first .
say first
parse value 'red green blue amber' with . . third
say third
```

```output
green amber
red
blue amber
```

## Targets

r[parse.templates.targets]
A [target](r:parse.index.elements) that is a simple symbol or a compound
symbol is assigned like the left side of an
[assignment](r:instr.assignment.value), and the tail of a compound symbol
is evaluated when the assignment is made, as [the order
rule](r:parse.index.order) says. A stem symbol is a target too, and assigning
to it [gives the stem that value as its default](r:expr.variables.stem), so
that every compound variable of the stem has the value at once. A message
term, such as `slots[2]` or `obj~attr`, receives its value as a [message
assignment](r:instr.message.assign) does, by sending the assignment message
with the value as its argument. The answer of that message is discarded, and
unlike the instruction a parse does not set `RESULT`. The receiver and the
arguments of the message are evaluated when the target is assigned, and so
see the targets assigned before it.

```rexx
parse value 'p q r' with s. t
say s.1 s.any t
slots = .array~new
k = 1
parse value '3 red blue' with k slots[k] slots[k + 1]
say k slots[3] slots[4]
result = 'untouched'
reg = .directory~new
box = .cell~new
parse value 'n:7 s:9' with 'n:' reg['north'] ' s:' box~amount
say reg['north'] box~amount result
exit
::class cell
::method 'amount='
  expose amount
  use arg amount
  return 'answered'
::method amount
  expose amount
  return amount
```

```output
p p q r
3 red blue
7 9 untouched
```

r[parse.templates.empty]
A template that has no tokens, whether it is missing after the keywords of
the instruction or stands between two commas, assigns nothing. It raises no
error, however much the source holds.

```rexx
parse value 'abc def' with
parse var undefined_name with
say 'no assignment, no error'
```

```output
no assignment, no error
```
