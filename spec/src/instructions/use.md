---
element: USE
kind: instruction
syntax: "USE [STRICT] ARG [target [= default]] [, ...] | USE LOCAL [name]..."
rexxref: "Keyword Instructions > USE"
deviations: [71]
---

# USE

USE has two unrelated forms. USE ARG takes the arguments of the running
program, routine or method and stores the argument objects themselves in
variables, where [ARG](arg.md) and [PARSE ARG](r:instr.parse.arg) make strings
of them and cut them up with a template. USE STRICT ARG is the same with a
check of the number of arguments. USE LOCAL is for methods only and decides
which of the method's variables are its own. This page specifies the targets
and defaults of USE ARG, the strict form and the errors of both forms. How the
arguments are counted and read is the matter of [the ARG
builtin](r:bif.arg.args).

## USE ARG

r[instr.use.arg]
The targets of USE ARG are filled from left to right. The target in position
`n` receives argument `n`, as an object, without any change: it is not
converted to a string, its case stays, and a collection is the same
collection that the caller passed. A target for which no argument was passed,
because the caller passed fewer arguments or left that position empty, is
dropped, so a variable it had before is unset. An argument beyond the last
target is ignored. The clause may run again, and it reads the arguments the
call was made with, not the current values of the variables. A target can be a
simple symbol, a compound symbol or a stem symbol, or it can be left empty,
which skips its argument and sets nothing.

```rexx
call show 1, 2, 3
call show 'a'
call show
call show , 'b'
call show 1, , 3, 4
call take .array~of('x', 'y'), 'Mixed Case', 12
exit
show:
  use arg x, y, z
  say 'x=' || x 'y=' || y 'z=' || z symbol('x') symbol('y') symbol('z') arg()
  return
take:
  use arg a, b, c
  a = 'changed'
  use arg again
  say again~class~id again~items b c + 1
  return
```

```output
x=1 y=2 z=3 VAR VAR VAR 3
x=a y=Y z=Z VAR LIT LIT 1
x=X y=Y z=Z LIT LIT LIT 0
x=X y=b z=Z LIT VAR LIT 2
x=1 y=Y z=3 VAR LIT VAR 4
Array 2 Mixed Case 13
```

```rexx
call skip 'v1', 'v2'
say symbol('a') symbol('b') b
exit
skip:
  use arg , b
  return
```

```output
LIT VAR v2
```

r[instr.use.compound]
The tail of a compound target is resolved when that target is filled, with the
values its variables have at that moment, as for [an
assignment](r:expr.variables.compound) to the symbol. A target to the left
that set a variable of the tail therefore counts. The argument is stored
under the resulting name.

```rexx
k = 5
call fill 'v1', 'v2'
say c.1 c.5
call chain 7, 'second'
say d.7
exit
fill:
  use arg c.1, c.k
  return
chain:
  use arg t, d.t
  return
```

```output
v1 v2
second
```

r[instr.use.stem]
A stem symbol as a target receives the argument as the stem. When the caller
passed a stem, as in `call s st.`, the target is that stem, not a copy, so
the routine's assignments to it are the caller's. When the argument is not a
stem, the target is a new stem that has the argument as its default value, as
[assigning a value to a stem](r:expr.variables.stem) does.

```rexx
st.1 = 'one'
call change st.
say st.1 st.2
exit
change: procedure
  use arg sub.
  say sub.1
  sub.2 = 'two'
  return
```

```output
one
one two
```

```rexx
call plain 'str'
exit
plain:
  use arg s.
  say s.1 s.xyz s.~class~id
  return
```

```output
str str Stem
```

r[instr.use.message]
A target can be a message term, such as `obj~name` or `arr[1]`, whose object
is a variable that has a value, one that an earlier target of the clause set
included. The argument is then sent as the assignment of that term, as [the
message assignment rule](r:instr.message.assign) describes, so it sets the
attribute or the item. The message terms are evaluated in order, and a message
term whose argument is missing is skipped: it sends nothing and changes
nothing.

```rexx
o = .thing~new
arr = .array~new
arr[2] = 'keep'
call fill o, arr, 'v1', 'v2'
say o~name arr[1] arr[2]
call fill o, arr
say o~name arr[1] arr[2]
call fill o, arr, 'w1'
say o~name arr[1] arr[2]
exit
fill:
  use arg obj, ar, obj~name, ar[1]
  return
::class thing
::attribute name
```

```output
v1 v2 keep
v1 v2 keep
w1 v2 keep
```

r[instr.use.reference]
A target written with a `<` before its name is an alias of a variable of the
caller. The caller passes the variable as a reference, a `<` or a `>` before its
name in the call ([the reference rule](r:class.variablereference.arguments)),
and the target is then the same variable: a value read through it is the
caller's, and an assignment changes the caller's variable. The target needs a
reference as its argument, and a plain value raises `SYNTAX 88.928`. The
argument must not be omitted, which raises `SYNTAX 88.931`. The alias name must
not be a variable of the routine that has a value, or the clause raises `SYNTAX
98.995`. A default after a reference target raises `SYNTAX 99.950` when the
program is read. The reference that such an argument passes is a
`VariableReference` ([its page](r:class.variablereference.creation)).

