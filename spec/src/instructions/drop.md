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
A stem symbol names the stem. DROP removes its default and the value of every
compound variable that it had, so that each variable of the stem has its
derived name as value, until the stem or the variable is assigned again.
The simple variable that has the stem's name is another variable and does not
change, as [the separation rule](r:expr.variables.separate) says.

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
A name in parentheses is not dropped. Its value is taken as a list of names
separated by blanks, and DROP drops each of them as if it had been written in
the clause, in upper case, from left to right. The names may be simple
symbols, stem symbols or compound symbols with a tail that is a constant, as
written. The list may be empty or hold only blanks, and then nothing is
dropped. The parentheses hold one name and no list holds parentheses itself.

```rexx
names = 'c d Eee s. t.1'
c = 1; d = 2; eee = 3; f = 4
s. = 'dflt'; t.1 = 'one'; t.2 = 'two'
drop (names) f
say symbol('c') symbol('d') symbol('eee') symbol('f') symbol('names')
say s.1 t.1 t.2
empty = ''; blank = '   '
drop (empty) (blank)
say 'nothing dropped' symbol('empty') symbol('blank')
list = 'x y'; x = 1; y = 2
drop x (list) y
say symbol('x') symbol('y') symbol('list')
```

```output
LIT LIT LIT LIT VAR
S.1 T.1 two
nothing dropped VAR VAR
LIT LIT VAR
```

r[instr.drop.list.error]
A name in a list that is not a variable symbol raises a `SYNTAX` error when
DROP reaches the list: `31.2` for a name that starts with a digit, `31.3` for
one that starts with a period, and `20.928` for any other text in a name's
place, such as a parenthesis, a semicolon or a sign. DROP checks the whole
list before it drops any name of it, so a name that precedes the faulty one in
the list is not dropped.

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

## Exposed variables

r[instr.drop.exposed]
A variable that a PROCEDURE EXPOSE or an EXPOSE has made visible is dropped in
the scope that owns it: DROP of the name unsets the original variable, so the
caller of the routine or the other methods of the object find it unset. A name
that is not exposed names a variable of the routine or the method, and DROP
leaves any variable of the caller with that name as it was. The same holds for
the names in a list.

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
in parentheses, or a malformed parenthesis, raises a `SYNTAX` error when the
program is read, so that no clause of the program runs and the exit code is
256 minus the error number. The codes are `20.901` for a missing operand, for
a string, a message, a comma, a sign or an `=` where a symbol should be, and
for a `)` with no `(`; `20.906` for an empty or doubled parenthesis; `46.1`
for two names inside one parenthesis; `46.901` for a parenthesis that is not
closed; and `31.2` or `31.3` for a name that starts with a digit or with a
period, written plainly or in parentheses. In the text of INTERPRET the same
errors are raised when the text is translated, before any clause of it runs.

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
