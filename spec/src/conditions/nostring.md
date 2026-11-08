---
element: "NOSTRING"
kind: concept
rexxref: "Conditions and Condition Traps > Action Taken when a Condition Is Trapped"
deviations: [77]
---

# NOSTRING

NOSTRING is the condition of needing the string value of an object that does
not provide one. Most values a program handles are strings or answer as
strings, and the interpreter converts them without notice. An object of a
class that has no way to give a string is different: Rexx can still print it
or join it to text, but it first tells the program, so that a program that
trusts its values to be text can stop at the point where one is not. This page
specifies which uses of an object raise NOSTRING, what the condition holds,
which traps answer it and what a clause does when nothing does.

## When it is raised

r[cond.nostring.raise]
NOSTRING is raised when a clause needs the string value of an object that is
not a string, and the object has no `MAKESTRING` method, or has one that
answers an object that is not a string. A string, a number and an instance of
a subclass of `String` never raise it, and neither does an object whose
`MAKESTRING` answers a string. The uses that raise it include the three
concatenation operators (the explicit `||`, the abuttal and the blank), the
operand of SAY, the source of `PARSE VALUE`, the text that INTERPRET runs, and
an argument of a builtin function that is to be a string. Sending the message
`STRING` to the object raises nothing, because the object is then not
converted. Which other positions convert, such as the arguments of method
calls, is not specified. What a builtin does with such an argument when no
trap answers is [the object rule of the conventions](r:conv.string.object).
The table below has one row for each object and one column for each of six
uses, `N` where NOSTRING is raised and `-` where it is not.

```rexx
objs = .array~of(.object~new, .made~new, .string.sub~new('xy'), 42)
say 'cols: || abuttal blank ~string parse upper'
do k = 1 to 4
  line = k
  do j = 1 to 6
    line = line probe(j, objs[k])
  end
  say line
end
exit
::routine probe
  use arg j, o
  signal on nostring name n
  select
    when j = 1 then x = 'a' || o
    when j = 2 then x = 'a'o
    when j = 3 then x = 'a' o
    when j = 4 then x = o~string~length
    when j = 5 then parse value o with x .
    when j = 6 then x = upper(o)
  end
  return '-'
n: return 'N'
::class made
::method makestring
  return 'made'
::class string.sub subclass string
```

```output
cols: || abuttal blank ~string parse upper
1 N N N - N N
2 - - - - - -
3 - - - - - -
4 - - - - - -
```

r[cond.nostring.information]
The description of a NOSTRING condition is the text that the `STRING` method
of the object answers, and its `ADDITIONAL` entry is the object. The condition
object has the entries that every condition has, `ADDITIONAL` and `POSITION`,
and neither `RC` nor `CODE`, as [the entries
rule](r:cond.information.object) lists them. The text is the one that the
object would have as its string value when no trap answers, as [the untrapped
rule](r:cond.nostring.untrapped) says.

```rexx
signal on nostring name n
say 'start'
say .object~new
say 'not reached'
n: say 'say:' condition('c') '['condition('d')']' sigl
signal on nostring name n2
interpret .object~new
n2: say 'interpret:' condition('c') '['condition('d')']'
o = condition('o')
say o~hasindex('ADDITIONAL') o~hasindex('POSITION') o~hasindex('CODE'),
    o~hasindex('RC') o~additional~class
signal on nostring name n3
x = 'a' || .plain~new
n3: say 'plain:' '['condition('d')']'
signal on nostring name n4
x = 'a' || .told~new
n4: say 'told:' '['condition('d')']' condition('o')~additional~class
signal on nostring name n5
x = 'a' || .badmake~new
n5: say 'badmake:' '['condition('d')']' condition('o')~additional~class
::class plain
::class told
::method string
  return 'told text'
::class badmake
::method makestring
  return .nil
::method string
  return 'bad text'
```

```output
start
say: NOSTRING [an Object] 3
interpret: NOSTRING [an Object]
1 1 0 0 The Object class
plain: [a PLAIN]
told: [told text] The TOLD class
badmake: [bad text] The BADMAKE class
```

## Which traps answer it

r[cond.nostring.trap]
NOSTRING is answered by SIGNAL ON NOSTRING and by SIGNAL ON ANY, and by no
CALL trap: CALL ON NOSTRING is `SYNTAX 25.1`, as [the CALL
rule](r:instr.call.on) says, and a CALL ON ANY trap does not see it, as [the
choice rule](r:cond.traps.choice) says. A SIGNAL trap ends the clause that
needed the string, so the clause makes none of its changes, and sets `SIGL` to
the line of that clause. The condition is raised in the code that needed the
string and goes no further, so the trap of a caller does not answer a NOSTRING
that a routine raised, as [the propagation
rule](r:cond.information.propagate.only) says.

```rexx
signal on syntax name s
interpret 'call on nostring name h'
s: say condition('c') condition('o')~code
call on any name h
x = 'kept'
signal on nostring name n
x = 'a' || .object~new
say 'not reached'
n: say 'trapped' x condition('c') sigl
signal on any name a
y = 'a' || .object~new
a: say 'ANY' condition('c') '['y']'
signal on nostring name main
call r
say 'back from r'
exit
main: say 'main trap'
h: say 'not called'
  return
::routine r
  z = 'a' || .object~new
  say 'in r' z
```

```output
SYNTAX 25.1
trapped kept NOSTRING 7
ANY NOSTRING [Y]
in r aan Object
back from r
```

## When nothing traps it

r[cond.nostring.untrapped]
A NOSTRING that neither its own trap nor the ANY trap answers is ignored, as
[the untrapped rule](r:cond.index.untrapped) says: the clause goes on and uses
as the string value of the object the text that its `STRING` method answers.
The directive `::OPTIONS` can turn an untrapped NOSTRING into a SYNTAX
condition, and the page of the directive specifies that.

```rexx
say 'a' || .plain~new
say 'b' || .told~new
say .badmake~new
say length(.told~new) 'c' .told~new
::class plain
::class told
::method string
  return 'told text'
::class badmake
::method makestring
  return .nil
::method string
  return 'bad text'
```

```output
aa PLAIN
btold text
bad text
9 c told text
```

## Deviations

r[cond.nostring.deviation.77]
The string value of a stem is its default value's, and NOSTRING is raised,
with the default's description, only when the default has no string value. A
stem that has no default is a string, its own name, and one with a string
default is that string. ooRexx raises NOSTRING for every stem in a position
that needs a string, with the name of the stem as description.
[Parsing](r:instr.parse.deviation.77) states the same for a stem that is the
source of PARSE.

```rexx
signal on nostring name n
t.1 = 'element'
say 'a' || t.
u. = 'dflt'
say 'b' || u.
v. = .plain~new
say 'c' || v.
say 'not reached'
n: say 'NOSTRING' '['condition('d')']' condition('o')~additional~class
::class plain
```

```output
aT.
bdflt
NOSTRING [a PLAIN] The Stem class
```
