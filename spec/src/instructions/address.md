---
element: ADDRESS
kind: instruction
syntax: "ADDRESS [name | VALUE expression | (expression)] [command] [WITH redirection]"
ansi: "6.3.2.21, 8.3.1"
rexxref: "Keyword Instructions > ADDRESS"
deviations: [52]
---

# ADDRESS

ADDRESS chooses the environment that receives the commands of a program, and
it can also send a single command to another environment, and redirect where a
command reads its input and writes its output. This page specifies the forms
of the instruction and the redirections. The environment that is current, the
name that [the ADDRESS function](r:bif.address.name) answers, the swap of the
current and the previous environment and the scope of both across calls are
specified with that function, and the page links them. What a command is, how
it is evaluated and sent and what its return code is belong to [the commands
page](r:instr.command.send). The environments that exist, other than the one
that runs commands in the host's shell, belong to the environment chapter.

The examples use the name `system` for that shell and, in a few, names of
environments that do not exist, because the result of the instruction shows in
the ADDRESS function.

## The forms

r[instr.address.form.name]
`ADDRESS name` makes `name` the current environment, and the environment that
was current the previous one, as [the swap rule](r:bif.address.swap) says. The
name is one symbol or one string, and [the function's name
rule](r:bif.address.name) says which characters it has. A symbol is not
looked up as a variable. The instruction sends no command.

```rexx
system = 'other'
address system
say address()
address Foo.Bar
say address()
```

```output
SYSTEM
FOO.BAR
```

