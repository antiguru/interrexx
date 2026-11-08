---
element: RAISE
kind: instruction
syntax: "RAISE condition [DESCRIPTION expr] [ADDITIONAL expr | ARRAY (expr, ...)] [RETURN [expr] | EXIT [expr]]"
rexxref: "Keyword Instructions > RAISE"
deviations: [65]
---

# RAISE

RAISE raises a condition on purpose, and it also ends the code that runs it.
The condition is raised in the code that called the code that ended, so the
instruction is how a routine or a method reports an event to its caller,
whose traps answer it as they answer any condition. SYNTAX and the main
program are the exceptions, and so is the text of an INTERPRET with RETURN;
this page specifies them. The instruction names the condition, carries the
values that the condition object holds, and chooses whether the code ends as
RETURN or as EXIT does, which also decides who is the caller.

This page specifies the form of the clause, what each option sets, where the
condition is raised for each kind of code, what happens when no trap answers,
and the PROPAGATE form that raises a trapped condition again. What a trap
does, and what a program reads from a condition, belong to [the conditions
chapter](../conditions/index.md), and the rules here link the ones they use.

## The clause

r[instr.raise.form]
RAISE is followed by a condition word: ERROR, FAILURE, HALT, LOSTDIGITS,
NOMETHOD, NOSTRING, NOTREADY, NOVALUE, PROPAGATE, SYNTAX or USER, in any case.
ERROR and FAILURE then take a code, SYNTAX takes a number and USER takes a
name, and the other words take nothing. The options come after that, in any
order, each at most once: DESCRIPTION and an expression, ADDITIONAL and an
expression or ARRAY and a parenthesized list of expressions (the two are one
option, so a clause has at most one of them), and RETURN or EXIT with an
optional expression (again one option). The code, the number and the
expression of an option are each one term: a literal string, a constant
symbol, an environment symbol or an expression in parentheses. A variable
symbol is not a term here, so its value is written in parentheses. A RETURN or
EXIT that has no expression ends the clause: any option after it is read as
its expression. ANY is not a condition word, because [the names rule](r:cond.index.names) says
that nothing raises it.

```rexx
call on user alpha name h
call on user beta name h
call on error name h
call on notready name h
signal on syntax name s
do i = 1 to 5
  call ('R'i)
end
exit
h: say 'h' condition('c') '['condition('d')']',
     condition('o')~hasindex('ADDITIONAL')
  return
s: say 'syntax' condition('o')~code
  exit
::routine r1
  RAISE USER Alpha
::routine r2
  raise user beta additional ('x') description 'two words'
::routine r3
  raise error 5 description ('a' || 'b') additional (.nil)
::routine r4
  raise notready description 'late' array (1, 2) return
::routine r5
  raise user alpha return 'v' description 'rv'
```

```output
h USER ALPHA [] 0
h USER BETA [two words] 1
h ERROR [ab] 1
h NOTREADY [late] 1
h USER ALPHA [rv] 0
```

r[instr.raise.errors]
A RAISE clause that breaks the form is an error found while the program is
read. Nothing is raised: the clause is not part of a program that runs, as
[the reading rule](r:cond.index.read) says, and the same error is an ordinary
SYNTAX condition in the text of an INTERPRET. The error is `20.914` when
nothing follows RAISE, `25.906` when the word after it is not a condition
word, `20.915` when USER has no symbol after it, and `35.1` when a code or
number is missing or is not a term, or when a RETURN or EXIT without an
expression is followed by another option, which it then reads as its
expression. A missing expression after DESCRIPTION or ADDITIONAL is `35.922`
or `35.923`, and an ARRAY without its opening parenthesis is `35.924`. An
ARRAY list whose parenthesis is not closed is `36.901`. The options that
appear twice are `25.908` for DESCRIPTION, `25.909` for ADDITIONAL or ARRAY
together, whichever two they are, and `25.911` for RETURN or EXIT together.
Any other word after the options is `25.907`.

```rexx
say 'before'
raise user quux return description
```

```output rc=221
```

```rexx
clause.1 = 'raise'
clause.2 = 'raise nosuchkeyword'
clause.3 = 'raise any'
clause.4 = 'raise user'
clause.5 = "raise user 'quoted'"
clause.6 = 'raise error'
clause.7 = 'raise syntax -3'
clause.8 = 'raise error somevar'
clause.9 = 'raise user q description'
clause.10 = 'raise user q additional'
clause.11 = 'raise user q array 1, 2'
clause.12 = 'raise user q array (1, 2'
clause.13 = "raise user q description 'a' description 'b'"
clause.14 = 'raise user q additional (1) array (2)'
clause.15 = 'raise user q return 1 exit 2'
clause.16 = "raise user q return description 'x'"
clause.17 = 'raise user q 5'
clause.18 = 'raise user q return 1 2'
do i = 1 to 18
  call try clause.i
end
exit
try: procedure
  signal on syntax name e
  interpret arg(1)
  say 'no error:' arg(1)
  return
e: say right(condition('o')~code, 6) '<-' arg(1)
  return
```

