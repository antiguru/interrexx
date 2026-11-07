---
element: TRACE
kind: instruction
syntax: "TRACE [setting | VALUE expression | (expression)]"
ansi: "6.3.2.85, 8.3.26"
rexxref: "Keyword Instructions > TRACE; Keyword Instructions > TRACE > The Format of Trace Output"
deviations: [0, 7, 50]
---

# TRACE

TRACE chooses how much of a running program the interpreter reports, and the
report is a stream of lines in a fixed format. This page specifies the forms
of the instruction, when a new setting takes effect, where the lines go and
what each kind of line looks like. The nine settings, what the TRACE function
answers and how a setting is scoped across calls belong to [the TRACE
function](r:bif.trace.setting), and this page links them instead of repeating
them.

The examples that show trace lines first send them to standard output with
`.traceoutput~destination(.output)`, because the lines go to the error stream
otherwise, and the examples are checked on standard output only.

## The forms

r[instr.trace.form.name]
`TRACE` followed by one symbol or one string takes it as the setting. A symbol
is never looked up: whatever the variable of that name holds, its own
characters, in either case, are the setting. A setting is chosen by its first
letter after [the prefix](r:bif.trace.prefix), as [the letter
rule](r:bif.trace.letter) says, so a whole word such as `Results` does as
well as `R`, and the string `'a'` does as well as `A`.

```rexx
r = 'I'
trace r
say trace()
trace 'a'
say trace()
trace Results
say trace()
trace "Normal"
say trace()
trace off
say trace()
```

```output
R
A
R
N
O
```

r[instr.trace.form.value]
`TRACE VALUE expression` evaluates the expression and takes its value as the
setting, with the same choice by first letter. A value that is not a string is
converted as [a string argument](r:conv.string.object) says, and `.nil` is not
a setting. VALUE may be left out when the expression starts with a
parenthesis, or with an operator character other than `+`, `-`, `=` and `~`,
so `TRACE (expression)` is the same instruction; the expression is then the
rest of the clause, as it is after VALUE. `TRACE = expression` is an
assignment to the variable `TRACE`, and `TRACE~name` a message instruction, as
[the kinds rule](r:instr.index.kinds) says; neither sets a trace setting.
A null value is the setting `N`. The
expression ends the clause, so `TRACE VALUE 'l' 'x'` takes the value `l x`,
which is the setting `L`.

```rexx
r = 'I'
trace value r
say trace()
trace value 'a' || 'c'
say trace()
trace (r)
say trace()
trace ('E')
say trace()
trace value ''
say trace()
trace value 'l' 'x'
say trace()
```

```output
I
A
I
E
N
L
```

```rexx
call try "trace (r) || 'x'"
call try "trace ('r') 'x'"
call try "trace \'r'"
call try "trace = 'r'"
call try "trace~length"
call try "trace value .array~of('r')"
call try "trace value .array~new"
call try "trace value .nil"
exit
try: procedure
  signal on syntax name e
  interpret arg(1)
  say arg(1) 'set' trace()
  return
e: say arg(1) condition('o')~code
  return
```

```output
trace (r) || 'x' set R
trace ('r') 'x' set R
trace \'r' 34.901
trace = 'r' set N
trace~length set N
trace value .array~of('r') set R
trace value .array~new set N
trace value .nil 24.1
```

r[instr.trace.form.bare]
`TRACE` with nothing after it sets `N`, and so does the null string in any
form of the instruction. Where interactive tracing is on, a setting is
ignored, as [the instruction rule](r:bif.trace.instruction) of the function
says; a number still raises the error of [the number
rule](r:instr.trace.number).

```rexx
trace r
say trace()
trace
say trace()
```

```output
R
N
```

r[instr.trace.form.interactive]
A question mark before the setting, or alone, turns interactive tracing on
or off in the way of [the prefix rule](r:bif.trace.prefix): `TRACE ?` alone
flips it and leaves the letter. The instruction cannot turn interactive
tracing off, which the function can. What an interactive session accepts as
input and what it writes is not specified; a program run with no input at all
goes on after each pause. An example that waits for input is never written
here.

```rexx
trace ?r
say trace()
call trace '?'
say trace()
trace ?
say trace()
```

```output
?R
R
?R
```

