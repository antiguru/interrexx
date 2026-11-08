---
element: ADDRESS
kind: builtin
syntax: "ADDRESS()"
ansi: "9.5.1"
rexxref: "Functions > Built-in Functions > ADDRESS"
---

# ADDRESS

ADDRESS answers the name of the environment to which the program sends the
commands it does not recognize as anything else: the name that the ADDRESS
instruction last set, or the name the program started with.

## Arguments

r[bif.address.args]
ADDRESS takes no argument. The argument count is checked by the [builtin call
rules](r:bif.index.call.count): none is required and none is the most, so a
call with any argument, whatever it is, raises `SYNTAX 40.4`, and places that
are only [omitted](r:conv.arg.omitted) add nothing. ADDRESS does not run as a
String method: sending the message `ADDRESS` to a string raises `SYNTAX 97.1`.

```rexx
d = address()
say address(,) == d
signal on syntax name a
say address('I')
a: say condition('o')~code
signal on syntax name b
say address(, 'O')
b: say condition('o')~code
signal on syntax name c
say 'x'~address
c: say condition('o')~code
```

```output
1
40.4
40.4
97.1
```

> **Note.** The ANSI draft (9.5.1) lets ADDRESS take an option that asks for
> the targets of command input and output as well. ooRexx does not, and
> neither does this book.

## The result

r[bif.address.initial]
Which environment a program starts with is not specified: it is the one that
the platform and the way the program was started choose, and its name is
different on different systems. What is fixed is that the [ADDRESS
instruction](r:instr.address.form.bare) with no operand, given before any
other ADDRESS instruction, leaves the environment as it was: the environment
the program started with is also the one that the instruction would swap back
to.

```rexx
d = address()
address
say address() == d
address
say address() == d
```

```output
1
1
```

r[bif.address.name]
After `ADDRESS name`, the result is the name written in uppercase when it is a
symbol, and exactly as written when it is a string, the case, leading and
trailing blanks included. After `ADDRESS VALUE expression`, or an expression
in parentheses, it is the value of the expression, also when that is the null
string; a value that is not a string is replaced by a string, as [a string
argument](r:conv.string.object) says. The name is not checked and need not
name any environment the program can use.

```rexx
x = 'dyn Name '
address foo
say '['address()']'
address 'Mixed Case '
say '['address()']'
address value x
say '['address()']'
address (x)
say '['address()']'
address ''
say '['address()']' length(address())
address foo.bar
say '['address()']'
```

```output
[FOO]
[Mixed Case ]
[dyn Name ]
[dyn Name ]
[] 0
[FOO.BAR]
```

> **Note.** The reference manual says trailing whitespace is removed from the
> result. Neither interrexx nor ooRexx does so, and this rule matches them.

r[bif.address.swap]
The [ADDRESS instruction](r:instr.address.form.bare) with no operand makes the
previous environment the current one and the current one the previous one, so
a second such instruction undoes the first. An instruction that sets a name
makes the environment that was current the previous one, also when the name is
the same as the current one, so after that the instruction with no operand
changes nothing.

```rexx
address first
address second
say address()
address
say address()
address
say address()
address second
address second
address
say address()
```

```output
SECOND
FIRST
SECOND
SECOND
```

r[bif.address.command]
`ADDRESS name command` sends one command to the named environment and changes
neither the current environment nor the previous one, so the result and what
the instruction with no operand swaps back to are as they were before it.

```rexx
d = address()
address first
address system 'exit 0'
say address() rc
address
say address() == d
```

```output
FIRST 0
1
```

## Scope

r[bif.address.scope.routine]
An internal routine, one that the program calls by its label, starts with the
caller's current environment and its previous one, and when it returns, the
caller has its own back: a setting made inside the routine, by [ADDRESS
instructions](../instructions/address.md) or in a routine it called, is lost
on return. This holds for a routine with [PROCEDURE](r:instr.procedure.scope)
and for one without it.

```rexx
address outer
call plain
say address()
call hidden
say address()
call nested
say address()
exit
plain:
say 'plain' address()
address inner
say 'plain' address()
return
hidden: procedure
say 'hidden' address()
address deeper
say 'hidden' address()
return
nested:
call plain
say 'nested' address()
return
```

```output
plain OUTER
plain INNER
OUTER
hidden OUTER
hidden DEEPER
OUTER
plain OUTER
plain INNER
nested OUTER
OUTER
```

r[bif.address.scope.method]
A method, and a `::ROUTINE`, starts with the environment the program started
with, not the caller's, and as for an internal routine, a setting made in it
is lost when it returns. Both have the start environment as the previous one
too, so the instruction with no operand there swaps back to it.

```rexx
d = address()
address outer
o = .o~new
say (o~start == d) (o~set == 'INNER') address()
say (o~start == d) address()
address one
say (word(rr(), 1) == d) (word(rr(), 2) == d) (word(rr(), 3) == d) address()
exit
::routine rr
a = address()
address
b = address()
address x
return a b address()
::class o
::method start
return address()
::method set
address inner
return address()
```

```output
1 1 OUTER
1 OUTER
1 1 0 ONE
```

r[bif.address.scope.interpret]
Code run by [INTERPRET](r:instr.interpret.scope) shares the environment of the
code that runs it: a setting made there is still in effect after the INTERPRET
ends.

```rexx
address outer
interpret 'address inner'
say address()
interpret 'say address()'
```

```output
INNER
INNER
```
