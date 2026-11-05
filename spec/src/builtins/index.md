---
element: "Built-in functions"
kind: chapter
ansi: "9.2.1"
rexxref: "Functions > Built-in Functions"
---

# Built-in functions

This chapter has a page for each builtin function, and each page specifies
the function call. The rules here hold for every builtin call; the pages
link to them, and to the [conventions](../conventions.md), instead of
restating them.

A page's syntax line names the arguments in order. A name in square
brackets is an optional argument; every other argument is required. The
number of names on the line is the most arguments the builtin takes, and
the number of required ones is the fewest.

r[bif.call.count]
The argument count of a call is the position of the last argument in it
that is not [omitted](r:conv.arg.omitted). Commas after that argument are
ignored, however many there are, and a call whose arguments are all omitted
has a count of 0.

```rexx
say substr('abc', 2, , ) word('a b', 2, , , )
```

```output
bc b
```

r[bif.call.too-few]
A call whose argument count is less than the number of required arguments
raises `SYNTAX 40.3`.

```rexx
signal on syntax name a
say substr('abc')
a: say condition('o')~code
signal on syntax name b
say substr('abc', )
b: say condition('o')~code
signal on syntax name c
say word(, , )
c: say condition('o')~code
```

```output
40.3
40.3
40.3
```

r[bif.call.too-many]
A call whose argument count is greater than the number of arguments on the
syntax line raises `SYNTAX 40.4`, whatever the values of the arguments,
and even when the arguments before the last are omitted.

```rexx
signal on syntax name a
say word('a b', 1, 2)
a: say condition('o')~code
signal on syntax name b
say substr(, , , , 1)
b: say condition('o')~code
```

```output
40.4
40.4
```

r[bif.call.missing]
A required argument that is omitted, in a call whose argument count reaches
past it, raises `SYNTAX 40.5`.

```rexx
signal on syntax name a
say substr(, 1)
a: say condition('o')~code
signal on syntax name b
say wordpos('a', , 1)
b: say condition('o')~code
```

```output
40.5
40.5
```

r[bif.call.order]
The [too-few](r:bif.call.too-few) and [too-many](r:bif.call.too-many) checks
come before every other check of a call. The check for an omitted
[required argument](r:bif.call.missing) comes next, before any argument's
value is checked; the value checks then follow
[their own order](r:conv.check.order).

```rexx
signal on syntax name a
say word(, 'x', 2)
a: say condition('o')~code
signal on syntax name b
say substr(, 'x')
b: say condition('o')~code
signal on syntax name c
say substr('abc', , , '**')
c: say condition('o')~code
```

```output
40.4
40.5
40.5
```
