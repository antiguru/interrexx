---
element: SYMBOL
kind: builtin
syntax: "SYMBOL(name)"
ansi: "9.8.4"
rexxref: "Functions > Built-in Functions > SYMBOL"
---

# SYMBOL

SYMBOL tells whether a string is a symbol and, if so, whether a variable of
that name holds a value where SYMBOL is called. The answer is one of the three
strings `BAD`, `VAR` and `LIT`.

## Arguments

r[bif.symbol.args]
SYMBOL takes one argument, `name`, which is a [string](r:conv.string.null),
and nothing else. The argument count is checked by the [builtin call
rules](r:bif.index.call.count): one argument is required and one is the
most. SYMBOL does not run as a String method: sending the message `SYMBOL`
to a string raises `SYNTAX 97.1`.

```rexx
signal on syntax name a
say symbol()
a: say condition('o')~code
signal on syntax name b
say symbol('a', 'b')
b: say condition('o')~code
signal on syntax name c
say symbol(, )
c: say condition('o')~code
signal on syntax name d
say 'a'~symbol
d: say condition('o')~code
```

```output
40.3
40.4
40.3
97.1
```

## The result

r[bif.symbol.result]
The answer is `BAD` when `name` is not a symbol, `VAR` when it is a symbol
that names a variable holding a value, and `LIT` for any other symbol. A
string is a symbol when it has at least one character and each is a letter,
a digit, or one of `.`, `!`, `?` and `_`, as it is for
[the symbol type of DATATYPE](r:bif.datatype.symbol), and it has at most 250
characters; the null string, a string with whitespace anywhere in it, even at
an end, one with any other character, and one of more than 250 characters,
whatever they are, is `BAD`. The string is read as it is, without trimming.
A `name` that is an object other than a string is replaced by a string as
[any string argument](r:conv.string.object) says.

```rexx
n = 7
say symbol('n') symbol('p') symbol('a_b') symbol('a!') symbol('a?') symbol('_')
say symbol('') symbol(' j') symbol('j ') symbol('a b') symbol('+5') symbol('a-b'),
    symbol('a$') symbol('"j"') symbol('é')
say symbol(.nil) symbol(.array~new) symbol(.mutablebuffer~new('n'))
say symbol(copies('a', 250)) symbol(copies('a', 251)) symbol(copies('1', 251)),
    symbol(copies('a', 100)'.'copies('b', 149)) symbol(copies('a', 100)'.'copies('b', 150))
```

```output
VAR LIT LIT LIT LIT LIT
BAD BAD BAD BAD BAD BAD BAD BAD BAD
BAD BAD VAR
LIT BAD BAD LIT BAD
```

> **Note.** The limit of 250 is on the whole string, stem and tail together.

r[bif.symbol.case]
SYMBOL reads `name` in uppercase, as a program reads a symbol, so `name`
names the same variable in either case.

```rexx
n = 7
say symbol('n') symbol('N') symbol('p') symbol('P')
```

```output
VAR VAR LIT LIT
```

r[bif.symbol.constant]
A symbol that starts with a digit or a period is a constant symbol and is
never a variable, so SYMBOL answers `LIT` for it. The name `.nil` and the
other names that stand for environment objects are of this kind, whether or
not the object exists.

```rexx
say symbol('3') symbol('1.5') symbol('.5') symbol('1e5') symbol('.') symbol('..'),
    symbol('.nil') symbol('.zzz')
```

```output
LIT LIT LIT LIT LIT LIT LIT LIT
```

r[bif.symbol.simple]
For any other symbol without a period inside it, SYMBOL answers `VAR` when
a variable of that name holds a value and `LIT` when it does not. A variable
that has been dropped holds no value. SYMBOL looks at the name itself, not at
the value: a variable whose value is the name of another variable makes the
answer for the name `VAR`, and the answer for the value is a separate one that
the call gets when it is given the value.