```output
20.914 <- raise
25.906 <- raise nosuchkeyword
25.906 <- raise any
20.915 <- raise user
20.915 <- raise user 'quoted'
  35.1 <- raise error
  35.1 <- raise syntax -3
  35.1 <- raise error somevar
35.922 <- raise user q description
35.923 <- raise user q additional
35.924 <- raise user q array 1, 2
36.901 <- raise user q array (1, 2
25.908 <- raise user q description 'a' description 'b'
25.909 <- raise user q additional (1) array (2)
25.911 <- raise user q return 1 exit 2
  35.1 <- raise user q return description 'x'
25.907 <- raise user q 5
25.907 <- raise user q return 1 2
```

## What each condition takes

r[instr.raise.condition]
Each condition word raises the condition of that name, and the clause sets
what the condition word takes. ERROR and FAILURE take a code, which is any
string: the text of the literal, of the constant symbol in upper case, or of
the value of the parenthesized expression, with no check that it is a number.
The code is the value that `RC` has in the code that answers the condition, as
[the rule for `RC`](r:cond.information.rc) says. The `RC` entry of the
condition object holds the code when it is a string. When the parenthesized
expression is an object that is not a string, the variable `RC` has its string
value, and what the entry holds is not specified. The user conditions take a name, which [the
name rule](r:cond.user.name) specifies. HALT, LOSTDIGITS, NOMETHOD, NOSTRING,
NOTREADY and NOVALUE take nothing, and the condition has the entries that [the
entries rule](r:cond.information.object) lists for a condition that an
instruction raised.

```rexx
names = 'HALT LOSTDIGITS NOMETHOD NOSTRING NOTREADY NOVALUE'
do w over names~makearray(' ')
  call go w
end
call go 'ERROR 12'
call go 'FAILURE 13'
call go 'USER plain'
exit
go: procedure
  signal on any name a
  parse arg kind code
  call (kind) code
  say kind 'ignored'
  return
a: o = condition('o')
  say arg(1) '=>' condition('c') '['condition('d')']' o~hasindex('RC')
  return
::routine halt; raise halt
::routine lostdigits; raise lostdigits
::routine nomethod; raise nomethod additional (.nil) description 'msg'
::routine nostring; raise nostring
::routine notready; raise notready
::routine novalue; raise novalue
::routine error; raise error (arg(1))
::routine failure; raise failure (arg(1))
::routine user; raise user plain
```

```output
HALT => HALT [] 0
LOSTDIGITS => LOSTDIGITS [] 0
NOMETHOD => NOMETHOD [msg] 0
NOSTRING => NOSTRING [] 0
NOTREADY => NOTREADY [] 0
NOVALUE => NOVALUE [] 0
ERROR 12 => ERROR [] 1
FAILURE 13 => FAILURE [] 1
USER plain => USER PLAIN [] 0
```

```rexx
call on error name e
call on failure name f
rc = 'unset'
call r1
say 'after r1' rc
call r2
say 'after r2' rc
call r3
say 'after r3' rc
call r4
say 'after r4' rc
call r5
say 'after r5 [' || rc || ']'
exit
e: say 'e' rc condition('o')~rc condition('c'); return
f: say 'f' rc condition('o')~rc condition('c'); return
::routine r1; raise error 3
::routine r2; raise failure (2 + 5) return
::routine r3; raise error "not a number"
::routine r4; raise error 1.5e2
::routine r5; raise error ''
```

```output
e 3 3 ERROR
after r1 3
f 7 7 FAILURE
after r2 7
e not a number not a number ERROR
after r3 not a number
e 1.5E2 1.5E2 ERROR
after r4 1.5E2
e   ERROR
after r5 []
```

r[instr.raise.number]
The number of RAISE SYNTAX is a major error number from 1 to 99, which may be
followed by a point and a subcode from 0 to 999. It is read from the text of
the value, and the digits after the point are one whole number, so `40.3`,
`40.003` and `040.03` are the error `40.3`, and `40.030` is the subcode 30. A
number without a point is the error of that number with the subcode 0. The
`CODE` entry of the condition holds the number as `m.n`, and the `RC` entry
holds the major number. A number of digits, or of digits, a point and digits,
that no error has raises `SYNTAX 98.941`. The null string, a minus sign, a
letter, a second point, a major number outside 1 to 99 and a subcode above 999
raise `SYNTAX 33.904`. The text of any other form, such as a number with a
plus sign, with blanks in it or with an exponent, is not specified. Both
errors are raised in the code that holds the RAISE, as a SYNTAX condition is
raised by [the SYNTAX rule](r:instr.raise.here).

