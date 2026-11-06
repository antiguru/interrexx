---
element: RXFUNCQUERY
kind: builtin
syntax: "RXFUNCQUERY(name)"
rexxref: "Functions > Built-in Functions > RXFUNCQUERY"
---

# RXFUNCQUERY

RXFUNCQUERY asks whether a function of the given name is available to the
program without a library having to be asked for it first. It answers `0`
for an available function and `1` otherwise, and it changes nothing. [RXFUNCADD](rxfuncadd.md) makes functions of a library available, and
[RXFUNCDROP](rxfuncdrop.md) removes a registration.

> **Note.** The examples register nothing that they do not drop, so they do
> not depend on registrations of other names. Making a function
> of the library `rxmath` available is not a registration that can be
> dropped, as [RXFUNCADD](rxfuncadd.md) explains: it lasts until the
> program ends, and the example that does it picks a function that no other
> example on this page uses.

## Arguments

r[bif.rxfuncquery.args]
`name` is a required [string](r:conv.string.null). RXFUNCQUERY does not run
as a String method: sending the message `RXFUNCQUERY` to a string raises
`SYNTAX 97.1`. The [call rules](r:bif.index.call.count) for the number of
arguments hold: a call without an argument raises `SYNTAX 40.3`, and a call
with a second argument raises `SYNTAX 40.4`. A trailing omitted place adds
nothing to the count, so `RXFUNCQUERY(,)` is a call without an argument.

```rexx
signal on syntax name a
say rxfuncquery()
a: say condition('o')~code
signal on syntax name b
say rxfuncquery(,)
b: say condition('o')~code
signal on syntax name c
say rxfuncquery('SysSleep', 'x')
c: say condition('o')~code
signal on syntax name d
say 'SysSleep'~rxfuncquery
d: say condition('o')~code
```

```output
40.3
40.3
40.4
97.1
```

## The result

r[bif.rxfuncquery.answer]
The answer is `0` for a name that is available: a function the interpreter
provides from the start, such as `SysSleep`, a function of a library that
[RXFUNCADD](rxfuncadd.md) or a `::requires` directive with the `LIBRARY`
keyword has loaded, and a name that [RXFUNCADD](rxfuncadd.md) has registered.
The answer is `1` for every other name, such as a name that was never
registered. The null string is a name like any other. The answer is written as
the digit `0` or `1`.

```rexx
say rxfuncquery('SysSleep')
say rxfuncquery('spec_rxfuncquery_never')
say rxfuncquery('')
say rxfuncadd('', 'no_such_library_spec')
say rxfuncquery('')
say rxfuncdrop('') rxfuncquery('')
say rxfuncquery('RxCalcTan')
say rxfuncadd('RxCalcTan', 'rxmath')
say rxfuncquery('RxCalcTan')
say rxfuncquery('RxCalcArcTan')
```

```output
0
1
1
1
0
0 1
1
0
0
0
```

r[bif.rxfuncquery.case]
The name is compared without regard to case, and it is compared as it is
written: blanks around it are part of it.

```rexx
say rxfuncquery('SYSSLEEP') rxfuncquery('syssleep') rxfuncquery('SysSleep')
say rxfuncquery(' SysSleep') rxfuncquery('SysSleep ')
```

```output
0 0 0
1 1
```

r[bif.rxfuncquery.registered]
A name that [RXFUNCADD](rxfuncadd.md) registered for a
[library it could not find](r:bif.rxfuncadd.unknown-library) is available to
RXFUNCQUERY, in any case, until [RXFUNCDROP](rxfuncdrop.md) removes the
registration, although a call of it raises `SYNTAX 43.1`. A name that another
program registered is available in the same way, whether or not that program
is still running.

```rexx
say rxfuncadd('spec_rxfuncquery_reg', 'no_such_library_spec', 'entry')
say rxfuncquery('spec_rxfuncquery_reg') rxfuncquery('SPEC_RXFUNCQUERY_REG')
say rxfuncdrop('Spec_RxFuncQuery_Reg')
say rxfuncquery('spec_rxfuncquery_reg')
```

```output
1
0 0
0
1
```
