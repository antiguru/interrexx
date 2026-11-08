---
element: "USER conditions"
kind: concept
rexxref: "Conditions and Condition Traps > Action Taken when a Condition Is Trapped"
---

# USER conditions

A user condition is a condition that only the program raises. The interpreter
never raises one, so a program can use it for its own events, such as a
missing record or a parse that found nothing, and give the code that deals
with it the same trap machinery that the built-in conditions have. Every user
condition has a name that the program chooses, and the name makes the
condition. This page specifies the names, the traps that answer a user
condition and what it holds. A user condition is raised with the RAISE
instruction, whose page is [RAISE](../instructions/raise.md), and the examples
here raise them with `raise user name`.

## The name

r[cond.user.name]
A user condition is raised by `RAISE USER` and a name, and by nothing else.
The name is a symbol that is read as a constant in upper case, whatever it
holds as a variable and whatever the spelling of its tail: `foo.bar` is the
name `FOO.BAR` and does not take the value of `bar`. The condition is called
`USER` and the name, joined by one blank, and `CONDITION('C')` answers that.
Names are compared as text after the change to upper case, so `Foo` and `FOO`
are one condition, and the numbers `1.5` and `1.50` are two. A user condition
is a condition of its own, so it is not the built-in condition that has the
same name, as [the names rule](r:cond.index.names) says.

```rexx
bar = 'x'
call on user foo.bar name h
call on user 1.50 name h
call on user Mixed name h
call on any name other
call r1
call r2
call r3
call r4
exit
r1: raise user Foo.Bar return
r2: raise user 1.50 return
r3: raise user 1.5 return
r4: raise user MIXED return
h: say 'h' condition('c') condition('s'); return
other: say 'other' condition('c'); return
```

```output
h USER FOO.BAR DELAY
h USER 1.50 DELAY
other USER 1.5
h USER MIXED DELAY
```

## Which traps answer it

r[cond.user.trap]
A user condition is answered by the trap of its own name, which CALL ON USER
and SIGNAL ON USER set, and otherwise by the ANY trap, as [the choice
rule](r:cond.traps.choice) says. Both kinds of instruction accept it, and the
default routine or label of the trap is the name alone, `FOO` for `user foo`.
A trap for one name does not answer another name, and a trap for `ERROR` does
not answer `USER ERROR`. The effects of the trap are those of [CALL
traps](r:cond.traps.call) and [SIGNAL traps](r:cond.traps.signal): the
handler of a CALL trap runs with the state `DELAY`, and a SIGNAL trap is `OFF`
at its label.

```rexx
call on user alpha
call on user beta name b
signal on error name e
signal on user gamma
call r1
call r2
call r3
call r4
exit
r1: raise user alpha return
r2: raise user beta return
r3: raise user error return
r4: raise user gamma return
alpha: say 'alpha' condition('c') condition('s'); return
b: say 'b' condition('c'); return
e: say 'error trap'
gamma: say 'gamma' condition('c') condition('s')
```

```output
alpha USER ALPHA DELAY
b USER BETA
gamma USER GAMMA OFF
```

## What it holds

r[cond.user.information]
The description of a user condition is the text of the DESCRIPTION option of
the RAISE instruction that raised it, and its `ADDITIONAL` entry is the value
of the ADDITIONAL option. Each entry is present exactly when the instruction
gave it, so a condition raised with neither has no description text and no
`ADDITIONAL`. The condition object has the entries that every condition has
and `POSITION`, and neither `RC` nor `CODE`, as [the entries
rule](r:cond.information.object) lists them.

```rexx
call on user foo name h
call r1
call r2
call r3
exit
r1: raise user foo return
r2: raise user foo description 'why' return
r3: raise user foo description 'why' additional ('more') return
h: o = condition('o')
  say '[' || condition('d') || ']' o~hasindex('ADDITIONAL'),
      o~hasindex('POSITION') o~hasindex('RC') o~hasindex('CODE')
  return
```

```output
[] 0 1 0 0
[why] 0 1 0 0
[why] 1 1 0 0
```

## When nothing traps it

r[cond.user.untrapped]
A user condition that neither its own trap nor the ANY trap answers is
ignored, as [the untrapped rule](r:cond.index.untrapped) says, and the program
goes on as the RAISE instruction specifies. It never becomes a SYNTAX
condition.

```rexx
say 'start'
call r
say 'back'
signal on syntax name s
call r
say 'still going'
exit
r: raise user lost description 'nobody listens' return
s: say 'not reached'
```

```output
start
back
still going
```