```rexx
n.1 = '40'
n.2 = '40.3'
n.3 = '40.003'
n.4 = '040.03'
n.5 = '41.1'
n.6 = '5'
n.7 = '40.030'
n.8 = '40.99'
n.9 = '1.1'
n.10 = '2'
n.11 = '0'
n.12 = '100'
n.13 = '40.1000'
n.14 = '40.3.5'
n.15 = '4.5'
n.16 = '-3'
n.17 = 'abc'
n.18 = ''
do i = 1 to 18
  call go n.i
end
exit
go: procedure
  signal on syntax name e
  call raiser arg(1)
  return
e: o = condition('o')
  say right(arg(1), 7) '=>' left(o~code, 7) o~rc
  return
::routine raiser
  raise syntax (arg(1))
```

```output
     40 => 40.0    40
   40.3 => 40.3    40
 40.003 => 40.3    40
 040.03 => 40.3    40
   41.1 => 41.1    41
      5 => 5.0     5
 40.030 => 98.941  98
  40.99 => 98.941  98
    1.1 => 98.941  98
      2 => 98.941  98
      0 => 33.904  33
    100 => 33.904  33
40.1000 => 33.904  33
 40.3.5 => 33.904  33
    4.5 => 98.941  98
     -3 => 33.904  33
    abc => 33.904  33
        => 33.904  33
```

r[instr.raise.syntax]
A SYNTAX condition that RAISE raised holds, besides `CODE` and `RC`, the
description that DESCRIPTION gave, which is the null string without it. Its
`ADDITIONAL` entry is always an array, whatever the clause gave: the array is
empty when the clause has neither ADDITIONAL nor ARRAY, it is the array of
ARRAY's values, and it is the value of ADDITIONAL made an array, where an
array stays as it is, a string becomes an array of one item and a list becomes
an array of its items. The null object `.nil` and an instance of the class
Object cannot be made an array, and an ADDITIONAL with such a value raises
`SYNTAX 98.939` in the raising code, as [the SYNTAX rule](r:instr.raise.here)
says; whether an object of any other class is converted is not specified. The
values of the array are what the message of the error names, and the text of
the message is not specified. Every other
condition has the entry only when the clause gave one, as [the rule for the
options](r:instr.raise.additional) says.

```rexx
do i = 1 to 5
  call go 'R'i
end
exit
go: procedure
  signal on syntax name s
  call (arg(1))
s: o = condition('o')
  a = o~additional
  say arg(1) o~code o~rc '['o~description']' a~class a~items
  if a~items > 0 then say '   first' a~at(1)
  return
::routine r1; raise syntax 40.3
::routine r2; raise syntax 40.3 description 'why' additional ('one')
::routine r3; raise syntax 40.3 additional (.array~of('p', 'q'))
::routine r4; raise syntax 40.3 array ('p', 'q', 'r')
::routine r5; raise syntax 40.3 additional (.list~of('l', 'm'))
```

```output
R1 40.3 40 [] The Array class 0
R2 40.3 40 [why] The Array class 1
   first one
R3 40.3 40 [] The Array class 2
   first p
R4 40.3 40 [] The Array class 3
   first p
R5 40.3 40 [] The Array class 2
   first l
```

```rexx
do i = 1 to 2
  call ('R'i)
end
exit
::routine r1
  signal on syntax name own
  raise syntax 40.3 additional (.nil)
  say 'not reached'
own: say 'caught' condition('o')~code
  return
::routine r2
  signal on syntax name own
  raise syntax 40.3 additional (.object~new)
  say 'not reached'
own: say 'caught' condition('o')~code
  return
```

```output
caught 98.939
caught 98.939
```

## The options

r[instr.raise.description]
DESCRIPTION gives the description of the condition. The `DESCRIPTION` entry of
the condition object holds the value of the expression as it is, whatever
object that is, and [`CONDITION('D')`](r:bif.condition.description) answers
the string value of it. Without DESCRIPTION the description is the null
string.

```rexx
call on user foo name h
x = 'value of x'
do i = 1 to 5
  call ('R'i)
end
exit
h: o = condition('o')
  say 'description [' || condition('d') || ']' o~hasindex('ADDITIONAL')
  return
::routine r1; raise user foo
::routine r2; raise user foo description 'plain text'
::routine r3; x = 'local x'; raise user foo description (x '!')
::routine r4; raise user foo description .nil
::routine r5; raise user foo description 17 return
```

