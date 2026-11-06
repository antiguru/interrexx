---
element: RXFUNCDROP
kind: builtin
syntax: "RXFUNCDROP(name)"
rexxref: "Functions > Built-in Functions > RXFUNCDROP"
---

# RXFUNCDROP

RXFUNCDROP removes one registration of a function name that
[RXFUNCADD](rxfuncadd.md) made. It answers `0` if it removed a registration
and `1` if it did not. [RXFUNCQUERY](rxfuncquery.md) asks
whether a name is available afterwards.

> **Note.** The examples drop every registration they make, so they leave
> nothing behind and do not depend on registrations of other names. A
> registration can be removed only by the program that made it.
> Making a function of the library `rxmath` available is not a registration,
> as [RXFUNCADD](rxfuncadd.md) explains, and the example that does it picks
> a function that no other example on this page uses.

## Arguments

r[bif.rxfuncdrop.args]
`name` is a required [string](r:conv.string.null). RXFUNCDROP does not run as
a String method: sending the message `RXFUNCDROP` to a string raises `SYNTAX
97.1`. The [call rules](r:bif.index.call.count) for the number of arguments
hold: a call without an argument raises `SYNTAX 40.3`, and a call with a
second argument raises `SYNTAX 40.4`. A trailing omitted place adds nothing to
the count, so `RXFUNCDROP(,)` is a call without an argument.

```rexx
signal on syntax name a
say rxfuncdrop()
a: say condition('o')~code
signal on syntax name b
say rxfuncdrop(,)
b: say condition('o')~code
signal on syntax name c
say rxfuncdrop('spec_rxfuncdrop_args', 'x')
c: say condition('o')~code
signal on syntax name d
say 'spec_rxfuncdrop_args'~rxfuncdrop
d: say condition('o')~code
```

```output
40.3
40.3
40.4
97.1
```

## The result

r[bif.rxfuncdrop.registered]
A name that this program had [RXFUNCADD](rxfuncadd.md) register for a [library
it could not find](r:bif.rxfuncadd.unknown-library) is removed, and the answer
is `0`. The
name is compared without regard to case, as
[RXFUNCQUERY](rxfuncquery.md) does. Afterwards the name is no longer
available, and a call of it raises `SYNTAX 43.1` as that of any unknown
function does. The answer is written as the digit `0` or `1`. This rule and
[the next but one](r:bif.rxfuncdrop.one-at-a-time) hold when no other program
has called RXFUNCADD for the name. When a second program has also called it,
what the program that registered the name first gets from its calls of
RXFUNCDROP, and whether the name stays available, is not specified; the second
program's side is in [RXFUNCADD's repeat rule](r:bif.rxfuncadd.repeat).

```rexx
call rxfuncadd 'spec_rxfuncdrop_one', 'no_such_library_spec', 'p'
say rxfuncquery('spec_rxfuncdrop_one')
say rxfuncdrop('SPEC_RXFUNCDROP_ONE')
say rxfuncquery('spec_rxfuncdrop_one')
signal on syntax name a
say spec_rxfuncdrop_one(1)
a: say condition('o')~code
```

```output
0
0
1
43.1
```

r[bif.rxfuncdrop.unregistered]
A name that has no registration to remove answers `1` and changes nothing.
That covers a name that was never registered, a name whose registration
an earlier call has removed, and a name that only another program registered:
the answer is `1` and the name stays available, whether or not that program
is still running.

```rexx
say rxfuncdrop('spec_rxfuncdrop_never')
call rxfuncadd 'spec_rxfuncdrop_twice', 'no_such_library_spec', 'p'
say rxfuncdrop('spec_rxfuncdrop_twice')
say rxfuncdrop('spec_rxfuncdrop_twice')
say rxfuncdrop('')
```

```output
1
0
1
1
```

r[bif.rxfuncdrop.one-at-a-time]
A call removes one registration. A name registered more than once, by calls
of [RXFUNCADD](rxfuncadd.md) that answered `1` for an unknown library, stays
available until as many calls of RXFUNCDROP as there were registrations have
removed them.

```rexx
call rxfuncadd 'spec_rxfuncdrop_many', 'no_such_library_spec', 'a'
call rxfuncadd 'spec_rxfuncdrop_many', 'no_such_library_spec', 'b'
call rxfuncadd 'spec_rxfuncdrop_many', 'other_library_spec', 'a'
do 4
  say rxfuncdrop('spec_rxfuncdrop_many') rxfuncquery('spec_rxfuncdrop_many')
end
```

```output
0 0
0 0
0 1
1 1
```

r[bif.rxfuncdrop.provided]
A name that is available without a registration of its own is not removed:
the answer is `1` and the function stays available. That covers the functions
the interpreter provides from the start, such as `SysSleep`, and the
functions of a library that is loaded, by [RXFUNCADD](rxfuncadd.md) or by a
`::requires` directive with the `LIBRARY` keyword, which stay callable.

```rexx
say rxfuncdrop('SysSleep') rxfuncquery('SysSleep')
call rxfuncadd 'RxCalcSqrt', 'rxmath'
say rxfuncdrop('RxCalcSqrt') rxfuncquery('RxCalcSqrt')
say rxcalcsqrt(16)
```

```output
1 0
1 0
4
```

```rexx
say rxfuncdrop('RxCalcLog') rxfuncquery('RxCalcLog')
say rxcalclog(1)
::requires 'rxmath' LIBRARY
```

```output
1 0
0
```