```rexx
v = 'orig'
call s <v
say v
call try 'plain'
call try
call taken <v
exit
s:
  use arg <x
  say x
  x = 'changed'
  return
try:
  signal on syntax name e
  use arg <alias
  say 'ok'
  return
taken:
  signal on syntax name e
  has = 'a value'
  use arg <has
  say 'ok'
  return
e: say arg() 'args =>' condition('o')~code
  return
```

```output
orig
changed
1 args => 88.928
0 args => 88.931
1 args => 98.995
```

```rexx
use arg <x = 1
```

```output rc=157
```

r[instr.use.default]
A target can be followed by `=` and a default. When the argument for that
position was not passed, the target takes the value of the default, which is
evaluated at that moment and only then, after the targets to its left have
been filled. An argument of the null string is passed, so it replaces the
default. The default is a literal string, a constant symbol such as a number,
an environment symbol such as `.nil`, or an expression in parentheses. Any
other text after the `=` raises `SYNTAX 35.1` for a variable symbol and
`35.930` for something that is not a single term or has no term at all.

```rexx
n = 0
call s 1
call s 1, 2
call s , 'x', ''
call bad 'a = b'
call bad 'a = 1 + 2'
call bad 'a ='
call bad 'a = (1 + 2)'
exit
s:
  use arg p, q = (p + 10), r = .nil, t = 12, u = (tick())
  say p q r t u arg()
  return
tick: n = n + 1
  return 'tick' n
bad:
  signal on syntax name e
  interpret 'use arg' arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
1 11 The NIL object 12 tick 1 1
1 2 The NIL object 12 tick 2 2
P x  12 tick 3 3
a = b => 35.1
a = 1 + 2 => 35.930
a = => 35.930
a = (1 + 2) => ok
```

## USE STRICT ARG

r[instr.use.strict]
USE STRICT ARG fills the targets as USE ARG does, after it has checked the
number of arguments. A target with no default is required, and a target with
a default is optional, so the number of arguments passed must not be smaller
than the number of required targets, and must not be greater than the number
of targets. A position that is left empty counts, but trailing positions left
empty do not, in the sense of [the ARG builtin](r:bif.arg.count). A required
position that is left empty is a third fault. In a program, an internal
routine or an external routine the three faults raise `SYNTAX 40.3`, `40.4`
and `40.5`; in a method they raise `SYNTAX 93.901`, `93.902` and `93.903`. A
required target may follow an optional one: the count must still reach it.
Without a target the clause demands no argument at all.

```rexx
call s
call s 1
call s 1, 2
call s 1, 2, 3
call s , 2
call none
call none 1
call late 1, 2
call late 1, 2, 3
exit
s: signal on syntax name e
  use strict arg a, b = 'B'
  say arg() 'ok' a b
  return
none: signal on syntax name e
  use strict arg
  say arg() 'ok'
  return
late: signal on syntax name e
  use strict arg a, b = 'B', c
  say arg() 'ok' a b c
  return
e: say arg() 'args =>' condition('o')~code
  return
```

```output
0 args => 40.3
1 ok 1 B
2 ok 1 2
3 args => 40.4
2 args => 40.5
0 ok
1 args => 40.4
2 args => 40.3
3 ok 1 2 3
```

```rexx
o = .m~new
call try 'none'
call try 'three'
call try 'trailing'
call try 'empty'
exit
try:
  signal on syntax name e
  select
    when arg(1) = 'none' then say o~m
    when arg(1) = 'three' then say o~m(1, 2, 3)
    when arg(1) = 'trailing' then say o~m(1, , )
    otherwise say o~m(, 'x')
  end
  return
e: say condition('o')~code
  return
::class m
::method m
  use strict arg a, b = 'B'
  return 'ok' arg() a b
```

```output
93.901
93.902
ok 1 1 B
93.903
```

r[instr.use.ellipsis]
Three periods after the last target of a USE STRICT ARG mean that more
arguments may follow, so the upper limit of the count is lifted and the
lower limit stays. The arguments beyond the targets are not stored, and the
routine reads them with the ARG builtin. The three periods are accepted after
USE ARG without strict too, where they change nothing. They must be the
last item of the clause: a target after them raises `SYNTAX 99.930`.

```rexx
call s 1
call s 1, 2, 3, 4, 5
call s 1, , 3
call s
call bad 'use arg a, ..., b'
call bad 'use strict arg a, ..., b'
call bad 'use arg a, ...'
exit
s: signal on syntax name e
  use strict arg a, b = 'B', ...
  say arg() 'ok' a b arg(arg())
  return
e: say arg() 'args =>' condition('o')~code
  return
bad:
  signal on syntax name f
  interpret arg(1)
  say arg(1) '=> ok'
  return
f: say arg(1) '=>' condition('o')~code
  return
```