```output
description [] 0
description [plain text] 0
description [local x !] 0
description [The NIL object] 0
description [17] 0
```

```rexx
call on user foo name h
call r1
call r2
exit
h: o = condition('o')
  say 'entry' o~description~class '/ string [' || condition('d') || ']'
  return
::routine r1; raise user foo description 'text' return
::routine r2; raise user foo description (.object~new) return
```

```output
entry The String class / string [text]
entry The Object class / string [an Object]
```

r[instr.raise.additional]
ADDITIONAL gives the object of the `ADDITIONAL` entry, any object, and ARRAY
gives an array of the values of its list, whose items may be omitted, and
which may be empty. A condition that neither option accompanies has no
`ADDITIONAL` entry, except SYNTAX, which always has one. The entry is what
[`CONDITION('A')`](r:bif.condition.additional) answers. An ADDITIONAL that
holds the null object `.nil` is still present. An ADDITIONAL belongs to the
clause that gave it: a RAISE that no trap answers leaves nothing for the
condition that is raised next.

```rexx
call on user foo name h
do i = 1 to 7
  call ('R'i)
end
exit
h: o = condition('o')
  if \o~hasindex('ADDITIONAL') then do
    say 'no ADDITIONAL'
    return
  end
  a = o~additional
  if a~isA(.array) then say 'array of' a~items
  else say 'value' a
  return
::routine r1; raise user foo
::routine r2; raise user foo additional 'a literal'
::routine r3; raise user foo additional (3 + 4)
::routine r4; raise user foo additional .nil
::routine r5; raise user foo additional (.array~of('p', 'q'))
::routine r6; raise user foo array ('p', 'q', 'r')
::routine r7; raise user foo array ()
```

```output
no ADDITIONAL
value a literal
value 7
value The NIL object
array of 2
array of 3
array of 0
```

```rexx
call on user b name h
call r1
call r2
exit
h: o = condition('o')
  say condition('c') o~hasindex('ADDITIONAL')
  return
::routine r1; raise user a description 'da' additional ('x')
::routine r2; raise user b
```

```output defect=#114
USER B 0
```

r[instr.raise.order]
The expressions of the clause are evaluated before anything is raised, in the
code that holds the RAISE and with its variables, in a fixed order that does
not depend on the order of the options: the code or the number first, then the
DESCRIPTION expression, then the ADDITIONAL expression or the ARRAY list from
left to right, and last the RETURN or EXIT expression. A name of a user
condition is not evaluated. When the number of a RAISE SYNTAX is not valid, the
error is raised as [the number rule](r:instr.raise.number) says, and whether
the expressions after the number are evaluated first is not specified.

```rexx
call on error name h
call on user foo name h
call one
call two
exit
h: say 'handler' condition('c'); return
::routine one
  raise error (t('code')) return (t('value')) additional (t('extra')),
        description (t('text'))
::routine two
  raise user foo array (t('first'), t('second')) exit (t('value'))
::routine t
  say 'evaluate' arg(1)
  return arg(1)
```

```output
evaluate code
evaluate text
evaluate extra
evaluate value
handler ERROR
evaluate first
evaluate second
evaluate value
handler USER FOO
```

## Where the condition is raised

r[instr.raise.leave]
RAISE ends the code that holds it, and the option says how: RETURN ends it as
[RETURN](r:instr.return.routine) does and EXIT ends it as
[EXIT](r:instr.exit.program) does, and EXIT is what a clause without either
option does. A method and a routine of a `::ROUTINE` directive end alone,
either way, as [RETURN](r:instr.return.unit) and [EXIT](r:instr.exit.unit)
say. The value of the option is handed back as that instruction hands it back:
it is `RESULT` after a CALL, the value of a function call, and the value of a
message. A clause without a value hands back none, so a function call raises
`SYNTAX 44.1`, [as RETURN does](r:expr.terms.call.result), and a message used
as a term raises `SYNTAX 91.999`, as [the rule for a
unit](r:instr.return.unit) says. The condition is raised after the value is
computed, and a CALL trap runs its handler once the clause of the caller has
finished, as [the CALL trap rule](r:cond.traps.call) says.

```rexx
call on user foo name h
say 'A'
call ru
say 'B' result
call rx
say 'C' result
x = ru()
say 'D' x
x = rx()
say 'E' x
o = .box~new
say 'F' o~m
o~m
say 'G' result
exit
h: say 'h' condition('c') sigl; return
::routine ru
  say 'in ru'
  raise user foo return 'from ru'
  say 'not reached'
::routine rx
  raise user foo exit 'from rx'
::class box
::method m
  raise user foo return 'from m'
```

