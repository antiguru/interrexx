---
element: VAR
kind: builtin
syntax: "VAR(name)"
rexxref: "Functions > Built-in Functions > VAR"
---

# VAR

VAR tells whether a string names a variable that holds a value where VAR is
called. The answer is `1` or `0`. It is the question that
[SYMBOL](symbol.md) answers with `VAR` or not, asked in a form that has only
two answers.

## Arguments

r[bif.var.args]
VAR takes one argument, `name`, which is a [string](r:conv.string.null), and
nothing else. The argument count is checked by the [builtin call
rules](r:bif.index.call.count): one argument is required and one is the most.
VAR does not run as a String method: sending the message `VAR` to a string
raises `SYNTAX 97.1`. An object other than a string is replaced by a string,
as [a string argument](r:conv.string.object) says.

```rexx
x = 1
say var('x') var('x', )
signal on syntax name a
say var()
a: say condition('o')~code
signal on syntax name b
say var('x', 'y')
b: say condition('o')~code
signal on syntax name c
say 'x'~var
c: say condition('o')~code
say var(.nil) var(.array~new) var(.mutablebuffer~new('x'))
```

```output
1 1
40.3
40.4
97.1
0 0 1
```

> **Note.** The ANSI draft has no VAR function; ooRexx adds it. The call is the
> one that the reference manual describes.

## The result

r[bif.var.result]
The answer is `1` when the string is a symbol that names a variable holding a
value, and `0` for every other string: one that is not a symbol, a constant
symbol, and a variable that holds no value. It is `1` for exactly the strings
for which [SYMBOL](r:bif.symbol.result) answers `VAR`, with the same reading of
the string, so that SYMBOL's rules for the [case](r:bif.symbol.case) of the
name, for the [simple](r:bif.symbol.simple), [compound](r:bif.symbol.compound)
and [stem](r:bif.symbol.stem) symbols, and for the [scope](r:bif.symbol.scope)
hold for VAR as well. A variable holding the null string holds a value, and so
does one holding an object that is not a string.

```rexx
j = 3
drop a.3
v = ''
say var('j') var(j) var('J') var('a.j') var(2) var('*') var('v')
say var('') var('j ') var(' j') var('a b') var('3') var('.nil') var('.x')
a.3 = 1
say var('a.j') var('a.3') var('A.J') var('a.4') var('a')
x = .array~new
say var('x') var(copies('a', 250)) var(copies('a', 251))
```

```output
1 0 1 0 0 0 1
0 0 0 0 0 0 0
1 1 1 0 0
1 0 0
```

r[bif.var.same]
For every string, VAR answers `1` when SYMBOL, called at the same place, would
answer `VAR`, and `0` when it would answer `LIT` or `BAD`. Calling one makes
the same references to a stem as calling the other.

```rexx
a = 1
b.1 = 1
c. = 0
drop d.1
e.1 = 1
drop e.
list = 'a b.1 b. b.2 c. c.zz d.1 d. e. e.1 f. f.1 f. 1 .x "a" a$'
do n over list~makearray(' ')
  if (var(n) == (symbol(n) == 'VAR')) then iterate
  say 'differs' n
end
say 'done'
```

```output
done
```

## Variables that do not hold a value

r[bif.var.dropped]
A variable that has been dropped holds no value, so the answer for it is `0`,
until it is assigned again. A compound variable of a stem that has been
assigned as a whole holds the value of the stem unless it was dropped, and a
compound variable of a dropped stem holds none. The answer for a stem is `1`
once the program has referred to it, as for [SYMBOL](r:bif.symbol.stem), and
stays `1` when the stem is dropped.

```rexx
say var('fresh.') var('fresh.1') var('fresh.')
a = 1
drop a
say var('a')
a = 2
say var('a')
e. = 'dflt'
drop e.1
say var('e.1') var('e.2') var('e.zz') var('e.')
drop e.
say var('e.') var('e.1') var('e.2')
```

```output
0 0 1
0
1
0 1 1 1
1 0 0
```

r[bif.var.compound]
The value of a tail variable is used as it is held, without a change to
uppercase: a tail variable that holds `abc` makes the compound variable one
whose tail is `abc`, and not the one whose tail is `ABC`.

```rexx
t = 'abc'
x.t = 2
w = 'x y'
c.w = 1
say var('x.abc') var('x.ABC') var('x.t') var('X.T') var('c.w') var('c.x y')
```

```output
0 0 1 1 1 0
```

r[bif.var.scope]
VAR sees the variables of the routine or method running it. In a routine with
PROCEDURE the caller's variables are not seen unless exposed, and in a method
an object variable is seen once it is exposed and holds a value.

```rexx
g = 1
call hide
call show
o = .o~new
say o~plain o~exposing
exit
hide: procedure
say var('g')
return
show: procedure expose g
say var('g')
return
::class o
::attribute at
::method plain
return var('at')
::method exposing
expose at un
at = 1
return var('at') var('un')
```

```output
0
1
0 1 0
```

r[bif.var.result-var]
The variable `RESULT` is a variable that holds a value after a CALL of a
routine that returned a value, and is not after a CALL of one that returned
none.

```rexx
call none
say var('result')
call one
say var('result')
call none
say var('result')
exit
none: return
one: return 1
```

```output
0
1
0
```
