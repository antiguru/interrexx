---
element: PROCEDURE
kind: instruction
syntax: "PROCEDURE [EXPOSE [name | (name)]...]"
ansi: "6.3.2.76, 8.3.18"
rexxref: "Keyword Instructions > PROCEDURE"
deviations: [70]
---

# PROCEDURE

PROCEDURE gives an internal routine variables of its own. Without it a
routine works on the variables of its caller. With it the routine starts with
none, the variables it sets are dropped when it returns, and only the names
that EXPOSE lists stay shared with the caller. This page specifies the
routine's own variables, the one place where PROCEDURE may stand, the forms
of the names, the list form `(name)`, and the errors. Stems and compound
names are those of [the variables chapter](r:expr.variables.stem), and
[DROP](r:instr.drop.exposed) says what dropping an exposed name does.

## Variables of its own

r[instr.procedure.scope]
After PROCEDURE the routine sees none of its caller's variables: a name that
the caller has set is unset in the routine, and a variable that the routine
sets belongs to the routine and is gone when it returns. The arguments stay
available through `ARG`. A routine that does not start with PROCEDURE, and a
routine it calls without PROCEDURE, work on the variables of the routine that
called them.

```rexx
a = 'caller a'
call hidden
say a symbol('b')
call shared
say a symbol('b')
call bare
say a symbol('b') symbol('c')
exit
hidden: procedure
  say symbol('a') arg(1)
  a = 'hidden a'
  b = 'hidden b'
  return
shared:
  a = 'shared a'
  b = 'shared b'
  return
bare: procedure
  call setter
  say symbol('c')
  return
setter:
  c = 'set in setter'
  return
```

```output
LIT 
caller a LIT
shared a VAR
VAR
shared a VAR LIT
```

r[instr.procedure.first]
PROCEDURE has to be the first instruction that runs after a CALL or a
function call has reached the label of an internal routine. Labels and null
clauses, which are comments and semicolons, may stand between that label and
the PROCEDURE. Any other clause before it, a NOP included, makes it a
misplaced one, and so does a PROCEDURE inside a DO or IF, in the text of an
INTERPRET, in the main program, in a method or in a routine of a `::ROUTINE`
directive, after a SIGNAL to its label, and a second PROCEDURE in the same
routine. A misplaced PROCEDURE raises `SYNTAX 17.1` when it runs. In the
example with a loop, the handler's RETURN leaves the failing routine, as [the
rule on a jump out of a routine](r:instr.signal.routine) says.

```rexx
call ok1
call ok2
say 'accepted'
procedure
say 'not reached'
exit
ok1: /* comment */ ; ok1b:
  procedure
  return
ok2:
  procedure expose a
  return
```

```output rc=239
accepted
```

```rexx
signal on syntax name e
call late
exit
e: say 'late' condition('o')~code; exit
late:
  nop
  procedure
```

```output
late 17.1
```

```rexx
signal on syntax name e
call twice
exit
e: say 'twice' condition('o')~code; exit
twice: procedure
  procedure expose x
```

```output
twice 17.1
```

```rexx
signal on syntax name e
call nested
exit
e: say 'nested' condition('o')~code; exit
nested:
  do 1
    procedure
  end
```

```output
nested 17.1
```

```rexx
signal on syntax name e
do n over .array~of('VIAIF', 'VIAINTERPRET', 'VIASIGNAL')
  call charout , n '=> '
  call (n)
end
exit
e: say condition('o')~code
  signal on syntax name e
  return
VIAIF: if 1 then procedure
  return
VIAINTERPRET: interpret 'procedure'
  return
VIASIGNAL: signal target
target: procedure
  return
```

```output
VIAIF => 17.1
VIAINTERPRET => 17.1
VIASIGNAL => 17.1
```

```rexx
signal on syntax name e
o = .box~new
o~go
exit
e: say 'method' condition('o')~code; exit
::class box
::method go
  procedure
```

```output
method 17.1
```

## Exposing names

r[instr.procedure.expose]
`PROCEDURE EXPOSE` lists names that stay shared with the caller. An exposed
variable is the caller's variable: the routine reads its value, and an
assignment in the routine changes the caller's variable. A name the caller has
not set stays unset in the routine, and setting it in the routine sets it in
the caller. A name that is not listed is the routine's own. The names are
variable symbols separated by blanks, and they are taken in upper case.

```rexx
a = 1; b = 2; c = 3
call r1
say a b c symbol('d') symbol('e')
exit
r1: procedure expose a d
  say symbol('a') symbol('b') symbol('c') symbol('d')
  a = a + 10
  d = 'new d'
  b = 'local b'
  e = 'local e'
  return
```

```output
VAR LIT LIT LIT
11 2 3 VAR LIT
```

r[instr.procedure.stem]
A stem symbol in the list, which is a name ending in a period, exposes the
whole stem: every compound variable of it, those the caller has set and those
that either side sets later, and the stem's default value.

```rexx
a.1 = 'one'; a.0 = 'zero'
call s2
say a.1 a.0 a.3 a.9
exit
s2: procedure expose a.
  say 'in' a.1 a.3
  a.3 = 'three'
  a. = 'reset'
  return
```

```output
in one A.3
reset reset reset reset
```

r[instr.procedure.compound]
A compound symbol in the list exposes the one variable that it names. Its
tail is replaced by the value that the variable has in the caller when
PROCEDURE runs, and a variable in the tail is not exposed by that. A later
change of the tail variable in the routine does not move the exposed name,
and a compound name that now has another tail is the routine's own variable.