r[instr.trace.number]
A setting that is a whole number, written as a symbol, as a string or as the
value of an expression, with a sign and blanks around it as a number may have,
raises `SYNTAX 24.901`. A number only has a meaning in interactive tracing,
which this book does not specify. A value that is not a whole number is not
numeric here and is judged by [the letter rule](r:bif.trace.letter) as any
other, so `1.5` raises `SYNTAX 24.1`, as does `.5`. A sign directly after
TRACE, followed by one symbol or string, makes a signed number of it: a whole
number raises `SYNTAX 24.901`, and any other operand raises `SYNTAX 26.7`.
A parenthesis after the sign raises `SYNTAX 35.1`, and data after the number
raises `SYNTAX 21.906`. A sign alone is no number: `TRACE -` is the bare
form, and `TRACE +` and a second sign after the first raise `SYNTAX 35.1`.

```rexx
call try '5'
call try '-3'
call try '007'
call try ' 5 '
call try '1.0'
call try '1e1'
call try '1.5'
call try '.5'
exit
try: procedure
  signal on syntax name e
  trace value arg(1)
  say arg(1) 'set'
  return
e: say arg(1) condition('o')~code
  return
```

```output
5 24.901
-3 24.901
007 24.901
 5  24.901
1.0 24.901
1e1 24.901
1.5 24.1
.5 24.1
```

```rexx
call try 'trace -3'
call try 'trace + 5'
call try "trace -'r'"
call try 'trace -3.5'
call try 'trace -(3)'
call try 'trace -3 x'
call try 'trace -'
call try 'trace +'
call try 'trace - -3'
exit
try: procedure
  signal on syntax name e
  interpret arg(1)
  say arg(1) 'set'
  return
e: say arg(1) condition('o')~code
  return
```

```output
trace -3 24.901
trace + 5 24.901
trace -'r' 26.7
trace -3.5 26.7
trace -(3) 35.1
trace -3 x 21.906
trace - set
trace + 35.1
trace - -3 35.1
```

r[instr.trace.invalid]
A setting that is neither a number nor [valid](r:bif.trace.error) raises
`SYNTAX 24.1`. A letter that is valid but is
followed by anything else is no error, so `a5` is `A`.

```rexx
call try 'x'
call try '? 5'
call try 'a5'
call try 'resultsx'
exit
try: procedure
  signal on syntax name e
  trace value arg(1)
  say arg(1) 'set' trace()
  return
e: say arg(1) condition('o')~code
  return
```

```output
x 24.1
? 5 24.1
a5 set A
resultsx set R
```

r[instr.trace.syntax]
Data after the setting, other than the expression of VALUE or the rest of a
clause that starts as [the value rule](r:instr.trace.form.value) says, raises
`SYNTAX 21.906`. VALUE with no expression raises `SYNTAX 35.916`. Both are
found when the program is read, so a program file that has one writes nothing
and ends with the return code of the error, and the subcode shows only in
INTERPRET text.

```rexx
say 'not run'
trace results extra
```

```output rc=235
```

```rexx
say 'not run'
trace value
```

```output rc=221
```

```rexx
call try 'trace results extra'
call try 'trace value'
call try 'trace'
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
trace results extra 21.906
trace value 35.916
trace ok
```

## When it takes effect

r[instr.trace.effect]
A clause is traced with the setting that was in force when it began. The
TRACE clause itself is therefore traced under the old setting and not the new
one: `TRACE R` after `N` shows nothing, and `TRACE N` after `R` is the last
line shown. The setting is in force from the next clause on.

```rexx
.traceoutput~destination(.output)
trace r
x = 1
trace n
x = 2
```

```output
     3 *-* x = 1
       >>>   "1"
     4 *-* trace n
```

r[instr.trace.scope]
The instruction changes the setting that the TRACE function reports and
changes, and the setting has [the same scope](r:bif.trace.scope): an internal
routine starts with the caller's setting and the caller has its own back when
the routine returns, wherever the routine changed it.

```rexx
trace r
call sub
say 'caller' trace()
exit
sub:
  say 'sub' trace()
  trace i
  say 'sub' trace()
  return
```

```output
sub R
sub I
caller R
```

## Where the lines go

