---
element: "NOVALUE"
kind: concept
ansi: "8.4.1"
rexxref: "Conditions and Condition Traps; Conditions and Condition Traps > Condition Information > Descriptive Strings; Conditions and Condition Traps > Action Taken when a Condition Is Trapped"
---

# NOVALUE

NOVALUE is the condition of reading a variable that has no value. A variable
that was never assigned, or that was dropped, is not an error in Rexx: its
symbol then stands for its own name, as [the variable
rule](r:expr.variables.value) says. A program that wants to find such reads
traps the condition, which turns a misspelled variable into a stop with the
name in hand. This page specifies which uses of a variable raise NOVALUE,
which do not, what the description holds, and which traps can answer it. The
directive `::OPTIONS` can make an untrapped NOVALUE into an error, and the
page of the directive specifies that.

## When it is raised

r[cond.novalue.raise]
NOVALUE is raised when the value of a simple or compound variable symbol is
needed and the variable has no value. That is so for a term of an expression
wherever an expression is evaluated, among them the right side of an
assignment, the condition of an IF or a WHEN, the bounds of a DO, an argument
of a call and the value of a NUMERIC setting; for the variable that PARSE VAR
reads; for a pattern in parentheses in a template; for a variable that names
the override class of a message term; and for a variable that names a list in
parentheses in DROP, in EXPOSE or in PROCEDURE EXPOSE. The clause ends at that
point, so an assignment it would have made is not made. The stem symbol alone,
as in `st.`, is never without a value, as [the stem
rule](r:expr.variables.stem) says, so reading it raises nothing. A variable
that is only assigned, dropped or named in an EXPOSE list, with no
parentheses, is not read. The list in parentheses of EXPOSE in a method shows
the read through the `::OPTIONS` directive, which turns the untrapped condition
into an error. An environment symbol such as `.foo`
that finds no object is not a variable, so it raises nothing and stands for
its own characters. A symbol in the tail of a compound symbol raises nothing,
and stays as its own name in the derived name. SYMBOL and VALUE do not raise
it, as [the function rules](r:bif.value.get) say.

```rexx
do k = 1 to 16
  call probe k
end
exit
::routine probe
  use arg k
  signal on novalue name raised
  select
    when k = 1 then x = 1 + unset1
    when k = 2 then if unset2 = 3 then nop
    when k = 3 then do i = unset3 to 2; end
    when k = 4 then call length unset4
    when k = 5 then numeric digits unset5
    when k = 6 then parse value 'a b' with (unset6) w1 w2
    when k = 7 then parse var unset7 w1 w2
    when k = 8 then say .probe~new~go
    when k = 9 then say st.
    when k = 10 then say symbol('unset10') value('unset10')
    when k = 11 then drop unset11
    when k = 12 then unset12 = 5
    when k = 13 then say st.1
    when k = 14 then call hasexpose
    when k = 15 then drop (unset17)
    when k = 16 then say .unsetenv
  end
  say k 'no NOVALUE'
  return
raised: say k 'NOVALUE' condition('d')
  return
hasexpose: procedure expose unset15
  return
::class probe
::method go
  signal on novalue name q
  return 'abc'~length:unset16()
  q: return 'method' condition('d')
```

```output
1 NOVALUE UNSET1
2 NOVALUE UNSET2
3 NOVALUE UNSET3
4 NOVALUE UNSET4
5 NOVALUE UNSET5
6 NOVALUE UNSET6
7 NOVALUE UNSET7
method UNSET16
8 no NOVALUE
ST.
9 no NOVALUE
LIT UNSET10
10 no NOVALUE
11 no NOVALUE
12 no NOVALUE
13 NOVALUE ST.1
14 no NOVALUE
15 NOVALUE UNSET17
.UNSETENV
16 no NOVALUE
```

```rexx
signal on novalue name n
call keep
say 'went on'
exit
n: say 'novalue' condition('d')
  exit
keep: procedure expose (names)
  return
```

```output
novalue NAMES
```

```rexx
say .store~new~fill
::options novalue syntax
::class store
::method fill
  expose (names)
  return 'filled'
```

```output rc=158
```

r[cond.novalue.description]
The description of a NOVALUE condition is the name of the variable that was
read, in upper case. For a compound symbol it is the derived name, in which a
tail symbol that has a value is replaced by it and one that has none stays as
its own name, as [the tail rule](r:expr.variables.tail) says. `CONDITION('D')`
and the `DESCRIPTION` entry answer it, and the condition has no `ADDITIONAL`
and no `RC`. A tail symbol that has no value is not a NOVALUE condition of its
own, so the one condition names the compound variable.

```rexx
signal on novalue name n
i = 3
x = abc
n: say condition('c') '['condition('d')']' sigl
signal on novalue name n2
x = st.i
n2: say '['condition('d')']'
signal on novalue name n3
x = st.i.j
n3: say '['condition('d')']'
signal on novalue name n4
key = 'a b'
x = stem.key
n4: say '['condition('d')']'
o = condition('o')
say o~hasindex('ADDITIONAL') o~hasindex('RC') o~description
```

```output
NOVALUE [ABC] 3
[ST.3]
[ST.3.J]
[STEM.a b]
0 0 STEM.a b
```

## Which traps answer it

r[cond.novalue.trap]
NOVALUE is answered by SIGNAL ON NOVALUE and by SIGNAL ON ANY, and by no CALL
trap: CALL ON NOVALUE is an error, as [the CALL ON rule](r:instr.call.on)
says, and a CALL ON ANY trap does not see it, as [the choice
rule](r:cond.traps.choice) says. With the trap `OFF`, and no ANY trap on, the
symbol has its own name as value and the clause goes on, as [the untrapped
rule](r:cond.index.untrapped) says. A SIGNAL trap ends the clause, so the
value that the clause would have computed is not used. NOVALUE is not passed
on, as [the propagation rule](r:cond.information.propagate.only) says. The
traps that answer it are the ones of the code that read the variable, which
for an internal routine are its copy of the traps of its caller, as [the scope
rule](r:cond.traps.scope) says. A `::ROUTINE` starts with every trap `OFF`,
and an internal routine that switches the trap off is not answered either.

```rexx
x = 'before'
call on any name h
y = abc
say 'ignored' y
signal on novalue name n
x = abc
say 'not reached'
n: say 'trapped' x condition('c')
signal on any name a
x = abd
a: say 'ANY' condition('c') condition('d')
call r
say 'back'
exit
h: say 'not called'
  return
::routine r
  say 'in r' abe
```

```output
ignored ABC
trapped before NOVALUE
ANY NOVALUE ABD
in r ABE
back
```

```rexx
signal on novalue name h
call copy
say 'back'
call quiet
say 'back again'
exit
h: say 'trap' condition('d') sigl
  return
copy:
  x = abe
  return
quiet:
  signal off novalue
  x = abf
  say 'quiet went on' x
  return
```

```output
trap ABE 10
back
quiet went on ABF
back again
```

```rexx
call on novalue name h
```

```output rc=231
```