```output
A
in ru
h USER FOO 3
B from ru
h USER FOO 5
C from rx
in ru
h USER FOO 7
D from ru
h USER FOO 9
E from rx
F from m
h USER FOO 12
h USER FOO 13
G from m
```

```rexx
call on user foo name h
call ru
say 'after' result
exit
h: say 'h'; return
::routine ru
  raise user foo
```

```output
h
after RESULT
```

```rexx
signal on syntax name s
x = nothing()
say 'not reached'
exit
s: say 'function' condition('o')~code
  signal on syntax name s2
  x = .box~new~m
  say 'not reached'
s2: say 'message' condition('o')~code
::routine nothing
  raise user foo return
::class box
::method m
  raise user foo return
```

```output
function 44.1
message 91.999
```

r[instr.raise.caller]
A condition other than SYNTAX that a RAISE with RETURN raises is raised in the
code that called the code that holds the RAISE, at the clause that made the
call, which is a CALL, a function call or a message, whether the code called
is a routine, an internal routine or a method. A RAISE with EXIT ends more
than the code that holds it, and [the exit rule](r:instr.raise.exit) says who
the caller is then. The traps of that caller answer it, as [the choice
rule](r:cond.traps.choice) says, so a SIGNAL trap abandons the clause, and an
assignment in it is not made. The raising code's own traps take no part in the
answer, even when they are on, and a caller that has no trap for the condition
does not hand it to its own caller: only [a SYNTAX
condition](r:cond.information.propagate.only) travels up. `SIGL` in the code
that answers is the line of the clause in the caller, and the `POSITION` entry
is the line of the RAISE clause.

```rexx
call on user foo name main_h
say 'start'
call tt
say 'after tt'
call il
say 'after il'
exit
il:
  say 'in il'
  raise user foo return
  say 'not reached'
main_h: say 'main' condition('c') 'sigl' sigl,
    'position' condition('o')~position
  return
::routine tt
  call on user foo name mine
  raise user foo
mine: say 'mine'
  return
```

```output
start
main USER FOO sigl 3 position 17
after tt
in il
main USER FOO sigl 5 position 10
after il
```

```rexx
signal on user foo name h
x = 'old'
x = f()
say 'not reached'
exit
h: say 'h' x
::routine f
  raise user foo return 'new'
```

```output
h old
```

```rexx
call on user foo name h
.box~new~outer
say 'after outer'
exit
h: say 'h at' sigl; return
::class box
::method outer
  call inner
  say 'outer goes on'
  return
inner: raise user foo return
```

```output
outer goes on
after outer
```

r[instr.raise.exit]
A RAISE with EXIT, which is what a clause with neither option does, ends the
whole unit that holds it, as [EXIT](r:instr.exit.unit) does: a RAISE in an
internal routine ends the `::ROUTINE`, the method or the main program that
the internal routine belongs to, and so does a RAISE in the text of an
INTERPRET in any of these. A condition other than SYNTAX is raised in the
caller of that unit, at the clause that called the unit, and the traps of the
unit, of its internal routines and of the code that the unit called play no
part in the answer. The main program has no caller, and [the rule for the main
program](r:instr.raise.main) says what happens there. A SYNTAX condition is
raised in the unit itself, at the clause that runs at the level of the unit.
That clause is the RAISE when it is in the code of the unit or in INTERPRET
text there, and it is the call of the internal routine when the RAISE is in an
internal routine. The traps that the unit has at that level answer the
condition, and a SYNTAX that they do not answer goes up to the caller of the
unit as [the propagation rule](r:cond.information.propagate) says. The other
cases of RAISE with EXIT are not specified.

```rexx
call on user foo name hm
say 'A' plain()
say 'B' .box~new~m
say 'C' viaint()
say 'D' viatext()
say 'E' outer()
exit
hm: say '   main' condition('c') sigl; return
::routine plain
  call on user foo name t
  raise user foo exit 'plain'
t: say '   plain trap' sigl; return
::class box
::method m
  call on user foo name t
  raise user foo exit 'm'
t: say '   m trap' sigl; return
::routine viaint
  call on user foo name t
  v = il()
  say '   viaint goes on'
  return 'viaint' v
il: raise user foo exit 'il'
t: say '   viaint trap' sigl; return
::routine viatext
  call on user foo name t
  interpret "raise user foo exit 'vt'"
t: say '   viatext trap' sigl; return
::routine outer
  call on user foo name t
  v = plain()
  say '   outer goes on'
  return 'outer' v
t: say '   outer trap' sigl; return
```

```output
A plain
   main USER FOO 2
B m
   main USER FOO 3
C il
   main USER FOO 4
D vt
   main USER FOO 5
   outer trap 31
   outer goes on
E outer plain
```