r[instr.trace.destination]
Every trace line is written by sending the message `LINEOUT` to the object
that the environment symbol `.TRACEOUTPUT` holds, with one argument, an object
whose string value is the line without a line end. The `.TRACEOUTPUT` object
is a monitor, and by default it writes to the error stream, so a program that
traces shows nothing on its standard output. A program sets another
destination with the message `DESTINATION` to `.TRACEOUTPUT`: any object that
understands `LINEOUT` will do, and `.OUTPUT` sends the lines to standard
output.

```rexx
trace r
x = 1
say 'done'
```

```output
done
```

```rexx
.traceoutput~destination(.sink~new)
trace r
x = 1
trace n
exit
::class sink
::method lineout
  use arg line
  .output~lineout('got <'line'>')
  return 0
```

```output
got <     3 *-* x = 1>
got <       >>>   "1">
got <     4 *-* trace n>
```

## The format of a line

r[instr.trace.line]
A line that shows a clause has the line number right-aligned in six columns,
a blank, the flag `*-*`, a blank, [the indentation](r:instr.trace.indent) and
the text of the clause. The line
number is the program line on which the clause starts, and clauses that share
a line share its number. A clause run by INTERPRET carries the line number of
the INTERPRET clause, and its text is the text of the string.

```rexx
.traceoutput~destination(.output)
trace r
x  =  3
y = x + 4 ; z = 'a'  'b'
-- 5
-- 6
-- 7
-- 8
-- 9
w = 10
```

```output
     3 *-* x  =  3
       >>>   "3"
     4 *-* y = x + 4 ;
       >>>   "7"
     4 *-* z = 'a'  'b'
       >>>   "a b"
    10 *-* w = 10
       >>>   "10"
```

```rexx
.traceoutput~destination(.output)
trace r
interpret 'x = 1; y = 2'
```

```output
     3 *-* interpret 'x = 1; y = 2'
       >>>   "x = 1; y = 2"
     3 *-* x = 1;
       >>>   "1"
     3 *-* y = 2
       >>>   "2"
```

r[instr.trace.text]
The text of a clause is shown as it was written, from its first token on: the
blanks between tokens are kept, a comment between tokens is kept, a comment
before the first token and a line end are not, and a clause that ends with a
semicolon shows the semicolon. A comment before that semicolon is shown, and
one after it is not. A comment after the last token on its line, with no
semicolon after it, is kept. A clause that ran over several lines, by a
continuation or by a comment, is shown on one line with the line ends
removed, the comma of a continuation and the blanks that begin the next line
kept, and carries the number of its first line. The text of an IF or WHEN
clause ends before THEN, so it keeps the blanks that stood before THEN.

```rexx
.traceoutput~destination(.output)
trace r
/* lead */ a  =  1 /* mid */ + 2 ;
b = 3 +,
    4
c = 5   /* tail */
```

```output
     3 *-* a  =  1 /* mid */ + 2 ;
       >>>   "3"
     4 *-* b = 3 +,    4
       >>>   "7"
     6 *-* c = 5   /* tail */
       >>>   "5"
```

```rexx
.traceoutput~destination(.output)
trace r
n = 1
if n=1 then nop
if (n=1)then nop
g = 7 ; -- after the semicolon
c = 3 /* before */ ;
f = 6 /* two
  lines */ + 1
```

```output
     3 *-* n = 1
       >>>   "1"
     4 *-* if n=1 
       >>>   "1"
     4 *-*   then
     4 *-*     nop
     5 *-* if (n=1)
       >>>   "1"
     5 *-*   then
     5 *-*     nop
     6 *-* g = 7 ;
       >>>   "7"
     7 *-* c = 3 /* before */ ;
       >>>   "3"
     8 *-* f = 6 /* two  lines */ + 1
       >>>   "7"
```

r[instr.trace.result]
A line that shows a value is written below the line of its clause, with seven
blanks, a three-character flag, a blank, the indentation of the clause, two
more blanks and the value in double quotation marks. Blanks inside the quotes
are the value's own. The tab `'09'x`, the line feed `'0A'x`
and the carriage return `'0D'x` are shown as themselves, every other byte
below `'20'x` is shown as `?`, and the bytes from `'20'x` up, `'7F'x` and
`'FF'x` among them, are shown as themselves. A value that is not a string is
shown as its string value.

