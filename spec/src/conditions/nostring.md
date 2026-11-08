---
element: "NOSTRING"
kind: concept
rexxref: "Conditions and Condition Traps; Conditions and Condition Traps > Condition Information > Descriptive Strings; Conditions and Condition Traps > Condition Information > Additional Object Information; Conditions and Condition Traps > Action Taken when a Condition Is Trapped"
deviations: [30, 77]
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
answers an object that is not a string. An object whose `MAKESTRING` or
`STRING` method is private raises no NOSTRING at the uses listed below: the
message is refused, as [the conventions](r:conv.string.object) say for
builtin arguments, and the clause raises `SYNTAX 97.2`. A string, a number
and an instance of a subclass of `String` never raise it, and neither does an
object whose `MAKESTRING` answers a string. The uses that raise it include
the three concatenation operators (the explicit `||`, the abuttal and the
blank), the operand of SAY, the source of `PARSE VALUE`, the text that
INTERPRET runs, an operand of an arithmetic or comparison operator whose left
operand is a string, the tail of a compound variable, and an argument of a
builtin function that is to be a string, except for the builtins that link
[the native call rule](r:bif.index.call.native), which raise `SYNTAX 88.909`
for it. Sending the message
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

> The next program tries the other positions and the private methods. A
> comparison or an arithmetic operator with the object on its left is a
> message to the object, so the operands here stand to the right of a string.

```rexx
do j = 1 to 8
  say probe(j)
end
exit
::routine probe
  use arg j
  signal on syntax name s
  signal on nostring name n
  select
    when j = 1 then x = 1 + .object~new
    when j = 2 then x = ('a' = .object~new)
    when j = 3 then do; t = .object~new; a.t = 5; end
    when j = 4 then x = 'a' || .ps~new
    when j = 5 then x = 'a' || .pm~new
    when j = 6 then x = filespec('n', .object~new)
    when j = 7 then parse value .ps~new with x .
    when j = 8 then do; t = .pm~new; a.t = 5; end
  end
  return j 'nothing'
n: return j 'NOSTRING'
s: return j 'SYNTAX' condition('o')~code
::class ps
::method string private
  return 'x'
::class pm
::method makestring private
  return 'x'
```

```output
1 NOSTRING
2 NOSTRING
3 NOSTRING
4 SYNTAX 97.2
5 SYNTAX 97.2
6 SYNTAX 88.909
7 SYNTAX 97.2
8 SYNTAX 97.2
```

r[cond.nostring.information]
The description of a NOSTRING condition is the text that the `STRING` method
of the object answers when that is a string, and its `ADDITIONAL` entry is the
object. When the answer is an object that is not a string but has a string
value, such as an array, the description is that value. When the answer has no
string value, [Deviation 30](r:cond.nostring.deviation.30) applies. The
condition object has the entries that every condition has, `ADDITIONAL` and
`POSITION`, and neither `RC` nor `CODE`, as [the entries
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
signal on nostring name n6
x = 'a' || .arr~new
n6: say 'arr:' '['condition('d')']' condition('o')~additional~class
::class plain
::class told
::method string
  return 'told text'
::class badmake
::method makestring
  return .nil
::method string
  return 'bad text'
::class arr
::method string
  return .array~of('arr text')
```

```output
start
say: NOSTRING [an Object] 3
interpret: NOSTRING [an Object]
1 1 0 0 The Object class
plain: [a PLAIN]
told: [told text] The TOLD class
badmake: [bad text] The BADMAKE class
arr: [arr text] The ARR class
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
as the string value of the object the text that its `STRING` method answers
when that is a string, and the string value of that answer when it is another
object that has one, such as an array. [Deviation
30](r:cond.nostring.deviation.30) applies when the answer has none. The
[::OPTIONS](r:dir.options.condition) directive can turn an untrapped NOSTRING
into a SYNTAX condition, and its rule specifies that.

```rexx
say 'a' || .plain~new
say 'b' || .told~new
say .badmake~new
say length(.told~new) 'c' .told~new
say 'd' || .arr~new
::class plain
::class told
::method string
  return 'told text'
::class badmake
::method makestring
  return .nil
::method string
  return 'bad text'
::class arr
::method string
  return .array~of('arr text')
```

```output
aa PLAIN
btold text
bad text
9 c told text
darr text
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

r[cond.nostring.deviation.30]
When the `STRING` method of an object answers an object with no string value,
interrexx gives ooRexx's answer wherever that is defined, and elsewhere ends
the program with exit code 120 after raising NOSTRING if it is trapped. For a
NOSTRING raised for such an object, the description is not specified. ooRexx
reads `.nil` as a string there, with garbage results or a crash. [The
conventions](r:conv.string.deviation.30) state it for builtin arguments.

```rexx
signal on nostring name n
say 'x' .weird~new
n: say 'trapped' condition('c') condition('o')~additional~class
x = 'a' || .weird~new
say 'not reached'
::class weird
::method string
  return .directory~new
```

```output rc=120
trapped NOSTRING The WEIRD class
```