```rexx
say 'A' same()
exit
::routine same
  signal on syntax name unit
  return step()
step: procedure
  signal on syntax name inner
  return deeper()
deeper: procedure
  raise syntax 40.3 exit 'deeper'
inner: say 'inner' sigl; return 'inner'
unit: say 'unit' sigl; return 'unit'
```

```output
unit 5
A unit
```

```rexx
signal on syntax name top
say 'A' bare()
say 'not reached'
exit
top: say 'top' sigl
::routine bare
  return step()
step: raise syntax 40.3 exit 'step'
```

```output
top 2
```

r[instr.raise.here]
A SYNTAX condition that a RAISE with RETURN raises is raised at the RAISE
clause itself, as an error found there would be. The trap of the raising code
for SYNTAX or ANY answers it when that trap is `ON`, so a routine that
switched the trap on, and an internal routine that has it on from its caller,
handle their own RAISE SYNTAX. When that trap is `OFF`, the condition goes up
to the caller as [the propagation rule](r:cond.information.propagate) says,
and the caller sees `POSITION` as the line of the RAISE, `SIGL` as the line of
its own clause and `PROPAGATED` as `1`. The SYNTAX that an unanswered HALT or
NOMETHOD becomes is raised in the same way. With EXIT the SYNTAX is raised in
the unit that holds the RAISE, as [the exit rule](r:instr.raise.exit) says.

```rexx
call go 'plain'
call go 'own'
call go 'mid'
call internal
signal on syntax name m
call inh
say 'after inh'
exit
go: procedure
  signal on syntax name s
  call (arg(1))
  say arg(1) 'returned'
  return
s: say arg(1) 'caught by go' sigl condition('o')~code,
      condition('o')~position condition('o')~propagated
  return
internal: procedure
  signal on syntax name is
  raise syntax 40.3 return
is: say 'internal caught' sigl condition('o')~code
  return
m: say 'inherited' sigl condition('o')~code
  return
inh: procedure
  raise syntax 40.3 return
::routine plain
  raise syntax 41.1 array ('x') return
::routine own
  signal on syntax name mine
  raise syntax 40.3 return
mine: say 'own caught' sigl condition('o')~code
  return
::routine mid
  call plain
  say 'not reached'
```

```output
plain caught by go 11 41.1 27 1
own caught 30 40.3
own returned
mid caught by go 11 41.1 27 1
internal caught 19 40.3
inherited 25 40.3
after inh
```

r[instr.raise.interpret]
In the text of an INTERPRET instruction, a RAISE with RETURN raises the
condition in the code that holds the INTERPRET, at the INTERPRET clause, and
not in the caller of that code. That code then ends as [the rule for leaving
the text](r:instr.interpret.transfer) says: RETURN returns from the routine,
method or internal routine that holds the INTERPRET. A routine whose own trap
is on therefore answers the condition that its INTERPRET raised, and a routine
with no trap ignores it, without the trap of its caller seeing it. A RAISE
with EXIT in INTERPRET text ends the unit, and the condition is raised as
[the exit rule](r:instr.raise.exit) says, in the caller of the unit. In the
main program, the INTERPRET clause is where a RAISE with RETURN in INTERPRET
text raises the condition, and the traps of the main program answer it there.
A RAISE with EXIT there ends the program, and [the rule for the main
program](r:instr.raise.main) says that nothing answers it.

```rexx
call on user foo name h
say 'A' with()
say 'B' without()
say 'C' leaving()
exit
h: say 'h' condition('c') sigl; return
::routine with
  call on user foo name inner
  interpret "raise user foo return 'with value'"
  say 'not reached'
inner: say 'inner' condition('c') sigl; return
::routine without
  interpret "raise user foo return 'without value'"
  say 'not reached'
::routine leaving
  call on user foo name inner
  interpret "raise user foo exit 'leaving value'"
inner: say 'inner' condition('c') sigl; return
```

```output
inner USER FOO 9
A with value
B without value
C leaving value
h USER FOO 4
```

```rexx
call on user foo name h
interpret "say 'text'; raise user foo return 4; say 'not reached'"
say 'not reached'
exit
h: say 'h' condition('c') sigl; return
```

```output rc=4
text
h USER FOO 2
```

