---
element: SYMBOL
kind: builtin
syntax: "SYMBOL(name)"
ansi: "9.8.4"
rexxref: "Functions > Built-in Functions > SYMBOL"
---

# SYMBOL

SYMBOL answers the state of the symbol that a string names: whether the
string is a symbol at all, and if so whether a variable of that name holds a
value where SYMBOL is called. The answer is one of the three strings `BAD`,
`VAR` and `LIT`.

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
[the symbol type of DATATYPE](r:bif.datatype.symbol); the null string, a
string with whitespace anywhere in it, even at an end, and one with any other
character, is `BAD`. The string is read as it is, without trimming. A
`name` that is an object other than a string is first replaced by the answer
of its `STRING` method, as [any string argument](r:conv.string.object) is.

```rexx
j = 3
say symbol('j') symbol('k') symbol('a_b') symbol('a!') symbol('a?') symbol('_')
say symbol('') symbol(' j') symbol('j ') symbol('a b') symbol('+5') symbol('a-b'),
    symbol('a$') symbol('"j"') symbol('é')
say symbol(.nil) symbol(.array~new) symbol(.mutablebuffer~new('j'))
```

```output
VAR LIT LIT LIT LIT LIT
BAD BAD BAD BAD BAD BAD BAD BAD BAD
BAD BAD VAR
```

r[bif.symbol.case]
Letters of `name` are translated to uppercase before the symbol is looked
up, as the symbols of a program are, so `name` names the same variable in
either case.

```rexx
j = 3
say symbol('j') symbol('J') symbol('k') symbol('K')
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
A symbol with a period in it that does not end at the period, and that does
not start with one, is a compound symbol: the part before the first period is
its stem, and the parts after it are its tail. Every part of the tail that
names a variable holding a value is replaced by that value, as in a
program, and the answer is `VAR` when the compound variable so named holds a
value, and `LIT` when it does not. The value of a tail part can be any
string, whitespace included; it is not itself checked to be a symbol. An
unset part of the tail stands for its own name in uppercase.

```rexx
j = 3
a.3 = 'x'
say symbol('a.3') symbol('a.j') symbol('A.J') symbol('a.4') symbol('a.k')
drop j
say symbol('j') symbol('a.j') symbol('a.3')
t = 'x y'
c.t = 1
say symbol('c.t') symbol('c.x y')
```

```output
VAR VAR VAR LIT LIT
LIT LIT VAR
VAR BAD
```

r[bif.symbol.stem]
A symbol that ends at its first period is a stem. A stem that has never
been touched is `LIT`. After an assignment to the whole stem, the stem is
`VAR`, and so is a compound variable of it that was never assigned or
dropped, because it has the value given to the stem.

```rexx
say symbol('stem.') symbol('stem.zzz')
stem. = 0
say symbol('stem.') symbol('stem.zzz') symbol('other.') symbol('other.1')
```

```output
LIT LIT
VAR VAR LIT LIT
```

> **Note.** This book does not specify the answer for a stem after one of its
> compound variables is assigned or after the stem is dropped.

## Scope

r[bif.symbol.scope]
SYMBOL looks at the variables that the routine or method running it can
see. A routine called with a PROCEDURE instruction sees none of its caller's
variables, except the ones it EXPOSEs; in a method, an attribute is seen as
a variable once it is exposed.

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
