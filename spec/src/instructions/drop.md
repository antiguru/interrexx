---
element: DROP
kind: instruction
syntax: "DROP name | (name) [name | (name)]..."
ansi: "6.3.2.59, 8.3.7"
rexxref: "Keyword Instructions > DROP"
deviations: [70]
---

# DROP

DROP returns variables to the state they had before any assignment: they have
no value, and [a reference to one](r:expr.variables.value) yields the name of
the variable in upper case. Its operands are variable symbols, written as the
names themselves or, in parentheses, as a variable whose value is a list of
names. This page specifies what each kind of name drops, the order in which
the names are processed, the list form, how DROP treats a variable that a
routine or a method has exposed, and the errors. How a variable gets a value
is the business of [the assignment page](assignment.md).

## What is dropped

r[instr.drop.simple]
A simple symbol names one variable, and DROP leaves that variable unset. It is
not an error to drop a variable that has no value, or a name that no earlier
clause has used, and a name that appears more than once is dropped once. The
special variables `RC`, `RESULT` and `SIGL` are ordinary variables here and
can be dropped. [The `SYMBOL` function](r:bif.symbol.result) then reports
`LIT` for a dropped name, and the interpreter sets `RC`, `RESULT` and `SIGL`
again when the event happens that sets them.

```rexx
a = 1; b = 2; rc = 5; result = 6; sigl = 7
say symbol('a') symbol('b') a
drop a b nosuch a
say symbol('a') symbol('b') a b symbol('nosuch')
drop rc result sigl
say rc result sigl symbol('rc')
```

```output
VAR VAR 1
LIT LIT A B LIT
RC RESULT SIGL LIT
```

r[instr.drop.compound]
A compound symbol names the variable that
[its derived name](r:expr.variables.compound) names, and DROP works the name
out when it reaches the symbol, from the values that the tail variables have
at that moment. It drops that variable only: the other variables of the stem
keep their values. A dropped compound variable has its derived name as value,
even when its stem has a [default](r:expr.variables.default).

```rexx
t.1 = 'a'; t.2 = 'b'; t.3 = 'c'
f. = 'dflt'; f.1 = 'one'; f.2 = 'two'
k = 2
drop t.k f.1
say t.1 t.2 t.3
say f.1 f.2 f.3
k = 3
drop t.k
say t.1 t.2 t.3
```

```output
a T.2 c
F.1 two dflt
a T.2 T.3
```

r[instr.drop.stem]
A stem symbol names the stem. DROP makes the symbol refer to a new stem that
has no default and no values, so that each variable of the stem has its
derived name as value, until the stem or the variable is assigned again. The
former stem object is not changed: any other reference to it, such as a
variable that holds the stem as a value, still sees its default and its
values. The simple variable that has the stem's name is another variable and
does not change, as [the separation rule](r:expr.variables.separate) says.

```rexx
s = 'simple'
s. = 'dflt'; s.1 = 'one'; s.x = 'ex'
drop s.
say s. s.1 s.x s.zzz symbol('s.1')
say s
s.2 = 'two'
say s.1 s.2 s.3
```

```output
S. S.1 S.X S.ZZZ LIT
simple
S.1 two S.3
```

```rexx
a.1 = 'x'; a.2 = 'y'
held = a.
drop a.
say held[1] held[2] held~items
say a.1 a.2 a.~items
a.3 = 'z'
say held~items a.~items
```

```output
x y 2
A.1 A.2 0
2 1
```

## Order

r[instr.drop.order]
DROP processes its names from left to right, and a name sees what the names
before it did: when an earlier name drops a variable that the tail of a later
compound symbol uses, the later symbol is worked out from that variable's name
and not from its former value. The same holds for a name that is dropped
twice, which the second mention leaves as it was.

```rexx
i = 1; a.1 = 'x'; a.2 = 'y'
drop i a.i
say symbol('i') symbol('a.1') symbol('a.2') symbol('a.i')
i = 1; a.1 = 'x'; a.2 = 'y'
drop a.i i
say symbol('i') symbol('a.1') symbol('a.2')
```

```output
LIT VAR VAR LIT
LIT LIT VAR
```

## The list form

r[instr.drop.list]
A name in parentheses is not dropped. Its value is converted to a string, as
[the conversion rule](r:conv.string.object) says for an object, and the
string is a list of names separated by
[whitespace](r:conv.string.whitespace). DROP drops each name of the list as
if it had been written in the clause, in upper case, from left to right, and
a name may be a simple symbol, a stem symbol or a compound symbol. The list
may be empty or hold only whitespace, and then nothing is dropped. The
parentheses hold one name and no list holds parentheses itself. Blanks may
stand inside the parentheses, and a name may precede them without a blank. A
list that names the variable which holds it drops that variable with the
others.

```rexx
tab = '09'x
set = 'alpha' || tab || 'Beta' 's.' 't.1'
alpha = 1; beta = 2; gamma = 3
s. = 'dflt'; t.1 = 'one'; t.2 = 'two'
drop gamma (set)
say symbol('alpha') symbol('beta') symbol('gamma') symbol('set')
say s.1 t.1 t.2
empty = ''; blank = '   '
drop (empty) (blank)
say 'nothing dropped' symbol('empty') symbol('blank')
arr = .array~of('m n'); m = 1; n = 2
drop (arr)
say symbol('m') symbol('n')
ref = 'ref r2'; r2 = 1
drop (ref)
say symbol('ref') symbol('r2')
w = 'o'; o = 1; o2 = 2; w2 = 'o2'
drop ( w ) (w2)
say symbol('o') symbol('o2')
p3 = 'o3'; o3 = 1
drop o(p3)
say symbol('o3')
```

```output
LIT LIT LIT VAR
S.1 T.1 two
nothing dropped VAR VAR
LIT LIT
LIT LIT
LIT LIT
LIT
```