r[instr.raise.main]
The main program has no caller. A RAISE in it ends the program, as RETURN or
EXIT would, with the value of the option as the exit code, as
[RETURN](r:instr.return.program) specifies, and the condition is raised in the
main program itself, at the RAISE clause. No trap answers a condition other
than SYNTAX there, ANY included, so it is ignored, except for an unanswered
HALT or NOMETHOD, which becomes a SYNTAX condition as [the untrapped
rule](r:instr.raise.untrapped) says. The same holds for a RAISE with EXIT in an
internal routine of the main program and in INTERPRET text of the main
program: the program ends, and as [the exit rule](r:instr.raise.exit) says, a
condition other than SYNTAX has no caller to be raised in, so no trap sees it,
and a SYNTAX is raised in the main program at the clause that was running
there, which is, for an internal routine, the call of it, and answered by the traps of
the main program. A RAISE with RETURN in an internal routine of the main
program is the case of [the caller rule](r:instr.raise.caller): the main
program is the caller.

```rexx
call on any name ca
signal on any name sa
signal on syntax name sy
say 'start'
raise user foo return 'x'
say 'not reached'
exit
ca: say 'call trap' condition('c'); return
sa: say 'signal trap' condition('c'); exit
sy: say 'sy' condition('c'); exit
```

```output
start
```

```rexx
say 'start'
raise error 3 exit 7
say 'not reached'
```

```output rc=7
start
```

```rexx
signal on syntax name s
say 'start'
raise halt description 'now'
say 'not reached'
exit
s: say 'syntax' condition('o')~code '['condition('d')']' sigl
  signal on syntax name s2
  raise syntax 40.3
s2: say 'again' condition('o')~code sigl
```

```output
start
syntax 4.1 [] 3
again 40.3 8
```

```rexx
signal on syntax name s
say 'start'
call ie
say 'not reached'
exit
ie: raise syntax 40.3 exit
s: say 'syntax' condition('o')~code sigl condition('o')~position
```

```output
start
syntax 40.3 3 6
```

```rexx
call on user foo name h
signal on user foo name s
say 'start'
call ie
say 'not reached'
exit
h: say 'call trap' condition('c'); return
s: say 'signal trap' condition('c'); exit
ie: raise user foo exit 3
```

```output rc=3
start
```

## When no trap answers

r[instr.raise.untrapped]
A condition that RAISE raises and that no trap answers is handled as [the
untrapped rule](r:cond.index.untrapped) says, and [the HALT
rule](r:cond.halt.untrapped) says that a HALT becomes `SYNTAX 4.1` whatever
description the clause gave. RAISE adds two cases. An untrapped NOMETHOD
becomes `SYNTAX 97.1` when the clause gave ADDITIONAL or ARRAY, and `SYNTAX
97.5` when it gave neither, whatever its description. A FAILURE that RAISE
raised is not answered as an ERROR when its own trap is off, unlike the
FAILURE of a command, which [falls back to the ERROR
trap](r:cond.error.fallback). A SYNTAX condition with no answering trap ends
the program, as [the SYNTAX rule](r:cond.index.syntax) says.

```rexx
do w over 'R1 R2 R3 R4 R5 R6 R7'~makearray(' ')
  call go w
end
exit
go: procedure
  signal on syntax name s
  call (arg(1))
  say arg(1) 'ignored'
  return
s: say arg(1) 'SYNTAX' condition('o')~code
  return
::routine r1; raise halt return
::routine r2; raise halt description 'stop'
::routine r3; raise nomethod
::routine r4; raise nomethod description 'bark'
::routine r5; raise nomethod additional ('obj') description 'bark'
::routine r6; raise nomethod array ('o', 'p')
::routine r7
  signal on syntax name mine
  raise halt return
mine: say 'r7 itself' condition('o')~code sigl
  return
```

```output
R1 SYNTAX 4.1
R2 SYNTAX 4.1
R3 SYNTAX 97.5
R4 SYNTAX 97.5
R5 SYNTAX 97.1
R6 SYNTAX 97.1
r7 itself 4.1 20
R7 ignored
```

```rexx
call on error name e
call r1
call r2
exit
e: say 'e' condition('c') rc; return
::routine r1; raise failure 7 return
::routine r2; raise error 8 return
```

```output
e ERROR 8
```

```rexx
say 'before'
call r
say 'not reached'
exit
r: raise syntax 40.3 array ('a', 'b') return
```

```output rc=216
before
```

## Raising a condition again

r[instr.raise.propagate]
RAISE PROPAGATE raises again the condition that the code holding the clause
has recorded, as [the record rule](r:cond.information.record) and [the scope
rule of CONDITION](r:bif.condition.scope) describe. The condition is raised in
the caller of that code, as for any other condition, and it keeps its name and
the entries that it held, so a user condition is still the same user
condition, a SYNTAX keeps its `CODE` and `RC`, and the `ADDITIONAL` entry
stays, unless the clause replaces it as [the replacement
rule](r:instr.raise.propagate.replace) says. In the new condition object
`PROPAGATED` is `1`, as it is in the object of any SYNTAX condition that
reached a caller, so for a SYNTAX the value does not tell a PROPAGATE from a
condition that travelled on its own. The `POSITION` entry
of a propagated user condition is still the line of the original RAISE, and
that of a propagated SYNTAX is the line of the clause in the caller, which is
also `SIGL`. A RETURN or EXIT value on the clause is the value that the code
hands back, as [the rule for leaving](r:instr.raise.leave) says, and [the
Deviation on values](r:instr.raise.deviation.65) says what the condition holds
of it.