```rexx
i = 1; a.1 = 'one'; a.2 = 'two'
call s1
say i a.1 a.2
exit
s1: procedure expose a.i i
  say 'in' a.i
  i = 2
  say a.i
  a.i = 'local'
  return
```

```output
in one
A.2
2 one two
```

r[instr.procedure.chain]
A routine that exposes a name of its caller shares the variable with the
caller's caller as well, if the caller shared it, so a chain of routines that
each expose the name reaches the original variable. A name that the caller did
not expose is the caller's own variable, and a change of it does not reach
the routine above.

```rexx
x = 'main x'; y = 'main y'
call outer
say x y symbol('z')
exit
outer: procedure expose x
  y = 'outer y'
  z = 'outer z'
  call inner
  say 'outer' x y z
  return
inner: procedure expose x y
  x = x'+'
  y = y'+'
  return
```

```output
outer main x+ outer y+ outer z
main x+ main y LIT
```

## The list form

r[instr.procedure.list]
A name in parentheses is not exposed as it stands. The value of the variable,
as the caller has it, is converted to a string as [the DROP list
rule](r:instr.drop.list) describes, split at
[whitespace](r:conv.string.whitespace), and each word is exposed as if it had
been written in the clause, as a simple, stem or compound symbol. A list
that is empty or all whitespace exposes nothing. The variable that holds the
list is exposed too, so the routine sees it and an assignment to it reaches
the caller. Blanks may stand inside the parentheses, and one list may follow
another without a blank.

```rexx
x = 'X'; y = 'Y'; s.1 = 's1'; i = 1
names = 'x s.'
empty = ''
arr = .array~of('y')
call p1
say x y s.2 names
call p2
say names
exit
p1: procedure expose (names) (empty)(arr)
  say symbol('x') symbol('y') symbol('names') symbol('i') s.1
  x = 'x set'
  s.2 = 's2 set'
  names = 'new list'
  return
p2: procedure expose ( names )
  say names
  return
```

```output
VAR VAR VAR LIT s1
x set Y s2 set new list
new list
new list
```

r[instr.procedure.list.error]
A word in a list that is not a variable symbol raises a `SYNTAX` error when
the PROCEDURE runs: `31.2` for a word that starts with a digit, `31.3` for
one that starts with a period, and `20.928` for any other text in a name's
place, such as a sign, a quote or a parenthesis. The routine has no variable
exposed then. In the example the handler runs in the routine, as [the rule on
a jump out of a routine](r:instr.signal.routine) says, so its RETURN leaves
the routine and the loop goes on.

```rexx
signal on syntax name e
do l over .array~of('c 3', 'c .x', 'c (d)', 'c "d"', 'c d+e', ' ', 'c')
  call charout , l '=> '
  call p
end
exit
e: say condition('o')~code
  signal on syntax name e
  return
p: procedure expose (l)
  say 'ok'
  return
```

```output
c 3 => 31.2
c .x => 31.3
c (d) => 20.928
c "d" => 20.928
c d+e => 20.928
  => ok
c => ok
```

## Errors

r[instr.procedure.syntax]
A fault in the form of the clause is raised when the program is read, so no
clause runs: `20.902` for an EXPOSE with no name and for text where a name
should be, such as a string, a comma or an `=`; `25.17` for a word
after PROCEDURE that is not EXPOSE; `20.906` for an empty or doubled
parenthesis; `46.1` for two names in one parenthesis; `46.901` for a
parenthesis that is not closed; and `31.2` or `31.3` for a name that starts
with a digit or a period. The text of an INTERPRET raises the same errors
when the INTERPRET runs, before the PROCEDURE can be found misplaced.

```rexx
say 'before'
procedure expose
```

```output rc=236
```

```rexx
say 'before'
procedure foo
```

```output rc=231
```

```rexx
call try 'procedure expose'
call try 'procedure expose "a"'
call try 'procedure expose a, b'
call try 'procedure expose a = 1'
call try 'procedure foo'
call try 'procedure 1'
call try 'procedure expose ()'
call try 'procedure expose (a b)'
call try 'procedure expose (a'
call try 'procedure expose 3a'
call try 'procedure expose .a'
call try 'procedure expose a.3'
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
procedure expose => 20.902
procedure expose "a" => 20.902
procedure expose a, b => 20.902
procedure expose a = 1 => 20.902
procedure foo => 25.17
procedure 1 => 25.17
procedure expose () => 20.906
procedure expose (a b) => 46.1
procedure expose (a => 46.901
procedure expose 3a => 31.2
procedure expose .a => 31.3
procedure expose a.3 => 17.1
```

## Deviations

r[instr.procedure.deviation.70]
When a clause holds both parenthesised lists and other names, PROCEDURE
EXPOSE processes every name and every list from left to right, so a variable
that an earlier name has exposed is already the caller's when a later list's
own variable is read. ooRexx processes the parenthesised lists first, in
reverse order, and then the other names, so the variable of a list is read
before any name of the clause is exposed. In the example, ooRexx reads `a.i`
while `i` is still the routine's own unset variable, and so exposes another
name than `x`.

```rexx
i = 1; a.1 = 'x'; x = 'X'
call f
exit
f: procedure expose i (a.i)
  say symbol('x')
  return
```

```output
VAR
```