```rexx
.traceoutput~destination(.output)
trace r
x = ' two  blanks '
y = .array~new
z = 1.50 + 1
```

```output
     3 *-* x = ' two  blanks '
       >>>   " two  blanks "
     4 *-* y = .array~new
       >>>   "an Array"
     5 *-* z = 1.50 + 1
       >>>   "2.50"
```

```rexx
.traceoutput~destination(.sink~new)
trace r
x = 'a' || '01'x || 'b' || '09'x || '1b'x || 'c' || '7f'x || 'ff'x
trace n
exit
::class sink
::method lineout
  use arg line
  line = '' || line
  if line~pos('>>>') > 0 then say line~right(10)~c2x
  return 0
```

```output
22613F62093F637FFF22
```

## What a setting lists

r[instr.trace.setting.clauses]
Under `A` every clause is shown, before it runs, and nothing else about it.
`R` shows the same clauses and below each of them the final value of its
expression, with the flag `>>>`. `I` shows in addition every step of the
evaluation, with the flags that [the tag rule](r:instr.trace.tags) lists. `O`
shows nothing. The TRACE clause that ends a setting is shown under the setting
it ends, as [the effect rule](r:instr.trace.effect) says.

```rexx
.traceoutput~destination(.output)
call work 'A'
say '--'
call work 'R'
say '--'
call work 'I'
say '--'
call work 'O'
exit
work: procedure
  trace value arg(1)
  n = 2
  x = n + 1
  trace off
  return
```

```output
    12 *-*   n = 2
    13 *-*   x = n + 1
    14 *-*   trace off
--
    12 *-*   n = 2
       >>>     "2"
    13 *-*   x = n + 1
       >>>     "3"
    14 *-*   trace off
--
    12 *-*   n = 2
       >L>     "2"
       >>>     "2"
       >=>     N <= "2"
    13 *-*   x = n + 1
       >V>     N => "2"
       >L>     "1"
       >O>     "+" => "3"
       >>>     "3"
       >=>     X <= "3"
    14 *-*   trace off
--
```

r[instr.trace.setting.commands]
A command is shown by `A`, `R` and `I` with the rest of the clauses, and three
more settings show commands and nothing else. `C` shows each command before it
runs. `E` shows a command that raised `ERROR` or `FAILURE`, `F` shows one that
raised `FAILURE`, and `N` shows what `F` shows. A shown command is followed by
a line with the flag `>>>` that holds the command string that was sent, and by
a line with the flag `+++` that holds `RC(n)` and the return code, when the
command raised one of the two conditions; the return code is 30 when the
environment does not exist. Which commands raise the conditions is specified
with [the commands page](r:instr.command.status).

```rexx
.traceoutput~destination(.output)
call cmds 'C'
say '--'
call cmds 'E'
say '--'
call cmds 'F'
say '--'
call cmds 'N'
exit
cmds: procedure
  trace value arg(1)
  address system
  'exit 0'
  'exit 3'
  address nosuch
  'exit 0'
  trace off
  return
```

```output
    13 *-*   'exit 0'
       >>>     "exit 0"
    14 *-*   'exit 3'
       >>>     "exit 3"
       +++     "RC(3)"
    16 *-*   'exit 0'
       >>>     "exit 0"
       +++     "RC(30)"
--
    14 *-*   'exit 3'
       >>>     "exit 3"
       +++     "RC(3)"
    16 *-*   'exit 0'
       >>>     "exit 0"
       +++     "RC(30)"
--
    16 *-*   'exit 0'
       >>>     "exit 0"
       +++     "RC(30)"
--
    16 *-*   'exit 0'
       >>>     "exit 0"
       +++     "RC(30)"
```

r[instr.trace.setting.labels]
`L` shows a label clause when control reaches it: by running into it, by a
CALL of the routine it names, or by a SIGNAL to it. It shows no other clause.

```rexx
.traceoutput~destination(.output)
trace l
x = 1
fall:
y = 2
call sub
signal there
z = 3
there:
exit
sub:
  return
```

```output
     4 *-* fall:
    11 *-*   sub:
     9 *-* there:
```

