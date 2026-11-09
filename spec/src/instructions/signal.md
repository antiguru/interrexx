---
element: SIGNAL
kind: instruction
syntax: "SIGNAL {label | [VALUE] expression | ON condition [NAME label] | OFF condition}"
ansi: "6.3.2.82, 8.3.25"
rexxref: "Keyword Instructions > SIGNAL"
---

# SIGNAL

SIGNAL moves control to a label, and it also switches the trap of a condition
on or off. The jump goes to a label by name, anywhere in the program, and it
abandons the blocks that are open in the routine, so it is the instruction for
leaving a deeply nested construct and for a handler that does not come back.
The other form, with ON or OFF, does not move control. This page specifies how
the label is found, what the jump ends, what it sets, and the shape of the ON
and OFF forms. What a trap does when its condition occurs belongs to [the
chapter on conditions](../conditions/traps.md), which also specifies [SIGNAL's
role](r:cond.traps.signal) there.

## The label

r[instr.signal.label]
`SIGNAL name` continues at the first label that has the text of the name,
among the labels of the code that holds the SIGNAL: the main program, or the
body of the one routine or method that the SIGNAL is in. A name written as a
symbol is read in upper case, and a name written as a string is used as it is
written. [The label rule](r:lex.clauses.label) says which labels these find
and which of several of one name is the first. A symbol is a constant here:
its value as a variable plays no part. When no label has the name, the
instruction raises `SYNTAX 16.1`. The error is raised when the SIGNAL runs, so
the clauses before it have run. A label of the main program is therefore not
one that a method or a routine can reach, and the labels of a method are not
those of the main program.

```rexx
signal Go
say 'skipped'
go: say 'at go'
signal 'GO2'
say 'skipped'
go2: say 'at go2'
signal "Mixed"
say 'skipped'
"Mixed": say 'at Mixed'
signal 5
say 'skipped'
5: say 'at five'
call try 'mixed'
call try '"mixed"'
call try 'nosuch'
exit
try:
  signal on syntax name e
  interpret 'signal' arg(1)
e: say arg(1) '=>' condition('o')~code
  return
```

```output
at go
at go2
at Mixed
at five
mixed => 16.1
"mixed" => 16.1
nosuch => 16.1
```

```rexx
.c~new~m
exit
mainlab: say 'main label'
::class c
::method m
  signal on syntax name e
  signal mainlab
e: say 'method' condition('o')~code
  signal here
  say 'skipped'
here: say 'method label'
```

```output
method 16.1
method label
```

r[instr.signal.value]
`SIGNAL VALUE expression` evaluates the expression and uses the string it
converts to as the name, by [the conversion rule](r:conv.string.object): the
answer of its `MAKESTRING` method when it has one and of its `STRING` method
otherwise. The value is compared with the labels as it is, so a lower-case
value finds no symbol label, which is written in capitals. A token after
SIGNAL that is neither a symbol nor a string, such as a parenthesis, starts an
expression, so VALUE is needed only when the expression itself starts with a
symbol or a string, which would otherwise be the name. A value that matches
no label raises `SYNTAX 16.1`, and so does one that is not a name at all, such
as `-1`.

```rexx
signal value .name~new
say 'skipped'
ms: say 'at ms'
st: say 'at st'
name = 'DEST'
signal value name
say 'skipped'
dest: say 'at dest'
signal value 'DE' || 'ST2'
say 'skipped'
dest2: say 'at dest2'
signal ('DE' || 'ST3')
say 'skipped'
dest3: say 'at dest3'
call try 'value "dest"'
call try 'value 1 2'
call try '-1'
call try 'value "DEST" || "x"'
exit
try:
  signal on syntax name e
  interpret 'signal' arg(1)
e: say arg(1) '=>' condition('o')~code
  return
::class name
::method makestring
  return 'MS'
::method string
  return 'ST'
```

```output
at ms
at st
at dest
at dest2
at dest3
value "dest" => 16.1
value 1 2 => 16.1
-1 => 16.1
value "DEST" || "x" => 16.1
```

r[instr.signal.syntax]
A SIGNAL with a name takes nothing after it: data after a symbol or a string
raises `SYNTAX 21.905`, so `signal 'a' || 'b'` is an error, since the string
is already a complete name. A SIGNAL with nothing after it raises `19.4`, and
a VALUE with no expression raises `35.915`. These are found when the program
is read, so none of its clauses run.

```rexx
say 'before'
signal a b
```

```output rc=235
```

```rexx
call try 'signal a b'
call try 'signal "a" || "b"'
call try 'signal'
call try 'signal value'
exit
try:
  signal on syntax name e
  interpret arg(1)
e: say arg(1) '=>' condition('o')~code
  return
```

```output
signal a b => 21.905
signal "a" || "b" => 21.905
signal => 19.4
signal value => 35.915
```

## What it ends