r[instr.drop.list.tail]
The tail of a compound symbol in the list is replaced by the value that the
variable has when DROP reaches the name, as it is in a compound symbol that is
written in the clause, so a name that precedes it in the list changes what it
drops.

```rexx
j = 2; a.1 = 'one'; a.2 = 'two'
list = 'a.j'
drop (list)
say symbol('a.1') symbol('a.2')
i = 1; b.1 = 'x'; b.2 = 'y'
list = 'i b.i'
drop (list)
say symbol('i') symbol('b.1') symbol('b.2')
```

```output
VAR LIT
LIT VAR VAR
```

r[instr.drop.list.error]
A name in a list that is not a variable symbol raises a `SYNTAX` error when
DROP reaches the list: `31.2` for a name that starts with a digit, `31.3` for
one that starts with a period, and `20.928` for any other text in a name's
place, such as a parenthesis, a semicolon or a sign, and for a name of more
than 250 characters, counted over the whole compound symbol with its
periods. DROP checks the whole list before it drops any name of it, so a name
that precedes the faulty one in the list is not dropped.

```rexx
c = 1; d = 2
call try 'l = "c 3"; drop (l)'
call try 'l = "c .x"; drop (l)'
call try 'l = "c (d)"; drop (l)'
call try 'l = "c d;e"; drop (l)'
call try 'l = "c d-e"; drop (l)'
call try 'l = 5; drop (l)'
say symbol('c') symbol('d')
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
31.2 <= l = "c 3"; drop (l)
31.3 <= l = "c .x"; drop (l)
20.928 <= l = "c (d)"; drop (l)
20.928 <= l = "c d;e"; drop (l)
20.928 <= l = "c d-e"; drop (l)
31.2 <= l = 5; drop (l)
VAR VAR
```

```rexx
signal on syntax name s
l = copies('a', 250)
drop (l)
say 'ok 250'
l = copies('a', 251)
drop (l)
say 'ok 251'
exit
s: say condition('o')~code length(l)
```

```output defect=#100
ok 250
20.928 251
```

## Exposed variables

r[instr.drop.exposed]
A variable that a PROCEDURE EXPOSE or an EXPOSE has made visible is dropped in
the scope that owns it: DROP of the name unsets the variable of the exposing
scope, so the caller of the routine or the other methods of the object find it
unset. A name that is not exposed names a variable of the routine or the
method, and DROP leaves any variable of the caller with that name as it was.
The same holds for the names in a list.

```rexx
x = 'outer x'; y = 'outer y'; v = 'outer v'
call drops
say symbol('x') symbol('y') symbol('v')
o = .box~new
o~drop
say o~peek
exit
drops: procedure expose x v
  y = 'local y'
  drop x y
  list = 'v'
  drop (list)
return
::class box
::method init
  expose n
  n = 5
::method drop
  expose n
  drop n
  n = 'back'
::method peek
  expose n
  return n
```

```output
LIT VAR LIT
back
```

## Errors

r[instr.drop.syntax]
A DROP that has no operand, an operand that is not a variable symbol or a name
in parentheses, or a malformed parenthesis, raises a `SYNTAX` error that [a
program file raises when it is read](r:lex.index.read-first). The codes are
`20.901` for a missing operand, for a string, a message, a comma, a sign or an
`=` where a symbol should be, and for a `)` with no `(`; `20.906` for an empty
or doubled parenthesis; `46.1` for two names inside one parenthesis; `46.901`
for a parenthesis that is not closed; and `31.2` or `31.3` for a name that
starts with a digit or with a period, written plainly or in parentheses. The
text of INTERPRET raises them as [it does for any error found in
reading](r:instr.interpret.whole), which the last example shows.

```rexx
say 'before'
drop
```

```output rc=236
```

```rexx
say 'before'
drop 3
```

```output rc=225
```

```rexx
say 'before'
drop (a b)
```

```output rc=210
```

```rexx
call try 'say "first"; drop'
call try 'say "first"; drop "a"'
call try 'say "first"; drop a,b'
call try 'say "first"; drop a +b'
call try 'say "first"; drop a = 1'
call try 'say "first"; drop a)'
call try 'say "first"; drop a~b'
call try 'say "first"; drop ()'
call try 'say "first"; drop ((a))'
call try 'say "first"; drop (a b)'
call try 'say "first"; drop (a'
call try 'say "first"; drop 3'
call try 'say "first"; drop (3)'
call try 'say "first"; drop .a'
call try 'say "first"; drop a.b. c..d e.1'
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
20.901 <= say "first"; drop
20.901 <= say "first"; drop "a"
20.901 <= say "first"; drop a,b
20.901 <= say "first"; drop a +b
20.901 <= say "first"; drop a = 1
20.901 <= say "first"; drop a)
20.901 <= say "first"; drop a~b
20.906 <= say "first"; drop ()
20.906 <= say "first"; drop ((a))
46.1 <= say "first"; drop (a b)
46.901 <= say "first"; drop (a
31.2 <= say "first"; drop 3
31.2 <= say "first"; drop (3)
31.3 <= say "first"; drop .a
first
ok say "first"; drop a.b. c..d e.1
```

## Deviations

r[instr.drop.deviation.70]
When a clause holds both parenthesised lists and other names, DROP processes
every name and every list from left to right, so a later name sees what an
earlier list dropped and an earlier name is dropped before a later list is
read. ooRexx processes the parenthesised lists first, in reverse order, and
then the other names, so a list is read before the names that precede it.

```rexx
i = 1; a.1 = 'x'; a.2 = 'y'; list = 'i'
drop a.i (list)
say symbol('i') symbol('a.1') symbol('a.2')
```

```output
LIT LIT VAR
```
