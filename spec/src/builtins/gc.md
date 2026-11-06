---
element: GC
kind: builtin
syntax: "GC([force])"
rexxref: "Functions > Built-in Functions > GC"
---

# GC

GC asks the interpreter to reclaim the storage of objects that the program no
longer reaches, and answers whether it did. A program has no need to call it:
the interpreter reclaims storage on its own, and a request costs time.

This book does not specify when storage is reclaimed, in what order, or when
the `UNINIT` method of an object runs, whether or not GC is called. A program
cannot rely on any of it, and the examples show none of it. What is specified
is the argument and the answer.

## Arguments

r[bif.gc.args]
`force` is optional. GC does not run as a String method: sending the message
`GC` to a string raises `SYNTAX 97.1`. The [call
rules](r:bif.index.call.count) for the number of arguments hold: a second
argument raises `SYNTAX 40.4`, even when the first is omitted. A trailing
omitted place adds nothing to the count, so `GC(,)` is a call without an
argument.

```rexx
signal on syntax name a
say gc(, 'force')
a: say condition('o')~code
signal on syntax name b
say gc('force', 'force')
b: say condition('o')~code
signal on syntax name c
say 'force'~gc
c: say condition('o')~code
signal off syntax
say gc(,) == gc()
```

```output
40.4
40.4
97.1
1
```

r[bif.gc.force]
`force` is [converted to a string](r:conv.string.object) and must begin with
the letter `F`, in either case: `force`, `Force`, `FORCE` and `f` are
accepted, and so is any other string that starts so. Any other value raises
`SYNTAX 40.904`: a string that starts with another character, including a
blank, the null string, and the digits `0` and `1`, and `.nil`. The argument
is checked after the count of arguments.

```rexx
signal on syntax name a
say gc(1)
a: say condition('o')~code
signal on syntax name b
say gc('')
b: say condition('o')~code
signal on syntax name c
say gc(' force')
c: say condition('o')~code
signal on syntax name d
say gc(.nil)
d: say condition('o')~code
signal on syntax name e
say gc(.false)
e: say condition('o')~code
signal on syntax name f
say gc('x', 'y')
f: say condition('o')~code
signal off syntax
say gc('force') gc('Force') gc('F') gc('f') gc('fx') gc(.mutablebuffer~new('f'))
```

```output
40.904
40.904
40.904
40.904
40.904
40.4
1 1 1 1 1 1
```

## The result

r[bif.gc.answer]
GC answers `1` when it reclaimed storage and `0` when it did not. A call with
a valid `force` always reclaims, and answers `1`. A call without `force`
reclaims only on an interpreter built for debugging, which is the case when
`.RexxInfo~debug` is true, and otherwise answers `0`; in both cases the answer
is the value of `.RexxInfo~debug`. The answer is the digit alone, `1` or `0`,
so it can be used as a condition.

```rexx
say (gc() == .RexxInfo~debug) gc('force') datatype(gc(), 'O')
call gc
r1 = result
call gc 'force'
say (r1 == .RexxInfo~debug) result
```

```output
1 1 1
1 1
```

r[bif.gc.keeps]
A call changes nothing that the program can see: the variables, stems and
objects it holds have the values they had, and the call has no other
effect.

```rexx
a = 'kept'
stem. = 'dflt'
stem.1 = 'one'
o = .array~of('x', 'y')
call gc 'force'
say a stem.1 stem.2 o~items o[2]
```

```output
kept one dflt 2 y
```
