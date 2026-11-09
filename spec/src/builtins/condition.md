---
element: CONDITION
kind: builtin
syntax: "CONDITION([option])"
ansi: "9.5.3"
rexxref: "Functions > Built-in Functions > CONDITION"
---

# CONDITION

CONDITION reports what the running routine knows about the condition that a
trap caught most recently: its name, the instruction that handled it, the
state of its trap, and the descriptive information it carried. The conditions
themselves, and the traps that catch them, are not defined on this page; it
specifies what CONDITION answers for each of its options, before any trap has
caught a condition, after a SIGNAL ON trap or a CALL ON trap has caught one,
and in the code that follows.

## Arguments

r[bif.condition.args]
CONDITION takes at most one argument, `option`, which is optional. The
argument count is checked by the [builtin call rules](r:bif.index.call.count):
none is required and one is the most, so a call with two arguments raises
`SYNTAX 40.4`, whatever they are, and a call with only omitted places counts
as a call with no arguments. CONDITION does not run as a String method:
sending the message `CONDITION` to a string raises `SYNTAX 97.1`.

```rexx
say '['condition()']' '['condition( )']' '['condition(,)']'
signal on syntax name a
say condition('C', 'D')
a: say condition('o')~code
signal on syntax name b
say 'x'~condition
b: say condition('o')~code
```

```output
[] [] []
40.4
97.1
```

r[bif.condition.option]
`option` is a string. Its first character, in either case, selects what
CONDITION answers, and every character after the first is ignored, so
`Condition`, `cond` and `C` select the same. The letters are `A`, `C`, `D`,
`E`, `I`, `O`, `R` and `S`. Any other first character, a blank included, and
the null string raise `SYNTAX 40.904`. When `option` is omitted, the call is
the call with the option `I`.

```rexx
signal on syntax name h
x = 1 + 'a'
h:
say condition('c') condition('Condition') condition('cOND') condition('Cx')
say condition() condition('I') condition('instruction') condition('i')
signal on syntax name a
say condition('x')
a: say condition('o')~code
signal on syntax name b
say condition('')
b: say condition('o')~code
signal on syntax name c
say condition(' c')
c: say condition('o')~code
signal on syntax name d
say condition(.nil)
d: say condition('o')~code
```

```output
SYNTAX SYNTAX SYNTAX SYNTAX
SIGNAL SIGNAL SIGNAL SIGNAL
40.904
40.904
40.904
40.904
```

## When nothing has been caught

r[bif.condition.none]
In a routine, method or program where no trap has caught a condition yet,
CONDITION answers the null string for the options `C`, `D`, `E`, `I` and `S`
and for `R`, and `.nil` for `A` and `O`.

```rexx
say '['condition('C')']' '['condition('D')']' '['condition('E')']',
    '['condition('I')']' '['condition('S')']' '['condition('R')']'
say (condition('A') == .nil) (condition('O') == .nil)
```

```output
[] [] [] [] [] []
1 1
```

## What a trapped condition answers

r[bif.condition.name]
After a trap has caught a condition, the option `C` answers the name of the
condition in capital letters: for a SYNTAX error `SYNTAX`, for an unset
variable under [NOVALUE](r:cond.novalue.raise) `NOVALUE`. For a user condition
raised by the [RAISE instruction](r:instr.raise.form), it answers `USER`, a
blank and the name of the condition in capital letters.

```rexx
signal on syntax name s
x = 1 + 'a'
s: say condition('C')
signal on novalue name n
y = unset + 1
n: say condition('C')
call on user Foo name u
call raiser
exit
raiser: procedure
raise user fOo return
u: say condition('C')
return
```

```output
SYNTAX
NOVALUE
USER FOO
```

r[bif.condition.instruction]
After a trap has caught a condition, the option `I` answers `SIGNAL` when the
trap is a SIGNAL ON trap and `CALL` when it is a CALL ON trap.

```rexx
signal on syntax name s
x = 1 + 'a'
s: say condition('I')
call on user foo name u
call raiser
exit
raiser: procedure
raise user foo return
u: say condition('I')
return
```

```output
SIGNAL
CALL
```