r[instr.signal.ends]
A jump ends every DO, LOOP, SELECT and IF that is open in the routine that
holds the SIGNAL, without running their tests and without stepping a control
variable, which keeps its value. A loop that was ended is not resumed, and a
jump to a label before a loop runs the loop afresh. A label cannot stand
inside a loop, as [the label rule](r:lex.clauses.label) says, so a SIGNAL
cannot enter one. The blocks of the callers of
the routine are not ended: a routine that jumps inside itself returns to a
caller that is still in its loop. A SIGNAL in the text of an INTERPRET also
leaves the text, as [the INTERPRET rule](r:instr.interpret.transfer) says.

```rexx
do i = 1 to 2
  call sub
  say 'caller loop' i
end
say 'end' i
do j = 1 to 3
  select
    when j = 2 then signal out
    otherwise nop
  end
end
out: say 'out' j
n = 0
top:
do k = 1 to 3
  n = n + 1
  if n = 2 then signal top
  say n k
end
say 'again' n k
exit
sub:
  do 2
    signal done
  end
done: say 'done' i
  return
```

```output
done 1
caller loop 1
done 2
caller loop 2
end 3
out 2
1 1
3 1
4 2
5 3
again 5 4
```

## What it sets

r[instr.signal.sigl]
When control reaches the label, the variable `SIGL` holds the number of the
line of the SIGNAL clause, which is the line where the clause starts when it
runs over several lines. `SIGL` is an ordinary variable that the interpreter
sets, so an assignment to it is replaced by the next SIGNAL or CALL, and
before the first of them it is unset. SIGNAL sets nothing else: `RESULT` and
`RC` are unchanged.

```rexx
say 'start' sigl
signal first
nop
first: say 'first' sigl
signal second /* the comment
                 spans two lines */
second: say 'second' sigl
sigl = 'mine'
rc = 'r'; result = 'v'
signal third
third: say 'third' sigl rc result
```

```output
start SIGL
first 2
second 5
third 10 r v
```

r[instr.signal.routine]
A SIGNAL that jumps to a label outside the routine that holds it does not
leave the routine. The routine keeps its variables, its arguments and its
PROCEDURE scope, so the code at the label sees them and not the caller's, and
`SIGL` is set in that scope. A RETURN that the code at the label then
reaches returns from the routine to the clause after its CALL, with the
value of the RETURN as `RESULT`.

```rexx
z = 'main'
call sub 'p', 'q'
say 'caller continues' result
exit
back: say 'at back' arg() symbol('z') y sigl
  return 'ret'
sub: procedure
  y = 'sub y'
  signal back
```

```output
at back 2 LIT sub y 9
caller continues ret
```

## ON and OFF

r[instr.signal.on]
`SIGNAL ON condition` enables the trap of a condition, and
`SIGNAL OFF condition` disables it. The condition is ERROR, FAILURE, HALT,
LOSTDIGITS, NOMETHOD, NOSTRING, NOTREADY, NOVALUE, SYNTAX or ANY, or USER
followed by the name of a user condition, which is a symbol read as a constant
in upper case, so `user foo` names the condition `USER FOO`, and not a string.
ON may be followed by `NAME` and a label, a symbol or a string, that the trap
goes to, and the default label is the name of the condition. The label is not
looked for when the trap is enabled, so an ON for a label that does not exist
is not an error then. What a trap does when its condition occurs, and the role
of the label, are the subject of [the conditions
chapter](../conditions/traps.md). OFF takes no NAME.

```rexx
do c over .array~of('error', 'failure', 'halt', 'lostdigits',,
    'nomethod', 'nostring', 'notready', 'novalue', 'syntax', 'any')
  interpret 'signal on' c
  interpret 'signal off' c
  say c 'on and off'
end
signal on user foo name lbl
signal off user foo
signal on error name nolabel
signal on novalue name 'Mixed Case'
signal off error
signal off novalue
say 'done'
lbl: nop
```

```output
error on and off
failure on and off
halt on and off
lostdigits on and off
nomethod on and off
nostring on and off
notready on and off
novalue on and off
syntax on and off
any on and off
done
```

```rexx
signal on user foo name h
call raiser
h: say condition('c')
exit
::routine raiser
  raise user foo
```

```output
USER FOO
```

r[instr.signal.on.error]
The errors of the ON and OFF forms are found when the program is read. A
SIGNAL ON with no condition raises `SYNTAX 20.911` and an OFF `20.912`, and a
USER with no name, or with a string for the name, `20.915`. A word that is not
a condition raises `25.3` after ON and `25.4` after OFF. A NAME with no label
raises `19.3`. Data after the label raises `21.903` on an ON and any NAME on
an OFF raises `21.904`. A word after the condition that is not NAME raises
`25.915`.

```rexx
call try 'signal on'
call try 'signal off'
call try 'signal on user'
call try 'signal on user "foo"'
call try 'signal off user "foo"'
call try 'signal on foo'
call try 'signal off foo'
call try 'signal on error name'
call try 'signal on error name lbl extra'
call try 'signal off error name lbl'
call try 'signal on error lbl'
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
signal on => 20.911
signal off => 20.912
signal on user => 20.915
signal on user "foo" => 20.915
signal off user "foo" => 20.915
signal on foo => 25.3
signal off foo => 25.4
signal on error name => 19.3
signal on error name lbl extra => 21.903
signal off error name lbl => 21.904
signal on error lbl => 25.915
```