r[instr.address.form.value]
`ADDRESS VALUE expression` takes the value of the expression as the name, as
[the function's name rule](r:bif.address.name) says. The expression is what
follows VALUE, up to a `WITH` that is not inside parentheses, so a variable
named `with` cannot end it and `ADDRESS VALUE WITH` has no expression. The
word VALUE may be left out when the expression starts with a parenthesis, so
that `ADDRESS (expression)` is the same instruction; the expression is then
what follows, up to that `WITH`, and not only the part in parentheses.

```rexx
x = 'dyn'
address value x 'env'
say '['address()']'
address (x 'foo')
say '['address()']'
address (x) 'bar'
say '['address()']'
call try "address value 'a' with"
call try "address value ('a' with)"
call try "address value 'a' || with"
call try "address value with"
exit
try: procedure
  signal on syntax name e
  interpret arg(1)
  say arg(1) '->' '['address()']'
  return
e: say arg(1) '->' condition('o')~code
  return
```

```output
[dyn env]
[dyn foo]
[dyn bar]
address value 'a' with -> 20.933
address value ('a' with) -> [a WITH]
address value 'a' || with -> 35.1
address value with -> 35.914
```

r[instr.address.form.bare]
`ADDRESS` with nothing after it swaps the current and the previous
environment, as [the swap rule](r:bif.address.swap) says. It sends no command.

```rexx
address first
address second
address
say address()
address value 'third' with output normal
say address()
address
say address()
```

```output
FIRST
third
FIRST
```

r[instr.address.command]
`ADDRESS name expression` has a symbol or a string for the name, and whatever
follows it, up to a `WITH` that starts a redirection, is an expression. It is
evaluated as the expression of [a command](r:instr.command.send) is, and its
value is sent to the named environment as one command. The current environment
and the previous one stay as they were, so
[the function](r:bif.address.command) answers the same name after the command.
`RC` is set as [the commands page](r:instr.command.rc) says. An error in the
expression raises that error and sends nothing.

```rexx
address first
address system 'exit 2'
say rc address()
address "system" 'exit' 1 + 2
say rc address()
signal on syntax name e
address system 'exit' + 1
e: say condition('o')~code address()
```

```output
2 FIRST
3 FIRST
41.1 FIRST
```

## Redirection

r[instr.address.with.syntax]
A clause of the forms above may end with `WITH` and one to three redirections,
in any order, each introduced by `INPUT`, `OUTPUT` or `ERROR` and each used at
most once. The redirection is then one of `NORMAL`; `STEM` followed by a stem
symbol, which ends in a period; `STREAM` followed by a string, an environment
symbol or an expression in parentheses, that names a stream; or `USING`
followed by an expression in parentheses, a string, a constant symbol or an
environment symbol. A plain variable symbol after STREAM or USING is not one
of these. OUTPUT and ERROR may put `APPEND` or `REPLACE`
before STEM, STREAM or USING, and replace is meant when neither is written.
The keywords may be in any case. The word `WITH` directly after `ADDRESS` is
the name of an environment and not the keyword, so
`ADDRESS WITH OUTPUT NORMAL` sends the command `OUTPUT NORMAL` to the
environment `WITH`.

```rexx
address first
i.0 = 1
i.1 = 'z'
address system 'cat' with Error stem e. OUTPUT append stem o. input stem i.
say o.0 o.1
address with output normal
say rc address()
```

```output
1 z
30 FIRST
```

r[instr.address.with.normal]
`NORMAL` is no redirection: the command reads and writes the program's own
standard input, output and error, as a command does with no redirection at
all.

```rexx
address system with output stem o.
address system 'echo kept' with output normal
say symbol('o.1')
```

```output
kept
LIT
```

r[instr.address.with.stem]
`STEM name.` reads or writes the lines of a stem. For input, the stem's `0`
tail holds the number of lines and the tails `1` to that number hold them. For
output and error, a replace clears every tail of the stem, keeps the default
value of the stem, and then sets the tails `1` and up and the tail `0`, which
holds the number of lines. An append keeps the stem and numbers the lines
after the number that its `0` tail holds. A line is the text up to a line end,
without it; a carriage return directly before a line feed is not part of the
line, text after the last line end is a line, and an empty line is kept. An
input line that holds a line feed is sent as it is, and the command reads it
as two lines.

```rexx
address system 'echo a; echo; echo c d' with output stem b.
say b.0 '['b.1']' '['b.2']' b.3
o.0 = 3
o.3 = 'old'
address system 'echo m' with output stem o.
say o.0 o.1 symbol('o.3')
address system 'echo x; echo y >&2' with output stem a. error stem e.
say a.0 a.1 e.0 e.1
i.0 = 2
i.1 = 'l1'
i.2 = 'l2'
address system 'cat' with input stem i. output stem o.
say o.0 o.1 o.2
o.0 = 5
address system 'echo n' with output append stem o.
say o.0 o.6
address system 'printf "p\r\nq"' with output stem c.
say c.0 c2x(c.1) c2x(c.2)
j.0 = 1
j.1 = 'u' || '0a'x || 'v'
address system 'cat' with input stem j. output stem c.
say c.0 c.1 c.2
```

```output
3 [a] [] c d
1 m LIT
1 x 1 y
2 l1 l2
6 n
2 70 71
2 u v
```

r[instr.address.with.stem.count]
The `0` tail of an input stem is the number of lines, a whole number of zero
or more, in any form that [the whole-number form](r:conv.whole.form) allows
and whose value is whole: `1.0`, `1e0`, `+1` and `' 1 '` are 1. A `0` tail
that is not set raises `SYNTAX 98.998`, and one that is not a whole number of
zero or more raises `SYNTAX 26.904`. A count of 0 sends nothing, and a tail
from 1 to the count that is not set sends the stem's own default value, which
for a stem that was never assigned is the stem's name with its period. For an
append, a `0` tail that is not set, or that holds only the default value of
the stem, counts as 0; any other `0` tail is checked as for input, and
`SYNTAX 26.904` is raised for a bad one.

```rexx
o. = 'def'
o.5 = 'five'
address system 'echo a' with output stem o.
say o.0 o.1 o.5 symbol('o.5') o.9
i.0 = 2
i.1 = 'x'
address system 'cat' with input stem i.
call try 'unset', "address system 'cat' with input stem j."
call try 'abc', "j.0 = 'abc'; address system 'cat' with input stem j."
call try 'minus', "j.0 = -1; address system 'cat' with input stem j."
call try 'frac', "j.0 = 2.5; address system 'cat' with input stem j."
call try 'zero', "j.0 = 0; address system 'cat' with input stem j."
t = "address system 'echo z' with output append stem m."
call try 'bad append', "m.0 = 'xyz';" t
call try 'blanks', "m.0 = ' 2 ';" t "; say m.3"
call try 'unset append', t "; say m.0 m.1"
exit
try: procedure
  signal on syntax name e
  interpret arg(2)
  say arg(1) 'ok'
  return
e: say arg(1) condition('o')~code
  return
```

```output
1 a def VAR def
x
I.
unset 98.998
abc 26.904
minus 26.904
frac 26.904
zero ok
bad append 26.904
z
blanks ok
1 z
unset append ok
```

```rexx
i.0 = 1.0
i.1 = 'a'
address system 'cat' with input stem i.
m.0 = 1.0
address system 'echo z' with output append stem m.
say m.0 m.2
```

```output defect=#104
a
2 z
```

r[instr.address.with.stream]
`STREAM` names a stream by the name that the stream functions take. An output
or error stream is written with the lines of the command, and a replace
empties it first. A command with an input stream reads the lines of the
stream.

```rexx
f = 'spec-address-with.tmp'
call sysfiledelete f
address system 'echo one' with output stream (f)
address system 'echo two' with output append stream (f)
say linein(f) linein(f) lines(f)
call stream f, 'c', 'close'
address system 'echo three' with output replace stream (f)
say linein(f) lines(f)
call stream f, 'c', 'close'
address system 'cat' with input stream (f)
call stream f, 'c', 'close'
call sysfiledelete f
```

```output
one two 0
three 0
three
```

r[instr.address.with.using]
For output and error, `USING` names the object that receives the lines. An
ordered collection, an Array, a List, a Queue or a subclass of one, gets each
line by the message `APPEND`, after it is emptied when the redirection is a
replace. A Stem object is written as `STEM` writes a stem: a replace clears it
and sets the tails `1` and up and `0`, and an append numbers the lines after
its `0` tail. A stream object gets each line by `LINEOUT`; it takes neither
`REPLACE` nor `APPEND`, and either raises `SYNTAX 98.997`. Any other object, a
string, `.nil`, a Set, a Directory, a Bag or a MutableBuffer among them, and
an object of a class of the program that has an `APPEND` method of its own,
raises `SYNTAX 98.996` when the command is about to be sent, whether or not
the command would have written anything.

```rexx
a = .array~new
address system 'echo a1; echo a2' with output using (a)
address system 'echo b1' with output append using (a)
say a~items a[1] a[2] a[3]
address system 'echo c1' with output using (a)
say a~items a[1]
l = .list~new
address system 'echo l1' with output using (l)
q = .queue~new
address system 'echo q1' with output using (q)
st = .stem~new
address system 'echo p; echo q' with output using (st)
address system 'echo s' with output append using (st)
say l~items q~items st[0] st[1] st[3]
```

```output
3 a1 a2 b1
1 c1
1 1 3 p s
```

```rexx
f = 'spec-address-using.tmp'
call sysfiledelete f
call try 'stream', ".stream~new(f)", ''
call try 'append', ".stream~new(f)", 'append'
call try 'replace', ".stream~new(f)", 'replace'
call try 'string', "'text'", ''
call try 'nil', ".nil", ''
call try 'set', ".set~new", ''
call try 'buffer', ".mutablebuffer~new", ''
call try 'own', ".own~new", ''
call sysfiledelete f
exit
try: procedure expose f
  signal on syntax name e
  interpret "address system 'exit 0' with output" arg(3) "using (" arg(2) ")"
  say arg(1) 'ok'
  return
e: say arg(1) condition('o')~code
  return
::class own
::method append
  return 1
```

```output
stream ok
append 98.997
replace 98.997
string 98.996
nil 98.996
set 98.996
buffer 98.996
own 98.996
```

r[instr.address.with.using.input]
For input, `USING` names the source of the lines. A string is one line, and so
is a MutableBuffer, by its string value. An object that has a `MAKEARRAY`
method, among them a List, a Queue, a Set, a Directory, a Bag, a Stem object
and an object of a class of the program with such a method, gives one line for
each item of the array that the method answers. Any other object, `.nil`
among them, raises `SYNTAX 98.924` when the command is about to be sent.

```rexx
address system 'cat' with input using ('one') output stem o.
say o.0 o.1
l = .list~of('x', 'y')
address system 'cat' with input using (l) output stem o.
say o.0 o.1 o.2
address system 'cat' with input using (.set~of('s')) output stem o.
say o.0 o.1
mb = .mutablebuffer~new('mb')
address system 'cat' with input using (mb) output stem o.
say o.0 o.1
address system 'cat' with input using (.mak~new) output stem o.
say o.0 o.1 o.2
signal on syntax name e
address system 'cat' with input using (.nil)
e: say condition('o')~code
::class mak
::method makearray
  return .array~of('m1', 'm2')
```

```output
1 one
2 x y
1 s
1 mb
2 m1 m2
98.924
```

r[instr.address.with.once]
A redirection written on a clause with a command applies to that command and
to no other. The streams the clause does not name keep what they had, which
is [the setting](r:instr.address.with.set) of the earlier clauses.

```rexx
address system with output stem s.
address system 'echo x; echo y >&2' with error stem e.
say s.0 s.1 e.0 e.1
address system 'echo p'
say s.1 symbol('e.2')
```

```output
1 x 1 y
p LIT
```

r[instr.address.with.set]
A clause with a redirection and no command sets the redirection of the
environment that it names, and makes that environment current as
`ADDRESS name` does. The commands sent to that environment from then on use
the setting, until another such clause names the same environment. It sets
all three streams at once: a stream the clause does not name goes back to
`NORMAL`. A name without `WITH` makes an environment current with the setting
it last had, or with none when it never had one, and
[the swap](r:bif.address.swap) of the current and the previous environment
brings an environment back with its setting. The setting of one environment
does not touch the commands that another receives. The stem, the stream
expression or the `USING` expression is kept as it was written and is
evaluated again when each command is sent, in the variables of the code that
sends it.

```rexx
address sh with output stem a.
address system with output stem b.
address path
address sh
'echo 1'
say 'A' a.0 a.1
address system
'echo 2'
say 'B' b.0 b.1
address path with output stem p.
address sh
'echo 3'
say 'C' a.0 a.1 p.0
address
'echo 4'
say 'D' p.0 p.1
address other with output stem t.
address system
'echo z'
say 'E' t.0 b.1
```

```output
A 1 1
B 1 2
C 1 3 P.0
D 1 4
E T.0 z
```

```rexx
a = .array~new
address system with output using (a)
b = a
a = .array~new
'echo x'
say b~items a~items
```

```output
0 1
```

r[instr.address.with.error]
Syntax that does not fit the forms is found when the program is read, so a
program file with one writes nothing and ends with the return code of the
error. The subcodes show in INTERPRET text. `SYNTAX 35.914` is VALUE with no
expression. `SYNTAX 20.933` is WITH with nothing after it, and `SYNTAX 20.932`
is STEM with no stem symbol. `SYNTAX 25.933` is a word that is not a type
where the type is due, among them APPEND or REPLACE before NORMAL or before an
input type, and the end of the clause; `SYNTAX 25.930`, `SYNTAX 25.931` and
`SYNTAX 25.932` are INPUT, OUTPUT and ERROR named twice, and `SYNTAX 25.934`
is a word where `INPUT`, `OUTPUT` or `ERROR` is due. `SYNTAX 35.935` is STREAM
or USING with no expression, and `SYNTAX 35.1` is a plain variable symbol
after either.

```rexx
say 'not run'
address system with output foo
```

```output rc=231
```

```rexx
say 'not run'
address value
```

```output rc=221
```

```rexx
call try 'address value'
call try 'address system with'
call try 'address system with output foo'
call try 'address system with output replace normal'
call try 'address system with output stem o. error'
call try 'address system with foo normal'
call try 'address system with output stem'
call try 'address system with output stem o. output normal'
call try 'address system with input normal input normal'
call try 'address system with error normal error normal'
call try 'address system with output stream foo'
call try 'address system with input append stem i.'
call try 'address system with error using'
call try 'address system with output stem o.'
exit
try: procedure
  signal on syntax name e
  interpret arg(1)
  say arg(1) 'ok'
  return
e: say arg(1) condition('o')~code
  return
```

```output
address value 35.914
address system with 20.933
address system with output foo 25.933
address system with output replace normal 25.933
address system with output stem o. error 25.933
address system with foo normal 25.934
address system with output stem 20.932
address system with output stem o. output normal 25.931
address system with input normal input normal 25.930
address system with error normal error normal 25.932
address system with output stream foo 35.1
address system with input append stem i. 25.933
address system with error using 35.935
address system with output stem o. ok
```

## Scope

r[instr.address.scope]
The redirections of [the setting rule](r:instr.address.with.set) have the
scope that the current environment has, and [the scope
rules](r:bif.address.scope.routine) of the function say it. An internal
routine starts with the caller's redirections, a stem it names is the one in
its own variables, and the caller has its own redirections back when the
routine returns. A routine with PROCEDURE and no EXPOSE therefore fills a stem
of its own and leaves the caller's stem as it was. A method, and a
`::ROUTINE`, starts with none, so its commands read and write the program's
own streams. Code run by INTERPRET shares them.

```rexx
address system with output stem o.
call plain
say 'A' o.0 o.1
call inherit
say 'B' o.0 o.1
say .o~new~m
say 'C' o.0 o.1
interpret "'echo int'"
say 'D' o.0 o.1
call plain2
'echo back'
say 'E' o.0 o.1
call hid
say 'F' o.0 o.1
exit
plain:
  'echo plain'
  address system with output normal
  'echo plainnormal'
  return
inherit: procedure expose o.
  'echo inherit'
  return
hid: procedure
  'echo h'
  say 'hid' o.0 o.1
  return
plain2:
  address system with output stem p.
  'echo plain2'
  return
::class o
::method m
  'echo meth'
  return 'm'
```

```output
plainnormal
A 1 plain
B 1 inherit
meth
m
C 1 inherit
D 1 int
E 1 back
hid 1 h
F 1 back
```

## Deviations

r[instr.address.deviation.52]
When a command writes through `USING` to a collection whose `APPEND` raises a
condition, the condition is raised whatever return code the command answered.
ooRexx raises it only when the command answered 0, and ends the clause with
the return code otherwise. The example shows the interrexx output; ooRexx
shows `ok 1`.

```rexx
signal on syntax name e
address system 'echo x; exit 1' with output using (.bad~new)
say 'ok' rc
exit
e: say 'caught' condition('o')~code rc
::class bad subclass array
::method append
  raise syntax 93.900
```

```output
caught 93.900 93
```