r[bif.condition.description]
After a trap has caught a condition, the option `D` answers the description
that the condition carries, or the null string when it carries none. A
user condition carries what the RAISE instruction's
[DESCRIPTION](r:instr.raise.description) gave, and a NOVALUE condition the
name of the variable, in capital letters. A SYNTAX condition that the
interpreter raises because of an error carries no description string; one that
the RAISE instruction raises carries what its DESCRIPTION gave, and none when
it has no DESCRIPTION, whether or not it has an ADDITIONAL.

```rexx
signal on syntax name s
x = 1 + 'a'
s: say '['condition('D')']'
signal on syntax name r
call rs
r: say '['condition('D')']'
signal on syntax name q
call rq
q: say '['condition('D')']'
signal on novalue name n
y = unset + 1
n: say '['condition('D')']'
call on user foo name u
call raiser
exit
raiser: procedure
raise user foo description 'a note' return
u: say '['condition('D')']'
return
rs: raise syntax 40.1 description 'dd' return
rq: raise syntax 40.1 additional ('ad') return
```

```output
[]
[dd]
[]
[UNSET]
[a note]
```

r[bif.condition.description.object]
The answer of `D` is the object that the DESCRIPTION entry of the condition
object holds, not a string made from it, so a RAISE whose DESCRIPTION is an
array makes `D` answer that array, which is equal to the entry.

```rexx
call on user u name h
call r
exit
r: raise user u description (.array~of(1, 2)) return
h: o = condition('o')
  d = condition('d')
  say d~class '/' (d == o~description) '/' o~description~class
  return
```

```output defect=#121
The Array class / 1 / The Array class
```

r[bif.condition.extra]
After a trap has caught a SYNTAX condition, the option `E` answers the subcode
of the error number: the part after the period, as `12` for `40.12`. An error
number raised without a subcode, as [`RAISE SYNTAX 40`](r:instr.raise.number)
does, has the subcode 0 and the code `40.0`. After any other condition `E`
answers the null string.

```rexx
signal on syntax name s
x = 1 + 'a'
s: say condition('E') condition('O')~code
signal on syntax name t
x = substr('abc', 0)
t: say condition('E') condition('O')~code
signal on novalue name n
y = unset + 1
n: say '['condition('E')']'
signal on syntax name z
call rz
z: say condition('E') condition('O')~code
exit
rz: raise syntax 40 return
```

```output
1 41.1
924 93.924
[]
0 40.0
```

r[bif.condition.additional]
After a trap has caught a condition, the option `A` answers the additional
information object the condition carries, and `.nil` when it carries none. A
user condition carries what the RAISE instruction's
[ADDITIONAL](r:instr.raise.additional) gave, as the object itself, and not a
string made from it.

```rexx
signal on novalue name n
y = unset + 1
n: say condition('A') == .nil
call on user foo name u
call raiser
exit
raiser: procedure
raise user foo additional (.array~of('p', 'q')) return
u: say condition('A')~class condition('A')~items condition('A')~at(2)
return
```

```output
1
The Array class 2 q
```

r[bif.condition.status]
After a trap has caught a condition, the option `S` answers the state of the
trap for that condition as it is now: `ON`, `OFF` or `DELAY`. A SIGNAL ON trap
that has caught a condition is `OFF`, and a CALL ON trap is `DELAY` while its
handler runs. The state changes with the code that follows: a SIGNAL ON or
CALL ON instruction for the condition makes it `ON`, and a SIGNAL OFF or CALL
OFF instruction makes it `OFF`.

```rexx
signal on syntax name s
x = 1 + 'a'
s: say condition('S')
signal on syntax name s2
say condition('S')
s2: signal off syntax
say condition('S')
call on user foo name u
call raiser
exit
raiser: procedure
raise user foo return
u: say condition('S')
call on user foo name u
say condition('S')
call off user foo
say condition('S')
return
```

```output
OFF
ON
OFF
DELAY
ON
OFF
```

