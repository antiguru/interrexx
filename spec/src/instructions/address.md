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
name is one symbol or one string. A symbol is never looked up: whatever the
variable of that name holds, its own characters in upper case are the name. A
string is the name exactly as written. The instruction sends no command.

```rexx
system = 'other'
address system
say address()
address 'Mixed Case'
say address()
address Foo.Bar
say address()
```

```output
SYSTEM
Mixed Case
FOO.BAR
```

r[instr.address.form.value]
`ADDRESS VALUE expression` evaluates the expression and makes its value the
name of the current environment. The expression is the rest of the clause, and
its value may be the null string, which is then the name. As for a
function result, a value that is not a string is replaced by a string. The
word VALUE may be left out when the expression starts with a parenthesis, so
that `ADDRESS (expression)` is the same instruction; the expression is then
the rest of the clause and not only the part in parentheses.

```rexx
x = 'dyn'
address value x 'env'
say '['address()']'
address (x)
say '['address()']'
address (x 'foo')
say '['address()']'
address value ''
say '['address()']'
address value 12 + 3
say '['address()']'
```

```output
[dyn env]
[dyn]
[dyn foo]
[]
[15]
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
symbol, which ends in a period; `STREAM` followed by an expression in
parentheses or a string that names a stream; or `USING` followed by an
expression in parentheses. OUTPUT and ERROR may put `APPEND` or `REPLACE`
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
output and error, a replace first drops the whole stem and then sets the tails
`1` and up and the tail `0`, which holds the number of lines. An append keeps
the stem and numbers the lines after the number that its `0` tail holds. A
line is the text up to a line end, without it, and an empty line is kept.

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
```

```output
3 [a] [] c d
1 m LIT
1 x 1 y
2 l1 l2
6 n
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
`USING` takes one of the ordered collections: an Array, a List, a Queue or a
subclass of one of them. Output and error lines
are added to a collection by the message `APPEND`, after emptying it when the
redirection is a replace; input is read from the items of a collection in
order, one line each. Any other object, a MutableBuffer among them, raises
`SYNTAX 98.996` when the command is about to be sent, whether or not the
command would have written anything.

```rexx
a = .array~new
address system 'echo a1; echo a2' with output using (a)
address system 'echo b1' with output append using (a)
say a~items a[1] a[2] a[3]
address system 'echo c1' with output using (a)
say a~items a[1]
l = .list~of('in1', 'in2')
address system 'cat' with input using (l)
signal on syntax name e
address system 'exit 4' with output using (.mutablebuffer~new)
e: say condition('o')~code rc
```

```output
3 a1 a2 b1
1 c1
in1
in2
98.996 98
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
commands that come later. It sets all three streams at once: a stream the
clause does not name goes back to `NORMAL`. The setting stays until another
such clause. It does not depend on the environment, so a change of the current
environment, by name or by swap, leaves it, and it is not part of what the
swap exchanges.

```rexx
address system with output stem s.
address system with error stem e2. input normal
'echo q'
say 'A' symbol('s.1')
address system with output stem s.
address other
address system
'echo r'
say 'B' s.1
address
address
'echo t'
say 'C' s.1
```

```output
q
A LIT
B r
C t
```

r[instr.address.with.error]
Syntax that does not fit the forms is found when the program is read, so a
program file with one writes nothing and ends with the return code of the
error. The subcodes show in INTERPRET text. `SYNTAX 35.914` is VALUE with no
expression. `SYNTAX 20.933` is WITH with nothing after it, and `SYNTAX 20.932`
is STEM with no stem symbol. `SYNTAX 25.933` is a word that is not a type
where the type is due, among them APPEND or REPLACE before NORMAL or before an
input type, and the end of the clause; `SYNTAX 25.931` is a stream named twice
and `SYNTAX 25.934` is a word where `INPUT`, `OUTPUT` or `ERROR` is due.
`SYNTAX 35.935` is STREAM or USING with no expression.

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
address system with input append stem i. 25.933
address system with error using 35.935
address system with output stem o. ok
```

## Scope

r[instr.address.scope]
The redirections of [the setting rule](r:instr.address.with.set) have the
scope that the current environment has, and
[the scope rules](r:bif.address.scope.routine) of the function say it. An
internal routine starts with the caller's redirections, a stem it names is the
one in its own variables, and the caller has its own redirections back when
the routine returns. A method, and a `::ROUTINE`, starts with none, so its
commands read and write the program's own streams. Code run by INTERPRET
shares them.

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
exit
plain:
  'echo plain'
  address system with output normal
  'echo plainnormal'
  return
inherit: procedure expose o.
  'echo inherit'
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