> **Note.** A method or routine that traces from its first clause also
> writes a line with the flag `>I>` when it is entered and `<I<` when it
> returns, and the text of those lines names the package. The book does not
> specify them.

## The tags

r[instr.trace.tags]
The flag of a value line says what produced the value. Under `R` the value
lines carry `>>>`, the final value of an expression, a value that a `PARSE`,
`ARG`, `PULL` or `USE` instruction took from its source, and the value that a
routine returns to its caller; a message instruction and a message assignment
write no value line. The flag `>K>` marks, under `R` and `I`, the value of a
keyword's expression in `DO`, `NUMERIC`, `PARSE`, `RAISE`, `FORWARD`, `GUARD`,
`SIGNAL VALUE` and `TRACE` with a value, and a redirection of an `ADDRESS`
command; it is shown as the keyword and its value. `ADDRESS VALUE` and
`OPTIONS` write only `>>>`. `>R>` is a variable reference that `USE ARG` took,
shown as `"V" => "W"`, and is written under `R` and `I`. The `+++` line of a
command is the other kind of line of this setting. The other flags belong to
`I` alone. `>L>` is a literal and `>V>` the value of a
variable, shown as `NAME => "value"`. `>C>` is a compound variable whose tail
was substituted, shown as the substituted name, as in `S.K => "S.1"`, `>E>` an
environment symbol, and `>N>` a symbol qualified by a namespace, shown as
`NS:C => "The C class"`. `>O>` is the result of a binary operator and `>P>`
that of a prefix operator, each shown as the operator and its result, `>F>`
the result of a function call and `>M>` that of a message, shown as the name
and the result, and `>A>` an argument of a call or of a message. `>=>` is an
assignment, shown as `NAME <= "value"`, and `>.>` a value that a placeholder
period of a template took. Names are shown in upper case.

```rexx
.traceoutput~destination(.output)
trace i
s.1 = 5
k = 1
y = s.k + -abs(-2)
a = .array~new
b = a~items
parse value 'p q' with . w
numeric digits 12
```

```output
     3 *-* s.1 = 5
       >L>   "5"
       >>>   "5"
       >C>   S.1 => "S.1"
       >=>   S.1 <= "5"
     4 *-* k = 1
       >L>   "1"
       >>>   "1"
       >=>   K <= "1"
     5 *-* y = s.k + -abs(-2)
       >C>   S.K => "S.1"
       >V>   S.K => "5"
       >L>   "2"
       >P>   "-" => "-2"
       >A>   "-2"
       >F>   ABS => "2"
       >P>   "-" => "-2"
       >O>   "+" => "3"
       >>>   "3"
       >=>   Y <= "3"
     6 *-* a = .array~new
       >E>   .ARRAY => "The Array class"
       >M>   "NEW" => "an Array"
       >>>   "an Array"
       >=>   A <= "an Array"
     7 *-* b = a~items
       >V>   A => "an Array"
       >M>   "ITEMS" => "0"
       >>>   "0"
       >=>   B <= "0"
     8 *-* parse value 'p q' with . w
       >L>   "p q"
       >K>   "VALUE" => "p q"
       >>>   "p q"
       >.>   "p"
       >=>   W <= "q"
     9 *-* numeric digits 12
       >L>   "12"
       >K>   "DIGITS" => "12"
```

```rexx
.traceoutput~destination(.output)
trace r
parse value 'a b c' with p . r
call sub 1, 2
exit
sub: use arg a, b
  return a + b
```

```output
     3 *-* parse value 'a b c' with p . r
       >K>   "VALUE" => "a b c"
       >>>   "a b c"
       >>>   "a"
       >>>   "c"
     4 *-* call sub 1, 2
     6 *-*   sub:
     6 *-*   use arg a, b
       >>>     "1"
       >>>     "2"
     7 *-*   return a + b
       >>>     "3"
       >>>   "3"
     5 *-* exit
```

```rexx
.traceoutput~destination(.output)
x = .array~new
v = 4
trace r
x~append(1)
x[1] = 2
call sub >v
signal on syntax name e
raise syntax 93.900 description 'd' additional 'ad'
e: exit
sub: use arg >w
  return
```