```rexx
a = 'b'
b = 'c'
say symbol('a') symbol('b') symbol('c') symbol(a) symbol(b)
drop a
say symbol('a')
```

```output
VAR VAR LIT VAR LIT
LIT
```

r[bif.symbol.compound]
A symbol whose first period has at least one character after it, and that
does not start with a period, is a compound symbol: the part before the first
period is its stem, and the parts after it are its tail, so `a.b.` is a
compound symbol with the tail `b.`. Every part of the tail that
names a variable holding a value is replaced by that value, as in a
program, and the answer is `VAR` when the compound variable so named holds a
value, and `LIT` when it does not. The value of a tail part is used as it
is, not translated to uppercase, and can be any string, whitespace included;
it is not itself checked to be a symbol. An unset part of the tail stands for
its own name in uppercase.

```rexx
n = 7
t.7 = 'x'
say symbol('t.7') symbol('t.n') symbol('T.N') symbol('t.8') symbol('t.m')
drop n
say symbol('n') symbol('t.n') symbol('t.7')
w = 'x y'
c.w = 1
say symbol('c.w') symbol('c.x y')
v = 'k'
d.k = 1
say symbol('d.v') symbol('d.k') symbol('d.K')
e.f. = 1
say symbol('e.f.') symbol('e.f')
```

```output
VAR VAR VAR LIT LIT
LIT LIT VAR
VAR BAD
LIT VAR VAR
VAR LIT
```

r[bif.symbol.stem]
A symbol that ends at its first period is a stem. A stem is `LIT` until the
program refers to the stem or to a compound variable of it, and `VAR` from
then on, also after the stem is dropped. A reference is an assignment, a
read, a DROP of the stem or of a compound variable of it, or a call of SYMBOL
that names a compound variable of it; a call that names the stem itself is
not a reference. After an assignment to the whole stem, a compound variable
of it that was never assigned or dropped is `VAR`, because it has the value
given to the stem. A compound variable that was dropped, and every compound
variable of a dropped stem, holds no value and is `LIT`.

```rexx
say symbol('a.')
say symbol('b.1') symbol('b.')
stem. = 0
say symbol('stem.') symbol('stem.zzz') symbol('other.')
h. = 0
drop h.1
say symbol('h.') symbol('h.1') symbol('h.2')
c.1 = 5
drop c.
say symbol('c.') symbol('c.1')
d = e.1
say symbol('e.')
```

```output
LIT
LIT VAR
VAR VAR LIT
VAR LIT VAR
VAR LIT
VAR
```

> **Note.** In the second line the call that names `b.1` is a reference to
> `b.`, which the call after it sees; the same two calls in the other order
> answer `LIT` `LIT`.

## Scope

r[bif.symbol.scope]
SYMBOL looks at the variables that the routine or method running it can
see. A routine called with a PROCEDURE instruction sees none of its caller's
variables, except the ones it EXPOSEs; a method sees an object variable only
once it exposes it, and the variable is `VAR` only when it holds a value.

```rexx
g = 1
call hide
call show
exit
hide: procedure
say symbol('g')
return
show: procedure expose g
say symbol('g')
return
```

```output
LIT
VAR
```

```rexx
o = .o~new
say o~plain '/' o~exposing '/' o~assigning '/' o~exposing
::class o
::attribute at
::method plain
return symbol('at') symbol('un')
::method exposing
expose at un
return symbol('at') symbol('un')
::method assigning
expose at
at = 1
return symbol('at')
```

```output
LIT LIT / LIT LIT / VAR / VAR LIT
```

r[bif.symbol.result-var]
The variable `RESULT` is `VAR` after a CALL of a routine that returned a
value and `LIT` after a CALL of one that returned none, as the instruction
sets or drops it.

```rexx
call none
say symbol('result')
call one
say symbol('result')
call none
say symbol('result')
exit
none: return
one: return 1
```

```output
LIT
VAR
LIT
```