```output
1 ok 1 B 1
5 ok 1 2 5
3 ok 1 B 3
0 args => 40.3
use arg a, ..., b => 99.930
use strict arg a, ..., b => 99.930
use arg a, ... => ok
```

## USE LOCAL

r[instr.use.local]
USE LOCAL reverses the division of a method's variables. With it every
variable of the method is an object variable, as if it were exposed, except
the names the clause lists, which are the method's own. A listed name can be
a simple or a stem symbol; a stem lists every compound variable of it. The
special variables `RC`, `RESULT`, `SIGL`, `SELF` and `SUPER` are never object
variables, whatever USE LOCAL lists. A USE LOCAL with no name leaves only
these local. [The EXPOSE
rule](r:instr.expose.object) says what an object variable is.

```rexx
o = .box~new
say o~a
say o~b
say o~c
::class box
::method a
  use local keep st.
  keep = 'local'
  st.1 = 'local too'
  shared = 'object'
  call sub
  return symbol('result') symbol('sigl') symbol('keep') symbol('st.1')
sub:
  return 'r'
::method b
  use local
  return symbol('shared') symbol('keep') symbol('st.1') symbol('result'),
    symbol('sigl')
::method c
  use local
  return symbol('shared') shared
```

```output
VAR VAR VAR VAR
VAR LIT LIT LIT LIT
VAR object
```

r[instr.use.local.place]
USE LOCAL must be the first instruction of a method, with only
comments and null clauses before it. Any other instruction before it, a label,
a USE ARG, an EXPOSE or a second USE LOCAL included, and a position inside a
DO or an IF, raise `SYNTAX 99.910` when the program is read. An EXPOSE after
it raises `99.907`, since it is not the first instruction. USE ARG may follow
it. In the main program or a routine, USE LOCAL as the first instruction
raises `SYNTAX 98.993` when it runs, and the text of an INTERPRET raises
`99.915` when the INTERPRET runs. A name that is a compound symbol raises
`99.948`, one that starts with a digit `31.2`, and a comma, a string, a
parenthesis or an `=` where a name should be `20.927`.

```rexx
call try 'use local x'
call try 'use local'
call try 'nop; use local x'
call try 'use local x; use local y'
call try 'use arg a; use local x'
call try 'use local x; use arg a'
call try 'use local x; expose y'
call try 'lab: use local x'
call try 'if 1 then use local x'
call try 'do 1; use local x; end'
call try 'use local x.1'
call try 'use local x.'
call try 'use local 3'
call try 'use local x, y'
call try 'use local (x)'
call try 'use local x = 1'
exit
try:
  signal on syntax name e
  m = .method~new('m', arg(1))
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
use local x => ok
use local => ok
nop; use local x => 99.910
use local x; use local y => 99.910
use arg a; use local x => 99.910
use local x; use arg a => ok
use local x; expose y => 99.907
lab: use local x => 99.910
if 1 then use local x => 99.910
do 1; use local x; end => 99.910
use local x.1 => 99.948
use local x. => ok
use local 3 => 31.2
use local x, y => 20.927
use local (x) => 20.927
use local x = 1 => 20.927
```

```rexx
signal on syntax name e
r = .routine~new('r', 'use local x')
r~call
e: say condition('o')~code
```

```output
98.993
```

```rexx
use local x
```

```output rc=158
```

```rexx
o = .box~new
signal on syntax name e
o~run
e: say condition('o')~code
::class box
::method run
  interpret 'use local x'
```

```output
99.915
```

## Errors of the clause

r[instr.use.syntax]
A fault in the form of the clause is raised when the program is read. USE
with no word after it, and USE with a word that is not ARG, STRICT or LOCAL,
raise `SYNTAX 25.905`. STRICT that is not followed by ARG raises `25.929`.
Two targets that follow each other with no comma raise `46.902`. A target
that starts with a digit raises `31.2`, one that starts with a period `31.3`,
and a string or a parenthesis in a target's place `89.1`. The text of an
INTERPRET raises the same errors when the INTERPRET runs.

```rexx
call try 'use'
call try 'use foo a'
call try 'use strict a'
call try 'use strict arg a b'
call try 'use arg 1'
call try 'use arg .a'
call try 'use arg "x"'
call try 'use arg (a)'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
use => 25.905
use foo a => 25.905
use strict a => 25.929
use strict arg a b => 46.902
use arg 1 => 31.2
use arg .a => 31.3
use arg "x" => 89.1
use arg (a) => 89.1
```

## Deviations

r[instr.use.deviation.71]
`.context~variables` lists the alias that a reference target made under the
name the routine reaches it by. ooRexx lists it under the name of the
variable that it aliases, so its listing holds the caller's name only.

```rexx
v = 'orig'
call s <v
exit
s:
  use arg <x
  say .context~variables~allIndexes~sort~makeString('l', ' ')
  return
```

```output
SIGL V X
```