```output
     5 *-* x~append(1)
     6 *-* x[1] = 2
     7 *-* call sub >v
    11 *-*   sub:
    11 *-*   use arg >w
       >R>     "V" => "W"
    12 *-*   return
     8 *-* signal on syntax name e
     9 *-* raise syntax 93.900 description 'd' additional 'ad'
       >K>   "SYNTAX" => "93.900"
       >K>   "DESCRIPTION" => "d"
       >K>   "ADDITIONAL" => "ad"
    10 *-* e:
    10 *-* exit
```

```rexx
.traceoutput~destination(.output)
trace i
v = 1
call sub >v
trace n
exit
sub: use arg >w
  return
```

```output
     3 *-* v = 1
       >L>   "1"
       >>>   "1"
       >=>   V <= "1"
     4 *-* call sub >v
       >O>   ">" => "V"
       >A>   "1"
     7 *-*   sub:
     7 *-*   use arg >w
       >R>     "V" => "W"
     8 *-*   return
     5 *-* trace n
```

```rexx
.traceoutput~destination(.output)
trace i
trace value 'r'
trace ('n')
```

```output defect=#103
     3 *-* trace value 'r'
       >L>   "r"
       >K>   "VALUE" => "r"
     4 *-* trace ('n')
       >K>   "VALUE" => "n"
```

```rexx
lib = 'spec-trace-lib.tmp'
use = 'spec-trace-use.tmp'
call sysfiledelete lib
call sysfiledelete use
call lineout lib, '::class c public'
call lineout lib, '::routine pub public'
call lineout lib, '  return 1'
call lineout lib
call lineout use, 'trace i'
call lineout use, 'x = ns:c~new'
call lineout use, 'y = ns:pub()'
call lineout use, 'trace n'
call lineout use, "::requires '"lib"' namespace ns"
call lineout use
.traceoutput~destination(.sink~new)
call (use)
call sysfiledelete lib
call sysfiledelete use
::class sink
::method lineout
  use arg line
  line = '' || line
  if line~pos('>N>') > 0 | line~pos('>F>') > 0 then .output~lineout(line)
  return 0
```

```output
       >N>   NS:C => "The C class"
       >F>   PUB => "1"
```

## Indentation

r[instr.trace.indent]
The text of a clause is indented by two blanks for each level of nesting. A
level is added inside a DO group or loop, after THEN or ELSE, after WHEN or
OTHERWISE, and for each active internal call. THEN, ELSE, WHEN and OTHERWISE
are shown on a line of their own, one level in from the IF or SELECT, and the
clause after them one level further. DO, SELECT and the IF clause stand at the
level of their surroundings, and the END that closes a DO or a SELECT stands
there too. The lines of values are indented by the level of their clause and
two blanks more. The label of an internal routine and the clauses of its body
are shown one level deeper than the CALL that started it.

```rexx
.traceoutput~destination(.output)
trace r
n = 2
do
  if n = 2 then do
    select
      when n = 1 then nop
      otherwise
        m = n
    end
  end
  call f
end
exit
f:
  i = 1
  return
```

```output
     3 *-* n = 2
       >>>   "2"
     4 *-* do
     5 *-*   if n = 2 
       >>>     "1"
     5 *-*     then
     5 *-*       do
     6 *-*         select
     7 *-*           when n = 1 
       >>>             "0"
     8 *-*           otherwise
     9 *-*             m = n
       >>>               "2"
    10 *-*         end
    11 *-*       end
    12 *-*   call f
    15 *-*     f:
    16 *-*     i = 1
       >>>       "1"
    17 *-*     return
    13 *-* end
    14 *-* exit
```

r[instr.trace.indent.signal]
A SIGNAL sets the level to zero, so the clauses after it are shown from the
left margin, also when the SIGNAL stood in a DO group or in a called routine.
A condition that SIGNAL ON traps does the same.

```rexx
.traceoutput~destination(.output)
trace r
call f
c = 3
exit
f:
  do
    d = 4
    signal g
  end
g:
  e = 5
  return
```

```output
     3 *-* call f
     6 *-*   f:
     7 *-*   do
     8 *-*     d = 4
       >>>       "4"
     9 *-*     signal g
    11 *-* g:
    12 *-* e = 5
       >>>   "5"
    13 *-* return
     4 *-* c = 3
       >>>   "3"
     5 *-* exit
```