```rexx
signal on syntax name top
call r
say 'not reached'
exit
top: o = condition('o')
  say 'top' o~code o~rc o~propagated o~position sigl
::routine r
  signal on syntax name lbl
  x = 1 + 'a'
lbl: say 'r' condition('c') condition('o')~propagated
  raise propagate
```

```output
r SYNTAX 0
top 41.1 41 1 2 2
```

```rexx
signal on user foo name top
call r
say 'not reached'
exit
top: o = condition('o')
  say 'top' o~condition o~propagated '['o~description']' o~position sigl,
      o~additional
::routine r
  signal on user foo name lbl
  call s
  say 'not reached'
lbl: o = condition('o')
  say 'r' o~condition o~propagated '['o~description']' o~position sigl,
      o~additional
  raise propagate
::routine s
  raise user foo description 'first' additional ('first')
```

```output
r USER FOO 0 [first] 17 10 first
top USER FOO 1 [first] 17 2 first
```

```rexx
call on user foo name top
x = r()
say 'x is' x
exit
top: say 'top' condition('o')~propagated; return
::routine r
  signal on user foo name lbl
  call s
lbl: raise propagate return 'handed back'
::routine s
  raise user foo return
```

```output
top 1
x is handed back
```

```rexx
signal on syntax name h
say 'start'
x = 1 + 'a'
h: say 'h' condition('c')
raise propagate
```

```output rc=215
start
h SYNTAX
```

r[instr.raise.propagate.replace]
DESCRIPTION, ADDITIONAL and ARRAY on a RAISE PROPAGATE replace the values that
the condition passed on holds: the description becomes the value of the
DESCRIPTION expression, and the `ADDITIONAL` entry becomes the value of the
ADDITIONAL expression or the array that ARRAY gives. An entry that the clause
does not name keeps the value that the condition had.

```rexx
call on user foo name top
do i = 1 to 4
  call ('MID'i)
end
exit
top: o = condition('o')
  a = o~additional
  if a~isA(.array) then a = 'array' a~toString('l', ',')
  say i '['o~description']' a
  return
::routine mid1
  signal on user foo name lbl
  call src
lbl: raise propagate
::routine mid2
  signal on user foo name lbl
  call src
lbl: raise propagate description 'new'
::routine mid3
  signal on user foo name lbl
  call src
lbl: raise propagate additional ('new add')
::routine mid4
  signal on user foo name lbl
  call src
lbl: raise propagate array ('p', 'q') description 'd4'
::routine src
  raise user foo description 'orig' additional ('orig add')
```

```output defect=#115
1 [orig] orig add
2 [new] orig add
3 [orig] new add
4 [d4] array p,q
```

r[instr.raise.propagate.none]
When the code that holds RAISE PROPAGATE has no recorded condition, the clause
raises `SYNTAX 98.918`, in the way [a SYNTAX](r:instr.raise.here) is raised.

```rexx
signal on syntax name s
v = 'before'
call r
say 'not reached'
exit
s: say 'syntax' condition('o')~code
::routine r
  raise propagate
```

```output
syntax 98.918
```

## Deviations

r[instr.raise.deviation.65]
A value on RAISE's RETURN or EXIT option is the condition object's `RESULT`
for every condition, and a RAISE PROPAGATE keeps it. ooRexx sets `RESULT` only
for SYNTAX.

```rexx
call on user foo name h
call on error name h
call r1
call r2
call r3
call r4
call keeper
signal on syntax name s
call r5
exit
h: o = condition('o')
  if o~hasindex('RESULT') then say condition('c') 'RESULT' o~result
  else say condition('c') 'no RESULT entry'
  return
s: o = condition('o')
  say condition('c') o~code 'RESULT' o~result
::routine r1; raise user foo return 'one'
::routine r2; raise user foo exit 'two'
::routine r3; raise error 3 return 'three'
::routine r4; raise user foo
::routine r5; raise syntax 40.3 return 'five'
::routine keeper
  signal on user foo name again
  call source
  say 'not reached'
again: raise propagate
::routine source
  raise user foo return 'kept'
```

```output
USER FOO RESULT one
USER FOO RESULT two
ERROR RESULT three
USER FOO no RESULT entry
USER FOO RESULT kept
SYNTAX 40.3 RESULT five
```