r[bif.condition.object]
After a trap has caught a condition, the option `O` answers a directory that
holds the information of the condition, the same information as the other
options answer, and more. Its entries `CONDITION`, `DESCRIPTION` and
`INSTRUCTION` equal what `C`, `D` and `I` answer, and so does `ADDITIONAL`
when the condition carries additional information. When it carries none, as
with NOVALUE, the directory has no `ADDITIONAL` entry, and `~additional`
answers `.nil` as for any absent entry, which is also what `A` answers. The
directory is a new copy of the condition's entries on each call, so two calls
answer two objects, and changing a copy changes nothing that CONDITION answers
or that another object holds. The condition itself is the object that a CALL
trap's routine gets as its argument, as [the argument
rule](r:cond.information.argument) says.

```rexx
signal on syntax name s
x = 1 + 'a'
s: o = condition('O')
say o~class (o~condition == condition('C')) (o~description == condition('D')),
    (o~instruction == condition('I')) (o~additional == condition('A'))
o~condition = 'changed'
say condition('C') condition('O')~condition
signal on novalue name n
y = unset + 1
n: o = condition('O')
say o~hasIndex('ADDITIONAL') (o~additional == .nil) (condition('A') == .nil)
```

```output
The Directory class 1 1 1 1
SYNTAX SYNTAX
0 1 1
```

## How long the information lasts

r[bif.condition.persist]
The information stays until it is replaced. After a SIGNAL ON trap has caught a
condition, the code from the handler's label on, running in the same routine,
method or program, still gets it from CONDITION, until another trap catches a
condition or the option `R` clears it. When the handler of a CALL ON trap
returns, the information is what it was before the handler ran: that of an
earlier trap, or none, in which case CONDITION answers as in
[the case of no condition](r:bif.condition.none).

```rexx
call on user foo name u
call raiser
say '['condition('C')']' '['condition('I')']' '['condition('S')']'
signal on syntax name s
x = 1 + 'a'
s: say condition('C') '/' condition('C')
call raiser
say condition('C') condition('I') condition('S')
exit
raiser: procedure
raise user foo return
u: say 'handler' condition('C')
return
```

```output
handler USER FOO
[] [] []
SYNTAX / SYNTAX
handler USER FOO
SYNTAX SIGNAL OFF
```

r[bif.condition.reset]
The option `R` returns the null string and clears the information of the
running routine, method or program, so that every option answers as in
[the case of no condition](r:bif.condition.none). A condition that a trap
catches afterwards is recorded as usual.

```rexx
signal on syntax name s
x = 1 + 'a'
s: say condition('C') '['condition('R')']' '['condition('C')']',
  (condition('A') == .nil) (condition('O') == .nil)
signal on syntax name t
y = 1 + 'b'
t: say condition('C') condition('S')
```

```output
SYNTAX [] [] 1 1
SYNTAX OFF
```

r[bif.condition.scope]
The information belongs to the code that holds it. An internal routine, a
routine that [CALL](r:instr.call.run) or a function call reaches by its label,
starts with a copy of the information of the code that called it, and what it
does to the copy, a trap that catches a condition in it or the option `R`, is
not seen by the caller. A [`::ROUTINE`](r:dir.routine.create), a method and a
routine made from other source text start with none.
[INTERPRET](r:instr.interpret.scope) runs in the code that holds it, and
shares its information.

```rexx
call on user foo name u
call raiser
exit
raiser: procedure
raise user foo return
u: say 'handler' condition('C')
say 'internal' inner()
say 'directive' dir()
say 'method' .box~new~m
say 'made' .routine~new('x', 'return "["condition("C")"]"')~call
interpret 'say "interpreted" condition("C")'
say 'reset' inner2()
say 'trap' inner3()
say 'caller' condition('C')
return
inner: return condition('C')
inner2: procedure
r = condition('R')
return '['condition('C')']'
inner3: procedure
signal on syntax name z
x = 1 + 'a'
z: return '['condition('C')']'
::routine dir
return '['condition('C')']'
::class box
::method m
return '['condition('C')']'
```

```output
handler USER FOO
internal USER FOO
directive []
method []
made []
interpreted USER FOO
reset []
trap [SYNTAX]
caller USER FOO
```

> **Note.** The ANSI draft (9.5.3) has the options `C`, `D`, `E`, `I` and `S`
> only, and says every answer is null when no condition has been trapped.
> ooRexx adds `A`, `O` and `R`, and interrexx follows ooRexx. The conditions
> chapter lists every condition, the information each carries, and the traps
> that catch them.