```rexx
.traceoutput~destination(.output)
trace r
signal on syntax name trap
call f
exit
f:
  do
    x = 1 / 0
  end
trap:
  z = 1
```

```output
     3 *-* signal on syntax name trap
     4 *-* call f
     6 *-*   f:
     7 *-*   do
     8 *-*     x = 1 / 0
    10 *-* trap:
    11 *-* z = 1
       >>>   "1"
```

## Concurrent activities

r[instr.trace.concurrent]
Each activity traces with its own setting and its own indentation, and the
lines of one activity reach the destination in the order that activity wrote
them. How the lines of two activities that trace at the same time are
interleaved is not specified, as [Deviation 7](../deviations.md) says.

```rexx
sink = .sink~new
.traceoutput~destination(sink)
m = .worker~new~start('run')
trace r
a1 = 1
a2 = 2
trace n
m~result
.traceoutput~destination(.output)
say 'main:'
do l over sink~lines
  if pos('*-* a', l) > 0 then say word(l, 1) word(l, 3)
end
say 'worker:'
do l over sink~lines
  if pos('*-* b', l) > 0 then say word(l, 1) word(l, 3)
end
::class sink
::method init
  expose lines
  lines = .array~new
::method lines
  expose lines
  return lines
::method lineout
  expose lines
  use arg l
  lines~append('' || l)
  return 0
::class worker
::method run
  trace r
  b1 = 1
  b2 = 2
  trace n
  return 1
```

```output
main:
5 a1
6 a2
worker:
33 b1
34 b2
```

## Deviations

r[instr.trace.deviation.0]
interrexx indents by the block structure alone, as [the indentation
rule](r:instr.trace.indent) says. ooRexx drops one level too many when a loop
ends because its test failed, and the lines after the loop stand one level
short until a construct around it restores the level. The example shows the
interrexx output; ooRexx shows the clause `x = 1` at the left margin.

```rexx
.traceoutput~destination(.output)
trace r
do
  do i = 1 to 2
    nop
  end
  x = 1
end
y = 2
```

```output
     3 *-* do
     4 *-*   do i = 1 to 2
       >K>     "TO" => "2"
     5 *-*     nop
     6 *-*   end
     4 *-*   do i = 1 to 2
       >>>       "1"
       >>>       "2"
     5 *-*     nop
     6 *-*   end
     4 *-*   do i = 1 to 2
       >>>       "2"
       >>>       "3"
     7 *-*   x = 1
       >>>     "1"
     8 *-* end
     9 *-* y = 2
       >>>   "2"
```

r[instr.trace.deviation.7]
The order in which the lines of two activities that trace at once reach the
destination is not specified; interrexx gives one of the orders ooRexx can
give. [The rule above](r:instr.trace.concurrent) is what a program may rely
on, together with the count of the lines, which does not depend on the order.

```rexx
sink = .sink~new
.traceoutput~destination(sink)
m = .worker~new~start('run')
trace r
a1 = 1
trace n
m~result
.traceoutput~destination(.output)
n = 0
do l over sink~lines
  if pos('*-*', l) > 0 then n = n + 1
end
say n 'clause lines'
::class sink
::method init
  expose lines
  lines = .array~new
::method lines
  expose lines
  return lines
::method lineout
  expose lines
  use arg l
  lines~append('' || l)
  return 0
::class worker
::method run
  trace r
  b1 = 1
  trace n
  return 1
```

```output
4 clause lines
```

r[instr.trace.deviation.50]
When the destination refuses a line by raising a condition, the condition is
the same as in ooRexx, but a clause may do its own work before the line that
refuses is written. In ooRexx the condition is raised before the clause does
anything. In the example the destination refuses the second line, the value
line of `say 'a'`; interrexx has written `a` by then, and ooRexx has not.

```rexx
.traceoutput~destination(.sink~new)
signal on syntax name e
trace o
trace r
say 'a'
x = 1
exit
e: trace o
.traceoutput~destination(.output)
say 'caught' condition('o')~code symbol('x')
::class sink
::method init
  expose n
  n = 0
::method lineout
  expose n
  n = n + 1
  if n = 2 then raise syntax 93.900
  return 0
```

```output
a
caught 93.900 LIT
```
